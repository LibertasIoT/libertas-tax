use crate::k1_activity::*;
use crate::tests::cross::interview::{PAGE_ADJUSTMENTS, accepted, document, open_topic};
use crate::tests::support::builders::k1res::*;
use crate::tests::*;
pub(crate) fn initial(scorp: bool) -> (Interview, P) {
    initial_with_properties(scorp, false)
}
pub(crate) fn initial_with_properties(scorp: bool, with_properties: bool) -> (Interview, P) {
    initial_with_sources(scorp, with_properties, false)
}
pub(crate) fn initial_with_sources(
    scorp: bool,
    with_properties: bool,
    nullable: bool,
) -> (Interview, P) {
    initial_on_return(scorp, with_properties, nullable, empty_return(FederalStatus::Single))
}
fn initial_on_return(
    scorp: bool,
    with_properties: bool,
    nullable: bool,
    draft: FederalDraft,
) -> (Interview, P) {
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    assert!(documents(&mut request).is_empty());
    let mut income = fresh_income(scorp);
    if with_properties {
        let (items, confirmed) = match &mut income {
            FederalIncome::PartnershipActivities { data } => {
                (&mut data.entity.investment_property, &mut data.entity.investment_confirmed)
            }
            FederalIncome::ScorpActivities { data } => {
                (&mut data.entity.investment_property, &mut data.entity.investment_confirmed)
            }
            _ => unreachable!(),
        };
        *items = [1_000_001, 2_000_001]
            .into_iter()
            .map(|basis| crate::K1InvestmentProperty::Energy {
                id: 0,
                own_form: None,
                basis,
                rate: crate::EnergyRate {
                    increased_rate: true,
                    domestic_content: false,
                    energy_community: false,
                },
            })
            .collect();
        *confirmed = Some(true);
    }
    if nullable {
        nullable_sources(&mut income);
    }
    let mut row = document(0, input(&income));
    assert_eq!(row.id, 0);
    let P::SaveIncomes { allowed_community, .. } = &request else { unreachable!() };
    row.allowed_community = allowed_community.clone();
    assert_eq!(page(&mut row.document).choices, K1ActivityChoiceContext::default());
    documents(&mut request).push(row);
    (session, request)
}
pub(crate) fn accepted_session(scorp: bool) -> Interview {
    let (mut session, request) = initial(scorp);
    accepted(&send(&mut session, request));
    session
}

/// Exercise the real source editor after admission without supplying financial
/// producer output that this source-only graph does not yet have.
pub(crate) fn unchanged_source_roundtrip(session: &mut Interview) {
    let before = session.draft.clone();
    let estimate = crate::federal_rules::estimate(&before);
    assert!(estimate.tax.is_none());
    assert!(
        estimate
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
    );
    let request = open_topic(session, TaxSection::PassThrough);
    accepted(&send(session, request));
    assert_eq!(session.draft, before);
    let mut cold = restart(before.clone());
    let request = open_topic(&mut cold, TaxSection::PassThrough);
    accepted(&send(&mut cold, request));
    assert_eq!(cold.draft, before);
}

pub(crate) fn open_activity_benefits(session: &mut Interview) -> P {
    let request = P::Navigate {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        from: PAGE_ADJUSTMENTS,
        section: TaxSection::SelfEmployedBenefits.into(),
        navigation: NavigationAvailability::default(),
    };
    match send(session, request) {
        Reply::Edit(request @ P::SaveSelfEmployedBenefits { .. }) => request,
        other => panic!("expected normal benefits editor, got {other:?}"),
    }
}

pub(crate) fn activity_benefits(request: &mut P) -> &mut SelfEmployedBenefits {
    let P::SaveSelfEmployedBenefits { value, .. } = request else { unreachable!() };
    value
}

pub(crate) fn partner_source(session: &Interview) -> &PartnerActivityK1 {
    let FederalIncome::PartnershipActivities { data } = &session.draft.income[0].income else {
        unreachable!()
    };
    data
}

pub(crate) fn partner_benefit_session() -> Interview {
    let (mut session, mut request) = initial(false);
    let IncomeDocument::K1PartnershipActivities(data) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    data.entity.regular.distributions = 50_000;
    data.person = None;
    accepted(&send(&mut session, request));
    session
}

