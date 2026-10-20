use super::*;
use crate::tests::cross::interview::{accepted, asset, business, save_businesses, stored_business};

#[test]
fn unit17_prepared_callback_borrows_selected_assets_at_the_hypothetical_stage() {
    let mut draft =
        business_with_bonus(10_000, elected_asset(10_000, 3_000, BonusElection::ElectOut));
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.assets[0].acquired = 20160928;
    let native_business = data.clone();
    // The native API fixture includes an election/office history combination
    // that the ordinary page cannot roundtrip. Save proves identity only;
    // restore every original native business fact after actual ID assignment.
    let mut page = business(data.receipts());
    let mut property = asset(data.assets[0].cost, data.assets[0].placed, AssetClass::Five);
    property.acquired = Some(data.assets[0].acquired);
    property.history = AssetHistory::New {
        first_year: FirstYearDeduction::Section179 {
            description: "Synthetic identity source".into(),
            elected: data.assets[0].elected179(),
        },
    };
    page.assets = vec![property];
    let mut session = restart(complete(FederalStatus::Single));
    let next_id = session.draft.next_id;
    accepted(&save_businesses(&mut session, vec![page]));
    let (source_id, saved_business) = stored_business(&session);
    assert_eq!(source_id, draft.income[1].id);
    let asset_id = saved_business.assets[0].id;
    assert!(asset_id > 0);
    assert_eq!(source_id, next_id);
    assert_eq!(session.draft.next_id, asset_id + 1);
    let mut draft = session.draft;
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    *data = native_business;
    data.assets[0].id = asset_id;
    add_simplified_office(&mut draft, source_id, 300);
    assert!(draft.valid_stored());
    let original = draft.clone();
    // The same $7,000 depreciable basis gives regular $1,400 or AMT $1,050
    // before179 MACRS, leaving these actual selected operating amounts.
    for (system, method, operating) in [
        (Section179IncomeSystem::Regular, EquipmentMethod::Regular, 860_000),
        (Section179IncomeSystem::Alternative, EquipmentMethod::Declining150, 895_000),
    ] {
        let sources = [Section179DirectBusinessIncomeSource {
            source: 2,
            system,
            operating_net_cents: Some(operating),
            section1231_net_cents: Some(0),
        }];
        let mut calls = 0;
        let selected = crate::section179::section179_with_pass_through_context(
            &draft,
            system,
            &sources,
            &[],
            Some(&crate::home_office::HomeContext::default()),
            &[],
            |view, allocation, _| {
                calls += 1;
                assert_eq!(view.deductions, draft.deductions);
                let FederalIncome::Business { data } = &view.income[1].income else {
                    unreachable!()
                };
                assert_eq!(data.assets[0].method, method);
                assert_eq!(
                    data.receipts()
                        - data.operating_cost_with_adjustment(
                            crate::depreciation::mid_quarter(view),
                            crate::farm_conservation::expense_adjustment(view, 2, data),
                        ),
                    operating,
                );
                assert_eq!(allocation.get(&2), Some(&300_000));
                let office = crate::home_office::deduction_with_context(
                    view,
                    source_id,
                    &alloc::collections::BTreeMap::from([(
                        source_id,
                        crate::home_office::HomeIncome {
                            gross_cents: data.receipts(),
                            operating_cents: data.receipts() - operating + allocation[&2],
                        },
                    )]),
                    &crate::home_office::HomeContext::default(),
                )
                .unwrap();
                assert_eq!(office, 150_000);
                Some(Section179OtherLimitedDeduction { system, amount_cents: office })
            },
        )
        .unwrap();
        assert_eq!(calls, 1);
        assert_eq!(selected.income.total_allowed_cents, 300_000);
        assert_eq!(selected.allowed(source_id), Ok(300_000));
        let before179_cost = 1_000_000 - operating;
        assert_eq!(selected.operating(source_id), Ok(before179_cost + 300_000));
        assert_eq!(selected.tentative_profit(source_id), Ok(operating - 300_000));
        assert_eq!(selected.office(source_id), Ok(150_000));
        assert_eq!(selected.expenses(source_id), Ok(before179_cost + 450_000));
        assert_eq!(selected.allowed(0), Err(Section179PreparedIssue::UnknownSource(0)));
        assert_eq!(
            selected.operating(draft.income[0].id),
            Err(Section179PreparedIssue::WrongSourceKind(draft.income[0].id))
        );
        // A different supplied other-deduction stage binds a smaller native
        // allowance. Every reader must use that allocation, never calculate(d).
        let limited = crate::section179::section179_with_pass_through_context(
            &draft,
            system,
            &sources,
            &[],
            Some(&crate::home_office::HomeContext::default()),
            &[],
            |_, _, _| {
                Some(Section179OtherLimitedDeduction {
                    system,
                    amount_cents: 6_000_000 + operating - 100_000,
                })
            },
        )
        .unwrap();
        assert_eq!(limited.allowed(source_id), Ok(100_000));
        assert_eq!(
            crate::section179_return::prepare_with_home(
                &draft,
                system,
                &crate::home_office::HomeContext::default()
            )
            .unwrap()
            .allocation
            .allowed(source_id),
            Ok(300_000)
        );
        assert_eq!(limited.operating(source_id), Ok(before179_cost + 100_000));
        assert_eq!(limited.tentative_profit(source_id), Ok(operating - 100_000));
        assert_eq!(limited.office(source_id), Ok(150_000));
        assert_eq!(limited.expenses(source_id), Ok(before179_cost + 250_000));
        assert_eq!(draft, original);
    }
}

