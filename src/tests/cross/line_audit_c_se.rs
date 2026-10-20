use crate::tests::support::assert::line_amount as cents;
// Line audit of Schedule C, Schedule SE, Form 4562 (with Form 4562-B) and Form 8829
// (2026-10-10, defects D1-D15; fix unit F6). Every expected amount is the audit's
// hand-worked fixture figure from the 2026 drafts (Schedule C 5/15/26 and its
// instructions of Oct 8, 2026; Schedule SE 4/27/26 and its instructions of Aug 5, 2026;
// Form 4562 5/29/26; Form 4562-B December 2026 draft; Form 8829 4/15/26 and its
// instructions of Jul 24, 2026), never the App's own output.
use super::*;

fn has(r: &FederalResult, id: &str) -> bool {
    r.lines.iter().any(|l| l.id == id)
}
fn business_id(d: &FederalDraft) -> i64 {
    d.income.iter().find(|e| matches!(e.income, FederalIncome::Business { .. })).unwrap().id
}
fn business_data(d: &mut FederalDraft) -> &mut FederalBusiness {
    d.income
        .iter_mut()
        .find_map(|e| match &mut e.income {
            FederalIncome::Business { data } => Some(data),
            _ => None,
        })
        .unwrap()
}
fn expense(category: BusinessExpenseCategory, cents: i64) -> FederalBusinessExpense {
    ordinary_business_expense(category, cents)
}
// The emitted Schedule C lines 8 through 27b (the category rows, line 9, 13 and
// 16b) added up, and line 28: they must foot.
fn schedule_c_footing(r: &FederalResult, id: i64) -> (i64, i64) {
    let prefix = alloc::format!("business_{id}_expense_");
    let mut printed: i64 =
        r.lines.iter().filter(|l| l.id.starts_with(&prefix)).map(|l| l.amount).sum();
    for suffix in ["car_and_truck", "vehicle_loan_interest", "depreciation", "depletion"] {
        let key = alloc::format!("business_{id}_{suffix}");
        if has(r, &key) {
            printed += cents(r, &key);
        }
    }
    (printed, cents(r, &alloc::format!("business_{id}_total_expenses")))
}
fn assert_schedule_c_foots(r: &FederalResult, id: i64) {
    let (printed, total) = schedule_c_footing(r, id);
    assert_eq!(printed, total, "Schedule C lines 8-27b vs line 28");
}

// D1: the business share of a standard-mileage vehicle's personal property tax is
// Schedule C line 23, not line 9 (2026 i1040sc line 9, "Standard mileage rate").
#[test]
fn d1_standard_mileage_vehicle_property_tax_is_line_23() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(50_000)));
    let id = business_id(&d);
    // 5,000 business miles January-June and 5,000 July-December of 20,000 miles;
    // vehicle property tax $400.
    business_data(&mut d).vehicles.push(BusinessMileage {
        id: 0,
        property_tax: 40_000,
        ..native_business_mileage(5_000, 5_000, 20_000)
    });
    let r = supported(&d);
    // Line 9 = 5,000 x $0.725 + 5,000 x $0.76 = $7,425; line 23 = $400 x 10,000 / 20,000.
    assert_eq!(cents(&r, &alloc::format!("business_{id}_car_and_truck")), 742_500);
    assert_eq!(
        printed_line(&r, &alloc::format!("business_{id}_car_and_truck")),
        (TaxForm::ScheduleC, "9".into())
    );
    let line23 = alloc::format!("business_{id}_expense_line23_taxes_and_licenses");
    assert_eq!(cents(&r, &line23), 20_000);
    assert_eq!(printed_line(&r, &line23), (TaxForm::ScheduleC, "23".into()));
    // Line 28 is unchanged by where the amount prints.
    assert_eq!(cents(&r, &alloc::format!("business_{id}_total_expenses")), 762_500);
    assert_schedule_c_foots(&r, id);
}

// D2: the employer FICA on reported tips that Form 8846 does not take is on line 23.
#[test]
fn d2_employer_tip_fica_not_taken_by_form_8846_is_on_line_23() {
    let mut d = restaurant_tip_credit_draft();
    let id = business_id(&d);
    let business = restaurant_business_mut(&mut d);
    // Reported tips $19,765; the minimum-wage shortfall excludes $6,695 of them, so
    // Form 8846 line 3 is $13,070 and line 4 $1,000. Employer FICA on the tips $1,530.
    let month = |hours_hundredths, wages_excluding_tips, reported_tips| EmployerTipMonth {
        hours_hundredths,
        wages_excluding_tips,
        reported_tips,
    };
    let claim = business.employer_tip_fica_credit.as_mut().unwrap();
    claim.employees[0].months = vec![
        month(100_000, 0, 600_000),
        month(30_000, 0, 300_000),
        month(100, 515, 1_076_500),
        month(0, 0, 0),
        month(0, 0, 0),
        month(0, 0, 0),
        month(0, 0, 0),
        month(0, 0, 0),
        month(0, 0, 0),
        month(0, 0, 0),
        month(0, 0, 0),
        month(0, 0, 0),
    ];
    claim.actual_employer_tip_fica_paid = 153_000;
    let BusinessPayroll::Yes { statements, .. } = &mut business.payroll else { panic!() };
    statements[0].wages = 1_977_015;
    statements[0].medicare = 1_977_015;
    business.expenses.push(expense(BusinessExpenseCategory::TaxesAndLicenses, 200_000));
    let r = supported(&d);
    assert_eq!(cents(&r, &alloc::format!("business_{id}_form8846_line4")), 100_000);
    // Line 23 = $2,000 + $1,530 - $1,000.
    let line23 = alloc::format!("business_{id}_expense_line23_taxes_and_licenses");
    assert_eq!(cents(&r, &line23), 253_000);
    assert_eq!(printed_line(&r, &line23), (TaxForm::ScheduleC, "23".into()));
    assert_schedule_c_foots(&r, id);
}

