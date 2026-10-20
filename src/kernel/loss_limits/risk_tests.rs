use super::*;
use crate::kernel::carry::{Cents, Dollars};

fn amount<Unit>(value: i64) -> Amount<Unit> {
    Amount::new(value).unwrap()
}

fn entry(key: &'static str, basis: i64, prior: i64) -> RiskEntry<&'static str, Dollars> {
    RiskEntry { key, basis_allowed: amount(basis), prior_risk: amount(prior) }
}

fn settled<'a>(
    entries: &'a [RiskEntry<&'static str, Dollars>],
    bypass: &'a [(&'static str, Amount<Dollars>)],
    income: i64,
    capacity: i64,
) -> RiskSettlement<'a, &'static str, Dollars> {
    let SourceFact::Reported(value) = settle_at_risk(SourceFact::Reported(RiskInput {
        entries,
        bypass,
        income: amount(income),
        capacity,
    }))
    .unwrap() else {
        panic!("known risk source")
    };
    value
}

#[test]
fn ordered_classes_expenses_prior_risk_and_bypass_retain_native_footing() {
    let entries =
        [entry("ordinary", 6000, 1000), entry("expense", 1000, 500), entry("rental", 4000, 500)];
    let bypass = [("179", amount(700)), ("gift", amount(300))];
    let r = settled(&entries, &bypass, 1000, 2000);
    assert_eq!(
        r.classes
            .iter()
            .map(|v| (*v.key, v.opening.get(), v.allowed.get(), v.suspended.get()))
            .collect::<Vec<_>>(),
        [
            ("ordinary", 7000, 1615, 5385),
            ("expense", 1500, 346, 1154),
            ("rental", 4500, 1039, 3461)
        ]
    );
    assert_eq!((r.gross_allowed.get(), r.net_loss.get()), (3000, 2000));
    assert_eq!(
        r.bypass.iter().map(|(key, v)| (*key, v.get())).collect::<Vec<_>>(),
        [("179", 700), ("gift", 300)]
    );
    assert!(core::ptr::eq(r.bypass, &bypass));
    assert!(core::ptr::eq(r.classes[0].key, &entries[0].key));

    // Nondeductible occupies the second native slot, not a separate priority cap.
    let ties = [entry("ordinary", 1, 0), entry("expense", 1, 0), entry("capital", 1, 0)];
    let r = settled(&ties, &[], 0, 2);
    assert_eq!(r.classes.iter().map(|v| v.allowed.get()).collect::<Vec<_>>(), [1, 1, 0]);
}

#[test]
fn activity_income_offsets_before_net_loss_and_units_are_not_converted() {
    let entries = [entry("ordinary", 4600, 0)];
    let r = settled(&entries, &[], 3100, 600);
    assert_eq!(
        (r.classes[0].allowed.get(), r.classes[0].suspended.get(), r.net_loss.get()),
        (3700, 900, 600)
    );
    let account = r.close_account(600, SourceFact::NotReported).unwrap();
    assert_eq!(account.ending, 0);

    // The caller's genuine source unit stays Cents; this is not Dollars*100.
    let cents =
        [RiskEntry::<_, Cents> { key: 91, basis_allowed: amount(49), prior_risk: amount(24) }];
    let SourceFact::Reported(r) = settle_at_risk(SourceFact::Reported(RiskInput {
        entries: &cents,
        bypass: &[],
        income: amount(20),
        capacity: 30,
    }))
    .unwrap() else {
        panic!("known cents")
    };
    assert_eq!(
        (
            r.classes[0].opening.get(),
            r.classes[0].allowed.get(),
            r.classes[0].suspended.get(),
            r.net_loss.get()
        ),
        (73, 50, 23, 30)
    );
}

