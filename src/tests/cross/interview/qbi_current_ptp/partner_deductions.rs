use super::*;

#[test]
fn ptp_partner_deductions_keep_original_basis_risk_and_passive_qualification() {
    use crate::tests::support::builders::current_ptp_ordinary_histories;

    for partial in [false, true] {
        let mut owner = current_ptp_ordinary_histories(true).record();
        if partial {
            owner.regular.ordinary = 30_000;
            owner.qbi.as_mut().unwrap().income = 30_000;
            owner.regular.basis.opening = Some(170_000);
            owner.regular.basis.prior_loss = 400_000;
            owner.regular.risk.start =
                ScorpRiskStart::Detailed { capacity: 20_000, profit: 0, loss: 0 };
            owner.regular.risk.prior_loss = 0;
            owner.regular.passive_loss = 0;
            owner.qbi_records.history = Some(PartnerQbiHistory {
                basis: vec![loss_year(2023, 4_000, 3_000, 4_000, 3_000)],
                risk: vec![],
                passive: vec![],
            });
        }
        let original = owner.clone();
        let pool = owner.pool(&owner.regular).unwrap();
        let input = owner.ptp_passive_activity(2, &owner.regular, &pool).unwrap();
        let passive = crate::ptp_passive::without_section179(2, false, &[input]).unwrap();
        let suspended = crate::kernel::carry::Amount::new(passive.suspended_cents).unwrap();
        assert_eq!(passive.suspended_cents, if partial { 20_000 } else { 0 });
        assert_eq!(pool.current_ordinary_with_items, if partial { -20_000 } else { 500_000 });
        let points = if partial {
            [(201_749, -2_500), (239_250, -12_500), (276_751, -22_500)]
        } else {
            [(201_749, 600_000), (239_250, 150_000), (276_751, -300_000)]
        };
        for (taxable, expected) in points {
            let prepared = owner.qbi.as_ref().unwrap().calculate_combined_with_pool(
                &owner, suspended, if partial { 100 } else { 1_000 },
                (taxable, FederalStatus::Single), &Default::default(),
                &Default::default(), Some(&pool),
            ).unwrap();
            // Current 10,000-1,000 or 300-100 receives this year's factor.
            // Original qualified losses (3,000 or225) never receive it again.
            assert_eq!(prepared.ptp, crate::kernel::ratio::exact::whole(expected));
            assert_eq!(prepared.historical_cents.get(), if partial { 22_500 } else { 300_000 });
            assert_eq!(prepared.current_loss_cents.get(), 0);
            let remaining: Vec<_> = prepared.carries.iter().map(|stage| {
                stage.remaining.iter().fold((0, 0), |(loss, qbi), row| {
                    (loss + row.remaining.get(), qbi + row.remaining_qbi.get())
                })
            }).collect();
            assert_eq!(remaining, if partial {
                vec![(200_000, 150_000), (150_000, 112_500), (20_000, 15_000)]
            } else { vec![(0, 0); 3] });
        }
        assert_eq!(owner, original);
    }
}

#[test]
fn ptp_deductions_with_ordinary_histories_require_actual_records_and_current_income() {
    let owner = crate::tests::support::builders::current_ptp_ordinary_histories(false).record();
    for stage in 0..4 {
        let mut missing = owner.clone();
        if stage == 3 {
            missing.qbi_records.history = None;
        } else {
            let history = missing.qbi_records.history.as_mut().unwrap();
            match stage {
                0 => history.basis.clear(),
                1 => history.risk.clear(),
                _ => history.passive.clear(),
            }
        }
        assert_eq!(missing.qbi.as_ref().unwrap().calculate(
            &missing, k1_amount(0), 1_000, 100_000, FederalStatus::Single,
        ).err().unwrap().0, if stage == 3 {
            "ISSUE_PARTNERSHIP_QBI_COMPLETE_HISTORICAL_PARTNERSHIP_QBI_LOSS_RECORDS_4E8D198F"
        } else {
            "ISSUE_PARTNERSHIP_QBI_RECONCILE_HISTORICAL_PARTNERSHIP_QUALIFIED_LOSS_RECORDS_WITH_36AD0363"
        });
    }
    let exhausted = owner.qbi.as_ref().unwrap().calculate(
        &owner, k1_amount(0), 10_000, 100_000, FederalStatus::Single,
    ).unwrap();
    assert_eq!(exhausted.ptp, crate::kernel::ratio::exact::whole(-300_000));
    for deduction in [-1, 10_001] {
        assert_eq!(owner.qbi.as_ref().unwrap().calculate(
            &owner, k1_amount(0), deduction, 100_000, FederalStatus::Single,
        ).err().unwrap().0, PTP_HISTORY);
    }
}

