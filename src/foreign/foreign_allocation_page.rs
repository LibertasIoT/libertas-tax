use crate::{
    ForeignAllocationSource, ForeignInvestmentAsset, ForeignInvestmentAssets, ForeignStateIncome,
    ForeignStatePayment, ForeignStateTaxes, ForeignWithholding, InvestmentAssetBasis,
    InvestmentAssetSource, InvestmentAssetYield,
};
use alloc::{string::String, vec::Vec};
use libertas_macros::*;

/// Existing foreign-income source used by an allocation worksheet
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignAllocationSourcePage {
    /// Ordinary foreign-income record
    Direct {
        /// Exact unique country and payer label from ordinary foreign-income records
        #[libertas_size(min = 1)]
        label: String,
    },
    /// Fiduciary passive portfolio allocation
    Estate {
        /// Accepted estate or trust Schedule K-1
        #[libertas_enum_source("$.estate_foreign_sources")]
        parent: i64,
        /// Country on that K-1's foreign-source allocation
        #[libertas_size(min = 2, max = 3)]
        country: String,
        /// Withholding treatment of that allocation
        withholding: ForeignWithholding,
    },
}

/// Foreign gross income produced by one portfolio asset
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentAssetYieldPage {
    /// Existing ordinary foreign-income record or saved fiduciary allocation
    pub source: crate::ForeignAllocationSourcePage,
    /// This asset's 2026 gross income from that source
    /// Before interest or other deductions. Already included in income and the foreign-source records; do not enter income again elsewhere.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub income: i64,
}

/// Income source of a portfolio investment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InvestmentAssetSourcePage {
    /// U.S.-source income
    Domestic,
    /// Foreign-source passive income
    Foreign {
        /// Existing ordinary foreign-income record or saved fiduciary allocation
        /// For an ordinary record, use its exact unique label. For a fiduciary allocation, select its saved K-1, country and withholding class. Link income generated, previously generated or reasonably expected from this asset; the account's location does not establish income source. Include a zero-income source record when necessary.
        source: crate::ForeignAllocationSourcePage,
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
        foreign: Vec<InvestmentAssetYieldPage>,
    },
}

/// Portfolio asset adjusted basis for interest allocation
/// Enter all taxable portfolio investment assets, not just assets bought with borrowed money. Exclude personal, business, passive-activity and exempt-income assets. Use tax adjusted basis, not market price or loan balance.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignInvestmentAssetPage {
    /// Investment or account label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Income source
    pub source: InvestmentAssetSourcePage,
    /// Basis records and averaging method
    pub basis: InvestmentAssetBasis,
}

/// Investment asset records for foreign interest allocation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignInvestmentAssetsPage {
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
        assets: Vec<ForeignInvestmentAssetPage>,
    },
}

/// Foreign income included in a state income-tax base
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignStateIncomePage {
    /// Existing ordinary foreign-income record or saved fiduciary allocation
    /// For an ordinary record, use its exact unique country and payer label. For a fiduciary allocation, select the saved K-1 and its actual country and withholding class. The link retains the existing category before federal high-tax reclassification.
    pub source: crate::ForeignAllocationSourcePage,
    /// Income attributed to this source by the state's tax law
    /// Use the state return and supporting income records underlying this payment, not a computed tax share. Exclude income exempted by that state; federal and state income amounts may differ.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub income: i64,
}

/// State or local income-tax payment and its source income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignStatePaymentPage {
    /// Taxing jurisdiction and source year
    #[libertas_size(min = 1)]
    pub label: String,
    /// Eligible income tax paid, already entered in itemized expenses
    /// Exclude sales and property taxes. Split payments only where their underlying state income differs; do not enter a federal deduction or duplicate the payment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub paid: i64,
    /// U.S.-source income in this payment's state tax base
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub domestic: i64,
    /// Foreign-source income in that state tax base
    /// Leave empty only when the tax relates entirely to U.S. income, including when the state exempts all foreign income. Identify each source once; all domestic and foreign amounts must reconcile to the income underlying this payment.
    pub foreign: Vec<ForeignStateIncomePage>,
}

/// Source allocation of itemized state and local income taxes
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignStateTaxesPage {
    // This carries unavailable state-source-income workpapers, not an unknown
    // confirmation. Federal AGI cannot establish another year's state tax base.
    // When needed, allocate stops at the source-fact boundary
    // ISSUE_SUPPLY_THE_SOURCE_INCOME_UNDERLYING_ITEMIZED_STATE_LOCAL_INCOME_TAX_5BB69CE3;
    // it never invents source income or treats unavailable records as zero.
    /// Not needed or not yet reviewed
    /// Required only when an itemized state/local income-tax deduction is used. Standard deduction and general sales taxes do not require these records.
    Unreviewed,
    /// Allocate from the state income records
    Records {
        /// Payments included in the itemized income-tax amount
        #[libertas_size(min = 1)]
        payments: Vec<ForeignStatePaymentPage>,
        /// Complete source attribution under each taxing jurisdiction's law
        /// Check each state return's source-income worksheet. No apportions the tax with your other deductions instead, with a notice. All payments reconcile to the itemized income-tax entry. Source income is nonnegative and reflects the tax base underlying each payment, including exemptions and resolved differences from federal law. No unresolved state apportionment, special election or loss allocation. Do not substitute current federal AGI when the payment relates to other income.
        confirmed: bool,
    },
}

