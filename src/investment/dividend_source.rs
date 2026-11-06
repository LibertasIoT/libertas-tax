use crate::capital_sale_model::GainTerm;
use crate::kernel::phase;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Property received in a taxable corporate liquidation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct LiquidationProperty {
    /// Asset or distributed share lot
    #[libertas_size(min = 1)]
    pub label: String,
    /// Fair market value allocated to this entry on distribution date
    /// Retain the corporation's valuation and allocation records. Do not also include this value in cash received.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
}

/// Type of capital distribution
/// Check the issuer's distribution and liquidation notices, including whether the shares were redeemed or cancelled.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum CapitalDistributionKind {
    /// Nondividend return of capital
    Nondividend,
    /// Liquidation — more payments remain
    InterimLiquidation,
    /// Liquidation — final payment and shares redeemed or cancelled
    FinalLiquidation,
    /// Liquidation — the records do not yet establish whether this is the final payment
    Liquidation,
}
impl CapitalDistributionKind {
    pub(crate) fn needs_liquidation_stage(self, amount: i64, basis: i64) -> bool {
        // Pub. 550: finality determines an unrecovered-basis loss. At exactly
        // zero gain it also determines whether Form 8949 reports a final row.
        self == Self::Liquidation && amount <= basis
    }

    pub(crate) fn gain(self, amount: i64, basis: i64) -> i64 {
        if self == Self::FinalLiquidation {
            amount - basis
        } else {
            phase::floor(amount - basis, 0)
        }
    }
}

/// Undistributed capital gains — Form 2439
/// Enter one statement for your own taxable-account shares. Component boxes are included in box 1a; do not also report this statement as a dividend or sale.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct UndistributedGain {
    /// Total undistributed long-term capital gain — box 1a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gain: i64,
    /// Unrecaptured section 1250 gain — box 1b
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub unrecaptured: i64,
    /// Section 1202 gain — box 1c
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub small_business: i64,
    /// Section 1202 exclusion for the box 1c gain
    #[libertas_default($null)]
    pub section_1202_exclusion: Option<FundQsbsExclusion>,
    /// Collectibles gain — box 1d
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub collectibles: i64,
    /// Tax paid by the fund or REIT — box 2
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub tax_paid: i64,
    /// Held in an IRA or another tax-exempt account, received as a nominee, allocated to you through a partnership, S corporation, estate or trust, or a fund year that doesn't end in 2026
    /// Yes leaves this statement out of the estimate with a notice; that treatment is not supported.
    pub special: bool,
}

/// Section 1202 exclusion on a fund's small business stock gain
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FundQsbsExclusion {
    /// None
    /// You did not own your fund shares on the day the fund bought the stock and every day after until it sold it, you bought more fund shares after it bought the stock, or the fund's statement does not give the exclusion.
    None,
    /// 50%: the fund bought the stock before February 18, 2009
    Fifty,
    /// 75%: the fund bought the stock from February 18, 2009 through September 27, 2010
    SeventyFive,
    /// 100%: the fund bought the stock from September 28, 2010 through July 4, 2025
    Hundred,
}

