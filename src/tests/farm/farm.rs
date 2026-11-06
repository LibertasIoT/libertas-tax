use super::support::assert::unit50_fuel::assert_fuel_payment;
use super::*;

#[test]
fn farm_aviation_kerosene_type_1_reaches_schedule_c_and_partner_payment() {
    let business_fuels = FederalFuelLines {
        aviation_kerosene_farm: Some(farm_aviation_kerosene_claim()),
        ..FederalFuelLines::default()
    };
    let partner_fuels = FederalFuelLines {
        aviation_kerosene_farm: Some(farm_aviation_kerosene_claim()),
        ..FederalFuelLines::default()
    };
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(20_000), business_fuels, partner_fuels, false);
    // Form 1040 reports the two $46.10 source credits in whole dollars.
    let report = assert_fuel_payment(&draft, &baseline, 9_200);
    for source in ["business_2", "partnership_3"] {
        for (line, amount) in [("5c", 2_430), ("5d", 2_180)] {
            assert_eq!(line_amount(&report, &alloc::format!("{source}_fuel_{line}")), amount);
        }
        assert_eq!(line_amount(&report, &alloc::format!("{source}_fuel_5e")), 0);
    }

    let mut rejected = draft.clone();
    let FederalIncome::Business { data } = &mut rejected.income[1].income else { unreachable!() };
    let claim =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_kerosene_farm.as_mut().unwrap();
    claim.applicator_was_ultimate_purchaser = false;
    fuel_use_left_out(&rejected);
    let FederalIncome::Business { data } = &mut rejected.income[1].income else { unreachable!() };
    let claim =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_kerosene_farm.as_mut().unwrap();
    claim.applicator_was_ultimate_purchaser = true;
    claim.no_waiver = false;
    fuel_use_left_out(&rejected);
    let FederalIncome::Business { data } = &mut rejected.income[1].income else { unreachable!() };
    let claim =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_kerosene_farm.as_mut().unwrap();
    claim.no_waiver = true;
    claim.distinct_gallons = false;
    fuel_use_left_out(&rejected);
    let FederalIncome::Business { data } = &mut rejected.income[1].income else { unreachable!() };
    let claim =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_kerosene_farm.as_mut().unwrap();
    claim.distinct_gallons = true;
    claim.taxed_at_219_direct_flight.as_mut().unwrap().qualified = false;
    fuel_use_left_out(&rejected);
    let FederalIncome::Business { data } = &mut rejected.income[1].income else { unreachable!() };
    let claim =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_kerosene_farm.as_mut().unwrap();
    claim.taxed_at_219_direct_flight.as_mut().unwrap().qualified = true;
    claim.taxed_at_244_application.as_mut().unwrap().not_claimed_elsewhere = false;
    fuel_use_left_out(&rejected);
    let FederalIncome::Business { data } = &mut rejected.income[1].income else { unreachable!() };
    let claim =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_kerosene_farm.as_mut().unwrap();
    claim.taxed_at_244_application.as_mut().unwrap().not_claimed_elsewhere = true;
    claim.taxed_at_244_application.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&rejected.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut rejected.income[1].income else { unreachable!() };
    let claim =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_kerosene_farm.as_mut().unwrap();
    claim.taxed_at_244_application.as_mut().unwrap().actual_cost = 20_000;
    claim.taxed_at_244_application.as_mut().unwrap().gallons = 0.0;
    assert!(crate::federal_rules::validate_income_amount(&rejected.income[1].income).is_err());

    let mut wrong_source = draft.clone();
    let FederalIncome::Business { data } = &mut wrong_source.income[1].income else {
        unreachable!()
    };
    data.source = BusinessSource::ForeignServices { confirmed: true, totalization_exempt: false };
    unsupported(&wrong_source, "Fuel tax credit");

    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().statement_complete = false;
    // An unconfirmed statement, or farm-operator fuel without a farm
    // activity, leaves this K-1's fuel credit out with a notice.
    let r = supported(&draft);
    assert!(noted(&r, "WARN_PARTNER_FUEL_CREDIT_LEFT_OUT"), "{r:?}");
}

#[test]
fn farm_and_foreign_trade_aviation_kerosene_round_once_per_printed_line() {
    let fuel = FederalFuelLines {
        aviation_kerosene_farm: Some(FederalFarmAviationKerosene {
            taxed_at_244_application: Some(fuel_use(21, 1)),
            taxed_at_244_direct_flight: None,
            taxed_at_219_application: None,
            taxed_at_219_direct_flight: Some(fuel_use(21, 1)),
            applicator_was_ultimate_purchaser: true,
            no_waiver: true,
            distinct_gallons: true,
        }),
        aviation_kerosene_foreign_trade: Some(FederalForeignTradeAviationKerosene {
            taxed_at_244: Some(fuel_use(21, 1)),
            taxed_at_219: Some(fuel_use(21, 1)),
            no_waiver: true,
        }),
        ..FederalFuelLines::default()
    };
    assert!(fuel.valid() && fuel.qualified());
    assert_eq!(
        fuel.labeled_lines().filter(|(_, amount)| *amount != 0).collect::<Vec<_>>(),
        vec![("5c", 1), ("5d", 1)]
    );
    // Separately rounded 5c claims would be 1 + 1; separately rounded 5d claims would be 0 + 0.
    assert_eq!(fuel.credit_cents(), 2);
}

#[test]
fn aviation_gasoline_2b_rounds_farm_off_highway_and_type_10_gallons_once() {
    let fuels = FederalFuelLines {
        aviation_gasoline_nonpropulsion_off_highway: Some(fuel_use(21, 1)),
        aviation_gasoline_farm: Some(FederalAviationGasolineFarmUse {
            aerial_application: Some(fuel_use(21, 1)),
            direct_airfield_farm_flight: None,
            applicator_was_ultimate_purchaser: true,
            distinct_gallons: true,
        }),
        aviation_gasoline_type_10: Some(FederalAviationGasolineType10Uses {
            helicopter_mineral_transport: None,
            helicopter_tree_work: None,
            helicopter_emergency_medical: Some(fuel_use(21, 1)),
            fixed_wing_tree_work: None,
            fixed_wing_emergency_medical: None,
            mineral_and_tree_airport_condition: None,
            fixed_wing_acute_care_condition: None,
            distinct_gallons: true,
        }),
        ..FederalFuelLines::default()
    };
    assert!(fuels.valid() && fuels.qualified());
    assert_eq!(fuels.labeled_lines().find(|(line, _)| *line == "2b"), Some(("2b", 1)));
    assert_eq!(fuels.credit_cents(), 1);
}

#[test]
fn farm_aviation_gasoline_type_1_reaches_schedule_f_and_partner_payment() {
    let business_fuels = FederalFuelLines {
        aviation_gasoline_farm: Some(farm_aviation_gasoline(100_000, Some(100_000))),
        ..FederalFuelLines::default()
    };
    let partner_fuels = FederalFuelLines {
        aviation_gasoline_farm: Some(farm_aviation_gasoline(100_000, None)),
        ..FederalFuelLines::default()
    };
    let (mut draft, baseline) =
        fuel_credit_sources(raised_farm(20_000, 5_000), business_fuels, partner_fuels, true);
    let report = assert_fuel_payment(&draft, &baseline, 5_800);
    for (source, amount) in [("business_2", 3_860), ("partnership_3", 1_930)] {
        assert_eq!(line_amount(&report, &alloc::format!("{source}_fuel_2b")), amount);
    }

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .aviation_gasoline_farm
        .as_mut()
        .unwrap()
        .applicator_was_ultimate_purchaser = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let farm =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_farm.as_mut().unwrap();
    farm.applicator_was_ultimate_purchaser = true;
    farm.distinct_gallons = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let farm =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_farm.as_mut().unwrap();
    farm.distinct_gallons = true;
    farm.direct_airfield_farm_flight.as_mut().unwrap().qualified = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let farm =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_farm.as_mut().unwrap();
    farm.direct_airfield_farm_flight.as_mut().unwrap().qualified = true;
    farm.aerial_application.as_mut().unwrap().not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let farm =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_farm.as_mut().unwrap();
    farm.aerial_application.as_mut().unwrap().not_claimed_elsewhere = true;
    farm.aerial_application.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let farm =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_farm.as_mut().unwrap();
    farm.aerial_application.as_mut().unwrap().actual_cost = 20_000;
    farm.aerial_application.as_mut().unwrap().gallons = 0.0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let farm =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_farm.as_mut().unwrap();
    farm.aerial_application.as_mut().unwrap().gallons = 100.0;
    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    let claim = data.fuel_tax_credit.as_mut().unwrap();
    claim.farm_activity = false;
    claim.fuels.gasoline_farm = Some(fuel_use(100_000, 20_000));
    // An unconfirmed statement, or farm-operator fuel without a farm
    // activity, leaves this K-1's fuel credit out with a notice.
    let r = supported(&draft);
    assert!(noted(&r, "WARN_PARTNER_FUEL_CREDIT_LEFT_OUT"), "{r:?}");
}

#[test]
fn custom_aerial_applicator_claims_farm_use_aviation_gasoline_on_schedule_c_and_k1() {
    let business_fuels = FederalFuelLines {
        aviation_gasoline_farm: Some(farm_aviation_gasoline(100_000, Some(100_000))),
        ..FederalFuelLines::default()
    };
    let partner_fuels = FederalFuelLines {
        aviation_gasoline_farm: Some(farm_aviation_gasoline(100_000, None)),
        ..FederalFuelLines::default()
    };
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(20_000), business_fuels, partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 5_800);
    for (source, amount) in [("business_2", 3_860), ("partnership_3", 1_930)] {
        assert_eq!(line_amount(&report, &alloc::format!("{source}_fuel_2b")), amount);
    }

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.aviation_gasoline_farm.as_mut().unwrap().applicator_was_ultimate_purchaser = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    let farm = fuels.aviation_gasoline_farm.as_mut().unwrap();
    farm.applicator_was_ultimate_purchaser = true;
    farm.distinct_gallons = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.aviation_gasoline_farm.as_mut().unwrap().distinct_gallons = true;
    fuels.gasoline_farm = Some(fuel_use(100_000, 20_000));
    unsupported(&draft, "Fuel tax credit");
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.gasoline_farm = None;
    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.diesel_farm = Some(fuel_use(100_000, 20_000));
    // An unconfirmed statement, or farm-operator fuel without a farm
    // activity, leaves this K-1's fuel credit out with a notice.
    let r = supported(&draft);
    assert!(noted(&r, "WARN_PARTNER_FUEL_CREDIT_LEFT_OUT"), "{r:?}");
}

#[test]
fn schedule_f_tips_are_capped_by_farm_net_income_and_other_farm_receipts() {
    let mut draft = complete(FederalStatus::Single);
    let mut farm = farm_business(
        vec![FarmReceipt::Raised { amount: 1_000_000 }, FarmReceipt::Other { amount: 1_000_000 }],
        19_000,
    );
    let FederalIncome::Business { data } = &mut farm else { panic!() };
    data.income_source = BusinessIncomeSource::Services;
    data.tips = Some(FederalBusinessTips { amount: 1_000_000, eligible: true });
    add(&mut draft, income(farm));
    let mut no_tips = draft.clone();
    let FederalIncome::Business { data } = &mut no_tips.income[1].income else { panic!() };
    data.tips = None;
    let baseline = supported(&no_tips);
    let result = supported(&draft);
    assert_eq!(value(&result, "business_2_qualified_tips"), 929);
    assert_eq!(value(&result, "tips_deduction"), 929);
    assert_eq!(value(&result, "agi"), value(&baseline, "agi"));
    assert_eq!(value(&result, "self_employment_tax"), value(&baseline, "self_employment_tax"));
    assert_eq!(restart(draft.clone()).draft, draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
    data.tips.as_mut().unwrap().amount += 1;
    unsupported(&draft, "Self-employed tips");
}

#[test]
fn farm_fuel_lines_use_farm_rates_and_one_rounded_payment() {
    let mut draft = complete(FederalStatus::Single);
    let mut farm = raised_farm(20_000, 5_000);
    let FederalIncome::Business { data } = &mut farm else { unreachable!() };
    data.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            gasoline_farm: Some(fuel_use(100_000, 30_000)),
            diesel_farm: Some(fuel_use(100_000, 40_000)),
            ..empty_fuel_lines()
        }
        .into(),
        expense_treatment: FederalFuelExpenseTreatment::FullCostDeducted,
        records_complete: true,
    });
    assert_eq!(data.receipts(), 2_000_000);
    add(&mut draft, income(farm));
    let report = supported(&draft);
    assert_eq!(line_amount(&report, "fuel_tax_credit"), 4260);
    assert_eq!(return_line(&report, "31"), 4300);
    assert_eq!(report.payments, Some(654_300));
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.emulsion_farm = Some(fuel_use(100_000, 20_000));
    let with_emulsion = supported(&draft);
    assert_eq!(with_emulsion.tax, report.tax);
    assert_eq!(return_line(&with_emulsion, "11a / 11b"), return_line(&report, "11a / 11b"));
    assert_eq!(line_amount(&with_emulsion, "business_2_fuel_14a"), 1_970);
    assert_eq!(return_line(&with_emulsion, "31"), 6_200);
    assert_eq!(with_emulsion.payments, Some(656_200));
    assert_eq!(estimate(&restart(draft.clone()).draft), with_emulsion);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 2_000_000,
        accounting: BusinessAccounting::Cash,
    };
    unsupported(&draft, "Fuel tax credit");
}

#[test]
fn farm_kerosene_fuel_uses_reach_form_4136_lines_4a_and_4b() {
    let mut draft = complete(FederalStatus::Single);
    let mut farm = raised_farm(20_000, 5_000);
    let FederalIncome::Business { data } = &mut farm else { unreachable!() };
    data.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines {
            kerosene_off_highway: Some(fuel_use(10_000, 1_000)),
            kerosene_farm: Some(fuel_use(20_000, 2_000)),
            ..empty_fuel_lines()
        }
        .into(),
        expense_treatment: FederalFuelExpenseTreatment::FullCostDeducted,
        records_complete: true,
    });
    assert_eq!(data.receipts(), 2_000_000); // Cash method: income when Form 4136 is filed.
    add(&mut draft, income(farm));
    let report = supported(&draft);
    assert_eq!(line_amount(&report, "business_2_fuel_4a"), 243);
    assert_eq!(line_amount(&report, "business_2_fuel_4b"), 486);
    assert_eq!(return_line(&report, "31"), 700);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 2_000_000,
        accounting: BusinessAccounting::Cash,
    };
    unsupported(&draft, "Fuel tax credit");
}

#[test]
fn farm_cash_sources_flow_once_to_schedule_one_se_qbi_and_return() {
    let mut d = empty_return(FederalStatus::Single);
    add(
        &mut d,
        income(farm_business(
            vec![
                FarmReceipt::Raised { amount: 2000000 },
                FarmReceipt::Resale { amount: 4000000, cost: 1000000 },
                FarmReceipt::CustomHire { amount: 500000 },
                FarmReceipt::Program { amount: 300000 },
                FarmReceipt::Other { amount: 200000 },
            ],
            10000,
        )),
    );
    let r = supported(&d);
    let prefix = alloc::format!("farm_{}", d.income[0].id);
    assert_eq!(value(&r, &alloc::format!("{prefix}_resale_proceeds")), 40000);
    assert_eq!(value(&r, &alloc::format!("{prefix}_resale_basis")), 10000);
    assert_eq!(value(&r, &alloc::format!("{prefix}_gross_income")), 60000);
    assert_eq!(value(&r, &alloc::format!("{prefix}_net_profit")), 50000);
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(value(&r, "agi"), 46467);
    assert_eq!(value(&r, "qbi_deduction"), 6073);
    assert_eq!(return_line(&r, "8"), 5000000);
    assert_eq!(return_line(&r, "10"), 353300);
    assert_eq!(r.tax, Some(973000));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn farm_and_schedule_c_share_owner_se_tax_but_keep_separate_profit_records() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(40000, 10000)));
    add(&mut d, income(cash_business(20000)));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(value(&r, "agi"), 46467);
    assert_eq!(return_line(&r, "8"), 5000000);
    assert_eq!(r.tax, Some(973000));
    assert_eq!(value(&r, &alloc::format!("farm_{}_net_profit", d.income[0].id)), 30000);
}

#[test]
fn farm_profit_is_schedule_se_line_1a_and_schedule_f_lines_are_tagged() {
    // Farm: $40,000 raised products (Schedule F line 2 and gross income line
    // 9) less $10,000 expenses (line 33) = $30,000 (line 34, Schedule SE line
    // 1a). Schedule C $20,000 is Schedule SE line 2; line 3 = $50,000; 4a =
    // $50,000 x 92.35% = $46,175; 12 = $46,175 x 15.3% = $7,065 = Schedule 2
    // line 4; 13 = $7,065 / 2 = $3,532.50, rounded to $3,533.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(40000, 10000)));
    add(&mut d, income(cash_business(20000)));
    let farm = d.income[0].id;
    let rows = crate::draft_return(&supported(&d));
    let of = |form: TaxForm| -> Vec<(&str, i64)> {
        rows.iter()
            .filter(|r| r.form_line.as_ref().is_some_and(|f| f.form == form))
            .map(|r| (r.form_line.as_ref().unwrap().line.as_str(), r.amount))
            .collect()
    };
    assert_eq!(
        of(TaxForm::ScheduleF),
        vec![
            ("2", 4_000_000),
            ("9", 4_000_000),
            ("33", 1_000_000),
            ("34", 3_000_000),
            ("B", 0),
            ("D", 0),
            ("F", 0),
            ("G", 0)
        ]
    );
    assert!(
        rows.iter()
            .filter(|r| r.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleF
                && matches!(f.line.as_str(), "B" | "D" | "F" | "G")))
            .all(|row| row.money.is_none()),
        "Header answers are text, never dollar amounts"
    );
    assert!(
        rows.iter()
            .any(|r| status_text(&r.label)
                == alloc::format!("Schedule F 34 — Farm {farm} net profit"))
    );
    assert_eq!(
        of(TaxForm::ScheduleSe),
        vec![
            ("1a", 3_000_000),
            ("2", 2_000_000),
            ("3", 5_000_000),
            ("4a", 4_617_500),
            ("4c", 4_617_500),
            ("12", 706_500),
            ("13", 353_300),
        ]
    );
    assert_eq!(of(TaxForm::Schedule2), vec![("4", 706_500), ("15", 706_500), ("21", 706_500)]);
}

#[test]
fn farm_joint_owners_use_separate_se_calculations_and_shared_return() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, income(raised_farm(40000, 10000)));
    let mut spouse = income(raised_farm(30000, 10000));
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(value(&r, "agi"), 46467);
    assert_eq!(value(&r, "qbi_deduction"), 2853);
    assert_eq!(r.tax, Some(820800));
}

#[test]
fn farm_employee_wages_use_remaining_social_security_base() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 18450000, 0)];
    add(&mut d, income(raised_farm(12000, 2000)));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 268);
    assert_eq!(value(&r, "agi"), 194366);
    assert_eq!(value(&r, "qbi_deduction"), 1973);
    assert_eq!(r.tax, Some(3517600));
}

#[test]
fn farm_health_and_sep_deductions_do_not_reduce_se_profit() {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = raised_farm(60000, 10000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    data.health_insurance = BusinessHealthInsurance::Plan {
        long_term_care: vec![],
        months: vec![BusinessHealthMonth {
            child_increase: 0,
            month: 1,
            premiums: 600000,
            employer_eligible: false,
            reimbursed: None,
        }],
        establishment: crate::HealthEstablishment::SoleProprietor,
    };
    data.retirement = BusinessRetirement::Sep { rate: 25.0, contribution: 800000 };
    add(&mut d, income(entry));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(value(&r, "agi"), 32467);
    assert_eq!(return_line(&r, "10"), 1753300);
    assert_eq!(value(&r, "qbi_deduction"), 3273);
    // Taxable $13,094: $1,240 plus 12% of $694, plus unchanged $7,065 SE tax.
    assert_eq!(r.tax, Some(838600));
}

#[test]
fn farm_unknown_elections_losses_and_duplicate_inventory_block_completed_return() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 1000000, 0)];
    add(&mut d, income(raised_farm(10000, 40000)));
    unsupported(&d, "Farm loss");
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.expenses.clear();
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    farm.specials.limitations = vec![3];
    let r = estimate(&d);
    assert_eq!(r.tax, None);
    assert!(
        r.issues.iter().any(|issue| issue.display.key == "ISSUE_FARM_METHOD_TRANSITION_SOURCE")
    );
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    farm.specials.limitations.clear();
    farm.accounting = FarmAccounting::Cash {
        receipts: vec![FarmReceipt::Resale { amount: 1000000, cost: 100000 }],
    };
    data.inventory = BusinessInventory::Cost {
        method: crate::InventoryMethod::Cost,
        opening: 0,
        purchases: 100000,
        personal: 0,
        labor: 0,
        materials: 0,
        other: 0,
        closing: 0,
        eligible: true,
    };
    unsupported(&d, "Farm inventory");
}

#[test]
fn farm_current_loss_offsets_wages_without_creating_a_fictitious_carryback() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(10000, 20000)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 50000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    assert_eq!(return_line(&r, "8"), -1000000);
    assert_eq!(r.tax, Some(382300));
}

#[test]
fn farm_equipment_uses_shared_bonus_and_section179_without_duplicate_deduction() {
    for asset in [bonus_equipment(10000, 100), elected_asset(10000, 4000, BonusElection::Apply)] {
        let mut d = empty_return(FederalStatus::Single);
        let mut entry = raised_farm(70000, 10000);
        let FederalIncome::Business { data } = &mut entry else { unreachable!() };
        data.assets = vec![asset];
        add(&mut d, income(entry));
        let r = supported(&d);
        assert_eq!(value(&r, &alloc::format!("farm_{}_net_profit", d.income[0].id)), 50000);
        assert_eq!(value(&r, "self_employment_tax"), 7065);
        assert_eq!(value(&r, "agi"), 46467);
        assert_eq!(value(&r, "qbi_deduction"), 6073);
        assert_eq!(r.tax, Some(973000));
    }
}

#[test]
fn farm_accrual_inventory_and_income_flow_through_the_full_return() {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = accrual_farm(60000, 10000, 30000, 20000, 20000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    let FarmAccounting::Accrual { custom_hire, program, other, .. } = &mut data.accounting else {
        unreachable!()
    };
    *custom_hire = 500000;
    *program = 300000;
    *other = 200000;
    add(&mut d, income(entry));
    let r = supported(&d);
    let id = d.income[0].id;
    for (name, amount) in [
        ("accrual_sales", 60000),
        ("opening_inventory", 10000),
        ("closing_inventory", 30000),
        ("product_purchases", 20000),
        // Custom hire 5,000 and other 2,000 (line 43); the 3,000 of program
        // income is Part III lines 39a/39b.
        ("other_accrual_income", 7000),
        ("program_payments", 3000),
        ("program_taxable", 3000),
        ("gross_income", 70000),
        ("expenses", 20000),
        ("net_profit", 50000),
    ] {
        assert_eq!(value(&r, &alloc::format!("farm_{id}_{name}")), amount);
    }
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(value(&r, "agi"), 46467);
    assert_eq!(value(&r, "qbi_deduction"), 6073);
    assert_eq!(return_line(&r, "8"), 5000000);
    assert_eq!(r.tax, Some(973000));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn farm_accrual_raised_inventory_can_exceed_opening_and_purchases() {
    let mut d = empty_return(FederalStatus::Single);
    // $40,000 raised closing inventory, no product purchases, $10,000 incurred
    // production expenses: $30,000 profit without requiring cash collections.
    add(&mut d, income(accrual_farm(0, 0, 40000, 0, 10000)));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 4239);
    assert_eq!(value(&r, "agi"), 27880);
    assert_eq!(value(&r, "qbi_deduction"), 2356);
    assert_eq!(return_line(&r, "8"), 3000000);
    assert_eq!(r.tax, Some(518200));
}

#[test]
fn farm_accrual_inventory_decrease_preserves_current_loss_and_nol_guard() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(accrual_farm(0, 20000, 0, 0, 0)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 40000);
    assert_eq!(return_line(&r, "8"), -2000000);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    assert_eq!(r.tax, Some(262300));
    d.income[0] = wage(1, Owner::Taxpayer, 1000000, 0);
    unsupported(&d, "Farm loss");
}

#[test]
fn farm_cash_and_accrual_activities_share_owner_limits_without_sharing_inventory() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(40000, 10000)));
    add(&mut d, income(accrual_farm(30000, 5000, 10000, 10000, 5000)));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(return_line(&r, "8"), 5000000);
    assert_eq!(r.tax, Some(973000));
}

#[test]
fn farm_accrual_unknown_accounting_and_invalid_source_amounts_do_not_finalize() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(accrual_farm(30000, 5000, 10000, 10000, 5000)));
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.specials.limitations = vec![3];
    let result = estimate(&d);
    assert_eq!(result.tax, None);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_FARM_METHOD_TRANSITION_SOURCE")
    );
}

#[test]
fn farm_nol_requires_explicit_valid_waiver_and_preserves_the_carryback_choice() {
    let mut d = farm_loss_return();
    d.screening.as_mut().unwrap().farm_nol = None;
    unsupported(&d, "Farm loss");
    // Keeping the carryback changes no 2026 line: the estimate computes with
    // the whole loss as carryforward and a note that the refund is not in it.
    d.screening.as_mut().unwrap().farm_nol = Some(FarmNolElection::Carryback);
    let kept = supported(&d);
    assert_eq!(value(&kept, "agi"), -20000);
    assert_eq!(value(&kept, "current_net_operating_loss"), 20000);
    // Carryover arithmetic: 2026 loss 20,000 less the amount 2024 and 2025
    // absorb (not computed) leaves at most 20,000 to carry forward.
    assert_eq!(value(&kept, "nol_carryforward"), 20000);
    assert_eq!(value(&kept, "amt_nol_carryforward"), 20000);
    assert_eq!(kept.tax, Some(0));
    assert!(noted(&kept, "NOTE_FARMING_LOSS_CARRYBACK_KEPT"));
    assert!(
        kept.review_notes.iter().any(|n| n.contains("carryback kept") && n.contains("Form 1045"))
    );
    assert!(!noted(&kept, "NOTE_FARMING_LOSS_CARRYBACK_WAIVER"));
    waive_farm_carryback(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), -20000);
    assert_eq!(value(&r, "current_net_operating_loss"), 20000);
    assert_eq!(value(&r, "nol_carryforward"), 20000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 20000);
    assert_eq!(r.tax, Some(0));
    assert_eq!(return_line(&r, "8"), -3000000);
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("waiver selected") && n.contains("not filed by this App"))
    );
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn farm_waiver_preserves_distinct_regular_and_amt_losses() {
    let mut d = farm_loss_return();
    waive_farm_carryback(&mut d);
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(native_exempt_interest(500000, 500000)),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "current_net_operating_loss"), 20000);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 15000);
    assert_eq!(value(&r, "nol_carryforward"), 20000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 15000);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn farm_waiver_keeps_nonfarm_and_excess_business_losses_once_in_total_carryforward() {
    let mut d = farm_loss_return();
    add(&mut d, income(cash_business(-10000)));
    waive_farm_carryback(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "nol_carryforward"), 30000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 30000);
    assert_eq!(r.tax, Some(0));
    d.income.clear();
    add(&mut d, income(raised_farm(0, 400000)));
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 144000);
    assert_eq!(value(&r, "current_net_operating_loss"), 256000);
    assert_eq!(value(&r, "nol_carryforward"), 400000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 400000);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn farm_joint_loss_waiver_is_return_wide_and_does_not_erase_other_spouse_se_tax() {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 1000000, 0)];
    add(&mut d, income(raised_farm(20000, 0)));
    let mut entry = income(raised_farm(0, 50000));
    entry.owner = crate::DocumentOwner::Spouses;
    add(&mut d, entry);
    waive_farm_carryback(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 2826);
    assert_eq!(value(&r, "agi"), -21413);
    assert_eq!(value(&r, "nol_carryforward"), 21413);
    assert_eq!(value(&r, "amt_nol_carryforward"), 21413);
    assert_eq!(r.tax, Some(282600));
}

#[test]
fn farm_choice_is_not_applied_when_only_form461_carryforward_exists() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 100000000, 0)];
    add(&mut d, income(raised_farm(0, 400000)));
    d.screening.as_mut().unwrap().farm_nol = Some(FarmNolElection::Carryback);
    let r = supported(&d);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    assert_eq!(value(&r, "nol_carryforward"), 144000);
    assert!(!r.review_notes.iter().any(|n| n.contains("waiver selected")));
    assert!(!noted(&r, "NOTE_FARMING_LOSS_CARRYBACK_KEPT"));
    let mut no_farm = complete(FederalStatus::Single);
    let baseline = supported(&no_farm);
    waive_farm_carryback(&mut no_farm);
    let after = supported(&no_farm);
    assert_eq!(after.tax, baseline.tax);
    assert!(!after.review_notes.iter().any(|n| n.contains("waiver selected")));
}

