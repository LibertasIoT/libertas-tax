// Current farmer-gross scope (2026-10-21): recognized positive capital/property
// sources remain gross after actual exclusions/timing and before cross-source
// losses/carryovers. Known farm machinery and livestock include current recapture and
// installment income once. K-1/K-3 whole-source gross and earlier property's
// farming character remain explicit source boundaries when qualification affects
// a positive penalty or requested forecast. Prior-year qualification is independent.
// Evidence: payments::penalty::farmer_gross_* and farmer_property_gross_*;
// IRS Pub. 225 chapter 15, https://www.irs.gov/publications/p225;
// Pub. 505 chapter 2, https://www.irs.gov/publications/p505.
// Coverage: Form 2210 — Supported (audited 2026-09-25).
//   Lines: Part I 1–9 (line 1 = Form 1040 line 22; line 2 = the Schedule 2 taxes the
//   instructions list: self-employment, Additional Medicare, NIIT, additional taxes on
//   retirement, ABLE, HSA and MSA distributions, education-credit recapture, the allowed negative
//   Form 8978 adjustment, the Form 4255 recapture (Schedule 2 line 13a), the Trump account
//   distributions tax (Form 5329 line 63) and household employment tax unless line 25d is zero
//   and, without it, line 7 is under $1,000 or the full 2025 year had no tax (section
//   3510(b)(2)(B), section 6654(e); line audit F10 2026-10-10); Forms 4137/8919, uncollected tip taxes, Form 8828 recapture and Form 5329
//   excess-contribution/accumulation excises are excluded as the list and Form 1040 line 38
//   instructions exclude them; line 3 = EIC, ACTC, refundable AOTC, refundable adoption, net
//   PTC, section 1341 credit and the Form 4136 fuel credit, which stays in Form 1040 line 31 and
//   is only read here; line 6 = line 25d plus excess Social Security; line 8 from the entered
//   2025 tax at 100% or 110% over $150,000 AGI ($75,000 MFS), skipped for a short 2025 year;
//   the flowchart stops at line 4 or line 7 under $1,000 or line 9 not above line 6). A zero
//   2025 tax on a full-year return gives line 9 = 0, so no penalty. Part II boxes A–C and E stay
//   unset (C24b/C24c); box D is the withholding_by_date election, which needs the four
//   per-period withholding entries summing to line 6 and is refused otherwise; each entry is
//   treated as paid on its column's due date. Part III regular method: line 10 = 25% of line 9;
//   line 11 = withholding share plus dated estimated payments in the column window plus any
//   Form 1041-T allocation in column (d) as paid January 15, 2027; lines 12–18 by applying
//   payments in date order to the oldest unpaid installment first; the penalty worksheet by
//   days over the four 2026 rate periods at the quarterly rates in
//   tests/fixtures/irs-underpayment-rates-2026.csv, each computation rounded to cents, line 19
//   their sum, and Form 1040 line 38 in whole dollars. Unpaid amounts accrue to April 15, 2027
//   (an extension payment and the balance due are treated as paid then).
//   Refused by name (the 2025 farmer/fisherman two-thirds test is a required Yes/No since the
//   Not-sure audit; a return that meets it figures Form 2210-F instead, see the next block):
//   Schedule AI with a section 1062 election lacking per-period tax attribution; Form 2439 tax
//   paid by a fund (no treated-as-paid date; also Form 2210-F line 12), an annual estimated
//   amount without dated payments, a dated list that disagrees with the annual amount, a payment
//   dated January 16-31, 2027 (the January 31 rule; Form 2210 only), and box D without matching
//   entries.
//   The annual methods include 75% of known Form 1062 applicable net tax liability
//   with refundable credits (Notice 2026-3 section 4); selected but unavailable
//   liability is a located Farming limitation. Schedule AI needs period attribution.
//   No prior-year facts: no Form 2210 is computed and the return is unchanged.
//   Evidence: src/tests/payments/penalty.rs form_2210_regular_method_matches_the_instructions_example,
//   no_penalty_under_1000_or_with_zero_prior_tax, estimated_payments_apply_by_date_to_the_periods.
//   Sources: draft Form 2210 (2026) irs.gov/pub/irs-dft/f2210--dft.pdf; Instructions for Form
//   2210 (2025) draft irs.gov/pub/irs-dft/i2210--dft.pdf (no 2026 instructions yet; dates and
//   thresholds carried to 2026); Form 1040 (2026) draft line 38 and the 2025 Form 1040
//   instructions for line 38; Rev. Rul. 2026-15 for the rates.
//   Unit C24b (2026-09-25): FederalPayments.annualized (box C) and annualized_income (AnnualizedIncome
//   with four AnnualizedPeriod columns: AGI, itemized, QBI, Schedule 1-A, SE net profit, Social
//   Security wages); Schedule AI Part I lines 1-27 and Part II lines 28-36 per column; line 27
//   replaces Part III line 10; column (d) must reconcile to the return (AGI, taxable income, SE tax
//   within $1) or the election is refused; line 18 credits: the child tax credit, credit for other
//   dependents and additional child tax credit follow the children's birth dates (a column takes
//   the share of children born by its last day; F10 2026-10-10), every other credit is its
//   annual amount in every column (an approximation: the instructions annualize each credit's
//   income and date its events); refused by name: non-Tax-Table tax on line 14 (QDCG, Form 8615, Schedule J,
//   foreign exclusion), line 16 other taxes (Additional Medicare, NIIT, AMT, household employment,
//   premium repayment), both spouses with SE income. Form 1040 line 37 is emitted once, including
//   the penalty. STORAGE_FORMAT 23. Tests: src/tests/payments/penalty.rs
//   schedule_ai_annualized_installments_replace_part_iii_line_10, schedule_ai_fourth_column_must_reconcile_to_the_return,
//   line_37_includes_the_penalty_once, schedule_ai_line_27_never_exceeds_the_regular_installment,
//   schedule_ai_part_ii_prorates_the_social_security_wage_base.
//   Unit C24c (2026-09-25): Part II box A/B (FederalPayments.waiver, a PenaltyWaiver Full/Partial
//   with a PenaltyWaiverReason and a free-text explanation) appends a review note
//   "Form 2210 Part II box A/B waiver requested: ..." and, for box B, a
//   form_2210_waiver_requested_amount return line. Unit F11 (2026-10-10, instructions win): the
//   request reduces line 19 and line 38 (box B capped at the penalty, box A to zero); the computed
//   penalty stays as form_2210_penalty_before_waiver.
//   Box E (PriorYearTax.spouse_tax, added at the end of that record) adds a joint filer's
//   spouse's separately filed 2025 tax to line 8 before the 100%/110% test.
//   Unit prior-agi (2026-10-09): PriorYearTax.agi is Option<i64>; blank (not a farmer) takes the
//   110% test as if over the threshold with one WARN_PRIOR_AGI_ASSUMED_HIGH note scoped to Payments
//   (only when line 8 is figured and reaches line 9); a farmer's Form 2210-F never reads it. Farmer/fisherman
//   (FederalPayments.farmer_fisherman_prior_year and farmer_paid_by_march, plain bools): see the Form
//   2210-F block; Form 2210 no longer sees a farmer (no 66⅔% line 5, no single installment in
//   column (d)). A balance paid January 16-31, 2027 counting as paid January 15 for column (d):
//   refused by name (a dated estimated payment in that window) rather than implemented — this
//   App has no fact recording that a dated payment specifically settles the balance due, and
//   adding one is a separate payment-date design decision. STORAGE_FORMAT 24. Out of scope,
//   reported not implemented: the 85% waiver history, Form 843 interest abatement. Tests:
//   src/tests/payments/penalty.rs waiver_requests_add_a_review_note_without_changing_the_penalty,
//   box_e_uses_the_combined_prior_year_tax_on_line_8,
//   column_d_january_31_payment_is_refused_by_name.
//   Unit T2-49b (2026-09-26, the user's per-year decision): the farmer/fisherman test is one
//   fact for three years. Form 2210 qualifies on 2025 or 2026: FederalPayments.farmer_fisherman_prior_year
//   (replacing C24c's farmer_fisherman, which asked "2025 or 2026") is asked; 2026 is figured
//   from this return by two_thirds_return_year per Pub. 505 (2026), chapter 2, "Farming and Fishing"
//   (gross income is the total of the income, not loss, amounts, both spouses on a joint return;
//   farming is Schedule F line 9, K-1 gross farming income and livestock gains; fishing is a
//   fishing Schedule C line 7, crew income and K-1 gross fishing income; farm-employee wages are
//   not farming). Implemented: farming/fishing = each positive Schedule F line 9, each positive
//   Schedule C line 7 of a business marked FederalBusiness.commercial_fishing, Form 1041 K-1 box
//   14 code F, Form 1065 K-1 box 14 code B where entered (PartnerSeGross::Farm); gross = the
//   positive amounts of Form 1040 lines 1z, 2b, 3b, 4b, 5b, 6b and 7 and of each Schedule 1 Part I
//   income component the return computes, with each business's gross income (Schedule C line 7,
//   Schedule F line 9, a gambler's receipts) in place of its net profit, Schedule 1 line 5 counted
//   with its code F part in full, and the NOL deduction, foreign exclusion, section 1244
//   stock loss and excess business loss adjustment left out (not income). Met when farming > 0
//   and 3 × farming ≥ 2 × gross; shown as return lines farming_fishing_gross_income_return_year and
//   gross_income_return_year when farming > 0. Approximations, named: amounts the return nets within a
//   line (Schedule 1 line 5, other rentals) count at their line net; Form 8949 sale gains count
//   gross, never netted against the sales' losses (F10 2026-10-10), other capital gains at the net; a Form 1065
//   code B amount is added beside the K-1's net income (overstating gross, so never a false Yes);
//   Form 4835 and 1099-MISC fishing boat proceeds are refused situations, livestock sales and
//   crew-member wages are not marked, so they do not count as farming or fishing. The page's
//   Farmer choice is No / Yes, for 2025 / Yes, for 2026 (ThisReturn, offered alone when 2026 is
//   met); the removal of the last business no longer resets the answer. STORAGE_FORMAT 57.
//   Tests: src/tests/payments/penalty.rs farmer_test_uses_2025_for_form_2210_and_2027_for_form_1040_es,
//   farmer_2027_not_sure_is_refused_by_name_when_2026_does_not_qualify,
//   farm_return_meeting_two_thirds_for_2026_needs_neither_answer,
//   commercial_fishing_business_counts_toward_the_2026_test; src/tests/passthrough/passthrough.rs
//   estate_trust_code_f_opens_the_farmer_test_and_cannot_exceed_box_6; src/tests/payments/interview.rs
//   farmer_choices_follow_the_2026_test_from_the_return,
//   commercial_fishing_round_trips_through_the_business_page.
//   C-fixture (2026-09-25, src/tests/payments/penalty.rs): underpayment_return_matches_hand_computation (a wage plus
//   Schedule C joint return through Parts I and III by hand), prior_year_agi_over_150000_uses_110_percent,
//   annualized_method_lowers_the_penalty_for_late_year_income, january_15_payment_clears_the_fourth_period_only.
// Coverage: Form 2210-F — Supported (audited 2026-10-08; map tax-forms/form-2210-f.md).
//   Underpayment of Estimated Tax by Farmers and Fishers, figured when the farmer and fisherman
//   test holds (farmer_2210: this return shows at least two-thirds of 2026 gross income from
//   farming or fishing, two_thirds_return_year, or the 2025 answer FederalPayments.farmer_fisherman_prior_year
//   is Yes); every other filer uses Form 2210. Mandatory, not a choice: section 6654(i)(1) gives a
//   farmer or fisher one installment, due January 15, 2027, and Form 2210-F has no Schedule AI and
//   no box C or D. Part II lines 1-13 (result ids form_2210_f_line_N in cents): line 1 = Form 1040
//   line 22; line 2 = the Schedule 2 taxes the instructions list (as Form 2210 line 2), household
//   employment tax joining unless line 25d is zero and the penalty is zero without it; line 3 =
//   1 + 2; line 4 = refundable credits (Form 2210 line 3's set); line 5 reserved (no row);
//   line 6 = 3 - 4; line 7 = 66⅔% of line 6 (the exact fraction, not the printed 0.667); line 8
//   = line 25d plus excess Social Security; line 9 = 6 - 8; line 10 = 100% of the 2025 tax
//   (plus a spouse's separate 2025 tax, box B), never 110% (section 6654(i)(1)(C) disregards
//   (d)(1)(C)), blank when the 2025 return did not cover 12 months; line 11 = the smaller of 7
//   and 10 (line 7 when line 10 is blank); line 12 = estimated payments dated through January
//   15, 2027, all 2026 withholding and the Form 1041-T allocation; line 13 = 11 - 12. Stops
//   (line 16 = 0 with the reason, later lines absent): line 3, 6 or 9 under $1,000, line 8 at
//   least line 11 (a zero 2025 tax gives line 11 = 0), line 13 zero. Part III: line 14 and 15
//   are internal (each later payment ends the period for what it pays, the rest runs to April
//   15, 2027; 90 days from January 15), line 16 at the one Q1 2027 underpayment rate (the
//   fixture's unpublished row takes the last published rate), each portion in cents, and Form
//   1040 line 38 in whole dollars explained by EXPLAIN_1040_38_3. The March 1 rule
//   (FederalPayments.farmer_paid_by_march, section 6654(i)(1)(D)) replaces Form 2210's January
//   31 rule: line 16 = 0 with the lines still shown, and a payment dated January 16-31 is
//   simply a payment on its date. Box A (both waiver choices) adds NOTE_2210F_WAIVER_REQUESTED
//   and, for a partial request, form_2210_f_waiver_requested_amount; line 16 is reduced by the
//   waiver (form_2210_f_penalty_before_waiver keeps the computed penalty). Box C and D answers saved while the return was not a farmer's are not used
//   (NOTE_2210F_PERIOD_ANSWERS_NOT_USED, linked to Payments); the Payments page blocks them for a
//   farmer (FARMER_EVEN_TIMING, client rule and server twin). Refused by name as on Form 2210:
//   unavailable section 1062 applicable liability, Form 2439 tax paid by a fund, annual estimated funding
//   without dated payments. Not identified by the 2026 test (so such a filer uses Form 2210, whose
//   penalty is never lower): Form 4797 livestock gains and fishing-vessel crew or shore wages
//   (deferred to the Schedule F / Form 4797 line audit); Form 4835 line 7 joins with its own
//   unit. Excluded: fiscal-year farmers, estates and trusts, Form 1040-NR, the printed form
//   (header, Part I box B, lines 14-15 as rows). No stored shape change.
//   Evidence: src/tests/payments/penalty.rs farmer_penalty_uses_100_percent_of_the_prior_year_never_110
//   (fixture A, A2), farmer_payment_in_late_january_is_figured_and_march_1_removes_the_penalty
//   (B), return_one_dollar_short_of_the_test_is_routed_to_form_2210 (C, C2),
//   fishing_income_on_a_joint_return_figures_form_2210_f (D), form_2210_f_boundaries_stop_at_the_form_lines
//   (E), stored_box_c_and_d_answers_are_not_used_on_a_farmer_return (F ii),
//   farmer_waiver_requests_use_form_2210_f_box_a, farmer_two_thirds_rule_uses_one_installment_at_66_percent;
//   src/tests/payments/interview.rs farmers_must_keep_withholding_and_income_even (F i).
//   Sources: draft Form 2210-F (2026) irs.gov/pub/irs-dft/f2210f--dft.pdf and the 2025 form
//   irs.gov/pub/irs-pdf/f2210f.pdf; Instructions for Form 2210-F (2025) irs.gov/pub/irs-pdf/i2210f.pdf
//   (no 2026 instructions yet; dates carried forward); Instructions for Form 2210 (2025),
//   "Farmers and fishers"; Publication 505 (2026) chapter 2; 26 U.S.C. 6654(a), (e), (f), (g),
//   (i); Rev. Rul. 2026-15 for the rates.
use crate::dates::ordinal;
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::kernel::source_fact::SourceFact;
use crate::line_emission::emit as emit_lines;
pub use crate::underpayment_model::{
    AnnualizedIncome, AnnualizedPeriod, DatedPayment, PenaltyWaiver, PenaltyWaiverReason,
    PeriodWithholding, PriorYearTax,
};
use crate::{FederalResult, FederalStatus};
use crate::{federal_rules::preferential_tax, line_emission::line_cents};
use alloc::vec::Vec;
use core::cmp::min;

