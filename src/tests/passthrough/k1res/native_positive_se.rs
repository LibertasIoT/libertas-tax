use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{documents, ordinary_positive_se_parent};
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{ordinary_multi_source_session, partner_source};
use crate::tests::*;

#[test]
fn native_positive_se_has_independent_amounts_and_normal_correction_lifecycle() {
    let source = ordinary_positive_se_parent(
        [1_800_000, 3_200_000],
        [800_000, 1_200_000],
        [2_400_000, 3_600_000],
    );
    let mut session = ordinary_multi_source_session(source);
    let id = session.draft.income[0].id;
    let result = supported(&session.draft);
    // Box 1 is $50,000; independently reported box 14A is $20,000.
    // $20,000 x .9235 = $18,470; .124 + .029 gives $2,826 tax, half $1,413.
    assert_eq!(line_amount(&result, &alloc::format!("partnership_{id}_schedule_e")), 5_000_000);
    assert_eq!(value(&result, "taxpayer_se_nonfarm_profit"), 20_000);
    assert_eq!(value(&result, "taxpayer_se_combined_profit"), 20_000);
    assert_eq!(value(&result, "taxpayer_se_net_earnings"), 18_470);
    assert_eq!(value(&result, "taxpayer_self_employment_tax"), 2_826);
    assert_eq!(value(&result, "taxpayer_half_self_employment_tax"), 1_413);
    assert_eq!(value(&result, "agi"), 48_587);
    assert!(noted(&result, "WARN_K1_QBI_NO_STATEMENT"));
    assert_eq!(value(&result, "qbi_deduction"), 0);
    let alternative = partner_source(&session).ordinary_source(true).unwrap();
    let earnings = &alternative.self_employment.as_ref().unwrap().net;
    assert_eq!((earnings.earnings, earnings.earnings_cents), (20_000, 2_000_000));

    let saved = session.draft.clone();
    let dirty = session.dirty;
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);

    let crate::K1Document::K1PartnershipActivities(page) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    page.person.as_mut().unwrap().reported_se = Some(1_000_000);
    let K1ComponentStatementPage::Entered { source } = &mut page.activities.statement else {
        unreachable!()
    };
    for (component, reported) in source.components.iter_mut().zip([300_000, 700_000]) {
        let K1StatementSe::Partnership { source } = &mut component.self_employment else {
            unreachable!()
        };
        source.reported = Some(reported);
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
    assert_eq!(session.draft.revision, saved.revision + 1);
    assert_eq!(session.draft.next_id, saved.next_id);
    let old_ids: Vec<_> =
        crate::k1_activity::source(&saved.income[0].income).unwrap().ids().collect();
    assert_eq!(
        crate::k1_activity::source(&session.draft.income[0].income)
            .unwrap()
            .ids()
            .collect::<Vec<_>>(),
        old_ids
    );
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "taxpayer_se_combined_profit"), 10_000);
    assert_eq!(value(&corrected, "taxpayer_se_net_earnings"), 9_235);
    assert_eq!(line_amount(&corrected, &alloc::format!("partnership_{id}_schedule_e")), 5_000_000);
    let accepted_source = expected_input(&session.draft, &session.draft.income[0]);
    let mut cold = restart(session.draft.clone());
    let mut reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    assert_eq!(documents(&mut reopened)[0].document, accepted_source);
    accepted(&send_fixture(&mut cold, reopened));
    assert_eq!(cold.draft, session.draft);
    assert_eq!(supported(&cold.draft), corrected);
}

#[test]
fn native_positive_se_shares_the_existing_owner_wage_base() {
    let mut session = ordinary_multi_source_session(ordinary_positive_se_parent(
        [1_800_000, 3_200_000],
        [800_000, 1_200_000],
        [2_400_000, 3_600_000],
    ));
    add(&mut session.draft, wage(0, Owner::Taxpayer, 18_450_000, 0));
    let result = supported(&session.draft);
    // The W-2 consumes the Social Security wage base. Only 2.9% of $18,470 remains.
    assert_eq!(value(&result, "taxpayer_self_employment_tax"), 536);
    assert_eq!(value(&result, "taxpayer_half_self_employment_tax"), 268);
}

