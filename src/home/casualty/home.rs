// HOME's casualty source interface, in cents. The casualty owner retains
// character; Form8829 and its ScheduleC/F caller own the remaining tiers.
// Dispatcher eac268d: current casualty, current depreciation, then prior
// carry; within characterized prior carry casualty precedes depreciation.
// The latter is an approved inference: the printed form carries one total.
use crate::kernel::carry::{self, Amount, Cents, LedgerError};
use crate::kernel::phase;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PropertyLoss {
    pub limited_cents: i64,
    pub loss_cents: i64,
    pub continuing_basis_cents: i64,
}

// Section B lines20-27: the source owner supplies the event-date adjusted
// basis and expected reimbursement. None denotes business property totally
// destroyed or stolen; damaged property supplies its actual FMV decrease.
// Section A always supplies the decrease, even for destroyed personal property.
// Restoration costs remain with the asset/improvement owner, separately.
pub(crate) fn property_loss(
    adjusted_basis_cents: i64,
    fair_value_decrease_cents: Option<i64>,
    reimbursement_cents: i64,
) -> Result<PropertyLoss, LedgerError> {
    Amount::<Cents>::new(adjusted_basis_cents)?;
    Amount::<Cents>::new(reimbursement_cents)?;
    let decrease = fair_value_decrease_cents.unwrap_or(adjusted_basis_cents);
    Amount::<Cents>::new(decrease)?;
    let limited_cents = phase::cap(adjusted_basis_cents, [decrease]);
    let loss_cents = carry::native_remaining::<Cents>(limited_cents, reimbursement_cents).get();
    let continuing_basis_cents =
        carry::native_remaining::<Cents>(adjusted_basis_cents, reimbursement_cents + loss_cents)
            .get();
    Ok(PropertyLoss { limited_cents, loss_cents, continuing_basis_cents })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ThirdTierCarry {
    pub total_cents: i64,
    // None retains a known combined balance whose character is unavailable.
    // It never supplies a zero casualty or depreciation component.
    pub casualty_cents: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ThirdTierResult {
    pub otherwise_allowed_casualty_cents: i64,
    pub excess_casualty_cents: i64,
    pub allowed_excess_casualty_cents: i64,
    pub allowed_current_depreciation_cents: i64,
    pub allowed_prior_casualty_cents: Option<i64>,
    pub allowed_prior_depreciation_cents: Option<i64>,
    pub current_casualty_carry_cents: i64,
    pub current_depreciation_carry_cents: i64,
    pub prior_carry_remaining_cents: i64,
    pub total_allowed_cents: i64,
    pub carry: ThirdTierCarry,
}

impl ThirdTierResult {
    // Form8829line35 requires the actual casualty character, not an unknown
    // prior carry relabeled depreciation. Current known amounts stay known.
    pub(crate) fn allowed_casualty_cents(&self) -> Option<i64> {
        Some(
            self.otherwise_allowed_casualty_cents
                + self.allowed_excess_casualty_cents
                + self.allowed_prior_casualty_cents?,
        )
    }
    pub(crate) fn allowed_depreciation_cents(&self) -> Option<i64> {
        Some(self.allowed_current_depreciation_cents + self.allowed_prior_depreciation_cents?)
    }
}

pub(crate) fn third_tier(
    otherwise_allowed_casualty_cents: i64,
    excess_casualty_cents: i64,
    current_depreciation_cents: i64,
    prior: ThirdTierCarry,
    remaining_income_cents: i64,
) -> Result<ThirdTierResult, LedgerError> {
    Amount::<Cents>::new(otherwise_allowed_casualty_cents)?;
    if prior.casualty_cents.is_some_and(|amount| amount < 0 || amount > prior.total_cents) {
        return Err(LedgerError::InvalidAmount);
    }
    let mut room = Amount::<Cents>::new(remaining_income_cents)?;
    let casualty = carry::consume_step(Amount::new(excess_casualty_cents)?, &mut room, false)?;
    let depreciation =
        carry::consume_step(Amount::new(current_depreciation_cents)?, &mut room, false)?;
    let previous = carry::consume_step(Amount::new(prior.total_cents)?, &mut room, false)?;
    let (allowed_prior_casualty_cents, allowed_prior_depreciation_cents, prior_casualty_left) =
        if let Some(casualty_cents) = prior.casualty_cents {
            let mut prior_room = Amount::<Cents>::new(previous.used.get())?;
            let casualty =
                carry::consume_step(Amount::new(casualty_cents)?, &mut prior_room, false)?;
            let depreciation = carry::consume_step(
                Amount::new(prior.total_cents - casualty_cents)?,
                &mut prior_room,
                false,
            )?;
            (Some(casualty.used.get()), Some(depreciation.used.get()), Some(casualty.closing.get()))
        } else {
            let no_use = (previous.used.get() == 0).then_some(0);
            (no_use, no_use, (previous.closing.get() == 0).then_some(0))
        };
    let result = ThirdTierResult {
        otherwise_allowed_casualty_cents,
        excess_casualty_cents,
        allowed_excess_casualty_cents: casualty.used.get(),
        allowed_current_depreciation_cents: depreciation.used.get(),
        allowed_prior_casualty_cents,
        allowed_prior_depreciation_cents,
        current_casualty_carry_cents: casualty.closing.get(),
        current_depreciation_carry_cents: depreciation.closing.get(),
        prior_carry_remaining_cents: previous.closing.get(),
        total_allowed_cents: casualty.used.get() + depreciation.used.get() + previous.used.get(),
        carry: ThirdTierCarry {
            total_cents: casualty.closing.get()
                + depreciation.closing.get()
                + previous.closing.get(),
            casualty_cents: prior_casualty_left.map(|amount| amount + casualty.closing.get()),
        },
    };
    // Check the typed destinations as well as the aggregate ledger. An
    // unavailable character remains unavailable, never a reconciliation zero.
    if result.excess_casualty_cents
        != result.allowed_excess_casualty_cents + result.current_casualty_carry_cents
        || current_depreciation_cents
            != result.allowed_current_depreciation_cents + result.current_depreciation_carry_cents
        || result.carry.total_cents
            != result.current_casualty_carry_cents
                + result.current_depreciation_carry_cents
                + result.prior_carry_remaining_cents
        || result.allowed_casualty_cents().zip(result.allowed_depreciation_cents()).is_some_and(
            |(casualty, depreciation)| {
                casualty + depreciation
                    != otherwise_allowed_casualty_cents + result.total_allowed_cents
            },
        )
    {
        return Err(LedgerError::InvalidAmount);
    }
    Ok(result)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ConversionResult {
    pub excluded_cents: i64,
    pub proceeds_after_exclusion_cents: i64,
    pub gain_after_exclusion_cents: i64,
    pub recognized_cents: i64,
    pub postponed_cents: i64,
    pub replacement_basis_cents: Option<i64>,
}

// The HOME owner establishes §121 eligibility and supplies its actual
// exclusion and eligible completed/planned replacement cost. None means no
// postponement election, not an unentered cost. This owner consumes §121
// before the §1033 replacement test; it never recalculates the exclusion.
pub(crate) fn conversion(
    proceeds_cents: i64,
    basis_cents: i64,
    excluded_cents: i64,
    replacement_cost_cents: Option<i64>,
) -> Result<ConversionResult, LedgerError> {
    let realized_gain = phase::floor(proceeds_cents - basis_cents, 0);
    if !(0..=realized_gain).contains(&excluded_cents)
        || replacement_cost_cents.is_some_and(|amount| amount < 0)
    {
        return Err(LedgerError::InvalidAmount);
    }
    let proceeds_after_exclusion_cents = proceeds_cents - excluded_cents;
    let gain_after_exclusion_cents = realized_gain - excluded_cents;
    let recognized_cents = replacement_cost_cents.map_or(gain_after_exclusion_cents, |cost| {
        phase::cap(
            gain_after_exclusion_cents,
            [phase::floor(proceeds_after_exclusion_cents - cost, 0)],
        )
    });
    let postponed_cents =
        carry::settle_values::<Cents>(gain_after_exclusion_cents, recognized_cents, false)?
            .closing
            .get();
    let result = ConversionResult {
        excluded_cents,
        proceeds_after_exclusion_cents,
        gain_after_exclusion_cents,
        recognized_cents,
        postponed_cents,
        replacement_basis_cents: replacement_cost_cents.map(|cost| cost - postponed_cents),
    };
    if result.excluded_cents + result.gain_after_exclusion_cents != realized_gain
        || result.recognized_cents + result.postponed_cents != result.gain_after_exclusion_cents
        || result.proceeds_after_exclusion_cents + result.excluded_cents != proceeds_cents
        || result.replacement_basis_cents.is_some_and(|basis| basis < 0)
    {
        return Err(LedgerError::InvalidAmount);
    }
    Ok(result)
}