// D3: start-up costs and section 197 amortization are Part V / line 27b.
#[test]
fn d3_startup_costs_and_section_197_amortization_are_line_27b() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(50_000)));
    let id = business_id(&d);
    let business = business_data(&mut d);
    // Begun January 5, 2026 with $20,000 of start-up costs: initial deduction $5,000,
    // amortization ($15,000 x 12 / 180) $1,000; bank fees $1,000 on the Other line.
    business.startup = BusinessStartup::Costs {
        costs: 2_000_000,
        started: 20260105,
        capitalize: false,
        eligible: true,
    };
    business
        .expenses
        .push(expense(BusinessExpenseCategory::Other { label: "Bank fees".into() }, 100_000));
    let r = supported(&d);
    let line27b = alloc::format!("business_{id}_expense_line27b_other");
    assert_eq!(cents(&r, &line27b), 700_000);
    assert_eq!(printed_line(&r, &line27b), (TaxForm::ScheduleC, "27b".into()));
    assert_eq!(cents(&r, &alloc::format!("business_{id}_total_expenses")), 700_000);
    assert_schedule_c_foots(&r, id);
    // Section 197 goodwill acquired July 1, 2026: $150,000 x 6 / 180 = $5,000.
    let business = business_data(&mut d);
    business.startup = BusinessStartup::None;
    business.expenses.clear();
    business.intangibles.push(BusinessIntangible {
        label: "Goodwill".into(),
        cost: 15_000_000,
        acquired: 20260701,
        additions: vec![],
        business_started: 20200101,
        eligible: true,
    });
    let r = supported(&d);
    // A business with amortization and no Other expense still prints line 27b.
    assert_eq!(cents(&r, &line27b), 500_000);
    assert_schedule_c_foots(&r, id);
}

// D4: the Form 8881 start-up credit reduces the line that holds the plan's costs.
#[test]
fn d4_form_8881_startup_credit_reduces_line_17() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(50_000)));
    let id = business_id(&d);
    let business = business_data(&mut d);
    // 10 employees, 2 eligible non-highly-compensated employees, $3,000 of plan
    // start-up costs under line 17: line 5 = min($3,000, max($500, 2 x $250)) = $500.
    business.retirement_plan_credit = Some(RetirementPlanCredit {
        startup: Some(PlanStartupCosts {
            first_credit_year: 2026,
            employees: 10,
            eligible_employees: 2,
            costs: 300_000,
        }),
        auto_enrollment_since: None,
        confirmed: true,
    });
    business.expenses.push(expense(BusinessExpenseCategory::LegalAndProfessional, 300_000));
    business.payroll = BusinessPayroll::Yes {
        statements: vec![PayrollW2 {
            label: "Employee A".into(),
            wages: 3_000_000,
            medicare: 3_000_000,
        }],
        attribution: crate::PayrollQbiAttribution::EntireBusiness,
    };
    let r = supported(&d);
    let line17 = alloc::format!("business_{id}_expense_line17_legal_and_professional");
    assert_eq!(cents(&r, &line17), 250_000);
    assert_eq!(printed_line(&r, &line17), (TaxForm::ScheduleC, "17".into()));
    assert_eq!(cents(&r, &alloc::format!("business_{id}_total_expenses")), 250_000);
    assert_schedule_c_foots(&r, id);
}

// D11: amortization that begins in 2026 is Form 4562-B, not a reason to file Form 4562.
fn continuing_five_year_asset() -> BusinessAsset {
    // $10,000 5-year equipment placed June 1, 2024, 200% declining balance, half-year
    // tables, no section 179 or bonus: 2024 20% ($2,000) and 2025 32% ($3,200) were
    // allowed, so 2026 (year 3) is 19.2% = $1,920.
    BusinessAsset {
        cost: 1_000_000,
        acquired: 20240601,
        placed: 20240601,
        history: EquipmentHistory::Continuing {
            initial_section179: 0,
            carryforward: 0,
            prior_depreciation: 520_000,
            amt: EquipmentAmtHistory::Same,
            mid_quarter: false,
            schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
        },
        ..bonus_equipment(0, 100)
    }
}
fn form_rows(r: &FederalResult, form: TaxForm) -> Vec<(alloc::string::String, i64)> {
    r.lines
        .iter()
        .filter_map(|l| {
            let f = l.form_line.as_ref()?;
            (f.form == form).then(|| (f.line.clone(), l.amount))
        })
        .collect()
}

