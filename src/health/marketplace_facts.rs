pub use crate::health_marketplace_source::MarketplaceBenchmarkSource;
pub use crate::marketplace_record_source::{
    MarketplaceAgreement, MarketplaceAllocatedBenchmark, MarketplaceAllocationKind,
    MarketplaceAllocationPeriod, MarketplaceAppeal, MarketplaceAttestation, MarketplaceAttester,
    MarketplaceBenchmarkGroup, MarketplaceBenchmarkMonth, MarketplaceCoverageEntry,
    MarketplaceCoveredPerson, MarketplaceEligibilityMonth, MarketplaceEnrollmentBasis,
    MarketplaceEventKind, MarketplaceFacts, MarketplaceFamily, MarketplaceFamilyRef,
    MarketplaceFamilyShare, MarketplaceFraction, MarketplaceIncomeQualification,
    MarketplaceLawfulPremiumMonth, MarketplaceLawfulPremiumSource, MarketplaceMaritalFamilyShare,
    MarketplaceMaritalPair, MarketplaceMarriage, MarketplaceMarriageFamily, MarketplaceMecKind,
    MarketplaceMedicalCostMonth, MarketplaceNoAdvanceShare, MarketplaceNonreturnFamily,
    MarketplacePaymentCoverage, MarketplacePaymentMonth, MarketplacePersonEligibility,
    MarketplacePersonFamily, MarketplacePersonIdentity, MarketplacePolicyCoverage,
    MarketplacePolicyRecord, MarketplaceQsehra, MarketplaceQsehraDetermination,
    MarketplaceQsehraEmployeeFacts, MarketplaceQsehraEmployeeHousehold,
    MarketplaceQsehraHouseholdMember, MarketplaceQsehraIncome, MarketplaceQsehraIncomeSource,
    MarketplaceQsehraMonth, MarketplaceQsehraSource, MarketplaceStatementStatus,
};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Marketplace statement sources and tax facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceFactsPage {
    /// Residence used for the poverty guideline
    pub residence: crate::MarketplaceResidence,
    /// Actual enrollment income-estimate facts
    #[libertas_default(Ordinary)]
    pub income_qualification: MarketplaceIncomeQualification,
    /// Forms 1095-A, each accepted source once
    #[libertas_unordered]
    #[libertas_read_only]
    pub forms: Vec<crate::MarketplaceDocumentView>,
    /// Other tax families; save before linking external people
    #[libertas_unordered]
    pub other_families: Vec<MarketplaceFamilyPage>,
    /// External enrollees only; use People identities for this return
    #[libertas_unordered]
    pub covered: Vec<MarketplaceCoveredPersonPage>,
    /// Other-family or no-family membership of existing People identities only
    #[libertas_unordered]
    pub memberships: Vec<MarketplacePersonFamilyPage>,
    /// Actual attesting taxpayer for people in no final tax family
    #[libertas_unordered]
    pub attestations: Vec<MarketplaceAttestationPage>,
    /// Actual eligibility for each covered person and month
    #[libertas_unordered]
    pub eligibility: Vec<MarketplaceEligibilityMonthPage>,
    /// Applicable eligible coverage-family benchmarks
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 0, max = 12)]
    #[libertas_unordered]
    pub benchmarks: Vec<MarketplaceBenchmarkMonthPage>,
    /// Employer QSEHRA arrangements for you, your spouse or a claimed dependent employee
    #[libertas_unordered]
    pub qsehra: Vec<MarketplaceQsehraPage>,
    /// Dependent employees' actual income and family source facts
    /// One record per existing People identity, shared by that employee's notices. An omitted record keeps the affected reconciliation unavailable; do not substitute this return's income or relatives.
    #[libertas_unordered]
    pub employee_qsehra_facts: Vec<MarketplaceQsehraEmployeeFactsPage>,
    /// Claimed dependents who were provided a QSEHRA; each employee needs their own affordability and family reconciliation
    #[libertas_unordered]
    #[libertas_enum_source("$.ptc_claimed_dependents")]
    pub dependent_qsehra_employees: Vec<i64>,
    /// Regular or elected alternative marriage calculation
    #[libertas_default(Regular)]
    #[libertas_constrained_by("$.allowed_marriage")]
    pub marriage: MarketplaceMarriagePage,
    /// Domestic abuse or abandonment relief on a separate return
    #[libertas_constrained_by("$.allowed_separate")]
    pub separate: crate::SeparateRelief,
    /// Required-to-file dependent returns, excluding refund-only returns
    #[libertas_unordered]
    pub dependents: Vec<crate::DependentReturn>,
}

