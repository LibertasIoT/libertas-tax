// Coverage: Schedule E rental specials (B10-3, 2026-10-10).
// Short-stay participation and 39-year transient depreciation, nondepreciable
// and self-rental positive-income recharacterization, assets, converted basis,
// Form 6198 before passive limits, home-use worksheet routes and passive carry,
// professional/former-passive rentals and line 43. Direct royalty columns,
// legal mineral rates (including 7.5%), cost/percentage depletion, regular/AMT
// oil 65% limits and Form 6251 line 2d live in royalties.rs. K-1 additions wait
// for their owning unit. Sources: Pub. 527/925; IRC 168,465,469,611–613A;
// Reg. 1.469-2T(f)(3),1.469-2(f)(6),1.1411-4(g)(6).
// Tests: src/tests/activity/rental_specials.rs fixtures A–H and Schedule E footing.
// Historical implementation record (superseded scopes are dated below):
// Coverage: Schedule E (page 1) — Supported (audited 2026-09-25).
//   Lines: Residential rental: rent, ordinary expenses, 27.5-year SL mid-month depreciation,
//   active-participation $25,000/$12,500 allowance with phaseout, Form 8582 passive netting and
//   suspended-loss history (regular and AMT), at-risk confirmation, rental QBI flag, payroll;
//   nonbusiness royalties and personal-property rent (an unsupported one is left out with a
//   notice). Refused via ordinary_rental:
//   personal use, short-term stays, services, self-rental, RE professional, disposition,
//   conversion, gifts/inheritance, partnership, foreign property, furniture, ADS.
//   Refused explicitly (unit A14, from the Schedule E map): nonresidential property (type 4)
//   through the NonresidentialRental situation (M1-P3; the rental's nonresidential answer is
//   removed), until unit C19b adds the 39-year period.
//   Evidence: extended_model.rs FederalRental, extended_rules.rs rentals_for, federal_model.rs
//   NonbusinessPropertyKind, PersonalRentalHistory; tests:
//   rental_profit_flows_to_schedule_e_agi_niit_and_qbi_once,
//   passive_rental_allowance_matches_irs_example_and_phaseout_boundaries,
//   rental_allocations_reconcile_with_multiple_tiny_loss_properties (investment.rs),
//   nonbusiness_royalties_flow_through_schedule_e_and_niit_without_se_or_qbi (business.rs); Pub
//   527, Pub 925
//   Unit C17a (2026-09-25): FederalRental.personal_use (No | Yes { fair_rental_days, personal_days,
//   overlap_days (F7), direct_expenses, prior_operating, prior_depreciation }); the 14-day/10% home test; expenses
//   divided by rental over rental-plus-personal days (IRS method); Pub 527 Worksheet 5-1 order
//   (interest and taxes, then operating, then depreciation) limited to rental income, carryovers
//   as return lines rental_{id}_home_operating_carryover / _home_depreciation_carryover; under 15
//   rental days excludes income and expenses; a home rental is nonpassive and cannot show a loss;
//   line 2 above the rent and a home with a prior suspended passive loss are refused by name.
//   STORAGE_FORMAT 14.
//   Tests: src/tests/activity/investment.rs vacation_home_expense_allocation_and_limit_match_publication_527_example,
//   rental_under_15_days_is_excluded, rental_without_personal_use_is_unchanged.
//   Unit C17b (2026-09-25): FederalScreening.real_estate_professionals (per owner: qualifies Answer,
//   hours_confirmed) makes that owner's non-home rentals nonpassive (no Form 8582 allowance); Yes
//   without the confirmation and Not sure are refused naming Pub 925; a rental with suspended prior
//   passive losses under the election is refused by name. The 365-day rule is a client validation
//   rule (RENTAL_DAYS_ABOVE_365) with the structural save gate as the server backstop; the hours
//   confirmation is checked on save (REAL_ESTATE_PROFESSIONAL_NEEDS_HOURS). STORAGE_FORMAT 15.
//   Tests: src/tests/activity/investment.rs real_estate_professional_rental_loss_is_nonpassive,
//   real_estate_professional_needs_the_hours_confirmation, rental_days_cannot_exceed_365.
//   Unit C17c (2026-09-25): FederalRental.expenses is Vec<RentalExpense> with RentalExpenseCategory
//   (lines 5-12, 13a vehicle loan, 13b other interest, 14-17, 19 with a label; 18 stays computed from
//   the building); one return line per category and property (rental_{id}_expense_{name}); the
//   home-use worksheet reads the MortgageInterest and TaxesAndLicenses categories directly. No total
//   changed. STORAGE_FORMAT 16. Test: src/tests/investment/investment.rs
//   schedule_e_expense_lines_are_reported_by_category_and_property.
//   C-fixture (2026-09-25, src/tests/activity/investment.rs): residential_rental_return_matches_hand_computation (lines
//   3-26 with the Form 8582 allowance inside the phase-out, by hand), passive_allowance_is_zero_at_150000_agi,
//   rental_profit_is_reported_without_form_8582, building_placed_in_service_in_october_uses_mid_month.
//   Unit C19b (2026-09-25): FederalRental.property_type (Schedule E line 1b types 1-8, default 1); types
//   1-2 depreciate over 27.5 years (19i), type 4 over 39 years from Table A-7a (19j); types 3, 5-8 are
//   refused by name; the UBIA recovery-period test follows the type; A14's NonresidentialRental now
//   refuses only the special commercial situations. Tests: src/tests/investment/investment.rs
//   commercial_rental_is_no_longer_refused_and_residential_is_unchanged, part_iii_lines_19i_and_19j_are_emitted.
//   Unit M3-P6 (2026-09-25): the rental page (interview.rs) maps onto FederalRental; the
//   NonresidentialRental situation and the REAL_ESTATE_PROFESSIONAL_NEEDS_HOURS page rule are gone:
//   choosing the RealEstateProfessional situation is the Pub 925 confirmation (qualifies Yes,
//   hours_confirmed), offered only with a rental and dropped with the last one.
//   Unit F7 (2026-10-10, line audit E/8582): RentalPersonalUse::Yes gains overlap_days, the rented days
//   that were also personal-use days: rental days for the expense division (fair_rental_days counts
//   them, personal_days does not), personal-use days and not fair-rental days for the section
//   280A(d)(1) home test; a carryover is limited in a year that is not a home too (section
//   280A(c)(5), last sentence; the unused part still carries); K-1 (1041) box 5 royalties are
//   Schedule E line 33 column (f) (estate_trust_{id}_line33f_royalties), not line 26; line 42 is
//   printed whenever any source feeds it. Tests: src/tests/cross/line_audit_e_8582.rs.
//   Unit B2 (2026-10-09): Schedule E page 3 (2026 draft) Part V: line 40 net farm rental income or
//   loss from Form 4835 (schedule_e_farm_rental_total) and line 42 reconciliation of farming and
//   fishing income (schedule_e_farming_fishing_gross: Form 4835 line 7 and the K-1 farming
//   amounts), both only with a farm rental; line 41 is Schedule 1 line 5 as before; line 43
//   (real estate professional) is not reached because a farm rental stays passive. Part I lines
//   3 to 26 never include a farm rental. Stored: FederalFarmRental (below FederalRental). Tests:
//   src/tests/farm/farm_rental.rs.
//   Unit 481a (2026-10-09): a section 481(a) adjustment of Form 3115 (FederalRental.method_changes,
//   method_change.rs) is added to line 3 (the row rental_<id>_line3_rents, and line 20 and the passive
//   net) when positive and to line 19 other (rental_<id>_expense_line19_other) when negative; a rental
//   sold in 2026 takes the rest of a multi-year increase; a rental with personal use leaves the record
//   out with WARN_METHOD_CHANGE_LEFT_OUT. Tests: src/tests/cost_recovery/method_change.rs.
// Coverage: Form 8829 — canonical home/owner/period sources live in home/sites.rs;
//   home_office.rs supplies the bound deduction to each business.
// Coverage: Form 3115 — see method_change.rs (unit 481a, 2026-10-09).

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
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Land portion of that cost
    /// Land is not depreciated. Enter zero for a building improvement.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub credit_reduction: i64,
}

