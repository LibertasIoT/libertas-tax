#[allow(unused_imports)]
use super::*;
pub(crate) fn retirement_paper_issue(
    document: &crate::RetirementDocument,
) -> Option<ValidationIssue> {
    match document {
        crate::RetirementDocument::R1099(form) => crate::retirement::paper_source_issue(form)
            .or_else(|| {
                (!crate::federal_rules::retirement_printed_amounts_valid(
                    [
                        form.gross,
                        form.withholding.unwrap_or(0),
                        form.taxable.unwrap_or(0),
                        form.box_3.unwrap_or(0),
                        form.box_6.unwrap_or(0),
                        form.box_8a.unwrap_or(0),
                    ],
                    form.box_8b_percent,
                    form.box_9a_percent,
                    form.gross,
                    form.box_10.unwrap_or(0),
                ))
                .then(retirement_printed_amount_issue)
            }),
        crate::RetirementDocument::Rrb1099R(form) => rrb_paper_issue(form),
        crate::RetirementDocument::Ssa1099(form) => ssa_paper_issue(form),
        crate::RetirementDocument::Rrb1099(form) => form.issue().or_else(|| {
            (!crate::federal_rules::social_printed_amounts_valid(form.benefits, form.withholding))
                .then(retirement_printed_amount_issue)
        }),
    }
}

pub(super) fn retirement_printed_amount_issue() -> ValidationIssue {
    ValidationIssue::new("ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26")
}

// Validate established answers through their original native owners.
pub(crate) fn special_retirement_answer_issue(income: &FederalIncome) -> Option<ValidationIssue> {
    let source = match income {
        FederalIncome::TrumpDeath { data }
            if matches!(data.code, Some(DistributionCode::TrumpDeath { basis: None, .. })) =>
        {
            return Some(
                ValidationIssue::new("R_TRUMP_BASIS_NEEDED").at_field("basis").in_field("code"),
            );
        }
        FederalIncome::Roth { data }
        | FederalIncome::TrumpDeath { data }
        | FederalIncome::TrumpAbleRollover { data } => data,
        _ => return None,
    };
    crate::federal_rules::validate_income_amount_issue(income)
        .err()
        .map(|(issue, _)| issue)
        .or_else(|| {
            let code = source.code.as_ref()?;
            if let DistributionCode::RothIra { roth, .. } = code {
                if let Some(issue) = crate::retirement::roth_other_boxes_issue(source) {
                    return Some(issue);
                }
                if source.box7 != 0 && !DistributionCodes::leads_to_roth(source.box7, roth) {
                    return Some(
                        ValidationIssue::new("R_ROTH_CODE_NOT_BOX7")
                            .at_field("roth")
                            .in_field("code"),
                    );
                }
            }
            (source.box7 != 0
                && !DistributionCodes::leads_to(source.box7, source.ira, source.trump, code))
            .then(|| ValidationIssue::new("R_BOX7_TREATMENT").at_field("code"))
        })
}

pub(crate) fn retirement_answer_issue(data: &crate::RetirementSource) -> Option<ValidationIssue> {
    crate::federal_rules::validate_income_amount_issue(&FederalIncome::Retirement {
        data: data.clone(),
    })
    .err()
    .map(|(issue, _)| issue)
}

