// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;
#[test]
fn itemized_medical_salt_mortgage_charity_floor() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    let v = d.deductions.as_mut().unwrap();
    itemized(v).medical = 1_000_000;
    itemized(v).personal_property_taxes = 1_000_000;
    fixture_mortgage_interest(v, 1_500_000);
    v.set_cash_charity(200_000);
    let r = supported(&d);
    assert_eq!(value(&r, "deduction"), 29000);
    assert_eq!(r.tax, Some(1_033_800));
}

#[test]
fn charitable_excess_cannot_remain_in_both_charity_and_regular_nol_carryforward() {
    let d = nol_cash_gift(&[(2017, 100000)]);
    let r = supported(&d);
    // Without the current NOL: 60,000 income, 35,700 permitted gifts,
    // 24,300 modified taxable income. With the NOL, all 40,000 gifts
    // are initially unused; 35,700 instead increases surviving NOL.
    assert_eq!(value(&r, "prior_nol_deduction"), 100000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 75700);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 0);
    assert_eq!(value(&r, "charity_return_year_nol_reduction"), 35700);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 4300);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn charitable_nol_adjustment_excludes_expired_losses_and_respects_available_balance() {
    for (year, loss, reduction) in [(2006, 100000, 0), (2017, 50000, 25700), (2017, 1000, 0)] {
        let d = nol_cash_gift(&[(year, loss)]);
        let r = supported(&d);
        assert_eq!(value(&r, "charity_return_year_nol_reduction"), reduction);
        assert_eq!(
            value(&r, "charity_return_year_carryforward"),
            40000 - value(&r, "charity_cash_sixty_current_deduction") - reduction
        );
    }
}

