// Additive Phase A owner. Existing source adapters and callers migrate in Phase B.
#![allow(dead_code)]

pub(crate) mod risk;
pub(crate) mod vintages;

use super::carry::{self, Amount, Cents};
use super::{phase, ratio, source_fact::SourceFact};
use alloc::vec::Vec;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LossLimitError {
    InvalidAmount,
    Overflow,
    Footing,
    AtRiskRecapture,
}

// The source owner establishes activity membership, earlier limits and
// recharacterization. Every amount in one PoolInput uses its explicit Unit.
// Keys retain actual source identity; their meaning is never inferred here.
pub(crate) struct PassiveActivity<Key> {
    pub key: Key,
    pub net: i64,
    pub rental_allowance: bool,
    pub ineligible_prior: i64,
}

pub(crate) struct PoolInput<'a, Key, Unit> {
    pub activities: &'a [PassiveActivity<Key>],
    pub other_passive_income: Amount<Unit>,
    pub allowance: Amount<Unit>,
}

pub(crate) struct ActivitySettlement<'a, Key> {
    pub key: &'a Key,
    pub net: i64,
    pub suspended: i64,
    pub allowed_net: i64,
}

// Original pre-held weights are retained for the existing annual projection.
// A settled net does not invent a split between current and prior releases.
pub(crate) struct PoolSettlement<'a, Key, Unit> {
    activities: &'a [PassiveActivity<Key>],
    suspended: Vec<i64>,
    remaining: Vec<i64>,
    losses: Vec<i64>,
    eligible: Vec<i64>,
    total_loss: Amount<Unit>,
    held_loss: Amount<Unit>,
    allowed_loss: Amount<Unit>,
    special_allowance: Amount<Unit>,
}

impl<Key, Unit> PoolSettlement<'_, Key, Unit> {
    pub(crate) fn suspended(&self) -> &[i64] {
        &self.suspended
    }

    pub(crate) fn total_loss(&self) -> i64 {
        self.total_loss.get()
    }

    pub(crate) fn allowed_loss(&self) -> i64 {
        self.allowed_loss.get()
    }

    pub(crate) fn held_loss(&self) -> i64 {
        self.held_loss.get()
    }

    pub(crate) fn special_allowance(&self) -> i64 {
        self.special_allowance.get()
    }

    pub(crate) fn settlements(&self) -> impl Iterator<Item = ActivitySettlement<'_, Key>> {
        self.activities.iter().zip(&self.suspended).map(|(activity, held)| ActivitySettlement {
            key: &activity.key,
            net: activity.net,
            suspended: *held,
            // Checked when the pool was settled.
            allowed_net: activity.net + held,
        })
    }
}

impl<Key> PoolSettlement<'_, Key, Cents> {
    pub(crate) fn annual_suspensions(&self) -> Result<Vec<i64>, LossLimitError> {
        // Preserve Form8582's existing annual recipe: special-allowance shares
        // precede common holds and use the original eligibility weights.
        // These annual rows are separate from raw held Cents. In particular,
        // zero annual weights retain the existing zero projection, not a new
        // residue or an admission refusal (100c loss / 50c special allowance).
        self.special_allowance.get().checked_add(50).ok_or(LossLimitError::Overflow)?;
        let annual_remaining;
        let weights = if self.special_allowance.get() == 0 {
            &self.remaining
        } else {
            let extras = ratio::allocate_remaining(
                ratio::dollars(self.special_allowance.get()),
                &self.eligible,
            );
            annual_remaining = self
                .losses
                .iter()
                .zip(extras)
                .map(|(loss, extra)| loss.checked_sub(extra.checked_mul(100)?))
                .collect::<Option<Vec<_>>>()
                .ok_or(LossLimitError::Overflow)?;
            &annual_remaining
        };
        weights
            .iter()
            .try_fold(0i64, |sum, value| sum.checked_add(*value))
            .ok_or(LossLimitError::Overflow)?;
        Ok(ratio::allocate_remaining(ratio::dollars(self.held_loss.get()), weights))
    }
}

