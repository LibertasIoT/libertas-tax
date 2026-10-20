// Coverage: Form 7203 — Supported (audited 2026-09-24).
//   Lines: Part I stock basis: opening, contributions, K-1 income/exempt income, distributions,
//   nondeductible expenses, losses-first election (Reg. 1.1367-1(g)), prior suspended losses;
//   outputs stock basis, distribution gain, allowed/suspended loss, nondeductible carry. Part II
//   debt basis per loan: notes vs open accounts ($25k threshold), reductions, restoration,
//   repayment gain (short/long, 8949/4797), independent AMT debt basis. Part III loss allocation by
//   character. Refused: inconsistent or unknown basis records.
//   Evidence: scorp_basis.rs, scorp.rs, scorp_risk.rs; tests/passthrough/scorp_stock_basis.rs
//   stock_losses_preserve_character_and_do_not_net_positive_income,
//   stock_allocation_conserves_every_dollar_at_rounding_boundaries,
//   debt_partial_repayment_needs_full_restoration_not_just_gain; tests/passthrough/passthrough.rs
//   scorp_unknown_basis_sources_do_not_publish_partial_basis_as_complete; sources i7203, f7203.pdf.
//   Unit T2-31 (2026-09-26): the line 6 distribution gain reaches Form 8949/Schedule D (scorp.rs
//   distribution_gain); box 12 code A is a Part III deduction class; a suspended gift is refused.
//   Unit B8 (2026-10-10): the Part III classes are the K1LossClass catalog (k1_loss.rs; LossAmounts
//   replaces the positional vectors of StockBasis, StockResult and BasisResult); the section 179,
//   gift, foreign tax and rental classes now carry what the basis holds back instead of being
//   refused, and BasisResult::release_history takes (class, vintages) pairs.

// Form 7203 stock-basis stage, before debt basis and subsequent loss limits.
// https://www.irs.gov/instructions/i7203
// https://www.irs.gov/pub/irs-pdf/f7203.pdf
// Amounts are actual source cents. Call separately for regular tax and AMT.
use crate::k1_loss::{K1LossClass, LossAmounts};
use crate::kernel::carry::{Amount, Cents, closing_values, native_remaining};
use crate::kernel::loss_limits::{risk, vintages};
use crate::kernel::phase;
use crate::kernel::source_fact::SourceFact;
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LoanKind {
    FormalNote,
    OpenAccount,
    // Previously crossed $25,000: no annual netting, but repayment gain stays
    // ordinary. Deemed note treatment does not create a written instrument.
    FormerOpenAccount,
}

