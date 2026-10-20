// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn prior_capital_source_ownership_and_return_lines_must_reconcile() {
    let mut d = complete(FederalStatus::Joint);
    let mut spouse = capital_history();
    spouse.owners = PriorReturnOwners::Spouse;
    d.screening.as_mut().unwrap().capital_history = vec![capital_history(), spouse];
    assert_eq!(value(&supported(&d), "capital_loss_carryforward"), 25000);
    d.screening.as_mut().unwrap().capital_history[1].owners = PriorReturnOwners::Taxpayer;
    unsupported(&d, "Capital loss history");
    d.screening.as_mut().unwrap().capital_history = vec![capital_history()];
    d.screening.as_mut().unwrap().capital_history[0].deducted_loss = 150_000;
    unsupported(&d, "Capital loss history");
    d.screening.as_mut().unwrap().capital_history[0] = capital_history();
    d.screening.as_mut().unwrap().capital_history[0].status = FederalStatus::Joint;
    unsupported(&d, "Capital loss history");
}
#[test]
fn nonbusiness_bad_debt_uses_unrecovered_principal_and_capital_loss_limits() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(bad_debt(10000, 2000)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 5000);
    assert_eq!(value(&r, "agi"), 57000);
    assert_eq!(restart(d.clone()).draft, d);
    add(&mut d, income(collectible_sale(15000, 5000, GainTerm::Long, false)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 2000);
    assert_eq!(value(&r, "rate_28_gain"), 2000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 0);
}

#[test]
fn refund_sales_tax_alternative_is_combined_with_the_salt_limit() {
    for (refund, income_paid, sales_available, salt_paid, salt_deducted, expected) in [
        (2500, 10000, 9000, 10000, 10000, 1000),
        (12000, 20000, 15000, 50000, 40000, 0),
        (15000, 20000, 8000, 45000, 40000, 7000),
        (3000, 10000, 12000, 10000, 10000, 0),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::TaxRefund {
                amount: refund * 100,
                withholding: 0,
                history: TaxRefundHistory::IncomeTax {
                    deduction_year: 2025,
                    income_paid: income_paid * 100,
                    sales_available: sales_available * 100,
                    salt_paid: salt_paid * 100,
                    salt_deducted: salt_deducted * 100,
                    itemized: 6000000,
                    standard: 1575000,
                    overall_limit: None,
                    property_refunded: 0,
                    ordinary: true,
                },
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "taxable_state_tax_refund"), expected);
        assert_eq!(value(&r, "agi"), 60000 + expected);
    }
}

#[test]
fn prior_capital_senior_deduction_rejects_inconsistent_source_amounts() {
    for (status, senior, deductions) in [
        (FederalStatus::Single, 600001, 1200000),
        (FederalStatus::Separate, 100, 1200000),
        (FederalStatus::Single, 600000, 500000),
    ] {
        let mut d = complete(FederalStatus::Single);
        let mut history = capital_history();
        history.status = status;
        history.senior_deduction = senior;
        history.deductions = deductions;
        history.investment_parts = None; // Keep this fixture focused on the inconsistent senior source.
        d.screening.as_mut().unwrap().capital_history = vec![history];
        unsupported(&d, "Capital loss history");
    }
    let mut history = capital_history();
    history.senior_deduction = -1;
    assert!(!crate::extended_rules::valid_capital_history(&[history]));
}

#[test]
fn prior_capital_gain_balance_has_class_vintage_election_and_coordination_controls() {
    let mut d = complete(FederalStatus::Single);
    d.deductions.as_mut().unwrap().charity_carryovers.push(prior_public_stock_carry());
    assert!(d.valid_stored());
    d.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&d), "WARN_CHARITY_CARRY_LEFT_OUT"));
    d.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = true;
    d.deductions.as_mut().unwrap().charity_carryovers.push(prior_cash_charity(2025, 100));
    supported(&d);
    d.deductions.as_mut().unwrap().property_gifts.push(public_stock_gift());
    d.deductions.as_mut().unwrap().basis_election = true;
    supported(&d);
    d.deductions.as_mut().unwrap().charity_carryovers[0].appreciation = None;
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().charity_carryovers[0].appreciation = Some(750_000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1_000)]);
    unsupported(&d, "Charitable carryover coordination");
    d.screening.as_mut().unwrap().nol = NolHistory::None;
    payee_statement_credit(&mut d, 10_000);
    unsupported(&d, "Charitable carryover coordination");
    d.credits.as_mut().unwrap().foreign_tax = ForeignTaxClaim::No;
    d.deductions.as_mut().unwrap().charity_carryovers[0].balance = 1_000_001;
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().charity_carryovers[0].balance = 1_000_000;
    d.deductions.as_mut().unwrap().charity_carryovers[0].year = 2026;
    assert!(!d.valid_stored());
}

