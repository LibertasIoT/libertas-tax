// Coverage: 1040-G22 (2026-10-18): a retained planned line 36 election is
// admitted by the existing finalized Form 1040 settlement, after penalty.
// Reported whole-dollar credit reduces the first estimated installment(s),
// once; unused credit and the remaining cash schedule are shown explicitly.
// Joint forecasts admit either account intent; separate taxpayer forecasts
// exclude spouse-account intent. No payment, filing or cross-year record is
// created. Sources: final 2025 Form 1040 instructions, lines 36/38,
// https://www.irs.gov/instructions/i1040gi, and Publication 505 (2026),
// "Credit an Overpayment", https://www.irs.gov/publications/p505.
// Evidence: payments/penalty applied_credit_* and planned_credit_*;
// payments/interview applied_credit_*; core/storage applied_credit_*.
// Coverage: Form 1040-ES (2027 estimated tax computation) — Supported (units T2-49 and T2-49b,
//   2026-09-26).
//   Lines: the Estimated Tax Worksheet for 2027, figured from the finished 2026 return when the
//   payments page asks for it (FederalPayments.next_year, a NextYearEstimate): line 1 expected
//   AGI (Form 1040 line 11); 2a deductions (line 12e's itemized deductions after the section 68
//   reduction, or the standard deduction plus the nonitemizer cash gifts and qualified disaster
//   loss); 2b the QBI deduction (line 13b); 2c the Schedule 1-A deductions (line 13a); 2d, 3;
//   4 the tax on line 3 by the Tax Rate Schedules (never the Tax Table), or by the Qualified
//   Dividends and Capital Gain / Schedule D Tax Worksheet at the Rate Schedules with this return's
//   qualified dividends, net capital gain, 28% and unrecaptured section 1250 amounts; 5 the AMT
//   (Form 6251); 6 = 4 + 5 + the positive Form 8978 adjustment on Form 1040 line 16; 7 the
//   nonrefundable credits (Form 1040 line 21) less the credits the package says cannot be claimed
//   again (new and previously owned clean vehicle, legacy EV, residential clean energy and
//   refueling); 8; 9 the Schedule SE tax; 10 the Schedule 2 Part II taxes the instructions list
//   (Additional Medicare, NIIT, early-distribution, RMD excess-accumulation, HSA/MSA/ABLE/IRA/Trump
//   account additional taxes and excises, education-credit recapture, the negative Form 8978
//   adjustment; household employment tax only under Exception 1: withholding expected, or the
//   worksheet already requires payments without it; excluded as the form excludes them: Forms
//   4137/8919, uncollected tip and group-term taxes, Form 8828 recapture); 11a; 11b the
//   refundable credits (EIC, ACTC, refundable AOTC and adoption, net PTC, section 1341, fuel tax
//   credit); 11c; 12a 90% (66⅔% when the Form 1040-ES farmer/fisherman answer is Yes);
//   12b this return's tax figured as the instructions say (Form 1040 line 24 without the excluded
//   Schedule 2 taxes, less the line 11b credits) at 100%, or 110% when this return's AGI is above
//   $150,000 ($75,000 married filing separately), never 110% for a farmer/fisherman; 12c the
//   smaller; 13 this return's line 25d withholding; 14a (zero or less: no payments required),
//   14b (under $1,000: no payments required); 15 one quarter of 14a, or the whole of 14a as the
//   single farmer/fisherman installment. Every line is a form_line-tagged whole-dollar return
//   line (TaxForm::Form1040Es, ids form_1040_es_line_*), with the installment schedule, the
//   figures' year and the omitted Schedule 2 Part I repayments as review notes. WithChanges
//   replaces line 1, 2a, 7 or 13 with the entered expected amount; every other line stays as this
//   return's (documented on the page). The 2027 Form 1040-ES is not published: the 2026 package's
//   Rate Schedules and worksheet shape are used and the review note says so.
//   Farmer/fisherman (T2-49b, the user's 2026-09-26 per-year decision): the test is two-thirds
//   of gross income from farming or fishing for 2026 or 2027 (2026 package, Special Rules). 2026
//   is figured from this return (underpayment::two_thirds_return_year, the same result Form 2210 uses);
//   2027 is asked as an expectation, the farmer_fisherman_next_year answer inside
//   NextYearEstimate::SameAsThisReturn and ::WithChanges, and ignored when 2026 qualifies. Either
//   sets the 66⅔% rate, the no-110% rule and the single January installment. STORAGE_FORMAT 57.
//   T2-49c (2026-09-26): the 2027 answer is its own FarmerNextYear (Yes, No, NotNeeded)
//   constrained by the payments page's allowed_farmer_next_year set: NotNeeded alone when 2026
//   qualifies (the client selects it), otherwise required Yes or No; checked again on save.
//   STORAGE_FORMAT 58.
//   Refused by name: a final return (no 2027 estimate); the 2027 farmer/fisherman answer Not
//   needed when this return does not show two-thirds for 2026 is reopened as an unanswered required choice; a return whose Form 1040 line 16 is not the Rate Schedule or QDCG/Schedule D
//   worksheet tax (Form 8615, Schedule J, the Form 2555 exclusion, Pub. 505's other tax
//   worksheets); an entered AGI or deduction change on a return with AMT (Form 6251 for 2027 is
//   not refigured). Not calculated, named in notes or the map: the annualized income installment
//   method for 2027 (Pub. 505 Worksheet 2-7), the Schedule 2 Part I premium and clean-vehicle repayments, which
//   the worksheet's lines do not carry, Form 1040-ES (NR) and the Form 1040-SS worksheet (no such
//   filer exists in this App), fiscal years.
//   Evidence: forms/form_1040_es.rs (compute); federal_rules.rs (the call after Form 2210);
//   federal_model.rs (NextYearEstimate and its farmer_fisherman_next_year answers,
//   FederalPayments.next_year, Payments.next_year,
//   TaxForm::Form1040Es); interview.rs (payments_page, payments_record, payments_candidate);
//   src/tests/payments/penalty.rs form_1040_es_worksheet_matches_hand_computation,
//   form_1040_es_110_percent_starts_above_150000_agi,
//   form_1040_es_stops_when_line_14b_is_under_1000,
//   form_1040_es_stops_when_withholding_covers_the_required_payment,
//   farmer_test_uses_2025_for_form_2210_and_2027_for_form_1040_es,
//   next_year_not_needed_cannot_supply_a_newly_applicable_farming_answer,
//   farm_return_meeting_two_thirds_for_2026_needs_neither_answer,
//   src/tests/payments/interview.rs farmer_2027_choices_follow_the_2026_test_and_are_checked_on_save (T2-49c),
//   commercial_fishing_business_counts_toward_the_2026_test (all T2-49b),
//   form_1040_es_expected_changes_replace_the_return_amounts,
//   form_1040_es_is_not_figured_for_a_final_return; src/tests/payments/interview.rs
//   next_year_estimate_round_trips_through_the_payments_page (T2-49b: the 2027 answer stored,
//   newly applicable answers constructed through WorkflowRequest),
//   farmer_choices_follow_the_2026_test_from_the_return.
//   IRS sources: 2026 Form 1040-ES package (Feb 12, 2026), irs.gov/pub/irs-pdf/f1040es.pdf,
//   fetched 2026-09-26: the 2026 Estimated Tax Worksheet, its instructions (lines 1, 7, 9, 10,
//   12b and "Figuring your 2025 tax"), the 2026 Tax Rate Schedules, the General Rule and Special
//   Rules; no 2027 package or 2027 revenue procedure exists yet. Due dates for 2027 follow the
//   package's rule (the 15th, or the next business day): April 15, June 15 and September 15,
//   2027 and January 18, 2028 (January 15 is a Saturday and January 17 a legal holiday).

