use super::unit47_credit_source_validation::credit_estimate_documents_from_save;
use super::*;

#[test]
fn unit47_related_exchange_transfer_guard_uses_submitted_data_answer_position() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let first = related_exchange_kind(20260430);
    let mut second = related_exchange_kind(20260101);
    second.property = "28 Ash St".into();
    second.last_transfer = 20251231;
    let corrected = save_topic(
        &mut session,
        TaxSection::InvestmentSales,
        vec![
            document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::RelatedExchange(first) },
            ),
            document(
                0,
                IncomeSource::Record { value: crate::IncomeRecord::RelatedExchange(second) },
            ),
        ],
    );
    let before = session.draft.clone();
    let dirty = session.dirty;
    accepted(&crate::tests::support::client::income_pages::candidate_save_review(
        &mut session,
        credit_estimate_documents_from_save(&corrected),
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, corrected));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "related_exchange_1_gain"), 90_000);
    assert_eq!(value(&result, "related_exchange_2_gain"), 90_000);
    assert_eq!(value(&result, "agi"), 240_000);
    let saved = session.draft.clone();
    let reopened = open_topic(&mut session, TaxSection::InvestmentSales);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let reopened = open_topic(&mut cold, TaxSection::InvestmentSales);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
    assert_eq!(supported(&cold.draft), result);
}
