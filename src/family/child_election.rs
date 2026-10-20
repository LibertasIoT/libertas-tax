//! Form 8814: a parent's election to report a child's interest and dividends. Amounts entering arithmetic are whole dollars.

// Coverage: Form 8814 — Supported (audited 2026-10-09).
//   Lines: A (the child's name, the row's label), C (box C, printed only), 1a–15 per child. The
//   page is a document kind of the Interest and dividends topic (IncomeDocument::Form8814,
//   FederalIncome::ChildElection), offered only when the return has a child who can be reported
//   (the hidden children list of SaveIncomes, EstimateDocuments and PreviewDocument: Relationship::Child,
//   under 19 or a full-time student under 24 at the end of 2026 by day-exact age, no joint return of
//   their own). Each child's 1099-INT, -DIV
//   and -OID statements are the filer's own importable structs (Int1099, Div1099, Oid1099), so AI
//   import reads them per statement; the Alaska Permanent Fund dividend, whose return reports the
//   child's income (OtherParentChoice) and the facts no statement shows (ChildElectionFacts) are
//   the person's. Line 1a is the statements' own per-payer netting (FederalInterest::net_taxable,
//   FederalOid::net_taxable); line 1b is the child's tax-exempt interest (its private-activity part
//   is the parent's Form 6251 line 2g preference and Pub 596 Worksheet 1 line 2); line 4 over $1,350
//   and under $13,500 reports the child (at or below $1,350 nothing is reported and
//   NOTE_CHILD_ELECTION_NOT_REQUIRED says so); lines 6-12 split the excess over $2,700 by the
//   dividend and capital gain distribution fractions (line 9 to Form 1040 lines 3a and 3b, line 10 to
//   Schedule D line 13 with its 28% and unrecaptured section 1250 shares for the worksheets, line 12
//   to Schedule 1 line 8z); line 15 (10% of the excess over $1,350, at most $135) joins Form 1040
//   line 16 and is never a credit base, Form 8615 line 10 or Schedule J income. Line 12 less the
//   Alaska Permanent Fund share (Pub 596 Worksheet 2) is the Form 8960 line 7 amount, Form 4952
//   investment income and the earned income credit's Worksheet 1 line 4 amount; the increased
//   AGI reduces every AGI-based deduction and credit through the shared AGI.
//   Left out with one notice naming the child (the election is optional, so the parent's return
//   without it is correct and the child files their own): line 4 at or above $13,500, any federal
//   withholding, any statement with a special treatment or nominee amount, other income or
//   estimated payments, a noncustodial parent (joint return or LivedApart with the child in the home
//   under 183 days), the other parent's return when their taxable income is higher, and an unresolved
//   choice of return (assumed the other parent's). Kept with a note: a foreign account or trust
//   (Schedule B Part III and Form 8938 are not prepared) and a Form 4952 deduction (the child's
//   qualified dividends and capital gain distributions are not offered for the line 4g election).
//   Not prepared: the child's own blind addition, early withdrawal penalty and charitable gifts
//   (the early withdrawal penalty box is ignored), the comparison with the child's own Form 8615 tax
//   (a later unit), printed Form 8814 and Form 1040 line 3c / 7b / 16 box 1 checkboxes (the return
//   has no checked-box line kind; box C is printed only). Form 8615 stays independent: a child
//   reported here files no return, a filer's own Form 8615 parent page is unchanged.
//   Evidence: Form 8814 and instructions (2026 draft, retrieved 2026-10-08), Rev. Proc. 2025-32
//   section 3.02, IRC 1(g)(7), Pub 596 Worksheets 1 and 2, i8960 "Form 8814 election", i6251 line 2g;
//   year_constants.rs CHILD_ELECTION_*; src/tests/family/family.rs child_election_* (fixtures A-E of the
//   map), src/tests/investment/investment.rs child_election_import_*.

use crate::federal_model::{
    ChildIncome, ChildStatementKept, FederalChildElection, FederalDependent,
};
use crate::kernel::phase;
use crate::year_constants::{
    CHILD_ELECTION_BASE_DOLLARS, CHILD_ELECTION_LIMIT_DOLLARS, CHILD_ELECTION_UNTAXED_DOLLARS,
};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport, libertas_bitflags};