/// Rental property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalRental {
    /// Actual physical buildings for section 179D
    /// ----
    /// Physical building
    pub energy_buildings: Vec<crate::RentalEnergyBuildingFacts>,
    /// Original-year QBI tracking for prior at-risk property-sale losses
    /// One row per original disallowance year for this rental's separate Form 4797 loss.
    /// Do not include operating losses or losses still suspended by the passive rules.
    /// Empty with a prior at-risk property loss means its QBI history is unavailable.
    pub property_qbi_history: Vec<crate::RentalPropertyLossYear>,
    /// Canonical saved mortgage loans for this return's whole-property ownership share
    /// Enter interest, points and mortgage insurance once on Mortgage, save the loan first, then select it here. A loan may belong to one whole-property rental; split-property loans are not supported.
    pub mortgage_loans: Vec<i64>,
    /// Your share of rent received, including advance rent and retained deposits
    /// Include tenant reimbursements of expenses, lease cancellation payments and other amounts received as rent.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub rent: i64,
    /// Your share of ordinary rental expenses
    /// For property without personal use, include rental mortgage interest, property tax, insurance, repairs, management and utilities. For personal-use property, select saved Mortgage loans: their interest, points and mortgage insurance are derived once from those records. Exclude land, improvements, loan principal and depreciation; do not deduct these expenses again elsewhere.
    /// ----
    /// Rental expense
    pub expenses: Vec<RentalExpense>,
    /// Purchased building and structural improvements
    /// Depreciation is calculated from cost, land allocation, service dates and the property type below, using straight-line GDS with the mid-month convention: 27.5 years for single-family or multi-family property. Ordinary nonresidential buildings first available from 1987 through May 12, 1993 retain their 31.5-year schedule; later ordinary commercial buildings use 39 years. Keep the original service date when a schedule has ended.
    /// ----
    /// Building or improvement
    #[libertas_size(min = 1)]
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
    /// U.S. rental without the special treatments listed below
    /// No extraordinary personal services or incidental rentals, pre-February 19, 1988 binding self-rental contracts, foreign property, alternative depreciation election, property received in a like-kind exchange before 2026, working interests, or disposition other than an entered sale, exchange or event-date casualty retirement. Your own rental, acquisition and use records show it. No leaves this rental out of the estimate with a notice; that situation is not supported.
    pub ordinary_rental: bool,
    /// Your investment at risk
    /// Cash investment, recourse borrowing and qualifying nonrecourse real-estate financing can qualify. Choose Limited for Form 6198 amounts and prior disallowed losses; the at-risk limit applies before the passive-loss rules.
    /// [NullText]
    /// Not answered
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub home_operating_carryover: i64,
    /// Regular-tax casualty and depreciation carried over from last year's home-use worksheet
    /// Enter the combined third-tier carry from the actual prior worksheet, including a documented zero.
    #[libertas_default(Unavailable)]
    pub home_depreciation_carryover: crate::SourceCents,
    /// AMT casualty and depreciation carried over from last year's home-use worksheet
    /// Use the independently tracked AMT worksheet, even when it matches regular tax.
    #[libertas_default(Unavailable)]
    pub amt_home_depreciation_carryover: crate::SourceCents,
    /// Casualty part of the regular-tax combined home-use carry
    /// The actual prior worksheet's casualty amount; it cannot exceed its combined carry.
    #[libertas_default(Unavailable)]
    pub home_casualty_carryover: crate::SourceCents,
    /// Casualty part of the AMT combined home-use carry
    /// Use the actual AMT casualty amount, not the regular-tax amount.
    #[libertas_default(Unavailable)]
    pub amt_home_casualty_carryover: crate::SourceCents,
    /// Schedule E line 1b property type
    /// The property before any sale or exchange; replacement property has its own use. Residential buildings use 27.5 years, commercial and transient buildings 39 years. Land is not depreciated. Royalties have their own record.
    #[libertas_default(SingleFamily)]
    pub property_type: RentalPropertyType,
    /// Sold, exchanged or kept in 2026
    #[libertas_default(No)]
    pub sale: crate::RentalSale,
    /// Form 1099-S for the sale or exchange
    #[libertas_default($null)]
    pub statement: Option<crate::Rental1099S>,
}

/// Farm rental paid in crop or livestock shares (Form 4835)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalFarmRental {
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
    /// Not answered
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

/// Schedule E line 1b type of property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalPropertyType {
    /// Single-family residence
    SingleFamily,
    /// Multi-family residence
    /// Residential when 80% or more of the building's gross rents are from dwelling units (section 168(e)(2)(A)); otherwise choose Commercial. Units in an establishment where more than half are used on a transient basis are not dwelling units.
    MultiFamily,
    /// Vacation or short-term rental
    /// A dwelling unit rented for short stays. Personal use is entered below; the stays decide the passive-activity treatment.
    VacationShortTerm {
        /// Average stay and services
        stays: ShortTermStays,
    },
    /// Commercial
    Commercial,
    /// Land
    /// Enter the land cost as the building's cost with the same amount as its land part; nothing is depreciated. Net income from a rental whose depreciable part is under 30% of its unadjusted basis is not passive and cannot absorb other passive losses; a loss stays passive.
    Land,
    /// Self-rental
    /// Property you rent to a trade or business in which you materially participated, such as your own Schedule C business. Net income is not passive and not net investment income; a loss stays passive.
    SelfRental,
    /// Other
    Other {
        /// Describe the property
        #[libertas_size(min = 1)]
        description: String,
    },
}

/// Average stay and services of a short-term rental
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ShortTermStays {
    /// Average stay of more than 7 days, or more than 30 days when you provided significant personal services
    /// Divide the days of all rental periods in 2026 by the number of rentals. A rental activity: your participation answer under More about this property applies.
    OverSevenDays,
    /// Average stay of 7 days or less
    /// Not a rental activity for the passive-activity rules but a business activity: no special rental-loss allowance, and the participation answer under More about this property does not apply.
    SevenDaysOrLess {
        /// You materially participated
        /// More than 500 hours; or substantially all the work; or more than 100 hours and no one else more; or another section 469 test (Pub. 925). Count your spouse's work. Yes makes the income or loss nonpassive; No makes it a passive business activity whose loss offsets only passive income.
        material_participation: bool,
    },
    /// Average stay of 30 days or less with significant personal services
    /// Services by people, such as daily cleaning or concierge service, that are significant next to the rent but are not hotel-like services for the guests' convenience. A business activity, like the choice above.
    ThirtyDaysOrLessWithServices {
        /// You materially participated
        /// More than 500 hours; or substantially all the work; or more than 100 hours and no one else more; or another section 469 test (Pub. 925). Count your spouse's work. Yes makes the income or loss nonpassive; No makes it a passive business activity whose loss offsets only passive income.
        material_participation: bool,
    },
    /// Substantial services primarily for the guests' convenience
    /// Regular cleaning during stays, linen changes, meals, maid or concierge service, as a hotel does. The income is business income on Schedule C with self-employment tax: enter the property as a business on the Self-employment topic, with the building as a 39-year asset, and remove it here.
    SubstantialServices,
}

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
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        prior_disallowed: i64,
        /// Prior ordinary loss for alternative minimum tax
        /// The separately tracked AMT amount; zero means none.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        amt_prior_disallowed: i64,
        /// Prior Form 4797 section 1231 loss disallowed by the at-risk rules
        /// Keep this loss separate from operating expenses; it retains its property-sale character even without a sale this year.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        prior_section1231_loss: i64,
        /// Prior section 1231 loss for alternative minimum tax
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        amt_prior_section1231_loss: i64,
    },
}

/// Royalty property (Schedule E line 1b type 6)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalRoyalty {
    /// Kind of property
    pub kind: RoyaltyKind,
    /// Your share of royalties received, not on a Form 1099-MISC you entered
    /// The App adds box 2 of each 1099-MISC that names this entry. Enter the gross amount before any severance tax the producer withheld; enter that tax as a line 16 expense.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub receipts: i64,
    /// Lease bonuses and advance royalties included in the royalties
    /// Amounts payable without regard to production, here or on the 1099-MISC. They are income. Oil and gas percentage depletion excludes them (section 613A(d)(5)); another mineral's percentage depletion includes them.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        carryover: i64,
        /// The same carryover for the alternative minimum tax
        /// Enter the separately tracked amount, even if it equals the regular-tax carryover.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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

/// Personal use of a rental property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalPersonalUse {
    /// No personal use this year
    No,
    /// Personal use this year
    Yes {
        /// Days rented at a fair rental price
        /// Count each day rented to others at a fair price, even if you or your family also used it that day. Do not count days available but not rented.
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 365)]
        fair_rental_days: u32,
        /// Days of personal use that were not rented
        /// Days used by you, another owner or a family member of either, anyone under a home-swap arrangement, or anyone paying less than a fair price, other than a day you also counted as rented at a fair price above. Do not count days spent working substantially full time on repairs and maintenance.
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 365)]
        personal_days: u32,
        /// Rented days that were also personal-use days
        /// Of the fair rental days above, the days you or someone in the groups listed for personal use also used the property. They divide the expenses as rental days, but they are personal-use days, not fair-rental days, when testing whether you used the property as a home (section 280A(d)(1)).
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 365)]
        overlap_days: u32,
        /// Expenses of renting only, not divided by days
        /// For example, advertising, rental agency fees and office supplies. Enter shared property taxes, property insurance, repairs and utilities as rental expenses above; the App divides them by days. Enter mortgage interest, points and mortgage insurance once on Mortgage and select those loans on the rental.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    pub wages: i64,
}

