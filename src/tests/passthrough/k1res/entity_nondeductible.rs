use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::assert::k1res::assert_atomic_rejection;
use crate::tests::support::assert::niit_optional_column;
use crate::tests::support::builders::k1res::{
    documents, entity_dividend_borrowing, entity_dividend_index, entity_dividend_output,
    entity_dividend_parts, graph, input, prepare_entity_nondeductible, set_entity_capital,
    set_entity_dividends, set_entity_nondeductible, set_partner_entity_interest,
    set_scorp_entity_interest,
};
use crate::tests::support::client::k1res::entity_dividend_session;
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::*;

#[test]
fn current_expenses_select_each_basis_owner_without_becoming_activity_expenses() {
    for scorp in [false, true] {
        let session =
            entity_dividend_session(scorp, complete(FederalStatus::Single), false, [0, 0], None);
        let index = entity_dividend_index(&session.draft);
        let mut income = session.draft.income[index].income.clone();
        prepare_entity_nondeductible(&mut income, 50_000, 20_000);
        for (amt, allowed, suspended, used) in
            [(false, 50_000, 50_000, 50_000), (true, 65_000, 35_000, 20_000)]
        {
            let output = entity_dividend_output(&income, amt).unwrap();
            assert_eq!(
                (
                    output.basis.ending.get(),
                    output.basis.nondeductible_used.get(),
                    output.basis.nondeductible_carry.get()
                ),
                (0, used, 0)
            );
            for component in &output.components {
                assert_eq!(
                    (
                        component.income.get(),
                        component.basis_allowed.get(),
                        component.basis_suspended.get(),
                        component.risk_allowed.get(),
                        component.risk_suspended.get()
                    ),
                    (0, allowed, suspended, allowed, 0)
                );
            }
            assert_eq!(output.groups.iter().map(|v| v.nonpassive).sum::<i64>(), -2 * allowed);
        }
        // The same source expense is applied once, alongside the whole ordinary
        // dividend amount; the qualified subset is not additional basis income.
        set_entity_dividends(&mut income, [100_000, 60_000], Some([100_000, 60_000]));
        set_entity_nondeductible(&mut income, 50_000, 20_000);
        assert!(crate::k1_activity::income_issue(&income).is_none());
        let output = entity_dividend_output(&income, false).unwrap();
        assert_eq!(output.components.iter().map(|v| v.basis_allowed.get()).sum::<i64>(), 200_000);
        assert_eq!(output.basis.ending.get(), 0);
    }
}

#[test]
fn mixed_portfolio_sources_use_basis_owners_with_independent_interest() {
    for scorp in [false, true] {
        let session =
            entity_dividend_session(scorp, complete(FederalStatus::Single), false, [0, 0], None);
        let baseline = &session.draft.income[entity_dividend_index(&session.draft)].income;
        for character in 0..3 {
            for expense_amt in [false, true] {
                for portfolio_amt in [false, true] {
                    let mut income = baseline.clone();
                    prepare_entity_nondeductible(
                        &mut income,
                        if expense_amt { 0 } else { 1 },
                        if expense_amt { 1 } else { 0 },
                    );
                    let (source, current, alternative) = entity_dividend_parts(&mut income);
                    let entity = if portfolio_amt {
                        let K1EntityOnlyAmt::Separate { amounts } =
                            &mut source.alternative_entity_only
                        else {
                            unreachable!()
                        };
                        amounts
                    } else {
                        source.entity_only.as_mut().unwrap()
                    };
                    let control =
                        if portfolio_amt { alternative.as_mut().unwrap() } else { current };
                    match character {
                        0 => {
                            entity.portfolio.short = 1;
                            control.portfolio.short = 1;
                        }
                        1 => {
                            entity.portfolio.long = 1;
                            control.portfolio.long = 1;
                        }
                        2 => {
                            entity.portfolio.interest = 1;
                            control.portfolio.interest = 1;
                        }
                        _ => unreachable!(),
                    }
                    assert!(crate::k1_activity::income_issue(&income).is_none());
                    for amt in [false, true] {
                        // 2026-10-20: IRS i1065sk1/i7203 increase basis for
                        // actual interest as well as gains, before current expenses.
                        let output = entity_dividend_output(&income, amt).unwrap();
                        let gain = i64::from(amt == portfolio_amt);
                        let expense = i64::from(amt == expense_amt);
                        if character < 2 {
                            assert_eq!(output.entity_capital[character], gain);
                            assert_eq!(output.entity_capital[1 - character], 0);
                        } else {
                            assert_eq!(output.entity_interest, gain);
                        }
                        assert_eq!(
                            output.components.iter().map(|v| v.basis_allowed.get()).sum::<i64>(),
                            150_000 + gain - expense
                        );
                        assert_eq!(output.basis.nondeductible_used.get(), expense);
                        assert_eq!(output.basis.nondeductible_carry.get(), 0);
                    }
                }
            }
        }
    }
}

