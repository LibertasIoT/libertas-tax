pub use crate::capital_history_model::{
    CapitalLossOrigin, InvestmentAmtCapitalParts, InvestmentCapitalParts, PriorAmtCapitalReturn,
    PriorCapitalReturn, PriorReturnOwners,
};
pub use crate::foreign_claim_source::ForeignTaxClaim;
pub use crate::foreign_earned_source::{ForeignQualification, ResidenceBasis};
pub use crate::foreign_source_basics::{
    ForeignAmtLimitation, ForeignInterestAllocation, ForeignOrdinaryTax, ForeignRateAdjustment,
    ForeignTaxBasis, ForeignTaxCategory, ForeignTaxDeducted, ForeignTaxIncome, ForeignWithholding,
};
pub use crate::hsa_source::HsaEligibility;
pub use crate::iso_source::{
    Form3921, Form3921More, Form3921Paper, IsoExerciseSource, IsoRecognition,
};
pub use crate::vehicle_model::BusinessMileage;
use crate::{RetirementTreatment, RolloverDestination};
// Coverage: Schedule E direct rentals: expense categories, current and prior
// regular/AMT depreciation, at-risk limits before passive limits, home-use
// worksheets, self-rental/nondepreciable recharacterization, professional and
// former-passive activities, and the source-linked disposition owners.
// Ordinary foreign real property uses required ADS: post-2017 residential
// property has a 30-year clock; earlier residential and nonresidential have 40.
// Foreign rent and expenses are translated USD and do not create section 199A
// income. Actual incidental-asset ADS and exceptional recovery histories
// retain their source boundaries. The ordinary scope answer retains foreign
// business units with their own functional currency and changed ADS history.
// Direct royalties/depletion belong to royalties.rs; Form 4835 farm rentals
// belong to farm_rental.rs; Form 8829 and Form 3115 retain their own sources.
// Sources: Pub.527/925/946; IRC168(g),199A,465,469,611–613A;
// Reg.1.469-2T(f)(3),1.469-2(f)(6),1.1411-4(g)(6).
// Tests: activity/rental_specials, rental_source_stages and rental_foreign;
// cost_recovery/unit29_rental_recovery, unit29_rental_transition.

use crate::Owner;
use crate::kernel::phase;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Actual binding written contract already selected by the person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the actual binding purchase or construction contract that the person selected for this property. Extract only its explicit contract-entry date and the printed contract or transfer record identifier. Use a YYYY-MM-DD date and a short exact identifying reference. Report missing, illegible or conflicting facts as an error. Do not decide whether the agreement is legally binding, whether a transferor qualifies, or which recovery period applies; preserve the selected source class and all unmarked property facts.",
    doc_type = "application/pdf,image/*"
)]
pub struct Commercial1993ContractSource {
    /// Date the actual binding written contract was entered
    #[libertas_date_only]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub effective: u32,
    /// Contract or transfer record identifying this property
    #[libertas_size(min = 1)]
    #[libertas_llm_doc_field]
    pub source_reference: String,
}

/// Actual construction commencement already selected by the person
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the actual construction commencement record that the person selected for this property. Extract only the explicit date construction began and the printed construction or transfer record identifier. Use a YYYY-MM-DD date and a short exact identifying reference. Report missing, illegible or conflicting facts as an error. Do not substitute a permit, approval or completion date, decide whether a transferor qualifies, or select a recovery period; preserve the selected source class and all unmarked property facts.",
    doc_type = "application/pdf,image/*"
)]
pub struct Commercial1993ConstructionSource {
    /// Actual construction commencement date
    #[libertas_date_only]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub began: u32,
    /// Construction or transfer record identifying this property
    #[libertas_size(min = 1)]
    #[libertas_llm_doc_field]
    pub source_reference: String,
}

/// Actual 1993 nonresidential recovery source
/// Applies only when this property became available to rent from May 13 through December 31, 1993. A qualified transferor transfers the contract or property rights before putting the property in service.
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
pub enum Commercial1993Transition {
    /// No qualifying pre-May 13 contract or construction commencement: ordinary 39-year recovery
    Ordinary39Year,
    /// You or a qualified transferor entered a binding written purchase or construction contract before May 13, 1993
    BindingContract(Commercial1993ContractSource),
    /// Construction began by or for you or a qualified transferor before May 13, 1993
    ConstructionBegun(Commercial1993ConstructionSource),
}

/// Rental building or structural improvement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalBuilding {
    /// Casualty or theft of this building or improvement
    /// [NullText]
    /// No casualty or theft is entered for this property
    /// [SetText]
    /// This property has casualty or theft records
    #[libertas_default($null)]
    pub casualty: Option<crate::RentalCasualty>,
    /// The same property in a saved personal casualty
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
    pub energy: Option<crate::RentalEnergyClaim>,
    /// Actual business basis and depreciation of this building in a mixed-use sale
    /// Supply the sale-date basis records when the sold property also had personal use.
    /// [NullText]
    /// Mixed-use sale records have not been supplied
    /// [SetText]
    /// The sale-date business records are
    #[libertas_default($null)]
    pub sale_history: Option<crate::RentalBuildingSaleHistory>,
    /// Your purchase or improvement cost, including capitalized closing costs
    /// Enter your ownership share. Include land here and enter its share below. Do not enter mortgage principal payments as expenses.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Land portion of that cost
    /// Land is not depreciated. Enter zero for a building improvement.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub land: i64,
    /// Date ready and available to rent
    #[libertas_date_only]
    pub in_service: u32,
    /// Historical 1993 commercial recovery source
    /// [NullText]
    /// The historical source is not established or does not apply
    /// [SetText]
    /// The actual historical source is
    #[libertas_default($null)]
    pub commercial_1993_transition: Option<crate::Commercial1993Transition>,
    /// How you acquired it
    #[libertas_default(Purchased)]
    pub acquired: RentalAcquisition,
    /// Rehabilitation credit (Form 3468)
    /// Only for an improvement row that holds just the qualified rehabilitation expenditures of a certified historic building.
    #[libertas_default($null)]
    pub rehabilitation: Option<crate::RehabilitationCredit>,
    /// Part of this building replaced or removed in 2026
    /// A roof, HVAC system or other structural component you replaced or removed, or a part you sold. Leave empty otherwise.
    #[libertas_default($null)]
    pub partial: Option<crate::PartialDisposition>,
    /// Credit that reduces its cost
    /// Leave zero unless a tax credit was taken for this property. A clean-vehicle or fuel-cell vehicle credit (sections 30D(f)(1), 30B(h)(4) and 45W(d)(1)) and an investment credit (section 50(c)(1)) reduce its cost by the whole credit; an energy credit under section 48 reduces it by half the credit (section 50(c)(3)). Enter the reduction itself. It cannot exceed cost less land. When an automatic rehabilitation credit applies, the App figures its reduction and ignores this amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub credit_reduction: i64,
}

/// Where the rental property is physically located
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalLocation {
    /// In the United States
    UnitedStates,
    /// Outside the United States
    Foreign {
        /// Country code from the property's location records
        /// Use the IRS two-letter country code, for example FR for France or JA for Japan. Do not enter a payer's address or a mutual fund code.
        // Legal format: IRS foreign-country identifiers are two letters.
        #[libertas_size(min = 2, max = 2)]
        country: String,
    },
}
impl RentalLocation {
    pub(crate) fn foreign(&self) -> bool {
        matches!(self, Self::Foreign { .. })
    }
}

/// Rental property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalRental {
    /// Physical buildings containing separately recorded improvements
    /// Enter each building once; its accepted identity is offered in the improvement picker.
    pub improvement_buildings: Vec<crate::ImprovementBuildingFacts>,
    /// Actual election out of the business interest limit
    pub interest_limit_election: Option<crate::InterestLimitElection>,
    /// Actual location of this rental property
    pub location: RentalLocation,
    /// Actual physical buildings for section 179D
    /// ----
    /// Physical building
    pub energy_buildings: Vec<crate::RentalEnergyBuildingFacts>,
    /// Original-year QBI tracking for prior at-risk property-sale losses
    /// One row per original disallowance year for this rental's separate Form 4797 loss.
    /// Do not include operating losses or losses still suspended by the passive rules.
    /// Empty with a prior at-risk property loss means its QBI history is unavailable.
    pub property_qbi_history: Vec<crate::RentalPropertyLossYear>,
    /// Accepted identities of related source components
    /// Forms remain owned and edited in Documents.
    pub related: Vec<i64>,
    /// Your share of rent received, including advance rent and retained deposits
    /// Include tenant reimbursements of expenses, lease cancellation payments and other amounts received as rent.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub rent: i64,
    /// Your share of ordinary rental expenses
    /// For property without personal use, include rental mortgage interest, property tax, insurance, repairs, management and utilities. For personal-use property, select saved Mortgage loans: their interest, points and mortgage insurance are derived once from those records. Exclude land, improvements, loan principal and depreciation; do not deduct these expenses again elsewhere.
    /// ----
    /// Rental expense
    pub expenses: Vec<RentalExpense>,
    /// Purchased building and structural improvements
    /// Depreciation is calculated from cost, land allocation, service dates and the property type below. Foreign real property uses required ADS: residential property placed after 2017 uses 30 years; earlier residential and nonresidential property use 40 years. Domestic property uses straight-line GDS with the mid-month convention: 27.5 years for single-family or multi-family property. Ordinary nonresidential buildings first available from 1987 through May 12, 1993 retain their 31.5-year schedule; later ordinary commercial buildings use 39 years. Keep the original service date when a schedule has ended.
    #[libertas_size(min = 1)]
    /// ----
    /// Building or improvement
    pub buildings: Vec<RentalBuilding>,
    /// Appliances, furniture, land improvements and other depreciable property
    /// Each asset once. No section 179 election; depreciation joins Schedule E line 18.
    /// ----
    /// Asset
    pub assets: Vec<crate::BusinessAsset>,
    /// How you took part in this rental
    /// For the seven-day and significant-services short-stay business choices, the stays' material-participation answer applies instead.
    /// This decides whether its loss is passive, may use the special rental-loss allowance, or is not passive at all.
    pub participation: RentalParticipation,
    /// Rental without the special treatments listed below
    /// No extraordinary personal services or incidental rentals, pre-February 19, 1988 binding self-rental contracts, a foreign business unit with its own functional currency, an alternative depreciation election or changed ADS history, property received in a like-kind exchange before 2026, working interests, or disposition other than an entered sale, exchange or event-date casualty retirement. Your own rental, acquisition and use records show it. No leaves this rental out of the estimate with a notice; that situation is not supported.
    pub ordinary_rental: bool,
    /// Your investment at risk
    /// Cash investment, recourse borrowing and qualifying nonrecourse real-estate financing can qualify. Choose Limited for Form 6198 amounts and prior disallowed losses; the at-risk limit applies before the passive-loss rules.
    /// [NullText]
    /// The investment at risk is not recorded
    #[libertas_default($null)]
    pub at_risk: Option<RentalAtRisk>,
    /// This rental qualifies as a U.S. trade or business for the QBI deduction
    /// Confirm section 162 trade-or-business treatment or the rental real-estate safe harbor. Choose No for investment property that does not qualify.
    pub qbi_business: RentalBusiness,
    /// Prior suspended passive losses for this property
    pub history: RentalHistory,
    /// Accounting-method changes with a section 481(a) adjustment in 2026
    /// ----
    /// Change
    pub method_changes: Vec<crate::MethodChange>,
    /// Employee payroll used by this rental business
    pub payroll: BusinessPayroll,
    /// Aggregation group for the QBI deduction
    /// Leave empty unless you elect to treat this activity as part of one combined trade or business. Give the same group name to every business or rental in the group. Each member must be at least 50% owned by the same person or group for most of the year, including the last day of the year; use the same tax year; not be a specified service trade or business; and share at least two of these: the same or customary related products or services, shared facilities or centralized business elements, and coordinated or interdependent operation. An elected group must be kept in later years. The group affects only the wage and property limit above the income threshold.
    #[libertas_size(min = 1)]
    #[libertas_default($null)]
    pub qbi_aggregation_group: Option<String>,
    /// Personal use of this property
    #[libertas_default(No)]
    pub personal_use: RentalPersonalUse,
    /// Operating expenses carried over from last year's home-use worksheet
    /// Deducted in full when the property is not a home this year.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub home_operating_carryover: i64,
    /// Last year's home-use worksheet carries
    /// From the 2025 Schedule E home-use worksheet (vacation home or rented home); they apply whether or not the property has personal use in 2026.
    /// [NullText]
    /// No home-use carry from 2025
    #[libertas_default($null)]
    pub prior_home_carry: Option<RentalHomeCarries>,
    /// Schedule E line 1b property type
    /// The property before any sale or exchange; replacement property has its own use. Residential buildings use 27.5 years, commercial and transient buildings 39 years. Land is not depreciated. Royalties have their own record.
    #[libertas_default(SingleFamily)]
    pub property_type: RentalPropertyType,
    /// Sold, exchanged or kept in 2026
    #[libertas_default(No)]
    pub sale: crate::RentalSale,
    /// Form 1099-S for the sale or exchange
    #[libertas_default($null)]
    pub statement: Option<crate::Form1099SPaper>,
    /// You actively conduct this rental as a trade or business, for the section 179 income limit
    /// Asked on the rentals page only while the return has a section 179 deduction (Reg. 1.179-2(c)).
    /// [NullText]
    /// Not asked: the return has no section 179 deduction
    #[libertas_default($null)]
    pub section179_business: Option<bool>,
}