/// Dividends
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct FederalDividend {
    /// Box 3 nondividend distributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub nondividend_distributions: i64,

    /// Box 9 cash liquidation distributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_liquidation: i64,

    /// Box 10 noncash liquidation distributions
    #[libertas_money("USD", 2)]
    // Carries the source property total without an independent cap.
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub noncash_liquidation: i64,
    /// Per-lot return-of-capital and liquidation facts for these printed boxes
    pub capital_returns: Vec<CapitalReturn1099Div>,

    /// Components legally belonging to someone outside this return
    /// Enter actual component amounts from ownership records, not money merely given away. Withholding stays on its legally entitled source. A nominee may need to furnish and file Forms 1099; this App does not file them.
    #[libertas_default($null)]
    pub nominee: Option<crate::DividendNominee>,

    /// Section 897 subset of ordinary dividends, already included in box 1a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_897_ordinary: i64,
    /// Section 897 subset of capital distributions, already included in box 2a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_897_capital: i64,
    /// Foreign country or U.S. territory printed in box 8
    #[libertas_default("")]
    pub foreign_country: String,
    /// Ordinary dividends, including qualified dividends
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub ordinary: i64,
    /// Qualified dividends meeting the holding-period rules
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub qualified: i64,
    /// Section 199A REIT dividends
    pub section199a: crate::ReitDividends,
    /// Total long-term capital gain distributions (Form 1099-DIV box 2a)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub capital_distributions: i64,
    /// Collectibles (28%) gain included in box 2a (box 2d)
    /// This is part of the total above, not additional income. Enter zero if the payer reports none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub collectible_distributions: i64,
    /// Unrecaptured section 1250 gain included in box 2a (box 2b)
    /// This component is not additional income. Enter zero if the payer reports none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub unrecaptured_distributions: i64,
    /// Section 1202 gain included in box 2a (box 2c)
    /// Part of box 2a, not additional income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_1202_gain: i64,
    /// Section 1202 exclusion for the box 2c gain
    /// Answered when box 2c shows an amount.
    #[libertas_default($null)]
    pub section_1202_exclusion: Option<FundQsbsExclusion>,
    /// Federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
    /// A treatment this page does not calculate
    /// The broker's supplemental statement and ownership records show it. Any choice but None of these leaves this statement out of the estimate with a notice; those treatments are not supported.
    pub special: DividendSpecial,
    /// Exempt-interest dividends (box 12)
    /// Copy Form 1099-DIV box 12. Not included in ordinary dividends above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt_dividends: i64,
    /// Specified private-activity bond interest dividends included in the exempt-interest dividends above (box 13)
    /// Copy Form 1099-DIV box 13; do not enter it twice as income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity_dividends: i64,
    /// Foreign tax paid (box 7)
    /// Read by the foreign tax credit: the section 904(j) payee-statement election takes it; a Form 1116 source records it again by country.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_tax: i64,
    /// The part paid after the owner's death
    /// Only for a person who died in 2026: the time after the death belongs to the estate or another recipient and is left off the final return.
    #[libertas_default($null)]
    pub after_death: Option<crate::DividendsAfterDeath>,
}

/// A treatment of dividends this page does not calculate
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
)]
pub enum DividendSpecial {
    /// None of these
    #[default]
    NoneOfThese,
    /// Foreign treatment beyond the foreign tax credit election
    /// A treaty rate or another foreign-tax treatment the credit election does not cover.
    Foreign,
    /// Another treatment this page does not calculate
    /// For example an unlisted REIT or publicly traded partnership treatment.
    Another,
}
impl DividendSpecial {
    // Answers and unknowns: the notice of the treatment that leaves the
    // statement out of the estimate; None for no treatment.
    pub(crate) fn notice(self) -> Option<&'static str> {
        match self {
            Self::NoneOfThese => None,
            Self::Foreign => Some("WARN_DIVIDEND_FOREIGN_LEFT_OUT"),
            Self::Another => Some("WARN_DIVIDEND_OTHER_LEFT_OUT"),
        }
    }
}

/// 1099-DIV
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into payer, recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1099-DIV exactly. Check a printed calendar year against the supplied year and report a mismatch as an error. Report an illegible year or amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Return the payer's name as printed, without truncation, or an empty string when it is absent. Read every amount box as printed: box 1a total ordinary dividends, 1b qualified dividends, 2a total capital gain distributions, 2b unrecaptured section 1250 gain, 2c section 1202 gain into section_1202_gain, 2d collectibles (28%) gain, 2e and 2f, 3 nondividend distributions, 4 federal income tax withheld, 5 section 199A dividends, 6 investment expenses, 7 foreign tax paid, 9 cash liquidation distributions, 10 noncash liquidation distributions, 12 exempt-interest dividends and 13 specified private activity bond interest dividends. Return null for each amount box that is clearly blank, never numeric zero; a printed zero stays zero. Read box 11 FATCA filing requirement as its printed checkbox. Read corrected as its printed checkbox. Read calendar_year as an integer number, or null when the calendar year is clearly blank; never substitute zero or the form revision year. Read payer_telephone and account_number exactly as printed; use empty text when absent. Do not infer share-lot basis, holding period, liquidation stage or property identity, and do not decide whether the section 199A dividends meet the holding-period rules, the section 1202 exclusion of box 2c, or whether a special treatment applies; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form. Read the payer federal ID and address and state boxes 14 through 16. For the state_details rows, copy each populated state row once and use the two-letter state code from the form. Keep federal and state IDs as text, preserving leading zeros, masking and punctuation. Return an empty string for absent payer identity text fields and null for a blank state or local income box; return zero for a clearly blank withholding box. Return empty lists when the corresponding rows are absent. Never copy federal income into a blank state income box or infer a state from the payer address. A populated row without a legible state or locality is an error. Copies of the same form must not duplicate state rows. From an included supplemental statement, read the identified issuing-state allocations into exempt_sources, selecting Amount for printed dollar amounts or Percentage for printed percentages. Return Percentage.value as a decimal number from 0 to 100 without a percent sign; for example, 27.43% is 27.43, not 0.2743. Read the fund's U.S. government obligation percentage on the same scale, returning a decimal number without a percent sign for a reported value, or null when absent; an explicit zero is 0. Return an empty allocation list when no allocation is supplied. Do not convert a percentage to dollars or assume the withholding state is the bond-issuing state. Read boxes 2e and 2f as included components, never add them to boxes 1a or 2a. Read box 8 foreign country or U.S. territory; a blank country is empty text.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.payer.federal_id,@.recipient.federal_id,@.calendar_year,@.ordinary,@.qualified,@.capital_distributions,@.unrecaptured_distributions,@.section_1202_gain,@.collectible_distributions,@.section_897_ordinary,@.section_897_capital,@.nondividend_distributions,@.withholding,@.section199a,@.investment_expenses,@.foreign_tax,@.cash_liquidation,@.noncash_liquidation,@.exempt_dividends,@.private_activity_dividends,@.account_number"
)]
pub struct Form1099DivPaper {
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,