// The annuitant age of a Simplified Method worksheet that takes it from the owner's date of birth
// follows that date and the starting date on every accepted save, so neither edit leaves it stale.
pub(super) fn follow_annuity_age(d: &mut FederalDraft) {
    let births = [Owner::Taxpayer, Owner::Spouse]
        .map(|owner| d.people.as_ref()?.person(owner).map(|person| person.birth_date));
    for entry in &mut d.income {
        let birth =
            entry.owner.tax_owner().and_then(|owner| births[usize::from(owner == Owner::Spouse)]);
        let FederalIncome::Retirement { data } = &mut entry.income else { continue };
        if let Some(basis) = simplified_worksheets(data) {
            basis.follow_birth(birth);
        }
    }
}
// The Simplified Method worksheets of one accepted retirement source.
fn simplified_worksheets(source: &mut crate::RetirementSource) -> Option<&mut crate::PensionBasis> {
    match source {
        crate::RetirementSource::R1099(source) => match &mut source.taxable_method {
            Some(crate::TaxableMethod::Simplified { data }) => Some(data),
            _ => None,
        },
        crate::RetirementSource::Rrb1099R(source) => match &mut source.recipient_kind {
            Some(
                crate::RailroadRecipient::Employee {
                    recovery: crate::RailroadRecovery::SimplifiedMethod { data },
                }
                | crate::RailroadRecipient::Survivor {
                    recovery: crate::RailroadRecovery::SimplifiedMethod { data },
                    ..
                },
            ) => Some(data),
            _ => None,
        },
    }
}
// The Simplified Method age comes from the owner's date of birth: refused while that date cannot give it.
pub(super) fn r1099_birth_issue(
    v: &crate::R1099Answered,
    birth: Option<u32>,
) -> Option<ValidationIssue> {
    match &v.taxable_method {
        Some(crate::TaxableMethod::Simplified { data }) if data.lacks_birth(birth) => Some(
            ValidationIssue::new("R_ANNUITANT_BIRTH_NEEDED")
                .at_field("age")
                .in_field("duration")
                .in_field("history")
                .in_field("data")
                .in_field("taxable_method"),
        ),
        _ => None,
    }
}
// A form with the answers whose original owners are outside the physical income row: the
// premiums the screening records hold and the repayment its owner's IRA basis record keeps.
pub(super) fn retirement_answered(
    draft: &FederalDraft,
    id: i64,
    owner: Owner,
    source: &R1099Source,
) -> crate::R1099Answered {
    let premiums: Vec<_> = draft
        .screening
        .iter()
        .flat_map(|s| &s.public_safety)
        .filter(|record| record.income_id == id)
        .collect();
    let mut source = source.clone();
    if let Some(
        DistributionCode::Early { exception, .. }
        | DistributionCode::EarlySimple {
            what_happened: crate::SimpleDisposition::Kept { exception },
        }
        | DistributionCode::ProhibitedTransaction { exception },
    ) = &mut source.code
        && let EarlyException::Applies { repaid, .. } = exception
    {
        *repaid = draft
            .adjustments
            .iter()
            .flat_map(|a| &a.ira_basis)
            .filter(|basis| basis.owner == owner)
            .flat_map(|basis| &basis.special_repayments)
            .find(|record| record.income == id)
            .map(|record| crate::ExceptionRepayment {
                payments: record.payments.clone(),
                confirmed: record.confirmed,
            });
    }
    crate::R1099Answered::new(source, crate::public_safety_source::paid_from_records(&premiums))
}
pub(super) fn retirement_personal_choices(
    answered: &crate::R1099Answered,
) -> (Vec<crate::PublicSafetyPaid>, Option<crate::ExceptionRepayment>) {
    let repaid = match &answered.code {
        Some(
            DistributionCode::Early { exception: EarlyException::Applies { repaid, .. }, .. }
            | DistributionCode::EarlySimple {
                what_happened:
                    crate::SimpleDisposition::Kept { exception: EarlyException::Applies { repaid, .. } },
            }
            | DistributionCode::ProhibitedTransaction {
                exception: EarlyException::Applies { repaid, .. },
            },
        ) => repaid.clone(),
        _ => None,
    };
    (answered.public_safety().to_vec(), repaid)
}
// The IRA basis worksheet taxes every distribution of an owner's own IRA once that owner's
// Form 8606 is on the return; without it the box 2b method alone decides.
pub(super) fn retirement_ira_basis(entries: &mut [FederalIncomeEntry], form8606_owners: &[Owner]) {
    for entry in entries {
        let owner = entry.owner;
        let FederalIncome::Retirement { data: crate::RetirementSource::R1099(source) } =
            &mut entry.income
        else {
            continue;
        };
        source.ira_basis = false;
        let record = FederalRetirement::from(&*source);
        source.ira_basis = form8606_owners.contains(&owner.owned())
            && record.kind == RetirementKind::Ira
            && !record.outside_form_8606();
    }
}
pub(super) fn retirement_personal_records(
    choices: impl Iterator<
        Item = ((i64, Owner), (Vec<crate::PublicSafetyPaid>, Option<crate::ExceptionRepayment>)),
    >,
) -> (Vec<crate::PublicSafetyPremium>, Vec<(Owner, crate::IraRepayment)>) {
    let mut premiums = Vec::new();
    let mut repayments = Vec::new();
    for ((id, owner), (paid, repaid)) in choices {
        premiums.extend(crate::public_safety_source::premium_records(&paid, id));
        if let Some(repaid) = repaid {
            repayments.push((
                owner,
                crate::IraRepayment {
                    income: id,
                    payments: repaid.payments,
                    confirmed: repaid.confirmed,
                },
            ));
        }
    }
    (premiums, repayments)
}
pub(super) fn apply_retirement_personal_records(
    draft: &mut FederalDraft,
    sources: &alloc::collections::BTreeSet<i64>,
    premiums: Vec<crate::PublicSafetyPremium>,
    repayments: Vec<(Owner, crate::IraRepayment)>,
) {
    let present: alloc::collections::BTreeSet<_> =
        draft.income.iter().map(|entry| entry.id).collect();
    if let Some(screening) = draft.screening.as_mut() {
        screening.public_safety.retain(|record| {
            present.contains(&record.income_id) && !sources.contains(&record.income_id)
        });
        screening.public_safety.extend(premiums);
    }
    if let Some(adjustments) = draft.adjustments.as_mut() {
        for basis in &mut adjustments.ira_basis {
            basis.special_repayments.retain(|record| {
                present.contains(&record.income) && !sources.contains(&record.income)
            });
            basis.special_repayments.extend(
                repayments
                    .iter()
                    .filter(|(owner, _)| *owner == basis.owner)
                    .map(|(_, record)| record.clone()),
            );
        }
    }
}
// The save rules that read more than one document of the Retirement page (server only: a
// client rule sees one row at a time), in the order they are reported:
// - IRA_ROLLOVER_ONCE_A_YEAR: a person's 60-day rollovers from IRAs to IRAs are tax-free once in
//   any 1-year period across all of their IRAs (section 408(d)(3)(B)); trustee-to-trustee
//   transfers and plan rollovers are not counted.
// - R_EXCESS_DEFERRAL_ABOVE_W2_EXCESS: the excess deferrals the 1099-R rows say they returned
//   are no more than the excess the App counted from the person's W-2s (Form 1040 line 1h).
// - QCD_SPLIT_INTEREST_ABOVE_LIMIT: the one-time split-interest gifts of a person are within
//   the indexed limit (section 408(d)(8)(F)).
// - DISASTER_ABOVE_QCD_REST: a gift to charity is excluded first, so the designated qualified
//   disaster distributions come from the rest of the person's own IRA distributions.
pub(super) fn retirement_save_rules(
    draft: &FederalDraft,
    adjustment_items: &[IncomeDocumentItem],
    origins: &[(i64, usize, bool)],
    adjustment_positions: &[usize],
    question_keys: Option<&[String]>,
    mut source_issue: Option<&mut Option<(Option<i64>, ValidationIssue)>>,
) -> Result<(), SaveFailure> {
    let mut source_failure = |key, ids: Vec<i64>, issue: ValidationIssue| {
        if let Some(slot) = source_issue.as_deref_mut() {
            *slot = Some((ids.first().copied(), issue.clone()));
        }
        if let Some(keys) = question_keys
            && let Some((field, from, to)) = document_followups::question_field(&issue)
            && let Some(question_key) = ids.iter().find_map(|id| {
                let target = alloc::format!("income/{id}/{field}");
                keys.iter().find(|key| **key == target)
            })
        {
            return issue
                .map_field_prefix(from, to)
                .in_field("answer")
                .in_key("followups", question_key.as_str())
                .into();
        }
        ids.iter().find_map(|id| origins.iter().find(|(source, _, _)| source == id)).map_or_else(
            || SaveFailure::from(status(key)),
            |(_, index, _)| SaveFailure::from(submitted_data_answer_issue(issue, *index)),
        )
    };
    for entry in &draft.income {
        let (ira, gross) = match &entry.income {
            FederalIncome::Retirement { data: source } => {
                let data = &crate::FederalRetirement::from(source);
                (data.kind.ira(), data.gross)
            }
            FederalIncome::Roth { data: source } => {
                let data = &crate::RothWithdrawal::from(source);
                (true, data.gross)
            }
            _ => continue,
        };
        let records: Vec<_> = draft
            .screening
            .iter()
            .flat_map(|s| &s.public_safety)
            .filter(|record| record.income_id == entry.id)
            .collect();
        let paid = crate::public_safety_source::paid_from_records(&records);
        if let Some(issue) = crate::public_safety_source::paid_issue(&paid, ira, gross) {
            return Err(source_failure(
                issue.key,
                alloc::vec![entry.id],
                issue.map_field_prefix(&["public_safety"], &["public_safety", "rows"]),
            ));
        }
    }
    let rows = |owner: Owner| {
        draft
            .income
            .iter()
            .filter(move |e| e.owner == crate::DocumentOwner::from(owner))
            .filter_map(|e| e.income.retirement())
    };
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let rolled_row = |v: &FederalRetirement| {
            v.kind == RetirementKind::Ira
                && v.box7
                    & (DistributionCodes::CODE_G.0
                        | DistributionCodes::CODE_H.0
                        | DistributionCodes::CODE_N.0)
                    == 0
                && matches!(
                    v.ordinary_distribution,
                    Some(RetirementTreatment::Rollover {
                        destination: crate::RolloverDestination::Ira
                            | crate::RolloverDestination::IraNextYear
                            | crate::RolloverDestination::SimpleIra,
                        ..
                    })
                )
        };
        let rolled = rows(owner).filter(|v| rolled_row(v)).count();
        // All of a person's IRAs, Roth ones too, are one for the rule (section 408(d)(3)(B)).
        let rolled_roth = draft
            .income
            .iter()
            .filter(|e| e.owner == crate::DocumentOwner::from(owner))
            .filter(|e| {
                e.income.roth().is_some_and(|data| {
                    data.rolled_over > 0 && data.inherited == crate::InheritedLink::No
                })
            })
            .count();
        if rolled + rolled_roth > 1 {
            let key = if rolled_roth > 0 {
                "ROTH_ROLLOVER_ONCE_A_YEAR"
            } else {
                "IRA_ROLLOVER_ONCE_A_YEAR"
            };
            let mut ids: Vec<i64> = draft
                .income
                .iter()
                .filter(|e| e.owner == crate::DocumentOwner::from(owner))
                .filter(|e| {
                    e.income.retirement().is_some_and(|data| rolled_row(&data))
                        || e.income.roth().is_some_and(|data| {
                            data.rolled_over > 0 && data.inherited == crate::InheritedLink::No
                        })
                })
                .map(|e| e.id)
                .collect();
            ids.rotate_left(1);
            return Err(source_failure(key, ids, ValidationIssue::new(key).at_field("code")));
        }
        // A Roth repayment is figured on the owner's Roth history (the 2026 Line 25c Worksheet).
        if draft.income.iter().any(|e| {
            e.owner == crate::DocumentOwner::from(owner)
                && e.income.roth().is_some_and(|data| data.repaid.is_some())
        }) && !draft
            .adjustments
            .iter()
            .flat_map(|a| &a.roth_history)
            .any(|history| history.owner == owner)
        {
            let key = "R_ROTH_REPAYMENT_NEEDS_HISTORY";
            let ids = draft
                .income
                .iter()
                .filter(|e| e.owner == crate::DocumentOwner::from(owner))
                .filter_map(|e| {
                    e.income.roth().is_some_and(|data| data.repaid.is_some()).then_some(e.id)
                })
                .collect();
            return Err(source_failure(key, ids, ValidationIssue::new(key).at_field("code")));
        }
        let returned: i64 = rows(owner)
            .map(|v| match v.ordinary_distribution {
                Some(RetirementTreatment::Corrective { excess_deferral }) if !v.kind.ira() => {
                    excess_deferral
                }
                _ => 0,
            })
            .sum();
        let person = draft.people.as_ref().and_then(|p| p.person(owner));
        let mut returned_cap = None;
        if returned > 0
            && person.is_none_or(|person| {
                let cap =
                    crate::elective_deferrals::excess(draft, owner, person.birth_date).taxable;
                returned_cap = Some(cap);
                returned > cap
            })
        {
            let key = "R_EXCESS_DEFERRAL_ABOVE_W2_EXCESS";
            let mut prefix = 0_i64;
            let ids = draft
                .income
                .iter()
                .filter(|e| e.owner == crate::DocumentOwner::from(owner))
                .filter_map(|e| {
                    if let Some(data) = e.income.retirement()
                        && !data.kind.ira()
                        && let Some(RetirementTreatment::Corrective { excess_deferral }) =
                            data.ordinary_distribution
                    {
                        let before = prefix;
                        prefix += excess_deferral;
                        let contributes_to_failure = match returned_cap {
                            Some(cap) => before <= cap && prefix > cap,
                            // Missing person facts provide no cap. Identify the
                            // first positive contribution without inventing one.
                            None => before == 0 && excess_deferral > 0,
                        };
                        if contributes_to_failure {
                            return Some(e.id);
                        }
                    }
                    None
                })
                .collect();
            return Err(source_failure(
                key,
                ids,
                ValidationIssue::new(key).at_field("excess_deferral").in_field("code"),
            ));
        }
        let gifts = |own_only: bool, split_only: bool| -> i64 {
            draft
                .adjustments
                .iter()
                .flat_map(|a| &a.qualified_charity)
                .filter(|gift| gift.owner == owner)
                .flat_map(|gift| &gift.payments)
                .filter(|gift| {
                    (!own_only || gift.inherited.is_none()) && (!split_only || gift.split_interest)
                })
                .map(|gift| gift.amount)
                .sum()
        };
        if gifts(false, true) > crate::year_constants::SPLIT_INTEREST_QCD_LIMIT_CENTS {
            let key = "QCD_SPLIT_INTEREST_ABOVE_LIMIT";
            // The gate above owns the sum. This scan only selects its first
            // crossing submitted contributor, so later zero rows are not blamed.
            let mut prefix = 0_i64;
            if let Some(index) = adjustment_items.iter().enumerate().find_map(|(index, item)| {
                if item.owner == crate::DocumentOwner::from(owner)
                    && let IncomeSource::Record { value: crate::IncomeRecord::Qcd(gift) } =
                        &item.document
                    && gift.split_interest
                {
                    prefix += gift.amount;
                    if prefix > crate::year_constants::SPLIT_INTEREST_QCD_LIMIT_CENTS {
                        return Some(index);
                    }
                }
                None
            }) {
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new(key).at_field("amount"),
                    adjustment_positions[index],
                )
                .into());
            }
            return Err(source_failure(
                key,
                Vec::new(),
                ValidationIssue::new(key).at_field("amount"),
            ));
        }
        let designated: i64 = rows(owner)
            .filter(|v| v.kind == RetirementKind::Ira)
            .filter_map(|v| match v.disaster {
                crate::DisasterDistribution::Qualified { amount, .. } => Some(amount),
                _ => None,
            })
            .sum();
        let own_gross: i64 =
            rows(owner).filter(|v| v.kind == RetirementKind::Ira).map(|v| v.gross).sum();
        if designated > 0 && designated + gifts(true, false) > own_gross {
            let key = "DISASTER_ABOVE_QCD_REST";
            let mut prefix = designated;
            if let Some(index) = adjustment_items.iter().enumerate().find_map(|(index, item)| {
                if item.owner == crate::DocumentOwner::from(owner)
                    && let IncomeSource::Record { value: crate::IncomeRecord::Qcd(gift) } =
                        &item.document
                    && gift.inherited.is_none()
                {
                    let before = prefix;
                    prefix += gift.amount;
                    if before <= own_gross && prefix > own_gross {
                        return Some(index);
                    }
                }
                None
            }) {
                return Err(submitted_data_answer_issue(
                    ValidationIssue::new(key).at_field("amount"),
                    adjustment_positions[index],
                )
                .into());
            }
            let ids = draft.income.iter().filter(|e| e.owner == crate::DocumentOwner::from(owner)).filter_map(|e|
                e.income.retirement().is_some_and(|data| data.kind == RetirementKind::Ira && matches!(data.disaster, crate::DisasterDistribution::Qualified { amount, .. } if amount > 0)).then_some(e.id)).collect();
            return Err(source_failure(
                key,
                ids,
                ValidationIssue::new(key).at_field("amount").in_field("disaster"),
            ));
        }
    }
    Ok(())
}
pub(crate) fn retirement_aggregate_issue(
    draft: &FederalDraft,
) -> Option<(Option<i64>, ValidationIssue)> {
    let mut issue = None;
    let _ = retirement_save_rules(draft, &[], &[], &[], None, Some(&mut issue));
    issue
}
pub(crate) fn retirement_aggregate_answer_issue(
    draft: &FederalDraft,
) -> Option<(i64, ValidationIssue)> {
    retirement_aggregate_issue(draft).and_then(|(id, issue)| Some((id?, issue)))
}
// QCD gift items grouped by owner into one QualifiedCharity worksheet per
// owner, each payment labelled from its date.
// The Retirement topic's lump-sum kind (u5-adjust): at most one for the
// return (LUMP_SUM_ELECTION_TWICE, the client rule on IncomeRecords,
// checked again); none is the regular method.
pub(super) fn social_election_record(
    draft: &FederalDraft,
    items: &[IncomeDocumentItem],
    positions: &[usize],
) -> Result<SocialElection, SaveFailure> {
    let mut found = items.iter().enumerate().filter_map(|(index, item)| match &item.document {
        IncomeSource::Record { value: crate::IncomeRecord::SocialLumpSums(v) } => Some((index, v)),
        _ => None,
    });
    let Some((_, first)) = found.next() else {
        return Ok(SocialElection::Regular);
    };
    if let Some((index, _)) = found.next() {
        return Err(submitted_data_answer_issue(
            ValidationIssue::new("LUMP_SUM_ELECTION_TWICE").at_record(),
            positions[index],
        )
        .into());
    }
    let election = first.election_for(draft);
    if !election.valid() {
        return Err(crate::interview::status(
            "MESSAGE_SOCIAL_SECURITY_FOR_EARLIER_YEARS_ENTER_EACH_EARLIER_YEAR_1984_2025_ONCE_WIT",
        )
        .into());
    }
    Ok(election)
}
// The Retirement topic's missed-RMD kind (u5-adjust): one Form 5329 Part IX
// group per row, the spouse's only on a joint return.
pub(super) fn missed_distribution_records(
    interview: &Interview,
    items: &[IncomeDocumentItem],
    positions: &[usize],
) -> Result<Vec<crate::RequiredDistribution>, SaveFailure> {
    let mut records = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let IncomeSource::Record { value: crate::IncomeRecord::MissedRmd(v) } = &item.document
        else {
            continue;
        };
        if item.owner == crate::DocumentOwner::Spouses && !interview.joint() {
            return Err(ValidationIssue::new("MESSAGE_MISSED_REQUIRED_DISTRIBUTION_YOUR_SPOUSE_S_GO_ON_THIS_RETURN_ONLY_WHEN_YOU_F")
                .at_field("owner").in_list("documents", positions[index]).in_field("documents").into());
        }
        let record = v.record(item.owner.owned());
        if record.label.trim().is_empty() || !record.valid() {
            return Err(crate::interview::status(
                "MESSAGE_MISSED_REQUIRED_DISTRIBUTION_ENTER_A_NAME_THE_AMOUNTS_CORRECTION_DATES_IN_20",
            ).into());
        }
        records.push(record);
    }
    Ok(records)
}
// The Retirement topic's Roth IRA history kind (u6-roth): one per owner
// (ROTH_HISTORY_TWICE, the client rule on IncomeRecords, checked again).
pub(super) fn roth_history_records(
    interview: &Interview,
    items: &[IncomeDocumentItem],
    positions: &[usize],
) -> Result<Vec<crate::RothHistory>, SaveFailure> {
    let mut records: Vec<crate::RothHistory> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let IncomeSource::Record { value: crate::IncomeRecord::RothHistory(v) } = &item.document
        else {
            continue;
        };
        if !interview.allowed_owners().contains(&(item.owner as i32)) {
            return Err(ValidationIssue::new("MESSAGE_ROTH_IRA_HISTORY_YOUR_SPOUSE_S_GOES_ON_THIS_RETURN_ONLY_WHEN_YOU_FILE_JOINTL")
                .at_field("owner").in_list("documents", positions[index]).in_field("documents").into());
        }
        if records.iter().any(|r| r.owner == item.owner.owned()) {
            return Err(ValidationIssue::new("ROTH_HISTORY_TWICE")
                .at_field("owner")
                .in_list("documents", positions[index])
                .in_field("documents")
                .into());
        }
        records.push(v.record(item.owner.owned()));
    }
    Ok(records)
}
pub(super) fn qcd_records(items: &[IncomeDocumentItem]) -> Vec<crate::QualifiedCharity> {
    let mut by_owner: Vec<(Owner, Vec<crate::QcdPayment>)> = Vec::new();
    for item in items {
        if let IncomeSource::Record { value: crate::IncomeRecord::Qcd(gift) } = &item.document {
            let payment = crate::QcdPayment {
                inherited: gift.inherited,
                date: gift.date,
                amount: gift.amount,
                split_interest: gift.split_interest,
            };
            match by_owner.iter_mut().find(|(owner, _)| *owner == item.owner.owned()) {
                Some((_, payments)) => payments.push(payment),
                None => by_owner.push((item.owner.owned(), alloc::vec![payment])),
            }
        }
    }
    by_owner
        .into_iter()
        .map(|(owner, payments)| crate::QualifiedCharity { owner, payments, prior_deductions: 0 })
        .collect()
}

