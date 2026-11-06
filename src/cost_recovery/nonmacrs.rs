// Actual non-MACRS property. Sources: IRC167(f),(g),168(f)(1); Reg1.167(a)-14;
// Pub946 and Pub534. Source eligibility stays here; arithmetic uses cost_recovery.
pub use crate::asset_model::{
    AcrsOriginalClass, AcrsPeriod, FiniteLifeKind, FiniteLifeRecovery, ForecastLookback,
    ForecastParticipations, ForecastPropertyKind, GdsEquipmentClass, HistoricalDepreciation,
    HistoricalPropertyKind, HistoricalRecoveryKind, IncomeForecastRecovery, NonMacrsRecovery,
    SoftwareKind, SoftwareRecovery, SoftwareTaxExemptLease, UnitProduction,
};
use crate::{AssetConvention, EquipmentClass};
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub(crate) use crate::asset_model::FACTS;
use crate::{BusinessAsset, EquipmentHistory, kernel::ratio};

impl BusinessAsset {
    pub(crate) fn nonmacrs_issue(&self) -> Option<&'static str> {
        if !self.nonmacrs() {
            return None;
        }
        if self.disposal_date().is_some()
            && !matches!(
                self.disposal,
                crate::AssetDisposal::Disposed { how: crate::DisposalKind::PersonalUse, .. }
            )
            && matches!(
                self.nonmacrs_route(),
                Some(
                    NonMacrsRecovery::IncomeForecast { .. }
                        | NonMacrsRecovery::FiniteLife {
                            facts: FiniteLifeRecovery::CreatedEligible
                                | FiniteLifeRecovery::CreatedRealPropertyBenefits
                        }
                )
            )
        {
            return Some("ASSET_NONMACRS_DISPOSITION_HISTORY");
        }
        if let EquipmentHistory::Continuing { carryforward, .. } = self.history
            && !matches!(self.amt_section179_carry(), Ok(actual) if actual == carryforward)
        {
            // This source has one non-MACRS recovery history. An unreported
            // or different independently limited AMT election needs its actual
            // source projection; the regular carry cannot stand in for it.
            return Some("ASSET_NONMACRS_AMT_HISTORY");
        }
        // IRC57(a)(1), i6251 line3: accelerated real property and leased
        // personal-property pre-1987 preferences require their actual separate
        // straight-line/AMT source. This owner never substitutes regular tax.
        if matches!(self.nonmacrs_route(), Some(NonMacrsRecovery::Historical { facts })
            if matches!(facts.property, HistoricalPropertyKind::RealProperty | HistoricalPropertyKind::PersonalLeased)
                && matches!(facts.kind, HistoricalRecoveryKind::Pre1981Declining { .. } | HistoricalRecoveryKind::Acrs { alternate: false, .. }))
        {
            return Some("ASSET_NONMACRS_AMT_HISTORY");
        }
        match self.nonmacrs_recovery(self.bonus_amount()) {
            Ok(recovery) => recovery.unavailable,
            Err(issue) => Some(issue),
        }
    }
}

/// Original recovery facts for ordinary equipment or actual non-MACRS property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AssetRecoveryPage {
    /// MACRS property
    Gds {
        /// Original GDS class and history
        facts: GdsAssetRecovery,
    },
    /// Units, original historical schedule, software, finite-life right or forecast
    NonMacrs {
        /// Original actual recovery source
        #[libertas_calculation_request("NonMacrsAssetCarryOptions")]
        facts: NonMacrsPropertyRecovery,
    },
}
/// Ordinary MACRS classification and original elections
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("AssetMacrsSourceOptions,AssetMacrsClassLifeOptions")]
pub struct GdsAssetRecovery {
    /// Server-projected owner-dependent question cohorts; absent on a newly constructed row
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub macrs_question_context: Option<alloc::boxed::Box<crate::MacrsSourceQuestionContext>>,
    /// Actual recovery source records
    /// [NullText]
    /// Source records not entered
    /// [SetText]
    /// Recovery source records
    #[libertas_calculation_request("AssetMacrsSourceOptions,AssetMacrsClassLifeOptions")]
    pub source: Option<crate::MacrsRecoverySourcePage>,
    /// Kind of property
    #[libertas_calculation_request("AssetMacrsClassLifeOptions")]
    pub class: crate::AssetClass,
    /// Depreciation history
    #[libertas_default(New)]
    #[libertas_calculation_request("AssetAmtCarryOptions,AssetMacrsSourceOptions")]
    pub history: crate::AssetHistory,
}
#[cfg(test)]
impl crate::Asset {
    pub(crate) fn gds(&self) -> Option<&GdsAssetRecovery> {
        match &self.recovery {
            AssetRecoveryPage::Gds { facts } => Some(facts),
            _ => None,
        }
    }
    pub(crate) fn gds_mut(&mut self) -> Option<&mut GdsAssetRecovery> {
        match &mut self.recovery {
            AssetRecoveryPage::Gds { facts } => Some(facts),
            _ => None,
        }
    }
}

