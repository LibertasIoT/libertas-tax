use crate::k1_activity::*;
use crate::tests::cross::interview::{accepted, open_topic};
use crate::tests::support::builders::k1res::{
    documents, input, ordinary_multi_retained_qbi_parent,
};
use crate::tests::support::client::income_pages::catalog_preview_document;
use crate::tests::support::client::k1res::ordinary_multi_retained_qbi_session;
use crate::tests::*;

#[test]
fn retained_component_qbi_reaches_native_forms_and_corrects_without_changing_tax_income() {
    let mut session = ordinary_multi_retained_qbi_session();
    let id = session.draft.income[0].id;
    let mut profit = ordinary_partner_source(10_000, true);
    profit.qbi = Some(partner_qbi_statement(10_000));
    add(&mut session.draft, income(FederalIncome::Partnership { data: profit }));
    add(&mut session.draft, wage(0, Owner::Taxpayer, 6_000_000, 0));
    session = restart(session.draft);
    let result = supported(&session.draft);
    // Component A supplies $4,000; B releases its actual $1,000 basis and
    // $1,000 risk carries. Their fixed qualified portions are $500 + $800.
    assert_eq!(line_amount(&result, &alloc::format!("partnership_{id}_schedule_e")), 200_000);
    assert_eq!(value(&result, "agi"), 72_000);
    assert_eq!(value(&result, "qbi_deduction"), 1_740);
    assert!(result.lines.iter().any(|line| {
        line.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8995 && f.line == "3")
            && line.amount == -130_000
    }));
    let saved = session.draft.clone();
    let dirty = session.dirty;
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let mut preview = catalog_preview_document(documents(&mut request)[0].document.clone());
    let P::PreviewDocument { id: preview_id, .. } = &mut preview else { unreachable!() };
    *preview_id = id;
    assert!(matches!(
        send(&mut session, preview),
        Reply::Response(P::IncomePreview { preview: Some(200_000), .. })
    ));
    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    let IncomeDocument::K1PartnershipActivities(page) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    let K1ComponentStatementPage::Entered { source } = &mut page.activities.statement else {
        unreachable!()
    };
    let K1ComponentQualified::Partnership { records, .. } =
        &mut source.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified
    else {
        unreachable!()
    };
    let risk = &mut records.history.as_mut().unwrap().risk[0];
    risk.qualified = 60_000;
    risk.remaining_qualified = 60_000;
    let running =
        send(&mut session, crate::tests::support::client::estimate_documents_from_save(&request));
    assert!(matches!(running, Reply::Response(P::IncomeRunningEstimate { .. })));
    assert_eq!(session.draft, saved);
    accepted(&send(&mut session, request));
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "agi"), 72_000);
    assert_eq!(value(&corrected, "qbi_deduction"), 1_780);
    assert!(corrected.lines.iter().any(|line| {
        line.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8995 && f.line == "3")
            && line.amount == -110_000
    }));
    let accepted_source = input(&session.draft.income[0].income);
    let mut cold = restart(session.draft.clone());
    let mut reopened = open_topic(&mut cold, TaxSection::PassThrough);
    assert_eq!(documents(&mut reopened)[0].document, accepted_source);
    accepted(&send(&mut cold, reopened));
    assert_eq!(supported(&cold.draft), corrected);
}

#[test]
fn retained_component_qualification_needs_explicit_current_source_controls() {
    let mut value = ordinary_multi_retained_qbi_parent();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source.components[1].qualified_current_controls.ordinary = None;
    assert_eq!(value.rule(), None);
    assert!(matches!(value.ordinary_source(false), Err(error)
        if error == crate::k1_activity::ordinary::UNAVAILABLE));
}

#[test]
fn retained_component_histories_foot_the_exact_native_stage() {
    let mut value = ordinary_multi_retained_qbi_parent();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    let K1ComponentQualified::Partnership { records, .. } =
        &mut source.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified
    else {
        unreachable!()
    };
    let history = records.history.as_mut().unwrap();
    // 2026-10-19 MoneyPrecision: retain all three exact vintages totaling
    // 100,000 cents; independent annual projections do not set stage capacity.
    history.basis = [(2023, 33_350, 10_000), (2024, 33_350, 20_000), (2025, 33_300, 20_000)]
        .into_iter()
        .map(|(year, original, qualified)| ScorpLossYear {
            year,
            original,
            qualified,
            remaining: original,
            remaining_qualified: qualified,
        })
        .collect();
    assert_eq!(value.rule(), None);
    let output = value.ordinary_source(false).unwrap();
    assert_eq!(output.qualified[0].carries[0].amount.get(), 100_000);
    assert_eq!(output.qualified[0].carries[0].qbi.get(), 50_000);
}
