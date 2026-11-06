use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::assert::k1res::assert_atomic_rejection;
use crate::tests::support::builders::k1res::{
    documents, entity_exempt_other_fields, entity_exempt_other_output,
    entity_exempt_other_passive_history, entity_exempt_other_rule, graph, input,
    set_entity_exempt_other,
};
use crate::tests::support::client::k1res::entity_exempt_other_session;
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::*;

#[test]
fn exempt_other_basis_uses_independent_tax_systems_and_exact_source_cents() {
    for scorp in [false, true] {
        let (_, mut income) = entity_exempt_other_session(scorp);
        for (regular_exempt, amt_exempt, totals, shares) in [
            (100_000, 100_000, [2_000, 1_500], [1_000, 750]),
            (0, 0, [1_000, 500], [500, 250]),
            (100_000, 50_000, [2_000, 1_000], [1_000, 500]),
        ] {
            set_entity_exempt_other(&mut income, regular_exempt, amt_exempt);
            assert_eq!(entity_exempt_other_rule(&income), None);
            for (system, amt) in [false, true].into_iter().enumerate() {
                let output = entity_exempt_other_output(&income, amt).unwrap();
                assert_eq!(crate::kernel::ratio::dollars(output.basis.ending.get()), 0);
                assert_eq!(
                    crate::kernel::ratio::dollars(
                        output.components.iter().map(|c| c.basis_allowed.get()).sum::<i64>()
                    ),
                    totals[system]
                );
                for component in &output.components {
                    assert_eq!(
                        (
                            crate::kernel::ratio::dollars(component.basis_allowed.get()),
                            crate::kernel::ratio::dollars(component.basis_suspended.get()),
                        ),
                        (shares[system], 2_000 - shares[system])
                    );
                    assert_eq!(
                        (
                            crate::kernel::ratio::dollars(component.income.get()),
                            crate::kernel::ratio::dollars(component.risk_allowed.get()),
                            crate::kernel::ratio::dollars(component.risk_suspended.get()),
                        ),
                        (0, shares[system], 0)
                    );
                }
                assert_eq!(
                    crate::kernel::ratio::dollars(
                        output.groups.iter().map(|g| g.nonpassive).sum::<i64>()
                    ),
                    -totals[system]
                );
            }
        }
        // The final allowance projection rounds once; the native stage retains every cent.
        for (cents, expected) in [(49, 2_000), (50, 2_001), (51, 2_001)] {
            set_entity_exempt_other(&mut income, 100_000 + cents, 100_000);
            assert_eq!(entity_exempt_other_rule(&income), None);
            let regular = entity_exempt_other_output(&income, false).unwrap();
            assert_eq!(
                crate::kernel::ratio::dollars(
                    regular.components.iter().map(|c| c.basis_allowed.get()).sum::<i64>()
                ),
                expected
            );
            // Actual source loss retains every exempt-income cent before reporting.
            assert_eq!(
                regular.groups.iter().map(|g| g.nonpassive).sum::<i64>(),
                -(200_000 + cents)
            );
            assert_eq!(
                regular.components.iter().map(|c| c.basis_allowed.get()).sum::<i64>(),
                200_000 + cents
            );
            let amt = entity_exempt_other_output(&income, true).unwrap();
            assert_eq!(
                crate::kernel::ratio::dollars(
                    amt.components.iter().map(|c| c.basis_allowed.get()).sum::<i64>()
                ),
                1_500
            );
        }
    }
}

#[test]
fn outside_exempt_other_does_not_increase_group_risk_or_release_passive_history() {
    for scorp in [false, true] {
        let (_, mut income) = entity_exempt_other_session(scorp);
        entity_exempt_other_passive_history(&mut income);
        let (source, _, _) = entity_exempt_other_fields(&mut income);
        for group in &mut source.risk_groups {
            match &mut group.source {
                K1RiskSource::Partnership { regular, alternative } => {
                    regular.start =
                        ScorpRiskStart::Detailed { capacity: 50_000, profit: 0, loss: 0 };
                    *alternative = Some(regular.clone());
                }
                K1RiskSource::Scorp { regular, alternative } => {
                    regular.start =
                        ScorpRiskStart::Detailed { capacity: 50_000, profit: 0, loss: 0 };
                    *alternative = Some(regular.clone());
                }
                _ => unreachable!(),
            }
        }
        for group in &mut source.passive_groups {
            match &mut group.participation {
                K1ActivityParticipation::Partnership { facts, .. } => {
                    *facts = PartnerParticipation::Limited {
                        minutes: 0,
                        prior_ten: 0,
                        personal_service: false,
                        service_years: 0,
                    };
                }
                K1ActivityParticipation::Scorp { facts, .. } => {
                    facts.minutes = 0;
                }
                _ => unreachable!(),
            }
        }
        assert_eq!(entity_exempt_other_rule(&income), None);
        for amt in [false, true] {
            let output = entity_exempt_other_output(&income, amt).unwrap();
            for component in &output.components {
                assert_eq!(crate::kernel::ratio::dollars(component.income.get()), 0);
                assert_eq!(crate::kernel::ratio::dollars(component.risk_allowed.get()), 500);
                assert_eq!(
                    crate::kernel::ratio::dollars(component.risk_suspended.get()),
                    if amt { 250 } else { 500 }
                );
                assert_eq!(crate::kernel::ratio::dollars(component.prior_passive.get()), 200);
            }
            for group in &output.groups {
                assert_eq!(
                    (
                        crate::kernel::ratio::dollars(group.current),
                        crate::kernel::ratio::dollars(group.prior),
                        crate::kernel::ratio::dollars(group.passive),
                        crate::kernel::ratio::dollars(group.nonpassive),
                        group.prior_released
                    ),
                    (-500, 200, -700, 0, 0)
                );
            }
        }
    }
}

