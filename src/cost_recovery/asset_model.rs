// Property source facts, original recovery records and local classifications.
use crate::Asset179Carryover;
use crate::building_model::{BuildingFirstService, BuildingUseSource, RecoveryBuildingKind};
use crate::investment_credit_model::{InvestmentCredit, RehabilitationCredit};
use crate::investment_credit_source::energy_percent;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::kernel::{
    phase,
    ratio::{self, exact},
    source_fact::SourceFact,
};
use crate::year_constants::{
    ENERGY_BASIS_REDUCTION_PERCENT, REHABILITATION_BASIS_REDUCTION_PERCENT,
};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Purchased business or investment equipment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct BusinessAsset {
    /// Property use throughout your ownership
    /// Use the lease and property-use records for this sold property.
    /// [NullText]
    /// Property-use records are unavailable
    /// [SetText]
    /// The property-use records show
    pub farm_rental_use: Option<crate::FarmRentalPropertyUse>,

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
    #[libertas_default($null)]
    pub history_details: Option<crate::HistoricalAssetRecords>,
    /// Accepted MACRS source records; absence does not establish GDS eligibility
    #[libertas_hidden]
    pub macrs_source: Option<crate::MacrsRecoverySource>,
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
    /// Only for an asset placed in service before 2026 whose business use this year is 50% or less. Enter its whole-number business use in the earlier years, the same in each. The App adds back the section 179 benefit (Form 4797 Part IV column (a)) or, for a listed vehicle, the depreciation allowed above straight-line (column (b)) as other income of the business, and raises the asset's basis by it. Leave empty if it was used 50% or less in every earlier year.
    /// [NullText]
    /// Business use did not fall from above 50% to 50% or less
    /// [SetText]
    /// Earlier business use above 50% was
    // Percentage encoding: business use cannot exceed 100 percent.
    #[libertas_number(min = 51, max = 100)]
    #[libertas_physical_unit("percent")]
    pub earlier_business_percent: Option<u32>,
    /// MACRS recovery class
    /// Use the IRS activity/asset classification, not an estimated useful life. Computers generally use 5 years and office furniture 7 years. Specialized equipment follows its assigned activity class. Nonresidential real property (a building or structural component used in this business) is generally 39-year straight-line with the mid-month convention; enter only the depreciable building cost, excluding land. Keep separately entered interior improvements in this class and link the registered building facts: qualifying improvements use 15-year GDS or 20-year ADS straight-line recovery.
    pub class: EquipmentClass,
    /// Depreciation history
    pub history: EquipmentHistory,
    /// GDS method for basis remaining after section 179 and bonus depreciation
    /// For new assets, apply the same method to this recovery class across businesses and nonbusiness rentals. For continuing assets, enter the original GDS method from the depreciation record; the App applies the required switch to straight line. With 100% bonus there is no remaining basis after section 179. Ignored for nonresidential real property and qualified improvement property, which use their statutory straight-line recovery.
    pub method: EquipmentMethod,
    // The pages that write this record ask only for qualifying purchased
    // equipment; the scope the calculation supports is checked by qualifies().
    /// Business or investment use
    /// For a car, van, pickup or panel truck whose actual costs you deduct, enter its exact business and total miles. Other property uses its documented percentage.
    pub listed: ListedProperty,
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

/// Owned vehicle classification for depreciation and section 179
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum VehicleKind {
    /// Four-wheeled passenger car made primarily for public roads, unloaded weight 6,000 pounds or less
    /// Use the unloaded-weight test for a car; trucks and vans use gross vehicle weight rating.
    PassengerCar,
    /// Truck or van at or below the legal passenger-auto weight threshold
    /// Its gross vehicle weight rating is 6,000 pounds or less.
    PassengerTruckVan,
    /// Four-wheeled public-road vehicle primarily designed or used to carry passengers, over 6,000 and not over 14,000 pounds GVWR
    HeavySuv {
        /// Actual vehicle design exception to the SUV section 179 cap
        exception: SuvException,
    },
    /// Other heavy truck or van outside the passenger and SUV caps, still subject to listed-property rules
    /// Choose Qualified nonpersonal for a cargo vehicle over 14,000 pounds loaded gross weight, or another vehicle meeting that separate exception.
    HeavyTruckVan,
    /// Qualified nonpersonal-use vehicle excepted from listed-property rules
    /// By its nature, this vehicle is unlikely to have more than minimal personal use, such as a qualifying specialized utility truck. An ordinary heavy truck or van is not automatically excepted. This vehicle editor supports five-year recovery only.
    QualifiedNonpersonal,
}

impl VehicleKind {
    pub(crate) fn passenger_automobile(self) -> bool {
        matches!(self, Self::PassengerCar | Self::PassengerTruckVan)
    }
    // IRC179(b)(5), Rev.Proc.2025-32 section4.24: only elected section179
    // cost is limited. Bonus and MACRS use the remaining basis normally.
    pub(crate) fn section179_limit(self) -> Option<i64> {
        matches!(self, Self::HeavySuv { exception: SuvException::None })
            .then_some(crate::year_constants::SECTION_179_SUV_LIMIT_2026_CENTS)
    }
}

/// Direct design exception to the SUV section 179 cap
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum SuvException {
    /// No listed design exception; apply the SUV limit
    None,
    /// Designed for more than nine passengers behind the driver
    TenRearPassengers,
    /// At least six-foot interior cargo area not readily accessible from the passenger compartment
    SixFootCargo,
    /// Integral enclosed driver and load body with no rear seats; no body section extends more than thirty inches ahead of the windshield's leading edge
    IntegralLoadEnclosure,
}

/// Vehicle deducted with actual expenses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ListedProperty {
    /// Not a vehicle
    No {
        /// Business or investment use
        /// Personal use is not deductible. Nonbusiness rentals require 100% investment use. For continuing property whose use fell to 50% or less this year, enter this year's use here and the earlier use above.
        // Percentage encoding: business use cannot exceed 100 percent.
        #[libertas_number(min = 1, max = 100)]
        #[libertas_physical_unit("percent")]
        business_percent: u32,
    },
    /// Vehicle using actual expenses
    Vehicle {
        /// Total distance driven during 2026
        /// Include business, commuting and personal driving from the odometer records.
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("mile")]
        total_miles: u32,
        /// Business distance driven during 2026
        /// Exclude commuting, personal travel and distance claimed by another business. Every business-use allocation divides this by the total, without rounding that fraction.
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("mile")]
        business_miles: u32,
        /// Fuel, insurance, repairs, registration and other operating costs for 2026
        /// Enter the whole-vehicle amount. The App deducts the business-mile share on the car and truck line. Do not enter loan interest, business parking and tolls, lease payments or depreciation here.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        operating_costs: i64,
        /// Annual vehicle-loan interest paid
        /// Enter the whole vehicle amount. The App allocates its business-mile share to the loan-interest line; do not deduct that share elsewhere. Enter zero when there is no loan.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        loan_interest: i64,
        /// Vehicle design and applicable legal cap
        kind: VehicleKind,
    },
}

/// Equipment recovery class under GDS
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EquipmentClass {
    /// 3-year property
    Three,
    /// 5-year property
    Five,
    /// 7-year property
    Seven,
    /// 10-year property
    Ten,
    /// 15-year property
    Fifteen,
    /// 20-year property
    Twenty,
    /// 39-year nonresidential real property
    ThirtyNine,
    /// Actual property outside MACRS
    NonMacrs { recovery: crate::NonMacrsRecovery },
}
impl EquipmentClass {
    pub(crate) fn years(self) -> i64 {
        match self {
            Self::Three => 3,
            Self::Five => 5,
            Self::Seven => 7,
            Self::Ten => 10,
            Self::Fifteen => 15,
            Self::Twenty => 20,
            Self::ThirtyNine => 39,
            Self::NonMacrs { .. } => unreachable!("non-MACRS has its own recovery clock"),
        }
    }
}

/// Section 179 property classification and election
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum Section179 {
    /// Property does not qualify for section 179
    Ineligible,
    /// Qualifying purchased equipment used predominantly in an active business
    /// Confirm eligible tangible property or qualifying off-the-shelf computer software under section 179, more than 50% business use, eligible purchase and domestic active-business use. All eligible purchases and active-business income are included; unentered working-capital interest, section 1231 gains/losses, entity allocations or amended separate-to-joint elections need their own treatment. No excluded land improvement, special noncorporate lease, tax-exempt/foreign use, like-kind basis or other exception. Include all eligible purchases even when electing zero: they count toward the annual phaseout.
    Eligible {
        /// Business cost elected for section 179
        /// Enter your chosen amount, including zero. It cannot exceed business cost or the shared annual dollar limit. The App applies the business-income limit and uses current elections first, then prior carryforwards oldest year first, with proportional asset allocation within each year. Undeducted elected cost remains a separate carryforward, never bonus or MACRS basis. Retain the election and asset allocation with Form 4562.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        elected: i64,
    },
}
impl Section179 {
    pub(crate) fn elected(self) -> i64 {
        match self {
            Self::Ineligible => 0,
            Self::Eligible { elected } => elected,
        }
    }
}

/// Equipment depreciation history
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EquipmentHistory {
    /// First placed in service in 2026
    New {
        /// Section 179 classification and elected business cost
        section179: Section179,
        /// Bonus depreciation election for this recovery class
        /// Apply the same election to bonus-eligible 2026 property in this class across your return. The acquisition date determines eligibility and percentage; equipment acquired before September 28, 2017 receives no bonus. Retain any election out with the timely return.
        bonus: BonusElection,
        /// Use IRS percentage tables
        /// Yes preserves the published table percentages, including their rounding. No uses the manual MACRS formula. Keep the chosen approach with this asset's depreciation records for later years.
        tables: bool,
    },
    /// Placed in service before 2026
    Continuing {
        /// Original section 179 elected cost
        /// Copy the elected business cost from the original asset schedule, or zero. It equals amounts already included in Prior depreciation plus any unused section 179 carryforward below. No recapture or change in business use. Do not enter a current-year election or deduct this historical amount again.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        initial_section179: i64,
        /// Unused section 179 carryforward attributable to this asset through 2025
        /// Use the prior Form 4562 and asset allocation records. This is not a QBI/NOL carryover or remaining MACRS basis. Enter zero when fully deducted. The asset remains owned in the same active business; no disposition, allocation change, recapture or unresolved suspended-loss limitation. The original suspended deduction must be wholly attributable to this qualified business for QBI when released. Pre-2018 amounts, mixed qualified/nonqualified allocations or changed specified-service history need separate QBI treatment.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        carryforward: i64,
        /// Prior depreciation allowed or allowable through 2025
        /// Copy the cumulative income-producing amount from the asset's depreciation records, including previously allowed amounts of the original section 179 election and first-year bonus. This reduces original income-producing basis; do not enter the remaining basis or deduct this amount again. If earlier returns claimed a different amount, enter the allowable amount here and the difference as a section 481(a) adjustment entered under Accounting-method changes (Form 3115, DCN 7).
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        prior_depreciation: i64,
        /// AMT depreciation records
        amt: EquipmentAmtHistory,
        /// Original convention was mid-quarter
        /// Use the convention on the original depreciation schedule. No means half-year. The App uses the original service quarter, not the 2026 purchase mix.
        mid_quarter: bool,
        /// Original computation approach
        /// Keep the approach used on the prior depreciation schedule. Table percentages apply to the original depreciable income-producing basis after original section 179 and first-year bonus, not the current remaining basis.
        schedule: EquipmentSchedule,
    },
}

/// Alternative minimum tax asset history
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EquipmentAmtHistory {
    /// Same MACRS method and computation approach as regular tax
    /// Independently income-limited section 179 and capped bonus still use the actual AMT source records.
    Same,
    /// Separate post-1998 150% declining-balance history
    /// For non-section-1250 equipment using regular 200% declining balance that requires AMT refiguring. Use the same recovery period and convention. Not property covered by the post-2015 bonus-eligible exception; no ADS, changed use, missing deductions or other basis differences.
    Separate {
        /// Cumulative AMT depreciation allowed or allowable through 2025
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        prior_depreciation: i64,
        /// Original AMT computation approach
        schedule: EquipmentSchedule,
    },
}

/// Original MACRS computation approach
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EquipmentSchedule {
    /// Manual MACRS calculation
    Manual,
    /// IRS percentage tables
    Tables {
        /// Original first-year bonus deduction
        /// Enter the actual income-producing bonus amount from the original depreciation schedule, or zero if elected out. It is already included in Prior depreciation; do not subtract it twice. Enter the actual bonus after the original section 179 election.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        initial_bonus: i64,
    },
}

/// Bonus depreciation election
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum BonusElection {
    /// Apply the available bonus allowance
    /// Ordinary qualifying 2026 equipment uses 100% if acquired after January 19, 2025, or 20% if acquired after September 27, 2017 and before January 20, 2025. MACRS recovers any remaining basis.
    Apply,
    /// Elect out for this recovery class
    ElectOut,
}

