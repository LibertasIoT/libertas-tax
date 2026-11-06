use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::assert::k1res::assert_atomic_rejection;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{
    current_scorp_qbi_source_with_statements, documents, set_scorp_entity_interest,
};
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{
    scorp_entity_interest_session, scorp_ordinary_multi_session, scorp_source_request,
};
use crate::tests::*;

#[test]
fn entity_interest_basis_return_and_native_correction_lifecycle() {
    let (mut session, mut source) = scorp_entity_interest_session(
        empty_return(FederalStatus::Single),
        100_000,
        [200_000, 100_000],
        1_000_000,
        0,
    );
    add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
    session = restart(session.draft);
    let before = session.draft.clone();
    let id = before.income[0].id;
    let ids: Vec<_> = source.activities.source().unwrap().ids().collect();
    let dirty = session.dirty;
    let request = scorp_source_request(&mut session, source.clone());
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            crate::tests::support::client::estimate_documents_from_save(&request)
        ),
        Reply::Response(P::Review { .. })
    ));
    assert_eq!((session.draft.clone(), session.dirty), (before.clone(), dirty));
    accepted(&send_fixture(&mut session, request));
    let saved = session.draft.clone();
    assert_eq!(saved.revision, before.revision + 2);
    assert_eq!(saved.next_id, before.next_id);
    assert_eq!(
        crate::k1_activity::source(&saved.income[0].income).unwrap().ids().collect::<Vec<_>>(),
        ids
    );
    let result = estimate(&saved);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    // $1,000 interest increases global basis once: REG permits $2,000,
    // AMT $1,500. Neither group receives portfolio income for risk or PAL.
    assert_eq!(line_amount(&result, &alloc::format!("scorp_{id}_schedule_e")), -200_000);
    assert_eq!(line_amount(&result, &alloc::format!("amt_scorp_{id}_schedule_e")), -150_000);
    assert_eq!(line_amount(&result, &alloc::format!("scorp_{id}_interest")), 100_000);
    assert_eq!(line_amount(&result, "agi"), 24_900_000);
    assert_eq!(line_amount(&result, "form_8960_net_investment_income"), 100_000);
    assert_eq!(line_amount(&result, "net_investment_income_tax"), 3_800);
    assert_eq!(return_line(&result, "2b"), 100_000);

    assert_eq!(session.draft, saved);
    let mut reopened =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    assert_eq!(documents(&mut reopened)[0].document, expected_input(&saved, &saved.income[0]));
    accepted(&send_fixture(&mut session, reopened));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    assert_eq!(estimate(&cold.draft), result);
    set_scorp_entity_interest(&mut source, 0);
    let request = scorp_source_request(&mut cold, source.clone());
    accepted(&send_fixture(&mut cold, request));
    let corrected = cold.draft.clone();
    assert_eq!((corrected.revision, corrected.next_id), (saved.revision + 2, saved.next_id));
    assert_eq!(source.activities.source().unwrap().ids().collect::<Vec<_>>(), ids);
    let result = estimate(&corrected);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    assert_eq!(line_amount(&result, &alloc::format!("scorp_{id}_schedule_e")), -100_000);
    assert_eq!(line_amount(&result, &alloc::format!("amt_scorp_{id}_schedule_e")), -50_000);
    assert_eq!(line_amount(&result, "agi"), 24_900_000);
    assert_eq!(line_amount(&result, "net_investment_income_tax"), 0);
    let mut cold = restart(corrected.clone());
    let request = scorp_source_request(&mut cold, source);
    accepted(&send_fixture(&mut cold, request));
    assert_eq!(cold.draft, corrected);
}

