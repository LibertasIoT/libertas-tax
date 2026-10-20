use crate::tests::support::assert::injured_income::niit_supporting_sources::check;
use crate::tests::support::builders::injured_income::niit_supporting_sources::Supporting;

#[test]
fn unit13_niit_supporting_qualified_mitigation_is_excluded() {
    check(Supporting::Mitigation);
}
#[test]
fn unit13_niit_supporting_age17_able_rollover_is_not_income() {
    check(Supporting::AbleRollover);
}
#[test]
fn unit13_niit_supporting_personal_acquisition_loss_is_not_income() {
    check(Supporting::PersonalLoss);
}
#[test]
fn unit13_niit_supporting_investment_acquisition_reuses_final_capital_owner() {
    check(Supporting::InvestmentGain);
}
