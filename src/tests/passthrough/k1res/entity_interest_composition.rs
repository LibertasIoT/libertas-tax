use crate::tests::cross::interview::accepted;
use crate::tests::support::assert::niit_optional_column;
use crate::tests::support::builders::k1res::{
    CurrentLossQbiFacts, documents, entity_dividend_borrowing, entity_dividend_index,
    entity_dividend_output, input, prepare_entity_nondeductible, set_entity_capital,
    set_entity_dividends, set_entity_exempt_interest, set_entity_exempt_other, set_entity_interest,
    set_entity_nondeductible,
};
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{
    entity_dividend_session, ordinary_multi_current_loss_qbi_session, scorp_cash_advance_candidate,
    scorp_cash_repayment_candidate,
};
use crate::tests::*;

#[test]
fn all_entity_income_characters_compose_with_expenses_and_native_correction() {
    for scorp in [false, true] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = complete(FederalStatus::Joint);
            draft.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
            draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
            draft.injured_spouse = Some(allocation());
            draft.injured_spouse.as_mut().unwrap().injured = owner;
            let mut session = entity_dividend_session(scorp, draft, false, [0, 0], None);
            let index = entity_dividend_index(&session.draft);
            session.draft.income[index].owner = crate::DocumentOwner::from(owner);
            let mut candidate = session.draft.income[index].income.clone();
            prepare_entity_nondeductible(&mut candidate, 20_000, 30_000);
            set_entity_capital(&mut candidate, [3_000, 7_000], [1_000, 3_000]);
            set_entity_dividends(&mut candidate, [10_000, 6_000], Some([15_000, 8_000]));
            set_entity_interest(&mut candidate, 10_000, 20_000);
            set_entity_exempt_other(&mut candidate, 5_000, 7_000);
            set_entity_exempt_interest(&mut candidate, 4_000, 6_000);
            set_entity_nondeductible(&mut candidate, 20_000, 30_000);
            let ids = crate::k1_activity::source_ids(&candidate);
            let before = session.draft.clone();
            let dirty = session.dirty;
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            documents(&mut request)[0].document = input(&candidate);

            assert!(matches!(
                crate::tests::support::client::income_pages::candidate_save_review(
                    &mut session,
                    crate::tests::support::client::estimate_documents_from_save(&request)
                ),
                Reply::Response(P::Review { .. })
            ));
            assert_eq!((session.draft.clone(), session.dirty), (before.clone(), dirty));
            accepted(&send_fixture(&mut session, request));
            let saved = session.draft.clone();
            assert_eq!((saved.revision, saved.next_id), (before.revision + 2, before.next_id));
            assert_eq!(crate::k1_activity::source_ids(&saved.income[index].income), ids);
            for (amt, allowed, expense, interest, dividends, capital, exempt) in [
                (false, 169_000, 20_000, 10_000, 10_000, [3_000, 7_000], 4_000),
                (true, 172_000, 30_000, 20_000, 15_000, [1_000, 3_000], 6_000),
            ] {
                let output = entity_dividend_output(&saved.income[index].income, amt).unwrap();
                // IRS i1065sk1/i7203: all income increases basis once;
                // qualified dividends are a subset, not another basis increase.
                assert_eq!(
                    output.components.iter().map(|c| c.basis_allowed.get()).sum::<i64>(),
                    allowed
                );
                assert_eq!(output.basis.nondeductible_used.get(), expense);
                assert_eq!(output.entity_interest, interest);
                assert_eq!(output.entity_dividends, dividends);
                assert_eq!(output.entity_capital, capital);
                assert_eq!(output.entity_exempt_interest, exempt);
                assert_eq!(output.basis.ending.get(), 0);
                assert!(
                    output
                        .components
                        .iter()
                        .all(|c| c.income.get() == 0 && c.risk_suspended.get() == 0)
                );
                assert_eq!(output.groups.iter().map(|g| g.nonpassive).sum::<i64>(), -allowed);
            }
            let result = supported(&saved);
            assert_eq!((return_line(&result, "2a"), return_line(&result, "2b")), (4_000, 10_000));
            assert_eq!((return_line(&result, "3b"), return_line(&result, "3a")), (10_000, 6_000));
            assert_eq!(value(&result, "form_8960_net_investment_income"), 300);
            assert_eq!(niit_optional_column(&result, "13b", "b"), Some(-1_390));
            let mut cold = restart(saved.clone());
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut cold);
            assert_eq!(
                documents(&mut request)[0].document,
                crate::tests::support::builders::k1res::expected_owned_fixture_input(
                    &session.draft,
                    owner,
                    &candidate
                )
            );
            accepted(&send_fixture(&mut cold, request));
            assert_eq!(cold.draft, saved);
            assert_eq!(estimate(&cold.draft), result);
            set_entity_interest(&mut candidate, 0, 20_000);
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut cold);
            documents(&mut request)[0].document = input(&candidate);
            accepted(&send_fixture(&mut cold, request));
            assert_eq!(
                (cold.draft.revision, cold.draft.next_id),
                (saved.revision + 2, saved.next_id)
            );
            assert_eq!(crate::k1_activity::source_ids(&cold.draft.income[index].income), ids);
            let corrected = supported(&cold.draft);
            assert_eq!(return_line(&corrected, "2b"), 0);
            assert_eq!(value(&corrected, "form_8960_net_investment_income"), 200);
            let amt = entity_dividend_output(&cold.draft.income[index].income, true).unwrap();
            assert_eq!(amt.components.iter().map(|c| c.basis_allowed.get()).sum::<i64>(), 172_000);
            assert_eq!(estimate(&restart(cold.draft.clone()).draft), corrected);
        }
    }
}

