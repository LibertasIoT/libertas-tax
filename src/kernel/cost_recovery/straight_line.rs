// Calendar-year statutory straight-line recovery with actual source history.
// Pub 946, "Straight Line Method" and "Excepted Property":
// https://www.irs.gov/publications/p946
// Source owners select the legal life/convention, establish eligibility and
// supply actual allowed/allowable prior recovery, not a reconstructed schedule.
use super::Month;
use crate::kernel::{
    carry::{self, Amount, Cents},
    ratio::exact::{self, Exact},
    source_fact::SourceFact,
};

type Money = Amount<Cents>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Convention {
    HalfYear,
    MidQuarter,
    MidMonth,
}
impl Convention {
    fn point(self, month: Month) -> i64 {
        let offset = match self {
            Self::HalfYear => 12,
            Self::MidQuarter => 3 + 6 * i64::from((month.month - 1) / 3),
            Self::MidMonth => 2 * (month.index() % 12) + 1,
        };
        i64::from(month.year) * 24 + offset
    }
}

pub(crate) struct Schedule {
    pub placed: Month,
    // Positive exact statutory life in half-months; a month-based life supplies
    // twice its months. No fixed GDS class or ADS eligibility is inferred here.
    pub period_half_months: Exact,
    pub convention: Convention,
    pub tax_year: u32,
    // Original recoverable basis AFTER actual bonus and other basis reductions.
    pub basis: Money,
    pub prior_recovery: SourceFact<Money>,
    pub disposed: Option<Month>,
}

pub(crate) struct Amounts {
    pub current: Money,
    pub opening_basis: Money,
    // Unrecovered basis, including after disposal; disposal does not deduct it.
    pub closing_basis: Money,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InvalidSchedule {
    Date,
    Period,
    PriorRecovery,
}

impl Schedule {
    pub(crate) fn recover(self) -> Result<SourceFact<Amounts>, InvalidSchedule> {
        if !self.placed.valid()
            || !(1..=9999).contains(&self.tax_year)
            || self.disposed.is_some_and(|d| !d.valid() || d.index() < self.placed.index())
        {
            return Err(InvalidSchedule::Date);
        }
        if self.period_half_months <= exact::whole(0) {
            return Err(InvalidSchedule::Period);
        }
        let prior = self.prior_recovery.check_reported(|amount| {
            if amount.get() > self.basis.get()
                || (self.tax_year <= self.placed.year && amount.get() != 0)
            {
                Err(InvalidSchedule::PriorRecovery)
            } else {
                Ok(())
            }
        })?;
        // NotReported is documented absence of prior recovery, hence zero.
        // Unavailable is unknown even for a zero basis or inactive year.
        let prior = match prior {
            SourceFact::Reported(amount) => amount.get(),
            SourceFact::NotReported => 0,
            SourceFact::Unavailable => return Ok(SourceFact::Unavailable),
        };
        let opening = carry::closing_values::<Cents>(self.basis.get(), prior)
            .map_err(|_| InvalidSchedule::PriorRecovery)?;
        let start = self.convention.point(self.placed);
        let end = exact::whole(start) + self.period_half_months;
        let year_start = i64::from(self.tax_year) * 24;
        let first = exact::whole(year_start.max(start));
        let last = exact::whole(year_start + 24).min(end.clone()).min(
            self.disposed.map_or_else(|| end.clone(), |d| exact::whole(self.convention.point(d))),
        );
        let same_year_disposal = self.disposed.is_some_and(|d| d.year == self.placed.year);
        let current = if same_year_disposal || last <= first {
            0
        } else if last == end {
            // Held through the recovery endpoint: deduct unrecovered basis.
            opening.get()
        } else {
            // Adjusted basis divided by remaining statutory life, using the
            // convention's partial year. Combine fractions before the single
            // cent projection. The annual SL rate cannot exceed 100%; an
            // early disposal in the final year still applies its convention
            // to that annual rate. No post-window catch-up is created.
            let remaining = if self.tax_year == self.placed.year {
                end - &first
            } else {
                (end - &first).max(exact::whole(24))
            };
            exact::rounded(&(exact::whole(opening.get()) * (last - first) / remaining))
                .ok_or(InvalidSchedule::Period)?
        };
        let settled = carry::settle(opening, Money::new(current).unwrap(), false)
            .map_err(|_| InvalidSchedule::PriorRecovery)?;
        Ok(SourceFact::Reported(Amounts {
            opening_basis: Money::new(self.basis.get() - prior).unwrap(),
            current: settled.used,
            closing_basis: settled.closing,
        }))
    }
}

#[cfg(test)]
#[path = "straight_line_tests.rs"]
mod tests;
