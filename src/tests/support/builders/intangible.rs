use crate::*;
use alloc::vec::Vec;

pub(in crate::tests) fn ordinary_intangible(
    cost: i64,
    acquired: u32,
    business_started: u32,
    prior: i64,
) -> BusinessIntangible {
    BusinessIntangible {
        id: 0,
        label: "Synthetic purchased business goodwill".into(),
        cost,
        acquired,
        business_started,
        additions: Vec::new(),
        source: IntangibleSource {
            kind: IntangibleKind::Goodwill,
            obtained: IntangibleObtained::Purchased,
            context: IntangibleAcquisitionContext::EntireOperatingBusinessAssets,
            first_existed: Some(IntangibleOriginEvidence::CreatedOn { date: acquired }),
            consideration: IntangibleConsideration::FixedCash,
            lease_status: IntangibleLeaseStatus::NotLeased,
        },
        prior_claimed: IntangiblePriorClaimed {
            regular: SourceCents::Reported { amount: prior },
            amt: SourceCents::Reported { amount: prior },
        },
        other_events: Some(Vec::new()),
        use_end: IntangibleUseEnd::StillHeldAndUsed,
    }
}
