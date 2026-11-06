use super::*;

#[test]
fn unit47_ltc_personal_policies_keep_native_hub_financial_and_cold_save() {
    let (mut session, _) = at_the_deductions_page(FederalStatus::Single);
    let request = save_deductions(&session.draft, Some(ItemizeOverride::Itemize), 0);
    itemized_hub(send_past_less_common(&mut session, request));
    let P::SaveMedical { value: mut facts, .. } =
        open_itemized_topic(&mut session, TaxSection::Medical)
    else {
        panic!("actual medical editor")
    };
    facts.expenses = 100;
    facts.long_term_care = vec![
        CarePremium {
            qualified: CareContractQualification::Qualified,
            person: -1,
            premiums: 100,
        };
        101
    ];
    let request = save_medical(&session.draft, facts.clone());
    let hub = itemized_hub(send_past_less_common(&mut session, request));
    assert!(hub_rows(&hub).contains(&row_of(
        "Medical and dental expenses",
        "$102.00",
        TaxSection::Medical,
    )));
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    assert_eq!(stored_facts(&session).long_term_care.len(), 101);
    assert_eq!(
        crate::health::medical_source_amount(100, &stored_facts(&session).long_term_care),
        Ok(10_200)
    );
    assert_eq!(value(&supported(&saved), "medical_expenses"), 102);
    let mut cold = restart(saved.clone());
    let P::SaveMedical { value: reopened, .. } =
        open_itemized_topic(&mut cold, TaxSection::Medical)
    else {
        panic!("cold medical editor")
    };
    assert_eq!(reopened, facts);
    let request = save_medical(&cold.draft, reopened);
    accepted(&send(&mut cold, request));
    assert_eq!(cold.draft, saved);
}

#[test]
fn unit47_ltc_business_policies_keep_native_limit_and_cold_save() {
    let mut draft = complete(FederalStatus::Single);
    let mut business = care_business(1_000, 1, false);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    let BusinessHealthInsurance::Plan { long_term_care, .. } = &mut data.health_insurance else {
        unreachable!()
    };
    long_term_care.resize(101, long_term_care[0].clone());
    add(&mut draft, income(business));
    let id = draft.income.last().unwrap().id;
    let mut session = Interview::new(TEST_USER, draft);
    let Reply::Edit(request) = from_review(&mut session, TaxSection::SelfEmployedBenefits) else {
        panic!("actual business benefits editor")
    };
    let P::SaveSelfEmployedBenefits { value: facts, .. } = &request else { unreachable!() };
    let SeHealth::Plan { long_term_care, .. } = &facts.businesses[0].health else { unreachable!() };
    assert_eq!(long_term_care.len(), 101);
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    assert!(saved.valid_stored());
    assert_eq!(value(&supported(&saved), &alloc::format!("business_{id}_health_insurance")), 101);
    let mut cold = restart(saved.clone());
    let Reply::Edit(reopened) = from_review(&mut cold, TaxSection::SelfEmployedBenefits) else {
        panic!("cold benefits editor")
    };
    let P::SaveSelfEmployedBenefits { value: facts, .. } = &reopened else { unreachable!() };
    let SeHealth::Plan { long_term_care, .. } = &facts.businesses[0].health else { unreachable!() };
    assert_eq!(long_term_care.len(), 101);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
}

#[test]
fn unit47_ltc_native_carriers_preserve_dollar_join_and_pool_failure() {
    // Native carrier boundaries, not a claim that MAX-valued input documents
    // pass their separate scalar validators. No giant source fixture is needed.
    assert_eq!(
        crate::health::medical_total(&[i64::MAX, i64::MAX], &crate::kernel::ratio::exact::whole(0),)
            .unwrap(),
        184_467_440_737_095_516,
    );
    let above_dollars = crate::kernel::ratio::exact::whole(i64::MAX)
        * crate::kernel::ratio::exact::whole(100)
        + crate::kernel::ratio::exact::whole(100);
    assert!(matches!(
        crate::health::medical_total(&[], &above_dollars),
        Err(crate::health::CalculationIssue::Source(_)),
    ));
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(care_business(1_000, 1, false)));
    let mut public_safety = crate::public_safety::PublicSafetyResult::default();
    public_safety.long_term.insert(-1, i64::MAX);
    let mut result = FederalResult::empty(draft.revision);
    let Err(failure) = crate::health::calculate_with_money(
        &draft,
        [0; 2],
        None,
        &alloc::collections::BTreeMap::new(),
        &crate::sep::SepResult::default(),
        &public_safety,
        None,
        crate::marketplace::CalculationMoney::Dollars,
        None,
        &mut result,
        None,
    ) else {
        panic!("actual insured-person pool must remain unavailable")
    };
    assert!(matches!(&failure, crate::health::CalculationIssue::Source(_)));
    failure.report(&mut result);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(result.lines.is_empty());
    assert!(result.return_lines.is_empty());
}
