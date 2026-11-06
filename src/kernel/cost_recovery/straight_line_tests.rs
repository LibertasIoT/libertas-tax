use super::*;
use SourceFact::{NotReported, Reported, Unavailable};

#[test]
fn arbitrary_ads_life_and_conventions_use_original_service_month() {
    // 660-cent basis, 27.5-year (660 half-month) life: one cent per half-month.
    for (convention, expected) in
        [(Convention::HalfYear, 12), (Convention::MidQuarter, 9), (Convention::MidMonth, 11)]
    {
        let schedule = Schedule {
            placed: Month { year: 2026, month: 7 },
            period_half_months: exact::whole(660),
            convention,
            tax_year: 2026,
            basis: Money::new(660).unwrap(),
            prior_recovery: NotReported,
            disposed: None,
        };
        let Reported(result) = schedule.recover().unwrap() else { panic!() };
        assert_eq!(
            (result.current.get(), result.opening_basis.get(), result.closing_basis.get()),
            (expected, 660, 660 - expected)
        );
    }
}

#[test]
fn continuing_recovery_uses_actual_prior_and_rounds_current_cents_once() {
    // 30-year ADS, 11 half-months in first year, actual prior $100.01.
    // Current: 89999 * 24 / 709 = 3046.51 cents, rounds to3047.
    let schedule = Schedule {
        placed: Month { year: 2025, month: 7 },
        period_half_months: exact::whole(720),
        convention: Convention::MidMonth,
        tax_year: 2026,
        basis: Money::new(100_000).unwrap(),
        prior_recovery: Reported(Money::new(10_001).unwrap()),
        disposed: None,
    };
    let Reported(result) = schedule.recover().unwrap() else { panic!() };
    assert_eq!(
        (result.current.get(), result.opening_basis.get(), result.closing_basis.get()),
        (3047, 89999, 86952)
    );
}

#[test]
fn disposal_and_final_year_respect_each_conventions_remaining_window() {
    // Actual opening basis is 21 cents. Final windows are12,21,23 half-months.
    for (convention, placed_month, disposal_month, current) in [
        (Convention::HalfYear, 6, 1, 21),
        (Convention::MidQuarter, 11, 5, 8),
        (Convention::MidQuarter, 11, 12, 21),
        (Convention::MidMonth, 12, 6, 10),
        (Convention::MidMonth, 12, 12, 21),
    ] {
        let schedule = Schedule {
            placed: Month { year: 2025, month: placed_month },
            period_half_months: exact::whole(24),
            convention,
            tax_year: 2026,
            basis: Money::new(100).unwrap(),
            prior_recovery: Reported(Money::new(79).unwrap()),
            disposed: Some(Month { year: 2026, month: disposal_month }),
        };
        let Reported(result) = schedule.recover().unwrap() else { panic!() };
        assert_eq!((result.current.get(), result.closing_basis.get()), (current, 21 - current));
    }
}

#[test]
fn same_year_and_prior_disposal_never_create_a_deduction() {
    for convention in [Convention::HalfYear, Convention::MidQuarter, Convention::MidMonth] {
        for tax_year in [2026, 2027] {
            let schedule = Schedule {
                placed: Month { year: 2026, month: 1 },
                period_half_months: exact::whole(120),
                convention,
                tax_year,
                basis: Money::new(1000).unwrap(),
                prior_recovery: NotReported,
                disposed: Some(Month { year: 2026, month: 12 }),
            };
            let Reported(result) = schedule.recover().unwrap() else { panic!() };
            assert_eq!((result.current.get(), result.closing_basis.get()), (0, 1000));
        }
    }
}

#[test]
fn future_disposal_and_post_window_year_do_not_shift_recovery() {
    for (tax_year, expected) in [(2024, 0), (2025, 5), (2026, 9), (2027, 0)] {
        let schedule = Schedule {
            placed: Month { year: 2025, month: 6 },
            period_half_months: exact::whole(24),
            convention: Convention::HalfYear,
            tax_year,
            basis: Money::new(9).unwrap(),
            prior_recovery: NotReported,
            disposed: Some(Month { year: 2028, month: 1 }),
        };
        let Reported(result) = schedule.recover().unwrap() else { panic!() };
        assert_eq!(result.current.get(), expected);
    }
}

#[test]
fn unknown_stays_unknown_and_reported_history_is_checked() {
    for (prior_recovery, basis, tax_year, expected) in [
        (Unavailable, 10, 2026, Ok(None)),
        (Unavailable, 0, 2024, Ok(None)),
        (Reported(Money::new(0).unwrap()), 10, 2026, Ok(Some(2))),
        (Reported(Money::new(11).unwrap()), 10, 2026, Err(InvalidSchedule::PriorRecovery)),
    ] {
        let schedule = Schedule {
            placed: Month { year: 2025, month: 1 },
            period_half_months: exact::whole(120),
            convention: Convention::HalfYear,
            tax_year,
            basis: Money::new(basis).unwrap(),
            prior_recovery,
            disposed: None,
        };
        let actual = schedule.recover().map(|result| match result {
            Unavailable => None,
            Reported(amount) => Some(amount.current.get()),
            NotReported => panic!(),
        });
        assert_eq!(actual, expected);
    }
}

