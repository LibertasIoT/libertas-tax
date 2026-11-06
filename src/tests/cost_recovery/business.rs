use super::*;
use crate::tests::business::business::{
    actual_vehicle, asset_sale_return, cents, form_4562_line, nonresidential_building, sold,
};
use crate::tests::support::builders::survey::survey_cash_business_data;

#[test]
fn macrs_class_elections_reject_conflicts_across_businesses() {
    let mut d = business_with_bonus(
        100000,
        macrs_equipment(10000, EquipmentClass::Five, EquipmentMethod::Regular, 20260601),
    );
    let mut second = cash_business(100000);
    let FederalIncome::Business { data } = &mut second else { panic!() };
    data.assets.push(bonus_equipment(10000, 100));
    add(&mut d, income(second));
    unsupported(&d, "Income:");
    let FederalIncome::Business { data } = &mut d.income[2].income else { panic!() };
    data.assets[0].method = EquipmentMethod::StraightLine;
    unsupported(&d, "Income:");
    let FederalIncome::Business { data } = &mut d.income[2].income else { panic!() };
    data.assets[0].method = EquipmentMethod::Regular;
    data.assets[0].history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 0 },
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    supported(&d);
    for entry in &mut d.income[1..] {
        let FederalIncome::Business { data } = &mut entry.income else { panic!() };
        data.assets[0].class = EquipmentClass::Fifteen;
    }
    let FederalIncome::Business { data } = &mut d.income[2].income else { panic!() };
    data.assets[0].method = EquipmentMethod::Declining150;
    supported(&d); // Ordinary 15-year GDS already is 150% declining balance.
}

#[test]
fn startup_initial_deduction_phaseout_and_opening_month_match_irs_example() {
    // Section 1.195-1 example: $54,500, July opening, $500 + $1,800.
    for (costs, initial, amortization) in [
        (400000, 400000, 0),
        (5000000, 500000, 150000),
        (5450000, 50000, 180000),
        (5500000, 0, 183333),
    ] {
        let d = startup_business(costs, 20260731, false);
        let r = supported(&d);
        for (suffix, expected) in [
            ("initial", initial),
            ("amortization", amortization),
            ("remaining_basis", costs - initial - amortization),
        ] {
            assert_eq!(line_amount(&r, &alloc::format!("business_2_startup_{suffix}")), expected);
        }
        assert_eq!(restart(d.clone()).draft, d);
    }
    let december = supported(&startup_business(2300000, 20261231, false));
    assert_eq!(value(&december, "business_2_startup_initial"), 5000);
    assert_eq!(value(&december, "business_2_startup_amortization"), 100);
}

#[test]
fn startup_deduction_flows_through_income_se_and_qbi_without_duplicate_cost() {
    let d = startup_business(5450000, 20260701, false);
    let r = supported(&d);
    let mut ordinary = d.clone();
    let FederalIncome::Business { data } = &mut ordinary.income[1].income else { panic!() };
    data.startup = BusinessStartup::None;
    data.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Equivalent deduction".into() },
        230000,
    ));
    let expected = supported(&ordinary);
    for id in ["agi", "self_employment_tax", "qbi_deduction"] {
        assert_eq!(value(&r, id), value(&expected, id));
    }
    assert_eq!(r.tax, expected.tax);
    let mut loss = startup_business(5450000, 20260701, false);
    let FederalIncome::Business { data } = &mut loss.income[1].income else { panic!() };
    match &mut data.activity {
        BusinessActivity::Ordinary { receipts, .. }
        | BusinessActivity::Gambling { receipts, .. } => *receipts = 0,
        _ => panic!("expected nonfarm test business"),
    };
    assert_eq!(value(&supported(&loss), "agi"), 57700);
}

#[test]
fn startup_unconfirmed_costs_take_no_deduction_and_out_of_range_dates_block() {
    // Answers and unknowns (reversing the earlier refusal): No takes no
    // deduction with its existing notice; the
    // costs stay as capital basis.
    let startup = |r: &FederalResult, suffix: &str| {
        r.lines
            .iter()
            .find(|l| l.id.contains("_startup_") && l.id.ends_with(suffix))
            .unwrap()
            .amount
    };
    let claimed = supported(&startup_business(1000000, 20260601, false));
    assert!(startup(&claimed, "_initial") > 0);
    {
        let answer = false;
        let mut d = startup_business(1000000, 20260601, false);
        let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
        let BusinessStartup::Costs { eligible, .. } = &mut data.startup else { panic!() };
        *eligible = answer;
        let r = supported(&d);
        assert_eq!(startup(&r, "_initial"), 0);
        assert_eq!(startup(&r, "_amortization"), 0);
        assert_eq!(startup(&r, "_remaining_basis"), 1000000);

        assert!(r.tax > claimed.tax);
    }
    unsupported(&startup_business(1000000, 20041022, false), "Income:");
}