/// Form 1095-A source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read each printed party block into recipient. Copy names, identification numbers and addresses exactly as printed, preserving masking, punctuation and leading zeros; use empty text for absent identity fields. Never infer a recipient from household names. Read one Form 1095-A exactly. Check the calendar year against the supplied year and report a mismatch. Read the line 3 issuer and line 2 policy number as printed, preserving punctuation and leading zeros. Read the corrected or void checkbox as its statement status; otherwise Original. Always return ByMonth with one row for each Part III monthly line 21 through 32 that shows any amount, mapping January to 1 through December 12. Ignore line 33 annual totals. Read columns A premium, B SLCSP and C advance. A populated row with blank or illegible A is an error. A clearly blank B is null, never numeric zero; blank C is zero. Conflicting or illegible amounts are errors. Copies of the same form are one source. Import only recipient, issuer, policy_number, amounts and status; never infer coverage-family identities, eligibility, payment, allocation or other facts absent from the statement.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*",
    form_date_order = "MonthDayYear"
)]
pub struct MarketplacePolicyRecordPage {
    /// Lines 4–9 and 12–15 recipient name, identifying number and address
    #[libertas_llm_doc_field]
    pub recipient: crate::PrintedParty,
    /// Accepted source identity; a new unsaved source has zero
    #[libertas_number(min = 0)]
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Line 3 policy issuer
    #[libertas_size(min = 1)]
    #[libertas_llm_doc_field]
    pub issuer: String,
    /// Line 2 policy number
    #[libertas_default("")]
    #[libertas_llm_doc_field(blank = "")]
    pub policy_number: String,
    /// Printed monthly source amounts; annual totals are not monthly inputs
    #[libertas_llm_doc_field]
    pub amounts: crate::PolicyAmounts,
    /// Original, corrected replacement or void statement
    #[libertas_default(Original)]
    #[libertas_llm_doc_field]
    pub status: MarketplaceStatementStatus,
}

/// Lawful covered-family enrollment premium
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceLawfulPremiumMonthPage {
    /// Month of coverage, 1 for January through 12 for December
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Exact lawful covered set
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    #[libertas_enum_source("$.ptc_people")]
    pub people: Vec<i64>,
    /// Applicable premium source
    pub source: MarketplaceLawfulPremiumSource,
    /// Actual net premium paid for lawful coverage when payments or refunds differed
    /// Leave on the source amount when net payments match it; otherwise enter the actual amount after refunds.
    /// [NullText]
    /// Use the lawful-coverage premium source as the net amount paid
    /// [SetText]
    /// The actual net premium paid for lawful coverage was
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub paid: Option<i64>,
}

/// Other tax family
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceFamilyPage {
    /// Accepted source identity; a new unsaved source has zero
    #[libertas_number(min = 0)]
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Other taxpayer label; filing SSN is supplied by the later filing owner
    #[libertas_size(min = 1)]
    pub label: String,
}

/// External enrollee
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceCoveredPersonPage {
    /// Accepted source identity; a new unsaved source has zero
    #[libertas_number(min = 0)]
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// External person and final tax-family membership
    pub identity: MarketplacePersonIdentityPage,
}

/// Existing person outside this return's calculated tax family
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplacePersonFamilyPage {
    /// Existing person outside this return’s calculated tax family
    #[libertas_number(min = -2)]
    #[libertas_enum_source("$.ptc_nonreturn_people")]
    pub person: i64,
    /// Actual final other-family or no-family membership
    pub family: MarketplaceNonreturnFamilyPage,
}

/// Attesting taxpayer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceAttestationPage {
    /// Person in no final tax family
    #[libertas_number(min = -2)]
    #[libertas_enum_source("$.ptc_people")]
    pub person: i64,
    /// Who attested that this person would be in their tax family
    pub attester: MarketplaceAttesterPage,
}

/// People covered this month
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplacePolicyCoveragePage {
    /// Month of coverage, 1 for January through 12 for December
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Actual enrolled people
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    #[libertas_enum_source("$.ptc_people")]
    pub people: Vec<i64>,
    /// Actual enrollment date changes or exceptions
    #[libertas_unordered]
    pub entry: Vec<MarketplaceCoverageEntryPage>,
}

/// Enrollment dates and basis
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceCoverageEntryPage {
    /// Covered person
    #[libertas_number(min = -2)]
    #[libertas_enum_source("$.ptc_people")]
    pub person: i64,
    /// Actual coverage start
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub start: u32,
    /// Actual coverage end
    #[libertas_date_only]
    // Tax structure: this event or eligibility period must fall by the end of the return year.
    #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
    pub end: u32,
    /// Enrollment basis
    #[libertas_default(Ordinary)]
    pub basis: MarketplaceEnrollmentBasis,
}

