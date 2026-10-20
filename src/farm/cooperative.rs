// Coverage: Form 1099-PATR retained direct-patron sources (2026).
// Raw payer boxes remain import facts. This module owns retained per-activity
// components, source guards and adapters to existing QBI/3800/income owners.
// Canonical FarmLand purchase refunds within both regular/AMT opening bases and
// current direct in-full sale proceeds use the accepted Farm owner. Taxable basis
// excess, positive property QBI qualifiers, historical/instalment dispositions,
// depreciable assets (Brown) and compound K-1/passive retain named dependencies.
// Farm section175 consumes source production character, not all taxable receipts.
// Tests preserve IRS Reg1.199A-7 examples A-F and derived source/integration probes;
// unmerged owner dependencies are not claimed as full 410778-weight closure.
// Source: IRS Form 1099-PATR Rev. April 2025; T.D. 9947 final Reg. 1.199A-7;
// 2026 draft Form 8995-A and Form 3800; Pub 225(2025).
use crate::BusinessExpenseCategory;
use crate::kernel::phase;
use crate::validation::ValidationIssue;
use alloc::string::String;
use alloc::vec::Vec;
use libertas_macros::*;

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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub patronage: i64,
    /// Portion of box 2
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub nonpatronage: i64,
    /// Portion of box 3
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub per_unit_retains: i64,
    /// Portion of box 5
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Issuer-qualified non-SSTB part included in this property amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// Issuer-qualified SSTB part included in this property amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub sstb_items: i64,
    /// Qualified-payment part included in this property amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Issuer-qualified non-SSTB part included in this property amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// Issuer-qualified SSTB part included in this property amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub sstb_items: i64,
    /// Qualified-payment part included in this property amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Notice original stated dollar amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        expenses_excluding_wages: i64,
        /// Actual W-2 wage expense attributable to qualified payments
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        wages: i64,
        /// Actual source-attributable half SE/health/retirement or other QBI deductions already calculated by their owner
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_payments: i64,
    /// Non-SSTB qualified items
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// SSTB qualified items
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    /// Same-owner accepted C/F/4835 activity
    #[libertas_number(min = 1)]
    pub activity: Option<i64>,
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_payments: i64,
    /// Box 8 non-SSTB items allocated by issuer/books
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// Box 9 SSTB items allocated by issuer/books
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub sstb_items: i64,
    /// Box 6 deduction allocated by issuer
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    /// Same-owner accepted C/F/4835 activity
    /// [NullText]
    /// The receiving activity has not been linked
    /// [SetText]
    /// The receiving activity is
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.cooperative_activities")]
    pub activity: Option<i64>,
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_payments: i64,
    /// Box 8 non-SSTB items allocated by issuer/books
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub qualified_items: i64,
    /// Box 9 SSTB items allocated by issuer/books
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub sstb_items: i64,
    /// Box 6 deduction allocated by issuer
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
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
    /// Printed credit or deduction kind
    pub kind: CoopOtherKind,
    /// Printed allocation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    /// Printed credit or deduction kind
    pub kind: CoopOtherKind,
    /// Printed allocation
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub amount: i64,
    /// Accepted activity generating this source
    /// [NullText]
    /// The originating activity has not been linked
    /// [SetText]
    /// The originating activity is
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.cooperative_activities")]
    pub activity: Option<i64>,
    /// Associated mandatory income status
    #[libertas_default(Unresolved)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub investment: i64,
    /// Printed box 11
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub investment: i64,
    /// Printed box 11
    #[libertas_llm_doc_field(blank = 0)]
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub work_opportunity: i64,
    /// Issuer breakdown of box 10
    pub investment_shares: Vec<CoopInvestmentShare>,
    /// Separate printed box 12 items
    pub other: Vec<CoopOtherItemInput>,
    /// Activity credit participation under existing passive owner
    /// [NullText]
    /// Credit participation has not been established
    /// [SetText]
    /// My activity credit participation is
    // Unprovided source-activity participation records cannot be reconstructed from box amounts.
    // With the credit source otherwise established, null retains WARN_COOP_CREDIT_PASSIVE
    // and passive-credit treatment.
    pub participation: Option<bool>,
    /// Actual activity receiving box 10 and 11 credits
    /// [NullText]
    /// I am not claiming the box 10 or 11 credits
    /// [SetText]
    /// The credits belong to
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.cooperative_activities")]
    #[libertas_default($null)]
    pub activity: Option<i64>,
    /// Credit allocation notice
    /// [NullText]
    /// The credit allocation notice facts are not established
    /// [SetText]
    /// The credit notice status is
    #[libertas_default($null)]
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
        activity: v.activity,
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
            || self.activity.is_some_and(|id| !(1..=100_000_000_000).contains(&id))
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

pub(crate) fn eligible_activities(d: &crate::FederalDraft, owner: crate::Owner) -> Vec<i64> {
    d.income
        .iter()
        .filter(|e| e.owner == owner)
        .filter_map(|e| match &e.income {
            crate::FederalIncome::Business { data }
                if data.source == crate::BusinessSource::UnitedStates
                    && matches!(
                        data.activity,
                        crate::BusinessActivity::Ordinary { .. }
                            | crate::BusinessActivity::Farming { .. }
                    ) =>
            {
                Some(e.id)
            }
            crate::FederalIncome::FarmRental { .. } => Some(e.id),
            _ => None,
        })
        .collect()
}

