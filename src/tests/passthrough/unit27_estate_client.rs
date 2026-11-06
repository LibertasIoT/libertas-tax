use super::*;
use crate::tests::cross::interview::{accepted, document};
use crate::tests::support::client::income_pages::{open_document_center, save_documents};
use alloc::string::ToString;
use libertas::SchemaPath;

// Optional Avro transport uses the ordinary editor, callbacks and Save path.
// The native expected source is only a list of visible synthetic answers;
// the actual client must construct its row from the empty editor's template.
#[test]
fn unit27_estate_fresh_client_saves_whole_source_and_reopens_after_restart() {
    extern crate std;
    let owner = if std::env::var("TAX_B11_ESTATE_OWNER").as_deref() == Ok("spouse") {
        Owner::Spouse
    } else {
        Owner::Taxpayer
    };
    let mut source = super::unit27_estate_community::source();
    // Fresh printed input has no final-entitlement answer when no final records apply.
    source.final_year_successor = None;
    let mut draft = super::unit27_estate_community::draft(owner);
    draft
        .income
        .retain(|entry| !matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. }));
    let mut session = Interview::new(TEST_USER, draft);
    let seed = open_document_center(&mut session);
    let P::SaveDocuments { documents, allowed_community, .. } = &seed else { unreachable!() };
    assert!(documents.common.k1.documents.is_empty());
    let mut row = document(
        0,
        IncomeSource::K1 {
            value: crate::K1Document::K1EstateTrust(estate_document(
                K1EstateTrust::from_record("Synthetic componentized trust", &source).unwrap(),
            )),
        },
    );
    row.owner = crate::DocumentOwner::from(owner);
    row.community = CommunityShare::Separate;
    row.allowed_community = allowed_community.clone();
    let expected_document = crate::tests::support::builders::expected_named_source(
        &session.draft,
        owner,
        row.document.clone(),
    );
    let IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(expected_document) } =
        expected_document
    else {
        unreachable!()
    };
    let mut expected_source = crate::tests::support::builders::estate_source(source.clone());
    expected_source.form = expected_document.form;
    let desired = vec![row];
    let mut request = save_documents(&mut session, desired.clone());
    if let Ok(path) = std::env::var("TAX_B11_ESTATE_CALC_INPUT") {
        let calculation = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(calculation, P::SaveDocuments { .. }));
        let before = session.draft.clone();
        let Reply::Response(reply) =
            crate::tests::support::client::income_pages::candidate_save_review(
                &mut session,
                calculation,
            )
        else {
            panic!("expected ordinary document calculation response")
        };
        assert_eq!(session.draft, before);
        std::fs::write(std::env::var("TAX_B11_ESTATE_CALC_OUTPUT").unwrap(), reply.to_avro())
            .unwrap();
        return;
    }
    for (name, value) in [("SEED", &seed), ("ANSWERS", &request)] {
        if let Ok(path) = std::env::var(alloc::format!("TAX_B11_ESTATE_{name}")) {
            std::fs::write(path, value.to_avro()).unwrap();
        }
    }
    if let Ok(path) = std::env::var("TAX_B11_ESTATE_PAYLOAD") {
        request = P::from_avro(&std::fs::read(path).unwrap()).unwrap();
        assert!(matches!(request, P::SaveDocuments { .. }));
    }
    accepted(&send(&mut session, request));
    let answers = crate::tests::support::client::income_pages::k1_fixture_answer_request(
        &mut session,
        &desired,
    )
    .unwrap();
    accepted(&send(&mut session, answers));
    let entry = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. }))
        .unwrap();
    assert!(entry.id > 0);
    assert_eq!(entry.owner, crate::DocumentOwner::from(owner));
    assert_eq!(entry.community, CommunityShare::Separate);
    assert_eq!(entry.income, FederalIncome::EstateTrustPortfolio { data: expected_source });
    let original = entry.clone();
    let projected = crate::community::record(&session.draft, entry);
    let FederalIncome::EstateTrustPortfolio { data } = projected.as_ref() else { unreachable!() };
    assert_eq!(
        crate::estate_trust::investment_adjustment(&FederalEstateTrustPortfolioK1::from(data)),
        Some([0; 2])
    );
    assert_eq!(data.form.interest, 50_000);
    assert_eq!(data.form.ordinary_dividends, 20_000);
    assert_eq!(data.form.qualified_dividends, 15_000);
    assert_eq!(
        (data.form.short_gain, data.form.long_gain),
        if owner == Owner::Taxpayer { (20_000, 100_000) } else { (0, 0) }
    );
    let report = supported(&session.draft);
    let reopened = open_document_center(&mut session);
    if let Ok(path) = std::env::var("TAX_B11_ESTATE_REOPEN") {
        std::fs::write(path, reopened.to_avro()).unwrap();
    }
    let P::SaveDocuments { documents, .. } = reopened else { unreachable!() };
    let crate::K1Document::K1EstateTrust(page) = &documents.common.k1.documents[0].document else {
        unreachable!()
    };
    assert_eq!(stored_k1_1041(&session).1, source);
    let saved = session
        .draft
        .income
        .iter()
        .find_map(|row| match &row.income {
            FederalIncome::EstateTrustPortfolio { data } => Some(data),
            _ => None,
        })
        .unwrap();
    assert_eq!(page, &crate::K1EstateTrustDocument::from_source(saved));
    let mut restarted = restart(session.draft.clone());
    assert!(restarted.draft.income.contains(&original));
    assert_eq!(supported(&restarted.draft), report);
    let P::SaveDocuments { documents, .. } = open_document_center(&mut restarted) else {
        unreachable!()
    };
    let crate::K1Document::K1EstateTrust(page) = &documents.common.k1.documents[0].document else {
        unreachable!()
    };
    assert_eq!(stored_k1_1041(&restarted).1, source);
    let saved = restarted
        .draft
        .income
        .iter()
        .find_map(|row| match &row.income {
            FederalIncome::EstateTrustPortfolio { data } => Some(data),
            _ => None,
        })
        .unwrap();
    assert_eq!(page, &crate::K1EstateTrustDocument::from_source(saved));
}

