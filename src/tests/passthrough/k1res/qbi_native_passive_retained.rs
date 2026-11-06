use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{
    documents, graph, input, ordinary_multi_passive_retained_qbi_parent, partner_activity_graph,
    partner_group_material, partner_retained_opening_risk, partnership_qualified_history,
};
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{
    ordinary_multi_current_retained_qbi_session, partner_source,
};
use crate::tests::*;

#[test]
fn passive_retained_qbi_corrects_original_qualification_through_native_lifecycle() {
    let mut session = ordinary_multi_current_retained_qbi_session(
        ordinary_multi_passive_retained_qbi_parent([400_000, 200_000], [Some((400_000, 0)), None]),
        6_000_000,
    );
    let entry = &session.draft.income[0];
    assert_eq!(income_issue(&entry.income), None);
    let id = entry.id;
    let output = partner_source(&session).ordinary_source(false).unwrap();
    assert!(output.groups.iter().all(|group| group.passive >= 0));
    assert_eq!(output.current_qualified.len(), 1);
    assert!(!output.current_qualified[0].activity.active);
    assert_eq!(output.qualified[0].released.get(), 130_000);
    let result = supported(&session.draft);
    // $6,000 actual income - $2,000 tax losses = $4,000 Schedule E.
    // ($4,000 current QBI - $500/$800 original qualified losses) * 20% = $540.
    assert_eq!(line_amount(&result, &alloc::format!("partnership_{id}_schedule_e")), 400_000);
    assert_eq!(value(&result, "agi"), 64_000);
    assert_eq!(value(&result, "qbi_deduction"), 540);
    let saved = session.draft.clone();
    let dirty = session.dirty;
    let ids = source(&saved.income[0].income).unwrap().ids().collect::<Vec<_>>();
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);

    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    let native = graph(&mut documents(&mut request)[0].document);
    let history = partnership_qualified_history(
        &mut native.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified,
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
    assert_eq!(session.draft.income[0].id, id);
    assert_eq!(source(&session.draft.income[0].income).unwrap().ids().collect::<Vec<_>>(), ids);
    let corrected = supported(&session.draft);
    assert_eq!(line_amount(&corrected, &alloc::format!("partnership_{id}_schedule_e")), 400_000);
    assert_eq!(value(&corrected, "agi"), 64_000);
    assert_eq!(value(&corrected, "qbi_deduction"), 580);
    assert_eq!(
        partner_source(&session).ordinary_source(false).unwrap().qualified[0].released.get(),
        110_000
    );
    let saved = session.draft.clone();
    let mut cold = restart(saved.clone());
    let mut reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    assert_eq!(documents(&mut reopened)[0].document, expected_input(&saved, &saved.income[0]));
    accepted(&send_fixture(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
    assert_eq!(supported(&cold.draft), corrected);
}

#[test]
fn zero_passive_group_keeps_partial_fifo_and_only_material_income_earns_minimum() {
    for material in [false, true] {
        for (qualified, expected) in [(90_000, 20), (100_000, if material { 400 } else { 40 })] {
            let mut native = ordinary_multi_passive_retained_qbi_parent(
                [100_000, 200_000],
                [Some((qualified, 0)), None],
            );
            partner_group_material(partner_activity_graph(&mut native), material);
            let session = ordinary_multi_current_retained_qbi_session(native, 6_000_000);
            assert_eq!(income_issue(&session.draft.income[0].income), None);
            let output = partner_source(&session).ordinary_source(false).unwrap();
            assert_eq!(output.groups[0].current, 0);
            assert_eq!(output.components[0].basis_allowed.get(), 100_000);
            assert_eq!(output.components[0].risk_allowed.get(), 100_000);
            assert_eq!(output.components[0].risk_suspended.get(), 100_000);
            assert_eq!(output.current_qualified[0].activity.active, material);
            let release = &output.qualified[0];
            assert_eq!(release.released.get(), 80_000);
            assert!(release.carries[0].remaining.iter().all(|row| row.remaining.get() == 0));
            assert_eq!(release.carries[1].remaining[0].remaining.get(), 0);
            let carried = &release.carries[1].remaining[1];
            assert_eq!(
                (carried.year, carried.original.get(), carried.original_qbi.get()),
                (2026, 100_000, 50_000)
            );
            assert_eq!((carried.remaining.get(), carried.remaining_qbi.get()), (100_000, 50_000));
            let result = supported(&session.draft);
            assert_eq!(
                line_amount(
                    &result,
                    &alloc::format!("partnership_{}_schedule_e", session.draft.income[0].id)
                ),
                200_000
            );
            assert_eq!(value(&result, "agi"), 62_000);
            // $900 - $800 => $20, or $1,000 - $800 => $40. Only the
            // latter's current material income reaches the $1,000/$400 minimum.
            assert_eq!(value(&result, "qbi_deduction"), expected);
        }
    }
}

#[test]
fn shared_positive_group_admits_current_only_passive_companion_in_retained_cohort() {
    let mut session = ordinary_multi_current_retained_qbi_session(
        ordinary_multi_passive_retained_qbi_parent(
            [100_000, 300_000],
            [Some((100_000, 0)), Some((300_000, 0))],
        ),
        6_000_000,
    );
    let mut changed = partner_source(&session).clone();
    let native = partner_activity_graph(&mut changed);
    let group = native.passive_groups[0].id;
    native.components[1].passive_group = Some(group);
    native.passive_groups.pop();
    let K1RiskSource::Partnership { regular, alternative } = &mut native.risk_groups[0].source
    else {
        unreachable!()
    };
    regular.start = ScorpRiskStart::Detailed { capacity: 200_000, profit: 0, loss: 0 };
    *alternative = Some(regular.clone());
    let income = FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(changed) };
    assert_eq!(income_issue(&income), None);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    documents(&mut request)[0].document = input(&income);
    accepted(&send_fixture(&mut session, request));
    assert_eq!(income_issue(&session.draft.income[0].income), None);
    let output = partner_source(&session).ordinary_source(false).unwrap();
    assert_eq!(output.groups.len(), 1);
    assert_eq!(output.groups[0].passive, 200_000);
    assert_eq!(
        output.components[0].income.get() - output.components[0].risk_allowed.get(),
        -100_000
    );
    assert_eq!(output.current_qualified.len(), 2);
    assert!(output.current_qualified.iter().all(|activity| !activity.activity.active));
    assert_eq!(output.qualified[0].released.get(), 130_000);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 62_000);
    assert_eq!(value(&result, "qbi_deduction"), 540);
}