#[test]
fn section197_uses_later_acquisition_or_business_month_and_no_startup_deduction() {
    for (acquired, started, deduction) in [
        (20260101, 20260731, 600),
        (20260930, 20200101, 400),
        (20260131, 20260101, 1200),
        (20261231, 20200101, 100),
    ] {
        let d = intangible_business(acquired, started);
        let r = supported(&d);
        assert_eq!(value(&r, "business_2_intangible_0_amortization"), deduction);
        assert_eq!(value(&r, "business_2_intangible_0_remaining_basis"), 18000 - deduction);
        assert!(!r.lines.iter().any(|l| l.id == "business_2_startup_initial"));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn section197_added_costs_before_business_start_share_original_period() {
    let mut d = intangible_business(20260101, 20260701);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    let asset = &mut data.intangibles[0];
    asset.cost = 0;
    asset.additions = vec![
        IntangibleCost { amount: 180000, date: 20260201 },
        IntangibleCost { amount: 180000, date: 20260731 },
    ];
    assert_eq!(asset.deduction(), 12000);
    assert_eq!(asset.remaining(), 348000);
    assert!(d.valid_stored());
    assert_eq!(value(&supported(&d), "business_2_intangible_0_amortization"), 120);
}

#[test]
fn continuing_macrs_subtracts_original_section179_and_bonus_once() {
    for (percent, elected, bonus, prior, deduction, remaining) in [
        (100, 20000, 32000, 61600, 15360, 23040),
        (60, 20000, 16000, 40800, 7680, 11520),
        (100, 100000, 0, 100000, 0, 0),
    ] {
        let mut asset = continuing_equipment(20250601, prior, false);
        asset.listed = ListedProperty::No { business_percent: percent };
        asset.history = EquipmentHistory::Continuing {
            amt: EquipmentAmtHistory::Same,
            prior_depreciation: prior,
            mid_quarter: false,
            initial_section179: elected,
            carryforward: 0,
            schedule: EquipmentSchedule::Tables { initial_bonus: bonus },
        };
        assert!(asset.valid() && asset.qualifies());
        assert_eq!(asset.macrs_amount(false), deduction);
        assert_eq!(asset.opening_basis() - asset.deduction(false), remaining);
        let d = business_with_bonus(100000, asset);
        let r = supported(&d);
        assert_eq!(value(&r, "business_2_asset_0_bonus"), 0);
        assert_eq!(line_amount(&r, "business_2_asset_0_macrs"), deduction);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn continuing_section179_rejects_impossible_history_and_low_business_use() {
    for (bonus, elected) in [(10000, 40001), (50001, 0)] {
        let mut asset = continuing_equipment(20250601, 50000, false);
        asset.history = EquipmentHistory::Continuing {
            amt: EquipmentAmtHistory::Same,
            prior_depreciation: 50000,
            mid_quarter: false,
            initial_section179: elected,
            carryforward: 0,
            schedule: EquipmentSchedule::Tables { initial_bonus: bonus },
        };
        assert!(!asset.valid());
        assert!(!business_with_bonus(100000, asset).valid_stored());
    }
    let mut asset = continuing_equipment(20250601, 20000, false);
    asset.listed = ListedProperty::No { business_percent: 50 };
    asset.history = EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Same,
        prior_depreciation: 20000,
        mid_quarter: false,
        initial_section179: 10000,
        carryforward: 0,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    assert!(asset.valid());
    unsupported(&business_with_bonus(100000, asset), "Income:");
}

#[test]
fn special_depreciation_classes_are_refused_by_line() {
    // Unit A19 through the situations list.
    let baseline = business_with_bonus(100000, bonus_equipment(100000, 100));
    let baseline_result = supported(&baseline);
    assert_situation_refused(
        &baseline,
        RareSituation::SpecialDepreciation,
        &["Form 4562 lines 14b", "19k"],
    );
    assert_eq!(estimate(&baseline), baseline_result);
}

#[test]
fn section179_precedes_bonus_macrs_and_preserves_ubia() {
    let asset = elected_asset(100000, 20000, BonusElection::Apply);
    assert_eq!(asset.bonus_amount(), 8000000);
    assert_eq!(asset.macrs_amount(false), 0);
    assert_eq!(asset.business_basis(), 10000000);
    let d = business_with_bonus(100000, asset);
    let r = supported(&d);
    assert_eq!(value(&r, "section179_deduction"), 20000);
    assert_eq!(value(&r, "section179_carryforward"), 0);
    assert_eq!(value(&r, "business_2_asset_0_remaining_basis"), 0);
    let ordinary = supported(&business_with_bonus(100000, bonus_equipment(100000, 100)));
    for id in ["agi", "self_employment_tax", "qbi_deduction"] {
        assert_eq!(value(&r, id), value(&ordinary, id));
    }
    assert_eq!(restart(d.clone()).draft, d);
    let mut partial = elected_asset(100000, 20000, BonusElection::Apply);
    partial.acquired = 20250119;
    assert_eq!(partial.bonus_amount(), 1600000);
    assert_eq!(partial.macrs_amount(false), 1280000);
}

#[test]
fn section179_employee_wages_and_business_losses_control_carryforward() {
    let d = business_with_bonus(0, elected_asset(100000, 100000, BonusElection::ElectOut));
    let r = supported(&d);
    assert_eq!(value(&r, "section179_business_income_limit"), 60000);
    assert_eq!(value(&r, "section179_deduction"), 60000);
    assert_eq!(value(&r, "section179_carryforward"), 40000);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 0);
    assert_eq!(value(&r, "agi"), 0);
    let mut loss = d.clone();
    let FederalIncome::Business { data } = &mut loss.income[1].income else { panic!() };
    data.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Ordinary costs".into() },
        1000000,
    ));
    let r = supported(&loss);
    assert_eq!(value(&r, "section179_deduction"), 50000);
    assert_eq!(value(&r, "section179_carryforward"), 50000);
    let mut no_wages = d;
    no_wages.income.remove(0);
    let r = supported(&no_wages);
    assert_eq!(value(&r, "section179_deduction"), 0);
    assert_eq!(value(&r, "section179_carryforward"), 100000);
}

#[test]
fn section179_business_income_limit_includes_materially_participated_k1_income() {
    // T2-29b: Reg. 1.179-2(c)(2)(v). A $100,000 own election with no wages
    // and no Schedule C profit has a $0 line 11; a materially participated
    // partnership's box 1 $30,000 raises line 11 to $30,000, so $30,000 is
    // deducted and $70,000 carries. A passive partner's share adds nothing.
    let mut d = business_with_bonus(0, elected_asset(100000, 100000, BonusElection::ElectOut));
    d.income.remove(0);
    let mut partner = ordinary_partner_source(30_000, true);
    partner.qbi = Some(partner_qbi_statement(30_000));
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = supported(&d);
    assert_eq!(value(&r, "section179_business_income_limit"), 30000);
    assert_eq!(value(&r, "section179_deduction"), 30000);
    assert_eq!(value(&r, "section179_carryforward"), 70000);
    let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income else {
        panic!()
    };
    *data = ordinary_partner_source(30_000, false);
    data.qbi = Some(partner_qbi_statement(30_000));
    let r = supported(&d);
    assert_eq!(value(&r, "section179_business_income_limit"), 0);
    assert_eq!(value(&r, "section179_deduction"), 0);
}

#[test]
fn section179_allocations_sum_exactly_and_ineligible_elections_are_explicit() {
    let mut d = business_with_bonus(0, elected_asset(70000, 70000, BonusElection::ElectOut));
    let mut second = cash_business(0);
    let FederalIncome::Business { data } = &mut second else { panic!() };
    data.assets.push(elected_asset(30000, 30000, BonusElection::ElectOut));
    add(&mut d, income(second));
    let calc = crate::section179::calculate(&d);
    assert_eq!(calc.allowed, 6000000);
    assert_eq!(calc.assets.values().sum::<i64>(), calc.allowed);
    assert_eq!(calc.businesses.values().sum::<i64>(), calc.allowed);
    assert_eq!(calc.businesses[&2], 4200000);
    assert_eq!(calc.businesses[&3], 1800000);
    supported(&d);
    let mut invalid = elected_asset(1000, 1001, BonusElection::ElectOut);
    assert!(!invalid.valid());
    invalid = elected_asset(1000, 500, BonusElection::ElectOut);
    invalid.listed = ListedProperty::No { business_percent: 50 };
    assert!(invalid.valid());
    unsupported(&business_with_bonus(10000, invalid), "Income:");
}

#[test]
fn section179_home_office_follows_two_income_limit_order() {
    for (elected, office) in [(500, 500), (1000, 0)] {
        let mut d =
            business_with_bonus(1000, elected_asset(elected, elected, BonusElection::ElectOut));
        d.income.remove(0);
        let source = d.income[0].id;
        // 2026-10-17: the native allocation API requires actual saved asset
        // identities. Keep the independently authored election and amounts.
        let mut page = crate::tests::cross::interview::business(1000 * 100);
        let mut property =
            crate::tests::cross::interview::asset(elected * 100, 20260601, AssetClass::Five);
        property.acquired = Some(20250120);
        property.gds_mut().unwrap().history = AssetHistory::New {
            first_year: FirstYearDeduction::Section179 {
                description: "Synthetic identity source".into(),
                elected: elected * 100,
            },
        };
        page.assets = vec![property];
        d = save_native_asset_identities(
            d,
            source,
            IncomeSource::Record { value: crate::IncomeRecord::Business(page) },
            TaxSection::SelfEmployment,
        );
        add_simplified_office(&mut d, source, 300);
        let r = supported(&d);
        assert_eq!(value(&r, "section179_deduction"), elected);
        let selected = crate::section179_return::prepare_with_home(
            &d,
            Section179IncomeSystem::Regular,
            &crate::home_office::HomeContext::default(),
        )
        .unwrap();
        assert_eq!(selected.allocation.office(source), Ok(office * 100));
        assert_eq!(value(&r, "agi"), 0);
    }
}

#[test]
fn section179_history_rejects_unreconciled_carry_and_retains_exhausted_assets() {
    let mut asset = continuing_equipment(20180601, 60000, false);
    asset.history = EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Same,
        initial_section179: 100000,
        carryforward: 40000,
        prior_depreciation: 60000,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    assert!(asset.valid() && asset.qualifies());
    assert_eq!(asset.macrs_amount(false), 0);
    assert_eq!(
        value(&supported(&business_with_bonus(0, asset.clone())), "section179_deduction"),
        400
    );
    for carry in [40001, 100001] {
        let mut invalid = asset.clone();
        let EquipmentHistory::Continuing { carryforward, .. } = &mut invalid.history else {
            panic!()
        };
        *carryforward = carry;
        assert!(!invalid.valid());
    }
    asset.placed = 20171201;
    unsupported(&business_with_bonus(0, asset), "Income:");
}

#[test]
fn section179_current_elections_precede_oldest_carryforward_regardless_of_list_order() {
    let mut d = business_with_bonus(0, carried_asset(2025, 30000));
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets.push(carried_asset(2018, 30000));
    data.assets.push(elected_asset(20000, 20000, BonusElection::ElectOut));
    let r = supported(&d);
    assert_eq!(value(&r, "section179_deduction"), 60000);
    assert_eq!(value(&r, "business_2_asset_2_section179"), 20000);
    assert_eq!(value(&r, "business_2_asset_1_section179"), 30000);
    assert_eq!(value(&r, "business_2_asset_0_section179"), 10000);
    assert_eq!(value(&r, "business_2_asset_0_section179_carryforward"), 20000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets[2] = elected_asset(70000, 70000, BonusElection::ElectOut);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_2_section179"), 60000);
    assert_eq!(value(&r, "business_2_asset_1_section179"), 0);
    assert_eq!(value(&r, "business_2_asset_0_section179"), 0);
}

#[test]
fn section179_owner_adjustments_use_their_annual_rounded_deduction_owners() {
    // Pub.946's two-income-limit ordering; Form7206 and Pub.560 retain
    // their own filed-dollar projections rather than source-cent deductions.
    for health in [false, true] {
        let mut d = complete(FederalStatus::Single);
        let mut business =
            if health { health_business(50000, 6000, false) } else { sep_business(50000, 5000) };
        let FederalIncome::Business { data } = &mut business else { panic!() };
        if let BusinessHealthInsurance::Plan { months, .. } = &mut data.health_insurance {
            months[0].premiums += 51;
        }
        if let BusinessRetirement::Sep { contribution, .. } = &mut data.retirement {
            *contribution += 51;
        }
        data.assets.push(elected_asset(10000, 10000, BonusElection::ElectOut));
        add(&mut d, income(business));
        d = save_elected_business_asset(d, 2, 10000, 10000);
        for wages in [60000, 0] {
            if wages == 0 {
                d.income.remove(0);
            }
            let r = supported(&d);
            let adjustment = if health { 6001 } else { 5001 };
            assert_eq!(value(&r, "section179_deduction"), 10000);
            assert_eq!(value(&r, "section179_business_income_limit"), wages + 50000 - adjustment);
            assert_eq!(
                value(
                    &r,
                    if health { "business_2_health_insurance" } else { "business_2_sep_deduction" }
                ),
                adjustment
            );
            let reopened = restart(d.clone());
            assert_eq!(reopened.draft, d);
            assert_eq!(value(&supported(&reopened.draft), "section179_deduction"), 10000);
        }
    }
}

#[test]
fn section179_hypothetical_profit_limits_health_before_the_actual_allowance() {
    // Pub.946's hypothetical179 consumes the full50,000 profit, leaving no
    // Form7206 capacity. Computing health at zero179 would incorrectly reduce
    // the actual179 allowance to44,000 and manufacture a circular benefit.
    let mut d = complete(FederalStatus::Single);
    d.income.clear();
    let mut business = health_business(50000, 6000, false);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.assets.push(elected_asset(50000, 50000, BonusElection::ElectOut));
    add(&mut d, income(business));
    d = save_elected_business_asset(d, 2, 50000, 50000);
    let r = supported(&d);
    assert_eq!(value(&r, "section179_deduction"), 50000);
    assert_eq!(value(&r, "section179_business_income_limit"), 50000);
    assert_eq!(value(&r, "business_2_health_insurance"), 0);
}

#[test]
fn section179_zero_regular_availability_supplies_real_se_half_to_positive_amt_carry() {
    let mut d = complete(FederalStatus::Single);
    let mut business = health_business(50000, 6000, false);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    let asset = claimed_regular_asset_with_amt_carry(2025, 10_000, 5_000);
    data.assets.push(asset);
    add(&mut d, income(business));
    d = save_elected_business_asset(d, 2, 10000, 10000);
    let home = crate::home_office::HomeContext::default();
    let claims = crate::related_forms::effective_claimants(&d);
    let regular = crate::section179_return::prepare_with_home_with_claims(
        &d,
        Section179IncomeSystem::Regular,
        &home,
        &claims,
        None,
    )
    .unwrap();
    assert_eq!(regular.allocation.allowed(2), Ok(0));
    assert!(regular.hypothetical_owner_half.is_some_and(|half| half[0] > 0));
    let alternative = crate::section179_return::prepare_with_home_with_claims(
        &d,
        Section179IncomeSystem::Alternative,
        &home,
        &claims,
        regular.hypothetical_owner_half,
    )
    .unwrap();
    assert_eq!(alternative.allocation.allowed(2), Ok(500000));
    supported(&d);
}

#[test]
fn section179_owner_deductions_keep_the_optional_se_dependency_explicit() {
    let mut d = complete(FederalStatus::Single);
    let mut business = health_business(50000, 6000, false);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.assets.push(elected_asset(10000, 10000, BonusElection::ElectOut));
    add(&mut d, income(business));
    d = save_elected_business_asset(d, 2, 10000, 10000);
    d.screening.as_mut().unwrap().optional_se.push(crate::SeElection {
        owner: Owner::Taxpayer,
        farm: true,
        nonfarm: None,
    });
    unsupported(&d, "Section 179");
}

#[test]
fn vehicle_280f_first_year_limit_caps_bonus_on_business_share() {
    // Same car with the 100% bonus applied (acquired after January 19, 2025):
    //   Tentative bonus 45,000.00; first-year limit with bonus (Rev. Proc.
    //   2026-15 Table 1) $20,300 x 75% = $15,225.00, which wins.
    //   MACRS 0 (no basis left for the tentative computation).
    //   Net profit: 100,000 - 15,225 - 3,000 = $81,775.
    let mut car = actual_vehicle(60000, 10_000, 7_500);
    car.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::Apply,
        tables: false,
    };
    let r = supported(&business_with_bonus(100000, car));
    assert_eq!(cents(&r, "business_2_asset_0_bonus"), 1_522_500);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 0);
    assert_eq!(cents(&r, "business_2_asset_0_listed_special_allowance"), 1_522_500);
    assert_eq!(return_line(&r, "8"), 8_177_500);
}

