// IRS Publication946, chapters2 and4: source records establish the recovery
// system and property classification; the shared schedule owns arithmetic.
// https://www.irs.gov/publications/p946
pub use crate::asset_model::{
    AdsClassLife, AdsElection, AdsLease, AdsLessee, AdsRestriction, AdsRestrictionPeriod,
    ForeignUseYear, MacrsRecoverySource, OriginalMacrsSystem, RealImprovement, RealImprovementKind,
};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Owner-dependent source question evidence supplied with the editor
/// This is transient question context, never an accepted tax source or an owner answer.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MacrsSourceQuestionContext {
    /// Placement year at which this editor's context was built; Save rechecks edited dates
    pub placed_year: u32,
    /// The actual page parent is a Schedule F farming activity
    pub farming: bool,
    /// Actual election belonging to this activity, supplied before source omission
    pub interest_limit_election: Option<crate::InterestLimitElection>,
    /// Actual class-wide elections for this owner
    pub elections: Vec<MacrsSourceClassElection>,
    /// Actual section263A election years for this owner
    pub capitalization_years: Vec<u32>,
    /// Preserve a supplied original-system answer when its question becomes otherwise unnecessary
    pub retained_original_system: bool,
    /// Preserve a supplied class-life record when its question becomes otherwise unnecessary
    pub retained_class_life: bool,
}
/// Editable source prerequisites, excluding answers constrained by calculation outputs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct MacrsSourceQuestionFacts {
    /// Actual voluntary election
    pub election: Option<AdsElection>,
    /// Actual statutory restriction periods
    pub restrictions: Option<Vec<AdsRestrictionPeriod>>,
    /// Actual annual foreign use
    pub foreign_years: Option<Vec<ForeignUseYear>>,
    /// Actual qualifying lease
    pub lease: Option<AdsLease>,
}
impl From<MacrsRecoverySourcePage> for MacrsSourceQuestionFacts {
    fn from(source: MacrsRecoverySourcePage) -> Self {
        Self {
            election: source.election,
            restrictions: source.restrictions,
            foreign_years: source.foreign_years,
            lease: source.lease,
        }
    }
}
impl MacrsSourceQuestionFacts {
    pub(crate) fn class_life_choices(
        &self,
        original: Option<&OriginalMacrsSystem>,
        statutory: bool,
        context: Option<&MacrsSourceQuestionContext>,
        class: crate::EquipmentClass,
    ) -> Vec<i32> {
        let invocation =
            context.map_or_else(AdsInvocation::default, |context| context.invocation(class));
        if context.is_some_and(|context| context.retained_class_life)
            || (!statutory
                && (actual_ads_trigger(
                    self.election.as_ref(),
                    self.restrictions.as_deref(),
                    self.foreign_years.as_deref(),
                    self.lease.as_ref(),
                    original,
                ) || invocation.election.is_some()
                    || invocation.required.is_some()))
        {
            alloc::vec![libertas_macros::variant_index!(ClassLifeRecord::Record) as i32]
        } else {
            Vec::new()
        }
    }
}
fn actual_ads_trigger(
    election: Option<&AdsElection>,
    restrictions: Option<&[AdsRestrictionPeriod]>,
    foreign_years: Option<&[ForeignUseYear]>,
    lease: Option<&AdsLease>,
    original: Option<&OriginalMacrsSystem>,
) -> bool {
    election.is_some()
        || lease.is_some()
        || restrictions.is_some_and(|rows| !rows.is_empty())
        || foreign_years.is_some_and(|rows| {
            rows.iter()
                .any(|row| u64::from(row.outside_us.used) * 2 > u64::from(row.outside_us.total))
        })
        || matches!(original, Some(OriginalMacrsSystem::Ads { .. }))
}

/// Actual class and election year used only to choose recovery questions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MacrsSourceClassElection {
    /// The original property class
    pub class: crate::GdsEquipmentClass,
    /// The original election
    pub election: AdsElection,
}
impl MacrsSourceQuestionContext {
    pub(crate) fn invocation(&self, class: crate::EquipmentClass) -> AdsInvocation<'_> {
        AdsInvocation {
            election: self
                .elections
                .iter()
                .find(|row| {
                    crate::EquipmentClass::from(row.class) == class
                        && row.election.tax_year == self.placed_year
                })
                .map(|row| &row.election),
            required: self
                .interest_limit_election
                .as_ref()
                .filter(|e| e.affects(class, crate::year_constants::TAX_YEAR))
                .map(|e| e.reason())
                .or_else(|| {
                    (self.farming
                        && self.capitalization_years.iter().any(|year| *year <= self.placed_year))
                    .then_some(AdsReason::UniformCapitalizationElection)
                }),
            unavailable: false,
            building: None,
            outside_us: None,
        }
    }
}

