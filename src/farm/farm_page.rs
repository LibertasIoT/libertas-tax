use crate::validation::ValidationIssue;
// Coverage: T2-45p Farming topic page types (2026-09-27). The page kinds of the Farming income
//   topic:
//   Farm (a Schedule F farm onto FederalBusiness with BusinessActivity::Farming), FarmAveragingElection
//   (FederalScreening.farm_averaging) and FarmLossWaiver (FederalScreening.farm_nol). The mapping
//   both ways, the rules' server twins and the page mechanics live in interview.rs (its T2-45p
//   Coverage block); the calculation stays in farm.rs and its sibling modules.
//   Unit B2 (2026-10-09): FarmRental and FarmRentalDetails, the Farm rental (Form 4835) kind, reuse
//   this file's CropInsurance, FarmExpenses, CccLoans, Conservation and LandSale types and map onto
//   the stored FederalFarmRental (farm_rental_record / farm_rental_item / farm_rental_rule). The
//   reused types keep their Schedule F line labels: a Display Template `label` cannot rename a
//   scalar field, so the Form 4835 numbers cannot be shown on them yet (see
//   rust-tax-design-improvements.md).
//   T2-src: FarmIncomeSource drops Services (Not needed, Capital, Both) and Both carries
//   FarmWorkValue (the 30% answer or an amount); see interview.rs's T2-src block.
//   T2-cy: income_source is ConstrainedBy allowed_farm_source ([Not needed] or [Capital, Both])
//   with no Default; see interview.rs's T2-cy block.
//   Gate fixes, batch C (2026-10-06, "Ask once, directly"): the More switch is the always-present
//   FarmDetails record (folded closed, tabbed by display.json); BaseYear carries its two Form
//   2555 / Form 4952 Options directly; PoultryRecords is a record of two lists that may be empty;
//   AveragingCapital is a record of four amounts (zero is no capital gain). A farm's income
//   source sits under its allowed set without a switch in front of it.
use crate::interview::{shared_business_issue, vehicle_and_asset_items, vehicle_and_asset_records};
use crate::{
    AveragingBaseYear, AveragingSource, BusinessActivity, BusinessExpenseCategory,
    BusinessExpenseKind, BusinessHealthInsurance, BusinessIncomeSource, BusinessInventory,
    BusinessRetirement, BusinessSource, CccDisposition, CccLot, CccOrigin, CropPayment, CrpPayment,
    EquipmentClass, EquipmentMethod, FarmAccounting, FarmAveraging, FarmCcc, FarmConservation,
    FarmIncome, FarmLandSale, FarmNolElection, FarmPoultryMethod, FarmPoultryResale,
    FarmProductionPoultryCarry, FarmReceipt, FarmSupplies, FarmSupplyAdvance, FarmSupplyCarry,
    FarmSupplyException, FarmSupplyYear, FederalBusiness, FederalBusinessExpense,
};
use alloc::{string::String, vec::Vec};
use libertas_macros::*;

/// Farm (Schedule F)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct Farm {
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
    /// Related forms
    /// Select this activity's cooperative distributions, farm program payments and crop-insurance forms. Edit printed amounts in Documents.
    pub related: crate::RelatedForms,
    /// Farm name or principal crop
    /// Titles this farm's row (Schedule F line A). Leave empty to use "Farm".
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub name: String,
    /// Accounting method (line C)
    /// Most farms use the cash method. Choose accrual only if this farm already reports on it.
    #[libertas_default(Cash)]
    pub accounting: FarmMethod,
    /// Line 6 crop insurance and federal crop disaster payments
    /// Additional payments absent from related Forms 1099-MISC. The App counts related printed amounts automatically. Open the crop-insurance details to answer only their missing timing or physical-loss facts.
    #[libertas_default(None)]
    pub insurance: CropInsurance,
    /// Part II expenses
    pub expenses: FarmExpenses,
    /// Line 10 cars and trucks
    #[libertas_unordered]
    /// ----
    /// Vehicle
    pub vehicles: Vec<crate::Vehicle>,
    /// Line 14 machinery, buildings and other property
    /// Include breeding and dairy livestock, orchards and farm buildings you depreciate.
    #[libertas_unordered]
    /// ----
    /// Asset
    pub assets: Vec<crate::Asset>,
    /// More about this farm
    pub more: FarmDetails,
    /// State information from this farm's 1099-NEC, 1099-MISC or 1099-K forms
    /// These records do not add federal receipts or withholding a second time.
    /// ----
    /// Information form
    pub state_documents: Vec<crate::BusinessStateDocument>,
}

/// Accounting method (line C)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmMethod {
    /// Cash method
    Cash {
        /// Line 1a sales of livestock and other items you bought for resale
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        resale: i64,
        /// Line 1b cost or other basis of those items
        /// What you paid for the items sold, even in an earlier year, and not deducted anywhere else.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        resale_cost: i64,
        /// Line 2 sales of livestock, produce, grains and other products you raised
        /// Not breeding, dairy or draft animals, machinery or land: those are sales of business property.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        raised: i64,
        /// Line 4 agricultural program payments not on a Form 1099-G
        /// Fully taxable payments only. A Form 1099-G box 7 amount is entered on that 1099-G and the App adds it here. CRP payments and CCC loans go under More about this farm.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        program: i64,
        /// Line 7 custom hire (machine work) income
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        custom_hire: i64,
        /// Line 8 other farm income
        /// Incidental operating income; not rent, interest, canceled debt or sales of business property.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        other: i64,
    },
    /// Accrual method
    Accrual {
        /// Line 37 sales of livestock, produce, grains and other products
        /// All livestock and products raised or purchased for sale; exclude equipment, land and depreciable livestock dispositions. Include earned receivables once, not again when collected. Exclude sales entered in CCC lot records; the App adds those separately.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        sales: i64,
        /// Line 39 agricultural program payments not on a Form 1099-G
        /// Exclude cooperative distributions, CCC loans, conservation-reserve payments, insurance and special subsidy treatments.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        program: i64,
        /// Line 42 custom hire (machine work) income
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        custom_hire: i64,
        /// Line 43 other farm income
        /// Exclude separately reported rent, interest, asset gains, debt discharge and tax-benefit recoveries needing another worksheet.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        other: i64,
        /// Line 45 inventory at the beginning of 2026
        /// Retained prior-year closing tax valuation, including required raised and purchased products and all crops pledged for or redeemed from loan-method CCC advances. Exclude depreciable assets and costs already deducted outside inventory.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        opening: i64,
        /// Line 46 cost of livestock and products you bought in 2026
        /// Costs recognized under the accrual method, including unsold products included in closing inventory. Do not repeat these costs in operating expenses or include depreciable asset purchases.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        purchases: i64,
        /// Line 48 inventory at the end of 2026
        /// Use the same established permitted valuation method and all required inventory classes, including unsold crops pledged for or redeemed from loan-method CCC advances. Raised inventory may increase without purchased livestock/product costs; the App does not cap closing inventory at purchases plus opening inventory.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        closing: i64,
    },
}

/// Line 6 crop insurance and federal crop disaster payments
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum CropInsurance {
    /// No crop insurance or disaster payments
    None,
    /// Payments in 2026, or an amount deferred from 2025
    Payments {
        /// Line 6a payments for physical crop damage or prevented planting
        #[libertas_unordered]
        /// ----
        /// Payment
        physical: Vec<CropLoss>,
        /// Line 6a payments based only on price or a rainfall index
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        price_only: i64,
        /// Line 6d amount deferred from 2025
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        from_prior_year: i64,
        /// Federal income tax withheld from these payments
        /// Exclude withholding already entered on a Form 1099-MISC in Documents.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        withholding: i64,
    },
}

/// Payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CropLoss {
    /// Amount received
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Year of the damage
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = 2026)]
    pub year: u32,
    /// Identified damaged crop from the insurer and farm records
    #[libertas_size(min = 1)]
    pub crop: String,
}

/// Part II expenses
/// Enter each line from your records; leave the others at zero. Car and truck, conservation and depreciation are figured from the vehicle, property and conservation records you enter elsewhere, not here. Feed, seed and fertilizer paid for in 2026 but unused at year end are not entered here either; answer where this farm asks about prepaid supplies or special situations.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmExpenses {
    /// Line 11 chemicals
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub chemicals: i64,
    /// Line 13 custom hire (machine work)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub custom_hire: i64,
    /// Line 15 employee benefit programs other than on line 23
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub employee_benefits: i64,
    /// Line 16 feed
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub feed: i64,
    /// Line 17 ordinary fertilizers and lime
    /// Keep capital costs entered under the section180 records out of this ordinary amount; their supported election is added once.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub fertilizers: i64,
    /// Line 18 freight and trucking
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub freight: i64,
    /// Line 19 gasoline, fuel and oil
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub fuel: i64,
    /// Line 20 insurance, other than health
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub insurance: i64,
    /// Line 21a mortgage interest paid to banks
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub mortgage_interest: i64,
    /// Line 21c other interest
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_interest: i64,
    /// Line 22 labor hired, less employment credits
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub labor: i64,
    /// Line 23 pension and profit-sharing plans
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub pension: i64,
    /// Line 24a rent or lease of vehicles, machinery and equipment
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub equipment_rent: i64,
    /// Line 24b rent or lease of land, animals and other property
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_rent: i64,
    /// Line 25 repairs and maintenance
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub repairs: i64,
    /// Line 26 seeds and plants
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub seeds: i64,
    /// Line 27 storage and warehousing
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub storage: i64,
    /// Line 28 supplies
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub supplies: i64,
    /// Line 29 taxes
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub taxes: i64,
    /// Line 30 utilities
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub utilities: i64,
    /// Line 31 veterinary, breeding and medicine
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub veterinary: i64,
    /// Line 32 business meals, before the 50% limit
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub meals: i64,
    /// Line 32 other expenses
    #[libertas_unordered]
    /// ----
    /// Other expense
    pub other: Vec<crate::OtherExpense>,
}

