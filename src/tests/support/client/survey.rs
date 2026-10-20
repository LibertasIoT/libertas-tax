use crate::tests::*;

// Complete native preview with no source catalogs; callers supply any actual catalogs explicitly.
pub(in crate::tests) fn survey_preview_document(document: IncomeDocument) -> P {
    P::PreviewDocument {
        initial_rental_recovery_choices: Vec::new(),
        rental_energy_buildings: Vec::new(),
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
        document,
        community: CommunityShare::NotNeeded,
    }
}

// One assertion owner for native refusal key/path and atomic accepted state.
pub(in crate::tests) fn survey_refusal(
    case: &str,
    session: &mut Interview,
    request: P,
    key: &str,
    path: &str,
) {
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

// Both refusal families start from the same accepted full-width native fixture.
// Build it once; each behavior receives its own normal cold session. The
// source-plan/first-answer lifecycle test retains its independent construction.
pub(in crate::tests) fn survey_injured_capacity_session() -> Interview {
    extern crate std;
    static ACCEPTED: std::sync::OnceLock<FederalDraft> = std::sync::OnceLock::new();
    let draft = ACCEPTED.get_or_init(|| super::injured_capacity_accepted_session().draft);
    restart(draft.clone())
}
