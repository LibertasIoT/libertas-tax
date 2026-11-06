// form_1040_es items of src/federal/federal_model.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

/// Your 2027 estimated tax (Form 1040-ES)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum NextYearEstimate {
    /// Not needed
    NotNeeded,
    /// Figure it from this return
    /// Your 2027 income, deductions, credits, other taxes and withholding are taken to be the same as on this return.
    SameAsThisReturn {
        /// At least two-thirds of your 2027 gross income will be from farming or fishing
        /// The App figures 2026 from this return. When at least two-thirds of your 2026 gross income was from farming or fishing, Not needed is the only choice. Otherwise, Yes uses 66⅔% of your expected 2027 tax instead of 90%, never 110% of your 2026 tax, and one payment due January 18, 2028. Check your expected farming/fishing receipts against total gross income.
        #[libertas_constrained_by("$.allowed_farmer_next_year")]
        farmer_fisherman_next_year: FarmerNextYear,
    },
    /// Figure it with changes you expect in 2027
    /// Leave a field empty to use this return's amount. Self-employment tax, other taxes and the qualified business income deduction stay as on this return.
    WithChanges {
        /// Adjusted gross income you expect in 2027
        /// [NullText]
        /// Use this return's adjusted gross income
        /// [SetText]
        /// My expected 2027 adjusted gross income is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        #[libertas_default($null)]
        agi: Option<i64>,
        /// Deductions you expect in 2027
        /// Your expected itemized deductions, or your standard deduction plus your cash gifts if you will not itemize.
        /// [NullText]
        /// Use this return's deductions
        /// [SetText]
        /// My expected 2027 deductions are
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        deductions: Option<i64>,
        /// Nonrefundable credits you expect in 2027
        /// Do not include withholding or refundable credits.
        /// [NullText]
        /// Use this return's nonrefundable credits
        /// [SetText]
        /// My expected 2027 nonrefundable credits are
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        credits: Option<i64>,
        /// Income tax you expect to have withheld during 2027
        /// [NullText]
        /// Use this return's federal income tax withholding
        /// [SetText]
        /// My expected 2027 federal income tax withholding is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default($null)]
        withholding: Option<i64>,
        /// At least two-thirds of your 2027 gross income will be from farming or fishing
        /// The App figures 2026 from this return. When at least two-thirds of your 2026 gross income was from farming or fishing, Not needed is the only choice. Otherwise, Yes uses 66⅔% of your expected 2027 tax instead of 90%, never 110% of your 2026 tax, and one payment due January 18, 2028. Check your expected farming/fishing receipts against total gross income.
        #[libertas_constrained_by("$.allowed_farmer_next_year")]
        farmer_fisherman_next_year: FarmerNextYear,
    },
}
/// Accepted next-year estimate intent (without page-root catalogs)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum NextYearEstimateRecord {
    /// No estimate requested
    NotNeeded,
    /// Same expected amounts as this return
    SameAsThisReturn { farmer_fisherman_next_year: FarmerNextYear },
    /// Expected changes from this return
    WithChanges {
        agi: Option<i64>,
        deductions: Option<i64>,
        credits: Option<i64>,
        withholding: Option<i64>,
        farmer_fisherman_next_year: FarmerNextYear,
    },
}
impl From<NextYearEstimate> for NextYearEstimateRecord {
    fn from(value: NextYearEstimate) -> Self {
        match value {
            NextYearEstimate::NotNeeded => Self::NotNeeded,
            NextYearEstimate::SameAsThisReturn { farmer_fisherman_next_year } => {
                Self::SameAsThisReturn { farmer_fisherman_next_year }
            }
            NextYearEstimate::WithChanges {
                agi,
                deductions,
                credits,
                withholding,
                farmer_fisherman_next_year,
            } => Self::WithChanges {
                agi,
                deductions,
                credits,
                withholding,
                farmer_fisherman_next_year,
            },
        }
    }
}
impl From<NextYearEstimateRecord> for NextYearEstimate {
    fn from(value: NextYearEstimateRecord) -> Self {
        match value {
            NextYearEstimateRecord::NotNeeded => Self::NotNeeded,
            NextYearEstimateRecord::SameAsThisReturn { farmer_fisherman_next_year } => {
                Self::SameAsThisReturn { farmer_fisherman_next_year }
            }
            NextYearEstimateRecord::WithChanges {
                agi,
                deductions,
                credits,
                withholding,
                farmer_fisherman_next_year,
            } => Self::WithChanges {
                agi,
                deductions,
                credits,
                withholding,
                farmer_fisherman_next_year,
            },
        }
    }
}
/// Farming or fishing in 2027
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum FarmerNextYear {
    /// Yes
    Yes,
    /// No
    No,
    /// Not needed: 2026 already qualifies
    NotNeeded,
}

/// Your 2027 estimated tax
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub enum NextYearInitial {
    NotNeeded,
    SameAsThisReturn,
    WithChanges {
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        agi: Option<i64>,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        deductions: Option<i64>,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        credits: Option<i64>,
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        withholding: Option<i64>,
    },
}