/// Covered person eligibility
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceEligibilityMonthPage {
    /// Month of coverage, 1 for January through 12 for December
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Covered person
    #[libertas_number(min = -2)]
    #[libertas_enum_source("$.ptc_people")]
    pub person: i64,
    /// Actual tax eligibility this month
    pub status: MarketplacePersonEligibility,
}

/// Shared-policy allocation period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceAllocationPeriodPage {
    /// First month
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_default(1)]
    pub from: u32,
    /// Last month
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_default(12)]
    pub to: u32,
    /// Applicable source-specific allocation rule
    pub kind: MarketplaceAllocationKindPage,
}

/// Two spouses' tax families
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceMaritalPairPage {
    /// First spouse
    pub one: MarketplaceFamilyRefPage,
    /// Second spouse
    pub two: MarketplaceFamilyRefPage,
}

/// Tax-family allocation agreement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceFamilySharePage {
    /// Tax family
    pub family: MarketplaceFamilyRefPage,
    /// Actual common agreement or prescribed default
    pub agreement: MarketplaceAgreement,
}

/// Third-family agreements with each spouse
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceMaritalFamilySharePage {
    /// Third tax family
    pub family: MarketplaceFamilyRefPage,
    /// Share agreed with the first spouse, or prescribed default
    pub one: MarketplaceAgreement,
    /// Share agreed with the second spouse, or prescribed default
    pub two: MarketplaceAgreement,
}

/// Applicable family benchmark
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceNoAdvanceSharePage {
    /// Tax family
    pub family: MarketplaceFamilyRefPage,
    /// This family's actual applicable SLCSP
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub benchmark: i64,
}

/// Eligible coverage-family benchmark components
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceBenchmarkMonthPage {
    /// Month of coverage, 1 for January through 12 for December
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Unallocated location and rating-area family benchmarks
    #[libertas_unordered]
    pub groups: Vec<MarketplaceBenchmarkGroupPage>,
    /// Part IV allocated components
    #[libertas_unordered]
    pub allocated: Vec<MarketplaceAllocatedBenchmarkPage>,
}

/// Applicable location and rating-area family benchmark
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceBenchmarkGroupPage {
    /// Coverage location state
    pub state: crate::TaxState,
    /// Marketplace location or rating area
    #[libertas_size(min = 1)]
    pub rating_area: String,
    /// Eligible covered people in this benchmark
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    #[libertas_enum_source("$.ptc_people")]
    pub people: Vec<i64>,
    /// Actual benchmark source
    pub source: MarketplaceBenchmarkSourcePage,
}

/// Allocated benchmark component
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceAllocatedBenchmarkPage {
    /// Shared policy
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.ptc_policies")]
    pub policy: i64,
    /// Pre-allocation applicable benchmark source
    pub source: MarketplaceBenchmarkSourcePage,
}

/// Premium payment and actual issuer coverage
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplacePaymentMonthPage {
    /// Month of coverage, 1 for January through 12 for December
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Premium received by issuer, including APTC
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub paid: i64,
    /// Same-year premium refunds
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub refunded: i64,
    /// Actual tax-free premium payments or reimbursements attributable to this return
    /// After any shared-policy allocation, enter premiums funded by tax-free employer/HRA payments, qualified HSA/FSA spending or tax-free public-safety retirement amounts. This is actual expense funding, not a QSEHRA permitted benefit or APTC. Do not subtract it from printed Form1095-A amounts.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tax_free_premiums: i64,
    /// Part of that actual tax-free funding attributable to coverage-family premiums
    /// Needed for a self-employed policy with mixed coverage when tax-free funding applies to particular people. Use actual reimbursement records; do not guess that the silver-plan ratio allocates a person's reimbursement.
    /// [NullText]
    /// A separate coverage-family funding allocation is not needed
    /// [SetText]
    /// The tax-free funding attributable to coverage-family premiums is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub tax_free_coverage: Option<i64>,
    /// Unpaid premium
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub unpaid: i64,
    /// Actual coverage and payment exception
    pub coverage: MarketplacePaymentCoverage,
    /// Successful eligibility appeal, if applicable
    /// [NullText]
    /// No successful eligibility appeal applies to this month
    /// [SetText]
    /// The successful eligibility appeal is
    pub appeal: Option<MarketplaceAppealPage>,
}

