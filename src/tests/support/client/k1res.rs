use crate::k1_activity::*;
use crate::tests::cross::interview::{PAGE_ADJUSTMENTS, accepted};
use crate::tests::support::builders::k1res::*;
use crate::tests::*;

/// Supplied fixture answers go through the printed editor and their actual topic.
pub(crate) fn send_fixture(session: &mut Interview, mut request: P) -> Reply {
    if matches!(request, P::SaveDocuments { .. }) {
        name_fixture_sources(&session.draft, &mut request);
        crate::tests::support::client::income_pages::save_source_fixture_request(session, request)
    } else {
        send(session, request)
    }
}

fn name_fixture_sources(draft: &FederalDraft, request: &mut P) {
    for row in documents(request) {
        let owner = draft
            .income
            .iter()
            .find(|entry| entry.id == row.id)
            .map_or(Owner::Taxpayer, |entry| entry.owner.owned());
        let named = crate::tests::support::builders::named_document(
            draft,
            if owner == Owner::Spouse { -2 } else { -1 },
            row.id,
            IncomeSource::K1 { value: row.document.clone() },
        );
        let IncomeSource::K1 { value } = named.document else { unreachable!() };
        row.document = value;
    }
}

fn accept_source_answers(session: &mut Interview, mut request: P) {
    name_fixture_sources(&session.draft, &mut request);
    let rows = crate::tests::support::client::income_pages::page_documents(&request)
        .into_iter()
        .filter(|row| matches!(row.document, IncomeSource::K1 { .. }))
        .collect();
    accepted(&crate::tests::support::client::income_pages::save_k1_sources_and_answers(
        session, rows,
    ));
}
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
    crate::tests::support::client::admit_native_fixture_facts(&mut session);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
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
    let mut row = source_row(0, input(&income));
    let named = crate::tests::support::builders::named_document(
        &session.draft,
        -1,
        0,
        IncomeSource::K1 { value: row.document },
    );
    let IncomeSource::K1 { value } = named.document else { unreachable!() };
    row.document = value;
    assert_eq!(row.id, 0);
    let P::SaveDocuments { allowed_community, .. } = &request else { unreachable!() };
    row.allowed_community = allowed_community.clone();
    assert_eq!(page(&mut row.document).choices, K1ActivityChoiceContext::default());
    documents(&mut request).push(row);
    (session, request)
}
pub(crate) fn accepted_session(scorp: bool) -> Interview {
    let (mut session, request) = initial(scorp);
    accept_source_answers(&mut session, request);
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
    let request = crate::tests::support::client::income_pages::open_document_center(session);
    accepted(&send(session, request));
    assert_eq!(session.draft, before);
    let mut cold = restart(before.clone());
    let request = crate::tests::support::client::income_pages::open_document_center(&mut cold);
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
    let crate::K1Document::K1PartnershipActivities(data) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    data.entity.regular.as_mut().unwrap().distributions = 50_000;
    data.person = None;
    accept_source_answers(&mut session, request);
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
    accept_source_answers(&mut session, request);
    let source_id = session
        .draft
        .income
        .iter()
        .find_map(|entry| {
            matches!(entry.income, FederalIncome::PartnershipActivities { .. }).then_some(entry.id)
        })
        .unwrap();
    let mut benefits = open_activity_benefits(&mut session);
    let benefit = activity_benefits(&mut benefits)
        .activity_partnerships
        .iter_mut()
        .find(|benefit| benefit.id == source_id)
        .unwrap();
    benefit.from_distributions = Some(0);
    benefit.health = Some(SeHealth::None);
    benefit.retirement = Some(PartnerPlan::None);
    accepted(&send(&mut session, benefits));
    let saved_income =
        &session.draft.income.iter().find(|entry| entry.id == source_id).unwrap().income;
    let saved_source = crate::k1_activity::source(saved_income).unwrap();
    let K1ComponentStatement::Entered { source: component_source } = &mut value.activities else {
        unreachable!()
    };
    let mut accepted_ids = alloc::collections::BTreeMap::new();
    for component in &component_source.components {
        accepted_ids.insert(
            component.id,
            saved_source.components.iter().find(|saved| saved.name == component.name).unwrap().id,
        );
    }
    for group in &component_source.risk_groups {
        accepted_ids.insert(
            group.id,
            saved_source.risk_groups.iter().find(|saved| saved.name == group.name).unwrap().id,
        );
    }
    for group in &component_source.passive_groups {
        accepted_ids.insert(
            group.id,
            saved_source.passive_groups.iter().find(|saved| saved.name == group.name).unwrap().id,
        );
    }
    for block in &component_source.qbi_blocks {
        accepted_ids.insert(
            block.id,
            saved_source.qbi_blocks.iter().find(|saved| saved.name == block.name).unwrap().id,
        );
    }
    for id in component_source.ids_mut() {
        *id = accepted_ids[id];
    }
    for component in &mut component_source.components {
        component.risk_group = component.risk_group.map(|id| accepted_ids[&id]);
        component.passive_group = component.passive_group.map(|id| accepted_ids[&id]);
    }
    for block in &mut component_source.qbi_blocks {
        for allocation in block.allocations.iter_mut().flatten() {
            allocation.component = accepted_ids[&allocation.component];
        }
    }
    for binding in component_source.investment_property_sources.iter_mut().flatten() {
        binding.component = accepted_ids[&binding.component];
    }
    for binding in component_source.section179_sale_sources.iter_mut().flatten() {
        binding.component = accepted_ids[&binding.component];
    }
    session
}