// Form8582 Parts VI/VII after basis, at-risk and activity classification.
// Nonpassive, portfolio, separately limited PTP and disposition-release
// amounts stay with their source owners. Input order is the allocation order.
pub(crate) fn passive_pool<Key, Unit>(
    input: SourceFact<PoolInput<'_, Key, Unit>>,
) -> Result<SourceFact<PoolSettlement<'_, Key, Unit>>, LossLimitError> {
    let input = match input {
        SourceFact::Reported(value) => value,
        SourceFact::NotReported => return Ok(SourceFact::NotReported),
        SourceFact::Unavailable => return Ok(SourceFact::Unavailable),
    };
    let mut profit = input.other_passive_income.get();
    let mut total_loss = 0i64;
    let mut active_profit = 0i64;
    let mut active_loss = 0i64;
    let mut losses = Vec::new();
    let mut eligible = Vec::new();
    for activity in input.activities {
        if activity.ineligible_prior < 0 {
            return Err(LossLimitError::InvalidAmount);
        }
        let loss = phase::checked_loss(activity.net).ok_or(LossLimitError::Overflow)?;
        profit =
            profit.checked_add(ratio::positive(activity.net)).ok_or(LossLimitError::Overflow)?;
        total_loss = total_loss.checked_add(loss).ok_or(LossLimitError::Overflow)?;
        let allowance_net = if activity.rental_allowance {
            activity.net.checked_add(activity.ineligible_prior).ok_or(LossLimitError::Overflow)?
        } else {
            activity.net
        };
        let allowance_loss = phase::checked_loss(allowance_net).ok_or(LossLimitError::Overflow)?;
        if activity.rental_allowance {
            active_loss =
                active_loss.checked_add(allowance_loss).ok_or(LossLimitError::Overflow)?;
            active_profit = active_profit
                .checked_add(ratio::positive(allowance_net))
                .ok_or(LossLimitError::Overflow)?;
        }
        losses.push(loss);
        eligible.push(if activity.rental_allowance { allowance_loss } else { 0 });
    }
    let offset = phase::cap(profit, [total_loss]);
    let special = phase::cap(
        phase::cap(input.allowance.get(), [ratio::positive(active_loss - active_profit)]),
        [total_loss - offset],
    );
    let extras = ratio::allocate_remaining(special, &eligible);
    let remaining: Vec<_> = losses
        .iter()
        .zip(extras)
        .map(|(loss, extra)| loss.checked_sub(extra))
        .collect::<Option<_>>()
        .ok_or(LossLimitError::Overflow)?;
    let settlement = carry::settle_values::<Unit>(
        total_loss,
        offset.checked_add(special).ok_or(LossLimitError::Overflow)?,
        false,
    )
    .map_err(|_| LossLimitError::Footing)?;
    let suspended = ratio::allocate_remaining(settlement.closing.get(), &remaining);
    let held_total = suspended
        .iter()
        .try_fold(0i64, |sum, value| sum.checked_add(*value))
        .ok_or(LossLimitError::Overflow)?;
    if held_total != settlement.closing.get()
        || input.activities.iter().zip(&suspended).zip(&remaining).any(
            |((activity, held), available)| {
                *held < 0 || *held > *available || activity.net.checked_add(*held).is_none()
            },
        )
    {
        return Err(LossLimitError::Footing);
    }
    Ok(SourceFact::Reported(PoolSettlement {
        activities: input.activities,
        suspended,
        remaining,
        losses,
        eligible,
        total_loss: Amount::new(total_loss).map_err(|_| LossLimitError::InvalidAmount)?,
        held_loss: settlement.closing,
        allowed_loss: settlement.used,
        special_allowance: Amount::new(special).map_err(|_| LossLimitError::InvalidAmount)?,
    }))
}

pub(crate) struct CharacterInput<'a, Key, Unit> {
    pub nets: &'a [(Key, i64)],
    pub suspended: Amount<Unit>,
}

pub(crate) struct CharacterSettlement<'a, Key, Unit> {
    pub key: &'a Key,
    pub net: i64,
    held: Amount<Unit>,
}

impl<Key, Unit> CharacterSettlement<'_, Key, Unit> {
    pub(crate) fn held(&self) -> i64 {
        self.held.get()
    }

    pub(crate) fn allowed_net(&self) -> i64 {
        self.net + self.held.get()
    }
}

// Form8582 Part IX. Character nets already include their actual prior losses.
// Positive characters receive no hold; no current/prior release is inferred.
pub(crate) fn passive_characters<Key, Unit>(
    input: SourceFact<CharacterInput<'_, Key, Unit>>,
) -> Result<SourceFact<Vec<CharacterSettlement<'_, Key, Unit>>>, LossLimitError> {
    let input = match input {
        SourceFact::Reported(value) => value,
        SourceFact::NotReported => return Ok(SourceFact::NotReported),
        SourceFact::Unavailable => return Ok(SourceFact::Unavailable),
    };
    let net = input
        .nets
        .iter()
        .try_fold(0i64, |sum, (_, value)| sum.checked_add(*value))
        .ok_or(LossLimitError::Overflow)?;
    let maximum = phase::checked_loss(net).ok_or(LossLimitError::Overflow)?;
    if input.suspended.get() > maximum {
        return Err(LossLimitError::Footing);
    }
    let weights: Vec<_> = input
        .nets
        .iter()
        .map(|(_, value)| phase::checked_loss(*value))
        .collect::<Option<_>>()
        .ok_or(LossLimitError::Overflow)?;
    weights
        .iter()
        .try_fold(0i64, |sum, value| sum.checked_add(*value))
        .ok_or(LossLimitError::Overflow)?;
    let held = ratio::allocate_remaining(input.suspended.get(), &weights);
    if held
        .iter()
        .try_fold(0i64, |sum, value| sum.checked_add(*value))
        .ok_or(LossLimitError::Overflow)?
        != input.suspended.get()
    {
        return Err(LossLimitError::Footing);
    }
    input
        .nets
        .iter()
        .zip(held)
        .map(|((key, net), held)| {
            net.checked_add(held).ok_or(LossLimitError::Overflow)?;
            let opening = phase::checked_loss(*net).ok_or(LossLimitError::Overflow)?;
            let settlement = carry::settle_values::<Unit>(opening, opening - held, false)
                .map_err(|_| LossLimitError::Footing)?;
            Ok(CharacterSettlement { key, net: *net, held: settlement.closing })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(SourceFact::Reported)
}