#[test]
fn capital_and_expenses_share_native_basis_and_preserve_independent_source_lifecycle() {
    for scorp in [false, true] {
        let mut session =
            entity_dividend_session(scorp, complete(FederalStatus::Single), false, [0, 0], None);
        let index = entity_dividend_index(&session.draft);
        let mut income = session.draft.income[index].income.clone();
        prepare_entity_nondeductible(&mut income, 50_000, 20_000);
        set_entity_capital(&mut income, [30_000, 70_000], [10_000, 30_000]);
        set_entity_nondeductible(&mut income, 50_000, 20_000);
        assert!(crate::k1_activity::income_issue(&income).is_none());
        let ids = crate::k1_activity::source_ids(&income);
        let before = session.draft.clone();
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut session, request));
        let saved = session.draft.clone();
        assert_eq!(saved.revision, before.revision + 2);
        assert_eq!(saved.next_id, before.next_id);
        for (amt, gains, used, allowed) in
            [(false, [30_000, 70_000], 50_000, 100_000), (true, [10_000, 30_000], 20_000, 85_000)]
        {
            let output = entity_dividend_output(&saved.income[index].income, amt).unwrap();
            assert_eq!(output.entity_capital, gains);
            assert_eq!(output.basis.ending.get(), 0);
            assert_eq!(output.basis.nondeductible_used.get(), used);
            assert_eq!(output.basis.nondeductible_carry.get(), 0);
            for component in &output.components {
                assert_eq!(component.income.get(), 0);
                assert_eq!(component.basis_allowed.get(), allowed);
                assert_eq!(component.basis_suspended.get(), 100_000 - allowed);
                assert_eq!(component.risk_allowed.get(), allowed);
                assert_eq!(component.risk_suspended.get(), 0);
            }
            for group in &output.groups {
                assert_eq!(group.current, -allowed);
                assert_eq!(group.nonpassive, -allowed);
                assert_eq!(group.passive, 0);
                assert_eq!(group.prior_released, 0);
            }
        }
        let result = supported(&saved);
        let mut cold = restart(saved.clone());
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(
            documents(&mut request)[0].document,
            crate::tests::support::builders::k1res::expected_fixture_input(&session.draft, &income)
        );
        accepted(&send_fixture(&mut cold, request));
        assert_eq!(cold.draft, saved);
        assert_eq!(estimate(&cold.draft), result);
        set_entity_capital(&mut income, [30_000, 20_000], [10_000, 30_000]);
        set_entity_nondeductible(&mut income, 40_000, 20_000);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut cold, request));
        assert_eq!(cold.draft.revision, saved.revision + 2);
        assert_eq!(cold.draft.next_id, saved.next_id);
        assert_eq!(crate::k1_activity::source_ids(&cold.draft.income[index].income), ids);
        // REG basis is $1,500 + $500 gains - $400 expenses = $1,600.
        // AMT retains its independent $1,500 + $400 - $200 = $1,700.
        for (amt, gains, used, allowed) in
            [(false, [30_000, 20_000], 40_000, 80_000), (true, [10_000, 30_000], 20_000, 85_000)]
        {
            let output = entity_dividend_output(&cold.draft.income[index].income, amt).unwrap();
            assert_eq!(output.entity_capital, gains);
            assert_eq!(output.basis.nondeductible_used.get(), used);
            assert_eq!(output.basis.nondeductible_carry.get(), 0);
            assert!(
                output.components.iter().all(|component| component.basis_allowed.get() == allowed)
            );
        }
        assert_eq!(value(&supported(&cold.draft), "agi"), value(&result, "agi") - 100);
        let corrected = cold.draft.clone();
        let mut final_cold = restart(corrected.clone());
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut final_cold);
        assert_eq!(
            documents(&mut request)[0].document,
            crate::tests::support::builders::k1res::expected_fixture_input(&session.draft, &income)
        );
        accepted(&send_fixture(&mut final_cold, request));
        assert_eq!(final_cold.draft, corrected);
        assert_eq!(estimate(&final_cold.draft), estimate(&cold.draft));
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut final_cold);
        graph(&mut documents(&mut request)[0].document)
            .entity_only
            .as_mut()
            .unwrap()
            .nondeductible += 1;
        assert_atomic_rejection(&mut final_cold, request, "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE");
    }
}

