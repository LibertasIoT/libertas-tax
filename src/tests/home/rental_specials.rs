// B10-3 fixtures A–H, checked against Pub. 527/925, IRC 613/613A and Form 6198.
use super::*;
use crate::tests::activity::rental_specials::{amount, entry, expense, home_fixture, put, wages};
#[test]
fn fixture_g_home_itemizer_and_standard_routes() {
    let mut d = wages(60000);
    let mut home = home_fixture();
    link_home_fixture(&mut d, &mut home);
    let id = put(&mut d, home);
    d.deductions.as_mut().unwrap().choice =
        DeductionSelection::Itemized { facts: itemized(d.deductions.as_mut().unwrap()).clone() };
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), -5167);
    assert_eq!(amount(&r, id, "home_operating_carryover"), 2000);
    assert_eq!(amount(&r, id, "home_depreciation_carryover"), 6667);
    assert_eq!(
        crate::extended_rules::home_personal_shares(&d),
        [crate::kernel::ratio::exact::whole(0), crate::kernel::ratio::exact::whole(133333)]
    );
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), 0);
    assert_eq!(amount(&r, id, "home_operating_carryover"), 7167);
    assert_eq!(amount(&r, id, "home_depreciation_carryover"), 6667);
}
#[test]
fn fixture_h_home_prior_passive_balance_unchanged() {
    let mut d = wages(60000);
    let mut v = home_fixture();
    v.rent = 20000 * 100;
    v.expenses = vec![expense(RentalExpenseCategory::Utilities, 18500)];
    if let RentalPersonalUse::Yes { direct_expenses, .. } = &mut v.personal_use {
        *direct_expenses = 0;
    }
    v.history = RentalHistory { regular: 3000 * 100, amt: 3000 * 100, active_in_loss_year: true };
    let id = put(&mut d, v);
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), 1000);
    assert_eq!(amount(&r, id, "suspended_loss"), 3000);
    assert_eq!(value(&r, &alloc::format!("amt_rental_{id}_suspended_loss")), 3000);
    assert_eq!(value(&r, "agi"), 61000);
    assert!(
        !r.lines
            .iter()
            .any(|l| { l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8582) })
    );
}