/// Farm rental paid in crop or livestock shares (Form 4835)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalFarmRental {
    /// Physical buildings containing separately recorded improvements
    /// Enter each building once; its accepted identity is offered in the improvement picker.
    pub improvement_buildings: Vec<crate::ImprovementBuildingFacts>,
    /// Actual election out of the business interest limit
    pub interest_limit_election: Option<crate::InterestLimitElection>,
    /// Production-share income and the farm records shared with Schedule F
    /// Line 1, 3 and 6 receipts on the cash record, crop insurance, CCC loans, conservation and land sales. Conservation Reserve Program payments and prepaid supplies are always empty here.
    pub farm: crate::FarmIncome,
    /// Part II expenses
    /// ----
    /// Expense
    pub expenses: Vec<crate::FederalBusinessExpense>,
    /// Vehicles using standard mileage (line 8)
    pub vehicles: Vec<crate::BusinessMileage>,
    /// Depreciable property and actual-expense vehicles (lines 8 and 12)
    pub assets: Vec<crate::BusinessAsset>,
    /// Actively participated (line A)
    pub active_participation: bool,
    /// Individual at-risk facts for this activity (Form6198)
    /// [NullText]
    /// The investment at risk is not recorded
    #[libertas_default($null)]
    pub at_risk: Option<crate::RentalAtRisk>,
    /// Prior suspended passive losses
    pub history: RentalHistory,
    /// Qualified business income treatment
    pub qbi_business: RentalBusiness,
    /// Aggregation group for the QBI deduction
    #[libertas_size(min = 1)]
    #[libertas_default($null)]
    pub qbi_aggregation_group: Option<String>,
    /// Employee payroll used by this rental
    pub payroll: BusinessPayroll,
}

pub use crate::rental_property_type::{RentalPropertyType, ShortTermStays};

/// How a rental building was acquired
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalAcquisition {
    /// Purchased or built
    Purchased,
    /// Converted from your home or other personal use
    /// Enter your adjusted basis at conversion as the cost above, including its land part.
    Converted {
        /// Fair market value of the building, without the land, on the date it became available to rent
        /// Depreciation uses the lower of this value and the cost above less its land part (Pub. 527, Basis of Property Changed to Rental Use).
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        fair_market_value: i64,
    },
    /// Inherited
    /// The cost above is its fair market value at the death or on the alternate valuation date.
    Inherited,
    /// Received as a gift
    /// The cost above is the giver's adjusted basis plus any gift tax paid on its appreciation.
    Gift,
}

/// At-risk amounts of an activity (Form 6198)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalAtRisk {
    /// All activity deductions are at risk, with no at-risk or recapture-deduction carry
    All,
    /// Compute limited investment, financing and recapture
    Limited {
        /// Regular and AMT risk computations
        /// Required for an ordinary rental and to calculate a limited current ordinary Schedule C loss. A Form 4835 source without method facts keeps only its existing known-profit or named-loss treatment.
        /// [NullText]
        /// Method facts are not supplied
        /// [SetText]
        /// Method facts are
        #[libertas_default($null)]
        record: Option<alloc::boxed::Box<crate::ActivityRiskRecord>>,
        /// Prior ordinary loss disallowed by the at-risk rules
        /// Keep the original schedule and character. Schedule E losses enter line 19; an ordinary Schedule C prior loss needs its original-form source before it can be released.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        prior_disallowed: i64,
        /// Prior ordinary loss for alternative minimum tax
        /// The separately tracked AMT amount; zero means none.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        amt_prior_disallowed: i64,
        /// Prior Form 4797 section 1231 loss disallowed by the at-risk rules
        /// Keep this loss separate from operating expenses; it retains its property-sale character even without a sale this year.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        prior_section1231_loss: i64,
        /// Prior section 1231 loss for alternative minimum tax
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        amt_prior_section1231_loss: i64,
    },
}

/// Royalty property (Schedule E line 1b type 6)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalRoyalty {
    /// Related source component identities
    pub related: Vec<i64>,
    /// Kind of property
    pub kind: RoyaltyKind,
    /// Your share of royalties received, not on a Form 1099-MISC you entered
    /// The App adds box 2 of each 1099-MISC related to this property. Enter the gross amount before any severance tax the producer withheld; enter that tax as a line 16 expense.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub receipts: i64,
    /// Lease bonuses and advance royalties included in the royalties
    /// Amounts payable without regard to production, here or on the 1099-MISC. They are income. Oil and gas percentage depletion excludes them (section 613A(d)(5)); another mineral's percentage depletion includes them.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub advance: i64,
    /// Expenses, lines 5–19
    /// ----
    /// Royalty expense
    pub expenses: Vec<RentalExpense>,
    /// Depletion (line 18)
    #[libertas_default(No)]
    pub depletion: RoyaltyDepletion,
    /// Directly owned nonbusiness royalty with no special treatment
    /// Not current writing, inventing, artistic or other self-employment services, and not name, image and likeness income from services; not an operating (working) interest; no amortizable basis, patent or rights sale, partnership or trust distribution, foreign allocation or loss protection; your average daily production is within 1,000 barrels of oil (6,000 cubic feet of gas per barrel) and you are not a retailer or refiner of oil or gas; every expense is paid out of pocket and fully at risk. Business royalties belong in Business. The Form 1099-MISC, the lease or license and your expense records show it. No leaves this income out of this estimate.
    pub ordinary: bool,
}

/// Kind of royalty property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RoyaltyKind {
    /// Oil or gas
    /// A nonoperating interest in a domestic well. Percentage depletion is 15% of the royalties, limited to the property's income and to 65% of your taxable income (section 613A). It is not limited to the property's basis for the alternative minimum tax, but the 65% limit is refigured with alternative minimum tax amounts and any difference goes on Form 6251 line 2d.
    OilGas,
    /// Other mineral deposit
    /// Coal, lignite, sand, gravel, stone, metal ores, geothermal and other natural deposits. Percentage depletion uses the section 613(b) rate for the mineral, limited to 50% of the property's income, and the part above the property's basis is an alternative minimum tax preference.
    Mineral {
        /// Percentage depletion rate for the mineral (section 613(b))
        /// 22% sulphur and uranium; 15% gold, silver, copper, iron ore, oil shale and geothermal deposits; 14% most metal mines and other minerals; 10% coal, lignite, perlite and sodium chloride; 7.5% clay or shale for sewer pipe and brick; 5% gravel, peat, pumice, sand, scoria and certain stone and clay. Choose the rate for this mineral.
        rate: RoyaltyMineralRate,
    },
    /// Copyright, patent, trademark or other intangible
    /// Including name, image and likeness licensing that is not income from services. No depletion.
    Intangible,
}

/// Depletion of a royalty property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RoyaltyDepletion {
    /// None
    /// The basis is fully recovered, or you take no depletion; always for a copyright, patent or other intangible.
    No,
    /// Cost depletion
    /// Your adjusted basis times the units sold over the units recoverable at the start of the year (section 612).
    Cost {
        /// Adjusted basis of the property at the start of 2026
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        adjusted_basis: i64,
        /// Units estimated recoverable at the start of 2026
        /// Barrels, thousands of cubic feet, tons or other units, from the operator's or your engineer's estimate.
        #[libertas_number(min = 0)]
        recoverable_units: i64,
        /// Units sold in 2026 that you were paid for
        #[libertas_number(min = 0)]
        units_sold: i64,
    },
    /// Percentage depletion
    /// The App takes the greater of cost and percentage depletion after the limits; enter the cost facts so it can compare (a zero basis gives zero cost depletion).
    Percentage {
        /// Adjusted basis of the property at the start of 2026
        /// Also the limit of the alternative minimum tax depletion of a mineral other than oil and gas.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        adjusted_basis: i64,
        /// Units estimated recoverable at the start of 2026
        /// Barrels, thousands of cubic feet, tons or other units, from the operator's or your engineer's estimate.
        #[libertas_number(min = 0)]
        recoverable_units: i64,
        /// Units sold in 2026 that you were paid for
        #[libertas_number(min = 0)]
        units_sold: i64,
        /// Oil and gas depletion carried from 2025 because of the 65% limit
        /// From your 2025 records; zero for another mineral or when none was carried.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        carryover: i64,
        /// The same carryover for the alternative minimum tax
        /// Enter the separately tracked amount, even if it equals the regular-tax carryover.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        amt_carryover: i64,
    },
}