/// Save the documented retained-QBI fixture through the normal source editor.
pub(crate) fn ordinary_multi_retained_qbi_session() -> Interview {
    let mut value = ordinary_multi_retained_qbi_parent();
    let mut session = ordinary_multi_parent_session(&mut value);
    let income = FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) };
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    documents(&mut request)[0].document = input(&income);
    accept_source_answers(&mut session, request);
    session
}

/// Save current and retained sources together using the original accepted native identities.
pub(crate) fn ordinary_multi_current_retained_qbi_session(
    mut value: PartnerActivityK1,
    wages: i64,
) -> Interview {
    let mut session = ordinary_multi_parent_session(&mut value);
    add(&mut session.draft, wage(0, Owner::Taxpayer, wages, 0));
    session = restart(session.draft);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    documents(&mut request)[0].document =
        input(&FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) });
    accept_source_answers(&mut session, request);
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
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let row = documents(&mut request)
        .iter_mut()
        .find(|row| matches!(row.document, crate::K1Document::K1PartnershipActivities(_)))
        .unwrap();
    row.document = input(&income);
    accept_source_answers(&mut session, request);
    session
}

fn scorp_ordinary_multi_on_return(draft: FederalDraft) -> (Interview, ScorpActivityK1) {
    let (mut session, request) = initial_on_return(true, false, false, draft);
    accept_source_answers(&mut session, request);
    let source_id = session
        .draft
        .income
        .iter()
        .find_map(|entry| {
            matches!(entry.income, FederalIncome::ScorpActivities { .. }).then_some(entry.id)
        })
        .unwrap();
    let mut benefits = open_activity_benefits(&mut session);
    activity_benefits(&mut benefits)
        .activity_scorporations
        .iter_mut()
        .find(|benefit| benefit.activity == source_id)
        .unwrap()
        .health = Some(SeHealth::None);
    accepted(&send(&mut session, benefits));
    let FederalIncome::ScorpActivities { data } =
        &session.draft.income.iter().find(|entry| entry.id == source_id).unwrap().income
    else {
        unreachable!()
    };
    let value = data.clone();
    (session, scorp_ordinary_multi_baseline(value))
}

/// Preserve the original no-wage baseline; its test supplies the original wage later.
pub(crate) fn scorp_ordinary_multi_session() -> (Interview, ScorpActivityK1) {
    scorp_ordinary_multi_on_return(empty_return(FederalStatus::Single))
}

pub(crate) fn scorp_entity_interest_session(
    draft: FederalDraft,
    interest: i64,
    losses: [i64; 2],
    capacity: i64,
    passive_history: i64,
) -> (Interview, ScorpActivityK1) {
    let (session, value) = scorp_ordinary_multi_on_return(draft);
    (session, scorp_entity_interest_source(value, interest, losses, capacity, passive_history))
}

pub(crate) fn scorp_source_request(session: &mut Interview, value: ScorpActivityK1) -> P {
    let mut request = crate::tests::support::client::income_pages::open_document_center(session);
    let row = documents(&mut request)
        .iter_mut()
        .find(|row| matches!(row.document, crate::K1Document::K1ScorpActivities(_)))
        .unwrap();
    row.document = input(&FederalIncome::ScorpActivities { data: value });
    request
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
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let row = documents(&mut request)
        .iter_mut()
        .find(|row| matches!(row.document, crate::K1Document::K1ScorpActivities(_)))
        .unwrap();
    row.document = input(&FederalIncome::ScorpActivities { data: value });
    accept_source_answers(&mut session, request);
    session
}

