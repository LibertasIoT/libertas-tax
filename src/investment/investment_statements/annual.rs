use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub use crate::bond_premium_source::BondPremiumSource;

/// Accepted annual instrument that a manually documented deduction can name
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BondPremiumInstrument {
    /// Accepted instrument identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Payer and instrument
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
    /// Current annual excess deduction computed from this instrument
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    pub computed_deduction: i64,
}