/// How you took part in a rental
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RentalParticipation {
    /// You did not actively participate
    /// Choose this when you owned less than 10% of the property, did not make management decisions, or leave the choice to a manager. A loss then offsets only passive income.
    NotActive,
    /// You actively participated
    /// You made management decisions in a significant and bona fide sense, such as approving tenants, setting rents or approving repairs, and owned at least 10% of the property all year. Up to $25,000 of a loss can then offset your other income.
    Active,
    /// Real estate professional and material participant, without current active-participation eligibility
    /// Your professional status is recorded on People, and you materially participated in this rental, counting your spouse's work. Choose this when you did not make significant bona fide management decisions or you and your spouse did not own at least 10% by value throughout 2026. Current income or loss is nonpassive; remaining prior passive losses cannot use the special allowance.
    Professional,
    /// Real estate professional and material participant, with current active-participation eligibility
    /// Your professional status is recorded on People, and you materially participated in this rental. You or your spouse also made significant bona fide management decisions and together owned at least 10% by value throughout 2026; you are not a limited partner. Remaining prior passive losses may use the special allowance only when you actively participated in the loss year too.
    ProfessionalActive,
}

impl RentalParticipation {
    pub(crate) fn professional(self) -> bool {
        matches!(self, Self::Professional | Self::ProfessionalActive)
    }

    pub(crate) fn active(self) -> bool {
        matches!(self, Self::Active | Self::ProfessionalActive)
    }
}

/// Last year's home-use worksheet carries for one rental
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalHomeCarries {
    /// Regular-tax casualty and depreciation carried over from last year's home-use worksheet
    /// Enter the combined third-tier carry from the actual prior worksheet, including a documented zero.
    pub home_depreciation_carryover: crate::SourceCents,
    /// AMT casualty and depreciation carried over from last year's home-use worksheet
    /// Use the independently tracked AMT worksheet, even when it matches regular tax.
    pub amt_home_depreciation_carryover: crate::SourceCents,
    /// Casualty part of the regular-tax combined home-use carry
    /// The actual prior worksheet's casualty amount; it cannot exceed its combined carry.
    pub home_casualty_carryover: crate::SourceCents,
    /// Casualty part of the AMT combined home-use carry
    /// Use the actual AMT casualty amount, not the regular-tax amount.
    pub amt_home_casualty_carryover: crate::SourceCents,
}
impl RentalHomeCarries {
    // No worksheet carry: every amount is a documented zero.
    pub(crate) const NONE: Self = Self {
        home_depreciation_carryover: crate::SourceCents::NotReported,
        amt_home_depreciation_carryover: crate::SourceCents::NotReported,
        home_casualty_carryover: crate::SourceCents::NotReported,
        amt_home_casualty_carryover: crate::SourceCents::NotReported,
    };
}
impl FederalRental {
    // The prior worksheet carries the rules read; none is a documented zero.
    pub(crate) fn home_carries(&self) -> &RentalHomeCarries {
        self.prior_home_carry.as_ref().unwrap_or(&RentalHomeCarries::NONE)
    }
}

/// Personal use of a rental property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalPersonalUse {
    /// No personal use this year
    No,
    /// Personal use this year
    Yes {
        /// Time rented at a fair rental price
        /// Count each day rented to others at a fair price, even if you or your family also used it that day. Do not count days available but not rented.
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 365)]
        #[libertas_physical_unit("day")]
        fair_rental_days: u32,
        /// Time of personal use that was not rented
        /// Days used by you, another owner or a family member of either, anyone under a home-swap arrangement, or anyone paying less than a fair price, other than a day you also counted as rented at a fair price above. Do not count days spent working substantially full time on repairs and maintenance.
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 365)]
        #[libertas_physical_unit("day")]
        personal_days: u32,
        /// Rented time that was also personal use
        /// Of the fair rental days above, the days you or someone in the groups listed for personal use also used the property. They divide the expenses as rental days, but they are personal-use days, not fair-rental days, when testing whether you used the property as a home (section 280A(d)(1)).
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 365)]
        #[libertas_physical_unit("day")]
        overlap_days: u32,
        /// Expenses of renting only, not divided by days
        /// For example, advertising, rental agency fees and office supplies. Enter shared property taxes, property insurance, repairs and utilities as rental expenses above; the App divides them by days. Enter mortgage interest, points and mortgage insurance once on Mortgage and select those loans on the rental.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        direct_expenses: i64,
    },
}

/// Schedule E Part I expense line
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalExpenseCategory {
    /// Line 5 advertising
    Advertising,
    /// Line 6 auto and travel
    AutoAndTravel,
    /// Line 7 cleaning and maintenance
    CleaningAndMaintenance,
    /// Line 8 commissions
    Commissions,
    /// Line 9 insurance
    Insurance,
    /// Line 10 legal and other professional fees
    LegalAndProfessional,
    /// Line 11 management fees
    ManagementFees,
    /// Line 12 mortgage interest paid to banks or other financial institutions
    MortgageInterest,
    /// Line 13a vehicle-loan interest
    VehicleLoanInterest,
    /// Line 13b other interest
    OtherInterest,
    /// Line 14 repairs
    Repairs,
    /// Line 15 supplies
    Supplies,
    /// Line 16 taxes
    TaxesAndLicenses,
    /// Line 17 utilities
    Utilities,
    /// Line 19 other
    Other {
        /// Description of this other expense
        #[libertas_size(min = 1)]
        label: String,
    },
}

impl RentalExpenseCategory {
    // Order matches the printed Schedule E line; used only to key the per-category
    // return-line totals below, never persisted or compared across categories.
    pub(crate) const COUNT: usize = 15;
    fn index(&self) -> usize {
        match self {
            Self::Advertising => 0,
            Self::AutoAndTravel => 1,
            Self::CleaningAndMaintenance => 2,
            Self::Commissions => 3,
            Self::Insurance => 4,
            Self::LegalAndProfessional => 5,
            Self::ManagementFees => 6,
            Self::MortgageInterest => 7,
            Self::VehicleLoanInterest => 8,
            Self::OtherInterest => 9,
            Self::Repairs => 10,
            Self::Supplies => 11,
            Self::TaxesAndLicenses => 12,
            Self::Utilities => 13,
            Self::Other { .. } => 14,
        }
    }
    // (return-line id suffix, explanation resource key) in `index()` order.
    pub(crate) const LINES: [(&'static str, &'static str); Self::COUNT] = [
        ("line5_advertising", "EXPLAIN_RENTAL_N_EXPENSE_LINE5_ADVERTISING"),
        ("line6_auto_and_travel", "EXPLAIN_RENTAL_N_EXPENSE_LINE6_AUTO_AND_TRAVEL"),
        (
            "line7_cleaning_and_maintenance",
            "EXPLAIN_RENTAL_N_EXPENSE_LINE7_CLEANING_AND_MAINTENANCE",
        ),
        ("line8_commissions", "EXPLAIN_RENTAL_N_EXPENSE_LINE8_COMMISSIONS"),
        ("line9_insurance", "EXPLAIN_RENTAL_N_EXPENSE_LINE9_INSURANCE"),
        ("line10_legal_and_professional", "EXPLAIN_RENTAL_N_EXPENSE_LINE10_LEGAL_AND_PROFESSIONAL"),
        ("line11_management_fees", "EXPLAIN_RENTAL_N_EXPENSE_LINE11_MANAGEMENT_FEES"),
        ("line12_mortgage_interest", "EXPLAIN_RENTAL_N_EXPENSE_LINE12_MORTGAGE_INTEREST"),
        ("line13a_vehicle_loan_interest", "EXPLAIN_RENTAL_N_EXPENSE_LINE13A_VEHICLE_LOAN_INTEREST"),
        ("line13b_other_interest", "EXPLAIN_RENTAL_N_EXPENSE_LINE13B_OTHER_INTEREST"),
        ("line14_repairs", "EXPLAIN_RENTAL_N_EXPENSE_LINE14_REPAIRS"),
        ("line15_supplies", "EXPLAIN_RENTAL_N_EXPENSE_LINE15_SUPPLIES"),
        ("line16_taxes", "EXPLAIN_RENTAL_N_EXPENSE_LINE16_TAXES"),
        ("line17_utilities", "EXPLAIN_RENTAL_N_EXPENSE_LINE17_UTILITIES"),
        ("line19_other", "EXPLAIN_RENTAL_N_EXPENSE_LINE19_OTHER"),
    ];
}

/// Rental expense
/// Enter each expense or bookkeeping category once. Exclude land, improvements, loan principal
/// and depreciation; the App computes depreciation from the building records above.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalExpense {
    /// Schedule E line
    pub category: RentalExpenseCategory,
    /// Rental expense amount
    /// Enter your share of this expense for the property.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Expense treatment
    pub treatment: crate::BusinessExpenseKind,
}

// Calculation-only row: the source amount and any same-row loan projection
// stay exact through the existing owner-first division and expense treatment.
#[derive(Clone)]
pub(crate) struct RentalExpenseAmount {
    pub category: RentalExpenseCategory,
    pub amount: crate::kernel::ratio::exact::Exact,
    pub treatment: crate::BusinessExpenseKind,
}

impl RentalExpense {
    // Group before the prescribed cent rounding. Printed categories below
    // deliberately retain their independent per-item rounding boundary.
    pub(crate) fn exact_rows(expenses: &[Self]) -> alloc::vec::Vec<RentalExpenseAmount> {
        expenses
            .iter()
            .map(|expense| RentalExpenseAmount {
                category: expense.category.clone(),
                amount: crate::kernel::ratio::exact::whole(expense.amount),
                treatment: expense.treatment,
            })
            .collect()
    }
    pub(crate) fn total_exact(expenses: &[Self]) -> crate::kernel::ratio::exact::Exact {
        Self::total_amounts(&Self::exact_rows(expenses))
    }
    pub(crate) fn total_amounts(
        expenses: &[RentalExpenseAmount],
    ) -> crate::kernel::ratio::exact::Exact {
        use crate::kernel::ratio::exact;
        let mut costs = core::array::from_fn::<_, 3, _>(|_| exact::whole(0));
        for expense in expenses {
            let index = match expense.treatment {
                crate::BusinessExpenseKind::Ordinary
                | crate::BusinessExpenseKind::DeMinimis { .. } => 0,
                crate::BusinessExpenseKind::Meals => 1,
                crate::BusinessExpenseKind::TransportationMeals => 2,
                crate::BusinessExpenseKind::Nondeductible => continue,
            };
            costs[index] += &expense.amount;
        }
        &costs[0]
            + exact::rounded_exact(&(&costs[1] * exact::ratio(1, 2).expect("positive denominator")))
            + exact::rounded_exact(&(&costs[2] * exact::ratio(4, 5).expect("positive denominator")))
    }

