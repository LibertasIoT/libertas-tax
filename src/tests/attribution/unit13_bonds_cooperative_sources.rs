use crate::tests::support::assert::injured_income::bonds_cooperative_sources::{
    bonds, cooperative,
};
use crate::tests::support::builders::injured_income::bonds_cooperative_sources::Route;

// Execute both selected spouses, both statement orders and complete native
// Avro restart before comparing attribution. Thus a missing first owner row
// does not prevent the remaining independent lifecycle controls from running.

#[test]
fn unit13_bonds_cooperative_full_bond_exclusion_keeps_linked_recipient() {
    bonds(3000, 0, 12_000, 14_000, 3000);
}

#[test]
fn unit13_bonds_cooperative_partial_bond_exclusion_keeps_linked_recipient() {
    //6000 /10000 actual education/proceeds ×4000 interest =2400 excluded.
    bonds(4000, 0, 10_000, 6000, 2400);
}

#[test]
fn unit13_bonds_cooperative_prior_inclusion_is_not_excluded_twice() {
    bonds(3000, 1000, 12_000, 14_000, 2000);
}

#[test]
fn unit13_bonds_cooperative_schedule_c_distribution_is_attributed_once() {
    cooperative(Route::Business);
}
#[test]
fn unit13_bonds_cooperative_schedule_f_distribution_is_attributed_once() {
    cooperative(Route::Farm);
}
#[test]
fn unit13_bonds_cooperative_form4835_distribution_is_attributed_once() {
    cooperative(Route::FarmRental);
}
