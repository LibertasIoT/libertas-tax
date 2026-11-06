// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn self_employed_tips_require_source_facts_and_claimant_ssn() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else { panic!() };
    business.tips = Some(FederalBusinessTips { amount: 200_000, eligible: false });
    add(&mut draft, income(FederalIncome::Business { data: business }));
    // Not sure claims no deduction, with a note.
    assert!(noted(&supported(&draft), "WARN_BUSINESS_TIPS_LEFT_OUT"));
    {
        let FederalIncome::Business { data: business } = &mut draft.income[1].income else {
            panic!()
        };
        business.tips.as_mut().unwrap().eligible = true;
        business.tips.as_mut().unwrap().amount = 1_100_000;
    }
    unsupported(&draft, "Self-employed tips");
    let FederalIncome::Business { data: business } = &mut draft.income[1].income else { panic!() };
    business.tips.as_mut().unwrap().amount = 200_000;
    draft.people.as_mut().unwrap().taxpayer.valid_ssn = false;
    unsupported(&draft, "Self-employed tips");
}

#[test]
fn declining_conditional_claim_removes_its_facts() {
    let mut draft = complete(FederalStatus::Single);
    draft.adjustments.as_mut().unwrap().student_loan = StudentLoanClaim {
        student_loan_interest: 12345,
        forms: Vec::new(),
        employer_paid_interest: 0,
    };
    assert!(value(&supported(&draft), "adjustments") > 0);
    draft.adjustments.as_mut().unwrap().student_loan = StudentLoanClaim::NONE;
    assert_eq!(value(&supported(&draft), "adjustments"), 0);
    let state = restart(draft);
    assert_eq!(state.draft.adjustments.unwrap().student_loan, StudentLoanClaim::NONE);
}

#[test]
fn proposed_refund_rules_are_advisory_and_do_not_reduce_the_refund() {
    let mut d = empty_return(FederalStatus::Single);
    // No earnings and no refundable credits: no proposed-credit review flag.
    let ordinary = supported(&d);
    assert!(ordinary.review_notes.is_empty());
    // Low earnings produce a childless EIC exceeding the calculated tax.
    d.income.push(wage(1, Owner::Taxpayer, 1_000_000, 0));
    let r = supported(&d);
    assert_eq!(
        r.refund,
        Some((r.payments.unwrap() + r.refundable_credits.unwrap() - r.tax.unwrap()).max(0))
    );
    assert!(r.refundable_credits.unwrap() > r.tax.unwrap());
    assert_eq!(r.review_notes.len(), 1);
    assert!(r.issues.is_empty());
}

#[test]
fn calculated_return_lines_reconcile_to_the_same_refund() {
    for status in [FederalStatus::Single, FederalStatus::Joint] {
        let result = supported(&complete(status));
        let l = |reference| return_line(&result, reference);
        assert_eq!(
            l("9"),
            ["1z", "2b", "3b", "4b", "5b", "6b", "7a", "8"].into_iter().map(l).sum::<i64>()
        );
        assert_eq!(l("11a / 11b"), l("9") - l("10"));
        assert_eq!(l("14"), ["12e", "12f", "13a", "13b"].into_iter().map(l).sum::<i64>());
        assert_eq!(l("15"), (l("11a / 11b") - l("14")).max(0));
        assert_eq!(l("24a"), l("22") + l("23"));
        assert_eq!(l("24a"), result.tax.unwrap());
        assert_eq!(l("24b"), 0);
        assert_eq!(l("24c"), l("24a") + l("24b"));
        assert_eq!(l("33"), l("25d") + l("26") + l("27a") + l("28") + l("29") + l("31"));
        assert_eq!(l("34"), result.refund.unwrap());
        assert_eq!(l("37"), result.balance.unwrap());
    }
}

#[test]
fn prior_return_facts_derive_carryovers_and_survive_restore() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().capital_history = vec![capital_history()];
    // Prior taxable income is -2,000: only 1,000 of the prior 3,000
    // deduction was used, leaving 4,000 short and 10,000 long.
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_short_carryforward"), 1000);
    assert_eq!(value(&r, "capital_long_carryforward"), 10000);
    assert_eq!(restart(d.clone()).draft, d);
    // With no income this year, the annual deduction uses no carryover.
    d.income.clear();
    let r = supported(&d);
    assert_eq!(value(&r, "capital_short_carryforward"), 4000);
    assert_eq!(value(&r, "capital_long_carryforward"), 10000);
}

#[test]
fn partial_2025_cash_fuel_recovery_guards_source_and_history() {
    let good = partial_cash_fuel_recovery();
    for case in [0, 1, 2, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] {
        let mut recovery = good.clone();
        match case {
            0 => recovery.amount = 375_000,
            1 => recovery.amount = 500_001,
            2 => {
                recovery.partial_prior_year_benefit.as_mut().unwrap().original_prior_year_line_14 =
                    1_200_000
            }
            3 => {
                recovery.partial_prior_year_benefit.as_mut().unwrap().refigured_prior_year_line_16 =
                    0
            }
            4 => {
                recovery
                    .partial_prior_year_benefit
                    .as_mut()
                    .unwrap()
                    .refigured_prior_year_line_11b = 1_592_900
            }
            6 => recovery.partial_prior_year_benefit.as_mut().unwrap().comparison_complete = false,
            7 => {
                recovery.partial_prior_year_benefit.as_mut().unwrap().active_unlimited_deduction =
                    false
            }
            8 => recovery.partial_prior_year_benefit.as_mut().unwrap().no_prior_amt = false,
            9 => {
                recovery
                    .partial_prior_year_benefit
                    .as_mut()
                    .unwrap()
                    .no_prior_credits_or_carryovers = false
            }
            10 => recovery.full_tax_benefit = true,
            11 => recovery.niit_recovery_excluded = false,
            12 => recovery.not_reported_elsewhere = false,
            13 => recovery.occurred_in_year = false,
            14 => {
                recovery.partial_prior_year_benefit.as_mut().unwrap().refigured_prior_year_line_14 =
                    1_900_000
            }
            15 => {
                recovery.partial_prior_year_benefit.as_mut().unwrap().originally_deducted_fuel_tax =
                    400_000
            }
            16 => {
                recovery
                    .partial_prior_year_benefit
                    .as_mut()
                    .unwrap()
                    .original_standard_only_deduction = false
            }
            _ => unreachable!(),
        }
        assert!(!recovery.valid() || !recovery.qualified(), "case {case}");
    }
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
        unreachable!()
    };
    business.cash_fuel_tax_recovery = Some(good);
    business.material_participation =
        crate::ParticipationFacts { hours: 0, test: crate::ParticipationTest::None };
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Business { data: business }));
    // 2026-10-24: passive C is calculated; this recovery still needs its
    // owning active/original-year tax-benefit source, not an assumed zero.
    let result = estimate(&draft);
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(result.issues.iter().any(|issue| issue.display.key
        == "ISSUE_CONFIRM_THE_CASH_METHOD_ORIGINAL_PRIOR_YEAR_FORM_4136_RETURN_WAS_FI_DAD735AB"));
    assert!(result.tax.is_none());
}
#[test]
fn simplified_office_caps_area_income_and_preserves_prior_actual_carries() {
    for (profit, expected) in [(10000, 1500), (1000, 1000), (-1000, 0)] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(cash_business(profit)));
        let source = d.income.last().unwrap().id;
        add_simplified_office(&mut d, source, 400);
        let id = d.income[1].id;
        let r = supported(&d);
        assert_eq!(value(&r, &alloc::format!("business_{id}_home_office")), expected);
        assert_eq!(value(&r, &office_line_id(&r, "operating_carryover")), 1000);
        assert_eq!(value(&r, &office_line_id(&r, "depreciation_carryover")), 2000);
        let mut reference = d.clone();
        reference.income[1].income = cash_business(profit - expected);
        reference.home_sites.clear();
        assert_eq!(r.tax, supported(&reference).tax);
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn prior_employer_grace_only_benefit_flows_through_2441_credit_and_full_return() {
    let mut d = benefit_return(FederalStatus::Single);
    grace_only_benefits(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "care_form2441_line12"), 0);
    assert_eq!(value(&r, "care_form2441_line13"), 2_000);
    assert_eq!(value(&r, "care_form2441_line15"), 2_000);
    assert_eq!(value(&r, "care_benefits_excluded"), 2_000);
    assert_eq!(value(&r, "care_benefits_taxable"), 0);
    assert_eq!(value(&r, "care_benefits_excluded"), 2_000);
    assert_eq!(value(&r, "care_credit"), 1_400);
    assert_eq!(return_line(&r, "1a"), 6_000_000);
    assert_eq!(return_line(&r, "1e"), 0);
    assert_eq!(return_line(&r, "25a"), 650_000);
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).return_lines, r.return_lines);
}
#[test]
fn prior_employer_grace_only_requires_actual_amount_and_plan_maximum() {
    let mut d = benefit_return(FederalStatus::Single);
    grace_only_benefits(&mut d);
    // The approved input is the amount actually used in the 2025 plan's
    // grace period. Employer/date/eligibility confirmations are retired;
    // an actual amount and complete plan maximum are the required facts.
    let mut partial = d.clone();
    partial.credits.as_mut().unwrap().care.grace[0].used_in_return_year = 100_000;
    assert_eq!(value(&supported(&partial), "care_benefits_excluded"), 1_000);
    let mut no_maximum = d.clone();
    no_maximum.credits.as_mut().unwrap().care.plan_maximum = CarePlanMaximum::NoBenefits;
    assert_eq!(
        no_maximum.credits.as_ref().unwrap().care.rule(&no_maximum),
        Some("CARE_PLAN_MAXIMUM_REQUIRED")
    );
    let mut missing_cost = d.clone();
    care_costs(&mut missing_cost, 100_000);
    assert_eq!(value(&supported(&missing_cost), "care_benefits_excluded"), 1_000);
    assert_eq!(value(&supported(&missing_cost), "care_benefits_taxable"), 1_000);
    let mut separate = d;
    answer_separate_pages(&mut separate);
    separate.setup.as_mut().unwrap().status =
        FilingChoice::Separate { section179: Section179Spouse::NotSupplied };
    separate.setup.as_mut().unwrap().marital_state =
        MaritalState::Married { living: SpouseLiving::Together };
    separate.credits.as_mut().unwrap().care.filing =
        CareFilingStatus::Separate { spouse_earned_income: None };
    assert_eq!(value(&supported(&separate), "care_benefits_taxable"), 2_000);
}

#[test]
fn domestic_abuse_exception_guards_anniversary_and_shares_claim_limit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, abuse_withdrawal(20000, 10000));
    supported(&d);
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    let mut data = edit_retirement(data);
    let Some(RetirementTreatment::Exception { distributed, .. }) = &mut data.ordinary_distribution
    else {
        unreachable!()
    };
    *distributed = 20260301;
    drop(data);
    unsupported(&d, "Income");
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    let mut data = edit_retirement(data);
    let Some(RetirementTreatment::Exception { distributed, .. }) = &mut data.ordinary_distribution
    else {
        unreachable!()
    };
    *distributed = 20260228;
    drop(data);
    add(&mut d, abuse_withdrawal_plan("Plan B", 20000, 1000));
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 50);
    assert_eq!(value(&r, "retirement_2_qualified_gross"), 10000);
    assert_eq!(value(&r, "retirement_3_qualified_gross"), 500);
    assert_eq!(value(&r, "domestic_abuse_used_taxpayer"), 10500);
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    for entry in d.income.iter_mut().skip(1) {
        let FederalIncome::Retirement { data } = &mut entry.income else { unreachable!() };
        let mut data = edit_retirement(data);
        let Some(RetirementTreatment::Exception {
            facts: RetirementException::DomesticAbuse { prior_used, .. },
            ..
        }) = &mut data.ordinary_distribution
        else {
            unreachable!()
        };
        *prior_used = 500000;
    }
    let prior = supported(&d);
    assert_eq!(value(&prior, "early_distribution_tax"), 550);
    assert_eq!(value(&prior, "domestic_abuse_used_taxpayer"), 10500);
    assert_eq!(estimate(&restart(d.clone()).draft), prior);
    let FederalIncome::Retirement { data } = &mut d.income[2].income else { unreachable!() };
    let mut data = edit_retirement(data);
    let Some(RetirementTreatment::Exception {
        facts: RetirementException::DomesticAbuse { prior_used, .. },
        ..
    }) = &mut data.ordinary_distribution
    else {
        unreachable!()
    };
    *prior_used = 0;
    drop(data);
    unsupported(&d, "Income");
}
#[test]
fn home_exclusion_rejects_unresolved_history_and_duplicate_full_claims() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::HomeSale { data: main_home_source(home_sale(false)) }));
    add(&mut d, income(FederalIncome::HomeSale { data: main_home_source(home_sale(false)) }));
    unsupported(&d, "Income:");
    d.income.pop();
    let FederalIncome::HomeSale { data } = &mut d.income[1].income else { panic!() };
    let mut data = edit_home(data);
    let HomeExclusion::Full { taxpayer, .. } = &mut data.exclusion else { panic!() };
    taxpayer.prior_exclusion = true;
    drop(data);
    unsupported(&d, "Income:");
}

#[test]
fn political_party_debtor_takes_no_bad_debt_loss() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(bad_debt(10000, 0)));
    let FederalIncome::NonbusinessBadDebt {
        data: crate::NonbusinessBadDebt { political_debtor, .. },
    } = &mut d.income[1].income
    else {
        panic!()
    };
    *political_debtor = true;
    let r = supported(&d);
    assert!(!r.lines.iter().any(|line| line.id.starts_with("bad_debt_")));
}

#[test]
fn contractor_deposit_requires_insolvency_personal_work_and_no_investment_claim() {
    for case in [0, 2, 3] {
        let mut d = complete(FederalStatus::Single);
        let mut debt = contractor_bad_debt(6000, 1000);
        let FederalIncome::NonbusinessBadDebt {
            data:
                crate::NonbusinessBadDebt {
                    source:
                        BadDebtSource::ContractorDeposit { contractor_insolvent, personal_work, .. },
                    investment,
                    ..
                },
        } = &mut debt
        else {
            panic!()
        };
        match case {
            0 => *contractor_insolvent = false,

            2 => *personal_work = false,
            _ => *investment = true,
        }
        add(&mut d, income(debt));
        // A personal claim marked investment is a page rule (F3); otherwise
        // no loss, with a note for Not sure and a notice for business work.
        if case == 3 {
            assert!(!d.valid_stored());
            let FederalIncome::NonbusinessBadDebt { data } = &d.income[1].income else { panic!() };
            assert_eq!(data.rule(), Some("BAD_DEBT_PERSONAL_CLAIM_NOT_INVESTMENT"));
            continue;
        }
        let r = supported(&d);
        assert!(!r.lines.iter().any(|line| line.id.starts_with("bad_debt_")));
        assert!(!noted(&r, "WARN_BAD_DEBT_NOT_DEDUCTED"));
        assert_eq!(noted(&r, "WARN_BAD_DEBT_LEFT_OUT"), case == 2);
    }
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(contractor_bad_debt(6000, 1000)));
    let FederalIncome::NonbusinessBadDebt {
        data:
            crate::NonbusinessBadDebt {
                source: BadDebtSource::ContractorDeposit { recovered, .. }, ..
            },
    } = &mut d.income[1].income
    else {
        panic!()
    };
    *recovered = 600_001;
    assert!(!d.valid_stored());
}

#[test]
fn state_refund_prior_and_current_year_parts_reconcile_to_net_schedule_a_once() {
    let mut before = complete(FederalStatus::Single);
    itemized(before.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
        paid: 2_000_000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    let baseline = supported(&before);
    let d = current_year_refund_draft(2_000_000, 1_990_000);
    let history = match &d.income[1].income {
        FederalIncome::GovernmentPayment { data: GovernmentPaymentSource { history, .. } } => {
            history
        }
        _ => unreachable!(),
    };
    assert_eq!(
        history.prior_current_breakdown(40_000).unwrap(),
        Some((vec![(2025, 30_000, 30_000)], 10_000))
    );
    let result = supported(&d);
    assert_eq!(value(&result, "state_refund_2_2025_allocated"), 300);
    assert_eq!(value(&result, "state_refund_2_2025_taxable"), 300);
    assert_eq!(value(&result, "state_refund_2_2026_allocated"), 100);
    assert_eq!(value(&result, "state_refund_2_2026_net_income_tax_paid"), 19_900);
    assert_eq!(value(&result, "taxable_state_tax_refund"), 300);
    assert_eq!(return_line(&result, "8"), 30_000);
    assert_eq!(value(&result, "agi"), value(&baseline, "agi") + 300);
    assert_eq!(value(&result, "deduction"), value(&baseline, "deduction") - 100);
    assert_eq!(
        value(&result, "alternative_minimum_taxable_income"),
        value(&baseline, "alternative_minimum_taxable_income")
    );
    assert!(result.review_notes.iter().any(|note| {
        note.contains("$300.00 to 2025")
            && note.contains("$100.00 to 2026")
            && note.contains("net state-income-tax payment")
    }));
    assert_eq!(estimate(&restart(d.clone()).draft), result);
}

#[test]
fn state_refund_current_share_is_source_only_with_standard_or_sales_tax() {
    for route in 0..3 {
        let mut d = complete(FederalStatus::Single);
        match route {
            0 => {} // Automatic, with no itemized expenses.
            1 => d.deductions.as_mut().unwrap().itemize_override = None,
            2 => {
                itemized(d.deductions.as_mut().unwrap()).personal_tax =
                    PersonalTaxElection::Sales { paid: 2_000_000 };
            }
            _ => unreachable!(),
        }
        let baseline = supported(&d);
        assert_eq!(d.deductions.as_ref().unwrap().income_tax_paid(), 0);
        add(&mut d, income(refund_income(40_000, 0, prior_and_current_refund_history(2_000_000))));
        let result = supported(&d);
        assert_eq!(value(&result, "state_refund_2_2025_allocated"), 300);
        assert_eq!(value(&result, "state_refund_2_2025_taxable"), 300);
        assert_eq!(value(&result, "state_refund_2_2026_allocated"), 100);
        assert_eq!(value(&result, "state_refund_2_2026_net_income_tax_paid"), 19_900);
        assert_eq!(value(&result, "taxable_state_tax_refund"), 300);
        assert_eq!(return_line(&result, "8"), 30_000);
        assert_eq!(value(&result, "deduction"), value(&baseline, "deduction"));
        assert_eq!(value(&result, "agi"), value(&baseline, "agi") + 300);
        assert_eq!(
            value(&result, "alternative_minimum_taxable_income"),
            value(&baseline, "alternative_minimum_taxable_income")
        );
        assert!(result.review_notes.iter().any(|note| {
            note.contains("$300.00 to 2025")
                && note.contains("$100.00 to 2026")
                && if route == 2 {
                    note.contains("net 2026 state-income-tax payment is not deducted")
                        && note.contains("general sales tax is elected")
                } else {
                    // 2026-11-02: this refund's automatic seed resolves to Standard.
                    note.contains("net 2026 state-income-tax payment is not deducted")
                        && note.contains("standard deduction is selected")
                }
        }));
        assert_eq!(estimate(&restart(d.clone()).draft), result);
    }
}

#[test]
fn nondeducted_return_year_state_refund_share_preserves_cents_and_prior_worksheets() {
    for sales_tax in [false, true] {
        let mut d = complete(FederalStatus::Single);
        if sales_tax {
            itemized(d.deductions.as_mut().unwrap()).personal_tax =
                PersonalTaxElection::Sales { paid: 2_000_000 };
        }
        let baseline = supported(&d);
        let mut history = multiple_prior_and_current_refund_history(2);
        let TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
            prior_years,
            current_payment_for_refunded_return,
            gross_current_income_tax_paid,
            ..
        } = &mut history
        else {
            unreachable!()
        };
        prior_years[0].payment_for_refunded_return = 200;
        prior_years[1].payment_for_refunded_return = 300;
        *current_payment_for_refunded_return = 500;
        *gross_current_income_tax_paid = 1_000;
        add(&mut d, income(refund_income(101, 0, history)));
        let result = supported(&d);
        for (key, amount) in [
            ("state_refund_2_2024_allocated", 20),
            ("state_refund_2_2024_taxable", 0),
            ("state_refund_2_2025_allocated", 30),
            ("state_refund_2_2025_taxable", 30),
            ("state_refund_2_2026_allocated", 51),
            ("state_refund_2_2026_net_income_tax_paid", 949),
        ] {
            assert_eq!(
                result.lines.iter().find(|line| line.id == key).unwrap().amount,
                amount,
                "{key}"
            );
        }
        assert_eq!(value(&result, "deduction"), value(&baseline, "deduction"));
        assert_eq!(estimate(&restart(d.clone()).draft), result);
    }
}

#[test]
fn nondeducted_return_year_state_refund_share_rejects_unresolved_sources() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(refund_income(40_000, 0, prior_and_current_refund_history(2_000_000))));
    let mut incomplete = d.clone();
    let FederalIncome::GovernmentPayment {
        data:
            GovernmentPaymentSource {
                history: TaxRefundHistory::IncomeTaxPriorYearsAndCurrent { allocation_complete, .. },
                ..
            },
    } = &mut incomplete.income[1].income
    else {
        unreachable!()
    };
    *allocation_complete = false;
    refund_left_out(&incomplete);

    let mut duplicate = d.clone();
    let mut second = prior_and_current_refund_history(2_000_000);
    let TaxRefundHistory::IncomeTaxPriorYearsAndCurrent { prior_years, .. } = &mut second else {
        unreachable!()
    };
    prior_years[0].payment_year = 2024;
    add(&mut duplicate, income(refund_income(40_000, 0, second)));
    unsupported(&duplicate, "Income:");

    elect_itemized(&mut d);
    itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::None;
    unsupported(&d, "Income:");
    // A spouse who itemizes requires the same reconciled itemizing source.
    d.setup = complete(FederalStatus::Separate).setup;
    answer_separate_pages(&mut d);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = None;
    ded.spouse_itemizes = Some(true);
    unsupported(&d, "Income:");
}

#[test]
fn state_refund_current_share_respects_existing_salt_cap() {
    let mut before = complete(FederalStatus::Single);
    itemized(before.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
        paid: 5_000_000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    let baseline = supported(&before);
    let d = current_year_refund_draft(5_000_000, 4_990_000);
    let result = supported(&d);
    assert_eq!(value(&result, "taxable_state_tax_refund"), 300);
    assert_eq!(value(&result, "deduction"), value(&baseline, "deduction"));
    assert_eq!(estimate(&restart(d.clone()).draft), result);
}

#[test]
fn state_refund_prior_and_current_year_parts_require_bound_unduplicated_sources() {
    let d = current_year_refund_draft(2_000_000, 1_990_000);
    for case in 0..4 {
        let mut bad = d.clone();
        match case {
            0 => {
                itemized(bad.deductions.as_mut().unwrap()).personal_tax =
                    PersonalTaxElection::Income {
                        paid: 2_000_000,
                        allocation: TaxIncomeAllocation::CurrentReturn,
                    }
            }
            1 => {
                elect_itemized(&mut bad);
                itemized(bad.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::None
            }
            2 => {
                let FederalIncome::GovernmentPayment {
                    data:
                        GovernmentPaymentSource {
                            history:
                                TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
                                    allocation_complete,
                                    ..
                                },
                            ..
                        },
                } = &mut bad.income[1].income
                else {
                    unreachable!()
                };
                *allocation_complete = false;
            }
            3 => {
                let FederalIncome::GovernmentPayment {
                    data:
                        GovernmentPaymentSource {
                            form: Form1099GPaper { state_or_local_income_tax_refunds: amount, .. },
                            ..
                        },
                } = &mut bad.income[1].income
                else {
                    unreachable!()
                };
                *amount = 400_001;
            }
            _ => unreachable!(),
        }
        refund_not_taxed_silently(&bad);
    }
    let mut duplicate = d.clone();
    let mut second = prior_and_current_refund_history(2_000_000);
    let TaxRefundHistory::IncomeTaxPriorYearsAndCurrent { prior_years, .. } = &mut second else {
        unreachable!()
    };
    prior_years[0].payment_year = 2024;
    add(&mut duplicate, income(refund_income(40_000, 0, second)));
    unsupported(&duplicate, "Income:");

    let mut invalid = d.clone();
    let FederalIncome::GovernmentPayment {
        data:
            GovernmentPaymentSource {
                history:
                    TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
                        current_payment_for_refunded_return,
                        gross_current_income_tax_paid,
                        ..
                    },
                ..
            },
    } = &mut invalid.income[1].income
    else {
        unreachable!()
    };
    *current_payment_for_refunded_return = *gross_current_income_tax_paid + 1;
    assert!(!invalid.valid_stored());
}

#[test]
fn state_refund_prior_and_current_year_allocation_preserves_cents() {
    let mut history = prior_and_current_refund_history(10_000);
    let TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
        prior_years,
        current_payment_for_refunded_return,
        ..
    } = &mut history
    else {
        unreachable!()
    };
    prior_years[0].payment_for_refunded_return = 200;
    *current_payment_for_refunded_return = 100;
    assert_eq!(history.prior_current_breakdown(100).unwrap(), Some((vec![(2025, 67, 67)], 33)));
}

#[test]
fn two_through_seven_prior_years_and_2026_share_reconcile_each_worksheet_once() {
    let mut before = complete(FederalStatus::Single);
    itemized(before.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
        paid: 2_000_000,
        allocation: TaxIncomeAllocation::CurrentReturn,
    };
    let baseline = supported(&before);
    for count in 2..=7 {
        let d = multiple_prior_and_current_refund_draft(count);
        let FederalIncome::GovernmentPayment {
            data:
                GovernmentPaymentSource {
                    history,
                    form: Form1099GPaper { state_or_local_income_tax_refunds: amount, .. },
                    ..
                },
        } = &d.income[1].income
        else {
            unreachable!()
        };
        let (years, current_share) = history.prior_current_breakdown(*amount).unwrap().unwrap();
        assert_eq!(years.len(), count);
        assert_eq!(current_share, 10_000);
        assert_eq!(years.iter().map(|(_, share, _)| share).sum::<i64>() + current_share, *amount);
        let taxable = (count as i64 - 2) * 10_000 + 5_000;
        assert_eq!(history.taxable(*amount).unwrap(), taxable);
        let result = supported(&d);
        for &(year, share, year_taxable) in &years {
            assert_eq!(share, 10_000);
            assert_eq!(
                year_taxable,
                if year == 2024 {
                    0
                } else if year == 2025 {
                    5_000
                } else {
                    10_000
                }
            );
            for (suffix, expected) in [("allocated", share), ("taxable", year_taxable)] {
                let key = alloc::format!("state_refund_2_{}_{}", year, suffix);
                assert_eq!(
                    result.lines.iter().find(|line| line.id == key).unwrap().amount,
                    expected,
                    "count={count}, {key}"
                );
            }
            assert!(
                result
                    .review_notes
                    .iter()
                    .any(|note| note.contains(&alloc::format!("$100.00 to {}", year)))
            );
        }
        assert_eq!(value(&result, "state_refund_2_2026_allocated"), 100);
        assert_eq!(value(&result, "state_refund_2_2026_net_income_tax_paid"), 19_900);
        assert_eq!(value(&result, "taxable_state_tax_refund"), taxable / 100);
        let summary = &result
            .review_notes
            .iter()
            .find(|note| note.contains("net state-income-tax payment") && note.contains("2026"))
            .unwrap()
            .display;
        assert_eq!(summary.key, "NOTE_STATE_REFUND_CURRENT_INCOME_TAX");
        assert_eq!(
            summary.args,
            vec![
                MessageArg::Number(2),
                MessageArg::usd_cents(10_000),
                MessageArg::usd_cents(taxable)
            ]
        );
        let allocations: Vec<_> = result
            .review_notes
            .iter()
            .map(|note| &note.display)
            .filter(|message| message.key == "NOTE_STATE_REFUND_PRIOR_ALLOCATION")
            .collect();
        assert_eq!(allocations.len(), count);
        assert_eq!(allocations[0].args[1], MessageArg::usd_cents(10_000));
        assert_eq!(return_line(&result, "8"), taxable);
        assert_eq!(value(&result, "deduction"), value(&baseline, "deduction") - 100);
        assert_eq!(
            value(&result, "alternative_minimum_taxable_income"),
            value(&baseline, "alternative_minimum_taxable_income")
        );
        assert!(result.review_notes.iter().any(|note| note.contains("$100.00 to 2026")
            && note.contains("net state-income-tax payment")));
        assert_eq!(estimate(&restart(d.clone()).draft), result);
    }
}

#[test]
fn multiple_prior_years_and_current_refund_preserves_each_cent() {
    let mut history = multiple_prior_and_current_refund_history(2);
    let TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
        prior_years,
        current_payment_for_refunded_return,
        ..
    } = &mut history
    else {
        unreachable!()
    };
    prior_years[0].payment_for_refunded_return = 200;
    prior_years[1].payment_for_refunded_return = 300;
    *current_payment_for_refunded_return = 500;
    assert_eq!(
        history.prior_current_breakdown(101).unwrap(),
        Some((vec![(2024, 20, 0), (2025, 30, 30)], 51))
    );
}

#[test]
fn multiple_prior_years_and_current_refund_rejects_unreconciled_sources() {
    let d = multiple_prior_and_current_refund_draft(3);
    for case in 0..5 {
        let mut bad = d.clone();
        match case {
            0 => {
                let FederalIncome::GovernmentPayment {
                    data:
                        GovernmentPaymentSource {
                            history:
                                TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
                                    allocation_complete,
                                    ..
                                },
                            ..
                        },
                } = &mut bad.income[1].income
                else {
                    unreachable!()
                };
                *allocation_complete = false;
            }
            1 => {
                let FederalIncome::GovernmentPayment {
                    data:
                        GovernmentPaymentSource {
                            form: Form1099GPaper { state_or_local_income_tax_refunds: amount, .. },
                            ..
                        },
                } = &mut bad.income[1].income
                else {
                    unreachable!()
                };
                *amount = 400_001;
            }
            2 => {
                let FederalIncome::GovernmentPayment {
                    data:
                        GovernmentPaymentSource {
                            history:
                                TaxRefundHistory::IncomeTaxPriorYearsAndCurrent { prior_years, .. },
                            ..
                        },
                } = &mut bad.income[1].income
                else {
                    unreachable!()
                };
                prior_years[0].ordinary = false;
            }
            3 => {
                itemized(bad.deductions.as_mut().unwrap()).personal_tax =
                    PersonalTaxElection::Income {
                        paid: 2_000_000,
                        allocation: TaxIncomeAllocation::CurrentReturn,
                    }
            }
            4 => {
                elect_itemized(&mut bad);
                itemized(bad.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::None
            }
            _ => unreachable!(),
        }
        refund_not_taxed_silently(&bad);
        if case == 0 {
            assert!(noted(&estimate(&bad), "WARN_TAX_REFUND_LEFT_OUT"));
        }
    }
    let mut duplicate = d.clone();
    add(&mut duplicate, income(refund_income(100, 0, refund_history())));
    unsupported(&duplicate, "Income:");

    for case in [0, 3, 4] {
        let mut bad = d.clone();
        let FederalIncome::GovernmentPayment {
            data:
                GovernmentPaymentSource {
                    history:
                        TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
                            prior_years,
                            current_payment_for_refunded_return,
                            gross_current_income_tax_paid,
                            ..
                        },
                    ..
                },
        } = &mut bad.income[1].income
        else {
            unreachable!()
        };
        match case {
            0 => prior_years[1].payment_year = prior_years[0].payment_year,
            3 => prior_years[0].payment_for_refunded_return = prior_years[0].income_paid + 1,
            4 => *current_payment_for_refunded_return = *gross_current_income_tax_paid + 1,
            _ => unreachable!(),
        }
        assert!(!bad.valid_stored(), "case {case}");
    }
    let mut too_many = multiple_prior_and_current_refund_draft(7);
    let FederalIncome::GovernmentPayment {
        data:
            GovernmentPaymentSource {
                history: TaxRefundHistory::IncomeTaxPriorYearsAndCurrent { prior_years, .. },
                ..
            },
    } = &mut too_many.income[1].income
    else {
        unreachable!()
    };
    let mut earliest = prior_years[0].clone();
    earliest.payment_year = 2018;
    earliest.standard = 1_200_000;
    prior_years.insert(0, earliest);
    assert!(!too_many.valid_stored());
}

