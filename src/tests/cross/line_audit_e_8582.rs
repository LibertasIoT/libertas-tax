use crate::tests::support::assert::line_amount as cents;
// Line audit of Schedule E, Form 8582, Form 4835 and Pub. 527 Worksheet 5-1
// (2026-10-10, defects D1-D13; fix unit F7). Every expected amount is the audit's
// hand-worked fixture figure from the Schedule E 2026 draft (5/6/26) and its
// instructions (Sep 21, 2026), the Form 8582 2026 draft with the 2025
// instructions, the Form 4835 2026 draft and instructions, Pub. 527 and 925 (2025),
// and sections 280A and 469, never the App's own output.
use super::*;

fn has(r: &FederalResult, id: &str) -> bool {
    r.lines.iter().any(|l| l.id == id)
}
fn rental_ids(d: &FederalDraft) -> Vec<i64> {
    d.income
        .iter()
        .filter(|e| matches!(e.income, FederalIncome::Rental { .. }))
        .map(|e| e.id)
        .collect()
}
fn id_of(d: &FederalDraft, f: impl Fn(&FederalIncome) -> bool) -> i64 {
    d.income.iter().find(|e| f(&e.income)).unwrap().id
}

// The Pub. 527 cottage (Worksheet 5-1): rent `rent`; mortgage interest $3,000,
// real estate taxes $1,200 and utilities $2,400 for the whole unit; a $110,000
// depreciable basis in service since January 2020 (2026 depreciation $4,000).
fn add_home(
    d: &mut FederalDraft,
    rent: i64,
    fair: u32,
    personal: u32,
    overlap: u32,
    prior: [i64; 2],
) {
    let mut entry = home(rent, fair, personal, overlap, prior);
    if let FederalIncome::Rental { data } = &mut entry.income {
        link_home_fixture(d, data);
    }
    add(d, entry);
}
fn home(rent: i64, fair: u32, personal: u32, overlap: u32, prior: [i64; 2]) -> FederalIncomeEntry {
    let expense = |category, amount: i64| crate::extended_model::RentalExpense {
        treatment: BusinessExpenseKind::Ordinary,
        category,
        amount: amount * 100,
    };
    let mut v = rental(rent, 0);
    v.expenses = vec![
        expense(crate::extended_model::RentalExpenseCategory::MortgageInterest, 3000),
        expense(crate::extended_model::RentalExpenseCategory::TaxesAndLicenses, 1200),
        expense(crate::extended_model::RentalExpenseCategory::Utilities, 2400),
    ];
    v.buildings[0].cost = 130_000 * 100;
    v.buildings[0].land = 20_000 * 100;
    v.personal_use = crate::extended_model::RentalPersonalUse::Yes {
        fair_rental_days: fair,
        personal_days: personal,
        overlap_days: overlap,
        direct_expenses: 0,
    };
    v.home_operating_carryover = prior[0] * 100;
    //2026-10-19: both synthetic tax worksheets explicitly carry depreciation only.
    v.prior_home_carry.get_or_insert(crate::RentalHomeCarries::NONE).home_depreciation_carryover =
        crate::SourceCents::Reported { amount: prior[1] * 100 };
    v.prior_home_carry
        .get_or_insert(crate::RentalHomeCarries::NONE)
        .amt_home_depreciation_carryover = crate::SourceCents::Reported { amount: prior[1] * 100 };
    income(FederalIncome::Rental { data: v })
}

// A rental with `income` of profit before any prior-year loss, in whole dollars
// (the helper's building adds $10,000 of depreciation): rent less $10,000.
fn profit_rental(profit: i64, active: bool, prior: i64) -> FederalIncomeEntry {
    let mut v = rental(profit + 10_000, 0);
    v.participation = if active {
        crate::RentalParticipation::Active
    } else {
        crate::RentalParticipation::NotActive
    };
    v.history = RentalHistory { regular: prior * 100, amt: prior * 100, active_in_loss_year: true };
    income(FederalIncome::Rental { data: v })
}

