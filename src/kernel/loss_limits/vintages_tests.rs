use super::*;
use crate::kernel::carry::Dollars;

fn amount(value: i64) -> Amount<Dollars> {
    Amount::new(value).unwrap()
}

fn vintage(
    key: &'static str,
    original: i64,
    qbi: i64,
    left: i64,
    left_qbi: i64,
) -> LossVintage<&'static str, Dollars> {
    LossVintage {
        key,
        original: amount(original),
        original_qualified: amount(qbi),
        remaining: amount(left),
        remaining_qualified: amount(left_qbi),
    }
}

fn release<'a>(
    history: &'a [LossVintage<&'static str, Dollars>],
    used: i64,
    order: &'a [usize],
) -> VintageRelease<'a, &'static str, Dollars> {
    let SourceFact::Reported(value) = release_vintages(SourceFact::Reported(VintageInput {
        vintages: history,
        amount: amount(used),
        year: 2026,
        order,
    }))
    .unwrap() else {
        panic!("known vintage source")
    };
    value
}

#[test]
fn original_percentage_survives_partial_release_and_qualified_exhaustion() {
    let history = [vintage("risk", 10, 7, 3, 1)];
    let r = release(&history, 1, &[0]);
    // Original7/10 rounds to1; remaining1/3 would incorrectly round to0.
    assert_eq!((r.amount.get(), r.qualified.get()), (1, 1));
    assert_eq!((r.remaining[0].remaining.get(), r.remaining[0].remaining_qualified.get()), (2, 0));
    assert_eq!((r.remaining[0].original.get(), r.remaining[0].original_qualified.get()), (10, 7));
    assert!(core::ptr::eq(r.remaining[0].key, &history[0].key));
    let r = release(&history, 3, &[0]);
    assert_eq!(
        (
            r.qualified.get(),
            r.remaining[0].remaining.get(),
            r.remaining[0].remaining_qualified.get()
        ),
        (1, 0, 0)
    );
}

#[test]
fn both_remaining_components_bound_rounding_at_final_exhaustion() {
    let history = [vintage("passive", 10, 4, 4, 3)];
    let r = release(&history, 3, &[0]);
    // Fixed4/10 gives1, but only1 nonqualified dollar remains, requiring2.
    assert_eq!(
        (
            r.qualified.get(),
            r.remaining[0].remaining.get(),
            r.remaining[0].remaining_qualified.get()
        ),
        (2, 1, 1)
    );
    let r = release(&history, 4, &[0]);
    assert_eq!(
        (
            r.qualified.get(),
            r.remaining[0].remaining.get(),
            r.remaining[0].remaining_qualified.get()
        ),
        (3, 0, 0)
    );
}

#[test]
fn source_supplied_order_is_debited_once_and_results_keep_source_indexing() {
    let history = [vintage("newer", 5, 4, 5, 4), vintage("older", 3, 1, 3, 1)];
    let r = release(&history, 5, &[1, 0]);
    assert_eq!((r.amount.get(), r.qualified.get()), (5, 3));
    assert_eq!(
        r.remaining
            .iter()
            .map(|v| (*v.key, v.remaining.get(), v.remaining_qualified.get()))
            .collect::<Vec<_>>(),
        [("newer", 3, 2), ("older", 0, 0)]
    );
}

#[test]
fn next_limit_keeps_its_actual_current_key_and_does_not_rewrite_prior_balances() {
    let history = [vintage("basis-origin", 5, 3, 5, 3)];
    let r = release(&history, 5, &[0]);
    let next = r.after_next_limit(amount(2), ("risk", 2026)).unwrap();
    assert_eq!(next.allowed_qualified.get(), 1);
    let held = next.held.unwrap();
    assert_eq!(held.key, ("risk", 2026));
    assert_eq!(
        (
            held.original.get(),
            held.original_qualified.get(),
            held.remaining.get(),
            held.remaining_qualified.get()
        ),
        (3, 2, 3, 2)
    );
    assert_eq!((r.remaining[0].remaining.get(), r.remaining[0].remaining_qualified.get()), (0, 0));
    let next = r.after_next_limit(amount(5), ("risk", 2026)).unwrap();
    assert_eq!(next.allowed_qualified.get(), 3);
    assert!(next.held.is_none());
    assert!(matches!(r.after_next_limit(amount(6), ("risk", 2026)), Err(LossLimitError::Footing)));
}

#[test]
fn availability_empty_history_invalid_order_and_overflow_do_not_invent_balances() {
    assert!(matches!(
        release_vintages::<i64, Dollars>(SourceFact::NotReported).unwrap(),
        SourceFact::NotReported
    ));
    assert!(matches!(
        release_vintages::<i64, Dollars>(SourceFact::Unavailable).unwrap(),
        SourceFact::Unavailable
    ));
    let r = release(&[], 0, &[]);
    assert!(r.remaining.is_empty());
    assert_eq!((r.amount.get(), r.qualified.get()), (0, 0));
    assert!(r.after_next_limit(amount(0), ("risk", 2026)).unwrap().held.is_none());
    let history = [vintage("a", 2, 1, 2, 1), vintage("b", 2, 1, 2, 1)];
    for order in [&[0, 0][..], &[0][..], &[0, 2][..]] {
        assert!(matches!(
            release_vintages(SourceFact::Reported(VintageInput {
                vintages: &history,
                amount: amount(1),
                year: 2026,
                order,
            })),
            Err(LossLimitError::Footing)
        ));
    }
    assert!(matches!(
        release_vintages(SourceFact::Reported(VintageInput {
            vintages: &history,
            amount: amount(5),
            year: 2026,
            order: &[0, 1],
        })),
        Err(LossLimitError::Footing)
    ));
    let invalid = [vintage("a", 5, 4, 3, 1)];
    assert!(matches!(
        release_vintages(SourceFact::Reported(VintageInput {
            vintages: &invalid,
            amount: amount(1),
            year: 2026,
            order: &[0],
        })),
        Err(LossLimitError::InvalidAmount)
    ));
    let overflow = [vintage("a", i64::MAX, 0, i64::MAX, 0), vintage("b", 1, 0, 1, 0)];
    assert!(matches!(
        release_vintages(SourceFact::Reported(VintageInput {
            vintages: &overflow,
            amount: amount(0),
            year: 2026,
            order: &[0, 1],
        })),
        Err(LossLimitError::Overflow)
    ));
}
