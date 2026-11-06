use crate::expense_source::BusinessExpenseCategory;
use crate::kernel::phase;
use crate::validation::ValidationIssue;
use alloc::string::String;
use alloc::vec::Vec;
use libertas_macros::*;

/// Form 1099-PATR issuer facts and actual recipient records
#[derive(
    Clone, Default, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct FederalCooperative {
    /// Printed box 1
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub patronage: i64,
    /// Printed box 2
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub nonpatronage: i64,
    /// Printed box 3
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub per_unit_retains: i64,
    /// Printed box 4
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub withholding: i64,
    /// Printed box 5
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub redeemed: i64,
    /// Printed box 6
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_199a_g: i64,
    /// Printed box 7
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_payments: i64,
    /// Printed box 8
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// Printed box 9
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub sstb_items: i64,
    /// Printed box 13 checkbox
    #[libertas_default(false)]
    pub specified_cooperative: bool,
    /// Actual activity shares
    pub allocations: Vec<CoopAllocation>,
    /// Printed10–12 and actual source facts
    pub credit_facts: CoopCreditFacts,
    /// Printed box 12 other credits and deductions
    // IRS Form 1099-PATR instructions require separately identified types and amounts:
    // https://www.irs.gov/instructions/i1099ptr
    // This aggregate alone does not identify a credit or establish entitlement.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub other_credits_and_deductions: i64,
}
impl FederalCooperative {
    pub(crate) fn distributions(&self) -> i64 {
        self.patronage + self.nonpatronage + self.per_unit_retains + self.redeemed
    }
    pub(crate) fn taxable(&self) -> i64 {
        if self.allocations.is_empty() {
            self.distributions()
        } else {
            self.allocations.iter().map(CoopAllocation::ordinary).sum()
        }
    }
    pub(crate) fn nonqualified(&self) -> i64 {
        if self.allocations.is_empty() {
            return phase::floor(self.taxable() - self.qualified_items, 0);
        }
        let qualified: i64 = self
            .allocations
            .iter()
            .filter_map(CoopAllocation::retained_qualification)
            .map(|v| v[1])
            .sum();
        phase::floor(self.taxable() - qualified, 0)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        for (field, value) in [
            ("patronage", self.patronage),
            ("nonpatronage", self.nonpatronage),
            ("per_unit_retains", self.per_unit_retains),
            ("redeemed", self.redeemed),
            ("withholding", self.withholding),
            ("section_199a_g", self.section_199a_g),
            ("qualified_payments", self.qualified_payments),
            ("qualified_items", self.qualified_items),
            ("sstb_items", self.sstb_items),
        ] {
            if !(0..=100_000_000_000).contains(&value) {
                return Some(ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field(field));
            }
        }
        if self.withholding > self.distributions() {
            return Some(ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("withholding"));
        }
        if self.qualified_items + self.sstb_items > self.distributions() {
            return Some(
                ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("qualified_items"),
            );
        }
        if self.allocations.len() > 100 {
            return Some(ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("allocations"));
        }
        for (index, allocation) in self.allocations.iter().enumerate() {
            if let Some(issue) = allocation.issue() {
                return Some(issue.in_list("allocations", index));
            }
        }
        self.credit_facts.issue().map(|issue| issue.in_field("credit_facts"))
    }
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
}

/// Distribution on the issuer statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoopDistributionBox {
    /// Box 1 patronage dividends
    Patronage,
    /// Box 2 nonpatronage distributions
    Nonpatronage,
    /// Box 3 per-unit retain allocations
    PerUnit,
    /// Box 5 nonqualified redemptions
    Redeemed,
}

/// Actual portions of printed distribution boxes
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct CoopDistributionAmounts {
    /// Portion of box 1
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub patronage: i64,
    /// Portion of box 2
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub nonpatronage: i64,
    /// Portion of box 3
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub per_unit_retains: i64,
    /// Portion of box 5
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub redeemed: i64,
}

/// Economic source of the income, not an elective tax character
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoopIncomeUse {
    /// Products sold or marketed through the cooperative
    ProductionSales,
    /// Refund of deductible business purchases
    DeductiblePurchases,
    /// Another described ordinary business source
    Other {
        /// Actual source description
        #[libertas_size(min = 1)]
        description: String,
    },
    /// Records do not establish farming-production gross character
    Unresolved,
}

