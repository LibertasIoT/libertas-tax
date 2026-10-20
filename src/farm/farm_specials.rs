use crate::validation::ValidationIssue;
// Coverage: F18 source identity/lifetime (2026-10-12). Supplemental farm
//   source rows reuse the accepted farm_sources catalog for page and stored
//   validation. Normal source deletion prunes unavailable rows; source character
//   does not establish a new calculation or source entitlement. Evidence:
//   src/tests/cross/fix_f18.rs D6 genuine W2 selection/delete/cold-load and nine absent99
//   negative controls; the existing EnumSource/client and walk contracts.
// Coverage: section126 mixed funding (batch11). An answered actual excluded
//   funding allocation supplies mixed-improvement basis once; supported FMV/PV
//   still own qualified income and1255. Client and Save bound the funding fact
//   and require zero on known nonexcluded branches. Missing mixed funding stays
//   explicit. Controls: farm_126_mixed_* in src/tests/farm/farm_specials.rs. Broader Farm
//   source, property and loss-owner dependencies remain explicit.
// Farm source facts for supported direct Schedule F and Form 4835 paths.
// Pending shared asset, K-1 and loss-limit owners are not replaced here.

use crate::kernel::phase;
use crate::year_constants::{TAX_YEAR, TAX_YEAR_END_DATE, TAX_YEAR_START_DATE};
use crate::*;
use alloc::{string::String, vec::Vec};
use libertas_macros::*;

/// Schedule F lines B, D, F and G
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmHeader {
    /// Principal activity code from the 17 legal Schedule F choices
    // Tax structure: Schedule F principal activity codes are six-digit codes from its fixed activity catalog.
    #[libertas_number(min = 1, max = 999999)]
    pub code: u32,
    /// Line D actual EIN as nine digits, or absent; preserves leading zeroes in formatted output
    // Legal identifier: an EIN has exactly nine decimal digits, including leading zeroes.
    #[libertas_number(min = 1, max = 999999999)]
    #[libertas_format("000000000")]
    #[libertas_default($null)]
    pub ein: Option<u32>,
    /// Lines F/G actual filing facts
    pub information_returns: FarmInformationReturns,
}

/// Actual information-return requirement and filing answer
/// Check the farm's payment ledger, information-return instructions and filed forms.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmInformationReturns {
    /// No reportable payments requiring information returns
    NotRequired,
    /// Required forms all filed or will be filed
    FiledOrWillFile,
    /// Required forms not all filed; disclosure notice
    NotAllFiled,
}

/// Farm income, production costs, elections and supporting source records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmSpecials {
    /// Capital conservation improvement records; do not repeat these payments in program/other receipts or opening property basis
    #[libertas_size(min = 0)]
    pub cost_sharing: Vec<FarmCostSharing>,
    /// Current weather elections by generic livestock class
    #[libertas_size(min = 0)]
    pub weather_sales: Vec<FarmWeatherSale>,
    /// Prior year elected income included now
    #[libertas_size(min = 0)]
    pub prior_weather: Vec<FarmPriorWeather>,
    /// Signed direct other-income facts; owner-generated amounts are not entered twice
    #[libertas_size(min = 0)]
    pub other_income: Vec<FarmOtherIncome>,
    /// Unrecovered preproductive costs and property links
    #[libertas_size(min = 0)]
    pub production: Vec<FarmProductionCohort>,
    /// Capital fertilizer election and benefit schedules
    #[libertas_size(min = 0)]
    pub fertilizer: Vec<FarmFertilizerCost>,
    /// Actual supplemental source statements linked to accepted documents
    #[libertas_size(min = 0)]
    pub external_sources: Vec<FarmExternalSource>,
    /// Original method/election/approval facts
    pub method_status: FarmMethodStatus,
    /// Named remaining actual tax situations, not an eligibility gate
    // Fixed code catalog: the ten Farm limitation choices are unique under native validation.
    #[libertas_size(min = 0, max = 10)]
    pub limitations: Vec<u32>,
    /// Additional source and the unresolved income, expense timing or property history
    /// Required only for the named Other source. Describe the actual legal source rather than a general completeness confirmation.
    #[libertas_size(min = 1)]
    #[libertas_default($null)]
    pub other_situation: Option<String>,
}

/// Section 126 improvement, actual cost and supported income-right valuation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmCostSharing {
    /// Tax year installation began; determines the three preceding affected-acre gross-receipt years
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub installation_year: u32,
    /// Improvement and location
    #[libertas_size(min = 1)]
    pub description: String,
    /// Government program and approval citation
    #[libertas_size(min = 1)]
    pub program: String,
    /// Program is listed or approved under section 126(a)
    pub program_approved: bool,
    /// Secretary certification covers identified conservation cost
    pub conservation_certified: bool,
    /// Government amount received or accrued under actual method
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub government_received: i64,
    /// Actual total improvement cost
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub total_cost: i64,
    /// Owner paid capital contribution
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub owner_contribution: i64,
    /// Actual fair market value of completed improvement
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub fmv_improvement: i64,
    /// Cost financed by a noneligible program
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub non126_program: i64,
    /// Cost not covered by conservation certification
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub uncertified: i64,
    /// Government payment for rent or services
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub rent_services: i64,
    /// Cost financed by payments associated with current allowable deductions
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub current_deductible: i64,
    /// Excluded cost-share funding applied to this improvement
    /// For mixed funding, use the actual excluded funding allocation, not the present value of an income right. This amount cannot exceed the improvement's cost or the government payment received.
    /// [NullText]
    /// The excluded funding applied is not established
    /// [SetText]
    /// Excluded funding applied
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub excluded_funding_applied: Option<i64>,
    /// Exclusion or actual timely election not to exclude
    pub election: Farm126Election,
    /// Date governing exclusion and recapture history
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub received: u32,
    /// Existing improvement/land basis owner selected once
    #[libertas_number(min = 1)]
    pub asset: i64,
}

/// Section 126 exclusion and timely election choice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum Farm126Election {
    /// Apply the supported section 126 exclusion
    Exclude { facts: Farm126Exclusion },
    /// Timely election not to exclude any payment
    /// No present-value appraisal is needed for this actual election.
    NotExcludeAll,
    /// Timely election not to exclude part of the otherwise excludable portion
    NotExcludePart {
        /// Portion elected out, from the actual election statement
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
        /// Exclusion source records
        facts: Farm126Exclusion,
    },
}

/// Source facts used only for an exclusion claim
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct Farm126Exclusion {
    /// Affected acreage in hundredths, for example 10000 means 100 acres
    #[libertas_number(min = 1)]
    pub acres_hundredths: u32,
    /// Gross receipts from the affected acreage in the three years before installation began
    // Tax structure: section 126 compares the three years preceding installation.
    #[libertas_size(min = 3, max = 3)]
    pub prior_income: Vec<FarmAffectedIncome>,
    /// Supported present values of the two income rights
    /// If unavailable, the estimate omits the exclusion and cannot establish recapture history. The ordinary fully funded worksheet also needs it to establish basis; mixed funding uses the separately recorded actual excluded funding allocation.
    /// [NullText]
    /// Supported section 126 valuation is unavailable
    /// [SetText]
    /// The supported section 126 valuation is
    pub valuation: Option<FarmIncomeRightValue>,
}
impl Farm126Election {
    fn facts(&self) -> Option<&Farm126Exclusion> {
        match self {
            Self::Exclude { facts } | Self::NotExcludePart { facts, .. } => Some(facts),
            Self::NotExcludeAll => None,
        }
    }
}

/// One original affected-acre annual source income
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmAffectedIncome {
    /// One of the three tax years before improvement began
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub year: u32,
    /// Actual annual affected-acre gross receipts, not profit, average or present value
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub income: i64,
}

/// Supported appraisal of income rights; these are source valuations, not an entered exclusion
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmIncomeRightValue {
    /// Present fair market value of right to receive ten-percent average annual income
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub ten_percent_value: i64,
    /// Present fair market value of right to receive two dollars fifty cents per affected acre annually
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub acre_value: i64,
    /// Appraisal method, date, source and income-right assumptions
    #[libertas_size(min = 1)]
    pub support: String,
}

/// Section 451(g) weather-forced generic livestock class election
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmWeatherSale {
    /// Generic class; no subdivision by age, breed or sex
    pub class: FarmLivestockClass,
    /// Income character and product-line allocation for this record
    pub source: FarmWeatherSource,
    /// Animals normally sold under usual business practice
    #[libertas_number(min = 0)]
    pub normal_sold: u32,
    /// All class animals sold in the current year
    #[libertas_number(min = 1)]
    pub all_sold: u32,
    /// Additional animals sold solely because weather affected their requirements
    #[libertas_number(min = 0)]
    pub weather_excess: u32,
    /// Current class realized income already included in product receipts, after resale basis; excludes prior deferral
    /// Include all animals of this generic class under the taxpayer's usual practice, not only the weather-forced animals. Do not add this amount to receipts again.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub class_income: i64,
    /// Farming is taxpayer principal trade or business
    pub principal_farming: bool,
    /// Federal assistance designation; the actual date is optional when unknown
    pub assistance: FarmWeatherAssistance,
    /// Water/feed/grazing impact and relationship of area to forced sales
    #[libertas_size(min = 1)]
    pub weather_relationship: String,
    /// Available preceding-year sales supporting usual practice; no invented history for a new farm
    // Tax structure: the usual-sales history covers the three preceding years, one record per year.
    #[libertas_size(min = 0, max = 3)]
    pub prior_sales: Vec<FarmAnimalYear>,
    /// Source for usual practice, including similarly situated regional farms when no own practice exists
    #[libertas_size(min = 1)]
    pub usual_practice_support: String,
    /// Actual timely section 451(g) election; false keeps income current
    pub elect: bool,
}

/// Actual income character of a generic livestock class
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmWeatherSource {
    /// Animals raised primarily for sale; Schedule F line 2
    RaisedForSale,
    /// Animals purchased primarily for resale; Schedule F line 1
    PurchasedForResale,
    /// Both product kinds within the generic class
    MixedProducts {
        /// Raised-for-sale part of this record's class income or retained deferred amount
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        raised: i64,
    },
    /// Class includes draft, breeding, sport or dairy animals; needs the actual business-asset character owner
    IncludesBusinessAnimals,
}
impl FarmWeatherSource {
    fn split(&self, amount: i64) -> Option<[i64; 2]> {
        match self {
            Self::RaisedForSale => Some([amount, 0]),
            Self::PurchasedForResale => Some([0, amount]),
            Self::MixedProducts { raised } if (0..=amount).contains(raised) => {
                Some([*raised, amount - raised])
            }
            _ => None,
        }
    }
}

/// Actual federal assistance designation, separate from knowing its date
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmWeatherAssistance {
    /// No federal assistance designation established
    NotDesignated,
    /// Area designated; the statement gives the date if known
    Designated {
        /// Actual designation date, including a designation after the sale year
        /// [NullText]
        /// The actual weather-assistance designation date is unavailable
        /// [SetText]
        /// The weather-assistance designation date is
        #[libertas_date_only]
        // Encoding safety: DateOnly encoding uses a four-digit Gregorian year, ending at 9999-12-31.
        #[libertas_number(min = 19000101, max = 99991231)]
        #[libertas_default($null)]
        date: Option<u32>,
    },
}

/// Generic livestock classes for weather income deferral
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmLivestockClass {
    /// Cattle
    Cattle,
    /// Hogs
    Hogs,
    /// Sheep
    Sheep,
    /// Goats
    Goats,
    /// Horses
    Horses,
    /// Poultry; no livestock replacement gain deferral
    Poultry,
    /// Other generic class described in source statement
    Other,
}

/// Retained class sales year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmAnimalYear {
    /// Original year
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub year: u32,
    /// Actual animals sold in that year
    #[libertas_number(min = 0)]
    pub count: u32,
}

/// Prior-year elected amount attributable to this farm
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmPriorWeather {
    /// Original election year
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub year: u32,
    /// Generic livestock class
    pub class: FarmLivestockClass,
    /// Income character and product-line allocation for this record
    pub source: FarmWeatherSource,
    /// Actual retained postponed income included automatically this year; do not repeat in product receipts
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub deferred: i64,
}

/// Direct line 8/43 facts; generated owner feeds stay out of this list
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmOtherIncome {
    /// Commodity ordinary business hedge settlement
    Hedge { facts: FarmHedge },
    /// Illegal irrigation subsidy actually received
    Irrigation { amount: FarmPositiveReceipt },
    /// Identified other farm business income or loss
    Other { facts: FarmSignedReceipt },
}

/// Actual gross receipt
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmPositiveReceipt {
    /// Receipt taxable under the identified source rule
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
}

/// Identified signed ordinary farm other receipt
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmSignedReceipt {
    /// Actual income/loss source and reason it belongs on line 8
    #[libertas_size(min = 1)]
    pub description: String,
    /// Recognized taxable receipt or ordinary loss
    #[libertas_money("USD", 2)]
    pub amount: i64,
}

/// Contemporaneously identified hedge with ordinary risk/timing facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmHedge {
    /// Hedge contract and source schedule
    #[libertas_size(min = 1)]
    pub contract: String,
    /// Original tax identification date
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub identified: u32,
    /// Contract entry date
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub entered: u32,
    /// Settlement date
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub settled: u32,
    /// Signed realized settlement after applicable hedge timing
    #[libertas_money("USD", 2)]
    pub settlement: i64,
    /// Ordinary farm inventory/input/borrowing risk hedged
    #[libertas_size(min = 1)]
    pub business_risk: String,
    /// Actual source timing category
    pub timing: FarmHedgeTiming,
}