#[test]
fn signed_account_recapture_does_not_increase_current_allowance() {
    let entries = [entry("ordinary", 80, 0)];
    let r = settled(&entries, &[], 40, 0);
    assert_eq!(
        (r.gross_allowed.get(), r.net_loss.get(), r.classes[0].suspended.get()),
        (40, 0, 40)
    );
    let account = r
        .close_account(
            -100,
            SourceFact::Reported(RecaptureHistory {
                prior_deducted: amount(150),
                prior_recaptured: amount(20),
                threshold_1978: 0,
            }),
        )
        .unwrap();
    assert_eq!(account.ending, -100);
    let SourceFact::Reported(recapture) = account.recapture else { panic!("known history") };
    assert_eq!((recapture.income.get(), recapture.next_year_deduction.get()), (100, 100));
    assert_eq!(r.gross_allowed.get(), 40);
    for history in [SourceFact::NotReported, SourceFact::Unavailable] {
        let absent = matches!(history, SourceFact::NotReported);
        let account = r.close_account(-100, history).unwrap();
        assert_eq!(account.ending, -100);
        assert_eq!(matches!(account.recapture, SourceFact::NotReported), absent);
        assert_eq!(matches!(account.recapture, SourceFact::Unavailable), !absent);
    }
    let empty = settled(&[], &[], 0, 0);
    for (deducted, recaptured, expected) in [(150, 20, 50), (150, 130, 20)] {
        let account = empty
            .close_account(
                -150,
                SourceFact::Reported(RecaptureHistory {
                    prior_deducted: amount(deducted),
                    prior_recaptured: amount(recaptured),
                    threshold_1978: -100,
                }),
            )
            .unwrap();
        let SourceFact::Reported(recapture) = account.recapture else {
            panic!("known grandfathered history")
        };
        assert_eq!(recapture.income.get(), expected);
    }
}

#[test]
fn detailed_prior_net_loss_reduces_investment_without_a_passive_history_guess() {
    let SourceFact::Reported(opening) =
        detailed_opening(SourceFact::Reported(DetailedRisk::<Dollars> {
            capacity: amount(100),
            profit: amount(0),
            net_loss: amount(40),
        }))
        .unwrap()
    else {
        panic!("known detailed history")
    };
    assert_eq!(opening.get(), 60);
    for (capacity, profit, loss) in [(100, 1, 1), (100, 0, 101)] {
        assert!(matches!(
            detailed_opening(SourceFact::Reported(DetailedRisk::<Dollars> {
                capacity: amount(capacity),
                profit: amount(profit),
                net_loss: amount(loss),
            })),
            Err(LossLimitError::InvalidAmount)
        ));
    }
    assert!(matches!(
        detailed_opening(SourceFact::Reported(DetailedRisk::<Dollars> {
            capacity: amount(i64::MAX),
            profit: amount(1),
            net_loss: amount(0),
        })),
        Err(LossLimitError::Overflow)
    ));
    assert!(matches!(
        detailed_opening::<Dollars>(SourceFact::NotReported).unwrap(),
        SourceFact::NotReported
    ));
    assert!(matches!(
        detailed_opening::<Dollars>(SourceFact::Unavailable).unwrap(),
        SourceFact::Unavailable
    ));
}

#[test]
fn absent_unknown_empty_overflow_and_recapture_boundaries_remain_distinct() {
    assert!(matches!(
        settle_at_risk::<i64, Dollars>(SourceFact::NotReported).unwrap(),
        SourceFact::NotReported
    ));
    assert!(matches!(
        settle_at_risk::<i64, Dollars>(SourceFact::Unavailable).unwrap(),
        SourceFact::Unavailable
    ));
    let empty = settled(&[], &[], 0, 0);
    assert!(empty.classes.is_empty());
    assert_eq!((empty.gross_allowed.get(), empty.net_loss.get()), (0, 0));
    assert!(matches!(
        settle_at_risk(SourceFact::Reported(RiskInput::<i64, Dollars> {
            entries: &[],
            bypass: &[],
            income: amount(0),
            capacity: -1,
        })),
        Err(LossLimitError::AtRiskRecapture)
    ));
    let overflowing = [entry("ordinary", i64::MAX, 1)];
    assert!(matches!(
        settle_at_risk(SourceFact::Reported(RiskInput {
            entries: &overflowing,
            bypass: &[],
            income: amount(0),
            capacity: 0,
        })),
        Err(LossLimitError::Overflow)
    ));
    assert!(matches!(
        settle_at_risk(SourceFact::Reported(RiskInput::<i64, Dollars> {
            entries: &[],
            bypass: &[],
            income: amount(i64::MAX),
            capacity: 1,
        })),
        Err(LossLimitError::Overflow)
    ));
    assert!(matches!(
        empty.close_account(1, SourceFact::NotReported),
        Err(LossLimitError::Footing)
    ));
    assert!(matches!(
        empty.close_account(
            i64::MIN,
            SourceFact::Reported(RecaptureHistory {
                prior_deducted: amount(1),
                prior_recaptured: amount(0),
                threshold_1978: 0,
            })
        ),
        Err(LossLimitError::Overflow)
    ));
    assert!(matches!(
        empty.close_account(
            0,
            SourceFact::Reported(RecaptureHistory {
                prior_deducted: amount(1),
                prior_recaptured: amount(2),
                threshold_1978: 0,
            })
        ),
        Err(LossLimitError::InvalidAmount)
    ));
}

