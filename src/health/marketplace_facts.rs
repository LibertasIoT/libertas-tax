// Canonical Marketplace sources and direct tax facts (approved Units20/24).
// Original statement amounts remain separate from transient tax projections.
// Picker paths exist only on page twins; accepted records hold stable IDs.
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Marketplace statement sources and tax facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceFacts {
    /// Residence used for the poverty guideline
    pub residence: crate::MarketplaceResidence,
    /// Actual enrollment income-estimate facts
    #[libertas_default(Ordinary)]
    pub income_qualification: MarketplaceIncomeQualification,
    /// Forms 1095-A, each accepted source once
    #[libertas_unordered]
    pub forms: Vec<MarketplacePolicyRecord>,
    /// Other tax families; save before linking external people
    #[libertas_unordered]
    pub other_families: Vec<MarketplaceFamily>,
    /// External enrollees only; use People identities for this return
    #[libertas_unordered]
    pub covered: Vec<MarketplaceCoveredPerson>,
    /// Other-family or no-family membership of existing People identities only
    #[libertas_unordered]
    pub memberships: Vec<MarketplacePersonFamily>,
    /// Actual attesting taxpayer for people in no final tax family
    #[libertas_unordered]
    pub attestations: Vec<MarketplaceAttestation>,
    /// Actual eligibility for each covered person and month
    #[libertas_unordered]
    pub eligibility: Vec<MarketplaceEligibilityMonth>,
    /// Applicable eligible coverage-family benchmarks
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 0, max = 12)]
    #[libertas_unordered]
    pub benchmarks: Vec<MarketplaceBenchmarkMonth>,
    /// Employer QSEHRA arrangements for you, your spouse or a claimed dependent employee
    #[libertas_unordered]
    pub qsehra: Vec<MarketplaceQsehra>,
    /// Dependent employees' actual income and family source facts
    /// One record per existing People identity, shared by that employee's notices.
    #[libertas_unordered]
    pub employee_qsehra_facts: Vec<MarketplaceQsehraEmployeeFacts>,
    /// Claimed dependents who were provided a QSEHRA; each employee needs their own affordability and family reconciliation
    #[libertas_unordered]
    pub dependent_qsehra_employees: Vec<i64>,
    /// Regular or elected alternative marriage calculation
    #[libertas_default(Regular)]
    pub marriage: MarketplaceMarriage,
    /// Domestic abuse or abandonment relief on a separate return
    pub separate: crate::SeparateRelief,
    /// Required-to-file dependent returns, excluding refund-only returns
    #[libertas_unordered]
    pub dependents: Vec<crate::MarketplaceDependent>,
}

/// Form 1095-A source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplacePolicyRecord {
    /// Accepted source identity; a new unsaved source has zero
    #[libertas_number(min = 1)]
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    /// Line 3 policy issuer
    #[libertas_size(min = 1)]
    pub issuer: String,
    /// Line 2 policy number
    #[libertas_default("")]
    pub policy_number: String,
    /// Printed monthly source amounts; annual totals are not monthly inputs
    pub amounts: crate::PolicyAmounts,
    /// Actual premiums excluding unlawful enrollees
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 0, max = 12)]
    #[libertas_unordered]
    pub lawful_premiums: Vec<MarketplaceLawfulPremiumMonth>,
    /// Family that actually enrolled the policy
    pub enrolling: MarketplaceFamilyRef,
    /// Original, corrected replacement or void statement
    #[libertas_default(Original)]
    pub status: MarketplaceStatementStatus,
    /// Actual covered people by month
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 1, max = 12)]
    #[libertas_unordered]
    pub coverage: Vec<MarketplacePolicyCoverage>,
    /// Shared-family allocation periods
    #[libertas_unordered]
    pub allocation: Vec<MarketplaceAllocationPeriod>,
    /// Actual payment problems or appeal facts
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 0, max = 12)]
    #[libertas_unordered]
    pub payments: Vec<MarketplacePaymentMonth>,
    /// Actual extra premium for children who are not medical dependents
    /// Optional expense-source facts, independent of the silver-plan allocation. Add a row only for a month covering an actual child under 27 whose costs are not medical expenses for this return. Missing cost evidence does not prevent a known premium-credit reconciliation.
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 0, max = 12)]
    #[libertas_unordered]
    pub medical_costs: Vec<MarketplaceMedicalCostMonth>,
}

