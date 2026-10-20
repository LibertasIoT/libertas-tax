// Shared class arithmetic; source owners retain recognition and eligibility.
#![allow(dead_code)]

use super::{phase, ratio::fraction, source_fact::SourceFact};

// Property operands are the native annual Dollars AFTER source recognition and
// class selection, BEFORE installment payments, activity limits and §1231 lookback.
// The source retains holding period, related-party rules, dates and selected system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RecaptureInput {
    LongSection1245 {
        recognized_gain_dollars: i64,
        depreciation_dollars: i64,
    },
    LongSection1250 {
        recognized_gain_dollars: i64,
        depreciation_dollars: i64,
        // Actual §179 depreciation, not the smaller amount recaptured.
        // ThirtyNine/rental source branches supply their actual known zero.
        section179_bucket_dollars: i64,
        additional_depreciation_dollars: i64,
    },
    ListedOr179Excess {
        // Source Cents: the original method/convention owns recomputation.
        allowed_cents: i64,
        recomputed_cents: i64,
    },
    InvestmentCredit {
        // Form 4255 native Dollars. The source owns history, full years and rate.
        used_dollars: i64,
        carried_dollars: i64,
        freed_dollars: i64,
        recapture_percent: i64,
        basis_reduction_percent: i64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RecaptureAmounts {
    Disposition {
        section1245_dollars: i64,
        section1250_dollars: i64,
        ordinary_dollars: i64,
        remaining1231_dollars: i64,
        unrecaptured1250_dollars: i64,
    },
    DeductionExcessCents(i64),
    InvestmentCredit {
        tax_dollars: i64,
        carryover_reduction_dollars: i64,
        basis_increase_cents: i64,
    },
}

// Known absence stays absence; unknown sources and unrepresentable calculations
// stay unavailable. The caller's existing SourceFact::require owns its named issue.
pub(crate) fn calculate(input: SourceFact<RecaptureInput>) -> SourceFact<RecaptureAmounts> {
    match input {
        SourceFact::Reported(input) => match calculate_reported(input) {
            Some(amounts) => SourceFact::Reported(amounts),
            None => SourceFact::Unavailable,
        },
        SourceFact::NotReported => SourceFact::NotReported,
        SourceFact::Unavailable => SourceFact::Unavailable,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Disposition {
    pub section1245_dollars: i64,
    pub section1250_dollars: i64,
    pub ordinary_dollars: i64,
    pub remaining1231_dollars: i64,
    pub unrecaptured1250_dollars: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CreditAmounts {
    pub tax_dollars: i64,
    pub carryover_reduction_dollars: i64,
    pub basis_increase_cents: i64,
}

// Native callers retain their original arithmetic failure/wrapping behavior.
// Checked callers use the same formulas with checked operations. Neither path
// adds a source guard or evaluates the other property class's arithmetic.
struct Arithmetic<E> {
    sub: fn(i64, i64) -> Result<i64, E>,
    add: fn(i64, i64) -> Result<i64, E>,
    mul: fn(i64, i64) -> Result<i64, E>,
}
const NATIVE: Arithmetic<core::convert::Infallible> =
    Arithmetic { sub: |a, b| Ok(a - b), add: |a, b| Ok(a + b), mul: |a, b| Ok(a * b) };
const CHECKED: Arithmetic<()> = Arithmetic {
    sub: |a, b| a.checked_sub(b).ok_or(()),
    add: |a, b| a.checked_add(b).ok_or(()),
    mul: |a, b| a.checked_mul(b).ok_or(()),
};
fn native<T>(value: Result<T, core::convert::Infallible>) -> T {
    match value {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

fn section1245_with<E>(
    gain: i64,
    depreciation: i64,
    sub: impl Fn(i64, i64) -> Result<i64, E>,
) -> Result<Disposition, E> {
    let ordinary = phase::cap(gain, [depreciation]);
    Ok(Disposition {
        section1245_dollars: ordinary,
        section1250_dollars: 0,
        ordinary_dollars: ordinary,
        remaining1231_dollars: sub(gain, ordinary)?,
        unrecaptured1250_dollars: 0,
    })
}

fn section1250_with<E>(
    gain: i64,
    depreciation: i64,
    bucket: i64,
    additional: i64,
    sub: impl Fn(i64, i64) -> Result<i64, E>,
    add: impl Fn(i64, i64) -> Result<i64, E>,
) -> Result<Disposition, E> {
    // Exclude the full §179 bucket, even when gain limits its recapture.
    let first = phase::cap(gain, [bucket]);
    let second = phase::cap(sub(gain, first)?, [additional]);
    let ordinary = add(first, second)?;
    let remaining = sub(gain, ordinary)?;
    let unrecaptured = phase::cap(remaining, [sub(sub(depreciation, bucket)?, additional)?]);
    Ok(Disposition {
        section1245_dollars: first,
        section1250_dollars: second,
        ordinary_dollars: ordinary,
        remaining1231_dollars: remaining,
        unrecaptured1250_dollars: unrecaptured,
    })
}

fn excess_with<E>(
    allowed: i64,
    recomputed: i64,
    sub: impl Fn(i64, i64) -> Result<i64, E>,
) -> Result<i64, E> {
    Ok(phase::floor(sub(allowed, recomputed)?, 0))
}

fn credit_with<E>(
    used: i64,
    carried: i64,
    freed: i64,
    percent: i64,
    reduction: i64,
    arithmetic: &Arithmetic<E>,
) -> Result<CreditAmounts, E> {
    // Round each native-Dollar amount before restoring basis in source Cents.
    let tax = fraction(phase::floor((arithmetic.sub)(used, freed)?, 0), percent, 100);
    let carryover = fraction(carried, percent, 100);
    let basis = fraction((arithmetic.mul)((arithmetic.add)(tax, carryover)?, 100)?, reduction, 100);
    Ok(CreditAmounts {
        tax_dollars: tax,
        carryover_reduction_dollars: carryover,
        basis_increase_cents: basis,
    })
}

pub(crate) fn section1245(gain: i64, depreciation: i64) -> Disposition {
    native(section1245_with(gain, depreciation, NATIVE.sub))
}

// The source supplies its actual prepared §179 bucket (already depreciation-
// limited where its source policy requires it), including known zero. Do not
// cap that zero against malformed negative depreciation or add a gain guard.
pub(crate) fn section1250(
    gain: i64,
    depreciation: i64,
    bucket: i64,
    additional: i64,
) -> Disposition {
    native(section1250_with(gain, depreciation, bucket, additional, NATIVE.sub, NATIVE.add))
}

pub(crate) fn deduction_excess(allowed: i64, recomputed: i64) -> i64 {
    native(excess_with(allowed, recomputed, NATIVE.sub))
}

pub(crate) fn investment_credit(
    used: i64,
    carried: i64,
    freed: i64,
    percent: i64,
    reduction: i64,
) -> CreditAmounts {
    native(credit_with(used, carried, freed, percent, reduction, &NATIVE))
}

fn calculate_reported(input: RecaptureInput) -> Option<RecaptureAmounts> {
    let sub = CHECKED.sub;
    let add = CHECKED.add;
    let disposition = match input {
        RecaptureInput::LongSection1245 { recognized_gain_dollars, depreciation_dollars }
            if recognized_gain_dollars > 0 =>
        {
            section1245_with(recognized_gain_dollars, depreciation_dollars, sub).ok()?
        }
        RecaptureInput::LongSection1250 {
            recognized_gain_dollars,
            depreciation_dollars,
            section179_bucket_dollars,
            additional_depreciation_dollars,
        } if recognized_gain_dollars > 0 => section1250_with(
            recognized_gain_dollars,
            depreciation_dollars,
            phase::cap(section179_bucket_dollars, [depreciation_dollars]),
            additional_depreciation_dollars,
            sub,
            add,
        )
        .ok()?,
        RecaptureInput::LongSection1245 { recognized_gain_dollars, .. }
        | RecaptureInput::LongSection1250 { recognized_gain_dollars, .. } => Disposition {
            section1245_dollars: 0,
            section1250_dollars: 0,
            ordinary_dollars: 0,
            remaining1231_dollars: recognized_gain_dollars,
            unrecaptured1250_dollars: 0,
        },
        RecaptureInput::ListedOr179Excess { allowed_cents, recomputed_cents } => {
            return Some(RecaptureAmounts::DeductionExcessCents(
                excess_with(allowed_cents, recomputed_cents, sub).ok()?,
            ));
        }
        RecaptureInput::InvestmentCredit {
            used_dollars,
            carried_dollars,
            freed_dollars,
            recapture_percent,
            basis_reduction_percent,
        } => {
            if !(0..=100).contains(&recapture_percent)
                || !(0..=100).contains(&basis_reduction_percent)
            {
                return None;
            }
            let amounts = credit_with(
                used_dollars,
                carried_dollars,
                freed_dollars,
                recapture_percent,
                basis_reduction_percent,
                &CHECKED,
            )
            .ok()?;
            return Some(RecaptureAmounts::InvestmentCredit {
                tax_dollars: amounts.tax_dollars,
                carryover_reduction_dollars: amounts.carryover_reduction_dollars,
                basis_increase_cents: amounts.basis_increase_cents,
            });
        }
    };
    Some(RecaptureAmounts::Disposition {
        section1245_dollars: disposition.section1245_dollars,
        section1250_dollars: disposition.section1250_dollars,
        ordinary_dollars: disposition.ordinary_dollars,
        remaining1231_dollars: disposition.remaining1231_dollars,
        unrecaptured1250_dollars: disposition.unrecaptured1250_dollars,
    })
}

#[cfg(test)]
#[path = "recapture_tests.rs"]
mod tests;