/// Child's interest and dividends (Form 8814)
/// Report your child's interest, dividends and capital gain distributions on your return instead of filing a return for the child: a child under 19, or a full-time student under 24, at the end of 2026, whose only income was interest and dividends. Income between $1,350 and $2,700 is taxed at 10% here even if the child's own rate would be lower.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct Form8814 {
    /// Child
    #[libertas_enum_source("$.children")]
    pub child: i64,
    /// The child's statements
    /// Forms 1099-INT, 1099-DIV and 1099-OID issued in the child's name and Social Security number, not yours.
    /// ----
    /// Statement
    #[libertas_unordered]
    pub statements: Vec<ChildStatement>,
    /// Alaska Permanent Fund dividend paid to the child
    /// The 2026 dividend from the State of Alaska. It counts as the child's ordinary dividends; it is not on a Form 1099-DIV.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub alaska_dividend: i64,
    /// Child's Social Security benefits that are not taxable
    /// Include the nontaxable portion for Marketplace household income. This is not taxable income on your Form 8814. Do not enter it again as a required-to-file dependent return.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub nontaxable_social: i64,
    /// Whose return reports this child's income
    /// Asked only when this is not a joint return. The child's other parent is the person who did not file this return; a new spouse of the parent the child lives with counts as the other parent.
    #[libertas_constrained_by("$.allowed_other_parent")]
    pub other_parent: OtherParentChoice,
    /// Does any of these apply to this child?
    #[libertas_bitflags(ChildElectionFacts)]
    #[libertas_default(0)]
    pub facts: u32,
}

/// Statement
#[derive(
    Clone,
    Debug,
    PartialEq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
#[allow(clippy::large_enum_variant)]
pub enum ChildStatement {
    /// 1099-INT
    Int1099(crate::Int1099),
    /// 1099-DIV
    Div1099(crate::Div1099),
    /// 1099-OID
    Oid1099(crate::Oid1099),
}

/// Whose return reports this child's income
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
pub enum OtherParentChoice {
    /// Not needed for this return
    NotNeeded,
    /// The child's other parent did not live with you in 2026
    /// Your return reports the child's income when the child lived with you for more than half the year.
    LivedApart,
    /// The child's other parent lived with you, and your taxable income is the higher
    TogetherYouHigher,
    /// The child's other parent lived with you, and their taxable income is the higher
    /// Their return reports the child's income; this child is left out of yours.
    TogetherOtherHigher,
    /// The other parent's taxable-income records are unavailable
    /// The child is left out of this estimate with a note, and would file their own return.
    // This return cannot reconstruct another parent's taxable income from the child's statements.
    // WARN_CHILD_ELECTION_PARENT_ASSUMED conservatively omits the election until that source is supplied.
    Unresolved,
}
impl OtherParentChoice {
    /// The choices the page offers: every answer when the question is needed (a return that is not a joint return), otherwise "Not needed" alone, which the client selects itself.
    pub fn allowed(needed: bool) -> Vec<i32> {
        use libertas_macros::variant_index;
        if needed {
            alloc::vec![
                variant_index!(OtherParentChoice::LivedApart) as i32,
                variant_index!(OtherParentChoice::TogetherYouHigher) as i32,
                variant_index!(OtherParentChoice::TogetherOtherHigher) as i32,
                variant_index!(OtherParentChoice::Unresolved) as i32,
            ]
        } else {
            alloc::vec![variant_index!(OtherParentChoice::NotNeeded) as i32]
        }
    }
    /// This choice's index in the allowed set.
    pub fn index(&self) -> i32 {
        use libertas_macros::variant_index;
        (match self {
            Self::NotNeeded => variant_index!(OtherParentChoice::NotNeeded),
            Self::LivedApart => variant_index!(OtherParentChoice::LivedApart),
            Self::TogetherYouHigher => variant_index!(OtherParentChoice::TogetherYouHigher),
            Self::TogetherOtherHigher => variant_index!(OtherParentChoice::TogetherOtherHigher),
            Self::Unresolved => variant_index!(OtherParentChoice::Unresolved),
        }) as i32
    }
}

