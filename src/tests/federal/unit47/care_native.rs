use crate::tests::cross::interview::{accepted, from_review};
use crate::tests::support::client::{care_w2_sources, exchange};
use crate::tests::*;

#[test]
fn unit47_native_care_keeps_every_w2_answer_and_source_refusal() {
    let mut session = care_w2_sources(21);
    let Reply::Edit(mut request) = from_review(&mut session, TaxSection::Care) else {
        panic!("actual Care editor")
    };
    let P::SaveCare { value: page, .. } = &mut request else {
        unreachable!()
    };
    assert_eq!(page.benefits.len(), 21);
    let ids: Vec<_> = session.draft.income.iter().map(|entry| entry.id).collect();
    assert_eq!(
        page.benefits.iter().map(|row| row.wage).collect::<Vec<_>>(),
        ids
    );
    page.plan_maximum = CarePlanMaximum::Plans {
        maximum: 750_000,
    };
    page.providers = crate::tests::support::builders::care_providers(21);
    page.providers[20].paid = 100_000;
    for row in &mut page.benefits {
        row.in_box_1 = Some(100);
    }
    let correct = request.clone();
    let P::SaveCare { value: page, .. } = &mut request else {
        unreachable!()
    };
    page.benefits[20].wage = i64::MAX;
    let Reply::Response(P::Problem { message, field }) =
        exchange(&mut session, request, "last actual W-2 identity")
    else {
        panic!("located source refusal")
    };
    assert_eq!(
        message,
        libertas::libertas_formatted_text("CARE_WAGE_SOURCE_INVALID", &[])
    );
    assert_eq!(field.as_deref(), Some("[value/benefits/20/wage]"));
    accepted(&send(&mut session, correct));
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    assert_eq!(saved.credits.as_ref().unwrap().care.benefits.len(), 21);
    let providers = &saved.credits.as_ref().unwrap().care.providers;
    assert_eq!(providers.len(), 21);
    assert!(providers.iter().all(|provider| provider.id > 0));
    assert_eq!(return_line(&estimate(&saved), "1a"), 441_000);
    assert_eq!(value(&estimate(&saved), "agi"), 4_410);
    let mut cold = restart(saved.clone());
    let Reply::Edit(reopened) = from_review(&mut cold, TaxSection::Care) else {
        panic!("Care reopen")
    };
    let P::SaveCare { value: page, .. } = &reopened else {
        unreachable!()
    };
    assert_eq!(
        page.benefits.iter().map(|row| row.wage).collect::<Vec<_>>(),
        ids
    );
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
    assert_eq!(
        estimate(&cold.draft).return_lines,
        estimate(&saved).return_lines
    );
}

#[test]
fn unit47_native_care_preserves_each_own_business_plan() {
    let mut session = Interview::new(
        TEST_USER,
        crate::tests::support::builders::care_own_plans(3),
    );
    let Reply::Edit(request) = from_review(&mut session, TaxSection::Care) else {
        panic!("actual Care editor")
    };
    let mut request = request;
    let P::SaveCare { value: page, .. } = &mut request else {
        unreachable!()
    };
    assert_eq!(page.own_business.len(), 3);
    page.own_business[0].received = 100_001;
    accepted(&send(&mut session, request));
    assert!(session.draft.valid_stored());
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
    assert_eq!(value(&result, "care_form2441_line12"), 1_002);
    assert_eq!(value(&result, "care_form2441_line22"), 1_002);
    assert_eq!(value(&result, "care_form2441_line24"), 0);
    let saved = session.draft.clone();
    let mut cold = restart(saved.clone());
    let Reply::Edit(reopened) = from_review(&mut cold, TaxSection::Care) else {
        panic!("Care reopen")
    };
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
}