#[test]
fn d11_amortization_beginning_in_the_year_is_form_4562b_and_not_form_4562() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(50_000)));
    let id = business_id(&d);
    let business = business_data(&mut d);
    business.assets.push(continuing_five_year_asset());
    // Goodwill of $150,000 acquired July 1, 2026 in a business begun in 2020:
    // 6 / 180 x $150,000 = $5,000.
    business.intangibles.push(BusinessIntangible {
        label: "Goodwill".into(),
        cost: 15_000_000,
        acquired: 20260701,
        additions: vec![],
        business_started: 20200101,
        eligible: true,
    });
    let r = supported(&d);
    // No Form 4562: the depreciation is Schedule C line 13 alone.
    assert!(form_rows(&r, TaxForm::Form4562).is_empty(), "{r:?}");
    assert_eq!(cents(&r, &alloc::format!("business_{id}_depreciation")), 192_000);
    // Form 4562-B line 1: the goodwill's amortizable amount, the 180-month period and
    // this year's amortization; line 2 is empty and line 3 is $5,000.
    let mut rows = form_rows(&r, TaxForm::Form4562B);
    rows.sort();
    assert_eq!(
        rows,
        vec![
            ("1(c)".into(), 15_000_000),
            ("1(e)".into(), 18_000),
            ("1(f)".into(), 500_000),
            ("2".into(), 0),
            ("3".into(), 500_000),
        ]
    );
    assert_eq!(
        printed_line(&r, &alloc::format!("business_{id}_form_4562b_total")),
        (TaxForm::Form4562B, "3".into())
    );
    assert_schedule_c_foots(&r, id);
}

#[test]
fn d11_amortization_begun_before_the_year_is_form_4562b_line_2() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(50_000)));
    let business = business_data(&mut d);
    // Start-up costs of $20,000 for a business begun March 2024: the 2024 initial
    // deduction was $5,000, and ($15,000 x 12 / 180) $1,000 is amortized in 2026.
    business.startup = BusinessStartup::Costs {
        costs: 2_000_000,
        started: 20240301,
        capitalize: false,
        eligible: true,
    };
    // A second business-wide item: goodwill bought in 2026, $5,000.
    business.intangibles.push(BusinessIntangible {
        label: "Goodwill".into(),
        cost: 15_000_000,
        acquired: 20260701,
        additions: vec![],
        business_started: 20200101,
        eligible: true,
    });
    let r = supported(&d);
    let mut rows = form_rows(&r, TaxForm::Form4562B);
    rows.sort();
    assert_eq!(
        rows,
        vec![
            ("1(c)".into(), 15_000_000),
            ("1(e)".into(), 18_000),
            ("1(f)".into(), 500_000),
            ("2".into(), 100_000),
            ("3".into(), 600_000),
        ]
    );
}

// The emitted Schedule SE lines foot: line 3 combines lines 1a, 1b and 2, line 4a is
// 92.35% of it, line 4c adds line 4b (zero under $400), line 6 adds church employee
// income, lines 10 and 11 give line 12, and line 13 is half of it.
fn assert_schedule_se_foots(r: &FederalResult, church_5b: i64, social_wages: i64) {
    let line = |suffix: &str| -> i64 {
        let id = alloc::format!("taxpayer_{suffix}");
        if has(r, &id) { value(r, &id) } else { 0 }
    };
    let three = line("se_combined_profit");
    assert_eq!(
        three,
        line("se_farm_profit") + line("se_crp_payments") + line("se_nonfarm_profit"),
        "line 3"
    );
    let four_a = if three > 0 { (three * 9_235 + 5_000) / 10_000 } else { three };
    assert_eq!(line("se_net_earnings_ordinary"), four_a, "line 4a");
    let before_test = four_a + line("se_optional_earnings");
    let four_c = if before_test >= 400 { before_test } else { 0 };
    assert_eq!(line("se_net_earnings"), four_c, "line 4c");
    let six = four_c + church_5b;
    let headroom = (18_450_000 - social_wages * 100).max(0) / 100;
    let tax = (six.min(headroom) * 124 / 10 + six * 29 / 10 + 50) / 100;
    assert_eq!(line("self_employment_tax"), tax, "line 12");
    assert_eq!(line("half_self_employment_tax"), (tax + 1) / 2, "line 13");
}

// D5: Schedule SE line 4c under $400 is entered as -0- when there is church employee income.
fn church_wage(box_1: i64) -> FederalIncomeEntry {
    income(FederalIncome::Wage {
        data: FederalWage {
            abroad: 0,
            deferred_pension: 0,
            inmate_pay: 0,
            disability: DisabilityIncome::None,
            wages: box_1,
            withholding: 0,
            social_wages: 0,
            social_tips: 0,
            employee_tips: None,
            work_deductions: FederalWorkDeductions::NONE,
            social_tax: 0,
            medicare_wages: 0,
            medicare_tax: 0,
            uncollected_tax: None,
            medicaid_waiver: 0,
            statutory_business: None,
            paying_scorporation: None,
            payroll: crate::W2Payroll::SocialSecurity,
            coverage: crate::W2Coverage::ChurchEmployee,
            care_benefits: 0,
            statutory_employee: false,
            retirement_plan: false,
            other_box12: Vec::new(),
        },
    })
}

