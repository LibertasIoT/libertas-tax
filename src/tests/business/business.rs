use super::*;
use crate::business_rules::{QbiActivity, qbi};
use crate::tests::support::builders::survey::survey_cash_business_data;
#[test]
fn self_employed_tips_use_each_business_net_income_and_share_the_return_cap() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut first } = cash_business(10_000) else { panic!() };
    first.tips = Some(FederalBusinessTips { amount: 1_000_000, eligible: true });
    add(&mut draft, income(FederalIncome::Business { data: first }));
    let mut no_tips = draft.clone();
    let FederalIncome::Business { data } = &mut no_tips.income[1].income else { panic!() };
    data.tips = None;
    let baseline = supported(&no_tips);
    let result = supported(&draft);
    assert_eq!(value(&result, "business_2_qualified_tips"), 9293);
    assert_eq!(value(&result, "tips_deduction"), 9293);
    assert_eq!(value(&result, "agi"), value(&baseline, "agi"));
    assert_eq!(value(&result, "self_employment_tax"), value(&baseline, "self_employment_tax"));
    assert_eq!(restart(draft.clone()).draft, draft);

    // An unprofitable tipped activity cannot borrow the first activity's profit.
    let FederalIncome::Business { data: mut second } = cash_business(1_000) else { panic!() };
    second.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 1_000_000,
        accounting: BusinessAccounting::Cash,
    };
    second.expenses.push(ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Operating costs".into() },
        1_000_000,
    ));
    second.tips = Some(FederalBusinessTips { amount: 300_000, eligible: true });
    add(&mut draft, income(FederalIncome::Business { data: second }));
    let result = supported(&draft);
    assert_eq!(value(&result, "business_3_qualified_tips"), 0);
    assert_eq!(value(&result, "tips_deduction"), 9293);

    let FederalIncome::Wage { data: wages } = &mut draft.income[0].income else { panic!() };
    let mut wages = edit_wage(wages);
    wages.work_deductions = work_deductions(2_000_000, 0);
    drop(wages);
    let result = supported(&draft);
    assert_eq!(value(&result, "tips_deduction"), 25000);
}

#[test]
fn reported_1099_overtime_shares_the_w2_cap_without_duplicating_business_income() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = cash_business(20_000) else { panic!() };
    business.overtime = Some(FederalBusinessOvertime { amount: 1_000_000, eligible: true });
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let mut no_overtime = draft.clone();
    let FederalIncome::Business { data } = &mut no_overtime.income[1].income else { panic!() };
    data.overtime = None;
    let baseline = supported(&no_overtime);
    let result = supported(&draft);
    assert_eq!(value(&result, "business_2_qualified_overtime"), 10_000);
    assert_eq!(value(&result, "overtime_deduction"), 10_000);
    assert_eq!(value(&result, "agi"), value(&baseline, "agi"));
    assert_eq!(value(&result, "self_employment_tax"), value(&baseline, "self_employment_tax"));
    assert_eq!(restart(draft.clone()).draft, draft);

    let FederalIncome::Wage { data } = &mut draft.income[0].income else { panic!() };
    let mut data = edit_wage(data);
    data.work_deductions = work_deductions(0, 500_000);
    drop(data);
    assert_eq!(value(&supported(&draft), "overtime_deduction"), 12_500);
}

#[test]
fn reported_1099_overtime_requires_source_and_ssn() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = cash_business(20_000) else { panic!() };
    business.overtime = Some(FederalBusinessOvertime { amount: 500_000, eligible: false });
    add(&mut draft, income(FederalIncome::Business { data: business }));
    // A negative eligibility answer leaves out the deduction with its notice.
    assert!(noted(&supported(&draft), "WARN_BUSINESS_OVERTIME_LEFT_OUT"));
    {
        let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
        data.overtime.as_mut().unwrap().eligible = true;
        data.overtime.as_mut().unwrap().amount = 2_100_000;
    }
    unsupported(&draft, "1099 overtime");
    let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
    data.overtime.as_mut().unwrap().amount = 500_000;
    draft.people.as_mut().unwrap().taxpayer.valid_ssn = false;
    unsupported(&draft, "1099 overtime");
}

#[test]
fn business_expenses_are_totaled_once_and_preview_matches_return() {
    let business = FederalBusiness {
        interest_limit_election: None,
        activity: BusinessActivity::Ordinary {
            returns: 0,
            receipts: 6_000_000,
            accounting: BusinessAccounting::Cash,
        },
        expenses: vec![
            ordinary_business_expense(
                BusinessExpenseCategory::Other { label: "Supplies".into() },
                600_000,
            ),
            ordinary_business_expense(
                BusinessExpenseCategory::Other { label: "Advertising".into() },
                400_000,
            ),
        ],
        ..survey_cash_business_data()
    };
    assert_eq!(business.expense_total(false), 1_000_000);
    let mut draft = empty_return(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Business { data: business.clone() }));
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 46_467);
    let mut invalid = business;
    let BusinessExpenseCategory::Other { label } = &mut invalid.expenses[0].category else {
        panic!()
    };
    label.clear();
    assert!(
        crate::federal_rules::validate_income_amount(&FederalIncome::Business { data: invalid })
            .is_err()
    );
}

#[test]
fn business_payroll_long_utf8_labels_remain_unique_source_keys() {
    // D-2026-10-15-13 removes the native descriptive maximum. Payroll
    // labels remain nonblank, unique source keys for the employer-credit picker.
    let payroll = |labels: &[alloc::string::String]| BusinessPayroll::Yes {
        statements: labels
            .iter()
            .map(|label| PayrollW2 { label: label.clone(), wages: 100_000, medicare: 100_000 })
            .collect(),
        attribution: crate::PayrollQbiAttribution::EntireBusiness,
    };
    assert!(payroll(&["a".repeat(80)]).valid());
    assert!(payroll(&["a".repeat(81)]).valid());
    assert!(payroll(&["é".repeat(40)]).valid());
    assert!(payroll(&["é".repeat(40) + "a"]).valid());
    assert!(payroll(&["é".repeat(80)]).valid());
    assert!(payroll(&["😀".repeat(20)]).valid());
    assert!(payroll(&["😀".repeat(20) + "a"]).valid());
    assert!(payroll(&["e\u{301}".repeat(26) + "aa"]).valid());
    assert!(payroll(&["e\u{301}".repeat(26) + "aaa"]).valid());
    for label in ["é".repeat(200), "😀".repeat(100), "e\u{301}".repeat(134)] {
        assert!(label.len() >= 400);
        assert!(payroll(core::slice::from_ref(&label)).valid());
        assert!(!payroll(&[label.clone(), label]).valid());
    }
    assert!(!payroll(&["".into()]).valid());
    assert!(payroll(&["A".into(), "B".into()]).valid());
    assert!(!payroll(&["A".into(), "A".into()]).valid());
}

#[test]
fn business_payroll_is_sourced_and_never_deducted_twice() {
    let payroll = BusinessPayroll::Yes {
        statements: vec![
            PayrollW2 { label: "A".into(), wages: 1_000_000, medicare: 2_000_000 },
            PayrollW2 { label: "B".into(), wages: 2_000_000, medicare: 1_000_000 },
        ],
        attribution: crate::PayrollQbiAttribution::EntireBusiness,
    };
    assert_eq!(payroll.qbi_wages(), 3_000_000); // Lesser of totals, not sum of lesser boxes.
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    add(
        &mut d,
        income(FederalIncome::Business {
            data: FederalBusiness {
                interest_limit_election: None,
                activity: BusinessActivity::Ordinary {
                    returns: 0,
                    receipts: 13_000_000,
                    accounting: BusinessAccounting::Cash,
                },
                expenses: vec![ordinary_business_expense(
                    BusinessExpenseCategory::Other { label: "Payroll and costs".into() },
                    3_000_000,
                )],
                payroll,
                ..survey_cash_business_data()
            },
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_deduction"), 15000);
    assert_eq!(value(&r, "agi"), 398661);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::Business { data: v } = &mut d.income[1].income else { panic!() };
    v.specified_service = true;
    assert_eq!(value(&supported(&d), "qbi_deduction"), 0);
}

#[test]
fn home_kerosene_is_a_fuel_payment_without_business_income() {
    let mut draft = complete(FederalStatus::Single);
    let baseline = supported(&draft);
    draft.payments.as_mut().unwrap().home_kerosene = Some(FederalHomeKerosene {
        gallons: 100.25,
        actual_cost: 20_000,
        qualified: true,
        not_claimed_elsewhere: true,
        records_complete: true,
    });
    let report = supported(&draft);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(report.payments, baseline.payments.map(|v| v + 2_400));
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 2_400);
    assert_eq!(line_amount(&report, "home_kerosene_fuel_4a"), 2_436);
    assert_eq!(estimate(&restart(draft).draft), report);
}

#[test]
fn business_inventory_cost_flows_through_profit_se_tax_qbi_and_restore() {
    let mut d = empty_return(FederalStatus::Single);
    let mut business = cash_business(100000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.inventory = BusinessInventory::Cost {
        method: crate::InventoryMethod::Cost,
        opening: 1_000_000,
        purchases: 3_000_000,
        personal: 100_000,
        labor: 500_000,
        materials: 200_000,
        other: 100_000,
        closing: 700_000,
        eligible: true,
    };
    assert_eq!(data.expense_total(false), 4_000_000);
    add(&mut d, income(business));
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("business_{}_cost_of_goods_sold", d.income[0].id)), 40000);
    let mut reference = d.clone();
    reference.income[0].income = cash_business(60000);
    let expected = supported(&reference);
    assert_eq!(r.tax, expected.tax);
    assert_eq!(value(&r, "qbi_deduction"), value(&expected, "qbi_deduction"));
    assert_eq!(r.return_lines, expected.return_lines);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn inventory_invalid_stock_blocks_totals_and_unconfirmed_method_is_left_out() {
    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(50000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.inventory = BusinessInventory::Cost {
        method: crate::InventoryMethod::Cost,
        opening: 0,
        purchases: 100_000,
        personal: 0,
        labor: 0,
        materials: 0,
        other: 0,
        closing: 100_001,
        eligible: true,
    };
    assert!(!data.inventory.valid());
    add(&mut d, income(business));
    assert!(!d.valid_stored());
    assert!(estimate(&d).tax.is_none());
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    if let BusinessInventory::Cost { closing, eligible, .. } = &mut data.inventory {
        *closing = 0;
        *eligible = false;
    }
    // An unsupported inventory method is left out with a notice: no cost of
    // goods sold is deducted.
    let r = supported(&d);
    assert!(noted(&r, "WARN_INVENTORY_LEFT_OUT"));
}

#[test]
fn business_expense_limits_apply_to_meals_and_exclude_entertainment() {
    let expenses = vec![
        ordinary_business_expense(
            BusinessExpenseCategory::Other { label: "Ordinary".into() },
            100_000,
        ),
        FederalBusinessExpense {
            category: BusinessExpenseCategory::Other { label: "Client meal".into() },
            amount: 100_000,
            treatment: BusinessExpenseKind::Meals,
        },
        FederalBusinessExpense {
            category: BusinessExpenseCategory::Other { label: "DOT travel".into() },
            amount: 100_000,
            treatment: BusinessExpenseKind::TransportationMeals,
        },
        FederalBusinessExpense {
            category: BusinessExpenseCategory::Other { label: "Entertainment".into() },
            amount: 100_000,
            treatment: BusinessExpenseKind::Nondeductible,
        },
    ];
    assert_eq!(FederalBusinessExpense::total(&expenses), 230_000);
    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(10000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.expenses = expenses;
    add(&mut d, income(business));
    let r = supported(&d);
    d.income[1].income = cash_business(7700);
    assert_eq!(r.tax, supported(&d).tax);
    let tiny: alloc::vec::Vec<_> = (0..100)
        .map(|_| FederalBusinessExpense {
            category: BusinessExpenseCategory::Other { label: "Meal".into() },
            amount: 1,
            treatment: BusinessExpenseKind::Meals,
        })
        .collect();
    assert_eq!(FederalBusinessExpense::total(&tiny), 50); // Aggregate before rounding.
}

#[test]
fn schedule_c_vehicle_loan_interest_is_line_16b_and_other_interest_16c_with_totals_unchanged() {
    // 2026 draft f1040sc (created 5/15/26) lines 16a mortgage, 16b "Vehicle
    // loan", 16c other; the 2026 draft i1040sc "New Line 16b" and "Line 16b"
    // report the business-use part of vehicle-loan interest there under either
    // vehicle method. Vehicle: 1,000 + 2,000 business miles of 4,000: mileage
    // $2,245 (line 9); loan interest $2,000 x 3,000 / 4,000 = $1,500 (16b);
    // personal property tax $400 x 3/4 = $300, which the standard mileage
    // rate reports on line 23 ("Standard mileage rate", line 9 instructions).
    // Lines 9 + 16b + 23 = $4,045, so net profit falls by exactly $4,045 with
    // the $500 of other interest.
    let plain = {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(cash_business(100_000)));
        d
    };
    let mut d = plain.clone();
    let FederalIncome::Business { data } = &mut d.income.last_mut().unwrap().income else {
        panic!()
    };
    data.vehicles.push(BusinessMileage {
        id: 0,
        interest: 200_000,
        property_tax: 40_000,
        ..native_business_mileage(1_000, 2_000, 4_000)
    });
    data.expenses.push(ordinary_business_expense(BusinessExpenseCategory::OtherInterest, 50_000));
    let before = supported(&plain);
    let r = supported(&d);
    let rows: alloc::vec::Vec<_> = crate::draft_return(&r)
        .iter()
        .filter(|row| row.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleC))
        .map(|row| (row.form_line.as_ref().unwrap().line.clone(), row.amount))
        .collect();
    assert!(rows.contains(&("9".into(), 224_500)), "{rows:?}");
    assert!(rows.contains(&("16b".into(), 150_000)), "{rows:?}");
    assert!(rows.contains(&("23".into(), 30_000)), "{rows:?}");
    assert!(rows.contains(&("16c".into(), 50_000)), "{rows:?}");
    assert_eq!(
        value(&before, "business_2_net_profit") - value(&r, "business_2_net_profit"),
        2_545 + 1_500 + 500
    );
}

#[test]
fn schedule_c_expense_lines_are_reported_by_category() {
    // One ordinary expense per enterable Schedule C Part II line (all lines except 9
    // car/truck, 12 depletion, 13 depreciation, 19 pension and 27a, which have no
    // `BusinessExpenseCategory` variant), $1,000 through $18,000 in $1,000 steps so
    // each line's return value is independently checkable by hand.
    let ordinary = [
        (BusinessExpenseCategory::Advertising, "line8_advertising"),
        (BusinessExpenseCategory::Commissions, "line10_commissions_and_fees"),
        (BusinessExpenseCategory::ContractLabor, "line11_contract_labor"),
        (BusinessExpenseCategory::EmployeeBenefits, "line14_employee_benefit_programs"),
        (BusinessExpenseCategory::Insurance, "line15_insurance"),
        (BusinessExpenseCategory::MortgageInterest, "line16a_mortgage_interest"),
        (BusinessExpenseCategory::OtherInterest, "line16c_other_interest"),
        (BusinessExpenseCategory::LegalAndProfessional, "line17_legal_and_professional"),
        (BusinessExpenseCategory::Office, "line18_office"),
        (BusinessExpenseCategory::VehicleAndEquipmentRent, "line20a_rent_vehicles_equipment"),
        (BusinessExpenseCategory::OtherRent, "line20b_rent_other"),
        (BusinessExpenseCategory::Repairs, "line21_repairs"),
        (BusinessExpenseCategory::Supplies, "line22_supplies"),
        (BusinessExpenseCategory::TaxesAndLicenses, "line23_taxes_and_licenses"),
        (BusinessExpenseCategory::Travel, "line24a_travel"),
        (BusinessExpenseCategory::Utilities, "line25_utilities"),
        (BusinessExpenseCategory::Wages, "line26_wages"),
    ];
    let mut expenses: alloc::vec::Vec<_> = ordinary
        .iter()
        .enumerate()
        .map(|(index, (category, _))| {
            ordinary_business_expense(category.clone(), (index as i64 + 1) * 100_000)
        })
        .collect();
    // Line 24b combines both meals treatments onto one printed line: $2,000 at the
    // 50% ordinary-meals limit ($1,000) and $1,000 at the 80% DOT limit ($800).
    expenses.push(FederalBusinessExpense {
        category: BusinessExpenseCategory::Meals,
        amount: 200_000,
        treatment: BusinessExpenseKind::Meals,
    });
    expenses.push(FederalBusinessExpense {
        category: BusinessExpenseCategory::Meals,
        amount: 100_000,
        treatment: BusinessExpenseKind::TransportationMeals,
    });
    // Line 27b combines every "Other" entry onto one printed line regardless of its
    // own free-text label: $9,000 + $9,000 = $18,000.
    for label in ["Bank fees", "Subscriptions"] {
        expenses.push(ordinary_business_expense(
            BusinessExpenseCategory::Other { label: label.into() },
            900_000,
        ));
    }
    let all_expenses = expenses.clone();

    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(300_000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.expenses = expenses;
    add(&mut d, income(business));
    let r = supported(&d);

    for (index, (_, suffix)) in ordinary.iter().enumerate() {
        let expected = (index as i64 + 1) * 1_000;
        assert_eq!(value(&r, &alloc::format!("business_2_expense_{suffix}")), expected);
    }
    assert_eq!(value(&r, "business_2_expense_line24b_meals"), 1_000 + 800);
    assert_eq!(value(&r, "business_2_expense_line27b_other"), 9_000 + 9_000);

    // Schedule C line 28 (total expenses before home office), computed by hand as the
    // sum of every entered line above: 17 ordinary lines $1,000..$17,000 step $1,000
    // (17 * 18 / 2 * $1,000 = $153,000) + line 27b's $18,000 + line 24b's $1,800.
    let line28 = 153_000 + 18_000 + 1_800;
    assert_eq!(FederalBusinessExpense::total(&all_expenses) / 100, line28);
}

// Boundary (1): form-8995.md flags the Form 8995-A Part III line 13
// formula as unverified when the wage/UBIA limit exceeds the unlimited
// 20% amount while mid-phase-in. Note: the card's own arithmetic assumed
// the pre-2026 $50,000 single phase-in range ($201,750-$251,750). The
// 2026 range enacted by P.L. 119-21 (OBBBA) and used by `qbi_phase`
// (verified in form-8995.md against the draft Form 8995-A/Schedule A) is
// $75,000 ($201,750-$276,750); this test uses that real range. At taxable
// income $226,750 the phase-in is $25,000 of that $75,000 range
// (one-third, not "halfway"), but the result below does not depend on the
// exact fraction because the wage/UBIA limit exceeds the unlimited
// amount, zeroing the reduction term regardless of the percentage.
#[test]
fn phase_in_with_the_limit_above_the_unlimited_amount_takes_the_full_deduction() {
    // One QBI activity: $200,000 QBI, $140,000 W-2 wages, no UBIA.
    //   Form 8995-A line 3 (20% of QBI): 200,000 x 20% = $40,000
    //   (unlimited amount).
    //   Lines 4-10 (wage/UBIA limit): greater of 140,000 x 50% = $70,000
    //   or 140,000 x 25% + 0 = $35,000 -> limit = $70,000.
    //   Line 13: component = 40,000 - (max(0, 40,000-70,000) x
    //   phase-in%) = 40,000 - (0 x phase-in%) = $40,000 -- the limit
    //   exceeds the unlimited amount, so the reduction term is zero
    //   regardless of the phase-in percentage.
    //   Taxable-income cap (line 36): 226,750 x 20% = $45,350, above
    //   $40,000, so it does not bind. The $400 active minimum does not
    //   bind either. Deduction: $40,000.
    let mut r = supported(&complete(FederalStatus::Single));
    let activity = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income: 200_000,
        wages: 140_000,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    };
    assert_eq!(qbi(&[activity], 226_750, 0, FederalStatus::Single, &mut r), (40_000, 0));
}

// Boundary (2): same mid-phase-in filer, but the wage/UBIA limit is below
// the unlimited amount, so the reduction term applies. See the note above
// boundary (1) about the real 2026 $75,000 single phase-in range.
#[test]
fn phase_in_reduces_the_excess_by_the_phase_in_percentage() {
    // Same filer and QBI ($200,000, unlimited amount $40,000) as boundary
    // (1), but W-2 wages $30,000: limit = max(30,000 x 50%, 30,000 x 25%)
    // = $15,000, below the unlimited amount.
    //   Excess (line 13's `positive(twenty-limit)`): 40,000-15,000 =
    //   $25,000.
    //   Phase-in percentage: taxable income $226,750 is $25,000 into the
    //   real 2026 $75,000 single range, i.e. 25,000/75,000 = 1/3 -- not
    //   the 50% the card's own arithmetic assumed under the pre-2026
    //   $50,000 range (which would have given a $27,500 deduction).
    //   Reduction: 25,000 x 25,000/75,000 = $8,333.33; the App's
    //   `fraction` helper rounds on the underlying product:
    //   (25,000 x 25,000 + 37,500) / 75,000 = 625,037,500 / 75,000 =
    //   8,333 (remainder 62,500, so it stays at 8,333).
    //   Component: 40,000 - 8,333 = $31,667.
    //   Taxable-income cap: 226,750 x 20% = $45,350, above $31,667, so it
    //   does not bind. Deduction: $31,667.
    let mut r = supported(&complete(FederalStatus::Single));
    let activity = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income: 200_000,
        wages: 30_000,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    };
    assert_eq!(qbi(&[activity], 226_750, 0, FederalStatus::Single, &mut r), (31_667, 0));
}

// Boundary (3): a specified-service business fully above the phase-out
// range gets nothing, but qualified REIT dividends are a separate
// component and still qualify.
#[test]
fn schedule_c_service_classification_computes_both_answers() {
    let estimate_with = |service: bool, wages: i64| {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, wage(0, Owner::Taxpayer, wages * 100, 0));
        let mut business = cash_business(100_000);
        let FederalIncome::Business { data } = &mut business else { unreachable!() };
        data.specified_service = service;
        add(&mut d, income(business));
        supported(&d)
    };
    assert!(estimate_with(true, 400_000).tax > estimate_with(false, 400_000).tax);
    assert_eq!(estimate_with(true, 20_000).tax, estimate_with(false, 20_000).tax);
}

