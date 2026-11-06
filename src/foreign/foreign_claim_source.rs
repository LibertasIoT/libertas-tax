use crate::foreign_source_basics::{
    ForeignAmtLimitation, ForeignInterestAllocation, ForeignOrdinaryTax, ForeignRateAdjustment,
    ForeignTaxBasis, ForeignTaxDeducted,
};
use crate::kernel::phase;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};
/// Foreign tax credit or deduction choice
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignTaxClaim {
    /// Credit for ordinary foreign income using Form 1116
    Ordinary {
        /// How you claim foreign taxes — Form 1116 Part II, Paid or Accrued
        basis: ForeignTaxBasis,
        /// Foreign income and tax by country
        /// One row for each country and category of foreign income already included once in Income, with the foreign tax on it. Employee pay, interest, dividends and royalties are entered with their amounts; a business run abroad and foreign-source investment sales are chosen from their own entries. A K-1's Schedule K-3 is read from its K-1 page; do not enter it here. The App allocates your deductions: the standard or itemized deductions by gross income, home mortgage interest by the Form 1116 worksheet, investment interest by your portfolio records or the $5,000 small-foreign-income choice, state income taxes by their source records; the expenses of your U.S. businesses, rentals, farms and gambling stay with U.S. income.
        /// ----
        /// Country and income category
        sources: Vec<ForeignOrdinaryTax>,
        /// Foreign tax situations that apply
        /// Check each that applies to you, or to your spouse on a joint return; leave all unchecked when none does. This estimate does not figure them, so a checked situation leaves the foreign tax credit out, with a note. A contested tax is not a situation: leave it out of the tax amounts until it is settled.
        // Each of the nine foreign-tax situation codes occurs at most once.
        #[libertas_size(max = 9)]
        situations: Vec<u32>,
        /// Unused foreign taxes from prior returns
        history: crate::ForeignCreditHistory,
        /// Prior-year treatment of newly generated excess taxes
        carryback: crate::ForeignCarryback,
        /// Alternative minimum tax foreign-credit limitation
        amt_limitation: ForeignAmtLimitation,
        /// Interest allocation method
        #[libertas_default(GrossIncome)]
        interest_allocation: ForeignInterestAllocation,
        /// Portfolio asset bases for investment-interest source allocation
        #[libertas_default(Unreviewed)]
        investment_assets: crate::ForeignInvestmentAssets,
        /// Prior loss accounts and current recapture choices
        #[libertas_default(None)]
        loss_history: crate::ForeignLossHistory,
        /// Source records for itemized state/local income taxes
        #[libertas_default(Unreviewed)]
        state_taxes: crate::ForeignStateTaxes,
        /// Qualified dividend and capital gain adjustment
        #[libertas_default(Exception)]
        rate_adjustment: ForeignRateAdjustment,
    },
    /// No foreign tax credit or deduction
    No,
    /// Small foreign tax credit from payee statements
    Statements {
        // None is only the server-discovered pending election. The page asks a
        // required bool and its accepted save always supplies Some(answer).
        /// All foreign gross income is passive income reported on qualified payee statements
        /// Includes all foreign income and taxes for both spouses on a joint return. This election is limited to $300 ($600 jointly), with no carryover to or from this election year. No refundable, contested, non-income or otherwise ineligible foreign taxes. Check every foreign income statement and your foreign-tax records. No claims no foreign tax credit, with a review note.
        /// [NullText]
        /// Eligibility has not yet been answered
        /// [SetText]
        /// The accepted eligibility answer is
        eligible: Option<bool>,
    },
    /// Deduct foreign income taxes on Schedule A line 6 instead
    /// No foreign tax credit is claimed this year. Foreign income taxes are outside the state and local tax limit and help only when you itemize. Retained tax on your Forms 1099-INT, 1099-DIV, qualified estate or trust payee statements and Schedules K-3 is included already, subject to the existing source and K-1 basis limits. The credit usually saves more; choose it yourself if you prefer it.
    Deduction {
        /// Other foreign income taxes paid in 2026
        /// Add only income taxes not already included from a Form 1099-INT, 1099-DIV, qualified estate or trust payee statement or Schedule K-3. Exclude taxes already deducted by a business, taxes credited in an earlier year, contested or refundable amounts, U.S. territory taxes (Taxes you paid), foreign real estate taxes, and sales or value-added taxes.
        #[libertas_unordered]
        /// ----
        /// Foreign tax
        taxes: Vec<ForeignTaxDeducted>,
    },
}

// The payee-statement election has no Form 1116 fractional limitation or
// carryover. Eligibility remains in extended_rules::foreign_credit; reuse this
// same liability cap when Form 1062 refigures the credit.
impl ForeignTaxClaim {
    // `documents` is the payee-statement tax the return itself shows, in
    // cents (extended_rules::payee_statement_taxes): the 1099 boxes and the
    // Schedule K-3 passive taxes. They are the election's only source.
    pub(crate) fn statement_credit(&self, regular: i64, documents: i64) -> Option<i64> {
        match self {
            Self::No | Self::Deduction { .. } => Some(0),
            Self::Statements { .. } => Some(phase::cap(
                crate::kernel::ratio::positive(regular),
                [crate::kernel::ratio::dollars(documents)],
            )),
            Self::Ordinary { .. } => None,
        }
    }
}