#[test]
fn qsbs_prior_nol_allows_itemizing_without_modified_agi_deductions() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000)]);
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).personal_property_taxes = 500_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let without_sale = supported(&d);
    add(&mut d, qsb_income("Synthetic A Inc.", 10_000, 110_000, 0));
    let with_sale = supported(&d);
    assert_eq!(value(&with_sale, "qsbs_2_excluded_gain"), 1_000);
    assert_eq!(value(&with_sale, "agi"), value(&without_sale, "agi"));
    assert_eq!(
        value(&with_sale, "prior_nol_deduction"),
        value(&without_sale, "prior_nol_deduction")
    );
    assert_eq!(value(&with_sale, "nol_2025_used") - value(&without_sale, "nol_2025_used"), 1_000);
    assert_eq!(estimate(&restart(d.clone()).draft), with_sale);
}

#[test]
fn qsbs_prior_nol_refigures_modified_itemization() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000)]);
    add(&mut d, qsb_income("Synthetic A Inc.", 10_000_000, 110_000_000, 0));
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).personal_property_taxes = 500_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let top_bracket = supported(&d);
    assert_eq!(value(&top_bracket, "qsbs_2_excluded_gain"), 1_000_000);
    assert_eq!(value(&top_bracket, "nol_2025_used"), 100_000);
    assert_eq!(estimate(&restart(d.clone()).draft), top_bracket);

    // Below the top bracket, the medical line 18 comparison is calculated
    // separately for prior-loss absorption, without changing actual AGI.
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000)]);
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.lots[0].basis = 10_000;
        data.lots[0].proceeds = 110_000;
    }
    if let DeductionSelection::Itemized { facts } = &mut d.deductions.as_mut().unwrap().choice {
        facts.medical = 1_000_000;
    }
    let with_sale = supported(&d);
    let mut without_sale = d.clone();
    without_sale.income.pop();
    let before = supported(&without_sale);
    assert_eq!(value(&with_sale, "agi"), value(&before, "agi"));
    assert_eq!(value(&with_sale, "prior_nol_deduction"), value(&before, "prior_nol_deduction"));
    assert_eq!(value(&with_sale, "nol_2025_used") - value(&before, "nol_2025_used"), 1_075);

    // The second vintage must use the medical threshold after the first
    // vintage's deduction, rather than reusing the initial AGI worksheet.
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2024, 20_000), (2025, 100_000)]);
    without_sale.screening.as_mut().unwrap().nol = prior_nol(&[(2024, 20_000), (2025, 100_000)]);
    let with_two = supported(&d);
    let before_two = supported(&without_sale);
    assert_eq!(value(&with_two, "nol_2024_used"), 20_000);
    assert_eq!(value(&with_two, "nol_2025_used") - value(&before_two, "nol_2025_used"), 1_075);
}

#[test]
fn qsbs_prior_nol_refigures_salt_cap_for_absorption() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 50_000_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1_000_000)]);
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).personal_property_taxes = 5_000_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let before = supported(&d);
    assert_eq!(value(&before, "deduction"), 40_400);

    add(&mut d, qsb_income("Synthetic A Inc.", 10_000, 2_010_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "qsbs_2_excluded_gain"), 20_000);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "prior_nol_deduction"), value(&before, "prior_nol_deduction"));
    // $20,000 excluded gain crosses the $505,000 modified-AGI threshold by
    // $15,000; 30% of that excess removes $4,500 of the SALT deduction.
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 24_500);
    assert_eq!(estimate(&restart(d.clone()).draft), after);
}

#[test]
fn qsbs_prior_nol_salt_phase_down_stops_at_ten_thousand_floor() {
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 50_000_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1_000_000)]);
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).personal_property_taxes = 5_000_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic B Inc.", 10_000, 11_010_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "qsbs_2_excluded_gain"), 110_000);
    assert_eq!(value(&after, "deduction"), 40_400);
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 140_400);
    assert_eq!(estimate(&restart(d.clone()).draft), after);
}