// Form 1040-ES: the 2027 Estimated Tax Worksheet projected from the finished
// 2026 return, with the filer's expected changes where entered.
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::{FarmerNextYear, FederalResult, FederalStatus, FormLine, NextYearEstimate, TaxForm};
use alloc::{format, vec::Vec};

const SOURCE: &str = "https://www.irs.gov/pub/irs-pdf/f1040es.pdf";
const INPUTS: &[&str] = &["payments.next_year", "income", "deductions", "credits"];
const CREDIT_INPUTS: &[&str] =
    &["payments.credit_applied_next_year", "payments.credit_recipient", "payments.next_year"];
#[cfg(test)]
const SECTION: &str = "Form 1040-ES 2027 estimated tax";
const ESTIMATE_YEAR: u32 = 2027;
// The 2027 package is unpublished; the 2026 Rate Schedules stand in.
const RATES_YEAR: u32 = 2026;

// Facts from the finished 2026 return, in whole dollars.
pub(crate) struct Form1040EsContext<'a> {
    pub request: &'a NextYearEstimate,
    // Admitted reported line 36, for this forecast's household/account only.
    pub applied_credit_cents: i64,
    pub status: FederalStatus,
    pub agi: i64,
    // Line 12e's itemized deductions after reduction, or the standard
    // deduction with the nonitemizer cash gifts and qualified disaster loss.
    pub deduction: i64,
    pub qbi: i64,
    pub additional: i64,
    // The return's qualified dividends plus net capital gain, 28% and
    // unrecaptured section 1250 amounts, capped as the worksheet caps them.
    pub preference: [i64; 3],
    // Form 1040 line 16 is the Rate Schedule / Tax Table or QDCG worksheet tax
    // (no Form 8615, Schedule J or Form 2555 computation).
    pub plain_tax: bool,
    pub amt: i64,
    // The positive Form 8978 adjustment included on Form 1040 line 16.
    pub line_16_other: i64,
    // Schedule 2 Part I repayments (excess advance PTC, transferred clean
    // vehicle credit), which the worksheet's lines do not carry.
    pub part_one_repayments: i64,
    pub credits: i64,
    pub expired_credits: i64,
    pub se_tax: i64,
    // The Schedule 2 Part II taxes the line 10 instructions list, without
    // household employment tax.
    pub other_taxes: i64,
    pub household_tax: i64,
    pub refundable_credits: i64,
    // This return's tax as the line 12b instructions figure it, including
    // household employment tax.
    pub this_year_tax: i64,
    pub withholding: i64,
    // At least two-thirds of 2026 gross income from farming or fishing, as
    // this return shows it (underpayment::two_thirds_return_year).
    pub farming_return_year: bool,
    pub deceased: bool,
    pub annualized: bool,
}

