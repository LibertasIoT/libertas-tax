// Coverage: Sale of your main home (section 121, Pub. 523; Form 8949/Schedule D) — Supported
//   (2026-09-28; calculation audited against Pub. 523 (2025 draft, Worksheets 1-3), the
//   2025 draft Schedule D and Form 8949 instructions and the Form 1099-S (Rev. 12-2026)).
//   Lines: realized gain (Worksheet 2 line 7; a loss is not deductible), the post-May 6, 1997
//   depreciation gain (unrecaptured section 1250, Schedule D line 19), the nonqualified-use gain
//   (after depreciation, by days), the $250,000 / $500,000 limit (joint: one spouse owns, both
//   live and neither excluded a sale in 2 years; otherwise each qualifying spouse's own $250,000),
//   the reduced exclusion (shortest of residence, ownership and time since the last excluded sale
//   over 730 days, per spouse), the surviving spouse's $500,000, licensed care-facility time, the
//   service suspension (section 121(d)(9): the page's Qualified official extended duty, each
//   spouse's days in the added period before the 5 years), the election to report the gain.
//   Separate business or rental space (Pub. 523, Business Use or Rental of Home): the price,
//   selling expenses and cost (adjusted basis plus its depreciation) are divided by floor area;
//   its gain or loss is a Form 4797 line 2 section 1231 source with its straight-line
//   depreciation as unrecaptured section 1250 gain (separate_sources, read by section1231.rs),
//   and the rest is the home's sale. Held a year or less, an unknown depreciation or a carried
//   passive loss is refused by name. How you got it (sections 121(d)(2)-(3), 1223(2), (9), 1041):
//   an inherited home is long-term, a gift adds the giver's holding period, a spouse or former
//   spouse transfer adds their ownership and holding period and the days the divorce agreement
//   gave them use; a surviving spouse adds the late spouse's ownership and use days; a home got in
//   a like-kind exchange within 5 years has no exclusion (121(d)(10)). Installment sale (Pub. 523,
//   Installment Sale): Form 6252 with the exclusion on line 15, the year's gain on Schedule D
//   line 4 or 11 with unrecaptured section 1250 gain first; a note below the federal rate, a
//   dealer or contingent sale and a separate space are refused by name. 1099-S box 6 property is
//   added to the price at its value; box 7 is a foreign seller, whose Form 8288-A withholding is a
//   Form 1040 line 25c credit. A foreclosure's gain is this sale at the foreclosure's amount
//   realized (debt.rs). Reporting:
//   Form 8949 box C/F with columns (d), (e), (g) codes E/H/L and (h), and Schedule D line 3/10,
//   whenever a 1099-S was received or gain is taxable; before, the taxable gain reached line 16
//   but no Form 8949 row or line 3/10 total, and a fully excluded sale without a 1099-S was not
//   distinguished. Page: the Document Center kind Form1099S ("Form 1099-S", 2026-10-31, G-13)
//   stores Form1099SPaper (all printed boxes) without asking its use; Investment sales asks
//   what was sold (RealEstateSaleUse: the main home with HomeSaleAnswers, other real estate as a
//   Form 8949 box C/F sale, or rental/business/farm property counted on its own page with a
//   box 2a price check, ISSUE_REAL_ESTATE_SALE_PRICE); a sale with no form uses the home-sale income
//   record and the same normalized transaction fields. HomeSaleSource preserves that provenance,
//   with the filer's HomeSaleAnswers beside it as the topic answer HomeSale (MainHomeSale;
//   unanswered, the sale is left out of the estimate); the holding period, days owned and the
//   5-year window (1,826 days in 2026) derive
//   from the dates; HomeSale (the calculation view of the source) .facts keeps the dates, the 1099-S answers, the earlier-sale dates, the report-the-gain
//   election and a refused situation. Refused by name: the expatriation tax (Form 8854).
//   Exchanging a former home held for rent or investment for like-kind property (Form 8824,
//   Rev. Proc. 2005-14): the exclusion first, then line 20 is the cash and other property received
//   less the exclusion, up to the gain; a rental's recognized gain is a section 1231 source, an
//   investment's a capital gain. Left out with a per-item notice (Answers and unknowns,
//   2026-10-05): non-straight-line depreciation (Form 4797), another situation. Rules on both sides: HOME_ACQUIRED_AFTER_SALE, HOME_LIVED_ABOVE_OWNED,
//   HOME_EARLIER_SALE_AFTER_SALE, HOME_NONQUALIFIED_ABOVE_OWNED, HOME_DEATH_AFTER_SALE,
//   HOME_SURVIVING_WITH_REASON, HOME_SURVIVING_ON_JOINT, HOME_SEPARATE_AREA_ABOVE_WHOLE,
//   HOME_SEPARATE_DEPRECIATION_ABOVE_COST, HOME_SERVICE_SPOUSE_NEEDED / _NOT_JOINT,
//   HOME_SERVICE_WITH_SURVIVING, HOME_SERVICE_LIVED_ABOVE_OWNED, HOME_PRIOR_OWNER_AFTER_ACQUIRED,
//   HOME_PROPERTY_VALUE_WITHOUT_BOX_6, HOME_FIRPTA_WITHOUT_BOX_7; server only HOME_SALE_NOT_IN_CURRENT_YEAR; the
//   spouse answers by ConstrainedBy ($.allowed_home_spouse) with HOME_SPOUSE_NEEDED /
//   HOME_SPOUSE_NOT_JOINT. Tests: src/tests/home/home_sale_page.rs (fixtures: fully excluded single,
//   joint $500,000 ($23,978 tax), work-move partial ($9,876), depreciation and nonqualified use
//   ($25,276), loss; boundaries: exactly 2 years, $250,000/$250,000.01 and $500,000/$500,000.01,
//   730/729 days since an earlier sale, the one-year holding period; Form 8828 on a sale; round
//   trips); src/tests/home/business.rs, src/tests/family/family.rs, src/tests/home/general.rs, src/tests/home/investment.rs (the engine cases).
//   Answers (2026-10-05): the facility, remarriage and event confirmations the page always
//   asserted are no longer stored; prior exclusions and the widowed facts are booleans; a service
//   election whose conditions are No suspends nothing (the ordinary window counts); a sale whose
//   `ordinary` is No is left out with WARN_HOME_SALE_LEFT_OUT.
use crate::kernel::phase;
use crate::{FederalStatus, GainTerm};
use alloc::string::String;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Residence in a care facility
/// Facility residence during the five-year sale window; both zero when there is none to count. Count it only when you became physically or mentally unable to care for yourself during that window and the facility is licensed by a state or political subdivision to care for people with your condition.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeCare {
    /// Time in the facility while still owning the home
    /// Exclude time already counted as residence in the home.
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_default(0)]
    #[libertas_physical_unit("day")]
    pub days: u32,
    /// Time both owning and living in the home during that window
    /// This time is already included in the home-residence count, not additional.
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_default(0)]
    #[libertas_physical_unit("day")]
    pub owned_residence_days: u32,
}
impl HomeCare {
    /// No facility residence to count.
    pub(crate) const NONE: Self = Self { days: 0, owned_residence_days: 0 };
    fn valid(&self, lived: u32, owned: u32) -> bool {
        self.days <= 1827
            && self.owned_residence_days <= lived.min(owned)
            && lived + self.days <= 1827
            && self.owned_residence_days + self.days <= owned
    }
    fn residence(&self, lived: u32) -> u32 {
        lived + if self.owned_residence_days >= 365 { self.days } else { 0 }
    }
}

/// This filer's home history
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeUse {
    /// Time owned in the five years ending on sale date
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_physical_unit("day")]
    pub owned: u32,
    /// Time used as main home in those five years
    /// Periods need not be continuous. Count qualifying short temporary absences; do not count rental use as residence.
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_physical_unit("day")]
    pub lived: u32,
    /// Additional care-facility residence
    pub care: HomeCare,
    /// Excluded gain on another home sold in the two years before this sale
    /// The earlier sale's closing date and that year's return (Form 8949 or Schedule D) show it.
    pub prior_exclusion: bool,
}
impl HomeUse {
    fn valid(&self) -> bool {
        self.owned <= 1827 && self.lived <= 1827 && self.care.valid(self.lived, self.owned)
    }
}

/// Service that can suspend the home-sale ownership and residence test period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HomeDutyKind {
    /// Uniformed Services, including NOAA and Public Health Service commissioned corps
    Uniformed { duty: QualifiedHomeDuty },
    /// Foreign Service of the United States
    ForeignService { duty: QualifiedHomeDuty },
    /// Employee of a qualifying U.S. intelligence-community element
    Intelligence { duty: QualifiedHomeDuty },
    /// Peace Corps employee serving outside the United States on qualified official extended duty
    PeaceCorpsEmployeeAbroad { duty: QualifiedHomeDuty },
    /// Enrolled Peace Corps volunteer or volunteer leader serving outside the United States
    /// This statutory path does not require an active-duty order or distant duty station.
    PeaceCorpsVolunteerAbroad,
    /// Another or unresolved service category
    Other,
}
impl HomeDutyKind {
    fn qualifies(&self) -> bool {
        match self {
            Self::Uniformed { duty }
            | Self::ForeignService { duty }
            | Self::Intelligence { duty }
            | Self::PeaceCorpsEmployeeAbroad { duty } => duty.qualifies(),
            Self::PeaceCorpsVolunteerAbroad => true,
            Self::Other => false,
        }
    }
}

/// Qualified official extended-duty facts for service personnel
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct QualifiedHomeDuty {
    /// Ordered active duty was indefinite or for more than 90 days
    /// The orders show it. No means the service suspension does not apply.
    pub extended_orders: bool,
    /// Duty station was at least 50 miles from this home, or government orders required living in government quarters
    /// The orders and the duty-station address show it. No means the service suspension does not apply.
    pub remote_or_quarters: bool,
}
impl QualifiedHomeDuty {
    fn qualifies(&self) -> bool {
        self.extended_orders && self.remote_or_quarters
    }
}

/// Election to suspend the home-sale test window for one property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeServiceElection {
    /// Qualifying service performed by this filer or a spouse
    pub service: HomeDutyKind,
    /// No other property used this suspension during the selected period
    /// Your earlier returns show any election for another property. No means the service suspension does not apply.
    pub one_property: bool,
    /// Time of qualifying service elected for this home
    /// At most ten years; do not include time elected for another property.
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 1, max = 3653)]
    #[libertas_physical_unit("day")]
    pub suspended_days: u32,
}
impl HomeServiceElection {
    fn valid(&self) -> bool {
        (1..=3653).contains(&self.suspended_days)
    }
    fn qualifies(&self) -> bool {
        self.service.qualifies() && self.one_property
    }
}

/// Licensed care-facility residence within a service-extended home-sale window
/// Facility residence while still owning the home; every count zero when there is none. Count it only when you became physically or mentally unable to care for yourself during the extended window and the facility is licensed to care for people with your condition.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DutyHomeCare {
    /// Facility time in the ordinary five-year window, not counted as residence in the home
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_default(0)]
    #[libertas_physical_unit("day")]
    pub recent_days: u32,
    /// Facility time in the additional period reached by the service election
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
    #[libertas_default(0)]
    #[libertas_physical_unit("day")]
    pub earlier_days: u32,
    /// Time both owning and living in the home in the ordinary five-year window
    /// Already included in home-residence time, not additional.
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_default(0)]
    #[libertas_physical_unit("day")]
    pub recent_owned_residence_days: u32,
    /// Time both owning and living in the home in the added window
    /// Already included in home-residence time, not additional.
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
    #[libertas_default(0)]
    #[libertas_physical_unit("day")]
    pub earlier_owned_residence_days: u32,
}
impl DutyHomeCare {
    /// No qualifying facility residence.
    #[cfg(test)]
    pub(crate) const NONE: Self = Self {
        recent_days: 0,
        earlier_days: 0,
        recent_owned_residence_days: 0,
        earlier_owned_residence_days: 0,
    };
    fn valid(&self, history: &DutyHomeUse, suspended_days: u32) -> bool {
        self.recent_days <= 1827
            && self.earlier_days <= suspended_days
            && history.recent_lived + self.recent_days <= 1827
            && history.earlier_lived + self.earlier_days <= suspended_days
            && self.recent_owned_residence_days <= history.recent_lived
            && self.earlier_owned_residence_days <= history.earlier_lived
            && self.recent_owned_residence_days + self.recent_days <= history.recent_owned
            && self.earlier_owned_residence_days + self.earlier_days <= history.earlier_owned
    }
    fn residence(&self, lived: u32) -> u32 {
        HomeCare {
            days: self.recent_days + self.earlier_days,
            owned_residence_days: self.recent_owned_residence_days
                + self.earlier_owned_residence_days,
        }
        .residence(lived)
    }
    // The ordinary five-year window alone, without the service election.
    fn recent(&self) -> HomeCare {
        HomeCare { days: self.recent_days, owned_residence_days: self.recent_owned_residence_days }
    }
}