// A Form 4835 farm rental with active participation: line 1 $8,000 and $28,000
// of repairs, line 7 $8,000 and line 31 $28,000.
fn farm_loss() -> FederalIncome {
    FederalIncome::FarmRental {
        data: FederalFarmRental {
            improvement_buildings: Vec::new(),
            interest_limit_election: None,
            farm: FarmIncome {
                related: vec![],
                specials: farm_specials(),
                land_basis: vec![],
                header: None,
                accounting: FarmAccounting::Cash {
                    receipts: vec![FarmReceipt::Raised { amount: 8_000 * 100 }],
                },
                insurance_papers: Default::default(),
                insurance: FarmInsuranceRecords::None,
                ccc: None,
                crp: vec![],
                supplies: None,
                conservation: None,
                land_sales: vec![],
            },
            expenses: vec![ordinary_business_expense(
                BusinessExpenseCategory::Repairs,
                28_000 * 100,
            )],
            vehicles: vec![],
            assets: vec![],
            active_participation: true,
            at_risk: Some(crate::RentalAtRisk::All),
            history: RentalHistory::NONE,
            qbi_business: RentalBusiness::No,
            qbi_aggregation_group: None,
            payroll: BusinessPayroll::No,
        },
    }
}

fn professional_return() -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 160_000 * 100, 0);
    // Rent $10,000 less $30,000 of expenses and $10,000 of depreciation.
    let mut data = rental(10_000, 30_000);
    data.participation = crate::RentalParticipation::Professional;
    add(&mut d, income(FederalIncome::Rental { data }));
    make_professional(&mut d, Owner::Taxpayer, false);
    d
}

// The identities Schedule E lines 24 through 26 and Form 8582 Parts I through III
// follow, over the rows a result holds.
fn assert_foots(r: &FederalResult) {
    let form8582 = |line: &str| -> Option<i64> {
        r.lines
            .iter()
            .find(|l| {
                l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8582 && f.line == line)
            })
            .map(|l| l.amount / 100)
    };
    let sum_e = |line: &str, positive_only: bool| -> i64 {
        r.lines
            .iter()
            .filter(|l| {
                l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::ScheduleE && f.line == line)
            })
            .map(|l| if positive_only { l.amount.max(0) } else { l.amount })
            .sum()
    };
    // Line 26 = line 24 (the positive line 21 amounts) + line 25 (the line 22
    // losses), over the properties of Part I.
    if has(r, "schedule_e_rental_royalty_total") {
        assert_eq!(
            cents(r, "schedule_e_rental_royalty_total"),
            sum_e("21", true) + sum_e("22", false),
            "line 26 = line 24 + line 25"
        );
    }
    // Part I: line 3 = 1d + 2d.
    if let Some(line3) = form8582("3") {
        assert_eq!(
            line3,
            form8582("1d").unwrap_or(0) + form8582("2d").unwrap(),
            "line 3 = 1d + 2d"
        );
    }
    // Part II: line 4 the smaller loss, line 7 = line 5 - line 6, line 8 half of
    // line 7 (a half dollar up) at most $25,000 ($12,500), line 9 the smaller of
    // lines 4 and 8.
    if let Some(line4) = form8582("4") {
        assert_eq!(line4, (-form8582("1d").unwrap()).min(-form8582("3").unwrap()));
        let (line5, line6, line7, line8, line9) = (
            form8582("5").unwrap(),
            form8582("6").unwrap(),
            form8582("7").unwrap(),
            form8582("8").unwrap(),
            form8582("9").unwrap(),
        );
        assert!(line6 >= 0, "line 6 is not below zero");
        assert_eq!(line7, (line5 - line6).max(0));
        let cap = if line5 == 75_000 { 12_500 } else { 25_000 };
        assert_eq!(line8, cap.min((line7 + 1) / 2));
        assert_eq!(line9, line4.min(line8));
    }
    // Part III: line 11 = line 9 + line 10.
    if let Some(line10) = form8582("10") {
        assert_eq!(
            form8582("11").unwrap(),
            line10 + form8582("9").unwrap_or(0),
            "line 11 = line 9 + line 10"
        );
    }
}

