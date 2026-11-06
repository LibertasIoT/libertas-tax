// Other income (unit b9-2, 2026-10-10): the Other income kind and its members, the
// recovery worksheet (Pub. 525 Worksheets 2 and 2a), Schedule 1 lines 8g, 8m with
// 24c, 8t, 8u and 8v, and the W-2 parts that move to 8t and 8u. Every amount is
// worked by hand from the 2026 drafts of Schedule 1 and the Form 1040
// instructions, Pub. 525 and the 2026 amounts the App uses ($16,100 single and
// $32,200 joint standard deduction; single 10% to $12,400 and 12% to $50,400;
// joint 10% to $24,800 and 12% to $100,800; Tax Table rows $50 wide taxed at
// their midpoint). `complete(Single)` holds one W-2 of $60,000 (taxable 43,900,
// tax 5,023) and `complete(Joint)` one of $100,000 (taxable 67,800, tax 7,643).
use super::*;

fn item(kind: OtherIncomeKind, amount: i64) -> FederalIncome {
    FederalIncome::OtherIncome {
        data: OtherIncomeItem { kind, amount: amount * 100, withholding: 0 },
    }
}
fn recovery_kind(
    year: i32,
    deducted: i64,
    itemized: i64,
    standard: i64,
    taxable_income: i64,
    ordinary: bool,
) -> OtherIncomeKind {
    OtherIncomeKind::Recovery {
        worksheet: PriorYearRecovery {
            year,
            deducted: deducted * 100,
            itemized: itemized * 100,
            standard: standard * 100,
            taxable_income: taxable_income * 100,
            ordinary,
        },
    }
}
fn tax(r: &FederalResult) -> i64 {
    r.tax.unwrap() / 100
}
fn with_item(status: FederalStatus, kind: OtherIncomeKind, amount: i64) -> FederalDraft {
    let mut d = complete(status);
    add(&mut d, income(item(kind, amount)));
    d
}

// ---- D: recovery of a prior-year deduction (Worksheet 2) -------------------

