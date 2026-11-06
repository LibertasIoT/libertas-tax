use crate::common::Owner;
use alloc::{boxed::Box, string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// One home used by this return's businesses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeOfficeSite {
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
    #[libertas_size(min = 0)]
    pub mortgage_loans: Vec<i64>,
    /// The home is owned or rented
    pub tenure: HomeTenure,
    /// Home-use choices for you and your spouse
    pub claims: Vec<HomeOfficeClaim>,
    /// Home costs paid in 2026 during qualifying business use
    /// Enter each bill once. Do not enter mortgage amounts again here.
    #[libertas_size(min = 0)]
    pub costs: Vec<HomeCost>,
    /// Casualty events affecting this home
    #[libertas_size(min = 0)]
    pub casualties: Vec<i64>,
}

/// Who paid the home costs included on this return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HomeCostOwners {
    /// You
    Taxpayer,
    /// Your spouse
    Spouse,
    /// Both of you
    Split {
        /// Your share, in parts
        #[libertas_number(min = 1)]
        you: u32,
        /// Your spouse's share, in the same parts
        #[libertas_number(min = 1)]
        spouse: u32,
    },
}

/// Owned or rented home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HomeTenure {
    /// Rented
    Rented,
    /// Owned
    Owned {
        /// Building and land depreciation history
        /// Simplified use does not require recovery records. Actual expenses need these records to calculate building depreciation.
        /// [NullText]
        /// Building recovery records are unavailable
        /// [SetText]
        /// Building recovery records are
        building: Option<Box<OfficeBuilding>>,
        /// Permanent improvements after first business use
        #[libertas_size(min = 0)]
        improvements: Vec<OfficeBuilding>,
    },
}

/// One owner's use of this home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeOfficeClaim {
    /// Owner
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
    pub activities: Vec<OfficeActivity>,
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

/// Qualifying business use
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum OfficeQualifiedUse {
    /// Regular and exclusive principal place of business
    PrincipalPlace,
    /// Regular and exclusive place to meet customers in the normal business
    MeetCustomers,
    /// Regular and exclusive separate business structure
    SeparateStructure,
    /// Regular inventory or product-sample storage; this is the only fixed business location
    InventoryStorage,
    /// Daycare under the special shared-use rule
    Daycare {
        /// State licensing or approval status
        license: DaycareLicense,
        /// Actual time the shared space was used for daycare in this period; zero if all daycare space is exclusive
        // Calendar capacity: at most366days ×24hours; native use dates provide the actual bound.
        #[libertas_number(min = 0, max = 8784)]
        #[libertas_physical_unit("hour")]
        hours: u32,
    },
    /// These uses are not established
    /// Leaves the requested home-use deduction out with a specific notice.
    NotEstablished,
}

/// Daycare license, registration, certification or approval
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum DaycareLicense {
    /// Applied and not rejected
    Applied,
    /// Granted and still effective
    Approved,
    /// Exempt under state law
    Exempt,
    /// None of these
    NoException,
}

/// Method for this owner and home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum OfficeMethod {
    /// Actual expenses
    Actual {
        /// Area used exclusively for qualifying business, excluding shared daycare space
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("square-foot")]
        exclusive_area: u32,
        /// Additional area shared between daycare and personal use
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("square-foot")]
        shared_daycare_area: u32,
    },
    /// Elect the simplified method on the timely original return
    Simplified {
        /// All twelve months; use zero days where there was no qualifying use
        #[libertas_size(min = 12, max = 12)]
        months: Vec<OfficeSimplifiedMonth>,
    },
}

/// Qualifying areas in one month, after allocating space to other people
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OfficeSimplifiedMonth {
    /// Month, January through December
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Time the unreduced qualifying space was available this month
    #[libertas_number(min = 0, max = 31)]
    #[libertas_physical_unit("day")]
    pub exclusive_days: u32,
    /// Qualifying area not subject to the daycare time reduction
    /// For an area change, use the area qualifying for at least 15 days. Includes qualifying inventory storage.
    #[libertas_number(min = 0)]
    #[libertas_physical_unit("square-foot")]
    pub exclusive_area: u32,
    /// Time the shared daycare space was available for all uses during its qualifying period
    #[libertas_number(min = 0, max = 31)]
    #[libertas_physical_unit("day")]
    pub shared_days: u32,
    /// Additional area shared between daycare and personal use
    /// Disjoint from the unreduced area; use the area qualifying for at least 15 days.
    #[libertas_number(min = 0)]
    #[libertas_physical_unit("square-foot")]
    pub shared_area: u32,
}

