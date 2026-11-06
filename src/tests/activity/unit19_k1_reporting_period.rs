use super::*;
use crate::tests::support::client::income_pages::{open_documents, save_documents};
use crate::tests::support::client::unit50_interview_native::NativeInterviewCase;
use crate::tests::support::client::{
    draft_from_native_records, native_draft_records, write_native_draft_records,
};

fn period(began: u32, ended: u32) -> K1EntityTaxPeriod {
    K1EntityTaxPeriod { began, ended }
}
fn page(partner: bool, name: &str, actual: Option<K1EntityTaxPeriod>) -> IncomeSource {
    if partner {
        let mut page = k1_1065(name);
        page.reporting_period = actual;
        IncomeSource::K1 { value: crate::K1Document::K1Partnership((page).into()) }
    } else {
        let mut page = k1_1120s(name);
        page.reporting_period = actual;
        IncomeSource::K1 { value: crate::K1Document::K1Scorp((page).into()) }
    }
}
fn entered(value: &crate::K1Document) -> Option<K1EntityTaxPeriod> {
    match value {
        crate::K1Document::K1Partnership(page) => page.form.reporting_period,
        crate::K1Document::K1Scorp(page) => page.form.reporting_period,
        _ => unreachable!(),
    }
}
fn set_period(value: &mut crate::K1Document, period: Option<K1EntityTaxPeriod>) {
    match value {
        crate::K1Document::K1Partnership(page) => page.form.reporting_period = period,
        crate::K1Document::K1Scorp(page) => page.form.reporting_period = period,
        _ => unreachable!(),
    }
}
fn stored(entry: &FederalIncomeEntry) -> Option<K1EntityTaxPeriod> {
    match &entry.income {
        FederalIncome::Partnership { data } => data.reporting_period,
        FederalIncome::Scorp { data } => ScorpK1::from(data).reporting_period,
        _ => unreachable!(),
    }
}

#[test]
fn k1_entity_period_save_checks_actual_dates_without_calendar_inference() {
    for partner in [true, false] {
        for actual in [
            None,
            Some(period(20260101, 20261231)),
            Some(period(20250701, 20260630)),
            Some(period(20251001, 20260930)),
            Some(period(20260301, 20260331)),
        ] {
            let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
            let request = save_documents(
                &mut session,
                vec![document(0, page(partner, "Actual entity", actual))],
            );
            accepted(&send(&mut session, P::from_avro(&request.to_avro()).unwrap()));
            assert_eq!(
                stored(
                    session
                        .draft
                        .income
                        .iter()
                        .find(|entry| matches!(
                            entry.income,
                            FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. }
                        ))
                        .unwrap()
                ),
                actual
            );
            let saved = session.draft.clone();
            let mut cold = restart(saved.clone());
            let reopened = open_documents(&mut cold);
            let P::SaveDocuments { documents, .. } = &reopened else { unreachable!() };
            assert_eq!(entered(&documents.common.k1.documents[0].document), actual);
            accepted(&send(&mut cold, reopened));
            assert_eq!(cold.draft, saved);
            {
                let invalid = period(20260630, 20250701);
                let mut request = open_documents(&mut session);
                let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
                set_period(&mut documents.common.k1.documents[0].document, Some(invalid));
                assert_eq!(
                    match &documents.common.k1.documents[0].document {
                        crate::K1Document::K1Partnership(page) =>
                            page.issue_with_mode(false).map(|issue| issue.key),
                        crate::K1Document::K1Scorp(page) =>
                            page.issue_with_mode(false).map(|issue| issue.key),
                        _ => unreachable!(),
                    },
                    Some("K1_ENTITY_PERIOD_DATES")
                );
                error(crate::tests::support::client::income_pages::save_source_fixture_request(
                    &mut session,
                    request,
                ));
                assert_eq!(session.draft, saved);
            }
        }
    }
}

