use crate::{Owner, kernel::source_fact::SourceFact};

// A complete physical pair requires the actual counterpart's reported fact.
// Preserve known absence and unknown separately; neither implies a zero share.
// Signed amounts retain the caller's units without subtraction or rounding.
pub(crate) fn counterpart_shares(
    known_owner: Owner,
    own: i64,
    counterpart: SourceFact<i64>,
) -> SourceFact<[i64; 2]> {
    match counterpart {
        SourceFact::Reported(other) => {
            let mut shares = [own, other];
            if super::owner_index(known_owner) == 1 {
                shares.swap(0, 1);
            }
            SourceFact::Reported(shares)
        }
        SourceFact::NotReported => SourceFact::NotReported,
        SourceFact::Unavailable => SourceFact::Unavailable,
    }
}

#[cfg(test)]
#[path = "../attribution_counterpart_tests.rs"]
mod tests;
