use super::support::assert::unit50_fuel::assert_fuel_payment;
use super::*;

#[test]
fn beauty_tip_credit_requires_eligible_services_and_reconciled_payroll() {
    for service in [
        EmployerTipService::BeautyServices {
            barbering_and_hair_care: false,
            nail_care: true,
            esthetics: false,
            body_and_spa_treatment: false,
            eligible_tips_only: true,
        },
        EmployerTipService::BeautyServices {
            barbering_and_hair_care: false,
            nail_care: false,
            esthetics: true,
            body_and_spa_treatment: false,
            eligible_tips_only: true,
        },
        EmployerTipService::BeautyServices {
            barbering_and_hair_care: false,
            nail_care: false,
            esthetics: false,
            body_and_spa_treatment: true,
            eligible_tips_only: true,
        },
    ] {
        let mut draft = beauty_tip_credit_draft();
        restaurant_business_mut(&mut draft).employer_tip_fica_credit.as_mut().unwrap().service =
            service;
        assert_eq!(value(&supported(&draft), "business_2_form8846_line4"), 8);
    }

    let mut draft = beauty_tip_credit_draft();
    restaurant_business_mut(&mut draft).employer_tip_fica_credit.as_mut().unwrap().service =
        EmployerTipService::Other;
    unsupported(&draft, "Current employer tip FICA credit");

    let mut draft = beauty_tip_credit_draft();
    restaurant_business_mut(&mut draft).employer_tip_fica_credit.as_mut().unwrap().service =
        EmployerTipService::BeautyServices {
            barbering_and_hair_care: false,
            nail_care: false,
            esthetics: false,
            body_and_spa_treatment: false,
            eligible_tips_only: true,
        };
    unsupported(&draft, "Current employer tip FICA credit");

    let mut draft = beauty_tip_credit_draft();
    let business = restaurant_business_mut(&mut draft);
    if let EmployerTipService::BeautyServices { eligible_tips_only, .. } =
        &mut business.employer_tip_fica_credit.as_mut().unwrap().service
    {
        *eligible_tips_only = false;
    }
    assert!(noted(&supported(&draft), "WARN_TIP_FICA_CREDIT_LEFT_OUT"));

    let mut draft = beauty_tip_credit_draft();
    let BusinessPayroll::Yes { statements, .. } = &mut restaurant_business_mut(&mut draft).payroll
    else {
        panic!()
    };
    statements[0].medicare += 100;
    unsupported(&draft, "Current employer tip FICA credit");
}

#[test]
fn restaurant_tip_credit_deduction_uses_reconciled_payroll_rounding() {
    let mut draft = restaurant_tip_credit_draft();
    let rounded_by_pay_period = restaurant_business(&draft).expense_total(false);
    restaurant_business_mut(&mut draft)
        .employer_tip_fica_credit
        .as_mut()
        .unwrap()
        .actual_employer_tip_fica_paid = 3_443;
    let rounded_once = restaurant_business(&draft).expense_total(false);
    assert_eq!(rounded_by_pay_period, rounded_once + 1);
    assert_eq!(
        supported(&draft)
            .lines
            .iter()
            .find(|line| { line.id == "business_2_form8846_remaining_employer_fica_deduction" })
            .unwrap()
            .amount,
        1_043
    );
}

#[test]
fn restaurant_tip_credit_uses_2025_credit_first_and_records_2025_carryback() {
    let mut draft = restaurant_tip_credit_draft();
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(tip_fica_carryforward(10_000_000));
    restaurant_business_mut(&mut draft)
        .employer_tip_fica_credit
        .as_mut()
        .unwrap()
        .used_prior_year_carryback = 500;
    let result = supported(&draft);
    let limit = value(&result, "general_business_credit_limit");
    assert!(limit > 0 && limit < 100_000);
    assert_eq!(value(&result, "general_business_credit"), limit);
    assert_eq!(value(&result, "tip_fica_credit_carryforward_to_next_year"), 100_000 - limit);
    assert_eq!(value(&result, "tip_fica_current_credit_used"), 0);
    assert_eq!(value(&result, "tip_fica_current_credit_carryback_to_prior_year"), 5);
    assert_eq!(value(&result, "tip_fica_current_credit_carryforward_to_next_year"), 19);
    assert_eq!(supported(&restart(draft).draft), result);
}

#[test]
fn restaurant_tip_credit_offsets_amt_but_preserves_uncredited_payroll_deduction() {
    let mut draft = restaurant_tip_credit_draft();
    draft.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic ISO", 2000.0, 1000, 11000)];
    let result = supported(&draft);
    assert!(value(&result, "alternative_minimum_tax") > 0);
    assert_eq!(value(&result, "business_2_form8846_remaining_employer_fica_deduction"), 10);
    assert_eq!(value(&result, "tip_fica_current_credit_used"), 24);
    assert_eq!(value(&result, "general_business_credit"), 24);
}

#[test]
fn restaurant_tip_credit_reduces_fica_deduction_even_when_tax_limit_uses_none() {
    let mut draft = restaurant_tip_credit_draft();
    draft.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
    data.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 100_000,
        accounting: BusinessAccounting::Cash,
    };
    let result = supported(&draft);
    assert_eq!(value(&result, "business_2_form8846_line4"), 24);
    assert_eq!(value(&result, "business_2_form8846_remaining_employer_fica_deduction"), 10);
    assert_eq!(value(&result, "general_business_credit_limit"), 0);
    assert_eq!(value(&result, "tip_fica_current_credit_used"), 0);
    assert_eq!(value(&result, "tip_fica_current_credit_carryforward_to_next_year"), 24);
    assert!(value(&result, "self_employment_tax") > 0);
}

#[test]
fn restaurant_form3800_credit_precedes_form8912_bond_credit() {
    let mut draft = restaurant_tip_credit_draft();
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![direct_tax_credit_bond(
        "RESTAURANT8912",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        10_000_000,
    )];
    let result = supported(&draft);
    assert_eq!(value(&result, "general_business_credit"), 24);
    assert_eq!(
        value(&result, "form_8912_line11"),
        value(&result, "income_tax") - value(&result, "general_business_credit")
    );
    assert_eq!(value(&result, "tax_credit_bond_credit"), value(&result, "form_8912_line11"));
    assert!(value(&result, "tax_credit_bond_carry_2027_1") > 0);
    assert_eq!(supported(&restart(draft).draft), result);
}

#[test]
fn restaurant_tip_credit_coexists_with_both_form8978_directions() {
    let mut draft = restaurant_tip_credit_draft();
    let baseline = supported(&draft);
    draft.credits.as_mut().unwrap().bba_audit_interest = Some(bba_interest_increase());
    let increase = supported(&draft);
    assert_eq!(value(&increase, "form_8978_line_14"), 220);
    assert_eq!(return_line(&increase, "16"), return_line(&baseline, "16") + 22_000);
    assert_eq!(value(&increase, "tip_fica_current_credit_used"), 24);
    assert_eq!(value(&increase, "business_2_form8846_remaining_employer_fica_deduction"), 10);
    assert_eq!(supported(&restart(draft.clone()).draft), increase);

    draft.credits.as_mut().unwrap().bba_audit_interest = Some(bba_interest_reduction());
    let decrease = supported(&draft);
    // Taxable117,962: rate tax20,909. Reserving220 leaves20,689 of
    // specified-credit capacity; the current24 credit fits unchanged.
    // Add ordinary SE tax14,075:20,909−220−24+14,075=34,740.
    assert_eq!(value(&decrease, "regular_income_tax"), 20909);
    assert_eq!(value(&decrease, "schedule_3_line_6l"), 220);
    assert_eq!(value(&decrease, "general_business_credit_limit"), 20689);
    assert_eq!(value(&decrease, "tip_fica_current_credit_used"), 24);
    assert_eq!(value(&decrease, "self_employment_tax"), 14075);
    assert_eq!(decrease.tax, Some(3_474_000));
    assert_eq!(supported(&restart(draft.clone()).draft), decrease);
}

#[test]
fn restaurant_tip_credit_rejects_unreconciled_source_and_excess_carryback() {
    let mut draft = restaurant_tip_credit_draft();
    let business = restaurant_business_mut(&mut draft);
    let BusinessPayroll::Yes { statements, .. } = &mut business.payroll else { panic!() };
    statements[0].medicare += 100;
    unsupported(&draft, "Current employer tip FICA credit");

    let mut draft = restaurant_tip_credit_draft();
    restaurant_business_mut(&mut draft)
        .employer_tip_fica_credit
        .as_mut()
        .unwrap()
        .actual_employer_tip_fica_paid = 2_300;
    unsupported(&draft, "Current employer tip FICA credit");

    let mut draft = restaurant_tip_credit_draft();
    let business = restaurant_business_mut(&mut draft);
    let claim = business.employer_tip_fica_credit.as_mut().unwrap();
    claim.employees[0].months[0] = EmployerTipMonth {
        hours_hundredths: 10_000,
        wages_excluding_tips: 0,
        reported_tips: 18_450_001,
    };
    let BusinessPayroll::Yes { statements, .. } = &mut business.payroll else { panic!() };
    statements[0].medicare += 18_450_001;
    unsupported(&draft, "Current employer tip FICA credit");

    let mut draft = restaurant_tip_credit_draft();
    restaurant_business_mut(&mut draft)
        .employer_tip_fica_credit
        .as_mut()
        .unwrap()
        .used_prior_year_carryback = 2_500;
    unsupported(&draft, "General business credit carryback");
    let carryback_issue = estimate(&draft)
        .issues
        .into_iter()
        .find(|issue| issue.section == "General business credit carryback")
        .unwrap();
    let display = carryback_issue.display;
    assert_eq!(display.key, "ISSUE_CREDIT_CARRYBACK_LIMIT");
    assert!(matches!(
        display.args.as_slice(),
        [MessageArg::Resource(form_key), MessageArg::Currency {
            currency_code,
            scale: 0,
            minor_units: _,
        }] if currency_code == "USD" && form_key == "BUSINESS_CREDIT_FORM_EMPLOYER_TIP_FICA"
    ));

    let mut draft = restaurant_tip_credit_draft();
    restaurant_business_mut(&mut draft).employer_tip_fica_credit.as_mut().unwrap().employees[0]
        .months
        .pop();
    assert!(!draft.valid_stored());
    let mut draft = restaurant_tip_credit_draft();
    draft.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Separate);
    draft.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::Together };
    // T3-3800: married filing separately is calculated with the $12,500
    // line 13 amount (section 38(c)(6)(A)), 25% of line 12 above it.
    let r = supported(&draft);
    let line12 = value(&r, "form_3800_line_12");
    assert!(line12 > 12_500);
    assert_eq!(
        value(&r, "form_3800_line_13"),
        crate::kernel::ratio::fraction(line12 - 12_500, 25, 100)
    );
    assert_eq!(value(&r, "tip_fica_current_credit_used"), 24);
}

#[test]
fn actual_2027_traditional_deposit_for_2026_preserves_deduction_and_saver_credit() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_500_000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(200_000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let original_year = supported(&d);
    let fact = &mut d.adjustments.as_mut().unwrap().iras.taxpayer;
    fact.next_year_deposits = Some(next_year_ira_deposits(200_000, 0, 20270415));
    let result = supported(&d);
    assert_eq!(value(&result, "ira_deduction"), 2000);
    assert_eq!(value(&result, "agi"), 23000);
    assert_eq!(value(&result, "taxpayer_2026_form_5498_box1_2027_deposit"), 2000);
    assert_eq!(value(&result, "saver_credit"), value(&original_year, "saver_credit"));
    assert_eq!(result.tax, original_year.tax);
    assert!(result.review_notes.iter().any(|n| n.contains("2027 Saver's Match")));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn extended_models_round_trip_and_invalid_credit_records_block_restore() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Rental { data: rental(30000, 5000) }));
    add(&mut d, income(foreign_pay(100000)));
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.qualification = ForeignQualification::Presence { starts_on: 20260101, foreign_days: 330 };
    set_claim(&mut d, claim);
    let data = native_draft_records(&d);
    let decoded = draft_from_native_records(d.tax_year, &data);
    assert_eq!(decoded, d);
    assert_eq!(native_draft_records(&decoded), data);
    // The payee-statement tax is a document amount: a negative one is invalid.
    payee_statement_credit(&mut d, -1);
    assert!(!d.valid_stored());
}

#[test]
fn low_income_retiree_gets_result_and_exempt_benefits_reduce_elderly_credit() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19620101;
    d.income[0] = wage(1, Owner::Taxpayer, 1_000_000, 0);
    assert_eq!(supported(&d).tax, Some(0));
    let v = d.deductions.as_mut().unwrap();
    v.choice = DeductionSelection::Itemized { facts: itemized(v).clone() };
    let r = supported(&d);
    assert_eq!(value(&r, "elderly_credit"), 403);
    assert_eq!(r.tax, Some(0));
    d.credits.as_mut().unwrap().exempt_pensions =
        vec![ExemptPension { label: "Synthetic veterans pension".into(), amount: 500_000 }];
    let r = supported(&d);
    assert_eq!(value(&r, "elderly_credit"), 0);
    assert_eq!(r.tax, Some(40_300));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn qualified_local_bus_uses_full_alternative_fuel_rates_and_conversions() {
    let physical = FederalFuelLines {
        bus_uses: Some(FederalBusFuelUses {
            lpg_local_bus: Some(alternative_fuel_use(
                FederalAlternativeFuelQuantity::LiquidGallons { thousandths: 135_300 },
            )),
            cng_local_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::CubicFeet {
                thousandths: 12_357_000,
            })),
            lng_local_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::Pounds {
                thousandths: 606_000,
            })),
            local_bus_qualified: Some(true),
            ..FederalBusFuelUses::default()
        }),
        ..FederalFuelLines::default()
    };
    assert!(physical.valid() && physical.qualified());
    for (line, expected) in [("11a", 1_830), ("11c", 1_830), ("11g", 2_430)] {
        assert_eq!(physical.labeled_lines().find(|(label, _)| *label == line).unwrap().1, expected);
    }
    let local = FederalBusFuelUses {
        lpg_local_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::Gge {
            thousandths: 100_000,
        })),
        p_series_local_bus: Some(fuel_use(100_000, 30_000)),
        cng_local_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::Gge {
            thousandths: 100_000,
        })),
        hydrogen_local_bus: Some(fuel_use(100_000, 30_000)),
        fischer_tropsch_local_bus: Some(fuel_use(100_000, 30_000)),
        biomass_liquid_local_bus: Some(fuel_use(100_000, 30_000)),
        lng_local_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::Dge {
            thousandths: 100_000,
        })),
        biomass_gas_local_bus: Some(fuel_use(100_000, 30_000)),
        local_bus_qualified: Some(true),
        ..FederalBusFuelUses::default()
    };
    let direct = FederalFuelLines {
        bus_uses: Some(FederalBusFuelUses {
            lpg_public_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::Gge {
                thousandths: 100_000,
            })),
            p_series_public_bus: Some(fuel_use(100_000, 30_000)),
            ..local.clone()
        }),
        ..FederalFuelLines::default()
    };
    let partner_fuels = FederalFuelLines { bus_uses: Some(local), ..FederalFuelLines::default() };
    assert!(direct.valid() && direct.qualified());
    assert!(partner_fuels.valid() && partner_fuels.qualified());
    let business_fuels = direct;
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 35_100);
    for (source, expected) in [
        ("business_2", [2_920, 2_930, 1_830, 1_830, 2_430, 2_430, 2_430, 1_830]),
        ("partnership_3", [1_830, 1_830, 1_830, 1_830, 2_430, 2_430, 2_430, 1_830]),
    ] {
        for (line, amount) in
            ["11a", "11b", "11c", "11d", "11e", "11f", "11g", "11h"].into_iter().zip(expected)
        {
            assert_eq!(
                line_amount(&report, &alloc::format!("{source}_fuel_{line}")),
                amount,
                "{source} line {line}"
            );
        }
    }

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let local = data.fuel_tax_credit.as_mut().unwrap().fuels.bus_uses.as_mut().unwrap();
    local.local_bus_qualified = Some(false);
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let local = data.fuel_tax_credit.as_mut().unwrap().fuels.bus_uses.as_mut().unwrap();
    local.local_bus_qualified = Some(true);
    local.p_series_local_bus.as_mut().unwrap().actual_cost = 1;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let local = data.fuel_tax_credit.as_mut().unwrap().fuels.bus_uses.as_mut().unwrap();
    local.p_series_local_bus.as_mut().unwrap().actual_cost = 30_000;
    local.lpg_local_bus.as_mut().unwrap().not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let local = data.fuel_tax_credit.as_mut().unwrap().fuels.bus_uses.as_mut().unwrap();
    local.lpg_local_bus.as_mut().unwrap().not_claimed_elsewhere = true;
    local.lng_local_bus.as_mut().unwrap().quantity =
        FederalAlternativeFuelQuantity::Gge { thousandths: 100_000 };
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
}

#[test]
fn alternative_fuel_physical_units_convert_to_gge_and_dge_on_return() {
    let fuels = FederalFuelLines {
        lpg_off_highway: Some(alternative_fuel_use(
            FederalAlternativeFuelQuantity::LiquidGallons { thousandths: 135_300 },
        )),
        cng_off_highway: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::CubicFeet {
            thousandths: 12_357_000,
        })),
        lng_off_highway: Some(alternative_fuel_use(
            FederalAlternativeFuelQuantity::LiquidGallons { thousandths: 171_000 },
        )),
        ..empty_fuel_lines()
    };
    // The independent invoice quantities each equal 100 GGE/DGE. Form 4136
    // lines 11a, 11c and 11g credit $18.30, $18.30 and $24.30.
    assert_eq!(
        fuels.labeled_lines().filter(|(_, v)| *v != 0).collect::<Vec<_>>(),
        vec![("11a", 1830), ("11c", 1830), ("11g", 2430)]
    );
    assert_eq!(fuels.credit_cents(), 6_090);
    let business_fuels = fuels.clone();
    let partner_fuels = fuels;
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 12_200);
    for source in ["business_2", "partnership_3"] {
        for (form_line, expected) in [("11a", 1830), ("11c", 1830), ("11g", 2430)] {
            assert_eq!(
                line_amount(&report, &alloc::format!("{source}_fuel_{form_line}")),
                expected
            );
        }
    }

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let claim = data.fuel_tax_credit.as_mut().unwrap();
    claim.fuels.lpg_off_highway.as_mut().unwrap().quantity =
        FederalAlternativeFuelQuantity::Dge { thousandths: 100_000 };
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let claim = data.fuel_tax_credit.as_mut().unwrap();
    claim.fuels.lpg_off_highway.as_mut().unwrap().quantity =
        FederalAlternativeFuelQuantity::Gge { thousandths: 100_000 };
    assert_eq!(claim.fuels.credit_cents(), 6_090);
    claim.fuels.lpg_off_highway.as_mut().unwrap().quantity =
        FederalAlternativeFuelQuantity::Pounds { thousandths: 575_000 };
    claim.fuels.cng_off_highway.as_mut().unwrap().quantity =
        FederalAlternativeFuelQuantity::Pounds { thousandths: 566_000 };
    claim.fuels.lng_off_highway.as_mut().unwrap().quantity =
        FederalAlternativeFuelQuantity::Pounds { thousandths: 606_000 };
    assert_eq!(claim.fuels.credit_cents(), 6_090);
    claim.fuels.lng_off_highway.as_mut().unwrap().quantity =
        FederalAlternativeFuelQuantity::Dge { thousandths: 100_000 };
    assert_eq!(claim.fuels.credit_cents(), 6_090);
    claim.fuels.cng_off_highway.as_mut().unwrap().qualified = false;
    fuel_use_left_out(&draft);
}

#[test]
fn home_kerosene_requires_qualified_use_and_purchase_records() {
    let mut draft = complete(FederalStatus::Single);
    draft.payments.as_mut().unwrap().home_kerosene = Some(FederalHomeKerosene {
        thousandths_of_gallon: 100_000,
        actual_cost: 20_000,
        qualified: false,
        not_claimed_elsewhere: true,
        records_complete: true,
    });
    // Fuel that does not qualify takes no credit.
    assert_eq!(value(&supported(&draft), "home_kerosene_fuel_4a"), 0);
    let home = draft.payments.as_mut().unwrap().home_kerosene.as_mut().unwrap();
    home.qualified = true;
    home.not_claimed_elsewhere = false;
    assert_eq!(value(&supported(&draft), "home_kerosene_fuel_4a"), 0);
    let home = draft.payments.as_mut().unwrap().home_kerosene.as_mut().unwrap();
    home.not_claimed_elsewhere = true;
    home.records_complete = false;
    let r = supported(&draft);
    assert_eq!(value(&r, "home_kerosene_fuel_4a"), 0);
    assert!(noted(&r, "WARN_HOME_KEROSENE_LEFT_OUT"));
    let home = draft.payments.as_mut().unwrap().home_kerosene.as_mut().unwrap();
    home.records_complete = true;
    home.thousandths_of_gallon = 0;
    assert!(!draft.valid_stored());
    draft.payments.as_mut().unwrap().home_kerosene.as_mut().unwrap().thousandths_of_gallon =
        100_000;
    draft.payments.as_mut().unwrap().home_kerosene.as_mut().unwrap().actual_cost = 1;
    assert!(!draft.valid_stored());
}