/// Actual non-MACRS source. Only software asks179 and bonus elections.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum NonMacrsPropertyRecovery {
    /// Original elected units-of-production property
    Units {
        /// Description from the original schedule
        description: alloc::string::String,
        /// Original election and output records
        facts: UnitProduction,
        /// Actual original depreciation history
        history: NonMacrsBasicHistory,
    },
    /// Actual ACRS or other historical property
    Historical {
        /// Description from the original schedule
        description: alloc::string::String,
        /// Original system, property and convention
        facts: HistoricalDepreciation,
        /// Actual original depreciation history
        history: NonMacrsBasicHistory,
    },
    /// Purchased software outside197
    Software {
        /// Description from the original schedule
        description: alloc::string::String,
        /// Actual source eligibility and lease facts
        facts: SoftwareRecovery,
        /// Original software deductions and history
        history: NonMacrsHistory,
    },
    /// Actual finite-life right outside197
    FiniteLife {
        /// Description from the original schedule
        description: alloc::string::String,
        /// Actual right and its remaining life
        facts: FiniteLifeRecovery,
        /// Actual original depreciation history
        history: NonMacrsBasicHistory,
    },
    /// Actual income-forecast property
    IncomeForecast {
        /// Description from the original schedule
        description: alloc::string::String,
        /// Actual forecast and lookback sources
        facts: IncomeForecastRecovery,
        /// Actual original depreciation history
        history: NonMacrsBasicHistory,
    },
}
/// Original history for property ineligible for179 and bonus
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum NonMacrsBasicHistory {
    /// First placed in service in2026
    New,
    /// Original schedule began before2026
    Continuing {
        /// Actual prior depreciation allowed or allowable
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        prior_depreciation: i64,
    },
}
impl NonMacrsBasicHistory {
    fn native(&self) -> crate::EquipmentHistory {
        match *self {
            Self::New => crate::EquipmentHistory::New {
                section179: crate::Section179::Ineligible,
                bonus: crate::BonusElection::ElectOut,
                tables: false,
            },
            Self::Continuing { prior_depreciation } => crate::EquipmentHistory::Continuing {
                prior_depreciation,
                initial_section179: 0,
                carryforward: 0,
                amt: crate::EquipmentAmtHistory::Same,
                mid_quarter: false,
                schedule: crate::EquipmentSchedule::Tables { initial_bonus: 0 },
            },
        }
    }
}
impl NonMacrsPropertyRecovery {
    pub(crate) fn from_source(
        description: alloc::string::String,
        system: NonMacrsRecovery,
        history: crate::EquipmentHistory,
    ) -> Self {
        let basic = match history {
            crate::EquipmentHistory::New { .. } => NonMacrsBasicHistory::New,
            crate::EquipmentHistory::Continuing { prior_depreciation, .. } => {
                NonMacrsBasicHistory::Continuing { prior_depreciation }
            }
        };
        match system {
            NonMacrsRecovery::Units { facts } => Self::Units { description, facts, history: basic },
            NonMacrsRecovery::Historical { facts } => {
                Self::Historical { description, facts, history: basic }
            }
            NonMacrsRecovery::Software { facts } => Self::Software {
                description,
                facts,
                history: NonMacrsHistory::from_native(history),
            },
            NonMacrsRecovery::FiniteLife { facts } => {
                Self::FiniteLife { description, facts, history: basic }
            }
            NonMacrsRecovery::IncomeForecast { facts } => {
                Self::IncomeForecast { description, facts, history: basic }
            }
        }
    }
    fn system(&self) -> NonMacrsRecovery {
        match *self {
            Self::Units { facts, .. } => NonMacrsRecovery::Units { facts },
            Self::Historical { facts, .. } => NonMacrsRecovery::Historical { facts },
            Self::Software { facts, .. } => NonMacrsRecovery::Software { facts },
            Self::FiniteLife { facts, .. } => NonMacrsRecovery::FiniteLife { facts },
            Self::IncomeForecast { facts, .. } => NonMacrsRecovery::IncomeForecast { facts },
        }
    }
    fn native_history(&self) -> crate::EquipmentHistory {
        match self {
            Self::Software { history, .. } => history.native(),
            Self::Units { history, .. }
            | Self::Historical { history, .. }
            | Self::FiniteLife { history, .. }
            | Self::IncomeForecast { history, .. } => history.native(),
        }
    }
    fn description(&self) -> &alloc::string::String {
        match self {
            Self::Software { description, .. }
            | Self::Units { description, .. }
            | Self::Historical { description, .. }
            | Self::FiniteLife { description, .. }
            | Self::IncomeForecast { description, .. } => description,
        }
    }
}

/// Actual recovery history, without MACRS class, table or convention questions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum NonMacrsHistory {
    /// First income-producing service in 2026
    New {
        /// Current section179 classification and elected cost (software only)
        section179: crate::Section179,
        /// Bonus election (qualifying software only)
        bonus: crate::BonusElection,
    },
    /// Continue the original dated schedule
    Continuing {
        /// Original elected section179 cost, including any unused carry
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        initial_section179: i64,
        /// Actual unused regular section179 income-limit carry
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        carryforward: i64,
        /// Actual prior depreciation including original allowed section179 and bonus
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        prior_depreciation: i64,
        /// Original bonus allowed or allowable
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        original_bonus: i64,
    },
}
impl NonMacrsHistory {
    pub(crate) fn native(&self) -> crate::EquipmentHistory {
        match *self {
            Self::New { section179, bonus } => {
                crate::EquipmentHistory::New { section179, bonus, tables: false }
            }
            Self::Continuing {
                initial_section179,
                carryforward,
                prior_depreciation,
                original_bonus,
            } => crate::EquipmentHistory::Continuing {
                initial_section179,
                carryforward,
                prior_depreciation,
                amt: crate::EquipmentAmtHistory::Same,
                mid_quarter: false,
                schedule: crate::EquipmentSchedule::Tables { initial_bonus: original_bonus },
            },
        }
    }
    pub(crate) fn from_native(value: crate::EquipmentHistory) -> Self {
        match value {
            crate::EquipmentHistory::New { section179, bonus, .. } => {
                Self::New { section179, bonus }
            }
            crate::EquipmentHistory::Continuing {
                initial_section179,
                carryforward,
                prior_depreciation,
                schedule,
                ..
            } => Self::Continuing {
                initial_section179,
                carryforward,
                prior_depreciation,
                original_bonus: match schedule {
                    crate::EquipmentSchedule::Tables { initial_bonus } => initial_bonus,
                    _ => 0,
                },
            },
        }
    }
}

