use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{amounts, documents, input, ordinary_multi_parent};
use crate::tests::support::client::k1res::ordinary_multi_parent_session;
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::*;

#[test]
fn ordinary_multi_entity_basis_risk_groups_passive_pool_and_native_lifecycle() {
    let mut value = ordinary_multi_parent();
    let regular = value.ordinary_source(false).unwrap();
    let alternative = value.ordinary_source(true).unwrap();
    // Outside basis: $1,000 opening + $4,000 income, shared once.
    // B has $6,000 current + $1,000 retained loss: $5,000 allowed,
    // $2,000 held. Its separate $3,000 risk investment permits $3,000
    // of the $5,000 basis-allowed + $1,000 risk carry: $3,000 held.
    assert_eq!(crate::kernel::ratio::dollars(regular.basis.ending.get()), 0);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(regular.components[1].basis_allowed.get()),
            crate::kernel::ratio::dollars(regular.components[1].basis_suspended.get())
        ),
        (5_000, 2_000)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(regular.components[1].risk_allowed.get()),
            crate::kernel::ratio::dollars(regular.components[1].risk_suspended.get())
        ),
        (3_000, 3_000)
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
        (5_000, 3_000)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(regular.groups[0].passive),
            crate::kernel::ratio::dollars(regular.groups[1].nonpassive),
            crate::kernel::ratio::dollars(regular.groups[1].passive)
        ),
        (4_000, -3_000, -2_000)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(alternative.groups[1].nonpassive),
            crate::kernel::ratio::dollars(alternative.groups[1].passive)
        ),
        (-5_000, -1_000)
    );

    let mut cents = value.clone();
    let K1ComponentStatement::Entered { source } = &mut cents.activities else { unreachable!() };
    source.components[0].regular.as_mut().unwrap().operating += 49;
    let loss = source.components[1].regular.as_mut().unwrap();
    loss.operating -= 51;
    *loss.operating_carry.basis.as_mut().unwrap() += 49;
    *loss.operating_carry.risk.as_mut().unwrap() += 49;
    let K1RiskSource::Partnership { regular, .. } = &mut source.risk_groups[1].source else {
        unreachable!()
    };
    regular.prior_loss += 49;
    cents.current.ordinary -= 2;
    cents.alternative_current.as_mut().unwrap().ordinary += 49;
    let K1ComponentQualified::Partnership { records, .. } =
        &mut source.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified
    else {
        unreachable!()
    };
    let history = records.history.as_mut().unwrap();
    for row in [&mut history.basis[0], &mut history.risk[0]] {
        row.original += 49;
        row.remaining += 49;
    }
    let rounded = cents.ordinary_source(false).unwrap();
    // The same basis stage preserves $6,000.51 current and $1,000.49 prior
    // source loss against $5,000.49 available basis: $2,000.51 remains.
    assert_eq!(rounded.components[1].basis_suspended.get(), 200_051);
    assert_eq!(crate::kernel::ratio::dollars(rounded.components[1].risk_allowed.get()), 3_000);
    let K1ComponentStatement::Entered { source } = &mut cents.activities else { unreachable!() };
    source.components[1].regular.as_mut().unwrap().operating += 2;
    cents.current.ordinary += 2;
    assert_eq!(cents.ordinary_source(false).unwrap().components[1].basis_suspended.get(), 200_049);

    let mut shared_risk = value.clone();
    let K1ComponentStatement::Entered { source } = &mut shared_risk.activities else {
        unreachable!()
    };
    source.components[1].risk_group = Some(source.risk_groups[0].id);
    source.risk_groups[0].source = source.risk_groups[1].source.clone();
    source.risk_groups.truncate(1);
    let shared_risk = shared_risk.ordinary_source(false).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(shared_risk.components[1].risk_allowed.get()),
            crate::kernel::ratio::dollars(shared_risk.components[1].risk_suspended.get())
        ),
        (6_000, 0)
    );
    assert_eq!(shared_risk.components.iter().map(|v| v.id).collect::<Vec<_>>(), vec![11, 12]);

    let mut session = ordinary_multi_parent_session(&mut value);
    add(&mut session.draft, wage(0, Owner::Taxpayer, 6_000_000, 0));
    session = restart(session.draft);
    let before = session.draft.clone();
    let dirty = session.dirty;
    let id = before.income[0].id;
    let income =
        FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value.clone()) };
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    documents(&mut request)[0].document = input(&income);
    let running = crate::tests::support::client::income_pages::candidate_save_review(
        &mut session,
        crate::tests::support::client::estimate_documents_from_save(&request),
    );
    assert!(matches!(running, Reply::Response(P::Review { .. })), "{running:?}");
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send_fixture(&mut session, request));
    assert_eq!(session.draft.revision, before.revision + 2);
    assert_eq!(session.draft.next_id, before.next_id);
    assert_eq!(session.draft.income[1], before.income[1]);
    let result = estimate(&session.draft);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    // A's passive $4,000 releases B's former passive $2,000. B's
    // nonpassive current loss is $3,000: Schedule E -$1,000, AGI $59,000.
    assert_eq!(line_amount(&result, &alloc::format!("partnership_{id}_schedule_e")), -100_000);
    assert_eq!(line_amount(&result, &alloc::format!("amt_partnership_{id}_schedule_e")), -200_000);
    assert_eq!(line_amount(&result, "agi"), 5_900_000);
    assert!(noted(&result, "WARN_K1_QBI_LOSS_NO_STATEMENT"));
    let mut investment = session.draft.clone();
    investment.income[1] = wage(investment.income[1].id, Owner::Taxpayer, 25_000_000, 0);
    let investment = estimate(&investment);
    assert!(investment.tax.is_some(), "{:?}", investment.issues);
    // Passive A $4,000 less $2,000 prior passive loss is investment
    // income; the nonpassive $3,000 loss is excluded. 3.8% x $2,000 = $76.
    assert_eq!(line_amount(&investment, "form_8960_net_investment_income"), 200_000);
    assert_eq!(line_amount(&investment, "net_investment_income_tax"), 7_600);
    let saved = session.draft.clone();
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    assert_eq!(documents(&mut request)[0].document, expected_input(&saved, &saved.income[0]));
    let preview_dirty = session.dirty;

    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, preview_dirty);
    let mut cold = restart(saved.clone());
    let mut reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    assert_eq!(documents(&mut reopened)[0].document, expected_input(&saved, &saved.income[0]));
    assert_eq!(estimate(&cold.draft), result);

    // Correct only regular B; the separate AMT source remains unchanged.
    let crate::K1Document::K1PartnershipActivities(page) =
        &mut documents(&mut reopened)[0].document
    else {
        unreachable!()
    };
    page.current.ordinary = 500_000;
    let K1ComponentStatementPage::Entered { source } = &mut page.activities.statement else {
        unreachable!()
    };
    source.components[1].regular.as_mut().unwrap().operating = 100_000;
    accepted(&send_fixture(&mut cold, reopened));
    assert_eq!(cold.draft.revision, saved.revision + 2);
    assert_eq!(cold.draft.next_id, saved.next_id);
    let corrected = estimate(&cold.draft);
    assert!(corrected.tax.is_some(), "{:?}", corrected.issues);
    assert_eq!(line_amount(&corrected, &alloc::format!("partnership_{id}_schedule_e")), 100_000);
    assert_eq!(
        line_amount(&corrected, &alloc::format!("amt_partnership_{id}_schedule_e")),
        -200_000
    );

    // Established groups do not license unavailable sources, a PTP, a
    // disposition, other characters, QBI or SE to bypass their owners.
    for boundary in 0..12 {
        let mut unsupported = value.clone();
        let K1ComponentStatement::Entered { source } = &mut unsupported.activities else {
            unreachable!()
        };
        match boundary {
            0 => unsupported.entity.publicly_traded = true,
            1 => unsupported.entity.alternative = None,
            2 => source.components[1].regular.as_mut().unwrap().operating_carry.risk = None,
            3 => source.passive_groups[1].participation = K1ActivityParticipation::Unavailable,
            4 => source.passive_groups[1].disposed = true,
            5 => {
                source.components[1].kind = K1StatementKind::OtherRental;
                unsupported.current.ordinary = 400_000;
                unsupported.current.other_rental = -600_000;
                let alternative = unsupported.alternative_current.as_mut().unwrap();
                alternative.ordinary = 400_000;
                alternative.other_rental = -600_000;
            }
            6 => source.qbi_blocks[0].statement = K1ReportedQbiStatement::Unavailable,
            7 => source.components[0].self_employment = K1StatementSe::Unavailable,
            8 => source.risk_groups[1].source = K1RiskSource::Unavailable,
            9 => source.qbi_blocks[1].allocations = Some(vec![]),
            10 => {
                let K1ComponentQualified::Partnership { records, .. } =
                    &mut source.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified
                else {
                    unreachable!()
                };
                let row = &mut records.history.as_mut().unwrap().basis[0];
                row.qualified = 1;
                row.remaining_qualified = 1;
            }
            _ => {}
        }
        let mut candidate = saved.clone();
        candidate.income[0].income =
            FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(unsupported) };
        if boundary == 11 {
            candidate.setup = complete(FederalStatus::Separate).setup;
            candidate.setup.as_mut().unwrap().community_property = community_arizona();
            for entry in &mut candidate.income {
                entry.community = CommunityShare::Separate;
            }
        }
        let refused = estimate(&candidate);
        assert!(refused.tax.is_none(), "boundary {boundary}");
        assert_eq!(
            refused
                .issues
                .iter()
                .filter(|issue| issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
                .count(),
            1,
            "boundary {boundary}: {:?}",
            refused.issues
        );
    }
    let mut refused_session = restart(saved.clone());
    let mut unavailable = value;
    unavailable.entity.publicly_traded = true;
    let refused_income =
        FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(unavailable) };
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut refused_session);
    documents(&mut request)[0].document = input(&refused_income);
    assert!(matches!(
        crate::tests::support::client::income_pages::candidate_save_review(
            &mut refused_session,
            crate::tests::support::client::estimate_documents_from_save(&request)
        ),
        Reply::Response(P::Review { .. })
    ));
    assert_eq!(refused_session.draft, saved);
    accepted(&send_fixture(&mut refused_session, request));
    assert!(estimate(&refused_session.draft).tax.is_none());
    let refused_saved = refused_session.draft.clone();
    let mut refused_cold = restart(refused_saved.clone());
    let mut reopened =
        crate::tests::support::client::income_pages::open_document_center(&mut refused_cold);
    assert_eq!(
        documents(&mut reopened)[0].document,
        crate::tests::support::builders::k1res::expected_fixture_input(
            &refused_saved,
            &refused_income
        )
    );
    assert_eq!(refused_cold.draft, refused_saved);
}

