use crate::underpayment_model::{AnnualizedPeriod, PenaltyWaiverReason};
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};
/// Underpayment penalty
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PenaltyChoice {
    /// Let the IRS figure it and send a bill
    IrsFigures,
    /// Figure it on this return (Form 2210 or 2210-F)
    Figure {
        /// Your 2025 tax (Form 2210 line 8; Form 2210-F line 10)
        /// Your 2025 income tax after nonrefundable credits, plus self-employment tax and the other additional taxes the Form 2210 instructions list, minus refundable credits. This is not always Form 1040 line 24. If you filed jointly for 2025 but not for 2026, enter your share.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        prior_tax: i64,
        /// Your 2025 adjusted gross income (line 11)
        /// Choose “Use the conservative 110% prior-year safe harbor when applicable” if you do not know it: the App then assumes it was over $150,000 ($75,000 married filing separately) and uses 110% of your 2025 tax as the prior-year safe harbor, which may overstate the penalty. A farmer or fisherman never needs it.
        /// [NullText]
        /// Use the conservative 110% prior-year safe harbor when applicable
        /// [SetText]
        /// My 2025 adjusted gross income was
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        #[libertas_default($null)]
        prior_agi: Option<i64>,
        /// Your 2025 return covered all 12 months
        full_year: bool,
        /// Your 2025 returns
        #[libertas_default(SameReturn)]
        #[libertas_constrained_by("$.allowed_prior_returns")]
        prior_return: PriorReturn,
        /// At least two-thirds of your gross income was from farming or fishing
        /// The App figures 2026 from this return. When 2026 qualifies, that is the only choice; otherwise answer for 2025 from your 2025 return: farming is Schedule F line 9, Form 4835 line 7, farming shares on Schedules K-1 and gains from selling breeding, dairy, draft or sporting livestock (not farm wages); fishing is fishing Schedule C line 7, pay as an officer or crew member of a fishing vessel and fishing shares on Schedules K-1. Gross income is the total of your income amounts, not losses, with your spouse's on a joint return. Yes figures the penalty on Form 2210-F: one payment due January 15, 2027 of 66⅔% of your 2026 tax or 100% of your 2025 tax, whichever is smaller.
        #[libertas_constrained_by("$.allowed_farmer")]
        farmer: Farmer,
        /// Withholding
        #[libertas_default(Evenly)]
        withholding: WithholdingTiming,
        /// Income
        #[libertas_default(Evenly)]
        income: IncomeTiming,
        /// Waiver request
        /// Form 2210 box A or B; for farmers and fishers, Form 2210-F box A.
        #[libertas_default(None)]
        waiver: Waiver,
    },
}
/// Your 2025 returns
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[libertas_inline_choices]
pub enum PriorReturn {
    /// The tax above is from one 2025 return
    SameReturn,
    /// We filed separate 2025 returns (box E)
    /// The tax above is yours; enter your spouse's separate 2025 tax here.
    SeparateReturns {
        /// Your spouse's 2025 total tax
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        spouse_tax: i64,
    },
}
/// Withholding
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum WithholdingTiming {
    /// Spread evenly over the year
    Evenly,
    /// On the dates it was actually withheld (box D)
    /// Include excess Social Security or tier 1 railroad retirement tax in the period it was withheld. The four amounts must add up to your total withholding. Not for farmers and fishers: Form 2210-F has one payment due January 15, 2027.
    ByPeriod {
        /// January 1 through April 15, 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        first: i64,
        /// April 16 through June 15, 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        second: i64,
        /// June 16 through September 15, 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        third: i64,
        /// September 16, 2026 through January 15, 2027
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        fourth: i64,
    },
}
/// Income
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum IncomeTiming {
    /// Received evenly over the year
    Evenly,
    /// Mostly late in the year (box C, annualized income)
    /// You enter your income and deductions for each period next. Not for farmers and fishers: Form 2210-F has one payment due January 15, 2027.
    ByPeriod,
}
/// At least two-thirds of your gross income was from farming or fishing
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[libertas_inline_choices]
pub enum Farmer {
    /// No, not for 2025
    No,
    /// Yes, for 2025
    Yes {
        /// You filed and paid all of your 2026 tax by March 1, 2027
        /// Then no penalty applies. Choose it only when your 2026 return is filed and its whole balance paid by that date.
        paid_by_march_1: bool,
    },
    /// Yes, for 2026, as this return shows
    ThisReturn {
        /// You filed and paid all of your 2026 tax by March 1, 2027
        /// Then no penalty applies. Choose it only when your 2026 return is filed and its whole balance paid by that date.
        paid_by_march_1: bool,
    },
}
/// Waiver request (boxes A and B)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum Waiver {
    /// No waiver
    None,
    /// Waive the whole penalty (box A)
    /// This does not change the penalty shown; attach your explanation when you file.
    Full {
        /// Reason
        reason: PenaltyWaiverReason,
        /// Explanation
        #[libertas_size(min = 1)]
        explanation: String,
    },
    /// Waive part of the penalty (box B)
    /// This does not change the penalty shown; attach your explanation when you file.
    Partial {
        /// Reason
        reason: PenaltyWaiverReason,
        /// Explanation
        #[libertas_size(min = 1)]
        explanation: String,
        /// Amount to waive
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        amount: i64,
    },
}

// The income by period page, opened after the payments page
// when box C is chosen: Form 2210 Schedule AI's cumulative columns (a)-(c)
// entered by the filer, column (d) derived from the accepted return and
// ignored on save. It is a request shape; the stored record is
// FederalPayments.annualized_income.
/// Income by period
/// Your income and deductions from January 1 through the end of each period. Each period starts as your full-year amounts spread evenly; change them to what you actually received and paid.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IncomeByPeriod {
    /// Through March 31
    pub through_march: AnnualizedPeriod,
    /// Through May 31
    pub through_may: AnnualizedPeriod,
    /// Through August 31
    pub through_august: AnnualizedPeriod,
    /// All of 2026
    /// From your return.
    #[libertas_read_only]
    pub full_year: AnnualizedPeriod,
}
