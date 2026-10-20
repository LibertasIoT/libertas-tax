// Source-authored remaining original-key controls. Parent gates own native
// execution and emitted-schema locator verification for this exact source pin.
use super::unit47_credit_source_validation::{credit_estimate_documents_from_save, credit_refusal};
use super::*;
use crate::tests::foreign::form_1116 as ftc;

fn residual_page(session: &mut Interview, topic: TaxSection, values: Vec<IncomeDocument>) -> P {
    save_topic(
        &session.draft,
        topic,
        values.into_iter().map(|value| document(0, value)).collect(),
    )
}

fn residual_income_case(
    case: &str,
    session: &mut Interview,
    positive: P,
    target: fn(&IncomeDocument) -> bool,
    expected: (&str, &str),
    financial: impl FnOnce(&FederalDraft, &FederalResult),
    mutate: impl FnOnce(&mut IncomeDocument),
) {
    let (key, local) = expected;
    let P::SaveIncomes { topic, .. } = &positive else {
        unreachable!()
    };
    let topic = *topic;
    accepted(&send(session, positive));
    let corrected = open_topic(session, topic);
    let rows = page_documents(&corrected);
    let index = rows.iter().position(|row| target(&row.document)).unwrap();
    // Capital-loss history and Form2555 are data answers, not invented stored
    // income identities. Real income documents use IDs allocated by this Save.
    if matches!(
        rows[index].document,
        IncomeDocument::CapitalLossCarryover(_) | IncomeDocument::Form2555(_)
    ) {
        assert_eq!(rows[index].id, 0);
    } else {
        assert!(rows[index].id > 0);
        assert!(
            session
                .draft
                .income
                .iter()
                .any(|row| row.id == rows[index].id)
        );
    }
    let source_ids: Vec<_> = session.draft.income.iter().map(|row| row.id).collect();
    assert!(source_ids.iter().all(|id| *id > 0));
    let baseline = supported(&session.draft);
    financial(&session.draft, &baseline);
    let mut invalid = corrected.clone();
    let P::SaveIncomes { documents, .. } = &mut invalid else {
        unreachable!()
    };
    mutate(&mut documents.documents[index].document);
    let path = alloc::format!("[documents/documents/{index}/document/{local}]");
    let estimate_request = credit_estimate_documents_from_save(&invalid);
    residual_forged_caps(case, session, &invalid, key, &path);
    credit_refusal(
        &alloc::format!("residual-income-{case}-save"),
        session,
        invalid,
        key,
        &path,
    );
    credit_refusal(
        &alloc::format!("residual-income-{case}-estimate"),
        session,
        estimate_request,
        key,
        &path,
    );
    let saved = session.draft.clone();
    let dirty = session.dirty;
    assert!(matches!(
        send(session, credit_estimate_documents_from_save(&corrected)),
        Reply::Response(P::IncomeRunningEstimate { .. })
    ));
    assert_eq!(session.draft, saved);
    assert_eq!(session.dirty, dirty);
    accepted(&send(session, corrected));
    assert_eq!(
        session.draft, saved,
        "{case}: correction altered accepted source facts/IDs"
    );
    assert_eq!(supported(&session.draft), baseline);
    let reopened = open_topic(session, topic);
    accepted(&send(session, reopened));
    assert_eq!(session.draft, saved);
    let mut cold = restart(saved.clone());
    let reopened = open_topic(&mut cold, topic);
    accepted(&send(&mut cold, reopened));
    assert_eq!(cold.draft, saved);
    assert_eq!(supported(&cold.draft), baseline);
    assert_eq!(
        cold.draft
            .income
            .iter()
            .map(|row| row.id)
            .collect::<Vec<_>>(),
        source_ids
    );
}