#[test]
fn entity_interest_binding_groups_close_risk_before_retained_passive_and_preserve_source_cents() {
    use crate::kernel::carry::{Amount, Cents};
    use crate::kernel::loss_limits::risk::{RecaptureHistory, close_economic_account};
    use crate::kernel::source_fact::SourceFact;
    let (_, source) = scorp_entity_interest_session(
        empty_return(FederalStatus::Single),
        100_000,
        [200_000, 100_000],
        50_000,
        200_000,
    );
    let graph = source.activities.source().unwrap();
    assert_eq!(
        crate::k1_activity::income_issue(&FederalIncome::ScorpActivities { data: source.clone() }),
        None
    );
    for interest in [100_000, 0] {
        let mut value = source.clone();
        set_scorp_entity_interest(&mut value, interest);
        for alternative in [false, true] {
            let output = value.ordinary_source(alternative).unwrap();
            let (expected, closing, passive, nonpassive) = match (interest, alternative) {
                (100_000, false) => (
                    [(133_333, 66_667, 50_000, 83_333), (66_667, 33_333, 50_000, 16_667)],
                    [0, 0],
                    -250_000,
                    -50_000,
                ),
                (100_000, true) => (
                    [(100_000, 100_000, 50_000, 50_000), (50_000, 50_000, 50_000, 0)],
                    [0, 0],
                    -250_000,
                    -50_000,
                ),
                (0, false) => (
                    [(66_667, 133_333, 50_000, 16_667), (33_333, 66_667, 33_333, 0)],
                    [0, 16_667],
                    -233_333,
                    -50_000,
                ),
                (0, true) => (
                    [(33_333, 166_667, 33_333, 0), (16_667, 83_333, 16_667, 0)],
                    [16_667, 33_333],
                    -216_667,
                    -33_333,
                ),
                _ => unreachable!(),
            };
            for (index, (component, expected)) in output.components.iter().zip(expected).enumerate()
            {
                assert_eq!(
                    (
                        component.basis_allowed.get(),
                        component.basis_suspended.get(),
                        component.risk_allowed.get(),
                        component.risk_suspended.get()
                    ),
                    expected
                );
                let K1RiskSource::Scorp { regular, alternative: amt } =
                    &graph.risk_groups[index].source
                else {
                    unreachable!()
                };
                let risk = if alternative { amt.as_ref().unwrap() } else { regular };
                let investment = risk
                    .component_investment(
                        graph.risk_groups[index].investment_parts.as_ref().unwrap(),
                        0,
                        true,
                    )
                    .unwrap();
                assert_eq!(investment, 50_000);
                // Consume the actual allowed Part I net through the existing
                // economic-account owner, with the actual zero recapture history.
                let account = close_economic_account::<Cents>(
                    investment,
                    output.groups[index].current,
                    SourceFact::Reported(RecaptureHistory {
                        prior_deducted: Amount::new(0).unwrap(),
                        prior_recaptured: Amount::new(0).unwrap(),
                        threshold_1978: 0,
                    }),
                )
                .unwrap();
                assert_eq!(account.ending, closing[index]);
                let SourceFact::Reported(recapture) = account.recapture else { unreachable!() };
                assert_eq!((recapture.income.get(), recapture.next_year_deduction.get()), (0, 0));
            }
            assert_eq!(
                (
                    output.groups[1].current,
                    output.groups[1].prior,
                    output.groups[1].prior_released,
                    output.groups[1].passive
                ),
                (passive + 200_000, 200_000, 0, passive)
            );
            assert_eq!(output.groups[0].nonpassive, nonpassive);
        }
    }
    // Source cents remain exact inside the global basis owner.
    // 2026-10-20: preserve the cents contract before printed-form rounding.
    for interest in [100_049, 100_050, 100_051] {
        let allowed = 100_000 + interest;
        let mut value = source.clone();
        set_scorp_entity_interest(&mut value, interest);
        for alternative in [false, true] {
            let output = value.ordinary_source(alternative).unwrap();
            assert_eq!(output.entity_interest, interest);
            assert_eq!(
                output.components.iter().map(|c| c.basis_allowed.get()).sum::<i64>(),
                if alternative { allowed - 50_000 } else { allowed }
            );
            assert_eq!(
                output.components.iter().map(|c| c.risk_allowed.get()).sum::<i64>(),
                100_000
            );
            assert_eq!(output.groups[1].passive, -250_000);
        }
    }
}