#[test]
fn native_entities_keep_subdollar_basis_and_risk_through_save_reopen_and_correction() {
    for scorp in [false, true] {
        let (mut session, mut income) = if scorp {
            let (session, value) =
                crate::tests::support::client::k1res::scorp_ordinary_multi_session();
            (session, FederalIncome::ScorpActivities { data: value })
        } else {
            let mut value = ordinary_multi_parent();
            let session = ordinary_multi_parent_session(&mut value);
            (session, FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) })
        };
        let source = match &mut income {
            FederalIncome::PartnershipActivities { data } => {
                data.current.ordinary = -149;
                data.alternative_current.as_mut().unwrap().ordinary = -149;
                data.entity.regular.opening = Some(50);
                data.entity.alternative.as_mut().unwrap().opening = Some(149);
                &mut data.activities
            }
            FederalIncome::ScorpActivities { data } => {
                data.current.ordinary = -149;
                data.alternative_current.as_mut().unwrap().ordinary = -149;
                data.entity.regular.opening = Some(50);
                data.entity.alternative.as_mut().unwrap().opening = Some(149);
                data.entity.loans.clear();
                &mut data.activities
            }
            _ => unreachable!(),
        };
        let K1ComponentStatement::Entered { source } = source else { unreachable!() };
        for (index, component) in source.components.iter_mut().enumerate() {
            let mut value = amounts(if index == 0 { 0 } else { -149 }, 0);
            if scorp {
                value.items = crate::tests::support::builders::k1res::scorp_items();
                value.capital_carry = crate::tests::support::builders::k1res::scorp_carry();
            }
            component.regular = Some(value.clone());
            component.alternative = K1StatementAmt::Separate { amounts: value };
        }
        for (index, group) in source.risk_groups.iter_mut().enumerate() {
            let capacity = if index == 0 { 0 } else { 50 };
            let alternative_capacity = if index == 0 { 0 } else { 149 };
            group.source = if scorp {
                let mut regular = scorp_risk_history();
                regular.start = ScorpRiskStart::Detailed { capacity, profit: 0, loss: 0 };
                regular.prior_loss = 0;
                regular.deducted = 0;
                let mut alternative = regular.clone();
                alternative.start =
                    ScorpRiskStart::Detailed { capacity: alternative_capacity, profit: 0, loss: 0 };
                K1RiskSource::Scorp { regular, alternative: Some(alternative) }
            } else {
                let mut regular = partner_risk_record();
                regular.start = ScorpRiskStart::Detailed { capacity, profit: 0, loss: 0 };
                let mut alternative = regular.clone();
                alternative.start =
                    ScorpRiskStart::Detailed { capacity: alternative_capacity, profit: 0, loss: 0 };
                K1RiskSource::Partnership { regular, alternative: Some(alternative) }
            };
        }
        for block in &mut source.qbi_blocks {
            block.allocations = Some(vec![]);
        }
        let selected = |income: &FederalIncome, alternative| match income {
            FederalIncome::PartnershipActivities { data } => data.ordinary_source(alternative),
            FederalIncome::ScorpActivities { data } => data.ordinary_source(alternative),
            _ => unreachable!(),
        };
        let regular = selected(&income, false).unwrap();
        let alternative = selected(&income, true).unwrap();
        assert_eq!(
            (
                regular.components[1].basis_allowed.get(),
                regular.components[1].basis_suspended.get()
            ),
            (50, 99)
        );
        assert_eq!(
            (regular.components[1].risk_allowed.get(), regular.components[1].risk_suspended.get()),
            (50, 0)
        );
        assert_eq!(
            (
                alternative.components[1].basis_allowed.get(),
                alternative.components[1].basis_suspended.get()
            ),
            (149, 0)
        );
        assert_eq!(alternative.components[1].risk_allowed.get(), 149);
        let mut unknown = income.clone();
        match &mut unknown {
            FederalIncome::PartnershipActivities { data } => {
                data.entity.regular.opening = None;
                data.entity.regular.complete = false;
            }
            FederalIncome::ScorpActivities { data } => {
                data.entity.regular.opening = None;
                data.entity.regular.complete = false;
            }
            _ => unreachable!(),
        }
        assert!(selected(&unknown, false).is_err());
        assert_eq!(selected(&unknown, true).unwrap().components[1].basis_allowed.get(), 149);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&income);
        accepted(&send_fixture(&mut session, request));
        let saved = session.draft.clone();
        let result = estimate(&saved);
        assert!(result.tax.is_some(), "{:?}", result.issues);
        let mut separate_rows = saved.clone();
        let activities = match &mut separate_rows.income[0].income {
            FederalIncome::PartnershipActivities { data } => {
                data.current.ordinary = 98;
                data.alternative_current.as_mut().unwrap().ordinary = 98;
                &mut data.activities
            }
            FederalIncome::ScorpActivities { data } => {
                data.current.ordinary = 98;
                data.alternative_current.as_mut().unwrap().ordinary = 98;
                &mut data.activities
            }
            _ => unreachable!(),
        };
        let K1ComponentStatement::Entered { source } = activities else { unreachable!() };
        for component in &mut source.components {
            component.regular.as_mut().unwrap().operating = 49;
            let K1StatementAmt::Separate { amounts } = &mut component.alternative else {
                unreachable!()
            };
            amounts.operating = 49;
        }
        assert_eq!(
            selected(&separate_rows.income[0].income, false).unwrap().basis.ending.get(),
            148
        );
        let report = estimate(&separate_rows);
        assert!(report.tax.is_some(), "{:?}", report.issues);
        // Distinct passive/nonpassive Schedule E rows each report 49c as zero;
        // the source basis still receives all 98c without a balancing allocation.
        let row = if scorp { "scorp" } else { "partnership" };
        assert_eq!(
            line_amount(&report, &alloc::format!("{row}_{}_schedule_e", saved.income[0].id)),
            0
        );
        let mut cold = restart(saved.clone());
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(documents(&mut reopened)[0].document, expected_input(&saved, &saved.income[0]));
        assert_eq!(estimate(&cold.draft), result);
        match &mut documents(&mut reopened)[0].document {
            crate::K1Document::K1PartnershipActivities(page) => {
                page.entity.regular.as_mut().unwrap().opening = Some(0)
            }
            crate::K1Document::K1ScorpActivities(page) => {
                page.entity.regular.as_mut().unwrap().opening = Some(0)
            }
            _ => unreachable!(),
        }
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(cold.draft.next_id, saved.next_id);
        assert_eq!(cold.draft.income[0].id, saved.income[0].id);
        let corrected = selected(&cold.draft.income[0].income, false).unwrap();
        assert_eq!(
            (
                corrected.components[1].basis_allowed.get(),
                corrected.components[1].basis_suspended.get()
            ),
            (0, 149)
        );
        assert_eq!(
            selected(&cold.draft.income[0].income, true).unwrap().components[1].basis_allowed.get(),
            149
        );
    }
}

