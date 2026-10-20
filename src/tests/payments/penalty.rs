use super::*;
use crate::tests::support::assert::line_amount as cents;

// Every expected amount below is figured by hand from the draft 2026 Form 2210,
// the 2025 Instructions for Form 2210 (penalty worksheet and Table 1 rules),
// the 2026 draft Tax Table and the rates in
// tests/fixtures/irs-underpayment-rates-2026.csv (Rev. Rul. 2026-15: 6% for
// April–June 2026, 7% for July–December 2026; the unpublished first quarter of
// 2027 takes the last published 7%).
//
// Base return: single, one W-2 with $81,100 of wages and the given
// withholding. Taxable income $81,100 − $16,100 standard deduction = $65,000;
// the Tax Table row $65,000–$65,050 (midpoint $65,025) gives $1,240 + $4,560
// + 22% × $14,625 = $9,017.50, shown as $9,018. No credits or other taxes, so
// Form 1040 line 22 and line 24 are $9,018 and Form 2210 line 4 is $9,018.
fn base(withholding: i64, prior_tax: i64, prior_agi: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 8_110_000, withholding)];
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year =
        Some(PriorYearTax { full_year: true, tax: prior_tax, agi: Some(prior_agi), spouse_tax: 0 });
    d
}
fn dated(d: &mut FederalDraft, payments: &[(u32, i64)]) {
    d.payments.as_mut().unwrap().estimated_payments = payments
        .iter()
        .map(|&(date, amount)| DatedPayment {
            payer: crate::PaymentPayer::Joint,
            refund_property: None,
            date,
            amount,
        })
        .collect();
    d.payments.as_mut().unwrap().refresh_estimated_projection();
}
fn labeled(r: &FederalResult, label: &str) -> Option<i64> {
    r.return_lines
        .iter()
        .find(|line| status_text(&line.label) == label)
        .map(|line| line.amount / 100)
}

#[test]
fn form_2210_regular_method_matches_the_instructions_example() {
    let mut d = base(400_000, 800_000, 10_000_000);
    dated(&mut d, &[(20260610, 150_000)]);
    let r = supported(&d);
    // Part I: line 4 $9,018; line 5 90% = $8,116.20; line 6 $4,000; line 7
    // $5,018; line 8 100% of $8,000 (2025 AGI not over $150,000); line 9 the
    // smaller, $8,000, which is more than line 6.
    for (line, expected) in [
        (1, 901_800),
        (2, 0),
        (3, 0),
        (4, 901_800),
        (5, 811_620),
        (6, 400_000),
        (7, 501_800),
        (8, 800_000),
        (9, 800_000),
    ] {
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_{line}")), expected);
    }
    // Part III: $2,000 per installment; $1,000 of withholding on each due
    // date. (a) $1,000 paid, $1,000 short. (b) the June 10 $1,500 pays (a)'s
    // $1,000 first, leaving $500 plus $1,000 withholding: $500 short. (c) the
    // $1,000 withholding pays (b)'s $500, leaving $500: $1,500 short. (d) the
    // $1,000 withholding pays $1,000 of (c): $2,000 short.
    for (column, paid, short) in [
        ("a", 100_000, 100_000),
        ("b", 250_000, 50_000),
        ("c", 100_000, 150_000),
        ("d", 100_000, 200_000),
    ] {
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_10_{column}")), 200_000);
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_11_{column}")), paid);
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_17_{column}")), short);
    }
    // Penalty worksheet, each computation rounded to cents:
    //   (a) $1,000, 4/15–6/10, 56 days at 6%: $9.21
    //   (b) $500 paid 9/15: 15 days at 6% $1.23 + 77 days at 7% $7.38
    //   (c) $1,000 paid 1/15/27: 15 days $2.88 + 92 days $17.64 + 15 days $2.88
    //   (c) $500 unpaid to 4/15/27: 15 days $1.44 + 92 days $8.82 + 105 days $10.07
    //   (d) $2,000 unpaid 1/15/27–4/15/27, 90 days at 7%: $34.52
    // Line 19 = $96.07; Form 1040 line 38 = $96.
    assert_eq!(cents(&r, "form_2210_line_19"), 9_607);
    assert_eq!(return_line(&r, "38"), 9_600);
    // Line 26 is the dated $1,500; payments $5,500; the balance $9,018 −
    // $5,500 = $3,518 plus the $96 penalty: line 37 is $3,614.
    assert_eq!(return_line(&r, "26"), 150_000);
    assert_eq!(return_line(&r, "37"), 361_400);
    assert_eq!(
        labeled(&r, "1040 37 — Amount you owe including the estimated tax penalty"),
        Some(3_614)
    );
    assert_eq!(r.balance, Some(361_400));
    assert_eq!(r.refund, Some(0));

    // Over $150,000 of 2025 AGI, line 8 is 110% of $8,000 = $8,800 and line 9
    // becomes line 5, $8,116.20.
    let r = supported(&base(400_000, 800_000, 15_000_001));
    assert_eq!(cents(&r, "form_2210_line_8"), 880_000);
    assert_eq!(cents(&r, "form_2210_line_9"), 811_620);
    // A 2025 return shorter than 12 months skips line 8; line 9 is line 5.
    let mut d = base(400_000, 800_000, 10_000_000);
    d.payments.as_mut().unwrap().prior_year.as_mut().unwrap().full_year = false;
    let r = supported(&d);
    assert!(!r.lines.iter().any(|l| l.id == "form_2210_line_8"));
    assert_eq!(cents(&r, "form_2210_line_9"), 811_620);
}

#[test]
fn no_penalty_under_1000_or_with_zero_prior_tax() {
    // Without 2025 facts nothing changes: no Form 2210 and no line 38.
    let mut d = base(0, 0, 0);
    d.payments.as_mut().unwrap().prior_year = None;
    let r = supported(&d);
    assert!(!r.lines.iter().any(|l| l.id.starts_with("form_2210")));
    assert!(!r.return_lines.iter().any(|l| status_text(&l.label).starts_with("1040 38 ")));
    assert_eq!(r.balance, Some(901_800));

    // Line 7 under $1,000: $9,018 − $8,100 withheld = $918.
    let r = supported(&base(810_000, 800_000, 10_000_000));
    assert_eq!(cents(&r, "form_2210_line_7"), 91_800);
    assert_eq!(cents(&r, "form_2210_line_19"), 0);
    assert_eq!(return_line(&r, "38"), 0);
    assert_eq!(r.balance, Some(91_800));

    // Line 4 under $1,000: $25,000 of wages, taxable income $8,900, Tax Table
    // row $8,900–$8,950 at 10% of $8,925 = $892.50, shown as $893.
    let mut d = base(0, 800_000, 10_000_000);
    d.income = vec![wage(1, Owner::Taxpayer, 2_500_000, 0)];
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_line_4"), 89_300);
    assert_eq!(cents(&r, "form_2210_line_19"), 0);
    assert_eq!(r.balance, Some(89_300));

    // Zero 2025 tax on a full-year return: line 8 is $0, so line 9 is $0 and
    // not more than line 6; nothing withheld and still no penalty.
    let r = supported(&base(0, 0, 10_000_000));
    assert_eq!(cents(&r, "form_2210_line_8"), 0);
    assert_eq!(cents(&r, "form_2210_line_9"), 0);
    assert_eq!(cents(&r, "form_2210_line_19"), 0);
    assert_eq!(r.balance, Some(901_800));
}

#[test]
fn estimated_payments_apply_by_date_to_the_periods() {
    // $4,000 withheld ($1,000 per due date) and three dated payments: $1,000
    // on April 15 (column a), $2,000 on September 16 (column d) and $1,000 on
    // February 1, 2027 (after the last due date, in no column).
    let mut d = base(400_000, 800_000, 10_000_000);
    dated(&mut d, &[(20270201, 100_000), (20260415, 100_000), (20260916, 200_000)]);
    let r = supported(&d);
    // (a) $2,000 paid, none short. (b) $1,000 paid: $1,000 short. (c) the
    // $1,000 withholding pays (b): $2,000 short. (d) the September 16 $2,000
    // pays (c), then the $1,000 withholding: $1,000 short.
    for (column, paid, short) in [
        ("a", 200_000, 0),
        ("b", 100_000, 100_000),
        ("c", 100_000, 200_000),
        ("d", 300_000, 100_000),
    ] {
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_11_{column}")), paid);
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_17_{column}")), short);
    }
    // (b) $1,000 paid 9/15: 15 days at 6% $2.47 + 77 days at 7% $14.77
    // (c) $2,000 paid 9/16: 1 day at 7% $0.38
    // (d) $1,000 paid 2/1/27: 17 days at 7% $3.26
    // Line 19 = $20.88; line 38 = $21.
    assert_eq!(cents(&r, "form_2210_line_19"), 2_088);
    assert_eq!(return_line(&r, "38"), 2_100);
    // Line 26 $4,000; payments $8,000; balance $1,018; line 37 with the
    // penalty $1,039.
    assert_eq!(return_line(&r, "26"), 400_000);
    assert_eq!(return_line(&r, "37"), 103_900);
    assert_eq!(r.balance, Some(103_900));

    // 2026-10-16, Derived carrier: the optional stored projection matches
    // the exact dated sources; a contradictory projection is invalid. An
    // annual source without dates retains the existing Form2210 boundary.
    let mut matching = d.clone();
    matching.payments.as_mut().unwrap().estimated = Some(400_000);
    assert_eq!(supported(&matching).balance, Some(103_900));
    let mut different = d.clone();
    different.payments.as_mut().unwrap().estimated = Some(300_000);
    unsupported(&different, "Estimated payments");
    let mut undated = base(400_000, 800_000, 10_000_000);
    undated.payments.as_mut().unwrap().estimated = Some(150_000);
    unsupported(&undated, "Form 2210");
    // Box D needs the four period amounts adding up to line 6.
    let mut by_date = d.clone();
    by_date.payments.as_mut().unwrap().withholding_by_date = true;
    unsupported(&by_date, "Form 2210");
    by_date.payments.as_mut().unwrap().withholding_periods =
        Some(PeriodWithholding { first: 100_000, second: 100_000, third: 100_000, fourth: 50_000 });
    unsupported(&by_date, "Form 2210");
    // Equal period amounts give the same result as the default.
    by_date.payments.as_mut().unwrap().withholding_periods = Some(PeriodWithholding {
        first: 100_000,
        second: 100_000,
        third: 100_000,
        fourth: 100_000,
    });
    assert_eq!(cents(&supported(&by_date), "form_2210_line_19"), 2_088);
}

// Schedule AI (box C). Base return: single, one W-2 with $100,000 of wages and
// $2,000 withheld, treated as $500 paid on each due date; no estimated
// payments; 2025 tax $12,000 on 2025 AGI of $100,000 (not over $150,000).
// Taxable income $100,000 − $16,100 = $83,900; Tax Table row $83,900–$83,950
// (midpoint $83,925): $1,240 + $4,560 + 22% × $33,525 ($7,375.50) =
// $13,175.50, shown as $13,176. Part I: line 4 $13,176; line 5 $11,858.40;
// line 6 $2,000; line 7 $11,176; line 8 $12,000; line 9 $11,858.40, so the
// regular installment (line 24) is $2,964.60.
fn period(agi: i64) -> AnnualizedPeriod {
    AnnualizedPeriod {
        agi,
        itemized: 0,
        qbi: 0,
        additional: 0,
        self_employment: 0,
        social_wages: 0,
    }
}
fn annualized(agis: [i64; 4]) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 10_000_000, 200_000)];
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 1_200_000,
        agi: Some(10_000_000),
        spouse_tax: 0,
    });
    pay.annualized = true;
    pay.annualized_income = Some(AnnualizedIncome {
        through_march: period(agis[0]),
        through_may: period(agis[1]),
        through_august: period(agis[2]),
        full_year: period(agis[3]),
    });
    d
}
fn columns(r: &FederalResult, line: &str) -> [i64; 4] {
    ["a", "b", "c", "d"].map(|column| cents(r, &alloc::format!("{line}_{column}")))
}

