use super::*;

#[test]
fn k1res_d6_actual_selected_allowance_forwards_to_rental_sources_without_amt_fallback() {
    let mut session = d6_ptp_source_session(k1_1065_continuing(
        10_000_000,
        ScorpRiskStart::Detailed {
            capacity: 10_000_000,
            profit: 0,
            loss: 0,
        },
    ));
    let original = session.draft.clone();
    let dirty = session.dirty;
    let calculation =
        estimate_documents_from_save(&open_topic(&mut session, TaxSection::PassThrough));
    accepted(&send(&mut session, calculation));
    assert_eq!(session.draft, original);
    assert_eq!(session.dirty, dirty);
    // 2026-10-15: regular carry uses main's gross-only source; no original-year
    // result is asserted. The separately authored AMT carry retains 2024.
    for (system, expected_a, expected_net) in [
        (crate::Section179IncomeSystem::Regular, 700_000, 20_000),
        (crate::Section179IncomeSystem::Alternative, 600_000, 21_000),
    ] {
        let prepared = d6_selected_source(&session.draft, system).unwrap();
        assert_eq!(prepared.income.k1[&2].allowed_cents, expected_a);
        assert_eq!(prepared.income.k1[&3].allowed_cents, 300_000);
        let mut output = FederalResult::empty(session.draft.revision);
        let result = crate::extended_rules::rentals_for_home_with_section179(
            &session.draft,
            100_000,
            &mut output,
            system == crate::Section179IncomeSystem::Alternative,
            [100_000; 2],
            (&crate::care::CareDeductions::default(), Some(&prepared)),
        );
        assert_eq!(
            result.section179_allowed_cents(&session.draft, 2),
            expected_a
        );
        assert_eq!(result.section179_allowed_cents(&session.draft, 3), 300_000);
        assert_eq!(result.income, expected_net);
        assert_eq!(result.income_by_id[&2], 20_000 - expected_a / 100);
        assert_eq!(result.income_by_id[&3], 7_000);
        assert_eq!(result.income_by_id.values().sum::<i64>(), expected_net);
        assert_eq!(session.draft, original);
        assert_eq!(session.dirty, dirty);
    }
    let reopened = open_topic(&mut session, TaxSection::PassThrough);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, original);
    let mut cold = restart(original.clone());
    let reopened = open_topic(&mut cold, TaxSection::PassThrough);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, original);
    // 2026-10-16: the production coordinator now supplies this invocation's
    // selected allocation to PAL and consumes only its actual PTP report inputs.
    // The original source figures above remain independent native oracles.
    let result = supported(&cold.draft);
    assert_eq!(value(&result, "section179_deduction"), 10_000);
    for (prefix, first) in [("", 13_000), ("amt_", 14_000)] {
        assert_eq!(
            value(
                &result,
                &alloc::format!("{prefix}partnership_2_activity_net")
            ),
            first
        );
        assert_eq!(
            value(
                &result,
                &alloc::format!("{prefix}partnership_3_activity_net")
            ),
            7_000
        );
    }
    assert!(result.issues.iter().all(|issue| !matches!(
        issue.display.key.as_str(),
        "ISSUE_SECTION179_PUBLICLY_TRADED" | "ISSUE_SECTION179_PASSIVE_ACTIVITY"
    )));
    assert_eq!(supported(&session.draft), result);
    assert_eq!(estimate(&restart(cold.draft.clone()).draft), result);
    assert_eq!(cold.draft, original);
}

#[test]
fn k1res_d6_missing_amt_income_carry_remains_unavailable_after_normal_save() {
    let mut session = d6_ptp_source_session(k1_1065_continuing(
        10_000_000,
        ScorpRiskStart::Detailed {
            capacity: 10_000_000,
            profit: 0,
            loss: 0,
        },
    ));
    let mut correction = open_topic(&mut session, TaxSection::PassThrough);
    let P::SaveIncomes { documents, .. } = &mut correction else {
        unreachable!()
    };
    let IncomeDocument::K1Partnership(page) = &mut documents.documents[0].document else {
        unreachable!()
    };
    page.more.amt_section179_income_carry = None;
    accepted(&send(&mut session, correction));
    let accepted_draft = session.draft.clone();
    assert!(d6_selected_source(&accepted_draft, crate::Section179IncomeSystem::Regular).is_ok());
    assert_eq!(
        d6_selected_source(&accepted_draft, crate::Section179IncomeSystem::Alternative),
        Err(crate::Section179PreparedIssue::K1Availability(
            2,
            crate::K1Section179BasisIssue::AlternativeIncomeCarryUnavailable,
        ))
    );
    let result = estimate(&accepted_draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(result.issues.iter().any(|issue| {
        issue.display.key == "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE"
            // The public estimate resolves the Income label from the saved
            // source (interview::resolved_issue_context); identity and manual
            // correction routing must remain this actual K1, ID2.
            && matches!(&issue.context, crate::IssueContext::Corrective { source, sections }
                if sections.is_empty() && matches!(source.as_ref(), crate::IssueContext::Income { id: 2, .. }))
    }));
    assert!(result.tax.is_none() && result.refund.is_none() && result.return_lines.is_empty());
    let dirty = session.dirty;
    let calculation =
        estimate_documents_from_save(&open_topic(&mut session, TaxSection::PassThrough));
    accepted(&send(&mut session, calculation));
    assert_eq!(session.draft, accepted_draft);
    assert_eq!(session.dirty, dirty);
    let mut cold = restart(accepted_draft.clone());
    let reopened = open_topic(&mut cold, TaxSection::PassThrough);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, accepted_draft);
    assert_eq!(estimate(&cold.draft), result);
}

