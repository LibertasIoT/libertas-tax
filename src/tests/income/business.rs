use super::*;
use crate::tests::support::client::roundtrip_native_draft;

#[test]
fn restaurant_tip_monthly_shortfall_cannot_exclude_more_than_its_tips() {
    let mut draft = restaurant_tip_credit_draft();
    let business = restaurant_business_mut(&mut draft);
    let claim = business.employer_tip_fica_credit.as_mut().unwrap();
    claim.employees[0].months[10] = EmployerTipMonth {
        hours_hundredths: 10_000,
        wages_excluding_tips: 0,
        reported_tips: 10_000,
    };
    let BusinessPayroll::Yes { statements, .. } = &mut business.payroll else { panic!() };
    statements[0].wages += 10_000;
    statements[0].medicare += 10_000;
    let result = supported(&draft);
    assert_eq!(value(&result, "business_2_form8846_line1"), 550);
    assert_eq!(value(&result, "business_2_form8846_line2"), 240);
    assert_eq!(value(&result, "business_2_form8846_line3"), 310);
    assert_eq!(value(&result, "business_2_form8846_line4"), 24);
    for printed in ["1", "2", "3", "4"] {
        assert_eq!(
            printed_line(&result, &alloc::format!("business_2_form8846_line{printed}")),
            (TaxForm::Form8846, printed.into())
        );
    }
}

#[test]
fn iso_spread_creates_amt_without_regular_income_and_persists() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic ISO", 2000.0, 1000, 11000)];
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 260000);
    assert_eq!(value(&r, "alternative_minimum_tax"), 39151);
    assert_eq!(r.tax, Some(4_417_400));
    assert_eq!(roundtrip_native_draft(d.clone()), d);
}

#[test]
fn missing_statutory_business_source_keeps_its_conservative_qbi_notice() {
    let mut d = empty_return(FederalStatus::Single);
    let mut entry = wage(0, Owner::Taxpayer, 40_000_000, 0);
    let FederalIncome::Wage { data } = &mut entry.income else { unreachable!() };
    data.statutory_employee = true;
    data.statutory_business = None;
    add(&mut d, entry);
    let result = supported(&d);
    assert!(noted(&result, "WARN_QBI_SPECIFIED_SERVICE_ASSUMED"));
    assert_eq!(value(&result, "qbi_deduction"), 0);
    assert_eq!(supported(&restart(d).draft), result);
}

#[test]
fn casual_gambling_is_unearned_and_losses_do_not_create_a_business_nol() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, gambling(10000, 50000));
    let r = supported(&d);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    assert_eq!(value(&r, "nol_carryforward"), 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(100000, 0, Answer::No);
    d.credits.as_mut().unwrap().saver = Some(saver(0, 0)).into();
    unsupported(&d, "IRA");
    d.adjustments.as_mut().unwrap().iras = FederalIras::NONE;
    d.credits.as_mut().unwrap().saver = None.into();
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100101;
    assert_eq!(value(&supported(&d), "kiddie_tax_net_unearned_income"), 0);
    add(&mut d, wage(0, Owner::Taxpayer, 2000000, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "kiddie_tax_direct_gambling_deduction"), 10000);
    assert_eq!(value(&r, "kiddie_tax_net_unearned_income"), 0);
    assert_eq!(value(&r, "nol_carryforward"), 0);
}

