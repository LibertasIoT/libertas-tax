use crate::tests::*;

// Complete native preview with no source catalogs; callers supply any actual catalogs explicitly.
pub(in crate::tests) fn survey_preview_document(document: IncomeSource) -> P {
    let topic = super::income_pages::topic_for(&document);
    match document {
        IncomeSource::Record { value } => P::PreviewDocument {
            source_choices: crate::IncomeSourceChoices::from_kinds(
                &super::income_pages::topic_kinds(topic),
            ),
            initial_asset_amt_carry_choices: Vec::new(),
            initial_interest_limit_choices: Vec::new(),
            initial_rental_recovery_choices: Vec::new(),
            rental_energy_buildings: Vec::new(),
            casualty_properties: Vec::new(),
            empty_k1_activity_choices: crate::tests::support::builders::empty_k1_activity_choices(),
            allowed_continuing_risk: ScorpRiskStart::continuing_choices(),
            allowed_injured_source_facts: vec![],
            allowed_injured_retained_owners: vec![],
            ltc_payee_sources: vec![],
            wage_scorporations: Vec::new(),
            allowed_wage_scorporations: Vec::new(),
            initial_k1_investment_choices: alloc::vec![0, 1],
            research_businesses: Vec::new(),
            retained_research_choices: Vec::new(),
            k1_credit_lines: Vec::new(),
            research_credit_lines: Vec::new(),
            renewable_credit_lines: Vec::new(),
            transferable_credit_lines: Vec::new(),
            qbi_businesses: vec![],
            basis_assets: vec![],
            farm_sources: vec![],
            farm_codes: vec![],
            farm_limitations: vec![],
            farm_method_changes: vec![],
            mortgage_loans: vec![],
            foreign_countries: vec![],
            interest_statements: vec![],
            user: TEST_USER,
            id: 0,
            owner: Owner::Taxpayer,
            allowed_business_source: vec![],
            allowed_farm_source: vec![],
            farms: vec![],
            businesses: vec![],
            rentals: vec![],
            royalty_properties: vec![],
            equipment_rentals: vec![],
            hobbies: vec![],
            ccc_lots: vec![],
            inherited_iras: vec![],
            inherited_roth_iras: vec![],
            ltc_insured: vec![],
            annuity_sources: vec![],
            recovery_sources: vec![],
            cooperative_activities: vec![],

            cooperative_properties: vec![],
            allowed_partner_source: vec![],
            allowed_home_spouse: vec![],
            people: vec![],
            allowed_other_parent: vec![],
            children: vec![],
            document: value,
            community: CommunityShare::NotNeeded,
        },
        _ => panic!("standalone document candidates use actual center Save"),
    }
}

pub(in crate::tests) fn unanswered_personal_refusal(request: &P, key: &str) -> bool {
    matches!(
        key,
        "K1_INVESTMENT_NEEDS_CONFIRMATION"
            | "K1_HOUSING_NEEDS_CONFIRMATION"
            | "W2_TP_OCCUPATION_NEEDED"
            | "W2_TT_FLSA_NEEDED"
            | "W2_BOX14_KIND_NEEDED"
            | "W2_BOX14_STATE_NEEDED"
            | "DIV_SECTION_1202_NEEDS_EXCLUSION"
            | "SOURCE_PARTNER_NEEDED"
            | "K3_WITHHOLDING_NEEDS_EACH_PASSIVE_COUNTRY"
    ) || (key == "K_PARTS_NOT_GROSS"
        && matches!(request,
            P::SaveIncomes { followups, .. } if followups.iter().any(|row|
                matches!(&row.answer, crate::DocumentAnswer::PaymentParts(crate::PaymentPartsAnswer { value }) if value.is_empty()))))
}

// One assertion owner for native refusal key/path and atomic accepted state.
pub(in crate::tests) fn survey_refusal(
    case: &str,
    session: &mut Interview,
    request: P,
    key: &str,
    path: &str,
) {
    // The native medical-account record and the printed Form 1099-SA use
    // different names for box 2. Refusal controls keep their native vocabulary.
    let printed_path = (key == "SA1099_EARNINGS_ABOVE_GROSS")
        .then(|| path.strip_suffix("/box2_earnings]"))
        .flatten()
        .map(|parent| alloc::format!("{parent}/earnings_on_excess_contributions]"));
    let path = printed_path.as_deref().unwrap_or(path);
    // The wages editor carries supporting records under its dedicated wrapper.
    let wages_path =
        matches!(&request, P::SaveWagesAnswers { .. } | P::EstimateWagesAnswers { .. })
            .then(|| path.strip_prefix("[documents/documents/"))
            .flatten()
            .map(|tail| alloc::format!("[records/documents/{tail}"));
    let path = wages_path.as_deref().unwrap_or(path);
    if let Some((mut candidate, request, path)) =
        super::income_pages::topic_fixture_control(session, &request, key, path)
    {
        survey_refusal(case, &mut candidate, request.clone(), key, &path);
        let running = super::estimate_documents_from_save(&request);
        if unanswered_personal_refusal(&request, key) {
            let before = candidate.draft.clone();
            let dirty = candidate.dirty;
            assert!(
                matches!(
                    send(&mut candidate, running),
                    Reply::Response(P::IncomeRunningEstimate { .. })
                ),
                "{case}: an unanswered personal election permits a provisional estimate"
            );
            assert_eq!(candidate.draft, before);
            assert_eq!(candidate.dirty, dirty);
        } else {
            survey_refusal(
                &alloc::format!("{case}-topic-estimate"),
                &mut candidate,
                running,
                key,
                &path,
            );
        }
        return;
    }
    let before = session.draft.clone();
    let dirty = session.dirty;
    let Reply::Response(problem @ P::Problem { .. }) = send(session, request.clone()) else {
        panic!("{case}: expected source refusal");
    };
    let P::Problem { message, field } = &problem else { unreachable!() };
    assert_eq!(*message, libertas::libertas_formatted_text(key, &[]), "{case}");
    assert_eq!(field.as_deref(), Some(path), "{case}");
    assert_eq!(session.draft, before, "{case}: rejected candidate wrote accepted source");
    assert_eq!(session.dirty, dirty, "{case}: rejected candidate changed dirty state");
    crate::tests::unit47_rule_cases::export_unit47_rule_case(case, &request, &problem);
}

pub(in crate::tests) fn survey_native_refusal(
    case: &str,
    session: &mut Interview,
    request: P,
    key: &str,
    path: &str,
) {
    let request = P::from_avro(&request.to_avro()).unwrap();
    survey_refusal(case, session, request, key, path);
}

// Both refusal families start from the same accepted two-row native fixture.
// Build it once; each behavior receives its own normal cold session. The
// source-plan/first-answer lifecycle test retains its independent construction.
pub(in crate::tests) fn survey_injured_capacity_session() -> Interview {
    extern crate std;
    static ACCEPTED: std::sync::OnceLock<FederalDraft> = std::sync::OnceLock::new();
    let draft = ACCEPTED.get_or_init(|| super::injured_capacity_accepted_session().draft);
    restart(draft.clone())
}