/// Successful eligibility appeal
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("CalculateMarketplaceAppeal")]
pub struct MarketplaceAppealPage {
    /// Appeal decision date
    #[libertas_date_only]
    #[libertas_calculation_request("CalculateMarketplaceAppeal")]
    pub determined: u32,
    /// Actual enrollment date
    #[libertas_date_only]
    pub enrolled: u32,
    /// Actual premium payment date
    /// [NullText]
    /// The premium has not been paid
    /// [SetText]
    /// The premium was paid on
    #[libertas_date_only]
    pub paid_on: Option<u32>,
    /// Applicable successful-appeal payment deadline, calculated from the decision
    #[libertas_date_only]
    #[libertas_read_only]
    pub required_deadline: u32,
}

/// Employer QSEHRA arrangement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceQsehraPage {
    /// Accepted source identity; a new unsaved source has zero
    #[libertas_number(min = 0)]
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub id: i64,
    /// Employee on this return, including a claimed dependent
    /// Select the actual employee named on the notice. A dependent's notice does not make the employee's parents or siblings their spouse or dependents.
    #[libertas_number(min = -2)]
    #[libertas_enum_source("$.ptc_employees")]
    pub employee: i64,
    /// Accepted W-2 or employer notice
    pub source: MarketplaceQsehraSourcePage,
    /// Actual permitted benefits and covered people
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 1, max = 12)]
    #[libertas_unordered]
    pub months: Vec<MarketplaceQsehraMonthPage>,
    /// Actual Marketplace determinations
    #[libertas_unordered]
    pub determinations: Vec<MarketplaceQsehraDetermination>,
}

/// Dependent employee household member
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceQsehraHouseholdMemberPage {
    /// Actual dependent of this employee, using an existing People identity
    #[libertas_enum_source("$.people")]
    #[libertas_unique(1)]
    pub person: i64,
    /// Income source if required to file because of income
    /// Leave absent only for a person not required to file. A refund-only return does not add household income.
    /// [NullText]
    /// This person is not required to file because of income
    /// [SetText]
    /// This person has a required income return
    pub income: Option<MarketplaceQsehraIncomeSource>,
}

/// Actual dependent employee household
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MarketplaceQsehraEmployeeHouseholdPage {
    /// All actual spouse and dependent identities are available on People
    /// No spouse and an empty dependent list explicitly mean none; do not select this if an actual person is unavailable in the catalog.
    Known {
        /// Actual spouse, if any
        /// [NullText]
        /// This employee has no spouse
        /// [SetText]
        /// This employee's actual spouse is
        #[libertas_enum_source("$.people")]
        spouse: Option<i64>,
        /// Actual dependents and their required-return sources
        dependents: Vec<MarketplaceQsehraHouseholdMemberPage>,
    },
    /// An actual spouse or dependent is not available on People
    OtherPeople,
}

/// Dependent employee QSEHRA source facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceQsehraEmployeeFactsPage {
    /// Actual employee, using an existing claimed dependent People identity
    #[libertas_number(min = 1)]
    #[libertas_unique(1)]
    #[libertas_enum_source("$.ptc_claimed_dependents")]
    pub employee: i64,
    /// Employee's own or joint-return income source
    /// [NullText]
    /// Employee household income source is not yet available
    /// [SetText]
    /// Employee household income source is available
    pub income: Option<MarketplaceQsehraIncomeSource>,
    /// Actual employee spouse and dependents
    /// Absence is unanswered; Known with no relatives is explicitly empty.
    /// [NullText]
    /// Employee household relationships are not yet available
    /// [SetText]
    /// Employee household relationships are available
    pub household: Option<MarketplaceQsehraEmployeeHouseholdPage>,
}

/// Monthly QSEHRA benefit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceQsehraMonthPage {
    /// Month of coverage, 1 for January through 12 for December
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Age-applicable self-only monthly permitted benefit
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub self_only: i64,
    /// Actual current-year monthly permitted benefit, not reimbursements
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual: i64,
    /// Actual self-only employee SLCSP
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub self_benchmark: i64,
    /// People covered by this arrangement
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    #[libertas_enum_source("$.ptc_people")]
    pub covered: Vec<i64>,
}

/// Premarriage coverage family
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceMarriageFamilyPage {
    /// Claimed dependents who separately qualify as this spouse’s dependents for the year, assigned once
    #[libertas_unordered]
    #[libertas_enum_source("$.ptc_claimed_dependents")]
    pub dependents: Vec<i64>,
    /// Actual Forms1095-A sent to this spouse, or sent to neither spouse for this alternative family. Consume each saved statement once; separate recipient statements may have the same insurance policy number
    #[libertas_unordered]
    #[libertas_enum_source("$.ptc_policies")]
    pub premarriage_policies: Vec<i64>,
    /// Actual alternative-family benchmark corrections when printed amounts are not applicable
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 0, max = 12)]
    #[libertas_unordered]
    pub benchmarks: Vec<MarketplaceBenchmarkMonthPage>,
}