/// One accepted business or farm using this home
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OfficeActivity {
    /// Business or farm
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
impl Eq for OfficeActivity {}

/// One home bill, excluding mortgage interest, points and mortgage insurance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeCost {
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
    pub scope: HomeCostScope,
}

/// Home cost category
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HomeCostKind {
    /// Real estate taxes
    PropertyTax,
    /// Home insurance, excluding mortgage insurance
    Insurance,
    /// Rent
    Rent,
    /// Repairs and maintenance, excluding permanent improvements
    Repair,
    /// Utilities
    Utilities,
    /// Other current home operating costs
    Other,
}

/// Allocation of this bill
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HomeCostScope {
    /// Whole home: divide by owner payment share and that owner's business percentage
    WholeHome,
    /// One owner's paid share of a whole-home cost during that owner's use period
    /// This amount is already divided between owners; apply only this owner's business percentage.
    WholeHomeForOwner {
        /// Owner of this paid share
        owner: Owner,
    },
    /// Only one owner's qualifying exclusive space
    Exclusive {
        /// Owner using this exclusive space
        owner: Owner,
    },
    /// Only one owner's shared daycare space: apply the daycare time fraction
    DaycareShared {
        /// Owner using this daycare space
        owner: Owner,
    },
    /// A separately substantiated business share; this is already the business amount
    DifferentShare {
        /// Owner
        owner: Owner,
        /// How this business amount was determined
        #[libertas_size(min = 1)]
        reason: String,
    },
}

/// Building or separate permanent improvement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OfficeBuilding {
    /// Description for your schedule
    #[libertas_size(min = 1)]
    pub description: String,
    /// Date first used for business; retain the original date through simplified years
    #[libertas_date_only]
    pub first_used: u32,
    /// Date sold or permanently stopped being used for business, if any
    #[libertas_date_only]
    /// [NullText]
    /// Still in qualifying business use
    /// [SetText]
    /// Business use ended on
    pub stopped: Option<u32>,
    /// Adjusted whole-property basis at first business use, including land if a home
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub adjusted_basis: i64,
    /// Whole-property fair market value on that date, including land if a home
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub conversion_value: i64,
    /// Land portion of adjusted basis; zero for an improvement
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub land_basis: i64,
    /// Land portion of conversion fair market value
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub land_value: i64,
    /// Depreciation method and history from the existing depreciation schedule
    pub schedule: OfficeRecoveryHistory,
    /// Prior depreciation allowed or allowable after May 6, 1997, excluding unallowed carryovers
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_post_1997: i64,
    /// Prior depreciation allowed or allowable before May 7, 1997
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prior_pre_1997: i64,
    /// Qualified production property election, if claiming one
    /// [NullText]
    /// No qualified production property election
    /// [SetText]
    /// Qualified production property election details
    pub production: Option<OfficeProduction>,
}

/// Continue the existing lawful depreciation schedule
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum OfficeRecovery {
    /// Normal 39-year MACRS straight-line mid-month method
    Macrs39,
    /// Historical 31.5-year MACRS straight-line mid-month method
    Macrs315,
    /// ADS 40-year straight-line mid-month method
    Ads40,
    /// Pre-1987 ACRS schedule
    Acrs {
        /// Recovery period actually elected: 15, 18, 19, 35 or 45
        // Tax structure: the historical ACRS elections below span15–45years.
        #[libertas_number(min = 15, max = 45)]
        #[libertas_physical_unit("year")]
        years: u32,
        /// The alternate straight-line method was elected
        alternate: bool,
    },
    /// Pre-1981 conventional depreciation schedule
    Pre1981 {
        /// Original depreciable basis, excluding land
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
        /// Salvage value from the original schedule
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        salvage: i64,
        /// Original useful life
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("month")]
        life_months: u32,
        /// Method used on the existing schedule
        method: OfficeOldMethod,
    },
}