#[test]
fn k1res_d6_mixed_direct_sources_keep_native_routing_property_and_amt_stages() {
    let session = d6_ptp_source_session(k1_1065_continuing(
        10_000_000,
        ScorpRiskStart::Detailed {
            capacity: 10_000_000,
            profit: 0,
            loss: 0,
        },
    ));
    let (session, business, farm) = crate::tests::support::client::d6_mixed_direct_sources(session);
    let original = session.draft.clone();
    // The canonical native source recipes independently give:
    // C:40000 +1500 patronage +2500 grant -3000 elected -1400 MACRS -1500 office.
    // F:10000 raised -3000 feed +1500 patronage +4000 agriculture payment.
    // The existing installment parcel recognizes20000 (16000 ordinary/4000
    // section1231); it is a property source, never another farm receipt.
    let result = supported(&original);
    assert_eq!(value(&result, "section179_deduction"), 13_000);
    assert_eq!(
        value(
            &result,
            &alloc::format!("business_{business}_section179_deduction")
        ),
        3_000
    );
    assert_eq!(
        value(&result, &alloc::format!("business_{business}_net_profit")),
        38_100
    );
    for (suffix, expected) in [
        ("gain", 20_000),
        ("ordinary", 16_000),
        ("section1231", 4_000),
    ] {
        assert_eq!(
            value(&result, &alloc::format!("farm_{farm}_land_0_{suffix}")),
            expected
        );
    }
    assert_eq!(
        value(&result, &alloc::format!("business_{business}_depreciation")),
        4_400
    );
    assert_eq!(
        value(&result, &alloc::format!("business_{business}_home_office")),
        1_500
    );
    assert_eq!(
        value(
            &result,
            &alloc::format!("business_{business}_tentative_profit")
        ),
        39_600
    );
    assert_eq!(
        value(&result, &alloc::format!("farm_{farm}_net_profit")),
        12_500
    );
    // AMT changes the same7000 MACRS basis from1400 to1050. Neither the
    // fully allowed3000 election nor the1500 office changes: add350 once.
    assert_eq!(value(&result, "amt_business_depreciation_adjustment"), 350);
    assert_eq!(value(&result, "partnership_2_activity_net"), 13_000);
    assert_eq!(value(&result, "amt_partnership_2_activity_net"), 14_000);
    assert_eq!(estimate(&restart(original.clone()).draft), result);
    assert_eq!(session.draft, original);
    // A consumed PTP allocation does not waive unrelated required property
    // history. Keep the original source facts and their unavailable boundary.
    let mut incomplete = original.clone();
    incomplete.screening.as_mut().unwrap().section1231 = None;
    let unavailable = estimate(&incomplete);
    assert_eq!(unavailable.state, ResultState::Unsupported);
    assert!(unavailable.issues.iter().any(|issue| issue.display.key
        == "ISSUE_SECTION1231_ADD__BUSINESS_PROPERTY_LOSSES_FROM_PRIOR_FIVE_YEARS__ONCE_F5B687DD"));
    assert!(unavailable.tax.is_none() && unavailable.return_lines.is_empty());
    assert_eq!(estimate(&restart(incomplete.clone()).draft), unavailable);
}

