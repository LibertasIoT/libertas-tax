// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::*;

#[test]
fn repayment_comparison_requires_reconciled_filing_unit_and_older_filer_facts() {
    for mode in 0..3 {
        let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
        if mode == 2 {
            d.people.as_mut().unwrap().taxpayer.birth_date = 19600101;
        } else {
            let years = repayment_years_mut(&mut d);
            if mode == 0 {
                years[0].status = FederalStatus::Joint;
            } else {
                years[0].filers = RepaymentFilers::Both;
                years[0].status = FederalStatus::Joint;
            }
        }
        unsupported(&d, "Repayment comparison");
    }
}

#[test]
fn repayment_comparison_includes_only_increase_over_original_eic() {
    let mut d = repayment_eic_draft();
    let years = repayment_years_mut(&mut d);
    years[0].benefits = 600000;
    years[0].income_tax = 2600;
    let RepaymentEic::Facts { filed_credit, .. } = &mut years[0].eic else { panic!() };
    *filed_credit = 23600;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_eic"), 236);
    assert_eq!(value(&r, "repayment_year_0_refigured_eic"), 618);
    assert_eq!(value(&r, "repayment_credit"), 408);
    assert_eq!(r.refundable_credits, Some(40800));
}

#[test]
fn repayment_comparison_historical_eic_assumes_unchanged_without_original_year_facts() {
    for mode in 0..3 {
        let mut d = repayment_eic_draft();
        let years = repayment_years_mut(&mut d);
        // Facts or timely SSNs not supplied: no change in the credit is
        // counted, with a warning. A filed credit the facts do not reproduce
        // still needs reconciling.
        match mode {
            0 => years[0].eic = RepaymentEic::NotSupplied,
            1 => years[0].ssns = RepaymentSsns::NotSupplied,
            _ => {
                let RepaymentEic::Facts { filed_credit, .. } = &mut years[0].eic else { panic!() };
                *filed_credit = 100;
                unsupported(&d, "Repayment comparison");
                continue;
            }
        }
        let r = supported(&d);
        let key = if mode == 0 {
            "WARN_REPAYMENT_EIC_ASSUMED_UNCHANGED"
        } else {
            "WARN_REPAYMENT_SSNS_ASSUMED_NONE"
        };
        assert!(noted(&r, key), "{r:?}");
        assert_eq!(value(&r, "repayment_year_0_refigured_eic"), 0);
    }
    for mode in 0..4 {
        let mut d = repayment_eic_draft();
        let years = repayment_years_mut(&mut d);
        let RepaymentEic::Facts { home, independent, allowed, .. } = &mut years[0].eic else {
            panic!()
        };
        match mode {
            0 => *home = false,
            1 => years[0].ssns = RepaymentSsns::Neither,
            2 => *independent = false,
            _ => *allowed = false,
        }
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_credit"), 428);
    }
}

#[test]
fn repayment_unknown_eic_facts_preserve_positive_filed_credit_without_refusal() {
    let mut draft = repayment_eic_draft();
    let years = repayment_years_mut(&mut draft);
    years[0].benefits = 600000;
    years[0].income_tax = 2600;
    let RepaymentEic::Facts { filed_credit, .. } = &mut years[0].eic else { panic!() };
    *filed_credit = 23600;
    years[0].ssns = RepaymentSsns::NotSupplied;
    let result = supported(&draft);
    assert_eq!(value(&result, "repayment_year_0_original_eic"), 236);
    assert_eq!(value(&result, "repayment_year_0_refigured_eic"), 236);
    assert_eq!(value(&result, "repayment_credit"), 26);
    let notes: Vec<_> = result
        .review_notes
        .iter()
        .filter(|note| note.display.key == "WARN_REPAYMENT_SSNS_ASSUMED_NONE")
        .collect();
    assert_eq!(notes.len(), 1);
    let context = notes[0].context.as_ref().unwrap();
    assert_eq!(crate::interview::issue_page(&draft, context, None), Some(TaxSection::Deductions));
    assert_eq!(context.label().key, "ISSUE_REPAYMENT_SOURCE_YEAR");
    // A fully known denial is still inconsistent with a positive filed credit.
    let mut draft = repayment_eic_draft();
    let years = repayment_years_mut(&mut draft);
    let RepaymentEic::Facts { allowed, filed_credit, .. } = &mut years[0].eic else { panic!() };
    *allowed = false;
    *filed_credit = 23600;
    let result = estimate(&draft);
    let issue = result.issues.iter().find(|issue| issue.display.key == "ISSUE_ORIGINAL_YEAR_EARNED_INCOME_CREDIT_FACTS_AND_PUBLISHED_TABLE_DO_NOT_3B979D7E").unwrap();
    assert_eq!(
        crate::interview::issue_page(&draft, &issue.context, Some(&issue.display.key)),
        Some(TaxSection::Deductions)
    );
}