/// Why this distribution relates to property
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoopPropertyTreatment {
    /// Purchase of property owned during this year
    Purchase,
    /// Additional proceeds of a sale made this year
    CurrentSale,
    /// Property not owned at any time this year
    EarlierDisposition {
        /// Actual disposition facts and tax treatment
        #[libertas_size(min = 1)]
        source: String,
        /// Existing character owner must determine the amount
        character: CoopEarlierCharacter,
    },
}

/// Known property character from the original sale owner
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoopEarlierCharacter {
    /// Long-term capital asset with deductible loss
    DeductibleLongTermCapital,
    /// Other business disposition with established ordinary treatment
    OrdinaryBusiness,
    /// Personal capital property with no deductible loss
    PersonalNondeductible,
    /// Exact sale or character history is missing
    Unresolved,
}

/// Property refund source facts, accepted
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoopPropertyRefund {
    /// Printed distribution containing this amount
    pub distribution: CoopDistributionBox,
    /// Accepted property identity
    #[libertas_number(min = 1)]
    pub property: Option<i64>,
    /// Actual purchase or disposition source record
    #[libertas_size(min = 1)]
    pub source: String,
    /// Actual property-related distribution
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Issuer-qualified non-SSTB part included in this property amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// Issuer-qualified SSTB part included in this property amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub sstb_items: i64,
    /// Qualified-payment part included in this property amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_payments: i64,
    /// Actual purchase or disposition relationship
    pub treatment: CoopPropertyTreatment,
}

/// Property refund source facts, editor
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoopPropertyRefundInput {
    /// Printed distribution containing this amount
    pub distribution: CoopDistributionBox,
    /// Accepted property identity
    /// [NullText]
    /// The property has not been linked
    /// [SetText]
    /// The refund relates to
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.cooperative_properties")]
    pub property: Option<i64>,
    /// Actual purchase or disposition source record
    #[libertas_size(min = 1)]
    pub source: String,
    /// Actual property-related distribution
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Issuer-qualified non-SSTB part included in this property amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// Issuer-qualified SSTB part included in this property amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub sstb_items: i64,
    /// Qualified-payment part included in this property amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_payments: i64,
    /// Actual purchase or disposition relationship
    pub treatment: CoopPropertyTreatment,
}

/// Original nonqualified notice ownership and basis
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoopNoticeHolding {
    /// Original patron, or inherited directly from that original patron
    OriginalOrInherited,
    /// Acquired from another holder
    Acquired {
        /// Actual notice basis
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        basis: i64,
        /// Actual acquisition and source facts
        #[libertas_size(min = 1)]
        source: String,
    },
    /// Required ownership or basis history missing
    Unresolved,
}

/// Original issue represented by the nonqualified redemption
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoopNoticeKind {
    /// Written notice paid as a patronage dividend
    Patronage,
    /// Written notice paid from nonpatronage sources
    Nonpatronage,
    /// Per-unit retain certificate
    PerUnit,
    /// Actual original notice character has not been established
    Unresolved,
}

/// Nonqualified notice or retain disposition facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoopRedemption {
    /// Actual original notice or certificate type
    #[libertas_default(Unresolved)]
    pub kind: CoopNoticeKind,
    /// Box 2 or5 containing this redemption
    pub distribution: CoopDistributionBox,
    /// Actual reported redemption part
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Notice original stated dollar amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub stated_value: i64,
    /// Actual ownership history
    #[libertas_default(Unresolved)]
    pub holding: CoopNoticeHolding,
    /// Notice source reference
    #[libertas_size(min = 1)]
    pub source: String,
}

/// Expense and wage facts for qualified payments
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoopPatronAllocation {
    /// No allocation is needed because actual eligible W-2 wages are zero
    NoWages,
    /// Below-threshold safe harbor uses owner-calculated eligible gross ratio
    SafeHarbor,
    /// Actual consistent books allocation, not computed QBI
    Books {
        /// Deductible business expenses attributable to qualified payments, excluding wages and separately attributed below-line deductions
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        expenses_excluding_wages: i64,
        /// Actual W-2 wage expense attributable to qualified payments
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        wages: i64,
        /// Actual source-attributable half SE/health/retirement or other QBI deductions already calculated by their owner
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        other_deductions: i64,
        /// Consistent allocation records and method
        #[libertas_size(min = 1)]
        source: String,
    },
    /// Required facts have not been established
    Unresolved,
}