pub(crate) fn scorp_benefit_session() -> (Interview, i64) {
    let mut session = accepted_session(true);
    add(&mut session.draft, wage(0, Owner::Taxpayer, 100_000, 0));
    let id = session.draft.income.last().unwrap().id;
    // The accepted W-2 is an independently established synthetic fixture source.
    let session = restart(session.draft);
    (session, id)
}

pub(crate) fn activity_benefit_calculation(request: &P) -> P {
    let P::SaveSelfEmployedBenefits {
        user,
        value,
        ptc_policies,
        health_activities,
        health_people,
        health_wages,
        ..
    } = request
    else {
        unreachable!()
    };
    P::EstimateSelfEmployedBenefits {
        user: *user,
        value: value.clone(),
        ptc_policies: ptc_policies.clone(),
        health_activities: health_activities.clone(),
        health_people: health_people.clone(),
        health_wages: health_wages.clone(),
    }
}

/// Establish accepted identities through the original benefits lifecycle and remap references.
pub(crate) fn ordinary_multi_parent_session(value: &mut PartnerActivityK1) -> Interview {
    ordinary_multi_parent_session_on_return(value, empty_return(FederalStatus::Single))
}

fn ordinary_multi_parent_session_on_return(
    value: &mut PartnerActivityK1,
    draft: FederalDraft,
) -> Interview {
    let (mut session, request) = initial_on_return(false, false, false, draft);
    accepted(&send(&mut session, request));
    let mut benefits = open_activity_benefits(&mut session);
    let benefit = &mut activity_benefits(&mut benefits).activity_partnerships[0];
    benefit.from_distributions = Some(0);
    benefit.health = Some(SeHealth::None);
    benefit.retirement = Some(PartnerPlan::None);
    accepted(&send(&mut session, benefits));
    let saved_source = session
        .draft
        .income
        .iter()
        .find_map(|entry| {
            if matches!(entry.income, FederalIncome::PartnershipActivities { .. }) {
                crate::k1_activity::source(&entry.income)
            } else {
                None
            }
        })
        .unwrap();
    let accepted_ids: Vec<_> = saved_source.ids().collect();
    let K1ComponentStatement::Entered { source: component_source } = &mut value.activities else {
        unreachable!()
    };
    let old_components: Vec<_> =
        component_source.components.iter().map(|component| component.id).collect();
    for (id, accepted) in component_source.ids_mut().zip(accepted_ids) {
        *id = accepted;
    }
    for index in 0..2 {
        component_source.components[index].risk_group =
            Some(component_source.risk_groups[index].id);
        component_source.components[index].passive_group =
            Some(component_source.passive_groups[index].id);
    }
    for block in &mut component_source.qbi_blocks {
        for allocation in block.allocations.iter_mut().flatten() {
            let index = old_components
                .iter()
                .position(|id| *id == allocation.component)
                .expect("fixture allocation names its original component");
            allocation.component = component_source.components[index].id;
        }
    }
    session
}

/// Save the documented retained-QBI fixture through the normal source editor.
pub(crate) fn ordinary_multi_retained_qbi_session() -> Interview {
    let mut value = ordinary_multi_retained_qbi_parent();
    let mut session = ordinary_multi_parent_session(&mut value);
    let income = FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) };
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    documents(&mut request)[0].document = input(&income);
    accepted(&send(&mut session, request));
    session
}

/// Save the current-statement fixture with accepted component/group/QBI identities.
pub(crate) fn ordinary_multi_current_qbi_session() -> Interview {
    ordinary_multi_current_qbi_session_with_statements(
        empty_return(FederalStatus::Single),
        [Some((300_000, 0)), None],
    )
}

/// Preserve the supplied return and Save actual current statements on accepted source identities.
pub(crate) fn ordinary_multi_current_qbi_session_with_statements(
    draft: FederalDraft,
    statements: [Option<(i64, i64)>; 2],
) -> Interview {
    let mut value = ordinary_multi_current_qbi_parent_with_statements(statements);
    let mut session = ordinary_multi_parent_session_on_return(&mut value, draft);
    let income = FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) };
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let row = documents(&mut request)
        .iter_mut()
        .find(|row| matches!(row.document, IncomeDocument::K1PartnershipActivities(_)))
        .unwrap();
    row.document = input(&income);
    accepted(&send(&mut session, request));
    session
}