#[test]
fn vehicle_280f_limit_wins_over_macrs_without_bonus() {
    // $80,000 car, 100% business use (9,000 of 9,000 miles), no bonus:
    //   MACRS 80,000 x 40% x 1/2 = $16,000.00 against the $12,300.00 limit;
    //   the limit wins. Operating $4,000 x 100% = $4,000.00.
    //   Net profit: 100,000 - 12,300 - 4,000 = $83,700.
    let r = supported(&business_with_bonus(100000, actual_vehicle(80000, 9_000, 9_000)));
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 1_230_000);
    assert_eq!(cents(&r, "business_2_car_and_truck"), 400_000);
    assert_eq!(return_line(&r, "8"), 8_370_000);
}

#[test]
fn vehicle_280f_second_year_uses_the_placed_in_service_year_table() {
    // $100,000 passenger automobile placed in service June 15, 2025 (Rev. Proc.
    // 2025-16 tables), 75% business (7,500 of 10,000 miles), no bonus, in its
    // second year (2026). Business basis 100,000 x 75% = $75,000.
    //   2026 GDS 5-year table, year 2: 32.00%, so 75,000 x 32% = $24,000.00.
    //   Section 280F second-year limit for 2025 vehicles (Rev. Proc. 2025-16):
    //   $19,600 x 75% = $14,700.00, which wins. The 2026 table's $19,800 x 75% =
    //   $14,850.00 does not apply to a 2025 vehicle.
    //   First-year depreciation already taken: 75,000 x 20% = 15,000 against
    //   $12,200 x 75% = $9,150.00, so $9,150.00 (Prior depreciation).
    let mut car = actual_vehicle(100000, 10_000, 7_500);
    car.acquired = 20250610;
    car.placed = 20250615;
    car.history = EquipmentHistory::Continuing {
        initial_section179: 0,
        carryforward: 0,
        prior_depreciation: 915_000,
        amt: EquipmentAmtHistory::Same,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    let r = supported(&business_with_bonus(100000, car));
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 1_470_000);
}

#[test]
fn vehicle_280f_first_year_limits_for_both_tables_in_2026() {
    // $100,000 car placed in service in 2026 (Rev. Proc. 2026-15), 100% business.
    //   Without bonus: MACRS 100,000 x 40% x 1/2 = $20,000.00 against the Table 2
    //   first-year limit $12,300.00; the limit wins.
    //   With 100% bonus: tentative bonus $100,000.00 against the Table 1
    //   first-year limit $20,300.00; the limit wins.
    let r = supported(&business_with_bonus(100000, actual_vehicle(100000, 10_000, 10_000)));
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 1_230_000);
    let mut car = actual_vehicle(100000, 10_000, 10_000);
    car.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::Apply,
        tables: false,
    };
    let r = supported(&business_with_bonus(100000, car));
    assert_eq!(cents(&r, "business_2_asset_0_bonus"), 2_030_000);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 0);
}

#[test]
fn listed_property_below_50_percent_uses_straight_line() {
    // 40% business use (4,000 of 10,000 miles) fails the more-than-50% test:
    // ADS straight line over 5 years, half-year, no section 179, no bonus,
    // even with the class bonus election applied and a 200% GDS method entered.
    //   Business basis: 60,000 x 40% = $24,000.00.
    //   ADS: 24,000 / 5 x 1/2 = $2,400.00 (under the $12,300 x 40% = $4,920 limit).
    //   Operating: 4,000 x 40% = $1,600.00.
    //   Net profit: 100,000 - 2,400 - 1,600 = $96,000.
    let mut car = actual_vehicle(60000, 10_000, 4_000);
    car.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::Apply,
        tables: false,
    };
    let r = supported(&business_with_bonus(100000, car.clone()));
    assert_eq!(cents(&r, "business_2_asset_0_bonus"), 0);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 240_000);
    assert_eq!(cents(&r, "business_2_car_and_truck"), 160_000);
    assert_eq!(form_4562_line(&r, "business_2_asset_0_listed_depreciation"), "27(h)");
    assert_eq!(return_line(&r, "8"), 9_600_000);
    // IRS table percentages give the same ADS 10% first-year rate.
    car.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::ElectOut,
        tables: true,
    };
    let r = supported(&business_with_bonus(100000, car.clone()));
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 240_000);
    // A section 179 election is refused at 50% or less.
    car.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 100_000 },
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    unsupported(&business_with_bonus(100000, car), "Actual-expense vehicle");
}

#[test]
fn listed_property_at_exactly_50_percent_uses_straight_line() {
    // 5,000 of 10,000 miles is not more than 50%: ADS straight line.
    //   60,000 x 50% = 30,000; 30,000 / 5 x 1/2 = $3,000.00.
    let r = supported(&business_with_bonus(100000, actual_vehicle(60000, 10_000, 5_000)));
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 300_000);
    // 5,001 miles is more than 50%: 200% DB on 60,000 x 5,001/10,000 =
    // 30,006.00; 30,006 x 20% = $6,001.20 (limit 12,300 x 50.01% = $6,151.23).
    let r = supported(&business_with_bonus(100000, actual_vehicle(60000, 10_000, 5_001)));
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 600_120);
    assert_eq!(cents(&r, "business_2_asset_0_listed_business_use"), 5_001);
}

#[test]
fn passenger_automobile_section179_above_first_year_limit_is_refused() {
    // Section 179 counts inside the first-year limit: $12,300 x 75% = $9,225.
    // Electing $9,225.00 is allowed and leaves no room for MACRS:
    //   section 179 9,150 + MACRS 0; net profit 100,000 - 9,225 - 3,000 = $87,775.
    let mut car = actual_vehicle(60000, 10_000, 7_500);
    car.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 922_500 },
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    let r = supported(&business_with_bonus(100000, car.clone()));
    assert_eq!(cents(&r, "business_2_asset_0_section179"), 922_500);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 0);
    assert_eq!(cents(&r, "business_2_asset_0_listed_section179"), 922_500);
    assert_eq!(return_line(&r, "8"), 8_777_500);
    // One cent more is refused.
    car.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 922_501 },
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    unsupported(&business_with_bonus(100000, car), "Section 179");
}

// C-fixture Form 4562 (2026-09-25, tests/business.rs, matrix row 19): a
// single New-history GDS asset with the given class, first-service date and
// elected section 179 (0 for ineligible), same-day acquisition, 100%
// business use, one consistent GDS method (Regular/200%) and published IRS
// tables, matching this module's `nonresidential_building`/`elected_asset`
// asset-builder style.

