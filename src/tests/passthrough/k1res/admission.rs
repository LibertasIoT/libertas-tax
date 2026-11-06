use crate::tests::cross::interview::{accepted, document};
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::client::income_pages::{open_document_center, save_documents};
use crate::tests::support::{assert::k1res::*, builders::k1res::*, client::k1res::*};
use crate::tests::*;

#[test]
fn activity_printed_correction_retains_allocations_until_actual_topic_reconciles_them() {
    use crate::tests::support::client::income_pages::open_topic;
    for scorp in [false, true] {
        let (mut fresh, request) = initial(scorp);
        accepted(&send(&mut fresh, request));
        let source = crate::k1_activity::source(&fresh.draft.income[0].income).unwrap();
        assert!(source.components.iter().all(|row| row.regular.is_none()
            && row.risk_group.is_none()
            && row.passive_group.is_none()));
        assert!(source.risk_groups.is_empty() && source.passive_groups.is_empty());
        assert!(source.qbi_blocks.iter().all(|row| row.allocations.is_none()));
        let mut session = accepted_session(scorp);
        let entry = session.draft.income[0].clone();
        let allocations = crate::k1_activity::source(&entry.income).unwrap().clone();
        let mut correction = open_document_center(&mut session);
        match &mut documents(&mut correction)[0].document {
            crate::K1Document::K1PartnershipActivities(v) => v.current.ordinary += 1,
            crate::K1Document::K1ScorpActivities(v) => v.current.ordinary += 1,
            _ => unreachable!(),
        }
        accepted(&send(&mut session, correction));
        assert!(session.draft.valid_stored());
        assert_eq!(crate::k1_activity::source(&session.draft.income[0].income), Some(&allocations));
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        let P::SaveIncomes { followups, .. } = &request else { unreachable!() };
        let row = followups
            .iter()
            .find(|row| row.key.starts_with(&alloc::format!("income/{}/", entry.id)))
            .expect("the accepted source has its named topic question");
        let prompt = crate::Message::decode(&row.prompt);
        let crate::MessageArg::Nested(document) = &prompt.args[1] else {
            panic!("the topic prompt retains its physical document")
        };
        let pending = crate::Message::new(
            "DOCUMENT_PENDING_ANSWER",
            alloc::vec![
                crate::MessageArg::Resource("K1_ACTIVITY_CURRENT_SOURCE_RECONCILE".into()),
                crate::MessageArg::Encoded(document.encode()),
            ],
        )
        .encode();
        let result = crate::federal_rules::estimate(&session.draft);
        assert!(
            crate::interview::remaining_items(&session.draft, &result, TaxSection::PassThrough)
                .iter()
                .any(|row| row.section == TaxSection::PassThrough && row.item == pending)
        );
        let before = session.draft.clone();
        assert_atomic_rejection(
            &mut session,
            request.clone(),
            "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE",
        );
        assert_eq!(session.draft, before);
        let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
        for row in followups {
            let graph = match &mut row.answer {
                crate::DocumentAnswer::K1PartnerActivityRecords { value } => {
                    value.activities.as_mut()
                }
                crate::DocumentAnswer::K1ScorpActivityRecords { value } => {
                    value.activities.as_mut()
                }
                _ => None,
            };
            if let Some(graph) = graph {
                graph.components[0].regular.as_mut().unwrap().operating += 1;
            }
        }
        accepted(&send(&mut session, request));
        let corrected = crate::k1_activity::source(&session.draft.income[0].income).unwrap();
        assert_eq!(
            corrected.components[0].regular.as_ref().unwrap().operating,
            allocations.components[0].regular.as_ref().unwrap().operating + 1
        );
        let result = crate::federal_rules::estimate(&session.draft);
        assert!(
            !crate::interview::remaining_items(&session.draft, &result, TaxSection::PassThrough)
                .iter()
                .any(|row| row.item == pending)
        );
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
    }
}

#[test]
fn k1res_nullable_current_and_retained_sources_save_reopen_restart_and_reject_known_errors() {
    for scorp in [false, true] {
        let (mut session, request) = initial_with_sources(scorp, false, true);
        accepted(&send_fixture(&mut session, request));
        let saved = session.draft.clone();
        let source = crate::k1_activity::source(&saved.income[0].income).unwrap();
        let amounts = source.components[0].regular.as_ref().unwrap();
        assert_eq!(amounts.operating, 400_000);
        assert_eq!(source.components[1].regular.as_ref().unwrap().operating, -600_000);
        assert_eq!(
            (
                amounts.operating_carry.basis,
                amounts.operating_carry.risk,
                amounts.passive_operating
            ),
            (Some(10_000), None, Some(5_000))
        );
        assert_eq!(
            amounts.property.as_ref().unwrap(),
            &K1StatementProperty {
                net: 12_345,
                unrecaptured: 54_321,
                basis_loss: None,
                risk_loss: Some(2_000),
                passive_loss: None
            }
        );
        let pooled = match &source.risk_groups[0].source {
            K1RiskSource::Partnership { regular, .. } => regular.prior_loss,
            K1RiskSource::Scorp { regular, .. } => regular.prior_loss,
            _ => unreachable!(),
        };
        assert_eq!(pooled, 50_000);
        assert_eq!(restart(saved.clone()).draft, saved);
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        assert_eq!(documents(&mut reopened)[0].document, expected_input(&saved, &saved.income[0]));
        accepted(&send_fixture(&mut session, reopened));
        assert_eq!(session.draft.income, saved.income);
        let mut bad =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        graph(&mut documents(&mut bad)[0].document).components[0]
            .regular
            .as_mut()
            .unwrap()
            .operating += 1;
        assert_atomic_rejection(&mut session, bad, "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE");
        assert!(crate::federal_rules::estimate(&session.draft).tax.is_none());
    }
}

