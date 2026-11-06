use crate::FederalStatus;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Parent return used for your investment tax
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ParentTax {
    /// Parent's filing status
    pub status: FederalStatus,
    /// Parent's taxable income
    /// Copy Form 1040 line 15 from the applicable parent return. Use zero for a negative amount. This is a source-return amount, not the parent's AGI or tax due.
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000, step = 1)]
    #[libertas_money("USD", 2)]
    pub taxable: i64,
    /// Parent's qualified dividends and capital gains
    /// Leave absent only if the parent has neither qualified dividends nor a net capital gain. Enter source-return lines without reducing them by the parent's deductions.
    pub gains: Option<ParentGains>,
    /// Other children using the same parent return
    /// Include every other child whose Form 8615 uses this parent, even if not this parent's dependent. Leave empty only when there are none. Do not include this return, or a brother or sister whose income your parent reported on Form 8814.
    pub others: Vec<OtherChildTax>,
    /// Correct parent return and complete family information
    /// Joint parents: use their joint return. Married separate parents: generally use the higher taxable-income return. Divorced, separated or parents living apart: apply the custodial-parent rules, including a remarried custodial parent's new spouse. Unmarried parents living together all year: use the higher taxable-income return. Use the parent tax year ending in your tax year. No when parent selection or any sibling return is unresolved; the tax at your parent's rate then waits for it.
    pub complete: bool,
    /// No Schedule J on the parent's or another child's return
    /// Income averaging for farmers and fishermen on your parent's return, or on another child's Form 8615, is not calculated; No leaves the tax at the parent's rate unfigured. Form 4952 elections and a parent's Form 2555 are entered above.
    pub ordinary: bool,
    /// Parent's Form 2555 excluded income and housing deduction
    /// Lines 45 and 50 of the parent's Form 2555 added together (line 2c of the parent's Foreign Earned Income Tax Worksheet). Leave absent when the parent files no Form 2555. The App adds it to the parent's taxable income on line 6 and figures the line 10 tax on that total, as the worksheet does.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub foreign_excluded: Option<i64>,
}

/// Parent's preferential-rate source amounts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ParentGains {
    /// Qualified dividends
    /// Copy the parent's Form 1040 line 3a, not total ordinary dividends on line 3b.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified: i64,
    /// Net long-term capital gain or loss
    /// Copy Schedule D line 15, including a loss as negative. If Schedule D is not required, enter the capital gain distributions reported on Form 1040 line 7a here and below.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub long_term: i64,
    /// Combined net capital gain or loss
    /// Copy Schedule D line 16, not the loss-limited amount reported on Form 1040. The App determines the positive net capital gain from these two lines.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub combined: i64,
    /// Parent's 28% rate gain
    /// Copy the positive amount from Schedule D line 18; enter zero if blank or nonpositive. Do not pre-limit it to line 16.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub collectibles: i64,
    /// Parent's unrecaptured section 1250 gain
    /// Copy Schedule D line 19. Keep the source amount before the Schedule D Tax Worksheet's net-capital-gain cap.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured: i64,
    /// Form 4952 line 4g on that return
    /// Qualified dividends and net capital gain elected as investment income; zero if none. Line 3 of the Schedule D Tax Worksheet and of the Qualified Dividends and Capital Gain Tax Worksheet for Form 8615 lines 9 and 15 subtracts it.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment_election: i64,
}

/// Other child's Form 8615
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct OtherChildTax {
    /// Child label
    #[libertas_size(min = 1)]
    pub label: alloc::string::String,
    /// Net unearned income
    /// Copy line 5 of that child's completed Form 8615 or its calculated worksheet output. Do not enter gross interest, AGI or tax. Recalculate all affected returns if any source return changes.
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000, step = 1)]
    #[libertas_money("USD", 2)]
    pub net: i64,
    /// Preferential income included in this child's line 5
    /// Copy the allocated amounts from the completed Form 8615 Line 5 Worksheet, not that child's gross dividends or Schedule D totals. Leave absent when both are zero.
    /// [NullText]
    /// This child's line 5 contains no qualified dividends or net capital gain
    /// [SetText]
    /// This child's allocated preferential income is
    pub gains: Option<ChildGains>,
}

/// Other child's allocated preferential income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ChildGains {
    /// Qualified dividends included in line 5
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified: i64,
    /// Net capital gain included in line 5
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub capital: i64,
    /// Form 4952 line 4g on that child's return
    /// Qualified dividends and net capital gain that child elected as investment income; zero if none. Line 3 of the line 9 and line 15 worksheets subtracts it.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment_election: i64,
    /// Original special-rate gain sources
    /// Supply when the other child has a positive Schedule D line 18 or 19. The App allocates these original amounts using that child's allocated capital gain above.
    /// [NullText]
    /// This child has no positive Schedule D line 18 or 19
    /// [SetText]
    /// This child's original special-rate gain sources are
    pub special: Option<ChildSpecialGains>,
}

/// Other child's original special-gain sources
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ChildSpecialGains {
    /// Net capital gain before allocation
    /// Copy line 2 of that child's Form 8615 Line 5 Worksheet: the smaller positive Schedule D line 15 or 16, before deduction allocation.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub net_capital: i64,
    /// Original 28% rate gain
    /// Positive Schedule D line 18; zero if blank or nonpositive. Do not pre-limit to the allocated gain.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub collectibles: i64,
    /// Original unrecaptured section 1250 gain
    /// Schedule D line 19 before allocation or the net-capital-gain cap.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured: i64,
}
