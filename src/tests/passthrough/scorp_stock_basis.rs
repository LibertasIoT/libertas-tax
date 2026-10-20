use crate::k1_loss::{K1LossClass, LossAmounts};
use crate::kernel::carry::Amount;
use crate::kernel::ratio::dollars;
use crate::scorp_basis::*;
use crate::tests::{k1_amount, next_loss_limit};
use alloc::vec;

// Class amounts in catalog order; missing classes are zero.
fn amounts(values: &[i64]) -> LossAmounts {
    LossAmounts::from_slice(
        &values.iter().map(|value| k1_amount(*value)).collect::<alloc::vec::Vec<_>>(),
    )
}

fn basis(losses: &[i64]) -> StockBasis<'static> {
    StockBasis {
        opening: k1_amount(10_000),
        additions: k1_amount(0),
        income: k1_amount(0),
        distributions: k1_amount(0),
        nondeductible: k1_amount(0),
        prior_nondeductible: k1_amount(0),
        losses_first: false,
        // A test fixture: the leak keeps the borrowed class amounts alive.
        losses: alloc::boxed::Box::leak(alloc::boxed::Box::new(amounts(losses))),
        credit_reduction: k1_amount(0),
    }
}

#[test]
fn stock_losses_preserve_character_and_do_not_net_positive_income() {
    let mut b = basis(&[12_000, 8_000]);
    b.income = k1_amount(4_000);
    b.distributions = k1_amount(2_000);
    b.nondeductible = k1_amount(2_000);
    let r = b.calculate().unwrap();
    assert_eq!(r.stock_allowed, amounts(&[6_000, 4_000]));
    assert_eq!(r.losses_to_debt, amounts(&[6_000, 4_000]));
    assert_eq!((dollars(r.remaining_stock.get()), dollars(r.net_increase.get())), (0, 0));
}

#[test]
fn stock_election_changes_order_and_preserves_expenses_for_debt_stage() {
    let mut b = basis(&[9_000]);
    b.nondeductible = k1_amount(3_000);
    assert_eq!(b.calculate().unwrap().stock_allowed, amounts(&[7_000]));
    b.losses_first = true;
    b.prior_nondeductible = k1_amount(2_000);
    let r = b.calculate().unwrap();
    assert_eq!(r.stock_allowed, amounts(&[9_000]));
    assert_eq!(dollars(r.nondeductible_to_debt.get()), 4_000);
    assert_eq!(dollars(r.remaining_stock.get()), 0);
}

#[test]
fn stock_distributions_do_not_create_basis_or_use_debt() {
    let mut b = basis(&[3_000]);
    b.distributions = k1_amount(15_000);
    b.nondeductible = k1_amount(2_000);
    let r = b.calculate().unwrap();
    assert_eq!(dollars(r.distribution_gain.get()), 5_000);
    assert_eq!(r.losses_to_debt, amounts(&[3_000]));
    assert_eq!(dollars(r.nondeductible_to_debt.get()), 2_000);
    assert_eq!(r.stock_allowed, amounts(&[0]));
}

#[test]
fn stock_contributions_cannot_restore_reduced_debt_basis() {
    let mut b = basis(&[1_000]);
    b.additions = k1_amount(20_000);
    b.income = k1_amount(5_000);
    b.distributions = k1_amount(1_000);
    b.nondeductible = k1_amount(500);
    let r = b.calculate().unwrap();
    assert_eq!(dollars(r.remaining_stock.get()), 32_500);
    assert_eq!(dollars(r.net_increase.get()), 2_500);
    b.income = k1_amount(0);
    assert_eq!(dollars(b.calculate().unwrap().net_increase.get()), 0);
}

#[test]
fn stock_allocation_conserves_every_dollar_at_rounding_boundaries() {
    for available in 0..12 {
        let mut b = basis(&[1, 3, 7]);
        b.opening = k1_amount(available);
        let r = b.calculate().unwrap();
        assert_eq!(r.stock_allowed.total().get(), available * 100);
        for class in K1LossClass::ALL {
            let (total, used, rest) =
                (b.losses[class], r.stock_allowed[class], r.losses_to_debt[class]);
            assert_eq!(total.get(), used.get() + rest.get());
            assert!(used.get() >= 0 && rest.get() >= 0);
        }
    }
    let r = basis(&[]).calculate().unwrap();
    assert_eq!(dollars(r.remaining_stock.get()), 10_000);
    assert!(!r.stock_allowed.any());
}

