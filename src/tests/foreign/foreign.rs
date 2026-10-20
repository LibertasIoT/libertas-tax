// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::unit50_fuel::assert_fuel_payment;
use super::*;
use crate::tests::support::builders::survey::survey_ordinary_foreign_credit;

#[test]
fn foreign_presence_period_prorates_limit_and_rejects_329_days() {
    use crate::extended_rules::exclusion_days;
    assert_eq!(
        exclusion_days(&ForeignQualification::Presence { starts_on: 20260101, foreign_days: 330 }),
        Ok(365)
    );
    assert_eq!(
        exclusion_days(&ForeignQualification::Presence { starts_on: 20260701, foreign_days: 330 }),
        Ok(184)
    );
    assert_eq!(
        exclusion_days(&ForeignQualification::Presence { starts_on: 20250701, foreign_days: 330 }),
        Ok(181)
    );
    // The 330-day test is answered before the rules run (unknowns): only a
    // stored count above the period's own days is invalid here.
    assert!(matches!(
        exclusion_days(&ForeignQualification::Presence { starts_on: 20260101, foreign_days: 366 }),
        Err(("ISSUE_FOREIGN_PRESENCE_PERIOD_INVALID", _))
    ));
    assert!(matches!(
        exclusion_days(&ForeignQualification::Presence { starts_on: 20260230, foreign_days: 330 }),
        Err(("ISSUE_FOREIGN_PRESENCE_PERIOD_INVALID", _))
    ));
}

#[test]
fn foreign_exclusion_aggregates_employers_caps_per_person_and_stacks_tax() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(100000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    add(&mut d, income(foreign_pay(50000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
    assert_eq!(value(&r, "agi"), 17100);
    assert_eq!(value(&r, "taxable_income"), 1000);
    assert_eq!(value(&r, "income_tax"), 240);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(return_line(&r, "1z"), 15_000_000);
    assert_eq!(return_line(&r, "8"), -13_290_000);
    assert_eq!(return_line(&r, "9"), 1_710_000);
}

#[test]
fn foreign_exclusion_is_separate_for_joint_filers_and_removes_actc() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, income(foreign_pay(150000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let mut e = income(foreign_pay(150000));
    e.owner = Owner::Spouse;
    add(&mut d, e);
    set_claim(&mut d, foreign_claim(Owner::Spouse));
    let dep = child(d.next_id);
    d.next_id += 1;
    d.dependents.push(dep);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 265800);
    assert_eq!(value(&r, "agi"), 34200);
    assert_eq!(value(&r, "additional_child_credit"), 0);
    assert_eq!(value(&r, "earned_income_credit"), 0);
}

#[test]
fn foreign_nonexcluded_pay_is_taxable_without_invented_payroll_tax() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(60000)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(
        value(&r, "income_tax"),
        value(&supported(&complete(FederalStatus::Single)), "income_tax")
    );
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(return_line(&r, "1a"), 0);
    assert_eq!(return_line(&r, "1h"), 6_000_000);
}

#[test]
fn small_foreign_credit_election_caps_and_ordering() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Interest { data: native_interest(100000, 0) }));
    let baseline = supported(&d);
    payee_statement_credit(&mut d, 30000);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_tax_credit"), 300);
    assert_eq!(r.tax, baseline.tax.map(|v| v - 30000));
    raise_payee_statement_tax(&mut d, 1);
    assert_eq!(estimate(&d).state, ResultState::Unsupported);
}

#[test]
fn ordinary_foreign_credit_limits_both_regular_and_amt_taxes() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(100000)));
    d.credits.as_mut().unwrap().foreign_tax =
        survey_ordinary_foreign_credit(vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Country A employer".into(),
            category: ForeignTaxCategory::Employment,
            income: 10_000_000,
            tax: 3_000_000,
        }]);
    let r = supported(&d);
    assert_eq!(r.tax, Some(0));
    assert_eq!(value(&r, "foreign_tax_credit"), value(&r, "income_tax"));
    assert!(value(&r, "foreign_general_unused_tax") > 0);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    // The same inputs with an exclusion take the Form 1116 line 12 reduction
    // (foreign_form_1116_* below): the estimate is calculated, not refused.
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    assert_ne!(estimate(&d).state, ResultState::Unsupported);
}

#[test]
fn foreign_and_rental_income_preview_and_survive_restart() {
    // The M1 income page offers W-2, 1099-INT and 1099-DIV only; these records
    // are built directly on the draft until their milestone adds them to a page.
    let mut d = complete(FederalStatus::Single);
    let v = FederalIncome::Rental { data: rental(30000, 5000) };
    assert_eq!(crate::interview::document_amount(&v, false), Some(1_500_000));
    add(&mut d, income(v));
    add(&mut d, income(foreign_pay(10000)));
    assert_eq!(d.income.len(), 3);
    let restored = restart(d.clone());
    assert_eq!(restored.draft, d);
}

#[test]
fn foreign_trade_aviation_gasoline_rounds_one_2b_pool_and_separate_2d_lust() {
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
        aviation_gasoline_foreign_trade: Some(FederalAviationGasolineForeignTrade {
            fuel: fuel_use(21, 1),
            qualifying_civil_aircraft_trade: true,
            registration: FederalForeignTradeAircraftRegistration::Foreign {
                reciprocal_privileges: true,
            },
            distinct_gallons: true,
        }),
        ..FederalFuelLines::default()
    };
    assert!(fuels.valid() && fuels.qualified());
    assert_eq!(
        fuels
            .labeled_lines()
            .filter(|(line, _)| *line == "2b" || *line == "2d")
            .collect::<Vec<_>>(),
        vec![("2b", 2), ("2d", 0)]
    );
    let lust_rounding = FederalFuelLines {
        aviation_gasoline_foreign_trade: Some(FederalAviationGasolineForeignTrade {
            fuel: fuel_use(5_000, 1_000),
            qualifying_civil_aircraft_trade: true,
            registration: FederalForeignTradeAircraftRegistration::UnitedStates,
            distinct_gallons: true,
        }),
        ..FederalFuelLines::default()
    };
    assert!(lust_rounding.valid() && lust_rounding.qualified());
    assert_eq!(
        lust_rounding
            .labeled_lines()
            .filter(|(line, _)| *line == "2b" || *line == "2d")
            .collect::<Vec<_>>(),
        vec![("2b", 97), ("2d", 1)]
    );
    assert_eq!(lust_rounding.credit_cents(), 98);
    let mut insufficient_cost = lust_rounding;
    insufficient_cost.aviation_gasoline_foreign_trade.as_mut().unwrap().fuel.actual_cost = 97;
    assert!(!insufficient_cost.valid());
}

#[test]
fn foreign_trade_aviation_combines_kerosene_lust_with_original_tax_recovery() {
    let small = FederalFuelLines {
        aviation_kerosene_foreign_trade: Some(FederalForeignTradeAviationKerosene {
            taxed_at_244: Some(fuel_use(5_000, 20_000)),
            taxed_at_219: Some(fuel_use(5_000, 20_000)),
            no_waiver: true,
        }),
        ..FederalFuelLines::default()
    };
    assert_eq!(small.labeled_lines().find(|(name, _)| *name == "5e").unwrap().1, 1); // One combined 10-gallon line, not two separately rounded 5-gallon lines.

    let fuels = FederalFuelLines {
        aviation_gasoline_foreign_trade: Some(FederalAviationGasolineForeignTrade {
            fuel: fuel_use(1_000_000, 200_000),
            qualifying_civil_aircraft_trade: true,
            registration: FederalForeignTradeAircraftRegistration::UnitedStates,
            distinct_gallons: true,
        }),
        aviation_kerosene_foreign_trade: Some(FederalForeignTradeAviationKerosene {
            taxed_at_244: Some(fuel_use(100_000, 20_000)),
            taxed_at_219: Some(fuel_use(100_000, 20_000)),
            no_waiver: true,
        }),
        ..FederalFuelLines::default()
    };
    assert_eq!(
        fuels.labeled_lines().filter(|(_, v)| *v != 0).collect::<Vec<_>>(),
        vec![("2b", 19_300), ("2d", 100), ("5c", 2_430), ("5d", 2_180), ("5e", 20)]
    );
    let business_fuels = fuels.clone();
    let partner_fuels = fuels;
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 48_100);
    for source in ["business_2", "partnership_3"] {
        for (form_line, amount) in
            [("2b", 19_300), ("2d", 100), ("5c", 2_430), ("5d", 2_180), ("5e", 20)]
        {
            assert_eq!(line_amount(&report, &alloc::format!("{source}_fuel_{form_line}")), amount);
        }
    }

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .aviation_kerosene_foreign_trade
        .as_mut()
        .unwrap()
        .no_waiver = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.aviation_kerosene_foreign_trade.as_mut().unwrap().no_waiver = true;
    fuels.aviation_gasoline_foreign_trade.as_mut().unwrap().fuel.not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.aviation_gasoline_foreign_trade.as_mut().unwrap().fuel.not_claimed_elsewhere = true;
    fuels
        .aviation_kerosene_foreign_trade
        .as_mut()
        .unwrap()
        .taxed_at_244
        .as_mut()
        .unwrap()
        .actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels
        .aviation_kerosene_foreign_trade
        .as_mut()
        .unwrap()
        .taxed_at_244
        .as_mut()
        .unwrap()
        .actual_cost = 20_000;
    let trade = fuels.aviation_gasoline_foreign_trade.as_mut().unwrap();
    trade.fuel.qualified = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let trade = data
        .fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .aviation_gasoline_foreign_trade
        .as_mut()
        .unwrap();
    trade.fuel.qualified = true;
    trade.qualifying_civil_aircraft_trade = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let trade = data
        .fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .aviation_gasoline_foreign_trade
        .as_mut()
        .unwrap();
    trade.qualifying_civil_aircraft_trade = true;
    trade.registration =
        FederalForeignTradeAircraftRegistration::Foreign { reciprocal_privileges: false };
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let trade = data
        .fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .aviation_gasoline_foreign_trade
        .as_mut()
        .unwrap();
    trade.registration =
        FederalForeignTradeAircraftRegistration::Foreign { reciprocal_privileges: true };
    trade.distinct_gallons = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let trade = data
        .fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .aviation_gasoline_foreign_trade
        .as_mut()
        .unwrap();
    trade.distinct_gallons = true;
    trade.fuel.actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let trade = data
        .fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .aviation_gasoline_foreign_trade
        .as_mut()
        .unwrap();
    trade.fuel.actual_cost = 20_000;
    trade.fuel.thousandths_of_gallon = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
}

#[test]
fn marketplace_repayment_increases_payee_statement_foreign_credit_limit() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 1510000, 0);
    add(&mut d, income(FederalIncome::Interest { data: native_interest(100000, 0) }));
    payee_statement_credit(&mut d, 30000);
    let before = supported(&d);
    assert_eq!(value(&before, "foreign_tax_credit"), 0);
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, (1..=12).map(|m| marketplace_month(m, 500, 100, 500)).collect());
    let r = supported(&d);
    let repayment = value(&r, "premium_credit_repayment");
    assert!(repayment > 300);
    assert_eq!(return_line(&r, "16"), 0);
    assert_eq!(value(&r, "foreign_tax_credit"), 300);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(return_line(&r, "20"), 30000);
    assert_eq!(r.tax, Some((repayment - 300) * 100));
}

#[test]
fn monthly_schedule_c_net_combines_with_w2_foreign_and_iso_once() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.dependents.pop();
    d.dependents[0].set_care_expenses(300_000);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 10_000, 0));
    let mut foreign = income(foreign_pay(100));
    foreign.owner = Owner::Spouse;
    add(&mut d, foreign);
    let mut sale = disqualifying_iso_sale();
    sale.exercised = 20260201;
    sale.sold = 20260421;
    sale.proceeds = 1_005_000;
    let mut iso = income(FederalIncome::IsoSale { data: sale });
    iso.owner = Owner::Spouse;
    add(&mut d, iso);
    let mut business = income(cash_business(300));
    business.owner = Owner::Spouse;
    add(&mut d, business);
    let mut months = care_months(Owner::Spouse, 5);
    months[0].foreign_pay = 10_000;
    months[1].wages = 10_000;
    months[2].business_net = 30_000;
    set_care_months(&mut d, months);

    care_aggregate(&mut d, Owner::Spouse, 55_000);
    let r = supported(&d);
    // Actual $550 plus only the $750 needed to reach the five monthly floors.
    assert_eq!(value(&r, "care_earned_spouse"), 1_250);
    assert_eq!(value(&r, "care_credit"), 438);
    assert_eq!(value(&r, "foreign_wages"), 100);

    assert_eq!(value(&r, "iso_unreported_compensation"), 50);
    assert_eq!(value(&r, "agi"), 100_550);
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).return_lines, r.return_lines);
}
#[test]
fn taxable_foreign_pay_uses_actual_student_month_with_w2_and_iso() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.dependents.pop();
    d.dependents[0].set_care_expenses(300000);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 20_000, 0));
    let mut foreign = income(foreign_pay(1_000));
    foreign.owner = Owner::Spouse;
    add(&mut d, foreign);
    let mut sale = disqualifying_iso_sale();
    sale.exercised = 20260201;
    sale.sold = 20260421;
    sale.proceeds = 1_020_000;
    let mut iso = income(FederalIncome::IsoSale { data: sale });
    iso.owner = Owner::Spouse;
    add(&mut d, iso);
    let mut months = care_months(Owner::Spouse, 5);
    months[0].foreign_pay = 100_000; // January's $1,000 exceeds the $250 deemed amount.
    months[1].wages = 20_000; // February's $200 leaves $50 deemed income.
    set_care_months(&mut d, months);

    care_aggregate(&mut d, Owner::Spouse, 140_000);
    let r = supported(&d);
    // $1,400 actual ($1,000 foreign, $200 W-2, $200 ISO) plus $600
    // deemed across February through May; there is no annual spreading.
    assert_eq!(value(&r, "care_earned_spouse"), 1_400);
    assert_eq!(value(&r, "care_credit"), 490);
    assert_eq!(value(&r, "agi"), 101_400);
    assert_eq!(return_line(&r, "1a"), 10_020_000);
    assert_eq!(return_line(&r, "1h"), 100_000);
    let cents = |id: &str| line_amount(&r, id);
    assert_eq!(cents("foreign_wages"), 100_000);

    assert_eq!(cents("iso_unreported_compensation"), 20_000);
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);
}

#[test]
fn taxable_foreign_pay_uses_actual_self_care_months_across_employers() {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().incapable_self_care = true;
    let mut first = income(foreign_pay(600));
    first.owner = Owner::Spouse;
    add(&mut d, first);
    let mut second = income(foreign_pay(400));
    second.owner = Owner::Spouse;
    add(&mut d, second);
    let mut months = care_months(Owner::Spouse, 0);
    for month in &mut months {
        month.incapable = month.month <= 11;
    }
    months[0].foreign_pay = 60_000;
    months[11].foreign_pay = 40_000;
    d.credits.as_mut().unwrap().care = FederalCare { prior_year: None, ..FederalCare::default() };
    set_care_months(&mut d, months);
    if let CareFilerSituation::Incapable { care_paid, .. } =
        &mut d.credits.as_mut().unwrap().care.filers[0].situation
    {
        *care_paid = 330_000;
    }
    document_care(&mut d);
    let r = supported(&d);
    // January's $600 beats $250; ten other self-care months add $250 each.
    // December's $400 is ordinary actual pay, with no deemed addition.
    assert_eq!(value(&r, "care_earned_spouse"), 3_150);
    assert_eq!(value(&r, "foreign_wages"), 1_000);

    assert_eq!(return_line(&r, "1h"), 100_000);
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).return_lines, r.return_lines);
}

#[test]
fn aggregate_foreign_care_pay_uses_selected_month_earnings_without_confirmation_flags() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.dependents.pop();
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let mut foreign = income(foreign_pay(1_000));
    foreign.owner = Owner::Spouse;
    add(&mut d, foreign);
    let mut months = care_months(Owner::Spouse, 5);
    months[0].foreign_pay = 100_000;
    set_care_months(&mut d, months);
    // The approved aggregate lower bound replaces individual payroll months
    // and their source confirmations: max($1,000, 5 * $250) = $1,250.
    let r = supported(&d);
    assert_eq!(value(&r, "care_earned_spouse"), 1_250);
    assert!(noted(&r, "WARN_CARE_DEEMED_INCOME_MONTHLY"));
    let mut aggregate = d.clone();
    if let CareFilerSituation::Student { earned_in_months, .. } =
        &mut aggregate.credits.as_mut().unwrap().care.filers[0].situation
    {
        *earned_in_months = Some(99_900);
    }
    supported(&aggregate); // A supplied aggregate is not an invented month reconciliation.
    if let CareFilerSituation::Student { earned_in_months, .. } =
        &mut aggregate.credits.as_mut().unwrap().care.filers[0].situation
    {
        *earned_in_months = Some(-1);
    }
    assert!(!aggregate.valid_stored());
    let spouse_owner = d.income[1].owner;
    let mut excluded = d.clone();
    set_claim(&mut excluded, foreign_claim(spouse_owner));
    supported(&excluded);
    let mut housed = d.clone();
    let mut claim = foreign_claim(spouse_owner);
    claim.exclusion = false;
    claim.housing = housing_claim(0, ForeignHousingLocation::General, None);
    set_claim(&mut housed, claim);
    supported(&housed);
    let mut business = d.clone();
    let mut entry = income(foreign_service_business(0));
    entry.owner = Owner::Spouse;
    add(&mut business, entry);
    supported(&business);
    let mut care_benefits = d.clone();
    add(&mut care_benefits, wage(20, Owner::Spouse, 0, 0));
    let id = w2(&care_benefits, 20);
    care_w2(&mut care_benefits, id, 500_000, 0, 0, 750_000);
    supported(&care_benefits);
    let mut adoption = d;
    employer_adoption(&mut adoption, 20_000, 0, false);
    if let AdoptionBenefits::Program { owner, .. } =
        &mut adoption.credits.as_mut().unwrap().adoptions[0].benefits
    {
        *owner = Owner::Spouse;
    }
    supported(&adoption);
}

#[test]
fn saver_credit_adds_back_foreign_exclusion() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(150000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    d.credits.as_mut().unwrap().saver = Some(saver(200000, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 17100);
    assert_eq!(value(&r, "saver_credit"), 0);
    assert_eq!(value(&r, "income_tax"), 240);
}

#[test]
fn foreign_magi_prevents_unnecessary_ira_saver_history_requirement() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 2000000, 0));
    add(&mut d, income(foreign_pay(50000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    d.adjustments.as_mut().unwrap().iras = ira_facts(100000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = None.into();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 19000);
    assert_eq!(value(&r, "saver_credit"), 0);
}

#[test]
fn zero_regular_taxable_income_does_not_bypass_foreign_kiddie_tax_stacking() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(100000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    add(&mut d, income(savings_income(3000, 0)));
    let p = &mut d.people.as_mut().unwrap().taxpayer;
    p.birth_date = 20060615;
    p.full_time_student = true;
    p.support_share = SupportShare::LessThanHalf;
    unsupported(&d, "Investment tax");
}

#[test]
fn short_term_collectible_and_foreign_long_term_collectible_use_correct_rates() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(collectible_sale(4000, 1000, GainTerm::Short, false)));
    let r = supported(&d);
    assert_eq!(value(&r, "rate_28_gain"), 0);
    assert_eq!(value(&r, "income_tax"), 5383);
    d.income[1].income = collectible_sale(4000, 1000, GainTerm::Long, false);
    add(&mut d, income(foreign_pay(10000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    // Taxable46900 plus exclusion10000: tax on56900 minus tax on10000.
    // The collectible stays in ordinary brackets below the28% ceiling.
    assert_eq!(supported(&d).tax, Some(623300));
}

#[test]
fn employer_adoption_prior_limit_and_foreign_finality() {
    let mut d = complete(FederalStatus::Single);
    employer_adoption(&mut d, 10000, 15000, false);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 2670);
    assert_eq!(return_line(&r, "1f"), 733000);
    d.credits.as_mut().unwrap().adoptions[0].kind =
        AdoptionKind::Foreign { finalized: AdoptionFinalization::Pending };
    unsupported(&d, "Adoption benefits");
    if let AdoptionBenefits::Program { history, .. } =
        &mut d.credits.as_mut().unwrap().adoptions[0].benefits
    {
        *history = AdoptionBenefitHistory::None;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 0);
    assert_eq!(value(&r, "agi"), 70000);
    let child = &mut d.credits.as_mut().unwrap().adoptions[0];
    child.kind = AdoptionKind::Foreign { finalized: AdoptionFinalization::Final { year: 2026 } };
    child.benefits = AdoptionBenefits::Program {
        owner: Owner::Taxpayer,
        received: 1000000,
        history: AdoptionBenefitHistory::None,
        ordinary: true,
    };
    assert_eq!(value(&supported(&d), "adoption_benefits_excluded"), 10000);
}

#[test]
fn prior_foreign_benefits_increase_exclusion_base_without_being_taxed_twice() {
    let mut d = foreign_adoption_benefits(10000, 10000);
    d.income[0] = wage(1, Owner::Taxpayer, 27508000, 0);
    let r = supported(&d);
    // Exclusion MAGI includes only the 2026 benefit: 275,080 + 10,000.
    // The finalization worksheet applies 50% to min(20,000, 17,670).
    assert_eq!(value(&r, "adoption_exclusion_magi"), 285080);
    assert_eq!(value(&r, "adoption_prior_taxed_benefits"), 10000);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 8835);
    assert_eq!(return_line(&r, "1f"), 116500);
    assert_eq!(value(&r, "agi"), 276245);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn pending_foreign_benefits_wait_for_finalization_without_retaxing_prior_payments() {
    let mut d = foreign_adoption_benefits(10000, 10000);
    d.credits.as_mut().unwrap().adoptions[0].kind =
        AdoptionKind::Foreign { finalized: AdoptionFinalization::Pending };
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_prior_taxed_benefits"), 0);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 0);
    assert_eq!(return_line(&r, "1f"), 1000000);
    assert_eq!(value(&r, "agi"), 70000);
}

#[test]
fn prior_foreign_benefits_respect_shared_limit_and_full_phaseout() {
    let mut d = foreign_adoption_benefits(10000, 10000);
    d.credits.as_mut().unwrap().adoptions[0].limits =
        AdoptionLimits::Shared { credit: 1767000, exclusion: 1200000, agreed: true };
    d.income[0] = wage(1, Owner::Taxpayer, 25508000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 12000);
    assert_eq!(return_line(&r, "1f"), -200000);
    assert_eq!(value(&r, "agi"), 253080);
    d.income[0] = wage(1, Owner::Taxpayer, 27508000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 6000);
    assert_eq!(return_line(&r, "1f"), 400000);
    d.income[0] = wage(1, Owner::Taxpayer, 29508000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "adoption_benefits_excluded"), 0);
    assert_eq!(return_line(&r, "1f"), 1000000);
    assert_eq!(value(&r, "agi"), 305080);
}

#[test]
fn prior_foreign_benefit_history_rejects_wrong_adoption_and_invalid_persistence() {
    let mut d = foreign_adoption_benefits(20000, 10000);
    for kind in [
        AdoptionKind::Domestic {
            finalized: AdoptionFinalization::Final { year: 2026 },
            special_needs: false,
        },
        AdoptionKind::Foreign { finalized: AdoptionFinalization::Final { year: 2025 } },
    ] {
        d.credits.as_mut().unwrap().adoptions[0].kind = kind;
        unsupported(&d, "Adoption benefit history");
    }
    if let AdoptionBenefits::Program { history, .. } =
        &mut d.credits.as_mut().unwrap().adoptions[0].benefits
    {
        *history = AdoptionBenefitHistory::TaxedForeign { received: -1 };
    }
    assert!(!crate::adoption::valid(&d.credits.as_ref().unwrap().adoptions, &[]));
}

#[test]
fn prior_foreign_benefit_exclusion_reduces_current_wages_after_finalization() {
    for (current, earlier, exclusion, line_1f, agi) in
        [(10000, 10000, 17670, -7670, 52330), (0, 10000, 10000, -10000, 50000)]
    {
        let d = foreign_adoption_benefits(current, earlier);
        let r = supported(&d);
        assert_eq!(value(&r, "adoption_prior_taxed_benefits"), earlier);
        assert_eq!(value(&r, "adoption_benefits_excluded"), exclusion);
        assert_eq!(value(&r, "adoption_wage_adjustment"), line_1f);
        assert_eq!(return_line(&r, "1a"), 6000000);
        assert_eq!(return_line(&r, "1f"), line_1f * 100);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_atnol_keeps_unimplemented_foreign_allocation_explicit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Interest { data: native_interest(100000, 0) }));
    d.credits.as_mut().unwrap().foreign_tax =
        survey_ordinary_foreign_credit(vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Synthetic foreign fund".into(),
            category: ForeignTaxCategory::Passive {
                sales: vec![],
                withholding: ForeignWithholding::Below15,
                qualified: 0,
            },
            income: 100000,
            tax: 10000,
        }]);
    supported(&d);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 1000, false)]);
    unsupported(&d, "AMT loss history");
}
#[test]
fn prior_domestic_nol_offsets_nonexcluded_foreign_wages_without_payroll_tax() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(60000)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 35120);
    assert_eq!(value(&r, "agi"), 24880);
    assert_eq!(value(&r, "nol_2025_carryforward"), 64880);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(return_line(&r, "1h"), 6000000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_domestic_nol_preserves_foreign_exclusion_and_tax_stacking() {
    for (year, deduction, closing, taxable, tax) in
        [(2025, 800, 99200, 200, 48), (2017, 100000, 99000, 0, 0)]
    {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(foreign_pay(150000)));
        set_claim(&mut d, foreign_claim(Owner::Taxpayer));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(year, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, "agi"), 17100 - deduction);
        assert_eq!(value(&r, "taxable_income"), taxable);
        assert_eq!(value(&r, "regular_income_tax"), tax);
        assert_eq!(value(&r, &alloc::format!("nol_{year}_carryforward")), closing);
        assert_eq!(value(&r, "earned_income_credit"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn fully_excluded_foreign_pay_does_not_consume_domestic_nol() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(100000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 100000);
    assert_eq!(value(&r, "prior_nol_deduction"), 0);
    assert_eq!(value(&r, "nol_2025_carryforward"), 100000);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_domestic_nol_foreign_exclusion_addback_preserves_niit_threshold() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    add(&mut d, income(foreign_pay(150000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    add(&mut d, income(savings_income(20000, 0)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1000000)]);
    // AMT dominates here. Automatic selection uses zero itemized deductions
    // to allow more NOL and lower NIIT; compare the explicit standard route.
    let automatic = supported(&d);
    assert_eq!(value(&automatic, "prior_nol_deduction"), 269680);
    assert_eq!(value(&automatic, "net_investment_income_tax"), 12);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
    assert_eq!(value(&r, "prior_nol_deduction"), 256800);
    assert_eq!(value(&r, "agi"), 80300);
    assert_eq!(value(&r, "net_investment_income_tax"), 502);
    assert_eq!(value(&r, "nol_2025_carryforward"), 743200);
    assert_eq!(restart(d.clone()).draft, d);
    d.credits.as_mut().unwrap().foreign_tax =
        survey_ordinary_foreign_credit(vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Synthetic foreign wages".into(),
            category: ForeignTaxCategory::Employment,
            income: 15000000,
            tax: 1000000,
        }]);
    unsupported(&d, "Regular loss history");
}

#[test]
fn prior_regular_and_amt_losses_keep_foreign_tax_stacking_and_distinct_balances() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(400000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1000000)]);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 100000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 200800);
    assert_eq!(value(&r, "agi"), 66300);
    assert_eq!(value(&r, "regular_income_tax"), 12048);
    assert_eq!(value(&r, "prior_atnol_deduction"), 100000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 167100);
    assert_eq!(value(&r, "alternative_minimum_tax"), 7972);
    assert_eq!(value(&r, "nol_2025_carryforward"), 799200);
    assert_eq!(value(&r, "atnol_2025_carryforward"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn netted_away_amt_special_gains_do_not_block_foreign_exclusion() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(150000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    add(&mut d, income(collectible_sale(5000, 0, GainTerm::Long, false)));
    add(&mut d, income(ordinary_investment_sale(0, 10000, GainTerm::Long)));
    d.screening.as_mut().unwrap().capital_history = vec![amt_only_capital_history()];
    let r = supported(&d);
    assert_eq!(value(&r, "amt_rate_28_gain"), 0);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 7000);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 5000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn draft_2026_table_is_used_inside_gain_and_foreign_worksheets() {
    use crate::income_tax::{Preference, RegularTaxWorksheet};
    let ordinary = RegularTaxWorksheet {
        taxable: 100_000,
        preference: Preference { dividends: 50_000, gain: 0, line18: 0, line19: 0 },
        excluded: 0,
        foreign_election: false,
        status: FederalStatus::Single,
    };
    // Schedule D's ordinary component is below $100,000 even though line 15 is not.
    assert_eq!(ordinary.tax(), draft_2026_table_tax(50_000, FederalStatus::Single) + 7_500);
    let foreign = RegularTaxWorksheet {
        taxable: 50_000,
        preference: Preference { dividends: 0, gain: 0, line18: 0, line19: 0 },
        excluded: 10_000,
        foreign_election: true,
        status: FederalStatus::Single,
    };
    assert_eq!(
        foreign.tax(),
        draft_2026_table_tax(60_000, FederalStatus::Single)
            - draft_2026_table_tax(10_000, FederalStatus::Single)
    );
}

#[test]
fn foreign_exclusion_caps_preference_before_stacking_regular_and_amt_tax() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(100000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    add(&mut d, income(qualified_dividend(70000)));
    let r = supported(&d);
    // Taxable53900 is all at15%; excluded100000 cannot absorb extra preference.
    assert_eq!(value(&r, "regular_income_tax"), 8085);
    assert_eq!(r.tax, Some(808500));
    // AMT base150000 - exemption90100 =59900, all preference at15%.
    assert_eq!(
        crate::extended_rules::amt_tax(150000, 70000, 100000, FederalStatus::Single, 100000),
        8985
    );
}

#[test]
fn foreign_special_gains_use_separate_regular_and_amt_excess_pools() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(100000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    add(&mut d, income(foreign_special_distribution(200000, 80000, 60000, 20000)));
    let r = supported(&d);
    // Regular excess16100: D18=63900,D19=60000, preference203900.
    // Stacked303900; ordinary201775 tax41024 +22125*.28 +80000*.15
    // less tax16712 on exclusion100000 =42507.
    assert_eq!(value(&r, "regular_income_tax"), 42507);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(r.tax, Some(4706700)); // NIIT4560.
    assert_eq!(supported(&restart(d.clone()).draft).tax, r.tax);
    let source = crate::income_tax::Preference {
        dividends: 20000,
        gain: 200000,
        line18: 80000,
        line19: 60000,
    };
    assert_eq!(source.foreign(129900).capped(), [129900, 0, 49900]);
    // AMT's own90,100 excess leaves49,900 at25% and80,000 at15%.
    assert_eq!(
        crate::extended_rules::amt_tax_with_foreign_gains(
            220000,
            80000,
            223900,
            FederalStatus::Single,
            100000,
            [80000, 60000],
            Some(source)
        ),
        24475
    );
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Foreign gain AMT", 1000.0, 1000, 21000)];
    let r = supported(&d);
    // AMTI420000 less90100, plus100000 stacking; AMT excess now zero.
    // Tax289900 at26/28% +60000*.25 +80000*.15 -26000 =77282.
    assert_eq!(value(&r, "alternative_minimum_tax"), 34775);
    assert_eq!(r.tax, Some(8184200));
}

#[test]
fn ordinary_foreign_credit_excludes_senior_deduction_from_worldwide_limit() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    add(&mut d, income(foreign_pay(40000)));
    d.credits.as_mut().unwrap().foreign_tax =
        survey_ordinary_foreign_credit(vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign employer".into(),
            category: ForeignTaxCategory::Employment,
            income: 4000000,
            tax: 2000000,
        }]);
    let r = supported(&d);
    // AGI100,000, standard18,150, enhanced senior4,500. Taxable77,350,
    // regular tax11,729. Form1116 denominator81,850 and foreign numerator
    // 40,000 - (18,150 *40%) =32,740: credit40% *11,729 =4,692.
    assert_eq!(value(&r, "senior_deduction"), 4500);
    assert_eq!(value(&r, "taxable_income"), 77350);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 81850);
    assert_eq!(value(&r, "foreign_tax_credit"), 4694);
    assert_eq!(value(&r, "foreign_general_unused_tax"), 15306);
    assert_eq!(r.tax, Some(704100));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::No;
    let before_credit = supported(&d);
    assert_eq!(before_credit.tax, Some(1173500));
    assert_eq!(value(&before_credit, "senior_deduction"), 4500);
}