#[test]
fn depreciation_return_matches_hand_computation() {
    // Single filer born 1980 (person()'s default birth date; turns 46 in
    // 2026, not a senior), one sole proprietorship (material participation,
    // receipts $220,000, one Supplies expense $8,000, cash method, no other
    // income) and three assets placed in service in 2026:
    //   (a) office equipment, 5-year class, cost $12,000, section 179
    //       elected in full, placed January 15 (Q1).
    //   (b) a machine, 7-year class, cost $60,000, no section 179, bonus
    //       depreciation elected out, placed June 1 (Q2).
    //   (c) a $390,000 nonresidential building (39-year, Table A-7a), placed
    //       May 15 (mid-month; excluded from the mid-quarter test).
    //
    // Mid-quarter test (Pub 946 ch. 4, excluding the real property in (c)):
    // total 2026 basis after elected section 179 = a $12,000 - $12,000
    // elected = $0, plus (b) $60,000 = $60,000 total. Neither (a) nor (b) is
    // placed in the fourth quarter (Q1 and Q2), so the fourth-quarter share
    // is $0 of $60,000 (0%), not more than 40%: half-year convention applies
    // to (b) (and (a), moot at $0 remaining basis).
    //
    // Form 4562 Part I section 179 (2026 dollar limit $2,560,000, phaseout
    // threshold $4,090,000; Pub 946):
    //   Line 1 maximum: $2,560,000. Line 2 total eligible 2026 cost:
    //   $12,000 (only asset (a) is section 179 property). Line 3 threshold
    //   $4,090,000; line 4 (excess over threshold): $0; line 5 dollar limit
    //   after reduction: $2,560,000 (unreduced). Line 6 elected cost (asset
    //   (a)): $12,000. Line 8/11 business income limit (Pub 946 Worksheet
    //   1): with no bonus/MACRS remaining on (a) and no wages, income before
    //   section 179 equals net Schedule C receipts before section 179:
    //   $220,000 receipts - $8,000 supplies - (b) $8,574.00 MACRS -
    //   (c) $6,259.50 MACRS = $197,166.50. Line 11 (smallest of line 5, the
    //   $12,000 elected cost, and the $197,166.50 income limit): $12,000, so
    //   line 12 (elected cost allowed): $12,000, fully allowed (line 13
    //   carryforward: $0).
    //
    // Form 4562 Part II line 14 (special/bonus allowance): $0 -- (b) elects
    // bonus out and (a)'s remaining basis is already $0; (c) is never bonus
    // eligible.
    //
    // Form 4562 Part III: line 17 (assets placed in service before 2026):
    // $0, none of the three assets is a continuing asset. Line 19c (7-year
    // property, half-year, Table A-1 200% declining balance Year 1 =
    // 14.29%): (b) $60,000 x 14.29% = $8,574.00. Line 19j (nonresidential
    // real property, Table A-7a, 39-year straight line, mid-month; May's
    // rate 1.605% from the existing C19b fixture): (c) $390,000 x 1.605% =
    // $6,259.50.
    //
    // Form 4562 Part IV line 22 (total depreciation, to Schedule C line 13):
    // $12,000 (a's line 12 section 179) + $0 (line 14 bonus) + $8,574.00 +
    // $6,259.50 (line 19c/19j MACRS) = $26,833.50.
    //
    // Schedule C: line 13 depreciation and section 179 expense deduction
    // $26,833.50; line 22 supplies $8,000.00; line 28 total expenses
    // $34,833.50; line 31 net profit: $220,000.00 - $34,833.50 = $185,166.50.
    //
    // Schedule SE (nonfarm, one business, no W-2 wages, no church income):
    //   Line 2/3 (net profit, rounded to whole dollars): $185,167. Line
    //   4a-4c: 185,167 x 92.35% -- 185,167 x 9,235 = 1,710,017,245; rounded
    //   to the nearest dollar (add 5,000, drop the last four digits):
    //   $171,002 (line 6, no church income). Line 7: $184,500 (2026 wage
    //   base); line 9: $184,500 (no W-2 wages). Line 10 (12.4% of the
    //   smaller of line 6 and line 9, $171,002): 171,002 x 12.4 = 2,120,424.8
    //   cents, truncated by the App's integer arithmetic to $21,204.24
    //   (2,120,424 cents). Line 11 (2.9% of line 6, never wage-base capped):
    //   171,002 x 2.9 = 4,959,058.0 cents, truncated to $49,590.58
    //   (495,905 cents -- one cent below the exact value from the same
    //   truncating integer division). Combined cents: 2,120,424 + 495,905 =
    //   2,616,329; (2,616,329 + 50) / 100 = 2,616,379 / 100 = $26,163 (line
    //   12 self-employment tax, Schedule 2 line 4). Line 13 half-SE
    //   deduction: (26,163 + 1) / 2 = 26,164 / 2 = $13,082 (Schedule 1 line
    //   15).
    //
    // Form 8995 (2026 single QBI threshold $201,750; taxable income before
    // QBI is far below it, so the phase-in percentage is zero and no
    // SSTB/wage-limit reduction applies): Line 1i business QBI = net profit
    // less its attributable half-SE share (the only business, so the full
    // half-SE amount): 185,167 - 13,082 = $172,085. Lines 2/4: $172,085 (no
    // losses to net). Line 5: 172,085 x 20% = $34,417.00 exactly. Line 6-9
    // REIT/PTP: $0 (none entered). Line 10: $34,417. Line 11 taxable income
    // before QBI: AGI $172,085 - standard deduction $16,100 = $155,985.
    // Line 12 (qualified dividends + net capital gain): $0. Line 13:
    // $155,985. Line 14: 155,985 x 20% = $31,197.00 exactly. Line 15
    // (smaller of line 10 and line 14): $31,197 -- the taxable-income cap
    // binds. Line 16 ($400 active-business minimum, aggregate active QBI
    // >= $1,000): $400, below line 15. Line 17: larger of line 15 and line
    // 16 = $31,197 QBI deduction.
    //
    // Additional Medicare tax (Form 8959, single $200,000 threshold):
    // Schedule SE line 6 ($171,002) is below $200,000, so none applies. NIIT
    // (Form 8960, $200,000 MAGI threshold): no investment income, so none
    // applies either.
    //
    // Form 1040:
    //   Line 8 additional income (Schedule 1 line 3, the business profit
    //   only, rounded): $185,167. Line 9 total income (no other income
    //   source): $185,167. Line 10 adjustments (half-SE deduction only):
    //   $13,082. Line 11 AGI: 185,167 - 13,082 = $172,085. Line 12e standard
    //   deduction (single, born 1980, not 65 in 2026): $16,100. Line 13b QBI
    //   deduction: $31,197. Line 15 taxable income: 172,085 - 16,100 -
    //   31,197 = $124,788.
    //   Line 16 regular tax: $124,788 is at or above the $100,000 Tax Table
    //   ceiling, so the exact 2026 single rate schedule applies directly (no
    //   $50-wide-bracket midpoint): 12,400 x 10% + (50,400-12,400) x 12% +
    //   (105,700-50,400) x 22% + (124,788-105,700) x 24% = 1,240.00 +
    //   4,560.00 + 12,166.00 + 4,581.12 = $22,547.12, which rounds to
    //   $22,547.
    //   Line 23 additional taxes (self-employment tax only; no Additional
    //   Medicare or NIIT): $26,163. Line 24a total tax: 22,547 + 26,163 =
    //   $48,710.
    //   Line 25a withholding: $0 (no W-2). Line 33 total payments: $0 (no
    //   withholding, no estimated payments, no refundable credits).
    //   Line 37 balance due: 48,710 - 0 = $48,710.
    let mut d = empty_return(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Business {
            data: FederalBusiness {
                interest_limit_election: None,
                activity: BusinessActivity::Ordinary {
                    returns: 0,
                    receipts: 22_000_000,
                    accounting: BusinessAccounting::Cash,
                },
                assets: vec![
                    depreciable_asset(12_000, EquipmentClass::Five, 20260115, 12_000),
                    depreciable_asset(60_000, EquipmentClass::Seven, 20260601, 0),
                    nonresidential_building(390_000, 20260515),
                ],
                expenses: vec![ordinary_business_expense(
                    BusinessExpenseCategory::Supplies,
                    800_000,
                )],
                ..survey_cash_business_data()
            },
        }),
    );
    assert!(!crate::depreciation::mid_quarter(&d));
    let r = supported(&d);

    // Part I section 179.
    assert_eq!(value(&r, "section179_dollar_limit"), 2_560_000);
    assert_eq!(value(&r, "section179_eligible_cost"), 12_000);
    // Kept at exact cents: the $197,166.50 business-income limit is not
    // whole dollars, unlike the whole-dollar lines above.
    assert_eq!(cents(&r, "section179_business_income_limit"), 19_716_650);
    assert_eq!(value(&r, "section179_deduction"), 12_000);
    assert_eq!(value(&r, "section179_carryforward"), 0);
    assert_eq!(cents(&r, "business_2_asset_0_section179"), 1_200_000);

    // Part II bonus (line 14) and Part III MACRS (lines 19c/19j).
    assert_eq!(cents(&r, "business_2_asset_0_bonus"), 0);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 0);
    assert_eq!(cents(&r, "business_2_asset_1_bonus"), 0);
    assert_eq!(cents(&r, "business_2_asset_1_macrs"), 857_400);
    assert_eq!(cents(&r, "business_2_asset_2_bonus"), 0);
    assert_eq!(cents(&r, "business_2_asset_2_macrs"), 625_950);
    assert_eq!(form_4562_line(&r, "business_2_asset_2_macrs"), "19j");

    // Part IV line 22 total depreciation (to Schedule C line 13):
    // $12,000 + $0 + $8,574.00 + $6,259.50 = $26,833.50.
    assert_eq!(
        cents(&r, "business_2_asset_0_section179")
            + cents(&r, "business_2_asset_0_bonus")
            + cents(&r, "business_2_asset_0_macrs")
            + cents(&r, "business_2_asset_1_bonus")
            + cents(&r, "business_2_asset_1_macrs")
            + cents(&r, "business_2_asset_2_bonus")
            + cents(&r, "business_2_asset_2_macrs"),
        2_683_350
    );

    // Schedule SE, QBI and AGI.
    assert_eq!(value(&r, "self_employment_tax"), 26_163);
    assert_eq!(value(&r, "adjustments"), 13_082);
    assert_eq!(value(&r, "agi"), 172_085);
    assert_eq!(value(&r, "qbi_deduction"), 31_197);
    assert_eq!(value(&r, "taxable_income"), 124_788);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);

    // Form 1040.
    assert_eq!(return_line(&r, "8"), 18_516_700);
    assert_eq!(return_line(&r, "9"), 18_516_700);
    assert_eq!(return_line(&r, "10"), 1_308_200);
    assert_eq!(return_line(&r, "11a / 11b"), 17_208_500);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "13b"), 3_119_700);
    assert_eq!(return_line(&r, "15"), 12_478_800);
    assert_eq!(return_line(&r, "16"), 2_254_700);
    assert_eq!(return_line(&r, "23"), 2_616_300);
    assert_eq!(return_line(&r, "24a"), 4_871_000);
    assert_eq!(return_line(&r, "37"), 4_871_000);
    assert_eq!(r.tax, Some(4_871_000));
    assert_eq!(r.payments, Some(0));
    assert_eq!(r.balance, Some(4_871_000));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn section179_business_income_limit_carries_the_excess_forward() {
    // Boundary (form-4562.md; Pub 946 ch. 2 section 179 business-income
    // limit): a $12,000 election limited by $5,000 of business income
    // carries the $7,000 excess forward as a separate asset-specific
    // carryforward, never as bonus or MACRS basis.
    //
    // Single office-equipment asset, 5-year class, cost $12,000, elected in
    // full; one sole proprietorship, receipts $5,000, no other expenses, no
    // wages. The asset's remaining basis after the full election is $0, so
    // bonus/MACRS are $0 and the section 179 business-income limit (Pub 946
    // Worksheet 1) equals receipts, $5,000. The $2,560,000 2026 dollar limit
    // and the $12,000 eligible cost both exceed the $5,000 income limit, so
    // the smallest of the three -- income -- binds: allowed $5,000,
    // carryforward $12,000 - $5,000 = $7,000. Schedule C line 31 net
    // profit: $5,000 receipts - $5,000 allowed section 179 = $0.
    let mut d = empty_return(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Business {
            data: FederalBusiness {
                interest_limit_election: None,
                activity: BusinessActivity::Ordinary {
                    returns: 0,
                    receipts: 500_000,
                    accounting: BusinessAccounting::Cash,
                },
                assets: vec![depreciable_asset(12_000, EquipmentClass::Five, 20260115, 12_000)],
                expenses: vec![],
                ..survey_cash_business_data()
            },
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "section179_business_income_limit"), 5_000);
    assert_eq!(value(&r, "section179_deduction"), 5_000);
    assert_eq!(value(&r, "section179_carryforward"), 7_000);
    assert_eq!(cents(&r, "business_2_asset_0_section179"), 500_000);
    assert_eq!(cents(&r, "business_2_asset_0_section179_carryforward"), 700_000);
    assert_eq!(value(&r, "agi"), 0);
}