/// Actual nonmedical-child premium cost in one policy month
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceMedicalCostMonth {
    /// Coverage month, January 1 through December 12
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Actual included premium increase for nonmedical children under 27
    /// Use the issuer's actual cost of adding the covered child group, including an actual zero for an unchanged family price tier. This is part of the net premium actually paid after refunds and allocated to this return, before tax-free funding or premium credit. Do not substitute a silver-plan ratio, sum independent per-child marginal prices, or use a full-month price for a partial payment.
    /// [NullText]
    /// The actual extra child premium cost is not available
    /// [SetText]
    /// The actual included extra premium for these children is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub child_increase: Option<i64>,
    /// Included actual tax-free funding for that child cost
    /// A subset of this policy month's existing tax-free funding, required when that funding and the child's positive added cost intersect. Coverage-family funding and child funding are different source facts.
    /// [NullText]
    /// A separate child-cost tax-free funding amount is not needed
    /// [SetText]
    /// The tax-free funding included in that child cost is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub tax_free_child: Option<i64>,
}

/// Lawful covered-family enrollment premium
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceLawfulPremiumMonth {
    /// Coverage month, January 1 through December 12
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Exact lawful covered set
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    pub people: Vec<i64>,
    /// Applicable premium source
    pub source: MarketplaceLawfulPremiumSource,
    /// Actual net premium paid for lawful coverage when payments or refunds differed
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub paid: Option<i64>,
}

/// Other tax family
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceFamily {
    /// Accepted source identity; a new unsaved source has zero
    #[libertas_number(min = 1)]
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    /// Other taxpayer label; filing SSN is supplied by the later filing owner
    #[libertas_size(min = 1)]
    pub label: String,
}

/// External enrollee
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceCoveredPerson {
    /// Accepted source identity; a new unsaved source has zero
    #[libertas_number(min = 1)]
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    /// External person and final tax-family membership
    pub identity: MarketplacePersonIdentity,
}

/// Existing person outside this return's calculated tax family
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplacePersonFamily {
    /// Existing person outside this return’s calculated tax family
    #[libertas_number(min = -2)]
    pub person: i64,
    /// Actual final other-family or no-family membership
    pub family: MarketplaceNonreturnFamily,
}

/// Attesting taxpayer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceAttestation {
    /// Person in no final tax family
    #[libertas_number(min = -2)]
    pub person: i64,
    /// Who attested that this person would be in their tax family
    pub attester: MarketplaceAttester,
}

/// People covered this month
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplacePolicyCoverage {
    /// Coverage month, January 1 through December 12
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Actual enrolled people
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    pub people: Vec<i64>,
    /// Actual enrollment date changes or exceptions
    #[libertas_unordered]
    pub entry: Vec<MarketplaceCoverageEntry>,
}

/// Enrollment dates and basis
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceCoverageEntry {
    /// Covered person
    #[libertas_number(min = -2)]
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
pub struct MarketplaceEligibilityMonth {
    /// Coverage month, January 1 through December 12
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Covered person
    #[libertas_number(min = -2)]
    pub person: i64,
    /// Actual tax eligibility this month
    pub status: MarketplacePersonEligibility,
}

/// Shared-policy allocation period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceAllocationPeriod {
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
    pub kind: MarketplaceAllocationKind,
}

/// Two spouses' tax families
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceMaritalPair {
    /// First spouse
    pub one: MarketplaceFamilyRef,
    /// Second spouse
    pub two: MarketplaceFamilyRef,
}

