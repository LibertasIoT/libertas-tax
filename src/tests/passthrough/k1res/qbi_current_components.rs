use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{documents, ordinary_multi_current_qbi_parent};
use crate::tests::support::client::k1res::ordinary_multi_current_qbi_session;
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::*;

#[test]
fn current_component_statement_corrects_qbi_without_replacing_taxable_operating_income() {
    let mut session = ordinary_multi_current_qbi_session();
    let id = session.draft.income[0].id;
    add(&mut session.draft, wage(0, Owner::Taxpayer, 6_000_000, 0));
    session = restart(session.draft);
    let result = supported(&session.draft);
    // Actual operations are $4,000 + $2,000. Only the first statement's
    // independently documented $3,000 portion is QBI: $3,000 x 20% = $600.
    assert_eq!(line_amount(&result, &alloc::format!("partnership_{id}_schedule_e")), 600_000);
    assert_eq!(value(&result, "agi"), 66_000);
    assert_eq!(value(&result, "qbi_deduction"), 600);
    let saved = session.draft.clone();
    let dirty = session.dirty;
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);

    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    let crate::K1Document::K1PartnershipActivities(page) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    let K1ComponentStatementPage::Entered { source } = &mut page.activities.statement else {
        unreachable!()
    };
    let K1ReportedQbiStatement::Partnership { statement: Some(statement) } =
        &mut source.qbi_blocks[0].statement
    else {
        unreachable!()
    };
    statement.income = 200_000;
    let K1ComponentQualified::Partnership { current_income, .. } =
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
    accepted(&send_fixture(&mut session, request));
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "agi"), 66_000);
    assert_eq!(value(&corrected, "qbi_deduction"), 400);
    let accepted_source = expected_input(&session.draft, &session.draft.income[0]);
    let mut cold = restart(session.draft.clone());
    let mut reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    assert_eq!(documents(&mut reopened)[0].document, accepted_source);
    accepted(&send_fixture(&mut cold, reopened));
    assert_eq!(supported(&cold.draft), corrected);
}

#[test]
fn current_component_statement_needs_actual_gross_operating_source_controls() {
    let mut value = ordinary_multi_current_qbi_parent();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source.components[0].qualified_current_controls.ordinary = None;
    assert_eq!(value.rule(), None);
    assert!(matches!(value.ordinary_source(false), Err(error)
        if error == crate::k1_activity::ordinary::UNAVAILABLE));
}

#[test]
fn current_component_source_controls_must_foot_native_operating_amounts() {
    let mut value = ordinary_multi_current_qbi_parent();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    // This still bounds $3,000 QBI, but does not establish gross $4,000 income
    // and zero loss. Net-positive operating income cannot supply that fact.
    source.components[0].qualified_current_controls.ordinary =
        Some(K1CurrentSourceParts { income: 500_000, loss: 100_000 });
    assert_eq!(value.rule(), None);
    assert!(matches!(value.ordinary_source(false), Err(error)
        if error == crate::k1_activity::ordinary::UNAVAILABLE));
}
