use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{documents, ordinary_farm_se_parent};
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{ordinary_multi_source_session, partner_source};
use crate::tests::*;

#[test]
fn farm_source_routes_actual_se_and_category_correction_through_native_lifecycle() {
    let mut session = ordinary_multi_source_session(ordinary_farm_se_parent(
        [1_800_000, 3_200_000],
        [800_000, 1_200_000],
        [2_400_000, 3_600_000],
    ));
    let id = session.draft.income[0].id;
    let result = supported(&session.draft);
    // Independent sources: box 1 $50,000; box 14A $20,000; code B gross $60,000.
    // .9235 x $20,000 = $18,470; SE tax rounds to $2,826, half $1,413.
    assert_eq!(value(&result, "taxpayer_se_farm_profit"), 20_000);
    assert_eq!(printed_line(&result, "taxpayer_se_farm_profit").1, "1a");
    assert_eq!(value(&result, "taxpayer_se_nonfarm_profit"), 0);
    assert_eq!(value(&result, "taxpayer_se_net_earnings"), 18_470);
    assert_eq!(value(&result, "taxpayer_self_employment_tax"), 2_826);
    assert_eq!(value(&result, "taxpayer_half_self_employment_tax"), 1_413);
    assert_eq!(value(&result, "agi"), 48_587);
    assert_eq!(line_amount(&result, &alloc::format!("partnership_{id}_schedule_e")), 5_000_000);
    assert_eq!(line_amount(&result, "schedule_e_farming_fishing_gross"), 6_000_000);
    assert_eq!(printed_line(&result, "schedule_e_farming_fishing_gross").1, "42");
    assert_eq!(value(&result, "qbi_deduction"), 0);
    assert!(noted(&result, "WARN_K1_QBI_NO_STATEMENT"));
    let projection = partner_source(&session).ordinary_self_employment().unwrap().unwrap();
    assert_eq!((projection.category, projection.gross), (0, 6_000_000));
    // AMT retains the same independently reported farm character for related deductions.
    let alternative = partner_source(&session).ordinary_source(true).unwrap();
    let alternative_se = alternative.self_employment.unwrap();
    assert_eq!((alternative_se.category, alternative_se.gross), (0, 6_000_000));
    assert_eq!(alternative_se.net.earnings, 20_000);

    let farm_accepted = session.draft.clone();
    let mut farm_cold = restart(farm_accepted.clone());
    let mut farm_reopened =
        crate::tests::support::client::income_pages::open_document_center(&mut farm_cold);
    assert_eq!(
        documents(&mut farm_reopened)[0].document,
        expected_input(&farm_accepted, &farm_accepted.income[0])
    );
    accepted(&send_fixture(&mut farm_cold, farm_reopened));
    assert_eq!(farm_cold.draft, farm_accepted);
    assert_eq!(supported(&farm_cold.draft), result);
    session = farm_cold;

    let saved = session.draft.clone();
    let dirty = session.dirty;
    let old_ids: Vec<_> =
        crate::k1_activity::source(&saved.income[0].income).unwrap().ids().collect();
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);

    let crate::K1Document::K1PartnershipActivities(page) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    // Correct both actual document controls from code B to code C, retaining every amount.
    page.person.as_mut().unwrap().se_gross =
        Some(vec![PartnerSeGross::Nonfarm { gross: 6_000_000 }]);
    let K1ComponentStatementPage::Entered { source } = &mut page.activities.statement else {
        unreachable!()
    };
    for component in &mut source.components {
        let K1StatementSe::Partnership { source } = &mut component.self_employment else {
            unreachable!()
        };
        source.gross =
            source.gross.as_ref().map(|gross| PartnerSeGross::Nonfarm { gross: gross.source().1 });
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
    assert_eq!(session.draft.income[0].id, id);
    assert_eq!(
        crate::k1_activity::source(&session.draft.income[0].income)
            .unwrap()
            .ids()
            .collect::<Vec<_>>(),
        old_ids
    );
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "taxpayer_se_nonfarm_profit"), 20_000);
    assert_eq!(printed_line(&corrected, "taxpayer_se_nonfarm_profit").1, "2");
    for omitted in [
        "taxpayer_se_farm_profit",
        "schedule_e_farming_fishing_gross",
        "farming_fishing_gross_income_return_year",
    ] {
        assert!(corrected.lines.iter().all(|line| line.id != omitted));
    }
    assert_eq!(value(&corrected, "taxpayer_self_employment_tax"), 2_826);
    assert_eq!(value(&corrected, "agi"), 48_587);
    assert_eq!(partner_source(&session).ordinary_self_employment().unwrap().unwrap().category, 1);
    let accepted_source = expected_input(&session.draft, &session.draft.income[0]);
    let mut cold = restart(session.draft.clone());
    let mut reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    assert_eq!(documents(&mut reopened)[0].document, accepted_source);
    accepted(&send_fixture(&mut cold, reopened));
    assert_eq!(cold.draft, session.draft);
    assert_eq!(supported(&cold.draft), corrected);
}

