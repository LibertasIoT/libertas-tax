#[allow(unused_imports)]
use super::*;

pub use crate::dividend_source::*;

/// Printed dividend values used by internal calculations
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Div1099 {
    /// PAYER'S name, TIN and address
    pub payer: crate::PrintedParty,
    /// RECIPIENT'S name, TIN and address
    pub recipient: crate::PrintedParty,
    /// Box 1a total ordinary dividends
    pub ordinary: i64,
    /// Box 1b qualified dividends
    pub qualified: i64,
    /// Box 3 nondividend distributions
    pub nondividend_distributions: i64,
    /// Box 9 cash liquidation distributions
    pub cash_liquidation: i64,
    /// Box 10 noncash liquidation distributions
    pub noncash_liquidation: i64,
    /// Box 2e section 897 ordinary dividends included in box 1a
    pub section_897_ordinary: i64,
    /// Box 2f section 897 capital gain included in box 2a
    pub section_897_capital: i64,
    /// Box 8 foreign country or U.S. territory
    pub foreign_country: String,
    /// Total long-term capital gain distributions (Form 1099-DIV box 2a)
    pub capital_distributions: i64,
    /// Unrecaptured section 1250 gain included in box 2a (box 2b)
    /// This component is not additional income. Enter zero if the payer reports none.
    pub unrecaptured_distributions: i64,
    /// Collectibles (28%) gain included in box 2a (box 2d)
    /// This is part of the total above, not additional income. Enter zero if the payer reports none.
    pub collectible_distributions: i64,
    /// Box 2c section 1202 gain
    /// Part of box 2a, not additional income.
    pub section_1202_gain: i64,
    /// Federal income tax withheld
    pub withholding: i64,
    /// Box 5 section 199A dividends
    /// Already included in box 1a ordinary dividends; not box 1b qualified dividends or capital gain distributions.
    pub section199a: i64,
    /// Box 7 foreign tax paid
    pub foreign_tax: i64,
    /// Box 12 exempt-interest dividends
    pub exempt_dividends: i64,
    /// Box 13 specified private activity bond interest dividends
    pub private_activity_dividends: i64,
    /// State boxes 14–16 and supplemental information
    pub state_details: crate::InvestmentStateFields,
}

// IRS Form 1099-DIV, https://www.irs.gov/pub/irs-pdf/f1099div.pdf
impl From<&Form1099DivPaper> for Div1099 {
    fn from(paper: &Form1099DivPaper) -> Self {
        let Form1099DivPaper {
            // Informational boxes are retained on the paper; no existing figure reads them.
            corrected: _,
            calendar_year: _,
            payer_telephone: _,
            account_number: _,
            investment_expenses: _,
            fatca: _,
            payer,
            recipient,
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
            state_details,
        } = paper;
        Self {
            payer: payer.clone(),
            recipient: recipient.clone(),
            ordinary: ordinary.unwrap_or(0),
            qualified: qualified.unwrap_or(0),
            nondividend_distributions: nondividend_distributions.unwrap_or(0),
            cash_liquidation: cash_liquidation.unwrap_or(0),
            noncash_liquidation: noncash_liquidation.unwrap_or(0),
            section_897_ordinary: section_897_ordinary.unwrap_or(0),
            section_897_capital: section_897_capital.unwrap_or(0),
            foreign_country: foreign_country.clone(),
            capital_distributions: capital_distributions.unwrap_or(0),
            unrecaptured_distributions: unrecaptured_distributions.unwrap_or(0),
            collectible_distributions: collectible_distributions.unwrap_or(0),
            section_1202_gain: section_1202_gain.unwrap_or(0),
            withholding: withholding.unwrap_or(0),
            section199a: section199a.unwrap_or(0),
            foreign_tax: foreign_tax.unwrap_or(0),
            exempt_dividends: exempt_dividends.unwrap_or(0),
            private_activity_dividends: private_activity_dividends.unwrap_or(0),
            state_details: state_details.clone(),
        }
    }
}

/// Reported components used to select personal follow-ups
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct DivCallBoxes {
    /// Reported capital distributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub capital_distributions: i64,
    /// Reported unrecaptured distributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unrecaptured_distributions: i64,
    /// Reported collectible distributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub collectible_distributions: i64,
    /// Reported section 1202 gain
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_1202_gain: i64,
    /// Reported section199a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section199a: i64,
    /// Reported exempt dividends
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub exempt_dividends: i64,
    /// Reported private activity dividends
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity_dividends: i64,
    /// Reported foreign tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub foreign_tax: i64,
    /// Reported nondividend distributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub nondividend_distributions: i64,
    /// Reported cash liquidation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cash_liquidation: i64,
    /// Reported noncash liquidation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub noncash_liquidation: i64,
}

/// Form 1099-DIV boxes that select personal follow-ups
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct DivBoxes {
    /// Reported qualified
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified: i64,

    /// Reported ordinary
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub ordinary: i64,
    /// Other reported components
    pub more: DivCallBoxes,
}

impl From<&Div1099> for DivBoxes {
    fn from(value: &Div1099) -> Self {
        Self {
            ordinary: value.ordinary,
            qualified: value.qualified,
            more: DivCallBoxes {
                capital_distributions: value.capital_distributions,
                unrecaptured_distributions: value.unrecaptured_distributions,
                collectible_distributions: value.collectible_distributions,
                section_1202_gain: value.section_1202_gain,
                section199a: value.section199a,
                exempt_dividends: value.exempt_dividends,
                private_activity_dividends: value.private_activity_dividends,
                foreign_tax: value.foreign_tax,
                nondividend_distributions: value.nondividend_distributions,
                cash_liquidation: value.cash_liquidation,
                noncash_liquidation: value.noncash_liquidation,
            },
        }
    }
}
impl From<&Form1099DivPaper> for DivBoxes {
    fn from(value: &Form1099DivPaper) -> Self {
        Self {
            ordinary: value.ordinary.unwrap_or(0),
            qualified: value.qualified.unwrap_or(0),
            more: DivCallBoxes {
                capital_distributions: value.capital_distributions.unwrap_or(0),
                unrecaptured_distributions: value.unrecaptured_distributions.unwrap_or(0),
                collectible_distributions: value.collectible_distributions.unwrap_or(0),
                section_1202_gain: value.section_1202_gain.unwrap_or(0),
                section199a: value.section199a.unwrap_or(0),
                exempt_dividends: value.exempt_dividends.unwrap_or(0),
                private_activity_dividends: value.private_activity_dividends.unwrap_or(0),
                foreign_tax: value.foreign_tax.unwrap_or(0),
                nondividend_distributions: value.nondividend_distributions.unwrap_or(0),
                cash_liquidation: value.cash_liquidation.unwrap_or(0),
                noncash_liquidation: value.noncash_liquidation.unwrap_or(0),
            },
        }
    }
}