/// Exact allocation fraction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceFraction {
    /// Numerator
    #[libertas_number(min = 0)]
    pub numerator: u32,
    /// Denominator
    #[libertas_number(min = 1)]
    #[libertas_default(100)]
    pub denominator: u32,
}

/// Tax-family allocation agreement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceFamilyShare {
    /// Tax family
    pub family: MarketplaceFamilyRef,
    /// Actual common agreement or prescribed default
    pub agreement: MarketplaceAgreement,
}

/// Third-family agreements with each spouse
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceMaritalFamilyShare {
    /// Third tax family
    pub family: MarketplaceFamilyRef,
    /// Share agreed with the first spouse, or prescribed default
    pub one: MarketplaceAgreement,
    /// Share agreed with the second spouse, or prescribed default
    pub two: MarketplaceAgreement,
}

/// Applicable family benchmark
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceNoAdvanceShare {
    /// Tax family
    pub family: MarketplaceFamilyRef,
    /// This family's actual applicable SLCSP
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub benchmark: i64,
}

/// Eligible coverage-family benchmark components
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceBenchmarkMonth {
    /// Coverage month, January 1 through December 12
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Unallocated location and rating-area family benchmarks
    #[libertas_unordered]
    pub groups: Vec<MarketplaceBenchmarkGroup>,
    /// Part IV allocated components
    #[libertas_unordered]
    pub allocated: Vec<MarketplaceAllocatedBenchmark>,
}

/// Applicable location and rating-area family benchmark
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceBenchmarkGroup {
    /// Coverage location state
    pub state: crate::TaxState,
    /// Marketplace location or rating area
    #[libertas_size(min = 1)]
    pub rating_area: String,
    /// Eligible covered people in this benchmark
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    pub people: Vec<i64>,
    /// Actual benchmark source
    pub source: MarketplaceBenchmarkSource,
}

/// Allocated benchmark component
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceAllocatedBenchmark {
    /// Shared policy
    #[libertas_number(min = 1)]
    pub policy: i64,
    /// Pre-allocation applicable benchmark source
    pub source: MarketplaceBenchmarkSource,
}

/// Premium payment and actual issuer coverage
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplacePaymentMonth {
    /// Coverage month, January 1 through December 12
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Premium received by issuer, including APTC
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub paid: i64,
    /// Same-year premium refunds
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub refunded: i64,
    /// Actual tax-free premium payments or reimbursements attributable to this return
    /// After any shared-policy allocation, enter premiums funded by tax-free employer/HRA payments, qualified HSA/FSA spending or tax-free public-safety retirement amounts. This is actual expense funding, not a QSEHRA permitted benefit or APTC. Do not subtract it from printed Form1095-A amounts.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub tax_free_premiums: i64,
    /// Part of that actual tax-free funding attributable to coverage-family premiums
    /// Needed for a self-employed policy with mixed coverage when tax-free funding applies to particular people. Use actual reimbursement records; do not guess that the silver-plan ratio allocates a person's reimbursement.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default($null)]
    pub tax_free_coverage: Option<i64>,
    /// Unpaid premium
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub unpaid: i64,
    /// Actual coverage and payment exception
    pub coverage: MarketplacePaymentCoverage,
    /// Successful eligibility appeal, if applicable
    #[libertas_default($null)]
    pub appeal: Option<MarketplaceAppeal>,
}

/// Successful eligibility appeal
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceAppeal {
    /// Appeal decision date
    #[libertas_date_only]
    pub determined: u32,
    /// Actual enrollment date
    #[libertas_date_only]
    pub enrolled: u32,
    /// Actual premium payment date
    #[libertas_date_only]
    #[libertas_default($null)]
    pub paid_on: Option<u32>,
}