#[test]
fn repayment_comparison_historical_eic_uses_historical_age() {
    // January 1 birthday reaches age 25 on the previous day.
    for (birth, eic) in [(20010101, 312), (20010102, 0)] {
        let mut d = repayment_eic_draft();
        d.people.as_mut().unwrap().taxpayer.birth_date = birth;
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_refigured_eic"), eic);
        assert_eq!(value(&r, "repayment_credit"), 428 + eic);
    }
}

#[test]
fn repayment_comparison_joint_historical_eic_uses_joint_column() {
    let source = repayment_eic_draft();
    let mut d = complete(FederalStatus::Joint);
    d.deductions.as_mut().unwrap().repayment = source.deductions.unwrap().repayment;
    add(&mut d, prior_unemployment_repayment(500000, Owner::Spouse));
    let years = repayment_years_mut(&mut d);
    years[0].filers = RepaymentFilers::Both;
    years[0].status = FederalStatus::Joint;
    years[0].standard = 3150000;
    years[0].benefits = 2000000;
    years[0].income_tax = 0;
    years[0].ssns = RepaymentSsns::Both;
    // Only the spouse meets the original-year age test.
    d.people.as_mut().unwrap().taxpayer.birth_date = 20030101;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_refigured_eic"), 91);
    assert_eq!(value(&r, "repayment_credit"), 91);
}

#[test]
fn repayment_comparison_historical_eic_coordinates_each_year_tax_and_credit() {
    for (year, standard, original_tax, refigured_tax, eic, credit) in [
        (2023, 13850, 618, 116, 200, 702),
        (2024, 14600, 543, 41, 273, 775),
        (2025, 15750, 428, 0, 312, 740),
    ] {
        let mut d = repayment_eic_draft();
        let FederalIncome::GovernmentPayment { data } = &mut d.income[1].income else { panic!() };
        data.repayments[0].received_year = year;
        let years = repayment_years_mut(&mut d);
        years[0].year = year;
        years[0].standard = standard * 100;
        years[0].income_tax = original_tax * 100;
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_refigured_tax"), refigured_tax);
        assert_eq!(value(&r, "repayment_year_0_refigured_eic"), eic);
        assert_eq!(value(&r, "repayment_credit"), credit);
    }
}

#[test]
fn repayment_comparison_historical_eic_checks_investment_limit_and_stored_credit() {
    for (interest, eic) in [(11950, 239), (11951, 0)] {
        let mut d = repayment_eic_draft();
        let years = repayment_years_mut(&mut d);
        years[0].wages = 400000;
        years[0].interest = interest * 100;
        years[0].benefits = 500000;
        years[0].income_tax = 52300;
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_refigured_eic"), eic);
        assert_eq!(value(&r, "repayment_credit"), 502 + eic);
    }
}

#[test]
fn repayment_senior_joint_deduction_follows_each_original_ssn() {
    for (ssns, deduction, original, credit) in
        [(RepaymentSsns::Taxpayer, 6000, 3642, 600), (RepaymentSsns::Both, 12000, 2922, 589)]
    {
        let source = repayment_comparison_draft(RepaymentMethod::Automatic);
        let mut d = complete(FederalStatus::Joint);
        d.people.as_mut().unwrap().taxpayer.birth_date = 19580615;
        d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19560101;
        d.deductions.as_mut().unwrap().repayment = source.deductions.unwrap().repayment;
        add(&mut d, prior_unemployment_repayment(500000, Owner::Spouse));
        let years = repayment_years_mut(&mut d);
        years[0].status = FederalStatus::Joint;
        years[0].filers = RepaymentFilers::Both;
        years[0].standard = 3470000;
        years[0].ssns = ssns;
        years[0].income_tax = original * 100;
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_senior_deduction"), deduction);
        assert_eq!(value(&r, "repayment_credit"), credit);
    }
}