#[test]
fn interest_and_expenses_keep_gross_investment_income_and_native_owner_lifecycle() {
    for scorp in [false, true] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = complete(FederalStatus::Joint);
            draft.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
            draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
            draft.injured_spouse = Some(allocation());
            draft.injured_spouse.as_mut().unwrap().injured = owner;
            let mut session = entity_dividend_session(scorp, draft, false, [0, 0], None);
            let index = entity_dividend_index(&session.draft);
            let mut income = session.draft.income[index].income.clone();
            prepare_entity_nondeductible(&mut income, 50_000, 20_000);
            match &mut income {
                FederalIncome::PartnershipActivities { data } => {
                    set_partner_entity_interest(data, 40_000)
                }
                FederalIncome::ScorpActivities { data } => set_scorp_entity_interest(data, 40_000),
                _ => unreachable!(),
            }
            set_entity_nondeductible(&mut income, 50_000, 20_000);
            let ids = crate::k1_activity::source_ids(&income);
            let before = session.draft.clone();
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            documents(&mut request)[0].owner = crate::DocumentOwner::from(owner);
            documents(&mut request)[0].document = input(&income);
            let request = fixture_owner_inputs(&session.draft, request);
            accepted(&send_fixture(&mut session, request));
            let saved = session.draft.clone();
            assert_eq!(saved.revision, before.revision + 2);
            assert_eq!(saved.next_id, before.next_id);
            for (amt, used, allowed) in [(false, 50_000, 70_000), (true, 20_000, 85_000)] {
                let output = entity_dividend_output(&saved.income[index].income, amt).unwrap();
                assert_eq!(output.entity_interest, 40_000);
                assert_eq!(output.basis.nondeductible_used.get(), used);
                assert_eq!(output.basis.nondeductible_carry.get(), 0);
                assert_eq!(output.basis.ending.get(), 0);
                for component in &output.components {
                    assert_eq!(component.income.get(), 0);
                    assert_eq!(component.basis_allowed.get(), allowed);
                    assert_eq!(component.risk_allowed.get(), allowed);
                }
            }
            let result = supported(&saved);
            assert_eq!(return_line(&result, "2b"), 40_000);
            assert_eq!(value(&result, "form_8960_net_investment_income"), 400);
            assert_eq!(value(&result, "net_investment_income_tax"), 15);
            // 2026-10-20: Form 8379 line 18 includes Additional Medicare tax.
            // The taxpayer's $300,000 wages exceed the joint $250,000 threshold
            // by $50,000; 0.9% is $450, independently of the K-1's $15 NIIT.
            for (key, expected) in [
                ("additional_medicare_wages_total", 300_000),
                ("additional_medicare_wages_threshold", 250_000),
                ("additional_medicare_wages_excess", 50_000),
                ("additional_medicare_tax", 450),
            ] {
                assert_eq!(value(&result, key), expected);
            }
            assert_eq!(niit_optional_column(&result, "18", "a"), Some(465));
            assert_eq!(
                niit_optional_column(&result, "18", "b"),
                Some(if owner == Owner::Taxpayer { 465 } else { 15 })
            );
            assert_eq!(
                niit_optional_column(&result, "18", "c"),
                Some(if owner == Owner::Taxpayer { 0 } else { 450 })
            );
            let mut borrowed = saved.clone();
            borrowed.injured_spouse = None;
            investment_borrowing(&mut borrowed, 1_000, 0, 0);
            let borrowing = supported(&borrowed);
            for prefix in ["", "amt_"] {
                assert_eq!(
                    value(&borrowing, &alloc::format!("{prefix}investment_form_line4a")),
                    400
                );
                assert_eq!(
                    value(&borrowing, &alloc::format!("{prefix}investment_interest_deduction")),
                    400
                );
            }
            set_entity_nondeductible(&mut income, 0, 20_000);
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            documents(&mut request)[0].document = input(&income);
            let request = fixture_owner_inputs(&session.draft, request);
            accepted(&send_fixture(&mut session, request));
            assert_eq!(session.draft.revision, saved.revision + 2);
            assert_eq!(session.draft.next_id, saved.next_id);
            assert_eq!(crate::k1_activity::source_ids(&session.draft.income[index].income), ids);
            for (amt, allowed) in [(false, 95_000), (true, 85_000)] {
                let output =
                    entity_dividend_output(&session.draft.income[index].income, amt).unwrap();
                assert!(
                    output
                        .components
                        .iter()
                        .all(|component| component.basis_allowed.get() == allowed)
                );
            }
            let corrected = supported(&session.draft);
            assert_eq!(value(&corrected, "agi"), value(&result, "agi") - 500);
            assert_eq!(return_line(&corrected, "2b"), 40_000);
            assert_eq!(value(&corrected, "net_investment_income_tax"), 15);
            assert_eq!(value(&corrected, "additional_medicare_tax"), 450);
            assert_eq!(niit_optional_column(&corrected, "18", "a"), Some(465));
            assert_eq!(
                niit_optional_column(&corrected, "18", "b"),
                Some(if owner == Owner::Taxpayer { 465 } else { 15 })
            );
            assert_eq!(
                niit_optional_column(&corrected, "18", "c"),
                Some(if owner == Owner::Taxpayer { 0 } else { 450 })
            );
            let mut cold = restart(session.draft.clone());
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut cold);
            assert_eq!(documents(&mut request)[0].owner, crate::DocumentOwner::from(owner));
            let expected = expected_named_source(
                &cold.draft,
                owner,
                IncomeSource::K1 { value: input(&income) },
            );
            let IncomeSource::K1 { value: expected } = expected else { unreachable!() };
            assert_eq!(documents(&mut request)[0].document, expected);
            let request = fixture_owner_inputs(&cold.draft, request);
            accepted(&send_fixture(&mut cold, request));
            assert_eq!(cold.draft, session.draft);
            assert_eq!(estimate(&cold.draft), corrected);
        }
    }
}