#[test]
fn stock_rejects_impossible_balances_and_unelected_carry() {
    let mut b = basis(&[]);
    let empty = LossAmounts::default();
    b.losses = &empty;
    b.prior_nondeductible = k1_amount(1);
    assert!(b.calculate().is_none());
    b.prior_nondeductible = k1_amount(0);
    b.opening = Amount::new(i64::MAX).unwrap();
    b.additions = k1_amount(1);
    assert!(b.calculate().is_none());
}

fn loan(
    kind: LoanKind,
    principal: i64,
    basis: i64,
    advances: i64,
    repayments: i64,
) -> ShareholderLoan {
    ShareholderLoan {
        kind,
        principal: k1_amount(principal),
        basis: k1_amount(basis),
        advances: k1_amount(advances),
        repayments: k1_amount(repayments),
    }
}

#[test]
fn debt_regulation_examples_one_and_two_preserve_separate_notes() {
    let mut b = basis(&[10_000]);
    b.opening = k1_amount(0);
    b.income = k1_amount(6_000);
    let r = b
        .with_loans(&[
            loan(LoanKind::FormalNote, 1_000, 0, 0, 0),
            loan(LoanKind::FormalNote, 5_000, 1_000, 0, 0),
            loan(LoanKind::FormalNote, 0, 0, 4_000, 0),
        ])
        .unwrap();
    assert_eq!(r.allowed, amounts(&[10_000]));
    assert_eq!(
        r.loans.iter().map(|l| dollars(l.basis.get())).collect::<alloc::vec::Vec<_>>(),
        vec![0, 200, 800]
    );
    let empty = LossAmounts::default();
    b.losses = &empty;
    b.income = k1_amount(4_500);
    let r = b
        .with_loans(&[
            loan(LoanKind::FormalNote, 1_000, 0, 0, 0),
            loan(LoanKind::FormalNote, 5_000, 200, 0, 0),
            loan(LoanKind::FormalNote, 4_000, 800, 0, 4_000),
        ])
        .unwrap();
    assert_eq!(dollars(r.stock.get()), 0);
    assert_eq!(
        r.loans.iter().map(|l| dollars(l.basis.get())).collect::<alloc::vec::Vec<_>>(),
        vec![224, 1_276, 0]
    );
    assert_eq!(dollars(r.loans[2].restoration.get()), 3_200);
    assert_eq!(dollars(r.loans[2].capital_gain.get()), 0);
}

#[test]
fn debt_partial_repayment_needs_full_restoration_not_just_gain() {
    let mut b = basis(&[]);
    b.opening = k1_amount(0);
    b.income = k1_amount(300);
    let r = b.with_loans(&[loan(LoanKind::FormalNote, 1_000, 900, 0, 950)]).unwrap();
    assert_eq!(
        (
            dollars(r.stock.get()),
            dollars(r.loans[0].basis.get()),
            dollars(r.loans[0].restoration.get())
        ),
        (200, 50, 100)
    );
    assert_eq!(dollars(r.loans[0].capital_gain.get()), 0);
    b.income = k1_amount(50);
    let r = b.with_loans(&[loan(LoanKind::FormalNote, 1_000, 900, 0, 950)]).unwrap();
    // 2026-10-19 MoneyPrecision: the existing ratio keeps 4,750c,
    // then each reported $47.50 balance rounds to $48.
    assert_eq!(
        (r.stock.get(), r.loans[0].basis.get(), r.loans[0].capital_gain.get()),
        (0, 4_750, 4_750)
    );
}

#[test]
fn debt_open_account_nets_and_remembers_threshold_crossing() {
    let mut b = basis(&[]);
    b.opening = k1_amount(0);
    let r = b.with_loans(&[loan(LoanKind::OpenAccount, 16_000, 8_000, 1_000, 4_000)]).unwrap();
    assert_eq!(
        (
            dollars(r.loans[0].principal.get()),
            dollars(r.loans[0].basis.get()),
            dollars(r.loans[0].ordinary_gain.get())
        ),
        (13_000, 6_500, 1_500)
    );
    let r = b.with_loans(&[loan(LoanKind::OpenAccount, 13_000, 6_500, 20_000, 5_000)]).unwrap();
    assert_eq!(
        (dollars(r.loans[0].principal.get()), dollars(r.loans[0].basis.get())),
        (28_000, 21_500)
    );
    assert_eq!(r.loans[0].kind_next_year, LoanKind::FormerOpenAccount);
    let r = b.with_loans(&[loan(LoanKind::FormerOpenAccount, 28_000, 21_500, 0, 28_000)]).unwrap();
    assert_eq!(dollars(r.loans[0].ordinary_gain.get()), 6_500);
    assert_eq!(dollars(r.loans[0].capital_gain.get()), 0);
}