#[test]
fn d5_line_4c_under_400_is_zero_with_church_employee_income() {
    // Church W-2 box 1 $20,000 (boxes 3 and 5 zero) and Schedule C net profit $300.
    // 4a = $300 x 92.35% = $277, so 4c is -0-; 5b = $18,470; line 6 = $18,470; line 10
    // = $2,290.28; line 11 = $535.63; line 12 = $2,826; line 13 = $1,413.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, church_wage(2_000_000));
    add(&mut d, income(cash_business(300)));
    let r = supported(&d);
    assert_eq!(value(&r, "taxpayer_se_net_earnings_ordinary"), 277);
    assert_eq!(value(&r, "taxpayer_se_net_earnings"), 0);
    assert_eq!(value(&r, "taxpayer_self_employment_tax"), 2_826);
    assert_eq!(value(&r, "taxpayer_half_self_employment_tax"), 1_413);
    assert_schedule_se_foots(&r, 18_470, 0);
    assert_eq!(crate::federal_rules::self_employment(30_000, 0, 2_000_000), (18_470, 2_826, 1_413));
    // The optional methods share the rule: a small optional amount is -0- beside church
    // employee income, and still below the $400 stop without it.
    assert_eq!(crate::federal_rules::se_from_net(399, 0, 2_000_000), (18_470, 2_826, 1_413));
    assert_eq!(crate::federal_rules::se_from_net(399, 0, 0), (0, 0, 0));
    assert_eq!(crate::federal_rules::se_from_net(400, 0, 2_000_000).0, 18_870);
}

// D6: the farm optional method's gross income includes Form 1099-G box 7 income.
fn routed_agriculture_payment(dollars: i64) -> FederalIncomeEntry {
    let mut entry = government_unemployment(0, &[], 0);
    let FederalIncome::Unemployment { data } = &mut entry.income else { panic!() };
    data.agriculture_payment = dollars * 100;
    entry
}

#[test]
fn d6_farm_optional_method_counts_1099_g_box_7_income() {
    // Farm sales $6,000, expenses $5,000, and a $3,000 box 7 payment routed to the
    // farm: Schedule F line 9 is $9,000 and line 34 $4,000, and the farm is eligible
    // (gross at most $11,340). Line 15 = min(2/3 x $9,000, $7,560) = $6,000.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(6000, 5000)));
    add(&mut d, routed_agriculture_payment(3000));
    elect_se(&mut d, SeMethod::Farm);
    let r = supported(&d);
    assert_eq!(value(&r, "optional_se_taxpayer_farm"), 6_000);
    assert_eq!(value(&r, "optional_se_taxpayer_net"), 6_000);
    assert_eq!(value(&r, "self_employment_tax"), 918);
    // D9 on the farm method: lines 1a and 1b are skipped, 4b holds line 15, 4c is $6,000.
    assert!(!has(&r, "taxpayer_se_farm_profit"));
    assert!(!has(&r, "taxpayer_se_crp_payments"));
    assert_eq!(value(&r, "taxpayer_se_optional_earnings"), 6_000);
    assert_eq!(value(&r, "taxpayer_se_net_earnings"), 6_000);
    assert_schedule_se_foots(&r, 0, 60_000);
}

// D7: church employee income stays on line 6 when the owner has a partnership K-1.
fn general_partner(box_14_a: i64) -> FederalIncomeEntry {
    let mut partner = ordinary_partner_source(box_14_a, true);
    partner.limited_for_se = false;
    partner.reported_se = Some(box_14_a * 100);
    income(FederalIncome::Partnership { data: partner })
}

#[test]
fn d7_church_employee_income_stays_on_line_6_with_a_partnership_k1() {
    // Church W-2 box 1 $20,000; K-1 (1065) as general partner, box 1 and box 14 code A
    // $10,000. 4c = $10,000 x 92.35% = $9,235; 5b = $18,470; line 6 = $27,705; line 10 =
    // $3,435.42; line 11 = $803.45; line 12 = $4,239; line 13 = $2,120.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, church_wage(2_000_000));
    add(&mut d, general_partner(10_000));
    let r = estimate(&d);
    assert!(r.tax.is_some(), "{:#?}", r.issues);
    assert_eq!(value(&r, "taxpayer_se_net_earnings"), 9_235);
    assert_eq!(value(&r, "taxpayer_self_employment_tax"), 4_239);
    assert_eq!(value(&r, "taxpayer_half_self_employment_tax"), 2_120);
    assert_schedule_se_foots(&r, 18_470, 0);
    assert_eq!(
        crate::federal_rules::self_employment(1_000_000, 0, 2_000_000),
        (27_705, 4_239, 2_120)
    );
}

