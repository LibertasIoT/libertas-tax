use crate::common::Owner;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Optional self-employment method election
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SeElection {
    /// Person making the election
    pub owner: Owner,
    /// Farm optional method, elected on the Farming page
    pub farm: bool,
    /// Nonfarm optional method and retained earnings history, elected on Self-employment
    /// [NullText]
    /// Do not elect the nonfarm optional method
    /// [SetText]
    /// The nonfarm optional method uses this retained earnings history
    #[libertas_default($null)]
    pub nonfarm: Option<NonfarmSeHistory>,
}
/// Nonfarm optional-method history
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct NonfarmSeHistory {
    /// Actual 2023 net earnings from self-employment
    /// Copy retained Schedule SE actual earnings, not optional earnings or gross receipts. Include both farming and nonfarming actual earnings.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub earnings_third_prior_year: i64,
    /// Actual 2024 net earnings from self-employment
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub earnings_second_prior_year: i64,
    /// Actual 2025 net earnings from self-employment
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub earnings_prior_year: i64,
    /// Earlier years using the nonfarm optional method
    /// List every prior use, including nonconsecutive years. Farm-only elections do not count toward the five-year lifetime limit.
    // Tax structure: the nonfarm optional method has a five-year lifetime limit; a current election allows at most four prior uses.
    #[libertas_size(max = 4)]
    pub prior_years: Vec<u32>,
}
impl NonfarmSeHistory {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        for (field, value) in [
            ("earnings_third_prior_year", self.earnings_third_prior_year),
            ("earnings_second_prior_year", self.earnings_second_prior_year),
            ("earnings_prior_year", self.earnings_prior_year),
        ] {
            if !(-100000000000..=100000000000).contains(&value) {
                return Some(
                    ValidationIssue::new("NONFARM_OPTIONAL_HISTORY_INVALID").at_field(field),
                );
            }
        }
        if self.prior_years.len() >= 5 {
            return Some(
                ValidationIssue::new("NONFARM_OPTIONAL_HISTORY_INVALID").at_field("prior_years"),
            );
        }
        if let Some((i, _)) = self
            .prior_years
            .iter()
            .enumerate()
            .find(|(i, y)| !(1951..=2025).contains(*y) || self.prior_years[..*i].contains(y))
        {
            return Some(
                ValidationIssue::new("NONFARM_OPTIONAL_HISTORY_INVALID")
                    .at_list_item("prior_years", i),
            );
        }
        None
    }
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
}