#[test]
fn ordinary_foreign_credit_senior_worldwide_income_uses_unfloored_deductions() {
    for (income_total, worldwide) in [(10000, 0), (20000, 1850)] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
        d.income.clear();
        add(&mut d, income(foreign_pay(income_total)));
        d.credits.as_mut().unwrap().foreign_tax =
            survey_ordinary_foreign_credit(vec![ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Foreign employer".into(),
                category: ForeignTaxCategory::Employment,
                income: income_total * 100,
                tax: 100000,
            }]);
        let r = supported(&d);
        assert_eq!(value(&r, "taxable_income"), 0);
        assert_eq!(value(&r, "senior_deduction"), 6000);
        assert_eq!(form_1116_any(&r, false, "_worldwide_income"), worldwide);
        assert_eq!(value(&r, "foreign_tax_credit"), 0);
        assert_eq!(r.tax, Some(0));
    }
}

#[test]
fn ordinary_foreign_credit_joint_senior_phaseout_keeps_both_deductions_out_of_limit() {
    let mut d = complete(FederalStatus::Joint);
    let people = d.people.as_mut().unwrap();
    people.taxpayer.birth_date = 19500101;
    people.spouse.as_mut().unwrap().birth_date = 19500101;
    d.income = vec![wage(1, Owner::Taxpayer, 9600000, 0)];
    add(&mut d, income(foreign_pay(64000)));
    d.credits.as_mut().unwrap().foreign_tax =
        survey_ordinary_foreign_credit(vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign employer".into(),
            category: ForeignTaxCategory::Employment,
            income: 6400000,
            tax: 2000000,
        }]);
    let r = supported(&d);
    // Each senior's6,000 deduction is reduced by6% of10,000: total10,800.
    // Standard35,500; worldwide124,500; actual taxable113,700. Regular
    // tax14,438, foreign share40%, credit5,775 after whole-dollar rounding.
    assert_eq!(value(&r, "senior_deduction"), 10800);
    assert_eq!(value(&r, "taxable_income"), 113700);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 124500);
    assert_eq!(value(&r, "foreign_tax_credit"), 5775);
    assert_eq!(r.tax, Some(866300));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn ordinary_foreign_credit_allocates_ira_using_gross_income_for_regular_and_amt() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12000000, 0);
    add(&mut d, income(foreign_pay(80000)));
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::No);
    d.credits.as_mut().unwrap().foreign_tax =
        survey_ordinary_foreign_credit(vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign employer".into(),
            category: ForeignTaxCategory::Employment,
            income: 8000000,
            tax: 2000000,
        }]);
    let r = supported(&d);
    // Gross200,000; IRA7,500; standard16,100; taxable176,400.
    // Regular tax34,934. Foreign net70,560 /176,400 =40%.
    // AMTI192,500; exemption90,100; tentative26,624.
    // AMT foreign net77,000 /192,500 =40%, not80,000 /192,500.
    assert_eq!(value(&r, "ira_deduction"), 7500);
    assert_eq!(value(&r, "agi"), 192500);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 176400);
    assert_eq!(value(&r, "income_tax"), 34934);
    assert_eq!(value(&r, "foreign_tax_credit"), 13974);
    assert_eq!(value(&r, "foreign_general_unused_tax"), 6026);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 9350);
    assert_eq!(r.tax, Some(2096000));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    // The same gross allocation applies to passive ordinary interest.
    let mut passive = d.clone();
    passive.income[1].income = FederalIncome::Interest { data: native_interest(8000000, 0) };
    if let ForeignTaxClaim::Ordinary { sources, .. } =
        &mut passive.credits.as_mut().unwrap().foreign_tax
    {
        sources[0].category = ForeignTaxCategory::Passive {
            sales: vec![],
            withholding: ForeignWithholding::AtLeast15,
            qualified: 0,
        };
    }
    let passive_result = supported(&passive);
    assert_eq!(value(&passive_result, "foreign_tax_credit"), 13974);
    assert_eq!(value(&passive_result, "foreign_passive_amt_unused_tax"), 9350);
    assert_eq!(passive_result.tax, r.tax);
    // An income-based IRA phaseout supplies zero; contributions are not deductions.
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::Yes);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
    data.retirement_plan = true;
    assert_eq!(value(&supported(&d), "ira_deduction"), 0);
}

#[test]
fn ordinary_foreign_credit_reconciles_foreign_gross_above_agi_after_ira() {
    for gross in [7500, 100000] {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(foreign_pay(gross)));
        d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::No);
        d.credits.as_mut().unwrap().foreign_tax =
            survey_ordinary_foreign_credit(vec![ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Only foreign employer".into(),
                category: ForeignTaxCategory::Employment,
                income: gross * 100,
                tax: 3000000,
            }]);
        d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), gross - 7500);
        assert_eq!(r.tax, Some(0));
        assert_eq!(value(&r, "foreign_tax_credit"), value(&r, "income_tax"));
        assert_eq!(
            value(&r, "foreign_general_amt_unused_tax"),
            if gross == 7500 { 30000 } else { 29376 }
        );
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn foreign_high_tax_classification_differs_for_regular_and_amt() {
    let d = foreign_high_tax_return(12000);
    let r = supported(&d);
    // Gross200,000; standard16,100; regular36,734; AMT28,574.
    // Passive regular net36,780:12,000 does not exceed37%.
    // AMT passive net40,000:12,000 exceeds26%, joins60,000 foreign pay.
    assert_eq!(value(&r, "income_tax"), 36734);
    assert_eq!(value(&r, "foreign_high_tax_net_income"), 0);
    assert_eq!(value(&r, "foreign_amt_high_tax_net_income"), 40000);
    assert_eq!(value(&r, "foreign_amt_high_tax_taxes"), 12000);
    assert_eq!(value(&r, "foreign_tax_credit"), 7347);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 4653);
    assert_eq!(value(&r, "foreign_passive_amt_unused_tax"), 0);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 0);
    assert_eq!(r.tax, Some(2938700));
    assert_eq!(supported(&restart(d).draft), r);
    let r = supported(&foreign_high_tax_return(15000));
    assert_eq!(value(&r, "foreign_high_tax_net_income"), 36780);
    assert_eq!(value(&r, "foreign_high_tax_taxes"), 15000);
    assert_eq!(value(&r, "foreign_tax_credit"), 15000);
    assert_eq!(value(&r, "foreign_general_unused_tax"), 0);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 713);
    assert_eq!(r.tax, Some(2173400));
}

#[test]
fn foreign_high_tax_groups_direct_sources_before_reclassification() {
    let mut d = foreign_high_tax_return(11000);
    let aggregate = supported(&d);
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[1].income = 2000000;
        sources[1].tax = 800000;
        let mut second = sources[1].clone();
        second.label = "Country C bank".into();
        second.tax = 300000;
        sources.push(second);
    }
    let grouped = supported(&d);
    assert_eq!(grouped, aggregate);
    // A different withholding treatment must not dilute the first group's rate.
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[2].category = ForeignTaxCategory::Passive {
            sales: vec![],
            withholding: ForeignWithholding::None,
            qualified: 0,
        };
    }
    let separate = supported(&d);
    assert_eq!(value(&separate, "foreign_high_tax_net_income"), 18390);
    assert_eq!(value(&separate, "foreign_high_tax_taxes"), 8000);
    assert_eq!(value(&separate, "foreign_amt_high_tax_net_income"), 20000);
    assert_eq!(value(&separate, "foreign_tax_credit"), 11000);
    assert_eq!(supported(&restart(d.clone()).draft), separate);
    // Untaxed passive income is its own group, including when the taxed
    // source has assessed tax without withholding. It cannot dilute that tax.
    for withholding in
        [ForeignWithholding::AtLeast15, ForeignWithholding::Below15, ForeignWithholding::None]
    {
        if let ForeignTaxClaim::Ordinary { sources, .. } =
            &mut d.credits.as_mut().unwrap().foreign_tax
        {
            sources[1].category =
                ForeignTaxCategory::Passive { withholding, qualified: 0, sales: vec![] };
            sources[2].tax = 0;
        }
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_high_tax_net_income"), 18390);
        assert_eq!(value(&r, "foreign_high_tax_taxes"), 8000);
        assert_eq!(value(&r, "foreign_amt_high_tax_net_income"), 20000);
    }
}

#[test]
fn foreign_high_tax_threshold_is_strict_and_amt_rate_uses_existing_brackets() {
    for (tax, expected) in [(10400, 0), (10401, 40000)] {
        let r = supported(&foreign_high_tax_return(tax));
        assert_eq!(value(&r, "foreign_amt_high_tax_net_income"), expected);
    }
    for (tax, expected) in [(13608, 0), (13609, 36780)] {
        let r = supported(&foreign_high_tax_return(tax));
        assert_eq!(value(&r, "foreign_high_tax_net_income"), expected);
    }
    for (domestic, expected) in [(234600, 40000), (234601, 0)] {
        let mut d = foreign_high_tax_return(10800);
        d.income[0] = wage(1, Owner::Taxpayer, domestic * 100, 0);
        assert_eq!(value(&supported(&d), "foreign_amt_high_tax_net_income"), expected);
    }
    // Above the AMT26% bracket,27% foreign tax is not high-taxed under28%.
    let mut d = foreign_high_tax_return(10800);
    d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_amt_high_tax_net_income"), 0);
    // Withholding assertion cannot coexist with zero paid tax.
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[1].tax = 0;
    }
    unsupported(&d, "Foreign withholding");
}

#[test]
fn simplified_foreign_amt_election_preserves_choice_even_when_tax_increases() {
    let mut d = complete(FederalStatus::Single);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    d.income.clear();
    add(&mut d, income(foreign_pay(200000)));
    d.credits.as_mut().unwrap().foreign_tax =
        survey_ordinary_foreign_credit(vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign employer".into(),
            category: ForeignTaxCategory::Employment,
            income: 20000000,
            tax: 10000000,
        }]);
    let normal = supported(&d);
    assert_eq!(normal.tax, Some(0));
    assert_eq!(form_1116_value(&normal, "foreign_amt_general_net_foreign_income"), 200000);
    for year in [1998, 2020, 2026] {
        set_foreign_amt_election(&mut d, year, true);
        let r = supported(&d);
        // Regular net183,900; AMTI200,000; tentative28,574.
        // Elected AMT credit28,574 *183,900 /200,000 =26,274.
        // Regular credit36,734 eliminates regular tax, leaving AMT2,300.
        assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 183900);
        assert_eq!(value(&r, "foreign_tax_credit"), 36734);
        assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 73726);
        assert_eq!(value(&r, "alternative_minimum_tax"), 2300);
        assert_eq!(r.tax, Some(230000));
        assert_eq!(supported(&restart(d.clone()).draft), r);
        assert!(matches!(&d.credits.as_ref().unwrap().foreign_tax,
            ForeignTaxClaim::Ordinary { amt_limitation: ForeignAmtLimitation::Simplified { first_claim_year, .. }, .. }
            if *first_claim_year == year));
    }
}

#[test]
fn simplified_foreign_amt_limit_reuses_regular_category_income_not_tax_classification() {
    let mut d = foreign_high_tax_return(12000);
    set_foreign_amt_election(&mut d, 2020, true);
    let r = supported(&d);
    // The AMT-only high-tax move carries12,000 tax to general income.
    // Elected general numerator stays regular55,170, not AMT100,000.
    // 28,574 *55,170 /200,000 =7,882; remainingAMT tax4,118.
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 55170);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 36780);
    assert_eq!(value(&r, "foreign_amt_high_tax_taxes"), 12000);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 4118);
    assert_eq!(value(&r, "foreign_passive_amt_unused_tax"), 0);
    assert_eq!(value(&r, "foreign_tax_credit"), 7347);
    assert_eq!(r.tax, Some(2938700));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn simplified_foreign_amt_election_requires_first_year_and_no_takes_the_normal_method() {
    let mut d = foreign_high_tax_return(12000);
    for year in [1997, 2027] {
        set_foreign_amt_election(&mut d, year, true);
        assert!(!d.valid_stored());
    }
    // No: the election is not in force, so the normal AMT limitation applies.
    set_foreign_amt_election(&mut d, 2020, false);
    let unconfirmed = supported(&d);
    if let ForeignTaxClaim::Ordinary { amt_limitation, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *amt_limitation = ForeignAmtLimitation::Normal;
    }
    assert_eq!(supported(&d).tax, unconfirmed.tax);
}

#[test]
fn foreign_amt_denominator_includes_married_separate_income_addition_once() {
    // The existing section55 owner supplies the addition, not a second formula.
    // Independently:50% over640,200, capped70,100; TMT28% minus2,445.
    for (base, amti, unused, tax, simplified_tax) in [
        (640200, 640200, 223189, 0, 4447),
        (640202, 640203, 223188, 0, 4447),
        (700000, 729900, 206345, 8272, 12726),
        (780400, 850500, 183731, 19426, 23888),
        (780402, 850502, 183731, 19427, 23888),
    ] {
        let mut d = wholly_foreign_credit_return(FederalStatus::Separate, base);
        let r = supported(&d);
        assert_eq!(value(&r, "alternative_minimum_taxable_income"), amti);
        assert_eq!(form_1116_any(&r, true, "_worldwide_income"), amti);
        assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), base);
        assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), base);
        assert_eq!(value(&r, "foreign_general_amt_unused_tax"), unused);
        assert_eq!(value(&r, "alternative_minimum_tax"), tax);
        assert_eq!(r.tax, Some(tax * 100));
        assert_eq!(supported(&restart(d.clone()).draft), r);
        set_foreign_amt_election(&mut d, 2020, true);
        let simplified = supported(&d);
        assert_eq!(form_1116_any(&simplified, true, "_worldwide_income"), amti);
        assert_eq!(
            form_1116_value(&simplified, "foreign_amt_general_net_foreign_income"),
            base - 16100
        );
        assert_eq!(simplified.tax, Some(simplified_tax * 100));
    }
}

#[test]
fn foreign_amt_state_tax_refund_uses_amt_gross_for_ira_allocation() {
    for (ira, agi, amt_income, tax, simplified_tax, unused) in
        [(0, 201000, 200000, 184, 2289, 371426), (7500, 193500, 192500, 175, 2211, 373376)]
    {
        let mut d = wholly_foreign_credit_return(FederalStatus::Single, 200000);
        add(
            &mut d,
            income(FederalIncome::TaxRefund {
                amount: 300000,
                withholding: 0,
                history: refund_history(),
            }),
        );
        if ira > 0 {
            d.adjustments.as_mut().unwrap().iras = ira_facts(ira * 100, 0, Answer::No);
        }
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 201000);
        assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), 200000);
        assert_eq!(form_1116_any(&r, true, "_worldwide_income"), amt_income);
        assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), amt_income);
        assert_eq!(value(&r, "foreign_general_amt_unused_tax"), unused);
        assert_eq!(r.tax, Some(tax * 100));
        set_foreign_amt_election(&mut d, 2020, true);
        let simplified = supported(&d);
        assert_eq!(simplified.tax, Some(simplified_tax * 100));
        assert_eq!(supported(&restart(d).draft), simplified);
    }
}

#[test]
fn foreign_carryovers_use_current_tax_then_oldest_balances_separately() {
    let mut d = foreign_high_tax_return(1000);
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[0].tax = 500000;
        sources[1].category = ForeignTaxCategory::Passive {
            sales: vec![],
            withholding: ForeignWithholding::Below15,
            qualified: 0,
        };
    }
    let balances = vec![
        foreign_balance(2017, ForeignCreditCategory::General, 10000, 6000),
        foreign_balance(2020, ForeignCreditCategory::Passive, 3000, 4000),
        foreign_balance(2016, ForeignCreditCategory::General, 20000, 5000),
    ];
    set_foreign_history(&mut d, balances.clone());
    let r = supported(&d);
    // Regular general cap11,020 minus current5,000 uses6,020 oldest taxes.
    // AMT general cap8,572 minus current5,000 uses3,572 independently.
    // Passive carry uses3,000 regular and4,000 AMT after current1,000.
    assert_eq!(value(&r, "foreign_general_regular_2016_carry_used"), 6020);
    assert_eq!(value(&r, "foreign_general_amt_2016_carry_used"), 3572);
    assert_eq!(value(&r, "foreign_general_regular_carry_expired"), 13980);
    assert_eq!(value(&r, "foreign_general_amt_carry_expired"), 1428);
    assert_eq!(value(&r, "foreign_general_regular_retained_carryforward"), 10000);
    assert_eq!(value(&r, "foreign_general_amt_retained_carryforward"), 6000);
    assert_eq!(value(&r, "foreign_tax_credit"), 15020);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(r.tax, Some(2171400));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let mut reordered = balances;
    reordered.reverse();
    set_foreign_history(&mut d, reordered);
    assert_eq!(supported(&d), r);
    if let ForeignTaxClaim::Ordinary { history: ForeignCreditHistory::Retained(records), .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        records.schedules[0]
            .columns
            .iter_mut()
            .find(|b| b.year == 2016)
            .unwrap()
            .regular
            .carryover = 500000;
    }
    let exhausted = supported(&d);
    assert_eq!(value(&exhausted, "foreign_general_regular_2016_carry_used"), 5000);
    assert_eq!(value(&exhausted, "foreign_general_regular_2017_carry_used"), 1020);
    assert_eq!(value(&exhausted, "foreign_general_regular_retained_carryforward"), 8980);
    assert_eq!(value(&exhausted, "foreign_general_regular_carry_expired"), 0);
    assert_eq!(exhausted.tax, r.tax);
}

#[test]
fn foreign_carryovers_keep_original_category_and_do_not_replace_current_taxes() {
    let mut d = foreign_high_tax_return(15000);
    set_foreign_history(
        &mut d,
        vec![
            foreign_balance(2016, ForeignCreditCategory::General, 5000, 5000),
            foreign_balance(2017, ForeignCreditCategory::Passive, 1000, 1000),
        ],
    );
    let r = supported(&d);
    // Current high-tax passive income moves to general; old passive balances do not.
    assert_eq!(value(&r, "foreign_general_regular_carry_used"), 3367);
    assert_eq!(value(&r, "foreign_general_regular_carry_expired"), 1633);
    assert_eq!(value(&r, "foreign_general_amt_carry_used"), 0);
    assert_eq!(value(&r, "foreign_general_amt_carry_expired"), 5000);
    assert_eq!(value(&r, "foreign_passive_regular_retained_carryforward"), 1000);
    assert_eq!(value(&r, "foreign_passive_amt_retained_carryforward"), 1000);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 713);
    assert_eq!(value(&r, "foreign_tax_credit"), 18367);
    assert_eq!(r.tax, Some(1836700));
}

#[test]
fn foreign_carryover_only_return_expires_tenth_year_without_foreign_income() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    d.credits.as_mut().unwrap().foreign_tax = survey_ordinary_foreign_credit(vec![]);
    set_foreign_history(
        &mut d,
        vec![
            foreign_balance(2016, ForeignCreditCategory::General, 1000, 700),
            foreign_balance(2017, ForeignCreditCategory::General, 2000, 900),
        ],
    );
    let r = supported(&d);
    assert_eq!(r.tax, baseline.tax);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert_eq!(value(&r, "foreign_general_regular_carry_expired"), 1000);
    assert_eq!(value(&r, "foreign_general_amt_carry_expired"), 700);
    assert_eq!(value(&r, "foreign_general_regular_2016_carry_remaining"), 0);
    assert_eq!(value(&r, "foreign_general_regular_retained_carryforward"), 2000);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    if let ForeignTaxClaim::Ordinary { history: ForeignCreditHistory::Retained(records), .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        records.same_filing_unit = false;
        let first = records.schedules[0].columns[0].clone();
        records.schedules[0].columns.push(first);
    }
    assert!(!d.valid_stored());
    set_foreign_history(
        &mut d,
        vec![foreign_balance(2015, ForeignCreditCategory::General, 1000, 700)],
    );
    assert!(!d.valid_stored());
    set_foreign_history(
        &mut d,
        vec![foreign_balance(2025, ForeignCreditCategory::General, 1000, 700)],
    );
    if let ForeignTaxClaim::Ordinary { history: ForeignCreditHistory::Retained(records), .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        records.same_filing_unit = false;
    }
    let r = estimate(&d);
    assert!(noted(&r, "WARN_FOREIGN_CREDIT_CARRYOVER_LEFT_OUT"), "{r:?}");
}

#[test]
fn foreign_retained_credit_balances_are_reduced_last_by_excluded_debt() {
    for (regular, amt) in [(1000, 0), (0, 1000)] {
        let mut d = complete(FederalStatus::Single);
        d.credits.as_mut().unwrap().foreign_tax = survey_ordinary_foreign_credit(vec![]);
        set_foreign_history(
            &mut d,
            vec![foreign_balance(2017, ForeignCreditCategory::General, regular, amt)],
        );
        supported(&d);
        add(&mut d, insolvency_debt(1500, 7500, 4600, 6100));
        // $1,500 excluded (insolvent by $2,900). Basis first takes the $100
        // room ($6,100 basis and cash after less $6,000 liabilities after);
        // the other $1,400 reaches Form 982 line 13, where the $1,000 2017
        // carryover of that tax system can absorb up to $3,000: the credit
        // falls by $1,400 / 3 = $466.67, leaving $533.33.
        let r = supported(&d);
        let prefix = if regular > 0 { "" } else { "amt_" };
        assert_eq!(value(&r, &alloc::format!("debt_2_{prefix}basis_reduction")), 100);
        let reduction = r
            .lines
            .iter()
            .find(|l| l.id == alloc::format!("debt_2_{prefix}ftc_reduction"))
            .unwrap();
        assert_eq!(reduction.amount, 46667);
        let after =
            r.lines.iter().find(|l| l.id == alloc::format!("debt_2_{prefix}ftc_after")).unwrap();
        assert_eq!(after.amount, 53333);
        assert_eq!(r.lines.iter().any(|l| l.id == "debt_2_form982_line13"), regular > 0);
        // No excluded debt means no attribute reduction; history remains usable.
        d.income.pop();
        add(&mut d, insolvency_debt(1500, 7500, 7600, 6100));
        supported(&d);
    }
}

#[test]
fn foreign_carryback_allocates_separate_prior_limits_without_claiming_refund() {
    let mut d = wholly_foreign_credit_return(FederalStatus::Single, 200000);
    let before = supported(&d);
    // Current regular credit36,734 and AMT credit28,574 from taxes400,000.
    // Retained 2025 regular capacity10,000-7,000=3,000;
    // AMT capacity8,000-3,000=5,000. These are not refund amounts.
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::PriorReturn {
            categories: vec![prior_foreign_credit(
                ForeignCreditCategory::General,
                [10000, 7000],
                [8000, 3000],
            )],
            confirmed: true,
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_general_regular_carryback_prior_year"), 3000);
    assert_eq!(value(&r, "foreign_general_amt_carryback_prior_year"), 5000);
    assert_eq!(value(&r, "foreign_general_regular_new_carryforward"), 360266);
    assert_eq!(value(&r, "foreign_general_amt_new_carryforward"), 366426);
    assert_eq!(r.tax, before.tax);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::PriorReturn {
            categories: vec![prior_foreign_credit(
                ForeignCreditCategory::General,
                [500000, 0],
                [500000, 0],
            )],
            confirmed: true,
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_general_regular_carryback_prior_year"), 363266);
    assert_eq!(value(&r, "foreign_general_amt_carryback_prior_year"), 371426);
    assert_eq!(value(&r, "foreign_general_regular_new_carryforward"), 0);
    assert_eq!(value(&r, "foreign_general_amt_new_carryforward"), 0);
}

#[test]
fn cash_method_tax_on_prior_foreign_wages_creates_carry_without_current_foreign_pay() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Ordinary {
        sources: vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Synthetic Country A 2025 employee tax paid in 2026".into(),
            category: ForeignTaxCategory::PriorEmployeeWageTax { income_year: 2025 },
            income: 0,
            tax: 20_000,
        }],
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::PriorReturn { categories: vec![], confirmed: true },
        amt_limitation: ForeignAmtLimitation::Normal,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    let r = supported(&d);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), value(&baseline, "agi"));
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert_eq!(value(&r, "foreign_general_regular_new_carryforward"), 200);
    assert_eq!(value(&r, "foreign_general_amt_new_carryforward"), 200);
    assert_eq!(r.tax, baseline.tax);
    assert_eq!(supported(&restart(d.clone()).draft), r);

    set_foreign_carryback(
        &mut d,
        ForeignCarryback::PriorReturn {
            categories: vec![prior_foreign_credit(
                ForeignCreditCategory::General,
                [100, 50],
                [100, 50],
            )],
            confirmed: true,
        },
    );
    let carried = supported(&d);
    assert_eq!(value(&carried, "foreign_general_regular_carryback_prior_year"), 50);
    assert_eq!(value(&carried, "foreign_general_amt_carryback_prior_year"), 50);
    assert_eq!(value(&carried, "foreign_general_regular_new_carryforward"), 150);
    assert_eq!(value(&carried, "foreign_general_amt_new_carryforward"), 150);
    assert_eq!(carried.tax, baseline.tax);

    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[0].category = ForeignTaxCategory::PriorEmployeeWageTax { income_year: 2025 };
    }
    // An earlier year's tax counts only when taxes are claimed when paid
    // (the rule and its server twin).
    let claim = d.credits.as_ref().unwrap().foreign_tax.clone();
    assert_eq!(crate::form_1116::claim_rule(&claim, &[]).map(|issue| issue.key), None);
    let mut accrued = claim;
    if let ForeignTaxClaim::Ordinary { basis, .. } = &mut accrued {
        *basis = ForeignTaxBasis::Accrued;
    }
    assert_eq!(
        crate::form_1116::claim_rule(&accrued, &[]).map(|issue| issue.key),
        Some("FOREIGN_PRIOR_TAX_NEEDS_PAID_BASIS")
    );
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[0].category = ForeignTaxCategory::PriorEmployeeWageTax { income_year: 2026 };
    }
    assert!(!d.valid_stored());
}

