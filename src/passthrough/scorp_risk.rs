use crate::kernel::carry::{Amount, Cents};
// Coverage: Form 6198 — Partial (audited 2026-09-24).
//   Lines: Detailed-method at-risk limit (Parts I, III via retained 2025 line 19b/21, Part IV),
//   financing adjustments, recapture (capped, next-year deduction, 1978 threshold) for one
//   ordinary-business S-corp K-1 or partnership K-1 activity. Refused: Schedule C and rental
//   require "fully at risk = Yes" (no 6198 computed), unknown history with a loss (T2-31r), simplified method (Part II),
//   qualified nonrecourse financing of an S corporation, pledged property, multiple activities.
//   Unit k1-rentals (2026-10-09): a partner's share of qualified nonrecourse financing secured by
//   real property (K-1 item K1) counts in the at-risk debt (section 465(b)(6)); K-1 box 2 and 3
//   rental income and losses are items of the K-1's one at-risk activity (box 22/18 refused), and
//   a rental loss this stage limits carries (unit B8, ScorpRisk::calculate through k1_loss::limit_classes).
//   Evidence: scorp_risk.rs, scorp_basis.rs (prior_detailed_risk), k1_loss.rs (limit_classes), partnership.rs
//   (PartnerRiskRecord), extended_rules.rs, federal_model.rs;
//   kernel/loss_limits/risk_tests.rs::at_risk_irs_example_offsets_activity_income_before_limiting_net_loss,
//   kernel/loss_limits/risk_tests.rs::at_risk_recapture_is_capped_and_becomes_next_year_deduction,
//   passthrough.rs::scorp_at_risk_interview_applies_history_after_basis_and_survives_avro,
//   passthrough.rs::at_risk_history_rejects_impossible_or_overflowing_records; Form 6198
//   instructions example (test comment), i7203, i1065sk1
//   Unit T2-31 (2026-09-26): an S-corporation distribution above stock basis is line 2a activity
//   gain (no longer refused); the box 12 code A gift stays outside the activity.
//   Unit T2-31r (2026-09-27, question C9 for shareholders, as T2-29p for partners): with an
//   Unknown opening history the stage is skipped (ScorpRisk::without_history; investment and
//   year_end None, no amount-at-risk notes) when the year's losses entering it (box 1 loss,
//   nondeductible expenses, capital and section 1231 losses) do not exceed the activity's line 3
//   income, nothing is carried at risk from 2025 (ordinary, nondeductible, capital, section
//   1231), no deducted loss is left to recapture and box 11 is zero; otherwise HISTORY_NEEDED is
//   refused by name. A box 11 carryover without the history is refused in section179.rs. Source:
//   Instructions for Form 6198 (Purpose of Form: section 465 limits only the loss deducted; Who
//   Must File: an activity that incurred a loss; line 21). Tests: tests/passthrough/passthrough.rs
//   shareholder_profit_without_at_risk_history_needs_no_form_6198,
//   shareholder_loss_equal_to_activity_income_needs_no_history_and_one_dollar_more_does,
//   shareholder_at_risk_carryover_or_recapture_history_still_needs_form_6198; tests/passthrough/interview.rs
//   k1_1120s_page_without_at_risk_history_reproduces_the_profit_fixture.
//   Unit B8 (2026-10-10): Form 6198 Parts I and IV for the K-1's one activity share the income and the
//   amount at risk among every at-risk class of the K1LossClass catalog (ordinary, rentals, capital,
//   section 1231) and the nondeductible expenses, each with the basis-allowed amount plus its at-risk
//   carry (ScorpK1.carried.*.risk for the rentals); the allowed part is ScorpRiskResult.ledger
//   risk_allowed and the remaining portion of each item carries (risk_suspended; rule: i6198 Part
//   IV line 21). A New start cannot carry anything (ScorpRisk::valid). With an unknown history the
//   stage is skipped only when the basis-allowed losses of every at-risk class do not exceed the
//   income and no at-risk carry exists (HISTORY_NEEDED otherwise). Section 179 stays outside this
//   stage (ISSUE_SECTION179_RISK_CARRYFORWARD, narrowed reason).