#[test]
fn eight_year_refund_reuses_historical_schedule_a_and_pease_worksheets() {
    let years = vec![
        historical_refund_payment(2010),
        historical_refund_payment(2011),
        historical_refund_payment(2012),
        historical_refund_payment(2013),
        historical_pease_payment(2017, true),
        historical_refund_payment(2018),
        historical_refund_payment(2024),
        historical_refund_payment(2025),
    ];
    let history =
        TaxRefundHistory::IncomeTaxMultiplePriorYears { years, allocation_complete: true };
    assert_eq!(
        history.multi_year_breakdown(80_000).unwrap(),
        Some(vec![
            (2010, 10_000, 10_000),
            (2011, 10_000, 10_000),
            (2012, 10_000, 10_000),
            (2013, 10_000, 10_000),
            (2017, 10_000, 2_000),
            (2018, 10_000, 10_000),
            (2024, 10_000, 0),
            (2025, 10_000, 5_000),
        ])
    );
    assert_eq!(history.taxable(80_000).unwrap(), 57_000);
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, income(refund_income(80_000, 0, history)));
    let result = supported(&d);
    for (year, taxable) in [
        (2010, 10_000),
        (2011, 10_000),
        (2012, 10_000),
        (2013, 10_000),
        (2017, 2_000),
        (2018, 10_000),
        (2024, 0),
        (2025, 5_000),
    ] {
        for (suffix, expected) in [("allocated", 10_000), ("taxable", taxable)] {
            let key = alloc::format!("state_refund_2_{}_{}", year, suffix);
            assert_eq!(result.lines.iter().find(|line| line.id == key).unwrap().amount, expected);
        }
        assert!(
            result
                .review_notes
                .iter()
                .any(|note| note.contains(&alloc::format!("$100.00 to {}", year)))
        );
    }
    assert_eq!(return_line(&result, "8"), 57_000);
    assert_eq!(value(&result, "agi"), value(&before, "agi") + 570);
    assert_eq!(
        value(&result, "alternative_minimum_taxable_income"),
        value(&before, "alternative_minimum_taxable_income")
    );
    assert_eq!(estimate(&restart(d.clone()).draft), result);
}

#[test]
fn historical_prior_years_and_2026_share_keep_current_salt_net_once() {
    let years = vec![
        historical_refund_payment(2010),
        historical_refund_payment(2012),
        historical_refund_payment(2013),
        historical_pease_payment(2017, true),
        historical_refund_payment(2018),
        historical_refund_payment(2024),
        historical_refund_payment(2025),
    ];
    for gross_current_paid in [2_000_000, 5_000_000] {
        let mut before = complete(FederalStatus::Single);
        itemized(before.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
            paid: gross_current_paid,
            allocation: TaxIncomeAllocation::CurrentReturn,
        };
        let baseline = supported(&before);
        let history = TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
            prior_years: years.clone(),
            current_payment_for_refunded_return: 100_000,
            gross_current_income_tax_paid: gross_current_paid,
            allocation_complete: true,
        };
        let (prior, current) = history.prior_current_breakdown(80_000).unwrap().unwrap();
        assert_eq!(prior.iter().map(|(_, share, _)| share).sum::<i64>() + current, 80_000);
        assert_eq!(prior.iter().map(|(_, _, taxable)| taxable).sum::<i64>(), 47_000);
        assert_eq!(current, 10_000);
        let mut d = complete(FederalStatus::Single);
        itemized(d.deductions.as_mut().unwrap()).personal_tax = PersonalTaxElection::Income {
            paid: gross_current_paid - current,
            allocation: TaxIncomeAllocation::CurrentReturn,
        };
        add(&mut d, income(refund_income(80_000, 0, history)));
        let result = supported(&d);
        assert_eq!(return_line(&result, "8"), 47_000);
        assert_eq!(value(&result, "state_refund_2_2026_allocated"), 100);
        assert_eq!(
            value(&result, "state_refund_2_2026_net_income_tax_paid"),
            (gross_current_paid - current) / 100
        );
        for &(year, share, taxable) in &prior {
            for (suffix, expected) in [("allocated", share), ("taxable", taxable)] {
                let key = alloc::format!("state_refund_2_{}_{}", year, suffix);
                assert_eq!(
                    result.lines.iter().find(|line| line.id == key).unwrap().amount,
                    expected
                );
            }
        }
        assert_eq!(
            value(&result, "deduction"),
            value(&baseline, "deduction") - if gross_current_paid == 2_000_000 { 100 } else { 0 }
        );
        assert_eq!(
            value(&result, "alternative_minimum_taxable_income"),
            value(&baseline, "alternative_minimum_taxable_income")
        );
        assert_eq!(estimate(&restart(d.clone()).draft), result);
    }
}

#[test]
fn historical_current_refund_cents_and_bad_retained_years_are_guarded() {
    let mut history = TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
        prior_years: vec![historical_refund_payment(2010), historical_pease_payment(2017, true)],
        current_payment_for_refunded_return: 500,
        gross_current_income_tax_paid: 2_000_000,
        allocation_complete: true,
    };
    let TaxRefundHistory::IncomeTaxPriorYearsAndCurrent { prior_years, .. } = &mut history else {
        unreachable!()
    };
    prior_years[0].payment_for_refunded_return = 200;
    prior_years[1].payment_for_refunded_return = 300;
    assert_eq!(
        history.prior_current_breakdown(101).unwrap(),
        Some((vec![(2010, 20, 20), (2017, 30, 6)], 51))
    );

    for case in [2, 3, 4, 5] {
        let mut years = vec![
            historical_refund_payment(2011),
            historical_pease_payment(2017, true),
            historical_refund_payment(2018),
        ];
        match case {
            2 => years[0].salt_deducted -= 1,
            3 => years[0].overall_limit = years[1].overall_limit.clone(),
            4 => years[2].overall_limit = years[1].overall_limit.clone(),
            5 => years[1].overall_limit.as_mut().unwrap().before_limit -= 100,
            _ => unreachable!(),
        }
        let history =
            TaxRefundHistory::IncomeTaxMultiplePriorYears { years, allocation_complete: true };
        assert!(!history.valid(), "case {case}");
    }
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(refund_income(
            20_000,
            0,
            TaxRefundHistory::IncomeTaxMultiplePriorYears {
                years: vec![historical_refund_payment(2010), historical_refund_payment(2018)],
                allocation_complete: false,
            },
        )),
    );
    refund_left_out(&d);
    let mut duplicate = complete(FederalStatus::Single);
    add(
        &mut duplicate,
        income(refund_income(
            20_000,
            0,
            TaxRefundHistory::IncomeTaxMultiplePriorYears {
                years: vec![historical_refund_payment(2010), historical_refund_payment(2018)],
                allocation_complete: true,
            },
        )),
    );
    add(
        &mut duplicate,
        income(refund_income(
            100,
            0,
            TaxRefundHistory::IncomeTax {
                deduction_year: 2010,
                income_paid: 1_000_000,
                sales_available: 0,
                salt_paid: 1_000_000,
                salt_deducted: 1_000_000,
                itemized: 2_000_000,
                standard: 570_000,
                overall_limit: None,
                property_refunded: 0,
                ordinary: true,
            },
        )),
    );
    unsupported(&duplicate, "Income:");
}

#[test]
fn two_year_refund_allocation_preserves_every_cent() {
    let mut history = two_year_refund_history();
    let TaxRefundHistory::IncomeTaxMultiplePriorYears { years, .. } = &mut history else {
        unreachable!()
    };
    years[0].payment_for_refunded_return = 200;
    years[1].payment_for_refunded_return = 100;
    assert_eq!(
        history.multi_year_breakdown(100).unwrap(),
        Some(vec![(2024, 67, 67), (2025, 33, 33)])
    );
    assert_eq!(history.taxable(100).unwrap(), 100);
}

#[test]
fn two_year_refund_requires_a_complete_nonduplicated_payment_history() {
    use super::support::assert::unit50_retirement::assert_multi_year_refund_guards;

    for (history, amount, three_years) in
        [(two_year_refund_history(), 400_000, false), (three_year_refund_history(), 100_000, true)]
    {
        assert_multi_year_refund_guards(history, amount, three_years);
    }
}

#[test]
fn one_state_refund_from_three_prior_payment_years_reconciles_each_year_and_return() {
    let history = three_year_refund_history();
    assert_eq!(
        history.multi_year_breakdown(100_000).unwrap(),
        Some(vec![(2023, 50_000, 50_000), (2024, 30_000, 0), (2025, 20_000, 10_000),])
    );
    assert_eq!(history.taxable(100_000).unwrap(), 60_000);

    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, income(refund_income(100_000, 0, history)));
    let result = supported(&d);
    assert_eq!(value(&result, "state_refund_2_2023_allocated"), 500);
    assert_eq!(value(&result, "state_refund_2_2023_taxable"), 500);
    assert_eq!(value(&result, "state_refund_2_2024_allocated"), 300);
    assert_eq!(value(&result, "state_refund_2_2024_taxable"), 0);
    assert_eq!(value(&result, "state_refund_2_2025_allocated"), 200);
    assert_eq!(value(&result, "state_refund_2_2025_taxable"), 100);
    assert_eq!(value(&result, "taxable_state_tax_refund"), 600);
    assert_eq!(return_line(&result, "8"), 60_000);
    assert_eq!(value(&result, "agi"), value(&before, "agi") + 600);
    assert_eq!(
        value(&result, "alternative_minimum_taxable_income"),
        value(&before, "alternative_minimum_taxable_income")
    );
    assert!(result.review_notes.iter().any(|note| {
        note.contains("$500.00 to 2023")
            && note.contains("$300.00 to 2024")
            && note.contains("$200.00 to 2025")
            && note.contains("$600.00 on Schedule 1")
    }));
    assert_eq!(estimate(&restart(d.clone()).draft), result);
}

#[test]
fn three_year_refund_allocation_preserves_all_cents_and_breaks_ties_by_year() {
    let mut history = three_year_refund_history();
    let TaxRefundHistory::IncomeTaxMultiplePriorYears { years, .. } = &mut history else {
        unreachable!()
    };
    for (year, weight) in years.iter_mut().zip([200, 300, 500]) {
        year.payment_for_refunded_return = weight;
    }
    let shares = history.multi_year_breakdown(101).unwrap().unwrap();
    assert_eq!(
        shares.iter().map(|(_, allocated, _)| *allocated).collect::<Vec<_>>(),
        vec![20, 30, 51]
    );
    assert_eq!(shares.iter().map(|(_, allocated, _)| allocated).sum::<i64>(), 101);

    let TaxRefundHistory::IncomeTaxMultiplePriorYears { years, .. } = &mut history else {
        unreachable!()
    };
    for year in years {
        year.payment_for_refunded_return = 1;
    }
    let shares = history.multi_year_breakdown(2).unwrap().unwrap();
    assert_eq!(
        shares.iter().map(|(_, allocated, _)| *allocated).collect::<Vec<_>>(),
        vec![1, 1, 0]
    );
}

#[test]
fn four_through_eight_prior_year_refunds_keep_every_cent_and_visible_share() {
    let single_standard =
        [1_200_000, 1_220_000, 1_240_000, 1_255_000, 1_295_000, 1_385_000, 1_460_000, 1_575_000];
    for count in 4..=8 {
        let years = (0..count)
            .map(|index| TaxRefundYearPayment {
                payment_year: 2018 + index as i32,
                payment_for_refunded_return: 10_000,
                income_paid: 10_000,
                sales_available: 0,
                salt_paid: 10_000,
                salt_deducted: 10_000,
                itemized: 2_000_000,
                standard: single_standard[index],
                overall_limit: None,
                ordinary: true,
            })
            .collect();
        let history =
            TaxRefundHistory::IncomeTaxMultiplePriorYears { years, allocation_complete: true };
        let refund = count as i64 * 100 + 1;
        let shares = history.multi_year_breakdown(refund).unwrap().unwrap();
        assert_eq!(shares.len(), count);
        assert_eq!(shares.iter().map(|(_, amount, _)| amount).sum::<i64>(), refund);
        for (index, &(year, allocated, taxable)) in shares.iter().enumerate() {
            assert_eq!(year, 2018 + index as i32);
            assert_eq!(allocated, if index == 0 { 101 } else { 100 });
            assert_eq!(taxable, allocated);
        }
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(refund_income(refund, 0, history)));
        let result = supported(&d);
        assert_eq!(return_line(&result, "8"), ((refund + 50) / 100) * 100);
        assert_eq!(
            result.lines.iter().find(|line| line.id == "taxable_state_tax_refund").unwrap().amount,
            ((refund + 50) / 100) * 100
        );
        for &(year, allocated, taxable) in &shares {
            for (suffix, expected) in [("allocated", allocated), ("taxable", taxable)] {
                let key = alloc::format!("state_refund_2_{}_{}", year, suffix);
                let amount = result.lines.iter().find(|line| line.id == key).unwrap().amount;
                assert_eq!(amount, expected, "count={count}, {key}");
            }
            assert!(result.review_notes.iter().any(|note| {
                note.contains(&alloc::format!(
                    "${}.{:02} to {}",
                    allocated / 100,
                    allocated % 100,
                    year
                ))
            }));
        }
        assert_eq!(estimate(&restart(d.clone()).draft), result);
    }
}

#[test]
fn eight_year_refund_rejects_ninth_entry_and_duplicate_deduction_document() {
    let mut history = TaxRefundHistory::IncomeTaxMultiplePriorYears {
        years: (2018..=2025)
            .map(|year| TaxRefundYearPayment {
                payment_year: year,
                payment_for_refunded_return: 10_000,
                income_paid: 10_000,
                sales_available: 0,
                salt_paid: 10_000,
                salt_deducted: 10_000,
                itemized: 2_000_000,
                standard: 1_000_000,
                overall_limit: None,
                ordinary: true,
            })
            .collect(),
        allocation_complete: true,
    };
    assert!(history.valid());
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(refund_income(800, 0, history.clone())));
    add(
        &mut d,
        income(refund_income(
            100,
            0,
            TaxRefundHistory::IncomeTax {
                deduction_year: 2019,
                income_paid: 10_000,
                sales_available: 0,
                salt_paid: 10_000,
                salt_deducted: 10_000,
                itemized: 2_000_000,
                standard: 1_000_000,
                overall_limit: None,
                property_refunded: 0,
                ordinary: true,
            },
        )),
    );
    unsupported(&d, "Income:");
    let TaxRefundHistory::IncomeTaxMultiplePriorYears { years, .. } = &mut history else {
        unreachable!()
    };
    years.push(years[7].clone());
    assert!(!history.valid());
}

#[test]
fn state_refund_is_limited_to_prior_itemization_benefit_and_removed_from_amt() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, income(refund_income(300000, 0, refund_history())));
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_state_tax_refund"), 1000);
    assert_eq!(value(&r, "agi"), 61000);
    assert_eq!(return_line(&r, "8"), 100000);
    assert_eq!(value(&r, "income_tax"), 5143);
    assert_eq!(
        value(&r, "alternative_minimum_taxable_income"),
        value(&before, "alternative_minimum_taxable_income")
    );
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn state_refund_without_prior_deduction_or_with_unused_salt_cap_is_not_income() {
    for history in [
        TaxRefundHistory::NotDeducted,
        TaxRefundHistory::SalesTax,
        TaxRefundHistory::IncomeTax {
            deduction_year: 2025,
            income_paid: 2000000,
            sales_available: 500000,
            salt_paid: 6000000,
            salt_deducted: 4000000,
            itemized: 4500000,
            standard: 1575000,
            overall_limit: None,
            property_refunded: 0,
            ordinary: true,
        },
    ] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(refund_income(500000, 0, history)));
        let r = supported(&d);
        assert_eq!(value(&r, "taxable_state_tax_refund"), 0);
        assert_eq!(value(&r, "agi"), 60000);
    }
}
#[test]
fn state_refund_special_recoveries_and_duplicate_deduction_benefits_are_guarded() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(refund_income(600000, 0, refund_history())));
    assert_eq!(value(&supported(&d), "taxable_state_tax_refund"), 1000);
    let FederalIncome::GovernmentPayment {
        data:
            GovernmentPaymentSource {
                history,
                form: Form1099GPaper { state_or_local_income_tax_refunds: amount, .. },
                ..
            },
    } = &mut d.income[1].income
    else {
        panic!()
    };
    *amount = 100000;
    let TaxRefundHistory::IncomeTax { ordinary, .. } = history else { panic!() };
    *ordinary = false;
    refund_left_out(&d);
    d.income[1].income = refund_income(100000, 0, refund_history());
    add(&mut d, income(refund_income(100000, 0, refund_history())));
    unsupported(&d, "Income:");
    let mut excessive = complete(FederalStatus::Single);
    add(&mut excessive, income(refund_income(1_000_001, 0, refund_history())));
    unsupported(&excessive, "Income:");
    if let FederalIncome::GovernmentPayment {
        data:
            GovernmentPaymentSource {
                form: Form1099GPaper { state_or_local_income_tax_refunds: amount, .. },
                ..
            },
    } = &mut excessive.income[1].income
    {
        *amount = 1_000_000;
    }
    supported(&excessive);
}

#[test]
fn historical_state_refunds_preserve_schedule_a_and_pease_branches() {
    for (year, amount, itemized, standard, pease_agi, expected, agi) in [
        (2017, 300_000, 2_000_000, 635_000, None, 3_000, Some(63_000)),
        (2017, 300_000, 1_200_000, 635_000, Some(60_000_000), 600, Some(60_600)),
        (2017, 30_000, 1_209_500, 635_000, Some(52_500_000), 155, None),
        (2010, 300_000, 2_000_000, 600_000, None, 3_000, None),
        (2011, 300_000, 2_000_000, 600_000, None, 3_000, None),
        (2012, 300_000, 2_000_000, 600_000, None, 3_000, None),
    ] {
        let overall_limit = pease_agi.map(|agi| PeaseWorksheet {
            agi,
            before_limit: 2_000_000,
            protected: 1_000_000,
            filing_status: PeaseFilingStatus::Single,
        });
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(refund_income(
                amount,
                0,
                historical_refund_history(year, itemized, standard, overall_limit),
            )),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "taxable_state_tax_refund"), expected, "{year} {pease_agi:?}");
        if let Some(agi) = agi {
            assert_eq!(value(&r, "agi"), agi);
        }
        if year == 2017 && pease_agi.is_none() {
            add(&mut d, income(refund_income(300_000, 0, refund_history())));
            let mixed = supported(&d);
            assert_eq!(value(&mixed, "taxable_state_tax_refund"), 4_000);
            assert_eq!(estimate(&restart(d.clone()).draft), mixed);
            let FederalIncome::GovernmentPayment {
                data:
                    GovernmentPaymentSource {
                        history: TaxRefundHistory::IncomeTax { ordinary, .. },
                        ..
                    },
            } = &mut d.income[1].income
            else {
                unreachable!()
            };
            *ordinary = false;
            refund_left_out(&d);
        } else {
            assert_eq!(estimate(&restart(d.clone()).draft), r, "{year} {pease_agi:?}");
            let FederalIncome::GovernmentPayment {
                data:
                    GovernmentPaymentSource {
                        history: TaxRefundHistory::IncomeTax { overall_limit, .. },
                        ..
                    },
            } = &mut d.income[1].income
            else {
                unreachable!()
            };
            if pease_agi == Some(60_000_000) {
                overall_limit.as_mut().unwrap().before_limit = 1_900_000;
                assert!(!d.valid_stored());
            } else if year <= 2012 {
                *overall_limit = Some(PeaseWorksheet {
                    agi: 60_000_000,
                    before_limit: 2_000_000,
                    protected: 1_000_000,
                    filing_status: PeaseFilingStatus::Single,
                });
                assert!(!d.valid_stored(), "{year}");
            }
        }
    }
}
#[test]
fn social_security_small_net_repayments_do_not_reduce_current_income() {
    for amount in [-1, -299999, -300000] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(social_security_net(amount, 0)));
        let r = supported(&d);
        assert_eq!(return_line(&r, "6a"), 0);
        assert_eq!(return_line(&r, "6b"), 0);
        assert_eq!(value(&r, "agi"), 60000);
        assert_eq!(r.tax, supported(&complete(FederalStatus::Single)).tax);
    }
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(social_security_net(-300001, 0)));
    unsupported(&d, "Social Security repayments");
    assert!(d.valid_stored());
}
#[test]
fn social_security_repayment_threshold_uses_return_total_before_rounding() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(social_security_net(-200000, 0)));
    let mut spouse = income(social_security_net(-100001, 0));
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    unsupported(&d, "Social Security repayments");
    d.income[2].income = social_security_net(-100000, 0);
    assert_eq!(value(&supported(&d), "agi"), 100000);
    d.income[2].income = social_security_net(200000, 0);
    assert_eq!(return_line(&supported(&d), "6a"), 0);
}

#[test]
fn pre_1987_general_rule_continues_after_recovering_original_cost() {
    for kind in [RetirementKind::Pension, RetirementKind::Nonqualified] {
        let mut d = complete(FederalStatus::Single);
        d.people.as_mut().unwrap().taxpayer.birth_date = 19210101;
        add(&mut d, old_general_entry(kind, old_general_annuity(19860702, 2000000)));
        assert!(d.valid_stored());
        let r = supported(&d);
        let id = d.income.last().unwrap().id;
        assert_eq!(value(&r, &alloc::format!("annuity_{id}_basis_recovery")), 504);
        assert_eq!(return_line(&r, "5b"), 1149600);
        assert!(!r.lines.iter().any(|l| l.id == alloc::format!("annuity_{id}_remaining_basis")));
        assert_eq!(restart(d.clone()).draft, d);
    }
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        old_general_entry(RetirementKind::Nonqualified, old_general_annuity(19870101, 1000000)),
    );
    assert_eq!(return_line(&supported(&d), "5b"), 1200000);
}

#[test]
fn full_contract_discharge_recovers_cost_before_income_and_early_tax() {
    for (gross, cost, taxable, loss) in [
        (15000, 10000, 5000, 0),
        (10000, 10000, 0, 0),
        (8000, 10000, 0, 2000),
        (0, 10000, 0, 10000),
    ] {
        for kind in [RetirementKind::Pension, RetirementKind::Nonqualified] {
            let d = contract_discharge(kind, gross, cost);
            let r = supported(&d);
            assert_eq!(return_line(&r, "5a"), gross * 100);
            assert_eq!(return_line(&r, "5b"), taxable * 100);
            assert_eq!(value(&r, "early_distribution_tax"), taxable / 10);
            assert_eq!(value(&r, "annuity_2_basis_recovery"), cost.min(gross));
            assert_eq!(value(&r, "annuity_2_surrender_loss"), loss);
            assert_eq!(
                value(&r, "net_investment_income_tax"),
                if kind == RetirementKind::Nonqualified { taxable * 38 / 1000 } else { 0 }
            );
            assert_eq!(restart(d.clone()).draft, d);
        }
    }
}

#[test]
fn continuing_macrs_matches_irs_declining_balance_switch_and_final_recovery() {
    // Pub. 946's $1,000 five-year example, with each recovery year mapped to 2026.
    for (placed, prior, deduction) in [
        (20250601, 20000, 32000),
        (20240601, 52000, 19200),
        (20230601, 71200, 11520),
        (20220601, 82720, 11520),
        (20210601, 94240, 5760),
    ] {
        let asset = continuing_equipment(placed, prior, false);
        assert_eq!(asset.macrs_amount(false), deduction);
        assert_eq!(asset.bonus_amount(), 0);
        let d = business_with_bonus(100000, asset);
        let r = supported(&d);
        assert_eq!(
            r.lines.iter().find(|l| l.id == "business_2_asset_0_macrs").unwrap().amount,
            deduction
        );
        assert_eq!(
            r.lines.iter().find(|l| l.id == "business_2_asset_0_remaining_basis").unwrap().amount,
            100000 - prior - deduction
        );
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn startup_prior_year_amortization_finishes_without_repeating_initial_deduction() {
    for (started, amortization, remaining) in
        [(20250701, 3600, 48600), (20110701, 1800, 0), (20110101, 0, 0), (20100101, 0, 0)]
    {
        let r = supported(&startup_business(5450000, started, false));
        assert_eq!(value(&r, "business_2_startup_initial"), 0);
        assert_eq!(value(&r, "business_2_startup_amortization"), amortization);
        assert_eq!(value(&r, "business_2_startup_remaining_basis"), remaining);
    }
    let r = supported(&startup_business(5450000, 20260701, true));
    assert_eq!(value(&r, "business_2_startup_initial"), 0);
    assert_eq!(value(&r, "business_2_startup_amortization"), 0);
    assert_eq!(value(&r, "business_2_startup_remaining_basis"), 54500);
}

#[test]
fn prior_atnol_limits_ordinary_loss_without_changing_regular_tax() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 100000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_atnol_deduction"), 54000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 6000);
    assert_eq!(value(&r, "atnol_2025_carryforward"), 46000);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(r.tax, before.tax);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_atnol_expires_old_losses_and_uses_oldest_year_first() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().atnol =
        prior_atnol(&[(2025, 40000, false), (2006, 20000, false), (2005, 3000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "atnol_2005_used"), 0);
    assert_eq!(value(&r, "atnol_2005_expired"), 3000);
    assert_eq!(value(&r, "atnol_2006_used"), 20000);
    assert_eq!(value(&r, "atnol_2025_used"), 34000);
    assert_eq!(value(&r, "amt_total_nol_carryforward"), 6000);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2006, 100000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "atnol_2006_expired"), 46000);
    assert_eq!(value(&r, "amt_total_nol_carryforward"), 0);
}

#[test]
fn prior_atnol_special_loss_can_fill_the_ten_percent_remainder() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().atnol =
        prior_atnol(&[(2025, 100000, false), (2008, 10000, true)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_atnol_deduction"), 60000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 0);
    assert_eq!(value(&r, "atnol_2008_used"), 10000);
    assert_eq!(value(&r, "atnol_2025_used"), 50000);
    assert_eq!(value(&r, "amt_total_nol_carryforward"), 50000);
}

#[test]
fn prior_atnol_zero_income_preserves_unexpired_losses() {
    let mut d = empty_return(FederalStatus::Single);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 1000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_atnol_deduction"), 0);
    assert_eq!(value(&r, "atnol_2025_carryforward"), 1000);
}

#[test]
fn prior_atnol_leaves_out_incomplete_history_and_rejects_invalid_vintages() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 1000, false)]);
    if let AtnolHistory::Records { confirmed, .. } = &mut d.screening.as_mut().unwrap().atnol {
        *confirmed = false;
    }
    // No: the AMT loss history is left out with a notice.
    assert!(noted(&supported(&d), "WARN_ATNOL_LEFT_OUT"));
    assert!(!prior_atnol(&[(2025, 1000, false), (2025, 2000, false)]).valid());
    assert!(!prior_atnol(&[(2025, 1000, true)]).valid());
}

#[test]
fn prior_atnol_does_not_consume_losses_in_a_current_loss_year() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(-80000)));
    let baseline = supported(&d);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 30000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_atnol_deduction"), 0);
    assert_eq!(value(&r, "atnol_2025_carryforward"), 30000);
    assert_eq!(
        value(&r, "amt_current_net_operating_loss"),
        value(&baseline, "amt_current_net_operating_loss")
    );
    assert_eq!(
        value(&r, "amt_total_nol_carryforward"),
        value(&baseline, "amt_nol_carryforward") + 30000
    );
}

#[test]
fn prior_atnol_older_ordinary_loss_cannot_absorb_special_only_capacity() {
    // IRS 20144201F, p. 6: $200 ordinary loss from 2008, $100 special
    // loss from 2009, $100 current AMTI => use $90 and $10, respectively.
    // Scale by 600 to use the existing $60,000 wage fixture.
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().atnol =
        prior_atnol(&[(2009, 60000, true), (2008, 120000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_atnol_deduction"), 60000);
    assert_eq!(value(&r, "atnol_2008_used"), 54000);
    assert_eq!(value(&r, "atnol_2009_used"), 6000);
    assert_eq!(value(&r, "atnol_2008_carryforward"), 66000);
    assert_eq!(value(&r, "atnol_2009_carryforward"), 54000);
    assert_eq!(value(&r, "amt_total_nol_carryforward"), 120000);
}

#[test]
fn prior_atnol_ordinary_capacity_is_shared_across_loss_years() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().atnol =
        prior_atnol(&[(2009, 10000, true), (2007, 40000, false), (2008, 40000, false)]);
    let r = supported(&d);
    assert_eq!(value(&r, "atnol_2007_used"), 40000);
    assert_eq!(value(&r, "atnol_2008_used"), 14000);
    assert_eq!(value(&r, "atnol_2009_used"), 6000);
    assert_eq!(value(&r, "amt_total_nol_carryforward"), 30000);
}

#[test]
fn prior_atnol_allows_eligible_statement_credit_in_regular_and_amt_returns() {
    for preference in [0, 200000] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Interest { data: interest_source(native_interest(100000, 0)) }),
        );
        if preference > 0 {
            d.screening.as_mut().unwrap().iso_exercises =
                vec![iso_exercise("Synthetic ISO", 1.0, 100, preference * 100 + 100)];
        }
        d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 1000, false)]);
        let before = supported(&d);
        payee_statement_credit(&mut d, 30000);
        let r = supported(&d);
        assert_eq!(value(&r, "prior_atnol_deduction"), 1000);
        assert_eq!(value(&r, "foreign_tax_credit"), 300);
        assert_eq!(value(&r, "alternative_minimum_tax"), value(&before, "alternative_minimum_tax"));
        assert_eq!(r.tax, before.tax.map(|tax| tax - 30000));
        assert_eq!(restart(d.clone()).draft, d);
        raise_payee_statement_tax(&mut d, 1);
        unsupported(&d, "Foreign tax credit");
    }
}

#[test]
fn prior_regular_nol_applies_eighty_percent_limit_and_restores_avro() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 35120);
    assert_eq!(value(&r, "agi"), 24880);
    assert_eq!(value(&r, "taxable_income"), 8780);
    assert_eq!(value(&r, "nol_2025_used"), 35120);
    assert_eq!(value(&r, "nol_2025_carryforward"), 64880);
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 64880);
    assert_eq!(return_line(&r, "8"), -3512000);
    assert_eq!(
        value(&r, "alternative_minimum_taxable_income"),
        value(&before, "alternative_minimum_taxable_income")
    );
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_regular_old_nol_deduction_exceeds_absorption_without_creating_new_loss() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 100000);
    assert_eq!(value(&r, "agi"), -40000);
    assert_eq!(value(&r, "nol_2017_used"), 43900);
    assert_eq!(value(&r, "nol_2017_carryforward"), 56100);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 0);
}