#[test]
fn component_capital_actual_history_shares_basis_and_risk_once_in_each_system() {
    use crate::tests::support::builders::k1res::{
        component_capital_source, entity_exempt_other_output,
    };
    for scorp in [false, true] {
        for scale in [1, 100] {
            let income = component_capital_source(scorp, scale);
            for (alternative, allowed, basis_held, risk_held, reported) in [
                (false, 350, 400, 350, if scale == 1 { -4 } else { -350 }),
                (true, 700, 50, 350, if scale == 1 { -7 } else { -700 }),
            ] {
                let output =
                    entity_exempt_other_output(&income, alternative).unwrap_or_else(|error| {
                        panic!("scorp={scorp}, scale={scale}, alternative={alternative}: {error:?}")
                    });
                assert_eq!(output.basis.ending.get(), 0);
                let component = &output.components[0];
                assert_eq!(
                    (component.id, component.risk_group, component.passive_group),
                    (11, 21, 31)
                );
                for capital in &component.capital {
                    assert_eq!(capital.income.get(), 0);
                    assert_eq!(
                        capital.basis_allowed.get(),
                        (if alternative { 950 } else { 600 }) * scale
                    );
                    assert_eq!(capital.basis_suspended.get(), basis_held * scale);
                    assert_eq!(capital.risk_allowed.get(), allowed * scale);
                    assert_eq!(capital.risk_suspended.get(), risk_held * scale);
                    assert_eq!(
                        crate::kernel::ratio::dollars(-capital.risk_allowed.get()),
                        reported
                    );
                }
                // Portfolio classes retain their own output. The ordinary
                // current and prior PAL inputs and qualified release stay zero.
                let ordinary = if alternative { 700 * scale } else { 0 };
                assert_eq!(
                    (
                        component.income.get(),
                        component.basis_allowed.get(),
                        component.risk_allowed.get()
                    ),
                    (ordinary, 0, 0)
                );
                assert_eq!(
                    (
                        output.groups[0].current,
                        output.groups[0].prior,
                        output.groups[0].passive,
                        output.groups[0].nonpassive
                    ),
                    (ordinary, 0, ordinary, 0)
                );
                assert!(output.qualified.is_empty());
                assert!(output.current_qualified.is_empty());
            }
        }
    }
}