/// Hedge timing categories, never investor 60/40 by default
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmHedgeTiming {
    /// Properly identified current-year single-year hedge
    CurrentSingleYear,
    /// Income/cost matched to linked farm inventory cohort
    InventoryMatched,
    /// Straddle, currency, dealer or unentered multiyear allocation needs its owner
    ComplexHistory,
}

/// Property costs separated from ordinary deductible expenses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmProductionCohort {
    /// Accepted production-cost record identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub id: i64,
    /// Produced plant/animal/inventory portion
    #[libertas_size(min = 1)]
    pub description: String,
    /// Actual property type
    pub property: FarmProducedKind,
    /// Production began
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub began: u32,
    /// Productive/service date, when reached
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    #[libertas_default($null)]
    pub productive: Option<u32>,
    /// Actual normal preproductive period
    #[libertas_number(min = 1)]
    pub normal_preproductive_months: u32,
    /// Legal original exemption or method facts
    pub tax_treatment: FarmCapitalization,
    /// Cost components allocated under retained source records
    #[libertas_size(min = 0)]
    pub costs: Vec<FarmProductionCost>,
    /// Unrecovered capitalized tax basis entering year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_basis: i64,
    /// Separate AMT retained basis entering year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_amt_basis: i64,
    /// Existing inventory or depreciable basis route
    pub destination: FarmBasisDestination,
}

/// Actual produced property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmProducedKind {
    /// Plant or bearer plant
    Plant,
    /// Animal raised for production/breeding
    Animal,
    /// Property held for sale, not a depreciable bearer asset
    ResaleInventory,
}

/// Original uniform capitalization method facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmCapitalization {
    /// Uniform capitalization applies under section 263 A
    Required,
    /// Actual gross-receipt and tax-shelter facts
    SmallBusiness { facts: FarmSmallBusiness },
    /// Eligible two-year-or-less plant exception, subject to source exclusions
    ShortPlant {
        /// Corporation, partnership or tax shelter required to use accrual accounting
        required_accrual_entity_or_shelter: bool,
    },
    /// Eligible farming animal exception, subject to source exclusions
    AnimalException {
        /// Corporation, partnership or tax shelter required to use accrual accounting
        required_accrual_entity_or_shelter: bool,
    },
    /// Retained valid original farmer election with ADS consequences
    OriginalElectionOut { year: FarmElectionYear },
    /// Required original indirect-interest or method allocation unentered
    UnresolvedAllocation,
}

/// Statutory receipt-test period and actual tax-shelter status
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmSmallBusiness {
    /// First tax year of the taxpayer/predecessor receipt-test period from actual source records
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub first_tax_year: u32,
    /// Required receipt years: three preceding years, or actual shorter existence; first year uses current receipts
    // Tax structure: the section 448(c) gross-receipts test covers at most three preceding years.
    #[libertas_size(min = 1, max = 3)]
    pub receipts: Vec<FarmGrossYear>,
    /// Tax shelter under section 448(a)(3), including applicable syndicate rule
    pub tax_shelter: bool,
}

/// Annual gross-receipt source for method exception
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmGrossYear {
    /// Original year
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub year: u32,
    /// Months in this actual tax year; short-year annualization is derived once
    // Tax structure: a tax year has at most twelve calendar months.
    #[libertas_number(min = 1, max = 12)]
    pub months: u32,
    /// Aggregated statutory gross receipts from original records before annualization
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub gross: i64,
}

/// Original documented farmer election
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmElectionYear {
    /// Original valid election year
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub year: u32,
    /// Original election and scope evidence
    #[libertas_size(min = 1)]
    pub statement: String,
}

/// Paid/incurred component and capital allocation retained from cost records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmProductionCost {
    /// Actual direct or indirect expense
    #[libertas_size(min = 1)]
    pub description: String,
    /// Existing expense category, not a second category enum
    pub category: BusinessExpenseCategory,
    /// Existing ordinary, meal or nondeductible expense treatment
    /// Applies before allocating tax cost to production; do not repeat these source costs in ordinary farm expenses. De minimis purchases belong in the existing ordinary business-expense election, which excludes inventory and section 263A costs.
    pub treatment: BusinessExpenseKind,
    /// Current paid/incurred tax cost before allocation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Required capital cost under rules other than section 263 A
    /// Actual acquisition/construction or inventory cost; an exemption from uniform capitalization does not turn this into a deduction. Keep it disjoint from the section 263 A allocation below.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub non263a_capital: i64,
    /// Separate required AMT capital cost outside section 263 A
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub non263a_amt_capital: i64,
    /// Actual cost portion allocable under section 263 A cost records
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub capital_allocated: i64,
    /// Corresponding AMT cost allocation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amt_capital_allocated: i64,
    /// Direct, indirect or capitalizable interest component
    pub kind: FarmProductionCostKind,
}

/// Actual production cost source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmProductionCostKind {
    /// Direct production cost
    Direct,
    /// Allocable indirect cost with source allocation record
    IndirectAllocated,
    /// Capitalized interest requires existing accounting owner computation
    InterestOwner,
}

/// One existing basis recipient
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmBasisDestination {
    /// Current established farm inventory cohort
    Inventory {
        /// Actual regular tax cost of the identified portion sold or consumed in 2026
        /// From its unrecovered source-cost allocation. Do not repeat it in ordinary expenses, farm inventory balances or purchased-product cost.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        released: i64,
        /// Corresponding actual AMT tax cost released
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amt_released: i64,
    },
    /// Existing breeding/bearer/improvement asset
    Asset { link: FarmAssetLink },
    /// Gift, inherited, exchange or ownership transfer basis owner missing
    UnresolvedTransfer,
}

/// Canonical property selector
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmAssetLink {
    /// Existing legal asset identity
    #[libertas_number(min = 1)]
    pub asset: i64,
}

/// Section 180 election or capital benefit recovery
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmFertilizerCost {
    /// Fertilizer/lime/material and land treatment
    #[libertas_size(min = 1)]
    pub description: String,
    /// Original capital fertilizer/material/application cost, kept out of ordinary fertilizer expenses
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Date paid under the cash method or incurred under the accrual method
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub paid_or_incurred: u32,
    /// Actual land use before or when these costs were paid or incurred
    pub land_use: FarmFertilizerLand,
    /// These costs are initial preparation of previously unfarmed land
    pub initial_preparation: bool,
    /// Application date
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub applied: u32,
    /// Original deduction or capitalization treatment
    pub treatment: FarmFertilizerTreatment,
}

/// Actual farming use of the treated land, including a tenant's use
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum FarmFertilizerLand {
    /// Used for farming before these expenditures
    PreviouslyFarmed,
    /// Used for farming simultaneously with these expenditures
    SimultaneouslyFarmed,
    /// Not yet used for farming
    NotYetFarmed,
    /// Nonfarming land
    Nonfarming,
}

/// Original fertilizer tax treatment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmFertilizerTreatment {
    /// Valid section 180 current deduction election
    Deduct180,
    /// Capitalized recovery over supported benefit period
    Capitalize { recovery: FarmFertilizerRecovery },
}

/// Original fertilizer recovery history; asked only for capitalized costs
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmFertilizerRecovery {
    /// Actual supported useful benefit period, not an arbitrary rate
    #[libertas_number(min = 1)]
    pub benefit_months: u32,
    /// Basis already recovered through prior returns
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_recovered: i64,
}

/// Retained supplemental statement facts unavailable from existing document totals
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmExternalSource {
    /// Existing accepted K1, W2, 4835 or asset-sale record identity
    #[libertas_number(min = 1)]
    pub source: i64,
    /// Actual activity/source legal character
    pub character: FarmSourceCharacter,
    /// Gross production sales from retained statement, not net profit
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub gross_sales: i64,
    /// Product cost/inventory reduction attributable to those sales
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub product_cost: i64,
    /// Required written pre-production crop/vessel share agreement facts
    #[libertas_default($null)]
    pub agreement: Option<FarmShareAgreement>,
    /// Farming/fishing S-corp wages belong to a shareholder employee
    pub shareholder_employee: bool,
    /// Linked disposed property was regularly used substantially in farming
    pub regularly_used: bool,
    /// Date operation ceased, if property sold afterward
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    #[libertas_default($null)]
    pub ceased: Option<u32>,
}

/// Different eligibility for different tax projections
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmSourceCharacter {
    /// Actual K1 farming production interest
    FarmK1,
    /// Actual K1 commercial fishing interest
    FishingK1,
    /// Actual S-corp farm distributive gross statement
    ScorpFarm,
    /// Shareholder employee farming/fishing W2 wages
    ScorpWages,
    /// Compensation based on share of catch/proceeds
    FishingCrewShare,
    /// Otherwise eligible crew fishing wages for farmer gross test, not automatic J
    FishingCrewWages,
    /// Production-share rent under written agreement
    CropShare,
    /// Fishing-vessel production-share lease
    VesselShare,
    /// Linked farm asset sale, excluding land/right gains from J
    FarmProperty,
    /// Source needs exact legal characterization
    Other,
}

/// Original production share agreement facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmShareAgreement {
    /// Written agreement signed
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub signed: u32,
    /// Tenant or lessee began significant production
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub significant_activity: u32,
    /// Payment is production/catch share, not fixed rent
    pub production_based: bool,
}

/// Actual method history; no approval implied by editing current method
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmMethodStatus {
    /// Actual established original accounting method
    pub original: FarmOriginalMethod,
    /// Year method adopted
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub adopted: u32,
    /// Existing approved method change and transition records
    #[libertas_default($null)]
    pub change: Option<FarmMethodChangeLink>,
}

/// Original farm accounting method
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmOriginalMethod {
    /// Cash method
    Cash,
    /// Accrual method
    Accrual,
    /// Entity or farming syndicate must use proper entity/K1 source owner
    EntitySyndicate,
}

/// Source-approved method change, one section 481 owner
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmMethodChangeLink {
    /// Stable existing accepted method-change record
    #[libertas_number(min = 1)]
    pub change: i64,
    /// Original approval/automatic-change authority and transition evidence
    #[libertas_size(min = 1)]
    pub approval: String,
}

/// Schedule F lines B, D, F and G
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmHeaderInput {
    /// Principal activity code from the 17 legal Schedule F choices
    #[libertas_enum_source("$.farm_codes")]
    // Tax structure: Schedule F principal activity codes are six-digit codes from its fixed activity catalog.
    #[libertas_number(min = 1, max = 999999)]
    pub code: u32,
    /// Line D actual EIN as nine digits, or absent; preserves leading zeroes in formatted output
    /// [NullText]
    /// No farm EIN supplied for the printed header
    /// [SetText]
    /// The farm EIN is
    // Legal identifier: an EIN has exactly nine decimal digits, including leading zeroes.
    #[libertas_number(min = 1, max = 999999999)]
    #[libertas_format("000000000")]
    #[libertas_default($null)]
    pub ein: Option<u32>,
    /// Lines F/G actual filing facts
    pub information_returns: FarmInformationReturns,
}

/// Farm income, production costs, elections and supporting source records
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmSpecialsInput {
    /// Capital conservation improvement records; do not repeat these payments in program/other receipts or opening property basis
    #[libertas_size(min = 0)]
    pub cost_sharing: Vec<FarmCostSharingInput>,
    /// Current weather elections by generic livestock class
    #[libertas_size(min = 0)]
    pub weather_sales: Vec<FarmWeatherSale>,
    /// Prior year elected income included now
    #[libertas_size(min = 0)]
    pub prior_weather: Vec<FarmPriorWeather>,
    /// Signed direct other-income facts; owner-generated amounts are not entered twice
    #[libertas_size(min = 0)]
    pub other_income: Vec<FarmOtherIncome>,
    /// Unrecovered preproductive costs and property links
    #[libertas_size(min = 0)]
    pub production: Vec<FarmProductionCohortInput>,
    /// Capital fertilizer election and benefit schedules
    #[libertas_size(min = 0)]
    pub fertilizer: Vec<FarmFertilizerCost>,
    /// Actual supplemental source statements linked to accepted documents
    #[libertas_size(min = 0)]
    pub external_sources: Vec<FarmExternalSourceInput>,
    /// Original method/election/approval facts
    pub method_status: FarmMethodStatusInput,
    /// Named remaining actual tax situations, not an eligibility gate
    // Fixed code catalog: the ten Farm limitation choices are unique under native validation.
    #[libertas_size(min = 0, max = 10)]
    #[libertas_enum_source("$.farm_limitations")]
    pub limitations: Vec<u32>,
    /// Additional source and the unresolved income, expense timing or property history
    /// Required only for the named Other source. Describe the actual legal source rather than a general completeness confirmation.
    /// [NullText]
    /// No named Other farm source applies
    /// [SetText]
    /// The additional Other farm source is
    #[libertas_size(min = 1)]
    pub other_situation: Option<String>,
}

/// Section 126 improvement, actual cost and supported income-right valuation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmCostSharingInput {
    /// Tax year installation began; determines the three preceding affected-acre gross-receipt years
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub installation_year: u32,
    /// Improvement and location
    #[libertas_size(min = 1)]
    pub description: String,
    /// Government program and approval citation
    #[libertas_size(min = 1)]
    pub program: String,
    /// Program is listed or approved under section 126(a)
    pub program_approved: bool,
    /// Secretary certification covers identified conservation cost
    pub conservation_certified: bool,
    /// Government amount received or accrued under actual method
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub government_received: i64,
    /// Actual total improvement cost
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub total_cost: i64,
    /// Owner paid capital contribution
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub owner_contribution: i64,
    /// Actual fair market value of completed improvement
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub fmv_improvement: i64,
    /// Cost financed by a noneligible program
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub non126_program: i64,
    /// Cost not covered by conservation certification
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub uncertified: i64,
    /// Government payment for rent or services
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub rent_services: i64,
    /// Cost financed by payments associated with current allowable deductions
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub current_deductible: i64,
    /// Excluded cost-share funding applied to this improvement
    /// For mixed funding, use the actual excluded funding allocation, not the present value of an income right. This amount cannot exceed the improvement's cost or the government payment received.
    /// [NullText]
    /// The excluded funding applied is not established
    /// [SetText]
    /// Excluded funding applied
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub excluded_funding_applied: Option<i64>,
    /// Exclusion or actual timely election not to exclude
    pub election: Farm126Election,
    /// Date governing exclusion and recapture history
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub received: u32,
    /// Existing improvement/land basis owner selected once
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.basis_assets")]
    pub asset: i64,
}

