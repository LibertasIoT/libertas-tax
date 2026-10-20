use super::lifecycle::{TEST_USER, send};
use crate::interview::Interview;
use crate::tests::cross::interview::{accepted, save_topic};
use crate::tests::support::builders::{complete, wage_documents};
use crate::{FederalStatus, TaxSection};

// Ordinary Wages Save supplies durable identities for Care membership.
pub(in crate::tests) fn care_w2_sources(count: usize) -> Interview {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request =
        save_topic(&session.draft, TaxSection::Wages, wage_documents(count, 21_000, 0, 100));
    accepted(&send(&mut session, request));
    session
}
