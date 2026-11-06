// Investment-credit property qualification and original source relationships.
// https://www.irs.gov/instructions/i3468
use crate::investment_credit_model::{EnergyProperty, InvestmentCredit, RehabilitationCredit};
use crate::kernel::ratio::{dollars, fraction};
use crate::year_constants::{
    ENERGY_BASE_RATE_PERCENT, ENERGY_BONUS_BASE_PERCENT, ENERGY_BONUS_INCREASED_PERCENT,
    ENERGY_INCREASED_RATE_PERCENT, GEOTHERMAL_HEAT_PUMP_CUTOFF_DATE, REHABILITATION_CREDIT_PERCENT,
    REHABILITATION_PERIOD_MONTHS, REHABILITATION_PERIOD_YEARS, REHABILITATION_PHASED_PERIOD_MONTHS,
    REHABILITATION_RATABLE_PERCENT, REHABILITATION_SUBSTANTIAL_FLOOR_CENTS,
};

// ---- dates -------------------------------------------------------------------------------------

// The last day of a measuring period of `months` months beginning on `start`
// (yyyymmdd): the same day `months` later, less one day (section 47(c)(1)(B)).
pub(crate) fn period_end(start: u32, months: u32) -> u32 {
    let (year, month, day) = (start / 10000, start / 100 % 100, start % 100);
    let total = year * 12 + (month - 1) + months;
    let (year, month) = (total / 12, total % 12 + 1);
    let day = day.min(crate::dates::days_in_month(year, month));
    if day > 1 {
        return year * 10000 + month * 100 + day - 1;
    }
    let (year, month) = if month == 1 { (year - 1, 12) } else { (year, month - 1) };
    crate::dates::last_day_of_month(year, month)
}

// ---- energy credit -------------------------------------------------------------------------------

// The credit percent of the credit base: the base rate or the increased rate
// plus the bonuses, which are 10% with the increased rate and 2% otherwise
// (section 48(a)(12), (14); 48E(a)(3)).
pub(crate) fn energy_percent(increased: bool, domestic: bool, community: bool) -> i64 {
    let (base, bonus) = if increased {
        (ENERGY_INCREASED_RATE_PERCENT, ENERGY_BONUS_INCREASED_PERCENT)
    } else {
        (ENERGY_BASE_RATE_PERCENT, ENERGY_BONUS_BASE_PERCENT)
    };
    base + if domestic { bonus } else { 0 } + if community { bonus } else { 0 }
}

impl InvestmentCredit {
    pub(crate) fn is_some(&self) -> bool {
        !matches!(self, Self::No)
    }

    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::No => true,
            Self::Rehabilitation(claim) => claim.valid(),
            Self::NewEnergy { kind, construction_began, .. } => {
                crate::dates::source_valid_date(*construction_began)
                    && *construction_began >= 20100101
                    && (*kind != EnergyProperty::GeothermalHeatPump
                        || *construction_began < GEOTHERMAL_HEAT_PUMP_CUTOFF_DATE)
            }
            Self::EarlierEnergy { credit, used, carried, freed, .. } => {
                (100..=100_000_000_000).contains(credit)
                    && [used, carried, freed].iter().all(|v| (0..=100_000_000_000).contains(*v))
                    && [credit, used, carried, freed].iter().all(|v| **v % 100 == 0)
            }
        }
    }
}

impl RehabilitationCredit {
    pub(crate) fn valid(&self) -> bool {
        crate::dates::source_valid_date(self.period_start)
            && (0..=100_000_000_000).contains(&self.building_basis)
            && (1..=100_000_000_000).contains(&self.period_expenditures)
            && (0..=100_000_000_000).contains(&self.freed)
            && self.earlier.len() <= 5
            && self.earlier.iter().all(|row| {
                (2021..=2025).contains(&row.year)
                    && (0..=100_000_000_000).contains(&row.used)
                    && (0..=100_000_000_000).contains(&row.carried)
            })
    }

    // The last day of the measuring period.
    pub(crate) fn period_end(&self) -> u32 {
        period_end(
            self.period_start,
            if self.phased {
                REHABILITATION_PHASED_PERIOD_MONTHS
            } else {
                REHABILITATION_PERIOD_MONTHS
            },
        )
    }

    // The relationships the server repeats from the page rules and adds
    // (the measuring period's calendar months, the years of the history and
    // each year's share), for a property placed in service on `placed` whose
    // qualified rehabilitation expenditures are `qre` (cents).
    pub(crate) fn issue(
        &self,
        placed: u32,
        qre: i64,
    ) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if self.period_expenditures
            <= self.building_basis.max(REHABILITATION_SUBSTANTIAL_FLOOR_CENTS)
        {
            return Some(
                ValidationIssue::new("REHABILITATION_NOT_SUBSTANTIAL")
                    .at_field("period_expenditures"),
            );
        }
        if let Some((i, _)) = self
            .earlier
            .iter()
            .enumerate()
            .find(|(i, row)| self.earlier[..*i].iter().any(|p| p.year == row.year))
        {
            return Some(
                ValidationIssue::new("INVESTMENT_CREDIT_YEAR_TWICE")
                    .at_field("year")
                    .in_list("earlier", i),
            );
        }
        if self.period_end() / 10000 != placed / 10000 {
            return Some(
                ValidationIssue::new("REHABILITATION_PERIOD_NOT_IN_SERVICE_YEAR")
                    .at_field("period_start"),
            );
        }
        let first = placed / 10000;
        if let Some((i, _)) =
            self.earlier.iter().enumerate().find(|(_, row)| {
                row.year < first || row.year >= first + REHABILITATION_PERIOD_YEARS
            })
        {
            return Some(
                ValidationIssue::new("INVESTMENT_CREDIT_YEAR_OUTSIDE_PERIOD")
                    .at_field("year")
                    .in_list("earlier", i),
            );
        }
        let share = dollars(fraction(qre, REHABILITATION_RATABLE_PERCENT, 100)) * 100;
        if let Some((i, _)) = self
            .earlier
            .iter()
            .enumerate()
            .find(|(_, row)| i128::from(row.used) + i128::from(row.carried) > i128::from(share))
        {
            return Some(
                ValidationIssue::new("INVESTMENT_CREDIT_HISTORY_ABOVE_CREDIT")
                    .at_field("carried")
                    .in_list("earlier", i),
            );
        }
        None
    }
    pub(crate) fn rule(&self, placed: u32, qre: i64) -> Option<&'static str> {
        self.issue(placed, qre).map(|issue| issue.key)
    }

    // The credit determined for the placed-in-service year (20% of the
    // expenditures), cents.
    pub(crate) fn determined(qre: i64) -> i64 {
        fraction(qre, REHABILITATION_CREDIT_PERCENT, 100)
    }
}