#[test]
fn exempt_other_native_preview_save_correction_reopen_and_cold_restart() {
    for scorp in [false, true] {
        let (mut session, mut income) = entity_exempt_other_session(scorp);
        add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
        session = restart(session.draft);
        let before = session.draft.clone();
        let dirty = session.dirty;
        let id = before.income[0].id;
        let kind = if scorp { "scorp" } else { "partnership" };
        let ids = crate::k1_activity::source_ids(&income);
        assert_eq!(entity_exempt_other_rule(&income), None);
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
        assert_eq!(saved.income[1], before.income[1]);
        let result = estimate(&saved);
        assert!(result.tax.is_some(), "{:?}", result.issues);
        assert_eq!(line_amount(&result, &alloc::format!("{kind}_{id}_schedule_e")), -200_000);
        assert_eq!(line_amount(&result, &alloc::format!("amt_{kind}_{id}_schedule_e")), -150_000);
        assert_eq!(line_amount(&result, "agi"), 24_800_000);
        assert_eq!(return_line(&result, "2a"), 0);
        assert_eq!(return_line(&result, "2b"), 0);
        assert_eq!(line_amount(&result, "form_8960_net_investment_income"), 0);
        assert_eq!(line_amount(&result, "net_investment_income_tax"), 0);
        let mut cold = restart(saved.clone());
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(
            documents(&mut reopened)[0].document,
            crate::tests::support::builders::k1res::expected_fixture_input(&session.draft, &income)
        );
        assert_eq!(estimate(&cold.draft), result);
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
        // Correct REG only; actual AMT controls and source remain independently known.
        set_entity_exempt_other(&mut income, 0, 100_000);
        reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
        documents(&mut reopened)[0].document = input(&income);
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(cold.draft.revision, saved.revision + 2);
        assert_eq!(cold.draft.next_id, saved.next_id);
        assert_eq!(crate::k1_activity::source_ids(&cold.draft.income[0].income), ids);
        let corrected = estimate(&cold.draft);
        assert!(corrected.tax.is_some(), "{:?}", corrected.issues);
        assert_eq!(line_amount(&corrected, &alloc::format!("{kind}_{id}_schedule_e")), -100_000);
        assert_eq!(
            line_amount(&corrected, &alloc::format!("amt_{kind}_{id}_schedule_e")),
            -150_000
        );
        assert_eq!(line_amount(&corrected, "agi"), 24_900_000);
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
fn exempt_other_valid_unknowns_remain_unavailable_and_bad_controls_save_atomically() {
    for scorp in [false, true] {
        let (mut session, income) = entity_exempt_other_session(scorp);
        add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
        session = restart(session.draft);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut session, request));
        for case in 0..8 {
            let mut unknown = income.clone();
            match case {
                0 => {
                    entity_exempt_other_fields(&mut unknown).0.entity_only = None;
                }
                1 => {
                    entity_exempt_other_fields(&mut unknown).0.alternative_entity_only =
                        K1EntityOnlyAmt::Unavailable;
                }
                2 => {
                    *entity_exempt_other_fields(&mut unknown).2 = None;
                }
                3 => {
                    entity_exempt_other_fields(&mut unknown).0.risk_groups[0].source =
                        K1RiskSource::Unavailable;
                }
                4 => {
                    entity_exempt_other_fields(&mut unknown).0.components[0]
                        .regular
                        .as_mut()
                        .unwrap()
                        .operating_carry
                        .basis = None;
                }
                5 => {
                    entity_exempt_other_fields(&mut unknown).0.components[0].alternative =
                        K1StatementAmt::Unavailable;
                }
                6 => match &mut unknown {
                    FederalIncome::PartnershipActivities { data } => {
                        data.entity.regular.complete = false;
                        data.entity.regular.opening = None;
                    }
                    FederalIncome::ScorpActivities { data } => {
                        data.entity.regular.complete = false;
                        data.entity.regular.opening = None;
                    }
                    _ => unreachable!(),
                },
                7 => match &mut unknown {
                    FederalIncome::PartnershipActivities { data } => {
                        data.entity.regular.complete = false;
                        data.entity.regular.prior_nondeductible = None;
                    }
                    FederalIncome::ScorpActivities { data } => {
                        data.entity.regular.complete = false;
                        data.entity.regular.prior_nondeductible = None;
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
            // FIRST establish native full source validity, before testing availability.
            assert_eq!(entity_exempt_other_rule(&unknown), None, "scorp={scorp} case={case}");
            if case >= 6 {
                assert!(entity_exempt_other_output(&unknown, false).is_err());
                assert_eq!(
                    crate::kernel::ratio::dollars(
                        entity_exempt_other_output(&unknown, true)
                            .unwrap()
                            .components
                            .iter()
                            .map(|c| c.basis_allowed.get())
                            .sum::<i64>()
                    ),
                    1_500
                );
            }
            let mut candidate = session.draft.clone();
            candidate.income[0].income = unknown;
            let result = estimate(&candidate);
            assert!(result.tax.is_none(), "scorp={scorp} case={case}");
            assert_eq!(
                result
                    .issues
                    .iter()
                    .filter(|i| i.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
                    .count(),
                1,
                "{:?}",
                result.issues
            );
        }
        for case in 0..2 {
            let mut bad =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let source = graph(&mut documents(&mut bad)[0].document);
            let key = match case {
                0 => {
                    source.entity_only.as_mut().unwrap().exempt_other += 1;
                    "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"
                }
                1 => {
                    let K1EntityOnlyAmt::Separate { amounts } = &mut source.alternative_entity_only
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
