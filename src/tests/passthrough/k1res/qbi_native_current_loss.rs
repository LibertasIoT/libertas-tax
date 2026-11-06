use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{CurrentLossQbiFacts, documents};
use crate::tests::support::client::k1res::ordinary_multi_current_loss_qbi_session;
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::*;

#[test]
fn native_current_loss_uses_owner_allowance_and_retains_correction_lifecycle() {
    for scorp in [false, true] {
        let mut session =
            ordinary_multi_current_loss_qbi_session(scorp, &CurrentLossQbiFacts::default(), |_| {});
        let index = session
            .draft
            .income
            .iter()
            .position(|entry| crate::k1_activity::source(&entry.income).is_some())
            .unwrap();
        let id = session.draft.income[index].id;
        let source = crate::k1_activity::source(&session.draft.income[index].income).unwrap();
        let ids: Vec<_> = source.ids().collect();
        assert!(ids.iter().all(|id| *id > 0));
        let output = match &session.draft.income[index].income {
            FederalIncome::PartnershipActivities { data } => data.ordinary_source(false),
            FederalIncome::ScorpActivities { data } => data.ordinary_source(false),
            _ => unreachable!(),
        }
        .unwrap();
        assert_eq!(output.components[1].basis_allowed.get(), 200_000);
        assert_eq!(output.components[1].risk_allowed.get(), 200_000);
        assert_eq!(output.current_qualified[1].activity.income, -1_500);
        assert!(output.current_qualified[1].activity.active);
        assert_eq!(output.current_qualified[1].activity.service, Some(false));
        // $60,000 wages + $4,000 business profit - $2,000 allowed loss.
        // QBI is the independent $4,000 - $1,500 statement amounts: 20% = $500.
        let result = supported(&session.draft);
        assert_eq!(value(&result, "agi"), 62_000);
        assert_eq!(value(&result, "qbi_deduction"), 500);
        let line =
            alloc::format!("{}_{id}_schedule_e", if scorp { "scorp" } else { "partnership" });
        assert_eq!(line_amount(&result, &line), 200_000);
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

        let statement = match &mut documents(&mut request)[row].document {
            crate::K1Document::K1PartnershipActivities(page) => &mut page.activities.statement,
            crate::K1Document::K1ScorpActivities(page) => &mut page.activities.statement,
            _ => unreachable!(),
        };
        let K1ComponentStatementPage::Entered { source } = statement else { unreachable!() };
        let block = &mut source.qbi_blocks[1];
        match (&mut block.statement, &mut block.allocations.as_mut().unwrap()[0].qualified) {
            (
                K1ReportedQbiStatement::Partnership { statement: Some(statement) },
                K1ComponentQualified::Partnership { current_income, .. },
            ) => {
                assert_eq!(*current_income, -150_000);
                *current_income = -100_000;
                statement.income = -100_000;
            }
            (
                K1ReportedQbiStatement::Scorp { statement: Some(statement) },
                K1ComponentQualified::Scorp { current_income, ordinary, .. },
            ) => {
                assert_eq!(*current_income, 0);
                assert_eq!(ordinary.current, 150_000);
                ordinary.current = 100_000;
                statement.income = -100_000;
            }
            _ => unreachable!(),
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
        assert_eq!(
            crate::k1_activity::source(&session.draft.income[index].income)
                .unwrap()
                .ids()
                .collect::<Vec<_>>(),
            ids
        );
        // The corrected $1,000 qualified loss changes QBI alone: $3,000 x 20%.
        let corrected = supported(&session.draft);
        assert_eq!(value(&corrected, "agi"), 62_000);
        assert_eq!(value(&corrected, "qbi_deduction"), 600);
        let accepted_source = expected_input(&session.draft, &session.draft.income[index]);
        let mut cold = restart(session.draft.clone());
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(documents(&mut reopened)[row].document, accepted_source);
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(supported(&cold.draft), corrected);
        assert_eq!(
            crate::k1_activity::source(&cold.draft.income[index].income)
                .unwrap()
                .ids()
                .collect::<Vec<_>>(),
            ids
        );
    }
}

#[test]
fn native_no_statement_zero_qualification_preserves_known_suspended_losses() {
    for scorp in [false, true] {
        for basis_suspended in [true, false] {
            let facts = CurrentLossQbiFacts {
                qualified: [0, 0],
                gross_loss: if basis_suspended { 600_000 } else { 200_000 },
                opening_basis: if basis_suspended { 0 } else { 1_000_000 },
                loss_risk_capacity: if basis_suspended { 1_000_000 } else { 0 },
            };
            let session = ordinary_multi_current_loss_qbi_session(scorp, &facts, |_| {});
            let income = &session
                .draft
                .income
                .iter()
                .find(|entry| crate::k1_activity::source(&entry.income).is_some())
                .unwrap()
                .income;
            let source = crate::k1_activity::source(income).unwrap();
            assert!(source.qbi_blocks.iter().all(|block| matches!(
                block.statement,
                K1ReportedQbiStatement::Partnership { statement: None }
                    | K1ReportedQbiStatement::Scorp { statement: None }
            )));
            let output = match income {
                FederalIncome::PartnershipActivities { data } => data.ordinary_source(false),
                FederalIncome::ScorpActivities { data } => data.ordinary_source(false),
                _ => unreachable!(),
            }
            .unwrap();
            assert!(output.current_qualified.is_empty());
            let loss = &output.components[1];
            assert_eq!(loss.basis_suspended.get(), if basis_suspended { 200_000 } else { 0 });
            assert_eq!(loss.risk_suspended.get(), if basis_suspended { 0 } else { 200_000 });
            let result = supported(&session.draft);
            // Basis permits $4,000 of $6,000; at-risk permits none of $2,000.
            assert_eq!(value(&result, "agi"), if basis_suspended { 60_000 } else { 64_000 });
            assert_eq!(value(&result, "qbi_deduction"), 0);
            assert_eq!(supported(&restart(session.draft).draft), result);
        }
    }
}

#[test]
fn qualified_current_losses_keep_partial_or_unattributed_sources_unavailable() {
    for scorp in [false, true] {
        for boundary in 0..6 {
            let facts = CurrentLossQbiFacts {
                gross_loss: if boundary == 0 { 600_000 } else { 200_000 },
                opening_basis: if boundary == 0 { 0 } else { 1_000_000 },
                loss_risk_capacity: if boundary == 1 { 0 } else { 1_000_000 },
                ..CurrentLossQbiFacts::default()
            };
            let session = ordinary_multi_current_loss_qbi_session(scorp, &facts, |income| {
                let statement = match income {
                    FederalIncome::PartnershipActivities { data } => &mut data.activities,
                    FederalIncome::ScorpActivities { data } => &mut data.activities,
                    _ => unreachable!(),
                };
                let K1ComponentStatement::Entered { source } = statement else { unreachable!() };
                match boundary {
                    0 | 1 => {}
                    2 => source.components[1].qualified_current_controls.ordinary = None,
                    3 => {
                        source.components[1].qualified_current_controls.ordinary =
                            Some(K1CurrentSourceParts { income: 100_000, loss: 300_000 })
                    }
                    4 => {
                        source.passive_groups[1].participation =
                            K1ActivityParticipation::Unavailable
                    }
                    5 => {
                        let component = &mut source.components[1];
                        component.regular.as_mut().unwrap().operating_carry.basis = Some(100);
                        component.alternative = K1StatementAmt::SameAsRegular;
                        let qualified =
                            &mut source.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified;
                        match qualified {
                            K1ComponentQualified::Partnership { records, .. } => {
                                records.history.as_mut().unwrap().basis =
                                    vec![loss_year(2025, 1, 0, 1, 0)];
                            }
                            K1ComponentQualified::Scorp { ordinary, .. } => {
                                ordinary.history = vec![loss_year(2025, 1, 0, 1, 0)];
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            });
            let entry = session
                .draft
                .income
                .iter()
                .find(|entry| crate::k1_activity::source(&entry.income).is_some())
                .unwrap();
            let output = match &entry.income {
                FederalIncome::PartnershipActivities { data } => data.ordinary_source(false),
                FederalIncome::ScorpActivities { data } => data.ordinary_source(false),
                _ => unreachable!(),
            };
            assert!(
                matches!(output, Err(error) if error == ordinary::UNAVAILABLE),
                "scorp={scorp}, boundary={boundary}"
            );
            let result = estimate(&session.draft);
            assert!(result.tax.is_none(), "scorp={scorp}, boundary={boundary}");
            assert_eq!(
                result
                    .issues
                    .iter()
                    .filter(|issue| {
                        issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE"
                    })
                    .count(),
                1,
                "scorp={scorp}, boundary={boundary}"
            );
        }
    }
}
