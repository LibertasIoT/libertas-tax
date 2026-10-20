use crate::tests::support::assert::line_amount as cents;
// Line audit of Form 8995 / 8995-A and Schedule F (2026-10-10, defects QBI-D1 to QBI-D5 and
// SF-D1 to SF-D7; fix unit F12; QBI-D6 and QBI-D7 are tested beside their fixtures in
// farm_rental.rs and partnership_no_statement.rs). Every expected amount is the audit's
// hand-worked figure from the 2026 drafts (Forms 8995, 8995-A, Schedule F, Schedule J, Form 4835
// and their instructions; the 2025 Form 8995-A instructions and Pub. 225 where no 2026 text
// exists), never the App's own output.
use super::*;
use crate::tests::cross::interview::{
    accepted, business as business_page, cash_farm, document, farm_save, reopened_businesses,
    reopened_farming, save_topic, stored_business,
};
use crate::tests::farm::farm::cooperative;

fn has(r: &FederalResult, id: &str) -> bool {
    r.lines.iter().any(|l| l.id == id)
}
// The amounts of the printed rows of one form line, in whole dollars.
fn form_rows(r: &FederalResult, form: TaxForm, line: &str) -> Vec<i64> {
    r.lines
        .iter()
        .filter(|l| l.form_line.as_ref().is_some_and(|f| f.form == form && f.line == line))
        .map(|l| l.amount / 100)
        .collect()
}
fn one_row(r: &FederalResult, form: TaxForm, line: &str) -> i64 {
    let rows = form_rows(r, form, line);
    assert_eq!(rows.len(), 1, "{form:?} line {line}: {rows:?}");
    rows[0]
}
// 20% of a whole-dollar amount, rounded half up (the forms' multiplication by 0.20).
fn twenty(amount: i64) -> i64 {
    (amount * 20 + 50) / 100
}
// The Form 8995 lines foot: each line is computed from the printed lines above it.
fn assert_form_8995_foots(r: &FederalResult) {
    let f = |line: &str| one_row(r, TaxForm::Form8995, line);
    let line_1: i64 = form_rows(r, TaxForm::Form8995, "1").iter().sum();
    assert_eq!(f("2"), line_1, "line 2 is the sum of the line 1 rows");
    assert_eq!(f("4"), (f("2") + f("3")).max(0), "line 4");
    assert_eq!(f("5"), twenty(f("4")), "line 5 is 20% of line 4");
    assert_eq!(f("8"), (f("6") + f("7")).max(0), "line 8");
    assert_eq!(f("9"), twenty(f("8")), "line 9");
    assert_eq!(f("10"), f("5") + f("9"), "line 10");
    assert_eq!(f("13"), (f("11") - f("12")).max(0), "line 13");
    assert_eq!(f("14"), twenty(f("13")), "line 14");
    assert_eq!(f("15"), f("10").min(f("14")), "line 15");
    assert_eq!(f("17"), f("15").max(f("16")), "line 17");
    assert_eq!(value(r, "qbi_deduction"), f("17"));
}
fn business_id(d: &FederalDraft) -> i64 {
    d.income.iter().find(|e| matches!(e.income, FederalIncome::Business { .. })).unwrap().id
}
fn business_mut(d: &mut FederalDraft) -> &mut FederalBusiness {
    d.income
        .iter_mut()
        .find_map(|e| match &mut e.income {
            FederalIncome::Business { data } => Some(data),
            _ => None,
        })
        .unwrap()
}
fn farm_of(b: &mut FederalBusiness) -> &mut FarmIncome {
    let BusinessActivity::Farming { data } = &mut b.activity else { unreachable!() };
    data
}
fn old_asset(placed: u32, class: EquipmentClass) -> BusinessAsset {
    BusinessAsset {
        cost: 100_000_000,
        acquired: placed,
        placed,
        class,
        history: EquipmentHistory::Continuing {
            initial_section179: 0,
            carryforward: 0,
            prior_depreciation: 100_000_000,
            amt: EquipmentAmtHistory::Same,
            mid_quarter: false,
            schedule: EquipmentSchedule::Manual,
        },
        method: EquipmentMethod::StraightLine,
        ..bonus_equipment(0, 100)
    }
}
fn ag_payment(dollars: i64) -> FederalIncomeEntry {
    let mut entry = government_unemployment(0, &[], 0);
    let FederalIncome::Unemployment { data } = &mut entry.income else { panic!() };
    data.agriculture_payment = dollars * 100;
    entry
}