#[test]
fn repayment_senior_and_eic_share_ssn_facts_without_sharing_age_tests() {
    for (ssns, eic) in [(RepaymentSsns::Taxpayer, 0), (RepaymentSsns::Both, 91)] {
        let source = repayment_eic_draft();
        let mut d = complete(FederalStatus::Joint);
        d.people.as_mut().unwrap().taxpayer.birth_date = 19580615;
        d.deductions.as_mut().unwrap().repayment = source.deductions.unwrap().repayment;
        add(&mut d, prior_unemployment_repayment(500000, Owner::Spouse));
        let years = repayment_years_mut(&mut d);
        years[0].status = FederalStatus::Joint;
        years[0].filers = RepaymentFilers::Both;
        years[0].standard = 3310000;
        years[0].benefits = 2000000;
        years[0].income_tax = 0;
        years[0].ssns = ssns;
        if eic == 0 {
            years[0].eic = RepaymentEic::NotSupplied;
        }
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_senior_deduction"), 6000);
        assert_eq!(value(&r, "repayment_year_0_refigured_eic"), eic);
        assert_eq!(value(&r, "repayment_credit_potential"), eic);
    }
}

#[test]
fn repayment_social_agi_recomputation_reaches_historical_eic() {
    let mut d = repayment_eic_draft();
    let years = repayment_years_mut(&mut d);
    years[0].income_tax = 67800;
    years[0].social =
        RepaymentSocial::Benefits { amount: 2000000, filed_taxable: 250000, ordinary: true };
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_agi"), 22500);
    assert_eq!(value(&r, "repayment_year_0_refigured_agi"), 15000);
    assert_eq!(value(&r, "repayment_year_0_refigured_eic"), 312);
    assert_eq!(value(&r, "repayment_credit"), 990);
}

#[test]
fn repayment_social_joint_return_combines_benefits_and_uses_joint_thresholds() {
    let source = repayment_comparison_draft(RepaymentMethod::Automatic);
    let mut d = complete(FederalStatus::Joint);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19580615;
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().birth_date = 19560615;
    d.deductions.as_mut().unwrap().repayment = source.deductions.unwrap().repayment;
    add(&mut d, prior_unemployment_repayment(500000, Owner::Spouse));
    let years = repayment_years_mut(&mut d);
    years[0].status = FederalStatus::Joint;
    years[0].filers = RepaymentFilers::Both;
    years[0].wages = 4000000;
    years[0].benefits = 1000000;
    years[0].standard = 3470000;
    years[0].income_tax = 203300;
    years[0].ssns = RepaymentSsns::Both;
    years[0].social =
        RepaymentSocial::Benefits { amount: 2000000, filed_taxable: 1700000, ordinary: true };
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_refigured_taxable_social"), 15350);
    assert_eq!(value(&r, "repayment_year_0_original_agi"), 67000);
    assert_eq!(value(&r, "repayment_year_0_refigured_agi"), 60350);
    assert_eq!(value(&r, "repayment_credit"), 665);
}

#[test]
fn repayment_exempt_interest_also_counts_for_eic_without_social_benefits() {
    for (exempt, eic) in [(11950, 308), (11951, 0)] {
        let mut d = repayment_eic_draft();
        let years = repayment_years_mut(&mut d);
        years[0].wages = 400000;
        years[0].benefits = 1600000;
        years[0].exempt_interest = exempt * 100;
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_original_agi"), 20000);
        assert_eq!(value(&r, "repayment_year_0_refigured_eic"), eic);
        assert_eq!(value(&r, "repayment_credit"), 428 + eic);
    }
}

#[test]
fn repayment_early_and_eic_changes_both_enter_credit() {
    let mut d = repayment_eic_draft();
    let years = repayment_years_mut(&mut d);
    years[0].wages = 500000;
    years[0].retirement = vec![RepaymentRetirement::TraditionalIra {
        gross: 500000,
        normal: true,
        treatment: RepaymentDistribution::Early {
            owner: Owner::Taxpayer,
            simple: false,
            medical: true,
        },
    }];
    years[0].early =
        RepaymentEarly::Facts { medical_expenses: 250000, filed_tax: 40000, confirmed: true };
    // Original AGI20,000, medical exemption1,000; revised AGI15,000, exemption1,375.
    // EIC at AGI15,000 =312 (below the earned-income table amount).
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_refigured_eic"), 312);
    assert_eq!(value(&r, "repayment_year_0_refigured_early_tax"), 363);
    assert_eq!(value(&r, "repayment_credit"), 777);
}