// The four required-installment due dates (Part III column headings) and the
// April 15, 2027 end of every penalty computation.
const DUE: [u32; 4] = [20260415, 20260615, 20260915, 20270115];
const END: u32 = 20270415;
const FORM: &str = "https://www.irs.gov/pub/irs-dft/f2210--dft.pdf";
const INSTRUCTIONS: &str = "https://www.irs.gov/pub/irs-dft/i2210--dft.pdf";
const FORM_F: &str = "https://www.irs.gov/pub/irs-dft/f2210f--dft.pdf";
const INSTRUCTIONS_F: &str = "https://www.irs.gov/pub/irs-pdf/i2210f.pdf";
const COLUMNS: [&str; 4] = ["a", "b", "c", "d"];

// Sum actual source cents exactly without a bounded stored projection.
pub(crate) fn payment_total(
    amounts: impl IntoIterator<Item = i64>,
) -> crate::kernel::ratio::exact::Exact {
    amounts.into_iter().map(crate::kernel::ratio::exact::whole).sum()
}

pub(crate) fn amount_unavailable(result: &mut FederalResult) {
    crate::issue_localized!(
        result,
        &crate::IssueContext::Section(crate::TaxSection::Payments),
        "Federal payments",
        "PAYMENT_AMOUNT_UNREPRESENTABLE",
        "A native payment, refund or penalty amount cannot be represented exactly; review the payment sources.",
    );
    crate::federal_rules::withhold_required_source_result(result);
}

