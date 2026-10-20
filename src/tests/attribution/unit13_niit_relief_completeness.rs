use super::*;
use crate::tests::support::assert::injured_income::niit_relief_completeness::{
    joint_money, owner_case,
};
use crate::tests::support::client::injured_income::niit_relief_completeness::accepted_case;

// These existing native categories change ordinary income/MAGI but never
// enter gross_nii (federal_rules.rs, Form8960 collector). No exclusion or
// taxpayer choice is inferred from the final joint NIIT amount.

#[test]
fn unit13_relief_joint_money_and_public_save_restart_remain_exact() {
    for case in 0..9 {
        let (draft, gross) = accepted_case(case, Owner::Spouse, Owner::Taxpayer);
        joint_money(&draft, gross);
    }
}

#[test]
fn unit13_health_loan_assistance_never_blocks_known_investment_owners() {
    owner_case(&[0, 1]);
}
#[test]
fn unit13_wildfire_relief_never_blocks_known_investment_owners() {
    owner_case(&[2, 3]);
}
#[test]
fn unit13_disaster_relief_never_blocks_known_investment_owners() {
    owner_case(&[4, 5]);
}
#[test]
fn unit13_living_insurance_never_blocks_known_investment_owners() {
    owner_case(&[6, 7]);
}
#[test]
fn unit13_trump_death_never_blocks_known_investment_owners() {
    owner_case(&[8]);
}