#[test]
fn k1res_nullable_entity_basis_keeps_current_facts_and_checks_known_values_and_completion() {
    for scorp in [false, true] {
        let (mut session, request) = initial_with_sources(scorp, false, true);
        accepted(&send_fixture(&mut session, request));
        let saved = session.draft.income[0].clone();
        let mut restarted = restart(session.draft.clone());
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut restarted);
        assert_eq!(documents(&mut request)[0].document, expected_input(&restarted.draft, &saved));
        {
            let (opening, prior, complete) = match &mut documents(&mut request)[0].document {
                crate::K1Document::K1PartnershipActivities(v) => {
                    let regular = v.entity.regular.as_mut().unwrap();
                    (&mut regular.opening, &mut regular.prior_nondeductible, &mut regular.complete)
                }
                crate::K1Document::K1ScorpActivities(v) => {
                    let regular = v.entity.regular.as_mut().unwrap();
                    (&mut regular.opening, &mut regular.prior_nondeductible, &mut regular.complete)
                }
                _ => unreachable!(),
            };
            assert_eq!((*opening, *prior, *complete), (None, None, false));
            *complete = true;
            let (key, field) = ("K1_ACTIVITY_COMPLETE_BASIS_SOURCE_MISSING", "complete");
            assert_located_rejection(
                &mut restarted,
                request,
                key,
                &alloc::format!(
                    "[documents/common/k1/documents/0/document/entity/regular/{field}]"
                ),
            );
            request =
                crate::tests::support::client::income_pages::open_document_center(&mut restarted);
        }
        match &mut documents(&mut request)[0].document {
            crate::K1Document::K1PartnershipActivities(v) => {
                v.entity.regular.as_mut().unwrap().opening = Some(0);
                v.entity.regular.as_mut().unwrap().prior_nondeductible = Some(0);
                v.entity.regular.as_mut().unwrap().complete = true;
            }
            crate::K1Document::K1ScorpActivities(v) => {
                v.entity.regular.as_mut().unwrap().opening = Some(0);
                v.entity.regular.as_mut().unwrap().prior_nondeductible = Some(0);
                v.entity.regular.as_mut().unwrap().complete = true;
            }
            _ => unreachable!(),
        }
        accepted(&send_fixture(&mut restarted, request));
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut restarted);
        let (opening, prior) = match &documents(&mut reopened)[0].document {
            crate::K1Document::K1PartnershipActivities(v) => (
                v.entity.regular.as_ref().unwrap().opening,
                v.entity.regular.as_ref().unwrap().prior_nondeductible,
            ),
            crate::K1Document::K1ScorpActivities(v) => (
                v.entity.regular.as_ref().unwrap().opening,
                v.entity.regular.as_ref().unwrap().prior_nondeductible,
            ),
            _ => unreachable!(),
        };
        assert_eq!((opening, prior), (Some(0), Some(0)));
        match &mut documents(&mut reopened)[0].document {
            crate::K1Document::K1PartnershipActivities(v) => {
                let mut alternative = v.entity.regular.as_ref().unwrap().clone();
                alternative.opening = None;
                alternative.complete = false;
                v.entity.alternative = Some(alternative);
            }
            crate::K1Document::K1ScorpActivities(v) => {
                let mut alternative = v.entity.regular.as_ref().unwrap().clone();
                alternative.opening = None;
                alternative.complete = false;
                v.entity.alternative = Some(alternative);
            }
            _ => unreachable!(),
        }
        accepted(&send_fixture(&mut restarted, reopened));
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut restarted);
        let (opening, prior) = match &mut documents(&mut request)[0].document {
            crate::K1Document::K1PartnershipActivities(v) => {
                let alternative = v.entity.alternative.as_mut().unwrap();
                (alternative.opening, &mut alternative.prior_nondeductible)
            }
            crate::K1Document::K1ScorpActivities(v) => {
                let alternative = v.entity.alternative.as_mut().unwrap();
                (alternative.opening, &mut alternative.prior_nondeductible)
            }
            _ => unreachable!(),
        };
        assert_eq!((opening, *prior), (None, Some(0)));
        assert!(crate::federal_rules::estimate(&restarted.draft).tax.is_none());
    }
}

#[test]
fn k1res_native_risk_sources_reuse_actual_intrinsic_owner_and_keep_context_uncomputed() {
    for scorp in [false, true] {
        let (mut session, request) = initial_with_sources(scorp, false, true);
        accepted(&send_fixture(&mut session, request));
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let start = match &mut graph(&mut documents(&mut request)[0].document).risk_groups[0].source
        {
            K1RiskSource::Partnership { regular, .. } => &mut regular.start,
            K1RiskSource::Scorp { regular, .. } => &mut regular.start,
            _ => unreachable!(),
        };
        *start = ScorpRiskStart::Detailed { capacity: 100, profit: 0, loss: 101 };
        // 2026-10-19 MoneyPrecision: native retained source history is exact cents.
        assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_SOURCE_AMOUNT_INVALID");
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        match &mut graph(&mut documents(&mut request)[0].document).risk_groups[0].source {
            K1RiskSource::Partnership { regular, alternative } => {
                let mut source = regular.clone();
                source.recaptured = 1;
                source.deducted = 0;
                *alternative = Some(source);
            }
            K1RiskSource::Scorp { regular, alternative } => {
                let mut source = regular.clone();
                source.recaptured = 1;
                source.deducted = 0;
                *alternative = Some(source);
            }
            _ => unreachable!(),
        }
        assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_SOURCE_AMOUNT_INVALID");
        assert!(crate::federal_rules::estimate(&session.draft).tax.is_none());
    }
}

#[test]
fn k1res_native_first_save_assigns_global_ids_then_reopens_actual_role_choices() {
    for scorp in [false, true] {
        let (mut session, request) = initial(scorp);
        let before_counter = session.draft.next_id;
        accepted(&send_fixture(&mut session, request));
        let entry = session.draft.income[0].clone();
        let ids = crate::k1_activity::source_ids(&entry.income);
        assert_eq!(ids.len(), 8);
        assert_eq!(entry.id, before_counter);
        let mut ordered_ids = ids.clone();
        ordered_ids.sort_unstable();
        assert_eq!(ordered_ids, (before_counter + 1..before_counter + 9).collect::<Vec<_>>());
        assert_eq!(session.draft.next_id, before_counter + 9);
        let source = crate::k1_activity::source(&entry.income).unwrap();
        assert_eq!(source.components[0].regular.as_ref().unwrap().operating, 400_000);
        assert_eq!(source.components[1].regular.as_ref().unwrap().operating, -600_000);
        assert!(
            source.components.iter().all(
                |component| component.risk_group.is_none() && component.passive_group.is_none()
            )
        );
        assert!(source.risk_groups.iter().all(|v| v.investment_parts.is_none()));
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let row = &mut documents(&mut reopened)[0];
        assert_eq!(row.id, entry.id);
        assert_eq!(
            page(&mut row.document).choices,
            K1ActivityChoiceContext::from_sources(Some(source))
        );
        assert_eq!(row.document, expected_input(&session.draft, &entry));
        let estimate = crate::federal_rules::estimate(&session.draft);
        assert!(
            estimate
                .issues
                .iter()
                .any(|v| v.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
        );
        assert!(estimate.tax.is_none());
        let accepted_draft = session.draft.clone();
        let mut restarted = restart(accepted_draft.clone());
        let mut restored =
            crate::tests::support::client::income_pages::open_document_center(&mut restarted);
        assert_eq!(documents(&mut restored)[0].document, row.document);
        assert_eq!(restarted.draft.income, accepted_draft.income);
    }
}

#[test]
fn k1res_native_second_save_binds_actual_roles_and_keeps_ids_through_reorder_restart() {
    for scorp in [false, true] {
        let mut session = accepted_session(scorp);
        let old_entry = session.draft.income[0].clone();
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let source = graph(&mut documents(&mut request)[0].document);
        for i in 0..2 {
            source.components[i].risk_group = Some(source.risk_groups[i].id);
            source.components[i].passive_group = Some(source.passive_groups[1 - i].id);
            source.qbi_blocks[i].allocations = Some(vec![K1QbiAllocationPage {
                component: source.components[i].id,
                qualified: K1ComponentQualified::Unavailable,
            }]);
        }
        source.components.reverse();
        source.risk_groups.reverse();
        source.passive_groups.reverse();
        source.qbi_blocks.reverse();
        source.components[0].name = "Renamed actual B".into();
        accepted(&send_fixture(&mut session, request));
        let entry = session.draft.income[0].clone();
        assert_eq!(entry.id, old_entry.id);
        let mut old_ids = crate::k1_activity::source_ids(&old_entry.income);
        let mut new_ids = crate::k1_activity::source_ids(&entry.income);
        old_ids.sort_unstable();
        new_ids.sort_unstable();
        assert_eq!(new_ids, old_ids);
        let source = crate::k1_activity::source(&entry.income).unwrap();
        assert_eq!(source.components[0].regular.as_ref().unwrap().operating, -600_000);
        assert_eq!(source.components[0].name, "Renamed actual B");
        assert!(
            source.qbi_blocks.iter().all(|v| v.allocations.as_ref().unwrap()[0].qualified
                == K1ComponentQualified::Unavailable)
        );
        let mut restarted = restart(session.draft.clone());
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut restarted);
        assert_eq!(documents(&mut request)[0].document, expected_input(&session.draft, &entry));
        accepted(&send_fixture(&mut restarted, request));
        assert_eq!(restarted.draft.income[0], entry);
    }
}

