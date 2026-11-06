// Arithmetic policies only. Source formulas, eligibility and units remain callers.
use alloc::{vec, vec::Vec};
use core::cmp::{max, min};

pub(crate) fn dollars(cents: i64) -> i64 {
    // Every signed64 cent value has a representable dollar result. Widen before
    // the signed half offset, including MIN's otherwise unrepresentable magnitude.
    let cents = i128::from(cents);
    ((cents + cents.signum() * 50) / 100) as i64
}

/// Project actual source cents to one native rounded joint amount. The selected
/// first source rounds once; the other receives the exact joint residual.
/// A failed footing is unavailable, never an attribution to the residual owner.
pub(crate) fn project_rounded_pair(joint: i64, cents: [i64; 2], first: usize) -> Option<[i64; 2]> {
    if first >= 2 || dollars(cents[0] + cents[1]) != joint {
        return None;
    }
    project_native_pair(joint, cents, first)
}

/// Project a known native target after the caller establishes the whole
/// category's source footing. Its class residual need not equal independently
/// rounded class cents. Unknown targets and invented ownership are not inputs.
/// An invalid owner index or unrepresentable native residual is unavailable.
pub(crate) fn project_native_pair(joint: i64, cents: [i64; 2], first: usize) -> Option<[i64; 2]> {
    if first >= 2 {
        return None;
    }
    let mut result = [0; 2];
    result[first] = dollars(cents[first]);
    result[1 - first] = joint.checked_sub(result[first])?;
    Some(result)
}

// Literal native stages for caller-established targets and owner indices.
// Keep the selected rounding, then native residual; add no footing or overflow guard.
#[allow(dead_code)] // Additive phase B API; attribution consumers land separately.
pub(crate) fn native_project_pair(joint: i64, cents: [i64; 2], first: usize) -> [i64; 2] {
    native_pair(joint, dollars(cents[first]), first)
}

#[allow(dead_code)] // Additive phase B API; attribution consumers land separately.
pub(crate) fn native_fraction_pair(joint: i64, sources: [i64; 2], first: usize) -> [i64; 2] {
    native_pair(joint, fraction(joint, sources[first], sources[0] + sources[1]), first)
}

fn native_pair(joint: i64, selected: i64, first: usize) -> [i64; 2] {
    let mut result = [0; 2];
    result[first] = selected;
    result[1 - first] = joint - result[first];
    result
}

/// Project one native rational component without rounding its source operands
/// at an intermediate precision. The caller owns the actual native joint
/// amount, units and fraction; a failed footing remains unavailable.
pub(crate) fn project_fraction_pair(
    joint: i64,
    sources: [i64; 2],
    numerator: i64,
    denominator: i64,
    first: usize,
) -> Option<[i64; 2]> {
    if first >= 2
        || denominator <= 0
        || fraction(sources[0] + sources[1], numerator, denominator) != joint
    {
        return None;
    }
    Some(native_pair(joint, fraction(sources[first], numerator, denominator), first))
}

pub(crate) fn fraction(value: i64, numerator: i64, denominator: i64) -> i64 {
    // Retain the legacy narrowing and zero-denominator behavior. Checked callers
    // use checked_fraction rather than relying on source-count assumptions.
    fraction_wide(value, numerator, denominator) as i64
}

fn fraction_wide(value: i64, numerator: i64, denominator: i64) -> i128 {
    // A signed64 product and signed64 half offset fit signed128. Preserve the
    // original negative-denominator rounding, including truncating denominator/2.
    let product = i128::from(value) * i128::from(numerator);
    (product + product.signum() * i128::from(denominator / 2)) / i128::from(denominator)
}

/// Same fractional policy, unavailable for zero denominator or a final native
/// amount outside signed64. Source units and rounding stages remain caller-owned.
#[allow(dead_code)] // Additive phase A owner; checked consumers migrate separately.
pub(crate) fn checked_fraction(value: i64, numerator: i64, denominator: i64) -> Option<i64> {
    (denominator != 0)
        .then(|| fraction_wide(value, numerator, denominator))
        .and_then(|value| i64::try_from(value).ok())
}