// D1: the Pub. 527 cottage. 85 days were rented at a fair price, 2 of them also
// used by the owner; 14 other days were personal. Section 280A(d)(1): 14 + 2 = 16
// personal days exceed the greater of 14 and 10% of the 83 fair-rental days not
// used personally (8.3), so the unit is a home. Section 280A(e) and Dividing
// Expenses: the overlap days are rental days, 85 of 99 days of use.
//   Worksheet 5-1: line 2 = 4,200 x 85/99 = 3,606.06; line 3 = 2,393.94;
//   line 4f = 2,400 x 85/99 = 2,060.61; line 5 = 333.33; line 6b = 4,000 x
//   85/99 = 3,434.34; line 6e = 333.33; line 7b = 3,101.01.
#[test]
fn d1_a_day_rented_and_used_is_a_rental_day_for_expenses_and_a_personal_day_for_the_home_test() {
    let mut d = complete(FederalStatus::Single);
    add_home(&mut d, 6_000, 85, 14, 2, [0, 0]);
    let r = supported(&d);
    let id = rental_ids(&d)[0];
    // A home: the Worksheet 5-1 carryovers are reported.
    assert_eq!(cents(&r, &alloc::format!("rental_{id}_home_operating_carryover")), 0);
    assert_eq!(cents(&r, &alloc::format!("rental_{id}_home_depreciation_carryover")), 310_101);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line18_depreciation")), 333);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line21_income")), 0);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_foots(&r);
    // The same days with no overlap entered are 14 personal days: not a home, and
    // the expenses are divided the same 85/99.
    let mut d = complete(FederalStatus::Single);
    add_home(&mut d, 6_000, 85, 14, 0, [0, 0]);
    let r = supported(&d);
    let id = rental_ids(&d)[0];
    assert!(!has(&r, &alloc::format!("rental_{id}_home_operating_carryover")));
    // 6,000 - 10,600 x 85/99 (9,101.01) = -3,101.01, allowed by the passive allowance.
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line21_income")), -3_101);
    // The overlap cannot exceed the days rented.
    let mut d = complete(FederalStatus::Single);
    add_home(&mut d, 6_000, 10, 0, 11, [0, 0]);
    let r = estimate(&d);
    assert!(
        r.issues.iter().any(|i| i.display.key.starts_with("ISSUE_ACCEPTED_DATA_IS_INVALID")),
        "{:?}",
        r.issues
    );
}

// D2: a prior-year unallowed loss that the property's own income absorbs is part of
// the Schedule E line 22 loss (2025 i8582, How To Report Allowed Losses, Schedule E
// Part I). D8: Form 8582 lines 10 and 11 use each activity's income before its
// prior-year loss (Part IV column (a)).
#[test]
fn d2_line_22_includes_the_prior_loss_the_propertys_own_income_used() {
    // Active participation: line 21 +5,000, prior loss 8,000. Part IV (a) 5,000,
    // (c) 8,000; lines 1d and 3 (3,000); line 9 3,000; Part VIII (a) 8,000,
    // (b) 0, c 8,000. Line 22 (8,000); line 26 = 5,000 - 8,000 = (3,000).
    let mut d = complete(FederalStatus::Single);
    add(&mut d, profit_rental(5_000, true, 8_000));
    let r = supported(&d);
    let id = rental_ids(&d)[0];
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line21_income")), 5_000);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line22_allowed_loss")), -8_000);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), -3_000);
    assert_eq!(value(&r, "form8582_line_3"), -3_000);
    assert_eq!(value(&r, "form8582_line_9"), 3_000);
    // D8: line 10 = 5,000, line 11 = 5,000 + 3,000.
    assert_eq!(value(&r, "form8582_line_10"), 5_000);
    assert_eq!(value(&r, "form8582_line_11"), 8_000);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_suspended_loss")), 0);
    assert_eq!(value(&r, "agi"), 57_000);
    assert_foots(&r);

    // No active participation: Part VII unallowed 3,000, Part VIII (c) 5,000.
    // Line 22 (5,000), line 26 = 0; D8: lines 10 and 11 are 5,000 and 5,000.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, profit_rental(5_000, false, 8_000));
    let r = supported(&d);
    let id = rental_ids(&d)[0];
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line22_allowed_loss")), -5_000);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 0);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_suspended_loss")), 3_000);
    assert_eq!(value(&r, "form8582_line_10"), 5_000);
    assert_eq!(value(&r, "form8582_line_11"), 5_000);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_foots(&r);

    // Prior loss 3,000 against 5,000 of income: line 3 is +2,000, every loss is
    // allowed in full, and line 22 still shows the 3,000.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, profit_rental(5_000, true, 3_000));
    let r = supported(&d);
    let id = rental_ids(&d)[0];
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line22_allowed_loss")), -3_000);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 2_000);
    assert_eq!(value(&r, "agi"), 62_000);
    assert_foots(&r);
}

