use crate::k1_activity::*;
use crate::tests::cross::interview::{accepted, open_topic};
use crate::tests::support::builders::k1res::{graph, qbi_aggregation_facts};
use crate::tests::support::client::income_pages::page_documents;
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{
    ordinary_multi_current_qbi_session_with_statements,
    ordinary_multi_current_scorp_qbi_session_with_statements,
};
use crate::tests::*;

#[test]
fn native_block_aggregation_uses_actual_wages_and_preserves_native_lifecycle() {
    for build in [
        ordinary_multi_current_qbi_session_with_statements
            as fn(FederalDraft, [Option<(i64, i64)>; 2]) -> Interview,
        ordinary_multi_current_scorp_qbi_session_with_statements,
    ] {
        let mut draft = complete(FederalStatus::Single);
        draft.income = vec![wage(1, Owner::Taxpayer, 40_000_000, 0)];
        let mut session = build(draft, [Some((300_000, 0)), Some((200_000, 10_000_000))]);
        let ungrouped = supported(&session.draft);
        // Above full phaseout: A has $3,000 QBI/no wages -> $0;
        // B has $2,000 QBI/$100,000 wages -> $400. Combined: $5,000 x20%=$1,000.
        assert_eq!(value(&ungrouped, "agi"), 406_000);
        assert_eq!(value(&ungrouped, "qbi_deduction"), 400);
        let entry = session
            .draft
            .income
            .iter()
            .find(|e| crate::k1_activity::source(&e.income).is_some())
            .unwrap();
        let parent = entry.id;
        let ids: Vec<_> = crate::k1_activity::source(&entry.income).unwrap().ids().collect();
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        let mut facts = qbi_aggregation_facts();
        for row in &mut documents.common.k1.documents {
            for block in &mut graph(&mut row.document).qbi_blocks {
                block.qbi_group = facts.facts.name.clone();
            }
        }
        accepted(&send_fixture(&mut session, request));
        let before = session.draft.clone();
        let dirty = session.dirty;
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        let P::SaveIncomes { documents, .. } = &mut request else { unreachable!() };
        documents.qbi_aggregations = vec![facts.clone()];
        let reply = crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            crate::tests::support::client::estimate_documents_from_save(&request),
        );
        let Reply::Response(P::IncomeRunningEstimate { groups, sources, .. }) = reply else {
            panic!("native aggregation calculation: {reply:?}")
        };
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].businesses, Some(2));
        assert_eq!(groups[0].members.len(), 2);
        assert!(groups[0].members.iter().all(|m| matches!(m.source,
            QbiAggregationSource::K1Block { document, block } if document == parent && ids.contains(&block))));
        facts.members = groups[0].members.clone();
        facts.disclosure_sources = groups[0].disclosure_sources.clone();
        facts.businesses = groups[0].businesses;
        facts.facts.disclosure = facts
            .members
            .iter()
            .map(|m| QbiBusinessDisclosurePage {
                source: m.key.clone(),
                entity_name: "Synthetic source-owned business".into(),
                ein: QbiEntityEin::Known { number: "123456789".into() },
                business_description: "Ordinary business on the actual statement".into(),
                changes: QbiBusinessChanges::None,
            })
            .collect();
        let P::SaveIncomes { documents, qbi_sources, .. } = &mut request else { unreachable!() };
        documents.qbi_aggregations = vec![facts];
        *qbi_sources = sources;

        assert!(matches!(
            crate::tests::support::client::income_pages::candidate_save_review(
                &mut session,
                crate::tests::support::client::estimate_documents_from_save(&request)
            ),
            Reply::Response(P::IncomeRunningEstimate { .. })
        ));
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        accepted(&send_fixture(&mut session, request));
        assert_eq!(session.draft.revision, before.revision + 1);
        assert_eq!(session.draft.next_id, before.next_id);
        assert_eq!(
            session.draft.income.iter().find(|e| e.id == 1),
            before.income.iter().find(|e| e.id == 1)
        );
        let actual = session.draft.income.iter().find(|e| e.id == parent).unwrap();
        assert_eq!(
            crate::k1_activity::source(&actual.income).unwrap().ids().collect::<Vec<_>>(),
            ids
        );
        let grouped = supported(&session.draft);
        assert_eq!(value(&grouped, "agi"), 406_000);
        assert_eq!(value(&grouped, "qbi_deduction"), 1_000);
        let accepted_draft = session.draft.clone();
        let accepted_dirty = session.dirty;
        let reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let pages = page_documents(&reopened);
        assert_eq!(session.draft, accepted_draft);
        assert_eq!(session.dirty, accepted_dirty);
        accepted(&send_fixture(&mut session, reopened));
        assert_eq!(session.draft, accepted_draft);
        let mut cold = restart(accepted_draft.clone());
        let reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(page_documents(&reopened), pages);
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(cold.draft, accepted_draft);
        assert_eq!(supported(&cold.draft), grouped);
    }
}

