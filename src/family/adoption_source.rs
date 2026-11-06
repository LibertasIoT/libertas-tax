// Accepted adoption expenses, employer-benefit histories and intrinsic source checks.
use crate::common::Owner;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Adoption finalization
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AdoptionFinalization {
    /// Not final, including an unsuccessful domestic attempt
    Pending,
    /// Final adoption
    Final {
        /// Year legally final under the Form 8839 rules
        // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
        #[libertas_number(min = 2000, max = 2026)]
        year: u32,
    },
}

/// Adoption jurisdiction
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AdoptionKind {
    /// U.S. citizen or resident child when the effort began
    Domestic {
        /// Finalization
        finalized: AdoptionFinalization,
        /// State or Indian tribal government determined special needs under the adoption-credit rules
        /// This is a formal determination that the child cannot or should not return home and probably cannot be adopted without assistance. A disability alone does not establish this status.
        special_needs: bool,
    },
    /// Foreign child
    Foreign {
        /// Finalization
        finalized: AdoptionFinalization,
    },
}

/// Prior credit for this child
/// From the last filed Form 8839 for this child; leave both amounts at zero when no credit was claimed before.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AdoptionPriorCredit {
    /// Line 3 of the last filed Form 8839 for this child
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub previous: i64,
    /// Line 6 of that same form, before the income phaseout
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub expenses: i64,
}

/// Qualified adoption expense paid
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AdoptionExpense {
    /// Payment year
    // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
    #[libertas_number(min = 2000, max = 2026)]
    pub year: u32,
    /// Unreimbursed qualified fees, court costs, attorney fees, travel or other directly related expense
    /// Exclude employer benefits, government reimbursements, surrogate costs and amounts used for another deduction or credit. Include an expense only once.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}

/// Adoption expenses and benefits for one child
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AdoptionChild {
    /// Child or adoption record label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Jurisdiction and finalization
    pub kind: AdoptionKind,
    /// Per-child limits available to this return
    pub limits: AdoptionLimits,
    /// Payments from your records
    pub expenses: Vec<AdoptionExpense>,
    /// Previous claim for the same child
    pub prior: AdoptionPriorCredit,
    /// Employer adoption assistance
    pub benefits: AdoptionBenefits,
    /// Child lived in your home for more than half of 2026
    /// Required for the current-credit or benefit-exclusion exception when married filing separately. The App combines this fact with your marital living arrangement and home-cost contribution in Setup. Other filing statuses do not require this adoption residence test. School, medical or custody records show the child's address.
    pub home_over_half_year: bool,
    /// Eligible child and expenses confirmed
    /// The child was under 18 or unable to care for themselves when expenses were paid; this is not adoption of your spouse's child. Requirements for identifying the child and lawful adoption are met. Combine related attempts for the same child, and resolve unsuccessful-attempt allocations before entry. Credit expenses exclude every employer reimbursement. Keep the per-child limit agreement consistent with every other adopting taxpayer. Adoption papers, the child's age or special-needs determination and your expense receipts show it. No leaves this child's credit and employer-benefit exclusion out of the estimate with a notice; that situation is not supported.
    pub eligible: bool,
}

/// Unused nonrefundable adoption credit from a filed return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AdoptionCarry {
    /// Year the credit originated
    // Tax-year structure: the recorded date/year cannot be later than this applicable tax-year window.
    #[libertas_number(min = 2021, max = 2025)]
    pub year: u32,
    /// Remaining credit in that year's row of the latest carryforward worksheet
    /// Enter an already phased-out credit balance from the filed worksheet, not expenses. Current carryovers are never refundable. Keep ownership unchanged from the prior return.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
    /// Your available credit from an eligible original return
    /// Confirm the documented share belongs to this filer and has not been used on another return. If married when the expenses first became allowable, a joint original return is required unless treated as unmarried under the applicable rules. Resolve allocation from a prior joint return before entering your share; never enter both spouses' combined balance on each separate return. Your prior Form 8839 carryforward worksheet shows it. No leaves this carryforward out of the estimate with a notice; that situation is not supported.
    pub eligible: bool,
}