#[test]
fn farm_crop_deferral_applies_to_all_eligible_payments_and_keeps_withholding_current() {
    for (defer, taxable, carried, tax) in [(false, 30000, 0, 973000), (true, 10000, 20000, 518200)]
    {
        let d = crop_return(crop_records(
            vec![
                CropPayment::Physical { crop: "Grain".into(), amount: 1200000, year: 2026 },
                CropPayment::Physical { crop: "Grain".into(), amount: 800000, year: 2026 },
                CropPayment::Other { amount: 500000 },
            ],
            5000,
            defer,
            1000,
        ));
        let r = supported(&d);
        let id = d.income[0].id;
        let insurance = &crate::farm::sources(&d).next().unwrap().farm.insurance;
        assert_eq!(insurance.current_total(), 2_500_000);
        let claims = crate::related_forms::effective_claimants(&d);
        let resolved = crate::farm_insurance_paper::resolved(&d, id, &claims).unwrap();
        assert_eq!(resolved.current_total(), 2_500_000);
        assert_eq!(resolved.income(true), taxable * 100);
        assert_eq!(value(&r, &alloc::format!("farm_{id}_crop_insurance_before_deferral")), 25000);
        assert_eq!(value(&r, &alloc::format!("farm_{id}_crop_insurance_taxable")), taxable);
        assert_eq!(
            value(&r, &alloc::format!("farm_{id}_crop_insurance_deferred_next_year")),
            carried
        );
        assert_eq!(return_line(&r, "25d"), 100000);
        assert_eq!(r.tax, Some(tax));
        assert_eq!(r.balance, Some(tax - 100000));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn farm_crop_prior_damage_and_prior_elected_income_are_not_deferred_again() {
    let d = crop_return(crop_records(
        vec![
            CropPayment::Physical { crop: "Grain".into(), amount: 1000000, year: 2026 },
            CropPayment::Physical { crop: "Grain".into(), amount: 500000, year: 2025 },
            CropPayment::Other { amount: 500000 },
        ],
        10000,
        true,
        0,
    ));
    let r = supported(&d);
    let id = d.income[0].id;
    assert_eq!(value(&r, &alloc::format!("farm_{id}_crop_insurance_deferred_next_year")), 10000);
    assert_eq!(value(&r, &alloc::format!("farm_{id}_crop_insurance_taxable")), 20000);
    assert_eq!(value(&r, "agi"), 37174);
    assert_eq!(value(&r, "self_employment_tax"), 5652);
    assert_eq!(r.tax, Some(742900));
    let d = crop_return(crop_records(vec![], 30000, false, 0));
    assert_eq!(supported(&d).tax, Some(973000));
    let d = crop_return(crop_records(vec![], 30000, true, 0));
    let r = supported(&d);
    assert_eq!(r.tax, Some(973000));
    assert!(noted(&r, "WARN_CROP_PRACTICE_DEFERRAL_LEFT_OUT"));
}

#[test]
fn farm_crop_accrual_includes_insurance_and_leaves_out_ineligible_deferral() {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = accrual_farm(30000, 0, 0, 0, 10000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    install_crop_records(
        data,
        crop_records(
            vec![CropPayment::Physical { crop: "Grain".into(), amount: 3000000, year: 2026 }],
            0,
            false,
            0,
        ),
    );
    add(&mut d, income(entry));
    assert_eq!(supported(&d).tax, Some(973000));
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    install_crop_records(
        data,
        crop_records(
            vec![CropPayment::Physical { crop: "Grain".into(), amount: 3000000, year: 2026 }],
            0,
            true,
            0,
        ),
    );
    let r = supported(&d);
    assert_eq!(r.tax, Some(973000));
    assert!(noted(&r, "WARN_CROP_PRACTICE_DEFERRAL_LEFT_OUT"));
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    install_crop_records(data, crop_records(vec![], 30000, false, 0));
    unsupported(&d, "Crop insurance accounting");
}

#[test]
fn farm_crop_separate_businesses_keep_separate_deferral_elections() {
    let mut d = empty_return(FederalStatus::Single);
    let mut first = crop_farm(crop_records(
        vec![CropPayment::Physical { crop: "Grain".into(), amount: 1000000, year: 2026 }],
        0,
        true,
        0,
    ));
    let FederalIncome::Business { data } = &mut first else { unreachable!() };
    data.expenses[0].amount = 2000000; // First farm has $10,000 current profit after deferring $10,000.
    add(&mut d, income(first));
    add(
        &mut d,
        income(crop_farm(crop_records(
            vec![CropPayment::Physical { crop: "Grain".into(), amount: 2000000, year: 2026 }],
            0,
            false,
            0,
        ))),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "8"), 5000000);
    assert_eq!(r.tax, Some(973000));
    assert_eq!(
        value(&r, &alloc::format!("farm_{}_crop_insurance_deferred_next_year", d.income[0].id)),
        10000
    );
    assert_eq!(
        value(&r, &alloc::format!("farm_{}_crop_insurance_deferred_next_year", d.income[1].id)),
        0
    );
}

#[test]
fn farm_crop_invalid_or_unsubstantiated_deferral_never_silently_qualifies() {
    for payment in [
        CropPayment::Other { amount: 3000000 },
        CropPayment::Physical { crop: "Grain".into(), amount: 3000000, year: 2025 },
    ] {
        let r = supported(&crop_return(crop_records(vec![payment], 0, true, 0)));
        assert_eq!(r.tax, Some(973000));
        assert!(noted(&r, "WARN_CROP_PRACTICE_DEFERRAL_LEFT_OUT"));
    }
}
#[test]
fn farm_ccc_current_loan_election_changes_components_not_same_year_net_sale() {
    for method in [CccMethod::Loan, CccMethod::Income] {
        let elected = method == CccMethod::Income;
        let d = ccc_return(
            CccOrigin::Current,
            method,
            CccDisposition::Redeemed { year: 2026, paid: 400000, sale: Some(650000) },
            42300,
        );
        let r = ccc_fifty_thousand(&d);
        assert_eq!(ccc_value(&d, &r, "advances"), if elected { 5200 } else { 0 });
        assert_eq!(ccc_value(&d, &r, "market_gain"), 1200);
        assert_eq!(ccc_value(&d, &r, "taxable_market_gain"), if elected { 0 } else { 1200 });
        assert_eq!(ccc_value(&d, &r, "sales_net"), if elected { 2500 } else { 6500 });
        assert_eq!(ccc_value(&d, &r, "income"), 7700);
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn farm_ccc_prior_inclusion_and_prior_redemption_are_not_taxed_twice() {
    for (origin, method, year, profit, taxable, market) in [
        (CccOrigin::PriorIncluded, CccMethod::Income, 2026, 47500, 2500, 0),
        (CccOrigin::PriorExcluded, CccMethod::Loan, 2026, 42300, 7700, 1200),
        (CccOrigin::PriorExcluded, CccMethod::Income, 2026, 42300, 7700, 1200),
        (CccOrigin::PriorIncluded, CccMethod::Income, 2025, 47500, 2500, 0),
        (CccOrigin::PriorExcluded, CccMethod::Loan, 2025, 43500, 6500, 0),
    ] {
        let d = ccc_return(
            origin,
            method,
            CccDisposition::Redeemed { year, paid: 400000, sale: Some(650000) },
            profit,
        );
        let r = ccc_fifty_thousand(&d);
        assert_eq!(ccc_value(&d, &r, "income"), taxable);
        assert_eq!(ccc_value(&d, &r, "advances"), 0);
        assert_eq!(ccc_value(&d, &r, "taxable_market_gain"), market);
    }
}
#[test]
fn farm_ccc_outstanding_and_forfeited_lots_preserve_prior_election_basis() {
    for (origin, method, elected, prior) in [
        (CccOrigin::Current, CccMethod::Income, true, false),
        (CccOrigin::Current, CccMethod::Loan, false, false),
        (CccOrigin::PriorIncluded, CccMethod::Income, true, true),
        (CccOrigin::PriorExcluded, CccMethod::Loan, false, true),
    ] {
        for forfeited in [false, true] {
            let expected = if elected && !prior || forfeited && !elected { 5200 } else { 0 };
            let d = ccc_return(
                origin.clone(),
                method.clone(),
                if forfeited {
                    CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 0 }
                } else {
                    CccDisposition::Outstanding
                },
                50000 - expected,
            );
            let r = ccc_fifty_thousand(&d);
            assert_eq!(ccc_value(&d, &r, "income"), expected);
            assert_eq!(
                ccc_value(&d, &r, "retained_basis"),
                if elected && !forfeited { 5200 } else { 0 }
            );
        }
    }
}
#[test]
fn farm_ccc_unsold_basis_and_below_basis_sales_are_preserved() {
    for (sale, profit, net, basis) in [
        (None, 50000, 0, 4000),
        (Some(100000), 53000, -3000, 0),
        (Some(400000), 50000, 0, 0),
        (Some(0), 54000, -4000, 0),
    ] {
        let d = ccc_return(
            CccOrigin::PriorIncluded,
            CccMethod::Income,
            CccDisposition::Redeemed { year: 2026, paid: 400000, sale },
            profit,
        );
        let r = ccc_fifty_thousand(&d);
        assert_eq!(ccc_value(&d, &r, "sales_net"), net);
        assert_eq!(ccc_value(&d, &r, "retained_basis"), basis);
    }
}
#[test]
fn farm_ccc_unestablished_source_and_inconsistent_elections_block_finalization() {
    let d =
        ccc_return(CccOrigin::PriorIncluded, CccMethod::Income, CccDisposition::Outstanding, 50000);
    // Unknown ownership cannot discard known CCC records or pretend that
    // their required commodity income/basis is established.
    let mut special = d.clone();
    ccc_mut(&mut special).source.producer_owned_raised = false;
    let r = estimate(&special);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|issue| issue.display.key == "ISSUE_CCC_COMMODITY_BASIS_SOURCE"));
    assert!(
        matches!(&special.income[0].income, FederalIncome::Business { data } if matches!(&data.activity, BusinessActivity::Farming { data: farm } if farm.ccc.is_some()))
    );
    let mut bad = d.clone();
    ccc_mut(&mut bad).method = CccMethod::Loan;
    unsupported(&bad, "CCC loan election");
    let mut bad = d.clone();
    ccc_mut(&mut bad).lots[0].disposition =
        CccDisposition::Redeemed { year: 2026, paid: 530000, sale: None };
    unsupported(&bad, "CCC loan settlement");
    let mut bad = d.clone();
    ccc_mut(&mut bad).lots[0].origin = CccOrigin::Current;
    ccc_mut(&mut bad).lots[0].disposition =
        CccDisposition::Redeemed { year: 2025, paid: 400000, sale: None };
    unsupported(&bad, "CCC loan settlement");
    let mut bad = d.clone();
    let other = ccc_return(CccOrigin::Current, CccMethod::Loan, CccDisposition::Outstanding, 0);
    // Another farm's loans are new records of this return: they take their own
    // identities, not the other draft's.
    let mut entry = other.income[0].clone();
    for id in crate::farm::ccc_lot_ids_mut(&mut entry.income) {
        *id = 0;
    }
    add(&mut bad, entry);
    unsupported(&bad, "CCC loan election");
    let mut bad = d.clone();
    let FederalIncome::Business { data } = &mut bad.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.accounting = FarmAccounting::Accrual {
        sales: 6000000,
        opening: 0,
        closing: 0,
        purchases: 0,
        custom_hire: 0,
        program: 0,
        other: 0,
    };
    unsupported(&bad, "CCC accrual inventory");
}
#[test]
fn farm_crp_excludes_only_eligible_payments_from_se_not_income_or_qbi() {
    for (benefits, se, agi, qbi, tax) in
        [(false, 7065, 46467, 6073, 973000), (true, 4239, 47880, 6356, 704200)]
    {
        let d = crp_return(30000, &[(20000, benefits)]);
        let r = supported(&d);
        assert_eq!(return_line(&r, "8"), 5000000);
        assert_eq!(value(&r, "self_employment_tax"), se);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "qbi_deduction"), qbi);
        assert_eq!(r.tax, Some(tax));
        assert_eq!(restart(d.clone()).draft, d);
    }
    let d = crp_return(20000, &[(10000, false), (20000, true)]);
    assert_eq!(supported(&d).tax, Some(704200));
}
#[test]
fn farm_crp_exclusion_keeps_expenses_and_zero_se_income() {
    let d = crp_return(0, &[(50000, true)]);
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "agi"), 50000);
    assert_eq!(r.tax, Some(300700));
    let d = crp_return(-5000, &[(50000, true)]);
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "agi"), 45000);
    assert_eq!(r.tax, Some(252700));
}
#[test]
fn farm_crp_health_and_sep_limits_use_se_subject_earnings() {
    let mut d = crp_return(30000, &[(20000, true)]);
    let id = d.income[0].id;
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.health_insurance = BusinessHealthInsurance::Plan {
        long_term_care: vec![],
        months: vec![BusinessHealthMonth {
            child_increase: 0,
            month: 1,
            premiums: 600000,
            employer_eligible: false,
            reimbursed: None,
        }],
        establishment: crate::HealthEstablishment::SoleProprietor,
    };
    data.retirement = BusinessRetirement::Sep { rate: 25.0, contribution: 500000 };
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("business_{id}_sep_limit")), 5576);
    assert_eq!(value(&r, "agi"), 36880);
    assert_eq!(r.tax, Some(598600));
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.retirement = BusinessRetirement::Sep { rate: 25.0, contribution: 600000 };
    unsupported(&d, "Business retirement");
}
#[test]
fn farm_crp_withholding_remains_current() {
    let mut d = crp_return(30000, &[(20000, true)]);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.crp[0].withholding = 100000;
    let r = supported(&d);
    assert_eq!(return_line(&r, "25d"), 100000);
    assert_eq!(r.balance, Some(604200));
}

#[test]
fn farm_crp_exclusion_limits_health_deduction_and_ira_compensation() {
    let mut d = crp_return(30000, &[(20000, true)]);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.health_insurance = BusinessHealthInsurance::Plan {
        long_term_care: vec![],
        months: vec![BusinessHealthMonth {
            child_increase: 0,
            month: 1,
            premiums: 3000000,
            employer_eligible: false,
            reimbursed: None,
        }],
        establishment: crate::HealthEstablishment::SoleProprietor,
    };
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 20000);
    assert_eq!(r.tax, Some(455200));
    let mut d = crp_return(0, &[(50000, true)]);
    d.adjustments.as_mut().unwrap().iras = ira_facts(10000, 0, Answer::No);
    let r = crate::federal_rules::estimate(&d);
    assert!(r.tax.is_none());
    assert!(r.issues.iter().any(|i| i.message.contains("compensation")));
}
#[test]
fn farm_crp_one_spouses_exclusion_does_not_remove_other_spouses_se_earnings() {
    let mut d = empty_return(FederalStatus::Joint);
    let a = crp_return(0, &[(20000, true)]);
    add(&mut d, a.income[0].clone());
    let mut b = income(raised_farm(40000, 10000));
    b.owner = crate::DocumentOwner::Spouses;
    add(&mut d, b);
    let r = supported(&d);
    assert_eq!(return_line(&r, "8"), 5000000);
    assert_eq!(value(&r, "self_employment_tax"), 4239);
    assert_eq!(value(&r, "agi"), 47880);
    // Joint standard deduction $32,200; QBI $3,136; ordinary tax $1,254.
    assert_eq!(r.tax, Some(549200));
}
#[test]
fn optional_farm_se_changes_tax_not_taxable_farm_profit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(6000, 5000)));
    elect_se(&mut d, SeMethod::Farm);
    let r = supported(&d);
    assert_eq!(value(&r, "optional_se_taxpayer_farm"), 4000);
    assert_eq!(return_line(&r, "8"), 100000);
    assert_eq!(value(&r, "self_employment_tax"), 612);
    assert_eq!(value(&r, "agi"), 60694);
    assert_eq!(r.tax, Some(570100));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn optional_farm_se_preserves_actual_loss_and_can_eliminate_small_se_tax() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(12000, 14000)));
    elect_se(&mut d, SeMethod::Farm);
    let r = supported(&d);
    assert_eq!(return_line(&r, "8"), -200000);
    assert_eq!(value(&r, "self_employment_tax"), 1157);
    assert_eq!(value(&r, "agi"), 57421);
    assert_eq!(r.tax, Some(586800));
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(540, 80)));
    elect_se(&mut d, SeMethod::Farm);
    let r = supported(&d);
    assert_eq!(value(&r, "optional_se_taxpayer_net"), 360);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(r.tax, Some(506500));
}
#[test]
fn optional_nonfarm_and_combined_methods_share_the_2026_cap() {
    let mut d = complete(FederalStatus::Single);
    let mut b = cash_business(1000);
    let FederalIncome::Business { data } = &mut b else { unreachable!() };
    data.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 1200000,
        accounting: BusinessAccounting::Cash,
    };
    data.expenses = vec![ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Operating expenses".into() },
        1100000,
    )];
    add(&mut d, income(b));
    elect_se(&mut d, SeMethod::Nonfarm { history: se_history() });
    let r = supported(&d);
    assert_eq!(value(&r, "optional_se_taxpayer_nonfarm"), 7560);
    assert_eq!(r.tax, Some(621600));
    add(&mut d, income(raised_farm(6000, 5000)));
    elect_se(&mut d, SeMethod::Both { history: se_history() });
    let r = supported(&d);
    assert_eq!(value(&r, "optional_se_taxpayer_farm"), 4000);
    assert_eq!(value(&r, "optional_se_taxpayer_nonfarm"), 3560);
    assert_eq!(value(&r, "self_employment_tax"), 1157);
    // Combined eligible QBI $1,421 activates the 2026 $400 minimum deduction.
    assert_eq!(value(&r, "qbi_deduction"), 400);
    assert_eq!(r.tax, Some(630000));
}
#[test]
fn optional_se_health_can_deduct_premiums_despite_actual_farm_loss() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(12000, 14000)));
    elect_se(&mut d, SeMethod::Farm);
    se_health(&mut d, 1, 6000);
    let r = supported(&d);
    assert_eq!(return_line(&r, "8"), -200000);
    assert_eq!(value(&r, "agi"), 51421);
    assert_eq!(r.tax, Some(514800));
}

#[test]
fn farm_averaging_recomputes_base_year_rate_schedules_not_filing_tables() {
    use crate::income_tax::Rates;
    let rates = Rates::for_year(2023, FederalStatus::Single).unwrap();
    let table = Rates::with_method(2023, FederalStatus::Single, true).unwrap();
    assert_eq!(rates.schedule_d(30000, 0, 0, 0, false), 3380);
    assert_eq!(table.schedule_d(30000, 0, 0, 0, false), 3383);
    assert_eq!(rates.schedule_d(100000, 50000, 0, 0, false), 13808);
    assert_eq!(table.schedule_d(100000, 50000, 0, 0, false), 13813);
    assert!(Rates::with_method(2026, FederalStatus::Single, true).is_some());
    assert_eq!(
        Rates::for_year(2026, FederalStatus::Single).unwrap().schedule_d(30000, 0, 0, 0, false),
        3352,
    );
    let d = averaging_return(60000);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 152935);
    assert_eq!(value(&r, "qbi_deduction"), 18587);
    assert_eq!(value(&r, "taxable_income"), 118248);
    assert_eq!(value(&r, "farm_averaging_eligible_income"), 74348);
    assert_eq!(value(&r, "farm_averaging_each_year"), 20000);
    assert_eq!(value(&r, "farm_averaging_2023_recomputed_tax"), 3380);
    assert_eq!(value(&r, "farm_averaging_2024_recomputed_tax"), 3368);
    assert_eq!(value(&r, "farm_averaging_2025_recomputed_tax"), 3362);
    assert_eq!(value(&r, "farm_averaging_current_tax"), 7522);
    // 7,522 + 3,380 + 3,368 + 3,362 - 3 * 1,003 = 14,623.
    assert_eq!(value(&r, "regular_income_tax"), 14623);
    assert_eq!(r.tax, Some(2875300));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    let mut ordinary = d.clone();
    ordinary.screening.as_mut().unwrap().farm_averaging = None;
    let baseline = supported(&ordinary);
    assert_eq!(baseline.tax, Some(3510800));
    for id in ["agi", "qbi_deduction", "self_employment_tax", "taxable_income"] {
        assert_eq!(value(&r, id), value(&baseline, id));
    }
}

#[test]
fn farm_averaging_does_not_create_offsetting_amt() {
    let mut d = averaging_return(60000);
    d.deductions.as_mut().unwrap().itemize_override = None;
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Averaging AMT test", 1000.0, 1000, 4000)];
    let r = supported(&d);
    // TMT $19,304 exceeds averaged $14,628 but not unaveraged $20,978.
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(r.tax, Some(2875300));
    d.screening.as_mut().unwrap().iso_exercises[0].form.market_price = 7000;
    let r = supported(&d);
    // 2026-11-02: automatic itemizing keeps TMT27,104 and uses the
    // unaveraged regular24,842 for AMT, giving2,262.
    assert_eq!(value(&r, "alternative_minimum_tax"), 2262);
    assert_eq!(r.tax, Some(3455700));
    let mut ordinary = d.clone();
    ordinary.screening.as_mut().unwrap().farm_averaging = None;
    // Without averaging both routes have the same total tax. The existing
    // tie preference keeps the larger standard deduction and its6,126 AMT.
    assert_eq!(value(&supported(&ordinary), "alternative_minimum_tax"), 6126);
}

#[test]
fn farm_averaging_negative_base_income_uses_retained_loss_components() {
    let mut d = averaging_return(60000);
    let b = &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years[0];
    b.source = AveragingSource::Return {
        agi: -1750000,
        deductions: 1240000,
        tax: 0,
        other_tax: vec![],
        loss: Some(AveragingLoss {
            capital_deduction: 300000,
            capital_loss: 700000,
            capital_carry: 700000,
            nol: AveragingNol::Current { amount: 1450000, prior_deduction: 0 },
        }),
    };
    let r = supported(&d);
    // IRS negative-income worksheet: -29,900 + 3,000 + 14,500 = -12,400.
    assert_eq!(value(&r, "farm_averaging_2023_base_income"), -12400);
    assert_eq!(value(&r, "farm_averaging_2023_allocated_income"), 7600);
    assert_eq!(value(&r, "farm_averaging_2023_recomputed_tax"), 760);
    assert_eq!(value(&r, "regular_income_tax"), 13006);
    assert_eq!(r.tax, Some(2713600));
    // An older NOL deducted in the original AGI is first removed when the base
    // year generated its own NOL; it must not create a second negative base.
    if let AveragingSource::Return { agi, loss: Some(loss), .. } =
        &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years[0].source
    {
        *agi -= 500000;
        loss.nol = AveragingNol::Current { amount: 1450000, prior_deduction: 500000 };
    }
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn farm_averaging_prior_elections_use_latest_retained_lines() {
    let mut d = averaging_return(18000);
    for b in &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years {
        b.source = AveragingSource::ScheduleJ {
            year: 2025,
            taxable: if b.year == 2023 { -620000 } else { 1000000 },
            tax: match b.year {
                2023 => 0,
                2024 => 100000,
                _ => 100300,
            },
        };
    }
    let r = supported(&d);
    assert_eq!(value(&r, "farm_averaging_2023_base_income"), -6200);
    assert_eq!(value(&r, "farm_averaging_2023_allocated_income"), 0);
    assert_eq!(value(&r, "farm_averaging_2024_allocated_income"), 16000);
    assert_eq!(value(&r, "regular_income_tax"), 18134);
    assert_eq!(r.tax, Some(3226400));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    // A newer Schedule J adjusts all earlier included base years, not just one.
    if let AveragingSource::ScheduleJ { year, .. } =
        &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years[0].source
    {
        *year = 2024;
    }
    unsupported(&d, "Farm averaging");
}

#[test]
fn farm_averaging_prior_dividends_and_non_section_one_taxes_are_distinct() {
    let mut d = averaging_return(60000);
    for b in &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years {
        b.dividends = 1000000;
        if let AveragingSource::Return { tax, other_tax, .. } = &mut b.source {
            *tax = 25000;
            *other_tax = vec![AveragingOtherTax { amount: 25000 }];
        }
    }
    let r = supported(&d);
    assert_eq!(value(&r, "farm_averaging_2023_base_tax"), 0);
    assert_eq!(value(&r, "farm_averaging_2023_recomputed_tax"), 2180);
    assert_eq!(value(&r, "farm_averaging_2024_recomputed_tax"), 2168);
    assert_eq!(value(&r, "farm_averaging_2025_recomputed_tax"), 2162);
    assert_eq!(value(&r, "regular_income_tax"), 14032);
    assert_eq!(r.tax, Some(2816200));
}

#[test]
fn farm_averaging_eligible_income_includes_owner_deductions_and_qbi() {
    let mut d = averaging_return(60000);
    se_health(&mut d, 1, 5000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.retirement = BusinessRetirement::Sep { contribution: 1000000, rate: 25.0 };
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 15587);
    assert_eq!(value(&r, "farm_averaging_eligible_income"), 62348);
    assert_eq!(r.tax, Some(2653000));
    d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().elected = 6234900;
    unsupported(&d, "Farm averaging");
}

#[test]
fn farm_averaging_incomplete_or_unallocated_sources_do_not_finalize() {
    let d = averaging_return(60000);
    let mut missing = d.clone();
    missing.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().selection.history_scope =
        FarmAveragingHistoryScope::ShortYearOrForeign;
    // A special averaging situation is left out: regular tax applies.
    let r = supported(&missing);
    assert!(noted(&r, "WARN_FARM_AVERAGING_HISTORY_LEFT_OUT"), "{r:?}");
    let mut regular = d.clone();
    regular.screening.as_mut().unwrap().farm_averaging = None;
    assert_eq!(r.tax, supported(&regular).tax);
    let mut duplicate = d.clone();
    duplicate.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years[0].year = 2024;
    unsupported(&duplicate, "Farm averaging");
    let mut loss_missing = d;
    if let AveragingSource::Return { agi, .. } =
        &mut loss_missing.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years[0]
            .source
    {
        *agi = 0;
    }
    unsupported(&loss_missing, "Farm averaging");
}

#[test]
fn farm_averaging_unused_nol_does_not_create_artificial_negative_base() {
    let mut d = averaging_return(60000);
    let b = &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years[0];
    b.source = AveragingSource::Return {
        agi: -815000,
        deductions: 0,
        tax: 0,
        other_tax: vec![],
        loss: Some(AveragingLoss {
            capital_deduction: 300000,
            capital_loss: 700000,
            capital_carry: 700000,
            nol: AveragingNol::Unused { amount: 515000 },
        }),
    };
    let r = supported(&d);
    assert_eq!(value(&r, "farm_averaging_2023_base_income"), 0);
    assert_eq!(value(&r, "farm_averaging_2023_recomputed_tax"), 2180);
    assert_eq!(value(&r, "regular_income_tax"), 14426);
    assert_eq!(r.tax, Some(2855600));
}

#[test]
fn farm_averaging_warns_but_does_not_replace_the_taxpayers_election() {
    let mut d = averaging_return(60000);
    for (b, tax) in d
        .screening
        .as_mut()
        .unwrap()
        .farm_averaging
        .as_mut()
        .unwrap()
        .years
        .iter_mut()
        .zip([146895, 145375, 144547])
    {
        b.source = AveragingSource::Return {
            agi: 51500000,
            deductions: 1500000,
            tax: tax * 100,
            other_tax: vec![],
            loss: None,
        };
    }
    let r = supported(&d);
    // All three added $20,000 portions are at 35%: $21,000 extra base-year tax.
    assert_eq!(value(&r, "regular_income_tax"), 28522);
    assert_eq!(r.tax, Some(4265200));
    assert!(r.review_notes.iter().any(|v| v.contains("increases regular income tax")));
    d.screening.as_mut().unwrap().farm_averaging = None;
    assert_eq!(supported(&d).tax, Some(3510800));
}

#[test]
fn farm_averaging_base_foreign_stacks_retained_exclusions_after_related_deductions() {
    let mut d = averaging_return(60000);
    for b in &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years {
        b.foreign = Some(averaging_foreign(60000, 10000, 20000));
        if let AveragingSource::Return { tax, .. } = &mut b.source {
            *tax = 220000;
        }
    }
    let r = supported(&d);
    for (year, expected) in [(2023, 6600), (2024, 6600), (2025, 6600)] {
        assert_eq!(value(&r, &alloc::format!("farm_averaging_{year}_foreign_stacking")), 50000);
        assert_eq!(value(&r, &alloc::format!("farm_averaging_{year}_recomputed_tax")), expected);
    }
    // $50,000 through $80,000 stays in the 22% bracket in all three years.
    // 7,527 + 3*(6,600-2,200) = 20,727.
    assert_eq!(value(&r, "regular_income_tax"), 20722);
    assert_eq!(r.tax, Some(3485200));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    assert_eq!(value(&r, "agi"), 152935); // No earlier exclusion deducted in 2026.
}

#[test]
fn farm_averaging_base_foreign_gain_excess_reduces_gains_before_dividends() {
    let mut d = averaging_return(60000);
    for b in &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years {
        b.foreign = Some(averaging_foreign(100000, 0, 0));
        b.dividends = 2000000;
        b.capital = AveragingCapital {
            long_term: 6000000,
            net: 6000000,
            collectibles: 2000000,
            unrecaptured: 3000000,
        };
        if let AveragingSource::Return { tax, .. } = &mut b.source {
            *tax = 150000;
        }
    }
    let r = supported(&d);
    // At $30,000 allocated income, $80,000 preferences have $50,000 excess.
    // Net gain becomes $10,000; dividends stay $20,000. The 28% pool absorbs
    // $20,000 of excess, then the $30,000 25% pool is exhausted. All $30,000
    // remaining preferences are at 15%, stacked above $100,000 ordinary income.
    for year in 2023..=2025 {
        for (suffix, expected) in [
            ("foreign_gain_excess", 50000),
            ("foreign_net_gain", 10000),
            ("foreign_dividends", 20000),
            ("foreign_line18", 0),
            ("foreign_line19", 0),
            ("recomputed_tax", 4500),
        ] {
            assert_eq!(value(&r, &alloc::format!("farm_averaging_{year}_{suffix}")), expected);
        }
    }
    assert_eq!(value(&r, "regular_income_tax"), 16522);
    assert_eq!(r.tax, Some(3065200));
}

#[test]
fn farm_averaging_base_foreign_excess_uses_28_percent_pool_before_25_percent_pool() {
    let mut d = averaging_return(60000);
    let b = &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years[0];
    b.foreign = Some(averaging_foreign(200000, 0, 0));
    b.dividends = 500000;
    b.capital = AveragingCapital {
        long_term: 3500000,
        net: 3500000,
        collectibles: 1500000,
        unrecaptured: 2000000,
    };
    if let AveragingSource::Return { tax, .. } = &mut b.source {
        *tax = 200000;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "farm_averaging_2023_foreign_gain_excess"), 10000);
    assert_eq!(value(&r, "farm_averaging_2023_foreign_line18"), 5000);
    assert_eq!(value(&r, "farm_averaging_2023_foreign_line19"), 20000);
    // $20,000*25% + $5,000*28% + $5,000*15% = $7,150.
    assert_eq!(value(&r, "farm_averaging_2023_recomputed_tax"), 7150);
    assert_eq!(value(&r, "regular_income_tax"), 17396);
    assert_eq!(r.tax, Some(3152600));

    // Now only $10,000 of allocated taxable income remains: $30,000 excess.
    // $15,000 exhausts collectibles and $15,000 reduces the 25% pool to $5,000.
    let election = d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap();
    election.elected = 1800000;
    if let AveragingSource::Return { agi, tax, .. } = &mut election.years[0].source {
        *agi = 1900000;
        *tax = 60000;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "farm_averaging_2023_foreign_gain_excess"), 30000);
    assert_eq!(value(&r, "farm_averaging_2023_foreign_line18"), 0);
    assert_eq!(value(&r, "farm_averaging_2023_foreign_line19"), 5000);
    assert_eq!(value(&r, "farm_averaging_2023_recomputed_tax"), 2000);
    assert_eq!(value(&r, "regular_income_tax"), 19531);
    assert_eq!(r.tax, Some(3366100));
}