/// Equipment depreciation choice
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum EquipmentMethod {
    /// Ordinary GDS declining balance
    /// 200% declining balance for 3-, 5-, 7- and 10-year classes; 150% for 15- and 20-year classes.
    Regular,
    /// 150% declining balance
    Declining150,
    /// Straight line over GDS recovery period
    StraightLine,
}

crate::community::divide_record!(BusinessAsset {
    amounts: [
        class,
        cost,
        credit_reduction,
        history,
        listed,
        investment_credit,
        history_details,
        amt_section179_income_carry
    ],
    gains: [disposal, partial],
    facts: [
        id,
        description,
        acquired,
        placed,
        earlier_business_percent,
        method,
        macrs_source,
        farm_rental_use
    ],
});
crate::community::divide_enum!(EquipmentHistory {
    records: [
        New { amounts: [section179], facts: [bonus, tables] },
        Continuing {
            amounts: [initial_section179, carryforward, prior_depreciation, amt, schedule],
            facts: [mid_quarter]
        },
    ],
    units: [],
    tuples: [],
});
crate::community::divide_enum!(EquipmentAmtHistory {
    records: [Separate { amounts: [prior_depreciation, schedule], facts: [] }],
    units: [Same],
    tuples: [],
});
crate::community::divide_enum!(EquipmentSchedule {
    records: [Tables { amounts: [initial_bonus], facts: [] }],
    units: [Manual],
    tuples: [],
});
crate::community::divide_enum!(ListedProperty {
    records: [
        No { amounts: [], facts: [business_percent] },
        Vehicle {
            amounts: [operating_costs, loan_interest],
            facts: [total_miles, business_miles, kind]
        }
    ],
    units: [],
    tuples: [],
});

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_enum!(Section179 {
    records: [Eligible { amounts: [elected], facts: [] }],
    units: [Ineligible],
    tuples: [],
});

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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub section179_allowed: i64,
    /// Section 179 actually allowed on the AMT schedule for this year
    /// Enter the independently established AMT amount, including documented zero. Unknown records cannot supply an AMT recovery schedule.
    #[libertas_default(Unavailable)]
    pub amt_section179_allowed: crate::SourceCents,
    /// Bonus allowed or allowable this year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub bonus: i64,
    /// Other regular depreciation allowed or allowable
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub regular: i64,
    /// Other AMT depreciation allowed or allowable
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt: i64,
    /// Documented increase other than current depreciation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis_increase: i64,
    /// Documented decrease other than current depreciation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis_decrease: i64,
    /// Deduction already recaptured in this year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub recaptured: i64,
}

/// Complete original schedule and annual history for the selected route
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct HistoricalAssetRecords {
    /// Original income-producing basis after documented acquisition and credit adjustments
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub original_business_basis: i64,
    /// Original income-producing use
    pub original_income_use: UseFraction,
    /// Original qualified business use
    pub original_qualified_use: UseFraction,
    /// Original elected amount, including its documented unused balance
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub original_section179_elected: i64,
    /// Original allowed or allowable bonus
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub first_year_capped_deduction: i64,
    /// Remaining basis entering the first year after recovery; reconciled to prior history
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
    amounts: [
        section179_allowed,
        amt_section179_allowed,
        bonus,
        regular,
        amt,
        basis_increase,
        basis_decrease,
        recaptured
    ],
    gains: [],
    facts: [year, income_use, qualified_use],
});
crate::community::divide_record!(AutoRecoveryHistory {
    amounts: [first_year_capped_deduction, post_recovery_basis],
    gains: [],
    facts: [first_year_bonus, safe_harbor_adopted_year, car_or_truck],
});

/// Actual non-MACRS recovery system and original source facts
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum NonMacrsRecovery {
    /// Elected units of production
    Units { facts: UnitProduction },
    /// ACRS or another historical section167 schedule
    Historical { facts: HistoricalDepreciation },
    /// Purchased software outside section197
    Software { facts: SoftwareRecovery },
    /// Finite-life right outside section197
    FiniteLife { facts: FiniteLifeRecovery },
    /// Section167(g) income forecast
    IncomeForecast { facts: IncomeForecastRecovery },
}

/// Original unit-of-production election and current extraction/output records
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct UnitProduction {
    /// Actual section168(c) property class for the section199A qualified-property period
    /// This does not change the elected units-of-production depreciation method.
    pub qbi_class: GdsEquipmentClass,
    /// Valid first-year election excluding this asset from MACRS
    pub elected_originally: bool,
    /// Actual current-year output units
    #[libertas_number(min = 0)]
    pub current_units: u32,
    /// Total remaining estimated output including current year
    #[libertas_number(min = 1)]
    pub remaining_units: u32,
    /// Adjusted basis remaining before current deduction
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub remaining_basis: i64,
    /// Reasonable salvage value not depreciated
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub salvage: i64,
}

/// Original ACRS or pre-MACRS recovery facts
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct HistoricalDepreciation {
    /// Actual section168(c) property class for the section199A qualified-property period
    /// This does not replace the original historical depreciation system.
    pub qbi_class: GdsEquipmentClass,
    /// Actual property character and leasing use on the original schedule
    pub property: HistoricalPropertyKind,
    /// Original legal system and election
    pub kind: HistoricalRecoveryKind,
    /// Adjusted recoverable basis entering this year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub remaining_basis: i64,
    /// Convention on original lawful schedule
    pub original_convention: AssetConvention,
}

/// Property character needed for pre-1987 AMT and disposition rules
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum HistoricalPropertyKind {
    /// Owned tangible personal property not leased to another person
    PersonalOwned,
    /// Tangible personal property leased to another person
    PersonalLeased,
    /// Building or other section1250 real property
    RealProperty,
}

/// Original system; rates supplied by source-versioned tables
#[derive(Clone, Copy, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HistoricalRecoveryKind {
    /// Original ACRS class and alternate-method election
    Acrs { period: AcrsPeriod, alternate: bool },
    /// Original pre-1981 straight-line recovery
    Pre1981StraightLine {
        /// Remaining useful life established by the current original schedule
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("month")]
        remaining_months: u32,
        /// Applicable salvage floor
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        salvage: i64,
    },
    /// Lawful declining-balance multiplier and original useful life
    Pre1981Declining {
        /// Original useful life establishing the original declining-balance rate
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("month")]
        original_useful_months: u32,
        /// Applicable salvage floor
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        salvage: i64,
        /// Original lawful declining-balance multiplier
        #[libertas_number(min = 1, max = 2, step = 0.01)]
        multiplier: f64,
    },
    /// Documented non-MACRS straight-line property
    Other167StraightLine {
        /// Remaining useful life established by the current original schedule
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("month")]
        remaining_months: u32,
        /// Applicable salvage floor
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        salvage: i64,
    },
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for HistoricalRecoveryKind {}

/// ACRS original recovery class; service date controls available class/table
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum AcrsPeriod {
    /// Three-year
    Three,
    /// Five-year
    Five,
    /// Ten-year
    Ten,
    /// Fifteen-year
    Fifteen,
    /// Eighteen-year real property
    Eighteen,
    /// Nineteen-year real property
    Nineteen,
    /// Source-permitted alternate recovery period
    Alternate {
        /// Original property class before the alternate-method election
        original_class: AcrsOriginalClass,
        /// Original source-permitted alternate recovery period
        #[libertas_number(min = 1, max = 45)]
        #[libertas_physical_unit("year")]
        years: u32,
    },
}

/// Original ACRS property class before an alternate recovery election
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum AcrsOriginalClass {
    /// Original three-year personal-property class
    Personal3,
    /// Original five-year personal-property class
    Personal5,
    /// Original ten-year personal-property class
    Personal10,
    /// Ordinary fifteen-year real property
    Real15,
    /// Eighteen-year real property
    Real18,
    /// Nineteen-year real property
    Real19,
    /// Fifteen-year low-income housing
    LowIncome15,
}

/// Purchased software outside section 197
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]

pub struct SoftwareRecovery {
    /// Statutory software classification
    pub classification: SoftwareKind,
    /// Tax-exempt-use lease term imposing a 125-percent minimum
    /// [NullText]
    /// This software is not subject to a tax-exempt-use lease
    /// [SetText]
    /// The tax-exempt-use lease term was
    pub exempt_lease: Option<SoftwareTaxExemptLease>,
    /// Documented bonus acquisition/original-use eligibility
    pub bonus_eligible: bool,
}

/// Actual agreement creating tax-exempt use under section168(h)
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct SoftwareTaxExemptLease {
    /// Date the actual lease agreement was entered into
    #[libertas_date_only]
    pub entered: u32,
    /// Actual lease term, including applicable renewal options
    #[libertas_number(min = 1)]
    #[libertas_physical_unit("month")]
    pub months: u32,
}

/// Purchased software or another owner
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum SoftwareKind {
    /// Purchased separately and not a section 197 intangible
    Standalone,
    /// Readily public, nonexclusive and not substantially modified
    PublicNonexclusive,
    /// Business-acquisition intangible routed to existing section 197
    BusinessAcquisition,
    /// Software development costs routed to research owner
    Development,
}

/// Separately acquired finite-life right outside section 197
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum FiniteLifeRecovery {
    /// Separately purchased patent or copyright, outside a business acquisition
    PatentCopyright {
        /// Actual remaining legal term
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("month")]
        legal_remaining_months: u32,
        /// Actual remaining economic useful life
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("month")]
        useful_months: u32,
        /// Reasonable salvage value on the original schedule
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        salvage: i64,
    },
    /// Separately acquired mortgage-servicing rights: statutory108 months
    MortgageServicing,
    /// Eligible created intangible under Reg1.167(a)-3(b): statutory180 months
    CreatedEligible,
    /// Eligible created real-property-benefit intangible: statutory300 months
    CreatedRealPropertyBenefits,
    /// Other separately purchased right to an unspecified amount over a fixed term
    OtherFiniteRight {
        /// Actual remaining term, including reasonably expected renewals
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("month")]
        remaining_months: u32,
    },
}
impl FiniteLifeRecovery {
    pub(crate) fn kind(&self) -> FiniteLifeKind {
        match self {
            Self::PatentCopyright { .. } => FiniteLifeKind::PatentCopyright,
            Self::MortgageServicing => FiniteLifeKind::MortgageServicing,
            Self::CreatedEligible => FiniteLifeKind::CreatedEligible,
            Self::CreatedRealPropertyBenefits => FiniteLifeKind::CreatedRealPropertyBenefits,
            Self::OtherFiniteRight { .. } => FiniteLifeKind::OtherFiniteRight,
        }
    }
    pub(crate) fn legal_remaining_months(&self) -> u32 {
        match *self {
            Self::PatentCopyright { legal_remaining_months, .. } => legal_remaining_months,
            Self::OtherFiniteRight { remaining_months } => remaining_months,
            Self::MortgageServicing => 108,
            Self::CreatedEligible => 180,
            Self::CreatedRealPropertyBenefits => 300,
        }
    }
    pub(crate) fn useful_months(&self) -> u32 {
        match *self {
            Self::PatentCopyright { useful_months, .. } => useful_months,
            _ => self.legal_remaining_months(),
        }
    }
    pub(crate) fn salvage(&self) -> i64 {
        match *self {
            Self::PatentCopyright { salvage, .. } => salvage,
            _ => 0,
        }
    }
}

/// Source-controlled non-section-197 right
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum FiniteLifeKind {
    /// Separately acquired patent or copyright
    PatentCopyright,
    /// Mortgage-servicing rights: 108 months
    MortgageServicing,
    /// Eligible created intangible under applicable fifteen-year regulation
    CreatedEligible,
    /// Eligible created intangible related to real-property benefits: twenty-five years
    CreatedRealPropertyBenefits,
    /// Other separately acquired finite right under regulation 1.167(a)-14
    OtherFiniteRight,
}

/// Annual income forecast and separate look-back facts
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]

pub struct IncomeForecastRecovery {
    /// Eligible type of income-forecast property
    pub kind: ForecastPropertyKind,
    /// Actual section 167(g) income attributable to this property this year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub current_income: i64,
    /// Total anticipated lifetime income through tenth following year, including earned income
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub forecast_total_income: i64,
    /// Remaining recoverable basis including lawful cost additions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub remaining_basis: i64,
    /// Original treatment of participations and residuals
    pub participations: ForecastParticipations,
    /// Required Form 8866 recomputation or statutory exception status
    pub lookback: ForecastLookback,
}

/// Section 167(g) income forecast property
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ForecastPropertyKind {
    /// Film or video
    FilmVideo,
    /// Sound recording
    Sound,
    /// Book or copyright
    BookCopyright,
    /// Patent
    Patent,
    /// Other legally eligible income-forecast property
    OtherEligible,
}

/// Original election for participations and residuals
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ForecastParticipations {
    /// Include eligible economic-performance amounts in basis
    Basis,
    /// Deduct eligible amounts when paid under original election
    PaidExpense,
    /// No participations or residuals
    None,
}