fn linked_property(rent: i64, interest: i64) -> (FederalDraft, i64) {
    let mut d = wages(60000);
    let mut v = rental(rent, 0);
    v.buildings[0].cost = v.buildings[0].land; // Focus this worksheet on loan costs.
    v.expenses = vec![expense(RentalExpenseCategory::MortgageInterest, interest)];
    v.personal_use = RentalPersonalUse::Yes {
        fair_rental_days: 60,
        personal_days: 60,
        overlap_days: 0,
        direct_expenses: 0,
    };
    link_home_fixture(&mut d, &mut v);
    let id = put(&mut d, v);
    let ded = d.deductions.as_mut().unwrap();
    ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
    (d, id)
}
fn linked_ledger(d: &mut FederalDraft) -> &mut MortgageLedger {
    match &mut d.deductions.as_mut().unwrap().mortgage[0].details {
        MortgageLoanDetails::FirstLender(v) => v,
        _ => unreachable!(),
    }
}
#[test]
fn linked_home_interest_uses_acquisition_tracing_debt_cap_and_one_personal_share() {
    let (mut d, id) = linked_property(100, 4000);
    let loan = &mut d.deductions.as_mut().unwrap().mortgage[0];
    loan.outstanding_principal = 1500000 * 100;
    linked_ledger(&mut d).average =
        MortgageAverageBalance::FirstAndLast { last_balance: 1500000 * 100 };
    linked_ledger(&mut d).purpose = MortgageUse::Partly(MortgageSplit {
        improvement: 1000000 * 100,
        improvement_incurred: 20200601,
        other: 500000 * 100,
    });
    let p = crate::mortgage_limit::loan_allowances(&d.deductions.as_ref().unwrap().mortgage, false)
        .remove(0);
    assert_eq!(p.qualified_interest_cents, 266667);
    assert_eq!(p.regular_interest_cents, 200000);
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), -900);
    assert_eq!(amount(&r, id, "home_operating_carryover"), 333);
    assert_eq!(value(&r, "mortgage_interest_8a"), 1000);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), 0);
    assert_eq!(amount(&r, id, "home_operating_carryover"), 1233);
    let ded = d.deductions.as_mut().unwrap();
    let facts = itemized(ded).clone();
    ded.choice = DeductionSelection::Itemized { facts };
    linked_ledger(&mut d).home = MortgageHome::NotQualified;
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), 0);
    assert_eq!(amount(&r, id, "home_operating_carryover"), 1233);
    assert_eq!(value(&r, "mortgage_interest_8a"), 0);
    if let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income {
        data.personal_use = RentalPersonalUse::Yes {
            fair_rental_days: 14,
            personal_days: 60,
            overlap_days: 0,
            direct_expenses: 0,
        };
    }
    let r = supported(&d);
    assert_eq!(amount(&r, id, "home_operating_carryover"), 0);
    assert_eq!(value(&r, "agi"), 60000);
}
// C1: the rental share of points is the loan's own 2026 amount (year-paid
// purchase points of a main home; otherwise straight line over the term).
#[test]
fn linked_home_points_use_otherwise_allowable_annual_interest_tier() {
    for (incurred, points, expected_fixed, personal_line, personal_amount, noted) in [
        (
            20260101,
            MortgagePoints::Purchase { amount: 600000 },
            3000,
            "mortgage_interest_8a",
            3000,
            false,
        ),
        (
            20200101,
            MortgagePoints::Purchase { amount: 600000 },
            0,
            "mortgage_interest_8a",
            0,
            false,
        ),
        (
            20200101,
            MortgagePoints::Amortized { amount: 600000, term_months: 360, ended: false },
            100,
            "mortgage_points_8c",
            100,
            true,
        ),
    ] {
        let (mut d, id) = linked_property(0, 0);
        let loan = &mut d.deductions.as_mut().unwrap().mortgage[0];
        loan.incurred = incurred;
        loan.points = points;
        let r = supported(&d);
        assert_eq!(
            r.review_notes.iter().any(|n| n.display.key == "WARN_RENTAL_POINTS_STRAIGHT_LINE"),
            noted
        );
        // The rental share of the points is personally allowable in full, so
        // it is all line 2a; the non-itemizer carries it under line 3 = 0.
        assert_eq!(amount(&r, id, "line21_income"), -expected_fixed);
        assert_eq!(amount(&r, id, "home_operating_carryover"), 0);
        assert_eq!(value(&r, personal_line), personal_amount);
        d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
        let r = supported(&d);
        assert_eq!(amount(&r, id, "line21_income"), 0);
        assert_eq!(amount(&r, id, "home_operating_carryover"), expected_fixed);
    }
}
#[test]
fn linked_home_duplicate_or_dangling_roots_are_refused() {
    let (mut d, _) = linked_property(0, 0);
    let loan = &mut d.deductions.as_mut().unwrap().mortgage[0];
    loan.points = MortgagePoints::Purchase { amount: 600000 };
    assert_eq!(crate::extended_rules::rental_mortgage_rule(&d), None);
    let FederalIncome::Rental { data } = d.income.last().unwrap().income.clone() else {
        unreachable!()
    };
    put(&mut d, data);
    assert_eq!(crate::extended_rules::rental_mortgage_rule(&d), Some("RENTAL_MORTGAGE_ROOT_TWICE"));
    d.income.pop();
    d.deductions.as_mut().unwrap().mortgage.clear();
    assert_eq!(
        crate::extended_rules::rental_mortgage_rule(&d),
        Some("MORTGAGE_RENTAL_ROOT_NEEDED")
    );
}
#[test]
fn linked_home_premiums_keep_cents_and_use_final_agi_after_rental_income() {
    let (mut d, id) = linked_property(0, 0);
    d.deductions.as_mut().unwrap().mortgage[0].mortgage_insurance_premiums = 150;
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), -1);
    assert_eq!(amount(&r, id, "home_operating_carryover"), 0);
    assert_eq!(value(&r, "mortgage_insurance_premiums_8d"), 1);
    assert_eq!(line_amount(&r, &alloc::format!("rental_{id}_line21_income")), -75);
    assert!(
        r.lines
            .iter()
            .filter(|l| l.id.starts_with(&alloc::format!("rental_{id}_expense_")))
            .all(|l| l.amount >= 0)
    );
    let (mut d, id) = linked_property(10000, 0);
    d.income[0] = wage(1, Owner::Taxpayer, 95000 * 100, 0);
    d.deductions.as_mut().unwrap().mortgage[0].mortgage_insurance_premiums = 2000 * 100;
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 104000);
    assert_eq!(value(&r, "mortgage_insurance_premiums_8d"), 600);
    assert_eq!(amount(&r, id, "line21_income"), 9000);
}
#[test]
fn linked_home_accepts_more_than_one_hundred_authored_expense_rows() {
    let (mut d, id) = linked_property(1000, 0);
    if let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income {
        data.expenses = (0..101)
            .map(|index| RentalExpense {
                category: RentalExpenseCategory::Other { label: alloc::format!("Expense {index}") },
                amount: if index < 99 { 100 } else { 50 },
                treatment: BusinessExpenseKind::Ordinary,
            })
            .collect();
    }
    d.deductions.as_mut().unwrap().mortgage[0].mortgage_insurance_premiums = 100;
    let d = save_reopen_native_sources(d, &[TaxSection::Rentals], &[TaxSection::Rentals]);
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), 950);
}
#[test]
fn linked_home_premium_phaseout_uses_actual_agi_with_foreign_exclusion() {
    let (mut d, id) = linked_property(0, 0);
    d.deductions.as_mut().unwrap().mortgage[0].mortgage_insurance_premiums = 2000 * 100;
    entry(&mut d, foreign_pay(120000));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 59000);
    assert_eq!(amount(&r, id, "line21_income"), -1000);
    assert_eq!(value(&r, "mortgage_insurance_premiums_8d"), 1000);
}
#[test]
fn linked_home_purchase_points_annual_tier_obeys_shared_debt_cap() {
    let (mut d, id) = linked_property(0, 0);
    let loan = &mut d.deductions.as_mut().unwrap().mortgage[0];
    loan.incurred = 20260101;
    loan.outstanding_principal = 1500000 * 100;
    loan.points = MortgagePoints::Purchase { amount: 600000 };
    linked_ledger(&mut d).average =
        MortgageAverageBalance::Statements { average_balance: 1500000 * 100 };
    // Rental share 3,000 of the year-paid points; 1,500 of it is personally
    // allowable under the $750,000 cap (line 2a), the rest waits on line 4b.
    let r = supported(&d);
    assert_eq!(amount(&r, id, "line21_income"), -1500);
    assert_eq!(amount(&r, id, "home_operating_carryover"), 1500);
    assert_eq!(value(&r, "mortgage_interest_8a"), 1500);
}
#[test]
fn linked_home_standard_uses_same_regular_magi_for_amt_passive_allowance() {
    let (mut d, _) = linked_property(0, 40000);
    d.income[0] = wage(1, Owner::Taxpayer, 130000 * 100, 0);
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let id = put(&mut d, rental(18000, 38000));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 120000);
    assert_eq!(amount(&r, id, "suspended_loss"), 20000);
    assert_eq!(value(&r, &alloc::format!("amt_rental_{id}_suspended_loss")), 20000);
}
#[test]
fn linked_home_personal_interest_and_points_round_after_removing_rental_cents() {
    let (mut d, id) = linked_property(0, 0);
    let loan = &mut d.deductions.as_mut().unwrap().mortgage[0];
    loan.interest = 100;
    loan.incurred = 20260101;
    loan.points = MortgagePoints::Purchase { amount: 100 };
    let r = supported(&d);
    // The two personal half-dollar components join one line before its one rounding.
    assert_eq!(value(&r, "mortgage_interest_8a"), 1);
    assert_eq!(line_amount(&r, &alloc::format!("rental_{id}_line21_income")), -100);
}