#[test]
fn component_capital_unknowns_and_actual_group_reconciliation_remain_unavailable() {
    use crate::tests::support::builders::k1res::{
        component_capital_source, entity_exempt_other_fields, entity_exempt_other_output,
    };
    for scorp in [false, true] {
        let original = component_capital_source(scorp, 1);
        for boundary in 0..6 {
            let mut income = original.clone();
            if boundary == 5 {
                match &mut income {
                    FederalIncome::PartnershipActivities { data } => data.health_insurance = None,
                    FederalIncome::ScorpActivities { data } => data.health_insurance = None,
                    _ => unreachable!(),
                }
            }
            let (source, _, _) = entity_exempt_other_fields(&mut income);
            match boundary {
                0 => source.components[0].risk_group = None,
                1 => {
                    source.components[0].regular.as_mut().unwrap().capital_carry =
                        K1StatementCapitalCarry::Unavailable
                }
                2 => match &mut source.components[0].regular.as_mut().unwrap().capital_carry {
                    K1StatementCapitalCarry::Partnership { amounts } => amounts.short.basis = None,
                    K1StatementCapitalCarry::Scorp { amounts } => amounts.basis_short = None,
                    _ => unreachable!(),
                },
                3 => match &mut source.risk_groups[0].source {
                    K1RiskSource::Partnership { regular, .. } => regular.prior_loss += 1,
                    K1RiskSource::Scorp { regular, .. } => regular.prior_loss += 1,
                    _ => unreachable!(),
                },
                4 => source.components[0].alternative = K1StatementAmt::Unavailable,
                5 => {}
                _ => unreachable!(),
            }
            assert!(
                entity_exempt_other_output(&income, boundary == 4).is_err(),
                "scorp={scorp}, boundary={boundary}"
            );
            if boundary == 4 {
                assert!(
                    entity_exempt_other_output(&income, false).is_ok(),
                    "scorp={scorp}, unavailable AMT preserves REG"
                );
            }
        }
    }
}