/// Mandatory look-back status; missing tax is never zero
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ForecastLookback {
    /// No current recomputation year or event
    NoRecomputation,
    /// Original cost basis is at most one hundred thousand dollars
    SmallCostException,
    /// Actual and forecast income through recomputation year are within ten percent
    WithinTenPercent {
        /// Actual income through the recomputation year
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        actual: i64,
        /// Corresponding income originally forecast for that period
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        forecast: i64,
    },
    /// Current recomputation requires unentered Form 8866 tax and rates
    WorksheetNeeded,
}

impl NonMacrsRecovery {
    pub(crate) fn section179_property(&self) -> bool {
        match self {
            Self::Software { facts } => {
                facts.exempt_lease.is_none()
                    && facts.classification == SoftwareKind::PublicNonexclusive
            }
            Self::Units { .. }
            | Self::Historical { .. }
            | Self::FiniteLife { .. }
            | Self::IncomeForecast { .. } => false,
        }
    }
}

impl NonMacrsRecovery {
    pub(crate) fn section1250(&self) -> bool {
        matches!(self, Self::Historical { facts } if facts.property == HistoricalPropertyKind::RealProperty)
    }
}

crate::community::divide_enum!(EquipmentClass {
    records: [NonMacrs { amounts: [recovery], facts: [] }],
    units: [Three, Five, Seven, Ten, Fifteen, Twenty, ThirtyNine],
    tuples: [],
});
crate::community::divide_enum!(NonMacrsRecovery {
    records: [
        Units { amounts: [facts], facts: [] },
        Historical { amounts: [facts], facts: [] },
        Software { amounts: [], facts: [facts] },
        FiniteLife { amounts: [facts], facts: [] },
        IncomeForecast { amounts: [facts], facts: [] },
    ],
    units: [],
    tuples: [],
});
crate::community::divide_record!(UnitProduction {
    amounts: [remaining_basis, salvage],
    gains: [],
    facts: [qbi_class, elected_originally, current_units, remaining_units],
});
crate::community::divide_record!(HistoricalDepreciation {
    amounts: [remaining_basis, kind],
    gains: [],
    facts: [qbi_class, property, original_convention],
});
crate::community::divide_enum!(HistoricalRecoveryKind {
    records: [
        Acrs { amounts: [], facts: [period, alternate] },
        Pre1981StraightLine { amounts: [salvage], facts: [remaining_months] },
        Pre1981Declining { amounts: [salvage], facts: [multiplier, original_useful_months] },
        Other167StraightLine { amounts: [salvage], facts: [remaining_months] }
    ],
    units: [],
    tuples: [],
});
crate::community::divide_enum!(FiniteLifeRecovery {
    records: [
        PatentCopyright { amounts: [salvage], facts: [legal_remaining_months, useful_months] },
        OtherFiniteRight { amounts: [], facts: [remaining_months] }
    ],
    units: [MortgageServicing, CreatedEligible, CreatedRealPropertyBenefits],
    tuples: [],
});
crate::community::divide_record!(IncomeForecastRecovery {
    amounts: [current_income, forecast_total_income, remaining_basis, lookback],
    gains: [],
    facts: [kind, participations],
});
crate::community::divide_enum!(ForecastLookback {
    records: [WithinTenPercent { amounts: [actual, forecast], facts: [] }],
    units: [NoRecomputation, SmallCostException, WorksheetNeeded],
    tuples: [],
});

/// GDS classification offered only by the ordinary property editor
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum GdsEquipmentClass {
    /// 3-year property
    Three,
    /// 5-year property
    Five,
    /// 7-year property
    Seven,
    /// 10-year property
    Ten,
    /// 15-year property
    Fifteen,
    /// 20-year property
    Twenty,
    /// 39-year nonresidential real property
    ThirtyNine,
}
impl From<GdsEquipmentClass> for EquipmentClass {
    fn from(value: GdsEquipmentClass) -> Self {
        match value {
            GdsEquipmentClass::Three => Self::Three,
            GdsEquipmentClass::Five => Self::Five,
            GdsEquipmentClass::Seven => Self::Seven,
            GdsEquipmentClass::Ten => Self::Ten,
            GdsEquipmentClass::Fifteen => Self::Fifteen,
            GdsEquipmentClass::Twenty => Self::Twenty,
            GdsEquipmentClass::ThirtyNine => Self::ThirtyNine,
        }
    }
}
impl TryFrom<EquipmentClass> for GdsEquipmentClass {
    type Error = ();
    fn try_from(value: EquipmentClass) -> Result<Self, Self::Error> {
        Ok(match value {
            EquipmentClass::Three => Self::Three,
            EquipmentClass::Five => Self::Five,
            EquipmentClass::Seven => Self::Seven,
            EquipmentClass::Ten => Self::Ten,
            EquipmentClass::Fifteen => Self::Fifteen,
            EquipmentClass::Twenty => Self::Twenty,
            EquipmentClass::ThirtyNine => Self::ThirtyNine,
            EquipmentClass::NonMacrs { .. } => return Err(()),
        })
    }
}

/// Actual MACRS elections, restrictions and property records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MacrsRecoverySource {
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
    pub original_system: Option<OriginalMacrsSystem>,
    /// Actual Table B asset classification and class life
    /// Supply a class life only when the property does not have a statutory ADS period.
    /// [NullText]
    /// No class-life source supplied
    /// [SetText]
    /// Class-life record
    pub class_life: Option<AdsClassLife>,
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
    pub improvement: Option<RealImprovement>,
}

/// Actual year of a voluntary election; the containing source owns its scope
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AdsElection {
    /// Tax year on the timely election
    pub tax_year: u32,
}

/// Period during which an actual statutory restriction applies
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AdsRestrictionPeriod {
    /// First applicable tax year
    pub first_year: u32,
    /// Final applicable tax year
    /// [NullText]
    /// Still applies
    /// [SetText]
    /// Last applicable year
    pub last_year: Option<u32>,
    /// Statutory reason from the source records
    pub kind: AdsRestriction,
}

/// Restrictions independent of foreign use, listed use and business elections
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum AdsRestriction {
    TaxExemptUse,
    TaxExemptBondFinanced,
    RestrictedImport,
}

/// Actual foreign-use fraction for a year not supplied by its source owner
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignUseYear {
    pub year: u32,
    pub outside_us: crate::UseFraction,
}

/// Original recovery system; straight-line method alone cannot distinguish it
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum OriginalMacrsSystem {
    Gds,
    Ads { began_year: u32 },
}

/// Table B source classification and class life, rather than a chosen ADS life
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AdsClassLife {
    pub table_asset_class: String,
    #[libertas_number(min = 1)]
    #[libertas_physical_unit("month")]
    pub months: u32,
}

/// Actual qualifying lease term, including periods required by the tax rules
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AdsLease {
    #[libertas_number(min = 1)]
    #[libertas_physical_unit("month")]
    pub term_months: u32,
    pub lessee: AdsLessee,
}

/// Actual lessee class used by the ADS lease-term floor
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum AdsLessee {
    TaxExempt,
    Government,
    ForeignNonPartnership,
}

/// Nature of a separate improvement and its actual physical-building identity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RealImprovement {
    pub building_id: i64,
    pub kind: RealImprovementKind,
}

/// Actual nature of the work; location never reclassifies section1245 equipment
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RealImprovementKind {
    Interior1250 { made_by_taxpayer: bool },
    Interior1245Equipment,
    Roof,
    Hvac,
    FireProtectionOrAlarm,
    Security,
    Enlargement,
    ElevatorOrEscalator,
    StructuralFramework,
    Other,
}

/// The actual reason an ADS schedule applies, retained only during calculation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AdsReason {
    VoluntaryElection,
    TaxExemptUse,
    TaxExemptBondFinanced,
    RestrictedImport,
    ForeignUse,
    ListedUse,
    RealPropertyTrade,
    FarmingTrade,
    UniformCapitalizationElection,
}
impl AdsReason {
    pub(crate) fn required(self) -> bool {
        self != Self::VoluntaryElection
    }
}

// These facts are supplied by their existing source owners. In particular,
// income earned abroad is not evidence that an asset was used abroad.
pub(crate) struct AdsOwnerFacts {
    pub outside_us: SourceFact<crate::UseFraction>,
    pub listed_use_requires_ads: bool,
    pub real_property_trade_requires_ads: bool,
    pub farming_trade_requires_ads: bool,
    pub capitalization_election_requires_ads: bool,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct AdsInvocation<'a> {
    pub election: Option<&'a AdsElection>,
    pub required: Option<AdsReason>,
    pub unavailable: bool,
    pub building: Option<&'a BuildingUseSource>,
    pub outside_us: Option<crate::UseFraction>,
}

pub(crate) struct AdsRecovery {
    pub reasons: Vec<AdsReason>,
    pub period_half_months: exact::Exact,
    pub qip: bool,
}
impl AdsRecovery {
    pub(crate) fn is_ads(&self) -> bool {
        !self.reasons.is_empty()
    }
    pub(crate) fn required(&self) -> bool {
        self.reasons.iter().any(|reason| reason.required())
    }
}

impl MacrsRecoverySource {
    // Missing records remain unavailable. An actual known mandatory reason
    // already proves required ADS, without pretending the other records exist.
    pub(crate) fn ads_reasons(
        &self,
        year: u32,
        owner: AdsOwnerFacts,
    ) -> SourceFact<Vec<AdsReason>> {
        let mut reasons = Vec::new();
        let mut unavailable = false;
        if self.election.as_ref().is_some_and(|e| e.tax_year <= year) {
            reasons.push(AdsReason::VoluntaryElection);
        }
        match &self.restrictions {
            None => unavailable = true,
            Some(periods) => {
                for period in periods {
                    if period.first_year <= year && period.last_year.is_none_or(|end| year <= end) {
                        reasons.push(match period.kind {
                            AdsRestriction::TaxExemptUse => AdsReason::TaxExemptUse,
                            AdsRestriction::TaxExemptBondFinanced => {
                                AdsReason::TaxExemptBondFinanced
                            }
                            AdsRestriction::RestrictedImport => AdsReason::RestrictedImport,
                        });
                    }
                }
            }
        }
        let outside_us = match owner.outside_us {
            SourceFact::Unavailable => self
                .foreign_years
                .as_ref()
                .and_then(|years| years.iter().find(|row| row.year == year))
                .map_or(SourceFact::Unavailable, |row| SourceFact::Reported(row.outside_us)),
            established => established,
        };
        match outside_us {
            SourceFact::Reported(fraction)
                if fraction.total > 0 && fraction.used <= fraction.total =>
            {
                if u64::from(fraction.used) * 2 > u64::from(fraction.total) {
                    reasons.push(AdsReason::ForeignUse);
                }
            }
            SourceFact::NotReported => {}
            _ => unavailable = true,
        }
        for (applies, reason) in [
            (owner.listed_use_requires_ads, AdsReason::ListedUse),
            (owner.real_property_trade_requires_ads, AdsReason::RealPropertyTrade),
            (owner.farming_trade_requires_ads, AdsReason::FarmingTrade),
            (owner.capitalization_election_requires_ads, AdsReason::UniformCapitalizationElection),
        ] {
            if applies {
                reasons.push(reason);
            }
        }
        if unavailable && !reasons.iter().any(|reason| reason.required()) {
            SourceFact::Unavailable
        } else if reasons.is_empty() {
            SourceFact::NotReported
        } else {
            SourceFact::Reported(reasons)
        }
    }

    // Pub946, ADS recovery periods and the qualifying-lease floor. A class
    // life is never inferred from the asset's different GDS recovery period.
    pub(crate) fn ads_period(&self, statutory_months: Option<u32>) -> SourceFact<exact::Exact> {
        let statutory_months = statutory_months.or_else(|| {
            self.class_life
                .as_ref()
                .and_then(|source| (source.table_asset_class.trim() == "00.12").then_some(60))
        });
        let months = match statutory_months {
            Some(months) if months > 0 => exact::whole(i64::from(months)),
            Some(_) => return SourceFact::Unavailable,
            None => match &self.class_life {
                Some(source)
                    if source.months > 0 && !source.table_asset_class.trim().is_empty() =>
                {
                    exact::whole(i64::from(source.months))
                }
                _ => return SourceFact::Unavailable,
            },
        };
        let months = match &self.lease {
            Some(lease) if lease.term_months > 0 => {
                months.max(exact::whole(i64::from(lease.term_months)) * exact::ratio(5, 4).unwrap())
            }
            Some(_) => return SourceFact::Unavailable,
            None => months,
        };
        SourceFact::Reported(months * exact::whole(2))
    }
}

pub(crate) struct Recovery {
    pub amount: i64,
    pub quarter_months: i64,
    pub method: &'static str,
    pub convention: AssetConvention,
    pub unavailable: Option<&'static str>,
    pub exact_amount: Option<ratio::exact::Exact>,
}
pub(crate) const FACTS: &str = "ASSET_NONMACRS_SOURCE_FACTS";
pub(crate) const HISTORY: &str = "ASSET_NONMACRS_ORIGINAL_HISTORY";