struct Changes {
    agi: Option<i64>,
    deductions: Option<i64>,
    credits: Option<i64>,
    withholding: Option<i64>,
}

// The worksheet's lines in order, in whole dollars; None where the
// worksheet stops before a line.
struct Worksheet {
    lines: Vec<(&'static str, i64, crate::Message)>,
    // The required annual payment is covered (14a ≤ 0) or under the $1,000
    // threshold (14b < 1,000).
    required: Option<i64>,
}

pub(crate) fn compute(ctx: &Form1040EsContext<'_>, r: &mut FederalResult) {
    // The farmer and fisherman test is two-thirds of 2026 gross income (this
    // return) or of expected 2027 gross income (the answer asked with the
    // worksheet).
    let (changes, answer_next_year) = match ctx.request {
        NextYearEstimate::NotNeeded => return,
        NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year } => (
            Changes { agi: None, deductions: None, credits: None, withholding: None },
            *farmer_fisherman_next_year,
        ),
        NextYearEstimate::WithChanges {
            agi,
            deductions,
            credits,
            withholding,
            farmer_fisherman_next_year,
        } => (
            Changes {
                agi: agi.map(dollars),
                deductions: deductions.map(dollars),
                credits: credits.map(dollars),
                withholding: withholding.map(dollars),
            },
            *farmer_fisherman_next_year,
        ),
    };
    let mut refused = false;
    let mut refuse = |r: &mut FederalResult, ok: bool, key: &str, #[cfg(test)] message: &str| {
        if !ok {
            let context = crate::IssueContext::Section(crate::TaxSection::Payments);
            let context =
                if key == "ISSUE_1040ES_AMT_CHANGED_INCOME" { context } else { context.manual() };
            crate::issue_localized!(r, &context, SECTION, key, message,);
            refused = true;
        }
    };
    if !ctx.farming_return_year && answer_next_year == FarmerNextYear::NotNeeded {
        return;
    }
    refuse(
        r,
        !ctx.deceased,
        "ISSUE_1040ES_FINAL_RETURN",
        #[cfg(test)]
        "Form 1040-ES: a 2027 estimated tax is not figured for a final return.",
    );
    refuse(
        r,
        ctx.plain_tax,
        "ISSUE_1040ES_TAX_WORKSHEET",
        #[cfg(test)]
        "Form 1040-ES line 4: this return's tax is not the Tax Rate Schedule or Qualified Dividends and Capital Gain Tax Worksheet tax (Form 8615, Schedule J or the Form 2555 exclusion applies); the Pub. 505 worksheet for that tax is not calculated.",
    );
    refuse(
        r,
        ctx.amt == 0 || (changes.agi.is_none() && changes.deductions.is_none()),
        "ISSUE_1040ES_AMT_CHANGED_INCOME",
        #[cfg(test)]
        "Form 1040-ES line 5: this return has alternative minimum tax, and Form 6251 for 2027 is not refigured for a changed income or deduction; choose the estimate from this return as it is.",
    );
    if refused {
        return;
    }
    let farmer = ctx.farming_return_year || answer_next_year == FarmerNextYear::Yes;
    // Exception 1 for household employment tax on line 10: include it when
    // 2027 income tax will be withheld, or when the worksheet without it
    // already requires estimated payments.
    let withholding = changes.withholding.unwrap_or(ctx.withholding);
    let without = worksheet(ctx, &changes, farmer, 0);
    let household =
        if withholding > 0 || without.required.is_some() { ctx.household_tax } else { 0 };
    let sheet = if household == 0 { without } else { worksheet(ctx, &changes, farmer, household) };
    emit_lines(r, &mut (), &sheet.lines, |(reference, amount, explanation), _, _| {
        Some(calculation_row(
            Some(FormLine::new(TaxForm::Form1040Es, reference)),
            &format!("form_1040_es_line_{}", reference),
            LineValue::MoneyDollars(*amount),
            INPUTS,
            SOURCE,
            explanation.clone(),
        ))
    });
    r.review_notes.push(crate::federal_model::ReviewNote::localized(
        #[cfg(test)] format!(
            "Form 1040-ES for {ESTIMATE_YEAR}: the IRS has not published the {ESTIMATE_YEAR} package, so the worksheet uses the {RATES_YEAR} Tax Rate Schedules and this return's {RATES_YEAR} deductions, credits and other taxes as the estimate. Refigure it with the {ESTIMATE_YEAR} Form 1040-ES when it is published."
        ),
        crate::Message::new(
            "NOTE_1040ES_UNPUBLISHED_PACKAGE",
            alloc::vec![
                crate::MessageArg::Number(i64::from(ESTIMATE_YEAR)),
                crate::MessageArg::Number(i64::from(RATES_YEAR)),
            ],
        ),
    ));
    match sheet.required {
        None => r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)] "Form 1040-ES line 14: you are not required to make 2027 estimated tax payments (the required annual payment is covered by expected withholding, or the expected balance is under $1,000). You may still owe tax when you file.",
            crate::Message::new("NOTE_1040ES_NO_REQUIRED_PAYMENT", alloc::vec![]),
        )),
        Some(_) if ctx.applied_credit_cents > 0 => r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)] "The planned applied credit is already deducted in the installment schedule below. Do not subtract it again.",
            crate::Message::new("NOTE_1040ES_CREDIT_ALREADY_APPLIED", alloc::vec![]),
        )),
        Some(installment) if farmer => r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)] format!("Form 1040-ES line 15 (farmers and fishermen): pay the ${installment} required annual payment in one installment by January 18, 2028, or file your 2027 return and pay the tax in full by March 1, 2028. Subtract any 2026 overpayment you apply to 2027."),
            crate::Message::new("NOTE_1040ES_FARMER_INSTALLMENT", alloc::vec![crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: installment }]),
        )),
        Some(installment) => r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)] format!("Form 1040-ES line 15: pay ${installment} by each of April 15, 2027, June 15, 2027, September 15, 2027 and January 18, 2028 (January 15, 2028 is a Saturday and January 17 a legal holiday). Subtract any 2026 overpayment you apply to the first installment; you may instead pay the whole amount by April 15, 2027."),
            crate::Message::new("NOTE_1040ES_QUARTERLY_INSTALLMENT", alloc::vec![crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: installment }]),
        )),
    }
    if ctx.applied_credit_cents > 0 {
        let mut available = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
            ctx.applied_credit_cents,
        )
        .expect("admitted nonnegative credit");
        if let Some(installment) = sheet.required {
            for index in 0..if farmer { 1 } else { 4 } {
                // Use line 15's already reported installment, retaining its
                // existing rounding convention when there is no election.
                let due = installment * 100;
                let settlement = crate::kernel::carry::consume_step(
                    crate::kernel::carry::Amount::new(due).expect("nonnegative installment"),
                    &mut available,
                    false,
                )
                .expect("nonnegative forecast credit and installment");
                let used = settlement.used.get();
                let cash = settlement.closing.get();
                emit_lines(r, &mut (), &[(index, cash)], |(index, cash), _, _| {
                    Some(calculation_row(
                        None,
                        &format!("form_1040_es_planned_cash_{}", index + 1),
                        LineValue::MoneyCents(*cash),
                        CREDIT_INPUTS,
                        "https://www.irs.gov/publications/p505",
                        crate::Message::new(
                            "NOTE_1040ES_APPLIED_INSTALLMENT",
                            alloc::vec![
                                crate::MessageArg::Number(index + 1),
                                crate::MessageArg::usd_cents(due),
                                crate::MessageArg::usd_cents(used),
                                crate::MessageArg::usd_cents(*cash),
                            ],
                        ),
                    ))
                });
            }
        }
        let available = available.get();
        if available > 0 {
            r.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)]
                format!("Planned credit remaining after the schedule: {available} cents."),
                crate::Message::new(
                    "NOTE_1040ES_CREDIT_REMAINDER",
                    alloc::vec![crate::MessageArg::usd_cents(available)],
                ),
            ));
        }
    }
    if ctx.part_one_repayments > 0 {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)] format!(
                "Form 1040-ES: this return repays ${} of excess advance premium tax credit or transferred clean-vehicle credit on Schedule 2 Part I; the Estimated Tax Worksheet's lines 5, 6 and 10 do not list those repayments, so they are not in the 2027 estimate. Add them yourself if you expect them again.",
                ctx.part_one_repayments
            ),
            crate::Message::new(
                "NOTE_1040ES_OMITTED_REPAYMENTS",
                alloc::vec![crate::MessageArg::Currency {
                    currency_code: "USD".into(),
                    scale: 0,
                    minor_units: ctx.part_one_repayments,
                }],
            ),
        ));
    }
    if ctx.annualized {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(
            #[cfg(test)] "Form 1040-ES line 15 divides the required annual payment into four equal installments; the annualized income installment method for 2027 (Pub. 505 Worksheet 2-7) is not calculated.",
            crate::Message::new("NOTE_1040ES_ANNUALIZED_NOT_CALCULATED", alloc::vec![]),
        ));
    }
}

