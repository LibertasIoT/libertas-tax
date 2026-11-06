use crate::SeparateRelief;
pub use crate::health_marketplace_source::MarketplaceBenchmarkSource;
use crate::kernel::ratio::exact::{Exact, ratio};
// Canonical Marketplace sources and direct tax facts.
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
    /// Printed recipient and issuer names
    /// [NullText]
    /// No printed names entered
    /// [SetText]
    /// Printed names
    #[libertas_default($null)]
    pub printed: Option<crate::PrintedDocumentNames>,
    /// Accepted source identity; a new unsaved source has zero
    #[libertas_number(min = 1)]
    #[libertas_hidden]
    #[libertas_read_only]
    pub id: i64,
    /// Form 1095-A as printed
    pub form: crate::Form1095APaper,
    /// Actual premiums excluding unlawful enrollees
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 0, max = 12)]
    #[libertas_unordered]
    pub lawful_premiums: Vec<MarketplaceLawfulPremiumMonth>,
    /// Family that actually enrolled the policy
    pub enrolling: MarketplaceFamilyRef,
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
    /// Month of coverage, 1 for January through 12 for December
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub child_increase: Option<i64>,
    /// Included actual tax-free funding for that child cost
    /// A subset of this policy month's existing tax-free funding, required when that funding and the child's positive added cost intersect. Coverage-family funding and child funding are different source facts.
    /// [NullText]
    /// A separate child-cost tax-free funding amount is not needed
    /// [SetText]
    /// The tax-free funding included in that child cost is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub tax_free_child: Option<i64>,
}

/// Lawful covered-family enrollment premium
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceLawfulPremiumMonth {
    /// Month of coverage, 1 for January through 12 for December
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
    /// Month of coverage, 1 for January through 12 for December
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
    /// Month of coverage, 1 for January through 12 for December
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub benchmark: i64,
}

/// Eligible coverage-family benchmark components
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct MarketplaceBenchmarkMonth {
    /// Month of coverage, 1 for January through 12 for December
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
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub agi: i64,
    /// Tax-exempt interest, Form1040 line2a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub exempt_interest: i64,
    /// Excluded foreign income and housing, Form2555 lines45 and50
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub foreign_exclusion: i64,
    /// Gross Social Security, Form1040 line6a
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub social: i64,
    /// Taxable Social Security, Form1040 line6b
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Same policy and same lawful covered set in an authorized reference month
    ReferenceMonth {
        /// Month of coverage, 1 for January through 12 for December
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
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
            separate: SeparateRelief::NotSeparate,
            dependents: Vec::new(),
        }
    }
}

impl MarketplaceFraction {
    pub(crate) fn exact(&self) -> Option<Exact> {
        self.checked_exact().ok()
    }

    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        self.checked_exact().err()
    }

    pub(crate) fn checked_exact(&self) -> Result<Exact, crate::validation::ValidationIssue> {
        if self.numerator > self.denominator || self.denominator == 0 {
            Err(crate::validation::ValidationIssue::new("PTC_SHARE_EXCEEDS_WHOLE").at_field(
                if self.numerator > self.denominator { "numerator" } else { "denominator" },
            ))
        } else {
            ratio(i64::from(self.numerator), i64::from(self.denominator)).ok_or_else(|| {
                crate::validation::ValidationIssue::new("PTC_SHARE_EXCEEDS_WHOLE")
                    .at_field("denominator")
            })
        }
    }
}
