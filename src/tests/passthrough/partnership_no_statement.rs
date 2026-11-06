use super::*;

fn with_current_qualified_profit(source: PartnerK1) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let mut profit = ordinary_partner_source(10_000, true);
    profit.qbi = Some(partner_qbi_statement(10_000));
    add(&mut draft, income(FederalIncome::Partnership { data: profit }));
    let mut entry = income(FederalIncome::Partnership { data: source });
    entry.label = "Retained partnership losses".into();
    add(&mut draft, entry);
    draft
}

#[test]
fn partnership_no_statement_releases_qualified_history_from_each_actual_limit() {
    for stage in 0..3 {
        let mut source = ordinary_partner_source(if stage == 2 { 4_000 } else { 0 }, true);
        let mut history = PartnerQbiHistory { basis: vec![], risk: vec![], passive: vec![] };
        match stage {
            0 => {
                source.regular.basis.prior_loss = 400_000;
                history.basis = vec![loss_year(2024, 4_000, 2_000, 4_000, 2_000)];
            }
            1 => {
                source.regular.risk.prior_loss = 400_000;
                history.risk = vec![loss_year(2024, 4_000, 2_000, 4_000, 2_000)];
            }
            2 => {
                source.regular.passive_loss = 400_000;
                history.passive = vec![loss_year(2024, 4_000, 2_000, 4_000, 2_000)];
            }
            _ => unreachable!(),
        }
        source.alternative = Some(source.regular.clone());
        source.qbi_records.history = Some(history);
        // Retained allocation data is independent of an absent current statement.
        source.qbi_records.qualified_se_gross = 900_000;
        assert!(source.qbi.is_none() && source.se_gross.is_none());
        let draft = with_current_qualified_profit(source);
        let result = supported(&draft);
        assert_eq!(value(&result, "qbi_deduction"), 1_600);
        let release = result
            .review_notes
            .iter()
            .find(|note| {
                note.display.key == "NOTE_PARTNERSHIP_HISTORICAL_QBI_RELEASE"
                    && note.context.as_ref().map(crate::IssueContext::identity)
                        == Some(crate::federal_model::NoteIdentity::Income(draft.income[2].id))
            })
            .unwrap();
        assert_eq!(
            release.display.args,
            vec![
                MessageArg::Text("Retained partnership losses".into()),
                MessageArg::usd_cents(200_000),
            ]
        );
        assert_eq!(
            release.context.as_ref().unwrap().identity(),
            crate::federal_model::NoteIdentity::Income(draft.income[2].id)
        );
        assert!(!noted(&result, "WARN_K1_QBI_LOSS_NO_STATEMENT"));
        assert!(!noted(&result, "WARN_QBI_GROSS_ALLOCATION_LEFT_OUT"));
        assert_eq!(supported(&restart(draft.clone()).draft), result);
        let FederalIncome::Partnership { data } = &draft.income[2].income else { unreachable!() };
        assert!(data.qbi.is_none());
    }
}

#[test]
fn partnership_no_statement_preserves_fifo_qualification_and_excludes_current_loss() {
    let mut source = ordinary_partner_source(-1_500, true);
    source.regular.basis.opening = Some(250_000);
    source.regular.basis.prior_loss = 300_000;
    source.regular.risk.start = ScorpRiskStart::Detailed { capacity: 250_000, profit: 0, loss: 0 };
    source.alternative = Some(source.regular.clone());
    source.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2022, 1_000, 500, 1_000, 500), loss_year(2017, 2_000, 0, 2_000, 0)],
        // This unrelated retained history has no current at-risk balance.
        risk: vec![loss_year(2023, 500, 500, 500, 500)],
        passive: vec![],
    });
    let retained = source.qbi_records.clone();
    let released = crate::partnership_qbi::historical_without_statement(
        &source,
        crate::kernel::carry::Amount::new(0).unwrap(),
    )
    .unwrap()
    .unwrap();
    assert_eq!(crate::kernel::ratio::dollars(released.historical.get()), 250);
    let remaining: Vec<_> = released.carries[0]
        .remaining
        .iter()
        .filter(|year| year.remaining.get() > 0)
        .map(|year| {
            (
                year.year,
                crate::kernel::ratio::dollars(year.remaining.get()),
                crate::kernel::ratio::dollars(year.remaining_qbi.get()),
            )
        })
        .collect();
    assert_eq!(remaining, vec![(2022, 500, 250), (2026, 1_500, 0)]);
    assert!(released.carries[1].remaining.iter().all(|year| year.remaining.get() == 0));
    assert_eq!(source.qbi_records, retained);
    let draft = with_current_qualified_profit(source);
    let result = supported(&draft);
    assert_eq!(value(&result, "qbi_deduction"), 1_950);
    assert!(noted(&result, "WARN_K1_QBI_LOSS_NO_STATEMENT"));
}

