use super::*;
use crate::able::{AbleRollover, AbleTreatment, Qa1099};

fn source(rollover: AbleRollover) -> Qa1099 {
    Qa1099 {
        gross: 10_000,
        earnings: 2_000,
        basis: 8_000,
        treatment: AbleTreatment::Annual {
            rollover,
            expenses: 0,
            following_expenses: 0,
        },
    }
}

fn preview(source: Qa1099) -> P {
    crate::tests::support::client::income_pages::catalog_preview_document(IncomeDocument::Qa1099(source))
}

#[test]
fn unit47_qa1099_native_save_and_preview_locate_each_rejected_source_atomically() {
    let valid = source(AbleRollover::None);
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = save_topic(
        &session.draft,
        TaxSection::OtherIncome,
        vec![
            document(0, IncomeDocument::Qa1099(valid.clone())),
            document(0, IncomeDocument::Qa1099(valid.clone())),
        ],
    );
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    let mut boxes = valid.clone();
    boxes.basis -= 1;
    let mut overflow = valid.clone();
    overflow.earnings = i64::MAX;
    overflow.basis = 1;
    let cases = [
        (boxes, "QA1099_BOXES_DO_NOT_ADD_UP", "gross"),
        (
            source(AbleRollover::Qualified {
                amount: 10_001,
                earnings: 2_000,
            }),
            "QA1099_ROLLOVER_ABOVE_GROSS",
            "treatment/rollover/amount",
        ),
        (
            source(AbleRollover::Qualified {
                amount: 1_000,
                earnings: 1_001,
            }),
            "QA1099_ROLLOVER_EARNINGS_ABOVE_AMOUNT",
            "treatment/rollover/earnings",
        ),
        (overflow, "QA1099_BOXES_DO_NOT_ADD_UP", "gross"),
    ];
    for (case_index, (invalid, key, field)) in cases.into_iter().enumerate() {
        let mut request = open_topic(&mut session, TaxSection::OtherIncome);
        let P::SaveIncomes { documents, .. } = &mut request else {
            unreachable!()
        };
        assert_eq!(documents.documents.len(), 2);
        documents.documents[1].document = IncomeDocument::Qa1099(invalid.clone());
        let requests = [
            (preview(invalid), alloc::format!("[document/{field}]")),
            (
                request,
                alloc::format!("[documents/documents/1/document/{field}]"),
            ),
        ];
        for (request_index, (request, expected_path)) in requests.into_iter().enumerate() {
            // Exercise the complete wire request as well as the native Save owner.
            let request = P::from_avro(&request.to_avro()).unwrap();
            let dirty = session.dirty;
            let reply = send(&mut session, request.clone());
            let Reply::Response(problem @ P::Problem { .. }) = &reply else {
                panic!("{key} must be refused before changing accepted state")
            };
            let P::Problem { message, field } = problem else {
                unreachable!()
            };
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
        let correction = open_topic(&mut session, TaxSection::OtherIncome);
        accepted(&send(&mut session, correction));
        assert_eq!(session.draft, saved);
    }
    let mut restored = restart(saved.clone());
    let reopened = open_topic(&mut restored, TaxSection::OtherIncome);
    accepted(&send(&mut restored, reopened));
    assert_eq!(restored.draft, saved);
}

#[test]
fn unit47_qa1099_rollover_equal_bounds_and_loss_keep_actual_source_through_restart() {
    let mut loss = source(AbleRollover::None);
    loss.earnings = -2_000;
    loss.basis = 12_000;
    for source in [
        source(AbleRollover::None),
        source(AbleRollover::Qualified {
            amount: 10_000,
            earnings: 2_000,
        }),
        source(AbleRollover::Qualified {
            amount: 2_000,
            earnings: 2_000,
        }),
        loss,
    ] {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let before = session.draft.clone();
        assert!(matches!(
            send(&mut session, preview(source.clone())),
            Reply::Response(P::IncomePreview { .. })
        ));
        assert_eq!(session.draft, before);
        let request = save_topic(
            &session.draft,
            TaxSection::OtherIncome,
            vec![document(0, IncomeDocument::Qa1099(source.clone()))],
        );
        accepted(&send(&mut session, request));
        let saved = session.draft.clone();
        assert!(saved.valid_stored());
        assert!(saved.income.iter().any(|entry| matches!(&entry.income,
            FederalIncome::Able { data } if *data == source.record())));
        let mut restored = restart(saved.clone());
        let reopened = open_topic(&mut restored, TaxSection::OtherIncome);
        assert!(
            page_documents(&reopened)
                .iter()
                .any(|item| matches!(&item.document,
            IncomeDocument::Qa1099(actual) if *actual == source))
        );
        accepted(&send(&mut restored, reopened));
        assert_eq!(restored.draft, saved);
    }
}

pub(crate) fn client_fixture(case: &str) -> (Interview, P, P) {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let initial = save_topic(
        &session.draft,
        TaxSection::OtherIncome,
        vec![document(
            0,
            IncomeDocument::Qa1099(source(AbleRollover::None)),
        )],
    );
    accepted(&send(&mut session, initial));
    let seed = open_topic(&mut session, TaxSection::OtherIncome);
    let mut guide = seed.clone();
    let P::SaveIncomes { documents, .. } = &mut guide else {
        unreachable!()
    };
    documents.documents = vec![
        document(0, IncomeDocument::Qa1099(source(AbleRollover::None))),
        document(0, IncomeDocument::Qa1099(client_source(case))),
    ];
    (session, seed, guide)
}

fn client_source(case: &str) -> Qa1099 {
    source(match case {
        "qa-boxes" => AbleRollover::None,
        "qa-rollover-amount" => AbleRollover::Qualified {
            amount: 10_000,
            earnings: 2_000,
        },
        "qa-rollover-earnings" => AbleRollover::Qualified {
            amount: 2_000,
            earnings: 2_000,
        },
        _ => panic!("unknown 1099-QA client case"),
    })
}

pub(crate) fn client_reopened(session: &mut Interview) -> P {
    open_topic(session, TaxSection::OtherIncome)
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
    assert_eq!(
        sources,
        vec![
            source(AbleRollover::None).record(),
            client_source(case).record()
        ]
    );
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}
