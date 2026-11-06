use super::*;
use crate::cohorts::{cost_cohorts, report_other_costs};
use crate::tests::cross::interview::{
    business, document, reopened_businesses, save_businesses, save_topic, stored_business,
};
use crate::tests::support::builders::cohorts::*;
#[test]
fn unit17_k_domestic_and_foreign_research_use_different_statutory_periods() {
    let domestic = cohort(
        1_200_000,
        20260301,
        20260701,
        OtherAmortizationKind::Research { facts: research(domestic_location(), capitalization()) },
    );
    let foreign = cohort(
        1_200_000,
        20260301,
        20260701,
        OtherAmortizationKind::Research {
            facts: research(ResearchLocation::Foreign, ResearchTreatment::ForeignMidYear),
        },
    );
    assert_eq!(domestic.recovery(2026, &[]).unwrap().amortization, 120_000);
    assert_eq!(foreign.recovery(2026, &[]).unwrap().amortization, 40_000);
    let expense = cohort(
        1_200_000,
        20260301,
        20260301,
        OtherAmortizationKind::Research {
            facts: research(domestic_location(), ResearchTreatment::CurrentExpense),
        },
    )
    .recovery(2026, &[])
    .unwrap();
    assert_eq!(
        (expense.current_expense, expense.amortization, expense.remaining),
        (1_200_000, 0, 0)
    );
}
#[test]
fn unit17_d1_drilling_starts_in_incurred_month_and_other_preferences_in_tax_year() {
    let drilling = cohort(
        1_200_000,
        20260719,
        20260701,
        OtherAmortizationKind::Preference { election: PreferenceWriteoff::Drilling60 },
    );
    assert_eq!(drilling.recovery(2026, &[]).unwrap().amortization, 120_000);
    let mining = cohort(
        1_200_000,
        20260719,
        20260101,
        OtherAmortizationKind::Preference { election: PreferenceWriteoff::Mining120 },
    );
    assert_eq!(mining.recovery(2026, &[]).unwrap().amortization, 120_000);
    let mut wrong = drilling;
    wrong.began = Some(20260101);
    assert_eq!(wrong.recovery(2026, &[]), Err("AMORTIZATION_CALENDAR_OR_BASIS"));
}
#[test]
fn unit17_d2_creative_end_preserves_schedule_and_current_proceeds_without_retaxing_history() {
    let mut cost = cohort(
        1_800_000,
        20240701,
        20240701,
        OtherAmortizationKind::Creative {
            facts: CreativePropertyCost {
                first_capitalized: 20200101,
                not_set_for_production_three_years: true,
                film_production_trade: true,
                book_writeoff: 20240701,
                original_election: true,
            },
        },
    );
    cost.prior_allowed = SourceCents::Reported { amount: 180_000 };
    cost.ended = Some(AmortizationEnd { date: 20260701, how: crate::DisposalKind::Scrapped });
    let abandoned = cost.recovery(2026, &[]).unwrap();
    assert_eq!((abandoned.amortization, abandoned.ordinary_proceeds), (120_000, Some(0)));
    cost.ended = Some(AmortizationEnd {
        date: 20260701,
        how: crate::DisposalKind::Sold {
            proceeds: 500_000,
            selling_expenses: 0,
            payment: crate::SalePayment::InFull,
            buyer: crate::Buyer::Unrelated,
            livestock: crate::SoldLivestock::No,
        },
    });
    let sold = cost.recovery(2026, &[]).unwrap();
    assert_eq!(
        (sold.amortization, sold.ordinary_proceeds, sold.remaining),
        (120_000, Some(500_000), 1_500_000)
    );
    cost.ended.as_mut().unwrap().date = 20250701;
    assert_eq!(cost.recovery(2026, &[]).unwrap().ordinary_proceeds, Some(0));
    assert_eq!(cost.recovery(2026, &[]).unwrap().amortization, 120_000);
}
#[test]
fn unit17_original_2025_domestic_research_acceleration_is_not_new_2026_full_expensing() {
    let facts = research(
        domestic_location(),
        old_domestic(DomesticResearchAcceleration::Half2025Half2026 {
            unamortized_2024: SourceCents::Reported { amount: 600_000 },
        }),
    );
    let mut cost = cohort(1_200_000, 20230701, 20230701, OtherAmortizationKind::Research { facts });
    cost.prior_allowed = SourceCents::Unavailable; // The actual2024 closing source owns acceleration.
    assert_eq!(cost.recovery(2026, &[]).unwrap().amortization, 300_000);
    let OtherAmortizationKind::Research { facts } = &mut cost.kind else { unreachable!() };
    facts.treatment = old_domestic(DomesticResearchAcceleration::All2025 {
        unamortized_2024: SourceCents::Reported { amount: 600_000 },
    });
    assert_eq!(cost.recovery(2026, &[]).unwrap().amortization, 0);
    cost.incurred = 20260101;
    assert_eq!(cost.recovery(2026, &[]), Err("RESEARCH_TRANSITION_2025"));
}