#[test]
fn hobby_goods_sales_use_inventory_cost_without_business_deductions_or_losses() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(hobby_sales(10000, 8000, 2000)));
    let r = supported(&d);
    assert_eq!(value(&r, "hobby_sales_income"), 4000);
    assert_eq!(value(&r, "agi"), 64000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(return_line(&r, "8"), 400000);
    assert_eq!(restart(d.clone()).draft, d);
    d.income[1].income = hobby_sales(1000, 8000, 2000);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "nol_carryforward"), 0);
}
#[test]
fn nonbusiness_royalties_flow_through_schedule_e_and_niit_without_se_or_qbi() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 21000000, 0);
    let mut source = royalty(30000, 5000);
    let FederalIncome::Royalty { data } = &mut source else { unreachable!() };
    data.expenses = (0..101)
        .map(|index| RentalExpense {
            category: RentalExpenseCategory::Other {
                label: alloc::format!("Actual royalty expense {index}"),
            },
            amount: if index == 100 { 5000 } else { 4950 },
            treatment: BusinessExpenseKind::Ordinary,
        })
        .collect();
    add(&mut d, income(source));
    let d = save_reopen_native_sources(d, &[TaxSection::Rentals], &[TaxSection::Rentals]);
    let r = supported(&d);
    assert_eq!(value(&r, "nonbusiness_royalty_income"), 25000);
    assert_eq!(value(&r, "agi"), 235000);
    assert_eq!(value(&r, "net_investment_income_tax"), 950);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(return_line(&r, "8"), 2500000);
    // Includes $90 of Additional Medicare withholding from the W-2; a royalty
    // carries no withholding of its own (a 1099-MISC form does).
    assert_eq!(r.payments, Some(9000));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn nonbusiness_royalty_losses_do_not_create_a_business_nol_or_earned_income() {
    let mut d = empty_return(FederalStatus::Single);
    add(&mut d, income(royalty(1000, 5000)));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), -4000);
    assert_eq!(value(&r, "current_net_operating_loss"), 0);
    assert_eq!(value(&r, "earned_income_credit"), 0);
    d.adjustments.as_mut().unwrap().iras = ira_facts(10000, 0, Answer::No);
    unsupported(&d, "IRA contributions");
}

#[test]
fn nonbusiness_royalty_net_income_supports_portfolio_interest_deduction() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(royalty(10000, 2000)));
    investment_borrowing(&mut d, 10000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 8000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 2000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 8000);
    assert_eq!(value(&r, "agi"), 68000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn qualifying_iso_sale_holding_periods_use_calendar_anniversaries() {
    let mut sale = qualifying_iso_sale();
    sale.exercised = 20250421;
    assert!(!sale.qualifying());
    sale.sold = 20260422;
    assert!(sale.qualifying());
    sale.granted = 20240422;
    assert!(!sale.qualifying());
    sale.sold = 20260423;
    assert!(sale.qualifying());
    sale.granted = 20240229;
    sale.exercised = 20240301;
    sale.sold = 20260228;
    assert!(!sale.qualifying());
    // No February 29 in 2026: the second anniversary is March 1, which is not yet "more than".
    sale.sold = 20260301;
    assert!(!sale.qualifying());
    sale.sold = 20260302;
    assert!(sale.qualifying());
}

#[test]
fn iso_sale_with_unsupported_special_treatment_is_left_out_with_a_notice() {
    let mut sale = qualifying_iso_sale();
    sale.granted = 20230101;
    sale.exercised = 20240201;
    sale.confirmed = false;
    let entry = FederalIncome::IsoSale { data: sale };
    assert_eq!(crate::interview::document_amount(&entry, false), None);
    let mut d = complete(FederalStatus::Single);
    let before = value(&supported(&d), "agi");
    add(&mut d, income(entry));
    let r = supported(&d);
    assert!(noted(&r, "WARN_ISO_SALE_LEFT_OUT"));
    assert_eq!(value(&r, "agi"), before);
}

#[test]
fn qualifying_iso_partial_lot_costs_and_withholding_are_counted_once() {
    let mut sale = qualifying_iso_sale();
    sale.shares = 10.0;
    sale.acquisition_costs = 2000;
    sale.proceeds = 1200000;
    sale.withholding = 10000;
    let entry = FederalIncome::IsoSale { data: sale };
    assert_eq!(crate::interview::document_amount(&entry, false), Some(1098000));
    let mut d = complete(FederalStatus::Single);
    let baseline = supported(&d);
    add(&mut d, income(entry));
    let r = supported(&d);
    assert_eq!(value(&r, "iso_sale_2_regular_basis"), 1020);
    assert_eq!(value(&r, "iso_sale_2_amt_basis"), 10020);
    assert_eq!(value(&r, "capital_gain"), 10980);
    assert_eq!(value(&r, "amt_capital_gain"), 1980);
    assert_eq!(r.payments, baseline.payments.map(|v| v + 10000));
}

#[test]
fn qualifying_iso_regular_gains_still_enter_net_investment_income() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30000000, 0);
    add(&mut d, income(FederalIncome::IsoSale { data: qualifying_iso_sale() }));
    let r = supported(&d);
    assert_eq!(value(&r, "net_investment_income_tax"), 1900);
    assert_eq!(value(&r, "amt_capital_gain"), -3000);
}