pub(crate) fn valid(children: &[AdoptionChild], carries: &[AdoptionCarry]) -> bool {
    let amount = |v| (0..=100000000000).contains(&v);
    children.len() <= 30
        && children.iter().all(|c| {
            let (finalized, _) = timing(&c.kind);
            !c.label.trim().is_empty()
                && finalized.is_none_or(|y| (2000..=2026).contains(&y))
                && c.benefits.valid()
                && c.limits.valid()
                && c.expenses.len() <= 100
                && c.expenses.iter().all(|v| (2000..=2026).contains(&v.year) && amount(v.amount))
                && amount(c.prior.previous)
                && amount(c.prior.expenses)
        })
        && carries.iter().enumerate().all(|(i, c)| {
            (2021..=2025).contains(&c.year)
                && amount(c.amount)
                && !carries[..i].iter().any(|p| p.year == c.year)
        })
}
pub(crate) fn timing(kind: &AdoptionKind) -> (Option<u32>, bool) {
    let (f, domestic) = match kind {
        AdoptionKind::Domestic { finalized, .. } => (finalized, true),
        AdoptionKind::Foreign { finalized } => (finalized, false),
    };
    (
        match f {
            AdoptionFinalization::Pending => None,
            AdoptionFinalization::Final { year } => Some(*year),
        },
        domestic,
    )
}
/// Earlier employer adoption benefits for this child
/// Keep a prior exclusion-limit claim separate from foreign-adoption benefits that were taxed before finalization. The App computes the current exclusion; do not enter a computed exclusion here.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AdoptionBenefitHistory {
    /// No earlier employer benefits
    None,
    /// Earlier exclusion-limit claim
    Exclusion {
        /// Prior Form 8839 line 20 plus line 24 for this child
        /// Use the last filed form's amounts before income phaseout, separately from prior credit amounts.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        used: i64,
    },
    /// Foreign-adoption benefits taxed before finalization
    TaxedForeign {
        /// Benefits received before 2026 and included in income on those returns
        /// Combine this employee's qualified plan benefits for this child. They must not have been excluded, repaid, or entered as 2026 benefits. The finalization-year worksheet applies when the foreign adoption becomes final in 2026.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        received: i64,
    },
}

/// One employer's adoption assistance for this child
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AdoptionEmployerPayment {
    /// Employee receiving the benefit
    pub owner: Owner,
    /// Employer's W-2
    /// Choose one ordinary W-2 of this employee. Use the corrected W-2 instead of entering the original and correction twice.
    pub wage: i64,
    /// This employer's W-2 box 12 code T for this child
    /// Enter the qualified-plan amount paid in 2026 as whole dollars. It is already in that W-2's Social Security and Medicare wages, but not box 1 or box 2; do not add a separate withholding amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
    pub received: i64,
}

/// Which employee receives a shared, partly available adoption exclusion
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum JointAdoptionExclusion {
    /// Split in proportion to each employee's code T
    /// When part of this child's benefits is excluded, each employee's taxable part follows their share of the code T; it only matters for each spouse's earned income on Form 2441.
    Automatic,
    /// Confirmed employee attribution of a partly available exclusion
    Attributed {
        /// Exclusion assigned to the taxpayer's code T benefits
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        taxpayer: i64,
        /// Exclusion assigned to the spouse's code T benefits
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        spouse: i64,
        /// Supporting allocation confirmed
        /// Use an allocation supported by the joint return's records. The two amounts must add to this child's Form 8839 line 29 exclusion when it is less than actual code T, and cannot exceed either employee's code T benefits. Form 8839 does not prescribe an employee ordering. A special-needs deemed excess above actual code T needs no employee attribution. No leaves this child's partly available exclusion out of the estimate with a notice, so all its code T benefits are taxable; that situation is not supported.
        confirmed: bool,
    },
}

impl JointAdoptionExclusion {
    fn valid(&self) -> bool {
        match self {
            Self::Automatic => true,
            Self::Attributed { taxpayer, spouse, .. } => [*taxpayer, *spouse]
                .iter()
                .all(|amount| (0..=100000000000).contains(amount) && amount % 100 == 0),
        }
    }
}

