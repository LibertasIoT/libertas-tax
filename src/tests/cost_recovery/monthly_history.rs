use crate::kernel::cost_recovery::{
    AmortizationRegime, InvalidMonthlySchedule, Month, MonthlyClock, MonthlySchedule,
    monthly_history::ActualMonthlySchedule,
};
use crate::tests::support::builders::timber::monthly_history_schedule;

#[test]
fn actual_window_rounds_once_and_preserves_distinct_cumulative_schedule() {
    let schedule = monthly_history_schedule();
    let recovered = schedule.recover().unwrap();
    assert_eq!((recovered.current_cents, recovered.remaining_cents), (7, 86));
    let cumulative = MonthlySchedule {
        clock: MonthlyClock::Original {
            regime: AmortizationRegime::Section197,
            started: schedule.started,
            period_months: schedule.period_months,
        },
        tax_year: schedule.tax_year,
        basis_cents: schedule.basis_cents,
        initial_cents: 0,
        allowed_before: None,
    }
    .allowable()
    .unwrap();
    assert_eq!(cumulative.current_allowable_cents, 6);
    let capped =
        ActualMonthlySchedule { prior_recovery_cents: Some(99), ..schedule }.recover().unwrap();
    assert_eq!((capped.current_cents, capped.remaining_cents), (1, 0));
}

#[test]
fn disposal_includes_its_month_termination_excludes_it_without_restarting() {
    let schedule = ActualMonthlySchedule {
        started: Month { year: 2024, month: 7 },
        period_months: 84,
        tax_year: 2026,
        basis_cents: 1_400_000,
        prior_recovery_cents: Some(300_000),
        allowed_through: Some(Month { year: 2026, month: 9 }),
        ..monthly_history_schedule()
    };
    let disposal = schedule.recover().unwrap();
    assert_eq!((disposal.current_cents, disposal.remaining_cents), (150_000, 950_000));
    let termination =
        ActualMonthlySchedule { allowed_before: schedule.allowed_through, ..schedule }
            .recover()
            .unwrap();
    assert_eq!((termination.current_cents, termination.remaining_cents), (133_333, 966_667));
    let historical =
        ActualMonthlySchedule { allowed_through: Some(Month { year: 2025, month: 9 }), ..schedule }
            .recover()
            .unwrap();
    assert_eq!((historical.current_cents, historical.remaining_cents), (0, 1_100_000));
    let last = ActualMonthlySchedule {
        tax_year: 2031,
        prior_recovery_cents: Some(1_300_000),
        allowed_through: None,
        ..schedule
    }
    .recover()
    .unwrap();
    assert_eq!((last.current_cents, last.remaining_cents), (100_000, 0));
    let expired = ActualMonthlySchedule { tax_year: 2032, allowed_through: None, ..schedule }
        .recover()
        .unwrap();
    assert_eq!((expired.current_cents, expired.remaining_cents), (0, 1_100_000));
}

#[test]
fn missing_and_invalid_histories_do_not_become_zero() {
    let schedule = ActualMonthlySchedule {
        started: Month { year: 2026, month: 7 },
        period_months: 84,
        tax_year: 2026,
        basis_cents: 0,
        prior_recovery_cents: Some(0),
        allowed_through: None,
        ..monthly_history_schedule()
    };
    assert_eq!(schedule.recover().unwrap().current_cents, 0);
    for invalid in [
        ActualMonthlySchedule { prior_recovery_cents: None, ..schedule },
        ActualMonthlySchedule { prior_recovery_cents: Some(1), ..schedule },
        ActualMonthlySchedule { basis_cents: -1, ..schedule },
        ActualMonthlySchedule { period_months: 0, ..schedule },
        ActualMonthlySchedule { tax_year: 0, ..schedule },
        ActualMonthlySchedule { started: Month { year: 2026, month: 13 }, ..schedule },
        ActualMonthlySchedule { allowed_through: Some(Month { year: 0, month: 1 }), ..schedule },
        ActualMonthlySchedule { allowed_before: Some(Month { year: 2026, month: 0 }), ..schedule },
    ] {
        assert_eq!(invalid.recover(), Err(InvalidMonthlySchedule));
    }
    let large = ActualMonthlySchedule { basis_cents: i64::MAX, ..schedule }.recover().unwrap();
    assert_eq!(large.current_cents + large.remaining_cents, i64::MAX);
}

#[test]
fn shared_money_sources_retain_availability_and_check_business_threshold() {
    use crate::kernel::source_fact::{SourceCents, SourceFact};
    assert_eq!(SourceCents::Reported { amount: 0 }.source_fact(), Ok(SourceFact::Reported(0)));
    assert_eq!(SourceCents::NotReported.source_fact(), Ok(SourceFact::NotReported));
    assert_eq!(SourceCents::Unavailable.source_fact(), Ok(SourceFact::Unavailable));
    let source = SourceCents::Reported { amount: 300_001 };
    assert!(
        source
            .source_fact()
            .unwrap()
            .check_reported(|amount| { if *amount <= 300_000 { Ok(()) } else { Err(()) } })
            .is_err()
    );
}
