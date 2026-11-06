// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn tip_fica_carryforward_uses_child_credit_first_and_minimum_credit_afterward() {
    let mut draft = complete(FederalStatus::Single);
    add_child(&mut draft);
    draft.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let without_business = supported(&draft);
    draft.credits.as_mut().unwrap().tip_fica_carryforward = Some(tip_fica_carryforward(200_000));
    let with_business = supported(&draft);
    let child = value(&with_business, "child_credit");
    let income_tax = value(&with_business, "income_tax");
    let business = value(&with_business, "general_business_credit");
    assert_eq!(value(&with_business, "general_business_credit_limit"), income_tax - child);
    assert_eq!(business, 2000);
    assert_eq!(
        value(&with_business, "minimum_tax_credit_limit"),
        (value(&without_business, "minimum_tax_credit_limit") - business).max(0)
    );
    assert_eq!(value(&with_business, "tip_fica_credit_carryforward_to_next_year"), 0);
    assert_eq!(supported(&restart(draft).draft), with_business);
}
#[test]
fn independent_wage_interest_joint_charity_fixtures() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Interest { data: interest_source(native_interest(20_000, 0)) }),
    );
    let r = supported(&d);
    assert_eq!(r.tax, Some(504_700));
    assert_eq!(r.refund, Some(145_300));
    assert_eq!(value(&r, "agi"), 60200);
    let gifts = d.deductions.as_mut().unwrap();
    gifts.set_cash_charity(100_000);
    assert_eq!(supported(&d).tax, Some(492_700));
    assert_eq!(supported(&complete(FederalStatus::Joint)).tax, Some(764_300));
}

