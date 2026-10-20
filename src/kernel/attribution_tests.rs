use super::*;
use crate::Owner::{Spouse, Taxpayer};

#[test]
fn signed_source_and_native_rounding_keep_the_declared_owner() {
    assert_eq!(exclusive(-123, Spouse), [0, -123]);
    assert_eq!(rounded_shares(1, [50, 50], Taxpayer), Some([1, 0]));
    assert_eq!(rounded_shares(1, [50, 50], Spouse), Some([0, 1]));
    assert_eq!(rounded_shares(-1, [-50, -50], Taxpayer), Some([-1, 0]));
    assert_eq!(rounded_shares(0, [-50, 50], Taxpayer), Some([-1, 1]));
    assert_eq!(rounded_shares(2, [50, 50], Taxpayer), None);
    assert_eq!(rounded_shares(0, [i64::MAX, 1], Taxpayer), None);
    assert_eq!(rounded_shares(0, [i64::MIN, 0], Taxpayer), None);
    assert_eq!(rounded_shares(i64::MIN, [50, 0], Taxpayer), None);
}

#[test]
fn capacities_bound_explicit_answers_without_inventing_a_ratio() {
    assert_eq!(allowed_bounds_for(500, [100, 1000], Taxpayer), Some((0, 100)));
    assert_eq!(allowed_bounds_for(500, [100, 1000], Spouse), Some((400, 500)));
    assert_eq!(unique_allowed(500, [100, 1000]), None);
    assert_eq!(accepted_allowed(500, [100, 1000], Spouse, 450), Some([50, 450]));
    assert_eq!(accepted_allowed(500, [100, 1000], Taxpayer, 101), None);
    assert_eq!(accepted_allowed(500, [100, 1000], Spouse, 399), None);
    assert_eq!(unique_allowed(100, [100, 0]), Some([100, 0]));
    assert_eq!(allowed_bounds(1, [-1, 2]), None);
    assert_eq!(allowed_bounds(-1, [1, 2]), None);
    assert_eq!(allowed_bounds(4, [1, 2]), None);
    assert_eq!(allowed_bounds(1, [i64::MAX, 1]), None);
    assert_eq!(allowed_capacities(500, [100, 1000]), Some([100, 500]));
}

#[test]
fn shared_ownership_is_established_capacity_and_unknown_is_not_shared() {
    assert_eq!(nonnegative_capacities(98, Some(DeductionOwners::Both)), Some([98, 98]));
    assert_eq!(nonnegative_capacities(98, None), None);
    assert_eq!(nonnegative_capacities(0, None), Some([0, 0]));
    let mut sources = ExpenseSourceCents::known([0, 0]);
    sources.add(98, Some(DeductionOwners::Both));
    assert_eq!(sources.annual_capacities(0), Some([1, 1]));
    sources.add(1, None);
    assert_eq!(sources.annual_capacities(0), None);
    let mut known_zero = ExpenseSourceCents::known([0, 0]);
    known_zero.add(49, None);
    assert_eq!(known_zero.annual_capacities(0), Some([0, 0]));
    let mut exclusive = ExpenseSourceCents::known([50, 50]);
    assert_eq!(exclusive.annual_capacities(1), Some([0, 1]));
    exclusive.add(-1, Some(DeductionOwners::Taxpayer));
    assert_eq!(exclusive.annual_capacities(0), None);
    assert_eq!(ExpenseSourceCents::known([i64::MAX, 1]).annual_capacities(0), None);
}

#[test]
fn subset_must_fit_both_accepted_whole_and_actual_complement() {
    assert_eq!(subset_bounds(40, 100, [40, 40], [20, 60], [50, 50], 0), Some((30, 40)));
    assert_eq!(subset_bounds(40, 100, [40, 40], [20, 60], [50, 50], 1), Some((0, 10)));
    assert_eq!(subset_bounds(40, 100, [40, 40], [20, 60], [49, 50], 0), None);
    assert_eq!(subset_bounds(101, 100, [101, 101], [0, 0], [50, 50], 0), None);
    assert_eq!(subset_bounds(40, 100, [40, 0], [0, 60], [0, 100], 0), None);
    assert_eq!(subset_bounds(40, 100, [40, 40], [20, 60], [50, 50], 2), None);
}

#[test]
fn canonical_attribution_invalid_sources_cannot_establish_zero() {
    use crate::kernel::attribution::ExpenseSourceCents;
    for amounts in [[-1, 0], [-1, 1], [i64::MIN, 0]] {
        assert_eq!(ExpenseSourceCents::known(amounts).annual_capacities(0), None);
    }
    let mut sources = ExpenseSourceCents::known([100, 0]);
    sources.add(-100, None);
    assert_eq!(sources.annual_capacities(0), None);
    // Genuine nonnegative zero rounding remains distinguishable from invalid
    // cancellation and from a positive, unidentified annual source.
    assert_eq!(ExpenseSourceCents::known([49, 0]).annual_capacities(0), Some([0, 0]));
    let mut unknown = ExpenseSourceCents::known([0, 0]);
    unknown.add(100, None);
    assert_eq!(unknown.annual_capacities(0), None);
}

