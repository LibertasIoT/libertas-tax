use crate::k1_activity::{
    K1ComponentQualified, K1ComponentStatement, K1CurrentSourceParts, K1ReportedQbiStatement,
    K1StatementSe,
};
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{
    add_qbi_gross_business, documents, graph, ordinary_multi_current_qbi_parent,
    ordinary_positive_se_parent, ordinary_positive_se_qbi_parent,
};
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::support::client::k1res::{ordinary_se_qbi_session, partner_source};
use crate::tests::*;

#[test]
fn independent_native_current_qbi_uses_actual_block_gross_halves() {
    let session = ordinary_se_qbi_session(ordinary_positive_se_qbi_parent());
    let parent = session.draft.income[0].id;
    let source = crate::k1_activity::source(&session.draft.income[0].income).unwrap();
    // 2026-10-18 correction: se_from_net rounds half of reported tax $283
    // to $142. Schedule SE line 13 takes 50% of line 12, preserving the
    // existing reported-line rounding: https://www.irs.gov/pub/irs-dft/f1040sse--dft.pdf
    let allocation = crate::partnership_qbi::half_se_by_gross(&session.draft, [142, 0]).unwrap();
    // One sequential residual allocation of $142 over actual source weights
    // [4500,4500,750,2250] gives [53,53,9,27], independent of box1/netSE/QBI.
    assert_eq!(allocation.native_blocks[&(parent, source.qbi_blocks[0].id)], 53);
    assert_eq!(allocation.native_blocks[&(parent, source.qbi_blocks[1].id)], 9);
    assert_eq!(allocation.sources[&parent], 62);
    let result = supported(&session.draft);
    assert_eq!(line_amount(&result, &alloc::format!("partnership_{parent}_schedule_e")), 600_000);
    assert_eq!(value(&result, "taxpayer_se_net_earnings"), 1_847);
    assert_eq!(value(&result, "taxpayer_self_employment_tax"), 283);
    assert_eq!(value(&result, "taxpayer_half_self_employment_tax"), 142);
    assert_eq!(value(&result, "agi"), 65_858);
    assert_eq!(value(&result, "qbi_deduction"), 788); // (3000+1000-53-9) x20%.
    let mut reversed = session.draft.clone();
    let FederalIncome::PartnershipActivities { data } = &mut reversed.income[0].income else {
        unreachable!()
    };
    let K1ComponentStatement::Entered { source } = &mut data.activities else { unreachable!() };
    source.components.reverse();
    source.qbi_blocks.reverse();
    let reordered = crate::partnership_qbi::half_se_by_gross(&reversed, [142, 0]).unwrap();
    assert_eq!(reordered.sources, allocation.sources);
    assert_eq!(reordered.native_blocks, allocation.native_blocks);
    assert_eq!(value(&supported(&reversed), "qbi_deduction"), 788);
}

#[test]
fn native_qbi_correction_preview_revision_and_cold_reopen_preserve_identity() {
    let mut session = ordinary_se_qbi_session(ordinary_positive_se_qbi_parent());
    let saved = session.draft.clone();
    let dirty = session.dirty;
    let parent = saved.income[0].id;
    let source_ids: Vec<_> =
        crate::k1_activity::source(&saved.income[0].income).unwrap().ids().collect();
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);

    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    let source = graph(&mut documents(&mut request)[0].document);
    let K1ReportedQbiStatement::Partnership { statement: Some(statement) } =
        &mut source.qbi_blocks[0].statement
    else {
        unreachable!()
    };
    statement.income += 50_000;
    let K1ComponentQualified::Partnership { current_income, .. } =
        &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
    else {
        unreachable!()
    };
    *current_income += 50_000;
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
    assert_eq!(session.draft.income[0].id, parent);
    assert_eq!(
        crate::k1_activity::source(&session.draft.income[0].income)
            .unwrap()
            .ids()
            .collect::<Vec<_>>(),
        source_ids
    );
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "qbi_deduction"), 888);
    assert_eq!(value(&corrected, "agi"), 65_858);
    assert_eq!(value(&corrected, "taxpayer_self_employment_tax"), 283);
    let accepted_draft = session.draft.clone();
    let accepted_document = expected_input(&accepted_draft, &accepted_draft.income[0]);
    let mut cold = restart(accepted_draft.clone());
    let mut reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    assert_eq!(documents(&mut reopened)[0].document, accepted_document);
    accepted(&send_fixture(&mut cold, reopened));
    assert_eq!(cold.draft, accepted_draft);
    assert_eq!(supported(&cold.draft), corrected);
}