#[test]
fn prior_passive_tax_with_zero_current_foreign_income_moves_to_general_carry() {
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Ordinary {
        sources: vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Synthetic Country B 2024 ordinary interest tax paid in 2026".into(),
            category: ForeignTaxCategory::PriorPassiveTax { income_year: 2024 },
            income: 0,
            tax: 20_000,
        }],
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::PriorReturn { categories: vec![], confirmed: true },
        amt_limitation: ForeignAmtLimitation::Normal,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert_eq!(value(&r, "foreign_high_tax_taxes"), 200);
    assert_eq!(value(&r, "foreign_general_regular_new_carryforward"), 200);
    assert_eq!(value(&r, "foreign_general_amt_new_carryforward"), 200);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 0);
    assert_eq!(r.tax, baseline.tax);
    assert_eq!(supported(&restart(d.clone()).draft), r);

    add(&mut d, income(savings_income(100, 0)));
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources.push(ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Synthetic Country B current untaxed interest".into(),
            category: ForeignTaxCategory::Passive {
                sales: vec![],
                withholding: ForeignWithholding::None,
                qualified: 0,
            },
            income: 10_000,
            tax: 0,
        });
    }
    let current_untaxed = supported(&d);
    assert_eq!(value(&current_untaxed, "foreign_high_tax_taxes"), 200);
    assert_eq!(value(&current_untaxed, "foreign_general_regular_new_carryforward"), 200);
    assert_eq!(value(&current_untaxed, "foreign_passive_unused_tax"), 0);
    assert_eq!(supported(&restart(d.clone()).draft), current_untaxed);

    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[1].tax = 1_000;
    }
    unsupported(&d, "Prior passive foreign tax");
}

#[test]
fn prior_passive_tax_uses_general_category_alongside_separate_current_withholding_groups() {
    for (withholding, current_tax) in
        [(ForeignWithholding::AtLeast15, 15_000), (ForeignWithholding::Below15, 10_000)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(savings_income(1_000, 0)));
        d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Ordinary {
            sources: vec![
                ForeignOrdinaryTax {
                    country: "ZZ".into(),
                    owner: Owner::Taxpayer,
                    label: "Synthetic current foreign interest".into(),
                    category: ForeignTaxCategory::Passive {
                        withholding,
                        qualified: 0,
                        sales: vec![],
                    },
                    income: 100_000,
                    tax: current_tax,
                },
                ForeignOrdinaryTax {
                    country: "ZZ".into(),
                    owner: Owner::Taxpayer,
                    label: "Synthetic prior ordinary dividend tax paid now".into(),
                    category: ForeignTaxCategory::PriorPassiveTax { income_year: 2025 },
                    income: 0,
                    tax: 20_000,
                },
            ],
            history: ForeignCreditHistory::None,
            carryback: ForeignCarryback::PriorReturn { categories: vec![], confirmed: true },
            amt_limitation: ForeignAmtLimitation::Normal,
            interest_allocation: ForeignInterestAllocation::GrossIncome,
            investment_assets: ForeignInvestmentAssets::Unreviewed,
            loss_history: ForeignLossHistory::None,
            state_taxes: ForeignStateTaxes::Unreviewed,
            basis: ForeignTaxBasis::Paid,
            situations: vec![],
            rate_adjustment: ForeignRateAdjustment::Exception,
        };
        let with_prior = supported(&d);
        assert_eq!(value(&with_prior, "foreign_high_tax_taxes"), 200);
        assert_eq!(value(&with_prior, "foreign_general_regular_new_carryforward"), 200);
        assert_eq!(supported(&restart(d.clone()).draft), with_prior);

        if let ForeignTaxClaim::Ordinary { sources, .. } =
            &mut d.credits.as_mut().unwrap().foreign_tax
        {
            sources.pop();
        }
        let without_prior = supported(&d);
        assert_eq!(
            value(&with_prior, "foreign_tax_credit"),
            value(&without_prior, "foreign_tax_credit")
        );
        assert_eq!(with_prior.tax, without_prior.tax);
        assert_eq!(value(&without_prior, "foreign_high_tax_taxes"), 0);
    }
}

#[test]
fn prior_foreign_tax_paid_now_uses_current_general_category_capacity() {
    for prior_category in [
        ForeignTaxCategory::PriorEmployeeWageTax { income_year: 2025 },
        ForeignTaxCategory::PriorPassiveTax { income_year: 2025 },
    ] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(foreign_pay(10_000)));
        d.credits.as_mut().unwrap().foreign_tax = survey_ordinary_foreign_credit(vec![
            ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Synthetic current foreign employee wages".into(),
                category: ForeignTaxCategory::Employment,
                income: 1_000_000,
                tax: 0,
            },
            ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Synthetic prior income tax paid now".into(),
                category: prior_category,
                income: 0,
                tax: 20_000,
            },
        ]);
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_tax_credit"), 200);
        assert_eq!(value(&r, "foreign_general_unused_tax"), 0);
        // No AMT liability creates a separate $200 AMT excess even though the
        // regular Form 1116 uses the entire payment in 2026.
        assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 200);
        assert_eq!(supported(&restart(d.clone()).draft), r);
    }
}

#[test]
fn foreign_carryback_small_election_and_unreviewed_history_are_distinct() {
    let mut d = wholly_foreign_credit_return(FederalStatus::Single, 200000);
    let pending = supported(&d);
    // The unfinished review leaves the whole excess as a provisional
    // carryforward (with its note).
    assert_eq!(value(&pending, "foreign_general_regular_new_carryforward"), 363266);
    assert!(pending.review_notes.iter().any(|n| n.contains("review the 2025 carryback")));
    set_foreign_carryback(&mut d, ForeignCarryback::SmallCreditElection);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_general_regular_carryback_prior_year"), 0);
    assert_eq!(value(&r, "foreign_general_regular_new_carryforward"), 363266);
    assert_eq!(value(&r, "foreign_general_amt_new_carryforward"), 371426);
    assert_eq!(r.tax, pending.tax);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_carryback_rejects_duplicate_unconfirmed_and_inconsistent_history() {
    let mut d = wholly_foreign_credit_return(FederalStatus::Single, 200000);
    let row = prior_foreign_credit(ForeignCreditCategory::General, [10000, 7000], [8000, 3000]);
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::PriorReturn {
            categories: vec![row.clone(), row.clone()],
            confirmed: true,
        },
    );
    assert!(!crate::extended_rules::valid_credit(&d.credits.as_ref().unwrap().foreign_tax));
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::PriorReturn { categories: vec![row.clone()], confirmed: false },
    );
    // Unconfirmed worksheets: the carryback stays unreviewed.
    supported(&d);
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::PriorReturn { categories: vec![row], confirmed: true },
    );
    for balances in [
        vec![foreign_balance(2020, ForeignCreditCategory::General, 1, 0)],
        vec![foreign_balance(2020, ForeignCreditCategory::General, 0, 1)],
    ] {
        set_foreign_history(&mut d, balances);
        unsupported(&d, "Foreign-credit carryback history");
    }
    // Capacity is exhausted by older taxes: no false negative bound or carryback.
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::PriorReturn {
            categories: vec![prior_foreign_credit(
                ForeignCreditCategory::General,
                [5000, 7000],
                [2000, 3000],
            )],
            confirmed: true,
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_general_regular_carryback_prior_year"), 0);
    assert_eq!(value(&r, "foreign_general_amt_carryback_prior_year"), 0);
    assert_eq!(value(&r, "foreign_general_amt_retained_carryforward"), 1);
}

#[test]
fn foreign_carryback_keeps_current_category_and_zero_capacity_separate() {
    let mut d = foreign_high_tax_return(12000);
    let before = supported(&d);
    let categories = vec![
        prior_foreign_credit(ForeignCreditCategory::General, [100000, 0], [100000, 0]),
        prior_foreign_credit(ForeignCreditCategory::Passive, [3000, 1000], [5000, 0]),
    ];
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::PriorReturn { categories: categories.clone(), confirmed: true },
    );
    let r = supported(&d);
    // Regular passive excess4,653 cannot use general capacity. AMT high-tax
    // kickout places current taxes in general, where none exceed the2026limit.
    assert_eq!(value(&r, "foreign_passive_regular_carryback_prior_year"), 2000);
    assert_eq!(value(&r, "foreign_passive_regular_new_carryforward"), 2653);
    assert_eq!(value(&r, "foreign_general_regular_carryback_prior_year"), 0);
    assert_eq!(value(&r, "foreign_passive_amt_carryback_prior_year"), 0);
    assert_eq!(value(&r, "foreign_general_amt_carryback_prior_year"), 0);
    assert_eq!(r.tax, before.tax);
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::PriorReturn {
            categories: categories.into_iter().rev().collect(),
            confirmed: true,
        },
    );
    assert_eq!(supported(&d), r);
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::PriorReturn { categories: vec![], confirmed: true },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_passive_regular_new_carryforward"), 4653);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn ordinary_foreign_credit_allocates_itemized_medical_and_personal_taxes() {
    let mut d = wholly_foreign_credit_return(FederalStatus::Single, 200000);
    foreign_medical_itemizing(&mut d, 35000, 10000, 20000);
    let r = supported(&d);
    // Medical35,000 minus15,000 AGIfloor=20,000; SALT30,000.
    // Regular taxable150,000, tax28,598; AMTI180,000, tax23,374.
    // Wholly foreign income uses all both limits independently.
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 150000);
    assert_eq!(form_1116_any(&r, true, "_worldwide_income"), 180000);
    assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), 200000);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 180000);
    assert_eq!(value(&r, "foreign_tax_credit"), 28598);
    assert_eq!(value(&r, "foreign_general_unused_tax"), 371402);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 376626);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    // Medical below its floor is not allocated as an expense.
    foreign_medical_itemizing(&mut d, 14000, 10000, 20000);
    let r = supported(&d);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 170000);
    assert_eq!(form_1116_any(&r, true, "_worldwide_income"), 200000);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 200000);
}

#[test]
fn ordinary_foreign_itemizing_changes_high_tax_grouping_and_coordinates_ira() {
    let mut d = foreign_high_tax_return(12000);
    foreign_medical_itemizing(&mut d, 35000, 10000, 20000);
    let r = supported(&d);
    // Deduction50,000 / gross200,000: passive net30,000, general45,000.
    // Passive tax12,000 now exceeds37% and moves to general, with net75,000.
    // AMT only deducts20,000 medical: combined foreign net90,000,
    // worldwide180,000 and tentative23,374 produce11,687 AMT credit.
    assert_eq!(value(&r, "foreign_high_tax_net_income"), 30000);
    assert_eq!(value(&r, "foreign_amt_high_tax_net_income"), 36000);
    assert_eq!(value(&r, "foreign_tax_credit"), 12000);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 313);
    assert_eq!(r.tax, Some(1659800));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::No);
    let r = supported(&d);
    // AGI192,500; medical floor14,438, deduction20,562; IRA7,500.
    // Regular base141,938 =>26,663; AMT base171,938 =>21,278.
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 141938);
    assert_eq!(form_1116_any(&r, true, "_worldwide_income"), 171938);
    assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), 200000);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 85969);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 1361);
    assert_eq!(r.tax, Some(1466300));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn ordinary_foreign_itemizing_preserves_salt_and_section68_limits() {
    let mut d = wholly_foreign_credit_return(FederalStatus::Single, 1000000);
    foreign_medical_itemizing(&mut d, 85000, 50000, 20000);
    let r = supported(&d);
    // SALT phases down to10,000; medical10,000; section68 reduces20,000
    // by2/37 rounded=1,081. AMT ignores SALT and section68 reduction.
    assert_eq!(value(&r, "itemized_deduction_reduction"), 1081);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 981081);
    assert_eq!(form_1116_any(&r, true, "_worldwide_income"), 990000);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 990000);
    assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), 1000000);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn ordinary_foreign_itemizing_keeps_income_tax_allocation_explicit() {
    let mut d = wholly_foreign_credit_return(FederalStatus::Single, 200000);
    foreign_medical_itemizing(&mut d, 35000, 10000, 20000);
    itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
        paid: 1000000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    unsupported(&d, "Foreign state tax allocation");
}

#[test]
fn ordinary_foreign_automatic_itemizing_can_reduce_simplified_amt_credit_tax() {
    let mut d = wholly_foreign_credit_return(FederalStatus::Single, 200000);
    set_foreign_amt_election(&mut d, 2020, true);
    let standard = supported(&d);
    assert_eq!(standard.tax, Some(230000));
    d.deductions.as_mut().unwrap().choice =
        DeductionSelection::Automatic { expenses: ItemizedChoice::No };
    let automatic = supported(&d);
    // Itemizing zero expenses gives up the standard deduction but leaves the
    // entire foreign numerator for the binding simplified AMT election.
    assert_eq!(form_1116_any(&automatic, false, "_worldwide_income"), 200000);
    assert_eq!(form_1116_value(&automatic, "foreign_amt_general_net_foreign_income"), 200000);
    assert_eq!(automatic.tax, Some(0));
    assert_eq!(supported(&restart(d.clone()).draft), automatic);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    assert_eq!(supported(&d), standard);
}

#[test]
fn ordinary_foreign_itemized_deductions_above_agi_leave_line_18_zero() {
    // Itemized deductions above AGI: line 18 is zero, so line 19 is zero, no
    // credit is allowed and every tax is unused (Schedule B line 6).
    let mut d = wholly_foreign_credit_return(FederalStatus::Single, 200000);
    foreign_medical_itemizing(&mut d, 250000, 10000, 20000);
    let r = supported(&d);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 0);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert!(value(&r, "foreign_general_unused_tax") > 0);
}

#[test]
fn ordinary_foreign_credit_separates_charity_from_mortgage_expenses() {
    let mut d = foreign_high_tax_return(12000);
    foreign_medical_itemizing(&mut d, 35000, 10000, 20000);
    foreign_mortgage(&mut d, 10000);
    let ded = d.deductions.as_mut().unwrap();
    ded.set_cash_charity(600000);
    let r = supported(&d);
    // Medical20k+SALT30k+mortgage10k+gift(6k-1kfloor)=65k.
    // Gift is domestic; foreign expense ratio only uses60k regular/30k AMT.
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 135000);
    assert_eq!(form_1116_any(&r, true, "_worldwide_income"), 165000);
    assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), 200000);
    assert_eq!(value(&r, "foreign_regular_domestic_deductions"), 5000);
    assert_eq!(value(&r, "foreign_amt_domestic_deductions"), 5000);
    assert_eq!(value(&r, "foreign_high_tax_net_income"), 28000);
    assert_eq!(value(&r, "foreign_amt_high_tax_net_income"), 34000);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 85000);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 1968);
    assert_eq!(value(&r, "foreign_tax_credit"), 12000);
    assert_eq!(r.tax, Some(1299800));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn ordinary_foreign_credit_small_interest_election_has_combined_exact_threshold() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 19500000, 0);
    add(&mut d, income(foreign_pay(5000)));
    foreign_medical_itemizing(&mut d, 0, 0, 0);
    foreign_mortgage(&mut d, 20000);
    d.credits.as_mut().unwrap().foreign_tax =
        survey_ordinary_foreign_credit(vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign wages".into(),
            category: ForeignTaxCategory::Employment,
            income: 500000,
            tax: 200000,
        }]);
    let gross = supported(&d);
    assert_eq!(value(&gross, "foreign_tax_credit"), 895);
    assert_eq!(form_1116_value(&gross, "foreign_amt_general_net_foreign_income"), 4500);
    assert_eq!(gross.tax, Some(3490300));
    if let ForeignTaxClaim::Ordinary { interest_allocation, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *interest_allocation = ForeignInterestAllocation::SmallForeignIncome;
    }
    let small = supported(&d);
    assert_eq!(value(&small, "foreign_regular_domestic_deductions"), 20000);
    assert_eq!(value(&small, "foreign_tax_credit"), 994);
    assert_eq!(form_1116_value(&small, "foreign_amt_general_net_foreign_income"), 5000);
    assert_eq!(value(&small, "foreign_general_amt_unused_tax"), 1351);
    assert_eq!(small.tax, Some(3480400));
    assert_eq!(supported(&restart(d.clone()).draft), small);
    // Separate source rows do not each get the5,000 exception; even one cent
    // beyond the combined factual boundary is ineligible.
    add(&mut d, income(savings_income(1, 0)));
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources.push(ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign interest".into(),
            category: ForeignTaxCategory::Passive {
                sales: vec![],
                withholding: ForeignWithholding::None,
                qualified: 0,
            },
            income: 1,
            tax: 0,
        });
    }
    unsupported(&d, "Foreign interest allocation");
}

#[test]
fn ordinary_foreign_credit_standard_cash_gifts_are_domestic() {
    let mut d = foreign_high_tax_return(12000);
    let ded = d.deductions.as_mut().unwrap();
    ded.choice = DeductionSelection::Standard;
    ded.set_cash_charity(600000);
    let r = supported(&d);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 182900);
    assert_eq!(form_1116_any(&r, true, "_worldwide_income"), 199000);
    assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), 200000);
    assert_eq!(value(&r, "foreign_regular_domestic_deductions"), 1000);
    assert_eq!(value(&r, "foreign_amt_domestic_deductions"), 1000);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 100000);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn ordinary_foreign_charity_creates_domestic_loss_accounts() {
    let mut d = wholly_foreign_credit_return(FederalStatus::Single, 200000);
    d.deductions.as_mut().unwrap().set_cash_charity(600000);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_regular_general_overall_domestic_loss_new"), 1000);
    assert_eq!(value(&r, "foreign_amt_general_overall_domestic_loss_new"), 1000);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 199000);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn ordinary_foreign_charity_and_mortgage_share_section68_reduction() {
    let mut d = wholly_foreign_credit_return(FederalStatus::Single, 500000);
    add(&mut d, wage(3, Owner::Taxpayer, 50000000, 0));
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[0].tax = 20000000;
    }
    foreign_medical_itemizing(&mut d, 85000, 50000, 20000);
    foreign_mortgage(&mut d, 20000);
    let ded = d.deductions.as_mut().unwrap();
    ded.set_cash_charity(1500000);
    let r = supported(&d);
    // Medical10k+SALT10k+mortgage20k+gift10k=50k; section68=2,703.
    // Charity reduction541 leaves9,459 allocated only to domestic income.
    assert_eq!(value(&r, "itemized_deduction_reduction"), 2703);
    assert_eq!(value(&r, "foreign_regular_domestic_deductions"), 9459);
    assert_eq!(value(&r, "foreign_amt_domestic_deductions"), 10000);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 952703);
    assert_eq!(form_1116_any(&r, true, "_worldwide_income"), 960000);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 485000);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 66670);
    assert_eq!(value(&r, "foreign_tax_credit"), 155760);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_state_tax_uses_state_source_income_not_federal_gross_ratio() {
    let d = foreign_state_return();
    let r = supported(&d);
    // Pub514 source example:100k federal income includes25k foreign, but
    // state taxable90k includes only15k foreign. State tax6k: foreign1k.
    // Regular income94k, tax15,392; foreign24k gives FTC3,930.
    assert_eq!(value(&r, "foreign_state_income_tax_group_0"), 1000);
    assert_eq!(value(&r, "foreign_state_income_tax_group_7"), 5000);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 94000);
    assert_eq!(value(&r, "foreign_tax_credit"), 3931);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 25000);
    assert_eq!(value(&r, "foreign_amt_domestic_deductions"), 0);
    assert_eq!(r.tax, Some(1146700));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_state_tax_keeps_exempt_jurisdiction_and_row_order_independent() {
    let mut d = foreign_state_return();
    let payments = vec![
        foreign_state_payment("State A", 3000, 75000, 15000),
        foreign_state_payment("State B exempts foreign income", 3000, 100000, 0),
    ];
    set_foreign_state(&mut d, payments.clone());
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_state_income_tax_group_0"), 500);
    assert_eq!(value(&r, "foreign_state_income_tax_group_7"), 5500);
    assert_eq!(value(&r, "foreign_tax_credit"), 4013);
    assert_eq!(r.tax, Some(1138500));
    set_foreign_state(&mut d, payments.into_iter().rev().collect());
    assert_eq!(supported(&d), r);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_state_tax_applies_salt_limit_before_source_allocation() {
    let mut d = foreign_state_return();
    let facts = itemized(d.deductions.as_mut().unwrap());
    facts.personal_tax = PersonalTaxElection::Income {
        paid: 6000000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    facts.personal_property_taxes = 4000000;
    set_foreign_state(&mut d, vec![foreign_state_payment("State A", 60000, 75000, 15000)]);
    let r = supported(&d);
    // SALT100k capped40,400; income tax24,240, property16,160.
    // State foreign share4,040; property foreign share4,040, net16,920.
    assert_eq!(value(&r, "foreign_state_income_tax_group_0"), 4040);
    assert_eq!(value(&r, "foreign_state_income_tax_group_7"), 20200);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 59600);
    assert_eq!(value(&r, "foreign_tax_credit"), 2223);
    assert_eq!(r.tax, Some(560700));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_state_tax_requires_complete_sources_and_does_not_hide_losses() {
    let mut d = foreign_state_return();
    set_foreign_state(&mut d, vec![foreign_state_payment("Mismatch", 5999, 75000, 15000)]);
    unsupported(&d, "Foreign state tax allocation");
    set_foreign_state(&mut d, vec![foreign_state_payment("No source base", 6000, 0, 0)]);
    unsupported(&d, "Foreign state tax allocation");
    let mut missing = foreign_state_payment("Missing source", 6000, 75000, 15000);
    missing.foreign[0].source = "Not an income source".into();
    set_foreign_state(&mut d, vec![missing]);
    unsupported(&d, "Foreign state tax allocation");
    // State may attribute prior source income differently, but a current foreign
    // deduction exceeding its income needs the loss worksheet, not a zero floor.
    let facts = itemized(d.deductions.as_mut().unwrap());
    facts.personal_tax = PersonalTaxElection::Income {
        paid: 4000000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    set_foreign_state(
        &mut d,
        vec![foreign_state_payment("Foreign-only state base", 40000, 0, 15000)],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_regular_general_overall_foreign_loss_new"), 15000);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 25000);
    assert_eq!(r.tax, Some(791800));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_small_income_allocates_separate_investment_deductions_domestically() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 17500000, 0);
    add(&mut d, income(savings_income(20000, 0)));
    add(&mut d, income(foreign_pay(5000)));
    foreign_medical_itemizing(&mut d, 0, 0, 0);
    investment_borrowing(&mut d, 10000, 10000, 0);
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Ordinary {
        sources: vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign wages".into(),
            category: ForeignTaxCategory::Employment,
            income: 500000,
            tax: 200000,
        }],
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        interest_allocation: ForeignInterestAllocation::SmallForeignIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment: ForeignRateAdjustment::Exception,
    };
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 20000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 10000);
    assert_eq!(value(&r, "foreign_regular_domestic_deductions"), 20000);
    assert_eq!(value(&r, "foreign_amt_domestic_deductions"), 10000);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 180000);
    assert_eq!(form_1116_any(&r, true, "_worldwide_income"), 190000);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 5000);
    assert_eq!(value(&r, "foreign_tax_credit"), 994);
    assert_eq!(r.tax, Some(3480400));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    if let ForeignTaxClaim::Ordinary { interest_allocation, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *interest_allocation = ForeignInterestAllocation::GrossIncome;
    }
    // Without portfolio records the interest is taken as relating to foreign
    // passive income; with only employment income abroad it stays U.S.
    let r = supported(&d);
    assert!(!noted(&r, "WARN_INVESTMENT_INTEREST_SOURCE_ASSUMED"));
    if let ForeignTaxClaim::Ordinary { interest_allocation, sources, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *interest_allocation = ForeignInterestAllocation::SmallForeignIncome;
        sources.push(ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign savings".into(),
            category: ForeignTaxCategory::Passive {
                sales: vec![],
                withholding: ForeignWithholding::None,
                qualified: 0,
            },
            income: 1,
            tax: 0,
        });
    }
    unsupported(&d, "Foreign interest allocation");
}

#[test]
fn foreign_investment_assets_allocate_released_carryforward_using_current_separate_bases() {
    let mut d = foreign_asset_return();
    investment_borrowing(&mut d, 1000, 1000, 0);
    if let ForeignTaxClaim::Ordinary {
        investment_assets: ForeignInvestmentAssets::TaxBook { assets, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        for (asset, amount) in assets.iter_mut().zip([8000000, 2000000]) {
            if let InvestmentAssetBasis::Average { amt_beginning, amt_ending, .. } =
                &mut asset.basis
            {
                *amt_beginning = amount;
                *amt_ending = amount;
            }
        }
    }
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_investment_regular_group_4"), 1200);
    assert_eq!(value(&r, "foreign_investment_amt_group_4"), 200);
    assert_eq!(form_1116_any(&r, true, "_worldwide_income"), 99000);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 5800);
    assert_eq!(r.tax, Some(1567800));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_investment_asset_allocation_precedes_independent_high_tax_tests() {
    let mut d = foreign_asset_return();
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[0].tax = 170000;
    }
    let r = supported(&d);
    // 1,700 is below37% of4,800 but above26%; AMT moves the group.
    assert_eq!(value(&r, "foreign_high_tax_net_income"), 0);
    assert_eq!(value(&r, "foreign_amt_high_tax_net_income"), 4800);
    assert_eq!(value(&r, "foreign_tax_credit"), 797);
    assert_eq!(r.tax, Some(1548100));
}

