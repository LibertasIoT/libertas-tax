use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Source of a manually documented Schedule A bond-premium deduction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BondPremiumSource {
    /// A different documented bond, with its own deduction records
    OtherDocumented {
        /// Instrument name or description, not an accepted record identity
        #[libertas_size(min = 1)]
        instrument: String,
    },
    /// An annual instrument already accepted on Interest and dividends
    AcceptedAnnualInstrument {
        /// Accepted instrument
        instrument: i64,
    },
}
