use crate::k1_activity::{K1EntityOnlyAmt, K1RiskSource, K1StatementAmt};
use crate::tests::cross::interview::{accepted, document, open_topic};
use crate::tests::support::assert::k1res::assert_atomic_rejection;
use crate::tests::support::builders::k1res::{
    documents, entity_exempt_other_fields, entity_exempt_other_output,
    entity_exempt_other_passive_history, entity_exempt_other_rule, fresh_income, graph, input,
    set_entity_exempt_interest, set_entity_exempt_other,
};
use crate::tests::support::client::income_pages::catalog_preview_document;
use crate::tests::support::client::k1res::{assert_native_review, entity_exempt_interest_session};
use crate::tests::*;

#[test]
fn exempt_interest_basis_selects_regular_and_amt_without_replacing_other_exempt_income() {
    for scorp in [false, true] {
        let (_, mut income) = entity_exempt_interest_session(scorp);
        for (regular, amt, expected) in [
            (100_000, 100_000, [2_000, 1_500]),
            (0, 100_000, [1_000, 1_500]),
            (100_000, 50_000, [2_000, 1_000]),
        ] {
            set_entity_exempt_interest(&mut income, regular, amt);
            assert_eq!(entity_exempt_other_rule(&income), None);
            for (system, alternative) in [false, true].into_iter().enumerate() {
                let output = entity_exempt_other_output(&income, alternative).unwrap();
                assert_eq!(output.entity_exempt_interest, [regular, amt][system]);
                assert_eq!(crate::kernel::ratio::dollars(output.basis.ending.get()), 0);
                assert_eq!(
                    crate::kernel::ratio::dollars(
                        output.components.iter().map(|c| c.basis_allowed.get()).sum::<i64>()
                    ),
                    expected[system]
                );
                assert!(output.components.iter().all(|c| c.income.get() == 0));
                assert_eq!(
                    crate::kernel::ratio::dollars(
                        output.groups.iter().map(|g| g.nonpassive).sum::<i64>()
                    ),
                    -expected[system]
                );
            }
        }
        set_entity_exempt_other(&mut income, 50_000, 50_000);
        // The inherited setter replaces the AMT source; explicitly restore its interest.
        set_entity_exempt_interest(&mut income, 100_000, 50_000);
        assert_eq!(entity_exempt_other_rule(&income), None);
        assert_eq!(
            crate::kernel::ratio::dollars(
                entity_exempt_other_output(&income, false)
                    .unwrap()
                    .components
                    .iter()
                    .map(|c| c.basis_allowed.get())
                    .sum::<i64>()
            ),
            2_500
        );
        assert_eq!(
            crate::kernel::ratio::dollars(
                entity_exempt_other_output(&income, true)
                    .unwrap()
                    .components
                    .iter()
                    .map(|c| c.basis_allowed.get())
                    .sum::<i64>()
            ),
            1_500
        );
        let (source, _, alternative) = entity_exempt_other_fields(&mut income);
        source.alternative_entity_only = K1EntityOnlyAmt::SameAsRegular;
        alternative.as_mut().unwrap().exempt_interest = 100_000;
        assert_eq!(entity_exempt_other_rule(&income), None);
        assert_eq!(
            crate::kernel::ratio::dollars(
                entity_exempt_other_output(&income, true)
                    .unwrap()
                    .components
                    .iter()
                    .map(|c| c.basis_allowed.get())
                    .sum::<i64>()
            ),
            2_000
        );
    }
}