// ---- Form 8995 and Form 8995-A ----------------------------------------------

// QBI-D1: an amount deductible as qualified tips under section 224(a) is not qualified business
// income (2026 i8995 What's New and line 1). Single, W-2 $60,000, Schedule C profit $60,000 with
// $10,000 of eligible tips: SE tax 60,000 x 92.35% x 15.3% = 8,478 (half 4,239); AGI 115,761;
// tips deduction 10,000; line 11 = 115,761 - 16,100 - 10,000 = 89,661, line 14 = 17,932; line 1 =
// 60,000 - 4,239 - 10,000 = 45,761, line 5 = 9,152.
#[test]
fn qbi_d1_qualified_tips_are_not_qualified_business_income() {
    let mut d = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut b } = cash_business(60_000) else { panic!() };
    b.tips = Some(FederalBusinessTips { amount: 1_000_000, eligible: true });
    add(&mut d, income(FederalIncome::Business { data: b }));
    let r = supported(&d);
    let id = business_id(&d);
    assert_eq!(value(&r, "self_employment_tax"), 8478);
    assert_eq!(value(&r, "agi"), 115_761);
    assert_eq!(value(&r, "tips_deduction"), 10_000);
    assert_eq!(value(&r, &alloc::format!("business_{id}_qbi")), 45_761);
    assert_eq!(one_row(&r, TaxForm::Form8995, "11"), 89_661);
    assert_eq!(one_row(&r, TaxForm::Form8995, "14"), 17_932);
    assert_eq!(one_row(&r, TaxForm::Form8995, "5"), 9_152);
    assert_eq!(value(&r, "qbi_deduction"), 9_152);
    assert_form_8995_foots(&r);
    // Tips not confirmed as eligible claim no deduction, so they stay in QBI.
    let mut capped = d.clone();
    business_mut(&mut capped).tips =
        Some(FederalBusinessTips { amount: 1_000_000, eligible: false });
    let r = supported(&capped);
    assert_eq!(value(&r, &alloc::format!("business_{id}_qbi")), 55_761);
}

// QBI-D1: the Business and Farm pages enter the tips the deduction reads.
#[test]
fn qbi_d1_the_business_and_farm_pages_take_qualified_tips() {
    let tips = Some(FederalBusinessTips { amount: 500_000, eligible: true });
    let mut s = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
    let mut page = business_page(6_000_000);
    page.more.tips = tips.clone();
    let request = save_topic(
        &s.draft.clone(),
        TaxSection::SelfEmployment,
        vec![document(0, IncomeDocument::Business(page.clone()))],
    );
    accepted(&send(&mut s, request));
    let (_, stored) = stored_business(&s);
    assert_eq!(stored.tips, tips);
    assert_eq!(reopened_businesses(s.draft.clone()), vec![page]);
    let mut s = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
    let mut farm = cash_farm(2_000_000, 500_000);
    farm.more.tips = tips.clone();
    let request =
        farm_save(&s.draft.clone(), vec![document(0, IncomeDocument::Farm(farm.clone()))]);
    accepted(&send(&mut s, request));
    let (_, stored) = stored_business(&s);
    assert_eq!(stored.tips, tips);
    assert_eq!(reopened_farming(&mut restart(s.draft.clone())).1, vec![IncomeDocument::Farm(farm)]);
}

