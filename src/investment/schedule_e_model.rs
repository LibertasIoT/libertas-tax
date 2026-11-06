// schedule_e items of src/federal/federal_model.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub use crate::filer_source::ProfessionalRentals;

/// Prior personal-property rental losses
/// Suspended expenses from retained Form 8582 worksheets; leave both at zero when there are none. Enter them only for the same owner and continuing activity, with complete expense records: only fully at-risk nonbusiness operating expenses or depreciation suspended by the passive rules and reportable on Schedule 1. Depreciation has already reduced the asset's basis; releasing this deduction does not reduce basis again. No capital losses, interest, credits, QBI, ownership changes, dispositions, grouping changes or change to a nonrental activity. Do not also enter these deductions as current paid expenses or current depreciation.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct PersonalRentalHistory {
    /// Regular-tax suspended deductions for this activity
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub regular: i64,
    /// Separately tracked AMT suspended deductions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amt: i64,
}
impl PersonalRentalHistory {
    /// No suspended deductions.
    #[cfg(test)]
    pub(crate) const NONE: Self = Self { regular: 0, amt: 0 };
    pub(crate) fn balance(self, alternative: bool) -> i64 {
        if alternative { self.amt } else { self.regular }
    }
}

/// Nonbusiness property income source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum NonbusinessPropertyKind {
    /// Personal-property rental for profit, outside a rental business
    /// Equipment and other movable property; not real estate. Business rentals belong in Business.
    PersonalRental {
        /// Related source component identities
        related: Vec<i64>,
        /// Total unadjusted basis of property in this rental activity
        /// Original tax basis before depreciation deductions, including all property rented together in this activity. Do not use current remaining basis or fair market value. Use zero when the original tax-basis records establish zero; a fully depreciated asset can still have a positive unadjusted basis.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
        /// Unadjusted basis of the part subject to depreciation
        /// Use the original basis of depreciable property even if it is now fully depreciated. The App compares this with total basis to determine whether net income is recharacterized for the investment-interest limit.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        depreciable_basis: i64,
        /// Purchased rental equipment
        /// Enter every asset requiring current depreciation, once in this activity. Use 100% income-producing use and Section 179 ineligible: a nonbusiness rental cannot claim section 179. Already fully recovered property may be omitted here but remains in the unadjusted basis above. Include this property in the return-wide bonus/method elections and convention test. No listed property, vehicles, ADS or personal conversions.
        assets: Vec<crate::BusinessAsset>,
        /// Prior suspended deductions
        history: PersonalRentalHistory,
    },
}

impl NonbusinessPropertyKind {
    pub(crate) fn rental_depreciation(
        &self,
        mid_quarter: bool,
        amt: bool,
        owner: Owner,
        activity: i64,
        macrs: &crate::macrs_source::MacrsElectionView,
    ) -> i64 {
        match self {
            Self::PersonalRental { assets, .. } => {
                crate::nonmacrs::asset_depreciation_amounts_for_view(
                    owner,
                    Some(activity),
                    assets,
                    mid_quarter,
                    amt,
                    macrs,
                )
                .expense_cents(0)
                .expect("non-MACRS financial availability checked before legacy consumers")
            }
        }
    }

    pub(crate) fn prior_rental_loss(&self, alternative: bool) -> i64 {
        match self {
            Self::PersonalRental { history, .. } => history.balance(alternative),
        }
    }
    pub(crate) fn investment_rental(&self) -> bool {
        matches!(self, Self::PersonalRental { basis, depreciable_basis, .. }
            if i128::from(*depreciable_basis) * 10 < i128::from(*basis) * 3)
    }
}