#[test]
fn partnership_no_statement_ignores_unassociated_histories_but_rejects_malformed_records() {
    let mut source = ordinary_partner_source(10_000, true);
    source.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2024, 1_000, 500, 1_000, 500)],
        risk: vec![],
        passive: vec![],
    });
    assert!(source.valid());
    assert!(
        crate::partnership_qbi::historical_without_statement(
            &source,
            crate::kernel::carry::Amount::new(0).unwrap()
        )
        .unwrap()
        .is_none()
    );
    let draft = with_current_qualified_profit(source.clone());
    let result = supported(&draft);
    assert_eq!(value(&result, "qbi_deduction"), 2_000);
    assert!(
        !result
            .review_notes
            .iter()
            .any(|note| note.display.key == "NOTE_PARTNERSHIP_HISTORICAL_QBI_RELEASE"
                && note.context.as_ref().map(crate::IssueContext::identity)
                    == Some(crate::federal_model::NoteIdentity::Income(draft.income[2].id)))
    );
}

#[test]
fn partnership_no_statement_requires_matching_actual_suspended_history() {
    let mut source = ordinary_partner_source(0, true);
    source.regular.basis.prior_loss = 400_000;
    source.alternative = Some(source.regular.clone());
    let draft = with_current_qualified_profit(source.clone());
    let result = supported(&draft);
    assert!(noted(&result, "WARN_PARTNERSHIP_QBI_HISTORY_LEFT_OUT"));
    source.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2024, 3_000, 1_500, 3_000, 1_500)],
        risk: vec![],
        passive: vec![],
    });
    let error = crate::partnership_qbi::historical_without_statement(
        &source,
        crate::kernel::carry::Amount::new(0).unwrap(),
    )
    .err()
    .unwrap();
    assert_eq!(
        error.0,
        "ISSUE_PARTNERSHIP_QBI_RECONCILE_HISTORICAL_PARTNERSHIP_QUALIFIED_LOSS_RECORDS_WITH_36AD0363"
    );
}

// Line audit QBI-D7 (fix unit F12): a released earlier-year qualified loss is part of Form 8995
// line 3 ("qualified portion of trade or business (loss) carryforward allowed in calculating
// taxable income"), not a line 1 row (2026 i8995 lines 1 and 3). A current partnership with QBI
// 10,000 and a second partnership whose 2024 basis-suspended loss of 4,000 (qualified 2,000) is
// released this year: line 1 is one row of 10,000, line 2 = 10,000, line 3 = (2,000), line 4 =
// 8,000, line 5 = 1,600.
#[test]
fn qbi_d7_a_released_qualified_loss_is_form_8995_line_3() {
    let mut source = ordinary_partner_source(0, true);
    source.regular.basis.prior_loss = 400_000;
    source.alternative = Some(source.regular.clone());
    source.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2024, 4_000, 2_000, 4_000, 2_000)],
        risk: vec![],
        passive: vec![],
    });
    source.qbi_records.qualified_se_gross = 900_000;
    let draft = with_current_qualified_profit(source);
    let result = supported(&draft);
    let rows = |line: &str| -> Vec<i64> {
        result
            .lines
            .iter()
            .filter(|l| {
                l.form_line.as_ref().is_some_and(|f| f.form == TaxForm::Form8995 && f.line == line)
            })
            .map(|l| l.amount / 100)
            .collect()
    };
    assert_eq!(rows("1"), vec![10_000]);
    assert_eq!(rows("2"), vec![10_000]);
    assert_eq!(rows("3"), vec![-2_000]);
    assert_eq!(rows("4"), vec![8_000]);
    assert_eq!(rows("5"), vec![1_600]);
    assert_eq!(value(&result, "qbi_deduction"), 1_600);
}