/// External person identity
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
pub enum MarketplacePersonIdentityPage {
    /// Person in another tax family
    OtherFamily {
        /// Other tax family
        #[libertas_number(min = 1)]
        #[libertas_enum_source("$.ptc_families")]
        family: i64,
        /// Person label
        #[libertas_size(min = 1)]
        label: String,
    },
    /// Person in no final tax family
    NoFamily {
        /// Person label
        #[libertas_size(min = 1)]
        label: String,
    },
}

/// Membership outside this return
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
pub enum MarketplaceNonreturnFamilyPage {
    /// Another tax family
    OtherFamily {
        /// Other tax family
        #[libertas_number(min = 1)]
        #[libertas_enum_source("$.ptc_families")]
        family: i64,
    },
    /// No final tax family
    NoFamily,
}

/// Tax family
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
pub enum MarketplaceFamilyRefPage {
    /// This return
    ThisReturn,
    /// Other tax family
    Other {
        /// Other tax family
        #[libertas_number(min = 1)]
        #[libertas_enum_source("$.ptc_families")]
        family: i64,
    },
}

/// Attesting taxpayer
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
pub enum MarketplaceAttesterPage {
    /// This return
    ThisReturn,
    /// Another tax family
    OtherFamily {
        /// Other tax family
        #[libertas_number(min = 1)]
        #[libertas_enum_source("$.ptc_families")]
        family: i64,
    },
    // Form 1095-A does not identify the actual income attester. Missing Exchange
    // or other-family evidence retains PTC_SOURCE_ATTESTER_REQUIRED.
    /// Not yet established
    Unknown,
}

/// Shared-policy allocation
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
pub enum MarketplaceAllocationKindPage {
    /// Former-spouse married months, then any other claiming families
    FormerSpouses {
        /// Actual former-spouse pair
        pair: MarketplaceMaritalPairPage,
        /// First spouse's share before third-family allocation
        agreement: MarketplaceAgreement,
        /// Other claiming families
        #[libertas_unordered]
        third_families: Vec<MarketplaceMaritalFamilySharePage>,
    },
    /// Spouses filing separately, after any other claiming-family allocation
    MarriedSeparate {
        /// Actual separate-return spouse pair
        pair: MarketplaceMaritalPairPage,
        /// First spouse's actual applicable coverage-family benchmark
        one_benchmark: MarketplaceBenchmarkSourcePage,
        /// Second spouse's actual applicable coverage-family benchmark
        two_benchmark: MarketplaceBenchmarkSourcePage,
        /// Other claiming families
        #[libertas_unordered]
        third_families: Vec<MarketplaceMaritalFamilySharePage>,
    },
    /// Shared policy without APTC; applicable-family benchmark ratio
    NoAdvance {
        /// Every covered tax family and its own applicable benchmark
        #[libertas_size(min = 1)]
        #[libertas_unordered]
        families: Vec<MarketplaceNoAdvanceSharePage>,
    },
    /// Other shared policy with APTC
    OtherShared {
        /// Covered tax families and actual agreement
        #[libertas_size(min = 1)]
        #[libertas_unordered]
        families: Vec<MarketplaceFamilySharePage>,
    },
}

