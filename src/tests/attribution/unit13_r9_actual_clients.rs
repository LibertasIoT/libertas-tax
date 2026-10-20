use super::*;

use crate::tests::support::builders::injured_credit::r9_actual_clients::fixture;

#[test]
fn r9_actual_fixture_preserves_independent_money_before_client() {
    for farm_owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                fixture(farm_owner, injured, reverse);
            }
        }
    }
}