#[test]
fn charitable_nol_adjustment_with_post2017_and_multiple_vintages_is_counted_once() {
    let d = nol_cash_gift(&[(2016, 10000), (2025, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "prior_nol_deduction"), 21440);
    assert_eq!(value(&r, "charity_cash_sixty_current_deduction"), 22943);
    assert_eq!(value(&r, "charity_return_year_nol_reduction"), 6807);
    assert_eq!(value(&r, "charity_return_year_carryforward"), 10250);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn taxable_loan_repayment_cannot_double_count_excluded_investment_returns() {
    let mut source = scorp_repaid_reduced_open_account(ScorpLoanKind::OpenAccount);
    source.risk.excluded_returns = 100_000;
    assert_eq!(
        source.risk.calculate(&source, &source.basis_result().unwrap()).unwrap_err(),
        crate::scorp_risk::ScorpRiskError::DebtRepaymentAllocation,
    );
}

#[test]
fn repayment_comparison_uses_each_original_year_filing_table() {
    // Independent rows in the IRS 2023/2024 tax-table fixtures.
    for (year, standard, original, refigured) in
        [(2023, 13850, 8766, 7666), (2024, 14600, 8347, 7247)]
    {
        let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
        let FederalIncome::GovernmentPayment { data } = &mut d.income[1].income else { panic!() };
        data.repayments[0].received_year = year;
        let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment
        else {
            panic!()
        };
        years[0].year = year;
        years[0].standard = standard * 100;
        years[0].income_tax = original * 100;
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_tax"), original);
        assert_eq!(value(&r, "repayment_year_0_refigured_tax"), refigured);
        assert_eq!(value(&r, "repayment_credit"), 1100);
    }
}

#[test]
fn repayment_comparison_recomputes_newly_available_historical_eic() {
    let d = repayment_eic_draft();
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_tax"), 428);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 0);
    assert_eq!(value(&r, "repayment_year_0_original_eic"), 0);
    assert_eq!(value(&r, "repayment_year_0_refigured_eic"), 312);
    assert_eq!(value(&r, "repayment_credit"), 740);
    assert_eq!(r.tax, Some(502300));
    assert_eq!(return_line(&r, "31"), 74000);
    assert_eq!(supported(&restart(d).draft).refund, r.refund);
}

#[test]
fn repayment_social_recomputes_taxable_benefits_and_agi_in_each_year() {
    for (year, standard, original, after, credit) in
        [(2023, 15700, 1199, 433, 766), (2024, 16550, 1098, 348, 750), (2025, 17750, 378, 0, 378)]
    {
        let d = repayment_social_draft(year, standard, original);
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_taxable_social"), 2500);
        assert_eq!(value(&r, "repayment_year_0_refigured_taxable_social"), 0);
        assert_eq!(value(&r, "repayment_year_0_original_agi"), 27500);
        assert_eq!(value(&r, "repayment_year_0_refigured_agi"), 20000);
        assert_eq!(value(&r, "repayment_year_0_refigured_tax"), after);
        assert_eq!(value(&r, "repayment_credit"), credit);
        assert_eq!(supported(&restart(d).draft).refund, r.refund);
    }
}

#[test]
fn repayment_social_nontaxable_benefits_reduce_schedule_r_without_duplicate_entry() {
    let mut d = repayment_senior_draft(2023);
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        panic!()
    };
    years[0].wages = 1000000;
    years[0].benefits = 1200000;
    years[0].standard = 1570000;
    years[0].income_tax = 63300;
    years[0].social =
        RepaymentSocial::Benefits { amount: 100000, filed_taxable: 0, ordinary: true };
    // Known nontaxable Social Security already eliminates any possible credit.
    // An unused pension worksheet must not be required.
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_refigured_elderly_credit"), 0);
    assert_eq!(value(&r, "repayment_credit"), 502);
}

#[test]
fn repayment_early_medical_recomputes_tax_and_refundable_credit() {
    // 6,625 medical - 5,625 floor = 1,000 original exception.
    // Revised exception 1,375; subject distributions 9,000 -> 8,625.
    for (simple, original, revised, credit) in [(false, 900, 863, 1137), (true, 2250, 2156, 1194)] {
        let d = repayment_early_draft(simple, true, 6625, original);
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_early_tax"), original);
        assert_eq!(value(&r, "repayment_year_0_refigured_early_tax"), revised);
        assert_eq!(value(&r, "repayment_credit"), credit);
        assert_eq!(return_line(&r, "31"), credit * 100);
        assert_eq!(r.tax, Some(502300));
        assert_eq!(r.refund, Some((1477 + credit) * 100));
        assert_eq!(supported(&restart(d).draft).refund, r.refund);
    }
}

#[test]
fn repayment_early_medical_caps_and_floor_crossings() {
    for (expenses, original, revised, credit) in
        [(0, 1000, 1000, 1100), (5400, 1000, 985, 1115), (15500, 13, 0, 1113), (20000, 0, 0, 1100)]
    {
        let r = supported(&repayment_early_draft(false, true, expenses, original));
        assert_eq!(value(&r, "repayment_year_0_refigured_early_tax"), revised);
        assert_eq!(value(&r, "repayment_credit"), credit);
    }
}

#[test]
fn repayment_early_medical_applies_floor_once_and_25_percent_first() {
    let mut d = repayment_early_draft(true, true, 6625, 1650);
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        panic!()
    };
    years[0].retirement = vec![
        RepaymentRetirement::TraditionalIra {
            gross: 600000,
            normal: true,
            treatment: RepaymentDistribution::Early {
                owner: Owner::Taxpayer,
                simple: true,
                medical: true,
            },
        },
        RepaymentRetirement::Pension {
            gross: 600000,
            taxable: 400000,
            normal: true,
            treatment: RepaymentDistribution::Early {
                owner: Owner::Taxpayer,
                simple: false,
                medical: true,
            },
        },
    ];
    // 5,000 * 25% + 4,000 * 10% = 1,650. Revised: 4,625 * 25% + 400 = 1,556.
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_refigured_early_tax"), 1556);
    assert_eq!(value(&r, "repayment_credit"), 1194);
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        panic!()
    };
    years[0].retirement.reverse();
    assert_eq!(supported(&d).refund, r.refund);
}

#[test]
fn repayment_early_uses_original_year_tables() {
    for (year, standard, original) in
        [(2023, 13850, 8766), (2024, 14600, 8347), (2025, 15750, 7955)]
    {
        let mut d = repayment_early_draft(false, true, 6625, 900);
        for entry in &mut d.income {
            if let FederalIncome::GovernmentPayment { data } = &mut entry.income {
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
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_tax"), original);
        assert_eq!(value(&r, "repayment_year_0_refigured_tax"), original - 1100);
        assert_eq!(value(&r, "repayment_credit"), 1137);
    }
}

#[test]
fn repayment_high_income_can_cross_filing_table_threshold() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        panic!()
    };
    years[0].wages = 10075000;
    years[0].income_tax = 1691400;
    let r = supported(&d);
    // Original TI100,000 uses the rate worksheet; revised TI95,000 uses the
    // published95,000–95,050 table row, not a continuous-rate approximation.
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 15820);
    assert_eq!(value(&r, "repayment_credit"), 1094);
}