#[test]
fn k1res_d6_percentage_depletion_boundary_keeps_cost_and_unselected_support() {
    use crate::tests::support::client::d6_business_boundary_source;
    for (selected, percentage, repayment) in [
        (true, true, false),
        (true, true, true),
        (true, false, false),
        (false, true, false),
    ] {
        let session = if selected {
            d6_ptp_source_session(k1_1065_continuing(
                10_000_000,
                ScorpRiskStart::Detailed {
                    capacity: 10_000_000,
                    profit: 0,
                    loss: 0,
                },
            ))
        } else {
            Interview::new(TEST_USER, complete(FederalStatus::Single))
        };
        let depletion = if percentage {
            RoyaltyDepletion::Percentage {
                adjusted_basis: 200_000,
                recoverable_units: 10,
                units_sold: 1,
                carryover: 0,
                amt_carryover: 0,
            }
        } else {
            RoyaltyDepletion::Cost {
                adjusted_basis: 200_000,
                recoverable_units: 10,
                units_sold: 1,
            }
        };
        let (mut session, id) = d6_business_boundary_source(
            session,
            10_000,
            true,
            Some(BusinessDepletion {
                kind: RoyaltyKind::OilGas,
                depletion,
            }),
        );
        if repayment {
            // Complete existing historical source records cause the ordinary
            // claim-of-right pass to infer current capital history and rerun.
            crate::tests::support::builders::add_repayment_inferred_capital(&mut session.draft);
        }
        let before = session.draft.clone();
        let dirty = session.dirty;
        let request = open_topic(&mut session, TaxSection::SelfEmployment);
        accepted(&send(&mut session, estimate_documents_from_save(&request)));
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        let result = estimate(&before);
        if repayment {
            assert!(noted(
                &result,
                "NOTE_CURRENT_CARRYOVER_INPUTS_REUSE_COMPLETE_ORIGINAL_2025_REPAYMENT_2F25B188"
            ));
            assert!(
                before
                    .screening
                    .as_ref()
                    .unwrap()
                    .capital_history
                    .is_empty()
            );
        }
        if selected && percentage {
            // Dispatcher 2026-10-16: this original owner ordering is parked
            // for phaseC; no legacy percentage limit substitutes for Some.
            assert_eq!(result.state, ResultState::Unsupported);
            assert!(result.issues.iter().any(|issue| {
                issue.display.key == "ISSUE_SECTION179_PERCENTAGE_DEPLETION"
                    && matches!(&issue.context, crate::IssueContext::Corrective { source, sections }
                        if sections.is_empty() && matches!(source.as_ref(), crate::IssueContext::Income { id: actual, .. } if *actual == id))
            }));
            assert!(result.tax.is_none() && result.return_lines.is_empty());
        } else {
            assert_eq!(result, supported(&before));
            // Same independently authored deposit:2000 basis /10units ×1
            // costs200; ordinary percentage is15% of10000 =1500.
            assert_eq!(
                value(&result, &alloc::format!("business_{id}_depletion")),
                if percentage { 1_500 } else { 200 }
            );
        }
        assert_eq!(estimate(&restart(before.clone()).draft), result);
        assert_eq!(session.draft, before);
    }
}

#[test]
fn k1res_d6_direct_at_risk_boundary_preserves_native_omission_and_known_controls() {
    use crate::tests::support::client::d6_business_boundary_source;
    for (fully_at_risk, profit) in [(false, -1_000), (true, -1_000), (false, 1_000)] {
        let session = d6_ptp_source_session(k1_1065_continuing(
            10_000_000,
            ScorpRiskStart::Detailed {
                capacity: 10_000_000,
                profit: 0,
                loss: 0,
            },
        ));
        let (session, id) = d6_business_boundary_source(session, profit, fully_at_risk, None);
        let before = session.draft.clone();
        assert!(before.valid_stored());
        assert!(crate::section179_return::needed(&before).unwrap());
        let prepared =
            crate::section179_return::prepare(&before, crate::Section179IncomeSystem::Regular);
        if !fully_at_risk && profit < 0 {
            assert!(
                matches!(prepared, Err(crate::section179_return::Issue::Activity(actual,
                "ISSUE_SECTION179_DIRECT_AT_RISK", _)) if actual == id)
            );
            // The existing public omission runs before selection on this
            // ordinary loss. Exercise the bound admission directly above;
            // never fabricate a retained loss to bypass the original owner.
            let result = supported(&before);
            assert!(
                result
                    .review_notes
                    .iter()
                    .any(|note| note.display.key == "WARN_BUSINESS_LOSS_LEFT_OUT")
            );
            assert!(
                result
                    .issues
                    .iter()
                    .all(|issue| issue.display.key != "ISSUE_SECTION179_DIRECT_AT_RISK")
            );
        } else {
            assert!(prepared.is_ok());
            let result = supported(&before);
            assert_eq!(
                value(&result, &alloc::format!("business_{id}_net_profit")),
                profit
            );
        }
        assert_eq!(session.draft, before);
        assert_eq!(restart(before.clone()).draft, before);
    }
}
