use super::*;
use Owner::{Spouse, Taxpayer};

#[test]
fn reported_counterpart_retains_signed_physical_amounts_without_rounding() {
    for (owner, own, other, expected) in [
        (Taxpayer, -123, 456, [-123, 456]),
        (Spouse, -123, 456, [456, -123]),
        (Taxpayer, 0, 0, [0, 0]),
        (Spouse, i64::MAX, i64::MIN, [i64::MIN, i64::MAX]),
    ] {
        assert_eq!(
            counterpart_shares(owner, own, SourceFact::Reported(other)),
            SourceFact::Reported(expected)
        );
    }
}

#[test]
fn absent_or_unavailable_counterpart_never_becomes_a_reported_zero() {
    for owner in [Taxpayer, Spouse] {
        for own in [0, 98] {
            assert_eq!(
                counterpart_shares(owner, own, SourceFact::NotReported),
                SourceFact::NotReported
            );
            assert_eq!(
                counterpart_shares(owner, own, SourceFact::Unavailable),
                SourceFact::Unavailable
            );
        }
    }
}
