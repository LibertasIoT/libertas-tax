use crate::tests::cross::interview::{accepted, open_topic, save_topic};
use crate::tests::support::builders::public_safety_pensions as pensions;
use crate::tests::support::client::estimate_documents_from_save;
use crate::tests::*;

fn attempt(session: &mut Interview, request: P, case: &str) -> P {
    let reply = crate::tests::support::client::exchange(session, request, case);
    let (Reply::Response(response) | Reply::Edit(response)) = reply;
    response
}

fn pension_at(request: &mut P, index: usize) -> &mut R1099 {
    let P::SaveIncomes { documents, .. } = request else {
        unreachable!()
    };
    let IncomeDocument::R1099(pension) = &mut documents.documents[index].document else {
        unreachable!()
    };
    pension
}

fn refused(session: &mut Interview, request: P, case: &str, key: &str, path: &str) {
    let problem = attempt(session, request, case);
    let P::Problem { message, field } = &problem else {
        panic!("{case}: expected source refusal")
    };
    assert_eq!(*message, libertas::libertas_formatted_text(key, &[]));
    assert_eq!(field.as_deref(), Some(path));
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
    let seed = save_topic(&session.draft, TaxSection::Retirement, pensions(50));
    accepted(&send(&mut session, seed));
    assert_eq!(
        session
            .draft
            .screening
            .as_ref()
            .unwrap()
            .public_safety
            .len(),
        100
    );
    let mut request = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut request else {
        unreachable!()
    };
    assert_eq!(documents.documents.len(), 50);
    for source in &documents.documents {
        assert!(source.id > 0);
        let IncomeDocument::R1099(pension) = &source.document else {
            unreachable!()
        };
        assert_eq!(pension.public_safety.len(), 2);
    }
    documents.documents.extend(pensions(1));
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
    assert!(
        rows.iter()
            .all(|row| row.premiums == 100 && row.funded == 100)
    );
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
        session
            .draft
            .income
            .iter()
            .find(|entry| entry.id == retained_wage.id),
        Some(&retained_wage)
    );
    let saved = session.draft.clone();
    let reopened = open_topic(&mut session, TaxSection::Retirement);
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
    let remove_neutral_wage = save_topic(&session.draft, TaxSection::Wages, Vec::new());
    let response = attempt(
        &mut session,
        remove_neutral_wage,
        "public-safety-max-removes-neutral-fixture-wage",
    );
    assert!(!matches!(response, P::Problem { .. }), "{response:?}");
    assert!(session.draft.income.is_empty());
    let mut request = open_topic(&mut session, TaxSection::Retirement);
    let P::SaveIncomes {
        people, documents, ..
    } = &mut request
    else {
        unreachable!()
    };
    // Actual emitted people catalog supplies both insured identities. The
    // source pension covers the officer and spouse under distinct policies.
    assert!(people.iter().any(|person| person.id == -1));
    assert!(people.iter().any(|person| person.id == -2));
    let mut sources = pensions(100);
    for source in &mut sources {
        let IncomeDocument::R1099(pension) = &mut source.document else {
            unreachable!()
        };
        pension.public_safety.extend([
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
    documents.documents = sources;
    let response = attempt(
        &mut session,
        request,
        "100-pensions-four-distinct-coverage-rows",
    );
    assert!(!matches!(response, P::Problem { .. }), "{response:?}");
    assert_eq!(session.draft.income.len(), 100);
    let saved = session.draft.clone();
    let rows = &saved.screening.as_ref().unwrap().public_safety;
    assert_eq!(rows.len(), 400);
    assert!(
        rows.iter()
            .all(|row| row.premiums == 100 && row.funded == 100)
    );
    for entry in &saved.income {
        assert!(entry.id > 0);
        let actual: Vec<_> = rows
            .iter()
            .filter(|row| row.income_id == entry.id)
            .collect();
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
    let valid = open_topic(&mut session, TaxSection::Retirement);
    let mut bad = valid.clone();
    pension_at(&mut bad, 1)
        .public_safety
        .push(PublicSafetyPaid {
            coverage: PublicSafetyPaidCoverage::Other,
            premiums: 100,
            funded: 100,
        });
    // D-2026-10-15-13: another actual policy is accepted; coverage kind
    // does not impose a row count. Financial failures retain their priority.
    let mut financial = bad.clone();
    pension_at(&mut financial, 1).public_safety[1].funded = 101;
    refused(
        &mut session,
        financial,
        "public-safety-financial-priority-with-five-rows",
        "PSO_FUNDED_ABOVE_PREMIUMS",
        "[documents/documents/1/document/public_safety/1/funded]",
    );
    let mut financial = valid.clone();
    let pension = pension_at(&mut financial, 1);
    pension.gross = 399;
    pension.taxable = 399;
    refused(
        &mut session,
        financial,
        "public-safety-funded-total-stays-within-source-gross",
        "PSO_FUNDED_ABOVE_GROSS",
        "[documents/documents/1/document/public_safety]",
    );
    // The old gross relationship still refuses a nonrepresentable positive
    // funded sum before any stored scalar bound, without debug/release wrap.
    let mut overflow = valid.clone();
    for row in &mut pension_at(&mut overflow, 1).public_safety[..2] {
        row.premiums = i64::MAX;
        row.funded = i64::MAX;
    }
    refused(
        &mut session,
        overflow,
        "public-safety-funded-sum-overflow-is-above-gross",
        "PSO_FUNDED_ABOVE_GROSS",
        "[documents/documents/1/document/public_safety]",
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
    accepted(&send(&mut session, bad));
    let completed = session.draft.clone();
    assert_eq!(
        completed.screening.as_ref().unwrap().public_safety.len(),
        401
    );
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
    let mut cold = restart(completed.clone());
    let reopened = open_topic(&mut cold, TaxSection::Retirement);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, completed);
}