#[test]
fn k1res_native_rejects_guessed_ids_and_references_before_first_save_atomically() {
    for scorp in [false, true] {
        let (mut session, mut request) = initial(scorp);
        graph(&mut documents(&mut request)[0].document).components[0].id =
            session.draft.next_id + 1;
        assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_SOURCE_ID_NOT_ACCEPTED");
        let (mut session, mut request) = initial(scorp);
        graph(&mut documents(&mut request)[0].document).components[0].risk_group =
            Some(session.draft.next_id + 3);
        assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED");
    }
}

#[test]
fn k1res_native_rejects_deleted_role_wrong_role_and_repeated_ids() {
    for scorp in [false, true] {
        for case in 0..3 {
            let mut session = accepted_session(scorp);
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let document = &mut documents(&mut request)[0].document;
            let key = match case {
                0 => {
                    let source = graph(document);
                    source.components[0].risk_group = Some(source.risk_groups[0].id);
                    source.risk_groups.remove(0);
                    "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED"
                }
                1 => {
                    let source = graph(document);
                    source.components[0].risk_group = Some(source.passive_groups[0].id);
                    "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED"
                }
                2 => {
                    let source = graph(document);
                    source.components[1].id = source.components[0].id;
                    "K1_ACTIVITY_SOURCE_ID_REPEATED"
                }
                _ => unreachable!(),
            };
            assert_atomic_rejection(&mut session, request, key);
        }
    }
}

#[test]
fn k1res_native_known_source_contradiction_is_atomic_while_unknown_facts_are_saved() {
    for scorp in [false, true] {
        let mut session = accepted_session(scorp);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        match &mut documents(&mut request)[0].document {
            crate::K1Document::K1PartnershipActivities(v) => v.current.ordinary += 1,
            crate::K1Document::K1ScorpActivities(v) => v.current.ordinary += 1,
            _ => unreachable!(),
        }
        assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE");
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        graph(&mut documents(&mut request)[0].document).entity_only = None;
        graph(&mut documents(&mut request)[0].document).alternative_entity_only =
            K1EntityOnlyAmt::Unavailable;
        accepted(&send_fixture(&mut session, request));
        let source = crate::k1_activity::source(&session.draft.income[0].income).unwrap();
        assert!(source.entity_only.is_none());
        assert_eq!(source.alternative_entity_only, K1EntityOnlyAmt::Unavailable);
        assert!(source.components.iter().all(|v| v.qualified_current_controls.ordinary.is_none()));
        assert!(crate::federal_rules::estimate(&session.draft).tax.is_none());
    }
}

#[test]
fn k1res_native_loan_source_reuses_canonical_note_rules_and_same_loan_amt_bound() {
    let loan = crate::ScorpLoan {
        kind: crate::ScorpLoanKind::FormalNote,
        repayment_term: crate::ScorpNoteHolding::Dated {
            advanced: 20250115,
            payments: vec![crate::ScorpNoteRepayment { date: 20260201, principal: 10_000 }],
        },
        principal: 100_000,
        basis: 50_000,
        advances: 0,
        repayments: 10_000,
    };
    for case in [0, 1, 3, 4] {
        let (mut session, mut request) = initial(true);
        let crate::K1Document::K1ScorpActivities(page) = &mut documents(&mut request)[0].document
        else {
            unreachable!()
        };
        page.entity
            .loans
            .push(ScorpEntityLoan { regular: loan.clone(), alternative_basis: Some(100_000) });
        match case {
            0 => {
                accepted(&send_fixture(&mut session, request));
                let FederalIncome::ScorpActivities { data } = &session.draft.income[0].income
                else {
                    unreachable!()
                };
                assert_eq!(data.entity.loans[0].regular, loan);
                assert_eq!(data.entity.loans[0].alternative_basis, Some(100_000));
                continue;
            }
            1 => page.entity.loans[0].alternative_basis = Some(100_001),
            3 => page.entity.loans[0].regular.repayments = 10_001,
            _ => page.entity.loans[0].regular.repayment_term = crate::ScorpNoteHolding::NoRepayment,
        }
        assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_SOURCE_AMOUNT_INVALID");
    }
}

