use super::*;

fn source_rental(rent_cents: i64, expense_cents: i64) -> FederalRental {
    let mut value = rental(0, 0);
    value.rent = rent_cents;
    value.expenses[0].amount = expense_cents;
    // Actual all-land basis makes this control's depreciation exactly zero.
    value.buildings[0].cost = 100_000;
    value.buildings[0].land = 100_000;
    value.property_type = RentalPropertyType::Land;
    value
}
fn add_rental(draft: &mut FederalDraft, value: FederalRental) -> i64 {
    add(draft, income(FederalIncome::Rental { data: value }));
    draft.income.last().unwrap().id
}
fn stages(draft: &FederalDraft, alternative: bool) -> crate::extended_rules::RentalResult {
    crate::extended_rules::rentals_for_home_with_care(
        draft,
        200_000,
        &mut FederalResult::empty(draft.revision),
        alternative,
        [200_000, 200_000],
        &crate::care::CareDeductions::default(),
    )
}

#[test]
fn unit19_rental_source_stage_retains_actual_cents_and_final_filing_boundary() {
    let mut draft = complete(FederalStatus::Single);
    let first = add_rental(&mut draft, source_rental(149, 0));
    let second = add_rental(&mut draft, source_rental(49, 0));
    for alternative in [false, true] {
        let pool = stages(&draft, alternative);
        for (id, cents, filed) in [(first, 149, 1), (second, 49, 0)] {
            let source = &pool.operating_sources[&id];
            assert_eq!(source.source, id);
            assert_eq!(source.owner, Owner::Taxpayer);
            assert_eq!(source.alternative, alternative);
            assert_eq!(source.before_at_risk_cents, cents);
            assert_eq!(source.after_at_risk_cents, cents);
            assert_eq!(source.prior_at_risk_cents, 0);
            assert_eq!(source.prior_passive_cents, 0);
            assert_eq!(source.prior_home_carry_cents, [0, 0]);
            assert!(!source.home_limited && !source.excluded && !source.property_disposition);
            assert_eq!(source.filed_ordinary_dollars, Some(filed));
            assert_eq!(pool.income_by_id.get(&id), Some(&filed));
        }
    }
    let before = supported(&draft);
    draft.income.reverse();
    let after = supported(&draft);
    assert_eq!(after.tax, before.tax);
    assert_eq!(value(&after, "agi"), value(&before, "agi"));
    let reordered = stages(&draft, false);
    assert_eq!(reordered.operating_sources[&first].before_at_risk_cents, 149);
    assert_eq!(reordered.operating_sources[&second].before_at_risk_cents, 49);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn unit19_rental_source_stage_keeps_regular_amt_priors_and_pal_boundary_separate() {
    let mut draft = complete(FederalStatus::Single);
    let mut rental = source_rental(0, 249);
    rental.at_risk = simplified_rental_risk(149, 0, 0, [50, 75, 0, 0]);
    rental.history = RentalHistory { regular: 100, amt: 200, active_in_loss_year: true };
    let id = add_rental(&mut draft, rental);
    for (alternative, prior, passive, before) in [(false, 50, 100, -299), (true, 75, 200, -324)] {
        let pool = stages(&draft, alternative);
        let source = &pool.operating_sources[&id];
        assert_eq!(source.prior_at_risk_cents, prior);
        assert_eq!(source.prior_passive_cents, passive);
        assert_eq!(source.before_at_risk_cents, before);
        assert_eq!(source.after_at_risk_cents, -149);
        // The annual PAL result does not recreate this unrounded stage.
        assert_eq!(source.filed_ordinary_dollars, Some(0));
        assert_eq!(pool.income_by_id[&id], 0);
        assert_ne!(source.after_at_risk_cents, source.filed_ordinary_dollars.unwrap() * 100);
    }
}

#[test]
fn unit19_rental_source_stage_retains_real_home_context_without_prior_addback() {
    let mut draft = complete(FederalStatus::Single);
    let mut rental = source_rental(600_000, 800_000);
    rental.property_type = RentalPropertyType::SingleFamily;
    rental.buildings[0].cost = 33_000_000;
    rental.buildings[0].land = 5_500_000;
    rental.personal_use = RentalPersonalUse::Yes {
        fair_rental_days: 90,
        personal_days: 30,
        overlap_days: 0,
        direct_expenses: 0,
    };
    rental.home_operating_carryover = 25_000;
    rental.at_risk = simplified_rental_risk(100_000, 0, 0, [100_000, 200_000, 0, 0]);
    let id = add_rental(&mut draft, rental);
    for (alternative, prior) in [(false, 100_000), (true, 200_000)] {
        let pool = stages(&draft, alternative);
        let source = &pool.operating_sources[&id];
        assert!(source.home_limited);
        assert!(!source.excluded);
        assert_eq!(source.prior_at_risk_cents, prior);
        assert_eq!(source.prior_home_carry_cents, [25_000, 0]);
        assert_eq!(source.before_at_risk_cents, 0);
        assert_eq!(source.after_at_risk_cents, 0);
        assert_eq!(source.filed_ordinary_dollars, Some(0));
    }
}

#[test]
fn unit19_rental_source_stage_marks_property_boundary_and_preserves_missing_invocation() {
    let mut draft = complete(FederalStatus::Single);
    draft.screening.as_mut().unwrap().section1231 = Some(section1231_history(&[], &[]));
    let mut property = source_rental(149, 0);
    property.sale = RentalSale::Sold {
        date: 20261001,
        price: 200_000,
        land_price: 200_000,
        selling_expenses: 0,
        payment: SalePayment::InFull,
        buyer: crate::Buyer::Unrelated,
        personal_share_percent: 0,
    };
    let property_id = add_rental(&mut draft, property);
    let mut missing = source_rental(149, 0);
    missing.buildings.clear();
    let missing_id = add_rental(&mut draft, missing);
    for alternative in [false, true] {
        let pool = stages(&draft, alternative);
        let source = &pool.operating_sources[&property_id];
        assert!(source.property_disposition);
        assert_eq!(source.before_at_risk_cents, 149);
        assert_eq!(
            source.filed_ordinary_dollars, None,
            "an incomplete pool cannot supply a final annual source"
        );
        // No property cents or zero-valued substitute is produced here.
        assert!(!pool.operating_sources.contains_key(&missing_id));
    }
}

#[test]
fn rental_expense_carrier_keeps_grouped_meals_and_item_category_rounding() {
    use crate::kernel::ratio::exact;
    let expenses = vec![
        RentalExpense {
            category: RentalExpenseCategory::Utilities,
            amount: 1,
            treatment: BusinessExpenseKind::Meals,
        };
        2
    ];
    assert_eq!(RentalExpense::total_exact(&expenses), exact::whole(1));
    assert_eq!(RentalExpense::category_totals_exact(&expenses)[13], exact::whole(2));
}

#[test]
fn rental_wide_home_expenses_pass_through_days_and_home_limits_before_projection() {
    let mut draft = complete(FederalStatus::Single);
    let mut rental = source_rental(100_000, i64::MAX);
    rental.expenses.push(rental.expenses[0].clone());
    rental.personal_use = RentalPersonalUse::Yes {
        fair_rental_days: 15,
        personal_days: 350,
        overlap_days: 0,
        direct_expenses: 0,
    };
    assert_eq!(crate::extended_rules::rental_net(&rental), Ok(0));
    let id = add_rental(&mut draft, rental);
    let mut report = FederalResult::empty(draft.revision);
    let pool = crate::extended_rules::rentals(&draft, 200_000, &mut report);
    assert!(report.issues.is_empty(), "{:?}", report.issues);
    assert_eq!(pool.income_by_id[&id], 0);
    for (suffix, expected) in
        [("home_operating_carryover", 758_085_372_892_073_354), ("expense_line19_other", 100_000)]
    {
        assert_eq!(line_amount(&report, &alloc::format!("rental_{id}_{suffix}")), expected);
    }
}

#[test]
fn rental_expense_native_output_failure_withholds_preview_and_preserves_source_priority() {
    let mut rental = source_rental(0, i64::MAX);
    rental.expenses.push(rental.expenses[0].clone());
    assert!(crate::extended_rules::rental_source_valid(&rental));
    assert_eq!(
        crate::extended_rules::rental_net(&rental).unwrap_err().0,
        "RENTAL_EXPENSE_AMOUNT_UNAVAILABLE"
    );
    assert_eq!(
        crate::interview::document_amount(&FederalIncome::Rental { data: rental.clone() }, false),
        None
    );
    let mut draft = complete(FederalStatus::Single);
    add_rental(&mut draft, rental.clone());
    let report = estimate(&draft);
    assert!(
        report.issues.iter().any(|issue| issue.display.key == "RENTAL_EXPENSE_AMOUNT_UNAVAILABLE")
    );
    assert_eq!(report.tax, None);
    assert!(!report.lines.iter().any(|line| line.id == "agi"));
    rental.expenses.push(RentalExpense { amount: -1, ..rental.expenses[0].clone() });
    assert_eq!(
        crate::extended_rules::rental_net(&rental).unwrap_err().0,
        "ISSUE_RENTAL_BUILDINGS_OR_EXPENSES_OUT_OF_LIMITS"
    );
}

#[test]
fn rental_all_risk_does_not_require_a_form6198_positive_loss_carrier() {
    let mut rental = source_rental(0, 0);
    let all = crate::extended_rules::rental_risk(&rental, i64::MIN, 0, false).unwrap();
    assert_eq!(all.ordinary, i64::MIN);
    rental.at_risk = simplified_rental_risk(0, 0, 0, [0; 4]);
    assert_eq!(
        crate::extended_rules::rental_risk(&rental, i64::MIN, 0, false).unwrap_err().key(),
        "ACTIVITY_LOSS_AMOUNT_UNREPRESENTABLE"
    );
}
