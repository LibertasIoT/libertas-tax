use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{
    documents, ordinary_multi_passive_qbi_parent, ordinary_multi_passive_scorp_qbi_parent,
};
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{
    ordinary_multi_passive_qbi_session, ordinary_multi_passive_scorp_qbi_session,
};
use crate::tests::*;

#[test]
fn passive_current_qbi_keeps_native_income_and_correction_lifecycle() {
    for (scorp, mut session) in [
        (false, ordinary_multi_passive_qbi_session(150_000, [false; 2])),
        (true, ordinary_multi_passive_scorp_qbi_session(150_000, [false; 2])),
    ] {
        let index =
            session.draft.income.iter().position(|entry| source(&entry.income).is_some()).unwrap();
        let id = session.draft.income[index].id;
        let native_ids = crate::k1_activity::source(&session.draft.income[index].income)
            .unwrap()
            .ids()
            .collect::<Vec<_>>();
        let result = supported(&session.draft);
        let prefix = if scorp { "scorp" } else { "partnership" };
        assert_eq!(line_amount(&result, &alloc::format!("{prefix}_{id}_schedule_e")), 600_000);
        assert_eq!(value(&result, "agi"), 66_000);
        // $1,500 qualified income x 20% = $300. Passive income earns no $400 minimum.
        assert_eq!(value(&result, "qbi_deduction"), 300);
        let saved = session.draft.clone();
        let dirty = session.dirty;
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let row = documents(&mut request)
            .iter()
            .position(|row| {
                matches!(
                    row.document,
                    crate::K1Document::K1PartnershipActivities(_)
                        | crate::K1Document::K1ScorpActivities(_)
                )
            })
            .unwrap();

        assert_eq!(session.draft, saved);
        assert_eq!(session.dirty, dirty);
        let statement_page = match &mut documents(&mut request)[row].document {
            crate::K1Document::K1PartnershipActivities(page) => &mut page.activities.statement,
            crate::K1Document::K1ScorpActivities(page) => &mut page.activities.statement,
            _ => unreachable!(),
        };
        let K1ComponentStatementPage::Entered { source } = statement_page else { unreachable!() };
        let block = source
            .qbi_blocks
            .iter_mut()
            .find(|block| {
                matches!(
                    block.statement,
                    K1ReportedQbiStatement::Partnership { statement: Some(_) }
                        | K1ReportedQbiStatement::Scorp { statement: Some(_) }
                )
            })
            .unwrap();
        match &mut block.statement {
            K1ReportedQbiStatement::Partnership { statement: Some(statement) } => {
                statement.income = 80_000
            }
            K1ReportedQbiStatement::Scorp { statement: Some(statement) } => {
                statement.income = 80_000
            }
            _ => unreachable!(),
        }
        let [allocation] = block.allocations.as_deref_mut().unwrap() else { unreachable!() };
        match &mut allocation.qualified {
            K1ComponentQualified::Partnership { current_income, .. }
            | K1ComponentQualified::Scorp { current_income, .. } => *current_income = 80_000,
            K1ComponentQualified::Unavailable => {
                unreachable!("fixture supplies qualified statement")
            }
        }
        assert!(matches!(
            crate::tests::support::client::income_pages::candidate_save_review(
                &mut session,
                crate::tests::support::client::estimate_documents_from_save(&request)
            ),
            Reply::Response(P::Review { .. })
        ));
        assert_eq!(session.draft, saved);
        assert_eq!(session.dirty, dirty);
        accepted(&send_fixture(&mut session, request));
        assert_eq!(session.draft.revision, saved.revision + 2);
        assert_eq!(session.draft.next_id, saved.next_id);
        assert_eq!(session.draft.income[index].id, id);
        assert_eq!(
            crate::k1_activity::source(&session.draft.income[index].income)
                .unwrap()
                .ids()
                .collect::<Vec<_>>(),
            native_ids
        );
        let corrected = supported(&session.draft);
        assert_eq!(value(&corrected, "agi"), 66_000);
        assert_eq!(value(&corrected, "qbi_deduction"), 160);
        let accepted_source = expected_input(&session.draft, &session.draft.income[index]);
        let mut cold = restart(session.draft.clone());
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(documents(&mut reopened)[row].document, accepted_source);
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(cold.draft.revision, session.draft.revision);
        assert_eq!(
            crate::k1_activity::source(&cold.draft.income[index].income)
                .unwrap()
                .ids()
                .collect::<Vec<_>>(),
            native_ids
        );
        assert_eq!(supported(&cold.draft), corrected);
    }
}