/// Sum known signed source amounts without an intermediate native-width cap.
/// Exact integral accumulation preserves cancellation; only the final projection
/// can be unavailable. An empty known sequence is zero, never a missing source.
#[allow(dead_code)] // Additive phase A owner; checked consumers migrate separately.
pub(crate) fn checked_sum(values: impl IntoIterator<Item = i64>) -> Option<i64> {
    let mut values = values.into_iter();
    let mut total = 0_i128;
    while let Some(value) = values.next() {
        if let Some(next) = total.checked_add(i128::from(value)) {
            total = next;
        } else {
            // Integral sources need no rational construction in the ordinary
            // path. Preserve arbitrary-width cancellation if even i128 fills.
            let total = exact::Exact::from_integer(total.into())
                + exact::whole(value)
                + values.map(exact::whole).sum::<exact::Exact>();
            return exact::rounded(&total);
        }
    }
    i64::try_from(total).ok()
}

pub(crate) fn positive(v: i64) -> i64 {
    max(v, 0)
}

pub(crate) fn ceiling(v: i64, unit: i64) -> i64 {
    (positive(v) + unit - 1) / unit
}

/// Native quotients preserve the caller's operand type, truncation, and failures.
pub(crate) const fn trunc_i64(numerator: i64, denominator: i64) -> i64 {
    numerator / denominator
}
pub(crate) const fn trunc_i128(numerator: i128, denominator: i128) -> i128 {
    numerator / denominator
}
pub(crate) const fn trunc_u32(numerator: u32, denominator: u32) -> u32 {
    numerator / denominator
}

// Literal legacy half-offset recipes, including their signed behavior.
pub(crate) const fn half_offset_i64(numerator: i64, denominator: i64) -> i64 {
    (numerator + denominator / 2) / denominator
}
pub(crate) const fn half_offset_i128(numerator: i128, denominator: i128) -> i128 {
    (numerator + denominator / 2) / denominator
}

pub(crate) fn allocate_remaining(amount: i64, weights: &[i64]) -> Vec<i64> {
    let mut remaining = amount;
    let mut total: i64 = weights.iter().sum();
    weights
        .iter()
        .map(|&w| {
            let value = if total == 0 { 0 } else { min(remaining, fraction(remaining, w, total)) };
            remaining -= value;
            total -= w;
            value
        })
        .collect()
}

/// The same ordered remaining/total policy for a nonnegative whole-cent
/// amount whose aggregate is wide. Source weights are nonnegative native
/// cents; only each actual allocated source amount is projected to native.
pub(crate) fn allocate_remaining_exact(amount: &exact::Exact, weights: &[i64]) -> Option<Vec<i64>> {
    if !amount.is_integer() || amount < &exact::whole(0) || weights.iter().any(|w| *w < 0) {
        return None;
    }
    let mut remaining = amount.clone();
    let mut total: exact::Exact = weights.iter().map(|w| exact::whole(*w)).sum();
    let mut result = Vec::with_capacity(weights.len());
    for &weight in weights {
        let value = if total == exact::whole(0) {
            exact::whole(0)
        } else {
            exact::rounded_exact(&(&remaining * exact::whole(weight) / &total))
                .min(remaining.clone())
        };
        let value = exact::rounded(&value)?;
        remaining -= exact::whole(value);
        total -= exact::whole(weight);
        result.push(value);
    }
    Some(result)
}

// Stream one cumulative projection at a time. The source owns its projection,
// denominator/zero policy and amount stages; this state adds no fallback.
#[derive(Default)]
pub(crate) struct Prefix {
    cumulative: i64,
    previous: i64,
}
impl Prefix {
    pub(crate) fn next(&mut self, weight: i64, project: impl FnOnce(i64) -> i64) -> i64 {
        self.cumulative += weight;
        let through = project(self.cumulative);
        let share = through - self.previous;
        self.previous = through;
        share
    }
}