#[test]
fn repayment_dividends_and_distributions_enter_eic_investment_limit_once() {
    for (gains, eic) in [(5950, 163), (5951, 0)] {
        let mut d = repayment_eic_draft();
        let years = repayment_years_mut(&mut d);
        years[0].wages = 400000;
        years[0].benefits = 600000;
        years[0].income_tax = 0;
        years[0].dividends = historical_dividends(6000, 6000, gains, 0, 0);
        let r = supported(&d);
        assert_eq!(value(&r, "repayment_year_0_refigured_eic"), eic);
        assert_eq!(value(&r, "repayment_credit"), eic);
    }
}

#[test]
fn repayment_capital_net_gain_supplies_eic_investment_income() {
    let mut d = repayment_eic_draft();
    let years = repayment_years_mut(&mut d);
    years[0].wages = 400000;
    years[0].benefits = 600000;
    years[0].income_tax = 0;
    years[0].dividends = historical_dividends(6000, 6000, 5950, 0, 0);
    years[0].capital = historical_capital(vec![], 1000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_capital_income"), 4950);
    assert_eq!(value(&r, "repayment_year_0_refigured_agi"), 15950);
    assert_eq!(value(&r, "repayment_year_0_refigured_eic"), 239);
    assert_eq!(value(&r, "repayment_credit"), 239);
}

#[test]
fn repayment_carry_propagates_into_dependent_year_without_repayment() {
    let d = repayment_carry_chain();
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_short_carry"), 7000);
    assert_eq!(value(&r, "repayment_year_0_refigured_short_carry"), 9600);
    assert_eq!(value(&r, "repayment_year_1_capital_income"), 13000);
    assert_eq!(value(&r, "repayment_year_1_refigured_capital_income"), 10400);
    assert_eq!(value(&r, "repayment_year_1_refigured_amt_capital_income"), 13000);
    assert_eq!(value(&r, "repayment_year_1_refigured_tax"), 6943);
    assert_eq!(value(&r, "repayment_credit"), 813);
    assert_eq!(r.refund, Some(229000));
    assert_eq!(supported(&restart(d).draft).refund, r.refund);
}

#[test]
fn repayment_carry_chronology_is_independent_of_source_order() {
    let mut d = repayment_carry_chain();
    let original = supported(&d);
    let years = repayment_years_mut(&mut d);
    years.reverse();
    let r = supported(&d);
    assert_eq!(r.refund, original.refund);
    assert_eq!(value(&r, "repayment_year_0_refigured_capital_income"), 10400);
    assert_eq!(value(&r, "repayment_year_1_refigured_short_carry"), 9600);
}

#[test]
fn repayment_carry_rejects_duplicate_dependent_returns() {
    let mut d = repayment_carry_chain();
    let years = repayment_years_mut(&mut d);
    years.push(years[1].clone());
    unsupported(&d, "Repayment comparison");
}

#[test]
fn repayment_carry_recomputes_eic_investment_income_eligibility() {
    let mut d = repayment_carry_chain();
    let years = repayment_years_mut(&mut d);
    years[1].wages = 400000;
    years[1].income_tax = 12600;
    years[1].ssns = RepaymentSsns::Taxpayer;
    years[1].eic =
        RepaymentEic::Facts { home: true, independent: true, allowed: true, filed_credit: 0 };
    let r = supported(&d);
    // Actual net gains13,000 exceed11,950; revised10,400 do not.
    // Published2025 childless table at4,000 earned income limits the credit to308.
    assert_eq!(value(&r, "repayment_year_1_original_eic"), 0);
    assert_eq!(value(&r, "repayment_year_1_refigured_eic"), 308);
    assert_eq!(value(&r, "repayment_credit"), 675);
}

#[test]
fn repayment_carry_keeps_amt_history_independent_when_amt_controls() {
    let mut d = repayment_carry_chain();
    let years = repayment_years_mut(&mut d);
    years[1].exempt_interest = 10000000;
    years[1].private_activity_interest = 10000000;
    years[1].income_tax = 2207400;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_1_original_amti"), 173000);
    assert_eq!(value(&r, "repayment_year_1_refigured_amti"), 173000);
    assert_eq!(value(&r, "repayment_year_1_decrease"), 0);
    assert_eq!(value(&r, "repayment_credit"), 241);
}

#[test]
fn repayment_amt_sources_use_independent_incoming_loss_balances() {
    let mut d = repayment_comparison_draft(RepaymentMethod::Credit);
    let years = repayment_years_mut(&mut d);
    years[0].capital =
        historical_capital(vec![historical_sale(GainTerm::Short, 20000, 0)], 5000, 0);
    let RepaymentCapital::Records { amt, .. } = &mut years[0].capital else { panic!() };
    *amt = Some(RepaymentAmtLosses { short: 1500000, long: 0 });
    years[0].income_tax = 1125500;
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_capital_income"), 15000);
    assert_eq!(value(&r, "repayment_year_0_original_amt_capital_income"), 5000);
    assert_eq!(value(&r, "repayment_credit"), 1100);
}

#[test]
fn repayment_amt_sources_reconcile_independent_balances_in_dependent_years() {
    let mut d = repayment_carry_chain();
    let years = repayment_years_mut(&mut d);
    let RepaymentCapital::Records { sales, .. } = &mut years[0].capital else { panic!() };
    sales[0].sale.amt = crate::SaleAmt::Different {
        record: CapitalSaleAmt { term: None, basis: 2000000, wash_sale: 0 },
    };
    let RepaymentCapital::Records { amt, .. } = &mut years[1].capital else { panic!() };
    *amt = Some(RepaymentAmtLosses { short: 1700000, long: 0 });
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_1_original_amt_capital_income"), 3000);
    assert_eq!(value(&r, "repayment_year_1_refigured_amt_capital_income"), 3000);
    assert_eq!(value(&r, "repayment_credit"), 813);
    let years = repayment_years_mut(&mut d);
    let RepaymentCapital::Records { amt, .. } = &mut years[1].capital else { panic!() };
    *amt = None;
    unsupported(&d, "Repayment capital");
}

#[test]
fn repayment_independent_exclusion_losses_follow_original_and_refigured_utilization() {
    let d = repayment_exclusion_loss_chain();
    let r = supported(&d);
    assert_eq!(value(&r, "repayment_year_0_original_exclusion_short_carry"), 11000);
    assert_eq!(value(&r, "repayment_year_0_refigured_exclusion_short_carry"), 14000);
    assert_eq!(value(&r, "repayment_year_1_original_exclusion_income"), 9000);
    assert_eq!(value(&r, "repayment_year_1_refigured_exclusion_income"), 6000);
    assert_eq!(value(&r, "repayment_year_1_original_tax"), 21554);
    assert_eq!(value(&r, "repayment_year_1_refigured_tax"), 20774);
    assert_eq!(value(&r, "minimum_tax_exclusion_tax"), 14179);
    assert_eq!(value(&r, "minimum_tax_credit_generated"), 520);
    assert_eq!(value(&r, "minimum_tax_credit"), 520);
    assert_eq!(value(&r, "repayment_credit"), 780);
    assert_eq!(supported(&restart(d).draft).refund, r.refund);
}

#[test]
fn whaling_and_hosted_student_share_one_thirty_percent_class_and_floor() {
    let mut draft = complete(FederalStatus::Single);
    let ded = draft.deductions.as_mut().unwrap();
    ded.whaling_captains.push(whaling_claim(vec![whaling_expense(
        WhalingExpenseKind::CrewAndProvisions,
        1_000_000,
    )]));
    ded.hosted_students.push(HostedStudent {
        label: "Synthetic hosted student".into(),
        sponsor: "Synthetic educational charity".into(),
        agreement: "Synthetic written agreement".into(),
        school: "Synthetic high school".into(),
        school_location: "Synthetic City, AK".into(),
        grade: 11,
        joined: 20260601,
        school_start: 20260601,
        school_end: 20261231,
        months: vec![HostedStudentMonth { month: 6, qualifying_days: 15, cost: 5_000 }],
        documented: true,
    });
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let result = supported(&draft);
    assert_eq!(value(&result, "charity_other_30_percent_deduction"), 9_750);
    assert_eq!(value(&result, "charity_other_30_percent_return_year_carryforward"), 0);
    assert_eq!(value(&result, "charity_return_year_carryforward"), 0);
    assert_eq!(estimate(&restart(draft).draft), result);
}
