use crate::Owner;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub use crate::home_site_source::{
    DaycareLicense, HomeCost, HomeCostKind, HomeCostOwners, HomeCostScope, HomeOfficeClaim,
    HomeOfficeSite, HomeTenure, OfficeActivity, OfficeBuilding, OfficeMethod, OfficeOldConvention,
    OfficeOldMethod, OfficeProduction, OfficeProductionAcquisition, OfficeProductionChange,
    OfficeProductionOperation, OfficeQualifiedUse, OfficeRecovery, OfficeRecoveryHistory,
    OfficeSimplifiedMonth,
};
pub(crate) use crate::home_site_source::{casualty_allowance, property_portions};

/// Homes used by this return's businesses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeOffices {
    /// Homes
    #[libertas_size(min = 0)]
    #[libertas_unordered]
    /// ----
    /// Home
    pub sites: Vec<HomeOfficeSitePage>,
}

pub(crate) fn graph_issue(
    draft: &crate::FederalDraft,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    for (index, site) in draft.home_sites.iter().enumerate() {
        if site.id <= 0 || draft.home_sites[..index].iter().any(|other| other.id == site.id) {
            return Some(
                ValidationIssue::new("HOME_SITE_IDENTITY_INVALID")
                    .at_field("id")
                    .in_list("sites", index),
            );
        }
        if let Some(issue) = site.issue() {
            return Some(issue.in_list("sites", index));
        }
        if site.mortgage_loans.iter().any(|id| {
            !draft.deductions.as_ref().is_some_and(|ded| {
                ded.mortgage_loans().unwrap_or_default().iter().any(|loan| {
                    loan.id == *id
                        && matches!(loan.details, crate::MortgageLoanDetails::FirstLender(_))
                })
            })
        }) {
            return Some(
                ValidationIssue::new("HOME_MORTGAGE_SOURCE_INVALID")
                    .at_field("mortgage_loans")
                    .in_list("sites", index),
            );
        }
        if site.mortgage_loans.iter().enumerate().any(|(position, loan)| {
            site.mortgage_loans[..position].contains(loan)
                || draft.home_sites[..index].iter().any(|other| other.mortgage_loans.contains(loan))
        }) {
            return Some(
                ValidationIssue::new("HOME_MORTGAGE_ROOT_TWICE")
                    .at_field("mortgage_loans")
                    .in_list("sites", index),
            );
        }
        for (claim_index, claim) in site.claims.iter().enumerate() {
            if claim
                .activities
                .iter()
                .any(|activity| !draft.income.iter().any(|entry| entry.id == activity.activity))
            {
                return Some(
                    ValidationIssue::new("HOME_ACTIVITY_SOURCE_UNAVAILABLE")
                        .at_field("claims")
                        .in_list("sites", index),
                );
            }
            if matches!(claim.method, OfficeMethod::Simplified { .. })
                && draft.home_sites[..index].iter().any(|other| {
                    other.claims.iter().any(|previous| {
                        previous.owner == claim.owner
                            && matches!(previous.method, OfficeMethod::Simplified { .. })
                    })
                })
            {
                return Some(
                    ValidationIssue::new("HOME_SIMPLIFIED_ONE_HOME_PER_OWNER")
                        .at_field("method")
                        .in_list("claims", claim_index)
                        .in_list("sites", index),
                );
            }
            if claim.activities.iter().any(|activity| {
                !draft.income.iter().any(|entry| {
                    entry.id == activity.activity
                        && entry.owner == crate::DocumentOwner::from(claim.owner)
                        && matches!(entry.income, crate::FederalIncome::Business { .. })
                })
            }) {
                return Some(
                    ValidationIssue::new("HOME_ACTIVITY_SOURCE_INVALID")
                        .at_field("claims")
                        .in_list("sites", index),
                );
            }
        }
    }
    for entry in &draft.income {
        let rows: Vec<_> = draft
            .home_sites
            .iter()
            .flat_map(|site| &site.claims)
            .flat_map(|claim| &claim.activities)
            .filter(|activity| activity.activity == entry.id)
            .collect();
        for select in [false, true] {
            let total = crate::kernel::ratio::checked_sum(rows.iter().map(|row| {
                crate::kernel::natural::units(
                    if select { row.expense_period_share } else { row.gross_share },
                    100,
                )
            }));
            if total.is_none_or(|total| total > 10000) {
                return Some(ValidationIssue::new("HOME_ACTIVITY_PERIOD_SHARES_EXCESS"));
            }
        }
    }
    if let Err(issue) = casualty_binding(draft) {
        return Some(issue);
    }
    None
}