impl Interview {
    // The traditional Inherited IRA records a Form 1099-R with code 4 and box
    // 7b or a QCD gift can name, keyed by their income identity (so a stored
    // choice never depends on the catalog's order).
    pub(super) fn income_inherited_iras(&self) -> Vec<crate::RecordChoice> {
        self.inherited_catalog(false)
    }
    // The Roth ones an inherited Roth withdrawal can name: each field is offered
    // only the records of its own kind (F3, 2026-10-09).
    pub(super) fn income_inherited_roth_iras(&self) -> Vec<crate::RecordChoice> {
        self.inherited_catalog(true)
    }
    pub(super) fn inherited_catalog(&self, roth: bool) -> Vec<crate::RecordChoice> {
        self.draft
            .income
            .iter()
            .filter_map(|entry| {
                let FederalIncome::InheritedIra { data } = &entry.income else {
                    return None;
                };
                if data.roth() != roth {
                    return None;
                }
                Some(crate::RecordChoice {
                    id: entry.id,
                    label: crate::Message::new(
                        if data.roth() { "INHERITED_ROTH_CHOICE" } else { "INHERITED_IRA_CHOICE" },
                        alloc::vec![
                            crate::MessageArg::Text(data.decedent.clone()),
                            crate::MessageArg::DateOnly(i64::from(data.died)),
                            crate::MessageArg::Resource(
                                if entry.owner == crate::DocumentOwner::Spouses {
                                    "PERSON_SPOUSE"
                                } else {
                                    "PERSON_YOU"
                                }
                                .into()
                            ),
                        ],
                    )
                    .encode(),
                })
            })
            .collect()
    }
}