    pub(crate) fn category_totals_exact(
        expenses: &[Self],
    ) -> [crate::kernel::ratio::exact::Exact; RentalExpenseCategory::COUNT] {
        Self::category_amounts(&Self::exact_rows(expenses))
    }
    pub(crate) fn category_amounts(
        expenses: &[RentalExpenseAmount],
    ) -> [crate::kernel::ratio::exact::Exact; RentalExpenseCategory::COUNT] {
        use crate::kernel::ratio::exact;
        let mut totals = core::array::from_fn(|_| exact::whole(0));
        for expense in expenses {
            let amount = expense.amount.clone();
            let contribution = match expense.treatment {
                crate::BusinessExpenseKind::Ordinary
                | crate::BusinessExpenseKind::DeMinimis { .. } => amount,
                crate::BusinessExpenseKind::Meals => exact::rounded_exact(
                    &(amount * exact::ratio(1, 2).expect("positive denominator")),
                ),
                crate::BusinessExpenseKind::TransportationMeals => exact::rounded_exact(
                    &(amount * exact::ratio(4, 5).expect("positive denominator")),
                ),
                crate::BusinessExpenseKind::Nondeductible => exact::whole(0),
            };
            totals[expense.category.index()] += contribution;
        }
        totals
    }
}

/// Foreign employment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignWages {
    /// Foreign-source employee pay received in 2026, in U.S. dollars
    /// For services physically performed abroad. Do not enter pay already included on a W-2 entry. Use applicable exchange rates from payment records.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wages: i64,
}

/// Foreign tax credit or deduction choice
// The page's form of ForeignTaxClaim: the same answers, with the choices of
// businesses, sales and situations an EnumSource of the request. The stored claim
// is plain keys (the persisted data has no request to host them).
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignTaxPage {
    /// Credit for ordinary foreign income using Form 1116
    Ordinary {
        /// How you claim foreign taxes — Form 1116 Part II, Paid or Accrued
        basis: ForeignTaxBasis,
        /// Foreign income and tax by country
        /// One row for each country and category of foreign income already included once in Income, with the foreign tax on it. Employee pay, interest, dividends and royalties are entered with their amounts; a business run abroad and foreign-source investment sales are chosen from their own entries. A K-1's Schedule K-3 is read from its K-1 page; do not enter it here. The App allocates your deductions: the standard or itemized deductions by gross income, home mortgage interest by the Form 1116 worksheet, investment interest by your portfolio records or the $5,000 small-foreign-income choice, state income taxes by their source records; the expenses of your U.S. businesses, rentals, farms and gambling stay with U.S. income.
        /// ----
        /// Country and income category
        sources: Vec<ForeignSourceRow>,
        /// Foreign tax situations that apply
        /// Check each that applies to you, or to your spouse on a joint return; leave all unchecked when none does. This estimate does not figure them, so a checked situation leaves the foreign tax credit out, with a note. A contested tax is not a situation: leave it out of the tax amounts until it is settled.
        #[libertas_enum_source("$.foreign_situations")]
        // Each of the nine foreign-tax situation codes occurs at most once.
        #[libertas_size(max = 9)]
        situations: Vec<u32>,
        /// Unused foreign taxes from prior returns
        history: crate::ForeignCreditHistory,
        /// Prior-year treatment of newly generated excess taxes
        carryback: crate::ForeignCarryback,
        /// Alternative minimum tax foreign-credit limitation
        amt_limitation: ForeignAmtLimitation,
        /// Interest allocation method
        #[libertas_default(GrossIncome)]
        interest_allocation: ForeignInterestAllocation,
        /// Portfolio asset bases for investment-interest source allocation
        #[libertas_default(Unreviewed)]
        investment_assets: crate::ForeignInvestmentAssetsPage,
        /// Prior loss accounts and current recapture choices
        #[libertas_default(None)]
        loss_history: crate::ForeignLossHistory,
        /// Source records for itemized state/local income taxes
        #[libertas_default(Unreviewed)]
        state_taxes: crate::ForeignStateTaxesPage,
        /// Qualified dividend and capital gain adjustment
        #[libertas_default(Exception)]
        rate_adjustment: ForeignRateAdjustment,
    },
    /// No foreign tax credit or deduction
    No,
    /// Small foreign tax credit from payee statements
    Statements {
        /// All foreign gross income is passive income reported on qualified payee statements
        /// Includes all foreign income and taxes for both spouses on a joint return. This election is limited to $300 ($600 jointly), with no carryover to or from this election year. No refundable, contested, non-income or otherwise ineligible foreign taxes. Check every foreign income statement and your foreign-tax records. No claims no foreign tax credit, with a review note.
        eligible: bool,
    },
    /// Deduct foreign income taxes on Schedule A line 6 instead
    /// No foreign tax credit is claimed this year. Foreign income taxes are outside the state and local tax limit and help only when you itemize. Retained tax on your Forms 1099-INT, 1099-DIV, qualified estate or trust payee statements and Schedules K-3 is included already, subject to the existing source and K-1 basis limits. The credit usually saves more; choose it yourself if you prefer it.
    Deduction {
        /// Other foreign income taxes paid in 2026
        /// Add only income taxes not already included from a Form 1099-INT, 1099-DIV, qualified estate or trust payee statement or Schedule K-3. Exclude taxes already deducted by a business, taxes credited in an earlier year, contested or refundable amounts, U.S. territory taxes (Taxes you paid), foreign real estate taxes, and sales or value-added taxes.
        #[libertas_unordered]
        /// ----
        /// Foreign tax
        taxes: Vec<ForeignTaxDeductedPage>,
    },
}
/// Foreign tax
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignTaxDeductedPage {
    /// Payer and country
    #[libertas_size(min = 1)]
    pub label: String,
    /// Income it was paid on
    pub income: ForeignTaxIncomePage,
    /// Tax paid in 2026, in U.S. dollars
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub tax: i64,
    /// People who paid this investment or passive-income tax
    /// Current earned-income taxes keep the earner specified above.
    /// [NullText]
    /// The payer is not established
    /// [SetText]
    /// This investment or passive tax was paid by
    #[libertas_default($null)]
    pub owners: Option<crate::DeductionOwners>,
}

/// Income it was paid on
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ForeignTaxIncomePage {
    /// Current-year pay for work or self-employment abroad
    /// The part on income excluded on Form 2555 is not deductible; the App takes it out for this person.
    Earned {
        /// Person who earned the income
        #[libertas_default(Taxpayer)]
        #[libertas_constrained_by("$.allowed_owners")]
        owner: Owner,
    },
    /// Interest, dividends, nonbusiness royalties or investment gains
    /// Income from property held for investment, included in net investment income and the Form 4952 investment-income pool.
    Investment,
    /// Passive business or rental income subject to net investment income tax
    /// These taxes reduce net investment income for Form 8960. Expenses used in passive activity income or loss do not enter the Form 4952 investment-expense pool.
    Passive,
    /// Other taxable income
    /// Includes earlier-year earnings taxed now. Exclude any tax on income excluded from U.S. tax in the year it was earned.
    Other,
}

impl From<ForeignTaxDeducted> for ForeignTaxDeductedPage {
    fn from(row: ForeignTaxDeducted) -> Self {
        Self {
            label: row.label,
            tax: row.tax,
            owners: row.owners,
            income: match row.income {
                ForeignTaxIncome::Earned { owner } => ForeignTaxIncomePage::Earned { owner },
                ForeignTaxIncome::Investment => ForeignTaxIncomePage::Investment,
                ForeignTaxIncome::Passive => ForeignTaxIncomePage::Passive,
                ForeignTaxIncome::Other => ForeignTaxIncomePage::Other,
            },
        }
    }
}
impl From<ForeignTaxDeductedPage> for ForeignTaxDeducted {
    fn from(row: ForeignTaxDeductedPage) -> Self {
        Self {
            label: row.label,
            tax: row.tax,
            owners: row.owners,
            income: match row.income {
                ForeignTaxIncomePage::Earned { owner } => ForeignTaxIncome::Earned { owner },
                ForeignTaxIncomePage::Investment => ForeignTaxIncome::Investment,
                ForeignTaxIncomePage::Passive => ForeignTaxIncome::Passive,
                ForeignTaxIncomePage::Other => ForeignTaxIncome::Other,
            },
        }
    }
}

/// Incentive stock option exercise
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IsoExercise {
    /// Accepted document identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Printed recipient and issuer names
    /// [NullText]
    /// No printed names entered
    /// [SetText]
    /// Printed names
    #[libertas_default($null)]
    pub printed: Option<crate::PrintedDocumentNames>,
    /// Employee
    pub owner: Owner,
    /// Employer or Form 3921 label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Exercise and stock-transfer date
    #[libertas_date_only]
    pub exercised: u32,
    /// AMT recognition event
    pub recognition: IsoRecognition,
    /// Form 3921 box 3 exercise price per share
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exercise_price: i64,
    /// Form 3921 box 4 fair market value per share on the exercise date
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub market_price: i64,
    /// Amount paid for the option itself
    /// Total option purchase cost for all the shares of this exercise, separate from the exercise price and commissions. Enter zero if nothing was paid for the option. Exclude interest.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub option_cost: i64,
    /// Form 3921 box 5 number of shares transferred
    /// Enter up to six decimal places. Quantity is normalized to integer millionths before the exercise spread is rounded to cents.
    #[libertas_number(min = 0.000001, step = 0.000001)]
    pub shares: f64,
    /// Form 3921 box 1 date option granted
    #[libertas_date_only]
    pub granted: u32,
    /// Form 3921 box 6 corporation whose stock was transferred, if not the employer
    #[libertas_size(min = 1)]
    pub corporation: Option<String>,
    /// Shares from this exercise sold or otherwise disposed of in 2026
    /// A disposition in the exercise year removes those shares from the AMT adjustment.
    #[libertas_number(min = 0, step = 0.000001)]
    #[libertas_default(0)]
    pub sold: f64,
}