impl crate::EstimatedPaymentEntryRecord {
    fn timing(&self, farmer: bool) -> SourceFact<Vec<(u32, crate::kernel::ratio::exact::Exact)>> {
        use crate::kernel::ratio::exact::{ratio, whole};
        match self {
            Self::None => SourceFact::NotReported,
            Self::Sources { payments } => {
                SourceFact::Reported(payments.iter().map(|p| (p.date, whole(p.amount))).collect())
            }
            Self::TimingUnknown { .. } => SourceFact::Unavailable,
            Self::EqualTimely { .. } => {
                let annual = payment_total(self.funding().map(|(amount, _, _)| amount));
                // Equal actual cash installments must be expressible in cents.
                // Pub. 505 permits one January installment for qualifying farmers.
                if !farmer
                    && self
                        .funding()
                        .map(|(amount, _, _)| amount % 4)
                        .fold(0, |sum, part| (sum + part) % 4)
                        != 0
                {
                    return SourceFact::Unavailable;
                }
                SourceFact::Reported(if farmer {
                    alloc::vec![(DUE[3], annual)]
                } else {
                    DUE.iter()
                        .map(|date| (*date, &annual * ratio(1, 4).expect("nonzero divisor")))
                        .collect()
                })
            }
        }
    }
}
fn timing_or_amount_unavailable(pay: &crate::FederalPayments, farmer: bool, r: &mut FederalResult) {
    if matches!(pay.estimated.timing(farmer), SourceFact::Unavailable) {
        crate::issue_localized!(
            r,
            &crate::IssueContext::Section(crate::TaxSection::Payments),
            "Form 2210 underpayment penalty",
            "ISSUE_2210_ESTIMATED_PAYMENT_DATES",
            "Form 2210 line 11 and Form 2210-F line 12 need the date of each estimated payment; enter the payments by date.",
        );
    } else {
        amount_unavailable(r);
    }
}

impl crate::FederalPayments {
    pub(crate) fn estimated_source(&self) -> crate::kernel::ratio::exact::Exact {
        payment_total(self.estimated.funding().map(|(amount, _, _)| amount))
    }
}

impl crate::Withheld {
    pub(crate) fn from_native(w2: i64, forms_1099: i64, other: i64, medicare: i64) -> Option<Self> {
        let other = crate::kernel::ratio::checked_sum([other, medicare])?;
        let total = crate::kernel::ratio::checked_sum([w2, forms_1099, other])?;
        Some(Self {
            w2: w2.checked_mul(100)?,
            forms_1099: forms_1099.checked_mul(100)?,
            other: other.checked_mul(100)?,
            total: total.checked_mul(100)?,
        })
    }
}
// The quarterly underpayment rates, in percent, for the four rate periods of
// the 2026 penalty worksheet (Q2 2026 through Q1 2027), read from the fixture.
// A blank rate is unpublished and takes the last published rate.
pub(crate) fn rates() -> [i64; 4] {
    let table = include_str!("../../tests/fixtures/irs-underpayment-rates-2026.csv");
    let mut rates = [0; 4];
    let mut last = None;
    let rows = table.lines().filter(|row| !row.starts_with("//") && !row.starts_with("quarter,"));
    for (slot, (row, quarter)) in
        rates.iter_mut().zip(rows.zip(["2026-Q2", "2026-Q3", "2026-Q4", "2027-Q1"]))
    {
        let mut cells = row.split(',');
        assert_eq!(cells.next(), Some(quarter), "rate fixture order");
        let rate = match cells.next().unwrap_or("") {
            "" => last.expect("an earlier published rate"),
            cell => cell.parse::<i64>().expect("whole-percent rate"),
        };
        *slot = rate;
        last = Some(rate);
    }
    rates
}

// The last day of each Schedule AI period of 2026.
pub(crate) const PERIOD_ENDS: [u32; 4] = [20260331, 20260531, 20260831, 20261231];

// Facts from the finished return that Form 2210 reads, in whole dollars:
// line 1 is Form 1040 line 22, line 2 excludes household employment tax,
// withholding is Form 1040 line 25d, and the Form 1041-T allocation is in cents.
pub(crate) struct Inputs {
    pub line_1: i64,
    pub line_2: i64,
    pub household_tax: i64,
    pub line_3: i64,
    pub withholding: i64,
    pub excess_social: i64,
    pub estate_estimated: crate::kernel::ratio::exact::Exact,
    pub separate: bool,
    // At least two-thirds of 2026 gross income from farming or fishing, as
    // this return shows it (two_thirds_return_year).
    pub farming_return_year: Result<bool, &'static str>,
    // Same-invocation Form 1062 applicable liability, in cents. No election
    // is known absence; a selected election without an owner result is unknown.
    pub farmland_applicable: SourceFact<i64>,
    pub fund_credit: bool,
    // Schedule AI facts, in whole dollars: the annual return's AGI and
    // taxable income for the column (d) reconciliation, the standard
    // deduction (line 7), the nonrefundable credits allowed (with line 3,
    // line 18), each person's self-employment tax, whether Form 1040 line 16
    // is the ordinary Tax Table or Tax Computation Worksheet tax on taxable
    // income, and whether any tax Schedule AI line 16 would take is present.
    pub status: FederalStatus,
    pub agi: i64,
    pub taxable: i64,
    pub standard: i64,
    pub itemizing: bool,
    pub credits: i64,
    // The child tax credit, credit for other dependents and additional child tax credit on the
    // return, and the part of them each Schedule AI column may take (line 18).
    pub child_credit: i64,
    pub child_credit_by_column: [i64; 4],
    pub se_tax: [i64; 2],
    pub ordinary_tax: bool,
    pub other_taxes: bool,
}

impl Inputs {
    // Notice 2026-3 section 4 excludes 75% of applicable net tax liability
    // from the required annual payment. The 2025 Form 2210 line 3 and
    // Form 2210-F line 4 include that reduction with refundable credits.
    // https://www.irs.gov/irb/2026-02_IRB#NOT-2026-3
    fn payment_credits_cents(&self) -> Option<i64> {
        let relief = match self.farmland_applicable {
            SourceFact::NotReported => 0,
            SourceFact::Reported(amount) if amount >= 0 => {
                // Source cents times 3/4, rounded once to the line's dollars.
                // This is not the residual after Form 1062's rounded quarter.
                crate::kernel::ratio::checked_fraction(amount, 3, 400)?
            }
            _ => return None,
        };
        crate::kernel::ratio::checked_sum([self.line_3, relief])?.checked_mul(100)
    }
}

// Schedule AI Part I by column, in cents: lines 1, 3, 6, 11, 13, 14, 15
// (Part II line 36) and 21, in that order.
type Annualized = [[i64; 4]; 8];
// Lines 23, 25 and 27 by column.
type AnnualizedInstallments = [[i64; 4]; 3];

struct Worksheet {
    // Part I lines 1 to 9 in cents; line 8 is None when skipped.
    part_1: [Option<i64>; 9],
    // Schedule AI when box C is elected: lines 1 to 21 and lines 23 to 27.
    schedule_ai: Option<(Annualized, AnnualizedInstallments)>,
    // Part III lines 10, 11 and 17 by column, when Part III is reached.
    part_3: Option<[[i64; 4]; 3]>,
    penalty: i64,
    // The resource of the reason no penalty applies; empty when one does.
    stop: &'static str,
}

// Form 2210-F Part II lines 1-13 in cents (index 4, line 5, is reserved and
// stays None; a line after a stop is None) and line 16; `stop` is the
// resource of the reason no penalty applies, empty when one does.
struct FarmerSheet {
    part_2: [Option<i64>; 13],
    penalty: i64,
    stop: &'static str,
}