pub(crate) fn allocate_prefix(amount: i64, weights: &[i64]) -> Vec<i64> {
    let total: i64 = weights.iter().sum();
    if weights.is_empty() {
        return Vec::new();
    }
    let weights: Vec<i64> = if total > 0 { weights.to_vec() } else { vec![1; weights.len()] };
    let total: i64 = weights.iter().sum();
    let mut prefix = Prefix::default();
    weights
        .iter()
        .map(|w| prefix.next(*w, |cumulative| fraction(amount, cumulative, total)))
        .collect()
}

/// Allocate a prevalidated nonnegative amount by largest remainders.
/// The caller guarantees nonempty nonnegative weights and a positive, representable
/// validated_total equal to their sum. The amount is nonnegative. Earlier input
/// positions break equal-remainder ties.
pub(crate) fn allocate_largest_nonnegative(
    amount: i64,
    weights: &[i64],
    validated_total: i64,
) -> Vec<i64> {
    let mut shares = alloc::vec![0_i64; weights.len()];
    let mut remainders = alloc::vec![0_i128; weights.len()];
    for (index, payment) in weights.iter().enumerate() {
        let numerator = i128::from(amount) * i128::from(*payment);
        shares[index] = trunc_i128(numerator, i128::from(validated_total)) as i64;
        remainders[index] = numerator % i128::from(validated_total);
    }
    let mut remaining = amount - shares.iter().sum::<i64>();
    while remaining > 0 {
        let mut best = 0;
        for index in 1..weights.len() {
            if remainders[index] > remainders[best] {
                best = index;
            }
        }
        shares[best] += 1;
        remainders[best] = -1;
        remaining -= 1;
    }
    shares
}

pub(crate) mod exact {
    // Transient arithmetic only: accepted amounts and schema fields remain integers.
    // A hundred lawful coprime allocation denominators can exceed any fixed-width
    // integer, even when the final amount is only a few cents. num-rational's
    // alloc-backed BigRational retains those fractions until the prescribed rounding.
    pub(crate) type Exact = num_rational::BigRational;

    /// Stream exact rational source weights without a per-row rounding loss.
    /// The caller owns source ordering, bounds and the final amount projection.
    #[derive(Default)]
    pub(crate) struct ExactPrefix {
        cumulative: Exact,
        previous: i64,
    }
    impl ExactPrefix {
        pub(crate) fn next(&mut self, weight: &Exact, project: impl FnOnce(&Exact) -> i64) -> i64 {
            self.cumulative += weight;
            let through = project(&self.cumulative);
            let share = through - self.previous;
            self.previous = through;
            share
        }
    }

    pub(crate) fn whole(value: i64) -> Exact {
        Exact::from_integer(value.into())
    }

    pub(crate) fn ratio(numerator: i64, denominator: i64) -> Option<Exact> {
        (denominator > 0).then(|| Exact::new(numerator.into(), denominator.into()))
    }

    /// Apply the existing signed nearest-integer policy without narrowing
    /// the carrier. The caller owns the prescribed rounding boundary.
    pub(crate) fn rounded_exact(value: &Exact) -> Exact {
        value.round()
    }

    pub(crate) fn rounded(value: &Exact) -> Option<i64> {
        i64::try_from(rounded_exact(value).to_integer()).ok()
    }

    pub(crate) fn dollars_exact(cents: &Exact) -> Exact {
        rounded_exact(&(cents / whole(100)))
    }

    pub(crate) fn dollars(cents: &Exact) -> Option<i64> {
        i64::try_from(dollars_exact(cents).to_integer()).ok()
    }