#[test]
fn special_depreciation_classes_stay_refused() {
    // Unit A19's global refusals (SpecialDepreciation: Form 4562 lines 14b,
    // 14c, 14d, 19g, 19h, 19k; AssetDisposed: Form 4797 Parts III-IV
    // recapture) must stay refused by name after unit C19b added
    // EquipmentClass::ThirtyNine (nonresidential real property, line 19j)
    // to the same asset model.
    let baseline = business_with_bonus(100000, nonresidential_building(390_000, 20260515));
    let baseline_result = supported(&baseline);
    assert_situation_refused(
        &baseline,
        RareSituation::SpecialDepreciation,
        &["Form 4562 lines 14b", "19k"],
    );
    assert_situation_refused(&baseline, RareSituation::AssetDisposed, &["Form 4797"]);
    assert_eq!(estimate(&baseline), baseline_result);
}

#[test]
fn disposal_year_depreciation_follows_the_convention() {
    // Half-year: placed March 1, 2022, 2026 is table year 5 (11.52%), half
    // of it $576. Placed March 1, 2021, 2026 is the last table year (5.76%),
    // already a half year and taken whole: $576.
    for (placed, prior) in [(20220301, 827_200), (20210301, 942_400)] {
        let asset = continuing_asset(
            1_000_000,
            placed,
            EquipmentClass::Five,
            prior,
            sold(20260301, 100_000, 0),
        );
        let r = supported(&asset_sale_return(vec![asset]));
        assert_eq!(value(&r, "business_2_asset_0_macrs"), 576, "{placed}");
    }
    // Mid-quarter, placed in the fourth quarter of 2024 (Table A-5: 5%, 38%,
    // 22.8%): 2026's $2,280 times 4.5/12 for a second-quarter sale = $855.
    let mut asset = continuing_asset(
        1_000_000,
        20241101,
        EquipmentClass::Five,
        430_000,
        sold(20260510, 100_000, 0),
    );
    let EquipmentHistory::Continuing { mid_quarter, .. } = &mut asset.history else { panic!() };
    *mid_quarter = true;
    let r = supported(&asset_sale_return(vec![asset]));
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 855);
}

#[test]
fn f7_section_179_property_still_in_use_at_40_percent_recaptures_in_part_iv_column_a() {
    // 7-year machine placed 2023-06-01, cost $14,000, all expensed under section 179,
    // used 100% through 2025 and 40% in 2026. Line 35(a) $14,000; line 36(a) the
    // depreciation that would have been allowable 2023-2026 (14.29% + 24.49% + 17.49%
    // + 12.49% = 68.76%) = $9,626.40; line 37(a) $4,373.60 is other income on Schedule C
    // line 6 and raises the basis. The 2026 year's depreciation is inside line 36(a).
    let mut machine = continuing_asset(
        1_400_000,
        20230601,
        EquipmentClass::Seven,
        1_400_000,
        crate::AssetDisposal::No,
    );
    if let EquipmentHistory::Continuing { initial_section179, .. } = &mut machine.history {
        *initial_section179 = 1_400_000;
    }
    machine.listed = ListedProperty::No { business_percent: 40 };
    machine.earlier_business_percent = Some(100);
    machine.history = match machine.history {
        EquipmentHistory::Continuing {
            initial_section179,
            carryforward,
            prior_depreciation,
            amt,
            mid_quarter,
            ..
        } => EquipmentHistory::Continuing {
            initial_section179,
            carryforward,
            prior_depreciation,
            amt,
            mid_quarter,
            schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
        },
        other => other,
    };
    assert!(machine.valid());
    let r = supported(&asset_sale_return(vec![machine]));
    assert_eq!(cents(&r, "business_2_asset_0_section179_prior"), 1_400_000);
    assert_eq!(cents(&r, "business_2_asset_0_section179_recomputed"), 962_640);
    assert_eq!(cents(&r, "business_2_asset_0_section179_recapture"), 437_360);
    for (id, line) in [
        ("section179_prior", "35(a)"),
        ("section179_recomputed", "36(a)"),
        ("section179_recapture", "37(a)"),
    ] {
        assert_eq!(
            printed_line(&r, &alloc::format!("business_2_asset_0_{id}")),
            (TaxForm::Form4797, line.into())
        );
    }
    assert_eq!(cents(&r, "business_2_net_profit"), 15_437_360);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 0);
}

#[test]
fn manual_section179_conversion_reuses_the_final_year_schedule() {
    // Pub. 946 ch. 4: HY final year is whole; Q4 placement leaves a
    // 10.5-month final year, of which a Q1 conversion uses 1.5 months.
    for (placed, mid_quarter, recomputed) in
        [(20210301, false, 1_000_051), (20211101, true, 917_967)]
    {
        let mut property = continuing_asset(
            1_000_051,
            placed,
            EquipmentClass::Five,
            1_000_051,
            crate::AssetDisposal::Disposed {
                date: 20260301,
                how: crate::DisposalKind::PersonalUse,
            },
        );
        property.history = EquipmentHistory::Continuing {
            initial_section179: 1_000_051,
            carryforward: 0,
            prior_depreciation: 1_000_051,
            amt: EquipmentAmtHistory::Same,
            mid_quarter,
            schedule: EquipmentSchedule::Manual,
        };
        assert!(property.valid());
        let result = supported(&asset_sale_return(vec![property]));
        assert_eq!(line_amount(&result, "business_2_asset_0_section179_recomputed"), recomputed);
        assert_eq!(
            line_amount(&result, "business_2_asset_0_section179_recapture"),
            1_000_051 - recomputed
        );
    }
}

