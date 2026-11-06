use crate::dates::source_valid_date;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Foreign gross income produced by one portfolio asset
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentAssetYield {
    /// Existing ordinary foreign-income record or saved fiduciary allocation
    pub source: crate::ForeignAllocationSource,
    /// This asset's 2026 gross income from that source
    /// Before interest or other deductions. Already included in income and the foreign-source records; do not enter income again elsewhere.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub income: i64,
}

/// Income source of a portfolio investment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InvestmentAssetSource {
    /// U.S.-source income
    Domestic,
    /// Foreign-source passive income
    Foreign {
        /// Existing ordinary foreign-income record or saved fiduciary allocation
        /// For an ordinary record, use its exact unique label. For a fiduciary allocation, select its saved K-1, country and withholding class. Link income generated, previously generated or reasonably expected from this asset; the account's location does not establish income source. Include a zero-income source record when necessary.
        source: crate::ForeignAllocationSource,
    },
    /// More than one income source
    /// Attribute this asset's basis using its current-year gross-income yield. Use source records before any expense deductions, with the same income characterization for regular tax and AMT. Unresolved or zero total yield needs a separate characterization worksheet.
    Mixed {
        /// U.S.-source gross income from this asset
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        domestic: i64,
        /// Foreign gross income from this asset
        #[libertas_size(min = 1)]
        foreign: Vec<InvestmentAssetYield>,
    },
}

/// Portfolio asset adjusted basis for interest allocation
/// Enter all taxable portfolio investment assets, not just assets bought with borrowed money. Exclude personal, business, passive-activity and exempt-income assets. Use tax adjusted basis, not market price or loan balance.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignInvestmentAsset {
    /// Investment or account label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Income source
    pub source: InvestmentAssetSource,
    /// Basis records and averaging method
    pub basis: InvestmentAssetBasis,
}

/// Dated change in an investment asset's adjusted basis
/// The values apply from this date through the day before the next change, or December 31 for the final record. Include every change, and enter zero while the asset is not held.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentBasisChange {
    /// Effective date in 2026
    #[libertas_date_only]
    // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
    #[libertas_number(min = 20260101, max = 20261231)]
    pub date: u32,
    /// Regular-tax adjusted basis
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub regular: i64,
    /// AMT adjusted basis
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt: i64,
}

/// Investment adjusted-basis averaging
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InvestmentAssetBasis {
    /// Beginning and end of year
    /// Use only when this average does not substantially distort the year's asset values.
    Average {
        /// Regular basis at the beginning of 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        beginning: i64,
        /// Regular basis at the end of 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        ending: i64,
        /// AMT basis at the beginning of 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amt_beginning: i64,
        /// AMT basis at the end of 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amt_ending: i64,
    },
    /// Daily weighting from complete basis history
    /// Use when endpoint averaging substantially distorts holdings during the year. The App weights each adjusted basis by its calendar days; do not enter a computed annual average or market value.
    Daily {
        /// All basis changes in date order, starting January 1
        /// Start with zero if the asset was acquired later. Include zero after disposal, and every intervening basis change. Combine changes effective on the same date into one record.
        // Calendar structure: dated entries are ordered and nonoverlapping within 2026 (365 days).
        #[libertas_size(min = 1, max = 365)]
        changes: Vec<InvestmentBasisChange>,
    },
}
impl InvestmentAssetBasis {
    // Common denominator730 keeps endpoint halves and 365-day averages
    // on the same exact scale without rounding an annual average first.
    pub(crate) fn weights(&self) -> Option<[i64; 2]> {
        let money = |v: i64| (0..=100000000000).contains(&v);
        match self {
            Self::Average { beginning, ending, amt_beginning, amt_ending } => {
                [*beginning, *ending, *amt_beginning, *amt_ending]
                    .into_iter()
                    .all(money)
                    .then(|| [(beginning + ending) * 365, (amt_beginning + amt_ending) * 365])
            }
            Self::Daily { changes } => {
                if changes.is_empty()
                    || changes.len() > 365
                    || changes[0].date != 20260101
                    || changes.iter().enumerate().any(|(i, c)| {
                        !(20260101..=20261231).contains(&c.date)
                            || !source_valid_date(c.date)
                            || !money(c.regular)
                            || !money(c.amt)
                            || i > 0 && changes[i - 1].date >= c.date
                    })
                {
                    return None;
                }
                let mut result = [0; 2];
                for (i, c) in changes.iter().enumerate() {
                    let end = changes.get(i + 1).map_or(20270101, |v| v.date);
                    let days = i64::from(crate::dates::days_between(c.date, end));
                    result[0] += c.regular * days * 2;
                    result[1] += c.amt * days * 2;
                }
                Some(result)
            }
        }
    }
}

/// Investment asset records for foreign interest allocation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignInvestmentAssets {
    // This carries unavailable portfolio source records, not an unanswered
    // qualification gate. Current income does not establish historical tax-book
    // basis. Without the actual records, allocate attributes all investment
    // interest to foreign passive income with WARN_INVESTMENT_INTEREST_SOURCE_ASSUMED.
    /// No records
    /// Not needed without an investment-interest deduction or with the $5,000 small-foreign-income choice. Otherwise the App treats all your investment interest as relating to your foreign passive income, which can lower the credit, with a note.
    Unreviewed,
    /// Tax book values from portfolio records
    TaxBook {
        /// All taxable investment assets
        /// Your brokerage statements and basis records. Use tax adjusted basis, including periods of nonownership; daily records where beginning and end of year averaging distorts holdings. Leave out assets producing tax-exempt income. A partnership interest held as an investment is an asset with mixed sources by its income. The App uses tax book value; the alternative tax book value election is not offered.
        #[libertas_size(min = 1)]
        /// ----
        /// Asset
        assets: Vec<ForeignInvestmentAsset>,
    },
}
impl ForeignInvestmentAssets {
    pub(crate) fn valid(&self) -> bool {
        let label = |s: &str| !s.trim().is_empty();
        match self {
            Self::Unreviewed => true,
            Self::TaxBook { assets, .. } => {
                !assets.is_empty()
                    // Temporary arithmetic exclusion: allocate accumulates native
                    // basis weights, domestic yield and per-source reported yields.
                    && assets.len() <= 100
                    && assets.iter().all(|a| {
                        label(&a.label)
                            && a.basis.weights().is_some()
                            && match &a.source {
                                InvestmentAssetSource::Domestic => true,
                                InvestmentAssetSource::Foreign { source } => source.valid(),
                                InvestmentAssetSource::Mixed { domestic, foreign } => {
                                    (0..=100000000000).contains(domestic)
                                        && !foreign.is_empty()
                                        // Temporary arithmetic exclusion: allocate sums
                                        // mixed yields into native group/source columns.
                                        && foreign.len() <= 100
                                        && foreign.iter().enumerate().all(|(i, y)| {
                                            y.source.valid()
                                                && (0..=100000000000).contains(&y.income)
                                                && !foreign[..i]
                                                    .iter()
                                                    .any(|v| v.source.same_as(&y.source))
                                        })
                                }
                            }
                    })
            }
        }
    }
}