#[test]
fn prior_regular_nol_orders_vintages_and_expires_twenty_year_balances() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol =
        prior_nol(&[(2025, 30000), (2017, 10000), (2005, 2000), (2006, 10000), (2020, 10000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 39120);
    assert_eq!(value(&r, "nol_2005_expired"), 2000);
    assert_eq!(value(&r, "nol_2006_used"), 10000);
    assert_eq!(value(&r, "nol_2017_used"), 10000);
    assert_eq!(value(&r, "nol_2020_used"), 10000);
    assert_eq!(value(&r, "nol_2025_used"), 9120);
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 20880);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2006, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "nol_2006_expired"), 56100);
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 0);
}

#[test]
fn prior_regular_nol_changes_qbi_income_limit_but_not_business_or_se_earnings() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(40000)));
    let before = supported(&d);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    for id in ["self_employment_tax", "wages"] {
        assert_eq!(value(&r, id), value(&before, id));
    }
    assert_eq!(
        return_line(&r, "8") + value(&r, "prior_nol_deduction") * 100,
        return_line(&before, "8")
    );
    assert!(value(&r, "qbi_deduction") < value(&before, "qbi_deduction"));
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
}

#[test]
fn prior_regular_nol_rejects_invalid_and_leaves_out_unsupported_history() {
    {
        let years = vec![(2025, 1), (2025, 2)];
        assert!(!prior_nol(&years).valid());
    }
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1000)]);
    if let NolHistory::Records { confirmed, .. } = &mut d.screening.as_mut().unwrap().nol {
        *confirmed = false;
    }
    // Something else applies: the loss is left out with a notice.
    let r = supported(&d);
    assert!(noted(&r, "WARN_NOL_LEFT_OUT"), "{r:?}");
}

#[test]
fn prior_regular_nol_supports_senior_and_cash_charity_deduction_routes() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 10000)]);
    supported(&d);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500615;
    supported(&d);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19800615;
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    {
        d.deductions.as_mut().unwrap().itemized = facts;
        d.deductions.as_mut().unwrap().itemize_override = Some(ItemizeOverride::Itemize);
    };
    supported(&d);
    d.deductions.as_mut().unwrap().set_cash_charity(10000000);
    supported(&d);
}

#[test]
fn prior_operating_nol_reduces_niit_magi_without_reducing_investment_income() {
    for (wages, expected) in [(300000, 3800), (180000, 1140)] {
        let mut d = complete(FederalStatus::Single);
        d.income = vec![wage(1, Owner::Taxpayer, wages * 100, 0)];
        add(&mut d, income(savings_income(100000, 0)));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 50000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), wages + 50000);
        assert_eq!(value(&r, "net_investment_income_tax"), expected);
        assert_eq!(value(&r, "prior_nol_deduction"), 50000);
    }
}

#[test]
fn prior_regular_nol_capital_loss_has_distinct_deduction_and_absorption_bases() {
    for year in [2017, 2025] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(bad_debt(10000, 0)));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(year, 100000)]);
        let r = supported(&d);
        let (deduction, used, capital_carry) =
            if year < 2018 { (100000, 43900, 10000) } else { (32720, 35720, 7000) };
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, &alloc::format!("nol_{year}_used")), used);
        assert_eq!(value(&r, &alloc::format!("nol_{year}_carryforward")), 100000 - used);
        assert_eq!(value(&r, "capital_short_carryforward"), capital_carry);
        assert_eq!(value(&r, "current_net_operating_loss"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_preserves_long_capital_loss_and_ignores_disallowed_personal_loss() {
    for personal in [false, true] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Sale {
                data: sale_source(FederalSale {
                    use_kind: if personal {
                        SaleUse::Personal
                    } else {
                        SaleUse::Investment { wash_sale: 0 }
                    },
                    ..native_sale(GainTerm::Long, 0, Some(1000000))
                }),
            }),
        );
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "prior_nol_deduction"), if personal { 35120 } else { 32720 });
        assert_eq!(value(&r, "nol_2025_used"), if personal { 35120 } else { 35720 });
        assert_eq!(value(&r, "capital_long_carryforward"), if personal { 0 } else { 7000 });
        assert_eq!(value(&r, "capital_short_carryforward"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_excludes_qbi_before_itemized_section68_limit() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 55060000, 0)];
    add(&mut d, income(reit_income(100000, 100000, true)));
    let ded = d.deductions.as_mut().unwrap();
    fixture_mortgage_interest(ded, 3000000);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let baseline = supported(&d);
    assert_eq!(value(&baseline, "itemized_deduction_reduction"), 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1000000)]);
    let r = supported(&d);
    // 650,600 income without QBI; 541 section-68 reduction to the
    // 30,000 itemized deduction gives 621,141 * 80% = 496,913.
    assert_eq!(value(&r, "prior_nol_deduction"), 496913);
    assert_eq!(value(&r, "agi"), 153687);
    assert_eq!(value(&r, "qbi_deduction"), 20000);
    assert_eq!(value(&r, "itemized_deduction_reduction"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_refigures_salt_phaseout_with_actual_deduction() {
    let mut d = complete(FederalStatus::Single);
    d.income = vec![wage(1, Owner::Taxpayer, 55000000, 0)];
    let ded = d.deductions.as_mut().unwrap();
    itemized(ded).personal_property_taxes = 4000000;
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let baseline = supported(&d);
    assert_eq!(return_line(&baseline, "12e"), 2690000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 100000);
    assert_eq!(value(&r, "agi"), 450000);
    assert_eq!(return_line(&r, "12e"), 4000000);
    assert_eq!(value(&r, "taxable_income"), 410000);
}

#[test]
fn nonitemizer_percentage_excess_survives_prior_nol_without_floor_or_double_carry() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = None;
    ded.set_cash_charity(10000000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 25680);
    assert_eq!(value(&r, "prior_nol_deduction"), 34320);
    assert_eq!(return_line(&r, "12f"), 100000);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 84592);
    assert_eq!(value(&r, "charity_return_year_nol_reduction"), 0);
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn prior_nol_refigures_itemized_cash_charity_when_no_percentage_carryover_arises() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    ded.set_cash_charity(100000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 47440);
    assert_eq!(value(&r, "agi"), 12560);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 937);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 0);
    assert_eq!(value(&r, "nol_2025_carryforward"), 52560);
}

#[test]
fn direct_cash_prior_cash_and_property_fit_both_2026_a_g_ceilings() {
    // AGI $100,000: the current $20,000 G gift, $10,000 A gift, and 2025
    // $15,000 G balance fit inside the stricter shared 50% ceiling. The
    // 2026 $500 floor charges A before either G cash vintage.
    let mut draft = direct_cash_prior_cash_and_property(10_000_000, 2_000_000, 15_000, 1_000_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_cash_sixty_current_deduction"), 20_000);
    assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 9_500);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 15_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 15_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&result, "charity_floor_not_carried"), 500);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(value(&result, "form_8283_section_b_value"), 10_000);
    assert_eq!(return_line(&result, "12e"), 4_450_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 15_000);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(estimate(&restart(draft).draft), automatic);
}

#[test]
fn direct_cash_prior_cash_and_property_at_exact_fifty_percent_boundary() {
    let draft = direct_cash_prior_cash_and_property(10_000_000, 2_000_000, 20_000, 1_000_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_cash_sixty_current_deduction"), 20_000);
    assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 9_500);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 0);
    assert_eq!(return_line(&result, "12e"), 4_950_000);
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn direct_cash_prior_cash_and_property_over_fifty_percent_need_guidance() {
    let mut draft = direct_cash_prior_cash_and_property(10_000_000, 2_000_000, 20_100, 1_000_000);
    let r = supported(&draft);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 9_500);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_used"), 20_100);
    assert_eq!(return_line(&r, "12e"), 4_960_000);
    assert_eq!(restart(draft.clone()).draft, draft);
    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let r = supported(&draft);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_used"), 20_100);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_remaining"), 0);
    assert_eq!(return_line(&r, "12f"), 0);

    let mut incomplete =
        direct_cash_prior_cash_and_property(10_000_000, 2_000_000, 15_000, 1_000_000);
    incomplete.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&incomplete), "WARN_CHARITY_CARRY_LEFT_OUT"));
}

#[test]
fn current_cash_and_prior_cash_precede_appreciated_stock_and_charge_stock_floor_first() {
    // AGI $100,000: $20,000 current cash + $20,000 old cash + $10,000
    // stock exactly fill the shared 50% ceiling. The $500 floor hits stock.
    let draft =
        current_and_prior_cash_with_appreciated_stock(10_000_000, 2_000_000, 20_000, 1_000_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_cash_sixty_current_deduction"), 20_000);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 20_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&result, "charity_capital_gain_thirty_current_deduction"), 9_500);
    assert_eq!(value(&result, "charity_capital_gain_thirty_return_year_carryforward"), 0);
    assert_eq!(value(&result, "charity_cash_sixty_return_year_carryforward"), 0);
    assert_eq!(value(&result, "charity_floor_not_carried"), 500);
    assert_eq!(return_line(&result, "12e"), 4_950_000);
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn prior_cash_and_current_vehicle_sale_use_proceeds_one_floor_and_original_vintage() {
    let mut draft = sold_vehicle_for_cash_carry(30_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 100_000);
    assert_eq!(value(&result, "charity_property_1_claim"), 15_000);
    assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 14_500);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 30_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 30_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(value(&result, "form_8283_section_a_value"), 15_000);
    assert_eq!(return_line(&result, "12e"), 4_450_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    let mut without_prior = draft.clone();
    without_prior.deductions.as_mut().unwrap().charity_carryovers.clear();
    let control = supported(&without_prior);
    assert_eq!(return_line(&control, "15") - return_line(&result, "15"), 3_000_000);
    assert!(result.tax < control.tax);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 30_000);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(estimate(&restart(draft.clone()).draft), automatic);
}

#[test]
fn prior_cash_and_current_vehicle_fmv_exceptions_require_appraisal_and_section_b() {
    for treatment in [
        VehicleUse::SignificantUse {
            details: "Regular daily charity meal deliveries for one year".into(),
        },
        VehicleUse::MaterialImprovement {
            details: "Charity-funded major engine repair before transfer".into(),
        },
        VehicleUse::NeedyPerson,
    ] {
        let mut draft = cash_carry_with_vehicle(treatment, 20_000);
        assert!(noted(&supported(&draft), "WARN_GIFT_LEFT_OUT"));
        set_gift_appraisal(draft.deductions.as_mut().unwrap(), 0, 2);
        draft.next_id = draft.next_id.max(3);
        draft.deductions.as_mut().unwrap().vehicle_gifts[0].donee_acknowledged = true;
        let result = supported(&draft);
        assert_eq!(value(&result, "charity_property_1_claim"), 20_000);
        assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 19_500);
        assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 20_000);
        assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
        assert_eq!(value(&result, "form_8283_section_b_value"), 20_000);
        assert_eq!(return_line(&result, "12e"), 3_950_000);
        assert_eq!(estimate(&restart(draft.clone()).draft), result);
    }
}

#[test]
fn prior_cash_with_vehicle_above_fifty_percent_uses_approved_a_g_order() {
    let mut draft = sold_vehicle_for_cash_carry(60_000);
    let vehicle = &mut draft.deductions.as_mut().unwrap().vehicle_gifts[0];
    vehicle.basis = 8_000_000;
    vehicle.value = 6_000_000;
    draft.deductions.as_mut().unwrap().vehicle_gift_documents[0].form.box_4c = 6_000_000;
    // A60000 at50%100000 leaves10000 excess; Gold60000 uses10000 of room60000−50000.
    let r = supported(&draft);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 49_500);
    assert_eq!(value(&r, "charity_property_fifty_return_year_carryforward"), 10_500);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_remaining"), 50_000);
    assert_eq!(return_line(&r, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), r);
    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let r = supported(&draft);
    assert_eq!(value(&r, "charity_property_fifty_return_year_carryforward"), 10_500);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_used"), 10_000);
    assert_eq!(return_line(&r, "12f"), 0);
}

#[test]
fn prior_cash_and_small_vehicle_claim_charge_floor_to_vehicle_before_cash() {
    let mut draft = sold_vehicle_for_cash_carry(10_000);
    let vehicle = &mut draft.deductions.as_mut().unwrap().vehicle_gifts[0];
    vehicle.basis = 20_000;
    vehicle.value = 10_000;
    draft.deductions.as_mut().unwrap().vehicle_gift_documents[0].form.box_4c = 10_000;
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_property_1_claim"), 100);
    assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 0);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 9_600);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 400);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&result, "12e"), 960_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);
}

#[test]
fn prior_cash_and_vehicle_guard_other_classes_and_unconfirmed_sources() {
    let draft = sold_vehicle_for_cash_carry(30_000);
    let mut older = draft.clone();
    older.deductions.as_mut().unwrap().charity_carryovers[0].year = 2024;
    assert_eq!(value(&supported(&older), "charity_deduction"), 44_500);

    let mut more_vehicles = draft.clone();
    push_gift(
        more_vehicles.deductions.as_mut().unwrap(),
        donated_personal_vehicle(VehicleUse::SignificantUse {
            details: "Charity regular transportation".into(),
        }),
    );
    assert!(noted(&supported(&more_vehicles), "WARN_GIFT_LEFT_OUT"));
    assert_eq!(value(&supported(&more_vehicles), "charity_deduction"), 44_500);

    let mut goods = draft.clone();
    goods
        .deductions
        .as_mut()
        .unwrap()
        .property_gifts
        .push(ordinary_property_for_cash_carry(10_000));
    assert_eq!(value(&supported(&goods), "charity_deduction"), 44_600);

    let mut stock = draft.clone();
    stock.deductions.as_mut().unwrap().property_gifts.push(public_stock_gift());
    assert_eq!(value(&supported(&stock), "charity_deduction"), 49_500);

    let mut current_cash = draft.clone();
    current_cash.deductions.as_mut().unwrap().set_cash_charity(100_000);
    let supported_cash = supported(&current_cash);
    assert_eq!(value(&supported_cash, "charity_cash_sixty_current_deduction"), 1_000);
    assert_eq!(value(&supported_cash, "charity_carry_2025_cash_sixty_used"), 30_000);
    assert_eq!(return_line(&supported_cash, "12e"), 4_550_000);
    assert_eq!(estimate(&restart(current_cash.clone()).draft), supported_cash);
    current_cash.deductions.as_mut().unwrap().set_cash_charity(600_000);
    assert_eq!(value(&supported(&current_cash), "charity_deduction"), 50_500);

    let mut incomplete = draft.clone();
    incomplete.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&incomplete), "WARN_CHARITY_CARRY_LEFT_OUT"));

    let mut changed_filer = draft.clone();
    changed_filer.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&changed_filer), "WARN_CHARITY_CARRY_LEFT_OUT"));

    let mut foreign_tax = draft;
    payee_statement_credit(&mut foreign_tax, 10_000);
    unsupported(&foreign_tax, "Charitable carryover coordination");
}

#[test]
fn prior_2025_cash_and_current_appreciated_stock_fit_distinct_limits_and_one_floor() {
    let mut draft = cash_carry_with_appreciated_stock(10_000_000, 1_000_000, 20_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 100_000);
    assert_eq!(value(&result, "charity_capital_gain_thirty_current_deduction"), 9_500);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 20_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 0);
    assert_eq!(value(&result, "charity_floor_not_carried"), 500);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(value(&result, "form_8283_section_a_value"), 10_000);
    assert_eq!(return_line(&result, "12e"), 2_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    let mut without_prior = draft.clone();
    without_prior.deductions.as_mut().unwrap().charity_carryovers.clear();
    let control = supported(&without_prior);
    assert_eq!(return_line(&control, "15") - return_line(&result, "15"), 2_000_000);
    assert!(result.tax < control.tax);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&automatic, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(estimate(&restart(draft.clone()).draft), automatic);
}

#[test]
fn prior_cash_and_appreciated_stock_preserve_current_stock_excess_and_floor_vintage() {
    let mut draft = cash_carry_with_appreciated_stock(10_000_000, 4_000_000, 10_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_capital_gain_thirty_current_deduction"), 29_500);
    assert_eq!(value(&result, "charity_capital_gain_thirty_return_year_carryforward"), 10_500);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 10_500);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 10_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 0);
    assert_eq!(value(&result, "form_8283_section_a_value"), 40_000);
    assert_eq!(return_line(&result, "12e"), 3_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 10_000);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_capital_gain_thirty_return_year_carryforward"), 10_500);
    assert_eq!(value(&automatic, "charity_return_year_carryforward"), 10_500);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(estimate(&restart(draft.clone()).draft), automatic);
}

#[test]
fn prior_cash_and_current_appreciated_stock_meet_shared_fifty_percent_boundary() {
    let draft = cash_carry_with_appreciated_stock(10_000_000, 3_000_000, 20_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_capital_gain_thirty_current_deduction"), 29_500);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&result, "12e"), 4_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);
}

#[test]
fn prior_cash_displaces_2026_appreciated_stock_at_fifty_percent_limit() {
    let mut draft = cash_carry_with_appreciated_stock(10_000_000, 1_000_000, 41_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 41_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 41_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&result, "charity_capital_gain_thirty_current_deduction"), 8_500);
    assert_eq!(value(&result, "charity_capital_gain_thirty_return_year_carryforward"), 1_500);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 1_500);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 0);
    assert_eq!(value(&result, "form_8283_section_a_value"), 10_000);
    assert_eq!(return_line(&result, "12e"), 4_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    let mut no_prior = draft.clone();
    no_prior.deductions.as_mut().unwrap().charity_carryovers.clear();
    assert!(result.tax < supported(&no_prior).tax);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 41_000);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_capital_gain_thirty_return_year_carryforward"), 1_500);
    assert_eq!(value(&automatic, "charity_return_year_carryforward"), 1_500);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(estimate(&restart(draft).draft), automatic);
}

#[test]
fn prior_cash_above_fifty_percent_leaves_stock_carry_and_retains_cash_vintage() {
    let draft = cash_carry_with_appreciated_stock(10_000_000, 3_000_000, 70_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 60_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 10_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 59_500);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 500);
    assert_eq!(value(&result, "charity_capital_gain_thirty_current_deduction"), 0);
    assert_eq!(value(&result, "charity_capital_gain_thirty_return_year_carryforward"), 30_000);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 30_000);
    assert_eq!(value(&result, "form_8283_section_a_value"), 30_000);
    assert_eq!(return_line(&result, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    let mut standard = draft;
    standard.deductions.as_mut().unwrap().itemize_override = None;
    let result = supported(&standard);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 60_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 10_000);
    assert_eq!(value(&result, "charity_capital_gain_thirty_return_year_carryforward"), 30_000);
    assert_eq!(return_line(&result, "12f"), 0);
    assert_eq!(estimate(&restart(standard).draft), result);
}

#[test]
fn prior_cash_and_appreciated_stock_guard_other_sources() {
    let draft = cash_carry_with_appreciated_stock(10_000_000, 1_000_000, 20_000);
    let mut older = draft.clone();
    older.deductions.as_mut().unwrap().charity_carryovers[0].year = 2024;
    assert_eq!(value(&supported(&older), "charity_deduction"), 29_500);

    let mut more_stock = draft.clone();
    more_stock.deductions.as_mut().unwrap().property_gifts.push(public_stock_gift());
    assert_eq!(value(&supported(&more_stock), "charity_deduction"), 49_500);

    let mut wrong_class = draft.clone();
    wrong_class.deductions.as_mut().unwrap().basis_election = true;
    assert_eq!(value(&supported(&wrong_class), "charity_deduction"), 22_000);

    let mut other_property = draft.clone();
    other_property
        .deductions
        .as_mut()
        .unwrap()
        .property_gifts
        .push(ordinary_property_for_cash_carry(10_000));
    assert_eq!(value(&supported(&other_property), "charity_deduction"), 29_600);

    let mut changed_filer = draft.clone();
    changed_filer.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&changed_filer), "WARN_CHARITY_CARRY_LEFT_OUT"));

    let mut foreign_tax = draft;
    payee_statement_credit(&mut foreign_tax, 10_000);
    unsupported(&foreign_tax, "Charitable carryover coordination");

    let mut net_operating_loss = cash_carry_with_appreciated_stock(10_000_000, 1_000_000, 20_000);
    net_operating_loss.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1_000)]);
    unsupported(&net_operating_loss, "Charitable carryover coordination");
}

#[test]
fn prior_2025_cash_and_current_property_use_available_class_capacity() {
    let mut draft = cash_carry_with_current_property(6_000_000, 500_000, 40_000);
    // A uses $5,000 first; G uses its remaining $31,000 room. The $300
    // floor is assigned to A, whose current contribution has no excess.
    let r = supported(&draft);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 4_700);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_used"), 31_000);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_remaining"), 9_000);
    assert_eq!(return_line(&r, "12e"), 3_570_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), r);
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let r = supported(&draft);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_used"), 31_000);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_remaining"), 9_000);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 0);
}

#[test]
fn prior_cash_and_appraised_property_use_approved_a_g_order() {
    let draft = cash_carry_with_current_property(10_000_000, 5_000_000, 60_000);
    // A50000 first; G room60000−50000=10000. Floor500 has no Aexcess and is lost.
    let r = supported(&draft);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 49_500);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_used"), 10_000);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_remaining"), 50_000);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&r, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft).draft), r);
}

#[test]
fn prior_cash_with_property_percentage_excess_uses_property_then_cash() {
    let mut draft = cash_carry_with_current_property(6_000_000, 3_500_000, 40_000);
    // A35000 at50%60000 uses30000; Golder40000 gets36000−30000=6000. Floor300 goes to A and carries with its5000 excess.
    let r = supported(&draft);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 29_700);
    assert_eq!(value(&r, "charity_property_fifty_return_year_carryforward"), 5_300);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_used"), 6_000);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_remaining"), 34_000);
    assert_eq!(return_line(&r, "12e"), 3_570_000);
    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let r = supported(&draft);
    assert_eq!(value(&r, "charity_property_fifty_return_year_carryforward"), 5_300);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_used"), 6_000);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_remaining"), 34_000);
    assert_eq!(estimate(&restart(draft).draft), r);
}

#[test]
fn prior_cash_with_small_property_charges_floor_to_property_then_old_cash() {
    let draft = cash_carry_with_current_property(6_000_000, 10_000, 10_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_property_fifty_current_deduction"), 0);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 10_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 9_800);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 200);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&result, "12e"), 980_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);
}

#[test]
fn prior_cash_with_property_requires_one_2025_balance_and_guarded_sources() {
    let draft = cash_carry_with_current_property(6_000_000, 500_000, 20_000);
    let mut older = draft.clone();
    older.deductions.as_mut().unwrap().charity_carryovers[0].year = 2024;
    assert_eq!(value(&supported(&older), "charity_deduction"), 24_700);

    let mut multiple = draft.clone();
    multiple.deductions.as_mut().unwrap().charity_carryovers.push(prior_cash_charity(2024, 1_000));
    assert_eq!(value(&supported(&multiple), "charity_deduction"), 25_700);

    let mut current_cash = draft.clone();
    current_cash.deductions.as_mut().unwrap().set_cash_charity(100_000);
    let supported_cash = supported(&current_cash);
    assert_eq!(value(&supported_cash, "charity_cash_sixty_current_deduction"), 1_000);
    assert_eq!(value(&supported_cash, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(return_line(&supported_cash, "12e"), 2_570_000);
    assert_eq!(estimate(&restart(current_cash.clone()).draft), supported_cash);
    current_cash.deductions.as_mut().unwrap().set_cash_charity(600_000);
    assert_eq!(value(&supported(&current_cash), "charity_deduction"), 30_700);

    let mut incomplete = draft.clone();
    incomplete.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&incomplete), "WARN_CHARITY_CARRY_LEFT_OUT"));

    let mut additional_property = draft.clone();
    additional_property
        .deductions
        .as_mut()
        .unwrap()
        .property_gifts
        .push(ordinary_property_for_cash_carry(10_000));
    // The original $5,000 jewelry plus the added $100 now cross the
    // similar-property threshold. Supply the required actual appraisal.
    set_gift_appraisal(additional_property.deductions.as_mut().unwrap(), 0, 2);
    set_gift_appraisal(additional_property.deductions.as_mut().unwrap(), 1, 2);
    additional_property.next_id = 3;
    assert_eq!(value(&supported(&additional_property), "charity_deduction"), 24_800);

    let mut changed_filer = draft.clone();
    changed_filer.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&changed_filer), "WARN_CHARITY_CARRY_LEFT_OUT"));

    let mut foreign_tax = draft;
    payee_statement_credit(&mut foreign_tax, 10_000);
    unsupported(&foreign_tax, "Charitable carryover coordination");
}

#[test]
fn prior_cash_charity_uses_oldest_vintage_and_2026_itemizer_floor() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    // The entered order is deliberately newest first; the 2021 balance uses
    // the first $30,000 of a $36,000 percentage-limit window.
    ded.charity_carryovers =
        vec![prior_cash_charity(2022, 20_000), prior_cash_charity(2021, 30_000)];
    let result = supported(&d);
    assert_eq!(value(&result, "agi"), 60_000);
    assert_eq!(value(&result, "charity_carry_2021_cash_sixty_used"), 30_000);
    assert_eq!(value(&result, "charity_carry_2022_cash_sixty_used"), 6_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 35_700);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 300);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 14_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_expired_after_return_year"), 0);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&result, "12e"), 3_570_000);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    d.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&d);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_deduction"), 35_700);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_remaining_following_year"), 14_000);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_floor_not_carried"), 300);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), automatic);
}

#[test]
fn prior_cash_charity_2021_residual_expires_and_unresolved_sources_block() {
    let mut d = complete(FederalStatus::Single);
    d.deductions.as_mut().unwrap().charity_carryovers = vec![prior_cash_charity(2021, 50_000)];
    let result = supported(&d);
    assert_eq!(value(&result, "charity_carry_2021_cash_sixty_used"), 36_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_expired_after_return_year"), 14_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    d.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&d), "WARN_CHARITY_CARRY_LEFT_OUT"));
    d.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = true;
    d.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&d), "WARN_CHARITY_CARRY_LEFT_OUT"));
    d.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = true;
    d.deductions.as_mut().unwrap().itemize_override = None;
    d.deductions.as_mut().unwrap().set_cash_charity(1_000_000);
    let mixed = supported(&d);
    assert_eq!(value(&mixed, "charity_carry_2021_cash_sixty_used"), 26_000);
    // 2026-11-02: the retained cash gifts select itemizing after the300 floor.
    assert_eq!(value(&mixed, "deduction"), 35_700);
    assert_eq!(value(&mixed, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&mixed, "charity_prior_cash_sixty_expired_after_return_year"), 24_000);
    assert_eq!(return_line(&mixed, "12f"), 0);

    d.deductions.as_mut().unwrap().set_cash_charity(0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1_000)]);
    unsupported(&d, "Charitable carryover coordination");
    d.screening.as_mut().unwrap().nol = NolHistory::None;

    let ded = d.deductions.as_mut().unwrap();
    ded.charity_carryovers.push(prior_cash_charity(2021, 1_000));
    assert!(!d.valid_stored());
    d.deductions.as_mut().unwrap().charity_carryovers.pop();
    d.deductions.as_mut().unwrap().charity_carryovers[0].balance += 1;
    assert!(!d.valid_stored());
}

#[test]
fn prior_cash_and_current_short_term_stock_use_derived_fifty_percent_class() {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![wage(1, Owner::Taxpayer, 10_000_000, 650_000)];
    let ded = draft.deductions.as_mut().unwrap();
    ded.charity_carryovers = vec![prior_cash_charity(2025, 60_000)];
    let mut g = public_stock_gift();
    g.acquired = 20260501;
    g.value = 5_000_000;
    g.basis = 6_000_000;
    ded.property_gifts.push(g);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    // A50000 is ordinary-value property; Golder60000 gets10000; floor500 loses within A without excess.
    let r = supported(&draft);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 49_500);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_used"), 10_000);
    assert_eq!(value(&r, "charity_carry_2025_cash_sixty_remaining"), 50_000);
    assert_eq!(return_line(&r, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft).draft), r);
}

#[test]
fn current_cash_gifts_use_room_before_prior_cash_and_share_one_floor() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    ded.set_cash_charity(1_000_000);
    ded.charity_carryovers =
        vec![prior_cash_charity(2022, 20_000), prior_cash_charity(2021, 30_000)];
    let r = supported(&d);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 10_000);
    assert_eq!(value(&r, "charity_prior_cash_sixty_deduction"), 25_700);
    assert_eq!(value(&r, "charity_prior_cash_sixty_floor_not_carried"), 300);
    assert_eq!(value(&r, "charity_carry_2021_cash_sixty_used"), 26_000);
    assert_eq!(value(&r, "charity_carry_2022_cash_sixty_used"), 0);
    assert_eq!(value(&r, "charity_prior_cash_sixty_expired_after_return_year"), 4_000);
    assert_eq!(value(&r, "charity_prior_cash_sixty_remaining_following_year"), 20_000);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&r, "12e"), 3_570_000);
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    d.deductions.as_mut().unwrap().set_cash_charity(3_590_000);
    let r = supported(&d);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 35_700);
    assert_eq!(value(&r, "charity_prior_cash_sixty_deduction"), 0);
    assert_eq!(value(&r, "charity_prior_cash_sixty_floor_not_carried"), 100);
    assert_eq!(value(&r, "charity_carry_2021_cash_sixty_used"), 100);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&r, "12e"), 3_570_000);

    d.deductions.as_mut().unwrap().set_cash_charity(4_000_000);
    let r = supported(&d);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 35_700);
    assert_eq!(value(&r, "charity_prior_cash_sixty_deduction"), 0);
    assert_eq!(value(&r, "charity_prior_cash_sixty_floor_not_carried"), 0);
    assert_eq!(value(&r, "charity_carry_2021_cash_sixty_used"), 0);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 4_300);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

#[test]
fn prior_nol_refigures_social_security_without_reducing_reported_benefits() {
    let mut d = nol_social();
    assert_eq!(return_line(&supported(&d), "6b"), 1700000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 32720);
    assert_eq!(return_line(&r, "6a"), 2000000);
    assert_eq!(return_line(&r, "6b"), 0);
    assert_eq!(value(&r, "agi"), 7280);
    assert_eq!(value(&r, "nol_2025_carryforward"), 67280);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_social_security_refigures_each_vintages_modified_income() {
    let mut d = nol_social();
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 10000), (2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 34720);
    assert_eq!(value(&r, "agi"), 5280);
    assert_eq!(value(&r, "nol_2017_used"), 10000);
    // Earlier loss lowers taxable benefits from $17,000 to $9,600 in the
    // modern vintage's absorption pass: $30,000 + $9,600 - $16,100 - $6,180.
    assert_eq!(value(&r, "nol_2025_used"), 17320);
    assert_eq!(value(&r, "nol_2025_carryforward"), 82680);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_capital_addback_refigures_social_security_for_absorption() {
    let mut d = nol_social();
    add(&mut d, income(collectible_sale(0, 10000, GainTerm::Short, false)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 29160);
    assert_eq!(value(&r, "agi"), 7840);
    // Adding back the $3,000 net capital deduction also increases modified
    // taxable benefits by $1,450. Both amounts affect loss consumption.
    assert_eq!(value(&r, "nol_2025_used"), 33610);
    assert_eq!(value(&r, "nol_2025_carryforward"), 66390);
}

#[test]
fn prior_nol_senior_deduction_reduces_limit_but_not_carryover_income() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19560615;
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 28680);
    assert_eq!(value(&r, "senior_deduction"), 6000);
    assert_eq!(value(&r, "agi"), 31320);
    assert_eq!(value(&r, "taxable_income"), 7170);
    assert_eq!(value(&r, "nol_2025_used"), 34680);
    assert_eq!(value(&r, "nol_2025_carryforward"), 65320);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_restores_senior_phaseout_without_retroactively_changing_limit() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19560615;
    d.income[0] = wage(1, Owner::Taxpayer, 10000000, 0);
    assert_eq!(value(&supported(&d), "senior_deduction"), 4500);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 61880);
    assert_eq!(value(&r, "senior_deduction"), 6000);
    assert_eq!(value(&r, "agi"), 38120);
    assert_eq!(value(&r, "nol_2025_used"), 66380);
    assert_eq!(value(&r, "nol_2025_carryforward"), 33620);
}

