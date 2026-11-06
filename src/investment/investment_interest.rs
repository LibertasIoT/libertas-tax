// Coverage: Form 4952 — Supported with source limits (audited 2026-10-11).
//   Lines: numeric regular/AMT 1–8; source paid/carry; investment ST/LT loss
//   portions; capital-first or allocated 4g election; allowed connected taxes;
//   gross direct royalties and allowed costs; source-determined Schedule A9 /
//   Schedule E13 destinations, including standard-deduction royalty returns.
//   Limits: an unresolved mixed-source allowance/carry or incomplete early
//   royalty context leaves only the named deduction/election benefit out.
//   Untyped bond-premium character has the same benefit-only boundary.
//   No independent business/6198 destination API is claimed; protected activity
//   owners must supply their actual regular/AMT allowed source amounts.
//   Current K-1 interest uses independent regular/AMT basis allowances; a binding
//   at-risk stage remains the named Form 4952/Form 6198 coordination boundary.
//   Evidence: src/tests/passthrough/k1_interest_amt.rs.
//   Election with missing family/foreign owners remains expressly limited.
//   Evidence: src/tests/deductions/deduction_pages.rs investment_royalty_source_routes_once_and_survives_standard_deduction,
//   investment_exclusive_royalty_keeps_separate_amt_carry_and_line2c,
//   investment_limited_mixed_destinations_leave_only_unavailable_benefit_out;
//   src/tests/investment/investment.rs; src/tests/investment/retirement.rs; 2026 draft f4952--dft.pdf pp.3–4,
//   final2025 i6251.pdf printed p.3 (separate AMT context), posting dates unknown.

pub use crate::investment_borrowing_source::{
    InvestmentBorrowing, InvestmentElection, InvestmentIncomeTax, InvestmentInterest,
    InvestmentInterestUse, InvestmentLimitedAllocation, InvestmentLoan, InvestmentTaxCapAllocation,
};

use crate::kernel::phase;
use alloc::{collections::BTreeMap, string::String, vec, vec::Vec};
use libertas_macros::*;

// Native Form4952 disposition pool after its actual investment carryovers.
// Keep the same money unit and operand order; this stage has no annual
// capital-loss floor, filing status, deduction, or election allocation.
pub(crate) fn gain_pool(
    short: i64,
    long: i64,
    ordinary: i64,
    stock_loss: i64,
    property_ordinary: i64,
) -> [i64; 2] {
    use crate::kernel::ratio::positive;
    let gain = positive(short + long + ordinary - stock_loss + property_ordinary);
    [gain, phase::cap(gain, [positive(long - positive(-short))])]
}

/// Investment borrowing costs and retained disallowed interest
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct InvestmentBorrowingPage {
    /// Interest traced to each income source
    #[libertas_size(min = 1)]
    pub loans: Vec<InvestmentLoanPage>,
    /// Preferential-income election
    pub election: InvestmentElection,
    /// Income produced by the financed investments
    pub income_use: InvestmentInterestUse,
    /// Attribution when the combined deduction is limited
    pub limited_allocation: InvestmentLimitedAllocation,
}
/// Borrowing traced to one income source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("InvestmentOriginOptions")]
pub struct InvestmentLoanPage {
    /// Your loan or retained borrowing source label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Income source financed by this borrowing
    #[libertas_enum_source("$.interest_sources")]
    #[libertas_calculation_request("InvestmentOriginOptions")]
    pub source: i64,
    /// Eligible interest paid in 2026, excluding amounts deducted elsewhere
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub paid: i64,
    /// Regular 2025 Form 4952 line 7 attributable to this source
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    #[libertas_calculation_request("InvestmentOriginOptions")]
    pub carry: i64,
    /// Separate retained AMT Form 4952 carryover; do not assume equality
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_calculation_request("InvestmentOriginOptions")]
    pub amt_carry: i64,
    /// Additional interest on debt financing private-activity bonds
    /// Excluded from regular paid interest above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub private_activity: i64,
    /// Applicable originating-entity question
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.investment_origin_choices")]
    #[libertas_output_only]
    pub allowed_origin_choices: Vec<i32>,
    /// K-1 that originated this retained interest
    /// Select the actual source of the prior balance, not the credit's source.
    /// Current K-1 interest expense is already entered on that K-1 and must not be entered twice.
    /// [NullText]
    /// This balance is not attributable to a K-1 entity on this return
    /// [SetText]
    /// This retained balance came from
    #[libertas_constrained_by("^.allowed_origin_choices")]
    pub originating_k1: Option<InvestmentK1Origin>,
    /// Person who paid the current portfolio interest
    /// This is independent of who owns an earlier disallowed carryover.
    /// [NullText]
    /// The current payer is not established
    /// [SetText]
    /// The current interest was paid by
    #[libertas_default($null)]
    pub paid_owners: Option<crate::DeductionOwners>,
    /// Person retaining the regular Form 4952 carryover
    /// A K-1 entity that originated the balance does not identify the individual retaining it.
    /// [NullText]
    /// The retained individual owner is not established
    /// [SetText]
    /// The retained carryover belongs to
    #[libertas_default($null)]
    pub carry_owners: Option<crate::DeductionOwners>,
}

