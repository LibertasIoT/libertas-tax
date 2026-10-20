use super::*;
use crate::tests::support::assert::injured_income::k1_activity_niit_sources::check;
use crate::tests::support::builders::injured_income::k1_activity_niit_sources::{Activity, Source};
// Independent activity-owner controls: known current ordinary/rental income
// and one continuing rental's actual prior PAL. There is no disposition,
// property gain, Section179, EBL, shared-source amount or inferred source split.
// These fixtures do not establish delivery of those separate source boundaries.

macro_rules! cases {
    ($(($name:ident, $source:ident, $activity:ident, $owner:ident)),+ $(,)?) => {$(
        #[test]
        fn $name() { check(Source::$source, Activity::$activity, Owner::$owner); }
    )+};
}

cases!(
    (unit13_k1_activity_niit_partner_active_taxpayer, Partner, Active, Taxpayer),
    (unit13_k1_activity_niit_partner_active_spouse, Partner, Active, Spouse),
    (unit13_k1_activity_niit_partner_passive_taxpayer, Partner, Passive, Taxpayer),
    (unit13_k1_activity_niit_partner_passive_spouse, Partner, Passive, Spouse),
    (unit13_k1_activity_niit_scorp_active_taxpayer, Scorp, Active, Taxpayer),
    (unit13_k1_activity_niit_scorp_active_spouse, Scorp, Active, Spouse),
    (unit13_k1_activity_niit_scorp_passive_taxpayer, Scorp, Passive, Taxpayer),
    (unit13_k1_activity_niit_scorp_passive_spouse, Scorp, Passive, Spouse),
    (unit13_k1_activity_niit_partner_rental_current_taxpayer, Partner, RentalCurrent, Taxpayer),
    (unit13_k1_activity_niit_partner_rental_current_spouse, Partner, RentalCurrent, Spouse),
    (unit13_k1_activity_niit_scorp_rental_current_taxpayer, Scorp, RentalCurrent, Taxpayer),
    (unit13_k1_activity_niit_scorp_rental_current_spouse, Scorp, RentalCurrent, Spouse),
    (unit13_k1_activity_niit_partner_rental_carry_taxpayer, Partner, RentalCarry, Taxpayer),
    (unit13_k1_activity_niit_partner_rental_carry_spouse, Partner, RentalCarry, Spouse),
    (unit13_k1_activity_niit_scorp_rental_carry_taxpayer, Scorp, RentalCarry, Taxpayer),
    (unit13_k1_activity_niit_scorp_rental_carry_spouse, Scorp, RentalCarry, Spouse),
    (unit13_k1_activity_niit_direct_rental_current_taxpayer, Rental, RentalCurrent, Taxpayer),
    (unit13_k1_activity_niit_direct_rental_current_spouse, Rental, RentalCurrent, Spouse),
    (unit13_k1_activity_niit_direct_rental_carry_taxpayer, Rental, RentalCarry, Taxpayer),
    (unit13_k1_activity_niit_direct_rental_carry_spouse, Rental, RentalCarry, Spouse),
);
