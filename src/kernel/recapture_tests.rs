use super::*;
use SourceFact::{NotReported, Reported, Unavailable};

#[test]
fn section1245_preserves_press_gain_cap_and_signed1231_loss() {
    // Existing business.rs press: depreciation $6,160; gain $8,160 leaves $2,000.
    for (gain, ordinary, remaining) in
        [(8160, 6160, 2000), (1960, 1960, 0), (0, 0, 0), (-1400, 0, -1400)]
    {
        assert_eq!(
            calculate(Reported(RecaptureInput::LongSection1245 {
                recognized_gain_dollars: gain,
                depreciation_dollars: 6160,
            })),
            Reported(RecaptureAmounts::Disposition {
                section1245_dollars: ordinary,
                section1250_dollars: 0,
                ordinary_dollars: ordinary,
                remaining1231_dollars: remaining,
                unrecaptured1250_dollars: 0,
            })
        );
    }
}

#[test]
fn section1250_preserves_barn_example_and179_before_additional_depreciation() {
    // Existing business.rs 20-year barn: gain $75,385, depreciation $55,385,
    // additional $5,385; ordinary $5,385, §1231 $70,000, 25% class $50,000.
    for (gain, depreciation, bucket, additional, first, second, remaining, unrecaptured) in [
        (75385, 55385, 0, 5385, 0, 5385, 70000, 50000),
        (400, 900, 300, 200, 300, 100, 0, 0),
        (1200, 900, 300, 200, 300, 200, 700, 400),
        (10000, 5000, 0, 0, 0, 0, 10000, 5000),
    ] {
        assert_eq!(
            calculate(Reported(RecaptureInput::LongSection1250 {
                recognized_gain_dollars: gain,
                depreciation_dollars: depreciation,
                section179_bucket_dollars: bucket,
                additional_depreciation_dollars: additional,
            })),
            Reported(RecaptureAmounts::Disposition {
                section1245_dollars: first,
                section1250_dollars: second,
                ordinary_dollars: first + second,
                remaining1231_dollars: remaining,
                unrecaptured1250_dollars: unrecaptured,
            })
        );
    }
}

#[test]
fn section179_retains_source_cents_and_cannot_recapture_negative_excess() {
    // Existing F7: $14,000 expensed, $9,626.40 allowable, $4,373.60 recaptured.
    for (recomputed, excess) in [(962_640, 437_360), (1_400_000, 0), (1_500_000, 0)] {
        assert_eq!(
            calculate(Reported(RecaptureInput::ListedOr179Excess {
                allowed_cents: 1_400_000,
                recomputed_cents: recomputed,
            })),
            Reported(RecaptureAmounts::DeductionExcessCents(excess))
        );
    }
}

#[test]
fn credit_examples_keep_freed_credit_carry_reduction_and_basis_restoration_separate() {
    // Existing Form 4255 examples: year-three 60%; rehab restores 100%, energy 50%.
    for (used, carried, freed, reduction, tax, carry, basis) in [
        (16000, 0, 0, 100, 9600, 0, 960000),
        (16000, 0, 1000, 100, 9000, 0, 900000),
        (16000, 3000, 0, 50, 9600, 1800, 570000),
        (6000, 2000, 0, 50, 3600, 1200, 240000),
    ] {
        assert_eq!(
            calculate(Reported(RecaptureInput::InvestmentCredit {
                used_dollars: used,
                carried_dollars: carried,
                freed_dollars: freed,
                recapture_percent: 60,
                basis_reduction_percent: reduction,
            })),
            Reported(RecaptureAmounts::InvestmentCredit {
                tax_dollars: tax,
                carryover_reduction_dollars: carry,
                basis_increase_cents: basis,
            })
        );
    }
}

#[test]
fn credit_rounds_each_native_dollar_fraction_before_restoring_basis() {
    assert_eq!(
        calculate(Reported(RecaptureInput::InvestmentCredit {
            used_dollars: 1,
            carried_dollars: 1,
            freed_dollars: 0,
            recapture_percent: 50,
            basis_reduction_percent: 50,
        })),
        Reported(RecaptureAmounts::InvestmentCredit {
            tax_dollars: 1,
            carryover_reduction_dollars: 1,
            basis_increase_cents: 100,
        })
    );
}

#[test]
fn source_absence_unknown_zero_and_unrepresentable_calculation_remain_distinct() {
    assert_eq!(calculate(NotReported), NotReported);
    assert_eq!(calculate(Unavailable), Unavailable);
    assert_eq!(
        calculate(Reported(RecaptureInput::ListedOr179Excess {
            allowed_cents: 0,
            recomputed_cents: 0,
        })),
        Reported(RecaptureAmounts::DeductionExcessCents(0))
    );
    // The largest representable native-Dollar→Cents lift is accepted, with no
    // arbitrary amount cap; the next Dollar is explicitly unavailable.
    for (used, expected) in [
        (
            i64::MAX / 100,
            Reported(RecaptureAmounts::InvestmentCredit {
                tax_dollars: i64::MAX / 100,
                carryover_reduction_dollars: 0,
                basis_increase_cents: i64::MAX / 100 * 100,
            }),
        ),
        (i64::MAX / 100 + 1, Unavailable),
    ] {
        assert_eq!(
            calculate(Reported(RecaptureInput::InvestmentCredit {
                used_dollars: used,
                carried_dollars: 0,
                freed_dollars: 0,
                recapture_percent: 100,
                basis_reduction_percent: 100,
            })),
            expected
        );
    }
    assert_eq!(
        calculate(Reported(RecaptureInput::InvestmentCredit {
            used_dollars: i64::MAX,
            carried_dollars: i64::MAX,
            freed_dollars: 0,
            recapture_percent: 100,
            basis_reduction_percent: 50,
        })),
        Unavailable
    );
}

#[test]
fn native_zero_bucket_does_not_reclassify_negative_source_depreciation() {
    // Rental/partial source paths supply their actual known-zero §179 bucket,
    // even for a transient malformed negative depreciation before validation.
    let amounts = section1250(7, -3, 0, 0);
    assert_eq!(amounts.ordinary_dollars, 0);
    assert_eq!(amounts.remaining1231_dollars, 7);
    assert_eq!(amounts.unrecaptured1250_dollars, -3);
    // The rental path still evaluates the class at zero recognized gain.
    assert_eq!(section1250(0, i64::MIN, 0, 0).unrecaptured1250_dollars, i64::MIN);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "overflow")]
fn native_listed_excess_keeps_source_subtraction_failure() {
    let _ = deduction_excess(i64::MAX, -1);
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "overflow")]
fn native_credit_keeps_cent_lift_failure_instead_of_new_unavailability() {
    let _ = investment_credit(i64::MAX / 100 + 1, 0, 0, 100, 100);
}