/// Qualifying foreign period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignQualification {
    /// Only pay for qualifying 2025 services received in 2026
    /// You have no qualifying 2026 foreign period. Enter the retained 2025 pay and return records below. No current-year foreign income or housing is excluded.
    PriorYearOnly,
    /// Bona fide resident of a foreign country
    /// For an uninterrupted period that includes a whole calendar year.
    Residence {
        /// Date your bona fide residence began (line 10)
        #[libertas_date_only]
        began: u32,
        /// Date it ended
        /// Leave empty if it continues.
        /// [NullText]
        /// Foreign residence continues
        /// [SetText]
        /// Foreign residence ended on
        #[libertas_date_only]
        ended: Option<u32>,
        /// Basis for your bona fide residence
        basis: ResidenceBasis,
        /// You told the foreign authorities you were not a resident and they did not tax you as one
        /// Yes means this residence test is not met; use the physical presence test if it fits.
        nonresident_statement: bool,
    },
    /// Physically present abroad 330 full days in 12 months
    Presence {
        /// First day of the 12-month period (line 16)
        /// Choose 12 consecutive months that include part of 2026 and in which you were abroad at least 330 full days; the period ends the day before the same date a year later.
        #[libertas_date_only]
        starts_on: u32,
        /// Full 24-hour days physically in foreign countries within that period
        /// Count from your travel record (line 18). A full day starts at midnight; exclude days in the United States or its territories and travel over international waters. The test needs at least 330.
        // Tax structure: the value counts months or days in the stated calendar period.
        #[libertas_number(min = 0, max = 366)]
        foreign_days: i32,
    },
}

/// Basis of bona fide foreign residence
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ResidenceBasis {
    /// U.S. citizen
    Citizen,
    /// Resident alien who is a citizen or national of a country with a U.S. income tax treaty
    TreatyNational,
}

/// Foreign-credit interest expense allocation
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ForeignInterestAllocation {
    /// Apportion home mortgage interest by gross income
    GrossIncome,
    /// Allocate interest to U.S. income when foreign gross income is at most $5,000
    /// Optional small-foreign-income treatment. The App checks combined foreign gross income across categories; this is not a separate $5,000 allowance per source.
    SmallForeignIncome,
}

/// Foreign tax credit or deduction choice
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignTaxClaim {
    /// Credit for ordinary foreign income using Form 1116
    Ordinary {
        /// How you claim foreign taxes — Form 1116 Part II, Paid or Accrued
        basis: ForeignTaxBasis,
        /// Foreign income and tax by country
        /// One row for each country and category of foreign income already included once in Income, with the foreign tax on it. Employee pay, interest, dividends and royalties are entered with their amounts; a business run abroad and foreign-source investment sales are chosen from their own entries. A K-1's Schedule K-3 is read from its K-1 page; do not enter it here. The App allocates your deductions: the standard or itemized deductions by gross income, home mortgage interest by the Form 1116 worksheet, investment interest by your portfolio records or the $5,000 small-foreign-income choice, state income taxes by their source records; the expenses of your U.S. businesses, rentals, farms and gambling stay with U.S. income.
        /// ----
        /// Country and income category
        sources: Vec<ForeignOrdinaryTax>,
        /// Foreign tax situations that apply
        /// Check each that applies to you, or to your spouse on a joint return; leave all unchecked when none does. This estimate does not figure them, so a checked situation leaves the foreign tax credit out, with a note. A contested tax is not a situation: leave it out of the tax amounts until it is settled.
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
        investment_assets: crate::ForeignInvestmentAssets,
        /// Prior loss accounts and current recapture choices
        #[libertas_default(None)]
        loss_history: crate::ForeignLossHistory,
        /// Source records for itemized state/local income taxes
        #[libertas_default(Unreviewed)]
        state_taxes: crate::ForeignStateTaxes,
        /// Qualified dividend and capital gain adjustment
        #[libertas_default(Exception)]
        rate_adjustment: ForeignRateAdjustment,
    },
    /// No foreign tax credit or deduction
    No,
    /// Small foreign tax credit from payee statements
    Statements {
        // None is only the server-discovered pending election. The page asks a
        // required bool and its accepted save always supplies Some(answer).
        /// All foreign gross income is passive income reported on qualified payee statements
        /// Includes all foreign income and taxes for both spouses on a joint return. This election is limited to $300 ($600 jointly), with no carryover to or from this election year. No refundable, contested, non-income or otherwise ineligible foreign taxes. Check every foreign income statement and your foreign-tax records. No claims no foreign tax credit, with a review note.
        /// [NullText]
        /// Eligibility has not yet been answered
        /// [SetText]
        /// The accepted eligibility answer is
        eligible: Option<bool>,
    },
    /// Deduct foreign income taxes on Schedule A line 6 instead
    /// No foreign tax credit is claimed this year. Foreign income taxes are outside the state and local tax limit and help only when you itemize. Retained tax on your Forms 1099-INT, 1099-DIV, qualified estate or trust payee statements and Schedules K-3 is included already, subject to the existing source and K-1 basis limits. The credit usually saves more; choose it yourself if you prefer it.
    Deduction {
        /// Other foreign income taxes paid in 2026
        /// Add only income taxes not already included from a Form 1099-INT, 1099-DIV, qualified estate or trust payee statement or Schedule K-3. Exclude taxes already deducted by a business, taxes credited in an earlier year, contested or refundable amounts, U.S. territory taxes (Taxes you paid), foreign real estate taxes, and sales or value-added taxes.
        /// ----
        /// Foreign tax
        #[libertas_unordered]
        taxes: Vec<ForeignTaxDeducted>,
    },
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
        investment_assets: crate::ForeignInvestmentAssets,
        /// Prior loss accounts and current recapture choices
        #[libertas_default(None)]
        loss_history: crate::ForeignLossHistory,
        /// Source records for itemized state/local income taxes
        #[libertas_default(Unreviewed)]
        state_taxes: crate::ForeignStateTaxes,
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
        /// ----
        /// Foreign tax
        #[libertas_unordered]
        taxes: Vec<ForeignTaxDeductedPage>,
    },
}
/// Foreign tax
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignTaxDeducted {
    /// Payer and country
    #[libertas_size(min = 1)]
    pub label: String,
    /// Income it was paid on
    pub income: ForeignTaxIncome,
    /// Tax paid in 2026, in U.S. dollars
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
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
pub enum ForeignTaxIncome {
    /// Current-year pay for work or self-employment abroad
    /// The part on income excluded on Form 2555 is not deductible; the App takes it out for this person.
    Earned {
        /// Person who earned the income
        #[libertas_default(Taxpayer)]
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
    #[libertas_number(min = 1)]
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

// The payee-statement election has no Form 1116 fractional limitation or
// carryover. Eligibility remains in extended_rules::foreign_credit; reuse this
// same liability cap when Form 1062 refigures the credit.
impl ForeignTaxClaim {
    // `documents` is the payee-statement tax the return itself shows, in
    // cents (extended_rules::payee_statement_taxes): the 1099 boxes and the
    // Schedule K-3 passive taxes. They are the election's only source.
    pub(crate) fn statement_credit(&self, regular: i64, documents: i64) -> Option<i64> {
        match self {
            Self::No | Self::Deduction { .. } => Some(0),
            Self::Statements { .. } => Some(phase::cap(
                crate::kernel::ratio::positive(regular),
                [crate::kernel::ratio::dollars(documents)],
            )),
            Self::Ordinary { .. } => None,
        }
    }
}

/// How you claim foreign taxes
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ForeignTaxBasis {
    /// Paid
    /// Taxes count in the year you pay them. This is your choice unless you use the accrual method or chose accrued taxes on an earlier return.
    Paid,
    /// Accrued
    /// Taxes count in the year they relate to, paid or not. Once chosen you keep this choice in every later year, and you can first choose it only on a return filed on time. Tax on earlier years' income paid now belongs to those years' returns.
    Accrued,
}

/// Qualified dividend and capital gain adjustment
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum ForeignRateAdjustment {
    /// Use the adjustment exception when you qualify
    /// Foreign qualified dividends stay unadjusted when your ordinary taxable income stays within the 24% bracket and your foreign qualified dividends total less than $20,000. Otherwise the App adjusts them as required.
    Exception,
    /// Always adjust qualified dividends and capital gain distributions
    /// Applies the rate adjustments even when you qualify for the exception. This can give a larger credit when little of your foreign income is qualified dividends.
    Adjust,
}

/// AMT foreign-credit limitation method
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignAmtLimitation {
    /// Normal AMT income and deductions
    /// No simplified limitation election is in force. An earlier election can be revoked only with IRS consent.
    Normal,
    /// Apply the simplified limitation election
    /// Uses regular-tax net foreign income for the AMT limit. This can increase AMT or unused credit. The election continues in later years and cannot be changed annually to choose the lower tax.
    Simplified {
        /// First year after 1997 for which you claimed an AMT foreign tax credit
        /// Enter 2026 only if this is your first such claim and you are making the election now. Otherwise use the first claim year from your retained returns.
        // Tax structure: this date or year belongs to the stated current or retained-return period.
        #[libertas_number(min = 1998, max = 2026)]
        first_claim_year: u32,
        /// The election was made in that first claim year and remains in force
        /// For a first claim in 2026, confirm that you elect this method. An election omitted in the first eligible year cannot be made later; no IRS-approved revocation or unresolved filing-unit change. The first claim year's Form 1116 (AMT) shows the election. No uses the normal AMT limitation instead.
        confirmed: bool,
    },
}
impl ForeignAmtLimitation {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::Normal => true,
            Self::Simplified { first_claim_year, .. } => (1998..=2026).contains(first_claim_year),
        }
    }
}

/// When the ISO shares enter AMT income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum IsoRecognition {
    /// On exercise
    /// Rights were transferable or no longer subject to substantial risk of forfeiture on exercise.
    Exercise,
    /// When restrictions ended, without an earlier election
    Vesting {
        /// First date rights became transferable or no longer subject to substantial risk of forfeiture
        /// Use the first qualifying event, not a later scheduled vesting date. No earlier AMT inclusion applies.
        #[libertas_date_only]
        date: u32,
        /// Fair market value per share on that date
        /// Ignore lapse restrictions in valuing the stock. This replaces the exercise-date value for these shares.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        market_price: i64,
    },
    /// Valid timely AMT election on transfer
    /// Restricted shares were included under a valid election made by the applicable 30-day deadline. Transfer occurred on exercise; the election remains effective. This input records an election already made, not a new election filing.
    Election,
}