// D3: a qualified real estate professional's rental loss is not from a passive
// activity: "Enter the (loss) from line 21 on line 22" (2026 i1040se, Line 22).
#[test]
fn d3_a_real_estate_professionals_loss_is_on_line_22() {
    let d = professional_return();
    let r = supported(&d);
    let id = rental_ids(&d)[0];
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line21_income")), -30_000);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line22_allowed_loss")), -30_000);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), -30_000);
    assert_eq!(value(&r, "agi"), 130_000);
    assert_foots(&r);
}

// D4: section 469(i)(3)(E)(iv) leaves "any loss allowable by reason of subsection
// (c)(7)" out of modified adjusted gross income (2025 i8582 line 6), so the
// professional's $30,000 rental loss does not lower line 6. W-2 $160,000 plus a
// Form 4835 rental with active participation and a $20,000 loss: line 6 is
// 160,000, line 7 is 0, so lines 8 and 9 and Form 4835 line 34c are 0 and the
// whole $20,000 is suspended. AGI = 160,000 - 30,000 = 130,000.
// D12: Form 4835 line 32 is blank for that loss (2026 i4835).
#[test]
fn d4_d12_the_professionals_loss_stays_out_of_magi_and_form_4835_line_32_is_blank() {
    let mut d = professional_return();
    add(&mut d, income(farm_loss()));
    let r = supported(&d);
    let farm = id_of(&d, |i| matches!(i, FederalIncome::FarmRental { .. }));
    assert_eq!(value(&r, "form8582_line_6"), 160_000);
    assert_eq!(value(&r, "form8582_line_7"), 0);
    assert_eq!(value(&r, "form8582_line_8"), 0);
    assert_eq!(value(&r, "form8582_line_9"), 0);
    assert_eq!(value(&r, &alloc::format!("farm_rental_{farm}_net_income")), 0);
    assert_eq!(value(&r, &alloc::format!("farm_rental_{farm}_line34c_allowed_loss")), 0);
    assert_eq!(value(&r, &alloc::format!("farm_rental_{farm}_suspended_loss")), 20_000);
    assert_eq!(value(&r, "schedule_e_farm_rental_total"), 0);
    assert_eq!(value(&r, "agi"), 130_000);
    assert_foots(&r);
}