/// Related deductible expense already entered on the activity
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoopSstbExpense {
    /// Existing expense category paid to this cooperative
    pub category: BusinessExpenseCategory,
    /// Actual related paid expense already entered on the activity; the SSTB allocation is capped at its related SSTB income
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Invoice/source evidence
    #[libertas_size(min = 1)]
    pub source: String,
}

/// Actual written allocation notice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoopNoticeStatus {
    /// Notice received with the required issuer designation and payment-period timing
    Received {
        /// Issuer notice reference and date/timing evidence
        #[libertas_size(min = 1)]
        source: String,
    },
    /// No qualifying designation has been established
    Missing,
}

/// Issuer-supported qualified parts of an adjustment
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct CoopQualification {
    /// Qualified-payment part
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_payments: i64,
    /// Non-SSTB qualified items
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// SSTB qualified items
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub sstb_items: i64,
}

/// Accepted activity allocation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoopAllocation {
    /// Accepted allocation identity; zero only for a new unsaved row
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub id: i64,
    /// Actual issuer distributions attributable to this activity
    pub distributions: CoopDistributionAmounts,
    /// Disjoint personal-purchase amounts within those boxes
    pub personal: CoopDistributionAmounts,
    /// Actual qualified parts included in the personal-purchase adjustment
    /// Leave absent when the source does not establish them. A documented zero is different from missing facts; no percentage is inferred.
    #[libertas_default($null)]
    pub personal_qualification: Option<CoopQualification>,
    /// Property distribution allocations
    pub properties: Vec<CoopPropertyRefund>,
    /// Economic character of remaining income
    #[libertas_default(Unresolved)]
    pub income_use: CoopIncomeUse,
    /// Box 7 qualified payments allocated by issuer/books
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_payments: i64,
    /// Box 8 non-SSTB items allocated by issuer/books
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// Box 9 SSTB items allocated by issuer/books
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub sstb_items: i64,
    /// Box 6 deduction allocated by issuer
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_199a_g: i64,
    /// Qualified payment and DPAD notice facts
    pub notice: Option<CoopNoticeStatus>,
    /// Activity-wide allocation method; only one governing record per activity
    pub patron_allocation: Option<CoopPatronAllocation>,
    /// Related existing deductible expenses
    pub sstb_expenses: Vec<CoopSstbExpense>,
    /// Notice dispositions included here
    /// Cover the entire box 5 redeemed amount with actual ownership, basis and stated-value records. Missing records leave the estimate incomplete.
    pub redemptions: Vec<CoopRedemption>,
}

/// Editable activity allocation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoopAllocationInput {
    /// Accepted allocation identity; zero only for a new unsaved row
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub id: i64,
    /// Actual issuer distributions attributable to this activity
    pub distributions: CoopDistributionAmounts,
    /// Disjoint personal-purchase amounts within those boxes
    pub personal: CoopDistributionAmounts,
    /// Actual qualified parts included in the personal-purchase adjustment
    /// Leave absent when the source does not establish them. A documented zero is different from missing facts; no percentage is inferred.
    /// [NullText]
    /// The qualified parts of personal purchases are not established
    /// [SetText]
    /// The documented qualified parts are
    #[libertas_default($null)]
    pub personal_qualification: Option<CoopQualification>,
    /// Property distribution allocations
    pub properties: Vec<CoopPropertyRefundInput>,
    /// Economic character of remaining income
    #[libertas_default(Unresolved)]
    pub income_use: CoopIncomeUse,
    /// Box 7 qualified payments allocated by issuer/books
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_payments: i64,
    /// Box 8 non-SSTB items allocated by issuer/books
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// Box 9 SSTB items allocated by issuer/books
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub sstb_items: i64,
    /// Box 6 deduction allocated by issuer
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub section_199a_g: i64,
    /// Qualified payment and DPAD notice facts
    /// [NullText]
    /// The qualifying-payment and deduction notice facts are not established
    /// [SetText]
    /// The notice status is
    #[libertas_default($null)]
    pub notice: Option<CoopNoticeStatus>,
    /// Activity-wide allocation method; only one governing record per activity
    /// [NullText]
    /// This record does not supply the governing patron allocation
    /// [SetText]
    /// The activity-wide allocation method is
    #[libertas_default($null)]
    pub patron_allocation: Option<CoopPatronAllocation>,
    /// Related existing deductible expenses
    pub sstb_expenses: Vec<CoopSstbExpense>,
    /// Notice dispositions included here
    /// Cover the entire box 5 redeemed amount with actual ownership, basis and stated-value records. Missing records leave the estimate incomplete.
    pub redemptions: Vec<CoopRedemption>,
}