// i8995 Tracking Losses: each later limitation retains the qualification
// actually delivered by the previous owner; a new pool is not a new business.
#[test]
fn ptp_retained_ordinary_losses_conserve_qualification_through_actual_three_limits() {
    for statement in [false, true] {
        let mut source = ordinary_partner_source(300, false);
        source.publicly_traded = true;
        source.regular.basis.opening = Some(170_000);
        source.regular.basis.prior_loss = 400_000;
        source.regular.risk.start =
            ScorpRiskStart::Detailed { capacity: 20_000, profit: 0, loss: 0 };
        source.qbi_records.history = Some(PartnerQbiHistory {
            basis: vec![loss_year(2023, 4_000, 3_000, 4_000, 3_000)],
            risk: vec![],
            passive: vec![],
        });
        source.alternative = Some(source.regular.clone());
        if statement {
            source.qbi = Some(partner_qbi_statement(0));
        }
        let pool = source.pool(&source.regular).unwrap();
        // Own nonqualified profit300 raises basis1700 to2000 and risk200
        // to500. It releases300 of the500 reaching the same PTP's PAL.
        assert_eq!(
            pool.activity.basis.allowed[crate::k1_loss::K1LossClass::Ordinary].get(),
            200_000
        );
        assert_eq!(
            pool.activity.ledger.risk_allowed[crate::k1_loss::K1LossClass::Ordinary].get(),
            50_000
        );
        let suspended = crate::kernel::carry::Amount::new(20_000).unwrap();
        assert_eq!(pool.passive, -20_000);
        let retained = source.qbi_records.clone();
        let (qualified, carries) = if let Some(qbi) = &source.qbi {
            let prepared = qbi
                .calculate_combined_with_pool(
                    &source,
                    suspended,
                    0,
                    (300_000, FederalStatus::Single),
                    &Default::default(),
                    &Default::default(),
                    Some(&pool),
                )
                .unwrap();
            assert_eq!(prepared.ptp, crate::kernel::ratio::exact::whole(-22_500));
            assert_eq!(prepared.current_loss_cents.get(), 0);
            (prepared.historical_cents.get(), prepared.carries)
        } else {
            let prepared = crate::partnership_qbi::historical_without_statement_with_pool(
                &source,
                suspended,
                Some(&pool),
            )
            .unwrap()
            .unwrap();
            (prepared.historical.get(), prepared.carries)
        };
        assert_eq!(qualified, 22_500);
        let remaining: Vec<_> = carries
            .iter()
            .map(|stage| {
                stage.remaining.iter().fold((0, 0), |(loss, qbi), row| {
                    (loss + row.remaining.get(), qbi + row.remaining_qbi.get())
                })
            })
            .collect();
        assert_eq!(remaining, vec![(200_000, 150_000), (150_000, 112_500), (20_000, 15_000)]);
        assert_eq!(source.qbi_records, retained);
    }
}

#[test]
fn ptp_current_loss_keeps_qualified_carry_at_its_actual_limiting_stage() {
    for (stage, loss, service, taxable, qualified) in [
        (2, 100_000, true, 239_250, 50_000),
        (0, 100_000, false, 300_000, 100_000),
        (1, 100_000, false, 300_000, 100_000),
        // One source cent at the50% SSTB midpoint becomes one qualified
        // carry cent, half-up at the qualified-loss owner's cent boundary.
        (0, 1, true, 239_250, 1),
    ] {
        let mut source = ordinary_partner_source(0, false);
        source.publicly_traded = true;
        source.regular.ordinary = -loss;
        source.regular.basis.opening = Some(if stage == 0 { 0 } else { loss });
        source.regular.risk.start = ScorpRiskStart::Detailed {
            capacity: if stage == 1 { 0 } else { loss },
            profit: 0,
            loss: 0,
        };
        let mut statement = partner_qbi_statement(0);
        statement.income = -loss;
        statement.service = service;
        source.qbi = Some(statement);
        let pool = source.pool(&source.regular).unwrap();
        let held = if stage == 2 { loss } else { 0 };
        assert_eq!(pool.passive, -held);
        let prepared = source
            .qbi
            .as_ref()
            .unwrap()
            .calculate_combined_with_pool(
                &source,
                crate::kernel::carry::Amount::new(held).unwrap(),
                0,
                (taxable, FederalStatus::Single),
                &Default::default(),
                &Default::default(),
                Some(&pool),
            )
            .unwrap();
        assert_eq!(prepared.ptp, crate::kernel::ratio::exact::whole(0));
        assert_eq!(prepared.current_loss_cents.get(), 0);
        let row =
            prepared.carries[stage].remaining.iter().find(|row| row.remaining.get() > 0).unwrap();
        assert_eq!(
            (row.year, row.remaining.get(), row.remaining_qbi.get()),
            (2026, loss, qualified)
        );
        for (other, carry) in
            prepared.carries.iter().enumerate().filter(|(other, _)| *other != stage)
        {
            assert!(carry.remaining.iter().all(|row| row.remaining.get() == 0), "stage{other}");
        }
    }
}