/// Does any of these apply to this child?
#[libertas_bitflags]
pub struct ChildElectionFacts(pub u32);
impl ChildElectionFacts {
    /// None of these
    pub const NONE: Self = Self(0);
    /// Had other income in 2026
    /// Wages, self-employment, a scholarship, Social Security, a K-1 or anything besides interest, dividends and the Alaska Permanent Fund dividend. The child then files their own return.
    pub const OTHER_INCOME: Self = Self(1 << 0);
    /// Estimated tax was paid under the child's name and Social Security number for 2026
    /// Including a 2025 overpayment applied to 2026. The child then files their own return.
    pub const ESTIMATED_PAYMENTS: Self = Self(1 << 1);
    /// A statement includes amounts received as a nominee for someone else
    /// Not supported; the child then files their own return.
    pub const NOMINEE: Self = Self(1 << 2);
    /// Had a foreign financial account, or a distribution from, or was the grantor of or transferor to, a foreign trust
    /// Schedule B Part III and possibly Form 8938 are needed on your return; they are not prepared here.
    pub const FOREIGN_ACCOUNT: Self = Self(1 << 3);
}

// One child who can be reported on this return. The label is the child's
// name as a read-only FormattedText, the way PersonChoice labels a person;
// the stored child identity is the key, so a catalog change never moves a
// row to another child.
/// Child choice
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ChildChoice {
    /// Child identity
    #[libertas_key]
    #[libertas_unique(1)]
    #[libertas_hidden]
    pub id: i64,
    /// Child
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}

impl Form8814 {
    // The page onto the stored election: each statement through the shared
    // 1099 record mapping, with the boxes the form never reads kept beside it.
    pub(crate) fn record(&self) -> FederalChildElection {
        use crate::interview::{dividend_record, interest_record, oid_record};
        FederalChildElection {
            child_id: self.child,
            statements: self
                .statements
                .iter()
                .map(|statement| match statement {
                    ChildStatement::Int1099(v) => ChildIncome::Interest {
                        data: interest_record(v),
                        kept: ChildStatementKept {
                            payer: v.payer.clone(),
                            state_details: v.state_details.clone(),
                        },
                    },
                    ChildStatement::Div1099(v) => ChildIncome::Dividend {
                        data: dividend_record(v),
                        kept: ChildStatementKept {
                            payer: v.payer.clone(),
                            state_details: v.state_details.clone(),
                        },
                    },
                    ChildStatement::Oid1099(v) => ChildIncome::Oid {
                        data: oid_record(v),
                        kept: ChildStatementKept {
                            payer: v.payer.clone(),
                            state_details: v.state_details.clone(),
                        },
                    },
                })
                .collect(),
            alaska_dividend: self.alaska_dividend,
            nontaxable_social: self.nontaxable_social,
            other_parent: self.other_parent,
            facts: self.facts,
        }
    }
    // The stored election as its page row.
    pub(crate) fn from_record(record: &FederalChildElection) -> Self {
        use crate::interview::{dividend_page, interest_page, oid_page};
        Self {
            child: record.child_id,
            statements: record
                .statements
                .iter()
                .map(|statement| match statement {
                    ChildIncome::Interest { data, kept } => {
                        ChildStatement::Int1099(interest_page(data, kept))
                    }
                    ChildIncome::Dividend { data, kept } => {
                        ChildStatement::Div1099(dividend_page(data, kept))
                    }
                    ChildIncome::Oid { data, kept } => {
                        ChildStatement::Oid1099(oid_page(data, kept))
                    }
                })
                .collect(),
            alaska_dividend: record.alaska_dividend,
            nontaxable_social: record.nontaxable_social,
            other_parent: record.other_parent,
            facts: record.facts,
        }
    }
}