#[test]
fn section_1250_fund_statement_preserves_credit_and_validates_components() {
    let mut d = complete(FederalStatus::Single);
    let mut fund = undistributed_gain();
    let FederalIncome::UndistributedGain { data } = &mut fund else { panic!() };
    data.unrecaptured = 200000;
    add(&mut d, income(fund));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 5000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 2000);
    assert_eq!(value(&r, "rate_28_gain"), 3000);
    assert_eq!(value(&r, "regular_income_tax"), 5623);
    assert_eq!(r.payments, Some(755000));
    d.income[1].income = mixed_gain_distribution();
    let FederalIncome::Dividend { data } = &mut d.income[1].income else { panic!() };
    data.unrecaptured_distributions = 700001;
    assert!(!d.valid_stored());
    let FederalIncome::Dividend { data } = &mut d.income[1].income else { panic!() };
    data.unrecaptured_distributions = -1;
    assert!(!d.valid_stored());
}

#[test]
fn final_savings_bond_election_matches_irs_example_and_can_be_declined() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died = Some(FinalReturn {
        date: 20260615,
        ordinary: true,
        bonds: Some(FinalBonds {
            cost: 50000,
            value: 59400,
            reported: 0,
            reporting: FinalBondReporting::Elect,
            eligible: true,
        }),
        estate_medical: 0,
    });
    let r = supported(&d);
    assert_eq!(return_line(&r, "2b"), 9400);
    assert_eq!(value(&r, "agi"), 60094);
    assert_eq!(value(&r, "final_bond_recipient_interest_0"), 0);
    assert_eq!(restart(d.clone()).draft, d);
    d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().bonds.as_mut().unwrap().reporting =
        FinalBondReporting::Defer;
    let r = supported(&d);
    assert_eq!(return_line(&r, "2b"), 0);
    assert_eq!(value(&r, "final_bond_recipient_interest_0"), 94);
}

#[test]
fn mortgage_certificate_credit_reduces_itemized_interest_before_tax() {
    let mut d = complete(FederalStatus::Single);
    let deductions = d.deductions.as_mut().unwrap();
    fixture_mortgage_interest(deductions, 2_000_000);
    d.credits.as_mut().unwrap().mortgage_credit =
        Some(mortgage_certificate_credit(2000, 10000, 1_000_000));
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_credit_current_potential"), 2000);
    assert_eq!(value(&r, "deduction"), 18000);
    assert_eq!(value(&r, "income_tax"), 4795);
    assert_eq!(value(&r, "mortgage_interest_credit"), 2000);
    assert_eq!(r.tax, Some(279_500));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);

    d.credits.as_mut().unwrap().mortgage_credit =
        Some(mortgage_certificate_credit(2500, 6000, 1_000_000));
    let shared = supported(&d);
    assert_eq!(value(&shared, "mortgage_credit_current_potential"), 1200);
    assert_eq!(value(&shared, "deduction"), 18800);
    assert_eq!(value(&shared, "income_tax"), 4699);
    assert_eq!(shared.tax, Some(349_900));

    fixture_mortgage_interest(d.deductions.as_mut().unwrap(), 1_700_000);
    d.credits.as_mut().unwrap().mortgage_credit =
        Some(mortgage_certificate_credit(2000, 10000, 1_000_000));
    let standard = supported(&d);
    assert_eq!(value(&standard, "deduction"), 16100);
    assert_eq!(value(&standard, "mortgage_interest_credit"), 2000);
    assert_eq!(standard.tax, Some(302_300));

    d.credits
        .as_mut()
        .unwrap()
        .mortgage_credit
        .as_mut()
        .unwrap()
        .current
        .as_mut()
        .unwrap()
        .history = MortgageCertificateHistory::Unreviewed;
    unsupported(&d, "Mortgage interest credit");
}

#[test]
fn certificate_reissued_before_2026_is_limited_by_original_scheduled_credit() {
    let mut d = complete(FederalStatus::Single);
    let deductions = d.deductions.as_mut().unwrap();
    fixture_mortgage_interest(deductions, 2_000_000);
    let mut claim = mortgage_certificate_credit(1500, 10000, 1_200_000);
    let certificate = claim.current.as_mut().unwrap();
    certificate.issue_date = 20240115;
    certificate.history = MortgageCertificateHistory::ReissuedBefore2026 {
        facts: MortgageReissueFacts {
            refinanced_on: 20240101,
            original_interest_method: MortgageOriginalInterestMethod::FixedRateSchedule,
            first_reissue: true,
            original_number: "MCC-ORIGINAL".into(),
            original_rate_hundredths_percent: 2000,
            original_scheduled_interest: 800_000,
            original_certified_balance: 20_000_000,
            reissued_certified_debt: 18_000_000,
            confirmed: true,
        },
    };
    d.credits.as_mut().unwrap().mortgage_credit = Some(claim);
    let limited = supported(&d);
    assert_eq!(value(&limited, "mortgage_credit_current_potential"), 1600);
    assert_eq!(value(&limited, "mortgage_original_schedule_cap"), 1600);
    assert_eq!(value(&limited, "mortgage_interest_credit"), 1600);
    assert_eq!(value(&limited, "deduction"), 18400);
    assert_eq!(supported(&restart(d.clone()).draft), limited);

    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history {
        facts.original_scheduled_interest = 1_000_000;
    }
    let new_certificate_controls = supported(&d);
    assert_eq!(value(&new_certificate_controls, "mortgage_credit_current_potential"), 1800);
    assert_eq!(value(&new_certificate_controls, "mortgage_original_schedule_cap"), 2000);

    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history {
        facts.original_interest_method = MortgageOriginalInterestMethod::VariableRateSchedule {
            worksheet_reference: "Synthetic original variable-rate terms and 2026 schedule".into(),
            schedule_confirmed: true,
            method_consistent: true,
        };
    }
    let variable_schedule = supported(&d);
    assert_eq!(value(&variable_schedule, "mortgage_credit_current_potential"), 1800);
    assert_eq!(supported(&restart(d.clone()).draft), variable_schedule);
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history {
        facts.original_interest_method = MortgageOriginalInterestMethod::VariableRateSchedule {
            worksheet_reference: "Synthetic original variable-rate terms and 2026 schedule".into(),
            schedule_confirmed: false,
            method_consistent: true,
        };
    }
    assert!(noted(&supported(&d), "WARN_MORTGAGE_CREDIT_LEFT_OUT"));
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history {
        facts.original_interest_method = MortgageOriginalInterestMethod::VariableRateSchedule {
            worksheet_reference: "Synthetic original variable-rate terms and 2026 schedule".into(),
            schedule_confirmed: true,
            method_consistent: false,
        };
    }
    assert!(noted(&supported(&d), "WARN_MORTGAGE_CREDIT_LEFT_OUT"));
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history {
        facts.original_interest_method = MortgageOriginalInterestMethod::FixedRateSchedule;
    }

    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history {
        facts.confirmed = false;
    }
    assert!(noted(&supported(&d), "WARN_MORTGAGE_CREDIT_LEFT_OUT"));
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history {
        facts.confirmed = true;
        facts.original_interest_method = MortgageOriginalInterestMethod::VariableRateHypothetical {
            worksheet_reference: "Synthetic level-payment hypothetical loan".into(),
            principal_at_refinance: 20_000_000,
            refinance_apr_hundredths_percent: 600,
            refinancing_maturity: 20540101,
            worksheet_confirmed: true,
            method_consistent: true,
        };
    }
    let hypothetical = supported(&d);
    assert_eq!(value(&hypothetical, "mortgage_credit_current_potential"), 1800);
    assert_eq!(supported(&restart(d.clone()).draft), hypothetical);
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history
        && let MortgageOriginalInterestMethod::VariableRateHypothetical {
            method_consistent, ..
        } = &mut facts.original_interest_method
    {
        *method_consistent = false;
    }
    assert!(noted(&supported(&d), "WARN_MORTGAGE_CREDIT_LEFT_OUT"));
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history
        && let MortgageOriginalInterestMethod::VariableRateHypothetical {
            principal_at_refinance,
            ..
        } = &mut facts.original_interest_method
    {
        *principal_at_refinance += 1;
    }
    assert!(!d.valid_stored());
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history {
        facts.original_interest_method = MortgageOriginalInterestMethod::FixedRateSchedule;
        facts.first_reissue = false;
    }
    assert!(noted(&supported(&d), "WARN_MORTGAGE_CREDIT_LEFT_OUT"));
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history {
        facts.confirmed = true;
        facts.first_reissue = true;
        facts.reissued_certified_debt = facts.original_certified_balance + 1;
    }
    assert!(!d.valid_stored());
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history {
        facts.reissued_certified_debt = facts.original_certified_balance;
        facts.original_rate_hundredths_percent = 1400;
    }
    assert!(!d.valid_stored());
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedBefore2026 { facts } = &mut certificate.history {
        facts.original_rate_hundredths_percent = 2000;
    }
    certificate.issue_date = 20260115;
    assert!(!d.valid_stored());
}

#[test]
fn current_mcc_credit_and_same_home_form_8828_recapture_reconcile() {
    let mut d = complete(FederalStatus::Single);
    let mut home = home_sale(false);
    home.price = 40_000_000;
    home.expenses = 2_000_000;
    home.basis = 30_000_000;
    add(&mut d, income(FederalIncome::HomeSale { data: home }));
    d.credits.as_mut().unwrap().mortgage_credit =
        Some(mortgage_certificate_credit(2000, 10000, 1_000_000));
    let credit_only = supported(&d);
    assert_eq!(value(&credit_only, "mortgage_interest_credit"), 2000);
    set_mortgage_subsidy(
        &mut d,
        MortgageSubsidyDisposition::Ordinary {
            subsidy: MortgageSubsidyKind::Mcc,
            transfer: MortgageSubsidyTransfer::Sale {
                home_sale_income_id: 2,
                qsml_gain_share: None,
            },
            facts: ordinary_subsidy_facts(),
        },
    );
    let both = supported(&d);
    assert_eq!(value(&both, "mortgage_interest_credit"), 2000);
    assert_eq!(value(&both, "mortgage_subsidy_recapture_tax"), 3750);
    assert_eq!(value(&both, "total_tax"), value(&credit_only, "total_tax") + 3750);
    assert_eq!(supported(&restart(d.clone()).draft), both);
}

#[test]
fn residential_energy_carryforward_uses_form_5695_and_follows_dc_credit() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().residential_energy_carry = Some(700_000);
    let energy_only = supported(&d);
    assert_eq!(value(&energy_only, "residential_energy_credit"), 5023);
    assert_eq!(value(&energy_only, "residential_energy_carryforward_2027"), 1977);
    assert_eq!(energy_only.tax, Some(0));
    assert!(energy_only.review_notes.iter().any(|note| note.contains("Form 5695")));

    d.credits.as_mut().unwrap().dc_homebuyer_carry = Some(300_000);
    d.credits.as_mut().unwrap().residential_energy_carry = Some(400_000);
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let combined = supported(&d);
    assert_eq!(value(&combined, "dc_homebuyer_credit"), 3000);
    assert_eq!(value(&combined, "residential_energy_credit"), 2023);
    assert_eq!(value(&combined, "residential_energy_carryforward_2027"), 1977);
    assert_eq!(value(&combined, "minimum_tax_credit"), 0);
    assert_eq!(value(&combined, "minimum_tax_credit_carryforward"), 15000);
    assert_eq!(return_line(&combined, "20"), 502_300);
    assert_eq!(supported(&restart(d.clone()).draft).tax, combined.tax);

    let mut child_energy = complete(FederalStatus::Single);
    child_energy.dependents.push(child(2));
    child_energy.next_id = 3;
    child_energy.credits.as_mut().unwrap().residential_energy_carry = Some(400_000);
    let child_result = supported(&child_energy);
    assert_eq!(value(&child_result, "residential_energy_credit"), 4000);
    assert_eq!(return_line(&child_result, "19"), 102_300);
    assert_eq!(return_line(&child_result, "28"), 117_700);

    d.credits.as_mut().unwrap().residential_energy_carry = Some(-100);
    assert!(!d.valid_stored());

    let mut farmland = farmland_installment_return();
    farmland.credits.as_mut().unwrap().residential_energy_carry = Some(100_000);
    unsupported(&farmland, "Farmland tax installments");
}

#[test]
fn residential_energy_carryforward_caps_at_available_tax_by_one_dollar() {
    // Single, wage $60,000, standard deduction $16,100 (Pub 501 Table 8,
    // Rev. Proc. 2025-32): taxable income = 60,000 - 16,100 = 43,900.
    // Below $100,000 the 2026 Tax Table midpoint rounds to the nearest $50
    // interval plus $25: 43,900 / 50 = 878 exactly, midpoint = 878*50+25 = 43,925.
    // 2026 single brackets (irb/2025-45): 10% to $12,400, 12% to $50,400.
    //   12,400 * 10% = $1,240.00
    //   (43,925 - 12,400) * 12% = 31,525 * 12% = $3,783.00
    //   income tax (line 16) = $1,240.00 + $3,783.00 = $5,023.00
    // No foreign tax credit, mortgage/adoption/DC-homebuyer credits, so
    // creditable_tax for the Form 5695 worksheet is the full $5,023.
    // 2025 Form 5695 line 16 carryforward = $5,024, one dollar above the
    // available tax: Form 5695 lines 1-3 (2026 draft, carryforward only)
    // limit the credit to creditable_tax = $5,023, leaving $1 unused.
    // Line 4: unused balance carried to 2027 = 5,024 - 5,023 = $1.
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().residential_energy_carry = Some(502_400);
    let r = supported(&d);
    assert_eq!(value(&r, "residential_energy_credit"), 5023);
    assert_eq!(value(&r, "residential_energy_carryforward_2027"), 1);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn residential_energy_credit_limit_worksheet_reserves_room_for_the_dc_homebuyer_credit() {
    // Single, wage $150,000, standard deduction $16,100: taxable income =
    // 150,000 - 16,100 = 133,900, at or above $100,000 so the rate
    // schedule applies directly (no Tax Table midpoint).
    // 2026 single brackets (irb/2025-45): 10% to 12,400; 12% to 50,400;
    // 22% to 105,700; 24% to 201,775.
    //   12,400 * 10%              = $1,240.00
    //   (50,400 - 12,400) * 12%   = 38,000 * 12% = $4,560.00
    //   (105,700 - 50,400) * 22%  = 55,300 * 22% = $12,166.00
    //   (133,900 - 105,700) * 24% = 28,200 * 24% = $6,768.00
    //   income tax (line 16) = 1,240 + 4,560 + 12,166 + 6,768 = $24,734.00
    // No foreign tax credit, so creditable_tax = $24,734.
    // Form 8859 line 1-3 DC first-time homebuyer carryforward of $20,000 is
    // taken first in the 2026 Form 5695 credit-limit worksheet's ordering
    // (Schedule 3 map row 5a: "after ... mortgage credit, adoption credit
    // and DC-homebuyer credit"): dc_homebuyer_credit = min(20,000, 24,734) =
    // $20,000 (fully used; no carryforward).
    // Form 5695 line 16 residential clean energy carryforward of $6,000
    // is then limited to the tax remaining after the DC-homebuyer credit:
    // 24,734 - 20,000 = $4,734 of room, so:
    //   residential_energy_credit = min(6,000, 4,734) = $4,734
    //   line 4 carryforward to 2027 = 6,000 - 4,734 = $1,266
    // (Not min(6,000, 24,734) = 6,000: the worksheet ordering after the
    // DC-homebuyer credit, not the raw creditable tax, is what limits it.)
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 15_000_000, 650_000);
    d.credits.as_mut().unwrap().dc_homebuyer_carry = Some(2_000_000);
    d.credits.as_mut().unwrap().residential_energy_carry = Some(600_000);
    let r = supported(&d);
    assert_eq!(value(&r, "dc_homebuyer_credit"), 20000);
    assert_eq!(value(&r, "residential_energy_credit"), 4734);
    assert_eq!(value(&r, "residential_energy_carryforward_2027"), 1266);
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn zero_tax_return_carries_the_full_residential_energy_credit_forward() {
    // Single, wage $10,000, standard deduction $16,100: taxable income =
    // positive(10,000 - 16,100) = $0, so income tax (line 16) = $0 and,
    // with no foreign tax credit, creditable_tax = $0.
    // Form 5695 line 16 carryforward of $3,000 has no available tax to
    // absorb it: residential_energy_credit = min(3,000, positive(0)) = $0.
    // Line 4: the entire $3,000 balance carries to 2027 unused.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 1_000_000, 650_000);
    d.credits.as_mut().unwrap().residential_energy_carry = Some(300_000);
    let r = supported(&d);
    assert_eq!(value(&r, "residential_energy_credit"), 0);
    assert_eq!(value(&r, "residential_energy_carryforward_2027"), 3000);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn personal_refueling_credit_caps_each_unit_and_reduces_schedule_three() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().personal_refueling =
        vec![personal_refueling(400_000, 20260115), personal_refueling(400_000, 20260630)];
    let r = supported(&d);
    assert_eq!(value(&r, "personal_refueling_credit"), 2000);
    assert_eq!(value(&r, "personal_refueling_credit_unusable"), 0);
    assert_eq!(return_line(&r, "20"), 200_000);
    assert_eq!(r.tax, Some(302_300));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);

    d.credits.as_mut().unwrap().personal_refueling[1].placed_in_service = 20260701;
    unsupported(&d, "Refueling property credit");
    d.credits.as_mut().unwrap().personal_refueling[1].placed_in_service = 20260630;
    // An ineligible tract takes no credit for that unit; business use is
    // left out with a notice.
    d.credits.as_mut().unwrap().personal_refueling[0].eligible_tract = false;
    assert_eq!(value(&supported(&d), "personal_refueling_credit"), 1000);
    d.credits.as_mut().unwrap().personal_refueling[0].eligible_tract = true;
    d.credits.as_mut().unwrap().personal_refueling[0].personal_only = false;
    let r = supported(&d);
    assert_eq!(value(&r, "personal_refueling_credit"), 1000);
    assert!(noted(&r, "WARN_REFUELING_PROPERTY_LEFT_OUT"));
    d.credits.as_mut().unwrap().personal_refueling[0].personal_only = true;
    d.credits.as_mut().unwrap().personal_refueling[0].census_geoid = "bad".into();
    assert!(!d.valid_stored());
}

#[test]
fn personal_refueling_unused_credit_expires_and_does_not_displace_earlier_credits() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_400_000, 0);
    d.credits.as_mut().unwrap().personal_refueling = vec![personal_refueling(400_000, 20260630)];
    let low_tax = supported(&d);
    assert_eq!(value(&low_tax, "income_tax"), 793);
    assert_eq!(value(&low_tax, "personal_refueling_credit"), 793);
    assert_eq!(value(&low_tax, "personal_refueling_credit_unusable"), 207);
    assert_eq!(low_tax.tax, Some(0));

    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
    d.credits.as_mut().unwrap().dc_homebuyer_carry = Some(400_000);
    let after_homebuyer = supported(&d);
    assert_eq!(value(&after_homebuyer, "dc_homebuyer_credit"), 4000);
    assert_eq!(value(&after_homebuyer, "personal_refueling_credit"), 1000);
    assert_eq!(after_homebuyer.tax, Some(2_300));

    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let after_minimum_credit = supported(&d);
    assert_eq!(value(&after_minimum_credit, "personal_refueling_credit"), 1000);
    assert_eq!(value(&after_minimum_credit, "minimum_tax_credit"), 23);
    assert_eq!(after_minimum_credit.tax, Some(0));
}

#[test]
fn personal_refueling_credit_respects_tentative_minimum_tax_and_defers_form_1062_overlap() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().personal_refueling = vec![personal_refueling(400_000, 20260101)];
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic ISO", 2000.0, 1000, 11000)];
    let r = supported(&d);
    assert_eq!(value(&r, "alternative_minimum_tax"), 39151);
    assert_eq!(value(&r, "personal_refueling_credit"), 0);
    assert_eq!(value(&r, "personal_refueling_credit_unusable"), 1000);

    let mut farmland = farmland_installment_return();
    farmland.credits.as_mut().unwrap().personal_refueling =
        vec![personal_refueling(400_000, 20260630)];
    unsupported(&farmland, "Farmland tax installments");
}

#[test]
fn clean_vehicle_credits_follow_used_then_new_form_8936_limits() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().clean_vehicles =
        Some(clean_vehicle_claim(vec![new_clean_vehicle(), used_clean_vehicle()]));
    let r = supported(&d);
    for id in
        ["form_8936_own_business_part", "form_8936_k1_business_part", "form_8936_business_part"]
    {
        assert!(!r.lines.iter().any(|line| line.id == id), "{id}");
    }
    assert_eq!(value(&r, "income_tax"), 5023);
    assert_eq!(value(&r, "previously_owned_clean_vehicle_credit"), 4000);
    assert_eq!(value(&r, "new_clean_vehicle_credit"), 1023);
    assert_eq!(value(&r, "new_clean_vehicle_credit_unusable"), 6477);
    assert_eq!(value(&r, "previously_owned_clean_vehicle_credit_unusable"), 0);
    assert_eq!(return_line(&r, "20"), 502_300);
    assert_eq!(r.tax, Some(0));
    assert_eq!(restart(d.clone()).draft, d);

    d.income[0] = wage(1, Owner::Taxpayer, 2_400_000, 0);
    let low_tax = supported(&d);
    assert_eq!(value(&low_tax, "income_tax"), 793);
    assert_eq!(value(&low_tax, "previously_owned_clean_vehicle_credit"), 793);
    assert_eq!(value(&low_tax, "new_clean_vehicle_credit"), 0);
    assert_eq!(value(&low_tax, "previously_owned_clean_vehicle_credit_unusable"), 3207);
}