impl BusinessAsset {
    pub(crate) fn nonmacrs_qbi_property(&self) -> bool {
        // IRC199A(b)(6): qualified property must be tangible. Software and
        // finite-life/forecast intangibles never become UBIA merely through179.
        if self.disposal != crate::AssetDisposal::No {
            return false;
        }
        let class = match self.nonmacrs_route() {
            Some(NonMacrsRecovery::Units { facts }) => facts.qbi_class,
            Some(NonMacrsRecovery::Historical { facts }) => facts.qbi_class,
            _ => return false,
        };
        let period = match class {
            GdsEquipmentClass::Three => 3,
            GdsEquipmentClass::Five => 5,
            GdsEquipmentClass::Seven => 7,
            GdsEquipmentClass::Ten => 10,
            GdsEquipmentClass::Fifteen => 15,
            GdsEquipmentClass::Twenty => 20,
            GdsEquipmentClass::ThirtyNine => 39,
        };
        let year = u64::from(self.placed / 10000);
        let ten_years = (year + 10) * 10000 + u64::from(self.placed % 10000);
        let recovery = (year + period - 1) * 10000 + 1231;
        ten_years.max(recovery) >= 20261231
    }
    pub(crate) fn nonmacrs_straight_line_through(&self, _base: i64, _sold: u32) -> i64 {
        // The admitted real-property route is original alternate straight-line
        // ACRS: there is no additional depreciation. Accelerated real histories
        // remain unavailable until their actual straight-line owner is present.
        match self.history {
            EquipmentHistory::Continuing { prior_depreciation, .. } => {
                prior_depreciation + self.deduction(false)
            }
            _ => self.deduction(false),
        }
    }
}

impl NonMacrsPropertyRecovery {
    pub(crate) fn record(&self, page: &crate::Asset) -> BusinessAsset {
        BusinessAsset {
            id: page.id,
            macrs_source: page.macrs_source.clone(),
            farm_rental_use: page.farm_rental_use.clone(),
            history_details: page.history_details.clone(),
            amt_section179_income_carry: self.supplied_amt_carry(&page.amt_section179_income_carry),
            description: self.description().clone(),
            cost: page.cost,
            credit_reduction: page.credit_reduction,
            acquired: page.acquired.unwrap_or(page.placed),
            placed: page.placed,
            class: crate::EquipmentClass::NonMacrs { recovery: self.system() },
            history: self.native_history(),
            // Unused legacy GDS method slot; all actual consumers dispatch on class.
            method: crate::EquipmentMethod::StraightLine,
            listed: crate::ListedProperty::No { business_percent: page.business_percent },
            disposal: page.disposal.clone(),
            investment_credit: page.investment_credit.clone(),
            earlier_business_percent: page.earlier_business_percent,
            partial: page.partial.clone(),
        }
    }
}
pub(crate) fn asset_page(record: &BusinessAsset, system: NonMacrsRecovery) -> crate::Asset {
    crate::Asset {
        macrs_source: record.macrs_source.clone(),
        id: record.id,
        farm_rental_use: record.farm_rental_use.clone(),
        allowed_farm_rental_use: alloc::vec::Vec::new(),
        history_details: record.history_details.clone(),
        amt_section179_income_carry: if record.amt_carry_choices().is_empty() {
            None
        } else {
            crate::AssetAmt179CarryAnswer::from_amount(record.amt_section179_income_carry)
        },
        allowed_amt_carry: record.amt_carry_choices(),
        cost: record.cost,
        credit_reduction: record.credit_reduction,
        placed: record.placed,
        recovery: AssetRecoveryPage::NonMacrs {
            facts: NonMacrsPropertyRecovery::from_source(
                record.description.clone(),
                system,
                record.history,
            ),
        },
        business_percent: match record.listed {
            crate::ListedProperty::No { business_percent } => business_percent,
            _ => unreachable!("non-MACRS source is not a vehicle"),
        },
        acquired: (record.acquired != record.placed).then_some(record.acquired),
        disposal: record.disposal.clone(),
        investment_credit: record.investment_credit.clone(),
        earlier_business_percent: record.earlier_business_percent,
        partial: record.partial.clone(),
    }
}

/// Actual non-MACRS property; entered once on the normal equipment page
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("NonMacrsCanonicalCarryOptions")]
pub struct NonMacrsAssetPage {
    /// Property use throughout your ownership
    /// Use the lease and property-use records for this sold property.
    /// [NullText]
    /// Property-use records are unavailable
    /// [SetText]
    /// The property-use records show
    #[libertas_hidden]
    #[libertas_read_only]
    pub farm_rental_use: Option<crate::FarmRentalPropertyUse>,

    /// Supplied MACRS-only records remain subject to their native source-shape refusal
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub macrs_source: Option<crate::MacrsRecoverySource>,
    /// Canonical property identity assigned on Save
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Original property cost, including capitalized acquisition costs
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub cost: i64,
    /// Actual credit basis reduction, not the credit amount unless fully basis-reducing
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub credit_reduction: i64,
    /// Original acquisition date
    #[libertas_date_only]
    pub acquired: u32,
    /// Original date ready and available for income-producing use
    #[libertas_date_only]
    pub placed: u32,
    /// Current income-producing use
    // A percentage's denominator is100 by definition.
    #[libertas_number(min = 1, max = 100)]
    #[libertas_physical_unit("percent")]
    pub business_percent: u32,
    /// Actual original schedule and recovery system
    #[libertas_calculation_request("NonMacrsCanonicalCarryOptions")]
    pub recovery: NonMacrsPropertyRecovery,
    /// Additional original annual records
    /// [NullText]
    /// No additional annual recovery records are supplied
    /// [SetText]
    /// The original annual recovery records are
    pub history_details: Option<crate::HistoricalAssetRecords>,
    /// Actual separately documented AMT unused section179 carry
    /// [NullText]
    /// No separate AMT unused section179 carry is supplied
    /// [SetText]
    /// The actual AMT unused section179 carry is
    #[libertas_constrained_by("^<NonMacrsAssetPage>.allowed_amt_carry")]
    pub amt_section179_income_carry: Option<crate::AssetAmt179CarryAnswer>,
    /// Applicable actual AMT carry source answer
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_asset_amt_carry_choices")]
    pub allowed_amt_carry: alloc::vec::Vec<i32>,
    /// Actual current-year disposition
    #[libertas_default(No)]
    pub disposal: crate::AssetDisposal,
}
impl From<BusinessAsset> for NonMacrsAssetPage {
    fn from(value: BusinessAsset) -> Self {
        let system = *value.nonmacrs_route().expect("actual non-MACRS source");
        let allowed_amt_carry = value.amt_carry_choices();
        Self {
            macrs_source: value.macrs_source,
            id: value.id,
            cost: value.cost,
            credit_reduction: value.credit_reduction,
            acquired: value.acquired,
            placed: value.placed,
            business_percent: match value.listed {
                crate::ListedProperty::No { business_percent } => business_percent,
                _ => unreachable!("non-MACRS source is not a vehicle"),
            },
            recovery: NonMacrsPropertyRecovery::from_source(
                value.description,
                system,
                value.history,
            ),
            farm_rental_use: value.farm_rental_use.clone(),
            history_details: value.history_details,
            amt_section179_income_carry: if allowed_amt_carry.is_empty() {
                None
            } else {
                crate::AssetAmt179CarryAnswer::from_amount(value.amt_section179_income_carry)
            },
            allowed_amt_carry,
            disposal: value.disposal,
        }
    }
}
impl From<NonMacrsAssetPage> for BusinessAsset {
    fn from(value: NonMacrsAssetPage) -> Self {
        Self {
            id: value.id,
            macrs_source: value.macrs_source,
            cost: value.cost,
            credit_reduction: value.credit_reduction,
            acquired: value.acquired,
            placed: value.placed,
            listed: crate::ListedProperty::No { business_percent: value.business_percent },
            class: crate::EquipmentClass::NonMacrs { recovery: value.recovery.system() },
            description: value.recovery.description().clone(),
            history: value.recovery.native_history(),
            farm_rental_use: value.farm_rental_use.clone(),
            history_details: value.history_details,
            amt_section179_income_carry: value
                .recovery
                .supplied_amt_carry(&value.amt_section179_income_carry),
            disposal: value.disposal,
            method: crate::EquipmentMethod::StraightLine,
            investment_credit: crate::InvestmentCredit::No,
            earlier_business_percent: None,
            partial: None,
        }
    }
}