/// More about this farm
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmDetails {
    /// Original methods and special source records
    pub specials: crate::FarmSpecialsInput,
    /// Farm parcel opening/acquisition basis and retained history
    /// Save new unlinked parcels, then reopen to select their accepted identities.
    pub land_basis: Vec<crate::FarmLandBasis>,
    /// Schedule F lines B, D, F and G
    pub header: crate::FarmHeaderInput,
    /// Line 5 Commodity Credit Corporation loans
    #[libertas_default(None)]
    pub ccc: CccLoans,
    /// Conservation Reserve Program payments
    /// Enter each payment here and not again as a program payment or on a Form 1099-G.
    #[libertas_unordered]
    /// ----
    /// CRP payment
    pub crp: Vec<CrpItem>,
    /// Line 4 conservation cost-sharing payments you exclude (section 126)
    /// A cost-sharing payment you do not exclude is an ordinary program payment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub cost_sharing: i64,
    /// Feed, seed, fertilizer and other supplies paid for but unused at the end of 2026
    #[libertas_default(None)]
    pub supplies: PrepaidSupplies,
    /// Line 12 soil and water conservation
    /// Terracing, ponds, erosion control and similar work under a conservation plan. Not ordinary upkeep or depreciable structures.
    #[libertas_default(None)]
    pub conservation: Conservation,
    /// Farm land sold in 2026
    /// Land only; machinery, buildings and livestock are property sold.
    #[libertas_unordered]
    /// ----
    /// Land sale
    pub land_sales: Vec<LandSale>,
    /// Start-up costs
    #[libertas_default(None)]
    pub startup: crate::BusinessStartup,
    /// Accounting-method changes (Form 3115)
    /// Only a change whose section 481(a) adjustment still has an amount in 2026.
    #[libertas_unordered]
    /// ----
    /// Change
    pub method_changes: Vec<crate::MethodChange>,
    /// Employees' W-2 wages
    /// Farm wages reported on Forms W-2 (Form 943).
    #[libertas_default(No)]
    pub payroll: crate::BusinessPayroll,
    /// Federal income tax withheld on a 1099-MISC, 1099-NEC or 1099-K for this farm
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Use the farm optional method for self-employment tax (Schedule SE Part II)
    /// It can give you Social Security credit in a low-income year. It applies to all your farms.
    #[libertas_default(false)]
    pub optional_se: bool,
    /// Qualifying work and material-participation test for this farm (line E)
    pub material_participation: crate::ParticipationFacts,
    /// You are at risk for all of this farm's expenses (line 36a)
    #[libertas_default(true)]
    pub at_risk: bool,
    /// Group this farm with others for the qualified business income deduction
    /// As on the Self-employment page.
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub qbi_group: String,
    /// What produces this farm's income
    /// Asked only when the tax on a child's unearned income (Form 8615) can apply to this return.
    #[libertas_constrained_by("$.allowed_farm_source")]
    pub income_source: FarmIncomeSource,
    /// Federal fuel tax credit (Form 4136)
    /// Federal excise tax on fuel this business used off-highway, on a farm or in another nontaxable use. The credit is a payment on Schedule 3 line 12.
    /// [NullText]
    /// No federal fuel tax credit claimed for this farm
    /// [SetText]
    /// The federal fuel tax credit claim is
    #[libertas_default($null)]
    pub fuel_tax_credit: Option<crate::FederalFuelTaxCredit>,
    /// Fuel tax credit or refund from an earlier year, received in 2026
    /// For a cash-method business: an earlier year's Form 4136 credit claimed on a return filed in 2026, or a fuel-tax refund received in 2026, when the fuel's full cost was deducted.
    /// [NullText]
    /// No earlier fuel-tax credit or refund is reportable in 2026
    /// [SetText]
    /// Reportable earlier fuel-tax credits or refunds are
    pub cash_fuel_tax_recovery: Option<crate::FederalCashFuelRecovery>,
    /// Qualified tips included in this farm's other operating receipts (Schedule 1-A)
    /// Only tips separately reported to you on a 2026 information return and already inside this farm's other farm operating receipts. The App deducts them on Schedule 1-A, within the annual cap and income phaseout, and leaves them out of the qualified business income deduction. Leave empty when there are none.
    /// [NullText]
    /// No qualified-tip deduction claimed for these farm receipts
    /// [SetText]
    /// Qualified tips claimed from these farm receipts are
    #[libertas_default($null)]
    pub tips: Option<crate::FederalBusinessTips>,
}

/// Farm rental (Form 4835)
/// Rent paid in crops or livestock (a share of production) from land you own or sublease, when you did not materially participate in producing them. You materially participate when your agreement with the tenant says you will and you do one of these: at least three of paying half the production costs, furnishing half the tools, equipment and livestock, advising the tenant, and inspecting production; regularly making important management decisions; or working 100 hours or more over 5 weeks. If you did, enter a Farm instead. Flat cash rent is a Rental property; an estate, trust or partnership does not use this form.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmRental {
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
    /// Related forms
    /// Select this activity's cooperative distributions, farm program payments and crop-insurance forms. Edit printed amounts in Documents.
    pub related: crate::RelatedForms,
    /// Farm name or location
    /// Titles this rental's row. Leave empty to use "Farm rental".
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub name: String,
    /// Line 1 crop and livestock shares converted to money
    /// Your shares sold in 2026, and the market value of shares you fed to your own livestock or gave away, whichever accounting method you use. Crops under a CCC loan go under More about this farm rental.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub production: i64,
    /// Line 3 agricultural program payments not on a Form 1099-G
    /// Fully taxable payments only. A Form 1099-G box 7 amount is entered on that 1099-G and related to this rental; the App adds it here. Conservation Reserve Program payments belong on a Farm.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub program: i64,
    /// Line 5 crop insurance and federal crop disaster payments
    /// Additional payments absent from related Forms 1099-MISC. The App counts related printed proceeds automatically.
    #[libertas_default(None)]
    pub insurance: CropInsurance,
    /// Line 6 other income
    /// Such as machinery rented with the land under the share lease, or a fuel tax credit or refund included in income. Not cash rent, interest, canceled debt or sales of land or equipment.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other: i64,
    /// Part II expenses
    pub expenses: FarmExpenses,
    /// Line 8 cars and trucks
    #[libertas_unordered]
    /// ----
    /// Vehicle
    pub vehicles: Vec<crate::Vehicle>,
    /// Line 12 buildings, fences, drainage and other property
    /// Property of this rental you depreciate. Land is not depreciated. Section 179 is not figured for a farm rental: the property is depreciated instead, with a notice.
    #[libertas_unordered]
    /// ----
    /// Asset
    pub assets: Vec<crate::Asset>,
    /// More about this farm rental
    pub more: FarmRentalDetails,
    /// State information from this rental's 1099-MISC forms
    /// These records do not add federal income or withholding a second time.
    /// ----
    /// Information form
    pub state_documents: Vec<crate::BusinessStateDocument>,
}

/// More about this farm rental
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmRentalDetails {
    /// Original methods and special source records
    pub specials: crate::FarmSpecialsInput,
    /// Farm parcel opening/acquisition basis and retained history
    /// Save new unlinked parcels, then reopen to select their accepted identities.
    pub land_basis: Vec<crate::FarmLandBasis>,
    /// Line 4 Commodity Credit Corporation loans
    /// Loans on crops from your shares. A forfeited crop's Form 1099-A from the CCC is entered as a Form 1099-A that names the loan.
    #[libertas_default(None)]
    pub ccc: CccLoans,
    /// Line 10 soil and water conservation
    /// Terracing, ponds, erosion control and similar work you paid for under a conservation plan. The deduction is limited to 25% of your gross income from farming, including this rental's.
    #[libertas_default(None)]
    pub conservation: Conservation,
    /// Farm land sold in 2026
    /// Land only; machinery and buildings are property sold.
    #[libertas_unordered]
    /// ----
    /// Land sale
    pub land_sales: Vec<LandSale>,
    /// You made management decisions and own at least 10% (line A)
    /// For example, approving the tenant, the lease terms or repairs; your spouse's share counts toward the 10%. No allows no special $25,000 rental-loss allowance.
    #[libertas_default(true)]
    pub active_participation: bool,
    /// Losses suspended in earlier years
    pub suspended: crate::SuspendedLoss,
    /// Qualified business income treatment
    #[libertas_default(No)]
    pub qbi: crate::RentalBusiness,
    /// Group this rental with others for the qualified business income deduction
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub qbi_group: String,
    /// Employees' W-2 wages
    #[libertas_default(No)]
    pub payroll: crate::BusinessPayroll,
    /// Individual at-risk facts for this rental (Form 6198)
    /// Choose All only when all deductions are at risk and no at-risk or recapture-deduction carry remains; Limited retains the separate regular and AMT computations.
    /// [NullText]
    /// Not answered
    #[libertas_default($null)]
    pub at_risk: Option<crate::RentalAtRisk>,
}

/// What produces this farm's income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmIncomeSource {
    /// Not needed for this return
    NotNeeded,
    /// Capital; your personal services are not a material factor
    Capital,
    /// Both your personal services and capital are material factors
    Mixed {
        /// Value of your work
        value: FarmWorkValue,
    },
}