    /// For calendar year
    /// [NullText]
    /// Calendar year is not printed
    /// [SetText]
    /// Calendar year is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_default($null)]
    #[libertas_format("0")]
    pub calendar_year: Option<u32>,

    /// PAYER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub payer: crate::PrintedParty,

    /// PAYER’S telephone no.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub payer_telephone: String,
    /// RECIPIENT'S name, TIN and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,

    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1a Total ordinary dividends
    /// [NullText]
    /// Box 1a is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub ordinary: Option<i64>,
    /// Box 1b Qualified dividends
    /// [NullText]
    /// Box 1b is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub qualified: Option<i64>,
    /// Box 2a Total capital gain distr.
    /// [NullText]
    /// Box 2a is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub capital_distributions: Option<i64>,
    /// Box 2b Unrecap. Sec. 1250 gain
    /// This component is not additional income. Leave it blank if the payer reports none.
    /// [NullText]
    /// Box 2b is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub unrecaptured_distributions: Option<i64>,
    /// Box 2c Section 1202 gain
    /// Part of box 2a, not additional income.
    /// [NullText]
    /// Box 2c is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub section_1202_gain: Option<i64>,
    /// Box 2d Collectibles (28%) gain
    /// This is part of the total above, not additional income. Leave it blank if the payer reports none.
    /// [NullText]
    /// Box 2d is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub collectible_distributions: Option<i64>,
    /// Box 2e Section 897 ordinary dividends
    /// [NullText]
    /// Box 2e is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub section_897_ordinary: Option<i64>,
    /// Box 2f Section 897 capital gain
    /// [NullText]
    /// Box 2f is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub section_897_capital: Option<i64>,
    /// Box 3 Nondividend distributions
    /// [NullText]
    /// Box 3 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub nondividend_distributions: Option<i64>,
    /// Box 4 Federal income tax withheld
    /// [NullText]
    /// Box 4 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub withholding: Option<i64>,
    /// Box 5 Section 199A dividends
    /// Already included in box 1a ordinary dividends; not box 1b qualified dividends or capital gain distributions.
    /// [NullText]
    /// Box 5 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub section199a: Option<i64>,

    /// Box 6 Investment expenses
    /// [NullText]
    /// Box 6 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub investment_expenses: Option<i64>,
    /// Box 7 Foreign tax paid
    /// [NullText]
    /// Box 7 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub foreign_tax: Option<i64>,
    /// Box 8 Foreign country or U.S. possession
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub foreign_country: String,
    /// Box 9 Cash liquidation distributions
    /// [NullText]
    /// Box 9 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub cash_liquidation: Option<i64>,
    /// Box 10 Noncash liquidation distributions
    /// [NullText]
    /// Box 10 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Carries the source property total without an independent cap.
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub noncash_liquidation: Option<i64>,

    /// Box 11 FATCA filing requirement
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub fatca: bool,
    /// Box 12 Exempt-interest dividends
    /// [NullText]
    /// Box 12 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub exempt_dividends: Option<i64>,
    /// Box 13 Specified private activity bond interest dividends
    /// [NullText]
    /// Box 13 is blank
    /// [SetText]
    /// The printed value is
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub private_activity_dividends: Option<i64>,
    /// Boxes 14 State, 15 State identification no., 16 State tax withheld and attached supplemental statement
    #[libertas_llm_doc_field]
    pub state_details: crate::InvestmentStateFields,
}