#[test]
fn ptp_retained_ordinary_history_requires_its_actual_suspended_balance() {
    let mut source = ordinary_partner_source(0, false);
    source.publicly_traded = true;
    source.regular.basis.prior_loss = 400_000;
    source.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2023, 4_000, 3_000, 3_000, 2_250)],
        risk: vec![],
        passive: vec![],
    });
    let expected = "ISSUE_PARTNERSHIP_QBI_RECONCILE_HISTORICAL_PARTNERSHIP_QUALIFIED_LOSS_RECORDS_WITH_36AD0363";
    let error = crate::partnership_qbi::historical_without_statement(
        &source,
        crate::kernel::carry::Amount::new(0).unwrap(),
    )
    .err()
    .unwrap();
    assert_eq!(error.0, expected);
    source.qbi = Some(partner_qbi_statement(0));
    let error = source
        .qbi
        .as_ref()
        .unwrap()
        .calculate(
            &source,
            crate::kernel::carry::Amount::new(0).unwrap(),
            0,
            300_000,
            FederalStatus::Single,
        )
        .err()
        .unwrap();
    assert_eq!(error.0, expected);
}

// Reg.1.199A-3(b)(1)(iv): an original qualified loss enters199A only
// when deductible under all Code limits. The zero ordinary-QBI placeholder
// cannot establish how section461(l) deferred this separate PTP component.
#[test]
fn ptp_retained_qualified_loss_with_excess_business_loss_requires_source_allocation() {
    for statement in [true, false] {
        let mut source = ordinary_partner_source(0, true);
        source.publicly_traded = true;
        source.regular.basis.opening = Some(40_000_000);
        source.regular.basis.prior_loss = 40_000_000;
        source.regular.risk.start =
            ScorpRiskStart::Detailed { capacity: 40_000_000, profit: 0, loss: 0 };
        source.qbi_records.history = Some(PartnerQbiHistory {
            basis: vec![loss_year(2023, 400_000, 300_000, 400_000, 300_000)],
            risk: vec![],
            passive: vec![],
        });
        source.alternative = Some(source.regular.clone());
        if statement {
            source.qbi = Some(partner_qbi_statement(0));
        }
        // These are the actual financial owner's allowed losses, with the
        // documented75% original qualification kept independently above.
        let pool = source.pool(&source.regular).unwrap();
        assert_eq!(pool.nonpassive, -40_000_000);
        assert_eq!(pool.passive, 0);
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(FederalIncome::Partnership { data: source }));
        let result = estimate(&draft);
        // A normal refusal clears calculation rows but retains the actual
        // financial worksheet. Its observed EBL144000 is independent of the
        // qualified-source requirement tested below; do not invent a row.
        let financial = result
            .review_notes
            .iter()
            .find(|note| note.display.key == "NOTE_PARTNERSHIP_REGULAR_EXCESS_BUSINESS_LOSS")
            .expect("actual regular financial loss worksheet");
        assert_eq!(
            financial.display.args,
            vec![MessageArg::usd_dollars(-400_000), MessageArg::usd_dollars(144_000),]
        );
        // Original qualification is75%: EBL144000 defers108000 qualified
        // and36000 nonqualified, leaving192000 deductible PTP loss. The
        // still-EBL-suspended108000 is not an ordinary or REIT/PTP carry.
        // Before this adapter, NoStatement reaches the existing97A91752
        // refusal; the statement path silently retains the full300000 carry.
        let ptp = result
            .lines
            .iter()
            .find(|line| line.id == "reit_ptp_loss_carryforward")
            .map(|line| line.amount);
        let ordinary = result
            .lines
            .iter()
            .find(|line| line.id == "qbi_loss_carryforward")
            .map(|line| line.amount);
        let character = result
            .review_notes
            .iter()
            .find(|note| note.display.key == "NOTE_PARTNERSHIP_EXCESS_LOSS_CHARACTER")
            .map(|note| &note.display.args);
        assert_eq!(
            (result.state, result.tax, ptp, ordinary),
            (ResultState::EstimatedForSupportedScope, Some(0), Some(19_200_000), Some(0)),
            "statement={statement}; issues={:?}; EBL character={character:?}",
            result.issues,
        );
        assert_eq!(
            character,
            Some(&vec![
                MessageArg::usd_dollars(144_000),
                MessageArg::usd_dollars(108_000),
                MessageArg::usd_dollars(36_000),
            ])
        );
    }
}