#[test]
fn unit17_prepared_callback_keeps_refusal_checks_and_zero_availability_timing() {
    let mut draft = ptp179_draft(20_000, 5_000);
    let system = Section179IncomeSystem::Regular;
    let sources = [Section179BusinessIncomeSource {
        source: 2,
        system,
        ordinary: Section179IncomeClass {
            current_net_cents: 2_000_000,
            prior_allowed_loss_cents: 0,
        },
        section1231: Section179IncomeClass { current_net_cents: 0, prior_allowed_loss_cents: 0 },
    }];
    for (answer, issue) in [
        (
            Some(Section179OtherLimitedDeduction {
                system: Section179IncomeSystem::Alternative,
                amount_cents: -1,
            }),
            Section179PreparedIssue::OtherDeductionWrongSystem,
        ),
        (
            Some(Section179OtherLimitedDeduction { system, amount_cents: -1 }),
            Section179PreparedIssue::NegativeOtherDeduction,
        ),
        (None, Section179PreparedIssue::OtherDeductionUnavailable),
    ] {
        let mut calls = 0;
        let contextual =
            crate::section179::section179_with_native_prepared_income_and_operating_context(
                &draft,
                system,
                &[],
                &sources,
                |view, allocation, _| {
                    calls += 1;
                    assert_eq!(view, &draft);
                    assert!(allocation.is_empty());
                    answer
                },
            );
        assert_eq!(calls, 1);
        assert_eq!(contextual, Err(issue));
    }
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.section179 = 0;
    let mut calls = 0;
    let zero = crate::section179::section179_with_native_prepared_income_and_operating_context(
        &draft,
        system,
        &[],
        &sources,
        |_, _, _| {
            calls += 1;
            None
        },
    )
    .unwrap();
    assert_eq!(calls, 0);
    assert_eq!(zero.income.total_allowed_cents, 0);
}

#[test]
fn unit17_prepared_direct_readers_preserve_selected_supply_costs_and_absent_pool() {
    let mut supplies = farm_supplies(0);
    supplies.advances.clear();
    supplies.retained =
        vec![FarmSupplyCarry { balance: 200_000, amt_balance: 300_000, consumed: true }];
    let mut draft = farm_supply_return(supplies, 10_000, 70_000);
    add(&mut draft, income(cash_business(10_000)));
    let before = draft.clone();
    let farm = draft.income[0].id;
    let business = draft.income[1].id;
    for (system, cost) in [
        (Section179IncomeSystem::Regular, 1_200_000),
        (Section179IncomeSystem::Alternative, 1_300_000),
    ] {
        let sources: Vec<_> = section179_direct_income_sources(&draft, system)
            .unwrap()
            .iter()
            .map(|source| source.prepared_input().unwrap())
            .collect();
        let selected =
            crate::section179::section179_with_native_prepared_income_and_operating_context(
                &draft,
                system,
                &sources,
                &[],
                |_, _, _| panic!("no available election does not call the other-deduction owner"),
            )
            .unwrap();
        assert_eq!(selected.allowed(farm), Ok(0));
        assert_eq!(selected.operating(farm), Ok(cost));
        assert_eq!(selected.expenses(farm), Ok(cost));
        assert_eq!(selected.tentative_profit(farm), Ok(7_000_000 - cost));
        assert_eq!(selected.office(farm), Ok(0));
        assert_eq!(selected.operating(business), Ok(0));
    }
    assert_eq!(draft, before);
    // Supplied allocation admission is unchanged by a missing expense source.
    // Only readers requiring the unresolved farm pool return its actual issue.
    let FederalIncome::Business { data } = &mut draft.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data: farm_data } = &mut data.activity else { unreachable!() };
    let FederalIncome::Business { data: accrual } = accrual_farm(70_000, 0, 0, 0, 10_000) else {
        unreachable!()
    };
    let BusinessActivity::Farming { data: accrual } = accrual.activity else { unreachable!() };
    farm_data.accounting = accrual.accounting;
    let system = Section179IncomeSystem::Regular;
    let sources: Vec<_> = draft
        .income
        .iter()
        .map(|entry| Section179DirectBusinessIncomeSource {
            source: entry.id,
            system,
            operating_net_cents: Some(0),
            section1231_net_cents: Some(0),
        })
        .collect();
    let selected = crate::section179::section179_with_native_prepared_income_and_operating_context(
        &draft,
        system,
        &sources,
        &[],
        |_, _, _| None,
    )
    .unwrap();
    assert_eq!(selected.allowed(farm), Ok(0));
    assert_eq!(
        selected.operating(farm),
        Err(Section179PreparedIssue::SourceBoundaryUnavailable(
            farm,
            "ISSUE_THIS_PREPAID_SUPPLY_WORKSHEET_USES_CASH_METHOD_TIMING_ACCRUAL_SUPPL_B035D15E"
        ))
    );
    assert_eq!(selected.operating(business), Ok(0));
}