/// A nominee or after-death part leaves a printed box: a blank box stays blank while nothing
/// moves out of it.
pub(crate) fn reduce_box(printed: &mut Option<i64>, part: i64) {
    if part != 0 {
        *printed = Some(printed.unwrap_or(0) - part);
    }
}

impl Form1099DivPaper {
    // Box 3 identifies a nondividend distribution; boxes 9 and 10 identify
    // liquidation, but do not establish whether a payment is final.
    // https://www.irs.gov/instructions/i1099div
    pub(crate) fn capital_return_kind(
        &self,
        lot: &CapitalReturn1099Div,
    ) -> CapitalDistributionKind {
        // Preserve malformed relationships for the shared input validator.
        if lot.issue().is_some() {
            return lot.kind;
        }
        let recorded = lot.kind;
        let nondividend = self.nondividend_distributions.unwrap_or(0) > 0;
        let liquidation =
            self.cash_liquidation.unwrap_or(0) > 0 || self.noncash_liquidation.unwrap_or(0) > 0;
        match (nondividend, liquidation) {
            (true, false) => CapitalDistributionKind::Nondividend,
            (false, true) => match recorded {
                CapitalDistributionKind::InterimLiquidation
                | CapitalDistributionKind::FinalLiquidation => recorded,
                _ => CapitalDistributionKind::Liquidation,
            },
            // Mixed boxes require the actual per-lot attribution. Empty boxes
            // do not establish a replacement kind for previously entered lots.
            _ => recorded,
        }
    }
}

/// Accepted Form 1099-DIV
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DividendSource {
    /// Form 1099-DIV as printed
    pub form: Form1099DivPaper,
    /// Applicable personal follow-up choices
    #[libertas_hidden]
    #[libertas_read_only]
    pub calls: DivCalls,
    /// Share lots receiving boxes 3, 9 or 10
    /// Split each printed box among its actual share lots. Enter basis and holding period from your records. Do not enter these distributions again as a separate return-of-capital document.
    #[libertas_hidden]
    /// ----
    /// Share lot
    pub capital_returns: Vec<CapitalReturn1099Div>,
    /// Components legally belonging to someone outside this return
    /// Enter actual component amounts from ownership records, not money merely given away. Withholding stays on its legally entitled source. A nominee may need to furnish and file Forms 1099; this App does not file them.
    /// [NullText]
    /// All reported dividend components belong to this return
    /// [SetText]
    /// These dividend components belong to someone outside this return
    #[libertas_hidden]
    pub nominee: Option<DivNomineeAnswer>,
    /// Box 5: all these dividends satisfy the holding-period and payment-obligation rules
    /// Answer when box 5 shows an amount. For directly held shares, generally hold them for more than 45 days in the applicable 91-day period, excluding days with diminished risk. Split differently treated lots into separate entries without duplicating amounts. Check the share-holding records or fiduciary’s qualified dividend statement. No takes no section 199A deduction on these dividends.
    /// [NullText]
    /// No box 5 section 199A dividends are reported
    /// [SetText]
    /// For the box 5 dividends, holding-period and payment-obligation eligibility is
    pub reit_eligible: Option<DividendEligibility>,
    /// Paid after the death
    /// Only for a person who died in 2026 whose statement also includes dividends for the time after the death: that part belongs to the estate or another recipient and is left off this final return (Pub. 559). Choose “No reported dividends belong to the period after death” otherwise. Your account statements show the dates.
    /// [NullText]
    /// No reported dividends belong to the period after death
    /// [SetText]
    /// The reported dividends paid after death are
    pub after_death: Option<DivAfterDeathAnswer>,
    /// Section 1202 exclusion for the box 2c gain
    /// Answer when box 2c shows an amount; the fund's statement gives the date it bought the stock.
    /// [NullText]
    /// No box 2c section 1202 gain is reported
    /// [SetText]
    /// The box 2c section 1202 exclusion treatment is
    #[libertas_hidden]
    pub section_1202_exclusion: Option<FundQsbsExclusion>,
    /// A treatment this page does not calculate
    /// Foreign treatment beyond the credit election or another unentered dividend treatment. Nominee components have their own personal record below. Any choice but None of these leaves this statement out of the estimate with a notice.
    #[libertas_default(NoneOfThese)]
    #[libertas_hidden]
    pub special: DividendSpecial,
}