fn worksheet(
    ctx: &Form1040EsContext<'_>,
    changes: &Changes,
    farmer: bool,
    household: i64,
) -> Worksheet {
    let mut lines: Vec<(&'static str, i64, crate::Message)> = Vec::new();
    let mut put = |reference: &'static str, amount: i64, explanation: crate::Message| {
        lines.push((reference, amount, explanation));
        amount
    };
    let year = |same: crate::Message, changed: crate::Message, changed_value: Option<i64>| {
        match changed_value {
            Some(_) => changed,
            None => same,
        }
    };
    let line_1 = put(
        "1",
        changes.agi.unwrap_or(ctx.agi),
        year(
            msg!("EXPLAIN_FORM_1040_ES_LINE_1"),
            msg!("EXPLAIN_FORM_1040_ES_LINE_1_2"),
            changes.agi,
        ),
    );
    let line_2a = put(
        "2a",
        changes.deductions.unwrap_or(ctx.deduction),
        year(
            msg!("EXPLAIN_FORM_1040_ES_LINE_2A"),
            msg!("EXPLAIN_FORM_1040_ES_LINE_2A_2"),
            changes.deductions,
        ),
    );
    let line_2b = put("2b", ctx.qbi, msg!("EXPLAIN_FORM_1040_ES_LINE_2B"));
    let line_2c = put("2c", ctx.additional, msg!("EXPLAIN_FORM_1040_ES_LINE_2C"));
    let line_2d = put("2d", line_2a + line_2b + line_2c, msg!("EXPLAIN_FORM_1040_ES_LINE_2D"));
    let line_3 = put("3", positive(line_1 - line_2d), msg!("EXPLAIN_FORM_1040_ES_LINE_3"));
    let line_4 = put(
        "4",
        rate_schedule_tax(ctx, line_3),
        if ctx.preference[0] > 0 {
            crate::Message::new(
                "EXPLAIN_FORM_1040_ES_LINE_4",
                alloc::vec![
                    crate::MessageArg::Number(i64::from(RATES_YEAR)),
                    crate::MessageArg::usd_dollars(ctx.preference[0]),
                ],
            )
        } else {
            crate::Message::new(
                "EXPLAIN_FORM_1040_ES_LINE_4_2",
                alloc::vec![
                    crate::MessageArg::Number(i64::from(RATES_YEAR)),
                    crate::MessageArg::Number(i64::from(ESTIMATE_YEAR)),
                ],
            )
        },
    );
    let line_5 = put("5", ctx.amt, msg!("EXPLAIN_FORM_1040_ES_LINE_5"));
    let line_6 = put(
        "6",
        line_4 + line_5 + ctx.line_16_other,
        if ctx.line_16_other > 0 {
            crate::Message::new(
                "EXPLAIN_FORM_1040_ES_LINE_6",
                alloc::vec![crate::MessageArg::usd_dollars(ctx.line_16_other),],
            )
        } else {
            msg!("EXPLAIN_FORM_1040_ES_LINE_6_2")
        },
    );
    let line_7 = put(
        "7",
        changes.credits.unwrap_or(positive(ctx.credits - ctx.expired_credits)),
        year(
            if ctx.expired_credits > 0 {
                msg!("EXPLAIN_FORM_1040_ES_LINE_7")
            } else {
                msg!("EXPLAIN_FORM_1040_ES_LINE_7_2")
            },
            msg!("EXPLAIN_FORM_1040_ES_LINE_7_3"),
            changes.credits,
        ),
    );
    let line_8 = put("8", positive(line_6 - line_7), msg!("EXPLAIN_FORM_1040_ES_LINE_8"));
    let line_9 = put("9", ctx.se_tax, msg!("EXPLAIN_FORM_1040_ES_LINE_9"));
    let line_10 = put(
        "10",
        ctx.other_taxes + household,
        if household > 0 {
            msg!("EXPLAIN_FORM_1040_ES_LINE_10")
        } else if ctx.household_tax > 0 {
            msg!("EXPLAIN_FORM_1040_ES_LINE_10_2")
        } else {
            msg!("EXPLAIN_FORM_1040_ES_LINE_10_3")
        },
    );
    let line_11a = put("11a", line_8 + line_9 + line_10, msg!("EXPLAIN_FORM_1040_ES_LINE_11A"));
    let line_11b = put("11b", ctx.refundable_credits, msg!("EXPLAIN_FORM_1040_ES_LINE_11B"));
    let line_11c = put("11c", positive(line_11a - line_11b), msg!("EXPLAIN_FORM_1040_ES_LINE_11C"));
    let line_12a = put(
        "12a",
        if farmer { fraction(line_11c, 2, 3) } else { fraction(line_11c, 90, 100) },
        if farmer {
            msg!("EXPLAIN_FORM_1040_ES_LINE_12A")
        } else {
            msg!("EXPLAIN_FORM_1040_ES_LINE_12A_2")
        },
    );
    let separate = ctx.status == FederalStatus::Separate;
    let high_income = ctx.agi > if separate { 75_000 } else { 150_000 };
    let line_12b = put(
        "12b",
        if high_income && !farmer {
            fraction(ctx.this_year_tax, 110, 100)
        } else {
            ctx.this_year_tax
        },
        if high_income && !farmer {
            crate::Message::new(
                "EXPLAIN_FORM_1040_ES_LINE_12B",
                alloc::vec![
                    crate::MessageArg::usd_dollars(ctx.this_year_tax),
                    crate::MessageArg::usd_dollars(if separate { 75_000 } else { 150_000 }),
                ],
            )
        } else if farmer && high_income {
            crate::Message::new(
                "EXPLAIN_FORM_1040_ES_LINE_12B_2",
                alloc::vec![crate::MessageArg::usd_dollars(ctx.this_year_tax),],
            )
        } else {
            crate::Message::new(
                "EXPLAIN_FORM_1040_ES_LINE_12B_3",
                alloc::vec![crate::MessageArg::usd_dollars(ctx.this_year_tax),],
            )
        },
    );
    let line_12c =
        put("12c", phase::cap(line_12a, [line_12b]), msg!("EXPLAIN_FORM_1040_ES_LINE_12C"));
    let line_13 = put(
        "13",
        changes.withholding.unwrap_or(ctx.withholding),
        year(
            msg!("EXPLAIN_FORM_1040_ES_LINE_13"),
            msg!("EXPLAIN_FORM_1040_ES_LINE_13_2"),
            changes.withholding,
        ),
    );
    let line_14a = put("14a", line_12c - line_13, msg!("EXPLAIN_FORM_1040_ES_LINE_14A"));
    if line_14a <= 0 {
        return Worksheet { lines, required: None };
    }
    let line_14b = put("14b", line_11c - line_13, msg!("EXPLAIN_FORM_1040_ES_LINE_14B"));
    if line_14b < 1_000 {
        return Worksheet { lines, required: None };
    }
    let installment = if farmer { line_14a } else { fraction(line_14a, 1, 4) };
    put(
        "15",
        installment,
        if farmer {
            msg!("EXPLAIN_FORM_1040_ES_LINE_15")
        } else {
            crate::Message::new(
                "EXPLAIN_FORM_1040_ES_LINE_15_2",
                alloc::vec![crate::MessageArg::usd_cents(line_14a * 25),],
            )
        },
    );
    Worksheet { lines, required: Some(installment) }
}

// Line 4: the tax on line 3 by the Tax Rate Schedules, or by the Qualified
// Dividends and Capital Gain / Schedule D Tax Worksheet at those schedules
// with this return's preferential amounts (Pub. 505 chapter 2).
fn rate_schedule_tax(ctx: &Form1040EsContext<'_>, taxable: i64) -> i64 {
    let rates =
        crate::income_tax::Rates::for_year(RATES_YEAR, ctx.status).expect("2026 rate schedules");
    let [preferential, collectibles, unrecaptured] = ctx.preference;
    let preferential = phase::cap(positive(preferential), [taxable]);
    if preferential == 0 {
        return dollars(rates.ordinary_cents(taxable));
    }
    let unrecaptured = phase::cap(positive(unrecaptured), [preferential]);
    let collectibles = phase::cap(positive(collectibles), [preferential - unrecaptured]);
    rates.schedule_d(
        taxable,
        preferential,
        collectibles + unrecaptured,
        unrecaptured,
        collectibles > 0,
    )
}