// Actual Form4562 class rows, or the single annual expense category when no
// Form4562 is required, own the rounding boundary. Original published fractional
// ACRS allowances must not be rounded to cents before this boundary.
pub(crate) struct AssetDepreciationAmounts {
    native_cents: ratio::exact::Exact,
    pub(crate) classes: alloc::collections::BTreeMap<&'static str, ratio::exact::Exact>,
    pub(crate) actual_report: bool,
    reported_form: bool,
}
impl AssetDepreciationAmounts {
    pub(crate) fn expense_cents(&self, allowed179: i64) -> Option<i64> {
        if !self.actual_report {
            return ratio::exact::rounded(&(&self.native_cents + ratio::exact::whole(allowed179)));
        }
        if !self.reported_form {
            let annual = self
                .classes
                .values()
                .fold(ratio::exact::whole(allowed179), |sum, value| sum + value);
            return ratio::exact::dollars(&annual).and_then(|amount| amount.checked_mul(100));
        }
        let total = self.classes.values().try_fold(0i64, |sum, value| {
            let amount = ratio::exact::dollars(value)?.checked_mul(100)?;
            sum.checked_add(amount)
        })?;
        total.checked_add(ratio::dollars(allowed179).checked_mul(100)?)
    }
    pub(crate) fn adjustment(&self, allowed179: i64) -> Option<i64> {
        self.expense_cents(allowed179)?
            .checked_sub(ratio::exact::rounded(&self.native_cents)?)?
            .checked_sub(allowed179)
    }
}
pub(crate) fn asset_depreciation_amounts(
    assets: &[BusinessAsset],
    mid_quarter: bool,
    amt: bool,
) -> AssetDepreciationAmounts {
    let mut amounts = AssetDepreciationAmounts {
        native_cents: ratio::exact::whole(0),
        classes: alloc::collections::BTreeMap::new(),
        actual_report: assets.iter().any(BusinessAsset::nonmacrs),
        reported_form: crate::depreciation::assets_need_form_4562(assets),
    };
    for asset in assets {
        let bonus = asset.bonus_amount();
        let regular = if amt {
            asset.deduction_for(mid_quarter, true) - bonus
        } else {
            asset.macrs_amount(mid_quarter)
        };
        amounts.native_cents += ratio::exact::whole(bonus) + ratio::exact::whole(regular);
        let actual = if asset.nonmacrs() {
            asset.nonmacrs_recovery(bonus).ok().and_then(|recovery| recovery.exact_amount)
        } else {
            None
        };
        for (suffix, amount) in [
            ("bonus", ratio::exact::whole(bonus)),
            ("macrs", actual.unwrap_or_else(|| ratio::exact::whole(regular))),
        ] {
            let class = crate::depreciation::form_4562_line(asset, suffix).unwrap_or("21");
            *amounts.classes.entry(class).or_insert_with(|| ratio::exact::whole(0)) += amount;
        }
    }
    amounts
}

