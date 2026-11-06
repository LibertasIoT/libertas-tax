use super::*;
use crate::related_forms::*;
use alloc::collections::BTreeMap;

/// Authorize the original parent and role before assigning new activity identities.
pub(super) fn prepare_activity_ids(
    entries: &mut [FederalIncomeEntry],
    accepted: &[FederalIncomeEntry],
    next_id: &mut i64,
) -> Result<(), (i64, ValidationIssue)> {
    for entry in entries.iter() {
        if let Some(issue) = crate::k1_activity::identity_issue(entry, accepted) {
            return Err((entry.id, issue));
        }
    }
    let positions: Vec<_> = entries
        .iter()
        .flat_map(|entry| {
            crate::k1_activity::source_identity_rows(&entry.income)
                .into_iter()
                .map(move |issue| (entry.id, issue))
        })
        .collect();
    let ids: alloc::collections::BTreeSet<_> = entries.iter().map(|entry| entry.id).collect();
    assign_record_ids(
        entries.iter_mut().flat_map(|entry| crate::k1_activity::source_ids_mut(&mut entry.income)),
        accepted
            .iter()
            .filter(|entry| ids.contains(&entry.id))
            .flat_map(|entry| crate::k1_activity::source_ids(&entry.income)),
        next_id,
    )
    .map_err(|(key, _, index)| {
        let (id, issue) = &positions[index];
        (*id, ValidationIssue { key, field: issue.field.clone() })
    })
}

/// Annual instrument rows use the accepted parent's identities on either editor.
pub(super) fn prepare_annual_ids(
    entries: &mut [FederalIncomeEntry],
    accepted: &[FederalIncomeEntry],
    next_id: &mut i64,
) -> Result<(), (i64, ValidationIssue)> {
    let positions: Vec<_> = entries
        .iter()
        .flat_map(|entry| {
            (0..crate::investment_statements::annual_ids(&entry.income).len())
                .map(move |index| (entry.id, index))
        })
        .collect();
    let ids: alloc::collections::BTreeSet<_> = entries.iter().map(|entry| entry.id).collect();
    assign_record_ids(
        entries
            .iter_mut()
            .flat_map(|entry| crate::investment_statements::annual_ids_mut(&mut entry.income)),
        accepted
            .iter()
            .filter(|entry| ids.contains(&entry.id))
            .flat_map(|entry| crate::investment_statements::annual_ids(&entry.income)),
        next_id,
    )
    .map_err(|(key, _, index)| {
        let (id, row) = positions[index];
        (
            id,
            ValidationIssue::new(key)
                .at_list_item("records", row)
                .in_field("instrument_records")
                .in_field("adjustments"),
        )
    })
}