/// Incentive stock option exercise
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct IsoExercise {
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
    #[libertas_number(min = 0)]
    pub exercise_price: i64,
    /// Form 3921 box 4 fair market value per share on the exercise date
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub market_price: i64,
    /// Amount paid for the option itself
    /// Total option purchase cost for all the shares of this exercise, separate from the exercise price and commissions. Enter zero if nothing was paid for the option. Exclude interest.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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

/// Form 3921
/// Exercise of an incentive stock option. One for each Form 3921 you received.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read the Form 3921 exactly. Check its tax year against the supplied year; report a mismatch. Return the transferor's (employer's) name as printed. Boxes 1 through 5 must show values; report a blank one as an error. Return box 1 and box 2 dates, box 3 and box 4 per-share amounts, and the box 5 number of shares with up to six decimals. Return box 6's corporation name, or an empty string when box 6 is blank. Report illegible or conflicting values as errors. Do not infer sales of these shares, when they enter AMT income, the amount paid for the option, or other facts absent from the Form 3921. The file may hold several copies of this form, such as Copy B and Copy C; copies of the same form are one form.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct Form3921 {
    /// Employer (transferor)
    #[libertas_llm_doc_field]
    #[libertas_size(min = 1)]
    pub transferor: String,
    /// Box 1 date option granted
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub granted: u32,
    /// Box 2 date option exercised
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_date_only]
    pub exercised: u32,
    /// Box 3 exercise price per share
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exercise_price: i64,
    /// Box 4 fair market value per share on exercise date
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub market_price: i64,
    /// Box 5 number of shares transferred
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_number(min = 0.000001, step = 0.000001)]
    pub shares: f64,
    /// Box 6 corporation whose stock was transferred
    /// Only when it is not the employer named above.
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_default("")]
    pub corporation: String,
    /// About these shares
    pub more: Form3921More,
}
// Box 5 is a validated finite quantity (share_units), so equality is
// reflexive; the income document union needs Eq.
impl Eq for Form3921 {}

/// About these shares
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form3921More {
    /// Shares from this exercise sold or otherwise disposed of in 2026
    /// Report the sale itself with its 1099-B. Shares sold in the year you exercised have no AMT adjustment.
    #[libertas_number(min = 0, step = 0.000001)]
    #[libertas_default(0)]
    pub sold: f64,
    /// When the shares enter AMT income
    #[libertas_default(Exercise)]
    pub recognition: IsoRecognition,
    /// Amount paid for the option itself
    /// Separate from the exercise price. Enter zero if you paid nothing for the option.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub option_cost: i64,
}
impl Default for Form3921More {
    fn default() -> Self {
        Self {
            sold: Default::default(),
            recognition: IsoRecognition::Exercise,
            option_cost: Default::default(),
        }
    }
}
impl Eq for Form3921More {}

/// Rollover destination and completion
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum RolloverDestination {
    /// Traditional IRA; deposited by December 31, 2026
    /// The receiving account's year-end balance belongs in the combined IRA balance.
    Ira,
    /// Traditional IRA; deposited in 2027 within the eligible rollover period
    /// The rollover must already be completed, not merely intended. An outgoing IRA amount is added to the 2026 basis worksheet's year-end balance automatically; do not add it there yourself. An employer-plan rollover is not an outstanding IRA rollover: its after-tax basis is reported separately for 2027 and does not shelter 2026 IRA withdrawals.
    IraNextYear,
    /// Eligible employer retirement plan
    /// The plan must accept this rollover. Only pretax IRA money can enter an employer plan; after-tax basis must remain outside it. This destination never adds an outstanding amount to the IRA basis worksheet.
    EmployerPlan,
    /// Direct employer-plan rollover with separate after-tax accounting
    DirectEmployerPlan {
        /// Direct plan-to-plan transfer to a qualified employee plan or 403(b), with separate taxable and nontaxable accounting confirmed
        /// The receiving plan accepted the rollover and keeps its after-tax contribution record separately. This choice is for an employer-plan source, not an IRA, a 60-day deposit paid through you, a governmental 457 plan or a Roth destination. Form 1099-R code G and the receiving plan's acceptance letter show it. No still counts the transfer the same way for 2026, because the after-tax part is untaxed either way, and adds a note: ask the receiving plan to account for it, or it may be taxed again when paid out.
        confirmed: bool,
    },
    /// Roth IRA or designated Roth account
    /// From an employer plan only; an IRA uses the conversion. Pretax money rolled into a Roth account is taxed and is not subject to the 10% additional tax; a designated Roth account distribution rolled over within 60 days is taxed only on the earnings not rolled over.
    Roth,
    /// Another SIMPLE IRA
    /// The only tax-free rollover from a SIMPLE IRA in its first two years (section 408(d)(3)(G)).
    SimpleIra,
}