#[test]
fn capital_dividends_and_expenses_compose_once_through_native_tax_owners() {
    for scorp in [false, true] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = complete(FederalStatus::Joint);
            draft.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
            draft.screening.as_mut().unwrap().iso_exercises.push(iso_exercise(
                "Synthetic held ISO shares",
                1.0,
                100,
                20_000_100,
            ));
            draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
            draft.injured_spouse = Some(allocation());
            draft.injured_spouse.as_mut().unwrap().injured = owner;
            let mut session = entity_dividend_session(scorp, draft, false, [0, 0], None);
            let index = entity_dividend_index(&session.draft);
            let mut income = session.draft.income[index].income.clone();
            prepare_entity_nondeductible(&mut income, 50_000, 20_000);
            set_entity_dividends(&mut income, [50_000, 20_000], Some([30_000, 10_000]));
            set_entity_capital(&mut income, [30_000, 70_000], [10_000, 30_000]);
            set_entity_nondeductible(&mut income, 50_000, 20_000);
            let ids = crate::k1_activity::source_ids(&income);
            let before = session.draft.clone();
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            documents(&mut request)[0].owner = crate::DocumentOwner::from(owner);
            documents(&mut request)[0].document = input(&income);
            let request = fixture_owner_inputs(&session.draft, request);
            accepted(&send_fixture(&mut session, request));
            let saved = session.draft.clone();
            assert_eq!(saved.revision, before.revision + 2);
            assert_eq!(saved.next_id, before.next_id);
            for (amt, capital, dividends, expense, ending) in [
                (false, [30_000, 70_000], [50_000, 20_000], 50_000, 50_000),
                (true, [10_000, 30_000], [30_000, 10_000], 20_000, 0),
            ] {
                let output = entity_dividend_output(&saved.income[index].income, amt).unwrap();
                assert_eq!(output.entity_capital, capital);
                assert_eq!([output.entity_dividends, output.entity_qualified_dividends], dividends);
                // Qualified dividends are already included in ordinary dividends.
                assert_eq!(output.basis.ending.get(), ending);
                assert_eq!(output.basis.nondeductible_used.get(), expense);
                assert_eq!(output.basis.nondeductible_carry.get(), 0);
                assert!(
                    output
                        .components
                        .iter()
                        .all(|component| component.basis_allowed.get() == 100_000)
                );
            }
            let result = supported(&saved);
            assert_eq!(return_line(&result, "3b"), 50_000);
            assert_eq!(return_line(&result, "3a"), 20_000);
            assert_eq!(value(&result, "amt_capital_gain"), 400);
            assert_eq!(value(&result, "amt_capital_line_13"), 400);
            assert_eq!(value(&result, "form_8960_net_investment_income"), 1_500);
            assert_eq!(value(&result, "net_investment_income_tax"), 57);
            // 2026-10-20: line 18 combines this source's $57 NIIT with the
            // taxpayer's $450 Additional Medicare tax on $50,000 excess wages.
            for (key, expected) in [
                ("additional_medicare_wages_total", 300_000),
                ("additional_medicare_wages_threshold", 250_000),
                ("additional_medicare_wages_excess", 50_000),
                ("additional_medicare_tax", 450),
            ] {
                assert_eq!(value(&result, key), expected);
            }
            assert_eq!(niit_optional_column(&result, "18", "a"), Some(507));
            assert_eq!(
                niit_optional_column(&result, "18", "b"),
                Some(if owner == Owner::Taxpayer { 507 } else { 57 })
            );
            assert_eq!(
                niit_optional_column(&result, "18", "c"),
                Some(if owner == Owner::Taxpayer { 0 } else { 450 })
            );
            let mut ordinary = saved.clone();
            ordinary.injured_spouse = None;
            set_entity_dividends(
                &mut ordinary.income[index].income,
                [50_000, 0],
                Some([30_000, 0]),
            );
            let ordinary = supported(&ordinary);
            // These amounts occupy the 24% regular and 28% AMT ordinary
            // brackets, versus 15% for qualified dividends and long-term gains.
            // 2026-10-20: income_tax includes the binding AMT from the held ISO;
            // regular_income_tax separately reports the regular preferential tax.
            assert_eq!(
                value(&ordinary, "regular_income_tax") - value(&result, "regular_income_tax"),
                18
            );
            assert!(value(&result, "alternative_minimum_tax") > 0);
            assert_eq!(value(&ordinary, "income_tax") - value(&result, "income_tax"), 13);
            assert_eq!(
                value(&ordinary, "amt_tax_before_foreign_credit")
                    - value(&result, "amt_tax_before_foreign_credit"),
                13
            );
            let mut borrowed = saved.clone();
            borrowed.injured_spouse = None;
            entity_dividend_borrowing(&mut borrowed);
            let borrowing = supported(&borrowed);
            // 2026-10-20: Form 4952 line 4a excludes disposition gains;
            // short-term gains enter line 4h through lines 4d minus 4e.
            // REG: $500 - $200 + $300 = $600; AMT: $300 - $100 + $100 = $300.
            for (prefix, gross, qualified, allowed) in
                [("", 500, 200, 600), ("amt_", 300, 100, 300)]
            {
                assert_eq!(
                    value(&borrowing, &alloc::format!("{prefix}investment_form_line4a")),
                    gross
                );
                assert_eq!(
                    value(&borrowing, &alloc::format!("{prefix}investment_form_line4b")),
                    qualified
                );
                assert_eq!(
                    value(&borrowing, &alloc::format!("{prefix}investment_interest_deduction")),
                    allowed
                );
            }
            set_entity_dividends(&mut income, [25_000, 10_000], Some([30_000, 10_000]));
            set_entity_capital(&mut income, [30_000, 20_000], [10_000, 30_000]);
            set_entity_nondeductible(&mut income, 40_000, 20_000);
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            documents(&mut request)[0].document = input(&income);
            let request = fixture_owner_inputs(&session.draft, request);
            accepted(&send_fixture(&mut session, request));
            assert_eq!(session.draft.revision, saved.revision + 2);
            assert_eq!(session.draft.next_id, saved.next_id);
            assert_eq!(crate::k1_activity::source_ids(&session.draft.income[index].income), ids);
            for (amt, allowed) in [(false, 92_500), (true, 100_000)] {
                let output =
                    entity_dividend_output(&session.draft.income[index].income, amt).unwrap();
                assert!(
                    output
                        .components
                        .iter()
                        .all(|component| component.basis_allowed.get() == allowed)
                );
                assert_eq!(output.basis.ending.get(), 0);
            }
            let corrected = supported(&session.draft);
            assert_eq!(value(&corrected, "agi"), value(&result, "agi") - 600);
            assert_eq!(return_line(&corrected, "3b"), 25_000);
            assert_eq!(return_line(&corrected, "3a"), 10_000);
            assert_eq!(value(&corrected, "form_8960_net_investment_income"), 750);
            assert_eq!(value(&corrected, "net_investment_income_tax"), 29);
            assert_eq!(value(&corrected, "additional_medicare_tax"), 450);
            assert_eq!(niit_optional_column(&corrected, "18", "a"), Some(479));
            assert_eq!(
                niit_optional_column(&corrected, "18", "b"),
                Some(if owner == Owner::Taxpayer { 479 } else { 29 })
            );
            assert_eq!(
                niit_optional_column(&corrected, "18", "c"),
                Some(if owner == Owner::Taxpayer { 0 } else { 450 })
            );
            let mut cold = restart(session.draft.clone());
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut cold);
            assert_eq!(documents(&mut request)[0].owner, crate::DocumentOwner::from(owner));
            let expected = expected_named_source(
                &cold.draft,
                owner,
                IncomeSource::K1 { value: input(&income) },
            );
            let IncomeSource::K1 { value: expected } = expected else { unreachable!() };
            assert_eq!(documents(&mut request)[0].document, expected);
            let request = fixture_owner_inputs(&cold.draft, request);
            accepted(&send_fixture(&mut cold, request));
            assert_eq!(cold.draft, session.draft);
            assert_eq!(estimate(&cold.draft), corrected);
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut cold);
            graph(&mut documents(&mut request)[0].document)
                .entity_only
                .as_mut()
                .unwrap()
                .portfolio
                .qualified_dividends = 25_001;
            assert_atomic_rejection(&mut cold, request, "K1_ACTIVITY_SOURCE_AMOUNT_INVALID");
        }
    }
}

