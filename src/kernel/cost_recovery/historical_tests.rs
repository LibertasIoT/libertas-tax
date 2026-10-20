use super::*;

#[test]
fn casualty_continues_original_mid_month_clock_from_actual_basis_history() {
    // Synthetic independent source record: July2015, $660,000 building;
    // $251,000 allowed/allowable before2026, $266,000 through August event,
    // $40,900 casualty basis reduction. Post-event basis=$353,100.
    // Pub946's casualty adjustment uses the whole adjustment year: add back
    // $15,000 through-event2026 depreciation, giving $368,100 / (409/24)
    // =$21,600. It does not restart27.5years or split at August's midpoint.
    // https://www.irs.gov/publications/p946 (casualty loss; straight line rate)
    let reduction = DatedBasisReduction {
        event_date: 20260810,
        adjusted_basis_cents: 35_310_000,
        depreciation_through_event_cents: 26_600_000,
        prior_depreciation_cents: 25_100_000,
    };
    let amount = |year, window, reduction| {
        rental_building_amount(
            RentalBuildingRecovery::Residential275,
            66_000_000,
            20150701,
            year,
            window,
            Some(reduction),
        )
    };
    assert_eq!(amount(2026, 24, reduction).unwrap().rounded_cents(), 2_160_000);
    // Next year retains385 halfmonths and the actual prior revised deduction.
    assert_eq!(
        amount(2027, 24, DatedBasisReduction { prior_depreciation_cents: 27_260_000, ..reduction })
            .unwrap()
            .rounded_cents(),
        2_160_000
    );
    // September disposition: 17/24 of the revised annual deduction.
    assert_eq!(amount(2026, 17, reduction).unwrap().rounded_cents(), 1_530_000);
    // Existing first-year windows cannot distinguish retained property from
    // a same-year disposition, which Pub946 excludes from depreciation.
    // An original earlier-year clock is required rather than guessing.
    assert_eq!(
        amount(
            2015,
            19,
            DatedBasisReduction {
                event_date: 20150810,
                adjusted_basis_cents: 59_100_000,
                depreciation_through_event_cents: 300_000,
                prior_depreciation_cents: 0,
            }
        ),
        Err(OfficeRecoveryIssue::OriginalMethodDateMismatch)
    );
    // January2043 is the original final month, with a $900 residual.
    let closing = DatedBasisReduction { prior_depreciation_cents: 61_820_000, ..reduction };
    assert_eq!(amount(2043, 1, closing).unwrap().rounded_cents(), 90_000);
    assert_eq!(amount(2044, 24, closing), Err(OfficeRecoveryIssue::InvalidBasis));
    assert_eq!(
        amount(2044, 24, DatedBasisReduction { prior_depreciation_cents: 61_910_000, ..reduction })
            .unwrap()
            .rounded_cents(),
        0
    );
    for invalid in [
        DatedBasisReduction { adjusted_basis_cents: -1, ..reduction },
        DatedBasisReduction { adjusted_basis_cents: 39_400_001, ..reduction },
        DatedBasisReduction { prior_depreciation_cents: 26_600_001, ..reduction },
        DatedBasisReduction { depreciation_through_event_cents: -1, ..reduction },
    ] {
        assert_eq!(amount(2026, 24, invalid), Err(OfficeRecoveryIssue::InvalidBasis));
    }
    assert_eq!(amount(2025, 24, reduction), Err(OfficeRecoveryIssue::InvalidDate));
    assert_eq!(amount(2027, 24, reduction), Err(OfficeRecoveryIssue::InvalidBasis));
}

#[test]
fn unchanged_none_preserves_historical_schedule_amounts() {
    // Existing published39-year percentages and exact other-clock fractions.
    // In particular39-year keeps its table rate, not the continuation SL rate.
    for (recovery, placed, year, expected) in [
        (RentalBuildingRecovery::Residential275, 20150701, 2026, 2_400_000),
        (RentalBuildingRecovery::Commercial315, 19920701, 2026, 0),
        (RentalBuildingRecovery::Commercial39, 20150701, 2026, 1_692_240),
        (RentalBuildingRecovery::Ads40, 20150701, 2026, 1_650_000),
    ] {
        assert_eq!(
            rental_building_amount(recovery, 66_000_000, placed, year, 24, None)
                .unwrap()
                .rounded_cents(),
            expected
        );
    }
}