#[test]
fn schedule_ai_annualized_installments_replace_part_iii_line_10() {
    let d = annualized([1_000_000, 2_500_000, 4_000_000, 10_000_000]);
    let r = supported(&d);
    // Line 3: $10,000 × 4 = $40,000; $25,000 × 2.4 = $60,000; $40,000 × 1.5 =
    // $60,000; $100,000 × 1. Line 11 subtracts the $16,100 standard deduction:
    // $23,900, $43,900, $43,900, $83,900. Line 14 from the Tax Table:
    //   $23,925 midpoint: $1,240 + 12% × $11,525 = $2,623
    //   $43,925 midpoint: $1,240 + 12% × $31,525 = $5,023
    //   $83,925 midpoint: $13,176 (above)
    // No self-employment tax, other taxes or credits, so line 19 = line 14.
    // Line 21: 22.5% × $2,623 = $590.175 → $590.18; 45% × $5,023 = $2,260.35;
    // 67.5% × $5,023 = $3,390.525 → $3,390.53; 90% × $13,176 = $11,858.40.
    assert_eq!(columns(&r, "form_2210_ai_line_1"), [1_000_000, 2_500_000, 4_000_000, 10_000_000]);
    assert_eq!(columns(&r, "form_2210_ai_line_3"), [4_000_000, 6_000_000, 6_000_000, 10_000_000]);
    assert_eq!(columns(&r, "form_2210_ai_line_11"), [2_390_000, 4_390_000, 4_390_000, 8_390_000]);
    assert_eq!(columns(&r, "form_2210_ai_line_14"), [262_300, 502_300, 502_300, 1_317_600]);
    assert_eq!(columns(&r, "form_2210_ai_line_21"), [59_018, 226_035, 339_053, 1_185_840]);
    // Lines 22–27, column by column (line 24 is $2,964.60 throughout):
    //   (a) 23 = $590.18; 25 = 0; 26 = $2,964.60; 27 = $590.18
    //   (b) 23 = $2,260.35 − $590.18 = $1,670.17; 25 = $2,964.60 − $590.18 =
    //       $2,374.42; 26 = $5,339.02; 27 = $1,670.17
    //   (c) 23 = $3,390.53 − $2,260.35 = $1,130.18; 25 = $5,339.02 −
    //       $1,670.17 = $3,668.85; 26 = $6,633.45; 27 = $1,130.18
    //   (d) 23 = $11,858.40 − $3,390.53 = $8,467.87; 25 = $6,633.45 −
    //       $1,130.18 = $5,503.27; 26 = $8,467.87; 27 = $8,467.87
    // The four line 27 amounts add up to line 9, $11,858.40.
    assert_eq!(columns(&r, "form_2210_ai_line_23"), [59_018, 167_017, 113_018, 846_787]);
    assert_eq!(columns(&r, "form_2210_ai_line_25"), [0, 237_442, 366_885, 550_327]);
    let line_27 = [59_018, 167_017, 113_018, 846_787];
    assert_eq!(columns(&r, "form_2210_ai_line_27"), line_27);
    assert_eq!(columns(&r, "form_2210_line_10"), line_27);
    // Part III with $500 withheld on each due date:
    //   (a) $90.18 short; (b) $500 pays (a) first, $409.82 left: $1,260.35
    //   short; (c) $500 pays (b): $1,130.18 short; (d) $500 pays (b):
    //   $8,467.87 short.
    assert_eq!(columns(&r, "form_2210_line_17"), [9_018, 126_035, 113_018, 846_787]);
    // Penalty worksheet (6% to June 30, 2026, then 7%), each computation
    // rounded to cents:
    //   (a) $90.18, 4/15–6/15, 61 days at 6%: $0.90
    //   (b) $500 paid 9/15: 15 days at 6% $1.23 + 77 days at 7% $7.38
    //   (b) $500 paid 1/15/27: $1.23 + 92 days $8.82 + 92 days $8.82 + 15 days $1.44
    //   (b) $260.35 to 4/15/27: $0.64 + $4.59 + $4.59 + 105 days $5.24
    //   (c) $1,130.18 from 9/15: 15 days $3.25 + 92 days $19.94 + 105 days $22.76
    //   (d) $8,467.87, 90 days at 7%: $146.16
    // Line 19 = $236.99; Form 1040 line 38 = $237.
    assert_eq!(cents(&r, "form_2210_line_19"), 23_699);
    assert_eq!(return_line(&r, "38"), 23_700);
    assert_eq!(r.balance, Some(1_117_600 + 23_700));

    // The regular method on the same return: $2,964.60 per installment.
    //   (a) $2,464.60 short: $500 paid 6/15 (61 days at 6% $5.01), $500 paid
    //       9/15 (76 days at 6% $6.25 + 77 days at 7% $7.38), $500 paid
    //       1/15/27 ($6.25 + $8.82 + $8.82 + $1.44) and $964.60 to 4/15/27
    //       ($12.05 + $17.02 + $17.02 + $19.42)
    //   (b) $2,964.60 from 6/15 to 4/15/27 ($7.31 + $52.31 + $52.31 + $59.70)
    //   (c) $2,964.60 from 9/15 to 4/15/27 ($8.53 + $52.31 + $59.70)
    //   (d) $2,964.60 for 90 days at 7%: $51.17
    // Line 19 = $452.82, more than the annualized $236.99.
    let mut regular = d.clone();
    regular.payments.as_mut().unwrap().annualized = false;
    let r = supported(&regular);
    assert_eq!(cents(&r, "form_2210_line_19"), 45_282);
    assert_eq!(return_line(&r, "38"), 45_300);
    assert!(!r.lines.iter().any(|l| l.id.starts_with("form_2210_ai_")));
}

#[test]
fn schedule_ai_fourth_column_must_reconcile_to_the_return() {
    let d = annualized([1_000_000, 2_500_000, 4_000_000, 10_000_000]);
    supported(&d);
    let refused = |d: &FederalDraft, text: &str| {
        let r = estimate(d);
        assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
        assert!(
            r.issues.iter().any(|i| i.section.contains("Form 2210") && i.message.contains(text)),
            "{:?}",
            r.issues
        );
        let issue = r
            .issues
            .iter()
            .find(|issue| {
                issue.section == "Form 2210 underpayment penalty" && issue.message.contains(text)
            })
            .unwrap();
        assert_eq!(
            crate::interview::issue_page(
                d,
                &issue.context,
                Some({
                    let m = &issue.display;
                    m.key.as_str()
                })
            ),
            Some(crate::TaxSection::IncomeByPeriod)
        );
    };
    // Column (d) line 1 is $99,000, not the $100,000 of AGI on the return.
    let mut short = d.clone();
    short.payments.as_mut().unwrap().annualized_income.as_mut().unwrap().full_year.agi = 9_900_000;
    refused(&short, "Schedule AI column (d)");
    // AGI matches but a $500 QBI deduction the return does not take makes
    // column (d) line 13 $83,400, not the $83,900 of taxable income.
    let mut deduction = d.clone();
    deduction.payments.as_mut().unwrap().annualized_income.as_mut().unwrap().full_year.qbi = 50_000;
    refused(&deduction, "Schedule AI column (d)");
    // Itemized deductions on a standard-deduction return.
    let mut itemized = d.clone();
    itemized
        .payments
        .as_mut()
        .unwrap()
        .annualized_income
        .as_mut()
        .unwrap()
        .through_march
        .itemized = 100_000;
    refused(&itemized, "Schedule AI line 4");
    // The election without the per-period record.
    let mut missing = d.clone();
    missing.payments.as_mut().unwrap().annualized_income = None;
    refused(&missing, "box C");
}

#[test]
fn line_37_includes_the_penalty_once() {
    let once = |r: &FederalResult, reference: &str| {
        r.return_lines
            .iter()
            .filter(|l| {
                l.form_line
                    .as_ref()
                    .is_some_and(|f| f.form == TaxForm::Form1040 && f.line == reference)
            })
            .count()
    };
    // Balance due: the Schedule AI return owes $13,176 − $2,000 = $11,176
    // plus the $237 penalty. Line 37 appears once, at $11,413; line 38 is $237.
    let r = supported(&annualized([1_000_000, 2_500_000, 4_000_000, 10_000_000]));
    assert_eq!(once(&r, "37"), 1);
    assert_eq!(once(&r, "38"), 1);
    assert_eq!(return_line(&r, "37"), 1_141_300);
    assert_eq!(return_line(&r, "38"), 23_700);
    // 2026-10-18: final 2025 Form 1040 instructions, Figuring the Penalty:
    // penalty above line34 requires printed zero35a/36, and the remainder37.
    assert_eq!(once(&r, "35a"), 1);
    assert_eq!(return_line(&r, "35a"), 0);
    assert_eq!(return_line(&r, "36"), 0);

    // Overpayment larger than the penalty: the C24a base return ($9,018 of
    // tax, 2025 tax $8,000, so $2,000 installments) with nothing withheld and
    // one $10,000 estimated payment on January 10, 2027, which pays columns
    // (a) to (c) late and covers (d).
    //   (a) $2,000 4/15–1/10/27: 76 days at 6% $24.99 + 92 days $35.29 +
    //       92 days $35.29 + 10 days $3.84
    //   (b) $2,000 6/15–1/10/27: 15 days at 6% $4.93 + $35.29 + $35.29 + $3.84
    //   (c) $2,000 9/15–1/10/27: 15 days at 7% $5.75 + $35.29 + $3.84
    // Line 19 = $223.64; line 38 $224. Line 34 is $10,000 − $9,018 = $982;
    // line 35a is $982 − $224 = $758; line 37 is $0, once.
    let mut d = base(0, 800_000, 10_000_000);
    dated(&mut d, &[(20270110, 1_000_000)]);
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_line_19"), 22_364);
    assert_eq!(return_line(&r, "34"), 98_200);
    assert_eq!(return_line(&r, "38"), 22_400);
    assert_eq!(return_line(&r, "35a"), 75_800);
    assert_eq!(once(&r, "37"), 1);
    assert_eq!(return_line(&r, "37"), 0);
    assert_eq!(r.refund, Some(75_800));
    assert_eq!(r.balance, Some(0));

    // No 2025 facts: no penalty, and line 37 is the balance due as before.
    let mut d = base(0, 0, 0);
    d.payments.as_mut().unwrap().prior_year = None;
    let r = supported(&d);
    assert_eq!(once(&r, "37"), 1);
    assert_eq!(once(&r, "38"), 0);
    assert_eq!(return_line(&r, "37"), 901_800);
}

#[test]
fn schedule_ai_line_27_never_exceeds_the_regular_installment() {
    // Income front-loaded: $40,000, $60,000, $80,000 and $100,000.
    //   (a) $160,000 − $16,100 = $143,900: $1,240 + $4,560 + $12,166 + 24% ×
    //       $38,200 = $27,134; 22.5% = $6,105.15
    //   (b) $144,000 − $16,100 = $127,900: $17,966 + 24% × $22,200 = $23,294;
    //       45% = $10,482.30
    //   (c) $120,000 − $16,100 = $103,900: $5,800 + 22% × $53,500 = $17,570;
    //       67.5% = $11,859.75
    //   (d) $11,858.40, as on the base return.
    // Line 23 is never below line 26 ($2,964.60 with nothing carried on line
    // 25): (a) $6,105.15; (b) $10,482.30 − $2,964.60 = $7,517.70; (c)
    // $11,859.75 − $5,929.20 = $5,930.55; (d) $11,858.40 − $8,893.80 =
    // $2,964.60. Each line 27 is the regular installment and the penalty is
    // the regular method's $452.82.
    let r = supported(&annualized([4_000_000, 6_000_000, 8_000_000, 10_000_000]));
    assert_eq!(columns(&r, "form_2210_ai_line_21"), [610_515, 1_048_230, 1_185_975, 1_185_840]);
    assert_eq!(columns(&r, "form_2210_ai_line_23"), [610_515, 751_770, 593_055, 296_460]);
    assert_eq!(columns(&r, "form_2210_ai_line_25"), [0; 4]);
    assert_eq!(columns(&r, "form_2210_ai_line_27"), [296_460; 4]);
    assert_eq!(cents(&r, "form_2210_line_19"), 45_282);
}

