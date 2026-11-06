use super::*;
use crate::tests::cross::interview::linked_mortgage_editor;
use crate::tests::support::builders::native_income::{
    mortgage_1098_source, mortgage_source_return,
};

#[test]
fn mortgage_conversion_issues_name_the_existing_answer_fields() {
    let mut source = mortgage_1098_source();
    source.paid_share = MortgagePaidShare::Amount(SourceCents::Unavailable);
    let issue = source.conversion_issue().unwrap();
    assert_eq!(issue.key, "MORTGAGE_PAID_INTEREST_NEEDED");
    assert_eq!(issue.field.unwrap().square(), "[paid_share]");
    source.paid_share =
        MortgagePaidShare::Amount(SourceCents::Reported { amount: source.form.box_1 + 1 });
    let issue = source.conversion_issue().unwrap();
    assert_eq!(issue.key, "MORTGAGE_PAID_INTEREST_ABOVE_BOX1");
    assert_eq!(issue.field.unwrap().square(), "[paid_share]");
    source.paid_share = MortgagePaidShare::All;
    source.debt_date = MortgageDebtDate::Draw { date: SourceDate::Unavailable };
    let issue = source.conversion_issue().unwrap();
    assert_eq!(issue.key, "MORTGAGE_DRAW_DATE_NEEDED");
    assert_eq!(issue.field.unwrap().square(), "[debt_date/date]");
    source.debt_date = MortgageDebtDate::Origination;
    source.points = MortgagePointTreatment::None;
    let issue = source.conversion_issue().unwrap();
    assert_eq!(issue.key, "MORTGAGE_POINTS_TREATMENT_NEEDED");
    assert_eq!(issue.field.unwrap().square(), "[points]");
    source.form.box_6 = 0;
    assert_eq!(source.conversion_issue(), None);
}

#[test]
fn unresolved_mortgage_sources_reopen_at_their_correction_page_and_clear_after_save() {
    for cause in 0..3 {
        let mut draft = mortgage_source_return();
        let source = &mut draft.deductions.as_mut().unwrap().mortgage[0];
        match cause {
            0 => source.paid_share = MortgagePaidShare::Amount(SourceCents::Unavailable),
            1 => source.debt_date = MortgageDebtDate::Draw { date: SourceDate::Unavailable },
            2 => source.points = MortgagePointTreatment::None,
            _ => unreachable!(),
        }
        let expected_issue = source.conversion_issue().unwrap();
        let paper = source.form.clone();
        let id = source.id;
        assert!(draft.valid_stored());
        let mut session = restart(draft);
        let result = estimate(&session.draft);
        assert_eq!(result.state, ResultState::Unsupported);
        assert!(result.tax.is_none());
        assert!(result.issues.iter().any(|issue| issue.display.key == expected_issue.key));
        let Reply::Response(P::Review { remaining_items, .. }) =
            send(&mut session, P::ReviewAccepted { user: TEST_USER })
        else {
            panic!("an unresolved mortgage remains reviewable")
        };
        let pending: Vec<_> =
            remaining_items.iter().filter(|row| row.section == TaxSection::Mortgage).collect();
        assert_eq!(pending.len(), 1);
        assert!(status_text(&pending[0].item).contains("Form 1098"));
        assert!(!status_text(&pending[0].item).contains("()"));
        let mut request = linked_mortgage_editor(&mut session);
        let P::SaveMortgages { value, .. } = &mut request else { unreachable!() };
        assert_eq!(value.loans[0].id, id);
        assert_eq!(value.loans[0].value, paper);
        match cause {
            0 => value.loans[0].paid_share = MortgagePaidShare::All,
            1 => {
                value.loans[0].debt_date =
                    MortgageDebtDate::Draw { date: SourceDate::Reported { date: 20260101 } };
            }
            2 => value.loans[0].points = MortgagePointTreatment::Purchase,
            _ => unreachable!(),
        }
        accepted(&send(&mut session, request));
        let saved = session.draft.clone();
        let mut cold = restart(saved.clone());
        assert_eq!(cold.draft, saved);
        assert_eq!(cold.draft.deductions.as_ref().unwrap().mortgage[0].form, paper);
        supported(&cold.draft);
        let Reply::Response(P::Review { remaining_items, .. }) =
            send(&mut cold, P::ReviewAccepted { user: TEST_USER })
        else {
            panic!("the corrected mortgage remains reviewable")
        };
        assert!(!remaining_items.iter().any(|row| row.section == TaxSection::Mortgage));
    }
}

#[test]
fn corrected_paper_keeps_the_paid_share_and_exposes_a_new_mortgage_conflict() {
    let mut draft = mortgage_source_return();
    draft.deductions.as_mut().unwrap().mortgage[0].paid_share =
        MortgagePaidShare::Amount(SourceCents::Reported { amount: 100_000 });
    let mut session = restart(draft);
    let mut paper_request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut paper_request else { unreachable!() };
    documents.common.mortgages.documents[0].form.box_1 = 99_999;
    accepted(&send(&mut session, paper_request));
    let source = &session.draft.deductions.as_ref().unwrap().mortgage[0];
    assert_eq!(source.form.box_1, 99_999);
    assert_eq!(
        source.paid_share,
        MortgagePaidShare::Amount(SourceCents::Reported { amount: 100_000 })
    );
    let mut cold = restart(session.draft.clone());
    let Reply::Response(P::Review { remaining_items, .. }) =
        send(&mut cold, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("a paper correction keeps the source reviewable")
    };
    assert_eq!(remaining_items.iter().filter(|row| row.section == TaxSection::Mortgage).count(), 1,);
    let request = linked_mortgage_editor(&mut cold);
    crate::tests::support::assert::k1res::assert_located_rejection(
        &mut cold,
        request,
        "MORTGAGE_PAID_INTEREST_ABOVE_BOX1",
        "[value/loans/0/paid_share]",
    );
    let mut request = linked_mortgage_editor(&mut cold);
    let P::SaveMortgages { value, .. } = &mut request else { unreachable!() };
    value.loans[0].paid_share = MortgagePaidShare::Amount(SourceCents::Reported { amount: 50_000 });
    accepted(&send(&mut cold, request));
    assert_eq!(
        MortgageLoan::try_from(&cold.draft.deductions.as_ref().unwrap().mortgage[0])
            .unwrap()
            .interest,
        50_000,
    );
    supported(&cold.draft);
}

#[test]
fn an_unresolved_source_keeps_distinct_mortgage_and_refund_questions_visible() {
    let mut draft = mortgage_source_return();
    let deductions = draft.deductions.as_mut().unwrap();
    deductions.mortgage[0].paid_share = MortgagePaidShare::Amount(SourceCents::Unavailable);
    let mut other = deductions.mortgage[0].clone();
    other.id = draft.next_id;
    draft.next_id += 1;
    deductions.mortgage.push(other.clone());
    other.id = draft.next_id;
    draft.next_id += 1;
    other.paid_share = MortgagePaidShare::All;
    other.form.box_4 = 1_000;
    deductions.mortgage.push(other);
    let mut session = restart(draft);
    let Reply::Response(P::Review { remaining_items, .. }) =
        send(&mut session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("all accepted mortgage sources remain reviewable")
    };
    assert_eq!(remaining_items.iter().filter(|row| row.section == TaxSection::Mortgage).count(), 3,);
}