/// Actual originating entity of a retained investment-interest balance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum InvestmentK1Origin {
    /// A K-1 entity on this return
    Entity {
        /// The actual originating Schedule K-1
        #[libertas_enum_source("$.investment_k1_sources")]
        source: i64,
    },
}
impl InvestmentK1Origin {
    pub(crate) fn id(&self) -> i64 {
        match self {
            Self::Entity { source } => *source,
        }
    }
}

pub(crate) fn k1_sources(d: &crate::FederalDraft) -> Vec<crate::RecordChoice> {
    crate::business_credit_page::research_businesses(d)
        .into_iter()
        .filter(|source| {
            d.income.iter().any(|entry| {
                entry.id == source.id
                    && matches!(
                        entry.income,
                        crate::FederalIncome::Partnership { .. }
                            | crate::FederalIncome::Scorp { .. }
                            | crate::FederalIncome::PartnershipActivities { .. }
                            | crate::FederalIncome::ScorpActivities { .. }
                    )
            })
        })
        .collect()
}
pub(crate) fn origin_choices(d: &crate::FederalDraft) -> Vec<i32> {
    let historical =
        d.credits.as_ref().and_then(|c| c.business_credits.as_ref()).is_some_and(|h| {
            h.credits.iter().any(|c| c.credit.research())
                || h.carryforwards.iter().any(|c| c.credit.research())
                || h.carrybacks.iter().any(|c| c.credit.research())
        });
    let current = d.income.iter().any(|entry| match &entry.income {
        crate::FederalIncome::Partnership { data } => {
            data.credits.iter().any(|c| c.credit().research())
        }
        crate::FederalIncome::Scorp { data } => {
            let data = &crate::ScorpK1::from(data);
            data.credits.iter().any(|c| c.credit().research())
        }
        crate::FederalIncome::PartnershipActivities { data } => {
            data.entity.credits.iter().any(|c| c.credit().research())
        }
        crate::FederalIncome::ScorpActivities { data } => {
            data.entity.credits.iter().any(|c| c.credit().research())
        }
        _ => false,
    });
    if (historical || current) && !k1_sources(d).is_empty() { vec![0] } else { vec![] }
}
pub(crate) fn loan_origin_choices(
    d: &crate::FederalDraft,
    source: i64,
    carry: i64,
    amt_carry: i64,
) -> Vec<i32> {
    if source == 0 && (carry > 0 || amt_carry > 0) { origin_choices(d) } else { vec![] }
}