#[test]
fn farm_averaging_base_foreign_zero_stacking_and_zero_income_follow_separate_rules() {
    let mut d = averaging_return(60000);
    for b in &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years {
        b.foreign = Some(averaging_foreign(50000, 0, 70000));
    }
    assert_eq!(supported(&d).tax, Some(2875300));
    let election = d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap();
    for b in &mut election.years {
        b.foreign = None;
        b.source = AveragingSource::ScheduleJ {
            year: 2025,
            taxable: if b.year == 2023 { -2000000 } else { 1000000 },
            tax: match b.year {
                2023 => 0,
                2024 => 100000,
                _ => 100300,
            },
        };
    }
    election.years[0].foreign = Some(averaging_foreign(100000, 0, 0));
    election.years[0].dividends = 1000000;
    let r = supported(&d);
    assert_eq!(value(&r, "farm_averaging_2023_allocated_income"), 0);
    assert_eq!(value(&r, "farm_averaging_2023_recomputed_tax"), 0);
    assert_eq!(r.tax, Some(2637900));
}

#[test]
fn farm_averaging_base_foreign_rejects_duplicate_or_wrong_filer_forms() {
    let mut d = averaging_return(60000);
    let b = &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years[0];
    let mut f = averaging_foreign(50000, 0, 0);
    f.forms.push(f.forms[0].clone());
    b.foreign = Some(f);
    unsupported(&d, "Farm averaging");
    let f = d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years[0]
        .foreign
        .as_mut()
        .unwrap();
    f.forms.pop();
    f.forms[0].owner = Owner::Spouse;
    unsupported(&d, "Farm averaging");
}

#[test]
fn farm_averaging_single_filer_uses_actual_joint_base_year_rates() {
    let mut d = averaging_return(60000);
    for b in &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years {
        b.status = FederalStatus::Joint;
    }
    let r = supported(&d);
    // Joint schedules: 2,200 + 8,000*.12; 2,320 + 6,800*.12;
    // 2,385 + 6,150*.12. Retained $1,003 tax is subtracted each year.
    for (year, tax) in [(2023, 3160), (2024, 3136), (2025, 3123)] {
        assert_eq!(value(&r, &alloc::format!("farm_averaging_{year}_recomputed_tax")), tax);
    }
    assert_eq!(value(&r, "regular_income_tax"), 13932);
    assert_eq!(r.tax, Some(2806200));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn farm_averaging_joint_sole_farmer_uses_own_single_base_years() {
    let mut d = averaging_return(60000);
    let joint = complete(FederalStatus::Joint);
    d.setup = joint.setup;
    d.people = joint.people;
    // AGI152935 - standard32200 - QBI18587 = taxable102148.
    // Residual42148: 2480 + (42148-24800)*.12 =4562.
    // Add the three historical increases: 2377 +2365 +2359.
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_income"), 102148);
    assert_eq!(value(&r, "regular_income_tax"), 11660);
    assert_eq!(r.tax, Some(2579000));
    // The sole farmer may be the current spouse; retained history follows
    // that farmer rather than being silently taken from the other filer.
    d.income
        .iter_mut()
        .find(|e| matches!(e.income, FederalIncome::Business { .. }))
        .unwrap()
        .owner = crate::DocumentOwner::Spouses;
    assert_eq!(supported(&d).tax, r.tax);
    let extra = income(raised_farm(1000, 0));
    add(&mut d, extra);
    let omitted = supported(&d);
    assert!(noted(&omitted, "WARN_FARM_AVERAGING_ALLOCATION_LEFT_OUT"));
    let mut regular = d.clone();
    regular.screening.as_mut().unwrap().farm_averaging = None;
    assert_eq!(omitted.tax, supported(&regular).tax);
}

#[test]
fn farm_averaging_retained_investment_election_uses_form_lines() {
    let mut d = averaging_return(60000);
    for b in &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years {
        b.dividends = 1000000;
        b.capital =
            AveragingCapital { long_term: 1000000, net: 1000000, collectibles: 0, unrecaptured: 0 };
        b.investment = Some(AveragingInvestment { elected: 1200000, capital: 1000000 });
        if let AveragingSource::Return { tax, .. } = &mut b.source {
            *tax = 20000;
        }
    }
    let r = supported(&d);
    for (year, tax) in [(2023, 2420), (2024, 2408), (2025, 2402)] {
        assert_eq!(value(&r, &alloc::format!("farm_averaging_{year}_recomputed_tax")), tax);
        assert_eq!(value(&r, &alloc::format!("farm_averaging_{year}_investment_capital")), 10000);
        assert_eq!(value(&r, &alloc::format!("farm_averaging_{year}_investment_dividends")), 2000);
    }
    // $22,000 ordinary, $8,000 dividends within the 0% band in each base year.
    assert_eq!(value(&r, "regular_income_tax"), 14152);
    assert_eq!(r.tax, Some(2828200));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    for b in &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years {
        b.investment.as_mut().unwrap().capital = 500000; // Retained dotted-line allocation.
    }
    let allocated = supported(&d);
    assert_eq!(allocated.tax, r.tax);
    assert_eq!(value(&allocated, "farm_averaging_2023_investment_dividends"), 7000);
}

#[test]
fn farm_averaging_investment_election_precedes_foreign_excess_and_special_caps() {
    let mut d = averaging_return(60000);
    for b in &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years {
        b.foreign = Some(averaging_foreign(300000, 0, 0));
        b.dividends = 2000000;
        b.capital = AveragingCapital {
            long_term: 6000000,
            net: 6000000,
            collectibles: 2000000,
            unrecaptured: 3000000,
        };
        b.investment = Some(AveragingInvestment { elected: 2500000, capital: 2000000 });
        if let AveragingSource::Return { tax, .. } = &mut b.source {
            *tax = 150000;
        }
    }
    let r = supported(&d);
    // Preference after election55k, then excess25k: net gain15k/dividends15k.
    // D18=0, D19=25k; capped25%-gain15k. Tax=3750+2250=6000.
    for year in 2023..=2025 {
        assert_eq!(value(&r, &alloc::format!("farm_averaging_{year}_foreign_gain_excess")), 25000);
        assert_eq!(value(&r, &alloc::format!("farm_averaging_{year}_recomputed_tax")), 6000);
    }
    assert_eq!(value(&r, "regular_income_tax"), 21022);
    assert_eq!(r.tax, Some(3515200));
}

#[test]
fn farm_averaging_investment_record_cannot_elect_missing_dividends() {
    let mut d = averaging_return(60000);
    d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years[0].investment =
        Some(AveragingInvestment { elected: 10000, capital: 0 });
    unsupported(&d, "Farm averaging");
}

#[test]
fn farm_averaging_current_foreign_wages_preserve_stacking_and_amt_comparison() {
    let mut d = averaging_return(60000);
    add(&mut d, income(foreign_pay(100000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    // Tax on158248 minus tax on100000 =13866; historical increments7101.
    assert_eq!(value(&r, "agi"), 152935);
    assert_eq!(value(&r, "farm_averaging_current_tax"), 13866);
    assert_eq!(value(&r, "regular_income_tax"), 20967);
    assert_eq!(r.tax, Some(3509700));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    // Averaging never changes the AMT comparison or creates offsetting AMT.
    let mut plain = d.clone();
    plain.screening.as_mut().unwrap().farm_averaging = None;
    assert_eq!(
        value(&supported(&plain), "alternative_minimum_tax"),
        value(&r, "alternative_minimum_tax")
    );
}

#[test]
fn farm_averaging_recomputes_foreign_excess_at_reduced_current_income() {
    let mut d = averaging_return(75000);
    d.income.retain(|e| !matches!(e.income, FederalIncome::Wage { .. }));
    add(&mut d, income(foreign_pay(100000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    add(&mut d, income(qualified_dividend(70000)));
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 15367);
    assert_eq!(value(&r, "taxable_income"), 131468);
    // Reduced56468 all at15%=8470, plus base35k taxes3980/3968/3962 less1003 each.
    assert_eq!(value(&r, "farm_averaging_current_tax"), 8470);
    assert_eq!(value(&r, "regular_income_tax"), 17371);
    assert_eq!(r.tax, Some(3389300)); // Includes SE14130 and NIIT2392.
}

#[test]
fn farm_averaging_current_foreign_special_gains_use_reduced_income() {
    let mut d = averaging_return(60000);
    add(&mut d, income(foreign_pay(100000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    add(&mut d, income(foreign_special_distribution(10000, 5000, 3000, 10000)));
    let r = supported(&d);
    // Reduced78248 plus exclusion100k. Normal preference12k at15%; the8k
    // special gain remains below24% ceiling. Remaining tax17586 + base7101.
    assert_eq!(value(&r, "farm_averaging_current_tax"), 17586);
    assert_eq!(value(&r, "regular_income_tax"), 24687);
    assert_eq!(r.tax, Some(3957700)); // SE14130 + NIIT760.
}

#[test]
fn farm_averaging_statement_foreign_credit_preserves_unaveraged_amt() {
    let mut d = averaging_return(60000);
    d.deductions.as_mut().unwrap().itemize_override = None;
    averaging_statement_credit(&mut d);
    let r = supported(&d);
    // Additional1k interest: reduced current tax7747 + base increments7101.
    assert_eq!(value(&r, "regular_income_tax"), 14843);
    assert_eq!(value(&r, "foreign_tax_credit"), 300);
    assert_eq!(r.tax, Some(2867300));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Foreign-credit averaging AMT", 1000.0, 1000, 7000)];
    let r = supported(&d);
    // 2026-11-02: automatic itemizing uses unaveraged25,082 against
    // TMT27,364. The same300 credit on each side leaves AMT2,282.
    assert_eq!(value(&r, "alternative_minimum_tax"), 2282);
    assert_eq!(r.tax, Some(3449700));
    let mut no_credit = d.clone();
    no_credit.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::No;
    assert_eq!(value(&supported(&no_credit), "alternative_minimum_tax"), 2282);
    raise_payee_statement_tax(&mut d, 1);
    unsupported(&d, "Foreign tax credit");
}

#[test]
fn farm_averaging_statement_credit_is_limited_by_elected_regular_tax() {
    let mut d = averaging_return(2500);
    d.income.clear();
    add(&mut d, income(raised_farm(20000, 0)));
    averaging_statement_credit(&mut d);
    for b in &mut d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().years {
        b.source = AveragingSource::Return {
            agi: 0,
            deductions: 0,
            tax: 0,
            other_tax: vec![],
            loss: None,
        };
    }
    let r = supported(&d);
    // AGI19587; standard16100; QBI697; taxable2790. Elect2500:
    // current29 + three83 base-year increments =278, all absorbed by FTC.
    assert_eq!(value(&r, "regular_income_tax"), 278);
    assert_eq!(value(&r, "foreign_tax_credit"), 278);
    assert_eq!(r.tax, Some(282600)); // The credit cannot offset self-employment tax.
}

#[test]
fn farm_averaging_mixed_businesses_use_only_farm_qbi_component() {
    let mut d = averaging_return(60000);
    add(&mut d, income(cash_business(10000)));
    let r = supported(&d);
    // SE15543, half7772. QBI uses gross120k/10k: half7174 farm/598 other.
    // QBI 92,826 + 9,402 = 102,228, line 5 = 20,445.60 -> 20,446 (the activities'
    // 18,565 + 1,880 sum to 20,445); the farm's share of 20,446 is 18,566 and the
    // farm income worksheet retains its separate allocation.
    assert_eq!(value(&r, "qbi_deduction"), 20446);
    assert_eq!(value(&r, "farm_averaging_qbi_deduction"), 18566);
    assert_eq!(value(&r, "farm_averaging_eligible_income"), 74369);
    assert_eq!(value(&r, "regular_income_tax"), 16262);
    assert_eq!(r.tax, Some(3180500));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn farm_averaging_nonfarm_losses_reduce_farm_qbi_once() {
    let mut d = averaging_return(60000);
    add(&mut d, income(cash_business(-10000)));
    let r = supported(&d);
    // Farm100k minus halfSE6359=93641. Nonfarm10k loss =>QBI83641.
    assert_eq!(value(&r, "qbi_deduction"), 16728);
    assert_eq!(value(&r, "farm_averaging_qbi_deduction"), 16728);
    assert_eq!(value(&r, "farm_averaging_eligible_income"), 76913);
    assert_eq!(value(&r, "regular_income_tax"), 12995);
    assert_eq!(r.tax, Some(2571200));
}

#[test]
fn farm_averaging_reit_qbi_is_not_a_farm_deduction() {
    let mut d = averaging_return(60000);
    d.deductions.as_mut().unwrap().itemize_override = None;
    add(
        &mut d,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                ordinary: 500000,
                section199a: ReitDividends::Reported { amount: 500000, eligible: Some(true) },
                special: crate::DividendSpecial::NoneOfThese,
                ..Default::default()
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 19587);
    assert_eq!(value(&r, "farm_averaging_qbi_deduction"), 18587);
    assert_eq!(value(&r, "farm_averaging_eligible_income"), 74348);
    assert_eq!(r.tax, Some(2963300));
    // 2026-11-02: without wages, automatic itemizing avoids the taxable-income
    // cap, retaining the source-supported farm QBI allocation and averaging.
    d.income.retain(|e| !matches!(e.income, FederalIncome::Wage { .. }));
    let kept = supported(&d);
    assert!(!noted(&kept, "WARN_FARM_AVERAGING_ALLOCATION_LEFT_OUT"));
    assert_eq!(value(&kept, "qbi_deduction"), 19587);
    assert_eq!(value(&kept, "farm_averaging_qbi_deduction"), 18587);
    assert_eq!(value(&kept, "farm_averaging_eligible_income"), 74348);
    let mut regular = d.clone();
    regular.screening.as_mut().unwrap().farm_averaging = None;
    assert!(kept.tax < supported(&regular).tax);
}

#[test]
fn farm_averaging_personal_medical_deduction_preserves_farm_income() {
    let mut d = averaging_return(30000);
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    {
        d.deductions.as_mut().unwrap().itemized = facts;
        d.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    };
    itemized(d.deductions.as_mut().unwrap()).medical = 10000000;
    let r = supported(&d);
    // AGI152935 less medical88530 (100000 minus 7.5% AGI11470).
    // QBI12881 under the income cap; taxable51524; remaining21524.
    // Current2335 + base-year increments3501 =5836; SE14130 unchanged.
    assert_eq!(value(&r, "qbi_deduction"), 12881);
    // Election cannot exceed current taxable income, even with larger farm income.
    assert_eq!(value(&r, "farm_averaging_eligible_income"), 51524);
    assert_eq!(value(&r, "regular_income_tax"), 5836);
    assert_eq!(r.tax, Some(1996600));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn farm_averaging_personal_property_tax_allows_itemizing_but_income_tax_needs_allocation() {
    let mut d = averaging_return(60000);
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    {
        d.deductions.as_mut().unwrap().itemized = facts;
        d.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    };
    itemized(d.deductions.as_mut().unwrap()).personal_property_taxes = 2500000;
    let r = supported(&d);
    // AGI152935 less personal SALT25000 and QBI18587 =109348.
    // Remaining49348 gives5674; base increments7101 =>12775; SE14130.
    assert_eq!(value(&r, "farm_averaging_eligible_income"), 74348);
    assert_eq!(value(&r, "regular_income_tax"), 12772);
    assert_eq!(r.tax, Some(2690200));
    itemized(d.deductions.as_mut().unwrap()).personal_tax =
        PersonalTaxElection::Income { paid: 10000, allocation: TaxIncomeAllocation::CurrentReturn };
    let r = supported(&d);
    assert!(noted(&r, "WARN_FARM_AVERAGING_ALLOCATION_LEFT_OUT"));
    let mut regular = d.clone();
    regular.screening.as_mut().unwrap().farm_averaging = None;
    assert_eq!(r.tax, supported(&regular).tax);
}

#[test]
fn farm_averaging_automatic_deductions_compare_total_tax_including_amt() {
    let mut d = averaging_return(60000);
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Automatic averaging deduction", 1000.0, 1000, 7000)];
    let r = supported(&d);
    // Elect zero itemized instead of standard16100: taxable134348,
    // residual74348 tax11069 +7101 increments =18170. Unaveraged24842;
    // TMT27104 gives AMT2262, total34562 vs standard-route34884.
    assert_eq!(value(&r, "regular_income_tax"), 18165);
    assert_eq!(value(&r, "alternative_minimum_tax"), 2262);
    assert_eq!(r.tax, Some(3455700));
    // 2026-11-02: clearing the override keeps the existing lower final liability.
    d.deductions.as_mut().unwrap().itemize_override = None;
    assert_eq!(supported(&d).tax, Some(3455700));
}

#[test]
fn farm_conservation_limit_carry_and_complete_return() {
    let d = conservation_return();
    let r = supported(&d);
    // Gross120k =>30k limit; costs40k + prior5k =>15k unused.
    // Ordinary20k expenses plus30k =>profit70k; SE9891, half4946.
    // AGI125054, QBI13011, taxable95943 =>regular15819; total25710.
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 30000);
    assert_eq!(value(&r, "farm_2_conservation_carryforward"), 15000);
    assert_eq!(value(&r, "farm_2_net_profit"), 70000);
    assert_eq!(value(&r, "self_employment_tax"), 9891);
    assert_eq!(value(&r, "qbi_deduction"), 13011);
    assert_eq!(value(&r, "regular_income_tax"), 15816);
    assert_eq!(r.tax, Some(2570700));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn farm_conservation_capitalization_does_not_deduct_or_carry() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = Some(FarmConservation::Capitalize {
        method: FarmConservationCapitalMethod::NeverElected,
        costs: vec![ConservationCost {
            property: 3,
            paid_or_incurred: None,
            facts: conservation_facts(),
            label: "Capital conservation work".into(),
            amount: 4000000,
        }],
    });
    let r = supported(&d);
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 0);
    assert_eq!(value(&r, "farm_2_conservation_carryforward"), 0);
    assert_eq!(value(&r, "farm_2_conservation_basis_addition"), 40000);
    assert_eq!(r.tax, Some(3510800));
}

#[test]
fn farm_conservation_unestablished_method_omits_only_the_deduction() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    let Some(FarmConservation::Deduct { method, .. }) = &mut data.conservation else {
        unreachable!()
    };
    *method = FarmConservationMethod::Unestablished;
    let r = supported(&d);
    assert!(noted(&r, "WARN_FARM_CONSERVATION_FACTS_OMITTED"));
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 0);
    assert_eq!(value(&r, "farm_2_net_profit"), 100000);
    // Accepted source facts remain; only the unestablished elective deduction is omitted.
    let FederalIncome::Business { data } = &d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &data.activity else { unreachable!() };
    assert!(data.conservation.is_some());
}

#[test]
fn farm_conservation_published_limit_and_no_income_carry() {
    let c = FarmConservation::Deduct {
        assessments: vec![],
        costs: vec![ConservationCost {
            property: 3,
            paid_or_incurred: None,
            facts: conservation_facts(),
            label: "Conservation source example".into(),
            amount: 1000000,
        }],
        carry: vec![],
        method: conservation_method(),
    };
    // Publication225: gross32000, costs10000 =>8000 deducted,2000 carried.
    assert_eq!(c.deduction(3200000), 800000);
    assert_eq!(c.deduction(0), 0);
    assert_eq!(c.deduction(-10000), 0);
}

#[test]
fn a_section_481a_increase_is_gross_income_from_farming_for_the_section_175_limit() {
    // F3 (2026-10-09). Gross farm income $120,000 limits the deduction to 25% = $30,000. A
    // $40,000 section 481(a) increase on the farm is farming income too, so the limit rises to
    // 25% of $160,000 = $40,000 and $40,000 of the $45,000 available is deducted.
    let mut d = conservation_return();
    assert_eq!(value(&supported(&d), "farm_2_conservation_deduction"), 30000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.method_changes.push(MethodChange {
        id: d.next_id,
        year: 2026,
        consent: MethodChangeConsent::Automatic { dcn: 122 },
        adjustment: Adjustment481::OneYear { amount: 4_000_000 },
        amt: crate::MethodChangeAmt::Same,
    });
    d.next_id += 1;
    let r = supported(&d);
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 40000);
    assert_eq!(value(&r, "farm_2_conservation_carryforward"), 5000);
}

#[test]
fn farm_conservation_does_not_assume_every_schedule_f_receipt_is_production_income() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    let FarmAccounting::Cash { receipts } = &mut data.accounting else { unreachable!() };
    receipts.push(FarmReceipt::CustomHire { amount: 1000000 });
    let result = supported(&d);
    // Services add ordinary farm income, but do not increase the production
    // gross pool: 25% of the unchanged120000 is still30000.
    assert_eq!(value(&result, "farm_2_conservation_deduction"), 30000);
    assert_eq!(value(&result, "farm_2_net_profit"), 80000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    let FarmAccounting::Cash { receipts } = &mut farm.accounting else { unreachable!() };
    receipts.push(FarmReceipt::Other { amount: 100000 });
    let result = supported(&d);
    assert!(noted(&result, "WARN_FARM_CONSERVATION_GROSS_SOURCE"));
    assert_eq!(value(&result, "farm_2_conservation_deduction"), 0);
    assert_eq!(value(&result, "farm_2_net_profit"), 111000);
    assert_eq!(estimate(&restart(d.clone()).draft), result);
}

#[test]
fn farm_conservation_counts_program_ccc_and_insurance_receipts_in_gross_farm_income() {
    // Section 175(b)(1), Reg. 1.175-5 and the Schedule F instructions (line 12):
    // production income, ordinary program payments, a CCC loan treated as
    // income and crop insurance proceeds are gross income from farming. The
    // base return has $120,000 (limit $30,000 against costs 40,000 + 5,000
    // carried). With a $20,000 program payment: gross 140,000, limit 25% x
    // 140,000 = $35,000 deducted, 45,000 - 35,000 = $10,000 carried.
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    let FarmAccounting::Cash { receipts } = &mut data.accounting else { unreachable!() };
    receipts.push(FarmReceipt::Program { amount: 2000000 });
    assert!(data.conservation_source_ready());
    let r = supported(&d);
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 35000);
    assert_eq!(value(&r, "farm_2_conservation_carryforward"), 10000);
    // Crop insurance records do not stop the pool either;
    // custom hire is excluded; unresolved cost-sharing, other income and CRP
    // character omit the elective pool, preserving ordinary income.
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.insurance =
        FarmInsuranceRecords::Payments { current: vec![], previous_deferred: 0, withholding: 0 };
    assert!(data.conservation_source_ready());
    for refused in [FarmReceipt::CostSharing { amount: 100 }, FarmReceipt::Other { amount: 100 }] {
        let FarmAccounting::Cash { receipts } = &mut data.accounting else { unreachable!() };
        receipts.push(refused);
        assert!(!data.conservation_source_ready());
        let FarmAccounting::Cash { receipts } = &mut data.accounting else { unreachable!() };
        receipts.pop();
    }
}

#[test]
fn conservation_district_published_examples_flow_to_full_return() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = Some(FarmConservation::Deduct {
        costs: vec![ConservationCost {
            property: 3,
            paid_or_incurred: None,
            facts: conservation_facts(),
            label: "District drainage ditches".into(),
            amount: 150000,
        }],
        assessments: vec![district_assessment(7000, 900, 2026, 900)],
        carry: vec![],
        method: conservation_method(),
    });
    let r = supported(&d);
    // Pub225 example1: share900 capped700; threshold570 exceeded,
    // so70 now,630 in following9 years;200 to basis. Add ditches1500.
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 1570);
    assert_eq!(value(&r, "farm_2_conservation_assessment_deferred"), 630);
    assert_eq!(value(&r, "farm_2_conservation_basis_addition"), 200);
    assert_eq!(value(&r, "farm_2_net_profit"), 98430);
    assert_eq!(value(&r, "self_employment_tax"), 13908);
    assert_eq!(value(&r, "regular_income_tax"), 20697);
    assert_eq!(r.tax, Some(3460500));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    let Some(FarmConservation::Deduct { costs, assessments, .. }) = &mut data.conservation else {
        unreachable!()
    };
    costs[0].amount = 185000;
    assessments[0] = district_assessment(5500, 550, 2026, 550);
    let r = supported(&d);
    // Pub225 example2:550 <=555 threshold, so the full2400 is available.
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 2400);
    assert_eq!(value(&r, "farm_2_conservation_assessment_deferred"), 0);
    assert_eq!(value(&r, "farm_2_conservation_basis_addition"), 0);
    assert_eq!(r.tax, Some(3433900));
}

#[test]
fn conservation_assessments_group_installments_before_annual_limit() {
    let mut a = district_assessment(7000, 900, 2026, 450);
    a.payments.push(ConservationAssessmentPayment {
        current_date: None,
        year: 2026,
        amount: 45000,
    });
    assert_eq!(a.amounts(), [7000, 63000, 20000]);
    a.payments[0].year = 2025;
    // Each year350 deductible/100 capital; neither350 exceeds570.
    assert_eq!(a.amounts(), [35000, 0, 10000]);
    let a = district_assessment(7000, 900, 2025, 900);
    assert_eq!(a.amounts(), [7000, 56000, 0]);
    assert_eq!(district_assessment(7000, 900, 2017, 900).amounts(), [7000, 0, 0]);
    assert_eq!(district_assessment(7000, 900, 2016, 900).amounts(), [0, 0, 0]);
}

#[test]
fn conservation_assessment_500_exception_is_a_threshold_not_an_extra_deduction() {
    assert_eq!(district_assessment(10000, 1000, 2026, 600).amounts(), [60000, 0, 0]);
    assert_eq!(district_assessment(10000, 1000, 2026, 601).amounts(), [10000, 50100, 0]);
    // Annual fractions conserve every cent over the complete nine-year spread.
    let mut sum = 10000;
    for elapsed in 1..=9 {
        sum += district_assessment(10000, 1000, 2026 - elapsed, 601).amounts()[0];
    }
    assert_eq!(sum, 60100);
}

#[test]
fn conservation_assessment_spread_is_distinct_from_income_limited_carry() {
    let c = FarmConservation::Deduct {
        costs: vec![],
        assessments: vec![district_assessment(7000, 900, 2026, 900)],
        carry: vec![],
        method: conservation_method(),
    };
    // Available70 is then limited to25% of100. The other45 carries under
    // the income limit;630 remains on the separate nine-year schedule.
    assert_eq!(c.deduction(10000), 2500);
}

#[test]
fn conservation_assessment_low_income_preserves_both_future_balances() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.expenses.clear();
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.accounting =
        FarmAccounting::Cash { receipts: vec![FarmReceipt::Raised { amount: 10000 }] };
    data.conservation = Some(FarmConservation::Deduct {
        costs: vec![],
        assessments: vec![district_assessment(7000, 900, 2026, 900)],
        carry: vec![],
        method: conservation_method(),
    });
    let r = supported(&d);
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 25);
    assert_eq!(value(&r, "farm_2_conservation_carryforward"), 45);
    assert_eq!(value(&r, "farm_2_conservation_assessment_deferred"), 630);
    assert_eq!(value(&r, "farm_2_conservation_basis_addition"), 200);
    assert_eq!(r.tax, Some(502900));
}

#[test]
fn conservation_assessment_rejects_payments_exceeding_source_assessment() {
    let c = FarmConservation::Deduct {
        costs: vec![],
        assessments: vec![district_assessment(7000, 900, 2026, 901)],
        carry: vec![],
        method: conservation_method(),
    };
    assert!(!c.valid());
}

#[test]
fn conservation_one_cost_farm_uses_same_filers_combined_farm_income_limit() {
    let mut d = conservation_return();
    add(&mut d, income(raised_farm(10000, 2000)));
    let r = supported(&d);
    // Combined gross130000 =>32500 limit on first farm's45000 costs/carry.
    // Profits67500 +8000 =75500; SE10668, half5334; QBI14033.
    // AGI130166, taxable100033 =>regular16719, total27387.
    assert_eq!(value(&r, "farm_2_conservation_income_limit"), 32500);
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 32500);
    assert_eq!(value(&r, "farm_2_conservation_carryforward"), 12500);
    assert_eq!(value(&r, "self_employment_tax"), 10668);
    assert_eq!(value(&r, "regular_income_tax"), 16719);
    assert_eq!(r.tax, Some(2738700));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn conservation_multiple_cost_farms_deduct_all_when_shared_limit_does_not_bind() {
    let mut d = conservation_return();
    let mut second = raised_farm(80000, 10000);
    let FederalIncome::Business { data } = &mut second else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = Some(FarmConservation::Deduct {
        costs: vec![ConservationCost {
            property: 0,
            paid_or_incurred: None,
            facts: conservation_facts(),
            label: "Second farm qualifying terraces".into(),
            amount: 500000,
        }],
        assessments: vec![],
        carry: vec![],
        method: conservation_method(),
    });
    add(&mut d, income(second));
    let r = supported(&d);
    // Combined gross200k permits all50k although first farm's own cap is30k.
    // Profits55k +65k =120k; SE16955, half8478. Activity QBI10223+12082.
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 45000);
    assert_eq!(value(&r, "farm_4_conservation_deduction"), 5000);
    assert_eq!(value(&r, "farm_2_conservation_carryforward"), 0);
    // Form 8995 line 5 is 20% of line 4: 20% x (120,000 - 8,478) = 20% x 111,522
    // = 22,304.40 -> 22,304 (the per-activity sum is 22,305).
    assert_eq!(value(&r, "qbi_deduction"), 22304);
    assert_eq!(r.tax, Some(4150100));
    // Reducing the second farm's gross to10k binds the joint cost pool.
    let FederalIncome::Business { data } = &mut d.income[2].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.accounting =
        FarmAccounting::Cash { receipts: vec![FarmReceipt::Raised { amount: 1000000 }] };
    let r = supported(&d);
    // 130,000 gross yields a 32,500 cap. Eligible costs 45,000:5,000
    // receive the approved proportional split 29,250:3,250.
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 29250);
    assert_eq!(value(&r, "farm_4_conservation_deduction"), 3250);
    assert_eq!(value(&r, "farm_2_conservation_carryforward"), 15750);
    assert_eq!(value(&r, "farm_4_conservation_carryforward"), 1750);
    d.income.reverse();
    let reordered = supported(&d);
    assert_eq!(value(&reordered, "farm_2_conservation_deduction"), 29250);
    assert_eq!(supported(&restart(d).draft).tax, reordered.tax);
}

