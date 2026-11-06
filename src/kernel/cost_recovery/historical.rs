// MACRS rental building schedules and shared published 39-year rates.
use crate::kernel::{phase, ratio};

// Native source adapters retain date validity and original recovery selection;
// this engine retains exact schedules and admitted annual cents.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OfficeRecoveryIssue {
    InvalidDate,
    InvalidBasis,
    OriginalMethodDateMismatch,
}

/// One exact source deduction, before the activity's current-use allocation
/// and section280A limit. The consumer chooses its final rounding unit once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfficeRecoveryDeduction {
    exact_cents: ratio::exact::Exact,
}
impl OfficeRecoveryDeduction {
    pub fn rounded_cents(&self) -> i64 {
        ratio::exact::rounded(&self.exact_cents).expect("bounded original property basis")
    }
}

// One MACRS building schedule owner for rental consumers.
// Source selection and original-history completeness remain with each consumer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RentalBuildingRecovery {
    Residential275,
    Commercial315,
    Commercial39,
    // Pub. 946: ADS residential rental property placed after December 31, 2017.
    // https://www.irs.gov/publications/p946
    Ads30,
    Ads40,
}
impl RentalBuildingRecovery {
    pub(crate) fn half_months(self) -> i64 {
        match self {
            Self::Residential275 => 660,
            Self::Commercial315 => 756,
            Self::Commercial39 => 936,
            Self::Ads30 => 720,
            Self::Ads40 => 960,
        }
    }
}

// Raw calendar half-months through the end of `year`, without source admission.
pub(crate) fn rental_elapsed_half_months(placed: u32, year: u32) -> i64 {
    let first = 25 - 2 * i64::from(placed / 100 % 100);
    first + 24 * (i64::from(year) - i64::from(placed / 10000))
}

// Placement/year validity is established by the native source adapter.
pub(crate) fn rental_recovery_window(
    recovery: RentalBuildingRecovery,
    placed: u32,
    year: u32,
) -> u32 {
    let Some(years) = year.checked_sub(placed / 10000) else {
        return 0;
    };
    if recovery == RentalBuildingRecovery::Commercial39 {
        return if years <= 39 { 24 } else { 0 };
    }
    let first = 25 - 2 * (placed / 100 % 100);
    if years == 0 {
        first
    } else {
        (recovery.half_months() - i64::from(first) - 24 * i64::from(years - 1)).clamp(0, 24) as u32
    }
}

// Exact amount before a consumer's existing cents/dollars rounding boundary.
// `window` is the current-year mid-month disposition/use slice, in half-months.
fn macrs_building_amount(
    recovery: RentalBuildingRecovery,
    basis: i64,
    placed: u32,
    year: u32,
    window: u32,
) -> ratio::exact::Exact {
    let Some(years) = year.checked_sub(placed / 10000) else {
        return ratio::exact::whole(0);
    };
    let month = placed / 100 % 100;
    if recovery == RentalBuildingRecovery::Commercial39 {
        let full = match years {
            0..=38 => nonresidential_rate(month, years)
                .map(|rate| ratio::exact::whole(basis) * ratio::exact::ratio(rate, 100000).unwrap())
                .unwrap(),
            39 => ratio::exact::whole(nonresidential_closing(basis, month)),
            _ => ratio::exact::whole(0),
        };
        full * ratio::exact::ratio(i64::from(window), 24).unwrap()
    } else {
        ratio::exact::whole(basis)
            * ratio::exact::ratio(
                i64::from(rental_recovery_window(recovery, placed, year).min(window)),
                recovery.half_months(),
            )
            .unwrap()
    }
}

// Source-qualified amounts on the same full-use REG or AMT basis. None means
// no reduction, not unavailable source history. The native adapter validates
// complete Gregorian dates; this owner uses their original calendar clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DatedBasisReduction {
    pub event_date: u32,
    // Basis after the reduction and allowed/allowable depreciation through event.
    pub adjusted_basis_cents: i64,
    pub depreciation_through_event_cents: i64,
    // Actual allowed/allowable depreciation before the requested tax year.
    pub prior_depreciation_cents: i64,
}