/// Assign only inside a candidate. Fixed boxes retain their accepted parent's identity;
/// list parts validate their own accepted identities and may be reordered independently.
pub(crate) fn prepare_component_ids(
    entries: &mut [FederalIncomeEntry],
    accepted: &[FederalIncomeEntry],
    next_id: &mut i64,
) -> Result<(), (i64, ValidationIssue)> {
    for entry in entries {
        let old = accepted.iter().find(|old| old.id == entry.id);
        match &mut entry.income {
            FederalIncome::ChildElection { data } => {
                let previous = match old.map(|entry| &entry.income) {
                    Some(FederalIncome::ChildElection { data }) => Some(data),
                    _ => None,
                };
                data.prepare_dividend_ids(previous, next_id).map_err(|issue| (entry.id, issue))?;
            }
            FederalIncome::EstateTrustPortfolio { data } => {
                let previous = match old.map(|row| &row.income) {
                    Some(FederalIncome::EstateTrustPortfolio { data }) => Some(data),
                    _ => None,
                };
                assign_record_ids(
                    data.statements.qbi.statements.iter_mut().map(|row| &mut row.id),
                    previous
                        .into_iter()
                        .flat_map(|old| old.statements.qbi.statements.iter().map(|row| row.id)),
                    next_id,
                )
                .map_err(|(key, _, index)| {
                    (
                        entry.id,
                        ValidationIssue::new(key)
                            .at_field("id")
                            .in_list("statements", index)
                            .in_field("qbi"),
                    )
                })?;
                if let Some(amt) = &mut data.statements.amt_reconciliation {
                    assign_record_ids(
                        amt.adjustments.iter_mut().map(|row| &mut row.id),
                        previous
                            .and_then(|old| old.statements.amt_reconciliation.as_ref())
                            .into_iter()
                            .flat_map(|old| old.adjustments.iter().map(|row| row.id)),
                        next_id,
                    )
                    .map_err(|(key, _, index)| {
                        (
                            entry.id,
                            ValidationIssue::new(key)
                                .at_field("id")
                                .in_list("adjustments", index)
                                .in_field("amt_reconciliation"),
                        )
                    })?;
                }
            }
            FederalIncome::K1099 { data } => {
                let ids = old.into_iter().flat_map(|old| match &old.income {
                    FederalIncome::K1099 { data } => data.parts.iter().map(|p| p.id).collect(),
                    _ => Vec::new(),
                });
                assign_record_ids(data.parts.iter_mut().map(|p| &mut p.id), ids, next_id).map_err(
                    |(key, _, index)| {
                        (entry.id, ValidationIssue::new(key).at_field("id").in_list("parts", index))
                    },
                )?;
            }
            FederalIncome::Misc1099 { data } => {
                data.component_ids = match old.map(|old| &old.income) {
                    Some(FederalIncome::Misc1099 { data }) => data.component_ids.clone(),
                    _ => Default::default(),
                };
                let boxes = crate::Form1099Misc::from(&*data);
                let ids = &mut data.component_ids;
                for (id, amount) in [
                    (&mut ids.rents, boxes.rents),
                    (&mut ids.royalties, boxes.royalties),
                    (&mut ids.other, boxes.other),
                    (&mut ids.fishing, boxes.fishing),
                    (&mut ids.medical, boxes.medical),
                    (&mut ids.crop_insurance, boxes.crop_insurance),
                    (&mut ids.attorney, boxes.attorney),
                    (&mut ids.fish_sales, boxes.fish_sales),
                ] {
                    if *id == 0 && amount != 0 {
                        assign_record_ids(core::iter::once(id), core::iter::empty(), next_id)
                            .map_err(|(key, _, _)| (entry.id, ValidationIssue::new(key)))?;
                    }
                }
            }
            FederalIncome::GovernmentPayment { data } => {
                data.component_ids = match old.map(|old| &old.income) {
                    Some(FederalIncome::GovernmentPayment { data }) => data.component_ids.clone(),
                    _ => Default::default(),
                };
                let boxes = crate::FederalUnemployment::from(&*data);
                for (id, amount) in [
                    (&mut data.component_ids.refund, data.form.state_or_local_income_tax_refunds),
                    (&mut data.component_ids.grant, boxes.taxable_grants),
                    (&mut data.component_ids.agriculture, boxes.agriculture_payment),
                ] {
                    if *id == 0 && amount != 0 {
                        assign_record_ids(core::iter::once(id), core::iter::empty(), next_id)
                            .map_err(|(key, _, _)| (entry.id, ValidationIssue::new(key)))?;
                    }
                }
            }
            _ => (),
        }
    }
    Ok(())
}

pub(crate) fn catalog(draft: &FederalDraft) -> Vec<CenterDocument> {
    catalog_with_provisional(draft, None)
}