// D8: farm 1099-G box 7 / 1099-PATR income is on line 2 and in SE earnings on the
// partnership path.
#[test]
fn d8_partnership_path_includes_routed_farm_income() {
    // Farm sales $10,000 with no expenses, a $5,000 box 7 payment routed to the farm and
    // a K-1 (1065) with code A $10,000: 1a $15,000, line 2 $10,000, line 3 $25,000,
    // 4a = $23,088 and SE tax $2,862.91 + $669.55 = $3,532.
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(10_000, 0)));
    add(&mut d, routed_agriculture_payment(5000));
    add(&mut d, general_partner(10_000));
    let r = estimate(&d);
    assert!(r.tax.is_some(), "{:#?}", r.issues);
    assert_eq!(value(&r, "taxpayer_se_farm_profit"), 15_000);
    assert_eq!(value(&r, "taxpayer_se_nonfarm_profit"), 10_000);
    assert_eq!(value(&r, "taxpayer_se_combined_profit"), 25_000);
    assert_eq!(value(&r, "taxpayer_se_net_earnings_ordinary"), 23_088);
    assert_eq!(value(&r, "taxpayer_self_employment_tax"), 3_532);
    assert_schedule_se_foots(&r, 0, 0);
}

// D9: with an optional method, Schedule SE lines 2, 3, 4a and 4b show what the
// instructions say.
#[test]
fn d9_nonfarm_optional_method_prints_lines_2_3_4a_and_4b() {
    // Schedule C receipts $9,000, expenses $6,000 (profit $3,000, under $8,186 and under
    // 72.189% x $9,000); earnings of $400 or more in 2023 and 2024; nonfarm method.
    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(0);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 900_000,
        accounting: BusinessAccounting::Cash,
    };
    data.expenses =
        vec![expense(BusinessExpenseCategory::Other { label: "Operating costs".into() }, 600_000)];
    add(&mut d, income(business));
    elect_se(&mut d, SeMethod::Nonfarm { history: se_history() });
    let r = supported(&d);
    // Line 2 is skipped; line 3 $0; 4a $0; 4b $6,000; 4c $6,000; line 12 $918.
    assert!(!has(&r, "taxpayer_se_nonfarm_profit"));
    assert_eq!(value(&r, "taxpayer_se_combined_profit"), 0);
    assert_eq!(value(&r, "taxpayer_se_net_earnings_ordinary"), 0);
    assert_eq!(value(&r, "taxpayer_se_optional_earnings"), 6_000);
    assert_eq!(
        printed_line(&r, "taxpayer_se_optional_earnings"),
        (TaxForm::ScheduleSe, "4b".into())
    );
    assert_eq!(value(&r, "taxpayer_se_net_earnings"), 6_000);
    assert_eq!(value(&r, "taxpayer_self_employment_tax"), 918);
    assert_schedule_se_foots(&r, 0, 60_000);
}

// D12: the Simplified Method Worksheet's line D (Form 4797 / 8949 losses) is subtracted.
#[test]
fn d12_simplified_office_gross_income_limit_subtracts_the_form_4797_loss() {
    // A $10,000 5-year press placed March 1, 2024 (5,200 depreciated through 2025) sold
    // July 15, 2026 for $3,040: 2026 depreciation 19.2% x $10,000 / 2 = $960, adjusted
    // basis $3,840, a Form 4797 loss of $800. Receipts of $1,960 leave Schedule C line
    // 29 at $1,000 after the $960 of depreciation. A simplified office of 300 square feet
    // all year is $1,500 (line 4); line 1 = $1,000 - $800 = $200 and line 5 = $200.
    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(0);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 196_000,
        accounting: BusinessAccounting::Cash,
    };
    let mut press = continuing_five_year_asset();
    press.acquired = 20240301;
    press.placed = 20240301;
    press.disposal = crate::AssetDisposal::Disposed {
        date: 20260715,
        how: crate::DisposalKind::Sold {
            livestock: crate::Livestock::No,
            proceeds: 304_000,
            selling_expenses: 0,
            payment: crate::SalePayment::InFull,
            buyer: crate::Buyer::Unrelated,
        },
    };
    data.assets = vec![press];
    add(&mut d, income(business));
    let source = d.income.last().unwrap().id;
    let mut claim = simplified_claim(300);
    claim.prior_operating = 0;
    claim.prior_depreciation = 0;
    let OfficeMethod::Simplified { months } = &mut claim.method else { unreachable!() };
    for month in months {
        month.exclusive_days = 15;
    }
    add_office_claim(&mut d, source, claim, 300);

    d.screening.as_mut().unwrap().section1231 =
        Some(Section1231History { regular: vec![], amt: vec![] });
    let r = supported(&d);
    let id = business_id(&d);
    assert_eq!(cents(&r, &alloc::format!("business_{id}_tentative_profit")), 100_000);
    let line30 = alloc::format!("business_{id}_home_office");
    assert_eq!(cents(&r, &line30), 20_000);
    assert_eq!(printed_line(&r, &line30), (TaxForm::ScheduleC, "30".into()));
    assert_eq!(cents(&r, &alloc::format!("business_{id}_net_profit")), 80_000);
}

// Form 8829 fixtures (D13-D15), hand-computed from the 2026 draft form and instructions.

