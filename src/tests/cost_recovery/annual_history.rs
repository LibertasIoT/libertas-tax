use crate::kernel::{
    carry::{Amount, Cents},
    cost_recovery::annual::{Schedule, calculate},
    source_fact::SourceFact,
};
type Money = Amount<Cents>;
use crate::tests::support::annual_recovery::{rounded_records, schedule};

#[test]
fn actual_annual_rounding_leaves_one_cent_for_post_recovery_then_closes() {
    let records = rounded_records();
    let result = calculate(SourceFact::Reported(schedule(&records)), |_| Money::new(100_001).ok());
    let SourceFact::Reported(result) = result else { panic!("complete actual schedule") };
    assert_eq!(result.opening_basis.get(), 1);
    assert_eq!(result.allowance.get(), 1);
    assert_eq!(result.closing_basis.get(), 0);
    assert_eq!(result.post_recovery_basis.unwrap().get(), 1);
}

#[test]
fn incomplete_duplicate_or_unreconciled_history_and_missing_caps_are_unavailable() {
    let mut records = rounded_records();
    let cap = |_| Money::new(100_001).ok();
    assert!(matches!(calculate(SourceFact::NotReported, cap), SourceFact::NotReported));
    assert!(matches!(calculate(SourceFact::Unavailable, cap), SourceFact::Unavailable));
    assert!(matches!(
        calculate(SourceFact::Reported(schedule(&records)), |_| None),
        SourceFact::Unavailable
    ));
    records[0].year = 2021;
    assert!(matches!(
        calculate(SourceFact::Reported(schedule(&records)), cap),
        SourceFact::Unavailable
    ));
    records[0].year = 2020;
    records[5].regular = Money::new(5_799).unwrap();
    assert!(matches!(
        calculate(SourceFact::Reported(schedule(&records)), cap),
        SourceFact::Unavailable
    ));
    records.pop();
    assert!(matches!(
        calculate(SourceFact::Reported(schedule(&records)), cap),
        SourceFact::Unavailable
    ));
}

#[test]
fn checked_cents_admit_maximum_basis_without_overflowing_annual_rounding() {
    let records = [];
    let original = schedule(&records);
    let maximum = Schedule {
        target_year: 2020,
        record_end_year: 2020,
        basis: Money::new(i64::MAX).unwrap(),
        bonus_percent: 100,
        first_year_deduction: Money::new(i64::MAX).unwrap(),
        original_bonus: Money::new(i64::MAX).unwrap(),
        prior_depreciation: Money::new(0).unwrap(),
        ..original
    };
    let SourceFact::Reported(result) =
        calculate(SourceFact::Reported(maximum), |_| Money::new(i64::MAX).ok())
    else {
        panic!("representable maximum basis")
    };
    assert_eq!(result.allowance.get(), i64::MAX);
    assert_eq!(result.closing_basis.get(), 0);
}
