use crate::k1_activity::{K1EntityOnlyAmt, K1StatementAmt, K1StatementCapitalCarry};
use crate::kernel::ratio::dollars;
use crate::tests::cross::interview::accepted;
use crate::tests::support::assert::k1res::assert_atomic_rejection;
use crate::tests::support::assert::{assert_injured_ordinary_unchanged, niit_optional_column};
use crate::tests::support::builders::k1res::{
    documents, entity_capital_debt, entity_exempt_other_fields, entity_exempt_other_output,
    entity_exempt_other_passive_history, entity_exempt_other_rule, fresh_income, input,
    set_entity_capital, zero_entity_capital_operating,
};
use crate::tests::support::client::k1res::entity_capital_session;
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::*;

#[test]
fn positive_capital_basis_and_source_cents_are_independent_in_both_systems() {
    for scorp in [false, true] {
        let (_, mut income) = entity_capital_session(scorp);
        for (regular, amt, allowed) in [
            ([30_000, 70_000], [10_000, 30_000], [150_000, 120_000]),
            ([30_000, 0], [0, 30_000], [80_000, 110_000]),
            ([0, 70_000], [10_000, 0], [120_000, 90_000]),
            ([30_025, 70_025], [10_000, 30_000], [150_050, 120_000]),
        ] {
            set_entity_capital(&mut income, regular, amt);
            assert_eq!(entity_exempt_other_rule(&income), None);
            for (system, alternative) in [false, true].into_iter().enumerate() {
                let result = entity_exempt_other_output(&income, alternative).unwrap();
                assert_eq!(result.entity_capital, [regular, amt][system]);
                assert_eq!(
                    result.components.iter().map(|v| v.basis_allowed.get()).sum::<i64>(),
                    allowed[system]
                );
                assert_eq!(
                    result.components.iter().map(|v| v.basis_suspended.get()).sum::<i64>(),
                    200_000 - allowed[system]
                );
                assert_eq!(result.basis.ending.get(), 0);
                assert!(result.components.iter().all(|v| v.income.get() == 0));
                assert_eq!(
                    result.groups.iter().map(|v| v.nonpassive).sum::<i64>(),
                    -allowed[system]
                );
            }
        }
        set_entity_capital(&mut income, [30_024, 70_025], [10_000, 30_000]);
        assert_eq!(
            entity_exempt_other_output(&income, false)
                .unwrap()
                .components
                .iter()
                .map(|v| v.basis_allowed.get())
                .sum::<i64>(),
            150_049
        );
        let (source, _, alternative) = entity_exempt_other_fields(&mut income);
        source.alternative_entity_only = K1EntityOnlyAmt::SameAsRegular;
        alternative.as_mut().unwrap().portfolio.short = 30_024;
        alternative.as_mut().unwrap().portfolio.long = 70_025;
        assert_eq!(entity_exempt_other_rule(&income), None);
        let amt = entity_exempt_other_output(&income, true).unwrap();
        assert_eq!(amt.entity_capital, [30_024, 70_025]);
        assert_eq!(
            dollars(amt.components.iter().map(|v| v.basis_allowed.get()).sum::<i64>()),
            1_800
        );
    }
}

