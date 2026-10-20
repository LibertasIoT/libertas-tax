// Additive Phase A API; consumers migrate only after the dispatcher starts Phase B.
#![allow(dead_code)]

// Canonical person relationship for an expense or a retained individual tax
// attribute. The originating entity and the deduction destination do not
// establish the payer or the individual retaining an earlier attribute.
use libertas_macros::*;

pub(crate) mod counterpart;

/// People who own this expense or retained deduction
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum DeductionOwners {
    /// Taxpayer alone
    Taxpayer,
    /// Spouse alone
    Spouse,
    /// Both spouses
    /// The actual records establish that each spouse owns part. This is not an answer for an unknown owner.
    Both,
}

// Actual nonnegative expense source Cents, before their native annual rounding.
// Both supplies established joint capacities; it never supplies a ratio.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ExpenseSourceCents {
    exclusive: [i64; 2],
    both: i64,
    unknown: i64,
    valid: bool,
}
impl ExpenseSourceCents {
    pub(crate) fn known(exclusive: [i64; 2]) -> Self {
        Self { exclusive, both: 0, unknown: 0, valid: exclusive.iter().all(|amount| *amount >= 0) }
    }
    pub(crate) fn add(&mut self, cents: i64, owners: Option<DeductionOwners>) {
        if !self.valid || cents < 0 {
            self.valid = false;
            return;
        }
        let target = match owners {
            Some(DeductionOwners::Taxpayer) => &mut self.exclusive[0],
            Some(DeductionOwners::Spouse) => &mut self.exclusive[1],
            Some(DeductionOwners::Both) => &mut self.both,
            None => &mut self.unknown,
        };
        if let Some(amount) = target.checked_add(cents) {
            *target = amount;
        } else {
            self.valid = false;
        }
    }
    // Maximum annual-Dollar source capacities at the existing category's
    // rounding boundary. Exclusive-only sources use the shared exact-footing
    // projection; a disagreement does not fabricate a residual owner.
    pub(crate) fn annual_capacities(&self, first: usize) -> Option<[i64; 2]> {
        if !self.valid || first > 1 {
            return None;
        }
        let cents = self.exclusive[0]
            .checked_add(self.exclusive[1])?
            .checked_add(self.both)?
            .checked_add(self.unknown)?;
        // The existing Dollar kernel adds50 before dividing. Reject an
        // unrepresentable input instead of overflowing or inventing zero.
        cents.checked_add(50)?;
        let total = crate::kernel::ratio::dollars(cents);
        if total == 0 {
            return Some([0; 2]);
        }
        if self.unknown != 0 {
            return None;
        }
        if self.both == 0 {
            return crate::kernel::ratio::project_rounded_pair(total, self.exclusive, first);
        }
        // Every feasible shared split lies between these actual Cents
        // endpoints. Project each through the one native category target
        // and declared first owner, retaining its exact residual. Independent
        // rounding of two maxima can exclude a truthful residual allocation.
        let taxpayer_endpoint = crate::kernel::ratio::project_rounded_pair(
            total,
            [self.exclusive[0].checked_add(self.both)?, self.exclusive[1]],
            first,
        )?;
        let spouse_endpoint = crate::kernel::ratio::project_rounded_pair(
            total,
            [self.exclusive[0], self.exclusive[1].checked_add(self.both)?],
            first,
        )?;
        Some([0, 1].map(|owner| core::cmp::max(taxpayer_endpoint[owner], spouse_endpoint[owner])))
    }
}

// Generic nonnegative amount bounds. Every amount/capacity has the same
// caller-declared unit; no tax cap, ownership fact or source amount is inferred.
pub(crate) fn allowed_bounds(allowed: i64, sources: [i64; 2]) -> Option<(i64, i64)> {
    sources[0].checked_add(sources[1])?;
    native_allowed_bounds(allowed, sources)
}