// The callback exposes native source fields for missing/mixed gross controls,
// passive participation and retained-history boundaries, without fixture copies.
pub(crate) fn ordinary_multi_current_loss_qbi_session(
    scorp: bool,
    facts: &CurrentLossQbiFacts,
    configure: impl FnOnce(&mut FederalIncome),
) -> Interview {
    let (mut session, mut income) = if scorp {
        let (session, value) = scorp_ordinary_multi_on_return(complete(FederalStatus::Single));
        (
            session,
            FederalIncome::ScorpActivities { data: current_loss_scorp_qbi_source(value, facts) },
        )
    } else {
        let mut value = ordinary_multi_parent();
        let session =
            ordinary_multi_parent_session_on_return(&mut value, complete(FederalStatus::Single));
        (
            session,
            FederalIncome::PartnershipActivities {
                data: alloc::boxed::Box::new(current_loss_partner_qbi_source(value, facts)),
            },
        )
    };
    configure(&mut income);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let row = documents(&mut request)
        .iter_mut()
        .find(|row| {
            matches!(
                row.document,
                crate::K1Document::K1PartnershipActivities(_)
                    | crate::K1Document::K1ScorpActivities(_)
            )
        })
        .unwrap();
    row.document = input(&income);
    accept_source_answers(&mut session, request);
    session
}

/// Normal accepted physical sources, followed by the existing association editor.
/// Financial assertions remain in the receiver's distinct property controls.
pub(crate) fn financial_property_session(scorp: bool) -> Interview {
    let draft = complete(FederalStatus::Single);
    let (mut session, request) = initial_on_return(scorp, true, false, draft);
    accept_source_answers(&mut session, request);
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
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let row = documents(&mut request)
        .iter_mut()
        .find(|row| {
            matches!(
                row.document,
                crate::K1Document::K1PartnershipActivities(_)
                    | crate::K1Document::K1ScorpActivities(_)
            )
        })
        .unwrap();
    row.document = input(&income);
    bind_properties(&mut row.document);
    accept_source_answers(&mut session, request);
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

/// Establish durable source IDs and existing benefits before the dividend Save.
pub(crate) fn entity_dividend_session(
    scorp: bool,
    draft: FederalDraft,
    losses: bool,
    regular: [i64; 2],
    alternative: Option<[i64; 2]>,
) -> Interview {
    entity_dividend_session_for_owner(scorp, draft, Owner::Taxpayer, losses, regular, alternative)
}

pub(crate) fn entity_dividend_session_for_owner(
    scorp: bool,
    draft: FederalDraft,
    owner: Owner,
    losses: bool,
    regular: [i64; 2],
    alternative: Option<[i64; 2]>,
) -> Interview {
    let iso_answers: Vec<_> = draft
        .screening
        .iter()
        .flat_map(|screening| &screening.iso_exercises)
        .map(|exercise| {
            crate::tests::support::builders::named_document(
                &draft,
                if exercise.owner == Owner::Spouse { -2 } else { -1 },
                exercise.id,
                IncomeSource::InvestmentSale {
                    value: crate::InvestmentSaleDocument::Form3921(
                        crate::tests::support::builders::iso_document(exercise.clone()),
                    ),
                },
            )
        })
        .collect();
    let (mut session, mut value) = if scorp {
        let (session, value) = scorp_ordinary_multi_on_return(draft);
        (session, FederalIncome::ScorpActivities { data: value })
    } else {
        let mut value = ordinary_multi_parent();
        let session = ordinary_multi_parent_session_on_return(&mut value, draft);
        (session, FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) })
    };
    prepare_entity_dividends(&mut value, losses);
    set_entity_dividends(&mut value, regular, alternative);
    assert!(crate::k1_activity::income_issue(&value).is_none());
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let mut document = IncomeSource::K1 { value: input(&value) };
    let recipient =
        fixture_document_names(&session.draft, if owner == Owner::Spouse { -2 } else { -1 })
            .expect("the source owner has household facts")
            .recipient;
    set_fixture_recipient(&mut document, &recipient);
    let IncomeSource::K1 { value: document } = document else { unreachable!() };
    documents(&mut request)[0].document = document;
    accept_source_answers(&mut session, request);
    if !iso_answers.is_empty() {
        crate::tests::support::client::income_pages::save_information_return_answers(
            &mut session,
            &iso_answers,
        );
    }
    session
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
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    documents(&mut request)[0].document = input(&income);
    accept_source_answers(&mut session, request);
    session
}

/// Accept an independently supplied ordinary graph through the native source editor.
pub(crate) fn ordinary_multi_source_session(mut value: PartnerActivityK1) -> Interview {
    let mut session = ordinary_multi_parent_session(&mut value);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let income = FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) };
    documents(&mut request)[0].document = input(&income);
    accept_source_answers(&mut session, request);
    session
}

