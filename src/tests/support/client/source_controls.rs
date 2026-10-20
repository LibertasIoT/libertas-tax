// Current typed source producers own these controls. Case names retain the
// original metadata regression identity, without retaining an older wire schema.
use super::income_pages::open_topic;
use crate::tests::cross::interview::{accepted, from_review};
use crate::tests::*;

#[derive(Clone, Copy)]
pub(in crate::tests) enum SourceScope {
    Income,
    Health,
    CommitOnly,
    Fieldless,
    Secondary(SecondaryCorrection),
}
#[derive(Clone, Copy)]
pub(in crate::tests) enum SecondaryCorrection {
    Employer,
    SharedEmployer,
    AdjustmentOwner,
    AnnuitySource,
    InvestmentSource,
    InvestmentOrigin,
    StudentSource,
    StudentDuplicate,
    RepaymentChoice,
    DeductionChoice,
    MedicalSource,
    SituationChoice,
    AlimonyRecapture,
}

pub(in crate::tests) fn source_calculation(request: &P) -> P {
    match request {
        P::SaveIncomes { .. } => super::estimate_documents_from_save(request),
        P::SaveHsa {
            user,
            value,
            allowed_owners,
            other_coverage_allowed,
            allowed_family_split,
            hsa_failed_deposits,
            archer_failed_deposits,
            hsa_return_documents,
            archer_return_documents,
            ira_sources,
            archer_income,
            ..
        } => P::EstimateHsa {
            user: *user,
            value: value.clone(),
            allowed_owners: allowed_owners.clone(),
            other_coverage_allowed: *other_coverage_allowed,
            allowed_family_split: allowed_family_split.clone(),
            hsa_failed_deposits: hsa_failed_deposits.clone(),
            archer_failed_deposits: archer_failed_deposits.clone(),
            hsa_return_documents: hsa_return_documents.clone(),
            archer_return_documents: archer_return_documents.clone(),
            ira_sources: ira_sources.clone(),
            archer_income: archer_income.clone(),
        },
        P::SaveSelfEmployedBenefits {
            user,
            value,
            ptc_policies,
            health_activities,
            health_people,
            health_wages,
            ..
        } => P::EstimateSelfEmployedBenefits {
            user: *user,
            value: value.clone(),
            ptc_policies: ptc_policies.clone(),
            health_activities: health_activities.clone(),
            health_people: health_people.clone(),
            health_wages: health_wages.clone(),
        },
        P::SaveMarketplace {
            user,
            value,
            people,
            ptc_people,
            ptc_families,
            ptc_policies,
            ptc_w2,
            ptc_employees,
            ptc_nonreturn_people,
            ptc_claimed_dependents,
            allowed_marriage,
            allowed_separate,
            ..
        } => P::EstimateMarketplace {
            user: *user,
            value: value.clone(),
            people: people.clone(),
            ptc_people: ptc_people.clone(),
            ptc_families: ptc_families.clone(),
            ptc_policies: ptc_policies.clone(),
            ptc_w2: ptc_w2.clone(),
            ptc_employees: ptc_employees.clone(),
            ptc_nonreturn_people: ptc_nonreturn_people.clone(),
            ptc_claimed_dependents: ptc_claimed_dependents.clone(),
            allowed_marriage: allowed_marriage.clone(),
            allowed_separate: allowed_separate.clone(),
        },
        P::SaveCare {
            user,
            value,
            allowed_injured_source_facts,
            people,
            allowed_owners,
            care_filers,
            care_months,
            businesses,
            farms,
            partnerships,
            ..
        } => P::EstimateCare {
            user: *user,
            value: value.clone(),
            allowed_injured_source_facts: allowed_injured_source_facts.clone(),
            people: people.clone(),
            allowed_owners: allowed_owners.clone(),
            care_filers: care_filers.clone(),
            care_months: care_months.clone(),
            businesses: businesses.clone(),
            farms: farms.clone(),
            partnerships: partnerships.clone(),
        },
        P::SaveDeductions {
            user, allowed_routes, allowed_repayment, home_sites, value, ..
        } => P::EstimateDeductions {
            user: *user,
            allowed_routes: allowed_routes.clone(),
            allowed_repayment: allowed_repayment.clone(),
            home_sites: home_sites.clone(),
            value: value.clone(),
        },
        P::SaveEducation { user, value, .. } => {
            P::EstimateEducation { user: *user, value: value.clone() }
        }
        P::SaveEmployeeExpenses { user, allowed_owners, value, .. } => {
            P::EstimateEmployeeExpenses {
                user: *user,
                allowed_owners: allowed_owners.clone(),
                value: value.clone(),
            }
        }
        P::SaveMedical { user, value, .. } => {
            P::EstimateMedical { user: *user, value: value.clone() }
        }
        P::SaveMoreAdjustments { user, allowed_owners, value, .. } => P::EstimateMoreAdjustments {
            user: *user,
            allowed_owners: allowed_owners.clone(),
            value: value.clone(),
        },
        P::SaveOtherItemized {
            user,
            allowed_owners,
            interest_sources,
            investment_k1_sources,
            investment_origin_choices,
            value,
            ..
        } => P::EstimateOtherItemized {
            user: *user,
            allowed_owners: allowed_owners.clone(),
            interest_sources: interest_sources.clone(),
            investment_k1_sources: investment_k1_sources.clone(),
            investment_origin_choices: investment_origin_choices.clone(),
            value: value.clone(),
        },
        P::SaveSituations { user, topic, allowed_situations, value, .. } => P::EstimateSituations {
            user: *user,
            topic: *topic,
            allowed_situations: allowed_situations.clone(),
            value: value.clone(),
        },
        _ => panic!("a current source editor has a calculation"),
    }
}