/// Value of your work
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmWorkValue {
    /// My work is worth at least 30% of this farm's net profit
    /// No amount is needed: your earned income is 30% of the net profit after half of the self-employment tax. In a year with no net profit, enter the amount instead.
    AtLeastThirtyPercent,
    /// Enter the amount
    Amount {
        /// Reasonable annual compensation for your personal services
        /// Enter the supported market value of your work, before applying the tax limit. This is not an owner draw, profit distribution or tax amount. The App applies the 30% limit when there is net profit; it does not apply that limit when there is no net profit.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        compensation: i64,
    },
}
impl FarmWorkValue {
    fn record(&self) -> BusinessIncomeSource {
        match self {
            Self::AtLeastThirtyPercent => BusinessIncomeSource::MixedThirtyPercent,
            Self::Amount { compensation } => {
                BusinessIncomeSource::Mixed { compensation: *compensation }
            }
        }
    }
}

/// Line 5 Commodity Credit Corporation loans
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum CccLoans {
    /// No CCC loans
    None,
    /// CCC marketing-assistance loans
    Loans {
        /// How you report CCC loans
        #[libertas_default(Loan)]
        #[libertas_inline_choices]
        method: crate::CccMethod,
        /// Loans and pledged crops
        #[libertas_size(min = 1)]
        #[libertas_unordered]
        /// ----
        /// Loan
        loans: Vec<CccLoan>,
        /// Federal income tax withheld from CCC payments
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        withholding: i64,
        /// Actual producer, retained production-cost and original section 77 source
        source: crate::CccSourceFactsInput,
    },
}

/// Loan
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum CccLoan {
    /// Loan received in 2026
    New {
        /// Accepted record identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        id: i64,
        /// Loan principal
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        principal: i64,
        /// What happened to the crop
        #[libertas_default(Outstanding)]
        outcome: CccOutcome,
    },
    /// Loan received before 2026
    Earlier {
        /// Accepted record identity
        #[libertas_hidden]
        #[libertas_read_only]
        #[libertas_default(0)]
        id: i64,
        /// You included this loan in income when you received it (section 77)
        #[libertas_default(false)]
        taxed: bool,
        /// Loan principal
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 1, max = 100000000000)]
        principal: i64,
        /// What happened to the crop
        #[libertas_inline_choices]
        outcome: crate::CccDisposition,
    },
}

/// What happened to the crop
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum CccOutcome {
    /// Still pledged at the end of 2026
    Outstanding,
    /// Forfeited to the CCC in 2026
    Forfeited {
        /// Surplus above the loan recognized from the CCC during 2026
        /// Additional CCC proceeds received this year for cash accounting or recognized this year under the accrual all-events rules. Exclude original loan proceeds. Zero means no current surplus income.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        surplus: i64,
        /// Required loan deficiency actually paid during 2026
        /// Use only the principal shortfall you were legally required to pay and paid this year, for a loan included in income under section 77. Zero if unpaid or released. Exclude interest, fees and redemption payments.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        deficiency: i64,
    },
    /// Repaid in 2026
    Redeemed {
        /// Principal you repaid
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        paid: i64,
        /// Proceeds when you sold the redeemed crop in 2026
        /// Leave empty if you still hold the crop.
        /// [NullText]
        /// The redeemed crop is still held
        /// [SetText]
        /// The crop sale after redemption is
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        sale: Option<i64>,
    },
}

/// CRP payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CrpItem {
    /// Payment
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Federal income tax withheld
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// You were receiving Social Security retirement or disability benefits when it was paid
    #[libertas_default(false)]
    pub benefits: bool,
}

/// Feed, seed, fertilizer and other supplies paid for but unused at the end of 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PrepaidSupplies {
    /// None, and none carried from an earlier year
    None,
    /// Supplies paid for in advance
    Supplies {
        /// Paid for in 2026 and unused at year end
        #[libertas_unordered]
        /// ----
        /// Purchase
        advances: Vec<SupplyPurchase>,
        /// Cost carried from an earlier year
        #[libertas_unordered]
        /// ----
        /// Carried cost
        carried: Vec<CarriedCost>,
        /// Prepaid supplies above half of your other farm expenses
        #[libertas_default(NotNeeded)]
        exception: SupplyException,
        /// Poultry
        poultry: PoultryRecords,
    },
}

/// Purchase
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct SupplyPurchase {
    /// Amount paid
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Actual unused-purchase terms and current refund
    pub purchase: crate::FarmSupplyPurchase,
}

/// Carried cost
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CarriedCost {
    /// Undeducted cost entering 2026
    /// From the retained supply record and earlier return, not the original cost if part was already deducted.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub balance: i64,
    /// Separate undeducted AMT cost entering 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amt_balance: i64,
    /// All of it was used in 2026
    #[libertas_default(true)]
    pub used: bool,
}

/// Prepaid supplies above half of your other farm expenses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum SupplyException {
    /// Not needed, or you do not qualify
    NotNeeded,
    /// Farm-related, and your prepaid supplies were less than half your other farm expenses in 2023–2025
    ThreeYears {
        /// How you are farm-related
        #[libertas_default(Business)]
        #[libertas_inline_choices]
        relation: crate::FarmSupplyRelation,
        /// 2023
        year_third_prior: SupplyYear,
        /// 2024
        year_second_prior: SupplyYear,
        /// 2025
        year_prior: SupplyYear,
    },
    /// Farm-related, and an extraordinary circumstance changed your operations and caused the excess
    Extraordinary {
        /// How you are farm-related
        #[libertas_default(Business)]
        #[libertas_inline_choices]
        relation: crate::FarmSupplyRelation,
    },
}

/// Farm expenses of one year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SupplyYear {
    /// Prepaid farm supplies for that year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub prepaid: i64,
    /// Other deductible farming expenses from that year's records
    /// Include depreciation and amortization; exclude prepaid supplies. Include all of the taxpayer's farms, not just this activity.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub other: i64,
}

/// Poultry
/// Poultry bought for resale, current production purchases and production costs carried from 2025. Leave the lists empty when there is none.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PoultryRecords {
    /// Poultry bought solely for resale
    #[libertas_unordered]
    /// ----
    /// Lot
    pub resale: Vec<PoultryResale>,
    /// Current production-poultry purchase and actual timing records
    pub current: Vec<crate::FarmProductionPoultryCurrent>,
    /// Production-poultry costs deferred from 2025
    #[libertas_unordered]
    /// ----
    /// Lot
    pub production: Vec<ProductionPoultry>,
}

/// Lot
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PoultryResale {
    /// Purchase or lot description
    #[libertas_size(min = 1)]
    pub label: String,
    /// How you recover the purchase cost
    #[libertas_default(Sold)]
    pub method: PoultryMethod,
    /// Year the purchase cost was paid
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = 2026)]
    pub year: u32,
    /// Original paid cost attributable to this identified portion
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Cost already deducted through 2025
    /// Current-year purchases have zero here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub deducted: i64,
    /// Sold or disposed of in 2026
    #[libertas_default(true)]
    pub disposed: bool,
}

/// How you recover the purchase cost
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum PoultryMethod {
    /// Deduct eligible purchases when paid, subject to the prepaid ceiling
    Paid,
    /// Recover purchase costs when the poultry is sold or otherwise disposed of
    Sold,
}

/// Lot
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ProductionPoultry {
    /// Purchase or lot description
    #[libertas_size(min = 1)]
    pub label: String,
    /// Cost paid in 2025 for this identified lot
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cost: i64,
    /// Cost deducted on the 2025 return
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub deducted: i64,
}

/// Line 12 soil and water conservation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum Conservation {
    /// No soil or water conservation costs
    None,
    /// Deduct them under section 175, following an NRCS or state conservation plan
    Deduct {
        /// Costs paid in 2026
        #[libertas_unordered]
        /// ----
        /// Cost
        costs: Vec<crate::ConservationCostInput>,
        /// Conservation district assessments for depreciable property
        #[libertas_unordered]
        /// ----
        /// Assessment
        assessments: Vec<crate::ConservationAssessmentInput>,
        /// Prior unused 25-percent-limited costs, by parcel and source year
        carry: Vec<crate::FarmConservationCarryInput>,
        /// Original section 175 election or approved change
        method: crate::FarmConservationMethod,
    },
    /// Add them to the land's basis
    Capitalize {
        /// Original capitalization history or actual IRS-approved change
        method: crate::FarmConservationCapitalMethod,
        /// Costs paid in 2026
        #[libertas_unordered]
        /// ----
        /// Cost
        costs: Vec<crate::ConservationCostInput>,
    },
}

/// Land sale
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct LandSale {
    /// Property use throughout your ownership
    /// Use the lease and property-use records for this sold property.
    /// [NullText]
    /// Property-use records are unavailable
    /// [SetText]
    /// The property-use records show
    #[libertas_constrained_by("^<LandSale>.allowed_farm_rental_use")]
    pub farm_rental_use: Option<crate::FarmRentalPropertyUse>,
    /// Applicable property-use choices supplied by the Farming editor
    #[libertas_hidden]
    #[libertas_read_only]
    pub allowed_farm_rental_use: Vec<i32>,

    /// Parcel
    #[libertas_size(min = 1)]
    pub label: String,
    /// Accepted sale identity; new records are assigned on Save
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    /// Accepted parcel from this farm; save new unlinked parcels, then reopen
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.basis_assets")]
    pub basis_property: i64,
    /// Date sold
    #[libertas_date_only]
    pub sold: u32,
    /// Sale price
    /// Including the buyer's note and any loan the buyer took over.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub proceeds: i64,
    /// Selling expenses
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub selling: i64,
    /// How you were paid
    #[libertas_default(InFull)]
    pub payment: crate::SalePayment,
    /// Pay the tax in four installments (Form 1062)
    #[libertas_default(No)]
    pub installments: LandInstallments,
    /// Actual transaction character; special cases need their named source owner
    #[libertas_default(Direct)]
    pub situation: crate::FarmLandSituation,
}