impl BusinessAsset {
    pub(crate) fn vehicle(&self) -> Option<(i64, i64, i64, bool)> {
        match self.listed {
            ListedProperty::No { .. } => None,
            ListedProperty::Vehicle {
                total_miles, business_miles, operating_costs, kind, ..
            } => Some((
                i64::from(total_miles),
                i64::from(business_miles),
                operating_costs,
                kind.passenger_automobile(),
            )),
        }
    }
    pub(crate) fn business_share_of(&self, amount: i64) -> i64 {
        self.business_share(amount)
    }
    pub(crate) fn is_vehicle(&self) -> bool {
        self.vehicle().is_some()
    }
    // Pub946, Other Property Used for Transportation / Excepted vehicles.
    // Physical vehicle identity and its exact-mile operating allocation remain
    // unchanged when the qualified nonpersonal exception removes listed rules.
    pub(crate) fn is_listed_vehicle(&self) -> bool {
        matches!(self.listed, ListedProperty::Vehicle { kind, .. }
            if kind != VehicleKind::QualifiedNonpersonal)
    }
    // Share of an amount attributable to business use: exact business miles
    // over all miles for a vehicle, the entered percentage otherwise.
    pub(crate) fn business_share(&self, amount: i64) -> i64 {
        match self.listed {
            ListedProperty::Vehicle { total_miles, business_miles, .. } => {
                crate::kernel::ratio::fraction(
                    amount,
                    i64::from(business_miles),
                    i64::from(total_miles),
                )
            }
            ListedProperty::No { business_percent } => {
                crate::kernel::ratio::fraction(amount, i64::from(business_percent), 100)
            }
        }
    }
    // More than 50% qualified business use (Pub. 946 chapter 5); a vehicle's
    // test uses exact miles.
    pub(crate) fn predominant(&self) -> bool {
        match self.listed {
            ListedProperty::Vehicle { total_miles, business_miles, .. } => {
                u64::from(business_miles) * 2 > u64::from(total_miles)
            }
            ListedProperty::No { business_percent } => business_percent > 50,
        }
    }
    // Listed property used 50% or less must use ADS straight line (5 years for
    // automobiles and light trucks), with no section 179 and no bonus.
    pub(crate) fn ads(&self) -> bool {
        self.is_listed_vehicle() && !self.predominant()
    }
    pub(crate) fn effective_method(&self) -> EquipmentMethod {
        if self.ads() { EquipmentMethod::StraightLine } else { self.method }
    }

    // The whole-vehicle loan interest as entered, for the rules that forbid it.
    pub(crate) fn vehicle_loan_interest_entered(&self) -> i64 {
        match self.listed {
            ListedProperty::No { .. } => 0,
            ListedProperty::Vehicle { loan_interest, .. } => loan_interest,
        }
    }
    // Business-use percentage in hundredths of a percent, for Part V column (c).
    pub(crate) fn business_use_hundredths(&self) -> i64 {
        self.business_share(10000)
    }
    // The section 280F cap on section 179, bonus and MACRS together, applied to
    // the business-use share; None when no cap applies.
    pub(crate) fn automobile_limit(&self) -> Option<i64> {
        let (_, _, _, passenger) = self.vehicle()?;
        if !passenger {
            return None;
        }
        if let Some(history) = self.history_details.as_ref().and_then(|h| h.auto_history.as_ref()) {
            return crate::auto_recovery::caps::original_auto_limit_cents(
                self.acquired,
                self.placed,
                2026,
                &history.car_or_truck,
                self.raw_bonus_amount() > 0,
            )
            .map(|amount| self.business_share(amount));
        }
        let year = 2026_u32.saturating_sub(self.placed / 10000);
        let index = match year {
            0 if self.raw_bonus_amount() > 0 => 0,
            0 => 1,
            1 => 2,
            2 => 3,
            _ => 4,
        };
        let table = match self.placed / 10000 {
            2025 => &crate::year_constants::PASSENGER_AUTOMOBILE_LIMITS_2025_CENTS,
            _ => &crate::year_constants::PASSENGER_AUTOMOBILE_LIMITS_2026_CENTS,
        };
        Some(self.business_share(table[index]))
    }
    // Room under the section 280F cap for bonus and MACRS after the elected
    // section 179 cost, which the cap includes.
    pub(crate) fn limit_room(&self) -> i64 {
        self.automobile_limit().map_or(i64::MAX, |cap| phase::floor(cap - self.elected179(), 0))
    }
    fn suv179_limit(&self) -> Option<i64> {
        match self.listed {
            ListedProperty::Vehicle { kind, .. } => kind.section179_limit(),
            ListedProperty::No { .. } => None,
        }
    }
    pub(crate) fn vehicle_valid(&self) -> bool {
        match self.listed {
            ListedProperty::No { business_percent } => (1..=100).contains(&business_percent),
            ListedProperty::Vehicle {
                total_miles,
                business_miles,
                operating_costs,
                loan_interest,
                ..
            } => {
                (1..=2000000).contains(&total_miles)
                    && (1..=total_miles).contains(&business_miles)
                    && (0..=100000000000).contains(&operating_costs)
                    && (0..=100000000000).contains(&loan_interest)
            }
        }
    }
    pub(crate) fn valid(&self) -> bool {
        if self.nonmacrs() {
            return self.nonmacrs_valid();
        }
        self.macrs_source_fact_issue().is_none()
            && self.id >= 0
            && (1..=100000000000).contains(&self.cost)
            && (0..=self.cost).contains(&self.credit_reduction)
            // Validate the exact use fraction before any basis calculations.
            && self.vehicle_valid()
            && self.investment_credit.valid()
            && self.investment_shape_ok()
            && self.amt_section179_carry_fact_valid()
            && crate::auto_recovery::valid_amt_records(self)
            && (self.history_details.as_ref().is_none_or(|h| h.auto_history.is_none())
                || self.documented_179_reserve()
                || crate::auto_recovery::amount(self, 2026).is_some())
            && crate::dates::valid_birth(self.acquired)
            && crate::dates::valid_birth(self.placed)
            && self.disposal.valid()
            // The earlier use matters only for a continuing asset whose use
            // fell to 50% or less this year.
            && self.earlier_business_percent.is_none_or(|percent| {
                (51..=100).contains(&percent)
                    && matches!(self.history, EquipmentHistory::Continuing { .. })
                    && !self.predominant()
            })
            && self.partial.as_ref().is_none_or(|part| {
                self.class == EquipmentClass::ThirtyNine
                    && (20260101..=20261231).contains(&part.date)
                    && crate::dates::source_valid_date(part.date)
                    && part.date > self.placed
                    && (1..self.cost).contains(&part.cost_of_part)
                    && (0..=100000000000).contains(&part.proceeds)
            })
            && match self.history {
                EquipmentHistory::New { section179, .. } => {
                    (0..=phase::cap(self.business_basis(), [self.suv179_limit().unwrap_or(i64::MAX)]))
                        .contains(&section179.elected())
                }
                EquipmentHistory::Continuing {
                    prior_depreciation,
                    initial_section179,
                    carryforward,
                    schedule,
                    ..
                } => {
                    self.amt_projection().is_none_or(|a| a.valid())
                        && (0..=self.history_basis()).contains(&prior_depreciation)
                        && (0..=self.history_basis() - prior_depreciation).contains(&carryforward)
                        && (carryforward..=prior_depreciation + carryforward)
                            .contains(&initial_section179)
                        && match schedule {
                            EquipmentSchedule::Manual => true,
                            EquipmentSchedule::Tables { initial_bonus } => {
                                (0..=prior_depreciation + carryforward - initial_section179)
                                    .contains(&initial_bonus)
                            }
                        }
                }
            }
    }
    pub(crate) fn elected179(&self) -> i64 {
        match self.history {
            EquipmentHistory::New { section179, .. } => section179.elected(),
            _ => 0,
        }
    }
    pub(crate) fn available179(&self) -> i64 {
        match self.history {
            // The unused carryforward of an asset disposed of in 2026 is added
            // back to its basis and is not available as a deduction to the
            // transferor (Reg. 1.179-3(f)(1)); one kept for personal use is
            // no longer available for expensing (Reg. 1.179-3(f)(2)).
            EquipmentHistory::Continuing { .. } if self.disposal != crate::AssetDisposal::No => 0,
            EquipmentHistory::Continuing { carryforward, .. } => carryforward,
            _ => self.elected179(),
        }
    }
    pub(crate) fn documented_179_reserve(&self) -> bool {
        // This predicate is also used before native source admission. Establish
        // safe basis operands without recursing through valid().
        if !self.vehicle_valid()
            || self.cost < 0
            || !(0..=self.cost).contains(&self.credit_reduction)
            || !self.investment_shape_ok()
            || self.earlier_business_percent.is_some()
        {
            return false;
        }
        let EquipmentHistory::Continuing {
            initial_section179,
            carryforward,
            prior_depreciation,
            ..
        } = self.history
        else {
            return false;
        };
        let Some(h) = &self.history_details else {
            return false;
        };
        let Some(auto) = &h.auto_history else {
            return false;
        };
        let Some((total, business, _, true)) = self.vehicle() else {
            return false;
        };
        if initial_section179 < 0
            || prior_depreciation < 0
            || carryforward <= 0
            || prior_depreciation.checked_add(carryforward) != Some(self.history_basis())
        {
            return false;
        }
        let Some(original_cap) = crate::auto_recovery::caps::original_auto_limit_cents(
            self.acquired,
            self.placed,
            self.placed / 10000,
            &auto.car_or_truck,
            false,
        ) else {
            return false;
        };
        let same_use = |fraction: crate::UseFraction| {
            fraction.total > 0
                && fraction.used <= fraction.total
                && i64::from(fraction.used) * total == business * i64::from(fraction.total)
        };
        self.placed < 20260101
            && self.acquired <= self.placed
            && self.automobile_limit().is_some()
            && self.predominant()
            && carryforward > 0
            && self.opening_basis() == 0
            && h.original_business_basis == self.history_basis()
            && h.original_section179_elected == initial_section179
            && initial_section179 == h.original_business_basis
            && h.original_bonus == 0
            && initial_section179 <= self.business_share(original_cap)
            && auto.first_year_capped_deduction >= 0
            && auto.first_year_capped_deduction
                == h.years
                    .iter()
                    .find(|year| year.year == self.placed / 10000)
                    .map_or(0, |year| year.section179_allowed)
            && auto.post_recovery_basis == 0
            && auto.first_year_bonus != crate::OriginalBonusTreatment::HundredPercent
            && auto.safe_harbor_adopted_year.is_none()
            && same_use(h.original_income_use)
            && same_use(h.original_qualified_use)
            && h.years.len() == (2026 - self.placed / 10000) as usize
            && h.years.iter().all(|year| {
                (self.placed / 10000..2026).contains(&year.year)
                    && h.years
                        .iter()
                        .filter(|other| other.year == year.year)
                        .count()
                        == 1
                    && same_use(year.income_use)
                    && same_use(year.qualified_use)
                    && year.section179_allowed >= 0
                    && year.section179_allowed <= initial_section179
                    && year.bonus == 0
                    && year.regular == 0
                    && year.amt == 0
                    && year.basis_increase == 0
                    && year.basis_decrease == 0
                    && year.recaptured == 0
            })
            && h.years
                .iter()
                .try_fold(0i64, |sum, row| sum.checked_add(row.section179_allowed))
                == Some(prior_depreciation)
            && h.carryovers
                .iter()
                .all(|carry| carry.year == self.placed / 10000 && carry.amount >= 0)
            // This source is one asset's original election, not a pooled history.
            && h.carryovers.len() == 1
            && h.carryovers[0].amount == carryforward
    }
    // The depreciable basis: the business share of the cost less the credit
    // reduction. A property carrying an investment credit (Form 3468) has the
    // section 50(c) reduction the App computes (100% of the rehabilitation
    // credit, 50% of an energy credit), which replaces the entered amount;
    // any other credit's reduction is the entered one.
    pub(crate) fn business_basis(&self) -> i64 {
        self.business_basis_on_cost(self.cost)
    }
    pub(crate) fn business_basis_on_cost(&self, cost: i64) -> i64 {
        if self.investment_credit.is_some() {
            return self.credit_cost_on(cost) - self.investment_basis_reduction_on(cost);
        }
        self.business_share(cost - self.credit_reduction)
    }
    pub(crate) fn bonus_eligible(&self) -> bool {
        self.bonus_eligible_with_election(None)
    }
    fn bonus_eligible_with_election(&self, election: Option<&crate::AdsElection>) -> bool {
        self.bonus_eligible_in_context(AdsInvocation { election, ..Default::default() })
    }
    pub(crate) fn bonus_eligible_in_context(&self, context: AdsInvocation<'_>) -> bool {
        if let Some(route) = self.nonmacrs_route() {
            return matches!(route, crate::NonMacrsRecovery::Software { facts } if facts.bonus_eligible && facts.exempt_lease.is_none() && matches!(facts.classification, crate::SoftwareKind::Standalone | crate::SoftwareKind::PublicNonexclusive))
                && self.acquired >= 20170928;
        }
        self.acquired >= 20170928
            && self
                .source_ads_recovery_in_context(context)
                .is_ok_and(|source| source.is_none_or(|ads| !ads.required()))
    }