#[test]
fn exempt_interest_rounds_the_combined_basis_subtotal_once() {
    for scorp in [false, true] {
        let (_, mut income) = entity_exempt_interest_session(scorp);
        for (cents, expected) in [(49, 2_000), (50, 2_001), (51, 2_001)] {
            set_entity_exempt_interest(&mut income, 100_000 + cents, 100_000);
            assert_eq!(entity_exempt_other_rule(&income), None);
            let output = entity_exempt_other_output(&income, false).unwrap();
            assert_eq!(
                crate::kernel::ratio::dollars(
                    output.components.iter().map(|c| c.basis_allowed.get()).sum::<i64>()
                ),
                expected
            );
            assert_eq!(
                crate::kernel::ratio::dollars(
                    entity_exempt_other_output(&income, true)
                        .unwrap()
                        .components
                        .iter()
                        .map(|c| c.basis_allowed.get())
                        .sum::<i64>()
                ),
                1_500
            );
        }
        set_entity_exempt_interest(&mut income, 100_025, 100_000);
        let (source, current, alternative) = entity_exempt_other_fields(&mut income);
        source.components[0].regular.as_mut().unwrap().operating = 100_025;
        current.ordinary = -99_975;
        alternative.as_mut().unwrap().ordinary = -99_975;
        assert_eq!(entity_exempt_other_rule(&income), None);
        let output = entity_exempt_other_output(&income, false).unwrap();
        // Opening $1,000 + rounded ($1,000.25 ordinary + $1,000.25 exempt)
        // leaves $1,001 after the other component's $2,000 loss.
        assert_eq!(crate::kernel::ratio::dollars(output.basis.ending.get()), 1_001);
        assert_eq!(
            crate::kernel::ratio::dollars(
                output.components.iter().map(|c| c.basis_allowed.get()).sum::<i64>()
            ),
            2_000
        );
    }
}

#[test]
fn outside_exempt_interest_never_funds_group_risk_or_releases_passive_losses() {
    for scorp in [false, true] {
        let (_, mut income) = entity_exempt_interest_session(scorp);
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
                crate::K1ActivityParticipation::Partnership { facts, .. } => {
                    *facts = PartnerParticipation::Limited {
                        minutes: 0,
                        prior_ten: 0,
                        personal_service: false,
                        service_years: 0,
                    };
                }
                crate::K1ActivityParticipation::Scorp { facts, .. } => facts.minutes = 0,
                _ => unreachable!(),
            }
        }
        assert_eq!(entity_exempt_other_rule(&income), None);
        for amt in [false, true] {
            let output = entity_exempt_other_output(&income, amt).unwrap();
            for component in &output.components {
                assert_eq!(
                    (
                        crate::kernel::ratio::dollars(component.income.get()),
                        crate::kernel::ratio::dollars(component.risk_allowed.get()),
                        crate::kernel::ratio::dollars(component.prior_passive.get()),
                    ),
                    (0, 500, 200)
                );
                assert_eq!(
                    crate::kernel::ratio::dollars(component.risk_suspended.get()),
                    if amt { 250 } else { 500 }
                );
            }
            for group in &output.groups {
                assert_eq!(
                    (
                        crate::kernel::ratio::dollars(group.current),
                        crate::kernel::ratio::dollars(group.passive),
                        crate::kernel::ratio::dollars(group.nonpassive),
                        crate::kernel::ratio::dollars(group.prior_released)
                    ),
                    (-500, -700, 0, 0)
                );
            }
        }
    }
}

