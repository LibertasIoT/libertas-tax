// Coverage: Sale of your main home (section 121, Pub. 523; Form 8949/Schedule D) — Supported
//   (u6-home, 2026-09-28; calculation audited against Pub. 523 (2025 draft, Worksheets 1-3), the
//   2025 draft Schedule D and Form 8949 instructions and the Form 1099-S (Rev. 12-2026)).
//   Lines: realized gain (Worksheet 2 line 7; a loss is not deductible), the post-May 6, 1997
//   depreciation gain (unrecaptured section 1250, Schedule D line 19), the nonqualified-use gain
//   (after depreciation, by days), the $250,000 / $500,000 limit (joint: one spouse owns, both
//   live and neither excluded a sale in 2 years; otherwise each qualifying spouse's own $250,000),
//   the reduced exclusion (shortest of residence, ownership and time since the last excluded sale
//   over 730 days, per spouse), the surviving spouse's $500,000, licensed care-facility time, the
//   service suspension (engine only), the election to report the gain. Reporting (fixed u6-home):
//   Form 8949 box C/F with columns (d), (e), (g) codes E/H/L and (h), and Schedule D line 3/10,
//   whenever a 1099-S was received or gain is taxable; before, the taxable gain reached line 16
//   but no Form 8949 row or line 3/10 total, and a fully excluded sale without a 1099-S was not
//   distinguished. Page: Investment sales kind MainHomeSale ("Sale of your main home (1099-S)"):
//   1099-S boxes 1, 2a, 3, 6, 7 (tab "1099-S"), filer facts in tab "Your home"; the holding
//   period, days owned and the 5-year window (1,826 days in 2026) derive from the dates; stored
//   HomeSale.facts keeps the dates, the 1099-S answers, the earlier-sale dates, the report-the-gain
//   election and a refused situation. Refused by name: expatriate (Form 8854, an added tax).
//   Left out with a per-item notice (Answers and unknowns, 2026-10-05): like-kind acquisition
//   (Form 8824), a separate business part or non-straight-line depreciation (Form 4797), the
//   service suspension on the page, an installment sale (Form 6252), 1099-S boxes 6 and 7,
//   another situation. Rules on both sides: HOME_ACQUIRED_AFTER_SALE, HOME_LIVED_ABOVE_OWNED,
//   HOME_EARLIER_SALE_AFTER_SALE, HOME_NONQUALIFIED_ABOVE_OWNED, HOME_DEATH_AFTER_SALE,
//   HOME_SURVIVING_WITH_REASON, HOME_SURVIVING_ON_JOINT; server only HOME_SALE_NOT_IN_CURRENT_YEAR; the
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
    /// Days in the facility while still owning the home
    /// Exclude days already counted as residence in the home.
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_default(0)]
    pub days: u32,
    /// Days both owning and living in the home during that window
    /// These days are already included in the home-residence count, not additional days.
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_default(0)]
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
    /// Days owned in the five years ending on sale date
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    pub owned: u32,
    /// Days used as main home in those five years
    /// Periods need not be continuous. Count qualifying short temporary absences; do not count rental use as residence.
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
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
    /// Days of qualifying service elected for this home
    /// At most ten years; do not include days elected for another property.
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 1, max = 3653)]
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
    /// Facility days in the ordinary five-year window, not counted as residence in the home
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_default(0)]
    pub recent_days: u32,
    /// Facility days in the additional period reached by the service election
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
    #[libertas_default(0)]
    pub earlier_days: u32,
    /// Days both owning and living in the home in the ordinary five-year window
    /// Already included in home-residence days, not additional days.
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    #[libertas_default(0)]
    pub recent_owned_residence_days: u32,
    /// Days both owning and living in the home in the added window
    /// Already included in home-residence days, not additional days.
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
    #[libertas_default(0)]
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
    /// Days owned in the ordinary five years before sale
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    pub recent_owned: u32,
    /// Days used as a main home in the ordinary five years before sale
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    pub recent_lived: u32,
    /// Days owned in the additional period reached by the service election
    /// This earlier period cannot exceed the selected suspension period.
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
    pub earlier_owned: u32,
    /// Days used as a main home in that additional period
    /// Count qualifying short temporary absences; do not count rental use as residence.
    // Section 121(d)(9) tax structure: qualified official-duty suspension is limited to ten years.
    #[libertas_number(min = 0, max = 3653)]
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
    /// Days since this filer's last excluded home sale
    /// Enter 730 if no prior excluded sale occurred within two years.
    // Section 121 tax structure: the two-year interval between exclusions.
    #[libertas_number(min = 0, max = 730)]
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
    /// Days owned during the five years ending on sale date
    /// On a joint return, include days either spouse owned the home, counting overlapping days only once.
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    pub owned: u32,
    /// Days this filer used the property as a main home in those five years
    // Section 121 tax structure: days within the five-year ownership/use test period.
    #[libertas_number(min = 0, max = 1827)]
    pub lived: u32,
    /// Additional care-facility residence
    pub care: HomeCare,
    /// Days since this filer's last excluded home sale
    /// Enter 730 if there was no prior excluded sale or it was at least two years earlier.
    // Section 121 tax structure: the two-year interval between exclusions.
    #[libertas_number(min = 0, max = 730)]
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
        /// Distance from the home to the former work location, in miles
        /// Enter zero if there was no former work location.
        #[libertas_number(min = 0)]
        old_miles: u32,
        /// Distance from the home to the new work location, in miles
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        depreciation: i64,
        /// Total days of ownership through sale
        /// Count the entire ownership period, including years before 2009.
        #[libertas_number(min = 1)]
        owned_days: u32,
        /// Days of nonqualified use after 2008
        /// Count days neither you nor your spouse or former spouse used the property as a main home. Exclude the period after its last use as a main home within the five-year sale window, qualifying temporary absences of up to two years for work, health or unforeseen circumstances, and qualifying official extended duty of up to ten years. Retain dated use records; overlapping days count once.
        #[libertas_number(min = 0)]
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