/// Issuer designated component of box 10
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoopInvestmentKind {
    /// Section 47 rehabilitation
    Rehabilitation,
    /// Section 48 energy
    Energy,
    /// Section 48E clean electricity
    CleanElectricity,
    /// Another component, named benefit unavailable
    Other {
        /// Issuer credit section/type
        #[libertas_size(min = 1)]
        description: String,
    },
}

/// Actual issuer component of box 10
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoopInvestmentShare {
    /// Credit component identified by issuer
    pub kind: CoopInvestmentKind,
    /// Actual allocated component
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Issuer source evidence
    #[libertas_size(min = 1)]
    pub source: String,
}

/// Actual box 12 statement credit or deduction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoopOtherKind {
    /// Section 45Z passed-through clean fuel credit
    CleanFuel,
    /// Form 8941 passed-through small employer health credit
    SmallEmployerHealth,
    /// Form 8844 empowerment-zone employment credit
    EmpowermentZone,
    /// Form 8896 low-sulfur diesel production credit
    LowSulfurDiesel,
    /// Form 8932 differential-wage credit
    DifferentialWage,
    /// Form 8864 small-agri-biodiesel producer credit
    SmallAgriBiodiesel,
    /// Section 198 small-refiner cooperative deduction
    SmallRefinerDeduction,
    /// Another precisely described issuer item
    Other {
        /// Printed type/section
        #[libertas_size(min = 1)]
        description: String,
    },
}

/// Associated required income identified by the actual source owner
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CoopCreditIncome {
    /// No additional income inclusion under the applicable source rule
    None,
    /// Actual required inclusion not included in printed distribution boxes
    Inclusion {
        /// Actual source-owner amount
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// Authority and notice identifying the income
        #[libertas_size(min = 1)]
        source: String,
    },
    /// Applicable mandatory income has not been established
    Unresolved,
}

/// Accepted box 12 statement item
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoopOtherItem {
    /// Accepted statement-item identity; zero before source admission
    pub id: i64,
    /// Printed credit or deduction kind
    pub kind: CoopOtherKind,
    /// Printed allocation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Accepted activity generating this source
    #[libertas_number(min = 1)]
    pub activity: Option<i64>,
    /// Associated mandatory income status
    #[libertas_default(Unresolved)]
    pub income: CoopCreditIncome,
    /// Issuer notice and applicable originating-period facts
    #[libertas_size(min = 1)]
    pub source: String,
}

/// Editable box 12 statement item
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CoopOtherItemInput {
    /// Accepted statement-item identity; zero only for a new unsaved row
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub id: i64,
    /// Printed credit or deduction kind
    pub kind: CoopOtherKind,
    /// Printed allocation
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Accepted activity generating this source
    /// [NullText]
    /// The originating activity has not been linked
    /// [SetText]
    /// The originating activity is
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.cooperative_activities")]
    #[libertas_hidden]
    pub activity: Option<i64>,
    /// Associated mandatory income status
    #[libertas_default(Unresolved)]
    #[libertas_hidden]
    pub income: CoopCreditIncome,
    /// Issuer notice and applicable originating-period facts
    #[libertas_size(min = 1)]
    pub source: String,
}

/// Accepted cooperative credit facts
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct CoopCreditFacts {
    /// Actual issuer EIN, preserve printed identifier
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub payer_ein: String,
    /// Printed box 10
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment: i64,
    /// Printed box 11
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub work_opportunity: i64,
    /// Issuer breakdown of box 10
    pub investment_shares: Vec<CoopInvestmentShare>,
    /// Separate printed box 12 items
    pub other: Vec<CoopOtherItem>,
    /// Activity credit participation under existing passive owner
    // Unprovided source-activity participation records cannot be reconstructed from box amounts.
    // With the credit source otherwise established, null retains WARN_COOP_CREDIT_PASSIVE
    // and passive-credit treatment.
    pub participation: Option<bool>,
    /// Actual activity receiving box 10 and 11 credits
    #[libertas_number(min = 1)]
    pub activity: Option<i64>,
    /// Credit allocation notice
    pub notice: Option<CoopNoticeStatus>,
}