#[test]
fn prior_nol_includes_work_and_vehicle_deductions_in_limit_and_absorption() {
    let mut d = nol_work_deductions(60000, 20000, 5000, 5000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 11120);
    assert_eq!(value(&r, "agi"), 48880);
    assert_eq!(value(&r, "taxable_income"), 2780);
    assert_eq!(value(&r, "tips_deduction"), 20000);
    assert_eq!(value(&r, "overtime_deduction"), 5000);
    assert_eq!(value(&r, "vehicle_interest_deduction"), 5000);
    assert_eq!(value(&r, "nol_2025_carryforward"), 88880);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_restores_work_phaseouts_in_final_return_and_later_vintage() {
    let mut d = nol_work_deductions(200000, 20000, 10000, 10000);
    let before = supported(&d);
    assert_eq!(value(&before, "tips_deduction"), 15000);
    assert_eq!(value(&before, "overtime_deduction"), 5000);
    assert_eq!(value(&before, "vehicle_interest_deduction"), 0);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 50000), (2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 141120);
    assert_eq!(value(&r, "tips_deduction"), 20000);
    assert_eq!(value(&r, "overtime_deduction"), 10000);
    assert_eq!(value(&r, "vehicle_interest_deduction"), 10000);
    assert_eq!(value(&r, "nol_2025_used"), 81120);
    assert_eq!(value(&r, "nol_2025_carryforward"), 18880);
}

#[test]
fn prior_nol_vehicle_phaseout_changes_modified_income_after_older_loss() {
    let mut d = nol_work_deductions(120000, 0, 0, 10000);
    assert_eq!(value(&supported(&d), "vehicle_interest_deduction"), 6000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 30000), (2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 84320);
    assert_eq!(value(&r, "vehicle_interest_deduction"), 10000);
    assert_eq!(value(&r, "nol_2025_used"), 50320);
    assert_eq!(value(&r, "nol_2025_carryforward"), 49680);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_rental_allowance_is_refigured_for_each_loss_vintage() {
    let mut d = nol_rental(150000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 30000), (2025, 200000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 113120);
    // The earlier $30,000 loss leaves $120,000 rental MAGI: $15,000 rental
    // loss is allowed in the modern vintage's modified-income pass.
    assert_eq!(value(&r, "nol_2025_used"), 68120);
    assert_eq!(value(&r, "nol_2025_carryforward"), 131880);
    assert_eq!(value(&r, "rental_income"), -25000);
    assert_eq!(value(&r, "rental_suspended_loss"), 6000);
    assert_eq!(value(&r, "agi"), 11880);
}

#[test]
fn prior_nol_does_not_make_inactive_rental_losses_nonpassive() {
    let mut d = nol_rental(150000);
    let FederalIncome::Rental { data } = &mut d.income[1].income else { unreachable!() };
    data.participation = crate::RentalParticipation::NotActive;
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 200000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 0);
    assert_eq!(value(&r, "rental_suspended_loss"), 31000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 66100);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_uses_prior_refund_tax_benefit_without_reintroducing_amt_income() {
    for (history, taxable, deduction) in
        [(refund_history(), 1000, 35920), (TaxRefundHistory::NotDeducted, 0, 35120)]
    {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(refund_income(300000, 0, history)));
        let before = supported(&d);
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "taxable_state_tax_refund"), taxable);
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, "agi"), 60000 + taxable - deduction);
        assert_eq!(value(&r, "nol_2025_carryforward"), 100000 - deduction);
        assert_eq!(
            value(&r, "alternative_minimum_taxable_income"),
            value(&before, "alternative_minimum_taxable_income")
        );
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_ordinary_awards_keep_exclusions_and_jury_remittance() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(prize(PrizeDisposition::Received)));
    add(&mut d, income(scholarship(5000, 2000, 3000)));
    add(
        &mut d,
        income(FederalIncome::JuryDuty {
            data: crate::JuryPay { received: 100000, remitted: 70000 },
        }),
    );
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_prizes"), 3500);
    assert_eq!(value(&r, "prior_nol_deduction"), 39760);
    assert_eq!(value(&r, "agi"), 26040);
    assert_eq!(value(&r, "nol_2025_carryforward"), 60240);
    assert_eq!(return_line(&r, "1a"), 6000000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_does_not_turn_nonservice_prize_into_earned_income() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(prize(PrizeDisposition::Received)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 10000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), -6500);
    assert_eq!(value(&r, "nol_2017_carryforward"), 10000);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_gambling_preserves_loss_limit_and_compares_complete_deduction_routes() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, gambling(30000, 30000));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    elect_itemized(&mut d);
    let r = supported(&d);
    assert_eq!(value(&r, "gambling_loss_deduction"), 27000);
    assert_eq!(value(&r, "prior_nol_deduction"), 50400);
    assert_eq!(value(&r, "agi"), 39600);
    assert_eq!(value(&r, "taxable_income"), 12600);
    assert_eq!(value(&r, "nol_2025_carryforward"), 49600);
    assert_eq!(restart(d.clone()).draft, d);
    // 2026-11-02: automatic selection keeps the retained eligible gambling loss.
    d.deductions.as_mut().unwrap().itemize_override = None;
    let r = supported(&d);
    assert_eq!(value(&r, "gambling_loss_deduction"), 27000);
    assert_eq!(value(&r, "prior_nol_deduction"), 50400);
    assert_eq!(value(&r, "taxable_income"), 12600);
}

#[test]
fn prior_nol_gambling_losses_cannot_exceed_winnings_or_create_new_nol() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, gambling(30000, 40000));
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "gambling_loss_deduction"), 30000);
    assert_eq!(value(&r, "agi"), -10000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 40000);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_home_sale_uses_gain_after_section121_exclusion() {
    for (price, gain, deduction) in [(80000000, 300000, 275120), (50000000, 0, 35120)] {
        let mut d = complete(FederalStatus::Single);
        let mut home = home_sale(false);
        home.price = price;
        add(&mut d, income(FederalIncome::HomeSale { data: main_home_source(home) }));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 500000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "home_2_excluded_gain"), 250000);
        assert_eq!(value(&r, "capital_gain"), gain);
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, "agi"), 60000 + gain - deduction);
        assert_eq!(value(&r, "nol_2025_carryforward"), 500000 - deduction);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_hobby_sales_recover_goods_cost_without_creating_business_losses() {
    for (receipts, hobby, deduction) in [(10000, 4000, 38320), (1000, 0, 35120)] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(hobby_sales(receipts, 8000, 2000)));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "hobby_sales_income"), hobby);
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, "nol_2025_carryforward"), 100000 - deduction);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(value(&r, "qbi_deduction"), 0);
        assert_eq!(value(&r, "current_net_operating_loss"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_cannot_restore_deduction_for_program_paid_interest() {
    let mut d = complete(FederalStatus::Single);
    let a = d.adjustments.as_mut().unwrap();
    a.set_student_loan_interest(300000);
    add(&mut d, health_loan_assistance(HealthLoanProgram::National, 300000));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "health_loan_interest_removed"), 3000);
    assert_eq!(value(&r, "agi"), -40000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 56100);
    assert_eq!(restart(d.clone()).draft, d);
    if let FederalIncome::HealthLoanAssistance { data } = &mut d.income[1].income {
        data.confirmed = false;
    }
    assert!(noted(&supported(&d), "WARN_HEALTH_LOAN_ASSISTANCE_ASSUMED_TAXABLE"));
}

#[test]
fn prior_nol_does_not_release_personal_rental_losses_against_portfolio_income() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_rent(1000, 13000)));
    add(&mut d, income(royalty(30000, 5000)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), -15000);
    assert_eq!(value(&r, "personal_rental_2_suspended_loss"), 12000);
    assert_eq!(value(&r, "amt_personal_rental_2_suspended_loss"), 12000);
    assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 1000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 31100);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_mixed_rental_pool_refigures_allowance_for_each_loss_vintage() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12000000, 0);
    add(&mut d, income(FederalIncome::Rental { data: rental(9000, 30000) }));
    add(&mut d, income(personal_rent(12000, 2000)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 20000), (2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "personal_rental_passive_income"), 10000);
    assert_eq!(value(&r, "rental_income"), -31000);
    assert_eq!(value(&r, "rental_suspended_loss"), 0);
    assert_eq!(value(&r, "prior_nol_deduction"), 75120);
    assert_eq!(value(&r, "agi"), 23880);
    assert_eq!(value(&r, "nol_2017_carryforward"), 0);
    assert_eq!(value(&r, "nol_2025_carryforward"), 50880);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_section1244_retains_ordinary_cap_and_capital_loss_carryforward() {
    for (year, deduction, nol_carry, capital_carry) in
        [(2025, 24720, 72280, 17000), (2017, 100000, 66100, 20000)]
    {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 10000000, 0);
        add(&mut d, income(section1244_loss(80000, 10000, 20200101)));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(year, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "section_1244_ordinary_loss"), 50000);
        assert_eq!(value(&r, "capital_gain"), -3000);
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, "agi"), 47000 - deduction);
        assert_eq!(value(&r, &alloc::format!("nol_{year}_carryforward")), nol_carry);
        assert_eq!(value(&r, "capital_long_carryforward"), capital_carry);
        assert_eq!(value(&r, "current_net_operating_loss"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_undistributed_gain_nets_capital_loss_without_changing_tax_paid_credit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(undistributed_gain()));
    add(&mut d, income(collectible_sale(0, 10000, GainTerm::Short, false)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "rate_28_gain"), 0);
    assert_eq!(value(&r, "prior_nol_deduction"), 32720);
    assert_eq!(value(&r, "nol_2025_carryforward"), 64280);
    assert_eq!(value(&r, "capital_short_carryforward"), 2000);
    assert_eq!(value(&r, "fund_2_basis_increase"), 3950);
    assert_eq!(value(&r, "undistributed_gain_tax_credit"), 1050);
    assert_eq!(restart(d.clone()).draft, d);
    if let FederalIncome::UndistributedGain { data } = &mut d.income[1].income {
        data.form.section_1202_gain = 10000;
        data.section_1202_exclusion = Some(crate::FundQsbsExclusion::Hundred);
    }
    // Box 1c is calculated through its exclusion choice: a code Q row, no notice.
    let r = supported(&d);
    assert!(r.lines.iter().any(|l| l.id.ends_with("_1_adjustment") && l.amount == -10000));
    assert!(!noted(&r, "WARN_UNDISTRIBUTED_GAIN_LEFT_OUT"));
}

#[test]
fn zero_insolvency_exclusion_does_not_reduce_prior_nol() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, insolvency_debt(1500, 7500, 7600, 6100));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "debt_2_insolvency_exclusion"), 0);
    assert_eq!(value(&r, "personal_debt_cancellation"), 1500);
    assert_eq!(value(&r, "nol_2017_carryforward"), 54600);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_domestic_atnol_uses_nonexcluded_amti_for_ordinary_limit() {
    for (wages, excluded, deduction, base, carry) in
        [(400000, 132900, 240390, 26710, 59610), (100000, 100000, 0, 0, 300000)]
    {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(foreign_pay(wages)));
        set_claim(&mut d, foreign_claim(Owner::Taxpayer));
        let before = supported(&d);
        d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2025, 300000, false)]);
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_earned_exclusion"), excluded);
        assert_eq!(value(&r, "prior_atnol_deduction"), deduction);
        assert_eq!(value(&r, "alternative_minimum_taxable_income"), base);
        assert_eq!(value(&r, "atnol_2025_carryforward"), carry);
        assert_eq!(value(&r, "agi"), value(&before, "agi"));
        assert_eq!(value(&r, "regular_income_tax"), value(&before, "regular_income_tax"));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn debt_reduces_current_regular_and_amt_losses_before_prior_years() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(-100000)));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 10000)]);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2020, 20000, false)]);
    let before = supported(&d);
    add(&mut d, insolvency_debt(50000, 80000, 10000, 40000));
    let r = supported(&d);
    assert_eq!(value(&r, "current_net_operating_loss"), 40000);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 40000);
    assert_eq!(value(&r, "current_nol_debt_reduction"), 40000);
    assert_eq!(value(&r, "amt_current_nol_debt_reduction"), 40000);
    assert_eq!(value(&r, "nol_2017_debt_reduction"), 10000);
    assert_eq!(value(&r, "atnol_2020_debt_reduction"), 10000);
    assert_eq!(value(&r, "nol_carryforward"), 0);
    assert_eq!(value(&r, "amt_nol_carryforward"), 0);
    assert_eq!(value(&r, "regular_total_nol_carryforward"), 0);
    assert_eq!(value(&r, "amt_total_nol_carryforward"), 10000);
    assert_eq!(value(&r, "debt_3_basis_reduction"), 0);
    assert_eq!(value(&r, "debt_3_amt_basis_reduction"), 0);
    assert_eq!(r.tax, before.tax);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn amt_only_prior_capital_loss_changes_amt_without_changing_regular_return() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic ISO", 2000.0, 1000, 11000)];
    let before = supported(&d);
    d.screening.as_mut().unwrap().capital_history = vec![amt_only_capital_history()];
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, "amt_capital_gain"), -3000);
    assert_eq!(value(&r, "amt_capital_adjustment"), -3000);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 7000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 257000);
    assert_eq!(r.tax, before.tax.map(|v| v - 78000));
    assert_eq!(value(&r, "regular_income_tax"), value(&before, "regular_income_tax"));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn minimum_tax_credit_uses_prior_return_facts_and_retains_unused_credit() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let r = supported(&d);
    assert_eq!(value(&r, "minimum_tax_exclusion_tax"), 0);
    assert_eq!(value(&r, "minimum_tax_credit_generated"), 10000);
    assert_eq!(value(&r, "minimum_tax_credit_available"), 15000);
    assert_eq!(value(&r, "minimum_tax_credit"), 5023);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 9977);
    assert_eq!(return_line(&r, "20"), 502300);
    assert_eq!(return_line(&r, "22"), 0);
    assert_eq!(r.tax, Some(0));
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(value(&r, "taxable_income"), value(&before, "taxable_income"));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn minimum_tax_signed_exclusions_and_negative_prior_income_are_preserved() {
    let mut d = complete(FederalStatus::Single);
    let mut h = minimum_tax_history();
    h.base = 10000000;
    h.regular = 0;
    h.exclusions =
        vec![MinimumTaxExclusion { source: "2025 Form 6251 line 2b".into(), amount: -2000000 }];
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(h));
    assert_eq!(value(&supported(&d), "minimum_tax_exclusion_tax"), 0);
    let h = &mut d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts;
    h.base = -10000000;
    h.exclusions[0].amount = 15000000;
    h.exclusions[0].source = "2025 Form 6251 line 2g".into();
    assert_eq!(value(&supported(&d), "minimum_tax_exclusion_tax"), 0);
}

#[test]
fn minimum_tax_credit_cannot_offset_current_amt_or_create_refund() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic ISO", 2000.0, 1000, 11000)];
    let before = supported(&d);
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    let r = supported(&d);
    assert_eq!(value(&r, "minimum_tax_credit_limit"), 0);
    assert_eq!(value(&r, "minimum_tax_credit"), 0);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 15000);
    assert_eq!(r.tax, before.tax);
    d.screening.as_mut().unwrap().iso_exercises.clear();
    d.income.clear();
    let r = supported(&d);
    assert_eq!(value(&r, "minimum_tax_credit"), 0);
    assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 15000);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn minimum_tax_prior_gains_use_regular_band_occupancy_and_2025_rates() {
    for (zero, fifteen, expected) in [
        (80000, 80000, 23594),   // $61,900 at 26% and $50,000 at 15%.
        (0, 0, 16342),           // $48,350 at zero, $1,650 at 15%, rounded with ordinary tax.
        (530000, 530000, 25924), // Only $3,400 of 15% room; remainder at 20%.
        (600000, 600000, 26094), // All preferential income at 20%.
    ] {
        let mut d = complete(FederalStatus::Single);
        let mut h = minimum_tax_history();
        h.regular = 0;
        h.amt = 5000000;
        h.carry = 0;
        h.gains = Some(minimum_tax_gains(50000, 50000, zero, fifteen));
        minimum_tax_exclusion_base(&mut h, 200000);
        d.credits.as_mut().unwrap().minimum_tax =
            Some(crate::MinimumTaxHistories::from_current_unit(h));
        let r = supported(&d);
        assert_eq!(value(&r, "minimum_tax_exclusion_tax"), expected);
        assert_eq!(value(&r, "minimum_tax_credit_generated"), 50000 - expected);
        assert_eq!(value(&r, "minimum_tax_credit_carryforward"), 50000 - expected - 5023);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn minimum_tax_prior_gains_keep_25_percent_pool_and_distinct_ordinary_bases() {
    let mut d = complete(FederalStatus::Single);
    let mut h = minimum_tax_history();
    h.base = 20000000;
    h.exclusions.clear();
    h.regular = 0;
    h.gains = Some(minimum_tax_gains(50000, 70000, 80000, 60000));
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(h));
    // $41,900 ordinary at 26%, $50,000 at 15%, $20,000 at 25%.
    assert_eq!(value(&supported(&d), "minimum_tax_exclusion_tax"), 23394);
    let g = d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0]
        .facts
        .gains
        .as_mut()
        .unwrap();
    let MinimumTaxRateFacts::ScheduleD(g) = g else { panic!() };
    g.regular_zero = 54000000;
    g.regular_fifteen = 50000000;
    minimum_tax_exclusion_base(
        &mut d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts,
        200000,
    );
    // The 15% band uses $500,000, not $540,000: $33,400 at 15% and $16,600 at 20%.
    assert_eq!(value(&supported(&d), "minimum_tax_exclusion_tax"), 24224);
}

#[test]
fn minimum_tax_prior_gain_bands_follow_each_prior_status() {
    for (status, exemption, zero, fifteen) in [
        (FederalStatus::Single, 88100, 48350, 533400),
        (FederalStatus::Head, 88100, 64750, 566700),
        (FederalStatus::Joint, 137000, 96700, 600050),
        (FederalStatus::Surviving, 137000, 96700, 600050),
        (FederalStatus::Separate, 68500, 48350, 300000),
    ] {
        let mut d =
            complete(if status == FederalStatus::Joint { status } else { FederalStatus::Single });
        let mut h = minimum_tax_history();
        h.status = status;
        h.base = (exemption + 5000) * 100;
        h.exclusions.clear();
        h.regular = 0;
        h.gains = Some(minimum_tax_gains(2000, 2000, zero - 1000, zero - 1000));
        d.credits.as_mut().unwrap().minimum_tax =
            Some(crate::MinimumTaxHistories::from_current_unit(h));
        // $3,000 at 26%, $1,000 free, $1,000 at 15%.
        assert_eq!(value(&supported(&d), "minimum_tax_exclusion_tax"), 930, "{status:?}");
        d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts.gains =
            Some(minimum_tax_gains(2000, 2000, fifteen - 1000, fifteen - 1000));
        minimum_tax_exclusion_base(
            &mut d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts,
            exemption + 5000,
        );
        // $3,000 at 26%, $1,000 at 15%, $1,000 at 20%.
        assert_eq!(value(&supported(&d), "minimum_tax_exclusion_tax"), 1130, "{status:?}");
    }
}

#[test]
fn minimum_tax_prior_gain_record_rejects_inconsistent_pools_and_missing_history() {
    let mut d = complete(FederalStatus::Single);
    let mut h = minimum_tax_history();
    h.gains = Some(minimum_tax_gains(50000, 50000, 100000, 100000));
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(h));
    // An inconsistent retained source is invalid, rather than a zero credit.
    assert!(!d.valid_stored());
    assert_eq!(estimate(&d).state, ResultState::Incomplete);
    let h = &mut d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts;
    let MinimumTaxRateFacts::ScheduleD(g) = h.gains.as_mut().unwrap() else { panic!() };
    // Repair the ordinary-income source before independently checking each pool.
    g.regular_zero = 3_000_000;
    g.regular_fifteen = 3_000_000;
    g.gain_cap -= 1;
    assert!(!d.valid_stored());
    let g = d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0]
        .facts
        .gains
        .as_mut()
        .unwrap();
    let MinimumTaxRateFacts::ScheduleD(g) = g else { panic!() };
    g.gain_cap += 1;
    g.regular_fifteen += 1;
    assert!(!d.valid_stored());
    let h = &mut d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts;
    let MinimumTaxRateFacts::ScheduleD(g) = h.gains.as_mut().unwrap() else { panic!() };
    g.regular_fifteen -= 1;
    h.mtcnol = crate::MinimumTaxLoss::Unavailable;
    assert!(noted(&supported(&d), "WARN_MTC_NOL_LEFT_OUT"));
}

#[test]
fn current_sstb_full_exclusion_does_not_erase_prior_qualified_loss() {
    let mut source = scorp_current_loss_statement();
    source.loss_qbi.passive = vec![loss_year(2020, 2_000, 2_000, 2_000, 2_000)];
    let statement = source.qbi_statement.as_ref().unwrap();
    let prepared = statement.prepare(&source, 300_000, FederalStatus::Single).unwrap();
    let activities =
        statement.activities(&prepared, crate::tests::support::builders::k1_amount(0), 0).unwrap();
    assert_eq!(activities.len(), 2);
    assert_eq!(activities[0].income, 0);
    assert_eq!(activities[1].income, -2_000);
    assert!(!activities[1].active);
}
#[test]
fn historical_excess_rejects_amount_above_ordinary_loss_without_mutation() {
    let source = historical_k1_excess(400_000, false);
    let mut activities = source
        .qbi_statement
        .as_ref()
        .unwrap()
        .activities(&source, crate::tests::support::builders::k1_amount(0), 0)
        .unwrap();
    assert!(
        crate::scorp_qbi::loss_only_business_excess(&[&source], &mut activities, 400_000, 400_001)
            .is_err()
    );
    assert_eq!(activities.iter().map(|v| v.income).sum::<i64>(), -400_000);
    assert_eq!(
        crate::scorp_qbi::loss_only_business_excess(&[&source], &mut activities, 400_000, 144_000)
            .unwrap(),
        144_000
    );
    assert_eq!(activities.iter().map(|v| v.income).sum::<i64>(), -256_000);
}

#[test]
fn historical_fifo_release_precedes_new_excess_loss_qualified_percentage() {
    for at_risk in [false, true] {
        let mut source = historical_k1_excess(200_000, at_risk);
        let years = vec![
            loss_year(2017, 100_000, 0, 100_000, 0),
            loss_year(2020, 400_000, 200_000, 400_000, 200_000),
        ];
        if at_risk {
            source.risk.prior_loss = 50_000_000;
            source.loss_qbi.at_risk = years;
        } else {
            source.prior_loss = 50_000_000;
            source.loss_qbi.history = years;
        }
        source.amt = Some(amt_records(&source));
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
        let r = supported(&d);
        // 400k released: oldest 100k non-QBI plus 300k at 50% = 150k QBI.
        // Form 461 suspends 144/400 of that 150k: 54k, leaving 96k.
        assert_eq!(value(&r, "qbi_loss_carryforward"), 96_000);
        assert_eq!(value(&r, "nol_carryforward"), 340_000);
        assert!(r.review_notes.iter().any(|n| n.starts_with(
            "K-1 excess-loss character: NOL carryforward $144000; qualified portion $54000."
        )));
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn historical_loss_only_allocation_preserves_total_when_sources_reverse() {
    let mut baseline = None;
    for reverse in [false, true] {
        let mut sources =
            [historical_k1_excess(100_000, false), historical_k1_excess(300_000, true)];
        if reverse {
            sources.reverse();
        }
        let mut d = complete(FederalStatus::Single);
        for data in sources {
            add(&mut d, income(FederalIncome::Scorp { data: scorp_source(data) }));
        }
        let r = supported(&d);
        assert_eq!(value(&r, "excess_business_loss_carryforward"), 544_000);
        assert_eq!(value(&r, "qbi_loss_carryforward"), 128_000);
        assert_eq!(value(&r, "nol_carryforward"), 740_000);
        let totals = (r.tax, r.refund, value(&r, "qbi_loss_carryforward"));
        if let Some(previous) = baseline {
            assert_eq!(totals, previous);
        }
        baseline = Some(totals);
    }
}
#[test]
fn written_note_repayment_uses_shared_capital_return_calculations() {
    let mut tax = vec![];
    for term in [GainTerm::Short, GainTerm::Long] {
        let source = repaid_written_note(term);
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
        let mut comparison = d.clone();
        add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
        add(&mut comparison, income(investment_sale_gain(term, 2_000, 2_000)));
        let r = supported(&d);
        let expected = supported(&comparison);
        assert_eq!(r.return_lines, expected.return_lines);
        assert_eq!(value(&r, "agi"), 302_000);
        assert_eq!(value(&r, "capital_gain"), 2_000);
        assert_eq!(value(&r, "net_investment_income_tax"), 76);
        assert_eq!(value(&r, "scorp_ordinary_loan_gain"), 0);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(value(&r, "qbi_deduction"), 0);
        k1_eic_investment_note(&r, 2_000);
        assert_eq!(return_line(&r, "8"), 0);
        assert_eq!(restart(d.clone()).draft, d);
        tax.push(r.tax.unwrap());
    }
    assert!(tax[1] < tax[0]);
}
#[test]
fn repaid_written_note_has_a_holding_period_and_open_accounts_cannot_claim_one() {
    // Full basis: no gain, but the repaid note still states its term.
    let mut source = repaid_written_note(GainTerm::Long);
    source.loans[0].basis = source.loans[0].principal;
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source.clone()) }));
    assert_eq!(value(&supported(&d), "capital_gain"), 0);
    source.loans[0].repayment_term = ScorpNoteHolding::NoRepayment;
    assert!(!source.valid());
    for kind in [ScorpLoanKind::OpenAccount, ScorpLoanKind::FormerOpenAccount] {
        let mut source = scorp_repaid_reduced_open_account(kind);
        source.loans[0].repayment_term = ScorpNoteHolding::Single { term: GainTerm::Long };
        assert!(!source.valid());
    }
}
#[test]
fn dated_note_repayments_split_regular_and_amt_gains_without_splitting_the_note() {
    let mut source = mixed_note_repayments();
    source.amt.as_mut().unwrap().loans[0].basis = 800_000;
    source.amt.as_mut().unwrap().risk.start =
        ScorpRiskStart::Detailed { capacity: 800_000, profit: 0, loss: 0 };
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    let mut comparison = d.clone();
    let id = d.next_id;
    add(&mut d, income(FederalIncome::Scorp { data: scorp_source(source) }));
    add(&mut comparison, income(investment_sale_gain(GainTerm::Short, 800, 400)));
    add(&mut comparison, income(investment_sale_gain(GainTerm::Long, 1_200, 600)));
    let r = supported(&d);
    assert_eq!(r.return_lines, supported(&comparison).return_lines);
    for (suffix, amount) in
        [("short_gain", 800), ("long_gain", 1_200), ("short_amt_gain", 400), ("long_amt_gain", 600)]
    {
        assert_eq!(value(&r, &alloc::format!("scorp_{id}_loan_{suffix}")), amount);
    }
    assert_eq!(value(&r, "capital_gain"), 2_000);
    assert_eq!(value(&r, "amt_capital_gain"), 1_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 76);
    scorp_note(&r, id, "loan 1 basis", 3_000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_niit_difference_applies_to_property_and_portfolio_gain_returns() {
    for property in [false, true] {
        let mut draft = if property {
            complete_property_gain_draft(false, 0)
        } else {
            complete(FederalStatus::Single)
        };
        draft.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
        if !property {
            add(
                &mut draft,
                income(sale_with_amt(ordinary_investment_sale(20000, 0, GainTerm::Long), 0, 0)),
            );
        }
        draft.screening.as_mut().unwrap().capital_history = vec![niit_loss_history(Some(7000))];
        let r = supported(&draft);
        // Regular carry-in $14,000 includes $7,000 excluded from NIIT.
        // $20,000 current gain gives $6,000 regular capital, $13,000 NII.
        assert_eq!(return_line(&r, "7a"), 600000);
        assert_eq!(value(&r, "niit_disposition_adjustment"), 7000);
        assert_eq!(value(&r, "niit_capital_carry_adjustment"), 0);
        assert_eq!(value(&r, "net_investment_income_tax"), 494);
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn unused_prior_niit_difference_remains_with_the_capital_loss_carryover() {
    let mut draft = complete(FederalStatus::Single);
    draft.screening.as_mut().unwrap().capital_history = vec![niit_loss_history(Some(7000))];
    let r = supported(&draft);
    assert_eq!(return_line(&r, "7a"), -300000);
    assert_eq!(value(&r, "niit_disposition_adjustment"), 0);
    assert_eq!(value(&r, "niit_capital_carry_adjustment"), 7000);
    // Prior/current excluded losses combine BEFORE the deferral limit.
    assert_eq!(crate::capital::niit_adjustment(7000, 10000, -24000, -3000), [0, 17000]);
    assert_eq!(crate::capital::niit_adjustment(7000, 10000, -10000, -3000), [10000, 7000]);
}

#[test]
fn historical_property_excess_loss_does_not_rephase_prior_sstb_loss() {
    let mut draft = historical_property_excess_draft();
    draft.income[0] = wage(1, Owner::Taxpayer, 51135000, 0);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.qbi_statement.as_mut().unwrap().service = true;
    drop(data);
    let r = supported(&draft);
    // Before-QBI taxable income $239,250: current SSTB loss is 50%
    // qualified ($100k); historical eligibility stays $50k. $150k * .64.
    assert_eq!(value(&r, "taxable_income"), 239250);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 96000);
    assert!(r.review_notes.iter().any(|n| n.starts_with(
        "K-1 excess-loss character: NOL carryforward $144000; qualified portion $54000."
    )));
    assert_eq!(value(&r, "nol_carryforward"), 144000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 44000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 256000);
    // 2026 Single brackets: 1,240 + 4,560 + 12,166 + 23,058 + 11,992.
    assert_eq!(value(&r, "regular_income_tax"), 53016);
    assert_eq!(return_line(&r, "16"), 5301600);
    assert_eq!(r.tax, Some(5581800)); // Includes $2,802 Additional Medicare tax.
}

#[test]
fn historical_property_excess_loss_excludes_still_passive_history() {
    let mut draft = historical_property_excess_draft();
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.section1231.as_mut().unwrap().passive_loss = 5000000;
    data.property_qbi.losses.passive = vec![loss_year(2023, 50000, 25000, 50000, 25000)];
    data.amt.as_mut().unwrap().section1231.as_mut().unwrap().passive_loss = 3000000;
    drop(data);
    let r = supported(&draft);
    // There is no positive activity income to release these old passive
    // amounts. They cannot enlarge either the excess loss or current QBI.
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 144000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 44000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 160000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 256000);
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("property passive suspension") && n.contains("$50000"))
    );
    assert_eq!(r.tax, Some(0));
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn historical_property_excess_loss_can_have_no_current_source_loss() {
    let mut draft = property_excess_draft(0, 0, 500000, 0, FederalStatus::Single);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    let property = data.section1231.as_mut().unwrap();
    property.basis_loss = 20000000;
    property.risk_loss = 20000000;
    let facts = &mut data.property_qbi;
    facts.losses.history = vec![loss_year(2020, 200000, 200000, 200000, 200000)];
    facts.losses.at_risk = vec![loss_year(2017, 200000, 0, 200000, 0)];
    data.amt = Some(amt_records(&data));
    drop(data);
    let r = supported(&draft);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 128000);
    assert_eq!(value(&r, "nol_carryforward"), 340000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 256000);
    assert_eq!(r.tax, Some(0));
    // Missing historical qualification must still prevent final totals.
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.property_qbi.losses.at_risk = vec![];
    drop(data);
    let r = estimate(&draft);
    assert!(!r.issues.is_empty());
    assert!(r.tax.is_none());
}

