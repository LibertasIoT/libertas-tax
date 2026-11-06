use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::assert::k1res::assert_atomic_rejection;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{
    documents, input, scorp_cash_repayment, scorp_repayment_priority,
};
use crate::tests::support::client::k1res::scorp_cash_repayment_candidate;
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::*;

#[test]
fn zero_gain_repayment_reaches_native_save_preview_correction_reopen_and_restart() {
    let (mut session, mut data) = scorp_cash_repayment_candidate(100_000);
    let income = FederalIncome::ScorpActivities { data: data.clone() };
    assert_eq!(crate::k1_activity::income_issue(&income), None);
    for (alternative, ending, allowed, suspended) in
        [(false, 2_000, 1_500, 6_500), (true, 3_000, 3_500, 4_500)]
    {
        let output = data.ordinary_source(alternative).unwrap();
        let loan = &output.basis.scorp.as_ref().unwrap().loans[0];
        // The actual owner recovers all $1,000 before losses reduce the
        // remaining $4,000 debt basis by $2,000 regular / $1,000 AMT.
        assert_eq!(
            (
                crate::kernel::ratio::dollars(loan.ordinary_gain.get()),
                crate::kernel::ratio::dollars(loan.capital_gain.get()),
                crate::kernel::ratio::dollars(loan.nontaxable_repayment.get())
            ),
            (0, 0, 1_000)
        );
        assert_eq!(
            (
                crate::kernel::ratio::dollars(loan.principal.get()),
                crate::kernel::ratio::dollars(loan.basis.get())
            ),
            (4_000, ending)
        );
        assert_eq!(
            (
                crate::kernel::ratio::dollars(output.components[1].basis_allowed.get()),
                crate::kernel::ratio::dollars(output.components[1].basis_suspended.get())
            ),
            (7_000, 0)
        );
        assert_eq!(
            (
                crate::kernel::ratio::dollars(output.components[1].risk_allowed.get()),
                crate::kernel::ratio::dollars(output.components[1].risk_suspended.get())
            ),
            (allowed, suspended)
        );
    }
    let id = session.draft.income[0].id;
    let ids: Vec<_> = data.activities.source().unwrap().ids().collect();
    let before = session.draft.clone();
    let dirty = session.dirty;
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    documents(&mut request)[0].document = input(&income);

    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut session,
            crate::tests::support::client::estimate_documents_from_save(&request)
        ),
        Reply::Response(P::Review { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
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
    // A's $4,000 less B's $2,000 former-passive / $1,500 nonpassive
    // leaves $500. AMT owns $1,000 former-passive / $3,500 nonpassive.
    assert_eq!(line_amount(&result, &alloc::format!("scorp_{id}_schedule_e")), 50_000);
    assert_eq!(line_amount(&result, &alloc::format!("amt_scorp_{id}_schedule_e")), -50_000);
    assert_eq!(value(&result, "agi"), 250_500);
    assert_eq!(line_amount(&result, "form_8960_net_investment_income"), 200_000);
    assert_eq!(line_amount(&result, "net_investment_income_tax"), 7_600);
    let mut reopened =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    assert_eq!(documents(&mut reopened)[0].document, expected_input(&saved, &saved.income[0]));
    let mut cold = restart(saved.clone());
    assert_eq!(cold.draft, saved);
    let mut unchanged =
        crate::tests::support::client::income_pages::open_document_center(&mut cold);
    assert_eq!(documents(&mut unchanged)[0].document, expected_input(&saved, &saved.income[0]));
    accepted(&send_fixture(&mut cold, unchanged));
    assert_eq!(cold.draft, saved);
    assert_eq!(estimate(&cold.draft), result);

    scorp_cash_repayment(&mut data, 0);
    let mut correction =
        crate::tests::support::client::income_pages::open_document_center(&mut cold);
    documents(&mut correction)[0].document = input(&FederalIncome::ScorpActivities { data });
    accepted(&send_fixture(&mut cold, correction));
    assert_eq!(cold.draft.revision, saved.revision + 1);
    assert_eq!(cold.draft.next_id, saved.next_id);
    assert_eq!(
        crate::k1_activity::source(&cold.draft.income[0].income).unwrap().ids().collect::<Vec<_>>(),
        ids
    );
    let corrected = estimate(&cold.draft);
    assert!(corrected.tax.is_some(), "{:?}", corrected.issues);
    assert_eq!(line_amount(&corrected, &alloc::format!("scorp_{id}_schedule_e")), -50_000);
    assert_eq!(line_amount(&corrected, &alloc::format!("amt_scorp_{id}_schedule_e")), -150_000);
    assert_eq!(value(&corrected, "agi"), 249_500);
    let accepted_draft = cold.draft.clone();
    let mut cold = restart(accepted_draft.clone());
    let unchanged = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    accepted(&send_fixture(&mut cold, unchanged));
    assert_eq!(cold.draft, accepted_draft);
    assert_eq!(estimate(&cold.draft), corrected);
}

#[test]
fn repayment_uses_actual_group_returns_cent_bounds_and_owner_restoration() {
    let (_, base) = scorp_cash_repayment_candidate(100_000);
    for (excluded, regular_allowed, alternative_allowed) in
        [(0, 1_500, 3_500), (50_000, 2_000, 4_000), (100_000, 2_500, 4_500)]
    {
        let mut candidate = base.clone();
        let K1ComponentStatement::Entered { source } = &mut candidate.activities else {
            unreachable!()
        };
        let K1RiskSource::Scorp { regular, alternative: Some(alternative) } =
            &mut source.risk_groups[1].source
        else {
            unreachable!()
        };
        regular.excluded_returns = excluded;
        alternative.excluded_returns = excluded;
        assert_eq!(
            crate::k1_activity::income_issue(&FederalIncome::ScorpActivities {
                data: candidate.clone()
            }),
            None
        );
        assert_eq!(
            crate::kernel::ratio::dollars(
                candidate.ordinary_source(false).unwrap().components[1].risk_allowed.get()
            ),
            regular_allowed
        );
        assert_eq!(
            crate::kernel::ratio::dollars(
                candidate.ordinary_source(true).unwrap().components[1].risk_allowed.get()
            ),
            alternative_allowed
        );
    }
    let mut assigned_a = base.clone();
    let K1ComponentStatement::Entered { source } = &mut assigned_a.activities else {
        unreachable!()
    };
    source.risk_groups[1].investment_parts.as_mut().unwrap().loan_repayments = 0;
    source.risk_groups[0].investment_parts.as_mut().unwrap().loan_repayments = 100_000;
    // Actual A investment cannot reduce B. Whole-interest debt basis is unchanged.
    let output = assigned_a.ordinary_source(false).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(output.components[1].basis_allowed.get()), 7_000);
    assert_eq!(crate::kernel::ratio::dollars(output.components[1].risk_allowed.get()), 2_500);
    assert_eq!(
        crate::kernel::ratio::dollars(
            assigned_a.ordinary_source(true).unwrap().components[1].risk_allowed.get()
        ),
        4_500
    );

    let mut cents = base.clone();
    scorp_cash_repayment(&mut cents, 51);
    cents.entity.regular.distributions = 49;
    cents.entity.alternative.as_mut().unwrap().distributions = 49;
    let K1ComponentStatement::Entered { source } = &mut cents.activities else { unreachable!() };
    source.risk_groups[1].investment_parts.as_mut().unwrap().distributions = 49;
    let K1RiskSource::Scorp { regular, alternative: Some(alternative) } =
        &mut source.risk_groups[1].source
    else {
        unreachable!()
    };
    regular.excluded_returns = 100;
    alternative.excluded_returns = 100;
    assert_eq!(
        crate::k1_activity::income_issue(&FederalIncome::ScorpActivities { data: cents.clone() }),
        None
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            cents.ordinary_source(false).unwrap().components[1].risk_allowed.get()
        ),
        2_500
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            cents.ordinary_source(true).unwrap().components[1].risk_allowed.get()
        ),
        4_500
    );
    let K1ComponentStatement::Entered { source } = &mut cents.activities else { unreachable!() };
    let K1RiskSource::Scorp { regular, .. } = &mut source.risk_groups[1].source else {
        unreachable!()
    };
    regular.excluded_returns = 101;
    // Bounds use 49+51 actual cents, even though both return exclusions report $1.
    assert_eq!(
        crate::k1_activity::income_issue(&FederalIncome::ScorpActivities { data: cents.clone() }),
        None
    );
    assert!(cents.ordinary_source(false).is_err());
    assert!(cents.ordinary_source(true).is_ok());

    let mut restored = base;
    restored.entity.loans[0].regular.basis = 100_000;
    restored.entity.loans[0].alternative_basis = Some(100_000);
    restored.current.ordinary = 500_000;
    restored.alternative_current.as_mut().unwrap().ordinary = 500_000;
    let K1ComponentStatement::Entered { source } = &mut restored.activities else { unreachable!() };
    source.components[1].regular.as_mut().unwrap().operating = 100_000;
    let K1StatementAmt::Separate { amounts } = &mut source.components[1].alternative else {
        unreachable!()
    };
    amounts.operating = 100_000;
    assert_eq!(
        crate::k1_activity::income_issue(&FederalIncome::ScorpActivities {
            data: restored.clone()
        }),
        None
    );
    for alternative in [false, true] {
        let output = restored.ordinary_source(alternative).unwrap();
        let loan = &output.basis.scorp.as_ref().unwrap().loans[0];
        // $5,000 income less $1,000 retained basis loss restores the full
        // $4,000 deficit before the existing owner recovers $1,000 principal.
        assert_eq!(
            (
                crate::kernel::ratio::dollars(loan.restoration.get()),
                crate::kernel::ratio::dollars(loan.nontaxable_repayment.get()),
                crate::kernel::ratio::dollars(loan.basis.get())
            ),
            (4_000, 1_000, 4_000)
        );
        assert_eq!(
            (
                crate::kernel::ratio::dollars(loan.ordinary_gain.get()),
                crate::kernel::ratio::dollars(loan.capital_gain.get())
            ),
            (0, 0)
        );
    }
}