// This second adversarial check is additional native authority evidence. Its
// forged display bounds are not mislabeled as a false original comparison.
fn residual_forged_caps(case: &str, session: &mut Interview, invalid: &P, key: &str, path: &str) {
    let P::SaveIncomes { documents, .. } = invalid else {
        unreachable!()
    };
    let Some(index) = documents
        .documents
        .iter()
        .position(|row| matches!(row.document, IncomeDocument::CapitalLossCarryover(_)))
    else {
        return;
    };
    let mut forged = invalid.clone();
    let P::SaveIncomes { documents, .. } = &mut forged else {
        unreachable!()
    };
    documents.documents[index].investment_caps = Some(InvestmentCapitalCap {
        short: 100_000_000_000,
        long: 100_000_000_000,
        amt_short: 100_000_000_000,
        amt_long: 100_000_000_000,
    });
    let calculation = credit_estimate_documents_from_save(&forged);
    credit_refusal(
        &alloc::format!("residual-income-{case}-forged-caps-save"),
        session,
        forged,
        key,
        path,
    );
    credit_refusal(
        &alloc::format!("residual-income-{case}-forged-caps-estimate"),
        session,
        calculation,
        key,
        path,
    );
}

#[test]
fn unit47_bad_debt_purchased_collections_and_personal_purpose_have_save_controls() {
    for branch in 0..5 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let source = if branch == 3 {
            contractor_bad_debt(4_200, 0)
        } else {
            purchased_bad_debt(10_000, 4_000, 200)
        };
        let FederalIncome::NonbusinessBadDebt { data } = source else {
            unreachable!()
        };
        let positive = residual_page(
            &mut session,
            TaxSection::InvestmentSales,
            vec![IncomeDocument::BadDebt(data)],
        );
        let (case, key, field) = match branch {
            0 => (
                "debt-collected-cost",
                "BAD_DEBT_COLLECTED_ABOVE_COST",
                "source/principal_collected",
            ),
            1 => (
                "debt-collected-face",
                "BAD_DEBT_COLLECTED_FACE",
                "source/principal_collected",
            ),
            2 => (
                "debt-collected-face-price",
                "BAD_DEBT_COLLECTED_NEEDS_FACE_PRICE",
                "source/price_paid",
            ),
            3 => {
                ("debt-personal-investment", "BAD_DEBT_PERSONAL_CLAIM_NOT_INVESTMENT", "investment")
            }
            _ => ("debt-purchased-future", "BAD_DEBT_PURCHASED_AFTER_2026", "source/acquired"),
        };
        residual_income_case(
            case,
            &mut session,
            positive,
            |v| matches!(v, IncomeDocument::BadDebt(_)),
            (key, field),
            |draft, result| {
                let id = draft
                    .income
                    .iter()
                    .find(|entry| matches!(entry.income, FederalIncome::NonbusinessBadDebt { .. }))
                    .unwrap()
                    .id;
                // Actual unrecovered purchase cost $4,000 + $200, or personal
                // deposit $4,200; the $3,000 current loss leaves $1,200 carry.
                assert_eq!(
                    value(result, &alloc::format!("bad_debt_{id}_short_term_loss")),
                    -4_200
                );
                assert_eq!(value(result, "capital_gain"), -3_000);
                assert_eq!(value(result, "capital_loss_carryforward"), 1_200);
            },
            |document| {
                let IncomeDocument::BadDebt(debt) = document else {
                    unreachable!()
                };
                if branch == 3 {
                    debt.investment = true;
                    return;
                }
                let BadDebtSource::PurchasedNonsecurity {
                    principal_collected,
                    acquired,
                    ..
                } = &mut debt.source
                else {
                    unreachable!()
                };
                match branch {
                    // Both are below face; source order distinguishes the cost
                    // branch from the later at-face purchase-price requirement.
                    0 => *principal_collected = 420_001,
                    1 => *principal_collected = 1_000_000,
                    2 => *principal_collected = 100_000,
                    _ => *acquired = 20270101,
                }
            },
        );
    }
}

