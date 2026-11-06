use super::*;

#[test]
fn self_employed_tip_net_limit_includes_owner_health_and_sep_deductions() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = sep_business(10_000, 1_000) else {
        panic!()
    };
    business.health_insurance = BusinessHealthInsurance::Plan {
        long_term_care: vec![],
        months: vec![BusinessHealthMonth {
            child_increase: 0,
            month: 1,
            premiums: 100_000,
            employer_eligible: false,
            reimbursed: None,
        }],
        establishment: crate::HealthEstablishment::SoleProprietor,
    };
    business.tips = Some(FederalBusinessTips { amount: 1_000_000, eligible: true });
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let result = supported(&draft);
    assert_eq!(value(&result, "business_2_qualified_tips"), 7293);
    assert_eq!(value(&result, "tips_deduction"), 7293);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn business_losses_apply_annual_limit_and_preserve_nol_and_qbi_carries() {
    for (status, wages, loss, excess, agi, nol, qbi) in [
        (FederalStatus::Single, 60000, 10000, 0, 50000, 0, 10000),
        (FederalStatus::Single, 10000, 30000, 0, -20000, 20000, 30000),
        (FederalStatus::Single, 1000000, 400000, 144000, 744000, 144000, 256000),
        (FederalStatus::Joint, 100000, 600000, 88000, -412000, 500000, 512000),
    ] {
        let mut d = empty_return(status);
        add(&mut d, wage(0, Owner::Taxpayer, wages * 100, 0));
        add(&mut d, income(cash_business(-loss)));
        let r = supported(&d);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(value(&r, "excess_business_loss"), excess);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "nol_carryforward"), nol);
        assert_eq!(value(&r, "excess_business_loss_carryforward"), excess);
        assert_eq!(value(&r, "current_net_operating_loss"), nol - excess);
        assert_eq!(value(&r, "qbi_loss_carryforward"), qbi);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn business_loss_does_not_remove_ira_wage_compensation() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 1_000_000, 0);
    add(&mut d, income(cash_business(-30000)));
    d.adjustments.as_mut().unwrap().iras = ira_facts(750_000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 7500);
    assert_eq!(value(&r, "nol_carryforward"), 20000); // Personal IRA deduction cannot enlarge this NOL.
}

#[test]
fn business_health_uses_each_plans_business_limit_and_each_owners_se_tax() {
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, income(health_business(10000, 20000, false)));
    add(&mut d, income(cash_business(30000)));
    add(&mut d, income(cash_business(-5000)));
    let mut spouse = income(health_business(1000, 20000, false));
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let r = supported(&d);
    // $35,000 net SE earnings: SE tax 4,945; half rounds to 2,473.
    // Form 7206 denominator excludes the $5,000 loss: 10,000 / 40,000.
    assert_eq!(value(&r, &alloc::format!("business_{}_health_insurance", d.income[0].id)), 9382);
    // Spouse's SE tax 141, half 71. No sharing of spouses' income limits.
    assert_eq!(value(&r, &alloc::format!("business_{}_health_insurance", d.income[3].id)), 929);
    assert_eq!(value(&r, &alloc::format!("business_{}_medical_premiums", d.income[0].id)), 10618);
}

#[test]
fn small_schedule_c_net_uses_aggregate_student_month_lower_bound() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.dependents.pop();
    d.dependents[0].set_care_expenses(300_000);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let mut business = income(cash_business(300));
    business.owner = crate::DocumentOwner::Spouses;
    if let FederalIncome::Business { data } = &mut business.income {
        if let BusinessActivity::Ordinary { receipts, .. } = &mut data.activity {
            *receipts = 50_000;
        }
        data.expenses.push(ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Same-month service supplies".into() },
            20_000,
        ));
    }
    add(&mut d, business);
    let mut months = care_months(Owner::Spouse, 5);
    months[0].business_net = 30_000;
    set_care_months(&mut d, months);

    let january = supported(&d);
    assert_eq!(value(&january, "self_employment_tax"), 0);
    assert_eq!(value(&january, "care_earned_spouse"), 1_250); // Approved aggregate lower bound.
    assert_eq!(value(&january, "care_credit"), 438);
    assert_eq!(value(&january, "agi"), 100_300);
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, january.return_lines);

    let CareFilerSituation::Student { earned_in_months, .. } =
        &mut d.credits.as_mut().unwrap().care.filers[0].situation
    else {
        panic!()
    };
    *earned_in_months = Some(0);
    let december = supported(&d);
    assert_eq!(value(&december, "care_earned_spouse"), 1_550);
    assert!(value(&december, "care_credit") > value(&january, "care_credit"));
    assert_eq!(value(&december, "agi"), value(&january, "agi"));
}