#[test]
fn passive_current_qbi_minimum_uses_qualified_active_income_only() {
    for active in [[true; 2], [false, true]] {
        for session in [
            ordinary_multi_passive_qbi_session(150_000, active),
            ordinary_multi_passive_scorp_qbi_session(150_000, active),
        ] {
            let entry =
                session.draft.income.iter().find(|entry| source(&entry.income).is_some()).unwrap();
            for alternative in [false, true] {
                let output = match &entry.income {
                    FederalIncome::PartnershipActivities { data } => {
                        data.ordinary_source(alternative)
                    }
                    FederalIncome::ScorpActivities { data } => data.ordinary_source(alternative),
                    _ => unreachable!(),
                }
                .unwrap();
                // A owns $4,000 operating income and B owns $2,000. Only A has QBI.
                assert_eq!(
                    output.groups.iter().map(|group| group.passive).sum::<i64>(),
                    if active[0] { 0 } else { 400_000 }
                );
                assert_eq!(
                    output.groups.iter().map(|group| group.nonpassive).sum::<i64>(),
                    if active[0] { 600_000 } else { 200_000 }
                );
                if !alternative {
                    assert_eq!(output.current_qualified.len(), 1);
                    assert_eq!(output.current_qualified[0].activity.income, 1_500);
                    assert_eq!(output.current_qualified[0].activity.active, active[0]);
                }
            }
            let result = supported(&session.draft);
            assert_eq!(value(&result, "agi"), 66_000);
            // Active B with no statement cannot supply active qualified income for the minimum.
            assert_eq!(value(&result, "qbi_deduction"), if active[0] { 400 } else { 300 });
        }
    }
}

#[test]
fn passive_current_qbi_needs_known_participation_and_available_carries() {
    for boundary in 0..3 {
        for mut income in [
            FederalIncome::PartnershipActivities {
                data: alloc::boxed::Box::new(ordinary_multi_passive_qbi_parent(
                    150_000, [false; 2],
                )),
            },
            FederalIncome::ScorpActivities {
                data: ordinary_multi_passive_scorp_qbi_parent(150_000, [false; 2]),
            },
        ] {
            let statement = match &mut income {
                FederalIncome::PartnershipActivities { data } => &mut data.activities,
                FederalIncome::ScorpActivities { data } => &mut data.activities,
                _ => unreachable!(),
            };
            let K1ComponentStatement::Entered { source } = statement else { unreachable!() };
            match boundary {
                0 => source.passive_groups[0].participation = K1ActivityParticipation::Unavailable,
                1 => source.components[0].regular.as_mut().unwrap().passive_operating = None,
                2 => {
                    // 2026-10-20: retained losses with actual history are supported;
                    // missing carry facts remain unavailable under the source contract.
                    source.components[0].regular.as_mut().unwrap().operating_carry.risk = None
                }
                _ => unreachable!(),
            }
            for alternative in [false, true] {
                let output = match &income {
                    FederalIncome::PartnershipActivities { data } => {
                        data.ordinary_source(alternative)
                    }
                    FederalIncome::ScorpActivities { data } => data.ordinary_source(alternative),
                    _ => unreachable!(),
                };
                assert!(
                    matches!(output, Err(error) if error == ordinary::UNAVAILABLE),
                    "boundary {boundary}"
                );
            }
        }
    }
}