#[test]
fn unit17_k_research_saves_reopens_and_reduces_business_profit_once() {
    let mut session = restart(empty_return(FederalStatus::Single));
    let mut page = business(10_000_000);
    let mut research = cost(
        1_200_000,
        20260701,
        OtherAmortizationKind::Research { facts: research(domestic_location(), capitalization()) },
    );
    research.incurred = 20260301;
    page.more.other_amortization.push(research.into());
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    let (id, stored) = stored_business(&session);
    assert_eq!(
        stored
            .other_amortization
            .into_iter()
            .map(OtherAmortizableCostPage::from)
            .collect::<Vec<_>>(),
        page.more.other_amortization
    );
    assert_eq!(reopened_businesses(session.draft.clone()), vec![page]);
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_net_profit")), 98_800);
    assert_eq!(value(&result, &alloc::format!("business_{id}_form_4562b_total")), 1_200);
    let rows = draft_return(&result);
    assert!(rows.iter().any(|row| {
        row.form_line
            .as_ref()
            .is_some_and(|line| line.form == TaxForm::Form4562B && line.line == "1(b)")
            && row.money.is_none()
            && status_text(&row.label).contains("2026")
    }));
    let mut again = restart(session.draft.clone());
    let reopened = reopened_businesses(again.draft.clone());
    let request = save_topic(
        &mut again,
        TaxSection::SelfEmployment,
        reopened
            .into_iter()
            .map(|page| {
                document(id, IncomeSource::Record { value: crate::IncomeRecord::Business(page) })
            })
            .collect(),
    );
    accepted(&send(&mut again, request));
    assert_eq!(again.draft, session.draft);
}

#[test]
fn unit17_d2_creative_sale_is_property_ordinary_income_and_not_se_receipts() {
    use crate::tests::cross::interview::{assert_round_trip, catalog_preview_document};

    let mut session = restart(empty_return(FederalStatus::Single));
    let mut page = business(10_000_000);
    let mut item = creative(1_800_000, 20240701);
    item.prior_allowed = SourceCents::Reported { amount: 180_000 };
    item.ended = Some(AmortizationEnd { date: 20260701, how: DisposalKind::Scrapped });
    page.more.other_amortization.push(item.into());
    let before = session.draft.clone();
    let preview = catalog_preview_document(IncomeSource::Record {
        value: crate::IncomeRecord::Business(page.clone()),
    });
    // Business previews deliberately have no standalone amount: profit depends
    // on the whole return. Save below establishes the actual deduction and gain.
    assert!(matches!(
        send(&mut session, preview),
        Reply::Response(P::IncomePreview { preview: None, investment_caps: None })
    ));
    assert_eq!(session.draft, before);
    assert!(!session.dirty);
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    let (id, _) = stored_business(&session);
    let abandoned = supported(&session.draft);
    page.more.other_amortization[0].ended.as_mut().unwrap().how = DisposalKind::Sold {
        proceeds: 500_000,
        selling_expenses: 0,
        payment: SalePayment::InFull,
        buyer: Buyer::Unrelated,
        livestock: crate::SoldLivestock::No,
    };
    let request = save_topic(
        &mut session,
        TaxSection::SelfEmployment,
        vec![document(
            id,
            IncomeSource::Record { value: crate::IncomeRecord::Business(page.clone()) },
        )],
    );
    accepted(&send(&mut session, request));
    let sold = supported(&session.draft);
    assert_eq!(value(&sold, &alloc::format!("business_{id}_net_profit")), 98_800);
    assert_eq!(value(&sold, "self_employment_tax"), value(&abandoned, "self_employment_tax"));
    assert_eq!(value(&sold, "agi") - value(&abandoned, "agi"), 5_000);
    assert_eq!(
        sold.lines
            .iter()
            .filter(|line| line.id == alloc::format!("business_{id}_amortization_0_ordinary"))
            .map(|line| line.amount)
            .sum::<i64>(),
        500_000
    );
    assert_round_trip(&mut session, &[page]);
}