// What one election reports, in whole dollars (Form 8814 lines 1a-15). The
// statements' cents are summed per box and converted once, as the form's
// lines take whole dollars.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ChildLines {
    pub(crate) line_1a: i64,
    pub(crate) line_1b: i64,
    pub(crate) line_2a: i64,
    pub(crate) line_2b: i64,
    pub(crate) line_3: i64,
    pub(crate) line_4: i64,
    pub(crate) line_6: i64,
    pub(crate) line_9: i64,
    pub(crate) line_10: i64,
    pub(crate) line_11: i64,
    pub(crate) line_12: i64,
    pub(crate) line_14: i64,
    pub(crate) line_15: i64,
    // The 28% rate and unrecaptured section 1250 shares of line 10.
    pub(crate) collectible: i64,
    pub(crate) unrecaptured: i64,
    // The Alaska Permanent Fund share of line 12 (Pub 596 Worksheet 2).
    pub(crate) alaska_share: i64,
    // Specified private-activity bond interest, in cents (Form 6251 line 2g).
    pub(crate) private_activity: i64,
    // Federal tax withheld on the statements, in cents.
    pub(crate) withholding: i64,
    // A statement with a treatment the App does not support.
    pub(crate) special: bool,
}
impl ChildLines {
    // The income the election reports: over the amount not taxed (line 13)
    // and, for a child who can be reported at all, under the line 4 ceiling.
    pub(crate) fn reported(&self) -> bool {
        self.line_4 > CHILD_ELECTION_UNTAXED_DOLLARS
    }
    // Over the ceiling, the election is unavailable (section 1(g)(7)(A)(ii)).
    pub(crate) fn too_high(&self) -> bool {
        self.line_4 >= CHILD_ELECTION_LIMIT_DOLLARS
    }
    // Line 12 less the Alaska Permanent Fund share: the Form 8960 line 7
    // amount, Form 4952 investment income and Pub 596 Worksheet 1 line 4.
    pub(crate) fn investment_income(&self) -> i64 {
        self.line_12 - self.alaska_share
    }
}

impl FederalChildElection {
    pub(crate) fn lines(&self) -> ChildLines {
        use crate::kernel::ratio::{dollars, fraction, positive};
        let mut interest = 0;
        let mut exempt = 0;
        let mut ordinary = self.alaska_dividend;
        let mut qualified = 0;
        let mut distributions = 0;
        let mut collectible = 0;
        let mut unrecaptured = 0;
        let mut l = ChildLines::default();
        for statement in &self.statements {
            match statement {
                ChildIncome::Interest { data, .. } => {
                    interest += data.net_taxable();
                    exempt += data.net_exempt();
                    l.private_activity += data.private_activity;
                    l.withholding += data.withholding;
                    l.special |= data.special != crate::InterestSpecial::NoneOfThese;
                }
                ChildIncome::Oid { data, .. } => {
                    interest += data.net_taxable();
                    exempt += data.exempt;
                    l.private_activity += data.private_activity;
                    l.withholding += data.withholding;
                    l.special |= data.special != crate::OidSpecial::NoneOfThese;
                }
                ChildIncome::Dividend { data, .. } => {
                    ordinary += data.ordinary;
                    qualified += data.qualified;
                    distributions += data.capital_distributions;
                    collectible += data.collectible_distributions;
                    unrecaptured += data.unrecaptured_distributions;
                    exempt += data.exempt_dividends;
                    l.private_activity += data.private_activity_dividends;
                    l.withholding += data.withholding;
                    l.special |= data.special != crate::DividendSpecial::NoneOfThese;
                    // Form8814 permits capital-gain distributions, not a share
                    // lot's actual sale/liquidation gain or loss. Reuse the
                    // capital owner rather than silently ignoring those facts.
                    l.special |= data.capital_returns.iter().any(|lot| {
                        lot.special
                            || lot.kind.gain(
                                lot.amount + lot.property.iter().map(|p| p.value).sum::<i64>(),
                                lot.basis,
                            ) != 0
                    });
                }
            }
        }
        l.line_1a = dollars(interest);
        l.line_1b = dollars(exempt);
        l.line_2a = dollars(ordinary);
        l.line_2b = dollars(qualified);
        l.line_3 = dollars(distributions);
        l.line_4 = l.line_1a + l.line_2a + l.line_3;
        l.line_6 = positive(l.line_4 - CHILD_ELECTION_BASE_DOLLARS);
        if l.line_6 > 0 {
            // Lines 7 and 8 are fractions of line 4; one rounding each of
            // line 6 replaces the form's three-place decimals. The two
            // parts never exceed line 6.
            l.line_9 = fraction(l.line_6, l.line_2b, l.line_4);
            l.line_10 = phase::cap(fraction(l.line_6, l.line_3, l.line_4), [l.line_6 - l.line_9]);
            if l.line_3 > 0 {
                // The 28% and unrecaptured section 1250 shares of line 10,
                // for the Schedule D worksheets (the form's own fractions).
                l.collectible = fraction(l.line_10, dollars(collectible), l.line_3);
                l.unrecaptured = fraction(l.line_10, dollars(unrecaptured), l.line_3);
            }
        }
        l.line_11 = l.line_9 + l.line_10;
        l.line_12 = l.line_6 - l.line_11;
        l.line_14 = positive(l.line_4 - CHILD_ELECTION_UNTAXED_DOLLARS);
        // Line 15: 10% of line 14 below the amount not taxed, else 10% of it.
        l.line_15 = fraction(phase::cap(l.line_14, [CHILD_ELECTION_UNTAXED_DOLLARS]), 10, 100);
        let income = l.line_1a + l.line_2a - l.line_2b;
        if l.line_12 > 0 && income > 0 {
            l.alaska_share = fraction(l.line_12, dollars(self.alaska_dividend), income);
        }
        l
    }
}