/// Ownership and residence in the ordinary and added test windows
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DutyHomeUse {
    /// Time owned in the ordinary five years before sale
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_physical_unit("day")]
    pub recent_owned: u32,
    /// Time used as a main home in the ordinary five years before sale
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_physical_unit("day")]
    pub recent_lived: u32,
    /// Time owned in the additional period reached by the service election
    /// This earlier period cannot exceed the selected suspension period.
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
    #[libertas_physical_unit("day")]
    pub earlier_owned: u32,
    /// Time used as a main home in that additional period
    /// Count qualifying short temporary absences; do not count rental use as residence.
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
    #[libertas_physical_unit("day")]
    pub earlier_lived: u32,
    /// Additional licensed care-facility residence in the extended window
    pub care: DutyHomeCare,
}
impl DutyHomeUse {
    fn valid(&self, suspended_days: u32) -> bool {
        self.recent_owned <= 1827
            && self.recent_lived <= 1827
            && self.earlier_owned <= suspended_days
            && self.earlier_lived <= suspended_days
            && self.care.valid(self, suspended_days)
    }
    fn ownership(&self) -> u32 {
        self.recent_owned + self.earlier_owned
    }
    fn residence_days(&self) -> u32 {
        self.recent_lived + self.earlier_lived
    }
    fn residence(&self) -> u32 {
        self.care.residence(self.residence_days())
    }
    // Ownership and residence in the ordinary five-year window, used when
    // the service election does not apply (a No on its conditions).
    fn ordinary(&self) -> (u32, u32) {
        (self.recent_owned, self.care.recent().residence(self.recent_lived))
    }
}

/// Filer history for a full home-sale exclusion with a service election
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FullDutyHomeUse {
    /// Ownership and residence in the ordinary and added test windows
    pub history: DutyHomeUse,
    /// Excluded gain on another home sold in the two years before this sale
    /// The service election does not suspend this separate look-back test. The earlier sale's closing date and that year's return show it.
    pub prior_exclusion: bool,
}
impl FullDutyHomeUse {
    fn valid(&self, suspended_days: u32) -> bool {
        self.history.valid(suspended_days)
    }
}

/// Filer periods for a reduced home-sale exclusion with a service election
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct DutyReducedHomeUse {
    /// Ownership and residence in the ordinary and added test windows
    /// On a joint return, count either spouse's ownership in each window without double-counting overlapping days; residence remains individual.
    pub history: DutyHomeUse,
    /// Time since this filer's last excluded home sale
    /// Enter 730 if no prior excluded sale occurred within two years.
    // Section 121 tax structure: the two-year interval between exclusions.
    #[libertas_number(min = 0, max = 730)]
    #[libertas_physical_unit("day")]
    pub since_prior: u32,
}
impl DutyReducedHomeUse {
    fn valid(&self, suspended_days: u32) -> bool {
        self.history.valid(suspended_days) && self.since_prior <= 730
    }
    // Without a qualifying election only the ordinary window counts.
    fn limit(&self, suspended: bool) -> i64 {
        let (owned, lived) = if suspended {
            (self.history.ownership(), self.history.residence())
        } else {
            self.history.ordinary()
        };
        crate::kernel::ratio::fraction(
            crate::year_constants::HOME_SALE_EXCLUSION_SINGLE_CENTS,
            i64::from(owned.min(lived).min(self.since_prior).min(730)),
            730,
        )
    }
}

/// Periods for a reduced home-sale exclusion
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ReducedHomeUse {
    /// Time owned during the five years ending on sale date
    /// On a joint return, include time either spouse owned the home, counting overlapping time only once.
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_physical_unit("day")]
    pub owned: u32,
    /// Time this filer used the property as a main home in those five years
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_physical_unit("day")]
    pub lived: u32,
    /// Additional care-facility residence
    pub care: HomeCare,
    /// Time since this filer's last excluded home sale
    /// Enter 730 if there was no prior excluded sale or it was at least two years earlier.
    // Section 121 tax structure: the two-year interval between exclusions.
    #[libertas_number(min = 0, max = 730)]
    #[libertas_physical_unit("day")]
    pub since_prior: u32,
}
impl ReducedHomeUse {
    fn valid(&self) -> bool {
        self.owned <= 1827
            && self.lived <= 1827
            && self.care.valid(self.lived, self.owned)
            && self.since_prior <= 730
    }
    fn limit(&self) -> i64 {
        crate::kernel::ratio::fraction(
            crate::year_constants::HOME_SALE_EXCLUSION_SINGLE_CENTS,
            i64::from(
                self.owned.min(self.care.residence(self.lived)).min(self.since_prior).min(730),
            ),
            730,
        )
    }
}

/// Event qualifying the home sale for a reduced exclusion
/// For personal events, the affected person is you, your spouse, a co-owner or someone whose residence was this home. Keep supporting records. Other facts-and-circumstances claims require separate review.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HomeSaleTrigger {
    /// New job or transfer farther from the home
    WorkMove {
        /// Distance from the home to the former work location
        /// Enter zero if there was no former work location.
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("mile")]
        old_miles: u32,
        /// Distance from the home to the new work location
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("mile")]
        new_miles: u32,
    },
    /// Move to obtain, provide or facilitate medical care
    /// The move addresses disease, illness or injury through diagnosis, treatment, mitigation or medical/personal care, not general well-being. The patient is the qualifying resident/co-owner or their spouse, parent, grandparent, child, grandchild, sibling, step-parent, step-child, step-sibling, eligible foster/adopted child, parent/child/sibling-in-law, aunt, uncle, niece or nephew. A doctor's recommendation is an alternative route, not required for this documented medical-care reason.
    MedicalCare,
    /// Doctor recommended changing residence because of a health problem
    PhysicianRecommended,
    /// Employment-status change made basic household expenses unaffordable
    /// The qualifying resident/co-owner could no longer pay basic food, clothing, housing, medication, transportation, taxes, court-ordered payments or expenses necessary to earn income because employment status changed. A preference for cheaper housing alone does not qualify.
    EmploymentHardship,
    /// Became eligible for unemployment compensation
    Unemployment,
    /// Birth of two or more children from one pregnancy
    MultipleBirth,
    /// Divorce, legal separation or separate-maintenance decree
    Divorce,
    /// Death of a qualifying resident or co-owner
    Death,
    /// Home destroyed or condemned
    Destruction,
    /// Home casualty loss from disaster or terrorism
    Casualty,
    /// Another or unresolved reason
    Other,
}
impl HomeSaleTrigger {
    fn valid(&self) -> bool {
        match self {
            Self::WorkMove { old_miles, new_miles } => *old_miles <= 25000 && *new_miles <= 25000,
            _ => true,
        }
    }
    fn qualifies(&self) -> bool {
        match self {
            Self::WorkMove { old_miles, new_miles } => *new_miles >= old_miles + 50,
            Self::Other => false,
            _ => true,
        }
    }
}

/// Home-sale exclusion choice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HomeExclusion {
    /// Evaluate the surviving-spouse exclusion
    /// You have not remarried before this sale.
    Widowed {
        /// Sale closing date in 2026
        #[libertas_date_only]
        sale_date: u32,
        /// Spouse's date of death
        #[libertas_date_only]
        death_date: u32,
        /// Surviving filer's ownership, residence and prior-exclusion history at sale
        /// Include qualifying late-spouse ownership and residence periods, counting overlapping days once. Basis must separately reflect inheritance adjustments from estate records.
        history: HomeUse,
        /// Late spouse excluded another home sale within two years before this sale
        /// The late spouse's prior returns show it. Yes limits the exclusion to $250,000.
        late_prior: bool,
        /// The joint $500,000 ownership, use and look-back conditions were met immediately before death
        /// In the five years ending at death, either spouse owned the home at least two years, both met the residence test with applicable statutory attribution, and neither had an excluded home sale in the preceding two years. Your and your late spouse's residence and sale records show it. No limits the exclusion to $250,000.
        joint_tests_at_death: bool,
    },
    /// Evaluate a reduced exclusion for a qualifying event
    Reduced {
        /// Qualifying event
        /// The event occurred while you owned and lived in the home and meets the selected description.
        reason: HomeSaleTrigger,
        /// Taxpayer periods
        taxpayer: ReducedHomeUse,
        /// Spouse periods for a joint return, absent otherwise
        spouse: Option<ReducedHomeUse>,
    },
    /// Elect to suspend the five-year ownership and residence test for qualifying service
    /// Extend the normal five-year window backward by elected qualifying-service days, up to a 15-year calendar span. Record each filer's ownership, home residence and licensed care-facility residence in the ordinary and added parts separately. Keep dated service, residence and ownership records.
    ServiceSuspension {
        /// Service and one-property election
        election: HomeServiceElection,
        /// Taxpayer's ownership, residence and prior-sale facts in the extended test window
        taxpayer: FullDutyHomeUse,
        /// Spouse's facts for a joint return; absent on other returns
        spouse: Option<FullDutyHomeUse>,
    },
    /// Evaluate a reduced exclusion for a qualifying event with a service-extended test window
    ReducedService {
        /// Service and one-property election
        election: HomeServiceElection,
        /// Qualifying reason for sale
        /// The event occurred while you owned and lived in the home and meets the selected description.
        reason: HomeSaleTrigger,
        /// Taxpayer periods in the extended window
        taxpayer: DutyReducedHomeUse,
        /// Spouse periods for a joint return, absent otherwise
        spouse: Option<DutyReducedHomeUse>,
    },
    /// Elect not to exclude this sale's gain
    /// Keeps the gain taxable, for example to preserve an exclusion for another sale. Do not choose this merely because eligibility is unknown.
    Decline,
    /// Evaluate the full exclusion from ownership and residence facts
    Full {
        /// Taxpayer history
        taxpayer: HomeUse,
        /// Spouse history for a joint return
        /// Required for a joint return, absent otherwise. Enter the entire jointly reported sale once.
        spouse: Option<HomeUse>,
    },
}

/// Home-sale use adjustments
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HomeSaleUse {
    /// No depreciation or nonqualified use
    Personal,
    /// Depreciation or periods of nonqualified use
    Adjusted {
        /// Depreciation allowed or allowable after May 6, 1997
        /// Straight-line section 1250 depreciation only, already reflected in adjusted basis. Include allowable deductions even if not claimed. No ordinary recapture, section 1231 look-back recharacterization or regular/AMT basis difference.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        depreciation: i64,
        /// Total time of ownership through sale
        /// Count the entire ownership period, including years before 2009.
        #[libertas_number(min = 1)]
        #[libertas_physical_unit("day")]
        owned_days: u32,
        /// Time of nonqualified use after 2008
        /// Count time neither you nor your spouse or former spouse used the property as a main home. Exclude the period after its last use as a main home within the five-year sale window, qualifying temporary absences of up to two years for work, health or unforeseen circumstances, and qualifying official extended duty of up to ten years. Retain dated use records; overlapping time counts once.
        #[libertas_number(min = 0)]
        #[libertas_physical_unit("day")]
        nonqualified_days: u32,
    },
}
impl HomeSaleUse {
    fn valid(&self) -> bool {
        match self {
            Self::Personal => true,
            Self::Adjusted { depreciation, owned_days, nonqualified_days } => {
                (0..=100_000_000_000).contains(depreciation)
                    && (1..=73050).contains(owned_days)
                    && nonqualified_days <= owned_days
            }
        }
    }
}

/// What you held the old home for
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ExchangeUse {
    /// A rental
    /// Reported on Schedule E; a gain is a section 1231 gain on Form 4797.
    Rental,
    /// An investment
    /// Land or other property held for investment; a gain is a capital gain on Schedule D.
    Investment,
}

/// Exchange for like-kind property
/// Rev. Proc. 2005-14: the exclusion applies first, then only the cash and other property you received, less the exclusion, is taxed (Form 8824 lines 15 and 20). The old home must have been held for rent or investment when you exchanged it.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeExchange {
    /// Fair market value of the like-kind property you received
    /// Real property held for rent or investment. Include it in the selling price above: the rest of the price is cash or other property (boot) and the loans the other party took over.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub value: i64,
    /// What you held the old home for
    pub held_for: ExchangeUse,
}

/// Form 8824 lines 15-25 for the exchange of a home, in cents
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ExchangeForm {
    pub line15: i64,
    pub line16: i64,
    pub line18: i64,
    pub line19: i64,
    pub line20: i64,
    pub line24: i64,
    pub line25: i64,
}