#[test]
fn conservation_checks_other_farm_sources_and_ownership_before_sharing_limit() {
    let mut d = conservation_return();
    add(&mut d, income(farm_business(vec![FarmReceipt::CustomHire { amount: 1000000 }], 2000)));
    let result = supported(&d);
    // This other activity contributes taxable service receipts but no
    // production receipts to the taxpayer-wide section175 gross limit.
    assert_eq!(value(&result, "farm_2_conservation_deduction"), 30000);
    let mut d = conservation_return();
    let mut e = income(raised_farm(10000, 2000));
    e.owner = crate::DocumentOwner::Spouses;
    add(&mut d, e);
    let mut joint = complete(FederalStatus::Joint);
    joint.next_id = d.next_id;
    joint.income = d.income;
    let r = supported(&joint);
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 32500);
}

#[test]
fn conservation_shared_limit_precedes_section179_business_income_limit() {
    let mut d = conservation_return();
    add(&mut d, income(raised_farm(10000, 2000)));
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.assets = vec![elected_asset(100000, 100000, BonusElection::ElectOut)];
    d.income.remove(0); // No wages to enlarge the section179 income limit.
    let r = supported(&d);
    // Farm operating profit108000 less shared conservation32500 =75500.
    assert_eq!(value(&r, "section179_business_income_limit"), 75500);
    assert_eq!(value(&r, "section179_deduction"), 75500);
    assert_eq!(value(&r, "section179_carryforward"), 24500);
    assert_eq!(value(&r, "agi"), 0);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn conservation_final_return_accelerates_prior_assessment_balance_once() {
    let d = conservation_final_return();
    let r = supported(&d);
    // Of eligible700,70 was available in2025. All remaining630 becomes
    // available in2026; the capital200 belongs to2025, not a new basis item.
    // Profit99370 ->SE14041, half7021; AGI152349, QBI18470;
    // taxable117779 ->regular20865, total34906.
    assert_eq!(value(&r, "farm_2_conservation_assessment_available"), 630);
    assert_eq!(value(&r, "farm_2_conservation_assessment_deferred"), 0);
    assert_eq!(value(&r, "farm_2_conservation_basis_addition"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 14041);
    assert_eq!(value(&r, "regular_income_tax"), 20865);
    assert_eq!(r.tax, Some(3490600));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn conservation_final_return_expires_income_limited_balance_without_estate_transfer() {
    let mut d = conservation_final_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.expenses.clear();
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.accounting =
        FarmAccounting::Cash { receipts: vec![FarmReceipt::Raised { amount: 10000 }] };
    let r = supported(&d);
    // Accelerated630 still faces25% of gross100. The unused605 expires.
    assert_eq!(value(&r, "farm_2_conservation_deduction"), 25);
    assert_eq!(value(&r, "farm_2_conservation_expired"), 605);
    assert_eq!(value(&r, "farm_2_conservation_carryforward"), 0);
    assert_eq!(value(&r, "farm_2_conservation_assessment_deferred"), 0);
    assert_eq!(r.tax, Some(502900));
}

#[test]
fn conservation_death_acceleration_follows_farm_owner_not_any_deceased_spouse() {
    let source = conservation_final_return();
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().died =
        source.people.as_ref().unwrap().taxpayer.died.clone();
    d.next_id = source.next_id;
    d.income = source.income;
    let living_farmer = supported(&d);
    assert_eq!(value(&living_farmer, "farm_2_conservation_assessment_available"), 70);
    assert_eq!(value(&living_farmer, "farm_2_conservation_assessment_deferred"), 560);
    d.income[1].owner = crate::DocumentOwner::Spouses;
    let deceased_farmer = supported(&d);
    assert_eq!(value(&deceased_farmer, "farm_2_conservation_assessment_available"), 630);
    assert_eq!(value(&deceased_farmer, "farm_2_conservation_assessment_deferred"), 0);
    assert_eq!(deceased_farmer.tax, Some(2583400));
}

#[test]
fn conservation_new_death_year_spread_is_not_treated_as_a_succeeding_year() {
    let mut d = conservation_final_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    let Some(FarmConservation::Deduct { assessments, .. }) = &mut data.conservation else {
        unreachable!()
    };
    assessments[0].payments[0].year = 2026;
    unsupported(&d, "Farm conservation");
}

#[test]
fn farm_land_conservation_recapture_respects_holding_year_boundaries() {
    for (acquired, sold, percent, recapture) in [
        (20210615, 20260615, 100, 20000),
        (20210615, 20260616, 80, 16000),
        (20200615, 20260616, 60, 12000),
        (20190615, 20260616, 40, 8000),
        (20180615, 20260616, 20, 4000),
        (20170615, 20260615, 20, 4000),
        (20170615, 20260616, 0, 0),
    ] {
        let sale = farm_land_sale(acquired, sold);
        assert!(sale.valid());
        let v = sale.calculate();
        assert_eq!(v.gain, 40000);
        assert_eq!(v.percent, percent);
        assert_eq!(v.recapture, recapture);
        assert_eq!(v.ordinary + v.section1231, 40000);
    }
}

#[test]
fn farm_land_recapture_is_gain_limited_and_losses_are_not_recaptured() {
    let mut sale = farm_land_sale(20220101, 20261201);
    sale.proceeds = 11500000;
    let v = sale.calculate();
    assert_eq!((v.gain, v.ordinary, v.section1231), (5000, 5000, 0));
    sale.proceeds = 10000000;
    let v = sale.calculate();
    assert_eq!((v.gain, v.recapture, v.section1231), (-10000, 0, -10000));
    sale.acquired = 20260101;
    let v = sale.calculate();
    assert_eq!((v.ordinary, v.section1231), (-10000, 0));
}

#[test]
fn farm_land_one_year_boundary_routes_to_correct_form4797_character() {
    let short = farm_land_sale(20250615, 20260615).calculate();
    assert_eq!((short.ordinary, short.section1231), (40000, 0));
    let long = farm_land_sale(20250615, 20260616).calculate();
    assert_eq!((long.ordinary, long.section1231), (20000, 20000));
}

#[test]
fn direct_farm_land_requires_complete_lookback_before_return_output() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    set_land_sales(data, vec![farm_land_sale(20210615, 20260616)], &mut d.next_id);
    let mut r = estimate(&complete(FederalStatus::Single));
    farmland_resolved_sale(&d, 0).report(2, 0, crate::farm::FarmForm::ScheduleF, &mut r);
    assert_eq!(value(&r, "farm_2_land_0_gain"), 40000);
    assert_eq!(value(&r, "farm_2_land_0_conservation_recapture"), 16000);
    assert_eq!(value(&r, "farm_2_land_0_section1231"), 24000);
    unsupported(&d, "Section 1231 lookback");
    assert_eq!(estimate(&restart(d).draft).tax, None);
}

#[test]
fn direct_land_sale_uses_shared_lookback_after_conservation_recapture() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    set_land_sales(data, vec![farm_land_sale(20210615, 20260616)], &mut d.next_id);
    // $40,000 gain minus $16,000 section 1252 recapture leaves $24,000
    // for the existing return-wide lookback. Independent AMT history applies.
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2023, 9000, 7000)], &[(2023, 5000, 3000)]));
    let regular = lookback_worksheet(&d, false).unwrap().unwrap();
    assert_eq!((regular.net, regular.ordinary, regular.long_term), (24000, 7000, 17000));
    let amt = lookback_worksheet(&d, true).unwrap().unwrap();
    assert_eq!((amt.net, amt.ordinary, amt.long_term), (24000, 3000, 21000));
    d.screening.as_mut().unwrap().section1231 = None;
    assert!(lookback_worksheet(&d, false).is_err());
    unsupported(&d, "Section 1231 lookback");
}

#[test]
fn farm_land_rental_magi_includes_recapture_and_net_capital_effect_once() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    set_land_sales(data, vec![farm_land_sale(20210615, 20260616)], &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let adjustment = |d: &FederalDraft, long| {
        crate::section1231::rental_magi_adjustment(
            d,
            crate::capital::CapitalAmounts { long, ..Default::default() },
            [0, 0],
            false,
        )
        .unwrap()
    };
    assert_eq!(adjustment(&d, 0), 40000);
    // Portfolio loss was already capped at -$3,000. Adding $24,000 capital
    // gain leaves it capped there: only the $16,000 ordinary recapture adds MAGI.
    assert_eq!(adjustment(&d, -30000), 16000);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2023, 7000, 7000)], &[]));
    assert_eq!(adjustment(&d, -30000), 23000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.land_sales[0].proceeds = 10000000;
    assert_eq!(adjustment(&d, 0), -10000);
}

#[test]
fn direct_farm_land_qbi_uses_ordinary_character_without_self_employment_income() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    set_land_sales(data, vec![farm_land_sale(20210615, 20260616)], &mut d.next_id);
    for (lookback, expected) in [(0, 16000), (7000, 23000), (30000, 40000)] {
        d.screening.as_mut().unwrap().section1231 = Some(section1231_history(
            &if lookback == 0 { vec![] } else { vec![(2023, lookback, lookback)] },
            &[],
        ));
        let mut r = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals_for(&d, 200000, &mut r, false);
        let character = crate::section1231::worksheet(&d, &pool, false).unwrap();
        assert_eq!(crate::farm_land::qbi(&d, &pool, character.as_ref(), 2), Ok(expected));
        assert_eq!(crate::farm_land::qbi(&d, &pool, character.as_ref(), 99), Ok(0));
        let FederalIncome::Business { data } = &d.income[1].income else { unreachable!() };
        assert_eq!(data.earned_receipts(), 12000000);
    }
    let mut mixed = d.clone();
    let mut other = mixed.income[1].clone();
    other.id = mixed.next_id;
    mixed.next_id += 1;
    mixed.income.push(other);
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals_for(&mixed, 200000, &mut r, false);
    let character = crate::section1231::worksheet(&mixed, &pool, false).unwrap();
    // Two $24,000 gains share $30,000 lookback. No invented proportional
    // business allocation is permitted from these source facts alone.
    assert!(crate::farm_land::qbi(&mixed, &pool, character.as_ref(), 2).is_err());
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.land_sales[0].proceeds = 10000000;
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals_for(&d, 200000, &mut r, false);
    let character = crate::section1231::worksheet(&d, &pool, false).unwrap();
    assert_eq!(crate::farm_land::qbi(&d, &pool, character.as_ref(), 2), Ok(-10000));
    supported(&d);
}

#[test]
fn direct_farm_land_niit_excludes_active_gain_under_net_gains_worksheet() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    set_land_sales(data, vec![farm_land_sale(20210615, 20260616)], &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let result = property_niit_worksheet(&d, 24000, 24000).unwrap();
    assert_eq!((result.adjustment, result.next_capital_adjustment), (-24000, 0));
    // Form 8960 worksheet 2(i)(1) retains only a positive adjustment.
    // Excluded gains remain a current negative adjustment even when regular
    // capital losses are capped; do not invent a negative carryover.
    let result = property_niit_worksheet(&d, -6000, -3000).unwrap();
    assert_eq!(result.adjustment, -24000);
    assert_eq!(result.next_capital_adjustment, 0);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2023, 7000, 7000)], &[]));
    let result = property_niit_worksheet(&d, 17000, 17000).unwrap();
    assert_eq!(result.adjustment, -17000);
    // A business without material participation never reaches the worksheet (the
    // estimate leaves it out first), so it has no refusal of its own here.
}

#[test]
fn farm_land_gain_and_loss_change_section179_income_without_changing_receipts() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.assets = vec![elected_asset(250000, 250000, BonusElection::ElectOut)];
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    set_land_sales(data, vec![farm_land_sale(20210615, 20260616)], &mut d.next_id);
    // $60,000 wages + $100,000 operating profit + $40,000 land gain.
    let deduction = crate::section179::calculate(&d);
    assert_eq!(deduction.income, 20000000);
    assert_eq!(deduction.allowed, 20000000);
    assert_eq!(deduction.available - deduction.allowed, 5000000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    assert_eq!(data.earned_receipts(), 12000000);
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.land_sales[0].proceeds = 10000000;
    // The $10,000 land loss reduces the same income limit.
    let deduction = crate::section179::calculate(&d);
    assert_eq!(deduction.income, 15000000);
    assert_eq!(deduction.allowed, 15000000);
    assert_eq!(deduction.available - deduction.allowed, 10000000);
}

#[test]
fn direct_farm_land_sale_produces_complete_return_and_survives_restart() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    set_land_sales(data, vec![farm_land_sale(20210615, 20260616)], &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&d);
    // Special treatment of the sale (No) leaves it out with a notice.
    let mut special = d.clone();
    farmland_sales_mut(&mut special)[0].situation = FarmLandSituation::Other;
    let unresolved = estimate(&special);
    assert_eq!(unresolved.state, ResultState::Unsupported);
    assert_eq!(unresolved.tax, None);
    assert!(!noted(&unresolved, "WARN_FARM_LAND_SALE_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 192935);
    assert_eq!(value(&r, "self_employment_tax"), 14130);
    assert_eq!(value(&r, "qbi_deduction"), 21787);
    assert_eq!(value(&r, "taxable_income"), 155048);
    assert_eq!(value(&r, "regular_income_tax"), 27650);
    assert_eq!(r.tax, Some(4178000));
    assert!(!r.return_lines.is_empty());
    let mut lookback = d.clone();
    lookback.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2023, 7000, 7000)], &[]));
    let v = supported(&lookback);
    assert_eq!(value(&v, "qbi_deduction"), 23187);
    assert_eq!(value(&v, "taxable_income"), 153648);
    assert_eq!(v.tax, Some(4207400));
    let mut investment = d.clone();
    add(
        &mut investment,
        income(FederalIncome::Interest { data: interest_source(native_interest(1000000, 0)) }),
    );
    let v = supported(&investment);
    assert_eq!(value(&v, "agi"), 202935);
    assert_eq!(value(&v, "net_investment_income_tax"), 112);
    assert_eq!(v.tax, Some(4429200));
    let mut loss = d.clone();
    let FederalIncome::Business { data } = &mut loss.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.land_sales[0].proceeds = 10000000;
    let v = supported(&loss);
    assert_eq!(value(&v, "agi"), 142935);
    assert_eq!(value(&v, "qbi_deduction"), 16587);
    assert_eq!(value(&v, "section1231_current_loss_history"), 10000);
    assert_eq!(v.tax, Some(3318800));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn farm_averaging_excludes_capital_only_land_gain_without_blocking_election() {
    let mut d = averaging_return(60000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    set_land_sales(data, vec![farm_land_sale(20100615, 20260616)], &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 192935);
    assert_eq!(value(&r, "qbi_deduction"), 18587);
    assert_eq!(value(&r, "taxable_income"), 158248);
    // $40,000 land gain is excluded from elected farm income and remains
    // current-year preferential income: $7,527 + $6,000 + $7,101 = $20,628.
    assert_eq!(value(&r, "farm_averaging_eligible_income"), 74348);
    assert_eq!(value(&r, "regular_income_tax"), 20623);
    assert_eq!(value(&r, "self_employment_tax"), 14130);
    assert_eq!(r.tax, Some(3475300));
    let mut recaptured = d.clone();
    recaptured.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2023, 7000, 7000)], &[]));
    let omitted = supported(&recaptured);
    assert!(noted(&omitted, "WARN_FARM_AVERAGING_ALLOCATION_LEFT_OUT"));
    let mut regular = recaptured.clone();
    regular.screening.as_mut().unwrap().farm_averaging = None;
    assert_eq!(omitted.tax, supported(&regular).tax);
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn direct_farm_land_excess_loss_defers_only_the_deductible_property_history() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    let mut sale = farm_land_sale(20210615, 20260616);
    sale.basis = 54000000; // $150,000 proceeds - $10,000 selling - $540,000 basis.
    set_land_sales(data, vec![sale], &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    waive_farm_carryback(&mut d);
    let r = supported(&d);
    // $100,000 operating profit - $7,065 half-SE - $400,000 land loss.
    assert_eq!(value(&r, "excess_business_loss"), 51065);
    assert_eq!(value(&r, "agi"), -196000);
    assert_eq!(value(&r, "section1231_current_loss_history"), 348935);
    assert_eq!(value(&r, "amt_section1231_current_loss_history"), 348935);
    assert_eq!(value(&r, "self_employment_tax"), 14130);
    assert_eq!(r.tax, Some(1413000));
    assert_eq!(value(&r, "qbi_loss_carryforward"), 256000);
    assert_eq!(value(&r, "nol_carryforward"), 247065);
    let mut operating = d.clone();
    let FederalIncome::Business { data } = &mut operating.income[1].income else { unreachable!() };
    data.expenses[0].amount = 52000000;
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.land_basis[0].regular = 10000000;
    data.land_basis[0].amt = 10000000;
    let v = supported(&operating);
    // $400,000 operating loss, $40,000 land gain; all $104,000 deferral
    // belongs to the ordinary loss, not a nonexistent section 1231 loss.
    assert_eq!(value(&v, "excess_business_loss"), 104000);
    assert_eq!(value(&v, "section1231_current_loss_history"), 0);
    assert_eq!(value(&v, "qbi_loss_carryforward"), 280000);
    assert_eq!(value(&v, "nol_carryforward"), 300000);
    assert_eq!(v.tax, Some(0));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn farm_land_independent_amt_basis_recomputes_gain_recapture_and_final_tax() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    let mut sale = farm_land_sale(20210615, 20260616);
    sale.amt = FarmLandAmt::Different { basis: 13000000, conservation: 500000 };
    assert_eq!(
        (sale.calculate_for(true).ordinary, sale.calculate_for(true).section1231),
        (4000, 6000)
    );
    assert_eq!((sale.calculate().ordinary, sale.calculate().section1231), (16000, 24000));
    set_land_sales(data, vec![sale], &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&d);
    assert_eq!(value(&r, "amt_capital_gain"), 6000);
    assert_eq!(value(&r, "amt_capital_adjustment"), -18000);
    assert_eq!(value(&r, "agi"), 192935);
    assert_eq!(value(&r, "qbi_deduction"), 21787);
    assert_eq!(r.tax, Some(4178000));
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.land_basis[0].amt = 0;
    data.land_basis[0].prior_amt_conservation = 2000000;
    let r = supported(&d);
    assert_eq!(value(&r, "amt_capital_gain"), 124000);
    assert_eq!(value(&r, "amt_capital_adjustment"), 100000);
    // AMTI $271,148 less $90,100 exemption: $57,048 ordinary at 26%
    // plus $124,000 capital at 15% = $33,432 TMT. AMT is $5,782.
    assert_eq!(r.tax, Some(4756200));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
}

#[test]
fn inherited_farm_land_uses_long_term_character_without_fictitious_acquisition_date() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    let mut sale = farm_land_sale(20260301, 20260616);
    sale.acquisition = FarmLandAcquisition::Inherited;
    sale.conservation = 0; // Estate FMV basis resets the decedent's deductions.
    assert_eq!((sale.calculate().ordinary, sale.calculate().section1231), (0, 40000));
    set_land_sales(data, vec![sale], &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 192935);
    assert_eq!(value(&r, "qbi_deduction"), 18587);
    assert_eq!(value(&r, "taxable_income"), 158248);
    assert_eq!(value(&r, "regular_income_tax"), 26978);
    assert_eq!(r.tax, Some(4110800));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    // An heir's own prior-year deductions remain subject to recapture;
    // deemed long-term character does not pretend ten actual years elapsed.
    let mut own = farm_land_sale(20251201, 20260616);
    own.acquisition = FarmLandAcquisition::Inherited;
    own.conservation = 500000;
    assert_eq!((own.calculate().ordinary, own.calculate().section1231), (5000, 35000));
}

#[test]
fn short_held_farm_land_does_not_require_unrelated_section1231_history() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    let mut sale = farm_land_sale(20260301, 20260616);
    sale.conservation = 0;
    set_land_sales(data, vec![sale], &mut d.next_id);
    assert!(d.screening.as_ref().unwrap().section1231.is_none());
    let r = supported(&d);
    assert_eq!(value(&r, "farm_2_land_0_ordinary"), 40000);
    assert_eq!(value(&r, "qbi_deduction"), 26587);
    assert_eq!(value(&r, "taxable_income"), 150248);
    assert_eq!(value(&r, "regular_income_tax"), 28658);
    assert_eq!(r.tax, Some(4278800));
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.land_basis[0].acquired = 20240301;
    unsupported(&d, "Section 1231 lookback");
}

#[test]
fn farmland_tax_installment_election_retains_facts_and_reconciles_complete_return() {
    let mut sale = farm_land_sale(20160616, 20260616);
    // Choosing the election states the use, buyer and covenant facts; only
    // a recognized gain is still checked.
    assert!(crate::farm_deferral::qualifies(&sale));
    sale.deferral = true;
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    set_land_sales(data, vec![sale], &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let result = supported(&d);
    // $40,000 gain is entirely capital after ten years; ordinary taxable
    // income is $118,248. Its $20,978 tax plus $6,000 capital tax is $26,978.
    assert_eq!(value(&result, "agi"), 192935);
    assert_eq!(value(&result, "qbi_deduction"), 18587);
    assert_eq!(value(&result, "regular_income_tax"), 26978);
    assert_eq!(result.tax, Some(4110800)); // Includes unchanged $14,130 SE tax.
    assert_eq!(
        result.farmland_installments,
        Some(FarmTaxInstallments {
            applicable: 600000,
            first: 150000,
            remaining: 450000,
            current: 3660800,
        })
    );
    assert_eq!(value(&result, "farm_installment_reduced_taxable"), 118248);
    assert_eq!(value(&result, "farm_installment_reduced_regular_tax"), 20978);
    assert_eq!(result.balance, Some(3660800 - result.payments.unwrap()));
    let form = |reference: &str| {
        result
            .return_lines
            .iter()
            .find(|line| {
                status_text(&line.label).starts_with(&alloc::format!("1040 {reference} —"))
            })
            .unwrap()
            .amount
    };
    assert_eq!(form("24a"), 4110800);
    assert_eq!(form("24b"), 150000);
    assert_eq!(form("24c"), 4260800);
    assert_eq!(form("33"), result.payments.unwrap() + 600000);
    assert_eq!(form("37"), result.balance.unwrap());
    let restored = restart(d.clone()).draft;
    assert_eq!(restored.income, d.income);
    assert_eq!(supported(&restored), result);

    let mut ordinary = d.clone();
    farmland_sales_mut(&mut ordinary)[0].deferral = false;
    let baseline = supported(&ordinary);
    assert_eq!(baseline.tax, result.tax);
    assert_eq!(baseline.payments, result.payments); // No fabricated cash payment.
    assert_eq!(baseline.refundable_credits, result.refundable_credits);
    assert!(baseline.farmland_installments.is_none());
    assert_eq!(baseline.balance.unwrap() - result.balance.unwrap(), 450000);

    let mut overpaid = d.clone();
    overpaid.payments.as_mut().unwrap().estimated =
        crate::tests::support::builders::unknown_estimated(5000000, PaymentPayer::Taxpayer);
    let overpaid = supported(&overpaid);
    assert_eq!(overpaid.refund, Some(overpaid.payments.unwrap() - 3660800));
    assert_eq!(overpaid.balance, Some(0));
}

#[test]
fn farmland_installments_aggregate_elections_and_keep_unselected_gain() {
    let mut d = conservation_return();
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.conservation = None;
    let mut sale = farm_land_sale(20160616, 20260616);
    sale.proceeds = 13000000; // $20,000 recognized gain per parcel.
    sale.deferral = true;
    set_land_sales(data, vec![sale.clone(), sale], &mut d.next_id);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let both = supported(&d);
    assert_eq!(both.tax, Some(4110800));
    assert_eq!(value(&both, "farm_installment_elected_gain"), 40000);
    assert_eq!(both.farmland_installments.as_ref().unwrap().applicable, 600000);
    farmland_sales_mut(&mut d)[1].deferral = false;
    let one = supported(&d);
    assert_eq!(one.tax, both.tax);
    assert_eq!(value(&one, "farm_installment_elected_gain"), 20000);
    assert_eq!(value(&one, "farm_installment_reduced_taxable"), 138248);
    assert_eq!(value(&one, "farm_installment_reduced_regular_tax"), 23978);
    assert_eq!(one.farmland_installments.as_ref().unwrap().applicable, 300000);
    assert_eq!(one.farmland_installments.as_ref().unwrap().first, 75000);

    // AMT without credits stays current; only regular tax is deferred.
    farmland_basis_mut(&mut d).amt = 0;
    farmland_basis_mut(&mut d).prior_amt_conservation = 0;
    let with_amt = supported(&d);
    assert_eq!(with_amt.farmland_installments.as_ref().unwrap().applicable, 300000);
    assert!(with_amt.tax.unwrap() > one.tax.unwrap());
}

#[test]
fn farmland_installments_relimit_credits_without_overwriting_actual_carryovers() {
    let mut d = farmland_installment_return();
    d.credits.as_mut().unwrap().adoption_carry =
        vec![AdoptionCarry { year: 2025, amount: 2500000, eligible: true }];
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 25000);
    assert_eq!(value(&r, "adoption_carryforward"), 0);
    assert_eq!(value(&r, "farm_installment_actual_credits"), 25000);
    assert_eq!(value(&r, "farm_installment_reduced_credits"), 20978);
    assert_eq!(r.tax, Some(1610800));
    assert_eq!(
        r.farmland_installments,
        Some(FarmTaxInstallments {
            applicable: 197800,
            first: 49500,
            remaining: 148300,
            current: 1462500,
        })
    );
    d.credits.as_mut().unwrap().adoption_carry[0].amount = 4000000;
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_carryforward"), 13022);
    assert_eq!(r.tax, Some(1413000));
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 0);
    assert_eq!(r.farmland_installments.as_ref().unwrap().current, 1413000);
}