// The data answers of M3-P15, each a kind of its topic: the foreign account
// or trust answers Schedule B Part III (FederalScreening.foreign_account,
// fbar_required/fbar_countries, foreign_trust), and the QBI loss carryforward is
// QbiHistory.
// Request-only representation of the existing election Boolean. Enumeration
// eligibility can hide the second joint editor's one permitted answer without
// introducing a second tax fact or changing the stored ProfessionalRentals.
/// Rental real estate election
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ProfessionalRentalElection {
    /// Separate rental activities
    SeparateActivities,
    /// All rental real estate as one activity
    SingleActivity,
}
/// Real estate professional rental facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ProfessionalRentalsPage {
    /// You elected to treat all your rental real estate as one activity
    /// The statement filed with an earlier return or with this one (section 469(c)(7)(A)). On a joint return this is one election for both spouses. Choose Professional/material participation on every rental and K-1 real estate activity, or on none; selling one property releases no suspended loss until the last is sold.
    #[libertas_copy_from("$.professional_election")]
    #[libertas_constrained_by("$.allowed_professional_elections")]
    pub single_activity: ProfessionalRentalElection,
    /// You worked more than 500 hours in each rental in which you chose Professional this year, or in any 5 of the last 10 years
    /// With the election, count all your rentals as one. Leave this off when your hours are not documented; the rental income then stays investment income. Each spouse's own recorded hours answer is preserved.
    #[libertas_default(false)]
    pub over_500_hours: bool,
}
impl From<ProfessionalRentals> for ProfessionalRentalsPage {
    fn from(value: ProfessionalRentals) -> Self {
        Self {
            single_activity: if value.single_activity {
                ProfessionalRentalElection::SingleActivity
            } else {
                ProfessionalRentalElection::SeparateActivities
            },
            over_500_hours: value.over_500_hours,
        }
    }
}
impl From<ProfessionalRentalsPage> for ProfessionalRentals {
    fn from(value: ProfessionalRentalsPage) -> Self {
        Self {
            single_activity: value.single_activity == ProfessionalRentalElection::SingleActivity,
            over_500_hours: value.over_500_hours,
        }
    }
}

/// Royalties
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RoyaltyProperty {
    /// Forms related to this royalty property
    pub related: crate::RelatedForms,
    /// Kind of property
    pub kind: crate::RoyaltyKind,
    /// Royalties not on a Form 1099-MISC you entered
    /// Enter the form in Documents and relate its box 2 to this property here; the App adds that amount. Enter the gross amount before any severance tax the producer withheld; enter that tax under taxes below.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub receipts: i64,
    /// Lease bonuses and advance royalties included in the royalties
    /// Amounts payable without regard to production, here or on the 1099-MISC. They take no percentage depletion.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub advance: i64,
    /// Expenses, lines 5–19
    pub expenses: RentalExpenses,
    /// Depletion (line 18)
    #[libertas_default(No)]
    pub depletion: crate::RoyaltyDepletion,
    /// An operating interest, royalties from your own current work or services, an amortizable basis, a sale of the rights, a partnership or trust share, foreign property, protected or borrowed investment, or production above 1,000 barrels a day
    /// Something else applies. No leaves this royalty in the estimate; Yes leaves it out with a notice.
    #[libertas_default(false)]
    pub special: bool,
}

/// Equipment you rent out
/// Rent for equipment or other movable property, outside a business. Rent for real estate belongs under Rental property.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EquipmentRent {
    /// Forms related to this rental
    pub related: crate::RelatedForms,
    /// Rent not on a Form 1099-MISC you entered
    /// Enter the form in Documents and relate its box 1 to this rental here; the App adds that amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub receipts: i64,
    /// Operating expenses
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub expenses: i64,
    /// Original cost of the rented property
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub basis: i64,
    /// Part of that cost you depreciate
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub depreciable_basis: i64,
    /// Equipment bought in 2026 or still depreciating
    /// ----
    /// Asset
    pub assets: Vec<Asset>,
    /// Deductions suspended in earlier years
    pub history: crate::PersonalRentalHistory,
}