/// Employer QSEHRA arrangement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceQsehra {
    /// Accepted source identity; a new unsaved source has zero
    #[libertas_number(min = 1)]
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    /// Employee on this return, including a claimed dependent
    #[libertas_number(min = -2)]
    pub employee: i64,
    /// Accepted W-2 or employer notice
    pub source: MarketplaceQsehraSource,
    /// Actual permitted benefits and covered people
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 1, max = 12)]
    #[libertas_unordered]
    pub months: Vec<MarketplaceQsehraMonth>,
    /// Actual Marketplace determinations
    #[libertas_unordered]
    pub determinations: Vec<MarketplaceQsehraDetermination>,
}

/// Documentary income components for the employee's own household
/// Use the employee's actual return, including both spouses' components on a joint return. Separate spouses' returns are not combined. Other required family returns are entered separately in the household members below; never use the claiming parent's income here.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceQsehraIncome {
    /// Adjusted gross income, Form1040 line11
    #[libertas_money("USD", 2)]
    pub agi: i64,
    /// Tax-exempt interest, Form1040 line2a
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub exempt_interest: i64,
    /// Excluded foreign income and housing, Form2555 lines45 and50
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub foreign_exclusion: i64,
    /// Gross Social Security, Form1040 line6a
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub social: i64,
    /// Taxable Social Security, Form1040 line6b
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub taxable_social: i64,
}

/// Employee household return source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MarketplaceQsehraIncomeSource {
    /// Use this person's required return already entered on Marketplace
    RequiredReturn,
    /// Actual documentary return components not already entered
    Documentary {
        /// Actual income components
        components: MarketplaceQsehraIncome,
    },
}

/// Dependent employee household member
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceQsehraHouseholdMember {
    /// Actual dependent of this employee, using an existing People identity
    #[libertas_unique(1)]
    pub person: i64,
    /// Income source if required to file because of income
    /// Leave absent only when this person is not required to file; refund-only filing does not add household income.
    /// [NullText]
    /// This person is not required to file because of income
    /// [SetText]
    /// This person has a required income return
    pub income: Option<MarketplaceQsehraIncomeSource>,
}

/// Actual dependent employee household
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum MarketplaceQsehraEmployeeHousehold {
    /// All actual spouse and dependent identities are available on People
    /// An empty list and no spouse explicitly mean no such relatives. These relationships are not inferred from people covered by a plan or from the claiming parent's family.
    Known {
        /// Actual spouse, if any
        /// [NullText]
        /// This employee has no spouse
        /// [SetText]
        /// This employee's actual spouse is
        spouse: Option<i64>,
        /// Actual dependents and their required-return sources
        dependents: Vec<MarketplaceQsehraHouseholdMember>,
    },
    /// An actual spouse or dependent is not available on People
    /// This precise employee source remains unavailable; do not omit the person or invent an identity.
    OtherPeople,
}

/// Dependent employee QSEHRA source facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceQsehraEmployeeFacts {
    /// Actual employee, using an existing claimed dependent People identity
    #[libertas_number(min = 1)]
    #[libertas_unique(1)]
    pub employee: i64,
    /// Employee's own or joint-return income source
    /// [NullText]
    /// Employee household income source is not yet available
    /// [SetText]
    /// Employee household income source is available
    pub income: Option<MarketplaceQsehraIncomeSource>,
    /// Actual employee spouse and dependents
    /// Absence means unanswered; Known with no relatives is an explicit empty household.
    /// [NullText]
    /// Employee household relationships are not yet available
    /// [SetText]
    /// Employee household relationships are available
    pub household: Option<MarketplaceQsehraEmployeeHousehold>,
}

/// Monthly QSEHRA benefit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceQsehraMonth {
    /// Coverage month, January 1 through December 12
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Age-applicable self-only monthly permitted benefit
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub self_only: i64,
    /// Actual current-year monthly permitted benefit, not reimbursements
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub actual: i64,
    /// Actual self-only employee SLCSP
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub self_benchmark: i64,
    /// People covered by this arrangement
    #[libertas_size(min = 1)]
    #[libertas_unordered]
    pub covered: Vec<i64>,
}

