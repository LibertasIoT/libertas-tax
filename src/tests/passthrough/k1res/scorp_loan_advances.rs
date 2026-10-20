use crate::k1_activity::*;
use crate::tests::cross::interview::{accepted, open_topic};
use crate::tests::support::assert::k1res::assert_atomic_rejection;
use crate::tests::support::builders::k1res::{documents, input, scorp_cash_advance};
use crate::tests::support::client::income_pages::catalog_preview_document;
use crate::tests::support::client::k1res::scorp_cash_advance_candidate;
use crate::tests::*;

#[test]
fn scorp_cash_advance_uses_actual_stock_debt_and_group_risk_through_native_lifecycle() {
    let (mut session, mut data) = scorp_cash_advance_candidate(100_000);
    let regular = data.ordinary_source(false).unwrap();
    let alternative = data.ordinary_source(true).unwrap();
    // $1,000 stock + $4,000 income + ($1,000 opening debt basis + $1,000
    // actual advance) allows all $7,000 current/prior basis losses.
    assert_eq!(
        (
            crate::kernel::ratio::dollars(regular.components[1].basis_allowed.get()),
            crate::kernel::ratio::dollars(regular.components[1].basis_suspended.get())
        ),
        (7_000, 0)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(regular.components[1].risk_allowed.get()),
            crate::kernel::ratio::dollars(regular.components[1].risk_suspended.get())
        ),
        (3_500, 4_500)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(alternative.components[1].basis_allowed.get()),
            crate::kernel::ratio::dollars(alternative.components[1].basis_suspended.get())
        ),
        (7_000, 0)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(alternative.components[1].risk_allowed.get()),
            crate::kernel::ratio::dollars(alternative.components[1].risk_suspended.get())
        ),
        (5_500, 2_500)
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            regular.basis.scorp.as_ref().unwrap().loans[0].principal.get()
        ),
        6_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(regular.basis.scorp.as_ref().unwrap().loans[0].basis.get()),
        0
    );
    let id = session.draft.income[0].id;
    let ids: Vec<_> = data.activities.source().unwrap().ids().collect();
    let income = FederalIncome::ScorpActivities { data: data.clone() };
    assert_eq!(crate::k1_activity::income_issue(&income), None);
    let before = session.draft.clone();
    let dirty = session.dirty;
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    documents(&mut request)[0].document = input(&income);
    let mut preview = catalog_preview_document(documents(&mut request)[0].document.clone());
    let P::PreviewDocument { id: preview_id, .. } = &mut preview else { unreachable!() };
    *preview_id = id;
    assert!(matches!(
        send(&mut session, preview),
        Reply::Response(P::IncomePreview { preview: Some(-150_000), .. })
    ));
    assert!(matches!(
        send(&mut session, crate::tests::support::client::estimate_documents_from_save(&request)),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    assert_eq!(saved.revision, before.revision + 1);
    assert_eq!(saved.next_id, before.next_id);
    assert_eq!(
        crate::k1_activity::source(&saved.income[0].income).unwrap().ids().collect::<Vec<_>>(),
        ids
    );
    let result = estimate(&saved);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    // A's $4,000 passive income releases B's $2,000 former-passive loss,
    // separately from B's $3,500 current/prior nonpassive allowed loss.
    assert_eq!(line_amount(&result, &alloc::format!("scorp_{id}_schedule_e")), -150_000);
    assert_eq!(line_amount(&result, &alloc::format!("amt_scorp_{id}_schedule_e")), -250_000);
    assert_eq!(value(&result, "agi"), 248_500);
    assert_eq!(line_amount(&result, "form_8960_net_investment_income"), 200_000);
    assert_eq!(line_amount(&result, "net_investment_income_tax"), 7_600);
    assert!(noted(&result, "WARN_K1_QBI_LOSS_NO_STATEMENT"));
    let mut cold = restart(saved.clone());
    assert_eq!(cold.draft, saved);
    let mut reopened = open_topic(&mut cold, TaxSection::PassThrough);
    assert_eq!(documents(&mut reopened)[0].document, input(&saved.income[0].income));
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
    assert_eq!(estimate(&cold.draft), result);

    scorp_cash_advance(&mut data, 0);
    let mut correction = open_topic(&mut cold, TaxSection::PassThrough);
    documents(&mut correction)[0].document = input(&FederalIncome::ScorpActivities { data });
    accepted(&send(&mut cold, correction));
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
    let unchanged = open_topic(&mut cold, TaxSection::PassThrough);
    accepted(&send(&mut cold, unchanged));
    assert_eq!(cold.draft, accepted_draft);
    assert_eq!(estimate(&cold.draft), corrected);
}

#[test]
fn scorp_cash_advance_excluded_funding_uses_actual_cent_parts_and_group_assignment() {
    let (_, base) = scorp_cash_advance_candidate(100_000);
    for (contribution, advance, excluded, allowed) in [
        (0, 100_000, 0, 3_500),
        (0, 100_000, 50_000, 3_000),
        (0, 100_000, 100_000, 2_500),
        (50_000, 100_000, 150_000, 2_500),
        // Actual cents foot: 49 + 51 = 100, all excluded. Contributions,
        // advances and excluded funding enter their existing rounded lines.
        (49, 51, 100, 2_500),
    ] {
        let mut candidate = base.clone();
        scorp_cash_advance(&mut candidate, advance);
        candidate.entity.regular.additions = contribution;
        candidate.entity.alternative.as_mut().unwrap().additions = contribution;
        let K1ComponentStatement::Entered { source } = &mut candidate.activities else {
            unreachable!()
        };
        source.risk_groups[1].investment_parts.as_mut().unwrap().contributions = contribution;
        let K1RiskSource::Scorp { regular, alternative: Some(alternative) } =
            &mut source.risk_groups[1].source
        else {
            unreachable!()
        };
        regular.excluded_funding = excluded;
        alternative.excluded_funding = excluded;
        assert_eq!(
            crate::k1_activity::income_issue(&FederalIncome::ScorpActivities {
                data: candidate.clone()
            }),
            None
        );
        let output = candidate.ordinary_source(false).unwrap();
        assert_eq!(
            crate::kernel::ratio::dollars(output.components[1].risk_allowed.get()),
            allowed,
            "{contribution}/{advance}/{excluded}"
        );
        assert_eq!(
            crate::kernel::ratio::dollars(
                candidate.ordinary_source(true).unwrap().components[1].risk_allowed.get()
            ),
            allowed + 2_000
        );
        if contribution == 49 {
            let K1ComponentStatement::Entered { source } = &mut candidate.activities else {
                unreachable!()
            };
            let K1RiskSource::Scorp { regular, .. } = &mut source.risk_groups[1].source else {
                unreachable!()
            };
            regular.excluded_funding = 101;
            // One actual cent above funding stays unknown even though both
            // excluded amounts report as $1. Never compare rounded bounds.
            assert_eq!(
                crate::k1_activity::income_issue(&FederalIncome::ScorpActivities {
                    data: candidate.clone()
                }),
                None
            );
            assert!(candidate.ordinary_source(false).is_err());
        }
    }
    let mut assigned_a = base;
    let K1ComponentStatement::Entered { source } = &mut assigned_a.activities else {
        unreachable!()
    };
    source.risk_groups[1].investment_parts.as_mut().unwrap().loan_advances = 0;
    source.risk_groups[0].investment_parts.as_mut().unwrap().loan_advances = 100_000;
    // Entity debt basis still increases, but A's actual investment cannot
    // supply B's capacity. No amount is inferred from the whole debt basis.
    let output = assigned_a.ordinary_source(false).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(output.components[1].basis_allowed.get()), 7_000);
    assert_eq!(crate::kernel::ratio::dollars(output.components[1].risk_allowed.get()), 2_500);
}