// D5: section 280A(c)(5), last sentence: a carryover stays limited "whether or not
// the dwelling unit is used as a residence during such taxable year". 200 fair
// rental days and 10 personal days (not a home: 10 is not above 14), rent 12,000,
// a 2025 line 7a carryover of 5,000. This year's rental share (x 200/210, each
// amount rounded to the cent): interest and taxes 4,000.00, operating 2,285.71,
// depreciation 3,809.52, together 10,095.23. The carryover may use only the
// 12,000 - 10,095.23 = 1,904.77 left; line 21 is 0 and 3,095.23 carries to 2027.
#[test]
fn d5_a_carryover_is_limited_in_a_year_the_unit_is_not_a_home() {
    let mut d = complete(FederalStatus::Single);
    add_home(&mut d, 12_000, 200, 10, 0, [5_000, 0]);
    let r = supported(&d);
    let id = rental_ids(&d)[0];
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line21_income")), 0);
    assert_eq!(cents(&r, &alloc::format!("rental_{id}_home_operating_carryover")), 309_523);
    assert_eq!(cents(&r, &alloc::format!("rental_{id}_home_depreciation_carryover")), 0);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_foots(&r);
    // Room for all of it: rent 20,000 leaves 9,904.77, so the whole 5,000 is used.
    let mut d = complete(FederalStatus::Single);
    add_home(&mut d, 20_000, 200, 10, 0, [5_000, 0]);
    let r = supported(&d);
    let id = rental_ids(&d)[0];
    assert_eq!(value(&r, "agi"), 60_000 + 4_905);
    assert_eq!(cents(&r, &alloc::format!("rental_{id}_home_operating_carryover")), 0);
}

// D6: Schedule K-1 (Form 1041) box 5 royalties are Schedule E line 33 column (f)
// (2026 draft K-1 beneficiary table), not Part I: line 26 totals lines 24 and 25.
#[test]
fn d6_estate_box_5_royalties_are_not_on_line_26() {
    let mut d = estate_trust_code_c_draft(0, 0);
    d.income[0] = wage(1, Owner::Taxpayer, 60_000 * 100, 0);
    {
        let mut data = estate_trust_code_d_data(&mut d);
        data.royalty_income = 1_000 * 100;
        data.box_5_royalty_only = true;
    }
    let r = supported(&d);
    let id = d.income[1].id;
    assert_eq!(value(&r, &alloc::format!("estate_trust_{id}_line33f_royalties")), 1_000);
    assert!(!has(&r, "schedule_e_rental_royalty_total"), "no property: no line 26");
    assert_eq!(value(&r, "schedule_1_rental_income"), 1_000);
    assert_eq!(value(&r, "agi"), 61_000);
    // A rental beside it: line 26 is the rental alone.
    add(&mut d, profit_rental(5_000, true, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), 5_000);
    assert_eq!(value(&r, "schedule_1_rental_income"), 6_000);
    assert_foots(&r);
}

// D7: Schedule E line 42 reconciles every source it names, not only a Form 4835:
// Schedule K-1 (Form 1041) box 14 code F here.
#[test]
fn d7_line_42_is_printed_without_a_form_4835() {
    let mut d = estate_trust_code_c_draft(0, 0);
    d.income[0] = wage(1, Owner::Taxpayer, 60_000 * 100, 0);
    {
        let child_id = d.next_id;
        d.next_id += 1;
        let mut data = estate_trust_code_d_data(&mut d);
        data.activities.box6 = 1_000_000;
        data.activities.activities.push(crate::EstateActivity {
            id: child_id,
            name: "Farming activity".into(),
            kind: crate::EstateActivityKind::Business,
            income: 1_000_000,
            depreciation: 0,
            depletion: 0,
            amortization: 0,
            participation: crate::EstateParticipation::Passive,
            carried: crate::EstatePassiveCarry { regular: 0, alternative: 0 },
            amt_difference: 0,
        });
        data.gross_farming_fishing_income = 10_000 * 100;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "schedule_e_farming_fishing_gross"), 10_000);
    assert_eq!(
        printed_line(&r, "schedule_e_farming_fishing_gross"),
        (TaxForm::ScheduleE, "42".into())
    );
    // No Form 4835: no line 40.
    assert!(!has(&r, "schedule_e_farm_rental_total"));
}

// D7, partnership: Schedule K-1 (Form 1065) box 14 code B.
#[test]
fn d7_partnership_box_14_code_b_feeds_line_42() {
    let mut data = ordinary_partner_source(10_000, true);
    data.se_gross = Some(crate::PartnerSeGross::Farm { gross: 10_000 * 100 });
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data }));
    let r = estimate(&d);
    assert_eq!(value(&r, "schedule_e_farming_fishing_gross"), 10_000);
}

