use crate::k1_activity::*;
use crate::kernel::carry::Amount;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{
    documents, graph, input, ordinary_multi_current_retained_qbi_parent,
    partnership_qualified_history,
};
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{
    ordinary_multi_current_retained_qbi_session, partner_source,
};
use crate::tests::*;

#[test]
fn current_and_original_year_qbi_save_correct_preview_and_reopen_once() {
    let mut session = ordinary_multi_current_retained_qbi_session(
        ordinary_multi_current_retained_qbi_parent([400_000, 200_000], [Some((400_000, 0)), None]),
        6_000_000,
    );
    let id = session.draft.income[0].id;
    let output = partner_source(&session).ordinary_source(false).unwrap();
    assert_eq!(output.current_qualified.len(), 1);
    assert_eq!(output.current_qualified[0].activity.income, 4_000);
    assert_eq!(output.qualified.len(), 1);
    assert_eq!(output.qualified[0].released.get(), 130_000);
    assert!(output.qualified[0].carries.iter().all(|carry| {
        carry
            .remaining
            .iter()
            .all(|year| year.remaining.get() == 0 && year.remaining_qbi.get() == 0)
    }));
    let result = supported(&session.draft);
    // $6,000 current operations less $2,000 actual released losses; current
    // QBI $4,000 less originally qualified $500 + $800 gives $540, above minimum.
    assert_eq!(line_amount(&result, &alloc::format!("partnership_{id}_schedule_e")), 400_000);
    assert_eq!(value(&result, "agi"), 64_000);
    assert_eq!(value(&result, "qbi_deduction"), 540);
    let saved = session.draft.clone();
    let dirty = session.dirty;
    let native_ids: Vec<_> = partner_source(&session).activities.source().unwrap().ids().collect();
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);

    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    let source = graph(&mut documents(&mut request)[0].document);
    let history = partnership_qualified_history(
        &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified,
    );
    history.risk[0].qualified = 60_000;
    history.risk[0].remaining_qualified = 60_000;
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
    assert_eq!(session.draft.revision, saved.revision + 1);
    assert_eq!(session.draft.next_id, saved.next_id);
    assert_eq!(
        partner_source(&session).activities.source().unwrap().ids().collect::<Vec<_>>(),
        native_ids
    );
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "agi"), 64_000);
    assert_eq!(value(&corrected, "qbi_deduction"), 580);
    assert_eq!(
        partner_source(&session).ordinary_source(false).unwrap().qualified[0].released.get(),
        110_000
    );
    let accepted_draft = session.draft.clone();
    let mut cold = restart(accepted_draft.clone());
    let mut reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    assert_eq!(
        documents(&mut reopened)[0].document,
        expected_input(&accepted_draft, &accepted_draft.income[0])
    );
    accepted(&send_fixture(&mut cold, reopened));
    assert_eq!(cold.draft, accepted_draft);
    assert_eq!(supported(&cold.draft), corrected);
}

