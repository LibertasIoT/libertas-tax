//! The one boundary between a natural-unit Number (the value as the form prints it, with a step)
//! and the exact integer the kernel computes with.
//!
//! A stored quantity is an `f64` in its natural unit (percent, gallon, hour, acre...). The kernel
//! never computes in `f64`: it reads the stored value once through [`exact`] or [`units`] and
//! continues in integers. Binary representation noise is accepted; extra decimal precision is not.

/// The value as an exact whole number of `1 / per_unit` steps, or `None` when it is not finite,
/// is outside the exact integer range, or is not on the step grid.
pub(crate) fn exact(value: f64, per_unit: u32) -> Option<i64> {
    if !value.is_finite() {
        return None;
    }
    let scaled = value * f64::from(per_unit);
    if scaled.abs() > 9.0e15 {
        return None;
    }
    // Round half away from zero (`f64::round` is not in core).
    let units = if scaled >= 0.0 { (scaled + 0.5) as i64 } else { (scaled - 0.5) as i64 };
    if (scaled - units as f64).abs() > scaled.abs() * f64::EPSILON * 4.0 {
        return None;
    }
    Some(units)
}

/// [`exact`], with 0 for a value that is not on the grid (every caller's own range check then
/// refuses it, because 0 steps is below each quantity's minimum or is the explicit zero).
pub(crate) fn units(value: f64, per_unit: u32) -> i64 {
    exact(value, per_unit).unwrap_or(0)
}

/// An exact count of `1 / per_unit` steps as the natural-unit value a page holds.
#[cfg(test)]
pub(crate) fn natural(units: i64, per_unit: u32) -> f64 {
    units as f64 / f64::from(per_unit)
}

/// Gallons as exact thousandths of a gallon.
pub(crate) fn thousandths(gallons: f64) -> i64 {
    units(gallons, 1_000)
}
