use crate::{InjuredAmount, Owner};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Legal borrowers of the claimed student loans
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum InjuredLoanBorrower {
    /// Taxpayer alone
    Taxpayer,
    /// Spouse alone
    Spouse,
    /// Both spouses, on the same or different claimed loans
    Both,
}

/// Injured Debt Facts
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[libertas_inline_choices]
pub enum InjuredDebtFacts {
    /// This state's allocation does not need these facts
    NotApplicable,
    /// Debt arose before marriage, including the cause of action
    BeforeMarriage,
    /// Debt arose during marriage and was not for the family's interest
    DuringMarriageNotFamily,
    /// Debt arose during marriage for the family's interest
    DuringMarriageFamily,
}

/// Injured Spouse Allocation
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InjuredSpouseAllocation {
    /// Injured spouse
    pub injured: Owner,
    /// Offset debt
    pub debt: OffsetDebt,
    /// Legal obligation on the debt
    pub obligated: bool,
    /// Community-property facts
    pub community: InjuredCommunityFacts,
    /// Joint estimated payment allocation
    pub estimated: EstimatedSplit,
    /// Who is legally obligated on the loans whose interest is deducted
    /// Leave empty when there is no student-loan deduction. A borrower who is only a dependent student does not make a spouse legally obligated.
    #[libertas_default($null)]
    pub student_loan_borrower: Option<InjuredLoanBorrower>,
    /// Accepted shared-component choices
    #[libertas_size(min = 0)]
    pub allocations: Vec<InjuredAmount>,
    /// Separate-return dependent claimants
    #[libertas_size(min = 0)]
    pub claimants: Vec<InjuredClaimantFacts>,
}

/// Injured Claimant Facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InjuredClaimantFacts {
    /// Accepted dependent identity
    #[libertas_unique]
    pub person: i64,
    /// Spouse who would claim this dependent
    pub claimant: Owner,
}

/// Injured Community Facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum InjuredCommunityFacts {
    /// No community-property residence
    No,
    /// Community-property facts
    Yes {
        /// States of domicile
        // Tax structure: the nine community-property states, each selected at most once.
        #[libertas_size(min = 1, max = 9)]
        #[libertas_unordered]
        states: Vec<u16>,
        /// Debt timing and family use
        debt_facts: InjuredDebtFacts,
    },
}

/// Debt the refund pays
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum OffsetDebt {
    /// Federal tax
    FederalTax,
    /// State income tax
    StateIncomeTax,
    /// State unemployment compensation
    StateUnemployment,
    /// Child support
    ChildSupport,
    /// Spousal support
    SpousalSupport,
    /// Federal nontax debt, such as a student loan
    FederalNontax,
    /// None of these, or not a debt of the other spouse alone
    NoneOfThese,
}

/// Estimated tax and extension payments
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum EstimatedSplit {
    /// By the IRS formula
    /// Each spouse's share of the payments follows their separate tax. Use this when you do not agree on a split.
    IrsFormula,
    /// As we agree
    Agreed {
        /// Injured spouse's part
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        injured: i64,
    },
}