/// Property costs separated from ordinary deductible expenses
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmProductionCohortInput {
    /// Accepted production-cost record identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub id: i64,
    /// Produced plant/animal/inventory portion
    #[libertas_size(min = 1)]
    pub description: String,
    /// Actual property type
    pub property: FarmProducedKind,
    /// Production began
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub began: u32,
    /// Productive/service date, when reached
    /// [NullText]
    /// This property has not become productive or entered service
    /// [SetText]
    /// This property became productive or entered service on
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub productive: Option<u32>,
    /// Actual normal preproductive period
    #[libertas_number(min = 1)]
    pub normal_preproductive_months: u32,
    /// Legal original exemption or method facts
    pub tax_treatment: FarmCapitalization,
    /// Cost components allocated under retained source records
    #[libertas_size(min = 0)]
    pub costs: Vec<FarmProductionCost>,
    /// Unrecovered capitalized tax basis entering year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_basis: i64,
    /// Separate AMT retained basis entering year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_amt_basis: i64,
    /// Existing inventory or depreciable basis route
    pub destination: FarmBasisDestinationInput,
}

/// One existing basis recipient
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmBasisDestinationInput {
    /// Current established farm inventory cohort
    Inventory {
        /// Actual regular tax cost of the identified portion sold or consumed in 2026
        /// From its unrecovered source-cost allocation. Do not repeat it in ordinary expenses, farm inventory balances or purchased-product cost.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        released: i64,
        /// Corresponding actual AMT tax cost released
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amt_released: i64,
    },
    /// Existing breeding/bearer/improvement asset
    Asset { link: FarmAssetLinkInput },
    /// Gift, inherited, exchange or ownership transfer basis owner missing
    UnresolvedTransfer,
}

/// Canonical property selector
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmAssetLinkInput {
    /// Existing depreciable-property source identity
    /// This recovery owner is not available in the current estimate. Land parcel selectors do not identify depreciable property.
    #[libertas_number(min = 1)]
    pub asset: i64,
}

/// Retained supplemental statement facts unavailable from existing document totals
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmExternalSourceInput {
    /// Existing accepted K1, W2, 4835 or asset-sale record identity
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.farm_sources")]
    pub source: i64,
    /// Actual activity/source legal character
    pub character: FarmSourceCharacter,
    /// Gross production sales from retained statement, not net profit
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub gross_sales: i64,
    /// Product cost/inventory reduction attributable to those sales
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub product_cost: i64,
    /// Required written pre-production crop/vessel share agreement facts
    /// [NullText]
    /// No qualifying written preproduction-sharing agreement
    /// [SetText]
    /// The written preproduction-sharing agreement is
    pub agreement: Option<FarmShareAgreement>,
    /// Farming/fishing S-corp wages belong to a shareholder employee
    pub shareholder_employee: bool,
    /// Linked disposed property was regularly used substantially in farming
    pub regularly_used: bool,
    /// Date operation ceased, if property sold afterward
    /// [NullText]
    /// The farming operation has not ceased
    /// [SetText]
    /// The farming operation ceased on
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub ceased: Option<u32>,
}

/// Actual method history; no approval implied by editing current method
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmMethodStatusInput {
    /// Actual established original accounting method
    pub original: FarmOriginalMethod,
    /// Year method adopted
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
    pub adopted: u32,
    /// Existing approved method change and transition records
    /// [NullText]
    /// No approved accounting-method change applies
    /// [SetText]
    /// The approved method-change and transition records are
    pub change: Option<FarmMethodChangeLinkInput>,
}

/// Source-approved method change, one section 481 owner
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmMethodChangeLinkInput {
    /// Stable existing accepted method-change record
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.farm_method_changes")]
    pub change: i64,
    /// Original approval/automatic-change authority and transition evidence
    #[libertas_size(min = 1)]
    pub approval: String,
}

/// Actual CCC collateral source facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CccSourceFactsInput {
    /// Producer owns the raised collateral crop
    pub producer_owned_raised: bool,
    /// Original tax production cost not already deducted
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub unrecovered_production_cost: i64,
    /// Actual original section 77 election history
    pub history: CccElectionHistory,
}

/// Exact election delta on existing FarmAveraging, retaining all base-year types
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmAveragingSelectionInput {
    /// Schedule J line 2 b net capital gain selected within total elected amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub capital_elected: i64,
    /// Line 2 c farm unrecaptured section 1250 gain within selected capital
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub unrecaptured_elected: i64,
    /// Distinct additional eligible sources shown in current accepted-record catalog
    #[libertas_size(min = 0)]
    #[libertas_enum_source("$.farm_sources")]
    pub sources: Vec<i64>,
    /// Actual retained base-year scope; known limitations affect averaging only
    pub history_scope: FarmAveragingHistoryScope,
}

/// Retained source facts for one farm land parcel
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmLandBasis {
    /// Accepted canonical property identity
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    #[libertas_number(min = 0)]
    pub id: i64,
    /// Parcel name from your records
    #[libertas_size(min = 1)]
    pub label: String,
    /// Actual acquisition character; retain the existing owner's special-basis boundaries
    pub acquisition: FarmLandAcquisition,
    /// Actual acquisition or inherited owner's death date
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub acquired: u32,
    /// Actual adjusted basis entering year, or supported acquisition basis for a current-year parcel; excludes live current126/175/COOP adjustments
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub regular: i64,
    /// Corresponding actual AMT opening/acquisition basis, excluding those same live adjustments
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amt: i64,
    /// Your prior section175 deductions allowed on this parcel, excluding current amounts and a decedent's reset balance
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_conservation: i64,
    /// Corresponding prior section 175 deductions actually allowed for AMT
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_amt_conservation: i64,
    /// Your prior section126 excludable portions on this parcel, excluding a decedent's reset balance under estate fair-market-value basis
    pub prior_cost_sharing: Vec<Farm126History>,
}

/// Original section 126 exclusion fact, not a current benefit computation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct Farm126History {
    /// Last certified government payment date for this improvement, from the retained exclusion statement
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub received: u32,
    /// Amount actually excluded under section 126 in that original year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub excluded: i64,
    /// Original payment/program and statement reference
    #[libertas_size(min = 1)]
    pub statement: String,
}

/// Conservation payment record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ConservationCostInput {
    /// Canonical farm land parcel
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.basis_assets")]
    pub property: i64,

    /// Work or payment description
    #[libertas_size(min = 1)]
    pub label: String,
    /// Cost paid or incurred under your farm accounting method
    /// Exclude reimbursements, amounts deducted elsewhere, ordinary maintenance and separately depreciable assets.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
    /// Actual conservation plan and work facts
    pub facts: FarmConservationFacts,
    /// Date paid or incurred under your method, when needed for a current land sale
    /// [NullText]
    /// The payment date is unavailable or not needed for a current sale
    /// [SetText]
    /// This conservation cost was paid or incurred on
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = $crate::year_constants::TAX_YEAR_START_DATE, max = $crate::year_constants::TAX_YEAR_END_DATE)]
    pub paid_or_incurred: Option<u32>,
}

/// Conservation district's depreciable property assessment
/// One record per property assessment, combining all installments. Use the district's original complete assessment and all payment/accrual records through 2026; do not enter a computed deductible amount. Revised assessment and reimbursement require their actual source recalculation; disposal uses the linked land basis owner. A final return uses the linked farm owner's death facts to accelerate eligible prior-year assessment balances.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ConservationAssessmentInput {
    /// Actual conservation plan and work records
    pub facts: FarmConservationFacts,
    /// Canonical farm land parcel
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.basis_assets")]
    pub property: i64,

    /// District property or project
    #[libertas_size(min = 1)]
    pub label: String,
    /// Total assessment against all district members for this property
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub total: i64,
    /// Your complete assessment for this property, including future installments
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub share: i64,
    /// Payments or accruals under your accounting method
    /// Cash filers enter each paid installment; accrual filers enter the amount in its proper incurred year, not again when paid. The App combines entries for the same year.
    #[libertas_size(min = 1)]
    pub payments: Vec<ConservationAssessmentPayment>,
}

/// Retained section 175 carry for one parcel and source year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmConservationCarry {
    /// Canonical farm land parcel
    #[libertas_number(min = 1)]
    pub property: i64,
    /// Original cost year from retained worksheets
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1954, max = $crate::year_constants::TAX_YEAR)]
    pub year: u32,
    /// Unused regular-tax 25-percent-limited cost
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub regular: i64,
    /// Corresponding unused AMT cost
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amt: i64,
}

/// Retained section 175 carry for one parcel and source year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmConservationCarryInput {
    /// Canonical farm land parcel
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.basis_assets")]
    pub property: i64,
    /// Original cost year from retained worksheets
    // Tax structure: this source year must be no later than the 2026 return year.
    #[libertas_number(min = 1954, max = $crate::year_constants::TAX_YEAR)]
    pub year: u32,
    /// Unused regular-tax 25-percent-limited cost
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub regular: i64,
    /// Corresponding unused AMT cost
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amt: i64,
}

/// Conservation plan, work and actual land-use facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmConservationFacts {
    /// Actual approved plan source
    pub plan: FarmConservationPlan,
    /// Actual work rather than an eligibility checkbox
    pub work: FarmConservationWork,
    /// Land used in farming now or previously by you or your tenant
    /// A newly acquired operating farm also qualifies when you substantially continue the immediately preceding owner's farming use. Newly developed unused land does not qualify merely because future farming is intended.
    pub land_farmed: bool,
    /// NRCS/state or endangered-species site-specific approval identifier
    /// [NullText]
    /// No approved conservation plan applies
    /// [SetText]
    /// The conservation-plan approval identifier is
    #[libertas_size(min = 1)]
    pub plan_reference: Option<String>,
}

/// Actual conservation plan source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmConservationPlan {
    /// NRCS-approved soil/water plan
    Nrcs,
    /// Comparable state plan
    ComparableState,
    /// Site-specific approved endangered-species recovery plan
    EndangeredRecovery,
    /// No approved qualifying plan; no elective deduction
    None,
}

/// Actual cost category
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmConservationWork {
    /// Conservation/erosion work otherwise eligible
    QualifyingLandWork,
    /// Site-specific endangered-species management
    EndangeredRecovery,
    /// Wetlands draining/filling; capitalize
    WetlandDrainage,
    /// Center-pivot irrigation land preparation; capitalize
    PivotPreparation,
    /// Owner depreciable facility; shared asset owner
    OwnerDepreciable,
    /// Ordinary repairs/maintenance; expense owner
    Maintenance,
}

/// Actual current unused-supply purchase and refund
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmSupplyPurchase {
    /// Purchase versus refundable deposit
    pub kind: FarmSupplyPaymentKind,
    /// Actual fixed quantity/price and delivery obligation
    pub quantity_price_terms: String,
    /// Source business reason beyond reducing tax
    pub business_purpose: String,
    /// Deduction materially distorts income under actual facts
    pub material_distortion: bool,
    /// Current refund attributable to this identified payment
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub refunded: i64,
    /// Original payment date
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub paid: u32,
}

/// Actual unused-supply payment terms
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmSupplyPaymentKind {
    /// Fixed purchase obligation with no refund/repurchase right
    BindingPurchase,
    /// Refundable deposit; carry asset until lawful consumption/refund
    Deposit,
    /// Terms not supplied; omit only unestablished current deduction
    UnresolvedTerms,
}

/// Actual current production-poultry purchase cohort
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmProductionPoultryCurrent {
    /// Identified poultry lot
    #[libertas_size(min = 1)]
    pub label: String,
    /// Actual current paid purchase cost before the prepaid ceiling
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub cost: i64,
    /// Original payment date
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub paid: u32,
    /// Actual start of business care or use, not the first egg or mature production date
    /// The supported monthly allocation begins on the first day of a month in the current year. Partial-month or earlier starts need their actual retained timing worksheet.
    #[libertas_date_only]
    // Tax structure: this source event must occur on or before the end of the 2026 return year.
    #[libertas_number(min = 19000101, max = $TAX_YEAR_END_DATE)]
    pub started: u32,
    /// Actual useful business life in whole months
    /// From the lot's production and replacement records. The hypothetical section 464 period is the lesser of this life and 12 months; do not enter a deduction percentage.
    #[libertas_number(min = 1)]
    pub useful_months: u32,
    /// Existing adopted paid/sold cost method where legally applicable
    pub adopted: FarmPoultryMethod,
}

