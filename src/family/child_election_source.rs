use crate::accepted_dependents::FederalDependent;
use crate::dividend_source::FederalDividend;
use crate::interest_source::FederalInterest;
use crate::kernel::phase;
use crate::oid_source::FederalOid;
use crate::year_constants::{
    CHILD_ELECTION_BASE_DOLLARS, CHILD_ELECTION_LIMIT_DOLLARS, CHILD_ELECTION_UNTAXED_DOLLARS,
};
use alloc::vec::Vec;
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport, libertas_bitflags};

/// Child's interest and dividends (Form 8814)
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalChildElection {
    /// Child identity
    #[libertas_hidden]
    pub child_id: i64,
    /// The child's statements
    pub statements: Vec<ChildIncome>,
    /// Alaska Permanent Fund dividend paid to the child
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub alaska_dividend: i64,
    /// Child's Social Security benefits that are not taxable
    /// For Marketplace household income only; do not enter taxable benefits here.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub nontaxable_social: i64,
    /// Whose return reports this child's income
    pub other_parent: crate::OtherParentChoice,
    /// Does any of these apply to this child?
    #[libertas_bitflags(crate::ChildElectionFacts)]
    pub facts: u32,
}

/// Child's statement
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ChildIncome {
    /// Interest
    Interest {
        /// Statement amounts
        data: alloc::boxed::Box<FederalInterest>,
        /// Page-only boxes
        kept: ChildStatementKept,
    },
    /// Dividends
    Dividend {
        /// Stable child statement identity
        statement_id: i64,
        /// Printed form and personal answers
        data: alloc::boxed::Box<crate::DividendSource>,
    },
    /// Original issue discount
    Oid {
        /// Statement amounts
        data: alloc::boxed::Box<FederalOid>,
        /// Page-only boxes
        kept: ChildStatementKept,
    },
}

// What the page shows of a child's statement beyond the amounts the form
// reads: the payer and the state boxes. A filer's own statement keeps these
// in its entry label and `state_source`; the child's belong to the child's
// return, so they stay with the election and are never projected.
/// Page-only boxes of a child's statement
#[derive(
    Clone, Debug, Default, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub struct ChildStatementKept {
    /// Payer
    pub payer: crate::PrintedParty,
    /// Recipient as printed
    pub recipient: crate::PrintedParty,
    /// State boxes and supplemental information
    pub state_details: crate::InvestmentStateFields,
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
    /// Validate statement identities within the same child before preserving
    /// accepted personal answers. Allocate new identities only in a candidate.
    pub(crate) fn prepare_dividend_ids(
        &mut self,
        accepted: Option<&Self>,
        next_id: &mut i64,
    ) -> Result<(), crate::validation::ValidationIssue> {
        let accepted = accepted.filter(|before| before.child_id == self.child_id);
        let positions: Vec<_> = self
            .statements
            .iter()
            .enumerate()
            .filter_map(|(index, statement)| {
                matches!(statement, ChildIncome::Dividend { .. }).then_some(index)
            })
            .collect();
        crate::interview::assign_record_ids(
            self.statements.iter_mut().filter_map(|statement| match statement {
                ChildIncome::Dividend { statement_id, .. } => Some(statement_id),
                _ => None,
            }),
            accepted.into_iter().flat_map(|before| &before.statements).filter_map(|statement| {
                match statement {
                    ChildIncome::Dividend { statement_id, .. } if *statement_id != 0 => {
                        Some(*statement_id)
                    }
                    _ => None,
                }
            }),
            next_id,
        )
        .map_err(|(key, _, index)| {
            crate::validation::ValidationIssue::new(key)
                .at_field("id")
                .in_list("statements", positions[index])
        })?;
        if let Some(accepted) = accepted {
            self.keep_dividend_answers(accepted);
        }
        Ok(())
    }

    /// Preserve personal answers by the accepted statement identity, never its
    /// position or payer. The caller validates identities within this election
    /// and child before applying the current topic answers.
    pub(crate) fn keep_dividend_answers(&mut self, accepted: &Self) {
        if self.child_id != accepted.child_id {
            return;
        }
        for statement in &mut self.statements {
            let ChildIncome::Dividend { statement_id, data, .. } = statement else {
                continue;
            };
            if *statement_id == 0 {
                continue;
            }
            let Some(before) = accepted.statements.iter().find_map(|statement| match statement {
                ChildIncome::Dividend { statement_id: before, data, .. }
                    if before == statement_id =>
                {
                    Some(data)
                }
                _ => None,
            }) else {
                continue;
            };
            data.keep_answers(before);
        }
    }

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
                    let data = FederalDividend::from(data.as_ref());
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
    let age = crate::accepted_filers::age_at_year_end(dependent.birth_date);
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

impl ChildStatementKept {
    // The boxes the form never reads, inside the limits of the filer's own.
    pub(crate) fn valid(&self) -> bool {
        crate::IncomeStateSource::Investment(self.state_details.clone()).valid()
    }
}