#[test]
fn k1res_native_properties_use_delivered_global_identity_and_native_reduction_once() {
    for scorp in [false, true] {
        let (mut session, request) = initial_with_properties(scorp, true);
        let first_id = session.draft.next_id;
        accepted(&send_fixture(&mut session, request));
        let entry = &session.draft.income[0];
        let property_ids = crate::investment_credit::k1_property_ids(&entry.income);
        assert_eq!(property_ids.len(), 2);
        assert_ne!(property_ids[0], property_ids[1]);
        assert!(property_ids.iter().all(|id| *id > first_id && *id < session.draft.next_id));
        let component_ids: Vec<_> = crate::k1_activity::source(&entry.income)
            .unwrap()
            .components
            .iter()
            .map(|row| row.id)
            .collect();
        let source = crate::investment_credit::k1_property_credit_sources(&entry.income).unwrap();
        assert_eq!(source.len(), 2);
        assert!(
            crate::k1_activity::source(&entry.income)
                .unwrap()
                .investment_property_sources
                .is_none()
        );
        assert_eq!(
            source.iter().map(|v| v.basis_decrease_cents).collect::<Vec<_>>(),
            [150_000, 300_000]
        );
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let document = &mut documents(&mut request)[0].document;
        assert_eq!(page(document).choices.properties.len(), 2);
        bind_properties(document);
        accepted(&send_fixture(&mut session, request));
        let accepted_entry = session.draft.income[0].clone();
        let source =
            crate::investment_credit::k1_property_credit_sources(&accepted_entry.income).unwrap();
        assert_eq!(
            source
                .iter()
                .map(|property| {
                    crate::k1_activity::source(&accepted_entry.income)
                        .unwrap()
                        .investment_property_sources
                        .as_ref()
                        .unwrap()
                        .iter()
                        .find(|row| row.property == property.property_id)
                        .unwrap()
                        .component
                })
                .collect::<Vec<_>>(),
            component_ids
        );
        assert_eq!(accepted_entry.id, first_id);
        assert_eq!(source.iter().map(|v| v.basis_decrease_cents).sum::<i64>(), 450_000);
        let mut restarted = restart(session.draft.clone());
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut restarted);
        let document = &mut documents(&mut request)[0].document;
        graph(document).components.reverse();
        let properties = match document {
            crate::K1Document::K1PartnershipActivities(v) => &mut v.entity.investment_property,
            crate::K1Document::K1ScorpActivities(v) => &mut v.entity.investment_property,
            _ => unreachable!(),
        };
        properties.reverse();
        if let crate::K1InvestmentProperty::Energy { basis, .. } = &mut properties[1] {
            *basis = 1_100_001;
        }
        accepted(&send_fixture(&mut restarted, request));
        let source =
            crate::investment_credit::k1_property_credit_sources(&restarted.draft.income[0].income)
                .unwrap();
        assert_eq!(
            source.iter().map(|v| v.property_id).collect::<Vec<_>>(),
            [property_ids[1], property_ids[0]]
        );
        assert_eq!(
            source
                .iter()
                .map(|property| {
                    crate::k1_activity::source(&restarted.draft.income[0].income)
                        .unwrap()
                        .investment_property_sources
                        .as_ref()
                        .unwrap()
                        .iter()
                        .find(|row| row.property == property.property_id)
                        .unwrap()
                        .component
                })
                .collect::<Vec<_>>(),
            [component_ids[1], component_ids[0]]
        );
        assert_eq!(
            source.iter().map(|v| v.basis_decrease_cents).collect::<Vec<_>>(),
            [300_000, 165_000]
        );
        assert_eq!(session.draft.income[0], accepted_entry);
    }
}

// The delivered storage-110 Page uses conditional code/line payloads, while
// the accepted entity retains one canonical source list and its actual bool.
#[test]
fn k1res_canonical_credit_pages_preserve_printed_lines_and_confirmation_through_save_restart() {
    for scorp in [false, true] {
        let (mut session, mut request) = initial(scorp);
        let credits = current_credit_sources();
        match &mut documents(&mut request)[0].document {
            crate::K1Document::K1PartnershipActivities(v) => {
                v.entity.credits = credits.iter().cloned().map(Into::into).collect();
                v.entity.investment_confirmed = Some(crate::K1InvestmentConfirmation::No);
                v.entity.housing_confirmed = Some(true);
            }
            crate::K1Document::K1ScorpActivities(v) => {
                v.entity.credits = credits.iter().cloned().map(Into::into).collect();
                v.entity.investment_confirmed = Some(crate::K1InvestmentConfirmation::No);
                v.entity.housing_confirmed = Some(true);
            }
            _ => unreachable!(),
        }
        accepted(&send_fixture(&mut session, request));
        let entry = session.draft.income[0].clone();
        let (saved, confirmed) = match &entry.income {
            FederalIncome::PartnershipActivities { data } => {
                (&data.entity.credits, data.entity.investment_confirmed)
            }
            FederalIncome::ScorpActivities { data } => {
                (&data.entity.credits, data.entity.investment_confirmed)
            }
            _ => unreachable!(),
        };
        assert_eq!(saved, &credits);
        assert_eq!(confirmed, None);
        assert_eq!(saved.len(), 10);
        let mut restarted = restart(session.draft.clone());
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut restarted);
        assert_eq!(documents(&mut request)[0].document, expected_input(&session.draft, &entry));
        accepted(&send_fixture(&mut restarted, request.clone()));
        assert_eq!(restarted.draft.income[0], entry);
        for duplicate in [false, true] {
            request =
                crate::tests::support::client::income_pages::open_document_center(&mut restarted);
            let credits = match &mut documents(&mut request)[0].document {
                crate::K1Document::K1PartnershipActivities(v) => &mut v.entity.credits,
                crate::K1Document::K1ScorpActivities(v) => &mut v.entity.credits,
                _ => unreachable!(),
            };
            if duplicate {
                credits.push(credits[0].clone());
            } else {
                credits[9].code = crate::K1CreditCodePage::Research {
                    line: crate::BusinessCredit::WorkOpportunity,
                };
            }
            assert_located_rejection(
                &mut restarted,
                request,
                "K1_CREDIT_CODE_TWICE",
                &alloc::format!(
                    "[documents/common/k1/documents/0/document/entity/credits/{}/code]",
                    if duplicate { current_credit_sources().len() } else { 9 }
                ),
            );
        }
        assert!(crate::federal_rules::estimate(&restarted.draft).tax.is_none());
    }
}