#[test]
fn specified_service_above_the_range_gets_nothing_while_reit_qualifies() {
    // Taxable income $300,000 is above the 2026 single complete phase-out
    // ($201,750 + $75,000 = $276,750), so `qbi_phase`'s phased amount
    // clamps at the full $75,000 range and this SSTB's applicable
    // percentage (Schedule A Part I) is 0/75,000 = 0%:
    // `eligible_qbi_activity` scales its $150,000 QBI down to $0, so its
    // Form 8995-A component is $0.
    //   $5,000 of qualified REIT dividends is a separate component (Form
    //   8995 line 6 / 8995-A line 28), unaffected by the SSTB gate:
    //   5,000 x 20% = $1,000.
    //   The $400 active-business minimum does not apply because the
    //   SSTB's scaled-to-zero income no longer counts as active QBI
    //   ($0 < $1,000).
    //   Deduction: $0 (business) + $1,000 (REIT) = $1,000.
    let mut r = supported(&complete(FederalStatus::Single));
    let sstb = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income: 150_000,
        wages: 0,
        property: 0,
        active: true,
        service: Some(true),
        group: None,
    };
    let result = crate::business_rules::qbi_with_history(
        &[sstb],
        300_000,
        0,
        FederalStatus::Single,
        crate::business_rules::QbiOther { business_loss: 0, reit: 5_000, reit_loss: 0 },
        &mut r,
    );
    assert_eq!(result.deduction, 1_000);
    assert_eq!(result.business_loss, 0);
    assert_eq!(result.reit_loss, 0);
}

// Boundary (4): below the QBI threshold, the taxable-income cap (Form
// 8995 line 14) can still limit the deduction below the unlimited 20%
// amount (line 10).
#[test]
fn taxable_income_cap_limits_the_qbi_deduction() {
    // Taxable income $50,000 is below the 2026 single QBI threshold
    // ($201,750), so the phase-in percentage is zero and the wage/UBIA
    // limit never applies -- Form 8995's simplified path. One QBI
    // activity: $100,000, no wages/UBIA.
    //   Line 5 (20% of QBI): 100,000 x 20% = $20,000 (line 10, no REIT).
    //   Line 11 taxable income before QBI: $50,000. Line 12 (gains): $0.
    //   Line 14 (20% of line 13): 50,000 x 20% = $10,000.
    //   Line 15: smaller of line 10 ($20,000) and line 14 ($10,000) =
    //   $10,000 -- the taxable-income cap binds below the unlimited
    //   amount.
    //   Line 16 ($400 minimum): $100,000 active QBI >= $1,000, so floor
    //   $400, below line 15. Line 17: $10,000.
    let mut r = supported(&complete(FederalStatus::Single));
    let activity = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income: 100_000,
        wages: 0,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    };
    assert_eq!(qbi(&[activity], 50_000, 0, FederalStatus::Single, &mut r), (10_000, 0));
}

#[test]
fn reservist_exception_checks_order_length_and_withdrawal_window() {
    for term in [ReserveTerm::Indefinite, ReserveTerm::Days { days: 180 }] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            retirement_entry(
                Owner::Taxpayer,
                RetirementKind::Ira,
                10000,
                RetirementTreatment::Exception {
                    second: None,
                    distributed: 20260630,
                    facts: RetirementException::Reservist {
                        ordered: 20260101,
                        through: 20260630,
                        term,
                        eligible: true,
                    },
                    early_simple: false,
                },
            ),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), 0);
        assert_eq!(value(&r, "agi"), 70000);
        assert_eq!(restart(d.clone()).draft, d);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        let mut data = edit_retirement(data);
        let Some(RetirementTreatment::Exception { distributed, .. }) =
            &mut data.ordinary_distribution
        else {
            unreachable!()
        };
        *distributed = 20260701;
        drop(data);
        unsupported(&d, "Income");
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
        let mut data = edit_retirement(data);
        let Some(RetirementTreatment::Exception { distributed, facts, .. }) =
            &mut data.ordinary_distribution
        else {
            unreachable!()
        };
        *distributed = 20260630;
        let RetirementException::Reservist { term, .. } = facts else { unreachable!() };
        *term = ReserveTerm::Days { days: 179 };
        drop(data);
        unsupported(&d, "Income");
    }
}
#[test]
fn last_residential_use_exception_does_not_erase_depreciation_gain() {
    let mut d = complete(FederalStatus::Single);
    let mut home = home_sale(false);
    home.price = 60000000;
    home.expenses = 0;
    home.basis = 37300000;
    home.use_history =
        HomeSaleUse::Adjusted { depreciation: 2700000, owned_days: 1800, nonqualified_days: 0 };
    assert_eq!(home.excluded(FederalStatus::Single), Ok(20000000));
    add(&mut d, income(FederalIncome::HomeSale { data: main_home_source(home) }));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 27000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 27000);
    assert_eq!(value(&r, "agi"), 87000);
}
#[test]
fn paid_nonbusiness_guarantee_uses_cash_payment_and_subrogation_timing() {
    let mut d = complete(FederalStatus::Single);
    let mut debt = bad_debt(10000, 2000);
    let FederalIncome::NonbusinessBadDebt { data: crate::NonbusinessBadDebt { source, .. } } =
        &mut debt
    else {
        panic!()
    };
    *source = BadDebtSource::PaidGuarantee {
        paid: 1_000_000,
        reimbursed: 200_000,
        paid_year: 2024,
        motive: GuaranteeMotive::ProtectInvestment,
        recovery_right: GuaranteeRight::WhollyWorthless,
    };
    add(&mut d, income(debt));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 5000);
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::NonbusinessBadDebt { data: crate::NonbusinessBadDebt { source, .. } } =
        &mut d.income[1].income
    else {
        panic!()
    };
    *source = BadDebtSource::PaidGuarantee {
        paid: 1_000_000,
        reimbursed: 200_000,
        paid_year: 2026,
        motive: GuaranteeMotive::Profit,
        recovery_right: GuaranteeRight::None,
    };
    assert_eq!(value(&supported(&d), "capital_gain"), -3000);
}
#[test]
fn mixed_business_source_persists_and_the_young_filers_return_is_calculated() {
    let mut d = young_interest_return();
    let mut source = cash_business(10000);
    let FederalIncome::Business { data } = &mut source else { panic!() };
    data.income_source = BusinessIncomeSource::Mixed { compensation: 600000 };
    add(&mut d, income(source));
    // Form 8615 earned income is the smaller of the $6,000 reasonable
    // compensation and 30% of the profit after half of the self-employment
    // tax: 30% x ($10,000 - $707) = $2,788 (the coordination with the dependent
    // standard deduction that the old refusal guarded is not required).
    let r = supported(&d);
    let id = d.income[1].id;
    assert_eq!(value(&r, &alloc::format!("business_{id}_kiddie_earned_income")), 2788);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn self_employed_insurance_exception_does_not_invent_unemployment_income() {
    let mut d = complete(FederalStatus::Single);
    self_employed_insurance(&mut d, true, 12, 20260302);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 70000);
    assert_eq!(return_line(&r, "4b"), 1000000);
    assert_eq!(return_line(&r, "8"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 600);
    assert_eq!(
        value(&r, &alloc::format!("insurance_exception_{}", d.income.last().unwrap().id)),
        4000
    );
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn self_employed_insurance_exception_preserves_period_and_reemployment_limits() {
    for (confirmed, weeks, date) in [(true, 11, 20260302), (true, 12, 20260303)] {
        let mut d = complete(FederalStatus::Single);
        self_employed_insurance(&mut d, confirmed, weeks, date);
        unsupported(&d, "Income");
    }
    // No: the exception is not applied, so the 10% tax applies.
    {
        let confirmed = false;
        let mut d = complete(FederalStatus::Single);
        self_employed_insurance(&mut d, confirmed, 12, 20260302);
        let r = supported(&d);
        assert_eq!(value(&r, "early_distribution_tax"), 1000);
    }
}

#[test]
fn older_acquisition_first_used_in_2026_refigures_amt_after_section179() {
    // Pub. 946 Tables A-5 and A-18: Q4 5-year rates are 5% regular
    // and 3.75% AMT. Section 179 first reduces both $10,000 bases by $3,000.
    let mut asset =
        macrs_equipment(10000, EquipmentClass::Five, EquipmentMethod::Regular, 20261015);
    asset.acquired = 20170927;
    asset.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 300000 },
        bonus: BonusElection::Apply,
        tables: true,
    };
    let d = business_with_bonus(100000, asset.clone());
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_macrs_mid_quarter"), 1);
    assert_eq!(value(&r, "business_2_asset_0_section179"), 3000);
    for (id, expected) in [
        ("bonus", 0),
        ("macrs", 35000),
        ("amt_depreciation", 26250),
        ("remaining_basis", 665000),
        ("amt_remaining_basis", 673750),
    ] {
        assert_eq!(line_amount(&r, &alloc::format!("business_2_asset_0_{id}")), expected, "{id}");
    }
    assert_eq!(
        crate::depreciation::business_amt_adjustment(
            match &d.income[1].income {
                FederalIncome::Business { data } => data,
                _ => unreachable!(),
            },
            true,
        ),
        8750
    );
    assert_eq!(restart(d.clone()).draft, d);

    // The same regular and AMT amounts follow the exact manual formula.
    let mut manual = asset;
    if let EquipmentHistory::New { tables, .. } = &mut manual.history {
        *tables = false;
    }
    assert_eq!(manual.deduction(true), 35000);
    assert_eq!(manual.deduction_for(true, true), 26250);
}

#[test]
fn other_business_property_dispositions_are_refused_by_name() {
    // Unit A6 through the situations list; T2-35 narrowed the situation to
    // the dispositions an asset's own fold does not calculate.
    let baseline = business_with_bonus(100000, bonus_equipment(100000, 100));
    let baseline_result = supported(&baseline);
    assert_situation_refused(
        &baseline,
        RareSituation::AssetDisposed,
        &["Form 4797", "section 197 intangible"],
    );
    // An empty list keeps today's result.
    assert_eq!(estimate(&baseline), baseline_result);
}
#[test]
fn insolvency_rejects_unknown_snapshots_multiple_events_and_reduces_capital_loss() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, insolvency_debt(1500, 7500, 4600, 6100));
    let mut bad = d.clone();
    add(&mut bad, insolvency_debt(1000, 3000, 1000, 4000));
    unsupported(&bad, "Insolvency tax attributes");
    add(
        &mut d,
        income(FederalIncome::Sale {
            data: sale_source(crate::tests::support::builders::native_sale(
                GainTerm::Long,
                0,
                Some(1000000),
            )),
        }),
    );
    // $10,000 long-term loss: $3,000 deducted, $7,000 carried to 2027.
    // Insolvent by $7,500 - $4,600 = $2,900, so all $1,500 is excluded
    // (Form 982 line 2). No NOL or credits: line 9 takes the $1,500 from the
    // long-term carryover, leaving $5,500; nothing reaches basis (line 10a).
    let r = supported(&d);
    assert_eq!(value(&r, "capital_loss_carryforward"), 7000);
    assert_eq!(value(&r, "debt_2_form982_line2"), 1500);
    assert_eq!(value(&r, "debt_2_form982_line9"), 1500);
    assert_eq!(value(&r, "debt_2_capital_reduction"), 1500);
    assert_eq!(value(&r, "debt_2_capital_after"), 5500);
    assert_eq!(value(&r, "debt_2_amt_capital_after"), 5500);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 0);
    assert!(!r.lines.iter().any(|l| l.id == "debt_2_form982_line10a"));
}

#[test]
fn alternative_nol_recomputes_nonbusiness_limit_with_private_activity_income() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(-80000)));
    let ded = d.deductions.as_mut().unwrap();
    fixture_mortgage_interest(ded, 2000000);
    let before = supported(&d);
    assert_eq!(value(&before, "current_net_operating_loss"), 20000);
    assert_eq!(value(&before, "amt_current_net_operating_loss"), 20000);
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(native_exempt_interest(3000000, 3000000)),
        }),
    );
    let r = supported(&d);
    // AMT nonbusiness income of $30,000 absorbs the $20,000 mortgage
    // deduction. Only its remaining $10,000 reduces the business NOL.
    assert_eq!(value(&r, "current_net_operating_loss"), 20000);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 10000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn alternative_nol_carryforward_includes_its_own_excess_business_loss_once() {
    let d = large_separate_amt_business(300000, 1000);
    let r = supported(&d);
    assert_eq!(value(&r, "current_net_operating_loss"), 196000);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 196000);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 99360);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 317947);
    assert_eq!(value(&r, "nol_carryforward"), 295360);
    assert_eq!(value(&r, "amt_nol_carryforward"), 513947);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn ordinary_investment_stock_losses_do_not_expand_form461_business_losses() {
    {
        let stock = section1244_loss(50000, 0, 20200101);
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(cash_business(-300000)));
        let before = supported(&d);
        add(&mut d, income(stock));
        let r = supported(&d);
        assert_eq!(value(&r, "excess_business_loss"), 44000);
        assert_eq!(value(&r, "excess_business_loss"), value(&before, "excess_business_loss"));
        assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 44000);
        assert_eq!(value(&r, "current_net_operating_loss"), 246000);
        assert_eq!(value(&r, "amt_current_net_operating_loss"), 246000);
        assert_eq!(value(&r, "nol_carryforward"), 290000);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn same_rate_2026_reissue_combines_paid_periods_before_original_schedule_ceiling() {
    let mut d = complete(FederalStatus::Single);
    let deductions = d.deductions.as_mut().unwrap();
    fixture_mortgage_interest(deductions, 2_000_000);
    let mut claim = mortgage_certificate_credit(2000, 10000, 900_000);
    let certificate = claim.current.as_mut().unwrap();
    certificate.issue_date = 20260701;
    certificate.history = MortgageCertificateHistory::ReissuedDuring2026SameRate {
        facts: MortgageReissueFacts {
            refinanced_on: 20260630,
            original_interest_method: MortgageOriginalInterestMethod::FixedRateSchedule,
            first_reissue: true,
            original_number: "MCC-ORIGINAL-2026".into(),
            original_rate: 20.0,
            original_scheduled_interest: 1_000_000,
            original_certified_balance: 20_000_000,
            reissued_certified_debt: 18_000_000,
            confirmed: true,
        },
        original_period_interest: MortgageInterestSource::Certified { amount: 300_000 },
    };
    d.credits.as_mut().unwrap().mortgage_credit = Some(claim);
    let r = supported(&d);
    // $3,000 paid before refinance plus $9,000 after it would yield $2,400;
    // the original schedule would have allowed only $2,000 for the year.
    assert_eq!(value(&r, "mortgage_original_schedule_cap"), 2000);
    assert_eq!(value(&r, "mortgage_credit_current_potential"), 2000);
    assert_eq!(value(&r, "mortgage_interest_credit"), 2000);
    assert_eq!(value(&r, "deduction"), 18000);
    assert_eq!(supported(&restart(d.clone()).draft), r);

    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedDuring2026SameRate { facts, .. } =
        &mut certificate.history
    {
        facts.original_scheduled_interest = 2_000_000;
    }
    let combined = supported(&d);
    assert_eq!(value(&combined, "mortgage_original_schedule_cap"), 4000);
    assert_eq!(value(&combined, "mortgage_credit_current_potential"), 2400);
    assert_eq!(value(&combined, "deduction"), 17600);

    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    if let MortgageCertificateHistory::ReissuedDuring2026SameRate { facts, .. } =
        &mut certificate.history
    {
        facts.original_rate = 25.0;
    }
    assert!(!d.valid_stored());
    let certificate =
        d.credits.as_mut().unwrap().mortgage_credit.as_mut().unwrap().current.as_mut().unwrap();
    certificate.history = MortgageCertificateHistory::Unreviewed;
    unsupported(&d, "Mortgage interest credit");
}

#[test]
fn sole_pre_1991_federal_mortgage_subsidy_has_no_form_8828_requirement() {
    let mut d = complete(FederalStatus::Single);
    let ordinary_return = supported(&d);
    set_mortgage_subsidy(
        &mut d,
        MortgageSubsidyDisposition::ExemptBefore1991 {
            original_closed_on: 19901231,
            disposed_on: 20260701,
            no_later_subsidy: true,
        },
    );
    let old_loan = supported(&d);
    assert_eq!(value(&old_loan, "mortgage_subsidy_recapture_tax"), 0);
    assert_eq!(old_loan.tax, ordinary_return.tax);
    assert_eq!(supported(&restart(d.clone()).draft), old_loan);

    if let MortgageSubsidyDisposition::ExemptBefore1991 { no_later_subsidy, .. } =
        mortgage_disposition(&mut d)
    {
        *no_later_subsidy = false;
    }
    unsupported(&d, "Mortgage subsidy recapture");
    if let MortgageSubsidyDisposition::ExemptBefore1991 {
        no_later_subsidy,
        original_closed_on,
        ..
    } = mortgage_disposition(&mut d)
    {
        *no_later_subsidy = true;
        *original_closed_on = 19910101;
    }
    assert!(!d.valid_stored());
}

#[test]
fn personal_investment_loan_gain_is_nonbusiness_income_for_nol_and_form461() {
    for source in [
        scorp_repaid_reduced_open_account(ScorpLoanKind::OpenAccount),
        repaid_written_note(GainTerm::Short),
        repaid_written_note(GainTerm::Long),
    ] {
        let mut loss = sole_qbi_loss(400_000, 400_000, 200_000);
        loss.amt = Some(amt_records(&loss));
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::Scorp { data: scorp_source(loss) }));
        add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
        let r = supported(&d);
        assert_eq!(value(&r, "excess_business_loss_carryforward"), 144_000);
        assert_eq!(value(&r, "nol_carryforward"), 340_000);
        assert_eq!(value(&r, "qbi_loss_carryforward"), 128_000);
    }
}

#[test]
fn nol_nonbusiness_capital_losses_cannot_absorb_business_property_gains() {
    use crate::business_rules::current_nol;
    // $20,000 ordinary business loss; $10,000 investment loss and $5,000
    // business property capital gain. Only the investment loss is disallowed.
    assert_eq!(current_nol(-23000, nol_capital(-3000, -10000, 5000, 0, 0), 0, 0), 15000);
    assert_eq!(current_nol(-23000, nol_capital(-3000, -10000, 0, 0, 0), 0, 0), 20000);
}

