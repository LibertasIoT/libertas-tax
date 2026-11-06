use super::*;
use crate::tests::cross::interview::{accepted, document};
use crate::tests::support::client::income_pages::{
    open_document_center, open_topic, save_documents,
};

// The Document Center row is the printed paper; an OID statement's question
// rows on the Interest and dividends page that ask for an answer.
fn pending_interest_questions(draft: &FederalDraft) -> usize {
    crate::interview::remaining_items(draft, &supported(draft), TaxSection::InterestDividends)
        .into_iter()
        .filter(|row| row.section == TaxSection::InterestDividends && row.field.is_some())
        .count()
}

fn save_oid_paper(session: &mut Interview, form: Form1099OidPaper) -> i64 {
    let request = save_documents(
        session,
        vec![document(
            0,
            IncomeSource::InterestDividend { value: InterestDividendDocument::Oid1099(form) },
        )],
    );
    accepted(&send(session, request));
    session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Oid { .. }))
        .unwrap()
        .id
}

fn stored_oid(session: &Interview, id: i64) -> &Form1099OidSource {
    let FederalIncome::Oid { data } =
        &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    data
}

fn answer_treatment(
    session: &mut Interview,
    id: i64,
    value: Option<OidSpecial>,
) -> Option<OidSpecial> {
    let mut page = open_topic(session, TaxSection::InterestDividends);
    let P::SaveIncomes { followups, .. } = &mut page else { unreachable!() };
    let row =
        followups.iter_mut().find(|row| row.key == alloc::format!("income/{id}/special")).unwrap();
    let DocumentAnswer::OidTreatment { value: shown } = row.answer else { unreachable!() };
    if let Some(value) = value {
        row.answer = DocumentAnswer::OidTreatment { value: Some(value) };
    }
    accepted(&send(session, page));
    shown
}

// AI Import shape of the Smith family's negative Treasury paper (round 3,
// batch 6): only the printed boxes arrive; no treatment is printed or decided.
#[test]
fn imported_negative_treasury_oid_counts_and_asks_its_treatment_on_the_topic() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let paper = Form1099OidPaper {
        payer: crate::PrintedParty {
            name: "Example Treasury Discount Custodian".into(),
            federal_id: "00-0000115".into(),
            address: "10 Example Finance Way\nExample Grove VA 00000".into(),
        },
        payer_telephone: "000-000-0100".into(),
        // The printed recipient is this household's filer (John Smith on the paper).
        recipient: crate::PrintedParty {
            name: fixture_document_names(&session.draft, -1).unwrap().recipient.name,
            federal_id: "987-65-4320".into(),
            address: "100 Example Orchard Lane\nExample Grove VA 00000".into(),
        },
        description: "Example Treasury inflation adjustment".into(),
        original_issue_discount_on_us_treasury_obligations: -1_525,
        ..Default::default()
    };
    let id = save_oid_paper(&mut session, paper.clone());
    assert_eq!(stored_oid(&session, id).special, None, "the import decides no treatment");
    let result = supported(&session.draft);
    assert!(!noted(&result, "WARN_OID_OTHER_LEFT_OUT"));
    assert!(!noted(&result, "WARN_OID_OBLIGATION_LEFT_OUT"));
    assert_eq!(crate::FederalOid::from(stored_oid(&session, id)).treasury_oid, -1_525);
    // The treatment question, and the page's situations question for the new paper.
    assert_eq!(pending_interest_questions(&session.draft), 2, "asked on the topic page");
    let before = session.draft.clone();
    let page = open_topic(&mut session, TaxSection::InterestDividends);
    let P::SaveIncomes { followups, .. } = &page else { unreachable!() };
    let row =
        followups.iter().find(|row| row.key == alloc::format!("income/{id}/special")).unwrap();
    assert_eq!(row.answer, DocumentAnswer::OidTreatment { value: None }, "shown unanswered");
    assert_eq!(session.draft, before, "opening the topic accepts no treatment");
    assert_eq!(answer_treatment(&mut session, id, Some(OidSpecial::NoneOfThese)), None);
    assert_eq!(stored_oid(&session, id).special, Some(OidSpecial::NoneOfThese));
    assert_eq!(pending_interest_questions(&session.draft), 0);
    let page = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &page else { unreachable!() };
    let reopened =
        documents.common.interest_dividends.documents.iter().find(|row| row.id == id).unwrap();
    assert_eq!(reopened.document, InterestDividendDocument::Oid1099(paper));
}