/// Source classification for ordinary Form 1116 income and taxes
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignSourceCategory {
    /// Employee pay
    Employment,
    /// Foreign income tax paid now on earlier employee pay
    /// Tax first paid in 2026 on employee pay reported on an earlier U.S. return, not credited, deducted or refunded before. It is general category tax with no 2026 income: enter zero income. Only when you claim foreign taxes when paid.
    PriorEmployeeWageTax {
        /// Year the foreign employee pay was included on a U.S. return
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 1900, max = 2025)]
        income_year: u32,
    },
    /// Foreign income tax paid now on earlier interest or ordinary dividends
    /// Tax first paid in 2026 on interest or nonqualified dividends reported on an earlier U.S. return, not credited, deducted or refunded before. Tax on zero passive income belongs to the general category: enter zero income. Only when you claim foreign taxes when paid.
    PriorPassiveTax {
        /// Year the passive income was included on a U.S. return
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 1900, max = 2025)]
        income_year: u32,
    },
    /// Interest, dividends, royalties or investment sales
    Passive {
        /// Foreign withholding treatment on the payment
        /// Use the rate on the payment record, not total foreign tax divided by net income. Keep payments with different withholding treatments in separate records. The App groups direct passive income across payers before testing high-tax treatment.
        withholding: ForeignWithholding,
        /// Qualified dividends included in this income
        /// The foreign-source part of Form 1099-DIV box 1b already counted in the income above. The App applies the qualified dividend rate adjustment, or the adjustment exception you choose below.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        qualified: i64,
        /// Foreign-source sales in this country
        /// Choose the investment sales, already entered under Investment sales, whose gain or loss is from this country: real property located there, or personal property sold while your tax home was abroad with at least 10% foreign tax paid on the gain. Your other sales are U.S. source.
        #[libertas_enum_source("$.capital_sales")]
        sales: Vec<i64>,
    },
    /// Business run abroad
    /// A Schedule C business whose services were performed wholly abroad. Its gross income and expenses come from the business itself: enter zero income here.
    Business {
        /// Business
        #[libertas_enum_source("$.foreign_businesses")]
        business: i64,
        /// The business keeps its own books and records
        /// A separate set of books of original entry and ledger accounts for this business, such as its own receipts and disbursements journal. Yes makes its income foreign branch category income (Form 1116 box b); No makes it general category income (box d).
        separate_books: bool,
    },
}

/// Country and category tax record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignSourceRow {
    /// Country and payer label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Country code — Form 1116 line i
    /// The IRS two-letter code of the country or U.S. territory the income is from and the tax was paid to, for example FR for France or JA for Japan (IRS.gov/CountryCodes). For taxes a mutual fund passed through on Form 1099-DIV, enter RIC.
    // Country identifiers use two- or three-letter ISO country codes.
    #[libertas_size(min = 2, max = 3)]
    pub country: String,
    /// Whose income and tax
    /// On a joint return, choose the spouse who earned the income and paid the tax. Foreign earned income excluded on Form 2555 is taken out of that person's employment sources.
    #[libertas_default(Taxpayer)]
    #[libertas_constrained_by("$.allowed_owners")]
    pub owner: Owner,
    /// Category of the foreign-source income already entered
    pub category: ForeignSourceCategory,
    /// Current-year gross income in this country and category
    /// Already included once in Income: interest, dividends, royalties or employee pay. Enter zero for a prior-year tax paid in 2026 and for a business run abroad.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub income: i64,
    /// Eligible foreign income tax for 2026 in this country and category
    /// Claiming taxes when paid: the tax paid in 2026, at the exchange rate on the date paid or withheld. Claiming them when accrued: the tax for 2026, at the 2026 average rate (the payment-date rate when paid before 2026, more than 2 years after it, or in an inflationary currency). Only tax you legally owe: leave out amounts you can get refunded, for example under a treaty, and a contested tax until it is settled. For a prior-year choice, only tax first paid in 2026 on the earlier income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub tax: i64,
}
/// Rental trade-or-business treatment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalBusiness {
    /// Investment only, not a qualified trade or business
    No,
    /// Evaluate as a qualified trade or business
    /// The section 162 trade-or-business or rental safe-harbor requirements are satisfied. A triple-net lease is excluded from the rental real-estate safe harbor but may qualify as a section 162 trade or business. Rental to a commonly controlled trade or business can qualify under section 199A.
    Yes {
        /// You materially participated under the section 469 tests
        /// Active management alone is not enough. This controls the minimum QBI deduction for an actively conducted business; it does not turn ordinary rental income into nonpassive income. Use your work-time, service and participation records to apply the material-participation tests. No takes no minimum QBI deduction.
        material_participation: bool,
    },
}

/// Calculated carryover caps for the edited prior-return source document
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentCapitalCap {
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    pub short: i64,
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    pub long: i64,
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    pub amt_short: i64,
    #[libertas_money("USD", 2)]
    #[libertas_read_only]
    pub amt_long: i64,
}

impl crate::FederalRetirement {
    // A distribution outside Form 8606 line 7: the return of an excess contribution (code 8, and
    // the earnings it carries) and the recharacterizations (N, R) and an earlier year's
    // return (P) are not distributions of the IRA's basis (i8606, "a distribution doesn't
    // include ... return of certain contributions").
    pub(crate) fn outside_form_8606(&self) -> bool {
        // An employer's Roth SEP or Roth SIMPLE contribution is reported as converted money the
        // person never held in a traditional IRA of their own.
        self.ordinary_distribution == Some(RetirementTreatment::EmployerRothContribution)
            || self.box7
                & (crate::DistributionCodes::CODE_8.0
                    | crate::DistributionCodes::CODE_N.0
                    | crate::DistributionCodes::CODE_P.0
                    | crate::DistributionCodes::CODE_R.0)
                != 0
    }
    pub(crate) fn calculated_plan_basis(&self) -> bool {
        matches!(
            self.taxable,
            Some(crate::RetirementTaxable::QualifiedWithdrawal { .. })
                | Some(crate::RetirementTaxable::FullDischarge { .. })
                | Some(crate::RetirementTaxable::PropertyProceeds { .. })
        )
    }
    pub(crate) fn pension_basis(&self) -> Option<(crate::PensionBasis, i64)> {
        let (mut basis, gross) = self.taxable.as_ref()?.pension_basis(self.gross)?;
        if basis.cost.is_none() && basis.started / 10000 == 2026 {
            basis.cost = self.box_9b;
        }
        Some((basis, gross))
    }
    pub(crate) fn supported_plan_rollover(&self) -> bool {
        if let Some(crate::RetirementTaxable::PropertyProceeds { ref data }) = self.taxable {
            return (0..=data.proceeds).contains(&self.rolled_amount());
        }
        self.rolled_amount() <= self.taxable_before_rollover()
            || matches!(
                self.ordinary_distribution,
                Some(RetirementTreatment::Rollover {
                    destination: RolloverDestination::Ira
                        | RolloverDestination::IraNextYear
                        | RolloverDestination::DirectEmployerPlan { .. }
                        | RolloverDestination::Roth,
                    ..
                })
            )
    }
    pub(crate) fn rolled_basis(&self) -> i64 {
        match &self.taxable {
            Some(crate::RetirementTaxable::PropertyProceeds { .. }) => 0,
            _ => phase::floor(self.rolled_amount() - self.taxable_before_rollover(), 0),
        }
    }
    pub(crate) fn rolled_amount(&self) -> i64 {
        match self.ordinary_distribution {
            Some(RetirementTreatment::Rollover { amount, .. }) => amount,
            _ => 0,
        }
    }
    // The taxable part of a first-time-homebuyer distribution (Form 8915-F Part IV) repaid in
    // 2026, within the amount distributed. Form 8606 line 7 leaves it out (i8606 line 7).
    pub(crate) fn home_repaid(&self) -> i64 {
        match self.disaster {
            crate::DisasterDistribution::HomePurchase { taxable_repaid, .. } => phase::floor(
                phase::cap(
                    taxable_repaid,
                    [self.gross - self.rolled_amount() - self.converted_amount()],
                ),
                0,
            ),
            _ => 0,
        }
    }
    pub(crate) fn outstanding_rollover(&self) -> i64 {
        match self.ordinary_distribution {
            Some(RetirementTreatment::Rollover {
                amount,
                destination: RolloverDestination::IraNextYear,
                ..
            }) if self.kind == crate::RetirementKind::Ira => amount,
            _ => 0,
        }
    }
    pub(crate) fn converted_amount(&self) -> i64 {
        match self.ordinary_distribution {
            Some(RetirementTreatment::Conversion { amount, .. }) => amount,
            _ => 0,
        }
    }
    pub(crate) fn designated_roth_qualified(&self) -> bool {
        crate::DistributionCodes::designated_roth_qualified(
            self.box7,
            self.form_boxes.first_roth_year,
        )
    }
    pub(crate) fn taxable_before_rollover(&self) -> i64 {
        if self.form_boxes.taxable_not_determined && self.designated_roth_qualified() {
            return 0;
        }
        // Basis selections use gross provisionally; the complete estimate must resolve
        // their owner-level worksheet before returning a result.
        match self.taxable {
            // Unanswered sources are withheld by the source availability guard.
            None => 0,
            Some(crate::RetirementTaxable::Payer) => {
                if !self.form_boxes.taxable_not_determined {
                    self.box_2a
                } else if self.ira {
                    // The IRA owner's Form 8606 calculation replaces this when basis exists.
                    self.gross
                } else {
                    0 // An undetermined pension needs the selected recovery worksheet.
                }
            }
            Some(crate::RetirementTaxable::CivilServiceDeath {
                data: crate::SurvivorLumpSum { cost, recovered, .. },
            }) => phase::floor(self.gross - (cost - recovered), 0),
            Some(crate::RetirementTaxable::PropertyProceeds { .. }) => self.gross,
            Some(crate::RetirementTaxable::IraBasis) => self.gross,
            // The Inherited IRA record's group decides (inherited_ira.rs).
            Some(crate::RetirementTaxable::Inherited { .. }) => 0,
            Some(crate::RetirementTaxable::FullDischarge {
                data: crate::ContractDischarge { investment, .. },
            }) => phase::floor(self.gross - investment, 0),
            Some(crate::RetirementTaxable::QualifiedWithdrawal {
                data: crate::PlanDistribution { value, investment, .. },
            }) => {
                self.gross
                    - phase::cap(
                        crate::kernel::ratio::fraction(self.gross, investment, value.max(1)),
                        [self.gross],
                    )
            }
            Some(crate::RetirementTaxable::LegacyAnnuity { ref data }) => data.taxable(self.gross),
            Some(crate::RetirementTaxable::DeferredWithdrawal { ref data }) => {
                let [value, investment] =
                    crate::retirement::annuity_balances(data.value, data.investment, &data.related);
                let taxable = (value - investment)
                    .max(crate::kernel::ratio::exact::whole(0))
                    .min(crate::kernel::ratio::exact::whole(self.gross));
                // A valid source's taxable portion is bounded by its native gross.
                crate::kernel::ratio::exact::rounded(&taxable)
                    .expect("annuity taxable portion is within the original gross")
            }
            Some(crate::RetirementTaxable::GeneralAnnuity { data }) => self.gross - data.excluded(),
            Some(crate::RetirementTaxable::VariableAnnuity { data }) => {
                self.gross - data.excluded(self.gross)
            }
            Some(crate::RetirementTaxable::Railroad { .. }) => {
                self.gross - self.pension_basis().map_or(0, |(basis, gross)| basis.excluded(gross))
            }
            Some(crate::RetirementTaxable::ChildAnnuity { ref data }) => {
                self.gross - data.result().0
            }
            Some(crate::RetirementTaxable::Simplified { ref data }) => {
                self.gross - data.excluded(self.gross)
            }
        }
    }
    pub(crate) fn taxable_income(&self) -> i64 {
        if let Some(crate::RetirementTaxable::PropertyProceeds { ref data }) = self.taxable {
            return data.pension(self.gross, self.rolled_amount());
        }
        let taxable = self.taxable_before_rollover();
        match self.ordinary_distribution {
            // Pretax money rolled into a Roth IRA or designated Roth account is taxed (sections
            // 408A(d)(3)(A), 402A(c)(4)(A)); a designated Roth account distribution (code B) is
            // earnings first, so what is rolled over comes out of the earnings (Pub. 575).
            Some(RetirementTreatment::Rollover {
                destination: RolloverDestination::Roth, ..
            }) if self.box7 & crate::DistributionCodes::CODE_B.0 == 0 => taxable,
            Some(RetirementTreatment::Rollover { amount, .. }) => {
                phase::floor(taxable - amount + self.rolled_basis(), 0)
            }
            // A corrective distribution is taxed as printed, less an excess deferral already
            // counted from the W-2s (the line adds that part in retirement::early_tax's caller).
            Some(RetirementTreatment::Corrective { .. }) => taxable,
            // The earlier year's return, or the other kind of IRA, owns these.
            _ if self.box7
                & (crate::DistributionCodes::CODE_P.0 | crate::DistributionCodes::CODE_R.0)
                != 0 =>
            {
                0
            }
            // Form 4972 takes the elected amount out of income; Part II alone leaves the
            // ordinary part.
            Some(RetirementTreatment::LumpSum { ref data }) => data.ordinary_part(taxable, self),
            _ => taxable,
        }
    }
}