#[test]
fn unit17_creative_annual_cohort_rounds_once_and_reports_one_asset() {
    // Two 15-cent costs in one July cohort: 30×6/180=one cent, whereas
    // rounding each property separately would incorrectly deduct two cents.
    let costs = vec![creative(15, 20260701), creative(15, 20260701)];
    let recoveries = cost_cohorts(&costs, 2026, &[]);
    assert_eq!(
        recoveries.parts.iter().map(|part| part.as_ref().unwrap().amortization).sum::<i64>(),
        1
    );
    assert_eq!(
        recoveries.parts.iter().map(|part| part.as_ref().unwrap().remaining).sum::<i64>(),
        29
    );
    let mut result = FederalResult::empty(0);
    report_other_costs(&costs, &[], &mut result, 2, "Synthetic film business");
    assert_eq!(
        result
            .lines
            .iter()
            .filter(|line| line
                .form_line
                .as_ref()
                .is_some_and(|form| form.form == TaxForm::Form4562B && form.line == "1(c)"))
            .map(|line| line.amount)
            .collect::<Vec<_>>(),
        vec![30]
    );
    assert_eq!(
        result
            .lines
            .iter()
            .filter(|line| line
                .form_line
                .as_ref()
                .is_some_and(|form| form.form == TaxForm::Form4562B && form.line == "1(f)"))
            .map(|line| line.amount)
            .collect::<Vec<_>>(),
        vec![1]
    );
}

#[test]
fn pollution_termination_excludes_its_month_and_keeps_original_basis() {
    let mut cost = pollution();
    assert_eq!(cost.recovery(2026, &[]).unwrap().amortization, 240_000);
    let OtherAmortizationKind::Pollution { facts } = &mut cost.kind else { unreachable!() };
    facts.terminated = Some(20260701);
    let amounts = cost.recovery(2026, &[]).unwrap();
    assert_eq!((amounts.amortization, amounts.remaining), (120_000, 1_080_000));
}

#[test]
fn lease_renewal_threshold_and_research_credit_use_actual_sources() {
    let mut cost = lease();
    assert_eq!(cost.recovery(2026, &[]).unwrap().amortization, 120_000);
    let OtherAmortizationKind::Lease { facts } = &mut cost.kind else { unreachable!() };
    facts.cost_attributed_to_remaining_term = 899_999;
    assert_eq!(cost.recovery(2026, &[]).unwrap().amortization, 60_000);
    let mut facts = research(domestic_location(), capitalization());
    facts.location =
        ResearchLocation::Domestic { credit_adjustment: SourceCents::Reported { amount: 200_000 } };
    cost = cohort(1_200_000, 20260301, 20260701, OtherAmortizationKind::Research { facts });
    assert_eq!(cost.recovery(2026, &[]).unwrap().amortization, 100_000);
    let OtherAmortizationKind::Research { facts } = &mut cost.kind else { unreachable!() };
    facts.location = ResearchLocation::Domestic { credit_adjustment: SourceCents::Unavailable };
    assert_eq!(cost.recovery(2026, &[]), Err("RESEARCH_CREDIT_ADJUSTMENT"));
    unsupported(&return_with_cohort(cost), "Income:");
}

