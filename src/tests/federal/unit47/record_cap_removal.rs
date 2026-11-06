use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::income_record_boundary_documents;
use crate::tests::*;

#[test]
fn large_income_record_count_saves_reopens_and_estimates() {
    // 101 income sources must save and reopen with the same $10,010,000 AGI.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        income_record_boundary_documents(),
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income.len(), 101);
    assert!(session.draft.valid_stored());
    let expected = supported(&session.draft);
    assert_eq!(value(&expected, "agi"), 10_010_000);
    let mut cold = restart(session.draft.clone());
    assert_eq!(cold.draft, session.draft);
    let request = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    let P::SaveDocuments { documents, .. } = &request else {
        panic!("ordinary income editor");
    };
    assert_eq!(documents.common.wages.documents.len(), 101);
    accepted(&send(&mut cold, request));
    assert_eq!(supported(&cold.draft), expected);
}

#[test]
fn restored_credit_notices_remain_unique_without_a_numeric_count_cap() {
    let mut draft = complete(FederalStatus::Single);
    draft.credit_refusals = vec![MoreCreditRefusal::OtherCredit];
    assert!(draft.valid_stored());
    draft.credit_refusals.push(MoreCreditRefusal::OtherCredit);
    assert!(!draft.valid_stored());
}
