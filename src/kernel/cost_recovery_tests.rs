use super::*;

#[test]
fn original_regimes_preserve_initial_stage_and_monthly_clock() {
    // IRS 1.248-1 example: 54,500 costs, 500 initial, 54,000 amortizable;
    // July through December gives 1,800, not another initial deduction.
    // https://www.irs.gov/irb/2011-39_IRB
    for regime in [
        AmortizationRegime::Section195,
        AmortizationRegime::Section197,
        AmortizationRegime::Section248,
    ] {
        let initial = if regime == AmortizationRegime::Section197 { 0 } else { 50_000 };
        let schedule = MonthlySchedule {
            clock: MonthlyClock::Original {
                regime,
                started: Month { year: 2011, month: 7 },
                period_months: 180,
            },
            tax_year: 2011,
            basis_cents: 5_400_000,
            initial_cents: initial,
            allowed_before: None,
        };
        assert_eq!(
            schedule.allowable(),
            Ok(MonthlyAmounts {
                initial_cents: initial,
                current_allowable_cents: 180_000,
                prior_allowable_cents: 0,
                cumulative_allowable_cents: 180_000,
                statutory_remaining_cents: 5_220_000,
            })
        );
        let last = MonthlySchedule { tax_year: 2026, initial_cents: 0, ..schedule };
        assert_eq!(last.allowable().unwrap().prior_allowable_cents, 5_220_000);
        assert_eq!(last.allowable().unwrap().current_allowable_cents, 180_000);
        assert_eq!(last.allowable().unwrap().statutory_remaining_cents, 0);
    }
}

#[test]
fn current_allowable_is_difference_of_rounded_cumulative_amounts() {
    let schedule = MonthlySchedule {
        clock: MonthlyClock::Original {
            regime: AmortizationRegime::Section195,
            started: Month { year: 2025, month: 1 },
            period_months: 180,
        },
        tax_year: 2026,
        basis_cents: 100,
        initial_cents: 0,
        allowed_before: None,
    };
    assert_eq!(
        schedule.allowable(),
        Ok(MonthlyAmounts {
            initial_cents: 0,
            current_allowable_cents: 6,
            prior_allowable_cents: 7,
            cumulative_allowable_cents: 13,
            statutory_remaining_cents: 87,
        })
    );
}

#[test]
fn later_197_cost_uses_original_remaining_months() {
    let schedule = MonthlySchedule {
        clock: MonthlyClock::Section197Addition {
            original_started: Month { year: 2020, month: 1 },
            original_period_months: 180,
            included: Month { year: 2025, month: 7 },
        },
        tax_year: 2026,
        basis_cents: 114_000,
        initial_cents: 0,
        allowed_before: None,
    };
    // 66 months elapsed before inclusion, leaving 114; 6 prior and 12 current.
    assert_eq!(
        schedule.allowable(),
        Ok(MonthlyAmounts {
            initial_cents: 0,
            current_allowable_cents: 12_000,
            prior_allowable_cents: 6_000,
            cumulative_allowable_cents: 18_000,
            statutory_remaining_cents: 96_000,
        })
    );
}

#[test]
fn addition_before_start_or_after_original_end_preserves_native_boundaries() {
    for (included, tax_year, current, prior) in [
        (Month { year: 2019, month: 12 }, 2020, 12, 0),
        (Month { year: 2035, month: 2 }, 2034, 0, 0),
        (Month { year: 2035, month: 2 }, 2035, 180, 0),
        (Month { year: 2035, month: 2 }, 2036, 0, 180),
    ] {
        let amount = MonthlySchedule {
            clock: MonthlyClock::Section197Addition {
                original_started: Month { year: 2020, month: 1 },
                original_period_months: 180,
                included,
            },
            tax_year,
            basis_cents: 180,
            initial_cents: 0,
            allowed_before: None,
        }
        .allowable()
        .unwrap();
        assert_eq!(amount.current_allowable_cents, current);
        assert_eq!(amount.prior_allowable_cents, prior);
        assert_eq!(amount.statutory_remaining_cents, 180 - current - prior);
    }
}