#[test]
fn exact_cent_boundary_and_excess_follow_native_owner_election_and_carry() {
    for scorp in [false, true] {
        let session =
            entity_dividend_session(scorp, complete(FederalStatus::Single), false, [0, 0], None);
        let index = entity_dividend_index(&session.draft);
        // 2026-10-20: basis worksheet stages retain exact source cents.
        // https://www.irs.gov/instructions/i1065sk1
        for (cents, allowed, used, carry) in [
            (0, 150_000, 0, 0),
            (49, 149_951, 49, 0),
            (50, 149_950, 50, 0),
            (200_000, 0, 150_000, if scorp { 0 } else { 50_000 }),
        ] {
            let mut income = session.draft.income[index].income.clone();
            prepare_entity_nondeductible(&mut income, cents, cents);
            let output = entity_dividend_output(&income, false).unwrap();
            assert_eq!(
                output.components.iter().map(|v| v.basis_allowed.get()).sum::<i64>(),
                allowed
            );
            assert_eq!(
                (output.basis.nondeductible_used.get(), output.basis.nondeductible_carry.get()),
                (used, carry)
            );
        }
        if scorp {
            let mut income = session.draft.income[index].income.clone();
            prepare_entity_nondeductible(&mut income, 50_000, 20_000);
            let FederalIncome::ScorpActivities { data } = &mut income else { unreachable!() };
            data.entity.regular.losses_first = true;
            data.entity.alternative.as_mut().unwrap().losses_first = true;
            for (amt, carry) in [(false, 50_000), (true, 20_000)] {
                let output = entity_dividend_output(&income, amt).unwrap();
                assert_eq!(
                    (output.basis.nondeductible_used.get(), output.basis.nondeductible_carry.get()),
                    (0, carry)
                );
                for component in output.components {
                    assert_eq!(
                        (component.basis_allowed.get(), component.basis_suspended.get()),
                        (75_000, 25_000)
                    );
                }
            }
        }
    }
}