fn line_8829(r: &FederalResult, suffix: &str) -> i64 {
    cents(r, &office_line_id(r, suffix))
}
fn printed_8829(r: &FederalResult, suffix: &str) -> alloc::string::String {
    printed_line(r, &office_line_id(r, suffix)).1
}
// Itemizing: the home's Form 1098 (interest and mortgage insurance premiums) and taxes.

#[test]
fn d13_standard_deduction_route_puts_mortgage_insurance_premiums_on_line_16() {
    // Office 200 of 2,000 square feet (10%); Form 1098 box 1 $12,000 and box 5 $1,800 on
    // the acquisition loan (contract after 2006); line 29 $50,000; no other home
    // expenses. Standard deduction: line 16(b) = $12,000 + $1,800 = $13,800; line 24 =
    // 10% x $13,800 = $1,380. Building $200,000 - $40,000 land = $160,000, business
    // basis $16,000, line 42 = 2.564% = $410.24; line 36 = $1,380 + $410.24.
    let office = actual_office(
        200,
        2_000,
        owned_office_home(1_200_000, 180_000, 0, 20_000_000, 4_000_000, 20200315),
    );
    let r = supported(&office_return(50_000, office));
    assert_eq!(line_8829(&r, "mortgage_interest"), 0);
    assert_eq!(line_8829(&r, "excess_interest"), 1_380_000);
    assert_eq!(printed_8829(&r, "excess_interest"), "16(b)");
    assert_eq!(line_8829(&r, "indirect_total"), 1_380_000);
    assert_eq!(line_8829(&r, "operating_share"), 138_000);
    assert_eq!(line_8829(&r, "depreciation_allowable"), 41_024);
    assert_eq!(line_8829(&r, "allowed"), 179_024);
}

#[test]
fn d13_itemizing_route_puts_the_premiums_on_line_10_with_the_interest() {
    // Itemizing, adjusted gross income under $80,000: the premiums are fully deductible.
    // Line 10(b) = $13,800; no real estate taxes; line 13 = 10% x $13,800 = $1,380.
    let office = actual_office(
        200,
        2_000,
        owned_office_home(1_200_000, 180_000, 0, 20_000_000, 4_000_000, 20200315),
    );
    let mut d = office_return(20_000, office);
    add(&mut d, wage(0, Owner::Taxpayer, 6_000_000, 0));
    office_itemize(&mut d, 900_000, 0, 1_200_000, 180_000);
    let r = supported(&d);
    assert_eq!(line_8829(&r, "mortgage_interest"), 1_380_000);
    assert_eq!(printed_8829(&r, "mortgage_interest"), "10(b)");
    assert_eq!(line_8829(&r, "deductible_indirect"), 1_380_000);
    assert_eq!(line_8829(&r, "indirect_share"), 138_000);
    assert_eq!(line_8829(&r, "unlimited"), 138_000);
    assert_eq!(line_8829(&r, "excess_interest"), 0);
    // 2026-10-17 canonical mortgage source is asked once; changing its premiums
    // changes that same source instead of manufacturing a duplicate mismatch.
    let mut short = d.clone();
    office_itemize(&mut short, 900_000, 0, 1_200_000, 100_000);
    assert_eq!(line_8829(&supported(&short), "mortgage_interest"), 1_300_000);
    // The approved HOME map now computes the premium phase-out from the
    // home-specific modified AGI; it no longer refuses that calculation.
    let mut high = d.clone();
    let wage_id = high.income[1].id;
    high.income[1] = wage(wage_id, Owner::Taxpayer, 15_000_000, 0);
    assert_eq!(line_8829(&supported(&high), "mortgage_interest"), 1_200_000);
}

#[test]
fn d14_line_11_step_1_puts_all_the_real_estate_taxes_in_column_b() {
    // Single, itemizing: state income tax $2,000, real estate tax on the home $6,000, no
    // other property taxes (total $8,000, not more than $10,000); office 10%; Form 1098
    // interest $15,000. Step 1: line 11(b) = $6,000; line 12(b) = $15,000 + $6,000 =
    // $21,000; line 13 = $2,100; line 14 = $2,100; line 17 is blank.
    let office = actual_office(
        250,
        2_500,
        owned_office_home(1_500_000, 0, 600_000, 40_000_000, 10_000_000, 20180105),
    );
    let mut d = office_return(60_000, office.clone());
    office_itemize(&mut d, 200_000, 600_000, 1_500_000, 0);
    let r = supported(&d);
    assert_eq!(line_8829(&r, "real_estate_taxes"), 600_000);
    assert_eq!(printed_8829(&r, "real_estate_taxes"), "11(b)");
    assert_eq!(line_8829(&r, "deductible_indirect"), 2_100_000);
    assert_eq!(printed_8829(&r, "deductible_indirect"), "12(b)");
    assert_eq!(line_8829(&r, "deductible_direct"), 0);
    assert_eq!(line_8829(&r, "indirect_share"), 210_000);
    assert_eq!(line_8829(&r, "unlimited"), 210_000);
    assert_eq!(line_8829(&r, "excess_taxes"), 0);
    // Schedule A still gives up the business share of the taxes (line 5b).
    assert_eq!(line_8829(&r, "worksheet_business_taxes"), 60_000);
    // Step 2 (state and local taxes above $10,000): the business share is line 11(a).
    let mut over = office_return(60_000, office);
    office_itemize(&mut over, 900_000, 600_000, 1_500_000, 0);
    let r = supported(&over);
    assert_eq!(line_8829(&r, "real_estate_taxes"), 60_000);
    assert_eq!(printed_8829(&r, "real_estate_taxes"), "11(a)");
    assert_eq!(line_8829(&r, "deductible_direct"), 60_000);
    assert_eq!(line_8829(&r, "deductible_indirect"), 1_500_000);
    assert_eq!(line_8829(&r, "unlimited"), 210_000);
}