#[test]
fn entity_interest_joint_retainer_keeps_actual_owner_on_correction() {
    for ordinary in [0, 6_000] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = injured_return((200_000, 100_000), (50_000, 30_000), 0, allocation());
            let mut other = income(savings_income(2_000, 0));
            other.owner = if owner == Owner::Taxpayer {
                crate::DocumentOwner::Spouses
            } else {
                crate::DocumentOwner::Yours
            };
            add(&mut draft, other);
            let (mut session, mut source) =
                scorp_entity_interest_session(draft, 100_000, [0, 0], 1_000_000, 0);
            if ordinary != 0 {
                // Existing source owner supplies actual $4,000 and $2,000
                // nonpassive components. Final PAL aggregates their $6,000 E
                // amount after collect retained the separate portfolio interest.
                source = current_scorp_qbi_source_with_statements(source, [None, None]);
            }
            let id = session
                .draft
                .income
                .iter()
                .find(|e| matches!(e.income, FederalIncome::ScorpActivities { .. }))
                .unwrap()
                .id;
            let income = FederalIncome::ScorpActivities { data: source.clone() };
            assert_eq!(crate::k1_activity::income_issue(&income), None);
            let mut entry =
                session.draft.income.iter().find(|entry| entry.id == id).unwrap().clone();
            entry.income = income;
            entry.owner = crate::DocumentOwner::from(owner);
            assert!(crate::federal_rules::validate_income_issue(&entry).is_ok());
            let mut request = scorp_source_request(&mut session, source.clone());
            documents(&mut request).iter_mut().find(|row| row.id == id).unwrap().owner =
                crate::DocumentOwner::from(owner);
            let request = fixture_owner_inputs(&session.draft, request);
            accepted(&send_fixture(&mut session, request));
            for (interest, joint_niit, source_niit, other_niit) in
                [(100_000, 114, 38, 76), (0, 76, 0, 76)]
            {
                if interest == 0 {
                    set_scorp_entity_interest(&mut source, 0);
                    let request = scorp_source_request(&mut session, source.clone());
                    let request = fixture_owner_inputs(&session.draft, request);
                    accepted(&send_fixture(&mut session, request));
                    session = restart(session.draft);
                }
                for injured in [Owner::Taxpayer, Owner::Spouse] {
                    let mut draft = session.draft.clone();
                    draft.injured_spouse.as_mut().unwrap().injured = injured;
                    let result = supported(&draft);
                    assert_eq!(return_line(&result, "2b"), 200_000 + interest);
                    assert_eq!(line_amount(&result, "agi"), 25_200_000 + interest + ordinary * 100);
                    if ordinary != 0 {
                        assert_eq!(
                            line_amount(&result, &alloc::format!("scorp_{id}_schedule_e")),
                            600_000
                        );
                        assert_eq!(
                            line_amount(&result, &alloc::format!("amt_scorp_{id}_schedule_e")),
                            600_000
                        );
                    }
                    assert_eq!(line_amount(&result, "net_investment_income_tax"), joint_niit * 100);
                    assert_eq!(
                        ["a", "b", "c"].map(|column| niit_optional_column(&result, "18", column)),
                        [
                            Some(joint_niit),
                            Some(if owner == injured { source_niit } else { other_niit }),
                            Some(if owner == injured { other_niit } else { source_niit })
                        ]
                    );
                    assert_eq!(
                        niit_optional_column(&result, "13b", "b"),
                        Some(if owner == injured {
                            ordinary + crate::kernel::ratio::dollars(interest)
                        } else {
                            2_000
                        })
                    );
                }
            }
        }
    }
}