#[test]
fn nol_personal_deductions_cannot_use_business_property_capital_gains() {
    use crate::business_rules::current_nol;
    // $20,000 ordinary business loss, $5,000 capital gain, $8,000 personal
    // deductions. The capital gain's source determines the available NOL.
    assert_eq!(current_nol(-23000, nol_capital(5000, 0, 5000, 0, 0), 8000, 0), 15000);
    assert_eq!(current_nol(-23000, nol_capital(5000, 5000, 0, 0, 0), 8000, 0), 20000);
}

#[test]
fn nol_business_capital_loss_uses_gain_after_personal_deductions() {
    // $20,000 ordinary business loss, $10,000 investment capital gain,
    // $20,000 business capital loss, and $5,000 personal deductions.
    assert_eq!(
        crate::business_rules::current_nol(
            -28000,
            nol_capital(-3000, 10000, -20000, 0, 0),
            5000,
            0
        ),
        20000
    );
}

#[test]
fn property_niit_requires_source_recapture_allocation_for_mixed_businesses() {
    let (mut draft, _) = passive_property_pool(former_passive_property_scorp(0, 10000, 0, 0), 0);
    add(
        &mut draft,
        income(FederalIncome::Scorp { data: scorp_source(passive_property_scorp(0, 20000)) }),
    );
    draft.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2025, 8000, 8000)], &[]));
    assert!(
        property_niit_worksheet(&draft, 22000, 22000)
            .unwrap_err()
            .contains("business-level gain allocation")
    );
}

#[test]
fn sole_property_business_lookback_qualifies_only_recaptured_gain() {
    for (qualified, expected) in [(20000, 8000), (0, 0)] {
        let mut source = former_passive_property_scorp(10000, 20000, 0, 0);
        source.property_qbi = property_qbi(0, qualified);
        source.qbi_statement = complete_ordinary_scorp(10000, true).qbi_statement;
        let (mut draft, _) = passive_property_pool(source.clone(), 0);
        draft.screening.as_mut().unwrap().section1231 =
            Some(section1231_history(&[(2025, 8000, 8000)], &[]));
        let items = property_qbi_character_items(&draft).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!((items[0].gain, items[0].loss), (expected, 0));
        let combined = source
            .qbi_statement
            .as_ref()
            .unwrap()
            .combined_activities(
                &source,
                [crate::kernel::carry::Amount::default(); 2],
                crate::section1231::PropertyCharacter::Recaptured(8000),
                Some(100000),
                FederalStatus::Single,
                0,
                0,
            )
            .unwrap();
        assert_eq!(combined.len(), 1);
        assert_eq!(combined[0].income, 10000 + expected);
    }
}

#[test]
fn property_lookback_service_gain_phases_once_with_combined_business_income() {
    let mut source = former_passive_property_scorp(10000, 20000, 0, 0);
    source.property_qbi = property_qbi(0, 20000);
    source.qbi_statement = complete_ordinary_scorp(10000, true).qbi_statement;
    let statement = source.qbi_statement.as_mut().unwrap();
    statement.service = true;
    statement.wages = 2000000;
    for (taxable, expected, wages) in
        [(100000, 18000, 20000), (239250, 9000, 10000), (300000, 0, 0)]
    {
        let values = source
            .qbi_statement
            .as_ref()
            .unwrap()
            .combined_activities(
                &source,
                [crate::kernel::carry::Amount::default(); 2],
                crate::section1231::PropertyCharacter::Recaptured(8000),
                Some(taxable),
                FederalStatus::Single,
                0,
                0,
            )
            .unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!((values[0].income, values[0].wages), (expected, wages));
    }
}

#[test]
fn niit_history_does_not_invent_business_capital_character_for_nol() {
    let mut draft = complete_property_gain_draft(true, 0);
    draft.screening.as_mut().unwrap().capital_history = vec![niit_loss_history(Some(7000))];
    let mut loss = sole_qbi_loss(100000, 100000, 100000);
    loss.amt = Some(amt_records(&loss));
    add(&mut draft, income(FederalIncome::Scorp { data: scorp_source(loss) }));
    unsupported(&draft, "NOL capital-loss source");
    unsupported(&draft, "AMT NOL capital-loss source");
}

#[test]
fn multiple_full_recapture_preserves_each_business_qualified_gain() {
    for (qualified, deduction, taxable, tax) in
        [(30000, 10000, 83900, 13176), (15000, 7000, 86900, 13836)]
    {
        let draft = multiple_fully_recaptured_gains(qualified);
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), 110000);
        assert_eq!(value(&r, "qbi_deduction"), deduction);
        assert_eq!(value(&r, "taxable_income"), taxable);
        assert_eq!(value(&r, "regular_income_tax"), tax);
        assert_eq!(return_line(&r, "7a"), 0);
        assert_eq!(return_line(&r, "8"), 5000000);
        assert_eq!(value(&r, "amt_capital_gain"), 0);
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn full_net_recapture_with_a_loss_business_remains_guarded() {
    let mut draft = multiple_fully_recaptured_gains(30000);
    let mut source = complete_ordinary_scorp(0, true);
    source.section1231 = Some(k1_section1231(-10000, 0));
    source.property_qbi = property_qbi(10000, 0);
    source.amt = Some(amt_records(&source));
    add(&mut draft, income(FederalIncome::Scorp { data: scorp_source(source) }));
    unsupported(&draft, "Section 1231 QBI character");
}

#[test]
fn qualified_rate_source_facts_feed_multi_business_return() {
    let mut draft = rate_ordered_recapture(8000, 5000);
    if let FederalIncome::Scorp { data } = &mut draft.income[1].income {
        let mut data = edit_scorp(data);
        data.section1231.as_mut().unwrap().unrecaptured = 1200000;
        let facts = &mut data.property_qbi;
        facts.gain = 800000;
        facts.qualified_1250 = Some(0);
    }
    let r = supported(&draft);
    assert_eq!(value(&r, "agi"), 110000);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(value(&r, "regular_income_tax"), 12716);
    assert_eq!(return_line(&r, "8"), 800000);
    assert_eq!(return_line(&r, "7a"), 4200000);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn box12_tt_overtime_deduction_applies_2026_magi_phaseout() {
    // W-2 wages $160,000.00 (AGI, no other income or adjustments) with box 12 TT qualified
    // overtime $6,000.00 claimed (`work_deductions(0, overtime)`, so the tips claim is $0
    // and cannot itself add a deduction). additional_deductions() in federal_rules.rs:
    // single-filer overtime shares the tips phaseout, $100 per $1,000 of AGI over
    // $150,000.00. By hand: work_reduction = floor(($160,000.00 - $150,000.00) / $1,000) x
    // $100 = 10 x $100 = $1,000.00; overtime_deduction = min($6,000.00, $12,500.00 single
    // cap) - $1,000.00 = $5,000.00.
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 16_000_000, 0);
    if let FederalIncome::Wage { data } = &mut draft.income[0].income {
        let mut data = edit_wage(data);
        data.work_deductions = work_deductions(0, 600_000);
    }
    let r = supported(&draft);
    assert_eq!(value(&r, "agi"), 160000);
    assert_eq!(value(&r, "overtime_deduction"), 5000);
    assert_eq!(value(&r, "tips_deduction"), 0);
}

#[test]
fn box12_tp_cash_tips_deduction_applies_2026_magi_phaseout() {
    // W-2 wages $160,000.00 (AGI, no other income or adjustments) with box 12 TP qualified
    // cash tips $8,000.00 claimed (`work_deductions(tips, 0)`, so the overtime claim is $0
    // and cannot itself add a deduction). Same 2026 phaseout formula as overtime, applied to
    // the $25,000.00 tips cap: work_reduction = floor(($160,000.00 - $150,000.00) / $1,000) x
    // $100 = $1,000.00; tips_deduction = min($8,000.00, $25,000.00) - $1,000.00 = $7,000.00.
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 16_000_000, 0);
    if let FederalIncome::Wage { data } = &mut draft.income[0].income {
        let mut data = edit_wage(data);
        data.work_deductions = work_deductions(800_000, 0);
    }
    let r = supported(&draft);
    assert_eq!(value(&r, "agi"), 160000);
    assert_eq!(value(&r, "tips_deduction"), 7000);
    assert_eq!(value(&r, "overtime_deduction"), 0);
}

#[test]
fn employee_8919_return_reports_wages_and_employee_tax_once() {
    for reason in [
        EmployeeWageReason::SameFirmForms,
        EmployeeWageReason::Ss8Determination { date: 20260901 },
        EmployeeWageReason::IrsCorrespondence { date: 20260902 },
        EmployeeWageReason::Ss8Pending { filed_on: 20260903 },
    ] {
        let mut draft = complete(FederalStatus::Single);
        let mut entry = omitted_employee_wages(10000);
        if let FederalIncome::EmployeeWages { data } = &mut entry {
            data.payroll = crate::EmployeePayrollSource::Form8919 { reason };
            data.withholding = 50000;
        }
        add(&mut draft, income(entry));
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), 70000);
        assert_eq!(value(&r, "taxable_income"), 53900);
        assert_eq!(value(&r, "employee_wages_0_social_tax"), 620);
        assert_eq!(value(&r, "employee_wages_0_medicare_tax"), 145);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(value(&r, "qbi_deduction"), 0);
        assert_eq!(return_line(&r, "25d"), 700000);
        assert_eq!(return_line(&r, "1a"), 6000000);
        assert_eq!(return_line(&r, "1g"), 1000000);
        assert_eq!(return_line(&r, "1z"), 7000000);
        assert_eq!(return_line(&r, "23"), 76500);
        // Single: 1,240 + 4,560 + 770 regular tax, plus 765 employee tax.
        assert_eq!(r.tax, Some(734100));
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn employee_8919_social_base_follows_tips_and_precedes_schedule_se() {
    let mut draft = wage_with_employee_tips(180000, allocated_employee_tips(3000));
    add(&mut draft, income(omitted_employee_wages(10000)));
    add(&mut draft, income(cash_business(10000)));
    let r = supported(&draft);
    crate::tests::support::assert::payroll::form8919_base(&r, 0, 183_000, 1_500);
    assert_eq!(value(&r, "employee_wages_0_social_base"), 1_500);
    assert_eq!(value(&r, "employee_tips_0_social_tax"), 186);
    assert_eq!(value(&r, "employee_tips_0_medicare_tax"), 44);
    assert_eq!(value(&r, "employee_wages_0_social_tax"), 93);
    assert_eq!(value(&r, "employee_wages_0_medicare_tax"), 145);
    assert_eq!(value(&r, "self_employment_tax"), 268);
    assert_eq!(return_line(&r, "1g"), 1000000);
    assert_eq!(value(&r, "agi"), 202866);
}

#[test]
fn employee_8919_requires_valid_reason_and_retains_income_outside_ordinary_coverage() {
    let mut entry = omitted_employee_wages(10000);
    if let FederalIncome::EmployeeWages { data } = &mut entry {
        data.form1099 = false;
        assert!(!data.valid());
        assert!(data.income().is_err());
        data.form1099 = true;
        data.payroll = crate::EmployeePayrollSource::Unavailable;
    }
    // Unavailable actual coverage retains income and preview; material payroll tax keeps the total unavailable.
    assert_eq!(crate::interview::document_amount(&entry, false), Some(1_000_000));
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(entry));
    let r = estimate(&draft);
    assert!(noted(&r, "WARN_EMPLOYEE_WAGES_PAYROLL_TAX_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "agi"), 70000);
    assert_eq!(r.tax, None);
    assert!(
        r.issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_EMPLOYEE_PAYROLL_COVERAGE_UNAVAILABLE")
    );
    // An invalid stored record (reason H without its Form 1099) is not accepted.
    if let FederalIncome::EmployeeWages { data } = &mut draft.income[1].income {
        data.payroll =
            crate::EmployeePayrollSource::Form8919 { reason: EmployeeWageReason::SameFirmForms };
        data.form1099 = false;
    }
    assert!(!draft.valid_stored());
}
#[test]
fn optional_se_eligibility_history_and_owner_are_not_inferred() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(12000, 3814)));
    elect_se(&mut d, SeMethod::Farm);
    assert!(noted(&supported(&d), "WARN_OPTIONAL_METHOD_NOT_AVAILABLE"));
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(-1000)));
    for h in [
        NonfarmSeHistory { earnings_second_prior_year: 39999, ..se_history() },
        NonfarmSeHistory { prior_years: vec![2020, 2020], ..se_history() },
    ] {
        elect_se(&mut d, SeMethod::Nonfarm { history: h });
        assert!(noted(&supported(&d), "WARN_OPTIONAL_METHOD_NOT_AVAILABLE"));
    }
    elect_se(&mut d, SeMethod::Nonfarm { history: se_history() });
    d.screening.as_mut().unwrap().optional_se[0].owner = Owner::Spouse;
    assert!(noted(&supported(&d), "WARN_OPTIONAL_METHOD_NOT_AVAILABLE"));
}
#[test]
fn optional_se_regular_losses_offset_optional_earnings_without_percentage_reduction() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(6000, 5000)));
    add(&mut d, income(cash_business(-1000)));
    elect_se(&mut d, SeMethod::Farm);
    let r = supported(&d);
    assert_eq!(value(&r, "optional_se_taxpayer_net"), 3000);
    assert_eq!(value(&r, "self_employment_tax"), 459);
    assert_eq!(return_line(&r, "8"), 0);
    assert_eq!(value(&r, "agi"), 59770);
}

#[test]
fn schedule_se_refuses_chapter11_estate_income() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(10_000)));
    // Hand computation (i1040sse): net earnings = $10,000 * 92.35% = $9,235.
    // The $60,000 of W-2 wages in `complete()` leaves $124,500 of Social
    // Security wage-base headroom, above the $9,235 net earnings, so the
    // full 12.4% + 2.9% = 15.3% combined rate applies:
    // $9,235 * 15.3% = $1,412.955, rounds to $1,413.
    let baseline = value(&supported(&d), "self_employment_tax");
    assert_eq!(baseline, 1413);

    // Unit41 computes clergy exemptions. Chapter11 remains a named refusal:
    // the business profit belongs on the bankruptcy estate's Form1041.
    assert_situation_refused(
        &d,
        RareSituation::Chapter11Debtor,
        &["Chapter 11 debtor", "self-employment income"],
    );

    // An empty list leaves today's Schedule SE total unchanged.
    assert_eq!(value(&supported(&d), "self_employment_tax"), baseline);
}

#[test]
fn schedule_c_depletion_pension_and_7205_are_refused() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(10_000)));
    // Hand computation (i1040sse): net earnings = $10,000 * 92.35% = $9,235.
    // The $60,000 of W-2 wages in `complete()` leaves $124,500 of Social
    // Security wage-base headroom, above the $9,235 net earnings, so the
    // full 12.4% + 2.9% = 15.3% combined rate applies:
    // $9,235 * 15.3% = $1,412.955, rounds to $1,413.
    let baseline = value(&supported(&d), "self_employment_tax");
    assert_eq!(baseline, 1413);

    // Unit A11, since M3-P5 through the business's special-treatment answer
    // (the situation was removed; the Schedule C page's "special" checkbox).
    let mut special = d.clone();
    let FederalIncome::Business { data } = &mut special.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.special_treatment = true;
    let r = supported(&special);
    assert!(noted(&r, "WARN_BUSINESS_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "self_employment_tax"), 0);

    // An empty list leaves today's result unchanged.
    assert_eq!(value(&supported(&d), "self_employment_tax"), baseline);
}

#[test]
fn self_employed_housing_uses_current_deduction_before_one_year_carry() {
    for (receipts, current, used, carry, expired) in [
        (140000, 7100, 0, 11506, 10000),
        (160000, 18606, 8494, 0, 1506),
        (200000, 18606, 10000, 0, 0),
    ] {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(excluded_foreign_business(receipts, 0)));
        set_claim(&mut d, foreign_claim(Owner::Taxpayer));
        self_employed_housing(&mut d, Some(HousingCarryover { amount: 1800000, deducted: 800000 }));
        let r = supported(&d);
        for (suffix, expected) in [
            ("current_deduction", current),
            ("prior_deduction_used", used),
            ("deduction_carryforward", carry),
            ("prior_deduction_expired", expired),
        ] {
            assert_eq!(value(&r, &alloc::format!("foreign_housing_taxpayer_{suffix}")), expected);
        }
        assert_eq!(value(&r, "foreign_housing_deduction"), current + used);
        assert_eq!(schedule_1_line(&r, "24j"), current + used);
        assert_eq!(schedule_1_line(&r, "25"), current + used);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        let ForeignHousing::Claim { prior: Some(prior), .. } =
            &mut claim_mut(&mut d, Owner::Taxpayer).housing
        else {
            unreachable!()
        };
        prior.deducted = prior.amount + 1;
        assert_eq!(estimate(&d).state, ResultState::Incomplete);
    }
}

#[test]
fn employee_stock_losses_wash_adjustments_and_reporting_boundaries() {
    let lot = EmployeeStockSale {
        acquired: 20260101,
        sold: 20260701,
        paid: 1000000,
        market_value: 3000000,
        fees: 0,
        proceeds: 2500000,
        wash_sale: 0,
        withholding: 0,
        reported_basis: None,
        reporting: crate::BasisReporting::ReportedToIrs,
        special: false,
    };
    for (wash, gain, carry) in [(0, -3000, 2000), (200000, -3000, 0), (500000, 0, 0)] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 8000000, 0);
        let mut data = lot.clone();
        data.wash_sale = wash;
        add(&mut d, income(FederalIncome::EmployeeStockSale { data }));
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), gain);
        assert_eq!(value(&r, "capital_loss_carryforward"), carry);
        assert_eq!(value(&r, "agi"), 80000 + gain);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        d.income[0] = wage(1, Owner::Taxpayer, 1000000, 0);
        unsupported(&d, "ISO compensation");
    }
    let mut invalid = lot.clone();
    invalid.wash_sale = 500001;
    assert!(!invalid.valid());
    invalid = lot.clone();
    invalid.sold = 20251231;
    assert!(!invalid.valid());
    let mut d = complete(FederalStatus::Single);
    let mut unconfirmed = lot;
    unconfirmed.special = true;
    add(&mut d, income(FederalIncome::EmployeeStockSale { data: unconfirmed }));
    let r = supported(&d);
    assert!(noted(&r, "WARN_EMPLOYEE_STOCK_SALE_LEFT_OUT"));
    assert_eq!(r.tax, supported(&complete(FederalStatus::Single)).tax);
}

#[test]
fn younger_railroad_survivor_uses_death_exception_without_disability_wages() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19760101;
    let mut entry = railroad_pension(12000, 15000, None);
    let FederalIncome::Retirement { data } = &mut entry.income else { unreachable!() };
    let mut data = edit_retirement(data);
    data.ordinary_distribution = Some(RetirementTreatment::Exception {
        second: None,
        distributed: 20260601,
        facts: RetirementException::Death { died: 20251201 },
        early_simple: false,
    });
    drop(data);
    add(&mut d, entry);
    add(&mut d, income(social_security_net(1000000, 0)));
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 1500000);
    assert_eq!(return_line(&r, "1h"), 0);
    assert_eq!(return_line(&r, "6b"), 850000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(value(&r, "agi"), 83500);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    let mut data = edit_retirement(data);
    let Some(RetirementTreatment::Exception { facts, .. }) = &mut data.ordinary_distribution else {
        unreachable!()
    };
    *facts = RetirementException::Death { died: 20260701 };
    drop(data);
    unsupported(&d, "Income:");
}