/// Conventional historical method
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum OfficeOldMethod {
    /// Straight line
    StraightLine,
    /// Declining balance, using the historical elected rate
    DecliningBalance {
        /// Annual declining-balance rate
        #[libertas_number(min = 0.01, max = 100, step = 0.01)]
        #[libertas_physical_unit("percent")]
        annual_rate: f64,
    },
    /// Sum of the years' digits
    SumOfYears,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for OfficeOldMethod {}

/// Section 168(n) production portion and election
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OfficeProduction {
    /// Business area actually integral to manufacturing, production or refining
    #[libertas_number(min = 1)]
    #[libertas_physical_unit("square-foot")]
    pub production_area: u32,
    /// What this area does
    pub operation: OfficeProductionOperation,
    /// Describe the actual product and process
    #[libertas_size(min = 1)]
    pub activity: String,
    /// The product is tangible personal property, excluding food or beverages made in the same building as the retail location that sells them
    pub qualified_product: bool,
    /// This process substantially transforms the product, rather than only packaging, relabeling or minor assembly
    pub substantial_transformation: bool,
    /// Construction began
    #[libertas_date_only]
    pub construction_started: u32,
    /// Original use begins with this taxpayer; no ADS and no excluded office/admin/lodging use in the claimed area
    pub qualifies: bool,
    /// Elect and designate this portion as QPP under Notice 2026-16 section 7
    pub elect: bool,
    /// Existing property acquisition exception instead of new construction
    /// [NullText]
    /// This was not an existing-property acquisition
    /// [SetText]
    /// Existing-property acquisition details
    pub acquired_existing: Option<OfficeProductionAcquisition>,
    /// Change to a nonqualifying use within ten years
    /// [NullText]
    /// No change to a nonqualifying use
    /// [SetText]
    /// The qualifying use changed
    pub changed_use: Option<OfficeProductionChange>,
}

/// Production operations under Notice 2026-16 section 5.02
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum OfficeProductionOperation {
    /// Manufacturing: materially changes inputs into a new product for customers; packaging, labeling or minor assembly alone do not qualify
    Manufacturing,
    /// Agricultural production: cultivating crops or raising livestock for customers, excluding non-livestock animals and services such as food marketing
    Agricultural,
    /// Chemical production: a chemical process formulates a product from organic or inorganic raw materials
    Chemical,
    /// Refining: purifies a substance into a useful, higher-value product
    Refining,
    /// None of these direct qualifying operations
    Other,
}

/// Notice 2026-16 section 4.06 acquisition facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OfficeProductionAcquisition {
    /// Acquisition date
    #[libertas_date_only]
    pub acquired: u32,
    /// Binding acquisition contract date, if any
    #[libertas_date_only]
    /// [NullText]
    /// No earlier binding acquisition contract
    /// [SetText]
    /// Binding acquisition contract date
    pub contract: Option<u32>,
    /// Neither this taxpayer nor a related person previously used the property; not a related-party or carryover-basis acquisition; no disqualifying production use in the statutory lookback period
    pub qualifies: bool,
}

/// Production use changed
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OfficeProductionChange {
    /// Date of change
    #[libertas_date_only]
    pub changed: u32,
    /// Portion of the elected production area changed
    #[libertas_number(min = 1)]
    #[libertas_physical_unit("square-foot")]
    pub area: u32,
}

/// Original schedule and subsequent history
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OfficeRecoveryHistory {
    /// Original method
    pub recovery: OfficeRecovery,
    /// Applicable historical convention
    pub convention: OfficeOldConvention,
    /// Total prior depreciation allowed or allowable, excluding undeducted carryover
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub accumulated: i64,
    /// Date a lawful historical switch to straight line began
    #[libertas_date_only]
    /// [NullText]
    /// No later switch to straight line
    /// [SetText]
    /// Switched to straight line on
    pub straight_line_from: Option<u32>,
    /// Prior years electing simplified for this home
    #[libertas_size(min = 0)]
    pub simplified_years: Vec<u32>,
}

/// Convention shown on the historical depreciation schedule
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum OfficeOldConvention {
    /// Mid-month
    MidMonth,
    /// Full month
    FullMonth,
    /// Half-year
    HalfYear,
    /// Actual days of use
    ActualDays,
}