/// Pay the tax in four installments (Form 1062)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum LandInstallments {
    /// No
    No,
    /// Yes: you farmed it or leased it to a farmer for substantially all of the last ten years, the buyer is a qualified farmer, and a ten-year farming covenant is recorded
    Yes,
}

/// Farm income averaging (Schedule J)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmAveragingElection {
    /// Farm income you elect to average (Schedule J line 2a)
    /// All or part of your farm profit; the App limits it and shows the tax with and without averaging.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub elected: i64,
    /// Your 2023 return
    pub base_third_prior_year: BaseYear,
    /// Your 2024 return
    pub base_second_prior_year: BaseYear,
    /// Your 2025 return
    pub base_prior_year: BaseYear,
    /// Current elected components and retained base-year history
    /// Ordinary production-farm income is included automatically. Additional linked sources and capital elections require their actual attributable owner output; known ordinary tax remains calculated when that averaging benefit is unavailable.
    pub selection: crate::FarmAveragingSelectionInput,
}

/// Base year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct BaseYear {
    /// Filing status on that return
    #[libertas_inline_choices]
    pub status: crate::FederalStatus,
    /// Where the figures come from
    #[libertas_default(Return)]
    pub source: BaseYearSource,
    /// Qualified dividends (Form 1040 line 3a)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub dividends: i64,
    /// Capital gains that year
    pub capital: crate::AveragingCapital,
    /// Foreign earned income or housing exclusion (Form 2555) that year
    /// Leave it empty when that year's return had none.
    /// [NullText]
    /// That base-year return had no foreign exclusions
    /// [SetText]
    /// That base-year foreign exclusion worksheet is
    pub foreign: Option<crate::AveragingForeign>,
    /// Investment interest election (Form 4952) that year
    /// Leave it empty when that year's return made none.
    /// [NullText]
    /// That base-year return made no investment-income election
    /// [SetText]
    /// That base-year investment-income election is
    pub investment: Option<crate::AveragingInvestment>,
}

/// Where the figures come from
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum BaseYearSource {
    /// That year's Form 1040
    Return {
        /// Form 1040 line 11 adjusted gross income
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        agi: i64,
        /// Form 1040 line 14 deductions
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        deductions: i64,
        /// Form 1040 line 16 tax
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        tax: i64,
        /// Non-section-1 taxes included in line 16
        /// Enter each retained component separately, such as Form 8814, Form 4972 or education-credit recapture.
        #[libertas_unordered]
        /// ----
        /// Other tax
        other_tax: Vec<crate::AveragingOtherTax>,
        /// Negative taxable-income source records
        /// Needed when the deductions are more than the adjusted gross income; leave empty otherwise.
        /// [NullText]
        /// Base-year deductions did not exceed adjusted gross income
        /// [SetText]
        /// The negative taxable-income source is
        loss: Option<crate::AveragingLoss>,
    },
    /// A later Schedule J that refigured that year
    ScheduleJ {
        /// Year of the retained Schedule J
        // Tax structure: this retained source belongs to a year before the 2026 return year.
        #[libertas_number(min = 2023, max = 2025)]
        year: u32,
        /// Taxable income from the specified retained line
        /// Preserve a negative amount on line 11 or 15.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = -100000000000, max = 100000000000)]
        taxable: i64,
        /// Tax from the paired retained line
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        tax: i64,
    },
}

/// 2026 farm loss: carryback choice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmLossWaiver {
    /// What to do with the farming loss
    #[libertas_default(Waive)]
    pub choice: FarmLossChoice,
}

/// What to do with the farming loss
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmLossChoice {
    /// Carry the whole loss forward: you attach the carryback waiver statement to your 2026 return filed by its due date
    Waive,
    /// Carry the farming loss back to 2024 and 2025
    /// The refund for those years is claimed on Form 1045 or Form 1040-X and is not in this estimate; the 2026 return shows the whole loss carried forward, before the amount the earlier years absorb.
    Carryback,
}

// ---- Mapping onto the stored records (T2-45p). The page writes the stored
// FarmIncome, FederalBusiness, FarmAveraging and FarmNolElection records; the
// reverse rebuilds the page and is accepted only when saving it again
// reproduces the stored record (the business_item rule).

// Placed in service before this date, a 3-, 5-, 7- or 10-year farm asset
// uses 150% declining balance (question F9).
const FARM_150_BEFORE: u32 = 20180101;

// Question F9: a pre-2018 farm asset of a 3-, 5-, 7- or 10-year class uses the
// 150% declining balance method the law required.
fn farm_asset_methods(assets: &mut [crate::BusinessAsset]) {
    for asset in assets {
        if asset.placed < FARM_150_BEFORE
            && matches!(
                asset.class,
                EquipmentClass::Three
                    | EquipmentClass::Five
                    | EquipmentClass::Seven
                    | EquipmentClass::Ten
            )
        {
            asset.method = EquipmentMethod::Declining150;
        }
    }
}

// Part II lines in Schedule F order, each with its stored category.
fn expense_lines(e: &FarmExpenses) -> [(BusinessExpenseCategory, i64); 22] {
    use BusinessExpenseCategory as C;
    [
        (C::Chemicals, e.chemicals),
        (C::CustomHire, e.custom_hire),
        (C::EmployeeBenefits, e.employee_benefits),
        (C::Feed, e.feed),
        (C::Fertilizers, e.fertilizers),
        (C::Freight, e.freight),
        (C::Fuel, e.fuel),
        (C::Insurance, e.insurance),
        (C::MortgageInterest, e.mortgage_interest),
        (C::OtherInterest, e.other_interest),
        (C::Wages, e.labor),
        (C::EmployeePension, e.pension),
        (C::VehicleAndEquipmentRent, e.equipment_rent),
        (C::OtherRent, e.other_rent),
        (C::Repairs, e.repairs),
        (C::Seeds, e.seeds),
        (C::Storage, e.storage),
        (C::Supplies, e.supplies),
        (C::TaxesAndLicenses, e.taxes),
        (C::Utilities, e.utilities),
        (C::Veterinary, e.veterinary),
        (C::Meals, e.meals),
    ]
}
fn expense_fields(e: &mut FarmExpenses) -> [&mut i64; 22] {
    [
        &mut e.chemicals,
        &mut e.custom_hire,
        &mut e.employee_benefits,
        &mut e.feed,
        &mut e.fertilizers,
        &mut e.freight,
        &mut e.fuel,
        &mut e.insurance,
        &mut e.mortgage_interest,
        &mut e.other_interest,
        &mut e.labor,
        &mut e.pension,
        &mut e.equipment_rent,
        &mut e.other_rent,
        &mut e.repairs,
        &mut e.seeds,
        &mut e.storage,
        &mut e.supplies,
        &mut e.taxes,
        &mut e.utilities,
        &mut e.veterinary,
        &mut e.meals,
    ]
}
// Each nonzero line is one FederalBusinessExpense; meals take the 50%
// treatment, every other line is ordinary; line 32 rows follow.
fn expense_records(e: &FarmExpenses) -> Vec<FederalBusinessExpense> {
    expense_lines(e)
        .into_iter()
        .filter(|(_, amount)| *amount != 0)
        .map(|(category, amount)| FederalBusinessExpense {
            treatment: if category == BusinessExpenseCategory::Meals {
                BusinessExpenseKind::Meals
            } else {
                BusinessExpenseKind::Ordinary
            },
            category,
            amount,
        })
        .chain(e.other.iter().map(|other| FederalBusinessExpense {
            category: BusinessExpenseCategory::Other { label: other.description.clone() },
            amount: other.amount,
            treatment: BusinessExpenseKind::Ordinary,
        }))
        .collect()
}
fn expense_items(records: &[FederalBusinessExpense]) -> FarmExpenses {
    let mut e = FarmExpenses {
        chemicals: 0,
        custom_hire: 0,
        employee_benefits: 0,
        feed: 0,
        fertilizers: 0,
        freight: 0,
        fuel: 0,
        insurance: 0,
        mortgage_interest: 0,
        other_interest: 0,
        labor: 0,
        pension: 0,
        equipment_rent: 0,
        other_rent: 0,
        repairs: 0,
        seeds: 0,
        storage: 0,
        supplies: 0,
        taxes: 0,
        utilities: 0,
        veterinary: 0,
        meals: 0,
        other: Vec::new(),
    };
    let lines = expense_lines(&e).map(|(category, _)| category);
    for record in records {
        match lines.iter().position(|c| *c == record.category) {
            Some(index) => *expense_fields(&mut e)[index] = record.amount,
            None => {
                // Schedule C lines the farm page has no field for fall here
                // too; the round-trip check then keeps the page closed.
                let label = match &record.category {
                    BusinessExpenseCategory::Other { label } => label.clone(),
                    _ => String::new(),
                };
                e.other.push(crate::OtherExpense { description: label, amount: record.amount });
            }
        }
    }
    e
}

