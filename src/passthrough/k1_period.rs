use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Entity's actual tax period
/// Use the beginning and ending dates on this entity's own source records. An unavailable period stays unknown; do not substitute the individual's calendar year or another entity's dates.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct K1EntityTaxPeriod {
    /// Beginning date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub began: u32,
    /// Ending date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub ended: u32,
}

impl K1EntityTaxPeriod {
    pub(crate) fn valid(&self) -> bool {
        crate::federal_rules::valid_date(self.began)
            && crate::federal_rules::valid_date(self.ended)
            && self.began <= self.ended
    }
}
