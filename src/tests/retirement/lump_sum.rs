// Form 4972 (lump_sum.rs): fixtures F1-F9 of the unit map, each worked by hand from the 2026 draft
// Form 4972, with the arithmetic in the comments. Amounts in the comments are dollars; the records
// carry cents. F1 and F2 reproduce Publication 575 (2024) Examples 1 and 2.
use super::*;
use crate::{Form4972Election, Form4972Participant, LumpSumEvent, LumpSumMethod};

const SEVEN_A: u32 = crate::DistributionCodes::CODE_7.0 | crate::DistributionCodes::CODE_A.0;
const FOUR_A: u32 = crate::DistributionCodes::CODE_4.0 | crate::DistributionCodes::CODE_A.0;

// A filer born on `birth` with no other income.
fn bare(birth: u32) -> FederalDraft {
    let mut d = empty_return(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = birth;
    d
}
// The election's conditions met: entire balance paid from 2026-03-01, no earlier payment.
fn election(
    participant: Form4972Participant,
    method: LumpSumMethod,
) -> (Form4972Election, crate::R1099) {
    (
        Form4972Election {
            participant,
            method,
            include_nua: false,
            first_payment: 20260301,
            last_prior_payment: None,
            entire_balance: true,
            confirmed: true,
        },
        crate::tests::cross::interview::r1099(
            0,
            DistributionCode::Normal { what_happened: Disposition::Kept },
            false,
        )
        .into_source()
        .form,
    )
}
fn own() -> Form4972Participant {
    Form4972Participant::Own { event: LumpSumEvent::Age }
}
fn decedent(born: u32, died: u32, estate_tax: i64) -> Form4972Participant {
    Form4972Participant::Decedent { death_benefit: 0, born, died, estate_tax }
}
// One 1099-R in dollars: box 1 gross, box 2a taxable, box 4 withholding.
fn record(
    gross: i64,
    taxable: i64,
    withheld: i64,
    data: (Form4972Election, crate::R1099),
) -> FederalIncomeEntry {
    let (data, boxes) = data;
    let box7 = match data.participant {
        Form4972Participant::Own { .. } => SEVEN_A,
        Form4972Participant::Decedent { .. } => FOUR_A,
    };
    income(FederalIncome::Retirement {
        data: retirement_source(FederalRetirement {
            form_boxes: Default::default(),
            trump: false,
            box_2a: taxable * 100,
            box_3: boxes.box_3.unwrap_or(0),
            box_6: boxes.box_6.unwrap_or(0),
            box_8a: boxes.box_8a.unwrap_or(0),
            box_8b_percent: boxes.box_8b_percent,
            box_9a_percent: boxes.box_9a_percent,
            ira: (RetirementKind::Pension).ira()
                && (box7) & crate::DistributionCodes::CODE_N.0 == 0,
            box_9b: None,
            box_10: 0,
            box_10_basis: None,
            disability: DisabilityIncome::None,
            disability_wages: crate::DisabilityWages::None,
            kind: RetirementKind::Pension,
            gross: gross * 100,
            taxable: Some(RetirementTaxable::Payer),
            withholding: withheld * 100,
            box7,
            ordinary_distribution: Some(RetirementTreatment::LumpSum { data }),
            disaster: DisasterDistribution::No,
            after_death: None,
        }),
    })
}
// A line of the first record's Form 4972, in dollars.
fn form(r: &FederalResult, id: i64, line: u32) -> i64 {
    value(r, &alloc::format!("lump_sum_{id}_line_{line}"))
}
fn has_line(r: &FederalResult, id: i64, line: u32) -> bool {
    let wanted = alloc::format!("lump_sum_{id}_line_{line}");
    r.lines.iter().any(|l| l.id == wanted)
}
fn added(d: &mut FederalDraft, entry: FederalIncomeEntry) -> i64 {
    add(d, entry);
    d.income.last().unwrap().id
}

#[test]
fn lump_sum_pub_575_example_1_parts_ii_and_iii() {
    // Publication 575 (2024) Example 1 (Robert): box 1 175,000, 2a 150,000, 3 10,000, 4 30,000,
    // Parts II and III, no annuity, no estate tax. Participant born 1935-06-01.
    //   6  = 10,000                      7  = 20% x 10,000 = 2,000
    //   8  = 150,000 - 10,000 = 140,000  10 = 140,000   11 = 0   12 = 140,000 (>= 70,000: 16 = 0)
    //   17 = 19 = 140,000                23 = 14,000
    //   24 = 2,160.30 + 23% x (14,000 - 13,710) = 2,227.00
    //   25 = 10 x 2,227.00 = 22,270      29 = 22,270
    //   30 = 2,000 + 22,270 = 24,270     (Publication 575: "includes the tax of $24,270")
    let mut d = bare(19350601);
    let mut e = election(own(), LumpSumMethod::Both);
    e.1.box_3 = Some(10_000 * 100);
    let id = added(&mut d, record(175_000, 150_000, 30_000, e));
    let r = supported(&d);
    assert_eq!(form(&r, id, 6), 10_000);
    assert_eq!(form(&r, id, 7), 2_000);
    assert_eq!(form(&r, id, 8), 140_000);
    assert_eq!(form(&r, id, 12), 140_000);
    assert_eq!(form(&r, id, 16), 0);
    assert_eq!(form(&r, id, 19), 140_000);
    assert_eq!(form(&r, id, 25), 22_270);
    assert_eq!(form(&r, id, 29), 22_270);
    assert_eq!(form(&r, id, 30), 24_270);
    assert!(!has_line(&r, id, 11));
    // Nothing from the record reaches Form 1040 lines 5a and 5b or AGI.
    assert_eq!(return_line(&r, "5a"), 0);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(value(&r, "agi"), 0);
    // Line 16 box 2 carries the tax; the box 4 withholding still counts on line 25b.
    assert_eq!(value(&r, "lump_sum_tax"), 24_270);
    assert_eq!(return_line(&r, "16"), 2_427_000);
    assert_eq!(return_line(&r, "25b"), 3_000_000);
    assert_eq!(
        printed_line(&r, &alloc::format!("lump_sum_{id}_line_30")),
        (crate::TaxForm::Form4972, "30".into())
    );
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn lump_sum_pub_575_example_2_annuity_contract() {
    // Publication 575 (2024) Example 2 (Mary): 2a 160,000, 8a 10,000, Part III only.
    //   8 = 10 = 160,000   11 = 10,000   12 = 170,000 (16 = 0)   17 = 19 = 170,000
    //   20 = 10,000 / 170,000 (line 21 = 0 x that = 0)           22 = 10,000 - 0 = 10,000
    //   23 = 17,000   24 = 2,160.30 + 23% x 3,290 = 2,917.00     25 = 29,170
    //   26 = 1,000    27 = 11% x 1,000 = 110.00                  28 = 1,100
    //   29 = 29,170 - 1,100 = 28,070    30 = 28,070   (Publication 575: $28,070)
    // The publication's printed line 10 of 140,000 contradicts its own lines 8 and 12; line 10 is
    // 160,000 here.
    let mut d = bare(19350601);
    let mut e = election(own(), LumpSumMethod::TenYear);
    e.1.box_8a = Some(10_000 * 100);
    let id = added(&mut d, record(160_000, 160_000, 0, e));
    let r = supported(&d);
    assert_eq!(form(&r, id, 8), 160_000);
    assert_eq!(form(&r, id, 11), 10_000);
    assert_eq!(form(&r, id, 12), 170_000);
    assert_eq!(form(&r, id, 16), 0);
    assert_eq!(form(&r, id, 21), 0);
    assert_eq!(form(&r, id, 22), 10_000);
    assert_eq!(form(&r, id, 25), 29_170);
    assert_eq!(form(&r, id, 28), 1_100);
    assert_eq!(form(&r, id, 29), 28_070);
    assert_eq!(form(&r, id, 30), 28_070);
    assert!(!has_line(&r, id, 6));
}

#[test]
fn lump_sum_beneficiary_minimum_allowance_with_an_annuity_share() {
    // A beneficiary born 1962-04-04 (age is irrelevant) of a participant born 1934-03-10 who died
    // 2026-02-01; first payment 2026-04-01; 2a 30,000; 8a 5,000; Part III only.
    //   8 = 10 = 30,000   11 = 5,000   12 = 35,000
    //   13 = min(50% x 35,000, 10,000) = 10,000   14 = 35,000 - 20,000 = 15,000
    //   15 = 20% x 15,000 = 3,000   16 = 10,000 - 3,000 = 7,000   17 = 19 = 28,000
    //   21 = 7,000 x 5,000 / 35,000 = 1,000   22 = 5,000 - 1,000 = 4,000
    //   25 = 10 x (260.50 + 14% x (2,800 - 2,270)) = 10 x 334.70 = 3,347
    //   28 = 10 x (11% x 400) = 440   29 = 3,347 - 440 = 2,907   30 = 2,907
    // The form's three-place 0.143 would give line 21 = 1,001 and 2,907.11; the exact ratio gives
    // 2,907.00, inside "at least three places".
    let mut d = bare(19620404);
    let mut e = election(decedent(19340310, 20260201, 0), LumpSumMethod::TenYear);
    e.0.first_payment = 20260401;
    e.1.box_8a = Some(5_000 * 100);
    let id = added(&mut d, record(30_000, 30_000, 0, e));
    let r = supported(&d);
    assert_eq!(form(&r, id, 12), 35_000);
    assert_eq!(form(&r, id, 16), 7_000);
    assert_eq!(form(&r, id, 17), 28_000);
    assert_eq!(form(&r, id, 21), 1_000);
    assert_eq!(form(&r, id, 22), 4_000);
    assert_eq!(form(&r, id, 25), 3_347);
    assert_eq!(form(&r, id, 28), 440);
    assert_eq!(form(&r, id, 29), 2_907);
    assert_eq!(form(&r, id, 30), 2_907);
    assert_eq!(return_line(&r, "5b"), 0);
}

#[test]
fn lump_sum_part_ii_only_leaves_the_ordinary_part_on_line_5b() {
    // 2a 50,000 with a box 3 of 8,000; only the 20% capital gain rate (Part II).
    //   6 = 8,000   7 = 1,600   30 = 1,600
    //   The ordinary part 50,000 - 8,000 = 42,000 is on lines 5a and 5b and in AGI.
    //   Line 16 = the regular tax on the 42,000 (the unchanged tax machinery) + 1,600.
    let mut d = bare(19350601);
    let mut e = election(own(), LumpSumMethod::CapitalGain);
    e.1.box_3 = Some(8_000 * 100);
    let id = added(&mut d, record(50_000, 50_000, 0, e));
    let r = supported(&d);
    assert_eq!(form(&r, id, 6), 8_000);
    assert_eq!(form(&r, id, 7), 1_600);
    assert_eq!(form(&r, id, 30), 1_600);
    assert!(!has_line(&r, id, 8));
    assert_eq!(return_line(&r, "5a"), 4_200_000);
    assert_eq!(return_line(&r, "5b"), 4_200_000);
    assert_eq!(value(&r, "agi"), 42_000);
    assert_eq!(return_line(&r, "16"), value(&r, "regular_income_tax") * 100 + 160_000);
    assert!(value(&r, "regular_income_tax") > 0);
}

#[test]
fn lump_sum_tax_stays_out_of_the_alternative_minimum_tax_comparison() {
    // Form 6251 line 10 subtracts the Form 4972 tax from Form 1040 line 16 (and Form 1116 line 20
    // does too). Private-activity bond interest of 200,000 makes the tentative minimum tax far
    // exceed the regular tax; the amount added to the regular tax by Form 4972 must not reduce
    // the alternative minimum tax. The same return without the lump sum has the same AMT.
    let interest = income(FederalIncome::Interest {
        data: interest_source(native_exempt_interest(20_000_000, 20_000_000)),
    });
    let mut without = bare(19350601);
    add(&mut without, interest.clone());
    let mut with = without.clone();
    let mut e = election(own(), LumpSumMethod::Both);
    e.1.box_3 = Some(10_000 * 100);
    add(&mut with, record(175_000, 150_000, 0, e));
    let (plain, lump) = (supported(&without), supported(&with));
    assert!(value(&plain, "alternative_minimum_tax") > 10_000);
    assert_eq!(value(&lump, "alternative_minimum_tax"), value(&plain, "alternative_minimum_tax"));
    assert_eq!(value(&lump, "lump_sum_tax"), 24_270);
}

#[test]
fn lump_sum_multiple_recipients_scale_by_box_9a() {
    // A beneficiary of a participant born 1933-05-05 who died 2026-03-01; 2a 40,000; box 9a 40%.
    //   8 = 40,000 / 40% = 100,000   12 = 100,000 (16 = 0)   19 = 100,000
    //   25 = 10 x (1,297.70 + 18% x (10,000 - 9,170)) = 10 x 1,447.10 = 14,471
    //   29 = 14,471 x 40% = 5,788.40   30 = 5,788
    let mut d = bare(19620404);
    let mut e = election(decedent(19330505, 20260301, 0), LumpSumMethod::TenYear);
    e.1.box_9a_percent = Some(40.0);
    let id = added(&mut d, record(40_000, 40_000, 0, e.clone()));
    let r = supported(&d);
    assert_eq!(form(&r, id, 8), 100_000);
    assert_eq!(form(&r, id, 25), 14_471);
    assert_eq!(value(&r, "lump_sum_tax"), 5_788);
    assert_eq!(line_amount(&r, &alloc::format!("lump_sum_{id}_line_29")), 578_840);
    // An estate tax share of 1,000 with box 9a 40%: line 18 = 1,000 / 40% = 2,500 (the share is
    // restated to the whole distribution as lines 8 and 11 are), line 19 = 100,000 - 2,500 =
    // 97,500, line 25 = 10 x (1,297.70 + 18% x (9,750 - 9,170)) = 10 x 1,402.10 = 14,021, line 29 =
    // 14,021 x 40% = 5,608.40, line 30 = 5,608.
    let mut d = bare(19620404);
    e.0.participant = decedent(19330505, 20260301, 1_000 * 100);
    let id = added(&mut d, record(40_000, 40_000, 0, e));
    let r = supported(&d);
    assert_eq!(form(&r, id, 18), 2_500);
    assert_eq!(form(&r, id, 19), 97_500);
    assert_eq!(form(&r, id, 25), 14_021);
    assert_eq!(value(&r, "lump_sum_tax"), 5_608);
}

#[test]
fn lump_sum_fractional_box_8b_share_converts_to_exact_hundredths() {
    // Box 9a 33.33% and box 8b 33.33% (3,333 hundredths) of a total distribution: 2a 20,000, 8a 3,000.
    //   8  = 20,000 / 0.3333 = 60,006.0006 -> 60,006.00 (cents 6,000,600)
    //   11 = 3,000 / 0.3333  =  9,000.9009 ->  9,000.90 (cents 900,090)
    //   12 = 69,006.90   13 = min(34,503.45, 10,000) = 10,000   14 = 49,006.90
    //   15 = 9,801.38    16 = 198.62      17 = 19 = 68,808.28
    //   21 = 198.62 x 9,000.90 / 69,006.90 = 25.91   22 = 9,000.90 - 25.91 = 8,974.99
    //   23 = 6,880.83 (the 16% bracket over 6,690): 24 = 900.90 + 16% x 190.83 = 931.43
    //   25 = 10 x 931.4432 = 9,314.32
    //   28 = 10 x 11% x 897.499 = 987.25   29 = 9,314.32 - 987.25 = 8,327.07
    //   x 33.33% = 2,775.41   30 = 2,775   (exact fractions give 2,775.41 as well)
    let mut d = bare(19620404);
    let mut e = election(decedent(19330505, 20260301, 0), LumpSumMethod::TenYear);
    e.1.box_9a_percent = Some(33.33);
    e.1.box_8b_percent = Some(33.33);
    e.1.box_8a = Some(3_000 * 100);
    let id = added(&mut d, record(23_000, 20_000, 0, e));
    let r = supported(&d);
    let cents = |line: u32| line_amount(&r, &alloc::format!("lump_sum_{id}_line_{line}"));
    assert_eq!(cents(8), 6_000_600);
    assert_eq!(cents(11), 900_090);
    assert_eq!(cents(12), 6_900_690);
    assert_eq!(cents(16), 19_862);
    assert_eq!(cents(21), 2_591);
    assert_eq!(cents(22), 897_499);
    assert_eq!(cents(25), 931_432);
    assert_eq!(cents(28), 98_725);
    assert_eq!(cents(29), 277_541);
    assert_eq!(form(&r, id, 30), 2_775);
    // The page's two-decimal percent converts once: 33.33 is exactly 3,333 hundredths and back.
    assert_eq!(crate::lump_sum::hundredths(Some(33.33)), Some(3_333));
    assert_eq!(crate::lump_sum::hundredths(Some(0.07)), Some(7));
    assert_eq!(crate::lump_sum::hundredths(Some(99.99)), Some(9_999));
    assert_eq!(crate::lump_sum::hundredths(None), None);
}

#[test]
fn lump_sum_net_unrealized_appreciation_worksheet() {
    // 2a 100,000, box 3 20,000, box 6 30,000 (not in 2a). Participant born 1935-06-01.
    // Both parts with the NUA included (NUA Worksheet): A = 20,000, B = 100,000, C = 0.2,
    //   D = 30,000, E = D x C = 6,000, F = D - E = 24,000, G = A + E = 26,000.
    //   6 = 26,000   7 = 5,200   8 = 100,000 - 20,000 + 24,000 = 104,000
    //   23 = 10,400 (the 18% bracket over 9,170): 24 = 1,297.70 + 18% x 1,230 = 1,519.10
    //   25 = 10 x 1,519.10 = 15,191
    //   30 = 5,200 + 15,191 = 20,391
    let build = |method, include_nua| {
        let mut d = bare(19350601);
        let mut e = election(own(), method);
        e.1.box_3 = Some(20_000 * 100);
        e.1.box_6 = Some(30_000 * 100);
        e.0.include_nua = include_nua;
        let id = added(&mut d, record(100_000, 100_000, 0, e));
        (supported(&d), id)
    };
    let (r, id) = build(LumpSumMethod::Both, true);
    assert_eq!(form(&r, id, 6), 26_000);
    assert_eq!(form(&r, id, 7), 5_200);
    assert_eq!(form(&r, id, 8), 104_000);
    assert_eq!(form(&r, id, 25), 15_191);
    assert_eq!(form(&r, id, 30), 20_391);
    // Part III only with the NUA: line 8 = 2a + box 6 = 130,000;
    //   25 = 10 x (1,706.30 + 20% x (13,000 - 11,440)) = 10 x 2,018.30 = 20,183 = 30.
    let (r, id) = build(LumpSumMethod::TenYear, true);
    assert_eq!(form(&r, id, 8), 130_000);
    assert_eq!(form(&r, id, 30), 20_183);
    // The NUA left out (box 6 kept untaxed): line 8 = 100,000 - 20,000 = 80,000, line 6 = 20,000;
    //   25 = 10 x (900.90 + 16% x (8,000 - 6,690)) = 10 x 1,110.50 = 11,105
    //   30 = 4,000 + 11,105 = 15,105
    let (r, id) = build(LumpSumMethod::Both, false);
    assert_eq!(form(&r, id, 6), 20_000);
    assert_eq!(form(&r, id, 8), 80_000);
    assert_eq!(form(&r, id, 30), 15_105);
    // Part II alone with the NUA: the ordinary part is 100,000 - 20,000 + F = 104,000 on line 5b.
    let (r, id) = build(LumpSumMethod::CapitalGain, true);
    assert_eq!(form(&r, id, 30), 5_200);
    assert_eq!(return_line(&r, "5b"), 10_400_000);
}

#[test]
fn lump_sum_estate_tax_capital_share_and_no_exclusion_without_an_answer() {
    // A beneficiary of a participant born 1935-01-01 who died 2026-05-01; 2a 60,000, box 3
    // 15,000, estate tax 4,000, Parts II and III. Death Benefit Worksheet: C = 15,000 / 60,000 =
    // 0.25, the capital-gain estate tax is 4,000 x 0.25 = 1,000.
    //   6 = 15,000 - 1,000 = 14,000   7 = 2,800
    //   8 = 60,000 - 15,000 = 45,000   12 = 45,000   13 = 10,000   14 = 25,000   15 = 5,000
    //   16 = 5,000   17 = 40,000   18 = 4,000 - 1,000 = 3,000   19 = 37,000
    //   25 = 10 x (260.50 + 14% x (3,700 - 2,270)) = 10 x 460.70 = 4,607
    //   30 = 2,800 + 4,607 = 7,407
    let build = |died: u32| {
        let mut d = bare(19620404);
        let mut e = election(decedent(19350101, died, 4_000 * 100), LumpSumMethod::Both);
        e.0.first_payment = 20260601;
        e.1.box_3 = Some(15_000 * 100);
        let id = added(&mut d, record(60_000, 60_000, 0, e));
        (supported(&d), id)
    };
    let (r, id) = build(20260501);
    assert_eq!(form(&r, id, 6), 14_000);
    assert_eq!(form(&r, id, 7), 2_800);
    assert_eq!(form(&r, id, 8), 45_000);
    assert_eq!(form(&r, id, 16), 5_000);
    assert_eq!(form(&r, id, 17), 40_000);
    assert_eq!(form(&r, id, 18), 3_000);
    assert_eq!(form(&r, id, 19), 37_000);
    assert_eq!(form(&r, id, 25), 4_607);
    assert_eq!(form(&r, id, 30), 7_407);
    // A participant who died 1996-08-20 (before August 21, 1996) with no exclusion entered: the
    // same lines (line 9 = 0), and no notice (the exclusion is the recipient's own answer).
    let (r, id) = build(19960820);
    assert_eq!(form(&r, id, 30), 7_407);
    assert!(!has_line(&r, id, 9));
}

#[test]
fn lump_sum_allowance_and_schedule_through_the_return() {
    // F8 through the whole return: a plain Part III distribution of 2a dollars (the unit tests in
    // lump_sum.rs hold every bracket boundary).
    //   20,000 -> 16 = 10,000, 19 = 10,000, 30 = 1,100
    //   50,000 -> 16 = 4,000,  19 = 46,000, 30 = 5,769 + 15% x 700 = 5,874
    //   70,000 -> 16 = 0,      19 = 70,000, 30 = 9,009 + 16% x 3,100 = 9,505
    for (taxable, allowance, averaged, tax) in [
        (20_000, 10_000, 10_000, 1_100),
        (50_000, 4_000, 46_000, 5_874),
        (70_000, 0, 70_000, 9_505),
    ] {
        let mut d = bare(19350601);
        let id =
            added(&mut d, record(taxable, taxable, 0, election(own(), LumpSumMethod::TenYear)));
        let r = supported(&d);
        assert_eq!(form(&r, id, 16), allowance);
        assert_eq!(form(&r, id, 19), averaged);
        assert_eq!(form(&r, id, 30), tax);
    }
}

#[test]
fn lump_sum_birth_date_boundary_is_day_exact() {
    // A participant attains age 50 the day before the birthday: born 1936-01-01 attained 50 on
    // 1985-12-31 (before 1986-01-01) and qualifies; born 1936-01-02 attained it on 1986-01-01 and
    // does not. Own plan (7A) tests the filer's People birth date.
    let build = |birth: u32| {
        let mut d = bare(birth);
        let id = added(&mut d, record(50_000, 50_000, 0, election(own(), LumpSumMethod::TenYear)));
        (supported(&d), id)
    };
    let (r, id) = build(19360101);
    assert_eq!(form(&r, id, 30), 5_874);
    assert_eq!(return_line(&r, "5b"), 0);
    assert!(!noted(&r, "NOTE_LUMP_SUM_PARTICIPANT_BORN_AFTER_1935"));
    // Born 1936-01-02: taxed with the other income (50,000 on lines 5a and 5b), one notice.
    let (r, id) = build(19360102);
    assert!(!has_line(&r, id, 30));
    assert_eq!(return_line(&r, "5a"), 5_000_000);
    assert_eq!(return_line(&r, "5b"), 5_000_000);
    assert!(noted(&r, "NOTE_LUMP_SUM_PARTICIPANT_BORN_AFTER_1935"));
    assert_eq!(value(&r, "agi"), 50_000);
    // The beneficiary case tests the entered participant birth date the same way (F3's record).
    let beneficiary = |born: u32| {
        let mut d = bare(19620404);
        let mut e = election(decedent(born, 20260201, 0), LumpSumMethod::TenYear);
        e.0.first_payment = 20260401;
        e.1.box_8a = Some(5_000 * 100);
        added(&mut d, record(30_000, 30_000, 0, e));
        supported(&d)
    };
    let r = beneficiary(19360101);
    assert_eq!(value(&r, "lump_sum_tax"), 2_907);
    // Born 1936-01-02: no Form 4972; the death exception treatment, 5b = 30,000.
    let r = beneficiary(19360102);
    assert!(noted(&r, "NOTE_LUMP_SUM_PARTICIPANT_BORN_AFTER_1935"));
    assert_eq!(return_line(&r, "5b"), 3_000_000);
    assert!(r.lines.iter().all(|l| l.id != "lump_sum_tax"));
}

#[test]
fn lump_sum_left_out_when_the_conditions_are_not_met() {
    // F1 with `confirmed` false: the distribution is taxed with the other income (5a 175,000,
    // 5b 150,000) and one notice names it; no Form 4972 lines.
    let mut d = bare(19350601);
    let mut e = election(own(), LumpSumMethod::Both);
    e.1.box_3 = Some(10_000 * 100);
    e.0.confirmed = false;
    let id = added(&mut d, record(175_000, 150_000, 30_000, e.clone()));
    let r = supported(&d);
    assert_eq!(return_line(&r, "5a"), 17_500_000);
    assert_eq!(return_line(&r, "5b"), 15_000_000);
    assert!(!has_line(&r, id, 30));
    assert_eq!(
        r.review_notes.iter().filter(|n| n.display.key == "WARN_LUMP_SUM_LEFT_OUT").count(),
        1
    );
    // The entire balance not paid in 2026 leaves it out as well.
    e.0.confirmed = true;
    e.0.entire_balance = false;
    let mut d = bare(19350601);
    added(&mut d, record(175_000, 150_000, 30_000, e.clone()));
    assert!(noted(&supported(&d), "WARN_LUMP_SUM_LEFT_OUT"));
    // An earlier-year payment after the event (a required distribution in 2023, after age 59 1/2)
    // prevents the Age event; a separation after it starts a new eligibility period.
    e.0.entire_balance = true;
    e.0.last_prior_payment = Some(20230601);
    let mut d = bare(19350601);
    added(&mut d, record(175_000, 150_000, 30_000, e.clone()));
    assert!(noted(&supported(&d), "WARN_LUMP_SUM_LEFT_OUT"));
    e.0.participant = Form4972Participant::Own {
        event: LumpSumEvent::Separation { date: 20240101, employee: true },
    };
    let mut d = bare(19350601);
    let id = added(&mut d, record(175_000, 150_000, 30_000, e.clone()));
    let r = supported(&d);
    assert!(!noted(&r, "WARN_LUMP_SUM_LEFT_OUT"));
    assert!(has_line(&r, id, 30));
    // The first payment before the separation does not follow the event.
    e.0.first_payment = 20260101;
    e.0.participant = Form4972Participant::Own {
        event: LumpSumEvent::Separation { date: 20260201, employee: true },
    };
    e.0.last_prior_payment = None;
    let mut d = bare(19350601);
    added(&mut d, record(175_000, 150_000, 30_000, e));
    assert!(noted(&supported(&d), "WARN_LUMP_SUM_LEFT_OUT"));
}

#[test]
fn lump_sum_records_of_one_participant_combine_or_are_refused() {
    let issues =
        |r: &FederalResult, key: &str| r.issues.iter().filter(|i| i.display.key == key).count();
    // Two records of the filer's plan, both Part III: one Form 4972 on the sum.
    //   2a 30,000 + 20,000 = 50,000 -> 8 = 50,000, 30 = 5,874 (as the 50,000 allowance case).
    let mut d = bare(19350601);
    let first = added(&mut d, record(30_000, 30_000, 0, election(own(), LumpSumMethod::TenYear)));
    added(&mut d, record(20_000, 20_000, 0, election(own(), LumpSumMethod::TenYear)));
    let r = supported(&d);
    assert_eq!(form(&r, first, 8), 50_000);
    assert_eq!(form(&r, first, 30), 5_874);
    assert_eq!(value(&r, "lump_sum_tax"), 5_874);
    // Different methods: blocking.
    let mut d2 = d.clone();
    let FederalIncome::Retirement { data } = &mut d2.income[1].income else { unreachable!() };
    edit_retirement(data).box_3 = 1_000 * 100;
    {
        let mut retirement_edit = edit_retirement(data);
        let Some(RetirementTreatment::LumpSum { data }) =
            &mut retirement_edit.ordinary_distribution
        else {
            unreachable!()
        };
        data.method = LumpSumMethod::Both;
    }
    let r = estimate(&d2);
    assert_eq!(r.state, ResultState::Unsupported);
    assert_eq!(issues(&r, "ISSUE_LUMP_SUM_TREATED_DIFFERENTLY"), 1);
    // One elected and one taxed with the other income: blocking.
    let mut d3 = d.clone();
    let FederalIncome::Retirement { data } = &mut d3.income[1].income else { unreachable!() };
    edit_retirement(data).ordinary_distribution = Some(RetirementTreatment::Ordinary);
    let r = estimate(&d3);
    assert_eq!(r.state, ResultState::Unsupported);
    assert_eq!(issues(&r, "ISSUE_LUMP_SUM_TREATED_DIFFERENTLY"), 1);
    // A combined group with a box 9a share: blocking.
    let mut d4 = d.clone();
    let FederalIncome::Retirement { data } = &mut d4.income[1].income else { unreachable!() };
    edit_retirement(data).box_9a_percent = Some(50.0);
    let r = estimate(&d4);
    assert_eq!(r.state, ResultState::Unsupported);
    assert_eq!(issues(&r, "ISSUE_LUMP_SUM_SHARED_AND_COMBINED"), 1);
}

#[test]
fn lump_sum_stored_shapes_are_checked() {
    // The stored record must be a qualified-plan Payer record with exactly 7A (own) or 4A
    // (decedent), amounts in range and box 3 within box 2a.
    let ok = {
        let mut d = bare(19350601);
        let mut e = election(own(), LumpSumMethod::Both);
        e.1.box_3 = Some(10_000 * 100);
        add(&mut d, record(175_000, 150_000, 0, e));
        d
    };
    assert_eq!(estimate(&ok).state, ResultState::EstimatedForSupportedScope);
    let wrong = |change: &dyn Fn(&mut FederalRetirement)| {
        let mut d = ok.clone();
        let FederalIncome::Retirement { data } = &mut d.income[0].income else { unreachable!() };
        change(&mut edit_retirement(data));
        assert_ne!(estimate(&d).state, ResultState::EstimatedForSupportedScope, "{d:?}");
    };
    wrong(&|v| v.box7 = crate::DistributionCodes::CODE_7.0);
    wrong(&|v| v.box7 = FOUR_A);
    wrong(&|v| v.kind = RetirementKind::Ira);
    wrong(&|v| v.taxable = Some(RetirementTaxable::IraBasis));
    wrong(&|v| {
        v.box_3 = 200_000 * 100;
    });
    wrong(&|v| {
        let Some(RetirementTreatment::LumpSum { data }) = &mut v.ordinary_distribution else {
            unreachable!()
        };
        data.first_payment = 20250101;
    });
}

// The age event reads the owner's People birth date through the one qualifying-event test
// (`lump_sum_conditions`), day-exact: born 1967-01-01 the 59th half-year is 2026-07-01, the first
// payment, so the event has happened; born 1967-01-02 it is 2026-07-02, a day after the payment.
#[test]
fn lump_sum_age_event_reads_the_people_birth_date_day_exact() {
    let mut e = election(own(), LumpSumMethod::TenYear);
    e.0.first_payment = 20260701;
    assert!(e.0.qualifies(Some(19670101)));
    assert!(!e.0.qualifies(Some(19670102)));
    assert!(!e.0.qualifies(None));
    // The spouse's record reads the spouse's birth date, not the taxpayer's: the spouse born
    // 1935-06-01 keeps the election (Form 4972 line 30), the taxpayer born 1970 does not.
    let mut d = empty_return(FederalStatus::Joint);
    let people = d.people.as_mut().unwrap();
    people.taxpayer.birth_date = 19700101;
    people.spouse.as_mut().unwrap().birth_date = 19350601;
    let mut entry = record(100_000, 100_000, 0, election(own(), LumpSumMethod::TenYear));
    entry.owner = crate::DocumentOwner::Spouses;
    let spouse = added(&mut d, entry);
    let taxpayer =
        added(&mut d, record(100_000, 100_000, 0, election(own(), LumpSumMethod::TenYear)));
    let r = supported(&d);
    assert!(has_line(&r, spouse, 30));
    assert!(!has_line(&r, taxpayer, 30));
}

// The election states the lump-sum facts once: the qualifying event, the first and the latest
// earlier payment and the entire-balance answer are read from it by the one test, and a
// decedent's election reads the death as its event.
#[test]
fn lump_sum_election_states_its_event_and_payment_facts_once() {
    use crate::NuaEvent;
    let e = election(own(), LumpSumMethod::TenYear);
    assert_eq!(e.0.nua_facts(), (NuaEvent::Age, 20260301, None, true));
    let mut e = election(decedent(19300101, 20250601, 0), LumpSumMethod::TenYear);
    e.0.last_prior_payment = Some(20250701);
    e.0.entire_balance = false;
    assert_eq!(
        e.0.nua_facts(),
        (NuaEvent::Death { date: 20250601 }, 20260301, Some(20250701), false)
    );
}

#[test]
fn lump_sum_death_benefit_exclusion_shares_the_capital_gain_with_part_ii() {
    // The participant died 1995-06-01; 2a 60,000, box 3 15,000, estate tax 4,000, death benefit
    // share 5,000, Parts II and III. C = 15,000 / 60,000 = 0.25: E = 5,000 x 0.25 = 1,250 against
    // the capital gain and 4,000 x 0.25 = 1,000 of estate tax.
    //   6 = 15,000 - 1,000 - 1,250 = 12,750   7 = 2,550
    //   8 = 45,000   9 = 5,000 - 1,250 = 3,750   10 = 41,250   12 = 41,250   13 = 10,000
    //   14 = 21,250   15 = 4,250   16 = 5,750   17 = 35,500   18 = 3,000   19 = 32,500
    //   25 = 10 x (260.50 + 14% x (3,250 - 2,270)) = 10 x 397.70 = 3,977   30 = 2,550 + 3,977
    let mut d = bare(19620404);
    let mut e = election(
        Form4972Participant::Decedent {
            death_benefit: 5_000 * 100,
            born: 19300201,
            died: 19950601,
            estate_tax: 4_000 * 100,
        },
        LumpSumMethod::Both,
    );
    e.0.first_payment = 20260601;
    e.1.box_3 = Some(15_000 * 100);
    let id = added(&mut d, record(60_000, 60_000, 0, e));
    let r = supported(&d);
    assert_eq!(form(&r, id, 6), 12_750);
    assert_eq!(form(&r, id, 7), 2_550);
    assert_eq!(form(&r, id, 9), 3_750);
    assert_eq!(form(&r, id, 12), 41_250);
    assert_eq!(form(&r, id, 19), 32_500);
    assert_eq!(form(&r, id, 25), 3_977);
    assert_eq!(form(&r, id, 30), 6_527);
}

#[test]
fn printed_retirement_boxes_survive_pending_election_and_source_correction() {
    use crate::tests::cross::interview::{accepted, document, normal_kept, r1099};
    use crate::tests::support::client::income_pages::{
        information_return_answers, save_documents, save_information_return_answers,
    };
    let mut draft = complete(FederalStatus::Single);
    draft.people.as_mut().unwrap().taxpayer.birth_date = 19350601;
    let mut session = Interview::new(TEST_USER, draft);
    let mut source = r1099(17_500_000, normal_kept(), false);
    source.source_mut().form.box7 = SEVEN_A;
    source.source_mut().form.taxable = Some(15_000_000);
    source.source_mut().form.box_3 = Some(1_000_000);
    source.source_mut().form.box_6 = Some(3_000_000);
    source.source_mut().form.box_8a = Some(500_000);
    source.source_mut().form.box_8b_percent = Some(33.33);
    source.source_mut().form.box_9a_percent = Some(50.0);
    source.source_mut().form.box_9b = Some(456);
    source.source_mut().form.box_10 = Some(123);
    let rows = |id, source| {
        vec![document(
            id,
            IncomeSource::Retirement {
                value: crate::RetirementDocument::R1099(retirement_document(source)),
            },
        )]
    };
    let request = save_documents(&mut session, rows(0, source.clone()));
    accepted(&send(&mut session, request));
    let id = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Retirement { .. }))
        .unwrap()
        .id;
    let read = |session: &Interview| match &session
        .draft
        .income
        .iter()
        .find(|entry| entry.id == id)
        .unwrap()
        .income
    {
        FederalIncome::Retirement { data } => data.clone(),
        _ => unreachable!(),
    };
    let saved = read(&session);
    // Printed determined box 2a supplies Payer; the independent distribution
    // treatment and optional Form 4972 election remain unanswered.
    assert!(matches!(
        crate::FederalRetirement::from(&saved).taxable,
        Some(RetirementTaxable::Payer)
    ));
    assert!(crate::FederalRetirement::from(&saved).ordinary_distribution.is_none());
    assert_eq!(
        (
            crate::FederalRetirement::from(&saved).box_2a,
            crate::FederalRetirement::from(&saved).box_3,
            crate::FederalRetirement::from(&saved).box_6,
            crate::FederalRetirement::from(&saved).box_8a
        ),
        (15_000_000, 1_000_000, 3_000_000, 500_000)
    );
    assert_eq!(
        (
            crate::FederalRetirement::from(&saved).box_8b_percent,
            crate::FederalRetirement::from(&saved).box_9a_percent,
            crate::FederalRetirement::from(&saved).box_9b,
            crate::FederalRetirement::from(&saved).box_10,
            crate::FederalRetirement::from(&saved).ira
        ),
        (Some(33.33), Some(50.0), Some(456), 123, false)
    );
    session = restart(session.draft);
    assert_eq!(read(&session), saved);
    let Reply::Response(P::Review { remaining_items, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        unreachable!()
    };
    assert!(remaining_items.iter().any(|row| row.section == TaxSection::Retirement));
    source.source_mut().form.box_10 = Some(0);
    let request = save_documents(&mut session, rows(id, source.clone()));
    accepted(&send(&mut session, request));
    source.source_mut().code = Some(DistributionCode::NormalLumpSum {
        tax: LumpSumTax::Form4972 {
            method: LumpSumMethod::Both,
            include_nua: false,
            event: LumpSumEvent::Age,
            first_payment: 20260301,
            last_prior_payment: None,
            entire_balance: true,
            confirmed: true,
        },
    });
    save_information_return_answers(&mut session, &rows(id, source.clone()));
    let established = read(&session);
    assert!(matches!(
        crate::FederalRetirement::from(&established).taxable,
        Some(RetirementTaxable::Payer)
    ));
    assert!(matches!(
        crate::FederalRetirement::from(&established).ordinary_distribution,
        Some(RetirementTreatment::LumpSum { .. })
    ));
    let mut result = supported(&session.draft);
    let request = save_documents(&mut session, rows(id, source.clone()));
    accepted(&send(&mut session, request));
    assert_eq!(read(&session), established);
    result.revision = session.draft.revision;
    assert_eq!(supported(&session.draft), result);
    source.source_mut().form.box_3 = Some(0);
    let request = save_documents(&mut session, rows(id, source.clone()));
    accepted(&send(&mut session, request));
    assert!(session.draft.valid_stored());
    assert_eq!(
        crate::FederalRetirement::from(&read(&session)).ordinary_distribution,
        crate::FederalRetirement::from(&established).ordinary_distribution
    );
    let before = session.draft.clone();
    let request = information_return_answers(&mut session, &rows(id, source.clone()));
    let Reply::Response(P::Problem { message, .. }) = send(&mut session, request) else {
        unreachable!()
    };
    assert_eq!(
        message,
        libertas::libertas_formatted_text("LUMP_SUM_CAPITAL_GAIN_WITHOUT_BOX_3", &[])
    );
    assert_eq!(session.draft, before);
    source.source_mut().form.box_3 = Some(1_000_000);
    let request = save_documents(&mut session, rows(id, source));
    accepted(&send(&mut session, request));
    result.revision = session.draft.revision;
    assert_eq!(supported(&session.draft), result);
}