fn receipts_total(method: &FarmMethod) -> i128 {
    match method {
        FarmMethod::Cash { resale, raised, program, custom_hire, other, .. } => {
            i128::from(*resale)
                + i128::from(*raised)
                + i128::from(*program)
                + i128::from(*custom_hire)
                + i128::from(*other)
        }
        FarmMethod::Accrual { sales, program, custom_hire, other, .. } => {
            i128::from(*sales)
                + i128::from(*program)
                + i128::from(*custom_hire)
                + i128::from(*other)
        }
    }
}
// Part I (cash: one FarmReceipt per nonzero line, in form order, the
// section 126 cost-sharing receipt from the More fold) or Part III.
fn accounting_record(method: &FarmMethod, cost_sharing: i64) -> FarmAccounting {
    match method {
        FarmMethod::Cash { resale, resale_cost, raised, program, custom_hire, other } => {
            let mut receipts = Vec::new();
            if *resale != 0 || *resale_cost != 0 {
                receipts.push(FarmReceipt::Resale { amount: *resale, cost: *resale_cost });
            }
            for (amount, receipt) in [
                (*raised, FarmReceipt::Raised { amount: *raised }),
                (*program, FarmReceipt::Program { amount: *program }),
                (cost_sharing, FarmReceipt::CostSharing { amount: cost_sharing }),
                (*custom_hire, FarmReceipt::CustomHire { amount: *custom_hire }),
                (*other, FarmReceipt::Other { amount: *other }),
            ] {
                if amount != 0 {
                    receipts.push(receipt);
                }
            }
            FarmAccounting::Cash { receipts }
        }
        FarmMethod::Accrual { sales, program, custom_hire, other, opening, purchases, closing } => {
            FarmAccounting::Accrual {
                sales: *sales,
                opening: *opening,
                closing: *closing,
                purchases: *purchases,
                custom_hire: *custom_hire,
                program: *program,
                other: *other,
            }
        }
    }
}
// The reverse, with the cost-sharing amount; a second receipt of one kind
// adds to the first and the round-trip check keeps the page closed.
fn accounting_item(accounting: &FarmAccounting) -> (FarmMethod, i64) {
    match accounting {
        FarmAccounting::Cash { receipts } => {
            let (mut resale, mut resale_cost, mut raised, mut program) = (0, 0, 0, 0);
            let (mut cost_sharing, mut custom_hire, mut other) = (0, 0, 0);
            for receipt in receipts {
                match receipt {
                    FarmReceipt::Resale { amount, cost } => {
                        resale += amount;
                        resale_cost += cost;
                    }
                    FarmReceipt::Raised { amount } => raised += amount,
                    FarmReceipt::Program { amount } => program += amount,
                    FarmReceipt::CostSharing { amount } => cost_sharing += amount,
                    FarmReceipt::CustomHire { amount } => custom_hire += amount,
                    FarmReceipt::Other { amount } => other += amount,
                }
            }
            (
                FarmMethod::Cash { resale, resale_cost, raised, program, custom_hire, other },
                cost_sharing,
            )
        }
        FarmAccounting::Accrual {
            sales,
            opening,
            closing,
            purchases,
            custom_hire,
            program,
            other,
        } => (
            FarmMethod::Accrual {
                sales: *sales,
                program: *program,
                custom_hire: *custom_hire,
                other: *other,
                opening: *opening,
                purchases: *purchases,
                closing: *closing,
            },
            0,
        ),
    }
}

fn insurance_record(value: &CropInsurance) -> crate::FarmInsuranceRecords {
    match value {
        CropInsurance::None => crate::FarmInsuranceRecords::None,
        CropInsurance::Payments { physical, price_only, from_prior_year, withholding } => {
            let mut current: Vec<CropPayment> = physical
                .iter()
                .map(|p| CropPayment::Physical {
                    crop: p.crop.clone(),
                    amount: p.amount,
                    year: p.year,
                })
                .collect();
            if *price_only != 0 {
                current.push(CropPayment::Other { amount: *price_only });
            }
            crate::FarmInsuranceRecords::Payments {
                current,
                previous_deferred: *from_prior_year,
                withholding: *withholding,
            }
        }
    }
}
fn insurance_item(record: &crate::FarmInsuranceRecords) -> CropInsurance {
    match record {
        crate::FarmInsuranceRecords::None => CropInsurance::None,
        crate::FarmInsuranceRecords::Payments {
            current, previous_deferred, withholding, ..
        } => CropInsurance::Payments {
            physical: current
                .iter()
                .filter_map(|p| match p {
                    CropPayment::Physical { crop, amount, year } => {
                        Some(CropLoss { crop: crop.clone(), amount: *amount, year: *year })
                    }
                    CropPayment::Other { .. } => None,
                })
                .collect(),
            price_only: current
                .iter()
                .map(|p| match p {
                    CropPayment::Other { amount } => *amount,
                    CropPayment::Physical { .. } => 0,
                })
                .sum(),

            from_prior_year: *previous_deferred,
            withholding: *withholding,
        },
    }
}

fn ccc_record(value: &CccLoans) -> Option<FarmCcc> {
    let CccLoans::Loans { method, loans, withholding, source } = value else {
        return None;
    };
    Some(FarmCcc {
        source: source.into(),
        method: method.clone(),
        lots: loans
            .iter()
            .map(|loan| match loan {
                CccLoan::New { id, principal, outcome } => CccLot {
                    id: *id,
                    origin: CccOrigin::Current,
                    principal: *principal,
                    disposition: match outcome {
                        CccOutcome::Outstanding => CccDisposition::Outstanding,
                        CccOutcome::Forfeited { surplus, deficiency } => {
                            CccDisposition::Forfeited {
                                year: 2026,
                                surplus: *surplus,
                                deficiency: *deficiency,
                            }
                        }
                        CccOutcome::Redeemed { paid, sale } => {
                            CccDisposition::Redeemed { year: 2026, paid: *paid, sale: *sale }
                        }
                    },
                },
                CccLoan::Earlier { id, taxed, principal, outcome } => CccLot {
                    id: *id,
                    origin: if *taxed {
                        CccOrigin::PriorIncluded
                    } else {
                        CccOrigin::PriorExcluded
                    },
                    principal: *principal,
                    disposition: outcome.clone(),
                },
            })
            .collect(),
        withholding: *withholding,
    })
}
fn ccc_item(record: &Option<FarmCcc>) -> CccLoans {
    let Some(ccc) = record else {
        return CccLoans::None;
    };
    CccLoans::Loans {
        method: ccc.method.clone(),
        loans: ccc
            .lots
            .iter()
            .map(|lot| match lot.origin {
                // A current-year loan settles in 2026; any other year keeps
                // the page closed through the round-trip check.
                CccOrigin::Current => CccLoan::New {
                    id: lot.id,
                    principal: lot.principal,
                    outcome: match lot.disposition {
                        CccDisposition::Outstanding => CccOutcome::Outstanding,
                        CccDisposition::Forfeited { surplus, deficiency, .. } => {
                            CccOutcome::Forfeited { surplus, deficiency }
                        }
                        CccDisposition::Redeemed { paid, sale, .. } => {
                            CccOutcome::Redeemed { paid, sale }
                        }
                    },
                },
                CccOrigin::PriorExcluded | CccOrigin::PriorIncluded => CccLoan::Earlier {
                    id: lot.id,
                    taxed: lot.origin == CccOrigin::PriorIncluded,
                    principal: lot.principal,
                    outcome: lot.disposition.clone(),
                },
            })
            .collect(),
        withholding: ccc.withholding,
        source: (&ccc.source).into(),
    }
}

fn supplies_record(value: &PrepaidSupplies) -> Option<FarmSupplies> {
    let PrepaidSupplies::Supplies { advances, carried, exception, poultry } = value else {
        return None;
    };
    let PoultryRecords { resale, production, current } = poultry;
    let year =
        |year: u32, v: &SupplyYear| FarmSupplyYear { year, prepaid: v.prepaid, other: v.other };
    Some(FarmSupplies {
        production_current: current.clone(),
        advances: advances
            .iter()
            .map(|a| FarmSupplyAdvance { amount: a.amount, purchase: a.purchase.clone() })
            .collect(),
        retained: carried
            .iter()
            .map(|c| FarmSupplyCarry {
                balance: c.balance,
                amt_balance: c.amt_balance,
                consumed: c.used,
            })
            .collect(),
        poultry: resale
            .iter()
            .map(|p| FarmPoultryResale {
                label: p.label.clone(),
                method: match p.method {
                    PoultryMethod::Paid => FarmPoultryMethod::Paid,
                    PoultryMethod::Sold => FarmPoultryMethod::Sold,
                },
                year: p.year,
                cost: p.cost,
                deducted: p.deducted,
                disposed: p.disposed,
            })
            .collect(),
        production_carry: production
            .iter()
            .map(|p| FarmProductionPoultryCarry {
                label: p.label.clone(),
                cost: p.cost,
                deducted: p.deducted,
            })
            .collect(),
        exception: match exception {
            SupplyException::NotNeeded => FarmSupplyException::None,
            SupplyException::ThreeYears {
                relation,
                year_third_prior,
                year_second_prior,
                year_prior,
            } => FarmSupplyException::History {
                relation: relation.clone(),
                years: alloc::vec![
                    year(2023, year_third_prior),
                    year(2024, year_second_prior),
                    year(2025, year_prior),
                ],
            },
            SupplyException::Extraordinary { relation } => {
                FarmSupplyException::Extraordinary { relation: relation.clone() }
            }
        },
    })
}
fn supplies_item(record: &Option<FarmSupplies>) -> Option<PrepaidSupplies> {
    let Some(s) = record else {
        return Some(PrepaidSupplies::None);
    };
    let year = |year: u32| -> Option<SupplyYear> {
        let mut found = match &s.exception {
            FarmSupplyException::History { years, .. } => years.iter().filter(|y| y.year == year),
            _ => return None,
        };
        let first = found.next()?;
        Some(SupplyYear { prepaid: first.prepaid, other: first.other })
    };
    let resale: Vec<PoultryResale> = s
        .poultry
        .iter()
        .map(|p| PoultryResale {
            label: p.label.clone(),
            method: match p.method {
                FarmPoultryMethod::Paid => PoultryMethod::Paid,
                FarmPoultryMethod::Sold => PoultryMethod::Sold,
            },
            year: p.year,
            cost: p.cost,
            deducted: p.deducted,
            disposed: p.disposed,
        })
        .collect();
    let production: Vec<ProductionPoultry> = s
        .production_carry
        .iter()
        .map(|p| ProductionPoultry { label: p.label.clone(), cost: p.cost, deducted: p.deducted })
        .collect();
    Some(PrepaidSupplies::Supplies {
        advances: s
            .advances
            .iter()
            .map(|a| SupplyPurchase { amount: a.amount, purchase: a.purchase.clone() })
            .collect(),
        carried: s
            .retained
            .iter()
            .map(|c| CarriedCost {
                balance: c.balance,
                amt_balance: c.amt_balance,
                used: c.consumed,
            })
            .collect(),
        exception: match &s.exception {
            FarmSupplyException::None => SupplyException::NotNeeded,
            FarmSupplyException::History { relation, .. } => SupplyException::ThreeYears {
                relation: relation.clone(),
                year_third_prior: year(2023)?,
                year_second_prior: year(2024)?,
                year_prior: year(2025)?,
            },
            FarmSupplyException::Extraordinary { relation, .. } => {
                SupplyException::Extraordinary { relation: relation.clone() }
            }
        },
        poultry: PoultryRecords { resale, production, current: s.production_current.clone() },
    })
}