#[test]
fn scorp_cash_advance_valid_unknown_sources_save_and_keep_named_boundaries() {
    let (session, base) = scorp_cash_advance_candidate(100_000);
    for boundary in 0..10 {
        let mut session = restart(session.draft.clone());
        let mut candidate = base.clone();
        let K1ComponentStatement::Entered { source } = &mut candidate.activities else {
            unreachable!()
        };
        match boundary {
            0 => source.risk_groups[1].investment_parts = None,
            1 => source.risk_groups[1].investment_parts.as_mut().unwrap().loan_advances = 0,
            2 => candidate.entity.loans[0].alternative_basis = None,
            3 => {
                let K1RiskSource::Scorp { alternative, .. } = &mut source.risk_groups[1].source
                else {
                    unreachable!()
                };
                *alternative = None;
            }
            4 => {
                let K1RiskSource::Scorp { regular, .. } = &mut source.risk_groups[1].source else {
                    unreachable!()
                };
                regular.start = ScorpRiskStart::Unknown;
            }
            5 => {
                let K1RiskSource::Scorp { regular, .. } = &mut source.risk_groups[1].source else {
                    unreachable!()
                };
                regular.excluded_funding = 100_001;
            }
            6 => {
                candidate.entity.loans[0].regular.repayments = 100;
                candidate.entity.loans[0].regular.repayment_term =
                    ScorpNoteHolding::Single { term: GainTerm::Long };
                source.risk_groups[1].investment_parts.as_mut().unwrap().loan_repayments = 100;
            }
            7 => source.components[1].risk_group = None,
            8 => source.passive_groups[1].participation = K1ActivityParticipation::Unavailable,
            9 => source.components[1].alternative = K1StatementAmt::Unavailable,
            _ => unreachable!(),
        }
        let income = FederalIncome::ScorpActivities { data: candidate.clone() };
        assert_eq!(
            crate::k1_activity::income_issue(&income),
            None,
            "valid source boundary {boundary}"
        );
        assert!(
            candidate.ordinary_source(boundary == 2 || boundary == 3 || boundary == 9).is_err(),
            "owner boundary {boundary}"
        );
        if boundary == 2 || boundary == 3 || boundary == 9 {
            assert_eq!(
                crate::kernel::ratio::dollars(
                    candidate.ordinary_source(false).unwrap().components[1].risk_allowed.get()
                ),
                3_500
            );
        }
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        documents(&mut request)[0].document = input(&income);
        accepted(&send(&mut session, request));
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
        let unchanged = open_topic(&mut cold, TaxSection::PassThrough);
        accepted(&send(&mut cold, unchanged));
        assert_eq!(cold.draft, saved);
        assert_eq!(estimate(&cold.draft), result);
    }
}

