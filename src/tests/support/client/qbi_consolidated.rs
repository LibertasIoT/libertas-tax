use crate::tests::support::builders::native_income::document;
use crate::tests::support::client::income_pages::{open_topic, page_documents, save_topic};
use crate::tests::*;

type Exchange = fn(&mut Interview, P, &str, bool) -> Reply;

pub(in crate::tests) fn ptp_save_prior_carry(
    session: &mut Interview,
    case: &str,
    exchange: Exchange,
) {
    let request = save_topic(
        &session.draft,
        TaxSection::SelfEmployment,
        vec![document(
            0,
            IncomeDocument::QbiLossCarryforward(QbiLossCarryforward {
                business: 0,
                reit_ptp: 300_000,
                injured_owners: None,
            }),
        )],
    );
    accepted(&exchange(session, request, case, false));
}

pub(in crate::tests) fn ptp_reopen_and_cold_resave(
    session: &mut Interview,
    expected: &[IncomeDocument],
    result: &FederalResult,
    case: &str,
    exchange: Exchange,
) {
    let saved = session.draft.clone();
    let reopened = open_topic(session, TaxSection::PassThrough);
    let documents = page_documents(&reopened);
    assert_eq!(documents.len(), expected.len());
    for (index, (actual, expected)) in documents.iter().zip(expected).enumerate() {
        assert!(actual.id > 0);
        assert_eq!(&actual.document, expected);
        assert!(documents[..index].iter().all(|prior| prior.id != actual.id));
    }
    accepted(&exchange(session, reopened, &alloc::format!("{case}-resave"), false));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let reopened = open_topic(&mut cold, TaxSection::PassThrough);
    accepted(&exchange(&mut cold, reopened, &alloc::format!("{case}-cold-resave"), false));
    assert_eq!(cold.draft, saved);
    assert_eq!(&estimate(&cold.draft), result);
}

pub(in crate::tests) fn ptp_estimate_and_save(
    session: &mut Interview,
    request: P,
    case: &str,
    exchange: Exchange,
    estimate_request: fn(&P) -> P,
) -> Vec<u8> {
    let estimated =
        exchange(session, estimate_request(&request), &alloc::format!("{case}-estimate"), true);
    let Reply::Response(P::IncomeRunningEstimate { estimate, .. }) = estimated else {
        panic!("actual candidate running estimate")
    };
    accepted(&exchange(session, request, &alloc::format!("{case}-save"), false));
    estimate
}