/// Retirement distribution treatment
/// Use the payer's Form 1099-R and your distribution facts. Use the owner-level worksheet for IRA basis and Adjustments for QCDs. Use the dedicated inherited IRA entry for inherited basis allocation and Adjustments for required-distribution shortfalls. Unlisted exceptions require review.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RetirementTreatment {
    /// Ordinary distribution, no additional tax
    Ordinary,
    /// Early distribution, 10% additional tax on the taxable amount, no exception
    Early,
    /// SIMPLE IRA within the first two years, 25% additional tax, no exception
    EarlySimple,
    /// Eligible rollover to a traditional IRA or retirement plan
    /// No Roth conversion of an IRA, required minimum distribution, inherited account, hardship withdrawal, deadline waiver, or SIMPLE IRA in its first two years. A qualified plan loan offset (code M) can be rolled over until the due date of the return, including extensions, and a rollover to a Roth IRA or designated Roth account is taxed on its pretax part. IRA-to-IRA indirect rollovers must satisfy the one-per-12-month rule. Employer plans may use calculated distribution basis. After-tax amounts may enter a traditional IRA by year-end when its basis record is provided; direct transfers to a qualifying employer plan require separate taxable/nontaxable accounting. After-tax IRA deposits completed in 2027 are reported as separate next-year basis, outside the 2026 IRA worksheet. IRA source basis uses the owner-level worksheet.
    Rollover {
        /// Receiving account and completion year
        destination: RolloverDestination,
        /// Amount actually deposited into the receiving retirement account
        /// Include any withheld amount you replaced with other funds. Do not enter an intended future deposit.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
        /// Any taxable amount kept is an early distribution subject to 10% additional tax
        early: bool,
    },
    /// Traditional IRA converted to Roth IRA
    /// Direct trustee conversion or completed eligible 60-day conversion; no RMD converted, inherited IRA, SIMPLE IRA in its first two years, employer-plan conversion or repayment. Enter any subsequent ordinary Roth withdrawal in its own Income entry with complete Roth history. With after-tax basis, use Calculate traditional IRA basis allocation for all this owner's traditional IRA distributions.
    Conversion {
        /// Amount actually deposited into the Roth IRA by December 31, 2026
        /// Include withheld money only if replaced with other funds. Any amount kept remains a withdrawal. Enter regular annual contributions separately in Adjustments.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
        /// Any amount kept is an early withdrawal with no exception
        early: bool,
    },
    /// Early distribution with a source-supported exception
    /// Use the payer-determined taxable amount or a supported basis calculation. Nonqualified annuities currently support death, permanent-disability and immediate-annuity exceptions. No Roth recapture; qualified reservist and supported special traditional IRA repayments use the owner's IRA basis record. Repayments of birth or adoption, emergency, domestic-abuse and terminal-illness IRA distributions are entered with the exception. Required distributions were met or shortfalls are entered separately. Beneficiary distributions must remain inherited rather than treated as the surviving spouse's own IRA.
    Exception {
        /// Date of this distribution in 2026
        /// Use separate entries for distinct distribution dates or treatments, splitting the payer totals only from actual account records.
        #[libertas_date_only]
        distributed: u32,
        /// Exception facts
        facts: crate::RetirementException,
        /// A second exception on the same distribution
        #[libertas_default($null)]
        second: Option<crate::RetirementException>,
        /// SIMPLE IRA within its first two years
        early_simple: bool,
    },
    /// Another treatment or not sure
    /// A box 7a code or combination this App does not calculate; the refusal names the record's box 7a codes.
    Unsupported,
    /// Lump-sum distribution taxed separately on Form 4972
    /// Form 1099-R code A with 7 or 4: a plan participant born before January 2, 1936.
    LumpSum {
        /// Form 4972 election
        data: crate::Form4972Election,
    },
    /// Corrective distribution of excess contributions or deferrals (code 8)
    /// No 10% additional tax and no rollover. From an employer plan it is wages on Form 1040 line 1h, less the excess deferral already counted from the W-2s; from an IRA it is on line 4b, outside Form 8606.
    Corrective {
        /// Part of box 2a that is a 2026 excess deferral already counted from the W-2s
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        excess_deferral: i64,
    },
    /// Qualified long-term-care cash-value charge, Form 1099-R code W
    /// Code 6 may also be printed for an accompanying tax-free section 1035 exchange. Source gross is not an additional deductible premium. Retain the actual reported investment reduction, not the gross charge or exchange value.
    QualifiedCareCharge {
        /// Actual investment reduction, Form 1099-R box 8a
        /// May be zero when no investment remained. Existing other-payment basis inputs must already reflect this adjustment; it is never subtracted twice.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        investment_reduction: i64,
    },
    /// Employer contribution to a Roth SEP or Roth SIMPLE IRA (code 2 or 7, box 7b)
    /// Reported as converted: taxable in full, no additional tax, outside Form 8606 Part I, a 2026 conversion for the Roth ordering.
    EmployerRothContribution,
}

/// Source classification for ordinary Form 1116 income and taxes
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignTaxCategory {
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
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        qualified: i64,
        /// Foreign-source sales in this country
        /// Choose the investment sales, already entered under Investment sales, whose gain or loss is from this country: real property located there, or personal property sold while your tax home was abroad with at least 10% foreign tax paid on the gain. Your other sales are U.S. source.
        sales: Vec<i64>,
    },
    /// Business run abroad
    /// A Schedule C business whose services were performed wholly abroad. Its gross income and expenses come from the business itself: enter zero income here.
    Business {
        /// Business
        business: i64,
        /// The business keeps its own books and records
        /// A separate set of books of original entry and ledger accounts for this business, such as its own receipts and disbursements journal. Yes makes its income foreign branch category income (Form 1116 box b); No makes it general category income (box d).
        separate_books: bool,
    },
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
        #[libertas_number(min = 0)]
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

/// Foreign withholding on direct passive income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ForeignWithholding {
    /// No foreign withholding tax
    /// Any separately assessed foreign income tax still belongs in the tax amount.
    None,
    /// Positive withholding rate below 15 percent
    Below15,
    /// Withholding rate of 15 percent or more
    AtLeast15,
}

impl ForeignWithholding {
    // The Form 1116 passive group (1.904-4(c)(3)) of a payment with this
    // withholding treatment and tax: 1 and 2 by rate, 3 for no tax, 4 for a
    // separately assessed tax with no withholding.
    pub(crate) fn group(&self, tax: i64) -> usize {
        match self {
            Self::AtLeast15 => 1,
            Self::Below15 => 2,
            Self::None if tax == 0 => 3,
            Self::None => 4,
        }
    }
}

/// Country and category tax record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ForeignOrdinaryTax {
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
    pub owner: Owner,
    /// Category of the foreign-source income already entered
    pub category: ForeignTaxCategory,
    /// Current-year gross income in this country and category
    /// Already included once in Income: interest, dividends, royalties or employee pay. Enter zero for a prior-year tax paid in 2026 and for a business run abroad.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub income: i64,
    /// Eligible foreign income tax for 2026 in this country and category
    /// Claiming taxes when paid: the tax paid in 2026, at the exchange rate on the date paid or withheld. Claiming them when accrued: the tax for 2026, at the 2026 average rate (the payment-date rate when paid before 2026, more than 2 years after it, or in an inflationary currency). Only tax you legally owe: leave out amounts you can get refunded, for example under a treaty, and a contested tax until it is settled. For a prior-year choice, only tax first paid in 2026 on the earlier income.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub tax: i64,
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
    #[libertas_number(min = 0)]
    pub income: i64,
    /// Eligible foreign income tax for 2026 in this country and category
    /// Claiming taxes when paid: the tax paid in 2026, at the exchange rate on the date paid or withheld. Claiming them when accrued: the tax for 2026, at the 2026 average rate (the payment-date rate when paid before 2026, more than 2 years after it, or in an inflationary currency). Only tax you legally owe: leave out amounts you can get refunded, for example under a treaty, and a contested tax until it is settled. For a prior-year choice, only tax first paid in 2026 on the earlier income.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub tax: i64,
}
impl ForeignOrdinaryTax {
    pub(crate) fn qualified(&self) -> i64 {
        match self.category {
            ForeignTaxCategory::Passive { qualified, .. } => qualified,
            _ => 0,
        }
    }
    // The Form 1116 grouping of a row (see `crate::form_1116::Group`): 0 general,
    // 1-4 the passive withholding groups, 5 foreign branch, 6 section 901(j)
    // (a row whose country is on the sanctioned list, whatever its category).
    pub(crate) fn group(&self) -> usize {
        if crate::year_constants::is_section_901j_country(&self.country) {
            return crate::form_1116::SANCTIONED;
        }
        match &self.category {
            ForeignTaxCategory::Employment | ForeignTaxCategory::PriorEmployeeWageTax { .. } => 0,
            ForeignTaxCategory::PriorPassiveTax { .. } => 4,
            ForeignTaxCategory::Passive { withholding, .. } => withholding.group(self.tax),
            ForeignTaxCategory::Business { separate_books: true, .. } => crate::form_1116::BRANCH,
            ForeignTaxCategory::Business { .. } => 0,
        }
    }
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

/// Owners of a prior return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PriorReturnOwners {
    /// Taxpayer only
    Taxpayer,
    /// Current spouse only
    Spouse,
    /// Taxpayer and current spouse jointly
    Joint,
}