use crate::k1_loss::{K1LossClass, LossLedger, limit_classes};
use crate::kernel::phase;
use crate::scorp_basis::{BasisResult, prior_detailed_risk};
use crate::{FederalResult, ScorpK1};
use libertas_macros::*;

/// Opening at-risk history
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    VariantIndex,
)]
pub enum ScorpRiskStart {
    /// First year of this activity, with no inherited or transferred history
    New,
    /// Retained detailed-method Form 6198 from 2025
    Detailed {
        /// 2025 Form 6198 line 19b
        /// Do not substitute line 10b or ending stock basis.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        capacity: i64,
        /// 2025 net profit from the activity
        /// Enter zero for a net-loss year; exclude prior recapture income.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        profit: i64,
        /// 2025 Form 6198 line 21 net loss
        /// Include nondeductible expenses absorbed by this limit; only the deductible portion enters the tax return. Enter the at-risk result even if passive limits later suspended deductible losses.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        loss: i64,
    },
    /// Missing history or another method requiring reconstruction
    Unknown,
}

impl ScorpRiskStart {
    pub(crate) fn continuing_choices() -> alloc::vec::Vec<i32> {
        alloc::vec![
            libertas_macros::variant_index!(ScorpRiskStart::Detailed) as i32,
            libertas_macros::variant_index!(ScorpRiskStart::Unknown) as i32,
        ]
    }
}