#[test]
fn native_block_catalog_preserves_source_counts_independently_of_financial_support() {
    let session = ordinary_multi_current_qbi_session_with_statements(
        complete(FederalStatus::Single),
        [Some((300_000, 0)), Some((200_000, 0))],
    );
    for (names, expected) in [
        (None, None),
        (Some(vec![]), Some(0)),
        (
            Some(vec![
                K1QbiBusinessName { name: "Actual A".into() },
                K1QbiBusinessName { name: "Actual B".into() },
            ]),
            Some(2),
        ),
    ] {
        let mut draft = session.draft.clone();
        let entry = draft
            .income
            .iter_mut()
            .find(|e| crate::k1_activity::source(&e.income).is_some())
            .unwrap();
        let parent = entry.id;
        let FederalIncome::PartnershipActivities { data } = &mut entry.income else {
            unreachable!()
        };
        let K1ComponentStatement::Entered { source } = &mut data.activities else { unreachable!() };
        let block_id = source.qbi_blocks[0].id;
        source.qbi_blocks[0].reported_aggregate = true;
        source.qbi_blocks[0].business_names = names;
        assert!(
            matches!(data.ordinary_source(false), Err(error) if error == ordinary::UNAVAILABLE)
        );
        let catalog = crate::qbi_aggregation::source_views(&draft, &draft);
        let actual = catalog
            .iter()
            .find(|s| {
                s.member.source
                    == QbiAggregationSource::K1Block { document: parent, block: block_id }
            })
            .unwrap();
        assert_eq!(actual.businesses, expected);
        assert!(matches!(actual.member.origin, QbiSourceOrigin::ReportedRpeAggregate { .. }));
        assert_eq!(catalog.len(), 2);
    }
}

#[test]
fn native_block_new_inner_identity_is_not_selectable_until_normal_save() {
    for build in [
        ordinary_multi_current_qbi_session_with_statements
            as fn(FederalDraft, [Option<(i64, i64)>; 2]) -> Interview,
        ordinary_multi_current_scorp_qbi_session_with_statements,
    ] {
        let mut session =
            build(complete(FederalStatus::Single), [Some((300_000, 0)), Some((200_000, 0))]);
        let before = session.draft.clone();
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
        let source = graph(&mut documents.common.k1.documents[0].document);
        let mut added = source.qbi_blocks[0].clone();
        added.id = 0;
        added.name = "New actual statement".into();
        added.business_names = Some(vec![]);
        added.allocations = Some(vec![]);
        added.statement = match added.statement {
            K1ReportedQbiStatement::Partnership { .. } => {
                K1ReportedQbiStatement::Partnership { statement: None }
            }
            K1ReportedQbiStatement::Scorp { .. } => {
                K1ReportedQbiStatement::Scorp { statement: None }
            }
            _ => unreachable!(),
        };
        source.qbi_blocks.push(added);
        let reply = crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            crate::tests::support::client::estimate_documents_from_save(&request),
        );
        assert!(
            matches!(reply, Reply::Response(P::Review { .. })),
            "new inner source calculation: {reply:?}"
        );
        let P::SaveIncomes { qbi_sources: sources, .. } =
            open_topic(&mut session, TaxSection::PassThrough)
        else {
            unreachable!()
        };
        assert_eq!(sources.len(), 2, "tentative positive identity must stay out of choices");
        assert_eq!(session.draft, before);
        accepted(&send_fixture(&mut session, request));
        let catalog = crate::qbi_aggregation::source_views(&session.draft, &session.draft);
        assert_eq!(catalog.len(), 3);
        let added = catalog
            .iter()
            .find(|s| !sources.iter().any(|old| old.source == s.member.source))
            .unwrap();
        assert_eq!(added.businesses, Some(0));
        assert!(
            matches!(added.member.source, QbiAggregationSource::K1Block { block, .. } if block > 0)
        );
    }
}