#[test]
fn real_zero_and_representable_basis_are_distinct_from_invalid_parameters() {
    let schedule = MonthlySchedule {
        clock: MonthlyClock::Original {
            regime: AmortizationRegime::Section248,
            started: Month { year: 2026, month: 12 },
            period_months: 180,
        },
        tax_year: 2026,
        basis_cents: 0,
        initial_cents: 0,
        allowed_before: None,
    };
    assert_eq!(schedule.allowable().unwrap().cumulative_allowable_cents, 0);
    let maximum = MonthlySchedule { basis_cents: i64::MAX, ..schedule }.allowable().unwrap();
    assert_eq!(maximum.cumulative_allowable_cents + maximum.statutory_remaining_cents, i64::MAX);
    for invalid in [
        MonthlySchedule { basis_cents: -1, ..schedule },
        MonthlySchedule { initial_cents: -1, ..schedule },
        MonthlySchedule { tax_year: 0, ..schedule },
        MonthlySchedule {
            clock: MonthlyClock::Original {
                regime: AmortizationRegime::Section248,
                started: Month { year: 2026, month: 0 },
                period_months: 180,
            },
            ..schedule
        },
        MonthlySchedule {
            clock: MonthlyClock::Original {
                regime: AmortizationRegime::Section248,
                started: Month { year: 2026, month: 1 },
                period_months: 0,
            },
            ..schedule
        },
    ] {
        assert_eq!(invalid.allowable(), Err(InvalidMonthlySchedule));
    }
}

#[test]
fn published_macrs_schedules_preserve_complete_positive_percentages() {
    for rates in crate::kernel::cost_recovery::macrs_tables::RATES {
        assert_eq!(rates.iter().sum::<i64>(), 100000);
        assert!(rates.iter().all(|rate| *rate > 0));
    }
}

#[test]
fn exclusive_end_preserves_cumulative_rounding_and_original_addition_clock() {
    let original = MonthlyClock::Original {
        regime: AmortizationRegime::Section197,
        started: Month { year: 2025, month: 1 },
        period_months: 180,
    };
    let addition = MonthlyClock::Section197Addition {
        original_started: Month { year: 2020, month: 1 },
        original_period_months: 180,
        included: Month { year: 2025, month: 7 },
    };
    for (clock, basis, prior, current) in [(original, 100, 7, 2), (addition, 114_000, 6_000, 5_000)]
    {
        let schedule = MonthlySchedule {
            clock,
            tax_year: 2026,
            basis_cents: basis,
            initial_cents: 0,
            allowed_before: Some(Month { year: 2026, month: 6 }),
        };
        let amounts = schedule.allowable().unwrap();
        assert_eq!(amounts.prior_allowable_cents, prior);
        assert_eq!(amounts.current_allowable_cents, current);
        assert_eq!(amounts.cumulative_allowable_cents, prior + current);
        assert_eq!(amounts.statutory_remaining_cents, basis - prior - current);
        let later = MonthlySchedule { tax_year: 2027, ..schedule }.allowable().unwrap();
        assert_eq!(later.current_allowable_cents, 0);
        assert_eq!(later.prior_allowable_cents, prior + current);
        for month in [Month { year: 2024, month: 12 }, Month { year: 2025, month: 1 }] {
            let before =
                MonthlySchedule { allowed_before: Some(month), ..schedule }.allowable().unwrap();
            assert_eq!(before.cumulative_allowable_cents, 0);
            assert_eq!(before.statutory_remaining_cents, basis);
        }
        for month in [Month { year: 0, month: 1 }, Month { year: 2026, month: 13 }] {
            assert_eq!(
                MonthlySchedule { allowed_before: Some(month), ..schedule }.allowable(),
                Err(InvalidMonthlySchedule)
            );
        }
    }
}