    pub(crate) fn amt_section179_carry(&self) -> Result<i64, ()> {
        let derived = self.derived_amt_section179_carry();
        match (derived, self.amt_section179_income_carry) {
            (Ok(actual), Some(reported)) if actual != reported => Err(()),
            (Ok(actual), _) => Ok(actual),
            (Err(()), Some(reported)) if self.amt_section179_carry_fact_valid() => Ok(reported),
            _ => Err(()),
        }
    }

    pub(crate) fn amt_section179_carry_fact_valid(&self) -> bool {
        let Some(reported) = self.amt_section179_income_carry else {
            return true;
        };
        let EquipmentHistory::Continuing { initial_section179, .. } = self.history else {
            return false;
        };
        if initial_section179 <= 0 || !(0..=initial_section179).contains(&reported) {
            return false;
        }
        match self.derived_amt_section179_carry() {
            Ok(actual) => actual == reported,
            Err(()) => {
                // A complete unchanged-use separate schedule must reconcile;
                // a manual balance cannot override contradictory actual rows.
                let complete = self.history_details.as_ref().is_some_and(|records| {
                    let year = self.placed / 10000;
                    // Match the post-1998 GDS/AMT source window in qualifies().
                    (1999..2026).contains(&year)
                        && records.years.len() == (2026 - year) as usize
                        && (year..2026).all(|year| {
                            records.years.iter().filter(|row| row.year == year).count() == 1
                        })
                        && records.auto_history.is_none()
                        && self.earlier_business_percent.is_none()
                        && records.years.iter().all(|row| {
                            row.basis_increase == 0
                                && row.basis_decrease == 0
                                && row.recaptured == 0
                        })
                });
                !complete
                    || matches!(
                        self.history,
                        EquipmentHistory::Continuing { amt: EquipmentAmtHistory::Same, .. }
                    )
            }
        }
    }

    pub(crate) fn derived_amt_section179_carry(&self) -> Result<i64, ()> {
        if self.history_details.as_ref().is_some_and(|records| records.auto_history.is_some()) {
            return crate::auto_recovery::amt_history(self).map(|(carry, _)| carry).ok_or(());
        }
        // This is also read while an ordinary editor's source facts change.
        // Reject an invalid fraction before evaluating its business basis.
        if !self.vehicle_valid()
            || self.cost < 0
            || !(0..=self.cost).contains(&self.credit_reduction)
        {
            return Err(());
        }
        let EquipmentHistory::Continuing {
            initial_section179,
            carryforward,
            prior_depreciation,
            amt,
            ..
        } = self.history
        else {
            return Ok(0);
        };
        if initial_section179 < 0
            || !(0..=initial_section179).contains(&carryforward)
            || prior_depreciation < 0
        {
            return Err(());
        }
        if initial_section179 == 0 {
            return Ok(0);
        }
        let EquipmentAmtHistory::Separate { prior_depreciation: alternative_prior, .. } = amt
        else {
            // The same MACRS method and other depreciation do not identify
            // independently income-limited AMT section179 allowances.
            return Err(());
        };
        if alternative_prior < 0 {
            return Err(());
        }
        let records = self.history_details.as_ref().ok_or(())?;
        let first_year = self.placed / 10000;
        // Match the post-1998 GDS/AMT source window in qualifies().
        if !(1999..2026).contains(&first_year)
            || records.years.len() != (2026 - first_year) as usize
            || records.original_section179_elected != initial_section179
            || records.original_business_basis != self.history_basis()
            || self.earlier_business_percent.is_some()
            || records.auto_history.is_some()
        {
            return Err(());
        }
        let (used, total) =
            self.vehicle().map(|(total, used, _, _)| (used, total)).unwrap_or_else(|| {
                match self.listed {
                    ListedProperty::No { business_percent } => (i64::from(business_percent), 100),
                    _ => unreachable!(),
                }
            });
        let same_use = |fraction: crate::UseFraction| {
            fraction.total > 0
                && fraction.used <= fraction.total
                && i64::from(fraction.used) * total == used * i64::from(fraction.total)
        };
        if !same_use(records.original_income_use) || !same_use(records.original_qualified_use) {
            return Err(());
        }
        let (mut regular_179, mut bonus, mut regular_other, mut alternative_other) =
            (0i64, 0i64, 0i64, 0i64);
        for year in first_year..2026 {
            let mut rows = records.years.iter().filter(|row| row.year == year);
            let row = rows.next().ok_or(())?;
            if rows.next().is_some()
                || !same_use(row.income_use)
                || !same_use(row.qualified_use)
                || [row.section179_allowed, row.bonus, row.regular, row.amt]
                    .iter()
                    .any(|value| *value < 0)
                || row.basis_increase != 0
                || row.basis_decrease != 0
                || row.recaptured != 0
            {
                return Err(());
            }
            regular_179 = regular_179.checked_add(row.section179_allowed).ok_or(())?;
            bonus = bonus.checked_add(row.bonus).ok_or(())?;
            regular_other = regular_other.checked_add(row.regular).ok_or(())?;
            alternative_other = alternative_other.checked_add(row.amt).ok_or(())?;
        }
        if regular_179.checked_add(carryforward).ok_or(())? != initial_section179
            || regular_179.checked_add(bonus).and_then(|v| v.checked_add(regular_other)).ok_or(())?
                != prior_depreciation
            || bonus != records.original_bonus
        {
            return Err(());
        }
        let alternative_179 = alternative_prior
            .checked_sub(bonus)
            .and_then(|v| v.checked_sub(alternative_other))
            .ok_or(())?;
        if !(0..=initial_section179).contains(&alternative_179) {
            return Err(());
        }
        Ok(crate::kernel::carry::native_balance(initial_section179, alternative_179))
    }

    pub(crate) fn amt_projection(&self) -> Option<Self> {
        if self.nonmacrs() {
            return None;
        }
        if self.history_details.as_ref().is_some_and(|records| records.auto_history.is_some()) {
            return crate::auto_recovery::amt_projection(self)
                .filter(|projected| projected != self);
        }
        if matches!(self.history, EquipmentHistory::New { .. }) {
            if !self.bonus_eligible()
                && self.method == EquipmentMethod::Regular
                && self.class.years() <= 10
            {
                // A 2026 first-use asset has no prior depreciation to enter.
                // Section 179 reduces both bases equally, and an acquisition
                // before September 28, 2017 cannot receive bonus in 2026.
                let mut asset = self.clone();
                asset.method = EquipmentMethod::Declining150;
                return Some(asset);
            }
            return None;
        }
        let EquipmentHistory::Continuing {
            prior_depreciation,
            carryforward: regular_carry,
            schedule,
            amt,
            ..
        } = self.history
        else {
            return None;
        };
        let actual_carry = self.amt_section179_carry().ok();
        let (alternative_prior, alternative_schedule, separate) = match amt {
            EquipmentAmtHistory::Separate { prior_depreciation, schedule } => {
                (prior_depreciation, schedule, true)
            }
            EquipmentAmtHistory::Same => {
                let carry = actual_carry?;
                if carry == regular_carry {
                    return None;
                }
                // The other MACRS allowances match. Replace only the actual
                // prior179 portion, preserving the original elected basis.
                (
                    prior_depreciation.checked_add(regular_carry)?.checked_sub(carry)?,
                    schedule,
                    false,
                )
            }
        };
        let mut asset = self.clone();
        asset.amt_section179_income_carry = None;
        if separate {
            asset.method = EquipmentMethod::Declining150;
        }
        if let EquipmentHistory::Continuing {
            prior_depreciation: prior,
            schedule: approach,
            carryforward,
            amt,
            ..
        } = &mut asset.history
        {
            *prior = alternative_prior;
            *approach = alternative_schedule;
            if let Some(actual) = actual_carry {
                *carryforward = actual;
            }
            *amt = EquipmentAmtHistory::Same;
        }
        Some(asset)
    }
    pub(crate) fn table_rates(&self, mid_quarter: bool) -> &'static [i64] {
        let method = match self.effective_method() {
            EquipmentMethod::Regular => 0,
            EquipmentMethod::Declining150 => 1,
            EquipmentMethod::StraightLine => 2,
        };
        let convention = if mid_quarter {
            1 + (self.placed / 100 % 100).saturating_sub(1).min(11) as usize / 3
        } else {
            0
        };
        let class = match self.class {
            EquipmentClass::Three => 0,
            EquipmentClass::Five => 1,
            EquipmentClass::Seven => 2,
            EquipmentClass::Ten => 3,
            EquipmentClass::Fifteen => 4,
            EquipmentClass::Twenty => 5,
            EquipmentClass::ThirtyNine | EquipmentClass::NonMacrs { .. } => {
                unreachable!("non-GDS property has its own schedule, not GDS table rates")
            }
        };
        crate::kernel::cost_recovery::macrs_tables::RATES[method * 30 + convention * 6 + class]
    }
    // The basis the earlier years' depreciation was figured on: the business
    // share at the earlier business use for nonlisted property whose use
    // fell to 50% or less this year, else this year's business basis.
    pub(crate) fn history_basis(&self) -> i64 {
        match self.earlier_business_percent {
            Some(percent) if !self.is_listed_vehicle() && !self.investment_credit.is_some() => {
                crate::kernel::ratio::fraction(
                    self.cost - self.credit_reduction,
                    i64::from(percent),
                    100,
                )
            }
            _ => self.business_basis(),
        }
    }
    pub(crate) fn opening_basis(&self) -> i64 {
        self.history_basis()
            - match self.history {
                EquipmentHistory::New { section179, .. } => section179.elected(),
                EquipmentHistory::Continuing { prior_depreciation, carryforward, .. } => {
                    prior_depreciation + carryforward
                }
            }
    }
    // Bonus and MACRS after the section 280F cap, bonus first.
    pub(crate) fn bonus_amount(&self) -> i64 {
        self.source_bonus_amount(None)
    }
    pub(crate) fn source_bonus_amount(&self, election: Option<&crate::AdsElection>) -> i64 {
        phase::cap(
            self.raw_bonus_on_with_election(self.opening_basis(), election),
            [self.limit_room()],
        )
    }
    // The tentative amounts the section 280F cap limits; later-year basis
    // follows these, not the capped deduction.
    pub(crate) fn raw_bonus_amount(&self) -> i64 {
        self.raw_bonus_on(self.opening_basis())
    }
    pub(crate) fn raw_bonus_on(&self, opening_basis: i64) -> i64 {
        self.raw_bonus_on_with_election(opening_basis, None)
    }
    fn raw_bonus_on_with_election(
        &self,
        opening_basis: i64,
        election: Option<&crate::AdsElection>,
    ) -> i64 {
        if self.nonmacrs() {
            return if self.bonus_eligible_with_election(election)
                && matches!(self.history, EquipmentHistory::New { bonus: BonusElection::Apply, .. })
                && !self.placed_and_disposed_in_current_year()
            {
                crate::kernel::ratio::fraction(
                    opening_basis,
                    if self.acquired >= 20250120 { 100 } else { 20 },
                    100,
                )
            } else {
                0
            };
        }
        self.raw_bonus_on_in_context(
            opening_basis,
            AdsInvocation { election, ..Default::default() },
        )
    }
    pub(crate) fn raw_bonus_on_in_context(
        &self,
        opening_basis: i64,
        context: AdsInvocation<'_>,
    ) -> i64 {
        if !self.bonus_eligible_in_context(context)
            || self.placed_and_disposed_in_current_year()
            || self.ads()
            || (self.class == EquipmentClass::ThirtyNine
                && self.qualified_improvement(context) != Ok(true))
            || !matches!(self.history, EquipmentHistory::New { bonus: BonusElection::Apply, .. })
        {
            return 0;
        }
        crate::kernel::ratio::fraction(
            opening_basis,
            if self.acquired >= 20250120 { 100 } else { 20 },
            100,
        )
    }
    // Property placed in service and disposed of in 2026 (sold, scrapped or
    // converted to personal use) takes no MACRS or bonus depreciation: no
    // depreciation deduction is allowed for it (Reg. 1.168(d)-1(b)(3)(ii);
    // Pub 946 ch. 4; i4562 "Special depreciation allowance"). The section 179
    // election stays (Reg. 1.179-1(e)(3)); a disposition recaptures it as
    // section 1245 gain.
    pub(crate) fn placed_and_disposed_in_current_year(&self) -> bool {
        matches!(self.history, EquipmentHistory::New { .. }) && self.disposal_date().is_some()
    }
}