#[test]
fn repayment_valid_unknown_sources_and_priority_save_without_invented_gains_or_history() {
    let (session, base) = scorp_cash_repayment_candidate(100_000);
    for boundary in 0..16 {
        let mut session = restart(session.draft.clone());
        let mut candidate = base.clone();
        if boundary == 14 || boundary == 15 {
            scorp_repayment_priority(&mut candidate, boundary == 15);
        } else {
            let K1ComponentStatement::Entered { source } = &mut candidate.activities else {
                unreachable!()
            };
            match boundary {
                0 => candidate.entity.loans[0].regular.basis = 100_000,
                1 => candidate.entity.loans[0].alternative_basis = Some(100_000),
                2 => candidate.entity.loans[0].alternative_basis = None,
                3 => {
                    let K1RiskSource::Scorp { alternative, .. } = &mut source.risk_groups[1].source
                    else {
                        unreachable!()
                    };
                    *alternative = None;
                }
                4 => source.risk_groups[1].investment_parts = None,
                5 => {
                    source.risk_groups[1].investment_parts.as_mut().unwrap().loan_repayments =
                        99_999
                }
                6..=9 => {
                    let K1RiskSource::Scorp { regular, .. } = &mut source.risk_groups[1].source
                    else {
                        unreachable!()
                    };
                    match boundary {
                        6 => regular.start = ScorpRiskStart::Unknown,
                        7 => regular.start = ScorpRiskStart::New,
                        8 => regular.complete = false,
                        9 => {
                            regular.start =
                                ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
                            regular.deducted = 100_000;
                        }
                        _ => unreachable!(),
                    }
                }
                10 => {
                    let K1ComponentQualified::Scorp { ordinary, .. } =
                        &mut source.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified
                    else {
                        unreachable!()
                    };
                    ordinary.history[0].qualified = 1;
                    ordinary.history[0].remaining_qualified = 1;
                }
                11 => {
                    source.qbi_blocks[1].statement = K1ReportedQbiStatement::Scorp {
                        statement: Some(K1ScorpBlockStatement {
                            income: 0,
                            section1231_qbi: 0,
                            wages: 0,
                            property: 0,
                            service: false,
                            reit_dividends: 0,
                            ptp_income: 0,
                        }),
                    }
                }
                12 => candidate.entity.regular.complete = false,
                13 => source.components[1].risk_group = None,
                _ => unreachable!(),
            }
        }
        let income = FederalIncome::ScorpActivities { data: candidate.clone() };
        assert_eq!(crate::k1_activity::income_issue(&income), None, "source validity {boundary}");
        let alternative_only = matches!(boundary, 1 | 2 | 3 | 15);
        assert!(candidate.ordinary_source(alternative_only).is_err(), "owner boundary {boundary}");
        if alternative_only {
            assert!(candidate.ordinary_source(false).is_ok(), "regular remains known {boundary}");
        }
        if boundary == 0 {
            assert!(candidate.ordinary_source(true).is_ok(), "AMT zero-gain remains known");
        }
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut session, request));
        let result = estimate(&session.draft);
        assert!(result.tax.is_none(), "boundary {boundary}");
        assert_eq!(
            result
                .issues
                .iter()
                .filter(|issue| issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
                .count(),
            1,
            "boundary {boundary}: {:?}",
            result.issues
        );
        let saved = session.draft.clone();
        let mut cold = restart(saved.clone());
        let mut unchanged =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(documents(&mut unchanged)[0].document, expected_input(&saved, &saved.income[0]));
        accepted(&send_fixture(&mut cold, unchanged));
        assert_eq!(cold.draft, saved);
        assert_eq!(estimate(&cold.draft), result);
    }
}