/// Rental property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("RentalRecoveryOptions")]
pub struct Rental {
    /// Physical buildings containing separately recorded improvements
    /// Enter each building once; its accepted identity is offered in the improvement picker.
    pub improvement_buildings: Vec<crate::ImprovementBuildingFacts>,
    /// Actual election out of the business interest limit
    /// [NullText]
    /// No election out of the business interest limit
    /// [SetText]
    /// Election record
    #[libertas_constrained_by("^.allowed_interest_limit_election")]
    pub interest_limit_election: Option<crate::InterestLimitElectionRecord>,
    /// Applicable activity election record
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_interest_limit_choices")]
    pub allowed_interest_limit_election: Vec<i32>,
    /// Actual location of this rental property
    /// Enter all amounts in U.S. dollars. Translate foreign receipts and payments using the rate on the date received or paid, or an appropriate yearly average rate. A foreign business unit with its own functional currency remains a special treatment below.
    #[libertas_calculation_request("RentalRecoveryOptions")]
    pub location: crate::RentalLocation,
    /// Address
    /// Schedule E line 1a; needed only for a printed return.
    #[libertas_default("")]
    pub address: String,
    /// Line 1b type of property
    /// The property this rental was before any sale or exchange. After an exchange, the type of the property you received is chosen under how you use it.
    #[libertas_default(SingleFamily)]
    #[libertas_calculation_request("RentalRecoveryOptions")]
    pub kind: crate::RentalPropertyType,
    /// Line 3 rents not on a Form 1099 you entered
    /// Include advance rent, retained deposits, tenant reimbursements and lease cancellation payments.
    /// Leave out each Form 1099-MISC box 1 and 1099-K part related to this property; the App adds those.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub rent: i64,
    /// Related forms
    /// Add this property's Forms 1099 and original mortgage loans. Later lenders follow the original loan automatically. Enter loan interest, points and insurance in Documents before rental-day allocation; one loan cannot be split across properties.
    pub related: crate::RelatedForms,
    /// Expenses, lines 5–19
    pub expenses: RentalExpenses,
    /// The building
    pub building: Building,
    /// Appliances, furniture and other depreciable property
    #[libertas_unordered]
    /// ----
    /// Asset
    pub assets: Vec<crate::CanonicalAssetPage>,
    /// Sold, exchanged or kept in 2026
    #[libertas_default(No)]
    pub sale: crate::RentalSale,
    /// Form 1099-S for the sale or exchange
    /// Copy the printed form here; the page checks its closing date and total gross proceeds against the choice above.
    /// [NullText]
    /// No Form 1099-S details are supplied for this sale or exchange
    /// [SetText]
    /// The Form 1099-S details are
    #[libertas_default($null)]
    pub statement: Option<crate::Form1099SPaper>,
    /// Personal use
    #[libertas_default(No)]
    pub personal_use: PersonalUse,
    /// More about this property
    pub more: RentalDetails,
    /// Answer kept for the section 179 income limit
    /// Answered on the rentals page; kept here so saving this record keeps it.
    /// [NullText]
    /// Not asked
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub section179_business: Option<bool>,
    /// Last year's home-use worksheet carries kept for this rental
    /// Answered on the rentals page while the checklist names them; kept here so saving this record keeps them.
    /// [NullText]
    /// No home-use carry from 2025
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default($null)]
    pub prior_home_carry: Option<crate::RentalHomeCarries>,
}

/// Expenses, lines 5–19
/// Depreciation (line 18) is figured from the building below.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalExpenses {
    /// Line 5 advertising
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub advertising: i64,
    /// Line 6 auto and travel
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub auto_and_travel: i64,
    /// Line 7 cleaning and maintenance
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cleaning: i64,
    /// Line 8 commissions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub commissions: i64,
    /// Line 9 insurance
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub insurance: i64,
    /// Line 10 legal and other professional fees
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub legal: i64,
    /// Line 11 management fees
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub management: i64,
    /// Line 12 mortgage interest for a property with no personal use
    /// For personal-use property, select saved Mortgage loans above and leave this zero. Do not duplicate mortgage points or insurance from those loans in other expenses.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub mortgage_interest: i64,
    /// Line 13a vehicle loan interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub vehicle_loan_interest: i64,
    /// Line 13b other interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_interest: i64,
    /// Line 14 repairs
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub repairs: i64,
    /// Line 15 supplies
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub supplies: i64,
    /// Line 16 taxes
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxes: i64,
    /// Line 17 utilities
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub utilities: i64,
    /// Line 19 other
    #[libertas_unordered]
    /// ----
    /// Other expense
    pub other: Vec<OtherExpense>,
}