// Matrix row 9 (Schedule C): a typical single sole proprietorship, hand-computed
// from the 2026 Schedule C/SE instructions (i1040sc, i1040sse), Form 8995 (i8995)
// and Form 1040 (i1040gi). No expected number below is copied from the App.
#[test]
fn sole_proprietor_return_matches_hand_computation() {
    // Schedule C:
    //   Line 1 gross receipts: $88,000.00. Line 7 gross income: $88,000.00 (no
    //     COGS, no other income).
    //   Line 8 advertising $1,200.00; line 11 contract labor $6,000.00; line 15
    //     insurance $2,400.00; line 18 office $900.00; line 22 supplies $1,500.00;
    //     line 23 taxes and licenses $600.00; line 25 utilities $1,800.00.
    //   Line 9 car and truck, standard mileage at the 2026 rates (72.5 cents
    //     through June, 76 cents from July): 8,000 mi x $0.725 = $5,800.00;
    //     6,000 mi x $0.76 = $4,560.00; total $10,360.00.
    //   Line 24b deductible meals: $1,000.00 x 50% = $500.00.
    //   Line 28 total expenses: 1,200 + 10,360 + 6,000 + 2,400 + 900 + 1,500 +
    //     600 + 500 + 1,800 = $25,260.00.
    //   Line 29 tentative profit: 88,000.00 - 25,260.00 = $62,740.00.
    //   Line 30 simplified home office: 200 sq ft x $5/sq ft, all 12 months,
    //     = $1,000.00 (far under the $62,740.00 income cap, so not limited).
    //   Line 31 net profit: 62,740.00 - 1,000.00 = $61,740.00.
    //
    // Schedule SE (ordinary nonfarm method, this is the only Schedule C/F business):
    //   Line 2/3: $61,740.00.
    //   Line 4a-4c: 61,740.00 x 92.35% = $57,016.89, rounds to $57,017.
    //   Line 6: $57,017 (no church employee income). Line 7: $184,500 (2026
    //     Social Security wage base). Line 8a-8d: $0 (no W-2). Line 9: $184,500.
    //   Line 10/11: the smaller of line 6 or line 9 ($57,017) is far under the
    //     $184,500 wage base, so lines 10 (12.4%: $7,070.108) and 11 (2.9%:
    //     $1,653.493) combine cleanly into 57,017 x 15.3% = $8,723.601, rounded
    //     once to the nearest dollar.
    //   Line 12 self-employment tax: $8,724.
    //   Line 13 deduction for one-half self-employment tax: 8,724 / 2 = $4,362.
    //
    // Form 8995 (2026 single QBI threshold $201,750, far above this return):
    //   Line 1/2 QBI: net profit less the attributable half-SE deduction,
    //     61,740 - 4,362 = $57,378 (no SSTB reduction and no wage/UBIA limit
    //     applies below the threshold).
    //   Line 3/4: $57,378 (no carryforward). Line 5: 57,378 x 20% = $11,475.60,
    //     rounds to $11,476. Lines 6-9: $0 (no REIT/PTP dividends). Line 10:
    //     $11,476.
    //   Line 11 taxable income before QBI: AGI $57,378 less the $16,100
    //     standard deduction = $41,278. Line 12: $0 (no net capital gain).
    //     Line 13: $41,278. Line 14: 41,278 x 20% = $8,255.60, rounds to $8,256.
    //   Line 15 QBI deduction: smaller of line 10 ($11,476) or line 14 ($8,256)
    //     = $8,256. Lines 16/17: $0.
    //
    // Form 1040:
    //   Line 8 additional income (Schedule 1 line 10, and line 3 since this
    //     business is the only Part I item): $61,740.
    //   Line 9 total income: $61,740.
    //   Line 10 adjustments (Schedule 1 line 26, and line 15 since the half-SE
    //     deduction is the only adjustment): $4,362.
    //   Line 11 AGI: 61,740 - 4,362 = $57,378.
    //   Line 12e standard deduction (single, not 65/blind): $16,100.
    //   Line 13b QBI deduction: $8,256.
    //   Line 15 taxable income: 57,378 - 16,100 - 8,256 = $33,022.
    //   Line 16 regular tax (2026 draft Tax Table, single, $33,000-$33,050 row,
    //     midpoint $33,025): 12,400 x 10% + (33,025 - 12,400) x 12% =
    //     1,240.00 + 2,475.00 = $3,715.00.
    //   Line 23 (Schedule 2 line 4, self-employment tax): $8,724.
    //   Line 24a total tax: 3,715 + 8,724 = $12,439.
    //   Line 33 total payments: $0 (no withholding, no estimated payments).
    //   Line 37 balance due: $12,439.
    let vehicle = native_business_mileage(8_000, 6_000, 14_000);
    let expenses = vec![
        ordinary_business_expense(BusinessExpenseCategory::Advertising, 120_000),
        ordinary_business_expense(BusinessExpenseCategory::ContractLabor, 600_000),
        ordinary_business_expense(BusinessExpenseCategory::Insurance, 240_000),
        ordinary_business_expense(BusinessExpenseCategory::Office, 90_000),
        ordinary_business_expense(BusinessExpenseCategory::Supplies, 150_000),
        ordinary_business_expense(BusinessExpenseCategory::TaxesAndLicenses, 60_000),
        ordinary_business_expense(BusinessExpenseCategory::Utilities, 180_000),
        FederalBusinessExpense {
            category: BusinessExpenseCategory::Meals,
            amount: 100_000,
            treatment: BusinessExpenseKind::Meals,
        },
    ];
    let business = FederalIncome::Business {
        data: FederalBusiness {
            interest_limit_election: None,
            activity: BusinessActivity::Ordinary {
                returns: 0,
                receipts: 8_800_000,
                accounting: BusinessAccounting::Cash,
            },
            vehicles: vec![vehicle],
            expenses,
            ..survey_cash_business_data()
        },
    };
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19850615;
    d.income.clear();
    add(&mut d, income(business));
    let source = d.income.last().unwrap().id;
    add_simplified_office(&mut d, source, 200);
    let r = supported(&d);

    assert_eq!(value(&r, "business_2_expense_line8_advertising"), 1_200);
    assert_eq!(value(&r, "business_2_expense_line11_contract_labor"), 6_000);
    assert_eq!(value(&r, "business_2_expense_line15_insurance"), 2_400);
    assert_eq!(value(&r, "business_2_expense_line18_office"), 900);
    assert_eq!(value(&r, "business_2_expense_line22_supplies"), 1_500);
    assert_eq!(value(&r, "business_2_expense_line23_taxes_and_licenses"), 600);
    assert_eq!(value(&r, "business_2_expense_line25_utilities"), 1_800);
    assert_eq!(value(&r, "business_2_expense_line24b_meals"), 500);
    assert_eq!(value(&r, "business_2_vehicle_0_expense"), 10_360);
    assert_eq!(value(&r, "business_2_home_office"), 1_000);
    assert_eq!(value(&r, "business_2_cost_of_goods_sold"), 0);

    assert_eq!(return_line(&r, "8"), 6_174_000);
    assert_eq!(return_line(&r, "9"), 6_174_000);
    assert_eq!(return_line(&r, "10"), 436_200);
    assert_eq!(return_line(&r, "11a / 11b"), 5_737_800);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "13b"), 825_600);
    assert_eq!(return_line(&r, "15"), 3_302_200);
    assert_eq!(return_line(&r, "16"), 371_500);
    assert_eq!(return_line(&r, "23"), 872_400);
    assert_eq!(return_line(&r, "24a"), 1_243_900);
    assert_eq!(return_line(&r, "33"), 0);
    assert_eq!(return_line(&r, "37"), 1_243_900);

    assert_eq!(value(&r, "self_employment_tax"), 8_724);
    assert_eq!(value(&r, "adjustments"), 4_362);
    assert_eq!(value(&r, "agi"), 57_378);
    assert_eq!(value(&r, "qbi_deduction"), 8_256);
    assert_eq!(value(&r, "taxable_income"), 33_022);
    assert_eq!(r.tax, Some(1_243_900));
    assert_eq!(r.payments, Some(0));
    assert_eq!(r.balance, Some(1_243_900));
    assert_eq!(restart(d.clone()).draft, d);
}

// Boundary (1): net profit under $400 owes no self-employment tax and gets no
// half-SE deduction (Schedule SE line 4c/6 below $400, i1040sse).
#[test]
fn net_profit_under_400_owes_no_self_employment_tax() {
    // $300 net profit: 300 x 92.35% = $277.05, rounds to $277, under the $400
    // Schedule SE floor, so self-employment tax and the half-SE deduction are
    // both zero (se_from_net's `net < 400` rule, i1040sse Schedule SE line 4c
    // instructions: "you don't owe self-employment tax ... unless ... $400 or more").
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(300)));
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(return_line(&r, "8"), 30_000);
}

// Boundary (2): meals under the DOT hours-of-service treatment deduct 80%,
// not the ordinary 50% (BusinessExpenseKind::TransportationMeals, i1040sc line 24b).
#[test]
fn dot_meals_deduct_eighty_percent() {
    // $1,000 receipts, one $1,000 DOT hours-of-service meal expense: deductible
    // at 80% = $800.00 (line 24b), leaving net profit 1,000.00 - 800.00 = $200.00.
    let mut d = empty_return(FederalStatus::Single);
    let mut business = cash_business(1000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.expenses = vec![FederalBusinessExpense {
        category: BusinessExpenseCategory::Meals,
        amount: 100_000,
        treatment: BusinessExpenseKind::TransportationMeals,
    }];
    add(&mut d, income(business));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_expense_line24b_meals"), 800);
    assert_eq!(return_line(&r, "8"), 20_000);
}

// Boundary (4): a net loss with material participation flows to Schedule 1
// line 3 as a negative amount and produces no QBI deduction (qbi_with_history's
// loss netting, i8995).
#[test]
fn business_loss_reduces_income_without_qbi() {
    // $0 receipts, a $2,000 expense: a $2,000.00 net loss. It is not a
    // qualifying positive activity, so it contributes no QBI deduction, and it
    // owes no self-employment tax (self_employment floors a non-positive
    // profit at $0 net earnings).
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(-2000)));
    let r = supported(&d);
    assert_eq!(return_line(&r, "8"), -200_000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "qbi_deduction"), 0);
}

pub(in crate::tests) fn aggregation_activity(
    income: i64,
    wages: i64,
    group: Option<&str>,
) -> QbiActivity {
    QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income,
        wages,
        property: 0,
        active: true,
        service: Some(false),
        group: group.map(Into::into),
    }
}

fn aggregation_business(receipts: i64, payroll: i64, group: Option<&str>) -> FederalIncomeEntry {
    income(FederalIncome::Business {
        data: FederalBusiness {
            interest_limit_election: None,
            activity: BusinessActivity::Ordinary {
                returns: 0,
                receipts,
                accounting: BusinessAccounting::Cash,
            },
            expenses: if payroll == 0 {
                vec![]
            } else {
                vec![ordinary_business_expense(
                    BusinessExpenseCategory::Other { label: "Payroll".into() },
                    payroll,
                )]
            },
            payroll: if payroll == 0 {
                BusinessPayroll::No
            } else {
                BusinessPayroll::Yes {
                    statements: vec![PayrollW2 {
                        label: "Staff".into(),
                        wages: payroll,
                        medicare: payroll,
                    }],
                    attribution: crate::PayrollQbiAttribution::EntireBusiness,
                }
            },
            qbi_aggregation_group: group.map(Into::into),
            ..survey_cash_business_data()
        },
    })
}

pub(in crate::tests) fn aggregation_draft(
    wages: i64,
    businesses: [(i64, i64); 2],
    group: Option<&str>,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
    for (receipts, payroll) in businesses {
        add(&mut d, aggregation_business(receipts, payroll, group));
    }
    d
}

#[test]
fn aggregated_loss_member_nets_inside_its_group_and_keeps_its_wages() {
    // Single, taxable income $400,000 (above the phase-in ceiling).
    // A: QBI $200,000, wages $0. C: QBI -$50,000, wages $120,000.
    let mut r = supported(&complete(FederalStatus::Single));
    let ungrouped =
        [aggregation_activity(200_000, 0, None), aggregation_activity(-50_000, 120_000, None)];
    // Ungrouped: C's loss nets against A (Schedule C), A = 150,000 with a $0
    // limit; C has no QBI. Components $0, but the active-business minimum
    // (Form 8995 line 16: active QBI $150,000 >= $1,000) gives $400; no carryforward.
    assert_eq!(qbi(&ungrouped, 400_000, 0, FederalStatus::Single, &mut r), (400, 0));
    // Grouped: QBI 200,000 - 50,000 = 150,000 with C's wages $120,000:
    //   min(20% x 150,000 = 30,000, 50% x 120,000 = 60,000) = $30,000.
    let grouped = ungrouped.clone().map(|v| QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        group: Some("Shops".into()),
        ..v
    });
    assert_eq!(qbi(&grouped, 400_000, 0, FederalStatus::Single, &mut r), (30_000, 0));
}

#[test]
fn aggregation_refuses_a_specified_service_member_above_the_threshold() {
    let service = QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        service: Some(true),
        ..aggregation_activity(50_000, 120_000, Some("Shops"))
    };
    let mut r = supported(&complete(FederalStatus::Single));
    qbi(
        &[aggregation_activity(200_000, 0, Some("Shops")), service.clone()],
        400_000,
        0,
        FederalStatus::Single,
        &mut r,
    );
    assert!(r.issues.iter().any(|i| i.message.contains("Form 8995-A Schedule B")));
    assert!(!r.review_notes.iter().any(|n| n.contains("Aggregation")));
    // Below the threshold the label has no effect and nothing is refused.
    let mut r = supported(&complete(FederalStatus::Single));
    qbi(
        &[aggregation_activity(200_000, 0, Some("Shops")), service],
        100_000,
        0,
        FederalStatus::Single,
        &mut r,
    );
    assert!(r.issues.is_empty());
}

#[test]
fn aggregation_has_no_effect_below_the_threshold() {
    // Single, taxable income $150,000 < $201,750 threshold: every business takes
    // 20% of QBI with no wage/UBIA limit. A 20% x 60,000 = 12,000; B 20% x
    // 40,000 = 8,000; total $20,000 (< 20% x 150,000 = $30,000 cap) either way.
    let ungrouped =
        [aggregation_activity(60_000, 0, None), aggregation_activity(40_000, 10_000, None)];
    let grouped = ungrouped.clone().map(|v| QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        group: Some("Shops".into()),
        ..v
    });
    for items in [ungrouped, grouped] {
        let mut r = supported(&complete(FederalStatus::Single));
        assert_eq!(qbi(&items, 150_000, 0, FederalStatus::Single, &mut r), (20_000, 0));
        assert!(!r.review_notes.iter().any(|n| n.contains("Aggregation")));
    }
    // Through the Schedule C records: W-2 wages $30,000 and two small shops.
    let plain =
        supported(&aggregation_draft(3_000_000, [(2_000_000, 0), (5_000_000, 4_000_000)], None));
    let labeled = supported(&aggregation_draft(
        3_000_000,
        [(2_000_000, 0), (5_000_000, 4_000_000)],
        Some("Shops"),
    ));
    assert_eq!(value(&labeled, "qbi_deduction"), value(&plain, "qbi_deduction"));
    assert!(!labeled.review_notes.iter().any(|n| n.contains("Aggregation")));
}

pub(in crate::tests) use crate::tests::support::builders::actual_vehicle;
pub(in crate::tests) fn cents(r: &FederalResult, id: &str) -> i64 {
    r.lines.iter().find(|l| l.id == id).unwrap_or_else(|| panic!("missing {id}")).amount
}
pub(in crate::tests) fn form_4562_line(r: &FederalResult, id: &str) -> alloc::string::String {
    r.lines
        .iter()
        .find(|l| l.id == id)
        .and_then(|l| l.form_line.as_ref())
        .map(|f| {
            assert_eq!(f.form, TaxForm::Form4562);
            f.line.clone()
        })
        .unwrap()
}

