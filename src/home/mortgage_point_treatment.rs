use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// How the accepted point amount is deducted
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum MortgagePointTreatment {
    /// No points reported
    None,
    /// Points to buy your main home
    Purchase,
    /// Points spread over the loan's term
    Amortized {
        /// Loan term
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("month")]
        term_months: u32,
        /// The loan ended in 2026
        #[libertas_default(false)]
        ended: bool,
    },
    /// Points on a loan that paid to improve your main home
    Improvement {
        /// Loan term
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("month")]
        term_months: u32,
    },
}