#[test]
fn iso_sale_source_validation_rejects_bad_dates_and_amounts() {
    let original = qualifying_iso_sale();
    assert!(original.valid());
    for change in 0..7 {
        let mut sale = original.clone();
        match change {
            0 => sale.granted = 20250229,
            1 => sale.exercised = 20220101,
            2 => sale.sold = 20270421,
            3 => sale.shares = 0.0,
            4 => sale.shares = 1000001.0,
            5 => sale.exercise_price = -1,
            _ => sale.market_price = 100000000000,
        }
        assert!(!sale.valid(), "case {change}");
        assert!(
            crate::federal_rules::validate_income(&income(FederalIncome::IsoSale { data: sale }))
                .is_err()
        );
    }
}

#[test]
fn disqualifying_iso_sale_computes_compensation_and_amt_ordinary_reversal() {
    let mut d = complete(FederalStatus::Single);
    let sale = disqualifying_iso_sale();
    assert_eq!(sale.compensation(), 5000000);
    assert_eq!(sale.sale().term, GainTerm::Short);
    add(&mut d, income(FederalIncome::IsoSale { data: sale }));
    let r = supported(&d);
    assert_eq!(value(&r, "iso_unreported_compensation"), 50000);
    assert_eq!(value(&r, "agi"), 110000);
    assert_eq!(return_line(&r, "1a"), 6000000);
    assert_eq!(return_line(&r, "8"), 5000000);
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, "amt_iso_compensation_adjustment"), -50000);
    assert_eq!(value(&r, "amt_capital_gain"), -3000);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 37000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 57000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn disqualifying_iso_w2_compensation_is_not_counted_twice() {
    let mut unreported = complete(FederalStatus::Single);
    add(&mut unreported, income(FederalIncome::IsoSale { data: disqualifying_iso_sale() }));
    let before = supported(&unreported);
    let mut reported = unreported.clone();
    let FederalIncome::Wage { data } = &mut reported.income[0].income else { panic!() };
    data.wages += 5000000;
    let FederalIncome::IsoSale { data } = &mut reported.income[1].income else { panic!() };
    data.in_wages = true;
    let r = supported(&reported);
    assert_eq!(value(&r, "iso_unreported_compensation"), 0);
    assert_eq!(return_line(&r, "1a"), 11000000);
    assert_eq!(return_line(&r, "8"), 0);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(r.tax, before.tax);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 57000);
    assert_eq!(restart(reported.clone()).draft, reported);
}

#[test]
fn same_year_iso_sale_has_compensation_but_no_amt_difference_or_payroll_tax() {
    let mut d = complete(FederalStatus::Single);
    let mut sale = disqualifying_iso_sale();
    sale.exercised = 20260201;
    sale.proceeds = 12000000;
    add(&mut d, income(FederalIncome::IsoSale { data: sale }));
    let r = supported(&d);
    assert_eq!(value(&r, "iso_unreported_compensation"), 90000);
    assert_eq!(value(&r, "capital_gain"), 20000);
    assert_eq!(value(&r, "amt_capital_gain"), 20000);
    assert_eq!(value(&r, "amt_iso_compensation_adjustment"), 0);
    assert_eq!(value(&r, "amt_capital_adjustment"), 0);
    assert_eq!(value(&r, "agi"), 170000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 170000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "additional_medicare_tax"), 0);
}

#[test]
fn iso_disqualification_preserves_long_term_capital_character() {
    let mut sale = disqualifying_iso_sale();
    sale.exercised = 20250107;
    sale.proceeds = 12000000;
    assert!(!sale.qualifying());
    assert_eq!(sale.compensation(), 9000000);
    assert_eq!(sale.sale().term, GainTerm::Long);
    assert_eq!(sale.sale().gain(), 2000000);
    assert_eq!(sale.sale().amt_gain(), 2000000);
}