#[test]
fn vehicle_actual_expenses_apply_280f_limits_and_business_use() {
    // $60,000 passenger automobile placed in service March 15, 2026; 7,500 of
    // 10,000 miles are business miles, so business use is 75%. No section 179,
    // bonus elected out, the only 2026 asset (half-year convention).
    //   Business basis: 60,000 x 75% = $45,000.00.
    //   MACRS 5-year 200% DB, half-year: 45,000 x 2/5 x 1/2 = $9,000.00.
    //   Section 280F first-year limit without bonus (Rev. Proc. 2026-15 Table 2):
    //   $12,300 x 75% = $9,225.00.
    //   Smaller wins ($9,000 < $9,225): depreciation (Schedule C line 13, Form 4562 26(h)) $9,000.00.
    //   Operating costs $4,000 x 75% = $3,000.00 on Schedule C line 9.
    //   Schedule C net profit: 100,000 - 9,000 - 3,000 = $88,000 (Form 1040 line 8).
    for (kind, description) in [
        (VehicleKind::PassengerCar, "Passenger car"),
        (VehicleKind::PassengerTruckVan, "Passenger truck or van"),
    ] {
        let mut vehicle = actual_vehicle(60000, 10_000, 7_500);
        let ListedProperty::Vehicle { kind: actual_kind, .. } = &mut vehicle.listed else {
            unreachable!()
        };
        *actual_kind = kind;
        let d = business_with_bonus(100000, vehicle);
        let r = supported(&d);
        assert_eq!(cents(&r, "business_2_asset_0_macrs"), 900_000);
        assert_eq!(cents(&r, "business_2_asset_0_bonus"), 0);
        assert_eq!(cents(&r, "business_2_car_and_truck"), 300_000);
        crate::tests::support::assert::assert_displayed_text_line(
            &r,
            "business_2_asset_0_listed_type",
            description,
        );
        assert_eq!(cents(&r, "business_2_asset_0_listed_business_use"), 7_500);
        assert_eq!(cents(&r, "business_2_asset_0_listed_cost"), 6_000_000);
        assert_eq!(cents(&r, "business_2_asset_0_listed_depreciation_basis"), 4_500_000);
        assert_eq!(cents(&r, "business_2_asset_0_listed_depreciation"), 900_000);
        assert_eq!(cents(&r, "business_2_asset_0_listed_section179"), 0);
        assert_eq!(form_4562_line(&r, "business_2_asset_0_listed_depreciation"), "26(h)");
        assert_eq!(form_4562_line(&r, "business_2_asset_0_listed_business_use"), "26(c)");
        assert_eq!(return_line(&r, "8"), 8_800_000);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn f04_actual_vehicle_keeps_fraction_below_one_percent() {
    // $30,000 vehicle, $4,000 operating costs, 20,000 total miles. With
    // <=50% business use, ADS five-year straight-line takes one half-year:
    // 10% of the exact business basis. Whole-percent use remains unchanged.
    for (business_miles, basis, depreciation, operating, displayed_use) in [
        (100, 15_000, 1_500, 2_000, 50),
        (67, 10_050, 1_005, 1_340, 34),
        (200, 30_000, 3_000, 4_000, 100),
        (10_000, 1_500_000, 150_000, 200_000, 5_000),
    ] {
        let vehicle = actual_vehicle(30_000, 20_000, business_miles);
        assert!(vehicle.valid());
        assert_eq!(vehicle.business_basis(), basis);
        assert_eq!(vehicle.business_use_hundredths(), displayed_use);
        assert_eq!(vehicle.vehicle_operating(), operating);
        let draft = business_with_bonus(100_000, vehicle);
        assert!(draft.valid_stored());
        let result = supported(&draft);
        assert_eq!(cents(&result, "business_2_asset_0_macrs"), depreciation);
        assert_eq!(cents(&result, "business_2_car_and_truck"), operating);
        assert_eq!(cents(&result, "business_2_asset_0_listed_depreciation_basis"), basis);
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn f04_actual_vehicle_rejects_zero_or_excess_miles_without_calculation() {
    for (total, business) in [(0, 0), (0, 1), (20_000, 0), (20_000, 20_001)] {
        let mut vehicle = actual_vehicle(30_000, 20_000, 100);
        if let ListedProperty::Vehicle { total_miles, business_miles, .. } = &mut vehicle.listed {
            *total_miles = total;
            *business_miles = business;
        }
        assert!(!vehicle.valid());
        let draft = business_with_bonus(100_000, vehicle);
        assert!(!draft.valid_stored());
        assert_ne!(estimate(&draft).state, ResultState::EstimatedForSupportedScope);
    }
}

#[test]
fn schedule_c_actual_vehicle_loan_interest_is_line_16b_with_totals_unchanged() {
    // The accepted addition to the U5 split: an actual-expense vehicle's loan
    // interest goes to Schedule C line 16b by business miles. Vehicle: 7,500 of
    // 10,000 miles (75%), operating costs $4,000, loan interest $2,000.
    //   Line 9 keeps the operating share 4,000 x 75% = $3,000.00.
    //   Line 16b = 2,000 x 75% = $1,500.00; net profit falls by exactly $1,500.
    let plain = business_with_bonus(100000, actual_vehicle(60000, 10_000, 7_500));
    let mut d = plain.clone();
    let FederalIncome::Business { data } = &mut d.income.last_mut().unwrap().income else {
        panic!()
    };
    let ListedProperty::Vehicle { loan_interest, .. } = &mut data.assets[0].listed else {
        panic!()
    };
    *loan_interest = 200_000;
    let before = supported(&plain);
    let r = supported(&d);
    let rows: alloc::vec::Vec<_> = crate::draft_return(&r)
        .iter()
        .filter(|row| row.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleC))
        .map(|row| (row.form_line.as_ref().unwrap().line.clone(), row.amount))
        .collect();
    assert!(rows.contains(&("9".into(), 300_000)), "{rows:?}");
    assert!(rows.contains(&("16b".into(), 150_000)), "{rows:?}");
    assert_eq!(value(&before, "business_2_net_profit") - value(&r, "business_2_net_profit"), 1_500);
    // Reopening a saved business shows the amount on the page's actual vehicle.
    let mut d = plain;
    let FederalIncome::Business { data } = &mut d.income.last_mut().unwrap().income else {
        panic!()
    };
    let ListedProperty::Vehicle { loan_interest, .. } = &mut data.assets[0].listed else {
        panic!()
    };
    *loan_interest = 200_000;
    let (items, _) =
        crate::interview::vehicle_and_asset_items(&data.vehicles, &data.assets).unwrap();
    assert!(matches!(items[0].method, VehicleMethod::Actual { loan_interest: 200_000, .. }));
}

#[test]
fn vehicle_uses_mileage_or_actual_never_both() {
    let mut d = business_with_bonus(100000, actual_vehicle(60000, 10_000, 7_500));
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    // A vehicle is one method by its page: a standard-mileage vehicle and an
    // actual-expense vehicle are different vehicles, with no label to collide
    // (F3, 2026-10-09: the positional labels are derived where shown).
    data.vehicles.push(native_business_mileage(1_000, 0, 10_000));
    // Line 9 adds 1,000 x $0.725 = $725.00 to the $3,000.00 actual-expense
    // share.
    let r = supported(&d);
    assert_eq!(cents(&r, "business_2_car_and_truck"), 372_500);
    // Vehicle use has one source: the exact miles, with no separate percentage.
    let car = actual_vehicle(60000, 10_000, 7_500);
    assert_eq!(car.business_use_hundredths(), 7_500);
    assert!(car.valid());
}

// Unit C19b: nonresidential real property (Form 4562 Part III line 19j on
// the 2026 draft), 39-year straight-line, mid-month convention. Bonus and
// section 179 never apply; the New/Continuing bonus and tables fields are
// inert for this class (documented on `BusinessAsset::method`/`eligible`).
pub(in crate::tests) fn nonresidential_building(cost: i64, placed: u32) -> BusinessAsset {
    BusinessAsset {
        cost: cost * 100,
        acquired: placed,
        placed,
        class: EquipmentClass::ThirtyNine,
        history: EquipmentHistory::New {
            section179: Section179::Ineligible,
            bonus: BonusElection::ElectOut,
            tables: true,
        },
        method: EquipmentMethod::StraightLine,
        ..bonus_equipment(0, 100)
    }
}

#[test]
fn nonresidential_property_depreciates_over_39_years_mid_month() {
    // $390,000 business building placed in service May 15, 2026: Pub 946
    // Table A-7a's May first-year percentage is 1.605% (by hand):
    //   $390,000 x 1.605% = $6,259.50.
    let d = business_with_bonus(100000, nonresidential_building(390_000, 20260515));
    let r = supported(&d);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 625_950);
    assert_eq!(cents(&r, "business_2_asset_0_bonus"), 0);
    assert_eq!(form_4562_line(&r, "business_2_asset_0_macrs"), "19j");
    // Real property is outside the half-year/mid-quarter convention test.
    assert!(!r.lines.iter().any(|l| l.id == "business_2_macrs_mid_quarter"));
    // Line 8 rounds to whole dollars: 100,000 - 6,259.50 = 93,740.50 -> 93,741.
    assert_eq!(return_line(&r, "8"), 9_374_100);
}

#[test]
fn mid_quarter_convention_applies_above_40_percent_in_the_fourth_quarter() {
    // Boundary (form-4562.md; Pub 946 ch. 4): the mid-quarter convention
    // applies return-wide only when more than 40% of the year's total
    // depreciable basis (excluding real property, after elected section
    // 179) is placed in service in the fourth quarter. Two 5-year assets,
    // no section 179, bonus elected out.
    //
    // Case 1 (exactly 40%, not "more than"): $60,000 (January, Q1) +
    // $40,000 (November, Q4) = $100,000 total; the Q4 share is
    // 40,000 / 100,000 = 40.00% exactly, so half-year applies to both
    // (Table A-1, 5-year, Year 1 = 20.00%):
    //   $60,000 x 20.00% = $12,000.00; $40,000 x 20.00% = $8,000.00.
    let asset_a = depreciable_asset(60_000, EquipmentClass::Five, 20260115, 0);
    let mut d = business_with_bonus(0, asset_a);
    let asset_b = depreciable_asset(40_000, EquipmentClass::Five, 20261115, 0);
    {
        let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
        data.assets.push(asset_b);
    }
    assert!(!crate::depreciation::mid_quarter(&d));
    let r = supported(&d);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 1_200_000);
    assert_eq!(cents(&r, "business_2_asset_1_macrs"), 800_000);

    // Case 2 (above 40%): raise the Q4 asset's cost to $50,000; total
    // $110,000, Q4 share 50,000 / 110,000 = 45.45% > 40%, so mid-quarter
    // applies return-wide. The Q4 asset now uses Table A-5 (5-year, Year 1
    // = 5.00%): $50,000 x 5.00% = $2,500.00. The Q1 asset switches from
    // Table A-1 to Table A-2 (5-year, Year 1 = 35.00%):
    // $60,000 x 35.00% = $21,000.00.
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets[1].cost = 50_000 * 100;
    assert!(crate::depreciation::mid_quarter(&d));
    let r = supported(&d);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 2_100_000);
    assert_eq!(cents(&r, "business_2_asset_1_macrs"), 250_000);
}

// The M3-F1 return (shared with the review test in tests/interview.rs).
pub(in crate::tests) fn m3_self_employed_rental_draft() -> FederalDraft {
    let vehicle = native_business_mileage(8_000, 6_000, 14_000);
    let expenses = vec![
        ordinary_business_expense(BusinessExpenseCategory::Advertising, 120_000),
        ordinary_business_expense(BusinessExpenseCategory::ContractLabor, 600_000),
        ordinary_business_expense(BusinessExpenseCategory::Insurance, 240_000),
        ordinary_business_expense(BusinessExpenseCategory::Office, 90_000),
        ordinary_business_expense(BusinessExpenseCategory::Supplies, 150_000),
        ordinary_business_expense(BusinessExpenseCategory::TaxesAndLicenses, 60_000),
        ordinary_business_expense(BusinessExpenseCategory::Utilities, 180_000),
        FederalBusinessExpense {
            category: BusinessExpenseCategory::Meals,
            amount: 100_000,
            treatment: BusinessExpenseKind::Meals,
        },
    ];
    let computer = BusinessAsset {
        cost: 240_000,
        acquired: 20260210,
        placed: 20260210,
        history: EquipmentHistory::New {
            section179: Section179::Ineligible,
            bonus: BonusElection::Apply,
            tables: true,
        },
        ..bonus_equipment(0, 100)
    };
    let business = FederalIncome::Business {
        data: FederalBusiness {
            interest_limit_election: None,
            activity: BusinessActivity::Ordinary {
                returns: 0,
                receipts: 8_800_000,
                accounting: BusinessAccounting::Cash,
            },
            vehicles: vec![vehicle],
            assets: vec![computer],
            expenses,
            ..survey_cash_business_data()
        },
    };
    let rental_expense = |category: RentalExpenseCategory, amount: i64| RentalExpense {
        category,
        amount: amount * 100,
        treatment: BusinessExpenseKind::Ordinary,
    };
    let rental_data = FederalRental {
        interest_limit_election: None,
        location: crate::RentalLocation::UnitedStates,
        property_qbi_history: Vec::new(),
        rent: 24_000 * 100,
        expenses: vec![
            rental_expense(RentalExpenseCategory::Insurance, 1_400),
            rental_expense(RentalExpenseCategory::ManagementFees, 2_400),
            rental_expense(RentalExpenseCategory::MortgageInterest, 9_600),
            rental_expense(RentalExpenseCategory::Repairs, 1_800),
            rental_expense(RentalExpenseCategory::TaxesAndLicenses, 3_200),
            rental_expense(RentalExpenseCategory::Utilities, 600),
        ],
        buildings: vec![purchased_rental_building(260_000 * 100, 40_000 * 100, 20210301)],
        ..rental(0, 0)
    };
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800314;
    d.income.clear();
    add(&mut d, income(business));
    let source = d.income.last().unwrap().id;
    add(&mut d, income(FederalIncome::Rental { data: rental_data }));
    add_simplified_office(&mut d, source, 200);
    d
}

// The review rows of one form, as (label, amount).
fn form_rows(d: &FederalDraft, form: TaxForm) -> Vec<(alloc::string::String, i64)> {
    crate::draft_return(&supported(d))
        .into_iter()
        .filter(|row| row.form_line.as_ref().is_some_and(|f| f.form == form))
        .map(|row| (status_text(&row.label), row.amount))
        .collect()
}

#[test]
fn prior_year_building_needs_no_form_4562() {
    // The M3-F1 rental alone: one house placed in service March 2021, cost
    // $260,000 less $40,000 land = $220,000 depreciable, a full middle year
    // of 27.5-year GDS in 2026: $220,000 x 24/660 = $8,000. A building
    // placed in service before 2026, with no section 179, listed property or
    // amortization, needs no Form 4562 (i4562 "Who Must File"): the $8,000
    // is on Schedule E line 18 alone.
    let mut d = m3_self_employed_rental_draft();
    d.income.retain(|e| matches!(e.income, FederalIncome::Rental { .. }));
    d.home_sites.clear(); // The retained rental has no business home-use claim.
    assert!(form_rows(&d, TaxForm::Form4562).is_empty());
    let schedule_e = form_rows(&d, TaxForm::ScheduleE);
    let id = d.income[0].id;
    assert!(
        schedule_e.contains(&(
            alloc::format!("Schedule E 18 — Rental {id} line 18 depreciation"),
            800_000
        )),
        "{schedule_e:?}"
    );
}

#[test]
fn current_year_asset_lists_its_form_4562_lines() {
    // Three assets, all 100% business use, half-year convention (no 2026
    // basis in the fourth quarter):
    // - asset 0, 7-year equipment placed June 2026, bonus elected out, manual
    //   200% DB: $10,000 x 2/7 x 1/2 = $1,428.57, Part III line 19c.
    // - asset 1, 5-year equipment placed June 2025 (cost $1,000, 2025
    //   half-year $200 already deducted): 2026 200% DB on the $800 left,
    //   $800 x 2/5 = $320 (above straight line $800 x 8/36 = $177.78), line 17.
    // - asset 2, 5-year equipment acquired January 20, 2025 and placed June
    //   2026, 100% bonus: $2,000 on Part II line 14a, no MACRS left.
    // Line 22 = $1,428.57 + $320 + $2,000 = $3,748.57, which is Schedule C
    // line 13. Section 179 line 12 is zero (elected $0) and not listed.
    let mut seven = bonus_equipment(10_000, 100);
    seven.class = EquipmentClass::Seven;
    seven.history = EquipmentHistory::New {
        section179: Section179::Ineligible,
        bonus: BonusElection::ElectOut,
        tables: false,
    };
    let mut d = business_with_bonus(100_000, seven);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets.push(continuing_equipment(20250601, 20_000, false));
    data.assets.push(bonus_equipment(2_000, 100));
    let id = d.income[1].id;
    // 2026-10-19, D-2026-10-19-01: print dollars after the exact total.
    let result = supported(&d);
    assert_eq!(line_amount(&result, &alloc::format!("business_{id}_asset_0_macrs")), 142_857);
    assert_eq!(line_amount(&result, &alloc::format!("business_{id}_depreciation_total")), 374_857);
    assert_eq!(
        form_rows(&d, TaxForm::Form4562),
        vec![
            (alloc::format!("Form 4562 14a — Business {id} asset 2 bonus"), 200_000),
            (alloc::format!("Form 4562 17 — Business {id} asset 1 MACRS"), 32_000),
            (alloc::format!("Form 4562 19c — Business {id} asset 0 MACRS"), 142_900),
            (alloc::format!("Form 4562 22 — Business {id} depreciation total"), 374_900),
        ]
    );
    // 2026-10-19 (acceptance defect 4): Schedule C prints whole dollars like Form 4562.
    assert!(
        form_rows(&d, TaxForm::ScheduleC)
            .contains(&(alloc::format!("Schedule C 13 — Business {id} depreciation"), 374_900))
    );
    // Only the 2025 asset left: no Form 4562; Schedule C line 13 carries $320.
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.assets.retain(|a| a.placed < 20260101);
    assert!(form_rows(&d, TaxForm::Form4562).is_empty());
    assert!(
        form_rows(&d, TaxForm::ScheduleC)
            .contains(&(alloc::format!("Schedule C 13 — Business {id} depreciation"), 32_000))
    );
}

#[test]
fn schedule_2_line_4_is_the_se_tax() {
    // M3-F1: net profit $59,340 x 92.35% = $54,800 x 15.3% = $8,384, the
    // one Schedule SE's line 12 and Schedule 2 line 4.
    let d = m3_self_employed_rental_draft();
    assert_eq!(
        form_rows(&d, TaxForm::Schedule2),
        vec![
            ("Schedule 2 4 — Self-employment tax".into(), 838_400),
            ("Schedule 2 15 — Schedule 2 income and other taxes subtotal".into(), 838_400),
            ("Schedule 2 21 — Schedule 2 total additional taxes".into(), 838_400),
        ]
    );
    assert!(
        form_rows(&d, TaxForm::ScheduleSe)
            .contains(&("Schedule SE 12 — Taxpayer self-employment tax".into(), 838_400))
    );
}

#[test]
fn joint_return_tags_each_spouses_schedule_se() {
    // A joint return where each spouse runs the M3-F1 business (no wages):
    // each has net profit $88,000 - $27,660 - $1,000 = $59,340, so each
    // Schedule SE line 4a is 59,340 x 92.35% = $54,800 and line 12 is
    // 54,800 x 15.3% = $8,384 (each spouse has a full $184,500 wage base).
    // Each spouse's Schedule SE is separate: two line 12 rows, never one
    // merged $16,768 total, and the taxpayer's form is listed whole first.
    let single = m3_self_employed_rental_draft();
    let business = single
        .income
        .iter()
        .find(|e| matches!(e.income, FederalIncome::Business { .. }))
        .unwrap()
        .income
        .clone();
    let mut d = empty_return(FederalStatus::Joint);
    add(&mut d, income(business.clone()));
    let mut spouse = income(business);
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    // 2026-10-17: home-use claims have their own canonical owner, rather
    // than being copied implicitly with each business's expense record.
    let sources: Vec<_> = d.income.iter().map(|entry| entry.id).collect();
    for source in sources {
        add_simplified_office(&mut d, source, 200);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 16_768);
    let rows = crate::draft_return(&r);
    let se: Vec<(alloc::string::String, i64)> = rows
        .iter()
        .filter(|row| row.form_line.as_ref().unwrap().form == TaxForm::ScheduleSe)
        .map(|row| (status_text(&row.label), row.amount))
        .collect();
    assert_eq!(
        se.iter()
            .filter(|(label, _)| label.starts_with("Schedule SE 12 — "))
            .map(|(label, amount)| (label.as_str(), *amount))
            .collect::<Vec<_>>(),
        vec![
            ("Schedule SE 12 — Taxpayer self-employment tax", 838_400),
            ("Schedule SE 12 — Spouse self-employment tax", 838_400),
        ]
    );
    let first_spouse = se.iter().position(|(label, _)| label.contains("Spouse")).unwrap();
    assert!(se[..first_spouse].iter().all(|(l, _)| l.contains("Taxpayer")));
    assert!(se[first_spouse..].iter().all(|(l, _)| l.contains("Spouse")));
    assert_eq!(se[first_spouse].0, "Schedule SE 2 — Spouse SE nonfarm profit");
    assert_eq!(se[first_spouse].1, 5_934_000);
}