#[test]
fn unit47_investment_capital_parts_use_recomputed_regular_and_amt_carries() {
    for branch in 0..7 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut history = capital_history();
        history.amt = Some(PriorAmtCapitalReturn {
            taxable_income: 10_000_000,
            short: -800_000,
            long: -1_200_000,
            deducted_loss: 300_000,
        });
        history.investment_parts.as_mut().unwrap().amt = Some(InvestmentAmtCapitalParts {
            short: 500_000,
            long: 1_200_000,
        });
        // The published regular worksheet gives $4,000/$10,000. The AMT
        // worksheet uses its full $3,000 deduction: $5,000/$12,000. Equality
        // is accepted; the editable parts never supply their own authority.
        let cap = crate::extended_rules::capital_caps(&history);
        assert_eq!(
            (cap.short, cap.long, cap.amt_short, cap.amt_long),
            (400_000, 1_000_000, 500_000, 1_200_000)
        );
        let positive = residual_page(
            &mut session,
            TaxSection::InvestmentSales,
            vec![IncomeDocument::CapitalLossCarryover(history.into())],
        );
        let (case, key, field) = match branch {
            0 => {
                ("investment-amt-required", "INVESTMENT_AMT_PARTS_REQUIRED", "investment_parts/amt")
            }
            1 => ("investment-long-upper", "INVESTMENT_LT_ABOVE_CARRY", "investment_parts/long"),
            2 => ("investment-long-lower", "INVESTMENT_LT_ABOVE_CARRY", "investment_parts/long"),
            3 => (
                "investment-amt-short-upper",
                "INVESTMENT_AMT_ST_ABOVE_CARRY",
                "investment_parts/amt/short",
            ),
            4 => (
                "investment-amt-short-lower",
                "INVESTMENT_AMT_ST_ABOVE_CARRY",
                "investment_parts/amt/short",
            ),
            5 => (
                "investment-amt-long-upper",
                "INVESTMENT_AMT_LT_ABOVE_CARRY",
                "investment_parts/amt/long",
            ),
            _ => (
                "investment-amt-long-lower",
                "INVESTMENT_AMT_LT_ABOVE_CARRY",
                "investment_parts/amt/long",
            ),
        };
        residual_income_case(
            case,
            &mut session,
            positive,
            |v| matches!(v, IncomeDocument::CapitalLossCarryover(_)),
            (key, field),
            |_, result| {
                assert_eq!(return_line(result, "7a"), -300_000);
                assert_eq!(value(result, "capital_short_carryforward"), 1_000);
                assert_eq!(value(result, "capital_long_carryforward"), 10_000);
            },
            |document| {
                let IncomeDocument::CapitalLossCarryover(history) = document else {
                    unreachable!()
                };
                let parts = history.investment_parts.as_mut().unwrap();
                match branch {
                    0 => parts.amt = None,
                    1 => parts.long = 1_000_001,
                    2 => parts.long = -1,
                    3 => parts.amt.as_mut().unwrap().short = 500_001,
                    4 => parts.amt.as_mut().unwrap().short = -1,
                    5 => parts.amt.as_mut().unwrap().long = 1_200_001,
                    _ => parts.amt.as_mut().unwrap().long = -1,
                }
            },
        );
    }
}

fn open_credits(session: &mut Interview) -> P {
    let Reply::Edit(page) = from_review(session, TaxSection::MoreCredits) else {
        panic!("actual MoreCredits editor")
    };
    page
}
fn estimate_credits(request: &P) -> P {
    let P::SaveMoreCredits {
        allowed_owners,
        foreign_businesses,
        capital_sales,
        foreign_situations,
        mortgage_loans,
        businesses,
        farms,
        clean_vehicle_properties,
        rentals,
        research_businesses,
        entered_credits,
        carryback_credits,
        allowed_carry_filers,
        initial_credit_elections,
        initial_research_choices,
        initial_carbon_oxide_origins,
        initial_renewable_facility_origins,
        value,
        ..
    } = request
    else {
        unreachable!()
    };
    P::EstimateMoreCredits {
        user: TEST_USER,
        allowed_owners: allowed_owners.clone(),
        foreign_businesses: foreign_businesses.clone(),
        capital_sales: capital_sales.clone(),
        foreign_situations: foreign_situations.clone(),
        mortgage_loans: mortgage_loans.clone(),
        businesses: businesses.clone(),
        farms: farms.clone(),
        clean_vehicle_properties: clean_vehicle_properties.clone(),
        rentals: rentals.clone(),
        research_businesses: research_businesses.clone(),
        entered_credits: entered_credits.clone(),
        carryback_credits: carryback_credits.clone(),
        allowed_carry_filers: allowed_carry_filers.clone(),
        initial_credit_elections: initial_credit_elections.clone(),
        initial_research_choices: initial_research_choices.clone(),
        initial_carbon_oxide_origins: initial_carbon_oxide_origins.clone(),
        initial_renewable_facility_origins: initial_renewable_facility_origins.clone(),
        value: value.clone(),
    }
}