pub(crate) fn catalog_with_provisional(
    draft: &FederalDraft,
    provisional: Option<&ProvisionalActivity>,
) -> Vec<CenterDocument> {
    use crate::information_returns::{Kind, candidates as accepted_candidates};
    let candidates = |draft: &FederalDraft, owner, kind| {
        let mut ids = accepted_candidates(draft, owner, kind);
        if let Some(p) = provisional {
            ids.retain(|id| *id != p.id);
            let matching = matches!(
                (p.kind, kind),
                (ActivityKind::Business, Kind::Business)
                    | (ActivityKind::Rental, Kind::Rental)
                    | (ActivityKind::Hobby, Kind::Hobby)
                    | (ActivityKind::Equipment, Kind::Equipment)
                    | (ActivityKind::Royalty, Kind::Royalty)
            );
            if matching
                && crate::information_returns::owner_eligible(
                    draft,
                    owner,
                    p.owner,
                    kind == Kind::Business,
                )
            {
                ids.push(p.id);
            }
        }
        ids
    };
    let cooperative_candidates = |owner| {
        let mut ids = crate::cooperative::eligible_activities(draft, owner);
        if let Some(p) = provisional {
            ids.retain(|id| *id != p.id);
            if p.owner == owner
                && p.cooperative_eligible
                && matches!(
                    p.kind,
                    ActivityKind::Business | ActivityKind::Farm | ActivityKind::FarmRental
                )
            {
                ids.push(p.id);
            }
        }
        ids
    };
    let farm_candidates = |owner| {
        cooperative_candidates(owner)
            .into_iter()
            .filter(|id| {
                if provisional.is_some_and(|p| p.id == *id) {
                    return provisional.is_some_and(|p| {
                        matches!(p.kind, ActivityKind::Farm | ActivityKind::FarmRental)
                    });
                }
                draft.income.iter().any(|e| {
                    e.id == *id
                        && match &e.income {
                            FederalIncome::Business { data } => {
                                matches!(data.activity, crate::BusinessActivity::Farming { .. })
                            }
                            FederalIncome::FarmRental { .. } => true,
                            _ => false,
                        }
                })
            })
            .collect::<Vec<_>>()
    };

    let mut out = Vec::new();
    for entry in &draft.income {
        let choice = source_choice(entry);
        let mut push = |id, kind, amount, label: Vec<u8>, part: Vec<u8>, eligible: Vec<i64>| {
            if id > 0 {
                out.push(CenterDocument {
                    id,
                    form_id: entry.id,
                    kind,
                    part,
                    eligible,
                    summary: RelatedFormSummary {
                        label,
                        payer: related_form_payer(entry),
                        amount: Some(amount),
                    },
                });
            }
        };
        let component_part = |key: &str, args: Vec<MessageArg>| Message::new(key, args).encode();
        let box_part = |number: i64| {
            component_part("RELATED_PART_BOX", alloc::vec![MessageArg::Number(number)])
        };
        let box_label = |number| {
            Message::new(
                "RELATED_FORM_COMPONENT_BOX",
                alloc::vec![MessageArg::Encoded(choice.label.clone()), MessageArg::Number(number)],
            )
            .encode()
        };
        match &entry.income {
            FederalIncome::Nec1099 { data }
                if crate::FederalForm1099Nec::from(data).compensation != 0 =>
            {
                let data = crate::FederalForm1099Nec::from(data);
                let eligible = match data.belongs {
                    None => candidates(draft, entry.owner.owned(), Kind::Business),
                    Some(crate::IncomeUse::NotForProfit) => Vec::new(),
                    _ => Vec::new(),
                };
                push(
                    entry.id,
                    DocumentKind::Form1099Nec,
                    data.compensation,
                    choice.label.clone(),
                    component_part("RELATED_PART_NEC", alloc::vec![]),
                    eligible,
                );
            }
            FederalIncome::K1099 { data } => {
                for part in &data.parts {
                    let eligible = match &part.belongs {
                        None => [Kind::Business, Kind::Rental, Kind::Hobby]
                            .into_iter()
                            .flat_map(|kind| candidates(draft, entry.owner.owned(), kind))
                            .collect(),
                        Some(crate::KUse::NotIncome | crate::KUse::PersonalGain) => {
                            continue;
                        }
                    };
                    let amount = part.amount - part.refunded;
                    let label = Message::new(
                        "RELATED_FORM_COMPONENT_PART",
                        alloc::vec![
                            MessageArg::Encoded(choice.label.clone()),
                            MessageArg::usd_cents(amount)
                        ],
                    )
                    .encode();
                    push(
                        part.id,
                        DocumentKind::Form1099KPart,
                        amount,
                        label,
                        component_part(
                            "RELATED_PART_AMOUNT",
                            alloc::vec![MessageArg::usd_cents(amount)],
                        ),
                        eligible,
                    );
                }
            }
            FederalIncome::Misc1099 { data } => {
                let data = &crate::Form1099Misc::from(data);
                let rents = [Kind::Business, Kind::Rental, Kind::Equipment]
                    .into_iter()
                    .flat_map(|kind| candidates(draft, entry.owner.owned(), kind))
                    .collect();
                let royalties = [Kind::Business, Kind::Royalty]
                    .into_iter()
                    .flat_map(|kind| candidates(draft, entry.owner.owned(), kind))
                    .collect();
                let other = match &data.other_use {
                    None => [Kind::Business, Kind::Hobby]
                        .into_iter()
                        .flat_map(|kind| candidates(draft, entry.owner.owned(), kind))
                        .collect(),
                    _ => Vec::new(),
                };
                for (id, amount, kind, number, eligible) in [
                    (
                        data.component_ids.rents,
                        data.rents,
                        DocumentKind::Form1099MiscRents,
                        1,
                        rents,
                    ),
                    (
                        data.component_ids.royalties,
                        data.royalties,
                        DocumentKind::Form1099MiscRoyalties,
                        2,
                        royalties,
                    ),
                    (
                        data.component_ids.other,
                        data.other,
                        DocumentKind::Form1099MiscOther,
                        3,
                        other,
                    ),
                    (
                        data.component_ids.fishing,
                        data.fishing,
                        DocumentKind::Form1099MiscBusiness,
                        5,
                        candidates(draft, entry.owner.owned(), Kind::Business),
                    ),
                    (
                        data.component_ids.medical,
                        data.medical,
                        DocumentKind::Form1099MiscBusiness,
                        6,
                        candidates(draft, entry.owner.owned(), Kind::Business),
                    ),
                    (
                        data.component_ids.crop_insurance,
                        data.crop_insurance,
                        DocumentKind::Form1099MiscCropInsurance,
                        9,
                        farm_candidates(entry.owner.owned()),
                    ),
                    (
                        data.component_ids.attorney,
                        data.attorney - data.client_funds,
                        DocumentKind::Form1099MiscBusiness,
                        10,
                        candidates(draft, entry.owner.owned(), Kind::Business),
                    ),
                    (
                        data.component_ids.fish_sales,
                        data.fish_sales,
                        DocumentKind::Form1099MiscBusiness,
                        11,
                        candidates(draft, entry.owner.owned(), Kind::Business),
                    ),
                ] {
                    if (number == 10 && data.attorney != 0) || (number != 10 && amount != 0) {
                        push(id, kind, amount, box_label(number), box_part(number), eligible);
                    }
                }
            }
            FederalIncome::Cooperative { data } => {
                let data = &crate::FederalCooperative::from(data);
                if data.allocations.is_empty() {
                    push(
                        entry.id,
                        DocumentKind::Patr1099,
                        data.distributions(),
                        choice.label.clone(),
                        component_part("RELATED_PART_PATR", alloc::vec![]),
                        cooperative_candidates(entry.owner.owned()),
                    );
                }
                for allocation in &data.allocations {
                    let amount = allocation.distributions.total();
                    let label = Message::new(
                        "RELATED_FORM_COMPONENT_ALLOCATION",
                        alloc::vec![
                            MessageArg::Encoded(choice.label.clone()),
                            MessageArg::usd_cents(amount)
                        ],
                    )
                    .encode();
                    push(
                        allocation.id,
                        DocumentKind::Patr1099,
                        amount,
                        label,
                        component_part(
                            "RELATED_PART_ALLOCATION",
                            alloc::vec![MessageArg::usd_cents(amount)],
                        ),
                        cooperative_candidates(entry.owner.owned()),
                    );
                }
            }
            FederalIncome::GovernmentPayment { data } => {
                let data = &crate::FederalUnemployment::from(data);
                if data.taxable_grants != 0 {
                    let eligible = match &data.grant_use {
                        None => candidates(draft, entry.owner.owned(), Kind::Business)
                            .into_iter()
                            .chain(farm_candidates(entry.owner.owned()))
                            .collect(),
                        _ => Vec::new(),
                    };
                    push(
                        data.component_ids.grant,
                        DocumentKind::Form1099GGrant,
                        data.taxable_grants,
                        box_label(6),
                        box_part(6),
                        eligible,
                    );
                }
                if data.agriculture_payment != 0 {
                    let eligible = farm_candidates(entry.owner.owned());
                    push(
                        data.component_ids.agriculture,
                        DocumentKind::Form1099GAgriculture,
                        data.agriculture_payment,
                        box_label(7),
                        box_part(7),
                        eligible,
                    );
                }
            }
            _ => (),
        }
    }
    for loan in draft.deductions.iter().flat_map(|d| d.mortgage_loans().unwrap_or_default()) {
        if loan.id > 0 && matches!(loan.details, crate::MortgageLoanDetails::FirstLender(_)) {
            out.push(CenterDocument {
                id: loan.id,
                form_id: loan.id,
                kind: DocumentKind::Mortgage1098,
                part: Message::new("RELATED_PART_AMOUNT", alloc::vec![MessageArg::usd_cents(0)]).encode(),
                eligible: draft
                    .income
                    .iter()
                    .filter(|e| matches!(&e.income, FederalIncome::Rental { data } if !matches!(data.personal_use, RentalPersonalUse::No)))
                    .map(|e| e.id)
                    .filter(|id| provisional.is_none_or(|p| p.id != *id))
                    .chain(
                        provisional
                            .into_iter()
                            .filter(|p| p.kind == ActivityKind::Rental)
                            .map(|p| p.id),
                    )
                    .collect(),
                summary: RelatedFormSummary {
                    label: Message::new(
                        "MORTGAGE_SAVED_LOAN",
                        alloc::vec![
                            MessageArg::Number(loan.id),
                            if loan.property.is_empty() {
                                MessageArg::Resource("MORTGAGE_PROPERTY_UNNAMED".into())
                            } else {
                                MessageArg::Text(loan.property.clone())
                            },
                            MessageArg::DateOnly(i64::from(loan.incurred))
                        ],
                    )
                    .encode(),
                    payer: mortgage_lender_name(&loan.lender),
                    amount: Some(loan.interest),
                },
            });
        }
    }
    out
}