#[test]
fn suv_179_limit_leaves_remaining_basis_for_bonus_in_both_systems() {
    // IRC 179(b)(5); Rev. Proc. 2025-32 section 4.24.
    let suv = suv_asset(32_000, SuvException::None);
    assert!(suv.valid() && suv.vehicle_supported());
    assert_eq!(suv.automobile_limit(), None);
    let draft = business_with_bonus(100_000, suv.clone());
    let r = supported(&draft);
    assert_eq!(cents(&r, "business_2_asset_0_section179"), 3_200_000);
    assert_eq!(cents(&r, "business_2_asset_0_bonus"), 3_800_000);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 0);
    assert_eq!(cents(&r, "section179_eligible_cost"), 7_000_000);
    assert_eq!(form_4562_line(&r, "business_2_asset_0_listed_section179"), "26(i)");
    crate::tests::support::assert::assert_displayed_text_line(
        &r,
        "business_2_asset_0_listed_type",
        "Heavy SUV (listed property)",
    );
    // Native selected-system APIs consume accepted durable asset identities.
    // Use the canonical Save helper while preserving the independently
    // authored native source and the figures above.
    let mut page = crate::tests::cross::interview::business(10_000_000);
    (page.vehicles, page.assets) =
        crate::interview::vehicle_and_asset_items(&[], core::slice::from_ref(&suv)).unwrap();
    let draft = save_native_asset_identities(
        draft,
        2,
        IncomeSource::Record { value: crate::IncomeRecord::Business(page) },
        TaxSection::SelfEmployment,
    );
    let regular =
        section179_direct_income_sources(&draft, Section179IncomeSystem::Regular).unwrap();
    let alternative =
        section179_direct_income_sources(&draft, Section179IncomeSystem::Alternative).unwrap();
    assert_eq!(regular[0].operating_net_cents, alternative[0].operating_net_cents);
    assert_eq!(suv.deduction(false), suv.deduction_for(false, true));
    assert!(!suv_asset(70_000, SuvException::None).valid());
}

#[test]
fn suv_each_statutory_design_exception_removes_only_the_suv_election_limit() {
    for exception in [
        SuvException::TenRearPassengers,
        SuvException::SixFootCargo,
        SuvException::IntegralLoadEnclosure,
    ] {
        let suv = suv_asset(70_000, exception);
        assert!(suv.valid() && suv.vehicle_supported());
        assert_eq!(suv.automobile_limit(), None);
        let r = supported(&business_with_bonus(100_000, suv));
        assert_eq!(cents(&r, "business_2_asset_0_section179"), 7_000_000);
        assert_eq!(cents(&r, "business_2_asset_0_bonus"), 0);
        assert_eq!(cents(&r, "business_2_asset_0_macrs"), 0);
    }
}

#[test]
fn suv_qualified_use_uses_the_exact_more_than_half_boundary() {
    let mut suv = suv_asset(0, SuvException::None);
    if let ListedProperty::Vehicle { total_miles, business_miles, .. } = &mut suv.listed {
        *total_miles = 1_000_000;
        *business_miles = 500_000;
    }
    suv.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::Apply,
        tables: true,
    };
    assert!(!suv.predominant());
    let r = supported(&business_with_bonus(100_000, suv.clone()));
    assert_eq!(cents(&r, "business_2_asset_0_bonus"), 0);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 350_000);
    if let ListedProperty::Vehicle { business_miles, .. } = &mut suv.listed {
        *business_miles = 500_001;
    }
    assert!(suv.predominant());
    // 70,000 x 500001/1000000 = 35,000.07, eligible for bonus.
    let r = supported(&business_with_bonus(100_000, suv));
    assert_eq!(cents(&r, "business_2_asset_0_bonus"), 3_500_007);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 0);
}

#[test]
fn suv_income_disallowed_election_remains_carry_not_bonus_basis() {
    let mut d = empty_return(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Business {
            data: FederalBusiness {
                interest_limit_election: None,
                activity: BusinessActivity::Ordinary {
                    returns: 0,
                    receipts: 500_000,
                    accounting: BusinessAccounting::Cash,
                },
                assets: vec![suv_asset(32_000, SuvException::None)],
                ..survey_cash_business_data()
            },
        }),
    );
    // The entire elected cost is excluded from bonus basis; only the
    // $38,000 residual receives bonus even though income permits no election.
    let r = supported(&d);
    assert_eq!(cents(&r, "business_2_asset_0_bonus"), 3_800_000);
    assert_eq!(cents(&r, "business_2_asset_0_section179"), 0);
    assert_eq!(cents(&r, "business_2_asset_0_section179_carryforward"), 3_200_000);
}

#[test]
fn qualified_nonpersonal_vehicle_uses_ordinary_recovery_recapture_and_form_rows() {
    // Pub946, Other Property Used for Transportation / Excepted vehicles:
    // physical vehicle mileage remains exact, but section280F does not apply.
    let mut vehicle = suv_asset(0, SuvException::None);
    if let ListedProperty::Vehicle { business_miles, kind, .. } = &mut vehicle.listed {
        *business_miles = 4_000;
        *kind = VehicleKind::QualifiedNonpersonal;
    }
    vehicle.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::Apply,
        tables: true,
    };
    assert!(!vehicle.ads());
    let r = supported(&business_with_bonus(100_000, vehicle.clone()));
    assert_eq!(cents(&r, "business_2_asset_0_bonus"), 2_800_000);
    assert_eq!(form_4562_line(&r, "business_2_asset_0_bonus"), "14a");
    assert!(!r.lines.iter().any(|row| row.id == "business_2_asset_0_listed_business_use"));
    // The ordinary exception does not remove section179's separate >50% test.
    vehicle.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 100 },
        bonus: BonusElection::Apply,
        tables: true,
    };
    assert!(!vehicle.qualifies());
    assert!(!vehicle.vehicle_supported());

    // $10,000 original cost,60% original use: $6,000 business basis and
    // first-year20%=$1,200. Current40% use receives $6,000*.32*(40/60)
    // =$1,280. It has no listed-property recapture or mandatory listed ADS.
    vehicle.cost = 1_000_000;
    vehicle.acquired = 20250310;
    vehicle.placed = 20250315;
    vehicle.earlier_business_percent = Some(60);
    vehicle.history = EquipmentHistory::Continuing {
        initial_section179: 0,
        carryforward: 0,
        prior_depreciation: 120_000,
        amt: EquipmentAmtHistory::Same,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    assert!(vehicle.valid() && vehicle.qualifies() && vehicle.vehicle_supported());
    assert!(!vehicle.ads());
    assert!(vehicle.recapture().is_none());
    let r = supported(&business_with_bonus(100_000, vehicle.clone()));
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 128_000);
    assert!(!r.lines.iter().any(|row| row.id == "business_2_asset_0_listed_recapture"));
    assert!(!crate::depreciation::assets_need_form_4562(&[vehicle.clone()]));

    // An actual prior179 deduction still uses the established ordinary
    // recapture owner (column(a)), never the listed-property column(b).
    // Both systems' source records establish no unused179 carry.
    vehicle.history = EquipmentHistory::Continuing {
        initial_section179: 100_000,
        carryforward: 0,
        prior_depreciation: 200_000,
        amt: EquipmentAmtHistory::Same,
        mid_quarter: false,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    vehicle.amt_section179_income_carry = Some(0);
    assert!(vehicle.recapture().is_none());
    assert!(vehicle.section179_recapture().is_some());
    let r = supported(&business_with_bonus(100_000, vehicle));
    let row =
        r.lines.iter().find(|row| row.id == "business_2_asset_0_section179_recapture").unwrap();
    let form = row.form_line.as_ref().unwrap();
    assert_eq!(form.form, TaxForm::Form4797);
    assert_eq!(form.line, "37(a)");
    assert!(!r.lines.iter().any(|row| row.id == "business_2_asset_0_listed_recapture"));
}

#[test]
fn section179_care_uses_zero_allowance_income_and_actual_return_uses_selected_income() {
    // Reg.1.179-2(c)(1): no179 profit1000, SE141 and half71. Earned1929
    // excludes1929 of care5000, leaving3071 compensation for line11=5071.
    // Actual179=5000 instead leaves a business loss and all5000 care taxable.
    let d = section179_care_return();
    let r = supported(&d);
    assert_eq!(value(&r, "section179_business_income_limit"), 5_071);
    assert_eq!(value(&r, "section179_deduction"), 5_000);
    assert_eq!(value(&r, "care_taxable_taxpayer"), 5_000);
    let claims = crate::related_forms::effective_claimants(&d);
    let home = crate::home_office::HomeContext::default();
    let regular = crate::section179_return::prepare_with_home_and_care_with_claims(
        &d,
        Section179IncomeSystem::Regular,
        &home,
        &claims,
        None,
        None,
    )
    .unwrap();
    assert_eq!(regular.care_compensation.unwrap().taxable_dollars, [3_071, 0]);
    assert_eq!(regular.allocation.allowed(2), Ok(500_000));
    assert!(
        crate::section179_return::prepare_with_home_and_care_with_claims(
            &d,
            Section179IncomeSystem::Alternative,
            &home,
            &claims,
            None,
            None,
        )
        .is_err()
    );
    let alternative = crate::section179_return::prepare_with_home_and_care_with_claims(
        &d,
        Section179IncomeSystem::Alternative,
        &home,
        &claims,
        regular.hypothetical_owner_half,
        regular.care_compensation,
    )
    .unwrap();
    assert_eq!(alternative.care_compensation, regular.care_compensation);
    assert_eq!(alternative.allocation.allowed(2), Ok(500_000));
}