#[test]
fn current_native_qualified_gross_shares_the_other_business_denominator_once() {
    let mut session = ordinary_se_qbi_session(ordinary_positive_se_qbi_parent());
    let parent = session.draft.income[0].id;
    let business = add_qbi_gross_business(&mut session.draft, 1_200_000, 100_000);
    assert!(parent < business);
    let source = crate::k1_activity::source(&session.draft.income[0].income).unwrap();
    let result = supported(&session.draft);
    assert_eq!(value(&result, "taxpayer_half_self_employment_tax"), 212);
    // $212 / $24,000 actual gross, sequential source residuals:
    // [4500,4500,750,2250,12000,0] => [40,40,7,20,105,0].
    let allocation = crate::partnership_qbi::half_se_by_gross(&session.draft, [212, 0]).unwrap();
    assert_eq!(allocation.native_blocks[&(parent, source.qbi_blocks[0].id)], 40);
    assert_eq!(allocation.native_blocks[&(parent, source.qbi_blocks[1].id)], 7);
    assert_eq!(allocation.sources[&parent], 47);
    assert_eq!(allocation.sources[&business], 105);
    assert_eq!(value(&result, "qbi_deduction"), 970); // (3000+1000+1000-40-7-105)x20%.
}

#[test]
fn zero_qualified_native_parent_retains_other_business_penny_residual() {
    let parent_source =
        ordinary_positive_se_parent([400_000, 200_000], [120_000, 80_000], [1_000_001, 1_000_001]);
    let mut session = ordinary_se_qbi_session(parent_source);
    let parent = session.draft.income[0].id;
    let business = add_qbi_gross_business(&mut session.draft, 1_000_001, 100_000);
    assert!(parent < business);
    let result = supported(&session.draft);
    assert_eq!(value(&result, "taxpayer_half_self_employment_tax"), 212);
    // Preserve [0,2000002,1000001,0] => [0,141,71,0]. Splitting the
    // nonqualified parent into two components would incorrectly leave C $70.
    let allocation = crate::partnership_qbi::half_se_by_gross(&session.draft, [212, 0]).unwrap();
    assert!(allocation.native_blocks.is_empty());
    assert_eq!(allocation.sources[&parent], 0);
    assert_eq!(allocation.sources[&business], 71);
    assert_eq!(value(&result, "qbi_deduction"), 186); // (1000-71)x20%.
}

#[test]
fn missing_same_component_se_gross_stays_unavailable_after_native_save() {
    let mut session = ordinary_se_qbi_session(ordinary_positive_se_qbi_parent());
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    let source = graph(&mut documents(&mut request)[0].document);
    let K1StatementSe::Partnership { source } = &mut source.components[0].self_employment else {
        unreachable!()
    };
    source.gross = None;
    accepted(&send_fixture(&mut session, request));
    assert!(matches!(partner_source(&session).ordinary_source(false),
        Err(error) if error == crate::k1_activity::ordinary::UNAVAILABLE));
    assert!(estimate(&session.draft).tax.is_none());
    let mut cold = restart(session.draft.clone());
    let mut reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    let source = graph(&mut documents(&mut reopened)[0].document);
    assert!(matches!(&source.components[0].self_employment,
        K1StatementSe::Partnership { source } if source.gross.is_none()));
}

#[test]
fn positive_qualified_gross_needs_its_current_statement_and_actual_se_source() {
    for no_statement in [false, true] {
        let mut value = ordinary_positive_se_qbi_parent();
        let K1ComponentStatement::Entered { source } = &mut value.activities else {
            unreachable!()
        };
        if no_statement {
            source.qbi_blocks[0].statement =
                K1ReportedQbiStatement::Partnership { statement: None };
            let K1ComponentQualified::Partnership { current_income, .. } =
                &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
            else {
                unreachable!()
            };
            *current_income = 0;
        } else {
            source.components[0].self_employment = K1StatementSe::NotApplicable;
            let person = value.person.as_mut().unwrap();
            person.reported_se = Some(80_000);
            person.se_gross = Some(vec![PartnerSeGross::Nonfarm { gross: 300_000 }]);
        }
        assert_eq!(value.rule(), None);
        assert!(value.ordinary_source(false).is_err());
    }
    // Explicit zero qualified gross remains a legitimate current QBI source.
    let mut value = ordinary_positive_se_qbi_parent();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    for block in &mut source.qbi_blocks {
        let K1ComponentQualified::Partnership { records, .. } =
            &mut block.allocations.as_mut().unwrap()[0].qualified
        else {
            unreachable!()
        };
        records.qualified_se_gross = 0;
    }
    assert!(value.ordinary_source(false).is_ok());
    assert!(ordinary_multi_current_qbi_parent().ordinary_source(false).is_ok());
}