// M3-F1 fixture `m3-self-employed-rental`; native acceptance evidence:
// current verification limits are in COVERAGE.md. Single filer born
// March 14, 1980, no dependents, one
// cash-method sole proprietorship and one residential rental, no other
// income, adjustments, credits or payments. Every expected line below is
// hand-computed from the 2026 Schedule C/SE instructions (i1040sc, i1040sse),
// Form 4562 (i4562, Pub 946), Form 8995 (i8995), Schedule E (i1040se) and
// Form 8582 (i8582) instructions, and Form 1040 (i1040gi); none is copied
// from the App's output. The business's receipts, expenses and vehicle
// mileage reuse `sole_proprietor_return_matches_hand_computation`'s numbers
// (only the new computer asset differs); the rental's depreciable basis and
// in-service date reuse `schedule_e_typical_rental`'s building, so its
// $8,000 mid-month depreciation is unchanged.
#[test]
fn m3_self_employed_rental_return_matches_hand_computation() {
    // Schedule C:
    //   Line 1/7: $88,000.00 (cash method, no COGS, no other gross income).
    //   Line 8 advertising $1,200.00; line 11 contract labor $6,000.00; line
    //     15 insurance $2,400.00; line 18 office $900.00; line 22 supplies
    //     $1,500.00; line 23 taxes/licenses $600.00; line 25 utilities
    //     $1,800.00.
    //   Line 9 car and truck, standard mileage (72.5 cents through June, 76
    //     cents from July, the 2026 rates): 8,000 mi x $0.725 = $5,800.00;
    //     6,000 mi x $0.76 = $4,560.00; total $10,360.00.
    //   Line 24b deductible meals: $1,000.00 x 50% = $500.00.
    //   Line 13 depreciation and section 179 (Form 4562): the $2,400.00
    //     computer (5-year property placed in service February 10, 2026, no
    //     section 179 elected) is acquired after January 19, 2025, so 100%
    //     bonus applies to its full $2,400.00 business basis (Part II line
    //     14); no basis remains for MACRS (Part III). Part IV line 22:
    //     $2,400.00 + $0.00 = $2,400.00.
    //   Line 28 total expenses: 1,200 + 10,360 + 6,000 + 2,400 + 900 + 1,500
    //     + 600 + 500 + 1,800 + 2,400 (line 13) = $27,660.00.
    //   Line 29 tentative profit: 88,000.00 - 27,660.00 = $60,340.00.
    //   Line 30 simplified home office: 200 sq ft x $5/sq ft, all 12 months
    //     = $1,000.00 (far under the $60,340.00 income cap, so not limited).
    //   Line 31 net profit: 60,340.00 - 1,000.00 = $59,340.00.
    //
    // Schedule SE (ordinary nonfarm method, the only business, no W-2 wages,
    // no church employee income):
    //   Line 2/3: $59,340.00. Line 4a-4c: 59,340 x 92.35% = $54,800.49,
    //     rounds to $54,800. Line 6: $54,800. Line 7: $184,500 (2026 Social
    //     Security wage base). Line 9: $184,500 (no W-2 wages).
    //   Lines 10/11 (12.4% and 2.9% of the smaller of line 6/9, $54,800):
    //     54,800 x 15.3% = $8,384.40, rounds to $8,384 (line 12
    //     self-employment tax, Schedule 2 line 4).
    //   Line 13 half-SE deduction (Schedule 1 line 15/26): 8,384 / 2 =
    //     $4,192.
    //
    // Form 8995 (2026 single QBI threshold $201,750, far above this return):
    //   Line 1i/2/4 QBI: net profit less its attributable half-SE deduction,
    //     59,340 - 4,192 = $55,148 (the only business; no SSTB/wage-UBIA
    //     reduction below the threshold).
    //   Line 5: 55,148 x 20% = $11,029.60, rounds to $11,030. Lines 6-9: $0
    //     (no REIT/PTP dividends). Line 10: $11,030.
    //   Line 11 taxable income before QBI: AGI $52,148 (below) less the
    //     $16,100 standard deduction = $36,048. Line 12: $0 (no capital
    //     gain/qualified dividends). Line 13: $36,048. Line 14: 36,048 x 20%
    //     = $7,209.60, rounds to $7,210.
    //   Line 15 (smaller of line 10 and line 14): $7,210. Line 16 ($400
    //     active-business minimum, aggregate active QBI $55,148 >= $1,000):
    //     $400, below line 15. Line 17 QBI deduction: $7,210.
    //
    // Schedule E Part I (the rental building's $220,000 depreciable basis --
    // $260,000 cost less $40,000 land -- and March 1, 2021 in-service date
    // reuse `schedule_e_typical_rental`'s building, so its mid-month
    // depreciation is unchanged):
    //   Line 3 rents received: $24,000.00.
    //   Line 9 insurance: $1,400.00. Line 11 management fees: $2,400.00.
    //   Line 12 mortgage interest: $9,600.00. Line 14 repairs: $1,800.00.
    //   Line 16 taxes: $3,200.00. Line 17 utilities: $600.00.
    //   Line 18 depreciation (27.5-year GDS, mid-month, 2026 is the 6th
    //     calendar year so a full 24/660 half-months apply): $220,000 / 27.5
    //     = $8,000.00.
    //   Line 20 total expenses: 1,400+2,400+9,600+1,800+3,200+600+8,000 =
    //     $27,000.00.
    //   Line 21 income or (loss): 24,000.00 - 27,000.00 = -$3,000.00.
    //   Line 22 deductible loss after Form 8582 (Worksheet 1): the MAGI for
    //     the active-participation allowance is AGI figured before this
    //     rental loss -- the $59,340 business profit less its $4,192
    //     half-SE deduction = $55,148 -- far under the $100,000 phase-out
    //     floor, so the full $25,000 allowance applies unphased. The
    //     $3,000.00 loss is smaller than the allowance, so it is entirely
    //     currently deductible; nothing is suspended.
    //   Line 26 total rental real estate income or (loss): -$3,000.00.
    //
    // Schedule 1: line 3 (business profit) $59,340; line 5 (rental loss)
    //   -$3,000; line 10 (total additional income) $56,340; line 15/26
    //   (half-SE adjustment) $4,192.
    //
    // Form 1040:
    //   Line 8 additional income (Schedule 1 line 10): $56,340.00.
    //   Line 9 total income (no other income source): $56,340.00.
    //   Line 10 adjustments (half-SE deduction, the only one): $4,192.00.
    //   Line 11 AGI: 56,340.00 - 4,192.00 = $52,148.00.
    //   Line 12e standard deduction (single, not 65/blind): $16,100.00.
    //   Line 13b QBI deduction: $7,210.00.
    //   Line 15 taxable income: 52,148.00 - 16,100.00 - 7,210.00 =
    //     $28,838.00.
    //   Line 16 regular tax (2026 draft Tax Table, single, $28,800-$28,850
    //     row, midpoint $28,825): 12,400 x 10% + (28,825 - 12,400) x 12% =
    //     1,240.00 + 1,971.00 = $3,211.00.
    //   Line 23 (Schedule 2 line 4, self-employment tax): $8,384.00.
    //   Line 24a total tax: 3,211.00 + 8,384.00 = $11,595.00.
    //   Line 25a withholding: $0.00 (the receipts are all on 1099-NEC with
    //     no withholding). Line 33 total payments: $0.00 (no estimated
    //     payments either).
    //   Line 37 balance due: $11,595.00.
    let d = m3_self_employed_rental_draft();
    // The computer is the only 2026 MACRS-relevant asset (the van is
    // mileage, not depreciated) and sits entirely in Q1, so the >40% test
    // trivially stays below the mid-quarter threshold.
    assert!(!crate::depreciation::mid_quarter(&d));
    let r = supported(&d);

    // Schedule C.
    assert_eq!(value(&r, "business_2_expense_line8_advertising"), 1_200);
    assert_eq!(value(&r, "business_2_expense_line11_contract_labor"), 6_000);
    assert_eq!(value(&r, "business_2_expense_line15_insurance"), 2_400);
    assert_eq!(value(&r, "business_2_expense_line18_office"), 900);
    assert_eq!(value(&r, "business_2_expense_line22_supplies"), 1_500);
    assert_eq!(value(&r, "business_2_expense_line23_taxes_and_licenses"), 600);
    assert_eq!(value(&r, "business_2_expense_line25_utilities"), 1_800);
    assert_eq!(value(&r, "business_2_expense_line24b_meals"), 500);
    assert_eq!(value(&r, "business_2_vehicle_0_expense"), 10_360);
    assert_eq!(value(&r, "business_2_home_office"), 1_000);
    assert_eq!(cents(&r, "business_2_asset_0_bonus"), 240_000);
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 0);

    // Schedule SE, Form 8995, AGI.
    assert_eq!(value(&r, "self_employment_tax"), 8_384);
    assert_eq!(value(&r, "adjustments"), 4_192);
    assert_eq!(value(&r, "qbi_deduction"), 7_210);
    assert_eq!(value(&r, "agi"), 52_148);

    // Schedule E / Form 8582.
    assert_eq!(value(&r, "rental_3_expense_line9_insurance"), 1_400);
    assert_eq!(value(&r, "rental_3_expense_line11_management_fees"), 2_400);
    assert_eq!(value(&r, "rental_3_expense_line12_mortgage_interest"), 9_600);
    assert_eq!(value(&r, "rental_3_expense_line14_repairs"), 1_800);
    assert_eq!(value(&r, "rental_3_expense_line16_taxes"), 3_200);
    assert_eq!(value(&r, "rental_3_expense_line17_utilities"), 600);
    assert_eq!(value(&r, "rental_3_depreciation"), 8_000);
    assert_eq!(value(&r, "rental_3_suspended_loss"), 0);
    assert_eq!(value(&r, "rental_income"), -3_000);
    assert_eq!(value(&r, "rental_suspended_loss"), 0);

    // Form 1040.
    assert_eq!(return_line(&r, "8"), 5_634_000);
    assert_eq!(return_line(&r, "9"), 5_634_000);
    assert_eq!(return_line(&r, "10"), 419_200);
    assert_eq!(return_line(&r, "11a / 11b"), 5_214_800);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "13b"), 721_000);
    assert_eq!(return_line(&r, "15"), 2_883_800);
    assert_eq!(return_line(&r, "16"), 321_100);
    assert_eq!(return_line(&r, "23"), 838_400);
    assert_eq!(return_line(&r, "24a"), 1_159_500);
    assert_eq!(return_line(&r, "33"), 0);
    assert_eq!(return_line(&r, "37"), 1_159_500);
    assert_eq!(r.tax, Some(1_159_500));
    assert_eq!(r.payments, Some(0));
    assert_eq!(r.balance, Some(1_159_500));
    assert_eq!(restart(d.clone()).draft, d);
}

// M3-F1 boundary (2 of 3): the July 1 standard-mileage rate change
// (schedule-c.md map; i1040sc line 9: 72.5 cents through June, 76 cents from
// July, the 2026 rates), isolated to one vehicle with no other business
// facts.
#[test]
fn vehicle_mileage_blends_the_july_rate_change() {
    // 100 miles at each half-year rate: 100 x $0.725 + 100 x $0.76 =
    // $72.50 + $76.00 = $148.50, which this whole-dollar Schedule C line
    // rounds up to $149 (standard "round to whole dollars" rule: exactly
    // $.50 rounds away from zero, i1040gi).
    let mut d = empty_return(FederalStatus::Single);
    let mut business = cash_business(1_000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.vehicles = vec![native_business_mileage(100, 100, 200)];
    add(&mut d, income(business));
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_vehicle_0_expense"), 149);
}

// ---- T2-35 Form 4797 business-asset dispositions ----

fn no_lookback() -> Section1231History {
    Section1231History { regular: vec![], amt: vec![] }
}
pub(in crate::tests) fn sold(
    date: u32,
    proceeds: i64,
    selling_expenses: i64,
) -> crate::AssetDisposal {
    crate::AssetDisposal::Disposed {
        date,
        how: crate::DisposalKind::Sold {
            livestock: crate::SoldLivestock::No,
            proceeds,
            selling_expenses,
            payment: crate::SalePayment::InFull,
            buyer: crate::Buyer::Unrelated,
        },
    }
}
pub(in crate::tests) fn asset_sale_return(assets: Vec<BusinessAsset>) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(150_000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.assets = assets;
    add(&mut d, income(business));
    d.screening.as_mut().unwrap().section1231 = Some(no_lookback());
    d
}

fn unit13_property_owner_fixture() -> FederalDraft {
    let mut draft = asset_sale_return(vec![press(sold(20260715, 600_000, 20_000))]);
    let joint = complete(FederalStatus::Joint);
    draft.setup = joint.setup;
    draft.people = joint.people;
    let mut second = income(cash_business(0));
    second.owner = crate::DocumentOwner::Spouses;
    let FederalIncome::Business { data } = &mut second.income else { panic!() };
    data.assets = vec![shop_building(sold(20260620, 42_000_000, 2_000_000))];
    add(&mut draft, second);
    draft
}

fn unit13_actual_property_sources(
    draft: &FederalDraft,
) -> Option<crate::section1231::PropertyOwnerAmounts> {
    // The actual draft contains no rentals or K-1s. Its empty activity pool
    // is complete; the source amounts come from the ordinary asset owner.
    let pool = crate::extended_rules::RentalResult { inputs_complete: true, ..Default::default() };
    let character = crate::section1231::worksheet(draft, &pool, false).unwrap();
    let claims = crate::related_forms::effective_claimants(draft);
    let sources =
        crate::asset_sale::allowed_direct_sources_with_claims(draft, &pool, false, &claims)
            .unwrap();
    crate::section1231::direct_owner_amounts(draft, &pool, character.as_ref(), &sources, &[]).map(
        |source| crate::section1231::PropertyOwnerAmounts {
            ordinary: source.ordinary.map(crate::kernel::ratio::dollars),
            long_term: source.long_term.map(crate::kernel::ratio::dollars),
            unrecaptured: source.unrecaptured.map(crate::kernel::ratio::dollars),
        },
    )
}

#[test]
fn unit13_property_owner_sources_keep_native_ordinary_long_and_rate_classes() {
    let mut draft = unit13_property_owner_fixture();
    let source = unit13_actual_property_sources(&draft).unwrap();
    assert_eq!(source.ordinary, [1960, 0]);
    assert_eq!(source.long_term, [0, 114177]);
    assert_eq!(source.unrecaptured, [0, 104177]);
    let before = supported(&draft);
    draft.income.reverse();
    assert_eq!(unit13_actual_property_sources(&draft), Some(source));
    let stored = native_draft_records(&draft);
    let restored = draft_from_native_records(draft.tax_year, &stored);
    assert_eq!(restored, draft);
    assert_eq!(native_draft_records(&restored), stored);
    assert_eq!(unit13_actual_property_sources(&restored), Some(source));
    let after = supported(&restored);
    assert_eq!(
        (before.tax, before.refund, before.balance),
        (after.tax, after.refund, after.balance)
    );
}

#[test]
fn unit13_property_lookback_uses_native_determined_classes_without_pro_rata() {
    let mut draft = unit13_property_owner_fixture();
    let year = Section1231LossYear { year: 2023, original: 300_000, remaining: 300_000 };
    draft.screening.as_mut().unwrap().section1231 =
        Some(Section1231History { regular: vec![year.clone()], amt: vec![year] });
    let source = unit13_actual_property_sources(&draft).unwrap();
    assert_eq!(source.ordinary, [1960, 3000]);
    assert_eq!(source.long_term, [0, 111177]);
    assert_eq!(source.unrecaptured, [0, 101177]);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
    data.assets = vec![shop_building(sold(20260620, 42_000_000, 2_000_000))];
    // Either owner's same-class gain could absorb the3000 lookback. Preserve
    // the ambiguity; matching amounts do not establish a legal allocation.
    assert_eq!(unit13_actual_property_sources(&draft), None);
}

#[test]
fn unit13_property_net_loss_keeps_opposite_owner_ordinary_sources() {
    let mut draft = unit13_property_owner_fixture();
    let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
    data.assets = vec![shop_building(sold(20260620, 42_000_000, 2_000_000))];
    let FederalIncome::Business { data } = &mut draft.income[2].income else { panic!() };
    data.assets = vec![shop_building(sold(20260620, 15_000_000, 0))];
    let source = unit13_actual_property_sources(&draft).unwrap();
    assert_eq!(source.ordinary, [114177, -135823]);
    assert_eq!(source.long_term, [0; 2]);
    assert_eq!(source.unrecaptured, [0; 2]);
}

#[test]
fn unit13_property_sources_reach_final_8379_without_changing_joint_figures() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let mut draft = unit13_property_owner_fixture();
        let plain = supported(&draft);
        draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
        draft.injured_spouse = Some(InjuredSpouseAllocation {
            injured,
            debt: OffsetDebt::ChildSupport,
            obligated: false,
            community: InjuredCommunityFacts::No,
            student_loan_borrower: None,
            estimated: EstimatedSplit::IrsFormula,
            allocations: vec![],
            claimants: vec![],
        });
        let result = supported(&draft);
        // TP:149040 ScheduleC +1960 ordinary asset gain. SP:−4583
        // ScheduleC +114177 long-term property gain. Each native form has
        // already rounded; the property owner never reruns either return.
        assert_eq!(value(&result, "form_8379_line_13b_a"), 260594);
        assert_eq!(
            value(&result, "form_8379_line_13b_b"),
            if injured == Owner::Taxpayer { 151000 } else { 109594 }
        );
        assert_eq!(
            value(&result, "form_8379_line_13b_c"),
            if injured == Owner::Taxpayer { 109594 } else { 151000 }
        );
        assert_eq!(
            (result.tax, result.refund, result.balance),
            (plain.tax, plain.refund, plain.balance)
        );
    }
}

fn unit13_property_editor_fixture(profile: &str) -> (FederalDraft, TaxSection, i64) {
    let (mut draft, topic, corrected) = match profile {
        "business" => (unit13_property_owner_fixture(), TaxSection::SelfEmployment, 262594),
        "rental" => {
            let mut draft = complete(FederalStatus::Joint);
            let mut data = rental(0, 0);
            // The native helper's zero-dollar placeholder is not an entered
            // expense row: the actual editor correctly requires a positive
            // amount in each other-expense row. No expenses means no rows.
            data.expenses.clear();
            data.sale = crate::RentalSale::Sold {
                date: 20260715,
                price: 40_000_000,
                land_price: 7_000_000,
                selling_expenses: 0,
                payment: crate::SalePayment::InFull,
                buyer: crate::Buyer::Unrelated,
                personal_share_percent: 0,
            };
            add(&mut draft, income(FederalIncome::Rental { data }));
            draft.screening.as_mut().unwrap().section1231 = Some(no_lookback());
            (draft, TaxSection::Rentals, 149583)
        }
        _ => panic!("unknown property editor profile"),
    };
    draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
    draft.injured_spouse = Some(InjuredSpouseAllocation {
        injured: Owner::Taxpayer,
        debt: OffsetDebt::ChildSupport,
        obligated: false,
        community: InjuredCommunityFacts::No,
        student_loan_borrower: None,
        estimated: EstimatedSplit::IrsFormula,
        allocations: vec![],
        claimants: vec![],
    });
    (draft, topic, corrected)
}

#[test]
fn unit13_property_actual_client_save_reopen_restart() {
    extern crate std;
    use crate::tests::cross::interview::{accepted, open_topic};
    let profile =
        std::env::var("TAX_UNIT13_PROPERTY_PROFILE").unwrap_or_else(|_| "business".into());
    let (draft, topic, expected) = unit13_property_editor_fixture(&profile);
    let before = supported(&draft);
    assert_eq!(
        value(&before, "form_8379_line_13b_a"),
        if profile == "business" { 260594 } else { 129583 }
    );
    let mut session = Interview::new(TEST_USER, draft);
    let mut page = open_topic(&mut session, topic);
    if let Ok(input) = std::env::var("TAX_UNIT13_PROPERTY_CALC_INPUT") {
        let bytes = std::fs::read(input).unwrap();
        let request = P::from_avro(&bytes).unwrap();
        assert_eq!(request.to_avro(), bytes);
        let Reply::Response(response) = send(&mut session, request) else { panic!() };
        std::fs::write(
            std::env::var("TAX_UNIT13_PROPERTY_CALC_OUTPUT").unwrap(),
            response.to_avro(),
        )
        .unwrap();
        return;
    }
    if let Ok(dir) = std::env::var("TAX_UNIT13_PROPERTY_SEED_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(std::path::Path::new(&dir).join("seed.avro"), page.to_avro()).unwrap();
        return;
    }
    let request = if let Ok(input) = std::env::var("TAX_UNIT13_PROPERTY_PAYLOAD") {
        let bytes = std::fs::read(input).unwrap();
        let request = P::from_avro(&bytes).unwrap();
        assert_eq!(request.to_avro(), bytes);
        request
    } else {
        let P::SaveIncomes { documents, .. } = &mut page else { panic!() };
        let row = &mut documents.documents[0];
        row.owner = Owner::Spouse;
        match &mut row.document {
            crate::IncomeRecord::Business(data) => {
                let crate::AssetDisposal::Disposed {
                    how: crate::DisposalKind::Sold { proceeds, .. },
                    ..
                } = &mut data.assets[0].disposal
                else {
                    panic!()
                };
                *proceeds = 800_000;
            }
            crate::IncomeRecord::Rental(data) => {
                let crate::RentalSale::Sold { price, .. } = &mut data.sale else { panic!() };
                *price = 42_000_000;
            }
            _ => panic!(),
        }
        page
    };
    accepted(&send(&mut session, request));
    let check = |draft: &FederalDraft| {
        let result = supported(draft);
        assert_eq!(value(&result, "form_8379_line_13b_a"), expected);
        assert_eq!(value(&result, "form_8379_line_13b_b"), 0);
        assert_eq!(value(&result, "form_8379_line_13b_c"), expected);
        let mut changed = draft.clone();
        changed.injured_spouse.as_mut().unwrap().injured = Owner::Spouse;
        let swapped = supported(&changed);
        assert_eq!(value(&swapped, "form_8379_line_13b_b"), expected);
        assert_eq!(value(&swapped, "form_8379_line_13b_c"), 0);
        changed.injured_spouse = None;
        let plain = supported(&changed);
        assert_eq!(
            (result.tax, result.refund, result.balance),
            (plain.tax, plain.refund, plain.balance)
        );
        result
    };
    let result = check(&session.draft);
    let stored = native_draft_records(&session.draft);
    let draft = draft_from_native_records(session.draft.tax_year, &stored);
    assert_eq!(native_draft_records(&draft), stored);
    assert_eq!(draft, session.draft);
    let mut reopened = restart(draft);
    let shown = open_topic(&mut reopened, topic);
    if let Ok(dir) = std::env::var("TAX_UNIT13_PROPERTY_OUTPUT") {
        std::fs::create_dir_all(&dir).unwrap();
        let dir = std::path::Path::new(&dir);
        std::fs::write(dir.join("reopened.avro"), shown.to_avro()).unwrap();
        write_native_draft_records(dir.join("persisted.avro"), &stored);
    }
    let accepted_draft = reopened.draft.clone();
    accepted(&send(&mut reopened, shown));
    assert_eq!(reopened.draft, accepted_draft);
    let after = check(&reopened.draft);
    assert_eq!((after.tax, after.refund), (result.tax, result.refund));
}

pub(in crate::tests) fn press(disposal: crate::AssetDisposal) -> BusinessAsset {
    // $10,000 5-year equipment placed March 1, 2024, 200% declining balance,
    // half-year, IRS table: 2024 20% ($2,000) and 2025 32% ($3,200) allowed.
    continuing_asset(1_000_000, 20240301, EquipmentClass::Five, 520_000, disposal)
}
pub(in crate::tests) fn shop_building(disposal: crate::AssetDisposal) -> BusinessAsset {
    // $390,000 nonresidential building placed January 15, 2016, Table A-7a:
    // 2016 2.461% ($9,597.90) and 2017-2025 nine years at 2.564% ($9,999.60
    // each, $89,996.40): $99,594.30 allowed through 2025.
    continuing_asset(39_000_000, 20160115, EquipmentClass::ThirtyNine, 9_959_430, disposal)
}
pub(in crate::tests) fn has_line(r: &FederalResult, id: &str) -> bool {
    r.lines.iter().any(|l| l.id == id)
}
pub(in crate::tests) fn refused_with(d: &FederalDraft, needle: &str) {
    let r = estimate(d);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(r.issues.iter().any(|i| i.message.contains(needle)), "{needle}: {:#?}", r.issues);
}