#[test]
fn native_save_review_reopen_unchanged_restart_and_regular_correction() {
    for scorp in [false, true] {
        let mut session =
            entity_dividend_session(scorp, complete(FederalStatus::Single), false, [0, 0], None);
        let index = entity_dividend_index(&session.draft);
        let mut income = session.draft.income[index].income.clone();
        prepare_entity_nondeductible(&mut income, 50_000, 20_000);
        let ids = crate::k1_activity::source_ids(&income);
        let before = session.draft.clone();
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut session, request));
        assert_eq!(
            (session.draft.revision, session.draft.next_id),
            (before.revision + 2, before.next_id)
        );
        let result = supported(&session.draft);
        assert_eq!(value(&result, "agi"), value(&supported(&before), "agi") - 1_000);
        for (key, expected) in [
            ("NOTE_K1_ENTITY_NONDEDUCTIBLE_USED", "$500"),
            ("NOTE_AMT_K1_ENTITY_NONDEDUCTIBLE_USED", "$200"),
        ] {
            let note = result.review_notes.iter().find(|v| v.display.key == key).unwrap();
            assert!(status_text(&note.display.encode()).contains(expected));
        }
        assert!(matches!(
            send_fixture(&mut session, P::ReviewAccepted { user: TEST_USER }),
            Reply::Response(P::Review { .. })
        ));
        let saved = session.draft.clone();
        let mut cold = restart(saved.clone());
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(
            documents(&mut request)[0].document,
            crate::tests::support::builders::k1res::expected_fixture_input(&session.draft, &income)
        );
        accepted(&send_fixture(&mut cold, request));
        assert_eq!(cold.draft, saved);
        assert_eq!(estimate(&cold.draft), result);
        set_entity_nondeductible(&mut income, 0, 20_000);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut cold, request));
        assert_eq!(cold.draft.revision, saved.revision + 2);
        assert_eq!(cold.draft.next_id, saved.next_id);
        assert_eq!(crate::k1_activity::source_ids(&cold.draft.income[index].income), ids);
        assert_eq!(
            entity_dividend_output(&income, true).unwrap().components[0].basis_allowed.get(),
            65_000
        );
        assert_eq!(value(&supported(&cold.draft), "agi"), value(&result, "agi") - 500);
        let mut final_cold = restart(cold.draft.clone());
        let request =
            crate::tests::support::client::income_pages::open_document_center(&mut final_cold);
        accepted(&send_fixture(&mut final_cold, request));
        assert_eq!(final_cold.draft, cold.draft);
        assert_eq!(estimate(&final_cold.draft), estimate(&cold.draft));
        // An affirmative identical-source statement survives the actual native
        // Save and persisted restart, rather than being replaced by Separate.
        set_entity_nondeductible(&mut income, 20_000, 20_000);
        entity_dividend_parts(&mut income).0.alternative_entity_only =
            K1EntityOnlyAmt::SameAsRegular;
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut final_cold);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut final_cold, request));
        let identical_saved = final_cold.draft.clone();
        let mut identical_cold = restart(identical_saved.clone());
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut identical_cold);
        assert_eq!(
            documents(&mut request)[0].document,
            crate::tests::support::builders::k1res::expected_fixture_input(&session.draft, &income)
        );
        accepted(&send_fixture(&mut identical_cold, request));
        assert_eq!(identical_cold.draft, identical_saved);
        assert_eq!(crate::k1_activity::source_ids(&identical_cold.draft.income[index].income), ids);
        for amt in [false, true] {
            let output =
                entity_dividend_output(&identical_cold.draft.income[index].income, amt).unwrap();
            assert_eq!(output.basis.nondeductible_used.get(), 20_000);
            assert_eq!(output.components[0].basis_allowed.get(), 65_000);
        }
    }
}