#[test]
fn foreign_investment_assets_require_resolved_sources_positive_bases_and_no_unhandled_loss() {
    let original = foreign_asset_return();
    for case in 0..4 {
        let mut d = original.clone();
        if case == 0 {
            // No records: all the investment interest is taken as relating to
            // the foreign passive income, with a note.
            if let ForeignTaxClaim::Ordinary { investment_assets, .. } =
                &mut d.credits.as_mut().unwrap().foreign_tax
            {
                *investment_assets = ForeignInvestmentAssets::Unreviewed;
            }
        }
        if let ForeignTaxClaim::Ordinary {
            investment_assets: ForeignInvestmentAssets::TaxBook { assets },
            ..
        } = &mut d.credits.as_mut().unwrap().foreign_tax
        {
            match case {
                0 => {}
                1 => assets[1].source = InvestmentAssetSource::Foreign { source: "Unknown".into() },
                2 => {
                    for a in assets {
                        if let InvestmentAssetBasis::Average { beginning, ending, .. } =
                            &mut a.basis
                        {
                            *beginning = 0;
                            *ending = 0;
                        }
                    }
                }
                _ => {
                    for a in assets {
                        if let InvestmentAssetBasis::Average { amt_beginning, amt_ending, .. } =
                            &mut a.basis
                        {
                            *amt_beginning = 0;
                            *amt_ending = 0;
                        }
                    }
                }
            }
        }
        if case == 0 {
            let r = supported(&d);
            assert!(noted(&r, "WARN_INVESTMENT_INTEREST_SOURCE_ASSUMED"), "{r:?}");
        } else {
            unsupported(&d, "Foreign investment interest allocation");
        }
    }
    let mut d = original;
    investment_borrowing(&mut d, 10000, 0, 0);
    if let ForeignTaxClaim::Ordinary {
        investment_assets: ForeignInvestmentAssets::TaxBook { assets, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        assets.remove(0);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_regular_passive_overall_foreign_loss_new"), 4000);
    assert_eq!(value(&r, "foreign_amt_passive_overall_foreign_loss_new"), 4000);
    assert_eq!(r.tax, Some(1451800));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_mixed_asset_keeps_multiple_withholding_groups_and_row_order() {
    let mut d = foreign_asset_return();
    if let ForeignTaxClaim::Ordinary { sources, investment_assets, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[0].income = 400000;
        sources[0].tax = 40000;
        sources.push(ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Withheld source".into(),
            category: ForeignTaxCategory::Passive {
                sales: vec![],
                withholding: ForeignWithholding::AtLeast15,
                qualified: 0,
            },
            income: 200000,
            tax: 60000,
        });
        *investment_assets = ForeignInvestmentAssets::TaxBook {
            assets: vec![foreign_asset(
                "Mixed asset",
                InvestmentAssetSource::Mixed {
                    domestic: 400000,
                    foreign: vec![
                        InvestmentAssetYield { source: "Foreign deposits".into(), income: 400000 },
                        InvestmentAssetYield { source: "Withheld source".into(), income: 200000 },
                    ],
                },
                100000,
                100000,
            )],
        };
    }
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_investment_regular_group_1"), 400);
    assert_eq!(value(&r, "foreign_investment_regular_group_4"), 800);
    assert_eq!(value(&r, "foreign_investment_regular_group_7"), 800);
    // Withheld group:600 tax >37% of1,600 net. Other group remains passive.
    assert_eq!(value(&r, "foreign_high_tax_net_income"), 1600);
    assert_eq!(value(&r, "foreign_high_tax_taxes"), 600);
    if let ForeignTaxClaim::Ordinary {
        investment_assets: ForeignInvestmentAssets::TaxBook { assets, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().foreign_tax
        && let InvestmentAssetSource::Mixed { foreign, .. } = &mut assets[0].source
    {
        foreign.reverse();
    }
    assert_eq!(supported(&d), r);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_mixed_asset_rejects_missing_yield_and_unreconciled_income() {
    for (domestic, foreign) in [(0, 0), (400000, 600001), (400100, 600000)] {
        let mut d = foreign_asset_return();
        if let ForeignTaxClaim::Ordinary { investment_assets, .. } =
            &mut d.credits.as_mut().unwrap().foreign_tax
        {
            *investment_assets = ForeignInvestmentAssets::TaxBook {
                assets: vec![foreign_asset(
                    "Mixed asset",
                    InvestmentAssetSource::Mixed {
                        domestic,
                        foreign: vec![InvestmentAssetYield {
                            source: "Foreign deposits".into(),
                            income: foreign,
                        }],
                    },
                    100000,
                    100000,
                )],
            };
        }
        unsupported(&d, "Foreign investment interest allocation");
    }
}

#[test]
fn foreign_daily_asset_constant_history_agrees_with_endpoint_and_mixed_yield() {
    let mut d = foreign_asset_return();
    let original = supported(&d);
    if let ForeignTaxClaim::Ordinary { investment_assets, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *investment_assets = ForeignInvestmentAssets::TaxBook {
            assets: vec![ForeignInvestmentAsset {
                label: "Mixed asset".into(),
                source: InvestmentAssetSource::Mixed {
                    domestic: 400000,
                    foreign: vec![InvestmentAssetYield {
                        source: "Foreign deposits".into(),
                        income: 600000,
                    }],
                },
                basis: InvestmentAssetBasis::Daily {
                    changes: vec![
                        daily_basis(20260101, 100000, 100000),
                        daily_basis(20260701, 100000, 100000),
                    ],
                },
            }],
        };
    }
    assert_eq!(supported(&d), original);
    assert_eq!(supported(&restart(d).draft), original);
}

#[test]
fn foreign_passive_losses_reduce_other_groups_before_high_tax_classification() {
    let mut d = foreign_passive_loss_return();
    let r = supported(&d);
    // Net groups -300,200,400. Reallocate300 proportionately to leave
    // 0,100,200. The40 tax from the zero group moves to general too.
    assert_eq!(value(&r, "foreign_regular_passive_loss_reallocated"), 300);
    assert_eq!(value(&r, "foreign_amt_passive_loss_reallocated"), 300);
    assert_eq!(value(&r, "foreign_high_tax_net_income"), 200);
    assert_eq!(value(&r, "foreign_high_tax_taxes"), 365);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 200);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 100);
    assert_eq!(value(&r, "foreign_tax_credit"), 43);
    assert_eq!(r.tax, Some(1632300));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    if let ForeignTaxClaim::Ordinary {
        sources,
        investment_assets: ForeignInvestmentAssets::TaxBook { assets, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources.reverse();
        assets.reverse();
    }
    assert_eq!(supported(&d), r);
}

#[test]
fn foreign_passive_loss_reallocation_is_independent_for_amt() {
    let mut d = foreign_passive_loss_return();
    investment_borrowing(&mut d, 900, 700, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_regular_passive_loss_reallocated"), 300);
    assert_eq!(value(&r, "foreign_high_tax_net_income"), 200);
    // AMT interest900:506 high-withholding,394 other-tax. Positive
    // net groups794 and6 need no loss reallocation; both are high taxed.
    assert_eq!(value(&r, "foreign_amt_high_tax_net_income"), 800);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 800);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 200);
    assert_eq!(r.tax, Some(1632300));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_passive_losses_exactly_absorb_other_groups_without_category_loss() {
    let mut d = foreign_passive_loss_return();
    investment_borrowing(&mut d, 1900, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_high_tax_net_income"), 0);
    assert_eq!(value(&r, "foreign_high_tax_taxes"), 375);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert_eq!(r.tax, Some(1630000));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_passive_income_below_standard_deduction_has_no_loss_offset_account() {
    let mut d = foreign_asset_return();
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    d.income.clear();
    add(&mut d, income(savings_income(10000, 0)));
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    if let ForeignTaxClaim::Ordinary { sources, investment_assets, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[0].income = 1000000;
        sources[0].tax = 100000;
        *investment_assets = ForeignInvestmentAssets::Unreviewed;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_income"), 0);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 0);
    assert_eq!(value(&r, "foreign_regular_passive_loss_reallocated"), 0);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_current_passive_loss_offsets_general_before_domestic_income() {
    for wages in [5000, 2000] {
        let mut d = foreign_asset_return();
        investment_borrowing(&mut d, 10000, 0, 0);
        add(&mut d, income(foreign_pay(wages)));
        if let ForeignTaxClaim::Ordinary {
            sources,
            investment_assets: ForeignInvestmentAssets::TaxBook { assets, .. },
            ..
        } = &mut d.credits.as_mut().unwrap().foreign_tax
        {
            assets.remove(0);
            sources.push(ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Foreign employment".into(),
                category: ForeignTaxCategory::Employment,
                income: wages * 100,
                tax: 0,
            });
        }
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_regular_passive_separate_loss_new"), wages.min(4000));
        assert_eq!(value(&r, "foreign_amt_passive_separate_loss_new"), wages.min(4000));
        if wages == 5000 {
            assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 1000);
            assert_eq!(value(&r, "foreign_tax_credit"), 164);
            assert_eq!(r.tax, Some(1545400));
        } else {
            assert_eq!(value(&r, "foreign_regular_passive_overall_foreign_loss_new"), 2000);
            assert_eq!(value(&r, "foreign_amt_passive_overall_foreign_loss_new"), 2000);
            assert_eq!(value(&r, "foreign_tax_credit"), 0);
            assert_eq!(r.tax, Some(1495800));
        }
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn foreign_current_general_loss_offsets_passive_income_and_preserves_amt_sources() {
    let mut d = foreign_state_return();
    add(&mut d, income(savings_income(20000, 0)));
    itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
        paid: 4000000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    set_foreign_state(
        &mut d,
        vec![foreign_state_payment("Foreign-only state base", 40000, 0, 15000)],
    );
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources.push(ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign portfolio".into(),
            category: ForeignTaxCategory::Passive {
                sales: vec![],
                withholding: ForeignWithholding::None,
                qualified: 0,
            },
            income: 2000000,
            tax: 200000,
        });
    }
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_regular_general_separate_loss_new"), 15000);
    assert_eq!(value(&r, "foreign_tax_credit"), 770);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 25000);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 20000);
    assert_eq!(r.tax, Some(1154800));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    set_foreign_amt_election(&mut d, 2020, true);
    let elected = supported(&d);
    // Regular SLL reduces passive20,000 to5,000; the elected AMT
    // numerator inherits that adjustment although AMT creates no loss.
    assert_eq!(form_1116_value(&elected, "foreign_amt_general_net_foreign_income"), 0);
    assert_eq!(form_1116_value(&elected, "foreign_amt_passive_net_foreign_income"), 5000);
    assert_eq!(elected.tax, Some(1154800));
    assert_eq!(supported(&restart(d).draft), elected);
}

#[test]
fn foreign_current_domestic_loss_is_proportional_between_categories() {
    let mut d = foreign_asset_return();
    d.income.clear();
    add(&mut d, income(foreign_pay(60000)));
    add(&mut d, income(savings_income(40000, 0)));
    foreign_medical_itemizing(&mut d, 0, 0, 0);
    d.deductions.as_mut().unwrap().investment_interest = None;
    d.deductions.as_mut().unwrap().set_cash_charity(600000);
    if let ForeignTaxClaim::Ordinary { sources, investment_assets, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *sources = vec![
            ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Foreign employment".into(),
                category: ForeignTaxCategory::Employment,
                income: 6000000,
                tax: 600000,
            },
            ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Foreign deposits".into(),
                category: ForeignTaxCategory::Passive {
                    sales: vec![],
                    withholding: ForeignWithholding::None,
                    qualified: 0,
                },
                income: 4000000,
                tax: 400000,
            },
        ];
        *investment_assets = ForeignInvestmentAssets::Unreviewed;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_regular_general_overall_domestic_loss_new"), 3300);
    assert_eq!(value(&r, "foreign_regular_passive_overall_domestic_loss_new"), 2200);
    assert_eq!(value(&r, "foreign_amt_general_overall_domestic_loss_new"), 3300);
    assert_eq!(value(&r, "foreign_amt_passive_overall_domestic_loss_new"), 2200);
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 56700);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 37800);
    assert_eq!(r.tax, Some(550800));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    if let ForeignTaxClaim::Ordinary { amt_limitation, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *amt_limitation =
            ForeignAmtLimitation::Simplified { first_claim_year: 2026, confirmed: true };
    }
    unsupported(&d, "Simplified foreign loss accounts");
}

#[test]
fn foreign_loss_history_observes_recapture_order_and_excludes_ofl_income_from_odl_cap() {
    let mut d = foreign_high_tax_return(12000);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    foreign_loss_records(
        &mut d,
        vec![
            ForeignLossAccount {
                category: ForeignCreditCategory::General,
                regular: foreign_loss_system(20000, 5000, 0, 0),
                amt: foreign_loss_system(0, 0, 0, 0),
            },
            ForeignLossAccount {
                category: ForeignCreditCategory::Passive,
                regular: foreign_loss_system(10000, 0, 100000, 0),
                amt: foreign_loss_system(0, 0, 0, 0),
            },
        ],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_regular_general_overall_foreign_loss_recaptured"), 20000);
    assert_eq!(value(&r, "foreign_regular_passive_overall_foreign_loss_recaptured"), 10000);
    assert_eq!(value(&r, "foreign_regular_general_separate_loss_recaptured"), 5000);
    // 50% of original U.S. net91,950; exclude30,000 of OFL recapture.
    assert_eq!(value(&r, "foreign_regular_passive_overall_domestic_loss_recaptured"), 45975);
    assert_eq!(value(&r, "foreign_regular_passive_overall_domestic_loss_remaining"), 54025);
    assert_eq!(r.tax, Some(2473400));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    if let ForeignTaxClaim::Ordinary {
        loss_history: ForeignLossHistory::Retained { accounts, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        accounts.reverse();
    }
    assert_eq!(supported(&d), r);
}

#[test]
fn foreign_loss_history_apportions_global_cap_and_honors_extra_recapture_choice() {
    let mut d = foreign_high_tax_return(12000);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    foreign_loss_records(
        &mut d,
        vec![
            ForeignLossAccount {
                category: ForeignCreditCategory::General,
                regular: foreign_loss_system(100000, 0, 0, 0),
                amt: foreign_loss_system(60000, 0, 0, 0),
            },
            ForeignLossAccount {
                category: ForeignCreditCategory::Passive,
                regular: foreign_loss_system(100000, 0, 0, 5000),
                amt: foreign_loss_system(50000, 0, 0, 0),
            },
        ],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_regular_general_overall_foreign_loss_recaptured"), 27585);
    assert_eq!(value(&r, "foreign_regular_passive_overall_foreign_loss_recaptured"), 23390);
    assert_eq!(value(&r, "foreign_regular_passive_overall_foreign_loss_remaining"), 76610);
    assert_eq!(value(&r, "foreign_amt_general_overall_foreign_loss_recaptured"), 50000);
    assert_eq!(value(&r, "foreign_amt_general_overall_foreign_loss_remaining"), 10000);
    assert_eq!(value(&r, "foreign_amt_passive_overall_foreign_loss_remaining"), 50000);
    assert_eq!(value(&r, "foreign_tax_credit"), 2675);
    assert_eq!(r.tax, Some(3405900));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    if let ForeignTaxClaim::Ordinary {
        loss_history: ForeignLossHistory::Retained { accounts, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        accounts[1].regular.additional_recapture = 3000000;
    }
    unsupported(&d, "Foreign loss recapture election");
}

#[test]
fn foreign_loss_history_nets_current_opposite_loss_before_recapturing_prior_balance() {
    let mut d = foreign_asset_return();
    investment_borrowing(&mut d, 10000, 0, 0);
    add(&mut d, income(foreign_pay(5000)));
    if let ForeignTaxClaim::Ordinary {
        sources,
        investment_assets: ForeignInvestmentAssets::TaxBook { assets, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        assets.remove(0);
        sources.push(ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "Foreign employment".into(),
            category: ForeignTaxCategory::Employment,
            income: 500000,
            tax: 0,
        });
    }
    foreign_loss_records(
        &mut d,
        vec![ForeignLossAccount {
            category: ForeignCreditCategory::General,
            regular: foreign_loss_system(0, 6000, 0, 0),
            amt: foreign_loss_system(0, 0, 0, 0),
        }],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_regular_general_separate_loss_netted"), 4000);
    assert_eq!(value(&r, "foreign_regular_general_separate_loss_recaptured"), 1000);
    assert_eq!(value(&r, "foreign_regular_general_separate_loss_remaining"), 1000);
    assert_eq!(value(&r, "foreign_regular_passive_separate_loss_remaining"), 0);
    // Income moves, but this year's foreign taxes do not move with recapture.
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert_eq!(r.tax, Some(1561800));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_domestic_loss_recapture_uses_retained_tax_credit_without_current_foreign_income() {
    let mut d = foreign_asset_return();
    d.income.clear();
    add(&mut d, wage(0, Owner::Taxpayer, 10000000, 0));
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    if let ForeignTaxClaim::Ordinary { sources, history, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources.clear();
        *history = foreign_history_of(&[ForeignCreditBalance {
            year: 2025,
            category: ForeignCreditCategory::General,
            regular: 1000000,
            amt: 0,
        }]);
    }
    foreign_loss_records(
        &mut d,
        vec![ForeignLossAccount {
            category: ForeignCreditCategory::General,
            regular: foreign_loss_system(0, 0, 20000, 0),
            amt: foreign_loss_system(0, 0, 0, 0),
        }],
    );
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_regular_general_overall_domestic_loss_recaptured"), 20000);
    assert_eq!(value(&r, "foreign_regular_general_overall_domestic_loss_remaining"), 0);
    assert_eq!(value(&r, "foreign_tax_credit"), 3141);
    assert_eq!(r.tax, Some(1003500));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let mut no_credit = d.clone();
    if let ForeignTaxClaim::Ordinary { history, .. } =
        &mut no_credit.credits.as_mut().unwrap().foreign_tax
    {
        *history = ForeignCreditHistory::None;
    }
    let no_credit_result = supported(&no_credit);
    assert_eq!(
        value(&no_credit_result, "foreign_regular_general_overall_domestic_loss_recaptured"),
        20000
    );
    assert_eq!(no_credit_result.tax, Some(1317600));
    if let ForeignTaxClaim::Ordinary {
        loss_history: ForeignLossHistory::Retained { same_filing_unit: confirmed, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *confirmed = false;
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_FOREIGN_LOSS_ACCOUNTS_LEFT_OUT"), "{r:?}");
}

#[test]
fn simplified_foreign_credit_accepts_confirmed_exhausted_loss_accounts() {
    let mut d = foreign_high_tax_return(12000);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    if let ForeignTaxClaim::Ordinary { amt_limitation, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *amt_limitation =
            ForeignAmtLimitation::Simplified { first_claim_year: 2026, confirmed: true };
    }
    let baseline = supported(&d);
    let zero = ForeignLossAccount {
        category: ForeignCreditCategory::General,
        regular: foreign_loss_system(0, 0, 0, 0),
        amt: foreign_loss_system(0, 0, 0, 0),
    };
    foreign_loss_records(&mut d, vec![zero.clone()]);
    let r = supported(&d);
    assert_eq!(r.tax, baseline.tax);
    for id in ["foreign_amt_general_net_foreign_income", "foreign_amt_passive_net_foreign_income"] {
        assert_eq!(value(&r, id), value(&baseline, id));
    }
    assert_eq!(value(&r, "foreign_regular_general_overall_domestic_loss_remaining"), 0);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    // Separate AMT balances still need the unresolved account coordination.
    for field in 0..4 {
        let mut row = zero.clone();
        let system = &mut row.amt;
        match field {
            0 => system.overall = 100,
            1 => system.separate = 100,
            2 => system.domestic = 100,
            _ => system.additional_recapture = 100,
        }
        foreign_loss_records(&mut d, vec![row]);
        unsupported(&d, "Simplified foreign loss accounts");
    }
    foreign_loss_records(&mut d, vec![zero]);
    if let ForeignTaxClaim::Ordinary {
        loss_history: ForeignLossHistory::Retained { same_filing_unit: confirmed, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *confirmed = false;
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_FOREIGN_LOSS_ACCOUNTS_LEFT_OUT"), "{r:?}");
}

#[test]
fn simplified_foreign_credit_uses_regular_loss_recapture_in_elected_numerator() {
    let mut d = foreign_high_tax_return(12000);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    set_foreign_amt_election(&mut d, 2020, true);
    foreign_loss_records(
        &mut d,
        vec![
            ForeignLossAccount {
                category: ForeignCreditCategory::General,
                regular: foreign_loss_system(20000, 5000, 0, 0),
                amt: foreign_loss_system(0, 0, 0, 0),
            },
            ForeignLossAccount {
                category: ForeignCreditCategory::Passive,
                regular: foreign_loss_system(10000, 0, 100000, 0),
                amt: foreign_loss_system(0, 0, 0, 0),
            },
        ],
    );
    let r = supported(&d);
    // Regular net55,170/36,780 minus OFL20,000/10,000, then SLL5,000
    // general to passive, then ODL45,975 to passive. AMT uses these final
    // regular amounts, not the independent normal-AMT net100,000/0.
    assert_eq!(form_1116_value(&r, "foreign_amt_general_net_foreign_income"), 30170);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 77755);
    assert_eq!(value(&r, "foreign_general_amt_unused_tax"), 7690);
    assert_eq!(value(&r, "foreign_regular_passive_overall_domestic_loss_remaining"), 54025);
    assert_eq!(value(&r, "foreign_tax_credit"), 12000);
    assert_eq!(r.tax, Some(2473400));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    if let ForeignTaxClaim::Ordinary {
        loss_history: ForeignLossHistory::Retained { accounts, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        accounts.reverse();
    }
    assert_eq!(supported(&d), r);
}

#[test]
fn foreign_deduction_year_absorbs_excess_without_a_credit_or_refund() {
    let mut d = foreign_high_tax_return(12000);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let baseline = supported(&d);
    let records = vec![
        prior_foreign_credit(ForeignCreditCategory::Passive, [3000, 1000], [5000, 0]),
        prior_foreign_credit(ForeignCreditCategory::General, [1000, 1000], [5000, 4000]),
    ];
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::DeductionYear { categories: records.clone(), confirmed: true },
    );
    let r = supported(&d);
    // Regular passive excess12,000 -7,347 =4,653; hypothetical capacity2,000.
    assert_eq!(value(&r, "foreign_passive_regular_deduction_year_absorbed_prior_year"), 2000);
    assert_eq!(value(&r, "foreign_passive_regular_new_carryforward"), 2653);
    // AMT high-tax classification moves all12,000 to general: current
    // credit14,287 exceeds paid taxes, so no AMT excess is absorbed.
    assert_eq!(value(&r, "foreign_general_amt_deduction_year_absorbed_prior_year"), 0);
    assert_eq!(r.tax, baseline.tax);
    assert!(!r.lines.iter().any(|l| l.id.ends_with("carryback_prior_year")));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let mut elected = d.clone();
    set_foreign_amt_election(&mut elected, 2020, true);
    let elected_result = supported(&elected);
    assert_eq!(
        value(&elected_result, "foreign_general_amt_deduction_year_absorbed_prior_year"),
        1000
    );
    assert_eq!(value(&elected_result, "foreign_general_amt_new_carryforward"), 3118);
    assert_eq!(elected_result.tax, r.tax);
    assert_eq!(supported(&restart(elected).draft), elected_result);
    let mut conflicting = d.clone();
    set_foreign_history(
        &mut conflicting,
        vec![foreign_balance(2020, ForeignCreditCategory::Passive, 1, 0)],
    );
    unsupported(&conflicting, "Foreign-credit carryback history");
    let mut reversed = records.clone();
    reversed.reverse();
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::DeductionYear { categories: reversed, confirmed: true },
    );
    assert_eq!(supported(&d), r);
    set_foreign_carryback(
        &mut d,
        ForeignCarryback::DeductionYear { categories: records, confirmed: false },
    );
    // Unconfirmed worksheets: the carryback stays unreviewed.
    supported(&d);
}

#[test]
fn foreign_prior_election_cannot_generate_same_year_carryovers() {
    for choice in [
        ForeignCarryback::SmallCreditElection,
        ForeignCarryback::DeductionYear { categories: vec![], confirmed: true },
    ] {
        for category in [ForeignCreditCategory::General, ForeignCreditCategory::Passive] {
            for (regular, amt) in [(1, 0), (0, 1)] {
                let mut d = foreign_high_tax_return(12000);
                set_foreign_carryback(&mut d, choice.clone());
                set_foreign_history(&mut d, vec![foreign_balance(2025, category, regular, amt)]);
                unsupported(&d, "Foreign-credit origin year");
                unsupported(&restart(d.clone()).draft, "Foreign-credit origin year");
                // Surviving older balances retain their true origin year.
                set_foreign_history(&mut d, vec![foreign_balance(2024, category, regular, amt)]);
                supported(&d);
                // A recorded zero is not a generated credit balance.
                set_foreign_history(&mut d, vec![foreign_balance(2025, category, 0, 0)]);
                supported(&d);
            }
        }
    }
}
#[test]
fn foreign_employee_housing_precedes_earned_exclusion_and_stacks_tax() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(200000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    employee_housing(&mut d, 50000, ForeignHousingLocation::General);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_housing_exclusion"), 18606);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
    assert_eq!(value(&r, "agi"), 48494);
    assert_eq!(value(&r, "taxable_income"), 32394);
    assert_eq!(value(&r, "income_tax"), 7775);
    assert_eq!(r.tax, Some(777500));
    assert_eq!(return_line(&r, "8"), -15150600);
    assert_eq!(supported(&restart(d).draft), r);
}
#[test]
fn foreign_employee_housing_can_be_elected_alone_and_cannot_exceed_pay() {
    for (pay, earned_election, housing, earned) in
        [(10000, false, 10000, 0), (140000, true, 18606, 121394), (200000, false, 18606, 0)]
    {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(foreign_pay(pay)));
        employee_housing(&mut d, 50000, ForeignHousingLocation::General);
        claim_mut(&mut d, Owner::Taxpayer).exclusion = earned_election;
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_housing_exclusion"), housing);
        assert_eq!(value(&r, "foreign_earned_exclusion"), earned);
        assert_eq!(value(&r, "agi"), pay - housing - earned);
        assert_eq!(value(&r, "earned_income_credit"), 0);
        assert_eq!(supported(&restart(d).draft), r);
    }
}
#[test]
fn foreign_housing_uses_published_location_limit_and_is_claimed_once_per_person() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(200000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    employee_housing(&mut d, 60000, ForeignHousingLocation::AustraliaWollongong);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_housing_exclusion"), 24736);
    // A second employer adds pay, never a second housing claim.
    let mut second = d.income[0].clone();
    second.id = 0;
    add(&mut d, second);
    assert_eq!(value(&supported(&d), "foreign_housing_exclusion"), 24736);
    d.income.pop();
    if let ForeignHousing::Claim { household, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing
    {
        *household = ForeignHousingHousehold::Adverse { confirmed: false };
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_FOREIGN_HOUSING_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "foreign_housing_exclusion"), 0);
}

#[test]
fn foreign_housing_partial_year_uses_published_daily_column() {
    let q = ForeignQualification::Presence { starts_on: 20250701, foreign_days: 365 };
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(100000)));
    employee_housing(&mut d, 40000, ForeignHousingLocation::AustraliaWollongong);
    claim_mut(&mut d, Owner::Taxpayer).qualification = q;
    if let ForeignHousing::Claim { stays, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing {
        stays[0].ends_on = 20260630;
    }
    let r = supported(&d);
    //181 days: published123.06/day gives22,274; base58.26/day gives10,545.
    assert_eq!(value(&r, "foreign_housing_exclusion"), 11729);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_housing_moves_sum_location_limits_and_reject_overlapping_dates() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(200000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    employee_housing(&mut d, 80000, ForeignHousingLocation::General);
    let records = vec![
        ForeignHousingStay {
            location: ForeignHousingLocation::General,
            starts_on: 20260101,
            ends_on: 20260630,
        },
        ForeignHousingStay {
            location: ForeignHousingLocation::ChinaHongKong,
            starts_on: 20260701,
            ends_on: 20261231,
        },
    ];
    if let ForeignHousing::Claim { stays, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing {
        *stays = records;
    }
    let r = supported(&d);
    //General181 days19,771 + Hong Kong184 days57,620 - annualbase21,264.
    assert_eq!(value(&r, "foreign_housing_exclusion"), 56127);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
    assert_eq!(value(&r, "agi"), 10973);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    if let ForeignHousing::Claim { stays, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing {
        stays.reverse();
    }
    assert_eq!(supported(&d), r);
    if let ForeignHousing::Claim { stays, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing {
        stays[0].starts_on = 20260630;
    }
    unsupported(&d, "Foreign housing dates");
}

#[test]
fn foreign_housing_repeated_stays_preserve_annual_limit_and_period_boundary() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(200000)));
    employee_housing(&mut d, 60000, ForeignHousingLocation::AustraliaWollongong);
    let baseline = supported(&d);
    if let ForeignHousing::Claim { stays, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing {
        stays[0].ends_on = 20260630;
        stays.push(ForeignHousingStay {
            location: ForeignHousingLocation::AustraliaWollongong,
            starts_on: 20260701,
            ends_on: 20261231,
        });
    }
    assert_eq!(supported(&d), baseline);
    claim_mut(&mut d, Owner::Taxpayer).qualification =
        ForeignQualification::Presence { starts_on: 20250701, foreign_days: 365 };
    unsupported(&d, "Foreign housing dates");
}

#[test]
fn foreign_housing_spouses_allocate_shared_expenses_without_duplication() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, income(foreign_pay(200000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    employee_housing(&mut d, 30000, ForeignHousingLocation::General);
    let mut spouse = d.income[0].clone();
    spouse.id = 0;
    spouse.owner = Owner::Spouse;
    add(&mut d, spouse);
    let mut spouse_claim = claim_mut(&mut d, Owner::Taxpayer).clone();
    spouse_claim.owner = Owner::Spouse;
    set_claim(&mut d, spouse_claim);
    for c in &mut d.screening.as_mut().unwrap().foreign_earned {
        if let ForeignHousing::Claim { household, .. } = &mut c.housing {
            *household = ForeignHousingHousehold::Shared { total_expenses: 6000000 };
        }
    }
    let r = supported(&d);
    let mut conflicting_location = d.clone();
    if let ForeignHousing::Claim { stays, .. } =
        &mut claim_mut(&mut conflicting_location, Owner::Spouse).housing
    {
        stays[0].location = ForeignHousingLocation::ChinaHongKong;
    }
    unsupported(&conflicting_location, "Foreign housing allocation");
    let mut split_stay = d.clone();
    if let ForeignHousing::Claim { stays, .. } =
        &mut claim_mut(&mut split_stay, Owner::Spouse).housing
    {
        stays[0].ends_on = 20260630;
        stays.push(ForeignHousingStay {
            location: ForeignHousingLocation::General,
            starts_on: 20260701,
            ends_on: 20261231,
        });
    }
    assert_eq!(supported(&split_stay), r);
    for owner in ["taxpayer", "spouse"] {
        assert_eq!(value(&r, &alloc::format!("foreign_housing_{owner}_expenses")), 30000);
        assert_eq!(value(&r, &alloc::format!("foreign_housing_{owner}_limit")), 39870);
        assert_eq!(value(&r, &alloc::format!("foreign_housing_{owner}_base")), 21264);
        assert_eq!(value(&r, &alloc::format!("foreign_housing_{owner}_exclusion")), 8736);
    }
    //Each spouse:30,000 -21,264 =8,736; FEIE132,900 independently.
    assert_eq!(value(&r, "foreign_housing_exclusion"), 17472);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 265800);
    assert_eq!(value(&r, "agi"), 116728);
    assert_eq!(value(&r, "taxable_income"), 84528);
    assert_eq!(value(&r, "income_tax"), 20287);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    for c in &mut d.screening.as_mut().unwrap().foreign_earned {
        if let ForeignHousing::Claim { household, .. } = &mut c.housing {
            *household = ForeignHousingHousehold::Shared { total_expenses: 5999999 };
        }
    }
    unsupported(&d, "Foreign housing allocation");
    for c in &mut d.screening.as_mut().unwrap().foreign_earned {
        if let ForeignHousing::Claim { household, .. } = &mut c.housing {
            *household = ForeignHousingHousehold::Separate { qualified: true };
        }
    }
    assert_eq!(supported(&d), r);
    if let ForeignHousing::Claim { household, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing
    {
        *household = ForeignHousingHousehold::Separate { qualified: false };
    }
    // Spouses answering the shared commuting question differently: it is
    // taken as No, so both housing claims are left out with a notice.
    let differing = supported(&d);
    assert!(noted(&differing, "WARN_FOREIGN_HOUSING_LEFT_OUT"), "{differing:?}");
    assert_eq!(value(&differing, "foreign_housing_exclusion"), 0);
    // Both not qualified: both housing claims are left out with a notice.
    for c in &mut d.screening.as_mut().unwrap().foreign_earned {
        if let ForeignHousing::Claim { household, .. } = &mut c.housing {
            *household = ForeignHousingHousehold::Separate { qualified: false };
        }
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_FOREIGN_HOUSING_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "foreign_housing_exclusion"), 0);
    assert_eq!(r, differing);
}

#[test]
fn foreign_housing_separate_return_uses_recorded_spouse_expenses() {
    let mut d = empty_return(FederalStatus::Separate);
    add(&mut d, income(foreign_pay(200000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    employee_housing(&mut d, 30000, ForeignHousingLocation::General);
    if let ForeignHousing::Claim { household, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing
    {
        *household = ForeignHousingHousehold::SeparateReturnShared {
            total_expenses: 6000000,
            spouse_expenses: 3000000,
            confirmed: true,
        };
    }
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_housing_exclusion"), 8736);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
    assert_eq!(value(&r, "agi"), 58364);
    assert_eq!(value(&r, "income_tax"), 10143);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let mut wrong_status = d.clone();
    wrong_status.setup = complete(FederalStatus::Single).setup;
    unsupported(&wrong_status, "Foreign housing allocation");
    let mut unconfirmed = d.clone();
    if let ForeignHousing::Claim {
        household: ForeignHousingHousehold::SeparateReturnShared { confirmed, .. },
        ..
    } = &mut claim_mut(&mut unconfirmed, Owner::Taxpayer).housing
    {
        *confirmed = false;
    }
    assert!(noted(&supported(&unconfirmed), "WARN_FOREIGN_HOUSING_LEFT_OUT"));
    if let ForeignHousing::Claim {
        household: ForeignHousingHousehold::SeparateReturnShared { spouse_expenses, .. },
        ..
    } = &mut claim_mut(&mut d, Owner::Taxpayer).housing
    {
        *spouse_expenses += 1;
    }
    unsupported(&d, "Foreign housing allocation");
}

#[test]
fn foreign_housing_separate_household_on_separate_return_requires_qualification() {
    let mut d = empty_return(FederalStatus::Separate);
    add(&mut d, income(foreign_pay(200000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    employee_housing(&mut d, 30000, ForeignHousingLocation::General);
    let baseline = supported(&d);
    for qualified in [true, false] {
        if let ForeignHousing::Claim { household, .. } =
            &mut claim_mut(&mut d, Owner::Taxpayer).housing
        {
            *household = ForeignHousingHousehold::Separate { qualified };
        }
        let r = supported(&d);
        if qualified {
            assert_eq!(r, baseline);
            assert_eq!(value(&r, "foreign_housing_exclusion"), 8736);
            assert_eq!(supported(&restart(d.clone()).draft), r);
        } else {
            // Not qualified: the housing claim is left out with a notice.
            assert!(noted(&r, "WARN_FOREIGN_HOUSING_LEFT_OUT"), "{r:?}");
            assert_eq!(value(&r, "foreign_housing_exclusion"), 0);
        }
    }
}

#[test]
fn foreign_service_business_keeps_income_and_se_tax_without_qbi() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_service_business(50000)));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(value(&r, "agi"), 46467);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(r.tax, Some(1046200));
    let id = d.income[0].id;
    assert_eq!(value(&r, &alloc::format!("business_{id}_foreign_net_income")), 50000);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let FederalIncome::Business { data } = &mut d.income[0].income else { unreachable!() };
    data.source = BusinessSource::ForeignServices { confirmed: false, totalization_exempt: false };
    // An unconfirmed wholly foreign business is left out with a notice.
    assert!(noted(&estimate(&d), "WARN_BUSINESS_LEFT_OUT"));
}

#[test]
fn foreign_service_business_allocates_se_deduction_before_domestic_qbi() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(50000)));
    add(&mut d, income(foreign_service_business(50000)));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 14130);
    assert_eq!(value(&r, "agi"), 92935);
    assert_eq!(value(&r, "qbi_deduction"), 9293);
    assert_eq!(r.tax, Some(2369800));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.income.reverse();
    let reverse = supported(&d);
    assert_eq!(reverse.tax, r.tax);
    assert_eq!(value(&reverse, "qbi_deduction"), 9293);
}

#[test]
fn foreign_service_business_deductions_do_not_create_qbi_loss() {
    let mut d = empty_return(FederalStatus::Single);
    let FederalIncome::Business { mut data } = foreign_service_business(50000) else {
        unreachable!()
    };
    data.retirement = BusinessRetirement::Sep { contribution: 500000, rate: 2500 };
    data.health_insurance = BusinessHealthInsurance::Plan {
        months: vec![BusinessHealthMonth {
            month: 1,
            premiums: 100000,
            child_increase: 0,
            employer_eligible: false,
            reimbursed: None,
        }],
        long_term_care: vec![],
        establishment: crate::HealthEstablishment::SoleProprietor,
    };
    add(&mut d, income(FederalIncome::Business { data }));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(value(&r, "agi"), 40467);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(r.tax, Some(974200));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_service_loss_reduces_income_without_becoming_qbi_loss() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(50000)));
    add(&mut d, income(foreign_service_business(-10000)));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 5652);
    assert_eq!(value(&r, "agi"), 97174);
    assert_eq!(value(&r, "qbi_deduction"), 9435);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 0);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn foreign_service_business_preserves_unresolved_combination_guards() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(foreign_service_business(50000)));
    // An unconfirmed foreign service business is left out with a notice.
    let mut unconfirmed = d.clone();
    let FederalIncome::Business { data } = &mut unconfirmed.income.last_mut().unwrap().income
    else {
        unreachable!()
    };
    data.source = BusinessSource::ForeignServices { confirmed: false, totalization_exempt: false };
    let r = supported(&unconfirmed);
    assert!(noted(&r, "WARN_BUSINESS_LEFT_OUT"));
    assert_eq!(value(&r, "self_employment_tax"), 0);
    let mut loss = d.clone();
    add(&mut loss, income(foreign_service_business(-400000)));
    add(&mut loss, income(cash_business(-10000)));
    unsupported(&loss, "Foreign business loss allocation");
}

#[test]
fn foreign_only_excess_loss_retains_nonqualified_nol_character() {
    for (status, wages, loss, excess, agi, nol) in [
        (FederalStatus::Single, 1000000, 400000, 144000, 744000, 144000),
        (FederalStatus::Joint, 100000, 600000, 88000, -412000, 500000),
    ] {
        let mut d = empty_return(status);
        add(&mut d, wage(0, Owner::Taxpayer, wages * 100, 0));
        add(&mut d, income(foreign_service_business(-loss)));
        let r = supported(&d);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "excess_business_loss"), excess);
        assert_eq!(value(&r, "business_nonqualified_excess_loss"), excess);
        assert_eq!(value(&r, "excess_business_loss_carryforward"), excess);
        assert_eq!(value(&r, "current_net_operating_loss"), nol - excess);
        assert_eq!(value(&r, "nol_carryforward"), nol);
        assert_eq!(value(&r, "qbi_loss_carryforward"), 0);
        assert_eq!(value(&r, "qbi_deduction"), 0);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        // The limitation depends on aggregate source amounts, not record count.
        let mut split = d.clone();
        split.income.pop();
        add(&mut split, income(foreign_service_business(-loss / 2)));
        add(&mut split, income(foreign_service_business(-loss / 2)));
        let two = supported(&split);
        assert_eq!(two.tax, r.tax);
        assert_eq!(value(&two, "nol_carryforward"), nol);
        assert_eq!(value(&two, "qbi_loss_carryforward"), 0);
        split.income.reverse();
        assert_eq!(supported(&split).tax, r.tax);
        add(&mut split, income(cash_business(0)));
        let zero_domestic = supported(&split);
        assert_eq!(zero_domestic.tax, r.tax);
        assert_eq!(
            zero_domestic
                .lines
                .iter()
                .filter(|line| line.id == "business_nonqualified_excess_loss")
                .count(),
            1
        );
        add(&mut split, income(cash_business(-10000)));
        supported(&split);
        add(&mut split, income(cash_business(1000)));
        unsupported(&split, "Foreign business loss allocation");
    }
}

#[test]
fn mixed_foreign_domestic_loss_only_allocates_suspended_character() {
    for (status, domestic, foreign, excess, qualified, nonqualified, qbi_carry, agi, nol, tax) in [
        (
            FederalStatus::Single,
            300000,
            100000,
            144000,
            108000,
            36000,
            192000,
            44000,
            144000,
            400300,
        ),
        (FederalStatus::Joint, 300000, 300000, 88000, 44000, 44000, 256000, -212000, 300000, 45000),
    ] {
        let mut d = empty_return(status);
        add(&mut d, wage(0, Owner::Taxpayer, 30000000, 0));
        add(&mut d, income(cash_business(-domestic)));
        add(&mut d, income(foreign_service_business(-foreign)));
        let r = supported(&d);
        assert_eq!(r.tax, Some(tax));
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "excess_business_loss"), excess);
        assert_eq!(value(&r, "business_qualified_excess_loss"), qualified);
        assert_eq!(value(&r, "business_nonqualified_excess_loss"), nonqualified);
        assert_eq!(qualified + nonqualified, excess);
        assert_eq!(value(&r, "qbi_loss_carryforward"), qbi_carry);
        assert_eq!(value(&r, "nol_carryforward"), nol);
        assert_eq!(value(&r, "current_net_operating_loss"), nol - excess);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        d.income.reverse();
        assert_eq!(supported(&d).tax, r.tax);
        add(&mut d, income(cash_business(1000)));
        unsupported(&d, "Foreign business loss allocation");
    }
}