#[test]
fn repayment_invalid_loan_returns_term_and_source_id_edits_are_atomic() {
    let (mut session, base) = scorp_cash_repayment_candidate(100_000);
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    documents(&mut request)[0].document =
        input(&FederalIncome::ScorpActivities { data: base.clone() });
    accepted(&send_fixture(&mut session, request));
    for boundary in [1, 3, 4, 5, 6, 7] {
        let mut candidate = base.clone();
        let K1ComponentStatement::Entered { source } = &mut candidate.activities else {
            unreachable!()
        };
        let key = match boundary {
            1 => {
                candidate.entity.loans[0].regular.repayments = 500_001;
                "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
            }
            3 => {
                candidate.entity.loans[0].regular.basis = 500_001;
                "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
            }
            4 => {
                candidate.entity.loans[0].alternative_basis = Some(500_001);
                "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
            }
            5 => {
                candidate.entity.loans[0].regular.repayment_term = ScorpNoteHolding::NoRepayment;
                "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
            }
            6 => {
                source.components[1].risk_group = Some(i64::MAX);
                "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED"
            }
            7 => {
                source.risk_groups[1].id = i64::MAX;
                "K1_ACTIVITY_SOURCE_ID_NOT_ACCEPTED"
            }
            _ => unreachable!(),
        };
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document =
            input(&FederalIncome::ScorpActivities { data: candidate });
        assert_atomic_rejection(&mut session, request, key);
    }
}
