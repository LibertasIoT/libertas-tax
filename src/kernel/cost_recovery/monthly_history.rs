// Actual source-history recovery, distinct from MonthlySchedule's rounded
// cumulative statutory allowances. Eligibility and original legal dates are
// supplied by the App. Never reconstruct a missing prior deduction here.
use super::{InvalidMonthlySchedule, Month};
use crate::kernel::ratio::fraction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActualMonthlySchedule {
    pub started: Month,
    pub period_months: u32,
    pub tax_year: u32,
    pub basis_cents: i64,
    pub prior_recovery_cents: Option<i64>,
    // Ordinary disposal includes its month; section 169 termination excludes it.
    pub allowed_through: Option<Month>,
    pub allowed_before: Option<Month>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActualMonthlyAmounts {
    pub current_cents: i64,
    pub remaining_cents: i64,
}

impl ActualMonthlySchedule {
    pub(crate) fn recover(self) -> Result<ActualMonthlyAmounts, InvalidMonthlySchedule> {
        let prior = self.prior_recovery_cents.ok_or(InvalidMonthlySchedule)?;
        if !self.started.valid()
            || !(1..=9999).contains(&self.tax_year)
            || self.period_months == 0
            || self.basis_cents < 0
            || !(0..=self.basis_cents).contains(&prior)
            || self.allowed_through.is_some_and(|month| !month.valid())
            || self.allowed_before.is_some_and(|month| !month.valid())
        {
            return Err(InvalidMonthlySchedule);
        }
        let first = self.started.index().max(i64::from(self.tax_year) * 12);
        let mut last = (i64::from(self.tax_year) * 12 + 12)
            .min(self.started.index() + i64::from(self.period_months));
        if let Some(month) = self.allowed_through {
            last = last.min(month.index() + 1);
        }
        if let Some(month) = self.allowed_before {
            last = last.min(month.index());
        }
        let remaining = self.basis_cents - prior;
        let current =
            fraction(self.basis_cents, (last - first).max(0), i64::from(self.period_months))
                .min(remaining);
        Ok(ActualMonthlyAmounts { current_cents: current, remaining_cents: remaining - current })
    }
}

// Section 194's actual original elections partition qualified costs. Neither
// today's filing status nor unused expense room creates an amortization election.
// Source: IRC 194(a), (b), (c)(3); original election/date eligibility is App-owned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TimberElection {
    pub capital_cost_cents: i64,
    pub excluded_reimbursement_cents: i64,
    pub original_expense_cents: i64,
    pub elected_basis_cents: i64,
    pub prior_amortization_cents: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TimberRecovery {
    pub current_expense_cents: i64,
    pub elected_basis_cents: i64,
    pub amortization_cents: i64,
    pub remaining_cents: i64,
}

impl TimberElection {
    // The App selects the source year's filing-status/property limit and the
    // actual other-source elections. No excess becomes a new basis election.
    pub(crate) fn expense_within_property_limit(self, other_cents: i64, limit_cents: i64) -> bool {
        other_cents >= 0
            && limit_cents >= 0
            && self.original_expense_cents >= 0
            && i128::from(self.original_expense_cents) + i128::from(other_cents)
                <= i128::from(limit_cents)
    }

    pub(crate) fn validate_basis(self) -> Result<(), InvalidMonthlySchedule> {
        if self.capital_cost_cents < 0
            || !(0..=self.capital_cost_cents).contains(&self.excluded_reimbursement_cents)
        {
            return Err(InvalidMonthlySchedule);
        }
        let qualified = self.capital_cost_cents - self.excluded_reimbursement_cents;
        if !(0..=qualified).contains(&self.original_expense_cents)
            || !(0..=qualified - self.original_expense_cents).contains(&self.elected_basis_cents)
            || self
                .prior_amortization_cents
                .is_some_and(|prior| !(0..=self.elected_basis_cents).contains(&prior))
        {
            return Err(InvalidMonthlySchedule);
        }
        Ok(())
    }

    pub(crate) fn recover(
        self,
        incurred_year: u32,
        tax_year: u32,
        allowed_through: Option<Month>,
    ) -> Result<TimberRecovery, InvalidMonthlySchedule> {
        self.validate_basis()?;
        if incurred_year > tax_year {
            return Err(InvalidMonthlySchedule);
        }
        let qualified = self.capital_cost_cents - self.excluded_reimbursement_cents;
        // IRC 194(a): seven years, beginning at the midpoint of the cost year.
        let recovery = ActualMonthlySchedule {
            started: Month { year: incurred_year, month: 7 },
            period_months: 84,
            tax_year,
            basis_cents: self.elected_basis_cents,
            prior_recovery_cents: self.prior_amortization_cents,
            allowed_through,
            allowed_before: None,
        }
        .recover()?;
        Ok(TimberRecovery {
            current_expense_cents: if incurred_year == tax_year {
                self.original_expense_cents
            } else {
                0
            },
            elected_basis_cents: self.elected_basis_cents,
            amortization_cents: recovery.current_cents,
            remaining_cents: qualified - self.original_expense_cents - self.elected_basis_cents
                + recovery.remaining_cents,
        })
    }
}