#[test]
fn multiple_property_excess_losses_preserve_historical_and_current_sources() {
    let mut draft = historical_property_excess_draft();
    let mut other = property_excess_draft(200000, 100000, 300000, 50000, FederalStatus::Single);
    add(&mut draft, other.income.remove(1));
    let r = supported(&draft);
    // First source: $400k loss, $250k qualified, including old limits.
    // Second: $200k loss, $50k qualified. EBL $344k; qualified half $172k.
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 344000);
    // 2026-10-19: exact-cent EBL allocation retains qualified activity balances
    // 8,533,333c + 2,133,333c + 2,133,334c. Each activity reports once as
    // $85,333 + $21,333 + $21,333; the existing annual owner sums those rows.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 127999);
    assert_eq!(value(&r, "nol_carryforward"), 540000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 144000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 340000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 256000);
    assert!(r.review_notes.iter().any(|n| n.starts_with(
        "K-1 excess-loss character: NOL carryforward $344000; qualified portion $172000."
    )));
    assert_eq!(r.tax, Some(0));
    // An explicit zero source in one tax system contributes no pool record.
    let FederalIncome::Scorp { data } = &mut draft.income[2].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = 0;
    drop(data);
    let r = supported(&draft);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 44000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 240000);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn operating_loss_property_gain_does_not_guess_historical_loss_character() {
    let mut draft = operating_loss_property_gain_draft(400000, 0, 100000);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.section1231.as_mut().unwrap().basis_loss = 1000000;
    data.property_qbi.losses.history = vec![loss_year(2025, 10000, 10000, 10000, 10000)];
    drop(data);
    unsupported(&draft, "Property excess business loss");
    let mut draft = operating_loss_property_gain_draft(400000, 0, 100000);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.amt.as_mut().unwrap().adjustment = 0;
    data.amt.as_mut().unwrap().section1231.as_mut().unwrap().passive_loss = 1000000;
    drop(data);
    unsupported(&draft, "Property excess business loss");
}

#[test]
fn historical_property_loss_with_profit_preserves_vintages_and_gain() {
    let mut draft = historical_property_excess_draft();
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.ordinary = 10000000;
    data.qbi_statement.as_mut().unwrap().income = 10000000;
    drop(data);
    // The existing independent AMT property sources total $300,000.
    let r = supported(&draft);
    // Qualified property loss $250k, qualified profit $100k; 44/400 of
    // $250k is deferred, leaving a $122,500 net qualified loss.
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 44000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 122500);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 356000);
    assert_eq!(value(&r, "nol_carryforward"), 240000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 140000);
    assert_eq!(return_line(&r, "8"), -25600000);
    assert_eq!(r.tax, Some(0));
    assert_eq!(restart(draft.clone()).draft, draft);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.section1231.as_mut().unwrap().passive_loss = 5000000;
    data.property_qbi.losses.passive = vec![loss_year(2023, 50000, 25000, 50000, 25000)];
    data.amt.as_mut().unwrap().section1231.as_mut().unwrap().passive_loss = 3000000;
    drop(data);
    let r = supported(&draft);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 122500);
    assert_eq!(value(&r, "nol_carryforward"), 240000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 140000);
    draft.income[0] = wage(1, Owner::Taxpayer, 51135000, 0);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.qbi_statement.as_mut().unwrap().service = true;
    drop(data);
    let r = supported(&draft);
    // Current loss $100k plus historical $50k, profit $50k; defer $16,500.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 83500);
    assert_eq!(value(&r, "regular_income_tax"), 53016);
    assert_eq!(value(&r, "alternative_minimum_tax"), 4509);
    assert_eq!(r.tax, Some(6032700));
}

#[test]
fn historical_operating_loss_with_property_gain_preserves_recapture() {
    let mut draft = operating_loss_property_gain_draft(200000, 20000, 100000);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.ordinary = -20000000;
    data.prior_loss = 10000000;
    data.risk.prior_loss = 10000000;
    let facts = &mut data.loss_qbi;
    facts.history = vec![loss_year(2017, 100000, 0, 100000, 0)];
    facts.at_risk = vec![loss_year(2020, 100000, 50000, 100000, 50000)];
    data.amt.as_mut().unwrap().adjustment = -10000000;
    drop(data);
    let r = supported(&draft);
    // $250k qualified operating loss, $20k qualified recapture, $27,500 deferred.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 202500);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 44000);
    assert_eq!(value(&r, "nol_carryforward"), 240000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 190000);
    assert_eq!(value(&r, "section1231_current_loss_history"), 0);
    assert_eq!(return_line(&r, "7a"), 8000000);
    assert_eq!(return_line(&r, "8"), -33600000);
    assert_eq!(r.tax, Some(0));
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn historical_operating_loss_with_gain_keeps_passive_losses_suspended() {
    let mut draft = operating_loss_property_gain_draft(0, 20000, 100000);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.ordinary = 0;
    data.prior_loss = 20000000;
    data.risk.prior_loss = 20000000;
    data.passive.loss = 5000000;
    let facts = &mut data.loss_qbi;
    facts.history = vec![loss_year(2020, 200000, 200000, 200000, 200000)];
    facts.at_risk = vec![loss_year(2017, 200000, 0, 200000, 0)];
    facts.passive = vec![loss_year(2023, 50000, 25000, 50000, 25000)];
    data.amt = Some(amt_records(&data));
    drop(data);
    let r = supported(&draft);
    // No current operating loss; old basis/risk loss $400k releases, old
    // passive loss stays suspended. Current qualified recapture stays $20k.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 158000);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 44000);
    assert_eq!(value(&r, "nol_carryforward"), 240000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 240000);
    assert_eq!(value(&r, "section1231_current_loss_history"), 0);
    assert_eq!(r.tax, Some(0));
    assert_eq!(restart(draft.clone()).draft, draft);
    // Omitting old qualification is not an implicit zero.
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    let mut data = edit_scorp(data);
    data.loss_qbi.at_risk = vec![];
    drop(data);
    let r = estimate(&draft);
    assert!(!r.issues.is_empty());
    assert!(r.tax.is_none());
}

#[test]
fn reservist_repayment_of_current_distribution_does_not_reverse_its_income() {
    let mut d = reservist_basis_return(20000, 80000);
    let repayment = &mut d.adjustments.as_mut().unwrap().ira_basis[0].reservist_repayments[0];
    repayment.ordered = 20260101;
    repayment.term = ReserveTerm::Indefinite;
    repayment.released = 20260630;
    repayment.distributed = 20260601;
    repayment.gross = 2000000;
    repayment.payments[0].date = 20260701;
    add(
        &mut d,
        basis_distribution(
            Owner::Taxpayer,
            20000,
            RetirementTreatment::Exception {
                second: None,
                distributed: 20260601,
                facts: RetirementException::Reservist {
                    ordered: 20260101,
                    through: 20260630,
                    term: ReserveTerm::Indefinite,
                    eligible: true,
                },
                early_simple: false,
            },
        ),
    );
    let r = supported(&d);
    assert_eq!(return_line(&r, "4a"), 2000000);
    assert_eq!(return_line(&r, "4b"), 1600000);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(r.tax, Some(2137400));
}

#[test]
fn reservist_repayment_checks_release_deadline_prior_use_and_source_confirmation() {
    let base = reservist_basis_return(3000, 3000);
    for (release, date, valid) in [
        (20240131, 20260131, true),
        (20240131, 20260201, false),
        (20240229, 20260228, true),
        (20240229, 20260301, false),
        (20260115, 20260115, false),
        (20260115, 20260116, true),
    ] {
        let mut d = base.clone();
        let p = &mut d.adjustments.as_mut().unwrap().ira_basis[0].reservist_repayments[0];
        p.released = release;
        p.distributed = 20240115;
        p.payments[0].date = date;
        if valid {
            supported(&d);
        } else {
            unsupported(&d, "Reservist IRA repayment");
        }
    }
    for mode in 0..4 {
        let mut d = base.clone();
        let p = &mut d.adjustments.as_mut().unwrap().ira_basis[0].reservist_repayments[0];
        match mode {
            0 => p.prior = 2800000,
            1 => p.term = ReserveTerm::Days { days: 179 },
            2 => p.distributed = 20250101,
            _ => p.ordered = 20010911,
        }
        unsupported(&d, "Reservist IRA repayment");
    }
}

#[test]
fn automatic_enrollment_saver_prior_year_deferrals_are_current_distribution() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 2300000, 0);
    add(&mut d, automatic_enrollment_income(Owner::Taxpayer, 2100, true));
    // New 2026 contributions are independent of the 2025 deferrals withdrawn.
    d.credits.as_mut().unwrap().saver = Some(saver(300000, 0)).into();
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 25100);
    assert_eq!(value(&r, "saver_credit"), 180);
    assert_eq!(r.tax, Some(72300));
}

#[test]
fn repayment_comparison_credit_wins_without_changing_saved_choice() {
    let d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_tax"), 7955);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 6855);
    assert_eq!(value(&r, "repayment_credit"), 1100);
    assert_eq!(value(&r, "repayment_itemized_amount"), 0);
    assert_eq!(r.tax, Some(502300));
    assert_eq!(r.refundable_credits, Some(110000));
    assert_eq!(return_line(&r, "31"), 110000);
    let restored = restart(d.clone()).draft;
    assert_eq!(
        restored.deductions.as_ref().unwrap().repayment.method(),
        Some(RepaymentMethod::Automatic)
    );
    assert_eq!(supported(&restored).refund, r.refund);
}

#[test]
fn repayment_comparison_deduction_wins_at_higher_current_rate() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    d.income[0] = wage(1, Owner::Taxpayer, 20000000, 0);
    fixture_mortgage_interest(d.deductions.as_mut().unwrap(), 2500000);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_itemized_amount"), 5000);
    assert_eq!(value(&r, "repayment_credit"), 0);
    assert_eq!(r.tax, Some(3339800));
    d.deductions.as_mut().unwrap().repayment.select(RepaymentMethod::Credit);
    let c = supported(&d);
    assert_eq!(c.tax, Some(3459800));
    assert_eq!(c.refundable_credits, Some(110000));
    d.deductions.as_mut().unwrap().repayment.select(RepaymentMethod::Deduction);
    assert_eq!(supported(&d).tax, r.tax);
}

#[test]
fn repayment_comparison_credit_can_exceed_current_tax() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    d.income.remove(0);
    let r = supported(&d);
    assert_eq!(r.tax, Some(0));
    assert_eq!(r.refundable_credits, Some(110000));
    assert_eq!(r.refund, Some(110000));
    assert_eq!(return_line(&r, "31"), 110000);
}

#[test]
fn repayment_comparison_requires_reconciled_complete_history() {
    for mode in 0..7 {
        let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
        let RepaymentClaim::Claim { years, confirmed, .. } =
            &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        match mode {
            0 => years[0].income_tax += 100,
            1 => years[0].year = 2024,
            2 => years.push(years[0].clone()),
            3 => years[0].benefits = 499999,
            4 => *confirmed = false,
            5 => years[0].wages = 1000000,
            _ => years[0].standard += 100,
        }
        if mode == 4 {
            // Claim of right not confirmed: no deduction or credit.
            let r = supported(&d);
            assert!(noted(&r, "WARN_REPAYMENT_LEFT_OUT"), "{r:?}");
            assert_eq!(value(&r, "repayment_credit"), 0);
            continue;
        }
        unsupported(&d, "Repayment comparison");
    }
}

#[test]
fn repayment_comparison_aggregates_matching_payments_once() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let FederalIncome::GovernmentPayment { data } = &mut d.income[1].income else { panic!() };
    data.repayments[0].amount = 250000;
    add(&mut d, prior_unemployment_repayment(250000, Owner::Taxpayer));
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_credit"), 1100);
    d.income.reverse();
    assert_eq!(supported(&d).refund, r.refund);
    let mut missing = prior_unemployment_repayment(100, Owner::Taxpayer);
    let FederalIncome::GovernmentPayment { data } = &mut missing.income else { panic!() };
    data.repayments[0].received_year = 2024;
    add(&mut d, missing);
    unsupported(&d, "Repayment comparison");
}

#[test]
fn repayment_comparison_automatic_does_not_hide_uncomputed_loss_route() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    d.income.remove(0);
    unsupported(&d, "Repayment comparison");
    d.deductions.as_mut().unwrap().repayment.select(RepaymentMethod::Credit);
    assert_eq!(supported(&d).refund, Some(110000));
}

#[test]
fn repayment_comparison_restored_history_validates_source_ranges() {
    {
        let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        *years = vec![years[0].clone(); 7];
        assert!(!d.valid_stored());
    }
}

#[test]
fn repayment_senior_recomputes_2025_phaseout_in_both_tax_methods() {
    let mut d = repayment_senior_draft(2025);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 7500000;
    years[0].standard = 1775000;
    years[0].income_tax = 969300;
    years[0].ssns = RepaymentSsns::Taxpayer;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_senior_deduction"), 5100);
    assert_eq!(value(&r, "repayment_year_0_refigured_senior_deduction"), 5400);
    assert_eq!(value(&r, "repayment_year_0_original_tax"), 9693);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 8527);
    assert_eq!(value(&r, "repayment_credit"), 1166);
    assert_eq!(r.tax, Some(405700));
    assert_eq!(r.refund, Some(360900));
    assert_eq!(supported(&restart(d).draft).refund, r.refund);
}

#[test]
fn repayment_senior_reconciles_original_elderly_credit_after_tax_cap() {
    let mut d = repayment_senior_draft(2023);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 1000000;
    years[0].benefits = 700000;
    years[0].standard = 1570000;
    years[0].income_tax = 9300;
    years[0].elderly = RepaymentElderly::Facts { exempt_pensions: vec![] };
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_elderly_credit"), 38);
    assert_eq!(value(&r, "repayment_year_0_refigured_elderly_credit"), 0);
    assert_eq!(value(&r, "repayment_credit"), 93);
}

#[test]
fn repayment_senior_historical_age_uses_day_before_birthday() {
    for (birth, standard, original, deduction, credit) in
        [(19610101, 17750, 6195, 6000, 880), (19610102, 15750, 7955, 0, 1100)]
    {
        let mut d = repayment_senior_draft(2025);
        d.people.as_mut().unwrap().taxpayer.birth_date = birth;
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        years[0].standard = standard * 100;
        years[0].income_tax = original * 100;
        years[0].ssns = RepaymentSsns::Taxpayer;
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_senior_deduction"), deduction);
        assert_eq!(value(&r, "repayment_credit"), credit);
    }
}

#[test]
fn repayment_social_includes_exempt_interest_only_in_provisional_income() {
    let mut d = repayment_social_draft(2025, 17750, 428);
    let years = repayment_years_mut(&mut d);
    years[0].exempt_interest = 100000;
    let RepaymentSocial::Benefits { filed_taxable, .. } = &mut years[0].social else { panic!() };
    *filed_taxable = 300000;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_taxable_social"), 3000);
    assert_eq!(value(&r, "repayment_year_0_refigured_taxable_social"), 500);
    assert_eq!(value(&r, "repayment_year_0_original_agi"), 28000);
    assert_eq!(value(&r, "repayment_year_0_refigured_agi"), 20500);
    assert_eq!(value(&r, "repayment_credit"), 428);
}

#[test]
fn repayment_social_coordinates_upper_taxability_and_senior_phaseout() {
    for (
        wages,
        original_social,
        refigured_social,
        original,
        after,
        senior_before,
        senior_after,
        credit,
    ) in [
        (25000, 13850, 9600, 2777, 1667, 6000, 6000, 1110),
        (50000, 17000, 17000, 6657, 5555, 5880, 6000, 1102),
    ] {
        let mut d = repayment_social_draft(2025, 17750, original);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        years[0].wages = wages * 100;
        years[0].social = RepaymentSocial::Benefits {
            amount: 2000000,
            filed_taxable: original_social * 100,
            ordinary: true,
        };
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_refigured_taxable_social"), refigured_social);
        assert_eq!(value(&r, "repayment_year_0_original_senior_deduction"), senior_before);
        assert_eq!(value(&r, "repayment_year_0_refigured_senior_deduction"), senior_after);
        assert_eq!(value(&r, "repayment_year_0_refigured_tax"), after);
        assert_eq!(value(&r, "repayment_credit"), credit);
    }
}

#[test]
fn repayment_social_rejects_incomplete_or_unreconciled_records() {
    for mode in 0..3 {
        let mut d = repayment_social_draft(2025, 17750, 378);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        let RepaymentSocial::Benefits { filed_taxable, ordinary, .. } = &mut years[0].social else {
            panic!()
        };
        if mode == 0 {
            *filed_taxable += 100;
            unsupported(&d, "Repayment comparison");
            continue;
        }
        // Unsupported Social Security records: only the deduction.
        *ordinary = false;
        let r = supported(&d);
        assert!(noted(&r, "WARN_REPAYMENT_CREDIT_LEFT_OUT"), "{r:?}");
        assert_eq!(value(&r, "repayment_credit"), 0);
    }
    {
        let mut d = repayment_social_draft(2025, 17750, 378);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        let RepaymentSocial::Benefits { amount, filed_taxable, .. } = &mut years[0].social else {
            panic!()
        };
        *filed_taxable = *amount + 1;
        assert!(!d.valid_stored());
    }
}

#[test]
fn repayment_early_unchanged_tax_cancels_without_creating_credit() {
    for (simple, tax) in [(false, 1000), (true, 2500)] {
        let d = repayment_early_draft(simple, false, 0, tax);
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_early_tax"), tax);
        assert_eq!(value(&r, "repayment_year_0_refigured_early_tax"), tax);
        assert_eq!(value(&r, "repayment_credit"), 1100);
    }
}

#[test]
fn repayment_early_rejects_missing_or_mismatched_source_records() {
    for mode in 0..7 {
        let mut d = repayment_early_draft(false, true, 6625, 900);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        match mode {
            0 => years[0].early = RepaymentEarly::None,
            1 => {
                if let RepaymentEarly::Facts { filed_tax, .. } = &mut years[0].early {
                    *filed_tax += 100;
                }
            }
            2 | 3 => {
                // Unsupported Form 5329 records: only the deduction.
                if let RepaymentEarly::Facts { confirmed, .. } = &mut years[0].early {
                    *confirmed = false;
                }
                let r = supported(&d);
                assert!(noted(&r, "WARN_REPAYMENT_CREDIT_LEFT_OUT"), "{r:?}");
                continue;
            }
            4 => {
                if let RepaymentRetirement::TraditionalIra { treatment, .. } =
                    &mut years[0].retirement[0]
                {
                    *treatment = RepaymentDistribution::Early {
                        owner: Owner::Spouse,
                        simple: false,
                        medical: true,
                    };
                }
            }
            5 => {
                years[0].retirement[0] = RepaymentRetirement::Pension {
                    gross: 1000000,
                    taxable: 1000000,
                    normal: true,
                    treatment: RepaymentDistribution::Early {
                        owner: Owner::Taxpayer,
                        simple: true,
                        medical: true,
                    },
                }
            }
            _ => {
                if let RepaymentRetirement::TraditionalIra { treatment, .. } =
                    &mut years[0].retirement[0]
                {
                    *treatment = RepaymentDistribution::Early {
                        owner: Owner::Taxpayer,
                        simple: false,
                        medical: false,
                    };
                }
            }
        }
        unsupported(&d, "Repayment early distribution");
    }
}

#[test]
fn repayment_historical_amt_uses_each_year_and_reaches_current_refund() {
    for (year, standard, original, revised) in
        [(2023, 13850, 24362, 23062), (2024, 14600, 23218, 21918), (2025, 15750, 22594, 21294)]
    {
        let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
        for e in &mut d.income {
            if let FederalIncome::GovernmentPayment { data } = &mut e.income {
                data.repayments[0].received_year = year;
            }
        }
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        years[0].year = year;
        years[0].standard = standard * 100;
        years[0].income_tax = original * 100;
        years[0].exempt_interest = 10000000;
        years[0].private_activity_interest = 10000000;
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_amti"), 175000);
        assert_eq!(value(&r, "repayment_year_0_refigured_amti"), 170000);
        assert_eq!(value(&r, "repayment_year_0_original_tentative_minimum_tax"), original);
        assert_eq!(value(&r, "repayment_year_0_refigured_tax"), revised);
        assert_eq!(value(&r, "repayment_credit"), 1300);
        assert_eq!(r.tax, Some(502300));
        assert_eq!(r.refund, Some(277700));
        assert_eq!(supported(&restart(d).draft).refund, r.refund);
    }
}

#[test]
fn repayment_historical_amt_can_stop_binding_after_exclusion() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    years[0].exempt_interest = 4400000;
    years[0].private_activity_interest = 4400000;
    years[0].income_tax = 803400;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_amt"), 79);
    assert_eq!(value(&r, "repayment_year_0_refigured_amt"), 0);
    assert_eq!(value(&r, "repayment_credit"), 1179);
}

#[test]
fn repayment_high_income_uses_rate_schedule_without_niit_credit() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 0;
    years[0].interest = 20000000;
    years[0].income_tax = 4082300;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_agi"), 215000);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 39467);
    // A hypothetical NIIT decrease of 190 is outside Chapter1 and is NOT added.
    assert_eq!(value(&r, "repayment_credit"), 1356);
    assert_eq!(r.refund, Some(283300));
}

#[test]
fn repayment_historical_amt_adds_back_enhanced_senior_deduction() {
    let mut d = repayment_senior_draft(2025);
    let years = repayment_years_mut(&mut d);
    years[0].standard = 1775000;
    years[0].ssns = RepaymentSsns::Taxpayer;
    years[0].exempt_interest = 10000000;
    years[0].private_activity_interest = 10000000;
    years[0].income_tax = 2259400;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_senior_deduction"), 6000);
    assert_eq!(value(&r, "repayment_year_0_original_amti"), 175000);
    assert_eq!(value(&r, "repayment_credit"), 1300);
}

#[test]
fn repayment_historical_pab_must_be_part_of_exempt_interest() {
    {
        let amount = 100001;
        let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        years[0].exempt_interest = 100000;
        years[0].private_activity_interest = amount;
        assert!(!d.valid_stored());
    }
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    years[0].exempt_interest = 10000000;
    years[0].private_activity_interest = 10000000;
    unsupported(&d, "Repayment comparison"); // Original filed tax omitted AMT.
}

#[test]
fn repayment_dividends_recomputes_zero_rate_room_in_each_year() {
    for (year, standard, original, revised, credit) in [
        (2023, 13850, 5700, 4350, 1350),
        (2024, 14600, 5125, 4019, 1106),
        (2025, 15750, 4610, 3875, 735),
    ] {
        let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
        for e in &mut d.income {
            if let FederalIncome::GovernmentPayment { data } = &mut e.income {
                data.repayments[0].received_year = year;
            }
        }
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        years[0].year = year;
        years[0].standard = standard * 100;
        years[0].wages = 4000000;
        years[0].income_tax = original * 100;
        years[0].dividends = historical_dividends(10000, 10000, 0, 0, 0);
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_agi"), 65000);
        assert_eq!(value(&r, "repayment_year_0_refigured_tax"), revised);
        assert_eq!(value(&r, "repayment_credit"), credit);
        assert_eq!(return_line(&r, "31"), credit * 100);
        assert_eq!(r.refund, Some((1477 + credit) * 100));
        assert_eq!(supported(&restart(d).draft).refund, r.refund);
    }
}

#[test]
fn repayment_dividends_mixed_ordinary_and_qualified_are_not_double_counted() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 5000000;
    years[0].income_tax = 753500;
    years[0].dividends = historical_dividends(10000, 6000, 0, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_agi"), 75000);
    assert_eq!(value(&r, "repayment_year_0_ordinary_dividends"), 10000);
    assert_eq!(value(&r, "repayment_year_0_qualified_dividends"), 6000);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 6440);
    assert_eq!(value(&r, "repayment_credit"), 1095);
}

#[test]
fn repayment_dividends_amt_uses_regular_ordinary_band_occupancy() {
    for (year, standard, original, revised) in
        [(2023, 13850, 21641, 19591), (2024, 14600, 20024, 17974), (2025, 15750, 19029, 16979)]
    {
        let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
        for e in &mut d.income {
            if let FederalIncome::GovernmentPayment { data } = &mut e.income {
                data.repayments[0].received_year = year;
            }
        }
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        years[0].year = year;
        years[0].standard = standard * 100;
        years[0].wages = 4000000;
        years[0].income_tax = original * 100;
        years[0].exempt_interest = 10000000;
        years[0].private_activity_interest = 10000000;
        years[0].dividends = historical_dividends(10000, 10000, 10000, 0, 0);
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_amti"), 175000);
        assert_eq!(value(&r, "repayment_year_0_original_tentative_minimum_tax"), original);
        assert_eq!(value(&r, "repayment_year_0_refigured_tentative_minimum_tax"), revised);
        assert_eq!(value(&r, "repayment_credit"), 2050);
    }
}

#[test]
fn repayment_dividends_keeps_25_and_28_percent_source_components() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 20000000;
    years[0].income_tax = 5262300;
    years[0].dividends = historical_dividends(0, 0, 50000, 30000, 10000);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_agi"), 265000);
    assert_eq!(value(&r, "repayment_year_0_unrecaptured_gain"), 30000);
    assert_eq!(value(&r, "repayment_year_0_collectibles_gain"), 10000);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 51237);
    assert_eq!(value(&r, "repayment_credit"), 1386);
}

#[test]
fn repayment_dividends_reallocates_fifteen_and_twenty_percent_bands() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 53000000;
    years[0].income_tax = 15857700;
    years[0].dividends = historical_dividends(20000, 20000, 0, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 156577);
    assert_eq!(value(&r, "repayment_credit"), 2000);
}

#[test]
fn repayment_dividends_feed_social_taxability_and_senior_calculations() {
    let mut d = repayment_social_draft(2025, 17750, 378);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 0;
    years[0].dividends = historical_dividends(15000, 0, 0, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_taxable_social"), 2500);
    assert_eq!(value(&r, "repayment_year_0_refigured_taxable_social"), 0);
    assert_eq!(value(&r, "repayment_year_0_original_agi"), 27500);
    assert_eq!(value(&r, "repayment_credit"), 378);
}

#[test]
fn repayment_dividends_validate_source_components_and_confirmation() {
    for (ordinary, qualified, gains, unrecaptured, collectibles) in
        [(0, 1, 0, 0, 0), (0, 0, 100, 101, 0), (0, 0, 100, 50, 51), (1000000001, 0, 0, 0, 0)]
    {
        let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        years[0].dividends =
            historical_dividends(ordinary, qualified, gains, unrecaptured, collectibles);
        assert!(!d.valid_stored());
    }
    // Unsupported original-year statements: only the deduction is figured.
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    years[0].dividends = RepaymentDividends::Statements {
        ordinary: 0,
        qualified: 0,
        gains: 0,
        unrecaptured: 0,
        collectibles: 0,
        confirmed: false,
    };
    let r = supported(&d);
    assert!(noted(&r, "WARN_REPAYMENT_CREDIT_LEFT_OUT"), "{r:?}");
    assert_eq!(value(&r, "repayment_credit"), 0);
}

#[test]
fn repayment_capital_sales_keep_short_and_long_character() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 4000000;
    years[0].income_tax = 945500;
    years[0].capital = historical_capital(
        vec![
            historical_sale(GainTerm::Short, 25000, 5000),
            historical_sale(GainTerm::Long, 17000, 7000),
        ],
        0,
        0,
    );
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_net_short_capital"), 20000);
    assert_eq!(value(&r, "repayment_year_0_net_long_capital"), 10000);
    assert_eq!(value(&r, "repayment_year_0_original_agi"), 85000);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 8355);
    assert_eq!(value(&r, "repayment_credit"), 1100);
    assert_eq!(supported(&restart(d).draft).refund, r.refund);
}

#[test]
fn repayment_capital_nets_wash_adjusted_losses_without_current_withholding() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 4000000;
    years[0].income_tax = 447500;
    let mut short = historical_sale(GainTerm::Short, 1000, 7000);
    short.use_kind = SaleUse::Investment { wash_sale: 100000 };
    short.withholding = 1000000;
    years[0].capital =
        historical_capital(vec![short, historical_sale(GainTerm::Long, 15000, 5000)], 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_net_short_capital"), -5000);
    assert_eq!(value(&r, "repayment_year_0_capital_income"), 5000);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 3875);
    assert_eq!(value(&r, "repayment_credit"), 600);
    assert_eq!(r.payments, Some(650000));
    assert_eq!(r.refund, Some(207700));
}

#[test]
fn repayment_capital_loss_limits_and_unchanged_carryovers() {
    for (status, original, short_carry, loss) in
        [(FederalStatus::Single, 7295, 8000, -3000), (FederalStatus::Separate, 7625, 9500, -1500)]
    {
        let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        years[0].status = status;
        years[0].income_tax = original * 100;
        years[0].capital = historical_capital(
            vec![
                historical_sale(GainTerm::Short, 0, 10000),
                historical_sale(GainTerm::Long, 0, 5000),
            ],
            1000,
            2000,
        );
        let mut prior = capital_history();
        prior.status = status;
        prior.agi = (75000 + loss) * 100;
        prior.deductions = 1575000;
        prior.short = -1100000;
        prior.long = -700000;
        prior.deducted_loss = -loss * 100;
        prior.investment_parts =
            Some(InvestmentCapitalParts { short: short_carry * 100, long: 700_000, amt: None });
        d.screening.as_mut().unwrap().capital_history = vec![prior];
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_capital_income"), loss);
        assert_eq!(value(&r, "repayment_year_0_original_short_carry"), short_carry);
        assert_eq!(value(&r, "repayment_year_0_refigured_short_carry"), short_carry);
        assert_eq!(value(&r, "repayment_year_0_original_long_carry"), 7000);
        assert_eq!(value(&r, "repayment_year_0_refigured_amt_long_carry"), 7000);
        assert_eq!(value(&r, "repayment_credit"), 1100);
        assert_eq!(value(&r, "capital_gain"), -3000);
        assert_eq!(r.refund, Some(293700));
    }
}