#[test]
fn unit27_estate_printed_correction_retains_activity_answer_until_actual_topic_fixes_it() {
    use crate::tests::cross::interview::open_topic;
    let mut native = empty_estate_portfolio();
    native.activities = crate::EstateActivities {
        box6: 100_000,
        box7: 0,
        box8: 0,
        activities: vec![crate::EstateActivity {
            id: 0,
            name: "Retained fiduciary activity".into(),
            kind: crate::EstateActivityKind::Business,
            income: 100_000,
            depreciation: 0,
            depletion: 0,
            amortization: 0,
            participation: crate::EstateParticipation::Material(ScorpParticipation {
                minutes: 30_001,
                ..participation_facts()
            }),
            carried: crate::EstatePassiveCarry { regular: 0, alternative: 0 },
            amt_difference: 0,
        }],
    };
    let original_participation = native.activities.activities[0].participation.clone();
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let desired = vec![document(
        0,
        IncomeSource::K1 {
            value: crate::K1Document::K1EstateTrust(estate_document(
                K1EstateTrust::from_record("Synthetic retained trust", &native).unwrap(),
            )),
        },
    )];
    accepted(&crate::tests::support::client::income_pages::save_source_fixtures(
        &mut session,
        desired,
    ));
    let baseline = crate::federal_rules::estimate(&session.draft);
    assert!(baseline.tax.is_some());
    let source = session
        .draft
        .income
        .iter()
        .find(|entry| matches!(entry.income, FederalIncome::EstateTrustPortfolio { .. }))
        .unwrap()
        .id;
    let activity_id =
        match &session.draft.income.iter().find(|entry| entry.id == source).unwrap().income {
            FederalIncome::EstateTrustPortfolio { data } => data.statements.activities[0].id,
            _ => unreachable!(),
        };
    assert!(activity_id > 0);
    let mut paper = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut paper else { unreachable!() };
    let row = documents.common.k1.documents.iter_mut().find(|row| row.id == source).unwrap();
    let crate::K1Document::K1EstateTrust(w) = &mut row.document else { unreachable!() };
    w.form.business_income = 0;
    w.form.rental_real_estate_income = 100_000;
    w.statements.activities[0].kind = crate::EstateActivityKind::RentalRealEstate;
    accepted(&send(&mut session, paper));
    let FederalIncome::EstateTrustPortfolio { data } =
        &session.draft.income.iter().find(|entry| entry.id == source).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.statements.activities[0].id, activity_id);
    assert_eq!(data.statements.activities[0].income, 100_000);
    assert_eq!(data.statements.activities[0].participation, original_participation);
    assert_eq!(data.statements.activities[0].carried.regular, 0);
    let blocked = crate::federal_rules::estimate(&session.draft);
    assert!(blocked.tax.is_none());
    assert!(
        blocked.issues.iter().any(|issue| issue.display.key == "ESTATE_ACTIVITY_PARTICIPATION")
    );
    assert!(
        crate::interview::remaining_items(&session.draft, &blocked, TaxSection::Review)
            .iter()
            .any(|row| row.section == TaxSection::PassThrough)
    );
    let topic = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { followups, .. } = &topic else { unreachable!() };
    let key = alloc::format!("income/{source}/estate/activity/{activity_id}");
    let row = followups.iter().find(|row| row.key == key).unwrap();
    let expected = SchemaPath::root()
        .field("followups")
        .key(row.key.as_str())
        .field("answer")
        .field("participation")
        .to_string();
    let before = session.draft.clone();
    let dirty = session.dirty;
    let Reply::Response(P::Problem { message, field }) = send(&mut session, topic) else {
        panic!("retained conflicting answer must be refused on its actual topic")
    };
    assert_eq!(crate::Message::decode(&message).key, "ESTATE_ACTIVITY_PARTICIPATION");
    assert_eq!(field.as_deref(), Some(expected.as_str()));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    let mut topic = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
    let row = followups.iter_mut().find(|row| row.key == key).unwrap();
    let crate::DocumentAnswer::EstateActivityRecords { participation, .. } = &mut row.answer else {
        unreachable!()
    };
    *participation = crate::EstateParticipation::Passive;
    accepted(&send(&mut session, topic));
    assert!(crate::federal_rules::estimate(&session.draft).tax.is_some());
    let mut paper = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut paper else { unreachable!() };
    let row = documents.common.k1.documents.iter_mut().find(|row| row.id == source).unwrap();
    let crate::K1Document::K1EstateTrust(w) = &mut row.document else { unreachable!() };
    w.form.business_income = 100_000;
    w.form.rental_real_estate_income = 0;
    w.statements.activities[0].kind = crate::EstateActivityKind::Business;
    accepted(&send(&mut session, paper));
    let mut topic = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
    let crate::DocumentAnswer::EstateActivityRecords { participation, .. } =
        &mut followups.iter_mut().find(|row| row.key == key).unwrap().answer
    else {
        unreachable!()
    };
    *participation = original_participation;
    accepted(&send(&mut session, topic));
    let restored = crate::federal_rules::estimate(&session.draft);
    assert_eq!(restored.tax, baseline.tax);
    assert_eq!(restored.return_lines, baseline.return_lines);
}