// Borrowed class elections also apply to equipment in another activity or job.
// Source records and monetary operands retain their accepted owners.
pub(crate) fn asset_depreciation_amounts_for_owner(
    draft: &crate::FederalDraft,
    owner: crate::Owner,
    assets: &[BusinessAsset],
    mid_quarter: bool,
    amt: bool,
    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> AssetDepreciationAmounts {
    let fallback = macrs.is_none().then(|| crate::macrs_source::MacrsElectionView::new(draft));
    asset_depreciation_amounts_for_view(
        owner,
        None,
        assets,
        mid_quarter,
        amt,
        macrs.or(fallback.as_ref()).unwrap(),
    )
}
pub(crate) fn asset_depreciation_amounts_for_view(
    owner: crate::Owner,
    activity: Option<i64>,
    assets: &[BusinessAsset],
    mid_quarter: bool,
    amt: bool,
    macrs: &crate::macrs_source::MacrsElectionView,
) -> AssetDepreciationAmounts {
    let mut amounts = asset_depreciation_amounts(assets, mid_quarter, amt);

    for asset in assets.iter().filter(|asset| !asset.nonmacrs()) {
        let invocation = macrs.context(owner, activity, asset);
        if invocation.election.is_none()
            && invocation.required.is_none()
            && !invocation.unavailable
            && asset.macrs_source.as_ref().is_none_or(|s| s.improvement.is_none())
        {
            continue;
        }
        let selected = if amt { asset.amt_projection_or_self() } else { asset.clone() };
        let native_bonus = selected.bonus_amount();
        let native_macrs = selected.macrs_amount(mid_quarter);
        let (bonus, macrs) = match selected.source_ads_recovery_in_context(invocation) {
            Ok(Some(_)) if selected.macrs_source_issue_in_context(invocation).is_none() => (
                selected.source_bonus_amount_in_context(invocation),
                selected.source_macrs_amount_in_context(mid_quarter, invocation).unwrap_or(0),
            ),
            Ok(None) => (native_bonus, native_macrs),
            _ => (0, 0),
        };
        amounts.native_cents += ratio::exact::whole(bonus) + ratio::exact::whole(macrs)
            - ratio::exact::whole(native_bonus)
            - ratio::exact::whole(native_macrs);
        for (suffix, native, current) in
            [("bonus", native_bonus, bonus), ("macrs", native_macrs, macrs)]
        {
            let original_class = crate::depreciation::form_4562_line(asset, suffix).unwrap_or("21");
            *amounts.classes.entry(original_class).or_insert_with(|| ratio::exact::whole(0)) -=
                ratio::exact::whole(native);
            let class = crate::depreciation::form_4562_line_in_context(asset, suffix, invocation)
                .unwrap_or("21");
            *amounts.classes.entry(class).or_insert_with(|| ratio::exact::whole(0)) +=
                ratio::exact::whole(current);
        }
    }
    amounts
}

// Contextual MACRS operands retain the purchase-refund owner's deductions.
// Non-MACRS sources keep the same annual Form 4562 class boundary.
pub(crate) fn asset_depreciation_amounts_for(
    draft: &crate::FederalDraft,
    id: i64,
    assets: &[BusinessAsset],
    mid_quarter: bool,
    amt: bool,
    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> Result<AssetDepreciationAmounts, crate::Section179PreparedIssue> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    let mut amounts = asset_depreciation_amounts(assets, mid_quarter, amt);
    let fallback = macrs.is_none().then(|| crate::macrs_source::MacrsElectionView::new(draft));
    let source_view = macrs.or(fallback.as_ref()).unwrap();
    let owner = draft.income.iter().find(|entry| entry.id == id).map(|entry| entry.owner);
    for asset in assets.iter().filter(|asset| !asset.nonmacrs()) {
        let bonus = asset.bonus_amount();
        let regular = if amt {
            asset.deduction_for(mid_quarter, true) - bonus
        } else {
            asset.macrs_amount(mid_quarter)
        };
        let owned = crate::cooperative_basis::source_amounts_with_claims(
            draft,
            id,
            asset,
            mid_quarter,
            amt,
            &source_claims,
            macrs,
        )?;
        for (suffix, original, current) in
            [("bonus", bonus, owned.bonus), ("macrs", regular, owned.macrs)]
        {
            let difference = ratio::exact::whole(current) - ratio::exact::whole(original);
            amounts.native_cents += &difference;
            let original_class = crate::depreciation::form_4562_line(asset, suffix).unwrap_or("21");
            let context = owner.map_or_else(crate::macrs_source::AdsInvocation::default, |owner| {
                source_view.context(owner.owned(), Some(id), asset)
            });
            let class = crate::depreciation::form_4562_line_in_context(asset, suffix, context)
                .unwrap_or("21");
            *amounts.classes.entry(original_class).or_insert_with(|| ratio::exact::whole(0)) -=
                ratio::exact::whole(original);
            *amounts.classes.entry(class).or_insert_with(|| ratio::exact::whole(0)) +=
                ratio::exact::whole(current);
        }
    }
    Ok(amounts)
}

// Rental allocation changes monetary facts, never the physical recovery schedule.
// Keep this transient source beside the existing community Share owner.
pub(crate) struct RentalRecoverySource<'a> {
    assets: &'a [BusinessAsset],
    share: Option<crate::community::Share>,
    owner: crate::Owner,
    activity: i64,
    macrs: crate::macrs_source::MacrsElectionView,
}
pub(crate) fn rental_recovery_source(
    draft: &crate::FederalDraft,
    id: i64,
    property: bool,
) -> Option<RentalRecoverySource<'_>> {
    rental_recovery_source_with_macrs(draft, id, property, None)
}
pub(crate) fn rental_recovery_source_with_macrs<'a>(
    draft: &'a crate::FederalDraft,
    id: i64,
    property: bool,
    macrs: Option<&crate::macrs_source::MacrsElectionView>,
) -> Option<RentalRecoverySource<'a>> {
    let entry = draft.income.iter().find(|entry| entry.id == id)?;
    let assets = match &entry.income {
        crate::FederalIncome::Rental { data } => &data.assets,
        crate::FederalIncome::FarmRental { data } => &data.assets,
        _ => return None,
    };
    let macrs =
        macrs.cloned().unwrap_or_else(|| crate::macrs_source::MacrsElectionView::new(draft));
    if !assets
        .iter()
        .any(|asset| asset.nonmacrs() || macrs.affects(entry.owner.owned(), Some(id), asset))
    {
        return None;
    }
    let share = draft
        .setup
        .as_ref()
        .and_then(|setup| {
            crate::community::share(setup, entry.owner.owned(), entry.community, &entry.income)
        })
        .map(|share| if property { share.for_gains() } else { share })
        .filter(|share| !share.whole());
    Some(RentalRecoverySource { assets, share, owner: entry.owner.owned(), activity: id, macrs })
}
impl RentalRecoverySource<'_> {
    pub(crate) fn context<'a>(
        &'a self,
        asset: &BusinessAsset,
    ) -> crate::macrs_source::AdsInvocation<'a> {
        self.macrs.context(self.owner, Some(self.activity), asset)
    }
    pub(crate) fn history_issue(&self, asset: &BusinessAsset) -> Option<&'static str> {
        asset.macrs_history_issue_in_context(self.context(asset))
    }
    fn amounts(
        &self,
        asset: &BusinessAsset,
        mid_quarter: bool,
        amt: bool,
    ) -> AssetDepreciationAmounts {
        asset_depreciation_amounts_for_view(
            self.owner,
            Some(self.activity),
            core::slice::from_ref(asset),
            mid_quarter,
            amt,
            &self.macrs,
        )
    }
    pub(crate) fn asset<'a>(
        &'a self,
        index: usize,
        projected: &'a BusinessAsset,
    ) -> &'a BusinessAsset {
        if projected.nonmacrs() { &self.assets[index] } else { projected }
    }
    pub(crate) fn project(&self, amount: &ratio::exact::Exact) -> ratio::exact::Exact {
        self.share.as_ref().map_or_else(
            || amount.clone(),
            |share| {
                if share.income_fraction_is_whole() {
                    if share.filer_owns() { amount.clone() } else { ratio::exact::whole(0) }
                } else {
                    share.filer_exact(amount)
                }
            },
        )
    }
}
pub(crate) fn rental_asset_parts(
    projected: &BusinessAsset,
    index: usize,
    source: Option<&RentalRecoverySource<'_>>,
    mid_quarter: bool,
) -> Option<[i64; 2]> {
    let actual = source.map_or(projected, |source| source.asset(index, projected));
    let bonus = actual.bonus_amount();
    if !actual.nonmacrs() {
        if let Some(source) = source {
            let amounts = source.amounts(actual, mid_quarter, false);
            let bonus =
                amounts.classes.get("14a").cloned().unwrap_or_else(|| ratio::exact::whole(0));
            let regular = amounts.native_cents - &bonus;
            return Some([ratio::exact::rounded(&bonus)?, ratio::exact::rounded(&regular)?]);
        }
        return Some([bonus, actual.macrs_amount(mid_quarter)]);
    }
    let regular = actual
        .nonmacrs_recovery(bonus)
        .ok()?
        .exact_amount
        .unwrap_or_else(|| ratio::exact::whole(actual.macrs_amount(mid_quarter)));
    let [bonus, regular] = [ratio::exact::whole(bonus), regular].map(|amount| {
        ratio::exact::rounded(
            &source.map_or_else(|| amount.clone(), |source| source.project(&amount)),
        )
    });
    Some([bonus?, regular?])
}
pub(crate) fn rental_asset_amounts(
    assets: &[BusinessAsset],
    source: Option<&RentalRecoverySource<'_>>,
    mid_quarter: bool,
    amt: bool,
) -> Result<alloc::collections::BTreeMap<&'static str, ratio::exact::Exact>, &'static str> {
    let mut classes = alloc::collections::BTreeMap::new();
    for (index, projected) in assets.iter().enumerate() {
        let actual = source.map_or(projected, |source| source.asset(index, projected));
        if actual.nonmacrs() && !actual.valid() {
            return Err("ASSET_NONMACRS_ORIGINAL_HISTORY");
        }
        if let Some(issue) = actual.nonmacrs_issue() {
            return Err(issue);
        }
        let amounts = source.map_or_else(
            || asset_depreciation_amounts(core::slice::from_ref(actual), mid_quarter, amt),
            |source| source.amounts(actual, mid_quarter, amt),
        );
        for (class, amount) in amounts.classes {
            let amount = if actual.nonmacrs() {
                source.map_or_else(|| amount.clone(), |source| source.project(&amount))
            } else {
                amount
            };
            *classes.entry(class).or_insert_with(|| ratio::exact::whole(0)) += amount;
        }
    }
    Ok(classes)
}