#[test]
fn ptp_excess_loss_uses_current_and_original_qualified_sources_without_rephasing() {
    for historical in [false, true] {
        let mut source =
            ordinary_partner_source(if historical { -200_000 } else { -400_000 }, true);
        source.publicly_traded = true;
        source.regular.basis.opening = Some(40_000_000);
        source.regular.risk.start =
            ScorpRiskStart::Detailed { capacity: 40_000_000, profit: 0, loss: 0 };
        source.qbi = Some(partner_qbi_statement(if historical { -100_000 } else { -300_000 }));
        if historical {
            source.regular.basis.prior_loss = 20_000_000;
            source.qbi_records.history = Some(PartnerQbiHistory {
                basis: vec![loss_year(2023, 200_000, 200_000, 200_000, 200_000)],
                risk: vec![],
                passive: vec![],
            });
        }
        source.alternative = Some(source.regular.clone());
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(FederalIncome::Partnership { data: source }));
        let result = supported(&draft);
        // Both actual source combinations have financial loss400000 and
        // qualified loss300000: either current75%, or current50% together
        // with a fully qualified retained200000. EBL144000 defers108000.
        assert_eq!(value(&result, "excess_business_loss"), 144_000);
        assert_eq!(value(&result, "reit_ptp_loss_carryforward"), 192_000);
        assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
        let character = result
            .review_notes
            .iter()
            .find(|note| note.display.key == "NOTE_PARTNERSHIP_EXCESS_LOSS_CHARACTER")
            .unwrap();
        assert_eq!(
            character.display.args,
            vec![
                MessageArg::usd_dollars(144_000),
                MessageArg::usd_dollars(108_000),
                MessageArg::usd_dollars(36_000),
            ]
        );
    }
}

#[test]
fn mixed_ptp_and_ordinary_loss_only_sources_keep_original_fractions_and_categories() {
    for (statement, ptp_qualified, ptp_carry, deferred, nonqualified) in [
        (false, 300_000, 153_600, 170_800, 73_200),
        (true, 300_000, 153_600, 170_800, 73_200),
        // A documented pre2018 financial loss has real zero qualification;
        // it still shares the financial EBL pool with the ordinary loss.
        (false, 0, 0, 24_400, 219_600),
        (true, 0, 0, 24_400, 219_600),
    ] {
        let mut draft = complete(FederalStatus::Single);
        for (ptp, loss, qualified) in [(true, 400_000, ptp_qualified), (false, 100_000, 50_000)] {
            let mut source = ordinary_partner_source(0, true);
            source.publicly_traded = ptp;
            source.regular.basis.opening = Some(loss * 100);
            source.regular.basis.prior_loss = loss * 100;
            source.regular.risk.start =
                ScorpRiskStart::Detailed { capacity: loss * 100, profit: 0, loss: 0 };
            source.qbi_records.history = Some(PartnerQbiHistory {
                basis: vec![loss_year(
                    if ptp && qualified == 0 { 2017 } else { 2023 },
                    loss,
                    qualified,
                    loss,
                    qualified,
                )],
                risk: vec![],
                passive: vec![],
            });
            if ptp && statement {
                source.qbi = Some(partner_qbi_statement(0));
            }
            source.alternative = Some(source.regular.clone());
            add(&mut draft, income(FederalIncome::Partnership { data: source }));
        }
        let result = supported(&draft);
        // Actual historical fractions75%/50%, never current SSTB guesses.
        // Total500000/EBL244000 defers350000×244000/500000=170800:
        // PTP146400 and ordinary24400, leaving153600 and25600.
        assert_eq!(value(&result, "excess_business_loss"), 244_000);
        assert_eq!(value(&result, "reit_ptp_loss_carryforward"), ptp_carry);
        assert_eq!(value(&result, "qbi_loss_carryforward"), 25_600);
        let character = result
            .review_notes
            .iter()
            .find(|note| note.display.key == "NOTE_PARTNERSHIP_EXCESS_LOSS_CHARACTER")
            .unwrap();
        assert_eq!(
            character.display.args,
            vec![
                MessageArg::usd_dollars(244_000),
                MessageArg::usd_dollars(deferred),
                MessageArg::usd_dollars(nonqualified),
            ]
        );
    }
}