pub(crate) fn automatic(v: &crate::FederalCooperative, activity: Option<i64>) -> CoopAllocation {
    CoopAllocation {
        id: 0,
        activity,
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

// Retained parent and child identities are never rewritten by this projection.
#[derive(Clone)]
pub(crate) struct CoopSource {
    pub parent: i64,
    pub owner: crate::Owner,
    pub allocation: CoopAllocation,
    // Transient position in the retained source allocation vector. None only for
    // sources() synthesizing a projection from an empty vector; Some is never
    // proof that the original submitted Page contained that row.
    pub allocation_index: Option<usize>,
}
pub(crate) fn sources(d: &crate::FederalDraft) -> Vec<CoopSource> {
    d.income
        .iter()
        .flat_map(|e| {
            let crate::FederalIncome::Cooperative { data } = &e.income else {
                return Vec::new();
            };
            let mut allocations = if data.allocations.is_empty() {
                let activities = eligible_activities(d, e.owner);
                alloc::vec![automatic(data, (activities.len() == 1).then(|| activities[0]))]
            } else {
                data.allocations.clone()
            };
            let eligible = eligible_activities(d, e.owner);
            if eligible.len() == 1 {
                for a in &mut allocations {
                    if a.activity.is_none() {
                        a.activity = Some(eligible[0]);
                    }
                }
            }
            let explicit = !data.allocations.is_empty();
            allocations
                .into_iter()
                .enumerate()
                .map(|(index, allocation)| CoopSource {
                    parent: e.id,
                    owner: e.owner,
                    allocation,
                    allocation_index: explicit.then_some(index),
                })
                .collect()
        })
        .collect()
}

// Canonical land and equipment use their accepted property owners. No new
// identity or basis pool is stored; historical-sale treatment stays explicit.
pub(crate) fn land_refund_supported(
    d: &crate::FederalDraft,
    s: &CoopSource,
    p: &CoopPropertyRefund,
) -> bool {
    if p.amount == 0 {
        return true;
    }
    let Some(property) = p.property else {
        return false;
    };
    if let Some(asset) =
        crate::cooperative_basis::asset(d, s.owner, s.allocation.activity, property)
    {
        if p.distribution != CoopDistributionBox::Patronage
            || p.treatment != CoopPropertyTreatment::Purchase
        {
            return false;
        }
        let Some(activity) = s.allocation.activity else {
            return false;
        };
        return [false, true].into_iter().all(|alternative| {
            match crate::cooperative_basis::source_amounts(
                d,
                activity,
                asset,
                crate::depreciation::mid_quarter(d),
                alternative,
            ) {
                Ok(_) | Err(crate::Section179PreparedIssue::AmountOverflow(_)) => true,
                Err(_) => false,
            }
        });
    }
    let Some(farm) = crate::farm::sources(d).find(|f| {
        f.entry.owner == s.owner
            && Some(f.entry.id) == s.allocation.activity
            && f.farm.land_basis.iter().any(|land| land.id == property)
    }) else {
        return false;
    };
    let land = farm.farm.land_basis.iter().find(|land| land.id == property).unwrap();
    if p.distribution != CoopDistributionBox::Patronage {
        return false;
    }
    match p.treatment {
        CoopPropertyTreatment::Purchase => {
            let total = land_adjustments(d, farm.entry.id, property).0;
            // Different regular/AMT taxable excess needs an income-owner seam;
            // do not suppress that required tax by merely clamping basis.
            total <= phase::cap(land.regular, [land.amt])
        }
        CoopPropertyTreatment::CurrentSale => {
            p.qualified_items + p.sstb_items + p.qualified_payments == 0
                && farm.farm.land_sales.iter().any(|sale| {
                    sale.basis_property == property
                        && sale.situation == crate::FarmLandSituation::Direct
                        && matches!(sale.payment, crate::SalePayment::InFull)
                })
        }
        CoopPropertyTreatment::EarlierDisposition { .. } => false,
    }
}

pub(crate) fn land_adjustments(
    d: &crate::FederalDraft,
    activity: i64,
    property: i64,
) -> (i64, i64) {
    sources(d)
        .iter()
        .filter(|s| s.allocation.activity == Some(activity))
        .flat_map(|s| &s.allocation.properties)
        .filter(|p| {
            p.property == Some(property) && p.distribution == CoopDistributionBox::Patronage
        })
        .fold((0, 0), |(basis, proceeds), p| match p.treatment {
            CoopPropertyTreatment::Purchase => (basis + p.amount, proceeds),
            CoopPropertyTreatment::CurrentSale => (basis, proceeds + p.amount),
            CoopPropertyTreatment::EarlierDisposition { .. } => (basis, proceeds),
        })
}

pub(crate) fn input_issue(v: &crate::Patr1099) -> Option<ValidationIssue> {
    let a: Vec<CoopAllocation> = v.more.allocations.iter().cloned().map(Into::into).collect();
    for (index, allocation) in a.iter().enumerate() {
        if let Some(issue) = allocation.issue() {
            return Some(issue.in_list("allocations", index).in_field("more"));
        }
    }
    if !a.is_empty() {
        let mut sums = [0; 8];
        for a in &a {
            for (to, n) in sums[..4].iter_mut().zip(a.distributions.values()) {
                *to += n;
            }
            for (to, n) in sums[4..].iter_mut().zip([
                a.qualified_payments,
                a.qualified_items,
                a.sstb_items,
                a.section_199a_g,
            ]) {
                *to += n;
            }
        }
        if sums
            != [
                v.patronage,
                v.nonpatronage,
                v.per_unit_retains,
                v.redeemed,
                v.more.qualified_payments,
                v.more.qualified_items,
                v.more.sstb_items,
                v.more.section_199a_g,
            ]
        {
            return Some(
                ValidationIssue::new("COOP_ALLOCATIONS_DO_NOT_RECONCILE")
                    .at_field("allocations")
                    .in_field("more"),
            );
        }
    }
    if let Some(issue) = CoopCreditFacts::from(v.more.credit_facts.clone()).issue() {
        return Some(issue.in_field("credit_facts").in_field("more"));
    }
    None
}

impl CoopCreditFacts {
    #[cfg(test)]
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        let key = "COOP_CREDIT_SOURCE_INVALID";
        for (field, amount) in
            [("investment", self.investment), ("work_opportunity", self.work_opportunity)]
        {
            if !money(amount) {
                return Some(ValidationIssue::new(key).at_field(field));
            }
        }
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
            let issue = if !money(v.amount) {
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

// These are the receipt source owner's existing report requirements. The
// prepared before179 view uses the same facts without collecting tax again.
fn receipt_requirements(
    d: &crate::FederalDraft,
    source: &CoopSource,
    eligible: &[i64],
) -> [(bool, &'static str, &'static str, &'static str); 4] {
    let a = &source.allocation;
    // Box 5 is gross redemption, not the holder's basis or notice character.
    let redeemed: i64 = a
        .redemptions
        .iter()
        .filter(|v| v.distribution == CoopDistributionBox::Redeemed)
        .map(|v| v.amount)
        .sum();
    let personal_redemptions: i64 = a
        .redemptions
        .iter()
        .filter(|v| {
            v.kind == CoopNoticeKind::Patronage && v.distribution == CoopDistributionBox::Redeemed
        })
        .map(|v| v.amount)
        .sum();
    [
        (a.ordinary() == 0 || a.activity.is_some_and(|id| eligible.contains(&id)),
         "ISSUE_COOP_ACTIVITY_NEEDED", "Cooperative recipient",
         "The known cooperative income needs its actual same-owner Schedule C, farm or farm-rental recipient; no activity or income is guessed."),
        (a.properties.iter().all(|p| land_refund_supported(d, source, p)),
         "ISSUE_COOP_PROPERTY_HISTORY_NEEDED", "Cooperative property refund",
         "The capital-purchase refund needs the canonical property owner's effective-dated regular/AMT basis or disposition result. The refund is not simply taxed as ordinary income."),
        (redeemed == a.distributions.redeemed && a.redemptions.iter().all(|v|
            v.kind != CoopNoticeKind::Unresolved && !matches!(v.holding,CoopNoticeHolding::Unresolved)
            && v.amount <= v.stated_value
            && !matches!(v.holding,CoopNoticeHolding::Acquired{basis,..} if v.amount < basis || a.personal.total() > 0 || !a.properties.is_empty())),
         "ISSUE_COOP_NOTICE_HISTORY_NEEDED", "Cooperative notice history",
         "Record notice history covering the entire box 5 redeemed amount, including actual ownership, basis and stated value. Above-stated-value proceeds, losses or basis shared with an excluded part need their source allocation; no basis or gain character is guessed."),
        (a.personal.nonpatronage == 0 && a.personal.per_unit_retains == 0 && a.personal.redeemed <= personal_redemptions,
         "ISSUE_COOP_DISTRIBUTION_CHARACTER_NEEDED", "Cooperative distribution character",
         "Purchase exclusions require the original patronage-dividend or patronage-notice source. Nonpatronage distributions and per-unit retains are not assumed to be personal-purchase refunds."),
    ]
}

fn credit_income_known(item: &CoopOtherItem, eligible: &[i64]) -> bool {
    !matches!(item.income, CoopCreditIncome::Unresolved)
        && (!matches!(item.income,CoopCreditIncome::Inclusion{amount,..} if amount > 0)
            || item.activity.is_some_and(|id| eligible.contains(&id)))
}

// The supplied working view already owns support/routing and accepted global
// graph validation. Certify only present contributors which can reach C/F;
// a known unrelated farm rental does not invalidate their receipt sources.
pub(crate) fn before179_receipt_source_issue(
    d: &crate::FederalDraft,
    activities: &[(i64, crate::Owner)],
) -> Option<(i64, &'static str)> {
    let sources = sources(d);
    for entry in &d.income {
        let crate::FederalIncome::Cooperative { data } = &entry.income else {
            continue;
        };
        if !activities.iter().any(|(_, owner)| *owner == entry.owner) {
            continue;
        }
        let eligible = eligible_activities(d, entry.owner);
        let applicable = |activity: Option<i64>| match activity {
            Some(id) if eligible.contains(&id) => activities.contains(&(id, entry.owner)),
            // A missing or invalid relationship cannot establish exclusion from
            // this owner's actual C/F activities.
            _ => true,
        };
        let mut relevant = false;
        for source in sources.iter().filter(|source| source.parent == entry.id) {
            if !applicable(source.allocation.activity) {
                continue;
            }
            relevant = true;
            if let Some((_, key, _, _)) = receipt_requirements(d, source, &eligible)
                .into_iter()
                .find(|(known, _, _, _)| !known)
            {
                return Some((entry.id, key));
            }
        }
        let income = data.credit_facts.other.iter().filter(|item| applicable(item.activity));
        let mut credit_issue = false;
        for item in income {
            relevant = true;
            credit_issue |= !credit_income_known(item, &eligible);
        }
        if relevant && !data.valid() {
            return Some((entry.id, "ISSUE_COOP_SOURCE_INVALID"));
        }
        if credit_issue {
            return Some((entry.id, "ISSUE_COOP_CREDIT_INCOME_NEEDED"));
        }
    }
    None
}

pub(crate) fn report_sources(
    d: &crate::FederalDraft,
    e: &crate::FederalIncomeEntry,
    r: &mut crate::FederalResult,
) {
    let crate::FederalIncome::Cooperative { data } = &e.income else {
        return;
    };
    let context = crate::interview::income_issue_context(e);
    let eligible = eligible_activities(d, e.owner);
    for v in sources(d).into_iter().filter(|v| v.parent == e.id) {
        for (available, key, _section, _english) in receipt_requirements(d, &v, &eligible) {
            crate::require_localized!(r, available, &context, _section, key, _english,);
        }
    }
    crate::require_localized!(
        r,
        data.valid() && graph_rule(d).is_none(),
        &context,
        "Cooperative source facts",
        "ISSUE_COOP_SOURCE_INVALID",
        "Correct the cooperative source amounts and component records before estimating.",
    );
}

// Validate accepted graph links without manufacturing a recipient or property ID.
pub(crate) fn graph_rule(d: &crate::FederalDraft) -> Option<&'static str> {
    graph_issue(d).map(|(_, issue)| issue.key)
}

// The ID is the actual accepted/candidate income record, never a request position.
// Local paths address FederalCooperative. The Save owner maps them to Patr1099.more
// and correlates record IDs with submitted documents. A prepared allocation can
// be absent in the original Page: the owner must then target its real collection.
pub(crate) fn graph_issue(d: &crate::FederalDraft) -> Option<(i64, ValidationIssue)> {
    let mut seen = alloc::collections::BTreeSet::new();
    let sources = sources(d);
    for e in &d.income {
        let crate::FederalIncome::Cooperative { data } = &e.income else {
            continue;
        };
        if let Some(mut issue) = data.issue() {
            // Preserve the graph gate's historical diagnostic context.
            issue.key = "COOP_ALLOCATION_INVALID";
            return Some((e.id, issue));
        }
        let eligible = eligible_activities(d, e.owner);
        for id in &eligible {
            let governing = sources
                .iter()
                .filter(|s| s.allocation.activity == Some(*id))
                .filter(|s| s.allocation.patron_allocation.is_some())
                .count();
            if governing > 1 {
                // The original global condition can fail while examining an
                // unrelated cooperative. Attribute it to a real second source.
                let offending = sources
                    .iter()
                    .filter(|s| s.allocation.activity == Some(*id))
                    .filter(|s| s.allocation.patron_allocation.is_some())
                    .nth(1)
                    .expect("multiple governing cooperative sources");
                let issue = ValidationIssue::new("COOP_PATRON_SOURCE_DUPLICATE");
                let issue = match offending.allocation_index {
                    Some(index) => {
                        issue.at_field("patron_allocation").in_list("allocations", index)
                    }
                    None => issue.at_field("allocations"),
                };
                return Some((offending.parent, issue));
            }
        }
        let mut sums = [0; 8];
        for (index, a) in data.allocations.iter().enumerate() {
            if a.id <= 0
                || !seen.insert(a.id)
                || d.income
                    .iter()
                    .any(|e| e.id == a.id || crate::farm::ccc_lot_ids(&e.income).contains(&a.id))
            {
                return Some((
                    e.id,
                    ValidationIssue::new("COOP_ALLOCATION_ID_INVALID")
                        .at_field("id")
                        .in_list("allocations", index),
                ));
            }
            if a.activity.is_some_and(|id| !eligible.contains(&id)) {
                return Some((
                    e.id,
                    ValidationIssue::new("COOP_RECIPIENT_LINK_INVALID")
                        .at_field("activity")
                        .in_list("allocations", index),
                ));
            }
            if let Some(property_index) = a.properties.iter().position(|p| {
                p.property.is_some_and(|property| {
                    crate::farm::sources(d)
                        .filter(|f| f.entry.owner == e.owner && Some(f.entry.id) == a.activity)
                        .map(|f| {
                            f.farm.land_basis.iter().filter(|land| land.id == property).count()
                        })
                        .sum::<usize>()
                        + usize::from(a.activity.is_some_and(|activity| {
                            crate::source_associations::SourceAssociations::new(d)
                                .asset(activity, property)
                                .is_ok()
                        }))
                        != 1
                })
            }) {
                return Some((
                    e.id,
                    ValidationIssue::new("COOP_PROPERTY_LINK_UNAVAILABLE")
                        .at_field("property")
                        .in_list("properties", property_index)
                        .in_list("allocations", index),
                ));
            }
            for (to, n) in sums[..4].iter_mut().zip(a.distributions.values()) {
                *to += n;
            }
            for (to, n) in sums[4..].iter_mut().zip([
                a.qualified_payments,
                a.qualified_items,
                a.sstb_items,
                a.section_199a_g,
            ]) {
                *to += n;
            }
        }
        if !data.allocations.is_empty()
            && sums
                != [
                    data.patronage,
                    data.nonpatronage,
                    data.per_unit_retains,
                    data.redeemed,
                    data.qualified_payments,
                    data.qualified_items,
                    data.sstb_items,
                    data.section_199a_g,
                ]
        {
            return Some((
                e.id,
                ValidationIssue::new("COOP_ALLOCATIONS_DO_NOT_RECONCILE").at_field("allocations"),
            ));
        }
        for (index, a) in data.allocations.iter().enumerate() {
            let Some(id) = a.activity else {
                continue;
            };
            let Some(business) = d.income.iter().find_map(|e| match &e.income {
                crate::FederalIncome::Business { data } if e.id == id => Some(data),
                _ => None,
            }) else {
                if !a.sstb_expenses.is_empty() {
                    return Some((
                        e.id,
                        ValidationIssue::new("COOP_SSTB_EXPENSE_INVALID")
                            .at_field("sstb_expenses")
                            .in_list("allocations", index),
                    ));
                }
                continue;
            };
            if matches!(a.patron_allocation, Some(CoopPatronAllocation::NoWages))
                && business.payroll.qbi_wages() != 0
            {
                return Some((
                    e.id,
                    ValidationIssue::new("COOP_PATRON_WAGES_NOT_ZERO")
                        .at_field("patron_allocation")
                        .in_list("allocations", index),
                ));
            }
            for (expense_index, expense) in a.sstb_expenses.iter().enumerate() {
                if sources
                    .iter()
                    .filter(|s| s.allocation.activity == Some(id))
                    .flat_map(|s| &s.allocation.sstb_expenses)
                    .filter(|x| x.source == expense.source)
                    .count()
                    > 1
                {
                    return Some((
                        e.id,
                        ValidationIssue::new("COOP_SSTB_SOURCE_REPEATED")
                            .at_field("source")
                            .in_list("sstb_expenses", expense_index)
                            .in_list("allocations", index),
                    ));
                }
                let total: i64 = sources
                    .iter()
                    .filter(|s| s.allocation.activity == Some(id))
                    .flat_map(|s| &s.allocation.sstb_expenses)
                    .filter(|x| x.category == expense.category)
                    .map(|x| x.amount)
                    .sum();
                let entries: Vec<_> = business
                    .expenses
                    .iter()
                    .filter(|x| x.category == expense.category)
                    .cloned()
                    .collect();
                if total > crate::FederalBusinessExpense::total(&entries) {
                    return Some((
                        e.id,
                        ValidationIssue::new("COOP_SSTB_EXPENSE_INVALID")
                            .at_field("amount")
                            .in_list("sstb_expenses", expense_index)
                            .in_list("allocations", index),
                    ));
                }
            }
        }
        if data.credit_facts.activity.is_some_and(|id| !eligible.contains(&id)) {
            return Some((
                e.id,
                ValidationIssue::new("COOP_RECIPIENT_LINK_INVALID")
                    .at_field("activity")
                    .in_field("credit_facts"),
            ));
        }
        if let Some(index) = data
            .credit_facts
            .other
            .iter()
            .position(|v| v.activity.is_some_and(|id| !eligible.contains(&id)))
        {
            return Some((
                e.id,
                ValidationIssue::new("COOP_RECIPIENT_LINK_INVALID")
                    .at_field("activity")
                    .in_list("other", index)
                    .in_field("credit_facts"),
            ));
        }
    }
    None
}

pub(crate) type QbiFiling = (Vec<[i64; 2]>, i64, [i64; 2], Vec<i64>);

#[cfg(test)]
pub(crate) fn qbi_filing(
    d: &crate::FederalDraft,
    activities: &mut [crate::business_rules::QbiActivity],
    business_ids: &alloc::collections::BTreeMap<usize, i64>,
    taxable: i64,
    status: crate::FederalStatus,
    _prior_loss: i64,
    r: &mut crate::FederalResult,
) -> QbiFiling {
    match qbi_filing_with_section179(d, activities, business_ids, taxable, status, None, r, None) {
        Ok(value) => value,
        Err(_) => unreachable!("legacy section179 path is infallible"),
    }
}

// Forward the existing invocation operands and its borrowed home context together.
#[allow(clippy::too_many_arguments)]
pub(crate) fn qbi_filing_with_section179(
    d: &crate::FederalDraft,
    activities: &mut [crate::business_rules::QbiActivity],
    business_ids: &alloc::collections::BTreeMap<usize, i64>,
    taxable: i64,
    status: crate::FederalStatus,
    selected: Option<&crate::Section179PreparedAllocation>,
    r: &mut crate::FederalResult,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<QbiFiling, crate::Section179PreparedIssue> {
    use crate::kernel::ratio::{dollars, fraction, positive};
    let sources = sources(d);
    let mut reductions = alloc::vec![[0;2]; activities.len()];
    let mut dpad = 0;
    let mut owner_dpad = [0; 2];
    let mut dpad_weights = alloc::vec![0;activities.len()];
    for (index, id) in business_ids {
        let Some(entry) = d.income.iter().find(|e| e.id == *id) else {
            continue;
        };
        let crate::FederalIncome::Business { data: business } = &entry.income else {
            continue;
        };
        let shares: Vec<_> =
            sources.iter().filter(|s| s.allocation.activity == Some(*id)).collect();
        if shares.is_empty() {
            continue;
        }
        let receipts = receipt_totals(d, entry.owner, *id, false)?;
        let original = activities[*index].clone();
        let share = d.setup.as_ref().and_then(|setup| {
            crate::community::share(setup, entry.owner, entry.community, &entry.income)
        });
        let divide = |amount: i64| share.map_or(amount, |s| s.filer(amount));
        let mut eligible_gross = receipts.qualification[0];
        let mut sstb = receipts.qualification[2];
        let mut related_expenses = receipts.related_sstb_expenses;
        let mut books = [0; 3];
        let mut books_complete = false;
        let mut safe_harbor = false;
        let mut qualification_complete = true;
        for share in &shares {
            let a = &share.allocation;
            let context = crate::IssueContext::income_id(share.parent);
            if let Some(q) = a.retained_qualification() {
                related_expenses +=
                    phase::cap(q[2], [a.sstb_expenses.iter().map(|e| e.amount).sum::<i64>()]);
            } else {
                qualification_complete = false;
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new("WARN_COOP_QUALIFICATION_LEFT_OUT", alloc::vec![]),
                    context.clone(),
                );
            }
            if a.section_199a_g > 0 {
                if matches!(&a.notice, Some(CoopNoticeStatus::Received { .. })) {
                    let amount = divide(a.section_199a_g);
                    dpad += amount;
                    owner_dpad[usize::from(share.owner == crate::Owner::Spouse)] += amount;
                    dpad_weights[*index] += amount;
                } else {
                    crate::federal_rules::note_once_scoped(
                        r,
                        crate::Message::new("WARN_COOP_DPAD_NOTICE_NEEDED", alloc::vec![]),
                        context.clone(),
                    );
                }
            }
        }
        let governing: Vec<_> =
            shares.iter().filter_map(|s| s.allocation.patron_allocation.as_ref()).collect();
        if governing.len() == 1 {
            match governing[0] {
                CoopPatronAllocation::Books {
                    expenses_excluding_wages,
                    wages,
                    other_deductions,
                    ..
                } => {
                    books = [*expenses_excluding_wages, *wages, *other_deductions];
                    books_complete = true;
                }
                CoopPatronAllocation::SafeHarbor => safe_harbor = true,
                _ => {}
            }
        }
        eligible_gross = divide(eligible_gross);
        sstb = divide(sstb);
        related_expenses = phase::cap(divide(related_expenses), [sstb]);
        books = books.map(divide);
        // The cooperative SSTB item has its own eligibility classification;
        // patron wages/UBIA remain the actual directly conducted trade's.
        // Related paid expenses move classes without a second tax deduction.
        if sstb > 0 {
            let service = crate::business_rules::QbiActivity {
                source_wages_cents: None,
                source_property_cents: None,
                source_income_cents: None,
                owner: None,
                income: dollars(sstb - related_expenses),
                wages: 0,
                property: 0,
                active: original.active,
                service: Some(true),
                group: None,
            };
            let eligible = if original.service != Some(false) {
                service.income
            } else {
                crate::business_rules::eligible_qbi_activity(&service, taxable, status)
                    .map_or(0, |a| a.income)
            };
            activities[*index].income += dollars(related_expenses) + eligible;
        }
        let wage_cents = divide(business.payroll.qbi_wages());
        if eligible_gross == 0 || wage_cents == 0 {
            continue;
        }
        let activity_cents = positive(activities[*index].income * 100);
        let gross =
            divide(business.receipts() + receipts.qualification[1] + receipts.qualification[2]);
        let expenses =
            divide(crate::section179_return::expenses(d, *id, business, selected, home)?);
        // Reconcile the actual below-line deduction pool before adding or
        // phasing the cooperative SSTB class. Excluded SSTB income and an
        // overall loss are not extra deduction-allocation capacity.
        let ordinary_gross = divide(business.receipts() + receipts.qualification[1]);
        let qbi_other = positive(ordinary_gross - expenses - original.income * 100);
        let source_values = if books_complete
            && books[1] <= wage_cents
            && books[0] <= positive(expenses - wage_cents)
            && books[2] <= qbi_other
        {
            Some((positive(eligible_gross - books.iter().sum::<i64>()), books[1]))
        } else if safe_harbor
            && crate::business_rules::qbi_phase(taxable, status).0 == 0
            && gross > 0
            && eligible_gross <= gross
        {
            Some((
                fraction(activity_cents, eligible_gross, gross),
                fraction(wage_cents, eligible_gross, gross),
            ))
        } else {
            None
        };
        if let Some((income, wages)) = source_values.filter(|_| qualification_complete) {
            let qualified = crate::business_rules::QbiActivity {
                source_wages_cents: None,
                source_property_cents: None,
                source_income_cents: None,
                owner: None,
                income: dollars(income),
                wages: dollars(wages),
                property: 0,
                active: original.active,
                service: original.service,
                group: None,
            };
            if let Ok(value) =
                crate::business_rules::eligible_qbi_activity(&qualified, taxable, status)
            {
                reductions[*index] = [value.income, value.wages];
            }
        } else {
            // Actual allocation evidence is missing; known taxable income
            // and qualified loss history remain intact.
            if activities[*index].income > 0 {
                activities[*index].income = phase::cap(activities[*index].income, [0]);
            }
            activities[*index].wages = 0;
            activities[*index].property = 0;
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("WARN_COOP_PATRON_ALLOCATION_NEEDED", alloc::vec![]),
                crate::IssueContext::income_id(*id),
            );
        }
    }
    // Form 4835 and pass-through QBI need their actual accepted statement
    // interface; source income is still reported by its existing owner.
    for share in sources
        .iter()
        .filter(|s| !business_ids.values().any(|id| Some(*id) == s.allocation.activity))
    {
        if share.allocation.qualified_items
            + share.allocation.sstb_items
            + share.allocation.section_199a_g
            > 0
        {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("WARN_COOP_QBI_RECIPIENT_NEEDED", alloc::vec![]),
                crate::IssueContext::income_id(share.parent),
            );
        }
    }
    Ok((
        reductions,
        dollars(dpad),
        owner_dpad,
        crate::kernel::ratio::allocate_remaining(dollars(dpad), &dpad_weights),
    ))
}

// Section 175 gross is production income, not every taxable Schedule F receipt.
// Keep this economic classification at the cooperative source owner; Farm owns
// the combined limit, deductions and carry records.
pub(crate) fn conservation_gross(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    activity: i64,
) -> Option<i64> {
    // Required credit income is retained in ordinary profit, but this source
    // shape does not establish that it arose from agricultural production.
    if additional_income(d, owner, activity) != 0 {
        return None;
    }
    sources(d)
        .into_iter()
        .filter(|s| s.owner == owner && s.allocation.activity == Some(activity))
        .try_fold(0, |gross, s| {
            let ordinary = s.allocation.ordinary();
            if ordinary == 0 {
                return Some(gross);
            }
            match s.allocation.income_use {
                CoopIncomeUse::ProductionSales => Some(gross + ordinary),
                CoopIncomeUse::DeductiblePurchases => Some(gross),
                CoopIncomeUse::Other { .. } | CoopIncomeUse::Unresolved => None,
            }
        })
}

// Native-cent receipt output for C/F/4835. Legally equivalent amounts
// combine inside their source owner before its once-cent output projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReceiptTotals {
    pub distributions: i64,
    pub taxable: i64,
    pub nonqualified: i64,
    pub qualification: [i64; 3],
    pub related_sstb_expenses: i64,
}
pub(crate) fn receipt_totals(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    activity: i64,
    alternative: bool,
) -> Result<ReceiptTotals, crate::Section179PreparedIssue> {
    use crate::kernel::{ratio::exact, source_fact::SourceFact};
    let additional = additional_income(d, owner, activity);
    let mut distributions = 0_i64;
    let mut related_sstb_expenses = 0;
    let mut taxable = exact::whole(additional);
    let mut nonqualified = exact::whole(additional);
    let mut qualification = [exact::whole(0), exact::whole(0), exact::whole(0)];
    for source in sources(d)
        .into_iter()
        .filter(|source| source.owner == owner && source.allocation.activity == Some(activity))
    {
        let ordinary = source.allocation.ordinary();
        let qualified = source.allocation.retained_qualification().unwrap_or([0; 3]);
        distributions = distributions
            .checked_add(source.allocation.distributions.total())
            .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?;
        taxable += exact::whole(ordinary);
        nonqualified += exact::whole(phase::floor(ordinary - qualified[1], 0));
        for (target, amount) in qualification.iter_mut().zip(qualified) {
            *target += exact::whole(amount);
        }
    }
    match crate::cooperative_basis::income(d, owner, activity, alternative)? {
        SourceFact::NotReported => (),
        SourceFact::Unavailable => {
            return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                activity,
                "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
            ));
        }
        SourceFact::Reported(excess) => {
            related_sstb_expenses = excess.related_sstb_expenses;
            nonqualified += exact::whole(excess.ordinary - excess.qualification[1]);
            taxable += exact::whole(excess.ordinary);
            for (target, amount) in qualification.iter_mut().zip(excess.qualification) {
                *target += exact::whole(amount);
            }
        }
    }
    let overflow = || crate::Section179PreparedIssue::AmountOverflow(activity);
    let [Some(payments), Some(non_sstb), Some(sstb)] =
        qualification.map(|amount| exact::rounded(&amount))
    else {
        return Err(overflow());
    };
    Ok(ReceiptTotals {
        distributions,
        taxable: exact::rounded(&taxable).ok_or_else(overflow)?,
        nonqualified: exact::rounded(&nonqualified).ok_or_else(overflow)?,
        qualification: [payments, non_sstb, sstb],
        related_sstb_expenses,
    })
}

