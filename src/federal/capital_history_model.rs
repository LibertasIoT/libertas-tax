use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Owners of a prior return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PriorReturnOwners {
    /// Taxpayer only
    Taxpayer,
    /// Current spouse only
    Spouse,
    /// Taxpayer and current spouse jointly
    Joint,
}

/// 2025 return with a regular or AMT capital loss
/// Copy the filed return's amounts. The App calculates short- and long-term carryovers; do not enter a carryover you calculated yourself.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PriorCapitalReturn {
    /// Who filed this 2025 return
    pub owners: PriorReturnOwners,
    /// Filing status on that return
    pub status: crate::FederalStatus,
    /// 2025 Form 1040 line 11b adjusted gross income
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub agi: i64,
    /// 2025 Form 1040 line 14 total deductions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deductions: i64,
    /// 2025 Schedule 1-A line 37 enhanced deduction for seniors
    /// Enter the amount already included in Form 1040 line 14 above, or zero if none. Do not include the age/blindness addition to the standard deduction. The enhanced deduction is added back only when measuring capital-loss use, not when calculating tax.
    #[libertas_money("USD", 2)]
    // Tax structure: The 2025 enhanced senior deduction is at most $6,000 per filer, $12,000 jointly.
    #[libertas_number(min = 0, max = 1200000)]
    pub senior_deduction: i64,
    /// 2025 Schedule D line 7 net short-term gain or loss
    /// Enter a loss as negative.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub short: i64,
    /// 2025 Schedule D line 15 net long-term gain or loss
    /// Enter a loss as negative.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub long: i64,
    /// Loss deducted on 2025 Schedule D line 21, entered as positive
    #[libertas_money("USD", 2)]
    // Tax structure: Schedule D limits the annual deducted capital loss to at most $3,000.
    #[libertas_number(min = 0, max = 300000)]
    pub deducted_loss: i64,
    /// Part of the loss carried to 2026 that came from section 1256 contracts
    /// Enter the part of your 2025 capital loss carryover that came from a net section 1256 contracts loss (Form 6781, Publication 550 "Net section 1256 contracts loss"), as a positive amount, or zero when none of it did. A 2026 carryback election can then count it as a 2026 section 1256 loss.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_1256: i64,
    /// Separate 2025 AMT capital-loss worksheet
    /// Supply retained AMT worksheet amounts when its carryover differs from regular tax, including an AMT-only loss. Leave absent only when the records establish equal carryovers, not when AMT records are unavailable.
    /// [NullText]
    /// AMT capital-loss carryovers equal the regular carryovers
    /// [SetText]
    /// The different AMT capital-loss worksheet is
    #[libertas_default($null)]
    pub amt: Option<PriorAmtCapitalReturn>,
    /// Retained loss balances by original loss year
    /// Optional supporting history from reconciled loss worksheets, after 2025 use. Leave empty when unavailable. Include every surviving year if supplied; totals must match the carryovers calculated from this return. Do not invent years from the aggregate Schedule D amounts. This records provenance; general canceled-debt reductions still require further worksheets.
    pub origins: Vec<CapitalLossOrigin>,
    /// These losses belong entirely to the people filing this return
    /// No allocation to a former spouse, canceled-debt basis reduction, unresolved special capital-gain category or AMT adjustment applies. Any different AMT carryover is supplied in the separate worksheet above. The prior Schedule D and any divorce or estate documents show it. No leaves this prior-year capital loss carryover out of the estimate with a notice; that situation is not supported.
    pub ordinary_history: bool,
    /// Investment-property portions of the calculated carryovers
    /// Use retained loss-source records. Absence means attribution is unavailable, not that no loss is investment-related. These portions do not change the ordinary Schedule D carryover.
    /// [NullText]
    /// Investment-property loss attribution is unavailable
    /// [SetText]
    /// Investment-property loss portions are
    #[libertas_default($null)]
    pub investment_parts: Option<InvestmentCapitalParts>,
    /// Prior Form 8960 excluded capital-loss difference
    /// Positive amount from line 2(i), column A, of the retained 2025 Form 8960 net-gains worksheet. Enter zero when that worksheet establishes none. Leave absent only when no regular loss carries forward. Section 163(d) investment-property attribution above does not establish the separate section 1411 amount. This is not the ordinary Schedule D carryover; the App calculates that from the filed return.
    /// [NullText]
    /// No regular capital loss carries forward
    /// [SetText]
    /// The prior Form 8960 excluded capital-loss difference is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub niit_difference: Option<i64>,
    /// Each spouse’s surviving capital-loss amounts
    /// Only for a Form 8379 claim. Use actual source records; this never changes the joint calculation.
    /// [NullText]
    /// The surviving spouse shares have not been established
    /// [SetText]
    /// The actual surviving spouse shares are
    #[libertas_default($null)]
    pub injured_owners: Option<crate::InjuredCapitalLossOwners>,
    /// Actual owners of the retained Form 8960 excluded-loss difference
    /// Only for a Form 8379 claim. This is separate from regular and AMT capital-loss shares and from Form 4952 investment-property portions. Use each spouse's retained section 1411 records.
    /// [NullText]
    /// Owners of the retained Form 8960 difference have not been established
    /// [SetText]
    /// The retained Form 8960 difference belongs to
    #[libertas_default($null)]
    pub injured_niit_owners: Option<crate::InjuredRetainedOwners>,
    /// Actual current research-business portions of the regular carryover
    pub research_parts: Option<crate::ResearchCapitalParts>,
}

/// Investment-property portions of the computed capital-loss carryover
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentCapitalParts {
    /// Regular short-term investment loss portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub short: i64,
    /// Regular long-term investment loss portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub long: i64,
    /// Separate AMT portions, required when the AMT carryover differs
    /// [NullText]
    /// AMT investment-loss portions equal the regular portions
    /// [SetText]
    /// Different AMT investment-loss portions are
    #[libertas_default($null)]
    pub amt: Option<InvestmentAmtCapitalParts>,
}
/// Investment-property portions of the AMT carryover
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentAmtCapitalParts {
    /// AMT short-term investment loss portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub short: i64,
    /// AMT long-term investment loss portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub long: i64,
}
/// Retained 2025 AMT capital-loss worksheet facts
/// Use the completed prior AMT Schedule D and AMT Capital Loss Carryover Worksheet for the same return and owners. These are source worksheet lines, not a carryover amount calculated for this App.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PriorAmtCapitalReturn {
    /// AMT carryover worksheet taxable income before exemption
    /// Copy the signed income used on line 1 of the retained AMT Capital Loss Carryover Worksheet, including a negative amount. It excludes the AMT exemption; do not substitute a zero-floored Form 6251 total or the regular Form 1040 taxable income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub taxable_income: i64,
    /// AMT Schedule D line 7 net short-term gain or loss
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub short: i64,
    /// AMT Schedule D line 15 net long-term gain or loss
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub long: i64,
    /// AMT Schedule D line 21 loss, as a positive amount
    #[libertas_money("USD", 2)]
    // Tax structure: Schedule D limits the annual deducted capital loss to at most $3,000.
    #[libertas_number(min = 0, max = 300000)]
    pub deducted_loss: i64,
}

/// Surviving capital loss from an original tax year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CapitalLossOrigin {
    /// Original loss year, not the year of the latest carryover worksheet
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 1900, max = 2025)]
    pub year: u32,
    /// Unused short-term loss entering 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub short: i64,
    /// Unused long-term loss entering 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub long: i64,
}
