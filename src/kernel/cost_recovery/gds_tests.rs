use super::*;
use SourceFact::{NotReported, Reported, Unavailable};

const NEW_FIVE: Schedule = Schedule {
    placed: Month { year: 2026, month: 6 },
    tax_year: 2026,
    period: Period::Five,
    method: Method::Regular,
    convention: Convention::HalfYear,
    approach: Approach::Manual,
    history_basis_cents: 100_000,
    history: History::New { elected179_cents: 0 },
    raw_bonus_cents: 0,
    current_use_fraction: None,
    disposed: None,
    continuation: None,
};

#[test]
fn original_pub946_manual_switch_and_final_recovery_are_unchanged() {
    // Existing repayment.rs $1,000 five-year example, each year mapped to2026.
    for (year, prior, current) in [
        (2025, 20000, 32000),
        (2024, 52000, 19200),
        (2023, 71200, 11520),
        (2022, 82720, 11520),
        (2021, 94240, 5760),
    ] {
        let schedule = Schedule {
            placed: Month { year, month: 6 },
            history: History::Continuing {
                prior_depreciation_cents: prior,
                carryforward179_cents: 0,
                initial179_cents: 0,
                initial_bonus_cents: 0,
            },
            ..NEW_FIVE
        };
        assert_eq!(
            calculate(Reported(schedule)),
            Reported(Amounts {
                basis_after_raw_bonus_cents: 100000 - prior,
                full_year_cents: current,
                current_macrs_cents: current,
            })
        );
    }
}

#[test]
fn final_disposition_uses_original_convention_and_short_mq_year() {
    // Existing business.rs: 2021-Q4 $12,000 asset; $1,149.60 remaining.
    // The last year's 21 half-months give May $492.69, December all $1,149.60.
    for approach in [Approach::Manual, Approach::Tables] {
        for (month, expected) in [(5, 49269), (12, 114960)] {
            let result = calculate(Reported(Schedule {
                placed: Month { year: 2021, month: 11 },
                history_basis_cents: 1_200_000,
                history: History::Continuing {
                    prior_depreciation_cents: 1_085_040,
                    carryforward179_cents: 0,
                    initial179_cents: 0,
                    initial_bonus_cents: 0,
                },
                convention: Convention::MidQuarter,
                approach,
                disposed: Some(Month { year: 2026, month }),
                ..NEW_FIVE
            }));
            assert_eq!(
                result,
                Reported(Amounts {
                    basis_after_raw_bonus_cents: 114960,
                    full_year_cents: 114960,
                    current_macrs_cents: expected,
                })
            );
        }
    }
    let final_hy = Schedule {
        placed: Month { year: 2021, month: 6 },
        history: History::Continuing {
            prior_depreciation_cents: 94240,
            carryforward179_cents: 0,
            initial179_cents: 0,
            initial_bonus_cents: 0,
        },
        disposed: Some(Month { year: 2026, month: 5 }),
        ..NEW_FIVE
    };
    assert_eq!(
        calculate(Reported(final_hy)),
        Reported(Amounts {
            basis_after_raw_bonus_cents: 5760,
            full_year_cents: 5760,
            current_macrs_cents: 5760,
        })
    );
}

#[test]
fn existing_table_percentage_and_manual_first_year_stay_distinct() {
    // Existing business.rs seven-year $60,000 source: published14.29% vs manual.
    for (approach, amount) in [(Approach::Tables, 857400), (Approach::Manual, 857143)] {
        assert_eq!(
            calculate(Reported(Schedule {
                period: Period::Seven,
                history_basis_cents: 6_000_000,
                approach,
                ..NEW_FIVE
            })),
            Reported(Amounts {
                basis_after_raw_bonus_cents: 6_000_000,
                full_year_cents: amount,
                current_macrs_cents: amount,
            })
        );
    }
}