#[test]
fn k1res_source_catalogs_use_actual_entity_ids_and_known_research_once() {
    for scorp in [false, true] {
        let (mut session, mut request) = initial(scorp);
        match &mut documents(&mut request)[0].document {
            crate::K1Document::K1PartnershipActivities(v) => {
                v.entity.credits = current_credit_sources().into_iter().map(Into::into).collect();
                v.entity.housing_confirmed = Some(true);
            }
            crate::K1Document::K1ScorpActivities(v) => {
                v.entity.credits = current_credit_sources().into_iter().map(Into::into).collect();
                v.entity.housing_confirmed = Some(true);
            }
            _ => unreachable!(),
        }
        accepted(&send_fixture(&mut session, request));
        let id = session.draft.income[0].id;
        for catalog in [
            crate::business_credit_page::research_businesses(&session.draft),
            crate::investment_interest::k1_sources(&session.draft),
        ] {
            assert_eq!(catalog.iter().map(|v| v.id).collect::<Vec<_>>(), [id]);
        }
        assert_eq!(crate::retained_research::available(&session.draft), [0]);
        assert_eq!(crate::investment_interest::origin_choices(&session.draft), [0]);
        assert_eq!(crate::retained_charity::origin_choices(&session.draft), [0, 1]);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        match &mut documents(&mut request)[0].document {
            crate::K1Document::K1PartnershipActivities(v) => v.entity.credits.clear(),
            crate::K1Document::K1ScorpActivities(v) => v.entity.credits.clear(),
            _ => unreachable!(),
        }
        accepted(&send_fixture(&mut session, request));
        assert!(crate::retained_research::available(&session.draft).is_empty());
        assert!(crate::investment_interest::origin_choices(&session.draft).is_empty());
        assert_eq!(
            crate::investment_interest::k1_sources(&session.draft)
                .iter()
                .map(|v| v.id)
                .collect::<Vec<_>>(),
            [id]
        );
        assert!(crate::federal_rules::estimate(&session.draft).tax.is_none());
    }
}

#[test]
fn k1res_w2_actual_new_scorp_source_survives_save_restart_and_reconciles_deletion() {
    use crate::tests::support::client::income_pages::open_topic;
    for scorp in [false, true] {
        let mut session = accepted_session(scorp);
        let source_id = session.draft.income[0].id;
        let request = open_document_center(&mut session);
        let P::SaveDocuments { wage_scorporations, allowed_wage_scorporations, .. } = &request
        else {
            unreachable!()
        };
        assert_eq!(
            wage_scorporations.iter().map(|v| v.id).collect::<Vec<_>>(),
            if scorp { vec![source_id] } else { vec![] }
        );
        assert_eq!(allowed_wage_scorporations, &if scorp { vec![0] } else { vec![] });
        let form = crate::tests::cross::interview::w2(4_000_000);
        let request = save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(form)) },
            )],
        );
        accepted(&send_fixture(&mut session, request));
        if !scorp {
            let wage = session.draft.income.last().unwrap();
            let FederalIncome::Wage { data } = &wage.income else { unreachable!() };
            assert_eq!(FederalWage::from(data).paying_scorporation, None);
            continue;
        }
        let wage_id = session.draft.income.last().unwrap().id;
        let mut request = open_topic(&mut session, TaxSection::Wages);
        let P::SaveWagesAnswers { wages, .. } = &mut request else { unreachable!() };
        let row = wages
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{wage_id}/wage"))
            .unwrap();
        let answers = &mut row.answers;
        answers.paying_scorporation =
            Some(crate::WageScorpEmployerView::Corporation { source: source_id });
        accepted(&send(&mut session, request));
        let mut restarted = restart(session.draft.clone());
        let mut paper = open_document_center(&mut restarted);
        let P::SaveDocuments { documents, .. } = &mut paper else { unreachable!() };
        let wage = documents.common.wages.documents.iter().find(|row| row.id == wage_id).unwrap();
        assert_eq!(wage.id, wage_id);
        let mut request = open_topic(&mut restarted, TaxSection::Wages);
        let P::SaveWagesAnswers { wages, .. } = &mut request else { unreachable!() };
        let row = wages
            .iter_mut()
            .find(|row| row.key == alloc::format!("income/{wage_id}/wage"))
            .unwrap();
        let answers = &mut row.answers;
        assert_eq!(
            answers.paying_scorporation,
            Some(crate::WageScorpEmployerView::Corporation { source: source_id })
        );
        answers.paying_scorporation =
            Some(crate::WageScorpEmployerView::Corporation { source: source_id + 100_000 });
        assert_atomic_rejection(&mut restarted, request, "W2_SCORP_EMPLOYER_UNAVAILABLE");
        let mut request = open_document_center(&mut restarted);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        documents.common.k1.documents.clear();
        accepted(&send_fixture(&mut restarted, request));
        assert_eq!(restarted.draft.income.len(), 1);
        let FederalIncome::Wage { data } = &restarted.draft.income[0].income else {
            unreachable!()
        };
        assert_eq!(FederalWage::from(data).paying_scorporation, None);
        assert_eq!(FederalWage::from(data).wages, 4_000_000);
        let request = open_document_center(&mut restarted);
        let P::SaveDocuments { wage_scorporations, .. } = request else { unreachable!() };
        assert!(wage_scorporations.is_empty());
    }
}

#[test]
fn k1res_native_property_associations_reject_duplicate_deleted_guessed_and_wrong_role() {
    for scorp in [false, true] {
        for case in 0..6 {
            let (mut session, mut request) = initial_with_properties(scorp, true);
            if case == 4 {
                let second = documents(&mut request)[0].clone();
                documents(&mut request).push(second);
            }
            accepted(&send_fixture(&mut session, request));
            let foreign = if case == 4 {
                Some(crate::investment_credit::k1_property_ids(&session.draft.income[1].income)[0])
            } else {
                None
            };
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let document = &mut documents(&mut request)[0].document;
            bind_properties(document);
            let key = match case {
                0 => {
                    let rows = graph(document).investment_property_sources.as_mut().unwrap();
                    rows[1].property = rows[0].property;
                    "K1_ACTIVITY_PROPERTY_SOURCE_TWICE"
                }
                1 => {
                    match document {
                        crate::K1Document::K1PartnershipActivities(v) => {
                            v.entity.investment_property.remove(0);
                        }
                        crate::K1Document::K1ScorpActivities(v) => {
                            v.entity.investment_property.remove(0);
                        }
                        _ => unreachable!(),
                    }
                    "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED"
                }
                2 => {
                    graph(document).investment_property_sources.as_mut().unwrap()[0].property =
                        session.draft.next_id;
                    "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED"
                }
                3 => {
                    let source = graph(document);
                    source.investment_property_sources.as_mut().unwrap()[0].component =
                        source.risk_groups[0].id;
                    "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED"
                }
                4 => {
                    graph(document).investment_property_sources.as_mut().unwrap()[0].property =
                        foreign.unwrap();
                    "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED"
                }
                _ => {
                    let properties = match document {
                        crate::K1Document::K1PartnershipActivities(v) => {
                            &mut v.entity.investment_property
                        }
                        crate::K1Document::K1ScorpActivities(v) => {
                            &mut v.entity.investment_property
                        }
                        _ => unreachable!(),
                    };
                    *properties[0].id_mut() = 0;
                    "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED"
                }
            };
            assert_atomic_rejection(&mut session, request, key);
        }
    }
}