/// Editable cooperative credit facts
#[derive(
    Clone, Debug, Default, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct CoopCreditFactsInput {
    /// Actual issuer EIN, preserve printed identifier
    #[libertas_llm_doc_field(allow_empty = true)]
    #[libertas_size(min = 0)]
    #[libertas_default("")]
    pub payer_ein: String,
    /// Printed box 10
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub investment: i64,
    /// Printed box 11
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub work_opportunity: i64,
    /// Issuer breakdown of box 10
    pub investment_shares: Vec<CoopInvestmentShare>,
    /// Separate printed box 12 items
    pub other: Vec<CoopOtherItemInput>,
    /// Activity credit participation under existing passive owner
    /// [NullText]
    /// I have no participation records; the credits are treated as passive
    /// [SetText]
    /// My activity credit participation is
    // Unprovided source-activity participation records cannot be reconstructed from box amounts.
    // With the credit source otherwise established, null retains WARN_COOP_CREDIT_PASSIVE
    // and passive-credit treatment.
    #[libertas_hidden]
    pub participation: Option<bool>,
    /// Actual activity receiving box 10 and 11 credits
    /// [NullText]
    /// I am not claiming the box 10 or 11 credits
    /// [SetText]
    /// The credits belong to
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.cooperative_activities")]
    #[libertas_default($null)]
    #[libertas_hidden]
    pub activity: Option<i64>,
    /// Credit allocation notice
    /// [NullText]
    /// I have no credit allocation notice
    /// [SetText]
    /// The credit notice status is
    #[libertas_default($null)]
    #[libertas_hidden]
    pub notice: Option<CoopNoticeStatus>,
}