#[test]
fn both_spouses_w2_and_one_grace_balance_share_one_form2441_and_care_credit() {
    for joint in [true, false] {
        let (d, benefits, total, credit, wages, withholding) = if joint {
            (joint_employer_with_one_grace_return(), 2_000, 4_000, 700, 14_000_000, 900_000)
        } else {
            let mut d = benefit_return(FederalStatus::Single);
            grace_and_current_benefits(&mut d);
            (d, 3_000, 5_000, 350, 6_000_000, 650_000)
        };
        let r = supported(&d);
        for (number, amount) in [
            (12, benefits),
            (13, 2_000),
            (14, 0),
            (15, total),
            (20, total),
            (21, 7_500),
            (22, 0),
            (23, total),
            (24, 0),
            (25, total),
            (26, 0),
        ] {
            assert_eq!(care_line_value(&r, number), amount);
        }
        if joint {
            assert!(matches!(&d.income[0].income,
                FederalIncome::Wage { data } if FederalWage::from(data).care_benefits == 100_000));
            assert!(matches!(&d.income[1].income,
                FederalIncome::Wage { data } if FederalWage::from(data).care_benefits == 100_000));
        } else {
            assert_eq!(value(&r, "care_benefits_excluded"), 5_000);
        }
        assert_eq!(value(&r, "care_credit"), credit);
        assert_eq!(return_line(&r, "1a"), wages);
        assert_eq!(return_line(&r, "1e"), 0);
        assert_eq!(return_line(&r, "25a"), withholding);
        let restored = restart(d.clone()).draft;
        assert_eq!(restored, d);
        assert_eq!(supported(&restored), r);
    }
}
#[test]
fn both_spouses_w2_one_grace_balance_earned_income_and_credit_limits() {
    let mut d = joint_employer_with_one_grace_return();
    if let FederalIncome::Wage { data } = &mut d.income[1].income {
        let mut data = edit_wage(data);
        data.wages = 250_000;
    }
    let r = supported(&d);
    for (id, expected) in [
        ("care_form2441_line19", 2500),
        ("care_form2441_line20", 2500),
        ("care_benefits_excluded", 2500),
        ("care_benefits_taxable", 1500),
    ] {
        assert_eq!(value(&r, id), expected);
    }
    // The approved source proportion replaces the old hand-entered1500/0:
    // line15 shares3000/1000 allocate taxable1500 as1125/375. Spouse earned
    // income is2875, so round(2875*.35)=1006 current credit.
    assert_eq!(value(&r, "care_earned_spouse"), 2875);
    assert_eq!(value(&r, "care_credit"), 1006);
    assert_eq!(return_line(&r, "1a"), 10_250_000);
    assert_eq!(return_line(&r, "1e"), 150_000);
    assert_eq!(return_line(&r, "25a"), 900_000);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn both_spouses_w2_one_grace_balance_preserves_source_identity_and_bounds() {
    let d = joint_employer_with_one_grace_return();
    let mut missing = d.clone();
    missing.credits.as_mut().unwrap().care.benefits[0].wage = 9999;
    assert!(!missing.valid_stored());
    let mut duplicate = d.clone();
    let b = duplicate.credits.as_ref().unwrap().care.benefits[0].clone();
    duplicate.credits.as_mut().unwrap().care.benefits.push(b);
    assert!(!duplicate.valid_stored());
    // A grace balance belongs to the employee's2025plan, not a2026W2.
    let mut grace = d.clone();
    grace.credits.as_mut().unwrap().care.grace[0].owner = Owner::Spouse;
    assert!(grace.valid_stored());
    grace.credits.as_mut().unwrap().care.grace[0].used_in_return_year = 100_000;
    assert!(grace.valid_stored());
    supported(&grace);
    // No supplied taxable split or repeated eligibility gate is required.
    let mut taxable = d.clone();
    care_box10(&mut taxable, 0, 500_000);
    care_box10(&mut taxable, 1, 500_000);
    care_costs(&mut taxable, 1_200_000);
    assert_eq!(value(&supported(&taxable), "care_benefits_taxable"), 4500);
    let mut lower_costs = d.clone();
    care_costs(&mut lower_costs, 100_000);
    assert_eq!(value(&supported(&lower_costs), "care_benefits_excluded"), 1000);
}

#[test]
fn low_wage_return_compares_eic_limits_with_unearned_income() {
    for (status, wages, interest, expected) in [
        (FederalStatus::Single, 5000, 10000, 345),
        (FederalStatus::Single, 1000, 10000, 78),
        (FederalStatus::Joint, 10000, 10000, 520),
    ] {
        let mut d = complete(status);
        d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
        add(&mut d, income(savings_income(interest, 0)));
        let r = supported(&d);
        assert_eq!(value(&r, "earned_income_credit"), expected);
        assert_eq!(value(&r, "agi"), wages + interest);
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn royalty_investment_income_uses_net_receipts_for_eic() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 1000000, 0);
    add_child(&mut d);
    add(&mut d, income(royalty(13200, 1000)));
    assert!(value(&supported(&d), "earned_income_credit") > 0);
    d.income[1].income = royalty(13201, 1000);
    assert_eq!(value(&supported(&d), "earned_income_credit"), 0);
    d.income[1].income = royalty(1000, 5000);
    add(&mut d, income(savings_income(12201, 0)));
    // A royalty loss cannot erase the worksheet's separately counted interest.
    assert_eq!(value(&supported(&d), "earned_income_credit"), 0);
}

#[test]
fn iso_unreported_compensation_uses_sale_month_for_deemed_care_earnings() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    set_care_months(&mut d, care_months(Owner::Spouse, 5));
    supported(&d);
    let mut sale = disqualifying_iso_sale();
    sale.exercised = 20260201;
    sale.proceeds = 1020000;
    let mut entry = income(FederalIncome::IsoSale { data: sale });
    entry.owner = crate::DocumentOwner::Spouses;
    add(&mut d, entry);
    care_aggregate(&mut d, Owner::Spouse, 20_000);
    let r = supported(&d);
    // April's 200 replaces part of that month's 500, not a sixth month.
    assert_eq!(value(&r, "iso_unreported_compensation"), 200);
    assert_eq!(value(&r, "care_earned_spouse"), 2500);
    assert_eq!(value(&r, "care_credit"), 875);
    assert_eq!(value(&r, "agi"), 100200);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn iso_care_month_allocation_respects_one_or_two_people_and_exception_months() {
    for (people, amount, sold, expected) in [
        (2, 200, 20260421, 2500),
        (2, 200, 20261021, 2700),
        (2, 800, 20260421, 2500),
        (1, 200, 20260421, 1250),
        (1, 800, 20260421, 1250),
    ] {
        let mut d = benefit_return(FederalStatus::Joint);
        clear_care_benefits(&mut d);
        if people == 1 {
            d.dependents.pop();
        }
        d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
        set_care_months(&mut d, care_months(Owner::Spouse, 5));
        let mut sale = disqualifying_iso_sale();
        sale.exercised = 20260201;
        sale.sold = sold;
        sale.proceeds = 1000000 + amount * 100;
        let mut entry = income(FederalIncome::IsoSale { data: sale });
        entry.owner = crate::DocumentOwner::Spouses;
        add(&mut d, entry);
        care_aggregate(&mut d, Owner::Spouse, if sold / 100 % 100 <= 5 { amount * 100 } else { 0 });
        let r = supported(&d);
        assert_eq!(value(&r, "care_earned_spouse"), expected);
        assert_eq!(value(&r, "care_credit"), (expected * 35 + 50) / 100);
        assert_eq!(value(&r, "agi"), 100000 + amount);
    }
}

#[test]
fn iso_care_month_sums_lots_and_does_not_repeat_compensation_already_in_payroll() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    set_care_months(&mut d, care_months(Owner::Spouse, 5));
    for _ in 0..3 {
        let mut sale = disqualifying_iso_sale();
        sale.exercised = 20260201;
        sale.proceeds = 1020000;
        let mut entry = income(FederalIncome::IsoSale { data: sale });
        entry.owner = crate::DocumentOwner::Spouses;
        add(&mut d, entry);
    }
    care_aggregate(&mut d, Owner::Spouse, 60_000);
    let before = supported(&d);
    assert_eq!(value(&before, "iso_unreported_compensation"), 600);
    assert_eq!(value(&before, "care_earned_spouse"), 2500);
    for entry in &mut d.income {
        if let FederalIncome::IsoSale { data } = &mut entry.income {
            data.in_wages = true;
        }
    }
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 60000, 0));
    let mut months = care_months(Owner::Spouse, 5);
    months[3].wages = 60000;
    set_care_months(&mut d, months);
    let reported = supported(&d);
    assert_eq!(value(&reported, "care_earned_spouse"), 2500);
    assert_eq!(value(&reported, "care_credit"), value(&before, "care_credit"));
    assert_eq!(reported.tax, before.tax);
    assert_eq!(value(&reported, "agi"), 100600);
}

