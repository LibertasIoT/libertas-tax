use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::ratio::{allocate_remaining, dollars};
use crate::{FederalResult, ForeignOrdinaryTax, ForeignTaxCategory};
use alloc::{format, string::String, vec::Vec};
use libertas_macros::*;

/// Foreign gross income produced by one portfolio asset
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentAssetYield {
    /// Matching country and payer label in foreign-income records
    #[libertas_size(min = 1)]
    pub source: String,
    /// This asset's 2026 gross income from that source
    /// Before interest or other deductions. Already included in income and the foreign-source records; do not enter income again elsewhere.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub income: i64,
}

/// Income source of a portfolio investment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InvestmentAssetSource {
    /// U.S.-source income
    Domestic,
    /// Foreign-source passive income
    Foreign {
        /// Matching country and payer label in foreign-income records
        /// Use the exact unique label for the income generated, previously generated or reasonably expected from this asset. Physical location of the account does not establish income source. Include a zero-income source record when necessary.
        #[libertas_size(min = 1)]
        source: String,
    },
    /// More than one income source
    /// Attribute this asset's basis using its current-year gross-income yield. Use source records before any expense deductions, with the same income characterization for regular tax and AMT. Unresolved or zero total yield needs a separate characterization worksheet.
    Mixed {
        /// U.S.-source gross income from this asset
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    pub regular: i64,
    /// AMT adjusted basis
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        beginning: i64,
        /// Regular basis at the end of 2026
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        ending: i64,
        /// AMT basis at the beginning of 2026
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amt_beginning: i64,
        /// AMT basis at the end of 2026
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
    fn weights(&self) -> Option<[i64; 2]> {
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
                            || !crate::federal_rules::valid_date(c.date)
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
        /// ----
        /// Asset
        #[libertas_size(min = 1)]
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
                                InvestmentAssetSource::Foreign { source } => label(source),
                                InvestmentAssetSource::Mixed { domestic, foreign } => {
                                    (0..=100000000000).contains(domestic)
                                        && !foreign.is_empty()
                                        // Temporary arithmetic exclusion: allocate sums
                                        // mixed yields into native group/source columns.
                                        && foreign.len() <= 100
                                        && foreign.iter().enumerate().all(|(i, y)| {
                                            label(&y.source)
                                                && (0..=100000000000).contains(&y.income)
                                                && !foreign[..i]
                                                    .iter()
                                                    .any(|v| v.source == y.source)
                                        })
                                }
                            }
                    })
            }
        }
    }

    // Without portfolio records the interest cannot be apportioned: all of it
    // is treated as relating to the foreign passive income, by that income
    // (Pub. 514, "Interest expense"; conservative), with one note.
    pub(crate) fn allocate(
        &self,
        allowed: [i64; 2],
        sources: &[ForeignOrdinaryTax],
        domestic_income: i64,
        passive_gross: [i64; crate::form_1116::SLOTS],
        r: &mut FederalResult,
    ) -> [[i64; crate::form_1116::SLOTS]; 2] {
        if allowed == [0; 2] {
            return [[0; crate::form_1116::SLOTS]; 2];
        }
        let Self::TaxBook { assets } = self else {
            let mut weights = [0; crate::form_1116::SLOTS];
            for group in crate::form_1116::PASSIVE_GROUPS {
                weights[group] = passive_gross[group];
            }
            if weights.iter().sum::<i64>() <= 0 {
                return [[0; crate::form_1116::SLOTS]; 2];
            }
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_INVESTMENT_INTEREST_SOURCE_ASSUMED"),
                crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            );
            let mut result = [[0; crate::form_1116::SLOTS]; 2];
            for context in 0..2 {
                result[context] =
                    allocate_remaining(allowed[context], &weights).try_into().unwrap();
            }
            return result;
        };
        let valid = self.valid()
            && sources
                .iter()
                .enumerate()
                .all(|(i, s)| !sources[..i].iter().any(|v| v.label == s.label));
        crate::require_localized!(
            r,
            valid,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Foreign investment interest allocation",
            "ISSUE_CONFIRM_COMPLETE_ORDINARY_PORTFOLIO_BASIS_RECORDS_APPROPRIATE_COMPL_4BD36A4B",
            "Enter valid portfolio basis records and unique foreign-income labels before allocating investment interest.",
        );
        if !valid {
            return [[0; crate::form_1116::SLOTS]; 2];
        }
        let source_group = |label: &str| {
            sources
                .iter()
                .find(|s| {
                    s.label == label && matches!(s.category, ForeignTaxCategory::Passive { .. })
                })
                .map(ForeignOrdinaryTax::group)
        };
        let mut weights = [[0; crate::form_1116::SLOTS]; 2];
        let mut domestic_yield = 0;
        let mut reported_yields = alloc::vec![0; sources.len()];
        for asset in assets {
            let mut yields = [0; crate::form_1116::SLOTS];
            match &asset.source {
                InvestmentAssetSource::Domestic => yields[crate::form_1116::DOMESTIC] = 1,
                InvestmentAssetSource::Foreign { source } => {
                    let Some(group) = source_group(source) else {
                        crate::require_localized!(
                            r,
                            false,
                            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                            "Foreign investment interest allocation",
                            "ISSUE_AN_INVESTMENT_ASSET_MUST_MATCH_A_UNIQUE_PASSIVE_FOREIGN_INCOME_SOUR_A971F68A",
                            "An investment asset must match a unique passive foreign-income source; employment income is not portfolio income.",
                        );
                        return [[0; crate::form_1116::SLOTS]; 2];
                    };
                    yields[group] = 1;
                }
                InvestmentAssetSource::Mixed { domestic, foreign } => {
                    yields[crate::form_1116::DOMESTIC] = *domestic;
                    domestic_yield += domestic;
                    for item in foreign {
                        let Some(group) = source_group(&item.source) else {
                            crate::require_localized!(
                                r,
                                false,
                                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                                "Foreign investment interest allocation",
                                "ISSUE_A_MIXED_ASSET_S_INCOME_MUST_MATCH_A_UNIQUE_PASSIVE_FOREIGN_INCOME_S_7FB18835",
                                "A mixed asset's income must match a unique passive foreign-income source.",
                            );
                            return [[0; crate::form_1116::SLOTS]; 2];
                        };
                        yields[group] += item.income;
                        let index = sources.iter().position(|s| s.label == item.source).unwrap();
                        reported_yields[index] += item.income;
                    }
                    if yields.iter().sum::<i64>() == 0 {
                        crate::require_localized!(
                            r,
                            false,
                            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                            "Foreign investment interest allocation",
                            "ISSUE_A_MIXED_SOURCE_ASSET_NEEDS_A_POSITIVE_CURRENT_YEAR_GROSS_INCOME_YIE_C7BB559B",
                            "A mixed-source asset needs a positive current-year gross-income yield to characterize its basis; unresolved or zero yield requires a separate worksheet.",
                        );
                        return [[0; crate::form_1116::SLOTS]; 2];
                    }
                }
            }
            // Both averaging methods use one common scale; mixed-source
            // characterization precedes whole-dollar interest allocation.
            for (context, basis) in asset.basis.weights().unwrap().into_iter().enumerate() {
                for (group, value) in allocate_remaining(basis, &yields).into_iter().enumerate() {
                    weights[context][group] += value;
                }
            }
        }
        crate::require_localized!(
            r,
            reported_yields
                .iter()
                .zip(sources)
                .all(|(yield_amount, source)| *yield_amount <= source.income),
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Foreign investment interest allocation",
            "ISSUE_COMBINED_MIXED_ASSET_GROSS_YIELDS_CANNOT_EXCEED_THEIR_ENTERED_FOREI_7E9019CA",
            "Combined mixed-asset gross yields cannot exceed their entered foreign-source income. Reconcile the source records without duplicating income.",
        );
        crate::require_localized!(
            r,
            dollars(domestic_yield) <= domestic_income,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Foreign investment interest allocation",
            "ISSUE_COMBINED_DOMESTIC_MIXED_ASSET_YIELD_CANNOT_EXCEED_ENTERED_DOMESTIC_710F1444",
            "Combined domestic mixed-asset yield cannot exceed entered domestic portfolio income.",
        );
        let mut result = [[0; crate::form_1116::SLOTS]; 2];
        for context in 0..2 {
            let total = weights[context].iter().sum::<i64>();
            crate::require_localized!(
                r,
                allowed[context] == 0 || total > 0,
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                "Foreign investment interest allocation",
                "ISSUE_A_DEDUCTIBLE_INTEREST_AMOUNT_REQUIRES_A_POSITIVE_INVESTMENT_ASSET_B_16FA0C18",
                "A deductible interest amount requires a positive investment-asset basis for that tax system; zero-basis or substantially distorted holdings require another valuation worksheet.",
            );
            if total == 0 {
                continue;
            }
            result[context] =
                allocate_remaining(allowed[context], &weights[context]).try_into().unwrap();
            let name = if context == 0 { "regular" } else { "amt" };
            emit_lines(r, &mut (), result[context].iter().enumerate(), |(group, amount), _, _| {
                Some(calculation_row(
                    None,
                    &format!("foreign_investment_{name}_group_{group}"),
                    LineValue::MoneyDollars(*amount),
                    &["credits.foreign_tax.investment_assets", "investment_interest_deduction"],
                    "https://www.irs.gov/publications/p514",
                    msg!("EXPLAIN_FOREIGN_INVESTMENT_N_GROUP_N"),
                ))
            });
        }
        result
    }
}