#[test]
fn qsbs_prior_nol_salt_cap_is_refigured_for_each_loss_vintage() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 50_000_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2024, 100_000), (2025, 1_000_000)]);
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).personal_property_taxes = 5_000_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic C Inc.", 10_000, 2_010_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "nol_2024_used"), 100_000);
    // Once the first vintage is deducted, modified AGI is below the SALT
    // phase-down threshold. The next vintage adds only the excluded gain.
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 20_000);
}

#[test]
fn qsbs_prior_nol_refigures_section_68_after_salt_phase_down() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 55_000_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1_000_000)]);
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).personal_property_taxes = 5_000_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic D Inc.", 10_000, 10_010_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "qsbs_2_excluded_gain"), 100_000);
    assert_eq!(value(&after, "itemized_deduction_reduction"), 0);
    assert_eq!(value(&after, "prior_nol_deduction"), value(&before, "prior_nol_deduction"));
    // Refigured $650,000 income exceeds the $640,600 top-bracket threshold
    // by $9,400: section 68 removes $508 of the $10,000 remaining itemized
    // deduction. The SALT cap separately falls from $26,900 to $10,000.
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 117_408);
    assert_eq!(estimate(&restart(d.clone()).draft), after);

    // When threshold excess exceeds the remaining $10,000 itemized amount,
    // section 68 itself caps the reduction: 2/37 of $10,000 rounds to $541.
    if let FederalIncome::QualifiedSmallBusinessSale { data } = &mut d.income[1].income {
        data.lots[0].proceeds = 12_010_000;
    }
    let capped = supported(&d);
    assert_eq!(value(&capped, "nol_2025_used") - value(&before, "nol_2025_used"), 137_441);
}

#[test]
fn qsbs_prior_nol_refigures_personal_casualty_agi_floor() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1_000_000)]);
    let ded = d.deductions.as_mut().unwrap();
    ded.personal_casualties.push(personal_disaster("Synthetic storm", 3_000_000));
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic F Inc.", 10_000, 110_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "qsbs_2_excluded_gain"), 1_000);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "deduction"), value(&before, "deduction"));
    // Form 172 restores $1,000 of excluded gain and the $100 ordinary
    // casualty deduction lost to its refigured 10%-of-AGI limit.
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 1_100);
    assert_eq!(estimate(&restart(d.clone()).draft), after);
}

#[test]
fn qsbs_prior_nol_refigures_current_cash_charity_floor() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000_000)]);
    let ded = d.deductions.as_mut().unwrap();
    ded.cash_charity = 100_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic G Inc.", 10_000, 110_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "qsbs_2_excluded_gain"), 1_000);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "deduction"), value(&before, "deduction"));
    // Form 172 restores $1,000 of excluded gain and $5 of the current
    // cash gift lost to its refigured 0.5%-of-AGI floor.
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 1_005);
    assert_eq!(estimate(&restart(d.clone()).draft), after);

    d.deductions.as_mut().unwrap().cash_charity = 20_000_000;
    supported(&d);
}

#[test]
fn qsbs_prior_nol_refigures_cash_percentage_limit_and_reduces_double_carry() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 10_000_000)]);
    let ded = d.deductions.as_mut().unwrap();
    ded.cash_charity = 9_000_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic K Inc.", 10_000, 110_000, 0));
    let after = supported(&d);
    assert_eq!(value(&before, "nol_2025_used"), 32_400);
    assert_eq!(value(&before, "charity_return_year_nol_reduction"), 19_278);
    assert_eq!(value(&before, "charity_return_year_carryforward"), 30_500);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "deduction"), value(&before, "deduction"));
    assert_eq!(value(&after, "prior_nol_deduction"), value(&before, "prior_nol_deduction"));
    // On the no-prior-NOL Form 172 pass, the $1,000 restored gain raises
    // the cash limit $600 and the 0.5% floor $5. Thus the larger gift
    // deduction offsets $595 of the added income in NOL absorption, so
    // $405 more of the prior NOL is used.
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 405);
    // The excess gift that protected the NOL carry cannot also remain in
    // the five-year charitable carry. The refigured gift increases the
    // section 170(d)(1)(B) reduction by $595.
    assert_eq!(
        value(&after, "charity_return_year_nol_reduction")
            - value(&before, "charity_return_year_nol_reduction"),
        595
    );
    assert_eq!(
        value(&after, "charity_return_year_carryforward")
            - value(&before, "charity_return_year_carryforward"),
        -595
    );
    assert_eq!(estimate(&restart(d.clone()).draft), after);
}

