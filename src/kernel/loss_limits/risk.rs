use super::LossLimitError;
use crate::kernel::carry::{self, Amount};
use crate::kernel::{phase, ratio, source_fact::SourceFact};
use alloc::vec::Vec;

// The caller supplies ordinary, nondeductible, then other active classes in
// native allocation order. Prior risk amounts have already passed basis.
pub(crate) struct RiskEntry<Key, Unit> {
    pub key: Key,
    pub basis_allowed: Amount<Unit>,
    pub prior_risk: Amount<Unit>,
}

pub(crate) struct RiskInput<'a, Key, Unit> {
    pub entries: &'a [RiskEntry<Key, Unit>],
    pub bypass: &'a [(Key, Amount<Unit>)],
    pub income: Amount<Unit>,
    // Same native Unit, excluding current activity income and deductions.
    // A negative limit operand needs recapture analysis, not an implicit floor.
    pub capacity: i64,
}

pub(crate) struct RiskClassSettlement<'a, Key, Unit> {
    pub key: &'a Key,
    pub opening: Amount<Unit>,
    pub allowed: Amount<Unit>,
    pub suspended: Amount<Unit>,
}

pub(crate) struct RiskSettlement<'a, Key, Unit> {
    pub classes: Vec<RiskClassSettlement<'a, Key, Unit>>,
    pub bypass: &'a [(Key, Amount<Unit>)],
    pub gross_allowed: Amount<Unit>,
    pub net_loss: Amount<Unit>,
    income: i64,
    capacity: i64,
}

pub(crate) fn settle_at_risk<Key, Unit>(
    input: SourceFact<RiskInput<'_, Key, Unit>>,
) -> Result<SourceFact<RiskSettlement<'_, Key, Unit>>, LossLimitError> {
    let input = match input {
        SourceFact::Reported(value) => value,
        SourceFact::NotReported => return Ok(SourceFact::NotReported),
        SourceFact::Unavailable => return Ok(SourceFact::Unavailable),
    };
    if input.capacity < 0 {
        return Err(LossLimitError::AtRiskRecapture);
    }
    let openings = input
        .entries
        .iter()
        .map(|entry| entry.basis_allowed.get().checked_add(entry.prior_risk.get()))
        .collect::<Option<Vec<_>>>()
        .ok_or(LossLimitError::Overflow)?;
    let total = openings
        .iter()
        .try_fold(0i64, |sum, amount| sum.checked_add(*amount))
        .ok_or(LossLimitError::Overflow)?;
    let available =
        input.income.get().checked_add(input.capacity).ok_or(LossLimitError::Overflow)?;
    let allowance = phase::cap(available, [total]);
    let allocated = ratio::allocate_remaining(allowance, &openings);
    let gross = allocated
        .iter()
        .try_fold(0i64, |sum, amount| sum.checked_add(*amount))
        .ok_or(LossLimitError::Overflow)?;
    if gross != allowance {
        return Err(LossLimitError::Footing);
    }
    let classes = input
        .entries
        .iter()
        .zip(openings)
        .zip(allocated)
        .map(|((entry, opening), allowed)| {
            let settled = carry::settle_values::<Unit>(opening, allowed, false)
                .map_err(|_| LossLimitError::Footing)?;
            Ok(RiskClassSettlement {
                key: &entry.key,
                opening: Amount::new(opening).map_err(|_| LossLimitError::InvalidAmount)?,
                allowed: settled.used,
                suspended: settled.closing,
            })
        })
        .collect::<Result<Vec<_>, LossLimitError>>()?;
    Ok(SourceFact::Reported(RiskSettlement {
        classes,
        bypass: input.bypass,
        gross_allowed: Amount::new(gross).map_err(|_| LossLimitError::InvalidAmount)?,
        net_loss: Amount::new(ratio::positive(gross - input.income.get()))
            .map_err(|_| LossLimitError::InvalidAmount)?,
        income: input.income.get(),
        capacity: input.capacity,
    }))
}

// Signed form components keep the original disallowed-first residual order.
// Keys and the common native Unit come from the source owner; capacity excludes
// the current components. A negative investment adds no current loss capacity.
pub(crate) struct SignedRiskSettlement<'a, Key, Unit> {
    pub components: Vec<super::CharacterSettlement<'a, Key, Unit>>,
    pub net: i64,
    pub allowed_net: i64,
}

pub(crate) fn settle_signed_components<Key, Unit>(
    components: SourceFact<&[(Key, i64)]>,
    capacity: i64,
) -> Result<SourceFact<SignedRiskSettlement<'_, Key, Unit>>, LossLimitError> {
    let components = match components {
        SourceFact::Reported(value) => value,
        SourceFact::NotReported => return Ok(SourceFact::NotReported),
        SourceFact::Unavailable => return Ok(SourceFact::Unavailable),
    };
    let net = ratio::checked_sum(components.iter().map(|(_, value)| *value))
        .ok_or(LossLimitError::Overflow)?;
    let loss = phase::checked_loss(net).ok_or(LossLimitError::Overflow)?;
    let held = carry::closing_values::<Unit>(loss, phase::cap(loss, [ratio::positive(capacity)]))
        .map_err(|_| LossLimitError::Footing)?;
    match super::passive_characters(SourceFact::Reported(super::CharacterInput {
        nets: components,
        suspended: held,
    }))? {
        SourceFact::Reported(components) => {
            let allowed_net = ratio::checked_sum(components.iter().map(|part| part.allowed_net()))
                .ok_or(LossLimitError::Overflow)?;
            Ok(SourceFact::Reported(SignedRiskSettlement { components, net, allowed_net }))
        }
        SourceFact::NotReported => Ok(SourceFact::NotReported),
        SourceFact::Unavailable => Ok(SourceFact::Unavailable),
    }
}