#[test]
fn open_account_threshold_uses_exact_source_cents() {
    let b = basis(&[]);
    let mut note = loan(LoanKind::OpenAccount, 25_000, 25_000, 0, 0);
    assert_eq!(
        b.with_loans(core::slice::from_ref(&note)).unwrap().loans[0].kind_next_year,
        LoanKind::OpenAccount
    );
    note.principal = Amount::new(2_500_001).unwrap();
    assert_eq!(b.with_loans(&[note]).unwrap_err(), BasisError::Invalid);
    let mut crossing = loan(LoanKind::OpenAccount, 25_000, 25_000, 0, 0);
    crossing.advances = Amount::new(1).unwrap();
    assert_eq!(
        b.with_loans(&[crossing]).unwrap().loans[0].kind_next_year,
        LoanKind::FormerOpenAccount
    );
}

#[test]
fn debt_loss_election_and_fully_repaid_notes() {
    let mut b = basis(&[8_000, 4_000]);
    b.opening = k1_amount(0);
    b.nondeductible = k1_amount(3_000);
    let loans = [loan(LoanKind::FormalNote, 10_000, 10_000, 0, 0)];
    let r = b.with_loans(&loans).unwrap();
    assert_eq!(dollars(r.allowed.total().get()), 7_000);
    assert_eq!(dollars(r.nondeductible_carry.get()), 0);
    b.losses_first = true;
    let r = b.with_loans(&loans).unwrap();
    assert_eq!(dollars(r.allowed.total().get()), 10_000);
    assert_eq!(dollars(r.suspended.total().get()), 2_000);
    assert_eq!(dollars(r.nondeductible_carry.get()), 3_000);
    let r = b.with_loans(&[loan(LoanKind::FormalNote, 10_000, 10_000, 0, 10_000)]).unwrap();
    assert_eq!(r.allowed, amounts(&[0, 0]));
    assert_eq!(dollars(r.loans[0].basis.get()), 0);
}

#[test]
fn debt_distributions_do_not_consume_loan_basis() {
    let mut b = basis(&[]);
    b.opening = k1_amount(0);
    b.income = k1_amount(10_000);
    b.distributions = k1_amount(11_000);
    let loans = [loan(LoanKind::FormalNote, 10_000, 8_000, 0, 0)];
    let r = b.with_loans(&loans).unwrap();
    assert_eq!(
        (
            dollars(r.stock.get()),
            dollars(r.distribution_gain.get()),
            dollars(r.loans[0].basis.get())
        ),
        (0, 1_000, 8_000)
    );
    b.distributions = k1_amount(8_000);
    let r = b.with_loans(&loans).unwrap();
    assert_eq!(
        (
            dollars(r.stock.get()),
            dollars(r.loans[0].basis.get()),
            dollars(r.loans[0].restoration.get())
        ),
        (0, 10_000, 2_000)
    );
}

#[test]
fn debt_invalid_and_underdetermined_sources_never_become_zero() {
    let mut b = basis(&[]);
    assert_eq!(
        b.with_loans(&[loan(LoanKind::FormalNote, 100, 101, 0, 0)]),
        Err(BasisError::Invalid)
    );
    assert_eq!(
        b.with_loans(&[loan(LoanKind::FormalNote, 100, 100, 0, 101)]),
        Err(BasisError::Invalid)
    );
    assert_eq!(
        b.with_loans(&[loan(LoanKind::OpenAccount, 25_001, 100, 0, 0)]),
        Err(BasisError::Invalid)
    );
    let no_restoration = b
        .with_loans(&[
            loan(LoanKind::FormalNote, 100, 0, 0, 50),
            loan(LoanKind::FormalNote, 100, 0, 0, 50),
        ])
        .unwrap();
    assert_eq!(
        no_restoration.loans.iter().map(|l| dollars(l.capital_gain.get())).sum::<i64>(),
        100
    );
    b.income = k1_amount(50);
    assert_eq!(
        b.with_loans(&[
            loan(LoanKind::FormalNote, 100, 0, 0, 50),
            loan(LoanKind::FormalNote, 100, 0, 0, 50),
        ]),
        Err(BasisError::RepaymentPriority)
    );
}