impl HomeOfficeSite {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if self.address.trim().is_empty() || self.home_area == 0 {
            return Some(ValidationIssue::new("HOME_SITE_ADDRESS_AREA_REQUIRED"));
        }
        if self.claims.is_empty() || self.claims.len() > 2 {
            return Some(ValidationIssue::new("HOME_OWNER_CLAIM_REQUIRED"));
        }
        for (index, claim) in self.claims.iter().enumerate() {
            let issue = if self.claims[..index].iter().any(|other| other.owner == claim.owner) {
                Some(ValidationIssue::new("HOME_OWNER_CLAIM_TWICE").at_field("owner"))
            } else {
                claim.rule(self.home_area)
            };
            if let Some(issue) = issue {
                return Some(issue.in_list("claims", index));
            }
        }
        if let HomeTenure::Owned { building, improvements } = &self.tenure {
            for building in building.as_deref().into_iter().chain(improvements) {
                if building.description.trim().is_empty()
                    || !crate::dates::source_valid_date(building.first_used)
                    || building.stopped.is_some_and(|date| {
                        !crate::dates::source_valid_date(date) || date < building.first_used
                    })
                    || [
                        building.adjusted_basis,
                        building.conversion_value,
                        building.land_basis,
                        building.land_value,
                        building.prior_post_1997,
                        building.prior_pre_1997,
                        building.schedule.accumulated,
                    ]
                    .iter()
                    .any(|amount| *amount < 0)
                    || matches!(building.schedule.recovery, OfficeRecovery::Acrs { years, .. } if ![15,18,19,35,45].contains(&years))
                    || matches!(building.schedule.recovery, OfficeRecovery::Pre1981 { basis, salvage, life_months, .. } if basis < 0 || salvage < 0 || salvage > basis || life_months == 0)
                    || building.land_basis > building.adjusted_basis
                    || building.land_value > building.conversion_value
                    || building.schedule.straight_line_from.is_some_and(|date| {
                        !crate::dates::source_valid_date(date) || date < building.first_used
                    })
                    || building.schedule.simplified_years.iter().enumerate().any(|(index, year)| {
                        *year < building.first_used / 10000
                            || *year >= 2026
                            || building.schedule.simplified_years[..index].contains(year)
                    })
                {
                    return Some(
                        ValidationIssue::new("HOME_BUILDING_SOURCE_INVALID").at_field("tenure"),
                    );
                }
            }
        }
        if matches!(
            self.paid_by,
            HomeCostOwners::Split { you: 0, .. } | HomeCostOwners::Split { spouse: 0, .. }
        ) {
            return Some(ValidationIssue::new("HOME_PAYMENT_SHARE_UNAVAILABLE"));
        }
        for (index, cost) in self.costs.iter().enumerate() {
            if !crate::dates::source_valid_date(cost.starts)
                || !crate::dates::source_valid_date(cost.ends)
                || cost.starts > cost.ends
                || cost.amount < 0
                || cost.starts / 10000 != 2026
                || cost.ends / 10000 != 2026
                || matches!(&cost.scope, HomeCostScope::DifferentShare { reason, .. } if reason.trim().is_empty())
            {
                return Some(
                    ValidationIssue::new("HOME_COST_PERIOD_AMOUNT_INVALID").in_list("costs", index),
                );
            }
            let owner = match cost.scope {
                HomeCostScope::WholeHome => None,
                HomeCostScope::WholeHomeForOwner { owner }
                | HomeCostScope::Exclusive { owner }
                | HomeCostScope::DaycareShared { owner }
                | HomeCostScope::DifferentShare { owner, .. } => Some(owner),
            };
            if !self
                .claims
                .iter()
                .filter(|claim| owner.is_none_or(|owner| claim.owner == owner))
                .all(|claim| cost.starts >= claim.starts && cost.ends <= claim.ends)
                || owner.is_some_and(|owner| !self.claims.iter().any(|claim| claim.owner == owner))
            {
                return Some(
                    ValidationIssue::new("HOME_COST_OUTSIDE_USE_PERIOD").in_list("costs", index),
                );
            }
        }
        None
    }
}

