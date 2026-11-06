use super::*;
use crate::able::{AbleRollover, AbleTreatment};
use alloc::string::ToString;
use libertas::SchemaPath;

fn source(rollover: AbleRollover) -> crate::AbleSource {
    crate::AbleSource {
        form: crate::Form1099QaPaper {
            payer: Default::default(),
            recipient: Default::default(),
            gross: 10_000,
            earnings: 2_000,
            basis: 8_000,
            program_to_program_transfer: false,
            account_terminated: false,
            recipient_not_designated_beneficiary: false,
            ..Default::default()
        },
        treatment: Some(AbleTreatment::Annual { rollover, expenses: 0, following_expenses: 0 }),
    }
}

#[test]
fn unit47_qa1099_native_save_and_preview_locate_each_rejected_source_atomically() {
    let valid = source(AbleRollover::None);
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![
            document(
                0,
                IncomeSource::OtherIncome {
                    value: crate::OtherIncomeDocument::Qa1099(able_document(valid.clone())),
                },
            ),
            document(
                0,
                IncomeSource::OtherIncome {
                    value: crate::OtherIncomeDocument::Qa1099(able_document(valid.clone())),
                },
            ),
        ],
    );
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    let mut boxes = valid.clone();
    boxes.form.basis -= 1;
    let cases = [(boxes, "QA1099_BOXES_DO_NOT_ADD_UP", "gross")];
    for (case_index, (invalid, key, field)) in cases.into_iter().enumerate() {
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        assert_eq!(documents.other.other_income.documents.len(), 2);
        documents.other.other_income.documents[1].document =
            crate::OtherIncomeDocument::Qa1099(able_document(invalid.clone()));
        let requests = [(
            request,
            alloc::format!("[documents/other/other_income/documents/1/document/{field}]"),
        )];
        for (request_index, (request, expected_path)) in requests.into_iter().enumerate() {
            // Exercise the complete wire request as well as the native Save owner.
            let request = P::from_avro(&request.to_avro()).unwrap();
            let dirty = session.dirty;
            let reply = send(&mut session, request.clone());
            let Reply::Response(problem @ P::Problem { .. }) = &reply else {
                panic!("{key} must be refused before changing accepted state")
            };
            let P::Problem { message, field } = problem else { unreachable!() };
            assert_eq!(*message, libertas::libertas_formatted_text(key, &[]));
            assert_eq!(field.as_deref(), Some(expected_path.as_str()));
            assert_eq!(session.draft, saved);
            assert_eq!(session.dirty, dirty);
            crate::tests::unit47_rule_cases::export_unit47_rule_case(
                &alloc::format!("qa-source-{case_index}-{request_index}"),
                &request,
                problem,
            );
        }
        let correction =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        accepted(&send(&mut session, correction));
        assert_eq!(session.draft, saved);
    }
    let mut restored = restart(saved.clone());
    let reopened = crate::tests::support::client::income_pages::open_document_center(&mut restored);
    accepted(&send(&mut restored, reopened));
    assert_eq!(restored.draft, saved);
}

#[test]
fn unit47_qa1099_rollover_equal_bounds_and_loss_keep_actual_source_through_restart() {
    let mut loss = source(AbleRollover::None);
    loss.form.earnings = -2_000;
    loss.form.basis = 12_000;
    for source in [
        source(AbleRollover::None),
        source(AbleRollover::Qualified { amount: 10_000, earnings: 2_000 }),
        source(AbleRollover::Qualified { amount: 2_000, earnings: 2_000 }),
        loss,
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let before = session.draft.clone();
        let _ = crate::tests::support::client::income_pages::saved_document_amount(
            &mut session,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Qa1099(able_document(source.clone())),
            },
        );
        assert_eq!(session.draft, before);
        let request = crate::tests::support::client::income_pages::save_documents(
            &mut session,
            vec![document(
                0,
                IncomeSource::OtherIncome {
                    value: crate::OtherIncomeDocument::Qa1099(able_document(source.clone())),
                },
            )],
        );
        accepted(&send(&mut session, request));
        crate::tests::support::client::income_pages::save_information_return_answers(
            &mut session,
            &[document(
                0,
                IncomeSource::OtherIncome {
                    value: crate::OtherIncomeDocument::Qa1099(able_document(source.clone())),
                },
            )],
        );
        let saved = session.draft.clone();
        assert!(saved.valid_stored());
        // The stored paper prints the household recipient the fixture Save named.
        let IncomeSource::OtherIncome { value: crate::OtherIncomeDocument::Qa1099(printed) } =
            expected_named_source(
                &saved,
                Owner::Taxpayer,
                IncomeSource::OtherIncome {
                    value: crate::OtherIncomeDocument::Qa1099(source.form.clone()),
                },
            )
        else {
            unreachable!()
        };
        let stored = crate::AbleSource { form: printed, ..source.clone() };
        assert!(saved.income.iter().any(|entry| matches!(&entry.income,
            FederalIncome::Able { data } if *data == stored)));
        let mut restored = restart(saved.clone());
        let reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut restored);
        let expected = expected_named_sources(
            &restored.draft,
            Owner::Taxpayer,
            vec![IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Qa1099(able_document(source)),
            }],
        );
        assert!(page_documents(&reopened).iter().any(|item| item.document == expected[0]));
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