// Reg1.53-3(d)(1): a limited deduction comprising entity and other items
// is prorated. This is a projection of the Form4952 owner's allowed amount;
// it never changes destinations, allowances or regular/AMT carry attribution.
pub(crate) fn research_parts(
    borrowing: Option<&InvestmentBorrowing>,
    current_k1: &BTreeMap<i64, i64>,
    allowed_dollars: i64,
) -> BTreeMap<i64, i64> {
    let mut cents = BTreeMap::<Option<i64>, i64>::new();
    if let Some(borrowing) = borrowing {
        for loan in borrowing.loans.iter().filter(|loan| loan.source == 0) {
            *cents.entry(None).or_default() += loan.paid;
            *cents.entry(loan.originating_k1).or_default() += loan.carry;
        }
    }
    for (&id, &amount) in current_k1 {
        *cents.entry(Some(id)).or_default() += amount;
    }
    let weights: Vec<_> = cents.values().copied().collect();
    let parts = crate::kernel::ratio::allocate_remaining(allowed_dollars, &weights);
    cents.into_keys().zip(parts).filter_map(|(id, amount)| id.map(|id| (id, amount))).collect()
}
/// Income source available for investment borrowing
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct InvestmentSourceChoice {
    #[libertas_key]
    #[libertas_unique]
    #[libertas_hidden]
    pub id: i64,
    /// Income source
    #[libertas_formatted_text]
    #[libertas_read_only]
    pub label: Vec<u8>,
}
impl InvestmentBorrowing {
    // A source group owns both its allowance and its remaining carry. The
    // aggregate limit never authorizes a guessed priority between sources.
    pub(crate) fn route(&self, forms: [[i64; 2]; 2]) -> Option<InvestmentRouting> {
        use crate::kernel::ratio::{dollars, positive};
        let mut cents = BTreeMap::<i64, [i64; 4]>::new();
        for loan in &self.loans {
            let amount = cents.entry(loan.source).or_default();
            for (sum, value) in amount.iter_mut().zip([
                loan.paid,
                loan.carry,
                loan.amt_carry,
                loan.private_activity,
            ]) {
                *sum += value;
            }
        }
        let costs: BTreeMap<_, _> = cents
            .into_iter()
            .map(|(source, v)| {
                (
                    source,
                    [dollars(v[0]) + dollars(v[1]), dollars(v[0]) + dollars(v[2]) + dollars(v[3])],
                )
            })
            .collect();
        let totals = core::array::from_fn::<_, 2, _>(|basis| forms[basis].iter().sum::<i64>());
        // Whole-dollar source groups must reconcile with the one form's
        // separately rounded paid/carry lines. No residual is assigned by fiat.
        if (0..2).any(|basis| costs.values().map(|v| v[basis]).sum::<i64>() != totals[basis]) {
            return None;
        }
        let mut sources = BTreeMap::new();
        for (source, cost) in costs {
            let mut allowed = [0; 2];
            for basis in 0..2 {
                let lower = positive(forms[basis][0] - (totals[basis] - cost[basis]));
                let upper = phase::cap(forms[basis][0], [cost[basis]]);
                if lower != upper {
                    return None;
                }
                allowed[basis] = lower;
            }
            sources.insert(
                source,
                [[allowed[0], cost[0] - allowed[0]], [allowed[1], cost[1] - allowed[1]]],
            );
        }
        Some(InvestmentRouting { forms, sources })
    }

    #[cfg(test)]
    pub(crate) fn sources_valid(&self, d: &crate::FederalDraft) -> bool {
        self.sources_issue(d).is_none()
    }
    pub(crate) fn sources_issue(
        &self,
        d: &crate::FederalDraft,
    ) -> Option<(usize, crate::validation::ValidationIssue)> {
        use crate::validation::ValidationIssue;
        let choices = sources(d);
        let entities = k1_sources(d);
        for (index, loan) in self.loans.iter().enumerate() {
            if !choices.iter().any(|choice| choice.id == loan.source) {
                return Some((
                    index,
                    ValidationIssue::new("INVESTMENT_BORROWING_RECORDS").at_field("source"),
                ));
            }
            if loan.originating_k1.is_some_and(|id| {
                !(loan.source == 0
                    && (loan.carry > 0 || loan.amt_carry > 0)
                    && entities.iter().any(|entity| entity.id == id))
            }) {
                return Some((
                    index,
                    ValidationIssue::new("INVESTMENT_BORROWING_RECORDS").at_field("originating_k1"),
                ));
            }
        }
        None
    }
}

// Transient projection of the existing Form 4952 result, not another ledger.
pub(crate) struct InvestmentRouting {
    pub forms: [[i64; 2]; 2],
    pub sources: BTreeMap<i64, [[i64; 2]; 2]>,
}
impl InvestmentRouting {
    pub(crate) fn portfolio(&self) -> [i64; 2] {
        self.sources.get(&0).map_or([0; 2], |v| [v[0][0], v[1][0]])
    }
    pub(crate) fn royalties(&self) -> [i64; 2] {
        core::array::from_fn(|basis| {
            self.sources.iter().filter(|(id, _)| **id != 0).map(|(_, v)| v[basis][0]).sum()
        })
    }
}
impl From<&InvestmentBorrowing> for InvestmentBorrowingPage {
    fn from(v: &InvestmentBorrowing) -> Self {
        Self {
            loans: v
                .loans
                .iter()
                .map(|l| InvestmentLoanPage {
                    label: l.label.clone(),
                    source: l.source,
                    paid: l.paid,
                    carry: l.carry,
                    amt_carry: l.amt_carry,
                    private_activity: l.private_activity,
                    paid_owners: l.paid_owners,
                    carry_owners: l.carry_owners,
                    allowed_origin_choices: vec![],
                    originating_k1: l
                        .originating_k1
                        .map(|source| InvestmentK1Origin::Entity { source }),
                })
                .collect(),
            election: v.election.clone(),
            income_use: v.income_use.clone(),
            limited_allocation: v.limited_allocation.clone(),
        }
    }
}
impl From<&InvestmentBorrowingPage> for InvestmentBorrowing {
    fn from(v: &InvestmentBorrowingPage) -> Self {
        Self {
            loans: v
                .loans
                .iter()
                .map(|l| InvestmentLoan {
                    label: l.label.clone(),
                    source: l.source,
                    paid: l.paid,
                    carry: l.carry,
                    amt_carry: l.amt_carry,
                    private_activity: l.private_activity,
                    paid_owners: l.paid_owners,
                    carry_owners: l.carry_owners,
                    originating_k1: l.originating_k1.as_ref().map(InvestmentK1Origin::id),
                })
                .collect(),
            election: v.election.clone(),
            income_use: v.income_use.clone(),
            limited_allocation: v.limited_allocation.clone(),
        }
    }
}
pub(crate) fn sources(d: &crate::FederalDraft) -> Vec<InvestmentSourceChoice> {
    let mut result = vec![InvestmentSourceChoice {
        id: 0,
        label: libertas::libertas_formatted_text("INVESTMENT_PORTFOLIO_SOURCE", &[]),
    }];
    result.extend(
        d.income
            .iter()
            .filter(|e| {
                matches!(
                    &e.income,
                    crate::FederalIncome::Royalty { data } if data.ordinary
                )
            })
            .map(|e| InvestmentSourceChoice {
                id: e.id,
                label: libertas::libertas_formatted_text(
                    "INVESTMENT_ROYALTY_SOURCE",
                    &[libertas::LibertasMessageArgument::Signed(e.id)],
                ),
            }),
    );
    result
}