impl From<ForeignAllocationSource> for ForeignAllocationSourcePage {
    fn from(value: ForeignAllocationSource) -> Self {
        match value {
            ForeignAllocationSource::Direct { label } => Self::Direct { label },
            ForeignAllocationSource::Estate { parent, country, withholding } => {
                Self::Estate { parent, country, withholding }
            }
        }
    }
}

impl From<ForeignAllocationSourcePage> for ForeignAllocationSource {
    fn from(value: ForeignAllocationSourcePage) -> Self {
        match value {
            ForeignAllocationSourcePage::Direct { label } => Self::Direct { label },
            ForeignAllocationSourcePage::Estate { parent, country, withholding } => {
                Self::Estate { parent, country, withholding }
            }
        }
    }
}

impl From<InvestmentAssetYield> for InvestmentAssetYieldPage {
    fn from(value: InvestmentAssetYield) -> Self {
        Self { source: value.source.into(), income: value.income }
    }
}

impl From<InvestmentAssetYieldPage> for InvestmentAssetYield {
    fn from(value: InvestmentAssetYieldPage) -> Self {
        Self { source: value.source.into(), income: value.income }
    }
}

impl From<InvestmentAssetSource> for InvestmentAssetSourcePage {
    fn from(value: InvestmentAssetSource) -> Self {
        match value {
            InvestmentAssetSource::Domestic => Self::Domestic,
            InvestmentAssetSource::Foreign { source } => Self::Foreign { source: source.into() },
            InvestmentAssetSource::Mixed { domestic, foreign } => {
                Self::Mixed { domestic, foreign: foreign.into_iter().map(Into::into).collect() }
            }
        }
    }
}

impl From<InvestmentAssetSourcePage> for InvestmentAssetSource {
    fn from(value: InvestmentAssetSourcePage) -> Self {
        match value {
            InvestmentAssetSourcePage::Domestic => Self::Domestic,
            InvestmentAssetSourcePage::Foreign { source } => {
                Self::Foreign { source: source.into() }
            }
            InvestmentAssetSourcePage::Mixed { domestic, foreign } => {
                Self::Mixed { domestic, foreign: foreign.into_iter().map(Into::into).collect() }
            }
        }
    }
}

impl From<ForeignInvestmentAsset> for ForeignInvestmentAssetPage {
    fn from(value: ForeignInvestmentAsset) -> Self {
        Self { label: value.label, source: value.source.into(), basis: value.basis }
    }
}

impl From<ForeignInvestmentAssetPage> for ForeignInvestmentAsset {
    fn from(value: ForeignInvestmentAssetPage) -> Self {
        Self { label: value.label, source: value.source.into(), basis: value.basis }
    }
}

impl From<ForeignInvestmentAssets> for ForeignInvestmentAssetsPage {
    fn from(value: ForeignInvestmentAssets) -> Self {
        match value {
            ForeignInvestmentAssets::Unreviewed => Self::Unreviewed,
            ForeignInvestmentAssets::TaxBook { assets } => {
                Self::TaxBook { assets: assets.into_iter().map(Into::into).collect() }
            }
        }
    }
}

impl From<ForeignInvestmentAssetsPage> for ForeignInvestmentAssets {
    fn from(value: ForeignInvestmentAssetsPage) -> Self {
        match value {
            ForeignInvestmentAssetsPage::Unreviewed => Self::Unreviewed,
            ForeignInvestmentAssetsPage::TaxBook { assets } => {
                Self::TaxBook { assets: assets.into_iter().map(Into::into).collect() }
            }
        }
    }
}

impl From<ForeignStateIncome> for ForeignStateIncomePage {
    fn from(value: ForeignStateIncome) -> Self {
        Self { source: value.source.into(), income: value.income }
    }
}

impl From<ForeignStateIncomePage> for ForeignStateIncome {
    fn from(value: ForeignStateIncomePage) -> Self {
        Self { source: value.source.into(), income: value.income }
    }
}

impl From<ForeignStatePayment> for ForeignStatePaymentPage {
    fn from(value: ForeignStatePayment) -> Self {
        Self {
            label: value.label,
            paid: value.paid,
            domestic: value.domestic,
            foreign: value.foreign.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ForeignStatePaymentPage> for ForeignStatePayment {
    fn from(value: ForeignStatePaymentPage) -> Self {
        Self {
            label: value.label,
            paid: value.paid,
            domestic: value.domestic,
            foreign: value.foreign.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ForeignStateTaxes> for ForeignStateTaxesPage {
    fn from(value: ForeignStateTaxes) -> Self {
        match value {
            ForeignStateTaxes::Unreviewed => Self::Unreviewed,
            ForeignStateTaxes::Records { payments, confirmed } => Self::Records {
                payments: payments.into_iter().map(Into::into).collect(),
                confirmed,
            },
        }
    }
}

impl From<ForeignStateTaxesPage> for ForeignStateTaxes {
    fn from(value: ForeignStateTaxesPage) -> Self {
        match value {
            ForeignStateTaxesPage::Unreviewed => Self::Unreviewed,
            ForeignStateTaxesPage::Records { payments, confirmed } => Self::Records {
                payments: payments.into_iter().map(Into::into).collect(),
                confirmed,
            },
        }
    }
}