// QBI-D2: Form 8995 line 5 is 20% of line 4, not a sum of rounded activity amounts (2026 i8995
// line 5; only Form 8995-A line 3 rounds each column). Two businesses with profit 1,003 each:
// line 4 = 1,864, line 5 = 372.80 -> 373 (the per-activity sum is 186 + 186 = 372); the businesses
// are active with QBI of at least $1,000, so the deduction is the $400 minimum (line 16). Profits
// 5,001 and 7,003: line 4 = 11,156, line 5 = 2,231.20 -> 2,231 (the sum is 930 + 1,302 = 2,232),
// which is also the deduction.
#[test]
fn qbi_d2_form_8995_line_5_is_twenty_percent_of_line_4() {
    for (a, b, line_4, line_5, deduction) in
        [(1_003, 1_003, 1_864, 373, 400), (5_001, 7_003, 11_156, 2_231, 2_231)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(cash_business(a)));
        add(&mut d, income(cash_business(b)));
        let r = supported(&d);
        assert_eq!(one_row(&r, TaxForm::Form8995, "4"), line_4);
        assert_eq!(one_row(&r, TaxForm::Form8995, "5"), line_5);
        assert_eq!(value(&r, "qbi_deduction"), deduction);
        assert_form_8995_foots(&r);
    }
}

fn patron_return() -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(40000, 10000)));
    let mut patr = cooperative(1_600_000, 100_000, 300_000, 200_000);
    cooperative_personal(&mut patr, 200_000);
    patr.withholding = 30_000;
    patr.qualified_payments = 800_000;
    patr.allocations[0].qualified_payments = 800_000;
    add(&mut d, income(FederalIncome::Cooperative { data: patr }));
    d
}

// QBI-D3: a patron of a specified cooperative (Form 1099-PATR box 7) files Form 8995-A with
// Schedule D, not Form 8995 (2026 Form 8995 header and i8995). Schedule D line 6 is the lesser of
// 9% of the QBI allocable to the qualified payments or 50% of the allocable W-2 wages: $0 for a
// farm with no W-2 wages. Farm line 34 is 50,000 (20,000 of it distributions); with box 8 empty
// the QBI is 50,000 - 20,000 - 3,533 = 26,467 (QBI-D4), column line 3 = 5,293, line 41 = 5,293.
#[test]
fn qbi_d3_a_cooperative_patron_files_form_8995a_with_schedule_d() {
    let d = patron_return();
    let r = supported(&d);
    let farm = d.income[0].id;
    assert_eq!(printed_line(&r, "qbi_deduction"), (TaxForm::Form8995A, "41".into()));
    assert!(form_rows(&r, TaxForm::Form8995, "17").is_empty());
    assert_eq!(
        printed_line(&r, "qbi_patron_reduction"),
        (TaxForm::Form8995A, "Schedule D 6".into())
    );
    assert_eq!(value(&r, "qbi_patron_reduction"), 0);
    let column = |suffix: &str| value(&r, &alloc::format!("business_{farm}_qbi{suffix}"));
    assert_eq!(column(""), 26_467);
    assert_eq!(column("_twenty_percent"), 5_293);
    assert_eq!(column("_limited"), 5_293);
    assert_eq!(column("_patron_reduction"), 0);
    assert_eq!(column("_component"), 5_293);
    assert_eq!(
        printed_line(&r, &alloc::format!("business_{farm}_qbi_limited")),
        (TaxForm::Form8995A, "13".into())
    );
    assert_eq!(
        printed_line(&r, &alloc::format!("business_{farm}_qbi_patron_reduction")),
        (TaxForm::Form8995A, "14".into())
    );
    assert_eq!(one_row(&r, TaxForm::Form8995A, "37"), 5_293);
    assert_eq!(one_row(&r, TaxForm::Form8995A, "41"), 5_293);
    // The same return without box 7 files Form 8995.
    let mut plain = d.clone();
    let FederalIncome::Cooperative { data } = &mut plain.income[1].income else { unreachable!() };
    data.qualified_payments = 0;
    data.allocations[0].qualified_payments = 0;
    let r = supported(&plain);
    assert_eq!(printed_line(&r, "qbi_deduction"), (TaxForm::Form8995, "17".into()));
    assert!(!has(&r, "qbi_patron_reduction"));
    assert_form_8995_foots(&r);
}

