extern crate std;

use super::*;
use alloc::string::String;
use std::panic::catch_unwind;

#[test]
fn native_projection_keeps_selected_rounding_and_unfooted_target() {
    for (joint, cents, first, expected) in [
        (101, [6000, 6000], 0, [60, 41]),
        (101, [6000, 6000], 1, [41, 60]),
        (0, [50, -50], 0, [1, -1]),
        (0, [50, -50], 1, [1, -1]),
        (-3, [-150, -150], 0, [-2, -1]),
        (-3, [-150, -150], 1, [-1, -2]),
        (7, [i64::MIN, i64::MAX], 1, [-92_233_720_368_547_751, 92_233_720_368_547_758]),
    ] {
        assert_eq!(native_project_pair(joint, cents, first), expected);
    }
}

#[test]
fn native_fraction_keeps_signed_weights_denominators_and_narrowing() {
    for (joint, sources, first, expected) in [
        (5, [1, 1], 0, [3, 2]),
        (5, [1, 1], 1, [2, 3]),
        (-5, [1, 1], 0, [-3, -2]),
        (5, [-1, 3], 0, [-3, 8]),
        (5, [-1, 3], 1, [-3, 8]),
        (5, [1, -3], 0, [-2, 7]),
        (5, [1, -3], 1, [-2, 7]),
        (0, [1, 1], 0, [0, 0]),
        (2, [i64::MAX, -i64::MAX + 1], 0, [-2, 4]),
    ] {
        assert_eq!(native_fraction_pair(joint, sources, first), expected);
    }
}

#[test]
fn native_pair_invalid_indices_still_panic() {
    for first in [2, usize::MAX] {
        assert!(catch_unwind(|| native_project_pair(0, [0, 0], first)).is_err());
        // Invalid indexing precedes even the overflowing denominator sum.
        let failure = catch_unwind(|| native_fraction_pair(0, [i64::MAX, 1], first));
        let payload = failure.unwrap_err();
        let message = payload.downcast_ref::<String>().unwrap();
        assert!(message.contains("index out of bounds"));
    }
}

#[test]
fn native_fraction_zero_denominator_still_panics() {
    for (joint, sources, first) in [(0, [0, 0], 0), (5, [1, -1], 1)] {
        assert!(catch_unwind(|| native_fraction_pair(joint, sources, first)).is_err());
    }
}

#[test]
fn native_pair_residual_preserves_native_overflow_policy() {
    for (joint, cents, first, wrapped) in
        [(i64::MAX, [-50, 0], 0, [-1, i64::MIN]), (i64::MIN, [0, 50], 1, [i64::MAX, 1])]
    {
        let result = catch_unwind(|| native_project_pair(joint, cents, first));
        if cfg!(debug_assertions) {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), wrapped);
        }
    }
    let narrowed = catch_unwind(|| native_fraction_pair(i64::MAX, [2, -1], 0));
    if cfg!(debug_assertions) {
        assert!(narrowed.is_err());
    } else {
        assert_eq!(narrowed.unwrap(), [-2, i64::MIN + 1]);
    }
}

#[test]
fn native_fraction_source_sum_preserves_native_overflow_policy() {
    for (sources, expected) in [([i64::MAX, 1], [0, 1]), ([i64::MIN, -1], [-1, 2])] {
        let result = catch_unwind(|| native_fraction_pair(1, sources, 0));
        if cfg!(debug_assertions) {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), expected);
        }
    }
}

#[test]
fn guarded_fraction_pair_keeps_existing_footing_and_divisor_guards() {
    for (joint, sources, numerator, denominator, first, expected) in [
        (3, [3, 3], 1, 2, 0, Some([2, 1])),
        (3, [3, 3], 1, 2, 1, Some([1, 2])),
        (4, [3, 3], 1, 2, 0, None),
        (3, [3, 3], 1, -2, 0, None),
        (3, [3, 3], 1, 0, 0, None),
        (0, [i64::MAX, 1], 1, 2, 2, None),
        (0, [i64::MAX, 1], 1, 0, 0, None),
    ] {
        assert_eq!(project_fraction_pair(joint, sources, numerator, denominator, first), expected);
    }
}

mod exact_tests {
    use crate::kernel::ratio::exact::{dollar_allocations, dollars, ratio, rounded, whole};

    #[test]
    fn coprime_denominators_keep_exact_residual_and_money_boundaries() {
        // 100 distinct primes produce a denominator far larger than u128.
        let mut used = 0;
        let mut candidate = 1009_i64;
        let mut sum = whole(0);
        while used < 100 {
            if (2..).take_while(|d| d * d <= candidate).all(|d| candidate % d != 0) {
                sum += ratio(1, candidate).unwrap();
                used += 1;
            }
            candidate += 1;
        }
        let residual = whole(1) - &sum;
        assert_eq!(&sum + residual, whole(1));
        let tiny = ratio(1, 999_999_937).unwrap();
        assert_eq!(dollars(&(whole(50) - &tiny)), Some(0));
        assert_eq!(dollars(&whole(50)), Some(1));
        assert_eq!(dollars(&(whole(50) + tiny)), Some(1));
        assert_eq!(dollars(&whole(-50)), Some(-1));
        assert_eq!(ratio(1, 0), None);
        assert_eq!(rounded(&(whole(i64::MAX) + whole(1))), None);
    }

    #[test]
    fn source_ratio_is_not_a_display_percentage_or_rounded_cent() {
        // 2025 Instructions for Form8962, p20, Gary/Jim example:
        // https://www.irs.gov/pub/irs-pdf/i8962.pdf
        let gary = whole(1_500_000) * ratio(12_000, 18_000).unwrap();
        assert_eq!(dollars(&gary), Some(10_000));
        // Two thirds of 74 cents rounds to $0 directly; rounding to cents
        // first would produce 49 cents here, while 74.5 cents crosses 50.
        let cents = ratio(149, 2).unwrap() * ratio(2, 3).unwrap();
        assert_eq!(dollars(&cents), Some(0));
        assert_eq!(dollars(&(cents + ratio(1, 3).unwrap())), Some(1));
    }

    #[test]
    fn shared_rounding_preserves_total_and_source_identity() {
        let amounts = [(7, whole(50)), (3, whole(50))].into_iter().collect();
        let result = dollar_allocations(&amounts);
        assert_eq!(result.values().sum::<i64>(), 1);
        assert_eq!(result[&3], 1);
        assert_eq!(result[&7], 0);
        let thirds = (1..=3).map(|id| (id, ratio(100, 3).unwrap())).collect();
        assert_eq!(dollar_allocations(&thirds).values().sum::<i64>(), 1);
    }
}
