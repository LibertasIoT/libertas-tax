use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::public_safety_pensions as pensions;
use crate::tests::support::client::estimate_documents_from_save;
use crate::tests::support::client::income_pages::{
    open_document_center, open_topic, page_documents, save_documents,
};
use crate::tests::*;

fn attempt(session: &mut Interview, request: P, case: &str) -> P {
    let reply = if matches!(request, P::SaveDocuments { .. }) {
        crate::tests::support::client::income_pages::save_source_fixture_request(session, request)
    } else {
        crate::tests::support::client::exchange(session, request, case)
    };
    let (Reply::Response(response) | Reply::Edit(response)) = reply;
    response
}

fn pension_at(request: &mut P, index: usize) -> &mut crate::Form1099RPaper {
    let P::SaveDocuments { documents, .. } = request else { unreachable!() };
    let crate::RetirementDocument::R1099(pension) =
        &mut documents.common.retirement.documents[index].document
    else {
        unreachable!()
    };
    pension
}

fn refused(session: &mut Interview, request: P, case: &str, key: &str, path: &str) {
    crate::tests::support::client::survey::survey_refusal(case, session, request, key, path);
}

#[test]
fn unit47_public_safety_stored_list_admits_each_pension_source_row() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let retained_wage = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
        .unwrap()
        .clone();
    accepted(&crate::tests::support::client::save_retirement_fixture(&mut session, pensions(50)));
    assert_eq!(session.draft.screening.as_ref().unwrap().public_safety.len(), 100);
    let request = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &request else { unreachable!() };
    assert_eq!(documents.common.retirement.documents.len(), 50);
    for source in &documents.common.retirement.documents {
        assert!(source.id > 0);
        let crate::RetirementDocument::R1099(_) = &source.document else { unreachable!() };
        assert_eq!(
            session
                .draft
                .screening
                .as_ref()
                .unwrap()
                .public_safety
                .iter()
                .filter(|row| row.income_id == source.id)
                .count(),
            2
        );
    }
    let mut sources: Vec<_> = page_documents(&request)
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::Retirement { .. }))
        .collect();
    sources.extend(pensions(1));
    let request = save_documents(&mut session, sources);
    let response = attempt(&mut session, request, "51-pensions-two-coverage-kinds");
    assert!(
        !matches!(response, P::Problem { .. }),
        "stored derived list must admit every independent pension's actual coverage rows: {response:?}"
    );
    // 2026-10-15 fixture correction: complete() supplies one neutral W-2.
    // Count the actual pension sources while keeping that unrelated source.
    assert_eq!(session.draft.income.len(), 52);
    assert_eq!(
        session
            .draft
            .income
            .iter()
            .filter(|entry| matches!(entry.income, FederalIncome::Retirement { .. }))
            .count(),
        51
    );
    let rows = &session.draft.screening.as_ref().unwrap().public_safety;
    assert_eq!(rows.len(), 102);
    assert!(rows.iter().all(|row| row.premiums == 100 && row.funded == 100));
    assert!(
        session
            .draft
            .income
            .iter()
            .filter(|entry| matches!(entry.income, FederalIncome::Retirement { .. }))
            .all(|entry| rows.iter().filter(|row| row.income_id == entry.id).count() == 2)
    );
    assert_eq!(
        session
            .draft
            .income
            .iter()
            .filter(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
            .count(),
        1
    );
    assert_eq!(
        session.draft.income.iter().find(|entry| entry.id == retained_wage.id),
        Some(&retained_wage)
    );
    let saved = session.draft.clone();
    let reopened = open_document_center(&mut session);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    assert_eq!(restart(saved.clone()).draft, saved);
}