#[test]
fn mixed_foreign_losses_fix_service_eligibility_in_loss_year() {
    for (wages, qualified, nonqualified, carry) in
        [(300000, 108000, 36000, 192000), (511350, 54000, 90000, 96000), (1000000, 0, 144000, 0)]
    {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, wage(0, Owner::Taxpayer, wages * 100, 0));
        add(&mut d, income(cash_business(-300000)));
        add(&mut d, income(foreign_service_business(-100000)));
        let ordinary = supported(&d);
        let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
        data.specified_service = true;
        let r = supported(&d);
        assert_eq!(r.tax, ordinary.tax);
        assert_eq!(value(&r, "business_qualified_excess_loss"), qualified);
        assert_eq!(value(&r, "business_nonqualified_excess_loss"), nonqualified);
        assert_eq!(value(&r, "excess_business_loss"), 144000);
        assert_eq!(value(&r, "qbi_loss_carryforward"), carry);
        assert_eq!(value(&r, "nol_carryforward"), 144000);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
        // Correcting the recorded classification to a nonservice business preserves ordinary tax.
        data.specified_service = false;
        let unsure = supported(&d);
        assert_eq!(unsure.tax, ordinary.tax);
    }
}

#[test]
fn foreign_business_and_other_spouse_employee_exclusions_are_independent() {
    for housing in [false, true] {
        for employee_owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut d = empty_return(FederalStatus::Joint);
            let mut pay = income(foreign_pay(200000));
            pay.owner = employee_owner;
            add(&mut d, pay);
            set_claim(&mut d, foreign_claim(employee_owner));
            if housing {
                employee_housing(&mut d, 50000, ForeignHousingLocation::General);
            }
            let mut business = income(foreign_service_business(50000));
            business.owner =
                if employee_owner == Owner::Taxpayer { Owner::Spouse } else { Owner::Taxpayer };
            add(&mut d, business);
            let r = supported(&d);
            assert_eq!(value(&r, "self_employment_tax"), 7065);
            assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
            assert_eq!(value(&r, "agi"), if housing { 94961 } else { 113567 });
            assert_eq!(value(&r, "qbi_deduction"), 0);
            assert_eq!(r.tax, Some(if housing { 2093000 } else { 2502300 }));
            if housing {
                assert_eq!(value(&r, "foreign_housing_exclusion"), 18606);
            }
            assert_eq!(supported(&restart(d.clone()).draft), r);
            let mut zero = d.clone();
            let FederalIncome::ForeignEmployment { data } = &mut zero.income[0].income else {
                unreachable!()
            };
            data.wages = 0;
            let zero_result = supported(&zero);
            assert_eq!(value(&zero_result, "foreign_earned_exclusion"), 0);
            assert_eq!(zero_result.tax, Some(849300));
            d.income.reverse();
            assert_eq!(supported(&d).tax, r.tax);
            // The same person's business joins the one exclusion limit of
            // the person's record.
            let business = d
                .income
                .iter_mut()
                .find(|e| matches!(e.income, FederalIncome::Business { .. }))
                .unwrap();
            business.owner = employee_owner;
            assert_eq!(value(&supported(&d), "foreign_earned_exclusion"), 132900);
        }
    }
}

#[test]
fn foreign_service_exclusion_uses_gross_receipts_and_restores_deductions() {
    for (receipts, expenses, se, related, net_exclusion, agi, tax) in [
        (60000, 10000, 7065, 13533, 46467, 0, 706500),
        (200000, 50000, 21194, 40267, 92633, 46770, 2829500),
    ] {
        let mut d = empty_return(FederalStatus::Single);
        let FederalIncome::Business { mut data } = foreign_service_business(receipts) else {
            unreachable!()
        };
        data.expenses.push(ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Service costs".into() },
            expenses * 100,
        ));
        data.source =
            BusinessSource::ForeignServices { confirmed: true, totalization_exempt: false };
        add(&mut d, income(FederalIncome::Business { data }));
        set_claim(&mut d, foreign_claim(Owner::Taxpayer));
        let r = supported(&d);
        assert_eq!(value(&r, "self_employment_tax"), se);
        assert_eq!(value(&r, "foreign_earned_exclusion"), receipts.min(132900));
        assert_eq!(value(&r, "foreign_business_taxpayer_disallowed_deductions"), related);
        assert_eq!(value(&r, "foreign_business_taxpayer_net_exclusion"), net_exclusion);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "qbi_deduction"), 0);
        assert_eq!(r.tax, Some(tax));
        assert_eq!(supported(&restart(d.clone()).draft), r);
        add(&mut d, income(cash_business(10000)));
        assert_eq!(supported(&restart(d.clone()).draft), supported(&d));
    }
}

#[test]
fn foreign_business_exclusion_and_spouse_employee_share_return_not_limits() {
    let mut d = empty_return(FederalStatus::Joint);
    let FederalIncome::Business { mut data } = foreign_service_business(50000) else {
        unreachable!()
    };
    data.source = BusinessSource::ForeignServices { confirmed: true, totalization_exempt: false };
    add(&mut d, income(FederalIncome::Business { data }));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let mut pay = income(foreign_pay(200000));
    pay.owner = Owner::Spouse;
    add(&mut d, pay);
    set_claim(&mut d, foreign_claim(Owner::Spouse));
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 182900);
    assert_eq!(value(&r, "agi"), 67100);
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(r.tax, Some(1480000));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn excluded_foreign_business_profit_does_not_absorb_spouse_business_loss() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, wage(0, Owner::Taxpayer, 30000000, 0));
    let FederalIncome::Business { mut data } = foreign_service_business(50000) else {
        unreachable!()
    };
    data.source = BusinessSource::ForeignServices { confirmed: true, totalization_exempt: false };
    add(&mut d, income(FederalIncome::Business { data }));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let mut loss = income(cash_business(-600000));
    loss.owner = Owner::Spouse;
    add(&mut d, loss);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), -212000);
    assert_eq!(value(&r, "excess_business_loss"), 88000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 88000);
    assert_eq!(value(&r, "nol_carryforward"), 300000);
    assert_eq!(value(&r, "business_qualified_excess_loss"), 88000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 512000);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_service_exclusion_prorates_period_and_rejects_unresolved_deductions() {
    let mut d = empty_return(FederalStatus::Single);
    let FederalIncome::Business { mut data } = foreign_service_business(200000) else {
        unreachable!()
    };
    data.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Service costs".into() },
        5000000,
    ));
    data.source = BusinessSource::ForeignServices { confirmed: true, totalization_exempt: false };
    add(&mut d, income(FederalIncome::Business { data }));
    let mut claim = foreign_claim(Owner::Taxpayer);
    claim.qualification = ForeignQualification::Presence { starts_on: 20250701, foreign_days: 365 };
    set_claim(&mut d, claim);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 65904);
    assert_eq!(value(&r, "agi"), 93467);
    assert_eq!(r.tax, Some(3812200));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let mut invalid = d.clone();
    let FederalIncome::Business { data } = &mut invalid.income[0].income else { unreachable!() };
    data.retirement = BusinessRetirement::Sep { contribution: 100000, rate: 2500 };
    unsupported(&invalid, "Foreign business exclusion");
    claim_mut(&mut d, Owner::Taxpayer).tax_home = false;
    // Explicit No: no exclusion for this person.
    let unsure = supported(&d);
    assert_eq!(value(&unsure, "foreign_earned_exclusion"), 0);
}

#[test]
fn multiple_foreign_service_businesses_share_one_exclusion_limit() {
    let mut d = empty_return(FederalStatus::Single);
    for _ in 0..2 {
        let FederalIncome::Business { mut data } = foreign_service_business(100000) else {
            unreachable!()
        };
        data.expenses.push(ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Service costs".into() },
            2500000,
        ));
        data.source =
            BusinessSource::ForeignServices { confirmed: true, totalization_exempt: false };
        add(&mut d, income(FederalIncome::Business { data }));
    }
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
    assert_eq!(value(&r, "foreign_business_taxpayer_gross_exclusion"), 132900);
    assert_eq!(value(&r, "foreign_business_taxpayer_disallowed_deductions"), 40267);
    assert_eq!(value(&r, "foreign_business_taxpayer_net_exclusion"), 92633);
    assert_eq!(value(&r, "self_employment_tax"), 21194);
    assert_eq!(value(&r, "agi"), 46770);
    assert_eq!(r.tax, Some(2829500));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
    let mut loss = d;
    let FederalIncome::Business { data } = &mut loss.income[0].income else { unreachable!() };
    data.expenses[0].amount = 15000000;
    let result = supported(&loss);
    assert_eq!(supported(&restart(loss.clone()).draft), result);
    loss.income.reverse();
    assert_eq!(supported(&loss).tax, result.tax);
}

#[test]
fn same_person_foreign_wages_and_business_share_cap_and_expense_ratio() {
    for (wages, business_gross, employee_gross, related, net, agi, tax) in [
        (100000, 66450, 66450, 20134, 46316, 56935, 2039700),
        (200000, 44300, 88600, 13422, 30878, 150223, 4693200),
    ] {
        let mut d = combined_foreign_wage_business(wages, FederalStatus::Single);
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
        assert_eq!(value(&r, "foreign_business_taxpayer_gross_exclusion"), business_gross);
        assert_eq!(value(&r, "foreign_employee_taxpayer_gross_exclusion"), employee_gross);
        assert_eq!(value(&r, "foreign_business_taxpayer_disallowed_deductions"), related);
        assert_eq!(value(&r, "foreign_business_taxpayer_net_exclusion"), net);
        assert_eq!(value(&r, "self_employment_tax"), 10597);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(r.tax, Some(tax));
        assert_eq!(supported(&restart(d.clone()).draft), r);
        d.income.reverse();
        assert_eq!(supported(&d).tax, r.tax);
        let mut housing = combined_foreign_wage_business(wages, FederalStatus::Single);
        employee_housing(&mut housing, 50000, ForeignHousingLocation::General);
        supported(&housing);
    }
}

#[test]
fn combined_foreign_exclusion_allocates_only_business_share_to_loss_limit() {
    let mut d = combined_foreign_wage_business(200000, FederalStatus::Joint);
    let mut loss = income(cash_business(-600000));
    loss.owner = Owner::Spouse;
    add(&mut d, loss);
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss"), 49177);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 49177);
    assert_eq!(value(&r, "agi"), -400600);
    assert_eq!(value(&r, "nol_carryforward"), 449777);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 550823);
    assert_eq!(r.tax, Some(1059700));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn negative_foreign_exclusion_does_not_reduce_tax_rate_stack() {
    use crate::income_tax::{AmtTaxWorksheet, Preference, RegularTaxWorksheet};
    // Form 1040 and Form 6251 foreign-earned-income worksheets, line 2c:
    // a signed Form 2555 loss adjustment cannot make the rate stack negative.
    for excluded in [0, -10000, -200000] {
        for (gain, regular_expected, amt_expected) in [(0, 16712, 28574), (20000, 15318, 26374)] {
            let preference = Preference { dividends: 0, gain, line18: 0, line19: 0 };
            assert_eq!(
                RegularTaxWorksheet {
                    taxable: 100000,
                    preference,
                    excluded,
                    foreign_election: true,
                    status: FederalStatus::Single,
                }
                .tax(),
                regular_expected
            );
            assert_eq!(
                AmtTaxWorksheet {
                    year: 2026,
                    status: FederalStatus::Single,
                    base: 200000,
                    preferential: gain,
                    regular_ordinary: 100000 - gain,
                    excluded,
                    special: [0, 0],
                    foreign: Some(preference),
                }
                .tax(),
                amt_expected
            );
        }
    }
}

#[test]
fn foreign_exclusion_restores_loss_without_inventing_ira_compensation() {
    for wages in [0, 60000] {
        let mut d = empty_return(FederalStatus::Single);
        if wages > 0 {
            add(&mut d, wage(0, Owner::Taxpayer, wages * 100, 0));
        }
        add(&mut d, income(excluded_foreign_business(20000, 30000)));
        set_claim(&mut d, foreign_claim(Owner::Taxpayer));
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_business_taxpayer_net_exclusion"), -10000);
        assert_eq!(value(&r, "foreign_business_taxpayer_disallowed_deductions"), 30000);
        assert_eq!(value(&r, "agi"), wages);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(value(&r, "nol_carryforward"), 0);
        assert_eq!(value(&r, "qbi_loss_carryforward"), 0);
        assert_eq!(r.tax, Some(if wages == 0 { 0 } else { 502300 }));
        assert_eq!(supported(&restart(d.clone()).draft), r);
        d.adjustments.as_mut().unwrap().iras = ira_facts(100000, 0, Answer::No);
        if wages == 0 {
            unsupported(&d, "IRA contributions");
        } else {
            assert_eq!(value(&supported(&d), "ira_deduction"), 1000);
        }
    }
}

#[test]
fn restored_foreign_loss_does_not_take_domestic_qbi_suspension() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    add(&mut d, income(excluded_foreign_business(20000, 30000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let mut domestic = income(cash_business(-600000));
    domestic.owner = Owner::Spouse;
    add(&mut d, domestic);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), -212000);
    assert_eq!(value(&r, "excess_business_loss"), 88000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 88000);
    assert_eq!(value(&r, "business_qualified_excess_loss"), 88000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 512000);
    assert_eq!(value(&r, "nol_carryforward"), 300000);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
    assert_eq!(value(&supported(&d), "qbi_loss_carryforward"), 512000);
}

#[test]
fn partial_foreign_loss_exclusion_retains_only_unexcluded_loss() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(excluded_foreign_business(200000, 300000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
    assert_eq!(value(&r, "foreign_business_taxpayer_disallowed_deductions"), 199350);
    assert_eq!(value(&r, "foreign_business_taxpayer_net_exclusion"), -66450);
    assert_eq!(value(&r, "agi"), -33550);
    assert_eq!(value(&r, "nol_carryforward"), 33550);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 0);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_profit_and_loss_share_exclusion_without_phantom_carryovers() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(excluded_foreign_business(60000, 10000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    add(&mut d, income(excluded_foreign_business(20000, 30000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 80000);
    assert_eq!(value(&r, "foreign_business_taxpayer_disallowed_deductions"), 42826);
    assert_eq!(value(&r, "foreign_business_taxpayer_net_exclusion"), 37174);
    assert_eq!(value(&r, "self_employment_tax"), 5652);
    assert_eq!(value(&r, "agi"), 0);
    assert_eq!(value(&r, "nol_carryforward"), 0);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 0);
    assert_eq!(r.tax, Some(565200));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
    d.adjustments.as_mut().unwrap().iras = ira_facts(100, 0, Answer::No);
    unsupported(&d, "IRA contributions");
}

#[test]
fn domestic_business_stays_taxable_beside_same_owner_foreign_exclusion() {
    for (domestic, receipts, expenses, se, excluded, agi, qbi, nol, qbi_carry, tax) in [
        (50000, 50000, 0, 14130, 46468, 46467, 6073, 0, 0, 19084),
        (-10000, 50000, 0, 5652, 47174, -10000, 0, 10000, 10000, 5652),
        (50000, 20000, 30000, 5652, -10000, 47174, 6215, 0, 0, 8389),
        (-600000, 20000, 30000, 0, -10000, -256000, 0, 600000, 256000, 0),
    ] {
        let mut d = empty_return(FederalStatus::Single);
        // Domestic business first owns the rounding dollar of a shared half-SE.
        add(&mut d, income(cash_business(domestic)));
        add(&mut d, income(excluded_foreign_business(receipts, expenses)));
        set_claim(&mut d, foreign_claim(Owner::Taxpayer));
        let r = supported(&d);
        assert_eq!(value(&r, "self_employment_tax"), se);
        assert_eq!(value(&r, "foreign_earned_exclusion"), receipts);
        assert_eq!(value(&r, "foreign_business_taxpayer_net_exclusion"), excluded);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "qbi_deduction"), qbi);
        assert_eq!(value(&r, "nol_carryforward"), nol);
        assert_eq!(value(&r, "qbi_loss_carryforward"), qbi_carry);
        assert_eq!(r.tax, Some(tax * 100));
        assert_eq!(supported(&restart(d.clone()).draft), r);
        d.income.reverse();
        let reordered = supported(&d);
        assert_eq!(reordered.tax, r.tax);
        assert_eq!(value(&reordered, "qbi_deduction"), qbi);
        assert_eq!(value(&reordered, "foreign_business_taxpayer_net_exclusion"), excluded);
        d.adjustments.as_mut().unwrap().iras = ira_facts(100000, 0, Answer::No);
        d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
        if domestic > 0 {
            assert_eq!(value(&supported(&d), "ira_deduction"), 1000);
        } else {
            unsupported(&d, "IRA contributions");
        }
    }
}

#[test]
fn foreign_housing_allowance_reduces_limited_amount_not_expense_input() {
    for (allowance, housing, agi, tax) in
        [(5000, 13606, 53494, 8975), (18606, 0, 67100, 12240), (100000, 0, 67100, 12240)]
    {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(foreign_pay(200000)));
        set_claim(&mut d, foreign_claim(Owner::Taxpayer));
        employee_housing(&mut d, 50000, ForeignHousingLocation::General);
        let ForeignHousing::Claim { nontaxable_allowances, .. } =
            &mut claim_mut(&mut d, Owner::Taxpayer).housing
        else {
            unreachable!()
        };
        *nontaxable_allowances = allowance * 100;
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_housing_taxpayer_nontaxable_allowances"), allowance);
        assert_eq!(value(&r, "foreign_housing_taxpayer_expenses"), 50000);
        assert_eq!(value(&r, "foreign_housing_exclusion"), housing);
        assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
        assert_eq!(value(&r, "foreign_wages"), 200000);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(r.tax, Some(tax * 100));
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn foreign_housing_allowance_precedes_employer_funding_cap() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_pay(10000)));
    employee_housing(&mut d, 50000, ForeignHousingLocation::General);
    let ForeignHousing::Claim { nontaxable_allowances, .. } =
        &mut claim_mut(&mut d, Owner::Taxpayer).housing
    else {
        unreachable!()
    };
    *nontaxable_allowances = 500000;
    let r = supported(&d);
    // min(39,870 - 21,264 - 5,000, 10,000), not min(18,606, 10,000) - 5,000.
    assert_eq!(value(&r, "foreign_housing_exclusion"), 10000);
    assert_eq!(value(&r, "agi"), 0);
    assert_eq!(r.tax, Some(0));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let ForeignHousing::Claim { nontaxable_allowances, .. } =
        &mut claim_mut(&mut d, Owner::Taxpayer).housing
    else {
        unreachable!()
    };
    *nontaxable_allowances = -1;
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
}