#[test]
fn mixed_portfolio_basis_stays_outside_binding_risk_and_retained_passive_groups() {
    for scorp in [false, true] {
        let mut session = entity_dividend_session(
            scorp,
            complete(FederalStatus::Single),
            true,
            [100_000, 60_000],
            Some([150_000, 80_000]),
        );
        let index = entity_dividend_index(&session.draft);
        let income = &mut session.draft.income[index].income;
        set_entity_interest(income, 100_000, 50_000);
        set_entity_capital(income, [3_000, 7_000], [1_000, 3_000]);
        set_entity_exempt_other(income, 5_000, 7_000);
        set_entity_exempt_interest(income, 4_000, 6_000);
        set_entity_nondeductible(income, 20_000, 30_000);
        for (amt, basis_allowed, basis_suspended, risk_suspended) in
            [(false, 699_000, 1_000, 749_000), (true, 687_000, 13_000, 737_000)]
        {
            let output = entity_dividend_output(income, amt).unwrap();
            let loss = &output.components[1];
            assert_eq!(
                (loss.basis_allowed.get(), loss.basis_suspended.get()),
                (basis_allowed, basis_suspended)
            );
            assert_eq!(
                (loss.risk_allowed.get(), loss.risk_suspended.get()),
                (50_000, risk_suspended)
            );
            // IRS i6198/i8582: outside portfolio/exempt income affects basis,
            // not this group's $500 risk allowance or its $2,000 PAL history.
            assert_eq!(
                (output.groups[0].passive, output.groups[1].nonpassive, output.groups[1].prior),
                (400_000, -50_000, 200_000)
            );
        }
    }
}

#[test]
fn interest_amt_zero_crossings_and_annual_rounding_feed_4952_and_6251() {
    for scorp in [false, true] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            for (regular, amt, regular_dollars, amt_dollars) in [
                (0, 100_000, 0, 1_000),
                (100_000, 0, 1_000, 0),
                (100_024, 100_025, 1_000, 1_001),
                (100_025, 100_024, 1_001, 1_000),
                (100_026, 100_025, 1_001, 1_001),
            ] {
                let mut session = entity_dividend_session(
                    scorp,
                    complete(FederalStatus::Joint),
                    false,
                    [0, 0],
                    None,
                );
                add(
                    &mut session.draft,
                    income(FederalIncome::Interest {
                        data: interest_source(native_interest(25, 0)),
                    }),
                );
                let index = entity_dividend_index(&session.draft);
                session.draft.income[index].owner = crate::DocumentOwner::from(owner);
                let direct = session.draft.income.last_mut().unwrap();
                direct.owner = if owner == Owner::Taxpayer {
                    crate::DocumentOwner::Spouses
                } else {
                    crate::DocumentOwner::Yours
                };
                let baseline = supported(&session.draft);
                set_entity_interest(&mut session.draft.income[index].income, regular, amt);
                let result = supported(&session.draft);
                assert_eq!(return_line(&result, "2b"), regular_dollars * 100);
                assert_eq!(value(&result, "agi") - value(&baseline, "agi"), regular_dollars);
                assert_eq!(
                    value(&result, "alternative_minimum_taxable_income")
                        - value(&baseline, "alternative_minimum_taxable_income"),
                    amt_dollars
                );
                for (alternative, cents) in [(false, regular), (true, amt)] {
                    let output =
                        entity_dividend_output(&session.draft.income[index].income, alternative)
                            .unwrap();
                    assert_eq!(output.basis.ending.get(), 100_000 + cents);
                }
                entity_dividend_borrowing(&mut session.draft);
                let borrowed = supported(&session.draft);
                for (prefix, expected) in [("", regular_dollars), ("amt_", amt_dollars)] {
                    assert_eq!(
                        value(&borrowed, &alloc::format!("{prefix}investment_form_line4a")),
                        expected
                    );
                    assert_eq!(
                        value(&borrowed, &alloc::format!("{prefix}investment_interest_deduction")),
                        expected
                    );
                    assert_eq!(
                        value(
                            &borrowed,
                            &alloc::format!("{prefix}investment_interest_carryforward")
                        ),
                        3_000 - expected
                    );
                }
            }
        }
    }
}