// Re-save the accepted income editor through its normal hot and cold lifecycle.
// Refusal, correction, Estimate and Preview policies remain with each caller.
pub(in crate::tests) fn income_editor_resave(
    case: &str,
    session: &mut Interview,
    topic: TaxSection,
) {
    let saved = session.draft.clone();
    let page = open_topic(session, topic);
    accepted(&super::send(session, page));
    assert_eq!(session.draft, saved, "{case}: accepted source reopen/resave");
    let mut restored = super::restart(saved.clone());
    assert_eq!(restored.draft, saved);
    let page = open_topic(&mut restored, topic);
    accepted(&super::send(&mut restored, page));
    assert_eq!(restored.draft, saved, "{case}: accepted source restart/resave");
}

fn source_topic(request: &P) -> TaxSection {
    match request {
        P::SaveIncomes { topic, .. } | P::SaveSituations { topic, .. } => *topic,
        P::SaveHsa { .. } => TaxSection::Hsa,
        P::SaveSelfEmployedBenefits { .. } => TaxSection::SelfEmployedBenefits,
        P::SaveMarketplace { .. } => TaxSection::Marketplace,
        P::SaveCare { .. } => TaxSection::Care,
        P::SaveDeductions { .. } => TaxSection::Deductions,
        P::SaveEducation { .. } => TaxSection::Education,
        P::SaveEmployeeExpenses { .. } => TaxSection::EmployeeExpenses,
        P::SaveMedical { .. } => TaxSection::Medical,
        P::SaveMoreAdjustments { .. } => TaxSection::MoreAdjustments,
        P::SaveOtherItemized { .. } => TaxSection::OtherItemized,
        _ => panic!("a current source editor has a topic"),
    }
}
fn reopen(session: &mut Interview, request: &P) -> P {
    if let P::SaveIncomes { topic, .. } = request {
        return open_topic(session, *topic);
    }
    match from_review(session, source_topic(request)) {
        Reply::Edit(page) => page,
        reply @ Reply::Response(P::BeginSituations { .. }) => {
            crate::tests::cross::interview::save_empty_situations(reply)
        }
        reply => panic!("accepted source editor must reopen: {reply:?}"),
    }
}
fn assert_problem(case: &str, reply: &Reply, key: &str, path: Option<&str>) -> Vec<u8> {
    let Reply::Response(P::Problem { message, field }) = reply else {
        panic!("{case}: expected source refusal, got {reply:?}")
    };
    assert_eq!(crate::Message::decode(message).key, key, "{case}: diagnostic");
    assert_eq!(field.as_deref(), path, "{case}: submitted source path");
    message.clone()
}
fn correct(session: &Interview, request: &mut P, kind: SecondaryCorrection) {
    use SecondaryCorrection as C;
    match request {
        P::SaveEmployeeExpenses { value, .. } => {
            if matches!(kind, C::SharedEmployer) {
                value.forms.truncate(1);
            }
            for form in &mut value.forms {
                let source = session
                    .draft
                    .income
                    .iter()
                    .find(|entry| {
                        entry.owner == form.owner
                            && matches!(entry.income, FederalIncome::Wage { .. })
                    })
                    .expect("actual same-owner W-2");
                assert!(source.id > 0);
                form.employers = vec![source.id];
            }
        }
        P::SaveMoreAdjustments { value, .. } => {
            if matches!(kind, C::AlimonyRecapture) {
                let MoreAdjustment::AlimonyPaid(row) = &mut value.adjustments[0] else {
                    unreachable!()
                };
                let amount = row.payment.amount();
                let crate::AlimonyRecapture::ThirdYear { current_excluded, .. } =
                    &mut row.payment.recapture
                else {
                    unreachable!()
                };
                *current_excluded = amount;
            } else {
                for row in &mut value.adjustments {
                    match row {
                        MoreAdjustment::AlimonyPaid(row) => row.owner = Owner::Taxpayer,
                        MoreAdjustment::MovingExpenses(row) => row.owner = Owner::Taxpayer,
                        _ => unreachable!(),
                    }
                }
            }
        }
        P::SaveOtherItemized { value, .. } => match kind {
            C::AnnuitySource => {
                let source = value.contracts[0].id;
                assert!(session.draft.income.iter().any(|entry| entry.id == source));
                value.annuity_losses[0].contract = source;
            }
            C::InvestmentSource => value.investment_interest.as_mut().unwrap().loans[0].source = 0,
            C::InvestmentOrigin => {
                let source = crate::investment_interest::k1_sources(&session.draft)[0].id;
                assert!(source > 0 && session.draft.income.iter().any(|entry| entry.id == source));
                value.investment_interest.as_mut().unwrap().loans[1].originating_k1 =
                    Some(crate::InvestmentK1Origin::Entity { source });
            }
            _ => unreachable!(),
        },
        P::SaveEducation { value, .. } => {
            if matches!(kind, C::StudentDuplicate) {
                value.students.truncate(1);
            }
            value.students[0].student = value.people[0].id;
        }
        P::SaveDeductions { value, .. } => match kind {
            C::RepaymentChoice => value.repayment = crate::RepaymentClaim::None,
            C::DeductionChoice => value.route = DeductionRoute::Standard,
            _ => unreachable!(),
        },
        P::SaveMedical { value, .. } => value.long_term_care[0].person = value.people[0].id,
        P::SaveSituations { value, .. } => value.situations.clear(),
        _ => unreachable!(),
    }
}