#[test]
fn component_capital_zero_basis_and_zero_risk_preserve_separate_carry_stages() {
    use crate::tests::support::builders::k1res::{
        component_capital_source, entity_exempt_other_fields, entity_exempt_other_output,
        entity_exempt_other_rule,
    };
    for scorp in [false, true] {
        let mut income = component_capital_source(scorp, 1);
        match &mut income {
            FederalIncome::PartnershipActivities { data } => {
                data.entity.regular.contributions = 0;
                data.entity.alternative.as_mut().unwrap().contributions = 0;
            }
            FederalIncome::ScorpActivities { data } => {
                data.entity.regular.additions = 0;
                data.entity.alternative.as_mut().unwrap().additions = 0;
            }
            _ => unreachable!(),
        }
        let (source, _, _) = entity_exempt_other_fields(&mut income);
        source.risk_groups[0].investment_parts.as_mut().unwrap().contributions = 0;
        match &mut source.risk_groups[0].source {
            K1RiskSource::Partnership { regular, alternative } => {
                regular.excluded_contributions = 0;
                *alternative = Some(regular.clone());
            }
            K1RiskSource::Scorp { regular, alternative } => {
                regular.excluded_funding = 0;
                *alternative = Some(regular.clone());
            }
            _ => unreachable!(),
        }
        assert_eq!(entity_exempt_other_rule(&income), None);
        let output = entity_exempt_other_output(&income, false)
            .unwrap_or_else(|error| panic!("scorp={scorp}: {error:?}"));
        for capital in &output.components[0].capital {
            assert_eq!(
                (
                    capital.basis_allowed.get(),
                    capital.basis_suspended.get(),
                    capital.risk_allowed.get(),
                    capital.risk_suspended.get()
                ),
                (0, 1_000, 0, 100)
            );
        }
    }
}

#[test]
fn component_capital_positive_income_offsets_bound_group_loss_without_entering_pal() {
    use crate::tests::support::builders::k1res::{
        component_capital_source, entity_exempt_other_fields, entity_exempt_other_output,
        entity_exempt_other_rule, scorp_carry,
    };
    for scorp in [false, true] {
        let mut income = component_capital_source(scorp, 1);
        match &mut income {
            FederalIncome::PartnershipActivities { data } => {
                data.entity.regular.contributions = 0;
                data.entity.alternative.as_mut().unwrap().contributions = 0;
            }
            FederalIncome::ScorpActivities { data } => {
                data.entity.regular.additions = 0;
                data.entity.alternative.as_mut().unwrap().additions = 0;
            }
            _ => unreachable!(),
        }
        let (source, current, alternative) = entity_exempt_other_fields(&mut income);
        current.portfolio.short = 600;
        *alternative = Some(current.clone());
        let values = source.components[0].regular.as_mut().unwrap();
        values.portfolio = current.portfolio.clone();
        values.capital_carry = if scorp { scorp_carry() } else { amounts(0, 0).capital_carry };
        source.components[0].alternative = K1StatementAmt::SameAsRegular;
        source.risk_groups[0].investment_parts.as_mut().unwrap().contributions = 0;
        match &mut source.risk_groups[0].source {
            K1RiskSource::Partnership { regular, alternative } => {
                regular.excluded_contributions = 0;
                regular.prior_loss = 0;
                *alternative = Some(regular.clone());
            }
            K1RiskSource::Scorp { regular, alternative } => {
                regular.excluded_funding = 0;
                regular.prior_loss = 0;
                *alternative = Some(regular.clone());
            }
            _ => unreachable!(),
        }
        assert_eq!(entity_exempt_other_rule(&income), None);
        let output = entity_exempt_other_output(&income, false)
            .unwrap_or_else(|error| panic!("scorp={scorp}: {error:?}"));
        let component = &output.components[0];
        assert_eq!(
            (
                component.capital[0].income.get(),
                component.capital[1].basis_allowed.get(),
                component.capital[1].risk_allowed.get()
            ),
            (600, 600, 600)
        );
        assert_eq!(
            (component.capital[1].basis_suspended.get(), component.capital[1].risk_suspended.get()),
            (0, 0)
        );
        assert_eq!(
            (component.income.get(), component.risk_allowed.get(), output.groups[0].current),
            (0, 0, 0)
        );
    }
}

#[test]
fn component_capital_competes_with_ordinary_carries_without_reclassifying_qualified_history() {
    use crate::tests::support::builders::k1res::ordinary_multi_retained_qbi_parent;
    let mut value = ordinary_multi_retained_qbi_parent();
    value.current.portfolio.short = -400_000;
    value.alternative_current = Some(value.current.clone());
    let K1ComponentStatement::Entered { source } = &mut value.activities else { unreachable!() };
    let values = source.components[1].regular.as_mut().unwrap();
    values.portfolio.short = -400_000;
    source.components[1].alternative = K1StatementAmt::SameAsRegular;
    assert_eq!(value.rule(), None);
    let output = value.ordinary_source(false).unwrap();
    let component = &output.components[1];
    // Basis opening1000 + A income4000 permits B ordinary basis carry1000
    // and capital loss4000. B's actual risk opening3000 shares6000 demand:
    // ordinary1000 basis +1000 retained risk; capital4000. No capacity reused.
    assert_eq!(
        (component.basis_allowed.get(), component.capital[0].basis_allowed.get()),
        (100_000, 400_000)
    );
    assert_eq!(
        (component.risk_allowed.get(), component.capital[0].risk_allowed.get()),
        (100_000, 200_000)
    );
    assert_eq!(
        (component.risk_suspended.get(), component.capital[0].risk_suspended.get()),
        (100_000, 200_000)
    );
    assert_eq!(output.groups[1].current, -100_000);
    // The actual original2025 risk vintage is80% qualified. Its1000 release
    // retains800 qualified; the new basis release's50% vintage stays at risk.
    assert_eq!(output.qualified[0].component, component.id);
    assert_eq!(output.qualified[0].released.get(), 80_000);
    let held = output.qualified[0].carries[1].remaining.iter().find(|v| v.year == 2026).unwrap();
    assert_eq!((held.remaining.get(), held.remaining_qbi.get()), (100_000, 50_000));
    let mut session = ordinary_multi_parent_session(&mut value);
    add(&mut session.draft, wage(0, Owner::Taxpayer, 6_000_000, 0));
    let id = session.draft.income[0].id;
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    documents(&mut request)[0].document =
        input(&FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(value) });
    accepted(&send_fixture(&mut session, request));
    let result = supported(&session.draft);
    // Passive A4000 and nonpassive B-1000 remain Schedule E3000;
    // the separately limited portfolio loss2000 leaves total income61000.
    assert_eq!(
        crate::tests::support::assert::value(
            &result,
            &alloc::format!("partnership_{id}_schedule_e")
        ),
        3_000
    );
    assert_eq!(crate::tests::support::assert::value(&result, "agi"), 61_000);
    assert_eq!(crate::tests::support::assert::value(&result, "qbi_loss_carryforward"), 800);
    assert_eq!(crate::tests::support::assert::value(&result, "qbi_deduction"), 0);
}