/// 2025 return with a regular or AMT capital loss
/// Copy the filed return's amounts. The App calculates short- and long-term carryovers; do not enter a carryover you calculated yourself.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PriorCapitalReturn {
    /// Who filed this 2025 return
    pub owners: PriorReturnOwners,
    /// Filing status on that return
    pub status: crate::FederalStatus,
    /// 2025 Form 1040 line 11b adjusted gross income
    #[libertas_money("USD", 2)]
    pub agi: i64,
    /// 2025 Form 1040 line 14 total deductions
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub deductions: i64,
    /// 2025 Schedule 1-A line 37 enhanced deduction for seniors
    /// Enter the amount already included in Form 1040 line 14 above, or zero if none. Do not include the age/blindness addition to the standard deduction. The enhanced deduction is added back only when measuring capital-loss use, not when calculating tax.
    #[libertas_money("USD", 2)]
    // Tax structure: The 2025 enhanced senior deduction is at most $6,000 per filer, $12,000 jointly.
    #[libertas_number(min = 0, max = 1200000)]
    pub senior_deduction: i64,
    /// 2025 Schedule D line 7 net short-term gain or loss
    /// Enter a loss as negative.
    #[libertas_money("USD", 2)]
    pub short: i64,
    /// 2025 Schedule D line 15 net long-term gain or loss
    /// Enter a loss as negative.
    #[libertas_money("USD", 2)]
    pub long: i64,
    /// Loss deducted on 2025 Schedule D line 21, entered as positive
    #[libertas_money("USD", 2)]
    // Tax structure: Schedule D limits the annual deducted capital loss to at most $3,000.
    #[libertas_number(min = 0, max = 300000)]
    pub deducted_loss: i64,
    /// Part of the loss carried to 2026 that came from section 1256 contracts
    /// Enter the part of your 2025 capital loss carryover that came from a net section 1256 contracts loss (Form 6781, Publication 550 "Net section 1256 contracts loss"), as a positive amount, or zero when none of it did. A 2026 carryback election can then count it as a 2026 section 1256 loss.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub section_1256: i64,
    /// Separate 2025 AMT capital-loss worksheet
    /// Supply retained AMT worksheet amounts when its carryover differs from regular tax, including an AMT-only loss. Leave absent only when the records establish equal carryovers, not when AMT records are unavailable.
    /// [NullText]
    /// AMT capital-loss carryovers equal the regular carryovers
    /// [SetText]
    /// The different AMT capital-loss worksheet is
    #[libertas_default($null)]
    pub amt: Option<PriorAmtCapitalReturn>,
    /// Retained loss balances by original loss year
    /// Optional supporting history from reconciled loss worksheets, after 2025 use. Leave empty when unavailable. Include every surviving year if supplied; totals must match the carryovers calculated from this return. Do not invent years from the aggregate Schedule D amounts. This records provenance; general canceled-debt reductions still require further worksheets.
    pub origins: Vec<CapitalLossOrigin>,
    /// These losses belong entirely to the people filing this return
    /// No allocation to a former spouse, canceled-debt basis reduction, unresolved special capital-gain category or AMT adjustment applies. Any different AMT carryover is supplied in the separate worksheet above. The prior Schedule D and any divorce or estate documents show it. No leaves this prior-year capital loss carryover out of the estimate with a notice; that situation is not supported.
    pub ordinary_history: bool,
    /// Investment-property portions of the calculated carryovers
    /// Use retained loss-source records. Absence means attribution is unavailable, not that no loss is investment-related. These portions do not change the ordinary Schedule D carryover.
    /// [NullText]
    /// Investment-property loss attribution is unavailable
    /// [SetText]
    /// Investment-property loss portions are
    #[libertas_default($null)]
    pub investment_parts: Option<InvestmentCapitalParts>,
    /// Prior Form 8960 excluded capital-loss difference
    /// Positive amount from line 2(i), column A, of the retained 2025 Form 8960 net-gains worksheet. Enter zero when that worksheet establishes none. Leave absent only when no regular loss carries forward. Section 163(d) investment-property attribution above does not establish the separate section 1411 amount. This is not the ordinary Schedule D carryover; the App calculates that from the filed return.
    /// [NullText]
    /// No regular capital loss carries forward
    /// [SetText]
    /// The prior Form 8960 excluded capital-loss difference is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub niit_difference: Option<i64>,
    /// Each spouse’s surviving capital-loss amounts
    /// Only for a Form 8379 claim. Use actual source records; this never changes the joint calculation.
    /// [NullText]
    /// The surviving spouse shares have not been established
    /// [SetText]
    /// The actual surviving spouse shares are
    #[libertas_default($null)]
    pub injured_owners: Option<crate::InjuredCapitalLossOwners>,
    /// Actual owners of the retained Form 8960 excluded-loss difference
    /// Only for a Form 8379 claim. This is separate from regular and AMT capital-loss shares and from Form 4952 investment-property portions. Use each spouse's retained section 1411 records.
    /// [NullText]
    /// Owners of the retained Form 8960 difference have not been established
    /// [SetText]
    /// The retained Form 8960 difference belongs to
    #[libertas_default($null)]
    pub injured_niit_owners: Option<crate::InjuredRetainedOwners>,
    /// Actual current research-business portions of the regular carryover
    pub research_parts: Option<crate::ResearchCapitalParts>,
}

/// Investment-property portions of the computed capital-loss carryover
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentCapitalParts {
    /// Regular short-term investment loss portion
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub short: i64,
    /// Regular long-term investment loss portion
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub long: i64,
    /// Separate AMT portions, required when the AMT carryover differs
    /// [NullText]
    /// AMT investment-loss portions equal the regular portions
    /// [SetText]
    /// Different AMT investment-loss portions are
    #[libertas_default($null)]
    pub amt: Option<InvestmentAmtCapitalParts>,
}
/// Investment-property portions of the AMT carryover
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentAmtCapitalParts {
    /// AMT short-term investment loss portion
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub short: i64,
    /// AMT long-term investment loss portion
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub long: i64,
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

/// Retained 2025 AMT capital-loss worksheet facts
/// Use the completed prior AMT Schedule D and AMT Capital Loss Carryover Worksheet for the same return and owners. These are source worksheet lines, not a carryover amount calculated for this App.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PriorAmtCapitalReturn {
    /// AMT carryover worksheet taxable income before exemption
    /// Copy the signed income used on line 1 of the retained AMT Capital Loss Carryover Worksheet, including a negative amount. It excludes the AMT exemption; do not substitute a zero-floored Form 6251 total or the regular Form 1040 taxable income.
    #[libertas_money("USD", 2)]
    pub taxable_income: i64,
    /// AMT Schedule D line 7 net short-term gain or loss
    #[libertas_money("USD", 2)]
    pub short: i64,
    /// AMT Schedule D line 15 net long-term gain or loss
    #[libertas_money("USD", 2)]
    pub long: i64,
    /// AMT Schedule D line 21 loss, as a positive amount
    #[libertas_money("USD", 2)]
    // Tax structure: Schedule D limits the annual deducted capital loss to at most $3,000.
    #[libertas_number(min = 0, max = 300000)]
    pub deducted_loss: i64,
}

/// Surviving capital loss from an original tax year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CapitalLossOrigin {
    /// Original loss year, not the year of the latest carryover worksheet
    // Tax structure: this date or year belongs to the stated current or retained-return period.
    #[libertas_number(min = 1900, max = 2025)]
    pub year: u32,
    /// Unused short-term loss entering 2026
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub short: i64,
    /// Unused long-term loss entering 2026
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub long: i64,
}