#[test]
fn d_recovery_is_limited_by_the_deduction_and_by_itemizing_above_the_standard_deduction() {
    // Joint. Insurance reimbursed 1,500 of 2025 medical costs; the deduction was 900
    // (above the 7.5% floor), itemized 33,000, standard 31,500, taxable income 80,000.
    // Worksheet 2: line 2 = min(1,500, 900) = 900; 3 = 900; 6 = 33,000; 8 = 1,500;
    // 9 = min(900, 1,500) = 900; 10 = 80,000; 11 = 900 -> Schedule 1 line 8z.
    // AGI 100,900; taxable 68,700; midpoint 68,725: 2,480 + 12% x 43,925 = 7,751.
    let d = with_item(
        FederalStatus::Joint,
        recovery_kind(2025, 900, 33_000, 31_500, 80_000, true),
        1_500,
    );
    let r = supported(&d);
    assert_eq!(value(&r, "recovery_2025_taxable"), 900);
    assert_eq!(value(&r, "agi"), 100_900);
    assert_eq!(value(&r, "taxable_income"), 68_700);
    assert_eq!(tax(&r), 7_751);
    assert_eq!(value(&r, "schedule_1_other_income"), 900);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn d_recovery_variations_follow_worksheet_2_lines_8_and_10() {
    // Itemized 31,800: line 8 = 31,800 - 31,500 = 300 -> 8z 300; taxable 68,100;
    // midpoint 68,125: 2,480 + 12% x 43,325 = 7,679.
    let r = supported(&with_item(
        FederalStatus::Joint,
        recovery_kind(2025, 900, 31_800, 31_500, 80_000, true),
        1_500,
    ));
    assert_eq!(value(&r, "recovery_2025_taxable"), 300);
    assert_eq!(value(&r, "taxable_income"), 68_100);
    assert_eq!(tax(&r), 7_679);
    // A negative taxable income of 200 reduces line 9 (300) to 100 (Example 33).
    let r = supported(&with_item(
        FederalStatus::Joint,
        recovery_kind(2025, 900, 31_800, 31_500, -200, true),
        1_500,
    ));
    assert_eq!(value(&r, "recovery_2025_taxable"), 100);
    // A standard deduction above the itemized deductions: nothing taxable (line 8 <= 0).
    let d = with_item(
        FederalStatus::Joint,
        recovery_kind(2025, 900, 31_000, 31_500, 80_000, true),
        1_500,
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 100_000);
    assert_eq!(tax(&r), 7_643);
}

#[test]
fn d_a_recovery_that_needs_the_earlier_tax_refigured_is_left_out_with_a_notice() {
    let d = with_item(
        FederalStatus::Joint,
        recovery_kind(2025, 900, 33_000, 31_500, 80_000, false),
        1_500,
    );
    let r = supported(&d);
    assert!(noted(&r, "WARN_RECOVERY_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), 100_000);
    assert_eq!(tax(&r), 7_643);
}

#[test]
fn d_the_deduction_cannot_be_more_than_the_itemized_total() {
    let item = OtherIncomeItem {
        kind: recovery_kind(2025, 34_000, 33_000, 31_500, 80_000, true),
        amount: 150_000,
        withholding: 0,
    };
    assert_eq!(
        crate::interview::document_rule(&IncomeSource::Record {
            value: crate::IncomeRecord::OtherIncome(item)
        }),
        Some("RECOVERY_DEDUCTED_ABOVE_ITEMIZED")
    );
}

#[test]
fn d_a_recovery_and_the_state_refund_of_the_same_year_share_one_worksheet() {
    // Pub. 525 Worksheet 2 lines 1a and 2 together against one line 8. Single. 2025
    // facts: state income tax paid 5,000 and deducted with no SALT cap (5d = 5e =
    // 5,000), itemized 17,000, standard 15,750, taxable income 40,000. A 1099-G
    // refund of 1,000 and a medical reimbursement of 700 (deducted 700).
    // Line 1a = the 1,000 refund (the sales-tax alternative is 0); line 2 = 700;
    // line 3 = 1,700; line 8 = 17,000 - 15,750 = 1,250; line 9 = 1,250; 11 = 1,250.
    // Lines A-C: 1,000 / 1,700 = 58.82%; line 1 = 1,250 x 1,000 / 1,700 = 735 (the
    // cents floor 735.29); line 8z = 1,250 - 735.29 = 514.71. Alone the refund would be
    // 1,000 and the recovery 700 (1,700 in all); together they share the 1,250 cap.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(refund_income(
            100_000,
            0,
            TaxRefundHistory::IncomeTax {
                deduction_year: 2025,
                income_paid: 500_000,
                sales_available: 0,
                salt_paid: 500_000,
                salt_deducted: 500_000,
                itemized: 1_700_000,
                standard: 1_575_000,
                overall_limit: None,
                property_refunded: 0,
                ordinary: true,
            },
        )),
    );
    add(&mut d, income(item(recovery_kind(2025, 700, 17_000, 15_750, 40_000, true), 700)));
    let r = supported(&d);
    assert_eq!(line_amount(&r, "recovery_2025_taxable"), 125_000);
    assert_eq!(value(&r, "taxable_state_tax_refund"), 735);
    assert_eq!(value(&r, "agi"), 61_250);
    // Alone, the refund is 1,000 and the recovery 700.
    let mut alone = d.clone();
    alone.income.pop();
    assert_eq!(value(&supported(&alone), "taxable_state_tax_refund"), 1_000);
    // The two entries state different Schedule A totals for 2025: one correction note.
    let mut differ = d.clone();
    let FederalIncome::OtherIncome { data } = &mut differ.income.last_mut().unwrap().income else {
        panic!()
    };
    let OtherIncomeKind::Recovery { worksheet } = &mut data.kind else { panic!() };
    worksheet.itemized += 100;
    let r = estimate(&differ);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.display.key == "ISSUE_RECOVERY_REFUND_YEAR_FACTS_DIFFER"));
}