fn conservation_record(value: &Conservation) -> Option<FarmConservation> {
    match value {
        Conservation::None => None,
        Conservation::Deduct { costs, assessments, carry, method } => {
            Some(FarmConservation::Deduct {
                costs: costs.iter().map(Into::into).collect(),
                assessments: assessments.iter().map(Into::into).collect(),
                carry: carry.iter().map(Into::into).collect(),
                method: method.clone(),
            })
        }
        Conservation::Capitalize { costs, method } => Some(FarmConservation::Capitalize {
            method: method.clone(),
            costs: costs.iter().map(Into::into).collect(),
        }),
    }
}
fn conservation_item(record: &Option<FarmConservation>) -> Conservation {
    match record {
        None => Conservation::None,
        Some(FarmConservation::Deduct { costs, assessments, carry, method }) => {
            Conservation::Deduct {
                costs: costs.iter().map(Into::into).collect(),
                assessments: assessments.iter().map(Into::into).collect(),
                carry: carry.iter().map(Into::into).collect(),
                method: method.clone(),
            }
        }
        Some(FarmConservation::Capitalize { costs, method }) => Conservation::Capitalize {
            method: method.clone(),
            costs: costs.iter().map(Into::into).collect(),
        },
    }
}

fn land_record(sale: &LandSale) -> FarmLandSale {
    FarmLandSale {
        farm_rental_use: sale.farm_rental_use.clone(),
        label: sale.label.clone(),
        id: sale.id,
        basis_property: sale.basis_property,
        sold: sale.sold,
        proceeds: sale.proceeds,
        selling: sale.selling,
        deferral: match sale.installments {
            LandInstallments::No => false,
            LandInstallments::Yes => true,
        },
        situation: sale.situation.clone(),
        payment: sale.payment.clone(),
    }
}
fn land_item(record: &FarmLandSale) -> LandSale {
    LandSale {
        farm_rental_use: record.farm_rental_use.clone(),
        allowed_farm_rental_use: Vec::new(),
        label: record.label.clone(),
        id: record.id,
        basis_property: record.basis_property,
        sold: record.sold,
        proceeds: record.proceeds,
        selling: record.selling,
        installments: if record.deferral { LandInstallments::Yes } else { LandInstallments::No },
        situation: record.situation.clone(),
        payment: record.payment.clone(),
    }
}

// The page's farm onto FederalBusiness. Owner health insurance and the
// retirement plan belong to the self-employed benefits page and are kept
// from the accepted record. Every other answer, including the always-present
// "More about this farm" record, is the person's own.
pub(crate) fn farm_record(f: &Farm, kept: Option<&FederalBusiness>) -> FederalBusiness {
    let (vehicles, mut assets) = vehicle_and_asset_records(&f.vehicles, &f.assets);
    farm_asset_methods(&mut assets);
    let more = &f.more;
    let farm = FarmIncome {
        related: f.related.forms.iter().map(|form| form.key).collect(),
        specials: (&more.specials).into(),
        land_basis: more.land_basis.clone(),
        header: Some((&more.header).into()),
        accounting: accounting_record(&f.accounting, more.cost_sharing),
        insurance: insurance_record(&f.insurance),
        insurance_papers: Default::default(),
        ccc: ccc_record(&more.ccc),
        crp: more
            .crp
            .iter()
            .map(|p| CrpPayment {
                amount: p.amount,
                benefits: p.benefits,
                withholding: p.withholding,
            })
            .collect(),
        supplies: supplies_record(&more.supplies),
        conservation: conservation_record(&more.conservation),
        land_sales: more.land_sales.iter().map(land_record).collect(),
    };
    FederalBusiness {
        improvement_buildings: f.improvement_buildings.clone(),
        interest_limit_election: f.interest_limit_election.clone().map(Into::into),
        related: Vec::new(),
        principal_business: Default::default(),
        business_code: None,
        ein: Default::default(),
        address: Default::default(),
        started_current_year: false,
        information_returns: crate::InformationReturnFiling::NoneRequired,
        other_income: Vec::new(),
        de_minimis: None,
        leased_vehicles: Vec::new(),
        depletion: None,
        ministerial: false,
        notary_profit: 0,
        notary_gross: None,
        source: BusinessSource::UnitedStates,
        income_source: match &more.income_source {
            FarmIncomeSource::NotNeeded => BusinessIncomeSource::Services,
            FarmIncomeSource::Capital => BusinessIncomeSource::Capital,
            FarmIncomeSource::Mixed { value } => value.record(),
        },
        activity: BusinessActivity::Farming { data: farm },
        inventory: BusinessInventory::None,
        startup: more.startup.clone(),
        method_changes: more.method_changes.clone(),
        intangibles: Vec::new(),
        timber: Vec::new(),
        other_amortization: Vec::new(),
        health_insurance: kept
            .map_or(BusinessHealthInsurance::None, |k| k.health_insurance.clone()),
        retirement: kept.map_or(BusinessRetirement::None, |k| k.retirement.clone()),
        vehicles,
        assets,
        expenses: expense_records(&f.expenses),
        tips: more.tips.clone(),
        overtime: None,
        fuel_tax_credit: more.fuel_tax_credit.clone(),
        cash_fuel_tax_recovery: more.cash_fuel_tax_recovery.clone(),
        employer_tip_fica_credit: None,
        work_opportunity_credit: None,
        retirement_plan_credit: None,
        small_employer_health_credit: None,
        paid_leave_credit: None,
        material_participation: more.material_participation.clone(),
        specified_service: false,
        at_risk: crate::RentalAtRisk::from_full_risk(more.at_risk),
        payroll: more.payroll.clone(),
        special_treatment: false,
        qbi_aggregation_group: crate::common::optional_text(&more.qbi_group),
        withholding: more.withholding,
        commercial_fishing: false,
        sale: None,
    }
}

// The reverse of farm_record; None for a stored farm the page's answers
// would not reproduce. `optional_se` is the owner's stored farm optional
// method. A stored Services source (what Not needed writes) reopens as Not
// needed.
pub(crate) fn farm_item(
    entry_label: &str,
    data: &FederalBusiness,
    optional_se: bool,
) -> Option<Farm> {
    let BusinessActivity::Farming { data: farm } = &data.activity else {
        return None;
    };
    // Farm pages derive the statutory method rather than editing it. Refuse
    // stored methods that this page cannot round-trip faithfully.
    let mut derived_assets = data.assets.clone();
    farm_asset_methods(&mut derived_assets);
    if derived_assets
        .iter()
        .zip(&data.assets)
        .any(|(derived, stored)| derived.method != stored.method)
    {
        return None;
    }
    let (vehicles, assets) = vehicle_and_asset_items(&data.vehicles, &data.assets)?;
    let (accounting, cost_sharing) = accounting_item(&farm.accounting);
    let item = Farm {
        improvement_buildings: data.improvement_buildings.clone(),
        interest_limit_election: data.interest_limit_election.clone().map(Into::into),
        allowed_interest_limit_election: crate::interest_limit::choices(
            data.interest_limit_election.is_some(),
        ),
        related: crate::RelatedForms::page::<Farm>(&farm.related, &[], &Default::default()),
        state_documents: Default::default(),
        name: entry_label.into(),
        accounting,
        insurance: insurance_item(&farm.insurance),
        expenses: expense_items(&data.expenses),
        vehicles,
        assets,
        more: FarmDetails {
            specials: (&farm.specials).into(),
            land_basis: farm.land_basis.clone(),
            header: farm.header.as_ref()?.into(),
            ccc: ccc_item(&farm.ccc),
            crp: farm
                .crp
                .iter()
                .map(|p| CrpItem {
                    amount: p.amount,
                    withholding: p.withholding,
                    benefits: p.benefits,
                })
                .collect(),
            cost_sharing,
            supplies: supplies_item(&farm.supplies)?,
            conservation: conservation_item(&farm.conservation),
            land_sales: farm.land_sales.iter().map(land_item).collect(),
            startup: data.startup.clone(),
            method_changes: data.method_changes.clone(),
            payroll: data.payroll.clone(),
            withholding: data.withholding,
            optional_se,
            material_participation: data.material_participation.clone(),
            at_risk: data.all_at_risk(),
            qbi_group: data.qbi_aggregation_group.clone().unwrap_or_default(),
            income_source: match &data.income_source {
                BusinessIncomeSource::Services => FarmIncomeSource::NotNeeded,
                BusinessIncomeSource::Capital => FarmIncomeSource::Capital,
                BusinessIncomeSource::Mixed { compensation } => FarmIncomeSource::Mixed {
                    value: FarmWorkValue::Amount { compensation: *compensation },
                },
                BusinessIncomeSource::MixedThirtyPercent => {
                    FarmIncomeSource::Mixed { value: FarmWorkValue::AtLeastThirtyPercent }
                }
            },
            fuel_tax_credit: data.fuel_tax_credit.clone(),
            cash_fuel_tax_recovery: data.cash_fuel_tax_recovery.clone(),
            tips: data.tips.clone(),
        },
    };
    let mut rebuilt = farm_record(&item, Some(data));
    if let BusinessActivity::Farming { data: rebuilt } = &mut rebuilt.activity {
        rebuilt.insurance_papers = farm.insurance_papers.clone();
    }
    (rebuilt == *data).then_some(item)
}