impl DividendSource {
    pub(crate) fn normalize_capital_return_kinds(&mut self) {
        for lot in &mut self.capital_returns {
            lot.kind = self.form.capital_return_kind(lot);
        }
    }

    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: Form1099DivPaper) -> Self {
        Self {
            form,
            calls: DivCalls::default(),
            capital_returns: Vec::new(),
            nominee: None,
            reit_eligible: None,
            after_death: None,
            section_1202_exclusion: None,
            special: DividendSpecial::NoneOfThese,
        }
    }
    /// The topic answers of an accepted source survive a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.capital_returns = accepted.capital_returns.clone();
        self.nominee = accepted.nominee.clone();
        self.reit_eligible = accepted.reit_eligible;
        self.after_death = accepted.after_death.clone();
        self.section_1202_exclusion = accepted.section_1202_exclusion;
        self.special = accepted.special;
        self.normalize_capital_return_kinds();
    }
}

// One-way: every printed box and answer either reaches the internal model or
// is named as not read by any calculation, so a new one cannot be forgotten.
impl From<&DividendSource> for FederalDividend {
    fn from(source: &DividendSource) -> Self {
        let DividendSource {
            form,
            // Presentation only: which follow-ups the page offers.
            calls: _,
            capital_returns,
            nominee,
            reit_eligible,
            after_death,
            section_1202_exclusion,
            special,
        } = source;
        let Form1099DivPaper {
            // Informational printed boxes are retained without changing figures.
            corrected: _,
            calendar_year: _,
            payer_telephone: _,
            account_number: _,
            investment_expenses: _,
            fatca: _,
            // The parties name and assign the document; no figure reads them.
            payer: _,
            recipient: _,
            ordinary,
            qualified,
            nondividend_distributions,
            cash_liquidation,
            noncash_liquidation,
            section_897_ordinary,
            section_897_capital,
            foreign_country,
            capital_distributions,
            unrecaptured_distributions,
            collectible_distributions,
            section_1202_gain,
            withholding,
            section199a,
            foreign_tax,
            exempt_dividends,
            private_activity_dividends,
            // State boxes reach the state return through the accepted entry.
            state_details: _,
        } = form;
        FederalDividend {
            nondividend_distributions: nondividend_distributions.unwrap_or(0),
            cash_liquidation: cash_liquidation.unwrap_or(0),
            noncash_liquidation: noncash_liquidation.unwrap_or(0),
            capital_returns: capital_returns
                .iter()
                .cloned()
                .map(|mut lot| {
                    lot.kind = form.capital_return_kind(&lot);
                    lot
                })
                .collect(),
            nominee: nominee.as_ref().map(|answer| answer.record().clone()),
            section_897_ordinary: section_897_ordinary.unwrap_or(0),
            section_897_capital: section_897_capital.unwrap_or(0),
            foreign_country: foreign_country.clone(),
            ordinary: ordinary.unwrap_or(0),
            qualified: qualified.unwrap_or(0),
            section199a: match (section199a.unwrap_or(0), reit_eligible) {
                (0, _) => crate::ReitDividends::None,
                (amount, eligible) => crate::ReitDividends::Reported {
                    amount,
                    eligible: eligible.map(|answer| answer == DividendEligibility::Yes),
                },
            },
            capital_distributions: capital_distributions.unwrap_or(0),
            collectible_distributions: collectible_distributions.unwrap_or(0),
            unrecaptured_distributions: unrecaptured_distributions.unwrap_or(0),
            section_1202_gain: section_1202_gain.unwrap_or(0),
            section_1202_exclusion: *section_1202_exclusion,
            withholding: withholding.unwrap_or(0),
            special: *special,
            exempt_dividends: exempt_dividends.unwrap_or(0),
            private_activity_dividends: private_activity_dividends.unwrap_or(0),
            foreign_tax: foreign_tax.unwrap_or(0),
            after_death: after_death.as_ref().map(|answer| answer.record().clone()),
        }
    }
}

/// Share-lot return of capital or liquidation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CapitalReturn1099Div {
    /// Distribution type
    /// Box 3 is a nondividend distribution; boxes 9 and 10 are a liquidation. Choose the unresolved liquidation status when the issuer's records do not establish whether more payments remain or the shares were finally redeemed or cancelled.
    pub kind: CapitalDistributionKind,
    /// Box 3 nondividend distributions, or box 9 cash liquidation distributions
    /// For this share lot only; split the box among lots by share count.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Box 10 noncash liquidation distributions
    /// The value of each property received, for this share lot.
    /// ----
    /// Property
    pub property: Vec<LiquidationProperty>,
    /// Your cost in this share lot just before the distribution
    /// What you paid, less earlier nondividend distributions. Your broker's cost-basis statement or your purchase records show it.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Holding period of this share lot on the distribution date
    pub term: GainTerm,
    /// Shares held for someone else, in a retirement account or a partnership or S corporation, debt the corporation assumed, a reorganization or another special treatment
    /// Yes leaves this distribution out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}