#[test]
fn d15_form_8829_lines_20_and_23_have_columns_not_letters() {
    // The standard-deduction fixture: 300 of 2,000 square feet (15%), an owned home first
    // used in 2020, insurance $1,200, repairs $800, direct office repairs $400,
    // utilities $3,000. Line 20 (a) $400 and (b) $800; line 23 (a) $400 and (b) $21,000.
    let mut office = actual_office(
        300,
        2_000,
        owned_office_home(1_200_000, 0, 400_000, 30_000_000, 6_000_000, 20200315),
    );
    office.insurance = 120_000;
    office.repairs = 80_000;
    office.utilities = 300_000;
    office.direct = 40_000;
    let r = supported(&office_return(50_000, office));
    for (suffix, amount, line) in [
        ("direct_repairs", 40_000, "20(a)"),
        ("repairs", 80_000, "20(b)"),
        ("direct_total", 40_000, "23(a)"),
        ("indirect_total", 2_100_000, "23(b)"),
        ("excess_interest", 1_200_000, "16(b)"),
        ("excess_taxes", 400_000, "17(b)"),
        ("insurance", 120_000, "18(b)"),
        ("utilities", 300_000, "21(b)"),
    ] {
        assert_eq!(line_8829(&r, suffix), amount, "{suffix}");
        assert_eq!(printed_8829(&r, suffix), line, "{suffix}");
        assert_eq!(printed_line(&r, &office_line_id(&r, suffix)).0, TaxForm::Form8829);
    }
}

// D10: recapture of excess depreciation when a vehicle's business use falls to 50% or less
// (section 280F(b)(2); Form 4797 Part IV; Schedule C line 6).
fn continuing_vehicle(
    cost: i64,
    placed: u32,
    prior_depreciation: i64,
    miles: (u32, u32),
    passenger: bool,
    earlier: Option<u32>,
) -> BusinessAsset {
    BusinessAsset {
        cost,
        acquired: placed,
        placed,
        history: EquipmentHistory::Continuing {
            initial_section179: 0,
            carryforward: 0,
            prior_depreciation,
            amt: EquipmentAmtHistory::Same,
            mid_quarter: false,
            schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
        },
        listed: ListedProperty::Vehicle {
            total_miles: miles.0,
            business_miles: miles.1,
            operating_costs: 0,
            loan_interest: 0,
            passenger_automobile: passenger,
        },
        earlier_business_percent: earlier,
        ..bonus_equipment(0, 100)
    }
}

#[test]
fn d10_passenger_car_used_50_percent_or_less_recaptures_the_excess_depreciation() {
    // A $30,000 passenger car placed in service March 1, 2025, used 80% in 2025, GDS 200%
    // half-year tables, no bonus or section 179: 2025 depreciation was
    // $30,000 x 80% x 20% = $4,800 (under the $12,200 x 80% limit). In 2026 it was driven
    // 4,000 of 10,000 miles (40%). ADS for 2025 would have been 10% x $24,000 = $2,400, so
    // the excess depreciation is $4,800 - $2,400 = $2,400, ordinary income on Schedule C
    // line 6. The 2026 line 27(h) is Table A-8 year 2: 20% x $12,000 = $2,400 (the
    // $19,600 x 40% limit does not bind).
    let car = continuing_vehicle(3_000_000, 20250301, 480_000, (10_000, 4_000), true, Some(80));
    assert!(car.valid());
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(100_000)));
    let id = business_id(&d);
    business_data(&mut d).assets.push(car.clone());
    let r = supported(&d);
    let asset = |suffix: &str| alloc::format!("business_{id}_asset_0_{suffix}");
    assert_eq!(cents(&r, &asset("macrs")), 240_000);
    assert_eq!(cents(&r, &asset("listed_depreciation")), 240_000);
    assert_eq!(
        printed_line(&r, &asset("listed_depreciation")),
        (TaxForm::Form4562, "27(h)".into())
    );
    // Form 4797 Part IV, section 280F(b)(2) column.
    for (suffix, amount, line) in [
        ("listed_prior_depreciation", 480_000, "35(b)"),
        ("listed_recomputed", 240_000, "36(b)"),
        ("listed_recapture", 240_000, "37(b)"),
    ] {
        assert_eq!(cents(&r, &asset(suffix)), amount, "{suffix}");
        assert_eq!(printed_line(&r, &asset(suffix)), (TaxForm::Form4797, line.into()));
    }
    // Schedule C line 6 carries the excess, so the profit is the receipts less line 13
    // plus the recapture; the basis is adjusted by it.
    assert_eq!(cents(&r, &alloc::format!("business_{id}_other_income")), 240_000);
    assert_eq!(
        printed_line(&r, &alloc::format!("business_{id}_other_income")),
        (TaxForm::ScheduleC, "6".into())
    );
    assert_eq!(cents(&r, &alloc::format!("business_{id}_net_profit")), 10_000_000);
    assert_eq!(cents(&r, &asset("remaining_basis")), 720_000);
    assert_schedule_c_foots(&r, id);
    // Without the earlier use (used 50% or less since it was placed in service) there is
    // no recapture.
    let plain = continuing_vehicle(3_000_000, 20250301, 480_000, (10_000, 4_000), true, None);
    business_data(&mut d).assets = vec![plain];
    let r = supported(&d);
    assert!(!has(&r, &asset("listed_recapture")));
    assert_eq!(cents(&r, &alloc::format!("business_{id}_other_income")), 0);
    // The earlier use is for a vehicle that was used more than 50% before and is not now.
    let mut over = car.clone();
    over.listed = ListedProperty::Vehicle {
        total_miles: 10_000,
        business_miles: 6_000,
        operating_costs: 0,
        loan_interest: 0,
        passenger_automobile: true,
    };
    over.earlier_business_percent = Some(80);
    assert!(!over.valid());
}