// Native callers establish individual nonnegative capacities without requiring
// their sum to fit the amount carrier. Comparing the interval endpoints retains
// those domains; the checked-sum API above keeps its stricter availability rule.
pub(crate) fn native_allowed_bounds(allowed: i64, sources: [i64; 2]) -> Option<(i64, i64)> {
    if allowed < 0 || sources.iter().any(|value| *value < 0) {
        return None;
    }
    // The difference of two nonnegative i64 operands is representable.
    let (lower, upper) = native_capacity_interval(allowed, sources, crate::Owner::Taxpayer);
    (lower <= upper).then_some((lower, upper))
}

// Some native controls retain an empty interval as a required correction row.
// Return that actual interval unchanged, with the original native subtraction;
// availability and control construction remain the caller's responsibility.
pub(crate) fn native_capacity_interval(
    joint: i64,
    capacities: [i64; 2],
    owner: crate::Owner,
) -> (i64, i64) {
    let index = owner_index(owner);
    (
        crate::kernel::phase::floor(joint - capacities[1 - index], 0),
        crate::kernel::phase::cap(joint, [capacities[index]]),
    )
}

// The final native cap is already decided. Assign it only if the source
// bounds have exactly one result; never choose a ratio or a source priority.
pub(crate) fn unique_allowed(allowed: i64, sources: [i64; 2]) -> Option<[i64; 2]> {
    let (lower, upper) = allowed_bounds(allowed, sources)?;
    (lower == upper).then_some([lower, allowed - lower])
}

pub(crate) fn nonnegative_capacities(
    amount: i64,
    owners: Option<DeductionOwners>,
) -> Option<[i64; 2]> {
    if amount < 0 {
        return None;
    }
    if amount == 0 {
        return Some([0; 2]);
    }
    match owners? {
        DeductionOwners::Taxpayer => Some([amount, 0]),
        DeductionOwners::Spouse => Some([0, amount]),
        DeductionOwners::Both => Some([amount; 2]),
    }
}

pub(crate) fn allowed_capacities(allowed: i64, capacities: [i64; 2]) -> Option<[i64; 2]> {
    allowed_bounds(allowed, capacities)?;
    Some(capacities.map(|amount| crate::kernel::phase::cap(amount, [allowed])))
}

// Final eligible amount D is a subset of a known whole amount. The person's
// accepted whole share Y must also admit the actual complementary amount E.
// All arguments use the SAME caller-declared unit (native annual Dollars or
// source Cents); this function neither converts nor rounds any operand.
pub(crate) fn subset_bounds(
    eligible: i64,
    whole: i64,
    eligible_capacities: [i64; 2],
    complement_capacities: [i64; 2],
    accepted_whole: [i64; 2],
    owner: usize,
) -> Option<(i64, i64)> {
    if owner > 1
        || accepted_whole.iter().any(|amount| *amount < 0)
        || accepted_whole[0].checked_add(accepted_whole[1])? != whole
        || whole < 0
        || eligible < 0
        || eligible > whole
    {
        return None;
    }
    let owner_bounds = |joint, capacities| {
        let (minimum, maximum) = allowed_bounds(joint, capacities)?;
        Some(if owner == 0 { (minimum, maximum) } else { (joint - maximum, joint - minimum) })
    };
    let (dmin, dmax) = owner_bounds(eligible, eligible_capacities)?;
    let (emin, emax) = owner_bounds(whole.checked_sub(eligible)?, complement_capacities)?;
    let minimum = super::phase::floor(dmin, accepted_whole[owner].checked_sub(emax)?);
    let maximum = super::phase::cap(dmax, [accepted_whole[owner].checked_sub(emin)?]);
    (minimum <= maximum).then_some((minimum, maximum))
}

// Physical source ownership is signed and exact; this does not round money.
pub(crate) fn exclusive(amount: i64, owner: crate::Owner) -> [i64; 2] {
    let mut shares = [0; 2];
    shares[owner_index(owner)] = amount;
    shares
}

// A single strictly positive source establishes ownership, not capacity.
// Preserve the caller's target unchanged, including signed and zero amounts.
pub(crate) fn sole_contributor(total: i64, sources: [i64; 2]) -> Option<[i64; 2]> {
    let owner = match sources {
        [source, 0] if source > 0 => crate::Owner::Taxpayer,
        [0, source] if source > 0 => crate::Owner::Spouse,
        _ => return None,
    };
    Some(exclusive(total, owner))
}