#[test]
fn unit47_foreign_business_rows_and_prior_tax_paid_basis_preserve_actual_sources() {
    for branch in 0..4 {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut first = business(1_000_000);
        first.more.source = BusinessSource::ForeignServices {
            confirmed: true,
            totalization_exempt: false,
        };
        let mut second = first.clone();
        second.name = "Second actual foreign business".into();
        accepted(&save_businesses(&mut session, vec![first, second]));
        let ids: Vec<_> = session
            .draft
            .income
            .iter()
            .filter(|entry| matches!(entry.income, FederalIncome::Business { .. }))
            .map(|entry| entry.id)
            .collect();
        assert_eq!(ids.len(), 2);
        assert!(ids[0] > 0 && ids[1] > 0 && ids[0] != ids[1]);
        let category = |id| ForeignTaxCategory::Business {
            business: id,
            separate_books: true,
        };
        let second_category = if branch < 2 {
            category(ids[1])
        } else if branch == 2 {
            ForeignTaxCategory::PriorEmployeeWageTax { income_year: 2025 }
        } else {
            ForeignTaxCategory::PriorPassiveTax { income_year: 2025 }
        };
        let claim = ftc::claim(
            vec![
                ftc::row("MX", category(ids[0]), 0, 100),
                ftc::row("FR", second_category, 0, 100),
            ],
            ForeignCreditHistory::None,
            ftc::no_carryback(),
        );
        let mut positive = open_credits(&mut session);
        let P::SaveMoreCredits {
            foreign_businesses,
            value: page,
            ..
        } = &mut positive
        else {
            unreachable!()
        };
        assert!(ids.iter().all(|id| foreign_businesses.iter().any(|choice| choice.id == *id)));
        page.credits = vec![MoreCredit::ForeignTax(claim.try_into().unwrap())];
        accepted(&send(&mut session, positive));
        let corrected = open_credits(&mut session);
        let baseline = supported(&session.draft);
        let profit: i64 = baseline
            .lines
            .iter()
            .filter(|line| {
                line.form_line
                    .as_ref()
                    .is_some_and(|tag| tag.form == TaxForm::ScheduleC && tag.line == "31")
            })
            .map(|line| line.amount)
            .sum();
        assert_eq!(profit, 2_000_000);
        assert_eq!(value(&baseline, "foreign_branch_MX_gross_income"), 10_000);
        if branch < 2 {
            assert_eq!(value(&baseline, "foreign_tax_credit"), 200);
        }
        let mut invalid = corrected.clone();
        let P::SaveMoreCredits { value: page, .. } = &mut invalid else {
            unreachable!()
        };
        let MoreCredit::ForeignTax(ForeignTaxPage::Ordinary { sources, basis, .. }) =
            &mut page.credits[0]
        else {
            unreachable!()
        };
        let (case, key, path) = match branch {
            0 => {
                sources[1].income = 1;
                (
                    "foreign-business-income",
                    "FOREIGN_BUSINESS_INCOME_FROM_SCHEDULE_C",
                    "[value/credits/0/sources/1/income]",
                )
            }
            1 => {
                let ForeignSourceCategory::Business { business, .. } = &mut sources[1].category
                else {
                    unreachable!()
                };
                *business = ids[0];
                (
                    "foreign-business-duplicate",
                    "FOREIGN_BUSINESS_TWICE",
                    "[value/credits/0/sources/1/category/business]",
                )
            }
            2 => {
                *basis = ForeignTaxBasis::Accrued;
                (
                    "foreign-prior-wage-paid",
                    "FOREIGN_PRIOR_TAX_NEEDS_PAID_BASIS",
                    "[value/credits/0/basis]",
                )
            }
            _ => {
                *basis = ForeignTaxBasis::Accrued;
                (
                    "foreign-prior-passive-paid",
                    "FOREIGN_PRIOR_TAX_NEEDS_PAID_BASIS",
                    "[value/credits/0/basis]",
                )
            }
        };
        let calculation = estimate_credits(&invalid);
        credit_refusal(
            &alloc::format!("residual-income-{case}-save"),
            &mut session,
            invalid,
            key,
            path,
        );
        credit_refusal(
            &alloc::format!("residual-income-{case}-estimate"),
            &mut session,
            calculation,
            key,
            path,
        );
        let saved = session.draft.clone();
        let dirty = session.dirty;
        assert!(matches!(
            send(&mut session, estimate_credits(&corrected)),
            Reply::Response(P::RunningEstimate { .. })
        ));
        assert_eq!(session.draft, saved);
        assert_eq!(session.dirty, dirty);
        accepted(&send(&mut session, corrected));
        assert_eq!(session.draft, saved);
        assert_eq!(supported(&session.draft), baseline);
        let reopened = open_credits(&mut session);
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft, saved);
        let mut cold = restart(saved.clone());
        let reopened = open_credits(&mut cold);
        accepted(&send(&mut cold, reopened));
        assert_eq!(cold.draft, saved);
        assert_eq!(supported(&cold.draft), baseline);
        assert_eq!(
            cold.draft
                .income
                .iter()
                .filter(|entry| matches!(entry.income, FederalIncome::Business { .. }))
                .map(|entry| entry.id)
                .collect::<Vec<_>>(),
            ids
        );
    }
}