impl CoopDistributionAmounts {
    pub(crate) fn values(&self) -> [i64; 4] {
        [self.patronage, self.nonpatronage, self.per_unit_retains, self.redeemed]
    }
    pub(crate) fn total(&self) -> i64 {
        self.values().into_iter().sum()
    }
    pub(crate) fn part(&self, kind: &CoopDistributionBox) -> i64 {
        match kind {
            CoopDistributionBox::Patronage => self.patronage,
            CoopDistributionBox::Nonpatronage => self.nonpatronage,
            CoopDistributionBox::PerUnit => self.per_unit_retains,
            CoopDistributionBox::Redeemed => self.redeemed,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.values().into_iter().all(money)
    }
}
fn money(value: i64) -> bool {
    (0..=100_000_000_000).contains(&value)
}
fn source(value: &str) -> bool {
    !value.trim().is_empty()
}

impl CoopAllocation {
    // Known purchase refunds leave income only through their actual basis owner.
    // This value is not proof that an unmerged owner has supplied the missing tax.
    pub(crate) fn ordinary(&self) -> i64 {
        self.distributions.total()
            - self.personal.total()
            - self.properties.iter().map(|p| p.amount).sum::<i64>()
            - self
                .redemptions
                .iter()
                .map(|r| match r.holding {
                    CoopNoticeHolding::Acquired { basis, .. }
                        if r.amount >= basis && r.amount <= r.stated_value =>
                    {
                        basis
                    }
                    _ => 0,
                })
                .sum::<i64>()
    }
}

record_page_conversions!(CoopPropertyRefund, CoopPropertyRefundInput, [], |v, Target| {
    Target {
        distribution: v.distribution,
        property: v.property,
        source: v.source,
        amount: v.amount,
        qualified_items: v.qualified_items,
        sstb_items: v.sstb_items,
        qualified_payments: v.qualified_payments,
        treatment: v.treatment,
    }
});

record_page_conversions!(CoopOtherItem, CoopOtherItemInput, [], |v, Target| {
    Target {
        id: v.id,
        kind: v.kind,
        amount: v.amount,
        activity: v.activity,
        income: v.income,
        source: v.source,
    }
});

record_page_conversions!(CoopAllocation, CoopAllocationInput, [], |v, Target| {
    Target {
        id: v.id,
        distributions: v.distributions,
        personal: v.personal,
        personal_qualification: v.personal_qualification,
        properties: v.properties.into_iter().map(Into::into).collect(),
        income_use: v.income_use,
        qualified_payments: v.qualified_payments,
        qualified_items: v.qualified_items,
        sstb_items: v.sstb_items,
        section_199a_g: v.section_199a_g,
        notice: v.notice,
        patron_allocation: v.patron_allocation,
        sstb_expenses: v.sstb_expenses,
        redemptions: v.redemptions,
    }
});

record_page_conversions!(CoopCreditFacts, CoopCreditFactsInput, [], |v, Target| {
    Target {
        payer_ein: v.payer_ein,
        investment: v.investment,
        work_opportunity: v.work_opportunity,
        investment_shares: v.investment_shares,
        other: v.other.into_iter().map(Into::into).collect(),
        participation: v.participation,
        activity: v.activity,
        notice: v.notice,
    }
});

impl CoopAllocation {
    pub(crate) fn retained_qualification(&self) -> Option<[i64; 3]> {
        let mut v = [self.qualified_payments, self.qualified_items, self.sstb_items];
        if v.iter().any(|n| *n > 0)
            && self
                .redemptions
                .iter()
                .any(|r| matches!(r.holding, CoopNoticeHolding::Acquired {basis,..} if basis > 0))
        {
            return None;
        }
        if self.personal.total() > 0 && v.iter().any(|v| *v != 0) {
            let p = self.personal_qualification.as_ref()?;
            v[0] -= p.qualified_payments;
            v[1] -= p.qualified_items;
            v[2] -= p.sstb_items;
        }
        for p in &self.properties {
            v[0] -= p.qualified_payments;
            v[1] -= p.qualified_items;
            v[2] -= p.sstb_items;
        }
        (v.into_iter().all(|v| v >= 0) && v[1] + v[2] <= self.ordinary()).then_some(v)
    }
    #[cfg(test)]
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if !(0..=100_000_000_000).contains(&self.id)
            || !self.distributions.valid()
            || !self.personal.valid()
            || self.properties.len() > 100
            || self.redemptions.len() > 100
            || self.sstb_expenses.len() > 20
            || ![
                self.qualified_payments,
                self.qualified_items,
                self.sstb_items,
                self.section_199a_g,
            ]
            .into_iter()
            .all(money)
        {
            return Some(ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("distributions"));
        }
        if self.qualified_items + self.sstb_items > self.distributions.total() {
            return Some(
                ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("qualified_items"),
            );
        }
        if self.qualified_payments > self.distributions.total() {
            return Some(
                ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("qualified_payments"),
            );
        }
        if let Some(p) = &self.personal_qualification
            && (![p.qualified_payments, p.qualified_items, p.sstb_items].into_iter().all(money)
                || p.qualified_items + p.sstb_items > self.personal.total()
                || p.qualified_payments > self.personal.total()
                || p.qualified_items > self.qualified_items
                || p.sstb_items > self.sstb_items
                || p.qualified_payments > self.qualified_payments)
        {
            return Some(ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("distributions"));
        }
        for (index, p) in self.properties.iter().enumerate() {
            for (field, amount) in [
                ("amount", p.amount),
                ("qualified_items", p.qualified_items),
                ("sstb_items", p.sstb_items),
                ("qualified_payments", p.qualified_payments),
            ] {
                if !money(amount) {
                    return Some(
                        ValidationIssue::new("COOP_ALLOCATION_INVALID")
                            .at_field(field)
                            .in_list("properties", index),
                    );
                }
            }
            if p.qualified_items + p.sstb_items > p.amount {
                return Some(
                    ValidationIssue::new("COOP_PROPERTY_QBI_ABOVE_AMOUNT")
                        .at_field("qualified_items")
                        .in_list("properties", index),
                );
            }
            if p.qualified_payments > p.amount {
                return Some(
                    ValidationIssue::new("COOP_PROPERTY_PAYMENTS_ABOVE_AMOUNT")
                        .at_field("qualified_payments")
                        .in_list("properties", index),
                );
            }
            if !source(&p.source) {
                return Some(
                    ValidationIssue::new("COOP_ALLOCATION_INVALID")
                        .at_field("source")
                        .in_list("properties", index),
                );
            }
            if p.property.is_some_and(|id| !(1..=100_000_000_000).contains(&id)) {
                return Some(
                    ValidationIssue::new("COOP_ALLOCATION_INVALID")
                        .at_field("property")
                        .in_list("properties", index),
                );
            }
            if matches!(&p.treatment,CoopPropertyTreatment::EarlierDisposition{source:s,..} if !source(s))
            {
                return Some(
                    ValidationIssue::new("COOP_ALLOCATION_INVALID")
                        .at_field("source")
                        .in_field("treatment")
                        .in_list("properties", index),
                );
            }
        }
        // These known source totals are independent of unavailable retained
        // qualification. Unknown personal facts and acquired basis stay intact.
        for (field, sum, total) in [
            (
                "qualified_items",
                self.properties.iter().map(|p| p.qualified_items).sum::<i64>(),
                self.qualified_items,
            ),
            (
                "sstb_items",
                self.properties.iter().map(|p| p.sstb_items).sum::<i64>(),
                self.sstb_items,
            ),
            (
                "qualified_payments",
                self.properties.iter().map(|p| p.qualified_payments).sum::<i64>(),
                self.qualified_payments,
            ),
        ] {
            if sum > total {
                return Some(ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field(field));
            }
        }
        for k in [
            CoopDistributionBox::Patronage,
            CoopDistributionBox::Nonpatronage,
            CoopDistributionBox::PerUnit,
            CoopDistributionBox::Redeemed,
        ] {
            let property: i64 =
                self.properties.iter().filter(|p| p.distribution == k).map(|p| p.amount).sum();
            let (field, key) = match k {
                CoopDistributionBox::Patronage => ("patronage", "COOP_PERSONAL_ABOVE_BOX1"),
                CoopDistributionBox::Nonpatronage => ("nonpatronage", "COOP_PERSONAL_ABOVE_BOX2"),
                CoopDistributionBox::PerUnit => ("per_unit_retains", "COOP_PERSONAL_ABOVE_BOX3"),
                CoopDistributionBox::Redeemed => ("redeemed", "COOP_PERSONAL_ABOVE_BOX5"),
            };
            let personal = self.personal.part(&k);
            let total = self.distributions.part(&k);
            if personal > total {
                return Some(ValidationIssue::new(key).at_field(field).in_field("personal"));
            }
            if personal + property > total {
                return Some(
                    ValidationIssue::new("COOP_ALLOCATION_INVALID")
                        .at_field(field)
                        .in_field("distributions"),
                );
            }
            let redemptions: i128 = self
                .redemptions
                .iter()
                .filter(|r| r.distribution == k)
                .map(|r| i128::from(r.amount))
                .sum();
            if redemptions > i128::from(self.distributions.part(&k)) {
                return Some(
                    ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("redemptions"),
                );
            }
        }
        if self.redemptions.iter().any(|r| !money(r.amount) || !money(r.stated_value) || !source(&r.source)
            || !matches!(r.distribution,CoopDistributionBox::Redeemed|CoopDistributionBox::Nonpatronage)
            || matches!(&r.holding,CoopNoticeHolding::Acquired{basis,source:s} if !money(*basis)||!source(s))) { return Some(ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("redemptions")); }
        for (index, e) in self.sstb_expenses.iter().enumerate() {
            if !money(e.amount) {
                return Some(
                    ValidationIssue::new("COOP_ALLOCATION_INVALID")
                        .at_field("amount")
                        .in_list("sstb_expenses", index),
                );
            }
            if !source(&e.source) {
                return Some(
                    ValidationIssue::new("COOP_ALLOCATION_INVALID")
                        .at_field("source")
                        .in_list("sstb_expenses", index),
                );
            }
            if self.sstb_expenses[..index].iter().any(|v| v.source == e.source) {
                return Some(
                    ValidationIssue::new("COOP_SSTB_SOURCE_REPEATED")
                        .at_field("source")
                        .in_list("sstb_expenses", index),
                );
            }
        }
        if matches!(&self.income_use,CoopIncomeUse::Other{description} if !source(description)) {
            return Some(ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("redemptions"));
        }
        if matches!(&self.notice,Some(CoopNoticeStatus::Received{source:s}) if !source(s)) {
            return Some(ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("redemptions"));
        }
        if let Some(CoopPatronAllocation::Books {
            expenses_excluding_wages,
            wages,
            other_deductions,
            source: s,
        }) = &self.patron_allocation
            && (![*expenses_excluding_wages, *wages, *other_deductions].into_iter().all(money)
                || !source(s))
        {
            return Some(ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("redemptions"));
        }
        let known_parts = self.personal.total() == 0
            || self.personal_qualification.is_some()
            || [self.qualified_payments, self.qualified_items, self.sstb_items]
                .iter()
                .all(|v| *v == 0);
        let acquired_basis = self
            .redemptions
            .iter()
            .any(|r| matches!(r.holding,CoopNoticeHolding::Acquired{basis,..} if basis>0));
        if self.ordinary() < 0
            || (known_parts && !acquired_basis && self.retained_qualification().is_none())
        {
            return Some(
                ValidationIssue::new("COOP_ALLOCATION_INVALID").at_field("qualified_items"),
            );
        }
        None
    }
}

