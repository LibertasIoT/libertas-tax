// form_8962 items of src/federal/federal_model.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

/// Part III monthly amounts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PolicyAmounts {
    /// The same amounts every month, January through December
    AllYear {
        /// Column A monthly enrollment premium
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        premium: i64,
        /// Column B monthly second lowest cost silver plan (SLCSP) premium
        /// Choose “Column B is blank or its applicable premium is unavailable” when column B is blank; an explicit zero remains zero. Obtain the applicable figure from the Marketplace tax tool when needed.
        /// [NullText]
        /// Column B is blank or its applicable premium is unavailable
        /// [SetText]
        /// The applicable monthly SLCSP premium is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        benchmark: Option<i64>,
        /// Column C monthly advance payment of premium tax credit
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        advance: i64,
    },
    /// Different amounts, or only some months
    #[libertas_llm_doc_field]
    ByMonth {
        /// Months on this form
        #[libertas_llm_doc_field]
        // Tax structure: one policy entry for each calendar month.
        #[libertas_size(min = 1, max = 12)]
        #[libertas_unordered]
        /// ----
        /// Month
        months: Vec<PolicyMonth>,
    },
}
/// Month
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PolicyMonth {
    /// Month
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_unique(1)]
    // Tax structure: the value counts months or days in the stated calendar period.
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Column A monthly enrollment premium
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub premium: i64,
    /// Column B monthly second lowest cost silver plan (SLCSP) premium
    /// Choose “Column B is blank or its applicable premium is unavailable” when column B is blank; an explicit zero remains zero. Obtain the applicable figure from the Marketplace tax tool when needed.
    /// [NullText]
    /// Column B is blank or its applicable premium is unavailable
    /// [SetText]
    /// The applicable SLCSP premium for this month is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub benchmark: Option<i64>,
    /// Column C monthly advance payment of premium tax credit
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub advance: i64,
}
/// Filing separately as a victim of domestic abuse or spousal abandonment
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum SeparateRelief {
    /// You are not filing separately
    NotSeparate,
    /// No
    No,
    /// Yes
    Yes,
}
/// Person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DependentReturn {
    /// Person
    #[libertas_enum_source("$.people")]
    #[libertas_unique(1)]
    pub person: i64,
    /// Their adjusted gross income (Form 1040 line 11)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub agi: i64,
    /// Their tax-exempt interest (line 2a)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt_interest: i64,
    /// Their excluded foreign income and housing (Form 2555 lines 45 and 50)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_exclusion: i64,
    /// Their Social Security benefits (line 6a)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub social: i64,
    /// Taxable part of those benefits (line 6b)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxable_social: i64,
}