#[test]
fn actual_method_and_long_class_factor_select_existing_first_year_rates() {
    for approach in [Approach::Manual, Approach::Tables] {
        for (period, method, expected) in [
            (Period::Five, Method::Regular, 20000),
            (Period::Five, Method::Declining150, 15000),
            (Period::Five, Method::StraightLine, 10000),
            (Period::Fifteen, Method::Regular, 5000),
        ] {
            assert_eq!(
                calculate(Reported(Schedule { period, method, approach, ..NEW_FIVE })),
                Reported(Amounts {
                    basis_after_raw_bonus_cents: 100_000,
                    full_year_cents: expected,
                    current_macrs_cents: expected,
                })
            );
        }
    }
}

#[test]
fn elected179_raw_bonus_and_original_table_deductions_have_distinct_roles() {
    let new = Schedule {
        history_basis_cents: 1_000_000,
        history: History::New { elected179_cents: 100_000 },
        // Current source20% of the $9,000 opening balance.
        raw_bonus_cents: 180_000,
        approach: Approach::Tables,
        ..NEW_FIVE
    };
    assert_eq!(
        calculate(Reported(new)),
        Reported(Amounts {
            basis_after_raw_bonus_cents: 720_000,
            full_year_cents: 144_000,
            current_macrs_cents: 144_000,
        })
    );
    // Opening: original minus prior depreciation AND179carry. Table basis:
    // original minus initial179 AND original bonus, not the opening balance.
    for (approach, expected) in [(Approach::Tables, 172800), (Approach::Manual, 160000)] {
        let continuing = Schedule {
            placed: Month { year: 2025, month: 6 },
            history_basis_cents: 1_000_000,
            history: History::Continuing {
                prior_depreciation_cents: 500_000,
                carryforward179_cents: 100_000,
                initial179_cents: 100_000,
                // Actual original2025-before-Jan20 acquisition:40% of900000.
                initial_bonus_cents: 360_000,
            },
            approach,
            ..NEW_FIVE
        };
        assert_eq!(
            calculate(Reported(continuing)),
            Reported(Amounts {
                basis_after_raw_bonus_cents: 400_000,
                full_year_cents: expected,
                current_macrs_cents: expected,
            })
        );
    }
}

#[test]
fn use_change_rounds_after_full_year_and_before_disposition() {
    // A 2-cent historical basis has 1-cent full-year MACRS. Actual50/100 use
    // rounds that to1; the subsequent half-year disposition also rounds to1.
    // Combining the two fractions would incorrectly round to0.
    let schedule = Schedule {
        placed: Month { year: 2025, month: 6 },
        history_basis_cents: 2,
        history: History::Continuing {
            prior_depreciation_cents: 0,
            carryforward179_cents: 0,
            initial179_cents: 0,
            initial_bonus_cents: 0,
        },
        current_use_fraction: Some((50, 100)),
        disposed: Some(Month { year: 2026, month: 5 }),
        ..NEW_FIVE
    };
    assert_eq!(
        calculate(Reported(schedule)),
        Reported(Amounts {
            basis_after_raw_bonus_cents: 2,
            full_year_cents: 1,
            current_macrs_cents: 1,
        })
    );
}

#[test]
fn zero_same_year_disposal_missing_and_overflow_keep_their_native_boundaries() {
    assert_eq!(calculate(NotReported), NotReported);
    assert_eq!(calculate(Unavailable), Unavailable);
    assert_eq!(
        calculate(Reported(Schedule { history_basis_cents: 0, ..NEW_FIVE })),
        Reported(Amounts {
            basis_after_raw_bonus_cents: 0,
            full_year_cents: 0,
            current_macrs_cents: 0
        })
    );
    assert_eq!(
        calculate(Reported(Schedule {
            disposed: Some(Month { year: 2026, month: 8 }),
            ..NEW_FIVE
        })),
        Reported(Amounts {
            basis_after_raw_bonus_cents: 100_000,
            full_year_cents: 0,
            current_macrs_cents: 0,
        })
    );
    // No arbitrary amount cap: the native fraction can represent this source.
    let large = Schedule { history_basis_cents: i64::MAX, ..NEW_FIVE };
    assert_eq!(
        calculate(Reported(large)),
        Reported(Amounts {
            basis_after_raw_bonus_cents: i64::MAX,
            full_year_cents: i64::MAX / 5,
            current_macrs_cents: i64::MAX / 5,
        })
    );
    assert_eq!(
        calculate(Reported(Schedule { current_use_fraction: Some((u32::MAX, 1)), ..large })),
        Unavailable
    );
    assert_eq!(
        calculate(Reported(Schedule {
            placed: Month { year: 2025, month: 6 },
            history: History::Continuing {
                prior_depreciation_cents: i64::MAX,
                carryforward179_cents: 1,
                initial179_cents: 0,
                initial_bonus_cents: 0,
            },
            ..large
        })),
        Unavailable
    );
}