pub(crate) fn section179_direct(session: &mut Interview, receipts: i64, elected: i64) -> i64 {
    use crate::tests::cross::interview::{asset, business, save_businesses, stored_business};
    let mut value = business(receipts);
    let mut property = asset(elected, 20260601, AssetClass::Five);
    property.acquired = Some(20250120);
    property.gds_mut().unwrap().history = AssetHistory::New {
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
    let mut request = crate::tests::support::client::income_pages::open_document_center(session);
    let mut row = documents(&mut request)[0].clone();
    row.id = 0;
    row.document = input(&income);
    documents(&mut request).push(row);
    accept_source_answers(session, request);
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
    let mut request = crate::tests::support::client::income_pages::open_document_center(session);
    documents(&mut request)[1].document = input(&income);
    accept_source_answers(session, request);
}

/// Native graph Save supplies identities; ordinary wages provide uncapped QBI capacity.
pub(crate) fn ordinary_se_qbi_session(value: PartnerActivityK1) -> Interview {
    let mut session = ordinary_multi_source_session(value);
    add(&mut session.draft, wage(0, Owner::Taxpayer, 6_000_000, 0));
    restart(session.draft)
}

// Preserve one complete-return wage and establish benefits before replacing the source.
pub(crate) fn ordinary_multi_passive_qbi_session(qualified: i64, active: [bool; 2]) -> Interview {
    let mut value = ordinary_multi_passive_qbi_parent(qualified, active);
    let mut session =
        ordinary_multi_parent_session_on_return(&mut value, complete(FederalStatus::Single));
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let row = documents(&mut request)
        .iter_mut()
        .find(|row| matches!(row.document, crate::K1Document::K1PartnershipActivities(_)))
        .unwrap();
    row.document =
        input(&FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) });
    accept_source_answers(&mut session, request);
    session
}

pub(crate) fn ordinary_multi_passive_scorp_qbi_session(
    qualified: i64,
    active: [bool; 2],
) -> Interview {
    let (mut session, value) = scorp_ordinary_multi_on_return(complete(FederalStatus::Single));
    let value = passive_scorp_qbi_source(value, qualified, active);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let row = documents(&mut request)
        .iter_mut()
        .find(|row| matches!(row.document, crate::K1Document::K1ScorpActivities(_)))
        .unwrap();
    row.document = input(&FederalIncome::ScorpActivities { data: value });
    accept_source_answers(&mut session, request);
    session
}

/// The ordinary native benefits lifecycle establishes IDs before this candidate Save.
pub(crate) fn partner_entity_interest_session(
    draft: FederalDraft,
    interest: i64,
    operating: [i64; 2],
    capacity: i64,
    passive_history: i64,
) -> (Interview, alloc::boxed::Box<PartnerActivityK1>) {
    let mut value = partner_entity_interest_source(interest, operating, capacity, passive_history);
    let session = ordinary_multi_parent_session_on_return(&mut value, draft);
    (session, alloc::boxed::Box::new(value))
}

pub(crate) fn partner_source_request(
    session: &mut Interview,
    value: alloc::boxed::Box<PartnerActivityK1>,
) -> P {
    let mut request = crate::tests::support::client::income_pages::open_document_center(session);
    let row = documents(&mut request)
        .iter_mut()
        .find(|row| matches!(row.document, crate::K1Document::K1PartnershipActivities(_)))
        .unwrap();
    row.document = input(&FederalIncome::PartnershipActivities { data: value });
    request
}

pub(crate) fn entity_capital_session(scorp: bool) -> (Interview, FederalIncome) {
    let (session, mut income) = entity_exempt_other_session(scorp);
    configure_entity_capital(&mut income);
    (session, income)
}

pub(crate) fn component_capital_session(scorp: bool, scale: i64) -> (Interview, FederalIncome) {
    let (session, mut income) = if scorp {
        let (session, value) = scorp_ordinary_multi_session();
        (session, FederalIncome::ScorpActivities { data: value })
    } else {
        let mut value = ordinary_multi_parent();
        let session = ordinary_multi_parent_session(&mut value);
        (session, FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) })
    };
    configure_component_capital_source(&mut income, scale);
    (session, income)
}

/// The actual keyed depletion question on its existing K-1 topic.
pub(crate) fn depletion_page(request: &mut P, id: i64) -> &mut crate::K1DepletionSourcesPage {
    let P::SaveIncomes { followups, .. } = request else { unreachable!() };
    let key = alloc::format!("income/{id}/depletion_sources");
    let row = followups.iter_mut().find(|row| row.key == key).unwrap();
    let crate::DocumentAnswer::K1DepletionSources { value } = &mut row.answer else {
        unreachable!()
    };
    value
}