fn scorp_ordinary_multi_on_return(draft: FederalDraft) -> (Interview, ScorpActivityK1) {
    let (mut session, request) = initial_on_return(true, false, false, draft);
    accepted(&send(&mut session, request));
    let mut benefits = open_activity_benefits(&mut session);
    activity_benefits(&mut benefits).activity_scorporations[0].health = Some(SeHealth::None);
    accepted(&send(&mut session, benefits));
    let value = session
        .draft
        .income
        .iter()
        .find_map(|entry| {
            if let FederalIncome::ScorpActivities { data } = &entry.income {
                Some(data.clone())
            } else {
                None
            }
        })
        .unwrap();
    (session, scorp_ordinary_multi_baseline(value))
}

/// Preserve the original no-wage baseline; its test supplies the original wage later.
pub(crate) fn scorp_ordinary_multi_session() -> (Interview, ScorpActivityK1) {
    scorp_ordinary_multi_on_return(empty_return(FederalStatus::Single))
}

/// One existing complete-return wage, then native benefits and current-source Saves.
pub(crate) fn ordinary_multi_current_scorp_qbi_session() -> Interview {
    ordinary_multi_current_scorp_qbi_session_with_statements(
        complete(FederalStatus::Single),
        [Some((300_000, 0)), None],
    )
}

/// Preserve the supplied return and Save actual current statements on accepted source identities.
pub(crate) fn ordinary_multi_current_scorp_qbi_session_with_statements(
    draft: FederalDraft,
    statements: [Option<(i64, i64)>; 2],
) -> Interview {
    let (mut session, value) = scorp_ordinary_multi_on_return(draft);
    let value = current_scorp_qbi_source_with_statements(value, statements);
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let row = documents(&mut request)
        .iter_mut()
        .find(|row| matches!(row.document, IncomeDocument::K1ScorpActivities(_)))
        .unwrap();
    row.document = input(&FederalIncome::ScorpActivities { data: value });
    accepted(&send(&mut session, request));
    session
}

/// Normal accepted physical sources, followed by the existing association editor.
/// Financial assertions remain in the receiver's distinct property controls.
pub(crate) fn financial_property_session(scorp: bool) -> Interview {
    let draft = complete(FederalStatus::Single);
    let (mut session, request) = initial_on_return(scorp, true, false, draft);
    accepted(&send(&mut session, request));
    let mut benefits = open_activity_benefits(&mut session);
    let value = activity_benefits(&mut benefits);
    if scorp {
        value.activity_scorporations[0].health = Some(SeHealth::None);
    } else {
        let row = &mut value.activity_partnerships[0];
        row.from_distributions = Some(0);
        row.health = Some(SeHealth::None);
        row.retirement = Some(PartnerPlan::None);
    }
    accepted(&send(&mut session, benefits));
    let mut income = session
        .draft
        .income
        .iter()
        .find(|entry| {
            matches!(
                entry.income,
                FederalIncome::PartnershipActivities { .. } | FederalIncome::ScorpActivities { .. }
            )
        })
        .unwrap()
        .income
        .clone();
    financial_property_sources(&mut income);
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    let row = documents(&mut request)
        .iter_mut()
        .find(|row| {
            matches!(
                row.document,
                IncomeDocument::K1PartnershipActivities(_) | IncomeDocument::K1ScorpActivities(_)
            )
        })
        .unwrap();
    row.document = input(&income);
    bind_properties(&mut row.document);
    accepted(&send(&mut session, request));
    session
}

/// Existing accepted identities and wage, with an advance candidate ready for native Save.
pub(crate) fn scorp_cash_advance_candidate(advance: i64) -> (Interview, ScorpActivityK1) {
    let (mut session, mut value) = scorp_ordinary_multi_session();
    add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
    session = restart(session.draft);
    scorp_cash_advance(&mut value, advance);
    (session, value)
}

/// Existing native identities and wage, with actual fully based loan repayment facts.
pub(crate) fn scorp_cash_repayment_candidate(repayment: i64) -> (Interview, ScorpActivityK1) {
    let (session, mut value) = scorp_cash_advance_candidate(0);
    value.entity.loans[0].regular.basis = 500_000;
    value.entity.loans[0].alternative_basis = Some(500_000);
    scorp_cash_repayment(&mut value, repayment);
    (session, value)
}