pub(crate) fn draft_claims(draft: &FederalDraft) -> Vec<ActivityClaim> {
    let mut explicit: BTreeMap<i64, Vec<i64>> = BTreeMap::new();
    for entry in &draft.income {
        for flow in crate::information_returns::flows(&entry.income) {
            if let (Some(activity), Some(key)) =
                (flow.choice, crate::information_returns::component_key(entry, flow.slot))
            {
                explicit.entry(activity).or_default().push(key);
            }
        }
        match &entry.income {
            FederalIncome::Business { data } => match &data.activity {
                crate::BusinessActivity::Farming { data: farm } => {
                    explicit.entry(entry.id).or_default().extend(&farm.related)
                }
                crate::BusinessActivity::Ordinary { .. } => {
                    explicit.entry(entry.id).or_default().extend(&data.related)
                }
                _ => (),
            },
            FederalIncome::FarmRental { data } => {
                explicit.entry(entry.id).or_default().extend(&data.farm.related);
            }
            FederalIncome::Rental { data } => {
                explicit.entry(entry.id).or_default().extend(&data.related)
            }
            _ => (),
        }
    }
    draft
        .income
        .iter()
        .filter_map(|entry| {
            let kinds = match &entry.income {
                FederalIncome::Business { data }
                    if matches!(data.activity, crate::BusinessActivity::Ordinary { .. }) =>
                {
                    BUSINESS_KINDS
                }
                FederalIncome::Rental { .. } => RENTAL_KINDS,
                FederalIncome::Business { data }
                    if matches!(data.activity, crate::BusinessActivity::Farming { .. }) =>
                {
                    FARM_KINDS
                }
                FederalIncome::FarmRental { .. } => FARM_RENTAL_KINDS,
                FederalIncome::HobbySales { .. } => HOBBY_KINDS,
                FederalIncome::Royalty { .. } => ROYALTY_KINDS,
                FederalIncome::NonbusinessProperty {
                    kind: crate::NonbusinessPropertyKind::PersonalRental { .. },
                    ..
                } => EQUIPMENT_KINDS,
                _ => return None,
            };
            let related = match &entry.income {
                FederalIncome::HobbySales { data } => data.related.clone(),
                FederalIncome::Royalty { data } => data.related.clone(),
                FederalIncome::NonbusinessProperty {
                    kind: crate::NonbusinessPropertyKind::PersonalRental { related, .. },
                    ..
                } => related.clone(),
                _ => explicit.remove(&entry.id).unwrap_or_default(),
            };
            Some(ActivityClaim { id: entry.id, related, kinds })
        })
        .collect()
}