#[test]
fn pub225_brown_uses_printed_rate_and_effective_opening_basis() {
    // IRS Pub225 (2025), ch3, Brown: original2024/2025 dates and dollars.
    // https://www.irs.gov/publications/p225
    let continuation = AdjustedBasisContinuation {
        effective_year: 2025,
        adjusted_basis_cents: 1_121_800,
        prior_depreciation_cents: 138_200,
        declining_rate: Some((2307, 10000)),
    };
    for approach in [Approach::Manual, Approach::Tables] {
        let schedule = Schedule {
            placed: Month { year: 2024, month: 7 },
            tax_year: 2025,
            period: Period::Seven,
            method: Method::Declining150,
            approach,
            history_basis_cents: 1_290_000,
            history: History::Continuing {
                prior_depreciation_cents: 138_200,
                carryforward179_cents: 0,
                initial179_cents: 0,
                initial_bonus_cents: 0,
            },
            continuation: Some(continuation),
            ..NEW_FIVE
        };
        let Reported(amounts) = calculate(Reported(schedule)) else { panic!("known Brown facts") };
        assert_eq!(amounts.basis_after_raw_bonus_cents, 1_121_800);
        assert_eq!(amounts.full_year_cents, 258_799);
        assert_eq!(ratio::dollars(amounts.current_macrs_cents), 2588);
        // Derived later-year probe: only post-adjustment actual depreciation
        // ($2,588) leaves the opening balance. Do not subtract2024again.
        let later = Schedule {
            tax_year: 2026,
            history: History::Continuing {
                prior_depreciation_cents: 397_000,
                carryforward179_cents: 0,
                initial179_cents: 0,
                initial_bonus_cents: 0,
            },
            ..schedule
        };
        assert_eq!(
            calculate(Reported(later)),
            Reported(Amounts {
                basis_after_raw_bonus_cents: 863_000,
                full_year_cents: 199_094,
                current_macrs_cents: 199_094,
            })
        );
        let before =
            Schedule { tax_year: 2024, history: History::New { elected179_cents: 0 }, ..schedule };
        assert_eq!(
            calculate(Reported(before)),
            calculate(Reported(Schedule { continuation: None, ..before }))
        );
    }
}