#[test]
fn repayment_capital_personal_losses_do_not_offset_taxable_gains() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Automatic);
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        panic!()
    };
    years[0].income_tax = 825500;
    let mut loss = historical_sale(GainTerm::Short, 1000, 11000);
    loss.use_kind = SaleUse::Personal;
    let mut gain = historical_sale(GainTerm::Long, 3000, 1000);
    gain.use_kind = SaleUse::Personal;
    years[0].capital = historical_capital(vec![loss, gain], 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_net_short_capital"), 0);
    assert_eq!(value(&r, "repayment_year_0_net_long_capital"), 2000);
    assert_eq!(value(&r, "repayment_year_0_refigured_tax"), 7155);
    assert_eq!(value(&r, "repayment_credit"), 1100);
}

#[test]
fn repayment_restoration_excludes_hsa_excess_excise_from_chapter_one_comparison() {
    let mut d = repayment_restoration_draft(RepaymentMethod::Credit);
    add(&mut d, income(capital_return(10000, 0, GainTerm::Short)));
    let control = supported(&d);

    let mut hsa = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    hsa.excess_accounting = Some(hsa_excess_accounting(
        100_000,
        Some(HsaPriorExcess { line_48: 100_000, line_49: 6_000 }),
        0,
        0,
        0,
    ));
    d.adjustments.as_mut().unwrap().hsa.push(hsa);
    let with_excise = supported(&d);
    assert_eq!(value(&with_excise, "hsa_return_year_excess_contribution_tax"), 60);
    assert_eq!(value(&with_excise, "hsa_additional_tax"), 0);
    assert_eq!(value(&with_excise, "hsa_testing_period_tax"), 0);
    for id in [
        "chapter_one_net_tax",
        "repayment_return_year_original_chapter_one_net_tax",
        "repayment_return_year_refigured_chapter_one_net_tax",
        "repayment_return_year_decrease",
        "repayment_credit",
    ] {
        assert_eq!(value(&with_excise, id), value(&control, id), "{id}");
    }
    assert_eq!(value(&with_excise, "total_tax"), value(&control, "total_tax") + 60);
    assert_eq!(with_excise.tax, control.tax.map(|tax| tax + 6_000));
    assert_eq!(return_line(&with_excise, "23"), return_line(&control, "23") + 6_000);
    assert_eq!(estimate(&restart(d.clone()).draft), with_excise);
}

#[test]
fn repayment_restoration_includes_changed_current_refundable_credit() {
    let mut d = repayment_restoration_draft(RepaymentMethod::Credit);
    d.income[0] = wage(1, Owner::Taxpayer, 1000000, 0);
    add(&mut d, income(capital_return(15000, 0, GainTerm::Short)));
    let r = supported(&d);
    // The draft EIC table gives AGI $18,000 a $116 EIC against $191 tax;
    // refigured AGI $15,000 gives a $345 EIC against zero tax. Difference $420.
    assert_eq!(value(&r, "earned_income_credit"), 116);
    assert_eq!(value(&r, "repayment_return_year_original_chapter_one_net_tax"), 75);
    assert_eq!(value(&r, "repayment_return_year_refigured_chapter_one_net_tax"), -345);
    assert_eq!(value(&r, "repayment_return_year_decrease"), 420);
    assert_eq!(value(&r, "repayment_credit"), 546);
    assert_eq!(r.refundable_credits, Some(66200));
    assert_eq!(r.refund, Some(47100));
}

#[test]
fn repayment_amt_sources_require_minimum_credit_coordination_when_amt_payable() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let RepaymentClaim::Claim { years, .. } = &mut d.deductions.as_mut().unwrap().repayment else {
        panic!()
    };
    let mut sale = historical_sale(GainTerm::Short, 100000, 100000);
    sale.amt =
        crate::SaleAmt::Different { record: CapitalSaleAmt { term: None, basis: 0, wash_sale: 0 } };
    years[0].capital = historical_capital(vec![sale], 0, 0);
    years[0].income_tax = 2259400;
    unsupported(&d, "Repayment minimum-tax credit");
}

#[test]
fn charitable_mileage_and_parking_join_cash_class_once_when_itemizing() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.set_cash_charity(100_000);
    ded.volunteer_expenses.push(volunteer_travel(VolunteerCarCost::Mileage { miles: 100 }, 600));
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let result = supported(&d);
    assert_eq!(value(&result, "charity_volunteer_1_expense"), 20);
    assert_eq!(value(&result, "charity_cash_sixty_current_deduction"), 720);
    assert_eq!(estimate(&restart(d.clone()).draft), result);

    d.deductions.as_mut().unwrap().volunteer_expenses[0].documented = false;
    assert!(noted(&supported(&d), "WARN_VOLUNTEER_EXPENSE_LEFT_OUT"));
}