#[test]
fn schedule_ai_part_ii_prorates_the_social_security_wage_base() {
    // Single: $150,000 of W-2 Social Security wages and a $40,000 cash
    // business. Schedule SE: $40,000 × 92.35% = $36,940; 12.4% of the
    // $34,500 left under $184,500 = $4,278 plus 2.9% × $36,940 = $1,071.26:
    // $5,349. Half, $2,675 (rounded), is an adjustment: AGI $187,325. QBI
    // deduction 20% × ($40,000 − $2,675) = $7,465 (below 20% of the $171,225
    // taxable income before it). Taxable income $171,225 − $7,465 = $163,760.
    // No Additional Medicare Tax ($186,940 is under $200,000).
    let mut d = annualized([0, 0, 0, 18_732_500]);
    d.income = vec![wage(1, Owner::Taxpayer, 15_000_000, 3_000_000)];
    add(&mut d, income(cash_business(40_000)));
    let facts = d.payments.as_mut().unwrap().annualized_income.as_mut().unwrap();
    for (period, (agi, profit, wages, qbi)) in [
        &mut facts.through_march,
        &mut facts.through_may,
        &mut facts.through_august,
        &mut facts.full_year,
    ]
    .into_iter()
    .zip([
        (4_500_000, 1_000_000, 5_000_000, 180_000),
        (8_000_000, 2_000_000, 6_250_000, 360_000),
        (12_500_000, 3_000_000, 10_000_000, 550_000),
        (18_732_500, 4_000_000, 15_000_000, 746_500),
    ]) {
        period.agi = agi;
        period.self_employment = profit;
        period.social_wages = wages;
        period.qbi = qbi;
    }
    let r = supported(&d);
    // Part II by column (line 28 = profit × 92.35%; line 31 = the prorated
    // base $46,125, $76,875, $123,000, $184,500 minus wages):
    //   (a) 28 $9,235; 31 $0 (wages $50,000 exceed $46,125): 33 $0;
    //       35 0.116 × $9,235 = $1,071.26; 36 $1,071.26
    //   (b) 28 $18,470; 31 $14,375: 33 0.2976 × $14,375 = $4,278;
    //       35 0.0696 × $18,470 = $1,285.512 → $1,285.51; 36 $5,563.51
    //   (c) 28 $27,705; 31 $23,000: 33 0.186 × $23,000 = $4,278;
    //       35 0.0435 × $27,705 = $1,205.1675 → $1,205.17; 36 $5,483.17
    //   (d) 28 $36,940; 31 $34,500: 33 0.124 × $34,500 = $4,278;
    //       35 0.029 × $36,940 = $1,071.26; 36 $5,349.26
    assert_eq!(columns(&r, "form_2210_ai_line_15"), [107_126, 556_351, 548_317, 534_926]);
    // Column (d) Part II within $1 of Schedule SE's $5,349 reconciles; a
    // full-year profit of $39,000 does not (line 28 $36,016.50; line 33
    // $4,278; line 35 $1,044.48; line 36 $5,322.48) and is refused.
    let mut off = d.clone();
    off.payments.as_mut().unwrap().annualized_income.as_mut().unwrap().full_year.self_employment =
        3_900_000;
    unsupported(&off, "Form 2210");
    assert!(estimate(&off).issues.iter().any(|i| i.message.contains("Part II column (d)")));
}

#[test]
fn waiver_requests_reduce_line_19_and_line_38_by_the_amount_waived() {
    // Same fixture as form_2210_regular_method_matches_the_instructions_example:
    // the penalty before any waiver is $96.07. The instructions have the filer
    // subtract the waiver from it (the IRS decides the waiver) (audit D2210-2).
    let mut d = base(400_000, 800_000, 10_000_000);
    dated(&mut d, &[(20260610, 150_000)]);
    let baseline = supported(&d);
    let penalty = cents(&baseline, "form_2210_line_19");
    assert_eq!(penalty, 9_607);
    assert_eq!(return_line(&baseline, "37"), 361_400);
    assert!(baseline.lines.iter().all(|l| l.id != "form_2210_penalty_before_waiver"));

    // Box A: waive the entire penalty.
    let mut full = d.clone();
    full.payments.as_mut().unwrap().waiver = Some(PenaltyWaiver::Full {
        reason: PenaltyWaiverReason::CasualtyOrDisasterOrUnusual,
        explanation: "A house fire destroyed our records in October 2026.".into(),
    });
    let r = supported(&full);
    assert_eq!(cents(&r, "form_2210_line_19"), 0);
    assert_eq!(cents(&r, "form_2210_penalty_before_waiver"), penalty);
    assert_eq!(return_line(&r, "38"), 0);
    assert!(r.review_notes.iter().any(|n| {
        n.contains("Form 2210 Part II box A waiver requested")
            && n.contains("casualty, disaster, or other unusual circumstance")
            && n.contains("A house fire destroyed our records in October 2026.")
            && n.contains("the IRS decides the waiver")
    }));
    assert!(!r.lines.iter().any(|l| l.id == "form_2210_waiver_requested_amount"));

    // Box B: waive $50 of the penalty; the requested amount is its own line.
    let mut partial = d.clone();
    partial.payments.as_mut().unwrap().waiver = Some(PenaltyWaiver::Partial {
        reason: PenaltyWaiverReason::RetirementOrDisability,
        explanation: "Retired at age 63 in March 2026.".into(),
        amount: 5_000,
    });
    let r = supported(&partial);
    assert_eq!(cents(&r, "form_2210_line_19"), 4_607);
    assert_eq!(cents(&r, "form_2210_penalty_before_waiver"), penalty);
    assert_eq!(return_line(&r, "38"), 4_600);
    assert_eq!(return_line(&r, "37"), 356_400);
    assert!(r.review_notes.iter().any(|n| {
        n.contains("Form 2210 Part II box B waiver requested")
            && n.contains("retirement after reaching age 62 or disability")
            && n.contains("Retired at age 63 in March 2026.")
            && n.contains("the IRS decides the waiver")
    }));
    assert_eq!(cents(&r, "form_2210_waiver_requested_amount"), 5_000);

    // A request above the penalty gives zero, never a negative penalty.
    let mut over = d;
    over.payments.as_mut().unwrap().waiver = Some(PenaltyWaiver::Partial {
        reason: PenaltyWaiverReason::RetirementOrDisability,
        explanation: "Retired at age 63 in March 2026.".into(),
        amount: 500_000,
    });
    let r = supported(&over);
    assert_eq!(cents(&r, "form_2210_line_19"), 0);
    assert_eq!(return_line(&r, "38"), 0);
    assert_eq!(cents(&r, "form_2210_penalty_before_waiver"), penalty);
}

#[test]
fn box_e_uses_the_combined_prior_year_tax_on_line_8() {
    // Box E: a joint filer whose 2025 filing status differed adds the
    // spouse's separately filed 2025 tax to the amount already entered.
    // $8,000 (filer) + $2,000 (spouse) = $10,000; 2025 AGI $100,000 is not
    // over $150,000, so line 8 is 100% of $10,000 = $10,000, which no longer
    // binds (line 5, 90% of the $9,018 current tax, is the smaller $8,116.20).
    let mut d = base(400_000, 800_000, 10_000_000);
    d.payments.as_mut().unwrap().prior_year.as_mut().unwrap().spouse_tax = 200_000;
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_line_8"), 1_000_000);
    assert_eq!(cents(&r, "form_2210_line_9"), 811_620);
}

#[test]
fn farmer_two_thirds_rule_uses_one_installment_at_66_percent() {
    // The 2025 answer Yes (T2-49b; the $0-gross farm leaves 2026 short of
    // two-thirds and does not change the wage-only tax) with $130,275 of
    // wages: taxable income $130,275 - $16,100 standard
    // deduction = $114,175, over $100,000 so the rate schedule applies
    // exactly (no Tax Table midpoint rounding): 10%×$12,400 + 12%×$38,000 +
    // 22%×$55,300 + 24%×$8,475 = $1,240 + $4,560 + $12,166 + $2,034 =
    // $20,000. No withholding, no other taxes or credits: Form 2210-F lines 1
    // and 3 are $20,000, line 4 is $0 and line 6 is $20,000. A farmer or
    // fisher figures the penalty on Form 2210-F, not Form 2210: line 7 is
    // 66⅔% of line 6, $13,333.33 (the exact 2/3 fraction, $13,333.333...,
    // truncated to cents). Line 8 is $0, so line 9 is $20,000. Line 10 is
    // 100% of the $100,000 2025 tax, never 110%; line 11, the smaller of
    // lines 7 and 10, is $13,333.33 in one installment due January 15, 2027.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 13_027_500, 0)];
    add(&mut d, income(farm_business(vec![], 0)));
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 10_000_000,
        agi: Some(10_000_000),
        spouse_tax: 0,
    });
    pay.farmer_fisherman_prior_year = true;
    dated(&mut d, &[(20270110, 1_000_000)]);
    let r = supported(&d);
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form_2210_line_")));
    assert_eq!(cents(&r, "form_2210_f_line_6"), 2_000_000);
    assert_eq!(cents(&r, "form_2210_f_line_7"), 1_333_333);
    assert_eq!(cents(&r, "form_2210_f_line_9"), 2_000_000);
    assert_eq!(cents(&r, "form_2210_f_line_10"), 10_000_000);
    assert_eq!(cents(&r, "form_2210_f_line_11"), 1_333_333);
    // The $10,000 paid January 10 is paid by the January 15 due date (line
    // 12), leaving $3,333.33 underpaid (line 13) from January 15 to April 15,
    // 2027: 90 days (16 in January, 28 in February, 31 in March, 15 in
    // April), all in the unpublished Q1 2027 period, which takes the last
    // published 7% rate: $3,333.33 × 90 × 7% / 365 = $57.53.
    assert_eq!(cents(&r, "form_2210_f_line_12"), 1_000_000);
    assert_eq!(cents(&r, "form_2210_f_line_13"), 333_333);
    assert_eq!(cents(&r, "form_2210_f_line_16"), 5_753);

    // No penalty at all when the return is filed and the tax paid in full by
    // March 1, 2027.
    d.payments.as_mut().unwrap().farmer_paid_by_march = true;
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_f_line_16"), 0);
}

#[test]
fn column_d_january_31_payment_is_refused_by_name() {
    // The map's open question (a balance paid January 16-31, 2027 treated as
    // paid January 15 for column (d)): this App refuses by name rather than
    // implement it, since it has no fact recording that a dated payment
    // specifically settles the balance due. It still computes ordinary dated
    // payments outside that window (as the other tests above show).
    let mut d = base(0, 800_000, 10_000_000);
    dated(&mut d, &[(20270120, 200_000)]);
    unsupported(&d, "Form 2210");
    // A payment on January 15 itself, or February 1, is unaffected.
    let mut on_time = base(0, 800_000, 10_000_000);
    dated(&mut on_time, &[(20270115, 200_000)]);
    supported(&on_time);
    let mut after = base(0, 800_000, 10_000_000);
    dated(&mut after, &[(20270201, 200_000)]);
    supported(&after);
}