#[test]
fn d_real_estate_tax_refunded_goes_through_worksheet_2a_to_line_8z() {
    // 2025: state income tax 6,000 + real estate tax 8,000 = 14,000 on line 5d, capped
    // to 10,000 on line 5e (SALT cap, so 4,000 was never deducted). Itemized 20,000,
    // standard 15,750. A 1099-G refund of 1,500 and a real estate tax refund of 1,500.
    // Worksheet 2a: line 3 = 3,000; line 4 = 14,000 - 10,000 = 4,000; line 3 is not
    // more than line 4, so none of the refunds is taxable.
    let history = |property_refunded| TaxRefundHistory::IncomeTax {
        deduction_year: 2025,
        income_paid: 600_000,
        sales_available: 0,
        salt_paid: 1_400_000,
        salt_deducted: 1_000_000,
        itemized: 2_000_000,
        standard: 1_575_000,
        overall_limit: None,
        property_refunded,
        ordinary: true,
    };
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(refund_income(150_000, 0, history(150_000))));
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_state_tax_refund"), 0);
    assert_eq!(value(&r, "agi"), 60_000);
    // A larger real estate refund: 5,000 + 1,500 = 6,500; line 5 = 6,500 - 4,000 =
    // 2,500; line 7 = 1,500 / 6,500 x 2,500 = 576.92 (line 1a), line 8 = 5,000 / 6,500
    // x 2,500 = 1,923.08 (line 1b). Line 8 of Worksheet 2 = 20,000 - 15,750 = 4,250 so
    // line 11 = 2,500; 1a goes to Schedule 1 line 1 (577 in whole dollars) and 1b to
    // line 8z (1,923).
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(refund_income(150_000, 0, history(500_000))));
    let r = supported(&d);
    assert_eq!(line_amount(&r, "taxable_state_tax_refund"), 57_700);
    assert_eq!(line_amount(&r, "recovery_2025_taxable"), 250_000);
    assert_eq!(value(&r, "schedule_1_other_income"), 1_923);
}

// ---- E: Alaska dividends, digital assets and the Olympic prize --------------

#[test]
fn e_lettered_lines_8g_8v_and_the_olympic_exclusion_24c() {
    // Single. Alaska dividend 1,000 (8g), digital assets 800 (8v), Olympic prize 37,500
    // (8m). AGI before 24c = 60,000 + 1,000 + 800 + 37,500 = 99,300, not above
    // 1,000,000, so 24c excludes 37,500. AGI 61,800; taxable 45,700; midpoint
    // 45,725: 1,240 + 12% x 33,325 = 5,239.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(item(OtherIncomeKind::AlaskaDividend, 1_000)));
    add(&mut d, income(item(OtherIncomeKind::DigitalAssets, 800)));
    add(&mut d, income(item(OtherIncomeKind::OlympicPrize, 37_500)));
    let r = supported(&d);
    assert_eq!(value(&r, "alaska_dividends"), 1_000);
    assert_eq!(value(&r, "digital_asset_income"), 800);
    assert_eq!(value(&r, "olympic_prizes"), 37_500);
    assert_eq!(value(&r, "olympic_exclusion"), 37_500);
    assert_eq!(value(&r, "agi"), 61_800);
    assert_eq!(value(&r, "taxable_income"), 45_700);
    assert_eq!(tax(&r), 5_239);
    assert_eq!(crate::federal_model::TaxForm::Schedule1, printed_line(&r, "olympic_prizes").0);
    assert_eq!(printed_line(&r, "olympic_exclusion").1, "24c");
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn e_the_olympic_exclusion_is_lost_above_the_agi_limit() {
    // W-2 box 1 961,000: AGI before 24c = 961,000 + 1,000 + 800 + 37,500 = 1,000,300,
    // above 1,000,000, so the prize is taxable and 24c is zero. At 960,700 the AGI
    // before 24c is exactly 1,000,000 and the prize is excluded.
    for (wages, excluded, agi) in [(961_000, 0, 1_000_300), (960_700, 37_500, 962_500)] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
        add(&mut d, income(item(OtherIncomeKind::AlaskaDividend, 1_000)));
        add(&mut d, income(item(OtherIncomeKind::DigitalAssets, 800)));
        add(&mut d, income(item(OtherIncomeKind::OlympicPrize, 37_500)));
        let r = estimate(&d);
        assert_eq!(value(&r, "olympic_prizes"), 37_500);
        assert_eq!(value(&r, "olympic_exclusion"), excluded, "{wages}");
        assert_eq!(value(&r, "agi"), agi, "{wages}");
    }
}