// The farmer and fisherman test (Form 2210 instructions, "Farmers and
// fishers"): two-thirds of 2025 gross income (the filer's answer) or of 2026
// gross income (this return). A filer who meets it figures the penalty on
// Form 2210-F; every other filer uses Form 2210.
fn farmer_2210(pay: &crate::FederalPayments, input: &Inputs) -> bool {
    input.farming_return_year == Ok(true) || pay.farmer_fisherman_prior_year
}

// The farmer and fisherman two-thirds test for 2026, figured from this return
// (Pub. 505 (2026), chapter 2, "Farming and Fishing"), in cents. `farming` is
// gross income from farming or fishing: each Schedule F line 9, Form 1041
// K-1 box 14 code F, recognized livestock gains, and each
// commercial fishing Schedule C line 7, each counted when it is income, not
// a loss. Missing partnership/S-corporation gross stays unavailable.
// `gross` is the total of the return's income (not loss) amounts,
// both spouses together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TwoThirds {
    pub farming: i64,
    pub gross: i64,
}
impl TwoThirds {
    pub(crate) fn qualifies(self) -> bool {
        self.farming > 0 && i128::from(self.farming) * 3 >= i128::from(self.gross) * 2
    }
}

// `other_income` is, in cents, the return's income (not loss) amounts other
// than sole-proprietor businesses, farm rentals and estate/trust activity
// income. Direct rental/royalty/personal-rent receipts are already gross.
// Each business contributes its own gross; `schedule_e` is the existing rental
// owner's per-source native NET dollars map. Only estate/trust entries read it,
// with their actual code F farming subset retained once.
#[cfg(test)]
pub(crate) fn two_thirds_return_year(
    d: &crate::FederalDraft,
    r: &FederalResult,
    other_income: Option<i64>,
    schedule_e: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Result<TwoThirds, &'static str>, crate::Section179PreparedIssue> {
    let source_claims = crate::related_forms::effective_claimants(d);
    two_thirds_return_year_with_claims(d, r, other_income, schedule_e, &source_claims)
}

pub(crate) fn two_thirds_return_year_with_claims(
    d: &crate::FederalDraft,
    r: &FederalResult,
    other_income: Option<i64>,
    schedule_e: &alloc::collections::BTreeMap<i64, i64>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Result<TwoThirds, &'static str>, crate::Section179PreparedIssue> {
    let mut routed = alloc::collections::BTreeMap::new();
    for entry in &crate::community::entries(d) {
        if matches!(&entry.income, crate::FederalIncome::Business { data } if matches!(data.activity, crate::BusinessActivity::Ordinary { .. }))
        {
            routed.insert(
                entry.id,
                crate::federal_rules::farm_routed_income_with_claims(
                    d,
                    usize::from(entry.owner == crate::DocumentOwner::Spouses),
                    entry.id,
                    source_claims,
                )?,
            );
        }
    }
    let farm_property_gross =
        crate::asset_sale::farming_property_gross_with_claims(d, source_claims)?;
    Ok((|| {
        // Outside the reconciled native statement cohort, K-1 net boxes and
        // K-3 foreign-tax replacements do not establish farmer/fisher gross.
        // The source producer owns basis/risk/PAL attribution and reconciliation.
        if d.income.iter().any(|entry| {
            matches!(
                entry.income,
                crate::FederalIncome::Partnership { .. }
                    | crate::FederalIncome::Scorp { .. }
                    | crate::FederalIncome::ScorpActivities { .. }
            )
        }) {
            return Err("ISSUE_FARMER_GROSS_SOURCE_UNAVAILABLE");
        }
        let capacity = "ISSUE_FARMER_GROSS_AMOUNT_UNAVAILABLE";
        let mut farming = farm_property_gross?
            .checked_add(crate::section1231::earlier_farmer_gross(d)?)
            .ok_or(capacity)?;
        let mut gross = other_income.ok_or(capacity)?;
        for entry in &crate::community::entries(d) {
            match &entry.income {
                crate::FederalIncome::Business { data } => {
                    let amount = match &data.activity {
                        crate::BusinessActivity::Farming { .. } => {
                            let id = alloc::format!("farm_{}_gross_income", entry.id);
                            let line = r.lines.iter().find(|l| l.id == id).map_or(0, |l| l.amount);
                            farming = farming.checked_add(positive(line)).ok_or(capacity)?;
                            line
                        }
                        crate::BusinessActivity::Ordinary { .. } => {
                            let line = data.receipts() + routed[&entry.id] - data.inventory.cost();
                            if data.commercial_fishing {
                                farming = farming.checked_add(positive(line)).ok_or(capacity)?;
                            }
                            line
                        }
                        crate::BusinessActivity::Gambling { .. } => data.receipts(),
                    };
                    gross = gross.checked_add(positive(amount)).ok_or(capacity)?;
                }
                // Form 4835 line 7 is farming gross income; the rental's net is not
                // in `schedule_e` (the caller takes it out) so its gross counts
                // here instead (Schedule E line 42, Pub. 505).
                crate::FederalIncome::FarmRental { .. } => {
                    let id = alloc::format!("farm_rental_{}_gross_income", entry.id);
                    let line = r.lines.iter().find(|l| l.id == id).map_or(0, |l| l.amount);
                    farming = farming.checked_add(positive(line)).ok_or(capacity)?;
                    gross = gross.checked_add(positive(line)).ok_or(capacity)?;
                }
                crate::FederalIncome::PartnershipActivities { data } => {
                    // The native statement owner reconciles this cohort's actual
                    // SE category and gross. Retain its positive Schedule E amount
                    // beside code B gross, as in Publication 505's farmer test.
                    let source = data
                        .ordinary_self_employment()
                        .map_err(|_| "ISSUE_FARMER_GROSS_SOURCE_UNAVAILABLE")?
                        .ok_or("ISSUE_FARMER_GROSS_SOURCE_UNAVAILABLE")?;
                    let net = schedule_e
                        .get(&entry.id)
                        .ok_or("ISSUE_FARMER_GROSS_SOURCE_UNAVAILABLE")?
                        .checked_mul(100)
                        .ok_or(capacity)?;
                    if source.category == 0 {
                        let farm = crate::community::income_amount(d, entry.id, source.gross);
                        farming = farming.checked_add(farm).ok_or(capacity)?;
                        gross = gross.checked_add(farm).ok_or(capacity)?;
                    }
                    gross = gross.checked_add(positive(net)).ok_or(capacity)?;
                }
                crate::FederalIncome::EstateTrustPortfolio { data } => {
                    let data = &crate::FederalEstateTrustPortfolioK1::from(data);
                    // Pub.225 chapter15 uses taxable NET estate/trust income,
                    // unlike gross direct rents/royalties. Keep actual code F as
                    // its existing subset, never gross up the fiduciary's expenses.
                    let net = match schedule_e.get(&entry.id) {
                        Some(amount) => amount.checked_mul(100).ok_or(capacity)?,
                        None if data.activities.activities.is_empty() => 0,
                        None => return Err("ISSUE_FARMER_GROSS_SOURCE_UNAVAILABLE"),
                    };
                    let farm = data.gross_farming_fishing_income;
                    farming = farming.checked_add(farm).ok_or(capacity)?;
                    let other = net.checked_sub(farm).ok_or(capacity)?;
                    gross = crate::kernel::ratio::checked_sum([gross, positive(other), farm])
                        .ok_or(capacity)?;
                }
                _ => {}
            }
        }
        Ok(TwoThirds { farming, gross })
    })())
}

// Section 6654(e)(2): a full 12-month 2025 year with no tax.
fn prior_year_zero(prior: &PriorYearTax) -> bool {
    prior.full_year && prior.tax + prior.spouse_tax == 0
}