pub(crate) fn rental_building_amount(
    recovery: RentalBuildingRecovery,
    basis: i64,
    placed: u32,
    year: u32,
    window: u32,
    reduction: Option<DatedBasisReduction>,
) -> Result<OfficeRecoveryDeduction, OfficeRecoveryIssue> {
    if !(0..=100_000_000_000).contains(&basis) {
        return Err(OfficeRecoveryIssue::InvalidBasis);
    }
    if let Some(reduction) = reduction {
        let placed_month = super::Month { year: placed / 10000, month: placed / 100 % 100 };
        let event_month = super::Month {
            year: reduction.event_date / 10000,
            month: reduction.event_date / 100 % 100,
        };
        if !placed_month.valid()
            || !event_month.valid()
            || !(1..=9999).contains(&year)
            || window > 24
            || reduction.event_date < placed
            || event_month.year > year
            || (event_month.year == year && 2 * event_month.month - 1 > window)
        {
            return Err(OfficeRecoveryIssue::InvalidDate);
        }
        // The existing first-year window is already admitted duration, not
        // an absolute January-to-disposition boundary. It cannot establish
        // whether a same-year casualty continues or disposes of the property.
        // This is the owner's input/support boundary, not a prohibition on
        // first-year casualty adjustments. Require an earlier-year clock;
        // consumers retain unsupported facts and report unavailability.
        if event_month.year <= placed_month.year {
            return Err(OfficeRecoveryIssue::OriginalMethodDateMismatch);
        }
        let through = reduction.depreciation_through_event_cents;
        let prior = reduction.prior_depreciation_cents;
        let adjusted = reduction.adjusted_basis_cents;
        if adjusted < 0
            || through < 0
            || prior < 0
            || through > basis
            || adjusted > basis - through
            || (year == event_month.year && prior > through)
            || (year > event_month.year && prior < through)
        {
            return Err(OfficeRecoveryIssue::InvalidBasis);
        }
        // Pub946, "Basis adjustment due to casualty loss": the adjustment
        // applies in the adjustment year using year-end adjusted basis BEFORE
        // that year's depreciation. Add back any through-event current-year
        // depreciation; do not split an annual deduction at the casualty date.
        // https://www.irs.gov/publications/p946
        // Pub547 "Adjustments to Basis" owns the supplied casualty reduction.
        // https://www.irs.gov/publications/p547
        let opening = adjusted + through - prior;
        if opening < 0 {
            return Err(OfficeRecoveryIssue::InvalidBasis);
        }
        let elapsed = rental_elapsed_half_months(placed, year - 1);
        let remaining = recovery.half_months() - elapsed;
        if remaining <= 0 {
            return if opening == 0 {
                Ok(OfficeRecoveryDeduction { exact_cents: ratio::exact::whole(0) })
            } else {
                Err(OfficeRecoveryIssue::InvalidBasis)
            };
        }
        // Without tables, straight-line uses the remaining original life at
        // the beginning of the year. The final year recovers its residual;
        // a shorter disposition window still receives only its actual slice.
        let slice = i64::from(window).min(remaining);
        // Pub946's SL rate is 100% when less than one year remains.
        // Holding through the final month instead deducts the full residual.
        let divisor = if remaining < 24 && slice < remaining { 24 } else { remaining };
        return Ok(OfficeRecoveryDeduction {
            exact_cents: ratio::exact::whole(opening)
                * ratio::exact::ratio(slice, divisor).unwrap(),
        });
    }
    let mut amount = macrs_building_amount(recovery, basis, placed, year, window);
    if recovery == RentalBuildingRecovery::Commercial315 && year > placed / 10000 {
        // The rental owner files each annual source amount in cents. Its
        // final year cannot deduct the few cents already admitted by those
        // earlier rounding boundaries. This is an unstored allowed/allowable
        // source fold, never an invented actual-history input.
        let admitted: i64 = (placed / 10000..year.min(placed / 10000 + 33))
            .map(|prior| {
                ratio::exact::rounded(&macrs_building_amount(recovery, basis, placed, prior, 24))
                    .unwrap()
            })
            .sum();
        amount = phase::cap(amount, [ratio::exact::whole(phase::floor(basis - admitted, 0))]);
    }
    Ok(OfficeRecoveryDeduction { exact_cents: amount })
}

// Pub 946 Table A-7a: 39-year nonresidential real property, mid-month
// convention, straight-line only. First-year percentage by month placed in
// service (scale matches `macrs_tables::RATES`: divisor 100,000); years 2-39
// are a flat 2.564%. Shared by `BusinessAsset` (Form 4562 line 19j) and
// `RentalBuilding` (Schedule E line 1b type 4, in extended_rules.rs).
const NONRESIDENTIAL_FIRST_YEAR: [i64; 12] =
    [2461, 2247, 2033, 1819, 1605, 1391, 1177, 963, 749, 535, 321, 107];
const NONRESIDENTIAL_ANNUAL_RATE: i64 = 2564;

// The percentage for the nth tax year after the year placed in service (0 =
// that year; `month` is the 1-12 service month). None once the 39-year
// recovery has closed (year 40); the caller then reconciles the exact
// remaining basis with `nonresidential_closing`.
pub(crate) fn nonresidential_rate(month: u32, years_since: u32) -> Option<i64> {
    match years_since {
        0 => Some(NONRESIDENTIAL_FIRST_YEAR[(month.clamp(1, 12) - 1) as usize]),
        1..=38 => Some(NONRESIDENTIAL_ANNUAL_RATE),
        _ => None,
    }
}

// The closing (40th) year amount: original basis less the 39 preceding
// years, so the published percentages' rounding still closes to zero.
pub(crate) fn nonresidential_closing(basis: i64, month: u32) -> i64 {
    let first = ratio::fraction(
        basis,
        NONRESIDENTIAL_FIRST_YEAR[(month.clamp(1, 12) - 1) as usize],
        100000,
    );
    let annual = ratio::fraction(basis, NONRESIDENTIAL_ANNUAL_RATE, 100000);
    ratio::positive(basis - first - annual * 38)
}

#[cfg(test)]
#[path = "historical_tests.rs"]
mod tests;