impl InvestmentInterest {
    // Only return an attribution fixed by the source facts and the allowed
    // deduction. Do not invent an ordering of mixed expenses when limited.
    pub(crate) fn preferential_deduction(&self, allowed: i64) -> Option<i64> {
        use crate::kernel::ratio::dollars;
        if allowed == 0 {
            return Some(0);
        }
        let total = dollars(self.paid) + dollars(self.carryover);
        let preferential = match self.income_use {
            InvestmentInterestUse::Ordinary => 0,
            InvestmentInterestUse::Preferential => total,
            InvestmentInterestUse::Mixed { paid, carryover } => dollars(paid) + dollars(carryover),
            InvestmentInterestUse::Unresolved => return None,
        };
        let minimum = phase::floor(allowed - (total - preferential), 0);
        let maximum = phase::cap(allowed, [preferential]);
        (minimum == maximum).then_some(minimum)
    }

    /// Whole-dollar Forms 4952 with each system's basis-eligible current interest.
    pub(crate) fn calculate(
        &self,
        paid: [i64; 2],
        income: i64,
        expenses: i64,
        amt_income_adjustment: i64,
        amt_expenses: i64,
    ) -> [[i64; 2]; 2] {
        use crate::kernel::ratio::{dollars, positive};
        let regular = dollars(paid[0]) + dollars(self.carryover);
        let amt = dollars(paid[1]) + dollars(self.amt_carryover) + dollars(self.private_activity);
        let regular_allowed = phase::cap(regular, [positive(income - expenses)]);
        // Investment-property taxes are an AMT addback, not an AMT expense.
        // The income adjustment includes private bonds and separate AMT
        // rental-history effects; it may be negative.
        let amt_allowed =
            phase::cap(amt, [positive(income + amt_income_adjustment - amt_expenses)]);
        [(regular, regular_allowed), (amt, amt_allowed)].map(|(opening, used)| {
            let settlement = crate::kernel::carry::settle_values::<crate::kernel::carry::Dollars>(
                opening, used, false,
            )
            .expect("validated investment-interest pool and deduction");
            [settlement.used.get(), settlement.closing.get()]
        })
    }
}

impl InvestmentElection {
    pub(crate) fn amt_amounts(
        &self,
        elected: [i64; 2],
        capital: i64,
        qualified: i64,
    ) -> Option<[i64; 2]> {
        if matches!(self, Self::Allocated { .. }) {
            (elected[0] <= capital && elected[1] <= qualified).then_some(elected)
        } else {
            let total = phase::cap(elected.iter().sum::<i64>(), [capital + qualified]);
            let gain = phase::cap(total, [capital]);
            Some([gain, total - gain])
        }
    }
    pub(crate) fn amounts(&self, capital_income: i64, qualified: i64) -> Option<[i64; 2]> {
        use crate::kernel::ratio::dollars;
        let [capital, dividends] = match self {
            Self::None => [0, 0],
            Self::CapitalFirst { amount } => {
                let amount = dollars(*amount);
                let capital = phase::cap(amount, [capital_income]);
                [capital, amount - capital]
            }
            Self::Allocated { capital, dividends } => [dollars(*capital), dollars(*dividends)],
        };
        (capital <= capital_income && dividends <= qualified).then_some([capital, dividends])
    }
}