pub(super) fn activity_page<A: Activity>(
    draft: &FederalDraft,
    activity: i64,
    keys: &[i64],
) -> RelatedForms {
    RelatedForms::page::<A>(
        keys,
        &crate::related_forms::catalog_for(draft, activity),
        &crate::related_forms::claimed_elsewhere(draft, activity),
    )
}

pub(super) fn fill_activity_page<A: Activity>(draft: &FederalDraft, activity: i64, page: &mut A) {
    let keys: Vec<_> = page.related().forms.iter().map(|row| row.key).collect();
    *page.related_mut() = activity_page::<A>(draft, activity, &keys);
}

// Rebuild row-owned calculated catalogs on a returned unsaved editor using
// the same provisional eligibility owner as its field calculations.
pub(super) fn refresh_submitted_activity_page(
    draft: &FederalDraft,
    item: &mut crate::IncomeRecordItem,
) {
    let kind = match &item.document {
        crate::IncomeRecord::Rental(_) => ActivityKind::Rental,
        crate::IncomeRecord::Royalties(_) => ActivityKind::Royalty,
        _ => return,
    };
    let mut center = crate::related_forms::catalog_for_provisional(
        draft,
        &ProvisionalActivity { id: item.id, owner: item.owner, kind, cooperative_eligible: false },
    );
    let claimed = crate::related_forms::claimed_elsewhere(draft, item.id);
    match &mut item.document {
        crate::IncomeRecord::Rental(page) => {
            page.more.known_at_risk = page.more.at_risk.iter().cloned().collect();
            if matches!(page.personal_use, PersonalUse::No) {
                center.retain(|source| source.kind != DocumentKind::Mortgage1098);
            }
            page.related.candidates =
                crate::RelatedForms::page::<crate::Rental>(&[], &center, &claimed).candidates;
        }
        crate::IncomeRecord::Royalties(page) => {
            page.related.candidates =
                crate::RelatedForms::page::<crate::RoyaltyProperty>(&[], &center, &claimed)
                    .candidates;
        }
        _ => unreachable!(),
    }
}

