use super::*;
use crate::tests::cross::interview::{accepted, open_hsa_page};
use crate::tests::support::builders::survey::survey_uncatalogued_document as item;
use crate::tests::support::client::income_pages::{open_document_center, save_documents};

#[test]
fn trustee_statement_saves_before_its_event_answer_and_retains_it_on_correction() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let statement = HsaTrusteeStatementPaper {
        trustee_payer: PrintedParty::default(),
        recipient: PrintedParty::default(),
        account_number: "Synthetic-MSA-1".into(),
        amount: 100_000,
        account: MedicalSaAccount::Archer,
    };
    let request = save_documents(
        &mut session,
        vec![item(
            Owner::Taxpayer,
            0,
            IncomeSource::OtherIncome {
                value: OtherIncomeDocument::HsaTrusteeStatement(statement),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let entry = session.draft.income.last().unwrap();
    let id = entry.id;
    let FederalIncome::MedicalDistribution { data } = &entry.income else { unreachable!() };
    assert_eq!(data.origin, HealthDocumentOrigin::TrusteeStatement);
    assert!(data.form1099sa().is_none());
    assert_eq!(data.internal().box3_code, None);
    assert!(data.payment.is_none());
    assert_eq!(data.topic_issue(true).unwrap().key, "SA1099_PAYMENT_RECORDS_NEEDED");
    assert!(session.draft.valid_stored());
    let Reply::Response(P::Review { remaining_items, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected Review")
    };
    assert!(remaining_items.iter().any(|row| row.section == TaxSection::Hsa
        && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER"));
    assert!(!estimate(&session.draft).financial_source_scope_complete);

    let mut request = open_hsa_page(&mut session);
    let P::SaveHsa { followups, .. } = &mut request else { unreachable!() };
    let row =
        followups.iter_mut().find(|row| row.key == alloc::format!("income/{id}/payment")).unwrap();
    assert!(matches!(row.answer, HsaDocumentAnswer::MedicalPayment { value: None }));
    assert_eq!(
        row.allowed_values,
        vec![libertas_macros::variant_index!(MedicalSaPayment::Unreported) as i32],
    );
    let payment = MedicalSaPayment::Unreported {
        event: MedicalDeemed::Pledged {
            assets: vec![MedicalAccountWithdrawal { date: 20260614, amount: 100_000 }],
        },
        disabled_since: None,
    };
    row.answer = HsaDocumentAnswer::MedicalPayment { value: Some(payment.clone()) };
    accepted(&send(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "archer_msa_taxable_distributions"), 1000);
    assert_eq!(value(&result, "archer_msa_additional_tax"), 200);

    let mut request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let row = documents.other.other_income.documents.iter_mut().find(|row| row.id == id).unwrap();
    let OtherIncomeDocument::HsaTrusteeStatement(statement) = &mut row.document else {
        unreachable!()
    };
    statement.amount = 110_000;
    accepted(&send(&mut session, request));
    session = restart(session.draft);
    let FederalIncome::MedicalDistribution { data } =
        &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.origin, HealthDocumentOrigin::TrusteeStatement);
    assert_eq!(data.trustee_statement_paper().unwrap().amount, 110_000);
    assert_eq!(data.payment, Some(payment));
    assert_eq!(data.topic_issue(true).unwrap().key, "SA1099_EVENT_VALUE_NOT_GROSS");
    assert!(!estimate(&session.draft).financial_source_scope_complete);
}

#[test]
fn form1099sa_without_its_printed_code_still_refuses_in_document_center() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_documents(
        &mut session,
        vec![item(
            Owner::Taxpayer,
            0,
            IncomeSource::OtherIncome {
                value: OtherIncomeDocument::Sa1099(Form1099SaPaper {
                    gross_distribution: 100_000,
                    account: MedicalSaAccount::Archer,
                    distribution_code: None,
                    ..Default::default()
                }),
            },
        )],
    );
    let before = session.draft.clone();
    let Reply::Response(P::Problem { message, field, .. }) = send(&mut session, request) else {
        panic!("expected missing printed-code refusal")
    };
    assert_eq!(message, libertas::libertas_formatted_text("SA1099_PRINTED_CODE_NEEDED", &[]));
    assert!(field.unwrap().contains("distribution_code"));
    assert_eq!(session.draft, before);
}

// A trustee statement's payment records are a required question with no
// default: asked first on their own page, stored once, and not asked again.
#[test]
fn medical_payment_records_are_a_required_question_asked_before_the_page() {
    use crate::tests::support::client::own_questions::{
        answer_questions, save_questions, wire, wire_page,
    };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let statement = HsaTrusteeStatementPaper {
        trustee_payer: PrintedParty::default(),
        recipient: PrintedParty::default(),
        account_number: "Synthetic-MSA-1".into(),
        amount: 100_000,
        account: MedicalSaAccount::Archer,
    };
    let request = save_documents(
        &mut session,
        vec![item(
            Owner::Taxpayer,
            0,
            IncomeSource::OtherIncome {
                value: OtherIncomeDocument::HsaTrusteeStatement(statement),
            },
        )],
    );
    accepted(&send(&mut session, request));
    let id = session.draft.income.last().unwrap().id;
    let page = open_hsa_page(&mut session);
    let begin = match wire_page(&mut session, Reply::Edit(page)) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. }) => begin,
        other => panic!("expected the questions page, got {other:?}"),
    };
    let P::BeginDocumentQuestions { topic, questions, .. } = &begin else { unreachable!() };
    assert_eq!(*topic, TaxSection::Hsa);
    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].key, alloc::format!("income/{id}/payment"));
    // The question blocks Save: nothing is stored without an answer.
    let before = session.draft.clone();
    assert!(matches!(
        save_questions(&mut session, &begin, Vec::new()),
        Reply::Response(P::Problem { .. })
    ));
    assert_eq!(session.draft, before);

    let payment = MedicalSaPayment::Unreported {
        event: MedicalDeemed::Pledged {
            assets: vec![MedicalAccountWithdrawal { date: 20260614, amount: 100_000 }],
        },
        disabled_since: None,
    };
    let answer = crate::DocumentStrictAnswer::MedicalPayment { value: payment.clone() };
    let reply = answer_questions(&mut session, &begin, vec![answer.clone()]);
    let Reply::Edit(page @ P::SaveHsa { .. }) = reply else {
        panic!("expected the health accounts page, got {reply:?}")
    };
    let P::SaveHsa { followups, questions, .. } = &page else { unreachable!() };
    assert!(followups.is_empty());
    assert_eq!(questions.len(), 1);
    assert_eq!(crate::OwnAsked::parts(questions[0].clone()).1, answer);
    let FederalIncome::MedicalDistribution { data } =
        &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.payment, Some(payment));

    // Saved through the endpoint and restarted, the page opens without asking.
    accepted(&wire(&mut session, page));
    let mut restored = restart(session.draft);
    let page = open_hsa_page(&mut restored);
    let reply = wire_page(&mut restored, Reply::Edit(page));
    let Reply::Edit(P::SaveHsa { questions, .. }) = reply else {
        panic!("a settled payment opens the page, got {reply:?}")
    };
    assert_eq!(crate::OwnAsked::parts(questions[0].clone()).1, answer);
}