/// Applicable benchmark source
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
pub enum MarketplaceBenchmarkSourcePage {
    /// Applicable printed benchmark
    Statement {
        /// Accepted policy
        #[libertas_number(min = 1)]
        #[libertas_enum_source("$.ptc_policies")]
        policy: i64,
    },
    /// Actual applicable Marketplace benchmark
    Marketplace {
        /// Applicable benchmark
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Authorized same-lawful-covered-set reference month
    ReferenceMonth {
        /// Accepted source policy
        #[libertas_number(min = 1)]
        #[libertas_enum_source("$.ptc_policies")]
        policy: i64,
        /// Month of coverage, 1 for January through 12 for December
        // Tax structure: calendar months are January (1) through December (12).
        #[libertas_number(min = 1, max = 12)]
        month: u32,
    },
}

/// QSEHRA notice source
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
pub enum MarketplaceQsehraSourcePage {
    /// Accepted W-2 code FF
    W2 {
        /// Employee-owned W-2
        #[libertas_number(min = 1)]
        #[libertas_enum_source("$.ptc_w2")]
        document: i64,
    },
    /// Employer notice when no W-2 exists
    Notice {
        /// Employer
        #[libertas_size(min = 1)]
        employer: String,
        /// Current-year permitted benefit, excluding prior carryover
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        annual_permitted: i64,
    },
}

/// Year-of-marriage calculation
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
pub enum MarketplaceMarriagePage {
    /// Regular calculation
    Regular,
    /// Elect alternative calculation if it lowers excess repayment
    Alternative {
        /// Actual wedding date in 2026
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
        wedding: u32,
        /// Both spouses were unmarried on the first day of the tax year
        #[libertas_default(false)]
        unmarried_at_start: bool,
        /// Your actual premarriage coverage family
        you: MarketplaceMarriageFamilyPage,
        /// Your spouse's actual premarriage coverage family
        spouse: MarketplaceMarriageFamilyPage,
    },
}

record_page_conversions!(MarketplaceFacts, MarketplaceFactsPage, [], |value, Target, Source| {
    let Source {
        residence,
        income_qualification,
        forms,
        other_families,
        covered,
        memberships,
        attestations,
        eligibility,
        benchmarks,
        qsehra,
        employee_qsehra_facts,
        dependent_qsehra_employees,
        marriage,
        separate,
        dependents,
    } = value;
    Target {
        residence,
        income_qualification,
        forms: forms.into_iter().map(Into::into).collect(),
        other_families: other_families.into_iter().map(Into::into).collect(),
        covered: covered.into_iter().map(Into::into).collect(),
        memberships: memberships.into_iter().map(Into::into).collect(),
        attestations: attestations.into_iter().map(Into::into).collect(),
        eligibility: eligibility.into_iter().map(Into::into).collect(),
        benchmarks: benchmarks.into_iter().map(Into::into).collect(),
        qsehra: qsehra.into_iter().map(Into::into).collect(),
        employee_qsehra_facts: employee_qsehra_facts.into_iter().map(Into::into).collect(),
        dependent_qsehra_employees,
        marriage: marriage.into(),
        separate,
        dependents: dependents.into_iter().map(Into::into).collect(),
    }
});

impl From<MarketplacePolicyRecord> for MarketplacePolicyRecordPage {
    fn from(value: MarketplacePolicyRecord) -> Self {
        Self { id: value.id, ..Self::from(&value.form) }
    }
}
impl From<crate::Form1095APaper> for MarketplacePolicyRecord {
    fn from(form: crate::Form1095APaper) -> Self {
        let recipient = crate::PrintedParty::from(&form.recipient);
        let printed =
            (recipient != crate::PrintedParty::default()).then(|| crate::PrintedDocumentNames {
                recipient,
                issuer: Some(crate::PrintedParty::from_name(form.issuer.clone())),
                ..Default::default()
            });
        Self {
            id: 0,
            printed,
            form,
            lawful_premiums: Vec::new(),
            enrolling: MarketplaceFamilyRef::ThisReturn,
            coverage: Vec::new(),
            allocation: Vec::new(),
            payments: Vec::new(),
            medical_costs: Vec::new(),
        }
    }
}

record_page_conversions!(
    MarketplaceLawfulPremiumMonth,
    MarketplaceLawfulPremiumMonthPage,
    [],
    |value, Target, Source| {
        let Source { month, people, source, paid } = value;
        Target { month, people, source, paid }
    }
);

record_page_conversions!(MarketplaceFamily, MarketplaceFamilyPage, [], |value, Target, Source| {
    let Source { id, label } = value;
    Target { id, label }
});

record_page_conversions!(
    MarketplaceCoveredPerson,
    MarketplaceCoveredPersonPage,
    [],
    |value, Target, Source| {
        let Source { id, identity } = value;
        Target { id, identity: identity.into() }
    }
);

record_page_conversions!(
    MarketplacePersonFamily,
    MarketplacePersonFamilyPage,
    [],
    |value, Target, Source| {
        let Source { person, family } = value;
        Target { person, family: family.into() }
    }
);

record_page_conversions!(
    MarketplaceAttestation,
    MarketplaceAttestationPage,
    [],
    |value, Target, Source| {
        let Source { person, attester } = value;
        Target { person, attester: attester.into() }
    }
);

record_page_conversions!(
    MarketplacePolicyCoverage,
    MarketplacePolicyCoveragePage,
    [],
    |value, Target, Source| {
        let Source { month, people, entry } = value;
        Target { month, people, entry: entry.into_iter().map(Into::into).collect() }
    }
);

record_page_conversions!(
    MarketplaceCoverageEntry,
    MarketplaceCoverageEntryPage,
    [],
    |value, Target, Source| {
        let Source { person, start, end, basis } = value;
        Target { person, start, end, basis }
    }
);

record_page_conversions!(
    MarketplaceEligibilityMonth,
    MarketplaceEligibilityMonthPage,
    [],
    |value, Target, Source| {
        let Source { month, person, status } = value;
        Target { month, person, status }
    }
);

record_page_conversions!(
    MarketplaceAllocationPeriod,
    MarketplaceAllocationPeriodPage,
    [],
    |value, Target, Source| {
        let Source { from, to, kind } = value;
        Target { from, to, kind: kind.into() }
    }
);

record_page_conversions!(
    MarketplaceMaritalPair,
    MarketplaceMaritalPairPage,
    [],
    |value, Target, Source| {
        let Source { one, two } = value;
        Target { one: one.into(), two: two.into() }
    }
);

record_page_conversions!(
    MarketplaceFamilyShare,
    MarketplaceFamilySharePage,
    [],
    |value, Target, Source| {
        let Source { family, agreement } = value;
        Target { family: family.into(), agreement }
    }
);

record_page_conversions!(
    MarketplaceMaritalFamilyShare,
    MarketplaceMaritalFamilySharePage,
    [],
    |value, Target, Source| {
        let Source { family, one, two } = value;
        Target { family: family.into(), one, two }
    }
);

record_page_conversions!(
    MarketplaceNoAdvanceShare,
    MarketplaceNoAdvanceSharePage,
    [],
    |value, Target, Source| {
        let Source { family, benchmark } = value;
        Target { family: family.into(), benchmark }
    }
);

record_page_conversions!(
    MarketplaceBenchmarkMonth,
    MarketplaceBenchmarkMonthPage,
    [],
    |value, Target, Source| {
        let Source { month, groups, allocated } = value;
        Target {
            month,
            groups: groups.into_iter().map(Into::into).collect(),
            allocated: allocated.into_iter().map(Into::into).collect(),
        }
    }
);

record_page_conversions!(
    MarketplaceBenchmarkGroup,
    MarketplaceBenchmarkGroupPage,
    [],
    |value, Target, Source| {
        let Source { state, rating_area, people, source } = value;
        Target { state, rating_area, people, source: source.into() }
    }
);

record_page_conversions!(
    MarketplaceAllocatedBenchmark,
    MarketplaceAllocatedBenchmarkPage,
    [],
    |value, Target, Source| {
        let Source { policy, source } = value;
        Target { policy, source: source.into() }
    }
);

record_page_conversions!(
    MarketplacePaymentMonth,
    MarketplacePaymentMonthPage,
    [],
    |value, Target, Source| {
        let Source {
            month,
            paid,
            refunded,
            tax_free_premiums,
            tax_free_coverage,
            unpaid,
            coverage,
            appeal,
        } = value;
        Target {
            month,
            paid,
            refunded,
            tax_free_premiums,
            tax_free_coverage,
            unpaid,
            coverage,
            appeal: appeal.map(Into::into),
        }
    }
);

impl From<MarketplaceAppeal> for MarketplaceAppealPage {
    fn from(value: MarketplaceAppeal) -> Self {
        let MarketplaceAppeal { determined, enrolled, paid_on } = value;
        Self {
            determined,
            enrolled,
            paid_on,
            required_deadline: crate::dates::add_days_checked(determined, 120)
                .expect("accepted appeal date has a representable deadline"),
        }
    }
}

impl From<MarketplaceAppealPage> for MarketplaceAppeal {
    fn from(value: MarketplaceAppealPage) -> Self {
        let MarketplaceAppealPage { determined, enrolled, paid_on, .. } = value;
        Self { determined, enrolled, paid_on }
    }
}

record_page_conversions!(MarketplaceQsehra, MarketplaceQsehraPage, [], |value, Target, Source| {
    let Source { id, employee, source, months, determinations } = value;
    Target {
        id,
        employee,
        source: source.into(),
        months: months.into_iter().map(Into::into).collect(),
        determinations,
    }
});

record_page_conversions!(
    MarketplaceQsehraMonth,
    MarketplaceQsehraMonthPage,
    [],
    |value, Target, Source| {
        let Source { month, self_only, actual, self_benchmark, covered } = value;
        Target { month, self_only, actual, self_benchmark, covered }
    }
);

record_page_conversions!(
    MarketplaceMarriageFamily,
    MarketplaceMarriageFamilyPage,
    [],
    |value, Target, Source| {
        let Source { dependents, premarriage_policies, benchmarks } = value;
        Target {
            dependents,
            premarriage_policies,
            benchmarks: benchmarks.into_iter().map(Into::into).collect(),
        }
    }
);

record_page_conversions!(
    MarketplacePersonIdentity,
    MarketplacePersonIdentityPage,
    [],
    |value, Target, Source| {
        match value {
            Source::OtherFamily { family, label } => Target::OtherFamily { family, label },
            Source::NoFamily { label } => Target::NoFamily { label },
        }
    }
);

record_page_conversions!(
    MarketplaceNonreturnFamily,
    MarketplaceNonreturnFamilyPage,
    [],
    |value, Target, Source| {
        match value {
            Source::OtherFamily { family } => Target::OtherFamily { family },
            Source::NoFamily => Target::NoFamily,
        }
    }
);

record_page_conversions!(
    MarketplaceFamilyRef,
    MarketplaceFamilyRefPage,
    [],
    |value, Target, Source| {
        match value {
            Source::ThisReturn => Target::ThisReturn,
            Source::Other { family } => Target::Other { family },
        }
    }
);

record_page_conversions!(
    MarketplaceAttester,
    MarketplaceAttesterPage,
    [],
    |value, Target, Source| {
        match value {
            Source::ThisReturn => Target::ThisReturn,
            Source::OtherFamily { family } => Target::OtherFamily { family },
            Source::Unknown => Target::Unknown,
        }
    }
);

record_page_conversions!(
    MarketplaceAllocationKind,
    MarketplaceAllocationKindPage,
    [],
    |value, Target, Source| {
        match value {
            Source::FormerSpouses { pair, agreement, third_families } => Target::FormerSpouses {
                pair: pair.into(),
                agreement,
                third_families: third_families.into_iter().map(Into::into).collect(),
            },
            Source::MarriedSeparate { pair, one_benchmark, two_benchmark, third_families } => {
                Target::MarriedSeparate {
                    pair: pair.into(),
                    one_benchmark: one_benchmark.into(),
                    two_benchmark: two_benchmark.into(),
                    third_families: third_families.into_iter().map(Into::into).collect(),
                }
            }
            Source::NoAdvance { families } => {
                Target::NoAdvance { families: families.into_iter().map(Into::into).collect() }
            }
            Source::OtherShared { families } => {
                Target::OtherShared { families: families.into_iter().map(Into::into).collect() }
            }
        }
    }
);

record_page_conversions!(
    MarketplaceBenchmarkSource,
    MarketplaceBenchmarkSourcePage,
    [],
    |value, Target, Source| {
        match value {
            Source::Statement { policy } => Target::Statement { policy },
            Source::Marketplace { amount } => Target::Marketplace { amount },
            Source::ReferenceMonth { policy, month } => Target::ReferenceMonth { policy, month },
        }
    }
);

record_page_conversions!(
    MarketplaceQsehraSource,
    MarketplaceQsehraSourcePage,
    [],
    |value, Target, Source| {
        match value {
            Source::W2 { document } => Target::W2 { document },
            Source::Notice { employer, annual_permitted } => {
                Target::Notice { employer, annual_permitted }
            }
        }
    }
);

record_page_conversions!(
    MarketplaceMarriage,
    MarketplaceMarriagePage,
    [],
    |value, Target, Source| {
        match value {
            Source::Regular => Target::Regular,
            Source::Alternative { wedding, unmarried_at_start, you, spouse } => {
                Target::Alternative {
                    wedding,
                    unmarried_at_start,
                    you: you.into(),
                    spouse: spouse.into(),
                }
            }
        }
    }
);

impl From<crate::MarketplaceDependent> for crate::DependentReturn {
    fn from(value: crate::MarketplaceDependent) -> Self {
        Self {
            person: value.person,
            agi: value.agi,
            exempt_interest: value.exempt_interest,
            foreign_exclusion: value.foreign_exclusion,
            social: value.social,
            taxable_social: value.taxable_social,
        }
    }
}
impl From<crate::DependentReturn> for crate::MarketplaceDependent {
    fn from(value: crate::DependentReturn) -> Self {
        Self {
            person: value.person,
            agi: value.agi,
            exempt_interest: value.exempt_interest,
            foreign_exclusion: value.foreign_exclusion,
            social: value.social,
            taxable_social: value.taxable_social,
        }
    }
}
impl Default for MarketplaceFactsPage {
    fn default() -> Self {
        MarketplaceFacts::default().into()
    }
}

record_page_conversions!(
    MarketplaceQsehraEmployeeFacts,
    MarketplaceQsehraEmployeeFactsPage,
    [],
    |v, Target| {
        Target { employee: v.employee, income: v.income, household: v.household.map(Into::into) }
    }
);

record_page_conversions!(
    MarketplaceQsehraHouseholdMember,
    MarketplaceQsehraHouseholdMemberPage,
    [],
    |value, Target| Target { person: value.person, income: value.income }
);
record_page_conversions!(
    MarketplaceQsehraEmployeeHousehold,
    MarketplaceQsehraEmployeeHouseholdPage,
    [],
    |value, Target, Source| {
        match value {
            Source::Known { spouse, dependents } => Target::Known {
                spouse,
                dependents: dependents.into_iter().map(Into::into).collect(),
            },
            Source::OtherPeople => Target::OtherPeople,
        }
    }
);