impl CapitalReturn1099Div {
    // Pub. 550 allows a liquidation loss only after the final payment redeems
    // or cancels the shares; a reported box does not establish that event.
    // https://www.irs.gov/publications/p550
    pub(crate) fn needs_liquidation_status(&self) -> bool {
        crate::kernel::ratio::checked_sum(
            core::iter::once(self.amount)
                .chain(self.property.iter().map(|property| property.value)),
        )
        .is_none_or(|amount| self.kind.needs_liquidation_stage(amount, self.basis))
    }

    pub(crate) fn valid_amounts(amount: i64, basis: i64, property: &[LiquidationProperty]) -> bool {
        [amount, basis].iter().all(|value| (0..=100_000_000_000).contains(value))
            && property.len() <= 100
            && property
                .iter()
                .all(|p| !p.label.trim().is_empty() && (0..=100_000_000_000).contains(&p.value))
    }
    pub(crate) fn validation_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if let Some(issue) = self.issue() {
            return Some(issue);
        }
        // These existing intrinsic checks combine scalar and property-list
        // bounds. The actual caller locates the failed lot, without guessing
        // which leaf failed or evaluating the relationship owner a second time.
        (!Self::valid_amounts(self.amount, self.basis, &self.property))
            .then(|| ValidationIssue::new("DIV_CAPITAL_RETURNS_RECONCILE"))
    }
    // The relationship between this page's fields that fails, by its rule's
    // key: property is box 10, a liquidation's.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        (!matches!(
            self.kind,
            CapitalDistributionKind::InterimLiquidation
                | CapitalDistributionKind::FinalLiquidation
                | CapitalDistributionKind::Liquidation
        ) && !self.property.is_empty())
        .then(|| {
            crate::validation::ValidationIssue::new("CAPITAL_RETURN_PROPERTY_NOT_LIQUIDATION")
                .at_field("property")
        })
    }
}

/// Form 2439
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into fund, shareholder. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 2439 exactly. Check that the tax year of the regulated investment company or real estate investment trust shown at the top of the form ends in the supplied year; report a mismatch as an error. Return the name of the RIC or REIT as printed into fund.name, or empty text when it is blank. Read box 1a total undistributed long-term capital gains, box 1b unrecaptured section 1250 gain, box 1c section 1202 gain, box 1d collectibles (28%) gain and box 2 tax paid by the RIC or REIT on the box 1a gains; return zero for a clearly blank box. Report an illegible amount, or two values that conflict, as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed on the form. Do not decide whether the shares are held in a tax-exempt account, as a nominee or through a pass-through entity, and do not report the section 1202 exclusion percentage; the filer answers those. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct Form2439 {
    /// Regulated investment company or real estate investment trust
    #[libertas_llm_doc_field]
    pub fund: crate::PrintedParty,
    /// Shareholder's name, identifying number and address
    #[libertas_llm_doc_field]
    pub shareholder: crate::PrintedParty,
    /// Box 1a total undistributed long-term capital gains
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub gain: i64,
    /// Box 1b unrecaptured section 1250 gain
    /// Part of box 1a, not additional income.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured: i64,
    /// Box 1c section 1202 gain
    /// Part of box 1a. The exclusion of this gain follows your answer on the next page.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub small_business: i64,
    /// Box 1d collectibles (28%) gain
    /// Part of box 1a, not additional income.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub collectibles: i64,
    /// Box 2 tax paid by the fund on the box 1a gains
    /// A refundable credit on Schedule 3 line 13a; the App also adds box 1a less box 2 to your cost in the shares.
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_paid: i64,
}
impl Form2439 {
    // The first relationship between the printed boxes that fails, by its
    // rule's key, in the order of the client rules.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if i128::from(self.unrecaptured)
            + i128::from(self.small_business)
            + i128::from(self.collectibles)
            > i128::from(self.gain)
        {
            return Some(
                crate::validation::ValidationIssue::new("FORM_2439_PARTS_ABOVE_1A")
                    .at_field("gain"),
            );
        }
        (self.tax_paid > self.gain).then(|| {
            crate::validation::ValidationIssue::new("FORM_2439_TAX_ABOVE_1A").at_field("tax_paid")
        })
    }
    // The label: the fund as typed.
    pub(crate) fn label(&self) -> String {
        self.fund.name.clone()
    }
}