#[test]
fn invalid_dates_period_and_first_year_history_are_rejected() {
    for (placed, tax_year, period, disposed, prior, expected) in [
        (Month { year: 0, month: 1 }, 2026, 120, None, 0, InvalidSchedule::Date),
        (Month { year: 2026, month: 13 }, 2026, 120, None, 0, InvalidSchedule::Date),
        (Month { year: 2026, month: 1 }, 10000, 120, None, 0, InvalidSchedule::Date),
        (
            Month { year: 2026, month: 2 },
            2026,
            120,
            Some(Month { year: 2026, month: 1 }),
            0,
            InvalidSchedule::Date,
        ),
        (Month { year: 2026, month: 1 }, 2026, 0, None, 0, InvalidSchedule::Period),
        (Month { year: 2026, month: 1 }, 2026, 120, None, 1, InvalidSchedule::PriorRecovery),
    ] {
        let result = Schedule {
            placed,
            tax_year,
            period_half_months: exact::whole(period),
            convention: Convention::HalfYear,
            basis: Money::new(10).unwrap(),
            prior_recovery: Reported(Money::new(prior).unwrap()),
            disposed,
        }
        .recover();
        assert!(matches!(result, Err(e) if e == expected));
    }
}

#[test]
fn maximum_source_cents_and_long_period_never_overflow() {
    let schedule = Schedule {
        placed: Month { year: 9999, month: 12 },
        period_half_months: exact::whole(i64::from(u32::MAX)),
        convention: Convention::MidMonth,
        tax_year: 9999,
        basis: Money::new(i64::MAX).unwrap(),
        prior_recovery: NotReported,
        disposed: None,
    };
    let Reported(result) = schedule.recover().unwrap() else { panic!() };
    assert_eq!(result.current.get(), 2_147_483_648);
    assert_eq!(result.opening_basis.get(), i64::MAX);
    assert_eq!(result.closing_basis.get(), i64::MAX - 2_147_483_648);
}

#[test]
fn fractional_life_keeps_each_conventions_service_window() {
    // 49 months at 125% is 61.25 months, or 122.5 half-months.
    for (convention, current) in
        [(Convention::HalfYear, 24), (Convention::MidQuarter, 18), (Convention::MidMonth, 22)]
    {
        let schedule = Schedule {
            placed: Month { year: 2026, month: 7 },
            period_half_months: exact::whole(49 * 2) * exact::ratio(5, 4).unwrap(),
            convention,
            tax_year: 2026,
            basis: Money::new(245).unwrap(),
            prior_recovery: NotReported,
            disposed: None,
        };
        let Reported(result) = schedule.recover().unwrap() else { panic!() };
        assert_eq!((result.current.get(), result.closing_basis.get()), (current, 245 - current));
    }
}

#[test]
fn fractional_life_rounds_only_the_current_cent_projection() {
    let schedule = Schedule {
        placed: Month { year: 2026, month: 1 },
        period_half_months: exact::ratio(80, 3).unwrap(),
        convention: Convention::HalfYear,
        tax_year: 2026,
        basis: Money::new(10).unwrap(),
        prior_recovery: NotReported,
        disposed: None,
    };
    let Reported(result) = schedule.recover().unwrap() else { panic!() };
    // 10 * 12 / (80/3) = 4.5 cents, rounded once to 5.
    assert_eq!((result.current.get(), result.closing_basis.get()), (5, 5));
}

#[test]
fn fractional_endpoint_and_earlier_disposal_keep_the_annual_rate_cap() {
    // The endpoint falls half a half-month after December's midpoint.
    for (tax_year, disposal_month, current) in
        [(2026, None, 21), (2026, Some(12), 20), (2026, Some(6), 10), (2027, None, 0)]
    {
        let schedule = Schedule {
            placed: Month { year: 2025, month: 12 },
            period_half_months: exact::ratio(49, 2).unwrap(),
            convention: Convention::MidMonth,
            tax_year,
            basis: Money::new(100).unwrap(),
            prior_recovery: Reported(Money::new(79).unwrap()),
            disposed: disposal_month.map(|month| Month { year: 2026, month }),
        };
        let Reported(result) = schedule.recover().unwrap() else { panic!() };
        assert_eq!((result.current.get(), result.closing_basis.get()), (current, 21 - current));
    }
}

#[test]
fn positive_sub_half_month_life_recovers_basis_at_its_endpoint() {
    let schedule = Schedule {
        placed: Month { year: 2026, month: 12 },
        period_half_months: exact::ratio(1, i64::MAX).unwrap(),
        convention: Convention::MidMonth,
        tax_year: 2026,
        basis: Money::new(i64::MAX).unwrap(),
        prior_recovery: NotReported,
        disposed: None,
    };
    let Reported(result) = schedule.recover().unwrap() else { panic!() };
    assert_eq!((result.current.get(), result.closing_basis.get()), (i64::MAX, 0));
}

#[test]
fn negative_lives_are_rejected_even_with_unknown_history() {
    for period_half_months in [exact::whole(-1), exact::ratio(-1, 2).unwrap()] {
        let result = Schedule {
            placed: Month { year: 2026, month: 1 },
            period_half_months,
            convention: Convention::HalfYear,
            tax_year: 2026,
            basis: Money::new(0).unwrap(),
            prior_recovery: Unavailable,
            disposed: None,
        }
        .recover();
        assert!(matches!(result, Err(InvalidSchedule::Period)));
    }
}