// QBI-D4: Form 1099-PATR payments are qualified business income only to the extent the
// cooperative reports them as qualified items in box 8 (2026 i8995, Form 1099-PATR items). With
// box 8 empty none of the $20,000 on line 3b is QBI: QBI = 50,000 - 20,000 - 3,533 = 26,467,
// component 5,293 against the taxable-income limit 6,073.
#[test]
fn qbi_d4_patronage_distributions_are_qualified_only_as_box_8_reports() {
    let mut d = patron_return();
    let FederalIncome::Cooperative { data } = &mut d.income[1].income else { unreachable!() };
    data.qualified_payments = 0;
    data.allocations[0].qualified_payments = 0;
    let r = supported(&d);
    let farm = d.income[0].id;
    assert_eq!(value(&r, "self_employment_tax"), 7_065);
    assert_eq!(one_row(&r, TaxForm::Form8995, "11"), 30_367);
    assert_eq!(one_row(&r, TaxForm::Form8995, "14"), 6_073);
    assert_eq!(value(&r, &alloc::format!("business_{farm}_qbi")), 26_467);
    assert_eq!(one_row(&r, TaxForm::Form8995, "5"), 5_293);
    assert_eq!(value(&r, "qbi_deduction"), 5_293);
    assert_form_8995_foots(&r);
    // Box 8 of 12,000 makes 12,000 of the 20,000 qualified.
    let FederalIncome::Cooperative { data } = &mut d.income[1].income else { unreachable!() };
    data.qualified_items = 1_200_000;
    data.allocations[0].qualified_items = 1_200_000;
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("business_{farm}_qbi")), 38_467);
    assert_eq!(value(&r, "qbi_deduction"), 6_073.min(twenty(38_467)));
}

// QBI-D5: Schedule C unadjusted basis counts only property whose depreciable period has not ended
// before the close of 2026 (2025 i8995a, Determining your UBIA). A $400,000-profit business with
// a fully depreciated 5-year asset of $1,000,000 placed in service June 1, 2016 (period ended
// June 1, 2026) has no UBIA: above the phase-in the component is 0 and the deduction is the $400
// minimum for $1,000 of active QBI. Placed December 31, 2016 (ends at the close of 2026) or in
// 2017 it counts: 2.5% x 1,000,000 = 25,000.
#[test]
fn qbi_d5_a_schedule_c_asset_past_its_depreciable_period_has_no_ubia() {
    let run = |placed: u32| {
        let mut d = empty_return(FederalStatus::Single);
        let FederalIncome::Business { data: mut b } = cash_business(400_000) else { panic!() };
        b.assets.push(old_asset(placed, EquipmentClass::Five));
        add(&mut d, income(FederalIncome::Business { data: b }));
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{:?}", r.issues);
        value(&r, "qbi_deduction")
    };
    assert_eq!(run(20150115), 400);
    assert_eq!(run(20160601), 400);
    assert_eq!(run(20161231), 25_000);
    assert_eq!(run(20170601), 25_000);
}