impl BusinessAsset {
    pub(crate) fn macrs_source_fact_issue(&self) -> Option<crate::validation::ValidationIssue> {
        let source = self.macrs_source.as_ref()?;
        use crate::validation::ValidationIssue;
        let issue = |field| ValidationIssue::new("ASSET_MACRS_SOURCE_FACTS").at_field(field);
        if self.nonmacrs() {
            return Some(issue("source"));
        }
        let year_valid = |year: u32| (1987..=9999).contains(&year);
        let placed_year = self.placed / 10000;
        if source.election.as_ref().is_some_and(|election| {
            !year_valid(election.tax_year) || election.tax_year != placed_year
        }) {
            return Some(issue("election"));
        }
        if let Some(rows) = &source.restrictions {
            for (index, row) in rows.iter().enumerate() {
                if !year_valid(row.first_year)
                    || row.last_year.is_some_and(|end| !year_valid(end) || end < row.first_year)
                {
                    return Some(
                        ValidationIssue::new("ASSET_MACRS_SOURCE_FACTS")
                            .at_field("first_year")
                            .in_list("restrictions", index),
                    );
                }
            }
        }
        if let Some(rows) = &source.foreign_years {
            for (index, row) in rows.iter().enumerate() {
                if !year_valid(row.year)
                    || row.year < placed_year
                    || row.outside_us.total == 0
                    || row.outside_us.used > row.outside_us.total
                    || rows[..index].iter().any(|other| other.year == row.year)
                {
                    return Some(
                        ValidationIssue::new("ASSET_MACRS_SOURCE_FACTS")
                            .at_field("year")
                            .in_list("foreign_years", index),
                    );
                }
            }
        }
        if source.class_life.as_ref().is_some_and(|life| {
            life.months == 0
                || life.table_asset_class.trim().is_empty()
                || (life.table_asset_class.trim() == "00.12" && life.months != 72)
        }) {
            return Some(issue("class_life"));
        }
        if source.lease.as_ref().is_some_and(|lease| lease.term_months == 0) {
            return Some(issue("lease"));
        }
        if source.improvement.as_ref().is_some_and(|improvement| improvement.building_id <= 0) {
            return Some(issue("improvement"));
        }
        if let Some(OriginalMacrsSystem::Ads { began_year }) = source.original_system
            && (!year_valid(began_year)
                || began_year < placed_year
                || began_year > crate::year_constants::TAX_YEAR)
        {
            return Some(issue("original_system"));
        }
        if matches!(self.history, crate::EquipmentHistory::New { .. })
            && self.elected179() > 0
            && self.source_ads_recovery().is_ok_and(|ads| {
                ads.is_some_and(|ads| {
                    ads.reasons.iter().any(|reason| {
                        matches!(reason, AdsReason::TaxExemptUse | AdsReason::ForeignUse)
                    })
                })
            })
        {
            return Some(issue("election"));
        }
        None
    }

    // IRC168(e)(6): the taxpayer's section1250 interior work must follow
    // original building service. A before-date record proves only its bound.
    // https://www.irs.gov/irb/2020-19_IRB
    pub(crate) fn qualified_improvement(
        &self,
        context: AdsInvocation<'_>,
    ) -> Result<bool, &'static str> {
        let Some(improvement) = self.macrs_source.as_ref().and_then(|s| s.improvement.as_ref())
        else {
            return Ok(false);
        };
        if self.class != crate::EquipmentClass::ThirtyNine
            || !matches!(
                improvement.kind,
                RealImprovementKind::Interior1250 { made_by_taxpayer: true }
            )
            || self.placed < 20180101
        {
            return Ok(false);
        }
        let building = context.building.ok_or("ASSET_QIP_SOURCE_CONTEXT_UNAVAILABLE")?;
        if !building.valid() {
            return Err("ASSET_QIP_SOURCE_CONTEXT_UNAVAILABLE");
        }
        if building.kind != RecoveryBuildingKind::Nonresidential {
            return Ok(false);
        }
        match building.first_service {
            Some(BuildingFirstService::ExactDate { date }) => Ok(date < self.placed),
            Some(BuildingFirstService::BeforeDate { date }) if date <= self.placed => Ok(true),
            _ => Err("ASSET_QIP_SOURCE_CONTEXT_UNAVAILABLE"),
        }
    }
    pub(crate) fn potential_real179(&self) -> bool {
        self.class == crate::EquipmentClass::ThirtyNine
            && self.placed >= 20180101
            && self.macrs_source.as_ref().and_then(|s| s.improvement.as_ref()).is_some_and(|row| {
                matches!(
                    row.kind,
                    RealImprovementKind::Interior1250 { made_by_taxpayer: true }
                        | RealImprovementKind::Roof
                        | RealImprovementKind::Hvac
                        | RealImprovementKind::FireProtectionOrAlarm
                        | RealImprovementKind::Security
                )
            })
    }
    pub(crate) fn real179_in_context(&self, context: AdsInvocation<'_>) -> bool {
        if self.qualified_improvement(context) == Ok(true) {
            return true;
        }
        let Some(building) = context.building else { return false };
        self.potential_real179()
            && building.valid()
            && building.kind == RecoveryBuildingKind::Nonresidential
            && match building.first_service {
                Some(BuildingFirstService::ExactDate { date }) => date < self.placed,
                Some(BuildingFirstService::BeforeDate { date }) => date <= self.placed,
                None => false,
            }
            && self.macrs_source.as_ref().and_then(|s| s.improvement.as_ref()).is_some_and(|row| {
                matches!(
                    row.kind,
                    RealImprovementKind::Roof
                        | RealImprovementKind::Hvac
                        | RealImprovementKind::FireProtectionOrAlarm
                        | RealImprovementKind::Security
                )
            })
    }
    pub(crate) fn source_ads_recovery(&self) -> Result<Option<AdsRecovery>, &'static str> {
        self.source_ads_recovery_with_election(None)
    }
    pub(crate) fn source_ads_recovery_with_election(
        &self,
        election: Option<&AdsElection>,
    ) -> Result<Option<AdsRecovery>, &'static str> {
        self.source_ads_recovery_in_context(AdsInvocation {
            election,
            required: None,
            unavailable: false,
            building: None,
            outside_us: None,
        })
    }
    pub(crate) fn source_ads_recovery_in_context(
        &self,
        context: AdsInvocation<'_>,
    ) -> Result<Option<AdsRecovery>, &'static str> {
        let election = context.election;
        if context.unavailable {
            return Err("ASSET_ADS_SOURCE_UNAVAILABLE");
        }
        let Some(source) = &self.macrs_source else {
            return if election.is_some() || context.required.is_some() {
                Err("ASSET_ADS_SOURCE_UNAVAILABLE")
            } else {
                Ok(None)
            };
        };
        let qip = self.qualified_improvement(context)?;
        let reasons = source.ads_reasons(
            crate::year_constants::TAX_YEAR,
            AdsOwnerFacts {
                outside_us: context
                    .outside_us
                    .map_or(SourceFact::Unavailable, SourceFact::Reported),
                listed_use_requires_ads: self.ads(),
                real_property_trade_requires_ads: false,
                farming_trade_requires_ads: false,
                capitalization_election_requires_ads: false,
            },
        );
        let reasons = match (reasons, context.required) {
            (SourceFact::Reported(mut reasons), Some(required)) => {
                reasons.push(required);
                SourceFact::Reported(reasons)
            }
            (_, Some(required)) => SourceFact::Reported(alloc::vec![required]),
            (other, None) => other,
        };
        let reasons = match reasons {
            SourceFact::NotReported if election.is_some() => {
                SourceFact::Reported(alloc::vec![AdsReason::VoluntaryElection])
            }
            SourceFact::Reported(mut reasons) if election.is_some() => {
                if !reasons.contains(&AdsReason::VoluntaryElection) {
                    reasons.push(AdsReason::VoluntaryElection);
                }
                SourceFact::Reported(reasons)
            }
            other => other,
        };
        let reasons = match reasons {
            SourceFact::Unavailable => return Err("ASSET_ADS_SOURCE_UNAVAILABLE"),
            SourceFact::NotReported => {
                if matches!(source.original_system, Some(OriginalMacrsSystem::Ads { .. })) {
                    return Err("ASSET_ADS_ORIGINAL_HISTORY_UNAVAILABLE");
                }
                if !qip && !self.real179_in_context(context) {
                    return Ok(None);
                }
                if matches!(self.history, crate::EquipmentHistory::Continuing { .. })
                    && source.original_system.is_none()
                {
                    return Err("ASSET_ADS_ORIGINAL_HISTORY_UNAVAILABLE");
                }
                return Ok(Some(AdsRecovery {
                    reasons: Vec::new(),
                    period_half_months: exact::whole(if qip { 360 } else { 936 }),
                    qip,
                }));
            }
            SourceFact::Reported(reasons) => reasons,
        };
        if matches!(self.history, crate::EquipmentHistory::Continuing { .. })
            && source.original_system.is_none()
            && source.election.is_none()
            && election.is_none()
        {
            return Err("ASSET_ADS_ORIGINAL_HISTORY_UNAVAILABLE");
        }
        // A change from GDS needs its actual conversion recovery; an original
        // ADS clock cannot establish that historical basis or remaining life.
        if matches!(self.history, crate::EquipmentHistory::Continuing { .. })
            && match source.original_system {
                Some(OriginalMacrsSystem::Gds) => true,
                Some(OriginalMacrsSystem::Ads { began_year }) => began_year != self.placed / 10000,
                None => false,
            }
        {
            return Err("ASSET_ADS_ORIGINAL_HISTORY_UNAVAILABLE");
        }
        let statutory_months = if qip {
            Some(240)
        } else if self.class == crate::EquipmentClass::ThirtyNine {
            Some(480)
        } else if self.is_listed_vehicle() && self.class == crate::EquipmentClass::Five {
            Some(60)
        } else {
            None
        };
        let SourceFact::Reported(period_half_months) = source.ads_period(statutory_months) else {
            return Err("ASSET_ADS_CLASS_LIFE_UNAVAILABLE");
        };
        Ok(Some(AdsRecovery { reasons, period_half_months, qip }))
    }
}

impl BusinessAsset {
    pub(crate) fn nonmacrs_route(&self) -> Option<&NonMacrsRecovery> {
        match &self.class {
            crate::EquipmentClass::NonMacrs { recovery } => Some(recovery),
            _ => None,
        }
    }
    pub(crate) fn nonmacrs(&self) -> bool {
        self.nonmacrs_route().is_some()
    }