#[test]
fn scorp_cash_advance_invalid_loan_entity_group_and_identity_edits_are_atomic() {
    let (mut session, base) = scorp_cash_advance_candidate(100_000);
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    documents(&mut request)[0].document =
        input(&FederalIncome::ScorpActivities { data: base.clone() });
    accepted(&send(&mut session, request));
    for boundary in 0..7 {
        let mut candidate = base.clone();
        let K1ComponentStatement::Entered { source } = &mut candidate.activities else {
            unreachable!()
        };
        let key = match boundary {
            0 => {
                candidate.entity.loans[0].regular.advances = -1;
                "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
            }
            1 => {
                candidate.entity.loans[0].regular.basis = 500_001;
                "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
            }
            2 => {
                candidate.entity.loans[0].alternative_basis = Some(500_001);
                "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
            }
            3 => {
                candidate.entity.regular.opening = Some(-1);
                "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
            }
            4 => {
                source.risk_groups[1].investment_parts.as_mut().unwrap().loan_advances = -1;
                "K1_ACTIVITY_SOURCE_AMOUNT_INVALID"
            }
            5 => {
                source.components[1].risk_group = Some(i64::MAX);
                "K1_ACTIVITY_SOURCE_REFERENCE_NOT_ACCEPTED"
            }
            6 => {
                source.risk_groups[1].id = i64::MAX;
                "K1_ACTIVITY_SOURCE_ID_NOT_ACCEPTED"
            }
            _ => unreachable!(),
        };
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        documents(&mut request)[0].document =
            input(&FederalIncome::ScorpActivities { data: candidate });
        assert_atomic_rejection(&mut session, request, key);
    }
}