#[test]
fn asset_held_one_year_or_less_is_ordinary_on_part_ii_line_10() {
    // $10,000 5-year equipment bought and placed July 15, 2025: 2025 20%
    // ($2,000); 2026 half of 32% = $1,600; adjusted basis $6,400; sold for
    // $5,000: a $1,400 loss.
    let asset = |date| {
        continuing_asset(1_000_000, 20250715, EquipmentClass::Five, 200_000, sold(date, 500_000, 0))
    };
    // July 15, 2026 is not more than one year: Part II line 10.
    let r = supported(&asset_sale_return(vec![asset(20260715)]));
    assert_eq!(value(&r, "business_2_asset_0_disposal_ordinary"), -1_400);
    assert!(!has_line(&r, "business_2_asset_0_disposal_section1231"));
    assert_eq!(value(&r, "form4797_line8"), 0);
    // July 16, 2026 is more than one year: a section 1231 loss on line 2,
    // the net loss on line 7, ordinary through line 11.
    let later = supported(&asset_sale_return(vec![asset(20260716)]));
    assert_eq!(value(&later, "business_2_asset_0_disposal_section1231"), -1_400);
    assert!(!has_line(&later, "business_2_asset_0_disposal_ordinary"));
    assert_eq!(value(&later, "form4797_line8"), -1_400);
    // Either way: $150,000 - $1,600 - $1,400 in 1040 line 8.
    assert_eq!(return_line(&r, "8"), 14_700_000);
    assert_eq!(return_line(&later, "8"), 14_700_000);
}

#[test]
fn section_1245_recapture_is_limited_to_depreciation() {
    // The press (adjusted basis $3,840, depreciation $6,160) sold for $12,000:
    // gain $8,160, line 27b $6,160, the $2,000 above depreciation is section
    // 1231 gain (Schedule D line 11).
    let r = supported(&asset_sale_return(vec![press(sold(20260715, 1_200_000, 0))]));
    assert_eq!(value(&r, "business_2_asset_0_disposal_gain"), 8_160);
    assert_eq!(value(&r, "business_2_asset_0_disposal_1245_recapture"), 6_160);
    assert_eq!(value(&r, "form4797_line10"), 2_000);
    assert_eq!(return_line(&r, "7a"), 200_000);
    // Schedule C $150,000 - $960 = $149,040, plus $6,160 recapture.
    assert_eq!(return_line(&r, "8"), 15_520_000);
    // Sold for exactly its cost: the $6,160 gain equals the depreciation, all
    // recaptured, no section 1231 gain.
    let r = supported(&asset_sale_return(vec![press(sold(20260715, 1_000_000, 0))]));
    assert_eq!(value(&r, "business_2_asset_0_disposal_1245_recapture"), 6_160);
    assert_eq!(value(&r, "form4797_line8"), 0);
    assert_eq!(return_line(&r, "8"), 15_520_000);
}

#[test]
fn stopped_business_use_ends_depreciation_without_a_sale() {
    // The press kept for personal use on July 15: half of $1,920, no Form
    // 4797 amounts, Schedule C $150,000 - $960.
    let d = asset_sale_return(vec![press(crate::AssetDisposal::Disposed {
        date: 20260715,
        how: crate::DisposalKind::PersonalUse,
    })]);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 960);
    assert!(!r.lines.iter().any(|l| l.id.contains("_disposal_")));
    assert_eq!(return_line(&r, "8"), 14_904_000);
    // Scrapped instead: the $3,840 adjusted basis is an ordinary loss on
    // line 10 (abandonment).
    let d = asset_sale_return(vec![press(crate::AssetDisposal::Disposed {
        date: 20260715,
        how: crate::DisposalKind::Scrapped,
    })]);
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_asset_0_disposal_ordinary"), -3_840);
    assert_eq!(return_line(&r, "8"), 14_520_000);
}

#[test]
fn asset_disposals_outside_the_calculation_are_refused_by_name() {
    // Unit B11 4797 computes the same-year sale, the carryforward, the personal-use
    // gain and the section 179 conversion; a date before service and a missing
    // section 1231 history stay refused.
    let mut early = bonus_equipment(10_000, 100);
    early.disposal = sold(20260101, 600_000, 0);
    refused_with(&asset_sale_return(vec![early]), "after the date placed in service");
    // No section 1231 lookback records: the gain cannot be characterized.
    let mut d = asset_sale_return(vec![shop_building(sold(20260620, 42_000_000, 0))]);
    d.screening.as_mut().unwrap().section1231 = None;
    unsupported(&d, "Section 1231 lookback");
}

#[test]
fn vehicle_kept_for_personal_use_after_low_business_use_recaptures_in_part_iv_column_b() {
    // A 2025 passenger car whose 2026 business miles are 40% of all miles, used
    // 60% in the earlier year and kept for personal use on 2026-04-01: the excess
    // depreciation over straight line (Form 4797 Part IV column (b)) is Schedule C
    // line 6 income. Without the earlier use (it was never above 50%) nothing is
    // recaptured and the conversion computes.
    let mut car = continuing_asset(
        3_000_000,
        20250301,
        EquipmentClass::Five,
        600_000,
        crate::AssetDisposal::Disposed { date: 20260401, how: crate::DisposalKind::PersonalUse },
    );
    car.listed = ListedProperty::Vehicle {
        total_miles: 10_000,
        business_miles: 4_000,
        operating_costs: 0,
        loan_interest: 0,
        kind: crate::VehicleKind::PassengerCar,
    };
    let r = supported(&asset_sale_return(vec![car.clone()]));
    assert!(!has_line(&r, "business_2_asset_0_listed_recapture"));
    car.earlier_business_percent = Some(60);
    assert!(car.valid());
    let r = supported(&asset_sale_return(vec![car.clone()]));
    assert!(has_line(&r, "business_2_asset_0_listed_recapture"));
    // Used 60% for business in 2026 before the change: no recapture on this
    // return; 2026 depreciation is 32% of the business basis ($18,000 x 32%
    // = $5,760) times 1/2, capped by nothing lower: $2,880.
    car.earlier_business_percent = None;
    car.listed = ListedProperty::Vehicle {
        total_miles: 10_000,
        business_miles: 6_000,
        operating_costs: 0,
        loan_interest: 0,
        kind: crate::VehicleKind::PassengerCar,
    };
    let r = supported(&asset_sale_return(vec![car]));
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 2_880);
}

// T3-982: Form 982 qualified farm and real property business debt, the full
// Part II order, and foreclosures (Form 1099-A and Form 1099-C box 7).
fn capital_loss(short: i64, long: i64) -> Vec<FederalIncomeEntry> {
    [(GainTerm::Short, short), (GainTerm::Long, long)]
        .into_iter()
        .filter(|(_, loss)| *loss > 0)
        .map(|(term, loss)| {
            income(FederalIncome::Sale {
                data: sale_source(crate::tests::support::builders::native_sale(
                    term,
                    0,
                    Some(loss * 100),
                )),
            })
        })
        .collect()
}
fn real_property_debt(value: i64, depreciable_basis: i64) -> FederalIncomeEntry {
    canceled_personal_debt(
        20000,
        PersonalDebtTreatment::RealPropertyDebt {
            data: RealPropertyDebtExclusion {
                insolvency: None,
                principal: 18_500_000,
                value: value * 100,
                other_secured: 0,
                depreciable_basis: depreciable_basis * 100,
                property: vec![
                    property("Store", DebtPropertyKind::Building { secured: true }, 205000),
                    property("Fixtures", DebtPropertyKind::Equipment { secured: false }, 50000),
                ],
                depreciable_first: 0,
            },
        },
    )
}

#[test]
fn qualified_farm_debt_follows_insolvency_within_attributes_plus_qualified_basis() {
    // Pub. 4681 Example 2 pattern. $10,000 of farm debt canceled; insolvent
    // by $11,000 - $7,000 = $4,000 (line 1b). A $10,000 long-term loss leaves
    // a $7,000 capital carryover, the only attribute. Insolvency takes $4,000
    // of it (line 9), leaving $3,000. Line 1c limit: $3,000 attributes plus
    // the $4,000 tractor basis = $7,000, above the remaining $6,000, so all
    // $10,000 is excluded (line 2). The farm $6,000 takes the other $3,000 of
    // capital (line 9 total $7,000) and $3,000 of the tractor (line 11a),
    // leaving it $1,000. Nothing is income.
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, farm_debt(10000, Some((11000, 7000)), 4000));
    for sale in capital_loss(0, 10000) {
        add(&mut d, sale);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), value(&before, "agi") - 3000);
    for (line, amount) in [("1b", 4000), ("1c", 6000), ("2", 10000), ("9", 7000), ("11a", 3000)] {
        assert_eq!(value(&r, &alloc::format!("debt_2_form982_line{line}")), amount, "{line}");
    }
    assert_eq!(value(&r, "debt_2_capital_after"), 0);
    assert_eq!(value(&r, "debt_2_farm_basis_reduction"), 3000);
    assert_eq!(value(&r, "debt_2_property_0_remaining_basis"), 1000);
    assert_eq!(value(&r, "debt_cancellation_2_taxable"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn qualified_farm_debt_limit_boundary_refuses_the_schedule_f_remainder() {
    // Solvent: the line 1c limit is the $7,000 capital carryover plus the
    // tractor basis. $10,000 canceled needs $3,000 of basis: exactly $3,000
    // is accepted; $2,999 leaves $1 above the limit, Schedule F line 8 income
    // this return refuses by name.
    for (tractor, ok) in [(3000, true), (2999, false)] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, farm_debt(10000, None, tractor));
        for sale in capital_loss(0, 10000) {
            add(&mut d, sale);
        }
        if ok {
            let r = supported(&d);
            assert_eq!(value(&r, "debt_2_form982_line1c"), 10000);
            assert_eq!(value(&r, "debt_2_form982_line9"), 7000);
            assert_eq!(value(&r, "debt_2_form982_line11a"), 3000);
        } else {
            unsupported(&d, "Qualified farm debt");
            assert!(estimate(&d).issues.iter().any(|i| {
                let m = &i.display;
                {
                    m.key == "ISSUE_DEBT_FARM_LIMIT_EXCEEDED"
                        && m.args == vec![MessageArg::usd_dollars(9_999)]
                }
            }));
        }
    }
}

#[test]
fn qualified_real_property_business_debt_reduces_the_securing_building() {
    // Pub. 4681 QRPBI example: $20,000 canceled; principal $185,000 over the
    // $165,000 value = $20,000 (limit 1); depreciable real basis $210,000
    // (limit 2). All $20,000 excluded (1d, line 2) and applied to the store
    // that secured it (line 4): $205,000 on January 1, 2027 becomes $185,000.
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, real_property_debt(165000, 210000));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    for (line, amount) in [("1d", 20000), ("2", 20000), ("4", 20000)] {
        assert_eq!(value(&r, &alloc::format!("debt_2_form982_line{line}")), amount, "{line}");
    }
    assert_eq!(value(&r, "debt_2_property_0_remaining_basis"), 185000);
    assert_eq!(value(&r, "debt_2_property_1_remaining_basis"), 50000);
    assert_eq!(value(&r, "debt_2_amt_real_property_basis_reduction"), 20000);
}

#[test]
fn qualified_real_property_business_debt_limits_are_boundaries() {
    // Limit 1: value $165,001 leaves $19,999 of principal above value; limit
    // 2: $19,999 of depreciable real property basis. Either refuses the $1
    // above the limit by name (Schedule C line 6 / Schedule E line 3).
    for (worth, basis, ok) in
        [(165000, 20000, true), (165001, 210000, false), (165000, 19999, false)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, real_property_debt(worth, basis));
        if ok {
            assert_eq!(value(&supported(&d), "debt_2_form982_line4"), 20000);
        } else {
            unsupported(&d, "Qualified real property business debt");
        }
    }
}

#[test]
fn depreciable_first_election_precedes_basis_of_personal_property() {
    // $1,500 chapter 13 discharge. Line 5 election of $1,000 reduces the
    // $1,000 equipment to zero first; the other $500 finds no NOL, credit or
    // capital loss and reduces basis (line 10a) within the $4,000 room
    // ($10,000 basis and cash less $6,000 liabilities): the furniture falls
    // from $5,000 to $4,500.
    let mut d = complete(FederalStatus::Single);
    let mut debt = bankruptcy_debt(1500, 10000, 6000);
    let FederalIncome::PersonalDebt { data } = &mut debt.income else { panic!() };
    let mut data = edit_debt(data);
    let Some(PersonalDebtTreatment::Bankruptcy { data: facts }) = &mut data.treatment else {
        panic!()
    };
    facts.property = vec![
        property("Furniture", DebtPropertyKind::PersonalUse, 5000),
        property("Mower", DebtPropertyKind::Equipment { secured: false }, 1000),
    ];
    facts.depreciable_first = 100000;
    drop(data);
    add(&mut d, debt);
    let r = supported(&d);
    assert_eq!(value(&r, "debt_2_form982_line5"), 1000);
    assert_eq!(value(&r, "debt_2_form982_line10a"), 500);
    assert_eq!(value(&r, "debt_2_property_0_remaining_basis"), 4500);
    assert_eq!(value(&r, "debt_2_property_1_remaining_basis"), 0);
}

#[test]
fn joint_carryovers_and_partial_mixed_capital_losses_are_refused_by_name() {
    // A joint return's capital carryover needs each spouse's share (Treas.
    // Reg. 1.108-7(d)); a single filer's partial reduction of a mixed
    // short/long carryover needs the year-of-origin ledger.
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, insolvency_debt(1500, 7500, 4600, 6100));
    for sale in capital_loss(0, 10000) {
        add(&mut d, sale);
    }
    unsupported(&d, "Insolvency tax attributes");
    assert!(estimate(&d).issues.iter().any(|i| {
        let m = &i.display;
        m.key == "ISSUE_DEBT_JOINT_CARRYOVER_SHARE" && m.args == vec![MessageArg::Number(9)]
    }));
    let mut d = complete(FederalStatus::Single);
    add(&mut d, insolvency_debt(1500, 7500, 4600, 6100));
    // $5,000 short and $5,000 long: $3,000 deducted from short leaves $2,000
    // short and $5,000 long; $1,500 would reduce only part of both.
    for sale in capital_loss(5000, 5000) {
        add(&mut d, sale);
    }
    unsupported(&d, "Insolvency tax attributes");
    assert!(estimate(&d).issues.iter().any(|i| {
        {
            let m = &i.display;
            m.key == "ISSUE_DEBT_PARTIAL_MIXED_CAPITAL"
        }
    }));
}

// A Form 1099-A for a crop forfeited to the CCC, naming the loan lot by its
// record identity.
pub(in crate::tests) fn crop_acquisition(
    balance: i64,
    value: i64,
    loan: i64,
) -> FederalIncomeEntry {
    let mut entry = acquisition(balance, value, false, TransferredProperty::Business);
    let FederalIncome::PropertyAcquisition { data } = &mut entry.income else { unreachable!() };
    let mut data = edit_acquisition(data);
    data.crop = Some(loan);
    drop(data);
    entry
}
pub(in crate::tests) fn acquisition(
    balance: i64,
    value: i64,
    liable: bool,
    property: TransferredProperty,
) -> FederalIncomeEntry {
    income(FederalIncome::PropertyAcquisition {
        data: acquisition_source(PropertyAcquisition {
            date: 20260801,
            balance: balance * 100,
            value: value * 100,
            personally_liable: liable,
            property: Some(property),
            description: alloc::string::String::new(),
            crop: None,
        }),
    })
}

#[test]
fn foreclosed_personal_car_loss_is_not_deductible_and_investment_land_gain_is_taxed() {
    // Recourse car loan: $8,500 balance, $7,000 value: amount realized
    // $7,000 against a $12,000 basis, a $5,000 personal loss that changes
    // nothing. Nonrecourse land: the whole $50,000 debt is realized against a
    // $40,000 basis, a $10,000 long-term gain in AGI.
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(
        &mut d,
        acquisition(
            8500,
            7000,
            true,
            TransferredProperty::PersonalUse {
                basis: 1_200_000,
                term: GainTerm::Long,
                collectible: false,
            },
        ),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(value(&r, "acquisition_2_amount_realized"), 7000);
    add(
        &mut d,
        acquisition(
            50000,
            30000,
            false,
            TransferredProperty::Investment {
                basis: 4_000_000,
                term: GainTerm::Long,
                collectible: false,
            },
        ),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "acquisition_3_amount_realized"), 50000);
    assert_eq!(value(&r, "acquisition_3_gain"), 10000);
    assert_eq!(value(&r, "agi"), value(&before, "agi") + 10000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn ccc_1099a_forfeiture_is_counted_once_with_or_without_the_section_77_election() {
    // Pub 225 (2025) ch. 3 "CCC loans" and the 2026 draft i1040sf lines 5a-5c:
    // the full forfeited amount is reported on line 5b (the CCC may send Form
    // 1099-A for it); with the section 77 election the loan proceeds were
    // already income (5a), so the forfeiture adds no income; without it the
    // forfeiture is the sale, income of the $5,200 loan principal on 5c. A
    // CCC loan is nonrecourse, so the Form 1099-A balance $5,200 is the amount
    // realized (Pub 4681 Table 1-1). Either way the Form 1099-A entry adds
    // nothing to the return: the tax equals the same return without it.
    for (method, note, income_5c) in [
        (CccMethod::Income, "NOTE_CCC_FORFEITURE_ALREADY_REPORTED", 0),
        (CccMethod::Loan, "NOTE_CCC_FORFEITURE_IS_THE_SALE", 5200),
    ] {
        let d = ccc_return(
            CccOrigin::Current,
            method.clone(),
            CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 0 },
            50000,
        );
        let without = supported(&d);
        // 5b is the whole forfeited principal; income is the 5a advance under
        // the election ($5,200) or the 5c sale without it ($5,200): once.
        assert_eq!(ccc_value(&d, &without, "forfeiture_reported"), 5200);
        assert_eq!(ccc_value(&d, &without, "forfeiture"), income_5c);
        assert_eq!(ccc_value(&d, &without, "income"), 5200);
        // The Form 1099-A names the lot by its own record identity.
        let lot = lot_id(&d, 0, 0);
        let mut with = d.clone();
        add(&mut with, crop_acquisition(5200, 4000, lot));
        let r = supported(&with);
        assert!(noted(&r, note), "{note}");
        assert_eq!(r.tax, without.tax);
        assert_eq!(ccc_value(&with, &r, "income"), 5200);
        // A named lot whose principal is not the box 2 balance, a lot that does
        // not exist and a lot of another farm each leave the form unmatched:
        // an issue names the loan.
        for wrong in [
            crop_acquisition(5300, 4000, lot),
            crop_acquisition(5200, 4000, lot + 100),
            crop_acquisition(5200, 4000, 99_000),
        ] {
            let mut other = d.clone();
            add(&mut other, wrong);
            unsupported(&other, "Income: ");
        }
        // The amount alone no longer matches a lot: a business-property
        // 1099-A with the same balance still needs the property's own sale.
        let mut other = d.clone();
        add(&mut other, acquisition(5200, 4000, false, TransferredProperty::Business));
        unsupported(&other, "Income: ");
    }
}

#[test]
fn form_1099c_box_7_property_nonrecourse_and_unsupported_situations() {
    // A recourse car repossession with $1,500 canceled: the $1,500 is income
    // (Schedule 1 line 8c) and the $7,000 box 7 value is the amount realized
    // on the car, whose personal loss is not deductible.
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    let mut debt = canceled_personal_debt(1500, PersonalDebtTreatment::Taxable);
    if let FederalIncome::PersonalDebt { data } = &mut debt.income {
        let mut data = edit_debt(data);
        data.box7_fmv = Some(700_000);
        data.transfer = Some(PropertyTransfer {
            property: Some(TransferredProperty::PersonalUse {
                basis: 1_200_000,
                term: GainTerm::Long,
                collectible: false,
            }),
        });
    }
    add(&mut d, debt.clone());
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), value(&before, "agi") + 1500);
    assert_eq!(value(&r, "debt_2_transfer_amount_realized"), 7000);
    // Box 5 unchecked (nonrecourse) is refused by name.
    let mut nonrecourse = d.clone();
    if let FederalIncome::PersonalDebt { data } = &mut nonrecourse.income[1].income {
        let mut data = edit_debt(data);
        data.personally_liable = false;
    }
    unsupported(&nonrecourse, "Income: ");
    // Each situation this return does not calculate names its line.
    for (situation, text) in [
        (UnsupportedDebt::BusinessDebt, "Schedule C line 6"),
        (UnsupportedDebt::PrincipalResidence, "lines 1e and 10b"),
        (UnsupportedDebt::LiquidationBankruptcy, "line 1a"),
        (UnsupportedDebt::DealerPropertyElection, "line 3"),
        (UnsupportedDebt::Other, "Pub. 4681"),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, canceled_personal_debt(1500, PersonalDebtTreatment::Unsupported { situation }));
        let r = estimate(&d);
        assert!(r.issues.iter().any(|i| i.message.contains(text)), "{situation:?}");
    }
}