/// The building
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Building {
    /// Casualty or theft of this building or improvement
    /// [NullText]
    /// No casualty or theft is entered for this property
    /// [SetText]
    /// This property has casualty or theft records
    #[libertas_default($null)]
    pub casualty: Option<crate::RentalCasualty>,
    /// The same property in a saved personal casualty
    /// Choose it when you also use this dwelling personally and entered it as
    /// a property of a saved personal casualty event, so one event's $100 or
    /// $500 reduction and one property's loss apply once. That event's date and
    /// disaster declaration must match this record.
    /// [NullText]
    /// This property is not in a saved personal casualty
    /// [SetText]
    /// The saved personal casualty property is
    #[libertas_enum_source("$.casualty_properties")]
    #[libertas_default($null)]
    pub casualty_property: Option<i64>,

    /// Cost record identity
    #[libertas_hidden]
    #[libertas_default(0)]
    pub id: i64,
    /// Section 179D claim from this cost source
    /// [NullText]
    /// No current section 179D claim on this source
    /// [SetText]
    /// Claim from this source using
    #[libertas_default($null)]
    pub energy: Option<crate::RentalEnergyClaimPage>,
    /// Actual sale-date business basis and depreciation for a mixed-use sale
    /// Use this building's basis records, including actual business depreciation allowed or allowable.
    /// [NullText]
    /// Mixed-use sale records have not been supplied
    /// [SetText]
    /// The sale-date business records are
    #[libertas_default($null)]
    pub sale_history: Option<crate::RentalBuildingSaleHistory>,
    /// Your cost, including closing costs and land
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub cost: i64,
    /// Credit that reduces its cost
    /// Leave zero unless a tax credit was taken for this property. A clean-vehicle or fuel-cell vehicle credit (sections 30D(f)(1), 30B(h)(4) and 45W(d)(1)) and an investment credit (section 50(c)(1)) reduce its cost by the whole credit; an energy credit under section 48 reduces it by half the credit (section 50(c)(3)). Enter the reduction itself. It cannot exceed cost less land. When an automatic rehabilitation credit applies, the App figures its reduction and ignores this amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub credit_reduction: i64,
    /// Part of that cost for the land
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub land: i64,
    /// Date first available to rent
    #[libertas_date_only]
    #[libertas_calculation_request("RentalRecoveryOptions")]
    pub in_service: u32,
    /// Historical 1993 commercial recovery source
    /// [NullText]
    /// The historical source is not established or does not apply
    /// [SetText]
    /// The actual historical source is
    #[libertas_default($null)]
    #[libertas_constrained_by("^.allowed_commercial_1993_transition")]
    pub commercial_1993_transition: Option<crate::Commercial1993Transition>,
    /// Applicable historical recovery choices for this source
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_rental_recovery_choices")]
    pub allowed_commercial_1993_transition: Vec<i32>,

    /// How you acquired it
    #[libertas_default(Purchased)]
    pub acquired: crate::RentalAcquisition,
    /// Part of this building replaced or removed in 2026
    /// Include partial sales and other mandatory component dispositions. For replacement or removal, enter the component when electing separate partial-disposition treatment; otherwise choose no separately recognized disposition.
    /// [NullText]
    /// No building-component disposition is recognized separately in 2026
    /// [SetText]
    /// The separately recognized building-component disposition is
    pub partial: Option<crate::PartialDisposition>,
}