#[test]
fn exempt_interest_native_lifecycle_transfers_regular_line_2a_once() {
    for scorp in [false, true] {
        let (mut session, mut income) = entity_exempt_interest_session(scorp);
        add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
        add(
            &mut session.draft,
            crate::tests::income(FederalIncome::Interest {
                data: native_exempt_interest(30_000, 0),
            }),
        );
        session = restart(session.draft);
        let before = session.draft.clone();
        let dirty = session.dirty;
        let id = before.income[0].id;
        let ids = crate::k1_activity::source_ids(&income);
        let kind = if scorp { "scorp" } else { "partnership" };
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        documents(&mut request)[0].document = input(&income);
        let mut preview = catalog_preview_document(input(&income));
        let P::PreviewDocument { id: preview_id, .. } = &mut preview else { unreachable!() };
        *preview_id = id;
        assert!(matches!(
            send(&mut session, preview),
            Reply::Response(P::IncomePreview { preview: Some(-200_000), .. })
        ));
        assert!(matches!(
            send(
                &mut session,
                crate::tests::support::client::estimate_documents_from_save(&request)
            ),
            Reply::Response(P::IncomeRunningEstimate { .. })
        ));
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        accepted(&send(&mut session, request));
        let saved = session.draft.clone();
        assert_native_review(&mut session);
        let unchanged = open_topic(&mut session, TaxSection::PassThrough);
        accepted(&send(&mut session, unchanged));
        assert_eq!(session.draft, saved);
        assert_native_review(&mut session);
        assert_eq!(saved.revision, before.revision + 1);
        assert_eq!(saved.next_id, before.next_id);
        assert_eq!(crate::k1_activity::source_ids(&saved.income[0].income), ids);
        assert_eq!(saved.income[1..], before.income[1..]);
        let result = estimate(&saved);
        assert!(result.tax.is_some(), "{:?}", result.issues);
        assert_eq!(return_line(&result, "2a"), 130_000);
        assert_eq!(return_line(&result, "2b"), 0);
        assert_eq!(line_amount(&result, &alloc::format!("{kind}_{id}_exempt_interest")), 100_000);
        assert_eq!(line_amount(&result, &alloc::format!("{kind}_{id}_schedule_e")), -200_000);
        assert_eq!(line_amount(&result, &alloc::format!("amt_{kind}_{id}_schedule_e")), -150_000);
        assert_eq!(line_amount(&result, "agi"), 24_800_000);
        assert_eq!(line_amount(&result, "form_8960_net_investment_income"), 0);
        assert_eq!(line_amount(&result, "net_investment_income_tax"), 0);
        assert_eq!(line_amount(&result, "eic_investment_income"), 130_000);
        let mut cold = restart(saved.clone());
        let mut reopened = open_topic(&mut cold, TaxSection::PassThrough);
        assert_eq!(documents(&mut reopened)[0].document, input(&income));
        assert_eq!(estimate(&cold.draft), result);
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
        assert_native_review(&mut cold);
        set_entity_exempt_interest(&mut income, 0, 100_000);
        reopened = open_topic(&mut cold, TaxSection::PassThrough);
        documents(&mut reopened)[0].document = input(&income);
        accepted(&send(&mut cold, reopened));
        let corrected_saved = cold.draft.clone();
        assert_native_review(&mut cold);
        assert_eq!(corrected_saved.revision, saved.revision + 1);
        assert_eq!(corrected_saved.next_id, saved.next_id);
        assert_eq!(crate::k1_activity::source_ids(&corrected_saved.income[0].income), ids);
        let corrected = estimate(&corrected_saved);
        assert!(corrected.tax.is_some(), "{:?}", corrected.issues);
        assert_eq!(return_line(&corrected, "2a"), 30_000);
        assert_eq!(line_amount(&corrected, &alloc::format!("{kind}_{id}_schedule_e")), -100_000);
        assert_eq!(
            line_amount(&corrected, &alloc::format!("amt_{kind}_{id}_schedule_e")),
            -150_000
        );
        assert_eq!(line_amount(&corrected, "agi"), 24_900_000);
        let mut final_cold = restart(corrected_saved.clone());
        let mut reopened = open_topic(&mut final_cold, TaxSection::PassThrough);
        assert_eq!(documents(&mut reopened)[0].document, input(&income));
        accepted(&send(&mut final_cold, reopened));
        assert_eq!(final_cold.draft, corrected_saved);
        assert_eq!(estimate(&final_cold.draft), corrected);
        assert_native_review(&mut final_cold);
        // A subsequent ordinary source obtains the next contiguous identities.
        let mut new_request = open_topic(&mut final_cold, TaxSection::PassThrough);
        let P::SaveIncomes { allowed_community, .. } = &new_request else { unreachable!() };
        let mut row = document(0, input(&fresh_income(scorp)));
        row.allowed_community = allowed_community.clone();
        documents(&mut new_request).push(row);
        accepted(&send(&mut final_cold, new_request));
        let new_entry = final_cold
            .draft
            .income
            .iter()
            .find(|entry| entry.id != id && crate::k1_activity::source(&entry.income).is_some())
            .unwrap();
        let mut allocated = crate::k1_activity::source_ids(&new_entry.income);
        allocated.push(new_entry.id);
        allocated.sort_unstable();
        assert_eq!(
            allocated,
            (corrected_saved.next_id..final_cold.draft.next_id).collect::<Vec<_>>()
        );
        assert_eq!(final_cold.draft.income[0], corrected_saved.income[0]);
    }
}