pub(crate) fn farm_label(f: &Farm) -> String {
    f.name.clone()
}
pub(crate) fn farm_optional_se(f: &Farm) -> bool {
    f.more.optional_se
}
// The income-source choice as its union index, for the page's allowed set.
pub(crate) fn farm_income_source_index(f: &Farm) -> i32 {
    match &f.more.income_source {
        FarmIncomeSource::NotNeeded => FarmIncomeSource::NOT_NEEDED,
        FarmIncomeSource::Capital => 1,
        FarmIncomeSource::Mixed { .. } => 2,
    }
}
// A farm reopened on a return that does not read the source shows Not
// needed, the only choice offered; the stored answer is not read.
pub(crate) fn farm_source_not_needed(f: &mut Farm) {
    f.more.income_source = FarmIncomeSource::NotNeeded;
}
impl FarmIncomeSource {
    const NOT_NEEDED: i32 = 0;
    /// The choices the page offers: Capital and Both when the return reads
    /// the source, otherwise "Not needed" alone, which the client selects
    /// itself.
    pub fn allowed(needed: bool) -> Vec<i32> {
        if needed { alloc::vec![1, 2] } else { alloc::vec![Self::NOT_NEEDED] }
    }
}

// The CCC loan, conservation assessment and land-sale rules a farm and a farm
// rental share, checked again on save.
fn farm_records_issue(
    ccc: &CccLoans,
    conservation: &Conservation,
    land_sales: &[LandSale],
) -> Option<ValidationIssue> {
    if let CccLoans::Loans { loans, .. } = ccc {
        for (i, loan) in loans.iter().enumerate() {
            let (principal, paid, deficiency) = match loan {
                CccLoan::New { principal, outcome, .. } => match outcome {
                    CccOutcome::Redeemed { paid, .. } => (*principal, *paid, 0),
                    CccOutcome::Forfeited { deficiency, .. } => (*principal, 0, *deficiency),
                    CccOutcome::Outstanding => (*principal, 0, 0),
                },
                CccLoan::Earlier { principal, outcome, .. } => match outcome {
                    CccDisposition::Redeemed { paid, .. } => (*principal, *paid, 0),
                    CccDisposition::Forfeited { deficiency, .. } => (*principal, 0, *deficiency),
                    CccDisposition::Outstanding => (*principal, 0, 0),
                },
            };
            if paid > principal {
                return Some(
                    ValidationIssue::new("CCC_REPAID_ABOVE_PRINCIPAL")
                        .at_field("paid")
                        .in_field("outcome")
                        .in_list("loans", i)
                        .in_field("ccc"),
                );
            }
            if deficiency > principal {
                return Some(
                    ValidationIssue::new("CCC_DEFICIENCY_ABOVE_PRINCIPAL")
                        .at_field("deficiency")
                        .in_field("outcome")
                        .in_list("loans", i)
                        .in_field("ccc"),
                );
            }
        }
    }
    if let Conservation::Deduct { assessments, .. } = conservation {
        for (i, a) in assessments.iter().enumerate() {
            if let Some(issue) = crate::ConservationAssessment::from(a).relationships_issue() {
                return Some(issue.in_list("assessments", i).in_field("conservation"));
            }
        }
    }
    for (i, sale) in land_sales.iter().enumerate() {
        if let Some(issue) = sale.payment.issue(sale.proceeds) {
            return Some(issue.in_field("payment").in_list("land_sales", i));
        }
        if sale.sold / 10000 != 2026 {
            return Some(
                ValidationIssue::new("FARM_LAND_SOLD_OUTSIDE_CURRENT_YEAR")
                    .at_field("sold")
                    .in_list("land_sales", i),
            );
        }
    }
    None
}
// The stored FarmIncome projection preserves these exact collections. Physical
// crop rows precede the optional price-only row; only Physical can fail crop
// source checks, so the retained index maps directly to the editable list.
fn farm_source_page_issue(issue: ValidationIssue) -> ValidationIssue {
    issue
        .map_field_prefix(&["insurance", "current"], &["insurance", "physical"])
        .map_field_prefix(&["ccc"], &["more", "ccc"])
        .map_field_prefix(&["supplies"], &["more", "supplies"])
        .map_field_prefix(&["specials"], &["more", "specials"])
        .map_field_prefix(&["conservation"], &["more", "conservation"])
        .map_field_prefix(&["land_basis"], &["more", "land_basis"])
        .map_field_prefix(&["land_sales"], &["more", "land_sales"])
}
pub(crate) fn farm_issue(f: &Farm) -> Option<ValidationIssue> {
    if let Some(issue) = f.more.payroll.issue() {
        return Some(issue.in_field("payroll").in_field("more"));
    }
    if let Some((i, _)) = f
        .vehicles
        .iter()
        .enumerate()
        .find(|(_, v)| matches!(v.method, crate::VehicleMethod::Leased { .. }))
    {
        return Some(
            ValidationIssue::new("FARM_LEASE_ACTUAL_NOT_SUPPORTED")
                .at_field("method")
                .in_list("vehicles", i),
        );
    }
    if let BusinessActivity::Farming { data } = &farm_record(f, None).activity
        && let Some(issue) = data.specials_issue()
    {
        return Some(farm_source_page_issue(issue));
    }
    if let Some(issue) = crate::FarmHeader::from(&f.more.header).issue() {
        return Some(issue.in_field("header").in_field("more"));
    }
    let FarmDetails { cost_sharing, withholding, ccc, conservation, land_sales, .. } = &f.more;
    if matches!(f.accounting, FarmMethod::Accrual { .. }) && *cost_sharing != 0 {
        return Some(
            ValidationIssue::new("FARM_COST_SHARING_NEEDS_CASH")
                .at_field("cost_sharing")
                .in_field("more"),
        );
    }
    if let CropInsurance::Payments { physical, price_only, from_prior_year, withholding, .. } =
        &f.insurance
        && i128::from(*withholding)
            > physical.iter().map(|p| i128::from(p.amount)).sum::<i128>()
                + i128::from(*price_only)
                + i128::from(*from_prior_year)
    {
        return Some(
            ValidationIssue::new("CROP_WITHHOLDING_ABOVE_PAYMENTS")
                .at_field("withholding")
                .in_field("insurance"),
        );
    }
    if i128::from(*withholding) > receipts_total(&f.accounting) {
        return Some(
            ValidationIssue::new("FARM_WITHHOLDING_ABOVE_RECEIPTS")
                .at_field("withholding")
                .in_field("more"),
        );
    }
    if let Some(issue) = farm_records_issue(ccc, conservation, land_sales) {
        return Some(issue.in_field("more"));
    }
    if let Some(issue) = crate::method_change::list_issue(&f.more.method_changes) {
        return Some(issue.in_field("more"));
    }
    shared_business_issue(&f.vehicles, &f.assets)
}