// QBI-D5/D6: the period test at its edges, for every recovery class.
#[test]
fn qbi_d5_the_depreciable_period_ends_on_the_later_of_ten_years_and_the_recovery_period() {
    let counts = |placed: u32, class: EquipmentClass| {
        let mut asset = old_asset(placed, class);
        asset.disposal = crate::AssetDisposal::No;
        asset.qbi_property()
    };
    assert!(!counts(20160601, EquipmentClass::Seven));
    assert!(counts(20161231, EquipmentClass::Seven));
    // 15-year property placed in 2012: the last full year of the recovery period is 2026.
    assert!(counts(20120601, EquipmentClass::Fifteen));
    assert!(!counts(20100601, EquipmentClass::Fifteen));
    assert!(counts(20000101, EquipmentClass::ThirtyNine));
    let mut disposed = old_asset(20240101, EquipmentClass::Five);
    disposed.disposal =
        crate::AssetDisposal::Disposed { date: 20260301, how: crate::DisposalKind::Scrapped };
    assert!(!disposed.qbi_property());
}

// ---- Schedule F ---------------------------------------------------------------

// SF-D1: a standard-mileage farm vehicle's business-use personal property tax is Schedule F line
// 29, not line 10 (2026 draft i1040sf line 10). 5,000 + 5,000 business miles of 20,000, property
// tax $400: line 10 = 5,000 x 72.5 cents + 5,000 x 76 cents = 7,425; line 29 = 400 x 10,000 /
// 20,000 = 200; line 33 = 7,625.
#[test]
fn sf_d1_standard_mileage_property_tax_is_schedule_f_line_29() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(40000, 0)));
    business_mut(&mut d).expenses.clear();
    business_mut(&mut d).vehicles.push(BusinessMileage {
        id: 0,
        property_tax: 40_000,
        ..native_business_mileage(5_000, 5_000, 20_000)
    });
    let r = supported(&d);
    let id = business_id(&d);
    assert_eq!(value(&r, &alloc::format!("business_{id}_car_and_truck")), 7_425);
    assert_eq!(
        printed_line(&r, &alloc::format!("business_{id}_car_and_truck")),
        (TaxForm::ScheduleF, "10".into())
    );
    assert_eq!(value(&r, &alloc::format!("farm_{id}_expense_line29_taxes")), 200);
    assert_eq!(
        printed_line(&r, &alloc::format!("farm_{id}_expense_line29_taxes")),
        (TaxForm::ScheduleF, "29".into())
    );
    assert_eq!(value(&r, &alloc::format!("farm_{id}_expenses")), 7_625);
    assert_schedule_f_part_ii_foots(&r, id);
}

// The emitted Schedule F lines 10 through 32f (the Part II rows) add up to line 33.
fn assert_schedule_f_part_ii_foots(r: &FederalResult, id: i64) {
    let prefix = alloc::format!("farm_{id}_expense_");
    let mut printed: i64 =
        r.lines.iter().filter(|l| l.id.starts_with(&prefix)).map(|l| l.amount).sum();
    for key in [
        alloc::format!("business_{id}_car_and_truck"),
        alloc::format!("business_{id}_vehicle_loan_interest"),
        alloc::format!("farm_{id}_depreciation"),
        alloc::format!("farm_{id}_conservation_deduction"),
        alloc::format!("business_{id}_home_office"),
    ] {
        if has(r, &key) {
            printed += cents(r, &key);
        }
    }
    printed += r
        .lines
        .iter()
        .filter(|l| l.id.starts_with(&alloc::format!("farm_{id}_method_change_")))
        .filter(|l| !l.id.ends_with("_remaining"))
        .map(|l| l.amount.min(0).abs())
        .sum::<i64>();
    assert_eq!(
        printed,
        cents(r, &alloc::format!("farm_{id}_expenses")),
        "Schedule F lines 10-32f vs line 33"
    );
}