/// Sale of your main home
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeSale {
    /// Selling price attributable to the interest reported on this return
    /// Use closing records, before selling expenses; do not subtract the mortgage payoff. Joint filers enter the combined interest once, separate filers only their own share.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub price: i64,
    /// Selling expenses for that interest
    /// Commissions and allowable closing costs, not mortgage principal or moving costs. Do not also include these expenses in basis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub expenses: i64,
    /// Adjusted basis for that interest before selling expenses
    /// Retain acquisition, capital improvement and other basis-adjustment records. Use adjusted basis, not current mortgage balance.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis: i64,
    /// Holding period on sale date
    pub term: GainTerm,
    /// Depreciation and nonqualified-use records
    pub use_history: HomeSaleUse,
    /// Exclusion choice and history
    pub exclusion: HomeExclusion,
    /// Home sale with known basis and no unlisted special treatment
    /// No separate business/rental part outside the dwelling unit, ordinary depreciation recapture, section 1231 look-back recharacterization, regular/AMT basis difference, installment payments, debt cancellation, like-kind acquisition in the last five years, expatriation, related-party restriction, unlisted divorce/death ownership rules or other special treatment. Your purchase and closing records and prior returns show it. No leaves this sale out of the estimate with a notice; that situation is not supported.
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
    pub(crate) fn gain(&self) -> i64 {
        phase::floor(self.price - self.expenses - self.basis, 0)
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
        // A sale whose `ordinary` is No is left out before the rules run.
        if self.depreciation_gain() > 0 && self.term != GainTerm::Long {
            return Err((
                "ISSUE_HOME_SALE_RESOLVE_HOME_SALE_BASIS__NONQUALIFIED_USE__DEPRECIATION_84E3D92F",
                crate::fixture_english!(
                    "Depreciation on a home held one year or less is not calculated (Form 4797)."
                ),
            ));
        }
        // The page's election to report the whole gain keeps its answers.
        if self.facts.as_ref().is_some_and(|f| f.decline) {
            return Ok(0);
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
    /// You got the home in a like-kind exchange in the 5 years before the sale
    LikeKindExchange,
    /// You are subject to the expatriation tax
    Expatriate,
    /// Part of the property was a separate business or rental outside your living area, or you took depreciation other than straight-line
    BusinessPart,
    /// You choose to suspend the 5-year period for uniformed, Foreign Service, intelligence or Peace Corps duty
    ServiceDuty,
    /// The buyer pays part of the price after 2026
    Installment,
    /// Another special situation
    Other,
}
impl HomeSaleSpecial {
    // Answers and unknowns: the notice of a situation the page does not
    // calculate, which leaves the sale out of the estimate; None for the
    // expatriation tax, an added tax that is refused by name instead.
    fn notice(self) -> Option<&'static str> {
        match self {
            Self::LikeKindExchange => Some("WARN_HOME_SALE_LIKE_KIND_LEFT_OUT"),
            Self::Expatriate => None,
            Self::BusinessPart => Some("WARN_HOME_SALE_BUSINESS_PART_LEFT_OUT"),
            Self::ServiceDuty => Some("WARN_HOME_SALE_SERVICE_DUTY_LEFT_OUT"),
            Self::Installment => Some("WARN_HOME_SALE_INSTALLMENT_LEFT_OUT"),
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
    /// A Form 1099-S reported the sale
    pub form_1099s: bool,
    /// Property or services were part of the price
    pub property_received: bool,
    /// The seller is a foreign person
    pub foreign_person: bool,
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
            && [self.earlier, self.spouse_earlier]
                .iter()
                .flatten()
                .all(|d| date(*d) && *d < self.closed)
    }
    // The page's named refusal: the expatriation tax (Form 8854), an added
    // tax the App does not calculate.
    fn refusal(&self) -> Option<(&'static str, &'static str)> {
        (self.special == Some(HomeSaleSpecial::Expatriate)).then_some((
            "ISSUE_HOME_SALE_SPECIAL_2",
            crate::fixture_english!(
                "Sale of your main home: a sale by a person subject to the expatriation tax is not calculated (Form 8854, Form 8949)."
            ),
        ))
    }
    // The notice of a page situation the estimate leaves out (Answers and
    // unknowns): another named situation, or 1099-S box 6 or 7.
    pub(crate) fn left_out(&self) -> Option<&'static str> {
        if let Some(special) = self.special {
            return special.notice();
        }
        if self.property_received {
            return Some("WARN_HOME_SALE_PROPERTY_RECEIVED_LEFT_OUT");
        }
        if self.foreign_person {
            return Some("WARN_HOME_SALE_FOREIGN_PERSON_LEFT_OUT");
        }
        None
    }
}

impl HomeSale {
    // Off a joint return the spouse's facts are not asked: erases them
    // (the walk's prune); true when something was erased.
    pub(crate) fn drop_spouse(&mut self) -> bool {
        let mut dropped = false;
        match &mut self.exclusion {
            HomeExclusion::Full { spouse, .. } => dropped |= spouse.take().is_some(),
            HomeExclusion::Reduced { spouse, .. } => dropped |= spouse.take().is_some(),
            HomeExclusion::ServiceSuspension { spouse, .. } => dropped |= spouse.take().is_some(),
            HomeExclusion::ReducedService { spouse, .. } => dropped |= spouse.take().is_some(),
            HomeExclusion::Widowed { .. } | HomeExclusion::Decline => {}
        }
        if let Some(facts) = self.facts.as_mut() {
            dropped |= facts.spouse_earlier.take().is_some();
        }
        dropped
    }
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
        /// Days lived there as your main home in the 5 years before the sale
        // Section 121 tax structure: days within the five-year ownership/use test period.
        #[libertas_number(min = 0, max = 1826)]
        days: u32,
    },
    /// Part of that time, then in a licensed care facility
    /// You became physically or mentally unable to care for yourself, and the facility is licensed to care for people with your condition. Facility time counts only if you lived in the home at least 365 days.
    CareFacility {
        /// Days lived in the home in the 5 years before the sale
        // Section 121 tax structure: days within the five-year ownership/use test period.
        #[libertas_number(min = 0, max = 1826)]
        home_days: u32,
        /// Days in the facility in those 5 years while you still owned the home
        // Section 121 tax structure: days within the five-year ownership/use test period.
        #[libertas_number(min = 0, max = 1826)]
        facility_days: u32,
    },
}
impl HomeLived {
    // Residence days in the test period and the facility time, as stored.
    fn record(&self, window: u32) -> (u32, HomeCare) {
        match self {
            Self::WholeTime => (window, HomeCare::NONE),
            Self::Days { days } => ((*days).min(window), HomeCare::NONE),
            Self::CareFacility { home_days, facility_days } => {
                let home = (*home_days).min(window);
                (
                    home,
                    HomeCare {
                        days: (*facility_days).min(window - home),
                        owned_residence_days: home,
                    },
                )
            }
        }
    }
    fn from_record(lived: u32, care: &HomeCare, window: u32) -> Option<Self> {
        Some(match care {
            care if *care == HomeCare::NONE && lived == window => Self::WholeTime,
            care if *care == HomeCare::NONE => Self::Days { days: lived },
            HomeCare { days, owned_residence_days } if *owned_residence_days == lived => {
                Self::CareFacility { home_days: lived, facility_days: *days }
            }
            _ => return None,
        })
    }
    // Whether the days fit the time owned (both sides:
    // HOME_LIVED_ABOVE_OWNED).
    fn fits(&self, owned: i64) -> bool {
        let fits = |n: u32| n <= WINDOW && i64::from(n) <= owned;
        match self {
            Self::WholeTime => true,
            Self::Days { days } => fits(*days),
            Self::CareFacility { home_days, facility_days } => fits(home_days + facility_days),
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
    fn from_date(date: Option<u32>) -> Self {
        date.map_or(Self::No, |sold| Self::Yes { sold })
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

/// Rental or business use of the home
/// Leave both at zero when the home was only ever your home.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeRentalUse {
    /// Depreciation allowed or allowable after May 6, 1997
    /// Straight-line depreciation on the home, including amounts you could have deducted but did not. Enter zero if none.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub depreciation: i64,
    /// Days after 2008 that nobody used it as a main home
    /// Do not count the time after you moved out within the 5 years before the sale, temporary absences of up to 2 years for work, health or an unforeseen event, or qualified official extended duty. Enter zero if none.
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub nonqualified_days: u32,
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

/// Sale of your main home
/// One for each home you sold in 2026, with its Form 1099-S if you received one. On a joint return, enter a home you sold together once.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the Form 1099-S exactly. Check its tax year against the supplied year; report a mismatch. Return the address or legal description of the property, or an empty string when it is blank. The date of closing and gross proceeds must show values; report a blank one as an error. When several 1099-S forms for the same property are supplied, their dates of closing and properties must agree (report a conflict as an error); return the sum of their gross proceeds. Return true for the property or services received as part of the consideration checkbox, and for the transferor is a foreign person checkbox, only when checked. Report illegible or conflicting values as errors. Do not infer when the home was bought, its basis, selling expenses, how long anyone lived in it, why it was sold, or other facts absent from the 1099-S. The file may hold several copies of this form, such as Copy B and Copy 2; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    accept_multiple_inputs = true
)]
pub struct MainHomeSale {
    /// 1099-S box 3 address of the property
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub address: String,
    /// 1099-S box 1 date of closing
    /// Without a 1099-S, the closing date on your settlement statement.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub closed: u32,
    /// 1099-S box 2a gross proceeds
    /// Without a 1099-S, the contract price on your settlement statement. On a joint return with a 1099-S for each of you, add the two.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub proceeds: i64,
    /// 1099-S box 6 property or services received as part of the price
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub property_received: bool,
    /// 1099-S box 7 the transferor is a foreign person
    #[libertas_llm_doc_field]
    #[libertas_default(false)]
    pub foreign_person: bool,
    /// You received a Form 1099-S for this sale
    /// Without one, a sale whose whole gain is excluded is not reported on your return.
    #[libertas_default(true)]
    pub form_1099s: bool,
    /// Date you bought or received the home
    /// If you and your spouse got it at different times, the earlier date.
    #[libertas_date_only]
    pub acquired: u32,
    /// Selling expenses
    /// Commissions, advertising, legal fees and the closing costs you paid as the seller. Not the mortgage payoff or moving costs.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub expenses: i64,
    /// Adjusted basis
    /// What you paid for the home with its purchase closing costs, plus improvements, less depreciation, casualty losses and energy credits. Not the mortgage balance.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub basis: i64,
    /// How long you lived in it as your main home
    #[libertas_default(WholeTime)]
    pub lived: HomeLived,
    /// You excluded gain on another home sold in the 2 years before this sale
    #[libertas_default(No)]
    pub earlier_sale: EarlierHomeSale,
    /// Your spouse
    /// Asked on a joint return.
    #[libertas_constrained_by("$.allowed_home_spouse")]
    pub spouse: HomeSpouse,
    /// Why you sold
    #[libertas_default(TwoYears)]
    pub reason: HomeSaleReason,
    /// Rental or business use
    #[libertas_foldable(initially_open = false)]
    pub use_history: HomeRentalUse,
    /// Something else
    #[libertas_default(Nothing)]
    pub more: HomeSaleMore,
}

// One filer's test-period facts as the stored exclusion keeps them.
struct FilerUse {
    owned: u32,
    lived: u32,
    care: HomeCare,
    prior: bool,
    since_prior: u32,
}

impl MainHomeSale {
    // Days owned in all, and in the 5-year test period.
    fn owned(&self) -> i64 {
        days(self.acquired, self.closed)
    }
    fn window(&self) -> u32 {
        self.owned().clamp(0, i64::from(WINDOW)) as u32
    }
    fn filer(&self, lived: &HomeLived, earlier: Option<u32>) -> FilerUse {
        let window = self.window();
        let (lived, care) = lived.record(window);
        let since = earlier.map_or(730, |sold| days(sold, self.closed).clamp(0, 730) as u32);
        FilerUse { owned: window, lived, care, prior: since < 730, since_prior: since }
    }
    // The first relationship between this page's fields that fails, by its
    // client rule's key; the 2026 closing date is checked only here (the
    // rule language has no date literal).
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if self.closed / 10000 != 2026 {
            return Some(ValidationIssue::new("HOME_SALE_NOT_IN_CURRENT_YEAR").at_field("closed"));
        }
        if self.acquired >= self.closed {
            return Some(ValidationIssue::new("HOME_ACQUIRED_AFTER_SALE").at_field("acquired"));
        }
        let owned = self.owned();
        let spouse = match &self.spouse {
            HomeSpouse::NotJoint => None,
            HomeSpouse::Joint { lived, earlier_sale } => Some((lived, earlier_sale)),
        };
        if !self.lived.fits(owned) {
            return Some(ValidationIssue::new("HOME_LIVED_ABOVE_OWNED").at_field("lived"));
        }
        if spouse.is_some_and(|(lived, _)| !lived.fits(owned)) {
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
        let you = self.filer(&self.lived, self.earlier_sale.date());
        let (spouse, spouse_earlier) = match &self.spouse {
            HomeSpouse::NotJoint => (None, None),
            HomeSpouse::Joint { lived, earlier_sale } => {
                (Some(self.filer(lived, earlier_sale.date())), earlier_sale.date())
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
        let exclusion = match (&self.more, &self.reason) {
            (HomeSaleMore::SurvivingSpouse { died, both_qualified, spouse_earlier_sale }, _) => {
                HomeExclusion::Widowed {
                    sale_date: self.closed,
                    death_date: *died,
                    history: full(&you),
                    late_prior: *spouse_earlier_sale,
                    joint_tests_at_death: *both_qualified,
                }
            }
            (_, HomeSaleReason::Event { reason }) => HomeExclusion::Reduced {
                reason: reason.clone(),
                taxpayer: reduced(&you),
                spouse: spouse.as_ref().map(reduced),
            },
            _ => HomeExclusion::Full { taxpayer: full(&you), spouse: spouse.as_ref().map(full) },
        };
        let facts = HomeSaleFacts {
            closed: self.closed,
            acquired: self.acquired,
            form_1099s: self.form_1099s,
            property_received: self.property_received,
            foreign_person: self.foreign_person,
            earlier: self.earlier_sale.date(),
            spouse_earlier,
            decline: self.more == HomeSaleMore::Decline,
            special: match self.more {
                HomeSaleMore::Refused { situation } => Some(situation),
                _ => None,
            },
        };
        HomeSale {
            price: self.proceeds,
            expenses: self.expenses,
            basis: self.basis,
            // Held more than one year: sold after the first anniversary.
            term: if crate::dates::held_more_than_one_year(self.acquired, self.closed) {
                GainTerm::Long
            } else {
                GainTerm::Short
            },
            use_history: match self.use_history {
                HomeRentalUse { depreciation: 0, nonqualified_days: 0 } => HomeSaleUse::Personal,
                HomeRentalUse { depreciation, nonqualified_days } => HomeSaleUse::Adjusted {
                    depreciation,
                    owned_days: self.owned().clamp(1, 73050) as u32,
                    nonqualified_days,
                },
            },
            exclusion,
            // The page's unsupported situations are its facts' `special` and
            // 1099-S boxes 6 and 7, left out or refused by name; this answer
            // is not asked.
            ordinary: true,
            facts: Some(facts),
        }
    }
    // The reverse of record; None for a stored sale this page could not
    // have written.
    pub(crate) fn from_record(label: &str, v: &HomeSale) -> Option<Self> {
        let f = v.facts.as_ref()?;
        let window = days(f.acquired, f.closed).clamp(0, i64::from(WINDOW)) as u32;
        let lived = |p: &HomeUse| HomeLived::from_record(p.lived, &p.care, window);
        let spouse_of = |lived: HomeLived| HomeSpouse::Joint {
            lived,
            earlier_sale: EarlierHomeSale::from_date(f.spouse_earlier),
        };
        let (you, spouse, reason, widowed) = match &v.exclusion {
            HomeExclusion::Full { taxpayer, spouse } => (
                lived(taxpayer)?,
                match spouse {
                    Some(s) => spouse_of(lived(s)?),
                    None => HomeSpouse::NotJoint,
                },
                HomeSaleReason::TwoYears,
                None,
            ),
            HomeExclusion::Reduced { reason, taxpayer, spouse, .. } => {
                let reduced = |p: &ReducedHomeUse| HomeLived::from_record(p.lived, &p.care, window);
                (
                    reduced(taxpayer)?,
                    match spouse {
                        Some(s) => spouse_of(reduced(s)?),
                        None => HomeSpouse::NotJoint,
                    },
                    HomeSaleReason::Event { reason: reason.clone() },
                    None,
                )
            }
            HomeExclusion::Widowed {
                death_date,
                history,
                late_prior,
                joint_tests_at_death,
                ..
            } => (
                lived(history)?,
                HomeSpouse::NotJoint,
                HomeSaleReason::TwoYears,
                Some(HomeSaleMore::SurvivingSpouse {
                    died: *death_date,
                    both_qualified: *joint_tests_at_death,
                    spouse_earlier_sale: *late_prior,
                }),
            ),
            _ => return None,
        };
        let more = if f.decline {
            HomeSaleMore::Decline
        } else if let Some(situation) = f.special {
            HomeSaleMore::Refused { situation }
        } else {
            widowed.unwrap_or(HomeSaleMore::Nothing)
        };
        let page = Self {
            address: label.into(),
            closed: f.closed,
            proceeds: v.price,
            property_received: f.property_received,
            foreign_person: f.foreign_person,
            form_1099s: f.form_1099s,
            acquired: f.acquired,
            expenses: v.expenses,
            basis: v.basis,
            lived: you,
            earlier_sale: EarlierHomeSale::from_date(f.earlier),
            spouse,
            reason,
            use_history: match v.use_history {
                HomeSaleUse::Personal => HomeRentalUse { depreciation: 0, nonqualified_days: 0 },
                HomeSaleUse::Adjusted { depreciation, nonqualified_days, .. } => {
                    HomeRentalUse { depreciation, nonqualified_days }
                }
            },
            more,
        };
        (page.record() == *v).then_some(page)
    }
    // The entry label: the box 3 address as typed.
    pub(crate) fn label(&self) -> String {
        self.address.clone()
    }
}