#[test]
fn native_nonfarm_optional_method_uses_reported_gross_in_the_existing_engine() {
    let mut session = ordinary_multi_source_session(ordinary_positive_se_parent(
        [1_800_000, 3_200_000],
        [40_000, 60_000],
        [400_000, 500_000],
    ));
    session.draft.screening.as_mut().unwrap().optional_se = vec![SeElection {
        owner: Owner::Taxpayer,
        farm: false,
        nonfarm: Some(NonfarmSeHistory {
            earnings_third_prior_year: 40_000,
            earnings_second_prior_year: 40_000,
            earnings_prior_year: 0,
            prior_years: vec![],
        }),
    }];
    let result = supported(&session.draft);
    // Independently reported gross $9,000 gives optional earnings $6,000,
    // replacing actual $1,000 profit's $923.50 reduction, not the $50,000 box 1.
    assert_eq!(value(&result, "optional_se_taxpayer_nonfarm"), 6_000);
    assert_eq!(value(&result, "taxpayer_se_optional_earnings"), 6_000);
    assert_eq!(value(&result, "taxpayer_se_net_earnings"), 6_000);
    assert_eq!(value(&result, "taxpayer_self_employment_tax"), 918);
    assert_eq!(value(&result, "taxpayer_half_self_employment_tax"), 459);
}

#[test]
fn native_nonqualified_gross_remains_in_another_business_qbi_denominator() {
    let mut session = ordinary_multi_source_session(ordinary_positive_se_parent(
        [1_800_000, 3_200_000],
        [800_000, 1_200_000],
        [4_000_000, 5_000_000],
    ));
    add(&mut session.draft, income(cash_business(10_000)));
    let business = session.draft.income.last().unwrap().id;
    let parent = session.draft.income[0].id;
    let result = supported(&session.draft);
    // Combined actual SE profit $30,000 -> tax $4,239, half $2,120.
    // C's $10,000 qualified gross / ($10,000 + $90,000) assigns $212
    // to C. Native NoStatement gross is in the denominator, qualified zero.
    assert_eq!(value(&result, "taxpayer_half_self_employment_tax"), 2_120);
    let allocation =
        crate::partnership_qbi::half_se_by_gross(&session.draft, [2_120, 0]).unwrap().sources;
    assert_eq!((allocation[&business], allocation[&parent]), (212, 0));
    assert_eq!(value(&result, "qbi_deduction"), 1_958);
}

#[test]
fn native_passive_se_excludes_only_its_actual_reported_subset_from_niit() {
    let mut source = ordinary_positive_se_parent(
        [1_800_000, 3_200_000],
        [800_000, 1_200_000],
        [2_400_000, 3_600_000],
    );
    let K1ComponentStatement::Entered { source: graph } = &mut source.activities else {
        unreachable!()
    };
    let K1ActivityParticipation::Partnership { facts, .. } =
        &mut graph.passive_groups[0].participation
    else {
        unreachable!()
    };
    *facts = PartnerParticipation::Limited {
        minutes: 0,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    };
    let mut session = ordinary_multi_source_session(source);
    add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
    let result = supported(&session.draft);
    // Passive box 1 $18,000 less its actual SE $8,000 = $10,000 NII.
    // The other $32,000 is nonpassive and was already excluded by its group.
    assert_eq!(value(&result, "form_8960_net_investment_income"), 10_000);
    assert_eq!(value(&result, "net_investment_income_tax"), 380);
}

#[test]
fn native_se_rounds_equivalent_component_cents_once_at_the_parent() {
    let session = ordinary_multi_source_session(ordinary_positive_se_parent(
        [100_000, 100_000],
        [49, 49],
        [200_000, 300_000],
    ));
    let source = partner_source(&session).ordinary_self_employment().unwrap().unwrap();
    assert_eq!((source.net.earnings, source.net.earnings_cents), (1, 98));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "taxpayer_se_combined_profit"), 1);
    assert_eq!(value(&result, "taxpayer_self_employment_tax"), 0);
}