#[test]
fn qsbs_prior_nol_refigures_cash_percentage_excess_with_section_68() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 100_000_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000_000)]);
    let ded = d.deductions.as_mut().unwrap();
    ded.cash_charity = 80_000_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let before = supported(&d);
    add(&mut d, qsb_income("Synthetic L Inc.", 10_000, 110_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(
        value(&after, "itemized_deduction_reduction"),
        value(&before, "itemized_deduction_reduction")
    );
    // Modified AGI raises the cash allowance $600 and the floor $5.
    // Section 68 independently reduces the itemized deduction another
    // $54 as modified income crosses further into the top-rate bracket.
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 459);
    assert_eq!(
        value(&after, "charity_return_year_nol_reduction")
            - value(&before, "charity_return_year_nol_reduction"),
        595
    );
    assert_eq!(estimate(&restart(d.clone()).draft), after);

    // At sufficiently high income, section 68's lesser amount is the
    // itemized deduction itself. The extra $595 gift allowance then
    // increases the section 68 reduction by $32 and protects only $563
    // of NOL from absorption.
    d.income[0] = wage(1, Owner::Taxpayer, 200_000_000, 0);
    d.deductions.as_mut().unwrap().cash_charity = 150_000_000;
    d.income.pop();
    let before_capped = supported(&d);
    add(&mut d, qsb_income("Synthetic L Inc.", 10_000, 110_000, 0));
    let after_capped = supported(&d);
    assert_eq!(value(&after_capped, "nol_2025_used") - value(&before_capped, "nol_2025_used"), 437);
    assert_eq!(
        value(&after_capped, "charity_return_year_nol_reduction")
            - value(&before_capped, "charity_return_year_nol_reduction"),
        563
    );
    assert_eq!(estimate(&restart(d.clone()).draft), after_capped);
}

#[test]
fn qsbs_prior_nol_accepts_passive_sources_with_unchanged_allowance() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12_000_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000_000)]);
    add(&mut d, income(FederalIncome::Rental { data: rental(30_000, 5_000) }));
    let before = supported(&d);
    assert_eq!(value(&before, "rental_income"), 15_000);
    add(&mut d, qsb_income("Synthetic passive A Inc.", 10_000, 110_000, 0));
    let after = supported(&d);
    assert_eq!(value(&after, "rental_income"), 15_000);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    assert_eq!(value(&after, "nol_2025_used") - value(&before, "nol_2025_used"), 1_000);
    assert_eq!(estimate(&restart(d.clone()).draft), after);

    // An inactive loss has no special allowance at either modified AGI.
    let mut loss = rental(1_000, 6_000);
    loss.participation = crate::RentalParticipation::NotActive;
    d.income[1].income = FederalIncome::Rental { data: loss };
    let suspended = supported(&d);
    assert_eq!(value(&suspended, "rental_suspended_loss"), 15_000);
    assert_eq!(value(&suspended, "rental_income"), 0);
}

#[test]
fn qsbs_prior_nol_keeps_changed_passive_allowance_explicit() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12_000_000, 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 10_000)]);
    add(&mut d, income(FederalIncome::Rental { data: rental(9_000, 30_000) }));
    assert_eq!(value(&supported(&d), "rental_income"), -20_000);
    add(&mut d, qsb_income("Synthetic passive B Inc.", 10_000, 110_000, 0));
    unsupported(&d, "Section 1202 prior NOL AGI");
}

#[test]
fn qsbs_prior_nol_refigures_social_security_for_absorption() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100_000)]);
    d.income[0] = wage(1, Owner::Taxpayer, 2_000_000, 200_000);
    add(&mut d, income(social_security_net(2_000_000, 0)));
    let without_sale = supported(&d);
    add(&mut d, qsb_income("Synthetic A Inc.", 10_000, 110_000, 0));
    let with_sale = supported(&d);
    assert_eq!(value(&with_sale, "agi"), value(&without_sale, "agi"));
    assert_eq!(
        value(&with_sale, "prior_nol_deduction"),
        value(&without_sale, "prior_nol_deduction")
    );
    assert_eq!(value(&with_sale, "nol_2025_used") - value(&without_sale, "nol_2025_used"), 1_500);

    // Form 172 line 13 uses the Social Security adjustment before the
    // medical 7.5%-of-AGI threshold is refigured on lines 14–18.
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).medical = 500_000;
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    let with_medical = supported(&d);
    let mut no_sale_medical = d.clone();
    no_sale_medical.income.pop();
    let without_medical_sale = supported(&no_sale_medical);
    assert_eq!(
        value(&with_medical, "nol_2025_used") - value(&without_medical_sale, "nol_2025_used"),
        1_612
    );
}