#[test]
fn farm_optional_method_uses_actual_code_b_gross_and_existing_engine() {
    let mut session = ordinary_multi_source_session(ordinary_farm_se_parent(
        [200_000, 250_000],
        [40_000, 60_000],
        [400_000, 500_000],
    ));
    session.draft.screening.as_mut().unwrap().optional_se =
        vec![SeElection { owner: Owner::Taxpayer, farm: true, nonfarm: None }];
    let result = supported(&session.draft);
    // $9,000 actual gross x 2/3 = $6,000 optional earnings, not $1,000 actual SE profit.
    assert_eq!(value(&result, "optional_se_taxpayer_farm"), 6_000);
    assert_eq!(value(&result, "taxpayer_se_optional_earnings"), 6_000);
    assert_eq!(value(&result, "taxpayer_se_net_earnings"), 6_000);
    assert_eq!(value(&result, "taxpayer_self_employment_tax"), 918);
    assert_eq!(value(&result, "taxpayer_half_self_employment_tax"), 459);
    assert_eq!(value(&result, "schedule_e_farming_fishing_gross"), 9_000);
    assert!(result.lines.iter().all(|line| line.id != "taxpayer_se_farm_profit"));
}

#[test]
fn farmer_denominator_keeps_positive_native_schedule_e_beside_code_b_gross() {
    for (operating, expected_total, qualifies) in
        [([200_000, 250_000], 1_350_000, true), ([200_000, 250_100], 1_350_100, false)]
    {
        let session = ordinary_multi_source_session(ordinary_farm_se_parent(
            operating,
            [40_000, 60_000],
            [400_000, 500_000],
        ));
        let result = supported(&session.draft);
        let schedule_e = line_amount(
            &result,
            &alloc::format!("partnership_{}_schedule_e", session.draft.income[0].id),
        );
        assert_eq!(schedule_e, expected_total - 900_000);
        assert_eq!(line_amount(&result, "farming_fishing_gross_income_return_year"), 900_000);
        assert_eq!(line_amount(&result, "gross_income_return_year"), expected_total);
        let source_net =
            alloc::collections::BTreeMap::from([(session.draft.income[0].id, schedule_e / 100)]);
        let actual = crate::underpayment::two_thirds_return_year(
            &session.draft,
            &result,
            Some(0),
            &source_net,
        )
        .unwrap()
        .unwrap();
        assert_eq!((actual.farming, actual.gross), (900_000, expected_total));
        // $9,000/$13,500 is exactly two-thirds; $9,000/$13,501 is below it.
        assert_eq!(actual.qualifies(), qualifies);
    }
}

#[test]
fn valid_unavailable_farm_sources_withhold_complete_estimate_instead_of_zero_gross() {
    for boundary in 0..8 {
        let mut value = ordinary_farm_se_parent(
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
            0 => se.gross = None,
            1 => value.person.as_mut().unwrap().se_gross = None,
            2 => {
                se.gross = Some(PartnerSeGross::Nonfarm { gross: 2_400_000 });
                value.person.as_mut().unwrap().se_gross = Some(vec![
                    PartnerSeGross::Nonfarm { gross: 2_400_000 },
                    PartnerSeGross::Farm { gross: 3_600_000 },
                ]);
            }
            3 => source.passive_groups[0].participation = K1ActivityParticipation::Unavailable,
            4 => {
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
            5 => {
                se.guaranteed_services = 100;
                value.person.as_mut().unwrap().guaranteed_services = Some(100);
            }
            6 => {
                value.person.as_mut().unwrap().se_gross =
                    Some(vec![PartnerSeGross::Farm { gross: 6_000_001 }])
            }
            7 => {
                value.person.as_mut().unwrap().se_gross =
                    Some(vec![PartnerSeGross::Nonfarm { gross: 6_000_000 }])
            }
            _ => unreachable!(),
        }
        // These are valid source records whose financial projection remains unavailable.
        assert_eq!(value.rule(), None, "boundary {boundary}");
        assert!(value.ordinary_self_employment().is_err(), "boundary {boundary}");
        let session = ordinary_multi_source_session(value);
        let result = estimate(&session.draft);
        assert!(result.tax.is_none(), "boundary {boundary}");
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE"),
            "boundary {boundary}: {:?}",
            result.issues
        );
    }
}

#[test]
fn duplicate_person_gross_controls_are_rejected_atomically_by_native_save() {
    let mut session = ordinary_multi_source_session(ordinary_farm_se_parent(
        [200_000, 250_000],
        [40_000, 60_000],
        [400_000, 500_000],
    ));
    let before = session.draft.clone();
    let dirty = session.dirty;
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let crate::K1Document::K1PartnershipActivities(page) = &mut documents(&mut request)[0].document
    else {
        unreachable!()
    };
    page.person.as_mut().unwrap().se_gross = Some(vec![
        PartnerSeGross::Farm { gross: 400_000 },
        PartnerSeGross::Farm { gross: 500_000 },
    ]);
    assert!(matches!(send_fixture(&mut session, request), Reply::Response(P::Problem { .. })));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
}
