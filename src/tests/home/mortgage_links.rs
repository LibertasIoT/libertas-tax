use super::*;

fn root(id: i64, home: MortgageHome, principal: i64, interest: i64) -> MortgageLoan {
    MortgageLoan {
        printed: None,
        id,
        property: "Synthetic home".into(),
        lender: MortgageLender::Institution,
        interest,
        outstanding_principal: principal,
        incurred: 20200101,
        details: MortgageLoanDetails::FirstLender(MortgageLedger {
            home,
            purpose: MortgageUse::Home,
            average: MortgageAverageBalance::Statements { average_balance: principal },
        }),
        refund_of_interest: MortgageInterestRefund::No,
        mortgage_insurance_premiums: 0,
        points: MortgagePoints::No,
    }
}
fn transferred(id: i64, original: &MortgageLoan, interest: i64) -> MortgageLoan {
    let mut loan = original.clone();
    loan.id = id;
    loan.details = MortgageLoanDetails::EarlierLender { loan: original.id };
    loan.interest = interest;
    loan.outstanding_principal = 1;
    loan.points = MortgagePoints::No;
    loan
}
fn ledger(loan: &mut MortgageLoan) -> &mut MortgageLedger {
    match &mut loan.details {
        MortgageLoanDetails::FirstLender(ledger) => ledger,
        _ => panic!("fixture canonical loan"),
    }
}

#[test]
fn transferred_lenders_use_the_same_monthly_mixed_use_allocation_once() {
    let mut original = root(10, MortgageHome::Main, 10_000_000, 160_000);
    ledger(&mut original).purpose = MortgageUse::Partly(MortgageSplit {
        improvement: 8_000_000,
        improvement_incurred: 20200101,
        other: 2_000_000,
    });
    ledger(&mut original).average = MortgageAverageBalance::Monthly {
        balances: vec![
            MortgageMonthBalance { month: 1, balance: 9_000_000 },
            MortgageMonthBalance { month: 2, balance: 7_000_000 },
        ],
    };
    let second = transferred(11, &original, 320_000);
    let third = transferred(12, &original, 160_000);
    let loans = vec![original.clone(), second.clone(), third.clone()];
    assert_eq!(crate::mortgage_limit::graph_rule(&loans), None);
    let projections = crate::mortgage_limit::loan_allowances(&loans, false);
    assert_eq!(
        projections.iter().map(|p| p.qualified_interest_cents).collect::<Vec<_>>(),
        vec![150_000, 300_000, 150_000]
    );
    assert!(projections.iter().all(|p| p.canonical_id == 10));
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    ded.mortgage = mortgage_loan_sources(loans);
    let result = supported(&d);
    assert_eq!(value(&result, "mortgage_average_balance"), 12_500);
    assert_eq!(value(&result, "mortgage_interest_8a"), 6_000);
    original.property = "Renamed property".into();
    let reordered = crate::mortgage_limit::loan_allowances(&[third, original, second], false);
    for p in projections {
        assert_eq!(reordered.iter().find(|v| v.id == p.id), Some(&p));
    }
}

#[test]
fn transferred_family_shares_combined_and_separate_return_debt_limits() {
    let original = root(10, MortgageHome::Main, 75_000_000, 3_000_000);
    let second = transferred(11, &original, 1_000_000);
    let other = root(12, MortgageHome::Second, 75_000_000, 4_000_000);
    let loans = [original, second, other];
    assert_eq!(
        crate::mortgage_limit::loan_allowances(&loans, false)
            .iter()
            .map(|p| p.regular_interest_cents)
            .sum::<i64>(),
        4_000_000
    );
    assert_eq!(
        crate::mortgage_limit::loan_allowances(&loans, true)
            .iter()
            .map(|p| p.regular_interest_cents)
            .sum::<i64>(),
        2_000_000
    );
}

#[test]
fn interest_over_rate_uses_all_lenders_interest_for_one_ledger() {
    let mut original = root(10, MortgageHome::Main, 100_000_000, 3_000_000);
    ledger(&mut original).average = MortgageAverageBalance::InterestOverRate { rate: 6.0 };
    let second = transferred(11, &original, 3_000_000);
    let allowances = crate::mortgage_limit::loan_allowances(&[original, second], false);
    assert_eq!(allowances.iter().map(|a| a.regular_interest_cents).sum::<i64>(), 4_500_000);
}

#[test]
fn transfer_graph_rejects_stale_self_duplicate_cyclic_and_mismatched_targets() {
    use crate::mortgage_limit::graph_rule;
    let original = root(10, MortgageHome::Main, 10_000_000, 100_000);
    let second = transferred(11, &original, 100_000);
    assert_eq!(
        graph_rule(core::slice::from_ref(&second)),
        Some("MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED")
    );
    assert_eq!(graph_rule(&[original.clone(), original.clone()]), Some("MORTGAGE_LOAN_ID_INVALID"));
    let mut bad = second.clone();
    bad.details = MortgageLoanDetails::EarlierLender { loan: 11 };
    assert_eq!(graph_rule(&[original.clone(), bad]), Some("MORTGAGE_LOAN_SELF_REFERENCE"));
    let mut changed = original.clone();
    changed.details = MortgageLoanDetails::EarlierLender { loan: 11 };
    assert_eq!(
        graph_rule(&[changed, second.clone()]),
        Some("MORTGAGE_EARLIER_LENDER_ENTRY_NEEDED")
    );
    let mut bad = second;
    bad.incurred = 20210101;
    assert_eq!(graph_rule(&[original, bad]), Some("MORTGAGE_LOAN_ORIGINATION_DIFFERS"));
}