#[test]
fn valid_unknowns_and_prior_history_keep_known_facts_without_zero_fallback() {
    for scorp in [false, true] {
        let session =
            entity_dividend_session(scorp, complete(FederalStatus::Single), false, [0, 0], None);
        let index = entity_dividend_index(&session.draft);
        let mut baseline = session.draft.income[index].income.clone();
        prepare_entity_nondeductible(&mut baseline, 50_000, 20_000);
        for boundary in 0..5 {
            let mut income = baseline.clone();
            match boundary {
                0 => entity_dividend_parts(&mut income).0.entity_only = None,
                1 => {
                    entity_dividend_parts(&mut income).0.alternative_entity_only =
                        K1EntityOnlyAmt::Unavailable
                }
                2 => *entity_dividend_parts(&mut income).2 = None,
                3..=4 => match &mut income {
                    FederalIncome::PartnershipActivities { data } => {
                        data.entity.regular.complete = boundary != 3;
                        data.entity.regular.prior_nondeductible =
                            if boundary == 4 { Some(1) } else { None };
                    }
                    FederalIncome::ScorpActivities { data } => {
                        data.entity.regular.complete = boundary != 3;
                        data.entity.regular.prior_nondeductible =
                            if boundary == 4 { Some(1) } else { None };
                        data.entity.regular.losses_first = boundary == 4;
                    }
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            }
            // Unknown prior history remains incomplete; a positive actual prior
            // retains its existing unsupported calculation boundary.
            assert!(
                crate::k1_activity::income_issue(&income).is_none(),
                "scorp={scorp}, boundary={boundary}"
            );
            let mut draft = session.draft.clone();
            draft.income[index].income = income.clone();
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
            assert_eq!(draft.income[index].income, income);
            if boundary >= 3 {
                assert_eq!(
                    entity_dividend_output(&income, true).unwrap().basis.nondeductible_used.get(),
                    20_000
                );
            }
        }
    }
}

#[test]
fn native_invalid_controls_are_atomic() {
    for scorp in [false, true] {
        let mut session =
            entity_dividend_session(scorp, complete(FederalStatus::Single), false, [0, 0], None);
        let index = entity_dividend_index(&session.draft);
        let mut income = session.draft.income[index].income.clone();
        prepare_entity_nondeductible(&mut income, 50_000, 20_000);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut session, request));
        for invalid in 0..2 {
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            let source = graph(&mut documents(&mut request)[0].document);
            let key = match invalid {
                0 => {
                    source.entity_only.as_mut().unwrap().nondeductible += 1;
                    "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"
                }
                1 => {
                    let K1EntityOnlyAmt::Separate { amounts } = &mut source.alternative_entity_only
                    else {
                        unreachable!()
                    };
                    amounts.nondeductible += 1;
                    "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"
                }
                _ => unreachable!(),
            };
            assert_atomic_rejection(&mut session, request, key);
        }
        // Completeness requires this separate prior-history control; it cannot
        // be inferred as zero from a fully known current expense.
        let mut missing_prior = income.clone();
        match &mut missing_prior {
            FederalIncome::PartnershipActivities { data } => {
                data.entity.regular.prior_nondeductible = None
            }
            FederalIncome::ScorpActivities { data } => {
                data.entity.regular.prior_nondeductible = None
            }
            _ => unreachable!(),
        }
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&missing_prior);
        assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_COMPLETE_BASIS_SOURCE_MISSING");
    }
}

#[test]
fn scorp_debt_basis_reduction_reuses_the_financial_owner() {
    let session =
        entity_dividend_session(true, complete(FederalStatus::Single), false, [0, 0], None);
    let mut income = session.draft.income[entity_dividend_index(&session.draft)].income.clone();
    prepare_entity_nondeductible(&mut income, 50_000, 20_000);
    let FederalIncome::ScorpActivities { data } = &mut income else { unreachable!() };
    data.entity.loans.push(ScorpEntityLoan {
        regular: ScorpLoan {
            kind: ScorpLoanKind::FormalNote,
            repayment_term: ScorpNoteHolding::NoRepayment,
            principal: 100_000,
            basis: 100_000,
            advances: 0,
            repayments: 0,
        },
        alternative_basis: Some(100_000),
    });
    assert!(crate::k1_activity::income_issue(&income).is_none());
    for (amt, debt) in [(false, 0), (true, 30_000)] {
        let output = entity_dividend_output(&income, amt).unwrap();
        assert_eq!(output.components.iter().map(|v| v.basis_allowed.get()).sum::<i64>(), 200_000);
        assert_eq!(output.basis.scorp.unwrap().loans[0].basis.get(), debt);
    }
}