    /// Project an exact conserved pair in its existing output unit. Round the
    /// selected source once, then give the other source the rounded joint's
    /// residual. Only the final person amounts must fit their native carriers.
    pub(crate) fn project_whole_pair(
        exact_joint: &Exact,
        sources: [Exact; 2],
        first: usize,
    ) -> Option<[i64; 2]> {
        if first >= 2 || &(&sources[0] + &sources[1]) != exact_joint {
            return None;
        }
        let selected = rounded_exact(&sources[first]);
        let mut result = [0; 2];
        result[first] = rounded(&selected)?;
        result[1 - first] = rounded(&(rounded_exact(exact_joint) - selected))?;
        Some(result)
    }

    /// Project actual signed rational owner cents to an independently
    /// supplied exact joint and its reported Dollars. Check source conservation
    /// before rounding: omitted cents cannot become a guessed owner residual.
    pub(crate) fn project_rounded_pair(
        joint: i64,
        exact_joint: &Exact,
        sources: [Exact; 2],
        first: usize,
    ) -> Option<[i64; 2]> {
        if first >= 2
            || &(&sources[0] + &sources[1]) != exact_joint
            || dollars(exact_joint)? != joint
        {
            return None;
        }
        project_native_pair(joint, sources, first)
    }

    /// Preserve a caller-established annual target for a source class whose
    /// residual need not equal independently rounded class cents. The first
    /// owner rounds once; the other receives the exact native residual.
    pub(crate) fn project_native_pair(
        joint: i64,
        sources: [Exact; 2],
        first: usize,
    ) -> Option<[i64; 2]> {
        if first >= 2 {
            return None;
        }
        let mut result = [0; 2];
        result[first] = dollars(&sources[first])?;
        result[1 - first] = joint.checked_sub(result[first])?;
        Some(result)
    }

    // Round one shared amount once, then distribute the whole-dollar result to
    // its existing source identities. Largest fractional remainders receive the
    // residual dollars; identity breaks ties, so reordering never changes a result.
    // This is a form-rounding projection, not a new allocation of source money.
    pub(crate) fn dollar_allocations(
        amounts: &alloc::collections::BTreeMap<i64, Exact>,
    ) -> alloc::collections::BTreeMap<i64, i64> {
        let total: Exact = amounts.values().sum();
        let target = dollars(&total).expect("bounded shared amount");
        let mut result = alloc::collections::BTreeMap::new();
        let mut remainders = alloc::vec::Vec::new();
        for (&id, amount) in amounts {
            assert!(*amount >= whole(0));
            let value = amount / whole(100);
            let floor = i64::try_from(value.to_integer()).expect("bounded source amount");
            result.insert(id, floor);
            remainders.push((id, value - whole(floor)));
        }
        remainders.sort_by(|(a_id, a), (b_id, b)| b.cmp(a).then(a_id.cmp(b_id)));
        let residual = target - result.values().sum::<i64>();
        for (id, _) in remainders.into_iter().take(residual as usize) {
            *result.get_mut(&id).expect("existing identity") += 1;
        }
        result
    }
}