// Optional complete Avro transport is the same native Save/reopen/restart
// witness. Each callback reconstructs the ordinary source editor and proves
// that the calculation leaves accepted storage unchanged.
#[test]
fn k1res_output_catalog_omission_or_forgery_does_not_change_accepted_sources() {
    use crate::tests::support::client::estimate_documents_from_save as credit_estimate_documents_from_save;
    for scorp in [false, true] {
        for omitted in [false, true] {
            let mut session = accepted_session(scorp);
            let before = session.draft.clone();
            let dirty = session.dirty;
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let choices = &mut page(&mut documents(&mut request)[0].document).choices;
            if omitted {
                *choices = K1ActivityChoiceContext::default();
            } else {
                choices.components[0].id = 999_999;
                choices.risk.clear();
                choices.sales.clear();
            }
            let calculation = credit_estimate_documents_from_save(&request);
            assert!(matches!(
                crate::tests::support::client::income_pages::candidate_save_review(
                    &mut session,
                    calculation
                ),
                Reply::Response(P::Review { .. })
            ));
            assert_eq!(session.draft, before);
            assert_eq!(session.dirty, dirty);
            accepted(&send_fixture(&mut session, request));
            assert_eq!(session.draft, before);
            let mut restored = restart(before.clone());
            let page =
                crate::tests::support::client::income_pages::open_document_center(&mut restored);
            accepted(&send_fixture(&mut restored, page));
            assert_eq!(restored.draft, before);
            assert!(crate::federal_rules::estimate(&restored.draft).tax.is_none());
        }
    }
}

#[test]
fn k1res_canonical_child_refusals_keep_actual_rows_and_atomic_corrections() {
    use crate::tests::support::client::estimate_documents_from_save as credit_estimate_documents_from_save;
    for scorp in [false, true] {
        for case in 0..6 {
            let mut session = accepted_session(scorp);
            let before = session.draft.clone();
            let dirty = session.dirty;
            let good =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let mut bad = good.clone();
            let document = &mut documents(&mut bad)[0].document;
            let (key, local) = match case {
                0 | 1 => {
                    let amounts = graph(document).components[1].regular.as_mut().unwrap();
                    match &mut amounts.items {
                        K1StatementItems::Partnership { income, deductions } => {
                            if case == 0 {
                                let row = PartnerIncome {
                                    code: PartnerIncomeCode::PortfolioIncome,
                                    amount: 100,
                                };
                                *income = vec![row.clone(), row];
                            } else {
                                let row = PartnerDeduction {
                                    code: PartnerDeductionCode::CashThirty,
                                    amount: 100,
                                };
                                *deductions = vec![row.clone(), row];
                            }
                        }
                        K1StatementItems::Scorp { income, deductions } => {
                            if case == 0 {
                                let row = ScorpIncome {
                                    code: ScorpIncomeCode::PortfolioIncome,
                                    amount: 100,
                                };
                                *income = vec![row.clone(), row];
                            } else {
                                let row = ScorpDeduction {
                                    code: ScorpDeductionCode::CashThirty,
                                    amount: 100,
                                };
                                *deductions = vec![row.clone(), row];
                            }
                        }
                    }
                    if case == 0 {
                        (
                            "K1_INCOME_CODE_TWICE",
                            "activities/statement/source/components/1/regular/items/income/1/code",
                        )
                    } else {
                        (
                            "K1_DEDUCTION_CODE_TWICE",
                            "activities/statement/source/components/1/regular/items/deductions/1/code",
                        )
                    }
                }
                2 => {
                    let row: crate::K1CreditPage = current_credit_sources()[0].clone().into();
                    match document {
                        crate::K1Document::K1PartnershipActivities(value) => {
                            value.entity.credits = vec![row.clone(), row]
                        }
                        crate::K1Document::K1ScorpActivities(value) => {
                            value.entity.credits = vec![row.clone(), row]
                        }
                        _ => unreachable!(),
                    }
                    ("K1_CREDIT_CODE_TWICE", "entity/credits/1/code")
                }
                3 => {
                    match document {
                        crate::K1Document::K1PartnershipActivities(value) => {
                            value.special = vec![PartnerSpecial::UnreimbursedExpenses; 2]
                        }
                        crate::K1Document::K1ScorpActivities(value) => {
                            value.special = vec![ScorpSpecial::HousingBefore1990; 2]
                        }
                        _ => unreachable!(),
                    }
                    ("K1_SITUATION_TWICE", "special/1")
                }
                4 => {
                    let mut k3 = crate::tests::support::builders::k3_every_field();
                    k3.items = vec![K3Item::Research; 2];
                    match document {
                        crate::K1Document::K1PartnershipActivities(value) => {
                            value.entity.schedule_k3 = Some(k3)
                        }
                        crate::K1Document::K1ScorpActivities(value) => {
                            value.entity.schedule_k3 = Some(k3)
                        }
                        _ => unreachable!(),
                    }
                    ("K3_ITEM_TWICE", "entity/schedule_k3/items/1")
                }
                _ => {
                    let mut sale = current_sale();
                    let amounts = sale.regular.as_mut().unwrap();
                    amounts.depreciation = amounts.cost;
                    *sale_fields(document).0 = vec![sale];
                    ("K1_SALE_BASIS_BELOW_ZERO", "section179_sales/0/regular/depreciation")
                }
            };
            let path = alloc::format!("[documents/common/k1/documents/0/document/{local}]");
            for (_route, request) in
                [("save", bad.clone()), ("estimate", credit_estimate_documents_from_save(&bad))]
            {
                let request = P::from_avro(&request.to_avro()).unwrap();
                assert_located_rejection(&mut session, request, key, &path);
                assert_eq!(session.draft, before);
                assert_eq!(session.dirty, dirty);
            }
            accepted(&send_fixture(&mut session, good));
            assert_eq!(session.draft, before);
            let mut restored = restart(before.clone());
            let page =
                crate::tests::support::client::income_pages::open_document_center(&mut restored);
            accepted(&send_fixture(&mut restored, page));
            assert_eq!(restored.draft, before);
        }
    }
}
#[test]
fn k1res_source_locators_name_submitted_rows_amounts_roles_and_actual_readonly_sources() {
    let (mut session, mut request) = initial(false);
    let second = documents(&mut request)[0].clone();
    documents(&mut request).push(second);
    let crate::K1Document::K1PartnershipActivities(data) = &mut documents(&mut request)[1].document
    else {
        unreachable!()
    };
    data.current.ordinary += 1;
    assert_located_rejection(
        &mut session,
        request,
        "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE",
        "[documents/common/k1/documents/1/document/current/ordinary]",
    );
    let (mut session, request) = initial(false);
    accepted(&send_fixture(&mut session, request));
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    graph(&mut documents(&mut request)[0].document).components[0].risk_group =
        Some(session.draft.next_id);
    assert_located_rejection(
        &mut session,
        request,
        "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED",
        "[documents/common/k1/documents/0/document/activities/statement/source/components/0/risk_group]",
    );
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let guessed = session.draft.next_id + 99;
    let document = &mut documents(&mut request)[0].document;
    page(document).choices.risk[0].id = guessed;
    graph(document).components[1].risk_group = Some(guessed);
    assert_located_rejection(
        &mut session,
        request,
        "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED",
        "[documents/common/k1/documents/0/document/activities/statement/source/components/1/risk_group]",
    );
}
#[test]
fn k1res_physical_sale_keeps_canonical_availability_identity_and_entity_period_through_correction_restart()
 {
    for scorp in [false, true] {
        let (mut session, mut request) = initial(scorp);
        add_sale_sources(&mut documents(&mut request)[0].document);
        let first_id = session.draft.next_id;
        accepted(&send_fixture(&mut session, request));
        let entry = &session.draft.income[0];
        let sources = crate::k1_activity::section179_sales(&entry.income);
        assert_eq!(sources.len(), 1);
        assert!(sources[0].id > first_id && sources[0].id < session.draft.next_id);
        let component_id = crate::k1_activity::source(&entry.income).unwrap().components[0].id;
        assert_eq!(sources[0].regular, current_sale().regular);
        assert_eq!(sources[0].monetary_view(true), None);
        assert!(
            crate::k1_activity::source(&entry.income).unwrap().section179_sale_sources.is_none()
        );
        let physical_id = sources[0].id;
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let document = &mut documents(&mut request)[0].document;
        assert_eq!(
            page(document).choices.sales.iter().map(|choice| choice.id).collect::<Vec<_>>(),
            [physical_id]
        );
        bind_sales(document);
        accepted(&send_fixture(&mut session, request));
        let mut restarted = restart(session.draft.clone());
        for alternative in [
            K1Section179SaleAmtSource::NotReported,
            K1Section179SaleAmtSource::Reported {
                amounts: K1Section179SaleAmounts {
                    price: 110_005,
                    cost: 210_006,
                    depreciation: 40_007,
                    section179: 25_004,
                },
            },
            K1Section179SaleAmtSource::Unavailable,
        ] {
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut restarted);
            let document = &mut documents(&mut request)[0].document;
            let (sales, period) = sale_fields(document);
            assert_eq!(*period, Some(K1EntityTaxPeriod { began: 20250701, ended: 20260630 }));
            sales[0].description = "Corrected actual description".into();
            sales[0].alternative = alternative.clone();
            graph(document).components.reverse();
            accepted(&send_fixture(&mut restarted, request));
            let source = crate::k1_activity::section179_sales(&restarted.draft.income[0].income);
            assert_eq!(source.len(), 1);
            assert_eq!(source[0].id, physical_id);
            assert_eq!(source[0].regular, current_sale().regular);
            assert_eq!(source[0].alternative, alternative);
            assert_eq!(
                (
                    restarted.draft.income[0].id,
                    crate::k1_activity::source(&restarted.draft.income[0].income)
                        .unwrap()
                        .section179_sale_sources
                        .as_ref()
                        .unwrap()[0]
                        .component
                ),
                (first_id, component_id)
            );
            assert!(crate::federal_rules::estimate(&restarted.draft).tax.is_none());
        }
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut restarted);
        let document = &mut documents(&mut request)[0].document;
        sale_fields(document).0.clear();
        graph(document).section179_sale_sources = Some(Vec::new());
        accepted(&send_fixture(&mut restarted, request));
        assert!(crate::k1_activity::section179_sales(&restarted.draft.income[0].income).is_empty());
    }
}

