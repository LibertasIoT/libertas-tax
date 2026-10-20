use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// What happened to the prize
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PrizeDisposition {
    /// Accepted or made available for your use
    Received,
    /// Refused before acceptance or use
    Refused,
    /// Entire qualifying achievement award transferred directly by the payer to charity
    DirectCharity {
        /// All section 74(b) requirements are met
        /// Check your award terms, designation and payer confirmation to answer. No taxes the prize as received. Religious, charitable, scientific, artistic, educational, literary or civic achievement; selected without entering; no substantial future services. Before presentation or use, you designated a governmental unit or qualifying charity and the payer transferred the entire award directly under that designation. Retain designation and payer confirmation. A donation after receiving the prize is not this exclusion; do not also deduct the excluded award as a charitable contribution.
        eligible: bool,
    },
}