// Form 2210 (regular method or Schedule AI), or Form 2210-F when the farmer
// and fisherman test holds. None when no 2025 facts are entered or the
// situation is refused; otherwise the penalty in cents and the explanation of
// Form 1040 line 38.
pub(crate) fn calculate(
    pay: &crate::FederalPayments,
    input: &Inputs,
    r: &mut FederalResult,
) -> Option<(i64, crate::Message)> {
    let prior = pay.prior_year.as_ref()?;
    #[cfg(test)]
    let section = "Form 2210 underpayment penalty";
    let refused = core::cell::Cell::new(false);
    let refuse = |r: &mut FederalResult, ok: bool, key: &str, #[cfg(test)] message: &str| {
        if !ok {
            let context = match key {
                "ISSUE_2210_ESTIMATED_PAYMENT_DATES" | "ISSUE_2210_WITHHOLDING_PERIODS" => {
                    crate::IssueContext::Section(crate::TaxSection::Payments)
                }
                "ISSUE_2210_AI_FARMLAND_PERIODS"
                | "ISSUE_2210_SCHEDULE_AI_PERIODS"
                | "ISSUE_2210_AI_ITEMIZED_ROUTE"
                | "ISSUE_2210_AI_FULL_YEAR_MATCH"
                | "ISSUE_2210_AI_SE_FULL_YEAR_MATCH" => {
                    crate::IssueContext::Section(crate::TaxSection::IncomeByPeriod)
                }
                "ISSUE_2210_FARMLAND_ELECTION" => {
                    crate::IssueContext::Section(crate::TaxSection::Farming)
                }
                _ => crate::IssueContext::Underpayment.manual(),
            };
            crate::issue_localized!(r, &context, section, key, message);
            refused.set(true);
        }
    };
    refuse(
        r,
        !matches!(input.farmland_applicable, SourceFact::Unavailable),
        "ISSUE_2210_FARMLAND_ELECTION",
        #[cfg(test)]
        "The selected section 1062 farmland election needs its applicable net tax liability before Form 2210 line 3 or Form 2210-F line 4 can be calculated.",
    );
    refuse(
        r,
        !input.fund_credit,
        "ISSUE_2210_FUND_CREDIT_DATE",
        #[cfg(test)]
        "Form 2439 tax paid by a fund or REIT has no payment date on Form 2210 line 11 or Form 2210-F line 12 in this App.",
    );
    // A farmer or fisherman has one installment, due January 15, 2027
    // (section 6654(i)(1)): Form 2210-F, with no box C or D, no Schedule AI,
    // and the March 1 rule in place of the January 31 rule.
    if farmer_2210(pay, input) {
        if refused.get() {
            return None;
        }
        return form_2210_f(pay, prior, input, r);
    }
    refuse(
        r,
        !pay.annualized
            || !matches!(input.farmland_applicable, SourceFact::Reported(amount) if amount > 0),
        "ISSUE_2210_AI_FARMLAND_PERIODS",
        #[cfg(test)]
        "Schedule AI needs the applicable section 1062 farmland tax liability attributable to each income period; the annual amount cannot establish those period amounts.",
    );
    refuse(
        r,
        !matches!(&pay.estimated, crate::EstimatedPaymentEntryRecord::Sources { payments } if payments.iter().any(|p| (20270116..=20270131).contains(&p.date))),
        "ISSUE_2210_JANUARY_PAYMENT_EXCEPTION",
        #[cfg(test)]
        "Form 2210 line 11 column (d): a balance paid January 16-31, 2027 may be treated as paid on the January 15 due date if you file your return and pay in full by January 31; this App charges interest to the actual payment date instead of applying that rule.",
    );
    let Some(line_6) = crate::kernel::ratio::checked_sum([input.withholding, input.excess_social])
    else {
        amount_unavailable(r);
        return None;
    };
    let periods = if pay.withholding_by_date {
        let entries = pay.withholding_periods.as_ref();
        refuse(
            r,
            entries.is_some_and(|v| {
                crate::kernel::ratio::checked_sum(v.amounts()).map(dollars) == Some(line_6)
            }),
            "ISSUE_2210_WITHHOLDING_PERIODS",
            #[cfg(test)]
            "Form 2210 Part II box D: withholding on the dates actually withheld needs the four period amounts, adding up to Form 1040 line 25d plus excess Social Security.",
        );
        entries.map(PeriodWithholding::amounts)
    } else {
        None
    };
    let annualized = if pay.annualized {
        let facts = pay.annualized_income.as_ref();
        refuse(
            r,
            facts.is_some(),
            "ISSUE_2210_SCHEDULE_AI_PERIODS",
            #[cfg(test)]
            "Form 2210 Part II box C: the annualized income installment method needs Schedule AI's income and deductions for each period.",
        );
        facts.map(|facts| {
            let (lines, message) = annualize(facts, input);
            if let Some((key, _message)) = message {
                #[cfg(test)]
                let message = _message;

                refuse(
                    r,
                    false,
                    key,
                    #[cfg(test)]
                    message,
                );
            }
            lines
        })
    } else {
        None
    };
    if refused.get() {
        return None;
    }
    let Some(line_6_cents) = line_6.checked_mul(100) else {
        amount_unavailable(r);
        return None;
    };
    let figure = |line_2: i64| {
        worksheet(pay, prior, input, line_2, line_6_cents, periods, annualized.as_ref())
    };
    let Some(mut sheet) = figure(input.line_2 + input.household_tax) else {
        timing_or_amount_unavailable(pay, false, r);
        return None;
    };
    // Household employment tax joins line 2 unless the filer had no income tax
    // withheld and, without it, no estimated payment was required: section 3510(b)(2)(B)
    // (no addition to tax by reason of section 6654(e): the balance under $1,000 or no tax
    // for a full 2025 year), the instructions' line 2 footnote.
    if input.withholding == 0 && input.household_tax > 0 {
        let Some(without) = figure(input.line_2) else {
            amount_unavailable(r);
            return None;
        };
        if without.part_1[6].is_some_and(|v| v < 100_000) || prior_year_zero(prior) {
            sheet = without;
        }
    }
    // A zero ordinary penalty is safe regardless of farmer qualification.
    if let Err(key) = input.farming_return_year
        && !pay.farmer_fisherman_prior_year
        && !prior_year_zero(prior)
        && sheet.penalty > 0
    {
        refuse(
            r,
            false,
            key,
            #[cfg(test)]
            "The current farmer or fisher gross-income test needs complete source-owner gross statements and the actual farming or fishing character of recognized gains before other sources' losses. Net income or an earlier sale's business-property label cannot replace those facts. A qualifying prior-year farmer or fisher answer remains independent.",
        );
        return None;
    }
    let waived = waived_cents(pay.waiver.as_ref(), sheet.penalty);
    emit(&sheet, waived, r);
    // A blank 2025 AGI takes the 110% safe harbor of line 8 (conservative);
    // the note appears only when line 8 was figured and reached line 9.
    if prior.full_year && prior.agi.is_none() && sheet.stop.is_empty() {
        crate::federal_rules::note_once_scoped(
            r,
            msg!("WARN_PRIOR_AGI_ASSUMED_HIGH"),
            crate::IssueContext::Section(crate::TaxSection::Payments),
        );
    }
    if let Some(waiver) = &pay.waiver {
        waiver_request(waiver, false, r);
    }
    let explanation = if sheet.schedule_ai.is_some() {
        msg!("EXPLAIN_1040_38")
    } else {
        msg!("EXPLAIN_1040_38_2")
    };
    Some((sheet.penalty - waived, explanation))
}

// The penalty a waiver request takes off line 19 (Form 2210) or line 16
// (Form 2210-F), in cents: a full request all of it, a partial one the amount
// requested capped at the penalty. The instructions have the filer subtract
// it; the IRS decides the waiver.
fn waived_cents(waiver: Option<&PenaltyWaiver>, penalty: i64) -> i64 {
    match waiver {
        None => 0,
        Some(PenaltyWaiver::Full { .. }) => penalty,
        Some(PenaltyWaiver::Partial { amount, .. }) => phase::bound(*amount, 0, penalty),
    }
}

// Part II box A/B (Form 2210) or Part I box A (Form 2210-F): a waiver request
// reduces line 19 (line 16) by the amount waived (see `waived_cents`); this
// adds the review note (and, for a partial request, the requested amount as a
// separate line) reminding the filer to attach the explanation.
fn waiver_request(waiver: &PenaltyWaiver, farmer: bool, r: &mut FederalResult) {
    let (letter, reason, explanation, amount) = match waiver {
        PenaltyWaiver::Full { reason, explanation } => ("A", reason, explanation, None),
        PenaltyWaiver::Partial { reason, explanation, amount } => {
            ("B", reason, explanation, Some(*amount))
        }
    };
    let (reason_key, _reason_label) = waiver_reason_label(*reason);
    #[cfg(test)]
    let reason_label = _reason_label;

    // Form 2210-F has the one box A for both kinds of request.
    let note = if farmer {
        crate::Message::new(
            "NOTE_2210F_WAIVER_REQUESTED",
            alloc::vec![
                crate::MessageArg::Resource(reason_key.into()),
                crate::MessageArg::Text(explanation.clone()),
            ],
        )
    } else {
        crate::Message::new(
            "NOTE_2210_WAIVER_REQUESTED",
            alloc::vec![
                crate::MessageArg::Text(letter.into()),
                crate::MessageArg::Resource(reason_key.into()),
                crate::MessageArg::Text(explanation.clone()),
            ],
        )
    };
    r.review_notes.push(crate::federal_model::ReviewNote::localized(
        #[cfg(test)]
        if farmer {
            alloc::format!(
                "Form 2210-F Part I box A waiver requested: {reason_label} - {explanation}; attach the explanation and any documentation; the IRS decides the waiver; the penalty after the amount you ask to waive is shown, and the computed penalty is its own row.",
            )
        } else {
            alloc::format!(
                "Form 2210 Part II box {letter} waiver requested: {reason_label} - {explanation}; attach the explanation and any documentation; the IRS decides the waiver; the penalty after the amount you ask to waive is shown, and the computed penalty is its own row.",
            )
        },
        note,
    ));
    if let Some(amount) = amount {
        if farmer {
            line_cents(
                r,
                "form_2210_f_waiver_requested_amount",
                amount,
                &["payments.waiver"],
                INSTRUCTIONS_F,
                msg!("EXPLAIN_FORM_2210_F_WAIVER_AMOUNT"),
            );
        } else {
            line_cents(
                r,
                "form_2210_waiver_requested_amount",
                amount,
                &["payments.waiver"],
                INSTRUCTIONS,
                msg!("EXPLAIN_FORM_N_WAIVER_REQUESTED_AMOUNT"),
            );
        }
    }
}

