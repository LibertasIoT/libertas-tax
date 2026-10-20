// Capped original annual schedules with actual admitted history. Source owners
// establish eligibility, stable use, unadjusted records and statutory caps.
use super::gds::Table;
use crate::kernel::{
    carry::{self, Amount, Cents},
    phase, ratio,
    source_fact::SourceFact,
};

type Money = Amount<Cents>;

pub(crate) struct Record {
    pub year: u32,
    pub section179: Money,
    pub bonus: Money,
    pub regular: Money,
}

pub(crate) struct Schedule<'a> {
    pub table: Table,
    pub target_year: u32,
    // Exclusive: one actual record for every original year before this year.
    pub record_end_year: u32,
    pub basis: Money,
    pub original_section179: Money,
    // Applicable original nominal rate, selected by the source's tax rules.
    pub bonus_percent: u32,
    pub safe_harbor: bool,
    pub first_year_deduction: Money,
    pub original_bonus: Money,
    pub records: &'a [Record],
    pub prior_depreciation: Money,
    pub post_recovery_basis: Money,
}

pub(crate) struct Amounts {
    pub allowance: Money,
    pub opening_basis: Money,
    // Projected target closing basis; actual earlier rows remain unchanged.
    pub closing_basis: Money,
    // Actual basis entering the first post-table year, when records reach it.
    pub post_recovery_basis: Option<Money>,
}

pub(crate) fn calculate(
    source: SourceFact<Schedule<'_>>,
    cap: impl Fn(u32) -> Option<Money>,
) -> SourceFact<Amounts> {
    match source {
        SourceFact::Reported(schedule) => {
            schedule.amounts(cap).map_or(SourceFact::Unavailable, SourceFact::Reported)
        }
        SourceFact::NotReported => SourceFact::NotReported,
        SourceFact::Unavailable => SourceFact::Unavailable,
    }
}

fn remaining(opening: i64, used: i64) -> Option<i64> {
    carry::settle(Money::new(opening).ok()?, Money::new(used).ok()?, false)
        .ok()
        .map(|amounts| amounts.closing.get())
}

impl Schedule<'_> {
    fn amounts(self, cap: impl Fn(u32) -> Option<Money>) -> Option<Amounts> {
        let original_year = self.table.placed.year;
        if !self.table.placed.valid()
            || !(original_year..=9999).contains(&self.record_end_year)
            || !(original_year..=self.record_end_year).contains(&self.target_year)
            || self.records.len() != (self.record_end_year - original_year) as usize
            || self.bonus_percent > 100
        {
            return None;
        }
        let basis = self.basis.get();
        let elected = self.original_section179.get();
        let nominal_bonus = ratio::checked_fraction(
            remaining(basis, elected)?,
            i64::from(self.bonus_percent),
            100,
        )?;
        let first_cap = cap(original_year)?.get();
        if elected > first_cap || (self.safe_harbor && basis <= first_cap) {
            return None;
        }
        let table_basis = if self.safe_harbor {
            remaining(basis, self.first_year_deduction.get())?
        } else {
            remaining(remaining(basis, elected)?, nominal_bonus)?
        };
        let rates = self.table.rates();
        let post_year = original_year.checked_add(u32::try_from(rates.len()).ok()?)?;
        let first_bonus = phase::cap(nominal_bonus, [first_cap - elected]);
        let first_regular = if self.safe_harbor {
            0
        } else {
            phase::cap(
                ratio::checked_fraction(table_basis, rates[0], 100000)?,
                [first_cap - elected - first_bonus],
            )
        };
        let first_total = elected.checked_add(first_bonus)?.checked_add(first_regular)?;
        if self.first_year_deduction.get() != first_total
            || self.original_bonus.get() != first_bonus
        {
            return None;
        }
        let annual = |year, prior| -> Option<i64> {
            let available = remaining(basis, prior)?;
            if year == original_year {
                return Some(first_total);
            }
            let tentative = if year >= post_year {
                available
            } else {
                ratio::checked_fraction(
                    table_basis,
                    rates[(year - original_year) as usize],
                    100000,
                )?
            };
            Some(phase::cap(tentative, [available, cap(year)?.get()]))
        };
        let mut prior = 0i64;
        let mut target = None;
        let mut post_basis = None;
        for year in original_year..self.record_end_year {
            if year == self.target_year {
                target = Some((annual(year, prior)?, remaining(basis, prior)?));
            }
            let mut rows = self.records.iter().filter(|row| row.year == year);
            let row = rows.next()?;
            let actual = row
                .section179
                .get()
                .checked_add(row.bonus.get())?
                .checked_add(row.regular.get())?;
            let expected = annual(year, prior)?;
            // A retained annual return may use exact cents or its whole-dollar
            // rounding. Fold the actual amount, preserving the historical cent.
            let rounded =
                expected.checked_add(50).and_then(|_| ratio::dollars(expected).checked_mul(100));
            if rows.next().is_some()
                || row.section179.get() != if year == original_year { elected } else { 0 }
                || row.bonus.get() != if year == original_year { first_bonus } else { 0 }
                || (actual != expected && Some(actual) != rounded)
            {
                return None;
            }
            prior = prior.checked_add(actual)?;
            remaining(basis, prior)?;
            if year + 1 == post_year {
                post_basis = Some(remaining(basis, prior)?);
            }
        }
        if prior != self.prior_depreciation.get()
            || (post_year <= self.record_end_year
                && post_basis != Some(self.post_recovery_basis.get()))
        {
            return None;
        }
        let (allowance, opening) = if self.target_year == self.record_end_year {
            (annual(self.target_year, prior)?, remaining(basis, prior)?)
        } else {
            target?
        };
        Some(Amounts {
            allowance: Money::new(allowance).ok()?,
            opening_basis: Money::new(opening).ok()?,
            closing_basis: Money::new(remaining(opening, allowance)?).ok()?,
            post_recovery_basis: post_basis.map(Money::new).transpose().ok()?,
        })
    }
}