#[test]
fn iso_disqualifying_sales_limit_compensation_before_selling_fees() {
    let mut sale = disqualifying_iso_sale();
    sale.proceeds = 5990000;
    sale.selling_fees = 10000;
    assert_eq!(sale.compensation(), 5000000);
    assert_eq!(sale.sale().gain(), -10000);
    sale.proceeds = 990000;
    assert_eq!(sale.compensation(), 0);
    assert_eq!(sale.sale().gain(), -10000);
    sale.proceeds = 800000;
    assert_eq!(sale.compensation(), 0);
    assert_eq!(sale.sale().gain(), -200000);
}

#[test]
fn iso_reported_compensation_validates_owner_and_aggregate_wages() {
    let mut zero = qualifying_iso_sale();
    zero.in_wages = true;
    assert!(!zero.valid());
    let mut d = complete(FederalStatus::Single);
    let mut sale = disqualifying_iso_sale();
    sale.in_wages = true;
    add(&mut d, income(FederalIncome::IsoSale { data: sale.clone() }));
    supported(&d); // One 50,000 compensation amount fits the 60,000 W-2.
    add(&mut d, income(FederalIncome::IsoSale { data: sale.clone() }));
    unsupported(&d, "ISO compensation"); // Two lots cannot both fit.
    let mut d = complete(FederalStatus::Joint);
    let mut entry = income(FederalIncome::IsoSale { data: sale });
    entry.owner = Owner::Spouse;
    add(&mut d, entry);
    unsupported(&d, "ISO compensation");
}

#[test]
fn young_filer_iso_wage_reversal_uses_statutory_amt_exemption() {
    let mut d = complete(FederalStatus::Single);
    let person = &mut d.people.as_mut().unwrap().taxpayer;
    person.birth_date = 20090615;
    person.parent_alive = true;
    add(&mut d, income(FederalIncome::IsoSale { data: disqualifying_iso_sale() }));
    let r = supported(&d);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 57000);
    assert_eq!(value(&r, "alternative_minimum_tax"), 0);
}

#[test]
fn actual_employee_tips_use_monthly_total_for_payroll_threshold() {
    let tips = EmployeeTips {
        cash: CashTipRecords::Actual {
            months: vec![
                CashTipMonth { month: 1, received: 1000000, reported: 700000 },
                CashTipMonth { month: 2, received: 1900, reported: 0 },
                CashTipMonth { month: 3, received: 2000, reported: 0 },
            ],
        },
        noncash: 10000,
        coverage: TipCoverage::SocialAndMedicare,
    };
    let draft = wage_with_employee_tips(60000, tips);
    let r = supported(&draft);
    assert_eq!(value(&r, "employee_tip_income"), 3139);
    assert_eq!(value(&r, "agi"), 63139);
    assert_eq!(value(&r, "employee_tips_0_social_tax"), 187);
    assert_eq!(value(&r, "employee_tips_0_medicare_tax"), 44);
    assert_eq!(r.tax, Some(562600));
    assert_eq!(restart(draft.clone()).draft, draft);
    let mut threshold = allocated_employee_tips(0);
    threshold.cash = CashTipRecords::Actual {
        months: vec![
            CashTipMonth { month: 1, received: 1999, reported: 0 },
            CashTipMonth { month: 2, received: 3000, reported: 2000 },
        ],
    };
    let result = threshold.calculate().unwrap();
    assert_eq!((result.income, result.social, result.medicare), (2999, 1000, 1000));
}