pub use crate::elderly_source::ExemptPension;

/// Prior suspended rental loss
/// Loss from prior filed returns; both zero when there is none. Enter it only when the records are complete and the activity's participation and QBI treatment are unchanged: same property and ownership; no basis or at-risk limitation. Active-participation losses require active participation in their original years. For a QBI business, all entered losses arose after 2017 and retain the same QBI character.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalHistory {
    /// Regular-tax unallowed loss from this property's prior Form 8582 records
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub regular: i64,
    /// AMT unallowed loss from this property's prior AMT Form 8582 records
    /// Enter the separately tracked amount, even if it equals the regular-tax loss.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amt: i64,
    /// You actively participated in the year the loss arose
    /// Form 8582 Part IV counts a prior-year unallowed loss toward the special allowance only for a year in which you actively participated. Leave on when there is no prior loss.
    #[libertas_default(true)]
    pub active_in_loss_year: bool,
}
impl RentalHistory {
    /// No prior suspended loss.
    #[cfg(test)]
    pub(crate) const NONE: Self = Self { regular: 0, amt: 0, active_in_loss_year: true };
    pub(crate) fn balance(&self, alternative: bool) -> i64 {
        if alternative { self.amt } else { self.regular }
    }
    /// Whether any prior suspended loss is entered.
    pub(crate) fn any(&self) -> bool {
        self.regular != 0 || self.amt != 0
    }
}

/// Employee wages for the QBI limit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessPayroll {
    /// No employees
    No,
    /// W-2 payroll
    Yes {
        /// Forms W-2 issued by this business
        #[libertas_size(min = 1)]
        /// ----
        /// Employee statement
        statements: Vec<PayrollW2>,
        /// Wages attributable to this business's qualified income
        /// Payroll expenses remain in the expense records. This source only supplies the QBI wage limit; do not deduct the wages again.
        attribution: PayrollQbiAttribution,
    },
}

/// How employee payroll is attributable for the QBI wage limit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PayrollQbiAttribution {
    /// Attribution source records are unavailable
    // Printed wage boxes cannot reconstruct the source's qualified-wage allocation.
    // WARN_QBI_WAGES_LEFT_OUT omits only unavailable QBI capacity.
    /// Keep payroll expenses and employer credits; omit QBI wage capacity with a notice.
    Unavailable,
    /// Entire payroll satisfies the qualified unmodified-box conditions
    /// The filed Forms W-2 and W-3 establish timely reported wages entirely attributable to this business's qualified income. No unresolved cooperative, predecessor, shared-employer, foreign or special-year allocation applies.
    EntireBusiness,
    /// A source worksheet establishes a qualified part of these wages
    Allocated {
        /// Documented W-2 wages attributable to this business's QBI
        /// Use the source worksheet's wage quantity after total-wage and business-allocation steps, not a final QBI deduction or a percentage of receipts.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        qualified_wages: i64,
        /// Associated wage expenses and the source worksheet establish this attribution
        /// Check the filed Forms W-2/W-3, payroll allocation worksheet and employer records.
        records_establish_attribution: bool,
        /// SSA reporting satisfies the applicable 60-day and correction rules
        /// Check the filed Forms W-2/W-3, payroll allocation worksheet and employer records.
        timely_correctly_reported: bool,
        /// These are this business's common-law employees and no other person claims these wages
        /// Check the filed Forms W-2/W-3, payroll allocation worksheet and employer records.
        employer_entitled: bool,
        /// Ordinary domestic unmodified-box source with no unresolved employer, year or cooperative allocation
        /// Check the filed Forms W-2/W-3, payroll allocation worksheet and employer records.
        ordinary_domestic_no_special_case: bool,
    },
}
impl PayrollQbiAttribution {
    fn confirmed(&self) -> bool {
        match self {
            Self::Unavailable => false,
            Self::EntireBusiness => true,
            Self::Allocated {
                records_establish_attribution,
                timely_correctly_reported,
                employer_entitled,
                ordinary_domestic_no_special_case,
                ..
            } => [
                *records_establish_attribution,
                *timely_correctly_reported,
                *employer_entitled,
                *ordinary_domestic_no_special_case,
            ]
            .iter()
            .all(|v| *v),
        }
    }
}

/// Employee W-2 issued by the business
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read an employee Form W-2 issued by this business for the supplied tax year. Report a mismatched year, illegible or conflicting values as errors. Return the printed employee name and employer name in label, separated by \" / \", together with printed box 1 wages and box 5 Medicare wages and tips; a blank numeric box is zero. Never infer a missing name or use a taxpayer identification number as the label. Copies of the same employee form are one form. Do not return taxpayer identification numbers, infer QBI attribution or confirmations, or add payroll expenses.",
    doc_type = "application/pdf,image/*"
)]
pub struct PayrollW2 {
    /// Employee and employer names as printed
    /// Identifies this W-2 when an employer credit lists the employee, so each label is used once in this business's payroll.
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_size(min = 1)]
    pub label: String,
    /// Box 1 wages
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wages: i64,
    /// Box 5 Medicare wages and tips
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub medicare: i64,
}
impl BusinessPayroll {
    fn totals(statements: &[PayrollW2]) -> Option<(i64, i64)> {
        statements.iter().try_fold((0_i64, 0_i64), |(wages, medicare), v| {
            Some((wages.checked_add(v.wages)?, medicare.checked_add(v.medicare)?))
        })
    }
    pub(crate) fn qbi_wages(&self) -> i64 {
        match self {
            Self::No => 0,
            Self::Yes { attribution, .. } if !attribution.confirmed() => 0,
            Self::Yes {
                attribution: PayrollQbiAttribution::Allocated { qualified_wages, .. },
                ..
            } => *qualified_wages,
            Self::Yes { statements, .. } => Self::totals(statements)
                .map_or(0, |(wages, medicare)| phase::cap(wages, [medicare])),
        }
    }
    pub(crate) fn qbi_unavailable(&self) -> bool {
        matches!(self, Self::Yes { attribution, .. } if !attribution.confirmed())
    }
    // Projection checks only source shape: independently rounded copies are
    // not another originating payroll against which to cap the quantity.
    fn shape_issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let Self::Yes { statements, attribution } = self else {
            return None;
        };
        let issue = || ValidationIssue::new("QBI_WAGE_PART_SOURCE");
        if statements.is_empty() || statements.len() > 100 {
            return Some(issue().at_field("statements"));
        }
        for (index, row) in statements.iter().enumerate() {
            let field = if row.label.trim().is_empty()
                || statements[..index].iter().any(|prior| prior.label == row.label)
            {
                Some("label")
            } else if !(0..=100_000_000_000).contains(&row.wages) {
                Some("wages")
            } else if !(0..=100_000_000_000).contains(&row.medicare) {
                Some("medicare")
            } else {
                None
            };
            if let Some(field) = field {
                return Some(issue().at_field(field).in_list("statements", index));
            }
        }
        if Self::totals(statements).is_none() {
            return Some(issue().at_field("statements"));
        }
        if let PayrollQbiAttribution::Allocated { qualified_wages, .. } = attribution
            && !(0..=100_000_000_000).contains(qualified_wages)
        {
            return Some(issue().at_field("qualified_wages").in_field("attribution"));
        }
        None
    }
    pub(crate) fn projected_shape_valid(&self) -> bool {
        self.shape_issue().is_none()
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        if let Some(issue) = self.shape_issue() {
            return Some(issue);
        }
        if let Self::Yes {
            statements,
            attribution: PayrollQbiAttribution::Allocated { qualified_wages, .. },
        } = self
            && !Self::totals(statements).is_some_and(|(wages, medicare)| {
                *qualified_wages <= wages && *qualified_wages <= medicare
            })
        {
            return Some(
                crate::validation::ValidationIssue::new("QBI_WAGE_PART_SOURCE")
                    .at_field("qualified_wages")
                    .in_field("attribution"),
            );
        }
        None
    }
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
}

/// Merchandise and production inventory
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessInventory {
    /// No inventory or cost of goods sold
    None,
    /// Inventory valued by the selected method
    Cost {
        /// Method used to value closing inventory
        #[libertas_default(Cost)]
        method: InventoryMethod,
        /// Opening inventory at cost
        /// Use the prior closing inventory; enter zero for a new business with no opening stock. In the year you change your inventory method, use last year's closing inventory refigured under the new method; the difference belongs in that change's section 481(a) adjustment under Accounting-method changes.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        opening: i64,
        /// Purchases for resale or production, net of returns
        /// Include acquisition costs consistently with your accounting method. Do not enter these purchases again as operating expenses.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        purchases: i64,
        /// Cost of goods withdrawn for personal use
        /// Use cost, not retail selling price.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        personal: i64,
        /// Production labor costs
        /// Exclude your own labor and any costs already included in operating expenses.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        labor: i64,
        /// Materials and supplies used in production
        /// Exclude amounts already included in purchases or operating expenses.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        materials: i64,
        /// Other costs included in inventory
        /// Exclude costs already counted elsewhere.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        other: i64,
        /// Closing inventory valued by the selected method
        /// Use a physical inventory or reliable perpetual records with the same valuation method as opening stock.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        closing: i64,
        /// Consistent cost valuation and permitted accounting method confirmed
        /// Include inventory once. No LIFO, section 263A capitalization adjustment, tax-shelter treatment or unentered inventory adjustment; an inventory method change is entered under Accounting-method changes. Your chosen accounting method must permit the receipts and costs reported here. Your books and the prior Schedule C Part III method show it. No leaves the inventory out of the estimate with a notice, so no cost of goods sold is deducted; that situation is not supported.
        eligible: bool,
    },
}
impl BusinessInventory {
    pub(crate) fn cost(&self) -> i64 {
        match self {
            Self::None => 0,
            Self::Cost {
                opening, purchases, personal, labor, materials, other, closing, ..
            } => opening + purchases - personal + labor + materials + other - closing,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::Cost {
                opening, purchases, personal, labor, materials, other, closing, ..
            } => {
                [opening, purchases, personal, labor, materials, other, closing]
                    .iter()
                    .all(|v| (0..=100000000000).contains(*v))
                    && self.cost() >= 0
            }
        }
    }
    pub(crate) fn eligible(&self) -> bool {
        matches!(self, Self::None | Self::Cost { eligible: true, .. })
    }
}