/// Actual Marketplace affordability determination
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct MarketplaceQsehraDetermination {
    /// First effective month
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    pub from: u32,
    /// Last effective month
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    pub to: u32,
    /// Accurate permitted-benefit information was supplied
    pub accurate_disclosure: bool,
    /// Marketplace determined this arrangement unaffordable
    pub unaffordable: bool,
    /// Actual determination date
    #[libertas_date_only]
    pub made_on: u32,
}

/// Premarriage coverage family
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceMarriageFamily {
    /// Claimed dependents who separately qualify as this spouse’s dependents for the year, assigned once
    #[libertas_unordered]
    pub dependents: Vec<i64>,
    /// Actual Forms1095-A sent to this spouse, or sent to neither spouse for this alternative family. Consume each saved statement once; separate recipient statements may have the same insurance policy number
    #[libertas_unordered]
    pub premarriage_policies: Vec<i64>,
    /// Actual alternative-family benchmark corrections when printed amounts are not applicable
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 0, max = 12)]
    #[libertas_unordered]
    pub benchmarks: Vec<MarketplaceBenchmarkMonth>,
}

/// Enrollment income estimate
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
pub enum MarketplaceIncomeQualification {
    /// Use actual household income
    Ordinary,
    /// Marketplace estimated at least 100% of the poverty line
    MarketplaceEstimate {
        /// Marketplace enrollment estimate was at least 100% FPL
        estimated_at_least_100: bool,
        /// You supplied accurate information at enrollment
        accurate_disclosure: bool,
        /// The estimate was not based on intentional or reckless disregard of facts
        not_intentional_or_reckless: bool,
    },
}

/// Statement status
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
pub enum MarketplaceStatementStatus {
    /// Original
    Original,
    /// Corrected replacement of this same source
    Corrected,
    /// Void; no policy amounts contribute
    Void,
}

