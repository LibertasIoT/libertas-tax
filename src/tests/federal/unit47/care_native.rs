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
        maximum: 100_000_000_001,
    };
    page.providers = crate::tests::support::builders::care_providers(21);
    page.providers[20].paid = 100_000_000_001;
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
    let mut invalid_parts = correct.clone();
    let P::SaveCare { value: page, .. } = &mut invalid_parts else {
        unreachable!()
    };
    page.benefits[20].in_box_1 = Some(i64::MAX);
    page.benefits[20].forfeited = Some(i64::MAX);
    let Reply::Response(P::Problem { message, field }) =
        exchange(&mut session, invalid_parts, "unrepresentable benefit parts")
    else {
        panic!("located benefit-parts refusal")
    };
    assert_eq!(
        message,
        libertas::libertas_formatted_text("CARE_BENEFIT_PARTS_ABOVE_BOX_10", &[])
    );
    assert_eq!(field.as_deref(), Some("[value/benefits/20/in_box_1]"));
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
fn unit47_care_native_aggregate_failure_precedes_form_output() {
    // Native owner boundary, not a claim that these oversized source documents
    // pass the separate W-2 amount validator. No giant record fixture is needed.
    let mut draft = care_w2_sources(2).draft;
    for entry in &mut draft.income {
        let FederalIncome::Wage { data } = &mut entry.income else {
            unreachable!()
        };
        data.care_benefits = i64::MAX / 2 + 1;
    }
    let care = &draft.credits.as_ref().unwrap().care;
    assert!(crate::forms::form_2441::benefit_sources(&draft, care).is_none());
    let context = crate::forms::form_2441::Form2441Context {
        draft: &draft,
        setup: draft.setup.as_ref().unwrap(),
        credits: draft.credits.as_ref().unwrap(),
        joint: false,
        care_people: 0,
        care_expenses: 0,
        earned: [0; 2],
        optional_eic: [0; 2],
        deduction_owners: [0; 2],
    };
    let mut result = FederalResult::empty(draft.revision);
    assert!(crate::forms::form_2441::compute(&context, &mut result).is_none());
    assert!(result.return_lines.is_empty() && result.lines.is_empty() && result.tax.is_none());
    assert!(
        draft.stored_issue().is_some(),
        "W-2 source validation still rejects its own oversized facts"
    );
    // Lines12 and13 round separately; do not invent their combined Cents
    // carrier when the actual employee net (after forfeiture) remains known.
    draft.income.truncate(1);
    let wage = draft.income[0].id;
    let FederalIncome::Wage { data } = &mut draft.income[0].income else {
        unreachable!()
    };
    data.care_benefits = i64::MAX;
    let care = &mut draft.credits.as_mut().unwrap().care;
    care.benefits = vec![CareBenefitAnswers {
        wage,
        in_box_1: None,
        forfeited: Some(500),
    }];
    care.grace = vec![CareGraceBalance {
        owner: Owner::Taxpayer,
        used_in_return_year: 200,
    }];
    assert!(
        crate::forms::form_2441::benefit_sources(&draft, &draft.credits.as_ref().unwrap().care)
            .is_some()
    );
    // Provider omission is another native source sum, with its own typed
    // failure; ordinary provider identity/amount validation still comes first.
    let care = &mut draft.credits.as_mut().unwrap().care;
    care.providers = crate::tests::support::builders::care_providers(2);
    for provider in &mut care.providers {
        provider.identification = ProviderIdentification::NotObtained;
        provider.paid = i64::MAX;
    }
    assert!(care.omitted_amount().is_none());
    draft.credits.as_mut().unwrap().care.providers[0]
        .name
        .clear();
    let issue = draft.credits.as_ref().unwrap().care.issue(&draft).unwrap();
    assert_eq!(issue.key, "CARE_INVALID_RECORD");
    assert_eq!(issue.field.unwrap().square(), "[providers/0/name]");
    let care = &mut draft.credits.as_mut().unwrap().care;
    care.providers[0].name = "Synthetic provider 0".into();
    care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
    let issue = draft.credits.as_ref().unwrap().care.issue(&draft).unwrap();
    assert_eq!(issue.key, "CARE_SOURCE_AMOUNTS_UNAVAILABLE");
    assert_eq!(issue.field.unwrap().square(), "[providers]");
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
    page.own_business[0].received = 100_000_000_001;
    accepted(&send(&mut session, request));
    assert!(session.draft.valid_stored());
    let result = estimate(&session.draft);
    assert_eq!(result.state, ResultState::EstimatedForSupportedScope);
    assert_eq!(value(&result, "care_form2441_line12"), 1_000_000_002);
    assert_eq!(value(&result, "care_form2441_line22"), 1_000_000_002);
    assert_eq!(value(&result, "care_form2441_line24"), 0);
    let saved = session.draft.clone();
    let mut cold = restart(saved.clone());
    let Reply::Edit(reopened) = from_review(&mut cold, TaxSection::Care) else {
        panic!("Care reopen")
    };
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
}