// SF-D2: start-up costs, amortization, the prepaid-supply deduction and a CCC paid deficiency are
// Part II amounts that foot to line 33 (2026 draft i1040sf lines 32a-32f and 33). A farm begun
// January 5, 2026 with $20,000 of start-up costs: initial deduction min(20,000, 5,000 - max(0,
// 20,000 - 50,000)) = 5,000; amortization 15,000 x 12 / 180 = 1,000; a start-up row of 6,000 and
// lines 10-32f = 7,000 = line 33 with the 1,000 Other expense.
#[test]
fn sf_d2_startup_costs_are_part_ii_rows_that_foot_to_line_33() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(40000, 1000)));
    business_mut(&mut d).startup = BusinessStartup::Costs {
        costs: 2_000_000,
        started: 20260105,
        capitalize: false,
        eligible: true,
    };
    let r = supported(&d);
    let id = business_id(&d);
    assert_eq!(value(&r, &alloc::format!("farm_{id}_expense_line32_startup_amortization")), 6_000);
    assert_eq!(value(&r, &alloc::format!("farm_{id}_expense_line32_other")), 1_000);
    assert_eq!(value(&r, &alloc::format!("farm_{id}_expenses")), 7_000);
    assert_eq!(value(&r, &alloc::format!("farm_{id}_net_profit")), 33_000);
    assert_schedule_f_part_ii_foots(&r, id);
}

// SF-D2: the prepaid-supply deduction ($8,000 of an advance against $10,000 of other expenses
// and $15,999 of receipts is limited to half of the other expenses) and the CCC paid deficiency
// join the same Part II rows.
#[test]
fn sf_d2_prepaid_supplies_and_the_ccc_deficiency_foot_to_line_33() {
    let d = farm_supply_return(farm_supplies(3000), 10000, 40000);
    let r = supported(&d);
    let id = business_id(&d);
    assert_eq!(value(&r, &alloc::format!("farm_{id}_expense_line32_prepaid_supplies")), 3_000);
    assert_eq!(value(&r, &alloc::format!("farm_{id}_expenses")), 13_000);
    assert_schedule_f_part_ii_foots(&r, id);
    // A section 77 forfeiture with a $1,000 required deficiency paid.
    let d = ccc_return(
        CccOrigin::Current,
        CccMethod::Income,
        CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 100_000 },
        40000,
    );
    let r = supported(&d);
    let id = business_id(&d);
    assert_eq!(value(&r, &alloc::format!("farm_{id}_expense_line32_ccc_deficiency")), 1_000);
    assert_eq!(value(&r, &alloc::format!("farm_{id}_expenses")), 11_000);
    assert_schedule_f_part_ii_foots(&r, id);
}

// The Schedule F footing: every Part II amount at once (expense categories, mileage with
// property tax and loan interest, start-up costs, prepaid supplies, a CCC deficiency).
#[test]
fn sf_part_ii_rows_add_to_line_33() {
    let mut d = ccc_return(
        CccOrigin::Current,
        CccMethod::Income,
        CccDisposition::Forfeited { year: 2026, surplus: 0, deficiency: 100_000 },
        60000,
    );
    let id = business_id(&d);
    let b = business_mut(&mut d);
    b.startup = BusinessStartup::Costs {
        costs: 2_000_000,
        started: 20260105,
        capitalize: false,
        eligible: true,
    };
    b.vehicles.push(BusinessMileage {
        id: 0,
        parking_tolls: 10_000,
        interest: 30_000,
        property_tax: 40_000,
        ..native_business_mileage(5_000, 5_000, 20_000)
    });
    b.expenses.push(ordinary_business_expense(BusinessExpenseCategory::TaxesAndLicenses, 50_000));
    farm_of(b).supplies = Some(farm_supplies(3000));
    let r = supported(&d);
    assert_schedule_f_part_ii_foots(&r, id);
    assert_eq!(value(&r, &alloc::format!("farm_{id}_expense_line29_taxes")), 700);
}