#[test]
fn foreign_housing_employer_share_follows_form_2555_before_cap() {
    use crate::foreign_housing::employer_exclusion;
    // 18,606 housing cost, 200,000 employer amounts / 300,000 total foreign
    // earned income: 12,404 excluded; Part IX must address the remaining 6,202.
    assert_eq!(employer_exclusion(18606, 200000, 300000), 12404);
    assert_eq!(employer_exclusion(18606, 100000, 300000), 6202);
    assert_eq!(employer_exclusion(18606, 300000, 300000), 18606);
    assert_eq!(employer_exclusion(18606, 0, 300000), 0);
    assert_eq!(employer_exclusion(18606, 0, 0), 0);
    // High housing amount still cannot exclude more than employer funding.
    assert_eq!(employer_exclusion(45000, 10000, 20000), 10000);
    // At-least-three-decimal instruction permits the exact ratio. Truncating
    // or rounding 1/3 to .333 first would incorrectly give 33,300.
    assert_eq!(employer_exclusion(100000, 200000, 600000), 33333);
    assert_eq!(employer_exclusion(-1, 200000, 300000), 0);
}

#[test]
fn fully_excluded_foreign_earnings_carry_entire_current_housing_amount() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(excluded_foreign_business(100000, 0)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    self_employed_housing(&mut d, Some(HousingCarryover { amount: 1800000, deducted: 800000 }));
    let r = supported(&d);
    // 26 CFR 1.911-4(f), Example 6: no income remains after FEIE,
    // but the entire current non-employer housing amount carries one year.
    assert_eq!(value(&r, "foreign_housing_deduction"), 0);
    assert_eq!(value(&r, "foreign_housing_taxpayer_deduction_carryforward"), 18606);
    assert_eq!(value(&r, "foreign_housing_taxpayer_prior_deduction_expired"), 10000);
    assert_eq!(value(&r, "agi"), 0);
    assert_eq!(r.tax, Some(1413000));
    assert_eq!(supported(&restart(d).draft), r);
}

// T2-26: Form 1116 line 1a qualified dividend adjustment and the Worksheet for
// Line 18 (i1116 2025, "Foreign Qualified Dividends and Capital Gains" and
// line 18; i6251 line 8 steps 2 and 4; 26 CFR 1.904(b)-1). Whole dollars.
pub(in crate::tests) fn fund_dividends(
    ordinary: i64,
    qualified: i64,
    capital: i64,
) -> FederalIncome {
    FederalIncome::Dividend {
        data: native_dividend(ordinary * 100, qualified * 100, capital * 100),
    }
}
pub(in crate::tests) fn fund_claim(
    income: i64,
    qualified: i64,
    tax: i64,
    rate_adjustment: ForeignRateAdjustment,
) -> ForeignTaxClaim {
    ForeignTaxClaim::Ordinary {
        history: ForeignCreditHistory::None,
        carryback: ForeignCarryback::Unreviewed,
        interest_allocation: ForeignInterestAllocation::GrossIncome,
        investment_assets: ForeignInvestmentAssets::Unreviewed,
        loss_history: ForeignLossHistory::None,
        state_taxes: ForeignStateTaxes::Unreviewed,
        amt_limitation: ForeignAmtLimitation::Normal,
        sources: vec![ForeignOrdinaryTax {
            country: "ZZ".into(),
            owner: Owner::Taxpayer,
            label: "RIC international fund".into(),
            category: ForeignTaxCategory::Passive {
                sales: vec![],
                withholding: ForeignWithholding::Below15,
                qualified: qualified * 100,
            },
            income: income * 100,
            tax: tax * 100,
        }],
        basis: ForeignTaxBasis::Paid,
        situations: vec![],
        rate_adjustment,
    }
}
fn fund_return(wages: i64, dividends: FederalIncome, claim: ForeignTaxClaim) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
    add(&mut d, income(dividends));
    d.credits.as_mut().unwrap().foreign_tax = claim;
    d
}

#[test]
fn form_1116_qualified_dividends_and_capital_gain_distributions_match_hand_computation() {
    // Single; W-2 $150,000; one 1099-DIV: box 1a $60,000, box 1b $50,000, box 2a
    // $10,000. The fund reports $30,000 foreign passive income, $24,000 of it
    // qualified, and $3,000 foreign tax (below 15% withholding).
    let d = fund_return(
        150_000,
        fund_dividends(60_000, 50_000, 10_000),
        fund_claim(30_000, 24_000, 3_000, ForeignRateAdjustment::Exception),
    );
    let r = supported(&d);
    // AGI 150,000 + 60,000 + 10,000 = 220,000; standard deduction 16,100;
    // Form 1040 line 15 = 203,900.
    assert_eq!(value(&r, "agi"), 220_000);
    assert_eq!(value(&r, "taxable_income"), 203_900);
    // QDCG worksheet: line 4 = 60,000; line 5 = 143,900; line 9 = 0 (line 5
    // fills the 49,450 zero band); line 17 = 60,000 at 15%; line 20 = 0.
    // Line 22 on 143,900: 1,240 + 4,560 + 12,166 + 24% of 38,200 (9,168) =
    // 27,134; line 23 = 27,134 + 9,000 = 36,134 < line 24 41,704 (tax on
    // 203,900), so the adjustment is required. Foreign qualified dividends
    // 24,000 are not below $20,000: no adjustment exception.
    assert_eq!(value(&r, "regular_income_tax"), 36_134);
    // Line 1a: 24,000 at 15% kept at 0.4054 = 9,730; reduction 14,270.
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 14_270);
    // Line 3d/3e use unadjusted gross: 16,100 x 30,000 / 220,000 = 2,195.
    // Line 15 = 30,000 - 14,270 - 2,195 = 13,535 (high-tax test on the
    // unadjusted 27,805: 3,000 is below 37%).
    // Line 18 worksheet: 203,900 - 0.5946 x 60,000 (35,676) = 168,224.
    assert_eq!(value(&r, "foreign_line_18_rate_adjustment"), 35_676);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 168_224);
    // Line 21 = 36,134 x 13,535 / 168,224 = 2,907.28 -> 2,907; credit is the
    // smaller of 3,000 and 2,907; 93 unused.
    assert_eq!(value(&r, "foreign_tax_credit"), 2_907);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 93);
    // AMT: AMTI 220,000, exemption 90,100, line 6 = 129,900; line 17 =
    // 69,900; line 30 = 60,000 at 15%; line 38 = 18,174 + 9,000 = 27,174 <
    // line 39 33,774. AMT line 1a keeps 24,000 x 0.5357 = 12,857: reduction
    // 11,143, AMT line 17 = 30,000 - 11,143 = 18,857 (no AMT deductions to
    // apportion). AMT line 18 = 220,000 - 0.4643 x 60,000 (27,858) = 192,142.
    assert_eq!(value(&r, "foreign_amt_qualified_dividend_adjustment"), 11_143);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 18_857);
    assert_eq!(value(&r, "foreign_amt_line_18_rate_adjustment"), 27_858);
    assert_eq!(form_1116_any(&r, true, "_worldwide_income"), 192_142);
    // AMTFTC = 27,174 x 18,857 / 192,142 = 2,666.88 -> 2,667; 333 unused. AMT
    // 27,174 - 2,667 = 24,507 is below 36,134 - 2,907: no AMT.
    assert_eq!(value(&r, "foreign_passive_amt_unused_tax"), 333);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn form_1116_splits_foreign_qualified_dividends_ratably_across_zero_and_fifteen_percent() {
    // W-2 $50,000; box 1a $40,000, box 1b $30,000. Foreign $30,000 with
    // $24,000 qualified and $2,000 tax. AGI 90,000; line 15 = 73,900.
    let d = fund_return(
        50_000,
        fund_dividends(40_000, 30_000, 0),
        fund_claim(30_000, 24_000, 2_000, ForeignRateAdjustment::Exception),
    );
    let r = supported(&d);
    // QDCG: line 5 = 43,900; line 7 = 49,450; line 9 = 5,550 at 0%; line 17 =
    // 24,450 at 15%. Line 22 table tax on 43,900 = 5,023; line 23 = 5,023 +
    // 3,667.50 = 8,690.50 -> 8,691 (below line 24).
    assert_eq!(value(&r, "regular_income_tax"), 8_691);
    // Ratable split of 24,000: 15% share 24,000 x 24,450 / 30,000 = 19,560
    // kept at 0.4054 = 7,930; the 0% share 4,440 is left out. Reduction 16,070.
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 16_070);
    // Line 18 = 73,900 - (5,550 + 0.5946 x 24,450 = 14,538) = 53,812.
    assert_eq!(value(&r, "foreign_line_18_rate_adjustment"), 20_088);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 53_812);
    // Line 15 = 30,000 - 16,070 - 16,100 x 30,000 / 90,000 (5,367) = 8,563.
    // Limit 8,691 x 8,563 / 53,812 = 1,382.98 -> 1,383; 617 unused.
    assert_eq!(value(&r, "foreign_tax_credit"), 1_383);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 617);
    // AMTI 90,000 is inside the 90,100 exemption: nothing to adjust for AMT.
    assert_eq!(value(&r, "foreign_amt_qualified_dividend_adjustment"), 0);
}

#[test]
fn form_1116_adjustment_exception_ends_at_20000_of_foreign_qualified_dividends() {
    // W-2 $150,000; box 1a $30,000, box 1b $25,000; foreign $25,000 and $2,000
    // tax. Line 15 = 163,900; QDCG line 5 = 138,900, within the 24% bracket
    // (201,775), and all 25,000 is taxed at 15%.
    let at = |qualified: i64, choice: ForeignRateAdjustment| {
        supported(&fund_return(
            150_000,
            fund_dividends(30_000, 25_000, 0),
            fund_claim(25_000, qualified, 2_000, choice),
        ))
    };
    // $19,999 is below $20,000: the presumed exception leaves line 1a and
    // line 18 unadjusted.
    let r = at(19_999, ForeignRateAdjustment::Exception);
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 0);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 163_900);
    // Choosing to adjust anyway: 19,999 x 0.4054 = 8,108; reduction 11,891;
    // line 18 = 163,900 - 0.5946 x 25,000 (14,865) = 149,035.
    let r = at(19_999, ForeignRateAdjustment::Adjust);
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 11_891);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 149_035);
    // $20,000 is not below $20,000: 20,000 - 8,108 = 11,892.
    let r = at(20_000, ForeignRateAdjustment::Exception);
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 11_892);
    assert_eq!(value(&r, "foreign_line_18_rate_adjustment"), 14_865);
}

#[test]
fn form_1116_adjustment_exception_ends_at_the_top_of_the_24_percent_bracket() {
    // Box 1a $12,000, box 1b $9,000; foreign $12,000 with $9,000 qualified and
    // $1,000 tax. QDCG line 5 = wages + 12,000 - 16,100 - 9,000.
    let at = |wages: i64| {
        supported(&fund_return(
            wages,
            fund_dividends(12_000, 9_000, 0),
            fund_claim(12_000, 9_000, 1_000, ForeignRateAdjustment::Exception),
        ))
    };
    // Wages 214,875: line 5 = 201,775, not above the 2026 single 24% bracket.
    let r = at(214_875);
    assert_eq!(value(&r, "taxable_income"), 210_775);
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 0);
    assert_eq!(value(&r, "foreign_amt_qualified_dividend_adjustment"), 0);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 210_775);
    // One dollar more: line 5 = 201,776. Line 17 = 9,000 at 15%: 9,000 x
    // 0.4054 = 3,649, reduction 5,351; line 18 = 210,776 - 5,351 = 205,425.
    // AMT: line 6 = 226,876 - 90,100 = 136,776; line 17 = 127,776; the exception
    // fails with the regular one, so 9,000 x 0.5357 = 4,821, reduction 4,179.
    let r = at(214_876);
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 5_351);
    assert_eq!(value(&r, "foreign_line_18_rate_adjustment"), 5_351);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 205_425);
    assert_eq!(value(&r, "foreign_amt_qualified_dividend_adjustment"), 4_179);
}

#[test]
fn form_1116_no_adjustment_when_qdcg_line_5_is_zero() {
    // W-2 $10,000; box 1a and 1b $50,000. Line 15 = 43,900 is below the
    // 50,000 of qualified dividends, so QDCG line 5 is zero and no adjustment
    // is required even when chosen; line 18 stays taxable income.
    let d = fund_return(
        10_000,
        fund_dividends(50_000, 50_000, 0),
        fund_claim(25_000, 25_000, 1_000, ForeignRateAdjustment::Adjust),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_income"), 43_900);
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 0);
    assert_eq!(value(&r, "foreign_line_18_rate_adjustment"), 0);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 43_900);
    // Everything is in the 0% band: no regular tax, so no credit.
    assert_eq!(value(&r, "regular_income_tax"), 0);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
}

#[test]
fn form_1116_refuses_schedule_d_gains_and_foreign_qualified_above_the_return() {
    let claim = fund_claim(12_000, 9_000, 1_000, ForeignRateAdjustment::Exception);
    // A sale whose AMT basis differs needs its own AMT line 18 worksheet.
    let mut d = fund_return(100_000, fund_dividends(12_000, 9_000, 5_000), claim.clone());
    add(
        &mut d,
        income(sale_with_amt(ordinary_investment_sale(9_000, 6_000, GainTerm::Long), 5_000, 0)),
    );
    supported(&d);
    // A Form 2439 undistributed gain is a Schedule D source: gross income.
    let mut d = fund_return(100_000, fund_dividends(12_000, 9_000, 5_000), claim.clone());
    add(&mut d, income(undistributed_gain()));
    supported(&d);
    // A Form 4952 line 4g election is assumed to come from U.S. items first.
    let mut d = fund_return(100_000, fund_dividends(12_000, 9_000, 5_000), claim.clone());
    investment_borrowing(&mut d, 2000, 0, 0);
    d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::CapitalFirst { amount: 100_000 };
    assert!(noted(&supported(&d), "WARN_FTC_4952_ELECTION_ASSUMED"));
    // Foreign qualified dividends above line 3a do not reconcile.
    let d = fund_return(100_000, fund_dividends(12_000, 8_000, 0), claim);
    unsupported(&d, "Foreign tax credit");
    // The special-situations confirmation's No claims no credit, with a notice.
    let mut d = fund_return(
        100_000,
        fund_dividends(12_000, 9_000, 0),
        fund_claim(12_000, 9_000, 1_000, ForeignRateAdjustment::Exception),
    );
    if let ForeignTaxClaim::Ordinary { situations, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *situations = vec![crate::ForeignTaxSituation::OilAndGas.code()];
    }
    let r = supported(&d);
    assert!(noted(&r, "WARN_FOREIGN_SITUATION_LEFT_OUT"));
    assert!(!r.lines.iter().any(|l| l.id == "foreign_tax_credit" && l.amount > 0));
}

// T2-26b: Form 1116 with U.S.-source-only Schedule D returns (i1116 2025,
// "Schedule D Filers", lines 3d/3e and the Worksheet for Line 18 with the
// Schedule D Tax Worksheet; i6251 line 8 steps 2 and 4; i1040sd 2025 Schedule D
// Tax Worksheet and Capital Loss Carryover Worksheet). No sale or gain is
// foreign source, so line 1a holds no gain and Worksheets A and B do not apply.
// 2026 single: 24% bracket ends at 201,775, 0% gain band 49,450, 15% band
// 545,500, standard deduction 16,100, AMT exemption 90,100 and 26% to 244,500.

#[test]
fn form_1116_takes_form_6781_and_earlier_form_6252_gains_together() {
    // Single; W-2 $60,000 and the international fund ($10,000 foreign passive,
    // none qualified, $2,000 tax). Form 6781 box 11 of $10,000 (line 8 40% =
    // 4,000 short-term, line 9 60% = 6,000 long-term) and an earlier-year
    // Form 6252 sale: gross profit 10,000 on a 20,000 contract price, 8,000
    // received in 2026, so 8,000 x 50% = 4,000 long-term gain. Schedule D:
    // short 4,000, long 10,000; net gain 14,000 is in AGI = 60,000 + 10,000 +
    // 14,000 = 84,000; line 3e = 84,000 (both gains are gross gains).
    // Taxable 84,000 - 16,100 = 67,900. QDCG worksheet: 10,000 of net long-term
    // gain at 15% (the ordinary 57,900 is above the 49,450 zero band) = 1,500;
    // ordinary 57,900: Tax Table midpoint 57,925, 1,240 + 4,560 + 22% x 7,525
    // = 7,455.50 -> 7,456; tax 8,956. Line 3a 16,100 x 10,000 / 84,000 =
    // 1,916.67 -> 1,917; line 17 = 8,083; line 21 = 8,956 x 8,083 / 67,900 =
    // 1,066.2 -> 1,066 (2,000 - 1,066 = 934 unused).
    let mut d = passive_fund_return(60_000);
    add(&mut d, section_1256_entry(1_000_000));
    d.screening.as_mut().unwrap().installment_payments.push(crate::EarlierInstallment {
        owner: Owner::Taxpayer,
        sale: crate::InstallmentPayments {
            property: "Sold property".into(),
            sold: 20230601,
            gross_profit: 1_000_000,
            contract_price: 2_000_000,
            earlier_payments: 0,
            payments: 800_000,
            kind: crate::PropertyCharacter::LongTermCapital,
            special: false,
            qbi: crate::InstallmentQbi::NotQualified,
        },
    });
    let r = supported(&d);
    assert!(r.issues.iter().all(|i| i.section != "Form 1116 capital gains"), "{:?}", r.issues);
    assert_eq!(value(&r, "agi"), 84_000);
    assert_eq!(value(&r, "taxable_income"), 67_900);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 84_000);
    assert_eq!(value(&r, "regular_income_tax"), 8_956);
    assert_eq!(value(&r, "foreign_tax_credit"), 1_066);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 934);
}

#[test]
fn form_1116_takes_an_earlier_related_party_exchange_gain() {
    // Single; W-2 $60,000 and the international fund ($10,000 foreign passive,
    // $2,000 tax). An earlier-year related-party exchange (Form 8824 line 24)
    // recognizes $90,000 of long-term capital gain in 2026 (disposed April 30,
    // inside the two-year period). AGI = 60,000 + 10,000 + 90,000 = 160,000 =
    // line 3e. Taxable 160,000 - 16,100 = 143,900. QDCG worksheet: 90,000 at
    // 15% = 13,500 (the ordinary 53,900 is above the 49,450 zero band);
    // ordinary 53,900: Tax Table midpoint 53,925, 1,240 + 4,560 + 22% x 3,525 =
    // 6,575.50 -> 6,576; tax 20,076. Line 3a 16,100 x 10,000 / 160,000 =
    // 1,006.25 -> 1,006; line 17 = 8,994; line 21 = 20,076 x 8,994 / 143,900 =
    // 1,254.8 -> 1,255 (2,000 - 1,255 = 745 unused).
    let mut d = passive_fund_return(60_000);
    d.screening.as_mut().unwrap().section1231 =
        Some(Section1231History { regular: vec![], amt: vec![] });
    d.screening.as_mut().unwrap().related_exchanges = vec![crate::EarlierRelatedExchange {
        owner: Owner::Taxpayer,
        exchange: crate::RelatedExchange {
            property: "12 Elm St".into(),
            last_transfer: 20240501,
            disposed: 20260430,
            exception: crate::RelatedException::None,
            deferred: 9_000_000,
            kind: crate::PropertyCharacter::LongTermCapital,
            special: false,
        },
    }];
    let r = supported(&d);
    assert!(r.issues.iter().all(|i| i.section != "Form 1116 capital gains"), "{:?}", r.issues);
    assert_eq!(value(&r, "agi"), 160_000);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 160_000);
    assert_eq!(value(&r, "regular_income_tax"), 20_076);
    assert_eq!(value(&r, "foreign_tax_credit"), 1_255);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 745);
}

#[test]
fn form_1116_schedule_d_return_with_25_and_28_percent_gain_matches_hand_computation() {
    // Single; W-2 $300,000. 1099-DIV: box 1a $20,000, box 1b $16,000, box 2a
    // $6,000 of which box 2b $2,000 and box 2d $1,000. Sales (Form 8949):
    // long-term stock +12,000 (30,000 - 18,000), long-term collectible coins
    // +3,000 (9,000 - 6,000), short-term stock -1,000 (5,000 - 6,000). The
    // international fund reports $10,000 foreign passive income, $8,000 of it
    // qualified, with $1,400 foreign tax.
    let mut dividends = fund_dividends(20_000, 16_000, 6_000);
    if let FederalIncome::Dividend { data } = &mut dividends {
        data.unrecaptured_distributions = 200_000;
        data.collectible_distributions = 100_000;
    }
    let mut d = fund_return(
        300_000,
        dividends,
        fund_claim(10_000, 8_000, 1_400, ForeignRateAdjustment::Exception),
    );
    add(&mut d, income(ordinary_investment_sale(30_000, 18_000, GainTerm::Long)));
    add(&mut d, income(collectible_sale(9_000, 6_000, GainTerm::Long, false)));
    add(&mut d, income(ordinary_investment_sale(5_000, 6_000, GainTerm::Short)));
    let r = supported(&d);
    // Schedule D: line 7 = -1,000; line 15 = 12,000 + 3,000 + 6,000 = 21,000;
    // line 16 = 20,000. 28% Rate Gain Worksheet: 3,000 + 1,000 - 1,000 short
    // loss = 3,000 (line 18). Unrecaptured 1250 gain 2,000 (line 19).
    // AGI = 300,000 + 20,000 + 20,000 = 340,000; line 15 = 323,900.
    assert_eq!(value(&r, "agi"), 340_000);
    assert_eq!(value(&r, "taxable_income"), 323_900);
    // Schedule D Tax Worksheet: line 10 = 16,000 + 20,000 = 36,000; line 12 =
    // min(20,000, 5,000) = 5,000; line 13 = 31,000; line 14 = 292,900; line 17
    // = 49,450, line 22 = 0; line 18 = 287,900; line 20 = 201,775; line 21 =
    // 287,900; line 23 = 31,000; line 29 = 323,900 - 287,900 = 36,000; line 30
    // = 31,000; line 33 = 0; line 35 = 2,000; line 38 = 36,000 + 287,900 -
    // 323,900 = 0; line 39 = 2,000; line 42 = 323,900 - 320,900 = 3,000.
    // Line 44 tax on 287,900 = 1,240 + 4,560 + 12,166 + 23,058 + 17,424 +
    // 11,086.25 = 69,534.25; line 45 = 69,534.25 + 4,650 + 500 + 840 =
    // 75,524.25 < line 46 82,134.25 (tax on 323,900): regular tax 75,524.
    assert_eq!(value(&r, "regular_income_tax"), 75_524);
    // Line 18 (287,900) is above 201,775: no adjustment exception, and the
    // worksheet is required (line 18 > 0, line 45 < line 46).
    // Line 1a: the 8,000 foreign qualified dividends are all in the 15% band
    // (bands 0 / 31,000 / 0): kept at 8,000 x 0.4054 = 3,243; reduction 4,757.
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 4_757);
    // Line 3e counts gains before losses: 300,000 + 20,000 + 12,000 + 3,000 +
    // 6,000 = 341,000 (the 1,000 short-term loss is not subtracted).
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 341_000);
    // Line 3a/3g: 16,100 x 10,000 / 341,000 = 472.14 -> 472. Line 15 = 10,000
    // - 4,757 - 472 = 4,771.
    // Worksheet for Line 18: line 3 = 3,000 x 0.2432 = 730; line 5 = 2,000 x
    // 0.3243 = 649; line 7 = 0; line 9 = 31,000 x 0.5946 = 18,433; line 10 =
    // 0; line 11 = 19,812; line 12 = 323,900 - 19,812 = 304,088.
    assert_eq!(value(&r, "foreign_line_18_rate_adjustment"), 19_812);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 304_088);
    // Line 21 = 75,524 x 4,771 / 304,088 = 1,184.94 -> 1,185; 215 unused.
    assert_eq!(value(&r, "foreign_tax_credit"), 1_185);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 215);
    // AMT: line 4 = 340,000; line 6 = 249,900. Part III: line 13 = 31,000;
    // line 14 = 2,000; line 15 = 33,000; line 16 = 33,000; line 17 = 216,900
    // (26% = 56,394); line 20 = 292,900, line 21 = 0; line 22 = 31,000; line
    // 23 = 0; line 27 = 287,900, line 29 = 257,600; line 30 = 31,000 (4,650);
    // line 33 = 0; line 36 = 249,900 - 247,900 = 2,000 (500); line 38 =
    // 61,544 < line 39 65,082: the AMT worksheet is required.
    // AMT line 1a keeps 8,000 x 0.5357 = 4,286: reduction 3,714; AMT line 17
    // = 10,000 - 3,714 = 6,286 (no AMT deductions to apportion).
    assert_eq!(value(&r, "foreign_amt_qualified_dividend_adjustment"), 3_714);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 6_286);
    // AMT Worksheet for Line 18: line 5 = 2,000 x 0.1071 = 214; line 7 = 0;
    // line 9 = 31,000 x 0.4643 = 14,393; line 12 = 340,000 - 14,607 = 325,393.
    assert_eq!(value(&r, "foreign_amt_line_18_rate_adjustment"), 14_607);
    assert_eq!(form_1116_any(&r, true, "_worldwide_income"), 325_393);
    // AMTFTC = 61,544 x 6,286 / 325,393 = 1,188.92 -> 1,189; 211 unused.
    // 61,544 - 1,189 = 60,355 is below 75,524 - 1,185: no AMT.
    assert_eq!(value(&r, "foreign_passive_amt_unused_tax"), 211);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn form_1116_capital_loss_limit_counts_gross_gains_on_line_3e() {
    // Single; W-2 $100,000; box 1a $12,000, box 1b $9,000, box 2a $5,000; a
    // long-term sale +2,000 (7,000 - 5,000). 2025 carryover worksheet from
    // Schedule D line 7 (5,000) and line 15 (10,000), line 21 (3,000) and 2025
    // taxable income -2,000: line 3 = 1,000, line 4 = 1,000, short-term carryover
    // 5,000 - 1,000 = 4,000, long-term 10,000 - 0 = 10,000. Foreign $12,000
    // with $9,000 qualified and $1,000 tax; the filer chooses to adjust.
    let mut d = fund_return(
        100_000,
        fund_dividends(12_000, 9_000, 5_000),
        fund_claim(12_000, 9_000, 1_000, ForeignRateAdjustment::Adjust),
    );
    add(&mut d, income(ordinary_investment_sale(7_000, 5_000, GainTerm::Long)));
    d.screening.as_mut().unwrap().capital_history = vec![capital_history()];
    let r = supported(&d);
    // Schedule D line 7 = -4,000; line 15 = 5,000 + 2,000 - 10,000 = -3,000;
    // line 16 = -7,000, line 21 = -3,000 (the $3,000 limit). AGI = 100,000 +
    // 12,000 - 3,000 = 109,000; line 15 = 92,900. No net capital gain.
    assert_eq!(value(&r, "agi"), 109_000);
    assert_eq!(value(&r, "taxable_income"), 92_900);
    // Line 3e: 100,000 + 12,000 + 5,000 + 2,000 = 119,000; neither the current
    // loss limit nor the carryovers reduce gross income.
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 119_000);
    // QDCG: line 4 = 9,000; line 5 = 83,900 (within 201,775, but Adjust is
    // chosen); line 9 = 0; line 17 = 9,000. Line 22 table tax on 83,900 =
    // 13,176; line 23 = 13,176 + 1,350 = 14,526 < line 24 15,156.
    assert_eq!(value(&r, "regular_income_tax"), 14_526);
    // Line 1a: 9,000 x 0.4054 = 3,649; reduction 5,351. Line 3a/3g: 16,100 x
    // 12,000 / 119,000 = 1,623.53 -> 1,624. Line 15 = 12,000 - 5,351 - 1,624 =
    // 5,025. Line 18 = 92,900 - 9,000 x 0.5946 (5,351) = 87,549.
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 5_351);
    assert_eq!(value(&r, "foreign_line_18_rate_adjustment"), 5_351);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 87_549);
    // Line 21 = 14,526 x 5,025 / 87,549 = 833.75 -> 834; 166 unused.
    assert_eq!(value(&r, "foreign_tax_credit"), 834);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 166);
}