#[test]
fn employee_tip_income_feeds_earned_credit_and_qualified_deduction() {
    let tips = wage_with_employee_tips(0, allocated_employee_tips(10000));
    let reported = wage_with_employee_tips(10000, allocated_employee_tips(0));
    let a = supported(&tips);
    let b = supported(&reported);
    assert_eq!(value(&a, "earned_income_credit"), value(&b, "earned_income_credit"));
    assert!(value(&a, "earned_income_credit") > 0);
    let mut draft = wage_with_employee_tips(60000, allocated_employee_tips(10000));
    if let FederalIncome::Wage { data } = &mut draft.income[0].income {
        data.work_deductions = work_deductions(1000000, 0);
    }
    let r = supported(&draft);
    assert_eq!(value(&r, "agi"), 70000);
    assert_eq!(value(&r, "taxable_income"), 43900);
    assert_eq!(r.tax, Some(578800));
}

#[test]
fn employee_tip_deduction_takes_the_larger_reported_amount_per_employer() {
    let mut draft = wage_with_employee_tips(
        60000,
        EmployeeTips {
            cash: CashTipRecords::Actual {
                months: vec![CashTipMonth { month: 1, received: 1_000_000, reported: 700_000 }],
            },
            noncash: 0,
            coverage: TipCoverage::SocialAndMedicare,
        },
    );
    let FederalIncome::Wage { data } = &mut draft.income[0].income else { panic!() };
    data.work_deductions = FederalWorkDeductions {
        tips_claim: TipsClaim::Yes {
            w2_tips: 800_000,
            form4137_tips: 1_000_000,
            tips_eligible: true,
        },
        overtime_claim: OvertimeClaim::No,
    };
    let result = supported(&draft);
    assert_eq!(value(&result, "tips_deduction"), 10_000);
    assert_eq!(value(&result, "employee_tip_income"), 3000);
    assert_eq!(restart(draft.clone()).draft, draft);

    if let FederalIncome::Wage { data } = &mut draft.income[0].income
        && let TipsClaim::Yes { form4137_tips, .. } = &mut data.work_deductions.tips_claim
    {
        *form4137_tips = 700_000;
    }
    let lower = supported(&draft);
    assert_eq!(value(&lower, "tips_deduction"), 8000);
    assert_eq!(value(&lower, "agi"), value(&result, "agi"));
    if let FederalIncome::Wage { data } = &mut draft.income[0].income
        && let TipsClaim::Yes { form4137_tips, .. } = &mut data.work_deductions.tips_claim
    {
        *form4137_tips = 1_100_000;
    }
    // W-2 code TP backs $8,000; only the $3,000 beyond it needs the records'
    // unreported tips, which they show.
    assert_eq!(value(&supported(&draft), "tips_deduction"), 11_000);
    if let FederalIncome::Wage { data } = &mut draft.income[0].income
        && let TipsClaim::Yes { form4137_tips, .. } = &mut data.work_deductions.tips_claim
    {
        *form4137_tips = 1_200_000;
    }
    unsupported(&draft, "Income");
}

#[test]
fn employee_tip_records_reject_duplicate_months() {
    let mut tips = allocated_employee_tips(1000);
    tips.cash = CashTipRecords::Actual {
        months: vec![
            CashTipMonth { month: 1, received: 2000, reported: 0 },
            CashTipMonth { month: 1, received: 3000, reported: 0 },
        ],
    };
    assert!(!tips.valid());
    assert!(tips.calculate().is_err());
    tips.cash = CashTipRecords::Actual {
        months: vec![CashTipMonth { month: 1, received: 2000, reported: 2001 }],
    };
    assert!(!tips.valid());
}

#[test]
fn employee_tips_feed_document_totals_and_section179_compensation() {
    let mut draft = business_with_bonus(0, elected_asset(100000, 100000, BonusElection::ElectOut));
    draft.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    if let FederalIncome::Wage { data } = &mut draft.income[0].income {
        data.employee_tips = Some(allocated_employee_tips(10000));
    }
    assert_eq!(crate::interview::document_amount(&draft.income[0].income, false), Some(1000000));
    let r = supported(&draft);
    assert_eq!(value(&r, "section179_business_income_limit"), 10000);
    assert_eq!(value(&r, "section179_deduction"), 10000);
    assert_eq!(value(&r, "section179_carryforward"), 90000);
    assert_eq!(value(&r, "agi"), 0);
    assert_eq!(r.tax, Some(76500));
}