/// Nature of a separate improvement and its registered physical building
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RealImprovementPage {
    /// Physical building from the accepted source records
    #[libertas_enum_source("$.rental_energy_buildings")]
    pub building_id: i64,
    pub kind: RealImprovementKind,
}
impl From<RealImprovementPage> for RealImprovement {
    fn from(value: RealImprovementPage) -> Self {
        Self { building_id: value.building_id, kind: value.kind }
    }
}
impl From<RealImprovement> for RealImprovementPage {
    fn from(value: RealImprovement) -> Self {
        Self { building_id: value.building_id, kind: value.kind }
    }
}

pub use crate::building_model::{
    BuildingFirstService, BuildingUseSource, ImprovementBuildingFacts, RecoveryBuildingKind,
};

pub(crate) use crate::asset_model::{AdsInvocation, AdsReason};
use crate::kernel::{ratio::exact, source_fact::SourceFact};

/// A supplied Table B class-life record, offered only when its source is needed
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum ClassLifeRecord {
    /// Actual class-life record
    Record {
        /// Table B classification and class life
        life: AdsClassLife,
    },
}
impl From<AdsClassLife> for ClassLifeRecord {
    fn from(life: AdsClassLife) -> Self {
        Self::Record { life }
    }
}
impl From<ClassLifeRecord> for AdsClassLife {
    fn from(record: ClassLifeRecord) -> Self {
        let ClassLifeRecord::Record { life } = record;
        life
    }
}

/// Actual MACRS elections, restrictions and property records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MacrsRecoverySourcePage {
    /// Voluntary ADS election from the original return
    /// [NullText]
    /// No voluntary ADS election
    /// [SetText]
    /// Election year
    pub election: Option<AdsElection>,
    /// Actual statutory restriction periods
    /// An empty record list establishes none; unavailable records do not.
    /// [NullText]
    /// Records unavailable
    /// [SetText]
    /// Source records
    pub restrictions: Option<Vec<AdsRestrictionPeriod>>,
    /// Annual foreign use not already supplied by another source
    /// Do not repeat a year established by the property's location and use records.
    /// [NullText]
    /// Records unavailable
    /// [SetText]
    /// Source records
    pub foreign_years: Option<Vec<ForeignUseYear>>,
    /// Original recovery system, when the original schedule does not establish it
    /// [NullText]
    /// Original system unavailable
    /// [SetText]
    /// Original system
    #[libertas_constrained_by("^<MacrsRecoverySourcePage>.allowed_original_system")]
    pub original_system: Option<OriginalMacrsSystem>,
    /// Actual Table B asset classification and class life
    /// Supply a class life only when the property does not have a statutory ADS period.
    /// [NullText]
    /// No class-life source supplied
    /// [SetText]
    /// Class-life record
    #[libertas_constrained_by("^<MacrsRecoverySourcePage>.allowed_class_life")]
    pub class_life: Option<ClassLifeRecord>,
    /// Qualifying lease from the actual agreement
    /// [NullText]
    /// No qualifying lease
    /// [SetText]
    /// Lease
    pub lease: Option<AdsLease>,
    /// Nature and physical building of a separately recorded improvement
    /// [NullText]
    /// No separate improvement
    /// [SetText]
    /// Improvement
    pub improvement: Option<RealImprovementPage>,
    /// Applicable original-system source choices
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_asset_amt_carry_choices")]
    pub allowed_original_system: Vec<i32>,
    /// Applicable separately supplied class-life source
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_asset_amt_carry_choices")]
    pub allowed_class_life: Vec<i32>,
}