// T2-33x fixture (matrix row 33, Form 1098-E box 2, 2026-09-26). Sources:
// Form 1098-E box 2 instructions (f1098e, "loan origination fees and/or
// capitalized interest are not included in box 1 for loans made before
// September 1, 2004") and Pub 970 chapter 4 ("Include as Interest", "Form
// 1098-E": qualifying interest not on the form is also deductible, and a
// lender that received less than $600 need not send the form), fetched from
// irs.gov on 2026-09-26. The deduction is the smaller of $2,500 or the
// interest paid, then phased out on MAGI ($85,000-$100,000 single, Rev.
// Proc. 2025-32). Every amount is worked by hand, never copied from the App.

#[test]
fn box_2_interest_and_interest_without_a_form_join_box_1() {
    // Single filer, wages $60,000 (only income; MAGI $60,000, below the
    // $85,000 phase-out start).
    //   Form 1098-E #1 (pre-September 2004 loan): box 1 $900.00, box 2
    //     checked; the filer figured $410.00 more under Pub 970: a $300
    //     origination fee over 60 months is $5 a month, 12 months = $60.00,
    //     plus $350.00 of capitalized interest paid off by 2026 principal.
    //   Form 1098-E #2: box 1 $700.00, box 2 not checked.
    //   A third lender received $250.00 and sent no form (under $600).
    //   interest paid: $900 + $410 + $700 + $250 = $2,260
    //   deduction: min($2,260, $2,500) = $2,260, no phase-out
    //   AGI: $60,000 - $2,260 = $57,740
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
    d.adjustments.as_mut().unwrap().student_loan = loan_claim(
        vec![
            box_2(90_000, 41_000),
            Form1098E { interest: 70_000, box_2: Form1098EBox2::NotChecked },
        ],
        25_000,
    );
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 2_260);
    assert_eq!(value(&r, "agi"), 57_740);
}

#[test]
fn box_2_interest_is_added_before_the_2500_cap() {
    // Pub970: box2 joins box1 before the cap and the MAGI phase-out.
    for (wages, box1, box2, no_form, deduction, agi) in [
        (5_000_000, 230_000, 40_000, 0, 2500, 47500),
        (9_500_000, 180_000, 60_000, 0, 800, 94200),
        (10_000_000, 100_000, 30_000, 20_000, 0, 100000),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
        d.adjustments.as_mut().unwrap().student_loan = loan_claim(vec![box_2(box1, box2)], no_form);
        let r = supported(&d);
        assert_eq!(value(&r, "adjustments"), deduction, "{wages}");
        assert_eq!(value(&r, "agi"), agi, "{wages}");
    }
}

// Employer-paid student loan interest (unit T2-fx). Interest an employer
// paid after March 27, 2020 under a section 127 educational assistance
// program is excluded from income and cannot also be deducted (Pub 970
// chapter 4 "No double benefit allowed"; section 221(e)(1)). The lender's
// Form 1098-E box 1 still reports it, so it is removed before the $2,500
// cap and the MAGI phase-out. Every amount is worked by hand.

#[test]
fn employer_paid_interest_is_removed_before_the_2500_cap() {
    // Pub970/IRC221(e)(1): employer assistance is subtracted before the
    // cap and phase-out. Independent expected figures include the $1 edge.
    for (wages, interest, no_form, employer, removed, deduction, agi) in [
        (6_000_000, 180_000, 15_000, 60_000, Some(600), 1350, 58650),
        (6_000_000, 350_000, 0, 100_000, None, 2500, 57500),
        (6_000_000, 350_000, 0, 100_100, None, 2499, 57501),
        (9_500_000, 240_000, 0, 90_000, None, 500, 94500),
        (6_000_000, 120_000, 0, 120_000, Some(1200), 0, 60000),
    ] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
        d.adjustments.as_mut().unwrap().student_loan =
            employer_paid(loan_claim(vec![box_1(interest)], no_form), employer);
        let r = supported(&d);
        if let Some(removed) = removed {
            assert_eq!(value(&r, "employer_loan_interest_removed"), removed);
        }
        assert_eq!(value(&r, "adjustments"), deduction, "{interest}/{employer}");
        assert_eq!(value(&r, "agi"), agi, "{interest}/{employer}");
        if employer == 120_000 {
            d.adjustments.as_mut().unwrap().student_loan =
                employer_paid(loan_claim(vec![box_1(120_000)], 0), 120_001);
            unsupported(&d, "Student loan interest");
        }
    }
}