impl crate::FederalRetirement {
    // A distribution outside Form 8606 line 7: the return of an excess contribution (code 8, and
    // the earnings it carries) and the recharacterizations (N, R) and an earlier year's
    // return (P) are not distributions of the IRA's basis (i8606, "a distribution doesn't
    // include ... return of certain contributions").
    pub(crate) fn outside_form_8606(&self) -> bool {
        // An employer's Roth SEP or Roth SIMPLE contribution is reported as converted money the
        // person never held in a traditional IRA of their own.
        self.ordinary_distribution == RetirementTreatment::EmployerRothContribution
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
            crate::RetirementTaxable::QualifiedWithdrawal { .. }
                | crate::RetirementTaxable::FullDischarge { .. }
                | crate::RetirementTaxable::PropertyProceeds { .. }
        )
    }
    pub(crate) fn supported_plan_rollover(&self) -> bool {
        if let crate::RetirementTaxable::PropertyProceeds { ref data } = self.taxable {
            return (0..=data.proceeds).contains(&self.rolled_amount());
        }
        self.rolled_amount() <= self.taxable_before_rollover()
            || matches!(
                self.ordinary_distribution,
                RetirementTreatment::Rollover {
                    destination: RolloverDestination::Ira
                        | RolloverDestination::IraNextYear
                        | RolloverDestination::DirectEmployerPlan { .. }
                        | RolloverDestination::Roth,
                    ..
                }
            )
    }
    pub(crate) fn rolled_basis(&self) -> i64 {
        match &self.taxable {
            crate::RetirementTaxable::PropertyProceeds { .. } => 0,
            _ => phase::floor(self.rolled_amount() - self.taxable_before_rollover(), 0),
        }
    }
    pub(crate) fn rolled_amount(&self) -> i64 {
        match self.ordinary_distribution {
            RetirementTreatment::Rollover { amount, .. } => amount,
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
            RetirementTreatment::Rollover {
                amount,
                destination: RolloverDestination::IraNextYear,
                ..
            } if self.kind == crate::RetirementKind::Ira => amount,
            _ => 0,
        }
    }
    pub(crate) fn converted_amount(&self) -> i64 {
        match self.ordinary_distribution {
            RetirementTreatment::Conversion { amount, .. } => amount,
            _ => 0,
        }
    }
    pub(crate) fn taxable_before_rollover(&self) -> i64 {
        // Basis selections use gross provisionally; the complete estimate must resolve
        // their owner-level worksheet before returning a result.
        match self.taxable {
            crate::RetirementTaxable::Payer { amount } => amount,
            crate::RetirementTaxable::CivilServiceDeath {
                data: crate::SurvivorLumpSum { cost, recovered, .. },
            } => phase::floor(self.gross - (cost - recovered), 0),
            crate::RetirementTaxable::PropertyProceeds { .. } => self.gross,
            crate::RetirementTaxable::IraBasis => self.gross,
            // The Inherited IRA record's group decides (inherited_ira.rs).
            crate::RetirementTaxable::Inherited { .. } => 0,
            crate::RetirementTaxable::FullDischarge {
                data: crate::ContractDischarge { investment, .. },
            } => phase::floor(self.gross - investment, 0),
            crate::RetirementTaxable::QualifiedWithdrawal {
                data: crate::PlanDistribution { value, investment, .. },
            } => {
                self.gross
                    - phase::cap(
                        crate::kernel::ratio::fraction(self.gross, investment, value.max(1)),
                        [self.gross],
                    )
            }
            crate::RetirementTaxable::LegacyAnnuity { ref data } => data.taxable(self.gross),
            crate::RetirementTaxable::DeferredWithdrawal { ref data } => {
                let [value, investment] =
                    crate::retirement::annuity_balances(data.value, data.investment, &data.related);
                let taxable = (value - investment)
                    .max(crate::kernel::ratio::exact::whole(0))
                    .min(crate::kernel::ratio::exact::whole(self.gross));
                // A valid source's taxable portion is bounded by its native gross.
                crate::kernel::ratio::exact::rounded(&taxable)
                    .expect("annuity taxable portion is within the original gross")
            }
            crate::RetirementTaxable::GeneralAnnuity { data } => self.gross - data.excluded(),
            crate::RetirementTaxable::VariableAnnuity { data } => {
                self.gross - data.excluded(self.gross)
            }
            crate::RetirementTaxable::Railroad { contributory, ref basis, .. } => {
                self.gross - basis.as_ref().map_or(0, |b| b.excluded(contributory))
            }
            crate::RetirementTaxable::ChildAnnuity { ref data } => self.gross - data.result().0,
            crate::RetirementTaxable::Simplified { ref data } => {
                self.gross - data.excluded(self.gross)
            }
        }
    }
    pub(crate) fn taxable_income(&self) -> i64 {
        if let crate::RetirementTaxable::PropertyProceeds { ref data } = self.taxable {
            return data.pension(self.gross, self.rolled_amount());
        }
        let taxable = self.taxable_before_rollover();
        match self.ordinary_distribution {
            // Pretax money rolled into a Roth IRA or designated Roth account is taxed (sections
            // 408A(d)(3)(A), 402A(c)(4)(A)); a designated Roth account distribution (code B) is
            // earnings first, so what is rolled over comes out of the earnings (Pub. 575).
            RetirementTreatment::Rollover { destination: RolloverDestination::Roth, .. }
                if self.box7 & crate::DistributionCodes::CODE_B.0 == 0 =>
            {
                taxable
            }
            RetirementTreatment::Rollover { amount, .. } => {
                phase::floor(taxable - amount + self.rolled_basis(), 0)
            }
            // A corrective distribution is taxed as printed, less an excess deferral already
            // counted from the W-2s (the line adds that part in retirement::early_tax's caller).
            RetirementTreatment::Corrective { .. } => taxable,
            // The earlier year's return, or the other kind of IRA, owns these.
            _ if self.box7
                & (crate::DistributionCodes::CODE_P.0 | crate::DistributionCodes::CODE_R.0)
                != 0 =>
            {
                0
            }
            // Form 4972 takes the elected amount out of income; Part II alone leaves the
            // ordinary part.
            RetirementTreatment::LumpSum { ref data } => data.ordinary_part(taxable),
            _ => taxable,
        }
    }
}

/// Nontaxable pension or disability benefit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ExemptPension {
    /// Benefit or payer
    #[libertas_size(min = 1)]
    pub label: String,
    /// Nontaxable amount received
    /// Include veterans' pensions and other pension, annuity or disability benefits excluded under federal law other than the Internal Revenue Code for Schedule R. Do not include the section 402(l) public-safety insurance exclusion. Exclude Social Security already entered, return of your contributions, rollovers, and VA disability compensation for personal injury or sickness.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
}

/// Prior suspended rental loss
/// Loss from prior filed returns; both zero when there is none. Enter it only when the records are complete and the activity's participation and QBI treatment are unchanged: same property and ownership; no basis or at-risk limitation. Active-participation losses require active participation in their original years. For a QBI business, all entered losses arose after 2017 and retain the same QBI character.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalHistory {
    /// Regular-tax unallowed loss from this property's prior Form 8582 records
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub regular: i64,
    /// AMT unallowed loss from this property's prior AMT Form 8582 records
    /// Enter the separately tracked amount, even if it equals the regular-tax loss.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
        /// ----
        /// Employee statement
        #[libertas_size(min = 1)]
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
        #[libertas_number(min = 0)]
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
    instruction = "Read an employee Form W-2 issued by this business for the supplied tax year. Report a mismatched year, illegible or conflicting values as errors. Return only printed box 1 wages and box 5 Medicare wages and tips; a blank box is zero. Copies of the same employee form are one form. Do not return employee personal identifiers, change the manual statement label, infer QBI attribution or confirmations, or add payroll expenses.",
    doc_type = "application/pdf,image/*"
)]
pub struct PayrollW2 {
    /// Statement label, without personal identifiers
    /// Identifies this W-2 when an employer credit lists the employee, so each label is used once in this business's payroll.
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_size(min = 1)]
    pub label: String,
    /// Box 1 wages
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub wages: i64,
    /// Box 5 Medicare wages and tips
    #[libertas_llm_doc_field(allow_empty = false)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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

/// HSA contribution eligibility during 2026
/// Use actual first-day eligibility, including retroactive Medicare enrollment. Eligible bronze/catastrophic coverage, telehealth and qualifying direct-primary-care exceptions follow the monthly coverage choices; owning an HSA alone does not establish contribution eligibility.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HsaEligibility {
    /// Same eligibility all year
    FullYear { coverage: crate::HsaCoverage },
    /// Eligibility changes during the year
    Monthly {
        /// January through December, in order
        /// Use eligibility on the first day of each month. Medicare enrollment, including retroactive enrollment, makes that month ineligible.
        // Tax structure: one eligibility entry for each of the twelve calendar months.
        #[libertas_size(min = 12, max = 12)]
        months: Vec<crate::HsaCoverage>,
    },
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
        #[libertas_number(min = 0)]
        opening: i64,
        /// Purchases for resale or production, net of returns
        /// Include acquisition costs consistently with your accounting method. Do not enter these purchases again as operating expenses.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        purchases: i64,
        /// Cost of goods withdrawn for personal use
        /// Use cost, not retail selling price.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        personal: i64,
        /// Production labor costs
        /// Exclude your own labor and any costs already included in operating expenses.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        labor: i64,
        /// Materials and supplies used in production
        /// Exclude amounts already included in purchases or operating expenses.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        materials: i64,
        /// Other costs included in inventory
        /// Exclude costs already counted elsewhere.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        other: i64,
        /// Closing inventory valued by the selected method
        /// Use a physical inventory or reliable perpetual records with the same valuation method as opening stock.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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