#[test]
fn ptp_loss_and_separate_business_property_keep_their_named_allocation_boundary() {
    let mut draft = property_excess_draft(100_000, 100_000, 100_000, 0, FederalStatus::Single);
    let mut source = ordinary_partner_source(-400_000, true);
    source.publicly_traded = true;
    source.regular.basis.opening = Some(40_000_000);
    source.regular.risk.start =
        ScorpRiskStart::Detailed { capacity: 40_000_000, profit: 0, loss: 0 };
    source.qbi = Some(partner_qbi_statement(-300_000));
    source.alternative = Some(source.regular.clone());
    assert_eq!(source.pool(&source.regular).unwrap().nonpassive, -40_000_000);
    add(&mut draft, income(FederalIncome::Partnership { data: source }));
    let result = estimate(&draft);
    // The independent property owner supplies an actual separate100000
    // ordinary business-property loss; it is not this PTP's loss character.
    let property = result
        .review_notes
        .iter()
        .find(|note| note.display.key == "NOTE_PROPERTY_RETURN_INCOME_STAGE")
        .expect("actual separate property-income worksheet");
    assert_eq!(property.display.args[0], MessageArg::usd_dollars(-100_000));
    assert_eq!(result.state, ResultState::Unsupported);
    assert_eq!(result.tax, None);
    let issue = result
        .issues
        .iter()
        .find(|issue| issue.display.key == "ISSUE_PARTNER_EXCESS_LOSS_QBI_AND_PROPERTY_CHARACTER")
        .unwrap_or_else(|| {
            panic!("missing source/property allocation boundary: {:?}", result.issues)
        });
    assert_eq!(
        issue.context.identity(),
        crate::IssueContext::Section(TaxSection::PassThrough).identity()
    );
}

#[test]
fn ptp_excess_loss_requires_actual_gross_attribution_for_positive_adjustments() {
    for (code, raw, qualified, deduction, financial, key) in [
        (
            crate::PartnerIncomeCode::BasisAdjustmentIncome,
            600_000,
            500_000,
            0,
            400_000,
            "ISSUE_PARTNERSHIP_QBI_A_PUBLICLY_TRADED_PARTNERSHIP_INTEREST_WITH_77211554",
        ),
        // F+200000 is still an individual positive adjustment when V400000
        // makes their combined box-one adjustment negative200000.
        (
            crate::PartnerIncomeCode::BasisAdjustmentIncome,
            400_000,
            300_000,
            400_000,
            600_000,
            "ISSUE_PARTNERSHIP_QBI_A_PUBLICLY_TRADED_PARTNERSHIP_INTEREST_WITH_77211554",
        ),
        // 2026-10-19: R flows through Form4797 ordinary income into the
        // separate property-business subtotal200000. Its existing outer
        // property-allocation boundary precedes PTP native reconciliation.
        (
            crate::PartnerIncomeCode::SpeciallyAllocatedOrdinary,
            600_000,
            500_000,
            0,
            400_000,
            "ISSUE_PARTNER_EXCESS_LOSS_QBI_AND_PROPERTY_CHARACTER",
        ),
    ] {
        let mut source = ordinary_partner_source(-raw, true);
        source.publicly_traded = true;
        source.regular.basis.opening = Some(60_000_000);
        source.regular.risk.start =
            ScorpRiskStart::Detailed { capacity: 60_000_000, profit: 0, loss: 0 };
        source.qbi = Some(partner_qbi_statement(-qualified));
        source.income = vec![crate::PartnerIncomeTotal { code, amount: 20_000_000 }];
        if deduction != 0 {
            source.deductions = vec![crate::PartnerDeduction {
                code: crate::PartnerDeductionCode::BasisAdjustmentDeduction,
                amount: deduction * 100,
            }];
        }
        source.alternative = Some(source.regular.clone());
        assert_eq!(source.pool(&source.regular).unwrap().combined.nonpassive, -financial * 100);
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(FederalIncome::Partnership { data: source }));
        let id = draft.income.last().unwrap().id;
        let result = estimate(&draft);
        let issue =
            result.issues.iter().find(|issue| issue.display.key == key).unwrap_or_else(|| {
                panic!("case {code:?}, raw={raw}, qualified={qualified}, deduction={deduction}: missing {key}: {:?}", result.issues)
            });
        let expected_context = if code == crate::PartnerIncomeCode::SpeciallyAllocatedOrdinary {
            crate::IssueContext::Section(TaxSection::PassThrough).identity()
        } else {
            crate::federal_model::NoteIdentity::Income(id)
        };
        assert_eq!(issue.context.identity(), expected_context);
        assert_eq!(result.state, ResultState::Unsupported);
        assert_eq!(result.tax, None);
    }
}