#[cfg(test)]
#[path = "ratio_tests.rs"]
mod native_pair_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remaining_allocation_keeps_order_before_native_source_projection() {
        assert_eq!(allocate_remaining_exact(&exact::whole(2), &[1, 1, 1]), Some(vec![1, 1, 0]));
        let wide = exact::whole(i64::MAX) + exact::whole(1);
        assert_eq!(
            allocate_remaining_exact(&wide, &[i64::MAX, i64::MAX]),
            Some(vec![4_611_686_018_427_387_904; 2])
        );
        assert_eq!(
            allocate_remaining_exact(&(exact::whole(i64::MAX) * exact::whole(3)), &[1, 1]),
            None
        );
    }

    #[test]
    fn native_pair_keeps_established_target_and_selected_owner_rounding() {
        for (joint, cents, first, expected) in [
            (101, [6000, 6000], 0, [60, 41]),
            (101, [6000, 6000], 1, [41, 60]),
            (0, [50, -50], 0, [1, -1]),
            (0, [50, -50], 1, [1, -1]),
            (-3, [-150, -150], 0, [-2, -1]),
            (-3, [-150, -150], 1, [-1, -2]),
            (i64::MIN, [i64::MIN, 0], 0, [-92_233_720_368_547_758, -9_131_138_316_486_228_050]),
        ] {
            assert_eq!(project_native_pair(joint, cents, first), Some(expected));
            assert_eq!(
                exact::project_native_pair(joint, cents.map(exact::whole), first),
                Some(expected)
            );
        }
        let wide = exact::whole(i64::MAX) + exact::whole(1);
        assert_eq!(
            exact::project_native_pair(0, [wide.clone(), -wide], 0),
            Some([92_233_720_368_547_758, -92_233_720_368_547_758])
        );
        assert_eq!(project_native_pair(0, [i64::MAX, i64::MAX], 2), None);
        assert_eq!(project_native_pair(0, [0, 0], usize::MAX), None);
        assert_eq!(project_native_pair(i64::MAX, [-50, 0], 0), None);
        assert_eq!(project_native_pair(i64::MIN, [50, 0], 0), None);
        assert_eq!(project_rounded_pair(101, [6000, 6000], 0), None);
        assert_eq!(project_rounded_pair(120, [6000, 6000], 0), Some([60, 60]));
        assert_eq!(project_rounded_pair(0, [i64::MAX, i64::MAX], 2), None);
    }

    #[test]
    fn dollars_accepts_all_signed64_cents_and_preserves_half_boundaries() {
        for (cents, expected) in [
            (i64::MIN, -92_233_720_368_547_758),
            (-151, -2),
            (-150, -2),
            (-149, -1),
            (-51, -1),
            (-50, -1),
            (-49, 0),
            (0, 0),
            (49, 0),
            (50, 1),
            (51, 1),
            (149, 1),
            (150, 2),
            (151, 2),
            (i64::MAX, 92_233_720_368_547_758),
        ] {
            assert_eq!(dollars(cents), expected);
        }
    }

    #[test]
    fn checked_fraction_preserves_signed_policy_and_checks_final_native_amount() {
        for (value, numerator, denominator, expected) in [
            (5, 1, 2, Some(3)),
            (-5, 1, 2, Some(-3)),
            (5, -1, 2, Some(-3)),
            (5, 1, -2, Some(-2)),
            (-5, 1, -2, Some(2)),
            (i64::MAX, i64::MAX, i64::MAX, Some(i64::MAX)),
            (i64::MIN, i64::MAX, i64::MAX, Some(i64::MIN)),
            (i64::MIN, -1, 1, None),
            (i64::MAX, 2, 1, None),
            (i64::MIN, 2, 1, None),
            (i64::MIN, i64::MIN, 0, None),
        ] {
            assert_eq!(checked_fraction(value, numerator, denominator), expected);
        }
    }

    #[test]
    fn checked_sum_preserves_cancellation_before_final_native_projection() {
        assert_eq!(checked_sum([]), Some(0));
        assert_eq!(checked_sum([i64::MAX, 1, i64::MIN]), Some(0));
        assert_eq!(checked_sum([i64::MIN, -1, i64::MAX]), Some(-2));
        assert_eq!(checked_sum([i64::MAX, i64::MAX, -i64::MAX]), Some(i64::MAX));
        assert_eq!(checked_sum([i64::MIN]), Some(i64::MIN));
        assert_eq!(checked_sum([i64::MAX, 1]), None);
        assert_eq!(checked_sum([i64::MIN, -1]), None);
    }

    #[test]
    fn rounding_policies_keep_signed_and_half_offset_distinctions() {
        // An integral-cent stage may be wider than the eventual native
        // output. Keep both signed half ties exact before that projection.
        let wide = exact::whole(i64::MAX) + exact::whole(1);
        let half = exact::ratio(1, 2).unwrap();
        let above = &wide + &half;
        let below = -&wide - &half;
        assert_eq!(exact::rounded_exact(&above), &wide + exact::whole(1));
        assert_eq!(exact::rounded_exact(&below), -&wide - exact::whole(1));
        assert_eq!(exact::rounded(&above), None);
        assert_eq!(exact::rounded(&below), None);
        assert_eq!(dollars(-50), -1);
        assert_eq!(fraction(-5, 1, 2), -3);
        assert_eq!(half_offset_i64(-5, 2), -2);
        assert_eq!(half_offset_i128(-5, 2), -2);
        assert_eq!(fraction(5, 1, -2), -2);
        assert_eq!(trunc_i64(-5, 2), -2);
        assert_eq!(trunc_i128(i128::from(i64::MAX) * 2, 2), i128::from(i64::MAX));
        assert_eq!(trunc_u32(9, 2), 4);
    }

    #[test]
    fn allocator_policies_keep_distinct_residual_and_zero_weight_behavior() {
        let weights = [1, 1, 1];
        let independent: Vec<_> = weights.iter().map(|&w| fraction(2, w, 3)).collect();
        assert_eq!(independent, [1, 1, 1]);
        assert_eq!(allocate_remaining(2, &weights), [1, 1, 0]);
        assert_eq!(allocate_prefix(2, &weights), [1, 0, 1]);
        assert_eq!(allocate_largest_nonnegative(2, &weights, 3), [1, 1, 0]);
        assert_eq!(allocate_remaining(1, &weights), [0, 1, 0]);
        assert_eq!(allocate_prefix(1, &weights), [0, 1, 0]);
        assert_eq!(allocate_largest_nonnegative(1, &weights, 3), [1, 0, 0]);
        assert_eq!(allocate_remaining(-2, &[1, 2]), [-2, 0]);
        assert_eq!(allocate_prefix(-2, &[1, 2]), [-1, -1]);
        assert_eq!(allocate_remaining(3, &[0, 0]), [0, 0]);
        assert_eq!(allocate_prefix(3, &[0, 0]), [2, 1]);
        assert!(allocate_remaining(3, &[]).is_empty());
        assert!(allocate_prefix(3, &[]).is_empty());
        assert_eq!(allocate_largest_nonnegative(1, &[0, 1, 1], 2), [0, 1, 0]);
        assert_eq!(allocate_largest_nonnegative(5, &[1, 1], 2), [3, 2]);
    }

    #[test]
    #[should_panic(expected = "divide by zero")]
    fn native_zero_denominator_keeps_failure() {
        let _ = trunc_i64(1, 0);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn native_signed_min_division_keeps_failure() {
        let _ = trunc_i64(i64::MIN, -1);
    }
}

#[cfg(test)]
mod prefix_tests {
    use super::*;

    #[test]
    fn streaming_prefix_preserves_negative_weights_and_projection_order() {
        let mut prefix = Prefix::default();
        let mut projected = Vec::new();
        let shares = [3, -2, 4].map(|weight| {
            prefix.next(weight, |cumulative| {
                projected.push(cumulative);
                fraction(7, cumulative, 5)
            })
        });
        assert_eq!(projected, [3, 1, 5]);
        assert_eq!(shares, [4, -3, 6]);
    }

    #[test]
    fn streaming_prefix_keeps_source_zero_policy_without_a_fallback() {
        let mut prefix = Prefix::default();
        assert_eq!(prefix.next(2, |_| 0), 0);
        assert_eq!(prefix.next(3, |cumulative| cumulative), 5);
        assert_eq!(prefix.next(-1, |cumulative| cumulative), -1);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "overflow")]
    fn native_increment_failure_precedes_projection() {
        let mut prefix = Prefix::default();
        prefix.next(i64::MAX, |_| 0);
        prefix.next(1, |_| panic!("projection ran before increment failure"));
    }
}
