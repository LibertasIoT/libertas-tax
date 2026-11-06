use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Record choice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RecordChoice {
    /// Record identity
    /// A record's income identity; a CCC loan's is its own record identity.
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_hidden]
    pub id: i64,
    /// Choice
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}
