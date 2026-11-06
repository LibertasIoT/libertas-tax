use super::*;
use crate::tests::support::assert::injured_income::estate_estimated_payment_sources::financial;
use crate::tests::support::builders::injured_income::estate_estimated_payment_sources::fixture;
use crate::tests::support::client::injured_income::estate_estimated_payment_sources::persist;
// R7: actual timely Form1041-T allocations belong to the beneficiary.
// Native line26 combines source Cents before its one whole-dollar boundary.

#[test]
fn unit13_estate_estimated_payments_preserve_native_money_and_public_save() {
    for beneficiary in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                for dated in [0, 49] {
                    let d = fixture(beneficiary, injured, reverse, dated);
                    financial(&d, dated != 0);
                    let mut plain = d.clone();
                    plain.injured_spouse = None;
                    plain.set_situations(SpouseSituations::INJURED_SPOUSE.0, false);
                    let ordinary = financial(&plain, dated != 0);
                    assert_eq!(estimate(&d).tax, ordinary.tax);
                    persist(d, dated != 0);
                }
            }
        }
    }
}

#[test]
fn unit13_estate_estimated_payments_keep_beneficiary_and_joint_category_rounding() {
    for beneficiary in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                for dated in [0, 49] {
                    let d = persist(fixture(beneficiary, injured, reverse, dated), dated != 0);
                    let r = financial(&d, dated != 0);
                    assert!(
                        !r.review_notes
                            .iter()
                            .any(|note| note.display.key == "WARN_8379_PAYMENT_OWNER_UNAVAILABLE")
                    );
                    let expected = if beneficiary == injured {
                        [1000, i64::from(dated != 0)]
                    } else {
                        [0, 1000 + i64::from(dated != 0)]
                    };
                    assert_eq!(column(&r, "20", "a"), expected.iter().sum::<i64>());
                    assert_eq!(column(&r, "20", "b"), expected[0]);
                    assert_eq!(column(&r, "20", "c"), expected[1]);
                }
            }
        }
    }
}
