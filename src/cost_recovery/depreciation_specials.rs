// Ordinary direct-property identity and original annual recovery sources.
// Special-route selectors remain in their later owner increments.
use crate::Asset179Carryover;
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Exact documented use ratio; no whole-percent rounding for tax
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct UseFraction {
    /// Units used for the stated legal purpose
    #[libertas_number(min = 0)]
    pub used: u32,
    /// All comparable units in the period
    #[libertas_number(min = 1)]
    pub total: u32,
}

/// Actual independently reported AMT unused section179 carry
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AssetAmt179CarryAnswer {
    /// Actual AMT recovery record
    Records {
        /// AMT unused elected cost remaining before this year's income limit
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
pub struct CanonicalAssetPage {
    /// Property facts
    #[libertas_calculation_request("CanonicalAssetAmtCarryOptions")]
    pub financial: crate::BusinessAsset,
    /// Actual AMT unused elected-cost section179 carry
    /// Complete annual recovery records supply this amount without another question.
    /// [NullText]
    /// No separately supplied actual AMT carry
    /// [SetText]
    /// Actual AMT unused elected-cost carry
    #[libertas_default($null)]
    #[libertas_constrained_by("^<CanonicalAssetPage>.allowed_amt_carry")]
    pub amt_section179_income_carry: Option<AssetAmt179CarryAnswer>,
    /// Applicable actual AMT source answer; an unsaved ordinary row starts empty.
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_amt_carry: Vec<i32>,
}
impl From<crate::BusinessAsset> for CanonicalAssetPage {
    fn from(financial: crate::BusinessAsset) -> Self {
        let allowed_amt_carry = financial.amt_carry_choices();
        let amt_section179_income_carry = if allowed_amt_carry.is_empty() {
            None
        } else {
            AssetAmt179CarryAnswer::from_amount(financial.amt_section179_income_carry)
        };
        Self { financial, amt_section179_income_carry, allowed_amt_carry }
    }
}
impl From<CanonicalAssetPage> for crate::BusinessAsset {
    fn from(value: CanonicalAssetPage) -> Self {
        let mut financial = value.financial;
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
impl core::ops::Deref for CanonicalAssetPage {
    type Target = crate::BusinessAsset;
    fn deref(&self) -> &Self::Target {
        &self.financial
    }
}
impl core::ops::DerefMut for CanonicalAssetPage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.financial
    }
}

/// One original schedule year, with amounts actually allowed or allowable
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AssetYearHistory {
    /// Tax year on this schedule
    pub year: u32,
    /// Business plus investment use
    pub income_use: UseFraction,
    /// Qualified business use under section 280F
    pub qualified_use: UseFraction,
    /// Regular-tax section 179 actually allowed this year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub section179_allowed: i64,
    /// Bonus allowed or allowable this year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub bonus: i64,
    /// Other regular depreciation allowed or allowable
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub regular: i64,
    /// Other AMT depreciation allowed or allowable
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amt: i64,
    /// Documented increase other than current depreciation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis_increase: i64,
    /// Documented decrease other than current depreciation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis_decrease: i64,
    /// Deduction already recaptured in this year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub recaptured: i64,
}

/// Complete original schedule and annual history for the selected route
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HistoricalAssetRecords {
    /// Original income-producing basis after documented acquisition and credit adjustments
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub original_business_basis: i64,
    /// Original income-producing use
    pub original_income_use: UseFraction,
    /// Original qualified business use
    pub original_qualified_use: UseFraction,
    /// Original elected amount, including its documented unused balance
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub original_section179_elected: i64,
    /// Original allowed or allowable bonus
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub original_bonus: i64,
    /// One row per year needed for the current depreciation or recapture calculation
    pub years: Vec<AssetYearHistory>,
    /// Unused income-limit section 179 by originating year
    pub carryovers: Vec<Asset179Carryover>,
    /// Original legal convention
    pub convention: AssetConvention,
    /// Original method and election
    pub original_method: OriginalAssetMethod,
    /// Passenger-auto bonus and safe-harbor adoption facts
    /// [NullText]
    /// No passenger-auto bonus or safe-harbor recovery history applies
    /// [SetText]
    /// The original passenger-auto recovery records are
    #[libertas_default($null)]
    pub auto_history: Option<AutoRecoveryHistory>,
}

/// Original convention from the lawful schedule
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum AssetConvention {
    /// Half-year
    HalfYear,
    /// Mid-quarter
    MidQuarter,
    /// Mid-month
    MidMonth,
    /// Actual-month non-MACRS method
    ActualMonths,
}

/// Original recovery method; table selection stays in the shared engine
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum OriginalAssetMethod {
    /// 200 percent declining balance with required switch
    Declining200,
    /// 150 percent declining balance with required switch
    Declining150,
    /// Straight line
    StraightLine,
    /// Rev. Proc. 2019-13 adopted in first succeeding year
    SafeHarbor2019,
    /// The typed historical route supplies its original method
    OtherDocumented,
}

/// Passenger-auto recovery records; annual caps derive from actual service year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AutoRecoveryHistory {
    /// Original allowance/election
    pub first_year_bonus: OriginalBonusTreatment,
    /// Year adopted by use on return, if applicable
    /// [NullText]
    /// No Rev. Proc. 2019-13 safe harbor was adopted
    /// [SetText]
    /// The safe harbor was adopted on the return for
    #[libertas_default($null)]
    pub safe_harbor_adopted_year: Option<u32>,
    /// Original cap-table classification
    pub car_or_truck: HistoricalAutoKind,
    /// Original combined deduction after the passenger-auto cap
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub first_year_capped_deduction: i64,
    /// Remaining basis entering the first year after recovery; reconciled to prior history
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub post_recovery_basis: i64,
}

/// Original bonus treatment established by the schedule
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum OriginalBonusTreatment {
    /// 100 percent allowance was legally allowable
    HundredPercent,
    /// Applicable historical allowance below 100 percent
    Partial,
    /// Valid original class election out
    ElectedOut,
    /// Property did not qualify
    Ineligible,
}

/// Category used in the original annual limitation table
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HistoricalAutoKind {
    /// Passenger car under the applicable unloaded-weight test
    Car,
    /// Truck or van under the applicable GVWR test
    TruckVan,
    /// Passenger car produced primarily to run on electricity and first placed in service after August 5, 1997, and before 2007
    /// The original electric-car table applies throughout its recovery and to later unused basis and section 179 releases. Cars placed after 2006 use the ordinary car or truck classification.
    ElectricCar,
}

crate::community::divide_record!(HistoricalAssetRecords {
    amounts: [
        original_business_basis,
        original_section179_elected,
        original_bonus,
        years,
        carryovers,
        auto_history
    ],
    gains: [],
    facts: [original_income_use, original_qualified_use, convention, original_method],
});
crate::community::divide_record!(AssetYearHistory {
    amounts: [section179_allowed, bonus, regular, amt, basis_increase, basis_decrease, recaptured],
    gains: [],
    facts: [year, income_use, qualified_use],
});
crate::community::divide_record!(AutoRecoveryHistory {
    amounts: [first_year_capped_deduction, post_recovery_basis],
    gains: [],
    facts: [first_year_bonus, safe_harbor_adopted_year, car_or_truck],
});