#[test]
fn section179_care_native_save_overlap_reopen_and_restart_keep_source_election() {
    use crate::tests::cross::interview::{accepted, from_review};
    let mut session = restart(section179_care_return());
    let Reply::Edit(mut request) = from_review(&mut session, TaxSection::Care) else {
        panic!("Care editor")
    };
    let P::SaveCare { value: page, .. } = &mut request else { unreachable!() };
    page.benefits[0].in_box_1 = Some(100_000);
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    let r = supported(&saved);
    assert_eq!(value(&r, "care_taxable_taxpayer"), 4_000);
    assert_eq!(value(&r, "section179_business_income_limit"), 4_071);
    assert_eq!(value(&r, "section179_deduction"), 4_071);
    let mut cold = restart(saved.clone());
    let Reply::Edit(reopened) = from_review(&mut cold, TaxSection::Care) else {
        panic!("Care reopen")
    };
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
    assert_eq!(value(&supported(&cold.draft), "section179_deduction"), 4_071);
    let FederalIncome::Business { data } = &cold.draft.income[1].income else { unreachable!() };
    assert!(matches!(
        data.assets[0].history,
        EquipmentHistory::New { section179: Section179::Eligible { elected: 500_000 }, .. }
    ));
}

#[test]
fn section179_care_keeps_native_deferrals_tips_and_unearned_sources() {
    let mut d = section179_care_return();
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    let mut data = edit_wage(data);
    data.other_box12.push(W2CodeAmount { code: W2Code::D, amount: 100_000 });
    data.employee_tips = Some(crate::EmployeeTips {
        cash: crate::CashTipRecords::Allocated { amount: 10_000 },
        noncash: 0,
        coverage: crate::TipCoverage::SocialAndMedicare,
    });
    drop(data);
    add(
        &mut d,
        income(FederalIncome::Interest { data: interest_source(native_interest(100_000, 0)) }),
    );
    let r = supported(&d);
    // Tips add100 earned and reduce taxable care100, preserving line11.
    assert_eq!(value(&r, "section179_business_income_limit"), 5_071);
    assert_eq!(value(&r, "section179_deduction"), 5_000);
}

#[test]
fn section179_care_missing_earned_origin_is_not_zero_and_correction_restores_calculation() {
    let mut d = section179_care_return();
    add(
        &mut d,
        income(FederalIncome::OtherIncome {
            data: OtherIncomeItem {
                kind: OtherIncomeKind::HouseholdWages,
                amount: 100_000,
                withholding: 0,
            },
        }),
    );
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    // Review says the return is refused, not that more answers are needed.
    assert_eq!(
        crate::Message::decode(&crate::interview::estimate_text(&r)).key,
        "ESTIMATE_OUTSIDE_COVERAGE"
    );
    assert!(
        r.issues.iter().any(|issue| issue.display.key == "SECTION179_CARE_EARNED_SOURCE_REQUIRED"
            || issue.display.key == "ISSUE_SECTION179_SOURCE_UNAVAILABLE")
    );
    // Remove the unsupported source through its actual native topic editor.
    let mut session = restart(d);
    let id = session.draft.income.last().unwrap().id;
    let request = crate::tests::support::client::income_pages::save_topic(
        &mut session,
        TaxSection::OtherIncome,
        vec![],
    );
    crate::tests::cross::interview::accepted(&send(&mut session, request));
    assert!(session.draft.income.iter().all(|entry| entry.id != id));
    let corrected = restart(session.draft);
    assert_eq!(value(&supported(&corrected.draft), "section179_business_income_limit"), 5_071);
}

#[test]
fn section179_care_successful_zero_compensation_still_certifies_the_income_limit() {
    let mut d = section179_care_return();
    care_w2(&mut d, 1, 100_000, 0, 0, 750_000);
    let r = supported(&d);
    assert_eq!(value(&r, "section179_business_income_limit"), 2_000);
    assert_eq!(value(&r, "section179_deduction"), 2_000);
    assert_eq!(value(&r, "care_taxable_taxpayer"), 1_000);
    let selected = crate::section179_return::prepare(&d, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(selected.care_compensation.unwrap().taxable_dollars, [0; 2]);
}

#[test]
fn section179_care_zero_regular_availability_preserves_regular_packet_for_amt() {
    let mut d = section179_care_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let asset = claimed_regular_asset_with_amt_carry(2025, 10_000, 5_000);
    data.assets = vec![asset];
    d = save_elected_business_asset(d, 2, 10_000, 10_000);
    let home = crate::home_office::HomeContext::default();
    let claims = crate::related_forms::effective_claimants(&d);
    let regular = crate::section179_return::prepare_with_home_and_care_with_claims(
        &d,
        Section179IncomeSystem::Regular,
        &home,
        &claims,
        None,
        None,
    )
    .unwrap();
    assert_eq!(regular.allocation.allowed(2), Ok(0));
    let care = regular.care_compensation.expect("regular zero allowance still computes care");
    assert!(care.taxable_dollars[0] > 0);
    let alternative = crate::section179_return::prepare_with_home_and_care_with_claims(
        &d,
        Section179IncomeSystem::Alternative,
        &home,
        &claims,
        regular.hypothetical_owner_half,
        Some(care),
    )
    .unwrap();
    assert_eq!(alternative.care_compensation, Some(care));
    assert!(alternative.allocation.allowed(2).unwrap() > 0);
    supported(&d);
}

#[test]
fn section179_care_includes_form8919_employee_pay() {
    // A 1099-NEC answered as Form 8919 employee pay is line 1g wages and
    // IRC129(e)(2) earned income: earned1929+1200 leaves care taxable1871.
    let mut d = section179_care_return();
    let payroll =
        crate::EmployeePayrollSource::Form8919 { reason: crate::EmployeeWageReason::SameFirmForms };
    add(
        &mut d,
        income(crate::tests::income::information_returns::nec_form(
            1_200,
            0,
            Some(crate::IncomeUse::EmployeePay { payroll }),
        )),
    );
    let r = supported(&d);
    assert!(!r.issues.iter().any(|i| i.display.key == "SECTION179_CARE_EARNED_SOURCE_REQUIRED"));
    // The estimate figures the NEC as its Form 8919 record (unknowns view).
    let mut pay = section179_care_return();
    add(&mut pay, income(omitted_employee_wages(1_200)));
    let regular = crate::section179_return::prepare(&pay, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(regular.care_compensation.unwrap().taxable_dollars, [1_871, 0]);
    assert_eq!(value(&supported(&pay), "section179_business_income_limit"), 5_071);
    assert_eq!(value(&r, "section179_business_income_limit"), 5_071);
    assert_eq!(value(&r, "section179_deduction"), 5_000);
    // Employee pay without a Form 8919 reason keeps the designed boundary.
    let mut other = section179_care_return();
    let FederalIncome::EmployeeWages { mut data } = omitted_employee_wages(1_200) else {
        unreachable!()
    };
    data.payroll = crate::EmployeePayrollSource::TotalizationCertificate;
    add(&mut other, income(FederalIncome::EmployeeWages { data }));
    assert!(
        estimate(&other)
            .issues
            .iter()
            .any(|i| i.display.key == "SECTION179_CARE_EARNED_SOURCE_REQUIRED")
    );
}

#[test]
fn section179_care_includes_native_employee_compensation_and_retains_correction() {
    use crate::tests::cross::interview::{accepted, from_review};
    let d = section179_care_employee_reimbursements(1_000);
    // Before section179, profit1000 less half-SE71 plus W2 wages1000 and
    // Form2106 excess1000 gives earned2929. Care5000 less excluded2929 adds
    //2071 compensation; the aggregate section179 income remains5071.
    let source = crate::employee_expenses::before179_compensation_sources(&d, None).unwrap();
    assert_eq!(source.total, 100_000);
    assert_eq!(source.by_owner, [100_000, 0]);
    let r = supported(&d);
    assert_eq!(value(&r, "section179_business_income_limit"), 5_071);
    assert_eq!(value(&r, "section179_deduction"), 5_000);
    assert_eq!(value(&r, "employee_expenses_1_excess_reimbursement"), 1_000);
    assert_eq!(value(&r, "care_taxable_taxpayer"), 5_000);
    let regular = crate::section179_return::prepare(&d, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(regular.care_compensation.unwrap().taxable_dollars, [2_071, 0]);
    let mut missing = d.clone();
    missing.adjustments.as_mut().unwrap().employee_expenses[0].employers = vec![i64::MAX];
    assert!(crate::employee_expenses::before179_compensation_sources(&missing, None).is_err());
    let unresolved = estimate(&missing);
    assert_eq!(unresolved.state, ResultState::Unsupported);
    assert!(
        unresolved
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_EMPLOYEE_EXPENSES_EMPLOYER_MISSING")
    );
    // Correct the actual reimbursement source through its native editor.
    let mut session = restart(d);
    let Reply::Edit(mut request) = from_review(&mut session, TaxSection::EmployeeExpenses) else {
        panic!("Employee expenses editor")
    };
    let P::SaveEmployeeExpenses { value: page, .. } = &mut request else { unreachable!() };
    page.forms[0].reimbursements = 100_000;
    accepted(&send(&mut session, request));
    let source =
        crate::employee_expenses::before179_compensation_sources(&session.draft, None).unwrap();
    assert_eq!(source.total, 0);
    assert_eq!(source.by_owner, [0; 2]);
    let regular =
        crate::section179_return::prepare(&session.draft, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(regular.care_compensation.unwrap().taxable_dollars, [3_071, 0]);
    let saved = session.draft.clone();
    let mut corrected = restart(saved.clone());
    let Reply::Edit(reopened) = from_review(&mut corrected, TaxSection::EmployeeExpenses) else {
        panic!("Employee expenses reopen")
    };
    let P::SaveEmployeeExpenses { value: page, .. } = &reopened else { unreachable!() };
    assert_eq!(page.forms[0].reimbursements, 100_000);
    accepted(&send(&mut corrected, reopened));
    assert_eq!(corrected.draft, saved);
    let r = supported(&corrected.draft);
    assert_eq!(value(&r, "section179_business_income_limit"), 5_071);
    assert_eq!(value(&r, "section179_deduction"), 5_000);
    assert_eq!(value(&r, "care_taxable_taxpayer"), 5_000);
}

#[test]
fn section179_owner_deductions_include_native_employee_compensation_once() {
    for health in [false, true] {
        let mut d = section179_care_employee_reimbursements(10_000);
        clear_care_benefits(&mut d);
        let benefit =
            if health { health_business(1_000, 100, false) } else { sep_business(1_000, 100) };
        let FederalIncome::Business { data: benefit } = benefit else { unreachable!() };
        let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
        data.health_insurance = benefit.health_insurance;
        data.retirement = benefit.retirement;
        let r = supported(&d);
        // W2 wages1000 + business10000 + employee excess1000 - owner deduction100.
        assert_eq!(value(&r, "section179_business_income_limit"), 11_900);
        assert_eq!(value(&r, "section179_deduction"), 5_000);
        assert_eq!(value(&r, "wages"), 2_000);
        assert_eq!(value(&r, "self_employment_tax"), 707);
        d.adjustments.as_mut().unwrap().employee_expenses[0].reimbursements = 100_000;

        // Actual business profit $10,000 less the $5,000 election leaves positive
        // earnings for the unchanged $100 SEP deposit or health premium.
        let corrected = supported(&d);
        assert_eq!(value(&corrected, "section179_business_income_limit"), 10_900);
        assert_eq!(value(&corrected, "section179_deduction"), 5_000);
        assert_eq!(value(&corrected, "wages"), 1_000);
        assert_eq!(value(&corrected, "self_employment_tax"), 707);
    }
}

#[test]
fn section179_employee_compensation_packet_preserves_the_spouses_native_source() {
    let (d, _) = non_code_l_wage_fixture(Owner::Spouse);
    // The spouse received $2,000 against native Form2106 expenses of $1,900.
    let source = crate::employee_expenses::before179_compensation_sources(&d, None).unwrap();
    assert_eq!(source.total, 10_000);
    assert_eq!(source.by_owner, [0, 10_000]);
}

// G-57: W-2 box 10 benefits with a limited partner's box 12 and an S
// corporation's box 11 section 179 deduction (both materially participating:
// a passive K-1 deduction keeps its own Form 8582 refusal). Neither reduces
// Form 2441 earned income, and the vested-stock sale adds none, so nothing refuses.
#[test]
fn section179_care_limited_partner_scorp_and_stock_sale_compute() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
    let mut partner = ordinary_partner_source(2_000, true);
    partner.qbi = Some(partner_qbi_statement(2_000));
    partner.section179 = 10_000;
    partner.amt_section179_income_carry =
        Some(crate::Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: Vec::new() });
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let mut scorp = complete_ordinary_scorp(1_000, true);
    scorp.section179 = 8_000;
    scorp.amt_section179_income_carry =
        Some(crate::Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: Vec::new() });
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(scorp) }));
    let lot = EmployeeStockSale {
        acquired: 20260101,
        sold: 20260701,
        paid: 0,
        market_value: 300_000,
        fees: 0,
        proceeds: 310_000,
        wash_sale: 0,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        special: false,
    };
    add(&mut d, income(FederalIncome::EmployeeStockSale { data: lot }));
    add_child(&mut d);
    care_costs(&mut d, 500_000);
    care_w2(&mut d, 1, 250_000, 0, 0, 750_000);
    let r = supported(&d);
    assert_eq!(value(&r, "section179_deduction"), 180);
    assert_eq!(value(&r, "care_taxable_taxpayer"), 0);
    let regular = crate::section179_return::prepare(&d, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(regular.care_compensation.unwrap().taxable_dollars, [0; 2]);
}