#[test]
fn k1_entity_period_actual_client_save_reopen_restart_and_correction() {
    extern crate std;
    let partner = std::env::var("TAX_UNIT19_PERIOD_KIND").as_deref() != Ok("scorp");
    let exact_periods = [Some(period(20250701, 20260630)), None];
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let seed = open_documents(&mut session);
    let native = NativeInterviewCase("TAX_UNIT19_PERIOD");
    if native.required_calculation(&mut session) {
        return;
    }
    let answers = save_documents(
        &mut session,
        vec![
            document(0, page(partner, "Known fiscal-period entity", exact_periods[0])),
            document(0, page(partner, "Unknown-period entity", exact_periods[1])),
        ],
    );
    let directory = std::env::var("TAX_UNIT19_PERIOD_DIR").ok().map(std::path::PathBuf::from);
    let write = |name: &str, request: &P| {
        if let Some(directory) = &directory {
            std::fs::create_dir_all(directory).unwrap();
            std::fs::write(directory.join(name), request.to_avro()).unwrap();
        }
    };
    write("seed.avro", &seed);
    write("answers.avro", &answers);
    if std::env::var("TAX_UNIT19_PERIOD_SEED_ONLY").as_deref() == Ok("1") {
        return;
    }
    let payload = native.input("PAYLOAD").unwrap_or(answers);
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        P::from_avro(&payload.to_avro()).unwrap(),
    ));
    let saved = session.draft.clone();
    let sources: Vec<_> = saved
        .income
        .iter()
        .filter(|entry| {
            matches!(entry.income, FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. })
        })
        .collect();
    assert_eq!(sources.iter().map(|entry| stored(entry)).collect::<Vec<_>>(), exact_periods);
    assert_ne!(sources[0].id, sources[1].id);
    let result = supported(&saved);
    let mut no_period = saved.clone();
    for entry in &mut no_period.income {
        match &mut entry.income {
            FederalIncome::Partnership { data } => data.reporting_period = None,
            FederalIncome::Scorp { data } => edit_scorp(data).reporting_period = None,
            _ => {}
        }
    }
    assert_eq!(supported(&no_period), result);
    let records = native_draft_records(&saved);
    let draft = draft_from_native_records(saved.tax_year, &records);
    assert_eq!(native_draft_records(&draft), records);
    assert_eq!(draft, saved);
    if let Some(directory) = &directory {
        write_native_draft_records(directory.join("saved-draft.avro"), &records);
    }
    let mut session = restart(draft);
    let reopened = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &reopened else { unreachable!() };
    assert_eq!(
        documents.common.k1.documents.iter().map(|row| entered(&row.document)).collect::<Vec<_>>(),
        exact_periods
    );
    write("reopened.avro", &reopened);
    let payload = native.input("REOPEN_PAYLOAD").unwrap_or(reopened);
    accepted(&crate::tests::support::client::income_pages::save_source_fixture_request(
        &mut session,
        payload,
    ));
    assert_eq!(session.draft, saved);
    let mut correction = open_documents(&mut session);
    let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
    set_period(&mut documents.common.k1.documents[0].document, Some(period(20260401, 20261231)));
    set_period(&mut documents.common.k1.documents[1].document, Some(period(20260101, 20261231)));
    documents.common.k1.documents.reverse();
    accepted(&send(&mut session, correction));
    for entry in session.draft.income.iter().filter(|entry| {
        matches!(entry.income, FederalIncome::Partnership { .. } | FederalIncome::Scorp { .. })
    }) {
        assert_eq!(
            stored(entry),
            if entry.id == sources[0].id {
                Some(period(20260401, 20261231))
            } else {
                Some(period(20260101, 20261231))
            }
        );
    }
    let corrected = supported(&session.draft);
    assert_eq!(
        (corrected.tax, corrected.refund, corrected.balance),
        (result.tax, result.refund, result.balance)
    );
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}