#[test]
fn clean_vehicle_used_price_model_year_and_transfer_are_explicit() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().clean_vehicles =
        Some(clean_vehicle_claim(vec![used_clean_vehicle()]));
    let r = supported(&d);
    assert_eq!(value(&r, "previously_owned_clean_vehicle_credit"), 4000);

    let claim = d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap();
    let FederalCleanVehicle::PreviouslyOwned { model_year, .. } = &mut claim.vehicles[0] else {
        unreachable!()
    };
    *model_year = 2024;
    assert!(!d.valid_stored());
    let claim = d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap();
    let FederalCleanVehicle::PreviouslyOwned { model_year, sales_price, .. } =
        &mut claim.vehicles[0]
    else {
        unreachable!()
    };
    *model_year = 2023;
    *sales_price = 2_600_000;
    unsupported(&d, "Clean vehicle");
    let claim = d.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap();
    let FederalCleanVehicle::PreviouslyOwned { sales_price, details, .. } = &mut claim.vehicles[0]
    else {
        unreachable!()
    };
    *sales_price = 2_000_000;
    details.dealer_transfer = Some(400_000);
    let transferred = supported(&d);
    assert_eq!(value(&transferred, "previously_owned_clean_vehicle_credit"), 0);
    assert_eq!(value(&transferred, "previously_owned_clean_vehicle_transfer_repayment"), 0);
    assert_eq!(return_line(&transferred, "17"), 0);
}

#[test]
fn legacy_ev_passive_credit_uses_form_8834_regular_tax_limit_and_expires() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().legacy_ev_passive_allowed = Some(600_000);
    let result = supported(&d);
    assert_eq!(value(&result, "income_tax"), 5023);
    assert_eq!(value(&result, "legacy_ev_passive_credit"), 5023);
    assert_eq!(value(&result, "legacy_ev_passive_credit_unusable"), 977);
    assert_eq!(return_line(&result, "20"), 502_300);
    assert_eq!(result.tax, Some(0));
    assert_eq!(supported(&restart(d.clone()).draft).tax, result.tax);

    d.credits.as_mut().unwrap().legacy_ev_passive_allowed = Some(-1);
    assert!(!d.valid_stored());
}

#[test]
fn minimum_tax_credit_excludes_permanent_items_and_keeps_negative_generation() {
    for (amt, carry, generated, available) in [
        (30000, 0, 10906, 10906),
        (10000, 12000, -9094, 2906),
        (0, 30000, -19094, 10906),
        (0, 1000, -19094, 0),
    ] {
        let mut d = complete(FederalStatus::Single);
        let mut h = minimum_tax_history();
        h.base = 18000000;
        h.regular = 1000000;
        h.amt = amt * 100;
        h.carry = carry * 100;
        d.credits.as_mut().unwrap().minimum_tax =
            Some(crate::MinimumTaxHistories::from_current_unit(h));
        let r = supported(&d);
        // $200,000 exclusion-only AMTI less $88,100 exemption, at 26%,
        // minus the retained $10,000 regular-tax comparison = $19,094.
        assert_eq!(value(&r, "minimum_tax_exclusion_tax"), 19094);
        assert_eq!(value(&r, "minimum_tax_credit_generated"), generated);
        assert_eq!(value(&r, "minimum_tax_credit_available"), available);
        assert_eq!(value(&r, "minimum_tax_credit"), available.min(5023));
        assert_eq!(value(&r, "minimum_tax_credit_carryforward"), (available - 5023).max(0));
    }
}

#[test]
fn minimum_tax_carryforward_is_reduced_a_third_of_each_excluded_dollar() {
    let mut d = complete(FederalStatus::Single);
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    add(&mut d, insolvency_debt(2000, 10000, 5000, 3000));
    // Insolvent by $5,000: all $2,000 excluded. No NOL or business credit;
    // the credit carried to 2027 is far above $2,000 / 3, so Form 982 line 8
    // takes all $2,000 and the credit falls by $666.67. Basis is not reached
    // (after the discharge basis and cash $3,000 < liabilities $8,000).
    let r = supported(&d);
    let carried = line_amount(&r, "minimum_tax_credit_carryforward");
    assert!(carried > 66667);
    assert_eq!(value(&r, "debt_2_form982_line8"), 2000);
    let reduction = r.lines.iter().find(|l| l.id == "debt_2_mtc_reduction").unwrap();
    assert_eq!(reduction.amount, 66667);
    let after = r.lines.iter().find(|l| l.id == "debt_2_mtc_after").unwrap();
    assert_eq!(after.amount, carried - 66667);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 0);
    // A credit fully used in 2026 leaves no credit attribute for reduction.
    let h = &mut d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts;
    h.carry = 0;
    h.amt = 100000;
    let r = supported(&d);
    assert_eq!(value(&r, "minimum_tax_credit"), 1000);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 0);
}

#[test]
fn minimum_tax_credit_stops_at_tentative_tax_even_without_current_amt() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10000000, 0);
    let before = supported(&d);
    assert_eq!(value(&before, "alternative_minimum_tax"), 0);
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let r = supported(&d);
    // 2026 tentative tax: ($100,000 AMTI - $90,100 exemption) * 26%.
    // Regular tax exceeds it, but Form 8801 cannot use that $2,574 as capacity.
    let limit = value(&before, "regular_income_tax") - 2574;
    assert_eq!(value(&r, "minimum_tax_credit_limit"), limit);
    assert_eq!(value(&r, "minimum_tax_credit"), limit);
    assert_eq!(r.tax, Some(257400));
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 15000 - limit);
}

#[test]
fn minimum_tax_credit_balance_reuses_explicit_tax_contexts_without_mutation() {
    let history = minimum_tax_history();
    let balance = history.balance(FederalStatus::Single).unwrap();
    assert_eq!(balance.excluded, 0);
    assert_eq!(balance.generated, 10000);
    assert_eq!(balance.available, 15000);
    let actual = balance.limit(26978, 22337);
    assert_eq!((actual.limit, actual.used, actual.carried), (4641, 4641, 10359));
    // Independent calculation contexts reuse a balance without mutation.
    // Section1062 excludes this credit, so it does not need either context.
    let changed_regular_only = balance.limit(20978, 22337);
    assert_eq!((changed_regular_only.used, changed_regular_only.carried), (0, 15000));
    let changed_both = balance.limit(20978, 16337);
    assert_eq!((changed_both.used, changed_both.carried), (4641, 10359));
    assert_eq!(balance.limit(26978, 22337), actual);
    assert_eq!(history.balance(FederalStatus::Single), Some(balance));
    assert_eq!(balance.limit(0, 0).carried, 15000);
    assert_eq!(balance.limit(20000, 0).used, 15000);
    assert_eq!(balance.limit(20000, 0).carried, 0);
    assert!(history.balance(FederalStatus::Joint).is_none());
    let mut unknown = history;
    unknown.mtcnol = crate::MinimumTaxLoss::Unavailable;
    assert!(unknown.balance(FederalStatus::Single).is_none());
    let mut d = farmland_installment_return();
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let r = supported(&d);
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 600000);
}

#[test]
fn minimum_credit_source_projection_preserves_two_band_occupants_and_amt_gains() {
    let rates = crate::income_tax::Rates::for_year(2025, FederalStatus::Single).unwrap();
    let source =
        rates.minimum_tax_gains(400000, [250000, 100000, 50000], [300000, 120000, 60000]).unwrap();
    assert_eq!(source.normalized().preferential, 12000000);
    assert_eq!(source.normalized().combined, 18000000);
    assert_eq!(source.normalized().regular_zero, 30000000);
    assert_eq!(source.normalized().regular_fifteen, 19730000);
    // The gain pool is retained before the later AMT taxable-income cap.
    let low = rates.minimum_tax_gains(5000, [20000, 0, 0], [20000, 0, 0]).unwrap();
    assert_eq!(low.normalized().preferential, 2000000);
    assert_eq!(low.normalized().combined, 2000000);
    assert_eq!(low.normalized().regular_zero, 0);
    assert_eq!(low.normalized().regular_fifteen, 0);
    assert!(rates.minimum_tax_gains(40000, [0; 3], [0; 3]).is_none());
}

#[test]
fn registered_emulsion_blender_credit_is_distinct_from_purchaser_use() {
    let blender = FederalEmulsionBlenderClaim {
        thousandths_of_diesel_gallon: 100_000,
        actual_cost: 20_000,
        m_registration_number: "M-SYNTHETIC-637".into(),
        statement_facts_confirmed: true,
        not_claimed_elsewhere: true,
    };
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
        unreachable!()
    };
    business.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            emulsion_off_highway: Some(fuel_use(100_000, 20_000)),
            emulsion_blender: Some(blender.clone()),
            ..FederalFuelLines::default()
        },
        expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
        records_complete: true,
    });
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let mut partner = ordinary_partner_source(20_000, true);
    partner.qbi = Some(partner_qbi_statement(20_000));
    partner.fuel_tax_credit = Some(PartnerFuelTaxCredit {
        fuels: FederalFuelLines { emulsion_blender: Some(blender), ..FederalFuelLines::default() },
        statement_complete: true,
        farm_activity: false,
    });
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));

    let mut baseline = draft.clone();
    let FederalIncome::Business { data } = &mut baseline.income[1].income else { unreachable!() };
    data.fuel_tax_credit = None;
    let FederalIncome::Partnership { data } = &mut baseline.income[2].income else {
        unreachable!()
    };
    data.fuel_tax_credit = None;
    let baseline = supported(&baseline);
    let report = supported(&draft);
    let cents = |id| line_amount(&report, id);
    assert_eq!(cents("business_2_fuel_14a"), 1_970);
    assert_eq!(cents("business_2_fuel_15a"), 460);
    assert_eq!(cents("partnership_3_fuel_15a"), 460);
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 2_900);
    assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    fn blender_mut(draft: &mut FederalDraft) -> &mut FederalEmulsionBlenderClaim {
        let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
        data.fuel_tax_credit.as_mut().unwrap().fuels.emulsion_blender.as_mut().unwrap()
    }
    blender_mut(&mut draft).m_registration_number.clear();
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
    blender_mut(&mut draft).m_registration_number = "M-SYNTHETIC-637".into();
    blender_mut(&mut draft).statement_facts_confirmed = false;
    fuel_use_left_out(&draft);
    blender_mut(&mut draft).statement_facts_confirmed = true;
    blender_mut(&mut draft).not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
    blender_mut(&mut draft).not_claimed_elsewhere = true;
    blender_mut(&mut draft).actual_cost = 100;
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
}

#[test]
fn exported_gasoline_blendstock_shares_line_16a_with_dyed_diesel() {
    let fuels = FederalFuelLines {
        gasoline_blendstock_export: Some(fuel_use(1_000_000, 200_000)),
        dyed_diesel_export: Some(fuel_use(1_000_000, 200_000)),
        export_proof_complete: Some(true),
        ..FederalFuelLines::default()
    };
    assert_eq!(fuels.labeled_lines().find(|(line, _)| *line == "16a").unwrap().1, 200);
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
        unreachable!()
    };
    business.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: fuels.clone(),
        expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
        records_complete: true,
    });
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let mut partner = ordinary_partner_source(20_000, true);
    partner.qbi = Some(partner_qbi_statement(20_000));
    partner.fuel_tax_credit =
        Some(PartnerFuelTaxCredit { fuels, statement_complete: true, farm_activity: false });
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));

    let mut baseline = draft.clone();
    let FederalIncome::Business { data } = &mut baseline.income[1].income else { unreachable!() };
    data.fuel_tax_credit = None;
    let FederalIncome::Partnership { data } = &mut baseline.income[2].income else {
        unreachable!()
    };
    data.fuel_tax_credit = None;
    let baseline = supported(&baseline);
    let report = supported(&draft);
    for source in ["business_2_fuel_16a", "partnership_3_fuel_16a"] {
        assert_eq!(line_amount(&report, source), 200);
    }
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 400);
    assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let small = FederalFuelLines {
        gasoline_blendstock_export: Some(fuel_use(5_000, 100)),
        dyed_diesel_export: Some(fuel_use(5_000, 100)),
        export_proof_complete: Some(true),
        ..FederalFuelLines::default()
    };
    assert_eq!(small.labeled_lines().find(|(line, _)| *line == "16a").unwrap().1, 1);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.export_proof_complete = Some(false);
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.export_proof_complete = Some(true);
    fuels.gasoline_blendstock_export.as_mut().unwrap().not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
}

#[test]
fn government_kerosene_vendor_uses_distinct_7a_line_and_payment() {
    let mut kerosene = government_vendor_claim(100_000);
    kerosene.sales[0].tax_relief = FederalVendorTaxRelief::BuyerWrittenConsent;
    let business_fuels = FederalFuelLines {
        vendor_state_diesel: Some(government_vendor_claim(100_000)),
        vendor_state_kerosene: Some(kerosene.clone()),
        ..FederalFuelLines::default()
    };
    let partner_fuels =
        FederalFuelLines { vendor_state_kerosene: Some(kerosene), ..FederalFuelLines::default() };
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = supported(&draft);
    for id in ["business_2_fuel_6a", "business_2_fuel_7a", "partnership_3_fuel_7a"] {
        assert_eq!(value(&report, id), 24, "{id}");
        assert_eq!(line_amount(&report, id), 2_430, "{id} retains its own cents");
    }
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 7_300);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    {
        let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
        let kerosene =
            data.fuel_tax_credit.as_mut().unwrap().fuels.vendor_state_kerosene.as_mut().unwrap();
        kerosene.sales[0].no_government_credit_card = false;
    }
    fuel_left_out(&draft);
    {
        let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
        let kerosene =
            data.fuel_tax_credit.as_mut().unwrap().fuels.vendor_state_kerosene.as_mut().unwrap();
        kerosene.sales[0].no_government_credit_card = true;
        kerosene.sales[0].taxed_244_undyed_fuel = false;
    }
    fuel_left_out(&draft);
}

#[test]
fn blocked_pump_vendor_kerosene_reaches_line_7b_and_form_1040_payment() {
    let business_fuels = FederalFuelLines {
        vendor_blocked_pump_kerosene: Some(blocked_pump_vendor_claim(
            100_000,
            FederalBlockedPumpAccess::CannotFillHighwayVehicleOrTrain,
        )),
        ..FederalFuelLines::default()
    };
    let partner_fuels = FederalFuelLines {
        vendor_blocked_pump_kerosene: Some(blocked_pump_vendor_claim(
            50_000,
            FederalBlockedPumpAccess::LockedBetweenSales,
        )),
        ..FederalFuelLines::default()
    };
    assert!(partner_fuels.valid());
    assert!(partner_fuels.qualified());
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = supported(&draft);
    for (id, amount) in [("business_2_fuel_7b", 2_430), ("partnership_3_fuel_7b", 1_215)] {
        assert_eq!(line_amount(&report, id), amount,);
    }
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 3_600);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    {
        let FederalIncome::Partnership { data } = &mut draft.income[2].income else {
            unreachable!()
        };
        data.fuel_tax_credit
            .as_mut()
            .unwrap()
            .fuels
            .vendor_blocked_pump_kerosene
            .as_mut()
            .unwrap()
            .sales[0]
            .buyer_address = "".into();
    }
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    let claim =
        data.fuel_tax_credit.as_mut().unwrap().fuels.vendor_blocked_pump_kerosene.as_mut().unwrap();
    claim.sales[0].buyer_address = "100 Example Street".into();
    claim.sales[0].not_claimed_elsewhere = false;
    fuel_left_out(&draft);
}

#[test]
fn unknown_blocked_pump_access_omits_only_that_claim() {
    let business_fuels = FederalFuelLines {
        vendor_blocked_pump_kerosene: Some(blocked_pump_vendor_claim(
            100_000,
            FederalBlockedPumpAccess::NotSure,
        )),
        vendor_state_kerosene: Some(government_vendor_claim(100_000)),
        ..FederalFuelLines::default()
    };
    let partner_fuels = FederalFuelLines {
        vendor_blocked_pump_kerosene: Some(blocked_pump_vendor_claim(
            50_000,
            FederalBlockedPumpAccess::LockedBetweenSales,
        )),
        ..FederalFuelLines::default()
    };
    let (draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels.clone(), partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 3_600);
    assert!(noted(&report, "WARN_FUEL_CLAIM_LEFT_OUT"));
    assert!(report.issues.is_empty(), "{:?}", report.issues);
    assert_eq!(line_amount(&report, "business_2_fuel_7b"), 0);
    assert_eq!(line_amount(&report, "business_2_fuel_7a"), 2_430);
    assert_eq!(line_amount(&report, "partnership_3_fuel_7b"), 1_215);
    let FederalIncome::Business { data } = &draft.income[1].income else { unreachable!() };
    assert_eq!(data.fuel_tax_credit.as_ref().unwrap().fuels, business_fuels);
}

#[test]
fn similar_fmv_exception_vehicles_aggregate_across_donees_for_section_b() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    let mut first = donated_personal_vehicle(VehicleUse::SignificantUse {
        details: "Charity used vehicle for regular deliveries".into(),
    });
    first.value = 300_000;
    let mut second = donated_personal_vehicle(VehicleUse::NeedyPerson);
    second.value = 300_000;
    if let GiftKind::Vehicle { vin, .. } = &mut second.kind {
        *vin = "TESTVIN1234567891".into();
    }
    second.charity = "Another synthetic public charity".into();
    ded.property_gifts.extend([first, second]);
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));
    set_gift_appraisal(d.deductions.as_mut().unwrap(), 0, 2);
    d.next_id = 3;
    assert!(noted(&supported(&d), "WARN_GIFT_LEFT_OUT"));
    set_gift_appraisal(d.deductions.as_mut().unwrap(), 1, 2);
    let result = supported(&d);
    assert_eq!(value(&result, "form_8283_section_b_value"), 6_000);
    assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 5_700);
    assert_eq!(estimate(&restart(d.clone()).draft), result);
}

#[test]
fn registered_bus_vendor_diesel_and_kerosene_are_distinct_form_4136_payments() {
    let business_fuels = FederalFuelLines {
        vendor_bus_diesel: Some(bus_vendor_claim(100_000)),
        vendor_bus_kerosene: Some(bus_vendor_claim(100_000)),
        ..FederalFuelLines::default()
    };
    let partner_fuels = FederalFuelLines {
        vendor_bus_diesel: Some(bus_vendor_claim(100_000)),
        vendor_bus_kerosene: Some(bus_vendor_claim(100_000)),
        ..FederalFuelLines::default()
    };
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = supported(&draft);
    for source in ["business_2", "partnership_3"] {
        for form_line in ["6b", "7c"] {
            assert_eq!(
                line_amount(&report, &alloc::format!("{source}_fuel_{form_line}")),
                1_700,
                "{source} line {form_line}"
            );
        }
    }
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 6_800);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.vendor_bus_diesel.as_mut().unwrap().sales[0]
        .buyer_waiver_current = false;
    fuel_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.vendor_bus_diesel.as_mut().unwrap().sales[0]
        .buyer_waiver_current = true;
    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .vendor_bus_kerosene
        .as_mut()
        .unwrap()
        .registration_active = false;
    fuel_left_out(&draft);
}

// A Not sure scheduled-routes answer is assumed nonscheduled: the sale then
// needs 20 adult seats, and a smaller bus is left out with its own note
// (Answers and unknowns).
#[test]
fn nonscheduled_bus_routes_require_twenty_adult_seats() {
    for (seats, credited) in [(19, false), (20, true)] {
        let mut draft = complete(FederalStatus::Single);
        let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
            unreachable!()
        };
        let mut claim = bus_vendor_claim(100_000);
        claim.sales[0].scheduled_regular_routes = false;
        claim.sales[0].adult_seats_excluding_driver = Some(seats);
        business.fuel_tax_credit = Some(FederalFuelTaxCredit {
            fuels: FederalFuelLines {
                vendor_bus_diesel: Some(claim),
                ..FederalFuelLines::default()
            },
            expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
            records_complete: true,
        });
        add(&mut draft, income(FederalIncome::Business { data: business }));
        let r = supported(&draft);

        assert_eq!(noted(&r, "WARN_FUEL_SALE_LEFT_OUT"), !credited, "{seats}");
        assert_eq!(r.lines.iter().any(|line| line.id == "business_2_fuel_6b"), credited, "{seats}");
    }
}

#[test]
fn directly_held_1097_btc_bonds_add_interest_and_schedule_3_6k_once() {
    let mut draft = complete(FederalStatus::Single);
    let baseline = supported(&draft);
    let mut qecb = direct_tax_credit_bond(
        "QECB123456",
        FederalTaxCreditBondKind::QualifiedEnergyConservation,
        100_049,
    );
    qecb.prior_carry_dollars = 100;
    let mut bab = direct_tax_credit_bond(
        "BAB1234567",
        FederalTaxCreditBondKind::BuildAmericaTaxCredit,
        45_049,
    );
    bab.issued_on = 20100615;
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![qecb, bab];
    let report = supported(&draft);
    assert_eq!(return_line(&report, "2b"), 145_100);
    assert_eq!(line_amount(&report, "bond_credit_deemed_interest"), 145_098);
    assert_eq!(value(&report, "form_8912_line1"), 1450);
    assert_eq!(value(&report, "form_8912_line3"), 100);
    assert_eq!(value(&report, "form_8912_line4"), 1550);
    assert_eq!(value(&report, "tax_credit_bond_credit"), 1550);
    assert_eq!(return_line(&report, "20"), return_line(&baseline, "20") + 155_000);
    assert_eq!(value(&report, "tax_credit_bond_carry_2027_1"), 0);
    assert_eq!(value(&report, "tax_credit_bond_carry_2027_2"), 0);
    assert_eq!(estimate(&restart(draft).draft), report);
}