#[test]
fn capital_native_save_review_reopen_restart_correction_and_new_id_lifecycle() {
    for scorp in [false, true] {
        let (mut session, mut income) = entity_capital_session(scorp);
        add(&mut session.draft, wage(0, Owner::Taxpayer, 6_000_000, 0));
        session = restart(session.draft);
        let before = session.draft.clone();
        let dirty = session.dirty;
        let id = before.income[0].id;
        let ids = crate::k1_activity::source_ids(&income);
        let kind = if scorp { "scorp" } else { "partnership" };
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
        assert_eq!(saved.income[1..], before.income[1..]);
        assert_eq!(crate::k1_activity::source_ids(&saved.income[0].income), ids);
        let result = supported(&saved);
        assert_eq!(value(&result, "agi"), 59_500);
        assert_eq!(value(&result, &alloc::format!("{kind}_{id}_short_gain")), 300);
        assert_eq!(value(&result, &alloc::format!("{kind}_{id}_long_gain")), 700);
        assert_eq!(value(&result, &alloc::format!("{kind}_{id}_schedule_e")), -1_500);
        assert_eq!(value(&result, &alloc::format!("amt_{kind}_{id}_schedule_e")), -1_200);
        assert!(matches!(
            send_fixture(&mut session, P::ReviewAccepted { user: TEST_USER }),
            Reply::Response(P::Review { .. })
        ));
        let mut cold = restart(saved.clone());
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(
            documents(&mut reopened)[0].document,
            crate::tests::support::builders::k1res::expected_fixture_input(&session.draft, &income)
        );
        assert_eq!(supported(&cold.draft), result);
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
        set_entity_capital(&mut income, [0, 0], [10_000, 30_000]);
        let mut correction =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        documents(&mut correction)[0].document = input(&income);
        accepted(&send_fixture(&mut cold, correction));
        let (source, _, alternative) = entity_exempt_other_fields(&mut income);
        source.alternative_entity_only = K1EntityOnlyAmt::SameAsRegular;
        alternative.as_mut().unwrap().portfolio.short = 0;
        alternative.as_mut().unwrap().portfolio.long = 0;
        let mut same = crate::tests::support::client::income_pages::open_document_center(&mut cold);
        documents(&mut same)[0].document = input(&income);
        accepted(&send_fixture(&mut cold, same));
        let corrected = cold.draft.clone();
        assert_eq!(corrected.revision, saved.revision + 3);
        assert_eq!(corrected.next_id, saved.next_id);
        assert_eq!(crate::k1_activity::source_ids(&corrected.income[0].income), ids);
        let corrected_result = supported(&corrected);
        assert_eq!(value(&corrected_result, "agi"), 59_500);
        assert_eq!(value(&corrected_result, &alloc::format!("{kind}_{id}_schedule_e")), -500);
        let mut cold = restart(corrected.clone());
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(
            documents(&mut reopened)[0].document,
            crate::tests::support::builders::k1res::expected_fixture_input(&session.draft, &income)
        );
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(cold.draft, corrected);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        let P::SaveDocuments { allowed_community, .. } = &request else { unreachable!() };
        let mut row =
            crate::tests::support::builders::k1res::source_row(0, input(&fresh_income(scorp)));
        row.allowed_community = allowed_community.clone();
        documents(&mut request).push(row);
        accepted(&send_fixture(&mut cold, request));
        let new = cold
            .draft
            .income
            .iter()
            .find(|v| v.id != id && crate::k1_activity::source(&v.income).is_some())
            .unwrap();
        let mut allocated = crate::k1_activity::source_ids(&new.income);
        allocated.push(new.id);
        allocated.sort_unstable();
        assert_eq!(allocated, (corrected.next_id..cold.draft.next_id).collect::<Vec<_>>());
        assert_eq!(cold.draft.income[0], corrected.income[0]);
    }
}

#[test]
fn positive_portfolio_gains_reach_niit_without_becoming_business_income() {
    for scorp in [false, true] {
        let (mut session, income) = entity_capital_session(scorp);
        session.draft.income[0].income = income;
        add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
        let result = supported(&session.draft);
        assert_eq!(value(&result, "agi"), 249_500);
        assert_eq!(value(&result, "form_8960_net_investment_income"), 1_000);
        assert_eq!(value(&result, "net_investment_income_tax"), 38);
        assert_eq!(value(&result, "eic_investment_income"), 1_000);
    }
}

