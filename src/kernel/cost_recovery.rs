// Additive Phase A owner; existing callers migrate in Phase B.
#![allow(dead_code)]

use super::ratio::fraction;

pub(crate) mod annual;
pub(crate) mod gds;
pub(crate) mod historical;
pub(crate) mod macrs_tables;
pub(crate) mod monthly_history;

// These schedules operate on real calendar months, not a fabricated service day.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Month {
    pub year: u32,
    pub month: u32,
}
impl Month {
    fn valid(self) -> bool {
        // Source DateOnly encoding has a four-digit Gregorian year and twelve months.
        (1..=9999).contains(&self.year) && (1..=12).contains(&self.month)
    }
    fn index(self) -> i64 {
        i64::from(self.year) * 12 + i64::from(self.month) - 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AmortizationRegime {
    Section195,
    Section197,
    Section248,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MonthlyClock {
    Original { regime: AmortizationRegime, started: Month, period_months: u32 },
    // Later section197 basis uses the original remaining period, not a new one.
    Section197Addition { original_started: Month, original_period_months: u32, included: Month },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MonthlySchedule {
    pub clock: MonthlyClock,
    pub tax_year: u32,
    // Actual amortizable source Cents, after the source's initial-expense stage.
    pub basis_cents: i64,
    // Actual current initial deduction, separately supplied by its owning stage.
    // This engine does not determine eligibility, elections or initial phaseout.
    pub initial_cents: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MonthlyAmounts {
    pub initial_cents: i64,
    pub current_allowable_cents: i64,
    pub prior_allowable_cents: i64,
    pub cumulative_allowable_cents: i64,
    pub statutory_remaining_cents: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct InvalidMonthlySchedule;

impl MonthlySchedule {
    // Native section 195/197 callers already own full DateOnly and amount
    // validation. A fixed statutory period keeps their infallible calculation
    // boundary without turning a checked error into a panic or default amount.
    pub(crate) fn original_180(
        regime: AmortizationRegime,
        started: Month,
        tax_year: u32,
        basis_cents: i64,
        initial_cents: i64,
    ) -> MonthlyAmounts {
        let schedule = Self {
            clock: MonthlyClock::Original { regime, started, period_months: 180 },
            tax_year,
            basis_cents,
            initial_cents,
        };
        schedule.amounts()
    }

    pub(crate) fn section197_addition_180(
        original_started: Month,
        included: Month,
        tax_year: u32,
        basis_cents: i64,
    ) -> MonthlyAmounts {
        let schedule = Self {
            clock: MonthlyClock::Section197Addition {
                original_started,
                original_period_months: 180,
                included,
            },
            tax_year,
            basis_cents,
            initial_cents: 0,
        };
        schedule.amounts()
    }

    // Statutory normal-schedule allowable amounts only. These are not reported
    // actual prior deductions. Actual-history adjustments remain a separate
    // engine component; no rejection, catch-up or cap is inferred here.
    pub(crate) fn allowable(self) -> Result<MonthlyAmounts, InvalidMonthlySchedule> {
        // Keep that source calendar range; eligible basis and initial expense are nonnegative.
        if !(1..=9999).contains(&self.tax_year) || self.basis_cents < 0 || self.initial_cents < 0 {
            return Err(InvalidMonthlySchedule);
        }
        // A positive recovery period is required for the fraction's divisor.
        match self.clock {
            MonthlyClock::Original { started, period_months, .. }
                if started.valid() && period_months > 0 => {}
            MonthlyClock::Section197Addition {
                original_started,
                original_period_months,
                included,
            } if original_started.valid() && included.valid() && original_period_months > 0 => {}
            _ => return Err(InvalidMonthlySchedule),
        }
        Ok(self.amounts())
    }

    // All entry paths supply a positive period: checked schedules validate it,
    // and native statutory schedules construct it as 180 months.
    fn amounts(self) -> MonthlyAmounts {
        let (started, period) = match self.clock {
            MonthlyClock::Original { started, period_months, .. } => {
                (started.index(), i64::from(period_months))
            }
            MonthlyClock::Section197Addition {
                original_started,
                original_period_months,
                included,
            } => {
                let original = original_started.index();
                let effective = included.index().max(original);
                (effective, (i64::from(original_period_months) - (effective - original)).max(1))
            }
        };
        // Preserve the native cumulative fraction difference, including its
        // cent rounding. Rounding this year's fraction separately differs.
        let cumulative = |year: i64| {
            let months = (year * 12 + 12 - started).clamp(0, period);
            fraction(self.basis_cents, months, period)
        };
        let prior = cumulative(i64::from(self.tax_year) - 1);
        let current = cumulative(i64::from(self.tax_year));
        MonthlyAmounts {
            initial_cents: self.initial_cents,
            current_allowable_cents: current - prior,
            prior_allowable_cents: prior,
            cumulative_allowable_cents: current,
            statutory_remaining_cents: self.basis_cents - current,
        }
    }
}

#[cfg(test)]
#[path = "cost_recovery_tests.rs"]
mod tests;