#[test]
fn entity_interest_valid_unknown_sources_stay_unknown_after_native_save_and_restart() {
    let (base, source) = scorp_entity_interest_session(
        empty_return(FederalStatus::Single),
        100_000,
        [200_000, 100_000],
        1_000_000,
        0,
    );
    for case in 0..15 {
        let mut value = source.clone();
        match case {
            0 => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                source.entity_only = None;
            }
            1 => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                source.alternative_entity_only = K1EntityOnlyAmt::Unavailable;
            }
            2 => value.alternative_current = None,
            3 => value.entity.alternative = None,
            4 | 5 => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                let mut alternative = source.entity_only.as_ref().unwrap().clone();
                if case == 4 {
                    alternative.portfolio.interest = 200_000;
                } else {
                    source.entity_only.as_mut().unwrap().portfolio.interest = 200_000;
                    value.current.portfolio.interest = 200_000;
                }
                value.alternative_current.as_mut().unwrap().portfolio.interest =
                    alternative.portfolio.interest;
                source.alternative_entity_only = K1EntityOnlyAmt::Separate { amounts: alternative };
            }
            6 => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                source.entity_only.as_mut().unwrap().exempt_interest = 100;
                value.current.exempt_interest = 100;
                value.alternative_current.as_mut().unwrap().exempt_interest = 100;
            }
            7 => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                source.components[0].regular.as_mut().unwrap().portfolio.interest = 100;
                value.current.portfolio.interest += 100;
                value.alternative_current = Some(value.current.clone());
            }
            8 => value = current_scorp_qbi_source_with_statements(value, [Some((100, 0)), None]),
            9 => value.entity.loans.push(ScorpEntityLoan {
                regular: ScorpLoan {
                    kind: ScorpLoanKind::FormalNote,
                    repayment_term: ScorpNoteHolding::NoRepayment,
                    principal: 100_000,
                    basis: 100_000,
                    advances: 0,
                    repayments: 0,
                },
                alternative_basis: Some(100_000),
            }),
            10 => value.special.push(ScorpSpecial::DebtForgiveness),
            11 => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                let K1RiskSource::Scorp { regular, .. } = &mut source.risk_groups[0].source else {
                    unreachable!()
                };
                regular.start = ScorpRiskStart::Unknown;
            }
            12 => {
                value.entity.regular.distributions = 300_000;
                value.entity.alternative.as_mut().unwrap().distributions = 300_000;
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                source.risk_groups[0].investment_parts.as_mut().unwrap().distributions = 300_000;
            }
            13 => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                let K1RiskSource::Scorp { alternative, .. } = &mut source.risk_groups[0].source
                else {
                    unreachable!()
                };
                *alternative = None;
            }
            _ => value.person.as_mut().unwrap().withholding = None,
        }
        let income = FederalIncome::ScorpActivities { data: value.clone() };
        assert_eq!(crate::k1_activity::income_issue(&income), None, "valid unknown {case}");
        let mut entry = base.draft.income[0].clone();
        entry.income = income;
        assert!(
            crate::federal_rules::validate_income_issue(&entry).is_ok(),
            "full validator {case}"
        );
        // 2026-10-18: case 11 changes only regular risk history. The
        // independent known AMT source stays known (source availability contract).
        // 2026-10-20: IRS i1120ssk separates portfolio income from business
        // QBI; i7203 includes it in stock income alongside actual debt records.
        // Known sources compose independently of another system's missing facts.
        let complete = [4, 5, 6, 8, 9].contains(&case);
        assert_eq!(value.ordinary_source(true).is_ok(), complete || case == 11, "AMT {case}");
        assert_eq!(
            value.ordinary_source(false).is_ok(),
            complete || [1, 2, 3, 13].contains(&case),
            "REG {case}"
        );
        let mut session = restart(base.draft.clone());
        let request = scorp_source_request(&mut session, value);
        accepted(&send_fixture(&mut session, request));
        let saved = session.draft.clone();
        let result = estimate(&saved);
        assert_eq!(result.tax.is_some(), complete, "case {case}: {:?}", result.issues);
        assert_eq!(
            result
                .issues
                .iter()
                .filter(|issue| issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
                .count(),
            usize::from(!complete),
            "case {case}"
        );
        let request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        accepted(&send_fixture(&mut session, request));
        assert_eq!(session.draft, saved);
        let mut cold = restart(saved.clone());
        let request = crate::tests::support::client::income_pages::open_document_center(&mut cold);
        accepted(&send_fixture(&mut cold, request));
        assert_eq!(cold.draft, saved);
    }
    // Actual zero portfolio income also retains the existing debt-basis path.
    let (_, existing) = scorp_ordinary_multi_session();
    assert!(existing.entity.loans.len() == 1);
    assert!(existing.ordinary_source(false).is_ok());
    assert!(existing.ordinary_source(true).is_ok());
    let mut zero = source;
    set_scorp_entity_interest(&mut zero, 0);
    assert!(zero.ordinary_source(false).is_ok());
    assert!(zero.ordinary_source(true).is_ok());
}

#[test]
fn entity_interest_invalid_amount_controls_and_source_links_reject_atomically() {
    let (mut session, source) = scorp_entity_interest_session(
        empty_return(FederalStatus::Single),
        100_000,
        [200_000, 100_000],
        1_000_000,
        0,
    );
    let request = scorp_source_request(&mut session, source.clone());
    accepted(&send_fixture(&mut session, request));
    for case in 1..3 {
        let mut value = source.clone();
        let key = match case {
            1 => {
                value.current.portfolio.interest += 1;
                "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE"
            }
            _ => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                source.components[0].risk_group = Some(i64::MAX);
                "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED"
            }
        };
        let request = scorp_source_request(&mut session, value);
        assert_atomic_rejection(&mut session, request, key);
    }
}