// The caller supplies its actual current typed candidate at the original refusal
// site. Extra lifecycle checks use an independent normal session, so the owning
// test continues with its own accepted source and independent financial assertions.
#[inline(never)]
pub(in crate::tests) fn current_source_refusal(
    case: &str,
    session: &mut Interview,
    request: P,
    key: &str,
    path: Option<&str>,
    scope: SourceScope,
) -> Reply {
    let before = session.draft.clone();
    if matches!(scope, SourceScope::Secondary(_) | SourceScope::Fieldless) {
        assert!(before.valid_stored(), "{case}: accepted Draft is valid");
    }
    let dirty = session.dirty;
    let reply = super::send(session, request.clone());
    let message = assert_problem(case, &reply, key, path);
    // Original802's living-owner statement has source ID0 and no display
    // label. Its diagnostic names that source with the independent Number0
    // argument; every other original refusal has no arguments.
    let expected_message = if case == "income-source-802" {
        crate::Message::new(key, vec![crate::MessageArg::Number(0)]).encode()
    } else {
        libertas::libertas_formatted_text(key, &[])
    };
    assert_eq!(message, expected_message, "{case}: complete source diagnostic");

    assert_eq!(session.draft, before, "{case}: full rejected Draft");
    assert_eq!(session.draft.to_avro(), before.to_avro(), "{case}: rejected Draft bytes");
    assert_eq!(session.dirty, dirty, "{case}: rejected dirty state");
    if path.is_some() {
        crate::tests::unit47_rule_cases::export_unit47_rule_case(
            case,
            &request,
            match &reply {
                Reply::Response(value) => value,
                _ => unreachable!(),
            },
        );
    }
    let mut lifecycle = super::restart(before.clone());
    assert_eq!(lifecycle.draft, before, "{case}: cold accepted source");
    if matches!(scope, SourceScope::Fieldless) {
        return reply;
    }
    if path.is_some() {
        let calculated = super::send(&mut lifecycle, source_calculation(&request));
        if matches!(scope, SourceScope::CommitOnly) {
            accepted(&calculated);
        } else {
            assert_eq!(
                assert_problem(case, &calculated, key, path),
                message,
                "{case}: complete Save/Estimate diagnostic"
            );
        }
        assert_eq!(lifecycle.draft, before, "{case}: calculation is transient");
        assert!(!lifecycle.dirty, "{case}: calculation keeps clean restart");
    }
    let baseline = estimate(&before);
    let correction = if let SourceScope::Secondary(kind) = scope {
        let mut corrected = request.clone();
        correct(&lifecycle, &mut corrected, kind);
        corrected
    } else {
        reopen(&mut lifecycle, &request)
    };
    let corrected_estimate = super::send(&mut lifecycle, source_calculation(&correction));
    if matches!(scope, SourceScope::Income) {
        assert!(
            matches!(corrected_estimate, Reply::Response(P::IncomeRunningEstimate { .. })),
            "{case}: accepted correction returns the income estimate"
        );
    } else {
        accepted(&corrected_estimate);
    }
    assert_eq!(lifecycle.draft, before, "{case}: correction estimate is transient");
    assert!(!lifecycle.dirty);
    accepted(&super::send(&mut lifecycle, correction));
    let saved = lifecycle.draft.clone();
    let financial = estimate(&saved);
    match scope {
        SourceScope::Income => assert_eq!(saved, before, "{case}: accepted-source correction"),
        SourceScope::Health | SourceScope::CommitOnly => {
            assert_eq!(saved.income, before.income, "{case}: source facts");
            assert_eq!(saved.next_id, before.next_id, "{case}: source identities");
            assert_eq!(financial.return_lines, baseline.return_lines, "{case}: return lines");
            assert_eq!(
                (
                    financial.tax,
                    financial.refundable_credits,
                    financial.payments,
                    financial.refund,
                    financial.balance
                ),
                (
                    baseline.tax,
                    baseline.refundable_credits,
                    baseline.payments,
                    baseline.refund,
                    baseline.balance
                ),
                "{case}: money"
            );
            if baseline.state == ResultState::Unsupported {
                assert_eq!(financial.state, ResultState::Unsupported);
            }
        }
        SourceScope::Secondary(SecondaryCorrection::SituationChoice) => {
            assert_eq!(saved.tax_refusals, before.tax_refusals);
            assert!(saved.tax_refusals.contains(&OtherTaxRefusal::Section965));
            assert_eq!(financial.state, ResultState::Unsupported);
            assert!(
                financial
                    .issues
                    .iter()
                    .any(|issue| issue.display.key == "ISSUE_SITUATION_SECTION965")
            );
        }
        SourceScope::Secondary(_) => {
            assert!(saved.valid_stored());
            supported(&saved);
        }
        SourceScope::Fieldless => unreachable!(),
    }
    let reopened = reopen(&mut lifecycle, &request);
    accepted(&super::send(&mut lifecycle, reopened));
    assert_eq!(lifecycle.draft, saved, "{case}: hot reopen/resave");
    let mut cold = super::restart(saved.clone());
    assert_eq!(cold.draft, saved);
    let reopened = reopen(&mut cold, &request);
    accepted(&super::send(&mut cold, reopened));
    assert_eq!(cold.draft, saved, "{case}: cold reopen/resave");
    assert_eq!(estimate(&cold.draft), financial, "{case}: complete cold result");
    reply
}