impl From<MacrsRecoverySourcePage> for MacrsRecoverySource {
    fn from(value: MacrsRecoverySourcePage) -> Self {
        Self {
            election: value.election,
            restrictions: value.restrictions,
            foreign_years: value.foreign_years,
            original_system: value.original_system,
            class_life: value.class_life.map(Into::into),
            lease: value.lease,
            improvement: value.improvement.map(Into::into),
        }
    }
}
impl MacrsRecoverySourcePage {
    pub(crate) fn from_source(
        value: MacrsRecoverySource,
        continuing: bool,
        statutory: bool,
    ) -> Self {
        let (original, life) = value.page_choices(continuing, statutory);
        Self {
            election: value.election,
            restrictions: value.restrictions,
            foreign_years: value.foreign_years,
            original_system: value.original_system,
            class_life: value.class_life.map(Into::into),
            lease: value.lease,
            improvement: value.improvement.map(Into::into),
            allowed_original_system: original,
            allowed_class_life: life,
        }
    }
}
impl MacrsRecoverySource {
    pub(crate) fn has_ads_trigger(&self) -> bool {
        actual_ads_trigger(
            self.election.as_ref(),
            self.restrictions.as_deref(),
            self.foreign_years.as_deref(),
            self.lease.as_ref(),
            self.original_system.as_ref(),
        )
    }
}

impl MacrsRecoverySource {
    pub(crate) fn page_choices(&self, continuing: bool, statutory: bool) -> (Vec<i32>, Vec<i32>) {
        self.page_choices_in_context(continuing, statutory, AdsInvocation::default())
    }
    pub(crate) fn page_choices_in_context(
        &self,
        continuing: bool,
        statutory: bool,
        context: AdsInvocation<'_>,
    ) -> (Vec<i32>, Vec<i32>) {
        (
            if continuing || self.original_system.is_some() {
                alloc::vec![0, 1]
            } else {
                Vec::new()
            },
            if self.class_life.is_some()
                || (!statutory
                    && (self.has_ads_trigger()
                        || context.election.is_some()
                        || context.required.is_some()))
            {
                alloc::vec![libertas_macros::variant_index!(ClassLifeRecord::Record) as i32]
            } else {
                Vec::new()
            },
        )
    }
}