fn vintage(year: u32, original: i64, qbi: i64) -> LossVintage {
    LossVintage {
        year,
        original: k1_amount(original),
        original_qbi: k1_amount(qbi),
        remaining: k1_amount(original),
        remaining_qbi: k1_amount(qbi),
    }
}

#[test]
fn loss_history_fifo_uses_pre2018_then_fixed_vintage_percentages() {
    let history =
        [vintage(2025, 10_000, 10_000), vintage(2017, 2_000, 0), vintage(2019, 8_000, 2_000)];
    let r = release_losses(&history, k1_amount(6_000)).unwrap();
    assert_eq!(dollars(r.qbi.get()), 1_000);
    assert_eq!(dollars(r.remaining[0].remaining.get()), 0);
    assert_eq!(
        (dollars(r.remaining[1].remaining.get()), dollars(r.remaining[1].remaining_qbi.get())),
        (4_000, 1_000)
    );
    let next = release_losses(&r.remaining, k1_amount(6_000)).unwrap();
    assert_eq!(dollars(next.qbi.get()), 3_000);
    assert_eq!(dollars(history[1].remaining.get()), 2_000); // calculation never mutates accepted source history
}

#[test]
fn loss_history_uses_original_ratio_and_conserves_rounding_remainders() {
    let h = [LossVintage {
        year: 2020,
        original: k1_amount(3),
        original_qbi: k1_amount(1),
        remaining: k1_amount(2),
        remaining_qbi: k1_amount(1),
    }];
    let first = release_losses(&h, k1_amount(1)).unwrap();
    assert_eq!(dollars(first.qbi.get()), 0); // original 1/3, not remaining 1/2
    let last = release_losses(&first.remaining, k1_amount(1)).unwrap();
    assert_eq!(dollars(last.qbi.get()), 1);
    assert_eq!(dollars(last.remaining[0].remaining_qbi.get()), 0);
}

#[test]
fn loss_history_moves_still_suspended_qbi_to_next_limit() {
    let r =
        release_losses(&[vintage(2017, 2_000, 0), vintage(2019, 8_000, 2_000)], k1_amount(6_000))
            .unwrap();
    let (qbi, next) = next_loss_limit(&r, 3_000).unwrap();
    assert_eq!(qbi, 500);
    assert_eq!(next, Some(vintage(2026, 3_000, 500)));
    assert_eq!(next_loss_limit(&r, 6_000).unwrap(), (1_000, None));
    assert_eq!(next_loss_limit(&r, 0).unwrap(), (0, Some(vintage(2026, 6_000, 1_000))));
    assert_eq!(next_loss_limit(&release_losses(&[], k1_amount(0)).unwrap(), 0).unwrap(), (0, None));
}

#[test]
fn loss_history_basis_release_is_not_immediately_a_qbi_deduction() {
    let b = basis(&[20_000]);
    let result = b.with_loans(&[]).unwrap();
    let released = result
        .release_history_item_part(
            K1LossClass::Ordinary,
            &[vintage(2017, 4_000, 0), vintage(2026, 16_000, 8_000)],
            k1_amount(0),
        )
        .unwrap();
    assert_eq!((dollars(released.amount.get()), dollars(released.qbi.get())), (10_000, 3_000));
    let (qbi, next) = next_loss_limit(&released, 4_000).unwrap();
    assert_eq!(qbi, 1_200);
    assert_eq!(next, Some(vintage(2026, 6_000, 1_800)));
    assert!(
        result
            .release_history_item_part(
                K1LossClass::Ordinary,
                &[vintage(2026, 19_999, 0)],
                k1_amount(0)
            )
            .is_err()
    );
}

#[test]
fn loss_history_rejects_missing_inconsistent_or_duplicate_source_records() {
    assert!(release_losses(&[vintage(2017, 100, 1)], k1_amount(0)).is_err());
    assert!(release_losses(&[vintage(2027, 100, 0)], k1_amount(0)).is_err());
    assert!(release_losses(&[vintage(2020, 100, 0), vintage(2020, 100, 0)], k1_amount(0)).is_err());
    assert!(release_losses(&[vintage(2020, 100, 0)], k1_amount(101)).is_err());
    let mut bad = vintage(2020, 100, 90);
    bad.remaining = k1_amount(50);
    bad.remaining_qbi = k1_amount(0);
    assert!(release_losses(&[bad], k1_amount(0)).is_err());
}