#[test]
fn unit27_estate_qbi_paper_saves_before_activity_links_and_topic_answers_survive_correction() {
    use crate::tests::cross::interview::open_topic;
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut paper = estate_document(
        K1EstateTrust::from_record("Synthetic QBI trust", &empty_estate_portfolio()).unwrap(),
    );
    assert!(paper.estate_activities.is_empty());
    paper.statements.qbi.push(crate::EstateQbiStatementPaper {
        id: 0,
        name: "Beneficiary business statement".into(),
        income: 100_000,
        wages: 0,
        property: 0,
        service: false,
        kind: crate::EstateQbiKind::Business,
    });
    let request = save_documents(
        &mut session,
        vec![document(0, IncomeSource::K1 { value: crate::K1Document::K1EstateTrust(paper) })],
    );
    accepted(&send(&mut session, request));
    let source_id = session.draft.income.last().unwrap().id;
    let mut correction = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
    let crate::K1Document::K1EstateTrust(paper) = &mut documents.common.k1.documents[0].document
    else {
        unreachable!()
    };
    assert_eq!(paper.statements.qbi[0].income, 100_000);
    let statement_id = paper.statements.qbi[0].id;
    assert!(statement_id > 0);
    paper.form.business_income = 100_000;
    paper.statements.activities.push(crate::EstateActivityPaper {
        id: 0,
        name: "Actual fiduciary business".into(),
        kind: crate::EstateActivityKind::Business,
        income: 100_000,
        depreciation: 0,
        depletion: 0,
        amortization: 0,
        amt_difference: 0,
    });
    accepted(&send(&mut session, correction));
    let mut topic = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { followups, .. } = &mut topic else { unreachable!() };
    let key = alloc::format!("income/{source_id}/estate/qbi/{statement_id}");
    let qbi = followups.iter_mut().find(|row| row.key == key).unwrap();
    assert_eq!(qbi.k1.estate_activities.len(), 1);
    let activity_id = qbi.k1.estate_activities[0].id;
    assert!(activity_id > 0);
    let crate::DocumentAnswer::EstateQbiHistory { activities, allocations, .. } = &mut qbi.answer
    else {
        unreachable!()
    };
    assert!(activities.is_empty());
    *activities = vec![activity_id];
    *allocations = vec![crate::EstateQbiAllocationHistory {
        activity: activity_id,
        income: 100_000,
        passive_history: Vec::new(),
    }];
    for row in followups {
        if let crate::DocumentAnswer::EstateActivityRecords { participation, .. } = &mut row.answer
        {
            *participation = crate::EstateParticipation::Passive;
        }
    }
    accepted(&send(&mut session, topic));
    let mut correction = open_document_center(&mut session);
    let P::SaveDocuments { documents, .. } = &mut correction else { unreachable!() };
    let crate::K1Document::K1EstateTrust(paper) = &mut documents.common.k1.documents[0].document
    else {
        unreachable!()
    };
    paper.statements.qbi[0].wages = 25_000;
    accepted(&send(&mut session, correction));
    let mut restarted = restart(session.draft.clone());
    let topic = open_topic(&mut restarted, TaxSection::PassThrough);
    let P::SaveIncomes { followups, .. } = topic else { unreachable!() };
    let qbi = followups.iter().find(|row| row.key == key).unwrap();
    let crate::DocumentAnswer::EstateQbiHistory { activities, allocations, .. } = &qbi.answer
    else {
        unreachable!()
    };
    assert_eq!(activities, &vec![activity_id]);
    assert_eq!(allocations.len(), 1);
    assert_eq!((allocations[0].activity, allocations[0].income), (activity_id, 100_000));
    let P::SaveDocuments { documents, .. } = open_document_center(&mut restarted) else {
        unreachable!()
    };
    let crate::K1Document::K1EstateTrust(paper) = &documents.common.k1.documents[0].document else {
        unreachable!()
    };
    assert_eq!((paper.statements.qbi[0].id, paper.statements.qbi[0].wages), (statement_id, 25_000));
}
