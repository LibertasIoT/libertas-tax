use crate::tests::support::builders::native_income::document;
use crate::tests::support::client::income_pages::{
    open_document_center, page_documents, save_topic,
};
use crate::tests::*;

type Exchange = fn(&mut Interview, P, &str, bool) -> Reply;

pub(in crate::tests) fn ptp_save_prior_carry(
    session: &mut Interview,
    case: &str,
    exchange: Exchange,
) {
    let request = save_topic(
        session,
        TaxSection::SelfEmployment,
        vec![document(
            0,
            IncomeSource::Record {
                value: crate::IncomeRecord::QbiLossCarryforward(QbiLossCarryforward {
                    business: 0,
                    reit_ptp: 300_000,
                    injured_owners: None,
                }),
            },
        )],
    );
    accepted(&exchange(session, request, case, false));
}

pub(in crate::tests) fn ptp_reopen_and_cold_resave(
    session: &mut Interview,
    expected: &[IncomeSource],
    result: &FederalResult,
    case: &str,
    exchange: Exchange,
) {
    let saved = session.draft.clone();
    let reopened = open_document_center(session);
    let documents: Vec<_> = page_documents(&reopened)
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::K1 { .. }))
        .collect();
    assert_eq!(documents.len(), expected.len());
    let mut expected = expected_named_sources(&session.draft, Owner::Taxpayer, expected.to_vec());
    for (actual, expected) in documents.iter().zip(&mut expected) {
        if let IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(document) } = expected {
            // Keep the supplied printed statements intact while assigning only
            // the identities allocated by the accepted source Save.
            let mut expected_source = estate_fixture_source(document);
            // 2026-10-27: the joint paper's zero section 199A amount has no
            // reported-dividend record; its separate printed code-I flag stays intact.
            if document.statements.section199a_dividends == 0 {
                expected_source.statements.section199a_dividends = crate::ReitDividends::None;
            }
            let entry = saved.income.iter().find(|entry| entry.id == actual.id).unwrap();
            let FederalIncome::EstateTrustPortfolio { data } = &entry.income else {
                unreachable!()
            };
            assert_eq!(
                expected_source.statements.qbi.statements.len(),
                data.statements.qbi.statements.len()
            );
            for (statement, accepted) in expected_source
                .statements
                .qbi
                .statements
                .iter_mut()
                .zip(&data.statements.qbi.statements)
            {
                if statement.id == 0 {
                    assert!(accepted.id > 0);
                    statement.id = accepted.id;
                }
            }
            *document = crate::K1EstateTrustDocument::from_source(&expected_source);
            assert_eq!(data.statements, expected_source.statements);
            assert_eq!(data.answers, expected_source.answers);
        }
    }
    for (index, (actual, expected)) in documents.iter().zip(&expected).enumerate() {
        assert!(actual.id > 0);
        assert_eq!(&actual.document, expected);
        assert!(documents[..index].iter().all(|prior| prior.id != actual.id));
    }
    accepted(&exchange(session, reopened, &alloc::format!("{case}-resave"), false));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let reopened = open_document_center(&mut cold);
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
    if matches!(request, P::SaveDocuments { .. }) {
        let rows = page_documents(&request);
        let mut candidate = restart(session.draft.clone());
        accepted(&super::income_pages::save_source_fixtures(&mut candidate, rows.clone()));
        let Reply::Response(P::Review { estimate, .. }) = exchange(
            &mut candidate,
            P::ReviewAccepted { user: TEST_USER },
            &alloc::format!("{case}-estimate"),
            false,
        ) else {
            panic!("actual source and topic candidate review")
        };
        accepted(&super::income_pages::save_source_fixtures(session, rows));
        return estimate;
    }
    let estimated =
        exchange(session, estimate_request(&request), &alloc::format!("{case}-estimate"), true);
    let estimate = match estimated {
        Reply::Response(P::IncomeRunningEstimate { estimate, .. }) => {
            estimate.map(|e| e.line).unwrap_or_default()
        }
        Reply::Response(P::Review { estimate, .. }) => estimate,
        _ => panic!("actual candidate save review or topic running estimate"),
    };
    accepted(&exchange(session, request, &alloc::format!("{case}-save"), false));
    estimate
}