#[test]
fn ptp_excess_loss_accepts_a_negative_only_box_one_adjustment() {
    let mut source = ordinary_partner_source(-400_000, true);
    source.publicly_traded = true;
    source.regular.basis.opening = Some(60_000_000);
    source.regular.risk.start =
        ScorpRiskStart::Detailed { capacity: 60_000_000, profit: 0, loss: 0 };
    source.qbi = Some(partner_qbi_statement(-300_000));
    source.deductions = vec![crate::PartnerDeduction {
        code: crate::PartnerDeductionCode::BasisAdjustmentDeduction,
        amount: 20_000_000,
    }];
    source.alternative = Some(source.regular.clone());
    assert_eq!(source.pool(&source.regular).unwrap().combined.nonpassive, -60_000_000);
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Partnership { data: source }));
    let result = supported(&draft);
    // Actual600000 financial loss/qualified300000; EBL344000 defers172000.
    assert_eq!(value(&result, "excess_business_loss"), 344_000);
    assert_eq!(value(&result, "reit_ptp_loss_carryforward"), 128_000);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 0);
}

#[test]
fn fractional_native_ptp_losses_join_ebl_even_when_each_financial_row_rounds_to_zero() {
    let mut source = ordinary_partner_source(0, true);
    source.regular.basis.opening = Some(40_000_000);
    source.regular.basis.prior_loss = 40_000_000;
    source.regular.risk.start =
        ScorpRiskStart::Detailed { capacity: 40_000_000, profit: 0, loss: 0 };
    source.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2023, 400_000, 300_000, 400_000, 300_000)],
        risk: vec![],
        passive: vec![],
    });
    source.alternative = Some(source.regular.clone());
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Partnership { data: source }));
    for cents in [49, 2] {
        let mut source = ordinary_partner_source(0, true);
        source.publicly_traded = true;
        source.regular.ordinary = -cents;
        let mut statement = partner_qbi_statement(0);
        statement.income = -cents;
        source.qbi = Some(statement);
        source.alternative = Some(source.regular.clone());
        assert_eq!(source.pool(&source.regular).unwrap().combined.nonpassive, -cents);
        add(&mut draft, income(FederalIncome::Partnership { data: source }));
    }
    let result = supported(&draft);
    // Existing reported Form461 fraction144000/400000, source-qualified
    // cents retained: tiny PTP51c loses18c, leaving33c→zeroDollar carry.
    // Skipping the Dollar-zero sources would instead join51c→oneDollar.
    assert_eq!(value(&result, "excess_business_loss"), 144_000);
    assert_eq!(value(&result, "reit_ptp_loss_carryforward"), 0);
    assert_eq!(value(&result, "qbi_loss_carryforward"), 192_000);
}