impl IsoExercise {
    pub(crate) fn valid_timing(&self) -> bool {
        let recognized = match self.recognition {
            IsoRecognition::Exercise | IsoRecognition::Election => self.exercised,
            IsoRecognition::Vesting { date, .. } => date,
        };
        crate::federal_rules::valid_birth(self.granted)
            && crate::federal_rules::valid_birth(self.exercised)
            && crate::federal_rules::valid_birth(recognized)
            && self.granted <= self.exercised
            && self.exercised <= recognized
            && recognized / 10000 == 2026
    }
    // The value per share when the shares enter AMT income: box 4, or the
    // value on the vesting date for restricted shares without an election.
    pub(crate) fn recognition_price(&self) -> i64 {
        match self.recognition {
            IsoRecognition::Vesting { market_price, .. } => market_price,
            IsoRecognition::Exercise | IsoRecognition::Election => self.market_price,
        }
    }
    // Shares still held at year-end in millionths: box 5 less the shares
    // disposed of in 2026; None for an invalid quantity or more sold than
    // transferred.
    pub(crate) fn held_units(&self) -> Option<i64> {
        let transferred = crate::employee_stock::share_units(self.shares)?;
        let sold =
            if self.sold == 0.0 { 0 } else { crate::employee_stock::share_units(self.sold)? };
        (sold <= transferred).then_some(transferred - sold)
    }
}

// One-way: every printed box and answer either reaches the internal model or is
// named as not read by any calculation, so a new one cannot be forgotten.
impl TryFrom<&IsoExerciseSource> for IsoExercise {
    type Error = ();

    fn try_from(source: &IsoExerciseSource) -> Result<Self, Self::Error> {
        let IsoExerciseSource { id, assigned, owner_source, joint_share, owner, form, more } =
            source;
        let form = &Form3921::from(form);
        let Form3921 {
            transferor,
            employee,
            granted,
            exercised,
            exercise_price,
            market_price,
            shares,
            corporation,
        } = form;
        let Form3921More { sold, recognition, option_cost } = more.clone().ok_or(())?;
        Ok(IsoExercise {
            id: *id,
            printed: Some(crate::PrintedDocumentNames {
                recipient: employee.clone(),
                issuer: Some(transferor.clone()),
                transfer_corporation: Some(corporation.clone()),
                assigned: *assigned,
                owner_source: *owner_source,
                joint_share: *joint_share,
                ..Default::default()
            }),
            owner: *owner,
            label: transferor.name.clone(),
            exercised: *exercised,
            recognition,
            exercise_price: *exercise_price,
            market_price: *market_price,
            option_cost,
            shares: *shares,
            granted: *granted,
            corporation: crate::common::optional_text(&corporation.name),
            sold,
        })
    }
}

// The stored claim and the page's form of it carry the same answers.
impl TryFrom<ForeignTaxClaim> for ForeignTaxPage {
    type Error = ();
    fn try_from(claim: ForeignTaxClaim) -> Result<Self, Self::Error> {
        Ok(match claim {
            ForeignTaxClaim::Ordinary {
                basis,
                sources,
                situations,
                history,
                carryback,
                amt_limitation,
                interest_allocation,
                investment_assets,
                loss_history,
                state_taxes,
                rate_adjustment,
            } => Self::Ordinary {
                basis,
                sources: sources.into_iter().map(ForeignSourceRow::from).collect(),
                situations,
                history,
                carryback,
                amt_limitation,
                interest_allocation,
                investment_assets: investment_assets.into(),
                loss_history,
                state_taxes: state_taxes.into(),
                rate_adjustment,
            },
            ForeignTaxClaim::No => Self::No,
            ForeignTaxClaim::Deduction { taxes } => Self::Deduction {
                taxes: taxes.into_iter().map(ForeignTaxDeductedPage::from).collect(),
            },
            ForeignTaxClaim::Statements { eligible: Some(eligible) } => {
                Self::Statements { eligible }
            }
            ForeignTaxClaim::Statements { eligible: None } => return Err(()),
        })
    }
}
impl From<ForeignTaxPage> for ForeignTaxClaim {
    fn from(page: ForeignTaxPage) -> Self {
        match page {
            ForeignTaxPage::Ordinary {
                basis,
                sources,
                situations,
                history,
                carryback,
                amt_limitation,
                interest_allocation,
                investment_assets,
                loss_history,
                state_taxes,
                rate_adjustment,
            } => Self::Ordinary {
                basis,
                sources: sources.into_iter().map(ForeignOrdinaryTax::from).collect(),
                situations,
                history,
                carryback,
                amt_limitation,
                interest_allocation,
                investment_assets: investment_assets.into(),
                loss_history,
                state_taxes: state_taxes.into(),
                rate_adjustment,
            },
            ForeignTaxPage::No => Self::No,
            ForeignTaxPage::Deduction { taxes } => {
                Self::Deduction { taxes: taxes.into_iter().map(ForeignTaxDeducted::from).collect() }
            }
            ForeignTaxPage::Statements { eligible } => {
                Self::Statements { eligible: Some(eligible) }
            }
        }
    }
}
impl From<ForeignOrdinaryTax> for ForeignSourceRow {
    fn from(row: ForeignOrdinaryTax) -> Self {
        Self {
            label: row.label,
            country: row.country,
            owner: row.owner,
            category: match row.category {
                ForeignTaxCategory::Employment => ForeignSourceCategory::Employment,
                ForeignTaxCategory::PriorEmployeeWageTax { income_year } => {
                    ForeignSourceCategory::PriorEmployeeWageTax { income_year }
                }
                ForeignTaxCategory::PriorPassiveTax { income_year } => {
                    ForeignSourceCategory::PriorPassiveTax { income_year }
                }
                ForeignTaxCategory::Passive { withholding, qualified, sales } => {
                    ForeignSourceCategory::Passive { withholding, qualified, sales }
                }
                ForeignTaxCategory::Business { business, separate_books } => {
                    ForeignSourceCategory::Business { business, separate_books }
                }
            },
            income: row.income,
            tax: row.tax,
        }
    }
}
impl From<ForeignSourceRow> for ForeignOrdinaryTax {
    fn from(row: ForeignSourceRow) -> Self {
        Self {
            label: row.label,
            country: row.country,
            owner: row.owner,
            category: match row.category {
                ForeignSourceCategory::Employment => ForeignTaxCategory::Employment,
                ForeignSourceCategory::PriorEmployeeWageTax { income_year } => {
                    ForeignTaxCategory::PriorEmployeeWageTax { income_year }
                }
                ForeignSourceCategory::PriorPassiveTax { income_year } => {
                    ForeignTaxCategory::PriorPassiveTax { income_year }
                }
                ForeignSourceCategory::Passive { withholding, qualified, sales } => {
                    ForeignTaxCategory::Passive { withholding, qualified, sales }
                }
                ForeignSourceCategory::Business { business, separate_books } => {
                    ForeignTaxCategory::Business { business, separate_books }
                }
            },
            income: row.income,
            tax: row.tax,
        }
    }
}

impl RentalAtRisk {
    // Mechanical mapping for the unchanged farm editor.
    pub(crate) fn from_full_risk(all: bool) -> Self {
        if all {
            Self::All
        } else {
            Self::Limited {
                record: None,
                prior_disallowed: 0,
                amt_prior_disallowed: 0,
                prior_section1231_loss: 0,
                amt_prior_section1231_loss: 0,
            }
        }
    }
    pub(crate) fn prior(&self, amt: bool) -> i64 {
        match self {
            Self::All => 0,
            Self::Limited { prior_disallowed, amt_prior_disallowed, .. } => {
                if amt {
                    *amt_prior_disallowed
                } else {
                    *prior_disallowed
                }
            }
        }
    }
    pub(crate) fn prior_sale(&self, amt: bool) -> i64 {
        match self {
            Self::All => 0,
            Self::Limited { prior_section1231_loss, amt_prior_section1231_loss, .. } => {
                if amt {
                    *amt_prior_section1231_loss
                } else {
                    *prior_section1231_loss
                }
            }
        }
    }
    pub(crate) fn issue_for_method(
        &self,
        category: &crate::RiskEffectiveCategory,
        optional_method: bool,
    ) -> Option<crate::validation::ValidationIssue> {
        match self {
            Self::All => None,
            Self::Limited {
                record,
                prior_disallowed,
                amt_prior_disallowed,
                prior_section1231_loss,
                amt_prior_section1231_loss,
            } => {
                for (field, value) in [
                    ("prior_disallowed", prior_disallowed),
                    ("amt_prior_disallowed", amt_prior_disallowed),
                    ("prior_section1231_loss", prior_section1231_loss),
                    ("amt_prior_section1231_loss", amt_prior_section1231_loss),
                ] {
                    if !(0..=100_000_000_000).contains(value) {
                        return Some(
                            crate::validation::ValidationIssue::new("RISK_METHOD_INVALID")
                                .at_field(field),
                        );
                    }
                }
                let Some(record) = record else {
                    return (!optional_method).then(|| {
                        crate::validation::ValidationIssue::new(
                            "ACTIVITY_GROUP_INDIVIDUAL_RISK_REQUIRED",
                        )
                        .at_field("record")
                    });
                };
                if !record.valid(category) {
                    return Some(
                        crate::validation::ValidationIssue::new("RISK_METHOD_INVALID")
                            .at_field("record"),
                    );
                }
                for amt in [false, true] {
                    let Some(balance) = record.compute(category, amt) else {
                        return Some(
                            crate::validation::ValidationIssue::new(
                                "ACTIVITY_LOSS_AMOUNT_UNREPRESENTABLE",
                            )
                            .at_field("record"),
                        );
                    };
                    if balance.detailed.is_none()
                        && balance.simplified.is_some_and(|value| value < 0)
                    {
                        let issue = crate::validation::ValidationIssue::new(
                            "RISK_RECAPTURE_DETAILED_REQUIRED",
                        );
                        return Some(
                            if amt && matches!(record.amt, crate::RiskAmtMethod::Separate { .. }) {
                                issue.at_field("facts").in_field("amt").in_field("record")
                            } else {
                                issue.at_field("regular").in_field("record")
                            },
                        );
                    }
                }
                None
            }
        }
    }
}

