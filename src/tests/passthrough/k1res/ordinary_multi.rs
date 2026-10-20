use crate::k1_activity::*;
use crate::tests::cross::interview::{accepted, open_topic};
use crate::tests::support::builders::k1res::{amounts, documents, input, ordinary_multi_parent};
use crate::tests::support::client::income_pages::catalog_preview_document;
use crate::tests::support::client::k1res::ordinary_multi_parent_session;
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
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    documents(&mut request)[0].document = input(&income);
    let running =
        send(&mut session, crate::tests::support::client::estimate_documents_from_save(&request));
    assert!(matches!(running, Reply::Response(P::IncomeRunningEstimate { .. })), "{running:?}");
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    accepted(&send(&mut session, request));
    assert_eq!(session.draft.revision, before.revision + 1);
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
    let mut request = open_topic(&mut session, TaxSection::PassThrough);
    assert_eq!(documents(&mut request)[0].document, input(&saved.income[0].income));
    let preview_dirty = session.dirty;
    let mut preview = catalog_preview_document(documents(&mut request)[0].document.clone());
    let P::PreviewDocument { id: preview_id, .. } = &mut preview else { unreachable!() };
    *preview_id = id;
    assert!(matches!(
        send(&mut session, preview),
        Reply::Response(P::IncomePreview { preview: Some(-100_000), .. })
    ));
    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, preview_dirty);
    let mut cold = restart(saved.clone());
    let mut reopened = open_topic(&mut cold, TaxSection::PassThrough);
    assert_eq!(documents(&mut reopened)[0].document, input(&saved.income[0].income));
    assert_eq!(estimate(&cold.draft), result);

    // Correct only regular B; the separate AMT source remains unchanged.
    let IncomeDocument::K1PartnershipActivities(page) = &mut documents(&mut reopened)[0].document
    else {
        unreachable!()
    };
    page.current.ordinary = 500_000;
    let K1ComponentStatementPage::Entered { source } = &mut page.activities.statement else {
        unreachable!()
    };
    source.components[1].regular.as_mut().unwrap().operating = 100_000;
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft.revision, saved.revision + 1);
    assert_eq!(cold.draft.next_id, saved.next_id);
    let corrected = estimate(&cold.draft);
    assert!(corrected.tax.is_some(), "{:?}", corrected.issues);
    assert_eq!(line_amount(&corrected, &alloc::format!("partnership_{id}_schedule_e")), 100_000);
    assert_eq!(
        line_amount(&corrected, &alloc::format!("amt_partnership_{id}_schedule_e")),
        -200_000
    );

    // Dispatcher decision 2026-10-19: exact cents and checked derived sums
    // must fit i64. Overflow is a located input-range refusal, not early rounding.
    for held_columns in [false, true] {
        let mut boundary = value.clone();
        boundary.entity.regular.opening = Some(i64::MAX);
        let K1ComponentStatement::Entered { source } = &mut boundary.activities else {
            unreachable!()
        };
        let loss = source.components[1].regular.as_mut().unwrap();
        if held_columns {
            loss.operating = -(i64::MAX - 50);
            loss.operating_carry.basis = Some(50);
            let K1ComponentQualified::Partnership { records, .. } =
                &mut source.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified
            else {
                unreachable!()
            };
            let row = &mut records.history.as_mut().unwrap().basis[0];
            row.original = 50;
            row.remaining = 50;
        } else {
            loss.operating = 100_000;
        }
        boundary.current.ordinary = 400_000 + loss.operating;
        assert_eq!(boundary.rule(), None);
        let (issue, _) = boundary.ordinary_source(false).unwrap_err();
        assert_eq!(issue.key, crate::k1_activity::ordinary::RANGE_KEY);
        assert_eq!(issue.field.unwrap().square(), "[entity/regular/opening]");
        let income =
            FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(boundary) };
        let issue = crate::k1_activity::income_issue(&income).unwrap();
        assert_eq!(issue.key, crate::k1_activity::ordinary::RANGE_KEY);
        assert_eq!(issue.field.unwrap().square(), "[entity/regular/opening]");
        let mut candidate = saved.clone();
        candidate.income[0].income = income;
        let stored = candidate.stored_issue().unwrap();
        assert_eq!(stored.key, crate::k1_activity::ordinary::RANGE_KEY);
        assert_eq!(stored.field.unwrap().square(), "[entity/regular/opening]");
        let result = estimate(&candidate);
        assert!(result.tax.is_none());
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.display.key == crate::k1_activity::ordinary::RANGE_KEY)
        );
    }
    // A realistic $10 billion opening basis remains inside the same cent owner.
    let mut large = value.clone();
    large.entity.regular.opening = Some(1_000_000_000_000);
    let K1ComponentStatement::Entered { source } = &mut large.activities else { unreachable!() };
    source.components[1].regular.as_mut().unwrap().operating = 100_000;
    large.current.ordinary = 500_000;
    assert_eq!(large.ordinary_source(false).unwrap().basis.ending.get(), 1_000_000_400_000);

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
    // Actual common-pool cents, rather than an arbitrary source ceiling,
    // decide arithmetic availability. One $5e18-cent source fits; two
    // individually supported sources overflow the i64 positive-income sum.
    let mut large = value.clone();
    large.current.ordinary = 5_000_000_000_000_000_000;
    large.alternative_current = Some(large.current.clone());
    large.entity.regular.opening = Some(0);
    large.entity.alternative.as_mut().unwrap().opening = Some(0);
    let K1ComponentStatement::Entered { source } = &mut large.activities else { unreachable!() };
    for (index, component) in source.components.iter_mut().enumerate() {
        component.regular =
            Some(amounts(if index == 0 { 5_000_000_000_000_000_000 } else { 0 }, 0));
        component.alternative = K1StatementAmt::SameAsRegular;
    }
    for block in &mut source.qbi_blocks {
        block.allocations = Some(vec![]);
    }
    for group in &mut source.risk_groups {
        group.source = K1RiskSource::Partnership {
            regular: partner_risk_record(),
            alternative: Some(partner_risk_record()),
        };
    }
    let mut one = saved.clone();
    one.income[0].income =
        FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(large) };
    let one_result = estimate(&one);
    assert!(one_result.tax.is_some(), "{:?}", one_result.issues);
    let mut second = one.income[0].clone();
    second.id = one.next_id;
    one.next_id += 1;
    let source = crate::k1_activity::source_ids_mut(&mut second.income);
    for id in source {
        *id = one.next_id;
        one.next_id += 1;
    }
    let FederalIncome::PartnershipActivities { data } = &mut second.income else { unreachable!() };
    let K1ComponentStatement::Entered { source } = &mut data.activities else { unreachable!() };
    for index in 0..2 {
        source.components[index].risk_group = Some(source.risk_groups[index].id);
        source.components[index].passive_group = Some(source.passive_groups[index].id);
    }
    one.income.push(second);
    let overflow = estimate(&one);
    assert!(overflow.tax.is_none());
    assert!(overflow.lines.is_empty());
    assert!(overflow.return_lines.is_empty());
    assert!(overflow.payments.is_none());
    assert!(overflow.refund.is_none());
    assert!(
        overflow
            .issues
            .iter()
            .any(|v| v.display.key == "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE")
    );

    let mut refused_session = restart(saved.clone());
    let mut unavailable = value;
    unavailable.entity.publicly_traded = true;
    let refused_income =
        FederalIncome::PartnershipActivities { data: alloc::boxed::Box::new(unavailable) };
    let mut request = open_topic(&mut refused_session, TaxSection::PassThrough);
    documents(&mut request)[0].document = input(&refused_income);
    assert!(matches!(
        send(
            &mut refused_session,
            crate::tests::support::client::estimate_documents_from_save(&request)
        ),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(refused_session.draft, saved);
    accepted(&send(&mut refused_session, request));
    assert!(estimate(&refused_session.draft).tax.is_none());
    let refused_saved = refused_session.draft.clone();
    let mut refused_cold = restart(refused_saved.clone());
    let mut reopened = open_topic(&mut refused_cold, TaxSection::PassThrough);
    assert_eq!(documents(&mut reopened)[0].document, input(&refused_income));
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
        let mut request = open_topic(&mut session, TaxSection::PassThrough);
        documents(&mut request)[0].document = input(&income);
        accepted(&send(&mut session, request));
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
        let mut reopened = open_topic(&mut cold, TaxSection::PassThrough);
        assert_eq!(documents(&mut reopened)[0].document, input(&saved.income[0].income));
        assert_eq!(estimate(&cold.draft), result);
        match &mut documents(&mut reopened)[0].document {
            IncomeDocument::K1PartnershipActivities(page) => page.entity.regular.opening = Some(0),
            IncomeDocument::K1ScorpActivities(page) => page.entity.regular.opening = Some(0),
            _ => unreachable!(),
        }
        accepted(&send(&mut cold, reopened));
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