#[test]
fn continuation_reuses_use_disposal_switch_and_original_recovery_end() {
    let continuation = AdjustedBasisContinuation {
        effective_year: 2026,
        adjusted_basis_cents: 2,
        prior_depreciation_cents: 0,
        declining_rate: Some((1, 2)),
    };
    let schedule = Schedule {
        placed: Month { year: 2025, month: 6 },
        history_basis_cents: 2,
        history: History::Continuing {
            prior_depreciation_cents: 0,
            carryforward179_cents: 0,
            initial179_cents: 0,
            initial_bonus_cents: 0,
        },
        current_use_fraction: Some((1, 2)),
        disposed: Some(Month { year: 2026, month: 5 }),
        continuation: Some(continuation),
        ..NEW_FIVE
    };
    assert_eq!(
        calculate(Reported(schedule)),
        Reported(Amounts {
            basis_after_raw_bonus_cents: 2,
            full_year_cents: 1,
            current_macrs_cents: 1,
        })
    );
    // Final Q4 short year: original recovery clock, not five new years.
    for (tax_year, month, full, current) in
        [(2030, 5, 2100, 900), (2030, 12, 2100, 2100), (2031, 5, 0, 0)]
    {
        assert_eq!(
            calculate(Reported(Schedule {
                tax_year,
                placed: Month { year: 2025, month: 11 },
                history_basis_cents: 10_000,
                convention: Convention::MidQuarter,
                current_use_fraction: None,
                disposed: Some(Month { year: tax_year, month }),
                continuation: Some(AdjustedBasisContinuation {
                    adjusted_basis_cents: 2100,
                    declining_rate: None, // Not needed at/after final recovery.
                    ..continuation
                }),
                ..schedule
            })),
            Reported(Amounts {
                basis_after_raw_bonus_cents: 2100,
                full_year_cents: full,
                current_macrs_cents: current,
            })
        );
    }
    assert_eq!(
        calculate(Reported(Schedule {
            tax_year: 2029,
            current_use_fraction: None,
            disposed: None,
            history_basis_cents: 10_000,
            continuation: Some(AdjustedBasisContinuation {
                adjusted_basis_cents: 2100,
                ..continuation
            }),
            ..schedule
        })),
        Reported(Amounts {
            basis_after_raw_bonus_cents: 2100,
            full_year_cents: 1400,
            current_macrs_cents: 1400,
        })
    );
    // SL uses its remaining clock without a declining-rate source.
    let straight = Schedule {
        method: Method::StraightLine,
        current_use_fraction: None,
        disposed: None,
        continuation: Some(AdjustedBasisContinuation { declining_rate: None, ..continuation }),
        ..schedule
    };
    assert_eq!(
        calculate(Reported(straight)),
        Reported(Amounts {
            basis_after_raw_bonus_cents: 2,
            full_year_cents: 0,
            current_macrs_cents: 0,
        })
    );
}

#[test]
fn continuation_validates_source_capacity_history_and_rate_without_guessing() {
    let continuation = AdjustedBasisContinuation {
        effective_year: 2026,
        adjusted_basis_cents: 500,
        prior_depreciation_cents: 100,
        declining_rate: Some((1, 2)),
    };
    let schedule = Schedule {
        placed: Month { year: 2025, month: 6 },
        history_basis_cents: 1000,
        history: History::Continuing {
            prior_depreciation_cents: 100,
            carryforward179_cents: 0,
            initial179_cents: 0,
            initial_bonus_cents: 0,
        },
        ..NEW_FIVE
    };
    for invalid in [
        AdjustedBasisContinuation { adjusted_basis_cents: -1, ..continuation },
        AdjustedBasisContinuation { adjusted_basis_cents: 901, ..continuation },
        AdjustedBasisContinuation { prior_depreciation_cents: -1, ..continuation },
        AdjustedBasisContinuation { prior_depreciation_cents: 99, ..continuation },
        AdjustedBasisContinuation { prior_depreciation_cents: 101, ..continuation },
        AdjustedBasisContinuation { effective_year: 2025, ..continuation },
        AdjustedBasisContinuation { declining_rate: None, ..continuation },
        AdjustedBasisContinuation { declining_rate: Some((-1, 2)), ..continuation },
        AdjustedBasisContinuation { declining_rate: Some((1, 0)), ..continuation },
    ] {
        assert_eq!(
            calculate(Reported(Schedule { continuation: Some(invalid), ..schedule })),
            Unavailable
        );
    }
    // A real exhausted basis needs no rate; an excess refund's taxable part
    // remains outside this kernel rather than making depreciation negative.
    assert_eq!(
        calculate(Reported(Schedule {
            continuation: Some(AdjustedBasisContinuation {
                adjusted_basis_cents: 0,
                declining_rate: None,
                ..continuation
            }),
            ..schedule
        })),
        Reported(Amounts {
            basis_after_raw_bonus_cents: 0,
            full_year_cents: 0,
            current_macrs_cents: 0,
        })
    );
    // A rate larger than remaining capacity is capped at that basis.
    assert_eq!(
        calculate(Reported(Schedule {
            continuation: Some(AdjustedBasisContinuation {
                declining_rate: Some((2, 1)),
                ..continuation
            }),
            ..schedule
        })),
        Reported(Amounts {
            basis_after_raw_bonus_cents: 500,
            full_year_cents: 500,
            current_macrs_cents: 500,
        })
    );
    // Later actual history cannot consume more than the adjusted opening pool.
    assert_eq!(
        calculate(Reported(Schedule {
            tax_year: 2027,
            history: History::Continuing {
                prior_depreciation_cents: 601,
                carryforward179_cents: 0,
                initial179_cents: 0,
                initial_bonus_cents: 0,
            },
            continuation: Some(continuation),
            ..schedule
        })),
        Unavailable
    );
}

