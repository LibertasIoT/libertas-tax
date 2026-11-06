use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// How the annual interest or OID amount is reported
#[derive(
    Clone,
    Copy,
    Default,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
)]
pub enum DebtReporting {
    /// Gross income with separately stated premium
    #[default]
    Gross,
    /// Already reduced by premium; never subtract it again
    Net,
}