impl FarmProductionPoultryCurrent {
    pub(crate) fn valid(&self) -> bool {
        !self.label.trim().is_empty()
            && (0..=100_000_000_000).contains(&self.cost)
            && (TAX_YEAR_START_DATE..=TAX_YEAR_END_DATE).contains(&self.paid)
            && crate::federal_rules::valid_date(self.paid)
            && (19000101..=TAX_YEAR_END_DATE).contains(&self.started)
            && crate::federal_rules::valid_date(self.started)
            && (1..=1200).contains(&self.useful_months)
    }
    // Supported actual whole-calendar-month ratable period. Section464 and
    // Pub225 specify lesser-of-12-months/life but no partial-month convention;
    // never silently supply that missing source convention.
    pub(crate) fn timing(&self) -> Option<[i64; 2]> {
        if !self.valid() {
            return None;
        }
        if self.cost == 0 {
            return Some([0, 0]);
        }
        if !matches!(self.adopted, FarmPoultryMethod::Paid)
            || self.started / 10000 != TAX_YEAR
            || self.started % 100 != 1
        {
            return None;
        }
        let period = phase::cap(self.useful_months, [12]);
        let current = phase::cap(13 - self.started / 100 % 100, [period]);
        let now = crate::kernel::ratio::fraction(self.cost, i64::from(current), i64::from(period));
        Some([now, self.cost - now])
    }
}

/// Exact election delta on existing FarmAveraging, retaining all base-year types
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmAveragingSelection {
    /// Schedule J line 2 b net capital gain selected within total elected amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub capital_elected: i64,
    /// Line 2 c farm unrecaptured section 1250 gain within selected capital
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub unrecaptured_elected: i64,
    /// Distinct additional eligible sources shown in current accepted-record catalog
    #[libertas_size(min = 0)]
    pub sources: Vec<i64>,
    /// Actual retained base-year scope; known limitations affect averaging only
    pub history_scope: FarmAveragingHistoryScope,
}

impl FarmAveragingSelection {
    pub(crate) fn issue(&self, elected: i64) -> Option<ValidationIssue> {
        if !(0..=elected).contains(&self.capital_elected) {
            return Some(
                ValidationIssue::new("FARM_AVERAGING_SELECTION_INVALID")
                    .at_field("capital_elected"),
            );
        }
        if !(0..=self.capital_elected).contains(&self.unrecaptured_elected) {
            return Some(
                ValidationIssue::new("FARM_AVERAGING_SELECTION_INVALID")
                    .at_field("unrecaptured_elected"),
            );
        }
        if self.sources.len() > 100 {
            return Some(
                ValidationIssue::new("FARM_AVERAGING_SOURCES_INVALID").at_field("sources"),
            );
        }
        for (i, source) in self.sources.iter().enumerate() {
            if !(1..=100_000_000_000).contains(source) || self.sources[..i].contains(source) {
                return Some(
                    ValidationIssue::new("FARM_AVERAGING_SOURCES_INVALID")
                        .at_list_item("sources", i),
                );
            }
        }
        None
    }
    pub(crate) fn rule(&self, elected: i64) -> Option<&'static str> {
        self.issue(elected).map(|issue| issue.key)
    }
    pub(crate) fn valid(&self, elected: i64) -> bool {
        self.rule(elected).is_none()
    }
}

/// Remaining original history boundaries
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum FarmAveragingHistoryScope {
    /// Full-year resident returns with retained return/J/foreign/investment source worksheets
    /// Use the actual worksheets. This choice does not establish missing current income, source attribution or spouse allocation.
    CompleteSupported,
    /// Two farming spouses with separate incomplete base-year ownership history
    SeparateSpouseHistories,
    /// Prior NOL components need HIST owner
    PriorNol,
    /// Prior/current MSA attribution not supplied
    ArcherMsa,
    /// Minor-filer base tax needs its owner
    MinorTax,
    /// Unmodeled short-year/nonresident tax worksheet
    ShortYearOrForeign,
}

/// Request-local keyed source catalog item
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmChoice {
    /// Stable accepted source identity
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_number(min = 1)]
    pub id: i64,
    /// Human source/line name
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}

/// Request-local stable statutory code/limitation item
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FarmCodeChoice {
    /// Stable source code, not list position
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    #[libertas_number(min = 1)]
    pub id: u32,
    /// Human code description
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}

/// Actual CCC collateral source facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CccSourceFacts {
    /// Producer owns the raised collateral crop
    pub producer_owned_raised: bool,
    /// Original tax production cost not already deducted
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub unrecovered_production_cost: i64,
    /// Actual original section 77 election history
    pub history: CccElectionHistory,
}

/// Original section 77 reporting history, not a general scope confirmation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum CccElectionHistory {
    /// Loan proceeds have never been elected into income
    NoIncomeElection,
    /// Original election applies to all current and later loans
    IncomeElection {
        /// Original election year from the retained return
        // Tax structure: this source year must be no later than the 2026 return year.
        #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
        year: u32,
        /// Retained timely election statement and loan-detail source
        #[libertas_size(min = 1)]
        statement: String,
    },
    /// An actual method change needs its original transition and commodity basis owner
    ChangedMethod {
        /// Original change year
        // Tax structure: this source year must be no later than the 2026 return year.
        #[libertas_number(min = 1900, max = $crate::year_constants::TAX_YEAR)]
        year: u32,
        /// Actual consent or automatic-change authority
        #[libertas_size(min = 1)]
        authority: String,
    },
}

/// Actual normal sales practice for each damaged crop
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct FarmCropPractice {
    /// Matches the Physical crop source label
    #[libertas_size(min = 1)]
    pub crop: String,
    /// Actual normal annual crop income supported by records
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub normal_total: i64,
    /// Normal portion belonging to a year after the damage year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub normal_following: i64,
}

record_page_conversions!(FarmHeader, FarmHeaderInput, [&], |value, Target| {
    Target {
        code: value.code,
        ein: value.ein,
        information_returns: value.information_returns.clone(),
    }
});

const FARM_CODES: &[(u32, &str)] = &[
    (111100, "FARM_CODE_111100"),
    (111210, "FARM_CODE_111210"),
    (111300, "FARM_CODE_111300"),
    (111400, "FARM_CODE_111400"),
    (111900, "FARM_CODE_111900"),
    (112111, "FARM_CODE_112111"),
    (112112, "FARM_CODE_112112"),
    (112120, "FARM_CODE_112120"),
    (112210, "FARM_CODE_112210"),
    (112300, "FARM_CODE_112300"),
    (112400, "FARM_CODE_112400"),
    (112510, "FARM_CODE_112510"),
    (112900, "FARM_CODE_112900"),
    (113000, "FARM_CODE_113000"),
    (113110, "FARM_CODE_113110"),
    (113210, "FARM_CODE_113210"),
    (113310, "FARM_CODE_113310"),
];
pub(crate) fn code_choices() -> Vec<FarmCodeChoice> {
    FARM_CODES
        .iter()
        .map(|&(id, key)| FarmCodeChoice { id, label: Message::new(key, alloc::vec![]).encode() })
        .collect()
}
impl FarmHeader {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if !FARM_CODES.iter().any(|&(code, _)| code == self.code) {
            return Some(ValidationIssue::new("FARM_HEADER_CODE_INVALID").at_field("code"));
        }
        if self.ein.is_some_and(|ein| ein == 0 || ein > 999999999) {
            return Some(ValidationIssue::new("FARM_HEADER_EIN_INVALID").at_field("ein"));
        }
        None
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
}
pub(crate) fn report_header(
    header: Option<&FarmHeader>,
    id: i64,
    context: IssueContext,
    r: &mut FederalResult,
) {
    crate::require_localized!(
        r,
        header.is_some(),
        &context,
        "Schedule F header",
        "ISSUE_FARM_HEADER_REQUIRED",
        "The Schedule F principal agricultural code and information-return answers are needed.",
    );
    let Some(header) = header else { return };
    let information = match header.information_returns {
        FarmInformationReturns::NotRequired => "FARM_INFORMATION_NOT_REQUIRED",
        FarmInformationReturns::FiledOrWillFile => "FARM_INFORMATION_FILED",
        FarmInformationReturns::NotAllFiled => "FARM_INFORMATION_NOT_FILED",
    };
    let ein = header.ein.map_or_else(
        || msg!("FARM_EIN_ABSENT"),
        |ein| Message::new("TEXT", alloc::vec![MessageArg::Text(alloc::format!("{ein:09}"))]),
    );
    let required = match header.information_returns {
        FarmInformationReturns::NotRequired => "PRINT_NO",
        FarmInformationReturns::FiledOrWillFile | FarmInformationReturns::NotAllFiled => {
            "PRINT_YES"
        }
    };
    let filed = match header.information_returns {
        FarmInformationReturns::NotRequired => "PRINT_NOT_APPLICABLE",
        FarmInformationReturns::FiledOrWillFile => "PRINT_YES",
        FarmInformationReturns::NotAllFiled => "PRINT_NO",
    };
    for (printed, suffix, value) in [
        (
            "B",
            "activity_code",
            Message::new(
                "TEXT",
                alloc::vec![MessageArg::Text(alloc::format!("{:06}", header.code))],
            ),
        ),
        ("D", "ein", ein.clone()),
        ("F", "information_required", Message::new(required, alloc::vec![])),
        ("G", "information_filed", Message::new(filed, alloc::vec![])),
    ] {
        crate::kernel::lines::form_value(
            r,
            FormLine::new(TaxForm::ScheduleF, printed),
            &alloc::format!("farm_{id}_header_{suffix}"),
            crate::kernel::lines::LineValue::Text,
            &["income.business.activity.farming.header"],
            "https://www.irs.gov/pub/irs-dft/f1040sf--dft.pdf",
            value,
        );
    }
    crate::federal_rules::note_once_scoped(
        r,
        Message::new(
            "NOTE_FARM_HEADER",
            alloc::vec![
                MessageArg::Number(header.code.into()),
                MessageArg::Nested(alloc::boxed::Box::new(ein)),
                MessageArg::Nested(alloc::boxed::Box::new(Message::new(
                    information,
                    alloc::vec![]
                )))
            ],
        ),
        context.clone(),
    );
    if matches!(header.information_returns, FarmInformationReturns::NotAllFiled) {
        crate::federal_rules::note_once_scoped(
            r,
            msg!("NOTE_FARM_INFORMATION_INCOMPLETE"),
            context,
        );
    }
}

record_page_conversions!(FarmSpecials, FarmSpecialsInput, [&], |value, Target| {
    Target {
        cost_sharing: value.cost_sharing.iter().map(Into::into).collect(),
        weather_sales: value.weather_sales.clone(),
        prior_weather: value.prior_weather.clone(),
        other_income: value.other_income.clone(),
        production: value.production.iter().map(Into::into).collect(),
        fertilizer: value.fertilizer.clone(),
        external_sources: value.external_sources.iter().map(Into::into).collect(),
        method_status: (&value.method_status).into(),
        limitations: value.limitations.clone(),
        other_situation: value.other_situation.clone(),
    }
});

record_page_conversions!(FarmCostSharing, FarmCostSharingInput, [&], |value, Target| {
    Target {
        installation_year: value.installation_year,
        description: value.description.clone(),
        program: value.program.clone(),
        program_approved: value.program_approved,
        conservation_certified: value.conservation_certified,
        government_received: value.government_received,
        total_cost: value.total_cost,
        owner_contribution: value.owner_contribution,
        fmv_improvement: value.fmv_improvement,
        non126_program: value.non126_program,
        uncertified: value.uncertified,
        rent_services: value.rent_services,
        current_deductible: value.current_deductible,
        excluded_funding_applied: value.excluded_funding_applied,
        election: value.election.clone(),
        received: value.received,
        asset: value.asset,
    }
});

record_page_conversions!(FarmProductionCohort, FarmProductionCohortInput, [&], |value, Target| {
    Target {
        id: value.id,
        description: value.description.clone(),
        property: value.property.clone(),
        began: value.began,
        productive: value.productive,
        normal_preproductive_months: value.normal_preproductive_months,
        tax_treatment: value.tax_treatment.clone(),
        costs: value.costs.clone(),
        prior_basis: value.prior_basis,
        prior_amt_basis: value.prior_amt_basis,
        destination: (&value.destination).into(),
    }
});

record_page_conversions!(FarmAssetLink, FarmAssetLinkInput, [&], |value, Target| {
    Target { asset: value.asset }
});

record_page_conversions!(FarmExternalSource, FarmExternalSourceInput, [&], |value, Target| {
    Target {
        source: value.source,
        character: value.character.clone(),
        gross_sales: value.gross_sales,
        product_cost: value.product_cost,
        agreement: value.agreement.clone(),
        shareholder_employee: value.shareholder_employee,
        regularly_used: value.regularly_used,
        ceased: value.ceased,
    }
});

record_page_conversions!(FarmMethodStatus, FarmMethodStatusInput, [&], |value, Target| {
    Target {
        original: value.original.clone(),
        adopted: value.adopted,
        change: value.change.as_ref().map(Into::into),
    }
});

record_page_conversions!(FarmMethodChangeLink, FarmMethodChangeLinkInput, [&], |value, Target| {
    Target { change: value.change, approval: value.approval.clone() }
});

record_page_conversions!(CccSourceFacts, CccSourceFactsInput, [&], |value, Target| {
    Target {
        producer_owned_raised: value.producer_owned_raised,
        unrecovered_production_cost: value.unrecovered_production_cost,
        history: value.history.clone(),
    }
});

record_page_conversions!(FarmAveragingSelection, FarmAveragingSelectionInput, [&], |value, Target| {
    Target {
        capital_elected: value.capital_elected,
        unrecaptured_elected: value.unrecaptured_elected,
        sources: value.sources.clone(),
        history_scope: value.history_scope.clone(),
    }
});

record_page_conversions!(FarmConservationCarry, FarmConservationCarryInput, [&], |value, Target| {
    Target { property: value.property, year: value.year, regular: value.regular, amt: value.amt }
});