/// What the separate space was used for
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum SeparateUse {
    /// A business you ran
    /// Reported on Schedule C or Schedule F; you took part in it regularly, continuously and substantially.
    Business,
    /// A rental
    /// Reported on Schedule E.
    Rental {
        /// Passive rental losses carried over from earlier years
        /// Form 8582 and your earlier returns show the unused loss. Enter zero if none; a carryover is released by this sale, which is not calculated here.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        carried_loss: i64,
    },
}

/// Business or rental space outside your living area
/// A separate building, or a separate unit such as the other half of a duplex. Space inside your own dwelling, such as a home office, is not entered here: enter its depreciation above. Part of the price, selling expenses and cost belongs to this space; its gain is reported on Form 4797 and cannot be excluded.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SeparateHomePart {
    /// What it was used for
    pub kind: SeparateUse,
    /// Floor area of the separate space
    /// Square feet, or any unit used for both areas. Rooms or value work too when they are the way you divided the cost for depreciation.
    // Allocation of the price, expenses and cost between the parts of one property (Pub. 523).
    #[libertas_number(min = 1)]
    pub area: u32,
    /// Floor area of the whole property
    /// The separate space and your own dwelling together, in the same unit.
    #[libertas_number(min = 1)]
    pub whole_area: u32,
    /// Depreciation allowed or allowable on the separate space
    /// Straight-line depreciation on its building, including amounts you could have deducted but did not. Your Schedule C, Schedule E and Form 4562 records show it.
    pub depreciation: crate::SourceCents,
}

/// What the separate space's share of the sale comes to, in cents
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SeparateSale {
    pub price: i64,
    pub expenses: i64,
    // Adjusted basis after the depreciation.
    pub basis: i64,
    pub depreciation: i64,
}
impl SeparateSale {
    pub(crate) fn gain(&self) -> i64 {
        self.price - self.expenses - self.basis
    }
    // Unrecaptured section 1250 gain: straight-line depreciation, up to the gain.
    pub(crate) fn unrecaptured(&self) -> i64 {
        phase::cap(phase::floor(self.gain(), 0), [self.depreciation])
    }
}
impl SeparateHomePart {
    fn depreciation_cents(&self) -> Option<i64> {
        source_cents(&self.depreciation)
    }
    // Its share of the price, selling expenses and cost (the adjusted basis
    // plus the depreciation taken on it), by floor area.
    fn sale(&self, sale: &HomeSale) -> Option<SeparateSale> {
        let depreciation = self.depreciation_cents()?;
        let share =
            |v| crate::kernel::ratio::fraction(v, i64::from(self.area), i64::from(self.whole_area));
        Some(SeparateSale {
            price: share(sale.price),
            expenses: share(sale.expenses),
            basis: share(sale.basis + depreciation) - depreciation,
            depreciation,
        })
    }
    fn valid(&self, sale: &HomeSale) -> bool {
        (1..self.whole_area).contains(&self.area)
            && self.depreciation_cents().is_none_or(|d| (0..=100_000_000_000).contains(&d))
            && self.sale(sale).is_none_or(|s| s.basis >= 0)
            && match self.kind {
                SeparateUse::Business => true,
                SeparateUse::Rental { carried_loss } => {
                    (0..=100_000_000_000).contains(&carried_loss)
                }
            }
    }
}