#[test]
fn noncreative_disposals_reuse_actual_property_recapture_and_preserve_se_profit() {
    for mut cost in [
        pollution(),
        lease(),
        cohort(
            1_200_000,
            20260719,
            20260701,
            OtherAmortizationKind::Preference { election: PreferenceWriteoff::Drilling60 },
        ),
    ] {
        cost.ended = Some(sale(1_500_000));
        let draft = return_with_cohort(cost);
        let (id, data) = cohort_business(&draft);
        let sales = crate::cohorts::property_sales(data).unwrap();
        assert_eq!(sales.len(), 1);
        let sale = &sales[0].1;
        // Short-held property is wholly ordinary. The operating profit still
        // subtracts only actual amortization, and does not count sale receipts.
        assert!(sale.part_two);
        assert_eq!(sale.value.section1231, 0);
        assert_eq!(sale.value.ordinary, sale.realized - sale.cost + sale.recovery);
        let result = if data.other_amortization[0].needs_amt_source(
            data.material_participation.individual(2026, false)
                == crate::participation::Participation::Yes,
        ) {
            unsupported(&draft, "Income:");
            let mut result = FederalResult::empty(0);
            crate::asset_sale::report_amortized_sale(
                &mut result,
                &alloc::format!("business_{id}_amortization_0_disposal"),
                sale,
                "income.business.other_amortization",
            );
            result
        } else {
            let result = supported(&draft);
            assert_eq!(
                value(&result, &alloc::format!("business_{id}_net_profit")),
                100_000 - crate::kernel::ratio::dollars(data.amortization_cents())
            );
            result
        };
        assert_eq!(
            value(&result, &alloc::format!("business_{id}_amortization_0_disposal_ordinary")),
            sale.value.ordinary
        );
    }
}

#[test]
fn condemnation_uses_existing_recapture_and_replacement_recognition() {
    let mut cost = pollution();
    cost.incurred = 20240101;
    cost.began = Some(20240201);
    cost.prior_allowed = SourceCents::Reported { amount: 460_000 };
    cost.ended = Some(AmortizationEnd {
        date: 20260901,
        how: DisposalKind::Condemned { award: 1_500_000, expenses: 0, replacement_cost: 1_400_000 },
    });
    let draft = return_with_cohort(cost);
    let (id, data) = cohort_business(&draft);
    let sales = crate::cohorts::property_sales(data).unwrap();
    assert_eq!(
        (sales[0].1.value.recapture, sales[0].1.value.section1231, sales[0].1.postponed),
        (1_000, 0, 8_400)
    );
    unsupported(&draft, "Income:");
    let mut result = FederalResult::empty(0);
    crate::asset_sale::report_amortized_sale(
        &mut result,
        &alloc::format!("business_{id}_amortization_0_disposal"),
        &sales[0].1,
        "income.business.other_amortization",
    );
    assert_eq!(
        value(&result, &alloc::format!("business_{id}_amortization_0_disposal_ordinary")),
        1_000
    );
}

#[test]
fn mandatory_research_disposition_does_not_invent_property_basis() {
    let facts = research(ResearchLocation::Foreign, ResearchTreatment::ForeignMidYear);
    let mut cost = cohort(1_200_000, 20260301, 20260701, OtherAmortizationKind::Research { facts });
    cost.ended = Some(sale(500_000));
    assert_eq!(cost.recovery(2026, &[]).unwrap().amortization, 40_000);
    unsupported(&return_with_cohort(cost), "Income:");
}

#[test]
fn disposal_form_columns_share_rounding_and_foot_before_classification() {
    // Native Form4797 observations: $10.40→10; $10.60→11; 40c→0.
    // The reported gain is therefore -1, exactly as on existing equipment.
    let end = sale(1_040);
    let sale = crate::asset_sale::amortized_property_sale(crate::asset_sale::AmortizedProperty {
        acquired: 20260101,
        end: &end,
        cost_cents: 1_060,
        recovery_cents: 40,
        section1254: false,
    })
    .unwrap()
    .unwrap();
    assert_eq!((sale.realized, sale.cost, sale.recovery, sale.value.ordinary), (10, 11, 0, -1));
}