#[test]
fn zero_qualified_negative_passive_group_keeps_existing_known_ordinary_output() {
    let mut native = ordinary_multi_passive_retained_qbi_parent([100_000, 200_000], [None, None]);
    let graph = partner_activity_graph(&mut native);
    partner_retained_opening_risk(graph);
    let history = partnership_qualified_history(
        &mut graph.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified,
    );
    for row in history.basis.iter_mut().chain(&mut history.risk) {
        row.qualified = 0;
        row.remaining_qualified = 0;
    }
    let income =
        FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(native.clone()) };
    assert_eq!(income_issue(&income), None);
    let output = native.ordinary_source(false).unwrap();
    assert_eq!(output.groups[0].passive, -100_000);
    assert_eq!(output.groups[1].passive, 200_000);
    assert!(output.current_qualified.is_empty());
    assert!(output.qualified.is_empty());
    let session = ordinary_multi_current_retained_qbi_session(native, 6_000_000);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 61_000);
    assert_eq!(value(&result, "qbi_deduction"), 0);
}

#[test]
fn independent_negative_amt_group_stays_known_without_qbi_outputs() {
    let mut native =
        ordinary_multi_passive_retained_qbi_parent([400_000, 200_000], [Some((400_000, 0)), None]);
    native.alternative_current.as_mut().unwrap().ordinary = 300_000;
    let graph = partner_activity_graph(&mut native);
    let mut amounts = graph.components[0].regular.as_ref().unwrap().clone();
    amounts.operating = 100_000;
    graph.components[0].alternative = K1StatementAmt::Separate { amounts };
    let K1RiskSource::Partnership { alternative: Some(alternative), .. } =
        &mut graph.risk_groups[0].source
    else {
        unreachable!()
    };
    alternative.start = ScorpRiskStart::Detailed { capacity: 200_000, profit: 0, loss: 0 };
    let income =
        FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(native.clone()) };
    assert_eq!(income_issue(&income), None);
    let regular = native.ordinary_source(false).unwrap();
    assert_eq!(regular.qualified[0].released.get(), 130_000);
    let amt = native.ordinary_source(true).unwrap();
    assert_eq!(amt.groups[0].passive, -100_000);
    assert!(amt.current_qualified.is_empty());
    assert!(amt.qualified.is_empty());
    let session = ordinary_multi_current_retained_qbi_session(native, 6_000_000);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "agi"), 64_000);
    assert_eq!(value(&result, "qbi_deduction"), 540);
}