// Form 2439 (Rev. November 2021), Copy B: https://www.irs.gov/pub/irs-pdf/f2439.pdf
/// Form 2439 — Notice to Shareholder of Undistributed Long-Term Capital Gains
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 2439 exactly as printed. Read the calendar year, or the other tax year's beginning and ending dates, at the top of the form. Return zero for a blank calendar year and null for a clearly blank other-tax-year date. Check that the RIC or REIT tax year ends in the supplied year and report a mismatch as an error. Read CORRECTED as true only when checked. Read the RIC or REIT name, identification number and address into ric_or_reit and the shareholder name, identifying number and address into shareholder, preserving masking, punctuation and leading zeros. Use empty text for absent identity fields. Never infer a shareholder from household names. Read box 1a total undistributed long-term capital gains, box 1b unrecaptured section 1250 gain, box 1c section 1202 gain, box 1d collectibles (28%) gain, and box 2 tax paid by the RIC or REIT on the box 1a gains; return zero for a clearly blank amount. Report an illegible or conflicting value as an error. Return amounts as decimal numbers without commas or currency signs. Never infer a value that is not printed. Do not report a section 1202 exclusion percentage or account ownership treatment; the filer answers those separately. Multiple copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.ric_or_reit.federal_id,@.shareholder.federal_id,@.calendar_year,@.total_undistributed_long_term_capital_gains,@.unrecaptured_section_1250_gain,@.section_1202_gain,@.collectibles_28_percent_gain,@.tax_paid_by_ric_or_reit",
    form_date_order = "MonthDayYear"
)]
pub struct Form2439Paper {
    /// For calendar year
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_default(0)]
    pub calendar_year: u32,
    /// Other tax year of the regulated investment company (RIC) or the real estate investment trust (REIT) beginning
    /// [NullText]
    /// Other tax year beginning date is not printed
    /// [SetText]
    /// Other tax year begins on
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub other_tax_year_beginning: Option<u32>,
    /// Other tax year ending
    /// [NullText]
    /// Other tax year ending date is not printed
    /// [SetText]
    /// Other tax year ends on
    #[libertas_llm_doc_field(blank = $null)]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub other_tax_year_ending: Option<u32>,
    /// VOID
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub void: bool,
    /// CORRECTED
    #[libertas_llm_doc_field(blank = false)]
    #[libertas_default(false)]
    pub corrected: bool,
    /// Name, address, and ZIP code of RIC or REIT; identification number of RIC or REIT
    #[libertas_llm_doc_field]
    pub ric_or_reit: crate::PrintedParty,
    /// Shareholder’s identifying number; shareholder’s name, address, and ZIP code
    #[libertas_llm_doc_field]
    pub shareholder: crate::PrintedParty,
    /// Box 1a Total undistributed long-term capital gains
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub total_undistributed_long_term_capital_gains: i64,
    /// Box 1b Unrecaptured section 1250 gain
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_section_1250_gain: i64,
    /// Box 1c Section 1202 gain
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_1202_gain: i64,
    /// Box 1d Collectibles (28%) gain
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub collectibles_28_percent_gain: i64,
    /// Box 2 Tax paid by the RIC or REIT on the box 1a gains
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_paid_by_ric_or_reit: i64,
}

impl Form2439Paper {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        Form2439::from(self).issue().map(|issue| {
            issue
                .map_field_prefix(&["gain"], &["total_undistributed_long_term_capital_gains"])
                .map_field_prefix(&["tax_paid"], &["tax_paid_by_ric_or_reit"])
        })
    }

    pub(crate) fn label(&self) -> String {
        Form2439::from(self).label()
    }
}

impl From<&Form2439Paper> for Form2439 {
    fn from(paper: &Form2439Paper) -> Self {
        let Form2439Paper {
            // Printed metadata has no calculation consumer.
            calendar_year: _,
            other_tax_year_beginning: _,
            other_tax_year_ending: _,
            void: _,
            corrected: _,
            ric_or_reit,
            shareholder,
            total_undistributed_long_term_capital_gains,
            unrecaptured_section_1250_gain,
            section_1202_gain,
            collectibles_28_percent_gain,
            tax_paid_by_ric_or_reit,
        } = paper;
        Self {
            fund: ric_or_reit.clone(),
            shareholder: shareholder.clone(),
            gain: *total_undistributed_long_term_capital_gains,
            unrecaptured: *unrecaptured_section_1250_gain,
            small_business: *section_1202_gain,
            collectibles: *collectibles_28_percent_gain,
            tax_paid: *tax_paid_by_ric_or_reit,
        }
    }
}