#[test]
fn rental_projection_keeps_acquisition_interest_when_personal_home_is_not_qualified() {
    let original = root(10, MortgageHome::NotQualified, 10_000_000, 600_000);
    let second = transferred(11, &original, 400_000);
    let projected = crate::mortgage_limit::loan_allowances(&[original, second], false);
    assert_eq!(projected.iter().map(|a| a.qualified_interest_cents).sum::<i64>(), 1_000_000);
    assert_eq!(projected.iter().map(|a| a.regular_interest_cents).sum::<i64>(), 0);
    assert_eq!(projected.iter().map(|a| a.amt_interest_cents).sum::<i64>(), 0);
}

// Pub. 527 "Points": the rental share is spread evenly over the loan term,
// including a main home's improvement share; purchase points of a main home
// keep the year-paid rule (section 461(g)(2)).
#[test]
fn rental_points_are_straight_lined_except_a_main_home_purchase() {
    let mut original = root(10, MortgageHome::Main, 10_000_000, 600_000);
    original.incurred = 20260101;
    original.points = MortgagePoints::Purchase { amount: 150_000 };
    original.mortgage_insurance_premiums = 120_000;
    let p =
        crate::mortgage_limit::loan_allowances(core::slice::from_ref(&original), false).remove(0);
    assert_eq!(p.qualified_points_cents, 150_000);
    assert_eq!(p.regular_points_cents, 150_000);
    assert_eq!(p.rental_points_cents, 150_000);
    assert_eq!(p.qualified_insurance_cents, 120_000);
    assert_eq!(p.regular_insurance_cents, 120_000);
    let rental = |loan: &MortgageLoan| {
        crate::mortgage_limit::loan_allowances(core::slice::from_ref(loan), false)[0]
            .rental_points_cents
    };
    original.incurred = 20200101;
    assert_eq!(rental(&original), 0);
    original.incurred = 20260101;
    original.points = MortgagePoints::Amortized { amount: 150_000, term_months: 360, ended: false };
    assert_eq!(rental(&original), 5_000);
    original.points = MortgagePoints::Improvement { amount: 150_000, term_months: 360 };
    let p =
        crate::mortgage_limit::loan_allowances(core::slice::from_ref(&original), false).remove(0);
    assert_eq!(p.regular_points_cents, 150_000);
    assert_eq!(p.rental_points_cents, 5_000);
}

#[test]
fn rental_projection_traces_points_and_premiums_before_personal_vehicle_amt_exclusion() {
    let mut original = root(10, MortgageHome::SecondVehicle, 10_000_000, 600_000);
    ledger(&mut original).purpose = MortgageUse::Partly(MortgageSplit {
        improvement: 8_000_000,
        improvement_incurred: 20200101,
        other: 2_000_000,
    });
    ledger(&mut original).average = MortgageAverageBalance::WithinLimit;
    original.points = MortgagePoints::Amortized { amount: 150_000, term_months: 360, ended: false };
    original.mortgage_insurance_premiums = 120_000;
    let p = crate::mortgage_limit::loan_allowances(&[original], false).remove(0);
    assert_eq!(p.qualified_interest_cents, 480_000);
    assert_eq!(p.regular_points_cents, 4_000);
    assert_eq!(p.rental_points_cents, 4_000);
    assert_eq!(p.qualified_insurance_cents, 96_000);
    assert_eq!(p.regular_insurance_cents, 96_000);
    assert_eq!(p.amt_points_cents, 0);
    assert_eq!(p.amt_insurance_cents, 0);
}

#[test]
fn unreported_purchase_points_remain_on_schedule_a_8c() {
    let mut original = root(10, MortgageHome::Main, 10_000_000, 600_000);
    original.incurred = 20260101;
    original.lender = MortgageLender::NoForm { name: "Synthetic lender".into() };
    original.points = MortgagePoints::Purchase { amount: 150_000 };
    let p =
        crate::mortgage_limit::loan_allowances(core::slice::from_ref(&original), false).remove(0);
    assert!(!p.points_reported_on_1098);
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    ded.mortgage = vec![mortgage_loan_source(original)];
    let result = supported(&d);
    assert_eq!(value(&result, "mortgage_interest_8a"), 0);
    assert_eq!(value(&result, "mortgage_interest_8b"), 6_000);
    assert_eq!(value(&result, "mortgage_points_8c"), 1_500);
}

#[test]
fn mortgage_insurance_phaseout_preserves_cents_before_rental_allocation() {
    use crate::mortgage_limit::{insurance_phaseout, insurance_phaseout_cents};
    assert_eq!(insurance_phaseout_cents(150, 0, false), 150);
    assert_eq!(insurance_phaseout_cents(150, 100_001, false), 135);
    assert_eq!(insurance_phaseout_cents(150, 50_001, true), 135);
    assert_eq!(insurance_phaseout_cents(150, 110_001, false), 0);
    assert_eq!(insurance_phaseout(150, 0, false), 2);
}