// Missing reciprocal answers are accepted source records awaiting the other
// page. Invalid identities or conflicting answers never become an allocation.
// Ok(Some(site)) is incomplete; Ok(None) is a complete source association.
pub(crate) fn casualty_binding(
    draft: &crate::FederalDraft,
) -> Result<Option<i64>, crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    let events = draft.deductions.as_ref().map_or(&[][..], |ded| &ded.personal_casualties);
    let mut incomplete = None;
    for (index, site) in draft.home_sites.iter().enumerate() {
        for (position, event_id) in site.casualties.iter().enumerate() {
            let invalid = || {
                ValidationIssue::new("HOME_CASUALTY_PROPERTY_LINK_REQUIRED")
                    .at_field("casualties")
                    .in_list("sites", index)
            };
            if *event_id <= 0
                || site.casualties[..position].contains(event_id)
                || events.iter().filter(|event| event.id == *event_id).count() != 1
            {
                return Err(invalid());
            }
            let event = events.iter().find(|event| event.id == *event_id).ok_or_else(invalid)?;
            let mut properties =
                event.properties.iter().filter(|property| property.home_site == Some(site.id));
            if properties.next().is_none() {
                incomplete.get_or_insert(site.id);
            }
            if properties.next().is_some() {
                return Err(invalid());
            }
        }
    }
    for event in events {
        for (index, property) in event.properties.iter().enumerate() {
            if property.home_site.is_some() != property.business_damage.is_some() {
                return Err(ValidationIssue::new("HOME_CASUALTY_DAMAGE_REQUIRED"));
            }
            if let Some(home) = property.home_site {
                let invalid = || ValidationIssue::new("HOME_CASUALTY_HOME_BINDING_INVALID");
                if event.id <= 0
                    || events.iter().filter(|other| other.id == event.id).count() != 1
                    || property.id <= 0
                    || event.properties.iter().filter(|other| other.id == property.id).count() != 1
                    || event.properties[..index].iter().any(|other| other.home_site == Some(home))
                {
                    return Err(invalid());
                }
                let site =
                    draft.home_sites.iter().find(|site| site.id == home).ok_or_else(invalid)?;
                if !site.casualties.contains(&event.id) {
                    incomplete.get_or_insert(home);
                }
            }
        }
    }
    Ok(incomplete)
}

// A calculation view of the same accepted whole-property records. No source
// is rewritten or independently entered for personal versus business use.
pub(crate) fn personal_casualties(
    draft: &crate::FederalDraft,
) -> Result<Vec<crate::PersonalCasualty>, crate::Section179PreparedIssue> {
    casualty_events(draft, true)
}

// Both Form4684 worksheets use the same accepted records and filing ownership.
// The hypothetical worksheet retains business use; the actual form removes it.
pub(crate) fn casualty_events(
    draft: &crate::FederalDraft,
    personal_only: bool,
) -> Result<Vec<crate::PersonalCasualty>, crate::Section179PreparedIssue> {
    match casualty_binding(draft) {
        Ok(None) => {}
        Ok(Some(site)) => {
            return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                site,
                "HOME_CASUALTY_PROPERTY_LINK_REQUIRED",
            ));
        }
        Err(_) => {
            return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                0,
                "HOME_CASUALTY_HOME_BINDING_INVALID",
            ));
        }
    }
    let mut events =
        draft.deductions.as_ref().map_or_else(Vec::new, |ded| ded.personal_casualties.clone());
    for event in events.iter_mut().filter(|_| personal_only) {
        for property in &mut event.properties {
            let Some(home) = property.home_site else {
                continue;
            };
            let site = draft.home_sites.iter().find(|site| site.id == home).ok_or(
                crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                    home,
                    "HOME_CASUALTY_HOME_BINDING_INVALID",
                ),
            )?;
            let portions = property_portions(site, property).map_err(|key| {
                crate::Section179PreparedIssue::SourceBoundaryUnavailable(home, key)
            })?;
            let targets = [
                &mut property.basis,
                &mut property.before,
                &mut property.after,
                &mut property.reimbursement,
                &mut property.received,
            ];
            for (index, target) in targets.into_iter().enumerate() {
                let business = crate::kernel::ratio::checked_sum(
                    portions.iter().map(|portion| portion.amounts[index]),
                )
                .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
                *target = target
                    .checked_sub(business)
                    .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
            }
        }
    }
    crate::casualty::rental_home::project_rental_properties(draft, &mut events, personal_only)
        .map_err(|(id, key)| crate::Section179PreparedIssue::SourceBoundaryUnavailable(id, key))?;
    Ok(events)
}

