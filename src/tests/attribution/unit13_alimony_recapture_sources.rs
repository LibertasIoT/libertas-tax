use crate::tests::support::assert::injured_income::alimony_recapture_sources::check;
use crate::tests::support::builders::injured_income::alimony_recapture_sources::Case;

#[test]
fn unit13_alimony_recapture_stopped_payments_keep_actual_recipient() {
    check(Case::Stopped);
}
#[test]
fn unit13_alimony_recapture_and_other_spouse_paid_deduction_are_one_category() {
    check(Case::CurrentAndPaid);
}
#[test]
fn unit13_alimony_recapture_plus_paid49_cents_keeps_native_rounding() {
    check(Case::Paid49Cents);
}
#[test]
fn unit13_alimony_recapture_plus_paid50_cents_keeps_native_rounding() {
    check(Case::Paid50Cents);
}