#[test]
fn iso_care_earnings_keep_the_shared_month_rule_when_both_spouses_are_students() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.income.clear();
    d.people.as_mut().unwrap().taxpayer.full_time_student = true;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    let mut months = care_months(Owner::Taxpayer, 5);
    months.extend(care_months(Owner::Spouse, 5));
    set_care_months(&mut d, months);
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let mut sale = disqualifying_iso_sale();
        sale.exercised = 20260101;
        sale.sold = 20260121;
        sale.proceeds = 1010000;
        let mut entry = income(FederalIncome::IsoSale { data: sale });
        entry.owner = crate::DocumentOwner::from(owner);
        add(&mut d, entry);
    }
    care_aggregate(&mut d, Owner::Taxpayer, 10_000);
    care_aggregate(&mut d, Owner::Spouse, 10_000);
    let r = supported(&d);
    let left = value(&r, "care_earned_taxpayer");
    let right = value(&r, "care_earned_spouse");
    // January can add 400 to one spouse; February–May add 500 once each.
    assert_eq!(left + right, 2600);
    assert_eq!(left.min(right), 100);
    assert_eq!(value(&r, "agi"), 200);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn employee_tip_social_base_precedes_self_employment_and_is_per_spouse() {
    let mut draft = wage_with_employee_tips(180000, allocated_employee_tips(10000));
    add(&mut draft, income(cash_business(10000)));
    let r = supported(&draft);
    assert_eq!(value(&r, "employee_tips_0_social_tax"), 279);
    assert_eq!(value(&r, "employee_tips_0_medicare_tax"), 145);
    assert_eq!(value(&r, "self_employment_tax"), 268);
    let mut joint = complete(FederalStatus::Joint);
    joint.income[0] = wage(1, Owner::Taxpayer, 6000000, 0);
    add(&mut joint, wage(2, Owner::Spouse, 18000000, 0));
    for entry in &mut joint.income {
        if let FederalIncome::Wage { data } = &mut entry.income {
            let mut data = edit_wage(data);
            data.employee_tips = Some(allocated_employee_tips(10000));
        }
    }
    let r = supported(&joint);
    assert_eq!(value(&r, "employee_tips_0_social_tax"), 620);
    assert_eq!(value(&r, "employee_tips_1_social_tax"), 279);
    assert_eq!(value(&r, "additional_medicare_tax"), 90);
    assert_eq!(return_line(&r, "23"), 127900);
}

#[test]
fn prior_unemployment_threshold_combines_spouses_sources_and_original_years() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, prior_unemployment_repayment(150000, Owner::Taxpayer));
    let mut spouse = prior_unemployment_repayment(150000, Owner::Spouse);
    let FederalIncome::GovernmentPayment { data } = &mut spouse.income else { panic!() };
    data.repayments[0].received_year = 2024;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_income_repayment_review_total"), 3000);
    assert_eq!(value(&r, "agi"), 100000);
    assert_eq!(r.tax, Some(764300));
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
    let FederalIncome::GovernmentPayment { data } = &mut d.income[0].income else { panic!() };
    data.repayments[0].amount += 1;
    unsupported(&d, "Earlier-income repayments");
}

#[test]
fn espp_compensation_uses_sale_month_for_dependent_care_earnings() {
    let mut d = benefit_return(FederalStatus::Joint);
    clear_care_benefits(&mut d);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().full_time_student = true;
    set_care_months(&mut d, care_months(Owner::Spouse, 5));
    let mut lot = ordinary_espp_lot();
    lot.exercised = 20260201;
    lot.sold = 20260401;
    lot.market_price = 2200;
    lot.proceeds = 220000;
    let mut entry = income(FederalIncome::EsppSale { data: lot });
    entry.owner = crate::DocumentOwner::Spouses;
    add(&mut d, entry);
    care_aggregate(&mut d, Owner::Spouse, 20_000);
    let r = supported(&d);
    assert_eq!(value(&r, "iso_unreported_compensation"), 200);
    assert_eq!(value(&r, "care_earned_spouse"), 2500);
    assert_eq!(value(&r, "care_credit"), 875);
    assert_eq!(value(&r, "agi"), 100200);
    assert_eq!(supported(&restart(d).draft), r);
}