// C-fixture (2026-09-25): an independent full-return Form 2210 fixture for
// the form's typical case (a joint return combining wages and a
// self-employed business), plus three boundary tests. Every total below is
// worked by hand from the 2026 draft Form 1040/Schedule SE/Form 8995/Form
// 2210, the 2025 Instructions for Form 2210 carried to 2026, and the rate
// fixture; none is copied from this App's own output.
//
// Base return: married filing jointly, taxpayer born June 15, 1979 and
// spouse born March 12, 1981 (both under 65 at the end of 2026, no
// additional standard deduction). One W-2 (Owner::Taxpayer): box 1 $96,000,
// box 2 (withholding) $6,000; boxes 3-6 follow from wages under the
// $184,500 wage base (Social Security wages $96,000, Medicare wages
// $96,000, no Additional Medicare withholding since wages are under
// $200,000). A sole proprietorship (same owner, material participation,
// not a specified service, no payroll or assets): receipts $70,000, one
// "Supplies" expense $10,000, net profit $60,000.
//
// Schedule SE: line 4a/4c $60,000 x 92.35% = $55,410 (over the $400 floor).
// This person's own $96,000 of Social Security wages leaves $184,500 -
// $96,000 = $88,500 of wage-base room, more than $55,410, so the whole
// amount takes the 12.4% portion: $55,410 x 12.4% = $6,870.84; the
// uncapped 2.9% portion: $55,410 x 2.9% = $1,606.89; line 12 SE tax
// $8,477.73, rounded to $8,478 (Schedule 2 line 4); line 13, half of an
// even $8,478, exactly $4,239 (Schedule 1 line 15).
//
// Form 1040 line 9 (total income): $96,000 wages + $60,000 Schedule C =
// $156,000. Line 11 (AGI): $156,000 - $4,239 half-SE = $151,761. Line 12
// (standard deduction, i1040gi line 38 table, MFJ, neither senior or
// blind): $32,200.
//
// QBI deduction (Form 8995): taxable income before QBI ($151,761 -
// $32,200 = $119,561) is far under the 2026 MFJ $403,550 threshold, so no
// wage/UBIA limit and no specified-service reduction apply. The
// business's QBI amount is its $60,000 net profit less its own $4,239
// half-SE deduction (its only source, so it carries all of it) = $55,761;
// 20% of that is $11,152.20, rounded $11,152. The income limit (20% of
// $119,561 = $23,912.20, rounded $23,912) and the $400 active-business
// floor do not bind. Line 13 (QBI deduction): $11,152. Line 15 (taxable
// income): $119,561 - $11,152 = $108,409.
//
// Line 16 (tax, Tax Computation Worksheet: taxable income is over the
// $100,000 Tax Table ceiling; 2026 MFJ brackets end at $24,800/$100,800 for
// the 10%/12% rates): 10% x $24,800 = $2,480.00; 12% x ($100,800-$24,800)
// = 12% x $76,000 = $9,120.00; 22% x ($108,409-$100,800) = 22% x $7,609 =
// $1,673.98; total $13,273.98, rounded $13,274.
//
// No AMT: Form 6251 line 1 (taxable income) plus the standard-deduction
// add-back is $108,409 + $32,200 = $140,609, only $409 over the 2026 MFJ
// $140,200 exemption (phase-out starts at $1,000,000 AMTI, not reached);
// tentative minimum tax 26% x $409 = $106.34, rounded $106, is below the
// $13,274 regular tax, so line 17/Schedule 2 line 1a stays $0. No
// Additional Medicare Tax (combined Medicare wages and net SE earnings
// $96,000 + $55,410 = $151,410 is under the $250,000 MFJ threshold) or
// NIIT (no investment income).
//
// Line 22 (tax after credits, none claimed): $13,274. Line 23 (Schedule 2
// line 21, SE tax only): $8,478. Line 24 (total tax): $21,752.
//
// Payments: withholding $6,000 (line 25d), treated as paid evenly, $1,500
// on each Form 2210 due date since box D is not elected. Three dated
// estimated payments of $1,500 on April 10, June 12 and September 14,
// 2026 (line 26: $4,500; none in January). Line 33 (total payments):
// $10,500.
//
// Form 2210 Part I: line 1 $13,274; line 2 $8,478 (no household
// employment tax); line 3 $0 (no refundable credits); line 4 $21,752;
// line 5 (90%) $19,576.80; line 6 (withholding) $6,000; line 7 $15,752;
// line 8 (prior-year tax $14,000, prior-year AGI $140,000, not over
// $150,000: 100%) $14,000; line 9 (the smaller of 5 and 8) $14,000, more
// than line 6, so Part III applies. $1,000 or more on both lines 4 and 7.
//
// Part III: each required installment (line 10) is 25% of $14,000 =
// $3,500. Each period receives $1,500 of withholding (paid on the due
// date) plus, for (a)-(c), the matching $1,500 estimated payment (paid
// before the due date), for $3,000 received against a $3,500 installment:
//   (a) 4/15: $3,000 received, $500 short.
//   (b) 6/15: $3,000 received; it first pays (a)'s $500 short (58 days,
//       4/15-6/12, at 6%: $4.77), leaving $2,500 for a $3,500 installment:
//       $500 newly short.
//   (c) 9/15: $3,000 received; it first pays (b)'s $500 short (91 days,
//       6/15-9/14, 15 at 6% $2.47 + 76 at 7% $14.58 = $17.05), leaving
//       $2,500 for a $3,500 installment: $1,000 short, on top of the (b)
//       shortfall already paid, for $1,500 carried into (d).
//   (d) 1/15/27: only the $1,500 withholding is received (no January
//       payment). It pays (c)'s carried $1,500 short (122 days, 9/15-1/15,
//       all at 7%: 15 days $4.32 + 92 days $26.47 + 15 days $4.32 =
//       $35.11), leaving nothing for (d)'s own $3,500 installment, which
//       stays fully unpaid to April 15, 2027 (90 days at 7%: $60.41).
// Line 19: $4.77 + $17.05 + $35.11 + $60.41 = $117.34. Form 1040 line 38
// (whole dollars): $117.
//
// Line 37: balance due before the penalty ($21,752 - $10,500 = $11,252)
// plus the $117 penalty = $11,369.
fn joint_business_return(prior_agi: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19790615;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19810312;
    d.income = vec![wage(1, Owner::Taxpayer, 9_600_000, 600_000)];
    add(
        &mut d,
        income(FederalIncome::Business {
            data: {
                let FederalIncome::Business { mut data } = cash_business(70_000) else {
                    unreachable!()
                };
                data.expenses = vec![ordinary_business_expense(
                    BusinessExpenseCategory::Other { label: "Supplies".into() },
                    1_000_000,
                )];
                data
            },
        }),
    );
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year =
        Some(PriorYearTax { full_year: true, tax: 1_400_000, agi: Some(prior_agi), spouse_tax: 0 });
    dated(&mut d, &[(20260410, 150_000), (20260612, 150_000), (20260914, 150_000)]);
    d
}

#[test]
fn underpayment_return_matches_hand_computation() {
    let d = joint_business_return(14_000_000);
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), 8_478);
    assert_eq!(value(&r, "adjustments"), 4_239);
    assert_eq!(value(&r, "agi"), 151_761);
    assert_eq!(value(&r, "deduction"), 32_200);
    assert_eq!(value(&r, "qbi_deduction"), 11_152);
    assert_eq!(value(&r, "taxable_income"), 108_409);
    assert_eq!(r.tax, Some(2_175_200));
    for (line, expected) in [
        (1, 1_327_400),
        (2, 847_800),
        (3, 0),
        (4, 2_175_200),
        (5, 1_957_680),
        (6, 600_000),
        (7, 1_575_200),
        (8, 1_400_000),
        (9, 1_400_000),
    ] {
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_{line}")), expected);
    }
    for (column, received, short) in [
        ("a", 300_000, 50_000),
        ("b", 300_000, 100_000),
        ("c", 300_000, 150_000),
        ("d", 150_000, 350_000),
    ] {
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_10_{column}")), 350_000);
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_11_{column}")), received);
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_17_{column}")), short);
    }
    assert_eq!(cents(&r, "form_2210_line_19"), 11_734);
    assert_eq!(return_line(&r, "33"), 1_050_000);
    assert_eq!(return_line(&r, "37"), 1_136_900);
    assert_eq!(return_line(&r, "38"), 11_700);
    assert_eq!(r.balance, Some(1_136_900));
    assert_eq!(r.refund, Some(0));
}

#[test]
fn prior_year_agi_over_150000_uses_110_percent() {
    // Same facts as above, except the 2025 AGI is $0.01 over $150,000:
    // line 8 becomes 110% of $14,000 = $15,400, still smaller than line 5
    // ($19,576.80), so line 9 rises to $15,400 and each required
    // installment (line 10) to 25% of that, $3,850.
    let d = joint_business_return(15_000_001);
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_line_8"), 1_540_000);
    assert_eq!(cents(&r, "form_2210_line_9"), 1_540_000);
    // Each period again receives $3,000 ((a)-(c)) or $1,500 ((d)) against a
    // bigger $3,850 installment, so the shortfall grows by $350 each
    // period: (a) $850, (b) $1,700, (c) $2,550, (d) $3,850 (the carried
    // $2,550 plus the unmet $1,300 balance of its own installment).
    for (column, short) in [("a", 85_000), ("b", 170_000), ("c", 255_000), ("d", 385_000)] {
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_17_{column}")), short);
    }
    // Penalty worksheet (same day counts and rates as the base fixture,
    // now on larger shortfalls): (a) $8.10 (58 days at 6% on $850); (b)
    // $28.99 (91 days, 15 at 6% + 76 at 7%, on $1,700); (c) $59.69 (122
    // days at 7% on $2,550); (d) $84.57 (90 days at 7% on $3,850).
    // Line 19 = $181.35; Form 1040 line 38 = $181.
    assert_eq!(cents(&r, "form_2210_line_19"), 18_135);
    assert_eq!(return_line(&r, "38"), 18_100);
    assert_eq!(return_line(&r, "37"), 1_143_300);
}

#[test]
fn january_15_payment_clears_the_fourth_period_only() {
    // Same base fixture, with one more estimated payment: $3,500 on
    // January 15, 2027, the whole shortfall the base fixture otherwise
    // carries into the fourth period. The estimated payments list is
    // ordered ahead of the synthetic withholding entries for the same
    // date, so the $3,500 payment is applied first: it pays the $1,500
    // carried from period (c) (the same 122 days at 7%, $35.11, as the
    // base fixture, since that money was unpaid from 9/15 to 1/15
    // regardless of what arrives on 1/15), leaving $2,000 credit, plus the
    // $1,500 withholding paid the same day, for $3,500 exactly against
    // the fourth period's own $3,500 installment: nothing left unpaid,
    // and no interest accrues on an installment paid by its own due date.
    // Periods (a)-(c) are unaffected: $4.77 + $17.05 + $35.11 = $56.93.
    let mut d = joint_business_return(14_000_000);
    let pay = d.payments.as_mut().unwrap();
    pay.estimated_payments.push(DatedPayment {
        payer: crate::PaymentPayer::Joint,
        refund_property: None,
        date: 20270115,
        amount: 350_000,
    });
    pay.refresh_estimated_projection();
    let r = supported(&d);
    for (column, short) in [("a", 50_000), ("b", 100_000), ("c", 150_000), ("d", 0)] {
        assert_eq!(cents(&r, &alloc::format!("form_2210_line_17_{column}")), short);
    }
    assert_eq!(cents(&r, "form_2210_line_19"), 5_693);
    assert_eq!(return_line(&r, "38"), 5_700);
    // Total payments $10,500 + $3,500 = $14,000; balance $21,752 - $14,000
    // = $7,752, plus the $57 penalty: $7,809.
    assert_eq!(return_line(&r, "33"), 1_400_000);
    assert_eq!(return_line(&r, "37"), 780_900);
}

#[test]
fn annualized_method_lowers_the_penalty_for_late_year_income() {
    // A joint, wage-only return with income heavily concentrated in the
    // last four months of the year: cumulative AGI $30,000 through March,
    // $50,000 through May, $70,000 through August and $200,000 for the
    // full year (no itemized deductions, QBI or self-employment in any
    // period, matching the return). Standard deduction $32,200; full-year
    // taxable income $200,000 - $32,200 = $167,800, matching line 1/13
    // column (d) exactly (no other adjustments).
    //
    // Schedule AI line 3 (annualizing factors 4, 2.4, 1.5, 1): (a)
    // $30,000 x 4 = $120,000; (b) $50,000 x 2.4 = $120,000; (c) $70,000 x
    // 1.5 = $105,000; (d) $200,000. Line 13 (less the $32,200 standard
    // deduction each time): (a)/(b) $87,800; (c) $72,800; (d) $167,800.
    // Line 14 (Tax Table under $100,000, midpoint rounding; Tax
    // Computation Worksheet at or above it):
    //   (a)/(b) $87,800: midpoint $87,825; 10% x $24,800 + 12% x $63,025 =
    //     $2,480.00 + $7,563.00 = $10,043.00, rounded $10,043.
    //   (c) $72,800: midpoint $72,825; $2,480.00 + 12% x $48,025 =
    //     $2,480.00 + $5,763.00 = $8,243.00, rounded $8,243.
    //   (d) $167,800 (no midpoint, at or above $100,000): $2,480.00 +
    //     12% x $76,000 + 22% x $67,000 = $2,480 + $9,120 + $14,740 =
    //     $26,340.00.
    // No self-employment tax or credits, so line 19 = line 14. Line 21
    // (applicable percentages 22.5%, 45%, 67.5%, 90%): (a) $2,259.68; (b)
    // $4,519.35; (c) $5,564.03; (d) $23,706.00.
    //
    // Installments (line 24, 25% of line 9 $23,706.00, is $5,926.50 every
    // period): (a) line 23 $2,259.68 is under line 26 ($5,926.50), so line
    // 27 (and line 10) is $2,259.68; (b) line 23 $4,519.35 - $2,259.68 =
    // $2,259.67, under its line 26 ($5,926.50 + the $3,666.82 line 25
    // carried from (a) = $9,593.32), so line 27 is $2,259.67; (c) line 23
    // $5,564.03 - $4,519.35 = $1,044.68, under its line 26 ($5,926.50 +
    // the $7,333.65 carried = $13,260.15), so line 27 is $1,044.68; (d)
    // line 23 $23,706.00 - $5,564.03 = $18,141.97, exactly its line 26
    // ($5,926.50 + the $12,215.47 carried), so line 27 is $18,141.97. The
    // four line 27 amounts total $23,706.00, line 9.
    //
    // Withholding $16,000, treated evenly: $4,000 per period, comfortably
    // covering the small (a)-(c) installments and banking the rest, so
    // only (d) is underpaid: $4,000 x 3 banked = $12,000 credit entering
    // (d) against installments of $2,259.68 + $2,259.67 + $1,044.68 =
    // $5,564.03, leaving $6,435.97 of credit, plus (d)'s own $4,000
    // withholding for $10,435.97 against an $18,141.97 installment:
    // $7,706.00 short, 90 days at 7% from 1/15/27 to 4/15/27: $133.01.
    // Form 1040 line 38 = $133.
    //
    // The regular (non-annualized) method on the same return needs $12,000
    // of 2025 tax (AGI $150,000, not over the threshold, so 100%) against
    // $26,340.00 x 90% = $23,706.00 (still the smaller); the same $23,706
    // required annual payment split evenly gives $5,926.50 every period,
    // each $1,926.50 short of the $4,000 received, growing the shortfall
    // every period and giving a materially larger $353.96 penalty —
    // confirming the annualized method's lower penalty for income
    // concentrated late in the year.
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![wage(1, Owner::Taxpayer, 20_000_000, 1_600_000)];
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 2_400_000,
        agi: Some(15_000_000),
        spouse_tax: 0,
    });
    pay.annualized = true;
    pay.annualized_income = Some(AnnualizedIncome {
        through_march: period(3_000_000),
        through_may: period(5_000_000),
        through_august: period(7_000_000),
        full_year: period(20_000_000),
    });
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 200_000);
    assert_eq!(value(&r, "taxable_income"), 167_800);
    assert_eq!(columns(&r, "form_2210_ai_line_1"), [3_000_000, 5_000_000, 7_000_000, 20_000_000]);
    assert_eq!(
        columns(&r, "form_2210_ai_line_3"),
        [12_000_000, 12_000_000, 10_500_000, 20_000_000]
    );
    assert_eq!(columns(&r, "form_2210_ai_line_13"), [8_780_000, 8_780_000, 7_280_000, 16_780_000]);
    assert_eq!(columns(&r, "form_2210_ai_line_14"), [1_004_300, 1_004_300, 824_300, 2_634_000]);
    assert_eq!(columns(&r, "form_2210_ai_line_21"), [225_968, 451_935, 556_403, 2_370_600]);
    let line_27 = [225_968, 225_967, 104_468, 1_814_197];
    assert_eq!(columns(&r, "form_2210_ai_line_27"), line_27);
    assert_eq!(columns(&r, "form_2210_line_10"), line_27);
    assert_eq!(columns(&r, "form_2210_line_17"), [0, 0, 0, 770_600]);
    assert_eq!(cents(&r, "form_2210_line_19"), 13_301);
    assert_eq!(return_line(&r, "38"), 13_300);

    let mut regular = d.clone();
    regular.payments.as_mut().unwrap().annualized = false;
    let r = supported(&regular);
    assert_eq!(cents(&r, "form_2210_line_19"), 35_396);
    assert_eq!(return_line(&r, "38"), 35_400);
    assert!(!r.lines.iter().any(|l| l.id.starts_with("form_2210_ai_")));
}

