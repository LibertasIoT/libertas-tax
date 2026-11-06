// Accepted Marketplace assignment and benchmark source facts.
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Premium tax credit and self-employed health deduction method
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[libertas_inline_choices]
pub enum HealthPtcMethod {
    /// Simplified method
    /// Uses the IRS two-pass simplified calculation, including all affected deductions.
    #[default]
    Simplified,
    /// Iterative method
    /// Refigures both amounts until each changes by less than one dollar. If this method cannot converge, the complete simplified method is used with a review note.
    Iterative,
}

/// Actual entity reimbursement of this assigned policy month
/// These are funded subsets of the canonical policy payment already allocated
/// to this business, not additional premiums or a second policy ledger.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ReimbursedPolicyPremiums {
    /// Included policy premiums actually reimbursed by this entity
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub premiums: i64,
    /// Included reimbursement attributable to the coverage family's specified premiums
    /// Use the actual funded portion; the SLCSP ratio does not establish which person's premium an entity reimbursed.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub coverage: i64,
    /// Included reimbursement of actual nonmedical-child premium cost
    /// Required only when a positive child cost intersects a partial entity reimbursement. This funded subset is independent of the coverage-family silver-plan allocation.
    /// [NullText]
    /// A separate child-cost reimbursement amount is not needed
    /// [SetText]
    /// The included child-cost reimbursement is
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub child: Option<i64>,
}

/// Marketplace policy established under a business
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthPolicyAllocation {
    /// Saved Marketplace statement
    #[libertas_number(min = 1)]
    pub policy: i64,
    /// Coverage month
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Business or entity under which the plan is established
    #[libertas_number(min = 1)]
    pub activity: i64,
    /// Parts allocated to this business
    /// Relative parts implement your actual consistent reasonable allocation among businesses. They allocate existing premiums and the same credit; they are not another premium payment.
    #[libertas_number(min = 1)]
    #[libertas_default(1)]
    pub parts: u32,
    /// Actual entity reimbursement of this assigned policy month
    /// Complete only for an entity Reimbursed plan; partial reimbursements require actual source attribution, including zero. Premium money remains in the saved Marketplace statement.
    #[libertas_default($null)]
    pub reimbursed: Option<ReimbursedPolicyPremiums>,
}

/// Actual benchmarks for a policy covering people outside the coverage family
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthPolicyBenchmark {
    /// Saved Marketplace statement
    #[libertas_number(min = 1)]
    pub policy: i64,
    /// Coverage month
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Actual applicable SLCSP for the policy's coverage-family members
    pub coverage: MarketplaceBenchmarkSource,
    /// Actual applicable SLCSP for the policy's other enrolled family members
    pub noncoverage: MarketplaceBenchmarkSource,
}

/// Shared Marketplace and self-employed health facts
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct HealthMarketplace {
    /// Calculation method
    #[libertas_default(Simplified)]
    pub method: HealthPtcMethod,
    /// Actual policy assignments, without repeated premium amounts
    #[libertas_unordered]
    pub policies: Vec<HealthPolicyAllocation>,
    /// Coverage-family allocation sources when policy coverage is mixed
    #[libertas_unordered]
    pub benchmarks: Vec<HealthPolicyBenchmark>,
}

impl ReimbursedPolicyPremiums {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if self.premiums < 0 {
            return Some(
                ValidationIssue::new("PROBLEM_INVALID_HEALTH_POLICY_ASSIGNMENT")
                    .at_field("premiums"),
            );
        }
        if !(0..=self.premiums).contains(&self.coverage) {
            return Some(
                ValidationIssue::new("HEALTH_REIMBURSED_COVERAGE_ABOVE_TOTAL").at_field("coverage"),
            );
        }
        self.child.is_some_and(|child| !(0..=self.premiums).contains(&child)).then_some(
            ValidationIssue::new("HEALTH_POLICY_CHILD_FUNDING_ABOVE_COST").at_field("child"),
        )
    }
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
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
    /// Authorized same-lawful-covered-set reference month
    ReferenceMonth {
        /// Accepted source policy
        #[libertas_number(min = 1)]
        policy: i64,
        /// Month of coverage, 1 for January through 12 for December
        // Tax structure: calendar months are January (1) through December (12).
        #[libertas_number(min = 1, max = 12)]
        month: u32,
    },
}