#[test]
fn form_1116_unrecaptured_gain_enters_the_25_percent_group_only_above_the_24_percent_bracket() {
    // Box 1a and 1b $10,000; box 2a $10,000 with box 2b $4,000. Foreign
    // $10,000, all qualified, $500 tax, Adjust chosen. Schedule D Tax Worksheet
    // line 10 = 20,000, line 13 = 16,000; line 1 = wages + 3,900; line 14 =
    // wages - 12,100.
    let at = |wages: i64| {
        let mut dividends = fund_dividends(10_000, 10_000, 10_000);
        if let FederalIncome::Dividend { data } = &mut dividends {
            data.unrecaptured_distributions = 400_000;
        }
        supported(&fund_return(
            wages,
            dividends,
            fund_claim(10_000, 10_000, 500, ForeignRateAdjustment::Adjust),
        ))
    };
    // Wages 213,875: line 14 = 201,775, line 21 = 201,775, line 30 = 16,000,
    // line 38 = 20,000 + 201,775 - 217,775 = 4,000, line 39 = 0: the 1250 gain
    // is taxed within line 21. Worksheet: 16,000 x 0.5946 = 9,514.
    let r = at(213_875);
    assert_eq!(value(&r, "taxable_income"), 217_775);
    assert_eq!(value(&r, "foreign_line_18_rate_adjustment"), 9_514);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 208_261);
    // Line 1a: 10,000 in the 15% band, kept at 4,054; reduction 5,946.
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 5_946);
    // Wages 214,875: line 14 = 202,775, line 21 stays 201,775, line 38 =
    // 3,000, line 39 = 1,000 at 25%: 9,514 + 1,000 x 0.3243 (324) = 9,838.
    let r = at(214_875);
    assert_eq!(value(&r, "foreign_line_18_rate_adjustment"), 9_838);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 208_937);
    assert_eq!(value(&r, "foreign_qualified_dividend_adjustment"), 5_946);
}

#[test]
fn form_1116_short_term_loss_first_offsets_the_28_percent_group() {
    // W-2 $300,000; box 1a and 1b $10,000; a long-term collectible +3,000
    // (9,000 - 6,000) and a short-term loss. Foreign $10,000, all qualified,
    // $1,000 tax; line 18 is above 201,775, so no exception.
    let at = |loss: i64| {
        let mut d = fund_return(
            300_000,
            fund_dividends(10_000, 10_000, 0),
            fund_claim(10_000, 10_000, 1_000, ForeignRateAdjustment::Exception),
        );
        add(&mut d, income(collectible_sale(9_000, 6_000, GainTerm::Long, false)));
        add(&mut d, income(ordinary_investment_sale(5_000, 5_000 + loss, GainTerm::Short)));
        supported(&d)
    };
    // Loss 3,000: Schedule D line 16 = 0 and line 18 = 3,000 - 3,000 = 0. Line
    // 15 = 293,900; QDCG line 5 = 283,900; line 17 = 10,000 at 15%. Worksheet
    // 10,000 x 0.5946 = 5,946; line 18 = 287,954. Line 3e = 300,000 + 10,000
    // + 3,000 = 313,000.
    let r = at(3_000);
    assert_eq!(value(&r, "taxable_income"), 293_900);
    assert_eq!(value(&r, "foreign_line_18_rate_adjustment"), 5_946);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 287_954);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 313_000);
    // Loss 2,000: line 16 = 1,000 and line 18 = 1,000. Line 1 = 294,900; line
    // 10 = 11,000; line 13 = 10,000; line 21 = 283,900; line 30 = 10,000;
    // line 42 = 294,900 - 293,900 = 1,000 at 28%: 5,946 + 1,000 x 0.2432
    // (243) = 6,189; line 18 = 288,711. Line 3e is again 313,000.
    let r = at(2_000);
    assert_eq!(value(&r, "taxable_income"), 294_900);
    assert_eq!(value(&r, "foreign_line_18_rate_adjustment"), 6_189);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 288_711);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 313_000);
}

// T2-26c: every Form 1116 Part I fact pattern beyond wages and portfolio income
// is calculated or refused by name with its line (i1116 2025 lines 2 through
// 4b, 3d/3e and 18; Pub 514 "Deductions not definitely related" and "Interest
// expense"). Whole dollars. 2026 single: standard deduction 16,100; brackets
// 10% to 12,400, 12% to 50,400, 22% to 105,700, 24% to 201,775; Tax Table
// below $100,000 (tax on the $50 row midpoint, rounded); AMT exemption 90,100.
fn passive_claim(income: i64, tax: i64) -> ForeignTaxClaim {
    let mut claim = fund_claim(income, 0, tax, ForeignRateAdjustment::Exception);
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut claim {
        sources[0].category = ForeignTaxCategory::Passive {
            sales: vec![],
            withholding: ForeignWithholding::AtLeast15,
            qualified: 0,
        };
    }
    claim
}
// W-2 wages plus one international fund paying $10,000 of nonqualified foreign
// dividends with $2,000 of foreign tax.
fn section_1256_entry(box11: i64) -> FederalIncomeEntry {
    income(FederalIncome::Section1256 {
        data: crate::FederalSection1256 {
            aggregate: box11,
            withholding: 0,
            special_treatment: false,
        },
    })
}
pub(in crate::tests) fn passive_fund_return(wages: i64) -> FederalDraft {
    fund_return(wages, fund_dividends(10_000, 0, 0), passive_claim(10_000, 2_000))
}
fn set_interest_allocation(d: &mut FederalDraft, allocation: ForeignInterestAllocation) {
    if let ForeignTaxClaim::Ordinary { interest_allocation, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *interest_allocation = allocation;
    }
}
fn student_loan(d: &mut FederalDraft, interest: i64) {
    d.adjustments.as_mut().unwrap().student_loan = StudentLoanClaim {
        student_loan_interest: interest * 100,
        forms: Vec::new(),
        employer_paid_interest: 0,
    };
}

// T2-26d: the deduction for one-half of self-employment tax, the owner's SEP,
// SIMPLE or qualified plan deduction, the self-employed health insurance
// deduction, remitted jury pay and the early withdrawal penalty on U.S. savings
// are definitely related to U.S. income (Pub 514 "Definitely related": incurred
// in the activity, or in connection with the property, producing the income)
// and stay off Form 1116 entirely; line 3e still counts gross income before
// them. Other Schedule 1 Part II adjustments (IRA and the rest) stay on line 3b
// ("deductions that don't definitely relate to any specific type of income").
fn owner_sep_business(receipts: i64, expenses: i64, contribution: i64) -> FederalIncome {
    let mut business = cash_business(receipts);
    if let FederalIncome::Business { data } = &mut business {
        data.expenses = vec![ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Supplies".into() },
            expenses * 100,
        )];
        data.retirement = BusinessRetirement::Sep { contribution: contribution * 100, rate: 2500 };
    }
    business
}

#[test]
fn form_1116_adjustments_on_line_3b_and_qbi_below_line_18_match_hand_computation() {
    // W-2 100,000; U.S. bank interest 5,000 with a 1,000 early-withdrawal
    // penalty (Schedule 1 line 18); a REIT fund paying 20,000 of section 199A
    // dividends; the international fund (10,000 foreign passive, 2,000 tax).
    let mut d = passive_fund_return(100_000);
    add(&mut d, income(savings_income(5_000, 1_000)));
    add(&mut d, income(reit_income(20_000, 20_000, true)));
    let r = supported(&d);
    // Total income 135,000; AGI 134,000. QBI deduction 20% of 20,000 = 4,000
    // (below 20% of 117,900). Taxable income 134,000 - 16,100 - 4,000 = 113,900.
    assert_eq!(value(&r, "agi"), 134_000);
    assert_eq!(value(&r, "taxable_income"), 113_900);
    // Line 3e = AGI + the penalty = 135,000 (gross income before it). The
    // penalty relates to the U.S. savings interest: line 3b = 0 (T2-26c: 1,000).
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 135_000);
    assert_eq!(form_1116_any(&r, false, "_other_deductions"), 0);
    // Line 3a 16,100 x 10,000 / 135,000 = 1,192.59 -> 1,193 = line 3g = line 6
    // (T2-26c: 17,100 share 1,267); line 7 = 10,000 - 1,193 = 8,807 = line 15
    // (2,000 is below 37% of 8,807: passive); line 16 = 0; line 17 = 8,807
    // (T2-26c: 8,733). Line 18 = taxable income 113,900: the QBI deduction,
    // definitely related to U.S. income, is not apportioned.
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 113_900);
    // Line 20: tax on 113,900 = 1,240 + 4,560 + 12,166 + 24% x 8,200 (1,968)
    // = 19,934. Line 21: 19,934 x 8,807 / 113,900 = 1,541.34 -> 1,541
    // (T2-26c: 1,528); credit 1,541, 2,000 - 1,541 = 459 unused (T2-26c: 472).
    assert_eq!(value(&r, "regular_income_tax"), 19_934);
    assert_eq!(value(&r, "foreign_tax_credit"), 1_541);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 459);
    // AMT: AMTI = 134,000 - 4,000 QBI = 130,000; gross 130,000 + 4,000 +
    // 1,000 = 135,000; no AMT deduction to apportion (no standard deduction,
    // line 3b 0), so AMT line 17 = 10,000 (T2-26c: 9,926). Tentative tax 26%
    // x 39,900 = 10,374 < 19,934: no AMT.
    assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), 135_000);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 10_000);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn form_1116_us_schedule_c_enters_line_3e_at_gross_income() {
    // W-2 60,000; U.S. Schedule C receipts 50,000 with 10,000 of expenses (net
    // 40,000); the international fund.
    let mut d = passive_fund_return(60_000);
    let mut business = cash_business(50_000);
    if let FederalIncome::Business { data } = &mut business {
        data.expenses = vec![ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Supplies".into() },
            10_000 * 100,
        )];
    }
    add(&mut d, income(business));
    let r = supported(&d);
    // SE tax 40,000 x 0.9235 x 15.3% = 5,651.82; half 2,825.91 -> 2,826.
    // AGI 110,000 - 2,826 = 107,174. QBI 20% x (40,000 - 2,826) = 7,434.80 ->
    // 7,435; taxable 107,174 - 16,100 - 7,435 = 83,639.
    assert_eq!(value(&r, "agi"), 107_174);
    assert_eq!(value(&r, "taxable_income"), 83_639);
    // Line 3e = 60,000 wages + 50,000 Schedule C line 7 + 10,000 dividends:
    // the business expenses and the half SE tax deduction relate to U.S.
    // income and never enter Form 1116: line 3b = 0 (T2-26c: 2,826).
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 120_000);
    assert_eq!(form_1116_any(&r, false, "_other_deductions"), 0);
    // Line 3a 16,100 x 10,000 / 120,000 = 1,341.67 -> 1,342 = line 6 (T2-26c:
    // 1,577); line 7 = line 15 = line 17 = 8,658 (T2-26c: 8,423); line 16 = 0.
    // Line 20: Tax Table 83,639 (row midpoint 83,625): 1,240 + 4,560 + 22% x
    // 33,225 = 13,109.50 -> 13,110. Line 21: 13,110 x 8,658 / 83,639 =
    // 1,357.10 -> 1,357 (T2-26c: 1,320); 2,000 - 1,357 = 643 unused (T2-26c:
    // 680).
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 83_639);
    assert_eq!(value(&r, "regular_income_tax"), 13_110);
    assert_eq!(value(&r, "foreign_tax_credit"), 1_357);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 643);
    // AMT gross = AMTI 99,739 + 7,435 + 2,826 + 10,000 = 120,000; AMT line 3b
    // 0, so AMT line 17 = 10,000 (T2-26c: 9,764).
    assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), 120_000);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 10_000);
}

#[test]
fn form_1116_ira_deduction_stays_on_line_3b_beside_an_owner_sep_that_does_not() {
    // W-2 10,000; U.S. Schedule C receipts 50,000, expenses 10,000 (net
    // 40,000) with a 5,000 owner SEP contribution; a 7,000 traditional IRA
    // contribution; the international fund (10,000 foreign passive, 2,000 tax).
    let mut d = passive_fund_return(10_000);
    add(&mut d, income(owner_sep_business(50_000, 10_000, 5_000)));
    d.adjustments.as_mut().unwrap().iras = ira_facts(700_000, 0, Answer::Yes);
    let r = supported(&d);
    // Half SE tax 40,000 x 0.9235 x 15.3% / 2 = 2,825.91 -> 2,826. SEP limit
    // 20% x (40,000 - 2,825.91) = 7,434.82, so the 5,000 is deductible. IRA
    // MAGI 60,000 - 2,826 - 5,000 = 52,174: the SEP makes the owner covered,
    // but MAGI is below the 81,000 single phaseout start, so 7,000 is allowed.
    // AGI 52,174 - 7,000 = 45,174. QBI 20% x (40,000 - 2,826 - 5,000) =
    // 6,434.80, capped at 20% x (45,174 - 16,100) = 5,814.80 -> 5,815;
    // taxable 45,174 - 16,100 - 5,815 = 23,259.
    assert_eq!(value(&r, "ira_deduction"), 7_000);
    assert_eq!(value(&r, "agi"), 45_174);
    assert_eq!(value(&r, "taxable_income"), 23_259);
    // Line 3e = 10,000 + 50,000 gross + 10,000 = 70,000. Line 3b = the IRA
    // deduction 7,000 only; the half SE tax and SEP stay off Form 1116 (T2-26c
    // would have put 14,826 on line 3b).
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 70_000);
    assert_eq!(form_1116_any(&r, false, "_other_deductions"), 7_000);
    // Lines 3a + 3b = 23,100 x 10,000 / 70,000 = 3,300 = line 6; line 7 =
    // line 15 = line 17 = 6,700 (2,000 is below 37%: passive). Line 20: Tax
    // Table 23,259 (midpoint 23,275): 1,240 + 12% x 10,875 = 2,545. Line 21:
    // 2,545 x 6,700 / 23,259 = 733.10 -> 733; 1,267 unused.
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 23_259);
    assert_eq!(value(&r, "regular_income_tax"), 2_545);
    assert_eq!(value(&r, "foreign_tax_credit"), 733);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 1_267);
    // AMT: AMTI 45,174 - 5,815 = 39,359; gross 39,359 + 5,815 + 7,000 + 2,826
    // + 5,000 + 10,000 = 70,000; AMT line 3b 7,000 x 10,000 / 70,000 = 1,000;
    // AMT line 17 = 9,000.
    assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), 70_000);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 9_000);
}

#[test]
fn form_1116_refuses_a_savings_penalty_on_possibly_foreign_interest() {
    // Boundary: 10,000 of foreign passive income fits exactly within the fund's
    // 10,000 of dividends, so the penalized 5,000 savings account is U.S.
    // source and its penalty stays off Form 1116 (line 3b 0).
    let mut d = passive_fund_return(60_000);
    add(&mut d, income(savings_income(5_000, 100)));
    let r = supported(&d);
    assert_eq!(form_1116_any(&r, false, "_other_deductions"), 0);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 75_000);
    // 10,001 of foreign passive income must include the penalized account's
    // interest: the penalty is then definitely related to foreign passive
    // income (line 2 of the passive category), assumed with one note.
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        sources[0].income = 10_001 * 100;
    }
    let assumed = supported(&d);
    assert!(noted(&assumed, "WARN_FTC_SAVINGS_PENALTY_ASSUMED"));
    assert_eq!(form_1116_value(&assumed, "foreign_passive_ZZ_related_expenses"), 100);
    // Without a penalty the same foreign interest is ordinary passive income.
    if let Some(FederalIncome::Interest { data }) = d.income.last_mut().map(|e| &mut e.income) {
        data.early_penalty = 0;
    }
    supported(&d);
}

#[test]
fn form_1116_us_rental_enters_line_3e_at_gross_rents() {
    // W-2 80,000; one U.S. rental: rent 12,000, expenses 3,000, depreciation
    // (330,000 - 55,000 land) / 27.5 = 10,000; net loss 1,000 allowed under the
    // active-participation allowance; the international fund.
    let mut d = passive_fund_return(80_000);
    add(&mut d, income(FederalIncome::Rental { data: rental(12_000, 3_000) }));
    let r = supported(&d);
    // AGI 80,000 - 1,000 + 10,000 = 89,000; taxable 72,900.
    assert_eq!(value(&r, "agi"), 89_000);
    assert_eq!(value(&r, "taxable_income"), 72_900);
    // Line 3e = 80,000 + 12,000 gross rents + 10,000 = 102,000.
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 102_000);
    // Line 3a 16,100 x 10,000 / 102,000 = 1,578.43 -> 1,578; line 15 = 8,422.
    // Tax Table 72,900 (midpoint 72,925): 1,240 + 4,560 + 22% x 22,525 =
    // 10,755.50 -> 10,756. Limit 10,756 x 8,422 / 72,900 = 1,242.62 -> 1,243.
    assert_eq!(value(&r, "regular_income_tax"), 10_756);
    assert_eq!(value(&r, "foreign_tax_credit"), 1_243);
}

#[test]
fn form_1116_student_loan_interest_goes_to_us_income_under_the_5000_election() {
    // Boundary: gross foreign income exactly $5,000. W-2 60,000; the fund pays
    // 5,000 foreign (1,000 tax); student loan interest 2,000 (below the MAGI
    // phaseout).
    let mut d = fund_return(60_000, fund_dividends(5_000, 0, 0), passive_claim(5_000, 1_000));
    student_loan(&mut d, 2_000);
    set_interest_allocation(&mut d, ForeignInterestAllocation::SmallForeignIncome);
    let r = supported(&d);
    // AGI 63,000; taxable 46,900; line 3e = 63,000 + 2,000 = 65,000. Line 4b
    // is zero (all interest to U.S. income); line 3a 16,100 x 5,000 / 65,000 =
    // 1,238.46 -> 1,238; line 15 = 3,762. Tax Table 46,900 (midpoint 46,925):
    // 1,240 + 12% x 34,525 = 5,383. Limit 5,383 x 3,762 / 46,900 = 431.79 -> 432.
    assert_eq!(value(&r, "agi"), 63_000);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 65_000);
    assert_eq!(form_1116_any(&r, false, "_other_deductions"), 0);
    assert_eq!(value(&r, "regular_income_tax"), 5_383);
    assert_eq!(value(&r, "foreign_tax_credit"), 432);
}

#[test]
fn form_1116_small_foreign_interest_election_ends_above_5000() {
    // Boundary: $5,001 of gross foreign income cannot assign interest to U.S.
    // income; the asset method is then required, and without all-asset values
    // the loan interest is taken as relating to the foreign income, with a note.
    let mut d = fund_return(60_000, fund_dividends(5_001, 0, 0), passive_claim(5_001, 1_000));
    student_loan(&mut d, 2_000);
    set_interest_allocation(&mut d, ForeignInterestAllocation::SmallForeignIncome);
    unsupported(&d, "Foreign interest allocation");
    set_interest_allocation(&mut d, ForeignInterestAllocation::GrossIncome);
    let r = supported(&d);
    assert!(noted(&r, "WARN_FTC_PERSONAL_INTEREST_ASSUMED"));
    assert_eq!(form_1116_value(&r, "foreign_passive_ZZ_other_interest"), 2_000);
    // Passenger vehicle loan interest (Schedule 1-A) is line 4b interest too.
    let mut d = passive_fund_return(60_000);
    d.deductions.as_mut().unwrap().vehicle_loans = vec![FederalVehicleInterest {
        label: "Synthetic vehicle".into(),
        interest: 3_000 * 100,
        business: 0,
        original_use: true,
        assembled_in_us: true,
    }];
    let r = supported(&d);
    assert!(noted(&r, "WARN_FTC_PERSONAL_INTEREST_ASSUMED"));
    assert!(form_1116_value(&r, "foreign_passive_ZZ_other_interest") > 0);
}

#[test]
fn form_1116_tips_deduction_reduces_line_18_only_and_floors_at_zero() {
    // W-2 60,000 including 5,000 of qualified tips: line 18 = 70,000 - 16,100
    // - 5,000 = 48,900; line 3e stays 70,000 (no Part I share of the tips
    // deduction, which relates to U.S. pay).
    let mut d = passive_fund_return(60_000);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    data.work_deductions = work_deductions(5_000 * 100, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "tips_deduction"), 5_000);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 70_000);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 48_900);
    // Boundary: W-2 20,000, all qualified tips, and the fund: 30,000 - 16,100
    // - 20,000 is below zero, so line 18 is zero and no credit is allowed.
    let mut d = passive_fund_return(20_000);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
    data.work_deductions = work_deductions(20_000 * 100, 0);
    let r = supported(&d);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 0);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
}

#[test]
fn form_1116_tax_exempt_interest_stays_out_of_line_3e() {
    // Exempt interest is not gross income (i1116 lines 3d/3e): the credit is
    // the same with or without 5,000 of municipal bond interest.
    let d = passive_fund_return(60_000);
    let base = supported(&d);
    let mut d = passive_fund_return(60_000);
    let mut municipal = savings_income(0, 0);
    if let FederalIncome::Interest { data } = &mut municipal {
        data.exempt = 5_000 * 100;
    }
    add(&mut d, income(municipal.clone()));
    let r = supported(&d);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 70_000);
    assert_eq!(value(&r, "foreign_tax_credit"), value(&base, "foreign_tax_credit"));
    // Private activity bond interest is an AMT preference: AMT line 3e adds
    // it (an income-type adjustment) and the AMT record stays calculated.
    if let FederalIncome::Interest { data } = &mut municipal {
        data.private_activity = 5_000 * 100;
    }
    let mut d = passive_fund_return(60_000);
    add(&mut d, income(municipal));
    let r = supported(&d);
    assert_eq!(form_1116_any(&r, true, "_gross_income_all_sources"), 75_000);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 70_000);
}

#[test]
fn form_1116_calculates_each_pattern_it_once_refused() {
    let base = supported(&passive_fund_return(60_000));
    let base_3e = form_1116_any(&base, false, "_gross_income_all_sources");
    assert_eq!(base_3e, 70_000);
    let with = |extra: FederalIncomeEntry| {
        let mut d = passive_fund_return(60_000);
        add(&mut d, extra);
        d
    };
    let gross_3e =
        |d: &FederalDraft| form_1116_any(&supported(d), false, "_gross_income_all_sources");
    // A business run abroad that is not chosen on a source row stays U.S.
    // income: its Schedule C line 7 is on line 3e.
    assert_eq!(gross_3e(&with(income(foreign_service_business(20_000)))), base_3e + 20_000);
    // A Schedule F farm, a gambler's winnings, an S corporation K-1 without a
    // Schedule K-3 (its net amount, one note) and royalties enter line 3e at
    // their gross income.
    let farm = with(income(raised_farm(30_000, 10_000)));
    let farm_gross = crate::farm::sources(&farm).next().unwrap().gross() / 100;
    assert_eq!(gross_3e(&farm), base_3e + farm_gross);
    assert_eq!(gross_3e(&with(income(professional_gambling(30_000, 10_000, 0)))), base_3e + 30_000);
    let k1 = estimate(&with(income(FederalIncome::Scorp { data: nonpassive_scorp(10_000, 0) })));
    // (The S corporation's own AMT records are another page's refusal.)
    assert!(noted(&k1, "WARN_FTC_K1_GROSS_ASSUMED"));
    assert_eq!(gross_3e(&with(income(royalty(5_000, 1_000)))), base_3e + 5_000);
    // Itemized gambling losses are definitely related to the winnings (U.S.).
    let mut d = with(gambling(30_000, 30_000));
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Itemized { facts };
    supported(&d);
    let mut d = passive_fund_return(60_000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 20_000)]);
    unsupported(&d, "Regular loss history");
    // A Form 461 excess business loss (a 300,000 Schedule C loss is above the
    // 256,000 single limit): the add-back is not gross income.
    let r = supported(&with(income(cash_business(-300_000))));
    assert!(
        form_1116_any(&r, false, "_gross_income_all_sources")
            < value(&r, "agi") + 300_000 + base_3e
    );
}

#[test]
fn form_1116_moving_deduction_is_definitely_related_to_wages() {
    // W-2 100,000 and the international fund (10,000 foreign passive, 2,000 tax);
    // an Armed Forces move with goods of 1,800 and no code P (Schedule 1 line 14).
    // The deduction is incurred incident to the new post's wages (Pub 514
    // "Definitely related"), so it is not a line 3b "not definitely related"
    // deduction and line 3e adds it back: AGI 100,000 + 10,000 - 1,800 =
    // 108,200; line 3e = 108,200 + 1,800 = 110,000; line 3b = 0.
    // Line 3a 16,100 x 10,000 / 110,000 = 1,463.64 -> 1,464; line 17 = 10,000
    // - 1,464 = 8,536. Line 18 = 110,000 - 16,100 - 1,800 = 92,100. Tax Table
    // row midpoint 92,125: 1,240 + 4,560 + 22% x 41,725 = 14,979.50 -> 14,980.
    // Line 21: 14,980 x 8,536 / 92,100 = 1,388.4 -> 1,388 (2,000 - 1,388 = 612
    // unused). Without the add-back line 3e is 108,200 and line 3a is 1,488.
    let mut d = passive_fund_return(100_000);
    d.adjustments.as_mut().unwrap().moving.push(MovingClaim {
        owner: Owner::Taxpayer,
        record: MilitaryMove {
            reason: MoveReason::ArmedForcesOrders,
            goods: 180_000,
            trip: None,
            reimbursed: 0,
        },
    });
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 108_200);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 110_000);
    assert_eq!(form_1116_any(&r, false, "_other_deductions"), 0);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 92_100);
    assert_eq!(value(&r, "foreign_tax_credit"), 1_388);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 612);
}

// Schedule K-3 (unit k3). Synthetic fixtures worked by hand from the 2026
// Schedule K-3 drafts and instructions (Part II line 24 and Part III Section 4),
// Form 1116 and its instructions ("Reporting Foreign Tax Information From
// Partnerships and S Corporations") and the 2026 brackets (Rev. Proc. 2025-32:
// joint standard deduction $32,200, single $16,100).

pub(in crate::tests) fn k3_blank() -> ScheduleK3 {
    ScheduleK3 {
        countries: vec![],
        withholding: vec![],
        us_gross: 0,
        branch: K3CategoryAmounts::default(),
        passive: K3CategoryAmounts::default(),
        general: K3CategoryAmounts::default(),
        partner_deductions: 0,
        partner_interest: K3PartnerInterest {
            business: 0,
            passive_activity: 0,
            business_assets: K3Assets::default(),
            passive_assets: K3Assets::default(),
        },
        items: vec![],
    }
}
// A row of Part II line 24 with its Part III taxes, whole dollars.
pub(in crate::tests) fn k3_row(
    country: &str,
    category: K3Category,
    income: i64,
    qualified: i64,
    tax: i64,
) -> K3Country {
    K3Country {
        country: country.into(),
        category,
        income: income * 100,
        qualified: qualified * 100,
        tax: tax * 100,
    }
}
// A passive country's withholding answer (the person's own record).
pub(in crate::tests) fn k3_answer(country: &str, treatment: ForeignWithholding) -> K3Withholding {
    K3Withholding { country: country.into(), treatment }
}
fn ordinary_claim_without_sources() -> ForeignTaxClaim {
    let mut claim = fund_claim(0, 0, 0, ForeignRateAdjustment::Exception);
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut claim {
        sources.clear();
    }
    claim
}
// A limited partner with box 1 zero, $10,000 of basis and the given box 5, 6a
// and 21 amounts (whole dollars).
pub(in crate::tests) fn k3_partner(
    interest: i64,
    dividends: i64,
    foreign_taxes: i64,
    k3: Option<ScheduleK3>,
) -> PartnerK1 {
    k3_partner_with(0, false, interest, dividends, foreign_taxes, k3)
}
// The same with box 1 (whole dollars) and material participation.
fn k3_partner_with(
    ordinary: i64,
    active: bool,
    interest: i64,
    dividends: i64,
    foreign_taxes: i64,
    k3: Option<ScheduleK3>,
) -> PartnerK1 {
    let mut partner = ordinary_partner_source(ordinary, active);
    partner.separate = Some(PartnerSeparateItems {
        interest: interest * 100,
        dividends: dividends * 100,
        qualified_dividends: 0,
        royalties: 0,
        short: 0,
        long: 0,
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
        cash_charity: 0,
        backup_withholding: 0,
        exempt_interest: 0,
        foreign_taxes: foreign_taxes * 100,
    });
    partner.schedule_k3 = k3;
    partner
}
pub(in crate::tests) fn add_named(d: &mut FederalDraft, name: &str, value: FederalIncome) -> i64 {
    let mut entry = income(value);
    entry.label = name.into();
    add(d, entry);
    d.income.last().unwrap().id
}

