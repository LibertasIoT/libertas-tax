use crate::kernel::carry::{Amount, Cents};
use crate::kernel::source_fact::SourceFact;
use alloc::{string::String, vec::Vec};
use core::cell::Cell;

#[test]
fn reported_zero_false_and_empty_are_values() {
    fn preserved<T: PartialEq + core::fmt::Debug>(value: T, expected: T) {
        let fact = SourceFact::Reported(value);
        assert_eq!(fact.require(|| "unused boundary"), Ok(Some(expected)));
    }
    preserved(0, 0);
    preserved(false, false);
    preserved(String::new(), String::new());
    preserved(Vec::<i32>::new(), Vec::new());
    assert_ne!(SourceFact::Reported(0), SourceFact::NotReported);
    assert_ne!(SourceFact::<i32>::NotReported, SourceFact::Unavailable);
}

#[test]
fn reported_bounds_are_inclusive_and_do_not_transform_values() {
    for value in [i64::MIN, -1, 0, 1, i64::MAX] {
        let fact = SourceFact::Reported(value).check_reported(|value| {
            if (-1..=1).contains(value) { Ok(()) } else { Err("source bound") }
        });
        if (-1..=1).contains(&value) {
            assert_eq!(fact, Ok(SourceFact::Reported(value)));
        } else {
            assert_eq!(fact, Err("source bound"));
        }
    }
}

#[test]
fn availability_skips_bounds_and_only_unknown_invokes_named_boundary() {
    let calls = Cell::new(0);
    for fact in [SourceFact::<i32>::NotReported, SourceFact::Unavailable] {
        assert_eq!(
            fact.clone().check_reported(|_| -> Result<(), &str> { panic!("no reported value") }),
            Ok(fact.clone())
        );
        let result = fact.require(|| {
            calls.set(calls.get() + 1);
            "existing source boundary"
        });
        match result {
            Ok(None) => assert_eq!(calls.get(), 0),
            Err("existing source boundary") => assert_eq!(calls.get(), 1),
            _ => panic!("availability changed"),
        }
    }
    assert_eq!(calls.get(), 1);
    assert_eq!(
        SourceFact::Reported(false).require(|| -> &str { panic!("known false") }),
        Ok(Some(false))
    );
}

#[test]
fn source_fact_retains_existing_unit_and_noncopy_value() {
    let fact = SourceFact::Reported(Amount::<Cents>::new(1).unwrap())
        .check_reported(|amount| if amount.get() <= 1 { Ok(()) } else { Err("source bound") })
        .unwrap();
    let amount: Amount<Cents> = fact.require(|| "existing source boundary").unwrap().unwrap();
    assert_eq!(amount.get(), 1);
}