#[test]
fn self_employed_insurance_exception_limits_relief_to_taxable_ira_and_simple_rate() {
    let mut d = complete(FederalStatus::Single);
    self_employed_insurance(&mut d, true, 12, 20260302);
    // The SIMPLE rate beside an exception (early_simple, early_distribution_tax 1,500) is not
    // storable on the 1099-R paper: only the ordinary-rate relief below is asserted.
    let FederalIncome::Retirement { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let mut data = edit_retirement(data);
    data.box_2a = 300000;
    data.taxable = Some(RetirementTaxable::Payer);
    drop(data);
    assert_eq!(value(&supported(&d), "early_distribution_tax"), 0);
    assert_eq!(value(&supported(&d), "agi"), 63000);
    let FederalIncome::Retirement { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    let mut data = edit_retirement(data);
    data.kind = RetirementKind::Pension;
    drop(data);
    unsupported(&d, "Income");
}

#[test]
fn separate_business_amt_depreciation_adjusts_amti_without_changing_regular_qbi_or_se() {
    let d = business_with_bonus(50000, separate_amt_equipment());
    let r = supported(&d);
    for (id, amount) in [
        ("business_2_asset_0_macrs", 65536),
        ("business_2_asset_0_amt_depreciation", 87395),
        ("business_2_asset_0_remaining_basis", 32768),
        ("business_2_asset_0_amt_remaining_basis", 43697),
    ] {
        assert_eq!(r.lines.iter().find(|l| l.id == id).unwrap().amount, amount);
    }
    assert_eq!(value(&r, "amt_business_depreciation_adjustment"), -219);
    assert_eq!(
        value(&r, "alternative_minimum_taxable_income"),
        value(&r, "agi") - value(&r, "qbi_deduction") - 219
    );
    // A different retained AMT balance cannot change regular computations.
    let mut changed = d.clone();
    let FederalIncome::Business { data } = &mut changed.income[1].income else { unreachable!() };
    if let EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Separate { prior_depreciation, .. },
        ..
    } = &mut data.assets[0].history
    {
        *prior_depreciation += 15000;
    }
    let alternative = supported(&changed);
    for id in ["agi", "self_employment_tax", "qbi_deduction", "regular_income_tax"] {
        assert_eq!(value(&r, id), value(&alternative, id));
    }
    assert_ne!(
        value(&r, "alternative_minimum_taxable_income"),
        value(&alternative, "alternative_minimum_taxable_income")
    );
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn optional_se_ira_combination_uses_actual_compensation() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(6000, 5000)));
    elect_se(&mut d, SeMethod::Farm);
    d.adjustments.as_mut().unwrap().iras = ira_facts(10000, 0, Answer::No);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 100);
}

#[test]
fn optional_se_sep_uses_actual_profit_and_exact_deposit_limit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(12000, 11000)));
    elect_se(&mut d, SeMethod::Farm);
    let id = d.income[1].id;
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.retirement = BusinessRetirement::Sep { contribution: 8420, rate: 25.0 };
    let r = supported(&d);
    // ($1,000 actual profit - $579 half-SE) * 25/125 = $84.20.
    assert_eq!(
        r.lines.iter().find(|l| l.id == alloc::format!("business_{id}_sep_limit")).unwrap().amount,
        8420
    );
    assert_eq!(value(&r, "agi"), 60337);
    assert_eq!(r.tax, Some(621000));
    se_health(&mut d, 1, 6000);
    assert_eq!(supported(&d).tax, Some(549600));
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.retirement = BusinessRetirement::Sep { contribution: 8421, rate: 25.0 };
    unsupported(&d, "Business retirement");
}
#[test]
fn optional_se_ira_does_not_create_compensation_from_optional_earnings() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(12000, 11000)));
    elect_se(&mut d, SeMethod::Farm);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    d.adjustments.as_mut().unwrap().iras = ira_facts(42100, 0, Answer::No);
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 421);
    assert_eq!(value(&r, "agi"), 0);
    assert_eq!(r.tax, Some(115700));
    d.adjustments.as_mut().unwrap().iras = ira_facts(42101, 0, Answer::No);
    assert!(estimate(&d).issues.iter().any(|i| i.message.contains("compensation")));
}
#[test]
fn optional_se_half_tax_reduces_ira_wages_even_with_an_actual_business_loss() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 100000, 0)];
    add(&mut d, income(raised_farm(12000, 14000)));
    elect_se(&mut d, SeMethod::Farm);
    waive_farm_carryback(&mut d);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    d.adjustments.as_mut().unwrap().iras = ira_facts(42100, 0, Answer::No);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), -2000);
    assert_eq!(r.tax, Some(115700));
    d.adjustments.as_mut().unwrap().iras = ira_facts(42101, 0, Answer::No);
    assert!(estimate(&d).issues.iter().any(|i| i.message.contains("compensation")));
}

#[test]
fn self_employed_housing_is_separate_adjustment_and_keeps_se_tax() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(excluded_foreign_business(200000, 50000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    self_employed_housing(&mut d, None);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
    assert_eq!(value(&r, "foreign_business_taxpayer_net_exclusion"), 92633);
    assert_eq!(value(&r, "foreign_housing_deduction"), 18606);
    assert_eq!(value(&r, "foreign_housing_taxpayer_deduction_carryforward"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 21194);
    assert_eq!(value(&r, "agi"), 28164);
    assert_eq!(value(&r, "net_investment_modified_agi"), 120797);
    assert_eq!(return_line(&r, "8"), 5736700);
    assert_eq!(return_line(&r, "9"), 5736700);
    assert_eq!(return_line(&r, "10"), 2920300);
    assert_eq!(r.tax, Some(2409000));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    // Housing deduction is not a second exclusion from IRA compensation.
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::No);
    assert_eq!(value(&supported(&d), "ira_deduction"), 7500);
}

#[test]
fn self_employed_housing_can_stand_alone_and_preserves_ira_compensation() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(foreign_service_business(50000)));
    self_employed_housing(&mut d, None);
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_earned_exclusion"), 0);
    assert_eq!(value(&r, "foreign_housing_deduction"), 18606);
    assert_eq!(value(&r, "agi"), 27861);
    assert_eq!(value(&r, "self_employment_tax"), 7065);
    assert_eq!(r.tax, Some(847500));
    assert_eq!(supported(&restart(d).draft), r);

    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(excluded_foreign_business(150000, 0)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    self_employed_housing(&mut d, None);
    assert_eq!(value(&supported(&d), "agi"), -1208);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::No);
    let r = supported(&d);
    // Pub. 54: do not reduce IRA compensation by the housing deduction.
    assert_eq!(value(&r, "ira_deduction"), 7500);
    assert_eq!(value(&r, "agi"), -8708);
    assert_eq!(supported(&restart(d).draft), r);
}