/// Employer adoption assistance for this child
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AdoptionBenefits {
    /// No employer adoption benefits or special-needs plan exclusion
    None,
    /// Employer adoption assistance program
    Program {
        /// Employee receiving the assistance
        owner: Owner,
        /// Current-year benefits, Form W-2 box 12 code T
        /// Combine assistance for this child from this employee's employers. These benefits must not already be in W-2 box 1. Use zero for an eligible special-needs plan exclusion without actual benefits.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        received: i64,
        /// Earlier benefits for this child
        history: AdoptionBenefitHistory,
        /// Ordinary qualified plan and complete benefit records
        /// Confirm a written qualified assistance plan; no more-than-2% S-corporation ownership, benefits for a second employee, benefits already included in W-2 box 1, or unreported earlier benefits. Credit expenses exclude all reimbursed amounts, even if the benefits become taxable. Your employer's plan documents show it. No treats these benefits as taxable, with a notice.
        /// Check the employer plan, matched Forms W-2 and earlier benefit records.
        ordinary: bool,
    },
    /// Qualified benefits paid to both spouses by separate employers
    JointEmployees {
        /// 2026 W-2 code T payments for this child
        /// Enter each employer once, including different employers of the same employee. Both spouses must have a payment. When two children use this path, every matched W-2's entire code T amount must belong to exactly one child; do not reuse a W-2 across children. The App applies one exclusion per child and never adds code T to W-2 box 1 or withholding.
        #[libertas_size(min = 2)]
        payments: Vec<AdoptionEmployerPayment>,
        /// Employee attribution of a partly available actual-benefit exclusion
        /// Automatic splits a partly excluded child's benefits in proportion to each employee's code T; it only moves each spouse's earned income for Form 2441. Choose a confirmed split only to document a different one from supporting records; it cannot change Form 8839 line 29.
        allocation: JointAdoptionExclusion,
        /// Earlier benefits for this child
        /// Benefits excluded for this child on an earlier Form 8839 reduce this year's $17,670 limit (line 20). Foreign benefits taxed before the adoption became final are not supported on this path.
        history: AdoptionBenefitHistory,
        /// Ordinary plans and complete W-2s confirmed
        /// Yes on a joint return for U.S. children: each employer had a written qualified adoption assistance plan, every matched W-2's whole code T amount is entered once for one child and sits in that W-2's Social Security and Medicare wages but not box 1 or box 2, and no reimbursed expense dollar belongs to another child. A special-needs child whose adoption became final in 2026 excludes the whole remaining limit whatever the code T; any excess reduces joint Form 1040 line 1f. Credit expenses exclude all reimbursements. No treats these benefits as taxable, with a notice.
        /// Check the employer plan, matched Forms W-2 and earlier benefit records.
        ordinary: bool,
    },
}
impl AdoptionBenefits {
    fn valid(&self) -> bool {
        match self {
            Self::None => true,
            Self::JointEmployees { payments, allocation, history, .. } => {
                (2..=20).contains(&payments.len())
                    && allocation.valid()
                    && payments.iter().any(|p| p.owner == Owner::Taxpayer)
                    && payments.iter().any(|p| p.owner == Owner::Spouse)
                    && payments.iter().enumerate().all(|(index, p)| {
                        p.wage > 0
                            && (100..=100000000000).contains(&p.received)
                            && p.received % 100 == 0
                            && !payments[..index]
                                .iter()
                                .any(|prior| prior.owner == p.owner && prior.wage == p.wage)
                    })
                    && match history {
                        AdoptionBenefitHistory::None => true,
                        AdoptionBenefitHistory::Exclusion { used } => {
                            (0..=100000000000).contains(used)
                        }
                        AdoptionBenefitHistory::TaxedForeign { received } => {
                            (0..=100000000000).contains(received)
                        }
                    }
            }
            Self::Program { received, history, .. } => [
                *received,
                match history {
                    AdoptionBenefitHistory::None => 0,
                    AdoptionBenefitHistory::Exclusion { used } => *used,
                    AdoptionBenefitHistory::TaxedForeign { received } => *received,
                },
            ]
            .iter()
            .all(|v| (0..=100000000000).contains(v)),
        }
    }
}

/// Per-child adoption-limit agreement
/// Credit and employer-benefit exclusion have separate $17,670 limits in 2026. When another taxpayer outside this joint return also adopts the same child, use the agreed shares for this return before subtracting prior claims. These are agreed limits, not expenses or a computed credit.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AdoptionLimits {
    /// Full limits available to this return
    /// No part of either limit is allocated to another return. Spouses filing jointly share one full limit of each kind.
    Full,
    /// Limits shared with another return
    Shared {
        /// This return's agreed credit limit
        #[libertas_money("USD", 2)]
        // Section 23/137 statutory per-child credit/exclusion share; native AdoptionLimits::valid checks it.
        #[libertas_number(min = 0, max = $ADOPTION_CREDIT_MAXIMUM_CENTS)]
        credit: i64,
        /// This return's agreed employer-benefit exclusion limit
        /// This may differ from the credit share. Use the full limit if only the credit limit is shared, or zero if all of this exclusion limit belongs to the other return.
        #[libertas_money("USD", 2)]
        // Section 23/137 statutory per-child credit/exclusion share; native AdoptionLimits::valid checks it.
        #[libertas_number(min = 0, max = $ADOPTION_CREDIT_MAXIMUM_CENTS)]
        exclusion: i64,
        /// All adopting taxpayers agreed and the shares reconcile
        /// For each limit independently, combined shares on all returns cannot exceed $17,670. Include prior-use records and use consistent whole-dollar reporting. No expense or benefit is duplicated between returns. No leaves this child's credit and exclusion out of the estimate with a notice; that situation is not supported.
        agreed: bool,
    },
}
impl AdoptionLimits {
    fn valid(&self) -> bool {
        match self {
            Self::Full => true,
            Self::Shared { credit, exclusion, .. } => [*credit, *exclusion]
                .iter()
                .all(|v| (0..=crate::year_constants::ADOPTION_CREDIT_MAXIMUM_CENTS).contains(v)),
        }
    }
}