// C2: a certificate names its saved loan; section 163(g) reduces that
// loan's interest deduction, the rental share of it on Schedule E.
#[test]
fn linked_home_mortgage_credit_reduces_the_certified_loan_in_both_shares() {
    let (mut d, id) = linked_property(2000, 10000);
    d.credits.as_mut().unwrap().mortgage_credit =
        Some(mortgage_certificate_credit(2000, 10000, 1000000));
    // Without the loan named, Schedule A bears the whole $2,000 credit.
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_credit_current_potential"), 2000);
    assert_eq!(amount(&r, id, "expense_line12_mortgage_interest"), 5000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 3000);
    // Named: the rental share (60 of 120 days) of the credit leaves line 12.
    d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().loan = Some(101);
    let r = supported(&d);
    assert_eq!(amount(&r, id, "expense_line12_mortgage_interest"), 4000);
    assert_eq!(value(&r, "mortgage_interest_8a"), 4000);
    assert_eq!(amount(&r, id, "line21_income"), -2000);
    // Non-itemizer: the reduced rental share is limited by the rent (line 4b).
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Standard;
    let r = supported(&d);
    assert_eq!(amount(&r, id, "expense_line12_mortgage_interest"), 2000);
    assert_eq!(amount(&r, id, "home_operating_carryover"), 2000);
    assert_eq!(amount(&r, id, "line21_income"), 0);
}

// D5: a linked second home that fails the section 280A(d)(1) residence test
// is not a qualified residence (section 163(h)(4)(A)(i)(II)); its personal
// interest share is nondeductible, while the rental share stays on line 12.
#[test]
fn second_home_failing_the_residence_test_has_no_personal_interest_share() {
    let (mut d, id) = linked_property(20000, 12000);
    linked_ledger(&mut d).home = MortgageHome::Second;
    let days = |d: &mut FederalDraft, personal_days| {
        let FederalIncome::Rental { data } = &mut d.income.last_mut().unwrap().income else {
            unreachable!()
        };
        data.personal_use = RentalPersonalUse::Yes {
            fair_rental_days: 200,
            personal_days,
            overlap_days: 0,
            direct_expenses: 0,
        };
    };
    days(&mut d, 10);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_interest_8a"), 0);
    assert_eq!(amount(&r, id, "expense_line12_mortgage_interest"), 11429);
    // 21 personal days exceed 10% of 200 fair-rental days: a residence again.
    days(&mut d, 21);
    let r = supported(&d);
    assert_eq!(value(&r, "mortgage_interest_8a"), 1140);
    assert_eq!(amount(&r, id, "expense_line12_mortgage_interest"), 10860);
    // A main home needs no day test.
    days(&mut d, 10);
    linked_ledger(&mut d).home = MortgageHome::Main;
    assert_eq!(value(&supported(&d), "mortgage_interest_8a"), 571);
}