// T2-49: the Form 1040-ES Estimated Tax Worksheet for 2027, figured from the
// finished 2026 return when the payments page asks for it. Every expected
// amount is figured by hand from the 2026 Form 1040-ES package (the 2027
// package is unpublished, so its 2026 Tax Rate Schedules stand in): single,
// one W-2 of $81,100 with the given withholding, no other income, credits or
// taxes. Line 3 = $81,100 − $16,100 = $65,000; the Rate Schedule (not the
// Tax Table) gives $1,240 + 12% × $38,000 + 22% × $14,600 = $1,240 + $4,560
// + $3,212 = $9,012, while the return's own Tax Table tax is $9,018.
fn next_year(withholding: i64, request: NextYearEstimate) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 8_110_000, withholding)];
    d.payments.as_mut().unwrap().next_year = request.into();
    d
}
fn es_line(r: &FederalResult, line: &str) -> Option<i64> {
    r.lines
        .iter()
        .find(|l| l.id == alloc::format!("form_1040_es_line_{line}"))
        .map(|l| l.amount / 100)
}

#[test]
fn form_1040_es_worksheet_matches_hand_computation() {
    use libertas::AvroDecode;

    // Withholding $4,000. Lines 1–15: AGI $81,100; 2a the $16,100 standard
    // deduction; 2b, 2c $0; 2d $16,100; 3 $65,000; 4 $9,012; 5 $0; 6 $9,012;
    // 7 $0; 8 $9,012; 9, 10 $0; 11a $9,012; 11b $0; 11c $9,012; 12a 90% =
    // $8,110.80 → $8,111; 12b 100% of the 2026 tax $9,018 (AGI not over
    // $150,000); 12c the smaller, $8,111; 13 $4,000; 14a $4,111 (more than
    // zero); 14b $5,012 (not under $1,000); 15 ¼ × $4,111 = $1,027.75 →
    // $1,028.
    let d = next_year(
        400_000,
        NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year: FarmerNextYear::No },
    );
    let r = supported(&d);
    for (line, expected) in [
        ("1", 81_100),
        ("2a", 16_100),
        ("2b", 0),
        ("2c", 0),
        ("2d", 16_100),
        ("3", 65_000),
        ("4", 9_012),
        ("5", 0),
        ("6", 9_012),
        ("7", 0),
        ("8", 9_012),
        ("9", 0),
        ("10", 0),
        ("11a", 9_012),
        ("11b", 0),
        ("11c", 9_012),
        ("12a", 8_111),
        ("12b", 9_018),
        ("12c", 8_111),
        ("13", 4_000),
        ("14a", 4_111),
        ("14b", 5_012),
        ("15", 1_028),
    ] {
        assert_eq!(es_line(&r, line), Some(expected), "line {line}");
    }
    assert!(
        r.lines
            .iter()
            .filter(|l| l.id.starts_with("form_1040_es_line_"))
            .all(|l| l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form1040Es))
    );
    assert!(
        r.review_notes.iter().any(|n| n.contains("pay $1028 by each of April 15, 2027")),
        "{:?}",
        r.review_notes
    );
    let note = r
        .review_notes
        .iter()
        .find(|n| {
            let m = &n.display;
            m.key == "NOTE_1040ES_QUARTERLY_INSTALLMENT"
        })
        .unwrap();
    let encoded = note.display.encode();
    let mut offset = 0;
    let decoded = crate::common::FormattedMessage::avro_decode(&encoded, &mut offset).unwrap();
    assert_eq!(offset, encoded.len());
    assert_eq!(
        decoded.1,
        alloc::vec![libertas::LibertasMessageArgumentDecode::Currency {
            currency_code: "USD".into(),
            scale: 0,
            minor_units: 1_028,
        }]
    );
    // The worksheet's exact quarter and its rounded payment remain distinct.
    let installment = r.lines.iter().find(|line| line.id == "form_1040_es_line_15").unwrap();
    let mut offset = 0;
    let decoded = crate::common::FormattedMessage::avro_decode(
        &installment.explanation.encode(),
        &mut offset,
    )
    .unwrap();
    assert_eq!(
        decoded.1,
        alloc::vec![libertas::LibertasMessageArgumentDecode::Currency {
            currency_code: "USD".into(),
            scale: 2,
            minor_units: 102_775,
        }]
    );
    // Not requested: no worksheet and no note.
    let r = supported(&next_year(400_000, NextYearEstimate::NotNeeded));
    assert!(es_line(&r, "1").is_none());
    assert!(!r.review_notes.iter().any(|n| n.contains("Form 1040-ES")));
}

#[test]
fn form_1040_es_110_percent_starts_above_150000_agi() {
    // Wages $150,000, no withholding: taxable $133,900, over $100,000 so the
    // return's own tax is the Rate Schedule too: $17,966 + 24% × $28,200 =
    // $24,734. Line 12b is 100% of $24,734 at exactly $150,000 of AGI. Line
    // 12a is 90% = $22,260.60 → $22,261, the smaller.
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 15_000_000, 0)];
    d.payments.as_mut().unwrap().next_year =
        NextYearEstimateRecord::SameAsThisReturn { farmer_fisherman_next_year: FarmerNextYear::No };
    let r = supported(&d);
    assert_eq!(es_line(&r, "11c"), Some(24_734));
    assert_eq!(es_line(&r, "12a"), Some(22_261));
    assert_eq!(es_line(&r, "12b"), Some(24_734));
    assert_eq!(es_line(&r, "12c"), Some(22_261));
    // One dollar more: taxable $133,901, tax $24,734.24 → $24,734, and line
    // 12b is 110% = $27,207.40 → $27,207.
    d.income = vec![wage(1, Owner::Taxpayer, 15_000_100, 0)];
    let r = supported(&d);
    assert_eq!(es_line(&r, "12b"), Some(27_207));
    assert_eq!(es_line(&r, "12c"), Some(22_261));
}

#[test]
fn form_1040_es_stops_when_line_14b_is_under_1000() {
    // Withholding $8,012: 14a = $8,111 − $8,012 = $99 (more than zero), 14b
    // = $9,012 − $8,012 = $1,000, not under $1,000, so payments are required:
    // line 15 = ¼ × $99 = $24.75 → $25.
    let r = supported(&next_year(
        801_200,
        NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year: FarmerNextYear::No },
    ));
    assert_eq!(es_line(&r, "14a"), Some(99));
    assert_eq!(es_line(&r, "14b"), Some(1_000));
    assert_eq!(es_line(&r, "15"), Some(25));
    // Withholding $8,013: 14b = $999, under $1,000: the worksheet stops at
    // line 14b with no line 15, and the note says no payments are required.
    let r = supported(&next_year(
        801_300,
        NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year: FarmerNextYear::No },
    ));
    assert_eq!(es_line(&r, "14a"), Some(98));
    assert_eq!(es_line(&r, "14b"), Some(999));
    assert_eq!(es_line(&r, "15"), None);
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("not required to make 2027 estimated tax payments")),
        "{:?}",
        r.review_notes
    );
}

#[test]
fn form_1040_es_stops_when_withholding_covers_the_required_payment() {
    // Withholding $8,111 equals line 12c: 14a = $0, zero or less, so the
    // worksheet stops at line 14a with no line 14b or 15.
    let r = supported(&next_year(
        811_100,
        NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year: FarmerNextYear::No },
    ));
    assert_eq!(es_line(&r, "12c"), Some(8_111));
    assert_eq!(es_line(&r, "14a"), Some(0));
    assert_eq!(es_line(&r, "14b"), None);
    assert_eq!(es_line(&r, "15"), None);
}

// T2-49b: the farmer and fisherman test is one fact, at least two-thirds of
// gross income from farming or fishing, for three years: 2025 asked
// (FederalPayments.farmer_fisherman_prior_year), 2026 figured from this return,
// 2027 asked with the Form 1040-ES worksheet. Form 2210 qualifies on 2025 or
// 2026, Form 1040-ES on 2026 or 2027. Every case is single with no
// withholding and a $100,000 2025 tax (line 8 never binds), and every farm
// or fishing business nets $0 so the tax is the wage tax alone.
fn farmer_return(wages: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
    d.next_id = 2;
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 10_000_000,
        agi: Some(10_000_000),
        spouse_tax: 0,
    });
    d
}
fn answers(d: &mut FederalDraft, year_prior: Answer, year_2027: FarmerNextYear) {
    let pay = d.payments.as_mut().unwrap();
    pay.farmer_fisherman_prior_year = year_prior == Answer::Yes;
    pay.next_year =
        NextYearEstimateRecord::SameAsThisReturn { farmer_fisherman_next_year: year_2027 };
}
// A farm or commercial fishing business with the given gross income and
// the same expenses, so it nets $0.
fn break_even_farm(gross: i64) -> FederalIncome {
    raised_farm(gross, gross)
}
fn break_even_fishing(gross: i64, fishing: bool) -> FederalIncome {
    let mut v = cash_business(gross);
    let FederalIncome::Business { data } = &mut v else { unreachable!() };
    data.expenses = vec![ordinary_business_expense(
        BusinessExpenseCategory::Other { label: "Boat, fuel and crew".into() },
        gross * 100,
    )];
    data.commercial_fishing = fishing;
    v
}

#[test]
fn farmer_test_uses_2025_for_form_2210_and_2027_for_form_1040_es() {
    // Wages $200,000, no farm: 2026 does not qualify (no farming or fishing
    // income, and no test lines). Taxable $183,900, tax $17,966 + 24% ×
    // $78,200 = $36,734 by the Rate Schedule; no Additional Medicare Tax at
    // exactly the $200,000 threshold.
    //
    // Farmed in 2025 only (2025 Yes, 2027 No): Form 2210-F applies, line 6
    // $36,734 and line 7 66⅔% = $24,489.33 (the exact fraction truncated to
    // cents). Form 1040-ES does not qualify: 12a 90% = $33,060.60 → $33,061; 12b,
    // AGI above $150,000, 110% of $36,734 = $40,407.40 → $40,407; 12c
    // $33,061; 13 $0; 14a $33,061; 15 ¼ × $33,061 = $8,265.25 → $8,265.
    let mut d = farmer_return(200_000);
    answers(&mut d, Answer::Yes, FarmerNextYear::No);
    let r = supported(&d);
    assert!(r.lines.iter().all(|l| l.id != "gross_income_return_year"));
    assert_eq!(cents(&r, "form_2210_f_line_6"), 3_673_400);
    assert_eq!(cents(&r, "form_2210_f_line_7"), 2_448_933);
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form_2210_line_")));
    assert_eq!(es_line(&r, "11c"), Some(36_734));
    assert_eq!(es_line(&r, "12a"), Some(33_061));
    assert_eq!(es_line(&r, "12b"), Some(40_407));
    assert_eq!(es_line(&r, "12c"), Some(33_061));
    assert_eq!(es_line(&r, "14a"), Some(33_061));
    assert_eq!(es_line(&r, "15"), Some(8_265));
    assert!(
        r.review_notes.iter().any(|n| n.contains("pay $8265 by each of April 15, 2027")),
        "{:?}",
        r.review_notes
    );

    // Starts farming in 2027 (2025 No, 2027 Yes): Form 2210 (not 2210-F) line 5 is 90%
    // of $36,734 = $33,060.60. Form 1040-ES qualifies: 12a 66⅔% =
    // $24,489.33 → $24,489; 12b stays 100% of the 2026 tax ($36,734)
    // although AGI is above $150,000; 12c $24,489; 14a $24,489; 15 the whole
    // of 14a, $24,489, one installment due January 18, 2028.
    answers(&mut d, Answer::No, FarmerNextYear::Yes);
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_line_5"), 3_306_060);
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form_2210_f_")));
    assert_eq!(es_line(&r, "12a"), Some(24_489));
    assert_eq!(es_line(&r, "12b"), Some(36_734));
    assert_eq!(es_line(&r, "12c"), Some(24_489));
    assert_eq!(es_line(&r, "14a"), Some(24_489));
    assert_eq!(es_line(&r, "15"), Some(24_489));
    assert!(
        r.review_notes.iter().any(|n| n.contains("one installment by January 18, 2028")),
        "{:?}",
        r.review_notes
    );
    // The 2027 question is asked with the expected changes too: Yes there,
    // with nothing changed, gives the same lines.
    d.payments.as_mut().unwrap().next_year = NextYearEstimateRecord::WithChanges {
        agi: None,
        deductions: None,
        credits: None,
        withholding: None,
        farmer_fisherman_next_year: FarmerNextYear::Yes,
    };
    let r = supported(&d);
    assert_eq!(es_line(&r, "12a"), Some(24_489));
    assert_eq!(es_line(&r, "15"), Some(24_489));
}

