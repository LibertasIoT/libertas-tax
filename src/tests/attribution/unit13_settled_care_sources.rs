use super::*;

#[test]
fn unit13_settled_care_w2_income_keeps_its_native_owner() {
    assert_settled_care_sources(SettledCareCase::W2);
}
#[test]
fn unit13_settled_care_grace_income_has_known_owner_without_inventing_w2_source() {
    assert_settled_care_sources(SettledCareCase::Grace);
}
#[test]
fn unit13_settled_care_mixed_income_keeps_native_line_rounding_and_unknown_source() {
    assert_settled_care_sources(SettledCareCase::Mixed);
}
#[test]
fn unit13_settled_care_fully_forfeited_is_known_zero() {
    assert_settled_care_sources(SettledCareCase::Forfeited);
}