/// Sale of your main home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeSale {
    /// Selling price attributable to the interest reported on this return
    /// Use closing records, before selling expenses; do not subtract the mortgage payoff. Joint filers enter the combined interest once, separate filers only their own share.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub price: i64,
    /// Selling expenses for that interest
    /// Commissions and allowable closing costs, not mortgage principal or moving costs. Do not also include these expenses in basis.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub expenses: i64,
    /// Adjusted basis for that interest before selling expenses
    /// Retain acquisition, capital improvement and other basis-adjustment records. Use adjusted basis, not current mortgage balance.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Holding period on sale date
    pub term: GainTerm,
    /// Depreciation and nonqualified-use records
    pub use_history: HomeSaleUse,
    /// Business or rental space outside the living area, absent when there is none
    #[libertas_default($null)]
    pub separate: Option<SeparateHomePart>,
    /// How the price was paid
    pub payment: crate::SalePayment,
    /// Exchange for like-kind property, absent for a sale
    #[libertas_default($null)]
    pub exchange: Option<HomeExchange>,
    /// Exclusion choice and history
    pub exclusion: HomeExclusion,
    /// Home sale with known basis and no unlisted special treatment
    /// No separate business/rental part outside the dwelling unit, ordinary depreciation recapture, section 1231 look-back recharacterization, regular/AMT basis difference, debt cancellation, expatriation, related-party restriction, unlisted divorce/death ownership rules or other special treatment. Your purchase and closing records and prior returns show it. No leaves this sale out of the estimate with a notice; that situation is not supported.
    pub ordinary: bool,
    /// Closing facts from the sale's page
    /// Absent for a sale not entered on the Sale of your main home page.
    #[libertas_default($null)]
    pub facts: Option<HomeSaleFacts>,
}
impl HomeSale {
    pub(crate) fn valid(&self) -> bool {
        [self.price, self.expenses, self.basis].iter().all(|n| (0..=100_000_000_000).contains(n))
            && self.facts.as_ref().is_none_or(HomeSaleFacts::valid)
            && self.use_history.valid()
            && self.separate.as_ref().is_none_or(|part| part.valid(self))
            && self.payment.valid(self.price)
            && self.exchange.as_ref().is_none_or(|e| (0..=self.price).contains(&e.value))
            && match &self.exclusion {
                HomeExclusion::Widowed { sale_date, death_date, history, .. } => {
                    crate::federal_rules::valid_date(*sale_date)
                        && *sale_date / 10000 == 2026
                        && crate::federal_rules::valid_date(*death_date)
                        && death_date <= sale_date
                        && history.valid()
                }
                HomeExclusion::Reduced { taxpayer, spouse, reason, .. } => {
                    taxpayer.valid()
                        && spouse.as_ref().is_none_or(ReducedHomeUse::valid)
                        && reason.valid()
                }
                HomeExclusion::ServiceSuspension { election, taxpayer, spouse, .. } => {
                    election.valid()
                        && taxpayer.valid(election.suspended_days)
                        && spouse.as_ref().is_none_or(|p| p.valid(election.suspended_days))
                }
                HomeExclusion::ReducedService { election, reason, taxpayer, spouse, .. } => {
                    election.valid()
                        && reason.valid()
                        && taxpayer.valid(election.suspended_days)
                        && spouse.as_ref().is_none_or(|p| p.valid(election.suspended_days))
                }
                HomeExclusion::Decline => true,
                HomeExclusion::Full { taxpayer, spouse } => {
                    taxpayer.valid() && spouse.as_ref().is_none_or(HomeUse::valid)
                }
            }
    }
    // The separate space's share of the sale; None without one, or while
    // its depreciation is unknown.
    pub(crate) fn separate_sale(&self) -> Option<SeparateSale> {
        self.separate.as_ref()?.sale(self)
    }
    // Price, selling expenses and adjusted basis of the part that was the
    // home: the whole sale less the separate space's share (Pub. 523,
    // "Business Use or Rental of Home").
    pub(crate) fn home_amounts(&self) -> (i64, i64, i64) {
        let separate = self.separate_sale();
        let less =
            |whole: i64, part: fn(&SeparateSale) -> i64| whole - separate.as_ref().map_or(0, part);
        (
            less(self.price, |s| s.price),
            less(self.expenses, |s| s.expenses),
            less(self.basis, |s| s.basis),
        )
    }
    // Form 4797 line 2 for the separate space: its gain and its unrecaptured
    // section 1250 gain in whole dollars (the section 1231 owner works in
    // dollars).
    pub(crate) fn separate_form(&self) -> Option<(i64, i64)> {
        let sale = self.separate_sale()?;
        let gain = crate::kernel::ratio::dollars(sale.gain());
        Some((
            gain,
            phase::cap(crate::kernel::ratio::dollars(sale.unrecaptured()), [phase::floor(gain, 0)]),
        ))
    }
    // The section 1231 gain a rental's exchange recognizes (Form 8824 line 22 to
    // Form 4797 line 5) and its unrecaptured section 1250 gain, whole dollars.
    pub(crate) fn exchange_1231(&self, excluded: i64) -> Option<(i64, i64)> {
        self.exchange.as_ref().filter(|e| e.held_for == ExchangeUse::Rental)?;
        let gain = crate::kernel::ratio::dollars(self.exchange_form(excluded)?.line20);
        Some((gain, phase::cap(gain, [crate::kernel::ratio::dollars(self.depreciation_gain())])))
    }
    pub(crate) fn gain(&self) -> i64 {
        let (price, expenses, basis) = self.home_amounts();
        phase::floor(price - expenses - basis, 0)
    }
    pub(crate) fn depreciation_gain(&self) -> i64 {
        match self.use_history {
            HomeSaleUse::Personal => 0,
            HomeSaleUse::Adjusted { depreciation, .. } => phase::cap(depreciation, [self.gain()]),
        }
    }
    pub(crate) fn nonqualified_gain(&self) -> i64 {
        match self.use_history {
            HomeSaleUse::Personal => 0,
            HomeSaleUse::Adjusted { owned_days, nonqualified_days, .. } => {
                crate::kernel::ratio::fraction(
                    self.gain() - self.depreciation_gain(),
                    i64::from(nonqualified_days),
                    i64::from(owned_days.max(1)),
                )
            }
        }
    }
    fn excludable_gain(&self) -> i64 {
        self.gain() - self.depreciation_gain() - self.nonqualified_gain()
    }
    // Section 121(d)(10): no exclusion in the 5-year period beginning with the
    // date a like-kind exchange acquired this home or the property its basis
    // comes from; that period ends the day before the fifth anniversary. None
    // when no exchange can be in the period; the answer unknown when the earlier
    // owner's exchange is not known.
    fn like_kind_bar(&self) -> Option<PriorExchange> {
        let f = self.facts.as_ref()?;
        let (date, exchange) = f.received.exchange_date(f.acquired)?;
        (exchange != PriorExchange::No && f.closed < crate::dates::add_years(date, 5))
            .then_some(exchange)
    }
    pub(crate) fn like_kind_denied(&self) -> bool {
        self.like_kind_bar() == Some(PriorExchange::Yes)
    }
    // Tax the buyer withheld from a foreign seller's price (cents), the credit
    // on Form 1040 line 25c; zero when none is reported.
    pub(crate) fn firpta_credit(&self) -> i64 {
        self.facts
            .as_ref()
            .filter(|f| f.foreign_person)
            .and_then(|f| source_cents(&f.firpta_withheld))
            .unwrap_or(0)
    }
    // What an exchange of the home does not calculate with it.
    fn exchange_refusal(&self) -> Option<(&'static str, &'static str)> {
        self.exchange.as_ref()?;
        (self.separate.is_some() || self.payment.installment() || self.term != GainTerm::Long)
            .then_some((
                "ISSUE_HOME_SALE_EXCHANGE_COMBINED",
                crate::fixture_english!(
                    "An exchange of a home is calculated for a whole property held more than a year and paid in full; a separate business or rental space, an installment note or a short holding period is not calculated with it (Form 8824)."
                ),
            ))
    }
    // Form 8824 for the exchange of the home (Rev. Proc. 2005-14): the old
    // home's gain after the exclusion, the cash and other property received
    // less the exclusion being what is recognized.
    pub(crate) fn exchange_form(&self, excluded: i64) -> Option<ExchangeForm> {
        let exchange = self.exchange.as_ref()?;
        let (price, expenses, basis) = self.home_amounts();
        let boot = price - exchange.value;
        let line15 = phase::floor(boot - expenses, 0);
        let line18 = basis + phase::floor(expenses - boot, 0);
        let line19 = exchange.value + line15 - line18;
        let line20 = phase::floor(phase::cap(line15 - excluded, [line19]), 0);
        Some(ExchangeForm {
            line15,
            line16: exchange.value,
            line18,
            line19,
            line20,
            line24: line19 - line20,
            line25: line18 + line20 - line15 + excluded,
        })
    }
    fn depreciation_taken(&self) -> i64 {
        match self.use_history {
            HomeSaleUse::Adjusted { depreciation, .. } => depreciation,
            HomeSaleUse::Personal => 0,
        }
    }
    // Form 6252 lines 5-24 for an installment sale (Pub. 523, Installment Sale):
    // the lines' whole dollars, the section 121 exclusion on line 15 (None when
    // paid in full or without gain) and the gross profit on line 16 after it.
    pub(crate) fn installment(
        &self,
        excluded: i64,
    ) -> Option<(crate::kernel::installment::Schedule, i64)> {
        use crate::kernel::ratio::{dollars, fraction};
        let (price, expenses, basis) = self.home_amounts();
        let depreciation = self.depreciation_taken();
        let form = self.payment.form(
            dollars(price),
            dollars(basis + depreciation),
            dollars(depreciation),
            dollars(expenses),
            0,
            0,
        )?;
        let line15 = phase::cap(dollars(excluded), [form.gross_profit]);
        let gross_profit = form.gross_profit - line15;
        let income = phase::bound(
            fraction(form.excess_debt + form.payments, gross_profit, form.contract),
            0,
            gross_profit,
        );
        Some((
            crate::kernel::installment::Schedule {
                gross_profit,
                income,
                ordinary: 0,
                gain: income,
                ..form
            },
            line15,
        ))
    }
    fn installment_refusal(&self) -> Option<(&'static str, &'static str)> {
        let crate::SalePayment::Installment { assumed_debt, .. } = &self.payment else {
            return None;
        };
        if self.separate.is_some() {
            return Some((
                "ISSUE_HOME_SALE_INSTALLMENT_SEPARATE",
                crate::fixture_english!(
                    "An installment sale that also sells a separate business or rental space is not calculated (a separate Form 6252 for each part)."
                ),
            ));
        }
        if crate::kernel::ratio::dollars(self.price) <= crate::kernel::ratio::dollars(*assumed_debt)
        {
            return Some((
                "ISSUE_HOME_SALE_INSTALLMENT_CONTRACT_PRICE",
                crate::fixture_english!(
                    "An installment sale needs a contract price above zero: the loans the buyer took over must be less than the sale price (Form 6252 line 18)."
                ),
            ));
        }
        self.payment.refusal(self.price)
    }
    // What the separate space needs that this sale does not calculate.
    fn separate_refusal(&self) -> Option<(&'static str, &'static str)> {
        let part = self.separate.as_ref()?;
        if part.depreciation_cents().is_none() {
            return Some((
                "ISSUE_HOME_SALE_SEPARATE_DEPRECIATION_UNKNOWN",
                crate::fixture_english!(
                    "The depreciation on the separate business or rental space is not known, so its gain and the share of the price left for your home cannot be figured (Form 4797; Pub. 523). Your Schedule C, Schedule E and Form 4562 records show it."
                ),
            ));
        }
        if self.term != GainTerm::Long {
            return Some((
                "ISSUE_HOME_SALE_SEPARATE_SHORT_TERM",
                crate::fixture_english!(
                    "A separate business or rental space sold within a year of acquisition is ordinary income on Form 4797 Part II, which is not calculated."
                ),
            ));
        }
        matches!(part.kind, SeparateUse::Rental { carried_loss } if carried_loss > 0).then_some((
            "ISSUE_HOME_SALE_SEPARATE_PASSIVE_LOSS",
            crate::fixture_english!(
                "Passive rental losses carried over from earlier years are released by the sale of the rental space (Form 8582, section 469(g)), which is not calculated here."
            ),
        ))
    }
    fn full_limit(
        &self,
        status: FederalStatus,
        taxpayer: (u32, u32, bool),
        spouse: Option<(u32, u32, bool)>,
    ) -> Result<i64, (&'static str, &'static str)> {
        if (status == FederalStatus::Joint) != spouse.is_some() {
            return Err((
                "ISSUE_HOME_SALE_PROVIDE_BOTH_SPOUSES__HOME_HISTORIES_FOR_325E7C69",
                crate::fixture_english!(
                    "Provide both spouses' home histories for a joint return and only the taxpayer's otherwise."
                ),
            ));
        }
        let ownership = taxpayer.0 >= 730 || spouse.is_some_and(|(owned, _, _)| owned >= 730);
        let shares = u32::from(taxpayer.1 >= 730 && !taxpayer.2)
            + u32::from(spouse.is_some_and(|(_, lived, prior)| lived >= 730 && !prior));
        if !ownership || shares == 0 || self.term != GainTerm::Long {
            return Err((
                "ISSUE_HOME_SALE_THE_FULL_OWNERSHIP_RESIDENCE_EXCLUSION_IS_NOT_A9D4EF05",
                crate::fixture_english!(
                    "The full ownership/residence exclusion is not established; evaluate reduced or special exclusions before completing this sale."
                ),
            ));
        }
        Ok(phase::cap(
            self.excludable_gain(),
            [i64::from(shares) * crate::year_constants::HOME_SALE_EXCLUSION_SINGLE_CENTS],
        ))
    }
    pub(crate) fn excluded(
        &self,
        status: FederalStatus,
    ) -> Result<i64, (&'static str, &'static str)> {
        if let Some(message) = self.facts.as_ref().and_then(HomeSaleFacts::refusal) {
            return Err(message);
        }
        if let Some(message) = self
            .separate_refusal()
            .or_else(|| self.installment_refusal())
            .or_else(|| self.exchange_refusal())
        {
            return Err(message);
        }
        // A sale whose `ordinary` is No is left out before the rules run.
        if self.depreciation_gain() > 0 && self.term != GainTerm::Long {
            return Err((
                "ISSUE_HOME_SALE_RESOLVE_HOME_SALE_BASIS__NONQUALIFIED_USE__DEPRECIATION_84E3D92F",
                crate::fixture_english!(
                    "Depreciation on a home held one year or less is not calculated (Form 4797)."
                ),
            ));
        }
        // The page's election to report the whole gain keeps its answers; a home
        // got in a like-kind exchange within 5 years has no exclusion at all.
        if self.like_kind_denied() || self.facts.as_ref().is_some_and(|f| f.decline) {
            return Ok(0);
        }
        if self.like_kind_bar() == Some(PriorExchange::Unknown) {
            return Err((
                "ISSUE_HOME_SALE_PRIOR_EXCHANGE_UNKNOWN",
                crate::fixture_english!(
                    "Whether the earlier owner got this home in a like-kind exchange within 5 years of the sale is not known, so the exclusion cannot be told (section 121(d)(10), Pub. 523)."
                ),
            ));
        }
        if let HomeExclusion::Widowed {
            sale_date,
            death_date,
            history,
            late_prior,
            joint_tests_at_death,
        } = &self.exclusion
        {
            if history.prior_exclusion
                || history.owned < 730
                || history.care.residence(history.lived) < 730
                || self.term != GainTerm::Long
            {
                return Err((
                    "ISSUE_HOME_SALE_RESOLVE_THE_SURVIVING_SPOUSE_HOME_SALE_HISTORY__REMARRIAGE_D8822A28",
                    crate::fixture_english!(
                        "The surviving-spouse exclusion needs your own two-year ownership and residence, no excluded sale in the two years before and a sale held more than a year; otherwise evaluate a reduced exclusion."
                    ),
                ));
            }
            // Calendar anniversary, not a 730-day elapsed-time test. For a
            // February 29 death, the numeric comparison includes February 28
            // and excludes March 1 in the non-leap anniversary year.
            let higher = *sale_date <= *death_date + 20000 && !*late_prior && *joint_tests_at_death;
            return Ok(phase::cap(
                self.excludable_gain(),
                [if higher {
                    crate::year_constants::HOME_SALE_EXCLUSION_JOINT_CENTS
                } else {
                    crate::year_constants::HOME_SALE_EXCLUSION_SINGLE_CENTS
                }],
            ));
        }
        if let HomeExclusion::Reduced { reason, taxpayer, spouse } = &self.exclusion {
            if !reason.qualifies() {
                return Err((
                    "ISSUE_HOME_SALE_THE_SELECTED_EVENT_DOES_NOT_ESTABLISH_BB23B05D",
                    crate::fixture_english!(
                        "The selected event does not establish a reduced home-sale exclusion; review other facts and circumstances."
                    ),
                ));
            }
            if (status == FederalStatus::Joint) != spouse.is_some() {
                return Err((
                    "ISSUE_HOME_SALE_PROVIDE_SEPARATE_REDUCED_EXCLUSION_PERIODS_FOR_EACH_2AB2C8F5",
                    crate::fixture_english!(
                        "Provide separate reduced-exclusion periods for each joint filer, and no spouse periods on other returns."
                    ),
                ));
            }
            return Ok(phase::cap(
                self.excludable_gain(),
                [taxpayer.limit() + spouse.as_ref().map_or(0, ReducedHomeUse::limit)],
            ));
        }
        if let HomeExclusion::ReducedService { election, reason, taxpayer, spouse } =
            &self.exclusion
        {
            // An election that does not qualify suspends nothing.
            let suspended = election.qualifies();
            if !reason.qualifies() {
                return Err((
                    "ISSUE_HOME_SALE_THE_SELECTED_EVENT_DOES_NOT_ESTABLISH_BB23B05D",
                    crate::fixture_english!(
                        "The selected event does not establish a reduced home-sale exclusion; review other facts and circumstances."
                    ),
                ));
            }
            if (status == FederalStatus::Joint) != spouse.is_some() {
                return Err((
                    "ISSUE_HOME_SALE_PROVIDE_SEPARATE_REDUCED_EXCLUSION_PERIODS_FOR_EACH_2AB2C8F5",
                    crate::fixture_english!(
                        "Provide separate reduced-exclusion periods for each joint filer, and no spouse periods on other returns."
                    ),
                ));
            }
            return Ok(phase::cap(
                self.excludable_gain(),
                [taxpayer.limit(suspended) + spouse.as_ref().map_or(0, |p| p.limit(suspended))],
            ));
        }
        match &self.exclusion {
            HomeExclusion::Decline => Ok(0),
            HomeExclusion::Full { taxpayer, spouse } => {
                let filer = |p: &HomeUse| (p.owned, p.care.residence(p.lived), p.prior_exclusion);
                self.full_limit(status, filer(taxpayer), spouse.as_ref().map(filer))
            }
            HomeExclusion::ServiceSuspension { election, taxpayer, spouse, .. } => {
                // An election that does not qualify suspends nothing: only
                // the ordinary five-year window counts.
                let suspended = election.qualifies();
                let filer = |p: &FullDutyHomeUse| {
                    let (owned, lived) = if suspended {
                        (p.history.ownership(), p.history.residence())
                    } else {
                        p.history.ordinary()
                    };
                    (owned, lived, p.prior_exclusion)
                };
                self.full_limit(status, filer(taxpayer), spouse.as_ref().map(filer))
            }
            HomeExclusion::Widowed { .. }
            | HomeExclusion::Reduced { .. }
            | HomeExclusion::ReducedService { .. } => unreachable!(),
        }
    }
}

/// A situation the sale page does not calculate
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum HomeSaleSpecial {
    /// You are subject to the expatriation tax
    Expatriate,
    /// You took depreciation other than straight-line on the home
    /// Accelerated or bonus depreciation, or a Section 179 deduction, on the building or on property sold with it.
    NonStraightLineDepreciation,
    /// Another special situation
    Other,
}
impl HomeSaleSpecial {
    // Answers and unknowns: the notice of a situation the page does not
    // calculate, which leaves the sale out of the estimate; None for the
    // expatriation tax, an added tax that is refused by name instead.
    fn notice(self) -> Option<&'static str> {
        match self {
            Self::Expatriate => None,
            Self::NonStraightLineDepreciation => Some("WARN_HOME_SALE_DEPRECIATION_LEFT_OUT"),
            Self::Other => Some("WARN_HOME_SALE_OTHER_LEFT_OUT"),
        }
    }
}