#[test]
fn signed_components_keep_disallowed_first_cents_and_positive_gains() {
    for (values, capacity, allowed, held) in [
        ([-1, -1, 0], 1, [0, -1, 0], [1, 0, 0]),
        (
            [-600_000, 200_000, -400_000],
            300_000,
            [-300_000, 200_000, -200_000],
            [300_000, 0, 200_000],
        ),
        ([-460_000, 310_000, 0], 60_000, [-370_000, 310_000, 0], [90_000, 0, 0]),
        ([70_000, -20_000, 0], -50_000, [70_000, -20_000, 0], [0, 0, 0]),
    ] {
        let rows = [(91, values[0]), (17, values[1]), (91, values[2])];
        let SourceFact::Reported(result) =
            settle_signed_components::<_, Cents>(SourceFact::Reported(&rows), capacity).unwrap()
        else {
            panic!("known signed components")
        };
        assert_eq!(result.net, values.into_iter().sum::<i64>());
        assert_eq!(result.allowed_net, allowed.into_iter().sum::<i64>());
        for (index, component) in result.components.iter().enumerate() {
            assert!(core::ptr::eq(component.key, &rows[index].0));
            assert_eq!(component.allowed_net(), allowed[index]);
            assert_eq!(component.held(), held[index]);
        }
    }
}

#[test]
fn signed_components_preserve_availability_empty_and_overflow() {
    assert!(matches!(
        settle_signed_components::<i64, Dollars>(SourceFact::NotReported, -1).unwrap(),
        SourceFact::NotReported
    ));
    assert!(matches!(
        settle_signed_components::<i64, Dollars>(SourceFact::Unavailable, 0).unwrap(),
        SourceFact::Unavailable
    ));
    let SourceFact::Reported(empty) =
        settle_signed_components::<i64, Dollars>(SourceFact::Reported(&[]), -1).unwrap()
    else {
        panic!("known empty source")
    };
    assert!(empty.components.is_empty());
    assert_eq!((empty.net, empty.allowed_net), (0, 0));
    for rows in [
        [(0, i64::MAX), (1, 1), (2, 0)],
        [(0, i64::MIN), (1, 0), (2, 0)],
        [(0, -i64::MAX), (1, -1), (2, 2)],
    ] {
        assert!(matches!(
            settle_signed_components::<_, Cents>(SourceFact::Reported(&rows), 0),
            Err(LossLimitError::Overflow)
        ));
    }
}

#[test]
fn economic_close_keeps_compare_investment_separate_from_selected_capacity() {
    let rows = [("operating", -200)];
    let SourceFact::Reported(result) =
        settle_signed_components::<_, Cents>(SourceFact::Reported(&rows), 100).unwrap()
    else {
        panic!("known Compare allowance")
    };
    assert_eq!((result.allowed_net, result.components[0].held()), (-100, 100));
    // Simplified capacity100 wins; actual detailed investment is negative50.
    // Its ending deficit150 is independent of the unchanged allowed loss100.
    for (deducted, recaptured, threshold, expected) in
        [(200, 20, 0, 150), (200, 130, 0, 70), (200, 20, -20, 130)]
    {
        let account = close_economic_account(
            -50,
            result.allowed_net,
            SourceFact::Reported(RecaptureHistory::<Cents> {
                prior_deducted: amount(deducted),
                prior_recaptured: amount(recaptured),
                threshold_1978: threshold,
            }),
        )
        .unwrap();
        assert_eq!(account.ending, -150);
        let SourceFact::Reported(recapture) = account.recapture else {
            panic!("known Compare history")
        };
        assert_eq!(
            (recapture.income.get(), recapture.next_year_deduction.get()),
            (expected, expected)
        );
    }
    let entries = [entry("operating", 200, 0)];
    let stronger = settled(&entries, &[], 0, 100);
    assert!(matches!(
        stronger.close_account(
            -50,
            SourceFact::Reported(RecaptureHistory {
                prior_deducted: amount(1),
                prior_recaptured: amount(2),
                threshold_1978: 1,
            })
        ),
        Err(LossLimitError::Footing)
    ));
}