/// An improvement
/// A capital improvement to the building, depreciated on its own from the date it was ready to rent. It carries no land.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Improvement {
    /// Casualty or theft of this building or improvement
    /// [NullText]
    /// No casualty or theft is entered for this property
    /// [SetText]
    /// This property has casualty or theft records
    #[libertas_default($null)]
    pub casualty: Option<crate::RentalCasualty>,
    /// The same property in a saved personal casualty
    /// Choose it when you also use this dwelling personally and entered it as
    /// a property of a saved personal casualty event, so one event's $100 or
    /// $500 reduction and one property's loss apply once. That event's date and
    /// disaster declaration must match this record.
    /// [NullText]
    /// This property is not in a saved personal casualty
    /// [SetText]
    /// The saved personal casualty property is
    #[libertas_enum_source("$.casualty_properties")]
    #[libertas_default($null)]
    pub casualty_property: Option<i64>,

    /// Cost record identity
    #[libertas_hidden]
    #[libertas_default(0)]
    pub id: i64,
    /// Section 179D claim from this cost source
    /// [NullText]
    /// No current section 179D claim on this source
    /// [SetText]
    /// Claim from this source using
    #[libertas_default($null)]
    pub energy: Option<crate::RentalEnergyClaimPage>,
    /// Actual sale-date business basis and depreciation for a mixed-use sale
    /// Use this building's basis records, including actual business depreciation allowed or allowable.
    /// [NullText]
    /// Mixed-use sale records have not been supplied
    /// [SetText]
    /// The sale-date business records are
    #[libertas_default($null)]
    pub sale_history: Option<crate::RentalBuildingSaleHistory>,
    /// Your cost, including closing costs
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub cost: i64,
    /// Credit that reduces its cost
    /// Leave zero unless a tax credit was taken for this property. A clean-vehicle or fuel-cell vehicle credit (sections 30D(f)(1), 30B(h)(4) and 45W(d)(1)) and an investment credit (section 50(c)(1)) reduce its cost by the whole credit; an energy credit under section 48 reduces it by half the credit (section 50(c)(3)). Enter the reduction itself. It cannot exceed cost less land. When an automatic rehabilitation credit applies, the App figures its reduction and ignores this amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub credit_reduction: i64,
    /// Date first available to rent
    #[libertas_date_only]
    #[libertas_calculation_request("RentalRecoveryOptions")]
    pub in_service: u32,
    /// Historical 1993 commercial recovery source
    /// [NullText]
    /// The historical source is not established or does not apply
    /// [SetText]
    /// The actual historical source is
    #[libertas_default($null)]
    #[libertas_constrained_by("^.allowed_commercial_1993_transition")]
    pub commercial_1993_transition: Option<crate::Commercial1993Transition>,
    /// Applicable historical recovery choices for this source
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_rental_recovery_choices")]
    pub allowed_commercial_1993_transition: Vec<i32>,

    /// Rehabilitation credit (Form 3468)
    /// Only for an improvement that holds just the qualified rehabilitation expenditures of a certified historic building, placed in service in 2018 or later.
    /// [NullText]
    /// No rehabilitation credit claimed for this improvement
    /// [SetText]
    /// Claim the rehabilitation credit using
    #[libertas_default($null)]
    pub rehabilitation: Option<crate::RehabilitationCredit>,
    /// Part of this improvement replaced or removed in 2026
    /// Include partial sales and other mandatory component dispositions. For replacement or removal, enter the component when electing separate partial-disposition treatment; otherwise choose no separately recognized disposition.
    /// [NullText]
    /// No improvement-component disposition is recognized separately in 2026
    /// [SetText]
    /// The separately recognized improvement-component disposition is
    pub partial: Option<crate::PartialDisposition>,
}