// The caller has established this class's target within its native category.
// An exclusive physical source owns that target unchanged; mixed sources use
// the rounding owner's first-person projection and exact native residual.
pub(crate) fn project_source_class(
    amount: i64,
    cents: [i64; 2],
    first: crate::Owner,
) -> Option<[i64; 2]> {
    match cents {
        [source, 0] if source != 0 => Some([amount, 0]),
        [0, source] if source != 0 => Some([0, amount]),
        _ => super::ratio::project_native_pair(amount, cents, owner_index(first)),
    }
}

pub(crate) fn project_source_class_exact(
    joint: i64,
    cents: [super::ratio::exact::Exact; 2],
    first: crate::Owner,
) -> Option<[i64; 2]> {
    let zero = super::ratio::exact::whole(0);
    if cents[0] != zero && cents[1] == zero {
        Some([joint, 0])
    } else if cents[0] == zero && cents[1] != zero {
        Some([0, joint])
    } else {
        super::ratio::exact::project_native_pair(joint, cents, owner_index(first))
    }
}

fn owner_index(owner: crate::Owner) -> usize {
    match owner {
        crate::Owner::Taxpayer => 0,
        crate::Owner::Spouse => 1,
    }
}

pub(crate) fn allowed_bounds_for(
    joint: i64,
    capacities: [i64; 2],
    owner: crate::Owner,
) -> Option<(i64, i64)> {
    let (lower, upper) = allowed_bounds(joint, capacities)?;
    Some(match owner {
        crate::Owner::Taxpayer => (lower, upper),
        crate::Owner::Spouse => (joint - upper, joint - lower),
    })
}

// The same native capacity interval in the selected physical owner's order.
pub(crate) fn native_allowed_bounds_for(
    joint: i64,
    capacities: [i64; 2],
    owner: crate::Owner,
) -> Option<(i64, i64)> {
    native_allowed_bounds(
        joint,
        [capacities[owner_index(owner)], capacities[1 - owner_index(owner)]],
    )
}

pub(crate) fn native_unique_allowed(joint: i64, capacities: [i64; 2]) -> Option<[i64; 2]> {
    let (lower, upper) = native_allowed_bounds(joint, capacities)?;
    (lower == upper).then_some([lower, joint - lower])
}

// Validate an explicitly supplied share. A non-unique range supplies no default.
pub(crate) fn accepted_allowed(
    joint: i64,
    capacities: [i64; 2],
    owner: crate::Owner,
    selected: i64,
) -> Option<[i64; 2]> {
    accepted_interval(joint, allowed_bounds_for(joint, capacities, owner)?, owner, selected)
}

// Signed native choices already have their actual caller-established interval.
// Do not infer capacities or impose nonnegative rules on that source interval.
// Native subtraction retains the existing residual and overflow behavior.
pub(crate) fn accepted_interval(
    joint: i64,
    (lower, upper): (i64, i64),
    owner: crate::Owner,
    selected: i64,
) -> Option<[i64; 2]> {
    if !(lower..=upper).contains(&selected) {
        return None;
    }
    let mut shares = [0; 2];
    let index = owner_index(owner);
    shares[index] = selected;
    shares[1 - index] = joint - selected;
    Some(shares)
}

// Check representability before delegating to the one money-rounding owner.
// Actual signed source Cents must foot to the caller's native annual Dollars.
// The named first owner rounds once; the counterpart receives the exact residual.
pub(crate) fn rounded_shares(
    joint: i64,
    source_cents: [i64; 2],
    first: crate::Owner,
) -> Option<[i64; 2]> {
    let total = source_cents[0].checked_add(source_cents[1])?;
    for cents in [total, source_cents[owner_index(first)]] {
        cents.checked_abs()?.checked_add(50)?;
    }
    let first_share = super::ratio::dollars(source_cents[owner_index(first)]);
    joint.checked_sub(first_share)?;
    super::ratio::project_rounded_pair(joint, source_cents, owner_index(first))
}

#[cfg(test)]
#[path = "attribution_tests.rs"]
mod tests;