#[test]
fn k1res_sale_associations_and_entity_period_reject_atomically_with_actual_source_paths() {
    for scorp in [false, true] {
        for case in 0..8 {
            let (mut session, mut request) = initial(scorp);
            add_sale_sources(&mut documents(&mut request)[0].document);
            if case == 4 {
                let row = documents(&mut request)[0].clone();
                documents(&mut request).push(row);
            }
            accepted(&send_fixture(&mut session, request));
            let foreign = (case == 4)
                .then(|| crate::k1_items::k1_sale_ids(&session.draft.income[1].income)[0]);
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let document = &mut documents(&mut request)[0].document;
            bind_sales(document);
            let (key, path) = match case {
                0 => {
                    let rows = graph(document).section179_sale_sources.as_mut().unwrap();
                    rows.push(rows[0].clone());
                    (
                        "K1_ACTIVITY_SALE_SOURCE_TWICE",
                        "[documents/common/k1/documents/0/document/activities/statement/source/section179_sale_sources/1/sale]",
                    )
                }
                1 => {
                    sale_fields(document).0.clear();
                    (
                        "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED",
                        "[documents/common/k1/documents/0/document/activities/statement/source/section179_sale_sources/0/sale]",
                    )
                }
                2 => {
                    graph(document).section179_sale_sources.as_mut().unwrap()[0].sale =
                        session.draft.next_id;
                    (
                        "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED",
                        "[documents/common/k1/documents/0/document/activities/statement/source/section179_sale_sources/0/sale]",
                    )
                }
                3 => {
                    let source = graph(document);
                    source.section179_sale_sources.as_mut().unwrap()[0].component =
                        source.risk_groups[0].id;
                    (
                        "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED",
                        "[documents/common/k1/documents/0/document/activities/statement/source/section179_sale_sources/0/component]",
                    )
                }
                4 => {
                    graph(document).section179_sale_sources.as_mut().unwrap()[0].sale =
                        foreign.unwrap();
                    (
                        "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED",
                        "[documents/common/k1/documents/0/document/activities/statement/source/section179_sale_sources/0/sale]",
                    )
                }
                5 => {
                    sale_fields(document).0[0].id = 0;
                    (
                        "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED",
                        "[documents/common/k1/documents/0/document/activities/statement/source/section179_sale_sources/0/sale]",
                    )
                }
                6 => {
                    *sale_fields(document).1 =
                        Some(K1EntityTaxPeriod { began: 20260701, ended: 20260630 });
                    (
                        "K1_ENTITY_PERIOD_DATES",
                        "[documents/common/k1/documents/0/document/reporting_period]",
                    )
                }
                _ => {
                    let guessed = session.draft.next_id + 99;
                    page(document).choices.sales[0].id = guessed;
                    graph(document).section179_sale_sources.as_mut().unwrap()[0].sale = guessed;
                    (
                        "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED",
                        "[documents/common/k1/documents/0/document/activities/statement/source/section179_sale_sources/0/sale]",
                    )
                }
            };
            assert_located_rejection(&mut session, request, key, path);
        }
        let (mut session, mut request) = initial(scorp);
        add_sale_sources(&mut documents(&mut request)[0].document);
        graph(&mut documents(&mut request)[0].document).section179_sale_sources =
            Some(vec![K1Section179SaleSourcePage {
                sale: session.draft.next_id + 9,
                component: session.draft.next_id + 1,
            }]);
        assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED");
    }
}