pub(crate) struct DetailedRisk<Unit> {
    pub capacity: Amount<Unit>,
    pub profit: Amount<Unit>,
    pub net_loss: Amount<Unit>,
}

// The prior net loss reduces investment even if a later passive limit held it.
// Current financing adjustments and unknown-history bypass stay source-owned.
pub(crate) fn detailed_opening<Unit>(
    prior: SourceFact<DetailedRisk<Unit>>,
) -> Result<SourceFact<Amount<Unit>>, LossLimitError> {
    let prior = match prior {
        SourceFact::Reported(value) => value,
        SourceFact::NotReported => return Ok(SourceFact::NotReported),
        SourceFact::Unavailable => return Ok(SourceFact::Unavailable),
    };
    if prior.net_loss.get() > prior.capacity.get()
        || (prior.profit.get() > 0 && prior.net_loss.get() > 0)
    {
        return Err(LossLimitError::InvalidAmount);
    }
    let opening =
        prior.capacity.get().checked_add(prior.profit.get()).ok_or(LossLimitError::Overflow)?;
    Ok(SourceFact::Reported(
        carry::closing_values::<Unit>(opening, prior.net_loss.get())
            .map_err(|_| LossLimitError::InvalidAmount)?,
    ))
}

pub(crate) struct RecaptureHistory<Unit> {
    pub prior_deducted: Amount<Unit>,
    pub prior_recaptured: Amount<Unit>,
    // Same native Unit: zero normally, or the actual nonpositive 1978 balance.
    pub threshold_1978: i64,
}

pub(crate) struct RiskRecapture<Unit> {
    pub income: Amount<Unit>,
    pub next_year_deduction: Amount<Unit>,
}

pub(crate) struct RiskAccount<Unit> {
    // Signed native Unit; a known closing balance does not supply missing history.
    pub ending: i64,
    pub recapture: SourceFact<RiskRecapture<Unit>>,
}

impl<Key, Unit> RiskSettlement<'_, Key, Unit> {
    pub(crate) fn close_account(
        &self,
        signed_investment: i64,
        history: SourceFact<RecaptureHistory<Unit>>,
    ) -> Result<RiskAccount<Unit>, LossLimitError> {
        // Source adapters retain actual signed investment separately while
        // supplying its nonnegative part to the loss stage, as before.
        if self.capacity != ratio::positive(signed_investment) {
            return Err(LossLimitError::Footing);
        }
        let allowed_part_one_net =
            self.income.checked_sub(self.gross_allowed.get()).ok_or(LossLimitError::Overflow)?;
        close_economic_account(signed_investment, allowed_part_one_net, history)
    }
}

// The selected loss capacity need not equal the signed economic investment
// (for example, a source's Compare method). Close the genuine investment with
// the already allowed Part I net; this never reruns or changes that allowance.
pub(crate) fn close_economic_account<Unit>(
    signed_investment: i64,
    allowed_part_one_net: i64,
    history: SourceFact<RecaptureHistory<Unit>>,
) -> Result<RiskAccount<Unit>, LossLimitError> {
    let ending =
        signed_investment.checked_add(allowed_part_one_net).ok_or(LossLimitError::Overflow)?;
    Ok(RiskAccount { ending, recapture: recapture_from_ending(ending, history)? })
}

// A genuinely known ending balance can be recaptured without inventing an
// investment or current activity result. History validation precedes reduction.
pub(crate) fn recapture_from_ending<Unit>(
    ending: i64,
    history: SourceFact<RecaptureHistory<Unit>>,
) -> Result<SourceFact<RiskRecapture<Unit>>, LossLimitError> {
    let history = match history {
        SourceFact::Reported(value) => value,
        SourceFact::NotReported => {
            return Ok(SourceFact::NotReported);
        }
        SourceFact::Unavailable => {
            return Ok(SourceFact::Unavailable);
        }
    };
    if history.prior_recaptured.get() > history.prior_deducted.get() || history.threshold_1978 > 0 {
        return Err(LossLimitError::InvalidAmount);
    }
    let reduction = if ending < history.threshold_1978 {
        history.threshold_1978.checked_sub(ending).ok_or(LossLimitError::Overflow)?
    } else {
        0
    };
    let remaining =
        carry::closing_values::<Unit>(history.prior_deducted.get(), history.prior_recaptured.get())
            .map_err(|_| LossLimitError::InvalidAmount)?;
    let income = phase::cap(reduction, [remaining.get()]);
    // Recapture is not added to this year's activity allowance.
    Ok(SourceFact::Reported(RiskRecapture {
        income: Amount::new(income).map_err(|_| LossLimitError::InvalidAmount)?,
        next_year_deduction: Amount::new(income).map_err(|_| LossLimitError::InvalidAmount)?,
    }))
}

#[cfg(test)]
#[path = "risk_tests.rs"]
mod tests;