// SF-D3: an accrual farm's crop insurance is Part III line 41; the form has no 41a, 41b or 41d
// (2026 draft and 2025 final Schedule F). Sales 50,000, expenses 10,000, one nondeferrable
// payment of 8,000: line 41 = 8,000, line 44 = 58,000 = line 9.
#[test]
fn sf_d3_accrual_crop_insurance_is_line_41() {
    let mut d = empty_return(FederalStatus::Single);
    let mut e = accrual_farm(50000, 0, 0, 0, 10000);
    let FederalIncome::Business { data } = &mut e else { unreachable!() };
    farm_of(data).insurance = FarmInsurance::Payments {
        current: vec![CropPayment::Other { amount: 800_000 }],
        previous_deferred: 0,
        election: CropDeferral::Include,
        withholding: 0,
    };
    add(&mut d, income(e));
    let r = supported(&d);
    let id = business_id(&d);
    assert_eq!(
        printed_line(&r, &alloc::format!("farm_{id}_crop_insurance_current_taxable")),
        (TaxForm::ScheduleF, "41".into())
    );
    assert_eq!(value(&r, &alloc::format!("farm_{id}_crop_insurance_current_taxable")), 8_000);
    for line in ["41a", "41b", "41d"] {
        assert!(
            !r.lines.iter().any(|l| l
                .form_line
                .as_ref()
                .is_some_and(|f| f.form == TaxForm::ScheduleF && f.line == line)),
            "no line {line}"
        );
    }
    assert_eq!(value(&r, &alloc::format!("farm_{id}_gross_income")), 58_000);
}

// SF-D4: Part III line 37 holds the sale of a CCC-pledged crop (2026 draft Schedule F line 37;
// line 44 adds 37 through 43). Accrual farm, sales 100,000, expenses 10,000, one current loan-
// method CCC lot of principal 20,000 redeemed for 18,000 and sold for 25,000: line 37 = 125,000;
// market gain 20,000 - 18,000 = 2,000 on lines 39a and 39b; line 44 = line 9 = 127,000; line 34 =
// 117,000.
#[test]
fn sf_d4_line_37_includes_a_ccc_pledged_crop_sale() {
    let mut d = empty_return(FederalStatus::Single);
    let mut e = accrual_farm(100000, 0, 0, 0, 10000);
    let FederalIncome::Business { data } = &mut e else { unreachable!() };
    farm_of(data).ccc = Some(FarmCcc {
        method: CccMethod::Loan,
        lots: vec![CccLot {
            id: 0,
            origin: CccOrigin::Current,
            principal: 2_000_000,
            disposition: CccDisposition::Redeemed {
                year: 2026,
                paid: 1_800_000,
                sale: Some(2_500_000),
            },
        }],
        withholding: 0,
        source: ccc_source(&CccMethod::Loan),
    });
    add(&mut d, income(e));
    let r = supported(&d);
    let id = business_id(&d);
    let line = |name: &str| value(&r, &alloc::format!("farm_{id}_{name}"));
    assert_eq!(line("accrual_sales"), 125_000);
    assert_eq!(
        printed_line(&r, &alloc::format!("farm_{id}_accrual_sales")),
        (TaxForm::ScheduleF, "37".into())
    );
    assert_eq!(line("program_payments"), 2_000);
    assert_eq!(line("program_taxable"), 2_000);
    assert_eq!(
        printed_line(&r, &alloc::format!("farm_{id}_program_payments")),
        (TaxForm::ScheduleF, "39a".into())
    );
    assert_eq!(line("gross_income"), 127_000);
    assert_eq!(line("net_profit"), 117_000);
    // Part III foots: 37 + 38b + 39b + other income = line 44 = line 9 (no inventories).
    assert_eq!(
        line("accrual_sales") + line("program_taxable") + line("other_accrual_income"),
        line("gross_income")
    );
}