    // Err is a named unavailable source, never permission to use ordinary MACRS.
    pub(crate) fn nonmacrs_recovery(&self, bonus: i64) -> Result<Recovery, &'static str> {
        if !crate::dates::source_valid_date(self.placed)
            || !crate::dates::source_valid_date(self.acquired)
            || self.acquired > self.placed
            || self.placed > 20261231
            || self.is_vehicle()
            || self.partial.is_some()
        {
            return Err(FACTS);
        }
        if matches!(self.history, EquipmentHistory::New { .. }) != (self.placed / 10000 == 2026)
            || matches!(
                self.history,
                EquipmentHistory::Continuing {
                    amt: crate::EquipmentAmtHistory::Separate { .. },
                    ..
                }
            )
        {
            return Err(HISTORY);
        }
        if let Some(records) = &self.history_details
            && (records.years.iter().any(|row| {
                row.basis_increase != 0
                    || row.basis_decrease != 0
                    || row.recaptured != 0
                    || row.regular != row.amt
            }) || records.auto_history.is_some())
        {
            return Err(HISTORY);
        }
        let remaining = self.opening_basis();
        if remaining < 0 || bonus < 0 || bonus > remaining {
            return Err(FACTS);
        }
        let mut result = Recovery {
            amount: 0,
            quarter_months: 0,
            method: "S/L",
            convention: AssetConvention::ActualMonths,
            unavailable: None,
            exact_amount: None,
        };
        let months = self.nonmacrs_months();
        match &self.nonmacrs_route().ok_or(FACTS)? {
            NonMacrsRecovery::Units { facts } => {
                if !facts.elected_originally
                    || facts.remaining_units == 0
                    || facts.current_units > facts.remaining_units
                    || facts.remaining_basis != remaining
                    || !(0..=remaining).contains(&facts.salvage)
                    || bonus != 0
                    || self.available179() != 0
                {
                    return Err(FACTS);
                }
                result.method = "Units of production";
                result.amount = ratio::fraction(
                    remaining - facts.salvage,
                    i64::from(facts.current_units),
                    i64::from(facts.remaining_units),
                );
            }
            NonMacrsRecovery::Software { facts } => {
                if !matches!(
                    facts.classification,
                    SoftwareKind::Standalone | SoftwareKind::PublicNonexclusive
                ) {
                    return Err("ASSET_SOFTWARE_OTHER_OWNER");
                }
                if facts.exempt_lease.is_some_and(|lease| {
                    lease.months == 0 || !crate::dates::source_valid_date(lease.entered)
                }) || (self.available179() != 0
                    && !self.nonmacrs_route().is_some_and(NonMacrsRecovery::section179_property))
                {
                    return Err(FACTS);
                }
                // Section167(f)(1)(C): retain exact quarter-month units for125%.
                let period = facts.exempt_lease.map_or(144, |lease| {
                    if lease.entered > 20040312 {
                        144.max(i64::from(lease.months) * 5)
                    } else {
                        144
                    }
                });
                if facts.exempt_lease.is_some_and(|lease| lease.entered > self.placed) {
                    return Err(HISTORY);
                }
                result.quarter_months = period;
                let elapsed = ((2026 - i64::from(self.placed / 10000)) * 12
                    - i64::from(self.placed / 100 % 100)
                    + 1)
                    * 4;
                if period <= elapsed && remaining != 0 {
                    return Err(HISTORY);
                }
                let exposure = phase::bound(period - elapsed, 0, months * 4);
                let (elected, original_bonus) = match self.history {
                    EquipmentHistory::New { section179, .. } => (section179.elected(), bonus),
                    EquipmentHistory::Continuing {
                        initial_section179,
                        schedule: EquipmentSchedule::Tables { initial_bonus },
                        ..
                    } => (initial_section179, initial_bonus),
                    EquipmentHistory::Continuing { initial_section179, .. } => {
                        let history = self.history_details.as_ref().ok_or(HISTORY)?;
                        (initial_section179, history.original_bonus)
                    }
                };
                let original = self.business_basis() - elected - original_bonus;
                if original < 0 {
                    return Err(FACTS);
                }
                result.amount = if period == 144 {
                    self.actual_monthly(original, remaining - bonus, 36)?
                } else {
                    ratio::checked_fraction(original, exposure, period)
                        .ok_or(HISTORY)?
                        .min(remaining - bonus)
                };
            }
            NonMacrsRecovery::FiniteLife { facts } => {
                if facts.legal_remaining_months() == 0
                    || facts.useful_months() == 0
                    || bonus != 0
                    || self.available179() != 0
                    || !(0..=self.business_basis()).contains(&facts.salvage())
                {
                    return Err(FACTS);
                }
                if matches!(
                    facts.kind(),
                    FiniteLifeKind::CreatedEligible | FiniteLifeKind::CreatedRealPropertyBenefits
                ) && self.acquired < 20031231
                {
                    return Err(HISTORY);
                }
                let statutory = match facts.kind() {
                    FiniteLifeKind::MortgageServicing => Some(108_u32),
                    FiniteLifeKind::CreatedEligible => Some(180),
                    FiniteLifeKind::CreatedRealPropertyBenefits => Some(300),
                    _ => None,
                };
                let elapsed = ((2026 - i64::from(self.placed / 10000)) * 12
                    - (i64::from(self.placed / 100 % 100) - 1))
                    .max(0) as u32;
                let period = if let Some(original) = statutory {
                    if facts.salvage() != 0 {
                        return Err(FACTS);
                    }
                    original.saturating_sub(elapsed)
                } else {
                    facts.legal_remaining_months().min(facts.useful_months())
                };
                if period == 0 && remaining != 0 {
                    return Err(HISTORY);
                }
                result.quarter_months = i64::from(period) * 4;
                let salvage = self.business_share_of(facts.salvage());
                if salvage > remaining {
                    return Err(FACTS);
                }
                if period > 0 {
                    result.amount = if let Some(original) = statutory {
                        self.actual_monthly(self.business_basis(), remaining, original)?
                    } else {
                        ratio::checked_fraction(
                            remaining - salvage,
                            months.min(i64::from(period)),
                            i64::from(period),
                        )
                        .ok_or(HISTORY)?
                    };
                }
            }
            NonMacrsRecovery::IncomeForecast { facts } => {
                let following_year = 2026 - self.placed / 10000;
                if facts.current_income < 0
                    || facts.forecast_total_income < 0
                    // Section167(g)(1)(C) owns the tenth-year deduction without
                    // an income ratio. Earlier ratios still need a denominator.
                    || (following_year < 10 && facts.forecast_total_income == 0)
                    || facts.current_income > facts.forecast_total_income
                    || facts.remaining_basis != remaining
                    || bonus != 0
                    || self.available179() != 0
                {
                    return Err(FACTS);
                }
                match facts.lookback {
                    ForecastLookback::SmallCostException if self.cost > 10_000_000 => {
                        return Err("FORECAST_SMALL_COST_EXCEPTION");
                    }
                    ForecastLookback::WithinTenPercent { actual, forecast }
                        if actual < 0
                            || forecast < 0
                            || (i128::from(actual) - i128::from(forecast)).abs() * 10
                                > i128::from(forecast) =>
                    {
                        return Err("FORECAST_NOT_WITHIN_TEN_PERCENT");
                    }
                    _ => {}
                }
                if following_year > 10 && remaining != 0 {
                    // Section167(g)(1)(C) owns the tenth following year. A later
                    // unexplained balance needs its original accounting owner.
                    return Err(HISTORY);
                }
                if matches!(
                    self.history,
                    EquipmentHistory::New { bonus: crate::BonusElection::Apply, .. }
                ) && matches!(
                    facts.kind,
                    crate::ForecastPropertyKind::FilmVideo | crate::ForecastPropertyKind::Sound
                ) {
                    // A qualifying production may have a separate168(k) source.
                    // These income-forecast facts do not establish that production's eligibility.
                    return Err(HISTORY);
                }
                result.method = "Income forecast";
                result.amount = if following_year >= 10 {
                    remaining
                } else {
                    phase::cap(
                        ratio::fraction(
                            self.business_basis(),
                            facts.current_income,
                            facts.forecast_total_income,
                        ),
                        [remaining],
                    )
                };
                // i8866: the exception uses unadjusted total capitalized cost,
                // not basis reduced by use, credits or prior depreciation.
                result.unavailable = if self.cost <= 10_000_000 {
                    None
                } else {
                    match facts.lookback {
                        ForecastLookback::WithinTenPercent { .. }
                            if facts.participations != crate::ForecastParticipations::Basis
                                || ![3, 10].contains(&following_year) =>
                        {
                            None
                        }
                        ForecastLookback::NoRecomputation if ![3, 10].contains(&following_year) => {
                            None
                        }
                        // Section167(g)(7)(C) requires each year's comparison
                        // when participations enter basis. A cumulative pair
                        // cannot establish those missing source rows.
                        _ => Some("ASSET_FORECAST_LOOKBACK_UNAVAILABLE"),
                    }
                };
            }
            NonMacrsRecovery::Historical { facts } => {
                if facts.remaining_basis != remaining || bonus != 0 || self.available179() != 0 {
                    return Err(FACTS);
                }
                if let HistoricalRecoveryKind::Acrs { period, alternate } = &facts.kind {
                    return self.acrs_recovery(facts, period, *alternate, result);
                }
                if matches!(facts.kind, HistoricalRecoveryKind::Pre1981Declining { multiplier, .. } if !crate::kernel::natural::exact(multiplier, 100).is_some_and(|multiplier| (100..=200).contains(&multiplier)))
                {
                    return Err(FACTS);
                }
                let (multiplier, months, salvage) = match facts.kind {
                    HistoricalRecoveryKind::Pre1981StraightLine { remaining_months, salvage }
                        if self.placed < 19810101 =>
                    {
                        (None, remaining_months, salvage)
                    }
                    HistoricalRecoveryKind::Pre1981Declining {
                        multiplier,
                        original_useful_months,
                        salvage,
                    } if self.placed < 19810101 => {
                        let hundredths =
                            u32::try_from(crate::kernel::natural::units(multiplier, 100));
                        (Some(hundredths.unwrap_or(0)), original_useful_months, salvage)
                    }
                    HistoricalRecoveryKind::Other167StraightLine { remaining_months, salvage } => {
                        (None, remaining_months, salvage)
                    }
                    _ => return Err(HISTORY),
                };
                if months == 0 || !(0..=remaining).contains(&salvage) {
                    return Err(FACTS);
                }
                let mut half_months = match (self.disposal_date(), &facts.original_convention) {
                    (_, AssetConvention::MidQuarter) => return Err(HISTORY),
                    (None, _) => 24,
                    (Some(_), AssetConvention::HalfYear) => 12,
                    (Some(date), AssetConvention::MidMonth) => 2 * (date / 100 % 100) - 1,
                    (Some(date), AssetConvention::ActualMonths) => 2 * (date / 100 % 100 - 1),
                };
                if self.placed / 10000 == 2026 {
                    let before = match facts.original_convention {
                        AssetConvention::ActualMonths => 2 * (self.placed / 100 % 100 - 1),
                        AssetConvention::MidMonth => 2 * (self.placed / 100 % 100) - 1,
                        AssetConvention::HalfYear => 12,
                        AssetConvention::MidQuarter => return Err(HISTORY),
                    };
                    half_months = half_months.saturating_sub(before);
                }
                result.quarter_months = i64::from(months) * 4;
                result.convention = facts.original_convention;
                result.method =
                    if multiplier.is_some() { "Original declining balance" } else { "S/L" };
                let (rate_basis, numerator, denominator) = match multiplier {
                    Some(value) => (
                        remaining,
                        i64::from(value) * i64::from(half_months),
                        i64::from(months) * 200,
                    ),
                    None => (remaining - salvage, i64::from(half_months), i64::from(months) * 2),
                };
                result.amount = ratio::checked_fraction(rate_basis, numerator, denominator)
                    .ok_or(HISTORY)?
                    .min(remaining - salvage);
            }
        }
        Ok(result)
    }

    // Actual ACRS dates and elections select the Pub. 534 source-versioned
    // tables below. Expired schedules can admit actual zero; a leftover
    // balance never becomes another current election or fabricated rate.
    fn acrs_recovery(
        &self,
        facts: &crate::HistoricalDepreciation,
        period: &crate::AcrsPeriod,
        alternate: bool,
        mut result: Recovery,
    ) -> Result<Recovery, &'static str> {
        use crate::AcrsPeriod as P;
        if !(19810101..=19861231).contains(&self.placed) {
            return Err(FACTS);
        }
        let years = match period {
            P::Three => 3,
            P::Five => 5,
            P::Ten => 10,
            P::Fifteen => 15,
            P::Eighteen => 18,
            P::Nineteen => 19,
            P::Alternate { years, .. }
                if alternate && [3, 5, 10, 12, 15, 18, 19, 25, 35, 45].contains(years) =>
            {
                *years
            }
            P::Alternate { .. } => return Err(FACTS),
        };
        if (years == 18 && !(19840316..=19850508).contains(&self.placed))
            || (years == 19 && self.placed < 19850509)
        {
            return Err(FACTS);
        }
        if self.earlier_business_percent.is_some() {
            return Err(HISTORY);
        }
        let real_class = match period {
            P::Alternate { original_class, years } => {
                // Pub534 (2016), Alternate ACRS Method: the original class
                // determines the lawful alternate periods, not the chosen year alone.
                let legal = match original_class {
                    AcrsOriginalClass::Personal3 => [3, 5, 12].contains(years),
                    AcrsOriginalClass::Personal5 => [5, 12, 25].contains(years),
                    AcrsOriginalClass::Personal10 => [10, 25, 35].contains(years),
                    AcrsOriginalClass::Real15 | AcrsOriginalClass::LowIncome15 => {
                        [15, 35, 45].contains(years)
                    }
                    AcrsOriginalClass::Real18 => [18, 35, 45].contains(years),
                    AcrsOriginalClass::Real19 => [19, 35, 45].contains(years),
                };
                if !legal {
                    return Err(FACTS);
                }
                match original_class {
                    AcrsOriginalClass::Personal3
                    | AcrsOriginalClass::Personal5
                    | AcrsOriginalClass::Personal10 => None,
                    _ => Some(*original_class),
                }
            }
            P::Fifteen if facts.property == HistoricalPropertyKind::RealProperty => {
                Some(AcrsOriginalClass::Real15)
            }
            P::Eighteen => Some(AcrsOriginalClass::Real18),
            P::Nineteen => Some(AcrsOriginalClass::Real19),
            _ => None,
        };
        if real_class.is_none() && facts.property == HistoricalPropertyKind::RealProperty {
            return Err(FACTS);
        }
        if let Some(class) = real_class {
            let legal = match class {
                AcrsOriginalClass::Real15 => self.placed <= 19840315,
                AcrsOriginalClass::Real18 => (19840316..=19850508).contains(&self.placed),
                AcrsOriginalClass::Real19 => self.placed >= 19850509,
                AcrsOriginalClass::LowIncome15 => true,
                AcrsOriginalClass::Personal3
                | AcrsOriginalClass::Personal5
                | AcrsOriginalClass::Personal10 => {
                    unreachable!("personal classes have no real-property table")
                }
            };
            if !legal || facts.property != HistoricalPropertyKind::RealProperty {
                return Err(FACTS);
            }
        }
        let convention_valid = match (real_class, facts.original_convention) {
            (Some(AcrsOriginalClass::Real18), AssetConvention::MidMonth) => self.placed >= 19840623,
            (Some(AcrsOriginalClass::Real18), AssetConvention::ActualMonths) => {
                self.placed < 19840623
            }
            (Some(AcrsOriginalClass::Real19), AssetConvention::MidMonth) => true,
            (
                Some(AcrsOriginalClass::Real15 | AcrsOriginalClass::LowIncome15),
                AssetConvention::ActualMonths,
            ) => true,
            (None, AssetConvention::HalfYear) => true,
            _ => false,
        };
        if !convention_valid {
            return Err(FACTS);
        }
        result.method = if alternate { "Alternate ACRS" } else { "ACRS" };
        result.quarter_months = i64::from(years) * 48;
        result.convention = facts.original_convention;
        if 2026 - self.placed / 10000 > years {
            if facts.remaining_basis != 0 {
                return Err(HISTORY);
            }
            return Ok(result);
        }
        // Only the45-year original real-property clock can still have a
        // current2026 allowance. Source class/convention select Table14/15.
        if years != 45 || !alternate || facts.property != HistoricalPropertyKind::RealProperty {
            return Err(HISTORY);
        }
        let month = (self.placed / 100 % 100 - 1) as usize;
        let index = 2026 - self.placed / 10000;
        // Literal published Pub534 Tables14/15, units0.1percent. These are
        // versioned original-table facts, never an entered annual-rate override.
        // https://www.irs.gov/pub/irs-pdf/p534.pdf (Rev.2016), Appendix14/15.
        let rate = match (facts.original_convention, index) {
            (AssetConvention::MidMonth, 0) => [21, 19, 18, 16, 14, 12, 10, 8, 6, 5, 3, 1][month],
            (AssetConvention::MidMonth, 1..=10) => 23,
            (AssetConvention::MidMonth, 11..=44) => 22,
            (AssetConvention::MidMonth, 45) => [1, 3, 4, 6, 8, 10, 12, 14, 16, 17, 19, 21][month],
            (AssetConvention::ActualMonths, 0) => {
                [23, 20, 19, 17, 15, 13, 12, 9, 7, 6, 4, 2][month]
            }
            (AssetConvention::ActualMonths, 1..=9) => 23,
            (AssetConvention::ActualMonths, 10..=44) => 22,
            (AssetConvention::ActualMonths, 45) => {
                [0, 3, 4, 6, 8, 10, 11, 14, 16, 17, 19, 21][month]
            }
            (AssetConvention::MidMonth | AssetConvention::ActualMonths, _) => 0,
            _ => return Err(FACTS),
        };
        let exposure = self.disposal_date().map_or(24, |date| match facts.original_convention {
            AssetConvention::MidMonth => 2 * (date / 100 % 100) - 1,
            _ => 2 * (date / 100 % 100 - 1),
        });
        let exact = ratio::exact::whole(self.business_basis())
            * ratio::exact::ratio(i64::from(rate) * i64::from(exposure), 24000).ok_or(HISTORY)?;
        let exact = phase::cap(exact, [ratio::exact::whole(facts.remaining_basis)]);
        result.amount = ratio::exact::rounded(&exact).ok_or(HISTORY)?;
        result.exact_amount = Some(exact);
        Ok(result)
    }

    // Monthly non-MACRS schedules include the first month of use. A disposition
    // stops their allowance before the disposition month; MACRS conventions do not apply.
    fn nonmacrs_months(&self) -> i64 {
        let first = if self.placed / 10000 == 2026 { self.placed / 100 % 100 } else { 1 };
        let after = self.disposal_date().map_or(13, |date| date / 100 % 100);
        i64::from(after.saturating_sub(first))
    }

    pub(crate) fn nonmacrs_fact_issue(&self) -> Option<&'static str> {
        if self.nonmacrs() {
            if !self.nonmacrs_source_shape_valid() {
                return Some(FACTS);
            }
            match self.nonmacrs_recovery(self.bonus_amount()) {
                Err(
                    issue @ (FACTS
                    | "FORECAST_SMALL_COST_EXCEPTION"
                    | "FORECAST_NOT_WITHIN_TEN_PERCENT"),
                ) => Some(issue),
                _ => None,
            }
        } else {
            None
        }
    }
    pub(crate) fn nonmacrs_valid(&self) -> bool {
        self.nonmacrs_source_shape_valid()
            && self.amt_section179_carry_fact_valid()
            && self.nonmacrs_fact_issue().is_none()
            && self.nonmacrs_annual_facts_valid()
    }
    fn nonmacrs_source_shape_valid(&self) -> bool {
        self.macrs_source.is_none()
            && self.id >= 0
            && self.cost > 0
            && (0..=self.cost).contains(&self.credit_reduction)
            && self.acquired <= self.placed
            && crate::dates::source_valid_date(self.acquired)
            && crate::dates::source_valid_date(self.placed)
            && self.placed <= 20261231
            && matches!(self.listed, crate::ListedProperty::No { business_percent: 1..=100 })
            && self.disposal.valid()
            && self
                .disposal_date()
                .is_none_or(|date| date >= self.placed && crate::dates::source_valid_date(date))
            && self.earlier_business_percent.is_none()
            && self.partial.is_none()
            && self.investment_credit == crate::InvestmentCredit::No
            && self.nonmacrs_history_valid()
    }
    fn nonmacrs_history_valid(&self) -> bool {
        let basis = self.business_basis();
        match self.history {
            EquipmentHistory::New { section179, .. } => {
                self.placed / 10000 == 2026
                    && (0..=basis).contains(&section179.elected())
                    && (matches!(section179, crate::Section179::Ineligible)
                        || (self.predominant()
                            && self
                                .nonmacrs_route()
                                .is_some_and(NonMacrsRecovery::section179_property)))
            }
            EquipmentHistory::Continuing {
                prior_depreciation,
                initial_section179,
                carryforward,
                schedule,
                amt,
                ..
            } => {
                let original_bonus = match schedule {
                    EquipmentSchedule::Tables { initial_bonus } => initial_bonus,
                    EquipmentSchedule::Manual => return false,
                };
                self.placed < 20260101
                    && prior_depreciation >= 0
                    && carryforward >= 0
                    && prior_depreciation
                        .checked_add(carryforward)
                        .is_some_and(|used| used <= basis)
                    && initial_section179 >= carryforward
                    && initial_section179 <= prior_depreciation + carryforward
                    && original_bonus >= 0
                    && original_bonus <= prior_depreciation + carryforward - initial_section179
                    && matches!(amt, crate::EquipmentAmtHistory::Same)
                    && (original_bonus == 0
                        || matches!(self.nonmacrs_route(), Some(NonMacrsRecovery::Software { .. })))
                    && (initial_section179 == 0
                        || self.nonmacrs_route().is_some_and(NonMacrsRecovery::section179_property))
            }
        }
    }
    fn actual_monthly(
        &self,
        original_basis: i64,
        remaining: i64,
        months: u32,
    ) -> Result<i64, &'static str> {
        use crate::kernel::cost_recovery::{Month, monthly_history::ActualMonthlySchedule};
        if remaining < 0 || remaining > original_basis {
            return Err(HISTORY);
        }
        let current = ActualMonthlySchedule {
            started: Month { year: self.placed / 10000, month: self.placed / 100 % 100 },
            period_months: months,
            tax_year: 2026,
            basis_cents: original_basis,
            prior_recovery_cents: Some(original_basis - remaining),
            allowed_through: None,
            allowed_before: self
                .disposal_date()
                .map(|date| Month { year: date / 10000, month: date / 100 % 100 }),
        }
        .recover()
        .map_err(|_| HISTORY)?;
        Ok(current.current_cents)
    }
    fn nonmacrs_annual_facts_valid(&self) -> bool {
        let Some(records) = &self.history_details else {
            return true;
        };
        let (elected, prior, carry, bonus) = match self.history {
            EquipmentHistory::New { section179, .. } => {
                (section179.elected(), 0, 0, self.bonus_amount())
            }
            EquipmentHistory::Continuing {
                initial_section179,
                prior_depreciation,
                carryforward,
                schedule,
                ..
            } => (
                initial_section179,
                prior_depreciation,
                carryforward,
                match schedule {
                    EquipmentSchedule::Tables { initial_bonus } => initial_bonus,
                    _ => return false,
                },
            ),
        };
        let same_use = |value: crate::UseFraction| {
            value.total > 0
                && value.used <= value.total
                && i64::from(value.used) * 10000
                    == self.business_use_hundredths() * i64::from(value.total)
        };
        let sum = |select: fn(&crate::AssetYearHistory) -> i64| {
            records.years.iter().try_fold(0i64, |sum, row| {
                let value = select(row);
                if value < 0 { None } else { sum.checked_add(value) }
            })
        };
        records.original_business_basis == self.business_basis()
            && records.original_section179_elected == elected
            && records.original_bonus == bonus
            && same_use(records.original_income_use)
            && records.years.len() == (2026 - self.placed / 10000) as usize
            && records.years.iter().all(|row| {
                (self.placed / 10000..2026).contains(&row.year)
                    && records.years.iter().filter(|other| other.year == row.year).count() == 1
                    && same_use(row.income_use)
                    && row.amt >= 0
                    && row.basis_increase >= 0
                    && row.basis_decrease >= 0
                    && row.recaptured >= 0
            })
            && sum(|row| row.section179_allowed).and_then(|value| value.checked_add(carry))
                == Some(elected)
            && sum(|row| row.bonus) == Some(bonus)
            && sum(|row| row.section179_allowed)
                .and_then(|value| value.checked_add(sum(|row| row.bonus)?))
                .and_then(|value| value.checked_add(sum(|row| row.regular)?))
                == Some(prior)
            && records.carryovers.iter().try_fold(0i64, |sum, row| {
                if row.amount >= 0 && row.year == self.placed / 10000 {
                    sum.checked_add(row.amount)
                } else {
                    None
                }
            }) == Some(carry)
    }
}