/// Actual ordinary MACRS property, with only lawful GDS class choices
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("PropertyMacrsSourceOptions,PropertyMacrsClassLifeOptions")]
pub struct GdsPropertyPage {
    /// Property use throughout your ownership
    /// Use the lease and property-use records for this sold property.
    /// [NullText]
    /// Property-use records are unavailable
    /// [SetText]
    /// The property-use records show
    #[libertas_hidden]
    #[libertas_read_only]
    pub farm_rental_use: Option<crate::FarmRentalPropertyUse>,

    /// Server-projected owner-dependent question cohorts; absent on a newly constructed row
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub macrs_question_context: Option<alloc::boxed::Box<crate::MacrsSourceQuestionContext>>,
    /// Actual recovery source records
    /// [NullText]
    /// Source records not entered
    /// [SetText]
    /// Recovery source records
    #[libertas_calculation_request("PropertyMacrsSourceOptions,PropertyMacrsClassLifeOptions")]
    pub source: Option<alloc::boxed::Box<crate::MacrsRecoverySourcePage>>,
    /// Canonical property identity assigned on Save
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Original schedules and annual use needed for historical recovery
    /// [NullText]
    /// No changed-use or original annual schedule is supplied
    /// [SetText]
    /// The original schedule and annual records are
    pub history_details: Option<crate::HistoricalAssetRecords>,
    /// Section 179 description (Form 4562 line 6)
    /// The description typed with a section 179 election; empty for every other asset, which a note names by its position.
    pub description: String,
    /// Original purchase cost, including capitalized acquisition and installation costs
    /// Do not also enter this amount in operating expenses, inventory, home-office costs or another asset record.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub cost: i64,
    /// Credit that reduces its cost
    /// Leave zero unless a tax credit was taken for this property. A clean-vehicle or fuel-cell vehicle credit (sections 30D(f)(1), 30B(h)(4) and 45W(d)(1)) and an investment credit (section 50(c)(1)) reduce its cost by the whole credit; an energy credit under section 48 reduces it by half the credit (section 50(c)(3)). Enter the reduction itself. It cannot be more than the cost.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub credit_reduction: i64,
    /// Acquisition date
    /// For a written binding purchase contract, use the acquisition date under that contract, not merely the delivery or payment date. Self-constructed property needs separate rules.
    #[libertas_date_only]
    pub acquired: u32,
    /// Original date ready and available for income-producing use
    #[libertas_date_only]
    pub placed: u32,
    /// Business use in earlier years, when it was more than 50%
    /// Only for an asset placed in service before 2026 whose business use this year is 50% or less. Enter its whole-number business use in the earlier years, the same in each. The App adds back the section 179 benefit (Form 4797 Part IV column (a)) or, for a vehicle, the depreciation allowed above straight-line (column (b)) as other income of the business, and raises the asset's basis by it. Leave empty if it was used 50% or less in every earlier year.
    /// [NullText]
    /// Business use did not fall from above 50% to 50% or less
    /// [SetText]
    /// Earlier business use above 50% was
    // Percentage encoding: business use cannot exceed 100 percent.
    #[libertas_number(min = 51, max = 100)]
    #[libertas_physical_unit("percent")]
    pub earlier_business_percent: Option<u32>,
    /// MACRS recovery class
    /// Use the IRS activity/asset classification, not an estimated useful life. Computers generally use 5 years and office furniture 7 years. Specialized equipment follows its assigned activity class. Nonresidential real property (a building or structural component used in this business) is 39-year straight-line with the mid-month convention; enter only the depreciable building cost, excluding land.
    #[libertas_calculation_request("PropertyMacrsClassLifeOptions")]
    pub class: GdsEquipmentClass,
    /// Depreciation history
    #[libertas_calculation_request("PropertyMacrsSourceOptions")]
    pub history: crate::EquipmentHistory,
    /// GDS method for basis remaining after section 179 and bonus depreciation
    /// For new assets, apply the same method to this recovery class across businesses and nonbusiness rentals. For continuing assets, enter the original GDS method from the depreciation record; the App applies the required switch to straight line. With 100% bonus there is no remaining basis after section 179. Ignored for nonresidential real property, which is always 39-year straight-line.
    pub method: crate::EquipmentMethod,
    // The pages that write this record ask only for qualifying purchased
    // equipment; the scope the calculation supports is checked by qualifies().
    /// Business or investment use
    /// For a car, van, pickup or panel truck whose actual costs you deduct, enter its exact business and total miles. Other property uses its documented percentage.
    pub listed: crate::ListedProperty,
    /// Sold or stopped using in 2026
    #[libertas_default(No)]
    pub disposal: crate::AssetDisposal,
    /// Investment credit (Form 3468)
    #[libertas_default(No)]
    pub investment_credit: crate::InvestmentCredit,
    /// Part of this building replaced or removed in 2026
    /// Only for a building (39-year): a roof, HVAC system or other structural component. Include partial sales and other mandatory dispositions. For replacement or removal, enter the component when electing separate partial-disposition treatment; otherwise choose no separately recognized disposition.
    /// [NullText]
    /// No building-component disposition is recognized separately in 2026
    /// [SetText]
    /// The separately recognized building-component disposition is
    pub partial: Option<crate::PartialDisposition>,
    /// Actual AMT unused elected cost; never inferred from the regular balance
    #[libertas_hidden]
    #[libertas_default($null)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt_section179_income_carry: Option<i64>,
}
impl From<GdsPropertyPage> for BusinessAsset {
    fn from(value: GdsPropertyPage) -> Self {
        Self {
            macrs_source: value.source.map(|source| (*source).into()),
            class: value.class.into(),
            id: value.id,
            farm_rental_use: value.farm_rental_use.clone(),
            history_details: value.history_details,
            description: value.description,
            cost: value.cost,
            credit_reduction: value.credit_reduction,
            acquired: value.acquired,
            placed: value.placed,
            earlier_business_percent: value.earlier_business_percent,
            history: value.history,
            method: value.method,
            listed: value.listed,
            disposal: value.disposal,
            investment_credit: value.investment_credit,
            partial: value.partial,
            amt_section179_income_carry: value.amt_section179_income_carry,
        }
    }
}
impl TryFrom<BusinessAsset> for GdsPropertyPage {
    type Error = ();
    fn try_from(value: BusinessAsset) -> Result<Self, Self::Error> {
        Ok(Self {
            macrs_question_context: None,
            source: value
                .macrs_source
                .clone()
                .map(|source| {
                    crate::MacrsRecoverySourcePage::from_source(
                        source,
                        matches!(value.history, EquipmentHistory::Continuing { .. }),
                        value.class == EquipmentClass::ThirtyNine,
                    )
                })
                .map(alloc::boxed::Box::new),
            class: value.class.try_into()?,
            id: value.id,
            farm_rental_use: value.farm_rental_use.clone(),
            history_details: value.history_details,
            description: value.description,
            cost: value.cost,
            credit_reduction: value.credit_reduction,
            acquired: value.acquired,
            placed: value.placed,
            earlier_business_percent: value.earlier_business_percent,
            history: value.history,
            method: value.method,
            listed: value.listed,
            disposal: value.disposal,
            investment_credit: value.investment_credit,
            partial: value.partial,
            amt_section179_income_carry: value.amt_section179_income_carry,
        })
    }
}
impl GdsPropertyPage {
    pub(crate) fn amt_carry_choices(&self) -> alloc::vec::Vec<i32> {
        BusinessAsset::from(self.clone()).amt_carry_choices()
    }
    #[cfg(test)]
    pub(crate) fn amt_section179_carry(&self) -> Result<i64, ()> {
        BusinessAsset::from(self.clone()).amt_section179_carry()
    }
}

