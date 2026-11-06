use crate::dates::source_valid_date;
use crate::injured_spouse_payments::PaymentPayer;
use crate::year_constants::{ESTIMATED_PAYMENT_LAST_DATE, ESTIMATED_PAYMENT_START_DATE};
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Estimated tax payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DatedPayment {
    /// Date paid
    /// Use the postmark date for a mailed payment. A 2025 overpayment applied to 2026 counts as paid April 15, 2026 when your 2025 return was fully paid by then.
    #[libertas_date_only]
    // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
    #[libertas_number(min = $ESTIMATED_PAYMENT_START_DATE, max = $ESTIMATED_PAYMENT_LAST_DATE)]
    pub date: u32,
    /// Amount paid
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Who made this payment
    pub payer: PaymentPayer,
    /// Source property for community refund allocation
    #[libertas_default($null)]
    pub refund_property: Option<crate::RefundSourcePropertyFacts>,
}

/// Your 2025 federal return
/// Used to figure the underpayment penalty for 2026 estimated tax.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PriorYearTax {
    /// Your 2025 return covered a full 12 months
    /// Choose No if you did not file a 2025 return or your 2025 tax year was shorter.
    pub full_year: bool,
    /// 2025 tax
    /// Your 2025 income tax after nonrefundable credits, plus self-employment and the other additional taxes the Form 2210 instructions list, minus refundable credits. If you filed jointly for 2025 but not for 2026, enter your share. If you file jointly for 2026 but did not file jointly with this spouse for 2025, enter your own 2025 tax here and your spouse's separate 2025 tax below.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub tax: i64,
    /// 2025 adjusted gross income
    /// Left blank when you do not know it; Form 2210 then takes the 110% prior-year safe harbor, as for an income over $150,000 ($75,000 married filing separately). A farmer or fisherman never needs it.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub agi: Option<i64>,
    /// Your spouse's 2025 tax, filed on a separate return
    /// Enter this only when you file jointly for 2026 but you and this spouse did not file jointly for 2025. It is added to the amount above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub spouse_tax: i64,
}

/// Reason for a Form 2210 penalty waiver request
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum PenaltyWaiverReason {
    /// Retirement after reaching age 62, or disability, with reasonable cause
    RetirementOrDisability,
    /// Casualty, disaster, or other unusual circumstance
    CasualtyOrDisasterOrUnusual,
    /// A federally declared disaster
    FederallyDeclaredDisaster,
}

/// Request a penalty waiver
/// This does not change the estimated penalty shown below. Attach your explanation and any documentation when you file.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PenaltyWaiver {
    /// Waive the entire penalty
    Full {
        /// Reason
        reason: PenaltyWaiverReason,
        /// Explanation
        explanation: String,
    },
    /// Waive part of the penalty
    Partial {
        /// Reason
        reason: PenaltyWaiverReason,
        /// Explanation
        explanation: String,
        /// Amount of the penalty to waive
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
}
impl PenaltyWaiver {
    pub(crate) fn invalid_field(&self) -> Option<&'static str> {
        let explanation = match self {
            PenaltyWaiver::Full { explanation, .. }
            | PenaltyWaiver::Partial { explanation, .. } => explanation,
        };
        if explanation.trim().is_empty() {
            Some("explanation")
        } else if matches!(self, PenaltyWaiver::Partial { amount, .. } if *amount <= 0) {
            Some("amount")
        } else {
            None
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.invalid_field().is_none()
    }
}

/// Federal income tax withheld in each payment period
/// Include excess Social Security or tier 1 railroad retirement tax in the period it was withheld. The four amounts must add up to your total withholding.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PeriodWithholding {
    /// January 1 through April 15, 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub first: i64,
    /// April 16 through June 15, 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub second: i64,
    /// June 16 through September 15, 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub third: i64,
    /// September 16, 2026 through January 15, 2027
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub fourth: i64,
}

/// Income and deductions from January 1 through the end of the period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AnnualizedPeriod {
    /// Adjusted gross income
    /// Income received in the period minus your adjustments to income, including the deductible part of self-employment tax and your share of partnership or S corporation items.
    #[libertas_money("USD", 2)]
    pub agi: i64,
    /// Itemized deductions paid
    /// Enter 0 if you take the standard deduction.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub itemized: i64,
    /// Qualified business income deduction
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qbi: i64,
    /// Deductions for tips, overtime, car loan interest and seniors
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub additional: i64,
    /// Net profit from self-employment
    /// Net profit from all your businesses in the period, before the 92.35% reduction. Enter 0 if none.
    #[libertas_money("USD", 2)]
    #[libertas_default(0)]
    pub self_employment: i64,
    /// Wages subject to Social Security tax
    /// Wages in the period of the person with self-employment income, including the Social Security part of railroad retirement tier 1 compensation.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub social_wages: i64,
}

/// Income and deductions by period
/// Each period starts on January 1 and includes everything from the earlier periods. The full-year amounts must match your return.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AnnualizedIncome {
    /// January 1 through March 31, 2026
    pub through_march: AnnualizedPeriod,
    /// January 1 through May 31, 2026
    pub through_may: AnnualizedPeriod,
    /// January 1 through August 31, 2026
    pub through_august: AnnualizedPeriod,
    /// January 1 through December 31, 2026
    pub full_year: AnnualizedPeriod,
}

const MAX: i64 = 100_000_000_000;

impl DatedPayment {
    pub(crate) fn valid(&self) -> bool {
        source_valid_date(self.date)
            && (ESTIMATED_PAYMENT_START_DATE..=ESTIMATED_PAYMENT_LAST_DATE).contains(&self.date)
            && self.amount > 0
    }
}

impl PriorYearTax {
    pub(crate) fn valid(&self) -> bool {
        // Arithmetic safety for the unchanged prior-tax pair and 110% safe-
        // harbor calculation. This operand class is separate from payments.
        (0..=MAX).contains(&self.tax)
            && self.agi.is_none_or(|agi| (-MAX..=MAX).contains(&agi))
            && (0..=MAX).contains(&self.spouse_tax)
    }
}
impl AnnualizedPeriod {
    fn valid(&self) -> bool {
        // Arithmetic safety for the unchanged ScheduleAI annualization, tax
        // and four-period difference calculations; these are income operands.
        [self.agi, self.self_employment].iter().all(|v| (-MAX..=MAX).contains(v))
            && [self.itemized, self.qbi, self.additional, self.social_wages]
                .iter()
                .all(|v| (0..=MAX).contains(v))
    }
}
impl AnnualizedIncome {
    pub(crate) fn periods(&self) -> [&AnnualizedPeriod; 4] {
        [&self.through_march, &self.through_may, &self.through_august, &self.full_year]
    }
    pub(crate) fn valid(&self) -> bool {
        self.periods().iter().all(|v| v.valid())
    }
}
impl PeriodWithholding {
    pub(crate) fn amounts(&self) -> [i64; 4] {
        [self.first, self.second, self.third, self.fourth]
    }
    pub(crate) fn valid(&self) -> bool {
        self.amounts().iter().all(|v| *v >= 0)
    }
}