#[test]
fn farmland_installments_net_lookback_and_capital_losses_in_shared_worksheets() {
    let mut d = farmland_installment_return();
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2023, 7000, 7000)], &[]));
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 19987);
    assert_eq!(value(&r, "taxable_income"), 156848);
    assert_eq!(value(&r, "regular_income_tax"), 27272);
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 20642);
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 663000);

    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut prior = capital_history();
    prior.agi = 10000000;
    prior.deductions = 2000000;
    prior.short = -3300000;
    prior.long = 0;
    prior.investment_parts = Some(InvestmentCapitalParts { short: 3_000_000, long: 0, amt: None });
    d.screening.as_mut().unwrap().capital_history = vec![prior];
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_income"), 128248);
    assert_eq!(value(&r, "regular_income_tax"), 22478);
    assert_eq!(value(&r, "farm_installment_reduced_taxable"), 88248);
    // $1,240 + $4,560 + 22% of $37,848 = $14,126.56, rounded once.
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 14122);
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 835600);
    assert_eq!(r.farmland_installments.as_ref().unwrap().first, 208900);
}

#[test]
fn farmland_installments_apply_rate_bands_once_for_all_elected_sales() {
    let mut d = farmland_installment_return();
    let sale = farmland_resolved_sale(&d, 0);
    d.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    d.income[1].income = raised_farm(0, 0);
    set_farmland_sales(&mut d, vec![sale.clone(), sale]);
    let r = supported(&d);
    // $80,000 capital income less $16,100 standard deduction leaves $63,900.
    // The first $49,450 is at 0%; $14,450 at 15% rounds to $2,168.
    assert_eq!(value(&r, "taxable_income"), 63900);
    assert_eq!(r.tax, Some(216800));
    assert_eq!(
        r.farmland_installments,
        Some(FarmTaxInstallments {
            applicable: 216800,
            first: 54200,
            remaining: 162600,
            current: 54200,
        })
    );
    farmland_sales_mut(&mut d)[1].deferral = false;
    let one = supported(&d);
    // Excluding either $40,000 already makes reduced tax zero. Summing two
    // independent differences would wrongly double the $2,168 liability.
    assert_eq!(one.farmland_installments.as_ref().unwrap().applicable, 216800);
}

#[test]
fn farmland_installments_leave_niit_and_self_employment_tax_current() {
    let mut d = farmland_installment_return();
    add(
        &mut d,
        income(FederalIncome::Interest { data: interest_source(native_interest(1000000, 0)) }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 202935);
    assert_eq!(value(&r, "self_employment_tax"), 14130);
    assert_eq!(value(&r, "net_investment_income_tax"), 112);
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 600000);
    let current = r.farmland_installments.as_ref().unwrap().current;
    assert_eq!(current, r.tax.unwrap() - 450000);
    farmland_sales_mut(&mut d)[0].deferral = false;
    let ordinary = supported(&d);
    assert_eq!(ordinary.tax, r.tax);
    assert_eq!(ordinary.payments, r.payments);
    assert_eq!(ordinary.balance.unwrap() - r.balance.unwrap(), 450000);
}

#[test]
fn farmland_installments_relimit_statement_credit_before_personal_credits() {
    let mut d = farmland_installment_return();
    farmland_statement_credit(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 193935);
    assert_eq!(value(&r, "regular_income_tax"), 27218);
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 21218);
    assert_eq!(value(&r, "foreign_tax_credit"), 300);
    assert_eq!(value(&r, "farm_installment_actual_foreign_credit"), 300);
    assert_eq!(value(&r, "farm_installment_reduced_foreign_credit"), 300);
    assert_eq!(r.tax, Some(4104800));
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 600000);

    d.credits.as_mut().unwrap().adoption_carry =
        vec![AdoptionCarry { year: 2025, amount: 2500000, eligible: true }];
    let r = supported(&d);
    // Actual net regular tax: $27,218 - $300 FTC - $25,000 adoption.
    // Reduced tax: $21,218 - $300 FTC - $20,918 adoption = zero.
    assert_eq!(value(&r, "farm_installment_actual_credits"), 25300);
    assert_eq!(value(&r, "farm_installment_reduced_credits"), 21218);
    assert_eq!(value(&r, "adoption_carryforward"), 0);
    assert_eq!(r.tax, Some(1604800));
    assert_eq!(
        r.farmland_installments,
        Some(FarmTaxInstallments {
            applicable: 191800,
            first: 48000,
            remaining: 143800,
            current: 1461000,
        })
    );
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let ForeignTaxClaim::Statements { eligible, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    else {
        unreachable!()
    };
    *eligible = None;
    // A source-discovered pending eligibility answer leaves the credit out with its notice.
    let unsure = supported(&d);
    assert!(noted(&unsure, "WARN_FOREIGN_TAX_CREDIT_NOT_CLAIMED"));
    assert!(!unsure.lines.iter().any(|l| l.id == "foreign_tax_credit" && l.amount > 0));
}

#[test]
fn farmland_installments_reduce_statement_credit_when_hypothetical_tax_is_zero() {
    let mut d = farmland_installment_return();
    let sale = farmland_resolved_sale(&d, 0);
    d.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    d.income[1].income = raised_farm(0, 0);
    set_farmland_sales(&mut d, vec![sale.clone(), sale]);
    farmland_statement_credit(&mut d);
    let r = supported(&d);
    // $81,000 income - $16,100 deduction; $15,450 above the zero-rate
    // band taxed at 15% gives $2,318. The actual $300 FTC is retained.
    assert_eq!(value(&r, "taxable_income"), 64900);
    assert_eq!(value(&r, "regular_income_tax"), 2318);
    assert_eq!(value(&r, "farm_installment_actual_foreign_credit"), 300);
    assert_eq!(value(&r, "farm_installment_reduced_foreign_credit"), 0);
    assert_eq!(r.tax, Some(201800));
    assert_eq!(
        r.farmland_installments,
        Some(FarmTaxInstallments {
            applicable: 201800,
            first: 50500,
            remaining: 151300,
            current: 50500,
        })
    );
    assert_eq!(return_line(&r, "20"), 30000);
    raise_payee_statement_tax(&mut d, 1);
    unsupported(&d, "Foreign tax credit"); // Threshold uses cents, not rounded dollars.
}

#[test]
fn farmland_installments_preserve_marketplace_repayment_and_relimit_credits() {
    let mut d = farmland_installment_return();
    farmland_marketplace_repayment(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 192935);
    assert_eq!(value(&r, "premium_credit_repayment"), 6000);
    assert_eq!(value(&r, "farm_installment_regular_additions"), 6000);
    assert_eq!(value(&r, "farm_installment_actual_regular_liability"), 32978);
    assert_eq!(value(&r, "farm_installment_reduced_regular_liability"), 26978);
    assert_eq!(r.tax, Some(4710800));
    assert_eq!(
        r.farmland_installments,
        Some(FarmTaxInstallments {
            applicable: 600000,
            first: 150000,
            remaining: 450000,
            current: 4260800,
        })
    );
    assert_eq!(return_line(&r, "17"), 600000);
    assert_eq!(return_line(&r, "18"), 3297800);
    d.credits.as_mut().unwrap().adoption_carry =
        vec![AdoptionCarry { year: 2025, amount: 3000000, eligible: true }];
    let r = supported(&d);
    // Repayment increases tax capacity in BOTH contexts before credit limits.
    assert_eq!(value(&r, "adoption_nonrefundable_credit"), 30000);
    assert_eq!(value(&r, "adoption_carryforward"), 0);
    assert_eq!(value(&r, "farm_installment_actual_credits"), 30000);
    assert_eq!(value(&r, "farm_installment_reduced_credits"), 26978);
    assert_eq!(r.tax, Some(1710800));
    assert_eq!(
        r.farmland_installments,
        Some(FarmTaxInstallments {
            applicable: 297800,
            first: 74500,
            remaining: 223300,
            current: 1487500,
        })
    );
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let actual = r;
    farmland_sales_mut(&mut d)[0].deferral = false;
    let ordinary = supported(&d);
    assert_eq!(ordinary.tax, actual.tax);
    assert_eq!(ordinary.payments, actual.payments);
    assert_eq!(ordinary.refundable_credits, actual.refundable_credits);
    assert_eq!(ordinary.balance.unwrap() - actual.balance.unwrap(), 223300);
}

#[test]
fn farmland_reduced_regular_liability_keeps_repayment_foreign_credit_capacity() {
    let mut d = farmland_installment_return();
    let sale = farmland_resolved_sale(&d, 0);
    d.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    d.income[1].income = raised_farm(0, 0);
    set_farmland_sales(&mut d, vec![sale.clone(), sale]);
    farmland_statement_credit(&mut d);
    farmland_marketplace_repayment(&mut d);
    let r = supported(&d);
    // The sale stays in household MAGI: $81,000 exceeds 400% of the poverty
    // line for this single filer. All $6,000 advance payments remain repayable.
    assert_eq!(value(&r, "marketplace_household_income"), 81000);
    assert_eq!(value(&r, "premium_credit_repayment"), 6000);
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 0);
    assert_eq!(value(&r, "farm_installment_reduced_regular_liability"), 6000);
    assert_eq!(value(&r, "farm_installment_reduced_foreign_credit"), 300);
    assert_eq!(r.tax, Some(801800));
    assert_eq!(
        r.farmland_installments,
        Some(FarmTaxInstallments {
            applicable: 231800,
            first: 58000,
            remaining: 173800,
            current: 628000,
        })
    );
    assert_eq!(return_line(&r, "17"), 600000);
    assert_eq!(return_line(&r, "20"), 30000);
    assert_eq!(return_line(&r, "24c"), 859800);
}

#[test]
fn farmland_installments_preserve_foreign_exclusion_stacking() {
    let mut d = farmland_installment_return();
    add(&mut d, income(foreign_pay(100000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 192935);
    assert_eq!(value(&r, "taxable_income"), 158248);
    // Ordinary tax on $218,248 is $46,295; tax on the excluded $100,000
    // is $16,712. Add $6,000 capital tax for the actual regular tax.
    assert_eq!(value(&r, "regular_income_tax"), 35583);
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 29583);
    assert_eq!(r.tax, Some(4971300));
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 600000);
    assert_eq!(supported(&restart(d.clone()).draft), r);

    d.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    let sale = farmland_resolved_sale(&d, 0);
    d.income[1].income = raised_farm(0, 0);
    set_farmland_sales(&mut d, vec![sale]);
    let r = supported(&d);
    // Foreign stacking leaves $23,900 taxable preference, all at 15%.
    assert_eq!(value(&r, "taxable_income"), 23900);
    assert_eq!(value(&r, "regular_income_tax"), 3585);
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 0);
    assert_eq!(
        r.farmland_installments,
        Some(FarmTaxInstallments {
            applicable: 358500,
            first: 89600,
            remaining: 268900,
            current: 89600,
        })
    );
    add(&mut d, income(qualified_dividend(70000)));
    let r = supported(&d);
    // Recompute the excess preference at each taxable-income level rather
    // than subtracting the sale from an already-capped preference pool.
    assert_eq!(value(&r, "taxable_income"), 93900);
    assert_eq!(value(&r, "regular_income_tax"), 14085);
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 8085);
    assert_eq!(value(&r, "net_investment_income_tax"), 380);
    assert_eq!(r.tax, Some(1446500));
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 600000);
}

#[test]
fn farmland_installments_reuse_schedule_j_election_and_retained_years() {
    let mut d = farmland_installment_return();
    d.screening.as_mut().unwrap().farm_averaging =
        averaging_return(60000).screening.unwrap().farm_averaging;
    let r = supported(&d);
    // Actual current Schedule J tax $13,527 ($7,527 ordinary + $6,000
    // capital), plus unchanged $7,101 across retained 2023–2025 years.
    assert_eq!(value(&r, "farm_averaging_elected_income"), 60000);
    assert_eq!(value(&r, "regular_income_tax"), 20623);
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 14623);
    assert_eq!(r.tax, Some(3475300));
    assert_eq!(
        r.farmland_installments,
        Some(FarmTaxInstallments {
            applicable: 600000,
            first: 150000,
            remaining: 450000,
            current: 3025300,
        })
    );
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let mut ordinary = d.clone();
    farmland_sales_mut(&mut ordinary)[0].deferral = false;
    let ordinary = supported(&ordinary);
    assert_eq!(ordinary.tax, r.tax);
    assert_eq!(ordinary.payments, r.payments);
    for year in 2023..=2025 {
        for suffix in ["base_income", "allocated_income", "base_tax", "recomputed_tax"] {
            let key = alloc::format!("farm_averaging_{year}_{suffix}");
            assert_eq!(value(&ordinary, &key), value(&r, &key));
        }
    }
    add(&mut d, income(foreign_pay(100000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "regular_income_tax"), 26967);
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 20967);
    assert_eq!(r.tax, Some(4109700));
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 600000);

    // The overall QBI cap leaves actual taxable income $101,468 and
    // hypothetical income $61,468. The comparison applies that ceiling to
    // the $62,000 election and reallocates $20,489 to each retained year;
    // the actual $62,000 election is preserved.
    d.income.retain(|entry| !matches!(entry.income, FederalIncome::ForeignEmployment { .. }));
    d.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    d.deductions.as_mut().unwrap().itemize_override = None;
    d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().elected = 6200000;
    let r = supported(&d);
    assert_eq!(value(&r, "farm_averaging_elected_income"), 62000);
    assert_eq!(value(&r, "farm_installment_averaging_elected_income"), 61468);
    // Each base has $10,000 income and filed tax $1,003. Rate-schedule
    // tax on $30,489: $3,439 / $3,427 / $3,420; total less $3,009 = $7,277.
    for (year, tax) in [(2023, 3439), (2024, 3427), (2025, 3420)] {
        assert_eq!(
            value(&r, &alloc::format!("farm_installment_averaging_{year}_allocated_income")),
            30489
        );
        assert_eq!(
            value(&r, &alloc::format!("farm_installment_averaging_{year}_recomputed_tax")),
            tax
        );
    }
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 7277);
    assert_eq!(value(&r, "regular_income_tax"), 7341);
    assert_eq!(r.tax, Some(2147100));
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 6400);
    assert_eq!(r.farmland_installments.as_ref().unwrap().first, 1600);
    assert_eq!(r.balance, Some(2142300));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    assert_eq!(d.screening.as_ref().unwrap().farm_averaging.as_ref().unwrap().elected, 6200000);
    for elected in [61467, 61468] {
        d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap().elected = elected * 100;
        let boundary = supported(&d);
        assert_eq!(value(&boundary, "farm_installment_averaging_elected_income"), elected);
        assert_eq!(value(&boundary, "farm_installment_reduced_regular_tax"), 7277);
    }
}

#[test]
fn farmland_installments_keep_amt_current_without_credit_coordination() {
    let mut d = farmland_installment_return();
    farmland_basis_mut(&mut d).amt = 0;
    farmland_basis_mut(&mut d).prior_amt_conservation = 0;
    let r = supported(&d);
    // AMTI 274,348 - exemption 90,100: ordinary 44,248 at 26%
    // plus capital 140,000 at 15% = TMT 32,504. Regular tax 26,978;
    // AMT 5,526 and SE tax 14,130 stay payable in full.
    assert_eq!(r.tax, Some(4663400));
    let installments = r.farmland_installments.as_ref().unwrap();
    assert_eq!(installments.applicable, 600000);
    assert_eq!(installments.first, 150000);
    assert_eq!(installments.remaining, 450000);
    assert_eq!(installments.current, 4213400);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    farmland_sales_mut(&mut d)[0].deferral = false;
    let ordinary = supported(&d);
    assert_eq!(ordinary.tax, r.tax);
    assert_eq!(ordinary.balance.unwrap() - r.balance.unwrap(), 450000);

    farmland_sales_mut(&mut d)[0].deferral = true;
    d.credits.as_mut().unwrap().adoption_carry =
        vec![AdoptionCarry { year: 2025, amount: 2500000, eligible: true }];
    let r = supported(&d);
    assert_eq!(r.tax, Some(2163400));
    assert_eq!(value(&r, "farm_installment_actual_credits"), 25000);
    assert_eq!(value(&r, "farm_installment_reduced_credits"), 20978);
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 197800);
    assert_eq!(r.farmland_installments.as_ref().unwrap().current, 2015100);
}

#[test]
fn farmland_installments_with_amt_cap_hypothetical_credits_but_preserve_actual_credits() {
    let mut d = farmland_installment_return();
    d.deductions.as_mut().unwrap().itemize_override = None;
    farmland_basis_mut(&mut d).amt = 0;
    farmland_basis_mut(&mut d).prior_amt_conservation = 0;
    d.credits.as_mut().unwrap().adoption_carry =
        vec![AdoptionCarry { year: 2025, amount: 2097800, eligible: true }];
    // The hypothetical regular liability is exactly $20,978; both contexts
    // absorb this entire credit independently of their AMT liability.
    let r = supported(&d);
    assert_eq!(value(&r, "farm_installment_actual_credits"), 20978);
    assert_eq!(value(&r, "farm_installment_reduced_credits"), 20978);
    assert_eq!(r.tax, Some(2565600));
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 600000);
    assert_eq!(r.farmland_installments.as_ref().unwrap().current, 2115600);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    // One extra credit dollar reduces actual net regular tax but the
    // reduced comparison has already exhausted line 12z capacity.
    d.credits.as_mut().unwrap().adoption_carry[0].amount += 100;
    let boundary = supported(&d);
    assert_eq!(boundary.farmland_installments.as_ref().unwrap().applicable, 599900);

    d.credits.as_mut().unwrap().adoption_carry.clear();
    farmland_statement_credit(&mut d);
    let r = supported(&d);
    // $1,000 bank income raises regular tax by $240 and TMT by $260.
    // The same $300 statement election reduces regular and AMT foreign tax.
    assert_eq!(r.tax, Some(4659400));
    assert_eq!(value(&r, "farm_installment_actual_foreign_credit"), 300);
    assert_eq!(value(&r, "farm_installment_reduced_foreign_credit"), 300);
    assert_eq!(r.farmland_installments.as_ref().unwrap().current, 4209400);
    d.credits.as_mut().unwrap().adoption_carry =
        vec![AdoptionCarry { year: 2025, amount: 2091800, eligible: true }];
    let r = supported(&d);
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 21218);
    assert_eq!(value(&r, "farm_installment_reduced_credits"), 21218);
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 600000);
    d.credits.as_mut().unwrap().adoption_carry[0].amount += 100;
    let boundary = supported(&d);
    assert_eq!(boundary.farmland_installments.as_ref().unwrap().applicable, 599900);
}

#[test]
fn farmland_credit_aggregate_cap_eliminates_hypothetical_amt_dependency() {
    use crate::income_tax::PersonalCreditPotentials;
    for tax in [0, 1, 500, 5000, 30000] {
        for amt in [0, 1, 3000, 50000] {
            for actc_limit in [0, 1700, 3400] {
                let potential = PersonalCreditPotentials {
                    care: 600,
                    elderly: 750,
                    education: 2500,
                    saver: 1000,
                    adoption: 15000,
                    child: 4400,
                    actc_limit,
                };
                // Form 1062 line 12z caps the aggregate, regardless of how
                // credit order or ACTC reservation distributes that aggregate.
                assert_eq!(
                    potential.limit(tax + amt).nonrefundable_total().min(tax),
                    potential.limit(tax).nonrefundable_total(),
                );
            }
        }
    }
    let mut d = farmland_installment_return();
    d.deductions.as_mut().unwrap().itemize_override = None;
    farmland_basis_mut(&mut d).amt = 0;
    farmland_basis_mut(&mut d).prior_amt_conservation = 0;
    // Part I can have negative net regular income tax: preserve actual
    // credits against AMT rather than artificially capping them to $26,978.
    for (carry, used, remaining, full) in [(30000, 30000, 0, 16634), (40000, 32504, 7496, 14130)] {
        d.credits.as_mut().unwrap().adoption_carry =
            vec![AdoptionCarry { year: 2025, amount: carry * 100, eligible: true }];
        let r = supported(&d);
        assert_eq!(value(&r, "farm_installment_actual_credits"), used);
        assert_eq!(value(&r, "farm_installment_reduced_credits"), 20978);
        assert_eq!(value(&r, "adoption_carryforward"), remaining);
        assert_eq!(r.tax, Some(full * 100));
        assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 0);
        assert_eq!(r.farmland_installments.as_ref().unwrap().current, full * 100);
    }
}

#[test]
fn farmland_installment_credit_excess_reconciles_payment_and_overpayment() {
    let mut d = farmland_installment_low_tax_credit(1000);
    // Form1062 excludes the1,000 section53 credit: applicable2,168,
    // first542, remaining1,626. Annual tax1,168 + first542 = line24c1,710.
    // Schedule3 includes2,168, so line33 minus line24c produces458 overpaid.
    let r = supported(&d);
    let installments = r.farmland_installments.as_ref().unwrap();
    assert_eq!(r.tax, Some(116800));
    assert_eq!(installments.applicable, 216800);
    assert_eq!(installments.first, 54200);
    assert_eq!(installments.remaining, 162600);
    assert_eq!(installments.current, -45800);
    assert_eq!(r.payments, Some(0)); // The deferral is not cash paid.
    assert_eq!(r.refundable_credits, Some(0));
    assert_eq!(return_line(&r, "24a"), 116800);
    assert_eq!(return_line(&r, "24b"), 54200);
    assert_eq!(return_line(&r, "24c"), 171000);
    assert_eq!(return_line(&r, "31"), 216800);
    assert_eq!(return_line(&r, "33"), 216800);
    assert_eq!(return_line(&r, "34"), 45800);
    assert_eq!(r.refund, Some(45800));
    assert_eq!(r.balance, Some(0));
    assert_eq!(supported(&restart(d.clone()).draft).refund, r.refund);
    d.payments.as_mut().unwrap().estimated =
        crate::tests::support::builders::unknown_estimated(20000, PaymentPayer::Taxpayer);
    let paid = supported(&d);
    assert_eq!(paid.tax, r.tax);
    assert_eq!(paid.farmland_installments, r.farmland_installments);
    assert_eq!(paid.payments, Some(20000));
    assert_eq!(return_line(&paid, "33"), 236800);
    assert_eq!(paid.refund, Some(65800));
    d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts.mtcnol =
        crate::MinimumTaxLoss::Unavailable;
    // Not sure leaves the prior-year credit unclaimed, with the assumption stated.
    let unsure = supported(&d);
    assert!(noted(&unsure, "WARN_MTC_NOL_LEFT_OUT"));
    assert!(unsure.tax >= paid.tax);
}

#[test]
fn farmland_installments_exclude_minimum_credit_at_zero_and_positive_capacity() {
    let mut d = farmland_installment_return();
    d.deductions.as_mut().unwrap().itemize_override = None;
    let mut history = minimum_tax_history();
    history.amt = 0;
    history.carry = 0;
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(history));
    let r = supported(&d);
    assert_eq!(value(&r, "minimum_tax_credit_available"), 0);
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 600000);
    // Positive minimum-credit capacity has no effect on Form1062.
    d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts.carry = 100;
    let r = supported(&d);
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 600000);
    assert_eq!(value(&r, "minimum_tax_credit"), 1);
    // Personal credits still participate and can exhaust reduced capacity.
    d.credits.as_mut().unwrap().adoption_carry =
        vec![AdoptionCarry { year: 2025, amount: 2500000, eligible: true }];
    let r = supported(&d);
    assert_eq!(value(&r, "farm_installment_reduced_regular_liability"), 20978);
    assert_eq!(value(&r, "farm_installment_reduced_credits"), 20978);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 1);
    assert_eq!(r.farmland_installments.as_ref().unwrap().applicable, 197800);
}