impl crate::BusinessAsset {
    pub(crate) fn macrs_source_issue(&self) -> Option<&'static str> {
        self.macrs_source_issue_with_election(None)
    }
    pub(crate) fn macrs_source_issue_with_election(
        &self,
        election: Option<&AdsElection>,
    ) -> Option<&'static str> {
        self.macrs_source_issue_in_context(AdsInvocation {
            election,
            required: None,
            unavailable: false,
            building: None,
            outside_us: None,
        })
    }
    pub(crate) fn macrs_source_issue_in_context(
        &self,
        context: AdsInvocation<'_>,
    ) -> Option<&'static str> {
        let election = context.election;
        if self.macrs_source.is_none()
            && election.is_none()
            && context.required.is_none()
            && !context.unavailable
        {
            return None;
        }
        if self.macrs_source_fact_issue().is_some() {
            return Some("ASSET_MACRS_SOURCE_FACTS");
        }
        match self.source_ads_recovery_in_context(context) {
            Err(issue) => Some(issue),
            Ok(Some(_))
                if self.partial.is_some()
                    || self.source_macrs_amount_in_context(false, context).is_none() =>
            {
                Some("ASSET_ADS_ORIGINAL_HISTORY_UNAVAILABLE")
            }
            Ok(Some(_))
                if matches!(
                    self.history,
                    crate::EquipmentHistory::Continuing {
                        schedule: crate::EquipmentSchedule::Manual,
                        ..
                    }
                ) && self.history_details.is_none() =>
            {
                Some("ASSET_ADS_ORIGINAL_HISTORY_UNAVAILABLE")
            }
            _ => None,
        }
    }

    // Section179(e) names these separate improvements, never the whole building.
    pub(crate) fn needs_building_source(&self) -> bool {
        self.potential_real179()
            && (self.macrs_source.as_ref().and_then(|s| s.improvement.as_ref()).is_some_and(
                |row| {
                    matches!(row.kind, RealImprovementKind::Interior1250 { made_by_taxpayer: true })
                },
            ) || self.qualified179_cost() > 0
                || self.first_year_179() > 0)
    }

    pub(crate) fn source_straight_line_through_sale(
        &self,
        context: AdsInvocation<'_>,
    ) -> Option<i64> {
        let recovery = self.source_ads_recovery_in_context(context).ok()??;
        let sold = self.disposal_date()?;
        if !recovery.qip && !self.real179_in_context(context) {
            return None;
        }
        let base = self.business_share_of(self.cost).checked_sub(self.first_year_179())?;
        let mid_quarter =
            matches!(self.history, crate::EquipmentHistory::Continuing { mid_quarter: true, .. });
        let period = recovery.period_half_months;
        let served = if self.class == crate::EquipmentClass::ThirtyNine && !recovery.qip {
            i64::from(sold / 10000 - self.placed / 10000) * 24
                + 2 * (i64::from(sold / 100 % 100) - i64::from(self.placed / 100 % 100))
        } else {
            let quarter = i64::from((self.placed / 100 % 100).clamp(1, 12) - 1) / 3;
            let (first, convention) = if mid_quarter {
                (21 - 6 * quarter, crate::asset_sale::Convention::MidQuarter)
            } else {
                (12, crate::asset_sale::Convention::HalfYear)
            };
            first
                + 24 * (i64::from(sold / 10000) - i64::from(self.placed / 10000) - 1)
                + crate::asset_sale::disposal_half_months(sold, convention)
        };
        exact::rounded(
            &(exact::whole(base) * exact::whole(served.max(0)).min(period.clone()) / period),
        )
    }

    pub(crate) fn source_ads_amount(&self, mid_quarter: bool) -> Option<i64> {
        self.source_ads_amount_with_election(mid_quarter, None)
    }
    pub(crate) fn source_ads_amount_with_election(
        &self,
        mid_quarter: bool,
        election: Option<&AdsElection>,
    ) -> Option<i64> {
        self.source_ads_amount_in_context(
            mid_quarter,
            AdsInvocation { election, ..Default::default() },
        )
    }
    pub(crate) fn source_ads_amount_in_context(
        &self,
        mid_quarter: bool,
        context: AdsInvocation<'_>,
    ) -> Option<i64> {
        let schedule = self.source_straight_line_schedule(mid_quarter, context)?;
        let SourceFact::Reported(amounts) = schedule.recover().ok()? else {
            return None;
        };
        let amount = amounts.current.get();
        self.changed_nonlisted_use().map_or(Some(amount), |(used, total)| {
            crate::kernel::ratio::checked_fraction(amount, i64::from(used), i64::from(total))
        })
    }
    pub(crate) fn source_straight_line_schedule(
        &self,
        mid_quarter: bool,
        context: AdsInvocation<'_>,
    ) -> Option<crate::kernel::cost_recovery::straight_line::Schedule> {
        use crate::kernel::{
            carry::{Amount, Cents},
            cost_recovery::{Month, straight_line},
        };
        if self.partial.is_some() {
            return None;
        }
        let recovery = self.source_ads_recovery_in_context(context).ok()??;
        let (basis, prior, convention) = match self.history {
            crate::EquipmentHistory::New { section179, .. } => (
                self.business_basis()
                    .checked_sub(section179.elected())?
                    .checked_sub(self.source_bonus_amount_in_context(context))?,
                SourceFact::NotReported,
                mid_quarter,
            ),
            crate::EquipmentHistory::Continuing {
                initial_section179,
                carryforward,
                prior_depreciation,
                mid_quarter,
                schedule,
                ..
            } => {
                let bonus = match schedule {
                    crate::EquipmentSchedule::Tables { initial_bonus } => initial_bonus,
                    crate::EquipmentSchedule::Manual => {
                        self.history_details.as_ref()?.original_bonus
                    }
                };
                let basis =
                    self.history_basis().checked_sub(initial_section179)?.checked_sub(bonus)?;
                let prior = prior_depreciation
                    .checked_sub(initial_section179.checked_sub(carryforward)?)?
                    .checked_sub(bonus)?;
                (basis, SourceFact::Reported(Amount::<Cents>::new(prior).ok()?), mid_quarter)
            }
        };
        let convention = if self.class == crate::EquipmentClass::ThirtyNine && !recovery.qip {
            straight_line::Convention::MidMonth
        } else if convention {
            straight_line::Convention::MidQuarter
        } else {
            straight_line::Convention::HalfYear
        };
        Some(straight_line::Schedule {
            placed: Month { year: self.placed / 10000, month: self.placed / 100 % 100 },
            period_half_months: recovery.period_half_months,
            convention,
            tax_year: crate::year_constants::TAX_YEAR,
            basis: Amount::new(basis).ok()?,
            prior_recovery: prior,
            disposed: self
                .disposal_date()
                .map(|date| Month { year: date / 10000, month: date / 100 % 100 }),
        })
    }
}

