// Direct-property page identity and recovery-page conversion.
pub use crate::asset_model::{
    AssetConvention, AssetYearHistory, AutoRecoveryHistory, HistoricalAssetRecords,
    HistoricalAutoKind, OriginalAssetMethod, OriginalBonusTreatment, UseFraction,
};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Actual independently reported AMT unused section179 carry
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AssetAmt179CarryAnswer {
    /// Actual AMT recovery record
    Records {
        /// AMT unused elected cost remaining before this year's income limit
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}
impl AssetAmt179CarryAnswer {
    pub(crate) fn from_amount(value: Option<i64>) -> Option<Self> {
        value.map(|amount| Self::Records { amount })
    }
    pub(crate) fn amount(value: &Option<Self>) -> Option<i64> {
        value.as_ref().map(Self::value)
    }
    pub(crate) fn value(&self) -> i64 {
        let Self::Records { amount } = self;
        *amount
    }
    pub(crate) fn page_choices(history: &crate::AssetHistory) -> Vec<i32> {
        if matches!(history, crate::AssetHistory::Continuing { first_year_179, .. } if *first_year_179 > 0)
        {
            alloc::vec![0]
        } else {
            Vec::new()
        }
    }
}

/// Ordinary property and its independently reported AMT income-limit carry
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("CanonicalAssetAmtCarryOptions")]
pub struct MacrsAssetPage {
    /// Property use throughout your ownership
    /// Use the lease and property-use records for this sold property.
    /// [NullText]
    /// Property-use records are unavailable
    /// [SetText]
    /// The property-use records show
    #[libertas_constrained_by("^<MacrsAssetPage>.allowed_farm_rental_use")]
    pub farm_rental_use: Option<crate::FarmRentalPropertyUse>,
    /// Applicable property-use choices supplied by the Farming editor
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_farm_rental_use: Vec<i32>,

    /// Property facts
    #[libertas_calculation_request("CanonicalAssetAmtCarryOptions")]
    pub financial: crate::GdsPropertyPage,
    /// Actual AMT unused elected-cost section179 carry
    /// Complete annual recovery records supply this amount without another question.
    /// [NullText]
    /// No separately supplied actual AMT carry
    /// [SetText]
    /// Actual AMT unused elected-cost carry
    #[libertas_constrained_by("^<MacrsAssetPage>.allowed_amt_carry")]
    pub amt_section179_income_carry: Option<AssetAmt179CarryAnswer>,
    /// Applicable actual AMT source answer; an unsaved ordinary row starts empty.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_asset_amt_carry_choices")]
    pub allowed_amt_carry: Vec<i32>,
}
impl From<crate::BusinessAsset> for MacrsAssetPage {
    fn from(financial: crate::BusinessAsset) -> Self {
        let allowed_amt_carry = financial.amt_carry_choices();
        let amt_section179_income_carry = if allowed_amt_carry.is_empty() {
            None
        } else {
            AssetAmt179CarryAnswer::from_amount(financial.amt_section179_income_carry)
        };
        Self {
            farm_rental_use: None,
            allowed_farm_rental_use: Vec::new(),
            financial: financial.try_into().expect("ordinary GDS editor route"),
            amt_section179_income_carry,
            allowed_amt_carry,
        }
    }
}
impl From<MacrsAssetPage> for crate::BusinessAsset {
    fn from(value: MacrsAssetPage) -> Self {
        let mut financial: crate::BusinessAsset = value.financial.into();
        if !value.allowed_farm_rental_use.is_empty() {
            financial.farm_rental_use = value.farm_rental_use;
        }
        if !matches!(financial.history, crate::EquipmentHistory::Continuing { initial_section179, .. } if initial_section179 > 0)
        {
            financial.amt_section179_income_carry = None;
        } else if value.amt_section179_income_carry.is_some()
            || !financial.amt_carry_choices().is_empty()
        {
            // Supplied answers still undergo the native reconciliation check.
            financial.amt_section179_income_carry =
                AssetAmt179CarryAnswer::amount(&value.amt_section179_income_carry);
        } else if financial.amt_section179_carry().is_err() {
            // Complete annual facts now derive the carry, so the null page
            // answer withdraws a stale hidden override, not the annual facts.
            financial.amt_section179_income_carry = None;
        }
        financial
    }
}
impl core::ops::Deref for MacrsAssetPage {
    type Target = crate::GdsPropertyPage;
    fn deref(&self) -> &Self::Target {
        &self.financial
    }
}
impl core::ops::DerefMut for MacrsAssetPage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.financial
    }
}

/// Property's actual recovery system
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CanonicalAssetPage {
    /// MACRS equipment and real property
    Gds {
        /// Ordinary property facts and elections
        facts: MacrsAssetPage,
    },
    /// Actual non-MACRS property
    NonMacrs {
        /// Original property facts and recovery sources
        facts: crate::NonMacrsAssetPage,
    },
}
impl From<crate::BusinessAsset> for CanonicalAssetPage {
    fn from(value: crate::BusinessAsset) -> Self {
        if value.nonmacrs() {
            Self::NonMacrs { facts: value.into() }
        } else {
            Self::Gds { facts: value.into() }
        }
    }
}
impl From<CanonicalAssetPage> for crate::BusinessAsset {
    fn from(value: CanonicalAssetPage) -> Self {
        match value {
            CanonicalAssetPage::Gds { facts } => facts.into(),
            CanonicalAssetPage::NonMacrs { facts } => facts.into(),
        }
    }
}
impl CanonicalAssetPage {
    #[cfg(test)]
    pub(crate) fn macrs(&self) -> Option<&MacrsAssetPage> {
        match self {
            Self::Gds { facts } => Some(facts),
            _ => None,
        }
    }
    #[cfg(test)]
    pub(crate) fn macrs_mut(&mut self) -> Option<&mut MacrsAssetPage> {
        match self {
            Self::Gds { facts } => Some(facts),
            _ => None,
        }
    }
    pub(crate) fn native(&self) -> alloc::borrow::Cow<'_, crate::BusinessAsset> {
        match self {
            Self::Gds { facts } => alloc::borrow::Cow::Owned(facts.financial.clone().into()),
            Self::NonMacrs { facts } => alloc::borrow::Cow::Owned(facts.clone().into()),
        }
    }
    pub(crate) fn elected179(&self) -> i64 {
        self.native().elected179()
    }
}