#[test]
fn uncollected_w2_taxes_enter_return_without_duplicating_wages() {
    for life in [false, true] {
        let draft = uncollected_wage(60000, 620, 145, life);
        let r = supported(&draft);
        assert_eq!(value(&r, "agi"), 60000);
        assert_eq!(value(&r, "w2_uncollected_payroll_tax"), 765);
        assert_eq!(return_line(&r, "1a"), 6000000);
        assert_eq!(return_line(&r, "1c"), 0);
        assert_eq!(return_line(&r, "23"), 76500);
        assert_eq!(return_line(&r, "25d"), 0);
        assert_eq!(r.tax, Some(578800));
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn uncollected_w2_taxes_coexist_with_new_unreported_tips() {
    let mut draft = uncollected_wage(60000, 620, 145, false);
    if let FederalIncome::Wage { data } = &mut draft.income[0].income {
        data.employee_tips = Some(allocated_employee_tips(5000));
    }
    let r = supported(&draft);
    assert_eq!(value(&r, "agi"), 65000);
    assert_eq!(value(&r, "employee_tips_0_social_tax"), 310);
    assert_eq!(value(&r, "employee_tips_0_medicare_tax"), 73);
    assert_eq!(return_line(&r, "23"), 114800);
    assert_eq!(r.tax, Some(677100));
}

#[test]
fn w2_box12_a_b_uncollected_tips_tax_reaches_schedule_2_line_13() {
    // Box 12 A/B (uncollected Social Security/Medicare tax on reported tips, the W-2 map's
    // "in scope" row 12): $620.00 code A, $145.00 code B, `complete: Yes`. i1040gi and
    // forms/w2.rs `collect` (`W2UncollectedTax`) route these as tax due, never as additional
    // wages or withholding already paid ("w2_uncollected_payroll_tax", Form W-2 line 12,
    // combined onto Schedule 2 line 13). By hand: $620.00 + $145.00 = $765.00. This
    // wage-only draft has no self-employment, household, Additional Medicare or other
    // Schedule 2 Part II tax, so that $765.00 is the entire Form 1040 line 23 additional-tax
    // total; box 1 wages ($60,000.00, line 1a) and box 2 withholding (line 25a) are
    // unaffected because the uncollected amounts were never actually paid as wages or
    // withheld.
    let draft = uncollected_wage(60_000, 620, 145, false);
    let r = supported(&draft);
    assert_eq!(value(&r, "w2_uncollected_payroll_tax"), 765);
    assert_eq!(return_line(&r, "1a"), 6_000_000);
    assert_eq!(return_line(&r, "23"), 76_500);
    assert_eq!(return_line(&r, "25a"), 0);
}

#[test]
fn w2_box8_allocated_tips_reach_line_1c_through_form_4137() {
    // Box 8 allocated tips $5,000.00 without adequate actual monthly records
    // (`CashTipRecords::Allocated`; employee_tips.rs `EmployeeTips::calculate` treats the
    // full allocated amount as unreported tip income and as covered Social Security/Medicare
    // tips). By hand: $5,000.00 is added to Form 1040 line 1c (Form 4137 unreported tips),
    // not line 1a, which keeps the $60,000.00 box 1 wages. Form 4137 employee payroll tax
    // (`employee_tips::payroll`): the $60,000.00 of already-covered box 3 wages leaves
    // $124,500.00 of the $184,500.00 2026 Social Security wage base, so all $5,000.00 of
    // tips is covered; 6.2% x $5,000.00 = $310.00 Social Security, 1.45% x $5,000.00 =
    // $72.50 rounded to $73.00 Medicare.
    let draft = wage_with_employee_tips(60_000, allocated_employee_tips(5_000));
    let r = supported(&draft);
    assert_eq!(return_line(&r, "1a"), 6_000_000);
    assert_eq!(return_line(&r, "1c"), 500_000);
    assert_eq!(value(&r, "employee_tips_0_social_tax"), 310);
    assert_eq!(value(&r, "employee_tips_0_medicare_tax"), 73);
}

#[test]
fn statutory_employee_w2_retains_other_business_se_tax() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(10_000)));
    // Hand computation (i1040sse): net earnings = $10,000 * 92.35% = $9,235.
    // The $60,000 of W-2 wages in `complete()` leaves $124,500 of Social
    // Security wage-base headroom, above the $9,235 net earnings, so the
    // full 12.4% + 2.9% = 15.3% combined rate applies:
    // $9,235 * 15.3% = $1,412.955, rounds to $1,413.
    let baseline = value(&supported(&d), "self_employment_tax");
    assert_eq!(baseline, 1413);

    // The $60,000 statutory job belongs on Schedule C, without SE tax.
    // The separate $10,000 ordinary business retains its $1,413 SE tax.
    if let FederalIncome::Wage { data } = &mut d.income[0].income {
        data.statutory_employee = true;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "self_employment_tax"), baseline);
    assert_eq!(return_line(&r, "1a"), 0);
    assert_eq!(value(&r, "agi"), 69_293);

    // Unset again leaves today's result unchanged.
    {
        let FederalIncome::Wage { data } = &mut d.income[0].income else { unreachable!() };
        data.statutory_employee = false;
    }
    assert_eq!(value(&supported(&d), "self_employment_tax"), baseline);
}