#[test]
fn mandatory_research_links_real_property_and_saves_its_disposal_once() {
    let mut session = restart(empty_return(FederalStatus::Single));
    let page = linked_research_business();
    accepted(&save_businesses(&mut session, vec![page]));
    let (id, stored) = stored_business(&session);
    let asset_id = stored.assets[0].id;
    assert!(asset_id > 0);
    assert_eq!(crate::cohorts::property_choices(&stored)[0].id, asset_id);
    let mut page = reopened_businesses(session.draft.clone()).remove(0);
    let OtherAmortizationKindPage::Research { facts } = &mut page.more.other_amortization[0].kind
    else {
        unreachable!()
    };
    facts.property = Some(asset_id);
    page.assets[0].disposal = AssetDisposal::Disposed { date: 20260901, how: sale(1_500_000).how };
    accepted(&crate::tests::support::client::cohorts::save_existing_business(
        &mut session,
        id,
        page.clone(),
    ));
    let result = supported(&session.draft);
    assert_eq!(
        line_amount(&result, &alloc::format!("business_{id}_amortization_0_deduction")),
        40_000
    );
    assert!(crate::cohorts::property_sales(&stored_business(&session).1).unwrap().is_empty());
    assert_eq!(reopened_businesses(restart(session.draft.clone()).draft), vec![page.clone()]);
    let before = session.draft.clone();
    page.assets.clear();
    crate::tests::support::assert::rejected_with(
        crate::tests::support::client::cohorts::save_existing_business(&mut session, id, page),
        "RESEARCH_PROPERTY_SOURCE_INVALID",
    );
    assert_eq!(session.draft, before);
}

#[test]
fn current_research_expense_submits_without_amortization_period_start_or_history() {
    let mut session = restart(empty_return(FederalStatus::Single));
    let mut page = business(10_000_000);
    let item = cohort(
        1_200_000,
        20260301,
        20260301,
        OtherAmortizationKind::Research {
            facts: research(domestic_location(), ResearchTreatment::CurrentExpense),
        },
    );
    assert_eq!(item.began, None);
    assert_eq!(item.prior_allowed, SourceCents::Unavailable);
    page.more.other_amortization.push(item.into());
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    let (id, _) = stored_business(&session);
    assert_eq!(
        value(&supported(&session.draft), &alloc::format!("business_{id}_net_profit")),
        88_000
    );
    assert_eq!(reopened_businesses(restart(session.draft).draft), vec![page]);
}

#[test]
fn elective_research_conversion_uses_actual_whole_basis_and_canonical_property_date() {
    let mut session = restart(empty_return(FederalStatus::Single));
    accepted(&save_businesses(&mut session, vec![converted_research_business()]));
    let (id, stored) = stored_business(&session);
    let mut page = reopened_businesses(session.draft.clone()).remove(0);
    let OtherAmortizationKindPage::Research { facts } = &mut page.more.other_amortization[0].kind
    else {
        unreachable!()
    };
    facts.property = Some(stored.assets[0].id);
    let ResearchTreatment::Capitalize { election } = &mut facts.treatment else { unreachable!() };
    election.transfer = Some(SourceCents::Reported { amount: 840_000 });
    accepted(&crate::tests::support::client::cohorts::save_existing_business(
        &mut session,
        id,
        page.clone(),
    ));
    let result = supported(&session.draft);
    // Jan–June:12,000/60×6=1,200; remaining8,400 enters actual property
    // basis once; its regular first-year depreciation is1,680.
    assert_eq!(
        line_amount(&result, &alloc::format!("business_{id}_amortization_0_deduction")),
        120_000
    );
    assert_eq!(line_amount(&result, &alloc::format!("business_{id}_amortization_0_remaining")), 0);
    assert_eq!(value(&result, &alloc::format!("business_{id}_net_profit")), 97_120);
    assert_eq!(reopened_businesses(restart(session.draft.clone()).draft), vec![page.clone()]);
    // A property credit reduces that owner's depreciable basis after the actual
    // deferred cost was transferred; it does not reduce the original transfer.
    page.assets[0].credit_reduction = 84_000;
    accepted(&crate::tests::support::client::cohorts::save_existing_business(
        &mut session,
        id,
        page.clone(),
    ));
    assert_eq!(
        value(&supported(&session.draft), &alloc::format!("business_{id}_net_profit")),
        97_288
    );
    let OtherAmortizationKindPage::Research { facts } = &mut page.more.other_amortization[0].kind
    else {
        unreachable!()
    };
    let ResearchTreatment::Capitalize { election } = &mut facts.treatment else { unreachable!() };
    election.transfer = Some(SourceCents::Unavailable);
    accepted(&crate::tests::support::client::cohorts::save_existing_business(
        &mut session,
        id,
        page,
    ));
    unsupported(&session.draft, "Income:");
}