/// Closing facts of a home sale entered on its page
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeSaleFacts {
    /// Date of closing
    #[libertas_date_only]
    pub closed: u32,
    /// Date you bought or received the home
    #[libertas_date_only]
    pub acquired: u32,
    /// How you got it
    pub received: HomeReceived,
    /// Date your late spouse got the home
    #[libertas_date_only]
    pub late_acquired: Option<u32>,
    /// Days your late spouse lived there while you did not
    pub late_lived: u32,
    /// A Form 1099-S reported the sale
    pub form_1099s: bool,
    /// Property or services were part of the price
    pub property_received: bool,
    /// Value of that property or those services, not in the box 2 price
    pub property_value: crate::SourceCents,
    /// The seller is a foreign person
    pub foreign_person: bool,
    /// Tax the buyer withheld from a foreign seller's price
    pub firpta_withheld: crate::SourceCents,
    /// Date of your earlier home sale whose gain you excluded
    #[libertas_date_only]
    pub earlier: Option<u32>,
    /// Date of your spouse's earlier home sale whose gain was excluded
    #[libertas_date_only]
    pub spouse_earlier: Option<u32>,
    /// Report the whole gain instead of excluding it
    pub decline: bool,
    /// A situation the page does not calculate
    pub special: Option<HomeSaleSpecial>,
}
impl HomeSaleFacts {
    fn valid(&self) -> bool {
        let date = crate::federal_rules::valid_date;
        date(self.closed)
            && self.closed / 10000 == 2026
            && date(self.acquired)
            && self.acquired < self.closed
            && [self.received.earlier_owner(), self.late_acquired]
                .iter()
                .flatten()
                .all(|d| date(*d) && *d <= self.acquired)
            && self.late_lived <= 1826
            && [&self.property_value, &self.firpta_withheld]
                .into_iter()
                .all(|v| source_cents(v).is_none_or(|v| (0..=100_000_000_000).contains(&v)))
            && [self.earlier, self.spouse_earlier]
                .iter()
                .flatten()
                .all(|d| date(*d) && *d < self.closed)
    }
    // Value of the property or services received, in cents, when known.
    fn property_cents(&self) -> Option<i64> {
        source_cents(&self.property_value)
    }
    // The page's named refusals: the expatriation tax (Form 8854), an added
    // tax the App does not calculate, and a price part or a withholding the
    // seller has not established.
    fn refusal(&self) -> Option<(&'static str, &'static str)> {
        if self.special == Some(HomeSaleSpecial::Expatriate) {
            return Some((
                "ISSUE_HOME_SALE_SPECIAL_2",
                crate::fixture_english!(
                    "Sale of your main home: the tax on a person who gave up citizenship or long-term residence (Form 8854, section 877A) is a return-wide tax on a dual-status return, which is not calculated."
                ),
            ));
        }
        if self.property_received && self.property_cents().is_none() {
            return Some((
                "ISSUE_HOME_SALE_PROPERTY_VALUE_UNKNOWN",
                crate::fixture_english!(
                    "The value of the property or services received as part of the price (1099-S box 6) is not known, so the amount realized cannot be figured (Pub. 523)."
                ),
            ));
        }
        (self.foreign_person && source_cents(&self.firpta_withheld).is_none()).then_some((
            "ISSUE_HOME_SALE_FIRPTA_WITHHOLDING_UNKNOWN",
            crate::fixture_english!(
                "The tax the buyer withheld from a foreign seller (Form 8288-A) is not known, so the credit on Form 1040 line 25c cannot be figured."
            ),
        ))
    }
    // The notice of a page situation the estimate leaves out (Answers and
    // unknowns): another named situation.
    pub(crate) fn left_out(&self) -> Option<&'static str> {
        self.special.and_then(HomeSaleSpecial::notice)
    }
}

// An established amount: reported, or established absent; None when unknown.
fn source_cents(fact: &crate::SourceCents) -> Option<i64> {
    match fact {
        crate::SourceCents::Reported { amount } => Some(*amount),
        crate::SourceCents::NotReported => Some(0),
        crate::SourceCents::Unavailable => None,
    }
}

impl HomeSale {
    // Pub. 523 "Reporting Your Home Sale" and the Schedule D instructions:
    // a sale goes on Form 8949 when a 1099-S was received or some gain is
    // taxable. A sale not entered on its page is treated as reported.
    pub(crate) fn reported(&self, taxable: i64) -> bool {
        taxable > 0 || self.facts.as_ref().is_none_or(|f| f.form_1099s)
    }
}

// Days from the earlier date to the later one.
fn days(from: u32, to: u32) -> i64 {
    i64::from(crate::dates::days_between(from, to))
}
// The 5-year test period of a 2026 sale is 1,826 days (it spans
// February 29, 2024).
const WINDOW: u32 = 1826;

/// How long you lived in the home as your main home
/// Count the 5 years before the sale. Periods need not be continuous; short absences such as vacations count, rental periods do not.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HomeLived {
    /// The whole time I owned it in those 5 years
    WholeTime,
    /// Part of that time
    Days {
        /// Time lived there as your main home in the 5 years before the sale
        // Section 121 tax structure: days within the five-year ownership/use test period.
        #[libertas_number(min = 0, max = 1826)]
        #[libertas_physical_unit("day")]
        days: u32,
    },
    /// Part of that time, then in a licensed care facility
    /// You became physically or mentally unable to care for yourself, and the facility is licensed to care for people with your condition. Facility time counts only if you both owned and lived in the home for at least 365 days.
    CareFacility {
        /// Time lived in the home in the 5 years before the sale
        // Section 121 tax structure: days within the five-year ownership/use test period.
        #[libertas_number(min = 0, max = 1826)]
        #[libertas_physical_unit("day")]
        home_days: u32,
        /// Time in the facility in those 5 years while you still owned the home
        // Section 121 tax structure: days within the five-year ownership/use test period.
        #[libertas_number(min = 0, max = 1826)]
        #[libertas_physical_unit("day")]
        facility_days: u32,
        /// Time both owning and living in the home in those 5 years
        /// Already included in home-residence time, not additional.
        // Section 121 tax structure: days within the five-year ownership/use test period.
        #[libertas_number(min = 0, max = 1826)]
        #[libertas_physical_unit("day")]
        owned_residence_days: u32,
    },
}
impl HomeLived {
    // Residence days in the test period and the facility time, as stored.
    fn record(&self, window: u32) -> (u32, HomeCare) {
        match self {
            Self::WholeTime => (window, HomeCare::NONE),
            Self::Days { days } => ((*days).min(window), HomeCare::NONE),
            Self::CareFacility { home_days, facility_days, owned_residence_days } => (
                *home_days,
                HomeCare { days: *facility_days, owned_residence_days: *owned_residence_days },
            ),
        }
    }
    // Whether the days fit the time owned (both sides:
    // HOME_LIVED_ABOVE_OWNED).
    fn fits(&self, owned: i64, extra: u32) -> bool {
        let fits = |n: u32| n + extra <= WINDOW && i64::from(n + extra) <= owned;
        match self {
            Self::WholeTime => true,
            Self::Days { days } => fits(*days),
            Self::CareFacility { home_days, facility_days, owned_residence_days } => {
                fits(home_days + facility_days) && owned_residence_days <= home_days
            }
        }
    }
}

/// Excluded gain on another home sold in the 2 years before this sale
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum EarlierHomeSale {
    /// No
    No,
    /// Yes
    Yes {
        /// Date that home was sold
        #[libertas_date_only]
        sold: u32,
    },
}
impl EarlierHomeSale {
    fn date(&self) -> Option<u32> {
        match self {
            Self::No => None,
            Self::Yes { sold } => Some(*sold),
        }
    }
}

/// Your spouse
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
#[libertas_inline_choices]
pub enum HomeSpouse {
    /// Not a joint return
    NotJoint,
    /// On this joint return
    Joint {
        /// How long your spouse lived in the home as a main home
        #[libertas_default(WholeTime)]
        lived: HomeLived,
        /// Your spouse excluded gain on another home sold in the 2 years before this sale
        #[libertas_default(No)]
        earlier_sale: EarlierHomeSale,
    },
}
impl HomeSpouse {
    // The page's allowed set: the spouse's facts exactly on a joint return.
    pub(crate) fn allowed(joint: bool) -> alloc::vec::Vec<i32> {
        alloc::vec![if joint {
            libertas_macros::variant_index!(HomeSpouse::Joint)
        } else {
            libertas_macros::variant_index!(HomeSpouse::NotJoint)
        } as i32]
    }
    pub(crate) fn joint(&self) -> bool {
        matches!(self, Self::Joint { .. })
    }
}

/// Whether the earlier owner got the home in a like-kind exchange
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum PriorExchange {
    /// No
    No,
    /// Yes
    Yes,
    /// I do not know
    /// Until it is known, whether the exclusion is available within 5 years of that exchange cannot be told.
    Unknown,
}

/// How you got the home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HomeReceived {
    /// Bought or built it
    Bought,
    /// Inherited it
    /// Enter the date of death above as the date you got it. An inherited home is long-term property whenever you sell it, and its basis is its value at death. Your own ownership and use start at the death, apart from a surviving spouse's.
    Inherited,
    /// Received it as a gift
    /// Your basis is the giver's adjusted basis, so enter that as the adjusted basis. The giver's holding period is added to yours, but your ownership and use for the exclusion start on the day you received it.
    Gift {
        /// Date the giver got it
        #[libertas_date_only]
        given_acquired: u32,
        /// The giver got it in a like-kind exchange
        /// Your basis comes from the giver's, so the 5-year bar of section 121(d)(10) runs from the exchange (Pub. 523).
        got_by_exchange: PriorExchange,
    },
    /// Received it from your spouse or former spouse
    /// A transfer between spouses, or to a former spouse because of a divorce, has no gain or loss, and your basis is your spouse's adjusted basis (section 1041). Your spouse's ownership counts as yours.
    FromSpouse {
        /// Date your spouse or former spouse got it
        #[libertas_date_only]
        spouse_acquired: u32,
        /// Time your former spouse lived there under your divorce or separation agreement while you did not
        /// That time counts as time you lived there. Enter zero if none.
        // Section 121(d)(3)(B) tax structure: days within the five-year ownership/use test period.
        #[libertas_number(min = 0, max = 1826)]
        #[libertas_default(0)]
        #[libertas_physical_unit("day")]
        agreement_days: u32,
        /// Your spouse or former spouse got it in a like-kind exchange
        /// Your basis comes from theirs, so the 5-year bar of section 121(d)(10) runs from the exchange (Pub. 523).
        got_by_exchange: PriorExchange,
    },
    /// Got it in a like-kind exchange
    /// Section 121(d)(10): no gain can be excluded when you sell within 5 years of the exchange. Your basis is that of the property you gave up, adjusted for the exchange.
    LikeKindExchange {
        /// Date you got the property you gave up
        /// Its holding period is added to yours.
        #[libertas_date_only]
        given_up_acquired: u32,
    },
}
impl HomeReceived {
    // The earlier owner's acquisition date that adds to your holding period.
    fn earlier_owner(&self) -> Option<u32> {
        match self {
            Self::Gift { given_acquired: date, .. }
            | Self::LikeKindExchange { given_up_acquired: date }
            | Self::FromSpouse { spouse_acquired: date, .. } => Some(*date),
            Self::Bought | Self::Inherited => None,
        }
    }
    // The date the property was got in a like-kind exchange that bars the
    // exclusion for five years (section 121(d)(10)): this home's own, or the
    // giver's or spouse's whose basis this home's comes from. Known whether it
    // was an exchange, or not.
    fn exchange_date(&self, acquired: u32) -> Option<(u32, PriorExchange)> {
        match self {
            Self::LikeKindExchange { .. } => Some((acquired, PriorExchange::Yes)),
            Self::Gift { given_acquired: date, got_by_exchange }
            | Self::FromSpouse { spouse_acquired: date, got_by_exchange, .. } => {
                Some((*date, *got_by_exchange))
            }
            Self::Bought | Self::Inherited => None,
        }
    }
    // Days a former spouse's use adds to yours.
    fn use_days(&self) -> u32 {
        match self {
            Self::FromSpouse { agreement_days, .. } => *agreement_days,
            _ => 0,
        }
    }
}

/// Why you sold
/// Choose the event only when you or your spouse did not live there 2 of the 5 years, or excluded gain on another home in the 2 years before.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HomeSaleReason {
    /// No special reason: we meet the 2-year tests
    TwoYears,
    /// A move for work, health or an unforeseen event
    /// The event happened while you owned and lived in the home and was the main reason for the sale.
    Event {
        /// What happened
        reason: HomeSaleTrigger,
    },
}

/// Licensed care-facility residence in the added service period
/// Count facility residence only when you became physically or mentally unable to care for yourself and the facility is licensed to care for people with your condition.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeServiceCare {
    /// Time in the facility in the added period while still owning the home
    /// Exclude time already counted as residence in the home.
    // Section 121(d)(9) tax structure: the added service period is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
    #[libertas_default(0)]
    #[libertas_physical_unit("day")]
    pub facility_days: u32,
    /// Time both owning and living in the home in the added period
    /// Already included in the added home-residence time, not additional.
    // Section 121(d)(9) tax structure: the added service period is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
    #[libertas_default(0)]
    #[libertas_physical_unit("day")]
    pub owned_residence_days: u32,
}