// D9: married filing separately and lived with the spouse at any time: "do not
// complete Part II. Instead, go to line 10", and Part V replaces Part IV (2025
// i8582). Line 1d is blank, line 2d (10,000), line 3 (10,000), no lines 4-9,
// lines 10 and 11 are 0 and the whole loss is suspended.
#[test]
fn d9_mfs_living_together_uses_part_v_and_skips_part_ii() {
    let mut d = complete(FederalStatus::Separate);
    add(&mut d, income(FederalIncome::Rental { data: rental(10_000, 10_000) }));
    let r = supported(&d);
    let id = rental_ids(&d)[0];
    assert!(!has(&r, "form8582_line_1d"), "line 1d is blank");
    assert_eq!(value(&r, "form8582_line_2d"), -10_000);
    assert_eq!(value(&r, "form8582_line_3"), -10_000);
    for n in 4..=9 {
        assert!(!has(&r, &alloc::format!("form8582_line_{n}")), "line {n}");
    }
    assert_eq!(value(&r, "form8582_line_10"), 0);
    assert_eq!(value(&r, "form8582_line_11"), 0);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_suspended_loss")), 10_000);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line22_allowed_loss")), 0);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_foots(&r);
}

// D10: line 6 is "modified adjusted gross income, but not less than zero". W-2
// $10,000, a Schedule C loss of $30,000 and a rental loss of $10,000 make MAGI
// -20,000: line 6 is 0, line 7 is 150,000, line 8 is 25,000 and line 9 is 10,000.
#[test]
fn d10_line_6_is_never_negative() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 10_000 * 100, 0));
    add(&mut d, income(cash_business(-30_000)));
    add(&mut d, income(FederalIncome::Rental { data: rental(10_000, 10_000) }));
    let r = estimate(&d);
    assert_eq!(value(&r, "form8582_line_6"), 0);
    assert_eq!(value(&r, "form8582_line_7"), 150_000);
    assert_eq!(value(&r, "form8582_line_8"), 25_000);
    assert_eq!(value(&r, "form8582_line_9"), 10_000);
    assert_foots(&r);
}

// D11: line 8 is half of line 7, and a half dollar rounds up. W-2 $100,001 and a
// $30,000 rental loss: line 7 = 150,000 - 100,001 = 49,999; line 8 = 24,999.50,
// 25,000; line 9 = 25,000; line 22 (25,000); 5,000 suspended; AGI =
// 100,001 - 25,000 = 75,001.
#[test]
fn d11_line_8_rounds_the_half_dollar_up() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 100_001 * 100, 0);
    add(&mut d, income(FederalIncome::Rental { data: rental(10_000, 30_000) }));
    let r = supported(&d);
    let id = rental_ids(&d)[0];
    assert_eq!(value(&r, "form8582_line_7"), 49_999);
    assert_eq!(value(&r, "form8582_line_8"), 25_000);
    assert_eq!(value(&r, "form8582_line_9"), 25_000);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_line22_allowed_loss")), -25_000);
    assert_eq!(value(&r, &alloc::format!("rental_{id}_suspended_loss")), 5_000);
    assert_eq!(value(&r, "agi"), 75_001);
    assert_foots(&r);
}