#[test]
fn source_valid_unknowns_save_but_never_invent_passive_qualified_allocation() {
    let baseline = ordinary_multi_current_retained_qbi_session(
        ordinary_multi_passive_retained_qbi_parent([100_000, 200_000], [Some((100_000, 0)), None]),
        6_000_000,
    );
    for boundary in 0..11 {
        let mut native = partner_source(&baseline).clone();
        let graph = partner_activity_graph(&mut native);
        match boundary {
            // Entity net remains positive, but A's own group is negative.
            0 => partner_retained_opening_risk(graph),
            1 => graph.components[0].regular.as_mut().unwrap().passive_operating = None,
            2 => {
                graph.components[0].regular.as_mut().unwrap().passive_operating = Some(10_000);
                partnership_qualified_history(
                    &mut graph.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified,
                )
                .passive = vec![loss_year(2025, 100, 0, 100, 0)];
            }
            3 => {
                partnership_qualified_history(
                    &mut graph.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified,
                )
                .passive = vec![loss_year(2025, 100, 50, 0, 0)]
            }
            4 => graph.components[0].regular.as_mut().unwrap().operating_carry.basis = None,
            5 => {
                let K1ComponentQualified::Partnership { records, .. } =
                    &mut graph.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
                else {
                    unreachable!()
                };
                records.history = None;
            }
            6 => graph.passive_groups[0].participation = K1ActivityParticipation::Unavailable,
            7 => graph.components[0].qualified_current_controls.ordinary = None,
            8 => {
                graph.components[0].qualified_current_controls.ordinary =
                    Some(K1CurrentSourceParts { income: 200_000, loss: 100_000 })
            }
            9 => {
                partnership_qualified_history(
                    &mut graph.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified,
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
                    .collect()
            }
            10 => {
                let mut facts = crate::tests::support::builders::participation_facts();
                facts.ordinary_scope = false;
                graph.passive_groups[0].participation = K1ActivityParticipation::Partnership {
                    facts: PartnerParticipation::General { facts },
                    trade_kind: ScorpTradeKind::Ordinary,
                };
            }
            _ => unreachable!(),
        }
        let income = FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(native) };
        assert_eq!(income_issue(&income), None, "source validity {boundary}");
        let FederalIncome::PartnershipActivities { data } = &income else { unreachable!() };
        if boundary == 9 {
            // Exact original-year cents retain their qualification through FIFO.
            let output = data.ordinary_source(false).unwrap();
            assert_eq!(output.qualified[0].released.get(), 80_000);
            assert_eq!(
                output.qualified[0].carries[0]
                    .remaining
                    .iter()
                    .map(|row| row.year)
                    .collect::<Vec<_>>(),
                vec![2023, 2024, 2025]
            );
        } else {
            assert!(
                matches!(data.ordinary_source(false), Err(error) if error == ordinary::UNAVAILABLE),
                "owner availability {boundary}"
            );
        }
        let mut session = restart(baseline.draft.clone());
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut session, request));
        if boundary == 9 {
            let result = supported(&session.draft);
            assert_eq!(value(&result, "agi"), 62_000);
            assert_eq!(value(&result, "qbi_deduction"), 40);
            continue;
        }
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
fn invalid_passive_retained_native_edits_leave_accepted_state_atomic() {
    let mut session = ordinary_multi_current_retained_qbi_session(
        ordinary_multi_passive_retained_qbi_parent([400_000, 200_000], [Some((400_000, 0)), None]),
        6_000_000,
    );
    for boundary in 0..5 {
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let native = graph(&mut documents(&mut request)[0].document);
        match boundary {
            0 => {
                native.components[0].qualified_current_controls.ordinary =
                    Some(K1CurrentSourceParts { income: 399_999, loss: 0 })
            }
            1 => {
                partnership_qualified_history(
                    &mut native.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified,
                )
                .basis[0]
                    .qualified = 100_001
            }
            2 => native.components[1].id = native.components[0].id,
            3 => {
                native.qbi_blocks[0].allocations.as_mut().unwrap()[0].component =
                    session.draft.next_id + 99
            }
            4 => {
                let K1ReportedQbiStatement::Partnership { statement: Some(statement) } =
                    &mut native.qbi_blocks[0].statement
                else {
                    unreachable!()
                };
                statement.income -= 100;
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
fn no_statement_keeps_original_qualification_distinct_from_documented_zero() {
    for (statement, zero_history, expected_current, released) in
        [(None, false, 0, 130_000), (Some((0, 0)), false, 1, 130_000), (Some((0, 0)), true, 1, 0)]
    {
        let mut native =
            ordinary_multi_passive_retained_qbi_parent([400_000, 200_000], [statement, None]);
        if zero_history {
            let history = partnership_qualified_history(
                &mut partner_activity_graph(&mut native).qbi_blocks[0]
                    .allocations
                    .as_mut()
                    .unwrap()[0]
                    .qualified,
            );
            for row in history.basis.iter_mut().chain(&mut history.risk) {
                row.qualified = 0;
                row.remaining_qualified = 0;
            }
        }
        let income =
            FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(native.clone()) };
        assert_eq!(income_issue(&income), None);
        let output = native.ordinary_source(false).unwrap();
        assert_eq!(output.current_qualified.len(), expected_current);
        assert!(
            output
                .current_qualified
                .iter()
                .all(|row| !row.activity.active && row.activity.income == 0)
        );
        assert_eq!(output.qualified[0].released.get(), released);
        let session = ordinary_multi_current_retained_qbi_session(native, 6_000_000);
        let result = supported(&session.draft);
        assert_eq!(value(&result, "agi"), 64_000);
        assert_eq!(value(&result, "qbi_deduction"), 0);
        assert_eq!(line_amount(&result, "qbi_loss_carryforward"), released);
    }
}

#[test]
fn passive_retained_source_cent_totals_round_once_before_reporting() {
    let native =
        ordinary_multi_passive_retained_qbi_parent([400_049, 200_001], [Some((400_049, 0)), None]);
    let mut session = ordinary_multi_current_retained_qbi_session(native, 6_000_000);
    let mut changed = partner_source(&session).clone();
    let graph = partner_activity_graph(&mut changed);
    graph.components[1].risk_group = Some(graph.risk_groups[0].id);
    graph.risk_groups.pop();
    let income = FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(changed) };
    assert_eq!(income_issue(&income), None);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    documents(&mut request)[0].document = input(&income);
    accepted(&send_fixture(&mut session, request));
    assert_eq!(income_issue(&session.draft.income[0].income), None);
    let owner = partner_source(&session);
    assert_eq!(owner.current.ordinary, 600_050);
    let output = owner.ordinary_source(false).unwrap();
    // The risk owner retains $6,000.50 income and $2,000 actual losses.
    // Each separate passive group supplies its own printed amount: $2,000.49
    // and $2,000.01 both print as $2,000. Their exact cents remain available.
    assert_eq!(output.components.iter().map(|row| row.income.get()).sum::<i64>(), 600_050);
    assert_eq!(output.components.iter().map(|row| row.risk_allowed.get()).sum::<i64>(), 200_000);
    assert_eq!(output.groups.iter().map(|group| group.passive).sum::<i64>(), 400_050);
    assert_eq!(output.current_qualified[0].activity.income, 4_000);
    assert_eq!(output.qualified[0].released.get(), 130_000);
    let result = supported(&session.draft);
    assert_eq!(
        line_amount(
            &result,
            &alloc::format!("partnership_{}_schedule_e", session.draft.income[0].id)
        ),
        400_000
    );
    assert_eq!(value(&result, "agi"), 64_000);
    assert_eq!(value(&result, "qbi_deduction"), 540);
}