#[test]
fn capital_unknowns_and_unsupported_characters_preserve_known_regular_sources() {
    for scorp in [false, true] {
        let (session, income) = entity_capital_session(scorp);
        for case in 0..9 {
            let mut unknown = income.clone();
            let (source, current, alternative) = entity_exempt_other_fields(&mut unknown);
            match case {
                0 => source.entity_only = None,
                1 => source.alternative_entity_only = K1EntityOnlyAmt::Unavailable,
                2 => *alternative = None,
                3 => source.components[0].alternative = K1StatementAmt::Unavailable,
                4 => {
                    source.entity_only.as_mut().unwrap().capital_carry =
                        K1StatementCapitalCarry::Unavailable
                }
                5 => {
                    source.entity_only.as_mut().unwrap().portfolio.short = -100;
                    current.portfolio.short = -100;
                }
                6 => {
                    source.entity_only.as_mut().unwrap().portfolio.collectibles = 100;
                    current.portfolio.collectibles = 100;
                }
                7 => {
                    source.entity_only.as_mut().unwrap().portfolio.unrecaptured_fund = 100;
                    current.portfolio.unrecaptured_fund = 100;
                }
                8 => {} // Available producer; the actual community context is below.
                _ => unreachable!(),
            }
            assert_eq!(entity_exempt_other_rule(&unknown), None, "scorp={scorp} case={case}");
            if matches!(case, 1..=3 | 8) {
                assert_eq!(
                    entity_exempt_other_output(&unknown, false).unwrap().entity_capital,
                    [30_000, 70_000]
                );
            }
            let mut draft = session.draft.clone();
            draft.income[0].income = unknown.clone();
            if case == 8 {
                let separate = complete(FederalStatus::Separate);
                draft.setup = separate.setup;
                draft.people = separate.people;
                draft.setup.as_mut().unwrap().community_property = community_arizona();
                draft.income[0].community = CommunityShare::Community;
            } else {
                let joint = complete(FederalStatus::Joint);
                draft.setup = joint.setup;
                draft.people = joint.people;
                draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
                draft.injured_spouse = Some(allocation());
            }
            let result = estimate(&draft);
            assert!(result.tax.is_none());
            assert_eq!(
                result
                    .issues
                    .iter()
                    .filter(|v| v.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
                    .count(),
                1
            );
            assert_eq!(draft.income[0].income, unknown);
            if case != 8 {
                // An uncomputed source is not a known zero for Form8379.
                assert_eq!(niit_optional_column(&result, "13b", "b"), None);
                assert_eq!(niit_optional_column(&result, "13b", "c"), None);
            }
        }
    }
}

#[test]
fn capital_controls_and_overflow_are_refused_atomically_without_amount_clamping() {
    for scorp in [false, true] {
        let (mut session, income) = entity_capital_session(scorp);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut session, request));
        for alternative in [false, true] {
            let mut invalid = income.clone();
            let (source, current, amt) = entity_exempt_other_fields(&mut invalid);
            if alternative {
                let K1EntityOnlyAmt::Separate { amounts } = &mut source.alternative_entity_only
                else {
                    unreachable!()
                };
                amounts.portfolio.long += 1;
            } else {
                current.portfolio.short += 1;
            }
            let _ = amt;
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            documents(&mut request)[0].document = input(&invalid);
            assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE");
        }
    }
}

#[test]
fn capital_gains_use_the_existing_social_security_and_4952_owners() {
    for scorp in [false, true] {
        let (mut session, mut income) = entity_capital_session(scorp);
        zero_entity_capital_operating(&mut income);
        set_entity_capital(&mut income, [0, 0], [0, 0]);
        session.draft.income[0].income = income.clone();
        add(&mut session.draft, wage(0, Owner::Taxpayer, 2_400_000, 0));
        add(&mut session.draft, crate::tests::income(social_security_net(1_000_000, 0)));
        let before = supported(&session.draft);
        assert_eq!(return_line(&before, "6b"), 200_000);
        set_entity_capital(&mut income, [30_000, 70_000], [10_000, 30_000]);
        session.draft.income[0].income = income;
        let after = supported(&session.draft);
        // $24,000 wages + $1,000 gains + half of $10,000 benefits:
        // $5,000 above the $25,000 base, half taxable = $2,500.
        assert_eq!(return_line(&after, "6b"), 250_000);
        assert_eq!(value(&after, "agi"), 27_500);
        investment_borrowing(&mut session.draft, 1_000, 0, 0);
        let no_election = supported(&session.draft);
        for (prefix, figures) in [("", [1_000, 700, 300]), ("amt_", [400, 300, 100])] {
            for (line, expected) in ["4d", "4e", "4f"].into_iter().zip(figures) {
                assert_eq!(
                    value(&no_election, &alloc::format!("{prefix}investment_form_line{line}")),
                    expected
                );
            }
        }
        session.draft.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
            InvestmentElection::CapitalFirst { amount: 70_000 };
        let elected = supported(&session.draft);
        assert_eq!(value(&elected, "investment_form_line4g"), 700);
        assert_eq!(value(&elected, "amt_investment_form_line4g"), 300);
    }
}