pub(super) fn review_forms(draft: &FederalDraft) -> Vec<ActivityRelatedForms> {
    draft
        .income
        .iter()
        .filter_map(|entry| {
            let related = match &entry.income {
                FederalIncome::Business { data } => match &data.activity {
                    crate::BusinessActivity::Ordinary { .. } => {
                        activity_page::<crate::Business>(draft, entry.id, &data.related)
                    }
                    crate::BusinessActivity::Farming { data } => {
                        activity_page::<crate::Farm>(draft, entry.id, &data.related)
                    }
                    _ => return None,
                },
                FederalIncome::FarmRental { data } => {
                    activity_page::<crate::FarmRental>(draft, entry.id, &data.farm.related)
                }
                FederalIncome::Rental { data } => {
                    activity_page::<crate::Rental>(draft, entry.id, &data.related)
                }
                FederalIncome::HobbySales { data } => {
                    activity_page::<crate::HobbySales>(draft, entry.id, &data.related)
                }
                FederalIncome::Royalty { data } => {
                    activity_page::<crate::RoyaltyProperty>(draft, entry.id, &data.related)
                }
                FederalIncome::NonbusinessProperty {
                    kind: crate::NonbusinessPropertyKind::PersonalRental { related, .. },
                    ..
                } => activity_page::<crate::EquipmentRent>(draft, entry.id, related),
                _ => return None,
            };
            let forms: Vec<_> = related
                .forms
                .into_iter()
                .filter_map(|row| {
                    row.summary.map(|summary| ReviewedRelatedForm { key: row.key, summary })
                })
                .collect();
            (!forms.is_empty())
                .then(|| ActivityRelatedForms { activity: source_choice(entry).label, forms })
        })
        .collect()
}