#[test]
fn k1res_actual_parent_amt179_history_remains_independent_of_unknown_component_source_and_regular_carry()
 {
    for scorp in [false, true] {
        let (mut session, mut request) = initial(scorp);
        let source = graph(&mut documents(&mut request)[0].document);
        for component in &mut source.components {
            component.alternative = K1StatementAmt::Unavailable;
        }
        source.alternative_entity_only = K1EntityOnlyAmt::Unavailable;
        accepted(&send_fixture(&mut session, request));
        for history in [
            Some(Section179IncomeCarryHistory {
                gross: 50_003,
                sold: 10_001,
                vintages: vec![
                    Asset179Carryover { year: 2024, amount: 30_001 },
                    Asset179Carryover { year: 2025, amount: 10_001 },
                ],
            }),
            Some(Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: Vec::new() }),
            None,
        ] {
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            *amt_income_carry(&mut documents(&mut request)[0].document) = history.clone();
            accepted(&send_fixture(&mut session, request));
            session = restart(session.draft.clone());
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let document = &mut documents(&mut request)[0].document;
            assert_eq!(*amt_income_carry(document), history);
            let source = graph(document);
            assert_eq!(
                source.components[0].regular.as_ref().unwrap().section179_business_carry,
                Some(0)
            );
            assert_eq!(source.components[0].regular.as_ref().unwrap().operating, 400_000);
            assert_eq!(source.components[1].regular.as_ref().unwrap().operating, -600_000);
            assert!(crate::federal_rules::estimate(&session.draft).tax.is_none());
        }
        for (history, key) in [
            (
                Section179IncomeCarryHistory { gross: 10_000, sold: 10_001, vintages: Vec::new() },
                "K1_AMT_179_CARRY_BOUNDS",
            ),
            (
                Section179IncomeCarryHistory {
                    gross: 10_000,
                    sold: 0,
                    vintages: vec![Asset179Carryover { year: 2025, amount: 9_999 }],
                },
                "K1_179_VINTAGE_TOTAL",
            ),
            (
                Section179IncomeCarryHistory {
                    gross: 10_000,
                    sold: 0,
                    vintages: vec![Asset179Carryover { year: 2026, amount: 10_000 }],
                },
                "K1_179_VINTAGE_BOUNDS",
            ),
            (
                Section179IncomeCarryHistory {
                    gross: 10_000,
                    sold: 0,
                    vintages: vec![
                        Asset179Carryover { year: 2025, amount: 5_000 },
                        Asset179Carryover { year: 2025, amount: 5_000 },
                    ],
                },
                "K1_179_VINTAGE_TWICE",
            ),
        ] {
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            *amt_income_carry(&mut documents(&mut request)[0].document) = Some(history);
            assert_located_rejection(
                &mut session,
                request,
                key,
                &alloc::format!(
                    "[documents/common/k1/documents/0/document/entity/amt_section179_income_carry/{}]",
                    match key {
                        "K1_AMT_179_CARRY_BOUNDS" => "sold",
                        "K1_179_VINTAGE_TOTAL" => "vintages",
                        "K1_179_VINTAGE_BOUNDS" => "vintages/0/year",
                        "K1_179_VINTAGE_TWICE" => "vintages/1/year",
                        _ => unreachable!(),
                    }
                ),
            );
        }
    }
}

#[test]
fn k1res_long_source_names_save_reopen_and_restart() {
    // Long UTF-8 source names remain editable and survive storage.
    let name = "é😀e\u{301}".repeat(100);
    let (mut session, mut request) = initial(false);
    let crate::K1Document::K1PartnershipActivities(data) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    data.partnership.name = name.clone();
    let sources = graph(&mut documents(&mut request)[0].document);
    sources.components[0].name = name.clone();
    sources.qbi_blocks[0].name = name.clone();
    accepted(&send_fixture(&mut session, request));
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut restored);
    assert_eq!(graph(&mut documents(&mut request)[0].document).components[0].name, name);
    accepted(&send_fixture(&mut restored, request));
    assert_eq!(restored.draft, saved);
}

#[test]
fn k1res_physical_sale_reference_retains_actual_global_ids_above_money_source_limits() {
    let (mut session, mut request) = initial(false);
    session.draft.next_id = 100_000_000_001;
    add_sale_sources(&mut documents(&mut request)[0].document);
    accepted(&send_fixture(&mut session, request));
    let physical_id = crate::k1_items::k1_sale_ids(&session.draft.income[0].income)[0];
    assert!(physical_id > 100_000_000_000);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    bind_sales(&mut documents(&mut request)[0].document);
    accepted(&send_fixture(&mut session, request));
    let restarted = restart(session.draft.clone());
    let sources = crate::k1_activity::section179_sales(&restarted.draft.income[0].income);
    assert_eq!(sources[0].id, physical_id);
    assert_eq!(
        (
            restarted.draft.income[0].id,
            crate::k1_activity::source(&restarted.draft.income[0].income)
                .unwrap()
                .section179_sale_sources
                .as_ref()
                .unwrap()[0]
                .component
        ),
        (100_000_000_001, 100_000_000_002)
    );
    assert_eq!(sources[0].regular, current_sale().regular);
}

#[test]
fn k1res_arithmetic_shared_concrete_history_above_source_keeps_scalar_priority_and_correction() {
    let mut session = accepted_session(false);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let source = graph(&mut documents(&mut request)[0].document);
    let component = source.components[0].id;
    source.components[0].regular.as_mut().unwrap().operating_carry.basis = Some(100_000_000_000);
    for block in &mut source.qbi_blocks {
        block.allocations = Some(vec![K1QbiAllocationPage {
            component,
            qualified: history(100_000_000_000, 100_000_000_000, 75_000_000_000, 75_000_000_000),
        }]);
    }
    // Each retained row is within its existing native 100b envelope; their
    // shared qualified total150b exceeds the one actual original100b control.
    let estimate = crate::tests::support::client::estimate_documents_from_save(&request);
    for bad in [request.clone(), estimate] {
        assert_located_rejection(
            &mut session,
            bad,
            "K1_QBI_SHARED_QUALIFIED_HISTORY_ABOVE_SOURCE",
            "[documents/common/k1/documents/0/document/activities/statement/source/qbi_blocks/1/allocations/0/qualified]",
        );
    }
    for block in &mut graph(&mut documents(&mut request)[0].document).qbi_blocks {
        block.allocations.as_mut().unwrap()[0].qualified =
            history(100_000_000_000, 100_000_000_000, 50_000_000_000, 50_000_000_000);
    }
    accepted(&send_fixture(&mut session, request));
    unchanged_source_roundtrip(&mut session);
}