#[test]
fn farm_ccc_surplus_receipts_preserve_loan_history_and_receipt_year() {
    for (origin, method, year, ordinary, principal, advance, expected) in [
        (CccOrigin::Current, CccMethod::Loan, 2026, 43300, 5200, 0, 6700),
        (CccOrigin::Current, CccMethod::Income, 2026, 43300, 0, 5200, 6700),
        (CccOrigin::PriorIncluded, CccMethod::Income, 2026, 48500, 0, 0, 1500),
        (CccOrigin::PriorExcluded, CccMethod::Loan, 2026, 43300, 5200, 0, 6700),
        (CccOrigin::PriorIncluded, CccMethod::Income, 2025, 48500, 0, 0, 1500),
        (CccOrigin::PriorExcluded, CccMethod::Loan, 2025, 48500, 0, 0, 1500),
    ] {
        let d = ccc_return(
            origin,
            method,
            CccDisposition::Forfeited { year, surplus: 150000, deficiency: 0 },
            ordinary,
        );
        let r = ccc_fifty_thousand(&d);
        assert_eq!(ccc_value(&d, &r, "forfeiture"), principal);
        assert_eq!(ccc_value(&d, &r, "advances"), advance);
        assert_eq!(ccc_value(&d, &r, "surplus"), 1500);
        assert_eq!(ccc_value(&d, &r, "income"), expected);
        assert_eq!(ccc_value(&d, &r, "retained_basis"), 0);
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn farm_ccc_forfeiture_release_is_not_a_loss_or_repeat_of_prior_income() {
    // No current payment after an earlier forfeiture: neither the principal
    // nor a presumed CCC resale shortfall creates current income or a loss.
    let d = ccc_return(
        CccOrigin::PriorIncluded,
        CccMethod::Income,
        CccDisposition::Forfeited { year: 2025, surplus: 0, deficiency: 0 },
        50000,
    );
    let r = ccc_fifty_thousand(&d);
    assert_eq!(ccc_value(&d, &r, "income"), 0);
    let mut bad = d.clone();
    ccc_mut(&mut bad).lots[0].origin = CccOrigin::Current;
    unsupported(&bad, "CCC loan settlement");
    for (year, surplus) in [(2027, 0), (1899, 0)] {
        let mut bad = d.clone();
        ccc_mut(&mut bad).lots[0].disposition =
            CccDisposition::Forfeited { year, surplus, deficiency: 0 };
        assert!(!bad.valid_stored());
    }
}

#[test]
fn farm_ccc_required_paid_deficiencies_flow_through_expenses_once() {
    for (origin, year, ordinary, advance) in [
        (CccOrigin::Current, 2026, 45100, 5200),
        (CccOrigin::PriorIncluded, 2026, 50300, 0),
        (CccOrigin::PriorIncluded, 2025, 50300, 0),
    ] {
        let d = ccc_return(
            origin,
            CccMethod::Income,
            CccDisposition::Forfeited { year, surplus: 0, deficiency: 30000 },
            ordinary,
        );
        let r = ccc_fifty_thousand(&d);
        assert_eq!(ccc_value(&d, &r, "income"), advance);
        assert_eq!(ccc_value(&d, &r, "paid_deficiency"), 300);
        assert_eq!(value(&r, &alloc::format!("farm_{}_expenses", d.income[0].id)), 10300);
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn farm_ccc_regulation_example_keeps_surplus_and_paid_losses_separate() {
    let mut d =
        ccc_return(CccOrigin::PriorIncluded, CccMethod::Income, CccDisposition::Outstanding, 50150);
    // 26 CFR 1.77-2(b): earlier income-included loans 500/700/900,
    // CCC shortfalls 100/200 and surplus 150. Only required paid shortfalls
    // are deducted; the surplus received is income in its own right.
    ccc_mut(&mut d).lots = [(500, 100, 0), (700, 200, 0), (900, 0, 150)]
        .into_iter()
        .map(|(principal, deficiency, surplus)| CccLot {
            id: 0,
            origin: CccOrigin::PriorIncluded,
            principal: principal * 100,
            disposition: CccDisposition::Forfeited {
                year: 2026,
                surplus: surplus * 100,
                deficiency: deficiency * 100,
            },
        })
        .collect();
    let r = ccc_fifty_thousand(&d);
    assert_eq!(ccc_value(&d, &r, "income"), 150);
    assert_eq!(ccc_value(&d, &r, "paid_deficiency"), 300);
    let mut bad = d.clone();
    ccc_mut(&mut bad).lots[0].origin = CccOrigin::PriorExcluded;
    unsupported(&bad, "CCC paid deficiency");
    {
        let amount = 50001;
        let mut bad = d.clone();
        ccc_mut(&mut bad).lots[0].disposition =
            CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: amount };
        unsupported(&bad, "CCC paid deficiency");
    }
}

#[test]
fn farm_ccc_identified_partial_release_preserves_outstanding_principal_and_basis() {
    for (origin, method, expected, advance, taxable_market, basis) in [
        (CccOrigin::Current, CccMethod::Income, 6450, 5200, 0, 2600),
        (CccOrigin::Current, CccMethod::Loan, 3850, 0, 600, 0),
        (CccOrigin::PriorIncluded, CccMethod::Income, 1250, 0, 0, 2600),
        (CccOrigin::PriorExcluded, CccMethod::Loan, 3850, 0, 600, 0),
    ] {
        let mut d =
            ccc_return(origin.clone(), method, CccDisposition::Outstanding, 50000 - expected);
        // One $5,200 advance covers two identified $2,600 collateral parts.
        // Only one is redeemed for $2,000 and sold for $3,250 this year.
        ccc_mut(&mut d).lots = vec![
            CccLot {
                id: 0,
                origin: origin.clone(),
                principal: 260000,
                disposition: CccDisposition::Redeemed {
                    year: 2026,
                    paid: 200000,
                    sale: Some(325000),
                },
            },
            CccLot { id: 0, origin, principal: 260000, disposition: CccDisposition::Outstanding },
        ];
        let r = ccc_fifty_thousand(&d);
        assert_eq!(ccc_value(&d, &r, "income"), expected);
        assert_eq!(ccc_value(&d, &r, "advances"), advance);
        assert_eq!(ccc_value(&d, &r, "market_gain"), 600);
        assert_eq!(ccc_value(&d, &r, "taxable_market_gain"), taxable_market);
        assert_eq!(ccc_value(&d, &r, "retained_basis"), basis);
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn farm_ccc_separately_redeemed_portions_reconcile_to_the_complete_loan() {
    for (origin, method, expected) in [
        (CccOrigin::Current, CccMethod::Income, 7700),
        (CccOrigin::Current, CccMethod::Loan, 7700),
        (CccOrigin::PriorIncluded, CccMethod::Income, 2500),
        (CccOrigin::PriorExcluded, CccMethod::Loan, 7700),
    ] {
        let whole = ccc_return(
            origin.clone(),
            method,
            CccDisposition::Redeemed { year: 2026, paid: 400000, sale: Some(650000) },
            50000 - expected,
        );
        let whole_result = ccc_fifty_thousand(&whole);
        let mut portions = whole.clone();
        ccc_mut(&mut portions).lots = [(180000, 300000), (220000, 350000)]
            .into_iter()
            .map(|(paid, sale)| CccLot {
                id: 0,
                origin: origin.clone(),
                principal: 260000,
                disposition: CccDisposition::Redeemed { year: 2026, paid, sale: Some(sale) },
            })
            .collect();
        let r = ccc_fifty_thousand(&portions);
        // Different release prices must retain their own market-gain/basis
        // computations, yet reconcile to the complete source amounts.
        for name in [
            "advances",
            "market_gain",
            "taxable_market_gain",
            "sales_net",
            "retained_basis",
            "income",
        ] {
            assert_eq!(ccc_value(&portions, &r, name), ccc_value(&whole, &whole_result, name));
        }
        assert_eq!(r.tax, whole_result.tax);
        assert_eq!(r.balance, whole_result.balance);
        assert_eq!(r.refundable_credits, whole_result.refundable_credits);
    }
}

#[test]
fn farm_ccc_accrual_loan_method_uses_inventory_once_and_keeps_pledged_crops() {
    for (origin, disposition, other_sales, opening, closing, ccc_income) in [
        (CccOrigin::Current, CccDisposition::Outstanding, 60000, 4000, 4000, 0),
        (CccOrigin::Current, CccDisposition::Outstanding, 56000, 0, 4000, 0),
        (
            CccOrigin::Current,
            CccDisposition::Redeemed { year: 2026, paid: 400000, sale: None },
            58800,
            4000,
            4000,
            1200,
        ),
        (
            CccOrigin::Current,
            CccDisposition::Redeemed { year: 2026, paid: 400000, sale: Some(650000) },
            56300,
            4000,
            0,
            7700,
        ),
        (
            CccOrigin::PriorExcluded,
            CccDisposition::Redeemed { year: 2025, paid: 400000, sale: Some(650000) },
            57500,
            4000,
            0,
            6500,
        ),
        (
            CccOrigin::PriorExcluded,
            CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 0 },
            58800,
            4000,
            0,
            5200,
        ),
    ] {
        let mut d = ccc_return(origin, CccMethod::Loan, disposition, 0);
        let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
        let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
        data.accounting = FarmAccounting::Accrual {
            sales: other_sales * 100,
            opening: opening * 100,
            closing: closing * 100,
            purchases: 0,
            custom_hire: 0,
            program: 0,
            other: 0,
        };
        let r = ccc_fifty_thousand(&d);
        assert_eq!(ccc_value(&d, &r, "income"), ccc_income);
        assert_eq!(
            value(&r, &alloc::format!("farm_{}_opening_inventory", d.income[0].id)),
            opening
        );
        assert_eq!(
            value(&r, &alloc::format!("farm_{}_closing_inventory", d.income[0].id)),
            closing
        );
        assert!(!r.lines.iter().any(|line| line.id == alloc::format!("farm_{}_ccc_retained_basis",d.income[0].id)));
        assert_eq!(supported(&restart(d.clone()).draft), r);
        ccc_mut(&mut d).method = CccMethod::Income;
        unsupported(&d, "CCC accrual inventory");
    }
}

#[test]
fn farm_ccc_reports_gross_forfeiture_even_when_principal_is_not_taxable() {
    let d = ccc_return(
        CccOrigin::PriorIncluded,
        CccMethod::Income,
        CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 0 },
        50000,
    );
    let r = ccc_fifty_thousand(&d);
    assert_eq!(ccc_value(&d, &r, "forfeiture_reported"), 5200);
    assert_eq!(ccc_value(&d, &r, "forfeiture"), 0);
    assert_eq!(ccc_value(&d, &r, "income"), 0);
}
#[test]
fn farm_prepaid_supplies_limit_reconciles_irs_example_and_full_return() {
    // IRS example: $55k paid, $100k other expenses => $50k deduction/$5k deferred.
    for (paid, deduction, remaining) in
        [(55000, 50000, 5000), (50000, 50000, 0), (49000, 49000, 0), (0, 0, 0)]
    {
        let d = farm_supply_return(farm_supplies(paid), 100000, 150000 + deduction);
        let r = farm_supply_fifty_thousand(&d);
        assert_eq!(farm_supply_value(&d, &r, "deduction"), deduction);
        assert_eq!(farm_supply_value(&d, &r, "remaining"), remaining);
    }
    let d = farm_supply_return(farm_supplies(5000), 0, 50000);
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 0);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 5000);
}
#[test]
fn farm_prepaid_history_exception_uses_strict_three_year_ratio() {
    for (prepaid, allowed) in [(149999, 55000), (150000, 50000), (150001, 50000)] {
        let mut supplies = farm_supplies(55000);
        supplies.exception = FarmSupplyException::History {
            relation: FarmSupplyRelation::Business,
            years: (2023..=2025)
                .map(|year| FarmSupplyYear {
                    year,
                    prepaid: if year == 2023 { prepaid * 100 } else { 0 },
                    other: 10000000,
                })
                .collect(),
        };
        let d = farm_supply_return(supplies, 100000, 150000 + allowed);
        let r = farm_supply_fifty_thousand(&d);
        assert_eq!(farm_supply_value(&d, &r, "deduction"), allowed);
        assert_eq!(farm_supply_value(&d, &r, "remaining"), 55000 - allowed);
    }
    let mut s = farm_supplies(1);
    s.exception = FarmSupplyException::History {
        relation: FarmSupplyRelation::Home,
        years: vec![FarmSupplyYear { year: 2023, prepaid: 0, other: 100 }; 3],
    };
    assert!(!s.valid());
}
#[test]
fn farm_prepaid_extraordinary_exception_lifts_the_ceiling() {
    // Choosing the extraordinary-circumstances exception states the
    // business-operation change; the whole prepayment is deducted.
    let mut s = farm_supplies(55000);
    s.exception = FarmSupplyException::Extraordinary { relation: FarmSupplyRelation::Family };
    let d = farm_supply_return(s, 100000, 150000 + 55000);
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 55000);
}
#[test]
fn farm_prepaid_retained_cost_releases_once_and_increases_other_expense_base() {
    let mut s = farm_supplies(7000);
    s.retained = vec![
        FarmSupplyCarry { balance: 200000, amt_balance: 200000, consumed: true },
        FarmSupplyCarry { balance: 300000, amt_balance: 300000, consumed: false },
    ];
    // $10k ordinary + $2k old consumed costs => $6k ceiling; deduction $8k.
    let d = farm_supply_return(s, 10000, 68000);
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "released"), 2000);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 8000);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 4000);
}
#[test]
fn farm_prepaid_incomplete_records_are_left_out_and_unresolved_coordination_blocks_totals() {
    let base = farm_supply_return(farm_supplies(5000), 10000, 65000);
    // Unestablished current purchase terms omit only this deduction and keep
    // the actual undeducted payment for future consumption.
    let mut d = base.clone();
    farm_supply_mut(&mut d).advances[0].purchase.kind = FarmSupplyPaymentKind::UnresolvedTerms;
    let r = supported(&d);
    assert!(noted(&r, "WARN_FARM_SUPPLIES_LEFT_OUT"));
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 0);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 5000);
    assert!(!noted(&supported(&base), "WARN_FARM_SUPPLIES_LEFT_OUT"));
    let mut d = base.clone();
    add(&mut d, income(raised_farm(10000, 1000)));
    assert_eq!(return_line(&supported(&d), "8"), 5900000);
    let mut d = base;
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.accounting = FarmAccounting::Accrual {
        sales: 6500000,
        opening: 0,
        closing: 0,
        purchases: 0,
        custom_hire: 0,
        program: 0,
        other: 0,
    };
    unsupported(&d, "Farm prepaid accounting");
}

#[test]
fn farm_prepaid_cap_includes_depreciation_and_valid_pre_2018_farm_method() {
    // $10k ordinary + $10k bonus depreciation supports a $10k prepayment.
    let mut d = farm_supply_return(farm_supplies(11000), 10000, 80000);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.assets = vec![bonus_equipment(10000, 100)];
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 10000);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 1000);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.assets = vec![separate_amt_equipment()];
    // Farm property before 2018 uses 150% or straight-line GDS, rather than the
    // generic business fixture's 200% method. Use actual matching retained
    // 150% schedules: remaining 131,092 / 1.5 = 87,395 current cents.
    data.assets[0].method = EquipmentMethod::Declining150;
    if let EquipmentHistory::Continuing { prior_depreciation, amt, .. } =
        &mut data.assets[0].history
    {
        *prior_depreciation = 868_908;
        *amt = EquipmentAmtHistory::Same;
    }
    let r = supported(&d);
    assert_eq!(crate::farm_supplies::shared_amounts(&d, d.income[0].id).unwrap()[2], 543_697);
    assert_eq!(
        crate::farm_supplies::shared_amounts_for(&d, d.income[0].id, true).unwrap()[2],
        543_697
    );
    assert_eq!(value(&r, "amt_farm_supplies_adjustment"), 0);
    assert_eq!(value(&r, "amt_business_depreciation_adjustment"), 0);
}
#[test]
fn farm_prepaid_exception_coordinates_with_shared_section179_expenses() {
    let mut s = farm_supplies(10000);
    s.exception = FarmSupplyException::Extraordinary { relation: FarmSupplyRelation::Business };
    let mut d = farm_supply_return(s, 10000, 80000);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.assets = vec![elected_asset(10000, 4000, BonusElection::Apply)];
    farm_supply_fifty_thousand(&d);
    farm_supply_mut(&mut d).exception = FarmSupplyException::None;
    unsupported(&d, "Farm prepaid expense coordination");
}

#[test]
fn farm_prepaid_ceiling_includes_simplified_office_without_double_deduction() {
    let mut d = farm_supply_return(farm_supplies(8000), 10000, 67250);
    let source = d.income[0].id;
    add_simplified_office(&mut d, source, 300);

    // Other expense10,000 + office1,500 => supply ceiling5,750.
    // Gross67,250 -10,000 -5,750 -1,500 =>50,000 profit.
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 5750);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 2250);
    assert_eq!(value(&r, &alloc::format!("business_{}_home_office", d.income[0].id)), 1500);
}
#[test]
fn farm_prepaid_and_office_limits_bind_together_at_exact_cent_boundaries() {
    for (receipts, deduction, office) in
        [(15000, 5000, 0), (15999, 5333, 666), (16650, 5550, 1100), (17250, 5750, 1500)]
    {
        let mut d = farm_supply_return(farm_supplies(8000), 10000, receipts);
        let source = d.income[0].id;
        add_simplified_office(&mut d, source, 300);

        let r = supported(&d);
        assert_eq!(farm_supply_value(&d, &r, "deduction"), deduction);
        assert_eq!(farm_supply_value(&d, &r, "remaining"), 8000 - deduction);
        assert_eq!(value(&r, &alloc::format!("business_{}_home_office", d.income[0].id)), office);
        assert_eq!(return_line(&r, "8"), 0);
        assert_eq!(r.tax, Some(0));
        assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    }
    // Whole cents, not a convergence tolerance: 15,999.01/3 =>5,333.00.
    let mut d = farm_supply_return(farm_supplies(8000), 10000, 15999);
    let source = d.income[0].id;
    add_simplified_office(&mut d, source, 300);

    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    let FarmAccounting::Cash { receipts } = &mut data.accounting else { unreachable!() };
    receipts[0] = FarmReceipt::Raised { amount: 1599901 };
    let r = supported(&d);
    let id = d.income[0].id;
    assert_eq!(line_amount(&r, &alloc::format!("farm_{id}_supplies_deduction")), 533300);
    assert_eq!(line_amount(&r, &alloc::format!("business_{id}_home_office")), 66600);
}
#[test]
fn farm_prepaid_retained_release_participates_in_office_and_supply_limits_once() {
    let mut s = farm_supplies(8000);
    s.retained = vec![
        FarmSupplyCarry { balance: 200000, amt_balance: 200000, consumed: true },
        FarmSupplyCarry { balance: 300000, amt_balance: 300000, consumed: false },
    ];
    let mut d = farm_supply_return(s, 10000, 70250);
    let source = d.income[0].id;
    add_simplified_office(&mut d, source, 300);

    // Other10k + released2k + office1.5k =>6.75k new plus2k old.
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 8750);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 4250);
}

#[test]
fn farm_prepaid_ceiling_uses_final_conservation_adjustment_before_limiting() {
    let mut d = conservation_final_return();
    d.income.remove(0);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.accounting =
        FarmAccounting::Cash { receipts: vec![FarmReceipt::Raised { amount: 8094500 }] };
    data.supplies = Some(farm_supplies(12000));
    // Actual conservation630 (not ordinary annual70) + expenses20k
    // =>10,315 ceiling; total costs30,945, gross80,945, profit50k.
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 10315);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 1685);
    assert_eq!(value(&r, &alloc::format!("farm_{}_conservation_deduction", d.income[0].id)), 630);
}

#[test]
fn farm_prepaid_office_does_not_assume_land_disposition_attribution() {
    let mut d = farm_supply_return(farm_supplies(8000), 10000, 15999);
    d.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let source = d.income[0].id;
    add_simplified_office(&mut d, source, 300);

    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    set_land_sales(data, vec![farm_land_sale(20210615, 20260616)], &mut d.next_id);
    unsupported(&d, "Farm prepaid home-office disposition");
}
#[test]
fn farm_poultry_current_resale_cost_distinguishes_held_and_disposed_lots() {
    for (disposed, deduction, remaining, sold_cost) in
        [(false, 5000, 7000, 0), (true, 12000, 0, 12000)]
    {
        let mut s = farm_supplies(0);
        s.advances.clear();
        s.poultry = vec![poultry_resale(2026, 12000, 0, disposed)];
        let d = farm_supply_return(s, 10000, 60000 + deduction);
        let r = farm_supply_fifty_thousand(&d);
        assert_eq!(farm_supply_value(&d, &r, "deduction"), deduction);
        assert_eq!(farm_supply_value(&d, &r, "remaining"), remaining);
        assert_eq!(farm_supply_value(&d, &r, "poultry_disposed_cost"), sold_cost);
    }
}
#[test]
fn farm_poultry_retained_resale_cost_never_repeats_prior_deductions() {
    for (deducted, disposed, allowed, remaining) in
        [(5000, false, 0, 7000), (5000, true, 7000, 0), (12000, true, 0, 0), (0, true, 12000, 0)]
    {
        let mut s = farm_supplies(0);
        s.advances.clear();
        s.poultry = vec![poultry_resale(2025, 12000, deducted, disposed)];
        let d = farm_supply_return(s, 10000, 60000 + allowed);
        let r = farm_supply_fifty_thousand(&d);
        assert_eq!(farm_supply_value(&d, &r, "deduction"), allowed);
        assert_eq!(farm_supply_value(&d, &r, "released"), allowed);
        assert_eq!(farm_supply_value(&d, &r, "remaining"), remaining);
    }
}
#[test]
fn farm_poultry_costs_share_one_supply_ceiling_and_expense_base() {
    // Current disposed12k is an ordinary current cost: other10k+12k =>11k limit.
    // Prior disposed7k has the same role: other10k+7k =>8.5k limit.
    for (year, prior, deduction, remaining) in [(2026, 0, 23000, 1000), (2025, 5000, 15500, 3500)] {
        let mut s = farm_supplies(12000);
        s.poultry = vec![poultry_resale(year, 12000, prior, true)];
        let d = farm_supply_return(s, 10000, 60000 + deduction);
        let r = farm_supply_fifty_thousand(&d);
        assert_eq!(farm_supply_value(&d, &r, "deduction"), deduction);
        assert_eq!(farm_supply_value(&d, &r, "remaining"), remaining);
    }
    let mut s = farm_supplies(6000);
    s.poultry = vec![poultry_resale(2026, 6000, 0, false)];
    let d = farm_supply_return(s, 10000, 65000);
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "eligible"), 12000);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 5000);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 7000);
}
#[test]
fn farm_poultry_resale_exception_and_shared_limit_remain_distinct() {
    let mut s = farm_supplies(0);
    s.advances.clear();
    s.poultry = vec![poultry_resale(2026, 12000, 0, false)];
    s.exception = FarmSupplyException::Extraordinary { relation: FarmSupplyRelation::Business };
    let mut d = farm_supply_return(s, 10000, 72000);
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 12000);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 0);
    farm_supply_mut(&mut d).exception = FarmSupplyException::None;
    add(&mut d, income(raised_farm(1000, 0)));
    let r = supported(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 5000);
    assert_eq!(return_line(&r, "8"), 5800000);
}
#[test]
fn farm_poultry_resale_requires_reconciled_basis() {
    let mut s = farm_supplies(0);
    s.poultry = vec![poultry_resale(2025, 12000, 5000, false)];
    s.poultry[0].year = 2026;
    assert!(!s.valid());
    s.poultry[0].year = 2025;
    s.poultry[0].deducted = 1200001;
    assert!(!s.valid());
}

#[test]
fn farm_poultry_sale_method_defers_held_cost_even_with_prepaid_exception() {
    for year in [2025, 2026] {
        for disposed in [false, true] {
            let mut s = farm_supplies(0);
            s.advances.clear();
            let mut lot = poultry_resale(year, 12000, 0, disposed);
            lot.method = FarmPoultryMethod::Sold;
            s.poultry = vec![lot];
            s.exception =
                FarmSupplyException::Extraordinary { relation: FarmSupplyRelation::Business };
            let deduction = if disposed { 12000 } else { 0 };
            let d = farm_supply_return(s, 10000, 60000 + deduction);
            let r = farm_supply_fifty_thousand(&d);
            assert_eq!(farm_supply_value(&d, &r, "eligible"), 0);
            assert_eq!(farm_supply_value(&d, &r, "deduction"), deduction);
            assert_eq!(farm_supply_value(&d, &r, "remaining"), 12000 - deduction);
        }
    }
}
#[test]
fn farm_poultry_sale_method_held_cost_cannot_enlarge_feed_deduction() {
    let mut s = farm_supplies(12000);
    let mut lot = poultry_resale(2026, 12000, 0, false);
    lot.method = FarmPoultryMethod::Sold;
    s.poultry = vec![lot];
    // Held resale basis is neither a current expense nor eligible new prepayment.
    // Other10k permits only5k feed; remaining7k feed+12k birds=19k.
    let d = farm_supply_return(s, 10000, 65000);
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 5000);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 19000);
}
#[test]
fn farm_poultry_cost_methods_cannot_mix_or_change_silently() {
    let mut s = farm_supplies(0);
    s.poultry = vec![poultry_resale(2026, 12000, 0, false)];
    s.poultry[0].method = FarmPoultryMethod::Sold;
    s.poultry.push(poultry_resale(2025, 12000, 5000, false));
    unsupported(&farm_supply_return(s.clone(), 10000, 65000), "Poultry cost method");
    s.poultry[1].method = FarmPoultryMethod::Sold;
    assert!(!s.valid()); // An identified held sale-method portion cannot have been expensed before sale.
}
#[test]
fn farm_production_poultry_prior_deferral_releases_without_sale_or_repeat_cost() {
    for (paid, deducted, release) in [(12000, 4000, 8000), (12000, 12000, 0), (12000, 0, 12000)] {
        let mut s = farm_supplies(0);
        s.advances.clear();
        s.production_carry = vec![production_poultry_carry(paid, deducted)];
        let d = farm_supply_return(s, 10000, 60000 + release);
        let r = farm_supply_fifty_thousand(&d);
        assert_eq!(farm_supply_value(&d, &r, "production_poultry_released"), release);
        assert_eq!(farm_supply_value(&d, &r, "released"), release);
        assert_eq!(farm_supply_value(&d, &r, "deduction"), release);
        assert_eq!(farm_supply_value(&d, &r, "remaining"), 0);
    }
}
#[test]
fn farm_production_poultry_release_enters_other_expenses_before_new_ceiling() {
    let mut s = farm_supplies(10000);
    s.production_carry = vec![production_poultry_carry(12000, 4000)];
    // Other10k + released8k permits9k new prepayment. Total deduction17k.
    let d = farm_supply_return(s, 10000, 77000);
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 17000);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 1000);
}
#[test]
fn farm_production_poultry_release_coordinates_with_simplified_office() {
    let mut s = farm_supplies(10000);
    s.production_carry = vec![production_poultry_carry(12000, 4000)];
    let mut d = farm_supply_return(s, 10000, 79250);
    let source = d.income[0].id;
    add_simplified_office(&mut d, source, 300);

    // Other10k + release8k + office1.5k =>9.75k new +8k old.
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 17750);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 250);
}
#[test]
fn farm_production_poultry_prior_records_require_reconciled_basis() {
    let mut s = farm_supplies(0);
    s.production_carry = vec![production_poultry_carry(12000, 4000)];
    s.production_carry[0].deducted = 1200001;
    assert!(!s.valid());
}
#[test]
fn farm_prepaid_single_cost_farm_uses_taxpayers_other_farm_expenses() {
    let mut d = farm_supply_return(farm_supplies(20000), 10000, 70000);
    add(&mut d, income(raised_farm(10000, 10000)));
    // Total ordinary expenses20k =>10k supply ceiling. First farm50k, other0.
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 10000);
    assert_eq!(farm_supply_value(&d, &r, "remaining"), 10000);
    assert_eq!(crate::interview::document_amount(&d.income[0].income, false), None);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}
#[test]
fn farm_prepaid_multiple_cost_farms_take_all_when_shared_limit_does_not_bind() {
    let mut d = farm_supply_return(farm_supplies(6000), 10000, 41000);
    add_supply_farm(&mut d, farm_supplies(4000), 10000, 39000);
    // 10k prepayments exactly meet half of20k other expenses; profits25k each.
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 6000);
    assert_eq!(value(&r, &alloc::format!("farm_{}_supplies_deduction", d.income[1].id)), 4000);
    assert_eq!(value(&r, &alloc::format!("farm_{}_supplies_remaining", d.income[1].id)), 0);
}
#[test]
fn farm_prepaid_shared_limit_includes_other_farms_simplified_office() {
    let mut d = farm_supply_return(farm_supplies(20000), 10000, 70750);
    add(&mut d, income(raised_farm(11500, 10000)));
    let source = d.income.last().unwrap().id;
    add_simplified_office(&mut d, source, 300);
    // Other farm's full1,500 office raises shared cap to10,750; profits50k/0.
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 10750);
    assert_eq!(value(&r, &alloc::format!("business_{}_home_office", d.income[1].id)), 1500);
}
#[test]
fn farm_prepaid_shared_limit_preserves_other_farms_released_cost_owner() {
    let mut d = farm_supply_return(farm_supplies(20000), 10000, 71000);
    let mut other = farm_supplies(0);
    other.advances.clear();
    other.retained = vec![FarmSupplyCarry { balance: 200000, amt_balance: 200000, consumed: true }];
    add_supply_farm(&mut d, other, 10000, 12000);
    // Shared other expense22k =>11k new supplies, while2k old stays in farm2.
    let r = farm_supply_fifty_thousand(&d);
    assert_eq!(farm_supply_value(&d, &r, "deduction"), 11000);
    assert_eq!(value(&r, &alloc::format!("farm_{}_supplies_deduction", d.income[1].id)), 2000);
    assert_eq!(value(&r, &alloc::format!("farm_{}_supplies_remaining", d.income[1].id)), 0);
}
#[test]
fn farm_prepaid_binding_multi_cost_is_proportional_and_joint_owner_is_combined() {
    let mut d = farm_supply_return(farm_supplies(6000), 10000, 41000);
    add_supply_farm(&mut d, farm_supplies(6000), 10000, 41000);
    let r = supported(&d);
    // 20,000 other expenses permit 10,000 out of 12,000 unused supplies.
    // Equal 6,000 purchases receive 5,000 each, retaining 1,000 each.
    for e in &d.income {
        assert_eq!(value(&r, &alloc::format!("farm_{}_supplies_deduction", e.id)), 5000);
        assert_eq!(value(&r, &alloc::format!("farm_{}_supplies_remaining", e.id)), 1000);
    }
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
    // A joint return pools both spouses' farming expenses.
    let mut d = farm_supply_return(farm_supplies(6000), 10000, 41000);
    add_supply_farm(&mut d, farm_supplies(3000), 10000, 38000);
    supported(&d);
    let mut joint = complete(FederalStatus::Joint);
    joint.next_id = d.next_id;
    joint.income = d.income.clone();
    joint.income[1].owner = crate::DocumentOwner::Spouses;
    let r = supported(&joint);
    assert_eq!(value(&r, &alloc::format!("farm_{}_supplies_deduction", joint.income[0].id)), 6000);
    assert_eq!(value(&r, &alloc::format!("farm_{}_supplies_deduction", joint.income[1].id)), 3000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data } = &mut data.activity else { unreachable!() };
    data.specials.limitations = vec![3];
    let result = estimate(&d);
    assert_eq!(result.tax, None);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_FARM_METHOD_TRANSITION_SOURCE")
    );
}

#[test]
fn farmland_minimum_credit_preserves_actual_tax_and_statutory_deferral_separately() {
    let mut d = farmland_installment_return();
    d.deductions.as_mut().unwrap().itemize_override = None;
    let before = supported(&d);
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let after = supported(&d);
    // AGI192,935 less QBI18,587 gives AMTI174,348. After90,100
    // exemption,44,248 at26% plus40,000 at15% yields TMT17,504.
    // Regular26,978 less17,504 allows9,474 of the15,000 balance.
    assert_eq!(value(&after, "minimum_tax_credit_available"), 15000);
    assert_eq!(value(&after, "minimum_tax_credit"), 9474);
    assert_eq!(value(&after, "minimum_tax_credit_carryforward"), 5526);
    assert_eq!(value(&after, "farm_installment_actual_credits"), 0);
    assert_eq!(value(&after, "farm_installment_reduced_credits"), 0);
    let first = before.farmland_installments.as_ref().unwrap();
    let second = after.farmland_installments.as_ref().unwrap();
    assert_eq!(second.applicable, 600000);
    assert_eq!(second.first, 150000);
    assert_eq!(second.remaining, 450000);
    assert_eq!(first.applicable, second.applicable);
    assert_eq!(first.remaining, second.remaining);
    assert_eq!(before.tax.unwrap() - after.tax.unwrap(), 947400);
    assert_eq!(first.current - second.current, 947400);
    assert_eq!(supported(&restart(d).draft).refund, after.refund);
}

