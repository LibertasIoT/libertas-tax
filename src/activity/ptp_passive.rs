// Coverage: assigned Unit19 PTP PAL source interface (2026-10-13).
//   Current/prior form components after basis/risk and activity classification,
//   separate regular/AMT inputs, same-PTP-only activity/form allocations and
//   checked Form4562 supplier consumption. Existing whole-return PTP179,
//   property source and unresolved179/risk ordering boundaries remain.
//   Evidence: tests/activity/passthrough.rs unit19_ptp_pal_*;
//   tests/cost_recovery/interview.rs unit17_ptp179_actual_two_rows_complete_save_reopen_restart.
// Native PTP source adapter, not a stored ledger or another PAL allocator.
// i8582 (2025), Special Instructions for PTPs: combine only this PTP's
// passive activities, then return the disallowance to each activity/form.
// Basis, at-risk, former-passive/disposition classification and Form4562
// must already be resolved by their existing owners. Actual source cents
// pass through this same owner until each reported endpoint.
pub use crate::k1_loss::K1LossClass as PtpPalCharacter;
use alloc::{collections::BTreeSet, vec::Vec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PtpPalComponent {
    pub character: PtpPalCharacter,
    pub passive_current_cents: i64,
    pub prior_passive_loss_cents: i64,
    // Already classified by the activity owner, including any actual former
    // passive release. This never provides income to another passive activity.
    pub nonpassive_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PtpPalActivity {
    pub k1: i64,
    // Actual accepted legal activity identity, not its display/list position.
    pub source: i64,
    pub components: Vec<PtpPalComponent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PtpPalComponentResult {
    pub source: i64,
    pub original: PtpPalComponent,
    pub allowed_passive_net_cents: i64,
    pub suspended_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PtpPalResult {
    pub k1: i64,
    pub alternative: bool,
    pub components: Vec<PtpPalComponentResult>,
    // Only the net gain of this PTP is nonpassive under section469(k).
    // Component form/tax character is retained separately above.
    pub net_gain_cents: i64,
    pub suspended_cents: i64,
    // The Form4562 owner retains this carry; it is never a prior PAL loss.
    pub business_income_carry_cents: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PtpPalIssue {
    InvalidSources,
    Section179SourceMismatch,
    AllocationUnavailable,
}

// Consume the merged179 supplier, including its checked before179 risk
// input. Its unresolved risk/order errors must reach this caller unchanged;
// this adapter never recalculates, clamps or assumes that input.
pub fn ptp_passive_source(
    k1: i64,
    alternative: bool,
    activities: &[PtpPalActivity],
    section179: &crate::Ptp179PalInput,
) -> Result<PtpPalResult, PtpPalIssue> {
    if k1 <= 0 || section179.k1 != k1 || section179.alternative != alternative {
        return Err(PtpPalIssue::Section179SourceMismatch);
    }
    let mut allowed = 0_i64;
    let mut available = 0_i64;
    for component in &section179.components {
        if component.allowed_cents < 0 || component.allowed_cents > component.available_cents {
            return Err(PtpPalIssue::Section179SourceMismatch);
        }
        allowed = allowed
            .checked_add(component.allowed_cents)
            .ok_or(PtpPalIssue::Section179SourceMismatch)?;
        available = available
            .checked_add(component.available_cents)
            .ok_or(PtpPalIssue::Section179SourceMismatch)?;
    }
    if allowed != section179.allowed_cents
        || available.checked_sub(allowed) != Some(section179.business_income_carry_cents)
    {
        return Err(PtpPalIssue::Section179SourceMismatch);
    }
    let mut section179_passive = 0_i64;
    let mut section179_nonpassive = 0_i64;
    for component in activities
        .iter()
        .flat_map(|activity| &activity.components)
        .filter(|component| component.character == PtpPalCharacter::Section179)
    {
        if component.passive_current_cents > 0 || component.nonpassive_cents > 0 {
            return Err(PtpPalIssue::Section179SourceMismatch);
        }
        section179_passive = section179_passive
            .checked_add(component.passive_current_cents)
            .ok_or(PtpPalIssue::Section179SourceMismatch)?;
        section179_nonpassive = section179_nonpassive
            .checked_add(component.nonpassive_cents)
            .ok_or(PtpPalIssue::Section179SourceMismatch)?;
    }
    let deduction = allowed;
    let expected = if section179.active { (0, -deduction) } else { (-deduction, 0) };
    if (section179_passive, section179_nonpassive) != expected {
        return Err(PtpPalIssue::Section179SourceMismatch);
    }
    let (components, net_gain_cents, suspended_cents) = allocate(k1, activities)?;
    Ok(PtpPalResult {
        k1,
        alternative,
        components,
        net_gain_cents,
        suspended_cents,
        business_income_carry_cents: section179.business_income_carry_cents,
    })
}

fn allocate(
    k1: i64,
    activities: &[PtpPalActivity],
) -> Result<(Vec<PtpPalComponentResult>, i64, i64), PtpPalIssue> {
    use crate::kernel::{
        carry::{Amount, Cents},
        loss_limits::{self, PassiveActivity},
        source_fact::SourceFact,
    };
    let mut sources = BTreeSet::new();
    let mut ordered: Vec<_> = activities.iter().collect();
    ordered.sort_by_key(|activity| activity.source);
    if activities.is_empty() || activities.len() > 500 {
        return Err(PtpPalIssue::InvalidSources);
    }
    let mut pool = Vec::new();
    let mut total = 0_i64;
    for activity in &ordered {
        let mut characters = BTreeSet::new();
        let mut net = 0_i64;
        if activity.k1 != k1
            || activity.source <= 0
            || !sources.insert(activity.source)
            || activity.components.is_empty()
        {
            return Err(PtpPalIssue::InvalidSources);
        }
        for component in &activity.components {
            if !characters.insert(component.character)
                || matches!(
                    component.character,
                    PtpPalCharacter::Charity
                        | PtpPalCharacter::OtherCharity
                        | PtpPalCharacter::ForeignTaxes
                        | PtpPalCharacter::InvestmentInterest
                )
                || component.passive_current_cents == i64::MIN
                || component.prior_passive_loss_cents < 0
                || component.nonpassive_cents == i64::MIN
            {
                return Err(PtpPalIssue::InvalidSources);
            }
            net = net
                .checked_add(component.passive_current_cents)
                .and_then(|value| value.checked_sub(component.prior_passive_loss_cents))
                .ok_or(PtpPalIssue::InvalidSources)?;
        }
        total = total.checked_add(net).ok_or(PtpPalIssue::InvalidSources)?;
        pool.push(PassiveActivity {
            key: activity.source,
            net,
            rental_allowance: false,
            ineligible_prior: 0,
        });
    }
    let SourceFact::Reported(settled) =
        loss_limits::passive_pool(SourceFact::Reported(loss_limits::PoolInput {
            activities: &pool,
            other_passive_income: Amount::<Cents>::new(0).expect("zero is nonnegative"),
            allowance: Amount::new(0).expect("zero is nonnegative"),
        }))
        .map_err(|_| PtpPalIssue::AllocationUnavailable)?
    else {
        unreachable!("reported pool remains reported")
    };
    let held = settled.suspended();
    let suspended = settled.held_loss();
    let mut result = Vec::new();
    for (activity, suspended) in ordered.into_iter().zip(held.iter().copied()) {
        let mut components: Vec<_> = activity.components.iter().collect();
        components.sort_by_key(|component| component.character);
        let nets: Vec<_> = components
            .iter()
            .map(|component| component.passive_current_cents - component.prior_passive_loss_cents)
            .collect();
        let keyed_nets: Vec<_> = components
            .iter()
            .zip(&nets)
            .map(|(component, net)| (component.character, *net))
            .collect();
        let SourceFact::Reported(held) =
            loss_limits::passive_characters(SourceFact::Reported(loss_limits::CharacterInput {
                nets: &keyed_nets,
                suspended: Amount::<Cents>::new(suspended)
                    .map_err(|_| PtpPalIssue::AllocationUnavailable)?,
            }))
            .map_err(|_| PtpPalIssue::AllocationUnavailable)?
        else {
            unreachable!("reported characters remain reported")
        };
        for ((component, net), held) in components.into_iter().zip(&nets).zip(held) {
            let suspended = held.held();
            result.push(PtpPalComponentResult {
                source: activity.source,
                original: component.clone(),
                allowed_passive_net_cents: *net + suspended,
                suspended_cents: suspended,
            });
        }
    }
    Ok((result, crate::kernel::phase::floor(total, 0), suspended))
}

// The source-cent single-activity route uses the same PAL owner. No179
// amount is introduced here: until the coordinator supplies checked179 source
// components, its existing named whole-return179 boundary remains in force.
pub(crate) fn single_activity_cents(
    source: i64,
    passive_net: i64,
) -> Result<(i64, i64), PtpPalIssue> {
    let (_, gain, held) = allocate(
        source,
        &[PtpPalActivity {
            k1: source,
            source,
            components: alloc::vec![PtpPalComponent {
                character: PtpPalCharacter::Ordinary,
                passive_current_cents: passive_net,
                prior_passive_loss_cents: 0,
                nonpassive_cents: 0,
            }],
        }],
    )?;
    Ok((gain, held))
}
