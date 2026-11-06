use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::assert::k1res::assert_atomic_rejection;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{documents, set_partner_entity_interest};
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{
    partner_entity_interest_session, partner_source_request,
};
use crate::tests::*;

#[test]
fn partner_interest_native_save_correction_and_restart_use_outside_basis_once() {
    let (mut session, mut value) = partner_entity_interest_session(
        empty_return(FederalStatus::Single),
        100_000,
        [-200_000, -100_000],
        1_000_000,
        0,
    );
    add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
    session = restart(session.draft);
    let before = session.draft.clone();
    let dirty = session.dirty;
    let id = before.income[0].id;
    let ids: Vec<_> = value.activities.source().unwrap().ids().collect();
    let request = partner_source_request(&mut session, value.clone());
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            crate::tests::support::client::estimate_documents_from_save(&request)
        ),
        Reply::Response(P::Review { .. })
    ));
    assert_eq!((&session.draft, session.dirty), (&before, dirty));
    accepted(&send_fixture(&mut session, request));
    let saved = session.draft.clone();
    assert_eq!((saved.revision, saved.next_id), (before.revision + 2, before.next_id));
    assert_eq!(
        crate::k1_activity::source(&saved.income[0].income).unwrap().ids().collect::<Vec<_>>(),
        ids
    );
    // The partner's outside-basis worksheet includes positive portfolio income
    // before losses: REG $1,000 + $1,000; AMT $500 + $1,000.
    let result = supported(&saved);
    assert_eq!(line_amount(&result, &alloc::format!("partnership_{id}_schedule_e")), -200_000);
    assert_eq!(line_amount(&result, &alloc::format!("amt_partnership_{id}_schedule_e")), -150_000);
    assert_eq!(line_amount(&result, &alloc::format!("partnership_{id}_interest")), 100_000);
    assert_eq!(return_line(&result, "2b"), 100_000);
    assert_eq!(line_amount(&result, "agi"), 24_900_000);
    assert_eq!(line_amount(&result, "net_investment_income_tax"), 3_800);

    let request = crate::tests::support::client::income_pages::open_document_center(&mut session);
    accepted(&send_fixture(&mut session, request));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let mut reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    assert_eq!(documents(&mut reopened)[0].document, expected_input(&saved, &saved.income[0]));
    assert_eq!(estimate(&cold.draft), result);
    set_partner_entity_interest(&mut value, 0);
    let request = partner_source_request(&mut cold, value);
    accepted(&send_fixture(&mut cold, request));
    let corrected = cold.draft.clone();
    assert_eq!((corrected.revision, corrected.next_id), (saved.revision + 2, saved.next_id));
    assert_eq!(
        crate::k1_activity::source(&corrected.income[0].income).unwrap().ids().collect::<Vec<_>>(),
        ids
    );
    let result = supported(&corrected);
    assert_eq!(line_amount(&result, &alloc::format!("partnership_{id}_schedule_e")), -100_000);
    assert_eq!(line_amount(&result, &alloc::format!("amt_partnership_{id}_schedule_e")), -50_000);
    assert_eq!(line_amount(&result, "agi"), 24_900_000);
    assert_eq!(line_amount(&result, "net_investment_income_tax"), 0);
    let mut cold = restart(corrected.clone());
    let request = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    accepted(&send_fixture(&mut cold, request));
    assert_eq!(cold.draft, corrected);
}