#[test]
fn native_positive_se_unavailable_controls_and_excluded_cohorts_stay_unknown() {
    for boundary in 0..10 {
        let mut value = ordinary_positive_se_parent(
            [1_800_000, 3_200_000],
            [800_000, 1_200_000],
            [2_400_000, 3_600_000],
        );
        let K1ComponentStatement::Entered { source } = &mut value.activities else {
            unreachable!()
        };
        let K1StatementSe::Partnership { source: se } = &mut source.components[0].self_employment
        else {
            unreachable!()
        };
        match boundary {
            0 => se.reported = None,
            1 => se.gross = None,
            2 => value.person.as_mut().unwrap().reported_se = None,
            3 => value.person.as_mut().unwrap().se_gross = None,
            4 => value.person.as_mut().unwrap().reported_se = Some(2_000_001),
            5 => {
                value.person.as_mut().unwrap().se_gross =
                    Some(vec![PartnerSeGross::Nonfarm { gross: 6_000_001 }])
            }
            6 => value.person.as_mut().unwrap().limited_for_se = Some(true),
            7 => {
                se.gross = Some(PartnerSeGross::Farm { gross: 2_400_000 });
                value.person.as_mut().unwrap().se_gross = Some(vec![
                    PartnerSeGross::Farm { gross: 2_400_000 },
                    PartnerSeGross::Nonfarm { gross: 3_600_000 },
                ]);
            }
            8 => {
                se.history.basis = Some(100);
                value.person.as_mut().unwrap().se_history.basis = Some(100);
                source.components[0].regular.as_mut().unwrap().operating_carry.basis = Some(100);
                let K1ComponentQualified::Partnership { records, .. } =
                    &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
                else {
                    unreachable!()
                };
                records.history.as_mut().unwrap().basis = vec![loss_year(2025, 1, 0, 1, 0)];
            }
            9 => {
                source.qbi_blocks[0].statement = K1ReportedQbiStatement::Partnership {
                    statement: Some(partner_qbi_statement(1_000)),
                };
                let K1ComponentQualified::Partnership { current_income, .. } =
                    &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
                else {
                    unreachable!()
                };
                *current_income = 100_000;
            }
            _ => unreachable!(),
        }
        assert_eq!(value.rule(), None, "boundary {boundary}");
        let session = ordinary_multi_source_session(value);
        if boundary == 9 {
            // 2026-10-20: A current statement with actual operating controls is
            // supported. Zero qualified SE gross assigns none of the half-SE
            // deduction to its $1,000 QBI (Reg. 1.199A-6(b)(3)). The 2026
            // active-QBI minimum raises the $200 calculation to $400:
            // https://www.irs.gov/pub/irs-dft/i8995--dft.pdf
            let result = supported(&session.draft);
            assert_eq!(crate::tests::value(&result, "qbi_deduction"), 400);
            assert_eq!(crate::tests::value(&result, "taxpayer_self_employment_tax"), 2_826);
            assert_eq!(crate::tests::value(&result, "taxpayer_half_self_employment_tax"), 1_413);
            continue;
        }
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
    }
}

