use super::{accrue, part_3};
use crate::kernel::ratio::exact::{ratio, whole};

#[test]
fn fractional_late_payment_leaves_exact_unpaid_balance() {
    // A quarter-cent paid one day after a one-cent installment leaves 3/4 cent.
    // A synthetic 100%-per-day rate makes that balance visible in cent output:
    // the paid portion rounds to 0; 76, 92, 92 and 105 days on 3/4 cent
    // round to 57 + 69 + 69 + 79 = 274 cents across the four rate periods.
    let (_, penalty) =
        part_3(&[(20260416, ratio(1, 4).unwrap())], [1, 0, 0, 0], [36_500; 4]).unwrap();
    assert_eq!(penalty, 274);
}

#[test]
fn fractional_installment_credit_preserves_the_unpaid_remainder() {
    // A 3/4-cent credit must leave 1/4 cent unpaid, rather than being rounded
    // up to pay the installment. At the same synthetic daily rate, its four
    // periods round to 19 + 23 + 23 + 26 = 91 cents.
    let (_, penalty) =
        part_3(&[(20260415, ratio(3, 4).unwrap())], [1, 0, 0, 0], [36_500; 4]).unwrap();
    assert_eq!(penalty, 91);
}

#[test]
fn accrual_rounds_the_interest_without_rounding_its_operand() {
    // 49.5 cents * 61 days * 6% / 365 is below half a cent;
    // rounding the operand to 50 cents would cross that boundary.
    assert_eq!(accrue(&ratio(99, 2).unwrap(), 20260415, 20260615, [6; 4]), Some(0));
    assert_eq!(accrue(&whole(50), 20260415, 20260615, [6; 4]), Some(1));
}