pub(crate) fn automatic(v: &FederalCooperative) -> CoopAllocation {
    CoopAllocation {
        id: 0,
        distributions: CoopDistributionAmounts {
            patronage: v.patronage,
            nonpatronage: v.nonpatronage,
            per_unit_retains: v.per_unit_retains,
            redeemed: v.redeemed,
        },
        personal: Default::default(),
        personal_qualification: None,
        properties: Vec::new(),
        income_use: CoopIncomeUse::Unresolved,
        qualified_payments: v.qualified_payments,
        qualified_items: v.qualified_items,
        sstb_items: v.sstb_items,
        section_199a_g: v.section_199a_g,
        notice: None,
        patron_allocation: None,
        sstb_expenses: Vec::new(),
        redemptions: Vec::new(),
    }
}

impl CoopCreditFacts {
    #[cfg(test)]
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    pub(crate) fn printed_issue(investment: i64, work_opportunity: i64) -> Option<ValidationIssue> {
        let key = "COOP_CREDIT_SOURCE_INVALID";
        for (field, amount) in [("investment", investment), ("work_opportunity", work_opportunity)]
        {
            if !money(amount) {
                return Some(ValidationIssue::new(key).at_field(field));
            }
        }
        None
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if let Some(issue) = Self::printed_issue(self.investment, self.work_opportunity) {
            return Some(issue);
        }
        let key = "COOP_CREDIT_SOURCE_INVALID";
        if self.investment_shares.len() > 20 {
            return Some(ValidationIssue::new(key).at_field("investment_shares"));
        }
        if self.other.len() > 30 {
            return Some(ValidationIssue::new(key).at_field("other"));
        }
        if self.activity.is_some_and(|id| !(1..=100_000_000_000).contains(&id)) {
            return Some(ValidationIssue::new(key).at_field("activity"));
        }
        for (index, v) in self.investment_shares.iter().enumerate() {
            let issue = if !money(v.amount) {
                Some(ValidationIssue::new(key).at_field("amount"))
            } else if !source(&v.source) {
                Some(ValidationIssue::new(key).at_field("source"))
            } else if matches!(&v.kind,CoopInvestmentKind::Other{description} if !source(description))
            {
                Some(ValidationIssue::new(key).at_field("description").in_field("kind"))
            } else {
                None
            };
            if let Some(issue) = issue {
                return Some(issue.in_list("investment_shares", index));
            }
        }
        if !self.investment_shares.is_empty()
            && self.investment_shares.iter().map(|v| v.amount).sum::<i64>() != self.investment
        {
            return Some(
                ValidationIssue::new(key)
                    .at_field("amount")
                    .in_list("investment_shares", self.investment_shares.len() - 1),
            );
        }
        for (index, v) in self.other.iter().enumerate() {
            let issue = if v.id < 0
                || (v.id > 0 && self.other[..index].iter().any(|old| old.id == v.id))
            {
                Some(ValidationIssue::new(key).at_field("id"))
            } else if !money(v.amount) {
                Some(ValidationIssue::new(key).at_field("amount"))
            } else if !source(&v.source) {
                Some(ValidationIssue::new(key).at_field("source"))
            } else if v.activity.is_some_and(|id| !(1..=100_000_000_000).contains(&id)) {
                Some(ValidationIssue::new(key).at_field("activity"))
            } else if matches!(&v.kind,CoopOtherKind::Other{description} if !source(description)) {
                Some(ValidationIssue::new(key).at_field("description").in_field("kind"))
            } else if let CoopCreditIncome::Inclusion { amount, source: s } = &v.income {
                if !money(*amount) {
                    Some(ValidationIssue::new(key).at_field("amount").in_field("income"))
                } else if !source(s) {
                    Some(ValidationIssue::new(key).at_field("source").in_field("income"))
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(issue) = issue {
                return Some(issue.in_list("other", index));
            }
        }
        if matches!(&self.notice,Some(CoopNoticeStatus::Received{source:s}) if !source(s)) {
            return Some(ValidationIssue::new(key).at_field("source").in_field("notice"));
        }
        None
    }
}