// Page adapters keep accepted-source catalog paths on the editable request.
// Persisted facts retain the same values without request-relative metadata.
/// One home used by this return's businesses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeOfficeSitePage {
    /// Saved home identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_default(0)]
    pub id: i64,
    /// Address or your name for this home
    #[libertas_size(min = 1)]
    pub address: String,
    /// Total area of the home
    #[libertas_number(min = 1)]
    #[libertas_physical_unit("square-foot")]
    pub home_area: u32,
    /// How this return's paid home costs are divided between you
    pub paid_by: HomeCostOwners,
    /// Mortgage loans on this home
    /// Save each original on Mortgage first; later lenders follow its original.
    #[libertas_enum_source("$.mortgage_loans")]
    #[libertas_size(min = 0)]
    pub mortgage_loans: Vec<i64>,
    /// The home is owned or rented
    pub tenure: HomeTenure,
    /// Home-use choices for you and your spouse
    pub claims: Vec<HomeOfficeClaimPage>,
    /// Home costs paid in 2026 during qualifying business use
    /// Enter each bill once. Do not enter mortgage amounts again here.
    #[libertas_size(min = 0)]
    pub costs: Vec<HomeCostPage>,
    /// Casualty events affecting this home
    #[libertas_enum_source("$.casualty_events")]
    #[libertas_size(min = 0)]
    pub casualties: Vec<i64>,
}

/// One owner's use of this home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeOfficeClaimPage {
    /// Owner
    #[libertas_constrained_by("$.allowed_home_owners")]
    pub owner: Owner,
    /// First day available for this owner's qualifying business use in 2026
    #[libertas_date_only]
    #[libertas_number(min = 20260101, max = 20261231)]
    pub starts: u32,
    /// Last day available for this owner's qualifying business use in 2026
    #[libertas_date_only]
    #[libertas_number(min = 20260101, max = 20261231)]
    pub ends: u32,
    /// How the space qualifies
    pub use_kind: OfficeQualifiedUse,
    /// Method for all this owner's businesses in this home
    pub method: OfficeMethod,
    /// Businesses or farms using this home
    pub activities: Vec<OfficeActivityPage>,
    /// Operating carryover from the last actual-expense worksheet
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_operating: i64,
    /// Depreciation carryover from that worksheet, excluding the casualty portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_depreciation: i64,
    /// Excess-casualty portion of that carryover, from the supporting schedule
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_casualty: i64,
}

/// One accepted business or farm using this home
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OfficeActivityPage {
    /// Business or farm
    #[libertas_enum_source("$.home_activities")]
    pub activity: i64,
    /// Share of this activity's annual gross income attributable to this home
    #[libertas_number(min = 0, max = 100, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub gross_share: f64,
    /// Share of annual non-home expenses attributable to this home's period
    /// Use 100 for one home even when some gross income was earned elsewhere.
    #[libertas_number(min = 0, max = 100, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub expense_period_share: f64,
    /// Reason for the income/period allocation
    /// Time, investment and other circumstances; leave empty only for both full shares.
    #[libertas_size(min = 0)]
    pub attribution_reason: String,
    /// Parts used to allocate this home's deduction among its activities
    #[libertas_number(min = 1)]
    pub deduction_parts: u32,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for OfficeActivityPage {}

/// One home bill, excluding mortgage interest, points and mortgage insurance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeCostPage {
    /// Kind of cost
    pub kind: HomeCostKind,
    /// Amount paid during the qualifying period, for the scope selected below
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// First day of the period this cost covers
    #[libertas_date_only]
    #[libertas_number(min = 20260101, max = 20261231)]
    pub starts: u32,
    /// Last day of that period
    /// Split a bill into substantiated period amounts if qualifying use changed.
    #[libertas_date_only]
    #[libertas_number(min = 20260101, max = 20261231)]
    pub ends: u32,
    /// Which space or business share this amount covers
    pub scope: HomeCostScopePage,
}

/// Allocation of this bill
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HomeCostScopePage {
    /// Whole home: divide by owner payment share and that owner's business percentage
    WholeHome,
    /// One owner's paid share of a whole-home cost during that owner's use period
    /// This amount is already divided between owners; apply only this owner's business percentage.
    WholeHomeForOwner {
        /// Owner of this paid share
        #[libertas_constrained_by("$.allowed_home_owners")]
        owner: Owner,
    },
    /// Only one owner's qualifying exclusive space
    Exclusive {
        /// Owner using this exclusive space
        #[libertas_constrained_by("$.allowed_home_owners")]
        owner: Owner,
    },
    /// Only one owner's shared daycare space: apply the daycare time fraction
    DaycareShared {
        /// Owner using this daycare space
        #[libertas_constrained_by("$.allowed_home_owners")]
        owner: Owner,
    },
    /// A separately substantiated business share; this is already the business amount
    DifferentShare {
        /// Owner
        #[libertas_constrained_by("$.allowed_home_owners")]
        owner: Owner,
        /// How this business amount was determined
        #[libertas_size(min = 1)]
        reason: String,
    },
}