#[test]
fn form_8912_net_income_tax_limit_retains_same_bond_carry() {
    let mut draft = complete(FederalStatus::Single);
    let mut first = direct_tax_credit_bond(
        "QSCB123456",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        2_000_000,
    );
    first.prior_carry_dollars = 250;
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![first];
    let report = supported(&draft);
    assert_eq!(return_line(&report, "2b"), 2_000_000);
    assert_eq!(value(&report, "form_8912_line1"), 20_000);
    assert_eq!(value(&report, "form_8912_line3"), 250);
    assert_eq!(value(&report, "form_8912_line4"), 20_250);
    assert_eq!(return_line(&report, "2b"), 2_000_000);
    assert_eq!(value(&report, "tax_credit_bond_credit"), value(&report, "form_8912_line11"));
    assert_eq!(report.tax, Some(0));
    assert_eq!(value(&report, "tax_credit_bond_used_1"), value(&report, "tax_credit_bond_credit"));
    assert_eq!(
        value(&report, "tax_credit_bond_carry_2027_1"),
        20_250 - value(&report, "tax_credit_bond_credit")
    );
    assert_eq!(estimate(&restart(draft).draft), report);
}

#[test]
fn multiple_partly_limited_bonds_allocate_allowed_credit_to_the_same_bond() {
    use libertas::AvroDecode;

    let mut draft = complete(FederalStatus::Single);
    let mut first = direct_tax_credit_bond(
        "PARTIAL001",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        1_000_000,
    );
    first.prior_carry_dollars = 250;
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![
        first,
        direct_tax_credit_bond(
            "PARTIAL002",
            FederalTaxCreditBondKind::QualifiedZoneAcademy,
            1_000_000,
        ),
    ];
    unsupported(&draft, "Tax credit bond carry allocation");
    let issue = estimate(&draft)
        .issues
        .into_iter()
        .find(|issue| issue.section == "Tax credit bond carry allocation")
        .unwrap();
    let encoded = issue.display.encode();
    let mut offset = 0;
    let decoded = crate::common::FormattedMessage::avro_decode(&encoded, &mut offset).unwrap();
    assert_eq!(offset, encoded.len());
    assert_eq!(decoded.0, "ISSUE_BOND_ALLOCATION_SUMMARY");
    assert!(matches!(
        &decoded.1[..],
        [
            libertas::LibertasMessageArgumentDecode::Currency { currency_code: a, scale: 0, minor_units: _ },
            libertas::LibertasMessageArgumentDecode::Currency { currency_code: b, scale: 0, minor_units: _ },
            libertas::LibertasMessageArgumentDecode::FormattedText(_)
        ] if a == "USD" && b == "USD"
    ));
    let libertas::LibertasMessageArgumentDecode::FormattedText(inner) = &decoded.1[2] else {
        panic!("missing allocation explanation");
    };
    let mut nested_offset = 0;
    let nested = crate::common::FormattedMessage::avro_decode(inner, &mut nested_offset).unwrap();
    assert_eq!(nested_offset, inner.len());
    assert_eq!(nested.0, "ISSUE_BOND_ALLOC_MISSING_SHARE");
    let line12 =
        value(&supported(&single_bond_with_same_total_credit(&draft)), "tax_credit_bond_credit");
    assert!(line12 > 0 && line12 < 20_250);
    assert!(estimate(&draft).issues.iter().any(|issue| {
        issue.section == "Tax credit bond carry allocation"
            && issue.message.contains(&alloc::format!("Form 8912 line 12 is ${line12}"))
    }));
    let first_used = line12 / 2;
    let second_used = line12 - first_used;
    let bonds = &mut draft.credits.as_mut().unwrap().tax_credit_bonds;
    bonds[0].allowed_credit_dollars = Some(first_used);
    bonds[1].allowed_credit_dollars = Some(second_used);
    bonds[0].allowed_credit_confirmed = true;
    bonds[1].allowed_credit_confirmed = true;
    let report = supported(&draft);
    assert_eq!(value(&report, "form_8912_line1"), 20_000);
    assert_eq!(value(&report, "form_8912_line3"), 250);
    assert_eq!(value(&report, "form_8912_line4"), 20_250);
    assert_eq!(value(&report, "tax_credit_bond_credit"), line12);
    assert_eq!(value(&report, "tax_credit_bond_used_1"), first_used);
    assert_eq!(value(&report, "tax_credit_bond_used_2"), second_used);
    assert_eq!(value(&report, "tax_credit_bond_carry_2027_1"), 10_250 - first_used);
    assert_eq!(value(&report, "tax_credit_bond_carry_2027_2"), 10_000 - second_used);
    assert!(report.lines.iter().any(|line| {
        line.id == "tax_credit_bond_carry_2027_1"
            && line.explanation.english().contains("PARTIAL001")
    }));
    assert!(report.lines.iter().any(|line| {
        line.id == "tax_credit_bond_carry_2027_2"
            && line.explanation.english().contains("PARTIAL002")
    }));
    assert_eq!(value(&report, "bond_credit_deemed_interest"), 20_000);
    assert_eq!(supported(&restart(draft.clone()).draft), report);
    assert_eq!(restart(draft.clone()).draft, draft);

    // A different documented bond split changes only the same-bond carries,
    // not Form 8912 line 12 or taxable deemed interest.
    let bonds = &mut draft.credits.as_mut().unwrap().tax_credit_bonds;
    let alternative_first = line12.min(10_250);
    bonds[0].allowed_credit_dollars = Some(alternative_first);
    bonds[1].allowed_credit_dollars = Some(line12 - alternative_first);
    let alternative = supported(&draft);
    assert_eq!(value(&alternative, "tax_credit_bond_credit"), line12);
    assert_eq!(value(&alternative, "tax_credit_bond_used_1"), alternative_first);
    assert_eq!(value(&alternative, "tax_credit_bond_carry_2027_1"), 10_250 - alternative_first);
    assert_eq!(alternative.tax, report.tax);
    assert_eq!(value(&alternative, "bond_credit_deemed_interest"), 20_000);
}

#[test]
fn multiple_bond_allocation_rejects_missing_stale_and_unbalanced_sources() {
    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![
        direct_tax_credit_bond(
            "SOURCE001",
            FederalTaxCreditBondKind::QualifiedSchoolConstruction,
            1_000_000,
        ),
        direct_tax_credit_bond(
            "SOURCE002",
            FederalTaxCreditBondKind::QualifiedZoneAcademy,
            1_000_000,
        ),
    ];
    let line12 =
        value(&supported(&single_bond_with_same_total_credit(&draft)), "tax_credit_bond_credit");
    assert!(line12 > 0 && line12 < 20_000);
    let bonds = &mut draft.credits.as_mut().unwrap().tax_credit_bonds;
    bonds[0].allowed_credit_dollars = Some(line12 / 2);
    bonds[0].allowed_credit_confirmed = true;
    unsupported(&draft, "Tax credit bond carry allocation");

    let bonds = &mut draft.credits.as_mut().unwrap().tax_credit_bonds;
    bonds[1].allowed_credit_dollars = Some(line12 - line12 / 2);
    unsupported(&draft, "Tax credit bond carry allocation");

    let bonds = &mut draft.credits.as_mut().unwrap().tax_credit_bonds;
    bonds[1].allowed_credit_dollars = Some(line12 - line12 / 2 - 1);
    bonds[1].allowed_credit_confirmed = true;
    unsupported(&draft, "Tax credit bond carry allocation");

    let bonds = &mut draft.credits.as_mut().unwrap().tax_credit_bonds;
    bonds[0].allowed_credit_dollars = Some(10_001);
    bonds[1].allowed_credit_dollars = Some(0);
    unsupported(&draft, "Tax credit bond carry allocation");

    draft.credits.as_mut().unwrap().tax_credit_bonds[0].allowed_credit_dollars = Some(-1);
    assert!(!draft.valid_stored());
}

#[test]
fn partly_limited_bonds_with_one_positive_rounded_credit_allocate_automatically() {
    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![
        direct_tax_credit_bond(
            "ONEPOSITIVE",
            FederalTaxCreditBondKind::QualifiedSchoolConstruction,
            2_000_000,
        ),
        direct_tax_credit_bond("ROUNDSZERO", FederalTaxCreditBondKind::QualifiedZoneAcademy, 49),
    ];
    let result = supported(&draft);
    let allowed = value(&result, "tax_credit_bond_credit");
    assert!(allowed > 0 && allowed < value(&result, "form_8912_line4"));
    assert_eq!(value(&result, "tax_credit_bond_used_1"), allowed);
    assert_eq!(value(&result, "tax_credit_bond_used_2"), 0);
    assert_eq!(value(&result, "tax_credit_bond_carry_2027_2"), 0);
    assert_eq!(line_amount(&result, "bond_credit_deemed_interest"), 2_000_049);
    assert_eq!(supported(&restart(draft.clone()).draft), result);

    draft.credits.as_mut().unwrap().tax_credit_bonds[1].box1_credit = 50;
    unsupported(&draft, "Tax credit bond carry allocation");
}

#[test]
fn uniquely_determined_bond_allocations_are_automatic_or_must_match() {
    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![
        direct_tax_credit_bond(
            "UNLIMITED1",
            FederalTaxCreditBondKind::QualifiedSchoolConstruction,
            10_000,
        ),
        direct_tax_credit_bond(
            "UNLIMITED2",
            FederalTaxCreditBondKind::QualifiedZoneAcademy,
            10_000,
        ),
    ];
    let unlimited = supported(&draft);
    assert_eq!(value(&unlimited, "form_8912_line4"), 200);
    assert_eq!(value(&unlimited, "tax_credit_bond_credit"), 200);
    assert_eq!(value(&unlimited, "tax_credit_bond_carry_2027_1"), 0);
    draft.credits.as_mut().unwrap().tax_credit_bonds[0].allowed_credit_dollars = Some(99);
    unsupported(&draft, "Tax credit bond carry allocation");

    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![
        direct_tax_credit_bond(
            "ZERO00001",
            FederalTaxCreditBondKind::QualifiedSchoolConstruction,
            10_000,
        ),
        direct_tax_credit_bond("ZERO00002", FederalTaxCreditBondKind::QualifiedZoneAcademy, 10_000),
    ];
    let zero = supported(&draft);
    assert_eq!(value(&zero, "tax_credit_bond_credit"), 0);
    assert_eq!(value(&zero, "tax_credit_bond_carry_2027_1"), 100);
    assert_eq!(value(&zero, "tax_credit_bond_carry_2027_2"), 100);

    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![direct_tax_credit_bond(
        "ONEBOND001",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        2_000_000,
    )];
    let automatic = supported(&draft);
    let used = value(&automatic, "tax_credit_bond_credit");
    assert!(used > 0 && used < 20_000);
    assert_eq!(value(&automatic, "tax_credit_bond_used_1"), used);
    draft.credits.as_mut().unwrap().tax_credit_bonds[0].allowed_credit_dollars = Some(used + 1);
    draft.credits.as_mut().unwrap().tax_credit_bonds[0].allowed_credit_confirmed = true;
    unsupported(&draft, "Tax credit bond carry allocation");
}

#[test]
fn direct_bond_credit_interest_enters_investment_tax_once() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    assert_eq!(value(&supported(&draft), "net_investment_income_tax"), 0);
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![direct_tax_credit_bond(
        "NIIT123456",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        100_000,
    )];
    let report = supported(&draft);
    assert_eq!(return_line(&report, "2b"), 100_000);
    assert_eq!(value(&report, "net_investment_income_tax"), 38);
    assert_eq!(value(&report, "tax_credit_bond_credit"), 1000);
}

#[test]
fn earlier_schedule_3_credit_reduces_form_8912_line_11_before_bond_credit() {
    let mut draft = complete(FederalStatus::Single);
    let credits = draft.credits.as_mut().unwrap();
    credits.dc_homebuyer_carry = Some(1_000_000);
    credits.tax_credit_bonds = vec![direct_tax_credit_bond(
        "ORDER12345",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        100_000,
    )];
    let report = supported(&draft);
    assert!(value(&report, "dc_homebuyer_credit") > 0);
    assert_eq!(value(&report, "form_8912_line11"), 0);
    assert_eq!(value(&report, "tax_credit_bond_credit"), 0);
    assert_eq!(value(&report, "tax_credit_bond_carry_2027_1"), 1000);
}

#[test]
fn form_8978_line_6l_reduces_form_8912_line_11_before_bond_credit() {
    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![direct_tax_credit_bond(
        "BBA1234567",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        1_000_000,
    )];
    let without_bba = supported(&draft);
    draft.credits.as_mut().unwrap().bba_audit_interest = Some(bba_interest_reduction());
    let with_bba = supported(&draft);
    assert_eq!(value(&with_bba, "schedule_3_line_6l"), 220);
    assert_eq!(value(&without_bba, "form_8912_line11") - value(&with_bba, "form_8912_line11"), 220);
    assert_eq!(
        value(&without_bba, "tax_credit_bond_credit") - value(&with_bba, "tax_credit_bond_credit"),
        220
    );
    assert_eq!(return_line(&with_bba, "20"), return_line(&without_bba, "20"));
    assert_eq!(estimate(&restart(draft).draft), with_bba);
}
#[test]
fn positive_form_8978_line_14_raises_form_8912_line_7_and_same_bond_capacity() {
    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![direct_tax_credit_bond(
        "BBAUP12345",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        1_000_000,
    )];
    let baseline = supported(&draft);
    draft.credits.as_mut().unwrap().bba_audit_interest = Some(bba_interest_increase());
    let calculated = supported(&draft);
    assert_eq!(return_line(&calculated, "16"), return_line(&baseline, "16") + 22_000);
    assert_eq!(value(&calculated, "form_8912_line11"), value(&baseline, "form_8912_line11") + 220,);
    assert_eq!(
        value(&calculated, "tax_credit_bond_credit"),
        value(&baseline, "tax_credit_bond_credit") + 220,
    );
    assert_eq!(
        value(&calculated, "tax_credit_bond_carry_2027_1"),
        value(&baseline, "tax_credit_bond_carry_2027_1") - 220,
    );
    assert_eq!(calculated.tax, baseline.tax);
    assert_eq!(supported(&restart(draft).draft), calculated);
}

#[test]
fn form_3800_line_6a_reduces_form_8912_line_11_before_bond_credit() {
    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().tax_credit_bonds = vec![direct_tax_credit_bond(
        "GBC1234567",
        FederalTaxCreditBondKind::QualifiedSchoolConstruction,
        1_000_000,
    )];
    let without_business = supported(&draft);
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(tip_fica_carryforward(200_000));
    let with_business = supported(&draft);
    let business = value(&with_business, "general_business_credit");
    assert_eq!(business, 2000);
    assert_eq!(
        value(&without_business, "form_8912_line11") - value(&with_business, "form_8912_line11"),
        business
    );
    assert_eq!(
        value(&without_business, "tax_credit_bond_credit")
            - value(&with_business, "tax_credit_bond_credit"),
        business
    );
    assert_eq!(return_line(&with_business, "20"), return_line(&without_business, "20"));
    assert_eq!(estimate(&restart(draft).draft), with_business);
}

#[test]
fn section_965_remains_refused() {
    // Unit A9 through the situations list.
    let d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    // M3-P15 split the one situation into its credit and its tax.
    assert_situation_refused(&d, OtherTax::Section965, &["Schedule 3 line 13d"]);
    // An empty list keeps today's result.
    assert_eq!(estimate(&d).tax, baseline.tax);
}