#[test]
fn current_native_qbi_needs_actual_gross_operating_controls() {
    let mut value = ordinary_positive_se_qbi_parent();
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    source.components[0].qualified_current_controls.ordinary = None;
    assert_eq!(value.rule(), None);
    assert!(value.ordinary_source(false).is_err());
}

#[test]
fn native_current_qbi_does_not_admit_farm_limited_loss_or_retained_sources() {
    for boundary in 0..7 {
        let mut value = ordinary_positive_se_qbi_parent();
        let K1ComponentStatement::Entered { source } = &mut value.activities else {
            unreachable!()
        };
        match boundary {
            0 => {
                let K1StatementSe::Partnership { source } =
                    &mut source.components[0].self_employment
                else {
                    unreachable!()
                };
                source.gross = Some(PartnerSeGross::Farm { gross: 900_000 });
                value.person.as_mut().unwrap().se_gross = Some(vec![
                    PartnerSeGross::Farm { gross: 900_000 },
                    PartnerSeGross::Nonfarm { gross: 300_000 },
                ]);
            }
            1 => value.person.as_mut().unwrap().limited_for_se = Some(true),
            2 => {
                source.components[0].regular.as_mut().unwrap().operating = -400_000;
                source.components[0].qualified_current_controls.ordinary =
                    Some(K1CurrentSourceParts { income: 0, loss: 400_000 });
                let K1ReportedQbiStatement::Partnership { statement: Some(statement) } =
                    &mut source.qbi_blocks[0].statement
                else {
                    unreachable!()
                };
                statement.income = -300_000;
                let K1ComponentQualified::Partnership { current_income, .. } =
                    &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
                else {
                    unreachable!()
                };
                *current_income = -300_000;
                value.current.ordinary = -200_000;
                value.alternative_current = Some(value.current.clone());
            }
            3 => {
                let K1StatementSe::Partnership { source } =
                    &mut source.components[0].self_employment
                else {
                    unreachable!()
                };
                source.history.basis = None;
            }
            4 => {
                source.components[0].regular.as_mut().unwrap().operating_carry.basis = Some(100);
                let K1ComponentQualified::Partnership { records, .. } =
                    &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
                else {
                    unreachable!()
                };
                records.history.as_mut().unwrap().basis.push(loss_year(2025, 1, 1, 1, 1));
            }
            5 => value.person.as_mut().unwrap().reported_se = None,
            _ => {
                for component in &mut source.components {
                    let K1StatementSe::Partnership { source } = &mut component.self_employment
                    else {
                        unreachable!()
                    };
                    source.gross = source
                        .gross
                        .as_ref()
                        .map(|gross| PartnerSeGross::Farm { gross: gross.source().1 });
                }
                value.person.as_mut().unwrap().se_gross =
                    Some(vec![PartnerSeGross::Farm { gross: 1_200_000 }]);
            }
        }
        assert_eq!(value.rule(), None, "boundary {boundary}");
        assert!(value.ordinary_source(false).is_err(), "boundary {boundary}");
    }
}

#[test]
fn over_gross_and_duplicate_block_sources_refuse_native_save_atomically() {
    let mut session = ordinary_se_qbi_session(ordinary_positive_se_qbi_parent());
    let saved = session.draft.clone();
    for duplicate in [false, true] {
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        let source = graph(&mut documents(&mut request)[0].document);
        if duplicate {
            source.qbi_blocks[1].id = source.qbi_blocks[0].id;
        } else {
            let K1ComponentQualified::Partnership { records, .. } =
                &mut source.qbi_blocks[0].allocations.as_mut().unwrap()[0].qualified
            else {
                unreachable!()
            };
            records.qualified_se_gross = 900_001;
        }
        error(send_fixture(&mut session, request));
        assert_eq!(session.draft, saved);
    }
}