// Form 2210-F, for a farmer or fisherman (section 6654(i)(1)): the penalty in
// cents and Form 1040 line 38's explanation.
fn form_2210_f(
    pay: &crate::FederalPayments,
    prior: &PriorYearTax,
    input: &Inputs,
    r: &mut FederalResult,
) -> Option<(i64, crate::Message)> {
    let Some(line_8) = crate::kernel::ratio::checked_sum([input.withholding, input.excess_social])
        .and_then(|value| value.checked_mul(100))
    else {
        amount_unavailable(r);
        return None;
    };
    let figure = |line_2: i64| farmer_sheet(pay, prior, input, line_2, line_8);
    let Some(mut sheet) = figure(input.line_2 + input.household_tax) else {
        timing_or_amount_unavailable(pay, true, r);
        return None;
    };
    // The same section 3510(b)(2)(B) exclusion as Form 2210 line 2 (Form 2210-F line 2).
    if input.withholding == 0 && input.household_tax > 0 {
        let Some(without) = figure(input.line_2) else {
            amount_unavailable(r);
            return None;
        };
        if without.part_2[8].is_none_or(|v| v < 100_000) || prior_year_zero(prior) {
            sheet = without;
        }
    }
    let waived = waived_cents(pay.waiver.as_ref(), sheet.penalty);
    emit_2210_f(&sheet, waived, r);
    // Form 2210-F has neither box C nor box D: answers saved while the
    // return was not a farmer's are not used.
    if pay.annualized || pay.withholding_by_date {
        crate::federal_rules::note_once_scoped(
            r,
            msg!("NOTE_2210F_PERIOD_ANSWERS_NOT_USED"),
            crate::IssueContext::Section(crate::TaxSection::Payments),
        );
    }
    if let Some(waiver) = &pay.waiver {
        waiver_request(waiver, true, r);
    }
    Some((sheet.penalty - waived, msg!("EXPLAIN_1040_38_3")))
}

// Form 2210-F Part II and Part III. Line 1-2 add to line 3; line 7 is 66⅔% of
// the current year tax (line 6); line 10 is 100% of the 2025 tax, never 110%
// (section 6654(i)(1)(C) disregards section 6654(d)(1)(C)), and is blank when
// the 2025 return did not cover 12 months. The one required installment is
// due January 15, 2027, so Part III is column (d) alone: `part_3` with the
// whole required annual payment in the fourth column, all 2026 withholding
// and the Form 1041-T allocation paid by that date (section 6654(g)(1)).
// `line_8` is withholding in cents.
fn farmer_sheet(
    pay: &crate::FederalPayments,
    prior: &PriorYearTax,
    input: &Inputs,
    line_2: i64,
    line_8: i64,
) -> Option<FarmerSheet> {
    let mut part_2 = [None; 13];
    let halt = |part_2: [Option<i64>; 13], stop: &'static str| {
        stopped(FarmerSheet { part_2, penalty: 0, stop }, pay)
    };
    let (l1, l2) = (input.line_1.checked_mul(100)?, line_2.checked_mul(100)?);
    let l3 = crate::kernel::ratio::checked_sum([l1, l2])?;
    part_2[..3].copy_from_slice(&[Some(l1), Some(l2), Some(l3)]);
    if l3 < 100_000 {
        return Some(halt(part_2, "EXPLAIN_FORM_2210_F_NO_PENALTY_3"));
    }
    let l4 = input.payment_credits_cents()?;
    let l6 = positive(l3 - l4);
    part_2[3] = Some(l4);
    part_2[5] = Some(l6);
    if l6 < 100_000 {
        return Some(halt(part_2, "EXPLAIN_FORM_2210_F_NO_PENALTY_6"));
    }
    let l7 = fraction(l6, 2, 3);
    let l9 = l6 - line_8;
    part_2[6] = Some(l7);
    part_2[7] = Some(line_8);
    part_2[8] = Some(l9);
    if l9 < 100_000 {
        return Some(halt(part_2, "EXPLAIN_FORM_2210_F_NO_PENALTY_9"));
    }
    let l10 = prior.full_year.then(|| prior.tax + prior.spouse_tax);
    let l11 = l10.map_or(l7, |v| phase::cap(l7, [v]));
    part_2[9] = l10;
    part_2[10] = Some(l11);
    if line_8 >= l11 {
        return Some(halt(part_2, "EXPLAIN_FORM_2210_F_NO_PENALTY_11"));
    }
    let mut stream = pay.estimated.timing(true).require(|| ()).ok()?.unwrap_or_default();
    stream.push((DUE[3], crate::kernel::ratio::exact::whole(line_8)));
    if input.estate_estimated > crate::kernel::ratio::exact::whole(0) {
        stream.push((DUE[3], input.estate_estimated.clone()));
    }
    stream.sort_by_key(|v| v.0);
    let (lines, penalty) = part_3(&stream, [0, 0, 0, l11], rates())?;
    part_2[11] = Some(crate::kernel::ratio::checked_sum(lines[1])?);
    part_2[12] = Some(lines[2][3]);
    let stop = if lines[2][3] == 0 { "EXPLAIN_FORM_2210_F_NO_PENALTY_13" } else { "" };
    Some(stopped(FarmerSheet { part_2, penalty, stop }, pay))
}

// Filing the 2026 return and paying the whole balance by March 1, 2027 removes
// the addition for the installment (section 6654(i)(1)(D)); the lines already
// figured stay.
fn stopped(sheet: FarmerSheet, pay: &crate::FederalPayments) -> FarmerSheet {
    if pay.farmer_paid_by_march {
        FarmerSheet { penalty: 0, stop: "EXPLAIN_FORM_2210_NO_PENALTY", ..sheet }
    } else {
        sheet
    }
}

fn emit_2210_f(sheet: &FarmerSheet, waived: i64, r: &mut FederalResult) {
    // Line 5 is reserved on the form and has no row.
    let text: [Option<crate::Message>; 13] = [
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_1")),
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_2")),
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_3")),
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_4")),
        None,
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_6")),
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_7")),
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_8")),
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_9")),
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_10")),
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_11")),
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_12")),
        Some(msg!("EXPLAIN_FORM_2210_F_LINE_13")),
    ];
    for (index, (value, explanation)) in sheet.part_2.iter().zip(text).enumerate() {
        if let (Some(value), Some(explanation)) = (value, explanation) {
            line_cents(
                r,
                &alloc::format!("form_2210_f_line_{}", index + 1),
                *value,
                &["payments.prior_year", "total_tax", "payments"],
                INSTRUCTIONS_F,
                explanation,
            );
        }
    }
    let line_16 = if sheet.stop.is_empty() {
        msg!("EXPLAIN_FORM_2210_F_LINE_16")
    } else {
        crate::Message::new(sheet.stop, alloc::vec![])
    };
    line_cents(
        r,
        "form_2210_f_line_16",
        sheet.penalty - waived,
        &["payments.prior_year", "total_tax", "payments", "payments.estimated"],
        FORM_F,
        line_16,
    );
    if waived > 0 {
        line_cents(
            r,
            "form_2210_f_penalty_before_waiver",
            sheet.penalty,
            &["form_2210_f_line_13"],
            INSTRUCTIONS_F,
            msg!("EXPLAIN_FORM_2210_F_PENALTY_BEFORE_WAIVER"),
        );
    }
}

