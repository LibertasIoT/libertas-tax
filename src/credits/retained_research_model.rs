use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Retained operating-loss portions belonging to actual current businesses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ResearchLossParts {
    /// Identified business portions
    Parts { sources: Vec<ResearchLossPart> },
}
/// One business's portion of a retained operating loss
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ResearchLossPart {
    pub source: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub amount: i64,
}
/// Retained regular capital-loss portions belonging to actual businesses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ResearchCapitalParts {
    /// Identified business portions
    Parts { sources: Vec<ResearchCapitalPart> },
}
/// One business's regular short- and long-term carryover portions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ResearchCapitalPart {
    pub source: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub short: i64,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub long: i64,
}