impl HomeOfficeClaim {
    fn rule(&self, home_area: u32) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let at = |key, field| Some(ValidationIssue::new(key).at_field(field));
        let in_method =
            |key, field| Some(ValidationIssue::new(key).at_field(field).in_field("method"));
        if !crate::dates::source_valid_date(self.starts)
            || !crate::dates::source_valid_date(self.ends)
            || self.starts > self.ends
            || self.starts / 10000 != 2026
            || self.ends / 10000 != 2026
        {
            let first =
                !crate::dates::source_valid_date(self.starts) || self.starts / 10000 != 2026;
            return at("HOME_USE_DATES_REVERSED", if first { "starts" } else { "ends" });
        }
        if self.prior_operating < 0 || self.prior_depreciation < 0 || self.prior_casualty < 0 {
            return at(
                "HOME_CARRY_NEGATIVE",
                if self.prior_operating < 0 {
                    "prior_operating"
                } else if self.prior_depreciation < 0 {
                    "prior_depreciation"
                } else {
                    "prior_casualty"
                },
            );
        }
        if self.activities.is_empty() {
            return at("HOME_ACTIVITY_REQUIRED", "activities");
        }
        for (index, activity) in self.activities.iter().enumerate() {
            if !crate::kernel::natural::exact(activity.gross_share, 100)
                .is_some_and(|share| (0..=10000).contains(&share))
                || !crate::kernel::natural::exact(activity.expense_period_share, 100)
                    .is_some_and(|share| (0..=10000).contains(&share))
                || activity.deduction_parts == 0
                || ((crate::kernel::natural::units(activity.gross_share, 100) != 10000
                    || crate::kernel::natural::units(activity.expense_period_share, 100) != 10000)
                    && activity.attribution_reason.trim().is_empty())
                || self.activities[..index].iter().any(|other| other.activity == activity.activity)
            {
                return Some(
                    ValidationIssue::new("HOME_ACTIVITY_ALLOCATION_INVALID")
                        .at_list_item("activities", index),
                );
            }
        }
        let daycare = matches!(self.use_kind, OfficeQualifiedUse::Daycare { .. });
        match &self.method {
            OfficeMethod::Actual { exclusive_area, shared_daycare_area } => {
                let total = u64::from(*exclusive_area) + u64::from(*shared_daycare_area);
                if total == 0 || total > u64::from(home_area) {
                    return in_method("HOME_BUSINESS_AREA_ABOVE_HOME", "exclusive_area");
                }
                if !daycare && *shared_daycare_area != 0 {
                    return in_method("HOME_SHARED_AREA_NEEDS_DAYCARE", "shared_daycare_area");
                }
            }
            OfficeMethod::Simplified { months } => {
                if months.len() != 12
                    || !(1..=12)
                        .all(|month| months.iter().filter(|m| m.month == month).count() == 1)
                {
                    return in_method("HOME_SIMPLIFIED_TWELVE_MONTHS", "months");
                }
                for month in months {
                    let days = crate::dates::days_in_month(2026, month.month);
                    if month.exclusive_days > days || month.shared_days > days {
                        return in_method("HOME_SIMPLIFIED_DAYS_INVALID", "months");
                    }
                    if u64::from(month.exclusive_area) + u64::from(month.shared_area)
                        > u64::from(home_area)
                    {
                        return in_method("HOME_SIMPLIFIED_AREA_ABOVE_HOME", "months");
                    }
                    if !daycare && (month.shared_area != 0 || month.shared_days != 0) {
                        return in_method("HOME_SIMPLIFIED_SHARED_NEEDS_DAYCARE", "months");
                    }
                }
            }
        }
        if let OfficeQualifiedUse::Daycare { hours, .. } = self.use_kind {
            let has_shared = match &self.method {
                OfficeMethod::Actual { shared_daycare_area, .. } => *shared_daycare_area > 0,
                OfficeMethod::Simplified { months } => {
                    months.iter().any(|month| month.shared_area > 0)
                }
            };
            if (!has_shared && hours != 0) || self.daycare_fraction().is_none() {
                return Some(
                    ValidationIssue::new("HOME_DAYCARE_TIME_INVALID")
                        .at_field("hours")
                        .in_field("use_kind"),
                );
            }
        }
        None
    }
}