#[test]
fn capital_gains_never_supply_risk_investment_or_release_passive_history() {
    for scorp in [false, true] {
        let (_, mut income) = entity_capital_session(scorp);
        entity_exempt_other_passive_history(&mut income);
        let (source, _, _) = entity_exempt_other_fields(&mut income);
        for group in &mut source.risk_groups {
            match &mut group.source {
                crate::K1RiskSource::Partnership { regular, alternative } => {
                    regular.start =
                        ScorpRiskStart::Detailed { capacity: 50_000, profit: 0, loss: 0 };
                    *alternative = Some(regular.clone());
                }
                crate::K1RiskSource::Scorp { regular, alternative } => {
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
                    }
                }
                crate::K1ActivityParticipation::Scorp { facts, .. } => facts.minutes = 0,
                _ => unreachable!(),
            }
        }
        assert_eq!(entity_exempt_other_rule(&income), None);
        for amt in [false, true] {
            let result = entity_exempt_other_output(&income, amt).unwrap();
            for component in &result.components {
                assert_eq!(
                    (
                        dollars(component.income.get()),
                        dollars(component.risk_allowed.get()),
                        dollars(component.prior_passive.get())
                    ),
                    (0, 500, 200)
                );
                assert_eq!(dollars(component.risk_suspended.get()), if amt { 100 } else { 250 });
            }
            assert!(result.groups.iter().all(|v| (
                dollars(v.current),
                dollars(v.passive),
                dollars(v.nonpassive),
                v.prior_released
            ) == (-500, -700, 0, 0)));
        }
    }
}

#[test]
fn capital_native_joint_both_owners_have_exact_injured_niit_sources() {
    for scorp in [false, true] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let (mut session, income) = entity_capital_session(scorp);
            let joint = complete(FederalStatus::Joint);
            session.draft.setup = joint.setup;
            session.draft.people = joint.people;
            session.draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
            session.draft.injured_spouse = Some(allocation());
            session.draft.income[0].owner = crate::DocumentOwner::from(owner);
            add(&mut session.draft, wage(0, Owner::Taxpayer, 30_000_000, 0));
            session = restart(session.draft);
            crate::tests::support::client::admit_native_fixture_facts(&mut session);
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            documents(&mut request)[0].document = input(&income);
            accepted(&send_fixture(&mut session, request));
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                session.draft.injured_spouse.as_mut().unwrap().injured = injured;
                let result = supported(&session.draft);
                assert_eq!(value(&result, "agi"), 299_500);
                assert_eq!(value(&result, "form_8960_net_investment_income"), 1_000);
                assert_eq!(value(&result, "net_investment_income_tax"), 38);
                // Corrected 2026-10-18: IRS i8379 (11/2024), line 13b,
                // https://www.irs.gov/instructions/i8379: all other income
                // includes $1,000 capital less $1,500 Schedule E = -$500.
                assert_eq!(
                    niit_optional_column(&result, "13b", "b"),
                    Some(if owner == injured { -500 } else { 0 })
                );
                // Corrected 2026-10-18: i8379 line 18 includes other taxes;
                // i8959 (2025), https://www.irs.gov/instructions/i8959:
                // taxpayer wages $300,000 less joint $250,000, at 0.9% = $450.
                assert_eq!(value(&result, "additional_medicare_tax"), 450);
                let niit = if owner == injured { 38 } else { 0 };
                let medicare = if injured == Owner::Taxpayer { 450 } else { 0 };
                assert_eq!(niit_optional_column(&result, "18", "b"), Some(niit + medicare));
                assert_injured_ordinary_unchanged(&session.draft, &result, supported);
                assert!(matches!(
                    send_fixture(&mut session, P::ReviewAccepted { user: TEST_USER }),
                    Reply::Response(P::Review { .. })
                ));
                let cold = restart(session.draft.clone());
                assert_eq!(supported(&cold.draft), result);
            }
        }
    }
}