#[test]
fn d10_recapture_sums_the_straight_line_years_of_a_heavier_vehicle() {
    // A $30,000 truck over 6,000 pounds placed June 1, 2024, used 60% earlier: GDS 200%
    // half-year allowed 20% + 32% = 52% of $18,000 = $9,360 through 2025; ADS would have
    // allowed (10% + 20%) x $18,000 = $5,400; this year 5,000 of 10,000 miles (50%) is
    // not more than 50%, so the excess is $9,360 - $5,400 = $3,960 (no section 280F limit
    // applies to a heavier vehicle).
    let truck = continuing_vehicle(3_000_000, 20240601, 936_000, (10_000, 5_000), false, Some(60));
    assert!(truck.valid());
    let recapture = truck.recapture().unwrap();
    assert_eq!((recapture.allowed, recapture.recomputed), (936_000, 540_000));
    assert_eq!(recapture.excess(), 396_000);
    // Used 60% in 2026 there is nothing to recapture.
    let mut still = truck.clone();
    still.listed = ListedProperty::Vehicle {
        total_miles: 10_000,
        business_miles: 6_000,
        operating_costs: 0,
        loan_interest: 0,
        passenger_automobile: false,
    };
    assert!(still.recapture().is_none());
}

#[test]
fn schedule_c_specials_lines_1_to_31_foot_with_every_new_expense_owner() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(cash_business(110_000)));
    let id = business_id(&d);
    let b = business_data(&mut d);
    if let BusinessActivity::Ordinary { returns, .. } = &mut b.activity {
        *returns = 120_000;
    }
    b.other_income =
        vec![OtherBusinessIncome { description: "Scrap sales".into(), amount: 70_000 }];
    b.inventory = BusinessInventory::Cost {
        method: InventoryMethod::Cost,
        opening: 1_200_000,
        purchases: 4_000_000,
        personal: 50_000,
        labor: 0,
        materials: 0,
        other: 0,
        closing: 900_000,
        eligible: true,
    };
    b.expenses = vec![
        expense(BusinessExpenseCategory::EmployeePension, 195_000),
        expense(BusinessExpenseCategory::Meals, 150_000),
    ];
    b.de_minimis = Some(DeMinimisElection {
        purchases: 420_000,
        largest_item: 180_000,
        financial_statement: false,
        eligible: true,
    });
    b.leased_vehicles = vec![BusinessLeasedVehicle {
        began: 20260301,
        ends: None,
        fair_market_value: 8_400_000,
        lease_payments: 780_000,
        total_miles: 16_000,
        business_miles: 12_000,
        commuting: 0,
        operating_costs: 320_000,
        parking_tolls: 15_000,
        passenger_automobile: true,
        personal_use_available: true,
        other_vehicle: true,
        evidence: VehicleEvidence::Written,
    }];
    b.depletion = Some(BusinessDepletion {
        kind: RoyaltyKind::Mineral { rate: RoyaltyMineralRate::Fifteen },
        depletion: RoyaltyDepletion::Cost {
            adjusted_basis: 200_000,
            recoverable_units: 10,
            units_sold: 1,
        },
    });
    let r = supported(&d);
    assert_schedule_c_foots(&r, id);
    assert_eq!(cents(&r, &alloc::format!("business_{id}_net_receipts")), 10_880_000);
    assert_eq!(cents(&r, &alloc::format!("business_{id}_gross_profit")), 6_630_000);
    assert_eq!(cents(&r, &alloc::format!("business_{id}_gross_income")), 6_700_000);
    assert_eq!(cents(&r, &alloc::format!("business_{id}_total_expenses")), 1_618_000);
    assert_eq!(cents(&r, &alloc::format!("business_{id}_net_profit")), 5_082_000);
}