// Why an election is left out of the estimate; the child then files their own
// return. Each has one notice naming the child.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Omission {
    // The child is no longer one who can be reported on this return.
    Ineligible,
    TooHigh,
    Withholding,
    Special,
    // Other income, or estimated tax paid under the child's number.
    OtherFacts,
    Noncustodial,
    OtherParent,
    ParentAssumed,
}
impl Omission {
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Ineligible | Self::OtherFacts => "WARN_CHILD_ELECTION_INELIGIBLE_LEFT_OUT",
            Self::TooHigh => "WARN_CHILD_ELECTION_INCOME_TOO_HIGH_LEFT_OUT",
            Self::Withholding => "WARN_CHILD_ELECTION_WITHHOLDING_LEFT_OUT",
            Self::Special => "WARN_CHILD_ELECTION_SPECIAL_LEFT_OUT",
            Self::Noncustodial => "WARN_CHILD_ELECTION_NONCUSTODIAL_LEFT_OUT",
            Self::OtherParent => "WARN_CHILD_ELECTION_OTHER_PARENT_LEFT_OUT",
            Self::ParentAssumed => "WARN_CHILD_ELECTION_PARENT_ASSUMED",
        }
    }
    // The custody and eligibility omissions can be corrected on the people
    // page as well as on the election's row.
    pub(crate) fn alternatives(self) -> bool {
        matches!(self, Self::Ineligible | Self::Noncustodial)
    }
}

// Days in the home that make the parent the custodial parent: more than half
// the year (IRC 152(e)).
const CUSTODIAL_DAYS: u32 = 183;

// Whether the dependent can be reported on this return: a child, adopted
// child or stepchild (a foster child files their own return), under 19 at
// the end of 2026 or a full-time student under 24, without a joint return of
// their own (IRC 1(g)(2), 1(g)(7)(A)). The age is the day-before rule the Form
// 8814 and 8615 instructions write ("A child born on January 1, 2008, is
// considered to be age 18 at the end of 2025", 2025 i1040), not the birthday
// rule of Rev. Rul. 2003-72, which names no section 1(g) test.
pub(crate) fn eligible_child(dependent: &FederalDependent) -> bool {
    let age = crate::federal_model::age_at_year_end(dependent.birth_date);
    dependent.relationship == crate::Relationship::Child
        && !dependent.joint_return_nonrefund()
        && (age < 19 || (dependent.student() && age < 24))
}

impl FederalChildElection {
    // The first reason this election cannot be reported, in the order the
    // conditions are explained; None when it can.
    pub(crate) fn omission(
        &self,
        dependents: &[FederalDependent],
        joint: bool,
    ) -> Option<Omission> {
        let child = dependents.iter().find(|v| v.id == self.child_id);
        let Some(child) = child.filter(|v| eligible_child(v)) else {
            return Some(Omission::Ineligible);
        };
        let lines = self.lines();
        let facts = self.facts;
        let has = |fact: ChildElectionFacts| facts & fact.0 != 0;
        if lines.too_high() {
            Some(Omission::TooHigh)
        } else if lines.withholding != 0 {
            Some(Omission::Withholding)
        } else if lines.special || has(ChildElectionFacts::NOMINEE) {
            Some(Omission::Special)
        } else if has(ChildElectionFacts::OTHER_INCOME)
            || has(ChildElectionFacts::ESTIMATED_PAYMENTS)
        {
            Some(Omission::OtherFacts)
        } else if (joint || self.other_parent == OtherParentChoice::LivedApart)
            && child.days_in_home() < CUSTODIAL_DAYS
        {
            Some(Omission::Noncustodial)
        } else if self.other_parent == OtherParentChoice::TogetherOtherHigher {
            Some(Omission::OtherParent)
        } else if self.other_parent == OtherParentChoice::Unresolved {
            Some(Omission::ParentAssumed)
        } else {
            None
        }
    }
    // Facts that keep the election but need a note.
    pub(crate) fn foreign_account(&self) -> bool {
        self.facts & ChildElectionFacts::FOREIGN_ACCOUNT.0 != 0
    }
}

