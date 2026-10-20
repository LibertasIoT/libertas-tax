// Equipment annual schedules. Source eligibility, selected tax system,
// bonus/179 elections and 280F limits remain with their existing owners.
use super::{Month, macrs_tables};
use crate::kernel::{phase, ratio, source_fact::SourceFact};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Period {
    Three,
    Five,
    Seven,
    Ten,
    Fifteen,
    Twenty,
}
impl Period {
    fn index(self) -> usize {
        match self {
            Self::Three => 0,
            Self::Five => 1,
            Self::Seven => 2,
            Self::Ten => 3,
            Self::Fifteen => 4,
            Self::Twenty => 5,
        }
    }
    fn years(self) -> i64 {
        [3, 5, 7, 10, 15, 20][self.index()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Method {
    Regular,
    Declining150,
    // The source may select effective SL for ADS. No ADS eligibility is inferred.
    StraightLine,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Convention {
    HalfYear,
    MidQuarter,
}
// Published table selection shared by native and capped actual-history schedules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Table {
    pub placed: Month,
    pub period: Period,
    pub method: Method,
    pub convention: Convention,
}
impl Table {
    pub(crate) fn rates(self) -> &'static [i64] {
        let method = match self.method {
            Method::Regular => 0,
            Method::Declining150 => 1,
            Method::StraightLine => 2,
        };
        let convention = match self.convention {
            Convention::HalfYear => 0,
            Convention::MidQuarter => 1 + (self.placed.month.saturating_sub(1) / 3).min(3) as usize,
        };
        macrs_tables::RATES[method * 30 + convention * 6 + self.period.index()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Approach {
    Tables,
    Manual,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum History {
    New {
        // The election, including any income-limit carry; not final allowed179.
        elected179_cents: i64,
    },
    Continuing {
        prior_depreciation_cents: i64,
        carryforward179_cents: i64,
        initial179_cents: i64,
        initial_bonus_cents: i64,
    },
}

// Source-qualified continuation, not a new placed-in-service schedule. Amounts
// are full-use cents, before the current 179 carry/raw bonus and use/disposal
// stages. Availability remains owned by calculate(SourceFact<Schedule>).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AdjustedBasisContinuation {
    pub effective_year: u32,
    pub adjusted_basis_cents: i64,
    // Actual cumulative depreciation before effective_year, on the same basis
    // as History::Continuing. Only depreciation after this anchor is deducted.
    pub prior_depreciation_cents: i64,
    // Source-qualified fixed declining rate on the adjusted opening balance,
    // e.g. Pub225 Brown's printed23.07%, not an inferred exact150%/6.5.
    // Required for declining methods; unused for StraightLine.
    pub declining_rate: Option<(i64, i64)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Schedule {
    pub placed: Month,
    pub tax_year: u32,
    pub period: Period,
    pub method: Method,
    pub convention: Convention,
    pub approach: Approach,
    // Actual history_basis(), which can differ from current business basis.
    pub history_basis_cents: i64,
    pub history: History,
    // Tentative raw_bonus_amount(), BEFORE any passenger/280F limitation.
    pub raw_bonus_cents: i64,
    // Actual current/earlier business-use fraction; applied AFTER full-year
    // rounding and BEFORE disposition rounding, just as the native source.
    // This source's fall-in-use fraction cannot exceed one.
    pub current_use_fraction: Option<(u32, u32)>,
    pub disposed: Option<Month>,
    // None establishes no continuation, never missing source facts. Callers
    // keep needed unknown facts in the outer SourceFact::Unavailable. Future
    // adjustments do not apply.
    pub continuation: Option<AdjustedBasisContinuation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Amounts {
    // Opening history basis less tentative raw bonus, before current MACRS.
    // This is NOT final remaining basis after business-use/280F limitations.
    pub basis_after_raw_bonus_cents: i64,
    // The native full-year helper when evaluated; same-year disposal returns
    // zero BEFORE that helper, not a hypothetical undisposed annual allowance.
    pub full_year_cents: i64,
    pub current_macrs_cents: i64,
}

pub(crate) fn calculate(source: SourceFact<Schedule>) -> SourceFact<Amounts> {
    match source {
        SourceFact::Reported(schedule) => {
            schedule.amounts().map_or(SourceFact::Unavailable, SourceFact::Reported)
        }
        SourceFact::NotReported => SourceFact::NotReported,
        SourceFact::Unavailable => SourceFact::Unavailable,
    }
}

impl Schedule {
    // Remaining-period rate for a source-qualified basis adjustment,
    // anchored to effective_year rather than the year later being calculated.
    // Pub225 Brown / Pub946 without tables: round the reciprocal to four
    // decimals, then multiply by the method factor and round to four decimals.
    // An explicitly supplied source rate remains a separate input.
    pub(crate) fn continuation_declining_rate(self, effective_year: u32) -> Option<(i64, i64)> {
        if !self.placed.valid()
            || !(1..=9999).contains(&effective_year)
            || effective_year <= self.placed.year
            || self.method == Method::StraightLine
        {
            return None;
        }
        let native = NativeSchedule {
            placed: self.placed,
            tax_year: effective_year,
            period: self.period,
            method: self.method,
            convention: self.convention,
            table_basis_cents: None,
            basis_after_raw_bonus_cents: 0,
            continuing: true,
            current_use_fraction: None,
            disposed: None,
        };
        let remaining = native.remaining_eighths();
        if remaining <= 0 {
            return None;
        }
        let reciprocal = ratio::checked_fraction(10000, 8, remaining)?;
        let rate = ratio::checked_fraction(reciprocal, native.factor(), 100)?;
        Some((rate, 10000))
    }

    fn amounts(self) -> Option<Amounts> {
        // These are the source calendar/basis requirements, not amount caps.
        if !self.placed.valid()
            || !(1..=9999).contains(&self.tax_year)
            || self.tax_year < self.placed.year
            || self.history_basis_cents < 0
            || self.raw_bonus_cents < 0
            || self.disposed.is_some_and(|date| {
                !date.valid() || date.year != self.tax_year || date.index() < self.placed.index()
            })
        {
            return None;
        }
        let (opening, original_deductions, new) = match self.history {
            History::New { elected179_cents }
                if elected179_cents >= 0 && self.placed.year == self.tax_year =>
            {
                (
                    self.history_basis_cents.checked_sub(elected179_cents)?,
                    elected179_cents.checked_add(self.raw_bonus_cents)?,
                    true,
                )
            }
            History::Continuing {
                prior_depreciation_cents,
                carryforward179_cents,
                initial179_cents,
                initial_bonus_cents,
            } if [
                prior_depreciation_cents,
                carryforward179_cents,
                initial179_cents,
                initial_bonus_cents,
            ]
            .iter()
            .all(|value| *value >= 0)
                && self.placed.year < self.tax_year =>
            {
                (
                    self.history_basis_cents.checked_sub(
                        prior_depreciation_cents.checked_add(carryforward179_cents)?,
                    )?,
                    initial179_cents.checked_add(initial_bonus_cents)?,
                    false,
                )
            }
            _ => return None,
        };
        let basis = opening.checked_sub(self.raw_bonus_cents)?;
        if basis < 0 || original_deductions > self.history_basis_cents {
            return None;
        }
        // The source's raw-bonus stage must actually supply zero for this
        // native early-return case. Never replace inconsistent source facts.
        if new && self.disposed.is_some() && self.raw_bonus_cents != 0 {
            return None;
        }
        if self.current_use_fraction.is_some_and(|(n, d)| d == 0 || n > d) {
            return None;
        }
        let native = NativeSchedule {
            placed: self.placed,
            tax_year: self.tax_year,
            period: self.period,
            method: self.method,
            convention: self.convention,
            table_basis_cents: (self.approach == Approach::Tables)
                .then_some((self.history_basis_cents, original_deductions)),
            basis_after_raw_bonus_cents: basis,
            continuing: !new,
            current_use_fraction: self.current_use_fraction,
            disposed: self.disposed,
        };
        let Some(continuation) = self.continuation.filter(|c| self.tax_year >= c.effective_year)
        else {
            return Some(native.amounts());
        };
        let History::Continuing { prior_depreciation_cents, carryforward179_cents, .. } =
            self.history
        else {
            return None;
        };
        if continuation.effective_year <= self.placed.year
            || continuation.adjusted_basis_cents < 0
            || continuation.prior_depreciation_cents < 0
            || continuation.prior_depreciation_cents > prior_depreciation_cents
            || (self.tax_year == continuation.effective_year
                && continuation.prior_depreciation_cents != prior_depreciation_cents)
            || continuation.adjusted_basis_cents
                > self.history_basis_cents.checked_sub(continuation.prior_depreciation_cents)?
        {
            return None;
        }
        let remaining = continuation
            .adjusted_basis_cents
            .checked_sub(prior_depreciation_cents - continuation.prior_depreciation_cents)?
            .checked_sub(carryforward179_cents)?
            .checked_sub(self.raw_bonus_cents)?;
        if remaining < 0 {
            return None;
        }
        let native = NativeSchedule {
            table_basis_cents: None,
            basis_after_raw_bonus_cents: remaining,
            ..native
        };
        let straight = NativeSchedule { method: Method::StraightLine, ..native }.full_year();
        let full = if remaining == 0 || native.remaining_eighths() <= 0 {
            0
        } else if native.remaining_eighths() <= 8 || self.method == Method::StraightLine {
            straight
        } else {
            let (numerator, denominator) = continuation.declining_rate?;
            if numerator < 0 || denominator <= 0 {
                return None;
            }
            phase::cap(
                phase::floor(straight, ratio::checked_fraction(remaining, numerator, denominator)?),
                [remaining],
            )
        };
        Some(native.amounts_after_full_year(full))
    }
}

// Native callers also evaluate unvalidated editor/preview inputs. This entry
// preserves their signed arithmetic, calendar normalization and division errors;
// the checked SourceFact entry above retains its separate availability contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct NativeSchedule {
    pub placed: Month,
    pub tax_year: u32,
    pub period: Period,
    pub method: Method,
    pub convention: Convention,
    // None selects manual. Some carries original history basis and the full
    // original179-plus-bonus deductions separately: their subtraction must
    // remain lazy, after the out-of-range and final-year table early returns.
    pub table_basis_cents: Option<(i64, i64)>,
    pub basis_after_raw_bonus_cents: i64,
    pub continuing: bool,
    pub current_use_fraction: Option<(u32, u32)>,
    pub disposed: Option<Month>,
}

impl NativeSchedule {
    fn quarter(self) -> i64 {
        i64::from(self.placed.month.saturating_sub(1) / 3).min(3)
    }
    fn first_eighths(self) -> i64 {
        match self.convention {
            Convention::HalfYear => 4,
            Convention::MidQuarter => 7 - 2 * self.quarter(),
        }
    }
    fn remaining_eighths(self) -> i64 {
        self.period.years() * 8
            - self.first_eighths()
            - (i64::from(self.tax_year) - i64::from(self.placed.year) - 1) * 8
    }
    fn factor(self) -> i64 {
        match self.method {
            Method::Regular if self.period.years() <= 10 => 200,
            Method::Regular | Method::Declining150 => 150,
            Method::StraightLine => 100,
        }
    }
    fn rates(self) -> &'static [i64] {
        Table {
            placed: self.placed,
            period: self.period,
            method: self.method,
            convention: self.convention,
        }
        .rates()
    }
    pub(crate) fn full_year(self) -> i64 {
        let basis = self.basis_after_raw_bonus_cents;
        if let Some((original_basis, initial_deductions)) = self.table_basis_cents {
            let rates = self.rates();
            let year = self.tax_year.saturating_sub(self.placed.year) as usize;
            return match rates.get(year) {
                None => 0,
                Some(_) if year + 1 == rates.len() => basis,
                Some(rate) => phase::cap(
                    ratio::fraction(original_basis - initial_deductions, *rate, 100000),
                    [basis],
                ),
            };
        }
        let years = self.period.years();
        let factor = self.factor();
        if self.continuing {
            let remaining = self.remaining_eighths();
            if remaining <= 0 {
                return 0;
            }
            let straight = ratio::fraction(basis, 8, remaining.max(8));
            let declining = ratio::fraction(basis, factor, 100 * years);
            return if self.method == Method::StraightLine {
                straight
            } else {
                phase::cap(phase::floor(straight, declining), [basis])
            };
        }
        ratio::fraction(basis, factor * self.first_eighths(), 800 * years)
    }
    pub(crate) fn amounts(self) -> Amounts {
        let full = if !self.continuing && self.disposed.is_some() { 0 } else { self.full_year() };
        self.amounts_after_full_year(full)
    }
    fn amounts_after_full_year(self, full: i64) -> Amounts {
        let mut current = full;
        if let Some((numerator, denominator)) = self.current_use_fraction {
            current = ratio::fraction(current, i64::from(numerator), i64::from(denominator));
        }
        if self.continuing
            && let Some(date) = self.disposed
        {
            let final_year = if self.table_basis_cents.is_some() {
                self.tax_year.saturating_sub(self.placed.year) as usize + 1 >= self.rates().len()
            } else {
                self.remaining_eighths() <= 8
            };
            // Keep the native final-year full-amount early returns.
            current = match self.convention {
                Convention::HalfYear if final_year => current,
                Convention::HalfYear => ratio::fraction(current, 12, 24),
                Convention::MidQuarter => {
                    let half_months = 3 + 6 * i64::from((date.month.clamp(1, 12) - 1) / 3);
                    let last = 24 - (21 - 6 * self.quarter());
                    if final_year && half_months >= last {
                        current
                    } else {
                        ratio::fraction(current, half_months, if final_year { last } else { 24 })
                    }
                }
            };
        }
        Amounts {
            basis_after_raw_bonus_cents: self.basis_after_raw_bonus_cents,
            full_year_cents: full,
            current_macrs_cents: current,
        }
    }
}

#[cfg(test)]
#[path = "gds_tests.rs"]
mod tests;