// C-fixture (2026-09-25, matrix row 7): a head-of-household filer with one
// young child, work-related care, an American Opportunity credit and a
// Saver's Credit all competing for a small regular tax. Every expected line
// below is computed by hand from the 2026 draft Schedule 3
// (irs.gov/pub/irs-dft/f1040s3--dft.pdf, "Created 4/27/26"), i1040gi, i2441,
// i8863, i8880, the 2026 draft Tax Table/rate schedule and Rev. Proc.
// 2025-32 as amended by P.L. 119-21, never copied from this crate's output.
fn schedule_3_matrix_draft() -> FederalDraft {
    let mut d = complete(FederalStatus::Head);
    // Filer born 1990 (age 36 in 2026): not under the AOTC refundable-credit
    // "kiddie" age bar and not senior.
    d.people.as_mut().unwrap().taxpayer.birth_date = 19900615;
    // One W-2: box 1 $36,000, box 2 $900 withheld, boxes 3-6 consistent
    // (`wage()` derives 6.2%/1.45% Social Security/Medicare amounts).
    d.income[0] = wage(1, Owner::Taxpayer, 3_600_000, 90_000);
    // One child born 2019 (age 7), SSN, lives with the filer all year
    // (`child()` defaults): a qualifying child for care, CTC and EIC.
    add_child(&mut d);
    d.dependents[0].birth_date = 20190615;
    // $3,000 of work-related care paid for that child (Form 2441 line 2/
    // Schedule 3 line 2).
    d.dependents[0].set_care_expenses(300_000);
    // $2,000 of American Opportunity qualified expenses for the filer's own
    // half-time enrollment (Form 8863 line 3 nonrefundable, line 29
    // refundable).
    let sid0_3414 = -1; // You, the first person of the catalog
    d.credits.as_mut().unwrap().education.push(FederalEducation {
        student: sid0_3414,
        expenses: education_expenses(200_000),
        method: education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
    });
    // $1,000 traditional IRA contribution, not covered by a workplace plan:
    // fully deductible (Schedule 1 line 20) and the Saver's Credit base
    // (Form 8880 line 4).
    d.adjustments.as_mut().unwrap().iras = ira_facts(100_000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = (Some(saver(0, 0))).into();
    d
}

#[test]
fn schedule_3_credits_limited_in_worksheet_order_match_hand_computation() {
    let mut d = schedule_3_matrix_draft();
    let r = {
        document_care(&mut d);
        supported(&d)
    };

    // Line 9: total income = line 11 (no adjustment besides the IRA
    // deduction changes total income vs. AGI here). AGI = $36,000 wages -
    // $1,000 IRA deduction (uncovered by a workplace plan, well under the
    // $7,500 2026 limit) = $35,000.
    assert_eq!(return_line(&r, "11a / 11b"), 3_500_000);
    // Standard deduction, head of household, under 65 and not blind: $24,150
    // (Rev. Proc. 2025-32 as amended by P.L. 119-21).
    assert_eq!(return_line(&r, "12e"), 2_415_000);
    // Taxable income = $35,000 - $24,150 = $10,850.
    assert_eq!(return_line(&r, "15"), 1_085_000);
    // 2026 draft Tax Table, head of household: $10,850 falls in the
    // "at least $10,850 but less than $10,900" row (this crate's $50-bracket
    // formula uses the $10,875 midpoint), all inside the 10% bracket
    // (up to $17,700): $10,875 x 10% = $1,087.50, rounds to $1,088.
    assert_eq!(return_line(&r, "16"), 108_800);

    // Credit Limit Worksheet order (Schedule 3 map row 7 / income_tax.rs
    // PersonalCreditPotentials::limit): care, then education, then saver,
    // before the child credit, each limited by the $1,088 remaining tax.
    //
    // Form 2441: 1 qualifying child (under 13, lived with the filer all
    // year), $3,000 <= the $3,000 one-child ceiling, filer's own $36,000
    // earned income caps nothing. Rate at AGI $35,000 (P.L. 119-21 two-step
    // phase-down): first phase 50% - (35,000-15,000)/2,000 = 50-10 = 40%
    // (floor 35%, so 40% stands); second phase-down starts at $75,000 AGI,
    // not reached, so the rate stays 40%. Potential credit = $3,000 x 40% =
    // $1,200, but only $1,088 of tax remains: Schedule 3 line 2 = $1,088,
    // consuming the entire regular tax.
    assert_eq!(value(&r, "care_credit"), 1_088);
    // The same credit is Schedule 3 line 2 and, with nothing else, line 8 (Form 1040 line 20).
    let schedule_3 = |line: &str| {
        crate::draft_return(&r)
            .iter()
            .find(|row| row.form_line == Some(FormLine::new(TaxForm::Schedule3, line)))
            .map_or(0, |row| row.amount)
    };
    assert_eq!(schedule_3("2"), 108_800);
    assert_eq!(schedule_3("8"), 108_800);
    assert_eq!(return_line(&r, "20"), 108_800);
    // Form 8863: $2,000 of AOTC-qualified expenses, no MAGI phaseout at
    // $35,000 (HoH phaseout starts at $90,000) -> 100% of the first $2,000 =
    // $2,000 nonrefundable-plus-refundable AOTC. 40% is refundable
    // regardless of the tax limit (filer is 36, not subject to the
    // refundable-AOTC age bar): $2,000 x 40% = $800 (Form 1040 line 29). The
    // $1,200 nonrefundable remainder (Schedule 3 line 3) competes for tax
    // that care already exhausted, so it is $0.
    assert_eq!(value(&r, "education_credit"), 0);
    assert_eq!(return_line(&r, "29"), 80_000);
    // Form 8880: $1,000 traditional IRA contribution, head-of-household rate
    // 50% at AGI $35,000 (at/under the $36,375 first-tier ceiling) = $500
    // potential, but no tax remains after care: Schedule 3 line 4 = $0.
    assert_eq!(value(&r, "saver_credit"), 0);

    // Schedule 8812: 1 qualifying child under 17 with an SSN -> $2,200
    // potential (no MAGI phaseout below $200,000). No tax remains after
    // care, so the nonrefundable child credit (Form 1040 line 19) is $0 and
    // the full $2,200 shifts to the additional child tax credit, capped at
    // $1,700/child and at 15% x ($36,000 earned income - $2,500) = $5,025
    // (Form 1040 line 28 = $1,700).
    assert_eq!(return_line(&r, "19"), 0);
    assert_eq!(return_line(&r, "28"), 170_000);

    // No foreign tax credit (Schedule 3 line 1 = $0); no 6a-6z credits
    // (Schedule 3 line 7 = $0). Schedule 3 line 8 = line 1 + care + education
    // + saver + line 7 = $0 + $1,088 + $0 + $0 + $0 = $1,088, feeding Form
    // 1040 line 20.
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert_eq!(return_line(&r, "20"), 108_800);
    // Line 21 = child credit ($0) + line 20 ($1,088) = $1,088; line 22 =
    // $1,088 tax - $1,088 credits = $0.
    assert_eq!(return_line(&r, "21"), 108_800);
    assert_eq!(return_line(&r, "22"), 0);
    // No self-employment or other additional taxes: line 24a = line 22.
    assert_eq!(return_line(&r, "24a"), 0);
    // Form W-2 box 2 withholding.
    assert_eq!(return_line(&r, "25a"), 90_000);
    // Schedule EIC, draft 2026 table, 1 child, head of household ("single"
    // column). Earned income $36,000 -> $2,488; AGI $35,000 is above the
    // $23,890 one-child phase-out start, so it is also looked up -> $2,648;
    // the worksheet uses the smaller amount, $2,488.
    assert_eq!(return_line(&r, "27a"), 248_800);
    // Line 33 = withholding ($900) + refundable (EIC $2,488 + ACTC $1,700 +
    // refundable education $800 = $4,988) = $5,888; with $0 total tax, line
    // 34 (overpayment) is the same $5,888.
    assert_eq!(return_line(&r, "33"), 588_800);
    assert_eq!(return_line(&r, "34"), 588_800);
}

#[test]
fn exhausted_tax_pushes_the_child_credit_to_the_additional_credit() {
    let mut d = schedule_3_matrix_draft();
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    // The $1,088 regular tax is fully consumed by the $1,200-potential care
    // credit (Schedule 3 line 2), in the first Credit Limit Worksheet slot.
    assert_eq!(value(&r, "care_credit"), 1_088);
    // Nothing remains for education or the Saver's Credit.
    assert_eq!(value(&r, "education_credit"), 0);
    assert_eq!(value(&r, "saver_credit"), 0);
    // The $2,200 child-credit potential has no tax capacity left, so the
    // nonrefundable child credit (Form 1040 line 19) is $0 and the entire
    // ACTC-eligible portion ($1,700, capped by both the $1,700/child ceiling
    // and 15% of earned income over $2,500) becomes the additional child tax
    // credit (Form 1040 line 28).
    assert_eq!(return_line(&r, "19"), 0);
    assert_eq!(return_line(&r, "28"), 170_000);
}

#[test]
fn schedule_3_line_8_feeds_form_1040_line_20() {
    let mut d = schedule_3_matrix_draft();
    let r = {
        document_care(&mut d);
        supported(&d)
    };
    // No foreign tax credit: Schedule 3 line 1 is $0.
    let foreign = value(&r, "foreign_tax_credit");
    assert_eq!(foreign, 0);
    // The 2026 draft Schedule 3 line 8 instructions read "Add lines 1
    // through 4, 5a, and 7." Every named nonrefundable-credit id besides
    // care is $0 in this return (education and saver are crowded out by the
    // exhausted tax; every other id below has no supporting fact at all, so
    // it is unconditionally emitted at $0; a fact-gated id such as
    // "general_business_credit" or "minimum_tax_credit" is not emitted at
    // all when its underlying facts are absent and is correctly left out of
    // this sum). This total, built from the model's own line-2-through-6z
    // ids rather than copied from any single aggregate, must equal Form 1040
    // line 20.
    let line8 = foreign
        + value(&r, "care_credit")
        + value(&r, "education_credit")
        + value(&r, "saver_credit")
        + value(&r, "residential_energy_credit")
        + value(&r, "adoption_nonrefundable_credit")
        + value(&r, "elderly_credit")
        + value(&r, "new_clean_vehicle_credit")
        + value(&r, "previously_owned_clean_vehicle_credit")
        + value(&r, "mortgage_interest_credit")
        + value(&r, "dc_homebuyer_credit")
        + value(&r, "personal_refueling_credit")
        + value(&r, "legacy_ev_passive_credit");
    assert_eq!(line8, 1_088);
    assert_eq!(return_line(&r, "20"), line8 * 100);
}

// T3-3800: Form 3800 with several component credits.
fn wotc_employee(
    label: &str,
    group: WorkOpportunityGroup,
    began: u32,
    hours: u32,
    first_2026: i64,
    first_2025: i64,
) -> WorkOpportunityEmployee {
    WorkOpportunityEmployee {
        label: label.into(),
        group,
        began,
        first_year_hours: hours,
        first_year_paid_in_year: first_2026,
        first_year_paid_in_prior_year: first_2025,
    }
}
fn payroll_w2(label: &str, wages: i64) -> PayrollW2 {
    PayrollW2 { label: label.into(), wages, medicare: wages }
}
// Single, W-2 wages $60,000; a Schedule C employer whose receipts equal its
// deductible expenses after the credit reductions, so its net profit is zero
// and the Form 1040 tax comes from the W-2 alone.
fn employer_credit_draft() -> FederalDraft {
    let mut draft = restaurant_tip_credit_draft();
    let business = restaurant_business_mut(&mut draft);
    business.employer_tip_fica_credit = None;
    business.expenses = vec![
        ordinary_business_expense(BusinessExpenseCategory::Wages, 3_000_000),
        ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Retirement plan set-up".into() },
            300_000,
        ),
    ];
    business.payroll = BusinessPayroll::Yes {
        statements: vec![
            payroll_w2("Employee A", 900_000),
            payroll_w2("Employee B", 1_000_000),
            payroll_w2("Employee C", 1_100_000),
        ],
        attribution: crate::PayrollQbiAttribution::EntireBusiness,
    };
    business.work_opportunity_credit = Some(WorkOpportunityCredit {
        employees: vec![
            wotc_employee(
                "Employee A",
                WorkOpportunityGroup::Standard,
                20250701,
                600,
                500_000,
                200_000,
            ),
            wotc_employee(
                "Employee B",
                WorkOpportunityGroup::LongTermFamilyAssistance {
                    second_year_paid_in_year: 800_000,
                    second_year_paid_in_prior_year: 500_000,
                },
                20240301,
                1_000,
                0,
                0,
            ),
            wotc_employee(
                "Employee C",
                WorkOpportunityGroup::DisabledVeteran,
                20251001,
                300,
                900_000,
                300_000,
            ),
        ],
        confirmed: true,
    });
    business.retirement_plan_credit = Some(RetirementPlanCredit {
        startup: Some(PlanStartupCosts {
            first_credit_year: 2025,
            employees: 12,
            eligible_employees: 10,
            costs: 300_000,
        }),
        auto_enrollment_since: Some(2025),
        confirmed: true,
    });
    // Receipts $24,150 = wages $30,000 - $6,350 credit + set-up $3,000 - $2,500.
    business.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 2_415_000,
        accounting: BusinessAccounting::Cash,
    };
    draft
}
fn carryforward(credit: BusinessCredit, year: u32, dollars: i64) -> BusinessCreditCarryforward {
    BusinessCreditCarryforward {
        filer: crate::CarryFiler::SameFilers,
        research_source: None,
        credit,
        origin_year: year,
        amount: dollars * 100,
        passive: None,
    }
}
fn credit_history(
    carryforwards: Vec<BusinessCreditCarryforward>,
    carrybacks: Vec<BusinessCreditCarryback>,
) -> BusinessCreditHistory {
    BusinessCreditHistory {
        credits: Vec::new(),
        carryforwards,
        carrybacks,
        spouse_without_credits: false,
    }
}

#[test]
fn form_3800_component_credits_match_hand_computation() {
    // Form 5884 (i5884):
    //   A: standard group, 400+ hours: min($5,000, $6,000 - $2,000) = $4,000
    //      x 40% = line 1b $1,600.
    //   B: long-term family assistance, second year: min($8,000, $10,000 -
    //      $5,000) = $5,000 x 50% = line 1c $2,500.
    //   C: disabled veteran, 300 hours: min($9,000, $12,000 - $3,000) =
    //      $9,000 x 25% = line 1a $2,250. Line 2 = $6,350 (Part III 4b).
    // Form 8881 (i8881): 12 employees on line A -> line 2 = 100% x $3,000;
    //   line 3 = 10 x $250 = $2,500; line 4 = max($500, $2,500) = $2,500;
    //   line 5 = $2,500 (Part III 1j). Line 9 = $500 (Part III 1dd).
    // Schedule C: wages $30,000 - $6,350 = $23,650 on line 26; set-up
    //   $3,000 - $2,500 = $500; receipts $24,150 -> net profit $0.
    // Form 1040: AGI $60,000; taxable $60,000 - $16,100 = $43,900; Tax Table
    //   $43,900-$43,950 single = $5,023. No AMT (AMTI below the exemption),
    //   tentative minimum tax $0, no other credits.
    // Form 3800 (i3800): carryforwards 2020 Form 8881 Part I $1,000 and 2023
    //   Form 5884 $2,000; 2025 carryback of the 2026 Form 5884 $1,350.
    //   Part I: line 1 = $2,500 + $500 = $3,000; line 4 = $1,000; line 6 =
    //   $4,000. Part II: lines 7, 9, 11, 12 = $5,023; 13 = 0 (under
    //   $25,000); 14, 15 = 0; 16 = $5,023; 17 = min($4,000, $5,023) =
    //   $4,000; 27 = $5,023; 28 = $4,000; 29 = $1,023; 30 = $6,350; 34 =
    //   $2,000; 36 = $8,350; 37 = $1,023; 38 = $5,023 (Schedule 3 line 6a).
    //   FIFO: Part I uses 2020 $1,000, then 2026 8881 Part I $2,500 and Part
    //   II $500. Section C uses the 2023 Form 5884 first: $1,023, leaving
    //   $977 for 2027; 2026 Form 5884 uses $0, $1,350 goes back to 2025,
    //   $5,000 carries to 2027. Tax after credits $0.
    let mut draft = employer_credit_draft();
    draft.credits.as_mut().unwrap().business_credits = Some(credit_history(
        vec![
            carryforward(BusinessCredit::PensionStartup, 2020, 1_000),
            carryforward(BusinessCredit::WorkOpportunity, 2023, 2_000),
        ],
        vec![BusinessCreditCarryback {
            research_source: None,
            credit: BusinessCredit::WorkOpportunity,
            amount: 135_000,
            year: 2025,
        }],
    ));
    let r = supported(&draft);
    for (id, amount) in [
        ("business_2_form5884_line1a", 2_250),
        ("business_2_form5884_line1b", 1_600),
        ("business_2_form5884_line1c", 2_500),
        ("business_2_form5884_line2", 6_350),
        ("business_2_form8881_line2", 3_000),
        ("business_2_form8881_line3", 2_500),
        ("business_2_form8881_line4", 2_500),
        ("business_2_form8881_line5", 2_500),
        ("business_2_form8881_line9", 500),
        ("business_2_expense_line26_wages", 23_650),
        ("business_2_net_profit", 0),
        ("regular_income_tax", 5_023),
        ("form_3800_line_1", 3_000),
        ("form_3800_line_4", 1_000),
        ("form_3800_line_6", 4_000),
        ("form_3800_line_11", 5_023),
        ("form_3800_line_13", 0),
        ("form_3800_line_16", 5_023),
        ("form_3800_line_17", 4_000),
        ("form_3800_line_29", 1_023),
        ("form_3800_line_30", 6_350),
        ("form_3800_line_34", 2_000),
        ("form_3800_line_36", 8_350),
        ("form_3800_line_37", 1_023),
        ("form_3800_line_38", 5_023),
        ("general_business_credit", 5_023),
        ("business_credit_pension_startup_2020_used", 1_000),
        ("business_credit_pension_startup_current_year_used", 2_500),
        ("business_credit_auto_enrollment_current_year_used", 500),
        ("business_credit_work_opportunity_2023_used", 1_023),
        ("business_credit_work_opportunity_2023_to_next_year", 977),
        ("business_credit_work_opportunity_current_year_used", 0),
        ("business_credit_work_opportunity_current_year_carryback_2025", 1_350),
        ("business_credit_work_opportunity_current_year_to_next_year", 5_000),
    ] {
        assert_eq!(value(&r, id), amount, "{id}");
    }
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(draft.clone()).draft), r);
}

#[test]
fn form_3800_part_i_credits_fill_line_16_before_specified_credits() {
    // Boundary: a 2020 Form 8881 carryforward of $5,024, one dollar above
    // line 16 ($5,023): line 17 = $5,023, leaving $1 for 2027; line 29 = 0,
    // so the $6,350 current Form 5884 specified credit is not used at all.
    let mut draft = employer_credit_draft();
    draft.credits.as_mut().unwrap().business_credits = Some(credit_history(
        vec![carryforward(BusinessCredit::PensionStartup, 2020, 5_024 - 3_000)],
        vec![],
    ));
    // Part I total = $2,024 carried + $3,000 current = $5,024.
    let r = supported(&draft);
    assert_eq!(value(&r, "form_3800_line_6"), 5_024);
    assert_eq!(value(&r, "form_3800_line_17"), 5_023);
    assert_eq!(value(&r, "form_3800_line_29"), 0);
    assert_eq!(value(&r, "form_3800_line_37"), 0);
    // FIFO: the 2020 credit first, then 8881 Part I $2,500, then Part II
    // gets $5,023 - $2,024 - $2,500 = $499 of its $500.
    assert_eq!(value(&r, "business_credit_pension_startup_2020_used"), 2_024);
    assert_eq!(value(&r, "business_credit_auto_enrollment_current_year_used"), 499);
    assert_eq!(value(&r, "business_credit_auto_enrollment_current_year_to_next_year"), 1);
    assert_eq!(value(&r, "business_credit_work_opportunity_current_year_to_next_year"), 6_350);
}

#[test]
fn form_3800_line_13_uses_12500_for_married_separate_unless_the_spouse_has_none() {
    // Married filing separately, W-2 wages $110,000: taxable $110,000 -
    // $16,100 = $93,900; Tax Table $93,900-$93,950 MFS = $15,376 (net regular
    // tax). Specified credit: a 2024 Form 5884 carryforward of $20,000.
    //   $12,500: line 13 = 25% x ($15,376 - $12,500) = $719; line 27 =
    //   $14,657 = the credit allowed.
    //   Spouse without credits ($25,000): line 13 = 0; credit $15,376.
    let mut draft = complete(FederalStatus::Separate);
    draft.income[0] = wage(1, Owner::Taxpayer, 11_000_000, 0);
    draft.credits.as_mut().unwrap().business_credits = Some(credit_history(
        vec![carryforward(BusinessCredit::WorkOpportunity, 2024, 20_000)],
        vec![],
    ));
    let r = supported(&draft);
    assert_eq!(value(&r, "regular_income_tax"), 15_376);
    assert_eq!(value(&r, "form_3800_line_13"), 719);
    assert_eq!(value(&r, "general_business_credit"), 14_657);
    draft.credits.as_mut().unwrap().business_credits.as_mut().unwrap().spouse_without_credits =
        true;
    let r = supported(&draft);
    assert_eq!(value(&r, "form_3800_line_13"), 0);
    assert_eq!(value(&r, "general_business_credit"), 15_376);
}

#[test]
fn form_3800_2006_carryforward_expires_and_pre_2007_work_opportunity_is_part_i() {
    // Boundary: 2006 is the last year whose credit reaches 2026 (20 years).
    // A 2006 Form 5884 credit is not specified (Part IV line 2j), so it
    // competes in Part I: tax $5,023 (single, W-2 $60,000), 2006 credit
    // $6,000 -> used $5,023, $977 expires (section 196 deduction in 2027).
    // A 2007 Form 5884 credit is specified (line 4b) and gets no room.
    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().business_credits = Some(credit_history(
        vec![
            carryforward(BusinessCredit::WorkOpportunity, 2006, 6_000),
            carryforward(BusinessCredit::WorkOpportunity, 2007, 1_000),
        ],
        vec![],
    ));
    let r = supported(&draft);
    assert_eq!(value(&r, "form_3800_line_4"), 6_000);
    assert_eq!(value(&r, "form_3800_line_34"), 1_000);
    assert_eq!(value(&r, "business_credit_work_opportunity_2006_used"), 5_023);
    assert_eq!(value(&r, "business_credit_work_opportunity_2006_expired"), 977);
    assert_eq!(value(&r, "business_credit_work_opportunity_2007_to_next_year"), 1_000);
    // A credit before its first year, or a 2025 Form 8846 in the list, is
    // not a valid stored carryforward.
    draft.credits.as_mut().unwrap().business_credits =
        Some(credit_history(vec![carryforward(BusinessCredit::AutoEnrollment, 2019, 500)], vec![]));
    assert!(!draft.valid_stored());
    draft.credits.as_mut().unwrap().business_credits = Some(credit_history(
        vec![carryforward(BusinessCredit::EmployerTipFica, 2025, 500)],
        vec![],
    ));
    assert!(!draft.valid_stored());
}

// Form 8910 (map form-8910.md, 2026-10-09): the section 30B credit ended for
// vehicles bought after 2021, so only Form 3800 Part IV line 2x carryforwards
// remain. Base: single, W-2 $60,000 -> taxable $43,900, tax $5,023.
#[test]
fn form_3800_line_2x_alternative_motor_vehicle_carryforward_is_used_in_full() {
    // Fixture A: a 2019 credit of $3,000 (not specified, Part I).
    //   Part I line 4 = 3,000; line 17 = min(3,000, 5,023) = 3,000;
    //   used 3,000, to 2027 0; tax 5,023 - 3,000 = 2,023.
    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().business_credits = Some(credit_history(
        vec![carryforward(BusinessCredit::AlternativeMotorVehicle, 2019, 3_000)],
        vec![],
    ));
    let r = supported(&draft);
    assert_eq!(value(&r, "form_3800_line_4"), 3_000);
    assert_eq!(value(&r, "form_3800_line_17"), 3_000);
    assert_eq!(value(&r, "general_business_credit"), 3_000);
    assert_eq!(value(&r, "business_credit_alternative_motor_vehicle_2019"), 3_000);
    assert_eq!(
        printed_line(&r, "business_credit_alternative_motor_vehicle_2019"),
        (crate::TaxForm::Form3800, "2x".into())
    );
    assert_eq!(value(&r, "business_credit_alternative_motor_vehicle_2019_used"), 3_000);
    assert_eq!(value(&r, "business_credit_alternative_motor_vehicle_2019_to_next_year"), 0);
    assert_eq!(r.tax, Some(202_300));
    assert_eq!(supported(&restart(draft.clone()).draft), r);
    assert_eq!(
        restart(draft).draft.credits.unwrap().business_credits.unwrap().carryforwards.len(),
        1
    );
}

#[test]
fn form_3800_line_2x_2006_credit_expires_in_its_twentieth_year() {
    // Fixture B: 2006 $6,000 and 2007 $1,000.
    //   Part I line 4 = 7,000; line 17 = min(7,000, 5,023) = 5,023.
    //   FIFO: 2006 used 5,023, left 6,000 - 5,023 = 977, 2006 + 20 = 2026 so
    //   977 expires (no section 196 deduction, 196(c)); 2007 used 0, to 2027
    //   1,000; tax 5,023 - 5,023 = 0.
    let mut draft = complete(FederalStatus::Single);
    draft.credits.as_mut().unwrap().business_credits = Some(credit_history(
        vec![
            carryforward(BusinessCredit::AlternativeMotorVehicle, 2006, 6_000),
            carryforward(BusinessCredit::AlternativeMotorVehicle, 2007, 1_000),
        ],
        vec![],
    ));
    let r = supported(&draft);
    assert_eq!(value(&r, "form_3800_line_4"), 7_000);
    assert_eq!(value(&r, "form_3800_line_17"), 5_023);
    assert_eq!(value(&r, "business_credit_alternative_motor_vehicle_2006_used"), 5_023);
    assert_eq!(value(&r, "business_credit_alternative_motor_vehicle_2006_expired"), 977);
    assert_eq!(value(&r, "business_credit_alternative_motor_vehicle_2007_used"), 0);
    assert_eq!(value(&r, "business_credit_alternative_motor_vehicle_2007_to_next_year"), 1_000);
    assert_eq!(r.tax, Some(0));
    // Section 30B starts after 2005, so 2005 is below the field minimum.
    let mut early = carryforward(BusinessCredit::AlternativeMotorVehicle, 2005, 100);
    early.origin_year = 2005;
    draft.credits.as_mut().unwrap().business_credits = Some(credit_history(vec![early], vec![]));
    assert!(!draft.valid_stored());
}