// One election's Form 8814 lines on the return, each one line of the form
// (a line with nothing on it is left off, as the other forms leave theirs).
pub(crate) fn report(r: &mut crate::FederalResult, id: i64, lines: &ChildLines) {
    use crate::kernel::lines::form_line;
    use crate::{FormLine, TaxForm};
    const SOURCE: &str = "https://www.irs.gov/instructions/i8814";
    let rows: [(&str, &str, i64, bool, crate::Message); 13] = [
        ("1a", "interest", lines.line_1a, false, msg!("EXPLAIN_CHILD_ELECTION_N_INTEREST")),
        (
            "1b",
            "exempt_interest",
            lines.line_1b,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_EXEMPT_INTEREST"),
        ),
        (
            "2a",
            "ordinary_dividends",
            lines.line_2a,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_ORDINARY_DIVIDENDS"),
        ),
        (
            "2b",
            "qualified_dividends",
            lines.line_2b,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_QUALIFIED_DIVIDENDS"),
        ),
        (
            "3",
            "capital_gain_distributions",
            lines.line_3,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_CAPITAL_GAIN_DISTRIBUTIONS"),
        ),
        ("4", "total_income", lines.line_4, true, msg!("EXPLAIN_CHILD_ELECTION_N_TOTAL_INCOME")),
        (
            "6",
            "income_over_base",
            lines.line_6,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_INCOME_OVER_BASE"),
        ),
        (
            "9",
            "qualified_dividends_reported",
            lines.line_9,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_QUALIFIED_DIVIDENDS_REPORTED"),
        ),
        (
            "10",
            "capital_gain_distributions_reported",
            lines.line_10,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_CAPITAL_GAIN_DISTRIBUTIONS_REPORTED"),
        ),
        (
            "11",
            "reported_total",
            lines.line_11,
            false,
            msg!("EXPLAIN_CHILD_ELECTION_N_REPORTED_TOTAL"),
        ),
        ("12", "other_income", lines.line_12, false, msg!("EXPLAIN_CHILD_ELECTION_N_OTHER_INCOME")),
        (
            "14",
            "taxable_over_untaxed",
            lines.line_14,
            true,
            msg!("EXPLAIN_CHILD_ELECTION_N_TAXABLE_OVER_UNTAXED"),
        ),
        ("15", "tax", lines.line_15, true, msg!("EXPLAIN_CHILD_ELECTION_N_TAX")),
    ];
    for (line, name, amount, always, explanation) in rows {
        if amount != 0 || always {
            form_line(
                r,
                FormLine::new(TaxForm::Form8814, line),
                &alloc::format!("child_election_{id}_{name}"),
                amount,
                &["income.child_election"],
                SOURCE,
                explanation,
            );
        }
    }
}

impl ChildStatementKept {
    // The boxes the form never reads, inside the limits of the filer's own.
    fn valid(&self) -> bool {
        crate::IncomeStateSource::Investment(self.state_details.clone()).valid()
    }
}

impl FederalChildElection {
    // Stored values inside the limits of the filer's own statements.
    pub(crate) fn valid(&self) -> bool {
        use crate::FederalIncome as F;
        let ok = |income: F| crate::federal_rules::validate_income_amount(&income).is_ok();
        self.statements.len() <= 20
            && (0..=100000000000).contains(&self.alaska_dividend)
            && (0..=100000000000).contains(&self.nontaxable_social)
            && self.facts
                & !(ChildElectionFacts::OTHER_INCOME.0
                    | ChildElectionFacts::ESTIMATED_PAYMENTS.0
                    | ChildElectionFacts::NOMINEE.0
                    | ChildElectionFacts::FOREIGN_ACCOUNT.0)
                == 0
            && self.statements.iter().all(|statement| match statement {
                ChildIncome::Interest { data, kept } => {
                    kept.valid() && ok(F::Interest { data: data.clone() })
                }
                ChildIncome::Dividend { data, kept } => {
                    kept.valid() && ok(F::Dividend { data: data.clone() })
                }
                ChildIncome::Oid { data, kept } => {
                    kept.valid() && ok(F::Oid { data: data.clone() })
                }
            })
    }
}