/// Qualified official extended duty
/// Elect to suspend the 5-year ownership and use test while you or your spouse served on qualified official extended duty, for up to 10 years in all and for one property at a time. The test then looks back that many days further before the 5 years. You make the election by filing this return.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeService {
    /// Service and election
    pub election: HomeServiceElection,
    /// Time you lived there as your main home in the added period
    /// The added period is the time of service elected, just before the 5 years. Do not count rental use.
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
    #[libertas_default(0)]
    #[libertas_physical_unit("day")]
    pub added_lived: u32,
    /// Your licensed care-facility residence in the added period
    /// [NullText]
    /// No qualifying care-facility residence in the added period
    /// [SetText]
    /// I had qualifying care-facility residence in the added period
    #[libertas_default($null)]
    pub added_care: Option<HomeServiceCare>,
    /// Time your spouse lived there as a main home in the added period
    /// Joint return only; leave empty otherwise.
    /// [NullText]
    /// Not a joint return
    /// [SetText]
    /// My spouse lived there for this long
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
    #[libertas_default($null)]
    #[libertas_physical_unit("day")]
    pub spouse_added_lived: Option<u32>,
    /// Your spouse's licensed care-facility residence in the added period
    /// Joint return only; leave empty otherwise.
    /// [NullText]
    /// No qualifying care-facility residence for my spouse in the added period
    /// [SetText]
    /// My spouse had qualifying care-facility residence in the added period
    #[libertas_default($null)]
    pub spouse_added_care: Option<HomeServiceCare>,
}

/// Rental or business use of the home
/// Leave both at zero when the home was only ever your home.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeRentalUse {
    /// Depreciation allowed or allowable after May 6, 1997
    /// Straight-line depreciation on the home, including amounts you could have deducted but did not. Enter zero if none.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub depreciation: i64,
    /// Time after 2008 that nobody used it as a main home
    /// Do not count the time after you moved out within the 5 years before the sale, temporary absences of up to 2 years for work, health or an unforeseen event, or qualified official extended duty. Enter zero if none.
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    #[libertas_physical_unit("day")]
    pub nonqualified_days: u32,
    /// Separate business or rental space
    /// Leave empty when all business or rental use was inside your own living space, such as a home office, or when there was none.
    /// [NullText]
    /// No separate business or rental space was sold with the home
    /// [SetText]
    /// A separate business or rental space was sold with the home
    #[libertas_default($null)]
    pub separate: Option<SeparateHomePart>,
}

/// Something else
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum HomeSaleMore {
    /// Nothing else
    Nothing,
    /// My spouse died before the sale, and I have not remarried
    SurvivingSpouse {
        /// Date your spouse died
        #[libertas_date_only]
        died: u32,
        /// Just before the death, you both had lived there 2 of the previous 5 years, and neither of you had excluded gain on another home in the 2 years before
        #[libertas_default(false)]
        both_qualified: bool,
        /// Your late spouse excluded gain on another home sold in the 2 years before this sale
        #[libertas_default(false)]
        spouse_earlier_sale: bool,
        /// Date your late spouse got the home
        /// Fill this in when your spouse owned it before the date above: you are treated as owning it for that time too.
        /// [NullText]
        /// The date above is the earliest
        /// [SetText]
        /// My late spouse got it on
        #[libertas_date_only]
        #[libertas_default($null)]
        late_acquired: Option<u32>,
        /// Time your late spouse lived there as a main home while you did not
        /// That time counts as time you lived there. Enter zero if none.
        // Section 121(d)(2) tax structure: days within the five-year ownership/use test period.
        #[libertas_number(min = 0, max = 1826)]
        #[libertas_default(0)]
        #[libertas_physical_unit("day")]
        late_lived: u32,
    },
    /// Report the whole gain instead of excluding it
    /// For example, to keep the exclusion for another home you sell within 2 years.
    Decline,
    /// A situation this page does not calculate
    Refused {
        /// Which situation
        situation: HomeSaleSpecial,
    },
}

/// Form 1099-S
/// Proceeds From Real Estate Transactions. One for each printed form.
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read one Form 1099-S exactly as printed. Copy the calendar year, the CORRECTED checkbox, FILER and TRANSFEROR party blocks, filer telephone number and account number. Preserve identity masking, punctuation and leading zeros; leave absent printed text empty. Read boxes 1, 2a, 2b, 2c, 3, 4, 6, 7 and 8a through 8d in form order. Box 5 is reserved for future use. Report a blank closing date as an error. Use zero for blank numeric boxes and false for unchecked checkboxes. Copy box 8c as a numeric quantity, including fractional units; leave a blank box 8d date null. Do not sum distinct forms, derive amounts, or infer basis, expenses, residence, ownership or tax treatment. Report illegible or conflicting values as errors. Check the printed year against the supplied year and report a mismatch. Copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    duplicate_identity = "@.filer.federal_id,@.transferor.federal_id,@.calendar_year,@.box_1_date_of_closing,@.box_3_address_or_legal_description,@.box_2a_total_gross_proceeds,@.box_2b_cash_gross_proceeds,@.box_2c_digital_asset_gross_proceeds,@.box_4_buyers_part_of_real_estate_tax,@.account_number",
    form_date_order = "MonthDayYear"
)]
pub struct Form1099SPaper {
    /// For calendar year
    #[libertas_llm_doc_field(allow_empty = false)]
    pub calendar_year: u32,
    /// CORRECTED (if checked)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub corrected: bool,
    /// FILER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub filer: crate::PrintedParty,
    /// Telephone number
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub filer_telephone: String,
    /// TRANSFEROR'S name, TIN and address
    #[libertas_llm_doc_field]
    pub transferor: crate::PrintedParty,
    /// Account number (see instructions)
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub account_number: String,
    /// Box 1 Date of closing
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub box_1_date_of_closing: u32,
    /// Box 2a Total gross proceeds
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_2a_total_gross_proceeds: i64,
    /// Box 2b Cash gross proceeds
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_2b_cash_gross_proceeds: i64,
    /// Box 2c Digital asset gross proceeds
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_2c_digital_asset_gross_proceeds: i64,
    /// Box 3 Address (including city, state, and ZIP code) or legal description
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_3_address_or_legal_description: String,
    /// Box 4 Buyer's part of real estate tax
    #[libertas_llm_doc_field]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub box_4_buyers_part_of_real_estate_tax: i64,
    /// Box 6 If checked, transferor received or will receive services or property (other than cash, notes, or digital assets) as part of the consideration
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_6_property_or_services: bool,
    /// Box 7 If checked, transferor is a foreign person (nonresident alien, foreign partnership, foreign estate, or foreign trust)
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub box_7_foreign_person: bool,
    /// Box 8a Code for digital asset received, or to be received, as consideration
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_8a_digital_asset_code: String,
    /// Box 8b Name of digital asset received, or to be received, as consideration
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub box_8b_digital_asset_name: String,
    /// Box 8c Number of digital asset units received, or to be received, as consideration
    #[libertas_llm_doc_field]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub box_8c_digital_asset_units: f64,
    /// Box 8d Date digital asset received, or to be received, as consideration
    /// [NullText]
    /// Box 8d is blank
    /// [SetText]
    /// Date printed in box 8d
    #[libertas_llm_doc_field]
    #[libertas_date_only]
    #[libertas_default($null)]
    pub box_8d_digital_asset_date: Option<u32>,
}
// The schema admits finite numeric quantities; the stored document union needs Eq.
impl Eq for Form1099SPaper {}

impl From<&Form1099SPaper> for Form1099S {
    fn from(paper: &Form1099SPaper) -> Self {
        let Form1099SPaper {
            // Printed metadata and the newly collected boxes have no calculation consumer.
            calendar_year: _,
            corrected: _,
            filer,
            filer_telephone: _,
            transferor,
            account_number: _,
            box_1_date_of_closing,
            box_2a_total_gross_proceeds,
            box_2b_cash_gross_proceeds: _,
            box_2c_digital_asset_gross_proceeds: _,
            box_3_address_or_legal_description,
            box_4_buyers_part_of_real_estate_tax: _,
            box_6_property_or_services,
            box_7_foreign_person,
            box_8a_digital_asset_code: _,
            box_8b_digital_asset_name: _,
            box_8c_digital_asset_units: _,
            box_8d_digital_asset_date: _,
        } = paper;
        Self {
            filer: filer.clone(),
            transferor: transferor.clone(),
            address: box_3_address_or_legal_description.clone(),
            closed: *box_1_date_of_closing,
            proceeds: *box_2a_total_gross_proceeds,
            property_received: *box_6_property_or_services,
            foreign_person: *box_7_foreign_person,
        }
    }
}

/// Sale of your main home (Form 1099-S)
/// One for each home you sold in 2026, with its Form 1099-S if you received one. On a joint return, enter a home you sold together once.
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into filer, transferor. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read the Form 1099-S exactly. Check its tax year against the supplied year; report a mismatch. Return the address or legal description of the property, or an empty string when it is blank. The date of closing and gross proceeds must show values; report a blank one as an error. When several 1099-S forms for the same property are supplied, their dates of closing and properties must agree (report a conflict as an error); return the sum of their gross proceeds. Return true for the property or services received as part of the consideration checkbox, and for the transferor is a foreign person checkbox, only when checked. Report illegible or conflicting values as errors. Do not infer when the home was bought, its basis, selling expenses, how long anyone lived in it, why it was sold, or other facts absent from the 1099-S. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true,
    form_date_order = "MonthDayYear"
)]
pub struct Form1099S {
    /// FILER'S name, TIN and address
    #[libertas_llm_doc_field]
    pub filer: crate::PrintedParty,
    /// TRANSFEROR'S name, TIN and address
    #[libertas_llm_doc_field]
    pub transferor: crate::PrintedParty,
    /// Box 3 address of the property
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub address: String,
    /// Box 1 date of closing
    /// Without a 1099-S, the closing date on your settlement statement.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub closed: u32,
    /// Box 2a gross proceeds
    /// Cash, notes and loans the buyer took over, not the value of any property or services you received: those go in box 6's value below. Without a 1099-S, the same from your settlement statement. On a joint return with a 1099-S for each of you, add the two.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
    /// Box 6 property or services received as part of the price
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub property_received: bool,
    /// Box 7 the transferor is a foreign person
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub foreign_person: bool,
}

fn closing_year_issue(closed: u32) -> Option<crate::validation::ValidationIssue> {
    (closed / 10000 != 2026).then(|| {
        crate::validation::ValidationIssue::new("HOME_SALE_NOT_IN_CURRENT_YEAR").at_field("closed")
    })
}

impl Form1099SPaper {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        closing_year_issue(self.box_1_date_of_closing)
            .map(|issue| issue.at_field("box_1_date_of_closing"))
    }
}

impl Form1099S {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        closing_year_issue(self.closed)
    }
    // The entry label: the box 3 address as typed.
    pub(crate) fn label(&self) -> String {
        self.address.clone()
    }
}

/// The facts about the home you sold
/// What the Pub. 523 worksheet asks beyond the Form 1099-S: when you got the home, what it cost, how long you lived in it and why you sold.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeSaleAnswers {
    /// Value of the property or services received
    /// Their fair market value, which box 2a leaves out and the amount realized includes (Pub. 523): enter it here and not in box 2a, so it is counted once. Used only when box 6 is checked.
    #[libertas_default(NotReported)]
    pub property_value: crate::SourceCents,
    /// Tax the buyer withheld
    /// The amount on the stamped Form 8288-A the buyer gave you, claimed on Form 1040 line 25c. Used only when box 7 is checked.
    #[libertas_default(NotReported)]
    pub firpta_withheld: crate::SourceCents,
    /// You received a Form 1099-S for this sale
    /// Without one, a sale whose whole gain is excluded is not reported on your return.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(true)]
    pub form_1099s: bool,
    /// Date you bought or received the home
    /// If you and your spouse got it at different times, the earlier date.
    #[libertas_date_only]
    pub acquired: u32,
    /// How you got it
    #[libertas_default(Bought)]
    pub received: HomeReceived,
    /// Selling expenses
    /// Commissions, advertising, legal fees and the closing costs you paid as the seller. Not the mortgage payoff or moving costs.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub expenses: i64,
    /// How you were paid
    #[libertas_default(InFull)]
    pub payment: crate::SalePayment,
    /// Exchange for like-kind property
    /// Leave empty for an ordinary sale.
    /// [NullText]
    /// I sold the home
    /// [SetText]
    /// I exchanged the home for like-kind property
    #[libertas_default($null)]
    pub exchange: Option<HomeExchange>,
    /// Adjusted basis
    /// What you paid for the home with its purchase closing costs, plus improvements, less depreciation, casualty losses and energy credits. Not the mortgage balance.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// How long you lived in it as your main home
    #[libertas_default(WholeTime)]
    pub lived: HomeLived,
    /// You excluded gain on another home sold in the 2 years before this sale
    #[libertas_default(No)]
    pub earlier_sale: EarlierHomeSale,
    /// Your spouse
    /// Asked on a joint return.
    pub spouse: HomeSpouse,
    /// Why you sold
    #[libertas_default(TwoYears)]
    pub reason: HomeSaleReason,
    /// Qualified official extended duty
    /// Leave empty unless you elect to suspend the 5-year test for service by you or your spouse.
    /// [NullText]
    /// I do not elect to suspend the 5-year test
    /// [SetText]
    /// I elect to suspend the 5-year test for service
    #[libertas_default($null)]
    pub service: Option<HomeService>,
    /// Rental or business use
    pub use_history: HomeRentalUse,
    /// Something else
    #[libertas_default(Nothing)]
    pub more: HomeSaleMore,
}

