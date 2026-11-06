use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::assert::k1res::assert_atomic_rejection;
use crate::tests::support::builders::k1res::{
    documents, entity_exempt_other_fields, entity_exempt_other_output, graph, input,
    set_component_exempt_other, set_entity_exempt_other,
};
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{assert_native_review, component_exempt_other_session};
use crate::tests::*;

#[test]
fn component_exempt_other_keeps_whole_basis_and_independent_risk_and_tax_systems() {
    for scorp in [false, true] {
        let (_, mut income) = component_exempt_other_session(scorp);
        for (amt, exempt, allowed) in [(false, 50_001, 100_001), (true, 30_001, 80_001)] {
            let output = entity_exempt_other_output(&income, amt).unwrap();
            assert_eq!(output.basis.ending.get(), exempt);
            assert_eq!(output.components[0].basis_allowed.get(), 100_000);
            assert_eq!(output.components[1].basis_allowed.get(), 100_000);
            assert_eq!(output.components[0].risk_allowed.get(), allowed - 25_000);
            assert_eq!(output.components[1].risk_allowed.get(), 25_000);
            assert_eq!(
                output.components.iter().map(|c| c.risk_suspended.get()).sum::<i64>(),
                200_000 - allowed
            );
            assert!(output.components.iter().all(|c| c.income.get() == 0));
            assert_eq!(output.groups.iter().map(|g| g.nonpassive).sum::<i64>(), -allowed);
            assert_eq!(output.entity_exempt_interest, 0);
        }
        // The same documented total outside all activities changes basis only.
        set_component_exempt_other(&mut income, [0, 0], [0, 0]);
        set_entity_exempt_other(&mut income, 50_001, 30_001);
        for amt in [false, true] {
            let output = entity_exempt_other_output(&income, amt).unwrap();
            assert_eq!(output.components.iter().map(|c| c.risk_allowed.get()).sum::<i64>(), 50_000);
        }
        // Move the actual component source, not an inferred financial share.
        set_entity_exempt_other(&mut income, 0, 0);
        set_component_exempt_other(&mut income, [0, 50_001], [0, 30_001]);
        let output = entity_exempt_other_output(&income, false).unwrap();
        assert_eq!(output.components[0].risk_allowed.get(), 25_000);
        assert_eq!(output.components[1].risk_allowed.get(), 75_001);
    }
}

#[test]
fn component_exempt_other_retains_cents_and_competes_for_whole_entity_basis_once() {
    for scorp in [false, true] {
        let (mut session, mut income) = component_exempt_other_session(scorp);
        add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
        for cents in [49, 50] {
            set_component_exempt_other(&mut income, [50_000 + cents, 0], [30_000 + cents, 0]);
            let output = entity_exempt_other_output(&income, false).unwrap();
            assert_eq!(output.components[0].risk_allowed.get(), 75_000 + cents);
            session.draft.income[0].income = income.clone();
            let result = supported(&session.draft);
            assert_eq!(value(&result, "agi"), if cents == 49 { 249_000 } else { 248_999 });
        }
        match &mut income {
            FederalIncome::PartnershipActivities { data } => {
                data.entity.regular.opening = Some(49_999);
            }
            FederalIncome::ScorpActivities { data } => {
                data.entity.regular.opening = Some(49_999);
            }
            _ => unreachable!(),
        }
        set_component_exempt_other(&mut income, [50_001, 0], [30_001, 0]);
        let output = entity_exempt_other_output(&income, false).unwrap();
        assert_eq!(output.basis.ending.get(), 0);
        assert_eq!(output.components.iter().map(|c| c.basis_allowed.get()).sum::<i64>(), 100_000);
        assert_eq!(output.components[0].risk_allowed.get(), 50_000);
        assert_eq!(output.components[1].risk_allowed.get(), 25_000);
    }
}