#[test]
fn business_not_fully_at_risk_is_left_out_only_with_a_loss() {
    // At-risk limits (Form 6198) apply only to a loss: a profit is calculated
    // normally, without a note.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(10_000)));
    let at_risk = supported(&d);
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.at_risk = crate::RentalAtRisk::from_full_risk(false);
    let r = supported(&d);
    assert_eq!(r.tax, at_risk.tax);
    assert_eq!(value(&r, "self_employment_tax"), 1413);
    assert!(!noted(&r, "WARN_BUSINESS_LOSS_LEFT_OUT"));
    assert!(!noted(&r, "WARN_BUSINESS_LEFT_OUT"));
    // A loss is not deducted: the business is left out with a notice.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(-5_000)));
    let FederalIncome::Business { data } = &mut d.income[1].income else { unreachable!() };
    data.at_risk = crate::RentalAtRisk::from_full_risk(false);
    let r = supported(&d);
    assert!(noted(&r, "WARN_BUSINESS_LOSS_LEFT_OUT"), "{r:?}");
    assert_eq!(r.tax, supported(&complete(FederalStatus::Single)).tax);
}

// 4797-D1: a 2023 net section 1231 loss of $200,000, all nonrecaptured, and a
// $114,177 gain on the shop building. Line 9 is the full balance (i4797 line 8
// and its example: $7,000 on the line against a $2,000 gain), line 10 is
// line 8 less line 9 and the whole gain is ordinary on line 13.
#[test]
fn f13_d1_line_9_is_the_whole_nonrecaptured_loss_balance() {
    let mut d = asset_sale_return(vec![shop_building(sold(20260620, 42_000_000, 2_000_000))]);
    d.screening.as_mut().unwrap().section1231 =
        Some(section1231_history(&[(2023, 200_000, 200_000)], &[(2023, 200_000, 200_000)]));
    let r = supported(&d);
    assert_eq!(value(&r, "form4797_line8"), 114_177);
    assert_eq!(value(&r, "form4797_line9"), 200_000);
    assert_eq!(value(&r, "form4797_line10"), 0);
    // The ordinary part stays the smaller of the two: Schedule 1 line 4.
    assert_eq!(value(&r, "schedule_1_other_gains"), 114_177);
    assert_eq!(value(&r, "capital_gain_11"), 0);
}

// 4797-D3: selling expenses are part of line 23, not netted off line 22.
// The press ($10,000, $6,160 depreciation) sold July 15, 2026 for $6,000 with
// $200 of selling expenses: line 22 $6,000, 23 $10,200, 24 $6,160, 25 $4,040,
// 26 $1,960 (the gain is the same either way).
#[test]
fn f13_d3_selling_expenses_join_line_23() {
    let r = supported(&asset_sale_return(vec![press(sold(20260715, 600_000, 20_000))]));
    for (suffix, amount) in [
        ("sale_price", 6_000),
        ("cost", 10_200),
        ("depreciation", 6_160),
        ("adjusted_basis", 4_040),
        ("gain", 1_960),
    ] {
        assert_eq!(
            value(&r, &alloc::format!("business_2_asset_0_disposal_{suffix}")),
            amount,
            "{suffix}"
        );
    }
}

// 4797-D5: a $7,500 clean-vehicle credit reduced a $60,000 pickup's basis.
// Line 23 keeps the $60,000 cost and the reduction joins line 24: prior
// depreciation 52% x $52,500 = $27,300, 2026 half of 19.2% = $5,040, credit
// $7,500, so line 24 is $39,840, line 25 $20,160 and the $34,840 gain is all
// section 1245 recapture on line 27b (not $2,500 of capital gain).
#[test]
fn f13_d5_clean_vehicle_credit_reduction_joins_line_24() {
    let mut pickup = continuing_asset(
        6_000_000,
        20240301,
        EquipmentClass::Five,
        2_730_000,
        sold(20260715, 5_500_000, 0),
    );
    pickup.credit_reduction = 750_000;
    assert!(pickup.valid());
    let r = supported(&asset_sale_return(vec![pickup]));
    for (suffix, amount) in [
        ("sale_price", 55_000),
        ("cost", 60_000),
        ("depreciation", 39_840),
        ("adjusted_basis", 20_160),
        ("gain", 34_840),
        ("1245_recapture", 34_840),
    ] {
        assert_eq!(
            value(&r, &alloc::format!("business_2_asset_0_disposal_{suffix}")),
            amount,
            "{suffix}"
        );
    }
    assert_eq!(value(&r, "form4797_line8"), 0);
    assert_eq!(value(&r, "schedule_1_other_gains"), 34_840);
}

fn barn(disposal: crate::AssetDisposal) -> BusinessAsset {
    // $100,000 general-purpose farm building (20-year, 150% declining
    // balance, half-year) placed January 15, 2016: Table A-1 allowed 53.154%
    // = $53,154 through 2025.
    continuing_asset(10_000_000, 20160115, EquipmentClass::Twenty, 5_315_400, disposal)
}

// 4797-D6: a 20-year farm building is section 1250 property. Sold July 15,
// 2026 for $120,000: 2026 half of 4.462% = $2,231, depreciation $55,385, gain
// $75,385. Straight line (2.5% + 9 x 5% + half of 5% = 50%) is $50,000, so the
// additional depreciation on line 28a is $5,385, ordinary on line 28g; the
// other $70,000 is section 1231 gain, $50,000 of it unrecaptured section 1250
// gain.
#[test]
fn f13_d6_a_20_year_farm_building_is_section_1250_property() {
    let r = supported(&asset_sale_return(vec![barn(sold(20260715, 12_000_000, 0))]));
    for (suffix, amount) in [
        ("depreciation", 55_385),
        ("gain", 75_385),
        ("additional_depreciation", 5_385),
        ("1250_recapture", 5_385),
    ] {
        assert_eq!(
            value(&r, &alloc::format!("business_2_asset_0_disposal_{suffix}")),
            amount,
            "{suffix}"
        );
    }
    assert!(!has_line(&r, "business_2_asset_0_disposal_1245_recapture"));
    assert_eq!(
        printed_line(&r, "business_2_asset_0_disposal_additional_depreciation"),
        (TaxForm::Form4797, "28a".into())
    );
    assert_eq!(
        printed_line(&r, "business_2_asset_0_disposal_1250_recapture"),
        (TaxForm::Form4797, "28g".into())
    );
    assert_eq!(value(&r, "form4797_line8"), 70_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 50_000);
    assert_eq!(value(&r, "schedule_1_other_gains"), 5_385);
    // A 15-year land improvement is the same kind of property.
    let fence = continuing_asset(
        10_000_000,
        20160115,
        EquipmentClass::Fifteen,
        6_700_000,
        sold(20260715, 12_000_000, 0),
    );
    let r = supported(&asset_sale_return(vec![fence]));
    assert!(has_line(&r, "business_2_asset_0_disposal_1250_recapture"));
    assert!(!has_line(&r, "business_2_asset_0_disposal_1245_recapture"));
}

pub(in crate::tests) fn cow(
    acquired: u32,
    livestock: crate::Livestock,
    date: u32,
) -> BusinessAsset {
    // A purchased $2,000 breeding animal, 5-year 200% half-year: $400 allowed
    // in 2025 and 32% / 2 = $320 in 2026 (placed in service January 10, 2025).
    let mut animal =
        continuing_asset(200_000, 20250110, EquipmentClass::Five, 40_000, sold(date, 260_000, 0));
    animal.acquired = acquired;
    animal.placed = acquired.max(20250110);
    if let crate::AssetDisposal::Disposed {
        how: crate::DisposalKind::Sold { livestock: kind, .. },
        ..
    } = &mut animal.disposal
    {
        *kind = livestock.into();
    }
    animal
}

// 4797-D7: a breeding cow needs 24 months and other livestock 12 months from
// the acquisition date (section 1231(b)(3)); held less, the whole $1,320 gain
// is ordinary on Part II (2026 line 18 for livestock) and line 8 is empty.
#[test]
fn f13_d7_livestock_holding_periods() {
    use crate::Livestock::{CattleOrHorse, No, Other};
    // Sold June 15, 2026 after 17 months: ordinary as cattle, but ordinary
    // property held more than a year is a section 1231 gain (1245 recapture
    // $720 of the $1,320, the rest $600 on line 8).
    let r = supported(&asset_sale_return(vec![cow(20250110, CattleOrHorse, 20260615)]));
    assert_eq!(value(&r, "business_2_asset_0_disposal_ordinary"), 1_320);
    assert_eq!(
        printed_line(&r, "business_2_asset_0_disposal_ordinary"),
        (TaxForm::Form4797, "18".into())
    );
    assert_eq!(value(&r, "form4797_line8"), 0);
    assert_eq!(value(&r, "schedule_1_other_gains"), 1_320);
    let r = supported(&asset_sale_return(vec![cow(20250110, No, 20260615)]));
    assert_eq!(value(&r, "form4797_line8"), 600);
    // Other livestock: 12 months from the acquisition date, so the
    // anniversary itself qualifies, a day earlier does not.
    let r = supported(&asset_sale_return(vec![cow(20250615, Other, 20260615)]));
    assert_eq!(value(&r, "form4797_line8"), 600);
    let r = supported(&asset_sale_return(vec![cow(20250616, Other, 20260615)]));
    assert_eq!(value(&r, "form4797_line8"), 0);
    assert_eq!(value(&r, "business_2_asset_0_disposal_ordinary"), 1_320);
    // Cattle and horses: 24 months, the anniversary qualifies.
    let r = supported(&asset_sale_return(vec![cow(20240615, CattleOrHorse, 20260615)]));
    assert_eq!(value(&r, "form4797_line8"), 600);
    let r = supported(&asset_sale_return(vec![cow(20240616, CattleOrHorse, 20260615)]));
    assert_eq!(value(&r, "form4797_line8"), 0);
}

// ---- Unit B11 4797: business and rental disposition specials (hand-computed) ----

pub(in crate::tests) fn has_note(r: &FederalResult, key: &str) -> bool {
    r.review_notes.iter().any(|n| n.display.key == key)
}

#[test]
fn f1_property_sold_in_the_year_it_is_placed_in_service_keeps_section_179_and_takes_no_macrs() {
    // A $10,000 7-year machine acquired 2024-11-01, placed 2026-02-01, section 179
    // $4,000, sold 2026-09-15 for $8,500 with $100 expenses. No MACRS or bonus
    // (Reg. 1.168(d)-1(b)(3)(ii)); line 22 $8,500, 23 $10,100, 24 $4,000, 25 $6,100,
    // 26 $2,400, 27b min($2,400, $4,000) = $2,400 ordinary, section 1231 $0.
    let mut machine = bonus_equipment(10_000, 100);
    machine.class = EquipmentClass::Seven;
    machine.acquired = 20241101;
    machine.placed = 20260201;
    machine.history = EquipmentHistory::New {
        section179: Section179::Eligible { elected: 400_000 },
        bonus: BonusElection::Apply,
        tables: true,
    };
    machine.disposal = sold(20260915, 850_000, 10_000);
    let r = supported(&asset_sale_return(vec![machine.clone()]));
    assert_eq!(value(&r, "business_2_asset_0_bonus"), 0);
    assert_eq!(value(&r, "business_2_asset_0_macrs"), 0);
    for (suffix, amount) in [
        ("sale_price", 8_500),
        ("cost", 10_100),
        ("depreciation", 4_000),
        ("adjusted_basis", 6_100),
        ("gain", 2_400),
        ("1245_recapture", 2_400),
    ] {
        assert_eq!(
            value(&r, &alloc::format!("business_2_asset_0_disposal_{suffix}")),
            amount,
            "{suffix}"
        );
    }
    // Schedule C $150,000 less the $4,000 section 179, plus the $2,400 recapture.
    assert_eq!(return_line(&r, "8"), 14_840_000);
    // Sold for $5,000: 5,000 - 100 - 6,000 = a $1,100 loss on line 2 (held more
    // than one year from the acquisition date).
    machine.disposal = sold(20260915, 500_000, 10_000);
    let r = supported(&asset_sale_return(vec![machine]));
    assert_eq!(value(&r, "business_2_asset_0_disposal_section1231"), -1_100);
}

#[test]
fn f2_unused_section_179_carryforward_of_a_sold_asset_restores_its_basis() {
    // Continuing 5-year asset: cost $20,000, section 179 $20,000 of which $12,000
    // was deducted and $8,000 carried; sold 2026-03-01 for $9,000. The carryforward
    // leaves Form 4562 line 10 and is part of the adjusted basis (Reg. 1.179-3(f)(1)):
    // line 24 $12,000, line 25 $8,000, gain $1,000, all line 27b.
    let mut asset = continuing_asset(
        2_000_000,
        20240301,
        EquipmentClass::Five,
        1_200_000,
        sold(20260301, 900_000, 0),
    );
    if let EquipmentHistory::Continuing { initial_section179, carryforward, .. } =
        &mut asset.history
    {
        *initial_section179 = 2_000_000;
        *carryforward = 800_000;
    }
    let r = supported(&asset_sale_return(vec![asset]));
    assert!(!has_line(&r, "business_2_asset_0_section179_carryforward"));
    for (suffix, amount) in [
        ("depreciation", 12_000),
        ("adjusted_basis", 8_000),
        ("gain", 1_000),
        ("1245_recapture", 1_000),
    ] {
        assert_eq!(
            value(&r, &alloc::format!("business_2_asset_0_disposal_{suffix}")),
            amount,
            "{suffix}"
        );
    }
}

#[test]
fn f3_mid_quarter_asset_in_its_last_recovery_year_takes_the_disposal_quarter_share() {
    // $12,000 5-year asset placed 2021-11-15 under the mid-quarter convention
    // (Table A-5: 5%, 38%, 22.8%, 13.68%, 10.94%, 9.58%): $10,850.40 allowed through
    // 2025, $1,149.60 unrecovered in the short last year of 21 half-months (24 less
    // the 3 of the first). Sold 2026-05-10 (second-quarter midpoint, 9 half-months):
    // $1,149.60 x 9 / 21 = $492.69; sold 2026-12-01 (21 half-months): the whole.
    let asset = |date| {
        let mut asset = continuing_asset(
            1_200_000,
            20211115,
            EquipmentClass::Five,
            1_085_040,
            sold(date, 50_000, 0),
        );
        let EquipmentHistory::Continuing { mid_quarter, .. } = &mut asset.history else { panic!() };
        *mid_quarter = true;
        asset
    };
    let r = supported(&asset_sale_return(vec![asset(20260510)]));
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 49_269);
    let r = supported(&asset_sale_return(vec![asset(20261201)]));
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 114_960);
}

#[test]
fn f4_gain_on_the_personal_part_is_a_form_8949_row_and_a_loss_is_not_deductible() {
    // 60% business, cost $5,000, sold for $6,000 after more than a year: the personal
    // 40% is $2,400 of the price against $2,000 of cost, a $400 long-term gain on
    // Form 8949 box F, line 10.
    let mut asset =
        continuing_asset(500_000, 20230301, EquipmentClass::Seven, 0, sold(20260715, 600_000, 0));
    asset.listed = ListedProperty::No { business_percent: 60 };
    let r = supported(&asset_sale_return(vec![asset.clone()]));
    assert_eq!(value(&r, "form_8949_2_1_proceeds"), 2_400);
    assert_eq!(value(&r, "form_8949_2_1_basis"), 2_000);
    assert_eq!(value(&r, "form_8949_2_1_gain"), 400);
    // Sold for $4,000: the personal part is a $400 loss, which is not deductible.
    asset.disposal = sold(20260715, 400_000, 0);
    let r = supported(&asset_sale_return(vec![asset]));
    assert!(!has_line(&r, "form_8949_2_1_gain"));
    assert!(has_note(&r, "NOTE_ASSET_PERSONAL_SALE_LOSS"));
}

#[test]
fn condemned_business_property_recognizes_gain_only_above_the_replacement_cost() {
    // The press condemned for $12,000 with $10,000 spent on replacement: the
    // $8,160 realized gain is recognized only to the extent $12,000 exceeds
    // $10,000, so $2,000 (all section 1245 recapture) and $6,160 is postponed.
    let r = supported(&asset_sale_return(vec![press(crate::AssetDisposal::Disposed {
        date: 20260715,
        how: crate::DisposalKind::Condemned {
            award: 1_200_000,
            expenses: 0,
            replacement_cost: 1_000_000,
        },
    })]));
    assert_eq!(value(&r, "business_2_asset_0_disposal_gain"), 8_160);
    assert_eq!(value(&r, "business_2_asset_0_disposal_postponed"), 6_160);
    assert_eq!(value(&r, "business_2_asset_0_disposal_1245_recapture"), 2_000);
    assert!(has_note(&r, "NOTE_CONDEMNATION_RECAPTURE_LIMITED"));
    assert_eq!(return_line(&r, "8"), 15_104_000);
}

#[test]
fn partial_disposition_of_a_business_building_is_a_loss_on_line_2_and_line_1c() {
    // The $390,000 building placed 2016-01-15 with $99,594.30 allowed through 2025:
    // a $39,000 part removed 2026-06-10 for nothing. Depreciation through June is
    // $99,594.30 + $9,999.60 x 11/24 = $104,177.45, the part's tenth is $10,417.75
    // = $10,418; the loss is $39,000 - ($39,000 - $10,418) = $28,582. The building
    // depreciates on 90% of its cost from June: $4,583.15 + $4,874.81 = $9,457.96.
    let mut building = shop_building(crate::AssetDisposal::No);
    building.partial =
        Some(crate::PartialDisposition { date: 20260610, cost_of_part: 3_900_000, proceeds: 0 });
    let r = supported(&asset_sale_return(vec![building]));
    assert_eq!(value(&r, "business_2_asset_0_partial_depreciation"), 10_418);
    assert_eq!(value(&r, "business_2_asset_0_partial_gain"), -28_582);
    assert_eq!(
        printed_line(&r, "business_2_asset_0_partial_gain"),
        (TaxForm::Form4797, "2".into())
    );
    assert_eq!(value(&r, "form4797_line1c"), 28_582);
    assert!(!has_line(&r, "form4797_line1b"));
    assert_eq!(cents(&r, "business_2_asset_0_macrs"), 945_796);
    assert!(has_note(&r, "NOTE_PARTIAL_DISPOSITION_REDUCE_COST"));
}

#[test]
fn a_whole_business_sale_has_inventory_goodwill_and_covenant_treatments() {
    // Inventory $10,000 is Schedule C line 1 income, self-created goodwill $50,000 a
    // long-term Form 8949 row (not section 1231), the $5,000 covenant Schedule 1
    // line 8z (not self-employment income).
    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(150_000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.sale = Some(crate::BusinessSale {
        date: 20261101,
        inventory: 1_000_000,
        goodwill: 5_000_000,
        covenant: 500_000,
        special: false,
    });
    add(&mut d, income(business));
    d.screening.as_mut().unwrap().section1231 = Some(no_lookback());
    let r = supported(&d);
    assert_eq!(value(&r, "business_2_net_profit"), 160_000);
    assert_eq!(value(&r, "business_2_sale_inventory"), 10_000);
    assert_eq!(value(&r, "business_2_sale_covenant"), 5_000);
    assert_eq!(value(&r, "form_8949_2_1_gain"), 50_000);
    assert_eq!(printed_line(&r, "form_8949_2_1_gain"), (TaxForm::Form8949, "1(h)".into()));
    // The Something-else answer leaves the record out with a notice.
    let Some(FederalIncome::Business { data }) = d
        .income
        .iter_mut()
        .map(|e| &mut e.income)
        .find(|i| matches!(i, FederalIncome::Business { .. }))
    else {
        panic!()
    };
    data.sale.as_mut().unwrap().special = true;
    let r = supported(&d);
    assert!(has_note(&r, "WARN_BUSINESS_SALE_LEFT_OUT"));
    assert!(!has_line(&r, "business_2_sale_inventory"));
}