// SF-D5: the section 175 limit is 25% of gross income from farming, which includes the Form 1099-G
// box 7 program payments routed to the farm (section 175(b); 2026 draft i1040sf line 12). Raised
// sales 40,000, other expense 5,000, 15,000 of terracing, a 20,000 box 7 payment: limit 25% x
// 60,000 = 15,000, deduction 15,000, no carryforward, line 33 = 20,000, line 34 = 40,000.
#[test]
fn sf_d5_the_section_175_limit_includes_the_routed_1099g_payment() {
    let mut d = empty_return(FederalStatus::Single);
    let mut e = raised_farm(40000, 5000);
    let FederalIncome::Business { data } = &mut e else { unreachable!() };
    farm_of(data).conservation = Some(FarmConservation::Deduct {
        costs: vec![ConservationCost {
            property: 0,
            paid_or_incurred: None,
            facts: conservation_facts(),
            label: "Terracing".into(),
            amount: 1_500_000,
        }],
        assessments: vec![],
        carry: vec![],
        method: conservation_method(),
    });
    add(&mut d, income(e));
    add(&mut d, ag_payment(20000));
    let r = supported(&d);
    let id = d.income[0].id;
    let line = |name: &str| value(&r, &alloc::format!("farm_{id}_{name}"));
    assert_eq!(line("conservation_income_limit"), 15_000);
    assert_eq!(line("conservation_deduction"), 15_000);
    assert_eq!(line("conservation_carryforward"), 0);
    assert_eq!(line("expenses"), 20_000);
    assert_eq!(line("net_profit"), 40_000);
    assert_schedule_f_part_ii_foots(&r, id);
}

// SF-D6: the Schedule J elected farm income reads the same farm figure as line 34, SE and QBI, so
// the routed 1099-G payment counts (2025 i1040sj line 2a). The averaging return with the farm at
// 100,000 raised and 20,000 of expenses plus a 20,000 box 7 payment (line 34 = 100,000 as in the
// original 120,000 / 20,000 farm): SE tax 14,130 (half 7,065), QBI 18,587, eligible farm income
// = 100,000 - 7,065 - 18,587 = 74,348, and an election of 60,000 is accepted.
#[test]
fn sf_d6_schedule_j_farm_income_includes_the_routed_1099g_payment() {
    let mut d = averaging_return(60000);
    d.income[1].income = raised_farm(100000, 20000);
    add(&mut d, ag_payment(20000));
    let r = estimate(&d);
    assert_eq!(value(&r, "self_employment_tax"), 14_130);
    assert_eq!(value(&r, "farm_averaging_qbi_deduction"), 18_587);
    assert_eq!(value(&r, "farm_averaging_eligible_income"), 74_348);
    assert!(!r.issues.iter().any(|i| i.message.contains("ELECTED_AMOUNT")), "{:?}", r.issues);
}

// SF-D7: a profitable farm with routed 1099-G income is not an at-risk loss (the line 36 test
// reads the same farm figure; 2026 draft i1040sf line 36). Raised sales 10,000, expenses 15,000,
// not at risk for all, a 10,000 box 7 payment: line 9 = 20,000, line 33 = 15,000, line 34 = 5,000;
// Schedule SE line 4a 5,000 x 92.35% = 4,617.50 -> 4,618, tax 4,618 x 15.3% = 706.55 -> 707, half
// 353.50 -> 354; AGI 5,000 - 354 = 4,646; no left-out notice.
#[test]
fn sf_d7_a_farm_profitable_with_routed_income_is_not_an_at_risk_loss() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(raised_farm(10000, 15000)));
    business_mut(&mut d).at_risk = crate::RentalAtRisk::from_full_risk(false);
    add(&mut d, ag_payment(10000));
    let r = supported(&d);
    assert!(!noted(&r, "WARN_BUSINESS_LOSS_LEFT_OUT"));
    assert_eq!(value(&r, "self_employment_tax"), 707);
    assert_eq!(value(&r, "agi"), 4_646);
    // Without the payment the farm is a loss, and the at-risk limit still leaves it out.
    let mut loss = empty_return(FederalStatus::Single);
    add(&mut loss, income(raised_farm(10000, 15000)));
    business_mut(&mut loss).at_risk = crate::RentalAtRisk::from_full_risk(false);
    let r = supported(&loss);
    assert!(noted(&r, "WARN_BUSINESS_LOSS_LEFT_OUT"));
}