#[test]
fn qa1099_printed_save_keeps_treatment_pending_and_corrections_preserve_topic_answers() {
    use crate::tests::support::client::income_pages::{
        open_topic, save_information_return_answers,
    };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let desired = source(AbleRollover::Qualified { amount: 4_000, earnings: 800 });
    let rows = vec![document(
        0,
        IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Qa1099(able_document(desired.clone())),
        },
    )];
    let request =
        crate::tests::support::client::income_pages::save_documents(&mut session, rows.clone());
    accepted(&send(&mut session, request));
    let id = session.draft.income.last().unwrap().id;
    let FederalIncome::Able { data } = &session.draft.income.last().unwrap().income else {
        unreachable!()
    };
    assert!(data.treatment.is_none());
    assert_eq!(crate::AbleDistribution::from(data).taxable(), None);
    assert!(session.draft.valid_stored());
    let Reply::Response(P::Review { allowed_actions, remaining_items, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected Review")
    };
    assert!(!allowed_actions.contains(&FINISH));
    assert!(remaining_items.iter().any(|row| row.section == TaxSection::OtherIncome
        && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER"));
    save_information_return_answers(&mut session, &rows);
    for not_beneficiary in [false, true] {
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        let row =
            documents.other.other_income.documents.iter_mut().find(|row| row.id == id).unwrap();
        let crate::OtherIncomeDocument::Qa1099(value) = &mut row.document else { unreachable!() };
        value.gross = 3_000;
        value.earnings = 600;
        value.basis = 2_400;
        value.recipient_not_designated_beneficiary = not_beneficiary;
        accepted(&send(&mut session, request));
        assert!(session.draft.valid_stored());
        session = restart(session.draft);
        let FederalIncome::Able { data } =
            &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
        else {
            unreachable!()
        };
        assert_eq!(data.treatment, desired.treatment);
        assert_eq!(
            crate::AbleDistribution::from(data).answer_issue().unwrap().key,
            "QA1099_ROLLOVER_ABOVE_GROSS"
        );
        assert!(!estimate(&session.draft).financial_source_scope_complete);
        let topic = open_topic(&mut session, TaxSection::OtherIncome);
        let P::SaveIncomes { followups, .. } = &topic else { unreachable!() };
        let index = followups
            .iter()
            .position(|row| row.key == alloc::format!("income/{id}/treatment"))
            .unwrap();
        let expected = SchemaPath::root()
            .field("followups")
            .key(followups[index].key.as_str())
            .field("answer")
            .field("value")
            .field("rollover")
            .field("amount")
            .to_string();
        let before = session.draft.clone();
        let Reply::Response(P::Problem { field, .. }) = send(&mut session, topic) else {
            panic!("expected topic refusal")
        };
        assert_eq!(field.as_deref(), Some(expected.as_str()));
        assert_eq!(session.draft, before);
    }
    let mut topic = open_topic(&mut session, TaxSection::OtherIncome);
    let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
    let row = followups
        .iter_mut()
        .find(|row| row.key == alloc::format!("income/{id}/treatment"))
        .unwrap();
    row.answer = crate::DocumentAnswer::AbleTreatment {
        value: Some(AbleTreatment::Corrective { year: 2026 }),
    };
    accepted(&send(&mut session, topic));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    assert!(session.draft.income.iter().all(|entry| match &entry.income {
        FederalIncome::Able { data } =>
            crate::AbleDistribution::from(data).answer_issue().is_none(),
        _ => true,
    }));
}

#[test]
fn sa1099_printed_save_keeps_payment_pending_and_retains_records_after_gross_correction() {
    use crate::tests::support::client::income_pages::{
        open_topic, save_information_return_answers,
    };
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut desired = sa_source(Sa1099 {
        trustee_payer: Default::default(),
        recipient: Default::default(),
        origin: crate::HealthDocumentOrigin::Form1099Sa,
        gross: 10_000,
        box2_earnings: None,
        box3_code: Some(1),
        box4_value: None,
        account: crate::MedicalSaAccount::Hsa,
    });
    desired.payment = Some(crate::MedicalSaPayment::Own {
        nonmedical: vec![crate::MedicalAccountWithdrawal { date: 20260615, amount: 5_000 }],
        medical: 5_000,
        rollover: None,
        disabled_since: None,
    });
    let rows = vec![document(
        0,
        IncomeSource::OtherIncome {
            value: crate::OtherIncomeDocument::Sa1099(
                crate::tests::support::client::income_pages::sa_document(desired.clone()),
            ),
        },
    )];
    let request =
        crate::tests::support::client::income_pages::save_documents(&mut session, rows.clone());
    accepted(&send(&mut session, request));
    let id = session.draft.income.last().unwrap().id;
    let FederalIncome::MedicalDistribution { data } = &session.draft.income.last().unwrap().income
    else {
        unreachable!()
    };
    assert!(data.payment.is_none());
    assert_eq!(data.record().preview(), None);
    assert!(session.draft.valid_stored());
    let Reply::Response(P::Review { allowed_actions, remaining_items, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected Review")
    };
    assert!(!allowed_actions.contains(&FINISH));
    assert!(
        remaining_items
            .iter()
            .any(|row| row.section == TaxSection::Hsa
                && crate::Message::decode(&row.item).key == "DOCUMENT_PENDING_ANSWER")
    );
    save_information_return_answers(&mut session, &rows);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let row = documents.other.other_income.documents.iter_mut().find(|row| row.id == id).unwrap();
    let crate::OtherIncomeDocument::Sa1099(value) = &mut row.document else { unreachable!() };
    value.gross_distribution = 3_000;
    accepted(&send(&mut session, request));
    assert!(session.draft.valid_stored());
    session = restart(session.draft);
    let FederalIncome::MedicalDistribution { data } =
        &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.payment, desired.payment);
    assert_eq!(data.record().answer_issue().unwrap().key, "SA1099_MEDICAL_ABOVE_GROSS");
    assert!(!estimate(&session.draft).financial_source_scope_complete);
    let mut topic = open_topic(&mut session, TaxSection::Hsa);
    let P::SaveHsa { followups, .. } = &topic else { unreachable!() };
    let index =
        followups.iter().position(|row| row.key == alloc::format!("income/{id}/payment")).unwrap();
    let before = session.draft.clone();
    let Reply::Response(P::Problem { field, .. }) = send(&mut session, topic.clone()) else {
        panic!("expected topic refusal")
    };
    assert_eq!(
        field,
        Some(
            SchemaPath::root()
                .field("followups")
                .key(followups[index].key.as_str())
                .field("answer")
                .field("value")
                .field("medical")
                .to_string()
        )
    );
    assert_eq!(session.draft, before);
    let P::SaveHsa { followups, .. } = &mut topic else { unreachable!() };
    followups[index].answer = crate::HsaDocumentAnswer::MedicalPayment {
        value: Some(crate::MedicalSaPayment::Own {
            nonmedical: vec![],
            medical: 3_000,
            rollover: None,
            disabled_since: None,
        }),
    };
    accepted(&send(&mut session, topic));
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    let FederalIncome::MedicalDistribution { data } =
        &session.draft.income.iter().find(|entry| entry.id == id).unwrap().income
    else {
        unreachable!()
    };
    assert!(data.record().answer_issue().is_none());
    assert_eq!(data.record().preview(), Some(0));
}

pub(crate) fn client_fixture(case: &str) -> (Interview, P, P) {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let initial = crate::tests::support::client::income_pages::save_documents(
        &mut session,
        vec![document(
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Qa1099(able_document(source(
                    AbleRollover::None,
                ))),
            },
        )],
    );
    accepted(&send(&mut session, initial));
    let seed = crate::tests::support::client::income_pages::open_document_center(&mut session);
    let rows = vec![
        document(
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Qa1099(able_document(source(
                    AbleRollover::None,
                ))),
            },
        ),
        document(
            0,
            IncomeSource::OtherIncome {
                value: crate::OtherIncomeDocument::Qa1099(able_document(client_source(case))),
            },
        ),
    ];
    let guide = crate::tests::support::client::income_pages::save_documents(&mut session, rows);
    (session, seed, guide)
}

fn client_source(case: &str) -> crate::AbleSource {
    source(match case {
        "qa-boxes" => AbleRollover::None,
        "qa-rollover-amount" => AbleRollover::Qualified { amount: 10_000, earnings: 2_000 },
        "qa-rollover-earnings" => AbleRollover::Qualified { amount: 2_000, earnings: 2_000 },
        _ => panic!("unknown 1099-QA client case"),
    })
}

pub(crate) fn client_reopened(session: &mut Interview) -> P {
    crate::tests::support::client::income_pages::open_document_center(session)
}

pub(crate) fn assert_client_sources(session: &Interview, case: &str) {
    let sources: Vec<_> = session
        .draft
        .income
        .iter()
        .filter_map(|entry| {
            if let FederalIncome::Able { data } = &entry.income {
                assert!(entry.id > 0);
                Some(data.clone())
            } else {
                None
            }
        })
        .collect();
    let mut expected = vec![source(AbleRollover::None), client_source(case)];
    for source in &mut expected {
        source.treatment = None;
    }
    assert_eq!(sources, expected);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}
