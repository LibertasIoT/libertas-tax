// Reusable bounds and phase arithmetic. Tax policy and source units stay callers.
use super::ratio::{ceiling, fraction, positive};

pub(crate) fn cap<T: Ord>(first: T, rest: impl IntoIterator<Item = T>) -> T {
    rest.into_iter().fold(first, core::cmp::min)
}

pub(crate) fn floor<T: Ord>(value: T, minimum: T) -> T {
    core::cmp::max(value, minimum)
}

pub(crate) fn bound<T: Ord>(value: T, minimum: T, maximum: T) -> T {
    value.clamp(minimum, maximum)
}

#[derive(Clone, Copy)]
pub(crate) enum StepRounding {
    Down,
    Up,
}

// Unlike a phase benefit, this factor is also suitable for signed QBI inputs.
pub(crate) fn phase_position(income: i64, start: i64, width: i64) -> (i64, i64) {
    assert!(width > 0);
    (cap(positive(income - start), [width]), width)
}

pub(crate) fn steps(excess: i64, unit: i64, rounding: StepRounding) -> i64 {
    assert!(unit > 0);
    match rounding {
        StepRounding::Down => super::ratio::trunc_i64(positive(excess), unit),
        StepRounding::Up => ceiling(excess, unit),
    }
}

// These two rounding stages are deliberately separate. Rounding the reduction
// before subtraction is not interchangeable with rounding the remaining amount.
pub(crate) fn reduce_by_fraction(original: i64, excess: i64, width: i64) -> i64 {
    original - fraction(original, excess, width)
}

pub(crate) fn remaining_fraction(original: i64, remaining: i64, width: i64) -> i64 {
    fraction(original, remaining, width)
}

pub(crate) fn phase_out(original: i64, rounded_reduction: i64) -> i64 {
    positive(original - rounded_reduction)
}

pub(crate) fn linear_reduction(original: i64, income: i64, start: i64, width: i64) -> i64 {
    let (excess, width) = phase_position(income, start, width);
    fraction(original, excess, width)
}

// The source evaluates its native-width product before entering this projector.
// Keep the legacy native offset and quotient; an Exact replacement could hide
// an overflow which the old IRA owner reports by the native arithmetic failure.
pub(crate) fn remaining_up(numerator: i64, width: i64, quantum: i64) -> i64 {
    assert!(width > 0 && quantum > 0);
    super::ratio::trunc_i64(numerator + width * quantum - 1, width * quantum) * quantum
}

pub(crate) fn linear_remaining(original: i64, income: i64, start: i64, width: i64) -> i64 {
    original - linear_reduction(original, income, start, width)
}

pub(crate) fn linear_up(
    original: i64,
    income: i64,
    start: i64,
    width: i64,
    quantum: i64,
    interior_minimum: i64,
) -> i64 {
    if income <= start {
        return original;
    }
    if income >= start + width {
        return 0;
    }
    let numerator = original * (start + width - income);
    cap(floor(remaining_up(numerator, width, quantum), interior_minimum), [original])
}

pub(crate) fn checked_loss(amount: i64) -> Option<i64> {
    if amount < 0 { amount.checked_neg() } else { Some(0) }
}

// Some older worksheets add a literal offset before division, rather than
// computing numerator + denominator - 1. Preserve that overflow boundary too.
pub(crate) fn offset_steps(excess: i64, offset: i64, unit: i64) -> i64 {
    super::ratio::trunc_i64(positive(excess) + offset, unit)
}