#[test]
fn oid_treatment_is_a_visible_topic_answer_the_paper_never_carries() {
    for special in [OidSpecial::NoneOfThese, OidSpecial::Obligation, OidSpecial::Another] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let paper = Form1099OidPaper {
            description: "STRIPPED BOND — original description".into(),
            original_issue_discount_for_the_year: 12_345,
            ..Default::default()
        };
        let id = save_oid_paper(&mut session, paper);
        let paper = stored_oid(&session, id).form.clone();
        assert_eq!(stored_oid(&session, id).special, None);
        assert_eq!(answer_treatment(&mut session, id, Some(special)), None);
        assert_eq!(stored_oid(&session, id).special, Some(special));
        let result = supported(&session.draft);
        assert_eq!(
            return_line(&result, "2b"),
            if special == OidSpecial::NoneOfThese { 12_300 } else { 0 },
        );
        assert_eq!(noted(&result, "WARN_OID_OTHER_LEFT_OUT"), special == OidSpecial::Another);
        // Still shown, with the saved answer, so the person can change it.
        let mut cold = restart(session.draft);
        assert_eq!(answer_treatment(&mut cold, id, None), Some(special));
        assert_eq!(pending_interest_questions(&cold.draft), 0);
        let page = open_document_center(&mut cold);
        let P::SaveDocuments { documents, .. } = &page else { unreachable!() };
        let reopened =
            documents.common.interest_dividends.documents.iter().find(|row| row.id == id).unwrap();
        assert_eq!(reopened.document, InterestDividendDocument::Oid1099(paper));
        assert_eq!(stored_oid(&cold, id).form.description, "STRIPPED BOND — original description");
    }
}

#[test]
fn oid_unresolved_paper_shows_its_treatment_unanswered() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let id = save_oid_paper(
        &mut session,
        Form1099OidPaper {
            description: "STRIPPED BOND".into(),
            original_issue_discount_for_the_year: 12_345,
            ..Default::default()
        },
    );
    assert_eq!(stored_oid(&session, id).special, None);
    assert_eq!(return_line(&supported(&session.draft), "2b"), 12_300);
    let before = session.draft.clone();
    let page = open_topic(&mut session, TaxSection::InterestDividends);
    let P::SaveIncomes { followups, .. } = &page else { unreachable!() };
    let row =
        followups.iter().find(|row| row.key == alloc::format!("income/{id}/special")).unwrap();
    assert_eq!(row.answer, DocumentAnswer::OidTreatment { value: None }, "no box 7 prefill");
    assert_eq!(session.draft, before, "opening the topic accepts no treatment");
    accepted(&send(&mut session, page));
    assert_eq!(stored_oid(&session, id).special, None, "saving leaves it unanswered");
    assert_eq!(pending_interest_questions(&session.draft), 1);
    assert_eq!(answer_treatment(&mut session, id, Some(OidSpecial::Obligation)), None);
    assert_eq!(stored_oid(&session, id).special, Some(OidSpecial::Obligation));
    let result = supported(&session.draft);
    assert_eq!(return_line(&result, "2b"), 0);
    assert!(noted(&result, "WARN_OID_OBLIGATION_LEFT_OUT"));
    let mut cold = restart(session.draft);
    assert_eq!(pending_interest_questions(&cold.draft), 0);
    assert_eq!(return_line(&supported(&cold.draft), "2b"), 0);
    assert_eq!(answer_treatment(&mut cold, id, None), Some(OidSpecial::Obligation));
}

#[test]
fn oid_topic_treatment_survives_restart_paper_correction_and_its_own_correction() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let id = save_oid_paper(
        &mut session,
        Form1099OidPaper {
            description: "STRIPPED BOND".into(),
            original_issue_discount_for_the_year: 12_345,
            ..Default::default()
        },
    );
    answer_treatment(&mut session, id, Some(OidSpecial::NoneOfThese));
    let mut cold = restart(session.draft);
    let mut page = open_document_center(&mut cold);
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    let row =
        documents.common.interest_dividends.documents.iter_mut().find(|row| row.id == id).unwrap();
    let InterestDividendDocument::Oid1099(form) = &mut row.document else { unreachable!() };
    form.corrected = true;
    form.description = "Contingent-payment debt instrument".into();
    form.original_issue_discount_for_the_year = 23_456;
    accepted(&send(&mut cold, page));
    assert_eq!(stored_oid(&cold, id).special, Some(OidSpecial::NoneOfThese));
    assert_eq!(answer_treatment(&mut cold, id, None), Some(OidSpecial::NoneOfThese));
    let result = supported(&cold.draft);
    assert_eq!(return_line(&result, "2b"), 23_500);
    assert!(!noted(&result, "WARN_OID_OBLIGATION_LEFT_OUT"));
    for special in [OidSpecial::Another, OidSpecial::Obligation, OidSpecial::NoneOfThese] {
        answer_treatment(&mut cold, id, Some(special));
        assert_eq!(stored_oid(&cold, id).special, Some(special));
    }
}