/// What you sold
/// Form 1099-S reports a real estate sale without saying how you used the property.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[expect(
    clippy::large_enum_variant,
    reason = "The stored answer carries its complete main-home facts"
)]
pub enum RealEstateSaleUse {
    /// Your main home
    /// The home you lived in as your main home. The questions below figure the gain you can exclude (Pub. 523).
    MainHome(HomeSaleAnswers),
    /// A second home, land or other real estate
    /// Property you used yourself or held for investment, not in a business or rental.
    Other(OtherRealEstateSale),
    /// Rental, business or farm property
    /// Enter the sale on that property's rental, business or farm page with box 2a as its sale price; it is counted there, not here.
    Business,
}

/// The sale of a second home, land or other real estate
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct OtherRealEstateSale {
    /// How you held it
    pub held: RealEstateHolding,
    /// How long you owned it
    /// Inherited property is long-term. A gift adds the giver's time.
    pub term: GainTerm,
    /// Adjusted basis
    /// What you paid with its purchase closing costs, plus improvements, less depreciation and casualty losses.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Selling expenses
    /// Commissions, advertising, legal fees and the closing costs you paid as the seller.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub expenses: i64,
}

/// How you held the real estate
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RealEstateHolding {
    /// For your own use
    /// A second home, vacation home or other real estate you used yourself. A gain is taxable; a loss is not deductible.
    Personal,
    /// For investment
    /// Land or other real estate held for investment.
    Investment,
}

#[cfg(test)]
impl RealEstateSaleUse {
    /// The main-home facts of a test fixture.
    pub(crate) fn facts_mut(&mut self) -> &mut HomeSaleAnswers {
        match self {
            Self::MainHome(answers) => answers,
            Self::Other(_) | Self::Business => panic!("not a main-home sale"),
        }
    }
}

#[cfg(test)]
impl MainHomeSale {
    /// The stored main-home facts of a test fixture.
    pub(crate) fn facts_mut(&mut self) -> &mut HomeSaleAnswers {
        self.answers.as_mut().expect("answered sale").facts_mut()
    }
}

impl OtherRealEstateSale {
    // Form 8949 box C or F (Form 1099-S is not a broker statement): box 2a
    // as proceeds (column d), the basis (column e) and the selling expenses
    // as code E (column g); a personal-use loss is not deductible.
    pub(crate) fn sale(&self, proceeds: i64) -> crate::FederalSale {
        crate::FederalSale {
            term: self.term,
            collectible: false,
            proceeds,
            basis: Some(self.basis),
            amt: crate::SaleAmt::SameAsRegular,
            withholding: 0,
            use_kind: match self.held {
                RealEstateHolding::Personal => crate::SaleUse::Personal,
                RealEstateHolding::Investment => crate::SaleUse::Investment { wash_sale: 0 },
            },
            special: crate::SaleSpecial::NoneOfThese,
            adjustment: if self.expenses > 0 {
                crate::SaleAdjustment::SellingExpenses { amount: self.expenses }
            } else {
                crate::SaleAdjustment::None
            },
            basis_reporting: crate::BasisReporting::NoForm1099B,
            digital_asset: false,
            market_discount: 0,
            several_sales: false,
        }
    }
}

/// Source of a main-home sale
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HomeSaleSource {
    /// Form 1099-S as printed
    Paper(Form1099SPaper),
    /// Sale from your own records, with no Form 1099-S
    Records(Form1099S),
}

impl HomeSaleSource {
    /// The actual printed form, when one was received.
    pub fn paper(&self) -> Option<&Form1099SPaper> {
        match self {
            Self::Paper(paper) => Some(paper),
            Self::Records(_) => None,
        }
    }
    /// The actual printed form to correct, when one was received.
    pub fn paper_mut(&mut self) -> Option<&mut Form1099SPaper> {
        match self {
            Self::Paper(paper) => Some(paper),
            Self::Records(_) => None,
        }
    }
    fn form_1099s(&self) -> bool {
        matches!(self, Self::Paper(_))
    }
}

impl From<&HomeSaleSource> for Form1099S {
    fn from(source: &HomeSaleSource) -> Self {
        match source {
            HomeSaleSource::Paper(paper) => Self::from(paper),
            HomeSaleSource::Records(records) => records.clone(),
        }
    }
}

/// Accepted Form 1099-S sale or sale of your main home
/// The printed Form 1099-S or your own sale records, and what was sold with its facts, kept apart.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MainHomeSale {
    /// Form 1099-S or your own sale records
    pub form: HomeSaleSource,
    /// What you sold
    /// Not answered until you give it; the sale is left out of the estimate meanwhile. A sale from your own records is your main home.
    /// [NullText]
    /// What you sold is not answered yet
    /// [SetText]
    /// I sold
    #[libertas_default($null)]
    pub answers: Option<RealEstateSaleUse>,
}

impl MainHomeSale {
    /// A newly entered form: what was sold is not given yet.
    pub(crate) fn printed(form: Form1099SPaper) -> Self {
        Self { form: HomeSaleSource::Paper(form), answers: None }
    }
    /// A sale entered from records when no Form 1099-S was received.
    pub(crate) fn records(form: Form1099S, answers: HomeSaleAnswers) -> Self {
        let mut source = Self { form: HomeSaleSource::Records(form), answers: None };
        source.set_use(Some(RealEstateSaleUse::MainHome(answers)));
        source
    }
    /// The transaction fields shared by printed forms and the person's own records.
    pub(crate) fn form(&self) -> Form1099S {
        Form1099S::from(&self.form)
    }
    /// What was sold, with reporting following the source of the sale.
    pub(crate) fn sale_use(&self) -> Option<RealEstateSaleUse> {
        self.answers.clone().map(|mut sold| {
            if let RealEstateSaleUse::MainHome(answers) = &mut sold {
                answers.form_1099s = self.form.form_1099s();
            }
            sold
        })
    }
    /// The main-home facts, when the sale was your main home.
    pub(crate) fn answers(&self) -> Option<HomeSaleAnswers> {
        match self.sale_use()? {
            RealEstateSaleUse::MainHome(answers) => Some(answers),
            RealEstateSaleUse::Other(_) | RealEstateSaleUse::Business => None,
        }
    }
    /// A Form 1099-S sale answered as property other than your main home.
    pub(crate) fn other_property(&self) -> bool {
        matches!(self.answers, Some(RealEstateSaleUse::Other(_) | RealEstateSaleUse::Business))
    }
    pub(crate) fn set_use(&mut self, mut sold: Option<RealEstateSaleUse>) {
        if let Some(RealEstateSaleUse::MainHome(answers)) = &mut sold {
            answers.form_1099s = self.form.form_1099s();
        }
        self.answers = sold;
    }
    /// The topic answers survive a correction; reporting follows the corrected source.
    pub(crate) fn keep_answers(&mut self, accepted: &Self) {
        self.set_use(accepted.sale_use());
    }
    // The first relationship between the given facts that fails: the spouse
    // answers exactly on a joint return (HOME_SPOUSE_NEEDED /
    // HOME_SPOUSE_NOT_JOINT), then the worksheet's own rules.
    pub(crate) fn answers_issue(&self, joint: bool) -> Option<crate::validation::ValidationIssue> {
        let answers = self.answers()?;
        if answers.spouse.joint() != joint {
            return Some(
                crate::validation::ValidationIssue::new(if joint {
                    "HOME_SPOUSE_NEEDED"
                } else {
                    "HOME_SPOUSE_NOT_JOINT"
                })
                .at_field("spouse"),
            );
        }
        HomeSaleWorksheet::new(&self.form(), &answers).issue()
    }
    // Off a joint return the spouse's facts are not asked: erases them
    // (the walk's prune); true when something was erased.
    pub(crate) fn drop_spouse(&mut self) -> bool {
        let Some(RealEstateSaleUse::MainHome(answers)) = self.answers.as_mut() else {
            return false;
        };
        let mut removed = answers.spouse.joint();
        answers.spouse = HomeSpouse::NotJoint;
        if let Some(service) = &mut answers.service {
            removed |= service.spouse_added_lived.take().is_some();
            removed |= service.spouse_added_care.take().is_some();
        }
        removed
    }
}

// One-way: the source and the answers together are the worksheet the
// calculation reads; the sale left unanswered is left out of the estimate.
impl From<&MainHomeSale> for HomeSale {
    fn from(source: &MainHomeSale) -> Self {
        let form = &source.form();
        match source.answers() {
            Some(answers) => HomeSaleWorksheet::new(form, &answers).record(),
            None => {
                let Form1099S {
                    filer: _,
                    transferor: _,
                    address: _,
                    closed: _,
                    proceeds,
                    property_received: _,
                    foreign_person: _,
                } = form;
                HomeSale {
                    price: *proceeds,
                    expenses: 0,
                    basis: 0,
                    term: GainTerm::Short,
                    use_history: HomeSaleUse::Personal,
                    separate: None,
                    payment: crate::SalePayment::InFull,
                    exchange: None,
                    exclusion: HomeExclusion::Decline,
                    // Without the home facts nothing is calculated: the notice
                    // names the sale as left out.
                    ordinary: false,
                    facts: None,
                }
            }
        }
    }
}

// The paper and the answers as the sale page once held them in one record.
struct HomeSaleWorksheet {
    closed: u32,
    proceeds: i64,
    property_received: bool,
    property_value: crate::SourceCents,
    foreign_person: bool,
    firpta_withheld: crate::SourceCents,
    form_1099s: bool,
    acquired: u32,
    received: HomeReceived,
    expenses: i64,
    payment: crate::SalePayment,
    exchange: Option<HomeExchange>,
    basis: i64,
    lived: HomeLived,
    earlier_sale: EarlierHomeSale,
    spouse: HomeSpouse,
    reason: HomeSaleReason,
    service: Option<HomeService>,
    use_history: HomeRentalUse,
    more: HomeSaleMore,
}

// One filer's test-period facts as the stored exclusion keeps them.
struct FilerUse {
    owned: u32,
    lived: u32,
    care: HomeCare,
    prior: bool,
    since_prior: u32,
}

// The date your ownership counts from: yours, a spouse's or former spouse's
// transfer to you (section 121(d)(3)(A)) and a surviving spouse's late spouse
// (section 121(d)(2)).
fn ownership_start(acquired: u32, received: &HomeReceived, late_acquired: Option<u32>) -> u32 {
    let spouse = match received {
        HomeReceived::FromSpouse { spouse_acquired, .. } => Some(*spouse_acquired),
        _ => None,
    };
    [Some(acquired), spouse, late_acquired].into_iter().flatten().min().unwrap_or(acquired)
}

