use super::*;

#[test]
fn nonbusiness_hobby_property_gain_is_included_in_niit_and_ira_magi() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 20000000, 0);
    add(&mut d, income(hobby_sales(10000, 8000, 2000)));
    assert_eq!(value(&supported(&d), "net_investment_income_tax"), 152);
    d.adjustments.as_mut().unwrap().iras = ira_facts(100000, 0, Answer::No);
    assert_eq!(value(&supported(&d), "agi"), 203000);
    d.adjustments.as_mut().unwrap().iras = FederalIras::NONE;
    let FederalIncome::HobbySales { data } = &mut d.income[1].income else { panic!() };
    data.ordinary = false;
    // Not hobby sales: left out with a notice.
    assert!(noted(&supported(&d), "WARN_HOBBY_SALES_LEFT_OUT"));
}

#[test]
fn hobby_goods_income_does_not_create_ira_compensation() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(hobby_sales(10000, 8000, 2000)));
    d.adjustments.as_mut().unwrap().iras = ira_facts(100000, 0, Answer::No);
    unsupported(&d, "IRA contributions");
}
#[test]
fn hobby_goods_income_reduces_workplace_covered_ira_deduction_through_magi() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = covered_wage(1, Owner::Taxpayer, 8100000, 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(750000, 0, Answer::Yes);
    let before = supported(&d);
    add(&mut d, income(hobby_sales(10000, 8000, 2000)));
    let after = supported(&d);
    assert!(value(&after, "adjustments") < value(&before, "adjustments"));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn iso_unreported_compensation_counts_for_ira() {
    let mut d = empty_return(FederalStatus::Single);
    let mut sale = disqualifying_iso_sale();
    sale.exercised = 20260201;
    sale.proceeds = 2000000;
    add(&mut d, income(FederalIncome::IsoSale { data: sale }));
    d.adjustments.as_mut().unwrap().iras = ira_facts(100000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "ira_deduction"), 1000);
    assert_eq!(value(&r, "agi"), 9000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(return_line(&r, "1a"), 0);
    assert_eq!(return_line(&r, "8"), 1000000);
}

#[test]
fn iso_two_sale_irs_example_separates_ordinary_adjustment_and_limited_capital_loss() {
    // Form 6251 line 2k example, shifted one year to the 2026 return.
    let mut first = disqualifying_iso_sale();
    first.granted = 20240103;
    first.exercised = 20250313;
    first.sold = 20260118;
    first.proceeds = 7500000;
    let mut second = first.clone();
    second.sold = 20260421;
    second.proceeds = 6000000;
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::IsoSale { data: first }));
    add(&mut d, income(FederalIncome::IsoSale { data: second }));
    let r = supported(&d);
    assert_eq!(value(&r, "iso_unreported_compensation"), 65000);
    assert_eq!(value(&r, "capital_gain"), 50000);
    assert_eq!(value(&r, "amt_iso_compensation_adjustment"), -65000);
    assert_eq!(value(&r, "amt_capital_adjustment"), -53000);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 22000);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 40000);
    assert_eq!(value(&r, "agi"), 175000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 57000);
}

#[test]
fn allocated_employee_tips_complete_return_keeps_w2_and_4137_separate() {
    let draft = wage_with_employee_tips(60000, allocated_employee_tips(10000));
    let r = supported(&draft);
    assert_eq!(value(&r, "agi"), 70000);
    assert_eq!(value(&r, "regular_income_tax"), 6576);
    assert_eq!(value(&r, "employee_tips_0_social_tax"), 620);
    assert_eq!(value(&r, "employee_tips_0_medicare_tax"), 145);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(return_line(&r, "1a"), 6000000);
    assert_eq!(return_line(&r, "1c"), 1000000);
    assert_eq!(return_line(&r, "1z"), 7000000);
    assert_eq!(return_line(&r, "23"), 76500);
    assert_eq!(r.tax, Some(734100));
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn unemployment_repayment_sources_keep_spouses_separate_and_ignore_row_order() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, government_unemployment(10000, &[2000], 0));
    let mut spouse = government_unemployment(5000, &[1000], 0);
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "unemployment_2_taxable"), 8000);
    assert_eq!(value(&r, "unemployment_3_taxable"), 4000);
    assert_eq!(value(&r, "agi"), 112000);
    assert_eq!(r.tax, Some(908300));
    d.income.reverse();
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn unemployment_net_benefits_reach_social_security_but_never_ira_compensation() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 3000000, 0);
    add(&mut d, government_unemployment(10000, &[2000], 0));
    add(&mut d, income(social_security_net(2000000, 0)));
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_social_security"), 16400);
    assert_eq!(value(&r, "agi"), 54400);
    assert_eq!(r.tax, Some(435100));
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, government_unemployment(10000, &[2000], 0));
    assert_eq!(supported(&d).tax, Some(0));
    d.adjustments.as_mut().unwrap().iras = ira_facts(50000, 0, Answer::No);
    unsupported(&d, "IRA");
}
