use super::*;

#[test]
fn medical_source_overflow_is_refused_before_reconciliation() {
    let (mut session, _) = at_the_deductions_page(FederalStatus::Single);
    let request = save_deductions(&session.draft, DeductionRoute::Compare, 0);
    itemized_hub(send_past_less_common(&mut session, request));
    let P::SaveMedical { mut value, .. } = open_itemized_topic(&mut session, TaxSection::Medical)
    else {
        panic!("expected medical editor")
    };
    // Original 5f361dc8 reproduction: valid catalog/LTC1 plus invalid MAX
    // expenses overflowed before admission. The 2026-10-16 allocation keeps
    // source priority. With the arbitrary field cap removed, MAX+1 is
    // refused at the actual combined source carrier before reconciliation.
    value.expenses = i64::MAX;
    value.long_term_care = vec![CarePremium {
        qualified: CareContractQualification::Qualified,
        person: -1,
        premiums: 1,
    }];
    let invalid = "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7";
    let before = session.draft.clone();
    let dirty = session.dirty;
    for case in 0..5 {
        let mut bad = value.clone();
        let (key, path) = match case {
            0 => {
                bad.people.clear();
                bad.long_term_care[0].person = 999_999;
                ("MESSAGE_THE_PEOPLE_ON_THIS_RETURN_CHANGED_REOPEN_THIS_PAGE_BEFORE_SAVING", None)
            }
            1 => {
                bad.long_term_care[0].premiums = -1;
                let mut missing = bad.long_term_care[0].clone();
                missing.person = 999_999;
                bad.long_term_care.push(missing);
                (
                    "MESSAGE_A_PREMIUM_NAMES_SOMEONE_WHO_IS_NOT_ON_THIS_RETURN_CHOOSE_THE_INSURED_PERSON",
                    Some("[value/long_term_care/1/person]"),
                )
            }
            2 => {
                bad.long_term_care[0].qualified = CareContractQualification::Grandfathered {
                    issued: 20260101,
                    state_compliant: true,
                    materially_changed: false,
                };
                bad.long_term_care[0].premiums = -1;
                (invalid, Some("[value/long_term_care/0/qualified]"))
            }
            3 => {
                bad.long_term_care[0].premiums = -1;
                (invalid, Some("[value/long_term_care/0/premiums]"))
            }
            _ => (invalid, Some("[value]")),
        };
        let estimate = P::EstimateMedical { user: TEST_USER, value: bad.clone() };
        let save = save_medical(&session.draft, bad);
        for request in [estimate, save] {
            let Reply::Response(P::Problem { message, field }) = send(&mut session, request) else {
                panic!("medical source must refuse before reconciliation: case {case}")
            };
            assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
            assert_eq!(field.as_deref(), path);
            assert_eq!(session.draft, before);
            assert_eq!(session.dirty, dirty);
        }
    }

    value.expenses = 100_000_000_001;
    assert!(matches!(
        send(&mut session, P::EstimateMedical { user: TEST_USER, value: value.clone() },),
        Reply::Response(P::RunningEstimate { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    let save = save_medical(&session.draft, value.clone());
    accepted(&send(&mut session, save));
    let saved = session.draft.clone();
    assert_eq!(
        crate::tests::support::assert::value(&supported(&saved), "medical_expenses"),
        1_000_000_000,
    );
    let P::SaveMedical { value: reopened, .. } =
        open_itemized_topic(&mut session, TaxSection::Medical)
    else {
        panic!("expected medical editor")
    };
    assert_eq!(reopened, value);
    let save = save_medical(&session.draft, reopened);
    accepted(&send(&mut session, save));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let P::SaveMedical { value: reopened, .. } =
        open_itemized_topic(&mut cold, TaxSection::Medical)
    else {
        panic!("expected cold medical editor")
    };
    assert_eq!(reopened, value);
    let save = save_medical(&cold.draft, reopened);
    accepted(&send(&mut cold, save));
    assert_eq!(cold.draft, saved);

    // Normal route change makes the itemized source unavailable. Its existing
    // owner boundary must win over the old invalid-scalar reproduction.
    let Reply::Edit(mut page) = from_review(&mut session, TaxSection::Deductions) else {
        panic!("expected deductions editor")
    };
    let P::SaveDeductions { value: route, .. } = &mut page else {
        panic!("expected deductions editor")
    };
    route.route = DeductionRoute::Standard;
    accepted(&send(&mut session, page));
    value.expenses = i64::MAX;
    let before = session.draft.clone();
    let dirty = session.dirty;
    let save = save_medical(&session.draft, value.clone());
    for request in [P::EstimateMedical { user: TEST_USER, value }, save] {
        let Reply::Response(P::Problem { message, field }) = send(&mut session, request) else {
            panic!("unavailable itemized source must precede its invalid amount")
        };
        assert_eq!(
            message,
            libertas::libertas_formatted_text("PROBLEM_SECTION_UNAVAILABLE_YET", &[])
        );
        assert_eq!(field, None);
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
    }
}

#[test]
fn unit47_ltc_personal_policies_keep_native_hub_financial_and_cold_save() {
    let (mut session, _) = at_the_deductions_page(FederalStatus::Single);
    let request = save_deductions(&session.draft, DeductionRoute::Itemize, 0);
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
    let mut invalid = facts.clone();
    invalid.long_term_care[100].premiums = -1;
    let before = session.draft.clone();
    let request = save_medical(&session.draft, invalid);
    let Reply::Response(P::Problem { field, .. }) = send(&mut session, request) else {
        panic!("last actual premium refusal")
    };
    assert_eq!(field.as_deref(), Some("[value/long_term_care/100/premiums]"));
    assert_eq!(session.draft, before);
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
    let Reply::Edit(mut request) = from_review(&mut session, TaxSection::SelfEmployedBenefits)
    else {
        panic!("actual business benefits editor")
    };
    let correct = request.clone();
    let P::SaveSelfEmployedBenefits { value: facts, .. } = &mut request else { unreachable!() };
    let SeHealth::Plan { long_term_care, .. } = &mut facts.businesses[0].health else {
        unreachable!()
    };
    assert_eq!(long_term_care.len(), 101);
    long_term_care[100].months[0].premiums = -1;
    let before = session.draft.clone();
    let Reply::Response(P::Problem { field, .. }) = send(&mut session, request) else {
        panic!("last actual business premium refusal")
    };
    assert_eq!(
        field.as_deref(),
        Some("[value/businesses/0/health/long_term_care/100/months/0/premiums]")
    );
    assert_eq!(session.draft, before);
    accepted(&send(&mut session, correct));
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

#[test]
fn unit47_medical_required_schedule_a_carriers_refuse_wide_rows_without_losing_sources() {
    for joined_medical in [true, false] {
        let mut draft = complete(FederalStatus::Single);
        let ded = draft.deductions.as_mut().unwrap();
        itemized(ded).medical = if joined_medical { i64::MAX } else { i64::MAX - 10_000 };
        ded.choice = DeductionSelection::Itemized { facts: itemized(ded).clone() };
        if joined_medical {
            let claim = marketplace_claim(&mut draft, vec![marketplace_month(1, 100, 100, 0)]);
            draft.credits.as_mut().unwrap().marketplace = claim;
        } else {
            draft.deductions.as_mut().unwrap().cash_charity = 700_000;
        }
        assert!(draft.valid_stored());
        let before = draft.clone();
        let result = estimate(&draft);
        // 2026-10-16 dispatcher defect seam: route each Schedule A failure to its owner.
        let (key, section) = if joined_medical {
            ("SCHEDULE_A_MEDICAL_SUBTOTAL_UNAVAILABLE", crate::TaxSection::Medical)
        } else {
            ("SCHEDULE_A_ITEMIZED_TOTAL_UNAVAILABLE", crate::TaxSection::Deductions)
        };
        assert!(result.issues.iter().any(|issue| {
            issue.display.key == key && issue.context == crate::IssueContext::Section(section)
        }));
        assert!(result.tax.is_none() && result.return_lines.is_empty());
        assert_eq!(draft, before);
    }
}