#[test]
fn economic_close_preserves_history_availability_and_failure_priority() {
    for history in [SourceFact::NotReported, SourceFact::Unavailable] {
        let absent = matches!(history, SourceFact::NotReported);
        let account = close_economic_account::<Dollars>(-10, 20, history).unwrap();
        assert_eq!(account.ending, 10);
        assert_eq!(matches!(account.recapture, SourceFact::NotReported), absent);
        assert_eq!(matches!(account.recapture, SourceFact::Unavailable), !absent);
    }
    for (investment, net, deducted, recaptured, threshold, expected) in [
        (i64::MAX, 1, 1, 2, 1, LossLimitError::Overflow),
        (i64::MIN, -1, 1, 0, 0, LossLimitError::Overflow),
        (i64::MIN, 0, 1, 0, 0, LossLimitError::Overflow),
        (0, 0, 1, 2, 0, LossLimitError::InvalidAmount),
        (0, 0, 1, 0, 1, LossLimitError::InvalidAmount),
    ] {
        assert!(matches!(
            close_economic_account(investment, net, SourceFact::Reported(RecaptureHistory::<Dollars> {
                prior_deducted: amount(deducted), prior_recaptured: amount(recaptured), threshold_1978: threshold,
            })),
            Err(actual) if actual == expected
        ));
    }
}

#[test]
fn at_risk_irs_example_offsets_activity_income_before_limiting_net_loss() {
    // Form 6198 instructions: $4,600 Schedule C loss, $3,100 capital gain,
    // $600 additional amount at risk. Gross deduction allowed is $3,700.
    let entries = [entry("0", 4_600, 0)];
    let r = settled(&entries, &[], 3_100, 600);
    assert_eq!(r.classes.iter().map(|v| v.allowed.get()).collect::<Vec<_>>(), alloc::vec![3_700]);
    assert_eq!(r.classes.iter().map(|v| v.suspended.get()).collect::<Vec<_>>(), alloc::vec![900]);
    assert_eq!(r.net_loss.get(), 600);
    let entries = [entry("0", 4_600, 0)];
    let r = settled(&entries, &[], 3_100, 0);
    assert_eq!(r.classes.iter().map(|v| v.allowed.get()).collect::<Vec<_>>(), alloc::vec![3_100]);
    assert_eq!(r.net_loss.get(), 0);
}

#[test]
fn at_risk_preserves_loss_character_and_never_offsets_other_activity_income() {
    let entries = [entry("0", 6_000, 0), entry("1", 4_000, 0)];
    let r = settled(&entries, &[], 1_000, 2_000);
    assert_eq!(
        r.classes.iter().map(|v| v.allowed.get()).collect::<Vec<_>>(),
        alloc::vec![1_800, 1_200]
    );
    assert_eq!(
        r.classes.iter().map(|v| v.suspended.get()).collect::<Vec<_>>(),
        alloc::vec![4_200, 2_800]
    );
    assert_eq!(r.net_loss.get(), 2_000);
    let entries = [entry("0", 1_000, 0)];
    let r = settled(&entries, &[], 10_000, 0);
    assert_eq!(r.classes.iter().map(|v| v.allowed.get()).collect::<Vec<_>>(), alloc::vec![1_000]);
    assert_eq!(r.net_loss.get(), 0);
}

#[test]
fn at_risk_recapture_is_capped_and_becomes_next_year_deduction() {
    for (ending, deducted, recaptured, expected) in [
        (-9_000, 10_000, 3_000, 7_000),
        (-2_000, 10_000, 3_000, 2_000),
        (0, 10_000, 3_000, 0),
        (500, 10_000, 3_000, 0),
        (-1_000, 3_000, 3_000, 0),
    ] {
        let SourceFact::Reported(result) = recapture_from_ending(
            ending,
            SourceFact::Reported(RecaptureHistory::<Dollars> {
                prior_deducted: amount(deducted),
                prior_recaptured: amount(recaptured),
                threshold_1978: 0,
            }),
        )
        .unwrap() else {
            panic!("reported recapture history")
        };
        assert_eq!((result.income.get(), result.next_year_deduction.get()), (expected, expected));
    }
}

#[test]
fn at_risk_recapture_preserves_1978_negative_threshold() {
    for (ending, expected) in [(-49, 0), (-50, 0), (-51, 1), (-100, 50)] {
        let SourceFact::Reported(result) = recapture_from_ending(
            ending,
            SourceFact::Reported(RecaptureHistory::<Dollars> {
                prior_deducted: amount(1_000),
                prior_recaptured: amount(0),
                threshold_1978: -50,
            }),
        )
        .unwrap() else {
            panic!("reported recapture history")
        };
        assert_eq!(result.income.get(), expected);
    }
}