pub(in crate::tests) fn current_income_refusal(
    case: &str,
    session: &mut Interview,
    request: P,
    key: &str,
    path: &str,
) -> Reply {
    current_source_refusal(case, session, request, key, Some(path), SourceScope::Income)
}

pub(in crate::tests) fn current_document_refusal(
    case: &str,
    session: &mut Interview,
    documents: Vec<IncomeDocumentItem>,
    key: &str,
    path: &str,
) -> Reply {
    let topic = super::income_pages::topic_for(&documents[0].document);
    let mut request = open_topic(session, topic);
    let P::SaveIncomes { documents: page, .. } = &mut request else { unreachable!() };
    page.documents = documents;
    current_income_refusal(case, session, request, key, path)
}
pub(in crate::tests) fn current_income_key(
    case: &str,
    draft: &FederalDraft,
    documents: Vec<IncomeDocumentItem>,
    key: &str,
    path: &str,
) -> Vec<u8> {
    let mut session = Interview::new(TEST_USER, draft.clone());
    let Reply::Response(P::Problem { message, .. }) =
        current_document_refusal(case, &mut session, documents, key, path)
    else {
        unreachable!()
    };
    message
}

#[inline(never)]
pub(in crate::tests) fn current_source_acceptance(
    case: &str,
    session: &mut Interview,
    request: P,
) -> Reply {
    let before = session.draft.clone();
    let dirty = session.dirty;
    let calculated = super::send(session, source_calculation(&request));
    if matches!(&request, P::SaveIncomes { .. }) {
        assert!(
            matches!(calculated, Reply::Response(P::IncomeRunningEstimate { .. })),
            "{case}: accepted source returns the income estimate"
        );
    } else {
        accepted(&calculated);
    }
    assert_eq!(session.draft, before, "{case}: positive estimate is transient");
    assert_eq!(session.dirty, dirty);
    let reply = super::send(session, request.clone());
    accepted(&reply);
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    let result = estimate(&saved);
    let reopened = reopen(session, &request);
    accepted(&super::send(session, reopened));
    assert_eq!(session.draft, saved, "{case}: hot reopen/resave");
    let mut cold = super::restart(saved.clone());
    assert_eq!(cold.draft, saved);
    let reopened = reopen(&mut cold, &request);
    accepted(&super::send(&mut cold, reopened));
    assert_eq!(cold.draft, saved, "{case}: cold reopen/resave");
    assert_eq!(estimate(&cold.draft), result, "{case}: complete cold result");
    reply
}