#[test]
fn unit47_public_safety_unbounded_coverage_rows_keep_financial_priority() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    // 2026-10-15 fixture correction: construct the pension-only maximum by
    // removing the inherited neutral W-2 through its ordinary source editor.
    // These 100 independent pensions are a representative count, not a cap.
    let remove_neutral_wage = save_documents(&mut session, Vec::new());
    let response = attempt(
        &mut session,
        remove_neutral_wage,
        "public-safety-max-removes-neutral-fixture-wage",
    );
    assert!(!matches!(response, P::Problem { .. }), "{response:?}");
    drop(response);
    assert!(session.draft.income.is_empty());
    let request = open_document_center(&mut session);
    let P::SaveDocuments { people, .. } = &request else { unreachable!() };
    // Actual emitted people catalog supplies both insured identities. The
    // source pension covers the officer and spouse under distinct policies.
    assert!(people.iter().any(|person| person.id == -1));
    assert!(people.iter().any(|person| person.id == -2));
    drop(request);
    // Reuse the identical paper and answer fixture for 100 independent source
    // rows; the normal Save still assigns each row its own accepted identity.
    let pension = pensions(1).pop().unwrap();
    accepted(&crate::tests::support::client::save_retirement_fixture(
        &mut session,
        vec![pension; 100],
    ));
    let mut request = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes { followups, .. } = &mut request else { unreachable!() };
    for question in followups {
        let DocumentAnswer::RetirementPublicSafety {
            value: Some(PublicSafetyAnswer::Premiums { rows }),
        } = &mut question.answer
        else {
            continue;
        };
        rows.extend([
            PublicSafetyPaid {
                coverage: PublicSafetyPaidCoverage::LongTermCare { person: -1 },
                premiums: 100,
                funded: 100,
            },
            PublicSafetyPaid {
                coverage: PublicSafetyPaidCoverage::LongTermCare { person: -2 },
                premiums: 100,
                funded: 100,
            },
        ]);
    }
    let response = attempt(&mut session, request, "100-pensions-four-distinct-coverage-rows");
    assert!(!matches!(response, P::Problem { .. }), "{response:?}");
    drop(response);
    assert_eq!(session.draft.income.len(), 100);
    let saved = session.draft.clone();
    let rows = &saved.screening.as_ref().unwrap().public_safety;
    assert_eq!(rows.len(), 400);
    assert!(rows.iter().all(|row| row.premiums == 100 && row.funded == 100));
    for entry in &saved.income {
        assert!(entry.id > 0);
        let actual: Vec<_> = rows.iter().filter(|row| row.income_id == entry.id).collect();
        assert_eq!(actual.len(), 4);
        for coverage in [
            PublicSafetyCoverage::Medical,
            PublicSafetyCoverage::Other,
            PublicSafetyCoverage::LongTermCare { person: -1 },
            PublicSafetyCoverage::LongTermCare { person: -2 },
        ] {
            assert!(actual.iter().any(|row| row.coverage == coverage));
        }
    }
    let valid = open_document_center(&mut session);
    let mut bad = open_topic(&mut session, TaxSection::Retirement);
    let source_id = saved.income[1].id;
    let P::SaveIncomes { followups, .. } = &mut bad else { unreachable!() };
    let question_index = followups
        .iter()
        .position(|question| question.key == alloc::format!("income/{source_id}/public_safety"))
        .unwrap();
    let DocumentAnswer::RetirementPublicSafety {
        value: Some(PublicSafetyAnswer::Premiums { rows }),
    } = &mut followups[question_index].answer
    else {
        unreachable!()
    };
    rows.push(PublicSafetyPaid {
        coverage: PublicSafetyPaidCoverage::Other,
        premiums: 100,
        funded: 100,
    });
    // Another actual policy is accepted; coverage kind
    // does not impose a row count. Financial failures retain their priority.
    let mut financial = bad.clone();
    let P::SaveIncomes { followups, .. } = &mut financial else { unreachable!() };
    let DocumentAnswer::RetirementPublicSafety {
        value: Some(PublicSafetyAnswer::Premiums { rows }),
    } = &mut followups[question_index].answer
    else {
        unreachable!()
    };
    rows[1].funded = 101;
    let path: alloc::string::String = libertas::SchemaPath::root()
        .field("followups")
        .key(followups[question_index].key.as_str())
        .field("answer")
        .field("value")
        .field("rows")
        .index(1)
        .field("funded")
        .into();
    refused(
        &mut session,
        financial,
        "public-safety-financial-priority-with-five-rows",
        "PSO_FUNDED_ABOVE_PREMIUMS",
        &path,
    );
    let mut financial = valid.clone();
    let pension = pension_at(&mut financial, 1);
    pension.gross = 399;
    pension.taxable = Some(399);
    refused(
        &mut session,
        financial,
        "public-safety-funded-total-stays-within-source-gross",
        "PSO_FUNDED_ABOVE_GROSS",
        "[documents/common/retirement/documents/1/document/public_safety/rows]",
    );
    accepted(&send(&mut session, valid));
    assert_eq!(session.draft, saved);
    assert_eq!(restart(saved.clone()).draft, saved);

    let estimate = estimate_documents_from_save(&bad);
    let dirty = session.dirty;
    assert!(matches!(
        send(&mut session, estimate),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    drop(saved);
    let response = attempt(&mut session, bad, "additional-public-safety-policy");
    assert!(!matches!(response, P::Problem { .. }), "{response:?}");
    drop(response);
    let completed = session.draft.clone();
    assert_eq!(completed.screening.as_ref().unwrap().public_safety.len(), 401);
    let source = completed.income[1].id;
    assert_eq!(
        completed
            .screening
            .as_ref()
            .unwrap()
            .public_safety
            .iter()
            .filter(|row| row.income_id == source)
            .count(),
        5
    );
    drop(session);
    let mut cold = restart(completed.clone());
    let reopened = open_document_center(&mut cold);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, completed);
}