#[test]
fn form_3800_line_2x_follows_form_8881_part_i_and_a_passive_amount_waits() {
    // Fixture C: pension 2020 $4,000, AMV 2020 $2,000, passive AMV 2021 $1,500
    //   (no passive income). Nonpassive Part I = 6,000; line 17 =
    //   min(6,000, 5,023) = 5,023. Order (i3800): Form 8881 Part I before
    //   alternative motor vehicle: pension used 4,000; AMV used
    //   5,023 - 4,000 = 1,023, to 2027 2,000 - 1,023 = 977. Passive 2021:
    //   Form 8582-CR line 5 = 0 -> used 0, to 2027 1,500. Tax 0.
    let mut draft = complete(FederalStatus::Single);
    let mut passive = carryforward(BusinessCredit::AlternativeMotorVehicle, 2021, 1_500);
    passive.passive = Some(PassiveWorksheet::Other);
    draft.credits.as_mut().unwrap().business_credits = Some(credit_history(
        vec![
            carryforward(BusinessCredit::PensionStartup, 2020, 4_000),
            carryforward(BusinessCredit::AlternativeMotorVehicle, 2020, 2_000),
            passive,
        ],
        vec![],
    ));
    let r = supported(&draft);
    assert_eq!(value(&r, "form_3800_line_17"), 5_023);
    assert_eq!(value(&r, "business_credit_pension_startup_2020_used"), 4_000);
    assert_eq!(value(&r, "business_credit_alternative_motor_vehicle_2020_used"), 1_023);
    assert_eq!(value(&r, "business_credit_alternative_motor_vehicle_2020_to_next_year"), 977);
    assert_eq!(
        value(&r, "business_credit_alternative_motor_vehicle_2021_passive_to_next_year"),
        1_500
    );
    assert_eq!(r.tax, Some(0));
    // A carryback of a credit that cannot arise in 2026 is rejected.
    let history = credit_history(
        vec![],
        vec![BusinessCreditCarryback {
            research_source: None,
            credit: BusinessCredit::AlternativeMotorVehicle,
            amount: 50_000,
            year: 2025,
        }],
    );
    assert_eq!(history.rule(), Some("BUSINESS_CREDIT_CARRYFORWARD_ONLY"));
    let source = include_str!("../../credits/general_business_credit.rs");
    assert!(source.contains("\"BUSINESS_CREDIT_CARRYFORWARD_ONLY\""));
}

#[test]
fn form_5884_and_8881_limits_by_hand() {
    // Form 5884 hours: 399 hours earns 25%, 400 earns 40% ($4,000 wages).
    let credit = |hours| WorkOpportunityCredit {
        employees: vec![wotc_employee(
            "E",
            WorkOpportunityGroup::Standard,
            20250601,
            hours,
            400_000,
            0,
        )],
        confirmed: true,
    };
    assert_eq!(credit(399).form_lines(), [1_000, 0, 0, 1_000]);
    assert_eq!(credit(400).form_lines(), [0, 1_600, 0, 1_600]);
    // Veteran limits: $24,000 less $20,000 counted in 2025 leaves $4,000.
    let veteran = WorkOpportunityCredit {
        employees: vec![wotc_employee(
            "V",
            WorkOpportunityGroup::DisabledLongUnemployedVeteran,
            20250601,
            1_000,
            1_000_000,
            2_000_000,
        )],
        confirmed: true,
    };
    assert_eq!(veteran.credit(), 1_600);
    // Form 8881 line 2: 50 employees earn 100%, 51 earn 50% of $4,000.
    let startup = |employees, eligible| PlanStartupCosts {
        first_credit_year: 2026,
        employees,
        eligible_employees: eligible,
        costs: 400_000,
    };
    assert_eq!(startup(50, 30).form_lines()[4], 4_000);
    assert_eq!(startup(51, 30).form_lines()[4], 2_000);
    // Line 4: at least $500 (one eligible employee is $250), at most $5,000.
    assert_eq!(startup(10, 1).form_lines()[3], 500);
    assert_eq!(startup(10, 1).form_lines()[4], 500);
    assert_eq!(startup(10, 30).form_lines()[3], 5_000);
}

#[test]
fn form_5884_start_dates_and_payroll_are_refused_by_name() {
    // An employee who began work in 2026 earns no credit (expired).
    let mut draft = employer_credit_draft();
    restaurant_business_mut(&mut draft).work_opportunity_credit.as_mut().unwrap().employees[0]
        .began = 20260105;
    unsupported(&draft, "Work opportunity credit");
    // First-year 2026 wages need a start after January 1, 2025.
    let mut draft = employer_credit_draft();
    restaurant_business_mut(&mut draft).work_opportunity_credit.as_mut().unwrap().employees[0]
        .began = 20250101;
    unsupported(&draft, "Work opportunity credit");
    // Credit wages above the matching W-2 Medicare wages.
    let mut draft = employer_credit_draft();
    let BusinessPayroll::Yes { statements, .. } = &mut restaurant_business_mut(&mut draft).payroll
    else {
        panic!()
    };
    statements[0].medicare = 499_999;
    unsupported(&draft, "Work opportunity credit");
    // The contributions or military spouse credit (Something else).
    let mut draft = employer_credit_draft();
    restaurant_business_mut(&mut draft).retirement_plan_credit.as_mut().unwrap().confirmed = false;
    assert!(noted(&supported(&draft), "WARN_RETIREMENT_PLAN_CREDIT_LEFT_OUT"));
}

#[test]
fn k1_credits_reach_part_iii_and_passive_credits_wait_for_net_passive_income() {
    // A materially participating shareholder's box 13 code J $1,000 is a
    // nonpassive Part III line 4b credit: with the $5,023 tax it is used in
    // full.
    let mut draft = complete(FederalStatus::Single);
    let mut source = complete_ordinary_scorp(0, true);
    source.credits = vec![K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::WorkOpportunity,
        amount: 100_000,
    }];
    add(&mut draft, income(FederalIncome::Scorp { data: source }));
    let r = supported(&draft);
    assert_eq!(value(&r, "business_credit_work_opportunity_current_year"), 1_000);
    assert_eq!(value(&r, "business_credit_work_opportunity_current_year_used"), 1_000);
    assert_eq!(value(&r, "general_business_credit"), 1_000);
    // Passive (no material participation) with no net passive income:
    // Form 8582-CR line 6 is zero, lines 32 = $1,000 and 33 = 0, and the
    // credit waits for 2027.
    let mut draft = complete(FederalStatus::Single);
    let mut source = complete_ordinary_scorp(0, false);
    source.credits = vec![K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::WorkOpportunity,
        amount: 100_000,
    }];
    add(&mut draft, income(FederalIncome::Scorp { data: source }));
    let r = supported(&draft);
    assert_eq!(value(&r, "form_3800_line_32"), 1_000);
    assert_eq!(value(&r, "form_3800_line_33"), 0);
    assert_eq!(value(&r, "general_business_credit"), 0);
    assert_eq!(
        value(&r, "business_credit_work_opportunity_current_year_passive_to_next_year"),
        1_000
    );
    // With $100 of net passive income Form 8582-CR line 6 is the tax on it:
    // the credit is Worksheet 4 (not low-income housing), so only line 6
    // allows it. The tax table taxes the middle of each $50 row.
    let mut draft = complete(FederalStatus::Single);
    let mut source = complete_ordinary_scorp(100, false);
    source.credits = vec![K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::WorkOpportunity,
        amount: 100_000,
    }];
    add(&mut draft, income(FederalIncome::Scorp { data: source }));
    let r = supported(&draft);
    let taxable = value(&r, "taxable_income");
    let table = |income: i64| {
        let middle = income / 50 * 50 + 25;
        1_240 + (middle - 12_400) * 12 / 100
    };
    let line6 = table(taxable) - table(taxable - 100);
    assert!(line6 > 0, "{line6}");
    assert_eq!(value(&r, "form8582cr_line_4c"), 1_000);
    assert_eq!(value(&r, "form8582cr_line_6"), line6);
    assert_eq!(value(&r, "form8582cr_line_36"), 0);
    assert_eq!(value(&r, "form8582cr_line_37"), line6);
    assert_eq!(value(&r, "form_3800_line_33"), line6);
    assert_eq!(value(&r, "general_business_credit"), line6);
    assert_eq!(value(&r, "business_credit_work_opportunity_current_year_passive_allowed"), line6);
    assert_eq!(
        value(&r, "business_credit_work_opportunity_current_year_passive_to_next_year"),
        1_000 - line6
    );
}

#[test]
fn partnership_code_ae_reaches_form_3800_line_1j() {
    // A general partner who materially participates: box 15 code AE $700 is
    // a Part I credit on line 1j; tax $5,023 covers it.
    let mut draft = complete(FederalStatus::Single);
    let mut source = ordinary_partner_source(0, true);
    source.qbi = Some(partner_qbi_statement(0));
    source.credits = vec![K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::PensionStartup,
        amount: 70_000,
    }];
    add(&mut draft, income(FederalIncome::Partnership { data: source }));
    let r = supported(&draft);
    assert_eq!(value(&r, "form_3800_line_1"), 700);
    assert_eq!(value(&r, "business_credit_pension_startup_current_year_used"), 700);
    assert_eq!(value(&r, "general_business_credit"), 700);
}

// No tuition earns both benefits: the credit takes its expenses first and
// only the rest is available to the savings bond exclusion, even where the
// credit computes to zero (with a note), and a filer whom the App allows no
// credit keeps all of it for the exclusion.
//   Joint; wages $178,000; box 3 $4,000; bonds: proceeds $10,000, interest
//   $4,000. The student's AOTC entry: tuition $11,000, scholarship $1,000.
//   Credit expenses 10,000, the credit uses 4,000; line 2 = 7,000, 3 = 1,000,
//   line 4 = 6,000, line 8 = 2,400.
//   Line 9 = 178,000 + 4,000 = 182,000; 13 = 2,400 x 29,350 / 30,000 = 2,348;
//   line 14 = 52. AGI = 182,000 - 52 = 181,948, above 180,000: AOTC is zero.
#[test]
fn savings_bond_tuition_is_the_credits_first_even_when_the_credit_is_phased_out() {
    let build = |wages: i64| {
        let mut d = complete(FederalStatus::Joint);
        d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
        d.people.as_mut().unwrap().taxpayer.birth_date = 19750101;
        add(&mut d, income(FederalIncome::Interest { data: native_treasury_interest(400_000) }));
        add_child(&mut d);
        let child = d.dependents[0].id;
        let credits = d.credits.as_mut().unwrap();
        credits.education.push(FederalEducation {
            student: child,
            expenses: FederalEducationExpenses {
                assistance: 100_000,
                ..education_expenses(1_100_000)
            },
            method: education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
        });
        add(
            &mut d,
            income(FederalIncome::SavingsBonds {
                data: FederalSavingsBonds {
                    proceeds: 1_000_000,
                    interest: 400_000,
                    earliest_issue_date: 20050101,
                    interest_statement: 2,
                    students: vec![FederalSavingsBondStudent {
                        person: child,
                        expenses: FederalSavingsBondExpenses::EducationCredit,
                        contributions: 0,
                    }],
                },
            }),
        );
        d
    };
    let r = supported(&build(178_000));
    assert_eq!(value(&r, "form_8815_qualified_expenses"), 7_000);
    assert_eq!(value(&r, "form_8815_phaseout_reduction"), 2_348);
    assert_eq!(value(&r, "savings_bond_exclusion"), 52);
    assert_eq!(value(&r, "agi"), 181_948);
    assert_eq!(value(&r, "education_credit"), 0);
    assert_eq!(value(&r, "refundable_education_credit"), 0);
    // The credit is zero, yet its 4,000 stays used: the note says so.
    assert!(noted(&r, "NOTE_SAVINGS_BONDS_CREDIT_EXPENSES"));
    // With a credit the note is absent.
    let r = supported(&build(120_000));
    assert!(!noted(&r, "NOTE_SAVINGS_BONDS_CREDIT_EXPENSES"));
    assert_eq!(value(&r, "form_8815_qualified_expenses"), 7_000);
    // A filer another taxpayer claims takes no education credit, so none of
    // the tuition is used and the exclusion sees all of it.
    //   Single; her own lifetime learning entry: tuition $6,000. An
    //   independent filer's credit uses all 6,000 (line 2 = 0); a claimed
    //   filer's uses none (line 2 = 6,000). Box 3 $1,500 = proceeds.
    let build = |dependency: FilerDependency| {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.dependency = dependency;
        add(&mut d, income(FederalIncome::Interest { data: native_treasury_interest(150_000) }));
        let sid0_4619 = -1; // You, the first person of the catalog
        d.credits.as_mut().unwrap().education.push(FederalEducation {
            student: sid0_4619,
            expenses: education_expenses(600_000),
            method: education_choice(EducationMethod::LifetimeLearning, Answer::Yes),
        });
        add(
            &mut d,
            income(FederalIncome::SavingsBonds {
                data: FederalSavingsBonds {
                    proceeds: 150_000,
                    interest: 150_000,
                    earliest_issue_date: 20050101,
                    interest_statement: 2,
                    students: vec![FederalSavingsBondStudent {
                        person: -1,
                        expenses: FederalSavingsBondExpenses::EducationCredit,
                        contributions: 0,
                    }],
                },
            }),
        );
        d
    };
    let independent = supported(&build(FilerDependency::Independent));
    assert_eq!(value(&independent, "form_8815_qualified_expenses"), 0);
    assert_eq!(value(&independent, "savings_bond_exclusion"), 0);
    let claimed = supported(&build(FilerDependency::CanBeClaimed { claimed: true }));
    assert_eq!(value(&claimed, "form_8815_qualified_expenses"), 6_000);
    assert_eq!(value(&claimed, "savings_bond_exclusion"), 1_500);
    assert_eq!(value(&claimed, "education_credit"), 0);
}

// i8815 (2026 draft) line 2 counts tuition and fees only, so the books the
// American opportunity credit consumed must not use up tuition. The credit's
// consumed expenses fall on the course materials first (EducationUse::used_tuition);
// Form 1099-Q still reads the whole `used`.
//   Single; wages $60,000; box 3 $1,500; bonds: proceeds $1,500, interest
//   $1,500. Her AOTC entry: tuition $3,000, books bought elsewhere $2,000.
//   Qualified 5,000; the credit uses 4,000 (2,000 books + 2,000 tuition), so
//   used_tuition = 2,000 and line 2 = 3,000 - 2,000 = 1,000 (before this
//   fix the whole 4,000 fell on tuition and line 2 was 0).
//   Line 4 = 1,000; line 8 = 1,500 x 1,000 / 1,500 = 1,000; line 9 = 61,500
//   is under 101,800, so line 14 = 1,000.
#[test]
fn savings_bond_line_2_loses_only_the_tuition_the_credit_needed() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19750101;
    add(&mut d, income(FederalIncome::Interest { data: native_treasury_interest(150_000) }));
    let sid0_4692 = -1; // You, the first person of the catalog
    d.credits.as_mut().unwrap().education.push(FederalEducation {
        student: sid0_4692,
        expenses: FederalEducationExpenses {
            other_materials: 200_000,
            ..education_expenses(300_000)
        },
        method: education_choice(EducationMethod::AmericanOpportunity, Answer::Yes),
    });
    add(
        &mut d,
        income(FederalIncome::SavingsBonds {
            data: FederalSavingsBonds {
                proceeds: 150_000,
                interest: 150_000,
                earliest_issue_date: 20050101,
                interest_statement: 2,
                students: vec![FederalSavingsBondStudent {
                    person: -1,
                    expenses: FederalSavingsBondExpenses::EducationCredit,
                    contributions: 0,
                }],
            },
        }),
    );
    let credits = d.credits.as_ref().unwrap();
    let used = credits.education_expenses_used(&crate::federal_rules::people_students(&d), true);
    assert_eq!((used[0].qualified, used[0].used), (5_000, 4_000));
    assert_eq!(used[0].used_tuition, 2_000);
    let r = supported(&d);
    assert_eq!(value(&r, "form_8815_qualified_expenses"), 1_000);
    assert_eq!(value(&r, "form_8815_eligible_interest"), 1_000);
    assert_eq!(value(&r, "savings_bond_exclusion"), 1_000);
    // The credit is unchanged: 2,000 + 25% of 2,000.
    assert_eq!(value(&r, "education_credit") + value(&r, "refundable_education_credit"), 2_500);
    // Books over the cap take nothing from tuition: tuition 3,000, books
    // 5,000 -> qualified 8,000, used 4,000 all on the books, line 2 = 3,000.
    let people = crate::federal_rules::people_students(&d);
    let credits = d.credits.as_mut().unwrap();
    credits.education[0].expenses.other_materials = 500_000;
    let used = credits.education_expenses_used(&people, true);
    assert_eq!((used[0].used, used[0].used_tuition), (4_000, 0));
    // Lifetime learning: books bought elsewhere do not count, so 3,000 of
    // tuition is used whole; with no credit allowed nothing is used.
    credits.education[0].method = EducationChoice::LifetimeLearning;
    let used = credits.education_expenses_used(&people, true);
    assert_eq!((used[0].used, used[0].used_tuition), (3_000, 3_000));
    assert_eq!(credits.education_expenses_used(&people, false)[0].used_tuition, 0);
}

// Form 8941 (draft 2026 form and instructions; Rev. Proc. 2025-32 section
// 3.09 dollar amount $34,100). The 2026 county average premiums are not
// published, so the fixtures use synthetic averages ($8,000 employee-only,
// $22,000 family unless stated). W-2 box 1 equals box 5 throughout.
pub(in crate::tests) fn health_employee(
    label: &str,
    service: HealthCreditService,
    coverage: Option<(HealthCoverageTier, u32, i64, i64)>,
) -> HealthCreditEmployee {
    HealthCreditEmployee {
        label: label.into(),
        service,
        coverage: coverage.map(|(tier, months, total, paid)| EmployeeHealthCoverage {
            tier,
            months,
            total_premium: total * 100,
            employer_paid: paid * 100,
        }),
    }
}
pub(in crate::tests) fn hours(hours: u32) -> HealthCreditService {
    HealthCreditService::Hours { hours }
}
pub(in crate::tests) fn health_credit(
    employees: Vec<HealthCreditEmployee>,
    employee_only: i64,
    family: i64,
    state_subsidies: i64,
) -> SmallEmployerHealthCredit {
    SmallEmployerHealthCredit {
        coverage: HealthCreditCoverage::Marketplace,
        first_credit_year: 2026,
        employee_only_average: employee_only * 100,
        family_average: family * 100,
        employees,
        state_subsidies: state_subsidies * 100,
        confirmed: true,
    }
}
pub(in crate::tests) fn health_payroll(w2s: &[(&str, i64)]) -> BusinessPayroll {
    BusinessPayroll::Yes {
        statements: w2s.iter().map(|(label, wages)| payroll_w2(label, wages * 100)).collect(),
        attribution: crate::PayrollQbiAttribution::EntireBusiness,
    }
}
// Single owner, one Schedule C with only wages on line 26 and the employee
// benefit programs on line 14, whole dollars.
pub(in crate::tests) fn health_credit_draft(
    credit: SmallEmployerHealthCredit,
    w2s: &[(&str, i64)],
    receipts: i64,
    wages: i64,
    benefits: i64,
) -> FederalDraft {
    let mut draft = employer_credit_draft();
    let business = restaurant_business_mut(&mut draft);
    business.work_opportunity_credit = None;
    business.retirement_plan_credit = None;
    business.expenses = vec![
        ordinary_business_expense(BusinessExpenseCategory::Wages, wages * 100),
        ordinary_business_expense(BusinessExpenseCategory::EmployeeBenefits, benefits * 100),
    ];
    business.payroll = health_payroll(w2s);
    business.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: receipts * 100,
        accounting: BusinessAccounting::Cash,
    };
    business.small_employer_health_credit = Some(credit);
    draft
}
// Fixture A: four employees, one seasonal, one partial year.
fn health_fixture_a() -> FederalDraft {
    use HealthCoverageTier::{EmployeeOnly, Family};
    health_credit_draft(
        health_credit(
            vec![
                health_employee("E1", hours(2_080), Some((EmployeeOnly, 12, 7_200, 5_040))),
                health_employee("E2", hours(2_080), Some((Family, 12, 18_000, 12_600))),
                health_employee("E3", hours(1_040), Some((EmployeeOnly, 6, 3_600, 2_520))),
                health_employee("E4", HealthCreditService::Seasonal, None),
            ],
            8_000,
            22_000,
            0,
        ),
        &[("E1", 26_000), ("E2", 24_000), ("E3", 14_000), ("E4", 8_000)],
        150_000,
        72_000,
        20_160,
    )
}
pub(in crate::tests) fn health_line(r: &FederalResult, id: i64, line: &str) -> i64 {
    value(r, &alloc::format!("business_{id}_form8941_line{line}"))
}
// Fifteen employees F01-F15, each paid $30,000 for 2,080 hours, enrolled
// employee-only for 12 months at total $9,000 / paid $6,300.
fn fifteen_rows(count: usize) -> (Vec<HealthCreditEmployee>, Vec<(alloc::string::String, i64)>) {
    let rows = (1..=count)
        .map(|n| {
            health_employee(
                &alloc::format!("F{n:02}"),
                hours(2_080),
                Some((HealthCoverageTier::EmployeeOnly, 12, 9_000, 6_300)),
            )
        })
        .collect();
    let w2s = (1..=count).map(|n| (alloc::format!("F{n:02}"), 30_000)).collect();
    (rows, w2s)
}
fn fifteen_draft(count: usize) -> FederalDraft {
    let (rows, w2s) = fifteen_rows(count);
    let w2s: Vec<(&str, i64)> = w2s.iter().map(|(l, w)| (l.as_str(), *w)).collect();
    let benefits = 6_300 * count as i64;
    health_credit_draft(
        health_credit(rows, 8_000, 22_000, 0),
        &w2s,
        900_000,
        30_000 * count as i64,
        benefits,
    )
}

