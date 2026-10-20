// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn saver_single_joint_and_nonrefundable_limits() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2_500_000, 0);
    d.credits.as_mut().unwrap().saver = (Some(saver(300_000, 0))).into();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 25000); // W-2 already excludes pretax deferrals.
    assert_eq!(value(&r, "income_tax"), 893);
    assert_eq!(value(&r, "saver_credit"), 400);
    assert_eq!(r.tax, Some(49_300));
    d.income[0] = wage(1, Owner::Taxpayer, 2_400_000, 0);
    assert_eq!(value(&supported(&d), "saver_credit"), 793);
    assert_eq!(supported(&d).tax, Some(0));
    d.income[0] = wage(1, Owner::Taxpayer, 1_000_000, 0);
    assert_eq!(value(&supported(&d), "saver_credit"), 0);

    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 2_500_000, 0);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 2_500_000, 0));
    d.credits.as_mut().unwrap().saver = (Some(saver(200_000, 200_000))).into();
    let r = supported(&d);
    assert_eq!(value(&r, "income_tax"), 1783);
    assert_eq!(value(&r, "saver_credit"), 800);
    assert_eq!(r.tax, Some(98_300));
}

#[test]
fn public_safety_disability_wages_and_earned_income_use_remaining_taxable_payments() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.disabled = true;
    d.income.clear();
    add(&mut d, income(disability_pension(20000)));
    let id = d.income[0].id;
    public_safety(&mut d, id, 3000, 3000, PublicSafetyCoverage::Medical);
    let r = supported(&d);
    assert_eq!(return_line(&r, "1h"), 1700000);
    assert_eq!(value(&r, "agi"), 17000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(value(&r, "elderly_credit"), 38);
    let mut reference = d.clone();
    reference.screening.as_mut().unwrap().public_safety.clear();
    reference.income[0].income = disability_pension(17000);
    let other = supported(&reference);
    for key in ["earned_income_credit", "elderly_credit", "total_tax"] {
        assert_eq!(value(&r, key), value(&other, key), "{key}");
    }
}

#[test]
fn public_safety_long_term_care_remaining_premiums_share_age_limit() {
    for (age, limit) in [
        (40, 500),
        (41, 930),
        (50, 930),
        (51, 1860),
        (60, 1860),
        (61, 4960),
        (70, 4960),
        (71, 6200),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            retirement_entry(
                Owner::Taxpayer,
                RetirementKind::Pension,
                10000,
                RetirementTreatment::Ordinary,
            ),
        );
        let id = d.income.last().unwrap().id;
        for _ in 0..2 {
            public_safety(
                &mut d,
                id,
                5000,
                1500,
                PublicSafetyCoverage::LongTermCare { person: -1 },
            );
        }
        set_age(&mut d, -1, age);
        let r = supported(&d);
        assert_eq!(value(&r, "public_safety_medical_before_coordination"), limit);
        assert_eq!(value(&r, "agi"), 67000);
    }
}

#[test]
fn public_safety_premiums_are_added_to_itemization_only_after_exclusion() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            RetirementTreatment::Ordinary,
        ),
    );
    let id = d.income.last().unwrap().id;
    public_safety(&mut d, id, 30000, 3000, PublicSafetyCoverage::Medical);
    let r = supported(&d);
    let mut reference = d.clone();
    reference.screening.as_mut().unwrap().public_safety.clear();
    let FederalIncome::Retirement { data } = &mut reference.income.last_mut().unwrap().income
    else {
        unreachable!()
    };
    data.taxable = RetirementTaxable::Payer { amount: 700000 };
    itemized(reference.deductions.as_mut().unwrap()).medical = 2700000;
    let other = supported(&reference);
    assert_eq!(value(&r, "taxable_income"), value(&other, "taxable_income"));
    assert_eq!(value(&r, "total_tax"), value(&other, "total_tax"));
    assert_eq!(value(&r, "public_safety_medical_before_coordination"), 27000);
}

#[test]
fn public_safety_limits_exclusion_to_taxable_portion_and_checks_source_values() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Pension,
            10000,
            RetirementTreatment::Ordinary,
        ),
    );
    let id = d.income.last().unwrap().id;
    let FederalIncome::Retirement { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.taxable = RetirementTaxable::Payer { amount: 100000 };
    public_safety(&mut d, id, 4000, 4000, PublicSafetyCoverage::Other);
    let r = supported(&d);
    assert_eq!(value(&r, "public_safety_1_excluded"), 1000);
    assert_eq!(value(&r, "public_safety_medical_before_coordination"), 0);
    assert_eq!(value(&r, "agi"), 60000);
    d.screening.as_mut().unwrap().public_safety[0].funded = 400001;
    assert!(!d.valid_stored());
}

#[test]
fn care_distribution_applies_2026_dollar_premium_and_balance_limits() {
    for (premium, balance, relief) in
        [(3000, 100000, 2600), (500, 100000, 500), (3000, 10000, 1000), (0, 100000, 0)]
    {
        let d = care_return(care_claim(premium, balance, false), 20000);
        let r = supported(&d);
        assert_eq!(value(&r, "care_distribution_2_exception"), relief);
        assert_eq!(return_line(&r, "5b"), 1600000);
        assert_eq!(value(&r, "agi"), 136000);
        assert_eq!(value(&r, "early_distribution_tax"), (16000 - relief) / 10);
        assert_eq!(r.tax, Some((21374 + (16000 - relief) / 10) * 100));
        assert_eq!(supported(&restart(d).draft).tax, r.tax);
    }
}

#[test]
fn care_distribution_uses_taxable_amount_not_gross_proportion() {
    let mut d = care_return(care_claim(3000, 100000, false), 20000);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    data.taxable = RetirementTaxable::Payer { amount: 100000 };
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 2000000);
    assert_eq!(return_line(&r, "5b"), 100000);
    assert_eq!(value(&r, "care_distribution_2_exception"), 1000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(r.tax, Some(1779000));
}

// Boundary (matrix row 21): a $1,500 nonmedical HSA withdrawal is ordinary
// income either way (Form 8889 line 16 -> Schedule 1 line 8f, via
// `hsa_taxable_distributions`), but the 20% additional tax (Form 8889 line
// 17b -> Schedule 2 line 13c, via `hsa_additional_tax`) applies only when
// the withdrawal happens on or before the owner's 65th birthday.
#[test]
fn nonmedical_distribution_is_taxed_20_percent_before_65() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19880315;
    let h = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    add_hsa_distribution(
        &mut d,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20260601, amount: 150_000 }],
        None,
        None,
    );
    d.adjustments.as_mut().unwrap().hsa.push(h);
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 1500);
    assert_eq!(value(&r, "hsa_additional_tax"), 300);
    assert_eq!(return_line(&r, "8"), 150_000);
    assert_eq!(return_line(&r, "23"), 30_000);
    assert_eq!(restart(d.clone()).draft, d);

    d.people.as_mut().unwrap().taxpayer.birth_date = 19610101;
    let r = supported(&d);
    assert_eq!(value(&r, "hsa_taxable_distributions"), 1500);
    assert_eq!(value(&r, "hsa_additional_tax"), 0);
    assert_eq!(return_line(&r, "23"), 0);
}