impl HomeOfficeClaim {
    pub(crate) fn fraction(&self, home_area: u32) -> Option<crate::kernel::ratio::exact::Exact> {
        use crate::kernel::ratio::exact;
        let OfficeMethod::Actual { exclusive_area, shared_daycare_area } = self.method else {
            return None;
        };
        let time = self.daycare_fraction()?;
        Some(
            (exact::whole(i64::from(exclusive_area))
                + exact::whole(i64::from(shared_daycare_area)) * time)
                / exact::whole(i64::from(home_area)),
        )
    }
    pub(crate) fn daycare_fraction(&self) -> Option<crate::kernel::ratio::exact::Exact> {
        use crate::kernel::ratio::exact;
        let OfficeQualifiedUse::Daycare { hours, .. } = self.use_kind else {
            return Some(exact::whole(0));
        };
        let days = match &self.method {
            OfficeMethod::Actual { .. } => {
                i64::from(crate::dates::days_between(self.starts, self.ends)) + 1
            }
            OfficeMethod::Simplified { months } => {
                months.iter().map(|m| i64::from(m.shared_days)).sum()
            }
        };
        if i64::from(hours) > days * 24 {
            return None;
        }
        if days == 0 {
            return (hours == 0).then(|| exact::whole(0));
        }
        exact::ratio(i64::from(hours), days * 24)
    }
    pub(crate) fn simplified_base(&self) -> Option<i64> {
        use crate::kernel::ratio::exact;
        let OfficeMethod::Simplified { months } = &self.method else {
            return None;
        };
        let time = self.daycare_fraction()?;
        let area = months.iter().fold(exact::whole(0), |area, month| {
            let exclusive =
                if month.exclusive_days >= 15 { month.exclusive_area.min(300) } else { 0 };
            let shared =
                if month.shared_days >= 15 { month.shared_area.min(300 - exclusive) } else { 0 };
            area + exact::whole(i64::from(exclusive)) + exact::whole(i64::from(shared)) * &time
        });
        exact::rounded(&(area * exact::whole(500) / exact::whole(12)))
    }
    pub(crate) fn qualifies(&self) -> bool {
        !matches!(
            self.use_kind,
            OfficeQualifiedUse::NotEstablished
                | OfficeQualifiedUse::Daycare { license: DaycareLicense::NoException, .. }
        )
    }
}

impl HomeOfficeSite {
    pub(crate) fn paid_fraction(&self, owner: Owner) -> Option<crate::kernel::ratio::exact::Exact> {
        use crate::kernel::ratio::exact;
        match self.paid_by {
            HomeCostOwners::Taxpayer => Some(exact::whole(i64::from(owner == Owner::Taxpayer))),
            HomeCostOwners::Spouse => Some(exact::whole(i64::from(owner == Owner::Spouse))),
            HomeCostOwners::Split { you, spouse } => {
                if you == 0 || spouse == 0 {
                    return None;
                }
                exact::ratio(
                    i64::from(if owner == Owner::Taxpayer { you } else { spouse }),
                    i64::from(you) + i64::from(spouse),
                )
            }
        }
    }
    pub(crate) fn business_costs(&self, claim: &HomeOfficeClaim) -> Option<[i64; 6]> {
        use crate::kernel::ratio::exact;
        let business = claim.fraction(self.home_area)?;
        let paid = self.paid_fraction(claim.owner)?;
        let time = claim.daycare_fraction()?;
        let mut costs = core::array::from_fn::<_, 6, _>(|_| exact::whole(0));
        for cost in &self.costs {
            let fraction = match &cost.scope {
                HomeCostScope::WholeHome => &paid * &business,
                HomeCostScope::WholeHomeForOwner { owner } if *owner == claim.owner => {
                    business.clone()
                }
                HomeCostScope::Exclusive { owner }
                | HomeCostScope::DifferentShare { owner, .. }
                    if *owner == claim.owner =>
                {
                    exact::whole(1)
                }
                HomeCostScope::DaycareShared { owner } if *owner == claim.owner => time.clone(),
                _ => continue,
            };
            let index = match cost.kind {
                HomeCostKind::PropertyTax => 0,
                HomeCostKind::Insurance => 1,
                HomeCostKind::Rent => 2,
                HomeCostKind::Repair => 3,
                HomeCostKind::Utilities => 4,
                HomeCostKind::Other => 5,
            };
            costs[index] += exact::whole(cost.amount) * fraction;
        }
        let mut native = [0; 6];
        for (target, value) in native.iter_mut().zip(costs) {
            *target = exact::rounded(&value)?;
        }
        Some(native)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct HomePropertyPortion {
    pub owner: Owner,
    // Whole-source cents in order: basis, FMV before, FMV after,
    // expected reimbursement, reimbursement received in the return year.
    pub amounts: [i64; 5],
}

pub(crate) fn property_portions(
    site: &HomeOfficeSite,
    property: &crate::CasualtyProperty,
) -> Result<Vec<HomePropertyPortion>, &'static str> {
    use crate::kernel::ratio::exact;
    let mut total = exact::whole(0);
    let mut portions = Vec::new();
    let mut prefixes: [exact::ExactPrefix; 5] = core::array::from_fn(|_| Default::default());
    let source = [
        property.basis,
        property.before,
        property.after,
        property.reimbursement,
        property.received,
    ];
    if source.iter().any(|value| *value < 0) {
        return Err("HOME_CASUALTY_SOURCE_UNAVAILABLE");
    }
    for claim in &site.claims {
        if !claim.qualifies() || !matches!(claim.method, OfficeMethod::Actual { .. }) {
            continue;
        }
        let fraction = claim.fraction(site.home_area).ok_or("HOME_USE_FRACTION_UNAVAILABLE")?;
        total += &fraction;
        if total > exact::whole(1) {
            return Err("HOME_COMBINED_USE_ABOVE_HOME");
        }
        let mut amounts = [0; 5];
        for (index, value) in source.iter().enumerate() {
            amounts[index] = prefixes[index].next(&(exact::whole(*value) * &fraction), |value| {
                exact::rounded(value).expect(
                    "cumulative business fraction is nonnegative and at most the whole source",
                )
            });
        }
        portions.push(HomePropertyPortion { owner: claim.owner, amounts });
    }
    Ok(portions)
}

// Allocate this property's hypothetical personal allowance through the same
// accepted owner geometry as its basis, values and reimbursement. A prefix
// keeps an odd cent with the original claim order without duplicating it.
pub(crate) fn casualty_allowance(
    site: &HomeOfficeSite,
    owner: Owner,
    whole: i64,
) -> Result<i64, &'static str> {
    use crate::kernel::ratio::exact;
    if whole < 0 {
        return Err("HOME_CASUALTY_SOURCE_UNAVAILABLE");
    }
    let mut total = exact::whole(0);
    let mut prefix = exact::ExactPrefix::default();
    for claim in &site.claims {
        if !claim.qualifies() || !matches!(claim.method, OfficeMethod::Actual { .. }) {
            continue;
        }
        let fraction = claim.fraction(site.home_area).ok_or("HOME_USE_FRACTION_UNAVAILABLE")?;
        total += &fraction;
        if total > exact::whole(1) {
            return Err("HOME_COMBINED_USE_ABOVE_HOME");
        }
        let amount = prefix.next(&(exact::whole(whole) * fraction), |value| {
            exact::rounded(value)
                .expect("an allocated nonnegative source fraction is within its whole amount")
        });
        if claim.owner == owner {
            return Ok(amount);
        }
    }
    Err("HOME_CASUALTY_SOURCE_UNAVAILABLE")
}