#[test]
fn canonical_attribution_overflow_and_invalid_first_are_unavailable() {
    use crate::kernel::attribution::{ExpenseSourceCents, allowed_bounds, subset_bounds};
    assert_eq!(ExpenseSourceCents::known([0, 0]).annual_capacities(2), None);
    assert_eq!(ExpenseSourceCents::known([i64::MAX, 1]).annual_capacities(0), None);
    assert_eq!(ExpenseSourceCents::known([i64::MAX, 0]).annual_capacities(0), None);
    let mut sources = ExpenseSourceCents::known([i64::MAX, 0]);
    sources.add(1, Some(DeductionOwners::Taxpayer));
    assert_eq!(sources.annual_capacities(0), None);
    let maximum_roundable = i64::MAX - 50;
    assert_eq!(
        ExpenseSourceCents::known([maximum_roundable, 0]).annual_capacities(0),
        Some([i64::MAX / 100, 0])
    );
    assert_eq!(allowed_bounds(0, [i64::MAX, 1]), None);
    assert_eq!(allowed_bounds(i64::MAX, [i64::MAX, 0]), Some((i64::MAX, i64::MAX)));
    assert_eq!(subset_bounds(0, i64::MAX, [0, 0], [i64::MAX, 0], [i64::MAX, 1], 0), None);
    assert_eq!(
        subset_bounds(i64::MAX, i64::MAX, [i64::MAX, 0], [0, 0], [i64::MAX, 0], 0),
        Some((i64::MAX, i64::MAX))
    );
    assert_eq!(subset_bounds(0, 0, [0, 0], [0, 0], [0, 0], 2), None);
}

#[test]
fn canonical_attribution_shared_fractional_capacities_keep_native_residual() {
    use crate::kernel::attribution::{ExpenseSourceCents, allowed_bounds};
    let mut sources = ExpenseSourceCents::known([25, 25]);
    sources.add(101, Some(DeductionOwners::Both));
    // The native category is151 Cents ->2 Dollars. The owner's actual
    // shared split1/100 Cents permits [0,2] with taxpayer rounding first;
    // independent rounding of both maxima would falsely force [1,1].
    assert_eq!(crate::kernel::ratio::project_rounded_pair(2, [26, 125], 0), Some([0, 2]));
    assert_eq!(sources.annual_capacities(0), Some([1, 2]));
    assert_eq!(allowed_bounds(2, sources.annual_capacities(0).unwrap()), Some((0, 1)));
    assert_eq!(crate::kernel::ratio::project_rounded_pair(2, [125, 26], 1), Some([2, 0]));
    assert_eq!(sources.annual_capacities(1), Some([2, 1]));
    assert_eq!(allowed_bounds(2, sources.annual_capacities(1).unwrap()), Some((1, 2)));
    assert_eq!(ExpenseSourceCents::known([25, 125]).annual_capacities(0), Some([0, 2]));
    assert_eq!(ExpenseSourceCents::known([25, 125]).annual_capacities(1), Some([1, 1]));
}

#[test]
fn native_capacity_domains_keep_wide_sums_and_empty_correction_intervals() {
    // Checked callers still reject an unrepresentable sum; native intervals
    // need only representable individual capacities, including a net MIN loss.
    assert_eq!(native_allowed_bounds_for(1, [i64::MAX, 1], Spouse), Some((0, 1)));
    assert_eq!(native_unique_allowed(i64::MAX, [i64::MAX, 1]), None);
    assert_eq!(native_capacity_interval(5, [1, 1], Taxpayer), (4, 1));
}

#[test]
fn signed_interval_choices_do_not_invent_nonnegative_capacities() {
    assert_eq!(accepted_interval(-3, (-3, 0), Spouse, -2), Some([-1, -2]));
    assert_eq!(accepted_interval(-3, (-3, 0), Spouse, 1), None);
}

#[test]
fn sole_contributor_establishes_ownership_without_bounding_the_target() {
    assert_eq!(sole_contributor(20, [1, 0]), Some([20, 0]));
    assert_eq!(sole_contributor(-20, [0, 1]), Some([0, -20]));
    assert_eq!(sole_contributor(0, [1, 0]), Some([0, 0]));
    for sources in [[0, 0], [1, 1], [-1, 0], [0, -1], [1, -1]] {
        assert_eq!(sole_contributor(0, sources), None);
    }
}