#[test]
fn ptp_qualified_history_loss_with_native_financial_profit_retains_its_source_boundary() {
    let mut source = ordinary_partner_source(500, true);
    source.publicly_traded = true;
    source.regular.passive_loss = 40_000;
    source.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![],
        risk: vec![],
        passive: vec![loss_year(2023, 400, 300, 400, 300)],
    });
    let mut statement = partner_qbi_statement(500);
    statement.service = true;
    source.qbi = Some(statement);
    source.alternative = Some(source.regular.clone());
    assert_eq!(source.pool(&source.regular).unwrap().combined.nonpassive, 10_000);
    let mut ordinary = ordinary_partner_source(-400_000, true);
    ordinary.regular.basis.opening = Some(40_000_000);
    ordinary.regular.risk.start =
        ScorpRiskStart::Detailed { capacity: 40_000_000, profit: 0, loss: 0 };
    ordinary.alternative = Some(ordinary.regular.clone());
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 60_000_000, 0);
    add(&mut draft, income(FederalIncome::Partnership { data: ordinary }));
    add(&mut draft, income(FederalIncome::Partnership { data: source }));
    let id = draft.income.last().unwrap().id;
    let result = estimate(&draft);
    // Business−399900 gives EBL143900. Wage600000 yields AGI344000
    // and before-QBI327900 after standard16100, fully excluding current
    // SSTB500 while preserving original historical qualified loss300.
    let financial = result
        .review_notes
        .iter()
        .find(|note| note.display.key == "NOTE_PARTNERSHIP_REGULAR_EXCESS_BUSINESS_LOSS")
        .unwrap();
    assert_eq!(
        financial.display.args,
        vec![MessageArg::usd_dollars(-399_900), MessageArg::usd_dollars(143_900),]
    );
    let ptp = result
        .review_notes
        .iter()
        .find(|note| note.display.key == "NOTE_PARTNERSHIP_PTP_QBI")
        .unwrap();
    assert_eq!(ptp.display.args, vec![MessageArg::Number(id), MessageArg::usd_dollars(-300)]);
    let issue = result
        .issues
        .iter()
        .find(|issue| {
            issue.display.key
                == "ISSUE_PARTNERSHIP_QBI_RECONCILE_PARTNERSHIP_QBI_WITH_THE_RESOLVED_708814BB"
        })
        .unwrap_or_else(|| {
            panic!("missing net-profit/history source boundary: {:?}", result.issues)
        });
    assert_eq!(issue.context.identity(), crate::federal_model::NoteIdentity::Income(id));
    assert_eq!(result.state, ResultState::Unsupported);
    assert_eq!(result.tax, None);
}

#[test]
fn negative_scalar_ptp_components_with_ebl_keep_each_issuers_source_boundary() {
    for partnership in [false, true] {
        let mut loss = ordinary_partner_source(-400_000, true);
        loss.regular.basis.opening = Some(40_000_000);
        loss.regular.risk.start =
            ScorpRiskStart::Detailed { capacity: 40_000_000, profit: 0, loss: 0 };
        loss.qbi = Some(partner_qbi_statement(0));
        loss.alternative = Some(loss.regular.clone());
        let negative = if partnership {
            let mut source = ordinary_partner_source(0, true);
            source.qbi = Some(partner_qbi_statement(0));
            source.qbi.as_mut().unwrap().ptp_income = -100_000;
            FederalIncome::Partnership { data: source }
        } else {
            let mut source = complete_ordinary_scorp(0, true);
            source.qbi_statement.as_mut().unwrap().ptp_income = -100_000;
            FederalIncome::Scorp { data: crate::tests::support::builders::scorp_source(source) }
        };
        let mut positive = ordinary_partner_source(0, true);
        positive.qbi = Some(partner_qbi_statement(0));
        positive.qbi.as_mut().unwrap().ptp_income = 200_000;
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(FederalIncome::Partnership { data: loss }));
        add(&mut draft, income(FederalIncome::Partnership { data: positive }));
        add(&mut draft, income(negative));
        let id = draft.income.last().unwrap().id;
        let result = estimate(&draft);
        // The financial loss400000 creates actual EBL144000. Although the
        // scalar PTP items net POSITIVE1000, this issuer's negative1000 has
        // no underlying financial gross-loss attribution for that limit.
        let financial = result
            .review_notes
            .iter()
            .find(|note| note.display.key == "NOTE_PARTNERSHIP_REGULAR_EXCESS_BUSINESS_LOSS")
            .unwrap();
        assert_eq!(
            financial.display.args,
            vec![MessageArg::usd_dollars(-400_000), MessageArg::usd_dollars(144_000),]
        );
        let issue = result
            .issues
            .iter()
            .find(|issue| issue.display.key == "ISSUE_QBI_PTP_COMPONENT_CLASSIFICATION")
            .unwrap_or_else(|| panic!("missing scalar-source boundary: {:?}", result.issues));
        assert_eq!(issue.context.identity(), crate::federal_model::NoteIdentity::Income(id));
        assert_eq!(result.state, ResultState::Unsupported);
        assert_eq!(result.tax, None);
    }
}