#[test]
fn component_exempt_other_native_preview_review_and_corrected_source_survive_restart() {
    for scorp in [false, true] {
        let (mut session, mut income) = component_exempt_other_session(scorp);
        add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
        session = restart(session.draft);
        let before = session.draft.clone();
        let dirty = session.dirty;
        let ids = crate::k1_activity::source_ids(&income);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&income);

        assert!(matches!(
            crate::tests::support::client::income_pages::candidate_save_review(
                &mut session,
                crate::tests::support::client::estimate_documents_from_save(&request)
            ),
            Reply::Response(P::Review { .. })
        ));
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        accepted(&send_fixture(&mut session, request));
        let saved = session.draft.clone();
        assert_eq!(saved.revision, before.revision + 2);
        assert_eq!(saved.next_id, before.next_id);
        assert_eq!(crate::k1_activity::source_ids(&saved.income[0].income), ids);
        let result = supported(&saved);
        assert_eq!(value(&result, "agi"), 249_000);
        assert_eq!(return_line(&result, "2a"), 0);
        assert_eq!(return_line(&result, "2b"), 0);
        assert_eq!(value(&result, "self_employment_tax"), 0);
        assert_eq!(value(&result, "qbi_deduction"), 0);
        assert_eq!(line_amount(&result, "form_8960_net_investment_income"), 0);
        assert_native_review(&mut session);
        let mut cold = restart(saved.clone());
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(
            documents(&mut reopened)[0].document,
            crate::tests::support::builders::k1res::expected_fixture_input(&session.draft, &income)
        );
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
        assert_eq!(estimate(&cold.draft), result);
        // Correct REG attribution only; independently supplied AMT still belongs to A.
        set_component_exempt_other(&mut income, [0, 0], [30_001, 0]);
        let mut correction =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        documents(&mut correction)[0].document = input(&income);
        accepted(&send_fixture(&mut cold, correction));
        assert_eq!(cold.draft.revision, saved.revision + 2);
        assert_eq!(cold.draft.next_id, saved.next_id);
        assert_eq!(crate::k1_activity::source_ids(&cold.draft.income[0].income), ids);
        let corrected = supported(&cold.draft);
        assert_eq!(value(&corrected, "agi"), 249_500);
        assert_eq!(
            entity_exempt_other_output(&cold.draft.income[0].income, true).unwrap().components[0]
                .risk_allowed
                .get(),
            55_001
        );
        assert_native_review(&mut cold);
        let corrected_saved = cold.draft.clone();
        let mut final_cold = restart(corrected_saved.clone());
        let mut final_open =
            crate::tests::support::client::income_pages::open_document_center(&mut final_cold);
        assert_eq!(
            documents(&mut final_open)[0].document,
            crate::tests::support::builders::k1res::expected_fixture_input(&session.draft, &income)
        );
        accepted(&send_fixture(&mut final_cold, final_open));
        assert_eq!(final_cold.draft, corrected_saved);
        assert_eq!(estimate(&final_cold.draft), corrected);
    }
}

#[test]
fn component_exempt_other_keeps_unknown_associations_and_classifications_unavailable() {
    for scorp in [false, true] {
        let (session, income) = component_exempt_other_session(scorp);
        for case in 0..6 {
            let mut unknown = income.clone();
            let (source, _, _) = entity_exempt_other_fields(&mut unknown);
            match case {
                0 => source.components[0].risk_group = None,
                1 => source.components[0].regular = None,
                2 => source.components[0].alternative = K1StatementAmt::Unavailable,
                3 => source.risk_groups[0].source = K1RiskSource::Unavailable,
                4 => source.components[0].regular.as_mut().unwrap().operating_carry.risk = None,
                5 => match &mut unknown {
                    FederalIncome::PartnershipActivities { data } => {
                        data.unentered.push(crate::PartnerK1Item::OtherInformation)
                    }
                    FederalIncome::ScorpActivities { data } => {
                        data.special.push(crate::ScorpSpecial::DebtForgiveness)
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
            if case == 5 {
                // The existing source-class exclusion remains with its owner;
                // a whole-source left-out notice can retain a complete estimate.
                assert!(entity_exempt_other_output(&unknown, false).is_err());
                continue;
            }
            let mut candidate = session.draft.clone();
            candidate.income[0].income = unknown;
            let result = estimate(&candidate);
            assert!(result.tax.is_none(), "scorp={scorp} case={case}: {:?}", result.issues);
        }
    }
}

#[test]
fn component_exempt_other_bad_controls_and_values_refuse_save_atomically() {
    for scorp in [false, true] {
        let (mut session, income) = component_exempt_other_session(scorp);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut session, request));
        for case in 0..2 {
            let mut bad =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let source = graph(&mut documents(&mut bad)[0].document);
            let key = match case {
                0 => {
                    source.components[0].regular.as_mut().unwrap().exempt_other += 1;
                    "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"
                }
                1 => {
                    let K1StatementAmt::Separate { amounts } =
                        &mut source.components[0].alternative
                    else {
                        unreachable!()
                    };
                    amounts.exempt_other += 1;
                    "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"
                }
                _ => unreachable!(),
            };
            assert_atomic_rejection(&mut session, bad, key);
        }
    }
}