#[test]
fn ptp_ordinary_histories_and_care_corrections_survive_native_save_reopen_restart() {
    let page = crate::tests::support::builders::current_ptp_ordinary_histories(false);
    let source = IncomeSource::K1 { value: K1Document::K1Partnership((page).into()) };
    let mut session = Interview::new(TEST_USER, current_ptp_care_return(60_000));
    let request = crate::tests::support::client::income_pages::save_documents(
        &mut session, vec![document(0, source.clone())],
    );
    accepted(&exchange(&mut session, request, "ptp-three-histories-documents", false));
    let id = session.draft.income.iter().find(|entry| {
        matches!(entry.income, FederalIncome::Partnership { .. })
    }).unwrap().id;
    for (amount, agi, rental, category, deduction) in [
        (100_000, 63_000, 3_000, 6_000, 1_200),
        (50_000, 63_500, 3_500, 6_500, 1_300),
    ] {
        let Reply::Edit(mut request @ P::SaveCare { .. }) =
            from_review(&mut session, TaxSection::Care)
        else { panic!("normal care editor") };
        let P::SaveCare { value: care, .. } = &mut request else { unreachable!() };
        care.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
        care.own_business = vec![CareOwnBusinessBenefitInput {
            owner: Owner::Taxpayer,
            source: CareBenefitSourceInput::Partnership { partnership: id },
            received: amount,
        }];
        let case = alloc::format!("ptp-three-histories-care-{amount}");
        accepted(&exchange(&mut session, request, &case, false));
        let result = supported(&session.draft);
        // Financial losses3,000+2,000+1,000 leave4,000, then care reduces
        // ScheduleE once. Their original qualified portions total3,000,
        // separately from current QBI10,000 less the selected care deduction.
        assert_eq!(value(&result, "agi"), agi);
        assert_eq!(value(&result, "rental_income"), rental);
        assert_eq!(value(&result, "qbi_reit_ptp"), category);
        assert_eq!(value(&result, "qbi_deduction"), deduction);
        assert_eq!(value(&result, "care_form2441_line24"), amount / 100);
        ptp_reopen_and_cold_resave(
            &mut session, core::slice::from_ref(&source), &result, &case, exchange,
        );
    }
}

#[test]
fn positive_ptp_deductions_keep_passive_fifo_separate_from_current_sstb() {
    for (income, suspended, historical, points) in [
        (10_000, 0, 100_000, [(201_749, 800_000), (239_250, 350_000), (276_751, -100_000)]),
        (1_500, 150_000, 25_000, [(201_749, 25_000), (239_250, 0), (276_751, -25_000)]),
    ] {
        let mut page = current_ptp_passive_history(income, true);
        page.participation = Some((LimitedParticipation::Investor { prior_years: 0 }).record());
        page.limited_for_se = Some(true);
        let owner = page.record();
        let pool = owner.pool(&owner.regular).unwrap();
        let input = owner.ptp_passive_activity(2, &owner.regular, &pool).unwrap();
        let passive = crate::ptp_passive::without_section179(2, false, &[input]).unwrap();
        assert_eq!(passive.suspended_cents, suspended);
        let actual_suspended = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
            passive.suspended_cents,
        ).unwrap();
        let before = alloc::format!("{pool:?}");
        for (taxable, expected) in points {
            let prepared = owner.qbi.as_ref().unwrap().calculate_combined_with_pool(
                &owner, actual_suspended, 1_000, (taxable, FederalStatus::Single),
                &Default::default(), &Default::default(), Some(&pool),
            ).unwrap();
            // Current QBI is 9,000 or 500 before its 100%/50%/0% factor.
            // FIFO releases the pre-2018 1,000 first, then 2,000 or 500
            // at its original 50% qualification, without another SSTB test.
            assert_eq!(prepared.ptp, crate::kernel::ratio::exact::whole(expected));
            assert_eq!(prepared.historical_cents.get(), historical);
            assert_eq!(prepared.current_loss_cents.get(), 0);
            assert_eq!(prepared.carries[2].remaining.iter().map(|v| v.remaining.get()).sum::<i64>(), suspended);
            assert_eq!(prepared.current.income, 0);
            assert_eq!(alloc::format!("{pool:?}"), before);
        }
        for (history, key) in [
            (None, "ISSUE_PARTNERSHIP_QBI_COMPLETE_HISTORICAL_PARTNERSHIP_QBI_LOSS_RECORDS_4E8D198F"),
            (Some(PartnerQbiHistory { basis: vec![], risk: vec![], passive: vec![] }), "ISSUE_PARTNERSHIP_QBI_RECONCILE_HISTORICAL_PARTNERSHIP_QUALIFIED_LOSS_RECORDS_WITH_36AD0363"),
        ] {
            let mut incomplete = owner.clone();
            incomplete.qbi_records.history = history;
            assert_eq!(incomplete.qbi.as_ref().unwrap().calculate(
                &incomplete, actual_suspended, 1_000, 100_000, FederalStatus::Single,
            ).err().unwrap().0, key);
        }
    }
    let mut current = ptp_page(10_000, false).record();
    current.qbi_records.history = Some(PartnerQbiHistory { basis: vec![], risk: vec![], passive: vec![] });
    assert_eq!(current.qbi.as_ref().unwrap().calculate(
        &current, k1_amount(0), 1_000, 100_000, FederalStatus::Single,
    ).unwrap().ptp, crate::kernel::ratio::exact::whole(900_000));
}