/// Accepted Form 2439
/// The printed form and the topic answers it calls for, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct UndistributedGainSource {
    /// Form 2439 as printed
    pub form: Form2439Paper,
    /// Section 1202 exclusion for the box 1c gain
    /// Answer when box 1c shows an amount; the fund's statement gives the date it bought the stock.
    /// [NullText]
    /// No box 1c section 1202 gain is reported
    /// [SetText]
    /// The box 1c section 1202 exclusion treatment is
    pub section_1202_exclusion: Option<FundQsbsExclusion>,
    /// Held in an IRA or another tax-exempt account, received as a nominee, allocated to you through a partnership, S corporation, estate or trust, or a fund year that doesn't end in 2026
    /// Yes leaves this statement out of the estimate with a notice; that treatment is not supported.
    #[libertas_default(false)]
    pub special: bool,
}

impl UndistributedGainSource {
    /// A newly entered form: no topic answer is given yet.
    pub(crate) fn printed(form: Form2439Paper) -> Self {
        Self { form, section_1202_exclusion: None, special: false }
    }
    /// The topic answers of an accepted source survive a correction of its paper.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.section_1202_exclusion = accepted.section_1202_exclusion;
        self.special = accepted.special;
    }
}

// One-way: every printed box either reaches the internal model or is named as
// not read by any calculation, so a new box cannot be forgotten.
impl From<&UndistributedGainSource> for UndistributedGain {
    fn from(source: &UndistributedGainSource) -> Self {
        let UndistributedGainSource { form, section_1202_exclusion, special } = source;
        let Form2439 {
            // The parties name and assign the document; no figure reads them.
            fund: _,
            shareholder: _,
            gain,
            unrecaptured,
            small_business,
            collectibles,
            tax_paid,
        } = &Form2439::from(form);
        UndistributedGain {
            gain: *gain,
            unrecaptured: *unrecaptured,
            small_business: *small_business,
            section_1202_exclusion: *section_1202_exclusion,
            collectibles: *collectibles,
            tax_paid: *tax_paid,
            special: *special,
        }
    }
}

/// Applicable personal answers for Form 1099-DIV
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct DivCalls {
    /// Allowed choices for this answer
    pub nominee: Vec<i32>,
    /// Allowed choices for this answer
    pub after_death: Vec<i32>,
    /// Allowed choices for this answer
    pub reit_eligible: Vec<i32>,
    /// Allowed choices for this answer
    pub section_1202_exclusion: Vec<i32>,
}

/// Reported personal allocation for this statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum DivNomineeAnswer {
    /// Enter the allocation
    Reported {
        /// Allocation
        value: crate::DividendNominee,
    },
}
impl From<crate::DividendNominee> for DivNomineeAnswer {
    fn from(value: crate::DividendNominee) -> Self {
        Self::Reported { value }
    }
}
impl DivNomineeAnswer {
    pub(crate) fn record(&self) -> &crate::DividendNominee {
        let Self::Reported { value } = self;
        value
    }
}

/// Reported personal allocation for this statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum DivAfterDeathAnswer {
    /// Enter the allocation
    Reported {
        /// Allocation
        value: crate::DividendsAfterDeath,
    },
}
impl From<crate::DividendsAfterDeath> for DivAfterDeathAnswer {
    fn from(value: crate::DividendsAfterDeath) -> Self {
        Self::Reported { value }
    }
}
impl DivAfterDeathAnswer {
    pub(crate) fn record(&self) -> &crate::DividendsAfterDeath {
        let Self::Reported { value } = self;
        value
    }
}

/// Holding-period and payment-obligation eligibility for box 5 dividends
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum DividendEligibility {
    /// No
    No,
    /// Yes
    Yes,
}
impl From<bool> for DividendEligibility {
    fn from(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }
}

impl core::ops::Deref for DivNomineeAnswer {
    type Target = crate::DividendNominee;
    fn deref(&self) -> &Self::Target {
        self.record()
    }
}
impl core::ops::DerefMut for DivNomineeAnswer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self::Reported { value } = self;
        value
    }
}

impl core::ops::Deref for DivAfterDeathAnswer {
    type Target = crate::DividendsAfterDeath;
    fn deref(&self) -> &Self::Target {
        self.record()
    }
}
impl core::ops::DerefMut for DivAfterDeathAnswer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self::Reported { value } = self;
        value
    }
}