#[test]
fn unit47_foreign_award_month_sum_refuses_actual_form2555_award_record() {
    // Existing Unit41 published monetary oracle retained, including the
    // accepted wage source. The invalid award changes only its service total.
    let mut session = Interview::new(TEST_USER, crate::tests::income::abroad_clergy::bonus_return());
    let wages = open_topic(&mut session, TaxSection::Wages);
    accepted(&send(&mut session, wages));
    let positive = open_topic(&mut session, TaxSection::WorkAbroad);
    residual_income_case(
        "foreign-award-months",
        &mut session,
        positive,
        |v| matches!(v, IncomeDocument::Form2555(_)),
        ("MULTI_YEAR_PAY_MONTHS_ABOVE_TOTAL", "multi_year_pay/0"),
        |_, result| {
            assert_eq!(value(result, "foreign_earned_taxpayer_income"), 122_500);
            assert_eq!(
                value(result, "foreign_prior_year_taxpayer_exclusion"),
                6_219
            );
            assert_eq!(value(result, "foreign_earned_exclusion"), 128_719);
            assert_eq!(value(result, "agi"), 21_281);
            assert_eq!(return_line(result, "15"), 518_100);
            assert_eq!(return_line(result, "16"), 124_300);
            assert_eq!(result.refund, Some(775_700));
        },
        |document| {
            let IncomeDocument::Form2555(form) = document else {
                unreachable!()
            };
            assert_eq!(
                (
                    form.multi_year_pay[0].months_in_return_year,
                    form.multi_year_pay[0].months_in_prior_year,
                    form.multi_year_pay[0].months_total
                ),
                (2, 7, 24)
            );
            form.multi_year_pay[0].months_total = 8;
        },
    );
}

#[test]
fn federal_cap_widening_compares_purchased_debt_cost_without_overflow() {
    let FederalIncome::NonbusinessBadDebt { mut data } = purchased_bad_debt(10_000, 4_000, 200)
    else {
        unreachable!()
    };
    let BadDebtSource::PurchasedNonsecurity {
        face,
        price_paid,
        acquisition_costs,
        principal_collected,
        ..
    } = &mut data.source
    else {
        unreachable!()
    };
    *face = i64::MAX;
    *price_paid = i64::MAX;
    *acquisition_costs = i64::MAX;
    *principal_collected = 1;
    // Collection is below both actual face and exact acquisition cost, and
    // bought-at-face satisfies the later relationship. Native caps stay owned
    // by source admission; this relationship must not overflow first.
    assert!(data.issue().is_none());
}
