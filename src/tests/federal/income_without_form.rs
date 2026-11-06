use super::*;
use crate::tests::cross::interview::from_review;

// Both questions are facts the rules test (Schedule C and Form 4137 routes),
// so neither has a Default (app.default.decided): the walk opens the unsaved
// page at once by WorkflowRequest with nothing answered and no refusal, and
// only the person's own answers are saved and reopened by EditRequest.
#[test]
fn documents_lead_to_unanswered_questions_then_beyond_documents() {
    let source = include_str!("../../core/income_without_form_model.rs");
    let fields = &source[source.find("pub struct IncomeWithoutFormAnswers").unwrap()..];
    let fields = &fields[..fields.find("\n}\n").unwrap()];
    assert!(fields.contains("pub self_work: bool,") && fields.contains("pub tips: bool,"));
    assert!(fields.contains("pub other: Option<WithoutFormOther>,"));
    assert!(!fields.contains("libertas_default"), "{fields}");
    let mut draft = complete(FederalStatus::Single);
    draft.without_form = None;
    draft.finished_sections.retain(|section| *section != TaxSection::IncomeWithoutForm);
    let mut session = Interview::new(TEST_USER, draft);
    let Reply::Edit(mut documents @ P::SaveDocuments { .. }) =
        from_review(&mut session, TaxSection::Documents)
    else {
        panic!("expected documents")
    };
    let P::SaveDocuments { review, .. } = &mut documents else { unreachable!() };
    *review = false;
    let begin @ Reply::Response(P::BeginIncomeWithoutForm { page, previous, .. }) =
        send(&mut session, documents)
    else {
        panic!("income without a form follows documents, unanswered")
    };
    let (documents, without_form) = (TaxSection::Documents as i32, TaxSection::IncomeWithoutForm);
    assert_eq!((page, previous), (without_form as i32, documents));
    assert!(session.draft.without_form.is_none(), "opening does not accept answers");
    // Navigating to the unsaved page opens it the same way.
    assert!(matches!(
        from_review(&mut session, TaxSection::IncomeWithoutForm),
        Reply::Response(P::BeginIncomeWithoutForm { .. })
    ));
    // The person answers No twice; only then is anything saved.
    let page = crate::tests::cross::interview::without_form_editor(begin);
    let P::SaveIncomeWithoutForm { value, .. } = &page else { unreachable!() };
    let no = IncomeWithoutFormAnswers { self_work: false, tips: false, other: None };
    assert_eq!(*value, no);
    assert!(matches!(send(&mut session, page), Reply::Edit(P::SaveChecklist { .. })));
    assert_eq!(session.draft.without_form, Some(no.clone()));
    let planned = crate::interview::planned_sections(&session.draft);
    assert!(!planned.contains(&TaxSection::SelfEmployment));
    assert!(!planned.contains(&TaxSection::Wages));
    // The saved answers reopen by EditRequest.
    let Reply::Edit(P::SaveIncomeWithoutForm { value, .. }) =
        from_review(&mut session, TaxSection::IncomeWithoutForm)
    else {
        panic!("the saved page reopens with its answers")
    };
    assert_eq!(value, no);
}

