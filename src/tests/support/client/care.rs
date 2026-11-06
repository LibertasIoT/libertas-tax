use super::lifecycle::{TEST_USER, send};
use crate::FederalStatus;
use crate::interview::Interview;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::{complete, wage_documents};

// The document center supplies durable W-2 identities for Care membership.
pub(in crate::tests) fn care_w2_sources(count: usize) -> Interview {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request =
        super::income_pages::save_documents(&mut session, wage_documents(count, 21_000, 0, 100));
    accepted(&send(&mut session, request));
    session
}