#[test]
fn repayment_capital_changed_carryovers_reuse_known_2025_source() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 0;
    years[0].benefits = 2000000;
    years[0].income_tax = 12600;
    years[0].capital = historical_capital(vec![historical_sale(GainTerm::Short, 0, 10000)], 0, 0);
    // The complete2025 worksheet supplies actual carry7,000 automatically;
    // exclusion refigures it to10,000. Both use3,000 in2026.
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_credit"), 126);
    assert_eq!(value(&r, "capital_short_carryforward"), 7000);
    assert_eq!(r.refund, Some(196300));
    assert!(d.screening.as_ref().unwrap().capital_history.is_empty());
}

#[test]
fn repayment_capital_losses_offset_collectibles_before_1250_gain() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let years = repayment_years_mut(&mut d);
    years[0].wages = 20000000;
    years[0].income_tax = 5122300;
    years[0].dividends = historical_dividends(0, 0, 50000, 30000, 10000);
    years[0].capital =
        historical_capital(vec![historical_sale(GainTerm::Short, 5000, 10000)], 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_unrecaptured_gain"), 30000);
    assert_eq!(value(&r, "repayment_year_0_collectibles_gain"), 5000);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 49837);
    assert_eq!(value(&r, "repayment_credit"), 1386);
}

#[test]
fn repayment_capital_rejects_unresolved_sources() {
    for mode in 1..3 {
        let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        let mut sale = historical_sale(GainTerm::Long, 1000, 1000);
        if mode == 1 {
            sale.special = crate::SaleSpecial::Another;
        }
        years[0].capital = historical_capital(vec![sale], 0, 0);
        if mode == 1 {
            unsupported(&d, "Repayment capital");
            continue;
        }
        // Unsupported capital records: only the deduction is figured.
        if let RepaymentCapital::Records { confirmed, .. } = &mut years[0].capital {
            *confirmed = false;
        }
        let r = supported(&d);
        assert!(noted(&r, "WARN_REPAYMENT_CREDIT_LEFT_OUT"), "{r:?}");
        assert_eq!(value(&r, "repayment_credit"), 0);
    }
    for mode in 0..3 {
        let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        let mut sale = historical_sale(GainTerm::Short, 1000, 2000);
        if mode == 0 {
            sale.use_kind = SaleUse::Investment { wash_sale: 100001 };
        }
        years[0].capital = historical_capital(
            if mode == 1 { vec![sale; 101] } else { vec![sale] },
            if mode == 2 { -1 } else { 0 },
            0,
        );
        assert!(!d.valid_stored());
    }
}

#[test]
fn repayment_carry_requires_actual_balance_reconciliation_and_consecutive_years() {
    let mut d = repayment_carry_chain();
    let years = repayment_years_mut(&mut d);
    let RepaymentCapital::Records { short_loss, .. } = &mut years[1].capital else { panic!() };
    *short_loss = 800000;
    unsupported(&d, "Repayment capital");
    let mut d = repayment_carry_chain();
    let FederalIncome::GovernmentPayment { data } = &mut d.income[1].income else { panic!() };
    data.repayments[0].received_year = 2023;
    let years = repayment_years_mut(&mut d);
    years[0].year = 2023;
    years[0].standard = 1385000;
    years[0].income_tax = 31800;
    unsupported(&d, "Repayment capital");
}

#[test]
fn repayment_restoration_preserves_future_losses_without_second_current_deduction() {
    let d = repayment_restoration_draft(RepaymentMethod::Credit);
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "repayment_return_year_decrease"), 0);
    assert_eq!(value(&r, "repayment_credit"), 126);
    assert_eq!(value(&r, "capital_short_carryforward"), 7000);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 4000);
    assert_eq!(r.tax, Some(466300));
    assert_eq!(r.refund, Some(196300));
    let restored = restart(d.clone()).draft;
    assert_eq!(restored, d);
    assert_eq!(supported(&restored).refund, r.refund);
    assert_eq!(restored.screening.unwrap().capital_history[0].agi, 1700000);
}

#[test]
fn repayment_restoration_includes_current_tax_decrease_once() {
    let mut d = repayment_restoration_draft(RepaymentMethod::Credit);
    add(&mut d, income(capital_return(10000, 0, GainTerm::Short)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 3000);
    assert_eq!(value(&r, "repayment_return_year_original_chapter_one_net_tax"), 5383);
    assert_eq!(value(&r, "repayment_return_year_refigured_chapter_one_net_tax"), 5023);
    assert_eq!(value(&r, "repayment_return_year_decrease"), 360);
    assert_eq!(value(&r, "repayment_credit"), 486);
    assert_eq!(value(&r, "capital_short_carryforward"), 0);
    assert_eq!(r.tax, Some(538300));
    assert_eq!(r.refund, Some(160300));
    assert_eq!(return_line(&r, "31"), 48600);
}

#[test]
fn repayment_restoration_excludes_niit_change_from_chapter_one_credit() {
    let mut d = repayment_restoration_draft(RepaymentMethod::Credit);
    d.income[0] = wage(1, Owner::Taxpayer, 20000000, 0);
    add(&mut d, income(capital_return(20000, 0, GainTerm::Short)));
    let r = supported(&d);
    assert_eq!(value(&r, "net_investment_income_tax"), 494);
    assert_eq!(value(&r, "repayment_return_year_decrease"), 720);
    assert_eq!(value(&r, "repayment_credit"), 846);
    // Hypothetical NIIT would fall114; it is not a Chapter1 credit.
    assert_ne!(value(&r, "repayment_credit"), 960);
}

#[test]
fn repayment_restoration_selected_method_controls_future_balances() {
    let mut d = repayment_restoration_draft(RepaymentMethod::Deduction);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_credit"), 0);
    assert_eq!(value(&r, "capital_short_carryforward"), 4000);
    d.deductions.as_mut().unwrap().repayment.select(RepaymentMethod::Automatic);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_credit"), 126);
    assert_eq!(value(&r, "capital_short_carryforward"), 7000);
    assert_eq!(d.deductions.unwrap().repayment.method(), Some(RepaymentMethod::Automatic));
}

#[test]
fn repayment_restoration_zero_actual_loss_needs_no_duplicate_source_record() {
    let mut d = repayment_restoration_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    years[0].capital = historical_capital(vec![historical_sale(GainTerm::Short, 0, 3000)], 0, 0);
    d.screening.as_mut().unwrap().capital_history.clear();
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, "repayment_credit"), 486);
    assert_eq!(value(&r, "capital_short_carryforward"), 0);
    assert_eq!(r.tax, Some(502300));
    assert_eq!(r.refund, Some(196300));
}

#[test]
fn repayment_restoration_reconciles_current_actual_loss_history() {
    let mut d = repayment_restoration_draft(RepaymentMethod::Credit);
    d.screening.as_mut().unwrap().capital_history[0].short = -1100000;
    unsupported(&d, "Repayment capital");
    let mut d = repayment_restoration_draft(RepaymentMethod::Credit);
    d.screening.as_mut().unwrap().capital_history[0].amt = Some(PriorAmtCapitalReturn {
        taxable_income: 1700000,
        short: -1100000,
        long: 0,
        deducted_loss: 300000,
    });
    d.screening.as_mut().unwrap().capital_history[0].investment_parts.as_mut().unwrap().amt =
        Some(InvestmentAmtCapitalParts { short: 800_000, long: 0 });
    unsupported(&d, "Repayment capital");
}

#[test]
fn repayment_restoration_preserves_loss_character_and_unabsorbed_low_income_balance() {
    let mut d = repayment_restoration_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    years[0].capital = historical_capital(vec![historical_sale(GainTerm::Long, 0, 10000)], 0, 0);
    let prior = &mut d.screening.as_mut().unwrap().capital_history[0];
    prior.short = 0;
    prior.long = -1000000;
    prior.investment_parts = Some(InvestmentCapitalParts { short: 0, long: 700_000, amt: None });
    let r = supported(&d);
    assert_eq!(value(&r, "capital_short_carryforward"), 0);
    assert_eq!(value(&r, "capital_long_carryforward"), 7000);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 4000);
    d.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "capital_long_carryforward"), 10000);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 7000);
    assert_eq!(r.refund, Some(12600));
}

#[test]
fn repayment_restoration_follows_three_years_before_current_comparison() {
    let mut d = repayment_restoration_draft(RepaymentMethod::Credit);
    let FederalIncome::GovernmentPayment { data } = &mut d.income[1].income else { panic!() };
    data.repayments[0].received_year = 2023;
    let years = repayment_years_mut(&mut d);
    years[0].year = 2023;
    years[0].standard = 1385000;
    years[0].income_tax = 31800;
    let mut second = years[0].clone();
    second.year = 2024;
    second.standard = 1460000;
    second.wages = 6000000;
    second.benefits = 0;
    second.income_tax = 485900;
    second.capital = historical_capital(vec![], 7000, 0);
    let mut third = second.clone();
    third.year = 2025;
    third.standard = 1575000;
    third.income_tax = 471500;
    third.capital = historical_capital(vec![], 4000, 0);
    years.extend([second, third]);
    let prior = &mut d.screening.as_mut().unwrap().capital_history[0];
    prior.agi = 5700000;
    prior.short = -400000;
    prior.investment_parts = Some(InvestmentCapitalParts { short: 100_000, long: 0, amt: None });
    add(&mut d, income(capital_return(2000, 0, GainTerm::Short)));
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_2_refigured_short_carry"), 2850);
    assert_eq!(value(&r, "repayment_return_year_decrease"), 222);
    assert_eq!(value(&r, "repayment_credit"), 540);
    assert_eq!(value(&r, "capital_short_carryforward"), 0);
    let years = repayment_years_mut(&mut d);
    years.reverse();
    assert_eq!(supported(&d).refund, r.refund);
}

#[test]
fn repayment_amt_sources_explicit_zero_differs_from_equal_balance_default() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    years[0].capital = historical_capital(vec![historical_sale(GainTerm::Short, 6000, 0)], 5000, 0);
    years[0].income_tax = 817500;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_amt_capital_income"), 1000);
    let years = repayment_years_mut(&mut d);
    let RepaymentCapital::Records { amt, .. } = &mut years[0].capital else { panic!() };
    *amt = Some(RepaymentAmtLosses { short: 0, long: 0 });
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_amt_capital_income"), 6000);
    assert_eq!(value(&r, "repayment_credit"), 1100);
}

#[test]
fn repayment_amt_sources_propagate_distinct_actual_balances_into_restoration() {
    let mut d = repayment_restoration_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    let RepaymentCapital::Records { sales, .. } = &mut years[0].capital else { panic!() };
    sales[0].sale.amt = crate::SaleAmt::Different {
        record: CapitalSaleAmt { term: None, basis: 2000000, wash_sale: 0 },
    };
    d.screening.as_mut().unwrap().capital_history[0].amt = Some(PriorAmtCapitalReturn {
        taxable_income: 1700000,
        short: -2000000,
        long: 0,
        deducted_loss: 300000,
    });
    d.screening.as_mut().unwrap().capital_history[0].investment_parts.as_mut().unwrap().amt =
        Some(InvestmentAmtCapitalParts { short: 1_700_000, long: 0 });
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_amt_short_carry"), 17000);
    assert_eq!(value(&r, "repayment_year_0_refigured_amt_short_carry"), 17000);
    assert_eq!(value(&r, "capital_short_carryforward"), 7000);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 14000);
    assert_eq!(value(&r, "repayment_credit"), 126);
}

#[test]
fn repayment_capital_reuses_unchanged_actual_loss_without_duplicate_entry() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    years[0].capital = historical_capital(vec![historical_sale(GainTerm::Short, 0, 10000)], 0, 0);
    years[0].income_tax = 729500;
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_short_carryforward"), 4000);
    assert_eq!(r.refund, Some(293700));
    assert!(d.screening.as_ref().unwrap().capital_history.is_empty());
    assert_eq!(supported(&restart(d.clone()).draft).refund, r.refund);
    let mut prior = capital_history();
    prior.agi = 7200000;
    prior.deductions = 1575000;
    prior.short = -1000000;
    prior.long = 0;
    prior.investment_parts = Some(InvestmentCapitalParts { short: 700_000, long: 0, amt: None });
    d.screening.as_mut().unwrap().capital_history = vec![prior];
    assert_eq!(supported(&d).refund, r.refund);
    d.screening.as_mut().unwrap().capital_history[0].short = -1100000;
    unsupported(&d, "Repayment capital");
}

#[test]
fn repayment_capital_does_not_invent_incoming_loss_investment_attribution() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    years[0].capital = historical_capital(vec![], 10000, 0);
    years[0].income_tax = 729500;
    unsupported(&d, "Investment-tax capital history");
}

#[test]
fn repayment_amt_sources_infer_unchanged_amt_only_loss_for_current_return() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    let mut sale = historical_sale(GainTerm::Short, 10000, 10000);
    sale.amt = crate::SaleAmt::Different {
        record: CapitalSaleAmt { term: None, basis: 2000000, wash_sale: 0 },
    };
    years[0].capital = historical_capital(vec![sale], 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, "capital_short_carryforward"), 0);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 4000);
    assert_eq!(value(&r, "repayment_credit"), 1100);
    assert_eq!(r.refund, Some(257700));
    assert!(d.screening.unwrap().capital_history.is_empty());
}

#[test]
fn repayment_minimum_credit_uses_each_preceding_year_exemption() {
    for (year, standard, tax, after, excluded) in [
        (2023, 13850, 8766, 7666, 14430),
        (2024, 14600, 8347, 7247, 13462),
        (2025, 15750, 7955, 6855, 12665),
    ] {
        let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
        let FederalIncome::GovernmentPayment { data } = &mut d.income[1].income else { panic!() };
        data.repayments[0].received_year = year;
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        years[0].year = year;
        years[0].standard = standard * 100;
        years[0].income_tax = (tax - 1000) * 100;
        years[0].minimum_tax = Some(repayment_minimum_source(year - 1));
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_minimum_exclusion_tax"), excluded);
        assert_eq!(value(&r, "repayment_year_0_minimum_credit_available"), 1000);
        assert_eq!(value(&r, "repayment_year_0_original_minimum_credit"), 1000);
        assert_eq!(value(&r, "repayment_year_0_refigured_minimum_credit"), 1000);
        assert_eq!(value(&r, "repayment_year_0_refigured_tax"), after - 1000);
        assert_eq!(value(&r, "repayment_credit"), 1100);
        assert_eq!(supported(&restart(d).draft).refund, r.refund);
    }
}

#[test]
fn repayment_minimum_credit_retains_signed_generation_before_combining_old_credit() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let mut history = repayment_minimum_source(2024);
    history.amt = 1000000;
    history.carry = 366500;
    let years = repayment_years_mut(&mut d);
    years[0].minimum_tax = Some(history);
    years[0].income_tax = 695500;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_minimum_credit_generated"), -2665);
    assert_eq!(value(&r, "repayment_year_0_minimum_credit_available"), 1000);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 5855);
    assert_eq!(value(&r, "repayment_credit"), 1100);
}

#[test]
fn repayment_minimum_credit_reuses_prior_preferential_rate_worksheet() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let mut history = repayment_minimum_source(2024);
    history.regular = 1565300;
    history.amt = 1286500;
    history.gains = Some(
        MinimumTaxGains {
            preferential: 2000000,
            combined: 2000000,
            regular_zero: 8000000,
            regular_fifteen: 8000000,
        }
        .into(),
    );
    let years = repayment_years_mut(&mut d);
    years[0].minimum_tax = Some(history);
    years[0].income_tax = 695500;
    let r = supported(&d);
    //2024 exclusion AMTI200,000:114,300 after exemption;20,000 at15%,
    //94,300 at26%=27,518; less15,653 regular tax=11,865.
    assert_eq!(value(&r, "repayment_year_0_minimum_exclusion_tax"), 11865);
    assert_eq!(value(&r, "repayment_year_0_minimum_credit_available"), 1000);
    assert_eq!(value(&r, "repayment_credit"), 1100);
}

#[test]
fn repayment_minimum_credit_restoration_uses_balance_left_only_by_refigured_return() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let mut history = repayment_minimum_source(2024);
    history.carry = 600000;
    let years = repayment_years_mut(&mut d);
    years[0].minimum_tax = Some(history);
    years[0].income_tax = 95500;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_decrease"), 955);
    assert_eq!(value(&r, "repayment_return_year_decrease"), 145);
    assert_eq!(value(&r, "repayment_credit"), 1100);
    assert_eq!(value(&r, "minimum_tax_credit"), 0);
    assert_eq!(value(&r, "regular_income_tax"), 5023);
    assert!(d.credits.as_ref().unwrap().minimum_tax.is_none());
    assert_eq!(supported(&restart(d.clone()).draft).refund, r.refund);
    assert_eq!(value(&r, "repayment_year_0_original_minimum_credit_carry"), 0);
    assert_eq!(value(&r, "repayment_year_0_refigured_minimum_credit_carry"), 145);
}

#[test]
fn repayment_minimum_credit_rejects_unconfirmed_or_invalid_history() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let mut history = repayment_minimum_source(2024);
    history.mtcnol = crate::MinimumTaxLoss::Unavailable;
    let years = repayment_years_mut(&mut d);
    years[0].minimum_tax = Some(history);
    unsupported(&d, "Repayment minimum-tax credit");
}

#[test]
fn repayment_minimum_credit_propagates_to_consecutive_historical_return() {
    let mut d = repayment_minimum_chain();
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_minimum_credit"), 8000);
    assert_eq!(value(&r, "repayment_year_0_refigured_minimum_credit"), 7247);
    assert_eq!(value(&r, "repayment_year_1_original_minimum_credit"), 0);
    assert_eq!(value(&r, "repayment_year_1_refigured_minimum_credit"), 753);
    assert_eq!(value(&r, "repayment_credit"), 1100);
    let years = repayment_years_mut(&mut d);
    years.reverse();
    assert_eq!(supported(&d).refund, r.refund);
}

#[test]
fn repayment_minimum_credit_requires_intervening_years() {
    let mut d = repayment_minimum_chain();
    let years = repayment_years_mut(&mut d);
    years.pop();
    unsupported(&d, "Repayment minimum-tax credit");
}

#[test]
fn repayment_minimum_credit_reconciles_duplicate_preceding_sources() {
    let mut d = repayment_minimum_chain();
    let years = repayment_years_mut(&mut d);
    years[1].minimum_tax = Some(MinimumTaxHistory {
        return_form: crate::MinimumTaxReturnForm::Form1040,
        status: FederalStatus::Single,
        base: 6040000,
        exclusions: vec![MinimumTaxExclusion {
            source: "Form6251 standard deduction".into(),
            amount: 1460000,
        }],
        regular: 834700,
        amt: 0,
        carry: 0,
        gains: None,
        regular_nol: 0,
        mtcnol: crate::MinimumTaxLoss::None,
        foreign_earned: None,
        foreign_credit: crate::MinimumTaxForeignCredit::None,
        old_vehicle_credit: 0,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_credit"), 1100);
    let years = repayment_years_mut(&mut d);
    years[1].minimum_tax.as_mut().unwrap().carry = 100;
    unsupported(&d, "Repayment minimum-tax credit");
}

#[test]
fn repayment_minimum_credit_reuses_actual_current_balance_and_reconciles_source() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    let mut source = repayment_minimum_source(2024);
    source.carry = 900000;
    years[0].minimum_tax = Some(source);
    years[0].income_tax = 0;
    let r = supported(&d);
    assert_eq!(value(&r, "minimum_tax_credit_available"), 2045);
    assert_eq!(value(&r, "minimum_tax_credit"), 2045);
    assert_eq!(value(&r, "repayment_return_year_decrease"), 1100);
    assert_eq!(value(&r, "repayment_credit"), 1100);
    assert!(d.credits.as_ref().unwrap().minimum_tax.is_none());
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(MinimumTaxHistory {
            return_form: crate::MinimumTaxReturnForm::Form1040,
            status: FederalStatus::Single,
            base: 5925000,
            exclusions: vec![MinimumTaxExclusion {
                source: "Retained standard deduction".into(),
                amount: 1575000,
            }],
            regular: 795500,
            amt: 0,
            carry: 204500,
            gains: None,
            regular_nol: 0,
            mtcnol: crate::MinimumTaxLoss::None,
            foreign_earned: None,
            foreign_credit: crate::MinimumTaxForeignCredit::None,
            old_vehicle_credit: 0,
        }));
    assert_eq!(supported(&d).refund, r.refund);
    d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts.carry += 100;
    unsupported(&d, "Repayment minimum-tax credit");
}

#[test]
fn repayment_minimum_credit_keeps_post_restoration_difference_explicit() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    let mut source = repayment_minimum_source(2024);
    source.carry = 2000000;
    years[0].minimum_tax = Some(source);
    years[0].income_tax = 0;
    unsupported(&d, "Repayment minimum-tax credit");
}

#[test]
fn repayment_minimum_credit_propagates_unchanged_balance_through_exclusion_amt() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    years[0].minimum_tax = Some(repayment_minimum_source(2024));
    years[0].exempt_interest = 10000000;
    years[0].private_activity_interest = 10000000;
    years[0].income_tax = 2259400;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_minimum_credit"), 0);
    assert_eq!(value(&r, "repayment_year_0_refigured_minimum_credit"), 0);
    assert_eq!(value(&r, "repayment_year_0_original_minimum_credit_carry"), 1000);
    assert_eq!(value(&r, "minimum_tax_credit_generated"), 0);
    assert_eq!(value(&r, "minimum_tax_credit"), 1000);
    assert_eq!(value(&r, "repayment_return_year_decrease"), 0);
    assert_eq!(value(&r, "repayment_credit"), 1300);
}

#[test]
fn repayment_minimum_credit_propagation_requires_unchanged_current_ownership() {
    let mut d = repayment_minimum_chain();
    let years = repayment_years_mut(&mut d);
    years[1].minimum_tax = None;
    years[1].wages = 0;
    years[1].income_tax = 0;
    // Source credit survives2025, but changing the current filing unit may not
    // transfer that credit without an allocation calculation.
    let mut joint = complete(FederalStatus::Joint);
    joint.deductions.as_mut().unwrap().repayment = d.deductions.unwrap().repayment;
    add(&mut joint, d.income.pop().unwrap());
    unsupported(&joint, "Repayment minimum-tax credit");
}

#[test]
fn repayment_minimum_credit_propagates_dividend_and_special_gain_sources() {
    for (ordinary, qualified, gains, unrecaptured, collectibles, original_credit, revised_credit) in [
        (10000, 10000, 0, 0, 0, 9455, 8355),
        (0, 0, 10000, 0, 0, 9455, 8355),
        (0, 0, 10000, 3000, 4000, 9945, 8845),
    ] {
        let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        let mut prior = repayment_minimum_source(2024);
        prior.carry = 900000;
        years[0].minimum_tax = Some(prior);
        years[0].income_tax = 0;
        years[0].dividends = RepaymentDividends::Statements {
            ordinary: ordinary * 100,
            qualified: qualified * 100,
            gains: gains * 100,
            unrecaptured: unrecaptured * 100,
            collectibles: collectibles * 100,
            confirmed: true,
        };
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_minimum_credit"), original_credit);
        assert_eq!(value(&r, "repayment_year_0_refigured_minimum_credit"), revised_credit);
        assert_eq!(value(&r, "minimum_tax_credit_available"), 10000 - original_credit);
        assert_eq!(value(&r, "minimum_tax_credit_generated"), 0);
        assert_eq!(value(&r, "repayment_return_year_decrease"), 1100);
        assert_eq!(value(&r, "repayment_credit"), 1100);
        assert_eq!(supported(&restart(d).draft).refund, r.refund);
    }
}

#[test]
fn repayment_minimum_credit_preferential_permanent_amt_generates_no_deferral_credit() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    years[0].minimum_tax = Some(repayment_minimum_source(2024));
    years[0].dividends = RepaymentDividends::Statements {
        ordinary: 1000000,
        qualified: 1000000,
        gains: 0,
        unrecaptured: 0,
        collectibles: 0,
        confirmed: true,
    };
    years[0].exempt_interest = 10000000;
    years[0].private_activity_interest = 10000000;
    years[0].income_tax = 2409400;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_tax"), 24094);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 22794);
    assert_eq!(value(&r, "minimum_tax_exclusion_tax"), 14639);
    assert_eq!(value(&r, "minimum_tax_credit_generated"), 0);
    assert_eq!(value(&r, "minimum_tax_credit"), 1000);
    assert_eq!(value(&r, "repayment_credit"), 1300);
}

#[test]
fn repayment_minimum_source_preserves_exclusion_line_rounding() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    let mut incoming = repayment_minimum_source(2024);
    incoming.carry = 900000;
    years[0].minimum_tax = Some(incoming);
    years[0].income_tax = 0;
    years[0].standard += 49;
    years[0].exempt_interest = 49;
    years[0].private_activity_interest = 49;
    // Each Form6251 line rounds independently; combining the two49-cent
    // remainders before rounding would falsely add a dollar of exclusion.
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(MinimumTaxHistory {
            return_form: crate::MinimumTaxReturnForm::Form1040,
            status: FederalStatus::Single,
            base: 5925000,
            exclusions: vec![MinimumTaxExclusion {
                source: "Separate filed line2a".into(),
                amount: 1575000,
            }],
            regular: 795500,
            amt: 0,
            carry: 204500,
            gains: None,
            regular_nol: 0,
            mtcnol: crate::MinimumTaxLoss::None,
            foreign_earned: None,
            foreign_credit: crate::MinimumTaxForeignCredit::None,
            old_vehicle_credit: 0,
        }));
    let r = supported(&d);
    assert_eq!(value(&r, "minimum_tax_credit"), 2045);
    assert_eq!(value(&r, "repayment_credit"), 1100);
}

#[test]
fn repayment_capital_deferral_generates_and_uses_distinct_minimum_credits() {
    let d = repayment_deferral_capital();
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_tax"), 22594);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 21294);
    assert_eq!(value(&r, "minimum_tax_credit_generated"), 3527);
    assert_eq!(value(&r, "minimum_tax_credit"), 3527);
    // Refigured2025 regular17,867 and TMT21,294 generate3,427.
    // Less credit in2026 offsets100 of the historical1,300 reduction.
    assert_eq!(value(&r, "repayment_return_year_decrease"), -100);
    assert_eq!(value(&r, "repayment_credit"), 1200);
    assert!(d.credits.as_ref().unwrap().minimum_tax.is_none());
    assert_eq!(supported(&restart(d).draft).refund, r.refund);
}

#[test]
fn repayment_capital_permanent_component_does_not_generate_minimum_credit() {
    let mut d = repayment_deferral_capital();
    let years = repayment_years_mut(&mut d);
    years[0].capital_exclusions = Some(vec![MinimumTaxExclusion {
        source: "Retained capital exclusion component".into(),
        amount: 5000000,
    }]);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_credit"), 1300);
    assert!(!r.lines.iter().any(|v| v.id == "minimum_tax_credit"));
}

#[test]
fn repayment_capital_minimum_credit_requires_resolved_exclusion_sources() {
    let mut d = repayment_deferral_capital();
    let years = repayment_years_mut(&mut d);
    years[0].capital_exclusions = None;
    unsupported(&d, "Repayment minimum-tax credit");
    let years = repayment_years_mut(&mut d);
    years[0].capital_exclusions = Some(vec![MinimumTaxExclusion { source: " ".into(), amount: 0 }]);
    assert!(!d.valid_stored());
}

#[test]
fn repayment_capital_mixed_permanent_and_deferral_components_share_exclusion_owner() {
    let mut d = repayment_deferral_capital();
    let years = repayment_years_mut(&mut d);
    years[0].capital_exclusions = Some(vec![MinimumTaxExclusion {
        source: "Permanent component of capital adjustment".into(),
        amount: 4000000,
    }]);
    let r = supported(&d);
    assert_eq!(value(&r, "minimum_tax_exclusion_tax"), 927);
    assert_eq!(value(&r, "minimum_tax_credit_generated"), 2600);
    assert_eq!(value(&r, "minimum_tax_credit"), 2600);
    assert_eq!(value(&r, "repayment_return_year_decrease"), 0);
    assert_eq!(value(&r, "repayment_credit"), 1300);
}

#[test]
fn repayment_capital_generated_credit_propagates_through_an_intervening_return() {
    let mut d = repayment_deferral_capital();
    for e in &mut d.income {
        if let FederalIncome::GovernmentPayment { data } = &mut e.income {
            data.repayments[0].received_year = 2024;
        }
    }
    let years = repayment_years_mut(&mut d);
    years[0].year = 2024;
    years[0].standard = 1460000;
    years[0].income_tax = 2321800;
    let mut next = years[0].clone();
    next.year = 2025;
    next.standard = 1575000;
    next.benefits = 0;
    next.capital = RepaymentCapital::None;
    next.capital_exclusions = None;
    next.income_tax = 139600;
    years.push(next);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_1_minimum_credit_available"), 3679);
    assert_eq!(value(&r, "repayment_year_1_refigured_minimum_credit_available"), 3579);
    assert_eq!(value(&r, "repayment_year_1_original_tax"), 1396);
    assert_eq!(value(&r, "repayment_year_1_refigured_tax"), 1496);
    assert_eq!(value(&r, "repayment_credit"), 1200);
    assert_eq!(supported(&restart(d).draft).refund, r.refund);
}

#[test]
fn repayment_capital_exclusion_record_is_not_frozen_across_changed_loss_utilization() {
    let mut d = repayment_carry_chain();
    let years = repayment_years_mut(&mut d);
    // A retained original capital adjustment does not describe the hypothetical
    // exclusion-only carryover after the preceding loss changes.
    if let RepaymentCapital::Records { sales, .. } = &mut years[0].capital {
        sales[0].sale.amt = crate::SaleAmt::Different {
            record: CapitalSaleAmt { term: None, basis: 1200000, wash_sale: 0 },
        };
    }
    if let RepaymentCapital::Records { amt, .. } = &mut years[1].capital {
        *amt = Some(RepaymentAmtLosses { short: 900000, long: 0 });
    }
    years[1].capital_exclusions = Some(vec![]);
    years[1].exempt_interest = 10000000;
    years[1].private_activity_interest = 10000000;
    unsupported(&d, "Repayment minimum-tax credit");
}

#[test]
fn repayment_zero_minimum_balance_reconciles_current_record() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(MinimumTaxHistory {
            return_form: crate::MinimumTaxReturnForm::Form1040,
            status: FederalStatus::Single,
            base: 5925000,
            exclusions: vec![MinimumTaxExclusion {
                source: "Retained deduction".into(),
                amount: 1575000,
            }],
            regular: 795500,
            amt: 0,
            carry: 0,
            gains: None,
            regular_nol: 0,
            mtcnol: crate::MinimumTaxLoss::None,
            foreign_earned: None,
            foreign_credit: crate::MinimumTaxForeignCredit::None,
            old_vehicle_credit: 0,
        }));
    assert_eq!(value(&supported(&d), "repayment_credit"), 1100);
    d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts.carry = 100000;
    unsupported(&d, "Repayment minimum-tax credit");
}