#[test]
fn component_capital_source_stages_preserve_subdollar_losses_before_line_projection() {
    use crate::tests::support::builders::k1res::{
        component_capital_source, entity_exempt_other_fields, entity_exempt_other_output,
        entity_exempt_other_rule, scorp_carry,
    };
    for scorp in [false, true] {
        for (loss, reported) in [(49, 0), (50, -1), (149, -1)] {
            let mut income = component_capital_source(scorp, 1);
            let (source, current, alternative) = entity_exempt_other_fields(&mut income);
            current.portfolio.short = -loss;
            current.portfolio.long = 0;
            *alternative = Some(current.clone());
            let values = source.components[0].regular.as_mut().unwrap();
            values.portfolio = current.portfolio.clone();
            values.capital_carry = if scorp { scorp_carry() } else { amounts(0, 0).capital_carry };
            source.components[0].alternative = K1StatementAmt::SameAsRegular;
            match &mut source.risk_groups[0].source {
                K1RiskSource::Partnership { regular, alternative } => {
                    regular.prior_loss = 0;
                    *alternative = Some(regular.clone());
                }
                K1RiskSource::Scorp { regular, alternative } => {
                    regular.prior_loss = 0;
                    *alternative = Some(regular.clone());
                }
                _ => unreachable!(),
            }
            assert_eq!(entity_exempt_other_rule(&income), None);
            let output = entity_exempt_other_output(&income, false)
                .unwrap_or_else(|error| panic!("scorp={scorp}, loss={loss}c: {error:?}"));
            let capital = &output.components[0].capital[0];
            assert_eq!((capital.basis_allowed.get(), capital.risk_allowed.get()), (loss, loss));
            assert_eq!((capital.basis_suspended.get(), capital.risk_suspended.get()), (0, 0));
            assert_eq!(crate::kernel::ratio::dollars(-capital.risk_allowed.get()), reported);
        }
    }
}

#[test]
fn component_capital_native_retained_sources_reach_annual_owners_and_correction() {
    use crate::tests::support::builders::k1res::entity_exempt_other_fields;
    use crate::tests::support::client::k1res::component_capital_session;
    for scorp in [false, true] {
        let (mut session, income) = component_capital_session(scorp, 100);
        add(&mut session.draft, wage(0, Owner::Taxpayer, 6_000_000, 0));
        session = restart(session.draft);
        let before = session.draft.clone();
        let dirty = session.dirty;
        let id = before.income[0].id;
        let ids = crate::k1_activity::source_ids(&income);
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
        assert_eq!(saved.next_id, before.next_id);
        assert_eq!(crate::k1_activity::source_ids(&saved.income[0].income), ids);
        let result = supported(&saved);
        assert_eq!(value(&result, "agi"), 59_300);
        assert_eq!(return_line(&result, "7a"), -70_000);
        let component = crate::k1_activity::source(&income).unwrap().components[0].id;
        for (prefix, allowed, basis_carry, risk_carry) in
            [("", 350, 400, 350), ("amt_", 700, 50, 350)]
        {
            for term in ["short", "long"] {
                for (stage, expected) in [
                    ("net", -allowed),
                    ("basis_carry", basis_carry),
                    ("risk_allowed", allowed),
                    ("risk_carry", risk_carry),
                ] {
                    assert_eq!(
                        value(
                            &result,
                            &alloc::format!("{prefix}k1_{id}_component_{component}_{term}_{stage}")
                        ),
                        expected
                    );
                }
            }
        }
        assert_eq!(value(&result, "capital_short_carryforward"), 0);
        assert_eq!(value(&result, "capital_long_carryforward"), 0);
        assert_eq!(value(&result, "amt_capital_adjustment"), -700);
        assert_eq!(value(&result, "alternative_minimum_taxable_income"), 59_300);
        crate::tests::support::client::k1res::assert_native_review(&mut session);
        let mut cold = restart(saved.clone());
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(
            documents(&mut reopened)[0].document,
            crate::tests::support::builders::k1res::expected_fixture_input(&saved, &income)
        );
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
        assert_eq!(supported(&cold.draft), result);
        let mut corrected = income.clone();
        let (source, current, _) = entity_exempt_other_fields(&mut corrected);
        current.portfolio.short = -500_000;
        source.components[0].regular.as_mut().unwrap().portfolio.short = -500_000;
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        documents(&mut request)[0].document = input(&corrected);
        accepted(&send_fixture(&mut cold, request));
        assert_eq!(cold.draft.next_id, saved.next_id);
        assert_eq!(crate::k1_activity::source_ids(&cold.draft.income[0].income), ids);
        let correction = supported(&cold.draft);
        // REG basis allocates 1200 over losses 5400 and1000:1012.50/187.50.
        // Risk prior100/class makes1112.50/287.50;700 capacity gives556.25/143.75.
        assert_eq!(value(&correction, "agi"), 59_300);
        assert_eq!(
            value(&correction, &alloc::format!("amt_k1_{id}_component_{component}_short_net")),
            -700
        );
        assert_eq!(supported(&restart(cold.draft.clone()).draft), correction);
    }
}