#[test]
fn current_ptp_qualified_deductions_precede_sstb_and_preserve_the_activity_pool() {
    // Rev. Proc. 2025-32 section 4.26: the same independently specified
    // 2026 single-filer thresholds as the current PTP controls.
    for (taxable, expected) in [(201_749, 7_000), (239_250, 3_500), (276_751, 0)] {
        let owner = ptp_page(10_000, true).record();
        let pool = owner.pool(&owner.regular).unwrap();
        let before = alloc::format!("{pool:?}");
        let result = owner
            .qbi
            .as_ref()
            .unwrap()
            .calculate_combined_with_pool(
                &owner,
                k1_amount(0),
                3_000,
                (taxable, FederalStatus::Single),
                &Default::default(),
                &Default::default(),
                Some(&pool),
            )
            .unwrap();
        // The qualified $3,000 is applied once, before the 100%, 50%, 0%
        // SSTB factor. It never becomes a basis, risk or passive deduction.
        assert_eq!(result.ptp, crate::kernel::ratio::exact::whole(expected * 100));
        assert_eq!(result.current.income, 0);
        assert_eq!(result.current.wages, 0);
        assert_eq!(result.current.property, 0);
        assert_eq!(alloc::format!("{pool:?}"), before);
        assert_eq!(pool.nonpassive + pool.passive, 1_000_000);
    }
    let mut page = ptp_page(10_000, false);
    // Independently reported REIT dividends are not an operand of the
    // ordinary-trade reduction. The caller joins them separately.
    let PartnerQbi::Statement { reit_dividends, .. } = &mut page.qbi else { unreachable!() };
    *reit_dividends = 200_000;
    let owner = page.record();
    for deduction in [9_999, 10_000, 10_001, -1] {
        let result = owner.qbi.as_ref().unwrap().calculate(
            &owner,
            k1_amount(0),
            deduction,
            239_250,
            FederalStatus::Single,
        );
        if (0..=10_000).contains(&deduction) {
            assert_eq!(
                result.unwrap().ptp,
                crate::kernel::ratio::exact::whole((10_000 - deduction) * 100)
            );
        } else {
            assert_eq!(result.err().unwrap().0, PTP_HISTORY);
        }
    }
    assert_eq!(owner.qbi.as_ref().unwrap().reit_dividends, 200_000);
    let mut penny = owner.clone();
    penny.qbi.as_mut().unwrap().income = 999_999;
    assert_eq!(
        penny
            .qbi
            .as_ref()
            .unwrap()
            .calculate(&penny, k1_amount(0), 10_000, 239_250, FederalStatus::Single,)
            .err()
            .unwrap()
            .0,
        PTP_HISTORY
    );
}