#[test]
fn repayment_permanent_amt_rejects_contradictory_current_credit_generation() {
    let mut d = repayment_deferral_capital();
    let years = repayment_years_mut(&mut d);
    years[0].capital_exclusions = Some(vec![MinimumTaxExclusion {
        source: "Permanent capital component".into(),
        amount: 5000000,
    }]);
    d.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(MinimumTaxHistory {
            return_form: crate::MinimumTaxReturnForm::Form1040,
            status: FederalStatus::Single,
            base: 10925000,
            exclusions: vec![MinimumTaxExclusion {
                source: "Standard deduction".into(),
                amount: 1575000,
            }],
            regular: 1906700,
            amt: 352700,
            carry: 0,
            gains: None,
            regular_nol: 0,
            mtcnol: crate::MinimumTaxLoss::None,
            foreign_earned: None,
            foreign_credit: crate::MinimumTaxForeignCredit::None,
            old_vehicle_credit: 0,
        }));
    // Omitting the50,000 permanent adjustment from the second copy would
    // invent a3,527 current credit despite the complete historical source.
    unsupported(&d, "Repayment minimum-tax credit");
    d.credits.as_mut().unwrap().minimum_tax.as_mut().unwrap().returns[0].facts.exclusions.push(
        MinimumTaxExclusion {
            source: "Same capital source, different label".into(),
            amount: 5000000,
        },
    );
    let r = supported(&d);
    assert_eq!(value(&r, "minimum_tax_credit"), 0);
    assert_eq!(value(&r, "repayment_credit"), 1300);
}

#[test]
fn repayment_exclusion_worksheet_reuses_proven_permanent_preceding_losses() {
    let mut d = repayment_carry_chain();
    let years = repayment_years_mut(&mut d);
    if let RepaymentCapital::Records { exclusion, .. } = &mut years[1].capital {
        *exclusion = Some(RepaymentExclusionLosses { short: 700000, long: 0 });
    }
    years[1].exempt_interest = 10000000;
    years[1].private_activity_interest = 10000000;
    years[1].income_tax = 2207400;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_1_original_exclusion_income"), 13000);
    assert_eq!(value(&r, "repayment_year_1_refigured_exclusion_income"), 13000);
    assert_eq!(value(&r, "repayment_credit"), 241);
}

#[test]
fn repayment_exclusion_losses_remaining_different_at_restoration_are_explicit() {
    let mut d = repayment_exclusion_loss_chain();
    let years = repayment_years_mut(&mut d);
    if let RepaymentCapital::Records { sales, .. } = &mut years[1].capital {
        sales.clear();
    }
    // The independently tracked losses have not been used by the end of2025.
    // This cannot silently become a complete restoration/future-credit result.
    unsupported(&d, "Repayment minimum-tax credit");
}

#[test]
fn self_employed_housing_rejects_duplicate_and_unresolved_claim_contexts() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(excluded_foreign_business(200000, 50000)));
    set_claim(&mut d, foreign_claim(Owner::Taxpayer));
    self_employed_housing(&mut d, None);
    // Housing is entered once for the person on the Form 2555 record, so a
    // second business adds no second claim.
    let mut duplicate = d.clone();
    let second = duplicate.income[0].clone();
    add(&mut duplicate, second);
    supported(&duplicate);
    let mut mixed = d.clone();
    add(&mut mixed, income(foreign_pay(10000)));
    supported(&mixed);
    // Housing stays outside a presence period that ends before them.
    let mut wrong_period = d.clone();
    claim_mut(&mut wrong_period, Owner::Taxpayer).qualification =
        ForeignQualification::Presence { starts_on: 20250701, foreign_days: 365 };
    unsupported(&wrong_period, "Foreign housing dates");
    self_employed_housing(&mut d, Some(HousingCarryover { amount: 1800000, deducted: 800000 }));
    // R3/A4: the actual prior amount less deductions supplies the carry;
    // the removed confirmation no longer discards a valid $10,000 balance.
    let r = supported(&d);
    assert!(!noted(&r, "WARN_HOUSING_CARRYOVER_LEFT_OUT"));
    assert_eq!(value(&r, "foreign_housing_taxpayer_prior_deduction_used"), 10000);
}

#[test]
fn prior_housing_carryover_does_not_require_current_housing_amount() {
    for (receipts, used, expired, agi, tax) in
        [(200000, 10000, 0, 36770, 26094), (100000, 0, 10000, 0, 14130), (0, 0, 10000, 0, 0)]
    {
        let mut d = empty_return(FederalStatus::Single);
        add(
            &mut d,
            income(excluded_foreign_business(receipts, if receipts == 200000 { 50000 } else { 0 })),
        );
        set_claim(&mut d, foreign_claim(Owner::Taxpayer));
        self_employed_housing(&mut d, Some(HousingCarryover { amount: 1800000, deducted: 800000 }));
        let ForeignHousing::Claim { expenses, .. } =
            &mut claim_mut(&mut d, Owner::Taxpayer).housing
        else {
            unreachable!()
        };
        *expenses = 0;
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_housing_taxpayer_current_deduction"), 0);
        assert_eq!(value(&r, "foreign_housing_taxpayer_prior_deduction_used"), used);
        assert_eq!(value(&r, "foreign_housing_taxpayer_prior_deduction_expired"), expired);
        assert_eq!(value(&r, "foreign_housing_taxpayer_deduction_carryforward"), 0);
        assert_eq!(value(&r, "foreign_housing_deduction"), used);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(r.tax, Some(tax * 100));
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn employee_income_uses_prior_self_employed_housing_carry() {
    for (wages, expenses, used, agi) in [
        (200000, 50000, 10000, 38494),
        (200000, 0, 10000, 57100),
        (155000, 50000, 3494, 0),
        (100000, 50000, 0, 0),
    ] {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(foreign_pay(wages)));
        set_claim(&mut d, foreign_claim(Owner::Taxpayer));
        employee_housing(&mut d, expenses, ForeignHousingLocation::General);
        let ForeignHousing::Claim { prior, .. } = &mut claim_mut(&mut d, Owner::Taxpayer).housing
        else {
            unreachable!()
        };
        *prior = Some(HousingCarryover { amount: 1800000, deducted: 800000 });
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_housing_deduction"), used);
        assert_eq!(value(&r, "foreign_housing_taxpayer_prior_deduction_expired"), 10000 - used);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(
            value(&r, "net_investment_modified_agi"),
            agi + core::cmp::min(
                132900,
                wages - core::cmp::min(wages, if expenses == 0 { 0 } else { 18606 })
            )
        );
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn mixed_housing_funding_uses_one_claim_and_preserves_business_allocation() {
    {
        let mut d = empty_return(FederalStatus::Single);
        add(&mut d, income(excluded_foreign_business(200000, 50000)));
        set_claim(&mut d, foreign_claim(Owner::Taxpayer));
        self_employed_housing(&mut d, None);
        add(&mut d, income(foreign_pay(100000)));
        let r = supported(&d);
        assert_eq!(value(&r, "foreign_housing_exclusion"), 6202);
        assert_eq!(value(&r, "foreign_housing_deduction"), 12404);
        assert_eq!(value(&r, "foreign_earned_exclusion"), 132900);
        assert_eq!(value(&r, "foreign_business_taxpayer_disallowed_deductions"), 26844);
        assert_eq!(value(&r, "agi"), 114741);
        assert_eq!(value(&r, "self_employment_tax"), 21194);
        assert_eq!(r.tax, Some(4659000));
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn mixed_housing_funding_orders_current_and_prior_for_each_owner() {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for exclude in [false, true] {
            let mut d = empty_return(FederalStatus::Joint);
            let mut business = income(excluded_foreign_business(50000, 0));
            business.owner = crate::DocumentOwner::from(owner);
            add(&mut d, business);
            self_employed_housing(
                &mut d,
                Some(HousingCarryover { amount: 1800000, deducted: 800000 }),
            );
            let claim = claim_mut(&mut d, owner);
            claim.exclusion = exclude;
            let ForeignHousing::Claim { expenses, .. } = &mut claim.housing else { unreachable!() };
            *expenses = 3626400;
            let mut pay = income(foreign_pay(50000));
            pay.owner = crate::DocumentOwner::from(owner);
            add(&mut d, pay);
            let r = supported(&d);
            let person = if owner == Owner::Spouse { "spouse" } else { "taxpayer" };
            assert_eq!(value(&r, "foreign_housing_exclusion"), 7500);
            assert_eq!(value(&r, "foreign_earned_exclusion"), if exclude { 92500 } else { 0 });
            assert_eq!(
                value(&r, &alloc::format!("foreign_housing_{person}_current_deduction")),
                if exclude { 0 } else { 7500 }
            );
            assert_eq!(
                value(&r, &alloc::format!("foreign_housing_{person}_prior_deduction_used")),
                if exclude { 0 } else { 10000 }
            );
            assert_eq!(
                value(&r, &alloc::format!("foreign_housing_{person}_deduction_carryforward")),
                if exclude { 7500 } else { 0 }
            );
            assert_eq!(
                value(&r, &alloc::format!("foreign_housing_{person}_prior_deduction_expired")),
                if exclude { 10000 } else { 0 }
            );
            assert_eq!(value(&r, "agi"), if exclude { -265 } else { 71467 });
            assert_eq!(value(&r, "self_employment_tax"), 7065);
            assert_eq!(supported(&restart(d.clone()).draft), r);
            d.income.reverse();
            assert_eq!(supported(&d), r);
        }
    }
}

#[test]
fn repayment_historical_railroad_recovery_cannot_offset_other_components() {
    let mut d = repayment_social_draft(2023, 15700, 1199);
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        unreachable!()
    };
    years[0].wages = 0;
    years[0].retirement = vec![RepaymentRetirement::Railroad {
        owner: Owner::Taxpayer,
        gross: 2000000,
        contributory: 500000,
        basis: Some(PensionBasis {
            earlier_months: 0,
            changes: vec![],
            share: None,
            started: 20000101,
            cost: Some(10000000),
            months: 12,
            history: PensionHistory::Prior { monthly: 100000, recovered: 8000000 },
            eligible: true,
        }),
        normal: true,
    }];
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_pension_taxable"), 15000);
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        unreachable!()
    };
    let RepaymentRetirement::Railroad { basis: Some(basis), .. } = &mut years[0].retirement[0]
    else {
        unreachable!()
    };
    basis.started = 20240101; // Cannot exist on the original 2023 return.
    assert!(!d.valid_stored());
}

#[test]
fn railroad_and_unemployment_repayments_share_return_threshold() {
    let mut d = railroad_repayment_draft();
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    let mut data = edit_retirement(data);
    let Some(RetirementTaxable::Railroad { repayments, repaid, .. }) = &mut data.taxable else {
        unreachable!()
    };
    repayments[0].amount = 300000;
    *repaid = 300000;
    drop(data);
    let mut unemployment = prior_unemployment_repayment(200000, Owner::Taxpayer);
    let FederalIncome::GovernmentPayment { data } = &mut unemployment.income else {
        unreachable!()
    };
    data.repayments[0].received_year = 2023;
    add(&mut d, unemployment);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_credit"), 766);
    assert_eq!(value(&r, "repayment_year_0_refigured_retirement_taxable"), 12000);
    assert_eq!(value(&r, "prior_income_repayment_review_total"), 5000);
    d.deductions.as_mut().unwrap().repayment = RepaymentClaim::None;
    unsupported(&d, "Earlier-income repayments");
    d.income.pop(); // Exactly $3,000 railroad repayment needs no claim worksheet.
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "repayment_credit"), 0);
}

#[test]
fn final_survivor_deduction_includes_earlier_common_recovery() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { date: 20260930, bonds: None, ordinary: true, estate_medical: 0 });
    let mut p = pension_basis();
    p.started = 20200101;
    p.cost = Some(3600000);
    p.history = PensionHistory::Prior { monthly: 10000, recovered: 720000 };
    p.earlier_months = 6;
    p.months = 3;
    let mut entry = pension_entry(p, 1500);
    let FederalIncome::Retirement { data } = &mut entry.income else { unreachable!() };
    let mut data = edit_retirement(data);
    data.ordinary_distribution = Some(RetirementTreatment::Exception {
        second: None,
        distributed: 20260901,
        facts: RetirementException::Death { died: 20260615 },
        early_simple: false,
    });
    drop(data);
    add(&mut d, entry);
    d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
        income_id: 2,
        started: 20200101,
        cost: 3600000,
        recovered: 720000,
        refunds: 100000,
        eligible: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "annuity_loss_2"), 26900);
    assert_eq!(value(&r, "unrecovered_annuity_deduction"), 26900);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.deductions.as_mut().unwrap().annuity_losses[0].recovered -= 100;
    unsupported(&d, "Annuity loss:");
}

#[test]
fn general_rule_delayed_first_fraction_is_recovered_only_on_receipt() {
    let mut a = fixed_annuity();
    a.started = 20251015;
    a.cost = 1080000;
    a.periodic = 10000;
    a.duration = AnnuityDuration::SingleLife { age: 65, delay: 1, guaranteed: 0 };
    a.received = 14;
    a.partial = 5000;
    a.recovered = 0;
    assert!(a.valid(145000));
    assert_eq!(a.excluded(), 65250);
    let mut d = complete(FederalStatus::Single);
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        1450,
        RetirementTreatment::Ordinary,
    );
    if let FederalIncome::Retirement { data } = &mut entry.income {
        let mut data = edit_retirement(data);
        data.taxable = Some(RetirementTaxable::GeneralAnnuity { data: a });
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 79800); // $797.50 rounded on Form 1040.
    assert_eq!(restart(d.clone()).draft, d);
    a.received = 15;
    assert!(!a.valid(155000)); // The fractional first month also occupies a period.
}

#[test]
fn final_general_rule_survivor_deduction_includes_earlier_recovery() {
    let mut a = joint_annuity(true);
    a.started = 20200101;
    a.received = 3;
    a.earlier_payments = 300000;
    a.recovered = 1000000;
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { date: 20260930, bonds: None, ordinary: true, estate_medical: 0 });
    let mut entry = retirement_entry(
        Owner::Taxpayer,
        RetirementKind::Nonqualified,
        1050,
        RetirementTreatment::Exception {
            second: None,
            distributed: 20260901,
            facts: RetirementException::Death { died: 20260615 },
            early_simple: false,
        },
    );
    if let FederalIncome::Retirement { data } = &mut entry.income {
        let mut data = edit_retirement(data);
        data.taxable = Some(RetirementTaxable::GeneralAnnuity { data: a });
    }
    add(&mut d, entry);
    d.deductions.as_mut().unwrap().annuity_losses.push(AnnuityLoss {
        income_id: 2,
        started: a.started,
        cost: a.cost,
        recovered: a.recovered,
        refunds: 0,
        eligible: true,
    });
    let r = supported(&d);
    assert_eq!(value(&r, "annuity_2_earlier_recovery"), 1551);
    // $62,712 - $10,000 prior recovery - $1,551 primary - $542.85 survivor.
    assert_eq!(value(&r, "annuity_loss_2"), 50618);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn general_rule_earlier_recovery_caps_new_contracts_but_not_1986_exclusions() {
    let mut a = joint_annuity(true);
    a.earlier_payments = 300000;
    a.received = 6;
    a.recovered = a.cost - 10000;
    assert_eq!(a.earlier_recovery(), 10000);
    assert_eq!(a.excluded(), 0);
    a.started = 19860702;
    a.recovered = a.cost + 10000;
    assert!(a.valid(210000));
    assert_eq!(a.earlier_recovery(), 155100);
    assert_eq!(a.excluded(), 108570);
    a.earlier_payments = 25000; // A fractional original primary payment.
    assert_eq!(a.earlier_recovery(), 12925);
}

#[test]
fn temporary_living_expense_insurance_rejects_bad_sources_and_duplicate_claims() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::CasualtyLivingInsurance {
            data: FederalCasualtyLivingInsurance {
                event_reference: "SYN-EVENT-2026-2".into(),
                loss_of_use: FederalHomeLossOfUse::Casualty,
                lost_use_on: 20260601,
                regained_use_on: Some(20260701),
                payments: vec![casualty_living_payment("SYN-2026-2", 20261001, 100_000, 0)],
                actual_living_expenses: 200_000,
                normal_living_expenses: 50_000,
                prior_taxable_reported: 0,
                federal_disaster_area: false,
                federal_declaration_reference: "".into(),
                confirmed: true,
            },
        }),
    );
    assert_eq!(value(&supported(&d), "taxable_casualty_living_insurance"), 0);
    let duplicate = d.income.last().unwrap().clone();
    add(&mut d, duplicate);
    assert!(!d.valid_stored());
    d.income.pop();
    if let FederalIncome::CasualtyLivingInsurance { data } =
        &mut d.income.last_mut().unwrap().income
    {
        data.regained_use_on = Some(20270101);
    }
    assert!(!d.valid_stored());
    if let FederalIncome::CasualtyLivingInsurance { data } =
        &mut d.income.last_mut().unwrap().income
    {
        data.regained_use_on = Some(20260531);
    }
    assert!(!d.valid_stored());
    if let FederalIncome::CasualtyLivingInsurance { data } =
        &mut d.income.last_mut().unwrap().income
    {
        data.regained_use_on = Some(20260701);
        data.payments[0].amount = 100_000;
        data.federal_disaster_area = true;
    }
    assert!(!d.valid_stored());
    if let FederalIncome::CasualtyLivingInsurance { data } =
        &mut d.income.last_mut().unwrap().income
    {
        data.federal_disaster_area = false;
        data.payments[0].received_on = 20250501;
    }
    assert!(!d.valid_stored());
    if let FederalIncome::CasualtyLivingInsurance { data } =
        &mut d.income.last_mut().unwrap().income
    {
        data.payments[0].received_on = 20261001;
        data.payments[0].payment_reference = "SYN-2026-2-20261001".into();
        data.payments.push(data.payments[0].clone());
    }
    assert!(!d.valid_stored());
}

#[test]
fn volunteer_actual_fuel_and_mileage_use_distinct_records_without_standard_cash_claim() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = None;
    ded.volunteer_expenses.extend([
        volunteer_travel(VolunteerCarCost::Mileage { miles: 50 }, 300),
        volunteer_travel(VolunteerCarCost::ActualFuel { gas_oil: 1_500 }, 500),
    ]);
    let result = supported(&d);
    assert_eq!(value(&result, "charity_volunteer_1_expense"), 10);
    assert_eq!(value(&result, "charity_volunteer_2_expense"), 20);
    assert_eq!(return_line(&result, "12f"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    d.deductions.as_mut().unwrap().set_cash_charity(100_000);
    let capped = supported(&d);
    assert_eq!(return_line(&capped, "12f"), 100_000);
}

#[test]
fn small_b_claim_uses_floor_before_prior_or_current_cash() {
    let draft =
        with_current_direct_cash(cash_carry_with_whaling(10_000_000, 10_000, 5_000), 2_000_000);
    let itemized = supported(&draft);
    assert_eq!(value(&itemized, "charity_whaling_deduction"), 0);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_floor_not_carried"), 400);
    assert_eq!(value(&itemized, "charity_prior_cash_sixty_deduction"), 4_600);
    assert_eq!(value(&itemized, "charity_cash_sixty_current_deduction"), 20_000);
    assert_eq!(value(&itemized, "charity_floor_not_carried"), 500);
    assert_eq!(value(&itemized, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&itemized, "12e"), 2_460_000);
    assert_eq!(estimate(&restart(draft).draft), itemized);
}

#[test]
fn current_cash_prior_cash_and_b_pair_requires_supported_sources_only() {
    let draft =
        with_current_direct_cash(cash_carry_with_whaling(10_000_000, 1_000_000, 20_000), 2_000_000);
    supported(&draft);

    let mut incomplete_prior = draft.clone();
    incomplete_prior.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&incomplete_prior), "WARN_CHARITY_CARRY_LEFT_OUT"));

    let mut extra_property = draft.clone();
    extra_property
        .deductions
        .as_mut()
        .unwrap()
        .property_gifts
        .push(ordinary_property_for_cash_carry(10_000));
    assert_eq!(value(&supported(&extra_property), "charity_deduction"), 49_600);

    let mut dual_b = draft.clone();
    let mut hosted = cash_carry_with_hosted_student(10_000_000, 1, 20_000);
    dual_b
        .deductions
        .as_mut()
        .unwrap()
        .hosted_students
        .push(hosted.deductions.as_mut().unwrap().hosted_students.remove(0));
    let dual_result = supported(&dual_b);
    assert_eq!(value(&dual_result, "charity_other_30_percent_deduction"), 9_550);

    let mut prior_nol_history = draft.clone();
    prior_nol_history.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 10_000)]);
    unsupported(&prior_nol_history, "Charitable carryover coordination");

    let mut foreign_tax = draft;
    payee_statement_credit(&mut foreign_tax, 10_000);
    unsupported(&foreign_tax, "Charitable carryover coordination");
}

#[test]
fn prior_2025_cash_and_whaling_claim_keep_class_and_vintage_through_full_return() {
    let mut draft = cash_carry_with_whaling(10_000_000, 1_000_000, 20_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "agi"), 100_000);
    assert_eq!(value(&result, "charity_whaling_statutory_claim"), 10_000);
    assert_eq!(value(&result, "charity_whaling_deduction"), 9_500);
    assert_eq!(value(&result, "charity_whaling_return_year_carryforward"), 0);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 20_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&result, "charity_floor_not_carried"), 500);
    assert_eq!(return_line(&result, "12e"), 2_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), result);

    let mut without_prior = draft.clone();
    without_prior.deductions.as_mut().unwrap().charity_carryovers.clear();
    let current_only = supported(&without_prior);
    assert_eq!(return_line(&current_only, "12e"), 950_000);
    assert_eq!(return_line(&current_only, "15") - return_line(&result, "15"), 2_000_000);
    assert!(result.tax < current_only.tax);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(value(&automatic, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_deduction"), 20_000);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&automatic, "charity_whaling_deduction"), 9_500);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(estimate(&restart(draft.clone()).draft), automatic);
}

#[test]
fn prior_cash_with_whaling_charges_floor_to_current_thirty_percent_class_first() {
    let draft = cash_carry_with_whaling(10_000_000, 10_000, 20_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_whaling_deduction"), 0);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_floor_not_carried"), 400);
    assert_eq!(value(&result, "charity_prior_cash_sixty_deduction"), 19_600);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&result, "12e"), 1_960_000);
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn prior_cash_with_whaling_preserves_thirty_percent_excess_and_statutory_cap() {
    let draft = cash_carry_with_whaling(10_000_000, 4_000_000, 10_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_whaling_statutory_claim"), 40_000);
    assert_eq!(value(&result, "charity_whaling_deduction"), 29_500);
    assert_eq!(value(&result, "charity_whaling_return_year_carryforward"), 10_500);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 10_500);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 10_000);
    assert_eq!(value(&result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(return_line(&result, "12e"), 3_950_000);
    assert_eq!(estimate(&restart(draft).draft), result);

    let capped = cash_carry_with_whaling(20_000_000, 5_500_000, 20_000);
    let capped_result = supported(&capped);
    assert_eq!(value(&capped_result, "charity_whaling_eligible_expenses"), 55_000);
    assert_eq!(value(&capped_result, "charity_whaling_statutory_claim"), 50_000);
    assert_eq!(value(&capped_result, "charity_whaling_deduction"), 49_000);
    assert_eq!(value(&capped_result, "charity_whaling_return_year_carryforward"), 0);
    assert_eq!(value(&capped_result, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(return_line(&capped_result, "12e"), 6_900_000);
    assert_eq!(estimate(&restart(capped).draft), capped_result);
}

#[test]
fn prior_cash_competes_with_whaling_and_uses_ten_percent_cash_carve_out() {
    // AGI $100,000: $50,000 of old (G) cash leaves $10,000 of (B) room,
    // rather than zero, under amended section 170(b)(1)(B)(ii).
    let mut draft = cash_carry_with_whaling(10_000_000, 3_000_000, 50_000);
    let itemized_result = supported(&draft);
    assert_eq!(value(&itemized_result, "charity_carry_2025_cash_sixty_used"), 50_000);
    assert_eq!(value(&itemized_result, "charity_prior_cash_sixty_deduction"), 50_000);
    assert_eq!(value(&itemized_result, "charity_prior_cash_sixty_remaining_following_year"), 0);
    assert_eq!(value(&itemized_result, "charity_whaling_deduction"), 9_500);
    assert_eq!(value(&itemized_result, "charity_whaling_return_year_carryforward"), 20_500);
    assert_eq!(value(&itemized_result, "charity_return_year_carryforward"), 20_500);
    assert_eq!(return_line(&itemized_result, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), itemized_result);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_whaling_return_year_carryforward"), 20_500);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(estimate(&restart(draft).draft), automatic);
}

#[test]
fn prior_cash_at_sixty_percent_cap_displaces_whaling_and_keeps_old_balance() {
    let mut draft = cash_carry_with_whaling(10_000_000, 1_000_000, 70_000);
    let itemized_result = supported(&draft);
    assert_eq!(value(&itemized_result, "charity_carry_2025_cash_sixty_used"), 60_000);
    assert_eq!(value(&itemized_result, "charity_prior_cash_sixty_deduction"), 59_500);
    assert_eq!(value(&itemized_result, "charity_prior_cash_sixty_floor_not_carried"), 500);
    assert_eq!(
        value(&itemized_result, "charity_prior_cash_sixty_remaining_following_year"),
        10_000
    );
    assert_eq!(value(&itemized_result, "charity_whaling_deduction"), 0);
    assert_eq!(value(&itemized_result, "charity_whaling_return_year_carryforward"), 10_000);
    assert_eq!(value(&itemized_result, "charity_return_year_carryforward"), 10_000);
    assert_eq!(return_line(&itemized_result, "12e"), 5_950_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), itemized_result);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&draft);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_consumed_without_deduction"), 0);
    assert_eq!(value(&automatic, "charity_prior_cash_sixty_remaining_following_year"), 10_000);
    assert_eq!(value(&automatic, "charity_whaling_return_year_carryforward"), 10_000);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(estimate(&restart(draft).draft), automatic);
}

#[test]
fn prior_cash_with_whaling_requires_complete_sources() {
    let draft = cash_carry_with_whaling(10_000_000, 3_000_000, 20_000);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_carry_2025_cash_sixty_used"), 20_000);
    assert_eq!(value(&result, "charity_whaling_deduction"), 29_500);

    let mut competing = draft.clone();
    competing.deductions.as_mut().unwrap().charity_carryovers[0].balance = 2_100_000;
    let competing_result = supported(&competing);
    assert_eq!(value(&competing_result, "charity_carry_2025_cash_sixty_used"), 21_000);
    assert_eq!(value(&competing_result, "charity_whaling_deduction"), 29_500);
    assert_eq!(return_line(&competing_result, "12e"), 5_050_000);

    let mut older = draft.clone();
    older.deductions.as_mut().unwrap().charity_carryovers[0].year = 2024;
    assert_eq!(value(&supported(&older), "charity_deduction"), 49_500);

    let mut unconfirmed_whaling = draft.clone();
    unconfirmed_whaling.deductions.as_mut().unwrap().whaling_captains[0]
        .eligibility_and_records_confirmed = false;
    assert!(noted(&supported(&unconfirmed_whaling), "WARN_WHALING_EXPENSES_LEFT_OUT"));

    let mut changed_filer = draft.clone();
    changed_filer.deductions.as_mut().unwrap().charity_carryovers[0].same_filers = false;
    assert!(noted(&supported(&changed_filer), "WARN_CHARITY_CARRY_LEFT_OUT"));

    let mut current_cash = draft.clone();
    current_cash.deductions.as_mut().unwrap().set_cash_charity(100_000);
    supported(&current_cash);

    let mut service_expense = current_cash;
    service_expense
        .deductions
        .as_mut()
        .unwrap()
        .volunteer_expenses
        .push(volunteer_travel(VolunteerCarCost::Mileage { miles: 50 }, 0));
    assert_eq!(value(&supported(&service_expense), "charity_deduction"), 50_507);

    let mut mixed_property = draft.clone();
    mixed_property
        .deductions
        .as_mut()
        .unwrap()
        .property_gifts
        .push(ordinary_property_for_cash_carry(10_000));
    assert_eq!(value(&supported(&mixed_property), "charity_deduction"), 49_600);

    let mut mixed_student = draft;
    mixed_student.deductions.as_mut().unwrap().hosted_students.push(HostedStudent {
        label: "Synthetic hosted student".into(),
        sponsor: "Synthetic educational charity".into(),
        agreement: "Synthetic written education agreement".into(),
        school: "Synthetic high school".into(),
        school_location: "Synthetic City, CA".into(),
        grade: 11,
        joined: 20260601,
        school_start: 20260601,
        school_end: 20260630,
        months: vec![HostedStudentMonth { month: 6, qualifying_days: 15, cost: 5_000 }],
        documented: true,
    });
    let mut second_student = mixed_student.deductions.as_ref().unwrap().hosted_students[0].clone();
    second_student.label = "Second synthetic hosted student".into();
    second_student.joined = 20260701;
    second_student.school_start = 20260701;
    second_student.school_end = 20260731;
    second_student.months[0].month = 7;
    mixed_student.deductions.as_mut().unwrap().hosted_students.push(second_student);
    assert_eq!(value(&supported(&mixed_student), "charity_deduction"), 49_500);
}

#[test]
fn prior_public_stock_retains_2025_vintage_and_uses_2026_floor() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.charity_carryovers.push(prior_public_stock_carry());
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let itemized_result = supported(&d);
    // Original-lot amounts are retained in historical audit evidence; this page carries the reconciled class/year balance.
    // Original-lot amounts are retained in historical audit evidence; this page carries the reconciled class/year balance.
    assert_eq!(value(&itemized_result, "charity_carry_2025_capital_gain_thirty_opening"), 10_000);
    assert_eq!(value(&itemized_result, "charity_carry_2025_capital_gain_thirty_used"), 10_000);
    assert_eq!(value(&itemized_result, "charity_carry_2025_capital_gain_thirty_deduction"), 9_700);
    assert_eq!(
        value(&itemized_result, "charity_carry_2025_capital_gain_thirty_floor_not_carried"),
        300
    );
    assert_eq!(value(&itemized_result, "charity_carry_2025_capital_gain_thirty_remaining"), 0);
    assert_eq!(value(&itemized_result, "charity_return_year_carryforward"), 0);
    assert_eq!(return_line(&itemized_result, "12e"), 970_000);
    assert!(
        itemized_result
            .review_notes
            .iter()
            .any(|note| note.contains("expires after 2030") && note.contains("Form 8283"))
    );
    assert_eq!(estimate(&restart(d.clone()).draft), itemized_result);

    d.deductions.as_mut().unwrap().itemize_override = None;
    let standard = supported(&d);
    assert_eq!(value(&standard, "charity_carry_2025_capital_gain_thirty_deduction"), 0);
    assert_eq!(
        value(&standard, "charity_carry_2025_capital_gain_thirty_consumed_without_deduction"),
        10_000
    );
    assert_eq!(value(&standard, "charity_carry_2025_capital_gain_thirty_floor_not_carried"), 0);
    assert_eq!(value(&standard, "charity_carry_2025_capital_gain_thirty_remaining"), 0);
    assert_eq!(estimate(&restart(d.clone()).draft), standard);
}

#[test]
fn prior_public_stock_blocks_current_operating_loss_carry_coordination() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, wage(0, Owner::Taxpayer, 1_000_000, 0));
    add(&mut d, income(cash_business(-30_000)));
    d.deductions.as_mut().unwrap().charity_carryovers.push(prior_public_stock_carry());
    unsupported(&d, "Charitable carryover current NOL");
}