#[test]
fn current_qualified_profit_and_fully_allowed_loss_keep_their_component_owners() {
    for scorp in [false, true] {
        let mut session =
            ordinary_multi_current_loss_qbi_session(scorp, &CurrentLossQbiFacts::default(), |_| {});
        let index = entity_dividend_index(&session.draft);
        set_entity_interest(&mut session.draft.income[index].income, 100_000, 150_000);
        let output = entity_dividend_output(&session.draft.income[index].income, false).unwrap();
        assert_eq!(
            output.current_qualified.iter().map(|v| v.activity.income).collect::<Vec<_>>(),
            vec![4_000, -1_500]
        );
        assert_eq!(output.components[1].basis_allowed.get(), 200_000);
        assert_eq!(output.components[1].risk_allowed.get(), 200_000);
        let result = supported(&session.draft);
        assert_eq!(value(&result, "qbi_deduction"), 500);
        assert_eq!(value(&result, "agi"), 63_000);
    }
}

#[test]
fn interest_uses_existing_stock_debt_restoration_advances_and_zero_gain_repayments() {
    for repayment in [false, true] {
        let (_, data) = if repayment {
            scorp_cash_repayment_candidate(100_000)
        } else {
            scorp_cash_advance_candidate(100_000)
        };
        let mut candidate = FederalIncome::ScorpActivities { data: data.clone() };
        set_entity_interest(&mut candidate, 10_000, 20_000);
        for (amt, extra) in [(false, 10_000), (true, 20_000)] {
            let before = data.ordinary_source(amt).unwrap();
            let after = entity_dividend_output(&candidate, amt).unwrap();
            // IRS i7203: these sources already allow all $7,000 of basis
            // losses, so additional interest remains in ending stock/debt basis.
            assert_eq!(before.components[1].basis_allowed.get(), 700_000);
            assert_eq!(after.components[1].basis_allowed.get(), 700_000);
            let old = before.basis.scorp.as_ref().unwrap();
            let new = after.basis.scorp.as_ref().unwrap();
            assert_eq!(
                new.stock.get() + new.loans[0].basis.get(),
                old.stock.get() + old.loans[0].basis.get() + extra
            );
            assert_eq!(new.loans[0].principal, old.loans[0].principal);
            assert_eq!((new.loans[0].ordinary_gain.get(), new.loans[0].capital_gain.get()), (0, 0));
            assert_eq!(after.components[1].risk_allowed, before.components[1].risk_allowed);
        }
        // IRS i7203: a repayment exceeding recovered basis still requires
        // its gain character; portfolio interest cannot suppress that boundary.
        if repayment {
            let FederalIncome::ScorpActivities { data } = &mut candidate else { unreachable!() };
            data.entity.loans[0].regular.basis = 0;
            data.entity.loans[0].alternative_basis = Some(0);
            data.entity.loans[0].regular.repayments = 500_000;
            let crate::K1ComponentStatement::Entered { source } = &mut data.activities else {
                unreachable!()
            };
            source.risk_groups[1].investment_parts.as_mut().unwrap().loan_repayments = 500_000;
            assert!(data.ordinary_source(false).is_err());
            assert!(data.ordinary_source(true).is_err());
        }
    }
}

#[test]
fn independent_interest_does_not_change_business_nol_character() {
    for scorp in [false, true] {
        let facts = CurrentLossQbiFacts {
            gross_loss: 2_000_000,
            qualified: [400_000, -1_500_000],
            opening_basis: 3_000_000,
            loss_risk_capacity: 3_000_000,
        };
        let mut session = ordinary_multi_current_loss_qbi_session(scorp, &facts, |income| {
            set_entity_interest(income, 100_000, 150_000);
        });
        session.draft.income.retain(|entry| !matches!(entry.income, FederalIncome::Wage { .. }));
        entity_dividend_borrowing(&mut session.draft);
        let result = supported(&session.draft);
        // IRS Pub. 536/i4952: the independently allowed investment-interest
        // deduction offsets each system's portfolio income, leaving the
        // $16,000 business loss in each NOL without a personal excess deduction.
        assert_eq!(value(&result, "current_net_operating_loss"), 16_000);
        assert_eq!(value(&result, "amt_current_net_operating_loss"), 16_000);
    }
}