record_page_conversions!(FarmBasisDestination, FarmBasisDestinationInput, [&], |value, Target, Source| {
    match value {
        Source::Inventory { released, amt_released } => {
            Target::Inventory { released: *released, amt_released: *amt_released }
        }
        Source::Asset { link } => Target::Asset { link: link.into() },
        Source::UnresolvedTransfer => Target::UnresolvedTransfer,
    }
});

pub(crate) fn limitation_choices() -> Vec<FarmCodeChoice> {
    (1..=10)
        .map(|id| FarmCodeChoice {
            id,
            label: Message::new(alloc::format!("FARM_LIMITATION_{id}"), alloc::vec![]).encode(),
        })
        .collect()
}
// The accepted-source catalog owns this relation for both page saves and
// stored returns. A document's character facts do not create its identity.
pub(crate) fn external_source_available(d: &FederalDraft, id: i64) -> bool {
    d.income.iter().any(|e| {
        e.id == id
            && matches!(
                e.income,
                FederalIncome::Partnership { .. }
                    | FederalIncome::Scorp { .. }
                    | FederalIncome::EstateTrustPortfolio { .. }
                    | FederalIncome::Wage { .. }
                    | FederalIncome::FarmRental { .. }
                    | FederalIncome::Business { .. }
            )
    })
}
pub(crate) fn external_source_rule(d: &FederalDraft) -> Option<&'static str> {
    crate::farm::sources(d)
        .any(|farm| {
            farm.farm
                .specials
                .external_sources
                .iter()
                .any(|source| !external_source_available(d, source.source))
        })
        .then_some("FARM_EXTERNAL_SOURCE_UNAVAILABLE")
}
pub(crate) fn source_choices(d: &FederalDraft) -> Vec<FarmChoice> {
    d.income
        .iter()
        .filter(|e| external_source_available(d, e.id))
        .map(|e| FarmChoice {
            id: e.id,
            label: crate::interview::income_display_label(e, d.dependents()).encode(),
        })
        .collect()
}
pub(crate) fn basis_choices(d: &FederalDraft) -> Vec<FarmChoice> {
    // Actual accepted land only; the pending shared asset owner provides its
    // own real IDs before depreciable improvements can join this catalog.
    crate::farm::sources(d)
        .flat_map(|source| {
            source.farm.land_basis.iter().filter(|land| land.id > 0).map(move |land| FarmChoice {
                id: land.id,
                label: Message::new(
                    "FARM_PARCEL_CHOICE",
                    alloc::vec![
                        MessageArg::Text(source.entry.label.clone()),
                        MessageArg::Text(land.label.clone())
                    ],
                )
                .encode(),
            })
        })
        .collect()
}
pub(crate) fn method_choices(d: &FederalDraft) -> Vec<FarmChoice> {
    crate::farm::sources(d)
        .flat_map(|source| {
            crate::method_change::records(&source.entry.income).into_iter().flat_map(
                move |records| {
                    records.iter().filter(|record| record.id > 0).map(move |record| FarmChoice {
                        id: record.id,
                        label: Message::new(
                            "FARM_METHOD_CHANGE_CHOICE",
                            alloc::vec![
                                MessageArg::Text(source.entry.label.clone()),
                                MessageArg::Text(alloc::format!("{}", record.year))
                            ],
                        )
                        .encode(),
                    })
                },
            )
        })
        .collect()
}
pub(crate) fn land_ids(income: &FederalIncome) -> Vec<i64> {
    let farm = match income {
        FederalIncome::Business { data } => match &data.activity {
            BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        FederalIncome::FarmRental { data } => Some(&data.farm),
        _ => None,
    };
    farm.map(|farm| {
        farm.land_basis
            .iter()
            .map(|land| land.id)
            .chain(farm.land_sales.iter().map(|sale| sale.id))
            .chain(farm.specials.production.iter().map(|cohort| cohort.id))
            .collect()
    })
    .unwrap_or_default()
}
pub(crate) fn parcel_ids(income: &FederalIncome) -> Vec<i64> {
    let farm = match income {
        FederalIncome::Business { data } => match &data.activity {
            BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        FederalIncome::FarmRental { data } => Some(&data.farm),
        _ => None,
    };
    farm.map(|farm| farm.land_basis.iter().map(|land| land.id).collect()).unwrap_or_default()
}
pub(crate) fn sale_ids(income: &FederalIncome) -> Vec<i64> {
    let farm = match income {
        FederalIncome::Business { data } => match &data.activity {
            BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        FederalIncome::FarmRental { data } => Some(&data.farm),
        _ => None,
    };
    farm.map(|farm| farm.land_sales.iter().map(|sale| sale.id).collect()).unwrap_or_default()
}
pub(crate) fn production_ids(income: &FederalIncome) -> Vec<i64> {
    let farm = match income {
        FederalIncome::Business { data } => match &data.activity {
            BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        FederalIncome::FarmRental { data } => Some(&data.farm),
        _ => None,
    };
    farm.map(|farm| farm.specials.production.iter().map(|cohort| cohort.id).collect())
        .unwrap_or_default()
}
pub(crate) fn production_ids_mut(income: &mut FederalIncome) -> Vec<&mut i64> {
    let farm = match income {
        FederalIncome::Business { data } => match &mut data.activity {
            BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        FederalIncome::FarmRental { data } => Some(&mut data.farm),
        _ => None,
    };
    farm.map(|farm| farm.specials.production.iter_mut().map(|cohort| &mut cohort.id).collect())
        .unwrap_or_default()
}
pub(crate) fn land_ids_mut(income: &mut FederalIncome) -> Vec<&mut i64> {
    let farm = match income {
        FederalIncome::Business { data } => match &mut data.activity {
            BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        FederalIncome::FarmRental { data } => Some(&mut data.farm),
        _ => None,
    };
    farm.map(|farm| farm.land_basis.iter_mut().map(|land| &mut land.id).collect())
        .unwrap_or_default()
}
pub(crate) fn sale_ids_mut(income: &mut FederalIncome) -> Vec<&mut i64> {
    let farm = match income {
        FederalIncome::Business { data } => match &mut data.activity {
            BusinessActivity::Farming { data } => Some(data),
            _ => None,
        },
        FederalIncome::FarmRental { data } => Some(&mut data.farm),
        _ => None,
    };
    farm.map(|farm| farm.land_sales.iter_mut().map(|sale| &mut sale.id).collect())
        .unwrap_or_default()
}

fn money(value: i64) -> bool {
    (0..=100_000_000_000).contains(&value)
}
fn current_date(value: u32) -> bool {
    crate::federal_rules::valid_date(value)
        && (TAX_YEAR_START_DATE..=TAX_YEAR_END_DATE).contains(&value)
}
fn retained_date(value: u32) -> bool {
    crate::federal_rules::valid_date(value) && (19000101..=TAX_YEAR_END_DATE).contains(&value)
}
fn text(value: &str) -> bool {
    !value.trim().is_empty()
}
impl FarmWeatherSale {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        for (field, invalid) in [
            ("all_sold", self.all_sold == 0 || self.all_sold > 1_000_000),
            ("normal_sold", self.normal_sold > 1_000_000),
            (
                "weather_excess",
                self.weather_excess > self.all_sold.saturating_sub(self.normal_sold),
            ),
            ("class_income", !money(self.class_income)),
        ] {
            if invalid {
                return Some(ValidationIssue::new("FARM_WEATHER_COUNTS").at_field(field));
            }
        }
        if matches!(self.source, FarmWeatherSource::MixedProducts { raised } if !(0..=self.class_income).contains(&raised))
        {
            return Some(
                ValidationIssue::new("FARM_WEATHER_INCOME_SPLIT")
                    .at_field("raised")
                    .in_field("source"),
            );
        }
        for (field, value) in [
            ("weather_relationship", &self.weather_relationship),
            ("usual_practice_support", &self.usual_practice_support),
        ] {
            if !text(value) {
                return Some(ValidationIssue::new("FARM_WEATHER_SOURCE").at_field(field));
            }
        }
        if let FarmWeatherAssistance::Designated { date: Some(date) } = self.assistance
            && (!crate::federal_rules::valid_date(date) || date < 19000101)
        {
            return Some(
                ValidationIssue::new("FARM_WEATHER_DESIGNATION_DATE")
                    .at_field("date")
                    .in_field("assistance"),
            );
        }
        for (i, record) in self.prior_sales.iter().enumerate() {
            let field = if !(TAX_YEAR - 3..TAX_YEAR).contains(&record.year) {
                Some("year")
            } else if record.count > 1_000_000 {
                Some("count")
            } else if self.prior_sales[..i].iter().any(|p| p.year == record.year) {
                Some("year")
            } else {
                None
            };
            if let Some(field) = field {
                return Some(
                    ValidationIssue::new("FARM_WEATHER_YEARS")
                        .at_field(field)
                        .in_list("prior_sales", i),
                );
            }
        }
        None
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn deferred(&self, cash: bool) -> i64 {
        if cash
            && self.elect
            && self.principal_farming
            && matches!(self.assistance, FarmWeatherAssistance::Designated { .. })
            && self.source.split(self.class_income).is_some()
            && self.rule().is_none()
        {
            crate::kernel::ratio::fraction(
                self.class_income,
                self.weather_excess.into(),
                self.all_sold.into(),
            )
        } else {
            0
        }
    }
}
impl FarmHedge {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        for (field, invalid) in [
            ("contract", !text(&self.contract)),
            ("business_risk", !text(&self.business_risk)),
            ("entered", !retained_date(self.entered)),
            ("settled", !current_date(self.settled)),
            ("identified", !retained_date(self.identified)),
            ("settled", self.settled < self.entered),
            ("settlement", !(-100_000_000_000..=100_000_000_000).contains(&self.settlement)),
        ] {
            if invalid {
                return Some(ValidationIssue::new("FARM_HEDGE_SOURCE").at_field(field));
            }
        }
        None
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn current_income(&self) -> Option<i64> {
        (self.rule().is_none()
            && current_date(self.entered)
            && self.identified == self.entered
            && matches!(self.timing, FarmHedgeTiming::CurrentSingleYear))
        .then_some(self.settlement)
    }
}
impl FarmSmallBusiness {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        let year = TAX_YEAR;
        if !(1900..=year).contains(&self.first_tax_year) {
            return Some(ValidationIssue::new("FARM263A_RECEIPT_YEARS").at_field("first_tax_year"));
        }
        let start =
            if self.first_tax_year == year { year } else { self.first_tax_year.max(year - 3) };
        let end = if start == year { year + 1 } else { year };
        if self.receipts.len() != (end - start) as usize {
            return Some(ValidationIssue::new("FARM263A_RECEIPT_YEARS").at_field("receipts"));
        }
        for (i, record) in self.receipts.iter().enumerate() {
            let field = if !(start..end).contains(&record.year) {
                Some("year")
            } else if !(1..=12).contains(&record.months) {
                Some("months")
            } else if !money(record.gross) {
                Some("gross")
            } else if self.receipts[..i].iter().any(|p| p.year == record.year) {
                Some("year")
            } else {
                None
            };
            if let Some(field) = field {
                return Some(
                    ValidationIssue::new("FARM263A_RECEIPT_YEARS")
                        .at_field(field)
                        .in_list("receipts", i),
                );
            }
        }
        None
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn qualifies(&self) -> bool {
        self.rule().is_none()
            && !self.tax_shelter
            && self
                .receipts
                .iter()
                .map(|record| {
                    // 27720 is divisible by every supported month count (1..=12).
                    // Compare the statutory average without rounding each short year.
                    crate::kernel::ratio::trunc_i128(
                        i128::from(record.gross) * 12 * 27720,
                        i128::from(record.months),
                    )
                })
                .sum::<i128>()
                <= i128::from(crate::year_constants::SMALL_BUSINESS_GROSS_RECEIPTS_LIMIT_CENTS)
                    * self.receipts.len() as i128
                    * 27720
    }
}

impl FarmSpecials {
    pub(crate) fn weather_lines(&self, cash: bool) -> [i64; 2] {
        let mut amounts = [0, 0];
        for prior in &self.prior_weather {
            if let Some(parts) = prior.source.split(prior.deferred) {
                for i in 0..2 {
                    amounts[i] += parts[i];
                }
            }
        }
        for current in &self.weather_sales {
            if let Some(parts) = current.source.split(current.class_income) {
                let deferred =
                    crate::kernel::ratio::allocate_remaining(current.deferred(cash), &parts);
                for i in 0..2 {
                    amounts[i] -= deferred[i];
                }
            }
        }
        amounts
    }
    pub(crate) fn other_income(&self) -> i64 {
        self.other_income
            .iter()
            .map(|item| match item {
                FarmOtherIncome::Hedge { facts } => facts.current_income().unwrap_or(0),
                FarmOtherIncome::Irrigation { amount } => amount.amount,
                FarmOtherIncome::Other { facts } => facts.amount,
            })
            .sum()
    }
    pub(crate) fn income_adjustment(&self, cash: bool) -> i64 {
        self.weather_lines(cash).iter().sum::<i64>()
            + self.other_income()
            + self.cost_sharing_income()
    }
}

impl FarmSpecials {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        let year = TAX_YEAR;
        for (field, len, max) in [
            ("weather_sales", self.weather_sales.len(), 10),
            ("prior_weather", self.prior_weather.len(), 20),
            ("other_income", self.other_income.len(), 100),
            ("cost_sharing", self.cost_sharing.len(), 100),
            ("production", self.production.len(), 100),
            ("fertilizer", self.fertilizer.len(), 100),
            ("external_sources", self.external_sources.len(), 100),
            ("limitations", self.limitations.len(), 10),
        ] {
            if len > max {
                return Some(ValidationIssue::new("FARM_SPECIALS_LIST_LIMIT").at_field(field));
            }
        }
        if !(1900..=year).contains(&self.method_status.adopted) {
            return Some(
                ValidationIssue::new("FARM_METHOD_ADOPTED_YEAR")
                    .at_field("adopted")
                    .in_field("method_status"),
            );
        }
        if let Some(source) = &self.method_status.change {
            let field = if source.change <= 0 || source.change > 100_000_000_000 {
                Some("change")
            } else if !text(&source.approval) {
                Some("approval")
            } else {
                None
            };
            if let Some(field) = field {
                return Some(
                    ValidationIssue::new("FARM_METHOD_SOURCE_ID")
                        .at_field(field)
                        .in_field("change")
                        .in_field("method_status"),
                );
            }
        }
        for (i, value) in self.limitations.iter().enumerate() {
            if !(1..=10).contains(value) || self.limitations[..i].contains(value) {
                return Some(
                    ValidationIssue::new("FARM_LIMITATIONS_INVALID").at_list_item("limitations", i),
                );
            }
        }
        if self.limitations.contains(&10) != self.other_situation.is_some()
            || self.other_situation.as_ref().is_some_and(|description| !text(description))
        {
            return Some(
                ValidationIssue::new("FARM_OTHER_SOURCE_REQUIRED").at_field("other_situation"),
            );
        }
        for (i, cost) in self.fertilizer.iter().enumerate() {
            if let Some(issue) = cost.issue() {
                return Some(issue.in_list("fertilizer", i));
            }
        }
        for (i, source) in self.cost_sharing.iter().enumerate() {
            if let Some(issue) = source.issue() {
                return Some(issue.in_list("cost_sharing", i));
            }
        }
        for (i, sale) in self.weather_sales.iter().enumerate() {
            if let Some(issue) = sale.issue() {
                return Some(issue.in_list("weather_sales", i));
            }
            if self.weather_sales[..i].iter().any(|other| other.class == sale.class) {
                return Some(
                    ValidationIssue::new("FARM_WEATHER_DUPLICATE_CLASS")
                        .at_field("class")
                        .in_list("weather_sales", i),
                );
            }
        }
        for (i, prior) in self.prior_weather.iter().enumerate() {
            let field = if prior.year != year - 1 {
                Some("year")
            } else if !money(prior.deferred) {
                Some("deferred")
            } else if matches!(prior.source, FarmWeatherSource::MixedProducts { raised } if !(0..=prior.deferred).contains(&raised))
            {
                Some("source")
            } else if self.prior_weather[..i].iter().any(|other| other.class == prior.class) {
                Some("class")
            } else {
                None
            };
            if let Some(field) = field {
                return Some(
                    ValidationIssue::new("FARM_WEATHER_YEARS")
                        .at_field(field)
                        .in_list("prior_weather", i),
                );
            }
        }
        for (i, cohort) in self.production.iter().enumerate() {
            if let Some(issue) = cohort.production_issue() {
                return Some(issue.in_list("production", i));
            }
        }
        for (i, item) in self.other_income.iter().enumerate() {
            match item {
                FarmOtherIncome::Hedge { facts } => {
                    if let Some(issue) = facts.issue() {
                        return Some(issue.in_field("facts").in_list("other_income", i));
                    }
                }
                FarmOtherIncome::Irrigation { amount } if !money(amount.amount) => {
                    return Some(
                        ValidationIssue::new("FARM_OTHER_SOURCE")
                            .at_field("amount")
                            .in_field("amount")
                            .in_list("other_income", i),
                    );
                }
                FarmOtherIncome::Other { facts } => {
                    let field = if !text(&facts.description) {
                        Some("description")
                    } else if !(-100_000_000_000..=100_000_000_000).contains(&facts.amount) {
                        Some("amount")
                    } else {
                        None
                    };
                    if let Some(field) = field {
                        return Some(
                            ValidationIssue::new("FARM_OTHER_SOURCE")
                                .at_field(field)
                                .in_field("facts")
                                .in_list("other_income", i),
                        );
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn source_limitations(&self) -> [(u32, &'static str, &'static str); 7] {
        [
            (
                1,
                "ISSUE_FARM_FOREIGN_SOURCE",
                "Foreign farm currency, sourcing and cross-border adjustments need their actual income and tax source owner; known domestic-style receipts do not establish those adjustments.",
            ),
            (
                2,
                "ISSUE_FARM_ENTITY_SOURCE",
                "An entity or farming syndicate must use its actual entity/K-1 and required accounting owner, rather than a direct individual Schedule F by assumption.",
            ),
            (
                3,
                "ISSUE_FARM_METHOD_TRANSITION_SOURCE",
                "The unentered accounting transition needs its actual consent, section481 adjustment and current timing records; changing the accounting-method answer does not compute it.",
            ),
            (
                4,
                "ISSUE_FARM126_ECONOMIC_VALUE_SOURCE",
                "The improvement's unfinished economic fair market value prevents required section126 payment income from being known. Missing optional income-right appraisal alone instead omits only the exclusion on its typed improvement record.",
            ),
            (
                6,
                "ISSUE_FARM_INTEREST_ALLOCATION_SOURCE",
                "Unresolved indirect interest allocation needs its actual current expense, capitalization and property/inventory source; it cannot be assumed wholly deductible or wholly absent.",
            ),
            (
                8,
                "ISSUE_FARM_HEDGE_ALLOCATION_SOURCE",
                "The complex hedge needs actual matched income/cost character and timing. A settlement amount alone does not establish current required income.",
            ),
            (
                10,
                "ISSUE_FARM_ADDITIONAL_SOURCE",
                "The additional described legal source has unresolved required income, expense timing or property history. Retain the known receipts and resolve that named source before relying on a completed tax estimate.",
            ),
        ]
    }
    fn limitation_available(&self, source: u32) -> bool {
        !(self.limitations.contains(&source)
            || (source == 2
                && matches!(self.method_status.original, FarmOriginalMethod::EntitySyndicate)))
    }
    fn source_timing(&self, cash: bool) -> [(bool, &'static str, &'static str); 4] {
        [
            (self.weather_sales.iter().all(|sale| sale.source.split(sale.class_income).is_some() || sale.class_income == 0), "ISSUE_FARM_WEATHER_CURRENT_CHARACTER", "Current class income from business-use livestock needs its retained Form 4797 character, basis and asset source. It cannot be omitted as an unclaimed postponement or assumed to be ordinary product income."),
            (self.prior_weather.iter().all(|prior| prior.source.split(prior.deferred).is_some()), "ISSUE_FARM_WEATHER_PRIOR_CHARACTER", "Prior postponed income from business-use livestock needs its retained Form4797 character and asset source. It is not ordinary product income by assumption."),
            (self.other_income.iter().all(|item| !matches!(item, FarmOtherIncome::Hedge { facts } if facts.current_income().is_none())), "ISSUE_FARM_HEDGE_TIMING_CHARACTER", "An unidentified, inventory-matched or multiyear hedge needs its actual income character and timing owner. A realized amount alone does not establish ordinary current income or an investor60/40 result."),
            (self.production.iter().all(|cohort| cohort.production_projection(cash, false).is_some() && cohort.production_projection(cash, true).is_some()), "ISSUE_FARM_PRODUCTION_BASIS_OWNER", "Preproductive costs need the linked inventory or depreciable property's regular and AMT basis, release and recovery facts; a paid cost cannot be both expensed and capitalized."),

        ]
    }
    pub(crate) fn before179_source_issue(&self, cash: bool) -> Option<&'static str> {
        self.source_limitations()
            .into_iter()
            .find(|(source, _, _)| !self.limitation_available(*source))
            .map(|(_, key, _)| key)
            .or_else(|| {
                self.source_timing(cash)
                    .into_iter()
                    .find(|(available, _, _)| !available)
                    .map(|(_, key, _)| key)
            })
    }

    pub(crate) fn report(
        &self,
        cash: bool,
        id: i64,
        form: crate::farm::FarmForm,
        context: IssueContext,
        r: &mut FederalResult,
    ) {
        self.report_cost_sharing(id, form, &context, r);
        crate::farm_production::report(self, cash, id, form, &context, r);
        for sale in &self.weather_sales {
            if sale.elect && sale.deferred(cash) == 0 && sale.weather_excess > 0 {
                crate::federal_rules::note_once_scoped(
                    r,
                    msg!("NOTE_FARM_WEATHER_NOT_POSTPONED"),
                    context.clone(),
                );
            }
        }
        let current = self.weather_sales.iter().map(|sale| sale.deferred(cash)).sum::<i64>();
        let prior = self
            .prior_weather
            .iter()
            .filter(|prior| prior.source.split(prior.deferred).is_some())
            .map(|prior| prior.deferred)
            .sum::<i64>();
        for (suffix, amount, key) in [
            ("weather_postponed", current, "EXPLAIN_FARM_WEATHER_POSTPONED"),
            ("weather_prior_included", prior, "EXPLAIN_FARM_WEATHER_PRIOR_INCLUDED"),
        ] {
            if amount != 0 {
                crate::kernel::lines::line_cents(
                    r,
                    &alloc::format!("farm_{id}_{suffix}"),
                    amount,
                    &[&form.input("specials")],
                    "https://www.irs.gov/publications/p225",
                    Message::new(key, alloc::vec![]),
                );
            }
        }
        if self.fertilizer.iter().any(|cost| cost.uncomputed_basis()) {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("NOTE_FARM_FERTILIZER_RECOVERY_OMITTED"),
                context.clone(),
            );
        }
        if self.fertilizer.iter().any(|cost| {
            matches!(cost.treatment, FarmFertilizerTreatment::Deduct180) && !cost.eligible180()
        }) {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("NOTE_FARM_FERTILIZER_180_NOT_ELIGIBLE"),
                context.clone(),
            );
        }
        for (source, key, english) in self.source_limitations() {
            crate::require_localized!(
                r,
                self.limitation_available(source),
                &context,
                "Farm source boundary",
                key,
                english,
            );
            #[cfg(not(test))]
            let _ = english;
        }
        for (available, key, english) in self.source_timing(cash) {
            crate::require_localized!(
                r,
                available,
                &context,
                "Farm source timing and character",
                key,
                english,
            );
            #[cfg(not(test))]
            let _ = english;
        }
    }
}
impl FarmIncome {
    pub(crate) fn specials_issue(&self) -> Option<ValidationIssue> {
        if let Some(issue) = self.insurance.source_issue() {
            return Some(issue.in_field("insurance"));
        }
        if let Some(issue) = self.ccc.as_ref().and_then(crate::FarmCcc::source_issue) {
            return Some(issue.in_field("ccc"));
        }
        if let Some(supplies) = &self.supplies {
            for (i, advance) in supplies.advances.iter().enumerate() {
                if advance.purchase.refunded > advance.amount {
                    return Some(
                        ValidationIssue::new("FARM_SUPPLY_REFUND_ABOVE_PAYMENT")
                            .at_field("refunded")
                            .in_field("purchase")
                            .in_list("advances", i)
                            .in_field("supplies"),
                    );
                }
                if let Some(issue) = advance.purchase.issue(advance.amount) {
                    return Some(
                        issue.in_field("purchase").in_list("advances", i).in_field("supplies"),
                    );
                }
            }
        }
        if let Some(issue) = self.specials.issue() {
            return Some(issue.in_field("specials"));
        }
        if let Some(conservation) = &self.conservation {
            if let Some(issue) = conservation.plan_reference_issue() {
                return Some(issue.in_field("conservation"));
            }
            if let Some(issue) = conservation.parcel_issue(self) {
                return Some(issue.in_field("conservation"));
            }
        }
        if self.land_basis.len() > 400 {
            return Some(ValidationIssue::new("FARM_LAND_BASIS_SOURCE").at_field("land_basis"));
        }
        for (i, land) in self.land_basis.iter().enumerate() {
            for (field, invalid) in [
                ("id", land.id < 0 || land.id > 100_000_000_000),
                ("label", !text(&land.label)),
                ("acquired", !retained_date(land.acquired)),
                (
                    "prior_conservation",
                    land.acquired >= TAX_YEAR_START_DATE && land.prior_conservation > 0,
                ),
                (
                    "prior_amt_conservation",
                    land.acquired >= TAX_YEAR_START_DATE && land.prior_amt_conservation > 0,
                ),
                ("regular", !money(land.regular)),
                ("amt", !money(land.amt)),
                ("prior_conservation", !money(land.prior_conservation)),
                ("prior_amt_conservation", !money(land.prior_amt_conservation)),
                ("prior_cost_sharing", land.prior_cost_sharing.len() > 100),
            ] {
                if invalid {
                    return Some(
                        ValidationIssue::new("FARM_LAND_BASIS_SOURCE")
                            .at_field(field)
                            .in_list("land_basis", i),
                    );
                }
            }
            for (j, prior) in land.prior_cost_sharing.iter().enumerate() {
                let field = if !retained_date(prior.received)
                    || prior.received >= TAX_YEAR_START_DATE
                    || prior.received < land.acquired
                {
                    Some("received")
                } else if !money(prior.excluded) {
                    Some("excluded")
                } else if !text(&prior.statement) {
                    Some("statement")
                } else {
                    None
                };
                if let Some(field) = field {
                    return Some(
                        ValidationIssue::new("FARM_LAND_BASIS_SOURCE")
                            .at_field(field)
                            .in_list("prior_cost_sharing", j)
                            .in_list("land_basis", i),
                    );
                }
            }
            if land.id > 0 && self.land_basis[..i].iter().any(|prior| prior.id == land.id) {
                return Some(
                    ValidationIssue::new("FARM_LAND_BASIS_SOURCE")
                        .at_field("id")
                        .in_list("land_basis", i),
                );
            }
        }
        if let Some((i, _)) = self.land_sales.iter().enumerate().find(|(i, sale)| {
            self.land_sales[..*i].iter().any(|prior| prior.basis_property == sale.basis_property)
        }) {
            return Some(
                ValidationIssue::new("FARM_LAND_DUPLICATE_PARCEL")
                    .at_field("basis_property")
                    .in_list("land_sales", i),
            );
        }
        for (i, sale) in self.land_sales.iter().enumerate() {
            let Some(parcel) =
                self.land_basis.iter().find(|p| p.id > 0 && p.id == sale.basis_property)
            else {
                return Some(
                    ValidationIssue::new("FARM126_PROPERTY_SOURCE")
                        .at_field("basis_property")
                        .in_list("land_sales", i),
                );
            };
            if parcel.acquired > sale.sold {
                return Some(
                    ValidationIssue::new("FARM_LAND_SOLD_BEFORE_ACQUIRED")
                        .at_field("sold")
                        .in_list("land_sales", i),
                );
            }
        }
        if let Some((i, _)) = self.specials.cost_sharing.iter().enumerate().find(|(_, source)| {
            !self.land_basis.iter().any(|parcel| {
                parcel.id > 0 && parcel.id == source.asset && parcel.acquired <= source.received
            })
        }) {
            return Some(
                ValidationIssue::new("FARM126_PROPERTY_SOURCE")
                    .at_field("asset")
                    .in_list("cost_sharing", i)
                    .in_field("specials"),
            );
        }
        if let FarmAccounting::Cash { receipts } = &self.accounting {
            let raised = receipts
                .iter()
                .filter(|r| matches!(r, FarmReceipt::Raised { .. }))
                .map(|receipt| i128::from(receipt.amount()))
                .sum::<i128>();
            let purchased = receipts
                .iter()
                .filter(|r| matches!(r, FarmReceipt::Resale { .. }))
                .map(|r| match r {
                    FarmReceipt::Resale { amount, cost } => {
                        phase::floor(i128::from(*amount) - i128::from(*cost), 0)
                    }
                    _ => 0,
                })
                .sum::<i128>();
            let totals = self
                .specials
                .weather_sales
                .iter()
                .filter_map(|sale| sale.source.split(sale.class_income))
                .fold([0i128; 2], |a, b| [a[0] + i128::from(b[0]), a[1] + i128::from(b[1])]);
            if totals[0] > raised || totals[1] > purchased {
                return Some(
                    ValidationIssue::new("FARM_WEATHER_ABOVE_PRODUCT_INCOME")
                        .at_field("weather_sales")
                        .in_field("specials"),
                );
            }
        }
        None
    }
    pub(crate) fn specials_rule(&self) -> Option<&'static str> {
        self.specials_issue().map(|issue| issue.key)
    }
}

// Section 126's economic benefit is not the government cash less income.
// Reg.16A.126-1(b),(c),(e); IRS Pub225 chapter3. Keep the qualified
// improvement calculation separate from other government payment character.
pub(crate) struct Farm126Amounts {
    value: i64,
    taxable: i64,
    pub(crate) excludable: i64,
    basis: Option<i64>,
    pub(crate) valuation_missing: bool,
}
impl FarmCostSharing {
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if self.total_cost == 0 {
            return Some(ValidationIssue::new("FARM126_COST_REQUIRED").at_field("total_cost"));
        }
        for (field, value) in [
            ("government_received", self.government_received),
            ("total_cost", self.total_cost),
            ("owner_contribution", self.owner_contribution),
            ("fmv_improvement", self.fmv_improvement),
            ("non126_program", self.non126_program),
            ("uncertified", self.uncertified),
            ("rent_services", self.rent_services),
            ("current_deductible", self.current_deductible),
        ] {
            if !money(value) {
                return Some(ValidationIssue::new("FARM126_COST_REQUIRED").at_field(field));
            }
        }
        for (field, value) in [
            ("owner_contribution", self.owner_contribution),
            ("government_received", self.government_received),
        ] {
            if value > self.total_cost {
                return Some(ValidationIssue::new("FARM126_OWNER_ABOVE_COST").at_field(field));
            }
        }
        if self.non126_program + self.uncertified + self.rent_services + self.current_deductible
            > self.total_cost
        {
            return Some(
                ValidationIssue::new("FARM126_COMPONENTS_ABOVE_COST")
                    .at_field("current_deductible"),
            );
        }
        if self.excluded_funding_applied.is_some_and(|amount| {
            !money(amount) || amount > self.total_cost || amount > self.government_received
        }) {
            return Some(
                ValidationIssue::new("FARM126_EXCLUDED_FUNDING_ABOVE_SOURCE")
                    .at_field("excluded_funding_applied"),
            );
        }
        if self.excluded_funding_applied.is_some_and(|amount| amount > 0)
            && (!self.program_approved
                || !self.conservation_certified
                || matches!(self.election, Farm126Election::NotExcludeAll))
        {
            return Some(
                ValidationIssue::new("FARM126_FUNDING_WITHOUT_EXCLUSION")
                    .at_field("excluded_funding_applied"),
            );
        }
        for (field, invalid) in [
            ("installation_year", !(1900..=TAX_YEAR).contains(&self.installation_year)),
            ("received", !current_date(self.received)),
            ("installation_year", self.installation_year > self.received / 10000),
            ("description", !text(&self.description)),
            ("program", !text(&self.program)),
            ("asset", self.asset <= 0 || self.asset > 100_000_000_000),
        ] {
            if invalid {
                return Some(ValidationIssue::new("FARM126_SOURCE_REQUIRED").at_field(field));
            }
        }
        if let Some(facts) = self.election.facts() {
            let first = self.installation_year;
            if !(1..=100_000_000).contains(&facts.acres_hundredths) {
                return Some(
                    ValidationIssue::new("FARM126_PRIOR_INCOME_YEARS")
                        .at_field("acres_hundredths")
                        .in_field("facts")
                        .in_field("election"),
                );
            }
            if facts.prior_income.len() != 3 {
                return Some(
                    ValidationIssue::new("FARM126_PRIOR_INCOME_YEARS")
                        .at_field("prior_income")
                        .in_field("facts")
                        .in_field("election"),
                );
            }
            for (i, record) in facts.prior_income.iter().enumerate() {
                let field = if !(first.saturating_sub(3)..first).contains(&record.year) {
                    Some("year")
                } else if !money(record.income) {
                    Some("income")
                } else if facts.prior_income[..i].iter().any(|prior| prior.year == record.year) {
                    Some("year")
                } else {
                    None
                };
                if let Some(field) = field {
                    return Some(
                        ValidationIssue::new("FARM126_PRIOR_INCOME_YEARS")
                            .at_field(field)
                            .in_list("prior_income", i)
                            .in_field("facts")
                            .in_field("election"),
                    );
                }
            }
            if let Some(value) = &facts.valuation {
                let field = if !money(value.ten_percent_value) {
                    Some("ten_percent_value")
                } else if !money(value.acre_value) {
                    Some("acre_value")
                } else if !text(&value.support) {
                    Some("support")
                } else {
                    None
                };
                if let Some(field) = field {
                    return Some(
                        ValidationIssue::new("FARM126_VALUATION_SOURCE")
                            .at_field(field)
                            .in_field("valuation")
                            .in_field("facts")
                            .in_field("election"),
                    );
                }
            }
            if let Farm126Election::NotExcludePart { amount, .. } = self.election
                && (!money(amount)
                    || facts.valuation.as_ref().is_some_and(|value| {
                        amount > value.ten_percent_value.max(value.acre_value)
                    }))
            {
                return Some(
                    ValidationIssue::new("FARM126_PARTIAL_ELECTION_AMOUNT")
                        .at_field("amount")
                        .in_field("election"),
                );
            }
        }
        None
    }

    fn qualified_amounts(&self) -> Farm126Amounts {
        let section_cost = self.total_cost
            - self.non126_program
            - self.uncertified
            - self.rent_services
            - self.current_deductible;
        let value = crate::kernel::ratio::fraction(
            self.fmv_improvement,
            section_cost,
            self.total_cost.max(1),
        );
        let available = self
            .election
            .facts()
            .and_then(|facts| facts.valuation.as_ref())
            .map(|value| phase::floor(value.ten_percent_value, value.acre_value));
        let excludable = match &self.election {
            Farm126Election::NotExcludeAll => 0,
            Farm126Election::Exclude { .. } => available.unwrap_or(0),
            Farm126Election::NotExcludePart { amount, .. } => {
                available.unwrap_or(0).saturating_sub(*amount)
            }
        };
        let taxable = phase::floor(value - self.owner_contribution - excludable, 0);
        let valuation_missing =
            available.is_none() && !matches!(self.election, Farm126Election::NotExcludeAll);
        Farm126Amounts {
            value,
            taxable,
            excludable,
            basis: (!valuation_missing).then_some(self.owner_contribution + taxable),
            valuation_missing,
        }
    }
    // The existing wholly capital, fully financed worksheet has its own
    // supported basis calculation. Mixed funding additionally needs the actual
    // excluded funding applied; the income-right present value is not that fact.
    fn simple_improvement(&self) -> bool {
        self.government_received + self.owner_contribution == self.total_cost
            && self.non126_program == 0
            && self.uncertified == 0
            && self.rent_services == 0
            && self.current_deductible == 0
    }
    pub(crate) fn current_amounts(&self) -> Option<Farm126Amounts> {
        if !self.simple_improvement() {
            let excluded_funding = self.excluded_funding_applied?;
            let mut amounts = if self.program_approved && self.conservation_certified {
                let mut amounts = self.qualified_amounts();
                // Reg.16A.126-1(b),(g): separately identified payment
                // categories do not disappear from taxable income when the
                // qualified improvement's economic value is calculated.
                amounts.taxable += self.non126_program
                    + self.uncertified
                    + self.rent_services
                    + self.current_deductible;
                amounts
            } else {
                Farm126Amounts {
                    value: self.fmv_improvement,
                    taxable: self.government_received,
                    excludable: 0,
                    basis: None,
                    valuation_missing: false,
                }
            };
            // Dispatcher-approved actual funding source, separately bounded on
            // both client and Save. Never substitute the income-right appraisal
            // or government cash minus taxable improvement value for this fact.
            amounts.basis = Some(self.total_cost - excluded_funding);
            return Some(amounts);
        }
        if self.program_approved && self.conservation_certified {
            Some(self.qualified_amounts())
        } else {
            // Ordinary taxable program financing: Pub225, Other Payments,
            // Improvements. Capitalize full actual improvement cost.
            Some(Farm126Amounts {
                value: self.fmv_improvement,
                taxable: self.government_received,
                excludable: 0,
                basis: Some(self.total_cost),
                valuation_missing: false,
            })
        }
    }
}
impl FarmSpecials {
    pub(crate) fn cost_sharing_income(&self) -> i64 {
        self.cost_sharing
            .iter()
            .filter_map(FarmCostSharing::current_amounts)
            .map(|amounts| amounts.taxable)
            .sum()
    }
    pub(crate) fn cost_sharing_gross(&self) -> i64 {
        self.cost_sharing.iter().map(|source| source.government_received).sum()
    }
    fn report_cost_sharing(
        &self,
        id: i64,
        form: crate::farm::FarmForm,
        context: &IssueContext,
        r: &mut FederalResult,
    ) {
        for (index, source) in self.cost_sharing.iter().enumerate() {
            let Some(amounts) = source.current_amounts() else {
                crate::require_localized!(
                    r,
                    false,
                    context,
                    "Section 126 payment character and basis",
                    "ISSUE_FARM126_MIXED_SOURCE_OWNER",
                    "Mixed payment character or incomplete improvement funding needs the actual excluded cost-share funding applied to this improvement. Supply that source amount; the income-right present value does not establish the funding allocation.",
                );
                continue;
            };
            if amounts.valuation_missing {
                crate::federal_rules::note_once_scoped(
                    r,
                    msg!("NOTE_FARM126_VALUATION_OMITTED"),
                    context.clone(),
                );
            }
            let rows = [
                ("value", Some(amounts.value), "EXPLAIN_FARM126_VALUE"),
                ("taxable", Some(amounts.taxable), "EXPLAIN_FARM126_TAXABLE"),
                (
                    "recapture_portion",
                    (!amounts.valuation_missing).then_some(amounts.excludable),
                    "EXPLAIN_FARM126_RECAPTURE_PORTION",
                ),
                ("basis", amounts.basis, "EXPLAIN_FARM126_BASIS"),
            ];
            for (name, amount, key) in rows {
                if let Some(amount) = amount {
                    crate::kernel::lines::line_cents(
                        r,
                        &alloc::format!("farm_{id}_costsharing_{index}_{name}"),
                        amount,
                        &[&form.input("specials.cost_sharing")],
                        "https://www.irs.gov/publications/p225",
                        Message::new(key, alloc::vec![]),
                    );
                }
            }
        }
    }
}
impl FarmIncome {
    pub(crate) fn parcel_basis(&self, property: i64, amt: bool) -> Option<i64> {
        // An actual revised assessment or unresolved basis/ownership source
        // must be fitted before any affected parcel disposition is known.
        // These current named sources have not identified a narrower parcel;
        // never silently allocate their missing basis among entered parcels.
        if self.specials.limitations.iter().any(|source| matches!(source, 5 | 7)) {
            return None;
        }
        // Unattributed capital fertilizer recovery must be fitted to its
        // actual property before a current sale's adjusted basis is known.
        if self.specials.fertilizer.iter().any(FarmFertilizerCost::uncomputed_basis) {
            return None;
        }
        let parcel = self.land_basis.iter().find(|parcel| parcel.id == property && property > 0)?;
        let mut basis = if amt { parcel.amt } else { parcel.regular };
        for source in self.specials.cost_sharing.iter().filter(|source| source.asset == property) {
            basis += source.current_amounts()?.basis?;
        }
        Some(basis)
    }
}

impl FarmIncome {
    pub(crate) fn report_parcels(
        &self,
        draft: &FederalDraft,
        id: i64,
        form: crate::farm::FarmForm,
        r: &mut FederalResult,
    ) {
        for parcel in &self.land_basis {
            if !self.specials.cost_sharing.iter().any(|source| source.asset == parcel.id)
                && crate::cooperative::land_adjustments(draft, id, parcel.id).0 == 0
            {
                continue;
            }
            for (name, amt) in [("regular", false), ("amt", true)] {
                if let Some(basis) = self.parcel_basis(parcel.id, amt) {
                    let basis =
                        basis - crate::cooperative::land_adjustments(draft, id, parcel.id).0;
                    crate::kernel::lines::line_cents(
                        r,
                        &alloc::format!("farm_{id}_landbasis_{}_{}", parcel.id, name),
                        basis,
                        &[&form.input("land_basis"), &form.input("specials.cost_sharing")],
                        "https://www.irs.gov/publications/p225",
                        msg!("EXPLAIN_FARM_PARCEL_BASIS"),
                    );
                }
            }
        }
    }
}

impl FarmFertilizerCost {
    fn eligible180(&self) -> bool {
        !self.initial_preparation
            && matches!(
                self.land_use,
                FarmFertilizerLand::PreviouslyFarmed | FarmFertilizerLand::SimultaneouslyFarmed
            )
    }
    fn uncomputed_basis(&self) -> bool {
        self.cost > 0
            && self.land_use != FarmFertilizerLand::Nonfarming
            && (matches!(&self.treatment,FarmFertilizerTreatment::Capitalize{recovery} if recovery.prior_recovered<self.cost)
                || !self.eligible180())
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        for (field, invalid) in [
            ("description", !text(&self.description)),
            ("cost", !money(self.cost)),
            ("applied", !retained_date(self.applied)),
            ("paid_or_incurred", !retained_date(self.paid_or_incurred)),
        ] {
            if invalid {
                return Some(ValidationIssue::new("FARM_FERTILIZER_SOURCE").at_field(field));
            }
        }
        if let FarmFertilizerTreatment::Capitalize { recovery } = &self.treatment {
            let field = if !(1..=1200).contains(&recovery.benefit_months) {
                Some("benefit_months")
            } else if !(0..=self.cost).contains(&recovery.prior_recovered)
                || (self.paid_or_incurred >= TAX_YEAR_START_DATE && recovery.prior_recovered > 0)
            {
                Some("prior_recovered")
            } else {
                None
            };
            if let Some(field) = field {
                return Some(
                    ValidationIssue::new("FARM_FERTILIZER_SOURCE")
                        .at_field(field)
                        .in_field("recovery")
                        .in_field("treatment"),
                );
            }
        }
        None
    }
}
impl FarmSpecials {
    pub(crate) fn fertilizer_deduction(&self) -> i64 {
        self.fertilizer
            .iter()
            .filter(|cost| {
                cost.eligible180()
                    && matches!(cost.treatment, FarmFertilizerTreatment::Deduct180)
                    && current_date(cost.paid_or_incurred)
            })
            .map(|cost| cost.cost)
            .sum()
    }
}

impl FarmIncome {
    pub(crate) fn report_property_history(
        &self,
        assets: &[crate::BusinessAsset],
        context: crate::IssueContext,
        result: &mut FederalResult,
    ) {
        crate::require_localized!(
            result,
            !self.specials.limitations.contains(&7)
                || assets.iter().all(|asset| matches!(asset.disposal, crate::AssetDisposal::No)),
            &context,
            "Farm property disposition history",
            "ISSUE_FARM_PROPERTY_DISPOSITION_HISTORY",
            "This actual property disposition has unresolved basis or ownership history. Its required gain cannot be removed as an unclaimed property deduction.",
        );
    }
}

record_page_conversions!(crate::ConservationCost, ConservationCostInput, [&], |value, Target| {
    Target {
        property: value.property,
        label: value.label.clone(),
        amount: value.amount,
        facts: value.facts.clone(),
        paid_or_incurred: value.paid_or_incurred,
    }
});

record_page_conversions!(crate::ConservationAssessment, ConservationAssessmentInput, [&], |value, Target| {
    Target {
        property: value.property,
        label: value.label.clone(),
        total: value.total,
        share: value.share,
        payments: value.payments.clone(),
        facts: value.facts.clone(),
    }
});

impl FarmSupplyPurchase {
    pub(crate) fn issue(&self, amount: i64) -> Option<ValidationIssue> {
        let binding = matches!(self.kind, FarmSupplyPaymentKind::BindingPurchase);
        for (field, invalid) in [
            ("quantity_price_terms", binding && self.quantity_price_terms.is_empty()),
            ("business_purpose", binding && self.business_purpose.is_empty()),
            ("refunded", !(0..=amount).contains(&self.refunded)),
            (
                "paid",
                !(TAX_YEAR_START_DATE..=TAX_YEAR_END_DATE).contains(&self.paid)
                    || !crate::federal_rules::valid_date(self.paid),
            ),
        ] {
            if invalid {
                return Some(ValidationIssue::new("FARM_SUPPLY_PURCHASE_TERMS").at_field(field));
            }
        }
        None
    }
    pub(crate) fn valid(&self, amount: i64) -> bool {
        self.issue(amount).is_none()
    }
    pub(crate) fn eligible(&self) -> bool {
        matches!(self.kind, FarmSupplyPaymentKind::BindingPurchase) && !self.material_distortion
    }
}

// Community-property projections preserve accepted source identity and facts.
crate::community::divide_record!(FarmConservationCarry {
    amounts: [regular, amt],
    gains: [],
    facts: [property, year],
});

crate::community::divide_record!(FarmConservationFacts {
    amounts: [plan, work],
    gains: [],
    facts: [land_farmed, plan_reference],
});

crate::community::divide_enum!(FarmConservationWork {
    records: [],
    units: [
        QualifyingLandWork,
        EndangeredRecovery,
        WetlandDrainage,
        PivotPreparation,
        OwnerDepreciable,
        Maintenance
    ],
    tuples: [],
});

crate::community::divide_enum!(FarmConservationPlan {
    records: [],
    units: [Nrcs, ComparableState, EndangeredRecovery, None],
    tuples: [],
});

crate::community::divide_record!(FarmProductionPoultryCurrent {
    amounts: [cost, adopted],
    gains: [],
    facts: [label, paid, started, useful_months],
});

crate::community::divide_record!(FarmSupplyPurchase {
    amounts: [kind, refunded],
    gains: [],
    facts: [quantity_price_terms, business_purpose, material_distortion, paid],
});

crate::community::divide_enum!(FarmSupplyPaymentKind {
    records: [],
    units: [BindingPurchase, Deposit, UnresolvedTerms],
    tuples: [],
});

crate::community::divide_record!(CccSourceFacts {
    amounts: [unrecovered_production_cost, history],
    gains: [],
    facts: [producer_owned_raised],
});

crate::community::divide_enum!(CccElectionHistory {
    records: [
        IncomeElection { amounts: [], facts: [year, statement] },
        ChangedMethod { amounts: [], facts: [year, authority] }
    ],
    units: [NoIncomeElection],
    tuples: [],
});

crate::community::divide_record!(FarmLandBasis {
    amounts: [
        acquisition,
        regular,
        amt,
        prior_conservation,
        prior_amt_conservation,
        prior_cost_sharing
    ],
    gains: [],
    facts: [id, label, acquired],
});

crate::community::divide_record!(Farm126History {
    amounts: [excluded],
    gains: [],
    facts: [received, statement],
});

crate::community::divide_record!(FarmSpecials {
    amounts: [
        cost_sharing,
        weather_sales,
        prior_weather,
        other_income,
        production,
        fertilizer,
        external_sources
    ],
    gains: [],
    facts: [method_status, limitations, other_situation],
});

crate::community::divide_record!(FarmExternalSource {
    amounts: [character, gross_sales, product_cost, agreement],
    gains: [],
    facts: [source, shareholder_employee, regularly_used, ceased],
});

crate::community::divide_record!(FarmShareAgreement {
    amounts: [],
    gains: [],
    facts: [signed, significant_activity, production_based],
});

crate::community::divide_enum!(FarmSourceCharacter {
    records: [],
    units: [
        FarmK1,
        FishingK1,
        ScorpFarm,
        ScorpWages,
        FishingCrewShare,
        FishingCrewWages,
        CropShare,
        VesselShare,
        FarmProperty,
        Other
    ],
    tuples: [],
});

crate::community::divide_record!(FarmFertilizerCost {
    amounts: [cost, land_use, treatment],
    gains: [],
    facts: [description, paid_or_incurred, initial_preparation, applied],
});

crate::community::divide_enum!(FarmFertilizerTreatment {
    records: [Capitalize { amounts: [recovery], facts: [] }],
    units: [Deduct180],
    tuples: [],
});

crate::community::divide_record!(FarmFertilizerRecovery {
    amounts: [prior_recovered],
    gains: [],
    facts: [benefit_months],
});

crate::community::divide_enum!(FarmFertilizerLand {
    records: [],
    units: [PreviouslyFarmed, SimultaneouslyFarmed, NotYetFarmed, Nonfarming],
    tuples: [],
});

crate::community::divide_record!(FarmProductionCohort {
    amounts: [tax_treatment, costs, prior_basis, prior_amt_basis, destination],
    gains: [],
    facts: [id, description, property, began, productive, normal_preproductive_months],
});

crate::community::divide_enum!(FarmBasisDestination {
    records: [
        Inventory { amounts: [released, amt_released], facts: [] },
        Asset { amounts: [link], facts: [] }
    ],
    units: [UnresolvedTransfer],
    tuples: [],
});

crate::community::divide_record!(FarmAssetLink { amounts: [], gains: [], facts: [asset] });

crate::community::divide_record!(FarmProductionCost {
    amounts: [
        category,
        treatment,
        cost,
        non263a_capital,
        non263a_amt_capital,
        capital_allocated,
        amt_capital_allocated,
        kind
    ],
    gains: [],
    facts: [description],
});

crate::community::divide_enum!(FarmProductionCostKind {
    records: [],
    units: [Direct, IndirectAllocated, InterestOwner],
    tuples: [],
});

crate::community::divide_enum!(FarmCapitalization {
    records: [
        SmallBusiness { amounts: [], facts: [facts] },
        ShortPlant { amounts: [], facts: [required_accrual_entity_or_shelter] },
        AnimalException { amounts: [], facts: [required_accrual_entity_or_shelter] },
        OriginalElectionOut { amounts: [year], facts: [] }
    ],
    units: [Required, UnresolvedAllocation],
    tuples: [],
});

crate::community::divide_record!(FarmElectionYear {
    amounts: [],
    gains: [],
    facts: [year, statement],
});

crate::community::divide_enum!(FarmOtherIncome {
    records: [
        Hedge { amounts: [facts], facts: [] },
        Irrigation { amounts: [amount], facts: [] },
        Other { amounts: [facts], facts: [] }
    ],
    units: [],
    tuples: [],
});

crate::community::divide_record!(FarmSignedReceipt {
    amounts: [amount],
    gains: [],
    facts: [description],
});

crate::community::divide_record!(FarmPositiveReceipt { amounts: [amount], gains: [], facts: [] });

crate::community::divide_record!(FarmHedge {
    amounts: [settlement, timing],
    gains: [],
    facts: [contract, identified, entered, settled, business_risk],
});

crate::community::divide_enum!(FarmHedgeTiming {
    records: [],
    units: [CurrentSingleYear, InventoryMatched, ComplexHistory],
    tuples: [],
});

crate::community::divide_record!(FarmPriorWeather {
    amounts: [class, source, deferred],
    gains: [],
    facts: [year],
});

crate::community::divide_enum!(FarmWeatherSource {
    records: [MixedProducts { amounts: [raised], facts: [] }],
    units: [RaisedForSale, PurchasedForResale, IncludesBusinessAnimals],
    tuples: [],
});

crate::community::divide_enum!(FarmLivestockClass {
    records: [],
    units: [Cattle, Hogs, Sheep, Goats, Horses, Poultry, Other],
    tuples: [],
});

crate::community::divide_record!(FarmWeatherSale {
    amounts: [class, source, class_income, assistance],
    gains: [],
    facts: [
        normal_sold,
        all_sold,
        weather_excess,
        principal_farming,
        weather_relationship,
        prior_sales,
        usual_practice_support,
        elect
    ],
});

crate::community::divide_enum!(FarmWeatherAssistance {
    records: [Designated { amounts: [], facts: [date] }],
    units: [NotDesignated],
    tuples: [],
});

crate::community::divide_record!(FarmCostSharing {
    amounts: [
        government_received,
        total_cost,
        owner_contribution,
        fmv_improvement,
        non126_program,
        uncertified,
        rent_services,
        current_deductible,
        excluded_funding_applied,
        election
    ],
    gains: [],
    facts: [
        installation_year,
        description,
        program,
        program_approved,
        conservation_certified,
        received,
        asset
    ],
});

crate::community::divide_enum!(Farm126Election {
    records: [
        Exclude { amounts: [facts], facts: [] },
        NotExcludePart { amounts: [amount, facts], facts: [] }
    ],
    units: [NotExcludeAll],
    tuples: [],
});

crate::community::divide_record!(Farm126Exclusion {
    amounts: [prior_income, valuation],
    gains: [],
    facts: [acres_hundredths],
});

crate::community::divide_record!(FarmIncomeRightValue {
    amounts: [ten_percent_value, acre_value],
    gains: [],
    facts: [support],
});

crate::community::divide_record!(FarmAffectedIncome {
    amounts: [income],
    gains: [],
    facts: [year],
});