#[test]
fn espp_source_records_apply_qualifying_and_early_sale_compensation() {
    for (exercised, compensation, basis) in [(20250101, 200, 2200), (20260101, 300, 2300)] {
        for in_wages in [false, true] {
            let mut d = complete(FederalStatus::Single);
            d.income[0] = wage(
                1,
                Owner::Taxpayer,
                (60000 + if in_wages { compensation } else { 0 }) * 100,
                0,
            );
            let lot = EsppSale {
                granted: 20230101,
                exercised,
                sold: 20260201,
                shares: 100.0,
                grant_market: 2200,
                grant_price: 2000,
                exercise_price: 2000,
                market_price: 2300,
                acquisition_costs: 0,
                proceeds: 300000,
                selling_fees: 0,
                in_wages,
                wash_sale: 0,
                withholding: 0,
                reported_basis: None,
                reporting: crate::BasisReporting::ReportedToIrs,
                special: false,
            };
            assert_eq!(lot.compensation(), compensation * 100);
            assert_eq!(lot.sale().basis_amount(), basis * 100);
            add(&mut d, income(FederalIncome::EsppSale { data: lot }));
            let r = supported(&d);
            assert_eq!(value(&r, "agi"), 61000);
            assert_eq!(value(&r, "capital_gain"), 3000 - basis);
            assert_eq!(supported(&restart(d).draft), r);
        }
    }
}

#[test]
fn espp_loss_fees_wash_and_holding_period_boundaries() {
    for (early, wash, compensation, gain, agi) in
        [(false, 0, 0, -500, 59500), (true, 0, 300, -800, 59500), (true, 30000, 300, -500, 59800)]
    {
        let mut lot = ordinary_espp_lot();
        if early {
            lot.exercised = 20260101;
        }
        lot.proceeds = 150000;
        lot.wash_sale = wash;
        assert_eq!(lot.compensation(), compensation * 100);
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::EsppSale { data: lot }));
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), gain);
        assert_eq!(value(&r, "agi"), agi);
        assert_eq!(supported(&restart(d).draft), r);
    }
    let mut lot = ordinary_espp_lot();
    lot.proceeds = 205000;
    lot.selling_fees = 5000;
    lot.acquisition_costs = 2000;
    assert_eq!(lot.compensation(), 10000);
    assert_eq!(lot.sale().basis_amount(), 212000);
    assert_eq!(lot.sale().gain(), -7000);
    lot = ordinary_espp_lot();
    lot.sold = 20260101;
    assert!(!lot.qualifying());
    assert_eq!(lot.sale().term, GainTerm::Short);
    lot.sold = 20260102;
    assert!(lot.qualifying());
    lot.granted = 20240102;
    assert!(!lot.qualifying());
    lot.sold = 20260103;
    assert!(lot.qualifying());
    lot.wash_sale = 1;
    assert!(!lot.valid());
}