#[test]
fn partner_liabilities_increase_outside_basis_without_becoming_group_risk_or_passive_income() {
    use crate::kernel::carry::{Amount, Cents};
    use crate::kernel::loss_limits::risk::{RecaptureHistory, close_economic_account};
    use crate::kernel::source_fact::SourceFact;
    let (_, base) = partner_entity_interest_session(
        empty_return(FederalStatus::Single),
        100_000,
        [-200_000, -100_000],
        50_000,
        200_000,
    );
    // The partnership has $1,000 additional ordinary nonrecourse liabilities.
    // No group records that debt as recourse/qualified nonrecourse financing.
    for liabilities in [0, 100_000] {
        let mut value = base.clone();
        value.entity.regular.ending_liabilities = liabilities;
        value.entity.alternative.as_mut().unwrap().ending_liabilities = liabilities;
        let income = FederalIncome::PartnershipActivities { data: value.clone() };
        assert_eq!(crate::k1_activity::income_issue(&income), None);
        let graph = value.activities.source().unwrap();
        for amt in [false, true] {
            let output = value.ordinary_source(amt).unwrap();
            let expected = match (liabilities, amt) {
                (0, false) => [(133_333, 66_667, 50_000, 83_333), (66_667, 33_333, 50_000, 16_667)],
                (0, true) => [(100_000, 100_000, 50_000, 50_000), (50_000, 50_000, 50_000, 0)],
                (100_000, false) => [(200_000, 0, 50_000, 150_000), (100_000, 0, 50_000, 50_000)],
                (100_000, true) => {
                    [(166_667, 33_333, 50_000, 116_667), (83_333, 16_667, 50_000, 33_333)]
                }
                _ => unreachable!(),
            };
            assert_eq!(output.basis.ending.get(), 0);
            for (index, component) in output.components.iter().enumerate() {
                assert_eq!(
                    (
                        component.basis_allowed.get(),
                        component.basis_suspended.get(),
                        component.risk_allowed.get(),
                        component.risk_suspended.get()
                    ),
                    expected[index]
                );
                let K1RiskSource::Partnership { regular, alternative } =
                    &graph.risk_groups[index].source
                else {
                    unreachable!()
                };
                let risk = if amt { alternative.as_ref().unwrap() } else { regular };
                let investment = risk
                    .investment(
                        crate::partnership::PartnerRiskFinancing {
                            contributions: 0,
                            distributions: 0,
                            opening_liabilities: 0,
                            ending_liabilities: liabilities,
                            assumed: 0,
                            transferred: 0,
                            has_prior: true,
                        },
                        0,
                        0,
                    )
                    .unwrap()
                    .unwrap();
                assert_eq!(investment, 50_000);
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
                assert_eq!(account.ending, 0);
                let SourceFact::Reported(recapture) = account.recapture else { unreachable!() };
                assert_eq!(recapture.income.get(), 0);
            }
            assert_eq!(
                (
                    output.groups[0].nonpassive,
                    output.groups[1].current,
                    output.groups[1].prior,
                    output.groups[1].prior_released,
                    output.groups[1].passive
                ),
                (-50_000, -50_000, 200_000, 0, -250_000)
            );
        }
    }
    // Source cents remain exact inside the outside-basis owner, independently
    // from group financing; retained PAL never absorbs this portfolio interest.
    for interest in [100_049, 100_050, 100_051] {
        let basis = 100_000 + interest;
        let mut value = base.clone();
        set_partner_entity_interest(&mut value, interest);
        for amt in [false, true] {
            let output = value.ordinary_source(amt).unwrap();
            assert_eq!(output.entity_interest, interest);
            assert_eq!(
                output.components.iter().map(|v| v.basis_allowed.get()).sum::<i64>(),
                basis - if amt { 50_000 } else { 0 }
            );
            assert_eq!(
                output.components.iter().map(|v| v.risk_allowed.get()).sum::<i64>(),
                100_000
            );
            assert_eq!(output.groups[1].passive, -250_000);
        }
    }
}