pub(crate) struct HomeCostColumns {
    pub direct: [crate::kernel::ratio::exact::Exact; 6],
    pub indirect: [crate::kernel::ratio::exact::Exact; 6],
}
impl HomeOfficeSite {
    pub(crate) fn cost_columns(&self, claim: &HomeOfficeClaim) -> Option<HomeCostColumns> {
        use crate::kernel::ratio::exact;
        let paid = self.paid_fraction(claim.owner)?;
        let time = claim.daycare_fraction()?;
        let mut result = HomeCostColumns {
            direct: core::array::from_fn(|_| exact::whole(0)),
            indirect: core::array::from_fn(|_| exact::whole(0)),
        };
        for cost in &self.costs {
            let (direct, fraction) = match &cost.scope {
                HomeCostScope::WholeHome => (false, paid.clone()),
                HomeCostScope::WholeHomeForOwner { owner } if *owner == claim.owner => {
                    (false, exact::whole(1))
                }
                HomeCostScope::Exclusive { owner }
                | HomeCostScope::DifferentShare { owner, .. }
                    if *owner == claim.owner =>
                {
                    (true, exact::whole(1))
                }
                HomeCostScope::DaycareShared { owner } if *owner == claim.owner => {
                    (true, time.clone())
                }
                _ => continue,
            };
            let index = match cost.kind {
                HomeCostKind::PropertyTax => 0,
                HomeCostKind::Insurance => 1,
                HomeCostKind::Rent => 2,
                HomeCostKind::Repair => 3,
                HomeCostKind::Utilities => 4,
                HomeCostKind::Other => 5,
            };
            let column = if direct { &mut result.direct } else { &mut result.indirect };
            column[index] += exact::whole(cost.amount) * fraction;
        }
        Some(result)
    }
}
