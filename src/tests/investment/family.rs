// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;
#[test]
fn section1244_limit_is_shared_across_lots_and_spouses() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(section1244_loss(90000, 0, 20200101)));
    let mut spouse = income(section1244_loss(25000, 0, 20200101));
    spouse.owner = Owner::Spouse;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 100000);
    assert_eq!(value(&r, "capital_long_carryforward"), 15000);
    d.income.swap(1, 2);
    assert_eq!(value(&supported(&d), "capital_long_carryforward"), 15000);
    let FederalIncome::SmallBusinessStock { data } = &mut d.income[1].income else { panic!() };
    data.acquired = 20260101;
    unsupported(&d, "Section 1244");
    let FederalIncome::SmallBusinessStock { data } = &mut d.income[1].income else { panic!() };
    data.basis = StockLossBasis::Cash { cost: 500000, added: 0 };
    let r = supported(&d);
    assert_eq!(value(&r, "section_1244_ordinary_loss"), 95000);
    assert_eq!(value(&r, "capital_gain"), 0);
}

#[test]
fn independent_capital_business_uses_full_statutory_amt_exemption() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    add(&mut d, income(capital_business(10000)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 19293);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 2554);
    assert_eq!(value(&r, "regular_income_tax"), 562);
    // AMTI 18,654 is below the ordinary 90,100 exemption, even with no earned income.
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    assert_eq!(value(&r, &alloc::format!("business_{}_kiddie_earned_income", d.income[1].id)), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn independent_capital_business_with_wages_uses_gross_unearned_worksheet() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    add(&mut d, income(capital_business(10000)));
    add(&mut d, wage(0, Owner::Taxpayer, 5000000, 0));
    let r = supported(&d);
    // $10,000 interest + $10,000 capital-produced business profit - $2,700.
    // With wages, line 1 does not use AGI's half-SE adjustment.
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 17300);
    assert_eq!(value(&r, "kiddie_tax_parent_share"), 3806);
}

#[test]
fn independent_multiple_capital_businesses_combine_without_service_earnings() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    add(&mut d, income(capital_business(14000)));
    add(&mut d, income(capital_business(-4000)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 19293);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 2554);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
    for entry in &d.income[1..] {
        assert_eq!(value(&r, &alloc::format!("business_{}_kiddie_earned_income", entry.id)), 0);
    }
    d.income[2].income = cash_business(-4000);
    let r = supported(&d);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
}

#[test]
fn independent_capital_business_loss_without_service_earnings_uses_agi() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    add(&mut d, income(capital_business(-2000)));
    d.people.as_mut().unwrap().parent_tax = None;
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 8000);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 0);
    assert_eq!(value(&r, "regular_income_tax"), 0);
}

#[test]
fn independent_capital_and_mixed_businesses_keep_se_deduction_attribution() {
    let mut d = young_interest_return();
    d.people.as_mut().unwrap().taxpayer.dependency = FilerDependency::Independent;
    add(&mut d, income(mixed_business(10000, 6000)));
    add(&mut d, income(capital_business(20000)));
    add(&mut d, wage(0, Owner::Taxpayer, 5000000, 0));
    let r = supported(&d);
    // Capital activity retains its own half-SE share; none is shifted to the
    // mixed activity merely because only that activity has service earnings.
    assert_eq!(
        value(&r, &alloc::format!("business_{}_kiddie_earned_income", d.income[1].id)),
        2788
    );
    assert_eq!(value(&r, &alloc::format!("business_{}_kiddie_earned_income", d.income[2].id)), 0);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 34512);
}

#[test]
fn investment_interest_directly_reduces_child_ordinary_unearned_income() {
    let mut d = young_interest_return();
    investment_borrowing(&mut d, 4000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 4650);
    assert_eq!(value(&r, "regular_income_tax"), 1159);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 6000);
}

#[test]
fn trading_property_interest_election_uses_independent_amt_character() {
    let mut draft = complete_trading_property_interest(10000, 20000);
    draft.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 8000, 8000)], &[]));
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        data.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = 1000000;
    }
    draft.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
        InvestmentElection::CapitalFirst { amount: 400000 };
    let r = supported(&draft);
    // Regular ordinary $10k + property recapture $8k + elected capital $4k.
    // AMT ordinary $10k, no property recapture, elected capital $4k.
    assert_eq!(value(&r, "investment_interest_deduction"), 22000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 3000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 14000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 11000);
    assert_eq!(return_line(&r, "7a"), 1200000);
    assert_eq!(return_line(&r, "8"), 1800000);
    assert_eq!(value(&r, "agi"), 90000);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn qsbs_issuer_cap_is_shared_across_documents_and_spouses() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, qsb_income("Synthetic A Inc.", 10_000_000, 710_000_000, 0));
    let mut spouse = qsb_income(" synthetic a inc. ", 10_000_000, 710_000_000, 0);
    spouse.owner = Owner::Spouse;
    add(&mut d, spouse);
    let r = supported(&d);
    assert_eq!(value(&r, "qsbs_2_realized_gain"), 14_000_000);
    assert_eq!(value(&r, "qsbs_2_excluded_gain"), 10_000_000);
    assert_eq!(value(&r, "qsbs_2_taxable_gain"), 4_000_000);
    assert_eq!(value(&r, "capital_gain"), 4_000_000);

    let mut inconsistent = d.clone();
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut inconsistent.income[2].income {
        data.prior_eligible_gain = 100_000_000;
    }
    unsupported(&inconsistent, "Section 1202 issuer history");
}

#[test]
fn qsbs_prior_nol_accepts_unchanged_employer_adoption_exclusion() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    employer_adoption(&mut d, 4_000, 0, false);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000_000)]);
    let before = supported(&d);
    assert_eq!(value(&before, "adoption_benefits_excluded"), 4_000);
    add(&mut d, qsb_income("Synthetic M Inc.", 10_000, 110_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "adoption_benefits_excluded"), 4_000);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 1_000);
    assert_eq!(estimate(&restart(d.clone()).draft), after);

    // Near Form 8839's phaseout, restored section 1202 gain changes the
    // exclusion. That case still needs the dependent line-6 worksheets.
    let mut phased = nol_adoption();
    phased.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 30_000_000)]);
    add(&mut phased, qsb_income("Synthetic N Inc.", 10_000, 110_000, 0));
    unsupported(&phased, "Section 1202 prior NOL AGI");
}

#[test]
fn qsbs_prior_nol_refigures_student_loan_phaseout_for_absorption() {
    for (nol, wages, withholding, social, additional_used) in
        [(100_000, 9_000_000, 650_000, false, 1_167), (200_000, 4_200_000, 500_000, true, 2_158)]
    {
        let mut d = complete(FederalStatus::Single);
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, nol)]);
        d.income[0] = wage(1, Owner::Taxpayer, wages, withholding);
        if social {
            add(&mut d, income(social_security_net(10_000_000, 0)));
        }
        d.adjustments.as_mut().unwrap().student_loan = StudentLoanClaim {
            student_loan_interest: 250_000,
            forms: Vec::new(),
            employer_paid_interest: 0,
        };
        let without_sale = supported(&d);
        add(&mut d, qsb_income("Synthetic A Inc.", 10_000, 110_000, 0));
        let with_sale = supported(&d);
        assert_eq!(value(&with_sale, "agi"), value(&without_sale, "agi"));
        if !social {
            assert_eq!(
                value(&with_sale, "prior_nol_deduction"),
                value(&without_sale, "prior_nol_deduction")
            );
        }
        assert_eq!(
            value(&with_sale, "nol_2025_used") - value(&without_sale, "nol_2025_used"),
            additional_used
        );
    }
}
