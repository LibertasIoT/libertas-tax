use crate::tests::support::assert::injured_income::home_employee_niit_sources::check;
use crate::tests::support::builders::home_employee::Source;

#[test]
fn unit13_home_employee_niit_fully_excluded_home_is_known_zero_capital() {
    check(Source::ExcludedHome);
}
#[test]
fn unit13_home_employee_niit_taxable_home_uses_final_capital_owner() {
    check(Source::TaxableHome);
}
#[test]
fn unit13_home_employee_niit_covered_pay_keeps_actual_8919_and_8959_sources() {
    check(Source::CoveredEmployee);
}
#[test]
fn unit13_home_employee_niit_noncovered_pay_preserves_native_payroll_omission() {
    check(Source::UncoveredEmployee);
}

#[test]
fn unit13_home_employee_niit_opposite_w2_and_8919_keep_actual_joint_8959_choice() {
    check(Source::OppositeEmployee);
}