/// Vehicle using the standard-mileage method
/// Car, van, pickup or panel truck; no five-or-more-vehicle simultaneous fleet. For owned vehicles, standard mileage was chosen in the first business-use year and no disqualifying accelerated depreciation, bonus, section 179 or special allowance was claimed. Leased vehicles use standard mileage throughout the lease and renewals. Maintain mileage/business-purpose records. No actual operating costs, lease payments or depreciation claimed again; no sale, trade-in or recapture this year. Enter this vehicle only once for this business and allocate any other business's use separately.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessMileage {
    /// Canonical vehicle identity assigned on Save
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Date placed in service for business (Schedule C Part IV line 43)
    #[libertas_date_only]
    #[libertas_default($null)]
    pub placed: Option<u32>,
    /// Available for personal use during off-duty hours (line 45)
    #[libertas_default(true)]
    pub personal_use_available: bool,
    /// Another vehicle available for personal use (line 46)
    #[libertas_default(true)]
    pub other_vehicle: bool,
    /// Evidence supporting the vehicle deduction (lines 47a and 47b)
    #[libertas_default(Written)]
    pub evidence: crate::VehicleEvidence,

    /// Business miles from January 1 through June 30
    /// Exclude commuting, personal travel and miles claimed by another business.
    #[libertas_number(min = 0)]
    pub first_half: u32,
    /// Business miles from July 1 through December 31
    #[libertas_number(min = 0)]
    pub second_half: u32,
    /// All miles driven during 2026
    /// Include business, commuting and personal miles; used to allocate interest and vehicle tax.
    #[libertas_number(min = 0)]
    pub total: u32,
    /// Commuting miles included in the annual total
    #[libertas_number(min = 0)]
    pub commuting: u32,
    /// Business parking and tolls, excluding commuting
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub parking_tolls: i64,
    /// Annual vehicle-loan interest paid
    /// Enter the whole vehicle amount. The App allocates its business share; do not deduct that share elsewhere. Enter zero for a leased vehicle.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub interest: i64,
    /// Annual value-based personal property tax on the vehicle
    /// Enter the whole vehicle amount. Do not include registration/license fees or deduct the calculated business share elsewhere.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub property_tax: i64,
    /// You own this vehicle
    /// Turn off for a leased vehicle. Only owned vehicles receive a depreciation basis-reduction amount.
    pub owned: bool,
}
impl BusinessMileage {
    pub(crate) fn records(income: &crate::FederalIncome) -> &[Self] {
        match income {
            crate::FederalIncome::Business { data } => &data.vehicles,
            crate::FederalIncome::FarmRental { data } => &data.vehicles,
            _ => &[],
        }
    }
    pub(crate) fn ids_mut(income: &mut crate::FederalIncome) -> Vec<&mut i64> {
        let rows = match income {
            crate::FederalIncome::Business { data } => &mut data.vehicles,
            crate::FederalIncome::FarmRental { data } => &mut data.vehicles,
            _ => return Vec::new(),
        };
        rows.iter_mut().map(|row| &mut row.id).collect()
    }
    pub(crate) fn miles(&self) -> i64 {
        i64::from(self.first_half) + i64::from(self.second_half)
    }
    pub(crate) fn mileage_cost_twice(&self) -> i64 {
        i64::from(self.first_half) * crate::year_constants::MILEAGE_RATE_2026_JAN_JUN_HALF_CENTS
            + i64::from(self.second_half)
                * crate::year_constants::MILEAGE_RATE_2026_JUL_DEC_HALF_CENTS
    }
    fn additional_cost(&self) -> i64 {
        self.parking_tolls
            + if self.total == 0 {
                0
            } else {
                crate::kernel::ratio::fraction(
                    self.interest + self.property_tax,
                    self.miles(),
                    i64::from(self.total),
                )
            }
    }
    // The business-use share of the vehicle-loan interest (Schedule C line 16b,
    // Schedule F line 21b in the 2026 drafts): annual interest times business
    // miles over total miles, the allocation `additional_cost` already makes.
    fn loan_interest_share(&self) -> i64 {
        if self.total == 0 {
            0
        } else {
            crate::kernel::ratio::fraction(self.interest, self.miles(), i64::from(self.total))
        }
    }
    // The business-use share of the vehicle's personal property tax, which a
    // standard-mileage vehicle reports on Schedule C line 23 (2026 i1040sc line 9,
    // "Standard mileage rate"), not on line 9.
    fn property_tax_share(&self) -> i64 {
        if self.total == 0 {
            0
        } else {
            crate::kernel::ratio::fraction(self.property_tax, self.miles(), i64::from(self.total))
        }
    }
    pub(crate) fn expense(&self) -> i64 {
        crate::kernel::ratio::fraction(self.mileage_cost_twice(), 1, 2) + self.additional_cost()
    }
    // The vehicles' business-use property tax, which prints on line 23.
    pub(crate) fn property_tax(vehicles: &[Self]) -> i64 {
        vehicles.iter().map(Self::property_tax_share).sum()
    }
    // The vehicles' business-use loan interest, which prints on its own line.
    pub(crate) fn loan_interest(vehicles: &[Self]) -> i64 {
        vehicles.iter().map(Self::loan_interest_share).sum()
    }
    pub(crate) fn total(vehicles: &[Self]) -> i64 {
        crate::kernel::ratio::fraction(vehicles.iter().map(Self::mileage_cost_twice).sum(), 1, 2)
            + vehicles.iter().map(Self::additional_cost).sum::<i64>()
    }
    pub(crate) fn basis_reduction(&self) -> i64 {
        if self.owned {
            self.miles() * crate::year_constants::MILEAGE_DEPRECIATION_COMPONENT_2026_CENTS_PER_MILE
        } else {
            0
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.first_half <= 1000000
            && self.second_half <= 1000000
            && self.total <= 2000000
            && self.commuting <= 2000000
            && self.miles() + i64::from(self.commuting) <= i64::from(self.total)
            && [self.parking_tolls, self.interest, self.property_tax]
                .iter()
                .all(|v| (0..=100000000000).contains(v))
            && (self.owned || self.interest == 0)
            && (self.miles() > 0 || self.parking_tolls == 0)
    }
}

/// Disability retirement pay included in this income entry
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum DisabilityIncome {
    /// None
    None,
    /// Taxable disability retirement income
    Qualifying {
        /// Qualifying disability amount already included in this entry
        /// Enter only employer accident, health or pension-plan payments replacing wages while absent due to permanent and total disability. Exclude accrued leave, Social Security, tax-free benefits and amounts after mandatory retirement age; do not add this amount to income again.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
        /// Retired on permanent and total disability, with the required medical certification
        /// You were disabled when you retired, had not reached mandatory retirement age on January 1, 2026, and retain the Schedule R physician/VA statement or qualify to rely on the prior statement. Wage versus pension reporting follows minimum retirement age separately. Check the physician/VA certification and employer retirement records. No claims no Schedule R credit on this income.
        eligible: bool,
    },
}
impl DisabilityIncome {
    pub(crate) fn amount(&self) -> i64 {
        match self {
            Self::None => 0,
            Self::Qualifying { amount, .. } => *amount,
        }
    }
    pub(crate) fn eligible(&self) -> bool {
        matches!(self, Self::None | Self::Qualifying { eligible: true, .. })
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

impl Form3921 {
    // The page's Form 3921 onto its stored exercise; the page's own
    // questions stand for the stored confirmation.
    pub(crate) fn record(&self, owner: Owner) -> IsoExercise {
        let Form3921More { sold, recognition, option_cost } = self.more.clone();
        IsoExercise {
            owner,
            label: self.transferor.clone(),
            exercised: self.exercised,
            recognition,
            exercise_price: self.exercise_price,
            market_price: self.market_price,
            option_cost,
            shares: self.shares,
            granted: self.granted,
            corporation: crate::common::optional_text(&self.corporation),
            sold,
        }
    }
    // The reverse of record; None for a stored exercise this page could not
    // have written.
    pub(crate) fn from_record(v: &IsoExercise) -> Option<Self> {
        let more = Form3921More {
            sold: v.sold,
            recognition: v.recognition.clone(),
            option_cost: v.option_cost,
        };
        Some(Self {
            transferor: v.label.clone(),
            granted: v.granted,
            exercised: v.exercised,
            exercise_price: v.exercise_price,
            market_price: v.market_price,
            shares: v.shares,
            corporation: v.corporation.clone().unwrap_or_default(),
            more,
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
                investment_assets,
                loss_history,
                state_taxes,
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
                investment_assets,
                loss_history,
                state_taxes,
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

impl RentalPropertyType {
    pub(crate) fn nonresidential(&self) -> bool {
        matches!(self, Self::Commercial | Self::SelfRental | Self::Other { .. })
            || matches!(self, Self::VacationShortTerm { stays } if !matches!(stays, ShortTermStays::OverSevenDays))
    }
    pub(crate) fn business_participation(&self) -> Option<bool> {
        match self {
            Self::VacationShortTerm {
                stays:
                    ShortTermStays::SevenDaysOrLess { material_participation }
                    | ShortTermStays::ThirtyDaysOrLessWithServices { material_participation },
            } => Some(*material_participation),
            _ => None,
        }
    }
    pub(crate) fn substantial_services(&self) -> bool {
        matches!(self, Self::VacationShortTerm { stays: ShortTermStays::SubstantialServices })
    }
}
impl RentalAtRisk {
    // Mechanical mapping for the unchanged farm editor and format124 rehearsal.
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
            home_depreciation_carryover,
            amt_home_depreciation_carryover,
            home_casualty_carryover,
            amt_home_casualty_carryover,
            sale,
            statement,
            mortgage_loans,
            participation,
            ordinary_rental,
            qbi_business,
            qbi_aggregation_group,
            property_type,
        } = self;
        Self {
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
            home_depreciation_carryover: crate::community::Divide::divided(
                home_depreciation_carryover,
                share,
            ),
            amt_home_depreciation_carryover: crate::community::Divide::divided(
                amt_home_depreciation_carryover,
                share,
            ),
            home_casualty_carryover: crate::community::Divide::divided(
                home_casualty_carryover,
                share,
            ),
            amt_home_casualty_carryover: crate::community::Divide::divided(
                amt_home_casualty_carryover,
                share,
            ),
            sale: crate::community::Divide::divided(sale, share),
            statement: crate::community::Divide::divided(statement, share),
            mortgage_loans: mortgage_loans.clone(),
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
    facts: [active_participation, qbi_business, qbi_aggregation_group],
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

        investment_assets: crate::ForeignInvestmentAssets,

        loss_history: crate::ForeignLossHistory,

        state_taxes: crate::ForeignStateTaxes,

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
