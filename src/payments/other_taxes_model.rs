#[allow(unused_imports)]
use super::*;
// Each tax kind carries its editable view. Leaving a kind out clears its
// corresponding stored records; the list supplies the whole answer.
/// Other taxes
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OtherTaxes {
    /// Corrective Trump-account distributions reported on Form 1099-R
    #[libertas_read_only]
    pub trump_returns: Vec<crate::TrumpExcessDocumentView>,
    /// Other taxes
    /// Add each that applies to 2026. Leave the list empty if none does.
    #[libertas_unordered]
    /// ----
    /// Tax
    pub taxes: Vec<OtherTax>,
}
/// Tax
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum OtherTax {
    /// Household employees, such as a nanny or housekeeper (Schedule H)
    /// Social Security, Medicare and federal unemployment tax on wages you paid, and any income tax you withheld from them.
    HouseholdEmployees {
        /// Household employers
        /// One entry for each of you who employed household workers.
        #[libertas_size(min = 1)]
        /// ----
        /// Employer
        employers: Vec<crate::HouseholdEmployerView>,
    },
    /// Contributions to your own Trump account above the $5,000 limit (Form 5329 Parts X and XI)
    /// Only on the return of the child the account is for. A parent's contribution to a child's account is not entered on the parent's return.
    TrumpAccounts {
        /// Trump accounts
        #[libertas_size(min = 1)]
        /// ----
        /// Account
        accounts: Vec<crate::TrumpAccountView>,
    },
    /// Selling or giving away a home bought with a mortgage credit certificate or a mortgage revenue bond loan (Form 8828)
    /// Within nine years of the loan, the federal mortgage subsidy can be recaptured. The lender's recapture notice gives the figures asked.
    MortgageSubsidyRecapture {
        /// Homes
        #[libertas_size(min = 1)]
        /// ----
        /// Home
        homes: Vec<MortgageSubsidyEntryView>,
    },
    /// Excess contributions to your own Coverdell ESA (Form 5329 Part V)
    /// Enter the beneficiary's accounts on that beneficiary's return, including retained prior excess.
    CoverdellExcess {
        /// Beneficiaries on this return
        #[libertas_size(min = 1)]
        beneficiaries: Vec<crate::CoverdellExcessPage>,
    },
}