// D13: section 469(k): a publicly traded partnership's passive items are limited
// by themselves and are not on Form 8582 (2025 i8582, Special Instructions for
// PTPs). A $5,000 PTP loss (no income from that PTP) is suspended and cannot use
// the $5,000 of passive rental income; Form 8582 lines 2d and 3 show only the
// rental, +5,000. AGI = 60,000 + 5,000. The same K-1 not marked publicly traded
// uses the rental income.
#[test]
fn d13_a_ptp_loss_is_outside_the_common_pool() {
    for (ptp, agi) in [(true, 65_000), (false, 60_000)] {
        let mut data = ordinary_partner_source(-5_000, false);
        data.regular.basis.opening = Some(5_000_000);
        data.regular.risk.start =
            ScorpRiskStart::Detailed { capacity: 5_000_000, profit: 0, loss: 0 };
        data.publicly_traded = ptp;
        data.alternative = Some(data.regular.clone());
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::Partnership { data }));
        add(&mut d, profit_rental(5_000, false, 0));
        let r = estimate(&d);
        let id = id_of(&d, |i| matches!(i, FederalIncome::Partnership { .. }));
        assert_eq!(value(&r, "agi"), agi, "ptp {ptp}");
        if ptp {
            assert_eq!(value(&r, &alloc::format!("partnership_{id}_passive_suspended")), 5_000);
            assert_eq!(value(&r, &alloc::format!("partnership_{id}_activity_net")), 0);
            assert_eq!(value(&r, "form8582_line_2d"), 5_000);
            assert_eq!(value(&r, "form8582_line_3"), 5_000);
        } else {
            assert_eq!(value(&r, &alloc::format!("partnership_{id}_passive_suspended")), 0);
            assert_eq!(value(&r, &alloc::format!("partnership_{id}_activity_net")), -5_000);
            assert_eq!(value(&r, "form8582_line_3"), 0);
        }
    }
    // A PTP with net income: nonpassive income of the return, not passive income
    // that frees other passive losses.
    let mut data = ordinary_partner_source(5_000, false);
    data.publicly_traded = true;
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data }));
    let mut loss = rental(10_000, 15_000);
    loss.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: loss }));
    let r = estimate(&d);
    let rental_id = rental_ids(&d)[0];
    // The rental loses 15,000; the PTP's 5,000 does not free any of it.
    assert_eq!(value(&r, &alloc::format!("rental_{rental_id}_suspended_loss")), 15_000);
    assert_eq!(value(&r, "agi"), 65_000);
}

// The footing identities hold over a return with three properties: A profit 5,000
// with an 8,000 prior loss, B a loss of 10,000, C profit 2,000.
//   Part IV: (a) 5,000 + 0 + 2,000; losses (b) 10,000; prior (c) 8,000.
//   Line 3 = 1d = (3,000) + (10,000) + 2,000 = (11,000); line 4 = 11,000;
//   MAGI 60,000, so line 7 = 90,000, line 8 = 25,000, line 9 = 11,000;
//   line 10 = 5,000 + 2,000 = 7,000; line 11 = 18,000, the 8,000 and 10,000 of
//   line 22. Line 24 = 5,000 + 2,000, line 25 = (18,000), line 26 = (11,000).
#[test]
fn schedule_e_lines_24_to_26_and_form_8582_parts_i_to_iii_foot() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, profit_rental(5_000, true, 8_000));
    add(&mut d, income(FederalIncome::Rental { data: rental(10_000, 10_000) }));
    add(&mut d, profit_rental(2_000, true, 0));
    let r = supported(&d);
    let ids = rental_ids(&d);
    assert_eq!(value(&r, "form8582_line_3"), -11_000);
    assert_eq!(value(&r, "form8582_line_4"), 11_000);
    assert_eq!(value(&r, "form8582_line_6"), 60_000);
    assert_eq!(value(&r, "form8582_line_7"), 90_000);
    assert_eq!(value(&r, "form8582_line_8"), 25_000);
    assert_eq!(value(&r, "form8582_line_9"), 11_000);
    assert_eq!(value(&r, "form8582_line_10"), 7_000);
    assert_eq!(value(&r, "form8582_line_11"), 18_000);
    assert_eq!(value(&r, &alloc::format!("rental_{}_line22_allowed_loss", ids[0])), -8_000);
    assert_eq!(value(&r, &alloc::format!("rental_{}_line22_allowed_loss", ids[1])), -10_000);
    assert!(!has(&r, &alloc::format!("rental_{}_line22_allowed_loss", ids[2])));
    assert_eq!(value(&r, "schedule_e_rental_royalty_total"), -11_000);
    assert_eq!(value(&r, "agi"), 49_000);
    assert_foots(&r);
}