#[test]
fn partial_retained_release_keeps_original_percentages_and_current_active_minimum() {
    let mut session = ordinary_multi_current_retained_qbi_session(
        ordinary_multi_current_retained_qbi_parent([100_000, 200_000], [Some((90_000, 0)), None]),
        6_000_000,
    );
    let output = partner_source(&session).ordinary_source(false).unwrap();
    assert_eq!(output.components[0].basis_allowed.get(), 100_000);
    assert_eq!(output.components[0].risk_allowed.get(), 100_000);
    assert_eq!(output.components[0].risk_suspended.get(), 100_000);
    let release = &output.qualified[0];
    assert_eq!(release.released.get(), 80_000);
    assert_eq!(
        release.carries[0].remaining[0],
        crate::scorp_basis::LossVintage {
            year: 2025,
            original: Amount::new(100000).unwrap(),
            original_qbi: Amount::new(50000).unwrap(),
            remaining: Amount::new(0).unwrap(),
            remaining_qbi: Amount::new(0).unwrap(),
        }
    );
    assert_eq!(
        release.carries[1].remaining[0],
        crate::scorp_basis::LossVintage {
            year: 2024,
            original: Amount::new(100000).unwrap(),
            original_qbi: Amount::new(80000).unwrap(),
            remaining: Amount::new(0).unwrap(),
            remaining_qbi: Amount::new(0).unwrap(),
        }
    );
    assert_eq!(
        release.carries[1].remaining[1],
        crate::scorp_basis::LossVintage {
            year: 2026,
            original: Amount::new(100000).unwrap(),
            original_qbi: Amount::new(50000).unwrap(),
            remaining: Amount::new(100000).unwrap(),
            remaining_qbi: Amount::new(50000).unwrap(),
        }
    );
    let result = supported(&session.draft);
    assert_eq!(
        line_amount(
            &result,
            &alloc::format!("partnership_{}_schedule_e", session.draft.income[0].id)
        ),
        200_000
    );
    assert_eq!(value(&result, "agi"), 62_000);
    assert_eq!(value(&result, "qbi_deduction"), 20);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let source = graph(&mut documents(&mut request)[0].document);
    let K1ReportedQbiStatement::Partnership { statement: Some(statement) } =
        &mut source.qbi_blocks[0].statement
    else {
        unreachable!()
    };
    statement.income = 100_000;
    let K1ComponentQualified::Partnership { current_income, .. } =
        &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
    else {
        unreachable!()
    };
    *current_income = 100_000;
    accepted(&send_fixture(&mut session, request));
    // Active current QBI reaches $1,000; the inactive retained loss does not
    // remove the existing $400 minimum, even though ordinary 20% would be $40.
    assert_eq!(value(&supported(&session.draft), "qbi_deduction"), 400);
}

#[test]
fn current_sstb_eligibility_never_reclassifies_original_qualified_release() {
    let mut source = ordinary_multi_current_retained_qbi_parent(
        [400_000, 1_000_000],
        [Some((300_000, 200_000)), Some((1_000_000, 500_000))],
    );
    let K1ComponentStatement::Entered { source: native_graph } = &mut source.activities else {
        unreachable!()
    };
    let K1ReportedQbiStatement::Partnership { statement: Some(statement) } =
        &mut native_graph.qbi_blocks[0].statement
    else {
        unreachable!()
    };
    statement.service = true;
    let mut session = ordinary_multi_current_retained_qbi_session(source, 40_000_000);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 412_000);
    assert_eq!(value(&result, "qbi_deduction"), 1_740);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let source = graph(&mut documents(&mut request)[0].document);
    let K1ReportedQbiStatement::Partnership { statement: Some(statement) } =
        &mut source.qbi_blocks[0].statement
    else {
        unreachable!()
    };
    statement.service = false;
    accepted(&send_fixture(&mut session, request));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 412_000);
    assert_eq!(value(&result, "qbi_deduction"), 2_340);
    assert_eq!(
        partner_source(&session).ordinary_source(false).unwrap().qualified[0].released.get(),
        130_000
    );
}

#[test]
fn valid_unavailable_current_retained_sources_save_without_becoming_zero() {
    let baseline = ordinary_multi_current_retained_qbi_session(
        ordinary_multi_current_retained_qbi_parent([400_000, 200_000], [Some((400_000, 0)), None]),
        6_000_000,
    );
    for boundary in 0..7 {
        let mut value = partner_source(&baseline).clone();
        let K1ComponentStatement::Entered { source } = &mut value.activities else {
            unreachable!()
        };
        match boundary {
            0 => source.components[0].qualified_current_controls.ordinary = None,
            1 => {
                let K1ComponentQualified::Partnership { records, .. } =
                    &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
                else {
                    unreachable!()
                };
                records.history = None;
            }
            2 => source.components[0].regular.as_mut().unwrap().operating_carry.basis = None,
            3 => {
                source.components[0].regular.as_mut().unwrap().passive_operating = Some(10_000);
                partnership_qualified_history(
                    &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified,
                )
                .passive = vec![loss_year(2025, 100, 0, 100, 0)];
            }
            4 => value.person.as_mut().unwrap().reported_se = Some(100),
            5 => {
                source.components[0].qualified_current_controls.ordinary =
                    Some(K1CurrentSourceParts { income: 500_000, loss: 100_000 })
            }
            6 => source.passive_groups[0].participation = K1ActivityParticipation::Unavailable,
            _ => unreachable!(),
        }
        assert_eq!(value.rule(), None, "source-valid boundary {boundary}");
        assert!(
            matches!(value.ordinary_source(false), Err(error) if error == ordinary::UNAVAILABLE),
            "boundary {boundary}"
        );
        let mut session = restart(baseline.draft.clone());
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document =
            input(&FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) });
        accepted(&send_fixture(&mut session, request));
        let result = crate::federal_rules::estimate(&session.draft);
        assert!(result.tax.is_none(), "boundary {boundary}");
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
        );
    }
}

