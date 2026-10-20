use crate::tests::*;

// Form 5329 Parts III and IV (unit T2-42): the excess record for one owner.
pub(in crate::tests) fn ira_excess(
    traditional_value: i64,
    roth_value: i64,
    prior_traditional: i64,
    prior_roth: i64,
) -> IraExcess {
    IraExcess {
        traditional_value: Some(traditional_value * 100),
        roth_value: roth_value * 100,
        prior_traditional: prior_traditional * 100,
        prior_roth: prior_roth * 100,
        returned_prior_excess: 0,
    }
}