#[test]
fn next_year_not_needed_cannot_supply_a_newly_applicable_farming_answer() {
    let mut d = farmer_return(200_000);
    answers(&mut d, Answer::No, FarmerNextYear::NotNeeded);
    assert_eq!(es_line(&supported(&d), "12a"), None);
    answers(&mut d, Answer::No, FarmerNextYear::No);
    assert!(es_line(&supported(&d), "12a").is_some());
}

#[test]
fn farm_return_meeting_two_thirds_for_2026_needs_neither_answer() {
    // Wages $150,000 and a farm with $300,000 of gross income (Schedule F
    // line 9) and $300,000 of expenses. Farming $300,000; total gross income
    // $150,000 + $300,000 = $450,000, of which two-thirds is exactly
    // $300,000: the 2026 test is met. Taxable $150,000 − $16,100 = $133,900,
    // tax $17,966 + 24% × $28,200 = $24,734 (Rate Schedule, over $100,000).
    // Neither answer is needed (the payments page offers the 2025 answer
    // as "Yes, for 2026" and the 2027 answer as Not needed alone; a stored
    // 2025 Not sure is ignored): Form 2210-F line 7 =
    // 66⅔% of $24,734 = $16,489.33; Form 1040-ES 12a = $16,489.33 →
    // $16,489, 12b 100% of $24,734 (AGI not above $150,000), 12c $16,489,
    // 14a $16,489, 15 $16,489 in one installment.
    let mut d = farmer_return(150_000);
    add(&mut d, income(break_even_farm(300_000)));
    answers(&mut d, Answer::NotSure, FarmerNextYear::NotNeeded);
    let r = supported(&d);
    assert_eq!(cents(&r, "farming_fishing_gross_income_return_year"), 30_000_000);
    assert_eq!(cents(&r, "gross_income_return_year"), 45_000_000);
    assert_eq!(cents(&r, "form_2210_f_line_6"), 2_473_400);
    assert_eq!(cents(&r, "form_2210_f_line_7"), 1_648_933);
    assert_eq!(es_line(&r, "12a"), Some(16_489));
    assert_eq!(es_line(&r, "12b"), Some(24_734));
    assert_eq!(es_line(&r, "12c"), Some(16_489));
    assert_eq!(es_line(&r, "14a"), Some(16_489));
    assert_eq!(es_line(&r, "15"), Some(16_489));

    // One dollar less of farm gross income: $299,999 of $449,999 is under
    // two-thirds (3 × 299,999 = 899,997 < 2 × 449,999 = 899,998), so the
    // answers are needed again: a 2027 Not sure or a stored 2027 Not needed
    // is taken as No with a warning, and No gives the
    // ordinary 90%: Form 2210 (not 2210-F) line 5 $22,260.60, Form 1040-ES 12a
    // $22,260.60 → $22,261 in four installments of $5,565.25 → $5,565.
    let mut d = farmer_return(150_000);
    add(&mut d, income(break_even_farm(299_999)));
    answers(&mut d, Answer::No, FarmerNextYear::No);
    let r = supported(&d);
    assert_eq!(cents(&r, "farming_fishing_gross_income_return_year"), 29_999_900);
    assert_eq!(cents(&r, "gross_income_return_year"), 44_999_900);
    assert_eq!(cents(&r, "form_2210_line_5"), 2_226_060);
    assert_eq!(es_line(&r, "12a"), Some(22_261));
    assert_eq!(es_line(&r, "15"), Some(5_565));
}

#[test]
fn commercial_fishing_business_counts_toward_the_2026_test() {
    // Wages $150,000 and a Schedule C business with $300,000 of gross income
    // (line 7) and $300,000 of expenses: the same $24,734 tax as above.
    // Marked commercial fishing, its $300,000 is gross income from fishing,
    // exactly two-thirds of $450,000: Form 1040-ES 12a $16,489 in one
    // installment with the 2027 answer Not needed. Not marked, the return has no
    // farming or fishing income and the 2027 No gives 90%, $22,261.
    let mut d = farmer_return(150_000);
    add(&mut d, income(break_even_fishing(300_000, true)));
    answers(&mut d, Answer::No, FarmerNextYear::NotNeeded);
    let r = supported(&d);
    assert_eq!(cents(&r, "farming_fishing_gross_income_return_year"), 30_000_000);
    assert_eq!(cents(&r, "gross_income_return_year"), 45_000_000);
    assert_eq!(es_line(&r, "12a"), Some(16_489));
    assert_eq!(es_line(&r, "15"), Some(16_489));
    assert_eq!(cents(&r, "form_2210_f_line_7"), 1_648_933);
    let mut d = farmer_return(150_000);
    add(&mut d, income(break_even_fishing(300_000, false)));
    answers(&mut d, Answer::No, FarmerNextYear::No);
    let r = supported(&d);
    assert!(r.lines.iter().all(|l| l.id != "gross_income_return_year"));
    assert_eq!(es_line(&r, "12a"), Some(22_261));
    assert_eq!(cents(&r, "form_2210_line_5"), 2_226_060);
}

#[test]
fn form_1040_es_expected_changes_replace_the_return_amounts() {
    // Withholding $4,000 on the return; the filer expects $90,000 of AGI,
    // $500 of credits and $6,000 of withholding in 2027, with the deduction
    // left as this return's. Line 3 = $90,000 − $16,100 = $73,900; line 4 =
    // $5,800 + 22% × $23,500 = $10,970; line 8 = $10,970 − $500 = $10,470 =
    // line 11c; 12a 90% = $9,423; 12b stays this return's $9,018 tax; 12c
    // $9,018; 13 $6,000; 14a $3,018; 14b $4,470; 15 = $754.50 → $755.
    let d = next_year(
        400_000,
        NextYearEstimate::WithChanges {
            agi: Some(9_000_000),
            deductions: None,
            credits: Some(50_000),
            withholding: Some(600_000),
            farmer_fisherman_next_year: FarmerNextYear::No,
        },
    );
    let r = supported(&d);
    for (line, expected) in [
        ("1", 90_000),
        ("2a", 16_100),
        ("3", 73_900),
        ("4", 10_970),
        ("7", 500),
        ("8", 10_470),
        ("11c", 10_470),
        ("12a", 9_423),
        ("12b", 9_018),
        ("12c", 9_018),
        ("13", 6_000),
        ("14a", 3_018),
        ("14b", 4_470),
        ("15", 755),
    ] {
        assert_eq!(es_line(&r, line), Some(expected), "line {line}");
    }
}

#[test]
fn form_1040_es_is_not_figured_for_a_final_return() {
    let mut d = next_year(
        400_000,
        NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year: FarmerNextYear::No },
    );
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20260601, ordinary: true, estate_medical: 0 });
    unsupported(&d, "Form 1040-ES");
}

// Form 2210-F (unit 2210-F, 2026-10-08): the farmer and fisher penalty. A
// return that meets the two-thirds test (2026 from the return, or the 2025
// answer) figures Form 2210-F: one installment, due January 15, 2027, of the
// smaller of 66⅔% of the 2026 tax and 100% of the 2025 tax (section
// 6654(i)(1)(C): never 110%, no Schedule AI), the March 1 rule in place of the
// January 31 rule. Every amount below is worked by hand from section 6654, the
// draft 2026 Form 2210-F lines 1-16 and the rate fixture (Q1 2027 takes the
// last published 7%); none is copied from this App's output.
//
// Base (fixtures A, B, E, F): single; W-2 $150,000 with the given amount
// withheld; a break-even farm with $300,000 of Schedule F line 9 (net $0, so no
// self-employment tax). Farming $300,000; gross income $150,000 + $300,000 =
// $450,000; 3 × 300,000 = 900,000 ≥ 2 × 450,000: the 2026 test is met.
// Taxable income $150,000 - $16,100 = $133,900; tax by the Rate Schedule
// $17,966 + 24% × $28,200 = $24,734.
fn farm_2210_f(farm_gross: i64, withheld: i64) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 15_000_000, withheld * 100)];
    d.next_id = 2;
    add(&mut d, income(break_even_farm(farm_gross)));
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 1_550_000,
        agi: Some(16_000_000),
        spouse_tax: 0,
    });
    answers(&mut d, Answer::No, FarmerNextYear::NotNeeded);
    d
}
fn line_38_explanation(r: &FederalResult) -> alloc::string::String {
    let line = FormLine::new(TaxForm::Form1040, "38");
    let row = r
        .return_lines
        .iter()
        .find(|row| row.form_line.as_ref() == Some(&line))
        .expect("Form 1040 line 38");
    status_message(&row.explanation).key.clone()
}

#[test]
fn farmer_penalty_uses_100_percent_of_the_prior_year_never_110() {
    // Fixture A: 2025 tax $15,500, 2025 AGI $160,000 (over $150,000), a full
    // year; $10,000 withheld; one $2,000 estimated payment on 2027-01-10.
    //   line 1 = 24,734.00; line 2 = 0; line 3 = 24,734.00; line 4 = 0;
    //   line 6 = 24,734.00
    //   line 7 = 24,734 × 2/3 = 16,489.333... → 16,489.33
    //   line 8 = 10,000.00; line 9 = 14,734.00 (at least $1,000)
    //   line 10 = 100% × 15,500 = 15,500.00 (110% would be 17,050.00: not
    //   applied, section 6654(i)(1)(C) disregards (d)(1)(C))
    //   line 11 = min(16,489.33, 15,500.00) = 15,500.00
    //   line 12 = 10,000 + 2,000 = 12,000.00; line 13 = 3,500.00
    //   line 16 = 3,500.00 × 90 / 365 × 7% = 60.410... → 60.41 (January 15 to
    //   April 15, 2027: 16 + 28 + 31 + 15 = 90 days)
    //   Form 1040 line 38 = $60. (The 110% defect gave line 11 = 16,489.33,
    //   line 13 = 4,489.33 and $77.49.)
    let mut d = farm_2210_f(300_000, 10_000);
    dated(&mut d, &[(20270110, 200_000)]);
    let r = supported(&d);
    for (line, expected) in [
        (1, 2_473_400),
        (2, 0),
        (3, 2_473_400),
        (4, 0),
        (6, 2_473_400),
        (7, 1_648_933),
        (8, 1_000_000),
        (9, 1_473_400),
        (10, 1_550_000),
        (11, 1_550_000),
        (12, 1_200_000),
        (13, 350_000),
        (16, 6_041),
    ] {
        assert_eq!(cents(&r, &alloc::format!("form_2210_f_line_{line}")), expected, "line {line}");
    }
    assert!(r.lines.iter().all(|l| l.id != "form_2210_f_line_5"));
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form_2210_line_")));
    assert_eq!(return_line(&r, "38"), 6_000);
    assert_eq!(line_38_explanation(&r), "EXPLAIN_1040_38_3");

    // A2: the 2025 answer Yes on a wage-only return with the same tax. The
    // $24,734 tax, so the same lines and the same $60.41.
    let mut d = farmer_return(150_000);
    d.income = vec![wage(1, Owner::Taxpayer, 15_000_000, 1_000_000)];
    let pay = d.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 1_550_000,
        agi: Some(16_000_000),
        spouse_tax: 0,
    });
    answers(&mut d, Answer::Yes, FarmerNextYear::No);
    dated(&mut d, &[(20270110, 200_000)]);
    let r = supported(&d);
    assert!(r.lines.iter().all(|l| l.id != "gross_income_return_year"));
    assert_eq!(cents(&r, "form_2210_f_line_7"), 1_648_933);
    assert_eq!(cents(&r, "form_2210_f_line_11"), 1_550_000);
    assert_eq!(cents(&r, "form_2210_f_line_13"), 350_000);
    assert_eq!(cents(&r, "form_2210_f_line_16"), 6_041);
}