// Fixture A. Joint; W-2 wages $150,000; Synthetic Global Fund LP, box 1 zero,
// box 5 $2,000, box 6a $10,000, box 21 $900; its Schedule K-3 shows Japan
// passive income $6,000 with $900 withheld (15%), U.S. gross income $6,000.
fn k3_fixture_a() -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 15_000_000, 0);
    let mut k3 = k3_blank();
    k3.countries = vec![k3_row("JA", K3Category::Passive, 6_000, 0, 900)];
    k3.withholding = vec![k3_answer("JA", ForeignWithholding::AtLeast15)];
    k3.us_gross = 600_000;
    add_named(
        &mut d,
        "Synthetic Global Fund LP",
        FederalIncome::Partnership { data: k3_partner(2_000, 10_000, 900, Some(k3)) },
    );
    d.credits.as_mut().unwrap().foreign_tax = ordinary_claim_without_sources();
    d
}

#[test]
fn k3_passive_tax_with_a_schedule_k3_matches_hand_computation() {
    let d = k3_fixture_a();
    let r = supported(&d);
    // AGI 150,000 + 2,000 + 10,000 = 162,000; taxable 129,800; tax 2,480 +
    // 9,120 + 22% x 29,000 = 17,980.
    assert_eq!(value(&r, "agi"), 162_000);
    assert_eq!(value(&r, "taxable_income"), 129_800);
    assert_eq!(value(&r, "regular_income_tax"), 17_980);
    // Line 3e: 162,000 + ((6,000 + 6,000) - (0 + 2,000 + 10,000)) = 162,000.
    assert_eq!(value(&r, "foreign_k3_gross_income_correction"), 0);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 162_000);
    // Group 1: line 3a 32,200 x 6,000 / 162,000 = 1,193; line 7 = 4,807; the
    // 900 tax is under 37% of it (1,778): no kickout. Line 18 = 129,800;
    // limit 17,980 x 4,807 / 129,800 = 665.9 -> 666; credit 666; Schedule B
    // line 6 carryover 900 - 666 = 234.
    assert_eq!(value(&r, "foreign_high_tax_net_income"), 0);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 129_800);
    assert_eq!(value(&r, "foreign_tax_credit"), 666);
    assert_eq!(value(&r, "foreign_passive_unused_tax"), 234);
    assert_eq!(value(&r, "foreign_general_unused_tax"), 0);
    assert_eq!(r.tax, Some(1_731_400));
    // The K-1 box 21 reduces the partner's basis and nothing else reads it.
    assert!(!noted(&r, "WARN_K1_FOREIGN_TAX_WITHOUT_K3"));
    assert_eq!(restart(d.clone()).draft, d);
}

// Fixture B. Single; W-2 wages $60,000; Synthetic Consulting Inc., box 1
// $80,000 with a $80,000 QBI statement, basis $50,000, box 16 code F $3,000;
// its Schedule K-3 shows Canada general income $20,000 with $3,000 of tax,
// U.S. gross income $100,000, general deductions $6,000 (line 26) and
// $8,000 of column (f) business interest (line 41) with line 6b assets of
// $300,000 U.S. and $100,000 general.
fn k3_scorp(k3: ScheduleK3) -> ScorpK1 {
    let mut source = complete_ordinary_scorp(80_000, true);
    source.opening = 5_000_000;
    source.loans.clear();
    source.foreign_taxes = 300_000;
    source.schedule_k3 = Some(k3);
    source.amt = Some(amt_records(&source));
    source
}
fn k3_fixture_b() -> (FederalDraft, i64) {
    let mut d = complete(FederalStatus::Single);
    let mut k3 = k3_blank();
    k3.countries = vec![k3_row("CA", K3Category::General, 20_000, 0, 3_000)];
    k3.us_gross = 10_000_000;
    k3.general.deductions = 600_000;
    k3.partner_interest.business = 800_000;
    k3.partner_interest.business_assets =
        K3Assets { us: 30_000_000, branch: 0, passive: 0, general: 10_000_000 };
    let id =
        add_named(&mut d, "Synthetic Consulting Inc.", FederalIncome::Scorp { data: k3_scorp(k3) });
    d.credits.as_mut().unwrap().foreign_tax = ordinary_claim_without_sources();
    (d, id)
}

#[test]
fn k3_general_category_with_apportioned_interest_matches_hand_computation() {
    let (d, _) = k3_fixture_b();
    let r = supported(&d);
    // AGI 140,000; taxable before QBI 123,900; QBI 16,000 (20% x 80,000 is
    // under the 24,780 limit); taxable 107,900; tax 1,240 + 4,560 + 12,166 +
    // 24% x 2,200 = 18,494.
    assert_eq!(value(&r, "agi"), 140_000);
    assert_eq!(value(&r, "taxable_income"), 107_900);
    assert_eq!(value(&r, "regular_income_tax"), 18_494);
    // Line 3e: 140,000 + (120,000 - 80,000) = 180,000.
    assert_eq!(value(&r, "foreign_k3_gross_income_correction"), 40_000);
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 180_000);
    // Line 2: 6,000; line 3g 16,100 x 20,000 / 180,000 = 1,789; line 4b
    // 8,000 x 100,000 / 400,000 = 2,000; line 7 = 20,000 - 9,789 = 10,211.
    assert_eq!(value(&r, "foreign_k3_definitely_related_deductions"), 6_000);
    assert_eq!(value(&r, "foreign_k3_interest_expense"), 2_000);
    // Line 18 = 107,900; limit 18,494 x 10,211 / 107,900 = 1,750; carryover
    // 3,000 - 1,750 = 1,250.
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 107_900);
    assert_eq!(value(&r, "foreign_tax_credit"), 1_750);
    assert_eq!(value(&r, "foreign_general_unused_tax"), 1_250);
    assert!(value(&r, "alternative_minimum_tax") == 0);
    // Form 7203 Part III line 46: 50,000 + 80,000 - 3,000 = 127,000 of basis.
    assert!(d.valid_stored());
}

// Fixture C. Single; W-2 wages $70,000; a Form 1099-DIV with box 1a $1,000 and
// box 7 foreign tax $120 (the payee-statement election, confirmed); Synthetic
// Income Partners, box 6a $4,000, box 21 `box21`, box 16 not checked.
fn k3_fixture_c(box21: i64, election: Option<bool>) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 7_000_000, 0);
    let mut dividend = fund_dividends(1_000, 0, 0);
    if let FederalIncome::Dividend { data } = &mut dividend {
        data.foreign_tax = 12_000;
    }
    add(&mut d, income(dividend));
    add_named(
        &mut d,
        "Synthetic Income Partners",
        FederalIncome::Partnership { data: k3_partner(0, 4_000, box21, None) },
    );
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Statements { eligible: election };
    d
}

#[test]
fn k1_foreign_tax_without_a_k3_is_never_credited() {
    // AGI 70,000 + 1,000 + 4,000 = 75,000; taxable 58,900; the Tax Table row
    // midpoint 58,925: 1,240 + 4,560 + 22% x 8,525 = 7,675.50 -> 7,676.
    let d = k3_fixture_c(150, Some(true));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 75_000);
    assert_eq!(value(&r, "regular_income_tax"), 7_676);
    // The election stands (120 + 150 = 270 is under $300) and credits only the
    // 1099 box 7 tax: the K-1 tax shows no income category.
    assert_eq!(value(&r, "foreign_tax_credit"), 120);
    assert_eq!(r.tax, Some(755_600));
    assert!(noted(&r, "WARN_K1_FOREIGN_TAX_WITHOUT_K3"), "{r:?}");
    let note =
        r.review_notes.iter().find(|n| n.display.key == "WARN_K1_FOREIGN_TAX_WITHOUT_K3").unwrap();
    assert!(note.english.contains("Synthetic Income Partners"), "{note:?}");
    // The $300 test counts the K-1 tax (section 904(j)(3)(B)-(C)): 120 + 200
    // is over the limit and the election is refused by name.
    let over = k3_fixture_c(200, Some(true));
    unsupported(&over, "Foreign tax credit");
    // Exactly $300 stands: 120 + 180.
    assert_eq!(value(&supported(&k3_fixture_c(180, Some(true))), "foreign_tax_credit"), 120);
    // An unconfirmed election claims nothing, with its own note; the K-1 note
    // stays.
    let r = supported(&k3_fixture_c(150, None));
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert!(noted(&r, "WARN_FOREIGN_TAX_CREDIT_NOT_CLAIMED"));
    assert!(noted(&r, "WARN_K1_FOREIGN_TAX_WITHOUT_K3"));
}

// Fixture A with the K-1 changed by `edit`, for the regression cases.
fn k3_fixture_a_with(edit: impl FnOnce(&mut PartnerK1)) -> FederalDraft {
    let mut d = k3_fixture_a();
    if let FederalIncome::Partnership { data } = &mut d.income[1].income {
        edit(data);
    }
    d
}
fn k3_of(d: &FederalDraft) -> ScheduleK3 {
    match &d.income[1].income {
        FederalIncome::Partnership { data } => data.schedule_k3.clone().unwrap(),
        FederalIncome::Scorp { data } => data.schedule_k3.clone().unwrap(),
        _ => unreachable!(),
    }
}

#[test]
fn k3_corporation_inclusions_and_disallowed_deductions_are_refused_by_name() {
    for item in [K3Item::ForeignCorporations, K3Item::DisallowedDeduction] {
        let mut k3 = k3_of(&k3_fixture_a());
        k3.items = vec![item];
        let d = k3_fixture_a_with(|p| p.schedule_k3 = Some(k3.clone()));
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
        assert!(
            r.issues.iter().any(|i| i.display.key == "ISSUE_PARTNER_K1_INTERNATIONAL"
                && i.message.contains("Schedule K-3 (Form 1065) Parts VI and VII")),
            "{:?}",
            r.issues
        );
        let (mut d, _) = k3_fixture_b();
        if let FederalIncome::Scorp { data } = &mut d.income[1].income {
            data.schedule_k3.as_mut().unwrap().items = vec![item];
        }
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
        assert!(
            r.issues.iter().any(|i| i.display.key == "ISSUE_SCORP_K1_INTERNATIONAL"
                && i.message.contains("Schedule K-3 (Form 1120-S) Parts V and VI")),
            "{:?}",
            r.issues
        );
    }
}

#[test]
fn k3_items_outside_the_calculation_drop_the_claim_with_one_note() {
    for item in [
        K3Item::OtherCategories,
        K3Item::ForeignRateGroups,
        K3Item::Research,
        K3Item::OilAndGas,
        K3Item::Loans,
    ] {
        let mut k3 = k3_of(&k3_fixture_a());
        k3.items = vec![item];
        // An Ordinary claim: the K-1 stays, no credit is claimed, one note.
        let d = k3_fixture_a_with(|p| p.schedule_k3 = Some(k3.clone()));
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 162_000);
        assert_eq!(value(&r, "foreign_tax_credit"), 0, "{item:?}");
        let count = |r: &FederalResult| {
            r.review_notes
                .iter()
                .filter(|n| n.display.key == "WARN_K3_FOREIGN_TAX_CREDIT_NOT_CLAIMED")
                .count()
        };
        assert_eq!(count(&r), 1, "{item:?}");
        assert!(!noted(&r, "WARN_K3_FOREIGN_TAX_NOT_CLAIMED"));
        assert!(
            r.review_notes
                .iter()
                .find(|n| n.display.key == "WARN_K3_FOREIGN_TAX_CREDIT_NOT_CLAIMED")
                .unwrap()
                .english
                .contains("Synthetic Global Fund LP's Schedule K-3")
        );
        // The payee-statement election too: nothing is claimed.
        let mut d = k3_fixture_a_with(|p| p.schedule_k3 = Some(k3.clone()));
        d.credits.as_mut().unwrap().foreign_tax =
            ForeignTaxClaim::Statements { eligible: Some(true) };
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_tax_credit"), 0);
        assert_eq!(count(&r), 1);
    }
    // No scope entry: the claim stands.
    assert_eq!(value(&supported(&k3_fixture_a()), "foreign_tax_credit"), 666);
}

#[test]
fn k3_taxes_with_no_claim_say_how_to_claim_them() {
    let mut d = k3_fixture_a();
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::No;
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_tax_credit"), 0);
    assert!(noted(&r, "WARN_K3_FOREIGN_TAX_NOT_CLAIMED"), "{r:?}");
    assert_eq!(r.tax, Some(1_798_000));
    // A K-3 with no tax has nothing to claim.
    let d = k3_fixture_a_with(|p| {
        let k3 = p.schedule_k3.as_mut().unwrap();
        k3.countries[0].tax = 0;
        k3.withholding[0].treatment = ForeignWithholding::None;
    });
    let mut d = d;
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::No;
    assert!(!noted(&supported(&d), "WARN_K3_FOREIGN_TAX_NOT_CLAIMED"));
}

#[test]
fn k3_passive_rows_group_by_withholding_and_reductions_floor_at_zero() {
    let group = |withholding, tax| {
        let mut k3 = k3_blank();
        k3.withholding = vec![k3_answer("JA", withholding)];
        k3.group(&k3_row("JA", K3Category::Passive, 1_000, 0, tax))
    };
    assert_eq!(group(ForeignWithholding::AtLeast15, 150), 1);
    assert_eq!(group(ForeignWithholding::Below15, 50), 2);
    assert_eq!(group(ForeignWithholding::None, 0), 3);
    assert_eq!(group(ForeignWithholding::None, 40), 4);
    assert_eq!(k3_blank().group(&k3_row("CA", K3Category::General, 1_000, 0, 40)), 0);
    // Two passive rows (taxes 900 and 100) and a reduction of 2,000 spread by
    // income 6,000 : 4,000 (1,200 and 800): the first row keeps 0 of 900 -> no,
    // 900 - 1,200 floors at zero and the second keeps 100 - 800 floored too.
    let mut k3 = k3_blank();
    k3.countries = vec![
        k3_row("JA", K3Category::Passive, 6_000, 0, 900),
        k3_row("UK", K3Category::Passive, 4_000, 0, 100),
    ];
    k3.withholding = vec![
        k3_answer("JA", ForeignWithholding::AtLeast15),
        k3_answer("UK", ForeignWithholding::Below15),
    ];
    k3.passive.reduction = 200_000;
    let credit = crate::schedule_k3::credit(&[&k3], false, &[]);
    assert_eq!(credit.tax, [0; 8]);
    assert_eq!(credit.reductions, 100_000);
    // A reduction below the taxes reduces each row by its income share:
    // 600 : 400 of 500 leaves 900 - 300 = 600 and 100 - 200 -> 0.
    k3.passive.reduction = 50_000;
    let credit = crate::schedule_k3::credit(&[&k3], false, &[]);
    assert_eq!(credit.tax[1], 60_000);
    assert_eq!(credit.tax[2], 0);
    assert_eq!(credit.reductions, 90_000 + 10_000 - 60_000);
    // Passive deductions without a passive row fall to the group with no tax.
    let mut k3 = k3_blank();
    k3.passive.deductions = 70_000;
    k3.general.interest = 30_000;
    let credit = crate::schedule_k3::credit(&[&k3], false, &[]);
    assert_eq!((credit.deductions[3], credit.interest[0]), (70_000, 30_000));
    // The small-foreign-income election assigns all interest to U.S. income.
    assert_eq!(crate::schedule_k3::credit(&[&k3], true, &[]).interest, [0; 8]);
}

#[test]
fn k3_withholding_rate_needs_a_tax() {
    // A positive withholding rate with no tax on the row is refused by name,
    // as for a Form 1116 source.
    let d = k3_fixture_a_with(|p| p.schedule_k3.as_mut().unwrap().countries[0].tax = 0);
    let r = estimate(&d);
    assert!(
        r.issues.iter().any(|i| i.display.key.starts_with("ISSUE_A_POSITIVE_WITHHOLDING_RATE")),
        "{:?}",
        r.issues
    );
}

#[test]
fn k3_box_21_above_the_remaining_basis_carries_and_limits_the_k3_taxes() {
    // Basis $10,000 + box 5 $2,000 + box 6a $10,000 = $22,000 absorbs box 21 up
    // to $22,000. One dollar more is a basis-only foreign tax carry (the basis
    // worksheet Part II foreign tax class, section 705(a)(2)(B)); the Schedule
    // K-3 is before that limit, so its foreign taxes are allowed in the ratio of
    // the foreign tax class the basis allows.
    let at = |box21: i64| {
        k3_fixture_a_with(|p| p.separate.as_mut().unwrap().foreign_taxes = box21 * 100)
    };
    assert_eq!(value(&supported(&at(22_000)), "foreign_tax_credit"), 666);
    let carried = |d: &FederalDraft| {
        let FederalIncome::Partnership { data } = &d.income[1].income else { unreachable!() };
        data.ledger(&data.regular).unwrap()
    };
    assert!(!carried(&at(22_000)).limited());
    let ledger = carried(&at(22_001));
    assert_eq!(
        crate::kernel::ratio::dollars(
            ledger.basis_suspended[crate::k1_loss::K1LossClass::ForeignTaxes].get()
        ),
        1
    );
    let r = supported(&at(22_001));
    assert!(value(&r, "foreign_tax_credit") <= 666);
    // The same with no 2025 at-risk history: the basis stage still runs and
    // carries (improvement row 241), and the claim is refused the same way.
    let unknown = |box21: i64| {
        let mut d = at(box21);
        if let FederalIncome::Partnership { data } = &mut d.income[1].income {
            for record in core::iter::once(&mut data.regular).chain(data.alternative.as_mut()) {
                record.risk.start = ScorpRiskStart::Unknown;
            }
        }
        d
    };
    assert_eq!(value(&supported(&unknown(22_000)), "foreign_tax_credit"), 666);
    assert_eq!(
        crate::kernel::ratio::dollars(
            carried(&unknown(22_001)).basis_suspended[crate::k1_loss::K1LossClass::ForeignTaxes]
                .get()
        ),
        1
    );
    supported(&unknown(22_001));
    // A released foreign tax carry: the 2026 K-3 shows no row for an earlier
    // year's tax, which is never credited from it.
    let released = {
        let mut d = at(22_000);
        if let FederalIncome::Partnership { data } = &mut d.income[1].income {
            data.regular.carried.foreign_taxes = 100;
            data.regular.basis.opening += 100;
            data.alternative = Some(data.regular.clone());
        }
        d
    };
    supported(&released);
}

#[test]
fn k3_interest_without_assets_follows_the_k3_income_unless_the_small_election_applies() {
    // B3: column (f) interest and no line 6b assets, no election.
    let (mut d, id) = k3_fixture_b();
    let strip = |d: &mut FederalDraft| {
        if let FederalIncome::Scorp { data } = &mut d.income[1].income {
            data.schedule_k3.as_mut().unwrap().partner_interest.business_assets =
                K3Assets::default();
        }
    };
    strip(&mut d);
    let assumed = supported(&d);
    assert!(noted(&assumed, "WARN_FTC_K3_INTEREST_ASSUMED"));
    // B2: the small-foreign-income election needs gross foreign income of
    // $5,000 or less; the K-3 row alone is $20,000.
    set_interest_allocation(&mut d, ForeignInterestAllocation::SmallForeignIncome);
    let _ = id;
    unsupported(&d, "Foreign interest allocation");
    // With the row at $4,000 the election holds and assigns the column (f)
    // interest (and the category interest) to U.S. income.
    if let FederalIncome::Scorp { data } = &mut d.income[1].income {
        let k3 = data.schedule_k3.as_mut().unwrap();
        k3.countries[0].income = 400_000;
        k3.general.deductions = 0;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_k3_interest_expense"), 0);
    assert_eq!(value(&r, "foreign_k3_definitely_related_deductions"), 0);
}

#[test]
fn k3_taxes_under_the_payee_statement_election() {
    // Joint, so the limit is $600. A passive-only K-3 with $500 of tax joins a
    // 1099-DIV box 7 tax of $100: both are payee statements and credit $600.
    let mut d = k3_fixture_a_with(|p| {
        p.schedule_k3.as_mut().unwrap().countries[0].tax = 50_000;
        p.separate.as_mut().unwrap().foreign_taxes = 50_000;
    });
    let mut dividend = fund_dividends(1_000, 0, 0);
    if let FederalIncome::Dividend { data } = &mut dividend {
        data.foreign_tax = 10_000;
    }
    add(&mut d, income(dividend));
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::Statements { eligible: Some(true) };
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_tax_credit"), 600);
    // $600.01 of tax is over the joint limit.
    if let FederalIncome::Dividend { data } = &mut d.income[2].income {
        data.foreign_tax += 1;
    }
    unsupported(&d, "Foreign tax credit");
    // A general row makes the election unavailable (the existing refusal).
    if let FederalIncome::Dividend { data } = &mut d.income[2].income {
        data.foreign_tax -= 1;
    }
    if let FederalIncome::Partnership { data } = &mut d.income[1].income {
        data.schedule_k3.as_mut().unwrap().countries[0].category = K3Category::General;
    }
    unsupported(&d, "Foreign tax credit");
}

#[test]
fn k3_limited_or_missing_k1_is_calculated_with_its_notes() {
    // A K-1 with a Schedule E Part II amount and no K-3 needs one (the claim
    // has a Form 1116 source of its own, so it is not dropped for lack of one).
    let mut d = k3_fixture_a_with(|p| {
        p.schedule_k3 = None;
        p.regular.ordinary = 500_000;
        p.alternative.as_mut().unwrap().ordinary = 500_000;
    });
    d.credits.as_mut().unwrap().foreign_tax = passive_claim(1_000, 120);
    // Its net amount stays the gross income on line 3e, with one note.
    let r = supported(&d);
    assert!(noted(&r, "WARN_FTC_K1_GROSS_ASSUMED"));
    // With its K-3 the same K-1 is calculated.
    let d = k3_fixture_a_with(|p| {
        p.regular.ordinary = 500_000;
        p.alternative.as_mut().unwrap().ordinary = 500_000;
    });
    assert_eq!(value(&supported(&d), "foreign_k3_gross_income_correction"), -5_000);
    // A loss its basis holds back is before the K-3's limits: the K-3's
    // deductions are allowed in the ratio of the loss the K-1 allows.
    let limited = k3_fixture_a_with(|p| {
        *p = k3_partner_with(-30_000, true, 2_000, 10_000, 0, p.schedule_k3.clone());
    });
    supported(&limited);
    // A loss its $22,000 of basis absorbs is not limited, and the correction
    // takes it out of the K-1 amount: 12,000 - (-5,000 + 12,000) = 5,000.
    let allowed = k3_fixture_a_with(|p| {
        *p = k3_partner_with(-5_000, true, 2_000, 10_000, 0, p.schedule_k3.clone());
    });
    let r = estimate(&allowed);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{:?}", r.issues);
    assert_eq!(value(&r, "foreign_k3_gross_income_correction"), 5_000);
}

#[test]
fn k3_taxes_without_a_k3_never_enter_form_1116_sources() {
    // C4: a Form 1116 claim, box 1 zero, box 21 $150 and no K-3: the K-1 tax
    // is not credited, its dividends stay U.S. income, and one note says so.
    let mut d = k3_fixture_c(150, Some(true));
    let mut claim = passive_claim(1_000, 120);
    if let ForeignTaxClaim::Ordinary { sources, .. } = &mut claim {
        sources[0].category = ForeignTaxCategory::Passive {
            sales: vec![],
            withholding: ForeignWithholding::Below15,
            qualified: 0,
        };
    }
    d.credits.as_mut().unwrap().foreign_tax = claim;
    let r = supported(&d);
    assert!(noted(&r, "WARN_K1_FOREIGN_TAX_WITHOUT_K3"));
    // Line 3e stays AGI 75,000; the passive group is the 1,000 source alone:
    // 3a 16,100 x 1,000 / 75,000 = 215; the limit is on 785 of 58,900.
    assert_eq!(form_1116_any(&r, false, "_gross_income_all_sources"), 75_000);
    let credit = value(&r, "foreign_tax_credit");
    assert!(credit > 0 && credit < 120, "{credit}");
    // C3: box 1 $5,000 without a K-3 is refused (line 3e needs the K-3).
    let mut with_income = d.clone();
    if let FederalIncome::Partnership { data } = &mut with_income.income[2].income {
        data.regular.ordinary = 500_000;
        data.alternative.as_mut().unwrap().ordinary = 500_000;
    }
    let r = supported(&with_income);
    assert!(noted(&r, "WARN_FTC_K1_GROSS_ASSUMED"));
}

#[test]
fn k3_withholding_answers_live_outside_the_imported_rows() {
    // F3 (2026-10-09): the withholding treatment of a passive country is the
    // person's own answer, kept beside the rows an import replaces. A passive
    // row needs its country's answer; an answer for a country that no longer
    // has a passive row goes when the schedule is saved.
    let mut k3 = k3_blank();
    k3.countries = vec![
        k3_row("JA", K3Category::Passive, 6_000, 0, 900),
        k3_row("CA", K3Category::General, 20_000, 0, 3_000),
    ];
    assert_eq!(k3.rule(), Some("K3_WITHHOLDING_NEEDS_EACH_PASSIVE_COUNTRY"));
    k3.withholding = vec![k3_answer("JA", ForeignWithholding::AtLeast15)];
    assert_eq!(k3.rule(), None);
    assert!(k3.valid());
    k3.withholding.push(k3_answer("JA", ForeignWithholding::None));
    assert_eq!(k3.rule(), Some("K3_WITHHOLDING_COUNTRY_TWICE"));
    k3.withholding.pop();
    // 2026-10-16 D-2026-10-15-13: unused withholding answers carry
    // metadata, not amounts; Save prunes them against the actual countries.
    for first in ['A', 'B'] {
        for last in 'A'..='Z' {
            k3.withholding
                .push(k3_answer(&alloc::format!("{first}{last}"), ForeignWithholding::None));
        }
    }
    assert!(k3.valid());
    k3 = k3.pruned();
    assert_eq!(k3.withholding.len(), 1);
    // The re-import replaces the rows: Japan stays, so its answer survives;
    // France arrives without an answer; Canada's general row never had one.
    k3.countries = vec![
        k3_row("JA", K3Category::Passive, 7_000, 0, 1_050),
        k3_row("FR", K3Category::Passive, 1_000, 0, 0),
    ];
    assert_eq!(k3.group(&k3.countries[0]), 1);
    assert_eq!(k3.rule(), Some("K3_WITHHOLDING_NEEDS_EACH_PASSIVE_COUNTRY"));
    k3.withholding.push(k3_answer("FR", ForeignWithholding::None));
    assert_eq!(k3.rule(), None);
    // Japan leaves the schedule: its answer is pruned on save.
    k3.countries.remove(0);
    assert_eq!(k3.pruned().withholding, vec![k3_answer("FR", ForeignWithholding::None)]);
}

#[test]
fn w2_pay_marked_abroad_is_read_only_with_a_form_2555_record() {
    // F3 (2026-10-09): a person claiming only the credit for tax on foreign W-2
    // pay does not mark the W-2 amount; the mark is a Form 2555 fact. W-2
    // $100,000, foreign tax $30,000 on all of it.
    let claim = |d: &mut FederalDraft| {
        d.credits.as_mut().unwrap().foreign_tax =
            survey_ordinary_foreign_credit(vec![ForeignOrdinaryTax {
                country: "ZZ".into(),
                owner: Owner::Taxpayer,
                label: "Country A employer".into(),
                category: ForeignTaxCategory::Employment,
                income: 10_000_000,
                tax: 3_000_000,
            }]);
    };
    let mark = |d: &mut FederalDraft, abroad: i64| {
        let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
        data.abroad = abroad;
    };
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    claim(&mut d);
    // No Form 2555: the unmarked W-2 pay is the employment source.
    let unmarked = supported(&d);
    assert!(value(&unmarked, "foreign_tax_credit") > 0);
    // A mark without a Form 2555 record is not read: the same result.
    mark(&mut d, 10_000_000);
    assert_eq!(supported(&d).tax, unmarked.tax);
    // With a Form 2555 record the mark is read: the source must equal the
    // marked pay, so a mark of half the pay no longer reconciles.
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    assert_ne!(estimate(&d).state, ResultState::Unsupported);
    mark(&mut d, 5_000_000);
    unsupported(&d, "Foreign tax credit");
}
