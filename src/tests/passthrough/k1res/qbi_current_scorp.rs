use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{documents, ordinary_multi_current_scorp_qbi_parent};
use crate::tests::support::client::k1res::ordinary_multi_current_scorp_qbi_session;
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::*;

#[test]
fn current_scorp_component_statement_preserves_taxable_income_and_native_lifecycle() {
    let mut session = ordinary_multi_current_scorp_qbi_session();
    let index = session
        .draft
        .income
        .iter()
        .position(|entry| matches!(entry.income, FederalIncome::ScorpActivities { .. }))
        .unwrap();
    let id = session.draft.income[index].id;
    session = restart(session.draft);
    let result = supported(&session.draft);
    // Two actual businesses earn $4,000 + $2,000; only A reports $3,000 QBI.
    assert_eq!(line_amount(&result, &alloc::format!("scorp_{id}_schedule_e")), 600_000);
    assert_eq!(value(&result, "agi"), 66_000);
    assert_eq!(value(&result, "qbi_deduction"), 600);
    assert!(noted(&result, "WARN_K1_QBI_NO_STATEMENT"));
    let saved = session.draft.clone();
    let dirty = session.dirty;
    let native_ids: Vec<_> =
        crate::k1_activity::source(&saved.income[index].income).unwrap().ids().collect();
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let row = documents(&mut request)
        .iter()
        .position(|row| matches!(row.document, crate::K1Document::K1ScorpActivities(_)))
        .unwrap();

    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    let crate::K1Document::K1ScorpActivities(page) = &mut documents(&mut request)[row].document
    else {
        unreachable!()
    };
    let K1ComponentStatementPage::Entered { source } = &mut page.activities.statement else {
        unreachable!()
    };
    let K1ReportedQbiStatement::Scorp { statement: Some(statement) } =
        &mut source.qbi_blocks[0].statement
    else {
        unreachable!()
    };
    statement.income = 200_000;
    let K1ComponentQualified::Scorp { current_income, .. } =
        &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
    else {
        unreachable!()
    };
    *current_income = 200_000;
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            crate::tests::support::client::estimate_documents_from_save(&request)
        ),
        Reply::Response(P::Review { .. })
    ));
    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    accepted(&send_fixture(&mut session, request));
    assert_eq!(session.draft.revision, saved.revision + 2);
    assert_eq!(session.draft.next_id, saved.next_id);
    assert_eq!(
        crate::k1_activity::source(&session.draft.income[index].income)
            .unwrap()
            .ids()
            .collect::<Vec<_>>(),
        native_ids
    );
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "agi"), 66_000);
    assert_eq!(value(&corrected, "qbi_deduction"), 400);
    let accepted_source = expected_input(&session.draft, &session.draft.income[index]);
    let mut cold = restart(session.draft.clone());
    let mut reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    let reopened_source = documents(&mut reopened)
        .iter()
        .find(|row| matches!(row.document, crate::K1Document::K1ScorpActivities(_)))
        .unwrap();
    assert_eq!(reopened_source.document, accepted_source);
    accepted(&send_fixture(&mut cold, reopened));
    assert_eq!(supported(&cold.draft), corrected);
    assert_eq!(
        crate::k1_activity::source(&cold.draft.income[index].income)
            .unwrap()
            .ids()
            .collect::<Vec<_>>(),
        native_ids
    );
}

#[test]
fn current_scorp_component_requires_explicit_gross_operating_controls() {
    let mut value = ordinary_multi_current_scorp_qbi_parent();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source.components[0].qualified_current_controls.ordinary = None;
    assert_eq!(value.activities.source().unwrap().source_rule(false), None);
    assert!(matches!(value.ordinary_source(false), Err(error) if error == ordinary::UNAVAILABLE));
}

#[test]
fn current_scorp_component_net_income_does_not_establish_gross_source_parts() {
    let mut value = ordinary_multi_current_scorp_qbi_parent();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source.components[0].qualified_current_controls.ordinary =
        Some(K1CurrentSourceParts { income: 500_000, loss: 100_000 });
    assert_eq!(value.activities.source().unwrap().source_rule(false), None);
    assert!(matches!(value.ordinary_source(false), Err(error) if error == ordinary::UNAVAILABLE));
}

#[test]
fn current_scorp_component_requires_complete_bound_nonpassive_ordinary_sources() {
    for boundary in 0..9 {
        let mut value = ordinary_multi_current_scorp_qbi_parent();
        let K1ComponentStatement::Entered { source } = &mut value.activities else {
            unreachable!()
        };
        match boundary {
            0 => source.qbi_blocks[0].allocations = None,
            1 => source.qbi_blocks[0].reported_aggregate = true,
            2 => source.qbi_blocks[0].business_names = None,
            3 => source.passive_groups[0].participation = K1ActivityParticipation::Unavailable,
            4 => source.components[0].regular.as_mut().unwrap().operating_carry.basis = Some(100),
            5 => {
                let K1ComponentQualified::Scorp { ordinary, .. } =
                    &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
                else {
                    unreachable!()
                };
                ordinary.history = vec![loss_year(2025, 1, 0, 1, 0)];
            }
            6 => source.qbi_blocks[0].allocations.as_mut().unwrap().clear(),
            7 => {
                source.qbi_blocks[1].allocations.as_mut().unwrap()[0].component =
                    source.components[0].id
            }
            8 => {
                let K1ReportedQbiStatement::Scorp { statement: Some(statement) } =
                    &mut source.qbi_blocks[0].statement
                else {
                    unreachable!()
                };
                statement.reit_dividends = 100;
            }
            _ => unreachable!(),
        }
        assert!(
            matches!(value.ordinary_source(false), Err(error) if error == ordinary::UNAVAILABLE),
            "boundary {boundary}"
        );
    }
}

#[test]
fn current_scorp_statement_uses_shared_current_sstb_and_wage_limits() {
    let mut session = ordinary_multi_current_scorp_qbi_session();
    let wage_entry = session
        .draft
        .income
        .iter_mut()
        .find(|entry| matches!(entry.income, FederalIncome::Wage { .. }))
        .unwrap();
    *wage_entry = wage(wage_entry.id, wage_entry.owner.owned(), 60_000_000, 650_000);
    let index = session
        .draft
        .income
        .iter()
        .position(|entry| matches!(entry.income, FederalIncome::ScorpActivities { .. }))
        .unwrap();
    let FederalIncome::ScorpActivities { data } = &mut session.draft.income[index].income else {
        unreachable!()
    };
    let K1ComponentStatement::Entered { source } = &mut data.activities else { unreachable!() };
    let K1ReportedQbiStatement::Scorp { statement: Some(statement) } =
        &mut source.qbi_blocks[0].statement
    else {
        unreachable!()
    };
    statement.wages = 100_000;
    // Above the full phaseout, non-SSTB QBI is limited to 50% of the
    // statement's $1,000 wages. Shareholder employee wages are not substituted.
    assert_eq!(value(&supported(&session.draft), "qbi_deduction"), 500);
    let FederalIncome::ScorpActivities { data } = &mut session.draft.income[index].income else {
        unreachable!()
    };
    let K1ComponentStatement::Entered { source } = &mut data.activities else { unreachable!() };
    let K1ReportedQbiStatement::Scorp { statement: Some(statement) } =
        &mut source.qbi_blocks[0].statement
    else {
        unreachable!()
    };
    statement.service = true;
    assert_eq!(value(&supported(&session.draft), "qbi_deduction"), 0);
}