pub(crate) fn additional_income(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    activity: i64,
) -> i64 {
    d.income
        .iter()
        .filter(|e| e.owner == owner)
        .filter_map(|e| match &e.income {
            crate::FederalIncome::Cooperative { data } => Some(&data.credit_facts.other),
            _ => None,
        })
        .flatten()
        .filter(|v| v.activity == Some(activity))
        .map(|v| match v.income {
            CoopCreditIncome::Inclusion { amount, .. } => amount,
            _ => 0,
        })
        .sum()
}

pub(crate) fn credit_entries(
    d: &crate::FederalDraft,
    r: &mut crate::FederalResult,
) -> Vec<crate::general_business_credit::CreditEntry> {
    use crate::{BusinessCredit as C, general_business_credit::CreditEntry};
    let mut result = Vec::new();
    for e in &d.income {
        let crate::FederalIncome::Cooperative { data } = &e.income else {
            continue;
        };
        let v = &data.credit_facts;
        let eligible = eligible_activities(d, e.owner);
        let context = crate::IssueContext::income_id(e.id);
        let notice = matches!(&v.notice, Some(CoopNoticeStatus::Received { .. }));
        let mut add =
            |credit: C, amount: i64, activity: Option<i64>, r: &mut crate::FederalResult| {
                if amount == 0 {
                    return;
                }
                if !notice || !activity.is_some_and(|id| eligible.contains(&id)) {
                    crate::federal_rules::note_once_scoped(
                        r,
                        crate::Message::new("WARN_COOP_CREDIT_SOURCE_NEEDED", alloc::vec![]),
                        context.clone(),
                    );
                    return;
                }
                if v.participation.is_none() {
                    crate::federal_rules::note_once_scoped(
                        r,
                        crate::Message::new("WARN_COOP_CREDIT_PASSIVE", alloc::vec![]),
                        context.clone(),
                    );
                }
                result.push(CreditEntry {
                    source_owner: Some(e.owner),
                    conditional_origin: None,
                    transfer: 0,
                    elective: 0,
                    activity: None,
                    filer: crate::CarryFiler::SameFilers,
                    credit,
                    year: crate::year_constants::TAX_YEAR,
                    amount: crate::kernel::ratio::dollars(amount),
                    passive: (v.participation != Some(true)).then_some(credit.worksheet()),
                });
            };
        if v.investment > 0 && v.investment_shares.is_empty() {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("WARN_COOP_CREDIT_SOURCE_NEEDED", alloc::vec![]),
                context.clone(),
            );
        }
        for share in &v.investment_shares {
            let credit = match share.kind {
                CoopInvestmentKind::Rehabilitation => Some(C::Rehabilitation),
                CoopInvestmentKind::Energy => Some(C::Energy),
                CoopInvestmentKind::CleanElectricity => Some(C::CleanElectricity),
                _ => None,
            };
            if let Some(credit) = credit {
                add(credit, share.amount, v.activity, r);
            } else if share.amount > 0 {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_COOP_OTHER_BENEFIT_LEFT_OUT",
                        alloc::vec![
                            crate::MessageArg::Text(match &share.kind {
                                CoopInvestmentKind::Other { description } => description.clone(),
                                _ => share.source.clone(),
                            }),
                            crate::MessageArg::Text(share.source.clone()),
                        ],
                    ),
                    context.clone(),
                );
            }
        }
        add(C::WorkOpportunity, v.work_opportunity, v.activity, r);
        for item in &v.other {
            let credit = match item.kind {
                CoopOtherKind::CleanFuel => Some(C::CleanFuel),
                CoopOtherKind::SmallEmployerHealth => Some(C::SmallEmployerHealth),
                _ => None,
            };
            if let Some(credit) = credit {
                add(credit, item.amount, item.activity, r);
            } else if item.amount > 0 {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_COOP_OTHER_BENEFIT_LEFT_OUT",
                        alloc::vec![
                            match &item.kind {
                                CoopOtherKind::EmpowermentZone =>
                                    crate::MessageArg::Resource("COOP_EMPOWERMENT_ZONE".into()),
                                CoopOtherKind::LowSulfurDiesel =>
                                    crate::MessageArg::Resource("COOP_LOW_SULFUR_DIESEL".into()),
                                CoopOtherKind::DifferentialWage =>
                                    crate::MessageArg::Resource("COOP_DIFFERENTIAL_WAGE".into()),
                                CoopOtherKind::SmallAgriBiodiesel =>
                                    crate::MessageArg::Resource("COOP_SMALL_AGRI_BIODIESEL".into()),
                                CoopOtherKind::SmallRefinerDeduction =>
                                    crate::MessageArg::Resource("COOP_SMALL_REFINER".into()),
                                CoopOtherKind::Other { description } =>
                                    crate::MessageArg::Text(description.clone()),
                                _ => crate::MessageArg::Text(item.source.clone()),
                            },
                            crate::MessageArg::Text(item.source.clone()),
                        ],
                    ),
                    context.clone(),
                );
            }
            crate::require_localized!(
                r,
                credit_income_known(item, &eligible),
                &context,
                "Cooperative credit income",
                "ISSUE_COOP_CREDIT_INCOME_NEEDED",
                "The actual source does not establish the required credit-related income or its accepted recipient. No required income is guessed.",
            );
        }
    }
    result
}