fn worksheet(
    pay: &crate::FederalPayments,
    prior: &PriorYearTax,
    input: &Inputs,
    line_2: i64,
    line_6: i64,
    periods: Option<[i64; 4]>,
    annualized: Option<&Annualized>,
) -> Option<Worksheet> {
    let mut part_1 = [None; 9];
    let (l1, l2, l3) =
        (input.line_1.checked_mul(100)?, line_2.checked_mul(100)?, input.payment_credits_cents()?);
    let l4 = positive(crate::kernel::ratio::checked_sum([l1, l2, -l3])?);
    let l5 = fraction(l4, 90, 100);
    let l7 = l4 - line_6;
    let threshold = if input.separate { 7_500_000 } else { 15_000_000 };
    // Box E: a joint filer whose 2025 filing status differed adds the
    // spouse's separately entered 2025 tax before applying the 100%/110% test.
    let l8 = prior.full_year.then(|| {
        fraction(
            prior.tax + prior.spouse_tax,
            if prior.agi.is_none_or(|agi| agi > threshold) { 110 } else { 100 },
            100,
        )
    });
    let l9 = l8.map_or(l5, |v| phase::cap(l5, [v]));
    part_1[..7].copy_from_slice(&[
        Some(l1),
        Some(l2),
        Some(l3),
        Some(l4),
        Some(l5),
        Some(line_6),
        Some(l7),
    ]);
    part_1[7] = l8;
    part_1[8] = Some(l9);
    let schedule_ai = annualized.map(|lines| (*lines, installments(&lines[7], l9)));
    let stop = if l4 < 100_000 {
        "EXPLAIN_FORM_2210_NO_PENALTY_2"
    } else if l7 < 100_000 {
        "EXPLAIN_FORM_2210_NO_PENALTY_3"
    } else if l9 <= line_6 {
        "EXPLAIN_FORM_2210_NO_PENALTY_4"
    } else {
        ""
    };
    if !stop.is_empty() {
        return Some(Worksheet { part_1, schedule_ai, part_3: None, penalty: 0, stop });
    }
    let withheld = periods.unwrap_or([fraction(line_6, 1, 4); 4]);
    let mut stream = pay.estimated.timing(false).require(|| ()).ok()?.unwrap_or_default();
    stream.extend(DUE.iter().copied().zip(withheld.map(crate::kernel::ratio::exact::whole)));
    if input.estate_estimated > crate::kernel::ratio::exact::whole(0) {
        stream.push((DUE[3], input.estate_estimated.clone()));
    }
    stream.sort_by_key(|v| v.0);
    // Box C: Schedule AI line 27 replaces 25% of line 9 in every column.
    let line_10 = schedule_ai.map_or([fraction(l9, 1, 4); 4], |(_, v)| v[2]);
    let (part_3, penalty) = part_3(&stream, line_10, rates())?;
    Some(Worksheet { part_1, schedule_ai, part_3: Some(part_3), penalty, stop })
}

// Section A applies each payment, in date order, first to the oldest unpaid
// installment; the remainder is an overpayment carried to later columns. The
// penalty worksheet charges each unpaid amount from its due date to the date
// it was paid, or April 15, 2027, split across the four rate periods.
fn part_3(
    stream: &[(u32, crate::kernel::ratio::exact::Exact)],
    line_10: [i64; 4],
    rates: [i64; 4],
) -> Option<([[i64; 4]; 3], i64)> {
    let mut lines = [line_10, [0; 4], [0; 4]];
    let mut unpaid: Vec<(u32, crate::kernel::ratio::exact::Exact)> = Vec::new();
    // Keep payment portions and carried balances exact until worksheet output
    // or the prescribed per-payment, per-rate-period penalty rounding.
    use crate::kernel::ratio::exact::{rounded, whole};
    let mut credit = whole(0);
    let mut penalty = 0_i128;
    let mut next = 0;
    let mut pay = |date: u32,
                   amount: &crate::kernel::ratio::exact::Exact,
                   unpaid: &mut Vec<(u32, crate::kernel::ratio::exact::Exact)>,
                   credit: &mut crate::kernel::ratio::exact::Exact| {
        let mut left = amount.clone();
        while left > whole(0) && !unpaid.is_empty() {
            let applied = phase::cap(left.clone(), [unpaid[0].1.clone()]);
            penalty += i128::from(accrue(&applied, unpaid[0].0, date, rates)?);
            left -= &applied;
            unpaid[0].1 -= applied;
            if unpaid[0].1 == whole(0) {
                unpaid.remove(0);
            }
        }
        *credit += left;
        Some(())
    };
    for (column, due) in DUE.iter().enumerate() {
        let installment = line_10[column];
        let mut paid = whole(0);
        while next < stream.len() && stream[next].0 <= *due {
            paid += &stream[next].1;
            pay(stream[next].0, &stream[next].1, &mut unpaid, &mut credit)?;
            next += 1;
        }
        lines[1][column] = rounded(&paid)?;
        let applied = phase::cap(credit.clone(), [whole(installment)]);
        credit -= &applied;
        let remaining = whole(installment) - applied;
        lines[2][column] = rounded(&remaining)?;
        if remaining > whole(0) {
            unpaid.push((*due, remaining));
        }
    }
    for (date, amount) in &stream[next..] {
        pay(*date, amount, &mut unpaid, &mut credit)?;
    }
    for (due, amount) in unpaid.drain(..) {
        penalty += i128::from(accrue(&amount, due, END, rates)?);
    }
    Some((lines, i64::try_from(penalty).ok()?))
}

// Schedule AI Part I lines 1 to 21 and Part II for each column, with the
// refusal that stops the annualized method, if any. Every column uses the
// annual standard deduction (line 7) and the annual credits (line 18); lines 12
// and 16 are zero because other taxes are refused.
fn annualize(
    facts: &AnnualizedIncome,
    input: &Inputs,
) -> (Annualized, Option<(&'static str, &'static str)>) {
    // Lines 2 and 5; lines 20, 29, 32 and 34 of the 2026 draft form.
    const ANNUALIZE: [(i64, i64); 4] = [(4, 1), (12, 5), (3, 2), (1, 1)];
    const APPLICABLE: [i64; 4] = [225, 450, 675, 900];
    const WAGE_BASE: [i64; 4] = [
        crate::kernel::ratio::trunc_i64(crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS, 4),
        crate::kernel::ratio::trunc_i64(
            crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS * 5,
            12,
        ),
        crate::kernel::ratio::trunc_i64(
            crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS * 2,
            3,
        ),
        crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS,
    ];
    const SOCIAL: [i64; 4] = [4960, 2976, 1860, 1240];
    const MEDICARE: [i64; 4] = [1160, 696, 435, 290];
    let mut lines = [[0; 4]; 8];
    let other_credits = positive(input.credits + input.line_3 - input.child_credit) * 100;
    for (column, period) in facts.periods().into_iter().enumerate() {
        let (times, per) = ANNUALIZE[column];
        let l3 = fraction(period.agi, times, per);
        let l6 = fraction(period.itemized, times, per);
        let l8 = phase::floor(l6, input.standard * 100);
        let l11 = l3 - (l8 + period.qbi + period.additional);
        let l13 = positive(l11);
        let l14 = preferential_tax(dollars(l13), 0, input.status) * 100;
        // Part II: net earnings under $400 are zero, as on Schedule SE.
        let l28 = fraction(positive(period.self_employment), 9235, 10_000);
        let l28 = if l28 < 40_000 { 0 } else { l28 };
        let l31 = positive(WAGE_BASE[column] - period.social_wages);
        let l36 = fraction(phase::cap(l28, [l31]), SOCIAL[column], 10_000)
            + fraction(l28, MEDICARE[column], 10_000);
        let line_18 = other_credits + input.child_credit_by_column[column] * 100;
        let l19 = positive(l14 + l36 - line_18);
        let l21 = fraction(l19, APPLICABLE[column], 1000);
        for (row, value) in [period.agi, l3, l6, l11, l13, l14, l36, l21].into_iter().enumerate() {
            lines[row][column] = value;
        }
    }
    let full = &facts.full_year;
    let message = if !input.itemizing && facts.periods().iter().any(|v| v.itemized != 0) {
        Some((
            "ISSUE_2210_AI_ITEMIZED_ROUTE",
            crate::fixture_english!(
                "Schedule AI line 4 takes itemized deductions only when you itemize on your return; enter 0 in every period."
            ),
        ))
    } else if full.agi != input.agi * 100 || lines[4][3] != input.taxable * 100 {
        Some((
            "ISSUE_2210_AI_FULL_YEAR_MATCH",
            crate::fixture_english!(
                "Schedule AI column (d) must match your full-year return: line 1 must equal your adjusted gross income and line 13 your taxable income."
            ),
        ))
    } else if !input.ordinary_tax {
        Some((
            "ISSUE_2210_AI_TAX_WORKSHEET",
            crate::fixture_english!(
                "Schedule AI line 14 is figured here only from the Tax Table or Tax Computation Worksheet; qualified dividends, capital gains, excluded foreign income, Form 8615 or Schedule J need their worksheets refigured for each period, which this App does not do."
            ),
        ))
    } else if input.other_taxes {
        Some((
            "ISSUE_2210_AI_OTHER_TAXES",
            crate::fixture_english!(
                "Schedule AI line 16 needs other taxes figured for each period (such as Additional Medicare Tax, net investment income tax, alternative minimum tax or household employment tax), which this App does not do."
            ),
        ))
    } else if input.se_tax.iter().all(|v| *v > 0) {
        Some((
            "ISSUE_2210_AI_TWO_SE_FILERS",
            crate::fixture_english!(
                "Schedule AI Part II is completed separately for each spouse with self-employment income; this App completes it for one person."
            ),
        ))
    } else if (lines[6][3] - input.se_tax.iter().sum::<i64>() * 100).abs() > 100 {
        Some((
            "ISSUE_2210_AI_SE_FULL_YEAR_MATCH",
            crate::fixture_english!(
                "Schedule AI Part II column (d) must match your full-year self-employment tax: check the full-year net profit and Social Security wages."
            ),
        ))
    } else {
        None
    };
    (lines, message)
}

