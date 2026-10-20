use crate::tests::support::assert::injured_income::medicare_withholding_sources::check;
use crate::tests::support::builders::medicare_withholding::Source;
// R7: keep native ordinary and RRTA withholding categories separate. Ordinary
// money and the public Wages/TaxAppData lifecycle precede source assertions.

#[test]
fn unit13_medicare_withholding_rrta_and_ordinary_opposite_owners() {
    check(Source::Railroad);
}
#[test]
fn unit13_medicare_withholding_two50c_categories_round_independently() {
    check(Source::HalfCategories);
}
#[test]
fn unit13_medicare_withholding_actual_ordinary_uncollected_credit() {
    check(Source::Uncollected);
}
#[test]
fn unit13_medicare_withholding_signed_opposite_one_cent_underage() {
    check(Source::SignedOrdinary);
}

// Actual W2 admission permits this one-cent ordinary-Medicare underage.
// Native Form8959 caps its negative difference at zero; no source-owner
// credit is manufactured and no required allocation question is needed.
#[test]
fn unit13_medicare_withholding_native_zero_underage_keeps_known_payments() {
    check(Source::ZeroUnderage);
}