/// Lawful enrollment premium source
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
pub enum MarketplaceLawfulPremiumSource {
    /// Actual Marketplace premium excluding unlawful enrollees
    Marketplace {
        /// Applicable enrollment premium
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
    },
    /// Same policy and same lawful covered set in an authorized reference month
    ReferenceMonth {
        /// Coverage month, January 1 through December 12
        // Tax structure: calendar months are January (1) through December (12).
        #[libertas_number(min = 1, max = 12)]
        month: u32,
    },
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
pub enum MarketplacePersonIdentity {
    /// Person in another tax family
    OtherFamily {
        /// Other tax family
        #[libertas_number(min = 1)]
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
pub enum MarketplaceNonreturnFamily {
    /// Another tax family
    OtherFamily {
        /// Other tax family
        #[libertas_number(min = 1)]
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
pub enum MarketplaceFamilyRef {
    /// This return
    ThisReturn,
    /// Other tax family
    Other {
        /// Other tax family
        #[libertas_number(min = 1)]
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
pub enum MarketplaceAttester {
    /// This return
    ThisReturn,
    /// Another tax family
    OtherFamily {
        /// Other tax family
        #[libertas_number(min = 1)]
        family: i64,
    },
    // Form 1095-A does not identify the actual income attester. Missing Exchange
    // or other-family evidence retains PTC_SOURCE_ATTESTER_REQUIRED.
    /// Not yet established
    Unknown,
}

/// Enrollment basis
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
pub enum MarketplaceEnrollmentBasis {
    /// Ordinary enrollment
    Ordinary,
    /// Birth; date comes from People
    Birth,
    /// Adoption
    Adoption {
        /// Actual adoption date
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
        event: u32,
    },
    /// Foster placement
    Foster {
        /// Actual foster-placement date
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
        event: u32,
    },
    /// Court order
    CourtOrder {
        /// Actual court-order date
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
        event: u32,
    },
    /// Income-only special enrollment, not eligible for 2026 PTC
    IncomeOnlySpecialEnrollment,
    /// Enrollment supported by a qualifying change
    QualifyingSpecialEnrollment {
        /// Actual qualifying event date
        #[libertas_date_only]
        // Tax structure: this event or eligibility period must fall by the end of the return year.
        #[libertas_number(min = $TAX_YEAR_START_DATE, max = $TAX_YEAR_END_DATE)]
        event: u32,
        /// Qualifying change
        kind: MarketplaceEventKind,
    },
}

/// Qualifying enrollment change
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
pub enum MarketplaceEventKind {
    /// Marriage
    Marriage,
    /// Birth or adoption
    BirthOrAdoption,
    /// Loss of coverage
    LossOfCoverage,
    /// Move
    Move,
    /// Other qualifying change
    OtherQualifyingChange,
}

/// Monthly tax eligibility
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
pub enum MarketplacePersonEligibility {
    /// Eligible for the premium tax credit
    Eligible,
    /// Other tax-eligible minimum essential coverage
    OtherMec {
        /// Actual applicable coverage eligibility
        kind: MarketplaceMecKind,
    },
    /// An affordable QSEHRA through an employee on another return bars this person as that employee’s spouse or dependent; this is not minimum essential coverage
    AffordableExternalQsehra,
    /// Not lawfully present
    NotLawfullyPresent,
    /// Incarcerated
    Incarcerated,
    // The policy statement does not prove person-month tax eligibility; missing
    // actual coverage/eligibility evidence retains PTC_SOURCE_ELIGIBILITY_REQUIRED.
    /// Eligibility not yet established
    Unknown,
}

/// Other coverage
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
pub enum MarketplaceMecKind {
    /// Affordable employee coverage
    EmployerEmployee,
    /// Affordable family coverage under the family-member test
    EmployerFamily,
    /// Medicare
    Medicare,
    /// Medicaid or CHIP
    MedicaidOrChip,
    /// TRICARE
    Tricare,
    /// Other eligible government coverage
    OtherGovernment,
    /// Other eligible minimum essential coverage
    OtherEligibleMec,
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
pub enum MarketplaceAllocationKind {
    /// Former-spouse married months, then any other claiming families
    FormerSpouses {
        /// Actual former-spouse pair
        pair: MarketplaceMaritalPair,
        /// First spouse's share before third-family allocation
        agreement: MarketplaceAgreement,
        /// Other claiming families
        #[libertas_unordered]
        third_families: Vec<MarketplaceMaritalFamilyShare>,
    },
    /// Spouses filing separately, after any other claiming-family allocation
    MarriedSeparate {
        /// Actual separate-return spouse pair
        pair: MarketplaceMaritalPair,
        /// First spouse's actual applicable coverage-family benchmark
        one_benchmark: MarketplaceBenchmarkSource,
        /// Second spouse's actual applicable coverage-family benchmark
        two_benchmark: MarketplaceBenchmarkSource,
        /// Other claiming families
        #[libertas_unordered]
        third_families: Vec<MarketplaceMaritalFamilyShare>,
    },
    /// Shared policy without APTC; applicable-family benchmark ratio
    NoAdvance {
        /// Every covered tax family and its own applicable benchmark
        #[libertas_size(min = 1)]
        #[libertas_unordered]
        families: Vec<MarketplaceNoAdvanceShare>,
    },
    /// Other shared policy with APTC
    OtherShared {
        /// Covered tax families and actual agreement
        #[libertas_size(min = 1)]
        #[libertas_unordered]
        families: Vec<MarketplaceFamilyShare>,
    },
}

/// Allocation agreement
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
pub enum MarketplaceAgreement {
    /// Actual common agreement
    Agreed {
        /// Agreed fraction
        share: MarketplaceFraction,
    },
    /// No agreement; use the prescribed default
    NoAgreement,
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
pub enum MarketplaceBenchmarkSource {
    /// Applicable printed benchmark
    Statement {
        /// Accepted policy
        #[libertas_number(min = 1)]
        policy: i64,
    },
    /// Actual applicable Marketplace benchmark
    Marketplace {
        /// Applicable benchmark
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
    },
    /// Authorized same-lawful-covered-set reference month
    ReferenceMonth {
        /// Accepted source policy
        #[libertas_number(min = 1)]
        policy: i64,
        /// Coverage month, January 1 through December 12
        // Tax structure: calendar months are January (1) through December (12).
        #[libertas_number(min = 1, max = 12)]
        month: u32,
    },
}

/// Payment and coverage treatment
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
pub enum MarketplacePaymentCoverage {
    /// Paid by the applicable unextended deadline
    PaidByDeadline,
    /// First grace-period month
    FirstGraceMonth,
    /// Issuer payment threshold met
    ThresholdMet {
        /// Issuer actually provided coverage
        issuer_provided: bool,
    },
    /// Declared-emergency order prevented termination
    EmergencyOrder {
        /// Issuer actually provided coverage
        issuer_provided: bool,
        /// Actual emergency-order date
        #[libertas_date_only]
        order_date: u32,
    },
    /// Issuer did not provide qualifying coverage
    NoQualifyingCoverage {
        /// APTC actually returned by issuer
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        issuer_returned_advance: i64,
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
pub enum MarketplaceQsehraSource {
    /// Accepted W-2 code FF
    W2 {
        /// Employee-owned W-2
        #[libertas_number(min = 1)]
        document: i64,
    },
    /// Employer notice when no W-2 exists
    Notice {
        /// Employer
        #[libertas_size(min = 1)]
        employer: String,
        /// Current-year permitted benefit, excluding prior carryover
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
pub enum MarketplaceMarriage {
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
        you: MarketplaceMarriageFamily,
        /// Your spouse's actual premarriage coverage family
        spouse: MarketplaceMarriageFamily,
    },
}

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
    pub forms: Vec<MarketplacePolicyRecordPage>,
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
    instruction = "Read one Form 1095-A exactly. Check the calendar year against the supplied year and report a mismatch. Read the line 3 issuer and line 2 policy number as printed, preserving punctuation and leading zeros. Read the corrected or void checkbox as its statement status; otherwise Original. Always return ByMonth with one row for each Part III monthly line 21 through 32 that shows any amount, mapping January to 1 through December 12. Ignore line 33 annual totals. Read columns A premium, B SLCSP and C advance. A populated row with blank or illegible A is an error. A clearly blank B is null, never numeric zero; blank C is zero. Conflicting or illegible amounts are errors. Copies of the same form are one source. Import only issuer, policy_number, amounts and status; never infer coverage-family identities, eligibility, payment, allocation or other facts absent from the statement.",
    validation = "$.document_facts",
    doc_type = "application/pdf,image/*"
)]
pub struct MarketplacePolicyRecordPage {
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
    /// Actual premiums excluding unlawful enrollees
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 0, max = 12)]
    #[libertas_unordered]
    pub lawful_premiums: Vec<MarketplaceLawfulPremiumMonthPage>,
    /// Family that actually enrolled the policy
    pub enrolling: MarketplaceFamilyRefPage,
    /// Original, corrected replacement or void statement
    #[libertas_default(Original)]
    #[libertas_llm_doc_field]
    pub status: MarketplaceStatementStatus,
    /// Actual covered people by month
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 1, max = 12)]
    #[libertas_unordered]
    pub coverage: Vec<MarketplacePolicyCoveragePage>,
    /// Shared-family allocation periods
    #[libertas_unordered]
    pub allocation: Vec<MarketplaceAllocationPeriodPage>,
    /// Actual payment problems or appeal facts
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 0, max = 12)]
    #[libertas_unordered]
    pub payments: Vec<MarketplacePaymentMonthPage>,
    /// Actual extra premium for children who are not medical dependents
    /// Complete only for covered children under 27 whose premiums are not medical expenses for this return. Actual cost and funding evidence is independent of the silver-plan allocation; an actual zero differs from an unknown cost. This information is not printed on Form1095-A and is not inferred by import.
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 0, max = 12)]
    #[libertas_unordered]
    pub medical_costs: Vec<MarketplaceMedicalCostMonth>,
}

/// Lawful covered-family enrollment premium
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceLawfulPremiumMonthPage {
    /// Coverage month, January 1 through December 12
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
    #[libertas_number(min = 0)]
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
    /// Coverage month, January 1 through December 12
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
    /// Coverage month, January 1 through December 12
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
    #[libertas_number(min = 0)]
    pub benchmark: i64,
}

/// Eligible coverage-family benchmark components
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceBenchmarkMonthPage {
    /// Coverage month, January 1 through December 12
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
    /// Coverage month, January 1 through December 12
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Premium received by issuer, including APTC
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub paid: i64,
    /// Same-year premium refunds
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub refunded: i64,
    /// Actual tax-free premium payments or reimbursements attributable to this return
    /// After any shared-policy allocation, enter premiums funded by tax-free employer/HRA payments, qualified HSA/FSA spending or tax-free public-safety retirement amounts. This is actual expense funding, not a QSEHRA permitted benefit or APTC. Do not subtract it from printed Form1095-A amounts.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub tax_free_premiums: i64,
    /// Part of that actual tax-free funding attributable to coverage-family premiums
    /// Needed for a self-employed policy with mixed coverage when tax-free funding applies to particular people. Use actual reimbursement records; do not guess that the silver-plan ratio allocates a person's reimbursement.
    /// [NullText]
    /// A separate coverage-family funding allocation is not needed
    /// [SetText]
    /// The tax-free funding attributable to coverage-family premiums is
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub tax_free_coverage: Option<i64>,
    /// Unpaid premium
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    /// Coverage month, January 1 through December 12
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    #[libertas_unique(1)]
    pub month: u32,
    /// Age-applicable self-only monthly permitted benefit
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub self_only: i64,
    /// Actual current-year monthly permitted benefit, not reimbursements
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub actual: i64,
    /// Actual self-only employee SLCSP
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        amount: i64,
    },
    /// Authorized same-lawful-covered-set reference month
    ReferenceMonth {
        /// Accepted source policy
        #[libertas_number(min = 1)]
        #[libertas_enum_source("$.ptc_policies")]
        policy: i64,
        /// Coverage month, January 1 through December 12
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
        #[libertas_number(min = 0)]
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

record_page_conversions!(
    MarketplacePolicyRecord,
    MarketplacePolicyRecordPage,
    [],
    |value, Target, Source| {
        let Source {
            id,
            issuer,
            policy_number,
            amounts,
            lawful_premiums,
            enrolling,
            status,
            coverage,
            allocation,
            payments,
            medical_costs,
        } = value;
        Target {
            id,
            issuer,
            policy_number,
            amounts,
            lawful_premiums: lawful_premiums.into_iter().map(Into::into).collect(),
            enrolling: enrolling.into(),
            status,
            coverage: coverage.into_iter().map(Into::into).collect(),
            allocation: allocation.into_iter().map(Into::into).collect(),
            payments: payments.into_iter().map(Into::into).collect(),
            medical_costs,
        }
    }
);

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
impl Default for MarketplaceFacts {
    fn default() -> Self {
        Self {
            residence: crate::MarketplaceResidence::Contiguous,
            income_qualification: MarketplaceIncomeQualification::Ordinary,
            forms: Vec::new(),
            other_families: Vec::new(),
            covered: Vec::new(),
            memberships: Vec::new(),
            attestations: Vec::new(),
            eligibility: Vec::new(),
            benchmarks: Vec::new(),
            qsehra: Vec::new(),
            employee_qsehra_facts: Vec::new(),
            dependent_qsehra_employees: Vec::new(),
            marriage: MarketplaceMarriage::Regular,
            separate: crate::SeparateRelief::NotSeparate,
            dependents: Vec::new(),
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