#[test]
fn e_married_filing_separately_uses_the_500000_limit() {
    // AGI before 24c of 500,001 loses the exclusion; 500,000 keeps it.
    for (wages, excluded) in [(462_501, 0), (462_500, 37_500)] {
        let mut d = complete(FederalStatus::Separate);
        d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 0);
        add(&mut d, income(item(OtherIncomeKind::OlympicPrize, 37_500)));
        let r = estimate(&d);
        assert_eq!(value(&r, "olympic_exclusion"), excluded, "{wages}");
    }
}

#[test]
fn e_prize_card_insurance_unemployment_fund_and_other_kinds_reach_line_8z() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(item(OtherIncomeKind::CardInsurance { premiums: 20_000 }, 150_000 / 100)));
    add(
        &mut d,
        income(item(OtherIncomeKind::UnemploymentFund { contributions: 100_000 }, 1_000_000 / 100)),
    );
    add(
        &mut d,
        income(item(OtherIncomeKind::Other { description: "Research study 🧾".repeat(30) }, 300)),
    );
    add(&mut d, income(item(OtherIncomeKind::InmatePay, 400)));
    let r = supported(&d);
    // Card insurance 1,500 less 200 premiums = 1,300; the fund 10,000 less 1,000 = 9,000
    // taxable; study pay 300; inmate pay 400 on line 8u.
    assert_eq!(value(&r, "other_income_2_card_insurance"), 1_300);
    assert_eq!(value(&r, "other_income_3_unemployment_fund"), 9_000);
    assert_eq!(value(&r, "other_income_4_other"), 300);
    assert_eq!(value(&r, "inmate_pay"), 400);
    assert_eq!(value(&r, "schedule_1_other_income"), 1_300 + 9_000 + 300);
    assert_eq!(value(&r, "agi"), 60_000 + 1_300 + 9_000 + 300 + 400);
    // Premiums and contributions above the benefits floor at zero.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(item(OtherIncomeKind::CardInsurance { premiums: 900_000 }, 100)));
    assert_eq!(value(&supported(&d), "other_income_2_card_insurance"), 0);
}

// ---- F: W-2 pay moved to 8t and 8u ------------------------------------------

fn head_with_one_child_and_w2(box1: i64, deferred: i64, inmate: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Head);
    add_child(&mut d);
    d.income[0] = wage(1, Owner::Taxpayer, box1 * 100, 0);
    let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
    let mut data = edit_wage(data);
    data.deferred_pension = deferred * 100;
    data.inmate_pay = inmate * 100;
    drop(data);
    d
}