fn base_year_record(year: u32, b: &BaseYear) -> AveragingBaseYear {
    AveragingBaseYear {
        year,
        status: b.status,
        source: match &b.source {
            BaseYearSource::Return { agi, deductions, tax, other_tax, loss } => {
                AveragingSource::Return {
                    agi: *agi,
                    deductions: *deductions,
                    tax: *tax,
                    other_tax: other_tax.clone(),
                    loss: loss.clone(),
                }
            }
            BaseYearSource::ScheduleJ { year, taxable, tax } => {
                AveragingSource::ScheduleJ { year: *year, taxable: *taxable, tax: *tax }
            }
        },
        dividends: b.dividends,
        capital: b.capital.clone(),
        foreign: b.foreign.clone(),
        investment: b.investment.clone(),
    }
}
fn base_year_item(record: &AveragingBaseYear) -> BaseYear {
    BaseYear {
        status: record.status,
        source: match &record.source {
            AveragingSource::Return { agi, deductions, tax, other_tax, loss } => {
                BaseYearSource::Return {
                    agi: *agi,
                    deductions: *deductions,
                    tax: *tax,
                    other_tax: other_tax.clone(),
                    loss: loss.clone(),
                }
            }
            AveragingSource::ScheduleJ { year, taxable, tax } => {
                BaseYearSource::ScheduleJ { year: *year, taxable: *taxable, tax: *tax }
            }
        },
        dividends: record.dividends,
        capital: record.capital.clone(),
        foreign: record.foreign.clone(),
        investment: record.investment.clone(),
    }
}
// Schedule J onto FederalScreening.farm_averaging: the three fixed base
// years replace the stored exactly-three list.
pub(crate) fn averaging_record(v: &FarmAveragingElection) -> FarmAveraging {
    FarmAveraging {
        elected: v.elected,
        years: alloc::vec![
            base_year_record(2023, &v.base_third_prior_year),
            base_year_record(2024, &v.base_second_prior_year),
            base_year_record(2025, &v.base_prior_year),
        ],
        selection: (&v.selection).into(),
    }
}
pub(crate) fn averaging_item(record: &FarmAveraging) -> Option<FarmAveragingElection> {
    let year = |year: u32| record.years.iter().find(|y| y.year == year).map(base_year_item);
    let item = FarmAveragingElection {
        elected: record.elected,
        base_third_prior_year: year(2023)?,
        base_second_prior_year: year(2024)?,
        base_prior_year: year(2025)?,
        selection: (&record.selection).into(),
    };
    (averaging_record(&item) == *record).then_some(item)
}
pub(crate) fn averaging_issue(v: &FarmAveragingElection) -> Option<ValidationIssue> {
    if let Some(issue) = averaging_record(v).selection.issue(v.elected) {
        return Some(issue.in_field("selection"));
    }
    for (field, b) in [
        ("base_third_prior_year", &v.base_third_prior_year),
        ("base_second_prior_year", &v.base_second_prior_year),
        ("base_prior_year", &v.base_prior_year),
    ] {
        if matches!(b.source,BaseYearSource::Return { agi,deductions,loss:None,.. } if agi < deductions)
        {
            return Some(
                ValidationIssue::new("AVERAGING_NEGATIVE_BASE_NEEDS_LOSS")
                    .at_field("loss")
                    .in_field("source")
                    .in_field(field),
            );
        }
    }
    for (field, b, base) in [
        ("base_second_prior_year", &v.base_second_prior_year, 2024),
        ("base_prior_year", &v.base_prior_year, 2025),
    ] {
        if matches!(b.source,BaseYearSource::ScheduleJ { year,.. } if year < base) {
            return Some(
                ValidationIssue::new("AVERAGING_SOURCE_BEFORE_BASE_YEAR")
                    .at_field("year")
                    .in_field("source")
                    .in_field(field),
            );
        }
    }
    None
}
// The farm loss choice onto FederalScreening.farm_nol (question F8).
pub(crate) fn farm_loss_record(v: &FarmLossWaiver) -> FarmNolElection {
    match v.choice {
        FarmLossChoice::Waive => FarmNolElection::Waive,
        FarmLossChoice::Carryback => FarmNolElection::Carryback,
    }
}
pub(crate) fn farm_loss_item(record: &FarmNolElection) -> Option<FarmLossWaiver> {
    Some(FarmLossWaiver {
        choice: match record {
            FarmNolElection::Waive => FarmLossChoice::Waive,
            FarmNolElection::Carryback => FarmLossChoice::Carryback,
        },
    })
}

// ---- Farm rental (Form 4835). The page writes the stored FederalFarmRental
// the way the Farm page writes a farm: the cash FarmIncome record carries the
// share receipts, crop insurance, CCC loans, conservation and land sales; the
// reverse rebuilds the page and is accepted only when saving it again
// reproduces the stored record.

pub(crate) fn farm_rental_record(f: &FarmRental) -> crate::FederalFarmRental {
    let (vehicles, mut assets) = vehicle_and_asset_records(&f.vehicles, &f.assets);
    farm_asset_methods(&mut assets);
    let more = &f.more;
    let receipts = [
        (f.production, FarmReceipt::Raised { amount: f.production }),
        (f.program, FarmReceipt::Program { amount: f.program }),
        (f.other, FarmReceipt::Other { amount: f.other }),
    ]
    .into_iter()
    .filter(|(amount, _)| *amount != 0)
    .map(|(_, receipt)| receipt)
    .collect();
    crate::FederalFarmRental {
        improvement_buildings: f.improvement_buildings.clone(),
        interest_limit_election: f.interest_limit_election.clone().map(Into::into),
        farm: FarmIncome {
            related: f.related.forms.iter().map(|form| form.key).collect(),
            specials: (&more.specials).into(),
            land_basis: more.land_basis.clone(),
            header: None,
            accounting: FarmAccounting::Cash { receipts },
            insurance: insurance_record(&f.insurance),
            insurance_papers: Default::default(),
            ccc: ccc_record(&more.ccc),
            crp: Vec::new(),
            supplies: None,
            conservation: conservation_record(&more.conservation),
            land_sales: more.land_sales.iter().map(land_record).collect(),
        },
        expenses: expense_records(&f.expenses),
        vehicles,
        assets,
        active_participation: more.active_participation,
        at_risk: more.at_risk.clone(),
        history: crate::interview::rental_history(&more.suspended),
        qbi_business: more.qbi.clone(),
        qbi_aggregation_group: crate::common::optional_text(&more.qbi_group),
        payroll: more.payroll.clone(),
    }
}

// The reverse of farm_rental_record; None for a stored rental the page's
// answers would not reproduce.
pub(crate) fn farm_rental_item(
    entry_label: &str,
    data: &crate::FederalFarmRental,
) -> Option<FarmRental> {
    let farm = &data.farm;
    let FarmAccounting::Cash { receipts } = &farm.accounting else {
        return None;
    };
    let (mut production, mut program, mut other) = (0, 0, 0);
    for receipt in receipts {
        match receipt {
            FarmReceipt::Raised { amount } => production += amount,
            FarmReceipt::Program { amount } => program += amount,
            FarmReceipt::Other { amount } => other += amount,
            _ => return None,
        }
    }
    let (vehicles, assets) = vehicle_and_asset_items(&data.vehicles, &data.assets)?;
    let item = FarmRental {
        improvement_buildings: data.improvement_buildings.clone(),
        interest_limit_election: data.interest_limit_election.clone().map(Into::into),
        allowed_interest_limit_election: crate::interest_limit::choices(
            data.interest_limit_election.is_some(),
        ),
        related: crate::RelatedForms::page::<FarmRental>(&farm.related, &[], &Default::default()),
        state_documents: Default::default(),
        name: entry_label.into(),
        production,
        program,
        insurance: insurance_item(&farm.insurance),
        other,
        expenses: expense_items(&data.expenses),
        vehicles,
        assets,
        more: FarmRentalDetails {
            specials: (&farm.specials).into(),
            land_basis: farm.land_basis.clone(),
            ccc: ccc_item(&farm.ccc),
            conservation: conservation_item(&farm.conservation),
            land_sales: farm.land_sales.iter().map(land_item).collect(),
            active_participation: data.active_participation,
            suspended: crate::SuspendedLoss {
                regular: data.history.regular,
                amt: data.history.amt,
                active_in_loss_year: data.history.active_in_loss_year,
            },
            qbi: data.qbi_business.clone(),
            qbi_group: data.qbi_aggregation_group.clone().unwrap_or_default(),
            payroll: data.payroll.clone(),
            at_risk: data.at_risk.clone(),
        },
    };
    let mut rebuilt = farm_rental_record(&item);
    rebuilt.farm.insurance_papers = data.farm.insurance_papers.clone();
    (rebuilt == *data).then_some(item)
}

pub(crate) fn farm_rental_label(f: &FarmRental) -> String {
    f.name.clone()
}

// The rental's page rules checked again on save: the shared farm rules that
// apply to its records, then the rules the client cannot express.
pub(crate) fn farm_rental_issue(f: &FarmRental) -> Option<ValidationIssue> {
    if let Some(issue) = f.more.payroll.issue() {
        return Some(issue.in_field("payroll").in_field("more"));
    }
    if let Some(issue) = farm_rental_record(f).farm.specials_issue() {
        return Some(farm_source_page_issue(issue));
    }
    if let Some(issue) = crate::activity_risk::farm_rental_source_issue(
        &f.more.at_risk,
        &crate::interview::rental_history(&f.more.suspended),
    ) {
        return Some(issue.in_field("more"));
    }
    let FarmRentalDetails { ccc, conservation, land_sales, .. } = &f.more;
    if let CropInsurance::Payments { physical, price_only, from_prior_year, withholding, .. } =
        &f.insurance
        && i128::from(*withholding)
            > physical.iter().map(|p| i128::from(p.amount)).sum::<i128>()
                + i128::from(*price_only)
                + i128::from(*from_prior_year)
    {
        return Some(
            ValidationIssue::new("CROP_WITHHOLDING_ABOVE_PAYMENTS")
                .at_field("withholding")
                .in_field("insurance"),
        );
    }
    if let Some(issue) = farm_records_issue(ccc, conservation, land_sales) {
        return Some(issue.in_field("more"));
    }
    if let Some((i, _)) =
        f.assets.iter().enumerate().find(|(_, asset)| asset.investment_credit.is_some())
    {
        return Some(
            ValidationIssue::new("FARM_RENTAL_NO_INVESTMENT_CREDIT")
                .at_field("investment_credit")
                .in_list("assets", i),
        );
    }
    shared_business_issue(&f.vehicles, &f.assets)
}

impl crate::related_forms::Activity for Farm {
    const KINDS: &'static [crate::related_forms::DocumentKind] = crate::related_forms::FARM_KINDS;
    fn related(&self) -> &crate::RelatedForms {
        &self.related
    }
    fn related_mut(&mut self) -> &mut crate::RelatedForms {
        &mut self.related
    }
}

impl crate::related_forms::Activity for FarmRental {
    const KINDS: &'static [crate::related_forms::DocumentKind] =
        crate::related_forms::FARM_RENTAL_KINDS;
    fn related(&self) -> &crate::RelatedForms {
        &self.related
    }
    fn related_mut(&mut self) -> &mut crate::RelatedForms {
        &mut self.related
    }
}