#[test]
fn farmer_payment_in_late_january_is_figured_and_march_1_removes_the_penalty() {
    // Fixture B: A with the $2,000 dated 2027-01-20 instead. A January 16-31
    // payment is no longer refused for a farmer (section 6654(i)(1)(D) puts
    // the March 1 rule in place of the January 31 rule).
    //   line 12 = 10,000.00 (the January 20 payment is after January 15)
    //   line 13 = 15,500.00 - 10,000.00 = 5,500.00
    //   line 16 = 2,000 × 5 / 365 × 7% = 1.917... → 1.92 (January 15 to 20)
    //           + 3,500 × 90 / 365 × 7% = 60.410... → 60.41
    //           = 62.33; Form 1040 line 38 = $62
    let mut d = farm_2210_f(300_000, 10_000);
    dated(&mut d, &[(20270120, 200_000)]);
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_f_line_12"), 1_000_000);
    assert_eq!(cents(&r, "form_2210_f_line_13"), 550_000);
    assert_eq!(cents(&r, "form_2210_f_line_16"), 6_233);
    assert_eq!(return_line(&r, "38"), 6_200);
    // The same payment on a return that is not a farmer's is still refused by
    // name (Form 2210's January 31 rule), see
    // column_d_january_31_payment_is_refused_by_name.

    // Filed and paid by March 1, 2027: lines 1-13 as above, line 16 is zero
    // with the March 1 explanation, and line 38 is $0.
    d.payments.as_mut().unwrap().farmer_paid_by_march = true;
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_f_line_13"), 550_000);
    assert_eq!(cents(&r, "form_2210_f_line_16"), 0);
    let line = r.lines.iter().find(|l| l.id == "form_2210_f_line_16").unwrap();
    assert_eq!(line.explanation.key, "EXPLAIN_FORM_2210_NO_PENALTY");
    assert_eq!(return_line(&r, "38"), 0);
}

#[test]
fn return_one_dollar_short_of_the_test_is_routed_to_form_2210() {
    // Fixture C: A with the farm's line 9 $299,999 and the 2025 answer No.
    // Farming 299,999; gross 150,000 + 299,999 = 449,999; 3 × 299,999 =
    // 899,997 < 2 × 449,999 = 899,998: the test is not met, so Form 2210 with
    // the same $24,734 tax.
    //   line 5 = 90% × 24,734 = 22,260.60
    //   line 8 = 110% × 15,500 = 17,050.00 (2025 AGI over $150,000)
    //   line 9 = 17,050.00; line 10 = 4,262.50 in each column
    //   line 11: (a) 2,500 (b) 2,500 (c) 2,500 (d) 2,000 (Jan 10) + 2,500
    //   = 4,500 (withholding $10,000 is $2,500 on each due date)
    //   line 17: (a) 1,762.50 (b) 3,525.00 (c) 4,262.50 (d) 4,262.50
    //   penalty worksheet (portion, from → to, rate periods, cents):
    //     1,762.50 Apr 15 → Jun 15   61 d × 6%                          = 17.67
    //     2,500.00 Jun 15 → Sep 15   15 d × 6% + 77 d × 7%   = 6.16 + 36.92 = 43.08
    //     1,025.00 Jun 15 → Jan 10   15 d 6%, 92 d 7%, 92 d 7%, 10 d 7%
    //                                = 2.53 + 18.08 + 18.08 + 1.97 = 40.66
    //       975.00 Sep 15 → Jan 10   15 d 7%, 92 d 7%, 10 d 7%
    //                                = 2.80 + 17.20 + 1.87 = 21.87
    //     2,500.00 Sep 15 → Jan 15   15 d 7%, 92 d 7%, 15 d 7%
    //                                = 7.19 + 44.11 + 7.19 = 58.49
    //       787.50 Sep 15 → Apr 15   15 d 7%, 92 d 7%, 105 d 7%
    //                                = 2.27 + 13.89 + 15.86 = 32.02
    //     4,262.50 Jan 15 → Apr 15   90 d 7%                            = 73.57
    //   line 19 = $287.36; Form 1040 line 38 = $287
    let mut d = farm_2210_f(299_999, 10_000);
    dated(&mut d, &[(20270110, 200_000)]);
    let r = supported(&d);
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form_2210_f_")));
    assert_eq!(cents(&r, "form_2210_line_5"), 2_226_060);
    assert_eq!(cents(&r, "form_2210_line_9"), 1_705_000);
    assert_eq!(cents(&r, "form_2210_line_19"), 28_736);
    assert_eq!(return_line(&r, "38"), 28_700);
    assert_eq!(line_38_explanation(&r), "EXPLAIN_1040_38_2");

    // C2: the 2025 answer Yes routes to Form 2210-F, the A lines and $60.41
    // (the route follows either year).
    answers(&mut d, Answer::Yes, FarmerNextYear::No);
    let r = supported(&d);
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form_2210_line_")));
    assert_eq!(cents(&r, "form_2210_f_line_7"), 1_648_933);
    assert_eq!(cents(&r, "form_2210_f_line_11"), 1_550_000);
    assert_eq!(cents(&r, "form_2210_f_line_16"), 6_041);
}

#[test]
fn fishing_income_on_a_joint_return_figures_form_2210_f() {
    // Fixture D: joint; the taxpayer's W-2 $90,000 with $2,000 withheld; the
    // spouse's break-even Schedule C with $200,000 on line 7, marked
    // commercial fishing. 2025 joint tax $5,000, AGI $95,000. Estimated
    // payment $1,000 on 2027-02-01.
    //   test: fishing 200,000; gross 90,000 + 200,000 = 290,000 (both
    //   spouses); 3 × 200,000 = 600,000 ≥ 2 × 290,000 = 580,000: met
    //   taxable = 90,000 - 32,200 = 57,800; Tax Table midpoint 57,825:
    //   2,480 + 12% × 33,025 = 6,443
    //   line 6 = 6,443.00; line 7 = 4,295.33; line 8 = 2,000.00;
    //   line 9 = 4,443.00; line 10 = 5,000.00; line 11 = 4,295.33;
    //   line 12 = 2,000.00; line 13 = 2,295.33
    //   line 16 = 1,000 × 17 / 365 × 7% = 3.260... → 3.26 (Jan 15 → Feb 1)
    //           + 1,295.33 × 90 / 365 × 7% = 22.357... → 22.36
    //           = 25.62; Form 1040 line 38 = $26
    let build = |fishing: bool| {
        let mut d = complete(FederalStatus::Joint);
        d.income = vec![wage(1, Owner::Taxpayer, 9_000_000, 200_000)];
        d.next_id = 2;
        let mut business = income(break_even_fishing(200_000, fishing));
        business.owner = Owner::Spouse;
        add(&mut d, business);
        let pay = d.payments.as_mut().unwrap();
        pay.prior_year = Some(PriorYearTax {
            full_year: true,
            tax: 500_000,
            agi: Some(9_500_000),
            spouse_tax: 0,
        });
        answers(&mut d, Answer::No, FarmerNextYear::NotNeeded);
        dated(&mut d, &[(20270201, 100_000)]);
        d
    };
    let r = supported(&build(true));
    assert_eq!(cents(&r, "farming_fishing_gross_income_return_year"), 20_000_000);
    assert_eq!(cents(&r, "gross_income_return_year"), 29_000_000);
    for (line, expected) in [
        (6, 644_300),
        (7, 429_533),
        (8, 200_000),
        (9, 444_300),
        (10, 500_000),
        (11, 429_533),
        (12, 200_000),
        (13, 229_533),
        (16, 2_562),
    ] {
        assert_eq!(cents(&r, &alloc::format!("form_2210_f_line_{line}")), expected, "line {line}");
    }
    assert_eq!(return_line(&r, "38"), 2_600);

    // Not marked commercial fishing: no farming or fishing income, no test
    // lines, and Form 2210 with line 5 = 90% × 6,443 = 5,798.70 and line 9 =
    // the smaller of that and 100% of $5,000 (AGI not over $150,000).
    let mut d = build(false);
    answers(&mut d, Answer::No, FarmerNextYear::No);
    let r = supported(&d);
    assert!(r.lines.iter().all(|l| l.id != "gross_income_return_year"));
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form_2210_f_")));
    assert_eq!(cents(&r, "form_2210_line_5"), 579_870);
    assert_eq!(cents(&r, "form_2210_line_9"), 500_000);
}

#[test]
fn form_2210_f_boundaries_stop_at_the_form_lines() {
    // Fixture E (i): a 2025 tax of $0 on a full-year return: line 10 = 0, line
    // 11 = 0, so line 8 ($10,000) is at least line 11 and the form stops
    // (section 6654(e)(2)); line 38 is $0.
    let mut d = farm_2210_f(300_000, 10_000);
    d.payments.as_mut().unwrap().prior_year.as_mut().unwrap().tax = 0;
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_f_line_10"), 0);
    assert_eq!(cents(&r, "form_2210_f_line_11"), 0);
    assert!(r.lines.iter().all(|l| l.id != "form_2210_f_line_12"));
    let line = r.lines.iter().find(|l| l.id == "form_2210_f_line_16").unwrap();
    assert_eq!(line.amount, 0);
    assert_eq!(line.explanation.key, "EXPLAIN_FORM_2210_F_NO_PENALTY_11");
    assert_eq!(return_line(&r, "38"), 0);

    // (ii) Withholding $24,000: line 9 = 24,734 - 24,000 = 734.00, under $1,000.
    let d = farm_2210_f(300_000, 24_000);
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_f_line_9"), 73_400);
    assert!(r.lines.iter().all(|l| l.id != "form_2210_f_line_10"));
    let line = r.lines.iter().find(|l| l.id == "form_2210_f_line_16").unwrap();
    assert_eq!(line.explanation.key, "EXPLAIN_FORM_2210_F_NO_PENALTY_9");
    assert_eq!(line.amount, 0);

    // (iii) A 2025 return under 12 months: line 10 is blank, line 11 = line 7 =
    // 16,489.33; with $10,000 + $2,000 paid by January 15, line 13 =
    // 4,489.33 and line 16 = 4,489.33 × 90 / 365 × 7% = 77.49 (the amount a
    // 110% rule produced for fixture A).
    let mut d = farm_2210_f(300_000, 10_000);
    d.payments.as_mut().unwrap().prior_year.as_mut().unwrap().full_year = false;
    dated(&mut d, &[(20270110, 200_000)]);
    let r = supported(&d);
    assert!(r.lines.iter().all(|l| l.id != "form_2210_f_line_10"));
    assert_eq!(cents(&r, "form_2210_f_line_11"), 1_648_933);
    assert_eq!(cents(&r, "form_2210_f_line_13"), 448_933);
    assert_eq!(cents(&r, "form_2210_f_line_16"), 7_749);
}

#[test]
fn stored_box_c_and_d_answers_are_not_used_on_a_farmer_return() {
    // Fixture F (ii): A with a stored annualized election (no periods) and a
    // stored box D election (no periods) on a return that meets the 2026 test.
    // Form 2210-F has neither box, so neither refuses the estimate: A's lines
    // and $60.41, one note linked to Payments, the result state unchanged, and
    // line 38 explained by Form 2210-F line 16.
    let mut d = farm_2210_f(300_000, 10_000);
    dated(&mut d, &[(20270110, 200_000)]);
    let pay = d.payments.as_mut().unwrap();
    pay.annualized = true;
    pay.withholding_by_date = true;
    let r = supported(&d);
    assert_eq!(cents(&r, "form_2210_f_line_11"), 1_550_000);
    assert_eq!(cents(&r, "form_2210_f_line_13"), 350_000);
    assert_eq!(cents(&r, "form_2210_f_line_16"), 6_041);
    assert!(r.lines.iter().all(|l| !l.id.starts_with("form_2210_line_")));
    let notes: alloc::vec::Vec<_> = r
        .review_notes
        .iter()
        .filter(|n| n.display.key == "NOTE_2210F_PERIOD_ANSWERS_NOT_USED")
        .collect();
    assert_eq!(notes.len(), 1, "{:?}", r.review_notes);
    assert_eq!(notes[0].context, Some(IssueContext::Section(TaxSection::Payments)));
    assert_eq!(line_38_explanation(&r), "EXPLAIN_1040_38_3");
    // Without the stored answers there is no note.
    let mut d = farm_2210_f(300_000, 10_000);
    dated(&mut d, &[(20270110, 200_000)]);
    assert!(!noted(&supported(&d), "NOTE_2210F_PERIOD_ANSWERS_NOT_USED"));
    // The same stored answers on a return that is not a farmer's still use
    // Form 2210 (box C asks for its periods).
    let mut d = farm_2210_f(299_999, 10_000);
    d.payments.as_mut().unwrap().annualized = true;
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    assert!(r.issues.iter().any(|i| i.message.contains("box C")));
}