#[test]
fn espp_fractional_shares_normalize_before_money_calculation() {
    for exercised in [20250101, 20260101] {
        let mut lot = ordinary_espp_lot();
        lot.shares = 0.125;
        lot.exercised = exercised;
        lot.proceeds = 375;
        lot.in_wages = true;
        assert!(lot.valid());
        let expected_compensation = if exercised == 20250101 { 25 } else { 38 };
        assert_eq!(lot.compensation(), expected_compensation);
        assert_eq!(lot.sale().basis_amount(), 250 + expected_compensation);
        let mut d = complete(FederalStatus::Single);
        let sale = lot.sale();
        add(&mut d, income(FederalIncome::EsppSale { data: lot }));
        let r = supported(&d);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        d.income[1].income = FederalIncome::Sale { data: sale };
        let comparison = supported(&d);
        assert_eq!(r.tax, comparison.tax);
        assert_eq!(value(&r, "agi"), value(&comparison, "agi"));
    }
    let mut lot = ordinary_espp_lot();
    for quantity in [0.000001, 0.1, 1.234567, 999999.999999, 1000000.0] {
        lot.shares = quantity;
        // Bound checks on money totals are independent of quantity normalization.
        lot.exercise_price = 0;
        lot.market_price = 0;
        lot.grant_price = 0;
        lot.grant_market = 0;
        assert!(lot.valid(), "{quantity}");
    }
    for quantity in [
        0.0,
        -1.0,
        0.0000001,
        1.2345678,
        1000000.000001,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        lot.shares = quantity;
        assert!(!lot.valid(), "{quantity}");
    }
}

#[test]
fn fractional_iso_held_shares_feed_amt_without_regular_income() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().iso_exercises =
        vec![iso_exercise("Synthetic fractional ISO", 2000.125, 1000, 11000)];
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    // $200,012.50 spread rounds to $200,013 on the return.
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 260013);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    let lot = &mut d.screening.as_mut().unwrap().iso_exercises[0];
    lot.shares = 1.0;
    lot.exercise_price = 0;
    lot.market_price = 20001250;
    assert_eq!(supported(&d), r);
    for quantity in [0.0000001, 1.2345678, f64::NAN, f64::INFINITY] {
        d.screening.as_mut().unwrap().iso_exercises[0].shares = quantity;
        assert!(!crate::extended_rules::valid_exercises(
            &d.screening.as_ref().unwrap().iso_exercises
        ));
    }
}

#[test]
fn restricted_iso_recognition_uses_current_year_event_and_event_value() {
    for recognition in [
        IsoRecognition::Exercise,
        IsoRecognition::Vesting { date: 20260601, market_price: 11000 },
        IsoRecognition::Election,
    ] {
        let mut d = complete(FederalStatus::Single);
        let exercised =
            if matches!(recognition, IsoRecognition::Vesting { .. }) { 20240101 } else { 20260101 };
        d.screening.as_mut().unwrap().iso_exercises = vec![IsoExercise {
            exercised,
            recognition,
            ..iso_exercise("Synthetic recognition event", 2000.0, 1000, 11000)
        }];
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 60000);
        assert_eq!(value(&r, "alternative_minimum_taxable_income"), 260000);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        // A later vesting event uses that event's FMV, not old exercise FMV.
        let lot = &mut d.screening.as_mut().unwrap().iso_exercises[0];
        match &mut lot.recognition {
            IsoRecognition::Vesting { market_price, .. } => *market_price = 21000,
            _ => lot.market_price = 21000,
        }
        assert_eq!(value(&supported(&d), "alternative_minimum_taxable_income"), 460000);
    }
    let mut event = iso_exercise("Synthetic invalid event", 1.0, 1000, 11000);
    event.exercised = 20250101;
    assert!(!event.valid_timing());
    event.recognition = IsoRecognition::Election;
    assert!(!event.valid_timing());
    event.recognition = IsoRecognition::Vesting { date: 20260101, market_price: 11000 };
    assert!(event.valid_timing());
    event.exercised = 20260701;
    assert!(!event.valid_timing());
    event.recognition = IsoRecognition::Vesting { date: 20270101, market_price: 11000 };
    assert!(!event.valid_timing());
}