impl HomeSaleWorksheet {
    fn new(form: &Form1099S, answers: &HomeSaleAnswers) -> Self {
        let Form1099S {
            // The parties and the address name the document; no figure reads them.
            filer: _,
            transferor: _,
            address: _,
            closed,
            proceeds,
            property_received,
            foreign_person,
        } = form;
        let HomeSaleAnswers {
            property_value,
            firpta_withheld,
            form_1099s,
            acquired,
            received,
            expenses,
            payment,
            exchange,
            basis,
            lived,
            earlier_sale,
            spouse,
            reason,
            service,
            use_history,
            more,
        } = answers;
        Self {
            closed: *closed,
            proceeds: *proceeds,
            property_received: *property_received,
            property_value: property_value.clone(),
            foreign_person: *foreign_person,
            firpta_withheld: firpta_withheld.clone(),
            form_1099s: *form_1099s,
            acquired: *acquired,
            received: received.clone(),
            expenses: *expenses,
            payment: payment.clone(),
            exchange: exchange.clone(),
            basis: *basis,
            lived: lived.clone(),
            earlier_sale: earlier_sale.clone(),
            spouse: spouse.clone(),
            reason: reason.clone(),
            service: service.clone(),
            use_history: use_history.clone(),
            more: more.clone(),
        }
    }
    fn late(&self) -> (Option<u32>, u32) {
        match self.more {
            HomeSaleMore::SurvivingSpouse { late_acquired, late_lived, .. } => {
                (late_acquired, late_lived)
            }
            _ => (None, 0),
        }
    }
    // Days owned in all (tacked ownership included), and in the 5-year test period.
    fn owned(&self) -> i64 {
        days(ownership_start(self.acquired, &self.received, self.late().0), self.closed)
    }
    // Days a spouse's or former spouse's use adds to yours.
    fn use_days(&self) -> u32 {
        self.received.use_days() + self.late().1
    }
    // Held more than one year (sections 1223(2) and (9)): an inherited home
    // always, a gift or spouse transfer from the earlier owner's date.
    fn long_term(&self) -> bool {
        matches!(self.received, HomeReceived::Inherited)
            || crate::dates::held_more_than_one_year(
                self.received.earlier_owner().map_or(self.acquired, |d| d.min(self.acquired)),
                self.closed,
            )
    }
    fn window(&self) -> u32 {
        self.owned().clamp(0, i64::from(WINDOW)) as u32
    }
    // Days owned in the ordinary 5-year window and in the added period a
    // service election reaches back before it (section 121(d)(9)).
    fn windows(&self, suspended: u32) -> (u32, u32) {
        let recent = self.window();
        let owned = self.owned().max(0) as u32;
        (recent, (owned - recent).min(suspended))
    }
    fn filer(&self, lived: &HomeLived, earlier: Option<u32>, extra: u32) -> FilerUse {
        let window = self.window();
        let (lived, care) = lived.record(window);
        let lived = lived + extra;
        let since = earlier.map_or(730, |sold| days(sold, self.closed).clamp(0, 730) as u32);
        FilerUse { owned: window, lived, care, prior: since < 730, since_prior: since }
    }
    // The first relationship between this page's fields that fails, by its
    // client rule's key; the 2026 closing date is checked only here (the
    // rule language has no date literal).
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if self.acquired >= self.closed {
            return Some(ValidationIssue::new("HOME_ACQUIRED_AFTER_SALE").at_field("acquired"));
        }
        if !self.property_received && self.property_value != crate::SourceCents::NotReported {
            return Some(
                ValidationIssue::new("HOME_PROPERTY_VALUE_WITHOUT_BOX_6")
                    .at_field("property_value"),
            );
        }
        if !self.foreign_person && self.firpta_withheld != crate::SourceCents::NotReported {
            return Some(
                ValidationIssue::new("HOME_FIRPTA_WITHOUT_BOX_7").at_field("firpta_withheld"),
            );
        }
        if let Some(issue) =
            self.payment.issue(self.record().price).map(|issue| issue.in_field("payment"))
        {
            return Some(issue);
        }
        if self.exchange.as_ref().is_some_and(|e| e.value > self.record().price) {
            return Some(
                ValidationIssue::new("HOME_EXCHANGE_VALUE_ABOVE_PRICE")
                    .at_field("value")
                    .in_field("exchange"),
            );
        }
        let prior_owner = |key, date: u32, field: &'static str, outer: &'static str| {
            (date > self.acquired)
                .then(|| ValidationIssue::new(key).at_field(field).in_field(outer))
        };
        if let Some(issue) = match &self.received {
            HomeReceived::Gift { given_acquired: d, .. } => {
                prior_owner("HOME_PRIOR_OWNER_AFTER_ACQUIRED", *d, "given_acquired", "received")
            }
            HomeReceived::FromSpouse { spouse_acquired: d, .. } => {
                prior_owner("HOME_PRIOR_OWNER_AFTER_ACQUIRED", *d, "spouse_acquired", "received")
            }
            _ => None,
        } {
            return Some(issue);
        }
        if let HomeSaleMore::SurvivingSpouse { late_acquired: Some(d), .. } = &self.more
            && let Some(issue) =
                prior_owner("HOME_PRIOR_OWNER_AFTER_ACQUIRED", *d, "late_acquired", "more")
        {
            return Some(issue);
        }
        let owned = self.owned();
        let spouse = match &self.spouse {
            HomeSpouse::NotJoint => None,
            HomeSpouse::Joint { lived, earlier_sale } => Some((lived, earlier_sale)),
        };
        if !self.lived.fits(owned, self.use_days()) {
            return Some(ValidationIssue::new("HOME_LIVED_ABOVE_OWNED").at_field("lived"));
        }
        if spouse.is_some_and(|(lived, _)| !lived.fits(owned, 0)) {
            return Some(
                ValidationIssue::new("HOME_LIVED_ABOVE_OWNED").at_field("lived").in_field("spouse"),
            );
        }
        if self.earlier_sale.date().is_some_and(|sold| sold >= self.closed) {
            return Some(
                ValidationIssue::new("HOME_EARLIER_SALE_AFTER_SALE")
                    .at_field("sold")
                    .in_field("earlier_sale"),
            );
        }
        if spouse.and_then(|(_, earlier)| earlier.date()).is_some_and(|sold| sold >= self.closed) {
            return Some(
                ValidationIssue::new("HOME_EARLIER_SALE_AFTER_SALE")
                    .at_field("sold")
                    .in_field("earlier_sale")
                    .in_field("spouse"),
            );
        }
        if i64::from(self.use_history.nonqualified_days) > owned {
            return Some(
                ValidationIssue::new("HOME_NONQUALIFIED_ABOVE_OWNED")
                    .at_field("nonqualified_days")
                    .in_field("use_history"),
            );
        }
        if let Some(part) = &self.use_history.separate {
            let separate = |key, field| {
                ValidationIssue::new(key)
                    .at_field(field)
                    .in_field("separate")
                    .in_field("use_history")
            };
            if part.area >= part.whole_area {
                return Some(separate("HOME_SEPARATE_AREA_ABOVE_WHOLE", "area"));
            }
            if self.record().separate_sale().is_some_and(|s| s.basis < 0) {
                return Some(separate("HOME_SEPARATE_DEPRECIATION_ABOVE_COST", "depreciation"));
            }
        }
        if let Some(service) = &self.service {
            let in_service =
                |key, field| ValidationIssue::new(key).at_field(field).in_field("service");
            if self.spouse.joint() != service.spouse_added_lived.is_some() {
                return Some(in_service(
                    if self.spouse.joint() {
                        "HOME_SERVICE_SPOUSE_NEEDED"
                    } else {
                        "HOME_SERVICE_SPOUSE_NOT_JOINT"
                    },
                    "spouse_added_lived",
                ));
            }
            if !self.spouse.joint() && service.spouse_added_care.is_some() {
                return Some(in_service("HOME_SERVICE_SPOUSE_NOT_JOINT", "spouse_added_care"));
            }
            if matches!(self.more, HomeSaleMore::SurvivingSpouse { .. }) {
                return Some(ValidationIssue::new("HOME_SERVICE_WITH_SURVIVING").at_field("more"));
            }
            let (_, added) = self.windows(service.election.suspended_days);
            if [Some(service.added_lived), service.spouse_added_lived]
                .into_iter()
                .flatten()
                .any(|lived| lived > added)
            {
                return Some(in_service("HOME_SERVICE_LIVED_ABOVE_OWNED", "added_lived"));
            }
        }
        if let HomeSaleMore::SurvivingSpouse { died, .. } = self.more {
            if died > self.closed {
                return Some(
                    ValidationIssue::new("HOME_DEATH_AFTER_SALE").at_field("died").in_field("more"),
                );
            }
            if self.reason != HomeSaleReason::TwoYears {
                return Some(ValidationIssue::new("HOME_SURVIVING_WITH_REASON").at_field("reason"));
            }
            if self.spouse.joint() {
                return Some(ValidationIssue::new("HOME_SURVIVING_ON_JOINT").at_field("spouse"));
            }
        }
        None
    }
    // The stored sale this page writes: the holding period, the days owned
    // and the test-period days derived from the dates.
    pub(crate) fn record(&self) -> HomeSale {
        let you = self.filer(&self.lived, self.earlier_sale.date(), self.use_days());
        let (spouse, spouse_earlier) = match &self.spouse {
            HomeSpouse::NotJoint => (None, None),
            HomeSpouse::Joint { lived, earlier_sale } => {
                (Some(self.filer(lived, earlier_sale.date(), 0)), earlier_sale.date())
            }
        };
        let full = |p: &FilerUse| HomeUse {
            owned: p.owned,
            lived: p.lived,
            care: p.care.clone(),
            prior_exclusion: p.prior,
        };
        let reduced = |p: &FilerUse| ReducedHomeUse {
            owned: p.owned,
            lived: p.lived,
            care: p.care.clone(),
            since_prior: p.since_prior,
        };
        let duty = |p: &FilerUse,
                    election: &HomeServiceElection,
                    added: u32,
                    care: Option<&HomeServiceCare>| {
            let (_, earlier_owned) = self.windows(election.suspended_days);
            DutyHomeUse {
                recent_owned: p.owned,
                recent_lived: p.lived,
                earlier_owned,
                earlier_lived: added,
                care: DutyHomeCare {
                    recent_days: p.care.days,
                    earlier_days: care.map_or(0, |care| care.facility_days),
                    recent_owned_residence_days: p.care.owned_residence_days,
                    earlier_owned_residence_days: care.map_or(0, |care| care.owned_residence_days),
                },
            }
        };
        let exclusion = match (&self.more, &self.reason) {
            (
                HomeSaleMore::SurvivingSpouse { died, both_qualified, spouse_earlier_sale, .. },
                _,
            ) => HomeExclusion::Widowed {
                sale_date: self.closed,
                death_date: *died,
                history: full(&you),
                late_prior: *spouse_earlier_sale,
                joint_tests_at_death: *both_qualified,
            },
            (_, HomeSaleReason::Event { reason }) if self.service.is_some() => {
                let service = self.service.as_ref().expect("checked");
                let reduced =
                    |p: &FilerUse, added: u32, care: Option<&HomeServiceCare>| DutyReducedHomeUse {
                        history: duty(p, &service.election, added, care),
                        since_prior: p.since_prior,
                    };
                HomeExclusion::ReducedService {
                    election: service.election.clone(),
                    reason: reason.clone(),
                    taxpayer: reduced(&you, service.added_lived, service.added_care.as_ref()),
                    spouse: spouse.as_ref().map(|p| {
                        reduced(
                            p,
                            service.spouse_added_lived.unwrap_or(0),
                            service.spouse_added_care.as_ref(),
                        )
                    }),
                }
            }
            (_, HomeSaleReason::Event { reason }) => HomeExclusion::Reduced {
                reason: reason.clone(),
                taxpayer: reduced(&you),
                spouse: spouse.as_ref().map(reduced),
            },
            _ => match &self.service {
                Some(service) => {
                    let full = |p: &FilerUse, added: u32, care: Option<&HomeServiceCare>| {
                        FullDutyHomeUse {
                            history: duty(p, &service.election, added, care),
                            prior_exclusion: p.prior,
                        }
                    };
                    HomeExclusion::ServiceSuspension {
                        election: service.election.clone(),
                        taxpayer: full(&you, service.added_lived, service.added_care.as_ref()),
                        spouse: spouse.as_ref().map(|p| {
                            full(
                                p,
                                service.spouse_added_lived.unwrap_or(0),
                                service.spouse_added_care.as_ref(),
                            )
                        }),
                    }
                }
                None => {
                    HomeExclusion::Full { taxpayer: full(&you), spouse: spouse.as_ref().map(full) }
                }
            },
        };
        let facts = HomeSaleFacts {
            closed: self.closed,
            acquired: self.acquired,
            received: self.received.clone(),
            late_acquired: self.late().0,
            late_lived: self.late().1,
            form_1099s: self.form_1099s,
            property_received: self.property_received,
            property_value: self.property_value.clone(),
            foreign_person: self.foreign_person,
            firpta_withheld: self.firpta_withheld.clone(),
            earlier: self.earlier_sale.date(),
            spouse_earlier,
            decline: self.more == HomeSaleMore::Decline,
            special: match self.more {
                HomeSaleMore::Refused { situation } => Some(situation),
                _ => None,
            },
        };
        HomeSale {
            // The box 2 price plus the value of property or services received
            // (unknown leaves it out; the facts refuse the sale then).
            price: self.proceeds + source_cents(&self.property_value).unwrap_or(0),
            expenses: self.expenses,
            basis: self.basis,
            // Held more than one year: sold after the first anniversary.
            term: if self.long_term() { GainTerm::Long } else { GainTerm::Short },
            use_history: match self.use_history {
                HomeRentalUse { depreciation: 0, nonqualified_days: 0, .. } => {
                    HomeSaleUse::Personal
                }
                HomeRentalUse { depreciation, nonqualified_days, .. } => HomeSaleUse::Adjusted {
                    depreciation,
                    owned_days: self.owned().clamp(1, 73050) as u32,
                    nonqualified_days,
                },
            },
            separate: self.use_history.separate.clone(),
            payment: self.payment.clone(),
            exchange: self.exchange.clone(),
            exclusion,
            // The page's unsupported situations are its facts' `special`,
            // left out or refused by name; this answer is not asked.
            ordinary: true,
            facts: Some(facts),
        }
    }
}