#[test]
fn capital_gains_preserve_the_existing_scorp_stock_then_debt_owner() {
    let (session, mut income) = entity_capital_session(true);
    entity_capital_debt(&mut income, &session.draft.income[0].income);
    assert_eq!(entity_exempt_other_rule(&income), None);
    for (amt, allowed) in [(false, 2_000), (true, 1_700)] {
        let result = entity_exempt_other_output(&income, amt).unwrap();
        let basis = result.basis.scorp.as_ref().unwrap();
        assert_eq!(
            dollars(result.components.iter().map(|v| v.basis_allowed.get()).sum::<i64>()),
            allowed
        );
        assert_eq!(
            dollars(result.components.iter().map(|v| v.basis_suspended.get()).sum::<i64>()),
            2_000 - allowed
        );
        assert_eq!(basis.stock.get(), 0);
        assert_eq!(basis.loans[0].basis.get(), 0);
        assert_eq!(basis.loans[0].ordinary_gain.get() + basis.loans[0].capital_gain.get(), 0);
    }
}

#[test]
fn capital_character_uses_preferential_rates_and_independent_amt_capital() {
    for scorp in [false, true] {
        let (mut session, mut income) = entity_capital_session(scorp);
        zero_entity_capital_operating(&mut income);
        add(&mut session.draft, wage(0, Owner::Taxpayer, 10_000_000, 0));
        set_entity_capital(&mut income, [70_000, 0], [0, 30_000]);
        session.draft.income[0].income = income.clone();
        let short = supported(&session.draft);
        set_entity_capital(&mut income, [0, 70_000], [10_000, 20_000]);
        session.draft.income[0].income = income;
        let long = supported(&session.draft);
        // IRS Rev. Proc.2025-32 (2026 rates), https://www.irs.gov/irb/2025-45_IRB:
        // $700 falls wholly in the 22% ordinary/15% capital bands here.
        assert_eq!(value(&short, "agi"), 100_700);
        assert_eq!(value(&long, "agi"), 100_700);
        assert_eq!(value(&short, "income_tax") - value(&long, "income_tax"), 49);
        assert_eq!(value(&short, "amt_capital_gain"), 300);
        assert_eq!(value(&long, "amt_capital_gain"), 300);
        assert_eq!(value(&short, "amt_capital_adjustment"), -400);
        assert_eq!(value(&long, "amt_capital_adjustment"), -400);
    }
}

#[test]
fn entity_capital_gains_preserve_nol_nonbusiness_deduction_order() {
    for scorp in [false, true] {
        let (mut session, income) = entity_capital_session(scorp);
        session.draft.income[0].income = income;
        let result = supported(&session.draft);
        // Corrected 2026-10-18: IRS i172 (12/2024), Part I line 6,
        // https://www.irs.gov/instructions/i172: standard deduction is
        // nonbusiness. Its $1,000 used against portfolio gains remains;
        // only its excess is added back, preserving the $1,500 business NOL.
        assert_eq!(value(&result, "agi"), -500);
        assert_eq!(value(&result, "nol_carryforward"), 1_500);
        // Separate AMT: $1,200 ordinary loss less $400 capital gain.
        assert_eq!(value(&result, "amt_nol_carryforward"), 800);
    }
}
