use crate::kernel::ratio::exact;
use crate::tests::support::builders::{bankruptcy_debt, farm_debt, retained_loss_projection};
use crate::tests::*;

#[test]
fn retained_loss_carrier_rounds_wide_source_cents_once_per_cohort() {
    // Carrier boundary, not a native-valid giant-document witness. MAX+1
    // cents rounds to 92,233,720,368,547,758 dollars, whose cents still fit.
    let projection = retained_loss_projection(&[i64::MAX, 1]);
    assert_eq!(projection.vintages.len(), 1);
    assert_eq!(
        projection.vintages[0].balance,
        exact::whole(i64::MAX) + exact::whole(1)
    );
    assert_eq!(projection.single_estate, None);
    assert!(projection.has_estate);
    let allocation = NolHistory::allocate(0, &projection.vintages, |_| 0).unwrap();
    assert_eq!(allocation.deduction, 0);
    assert_eq!(
        allocation.carryforward,
        exact::whole(92_233_720_368_547_758)
    );
    let mut regular = FederalResult::empty(0);
    allocation.record(0, 0, &mut regular).unwrap();
    assert_eq!(line_amount(&regular, "nol_2018_opening"), 9_223_372_036_854_775_800);
    assert_eq!(line_amount(&regular, "regular_total_nol_carryforward"), 9_223_372_036_854_775_800);
    let mut alternative = FederalResult::empty(0);
    let allocation =
        AtnolHistory::calculate(0, 0, None, true, &projection.vintages, &mut alternative).unwrap();
    assert_eq!(allocation.deduction, 0);
    allocation.record(0, 0, &mut alternative).unwrap();
    assert_eq!(line_amount(&alternative, "atnol_2018_opening"), 9_223_372_036_854_775_800);
    assert_eq!(line_amount(&alternative, "amt_total_nol_carryforward"), 9_223_372_036_854_775_800);
}

#[test]
fn retained_loss_carrier_failure_names_the_actual_native_report() {
    // MAX+50 rounds upward; the actual required opening MoneyCents row
    // cannot represent it. Pooling and the Dollar owner still remain known.
    let projection = retained_loss_projection(&[i64::MAX, 50]);
    for alternative in [false, true] {
        let mut result = FederalResult::empty(0);
        let issue = if alternative {
            AtnolHistory::calculate(0, 0, None, true, &projection.vintages, &mut result)
                .unwrap()
                .record(0, 0, &mut result)
                .unwrap_err()
        } else {
            NolHistory::allocate(0, &projection.vintages, |_| 0)
                .unwrap()
                .record(0, 0, &mut result)
                .unwrap_err()
        };
        assert_eq!(issue.year, Some(2018));
        assert_eq!(issue.alternative, alternative);
        result.tax = Some(100);
        result.payments = Some(200);
        result.refund = Some(100);
        issue.report(&mut result);
        assert_eq!(result.state, ResultState::Unsupported);
        assert!(result.lines.is_empty() && result.return_lines.is_empty());
        assert!(result.tax.is_none() && result.payments.is_none() && result.refund.is_none());
        assert_eq!(
            result.issues.last().unwrap().display.key,
            if alternative {
                "ATNOL_AMOUNT_UNAVAILABLE"
            } else {
                "NOL_AMOUNT_UNAVAILABLE"
            }
        );
    }
}

#[test]
fn retained_loss_carrier_debt_uses_wide_known_balances_before_native_projection() {
    // The actual debt is an ordinary small, valid source. Only the supplied
    // retained-loss owner packet is seeded beyond source-document bounds.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, bankruptcy_debt(2, 10_000, 0));
    assert!(d.valid_stored());
    let opening = exact::whole(92_233_720_368_547_758);
    let mut result = FederalResult::empty(d.revision);
    let reduction = crate::debt::reduce_attributes(
        &d,
        &mut result,
        0,
        [crate::debt::LossCarryovers {
            current_nol: 1,
            excess_business_loss: 0,
        }; 2],
        [opening.clone(), opening.clone()],
    );
    assert_eq!(reduction.regular.current, 100);
    assert_eq!(reduction.regular.prior, 100);
    assert_eq!(reduction.alternative.current, 100);
    assert_eq!(reduction.alternative.prior, 100);
    let allocation =
        NolHistory::allocate(0, &retained_loss_projection(&[i64::MAX, 1]).vintages, |_| 0).unwrap();
    allocation
        .record(0, reduction.regular.prior, &mut result)
        .unwrap();
    assert_eq!(line_amount(&result, "regular_total_nol_carryforward"), 9_223_372_036_854_775_700);

    // A known adjusted-attribute limit wider than native Dollars still
    // establishes that a small qualified farm discharge is within the limit.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, farm_debt(2, None, 0));
    assert!(d.valid_stored());
    let mut result = FederalResult::empty(d.revision);
    let wide = exact::whole(i64::MAX) + exact::whole(1);
    let reduction = crate::debt::reduce_attributes(
        &d,
        &mut result,
        0,
        [crate::debt::LossCarryovers {
            current_nol: 0,
            excess_business_loss: 0,
        }; 2],
        [wide.clone(), wide],
    );
    assert_eq!(reduction.regular.prior, 200);
    assert_eq!(reduction.alternative.prior, 200);
    assert!(result.issues.is_empty());
}