impl From<HomeOfficeSite> for HomeOfficeSitePage {
    fn from(value: HomeOfficeSite) -> Self {
        Self {
            id: value.id,
            address: value.address,
            home_area: value.home_area,
            paid_by: value.paid_by,
            mortgage_loans: value.mortgage_loans,
            tenure: value.tenure,
            claims: value.claims.into_iter().map(Into::into).collect(),
            costs: value.costs.into_iter().map(Into::into).collect(),
            casualties: value.casualties,
        }
    }
}

impl From<HomeOfficeSitePage> for HomeOfficeSite {
    fn from(value: HomeOfficeSitePage) -> Self {
        Self {
            id: value.id,
            address: value.address,
            home_area: value.home_area,
            paid_by: value.paid_by,
            mortgage_loans: value.mortgage_loans,
            tenure: value.tenure,
            claims: value.claims.into_iter().map(Into::into).collect(),
            costs: value.costs.into_iter().map(Into::into).collect(),
            casualties: value.casualties,
        }
    }
}

impl From<HomeOfficeClaim> for HomeOfficeClaimPage {
    fn from(value: HomeOfficeClaim) -> Self {
        Self {
            owner: value.owner,
            starts: value.starts,
            ends: value.ends,
            use_kind: value.use_kind,
            method: value.method,
            activities: value.activities.into_iter().map(Into::into).collect(),
            prior_operating: value.prior_operating,
            prior_depreciation: value.prior_depreciation,
            prior_casualty: value.prior_casualty,
        }
    }
}

impl From<HomeOfficeClaimPage> for HomeOfficeClaim {
    fn from(value: HomeOfficeClaimPage) -> Self {
        Self {
            owner: value.owner,
            starts: value.starts,
            ends: value.ends,
            use_kind: value.use_kind,
            method: value.method,
            activities: value.activities.into_iter().map(Into::into).collect(),
            prior_operating: value.prior_operating,
            prior_depreciation: value.prior_depreciation,
            prior_casualty: value.prior_casualty,
        }
    }
}

impl From<OfficeActivity> for OfficeActivityPage {
    fn from(value: OfficeActivity) -> Self {
        Self {
            activity: value.activity,
            gross_share: value.gross_share,
            expense_period_share: value.expense_period_share,
            attribution_reason: value.attribution_reason,
            deduction_parts: value.deduction_parts,
        }
    }
}

impl From<OfficeActivityPage> for OfficeActivity {
    fn from(value: OfficeActivityPage) -> Self {
        Self {
            activity: value.activity,
            gross_share: value.gross_share,
            expense_period_share: value.expense_period_share,
            attribution_reason: value.attribution_reason,
            deduction_parts: value.deduction_parts,
        }
    }
}

impl From<HomeCost> for HomeCostPage {
    fn from(value: HomeCost) -> Self {
        Self {
            kind: value.kind,
            amount: value.amount,
            starts: value.starts,
            ends: value.ends,
            scope: value.scope.into(),
        }
    }
}

impl From<HomeCostPage> for HomeCost {
    fn from(value: HomeCostPage) -> Self {
        Self {
            kind: value.kind,
            amount: value.amount,
            starts: value.starts,
            ends: value.ends,
            scope: value.scope.into(),
        }
    }
}

impl From<HomeCostScope> for HomeCostScopePage {
    fn from(value: HomeCostScope) -> Self {
        match value {
            HomeCostScope::WholeHome => Self::WholeHome,
            HomeCostScope::WholeHomeForOwner { owner } => Self::WholeHomeForOwner { owner },
            HomeCostScope::Exclusive { owner } => Self::Exclusive { owner },
            HomeCostScope::DaycareShared { owner } => Self::DaycareShared { owner },
            HomeCostScope::DifferentShare { owner, reason } => {
                Self::DifferentShare { owner, reason }
            }
        }
    }
}

impl From<HomeCostScopePage> for HomeCostScope {
    fn from(value: HomeCostScopePage) -> Self {
        match value {
            HomeCostScopePage::WholeHome => Self::WholeHome,
            HomeCostScopePage::WholeHomeForOwner { owner } => Self::WholeHomeForOwner { owner },
            HomeCostScopePage::Exclusive { owner } => Self::Exclusive { owner },
            HomeCostScopePage::DaycareShared { owner } => Self::DaycareShared { owner },
            HomeCostScopePage::DifferentShare { owner, reason } => {
                Self::DifferentShare { owner, reason }
            }
        }
    }
}