#[test]
fn component_capital_current_netting_caps_and_category_cents_use_existing_owners() {
    use crate::tests::support::builders::k1res::configure_component_capital_current;
    use crate::tests::support::client::k1res::entity_exempt_other_session;
    for scorp in [false, true] {
        let (mut session, mut income) = entity_exempt_other_session(scorp);
        add(&mut session.draft, wage(0, Owner::Taxpayer, 6_000_000, 0));
        for (status, sources, capital, short, long) in [
            (FederalStatus::Single, [[-500_000, 0], [0, -200_000]], -3_000, 2_000, 2_000),
            (FederalStatus::Separate, [[-500_000, 0], [0, -200_000]], -1_500, 3_500, 2_000),
            (FederalStatus::Single, [[100_000, 0], [0, -800_000]], -3_000, 0, 4_000),
        ] {
            session.draft.setup = complete(status).setup;
            if status == FederalStatus::Separate {
                answer_separate_pages(&mut session.draft);
            }
            configure_component_capital_current(&mut income, sources, [[0; 2]; 2]);
            session.draft.income[0].income = income.clone();
            let result = supported(&session.draft);
            assert_eq!(return_line(&result, "7a"), capital * 100);
            assert_eq!(value(&result, "capital_short_carryforward"), short);
            assert_eq!(value(&result, "capital_long_carryforward"), long);
            assert_eq!(value(&result, "amt_capital_short_carryforward"), 0);
            assert_eq!(value(&result, "amt_capital_long_carryforward"), 0);
        }
        session.draft.setup = complete(FederalStatus::Single).setup;
        for (cents, reported) in [(49, -1), (50, -1), (149, -3)] {
            configure_component_capital_current(
                &mut income,
                [[-cents, 0], [-cents, 0]],
                [[0, cents], [0, cents]],
            );
            session.draft.income[0].income = income.clone();
            let result = supported(&session.draft);
            assert_eq!(return_line(&result, "7a"), reported * 100);
            assert_eq!(value(&result, "agi"), 60_000 + reported);
            assert_eq!(result.capital_footing, Some([0, 0]));
        }
        for (cents, reported) in [(49, 1), (50, 2), (149, 4)] {
            configure_component_capital_current(
                &mut income,
                [[0, cents], [0, cents]],
                [[0, cents], [0, cents]],
            );
            let mut draft = session.draft.clone();
            draft.income[0].income = income.clone();
            add(
                &mut draft,
                crate::tests::support::builders::income(FederalIncome::Dividend {
                    data: dividend_source(native_dividend(0, 0, cents)),
                }),
            );
            let result = supported(&draft);
            assert_eq!(return_line(&result, "7a"), reported * 100);
        }
        configure_component_capital_current(
            &mut income,
            [[30_024, 0], [30_025, 0]],
            [[0, 30_025], [0, 30_025]],
        );
        session.draft.income[0].income = income;
        let result = supported(&session.draft);
        assert_eq!(return_line(&result, "7a"), 60_000);
        assert_eq!(value(&result, "agi"), 60_600);
        investment_borrowing(&mut session.draft, 1_000, 0, 0);
        let result = supported(&session.draft);
        assert_eq!(value(&result, "investment_form_line4f"), 600);
        assert_eq!(value(&result, "amt_investment_form_line4f"), 0);
        assert_eq!(value(&result, "amt_capital_adjustment"), 1);
        assert_eq!(value(&result, "alternative_minimum_taxable_income"), 60_601);
        session.draft.income.truncate(1);
        configure_component_capital_current(
            &mut session.draft.income[0].income,
            [[-500_000, 0], [0, -200_000]],
            [[-100_000, 0], [0, -100_000]],
        );
        let result = supported(&session.draft);
        assert_eq!(value(&result, "nol_carryforward"), 0);
        assert_eq!(value(&result, "amt_nol_carryforward"), 0);
    }
}

#[test]
fn component_capital_joint_sources_keep_niit_and_injured_spouse_ownership() {
    use crate::tests::support::assert::{assert_injured_ordinary_unchanged, niit_optional_column};
    use crate::tests::support::builders::k1res::configure_component_capital_current;
    use crate::tests::support::client::k1res::entity_exempt_other_session;
    for scorp in [false, true] {
        for owner in [Owner::Taxpayer, Owner::Spouse] {
            let (mut session, mut income) = entity_exempt_other_session(scorp);
            let joint = complete(FederalStatus::Joint);
            session.draft.setup = joint.setup;
            session.draft.people = joint.people;
            session.draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
            session.draft.injured_spouse = Some(allocation());
            session.draft.income[0].owner = crate::DocumentOwner::from(owner);
            add(&mut session.draft, wage(0, Owner::Taxpayer, 30_000_000, 0));
            configure_component_capital_current(
                &mut income,
                [[100_000, 0], [0, -30_000]],
                [[40_000, 0], [0, -10_000]],
            );
            crate::tests::support::client::admit_native_fixture_facts(&mut session);
            let mut request =
                crate::tests::support::client::income_pages::open_document_center(&mut session);
            documents(&mut request)[0].document = input(&income);
            accepted(&send_fixture(&mut session, request));
            for injured in [Owner::Taxpayer, Owner::Spouse] {
                session.draft.injured_spouse.as_mut().unwrap().injured = injured;
                let result = supported(&session.draft);
                assert_eq!(value(&result, "agi"), 300_700);
                assert_eq!(value(&result, "form_8960_net_investment_income"), 700);
                assert_eq!(value(&result, "net_investment_income_tax"), 27);
                assert_eq!(value(&result, "self_employment_tax"), 0);
                assert_eq!(value(&result, "qbi_deduction"), 0);
                assert_eq!(value(&result, "amt_capital_adjustment"), -400);
                assert_eq!(value(&result, "alternative_minimum_taxable_income"), 300_300);
                assert_eq!(
                    niit_optional_column(&result, "13b", "b"),
                    Some(if owner == injured { 700 } else { 0 })
                );
                let medicare = if injured == Owner::Taxpayer { 450 } else { 0 };
                assert_eq!(
                    niit_optional_column(&result, "18", "b"),
                    Some(medicare + if owner == injured { 27 } else { 0 })
                );
                assert_injured_ordinary_unchanged(&session.draft, &result, supported);
                crate::tests::support::client::k1res::assert_native_review(&mut session);
            }
        }
    }
}