// A general partner's box 12 deduction reduces box 14 code A. Line 11 uses
// the before-179 earned income (Reg.1.179-2(c)(1)); Form 2441 uses the
// earned income the deduction reduced.
#[test]
fn section179_care_general_partner_computes_reduced_earned_income() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 100_000, 0);
    let mut partner = ordinary_partner_source(5_000, true);
    partner.qbi = Some(partner_qbi_statement(5_000));
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(500_000);
    partner.section179 = 300_000;
    partner.amt_section179_income_carry =
        Some(crate::Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: Vec::new() });
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    add_child(&mut d);
    care_costs(&mut d, 500_000);
    care_w2(&mut d, 1, 500_000, 0, 0, 750_000);
    let r = supported(&d);
    assert_eq!(value(&r, "section179_deduction"), 3_000);
    let regular = crate::section179_return::prepare(&d, Section179IncomeSystem::Regular).unwrap();
    let before = regular.care_compensation.unwrap().taxable_dollars;
    let earned = value(&r, "care_form2441_line18");
    assert_eq!(value(&r, "care_taxable_taxpayer"), 5_000 - earned, "{before:?} {earned}");
    assert_eq!(before, [0; 2]);
    assert!(earned < 5_000);
}

// ISO pay not in W-2 box 1 is Form 1040 line 1h wages in the line 11 income
// and Form 2441 earned income: $1,000 W-2 + $1,000 profit + $50,000 ISO pay,
// and the earned income now excludes all $5,000 of benefits.
#[test]
fn section179_line11_includes_iso_compensation_not_in_wages() {
    let mut d = section179_care_return();
    add(&mut d, income(FederalIncome::IsoSale { data: disqualifying_iso_sale() }));
    let r = supported(&d);
    assert_eq!(value(&r, "section179_business_income_limit"), 52_000);
    assert_eq!(value(&r, "section179_deduction"), 5_000);
    let regular = crate::section179_return::prepare(&d, Section179IncomeSystem::Regular).unwrap();
    assert_eq!(regular.care_compensation.unwrap().taxable_dollars, [0; 2]);
}

// Smith facts: both K-1s passive. Each box 12 / box 11 deduction allowed on
// Form 4562 line 12 joins its activity's Form 8582 net; with box 1 income
// above it, the pool allows it in full and nothing refuses.
#[test]
fn section179_passive_k1s_with_care_benefits_estimate() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
    let carry =
        Some(crate::Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: Vec::new() });
    let mut partner = ordinary_partner_source(2_000, false);
    partner.qbi = Some(partner_qbi_statement(2_000));
    partner.section179 = 10_000;
    partner.amt_section179_income_carry = carry.clone();
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let mut scorp = complete_ordinary_scorp(1_000, false);
    scorp.section179 = 8_000;
    scorp.amt_section179_income_carry = carry;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(scorp) }));
    add_child(&mut d);
    care_costs(&mut d, 500_000);
    care_w2(&mut d, 1, 250_000, 0, 0, 750_000);
    let r = supported(&d);
    assert_eq!(value(&r, "section179_deduction"), 180);
    assert_eq!(value(&r, "partnership_2_section179"), 100);
    assert_eq!(value(&r, "scorp_3_section179"), 80);
    assert_eq!(value(&r, "partnership_2_section179_passive_allowed"), 100);
    assert_eq!(value(&r, "scorp_3_section179_passive_allowed"), 80);
    assert_eq!(value(&r, "partnership_2_section179_passive_unallowed"), 0);
    // $60,000 wages + ($2,000 - $100) + ($1,000 - $80).
    assert_eq!(return_line(&r, "11a / 11b") / 100, 62_820);
}

// A passive loss activity: box 1 loss $3,000 and box 12 $1,000 with no other
// passive income. Form 8582 allows none; the held $4,000 keeps its section
// 179 share ($1,000 of $4,000 losses) and no deduction reaches AGI.
#[test]
fn section179_passive_loss_activity_holds_its_section179_share() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
    let mut partner = ordinary_partner_source(-3_000, false);
    partner.qbi = Some(partner_qbi_statement(-3_000));
    partner.section179 = 100_000;
    partner.amt_section179_income_carry =
        Some(crate::Section179IncomeCarryHistory { gross: 0, sold: 0, vintages: Vec::new() });
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = supported(&d);
    assert_eq!(value(&r, "partnership_2_section179_passive_unallowed"), 1_000);
    assert_eq!(value(&r, "partnership_2_section179_passive_allowed"), 0);
    // The ordinary unallowed loss is a separate carryover entry.
    assert_eq!(value(&r, "partnership_2_passive_suspended"), 3_000);
    assert_eq!(return_line(&r, "11a / 11b") / 100, 60_000);
}