#[test]
fn three_yes_answers_preview_save_restart_and_reopen_without_duplicate_income() {
    use crate::tests::support::builders::native_income::{cash_tips_without_w2, owned};
    use crate::tests::support::client::{
        cohorts::save_existing_business, income_pages::save_topic,
    };

    let mut session = Interview::new(TEST_USER, empty_return(FederalStatus::Single));
    let answers = IncomeWithoutFormAnswers {
        self_work: true,
        tips: true,
        other: Some(WithoutFormOther { amount: 12_345, description: "Found property".into() }),
    };
    let before = session.draft.clone();
    let dirty = session.dirty;
    accepted(&send(
        &mut session,
        P::EstimateIncomeWithoutForm { user: TEST_USER, value: answers.clone() },
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    let mut page = crate::tests::cross::interview::without_form_editor(from_review(
        &mut session,
        TaxSection::IncomeWithoutForm,
    ));
    let P::SaveIncomeWithoutForm { value: answer, .. } = &mut page else { unreachable!() };
    *answer = answers.clone();
    accepted(&send(&mut session, page));
    assert!(session.draft.income.is_empty(), "routes create no invented source records");
    let planned = crate::interview::planned_sections(&session.draft);
    assert!(planned.contains(&TaxSection::SelfEmployment));
    assert!(planned.contains(&TaxSection::Wages));
    assert!(!planned.contains(&TaxSection::OtherIncome), "the amount is already answered");
    let mut business = crate::tests::cross::interview::business(100_000);
    business.name = "Synthetic repair business".into();
    accepted(&save_existing_business(&mut session, 0, business));
    let tips = cash_tips_without_w2(
        vec![CashTipMonth { month: 3, received: 100_000, reported: 0 }],
        TipCoverage::SocialAndMedicare,
        false,
    );
    let request =
        save_topic(&mut session, TaxSection::Wages, vec![owned(Owner::Taxpayer, 0, tips)]);
    accepted(&send(&mut session, request));
    let result = estimate(&session.draft);
    // $1,000 receipts with no expenses; $1,000 unreported March tips.
    // Form 4137: $1,000 × 6.2% = $62, and × 1.45% = $14.50, rounded to $15.
    assert_eq!(schedule_1_line(&result, "3"), 1_000);
    assert_eq!(return_line(&result, "1c"), 100_000);
    assert_eq!(value(&result, "employee_tips_0_social_tax"), 62);
    assert_eq!(value(&result, "employee_tips_0_medicare_tax"), 15);
    assert_eq!(schedule_1_line(&result, "8z"), 123);
    assert_eq!(return_line(&result, "8"), 112_300);
    assert_eq!(
        result.lines.iter().filter(|line| line.id == "income_without_form_other").count(),
        1
    );
    let mut cold = restart(session.draft.clone());
    assert_eq!(cold.draft.without_form, Some(answers.clone()));
    assert_eq!(cold.draft.income, session.draft.income);
    assert_eq!(estimate(&cold.draft), result);
    let Reply::Edit(P::SaveIncomeWithoutForm { value, .. }) =
        from_review(&mut cold, TaxSection::IncomeWithoutForm)
    else {
        panic!("expected reopened income without a form")
    };
    assert_eq!(value, answers);
    for topic in [TaxSection::SelfEmployment, TaxSection::Wages] {
        let reply = from_review(&mut cold, topic);
        assert!(
            matches!(
                (topic, &reply),
                (TaxSection::Wages, Reply::Edit(P::SaveWagesAnswers { .. }))
                    | (
                        TaxSection::SelfEmployment,
                        Reply::Edit(P::SaveIncomes { topic: TaxSection::SelfEmployment, .. })
                    )
            ),
            "{topic:?}: {reply:?}"
        );
    }
}

#[test]
fn generic_other_income_joins_existing_8z_before_annual_rounding() {
    let mut draft = empty_return(FederalStatus::Single);
    draft.without_form = Some(IncomeWithoutFormAnswers {
        other: Some(WithoutFormOther { amount: 10_049, description: "Found property".into() }),
        ..Default::default()
    });
    add(
        &mut draft,
        income(FederalIncome::OtherIncome {
            data: OtherIncomeItem {
                kind: OtherIncomeKind::Other { description: "Taxable settlement".into() },
                amount: 10_049,
                withholding: 0,
            },
        }),
    );
    // $100.49 + $100.49 = $200.98, rounded once to $201.
    let result = estimate(&draft);
    assert_eq!(schedule_1_line(&result, "8z"), 201);
    assert_eq!(return_line(&result, "8"), 20_100);
    assert_eq!(return_line(&result, "9"), 20_100);
}

#[test]
fn unchanged_positive_answers_keep_finished_topics_and_correction_withdraws_routes() {
    let mut draft = empty_return(FederalStatus::Single);
    draft.without_form =
        Some(IncomeWithoutFormAnswers { self_work: true, tips: true, other: None });
    draft.finished_sections.extend([TaxSection::SelfEmployment, TaxSection::Wages]);
    let mut session = Interview::new(TEST_USER, draft);
    let revision = session.draft.revision;
    let Reply::Edit(page @ P::SaveIncomeWithoutForm { .. }) =
        from_review(&mut session, TaxSection::IncomeWithoutForm)
    else {
        panic!("expected income without a form")
    };
    accepted(&send(&mut session, page));
    assert_eq!(session.draft.revision, revision);
    for topic in [TaxSection::SelfEmployment, TaxSection::Wages] {
        assert!(session.draft.finished_sections.contains(&topic), "{topic:?}");
    }
    let Reply::Edit(mut page @ P::SaveIncomeWithoutForm { .. }) =
        from_review(&mut session, TaxSection::IncomeWithoutForm)
    else {
        panic!("expected correction editor")
    };
    let P::SaveIncomeWithoutForm { value, .. } = &mut page else { unreachable!() };
    *value = IncomeWithoutFormAnswers::default();
    accepted(&send(&mut session, page));
    let planned = crate::interview::planned_sections(&session.draft);
    assert!(!planned.contains(&TaxSection::SelfEmployment));
    assert!(!planned.contains(&TaxSection::Wages));
}

#[test]
fn withdrawing_front_page_routes_preserves_actual_business_and_wage_records() {
    let mut draft = complete(FederalStatus::Single);
    draft.without_form =
        Some(IncomeWithoutFormAnswers { self_work: true, tips: true, other: None });
    add(&mut draft, income(cash_business(1_000)));
    let records = draft.income.clone();
    let mut session = Interview::new(TEST_USER, draft);
    let Reply::Edit(mut page @ P::SaveIncomeWithoutForm { .. }) =
        from_review(&mut session, TaxSection::IncomeWithoutForm)
    else {
        panic!("expected income without a form")
    };
    let P::SaveIncomeWithoutForm { value, .. } = &mut page else { unreachable!() };
    *value = IncomeWithoutFormAnswers::default();
    accepted(&send(&mut session, page));
    assert_eq!(session.draft.income, records);
    assert_eq!(schedule_1_line(&estimate(&session.draft), "3"), 1_000);
    assert_eq!(return_line(&estimate(&session.draft), "1a"), 6_000_000);
}