// Schedule AI lines 22 to 27: each column takes the smaller of the annualized
// installment not yet required (line 23) and the regular installment plus
// what earlier columns saved (line 26).
fn installments(line_21: &[i64; 4], line_9: i64) -> AnnualizedInstallments {
    let line_24 = fraction(line_9, 1, 4);
    let mut lines = [[0; 4]; 3];
    let (mut line_22, mut line_25) = (0, 0);
    for column in 0..4 {
        let line_23 = positive(line_21[column] - line_22);
        let line_26 = line_24 + line_25;
        let line_27 = phase::cap(line_23, [line_26]);
        lines[0][column] = line_23;
        lines[1][column] = line_25;
        lines[2][column] = line_27;
        line_22 += line_27;
        line_25 = line_26 - line_27;
    }
    lines
}

// One penalty-worksheet computation per rate period: amount × days / 365 ×
// rate, each rounded to cents. A rate period runs from the day after its
// computation starting date through its end date.
fn accrue(
    amount: &crate::kernel::ratio::exact::Exact,
    due: u32,
    paid: u32,
    rates: [i64; 4],
) -> Option<i64> {
    use crate::kernel::ratio::exact::{ratio, rounded};
    const PERIODS: [(u32, u32); 4] =
        [(20260415, 20260630), (20260630, 20260930), (20260930, 20261231), (20261231, END)];
    let paid = min(paid, END);
    crate::kernel::ratio::checked_sum(
        PERIODS
            .iter()
            .zip(rates)
            .map(|(&(start, end), rate)| {
                let from = ordinal(start).max(ordinal(due));
                let days = i64::from(ordinal(paid).min(ordinal(end)) - from);
                if days > 0 { rounded(&(amount * ratio(days * rate, 36_500)?)) } else { Some(0) }
            })
            .collect::<Option<Vec<_>>>()?,
    )
}

fn waiver_reason_label(reason: PenaltyWaiverReason) -> (&'static str, &'static str) {
    match reason {
        PenaltyWaiverReason::RetirementOrDisability => (
            "WAIVER_REASON_RETIREMENT_OR_DISABILITY",
            "retirement after reaching age 62 or disability, with reasonable cause",
        ),
        PenaltyWaiverReason::CasualtyOrDisasterOrUnusual => (
            "WAIVER_REASON_CASUALTY_DISASTER_OR_UNUSUAL",
            "casualty, disaster, or other unusual circumstance",
        ),
        PenaltyWaiverReason::FederallyDeclaredDisaster => {
            ("WAIVER_REASON_FEDERALLY_DECLARED_DISASTER", "a federally declared disaster")
        }
    }
}
fn emit(sheet: &Worksheet, waived: i64, r: &mut FederalResult) {
    let part_1_text: [crate::Message; 9] = [
        msg!("EXPLAIN_FORM_2210_PART_1"),
        msg!("EXPLAIN_FORM_2210_PART_1_2"),
        msg!("EXPLAIN_FORM_2210_PART_1_3"),
        msg!("EXPLAIN_FORM_2210_PART_1_4"),
        msg!("EXPLAIN_FORM_2210_PART_1_6"),
        msg!("EXPLAIN_FORM_2210_PART_1_7"),
        msg!("EXPLAIN_FORM_2210_PART_1_8"),
        msg!("EXPLAIN_FORM_2210_PART_1_9"),
        msg!("EXPLAIN_FORM_2210_PART_1_10"),
    ];
    for (index, (value, explanation)) in sheet.part_1.iter().zip(part_1_text).enumerate() {
        if let Some(value) = value {
            line_cents(
                r,
                &alloc::format!("form_2210_line_{}", index + 1),
                *value,
                &["payments.prior_year", "total_tax", "payments"],
                INSTRUCTIONS,
                explanation,
            );
        }
    }
    if let Some((part_1, installments)) = &sheet.schedule_ai {
        const LINES: [(&str, &str); 11] = [
            ("1", "EXPLAIN_FORM_2210_AI_LINE_N"),
            ("3", "EXPLAIN_FORM_2210_AI_LINE_N_2"),
            ("6", "EXPLAIN_FORM_2210_AI_LINE_N_3"),
            ("11", "EXPLAIN_FORM_2210_AI_LINE_N_4"),
            ("13", "EXPLAIN_FORM_2210_AI_LINE_N_5"),
            ("14", "EXPLAIN_FORM_2210_AI_LINE_N_6"),
            ("15", "EXPLAIN_FORM_2210_AI_LINE_N_7"),
            ("21", "EXPLAIN_FORM_2210_AI_LINE_N_8"),
            ("23", "EXPLAIN_FORM_2210_AI_LINE_N_9"),
            ("25", "EXPLAIN_FORM_2210_AI_LINE_N_10"),
            ("27", "EXPLAIN_FORM_2210_AI_LINE_N_11"),
        ];
        for (values, (number, explanation)) in part_1.iter().chain(installments).zip(LINES) {
            emit_lines(r, &mut (), values.iter().zip(COLUMNS), |(value, column), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("form_2210_ai_line_{number}_{column}"),
                    LineValue::MoneyCents(*value),
                    &["payments.annualized_income"],
                    INSTRUCTIONS,
                    crate::Message::new(explanation, alloc::vec![]),
                ))
            });
        }
    }
    let line_10 = if sheet.schedule_ai.is_some() {
        msg!("EXPLAIN_FORM_2210_LINE_10_2")
    } else {
        msg!("EXPLAIN_FORM_2210_LINE_10_3")
    };
    if let Some(part_3) = &sheet.part_3 {
        for (row, (number, explanation)) in part_3.iter().zip([
            (10, line_10),
            (11, msg!("EXPLAIN_FORM_2210_LINE_11")),
            (17, msg!("EXPLAIN_FORM_2210_LINE_17")),
        ]) {
            emit_lines(r, &mut (), row.iter().zip(COLUMNS), |(value, column), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("form_2210_line_{number}_{column}"),
                    LineValue::MoneyCents(*value),
                    &["payments.estimated", "payments.withholding_periods"],
                    FORM,
                    explanation.clone(),
                ))
            });
        }
    }
    let line_19 = if sheet.stop.is_empty() {
        msg!("EXPLAIN_FORM_N_LINE_N_2")
    } else {
        crate::Message::new(sheet.stop, alloc::vec![])
    };
    line_cents(
        r,
        "form_2210_line_19",
        sheet.penalty - waived,
        &["form_2210_line_17_a", "form_2210_line_17_d"],
        INSTRUCTIONS,
        line_19,
    );
    if waived > 0 {
        line_cents(
            r,
            "form_2210_penalty_before_waiver",
            sheet.penalty,
            &["form_2210_line_17_a", "form_2210_line_17_d"],
            INSTRUCTIONS,
            msg!("EXPLAIN_FORM_N_PENALTY_BEFORE_WAIVER"),
        );
    }
}

#[cfg(test)]
#[path = "../tests/payments/exact_chronology.rs"]
mod exact_chronology;