pub(crate) struct ShareholderLoan {
    pub kind: LoanKind,
    pub principal: Amount<Cents>,
    pub basis: Amount<Cents>,
    pub advances: Amount<Cents>,
    pub repayments: Amount<Cents>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LoanResult {
    pub kind_next_year: LoanKind,
    pub principal: Amount<Cents>,
    pub basis: Amount<Cents>,
    pub restoration: Amount<Cents>,
    pub nontaxable_repayment: Amount<Cents>,
    pub capital_gain: Amount<Cents>,
    pub ordinary_gain: Amount<Cents>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BasisResult {
    pub stock: Amount<Cents>,
    pub distribution_gain: Amount<Cents>,
    pub loans: Vec<LoanResult>,
    // Still subject to at-risk, passive, and other shareholder limits.
    pub allowed: LossAmounts,
    pub suspended: LossAmounts,
    pub nondeductible_carry: Amount<Cents>,
    pub nondeductible_used: Amount<Cents>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BasisError {
    Invalid,
    Overflow,
    // The annual source model does not yet establish restoration priority
    // between multiple repaid debts when their combined deficits exceed income.
    RepaymentPriority,
    AtRiskRecapture,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AtRiskRecapture {
    pub income: Amount<Cents>,
    // Never offset this year's activity loss with the recapture income.
    pub next_year_deduction: Amount<Cents>,
}

// Roll forward a retained detailed-method Form 6198. Line 19b was calculated
// before that year's activity results. Prior line 21 net loss reduces next
// year's investment even when a later passive limit suspended its deduction.
// The current year's financing adjustments belong in Parts III lines 16/18.
pub(crate) fn prior_detailed_risk(
    capacity: Amount<Cents>,
    profit: Amount<Cents>,
    loss: Amount<Cents>,
) -> Result<Amount<Cents>, BasisError> {
    let SourceFact::Reported(opening) =
        risk::detailed_opening(SourceFact::Reported(risk::DetailedRisk::<Cents> {
            capacity,
            profit,
            net_loss: loss,
        }))
        .map_err(|_| BasisError::Invalid)?
    else {
        unreachable!("reported detailed history")
    };
    Ok(opening)
}

fn sum(mut values: impl Iterator<Item = i64>) -> Result<i64, BasisError> {
    values.try_fold(0i64, |a, b| a.checked_add(b)).ok_or(BasisError::Overflow)
}

// Form 8995 Loss Tracking Worksheet. Use a separate ledger for each limiting
// provision/activity/tax-character bucket. These are positive magnitudes.
// Original amounts preserve the fixed QBI percentage after partial releases.
pub(crate) struct LossVintage<Unit = Cents> {
    pub year: u32,
    pub original: Amount<Unit>,
    pub original_qbi: Amount<Unit>,
    pub remaining: Amount<Unit>,
    pub remaining_qbi: Amount<Unit>,
}
impl<Unit> Clone for LossVintage<Unit> {
    fn clone(&self) -> Self {
        Self {
            year: self.year,
            original: self.original,
            original_qbi: self.original_qbi,
            remaining: self.remaining,
            remaining_qbi: self.remaining_qbi,
        }
    }
}
impl<Unit> core::fmt::Debug for LossVintage<Unit> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LossVintage")
            .field("year", &self.year)
            .field("original", &self.original)
            .field("original_qbi", &self.original_qbi)
            .field("remaining", &self.remaining)
            .field("remaining_qbi", &self.remaining_qbi)
            .finish()
    }
}
impl<Unit> PartialEq for LossVintage<Unit> {
    fn eq(&self, other: &Self) -> bool {
        self.year == other.year
            && self.original == other.original
            && self.original_qbi == other.original_qbi
            && self.remaining == other.remaining
            && self.remaining_qbi == other.remaining_qbi
    }
}
impl<Unit> Eq for LossVintage<Unit> {}

pub(crate) struct LossRelease<Unit = Cents> {
    pub amount: Amount<Unit>,
    pub qbi: Amount<Unit>,
    pub remaining: Vec<LossVintage<Unit>>,
}
impl<Unit> core::fmt::Debug for LossRelease<Unit> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LossRelease")
            .field("amount", &self.amount)
            .field("qbi", &self.qbi)
            .field("remaining", &self.remaining)
            .finish()
    }
}
impl<Unit> PartialEq for LossRelease<Unit> {
    fn eq(&self, other: &Self) -> bool {
        self.amount == other.amount && self.qbi == other.qbi && self.remaining == other.remaining
    }
}
impl<Unit> Eq for LossRelease<Unit> {}

pub(crate) fn release_losses<Unit>(
    history: &[LossVintage<Unit>],
    amount: Amount<Unit>,
) -> Result<LossRelease<Unit>, BasisError> {
    let mut history = history.to_vec();
    history.sort_by_key(|v| v.year);
    if history.windows(2).any(|w| w[0].year == w[1].year)
        || history
            .iter()
            .any(|v| v.year == 0 || v.year > 2026 || (v.year < 2018 && v.original_qbi.get() != 0))
    {
        return Err(BasisError::Invalid);
    }
    // Tax-year admission and oldest-first order belong to this source; the
    // shared owner consumes its actual years and fixed qualified percentages.
    let vintages = history
        .iter()
        .map(|v| {
            Ok(vintages::LossVintage::<_, Unit> {
                key: v.year,
                original: v.original,
                original_qualified: v.original_qbi,
                remaining: v.remaining,
                remaining_qualified: v.remaining_qbi,
            })
        })
        .collect::<Result<Vec<_>, BasisError>>()?;
    let order = (0..vintages.len()).collect::<Vec<_>>();
    let SourceFact::Reported(release) =
        vintages::release_vintages(SourceFact::Reported(vintages::VintageInput {
            vintages: &vintages,
            amount,
            year: 2026,
            order: &order,
        }))
        .map_err(|_| BasisError::Invalid)?
    else {
        unreachable!("reported loss history")
    };
    Ok(LossRelease {
        amount: release.amount,
        qbi: release.qualified,
        remaining: release
            .remaining
            .into_iter()
            .map(|v| LossVintage {
                year: *v.key,
                original: v.original,
                original_qbi: v.original_qualified,
                remaining: v.remaining,
                remaining_qbi: v.remaining_qualified,
            })
            .collect(),
    })
}

