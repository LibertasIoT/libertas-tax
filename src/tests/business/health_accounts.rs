// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn business_health_deducts_above_line_without_reducing_se_tax_or_double_counting() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(health_business(50000, 6000, false)));
    let baseline = {
        let mut reference = d.clone();
        reference.income[1].income = cash_business(50000);
        supported(&reference)
    };
    let r = supported(&d);
    let id = d.income[1].id;
    assert_eq!(value(&r, &alloc::format!("business_{id}_health_insurance")), 6000);
    assert_eq!(value(&r, &alloc::format!("business_{id}_medical_premiums")), 0);
    assert_eq!(value(&r, "self_employment_tax"), value(&baseline, "self_employment_tax"));
    assert_eq!(value(&r, "agi"), value(&baseline, "agi") - 6000);
    assert_eq!(value(&r, "adjustments"), value(&baseline, "adjustments") + 6000);
    assert_eq!(value(&r, "qbi_deduction"), value(&baseline, "qbi_deduction") - 1200);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn business_health_employer_eligibility_keeps_premiums_in_medical_itemization() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(health_business(50000, 30000, true)));
    let r = supported(&d);
    let mut reference = d.clone();
    reference.income[1].income = cash_business(50000);
    itemized(reference.deductions.as_mut().unwrap()).medical = 3000000;
    let expected = supported(&reference);
    assert_eq!(r.tax, expected.tax);
    assert_eq!(value(&r, "agi"), value(&expected, "agi"));
    assert_eq!(value(&r, &alloc::format!("business_{}_health_insurance", d.income[1].id)), 0);
}

#[test]
fn business_health_zero_income_and_duplicate_months() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(health_business(-1000, 5000, false)));
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("business_{}_health_insurance", d.income[1].id)), 0);
    assert_eq!(value(&r, &alloc::format!("business_{}_medical_premiums", d.income[1].id)), 5000);
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    let BusinessHealthInsurance::Plan { months, .. } = &mut data.health_insurance else {
        unreachable!()
    };
    months.push(months[0].clone());
    assert!(!d.valid_stored());
}

#[test]
fn independent_mixed_business_loss_uses_uncapped_reasonable_compensation() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    add(&mut d, income(mixed_business(10000, 6000)));
    add(&mut d, income(mixed_business(-2000, 1000)));
    add(&mut d, wage(0, Owner::Taxpayer, 5000000, 0));
    let r = supported(&d);
    // $565 half-SE goes to the profitable activity. $9,435 * .30 = $2,831.
    // Loss activity keeps reasonable $1,000 compensation without a 30% cap.
    assert_eq!(
        value(&r, &alloc::format!("business_{}_kiddie_earned_income", d.income[1].id)),
        2831
    );
    assert_eq!(
        value(&r, &alloc::format!("business_{}_kiddie_earned_income", d.income[2].id)),
        1000
    );
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 11469);
    let FederalIncome::Business { data } = &mut d.income[2].income else { unreachable!() };
    // Documented capital produces no earned income from this loss activity.
    data.income_source = BusinessIncomeSource::Capital;
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("business_{}_kiddie_earned_income", d.income[2].id)), 0);
}
#[test]
fn optional_se_health_limit_uses_optional_earnings_and_retains_actual_profit() {
    for (premium, agi, tax) in [(6000, 54421, 550800), (10000, 53440, 538800)] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(raised_farm(12000, 11000)));
        elect_se(&mut d, SeMethod::Farm);
        se_health(&mut d, 1, premium);
        let r = supported(&d);
        assert_eq!(return_line(&r, "8"), 100000);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(value(&r, "self_employment_tax"), 1157);
        assert_eq!(r.tax, Some(tax));
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn optional_se_health_attributes_shared_optional_cap_to_the_insured_business() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(6000, 5000)));
    add(&mut d, income(raised_farm(6000, 5000)));
    elect_se(&mut d, SeMethod::Farm);
    se_health(&mut d, 1, 4000);
    let r = supported(&d);
    // Each farm receives $3,780 optional earnings. Form 7206 allocates
    // $290 of the $579 half-SE deduction to the insured farm: cap $3,490.
    assert_eq!(value(&r, "agi"), 57931);
    assert_eq!(r.tax, Some(592800));
}
#[test]
fn optional_se_health_mixes_regular_and_optional_business_earnings_once() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(6000, 5000)));
    add(&mut d, income(cash_business(50000)));
    elect_se(&mut d, SeMethod::Farm);
    se_health(&mut d, 1, 6000);
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 7677);
    assert_eq!(value(&r, "agi"), 103445);
    assert_eq!(r.tax, Some(1969800));
}
#[test]
fn optional_se_archer_limit_retains_actual_insured_business_income() {
    for (receipts, expenses, business_profit, contribution, agi, tax) in
        [(12000, 11000, 0, 42100, 60000, 616800), (6000, 5000, 50000, 71563, 106445, 2019300)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(raised_farm(receipts, expenses)));
        if business_profit != 0 {
            add(&mut d, income(cash_business(business_profit)));
        }
        elect_se(&mut d, SeMethod::Farm);
        let mut a = archer_contributor(Owner::Taxpayer, archer_self(2900), 0);
        a.income = vec![d.income[1].id];
        // Mixed case: $3,839 half-SE * $4,000/$54,000 = $284.37 attributable
        // to farm. Actual $1,000 profit permits $715.63, not optional earnings.
        a.personal = contribution;
        set_archer(&mut d, vec![a.clone()], ArcherSharing::Equal);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(r.tax, Some(tax));
        a.personal += 1;
        set_archer(&mut d, vec![a], ArcherSharing::Equal);
        unsupported(&d, "Archer MSA");
    }
}