#[test]
fn printed_depletion_uses_actual_activity_answers_and_preserves_regular_se_on_restart() {
    use crate::tests::support::client::income_pages::{open_document_center, open_topic};
    let source = ordinary_positive_se_parent(
        [1_800_000, 3_200_000],
        [1_800_000, 3_200_000],
        [2_400_000, 3_600_000],
    );
    let mut session = ordinary_multi_source_session(source);
    let original = supported(&session.draft);
    let regular_se = value(&original, "taxpayer_self_employment_tax");
    let regular_half = value(&original, "taxpayer_half_self_employment_tax");
    let mut paper = open_document_center(&mut session);
    let crate::K1Document::K1PartnershipActivities(page) = &mut documents(&mut paper)[0].document
    else {
        unreachable!()
    };
    page.amt = [120_000, -20_000]
        .into_iter()
        .map(|amount| crate::K1CodeRow {
            code: "C".into(),
            amount: Some(amount),
            text: "Synthetic signed activity statement".into(),
            statement: true,
        })
        .collect();
    accepted(&send(&mut session, paper));
    let id = session.draft.income[0].id;
    let mut topic = open_topic(&mut session, TaxSection::PassThrough);
    let page = crate::tests::support::client::k1res::depletion_page(&mut topic, id);
    assert_eq!(page.printed_total, Some(100_000));
    assert_eq!(page.sources.len(), 2);
    assert!(page.allocations.iter().all(|row| row.adjustment.is_none()));
    let ids: Vec<_> = page.sources.iter().map(|source| source.id).collect();
    accepted(&send(&mut session, topic));
    assert!(partner_source(&session).depletion_sources.is_none());
    assert!(estimate(&session.draft).tax.is_none());
    let mut partial = open_topic(&mut session, TaxSection::PassThrough);
    crate::tests::support::client::k1res::depletion_page(&mut partial, id).allocations[0]
        .adjustment = Some(150_000);
    accepted(&send(&mut session, partial));
    assert_eq!(
        partner_source(&session).depletion_sources,
        Some(vec![crate::K1DepletionSourceAllocation { source: ids[0], adjustment: 150_000 },])
    );
    assert!(estimate(&session.draft).tax.is_none());
    session = restart(session.draft.clone());
    let mut topic = open_topic(&mut session, TaxSection::PassThrough);
    let page = crate::tests::support::client::k1res::depletion_page(&mut topic, id);
    assert_eq!(page.allocations[0].adjustment, Some(150_000));
    assert_eq!(page.allocations[1].adjustment, None);
    page.allocations[1].adjustment = Some(-40_000);
    let before = session.draft.clone();
    let problem = send(&mut session, topic);
    let Reply::Response(P::Problem { field: Some(field), .. }) = problem else {
        panic!("{problem:?}");
    };
    let target: alloc::string::String =
        libertas::SchemaPath::root().field("allocations").key(ids[0]).field("adjustment").into();
    assert!(field.ends_with(target.trim_start_matches('[')), "{field}");
    assert_eq!(session.draft, before);
    let mut topic = open_topic(&mut session, TaxSection::PassThrough);
    let page = crate::tests::support::client::k1res::depletion_page(&mut topic, id);
    page.allocations = ids
        .iter()
        .copied()
        .zip([150_000, -50_000])
        .map(|(source, adjustment)| crate::K1DepletionSourceAllocationPage {
            source,
            adjustment: Some(adjustment),
        })
        .collect();
    page.printed_total = None;
    page.sources.clear();
    accepted(&send(&mut session, topic));
    let actual = partner_source(&session);
    assert_eq!(actual.current.ordinary, 5_000_000);
    assert_eq!(
        actual.amt.iter().map(|row| row.amount.unwrap()).collect::<Vec<_>>(),
        vec![120_000, -20_000]
    );
    let alternative = actual.ordinary_source(true).unwrap();
    let earnings = &alternative.self_employment.as_ref().unwrap().net;
    assert_eq!((earnings.earnings, earnings.earnings_cents), (51_000, 5_100_000));
    let result = supported(&session.draft);
    assert_eq!(value(&result, "form6251_line2d_depletion"), 1_000);
    assert!(!result.lines.iter().any(|row| matches!(
        row.id.as_str(),
        "form6251_line2m_k1_passive" | "form6251_line2n_k1_loss_limits"
    )));
    assert_eq!(value(&result, "taxpayer_self_employment_tax"), regular_se);
    assert_eq!(value(&result, "taxpayer_half_self_employment_tax"), regular_half);
    let accepted_source = actual.clone();
    let mut cold = restart(session.draft.clone());
    assert_eq!(partner_source(&cold), &accepted_source);
    assert_eq!(supported(&cold.draft), result);
    let mut reopened = open_document_center(&mut cold);
    let crate::K1Document::K1PartnershipActivities(page) =
        &mut documents(&mut reopened)[0].document
    else {
        unreachable!()
    };
    page.amt.reverse();
    accepted(&send(&mut cold, reopened));
    assert_eq!(partner_source(&cold).depletion_sources, accepted_source.depletion_sources);
    let topic = open_topic(&mut cold, TaxSection::PassThrough);
    let P::SaveIncomes { followups, .. } = topic else { unreachable!() };
    let page = followups
        .into_iter()
        .find_map(|row| match row.answer {
            crate::DocumentAnswer::K1DepletionSources { value } => Some(value),
            _ => None,
        })
        .unwrap();
    assert_eq!(page.sources.iter().map(|source| source.id).collect::<Vec<_>>(), ids);
    assert_eq!(page.record(), accepted_source.depletion_sources);
}

#[test]
fn offsetting_depletion_stays_source_bound_and_separate_records_are_not_adjusted_again() {
    let mut source = ordinary_positive_se_parent(
        [1_800_000, 3_200_000],
        [1_800_000, 3_200_000],
        [2_400_000, 3_600_000],
    );
    source.amt = [50, -50]
        .into_iter()
        .map(|amount| crate::K1CodeRow {
            code: "C".into(),
            amount: Some(amount),
            text: "Synthetic offsetting adjustments".into(),
            statement: true,
        })
        .collect();
    assert!(source.depletion_page().is_some());
    assert_eq!(source.depletion_allocations().unwrap_err().key, "K1_DEPLETION_SOURCE_NEEDED");
    source.depletion_sources = Some(
        source
            .depletion_choices()
            .iter()
            .zip([50, -50])
            .map(|(source, adjustment)| crate::K1DepletionSourceAllocation {
                source: source.id,
                adjustment,
            })
            .collect(),
    );
    let adjusted = source.ordinary_source(true).unwrap();
    assert_eq!(
        adjusted.components.iter().map(|row| row.income.get()).collect::<Vec<_>>(),
        vec![1_800_050, 3_199_950]
    );
    let crate::K1ComponentStatement::Entered { source: activities } = &mut source.activities else {
        unreachable!()
    };
    for component in &mut activities.components {
        component.alternative = crate::K1StatementAmt::Separate {
            amounts: component.regular.as_ref().unwrap().clone(),
        };
    }
    let separate = source.ordinary_source(true).unwrap();
    assert_eq!(
        separate.components.iter().map(|row| row.income.get()).collect::<Vec<_>>(),
        vec![1_800_000, 3_200_000]
    );
    assert_eq!(separate.self_employment.as_ref().unwrap().net.earnings_cents, 5_000_000);
}