impl BusinessAsset {
    pub(crate) fn credit_cost_on(&self, cost: i64) -> i64 {
        self.business_share_of(cost)
    }

    // The energy credit determined for the supplied contextual cost, whole dollars.
    pub(crate) fn energy_credit_determined_on(&self, cost: i64) -> i64 {
        match &self.investment_credit {
            InvestmentCredit::NewEnergy {
                increased_rate,
                domestic_content,
                energy_community,
                ..
            } => dollars(fraction(
                positive(self.credit_cost_on(cost) - self.elected179()),
                energy_percent(*increased_rate, *domestic_content, *energy_community),
                100,
            )),
            _ => 0,
        }
    }
    pub(crate) fn investment_basis_reduction_on(&self, cost: i64) -> i64 {
        match &self.investment_credit {
            InvestmentCredit::No => 0,
            InvestmentCredit::Rehabilitation(_) => fraction(
                RehabilitationCredit::determined(self.credit_cost_on(cost)),
                REHABILITATION_BASIS_REDUCTION_PERCENT,
                100,
            ),
            InvestmentCredit::NewEnergy { .. } => fraction(
                self.energy_credit_determined_on(cost) * 100,
                ENERGY_BASIS_REDUCTION_PERCENT,
                100,
            ),
            InvestmentCredit::EarlierEnergy { credit, .. } => {
                fraction(*credit, ENERGY_BASIS_REDUCTION_PERCENT, 100)
            }
        }
    }
    // The shape the investment credit needs from the asset row (the page
    // rules' server twin): a rehabilitation credit on a 39-year building row
    // placed in service after 2017, new energy property on 5-year equipment
    // placed in service in 2026, an earlier energy credit on continuing
    // 5-year equipment.
    pub(crate) fn investment_shape_ok(&self) -> bool {
        self.investment_shape_on_cost(self.cost)
    }
    pub(crate) fn investment_shape_on_cost(&self, cost: i64) -> bool {
        let new = matches!(self.history, crate::EquipmentHistory::New { .. });
        match &self.investment_credit {
            InvestmentCredit::No => true,
            InvestmentCredit::Rehabilitation(claim) => {
                self.class == crate::EquipmentClass::ThirtyNine
                    && self.placed >= 20180101
                    && !self.is_vehicle()
                    && claim.period_start < self.placed
                    && claim.rule(self.placed, self.credit_cost_on(cost)).is_none()
            }
            InvestmentCredit::NewEnergy { construction_began, .. } => {
                self.class == crate::EquipmentClass::Five
                    && new
                    && !self.is_vehicle()
                    && self.placed / 10000 == 2026
                    && *construction_began <= self.placed
            }
            InvestmentCredit::EarlierEnergy { credit, used, carried, .. } => {
                self.class == crate::EquipmentClass::Five
                    && !new
                    && !self.is_vehicle()
                    && used + carried <= *credit
            }
        }
    }
    pub(crate) fn disposal_date(&self) -> Option<u32> {
        self.disposal.date()
    }
}