#[test]
fn native_block_history_remains_associated_when_labels_or_financial_support_change() {
    let session = ordinary_multi_current_qbi_session_with_statements(
        complete(FederalStatus::Single),
        [Some((300_000, 0)), Some((200_000, 0))],
    );
    for history in [
        QbiIrsDisaggregationHistory::Unavailable,
        QbiIrsDisaggregationHistory::Disaggregated { year: 2025 },
    ] {
        let mut previous = session.draft.clone();
        let mut facts = qbi_aggregation_facts();
        facts.facts.election = QbiElectionHistory::Continuing { original_year: 2024 };
        facts.facts.irs_history = history;
        let entry = previous
            .income
            .iter_mut()
            .find(|e| crate::k1_activity::source(&e.income).is_some())
            .unwrap();
        let FederalIncome::PartnershipActivities { data } = &mut entry.income else {
            unreachable!()
        };
        let K1ComponentStatement::Entered { source } = &mut data.activities else { unreachable!() };
        for block in &mut source.qbi_blocks {
            block.qbi_aggregation_group = Some(facts.facts.name.clone());
        }
        let sources = crate::qbi_aggregation::source_views(&previous, &previous);
        crate::qbi_aggregation::refresh_derived(core::slice::from_mut(&mut facts), &sources);
        // Keep actual typed disclosure links while testing retained history separately
        // from financial eligibility and current-year disclosure completeness.
        facts.facts.disclosure = sources
            .iter()
            .map(|s| QbiBusinessDisclosurePage {
                source: s.member.key.clone(),
                entity_name: "Actual source".into(),
                ein: QbiEntityEin::Known { number: "123456789".into() },
                business_description: "Source-owned ordinary business".into(),
                changes: QbiBusinessChanges::None,
            })
            .collect();
        previous.screening.as_mut().unwrap().qbi_aggregations =
            crate::qbi_aggregation::accept_pages(&[facts], &sources).unwrap();
        assert_eq!(
            crate::qbi_aggregation::computation_issue(&previous),
            Some("QBI_AGGREGATION_FACTS")
        );
        for group in [None, Some("Renamed actual group".into())] {
            let mut candidate = previous.clone();
            candidate.screening.as_mut().unwrap().qbi_aggregations.clear();
            let entry = candidate
                .income
                .iter_mut()
                .find(|e| crate::k1_activity::source(&e.income).is_some())
                .unwrap();
            let FederalIncome::PartnershipActivities { data } = &mut entry.income else {
                unreachable!()
            };
            let K1ComponentStatement::Entered { source } = &mut data.activities else {
                unreachable!()
            };
            for block in &mut source.qbi_blocks {
                block.qbi_aggregation_group = group.clone();
                block.reported_aggregate = true;
                block.business_names = None;
            }
            assert_eq!(
                crate::qbi_aggregation::history_transition_rule(&previous, &candidate),
                Some("QBI_AGGREGATION_HISTORY")
            );
            assert_eq!(crate::qbi_aggregation::source_views(&candidate, &previous).len(), 2);
        }
    }
}