impl BasisResult {
    pub(crate) fn release_history_item_part(
        &self,
        class: K1LossClass,
        years: &[LossVintage],
        other_allowed: Amount<Cents>,
    ) -> Result<LossRelease, BasisError> {
        if !(0..=self.allowed[class].get()).contains(&other_allowed.get()) {
            return Err(BasisError::Invalid);
        }
        let allowed = self.allowed[class].get() - other_allowed.get();
        let suspended = self.suspended[class].get();
        if sum(years.iter().map(|v| v.remaining.get()))?
            != allowed.checked_add(suspended).ok_or(BasisError::Invalid)?
        {
            return Err(BasisError::Invalid);
        }
        release_losses(years, Amount::new(allowed).map_err(|_| BasisError::Invalid)?)
    }
}

pub(crate) struct StockBasis<'a> {
    pub opening: Amount<Cents>,
    pub additions: Amount<Cents>,
    // Positive basis-increasing items, including eligible tax-exempt income.
    // Excludes S-corporation excluded COD and shareholder distribution gains.
    pub income: Amount<Cents>,
    pub distributions: Amount<Cents>,
    pub nondeductible: Amount<Cents>,
    // The section 50(c)(5) reduction of stock basis for an investment credit
    // (Form 7203 line 8c), not below zero and never carried.
    pub credit_reduction: Amount<Cents>,
    // Prior nondeductible carry exists only with the continuing 1.1367-1(g) election.
    pub prior_nondeductible: Amount<Cents>,
    pub losses_first: bool,
    // One entry per tax-character bucket, combining current and prior basis-
    // suspended amounts. Preserve the original records outside this stage for
    // QBI vintage allocation. Do not net positive income against losses here.
    pub losses: &'a LossAmounts,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct StockResult {
    pub distribution_gain: Amount<Cents>,
    // Preliminary stock basis; debt restoration and other stock adjustments
    // have NOT been deducted. Not an end-of-year Form 7203 line 15 result.
    pub remaining_stock: Amount<Cents>,
    pub stock_allowed: LossAmounts,
    // These amounts next enter the debt-basis stage. They are not yet a final
    // suspended loss or deductible loss on the shareholder's return.
    pub losses_to_debt: LossAmounts,
    pub nondeductible_to_debt: Amount<Cents>,
    // Income less all applicable reductions, never capital contributions.
    // The debt stage caps this potential restoration by reduced debt basis.
    pub net_increase: Amount<Cents>,
}