#[test]
fn cash_retained_card_issuer_claim_is_2026_payment_without_2026_income() {
    let business = cash_retained_card_issuer_business();
    let issuer = business.fuel_tax_credit.as_ref().unwrap().fuels.card_issuer.as_ref().unwrap();
    assert!(issuer.valid() && issuer.qualified());
    assert_eq!(issuer.lines(), [2_430, 2_430, 2_180, 2_430]);
    assert_eq!(issuer.retained_income_cents(), 9_470);
    assert_eq!(business.receipts(), 4_000_000);

    let mut draft = complete(FederalStatus::Single);
    let mut without_credit = draft.clone();
    let mut baseline_business = business.clone();
    baseline_business.fuel_tax_credit = None;
    add(&mut without_credit, income(FederalIncome::Business { data: baseline_business }));
    let baseline = supported(&without_credit);
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let report = supported(&draft);
    for (label, amount) in [("13a", 2_430), ("13b", 2_430), ("13c_219", 2_180), ("13c_244", 2_430)]
    {
        assert_eq!(
            report
                .lines
                .iter()
                .find(|line| line.id == alloc::format!("business_2_fuel_{label}"))
                .unwrap()
                .amount,
            amount,
        );
    }
    assert!(
        report.lines.iter().all(|line| line.id != "business_2_card_issuer_retained_schedule_c_6")
    );
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 9_500);
    assert_eq!(value(&report, "agi"), value(&baseline, "agi"));
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(report.refundable_credits, baseline.refundable_credits);
    assert_eq!(report.payments.unwrap(), baseline.payments.unwrap() + 9_500);
    assert_eq!(
        report.refund.unwrap() - report.balance.unwrap(),
        baseline.refund.unwrap() - baseline.balance.unwrap() + 9_500
    );
    assert!(report.review_notes.iter().any(|note| {
        note.contains("$94.70 retained buyer-funded")
            && note.contains("2026 refundable payment")
            && note.contains("often 2027")
            && note.contains("actually or constructively received")
    }));
    let note = &report
        .review_notes
        .iter()
        .find(|note| note.contains("$94.70 retained buyer-funded"))
        .unwrap()
        .display;
    assert_eq!(note.key, "NOTE_CARD_ISSUER_RETAINED_FUEL_CREDIT");
    assert_eq!(note.args[1], MessageArg::usd_cents(9_470));
    assert_eq!(estimate(&restart(draft.clone()).draft), report);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn cash_retained_card_issuer_requires_settled_and_unreceived_claim_facts() {
    let good = cash_retained_card_issuer_business();
    for case in 0..12 {
        let mut business = good.clone();
        let issuer = business.fuel_tax_credit.as_mut().unwrap().fuels.card_issuer.as_mut().unwrap();
        match case {
            0 => issuer.retained_cash_accounting_confirmed = None,
            1 => issuer.retained_cash_accounting_confirmed = Some(false),
            2 => issuer.retained_accrual_accounting_confirmed = Some(true),
            3 => issuer.sales[2].retained_claimable_tax -= 1,
            4 => issuer.sales[3].retained_claimable_tax += 1,
            5 => issuer.sales[0].buyer_written_consent = false,
            6 => issuer.settlement_outside_business_income_and_expenses = false,
            7 => {
                business.fuel_tax_credit.as_mut().unwrap().expense_treatment =
                    FederalFuelExpenseTreatment::FullCostDeducted
            }
            8 => business.cash_fuel_tax_recovery = Some(cash_fuel_recovery(10_000)),
            9 => {
                business.material_participation =
                    crate::ParticipationFacts { hours: 0, test: crate::ParticipationTest::None }
            }
            10 => {
                business.activity = BusinessActivity::Ordinary {
                    returns: 0,
                    receipts: 4_000_000,
                    accounting: BusinessAccounting::Accrual,
                }
            }
            11 => {
                issuer.sales[0].vendor_settlement =
                    FederalCardVendorSettlement::VendorWrittenConsent
            }
            _ => unreachable!(),
        }
        let mut draft = complete(FederalStatus::Single);
        add(&mut draft, income(FederalIncome::Business { data: business }));
        if case == 9 {
            // 2026-10-24: passive C remains; the retained-claim owner still
            // requires its native active source and withholds the estimate.
            let result = estimate(&draft);
            assert_eq!(result.state, ResultState::Unsupported);
            assert!(result.issues.iter().any(|issue| issue.display.key == "ISSUE_CONFIRM_THE_U_S_SCHEDULE_C_OR_FARM_ACTIVITY_TAX_PAID_QUALIFIED_USE_5EDA324A"));
            assert!(result.tax.is_none());
            continue;
        }
        fuel_not_claimed(&draft);
    }
}

#[test]
fn registered_card_issuer_buyer_consent_requires_refund_and_source_records() {
    let good = consented_card_issuer_claim("guard-consent", 100_000);
    assert!(good.valid() && good.qualified());
    for case in 0..11 {
        let mut claim = good.clone();
        match case {
            0 => claim.sales[0].buyer_written_consent = false,
            1 => claim.sales[0].claimable_tax_refunded_in_full = false,
            2 => claim.sales[0].buyer_tax_refund_on = None,
            3 => claim.sales[0].buyer_tax_refund_on = Some(20270101),
            4 => claim.sales[0].buyer_tax_refund_on = Some(20260101),
            5 => claim.sales[0].tax_not_collected_from_buyer = true,
            6 => claim.settlement_outside_business_income_and_expenses = false,
            7 => claim.sales[0].vendor_settlement = FederalCardVendorSettlement::NotSure,
            8 => claim.sales[0].buyer_certificate_current = false,
            9 => claim.registration_active = false,
            10 => claim.sales[0].not_claimed_elsewhere = false,
            _ => unreachable!(),
        }
        assert!(!claim.valid() || !claim.qualified(), "case {case}");
    }

    // Written consent without the separate accounting for retained buyer tax
    // must not silently produce a Form 1040 refundable credit.
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
        unreachable!()
    };
    let mut retained_tax = good;
    retained_tax.sales[0].claimable_tax_refunded_in_full = false;
    business.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines { card_issuer: Some(retained_tax), ..FederalFuelLines::default() }
            .into(),
        expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
        records_complete: true,
    });
    add(&mut draft, income(FederalIncome::Business { data: business }));
    fuel_left_out(&draft);
}

#[test]
fn registered_card_issuer_no_collection_rejects_consent_and_refund_markers() {
    let good = card_issuer_claim("no-collection", 100_000);
    assert!(good.valid() && good.qualified());
    for case in 0..3 {
        let mut claim = good.clone();
        match case {
            0 => claim.sales[0].buyer_written_consent = true,
            1 => claim.sales[0].buyer_tax_refund_on = Some(20261231),
            2 => claim.sales[0].claimable_tax_refunded_in_full = true,
            _ => unreachable!(),
        }
        assert!(!claim.valid() || !claim.qualified(), "case {case}");
    }
}

#[test]
fn registered_card_issuer_guards_actual_source_and_no_duplicate_claim() {
    let good = card_issuer_claim("guard", 100_000);
    assert!(good.valid() && good.qualified());
    for arrangement in [
        FederalCardVendorSettlement::VendorWrittenConsent,
        FederalCardVendorSettlement::OtherDocumentedReimbursement,
    ] {
        let mut alternative = good.clone();
        alternative.sales[0].vendor_settlement = arrangement;
        assert!(alternative.valid() && alternative.qualified());
    }
    for case in 0..12 {
        let mut claim = good.clone();
        match case {
            0 => claim.registration_active = false,
            1 => claim.records_complete = false,
            2 => claim.settlement_outside_business_income_and_expenses = false,
            3 => claim.sales[0].taxed_at_stated_rate = false,
            4 => claim.sales[0].issuer_card_used_by_buyer = false,
            5 => claim.sales[0].exclusive_government_use = false,
            6 => claim.sales[0].tax_not_collected_from_buyer = false,
            7 => claim.sales[0].vendor_settlement = FederalCardVendorSettlement::NotSure,
            8 => claim.sales[0].buyer_certificate_current = false,
            9 => claim.sales[0].not_claimed_elsewhere = false,
            10 => claim.cc_registration_number = "UV-WRONG-ACTIVITY".into(),
            11 => claim.sales[1].sale_reference = claim.sales[0].sale_reference.clone(),
            _ => unreachable!(),
        }
        assert!(!claim.valid() || !claim.qualified(), "case {case}");
    }
    for case in [0, 2] {
        let mut claim = good.clone();
        match case {
            0 => claim.sales[0].date = 20270201,
            2 => claim.sales[0].actual_fuel_cost = 0,
            _ => unreachable!(),
        }
        assert!(!claim.valid(), "source case {case}");
    }
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Business { data: mut business } = cash_business(10_000) else {
        unreachable!()
    };
    business.fuel_tax_credit = Some(FederalFuelTaxCredit {
        fuels: FederalFuelLines { card_issuer: Some(good), ..FederalFuelLines::default() }.into(),
        expense_treatment: FederalFuelExpenseTreatment::TaxExcludedFromExpenses,
        records_complete: true,
    });
    add(&mut draft, income(FederalIncome::Business { data: business }));
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.card_issuer.as_mut().unwrap().sales[0]
        .buyer_certificate_current = false;
    fuel_left_out(&draft);

    let mut partner_draft = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(20_000, true);
    partner.qbi = Some(partner_qbi_statement(20_000));
    partner.fuel_tax_credit = Some(PartnerFuelTaxCredit {
        fuels: FederalFuelLines {
            card_issuer: Some(card_issuer_claim("partner-guard", 100_000)),
            ..FederalFuelLines::default()
        }
        .into(),
        statement_complete: false,
        farm_activity: false,
    });
    add(&mut partner_draft, income(FederalIncome::Partnership { data: partner }));
    let r = supported(&partner_draft);
    assert!(noted(&r, "WARN_PARTNER_FUEL_CREDIT_LEFT_OUT"), "{r:?}");
}

// C-fixture Schedule 1 (matrix row 3, 2026-09-25): one filer with a W-2,
// current-year unemployment compensation, educator expenses, student-loan
// interest and a 1099-INT early-withdrawal penalty. Every expected amount
// below is computed by hand from the 2026 draft Schedule 1/i1040gi, never
// copied from the App. Schedule 1 lines 11 and 21 (educator, student loan)
// have no dedicated line id in the App; only the aggregate `adjustments`
// line (Schedule 1 line 26) is exposed, per coverage matrix row 33.
#[test]
fn schedule_1_typical_return_lines_match_hand_computation() {
    // Facts: W-2 box 1 $52,000, box 2 $4,100, boxes 3-6 consistent (6.2%/1.45%
    // of box 1, both under their wage bases); 1099-G $6,000 unemployment, $600
    // withheld, no repayment; $500 educator expenses (single, one educator);
    // $3,000 student-loan interest paid; 1099-INT box 1 $400, box 2 $150.
    //
    // Schedule 1:
    //   Line 7  unemployment, no repayment:                       $6,000
    //   Line 10 Part I total (only line 7 is nonzero):             $6,000
    //   Line 11 educator, min($500, 2026 $350 cap):                  $350
    //   Line 18 early-withdrawal penalty (box 2):                    $150
    //   Line 21 student loan, min($3,000, $2,500 cap); MAGI
    //           $57,900 ($52,000+$400+$6,000-$350-$150) is under
    //           the 2026 single phase-out start of $85,000 (Rev.
    //           Proc. 2025-32/IRB 2025-45; matrix row 33), so the
    //           full cap applies:                                $2,500
    //   Line 26 (11+18+21 = 350+150+2,500):                       $3,000
    //
    // Form 1040:
    //   1a  W-2 wages:                                              $52,000
    //   2b  Taxable interest (box 2 penalty does not reduce it):       $400
    //   8   Additional income (Schedule 1 line 10):                 $6,000
    //   9   Total income (1a+2b+8 = 52,000+400+6,000):              $58,400
    //   10  Adjustments (Schedule 1 line 26):                        $3,000
    //   11  AGI (9-10 = 58,400-3,000):                              $55,400
    //   12e Standard deduction (single, 2026):                      $16,100
    //   15  Taxable income (11-12e = 55,400-16,100):                $39,300
    //   16  Tax: 2026 single Tax Table row $39,300-$39,350, midpoint
    //       $39,325: 10% of the first $12,400 ($1,240) plus 12% of
    //       ($39,325-$12,400 = $26,925 -> $3,231):                   $4,471
    //   24a Total tax (no credits or other taxes):                   $4,471
    //   25a W-2 withholding:                                         $4,100
    //   25b 1099 withholding (unemployment):                           $600
    //   25d Total withholding (25a+25b):                             $4,700
    //   33  Total payments (no other payments/credits):              $4,700
    //   34  Overpayment (33-24a = 4,700-4,471):                        $229
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 5_200_000, 410_000);
    add(&mut d, government_unemployment(6000, &[], 600));
    add(&mut d, income(savings_income(400, 150)));
    let a = d.adjustments.as_mut().unwrap();
    a.set_educator_taxpayer(50_000);
    a.set_student_loan_interest(300_000);
    let r = supported(&d);
    assert_eq!(value(&r, "unemployment_2_taxable"), 6000);
    assert_eq!(value(&r, "savings_withdrawal_penalty"), 150);
    assert_eq!(value(&r, "adjustments"), 3000);
    assert_eq!(value(&r, "agi"), 55400);
    assert_eq!(value(&r, "deduction"), 16100);
    assert_eq!(value(&r, "taxable_income"), 39300);
    assert_eq!(return_line(&r, "1a"), 5_200_000);
    assert_eq!(return_line(&r, "2b"), 40_000);
    assert_eq!(return_line(&r, "8"), 600_000);
    assert_eq!(return_line(&r, "9"), 5_840_000);
    assert_eq!(return_line(&r, "10"), 300_000);
    assert_eq!(return_line(&r, "11a / 11b"), 5_540_000);
    assert_eq!(return_line(&r, "12e"), 1_610_000);
    assert_eq!(return_line(&r, "15"), 3_930_000);
    assert_eq!(return_line(&r, "16"), 447_100);
    assert_eq!(return_line(&r, "24a"), 447_100);
    assert_eq!(return_line(&r, "25a"), 410_000);
    assert_eq!(return_line(&r, "25b"), 60_000);
    assert_eq!(return_line(&r, "25d"), 470_000);
    assert_eq!(return_line(&r, "33"), 470_000);
    assert_eq!(return_line(&r, "34"), 22_900);
    assert_eq!(r.tax, Some(447_100));
    assert_eq!(r.payments, Some(470_000));
    assert_eq!(r.refund, Some(22_900));
    assert_eq!(r.balance, Some(0));
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(supported(&restart(d).draft).tax, r.tax);
}

#[test]
fn educator_expenses_cap_each_educator_separately_on_a_joint_return() {
    // Joint return, two educators, each with $400 of eligible expenses. The
    // 2026 per-educator cap is $350 (Schedule 1 line 11 map, i1040gi), so
    // each educator's excess is disallowed separately: min($400,$350) +
    // min($400,$350) = $700. A single combined $350 cap on the total would
    // instead give $350; a combined $800 (no cap at all) would also be wrong.
    let mut d = complete(FederalStatus::Joint);
    d.adjustments.as_mut().unwrap().educators = EducatorClaim {
        taxpayer: Some(EducatorRole::Educator { classroom: 40_000, instructional: 0 }),
        spouse: Some(EducatorRole::Educator { classroom: 40_000, instructional: 0 }),
    };
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 700);
    assert_eq!(value(&r, "agi"), 99_300);
}

#[test]
fn student_loan_interest_phases_out_by_2026_magi() {
    // Single filer, $90,000 of wages and no other income or adjustments, so
    // MAGI for this deduction equals AGI before the student-loan deduction:
    // $90,000. The 2026 single phase-out is $85,000-$100,000 (Rev. Proc.
    // 2025-32/IRB 2025-45; matrix row 33, Form 1098-E). $3,000 of interest
    // paid is first capped at $2,500, then reduced by the fraction of the
    // phase-out range already used:
    //   remaining room = $100,000 - $90,000 = $10,000 of the $15,000 range
    //   deduction = $2,500 * $10,000 / $15,000 = $1,666.67 -> $1,667
    // (the App's `fraction` helper rounds by adding half the denominator
    // before truncating, i.e. ordinary round-half-up).
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 9_000_000, 0);
    d.adjustments.as_mut().unwrap().student_loan = StudentLoanClaim {
        student_loan_interest: 300_000,
        forms: Vec::new(),
        employer_paid_interest: 0,
    };
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 1_667);
    assert_eq!(value(&r, "agi"), 88_333);
}

#[test]
fn early_withdrawal_penalty_deducts_on_line_18_without_reducing_interest() {
    // The default single wage is $60,000 (`complete`). Add a 1099-INT with
    // $400 box 1 interest and a $150 box 2 early-withdrawal penalty. The
    // penalty is a Schedule 1 line 18 adjustment, not a reduction of the
    // box 1 interest reported on Form 1040 line 2b: the full $400 stays on
    // 2b, and the $150 only lowers AGI through the separate adjustment.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(savings_income(400, 150)));
    let r = supported(&d);
    assert_eq!(return_line(&r, "2b"), 40_000);
    assert_eq!(value(&r, "savings_withdrawal_penalty"), 150);
    assert_eq!(value(&r, "adjustments"), 150);
    assert_eq!(value(&r, "agi"), 60_250);
}

#[test]
fn taxable_grants_and_agriculture_payments_reach_their_lines() {
    // Box 6: a $2,500 taxable grant with no farm activity anywhere on the
    // return reaches Schedule 1 line 8z. No other line-8 item is present, so
    // the "Additional income" total equals the grant alone.
    let mut grants = complete(FederalStatus::Single);
    let mut grant = government_unemployment(0, &[], 0);
    let FederalIncome::GovernmentPayment { data } = &mut grant.income else { panic!() };
    data.form.taxable_grants = Some(250_000); // $2,500.00
    // 2026-10-23: this fixture establishes a personal grant, not an unclaimed activity grant.
    data.grant_use = Some(GrantUse::You);
    add(&mut grants, grant);
    let r = supported(&grants);
    assert_eq!(value(&r, "taxable_grants_1099g"), 2_500);
    assert_eq!(value(&r, "unemployment_2_taxable_grants"), 2_500);
    assert_eq!(return_line(&r, "8"), 250_000);

    // Box 7: a $4,000 agriculture payment routes into the existing farm
    // program-payment field's total when a farm activity exists for the same
    // owner. `raised_farm(10_000, 3_000)` reports a $7,000 Schedule F net
    // profit ($10,000 raised-product receipts less $3,000 expenses, i1040sf);
    // adding the $4,000 box 7 payment brings the combined business income
    // reaching Form 1040 line 8 ("Additional income") to $11,000.
    let mut farm = complete(FederalStatus::Single);
    add(&mut farm, income(raised_farm(10_000, 3_000)));
    let mut program = government_unemployment(0, &[], 0);
    let FederalIncome::GovernmentPayment { data } = &mut program.income else { panic!() };
    data.form.agriculture_payments = Some(400_000); // $4,000.00
    add(&mut farm, program);
    let r = supported(&farm);
    assert_eq!(value(&r, "unemployment_3_agriculture_payment"), 4_000);
    assert_eq!(return_line(&r, "8"), (7_000 + 4_000) * 100);

    // Box 7 with no farm activity anywhere on the return is refused by name:
    // there is no existing farm program-payment field to route it into.
    let mut no_farm = complete(FederalStatus::Single);
    let mut program = government_unemployment(0, &[], 0);
    let FederalIncome::GovernmentPayment { data } = &mut program.income else { panic!() };
    data.form.agriculture_payments = Some(100_000); // $1,000.00
    add(&mut no_farm, program);
    let r = estimate(&no_farm);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    missing_related_activity(
        &no_farm,
        &r,
        crate::related_forms::DocumentKind::Form1099GAgriculture,
        &[TaxSection::Farming],
    );
}

#[test]
fn family_leave_benefits_reach_line_8z_and_medical_leave_is_taxed_in_full() {
    // Box 10: a $3,000 family-leave-only amount (no medical-leave portion)
    // is fully includible on Schedule 1 line 8z under Rev. Rul. 2025-4.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, government_unemployment(0, &[], 0));
    let FederalIncome::GovernmentPayment { data } = &mut d.income[1].income else { panic!() };
    data.form.family_leave_benefits = 300_000; // $3,000.00
    data.family_leave_medical = false;
    let r = supported(&d);
    assert_eq!(value(&r, "family_leave_1099g"), 3_000);
    assert_eq!(value(&r, "unemployment_2_family_leave"), 3_000);
    assert_eq!(return_line(&r, "8"), 300_000);

    // A box 10 amount that includes (or may include) a medical-leave portion
    // is taxed in full with a note: the excludable medical-leave share needs
    // facts (the employee's own paid-medical-leave contributions) the App lacks.
    {
        let answer = true;
        let FederalIncome::GovernmentPayment { data } = &mut d.income[1].income else { panic!() };
        data.family_leave_medical = answer;
        let r = supported(&d);
        assert_eq!(value(&r, "unemployment_2_family_leave"), 3_000);
        assert!(noted(&r, "WARN_FAMILY_LEAVE_TAXED_IN_FULL"));
    }
}

#[test]
fn state_refund_withholding_reaches_line_25b_once() {
    // A `TaxRefund` document with no prior-year itemized deduction
    // (`NotDeducted`) is never taxable regardless of amount, isolating its
    // box 4 withholding: $5,000 reaches Form 1040 line 25b exactly once,
    // separate from the $6,500 wage withholding on line 25a (`complete`).
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(refund_income(100_000, 500_000, TaxRefundHistory::NotDeducted)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(return_line(&r, "25a"), 650_000);
    assert_eq!(return_line(&r, "25b"), 500_000);
    assert_eq!(return_line(&r, "25d"), 1_150_000);
}

#[test]
fn rtaa_is_other_income_and_market_gain_stays_refused() {
    let mut base = complete(FederalStatus::Single);
    add(&mut base, government_unemployment(0, &[], 0));

    // Box 5 RTAA payments are Schedule 1 line 8z other income.
    let mut rtaa = base.clone();
    let FederalIncome::GovernmentPayment { data } = &mut rtaa.income[1].income else { panic!() };
    data.form.rtaa_payments = Some(100_000); // $1,000.00
    let r = supported(&rtaa);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope, "{r:?}");
    assert_eq!(value(&r, "other_income_2_rtaa"), 1_000);
    assert_eq!(value(&r, "agi"), value(&supported(&base), "agi") + 1_000);

    // Box 9 market gain cannot be reconciled to the return's own CCC
    // calculation and is refused by name.
    let mut market = base.clone();
    let FederalIncome::GovernmentPayment { data } = &mut market.income[1].income else { panic!() };
    data.form.market_gain = Some(100_000); // $1,000.00
    let r = estimate(&market);
    assert_eq!(r.state, ResultState::Unsupported, "{r:?}");
    assert!(r.issues.iter().any(|i| i.message.contains("box 9")));
}

#[test]
fn state_refund_is_taxable_up_to_the_prior_year_benefit() {
    // Pub 525's recovery-of-prior-deduction worksheet: the prior year's
    // itemized deductions ($17,000) exceeded that year's standard
    // deduction ($16,100) by only $900, so recovering $2,000 of previously
    // deducted state income tax cannot restore more benefit than that
    // $900, even though the full $2,000 of tax was lost.
    //
    // By hand, following `TaxRefundHistory::taxable()`'s worksheet:
    // other_taxes = salt_paid($5,000) - income_paid($5,000) = $0.
    // revised_taxes = max(income_paid($5,000) - refund($2,000), sales_available($0)) + other_taxes($0) = $3,000.
    // lost_taxes = max(salt_deducted($5,000) - revised_taxes($3,000), 0) = $2,000.
    // lost_deduction = lost_taxes (no 2013-2017 overall limit) = $2,000.
    // taxable = min(lost_deduction($2,000), itemized($17,000) - standard($16,100) = $900) = $900.
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(refund_income(
            200_000,
            0,
            TaxRefundHistory::IncomeTax {
                deduction_year: 2025,
                income_paid: 500_000, // $5,000.00 state income tax paid that year
                sales_available: 0,
                salt_paid: 500_000,
                salt_deducted: 500_000, // fully allowed; no SALT cap effect
                itemized: 1_700_000,    // $17,000.00 prior total itemized deductions
                standard: 1_610_000,    // $16,100.00 that year's standard deduction
                overall_limit: None,
                property_refunded: 0,
                ordinary: true,
            },
        )),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "taxable_state_tax_refund"), 900);
}

#[test]
fn agriculture_payment_shows_in_the_farm_lines_once() {
    // The named fix for unit C23a: box 7 agriculture-program payments
    // already reached the owner's SE-tax and QBI totals through
    // `farm_1099g_program` (federal_rules.rs), but not the farm's own
    // displayed Schedule F lines (farm.rs `FarmIncome::report`).
    // `raised_farm(10_000, 3_000)` reports $10,000 raised-product receipts
    // less $3,000 expenses (i1040sf); adding a $4,000 box 7 payment now
    // brings this farm's own displayed gross income to
    // $10,000 + $4,000 = $14,000 and its own net profit to
    // $7,000 + $4,000 = $11,000, matching the $11,000 that already reached
    // Form 1040 line 8 before the fix, so the per-farm breakdown agrees
    // with the totals and the payment is counted once.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(raised_farm(10_000, 3_000)));
    let farm_id = d.income[1].id;
    let mut program = government_unemployment(0, &[], 0);
    let FederalIncome::GovernmentPayment { data } = &mut program.income else { panic!() };
    data.form.agriculture_payments = Some(400_000); // $4,000.00
    add(&mut d, program);
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("farm_{farm_id}_gross_income")), 14_000);
    assert_eq!(value(&r, &alloc::format!("farm_{farm_id}_net_profit")), 11_000);
    // The combined business total reaching Form 1040 line 8 still counts
    // the $4,000 payment once: $7,000 net profit + $4,000 payment =
    // $11,000, unchanged by the fix (only the farm's own display lines
    // changed).
    assert_eq!(return_line(&r, "8"), 11_000 * 100);
}

// T2-33f fixture (matrix row 33, Form 1098-E, 2026-09-26): the student-loan
// interest deduction. Every expected amount below is computed by hand from
// the 2026 draft Schedule 1 line 21 (i1040gi) and the 2026 phase-out amounts
// (Rev. Proc. 2025-32/IRB 2025-45), never copied from the App. These tests
// enter the interest as the claim's no-form amount (`student_loan_interest`),
// with each form's own amount shown in the comment; since T2-33x the claim
// also stores the Forms 1098-E themselves (see the T2-33x tests below).

#[test]
fn single_filer_two_1098es_sum_before_the_cap_and_magi_phaseout() {
    // Two Forms 1098-E for one single filer: $1,400.00 and $1,600.00, which
    // must add to $3,000.00 before either the $2,500 cap or the phase-out
    // applies (a bug that used only one form, or the smaller/larger of the
    // two, would give a different capped amount here since $1,600 and
    // $1,400 are each already below the $2,500 cap alone).
    //
    // Wages (only income, no other adjustments) $92,000, so MAGI for this
    // deduction equals AGI before it: $92,000. The 2026 single phase-out is
    // $85,000-$100,000 (a $15,000 range; Rev. Proc. 2025-32/IRB 2025-45;
    // matrix row 33), and $92,000 is inside it:
    //   interest capped:             min($3,000, $2,500) = $2,500
    //   remaining phase-out room:     $100,000 - $92,000  = $8,000 of $15,000
    //   deduction:  $2,500 * $8,000 / $15,000 = $1,333.33 -> $1,333
    //   (the App's `fraction` helper rounds by adding half the denominator
    //   before truncating, i.e. ordinary round-half-up)
    //   AGI: $92,000 - $1,333 = $90,667
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 9_200_000, 0);
    let form_1 = 140_000; // $1,400.00, Form 1098-E #1
    let form_2 = 160_000; // $1,600.00, Form 1098-E #2
    d.adjustments.as_mut().unwrap().set_student_loan_interest(form_1 + form_2);
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 1_333);
    assert_eq!(value(&r, "agi"), 90_667);
}

#[test]
fn joint_return_at_the_2500_cap_below_the_phaseout() {
    // Joint return, wages $150,000 (only income), student-loan interest
    // $2,800.00 — above the $2,500 cap, so the cap alone limits the
    // deduction here, not the phase-out. The 2026 joint phase-out starts at
    // $175,000 (matrix row 33), and MAGI $150,000 is below that start, so
    // the capped amount survives in full:
    //   interest capped: min($2,800, $2,500) = $2,500
    //   remaining phase-out room: $175,000 + $30,000 - $150,000 = $55,000,
    //   which is above the full $30,000 range, so the fraction is 30,000/
    //   30,000 = 1 (no reduction): deduction = $2,500
    //   AGI: $150,000 - $2,500 = $147,500
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 15_000_000, 0);
    d.adjustments.as_mut().unwrap().set_student_loan_interest(280_000);
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 2_500);
    assert_eq!(value(&r, "agi"), 147_500);
}

#[test]
fn joint_magi_phaseout_uses_the_175000_to_205000_range_and_formula() {
    // Joint return, wages $190,000 (only income) — the midpoint of the 2026
    // joint phase-out range $175,000-$205,000 (a $30,000 range; matrix row
    // 33). Student-loan interest $3,000.00, above the $2,500 cap:
    //   interest capped: min($3,000, $2,500) = $2,500
    //   remaining phase-out room: $175,000 + $30,000 - $190,000 = $15,000 of
    //   the $30,000 range (exactly half)
    //   deduction: $2,500 * $15,000 / $30,000 = $1,250.00 -> $1,250
    //   AGI: $190,000 - $1,250 = $188,750
    let mut d = complete(FederalStatus::Joint);
    d.income[0] = wage(1, Owner::Taxpayer, 19_000_000, 0);
    d.adjustments.as_mut().unwrap().set_student_loan_interest(300_000);
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 1_250);
    assert_eq!(value(&r, "agi"), 188_750);
}

#[test]
fn separate_return_excludes_the_student_loan_deduction_regardless_of_magi() {
    // Married filing separately: section 221(e)(2) disallows the deduction
    // entirely (matrix row 33, "MFS = 0"), regardless of MAGI or the amount
    // of interest paid. Wages $40,000 (well under even the single phase-out
    // start), interest $2,000.00 — inside the cap and range that would
    // otherwise apply to a single filer — but the deduction is $0 because
    // the return is Separate:
    //   AGI: $40,000 - $0 = $40,000
    let mut d = complete(FederalStatus::Separate);
    d.income[0] = wage(1, Owner::Taxpayer, 4_000_000, 0);
    d.adjustments.as_mut().unwrap().set_student_loan_interest(200_000);
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(value(&r, "agi"), 40_000);
}

#[test]
fn dependent_claimed_by_another_return_excludes_the_student_loan_deduction() {
    // Single filer actually claimed as a dependent on another return
    // (`FilerDependency::CanBeClaimed { claimed: true }`): Pub 970
    // disallows the deduction to a filer who is claimed as a dependent,
    // regardless of otherwise-eligible interest and a MAGI well under the
    // $85,000 phase-out start. Wages $50,000, interest $2,000.00:
    //   Not claimed (Answer::No): deduction min($2,000, $2,500) = $2,000,
    //     AGI $50,000 - $2,000 = $48,000
    //   Claimed (Answer::Yes): deduction $0, AGI $50,000 - $0 = $50,000
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 5_000_000, 0);
    d.adjustments.as_mut().unwrap().set_student_loan_interest(200_000);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: false };
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 2_000);
    assert_eq!(value(&r, "agi"), 48_000);
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    let r = supported(&d);
    assert_eq!(value(&r, "adjustments"), 0);
    assert_eq!(value(&r, "agi"), 50_000);
}