/// At-risk financing and retained loss records
/// For one ordinary-business activity within this corporation. This worksheet reuses contributions, distributions and direct loans already entered, together with section 1231 and portfolio interest/dividends and capital amounts confirmed as belonging to this at-risk activity. Do not enter those amounts again here. Passive, QBI and separate AMT records determine subsequent treatment.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ScorpRisk {
    /// Opening history
    pub start: ScorpRiskStart,
    /// New contributions and loan advances funded by amounts not at risk
    /// The portion financed with nonqualifying nonrecourse/related-party borrowing or protected against loss. Never count the same funds twice. Qualified nonrecourse real-estate financing and pledged outside-property exceptions need separate analysis.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub excluded_funding: i64,
    /// Previously at-risk investment newly protected during 2026
    /// Only amounts already included in the opening investment. Exclude casualty and tort insurance.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub protected: i64,
    /// Previously protected investment becoming at risk during 2026
    /// Only previously excluded investment; do not repeat current contributions or loan advances.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub released: i64,
    /// Withdrawals and principal repayments returning amounts already excluded from at-risk investment
    /// Included in the distributions and loan repayments entered above. Identify only the portion attributable to excluded investment; otherwise those withdrawals reduce the at-risk balance.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub excluded_returns: i64,
    /// Prior ordinary loss suspended by at-risk limits
    /// Exclude basis-suspended and passive-suspended losses; this amount already passed basis limits.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub prior_loss: i64,
    /// Prior nondeductible expenses suspended only by at-risk limits
    /// From the retained at-risk worksheet. These already reduced stock/debt basis. Never repeat a basis-only nondeductible carryforward here; do not include ordinary deductible losses.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub prior_nondeductible: i64,
    /// Losses deducted in previous tax years beginning after 1978
    /// Retained cumulative recapture history for this activity; do not include the current year.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub deducted: i64,
    /// At-risk recapture income included in previous years
    /// Used only to cap further recapture; do not enter a current-year calculated amount.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub recaptured: i64,
    /// These financing records are complete for the supported activity
    /// Confirm one continuing ordinary-business activity, with no property distributions/contributions, capital expenditures included as nondeductible expenses, outside collateral, qualified nonrecourse real-estate financing, pre-1979 negative investment, debt transfers, or other special adjustments. Entered tax-exempt income belongs to this activity and increases its at-risk amount. All capital contributions and loan advances are actual cash investments; excluded new funding is identified above. An earlier recapture requiring an unentered next-year deduction needs reconstruction. Entered nondeductible expenses are noncapital costs of this activity. Prior at-risk expense balances have already passed basis and do not overlap basis carryforwards. Other required K-1 worksheets must also be complete. No leaves this K-1 out of the estimate with a notice; that situation is not supported.
    pub complete: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ScorpRiskResult {
    // None (with year_end) when the at-risk stage limited nothing and the
    // opening history is unknown (question C9): the amount at risk is then
    // not determined.
    pub investment: Option<i64>,
    pub income: Amount<Cents>,
    // What the basis and at-risk limits allowed and held back of each class.
    pub ledger: LossLedger,
    pub nondeductible_used: Amount<Cents>,
    pub nondeductible_suspended: Amount<Cents>,
    pub year_end: Option<i64>,
    pub recapture: crate::scorp_basis::AtRiskRecapture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScorpRiskError {
    SourceInconsistent,
    FinancingUnresolved,
    DebtRepaymentAllocation,
    BasisRiskCoordination,
    DetailedHistoryInconsistent,
    LossInconsistent,
    RecaptureInconsistent,
    HistoryNeeded,
}

impl ScorpRiskError {
    pub(crate) fn message(self) -> (&'static str, &'static str) {
        match self {
            Self::SourceInconsistent => (
                "ISSUE_SCORP_RISK_SOURCE_INCONSISTENT",
                crate::fixture_english!("The basis and at-risk source records are inconsistent."),
            ),
            Self::FinancingUnresolved => (
                "ISSUE_SCORP_RISK_FINANCING_UNRESOLVED",
                crate::fixture_english!(
                    "Unknown or additional financing, gain or basis adjustments prevent the limited at-risk worksheet; retain the separate source records."
                ),
            ),
            Self::DebtRepaymentAllocation => (
                "ISSUE_SCORP_RISK_DEBT_REPAYMENT_ALLOCATION",
                crate::fixture_english!(
                    "Taxable debt repayments combined with returns of excluded investment need source-level allocation to avoid counting the same repayment twice."
                ),
            ),
            Self::BasisRiskCoordination => (
                "ISSUE_SCORP_RISK_BASIS_COORDINATION",
                crate::fixture_english!(
                    "Simultaneous basis and at-risk limits on deductible and nondeductible expenses need their coordinated allocation worksheet."
                ),
            ),
            Self::DetailedHistoryInconsistent => (
                "ISSUE_SCORP_RISK_DETAILED_HISTORY_INCONSISTENT",
                crate::fixture_english!("The retained detailed-method history is inconsistent."),
            ),
            Self::LossInconsistent => (
                "ISSUE_SCORP_RISK_LOSS_INCONSISTENT",
                crate::fixture_english!("The at-risk loss amounts are inconsistent."),
            ),
            Self::RecaptureInconsistent => (
                "ISSUE_SCORP_RISK_RECAPTURE_INCONSISTENT",
                crate::fixture_english!("The at-risk recapture history is inconsistent."),
            ),
            Self::HistoryNeeded => ("ISSUE_SCORP_RISK_HISTORY_NEEDED", ScorpRisk::HISTORY_NEEDED),
        }
    }
}

impl ScorpRisk {
    // Intrinsic recorded facts only. Incomplete records remain recordable;
    // financing ownership, New compatibility and allowance stay contextual.
    pub fn source_valid(&self) -> bool {
        // Arithmetic safety: activity_income/without_history and the ScorpSourceCapacity
        // context retain actual source cents through the loss classes.
        let valid = |v: i64| (0..=100000000000).contains(&v);
        let start = match self.start {
            ScorpRiskStart::Detailed { capacity, profit, loss } => {
                [capacity, profit, loss].into_iter().all(valid)
                    && loss <= capacity
                    && (profit == 0 || loss == 0)
            }
            _ => true,
        };
        start
            && [
                self.excluded_funding,
                self.excluded_returns,
                self.protected,
                self.released,
                self.prior_loss,
                self.prior_nondeductible,
                self.deducted,
                self.recaptured,
            ]
            .into_iter()
            .all(valid)
            && self.recaptured <= self.deducted
    }

    pub(crate) fn valid(&self, owner: &ScorpK1) -> bool {
        self.source_valid()
            && self.excluded_returns
                <= owner.distributions + owner.loans.iter().map(|l| l.repayments).sum::<i64>()
            && self.excluded_funding
                <= owner.additions + owner.loans.iter().map(|l| l.advances).sum::<i64>()
            && (!matches!(self.start, ScorpRiskStart::New)
                || (owner.opening == 0
                    && !owner.class_input().carries()
                    && owner.section1231.as_ref().is_none_or(|v| v.passive_loss == 0)
                    && owner.loans.iter().all(|l| l.principal == 0)
                    && self.prior_nondeductible == 0
                    && self.deducted == 0
                    && self.recaptured == 0
                    && self.protected == 0
                    && self.released == 0))
    }

    // Per-activity source adapter: these are actual attributed financing
    // parts, never stock/debt basis used as a substitute for at-risk history.
    pub(crate) fn component_investment(
        &self,
        parts: &crate::K1RiskInvestmentParts,
        exempt: i64,
        has_prior: bool,
    ) -> Result<i64, (&'static str, &'static str)> {
        let unavailable = (
            crate::k1_activity::ordinary::UNAVAILABLE.0.key,
            crate::k1_activity::ordinary::UNAVAILABLE.1,
        );
        let funding = parts.contributions.checked_add(parts.loan_advances).ok_or(unavailable)?;
        let returns = parts.distributions.checked_add(parts.loan_repayments).ok_or(unavailable)?;
        if !self.source_valid()
            || !self.complete
            || self.prior_nondeductible != 0
            || self.excluded_funding > funding
            || self.excluded_returns > returns
            || matches!(self.start, ScorpRiskStart::Unknown)
            || (matches!(self.start, ScorpRiskStart::New)
                && (has_prior
                    || self.deducted != 0
                    || self.recaptured != 0
                    || self.protected != 0
                    || self.released != 0))
        {
            return Err(unavailable);
        }
        self.investment_from_parts(
            parts.contributions,
            parts.distributions,
            parts.loan_advances,
            parts.loan_repayments,
        )
        .map_err(|_| unavailable)?
        .checked_add(exempt)
        .ok_or((
            crate::k1_activity::ordinary::RANGE_KEY,
            crate::fixture_english!(
                "Review the document amounts; they must be within the supported input range."
            ),
        ))
    }

    fn investment_from_parts(
        &self,
        additions: i64,
        distributions: i64,
        advances: i64,
        repayments: i64,
    ) -> Result<i64, ScorpRiskError> {
        let opening = match self.start {
            ScorpRiskStart::New => 0,
            ScorpRiskStart::Detailed { capacity, profit, loss } => prior_detailed_risk(
                Amount::<Cents>::new(capacity).expect("validated risk history"),
                Amount::<Cents>::new(profit).expect("validated risk history"),
                Amount::<Cents>::new(loss).expect("validated risk history"),
            )
            .map(|value| value.get())
            .map_err(|_| ScorpRiskError::DetailedHistoryInconsistent)?,
            ScorpRiskStart::Unknown => return Err(ScorpRiskError::HistoryNeeded),
        };
        Ok(opening + additions + advances + self.released
            - self.excluded_funding
            - self.protected
            - distributions
            - repayments
            + self.excluded_returns)
    }

    pub(crate) fn calculate(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
    ) -> Result<ScorpRiskResult, ScorpRiskError> {
        self.calculate_with_events(owner, basis, None)
    }

    pub(crate) fn calculate_with_events(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> Result<ScorpRiskResult, ScorpRiskError> {
        self.calculate_in(owner, basis, events, crate::scorp_amt::ScorpSourceCapacity::regular())
    }

    pub(crate) fn calculate_in(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
        events: Option<&crate::scorp::ScorpEvents>,
        capacity: crate::scorp_amt::ScorpSourceCapacity,
    ) -> Result<ScorpRiskResult, ScorpRiskError> {
        if !owner.valid_with_sale_carry(true, capacity) || !self.valid(owner) {
            return Err(ScorpRiskError::SourceInconsistent);
        }
        let input = owner.class_input();
        if basis.allowed.plus(&basis.suspended) != input.basis()
            || !basis.allowed.valid()
            || !basis.suspended.valid()
        {
            return Err(ScorpRiskError::SourceInconsistent);
        }
        if !self.complete || owner.prior_nondeductible != 0 {
            return Err(ScorpRiskError::FinancingUnresolved);
        }
        let repayment_gain: i64 = events.map_or_else(
            || {
                basis
                    .loans
                    .iter()
                    .map(|loan| loan.ordinary_gain.get() + loan.capital_gain.get())
                    .sum()
            },
            |value| value.loans.iter().sum(),
        );
        if repayment_gain > 0 && self.excluded_returns > 0 {
            return Err(ScorpRiskError::DebtRepaymentAllocation);
        }
        let expenses = owner.nondeductible;
        if (expenses > 0 || self.prior_nondeductible > 0)
            && (basis.suspended.any() || basis.nondeductible_used.get() != expenses)
        {
            return Err(ScorpRiskError::BasisRiskCoordination);
        }
        if matches!(self.start, ScorpRiskStart::Unknown) {
            return self.without_history(owner, basis, events);
        }
        let capacity = self.investment_from_parts(
            owner.additions, owner.distributions,
            owner.loans.iter().map(|l| l.advances).sum(),
            owner.loans.iter().map(|l| l.repayments).sum(),
        )?
            + owner.exempt
            + owner.exempt_interest
            // Repayment reduces at-risk investment only by adjusted debt
            // basis recovered. The gain itself does not increase stock basis
            // or constitute ordinary income of the corporation's activity.
            + repayment_gain;
        // Form 6198 line 3 includes separately stated income of this activity.
        // The source confirmation establishes this allocation. Tax character
        // stays separate when the allowed ordinary loss enters the return.
        // Line 2a: a distribution above stock basis is gain from disposing
        // of part of the interest in the activity; the full distribution
        // already reduced the investment above.
        let income = Self::activity_income(owner, basis, events);
        let nondeductible = expenses + self.prior_nondeductible;
        // Prior at-risk carries already passed basis; the box 2 and 3 rental
        // losses, like every other class, are losses of this one activity
        // (box 18 refuses separate at-risk activities).
        let stage = limit_classes(
            Amount::new(income).map_err(|_| ScorpRiskError::LossInconsistent)?,
            Amount::new(phase::floor(capacity, 0)).map_err(|_| ScorpRiskError::LossInconsistent)?,
            &basis.allowed,
            &input.risk_carry,
            Amount::new(nondeductible).map_err(|_| ScorpRiskError::LossInconsistent)?,
        )
        .map_err(|_| ScorpRiskError::LossInconsistent)?;
        let account = (|| {
            use crate::kernel::carry::{Amount, Cents};
            use crate::kernel::loss_limits::risk::{RecaptureHistory, close_economic_account};
            use crate::kernel::source_fact::SourceFact;
            let history = RecaptureHistory::<Cents> {
                prior_deducted: Amount::new(self.deducted)?,
                prior_recaptured: Amount::new(self.recaptured)?,
                threshold_1978: 0,
            };
            close_economic_account(
                capacity,
                income - stage.allowed_total.get(),
                SourceFact::Reported(history),
            )
            .map_err(|_| crate::kernel::carry::LedgerError::InvalidAmount)
        })()
        .map_err(|_| ScorpRiskError::RecaptureInconsistent)?;
        let year_end = account.ending;
        let crate::kernel::source_fact::SourceFact::Reported(recapture) = account.recapture else {
            unreachable!("reported recapture history")
        };
        let recapture = crate::scorp_basis::AtRiskRecapture {
            income: recapture.income,
            next_year_deduction: recapture.next_year_deduction,
        };
        Ok(ScorpRiskResult {
            investment: Some(capacity),
            income: Amount::new(income).map_err(|_| ScorpRiskError::LossInconsistent)?,
            ledger: LossLedger::of(basis.allowed, basis.suspended, &stage),
            nondeductible_used: stage.nondeductible_allowed,
            nondeductible_suspended: stage.nondeductible_suspended,
            year_end: Some(year_end),
            recapture,
        })
    }

    // Form 6198 line 3 income of this activity: box 1 income, the portfolio
    // income, capital and section 1231 gains confirmed as belonging to it,
    // and the line 2a distribution gain.
    fn activity_income(
        owner: &ScorpK1,
        basis: &BasisResult,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> i64 {
        phase::floor(owner.ordinary, 0)
            + owner.portfolio_income()
            + owner.capital_gains()
            + owner.section1231_gain()
            + owner.rentals.income()
            + owner.k1_items().income()
            + events.map_or_else(|| owner.disposal(basis), |value| value.disposal).activity_gain()
    }

    // Question C9 (unit T2-31r, as T2-29p for partners): Form 6198 is filed
    // only for an activity that incurred a loss, and the at-risk rules limit
    // only the loss deducted. A continuing shareholder without 2025 at-risk
    // figures needs none when the stage limits nothing: the year's losses
    // entering it (box 1 loss and basis-released carries, nondeductible
    // expenses, capital losses, the section 1231 loss) do not exceed the
    // activity's income, no loss or expense is carried at risk from 2025, no
    // box 11 section 179 amount waits for its own at-risk test, and no
    // earlier deducted loss is left to recapture. The amount at risk itself
    // stays unknown; any other case still needs the history.
    fn without_history(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
        events: Option<&crate::scorp::ScorpEvents>,
    ) -> Result<ScorpRiskResult, ScorpRiskError> {
        let income = Self::activity_income(owner, basis, events);
        let input = owner.class_input();
        let nondeductible = owner.nondeductible;
        let losses: i64 = K1LossClass::ALL
            .into_iter()
            .filter(|class| class.at_risk())
            .map(|class| basis.allowed[class].get())
            .sum();
        // A released section 179 basis carry is a Form 4562 line 6 item with
        // the current box 11 amount, tested against the remaining amount at
        // risk, which needs the history.
        if self.prior_nondeductible != 0
            || input.risk_carry.any()
            || self.deducted != self.recaptured
            || owner.section179 != 0
            || basis.allowed[K1LossClass::Section179].get() != 0
            || losses + nondeductible > income
        {
            return Err(ScorpRiskError::HistoryNeeded);
        }
        Ok(ScorpRiskResult {
            investment: None,
            income: Amount::new(income).map_err(|_| ScorpRiskError::LossInconsistent)?,
            ledger: LossLedger::without_risk_stage(basis.allowed, basis.suspended),
            nondeductible_used: Amount::new(nondeductible)
                .map_err(|_| ScorpRiskError::LossInconsistent)?,
            nondeductible_suspended: Amount::default(),
            year_end: None,
            recapture: crate::scorp_basis::AtRiskRecapture {
                income: Amount::default(),
                next_year_deduction: Amount::default(),
            },
        })
    }

    pub(crate) const HISTORY_NEEDED: &'static str = crate::fixture_english!(
        "Schedule K-1 (1120-S): reconstruct the opening at-risk history (Form 6198); stock and debt basis cannot substitute for it. Your 2025 Form 6198 figures are needed when this K-1 has a loss, a loss carried because of the at-risk limits, a box 11 section 179 deduction or earlier losses to recapture."
    );

    pub(crate) fn report(
        &self,
        owner: &ScorpK1,
        basis: &BasisResult,
        id: i64,
        result: &mut FederalResult,
    ) {
        let computed = match self.calculate(owner, basis) {
            Ok(v) => v,
            Err(error) => {
                let (key, _message) = error.message();
                #[cfg(test)]
                let message = _message;

                crate::issue_localized!(
                    result,
                    &crate::IssueContext::income_id(id),
                    "S-corporation at-risk",
                    key,
                    message,
                );
                return;
            }
        };
        let ScorpRiskResult {
            investment: capacity,
            income,
            ledger,
            nondeductible_used,
            nondeductible_suspended,
            year_end,
            recapture,
        } = computed;
        let allowed = ledger.risk_allowed[K1LossClass::Ordinary].get();
        let suspended = ledger.risk_suspended[K1LossClass::Ordinary].get();
        // Form 6198 line 21: every loss item of the activity that the stage
        // allowed.
        let activity_allowed: i64 = K1LossClass::ALL
            .into_iter()
            .filter(|class| class.at_risk())
            .map(|class| ledger.risk_allowed[class].get())
            .sum();
        if year_end.is_none() {
            result.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!("S-corporation document {id}: this K-1's losses do not exceed its activity income and nothing is carried at risk from 2025, so no Form 6198 is needed and your amount at risk is not figured."), crate::Message::new("NOTE_S_CORPORATION_DOCUMENT_ID_THIS_K_1_S_LOSSES_DO_NOT_EXCEE_2A4E36A2", alloc::vec![crate::MessageArg::Text(alloc::format!("{id}"))])));
        }
        for (_name, name_key, amount) in [
            (
                crate::fixture_english!("current activity income before recapture"),
                "SCORP_RISK_CURRENT_INCOME",
                Some(income.get()),
            ),
            (
                crate::fixture_english!("year-end at-risk investment before recapture"),
                "SCORP_RISK_YEAR_END_INVESTMENT",
                year_end,
            ),
            (
                crate::fixture_english!("investment before current income"),
                "SCORP_RISK_OPENING_INVESTMENT",
                capacity,
            ),
            (
                crate::fixture_english!("loss allowed through at-risk"),
                "SCORP_RISK_ALLOWED_LOSS",
                Some(allowed),
            ),
            (
                crate::fixture_english!("loss still suspended by at-risk"),
                "SCORP_RISK_SUSPENDED_LOSS",
                Some(suspended),
            ),
            (
                crate::fixture_english!(
                    "net loss absorbed by at-risk including nondeductible expenses"
                ),
                "SCORP_RISK_NET_LOSS_ABSORBED",
                Some(phase::floor(activity_allowed + nondeductible_used.get() - income.get(), 0)),
            ),
            (
                crate::fixture_english!("nondeductible expenses absorbed by at-risk investment"),
                "SCORP_RISK_NONDEDUCTIBLE_ABSORBED",
                Some(nondeductible_used.get()),
            ),
            (
                crate::fixture_english!("nondeductible expenses still suspended by at-risk"),
                "SCORP_RISK_NONDEDUCTIBLE_SUSPENDED",
                Some(nondeductible_suspended.get()),
            ),
            (
                crate::fixture_english!("recapture income"),
                "SCORP_RISK_RECAPTURE_INCOME",
                Some(recapture.income.get()),
            ),
            (
                crate::fixture_english!("recapture deduction for next year"),
                "SCORP_RISK_RECAPTURE_DEDUCTION",
                Some(recapture.next_year_deduction.get()),
            ),
        ] {
            #[cfg(test)]
            let name = _name;

            let Some(amount) = amount else { continue };
            result.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] alloc::format!("S-corporation document {id}, {name}: ${}. Amount after at-risk limits; passive, QBI, AMT and return-level rules apply separately.", crate::kernel::ratio::dollars(amount)),
                crate::Message::new(
                    "NOTE_SCORP_AT_RISK_AMOUNT",
                    alloc::vec![
                        crate::MessageArg::Number(id),
                        crate::MessageArg::Resource(name_key.into()),
                        crate::MessageArg::Currency {
                            currency_code: "USD".into(),
                            scale: 2,
                            minor_units: amount,
                        },
                    ],
                ),
            ));
        }
    }
}

// Community-property allocation of this source record; facts retain their meaning.
crate::community::divide_record!(ScorpRisk {
    amounts: [
        start,
        excluded_funding,
        protected,
        released,
        excluded_returns,
        prior_loss,
        prior_nondeductible,
        deducted,
        recaptured
    ],
    gains: [],
    facts: [complete],
});

// Allocate amounts; legal character, dates and retained facts are unchanged.
crate::community::divide_enum!(ScorpRiskStart {
    records: [Detailed { amounts: [capacity, profit, loss], facts: [] },],
    units: [New, Unknown],
    tuples: [],
});