/// Percentage depletion rate (section 613(b))
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RoyaltyMineralRate {
    /// 5% — gravel, peat, pumice, sand, scoria and certain stone and clay
    Five,
    /// 7.5% — clay or shale for sewer pipe and brick
    SevenAndHalf,
    /// 10% — coal, lignite, perlite and sodium chloride
    Ten,
    /// 14% — most metal mines and other minerals
    Fourteen,
    /// 15% — gold, silver, copper, iron ore, oil shale and geothermal
    Fifteen,
    /// 22% — sulphur and uranium
    TwentyTwo,
}
impl RoyaltyMineralRate {
    pub(crate) fn hundredths(self) -> i64 {
        match self {
            Self::Five => 500,
            Self::SevenAndHalf => 750,
            Self::Ten => 1000,
            Self::Fourteen => 1400,
            Self::Fifteen => 1500,
            Self::TwentyTwo => 2200,
        }
    }
}

/// Method used to value closing inventory (Schedule C line 33)
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum InventoryMethod {
    /// Cost
    /// Each item at its invoice cost plus the costs of getting it ready for sale (box 33a).
    Cost,
    /// Lower of cost or market
    /// Each item on hand at the lower of its cost and its market (replacement) value on the inventory date, item by item (box 33b). Enter the closing inventory valued that way.
    LowerOfCostOrMarket,
    /// Materials and supplies of a small business
    /// You had average annual gross receipts of $32 million or less for the three prior years for this business and are not a tax shelter, and you treat inventory as non-incidental materials and supplies: items are deducted when first used or consumed (or paid for, if later). Enter the items on hand and not yet used as opening and closing inventory (box 33c).
    MaterialsAndSupplies,
    /// Book method of a small business
    /// The same gross-receipts test, with inventory valued as in your applicable financial statement or, without one, your books and records (box 33c).
    BookConformity,
}

// Community-property allocation of this source record; facts retain their meaning.
impl crate::community::Divide for FederalRental {
    fn divided(&self, share: crate::community::Share) -> Self {
        // Exhaustive fields retain the same money/fact classification; only
        // this multi-vintage source needs one aggregate-preserving projection.
        let Self {
            improvement_buildings,
            interest_limit_election,
            location,
            energy_buildings,
            property_qbi_history,
            rent,
            expenses,
            buildings,
            assets,
            at_risk,
            history,
            method_changes,
            payroll,
            personal_use,
            home_operating_carryover,
            prior_home_carry,
            sale,
            statement,
            related,
            participation,
            ordinary_rental,
            qbi_business,
            qbi_aggregation_group,
            property_type,
            section179_business,
        } = self;
        Self {
            improvement_buildings: improvement_buildings.clone(),
            interest_limit_election: interest_limit_election.clone(),
            location: location.clone(),
            energy_buildings: energy_buildings.clone(),
            property_qbi_history: crate::rental_property_history::divided_history(
                property_qbi_history,
                share,
            ),
            rent: crate::community::Divide::divided(rent, share),
            expenses: crate::community::Divide::divided(expenses, share),
            buildings: crate::community::Divide::divided(buildings, share),
            assets: crate::community::Divide::divided(assets, share),
            at_risk: crate::community::Divide::divided(at_risk, share),
            history: crate::community::Divide::divided(history, share),
            method_changes: crate::community::Divide::divided(method_changes, share),
            payroll: crate::community::Divide::divided(payroll, share),
            personal_use: crate::community::Divide::divided(personal_use, share),
            home_operating_carryover: crate::community::Divide::divided(
                home_operating_carryover,
                share,
            ),
            prior_home_carry: prior_home_carry.as_ref().map(|carry| RentalHomeCarries {
                home_depreciation_carryover: crate::community::Divide::divided(
                    &carry.home_depreciation_carryover,
                    share,
                ),
                amt_home_depreciation_carryover: crate::community::Divide::divided(
                    &carry.amt_home_depreciation_carryover,
                    share,
                ),
                home_casualty_carryover: crate::community::Divide::divided(
                    &carry.home_casualty_carryover,
                    share,
                ),
                amt_home_casualty_carryover: crate::community::Divide::divided(
                    &carry.amt_home_casualty_carryover,
                    share,
                ),
            }),
            section179_business: *section179_business,
            sale: crate::community::Divide::divided(sale, share),
            statement: statement.as_ref().map(|paper| {
                let mut paper = paper.clone();
                paper.box_2a_total_gross_proceeds =
                    crate::community::Divide::divided(&paper.box_2a_total_gross_proceeds, share);
                paper
            }),
            related: related.clone(),
            participation: *participation,
            ordinary_rental: *ordinary_rental,
            qbi_business: qbi_business.clone(),
            qbi_aggregation_group: qbi_aggregation_group.clone(),
            property_type: property_type.clone(),
        }
    }
}

// Community-property allocation of this source record; facts retain their meaning.
crate::community::divide_record!(FederalFarmRental {
    amounts: [farm, expenses, vehicles, assets, at_risk, history, payroll],
    gains: [],
    facts: [
        improvement_buildings,
        interest_limit_election,
        active_participation,
        qbi_business,
        qbi_aggregation_group
    ],
});

// Community-property allocation of this source record; facts retain their meaning.
crate::community::divide_record!(RentalHistory {
    amounts: [regular, amt],
    gains: [],
    facts: [active_in_loss_year],
});

// Community-property allocation of this source record; facts retain their meaning.
crate::community::divide_record!(RentalExpense {
    amounts: [amount],
    gains: [],
    facts: [category, treatment],
});

// Community-property allocation of this source record; facts retain their meaning.
impl crate::community::Divide for RentalBuilding {
    fn divided(&self, share: crate::community::Share) -> Self {
        let Self {
            sale_history,
            cost,
            land,
            acquired,
            rehabilitation,
            partial,
            credit_reduction,
            casualty,
            casualty_property,
            energy,
            id,
            in_service,
            commercial_1993_transition,
        } = self;
        Self {
            sale_history: sale_history.divided(share),
            cost: cost.divided(share),
            land: land.divided(share),
            acquired: acquired.divided(share),
            rehabilitation: rehabilitation.divided(share),
            partial: partial.divided(share),
            credit_reduction: credit_reduction.divided(share),
            casualty: casualty.as_ref().map(|source| source.divided_at(share, *land)),
            casualty_property: *casualty_property,
            energy: energy.divided(share),
            id: *id,
            in_service: *in_service,
            commercial_1993_transition: commercial_1993_transition.clone(),
        }
    }
}

// Allocate amounts; legal character, dates and retained facts are unchanged.
crate::community::divide_enum!(RentalAtRisk {
    records: [Limited {
        amounts: [
            record,
            prior_disallowed,
            amt_prior_disallowed,
            prior_section1231_loss,
            amt_prior_section1231_loss
        ],
        facts: []
    }],
    units: [All],
    tuples: [],
});

// Allocate amounts; legal character, dates and retained facts are unchanged.
crate::community::divide_enum!(BusinessPayroll {
    records: [Yes { amounts: [statements, attribution], facts: [] },],
    units: [No],
    tuples: [],
});

// Project only the actual wage quantity; retain all source confirmations.
crate::community::divide_enum!(PayrollQbiAttribution {
    records: [Allocated {
        amounts: [qualified_wages],
        facts: [
            records_establish_attribution,
            timely_correctly_reported,
            employer_entitled,
            ordinary_domestic_no_special_case
        ]
    },],
    units: [Unavailable, EntireBusiness],
    tuples: [],
});

// Allocate amounts; legal character, dates and retained facts are unchanged.
crate::community::divide_enum!(RentalPersonalUse {
    records: [Yes {
        amounts: [direct_expenses],
        facts: [fair_rental_days, personal_days, overlap_days]
    },],
    units: [No],
    tuples: [],
});

// Allocate amounts; legal character, dates and retained facts are unchanged.
crate::community::divide_enum!(RentalAcquisition {
    records: [Converted { amounts: [fair_market_value], facts: [] },],
    units: [Purchased, Inherited, Gift],
    tuples: [],
});

// Community-property projection; accepted source facts are retained.

crate::community::divide_record!(PayrollW2 {
    amounts: [wages, medicare],
    gains: [],
    facts: [label],
});
// Mileage is a source quantity. Its computed monetary deduction is allocated
// by the farm-rental owner after applying the statutory rate.
crate::community::divide_record!(BusinessMileage {
    amounts: [parking_tolls, interest, property_tax],
    gains: [],
    facts: [
        id,
        placed,
        personal_use_available,
        other_vehicle,
        evidence,
        first_half,
        second_half,
        total,
        commuting,
        owned
    ],
});

/// Foreign tax credit or deduction choice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub enum ForeignTaxInitial {
    Ordinary {
        basis: ForeignTaxBasis,
        sources: Vec<ForeignSourceRow>,

        // Each of the nine foreign-tax situation codes occurs at most once.
        situations: Vec<u32>,
        history: crate::ForeignCreditHistory,
        carryback: crate::ForeignCarryback,
        amt_limitation: ForeignAmtLimitation,

        interest_allocation: ForeignInterestAllocation,

        investment_assets: crate::ForeignInvestmentAssetsPage,

        loss_history: crate::ForeignLossHistory,

        state_taxes: crate::ForeignStateTaxesPage,

        rate_adjustment: ForeignRateAdjustment,
    },
    No,
    Statements {
        /// [NullText]
        /// Eligibility has not yet been answered
        /// [SetText]
        /// The accepted eligibility answer is
        eligible: Option<bool>,
    },
    Deduction {
        taxes: Vec<ForeignTaxDeductedPage>,
    },
}

impl From<ForeignTaxPage> for ForeignTaxInitial {
    fn from(value: ForeignTaxPage) -> Self {
        match value {
            ForeignTaxPage::Ordinary {
                basis,
                sources,
                situations,
                history,
                carryback,
                amt_limitation,
                interest_allocation,
                investment_assets,
                loss_history,
                state_taxes,
                rate_adjustment,
            } => Self::Ordinary {
                basis,
                sources,
                situations,
                history,
                carryback,
                amt_limitation,
                interest_allocation,
                investment_assets,
                loss_history,
                state_taxes,
                rate_adjustment,
            },
            ForeignTaxPage::No => Self::No,
            ForeignTaxPage::Statements { eligible } => {
                Self::Statements { eligible: Some(eligible) }
            }
            ForeignTaxPage::Deduction { taxes } => Self::Deduction { taxes },
        }
    }
}