#[test]
fn f_nonqualified_pension_and_inmate_pay_leave_line_1a_and_the_earned_income_credit() {
    // HOH with one qualifying child; W-2 box 1 15,000 with a 6,000 nonqualified-plan
    // pension. Line 1a = 9,000, line 8t = 6,000, AGI 15,000, taxable 0. EIC earned
    // income 9,000: table row 9,000 (midpoint x 2 = 18,050), 18,050 x 34% / 2 =
    // 3,068.50 -> 3,069. Before this unit the 15,000 counted and the credit was
    // capped at 4,427.
    let d = head_with_one_child_and_w2(15_000, 6_000, 0);
    let r = supported(&d);
    assert_eq!(return_line(&r, "1a"), 900_000);
    assert_eq!(value(&r, "deferred_pension"), 6_000);
    assert_eq!(value(&r, "agi"), 15_000);
    assert_eq!(value(&r, "taxable_income"), 0);
    assert_eq!(value(&r, "earned_income_credit"), 3_069);
    // The same amount as inmate pay on the W-2 moves to 8u.
    let d = head_with_one_child_and_w2(15_000, 0, 6_000);
    let r = supported(&d);
    assert_eq!(value(&r, "inmate_pay"), 6_000);
    assert_eq!(value(&r, "earned_income_credit"), 3_069);
    assert_eq!(value(&r, "agi"), 15_000);
}

#[test]
fn f_the_two_parts_cannot_exceed_box_1() {
    // Box 1 is 10,000: 8,000 + 3,000 is more than the wages; 8,000 + 2,000 is not.
    for (deferred, inmate, rule) in
        [(800_000, 300_000, Some("W2_SCHEDULE_1_PAY_ABOVE_WAGES")), (800_000, 200_000, None)]
    {
        let mut w = w2_source(crate::tests::cross::interview::w2(1_000_000));
        w.answers.deferred_pension = W2Amount::of(deferred);
        w.answers.inmate_pay = inmate;
        // The two parts are topic answers: the answer rule reads the stored source.
        let mut entry = wage(1, Owner::Taxpayer, 1_000_000, 0);
        entry.income = FederalIncome::Wage { data: WageSource::W2(w.wage_source()) };
        assert_eq!(crate::interview::wage_answer_issue(&entry).map(|issue| issue.key), rule);
    }
}

#[test]
fn f_dependent_standard_deduction_keeps_both_parts_and_ira_compensation_only_the_inmate_pay() {
    // A claimable dependent's earned income for the standard deduction keeps both parts
    // (the Form 1040 instructions' footnote to the Standard Deduction Worksheet for
    // Dependents: "lines 8r, 8t, and 8u"): W-2 box 1 5,000 with 2,000 moved to 8t:
    // earned 5,000, standard deduction = 5,000 + 450 = 5,450, taxable income 0.
    for (deferred, inmate) in [(200_000, 0), (0, 200_000)] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.dependency =
            FilerDependency::CanBeClaimed { claimed: false };
        d.income[0] = wage(1, Owner::Taxpayer, 500_000, 0);
        let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
        let mut data = edit_wage(data);
        data.deferred_pension = deferred;
        data.inmate_pay = inmate;
        drop(data);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 5_000);
        assert_eq!(value(&r, "taxable_income"), 0);
    }
    // IRA compensation keeps inmate pay (wages for services) and leaves out the
    // pension (section 219(f)(1)): W-2 box 1 10,000 of which 8,000 moved; a 7,000
    // contribution is within 10,000 of compensation but above the 2,000 left with
    // the pension moved (an excess the page then asks about).
    let ira_case = |deferred, inmate| {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 1_000_000, 0);
        let FederalIncome::Wage { data } = &mut d.income[0].income else { panic!() };
        let mut data = edit_wage(data);
        data.deferred_pension = deferred;
        data.inmate_pay = inmate;
        drop(data);
        d.adjustments.as_mut().unwrap().iras = ira_facts(700_000, 0, Answer::No);
        d
    };
    assert_eq!(value(&supported(&ira_case(0, 800_000)), "ira_deduction"), 7_000);
    let r = estimate(&ira_case(800_000, 0));
    assert!(
        r.issues
            .iter()
            .any(|i| i.display.key == "ISSUE_IRA_TRADITIONAL_EXCESS_NEEDS_YEAR_END_VALUES")
    );
}