#[test]
fn long_preference_and_timber_sales_keep_real_recapture_classes() {
    let mut cost = cohort(
        1_200_000,
        20240719,
        20240101,
        OtherAmortizationKind::Preference { election: PreferenceWriteoff::Mining120 },
    );
    cost.prior_allowed = SourceCents::Reported { amount: 240_000 };
    cost.ended = Some(sale(1_500_000));
    let draft = return_with_cohort(cost);
    let (_, business) = cohort_business(&draft);
    let sales = crate::cohorts::property_sales(business).unwrap();
    assert_eq!((sales[0].1.value.ordinary, sales[0].1.value.section1231), (3_300, 3_000));
    let mut result = FederalResult::empty(0);
    crate::asset_sale::report_amortized_sale(
        &mut result,
        "business_2_amortization_0_disposal",
        &sales[0].1,
        "income.business.other_amortization",
    );
    assert_eq!(
        printed_line(&result, "business_2_amortization_0_disposal_ordinary"),
        (TaxForm::Form4797, "30b".into())
    );
    let mut timber = crate::tests::support::builders::timber::timber_cost();
    timber.incurred = 20240401;
    timber.prior_allowed = SourceCents::Reported { amount: 300_000 };
    timber.ended = Some(sale(3_000_000));
    let sale = crate::cohorts::timber_sale(&timber).unwrap().unwrap();
    assert_eq!((sale.value.ordinary, sale.value.section1231), (14_500, 6_000));
}

#[test]
fn research_ten_year_election_applies_actual_credit_adjustment_without_period_filler() {
    let source = cohort(
        1_200_000,
        20260719,
        20260101,
        OtherAmortizationKind::Research {
            facts: research(
                ResearchLocation::Domestic {
                    credit_adjustment: SourceCents::Reported { amount: 200_000 },
                },
                ResearchTreatment::TenYearElection,
            ),
        },
    );
    let recovery = source.recovery(2026, &[]).unwrap();
    assert_eq!(
        (recovery.basis, recovery.amortization, recovery.remaining),
        (1_000_000, 100_000, 900_000)
    );
}

#[test]
fn separate_amt_sources_remain_unavailable_without_replacing_regular_recovery() {
    let mut draft = return_with_cohort(cohort(
        1_200_000,
        20260301,
        20260301,
        OtherAmortizationKind::Research {
            facts: research(domestic_location(), ResearchTreatment::CurrentExpense),
        },
    ));
    supported(&draft);
    for entry in &mut draft.income {
        if let FederalIncome::Business { data } = &mut entry.income {
            data.material_participation =
                crate::ParticipationFacts { hours: 0, test: crate::ParticipationTest::None };
        }
    }
    // 2026-10-24: current passive C is admitted, exposing the actual
    // missing AMT source. The separate recovery is still never zeroed.
    let excluded = estimate(&draft);
    assert_eq!(excluded.state, ResultState::Unsupported);
    assert!(
        excluded
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_AMORTIZATION_AMT_SOURCE_NEEDED")
    );
    assert!(excluded.tax.is_none());
    let (id, data) = cohort_business(&draft);
    assert!(data.other_amortization[0].needs_amt_source(false));
    assert!(
        !excluded.lines.iter().any(|line| line.id == alloc::format!("business_{id}_net_profit"))
    );
    let mut elected = cohort(
        1_200_000,
        20260301,
        20260101,
        OtherAmortizationKind::Research {
            facts: research(domestic_location(), ResearchTreatment::TenYearElection),
        },
    );
    assert!(!elected.needs_amt_source(false));
    elected.kind =
        OtherAmortizationKind::Research { facts: research(domestic_location(), capitalization()) };
    assert!(!elected.needs_amt_source(false));
    elected.kind = OtherAmortizationKind::Research {
        facts: research(ResearchLocation::Foreign, ResearchTreatment::ForeignMidYear),
    };
    assert!(elected.needs_amt_source(false));
    assert!(!elected.needs_amt_source(true));
    assert!(pollution().needs_amt_source(true));
    unsupported(&return_with_cohort(pollution()), "Income:");
}