// IRC168(g)(7): the election covers one taxpayer's entire placed-year
// property class, except buildings elected separately. Borrow the actual
// election; never manufacture another source packet or require it twice.

// Immutable invocation view retains actual election origins across conservative
// omission of another source's income. It is never serialized with the return.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct MacrsElectionView {
    sources: Vec<(crate::Owner, crate::EquipmentClass, u32, i64, AdsElection)>,
    farming: Vec<(crate::Owner, i64)>,
    interest: Vec<(crate::Owner, i64, crate::InterestLimitElection)>,
    capitalization: Vec<(crate::Owner, u32)>,
    rental_farming: Vec<(crate::Owner, i64)>,
    buildings: Vec<(i64, BuildingUseSource)>,
    rental_locations: Vec<(crate::Owner, i64, bool)>,
    building_links: Vec<i64>,
}
impl MacrsElectionView {
    pub(crate) fn new(draft: &crate::FederalDraft) -> Self {
        let associations = crate::source_associations::SourceAssociations::new(draft);
        let buildings: Vec<_> = associations
            .recovery_buildings()
            .into_iter()
            .map(|(id, facts)| (id, facts.clone()))
            .collect();
        let mut sources = Vec::new();
        let mut add = |owner, asset: &crate::BusinessAsset| {
            let building =
                asset.macrs_source.as_ref().and_then(|s| s.improvement.as_ref()).and_then(|row| {
                    buildings
                        .iter()
                        .find_map(|(id, facts)| (*id == row.building_id).then_some(facts))
                });
            let qip = asset.qualified_improvement(AdsInvocation { building, ..Default::default() })
                == Ok(true);
            if asset.nonmacrs()
                || (asset.class == crate::EquipmentClass::ThirtyNine && !qip)
                || asset.macrs_source_fact_issue().is_some()
            {
                return;
            }
            if let Some(election) =
                asset.macrs_source.as_ref().and_then(|source| source.election.as_ref())
            {
                sources.push((
                    owner,
                    if qip { crate::EquipmentClass::Fifteen } else { asset.class },
                    asset.placed / 10000,
                    asset.id,
                    election.clone(),
                ));
            }
        };
        for entry in &draft.income {
            if let Some(assets) = crate::nonmacrs::recovery_assets(&entry.income) {
                for asset in assets {
                    add(entry.owner.owned(), asset);
                }
            }
        }
        for claim in draft.adjustments.iter().flat_map(|a| &a.employee_expenses) {
            for asset in claim.vehicle_assets.iter().chain(&claim.equipment_assets) {
                add(claim.owner, asset);
            }
        }
        let mut farming = Vec::new();
        let mut capitalization = Vec::new();
        let mut rental_farming = Vec::new();
        for entry in &draft.income {
            let farm = match &entry.income {
                crate::FederalIncome::Business { data } => match &data.activity {
                    crate::BusinessActivity::Farming { data } => {
                        farming.push((entry.owner.owned(), entry.id));
                        data
                    }
                    _ => continue,
                },
                crate::FederalIncome::FarmRental { data } => {
                    rental_farming.push((entry.owner.owned(), entry.id));
                    &data.farm
                }
                _ => continue,
            };
            for cohort in &farm.specials.production {
                if let crate::FarmCapitalization::OriginalElectionOut { year } =
                    &cohort.tax_treatment
                    && !year.statement.trim().is_empty()
                {
                    capitalization.push((entry.owner.owned(), year.year));
                }
            }
        }
        let interest = draft
            .income
            .iter()
            .filter_map(|entry| {
                crate::interest_limit::source(&entry.income)
                    .map(|election| (entry.owner.owned(), entry.id, election.clone()))
            })
            .collect();
        let rental_locations = draft
            .income
            .iter()
            .filter_map(|entry| match &entry.income {
                crate::FederalIncome::Rental { data } => {
                    Some((entry.owner.owned(), entry.id, data.location.foreign()))
                }
                _ => None,
            })
            .collect();
        let building_links = associations.improvement_building_links();
        Self {
            sources,
            farming,
            capitalization,
            rental_farming,
            interest,
            buildings,
            rental_locations,
            building_links,
        }
    }
    pub(crate) fn question_context(
        &self,
        owner: crate::Owner,
        activity: Option<i64>,
        placed: u32,
    ) -> MacrsSourceQuestionContext {
        MacrsSourceQuestionContext {
            placed_year: placed / 10000,
            farming: false,
            interest_limit_election: self.interest.iter().find_map(|(actual, id, e)| {
                (*actual == owner && activity == Some(*id)).then(|| e.clone())
            }),
            elections: self
                .sources
                .iter()
                .filter(|(actual, ..)| *actual == owner)
                .filter_map(|(_, class, _, _, election)| {
                    Some(MacrsSourceClassElection {
                        class: (*class).try_into().ok()?,
                        election: election.clone(),
                    })
                })
                .collect(),
            retained_original_system: false,
            retained_class_life: false,
            capitalization_years: self
                .capitalization
                .iter()
                .filter_map(|(actual, year)| (*actual == owner).then_some(*year))
                .collect(),
        }
    }
    pub(crate) fn building_recovery_needed(&self, id: i64) -> bool {
        id > 0 && self.building_links.contains(&id)
    }
    pub(crate) fn building_context(&self, asset: &crate::BusinessAsset) -> AdsInvocation<'_> {
        AdsInvocation {
            building: asset.macrs_source.as_ref().and_then(|s| s.improvement.as_ref()).and_then(
                |row| {
                    self.buildings
                        .iter()
                        .find_map(|(id, facts)| (*id == row.building_id).then_some(facts))
                },
            ),
            ..Default::default()
        }
    }
    pub(crate) fn context<'a>(
        &'a self,
        owner: crate::Owner,
        activity: Option<i64>,
        asset: &crate::BusinessAsset,
    ) -> AdsInvocation<'a> {
        let mut context =
            self.page_context(owner, activity, asset.class, asset.placed / 10000, false);
        context.building = self.building_context(asset).building;
        if asset.qualified_improvement(context) == Ok(true) {
            context.election =
                self.sources.iter().find_map(|(actual, class, year, _, election)| {
                    (*actual == owner
                        && *class == crate::EquipmentClass::Fifteen
                        && *year == asset.placed / 10000)
                        .then_some(election)
                });
        }
        context
    }
    pub(crate) fn affects(
        &self,
        owner: crate::Owner,
        activity: Option<i64>,
        asset: &crate::BusinessAsset,
    ) -> bool {
        let invocation = self.context(owner, activity, asset);
        asset.macrs_source.is_some()
            || invocation.election.is_some()
            || invocation.required.is_some()
            || invocation.unavailable
    }
    pub(crate) fn page_context(
        &self,
        owner: crate::Owner,
        activity: Option<i64>,
        class: crate::EquipmentClass,
        year: u32,
        farming: bool,
    ) -> AdsInvocation<'_> {
        // The actual non-MACRS source remains outside section168's system election.
        if matches!(class, crate::EquipmentClass::NonMacrs { .. }) {
            return AdsInvocation::default();
        }
        let election = if class == crate::EquipmentClass::ThirtyNine
            || matches!(class, crate::EquipmentClass::NonMacrs { .. })
        {
            None
        } else {
            self.sources.iter().find_map(|(actual, actual_class, actual_year, _, election)| {
                (*actual == owner && *actual_class == class && *actual_year == year)
                    .then_some(election)
            })
        };
        let capital = self
            .capitalization
            .iter()
            .any(|(actual, elected)| *actual == owner && *elected <= year);
        let interest = self.interest.iter().find_map(|(actual, id, source)| {
            (*actual == owner
                && activity == Some(*id)
                && source.affects(class, crate::year_constants::TAX_YEAR))
            .then_some(source.reason())
        });
        let outside_us = self
            .rental_locations
            .iter()
            .find_map(|(actual, id, foreign)| {
                (*actual == owner
                    && activity == Some(*id)
                    && class == crate::EquipmentClass::ThirtyNine)
                    .then_some(*foreign)
            })
            .map(|foreign| crate::UseFraction { used: u32::from(foreign), total: 1 });
        AdsInvocation {
            election,
            required: interest
                .or_else(|| {
                    outside_us
                        .filter(|fraction| fraction.used * 2 > fraction.total)
                        .map(|_| AdsReason::ForeignUse)
                })
                .or_else(|| {
                    (capital
                        && (farming
                            || activity.is_some_and(|id| self.farming.contains(&(owner, id)))))
                    .then_some(AdsReason::UniformCapitalizationElection)
                }),
            building: None,
            outside_us,
            unavailable: interest.is_none()
                && capital
                && activity.is_some_and(|id| self.rental_farming.contains(&(owner, id))),
        }
    }
}