#[test]
fn partner_interest_retainer_preserves_both_owners_with_zero_and_positive_schedule_e() {
    for operating in [[0, 0], [400_000, 200_000]] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let mut draft = injured_return((200_000, 100_000), (50_000, 30_000), 0, allocation());
            let mut other = income(savings_income(2_000, 0));
            other.owner = if owner == Owner::Taxpayer {
                crate::DocumentOwner::Spouses
            } else {
                crate::DocumentOwner::Yours
            };
            add(&mut draft, other);
            let (mut session, mut value) =
                partner_entity_interest_session(draft, 100_000, operating, 1_000_000, 0);
            let id = session
                .draft
                .income
                .iter()
                .find(|e| matches!(e.income, FederalIncome::PartnershipActivities { .. }))
                .unwrap()
                .id;
            let mut request = partner_source_request(&mut session, value.clone());
            documents(&mut request).iter_mut().find(|row| row.id == id).unwrap().owner =
                crate::DocumentOwner::from(owner);
            let request = fixture_owner_inputs(&session.draft, request);
            accepted(&send_fixture(&mut session, request));
            for (interest, joint, source) in [(100_000, 114, 38), (0, 76, 0)] {
                if interest == 0 {
                    set_partner_entity_interest(&mut value, 0);
                    let request = partner_source_request(&mut session, value.clone());
                    let request = fixture_owner_inputs(&session.draft, request);
                    accepted(&send_fixture(&mut session, request));
                    session = restart(session.draft);
                }
                for injured in [Owner::Taxpayer, Owner::Spouse] {
                    let mut draft = session.draft.clone();
                    draft.injured_spouse.as_mut().unwrap().injured = injured;
                    let result = supported(&draft);
                    let ordinary = operating.iter().sum::<i64>();
                    assert_eq!(return_line(&result, "2b"), 200_000 + interest);
                    assert_eq!(line_amount(&result, "agi"), 25_200_000 + interest + ordinary);
                    if ordinary != 0 {
                        assert_eq!(
                            line_amount(&result, &alloc::format!("partnership_{id}_schedule_e")),
                            ordinary
                        );
                        assert_eq!(
                            line_amount(
                                &result,
                                &alloc::format!("amt_partnership_{id}_schedule_e")
                            ),
                            ordinary
                        );
                    }
                    assert_eq!(line_amount(&result, "net_investment_income_tax"), joint * 100);
                    assert_eq!(
                        ["a", "b", "c"].map(|c| niit_optional_column(&result, "18", c)),
                        [
                            Some(joint),
                            Some(if owner == injured { source } else { 76 }),
                            Some(if owner == injured { 76 } else { source })
                        ]
                    );
                    assert_eq!(
                        niit_optional_column(&result, "13b", "b"),
                        Some(if owner == injured {
                            crate::kernel::ratio::dollars(ordinary + interest)
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
fn partner_interest_valid_unknown_sources_save_and_preserve_independent_known_amt() {
    let (base, source) = partner_entity_interest_session(
        empty_return(FederalStatus::Single),
        100_000,
        [-200_000, -100_000],
        1_000_000,
        0,
    );
    for case in 0..13 {
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
            4 => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                let mut amounts = source.entity_only.as_ref().unwrap().clone();
                amounts.portfolio.interest += 100;
                value.alternative_current.as_mut().unwrap().portfolio.interest += 100;
                source.alternative_entity_only = K1EntityOnlyAmt::Separate { amounts };
            }
            5 => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                source.components[0].regular.as_mut().unwrap().portfolio.interest = 100;
                value.current.portfolio.interest += 100;
                value.alternative_current = Some(value.current.clone());
            }
            6 | 7 => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                let K1RiskSource::Partnership { regular, alternative } =
                    &mut source.risk_groups[0].source
                else {
                    unreachable!()
                };
                if case == 6 {
                    regular.start = ScorpRiskStart::Unknown;
                } else {
                    *alternative = None;
                }
            }
            8 => value.person.as_mut().unwrap().reported_se = None,
            9 | 10 => {
                // An actual missing regular fact leaves its worksheet incomplete;
                // the separately established AMT worksheet remains complete.
                value.entity.regular.complete = false;
                if case == 9 {
                    value.entity.regular.opening = None;
                } else {
                    value.entity.regular.prior_nondeductible = None;
                }
            }
            11 => {
                value.entity.regular.distributions = 400_000;
                value.entity.alternative.as_mut().unwrap().distributions = 400_000;
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                source.risk_groups[0].investment_parts.as_mut().unwrap().distributions = 400_000;
            }
            _ => {
                let K1ComponentStatement::Entered { source } = &mut value.activities else {
                    unreachable!()
                };
                source.qbi_blocks[0].allocations = None;
            }
        }
        let mut entry = base.draft.income[0].clone();
        entry.income = FederalIncome::PartnershipActivities { data: value.clone() };
        // First establish that each boundary is a valid accepted source, rather
        // than a malformed fixture rejected before the calculation is reached.
        assert_eq!(crate::k1_activity::income_issue(&entry.income), None, "source {case}");
        assert!(
            crate::federal_rules::validate_income_issue(&entry).is_ok(),
            "full validator {case}"
        );
        // 2026-10-20: IRS i1065sk1 and i6251 use independently documented
        // regular and AMT interest; one system's absence does not erase the other.
        assert_eq!(
            value.ordinary_source(false).is_ok(),
            [1, 2, 3, 4, 7].contains(&case),
            "REG {case}"
        );
        assert_eq!(
            value.ordinary_source(true).is_ok(),
            [4, 6, 9, 10].contains(&case),
            "AMT {case}"
        );
        let mut session = restart(base.draft.clone());
        let request = partner_source_request(&mut session, value);
        accepted(&send_fixture(&mut session, request));
        let saved = session.draft.clone();
        let result = estimate(&saved);
        assert_eq!(result.tax.is_some(), case == 4, "case {case}: {:?}", result.issues);
        assert_eq!(
            result
                .issues
                .iter()
                .filter(|issue| issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
                .count(),
            usize::from(case != 4),
            "case {case}"
        );
        let mut cold = restart(saved.clone());
        let request = crate::tests::support::client::income_pages::open_document_center(&mut cold);
        accepted(&send_fixture(&mut cold, request));
        assert_eq!(cold.draft, saved);
    }
}

#[test]
fn partner_interest_invalid_controls_and_stale_group_links_are_atomic() {
    let (mut session, source) = partner_entity_interest_session(
        empty_return(FederalStatus::Single),
        100_000,
        [-200_000, -100_000],
        1_000_000,
        0,
    );
    let request = partner_source_request(&mut session, source.clone());
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
        let request = partner_source_request(&mut session, value);
        assert_atomic_rejection(&mut session, request, key);
    }
}