#[test]
fn form_8941_full_credit_with_a_seasonal_worker_and_a_partial_year() {
    // Fixture A (no phaseout). Premiums $20,160; line 14 expense $20,160.
    //   l1 = 4 employees; l2 = floor(5,200 h / 2,080) = floor(2.5) = 2
    //   l3 = (26,000 + 24,000 + 14,000) / 2 = 32,000 (E4's $8,000 excluded)
    //   cap E1 = 8,000 x 12/12 x 5,040/7,200   =  5,600.00
    //   cap E2 = 22,000 x 12/12 x 12,600/18,000 = 15,400.00
    //   cap E3 = 8,000 x 6/12 x 2,520/3,600    =  2,800.00
    //   l4 = 5,040 + 12,600 + 2,520 = 20,160; l5 = 5,600 + 15,400 + 2,800 = 23,800
    //   l6 = 20,160; l7 = 10,080; l8 = 10,080 (l2 <= 10); l9 = 10,080 (l3 <= 34,000)
    //   l10 = 0; l11 = 20,160; l12 = 10,080; l13 = 3; l14 = floor(5,200/2,080) = 2
    //   l15 = 0; l16 = 10,080 to Form 3800 Part III line 4h.
    //   Schedule C: line 14 = 20,160 - 10,080 = 10,080; line 26 = 72,000;
    //   line 28 = 82,080; net profit = 150,000 - 82,080 = 67,920.
    let draft = health_fixture_a();
    let id = draft.income[1].id;
    let r = supported(&draft);
    for (line, expected) in [
        ("1", 4),
        ("2", 2),
        ("3", 32_000),
        ("4", 20_160),
        ("5", 23_800),
        ("6", 20_160),
        ("7", 10_080),
        ("8", 10_080),
        ("9", 10_080),
        ("10", 0),
        ("11", 20_160),
        ("12", 10_080),
        ("13", 3),
        ("14", 2),
        ("15", 0),
        ("16", 10_080),
    ] {
        assert_eq!(health_line(&r, id, line), expected, "line {line}");
    }
    assert_eq!(
        printed_line(&r, &alloc::format!("business_{id}_form8941_line12")),
        (TaxForm::Form8941, "12".into())
    );
    // Lines 1, 2, 13 and 14 are counts of people, not dollars: the line kind
    // says so and the review shows the number in the row label (a typed Count
    // argument) with no dollar amount; every other line stays money.
    // The whole result, so every form-tagged line needs its review row resource.
    let rows = crate::federal_model::draft_return(&r);
    // The breakdown of every calculation step does the same for a count.
    let breakdown = crate::FederalEstimate::from(r.clone()).breakdown;
    let counts = breakdown.iter().filter(|row| row.money.is_none()).count();
    assert!(counts >= 4, "{counts}");
    assert!(breakdown.iter().filter(|row| row.money.is_none()).all(|row| row.amount == 0));
    for line in 1_u8..=16 {
        let line = alloc::format!("{line}");
        let calculation = r
            .lines
            .iter()
            .find(|l| l.id == alloc::format!("business_{id}_form8941_line{line}"))
            .unwrap();
        let row = rows
            .iter()
            .find(|row| row.form_line == Some(crate::FormLine::new(TaxForm::Form8941, &line)));
        if matches!(line.as_str(), "1" | "2" | "13" | "14") {
            assert_eq!(calculation.kind, crate::LineKind::Count, "line {line}");
            let row = row.unwrap();
            // No dollar amount at all: the shown Amount cell is empty.
            assert_eq!(row.amount, 0);
            assert_eq!(row.money, None, "line {line}");
            let expected = crate::Message::new(
                "DRAFT_ROW_WITH_COUNT",
                vec![
                    crate::MessageArg::Nested(alloc::boxed::Box::new(
                        calculation.form_line.as_ref().unwrap().row_label(&calculation.id),
                    )),
                    crate::MessageArg::Count(calculation.amount / 100),
                ],
            );
            assert_eq!(row.label, expected.encode(), "line {line}");
            assert!(
                expected.english().ends_with(&alloc::format!(": {}", calculation.amount / 100))
            );
        } else if let Some(row) = row {
            assert_eq!(calculation.kind, crate::LineKind::Money, "line {line}");
            assert_eq!(row.amount, calculation.amount, "line {line}");
            assert_eq!(row.money, Some(calculation.amount), "line {line}");
        }
    }
    assert_eq!(
        value(&r, &alloc::format!("business_{id}_expense_line14_employee_benefit_programs")),
        10_080
    );
    assert_eq!(value(&r, &alloc::format!("business_{id}_net_profit")), 67_920);
    assert_eq!(value(&r, "business_credit_small_employer_health_current_year"), 10_080);
    // The same estimate through the stored record's own arithmetic.
    let business = restaurant_business(&draft);
    let claim = business.small_employer_health_credit.as_ref().unwrap();
    assert_eq!(claim.credit(&business.payroll), 10_080);
    // Variant A2: the owner's K-1 (1065) box 15 code BA $500 is line 15 and
    // joins line 16 and the Form 3800 line 4h entry; Schedule C line 14 is
    // reduced by line 12 alone (section 280C(h)): $10,080 still.
    let mut draft = health_fixture_a();
    let mut source = ordinary_partner_source(0, true);
    source.qbi = Some(partner_qbi_statement(0));
    source.credits = vec![K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::SmallEmployerHealth,
        amount: 50_000,
    }];
    add(&mut draft, income(FederalIncome::Partnership { data: source }));
    let r = supported(&draft);
    assert_eq!(health_line(&r, id, "12"), 10_080);
    assert_eq!(health_line(&r, id, "15"), 500);
    assert_eq!(health_line(&r, id, "16"), 10_580);
    assert_eq!(value(&r, "business_credit_small_employer_health_current_year"), 10_580);
    assert_eq!(
        value(&r, &alloc::format!("business_{id}_expense_line14_employee_benefit_programs")),
        10_080
    );
}

#[test]
fn form_8941_fte_phaseout_and_the_premium_cap() {
    // Fixture B: 15 employees F01-F15, W-2 $30,000, 2,080 h, employee-only 12
    // months, total $9,000, paid $6,300 (70%); line 14 expense $94,500.
    //   l2 = 31,200 / 2,080 = 15; l3 = 450,000 / 15 = 30,000
    //   cap each = 8,000 x 6,300/9,000 = 5,600 (below the $6,300 paid: it binds)
    //   l4 = 94,500; l5 = 84,000; l6 = 84,000; l7 = 42,000
    //   WS5: 42,000 x (15 - 10)/15 = 14,000; l8 = 28,000; l9 = 28,000
    //   l11 = 94,500; l12 = 28,000; l13 = 15; l14 = 15.
    //   Schedule C line 14 = 94,500 - 28,000 = 66,500.
    let draft = fifteen_draft(15);
    let id = draft.income[1].id;
    let r = supported(&draft);
    for (line, expected) in [
        ("2", 15),
        ("3", 30_000),
        ("4", 94_500),
        ("5", 84_000),
        ("6", 84_000),
        ("7", 42_000),
        ("8", 28_000),
        ("9", 28_000),
        ("11", 94_500),
        ("12", 28_000),
        ("13", 15),
        ("14", 15),
    ] {
        assert_eq!(health_line(&r, id, line), expected, "line {line}");
    }
    assert_eq!(
        value(&r, &alloc::format!("business_{id}_expense_line14_employee_benefit_programs")),
        66_500
    );
    // B2: F15 has 2,079 hours: 31,199 / 2,080 = 14.9995 rounds down to 14;
    // l3 = 450,000 / 14 = 32,142.86 -> 32,000; l8 = 42,000 - 42,000 x 4/15 =
    // 42,000 - 11,200 = 30,800; l12 = 30,800; l14 = 14.
    let mut draft = fifteen_draft(15);
    restaurant_business_mut(&mut draft).small_employer_health_credit.as_mut().unwrap().employees
        [14]
    .service = hours(2_079);
    let r = supported(&draft);
    for (line, expected) in [("2", 14), ("3", 32_000), ("8", 30_800), ("12", 30_800), ("14", 14)] {
        assert_eq!(health_line(&r, id, line), expected, "B2 line {line}");
    }
    // B3: 25 employees of 2,080 hours is 25 FTEs: no credit, and the
    // form's lines 3-14 are skipped (shown as zero).
    let draft = fifteen_draft(25);
    let r = supported(&draft);
    assert_eq!(health_line(&r, id, "1"), 25);
    assert_eq!(health_line(&r, id, "2"), 25);
    for line in ["3", "4", "7", "9", "12", "13", "14", "16"] {
        assert_eq!(health_line(&r, id, line), 0, "B3 line {line}");
    }
    // The premiums stay deducted in full: 25 x 6,300.
    assert_eq!(
        value(&r, &alloc::format!("business_{id}_expense_line14_employee_benefit_programs")),
        157_500
    );
    assert_eq!(
        r.lines
            .iter()
            .find(|l| l.id == "business_credit_small_employer_health_current_year")
            .map_or(0, |l| l.amount),
        0
    );
    // Record-level boundaries around 10 and 24 FTEs (each row adds $2,800 of
    // line 7, so l7 = 2,800 x rows):
    //   10 rows: l8 = l7 = 28,000 (no reduction at 10 FTEs)
    //   11 rows: l7 = 30,800; reduction 30,800 x 1/15 = 2,053.33 -> 2,053;
    //            l8 = 28,747
    //   24 rows: l7 = 67,200; reduction 67,200 x 14/15 = 62,720; l8 = 4,480
    let lines = |rows| {
        let (employees, w2s) = fifteen_rows(rows);
        let w2s: Vec<(&str, i64)> = w2s.iter().map(|(l, w)| (l.as_str(), *w)).collect();
        health_credit(employees, 8_000, 22_000, 0).form_lines(&health_payroll(&w2s))
    };
    assert_eq!(lines(10)[7], 28_000);
    assert_eq!(lines(11)[7], 28_747);
    assert_eq!(lines(24)[7], 4_480);
    assert_eq!(lines(24)[11], 4_480);
    assert_eq!(lines(25)[11], 0);
}

#[test]
fn form_8941_paths_notices_and_consistency() {
    let id = health_fixture_a().income[1].id;
    let line14 = |r: &FederalResult| {
        value(r, &alloc::format!("business_{id}_expense_line14_employee_benefit_programs"))
    };
    let no_form_lines = |r: &FederalResult| {
        assert!(r.lines.iter().all(|l| !l.id.contains("form8941")
            && l.id != "business_credit_small_employer_health_current_year"));
    };
    // (i) Coverage that is not SHOP earns no credit: one notice, no Form 8941
    // lines, the premiums stay deducted in full.
    let mut draft = health_fixture_a();
    restaurant_business_mut(&mut draft).small_employer_health_credit.as_mut().unwrap().coverage =
        HealthCreditCoverage::Other;
    let r = supported(&draft);
    assert!(noted(&r, "WARN_SMALL_EMPLOYER_HEALTH_NOT_SHOP"));
    assert_eq!(
        r.review_notes
            .iter()
            .filter(|n| n.display.key == "WARN_SMALL_EMPLOYER_HEALTH_NOT_SHOP")
            .count(),
        1
    );
    no_form_lines(&r);
    assert_eq!(line14(&r), 20_160);
    // (ii) An unconfirmed scope leaves the credit out the same way.
    let mut draft = health_fixture_a();
    restaurant_business_mut(&mut draft).small_employer_health_credit.as_mut().unwrap().confirmed =
        false;
    let r = supported(&draft);
    assert!(noted(&r, "WARN_SMALL_EMPLOYER_HEALTH_CREDIT_LEFT_OUT"));
    no_form_lines(&r);
    assert_eq!(line14(&r), 20_160);
    // (iii) The county exception needs a 2025 first year: 2026 is refused
    // (the page rejects the save too); 2025 computes as fixture A.
    let mut draft = health_fixture_a();
    {
        let claim =
            restaurant_business_mut(&mut draft).small_employer_health_credit.as_mut().unwrap();
        claim.coverage = HealthCreditCoverage::CountyException;
        claim.first_credit_year = 2026;
    }
    unsupported(&draft, "Small employer health insurance credit");
    restaurant_business_mut(&mut draft)
        .small_employer_health_credit
        .as_mut()
        .unwrap()
        .first_credit_year = 2025;
    let r = supported(&draft);
    assert_eq!(health_line(&r, id, "12"), 10_080);
    // (iv) A listed employee with no W-2 in this business's payroll.
    let mut draft = health_fixture_a();
    restaurant_business_mut(&mut draft).small_employer_health_credit.as_mut().unwrap().employees
        [3]
    .label = "E9".into();
    unsupported(&draft, "Small employer health insurance credit");
    // (v) Premiums paid above the employee benefit programs expense.
    let mut draft = health_fixture_a();
    restaurant_business_mut(&mut draft).expenses[1].amount = 2_000_000;
    unsupported(&draft, "Small employer health insurance credit");
    // (ix) A second business of the same owner with its own record: all of
    // one owner's businesses are one employer. On a joint return the
    // spouse's own business computes separately (instructions Example 2).
    let business = restaurant_business(&health_fixture_a()).clone();
    let mut draft = complete(FederalStatus::Joint);
    add(&mut draft, income(FederalIncome::Business { data: business.clone() }));
    let mut second = income(FederalIncome::Business { data: business.clone() });
    second.owner = Owner::Spouse;
    add(&mut draft, second);
    let r = supported(&draft);
    let (first_id, second_id) = (draft.income[1].id, draft.income[2].id);
    assert_eq!(health_line(&r, first_id, "12"), 10_080);
    assert_eq!(health_line(&r, second_id, "12"), 10_080);
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Business { data: business.clone() }));
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let r = estimate(&draft);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "CREDIT_ONE_BUSINESS_PER_OWNER"), "{r:?}");
}

// Form 8994 (2026 draft form Rev. December 2026; section 45S as amended by
// P.L. 119-21 section 70304; Notice 2026-28). W-2 box 1 equals box 5 in every
// fixture.
fn leave_employee(
    label: &str,
    weekly_hours: u32,
    prior_year_compensation: i64,
    normal_hourly_rate: i64,
    leave_hours_hundredths: u32,
    leave_wages: i64,
    rate_hundredths: u32,
) -> PaidLeaveEmployee {
    PaidLeaveEmployee {
        label: label.into(),
        weekly_hours,
        prior_year_compensation,
        normal_hourly_rate,
        leave_hours_hundredths,
        leave_wages,
        rate_hundredths,
    }
}
fn leave_credit(employees: Vec<PaidLeaveEmployee>) -> PaidLeaveCredit {
    PaidLeaveCredit { employees, insurance: None, confirmed: true }
}
// Fixture A's two employees: both work 40 hours a week at $25.00 an hour and
// received $52,000 in 2025.
fn leave_credit_a() -> PaidLeaveCredit {
    leave_credit(vec![
        leave_employee("A", 40, 5_200_000, 2_500, 16_000, 400_000, 10_000),
        leave_employee("B", 40, 5_200_000, 2_500, 8_000, 150_000, 7_500),
    ])
}
// Single owner, one Schedule C with only wages on line 26 and (when above
// zero) the employee benefit programs on line 14, whole dollars.
fn leave_credit_draft(
    credit: PaidLeaveCredit,
    w2s: &[(&str, i64)],
    receipts: i64,
    wages: i64,
    benefits: i64,
) -> FederalDraft {
    let mut draft = employer_credit_draft();
    let business = restaurant_business_mut(&mut draft);
    business.work_opportunity_credit = None;
    business.retirement_plan_credit = None;
    business.expenses =
        vec![ordinary_business_expense(BusinessExpenseCategory::Wages, wages * 100)];
    if benefits > 0 {
        business.expenses.push(ordinary_business_expense(
            BusinessExpenseCategory::EmployeeBenefits,
            benefits * 100,
        ));
    }
    business.payroll = health_payroll(w2s);
    business.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: receipts * 100,
        accounting: BusinessAccounting::Cash,
    };
    business.paid_leave_credit = Some(credit);
    draft
}
// Fixture A: wages expense $104,000, W-2s A and B $52,000 each.
pub(in crate::tests) fn leave_fixture_a() -> FederalDraft {
    leave_credit_draft(leave_credit_a(), &[("A", 52_000), ("B", 52_000)], 300_000, 104_000, 0)
}
pub(in crate::tests) fn leave_line(r: &FederalResult, id: i64, line: &str) -> i64 {
    value(r, &alloc::format!("business_{id}_form8994_line{line}"))
}
pub(in crate::tests) fn leave_expense_line(r: &FederalResult, id: i64, suffix: &str) -> i64 {
    value(r, &alloc::format!("business_{id}_expense_{suffix}"))
}

#[test]
fn form_8994_rate_floor_cap_and_the_twelve_week_limit() {
    // Fixture B: W-2s C $40,000 and D $62,400 (amounts in cents).
    //   C: 30 h/week, $20.00/h, 360.00 h (36000 = 1200 x 30, the 12-week
    //      limit), wages paid $7,800 including a $600 bonus, rate 100%
    //      cap 20.00 x 360 = 7,200.00 -> credit wages 7,200.00
    //      7,200.00 x 10000 / 40000 = 1,800.00 (25%)
    //   D: 40 h/week, $30.00/h, 80.00 h, wages $1,600.08, rate 6667 (66.67%)
    //      cap 2,400.00; 1,600.08 x 6667 / 40000 = 266.69
    //      (12.5% + 0.25 x 16.67 = 16.6675%; 160,008 x 6667 / 40000 = 26,669.3)
    //   l1a = dollars(180,000 + 26,669) = 2,067.
    let draft = leave_credit_draft(
        leave_credit(vec![
            leave_employee("C", 30, 4_000_000, 2_000, 36_000, 780_000, 10_000),
            leave_employee("D", 40, 6_240_000, 3_000, 8_000, 160_008, 6_667),
        ]),
        &[("C", 40_000), ("D", 62_400)],
        300_000,
        102_400,
        0,
    );
    let id = draft.income[1].id;
    let r = supported(&draft);
    assert_eq!(leave_line(&r, id, "1a"), 2_067);
    assert_eq!(leave_line(&r, id, "3"), 2_067);
    assert_eq!(leave_expense_line(&r, id, "line26_wages"), 100_333);
    // The floor and the cap, per $1,000 of leave wages (hourly cap not binding):
    //   50% rate: 12.5% x 1,000.00 = 125.00; 100% rate: 25% = 250.00;
    //   75%: 18.75% = 187.50; an odd rate 5001: 12.5025% = 125.03 (125.025).
    let one = |rate: u32| {
        leave_credit(vec![leave_employee("C", 40, 5_200_000, 2_500, 4_000, 100_000, rate)])
            .employees[0]
            .clone()
    };
    let cents = |rate: u32| {
        leave_credit(vec![leave_employee("C", 40, 5_200_000, 10_000, 4_000, 100_000, rate)])
            .form_lines()[0]
    };
    assert_eq!(one(5_000).rate_hundredths, 5_000);
    assert_eq!(cents(5_000), 125);
    assert_eq!(cents(7_500), 188);
    assert_eq!(cents(10_000), 250);
    // Boundaries of the record's ranges: 12 weeks of 30 hours is accepted and
    // one hundredth of an hour more is not; $96,000 of 2025 compensation is
    // accepted and a cent more is not; 20 weekly hours and a 50% rate are the
    // lowest accepted.
    let valid = |edit: &dyn Fn(&mut PaidLeaveEmployee)| {
        let mut employee = leave_employee("C", 30, 4_000_000, 2_000, 36_000, 780_000, 10_000);
        edit(&mut employee);
        leave_credit(vec![employee]).valid()
    };
    assert!(valid(&|_| {}));
    assert!(!valid(&|e| e.leave_hours_hundredths = 36_001));
    assert!(valid(&|e| e.prior_year_compensation = 9_600_000));
    assert!(!valid(&|e| e.prior_year_compensation = 9_600_001));
    assert!(valid(&|e| {
        e.weekly_hours = 20;
        e.leave_hours_hundredths = 24_000;
    }));
    assert!(!valid(&|e| e.weekly_hours = 19));
    assert!(valid(&|e| e.rate_hundredths = 5_000));
    assert!(!valid(&|e| e.rate_hundredths = 4_999));
    assert!(!valid(&|e| e.rate_hundredths = 10_001));
    // The rule names the 12-week limit; an estimate with it is refused.
    let mut over =
        leave_credit(vec![leave_employee("C", 30, 4_000_000, 2_000, 36_001, 780_000, 10_000)]);
    assert_eq!(over.rule(), Some("PAID_LEAVE_ABOVE_12_WEEKS"));
    over.employees[0].leave_hours_hundredths = 36_000;
    assert_eq!(over.rule(), None);
}