#[test]
fn prepared_continuation_rate_uses_original_clock_method_and_effective_year() {
    for (period, method, effective_year, expected) in [
        (Period::Five, Method::Regular, 2026, (4444, 10000)),
        (Period::Five, Method::Declining150, 2026, (3333, 10000)),
        (Period::Five, Method::Regular, 2027, (5714, 10000)),
        (Period::Fifteen, Method::Regular, 2026, (1035, 10000)),
        // .0513 *1.5=.07695 rounds half-up at the second stage.
        (Period::Twenty, Method::Regular, 2026, (770, 10000)),
    ] {
        let schedule = Schedule {
            placed: Month { year: 2025, month: 6 },
            tax_year: 2029, // The later calculation must not move the rate anchor.
            period,
            method,
            ..NEW_FIVE
        };
        assert_eq!(schedule.continuation_declining_rate(effective_year), Some(expected));
    }
    for (month, rate) in [(1, 4848), (4, 4572), (7, 4324), (10, 4102)] {
        let schedule = Schedule {
            placed: Month { year: 2025, month },
            convention: Convention::MidQuarter,
            ..NEW_FIVE
        };
        assert_eq!(schedule.continuation_declining_rate(2026), Some((rate, 10000)));
    }
}

#[test]
fn prepared_continuation_rate_rejects_invalid_or_exhausted_clocks_and_straight_line() {
    let schedule = Schedule { placed: Month { year: 2025, month: 6 }, ..NEW_FIVE };
    for year in [0, 2024, 2025, 2031, 10000] {
        assert_eq!(schedule.continuation_declining_rate(year), None);
    }
    assert_eq!(schedule.continuation_declining_rate(2030), Some((40000, 10000)));
    for placed in [Month { year: 0, month: 6 }, Month { year: 2025, month: 13 }] {
        assert_eq!(Schedule { placed, ..schedule }.continuation_declining_rate(2026), None);
    }
    assert_eq!(
        Schedule { method: Method::StraightLine, ..schedule }.continuation_declining_rate(2026),
        None
    );
}

#[test]
fn prepared_continuation_rate_reproduces_browns_printed_rate_and_deduction() {
    // IRS Pub225 (2025), ch3, Brown prints .1538 *1.5=.2307.
    // https://www.irs.gov/publications/p225
    // Pub946, Figuring Depreciation Without Using the Tables, supplies the
    // convention's remaining-period clock and method factors.
    // https://www.irs.gov/publications/p946
    let schedule = Schedule {
        placed: Month { year: 2024, month: 7 },
        tax_year: 2025,
        period: Period::Seven,
        method: Method::Declining150,
        history_basis_cents: 1_290_000,
        history: History::Continuing {
            prior_depreciation_cents: 138_200,
            carryforward179_cents: 0,
            initial179_cents: 0,
            initial_bonus_cents: 0,
        },
        ..NEW_FIVE
    };
    let rate = schedule.continuation_declining_rate(2025);
    assert_eq!(rate, Some((2307, 10000))); // .1538 *1.5=.2307, not exact3/13.
    let Reported(amounts) = calculate(Reported(Schedule {
        continuation: Some(AdjustedBasisContinuation {
            effective_year: 2025,
            adjusted_basis_cents: 1_121_800,
            prior_depreciation_cents: 138_200,
            declining_rate: rate,
        }),
        ..schedule
    })) else {
        panic!("known prepared-rate schedule")
    };
    assert_eq!(amounts.full_year_cents, 258_799);
    assert_eq!(ratio::dollars(amounts.current_macrs_cents), 2588);
}