#[test]
fn current_ptp_uses_actual_qualified_health_and_half_se_deductions() {
    for partial in [false, true] {
        let mut source = partner_distribution_health_source();
        source.publicly_traded = true;
        if partial {
            source.qbi.as_mut().unwrap().income = 1_000_000;
            source.qbi_records.qualified_se_gross = 1_000_000;
        }
        let mut draft = complete(FederalStatus::Single);
        draft.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 0);
        add(&mut draft, income(FederalIncome::Partnership { data: source }));
        let original = draft.clone();
        let result = supported(&draft);
        // SE: 20,000 x .9235 x .153 = 2825.91 ->2826; half1413.
        // Health6000. Full QBI20,000-1413-6000=12,587.
        // Actual qualified gross10,000/20,000 gives half707 and health3000;
        // partial QBI10,000-707-3000=6293. No net-income ratio is inferred.
        assert_eq!(value(&result, "self_employment_tax"), 2_826);
        assert_eq!(value(&result, "agi"), 72_587);
        assert_eq!(value(&result, "qbi_reit_ptp"), if partial { 6_293 } else { 12_587 });
        assert_eq!(value(&result, "qbi_deduction"), if partial { 1_259 } else { 2_517 });
        assert_eq!(value(&result, "rental_income"), 20_000);
        assert_eq!(draft, original);
    }
}

#[test]
fn current_ptp_partner_deductions_keep_unresolved_source_boundaries() {
    for case in 0..8 {
        let mut owner = ptp_page(10_000, false).record();
        match case {
            0 => {
                // 2026-10-24: Reg. 1.199A-3(b)(1)(iv) permits the ordinary
                // passive FIFO release. An unassociated basis history must
                // still reconcile to its actual suspended balance.
                owner.qbi_records.history =
                    Some(PartnerQbiHistory { basis: vec![loss_year(2023, 1, 1, 1, 1)], risk: vec![], passive: vec![] })
            }
            1 => owner.regular.basis.prior_loss = 1,
            2 => owner.regular.carried.charity = 1,
            3 => owner.qbi.as_mut().unwrap().ptp_income = 1,
            4 => owner.section179 = 1,
            5 => {
                owner.regular.ordinary = -100_000;
                owner.qbi.as_mut().unwrap().income = -100_000;
                owner.regular.basis.opening = Some(0);
            }
            6 => {
                owner.regular.ordinary = -100_000;
                owner.qbi.as_mut().unwrap().income = -100_000;
                owner.regular.risk.start =
                    ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
            }
            7 => owner.disposition = Some(K1Disposition::Sold { proceeds: 0 }),
            _ => unreachable!(),
        }
        assert_eq!(
            owner
                .qbi
                .as_ref()
                .unwrap()
                .calculate(&owner, k1_amount(0), 1, 100_000, FederalStatus::Single,)
                .err()
                .unwrap()
                .0,
            // 2026-10-24: i8995 Tracking Losses admits ordinary basis/risk
            // histories; missing or mismatched records keep their own notices.
            match case {
                0 => "ISSUE_PARTNERSHIP_QBI_RECONCILE_HISTORICAL_PARTNERSHIP_QUALIFIED_LOSS_RECORDS_WITH_36AD0363",
                1 => "ISSUE_PARTNERSHIP_QBI_COMPLETE_HISTORICAL_PARTNERSHIP_QBI_LOSS_RECORDS_4E8D198F",
                _ => PTP_HISTORY,
            },
            "source case {case}"
        );
    }
    let mut owner = ptp_page(10_000, false).record();
    owner.qbi.as_mut().unwrap().income = 500_000;
    assert_eq!(
        owner.qbi.as_ref().unwrap().qualified_care_deduction(&owner, 100).err().unwrap().0,
        "ISSUE_PARTNERSHIP_QBI_PROVIDE_PARTNERSHIP_GROSS_INCOME_FOR_PARTNER_DEDUCTION_67A6F2C5"
    );
    assert_eq!(
        owner.qbi.as_ref().unwrap().qualified_deduction(&owner, 100).err().unwrap().0,
        "ISSUE_PARTNERSHIP_QBI_PROVIDE_PARTNERSHIP_GROSS_INCOME_FOR_PARTNER_DEDUCTION_67A6F2C5"
    );
}