/// Personal use
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PersonalUse {
    /// None
    No,
    /// You or your family also used it
    Yes {
        /// Time rented at a fair price
        /// Count each day rented to others at a fair price, even if you or your family also used it that day.
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 365)]
        #[libertas_physical_unit("day")]
        fair_rental_days: u32,
        /// Time of personal use that was not rented
        /// Do not count a day you also counted as rented at a fair price above.
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 365)]
        #[libertas_physical_unit("day")]
        personal_days: u32,
        /// Rented time that was also personal use
        /// Of the fair rental days, the days you or your family also used the property. They count as rental days when dividing expenses, but as personal-use days for the test of whether you used it as a home.
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 365)]
        #[libertas_physical_unit("day")]
        overlap_days: u32,
        /// Expenses of renting only, such as advertising and agency fees
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        direct_expenses: i64,
    },
}

/// More about this property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalDetails {
    /// Actual physical buildings for section 179D
    /// Each actual building once. Save these facts before selecting the building on cost-row claims.
    /// ----
    /// Physical building
    pub energy_buildings: Vec<crate::RentalEnergyBuildingFactsPage>,
    /// Original QBI history of prior at-risk property-sale losses
    /// Reconcile this rental's regular Form 6198 property-sale carry using its original QBI loss-tracking worksheets. This does not replace ordinary or AMT carry amounts.
    /// ----
    /// Loss year
    pub property_qbi_history: Vec<crate::RentalPropertyLossYear>,
    /// Improvements
    #[libertas_unordered]
    #[libertas_calculation_request("RentalRecoveryOptions")]
    /// ----
    /// Improvement
    pub improvements: Vec<Improvement>,
    /// How you took part in this rental
    /// Active participation (approving tenants, rents or repairs, with at least 10% ownership) allows up to $25,000 of a loss against other income; a real estate professional who materially participated in this rental is not passive at all.
    #[libertas_default(Active)]
    pub participation: crate::RentalParticipation,
    /// Losses suspended in earlier years
    pub suspended: SuspendedLoss,
    /// Accounting-method changes (Form 3115)
    /// Only a change whose section 481(a) adjustment still has an amount in 2026.
    #[libertas_unordered]
    /// ----
    /// Change
    pub method_changes: Vec<crate::MethodChange>,
    /// Operating expenses carried over from last year's home-use worksheet
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub home_operating_carryover: i64,
    /// Qualified business income treatment
    #[libertas_default(No)]
    pub qbi: crate::RentalBusiness,
    /// Group this rental with others for the qualified business income deduction
    #[libertas_default("")]
    pub qbi_group: String,
    /// Employees' W-2 wages
    #[libertas_default(No)]
    pub payroll: crate::BusinessPayroll,
    /// Your investment at risk
    /// Choose the investment and financing that can bear this rental's losses.
    /// [NullText]
    /// Choose your investment at risk
    #[libertas_read_only]
    #[libertas_link("AnswerRentalRisk")]
    #[libertas_default($null)]
    pub at_risk: Option<crate::RentalAtRisk>,
    /// Known at-risk answer for the question editor
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub known_at_risk: Vec<crate::RentalAtRisk>,
    /// Extraordinary personal services, an incidental rental, a pre-1988 self-rental contract, a foreign business unit with its own functional currency, an alternative depreciation election or changed ADS history, a working interest, a pre-2026 like-kind exchange, or a disposition other than an entered sale, exchange or event-date casualty retirement
    #[libertas_default(false)]
    pub special: bool,
}

/// Losses suspended in earlier years
/// Unallowed losses from earlier years' Form 8582; leave both at zero when there are none.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SuspendedLoss {
    /// Regular tax unallowed loss
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub regular: i64,
    /// Alternative minimum tax unallowed loss
    /// Enter the separately tracked amount, even if it equals the regular tax loss.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amt: i64,
    /// You actively participated in the year the loss arose
    /// Form 8582 counts a prior-year unallowed loss toward the special rental-loss allowance only for a year in which you actively participated. Leave on when there is no prior loss.
    #[libertas_default(true)]
    pub active_in_loss_year: bool,
}