impl StockBasis<'_> {
    // Loans must be bona fide direct shareholder debt, not unpaid guarantees.
    // This handles annual Form 7203 adjustments, not forgiveness, transfers,
    // partial stock dispositions or debt with other adjusted-basis differences.
    pub(crate) fn with_loans(&self, loans: &[ShareholderLoan]) -> Result<BasisResult, BasisError> {
        if !self.losses_first && self.prior_nondeductible.get() != 0 {
            return Err(BasisError::Invalid);
        }
        let stock = self.calculate().ok_or(BasisError::Overflow)?;
        if loans.iter().filter(|l| l.kind == LoanKind::OpenAccount).count() > 1 {
            return Err(BasisError::Invalid);
        }
        let mut normalized = Vec::new();
        for loan in loans {
            if [loan.principal.get(), loan.basis.get(), loan.advances.get(), loan.repayments.get()]
                .iter()
                .any(|v| *v < 0)
                || loan.basis.get() > loan.principal.get()
                || (loan.kind == LoanKind::OpenAccount && loan.principal.get() > 2_500_000)
            {
                return Err(BasisError::Invalid);
            }
            let (advance, repaid) = if loan.kind == LoanKind::OpenAccount {
                (
                    native_remaining::<Cents>(loan.advances.get(), loan.repayments.get()).get(),
                    native_remaining::<Cents>(loan.repayments.get(), loan.advances.get()).get(),
                )
            } else {
                (loan.advances.get(), loan.repayments.get())
            };
            let principal =
                loan.principal.get().checked_add(advance).ok_or(BasisError::Overflow)?;
            if repaid > principal {
                return Err(BasisError::Invalid);
            }
            normalized.push((principal, loan.basis.get() + advance, repaid));
        }
        let deficits: Vec<_> = loans
            .iter()
            .map(|l| {
                closing_values::<Cents>(l.principal.get(), l.basis.get())
                    .expect("validated loan basis is capped by principal")
                    .get()
            })
            .collect();
        let available = phase::cap(stock.net_increase.get(), [sum(deficits.iter().copied())?]);
        let priority: Vec<_> = deficits
            .iter()
            .zip(&normalized)
            .map(|(deficit, (_, _, repaid))| if *repaid > 0 { *deficit } else { 0 })
            .collect();
        let priority_total = sum(priority.iter().copied())?;
        if available > 0
            && priority.iter().filter(|v| **v > 0).count() > 1
            && available < priority_total
        {
            return Err(BasisError::RepaymentPriority);
        }
        // A partial repayment also requires restoring the whole reduced basis
        // to eliminate gain: restoring merely the would-be gain is insufficient.
        let mut restored = crate::kernel::ratio::allocate_remaining(
            phase::cap(available, [priority_total]),
            &priority,
        );
        let residual: Vec<_> = deficits
            .iter()
            .zip(&restored)
            .map(|(d, r)| {
                closing_values::<Cents>(*d, *r).expect("restoration is capped by its deficit").get()
            })
            .collect();
        let remainder = crate::kernel::ratio::allocate_remaining(
            closing_values::<Cents>(available, phase::cap(available, [priority_total]))
                .expect("priority restoration is capped by availability")
                .get(),
            &residual,
        );
        for (r, extra) in restored.iter_mut().zip(remainder) {
            *r += extra;
        }
        let mut results = Vec::new();
        let mut bases = Vec::new();
        for ((loan, &(principal, basis, repaid)), &restoration) in
            loans.iter().zip(&normalized).zip(&restored)
        {
            let basis = basis + restoration;
            let nontaxable = if principal == 0 {
                0
            } else {
                crate::kernel::ratio::fraction(repaid, basis, principal)
            };
            bases.push(
                closing_values::<Cents>(basis, nontaxable)
                    .expect("proportional repayment is capped by debt basis")
                    .get(),
            );
            let gain = repaid - nontaxable;
            let principal = closing_values::<Cents>(principal, repaid)
                .expect("validated repayment is capped by principal")
                .get();
            let kind_next_year = if loan.kind == LoanKind::OpenAccount && principal > 2_500_000 {
                LoanKind::FormerOpenAccount
            } else {
                loan.kind
            };
            results.push(LoanResult {
                kind_next_year,
                principal: Amount::<Cents>::new(principal).expect("bounded basis result"),
                basis: Amount::<Cents>::new(0).expect("bounded basis result"),
                restoration: Amount::<Cents>::new(restoration).expect("bounded basis result"),
                nontaxable_repayment: Amount::<Cents>::new(nontaxable)
                    .expect("bounded basis result"),
                capital_gain: Amount::<Cents>::new(if loan.kind == LoanKind::FormalNote {
                    gain
                } else {
                    0
                })
                .expect("bounded basis result"),
                ordinary_gain: Amount::<Cents>::new(if loan.kind == LoanKind::FormalNote {
                    0
                } else {
                    gain
                })
                .expect("bounded basis result"),
            });
        }
        let total_basis = sum(bases.iter().copied())?;
        let total_loss = stock.losses_to_debt.total().get();
        let expenses = phase::cap(
            stock.nondeductible_to_debt.get(),
            [if self.losses_first {
                phase::floor(total_basis - total_loss, 0)
            } else {
                total_basis
            }],
        );
        let allowed = phase::cap(total_loss, [total_basis - expenses]);
        let debt_allowed =
            stock.losses_to_debt.allocate(Amount::new(allowed).expect("bounded allowance"));
        let reductions = crate::kernel::ratio::allocate_remaining(expenses + allowed, &bases);
        for ((result, basis), reduction) in results.iter_mut().zip(bases).zip(reductions) {
            result.basis = closing_values::<Cents>(basis, reduction)
                .expect("allocated reduction is capped by loan basis");
        }
        Ok(BasisResult {
            stock: Amount::<Cents>::new(
                closing_values::<Cents>(stock.remaining_stock.get(), available)
                    .map_err(|_| BasisError::Invalid)?
                    .get(),
            )
            .expect("bounded basis result"),
            distribution_gain: Amount::<Cents>::new(stock.distribution_gain.get())
                .expect("bounded basis result"),
            loans: results,
            allowed: stock.stock_allowed.plus(&debt_allowed),
            suspended: stock.losses_to_debt.minus(&debt_allowed),
            nondeductible_used: Amount::<Cents>::new(
                self.nondeductible.get() + self.prior_nondeductible.get()
                    - stock.nondeductible_to_debt.get()
                    + expenses,
            )
            .expect("bounded basis result"),
            nondeductible_carry: Amount::<Cents>::new(if self.losses_first {
                closing_values::<Cents>(stock.nondeductible_to_debt.get(), expenses)
                    .expect("debt expenses are capped by their source balance")
                    .get()
            } else {
                0
            })
            .expect("bounded basis result"),
        })
    }

    pub(crate) fn calculate(&self) -> Option<StockResult> {
        // Reject invalid/internal overflow rather than manufacturing zero tax.
        if [
            self.opening.get(),
            self.additions.get(),
            self.income.get(),
            self.distributions.get(),
            self.nondeductible.get(),
            self.credit_reduction.get(),
            self.prior_nondeductible.get(),
        ]
        .iter()
        .copied()
        .chain(self.losses.as_slice().iter().map(Amount::get))
        .any(|v| v < 0)
            || (!self.losses_first && self.prior_nondeductible.get() != 0)
        {
            return None;
        }
        let loss =
            self.losses.as_slice().iter().try_fold(0i64, |sum, v| sum.checked_add(v.get()))?;
        let expenses = self.nondeductible.get().checked_add(self.prior_nondeductible.get())?;
        let before =
            self.opening.get().checked_add(self.additions.get())?.checked_add(self.income.get())?;
        let distribution_gain = phase::floor(self.distributions.get() - before, 0);
        let mut remaining = native_remaining::<Cents>(before, self.distributions.get()).get();
        let credit_reduction = phase::cap(self.credit_reduction.get(), [remaining]);
        remaining = closing_values::<Cents>(remaining, credit_reduction).ok()?.get();
        let expenses_from_stock = if self.losses_first {
            phase::cap(expenses, [phase::floor(remaining - loss, 0)])
        } else {
            phase::cap(expenses, [remaining])
        };
        remaining = closing_values::<Cents>(remaining, expenses_from_stock).ok()?.get();
        let allowed = phase::cap(loss, [remaining]);
        let stock_allowed = self.losses.allocate(Amount::new(allowed).expect("bounded allowance"));
        let losses_to_debt = self.losses.minus(&stock_allowed);
        remaining = closing_values::<Cents>(remaining, allowed).ok()?.get();
        let reductions = self
            .distributions
            .get()
            .checked_add(credit_reduction)?
            .checked_add(expenses)?
            .checked_add(loss)?;
        Some(StockResult {
            distribution_gain: Amount::<Cents>::new(distribution_gain)
                .expect("bounded basis result"),
            remaining_stock: Amount::<Cents>::new(remaining).expect("bounded basis result"),
            stock_allowed,
            losses_to_debt,
            nondeductible_to_debt: closing_values::<Cents>(expenses, expenses_from_stock).ok()?,
            net_increase: native_remaining::<Cents>(self.income.get(), reductions),
        })
    }
}