#[test]
fn component_capital_missing_histories_save_unknown_but_invalid_sources_are_atomic() {
    use crate::tests::support::assert::k1res::assert_atomic_rejection;
    use crate::tests::support::builders::k1res::{
        entity_exempt_other_fields, entity_exempt_other_output,
    };
    use crate::tests::support::client::k1res::component_capital_session;
    for scorp in [false, true] {
        let (mut session, income) = component_capital_session(scorp, 100);
        let mut invalid = income.clone();
        entity_exempt_other_fields(&mut invalid).1.portfolio.short += 1;
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&invalid);
        assert_atomic_rejection(&mut session, request, "K1_ACTIVITY_CURRENT_SOURCE_RECONCILE");
        let expected_amt: Vec<_> =
            entity_exempt_other_output(&income, true).unwrap().capital_sources().collect();
        let mut unknown = income;
        let (source, _, _) = entity_exempt_other_fields(&mut unknown);
        match &mut source.components[0].regular.as_mut().unwrap().capital_carry {
            K1StatementCapitalCarry::Partnership { amounts } => amounts.short.basis = None,
            K1StatementCapitalCarry::Scorp { amounts } => amounts.basis_short = None,
            _ => unreachable!(),
        }
        assert!(entity_exempt_other_output(&unknown, false).is_err());
        assert_eq!(
            entity_exempt_other_output(&unknown, true)
                .unwrap()
                .capital_sources()
                .collect::<Vec<_>>(),
            expected_amt
        );
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&unknown);
        accepted(&send_fixture(&mut session, request));
        let saved = session.draft.clone();
        let result = estimate(&saved);
        assert!(result.tax.is_none());
        assert_eq!(
            result
                .issues
                .iter()
                .filter(|issue| issue.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
                .count(),
            1
        );
        assert!(result.lines.is_empty());
        assert!(result.return_lines.is_empty());
        let mut cold = restart(saved.clone());
        let mut reopened =
            crate::tests::support::client::income_pages::open_document_center(&mut cold);
        assert_eq!(
            documents(&mut reopened)[0].document,
            crate::tests::support::builders::k1res::expected_fixture_input(&saved, &unknown)
        );
        accepted(&send_fixture(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
    }
}

#[test]
fn component_capital_same_term_prior_release_preserves_regular_and_amt_foreign_gross() {
    use crate::tests::support::builders::k1res::{
        configure_component_capital_positive_release, entity_exempt_other_output,
    };
    use crate::tests::support::client::k1res::component_capital_session;
    for scorp in [false, true] {
        let (mut session, mut source) = component_capital_session(scorp, 100);
        configure_component_capital_positive_release(&mut source);
        let regular = entity_exempt_other_output(&source, false).unwrap();
        let amt = entity_exempt_other_output(&source, true).unwrap();
        // Current contribution 1200 funds actual basis carries 400 REG/200 AMT.
        // Neither carry passed basis before 2026; prior risk loss/deduction is 0.
        // Current ST gain 1000 remains gross income before either released loss.
        for (output, loss, net) in [(&regular, 40_000, 60_000), (&amt, 20_000, 80_000)] {
            let capital = &output.components[0].capital[0];
            assert_eq!(capital.income.get(), 100_000);
            assert_eq!(capital.basis_allowed.get(), loss);
            assert_eq!(capital.risk_allowed.get(), loss);
            assert_eq!(capital.basis_suspended.get(), 0);
            assert_eq!(capital.risk_suspended.get(), 0);
            assert_eq!(output.capital_sources().nth(1), Some([net, 0]));
        }
        let foreign = foreign_high_tax_return(1_000);
        session.draft.credits = foreign.credits;
        for entry in foreign.income {
            add(&mut session.draft, entry);
        }
        crate::tests::support::client::admit_native_fixture_facts(&mut session);
        let mut request =
            crate::tests::support::client::income_pages::open_document_center(&mut session);
        documents(&mut request)[0].document = input(&source);
        accepted(&send_fixture(&mut session, request));
        let result = supported(&session.draft);
        assert_eq!(return_line(&result, "7a"), 60_000);
        assert_eq!(value(&result, "amt_capital_adjustment"), 200);
        assert_eq!(result.capital_footing, Some([100_000, 100_000]));
        assert_eq!(value(&result, "agi"), 200_600);
        assert_eq!(value(&result, "alternative_minimum_taxable_income"), 200_800);
        for amt in [false, true] {
            assert_eq!(form_1116_any(&result, amt, "_gross_income_all_sources"), 201_000);
        }
    }
}