#[test]
fn exempt_interest_reaches_existing_social_security_worksheet() {
    for scorp in [false, true] {
        let (mut session, mut income) = entity_exempt_interest_session(scorp);
        let (source, current, alternative) = entity_exempt_other_fields(&mut income);
        for component in &mut source.components {
            component.regular.as_mut().unwrap().operating = 0;
        }
        current.ordinary = 0;
        alternative.as_mut().unwrap().ordinary = 0;
        set_entity_exempt_interest(&mut income, 0, 0);
        assert_eq!(entity_exempt_other_rule(&income), None);
        session.draft.income[0].income = income.clone();
        add(&mut session.draft, wage(0, Owner::Taxpayer, 2_400_000, 0));
        add(&mut session.draft, crate::tests::income(social_security_net(1_000_000, 0)));
        let before = estimate(&session.draft);
        assert!(before.tax.is_some(), "{:?}", before.issues);
        assert_eq!(return_line(&before, "6b"), 200_000);
        assert_eq!(line_amount(&before, "agi"), 2_600_000);
        set_entity_exempt_interest(&mut income, 200_000, 50_000);
        assert_eq!(entity_exempt_other_rule(&income), None);
        session.draft.income[0].income = income;
        let after = estimate(&session.draft);
        assert!(after.tax.is_some(), "{:?}", after.issues);
        assert_eq!(return_line(&after, "2a"), 200_000);
        assert_eq!(return_line(&after, "6b"), 300_000);
        assert_eq!(line_amount(&after, "agi"), 2_700_000);
    }
}

#[test]
fn valid_unknown_exempt_sources_keep_the_named_boundary_and_invalid_saves_are_atomic() {
    for scorp in [false, true] {
        let (mut session, income) = entity_exempt_interest_session(scorp);
        add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
        session = restart(session.draft);
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        documents(&mut request)[0].document = input(&income);
        accepted(&send(&mut session, request));
        for case in 0..6 {
            let mut unknown = income.clone();
            match case {
                0 => entity_exempt_other_fields(&mut unknown).0.entity_only = None,
                1 => {
                    entity_exempt_other_fields(&mut unknown).0.alternative_entity_only =
                        K1EntityOnlyAmt::Unavailable
                }
                2 => *entity_exempt_other_fields(&mut unknown).2 = None,
                3 => {
                    entity_exempt_other_fields(&mut unknown).0.risk_groups[0].source =
                        K1RiskSource::Unavailable
                }
                4 => {
                    entity_exempt_other_fields(&mut unknown).0.components[0].alternative =
                        K1StatementAmt::Unavailable
                }
                5 => match &mut unknown {
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
                _ => unreachable!(),
            }
            // First prove the complete source validator accepts this actual Unknown.
            assert_eq!(entity_exempt_other_rule(&unknown), None, "scorp={scorp} case={case}");
            if case == 5 {
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
        for case in 0..4 {
            let mut bad = open_topic(&mut session, TaxSection::PassThrough);
            let source = graph(&mut documents(&mut bad)[0].document);
            let key = match case {
                0 => {
                    source.entity_only.as_mut().unwrap().exempt_interest += 1;
                    "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"
                }
                1 => {
                    let K1EntityOnlyAmt::Separate { amounts } = &mut source.alternative_entity_only
                    else {
                        unreachable!()
                    };
                    amounts.exempt_interest += 1;
                    "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"
                }
                2 => {
                    source.entity_only.as_mut().unwrap().exempt_interest = -1;
                    "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
                }
                3 => {
                    source.components[1].id = source.components[0].id;
                    "K1_ACTIVITY_SOURCE_ID_REPEATED"
                }
                _ => unreachable!(),
            };
            assert_atomic_rejection(&mut session, bad, key);
        }
    }
}