#[test]
fn form_8994_insurance_premiums_and_k1_code_bb() {
    // Fixture C: fixture A plus creditable premiums $6,000 at a 60% policy
    // rate (6000), employee benefit programs expense $8,000, and the same
    // owner's K-1 (1065) box 15 code BB $400 from a materially participating
    // partner.
    //   l1b = 600,000 x 6000 / 40000 = 900.00 (15%)
    //   l1a 1,281; l2 400; l3 = 1,281 + 900 + 400 = 2,581
    //   Schedule C line 14 = 8,000 - 900 = 7,100; line 26 = 102,719
    //   Form 3800 line 4j: business 2,181 + K-1 400, both nonpassive 2026,
    //   merged 2,581 (Section C, a specified credit).
    let credit = |employees| PaidLeaveCredit {
        employees,
        insurance: Some(PaidLeaveInsurance { premiums: 600_000, rate_hundredths: 6_000 }),
        confirmed: true,
    };
    let mut draft = leave_credit_draft(
        credit(leave_credit_a().employees),
        &[("A", 52_000), ("B", 52_000)],
        300_000,
        104_000,
        8_000,
    );
    let id = draft.income[1].id;
    let mut source = ordinary_partner_source(0, true);
    source.qbi = Some(partner_qbi_statement(0));
    source.credits = vec![K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::PaidLeave,
        amount: 40_000,
    }];
    add(&mut draft, income(FederalIncome::Partnership { data: source }));
    let r = supported(&draft);
    for (line, expected) in [("1a", 1_281), ("1b", 900), ("2", 400), ("3", 2_581)] {
        assert_eq!(leave_line(&r, id, line), expected, "line {line}");
    }
    assert_eq!(leave_expense_line(&r, id, "line14_employee_benefit_programs"), 7_100);
    assert_eq!(leave_expense_line(&r, id, "line26_wages"), 102_719);
    assert_eq!(value(&r, "business_credit_paid_leave_current_year"), 2_581);
    // Variant C2: premiums only, no employee rows.
    let draft = leave_credit_draft(credit(vec![]), &[("A", 52_000)], 300_000, 104_000, 8_000);
    let r = supported(&draft);
    assert_eq!(leave_line(&r, id, "1a"), 0);
    assert_eq!(leave_line(&r, id, "1b"), 900);
    assert_eq!(value(&r, "business_credit_paid_leave_current_year"), 900);
    assert_eq!(leave_expense_line(&r, id, "line26_wages"), 104_000);
    // Variant C3: the premium reduction shares line 14 with Form 8941's
    // (section 280C(a) and (h)): the Form 8941 fixture's $20,160 premiums less
    // its $10,080 credit, less this $900.
    let mut draft = health_fixture_a();
    {
        let business = restaurant_business_mut(&mut draft);
        business.expenses[1].amount = 2_816_000;
        business.paid_leave_credit = Some(PaidLeaveCredit {
            employees: vec![],
            insurance: credit(vec![]).insurance,
            confirmed: true,
        });
    }
    let r = supported(&draft);
    assert_eq!(
        leave_expense_line(&r, id, "line14_employee_benefit_programs"),
        28_160 - 10_080 - 900
    );
    // Premiums of both credits above the benefits expense are refused.
    restaurant_business_mut(&mut draft).expenses[1].amount = 2_600_000;
    unsupported(&draft, "Paid family and medical leave credit");
    // The passive K-1 (1120-S) code BB of an investor waits for 2027 like
    // every passive credit (Form 8582-CR line 5 is zero without net passive
    // income).
    let mut draft = complete(FederalStatus::Single);
    let mut source = complete_ordinary_scorp(0, false);
    source.credits = vec![K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::PaidLeave,
        amount: 100_000,
    }];
    add(&mut draft, income(FederalIncome::Scorp { data: source }));
    let r = supported(&draft);
    assert_eq!(value(&r, "form_3800_line_32"), 1_000);
    assert_eq!(value(&r, "general_business_credit"), 0);
    assert_eq!(value(&r, "business_credit_paid_leave_current_year_passive_to_next_year"), 1_000);
    // A material participant's code BB $1,000 is nonpassive and used.
    let mut draft = complete(FederalStatus::Single);
    let mut source = complete_ordinary_scorp(0, true);
    source.credits = vec![K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::PaidLeave,
        amount: 100_000,
    }];
    add(&mut draft, income(FederalIncome::Scorp { data: source }));
    let r = supported(&draft);
    assert_eq!(value(&r, "business_credit_paid_leave_current_year_used"), 1_000);
    assert_eq!(value(&r, "general_business_credit"), 1_000);
}

#[test]
fn form_8994_paths_notices_and_the_employee_tests() {
    let id = leave_fixture_a().income[1].id;
    let no_form_lines = |r: &FederalResult| {
        assert!(r.lines.iter().all(
            |l| !l.id.contains("form8994") && l.id != "business_credit_paid_leave_current_year"
        ));
    };
    // (i) An unconfirmed policy or eligibility (the written policy, the
    // one-year or elected six-month service, the leave purposes and the
    // controlled-group statement are all attested by it) leaves the credit
    // out with one notice scoped to the business: no Form 8994 lines, line
    // 26 stays $104,000, the estimate is still supported.
    let mut draft = leave_fixture_a();
    restaurant_business_mut(&mut draft).paid_leave_credit.as_mut().unwrap().confirmed = false;
    let r = supported(&draft);
    assert!(noted(&r, "WARN_PAID_LEAVE_CREDIT_LEFT_OUT"));
    assert_eq!(
        r.review_notes
            .iter()
            .filter(|n| n.display.key == "WARN_PAID_LEAVE_CREDIT_LEFT_OUT")
            .count(),
        1
    );
    no_form_lines(&r);
    assert_eq!(leave_expense_line(&r, id, "line26_wages"), 104_000);
    let source = include_str!("../../credits/general_business_credit.rs");
    assert!(source.contains("at least a year (or at least six months, if you chose that period)"));
    // (ii) Nothing claimed, and a duplicate label, are the page rules.
    assert_eq!(leave_credit(vec![]).rule(), Some("PAID_LEAVE_NOTHING_CLAIMED"));
    let mut twice = leave_credit_a();
    twice.employees[1].label = "A".into();
    assert_eq!(twice.rule(), Some("PAID_LEAVE_EMPLOYEE_TWICE"));
    assert!(!twice.valid());
    // (iii) Employees who are not qualifying: under 20 hours a week, or 2025
    // compensation above $96,000, are outside the stored ranges, so the
    // estimate refuses the record.
    for edit in
        [(|e: &mut PaidLeaveEmployee| e.weekly_hours = 19) as fn(&mut PaidLeaveEmployee), |e| {
            e.prior_year_compensation = 9_600_001
        }]
    {
        let mut draft = leave_fixture_a();
        edit(
            &mut restaurant_business_mut(&mut draft).paid_leave_credit.as_mut().unwrap().employees
                [0],
        );
        // Stored data outside the ranges is invalid: no calculation runs.
        let r = estimate(&draft);
        assert_eq!(r.state, ResultState::Incomplete, "{r:?}");
        assert!(r.tax.is_none() && r.lines.is_empty());
    }
    // (iv) A label with no W-2 in this business's payroll.
    let mut draft = leave_fixture_a();
    restaurant_business_mut(&mut draft).paid_leave_credit.as_mut().unwrap().employees[1].label =
        "E9".into();
    unsupported(&draft, "Paid family and medical leave credit");
    // (v) Leave wages above the W-2's Medicare wages: A's $52,001.
    let mut draft = leave_fixture_a();
    {
        let claim = restaurant_business_mut(&mut draft).paid_leave_credit.as_mut().unwrap();
        claim.employees[0].normal_hourly_rate = 100_000;
        claim.employees[0].leave_hours_hundredths = 96_000;
        claim.employees[0].weekly_hours = 80;
        claim.employees[0].leave_wages = 5_200_100;
    }
    unsupported(&draft, "Paid family and medical leave credit");
    // (vi) Leave wages above the wages expense: $5,500 against $5,000.
    let mut draft = leave_fixture_a();
    restaurant_business_mut(&mut draft).expenses[0].amount = 500_000;
    unsupported(&draft, "Paid family and medical leave credit");
    // (vii) Section 45S(g): wages used for another credit are not leave
    // wages. A Form 5884 record with employee A's first-year wages of $50,000
    // plus A's $4,000 of leave wages is $54,000 against the $52,000 W-2.
    let mut draft = leave_fixture_a();
    restaurant_business_mut(&mut draft).work_opportunity_credit = Some(WorkOpportunityCredit {
        employees: vec![wotc_employee(
            "A",
            WorkOpportunityGroup::Standard,
            20250701,
            600,
            5_000_000,
            0,
        )],
        confirmed: true,
    });
    unsupported(&draft, "Paid family and medical leave credit");
    // The same Form 5884 wages for another employee, or smaller wages for A
    // ($48,000 + $4,000 = $52,000), leave the leave credit as it was.
    restaurant_business_mut(&mut draft).work_opportunity_credit.as_mut().unwrap().employees[0]
        .first_year_paid_in_year = 4_800_000;
    let r = supported(&draft);
    assert_eq!(leave_line(&r, id, "1a"), 1_281);
    // (viii) Payroll `No` with the record is a page rule (interview tests); a
    // passive business is left out whole with its own notice, credit included.
    let mut draft = leave_fixture_a();
    restaurant_business_mut(&mut draft).material_participation = false;
    let r = supported(&draft);
    assert!(noted(&r, "WARN_BUSINESS_LEFT_OUT"));
    no_form_lines(&r);
    // (ix) One record per owner: a second business with its own record.
    let business = restaurant_business(&leave_fixture_a()).clone();
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Business { data: business.clone() }));
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let r = estimate(&draft);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "CREDIT_ONE_BUSINESS_PER_OWNER"), "{r:?}");
}

// `K1CreditCode::ALL` is the one catalog of the K-1 credit codes. The texts
// that list the codes are attribute and resource literals, so they stay
// hand-written; this test builds each sentence from the catalog and finds
// every one of them, so a new code that misses a place fails here.
#[test]
fn k1_credit_code_texts_follow_the_catalog() {
    let page = include_str!("../../credits/business_credit_page.rs");
    let imports = include_str!("../../passthrough/k1_page.rs");
    for code in crate::K1CreditCode::ALL {
        let named = alloc::format!("{} ({code:?})", code.letters());
        assert_eq!(imports.matches(&named).count(), 2, "{named}");
        assert!(page.contains(&alloc::format!("/// Code {} {code:?}", code.letters())));
    }
    assert_eq!(imports.matches("Credits for box 13 codes H and AP through AU;").count(), 1);
    assert_eq!(imports.matches("Credits for box 15 codes H and AP through AU;").count(), 1);
    assert!(crate::K1CreditCode::ALL.len() <= 60);
    assert_eq!(imports.matches("pub credits: Vec<crate::K1CreditPage>").count(), 2);
    assert!(imports.contains("Code I is expired and must be reported as an error for 2026."));
}

// Reg. 1.41-7(c) imports Reg. 1.53-3(d)(4), Example 1: salary paid
// by the same S corporation joins the income attributable to that entity.
// The approved actual employer link now distinguishes this source fact.
#[test]
fn unit16_research_salary_source_answer_changes_only_its_research_cap() {
    extern crate std;
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 4_000_000, 0);
    add(&mut d, wage(0, Owner::Taxpayer, 5_610_000, 0));
    let mut corporation = complete_ordinary_scorp(20_000, true);
    corporation.qbi_statement.as_mut().unwrap().income = 0;
    add(&mut d, income(FederalIncome::Scorp { data: corporation }));
    let id = d.income.last().unwrap().id;
    d.credits.as_mut().unwrap().business_credits = Some(BusinessCreditHistory {
        credits: vec![],
        carryforwards: vec![BusinessCreditCarryforward {
            credit: BusinessCredit::Research,
            origin_year: 2024,
            amount: 1_000_000,
            passive: None,
            filer: CarryFiler::SameFilers,
            research_source: Some(id),
        }],
        carrybacks: vec![],
        spouse_without_credits: false,
    });
    let mut world_a = d.clone();
    if let FederalIncome::Wage { data } = &mut world_a.income[0].income {
        data.paying_scorporation = Some(id);
    }
    let world_b = d.clone();
    assert_ne!(world_a, world_b);
    assert_eq!(restart(world_a.clone()).draft, world_a);
    let a = estimate(&world_a);
    let b = estimate(&world_b);
    assert_eq!(value(&a, "general_business_credit"), 8637);
    assert_eq!(value(&b, "general_business_credit"), 2879);
    assert_eq!(value(&a, "taxable_income"), 100_000);
    assert_eq!(value(&a, "regular_income_tax"), 16_712);
    // Independent whole-dollar regulation computation, using the same
    // shared fraction rounding as the App. The standard deduction is the
    // only non-activity deduction; no current QBI deduction is claimed.
    let numerator = |income| income - crate::kernel::ratio::fraction(16_100, income, 116_100);
    let source_a = numerator(60_000);
    let source_b = numerator(20_000);
    let cap_a = crate::kernel::ratio::fraction(16_712, source_a, 100_000);
    let cap_b = crate::kernel::ratio::fraction(16_712, source_b, 100_000);
    assert_eq!((source_a, source_b), (51_680, 17_227));
    assert_eq!((cap_a, cap_b), (8_637, 2_879));
    assert_ne!(cap_a, cap_b);
    std::println!(
        "Unit16 actual employer source witness: entity={id}, TI=100000, line11=16712; same-entity salary=40000 => numerator={source_a}/cap={cap_a}; unrelated salary => numerator={source_b}/cap={cap_b}"
    );
}

#[test]
fn new_clean_vehicle_business_f1_uses_native_credit_limits() {
    // Approved F1: AGI58587; taxable38770; income tax4405; personal3000;
    // remaining Form3800 capacity1405; business4500 carries3095; SE2826.
    let draft = clean_vehicle_business_return();
    let result = supported(&draft);
    for (id, expected) in [
        ("agi", 58_587),
        ("income_tax", 4_405),
        ("clean_vehicle_1_business_part", 4_500),
        ("clean_vehicle_1_potential", 3_000),
        ("form_8936_business_part", 4_500),
        ("new_clean_vehicle_credit", 3_000),
        ("general_business_credit", 1_405),
        ("business_credit_clean_vehicle_current_year_to_next_year", 3_095),
    ] {
        assert_eq!(value(&result, id), expected, "{id}");
    }
    assert_eq!(result.tax, Some(282_600));
    assert!(!noted(&result, "WARN_CLEAN_VEHICLE_BASIS_REDUCTION"));
}

#[test]
fn new_clean_vehicle_form8936_combines_own_and_current_k1_without_prior_carries() {
    let mut draft = clean_vehicle_business_return();
    let mut source = complete_ordinary_scorp(0, true);
    source.credits = vec![K1Credit {
        carbon_oxide_origin: None,
        renewable_facility_origin: None,
        line: None,
        code: K1CreditCode::CleanVehicle,
        amount: 120_000,
    }];
    add(&mut draft, income(FederalIncome::Scorp { data: source }));
    draft.credits.as_mut().unwrap().business_credits =
        Some(credit_history(vec![carryforward(BusinessCredit::CleanVehicle, 2025, 999)], vec![]));
    let result = supported(&draft);
    // Schedule A contributes $4,500; current K-1 AY adds $1,200.
    // The $999 earlier-year carry belongs to Form 3800, not Form 8936.
    for (id, number, expected) in [
        ("form_8936_own_business_part", "6", 4_500),
        ("form_8936_k1_business_part", "7", 1_200),
        ("form_8936_business_part", "8", 5_700),
    ] {
        assert_eq!(value(&result, id), expected, "{id}");
        let line = result.lines.iter().find(|line| line.id == id).unwrap();
        assert_eq!(line.form_line, Some(FormLine::new(TaxForm::Form8936, number)));
    }
}

#[test]
fn new_clean_vehicle_credit_fraction_is_distinct_from_annual_mileage() {
    let mut draft = clean_vehicle_business_return();
    let entry = draft.income.last_mut().unwrap();
    let FederalIncome::Business { data } = &mut entry.income else { unreachable!() };
    let source_id = entry.id;
    let id = data.assets[0].id;
    data.assets.clear();
    data.vehicles.push(BusinessMileage { id, ..native_business_mileage(1, 0, 3) });
    let FederalCleanVehicle::New { business_use: Some(use_), .. } =
        &mut draft.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap().vehicles[0]
    else {
        unreachable!()
    };
    // A conversion to 1/3 business use for six months is 1/6 on line10.
    use_.credit_use = crate::UseFraction { used: 1, total: 6 };
    let result = supported(&draft);
    assert_eq!(value(&result, "clean_vehicle_1_business_part"), 1_250);
    assert_eq!(value(&result, "clean_vehicle_1_potential"), 6_250);
    assert!(!noted(&result, "WARN_CLEAN_VEHICLE_BASIS_REDUCTION"));
    let sources = crate::source_associations::SourceAssociations::new(&draft);
    assert_eq!(sources.mileage(source_id, id).unwrap().id, id);
    assert!(sources.mileage(draft.income[0].id, id).is_err());
}

#[test]
fn new_clean_vehicle_fraction_rounds_only_once_at_the_reported_dollar() {
    let mut draft = clean_vehicle_business_return();
    let FederalCleanVehicle::New { business_use: Some(use_), .. } =
        &mut draft.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap().vehicles[0]
    else {
        unreachable!()
    };
    use_.credit_use = crate::UseFraction { used: 1, total: 15_001 };
    // $7500 / 15001 = $0.499966...: business rounds to $0 once.
    // Rounding to cents first would produce $0.50 and incorrectly claim $1.
    let result = supported(&draft);
    assert_eq!(value(&result, "clean_vehicle_1_business_part"), 0);
    assert_eq!(value(&result, "clean_vehicle_1_potential"), 7_500);
}

#[test]
fn new_clean_vehicle_employee_personal_use_is_a_full_business_credit() {
    let mut draft = clean_vehicle_business_return();
    let FederalCleanVehicle::New { business_use: Some(use_), .. } =
        &mut draft.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap().vehicles[0]
    else {
        unreachable!()
    };
    // Employer's documented inclusion/reimbursement establishes 100% under
    // line10 even though the same property's actual operating mileage is 60%.
    use_.credit_use = crate::UseFraction { used: 1, total: 1 };
    let result = supported(&draft);
    assert_eq!(value(&result, "clean_vehicle_1_business_part"), 7_500);
    assert_eq!(value(&result, "clean_vehicle_1_potential"), 0);
}

#[test]
fn new_clean_vehicle_basis_correction_targets_the_actual_linked_property() {
    let mut draft = clean_vehicle_business_return();
    let entry = draft.income.last_mut().unwrap();
    let source = entry.id;
    let FederalIncome::Business { data } = &mut entry.income else { unreachable!() };
    data.assets[0].credit_reduction = 0;
    let result = supported(&draft);
    let note = result
        .review_notes
        .iter()
        .find(|note| note.display.key == "WARN_CLEAN_VEHICLE_BASIS_REDUCTION")
        .unwrap();
    assert_eq!(
        note.context.as_ref().unwrap().identity(),
        crate::IssueContext::income_id(source).item("business_assets", 0).identity()
    );
}

#[test]
fn new_clean_vehicle_thirty_day_disposition_zeros_business_and_basis() {
    for disposition in [
        crate::CleanVehicleDisposition::ReturnedWithin30Days,
        crate::CleanVehicleDisposition::ResoldWithin30Days,
    ] {
        let mut draft = clean_vehicle_business_return();
        let FederalCleanVehicle::New { details, .. } =
            &mut draft.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap().vehicles[0]
        else {
            unreachable!()
        };
        details.disposition = disposition;
        let result = supported(&draft);
        assert_eq!(value(&result, "form_8936_business_part"), 0);
        assert_eq!(value(&result, "clean_vehicle_1_potential"), 0);
        assert!(noted(&result, "WARN_CLEAN_VEHICLE_BASIS_REDUCTION"));
        let FederalIncome::Business { data } = &mut draft.income.last_mut().unwrap().income else {
            unreachable!()
        };
        data.assets[0].credit_reduction = 0;
        assert!(!noted(&supported(&draft), "WARN_CLEAN_VEHICLE_BASIS_REDUCTION"));
        if disposition == crate::CleanVehicleDisposition::ReturnedWithin30Days {
            let FederalCleanVehicle::New { details, .. } =
                &mut draft.credits.as_mut().unwrap().clean_vehicles.as_mut().unwrap().vehicles[0]
            else {
                unreachable!()
            };
            details.dealer_transfer = Some(750_000);
            let returned = supported(&draft);
            assert_eq!(value(&returned, "form_8936_business_part"), 0);
            assert_eq!(return_line(&returned, "17"), 0);
        }
    }
}

#[test]
fn locked_pump_buyer_record_refusals_locate_fields_and_keep_accepted_return() {
    use crate::tests::cross::interview::{
        accepted, business, document, reopened_businesses, save_businesses, save_topic,
        stored_business,
    };
    let mut page = business(1_000_000);
    page.more.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            vendor_blocked_pump_kerosene: Some(blocked_pump_vendor_claim(
                5_001,
                FederalBlockedPumpAccess::LockedBetweenSales,
            )),
            ..FederalFuelLines::default()
        },
        expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
        records_complete: true,
    });
    let mut session = restart(empty_return(FederalStatus::Single));
    accepted(&save_businesses(&mut session, vec![page]));
    let accepted_draft = session.draft.clone();
    let result = supported(&accepted_draft);
    session = restart(accepted_draft.clone());
    let (id, _) = stored_business(&session);
    let page = reopened_businesses(session.draft.clone()).remove(0);
    let request = save_topic(
        &session.draft,
        TaxSection::SelfEmployment,
        vec![document(id, IncomeDocument::Business(page.clone()))],
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(supported(&session.draft), result);
    for (name, address, field) in [("", "", "buyer_name"), ("Synthetic Buyer", "", "buyer_address")]
    {
        let mut invalid = page.clone();
        let sale = &mut invalid
            .more
            .fuel_tax_credit
            .as_mut()
            .unwrap()
            .fuels
            .vendor_blocked_pump_kerosene
            .as_mut()
            .unwrap()
            .sales[0];
        sale.buyer_name = name.into();
        sale.buyer_address = address.into();
        let request = save_topic(
            &session.draft,
            TaxSection::SelfEmployment,
            vec![document(id, IncomeDocument::Business(invalid))],
        );
        crate::tests::support::assert::k1res::assert_located_rejection(
            &mut session,
            request,
            "FUEL_BLOCKED_PUMP_BUYER_RECORD",
            &alloc::format!(
                "[documents/documents/0/document/more/fuel_tax_credit/fuels/vendor_blocked_pump_kerosene/sales/0/{field}]"
            ),
        );
    }
}