#[test]
fn invalid_current_retained_edits_refuse_atomically_at_native_save() {
    let mut session = ordinary_multi_current_retained_qbi_session(
        ordinary_multi_current_retained_qbi_parent([400_000, 200_000], [Some((400_000, 0)), None]),
        6_000_000,
    );
    for boundary in 0..5 {
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let source = graph(&mut documents(&mut request)[0].document);
        match boundary {
            0 => {
                let K1ReportedQbiStatement::Partnership { statement: Some(statement) } =
                    &mut source.qbi_blocks[0].statement
                else {
                    unreachable!()
                };
                statement.income -= 100;
            }
            1 => {
                partnership_qualified_history(
                    &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified,
                )
                .basis[0]
                    .qualified = 100_001
            }
            2 => {
                partnership_qualified_history(
                    &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified,
                )
                .basis[0]
                    .remaining -= 100
            }
            3 => source.components[1].id = source.components[0].id,
            4 => {
                source.qbi_blocks[0].allocations.as_mut().unwrap()[0].component =
                    session.draft.next_id + 99
            }
            _ => unreachable!(),
        }
        let before = session.draft.clone();
        let dirty = session.dirty;
        let request = P::from_avro(&request.to_avro()).unwrap();
        assert!(
            matches!(send_fixture(&mut session, request), Reply::Response(P::Problem { .. })),
            "boundary {boundary}"
        );
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
    }
}

#[test]
fn current_qualified_income_keeps_exact_retained_years_through_fifo() {
    let mut parent =
        ordinary_multi_current_retained_qbi_parent([400_000, 200_000], [Some((400_000, 0)), None]);
    let K1ComponentStatement::Entered { source } = &mut parent.activities else { unreachable!() };
    // Form 8995 loss tracking preserves the source years and exact cents;
    // rounding each original-year row does not set the basis-stage capacity.
    partnership_qualified_history(
        &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified,
    )
    .basis = [(2023, 33_350, 10_000), (2024, 33_350, 20_000), (2025, 33_300, 20_000)]
        .into_iter()
        .map(|(year, original, qualified)| ScorpLossYear {
            year,
            original,
            qualified,
            remaining: original,
            remaining_qualified: qualified,
        })
        .collect();
    assert_eq!(parent.rule(), None);
    let mut session = ordinary_multi_current_retained_qbi_session(parent, 6_000_000);
    let output = partner_source(&session).ordinary_source(false).unwrap();
    assert_eq!(output.current_qualified[0].activity.income, 4_000);
    let release = &output.qualified[0];
    assert_eq!(release.carries[0].amount.get(), 100_000);
    assert_eq!(release.carries[0].qbi.get(), 50_000);
    assert_eq!(release.released.get(), 130_000);
    assert_eq!(
        release.carries[0].remaining.iter().map(|v| v.year).collect::<Vec<_>>(),
        vec![2023, 2024, 2025]
    );
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 64_000);
    assert_eq!(value(&result, "qbi_deduction"), 540);
    let accepted_draft = session.draft.clone();
    let request = crate::tests::support::client::income_pages::open_document_center(&mut session);
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            crate::tests::support::client::estimate_documents_from_save(&request)
        ),
        Reply::Response(P::Review { .. })
    ));
    assert_eq!(session.draft, accepted_draft);
    let mut cold = restart(accepted_draft);
    let reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    accepted(&send_fixture(&mut cold, reopened));
    assert_eq!(supported(&cold.draft), result);
}
