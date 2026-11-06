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
    pub opening_section179_carry: Money,
    pub current_section179: Money,
    pub disposed: Option<super::Month>,
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
    // Bonus and regular depreciation, excluding actual selected179.
    pub allowance: Money,
    pub section179: Money,
    pub closing_section179_carry: Money,
    // Original basis after elected179 and nominal bonus, before annual recovery.
    pub original_depreciation_basis: Money,
    // Depreciation basis excludes the separately reserved179 carry.
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

// Original nominal bonus and its actual selected179 share of the automobile
// cap. Both tax systems use this owner, with their own actual179 source.
pub(crate) fn first_bonus(
    basis: &Money,
    elected: &Money,
    percent: u32,
    cap: Money,
    allowed179: Money,
) -> Option<Money> {
    if percent > 100 || allowed179.get() > elected.get() || allowed179.get() > cap.get() {
        return None;
    }
    let nominal =
        ratio::checked_fraction(remaining(basis.get(), elected.get())?, i64::from(percent), 100)?;
    Money::new(phase::cap(nominal, [cap.get() - allowed179.get()])).ok()
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
        let original_depreciation_basis = remaining(remaining(basis, elected)?, nominal_bonus)?;
        let table_basis = if self.safe_harbor {
            remaining(basis, self.first_year_deduction.get())?
        } else {
            original_depreciation_basis
        };
        let rates = self.table.rates();
        let post_year = original_year.checked_add(u32::try_from(rates.len()).ok()?)?;
        let annual = |year: u32, prior: i64, reserve: i64, allowed179: i64| -> Option<i64> {
            if allowed179 > reserve || allowed179 > cap(year)?.get() {
                return None;
            }
            let available = remaining(remaining(basis, prior)?, reserve)?;
            let bonus = if year == original_year {
                first_bonus(
                    &self.basis,
                    &self.original_section179,
                    self.bonus_percent,
                    cap(year)?,
                    Money::new(allowed179).ok()?,
                )?
                .get()
            } else {
                0
            };
            let tentative = if year >= post_year {
                available
            } else if self.safe_harbor && year == original_year {
                0
            } else {
                ratio::checked_fraction(
                    table_basis,
                    rates[(year - original_year) as usize],
                    100000,
                )?
            };
            let tentative = match self.disposed.filter(|date| date.year == year) {
                //1.280F-2T(c)(4): post-recovery unrecovered basis is not
                // deductible in a disposal year (including its Example3).
                Some(_) if year >= post_year => 0,
                Some(date) => self.table.disposal_amount(year, tentative, date),
                None => tentative,
            };
            let regular = phase::cap(
                tentative,
                [remaining(available, bonus)?, cap(year)?.get() - allowed179 - bonus],
            );
            bonus.checked_add(regular)?.checked_add(allowed179)
        };
        let mut prior = 0i64;
        let mut reserve = elected;
        let mut target = None;
        let mut post_basis = None;
        for year in original_year..self.record_end_year {
            let mut rows = self.records.iter().filter(|row| row.year == year);
            let row = rows.next()?;
            let allowed179 = row.section179.get();
            let actual = allowed179.checked_add(row.bonus.get())?.checked_add(row.regular.get())?;
            let expected = annual(year, prior, reserve, allowed179)?;
            let rounded =
                expected.checked_add(50).and_then(|_| ratio::dollars(expected).checked_mul(100));
            if rows.next().is_some()
                || row.bonus.get()
                    != if year == original_year { self.original_bonus.get() } else { 0 }
                || (year == original_year
                    && (actual != self.first_year_deduction.get()
                        || row.bonus.get() != phase::cap(nominal_bonus, [first_cap - allowed179])))
                || (actual != expected && Some(actual) != rounded)
            {
                return None;
            }
            if year == self.target_year {
                target = Some((
                    expected,
                    remaining(remaining(basis, prior)?, reserve)?,
                    allowed179,
                    reserve,
                ));
            }
            reserve = remaining(reserve, allowed179)?;
            prior = prior.checked_add(actual)?;
            remaining(remaining(basis, prior)?, reserve)?;
            if year + 1 == post_year {
                post_basis = Some(remaining(remaining(basis, prior)?, reserve)?);
            }
        }
        if reserve != self.opening_section179_carry.get()
            || prior != self.prior_depreciation.get()
            || (post_year <= self.record_end_year
                && post_basis != Some(self.post_recovery_basis.get()))
        {
            return None;
        }
        let (allowance, opening, selected, target_reserve) =
            if self.target_year == self.record_end_year {
                let selected = self.current_section179.get();
                if self.disposed.is_some() && selected != 0 {
                    return None;
                }
                (
                    annual(self.target_year, prior, reserve, selected)?,
                    remaining(remaining(basis, prior)?, reserve)?,
                    selected,
                    reserve,
                )
            } else {
                target?
            };
        Some(Amounts {
            allowance: Money::new(remaining(allowance, selected)?).ok()?,
            section179: Money::new(selected).ok()?,
            closing_section179_carry: Money::new(
                if self.disposed.is_some_and(|date| date.year == self.target_year) {
                    0
                } else {
                    remaining(target_reserve, selected)?
                },
            )
            .ok()?,
            original_depreciation_basis: Money::new(original_depreciation_basis).ok()?,
            opening_basis: Money::new(opening).ok()?,
            closing_basis: Money::new(
                remaining(opening, remaining(allowance, selected)?)?.checked_add(
                    if self.disposed.is_some_and(|date| date.year == self.target_year) {
                        target_reserve
                    } else {
                        0
                    },
                )?,
            )
            .ok()?,
            post_recovery_basis: post_basis.map(Money::new).transpose().ok()?,
        })
    }
}