#[test]
fn farmland_installment_offset_crosses_zero_without_capping_or_duplicate_credit() {
    for (carry, tax, net, credit_left) in [
        (0, 2168, 542, 0),
        (542, 1626, 0, 0),
        (543, 1625, -1, 0),
        (1000, 1168, -458, 0),
        (5000, 0, -1626, 2832),
    ] {
        let d = farmland_installment_low_tax_credit(carry);
        let r = supported(&d);
        let schedule = r.farmland_installments.as_ref().unwrap();
        assert_eq!(r.tax, Some(tax * 100));
        assert_eq!(schedule.applicable, 216800);
        assert_eq!(schedule.first, 54200);
        assert_eq!(schedule.remaining, 162600);
        assert_eq!(schedule.current, net * 100);
        assert_eq!(value(&r, "minimum_tax_credit_carryforward"), credit_left);
        assert_eq!(r.refund, Some((-net).max(0) * 100));
        assert_eq!(r.balance, Some(net.max(0) * 100));
        assert_eq!(return_line(&r, "24c") - return_line(&r, "33"), net * 100);
        assert_eq!(r.payments, Some(0));
        assert_eq!(r.refundable_credits, Some(0));
    }
}

#[test]
fn farmland_averaging_zero_comparison_has_no_rounding_residual_or_source_mutation() {
    let d = averaging_return(60000);
    let mut r = supported(&d);
    let original_lines = r.lines.clone();
    let facts = d.screening.as_ref().unwrap().farm_averaging.as_ref();
    let result = crate::farm_averaging::calculate(
        facts,
        crate::farm_averaging::Current {
            status: FederalStatus::Single,
            taxable: 100000,
            regular_tax: 20000,
            preference: crate::income_tax::Preference {
                dividends: 0,
                gain: 0,
                line18: 0,
                line19: 0,
            },
            eligible: 100000,
            excluded: 0,
            foreign_election: false,
            shared_history_required: false,
            scope_ready: true,
        },
        &mut r,
    );
    let comparison = result.comparison.unwrap();
    let tax = comparison.tax(
        crate::income_tax::RegularTaxWorksheet {
            status: FederalStatus::Single,
            taxable: 0,
            preference: crate::income_tax::Preference {
                dividends: 0,
                gain: 0,
                line18: 0,
                line19: 0,
            },
            excluded: 0,
            foreign_election: false,
        },
        &mut r,
    );
    assert_eq!(tax, 0);
    assert_eq!(value(&r, "farm_installment_averaging_elected_income"), 0);
    assert_eq!(&r.lines[..original_lines.len()], original_lines.as_slice());
    assert_eq!(facts.unwrap().elected, 6000000);
}

#[test]
fn farmland_capped_averaging_reuses_foreign_preference_sources_with_separate_output() {
    let mut d = farmland_installment_return();
    d.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    d.deductions.as_mut().unwrap().itemize_override = None;
    d.screening.as_mut().unwrap().farm_averaging =
        averaging_return(62000).screening.unwrap().farm_averaging;
    let facts = d.screening.as_mut().unwrap().farm_averaging.as_mut().unwrap();
    for year in &mut facts.years {
        year.dividends = 500000;
        year.foreign = Some(AveragingForeign {
            forms: vec![AveragingForeignForm {
                owner: Owner::Taxpayer,
                exclusion: 10000000,
                housing: 0,
            }],
            disallowed: 0,
        });
        let AveragingSource::Return { tax, .. } = &mut year.source else { unreachable!() };
        *tax = match year.year {
            2023 => 195000,
            2024 => 194000,
            _ => 188300,
        };
    }
    let r = supported(&d);
    // $30,489 allocated taxable income includes $5,000 qualified dividends.
    // Stack $100,000 foreign income, tax $125,489 ordinary plus $750 capital,
    // subtract tax on $100,000: $6,867 / $6,857 / $6,800. Subtract retained
    // base taxes $1,950 / $1,940 / $1,883 => $14,751 comparison tax.
    assert_eq!(value(&r, "farm_installment_reduced_regular_tax"), 14751);
    for year in 2023..=2025 {
        let id = alloc::format!("farm_installment_averaging_{year}_foreign_stacking");
        assert_eq!(value(&r, &id), 100000);
        assert!(breakdown(&r).iter().any(|line| line.label == crate::line_label(&id).english()));
    }
    let mut ordinary = d.clone();
    farmland_sales_mut(&mut ordinary)[0].deferral = false;
    let plain = supported(&ordinary);
    assert_eq!(plain.tax, r.tax);
    for line in plain.lines.iter().filter(|line| line.id.starts_with("farm_averaging_")) {
        assert_eq!(r.lines.iter().find(|v| v.id == line.id), Some(line));
    }
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn farm_land_final_return_rejects_post_death_sale_and_preserves_prior_sale() {
    let mut d = farmland_installment_return();
    farmland_sales_mut(&mut d)[0].deferral = false;
    let sold = farmland_sales_mut(&mut d)[0].sold;
    let before = supported(&d);
    d.people.as_mut().unwrap().taxpayer.died = death(sold);
    let final_return = d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap();
    final_return.ordinary = true;
    let r = supported(&d);
    assert_eq!(r.tax, before.tax);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    farmland_sales_mut(&mut d)[0].sold = 20261231;
    d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().date = 20261230;
    unsupported(&d, "Farm land final-return allocation");
}

#[test]
fn farm_land_final_return_uses_linked_owner_on_joint_return() {
    let mut d = farmland_installment_return();
    farmland_sales_mut(&mut d)[0].deferral = false;
    farmland_sales_mut(&mut d)[0].sold = 20261231;
    let joint = complete(FederalStatus::Joint);
    d.setup = joint.setup;
    d.people = joint.people;
    let spouse = d.people.as_mut().unwrap().spouse.as_mut().unwrap();
    spouse.died = death(20260615);
    spouse.died.as_mut().unwrap().ordinary = true;
    let r = supported(&d); // Taxpayer owns this farm and is alive.
    assert!(r.tax.is_some());
    d.income[1].owner = crate::DocumentOwner::Spouses;
    unsupported(&d, "Farm land final-return allocation");
    farmland_sales_mut(&mut d)[0].sold = 20260615;
    let r = supported(&d);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn partner_nonfarm_optional_method_uses_shared_source_totals_and_weights() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(1000)));
    let business_id = d.income.last().unwrap().id;
    let mut partner = ordinary_partner_source(1000, true);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(100000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 900000 });
    let source = partner
        .se_after_passive(
            crate::tests::support::builders::k1_amount(0),
            crate::tests::support::builders::k1_amount(0),
        )
        .unwrap();
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let id = d.income.last().unwrap().id;
    elect_se(&mut d, SeMethod::Nonfarm { history: se_history() });
    let mut report = supported(&complete(FederalStatus::Single));
    let optional = crate::optional_se::calculate_with_partners(
        &d,
        Owner::Taxpayer,
        Some(&[(id, source)]),
        &mut report,
    )
    .unwrap();
    assert!(report.issues.is_empty());
    assert_eq!((optional.net, optional.eic_adjustment, optional.child_adjustment), (6667, 4667, 0));
    let weights: alloc::collections::BTreeMap<_, _> = optional.weights.into_iter().collect();
    assert_eq!(weights.get(&business_id), Some(&667));
    assert_eq!(weights.get(&id), Some(&6000));
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    let live = estimate(&restored);
    assert!(live.review_notes.iter().any(|note| {
        note.contains("elected optional-method SE projection for taxpayer")
            && note.contains("net earnings $6667")
            && note.contains("SE tax $1020")
    }));
    // The K-1 has no box 20 code Z statement: it adds no QBI.
    assert!(noted(&live, "WARN_K1_QBI_NO_STATEMENT"), "{live:?}");
    assert!(!live.issues.iter().any(|i| i.section == "Partnership optional SE"));
    assert!(live.review_notes.iter().any(|note| note.contains("coordination stabilized")
        && note.contains("SE tax $1020")
        && note.contains("half-SE deduction $510")));
    if let FederalIncome::Partnership { data } = &mut d.income[2].income {
        data.se_gross = None;
    }
    let mut report = supported(&complete(FederalStatus::Single));
    assert!(
        crate::optional_se::calculate_with_partners(&d, Owner::Taxpayer, Some(&[]), &mut report)
            .is_none()
    );
    assert!(report.issues.iter().any(|i| i.section == "Partnership optional SE"));
}

#[test]
fn partner_farm_and_nonfarm_optional_methods_share_one_cap() {
    let mut d = complete(FederalStatus::Single);
    let mut sources = vec![];
    for gross in [PartnerSeGross::Farm { gross: 900000 }, PartnerSeGross::Nonfarm { gross: 600000 }]
    {
        let mut partner = ordinary_partner_source(1000, true);
        partner.limited_for_se = Some(false);
        partner.reported_se = Some(100000);
        partner.se_gross = Some(gross);
        let source = partner
            .se_after_passive(
                crate::tests::support::builders::k1_amount(0),
                crate::tests::support::builders::k1_amount(0),
            )
            .unwrap();
        add(&mut d, income(FederalIncome::Partnership { data: partner }));
        sources.push((d.income.last().unwrap().id, source));
    }
    elect_se(&mut d, SeMethod::Both { history: se_history() });
    let mut report = supported(&complete(FederalStatus::Single));
    let result = crate::optional_se::calculate_with_partners(
        &d,
        Owner::Taxpayer,
        Some(&sources),
        &mut report,
    )
    .unwrap();
    assert!(report.issues.is_empty());
    assert_eq!(result.net, 7560);
    assert_eq!(result.weights, vec![(sources[0].0, 6000), (sources[1].0, 1560)]);
    assert_eq!(result.eic_adjustment, 5560);
    assert_eq!(result.child_adjustment, 0);
    elect_se(&mut d, SeMethod::Farm);
    let mut report = supported(&complete(FederalStatus::Single));
    let result = crate::optional_se::calculate_with_partners(
        &d,
        Owner::Taxpayer,
        Some(&sources),
        &mut report,
    )
    .unwrap();
    assert_eq!(result.net, 6924); // Nonfarm remains actual, farm optional gets no 92.35% reduction.
    if let FederalIncome::Partnership { data } = &mut d.income[1].income {
        data.limited_for_se = Some(true);
    }
    assert!(
        crate::optional_se::calculate_with_partners(
            &d,
            Owner::Taxpayer,
            Some(&sources),
            &mut report
        )
        .is_none()
    );
}

#[test]
fn partner_service_payments_enter_limited_partner_optional_farm_gross() {
    let mut partner = ordinary_partner_source(20000, false);
    partner.guaranteed_services = 600000;
    partner.reported_se = Some(600000);
    partner.se_gross = Some(PartnerSeGross::Farm { gross: 600000 });
    let source = partner
        .se_after_passive(
            crate::tests::support::builders::k1_amount(0),
            crate::tests::support::builders::k1_amount(0),
        )
        .unwrap();
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let id = d.income.last().unwrap().id;
    elect_se(&mut d, SeMethod::Farm);
    let mut r = supported(&complete(FederalStatus::Single));
    let optional = crate::optional_se::calculate_with_partners(
        &d,
        Owner::Taxpayer,
        Some(&[(id, source)]),
        &mut r,
    )
    .unwrap();
    assert!(r.issues.is_empty(), "{:?}", r.issues);
    assert_eq!(optional.net, 4000); // Two thirds of service gross, excluding box 1.
}

// T2-45: Form 1099-PATR cooperative distributions (Schedule F line 3a/3b),
// program payments 4a/4b and the section 126 refusal.
pub(in crate::tests) fn cooperative(
    patronage: i64,
    nonpatronage: i64,
    retains: i64,
    redeemed: i64,
) -> FederalCooperative {
    let mut data = FederalCooperative {
        patronage,
        nonpatronage,
        per_unit_retains: retains,
        withholding: 0,
        redeemed,
        section_199a_g: 0,
        qualified_payments: 0,
        qualified_items: 0,
        sstb_items: 0,

        ..Default::default()
    };
    // These synthetic source cases use original-patron notices redeemed at
    // stated value. The printed box 5 amount does not establish that history.
    if redeemed > 0 {
        let mut allocation = crate::cooperative::automatic(&data);
        allocation.redemptions.push(CoopRedemption {
            kind: CoopNoticeKind::Patronage,
            distribution: CoopDistributionBox::Redeemed,
            amount: redeemed,
            stated_value: redeemed,
            holding: CoopNoticeHolding::OriginalOrInherited,
            source: "Synthetic original-patron notice redeemed at stated value".into(),
        });
        data.allocations.push(allocation);
    }
    data
}
pub(in crate::tests) fn schedule_f_lines(r: &FederalResult) -> Vec<(alloc::string::String, i64)> {
    crate::draft_return(r)
        .iter()
        .filter(|row| row.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleF))
        .map(|row| (row.form_line.as_ref().unwrap().line.clone(), row.amount))
        .collect()
}

#[test]
fn farm_cooperative_distributions_reach_schedule_f_line_3_se_qbi_and_the_return() {
    // Hand-worked from the 2025 Schedule F instructions (lines 3a/3b; the
    // 2026 draft form keeps the same lines), Schedule SE, Form 8995 and the
    // 2026 single filing parameters.
    // Form 1099-PATR: box 1 $16,000, box 2 $1,000, box 3 $3,000, box 5
    // $2,000, box 4 withholding $300, box 7 $8,000 qualified payments; of
    // boxes 1-3, $2,000 came from buying personal items.
    //   3a = 16,000 + 1,000 + 3,000 + 2,000 = 22,000
    //   3b = 22,000 - 2,000 personal = 20,000 (personal-item dividends are
    //        on 3a only)
    // Farm: line 2 raised sales 40,000; line 9 = 40,000 + 20,000 = 60,000;
    // line 33 expenses 10,000; line 34 = 50,000.
    // Schedule SE: 50,000 x 92.35% = 46,175; x 15.3% = 7,064.775 -> 7,065;
    // half = 3,532.50 -> 3,533. AGI = 50,000 - 3,533 = 46,467.
    // Box 4 withholding $300 is Form 1040 line 25b.
    // Form 8995-A Schedule D patron reduction: lesser of 9% of the QBI
    // allocable to the $8,000 and 50% of the allocable W-2 wages; the farm
    // pays no W-2 wages, so 50% x 0 = 0.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(40000, 10000)));
    let mut patr = cooperative(1_600_000, 100_000, 300_000, 200_000);
    cooperative_personal(&mut patr, 200_000);
    patr.withholding = 30_000;
    patr.qualified_payments = 800_000;
    patr.allocations[0].qualified_payments = 800_000;
    add(&mut d, income(FederalIncome::Cooperative { data: cooperative_source(patr) }));
    let r = supported(&d);
    let farm = d.income[0].id;
    let coop = d.income[1].id;
    let prefix = alloc::format!("farm_{farm}");
    assert_eq!(value(&r, &alloc::format!("{prefix}_cooperative_distributions")), 22000);
    assert_eq!(value(&r, &alloc::format!("{prefix}_cooperative_taxable")), 20000);
    assert_eq!(value(&r, &alloc::format!("{prefix}_gross_income")), 60000);
    assert_eq!(value(&r, &alloc::format!("{prefix}_net_profit")), 50000);
    assert_eq!(value(&r, &alloc::format!("cooperative_{coop}_distributions")), 22000);
    assert_eq!(value(&r, &alloc::format!("cooperative_{coop}_taxable")), 20000);
    assert_eq!(value(&r, &alloc::format!("cooperative_{coop}_patron_reduction")), 0);
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(value(&r, "agi"), 46467);
    // Box 8 is empty, so none of the 20,000 is QBI (2026 i8995, Form 1099-PATR
    // items) and box 7 files Form 8995-A: QBI 26,467, 20% = 5,293.40 -> 5,293.
    // Taxable income 46,467 - 16,100 - 5,293 = 25,074; tax table
    // (25,050-25,100) 1,240 + 12% x (25,075 - 12,400) = 2,761; total tax
    // 2,761 + 7,065 = 9,826.
    assert_eq!(value(&r, "qbi_deduction"), 5293);
    assert_eq!(return_line(&r, "25b"), 30_000);
    assert_eq!(r.tax, Some(982600));
    let lines = schedule_f_lines(&r);
    assert!(lines.contains(&("3a".into(), 2_200_000)), "{lines:?}");
    assert!(lines.contains(&("3b".into(), 2_000_000)), "{lines:?}");
    assert!(lines.contains(&("9".into(), 6_000_000)), "{lines:?}");
    assert!(lines.contains(&("34".into(), 5_000_000)), "{lines:?}");
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn farm_cooperative_box_8_below_the_taxable_distributions_leaves_the_farm_qbi() {
    // Boundary: box 8 qualified items reported below the taxable
    // distributions. Same farm and 1099-PATR as above without box 7 or
    // withholding, plus a $40,000 W-2 so the taxable-income cap does not
    // bind: taxable income before QBI = 40,000 + 50,000 - 3,533 - 16,100 =
    // 70,367 (20% = 14,073). Box 8 unreported: no distribution is QBI, QBI
    // 26,467, deduction 5,293.40 -> 5,293. Box 8 = 19,000 against taxable
    // distributions 20,000: 1,000 is not a qualified item, QBI = 45,467,
    // deduction 9,093.40 -> 9,093. Box 8 = 20,000 (equal): QBI 46,467,
    // deduction 9,293.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 4_000_000, 0));
    add(&mut d, income(raised_farm(40000, 10000)));
    let mut patr = cooperative(1_600_000, 100_000, 300_000, 200_000);
    cooperative_personal(&mut patr, 200_000);
    add(&mut d, income(FederalIncome::Cooperative { data: cooperative_source(patr) }));
    assert_eq!(value(&supported(&d), "qbi_deduction"), 5293);
    let coop = d.income[2].id;
    let FederalIncome::Cooperative { data } = &mut d.income[2].income else { unreachable!() };
    data.form.qualified_items = 2_000_000;
    data.more.allocations[0].qualified_items = 2_000_000;
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 9293);
    assert_eq!(value(&r, &alloc::format!("cooperative_{coop}_nonqualified")), 0);
    let FederalIncome::Cooperative { data } = &mut d.income[2].income else { unreachable!() };
    data.form.qualified_items = 1_900_000;
    data.more.allocations[0].qualified_items = 1_900_000;
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 9093);
    assert_eq!(value(&r, &alloc::format!("cooperative_{coop}_nonqualified")), 1000);
    // The farm's Schedule F profit and SE tax are unchanged by box 8.
    assert_eq!(value(&r, "self_employment_tax"), 7065);
}

#[test]
fn farm_cooperative_source_limits_preserve_known_income_and_benefit_only_omissions() {
    let base = {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(raised_farm(40000, 10000)));
        d
    };
    let with = |edit: fn(&mut FederalCooperative)| {
        let mut d = base.clone();
        let mut patr = cooperative(1_000_000, 0, 0, 0);
        edit(&mut patr);
        add(&mut d, income(FederalIncome::Cooperative { data: cooperative_source(patr) }));
        d
    };
    let plain = supported(&with(|_| {}));
    let r = supported(&with(|p| p.section_199a_g = 1));
    assert!(noted(&r, "WARN_COOP_DPAD_NOTICE_NEEDED"));
    assert_eq!(r.tax, plain.tax);
    let r = supported(&with(|p| p.credit_facts.investment = 100));
    assert!(noted(&r, "WARN_COOP_CREDIT_SOURCE_NEEDED"));
    assert_eq!(r.tax, plain.tax);
    let mut wages = with(|p| p.qualified_payments = 100_000);
    let FederalIncome::Business { data } = &mut wages.income[0].income else { unreachable!() };
    data.payroll = BusinessPayroll::Yes {
        statements: vec![PayrollW2 { label: "Hand".into(), wages: 1_000_000, medicare: 1_000_000 }],
        attribution: crate::PayrollQbiAttribution::EntireBusiness,
    };
    let r = supported(&wages);
    assert!(noted(&r, "WARN_COOP_PATRON_ALLOCATION_NEEDED"));
    assert!(value(&r, "agi") > 0);
    // Section 175 with cooperative distributions on the same farm.
    let mut conservation = with(|_| {});
    let FederalIncome::Business { data } = &mut conservation.income[0].income else {
        unreachable!()
    };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    farm.conservation = Some(FarmConservation::Deduct {
        assessments: vec![],
        costs: vec![ConservationCost {
            property: 3,
            paid_or_incurred: None,
            facts: conservation_facts(),
            label: "Terracing".into(),
            amount: 100_000,
        }],
        carry: vec![],
        method: conservation_method(),
    });
    // This fixture adds a real retained parcel before linking its new cost.
    // The cooperative entry already owns ID3, so the shared allocator must
    // supply the parcel identity rather than assuming that ID is available.
    let mut next_id = conservation.next_id;
    let FederalIncome::Business { data } = &mut conservation.income[0].income else {
        unreachable!()
    };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    if let Some(FarmConservation::Deduct { costs, .. }) = &mut farm.conservation {
        costs[0].property = 0;
    }
    fixture_conservation_parcel(farm, &mut next_id);
    conservation.next_id = next_id;
    let result = supported(&conservation);
    assert!(noted(&result, "WARN_FARM_CONSERVATION_GROSS_SOURCE"));
    assert_eq!(value(&result, "farm_2_conservation_deduction"), 0);
    // No farm for the owner: the taxable distributions have no Schedule F.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 4_000_000, 0));
    add(
        &mut d,
        income(FederalIncome::Cooperative {
            data: cooperative_source(cooperative(1_000_000, 0, 0, 0)),
        }),
    );
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    missing_related_activity(
        &d,
        &r,
        crate::related_forms::DocumentKind::Patr1099,
        &[TaxSection::SelfEmployment, TaxSection::Farming],
    );
    // A statement that is entirely personal purchases needs no farm and
    // adds nothing but its withholding.
    let mut personal = cooperative(1_000_000, 0, 0, 0);
    cooperative_personal(&mut personal, 1_000_000);
    personal.withholding = 5_000;
    personal.allocations[0].id = d.next_id;
    d.next_id += 1;
    d.income[1].income = FederalIncome::Cooperative { data: cooperative_source(personal) };
    let r = supported(&d);
    assert_eq!(return_line(&r, "25b"), 5_000);
    assert_eq!(value(&r, "agi"), 40000);
}

#[test]
fn farm_program_payments_are_tagged_4a_4b_and_cost_sharing_is_refused_by_name() {
    // Schedule F line 4a = 4b = the fully taxable $3,000 program payment
    // (Pub 225: price loss coverage and similar payments are fully
    // taxable). A section 126 cost-sharing payment sits on line 4a; its
    // excludable portion needs the present-value computation, refused by
    // name.
    let mut d = empty_return(FederalStatus::Single);
    add(
        &mut d,
        income(farm_business(
            vec![
                FarmReceipt::Raised { amount: 4_000_000 },
                FarmReceipt::Program { amount: 300_000 },
            ],
            10000,
        )),
    );
    let lines = schedule_f_lines(&supported(&d));
    assert!(lines.contains(&("4a".into(), 300_000)), "{lines:?}");
    assert!(lines.contains(&("4b".into(), 300_000)), "{lines:?}");
    assert!(lines.contains(&("9".into(), 4_300_000)), "{lines:?}");
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    let FarmAccounting::Cash { receipts } = &mut farm.accounting else { unreachable!() };
    receipts.push(FarmReceipt::CostSharing { amount: 50_000 });
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(
        r.issues.iter().any(|i| i.section == "Farm program payments"
            && i.message.contains("section 126")
            && i.message.contains("line 4b")),
        "{r:?}"
    );
    assert_eq!(restart(d.clone()).draft, d);
}

// Schedule F Part II rows of a farm, tagged and untagged (T2-45x).
fn farm_expense(category: BusinessExpenseCategory, dollars: i64) -> FederalBusinessExpense {
    FederalBusinessExpense {
        treatment: if category == BusinessExpenseCategory::Meals {
            BusinessExpenseKind::Meals
        } else {
            BusinessExpenseKind::Ordinary
        },
        category,
        amount: dollars * 100,
    }
}

#[test]
fn farm_expenses_print_on_schedule_f_part_ii_lines_not_schedule_c() {
    // 2026 draft i1040sf, Part II (pp. 7–10): line 15 employee benefit
    // programs, 20 insurance, 21a mortgage interest, 21c other interest, 22
    // labor hired ("the amounts you paid for farm labor"), 24a rent of
    // vehicles, machinery and equipment, 24b other rent ("such as pasture or
    // farmland"), 25 repairs, 28 supplies, 29 taxes, 30 utilities, 32a–32e
    // other expenses (travel and 50% of meals, p. 10). The farm-only lines
    // (T2-45y): 11 chemicals, 13 custom hire ("the machine operator furnished
    // the equipment"), 16 feed, 17 fertilizers and lime, 18 freight and
    // trucking, 19 gasoline, fuel and oil, 23 employees' pension and
    // profit-sharing plans (the owner's own plan is Schedule 1 line 16, not
    // Schedule F), 26 seeds and plants, 27 storage and warehousing, 31
    // veterinary, breeding and medicine: each its own tagged line, all
    // ordinary deductible expenses.
    // Hand totals: labor 22 = $20,000 wages + $5,000 contract labor = $25,000;
    // meals $1,000 × 50% = $500; other "Soil testing" $10,000 (line 32).
    // Schedule C categories: 700 + 2,500 + 3,000 + 800 + 25,000 + 4,000 +
    // 6,000 + 2,000 + 3,000 + 1,500 + 1,200 + 500 + 10,000 = $60,200.
    // Farm-only lines: 1,100 + 1,300 + 1,600 + 1,700 + 1,800 + 1,900 + 2,300 +
    // 2,600 + 2,700 + 3,100 = $20,100. Line 33 = 60,200 + 20,100 = $80,300;
    // line 9 = $100,000 raised products (line 2); line 34 = 100,000 − 80,300 =
    // $19,700.
    use BusinessExpenseCategory as C;
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = raised_farm(100_000, 0);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    data.expenses = vec![
        farm_expense(C::Wages, 20_000),
        farm_expense(C::ContractLabor, 5_000),
        farm_expense(C::EmployeeBenefits, 700),
        farm_expense(C::Insurance, 2_500),
        farm_expense(C::MortgageInterest, 3_000),
        farm_expense(C::OtherInterest, 800),
        farm_expense(C::VehicleAndEquipmentRent, 4_000),
        farm_expense(C::OtherRent, 6_000),
        farm_expense(C::Repairs, 2_000),
        farm_expense(C::Supplies, 3_000),
        farm_expense(C::TaxesAndLicenses, 1_500),
        farm_expense(C::Utilities, 1_200),
        farm_expense(C::Meals, 1_000),
        farm_expense(C::Other { label: "Soil testing".into() }, 10_000),
        farm_expense(C::Chemicals, 1_100),
        farm_expense(C::CustomHire, 1_300),
        farm_expense(C::Feed, 1_600),
        farm_expense(C::Fertilizers, 1_700),
        farm_expense(C::Freight, 1_800),
        farm_expense(C::Fuel, 1_900),
        farm_expense(C::EmployeePension, 2_300),
        farm_expense(C::Seeds, 2_600),
        farm_expense(C::Storage, 2_700),
        farm_expense(C::Veterinary, 3_100),
    ];
    add(&mut d, income(entry));
    let id = d.income[0].id;
    let r = supported(&d);
    let lines = schedule_f_lines(&r);
    for (line, dollars) in [
        ("2", 100_000),
        ("9", 100_000),
        ("15", 700),
        ("20", 2_500),
        ("21a", 3_000),
        ("21c", 800),
        ("22", 25_000),
        ("24a", 4_000),
        ("24b", 6_000),
        ("25", 2_000),
        ("28", 3_000),
        ("29", 1_500),
        ("30", 1_200),
        ("11", 1_100),
        ("13", 1_300),
        ("16", 1_600),
        ("17", 1_700),
        ("18", 1_800),
        ("19", 1_900),
        ("23", 2_300),
        ("26", 2_600),
        ("27", 2_700),
        ("31", 3_100),
        ("33", 80_300),
        ("34", 19_700),
    ] {
        assert!(lines.contains(&(line.into(), dollars * 100)), "line {line}: {lines:?}");
    }
    // One line 22 row for both labor categories.
    assert_eq!(lines.iter().filter(|(l, _)| l == "22").count(), 1);
    // Other expenses stay untagged rows with Schedule F explanations.
    for (suffix, dollars) in [("line32_meals", 500), ("line32_other", 10_000)] {
        let row = r
            .lines
            .iter()
            .find(|l| l.id == alloc::format!("farm_{id}_expense_{suffix}"))
            .unwrap_or_else(|| panic!("missing {suffix}: {r:?}"));
        assert_eq!(row.amount, dollars * 100);
        assert!(row.form_line.is_none());
        assert!(row.explanation.english().starts_with("Schedule F lines 32a–32e"));
    }
    // Form 3903 moving_expense_deduction is not a farm expense row.
    for row in r.lines.iter().filter(|l| l.id.contains("_expense_") && !l.id.starts_with("moving_"))
    {
        assert!(row.id.starts_with("farm_"), "{}", row.id);
        assert!(row.explanation.english().starts_with("Schedule F"), "{}", row.id);
        assert!(row.source.ends_with("i1040sf"), "{}", row.id);
    }
    assert!(
        !r.lines.iter().any(|l| l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleC)),
        "{r:?}"
    );
    // The owner's own plan is not line 23.
    let pension = r
        .lines
        .iter()
        .find(|l| l.id == alloc::format!("farm_{id}_expense_line23_pension_and_profit_sharing"))
        .unwrap();
    assert!(pension.explanation.english().contains("Schedule 1 line 16"));
    assert_eq!(value(&r, "agi"), 19_700 - value(&r, "adjustments"));
}