/// Actual native identities from the existing ordinary K-1 editor lifecycles.
pub(crate) fn entity_exempt_other_session(scorp: bool) -> (Interview, FederalIncome) {
    let (session, mut income) = if scorp {
        let (session, value) = scorp_ordinary_multi_session();
        (session, FederalIncome::ScorpActivities { data: value })
    } else {
        let mut value = ordinary_multi_parent();
        let session = ordinary_multi_parent_session(&mut value);
        (session, FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) })
    };
    configure_entity_exempt_other(&mut income);
    (session, income)
}

pub(crate) fn entity_exempt_interest_session(scorp: bool) -> (Interview, FederalIncome) {
    let (session, mut income) = entity_exempt_other_session(scorp);
    set_entity_exempt_other(&mut income, 0, 0);
    set_entity_exempt_interest(&mut income, 100_000, 100_000);
    (session, income)
}

/// Request the actual native Review reply, including every localized return row.
pub(crate) fn assert_native_review(session: &mut Interview) {
    let expected = FederalEstimate::from(reviewed(session));
    let Reply::Response(P::Review { result, .. }) =
        send(session, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("expected the native Review reply")
    };
    assert_eq!(result.breakdown, expected.breakdown);
    assert_eq!(result.return_lines, expected.return_lines);
    assert_eq!(result.tax, expected.tax);
}

/// Existing native ordinary/exempt fixture with an explicit component source partition.
pub(crate) fn component_exempt_other_session(scorp: bool) -> (Interview, FederalIncome) {
    let (session, mut income) = entity_exempt_other_session(scorp);
    configure_component_exempt_other(&mut income);
    (session, income)
}

pub(crate) fn section179_session(scorp: bool, regular: [i64; 2], amt: [i64; 2]) -> Interview {
    let (mut session, mut income) = entity_exempt_other_session(scorp);
    configure_section179(&mut income, regular, amt);
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    documents(&mut request)[0].document = input(&income);
    accepted(&send(&mut session, request));
    session
}

pub(crate) fn section179_direct(session: &mut Interview, receipts: i64, elected: i64) -> i64 {
    use crate::tests::cross::interview::{asset, business, save_businesses, stored_business};
    let mut value = business(receipts);
    let mut property = asset(elected, 20260601, AssetClass::Five);
    property.acquired = Some(20250120);
    property.history = AssetHistory::New {
        first_year: FirstYearDeduction::Section179 {
            description: "Actual direct equipment".into(),
            elected,
        },
    };
    value.assets.push(property);
    accepted(&save_businesses(session, vec![value]));
    stored_business(session).0
}

/// Add a second physical source through the ordinary allocator, then explicitly
/// select each of its own accepted risk/passive groups in the source editor.
pub(crate) fn section179_second_parent(session: &mut Interview) {
    let mut income = session.draft.income[0].income.clone();
    let FederalIncome::PartnershipActivities { data } = &mut income else { unreachable!() };
    // A new physical source has no accepted benefits projection to copy.
    data.person.as_mut().unwrap().health_distributions = None;
    data.health_insurance = None;
    data.retirement = None;
    for id in crate::k1_activity::source_ids_mut(&mut income) {
        *id = 0;
    }
    let (actual_source, _, _) = entity_exempt_other_fields(&mut income);
    for component in &mut actual_source.components {
        component.risk_group = None;
        component.passive_group = None;
    }
    let mut request = open_topic(session, TaxSection::PassThrough);
    let mut row = documents(&mut request)[0].clone();
    row.id = 0;
    row.document = input(&income);
    documents(&mut request).push(row);
    accepted(&send(session, request));
    let parent_id = session.draft.income[1].id;
    let mut benefits = open_activity_benefits(session);
    let benefit = activity_benefits(&mut benefits)
        .activity_partnerships
        .iter_mut()
        .find(|benefit| benefit.id == parent_id)
        .unwrap();
    benefit.from_distributions = Some(0);
    benefit.health = Some(SeHealth::None);
    benefit.retirement = Some(PartnerPlan::None);
    accepted(&send(session, benefits));
    let mut income = session.draft.income[1].income.clone();
    let (actual_source, _, _) = entity_exempt_other_fields(&mut income);
    for (index, component) in actual_source.components.iter_mut().enumerate() {
        component.risk_group = Some(actual_source.risk_groups[index].id);
        component.passive_group = Some(actual_source.passive_groups[index].id);
    }
    let mut request = open_topic(session, TaxSection::PassThrough);
    documents(&mut request)[1].document = input(&income);
    accepted(&send(session, request));
}