impl BusinessAsset {
    pub(crate) fn nonmacrs_explanation(&self) -> crate::Message {
        let Ok(recovery) = self.nonmacrs_recovery(self.bonus_amount()) else {
            return crate::Message::new("EXPLAIN_BUSINESS_N_ASSET_N_N_NONMACRS", alloc::vec![]);
        };
        if recovery.quarter_months == 0 {
            return crate::Message::new("EXPLAIN_BUSINESS_N_ASSET_N_N_NONMACRS", alloc::vec![]);
        }
        let convention = match recovery.convention {
            AssetConvention::HalfYear => "RECOVERY_HALF_YEAR",
            AssetConvention::MidQuarter => "RECOVERY_MID_QUARTER",
            AssetConvention::MidMonth => "RECOVERY_MID_MONTH",
            AssetConvention::ActualMonths => "RECOVERY_ACTUAL_MONTHS",
        };
        crate::Message::new(
            "EXPLAIN_BUSINESS_N_ASSET_N_N_NONMACRS_DETAILS",
            alloc::vec![
                crate::MessageArg::Text(recovery.method.into()),
                crate::MessageArg::Text(alloc::format!(
                    "{}.{:02}",
                    recovery.quarter_months / 4,
                    (recovery.quarter_months % 4) * 25
                )),
                crate::MessageArg::Resource(convention.into()),
            ],
        )
    }
}