#[test]
fn schedule_c_business_with_a_farm_only_expense_line_is_refused_by_name() {
    // Schedule C Part II has no line for the Schedule F farm-only expenses
    // (i1040sc lines 8–27b; i1040sf lines 11, 13, 16–19, 23, 26, 27, 31): a
    // non-farm business storing one is refused, naming the line.
    use BusinessExpenseCategory as C;
    for (category, needle) in
        [(C::Feed, "Schedule F line 16"), (C::Veterinary, "Schedule F line 31")]
    {
        let mut d = complete(FederalStatus::Single);
        let mut entry = cash_business(20_000);
        let FederalIncome::Business { data } = &mut entry else { unreachable!() };
        data.expenses = vec![farm_expense(category, 1_000)];
        add(&mut d, income(entry));
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
        assert!(
            r.issues.iter().any(|i| i.message == "FARM_ONLY_EXPENSE_IN_BUSINESS" && {
                let message = &i.display;
                message.english().contains(needle)
            }),
            "{:?}",
            r.issues
        );
    }
    // The same record on a farm is accepted.
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = raised_farm(20_000, 0);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    data.expenses = vec![farm_expense(C::Feed, 1_000)];
    add(&mut d, income(entry));
    supported(&d);
}

#[test]
fn farm_standard_mileage_is_schedule_f_line_10_not_schedule_c_line_9() {
    // 2026 draft i1040sf "What's New": 72.5 cents a mile January–June and
    // 76 cents July–December. 1,000 + 2,000 miles: $725 + $1,520 = $2,245 on
    // Schedule F line 10 (car and truck expenses), in line 33 with the $10,000
    // other expenses: $12,245; line 34 = 40,000 − 12,245 = $27,755.
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = raised_farm(40_000, 10_000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    data.vehicles.push(native_business_mileage(1_000, 2_000, 4_000));
    add(&mut d, income(entry));
    let r = supported(&d);
    let lines = schedule_f_lines(&r);
    assert!(lines.contains(&("10".into(), 224_500)), "{lines:?}");
    assert!(lines.contains(&("33".into(), 1_224_500)), "{lines:?}");
    assert!(lines.contains(&("34".into(), 2_775_500)), "{lines:?}");
    assert!(
        !r.lines.iter().any(|l| l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleC)),
        "{r:?}"
    );
}

#[test]
fn farm_vehicle_loan_interest_is_schedule_f_line_21b_with_totals_unchanged() {
    // 2026 draft f1040sf (created 4/24/26) adds line 21b "Vehicle loan"; other
    // interest moves to 21c. The 2026 draft i1040sf (Sep 17, 2026), "New line
    // 21b" and "Line 21b": report the business-use part of vehicle-loan
    // interest on 21b whether the standard mileage rate or actual costs are
    // used. The standard mileage rate does not include it (line 10 holds the
    // mileage, parking and tolls; the personal property tax is line 29).
    // Vehicle: 1,000 + 2,000 business miles of 4,000. Mileage 1,000 x 72.5c +
    // 2,000 x 76c = $725 + $1,520 = $2,245. Loan interest $2,000 x 3,000 /
    // 4,000 = $1,500 (21b). Value-based tax $400 x 3,000 / 4,000 = $300 is
    // line 29 (2026 draft i1040sf line 10). Line 10 + line 21b + line 29 =
    // $4,045, what the car-and-truck line held before the split. Mortgage interest
    // $300 is 21a; other interest $500 is 21c. Line 33 = $10,000 other
    // expenses + 300 + 500 + 4,045 = $14,845; line 34 = 40,000 - 14,845 =
    // $25,155.
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = raised_farm(40_000, 10_000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    data.vehicles.push(BusinessMileage {
        id: 0,
        interest: 200_000,
        property_tax: 40_000,
        ..native_business_mileage(1_000, 2_000, 4_000)
    });
    for (category, amount) in [
        (BusinessExpenseCategory::MortgageInterest, 30_000),
        (BusinessExpenseCategory::OtherInterest, 50_000),
    ] {
        data.expenses.push(ordinary_business_expense(category, amount));
    }
    add(&mut d, income(entry));
    let r = supported(&d);
    // Unit17 SF-G4/R8 (approved 2026-10-10): a farm's standard-mileage
    // vehicle requires Form4562 even without a depreciable asset.
    assert!(r.lines.iter().any(|row| {
        row.form_line
            .as_ref()
            .is_some_and(|form| form.form == TaxForm::Form4562 && form.line == "22")
    }));
    let lines = schedule_f_lines(&r);
    // The 300 of property tax on 3,000 of 4,000 miles is line 29.
    assert!(lines.contains(&("10".into(), 224_500)), "{lines:?}");
    assert!(lines.contains(&("29".into(), 30_000)), "{lines:?}");
    assert!(lines.contains(&("21a".into(), 30_000)), "{lines:?}");
    assert!(lines.contains(&("21b".into(), 150_000)), "{lines:?}");
    assert!(lines.contains(&("21c".into(), 50_000)), "{lines:?}");
    assert!(lines.contains(&("33".into(), 1_484_500)), "{lines:?}");
    assert!(lines.contains(&("34".into(), 2_515_500)), "{lines:?}");
    // The split moves no dollars: lines 10, 21b and 29 add to the $4,045 the one
    // car-and-truck line held, so line 33 is the hand sum above.
    let line = |name: &str| lines.iter().filter(|l| l.0 == name).map(|l| l.1).sum::<i64>();
    assert_eq!(line("10") + line("21b") + line("29"), 404_500);
    // No old label survives: other interest is not 21b.
    assert!(!r.lines.iter().any(|l| l.id.ends_with("line21b_other_interest")));
}

#[test]
fn farm_crp_and_ccc_market_gain_are_schedule_f_line_4_and_ccc_sales_lines_1_and_2() {
    // Where the amounts print, from the sources:
    // - CRP annual payments go "on Schedule F, lines 4a and 4b" (Pub 225
    //   (2025) ch. 12, p. 80; Schedule SE line 1b: "Conservation Reserve
    //   Program payments included on Schedule F, line 4b").
    // - CCC market gain is a line 4a agricultural program payment (2026
    //   draft i1040sf p. 5, lines 4a and 4b), taxable on 4b only when the
    //   loan was not included in income (Pub 225 (2025) ch. 3, p. 13,
    //   Example 1: principal $5,200, redeemed for $4,000, sold for $6,500).
    // - A redeemed commodity sold: with the loan included it was "repurchased"
    //   for $4,000 (line 1a $6,500, line 1b $4,000); without it, a raised
    //   product with no basis (line 2).
    // Farm: $52,300 raised, $3,000 program payment, $5,000 CRP, $10,000
    // other expenses.
    // Loan method: 2 = 52,300 + 6,500 = $58,800; 4a = 4b = 3,000 + 5,000 +
    // 1,200 = $9,200; 9 = 58,800 + 9,200 = $68,000.
    // Income method: 1a $6,500, 1b $4,000 (1c $2,500); 2 = $52,300; 4a =
    // $9,200; 4b = 3,000 + 5,000 = $8,000; 5a $5,200; 9 = 2,500 + 52,300 +
    // 8,000 + 5,200 = $68,000. Both: 33 $10,000, 34 $58,000.
    for (method, expected) in [
        (CccMethod::Loan, [("2", 58_800), ("4a", 9_200), ("4b", 9_200), ("9", 68_000)].as_slice()),
        (
            CccMethod::Income,
            [
                ("1a", 6_500),
                ("1b", 4_000),
                ("2", 52_300),
                ("4a", 9_200),
                ("4b", 8_000),
                ("9", 68_000),
            ]
            .as_slice(),
        ),
    ] {
        let mut d = ccc_return(
            CccOrigin::Current,
            method.clone(),
            CccDisposition::Redeemed { year: 2026, paid: 400_000, sale: Some(650_000) },
            42_300,
        );
        let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
        let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
        farm.crp.push(CrpPayment { amount: 500_000, benefits: false, withholding: 0 });
        let FarmAccounting::Cash { receipts } = &mut farm.accounting else { unreachable!() };
        receipts.push(FarmReceipt::Program { amount: 300_000 });
        let r = supported(&d);
        let lines = schedule_f_lines(&r);
        for (line, dollars) in expected.iter().chain(&[("33", 10_000), ("34", 58_000)]) {
            assert!(
                lines.contains(&((*line).into(), dollars * 100)),
                "{method:?} line {line}: {lines:?}"
            );
        }
        assert_eq!(return_line(&r, "8"), 5_800_000);
    }
}

#[test]
fn farm_fuel_tax_recovery_is_in_schedule_f_lines_9_and_34() {
    // 2026 draft i1040sf line 8 (p. 6): "The amount of credit for federal
    // tax paid on fuels claimed on your 2025 Schedule 3". A $1,000 cash
    // recovery joins line 9: 40,000 + 1,000 = $41,000; line 34 = 41,000 −
    // 10,000 = $31,000, the same amount as Schedule 1 line 6 and Schedule SE
    // line 1a.
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = raised_farm(40_000, 10_000);
    let FederalIncome::Business { data } = &mut entry else { unreachable!() };
    data.cash_fuel_tax_recovery = Some(cash_fuel_recovery(100_000));
    add(&mut d, income(entry));
    let id = d.income[0].id;
    let r = supported(&d);
    let lines = schedule_f_lines(&r);
    assert!(lines.contains(&("9".into(), 4_100_000)), "{lines:?}");
    assert!(lines.contains(&("34".into(), 3_100_000)), "{lines:?}");
    let se = crate::draft_return(&r)
        .iter()
        .find(|row| {
            row.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleSe && f.line == "1a")
        })
        .map(|row| row.amount);
    assert_eq!(se, Some(3_100_000));
    let recovery = r
        .lines
        .iter()
        .find(|l| l.id == alloc::format!("business_{id}_cash_fuel_tax_recovery"))
        .unwrap();
    assert!(recovery.explanation.english().contains("Schedule F line 8"));
}

#[test]
fn form_6252_farm_land_installment_takes_section_1252_recapture_first() {
    // Land bought June 15, 2021, sold June 16, 2026 for $150,000 (a note),
    // $10,000 selling expenses, $100,000 basis, $20,000 prior section 175
    // deductions: gain $40,000, recapture 80% (sixth year) = $16,000.
    // Form 6252: line 13 $110,000 (line 12 zero: section 1252 is not
    // section 453(i) income); 16 $40,000; 18 $150,000.
    let installment = |payments| {
        let mut sale = farm_land_sale(20210615, 20260616);
        sale.payment = crate::SalePayment::Installment {
            assumed_debt: 0,
            payments,
            interest: crate::NoteInterest::Adequate,
            special: false,
        };
        sale
    };
    // $37,500 received: line 24 $37,500 x 40/150 = $10,000; line 25 the
    // smaller of $10,000 and $16,000; line 26 $0.
    let v = installment(3_750_000).calculate();
    assert_eq!((v.gain, v.ordinary, v.section1231), (10000, 10000, 0));
    // $75,000 received: line 24 $20,000; line 25 $16,000; line 26 $4,000.
    let sale = installment(7_500_000);
    let v = sale.calculate();
    assert_eq!((v.gain, v.ordinary, v.section1231), (20000, 16000, 4000));
    let mut r = estimate(&complete(FederalStatus::Single));
    sale.report(2, 0, crate::farm::FarmForm::ScheduleF, &mut r);
    assert_eq!(value(&r, "farm_2_land_0_form6252_total"), 110000);
    assert_eq!(value(&r, "farm_2_land_0_form6252_income"), 20000);
    assert_eq!(value(&r, "farm_2_land_0_form6252_ordinary"), 16000);
    assert_eq!(value(&r, "farm_2_land_0_form6252_gain"), 4000);
    assert!(r.issues.is_empty(), "{:?}", r.issues);
    // Held one year or less, or with the Form 1062 election: refused.
    let mut short = installment(7_500_000);
    short.acquired = 20250616;
    let mut r = estimate(&complete(FederalStatus::Single));
    short.report(2, 0, crate::farm::FarmForm::ScheduleF, &mut r);
    assert!(r.issues.iter().any(|i| {
        i.message.contains("one year or less") && {
            let m = &i.display;
            m.key == "ISSUE_FARM_LAND_INSTALLMENT_SHORT_HOLDING_PERIOD"
        }
    }));
    let mut elected = installment(7_500_000);
    elected.deferral = true;
    let mut r = estimate(&complete(FederalStatus::Single));
    elected.report(2, 0, crate::farm::FarmForm::ScheduleF, &mut r);
    assert!(r.issues.iter().any(|i| {
        i.message.contains("Form 1062") && {
            let m = &i.display;
            m.key == "ISSUE_FARM_LAND_INSTALLMENT_WITH_DEFERRAL"
        }
    }));
    let mut special = installment(7_500_000);
    let crate::SalePayment::Installment { special: flag, .. } = &mut special.payment else {
        unreachable!()
    };
    *flag = true;
    let mut r = estimate(&complete(FederalStatus::Single));
    special.report(2, 0, crate::farm::FarmForm::ScheduleF, &mut r);
    assert!(r.issues.iter().any(|i| {
        {
            let m = &i.display;
            m.key == "ISSUE_INSTALLMENT_SPECIAL_TRANSACTION"
        }
    }));
}

// A named unentered transition retains the actual farm and its routed
// cooperative source. Required unknown timing does not manufacture zero income.
#[test]
fn farm_transition_boundary_retains_its_routed_cooperative_statement() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 4_000_000, 0));
    let mut farm = raised_farm(40_000, 10_000);
    let FederalIncome::Business { data } = &mut farm else { panic!() };
    let BusinessActivity::Farming { data: records } = &mut data.activity else { panic!() };
    records.specials.limitations = vec![3];
    add(&mut d, income(farm));
    add(
        &mut d,
        income(FederalIncome::Cooperative {
            data: cooperative_source(cooperative(1_000_000, 0, 0, 0)),
        }),
    );
    let before = d.clone();
    let result = estimate(&d);
    assert_eq!(result.tax, None);
    assert!(
        result
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_FARM_METHOD_TRANSITION_SOURCE")
    );
    let mut notices = vec![];
    let working = crate::unknowns::supported(&d, &mut notices).unwrap_or_else(|| d.clone());
    assert_eq!(working.income.len(), 3);
    assert!(working.income.iter().any(|entry|matches!(&entry.income,FederalIncome::Cooperative{data} if data.form.patronage==1_000_000)));
    assert_eq!(d, before);
}

#[test]
fn farm_prepaid_deposit_does_not_require_an_unclaimed_section179_supply_ceiling() {
    let mut supplies = farm_supplies(10_000);
    supplies.advances[0].purchase.kind = FarmSupplyPaymentKind::Deposit;
    let mut d = farm_supply_return(supplies, 10_000, 80_000);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.assets = vec![elected_asset(10_000, 4_000, BonusElection::Apply)];
    let result = supported(&d);
    assert_eq!(farm_supply_value(&d, &result, "deduction"), 0);
    assert_eq!(farm_supply_value(&d, &result, "remaining"), 10_000);
}

#[test]
fn conservation_livestock_production_gross_matches_cfr_and_irs_examples() {
    // Reg1.175-5(a)(2): $10,000 crops + $500 breeding gain; $100 tractor,
    // $8,000 land gain and $100 interest are not agricultural production gross.
    // https://www.govinfo.gov/content/pkg/CFR-2025-title26-vol4/pdf/CFR-2025-title26-vol4-sec1-175-5.pdf
    let mut draft = conservation_livestock_return(10_000, 500, Some(LivestockPurpose::Breeding));
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.assets.push(farmer_gross_asset(100, Livestock::No));
    add(
        &mut draft,
        income(FederalIncome::Interest { data: interest_source(native_interest(10_000, 0)) }),
    );
    let mut land_source = raised_farm(0, 0);
    let FederalIncome::Business { data } = &mut land_source else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &mut data.activity else { unreachable!() };
    let mut sale = farm_land_sale(20000101, 20260601);
    sale.basis = 1_000_000;
    sale.proceeds = 1_800_000;
    sale.selling = 0;
    sale.conservation = 0;
    set_land_sales(farm, vec![sale], &mut draft.next_id);
    add(&mut draft, income(land_source));
    let result = supported(&draft);
    assert_eq!(value(&result, "farm_2_conservation_income_limit"), 2625);
    assert_eq!(value(&result, "farm_2_conservation_deduction"), 2625);
    // Pub225ch5: $85,000 production + $15,000 cull breeding gain = $100,000.
    let draft = conservation_livestock_return(85_000, 15_000, Some(LivestockPurpose::Breeding));
    assert_eq!(value(&supported(&draft), "farm_2_conservation_deduction"), 25_000);
}

#[test]
fn conservation_livestock_purpose_distinguishes_production_sport_and_unavailable() {
    for (purpose, expected) in [
        (Some(LivestockPurpose::Draft), 2625),
        (Some(LivestockPurpose::Dairy), 2625),
        (Some(LivestockPurpose::Sport), 2500),
        (None, 0),
    ] {
        let draft = conservation_livestock_return(10_000, 500, purpose);
        let result = supported(&draft);
        assert_eq!(value(&result, "farm_2_conservation_deduction"), expected);
        assert_eq!(noted(&result, "WARN_FARM_CONSERVATION_GROSS_SOURCE"), purpose.is_none());
        assert_eq!(value(&result, "business_2_asset_0_disposal_gain"), 500);
        if purpose.is_none() {
            let note = result
                .review_notes
                .iter()
                .find(|note| note.display.key == "WARN_FARM_CONSERVATION_GROSS_SOURCE")
                .unwrap();
            assert_eq!(
                crate::interview::issue_pages(&draft, note.context.as_ref().unwrap(), None),
                vec![TaxSection::Farming]
            );
        }
    }
}

#[test]
fn conservation_livestock_excluded_current_transactions_keep_gross_unavailable() {
    for case in 0..3 {
        let mut draft =
            conservation_livestock_return(10_000, 500, Some(LivestockPurpose::Breeding));
        let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
        let AssetDisposal::Disposed { how: DisposalKind::Sold { payment, buyer, .. }, .. } =
            &mut data.assets[0].disposal
        else {
            unreachable!()
        };
        match case {
            0 => {
                *payment = SalePayment::Installment {
                    assumed_debt: 0,
                    payments: 25_000,
                    interest: NoteInterest::Adequate,
                    special: false,
                }
            }
            1 => *buyer = Buyer::Relative,
            _ => {
                data.assets[0].disposal = AssetDisposal::Disposed {
                    date: 20260601,
                    how: DisposalKind::Condemned {
                        award: 50_000,
                        expenses: 0,
                        replacement_cost: 0,
                    },
                }
            }
        }
        let result = supported(&draft);
        assert_eq!(value(&result, "farm_2_conservation_deduction"), 0);
        assert!(noted(&result, "WARN_FARM_CONSERVATION_GROSS_SOURCE"));
    }
}

#[test]
fn conservation_livestock_regular_and_amt_gains_use_their_own_basis() {
    let mut draft = conservation_livestock_return(10_000, 500, Some(LivestockPurpose::Breeding));
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let id = data.assets[0].id;
    let disposal = data.assets[0].disposal.clone();
    data.assets[0] = continuing_asset(200_000, 20250110, EquipmentClass::Five, 40_000, disposal);
    data.assets[0].id = id;
    // Synthetic purchase and first-service dates are separate source facts.
    // Acquisition before September28,2017 requires the distinct150% AMT history.
    data.assets[0].acquired = 20170927;
    let EquipmentHistory::Continuing { amt, .. } = &mut data.assets[0].history else {
        unreachable!()
    };
    *amt = EquipmentAmtHistory::Separate {
        prior_depreciation: 30_000,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    let AssetDisposal::Disposed { how: DisposalKind::Sold { proceeds, .. }, .. } =
        &mut data.assets[0].disposal
    else {
        unreachable!()
    };
    *proceeds = 260_000;
    // Regular: $2,000 - $400 - $320 = $1,280 basis; $1,320 gain.
    // AMT150%: $2,000 - $300 - $255 = $1,445 basis; $1,155 gain.
    let claims = crate::related_forms::effective_claimants(&draft);
    let regular =
        crate::asset_sale::conservation_livestock_gross_with_claims(&draft, false, &claims)
            .unwrap();
    let alternative =
        crate::asset_sale::conservation_livestock_gross_with_claims(&draft, true, &claims).unwrap();
    assert_eq!(regular.get(&2), Some(&132_000));
    assert_eq!(alternative.get(&2), Some(&115_500));
    let result = supported(&draft);
    assert_eq!(line_amount(&result, "farm_2_conservation_income_limit"), 283_000);
    assert_eq!(line_amount(&result, "farm_2_conservation_deduction"), 283_000);
    assert_eq!(line_amount(&result, "farm_2_conservation_amt_deduction"), 278_875);
}

#[test]
fn conservation_livestock_farm_rental_uses_the_same_pool() {
    let mut draft = conservation_livestock_return(10_000, 500, Some(LivestockPurpose::Breeding));
    let FederalIncome::Business { data } = &draft.income[1].income else { unreachable!() };
    let BusinessActivity::Farming { data: farm } = &data.activity else { unreachable!() };
    let mut rental = farm_rental_data(0, 0);
    rental.farm = farm.clone();
    rental.assets = data.assets.clone();
    draft.income[1].income = FederalIncome::FarmRental { data: rental };
    let result = supported(&draft);
    assert_eq!(line_amount(&result, "farm_2_conservation_income_limit"), 262_500);
    assert_eq!(line_amount(&result, "farm_2_conservation_deduction"), 262_500);
}

#[test]
fn conservation_zero_livestock_gain_needs_no_production_purpose() {
    let draft = conservation_livestock_return(10_000, 0, None);
    let result = supported(&draft);
    assert_eq!(value(&result, "farm_2_conservation_deduction"), 2500);
    assert!(!noted(&result, "WARN_FARM_CONSERVATION_GROSS_SOURCE"));
}

#[test]
fn conservation_livestock_community_gain_is_attributed_once() {
    let mut draft = conservation_livestock_return(10_000, 500, Some(LivestockPurpose::Breeding));
    draft.setup = complete(FederalStatus::Separate).setup;
    answer_separate_pages(&mut draft);
    draft.setup.as_mut().unwrap().community_property = community_arizona();
    let FederalIncome::Wage { data } = &draft.income[0].income else { unreachable!() };
    let data = FederalWage::from(data);
    draft.income[0] = community_w2_draft(data.wages, 0, data.withholding, 0).income.remove(0);
    draft.income[1].community = CommunityShare::Community;
    let claims = crate::related_forms::effective_claimants(&draft);
    let gross = crate::asset_sale::conservation_livestock_gross_with_claims(&draft, false, &claims)
        .unwrap();
    assert_eq!(gross.get(&2), Some(&25_000));
    let result = supported(&draft);
    assert_eq!(line_amount(&result, "form8958_gain_2_total"), 50_000);
    assert_eq!(line_amount(&result, "form8958_gain_2_this_return"), 25_000);
    crate::tests::federal::schedules_1_3::assert_form_8958_divided_sources_conserve(
        &draft, &result,
    );
    assert_eq!(
        result
            .lines
            .iter()
            .find(|line| line.id == "farm_2_conservation_income_limit")
            .unwrap()
            .amount,
        131_250
    );
}

#[test]
fn farm_ccc_printed_market_gain_reconciles_by_owner_and_counts_once() {
    const ISSUE: &str =
        "ISSUE_FORM_1099_G_BOX_9_MARKET_GAIN_CANNOT_BE_RECONCILED_TO_THIS_RETURN_S_165FF0EF";
    for split in [false, true] {
        let mut draft = ccc_return(
            CccOrigin::Current,
            CccMethod::Loan,
            CccDisposition::Redeemed { year: 2026, paid: 285_000, sale: None },
            0,
        );
        ccc_mut(&mut draft).lots[0].principal = 300_000;
        ccc_mut(&mut draft).withholding = 0;
        for amount in if split { vec![10_000, 5_000] } else { vec![15_000] } {
            let mut paper = government_unemployment(0, &[], 0);
            let FederalIncome::GovernmentPayment { data } = &mut paper.income else {
                unreachable!()
            };
            data.form.market_gain = Some(amount);
            add(&mut draft, paper);
        }
        let report = supported(&draft);
        assert!(!report.issues.iter().any(|issue| issue.display.key == ISSUE));
        assert_eq!(return_line(&report, "8"), 15_000);
        assert_eq!(ccc_value(&draft, &report, "taxable_market_gain"), 150);
        assert!(schedule_f_lines(&report).contains(&("4b".into(), 15_000)));
        let FederalIncome::GovernmentPayment { data } = &mut draft.income[1].income else {
            unreachable!()
        };
        *data.form.market_gain.as_mut().expect("printed box 9") += 100;
        assert!(estimate(&draft).issues.iter().any(|issue| issue.display.key == ISSUE));
        let FederalIncome::GovernmentPayment { data } = &mut draft.income[1].income else {
            unreachable!()
        };
        data.form.market_gain = Some(if split { 10_000 } else { 15_000 });
        // Equal totals from another recipient never substantiate this owner's loan.
        let joint = empty_return(FederalStatus::Joint);
        draft.setup = joint.setup;
        draft.people = joint.people;
        draft.income[1].owner = crate::DocumentOwner::Spouses;
        assert!(estimate(&draft).issues.iter().any(|issue| issue.display.key == ISSUE));
    }
}

#[test]
fn farm_linked_taxable_grant_enters_other_income_once_and_retains_source_withholding() {
    for accrual in [false, true] {
        let mut draft = empty_return(FederalStatus::Single);
        add(
            &mut draft,
            income(if accrual { accrual_farm(10_000, 0, 0, 0, 0) } else { raised_farm(10_000, 0) }),
        );
        let farm = draft.income[0].id;
        let mut paper = government_unemployment(0, &[], 37);
        let FederalIncome::GovernmentPayment { data } = &mut paper.income else { unreachable!() };
        data.form.taxable_grants = Some(50_000);
        add(&mut draft, paper);
        let FederalIncome::GovernmentPayment { data } = &draft.income[1].income else {
            unreachable!()
        };
        let component = data.component_ids.grant;
        claim_related_component(&mut draft, farm, component);
        let report = supported(&draft);
        let lines = schedule_f_lines(&report);
        assert!(lines.contains(&((if accrual { "43" } else { "8" }).into(), 50_000)), "{lines:?}");
        let program = if accrual { "39a" } else { "4a" };
        assert_eq!(
            lines.iter().find(|(line, _)| line == program).map_or(0, |(_, amount)| *amount),
            0,
            "{lines:?}",
        );
        assert_eq!(return_line(&report, "8"), 1_050_000);
        assert_eq!(report.payment_withholding.as_ref().unwrap().forms_1099, 3_700);
        assert_eq!(value(&report, &alloc::format!("farm_{farm}_net_profit")), 10_500);
    }
}

#[test]
fn farm_rental_linked_taxable_grant_enters_form_4835_other_income_once() {
    let mut draft = empty_return(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::FarmRental { data: farm_rental_data(10_000, 0) }));
    let farm = draft.income[0].id;
    let mut paper = government_unemployment(0, &[], 37);
    let FederalIncome::GovernmentPayment { data } = &mut paper.income else { unreachable!() };
    data.form.taxable_grants = Some(50_000);
    add(&mut draft, paper);
    let FederalIncome::GovernmentPayment { data } = &draft.income[1].income else { unreachable!() };
    let component = data.component_ids.grant;
    claim_related_component(&mut draft, farm, component);
    let report = supported(&draft);
    assert_eq!(value(&report, &alloc::format!("farm_rental_{farm}_other_income")), 500);
    assert_eq!(value(&report, &alloc::format!("farm_rental_{farm}_gross_income")), 10_500);
    assert_eq!(value(&report, &alloc::format!("farm_rental_{farm}_program_payments")), 0);
    assert_eq!(return_line(&report, "8"), 1_050_000);
    assert_eq!(report.payment_withholding.as_ref().unwrap().forms_1099, 3_700);
}