#[test]
fn current_ptp_care_native_save_correct_reopen_and_restart_debits_only_its_owner() {
    for retained in [false, true] {
    let mut session = Interview::new(TEST_USER, current_ptp_care_return(60_000));
    let pages = [if retained { current_ptp_passive_history(50_000, false) } else { ptp_page(50_000, false) }, ptp_page(10_000, false)];
    let mut documents = pages
        .iter()
        .cloned()
        .map(|page| document(0, IncomeSource::K1 { value: K1Document::K1Partnership((page).into()) }))
        .collect::<Vec<_>>();
    let reit = {
        let mut source = crate::DividendSource::printed(dividend_document(Div1099 {
         section199a: 200_000,

        ..native_div1099("Independent qualifying REIT".into(), 200_000, 0)
        }));
        source.reit_eligible = Some(true.into());
        dividend_document(source)
    };
    documents.push(document(
        0,
        IncomeSource::InterestDividend { value: InterestDividendDocument::Div1099(reit) },
    ));
    let request =
        crate::tests::support::client::income_pages::save_documents(&mut session, documents);
    accepted(&exchange(&mut session, request, &alloc::format!("ptp-care-{retained}-documents"), false));
    let source_ids: Vec<_> = session
        .draft
        .income
        .iter()
        .filter(|entry| matches!(entry.income, FederalIncome::Partnership { .. }))
        .map(|entry| entry.id)
        .collect();
    // Financial FIFO releases all $3,000. Only the post-2017 half-
    // qualified $2,000 vintage reduces QBI, by $1,000.
    let cases = if retained {
        [(300_000, 116_000, 54_000, 58_000, 11_600), (200_000, 117_000, 55_000, 59_000, 11_800)]
    } else {
        [(300_000, 119_000, 57_000, 59_000, 11_800), (200_000, 120_000, 58_000, 60_000, 12_000)]
    };
    for (amount, expected_agi, expected_rental, category, deduction) in cases {
        let Reply::Edit(mut request @ P::SaveCare { .. }) =
            from_review(&mut session, TaxSection::Care)
        else {
            panic!("normal care editor")
        };
        let P::SaveCare { value: page, .. } = &mut request else { unreachable!() };
        page.plan_maximum = CarePlanMaximum::Plans { maximum: 750_000 };
        page.own_business = vec![CareOwnBusinessBenefitInput {
            owner: Owner::Taxpayer,
            source: CareBenefitSourceInput::Partnership { partnership: source_ids[0] },
            received: amount,
        }];
        accepted(&exchange(
            &mut session,
            request,
            &alloc::format!("ptp-care-{retained}-save-{amount}"),
            false,
        ));
        let result = supported(&session.draft);
        // Only the selected K-1's Schedule E deduction changes: 50,000-3000
        // plus10,000, then 50,000-2000 plus10,000. The separate REIT2000
        // joins the category unchanged. No SE income for this limited partner;
        // QBI mirrors the selected owner's one current debit once.
        assert_eq!(value(&result, "agi"), expected_agi);
        assert_eq!(value(&result, "rental_income"), expected_rental);
        assert_eq!(value(&result, "qbi_reit_ptp"), category);
        assert_eq!(value(&result, "qbi_deduction"), deduction);
        assert_eq!(value(&result, "care_form2441_line24"), amount / 100);
        assert_eq!(
            value(&result, &alloc::format!("partnership_{}_care_deduction", source_ids[0])),
            amount / 100
        );
        assert!(
            !result
                .lines
                .iter()
                .any(|line| line.id
                    == alloc::format!("partnership_{}_care_deduction", source_ids[1]))
        );
        let saved = session.draft.clone();
        let Reply::Edit(reopened @ P::SaveCare { .. }) =
            from_review(&mut session, TaxSection::Care)
        else {
            panic!("reopened care editor")
        };
        accepted(&exchange(
            &mut session,
            reopened,
            &alloc::format!("ptp-care-{retained}-resave-{amount}"),
            false,
        ));
        assert_eq!(session.draft, saved);
        let mut cold = restart(saved.clone());
        let Reply::Edit(reopened @ P::SaveCare { .. }) = from_review(&mut cold, TaxSection::Care)
        else {
            panic!("cold care editor")
        };
        accepted(&exchange(&mut cold, reopened, &alloc::format!("ptp-care-{retained}-cold-{amount}"), false));
        assert_eq!(cold.draft, saved);
        assert_eq!(estimate(&cold.draft), result);
        let reopened = crate::tests::support::client::income_pages::open_document_center(&mut cold);
        accepted(&exchange(
            &mut cold,
            reopened,
            &alloc::format!("ptp-care-{retained}-source-resave-{amount}"),
            false,
        ));
        assert_eq!(cold.draft, saved);
    }
    }
}