#[test]
fn farmer_waiver_requests_use_form_2210_f_box_a() {
    // Form 2210-F has one waiver box, A, for both kinds of request: the note
    // names it, a partial request adds the requested amount, and line 16 is
    // the penalty less the amount waived (audit D2210F-1).
    let mut d = farm_2210_f(300_000, 10_000);
    dated(&mut d, &[(20270110, 200_000)]);
    d.payments.as_mut().unwrap().waiver = Some(PenaltyWaiver::Partial {
        reason: PenaltyWaiverReason::CasualtyOrDisasterOrUnusual,
        explanation: "Barn fire in November".into(),
        amount: 2_000,
    });
    let r = supported(&d);
    assert!(noted(&r, "NOTE_2210F_WAIVER_REQUESTED"), "{r:?}");
    assert!(r.review_notes.iter().any(|n| n.contains("Form 2210-F Part I box A waiver requested")
        && n.contains("Barn fire in November")
        && n.contains("the IRS decides the waiver")));
    assert_eq!(cents(&r, "form_2210_f_waiver_requested_amount"), 2_000);
    assert_eq!(cents(&r, "form_2210_f_line_16"), 4_041);
    assert_eq!(cents(&r, "form_2210_f_penalty_before_waiver"), 6_041);
    assert_eq!(return_line(&r, "38"), 4_000);
    assert!(r.lines.iter().all(|l| l.id != "form_2210_waiver_requested_amount"));

    // A full request gives zero; a request above the penalty is capped.
    for waiver in [
        PenaltyWaiver::Full {
            reason: PenaltyWaiverReason::CasualtyOrDisasterOrUnusual,
            explanation: "Barn fire in November".into(),
        },
        PenaltyWaiver::Partial {
            reason: PenaltyWaiverReason::CasualtyOrDisasterOrUnusual,
            explanation: "Barn fire in November".into(),
            amount: 100_000,
        },
    ] {
        d.payments.as_mut().unwrap().waiver = Some(waiver);
        let r = supported(&d);
        assert_eq!(cents(&r, "form_2210_f_line_16"), 0);
        assert_eq!(cents(&r, "form_2210_f_penalty_before_waiver"), 6_041);
        assert_eq!(return_line(&r, "38"), 0);
    }
}

// A blank 2025 AGI (PriorYearTax.agi None) takes the conservative path: line 8
// uses 110% as if 2025 AGI were over the threshold, with one note.
fn prior_agi_example(agi: Option<i64>) -> FederalResult {
    let mut d = base(400_000, 800_000, 0);
    d.payments.as_mut().unwrap().prior_year.as_mut().unwrap().agi = agi;
    dated(&mut d, &[(20260610, 150_000)]);
    supported(&d)
}

#[test]
fn blank_prior_agi_assumes_110_percent_with_one_note() {
    // Line 4 $9,018, line 5 90% = $8,116.20, 2025 tax $8,000. Blank: line 8 =
    // 110% x $8,000 = $8,800; line 9 = smaller of $8,116.20 and $8,800 =
    // $8,116.20, so each installment is 25% = $2,029.05 (not $2,000).
    let r = prior_agi_example(None);
    assert_eq!(cents(&r, "form_2210_line_8"), 880_000);
    assert_eq!(cents(&r, "form_2210_line_9"), 811_620);
    assert_eq!(cents(&r, "form_2210_line_10_a"), 202_905);
    let notes: alloc::vec::Vec<_> =
        r.review_notes.iter().filter(|n| n.display.key == "WARN_PRIOR_AGI_ASSUMED_HIGH").collect();
    assert_eq!(notes.len(), 1, "{:?}", r.review_notes);
    assert_eq!(notes[0].context, Some(IssueContext::Section(TaxSection::Payments)));
    // The same answer as an AGI above $150,000, without the note.
    let above = prior_agi_example(Some(16_000_000));
    assert_eq!(cents(&above, "form_2210_line_19"), cents(&r, "form_2210_line_19"));
    assert!(!noted(&above, "WARN_PRIOR_AGI_ASSUMED_HIGH"));
}

#[test]
fn entering_prior_agi_at_or_under_the_threshold_removes_the_note_and_lowers_the_penalty() {
    // Entering $100,000 (not over $150,000): line 8 = 100% x $8,000 = $8,000,
    // line 9 = $8,000, installments $2,000, penalty $96.07 (the instructions
    // example above) against about $101 with the 110% assumption.
    let blank = prior_agi_example(None);
    let entered = prior_agi_example(Some(10_000_000));
    assert_eq!(cents(&entered, "form_2210_line_8"), 800_000);
    assert_eq!(cents(&entered, "form_2210_line_19"), 9_607);
    assert!(!noted(&entered, "WARN_PRIOR_AGI_ASSUMED_HIGH"));
    assert!(cents(&blank, "form_2210_line_19") > 9_607);
    assert!(cents(&blank, "form_2210_line_19") < 10_500);
    // Exactly $150,000 is not over the threshold.
    let edge = prior_agi_example(Some(15_000_000));
    assert_eq!(cents(&edge, "form_2210_line_8"), 800_000);
    assert!(!noted(&edge, "WARN_PRIOR_AGI_ASSUMED_HIGH"));
}

#[test]
fn blank_prior_agi_needs_no_note_for_a_short_year_or_a_farmer() {
    // A short 2025 year leaves line 8 blank, so the field is unused.
    let mut d = base(400_000, 800_000, 0);
    let prior = d.payments.as_mut().unwrap().prior_year.as_mut().unwrap();
    prior.agi = None;
    prior.full_year = false;
    dated(&mut d, &[(20260610, 150_000)]);
    assert!(!noted(&supported(&d), "WARN_PRIOR_AGI_ASSUMED_HIGH"));
    // A farmer's Form 2210-F never reads it: same result, no note.
    let with = supported(&farm_2210_f(300_000, 10_000));
    let mut d = farm_2210_f(300_000, 10_000);
    d.payments.as_mut().unwrap().prior_year.as_mut().unwrap().agi = None;
    let blank = supported(&d);
    assert!(!noted(&blank, "WARN_PRIOR_AGI_ASSUMED_HIGH"));
    assert_eq!(cents(&blank, "form_2210_f_line_16"), cents(&with, "form_2210_f_line_16"));
    assert_eq!(cents(&blank, "form_2210_f_line_10"), cents(&with, "form_2210_f_line_10"));
}

#[test]
fn applied_credit_rounds_once_and_conserves_the_finalized_refund() {
    // 2026 tax $9,018; $11,918 withheld gives exactly $2,900 capacity.
    // D-2026-10-17-05 D2-R: 49/50/51 cents must never independently
    // round the remaining cash and thereby create an extra dollar.
    for (intent, reported, cash) in [
        (0, 0, 290_000),
        (100_049, 100_000, 190_000),
        (100_050, 100_100, 189_900),
        (100_051, 100_100, 189_900),
        (290_000, 290_000, 0),
    ] {
        let mut d = base(1_191_800, 800_000, 10_000_000);
        let pay = d.payments.as_mut().unwrap();
        pay.credit_applied_next_year = intent;
        pay.credit_recipient = Some(OverpaymentRecipient::TaxpayerAccount);
        let r = supported(&d);
        assert_eq!(r.credit_capacity, Some(290_000));
        let line36 = r.return_lines.iter().find(|line| {
            line.form_line
                .as_ref()
                .is_some_and(|form| form.form == TaxForm::Form1040 && form.line == "36")
        });
        assert_eq!(line36.map(|line| line.amount), Some(reported));
        assert_eq!(r.refund, Some(cash));
        assert_eq!(return_line(&r, "35a") + reported + return_line(&r, "38"), 290_000);
        assert_eq!(d.payments.as_ref().unwrap().credit_applied_next_year, intent);
        let rounded =
            r.review_notes.iter().any(|n| n.display.key == "NOTE_APPLIED_CREDIT_ROUNDING");
        assert_eq!(rounded, intent != reported);
    }
}

#[test]
fn applied_credit_uses_penalty_capacity_and_does_not_clamp_retained_intent() {
    let mut d = base(0, 800_000, 10_000_000);
    dated(&mut d, &[(20270110, 1_000_000)]);
    let pay = d.payments.as_mut().unwrap();
    pay.credit_applied_next_year = 75_800;
    pay.credit_recipient = Some(OverpaymentRecipient::TaxpayerAccount);
    // Existing hand calculation above: line34 $982, penalty $224.
    let r = supported(&d);
    assert_eq!((r.credit_capacity, r.refund), (Some(75_800), Some(0)));
    assert_eq!(return_line(&r, "36"), 75_800);
    let order: Vec<_> = crate::draft_return(&r)
        .into_iter()
        .filter_map(|row| row.form_line)
        .filter(|line| {
            line.form == TaxForm::Form1040
                && matches!(line.line.as_str(), "35a" | "36" | "37" | "38")
        })
        .map(|line| line.line)
        .collect();
    assert_eq!(order, ["35a", "36", "37", "38"]);
    assert_eq!(return_line(&r, "35a") + return_line(&r, "36") + return_line(&r, "38"), 98_200);
    d.payments.as_mut().unwrap().credit_applied_next_year = 75_801;
    let invalid = estimate(&d);
    assert_ne!(invalid.state, ResultState::EstimatedForSupportedScope);
    assert!(invalid.issues.iter().any(|i| i.display.key == "ISSUE_APPLIED_CREDIT_CAPACITY"));
    assert_eq!(d.payments.as_ref().unwrap().credit_applied_next_year, 75_801);
    assert!(!invalid.review_notes.iter().any(|n| n.display.key == "NOTE_APPLIED_CREDIT_PLAN"));
    // $9,100 still covers the $8,000 prior-year required payment, leaving
    // the same $224 penalty, greater than this return's $82 overpayment.
    d.payments.as_mut().unwrap().credit_applied_next_year = 0;
    dated(&mut d, &[(20270110, 910_000)]);
    let r = supported(&d);
    assert_eq!((return_line(&r, "35a"), return_line(&r, "36")), (0, 0));
    assert_eq!((return_line(&r, "37"), return_line(&r, "38")), (14_200, 22_400));
}

#[test]
fn planned_credit_rolls_through_estimated_installments_without_becoming_a_payment() {
    let mut d = next_year(
        400_000,
        NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year: FarmerNextYear::No },
    );
    let pay = d.payments.as_mut().unwrap();
    pay.extension = 1_000_000;
    pay.extension_payer = Some(PaymentPayer::Taxpayer);
    pay.credit_applied_next_year = 250_000;
    pay.credit_recipient = Some(OverpaymentRecipient::TaxpayerAccount);
    let r = supported(&d);
    // Existing worksheet: $1,028 each quarter. $2,500 covers the first
    // two quarters and $444 of the third; remaining cash $584, then $1,028.
    let cash: Vec<_> =
        (1..=4).map(|i| cents(&r, &alloc::format!("form_1040_es_planned_cash_{i}"))).collect();
    assert_eq!(cash, [0, 0, 58_400, 102_800]);
    assert_eq!(r.payments, Some(1_400_000));
    assert_eq!(return_line(&r, "26"), 0);
    assert_eq!(es_line(&r, "15"), Some(1_028));
    assert!(!r.review_notes.iter().any(|n| n.display.key == "NOTE_1040ES_QUARTERLY_INSTALLMENT"));
    // Exercise actual native Review conversion, including registered labels
    // for every forecast cash row, rather than only the raw owner result.
    let (mut session, page) = crate::tests::cross::interview::at_the_payments_page(d.clone());
    let value = crate::tests::cross::interview::payments_value(&page).0.clone();
    let Reply::Response(P::Review { result, .. }) =
        crate::tests::cross::interview::submit(&mut session, value)
    else {
        panic!("native forecast review")
    };
    assert_eq!(
        result
            .breakdown
            .iter()
            .filter(|row| status_text(&row.label).contains("remaining cash"))
            .count(),
        4
    );
    // A spouse-account intent does not reduce a separate taxpayer forecast.
    d.setup.as_mut().unwrap().status = filing_choice(FederalStatus::Separate);
    d.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::Together };
    d.payments.as_mut().unwrap().joint = false;
    d.payments.as_mut().unwrap().credit_recipient = Some(OverpaymentRecipient::SpouseAccount);
    let separate = supported(&d);
    assert!(!separate.lines.iter().any(|l| l.id.starts_with("form_1040_es_planned_cash_")));
}