impl Interview {
    pub(super) fn view_related_form(&self, key: i64) -> Reply {
        let Some(source) = catalog(&self.draft).into_iter().find(|d| d.id == key) else {
            return problem_key("PROBLEM_RELATED_FORM_UNAVAILABLE");
        };
        if let Some(entry) = self.draft.income.iter().find(|e| e.id == source.form_id) {
            let Some(item) =
                document_item(entry, &[], crate::federal_rules::separate_return(&self.draft))
            else {
                return problem_key("PROBLEM_RELATED_FORM_UNAVAILABLE");
            };
            let form = match (&entry.income, item.document) {
                (
                    FederalIncome::Nec1099 { data },
                    IncomeSource::SelfEmployment {
                        value: crate::SelfEmploymentDocument::Form1099Nec(v),
                    },
                ) => RelatedForm::Form1099Nec {
                    payer: v.payer,
                    recipient: v.recipient,
                    document: data.into(),
                    state_details: v.state_details,
                },
                (
                    FederalIncome::K1099 { data },
                    IncomeSource::SelfEmployment {
                        value: crate::SelfEmploymentDocument::Form1099K(v),
                    },
                ) => {
                    let v = crate::Form1099K::from(&v);
                    RelatedForm::Form1099K {
                        payer: v.payer,
                        recipient: v.recipient,
                        document: data.into(),
                        state_details: v.state_details,
                    }
                }
                (
                    FederalIncome::Misc1099 { data },
                    IncomeSource::OtherIncome {
                        value: crate::OtherIncomeDocument::Form1099Misc(v),
                    },
                ) => RelatedForm::Form1099Misc {
                    payer: v.payer,
                    recipient: v.recipient,
                    document: data.into(),
                    state_details: (&v.state_details).into(),
                },
                (FederalIncome::Cooperative { data }, _) => RelatedForm::Patr1099(data.into()),
                (
                    FederalIncome::GovernmentPayment { data },
                    IncomeSource::GovernmentPayment { value: v, .. },
                ) => RelatedForm::Form1099G {
                    payer: v.payer,
                    recipient: v.recipient,
                    document: data.into(),
                    state_details: v.state_details,
                },
                _ => return problem_key("PROBLEM_RELATED_FORM_UNAVAILABLE"),
            };
            return Reply::Response(P::FormViewed {
                owner: Some(entry.owner.owned()),
                payer: related_form_payer(entry),
                form,
            });
        }
        match self
            .draft
            .deductions
            .iter()
            .flat_map(|d| d.mortgage_loans().unwrap_or_default())
            .find(|loan| loan.id == source.form_id)
        {
            Some(loan) => Reply::Response(P::FormViewed {
                owner: None,
                payer: mortgage_lender_name(&loan.lender),
                form: RelatedForm::Mortgage1098(loan.clone()),
            }),
            None => problem_key("PROBLEM_RELATED_FORM_UNAVAILABLE"),
        }
    }
}

fn related_form_payer(entry: &FederalIncomeEntry) -> alloc::string::String {
    if !entry.label.trim().is_empty() {
        return entry.label.clone();
    }
    entry
        .printed_names()
        .and_then(|names| names.issuer.as_ref())
        .map(|issuer| issuer.name.clone())
        .unwrap_or_default()
}

fn mortgage_lender_name(lender: &crate::MortgageLender) -> alloc::string::String {
    match lender {
        crate::MortgageLender::NoForm { name }
        | crate::MortgageLender::PrivateSeller { name, .. } => name.clone(),
        // The institutional-lender answer records no lender name.
        crate::MortgageLender::Institution => alloc::string::String::new(),
    }
}
