use crate::tests::cross::interview::{accepted, open_topic, save_topic};
use crate::tests::support::builders::wage_documents;
use crate::tests::*;

#[test]
fn large_income_record_count_saves_reopens_and_estimates() {
    // D-2026-10-15-13: record count is bounded only by the platform payload,
    // not the former 100-source prototype cap. No financial limit changes.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_topic(
        &session.draft,
        TaxSection::Wages,
        wage_documents(1_001, 1_000_000, 0, 0),
    );
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.income.len(), 1_001);
    assert!(session.draft.valid_stored());
    let expected = supported(&session.draft);
    assert_eq!(value(&expected, "agi"), 10_010_000);
    let mut cold = restart(session.draft.clone());
    assert_eq!(cold.draft, session.draft);
    let request = open_topic(&mut cold, TaxSection::Wages);
    let P::SaveIncomes { documents, .. } = &request else {
        panic!("ordinary income editor");
    };
    assert_eq!(documents.documents.len(), 1_001);
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
