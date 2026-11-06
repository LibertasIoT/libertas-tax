use crate::k1_activity::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::builders::k1res::expected_input;
use crate::tests::support::builders::k1res::{documents, input};
use crate::tests::support::client::k1res::scorp_ordinary_multi_session;
use crate::tests::support::client::k1res::send_fixture;
use crate::tests::*;

#[test]
fn scorp_ordinary_multi_stock_debt_risk_and_native_return_lifecycle() {
    let (mut session, value) = scorp_ordinary_multi_session();
    let regular = value.ordinary_source(false).unwrap();
    let alternative = value.ordinary_source(true).unwrap();
    // Whole stock basis $1,000 + $4,000 positive income, then $1,000
    // debt basis, permits $6,000 of B's $7,000 current/prior loss.
    assert_eq!(
        (
            crate::kernel::ratio::dollars(regular.basis.ending.get()),
            crate::kernel::ratio::dollars(
                regular.basis.scorp.as_ref().unwrap().loans[0].basis.get()
            )
        ),
        (0, 0)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(regular.components[1].basis_allowed.get()),
            crate::kernel::ratio::dollars(regular.components[1].basis_suspended.get())
        ),
        (6_000, 1_000)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(regular.components[1].risk_allowed.get()),
            crate::kernel::ratio::dollars(regular.components[1].risk_suspended.get())
        ),
        (2_500, 4_500)
    );
    // AMT owns $2,000 stock + $4,000 income + $500 debt basis and an
    // independently documented $4,500 investment for B.
    assert_eq!(
        (
            crate::kernel::ratio::dollars(alternative.components[1].basis_allowed.get()),
            crate::kernel::ratio::dollars(alternative.components[1].basis_suspended.get())
        ),
        (6_500, 500)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(alternative.components[1].risk_allowed.get()),
            crate::kernel::ratio::dollars(alternative.components[1].risk_suspended.get())
        ),
        (4_500, 3_000)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(regular.groups[0].passive),
            crate::kernel::ratio::dollars(regular.groups[1].nonpassive),
            crate::kernel::ratio::dollars(regular.groups[1].passive)
        ),
        (4_000, -2_500, -2_000)
    );

    // A distinct S-corp basis case: positive current income restores the
    // existing reduced debt before remaining stock basis is reported.
    let mut profit = value.clone();
    profit.current.ordinary = 500_000;
    let K1ComponentStatement::Entered { source } = &mut profit.activities else { unreachable!() };
    source.components[1].regular.as_mut().unwrap().operating = 100_000;
    let restored = profit.ordinary_source(false).unwrap();
    let debt = &restored.basis.scorp.as_ref().unwrap().loans[0];
    assert_eq!(
        (
            crate::kernel::ratio::dollars(restored.basis.ending.get()),
            crate::kernel::ratio::dollars(debt.restoration.get()),
            crate::kernel::ratio::dollars(debt.basis.get())
        ),
        (1_000, 4_000, 5_000)
    );

    // Current and retained source cents remain exact inside the basis stage.
    let mut cents = value.clone();
    cents.current.ordinary -= 2;
    let K1ComponentStatement::Entered { source } = &mut cents.activities else { unreachable!() };
    source.components[1].regular.as_mut().unwrap().operating -= 2;
    let loss = source.components[1].regular.as_mut().unwrap();
    loss.operating -= 49;
    *loss.operating_carry.basis.as_mut().unwrap() += 49;
    cents.current.ordinary -= 49;
    let K1ComponentQualified::Scorp { ordinary, .. } =
        &mut source.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified
    else {
        unreachable!()
    };
    ordinary.history[0].original += 49;
    ordinary.history[0].remaining += 49;
    let rounded = cents.ordinary_source(false).unwrap();
    assert_eq!(rounded.components[1].basis_suspended.get(), 100_100);
    assert_eq!(crate::kernel::ratio::dollars(rounded.components[1].risk_allowed.get()), 2_500);

    add(&mut session.draft, wage(0, Owner::Taxpayer, 25_000_000, 0));
    session = restart(session.draft);
    let before = session.draft.clone();
    let id = before.income[0].id;
    let dirty = session.dirty;
    let mut request =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    documents(&mut request)[0].document =
        input(&FederalIncome::ScorpActivities { data: value.clone() });
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
    let result = estimate(&saved);
    assert!(result.tax.is_some(), "{:?}", result.issues);
    // $4,000 passive A releases $2,000 former passive B. B's $2,500
    // nonpassive loss yields ScheduleE -$500. NIIT retains $2,000.
    assert_eq!(line_amount(&result, &alloc::format!("scorp_{id}_schedule_e")), -50_000);
    assert_eq!(line_amount(&result, &alloc::format!("amt_scorp_{id}_schedule_e")), -150_000);
    assert_eq!(line_amount(&result, "agi"), 24_950_000);
    assert_eq!(line_amount(&result, "form_8960_net_investment_income"), 200_000);
    assert_eq!(line_amount(&result, "net_investment_income_tax"), 7_600);
    assert!(noted(&result, "WARN_K1_QBI_LOSS_NO_STATEMENT"));
    assert!(result.issues.is_empty());
    let mut reopened =
        crate::tests::support::client::income_pages::open_document_center(&mut session);
    assert_eq!(documents(&mut reopened)[0].document, expected_input(&saved, &saved.income[0]));

    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let mut reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
    documents(&mut reopened)[0].document = input(&FederalIncome::ScorpActivities { data: profit });
    accepted(&send_fixture(&mut cold, reopened));
    assert_eq!(cold.draft.revision, saved.revision + 2);
    assert_eq!(cold.draft.next_id, saved.next_id);
    let corrected = estimate(&cold.draft);
    assert!(corrected.tax.is_some(), "{:?}", corrected.issues);
    // 2026-10-17 correction: settle_at_risk retains the separate $1,000
    // prior risk carry as well as the $1,000 prior basis carry. B's $1,000
    // current income less both allowed carries is -$1,000; A's $4,000
    // passive income also releases B's $2,000 prior passive loss.
    assert_eq!(line_amount(&corrected, &alloc::format!("scorp_{id}_schedule_e")), 100_000);
    assert_eq!(line_amount(&corrected, &alloc::format!("amt_scorp_{id}_schedule_e")), -150_000);

    // B9-1-multi acceptance, 2026-10-18: native S-corporation groups
    // must never also appear as direct Part I rentals, in either tax system.
    for (draft, report, regular_b, ordinary) in
        [(&saved, &result, -450_000, -500), (&cold.draft, &corrected, -300_000, 1_000)]
    {
        for (prefix, loss, expected) in [("", regular_b, ordinary), ("amt_", -550_000, -1_500)] {
            for (group, amount) in [(regular.groups[0].id, 400_000), (regular.groups[1].id, loss)] {
                let row = alloc::format!("{prefix}scorp_{id}_group_{group}_activity_net");
                assert_eq!(line_amount(report, &row), amount);
                let row = report.lines.iter().find(|line| line.id == row).unwrap();
                assert_eq!(row.explanation.key, "EXPLAIN_SCORP_ACTIVITY_NET");
                assert_eq!(
                    line_amount(
                        report,
                        &alloc::format!("{prefix}scorp_{id}_group_{group}_passive_suspended"),
                    ),
                    0,
                );
            }
            let mut detail = FederalResult::empty(draft.revision);
            let pool = crate::extended_rules::rentals_for(draft, 0, &mut detail, prefix == "amt_");
            assert_eq!(pool.part_one, 0);
            assert_eq!(pool.scorp_business, expected);
        }
        assert!(!report.lines.iter().any(|line| {
            line.id.starts_with(&alloc::format!("rental_{id}_"))
                || line.id.starts_with(&alloc::format!("amt_rental_{id}_"))
        }));
    }

    for boundary in 0..10 {
        let mut unavailable = value.clone();
        let K1ComponentStatement::Entered { source } = &mut unavailable.activities else {
            unreachable!()
        };
        match boundary {
            0 => unavailable.entity.loans[0].alternative_basis = None,
            1 => {
                unavailable.entity.loans[0].regular.repayments = 100;
                unavailable.entity.loans[0].regular.repayment_term =
                    ScorpNoteHolding::Single { term: GainTerm::Long };
            }
            2 => unavailable.health_insurance = None,
            3 => source.risk_groups[1].investment_parts = None,
            4 => source.passive_groups[1].participation = K1ActivityParticipation::Unavailable,
            5 => source.qbi_blocks[1].allocations = Some(vec![]),
            6 => {
                let K1ComponentQualified::Scorp { ordinary, .. } =
                    &mut source.qbi_blocks[1].allocations.as_mut().unwrap()[0].qualified
                else {
                    unreachable!()
                };
                ordinary.history[0].qualified = 1;
                ordinary.history[0].remaining_qualified = 1;
            }
            7 => unavailable.entity.loans[0].regular.advances = 100,
            8 => unavailable.person.as_mut().unwrap().withholding = None,
            _ => {}
        }
        let mut draft = saved.clone();
        draft.income[0].income = FederalIncome::ScorpActivities { data: unavailable };
        if boundary == 9 {
            draft.setup = complete(FederalStatus::Separate).setup;
            draft.setup.as_mut().unwrap().community_property = community_arizona();
            for entry in &mut draft.income {
                entry.community = CommunityShare::Separate;
            }
        }
        let result = estimate(&draft);
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
        // 2026-10-18: QBI reads admitted regular-tax output. Missing only
        // the independent AMT loan basis does not erase the established
        // regular no-statement fact (app-contract: source availability).
        assert_eq!(
            noted(&result, "WARN_K1_QBI_LOSS_NO_STATEMENT"),
            boundary == 0,
            "boundary {boundary}"
        );
        assert!(!noted(&result, "WARN_K1_QBI_NO_STATEMENT"));
    }
}