impl NonMacrsPropertyRecovery {
    fn supplied_amt_carry(&self, answer: &Option<crate::AssetAmt179CarryAnswer>) -> Option<i64> {
        if self.amt_carry_choices().is_empty() {
            None
        } else {
            crate::AssetAmt179CarryAnswer::amount(answer)
        }
    }
    pub(crate) fn amt_carry_choices(&self) -> alloc::vec::Vec<i32> {
        if matches!(self.native_history(), EquipmentHistory::Continuing { initial_section179, .. } if initial_section179 > 0)
        {
            alloc::vec![0]
        } else {
            alloc::vec![]
        }
    }
}

// Legacy financial consumers use signed64 cents. Preserve the actual sources
// and check their wide annual projection before those consumers run; failure is
// source unavailability, never a guessed zero or an artificial amount cap.
pub(crate) fn financial_issue(draft: &crate::FederalDraft) -> Option<(i64, &'static str)> {
    const OVERFLOW: &str = "ASSET_NONMACRS_FINANCIAL_UNAVAILABLE";
    let contains_new = draft
        .income
        .iter()
        .filter_map(|entry| recovery_assets(&entry.income))
        .any(|assets| assets.iter().any(BusinessAsset::nonmacrs))
        || draft
            .adjustments
            .iter()
            .flat_map(|a| &a.employee_expenses)
            .any(|claim| claim.equipment_assets.iter().any(BusinessAsset::nonmacrs));
    if !contains_new {
        return None;
    }
    for entry in &draft.income {
        let Some(assets) = recovery_assets(&entry.income) else {
            continue;
        };
        if assets.iter().any(|asset| asset.nonmacrs() && !asset.valid()) {
            return Some((entry.id, FACTS));
        }
    }
    let source_claims = crate::related_forms::effective_claimants(draft);
    let mid_quarter = crate::depreciation::mid_quarter(draft);
    let mut qualified = ratio::exact::whole(0);
    let mut available = ratio::exact::whole(0);
    let mut total = [ratio::exact::whole(0), ratio::exact::whole(0)];
    for entry in &draft.income {
        let Some(assets) = recovery_assets(&entry.income) else {
            continue;
        };
        qualified += assets
            .iter()
            .map(|asset| ratio::exact::whole(asset.qualified179_cost()))
            .sum::<ratio::exact::Exact>();
        available += assets
            .iter()
            .map(|asset| ratio::exact::whole(asset.available179()))
            .sum::<ratio::exact::Exact>();
        if ratio::exact::rounded(&qualified).is_none()
            || ratio::exact::rounded(&available).is_none()
        {
            return Some((entry.id, OVERFLOW));
        }
        for (system, amt) in [false, true].into_iter().enumerate() {
            let amounts = asset_depreciation_amounts(assets, mid_quarter, amt);
            let Some(annual) = amounts.expense_cents(0) else {
                return Some((entry.id, OVERFLOW));
            };
            let Some(native) = ratio::exact::rounded(&amounts.native_cents) else {
                return Some((entry.id, OVERFLOW));
            };
            let base = match &entry.income {
                crate::FederalIncome::Business { data } => {
                    let mut without_assets = data.clone();
                    without_assets.assets.clear();
                    without_assets.operating_cost_with_adjustment(mid_quarter, 0)
                }
                crate::FederalIncome::FarmRental { data } => {
                    let mut without_assets = data.clone();
                    without_assets.assets.clear();
                    without_assets.expenses_total_with_claims(
                        draft,
                        entry.id,
                        mid_quarter,
                        amt,
                        &source_claims,
                    )
                }
                crate::FederalIncome::NonbusinessProperty { expenses, .. } => *expenses,
                // Schedule E's source owner already carries its expenses,
                // building recovery and personal-day fractions in Exact.
                crate::FederalIncome::Rental { .. } => 0,
                _ => unreachable!(),
            };
            let vehicle_costs = assets.iter().fold(ratio::exact::whole(0), |sum, asset| {
                sum + ratio::exact::whole(asset.vehicle_operating())
                    + ratio::exact::whole(asset.vehicle_loan_interest())
            });
            let base = ratio::exact::whole(base) + vehicle_costs;
            total[system] += &base + ratio::exact::whole(native.max(annual));
            if ratio::exact::rounded(&total[system]).is_none() {
                return Some((entry.id, OVERFLOW));
            }
            if ratio::exact::rounded(&(&base + ratio::exact::whole(native))).is_none()
                || ratio::exact::rounded(&(base + ratio::exact::whole(annual))).is_none()
            {
                return Some((entry.id, OVERFLOW));
            }
        }
    }
    for (position, claim) in draft.adjustments.iter().flat_map(|a| &a.employee_expenses).enumerate()
    {
        let id = crate::asset_sale::employee_source_id(position);
        if claim.equipment_assets.iter().any(|asset| asset.nonmacrs() && !asset.valid()) {
            return Some((id, FACTS));
        }
        for asset in &claim.equipment_assets {
            qualified += ratio::exact::whole(asset.qualified179_cost());
            available += ratio::exact::whole(asset.available179());
        }
        if ratio::exact::rounded(&qualified).is_none()
            || ratio::exact::rounded(&available).is_none()
        {
            return Some((id, OVERFLOW));
        }
        for (system, amt) in [false, true].into_iter().enumerate() {
            let amounts = asset_depreciation_amounts(&claim.equipment_assets, mid_quarter, amt);
            let Some(annual) = amounts.expense_cents(0) else {
                return Some((id, OVERFLOW));
            };
            let other = ratio::exact::whole(claim.other)
                + ratio::exact::whole(claim.local_transportation)
                + ratio::exact::whole(claim.travel)
                + ratio::exact::whole(claim.meals);
            total[system] += other + ratio::exact::whole(annual);
            if ratio::exact::rounded(&total[system]).is_none() {
                return Some((id, OVERFLOW));
            }
        }
    }
    None
}

pub(crate) fn recovery_assets(income: &crate::FederalIncome) -> Option<&[BusinessAsset]> {
    match income {
        crate::FederalIncome::Business { data } => Some(&data.assets),
        crate::FederalIncome::FarmRental { data } => Some(&data.assets),
        crate::FederalIncome::Rental { data } => Some(&data.assets),
        crate::FederalIncome::NonbusinessProperty {
            kind: crate::NonbusinessPropertyKind::PersonalRental { assets, .. },
            ..
        } => Some(assets),
        _ => None,
    }
}
