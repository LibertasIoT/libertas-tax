use crate::annual_debt_source::{AnnualDebtRecords, DebtPremiumOffset};
use crate::debt_reporting_source::DebtReporting;
use crate::validation::ValidationIssue;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Interest belonging to someone outside this return
#[derive(
    Clone, Default, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct InterestNominee {
    /// Taxable premium belonging to the other owner
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Treasury premium belonging to the other owner
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub treasury_bond_premium: i64,
    /// Exempt premium belonging to the other owner
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,
    /// Purchased accrued interest belonging to the other owner
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub accrued_interest: i64,
    /// Current-election market discount belonging to the other owner
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Taxable interest or OID belonging to others, excluding U.S. obligations
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxable: i64,
    /// U.S. savings-bond and Treasury interest belonging to others
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub us_obligations: i64,
    /// Exempt interest belonging to others
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Private-activity amount within the exempt nominee part
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Foreign tax associated with those other owners' income
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_tax: i64,
}

/// Dividend components belonging to someone outside this return
#[derive(
    Clone, Default, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct DividendNominee {
    /// Section897 subset of ordinary dividends
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_897_ordinary: i64,
    /// Section897 subset of capital distributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_897_capital: i64,
    /// Ordinary dividends
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub ordinary: i64,
    /// Qualified part of ordinary dividends
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified: i64,
    /// Capital gain distributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub capital: i64,
    /// Unrecaptured section 1250 part
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured: i64,
    /// Collectibles part
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub collectible: i64,
    /// Section 1202 part
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_1202: i64,
    /// Section 199A dividends
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_199a: i64,
    /// Exempt-interest dividends
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Private-activity part of exempt dividends
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Foreign tax belonging to others
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_tax: i64,
}

/// OID statement components belonging to someone outside this return
#[derive(
    Clone, Default, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct OidNominee {
    /// Box 1 OID
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub oid: i64,
    /// Box 2 other periodic interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_periodic_interest: i64,
    /// Box 8 Treasury OID, including signed deflation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    #[libertas_default(0)]
    pub treasury_oid: i64,
    /// Current-election market discount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub market_discount: i64,
    /// Acquisition premium belonging to this owner
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub acquisition_premium: i64,
    /// Taxable bond premium belonging to this owner
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub bond_premium: i64,
    /// Exempt OID
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt: i64,
    /// Exempt premium belonging to this owner
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_exempt_bond_premium: i64,
    /// Private-activity part of exempt OID
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity: i64,
}

/// Personal reporting facts, not imported from the form
#[derive(
    Clone, Default, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct InterestAdjustments {
    /// Instrument worksheets replacing this payer’s calculation, with separate ownership and reporting facts
    /// [NullText]
    /// Use the payer-level interest amounts and adjustments
    /// [SetText]
    /// Replace the payer-level calculation with
    #[libertas_default($null)]
    pub instrument_records: Option<AnnualDebtRecords>,
    /// Actual taxable-premium offset already applied to retained payer net income, after outside-owner allocation
    /// [NullText]
    /// No taxable-premium offset has been documented for the retained net amount
    /// [SetText]
    /// The documented taxable-premium offset already applied is
    pub net_premium_offset: Option<DebtPremiumOffset>,
    /// Savings-bond interest already included on an earlier return, within retained box 3
    /// Include a documented prior owner's final-return inclusion only once. Not an OID adjustment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub previously_reported: i64,
    /// Whether reported income has already been reduced by the reported premium
    #[libertas_default(Gross)]
    pub reporting: DebtReporting,
}

/// Buyer whose home purchase you financed
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SellerMortgageParty {
    /// Buyer name
    #[libertas_size(min = 1)]
    pub buyer: alloc::string::String,
    /// Buyer's Social Security number or taxpayer identification number
    /// Enter all nine digits, including leading zeroes, without punctuation.
    // Legal identifier: the buyer taxpayer identification number has nine decimal digits.
    #[libertas_size(min = 9, max = 9)]
    #[libertas_pattern(r"[0-9]{9}")]
    pub buyer_tin: alloc::string::String,
    /// Buyer address
    #[libertas_size(min = 1)]
    pub buyer_address: alloc::string::String,
}
impl SellerMortgageParty {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        let field = if self.buyer.trim().is_empty() {
            Some("buyer")
        } else if self.buyer_tin.len() != 9 || !self.buyer_tin.bytes().all(|b| b.is_ascii_digit()) {
            // Legal TIN format: exactly nine ASCII digits.
            Some("buyer_tin")
        } else if self.buyer_address.trim().is_empty() {
            Some("buyer_address")
        } else {
            None
        };
        field.map(|field| ValidationIssue::new("SELLER_MORTGAGE_BUYER_INVALID").at_field(field))
    }
}
