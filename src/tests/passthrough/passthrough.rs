use super::support::assert::unit50_fuel::assert_fuel_payment;
use super::*;

#[test]
fn bba_audit_interest_increase_enters_line_16_and_separate_net_payment() {
    for scenario in 0..3 {
        let mut draft = complete(FederalStatus::Single);
        match scenario {
            1 => {
                draft.screening.as_mut().unwrap().iso_exercises =
                    vec![iso_exercise("Synthetic ISO", 2000.0, 1000, 11000)];
            }
            2 => {
                draft.credits.as_mut().unwrap().tip_fica_carryforward =
                    Some(tip_fica_carryforward(5_000_000))
            }
            _ => {}
        }
        let baseline = supported(&draft);
        if scenario == 1 {
            assert!(value(&baseline, "alternative_minimum_tax") > 220);
        }
        draft.credits.as_mut().unwrap().bba_audit_interest =
            Some(crate::tests::support::builders::added_bba(bba_interest_increase()));
        let calculated = supported(&draft);
        match scenario {
            0 => {
                assert_eq!(value(&calculated, "form_8978_prior_income_tax"), 5920);
                assert_eq!(value(&calculated, "form_8978_corrected_income_tax"), 6140);
                assert_eq!(value(&calculated, "form_8978_line_14"), 220);
                assert_eq!(value(&calculated, "form_8978_line_17_interest"), 35);
                assert_eq!(return_line(&calculated, "16"), return_line(&baseline, "16") + 22_000);
                assert_eq!(return_line(&calculated, "20"), return_line(&baseline, "20"));
                assert_eq!(calculated.tax, baseline.tax.map(|tax| tax + 22_000));
                let note = &calculated
                    .review_notes
                    .iter()
                    .find(|n| n.starts_with("Form 8978 adds"))
                    .unwrap()
                    .display;
                assert_eq!(note.key, "NOTE_FORM_8978_ADDITIONAL_TAX_AND_INTEREST");
                assert_eq!(note.args[0], MessageArg::usd_dollars(220));
                assert_eq!(note.args[1], MessageArg::usd_cents(3_500));
                assert_eq!(
                    value(&calculated, "form_8978_net_payment_after_interest") * 100,
                    calculated.balance.unwrap() - calculated.refund.unwrap() + 3_500,
                );
            }
            1 => {
                assert_eq!(
                    value(&calculated, "alternative_minimum_tax"),
                    value(&baseline, "alternative_minimum_tax") - 220,
                );
                assert_eq!(calculated.tax, baseline.tax);
                assert_eq!(return_line(&calculated, "16"), return_line(&baseline, "16") + 22_000);
            }
            _ => {
                assert_eq!(
                    value(&calculated, "general_business_credit_limit"),
                    value(&baseline, "general_business_credit_limit") + 220,
                );
                assert_eq!(
                    value(&calculated, "general_business_credit"),
                    value(&baseline, "general_business_credit") + 220,
                );
                assert_eq!(calculated.tax, baseline.tax);
            }
        }
        assert_eq!(supported(&restart(draft.clone()).draft), calculated);
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}
#[test]
fn bba_audit_interest_increase_requires_documented_interest_and_prior_reconciliation() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = bba_interest_increase();
    source.interest_calculation_prepared = false;
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(source.clone()));
    unsupported(&draft, "Form 8978");
    assert!(estimate(&draft).issues.iter().any(|issue| {
        issue.section == "Form 8978 BBA audit"
            && Some({
                let text = &issue.display;
                text.key.as_str()
            }) == Some("ISSUE_BBA_AUDIT_INTEREST_DOCUMENTATION")
    }));
    source.interest_calculation_prepared = true;
    source.prior_income_tax += 100;
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(source.clone()));
    unsupported(&draft, "Form 8978");
    source.prior_income_tax -= 100;
    source.interest_adjustment = 5_000_000;
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(source));
    unsupported(&draft, "Form 8978");
}

// Smith Review (round 3, G-54): an ordinary 2025 joint return as filed. A $20
// increase that moves the Tax Table row adds its tax; one that stays in the row
// changes nothing and is accepted; figures that do not reconcile name what this
// route does not support and link to More credits.
#[test]
fn bba_audit_ordinary_joint_2025_return_is_refigured_not_refused() {
    let mut draft = complete(FederalStatus::Joint);
    let ordinary = supported(&draft);
    let filed = FederalBbaAuditInterest {
        source_id: "SYNTH8986JOHN2026".into(),
        furnished_on: 20261015,
        interest_adjustment: 20_00,
        original_partnership_interest: 50_00,
        prior_wages: 12_003_000,
        prior_taxable_income: 8_858_000,
        prior_income_tax: 1_015_200,
        prior_status: FederalStatus::Joint,
        line_17_interest: 1_00,
        interest_calculation_prepared: true,
        ..bba_interest_reduction()
    };
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(filed.clone()));
    let calculated = supported(&draft);
    assert_eq!(value(&calculated, "form_8978_prior_income_tax"), 10_152);
    assert_eq!(value(&calculated, "form_8978_corrected_income_tax"), 10_158);
    assert_eq!(value(&calculated, "form_8978_line_14"), 6);
    assert_eq!(calculated.tax, ordinary.tax.map(|tax| tax + 6_00));

    // Taxable income $88,550: $88,570 stays in the $88,550-$88,600 row.
    let same_row = FederalBbaAuditInterest {
        prior_wages: 12_000_000,
        prior_taxable_income: 8_855_000,
        interest_calculation_prepared: false,
        ..filed.clone()
    };
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(same_row));
    let calculated = supported(&draft);
    assert_eq!(value(&calculated, "form_8978_line_14"), 0);
    assert_eq!(calculated.tax, ordinary.tax);
    assert_eq!(calculated.refund, ordinary.refund, "no line 17 interest without added tax");

    let other_income = FederalBbaAuditInterest { prior_taxable_income: 8_800_000, ..filed };
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(other_income));
    unsupported(&draft, "Form 8978");
    let result = estimate(&draft);
    let issue = result
        .issues
        .iter()
        .find(|issue| issue.display.key == "ISSUE_BBA_AUDIT_PRIOR_INCOME_RECONCILIATION")
        .expect("names the unsupported prior-return income or deductions");
    assert_eq!(
        crate::interview::issue_pages(&draft, &issue.context, None),
        vec![TaxSection::MoreCredits]
    );
}

#[test]
fn bba_audit_interest_reduction_flows_through_form_8978_and_schedule_3() {
    let mut draft = complete(FederalStatus::Single);
    let ordinary = supported(&draft);
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_interest_reduction()));
    let calculated = supported(&draft);
    assert_eq!(value(&calculated, "form_8978_prior_income_tax"), 5920);
    assert_eq!(value(&calculated, "form_8978_corrected_income_tax"), 5700);
    assert_eq!(value(&calculated, "form_8978_line_14"), -220);
    assert_eq!(value(&calculated, "schedule_3_line_6l"), 220);
    assert_eq!(value(&calculated, "alternative_minimum_tax"), 0);
    assert_eq!(calculated.tax, ordinary.tax.map(|tax| tax - 22_000));
    assert_eq!(calculated.refund, ordinary.refund.map(|refund| refund + 22_000));
    assert_eq!(supported(&restart(draft.clone()).draft).tax, calculated.tax);
    assert_eq!(restart(draft.clone()).draft, draft);
    assert!(
        calculated.return_lines.iter().any(|line| {
            status_text(&line.label).starts_with("1040 20") && line.amount == 22_000
        })
    );
}
#[test]
fn bba_audit_interest_requires_source_and_tax_reconciliation() {
    let mut draft = complete(FederalStatus::Single);
    let mut source = bba_interest_reduction();
    source.single_audit_adjustment = false;
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(source.clone()));
    unsupported(&draft, "Form 8978");
    source.single_audit_adjustment = true;
    source.prior_income_tax = 591_900;
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(source.clone()));
    unsupported(&draft, "Form 8978");
    source.prior_income_tax = 592_000;
    source.prior_wages = 6_374_900;
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(source.clone()));
    unsupported(&draft, "Form 8978");
    source.prior_wages = 6_375_000;
    source.interest_adjustment = -300_000;
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(source.clone()));
    unsupported(&draft, "Form 8978");
    source.interest_adjustment = -100_000;
    source.furnished_on = 20251231;
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(source.clone()));
    unsupported(&draft, "Form 8978");
    source.furnished_on = 20260615;
    source.no_other_effects = false;
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(source));
    unsupported(&draft, "Form 8978");
}
#[test]
fn bba_audit_credit_reduces_form_6251_comparison_before_amt() {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![wage(1, Owner::Taxpayer, 13_000_000, 0)];
    let baseline = supported(&draft);
    let mut source = bba_interest_reduction();
    source.interest_adjustment = -9_000_000;
    source.original_partnership_interest = 9_000_000;
    source.prior_wages = 2_500_000;
    source.prior_taxable_income = 9_925_000;
    source.prior_income_tax = 1_675_500;
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(source));
    let calculated = supported(&draft);
    assert_eq!(value(&calculated, "schedule_3_line_6l"), 15_827);
    assert!(value(&calculated, "alternative_minimum_tax") > 0);
    assert!(calculated.tax.unwrap() > baseline.tax.unwrap() - 1_582_700);
    assert_eq!(supported(&restart(draft).draft).tax, calculated.tax);
}
#[test]
fn bba_audit_negative_adjustment_caps_schedule_3_and_exposes_unused_decrease() {
    let mut low_tax = complete(FederalStatus::Single);
    low_tax.income = vec![wage(1, Owner::Taxpayer, 1_700_000, 0)];
    low_tax.deductions.as_mut().unwrap().itemize_override = None;
    let ordinary = supported(&low_tax);
    let line_18 = return_line(&ordinary, "18") / 100;
    assert!(line_18 > 0 && line_18 < 220);
    low_tax.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_interest_reduction()));
    let calculated = supported(&low_tax);
    assert_eq!(value(&calculated, "form_8978_line_14"), -220);
    assert_eq!(value(&calculated, "schedule_3_line_6l"), line_18);
    assert_eq!(value(&calculated, "schedule_2_line_13z_form_8978"), 0);
    assert_eq!(value(&calculated, "form_8978_unusable_decrease"), 220 - line_18);
    assert_eq!(return_line(&calculated, "20"), line_18 * 100);
    assert_eq!(return_line(&calculated, "23"), return_line(&ordinary, "23"));
    assert_eq!(calculated.tax, Some(0));
    assert_eq!(calculated.refund, ordinary.refund.map(|refund| refund + line_18 * 100));
    assert_eq!(supported(&restart(low_tax.clone()).draft), calculated);
    assert_eq!(restart(low_tax.clone()).draft, low_tax);

    let mut zero_tax = complete(FederalStatus::Single);
    zero_tax.income = vec![wage(1, Owner::Taxpayer, 1_000_000, 0)];
    zero_tax.deductions.as_mut().unwrap().itemize_override = None;
    zero_tax.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_interest_reduction()));
    let zero_result = supported(&zero_tax);
    assert_eq!(return_line(&zero_result, "18"), 0);
    assert_eq!(value(&zero_result, "schedule_3_line_6l"), 0);
    assert_eq!(value(&zero_result, "form_8978_unusable_decrease"), 220);
    assert_eq!(zero_result.tax, Some(0));

    // Schedule 2 self-employment tax is outside Chapter 1 and cannot absorb
    // the excess even when it makes the return's total tax positive.
    let mut se_case = zero_tax.clone();
    se_case.credits.as_mut().unwrap().bba_audit_interest = None;
    add(&mut se_case, income(cash_business(1000)));
    let se_ordinary = supported(&se_case);
    assert_eq!(return_line(&se_ordinary, "18"), 0);
    assert!(value(&se_ordinary, "self_employment_tax") > 0);
    se_case.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_interest_reduction()));
    let se_with_bba = supported(&se_case);
    assert_eq!(value(&se_with_bba, "schedule_2_line_13z_form_8978"), 0);
    assert_eq!(value(&se_with_bba, "form_8978_unusable_decrease"), 220);
    assert_eq!(return_line(&se_with_bba, "23"), return_line(&se_ordinary, "23"));
    assert_eq!(se_with_bba.tax, se_ordinary.tax);
}

#[test]
fn bba_audit_negative_adjustment_partly_spills_into_chapter_one_early_tax() {
    let mut draft = complete(FederalStatus::Single);
    draft.income = vec![wage(1, Owner::Taxpayer, 1_700_000, 0)];
    draft.deductions.as_mut().unwrap().itemize_override = None;
    let mut early =
        retirement_entry(Owner::Taxpayer, RetirementKind::Ira, 1000, RetirementTreatment::Early);
    let FederalIncome::Retirement { data } = &mut early.income else { unreachable!() };
    {
        let mut data = edit_retirement(data);
        data.withholding = 0;
    }
    add(&mut draft, early);
    let ordinary = supported(&draft);
    let line_18 = return_line(&ordinary, "18") / 100;
    let early_tax = value(&ordinary, "early_distribution_tax");
    assert!(line_18 > 0 && line_18 < 220);
    assert!(early_tax > 220 - line_18);
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_interest_reduction()));
    let calculated = supported(&draft);
    let spill = 220 - line_18;
    assert_eq!(value(&calculated, "schedule_3_line_6l"), line_18);
    assert_eq!(value(&calculated, "schedule_2_line_13z_form_8978"), -spill);
    assert_eq!(value(&calculated, "form_8978_unusable_decrease"), 0);
    assert_eq!(return_line(&calculated, "20"), line_18 * 100);
    assert_eq!(return_line(&calculated, "23"), (early_tax - spill) * 100);
    assert_eq!(calculated.tax, Some((early_tax - spill) * 100));
    assert_eq!(supported(&restart(draft.clone()).draft), calculated);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn bba_audit_spillover_uses_section_223_hsa_tax_but_not_section_4973_excise() {
    let mut hsa_case = complete(FederalStatus::Single);
    hsa_case.income = vec![wage(1, Owner::Taxpayer, 1_000_000, 0)];
    hsa_case.deductions.as_mut().unwrap().itemize_override = None;
    let hsa = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    add_hsa_distribution(
        &mut hsa_case,
        Owner::Taxpayer,
        0,
        vec![MedicalAccountWithdrawal { date: 20260630, amount: 100_000 }],
        None,
        None,
    );
    hsa_case.adjustments.as_mut().unwrap().hsa.push(hsa);
    let hsa_control = supported(&hsa_case);
    assert_eq!(return_line(&hsa_control, "18"), 0);
    assert_eq!(value(&hsa_control, "hsa_additional_tax"), 200);
    hsa_case.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_interest_reduction()));
    let hsa_result = supported(&hsa_case);
    assert_eq!(value(&hsa_result, "schedule_3_line_6l"), 0);
    assert_eq!(value(&hsa_result, "schedule_2_line_13z_form_8978"), -200);
    assert_eq!(value(&hsa_result, "form_8978_unusable_decrease"), 20);
    assert_eq!(return_line(&hsa_result, "23"), 0);
    assert_eq!(supported(&restart(hsa_case).draft), hsa_result);

    let mut excise_case = complete(FederalStatus::Single);
    excise_case.income = vec![wage(1, Owner::Taxpayer, 1_000_000, 0)];
    excise_case.deductions.as_mut().unwrap().itemize_override = None;
    let mut hsa = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    hsa.excess_accounting = Some(HsaExcessAccounting {
        prior_year: Some(HsaPriorExcess { line_48: 100_000, line_49: 6_000 }),
        employer_excess_in_wages: 0,
        year_end_value: 100_000,
        later_return_year_deposits: 0,
        later_employer_deposits: 0,
    });
    excise_case.adjustments.as_mut().unwrap().hsa.push(hsa);
    let excise_control = supported(&excise_case);
    assert_eq!(return_line(&excise_control, "18"), 0);
    assert_eq!(value(&excise_control, "hsa_return_year_excess_contribution_tax"), 60);
    excise_case.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_interest_reduction()));
    let excise_result = supported(&excise_case);
    assert_eq!(value(&excise_result, "schedule_2_line_13z_form_8978"), 0);
    assert_eq!(value(&excise_result, "form_8978_unusable_decrease"), 220);
    assert_eq!(return_line(&excise_result, "23"), 6_000);
    assert_eq!(supported(&restart(excise_case).draft), excise_result);
}

#[test]
fn bba_audit_negative_spillover_keeps_education_recapture_in_line_18() {
    let mut unclassified = complete(FederalStatus::Single);
    unclassified.income = vec![wage(1, Owner::Taxpayer, 1_000_000, 0)];
    unclassified.deductions.as_mut().unwrap().itemize_override = None;
    unclassified.credits.as_mut().unwrap().education_recapture =
        Some(PriorYearEducationCreditRecapture {
            method: EducationMethod::AmericanOpportunity,
            filed_qualified_expenses: 700_000,
            after_filing_reduction: 400_000,
            filed_education_magi: 5_000_000,
            filed_joint: false,
            filed_credit_used: 250_000,
            ordinary_prior_return: true,
        });
    let unclassified_baseline = supported(&unclassified);
    assert_eq!(value(&unclassified_baseline, "education_credit_recapture_tax"), 250);
    // The recapture is Form 1040 line 16 box 3, so it is line 18.
    assert_eq!(return_line(&unclassified_baseline, "18"), 25_000);
    // A decrease the recapture on line 18 absorbs (220 of 250) is credited on Schedule 3 line 6l.
    unclassified.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_interest_reduction()));
    let absorbed = supported(&unclassified);
    assert_eq!(value(&absorbed, "schedule_3_line_6l"), 220);
    assert_eq!(return_line(&absorbed, "18"), 25_000);
    assert_eq!(schedule_1_line(&absorbed, "26"), 0);
    // 2026-10-18: approved Unit18 §3A13z/D4; IRS2025 i1040 p114 negative-8978
    // worksheet (https://www.irs.gov/pub/irs-prior/i1040gi--2025.pdf).
    // Education recapture is already on line18, not eligible PartII tax.
    // 2026-10-18 fixture correction (§5): original inherited wages $63,750
    // plus $11,000 interest did not reconcile to the declared $50,000 taxable
    // income. $54,750 + $11,000 - 2025 standard deduction $15,750 = $50,000.
    unclassified.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(FederalBbaAuditInterest {
            interest_adjustment: -1_000_000,
            original_partnership_interest: 1_100_000,
            prior_wages: 5_475_000,
            ..bba_interest_reduction()
        }));
    let residual = supported(&unclassified);
    assert_eq!(value(&residual, "form_8978_line_14"), -1355);
    assert_eq!(value(&residual, "schedule_3_line_6l"), 250);
    assert_eq!(value(&residual, "schedule_2_line_13z_form_8978"), 0);
    assert_eq!(value(&residual, "form_8978_unusable_decrease"), 1105);
}

#[test]
fn bba_audit_negative_adjustment_shares_current_capacity_with_dc_carry() {
    let mut competing = complete(FederalStatus::Single);
    let credits = competing.credits.as_mut().unwrap();
    credits.bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_interest_reduction()));
    credits.dc_homebuyer_carry = Some(10_000);
    let r = supported(&competing);
    assert_eq!(value(&r, "schedule_3_line_6l"), 220);
    assert_eq!(value(&r, "dc_homebuyer_credit"), 100);
    assert_eq!(r.tax, Some(470_300));
    assert_eq!(supported(&restart(competing).draft), r);
}

#[test]
fn restaurant_tip_credit_uses_monthly_515_basis_and_reduces_schedule_c_deduction() {
    let draft = restaurant_tip_credit_draft();
    let claimed = supported(&draft);
    assert_eq!(value(&claimed, "business_2_form8846_hourly_basis"), 5);
    assert_eq!(value(&claimed, "business_2_form8846_line1"), 450);
    assert_eq!(value(&claimed, "business_2_form8846_line2"), 140);
    assert_eq!(value(&claimed, "business_2_form8846_line3"), 310);
    assert_eq!(value(&claimed, "business_2_form8846_line4"), 24);
    assert_eq!(line_amount(&claimed, "business_2_form8846_actual_employer_tip_fica_paid"), 3_444);
    assert_eq!(
        line_amount(&claimed, "business_2_form8846_remaining_employer_fica_deduction"),
        1_044
    );
    assert_eq!(value(&claimed, "business_2_form8846_remaining_employer_fica_deduction"), 10);
    assert_eq!(value(&claimed, "tip_fica_current_credit"), 24);
    assert_eq!(value(&claimed, "tip_fica_current_credit_used"), 24);
    assert_eq!(value(&claimed, "general_business_credit"), 24);
    assert_eq!(value(&claimed, "tip_fica_current_credit_carryforward_to_next_year"), 0);

    let mut no_claim = draft.clone();
    let business = restaurant_business_mut(&mut no_claim);
    business.employer_tip_fica_credit = None;
    // Without a credit, the entire $34.44 actually paid on the tips is an
    // ordinary expense. Claiming line 4 reduces that deduction by $24.00.
    business.expenses[0].amount += 3_444;
    let before = supported(&no_claim);
    assert_eq!(restaurant_business(&draft).expense_total(false), 38_544);
    assert_eq!(restaurant_business(&no_claim).expense_total(false), 40_944);
    assert!(value(&claimed, "agi") > value(&before, "agi"));
    assert!(value(&claimed, "self_employment_tax") >= value(&before, "self_employment_tax"));
    assert_eq!(supported(&restart(draft.clone()).draft), claimed);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn beauty_tip_credit_uses_725_basis_and_reconciled_tax_deduction() {
    let food = restaurant_tip_credit_draft();
    let beauty = beauty_tip_credit_draft();
    let food_result = supported(&food);
    let beauty_result = supported(&beauty);
    for (result, basis, excluded, credit, deduction) in
        [(&food_result, 515, 140, 24, 1_044), (&beauty_result, 725, 350, 8, 2_644)]
    {
        assert_eq!(
            result
                .lines
                .iter()
                .find(|line| { line.id == "business_2_form8846_hourly_basis" })
                .unwrap()
                .amount,
            basis
        );
        assert_eq!(value(result, "business_2_form8846_line1"), 450);
        assert_eq!(value(result, "business_2_form8846_line2"), excluded);
        assert_eq!(value(result, "business_2_form8846_line3"), 450 - excluded);
        assert_eq!(value(result, "business_2_form8846_line4"), credit);
        assert_eq!(value(result, "tip_fica_current_credit_used"), credit);
        assert_eq!(
            result
                .lines
                .iter()
                .find(|line| { line.id == "business_2_form8846_remaining_employer_fica_deduction" })
                .unwrap()
                .amount,
            deduction
        );
    }
    assert_eq!(restaurant_business(&beauty).expense_total(false), 40_144);
    assert_eq!(restaurant_business(&food).expense_total(false), 38_544);
    assert_eq!(supported(&restart(beauty.clone()).draft), beauty_result);
    assert_eq!(restart(beauty.clone()).draft, beauty);
}
#[test]
fn setup_page_keeps_basis_and_joint_to_single_drops_the_spouse() {
    let mut d = complete(FederalStatus::Joint);
    let id = d.next_id;
    add(&mut d, wage(id, Owner::Spouse, 2_000_000, 200_000));
    let mut s = Interview::new(TEST_USER, d.clone());
    let request = |value| P::SaveSetup {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: 0,
        page: 0,
        previous: 0,
        review: true,
        estimate: None,
        warnings: vec![],
        progress: vec![],
        value,
    };
    // The basis is no longer asked; resubmitting the page leaves the draft alone.
    send(&mut s, request(crate::interview::return_setup(d.setup.as_ref().unwrap())));
    assert_eq!(s.draft, d);
    assert!(!s.dirty);
    // Household removal retains the spouse's physical document and requests
    // a new association. Only an explicit Document center save removes it.
    let reply = send(&mut s, request(ReturnSetup { filing: Filing::Single }));
    assert_eq!(s.draft.income, d.income);
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    // The spouse's W-2 keeps its stored owner; the single return's own issue
    // names it, and Wages offers it to you.
    assert!(
        !crate::tests::support::builders::unknown_documents(&s.draft)
            .iter()
            .any(|(document, _)| *document == id)
    );
    let mut center = crate::tests::support::client::income_pages::open_document_center(&mut s);
    let P::SaveDocuments { documents, .. } = &mut center else { unreachable!() };
    documents.common.wages.documents.retain(|row| row.owner != crate::DocumentOwner::Spouses);
    crate::tests::cross::interview::accepted(&send(&mut s, center));
    assert_eq!(s.draft.setup.as_ref().unwrap().basis, AmountBasis::Estimate);
    assert_eq!(s.draft.people.as_ref().unwrap().spouse, SpouseFiler::No);
    assert_eq!(s.draft.people.as_ref().unwrap().taxpayer, d.people.as_ref().unwrap().taxpayer);
    assert_eq!(s.draft.income, d.income[..1]);
    assert!(s.draft.finished_sections.contains(&TaxSection::Income));
    assert_eq!(s.draft.first_incomplete(), 9);
    assert_eq!(estimate(&s.draft).state, ResultState::EstimatedForSupportedScope);
    let mut restarted = restart(s.draft);
    assert!(matches!(
        send(&mut restarted, P::OpenInterview { user: TEST_USER }),
        Reply::Response(P::Review { .. })
    ));
}

#[test]
fn school_bus_and_public_bus_fuel_rates_reach_direct_and_partner_form_4136() {
    let bus = FederalBusFuelUses {
        diesel_school_bus: Some(fuel_use(100_000, 30_000)),
        kerosene_school_bus: Some(fuel_use(100_000, 30_000)),
        emulsion_school_bus: Some(fuel_use(100_000, 30_000)),
        lpg_public_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::Gge {
            thousandths: 100_000,
        })),
        p_series_public_bus: Some(fuel_use(100_000, 30_000)),
        cng_public_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::Gge {
            thousandths: 100_000,
        })),
        hydrogen_public_bus: Some(fuel_use(100_000, 30_000)),
        fischer_tropsch_public_bus: Some(fuel_use(100_000, 30_000)),
        biomass_liquid_public_bus: Some(fuel_use(100_000, 30_000)),
        lng_public_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::Dge {
            thousandths: 100_000,
        })),
        biomass_gas_public_bus: Some(fuel_use(100_000, 30_000)),
        diesel_local_bus: None,
        kerosene_local_bus: None,
        emulsion_local_bus: None,
        local_bus_qualified: None,
        ..FederalBusFuelUses::default()
    };
    let physical_bus = FederalBusFuelUses {
        lpg_public_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::LiquidGallons {
            thousandths: 135_300,
        })),
        cng_public_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::CubicFeet {
            thousandths: 12_357_000,
        })),
        lng_public_bus: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::Pounds {
            thousandths: 606_000,
        })),
        ..FederalBusFuelUses::default()
    };
    let physical_fuels =
        FederalFuelLines { bus_uses: Some(physical_bus), ..FederalFuelLines::default() };
    assert!(physical_fuels.valid());
    let amount_for =
        |name| physical_fuels.labeled_lines().find(|(line, _)| *line == name).unwrap().1;
    assert_eq!(amount_for("11a"), 1_090);
    assert_eq!(amount_for("11c"), 1_090);
    assert_eq!(amount_for("11g"), 1_690);
    let fuels = FederalFuelLines {
        lpg_off_highway: Some(alternative_fuel_use(FederalAlternativeFuelQuantity::Gge {
            thousandths: 100_000,
        })),
        bus_uses: Some(bus),
        ..FederalFuelLines::default()
    };
    assert!(fuels.valid() && fuels.qualified());
    let business_fuels = fuels.clone();
    let partner_fuels = fuels;
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 38_500);
    for source in ["business_2", "partnership_3"] {
        for (form_line, expected) in [
            ("3a", 2_430),
            ("4a", 2_430),
            ("11a", 2_920),
            ("11b", 1_100),
            ("11c", 1_090),
            ("11d", 1_100),
            ("11e", 1_700),
            ("11f", 1_700),
            ("11g", 1_690),
            ("11h", 1_100),
            ("14a", 1_970),
        ] {
            assert_eq!(
                line_amount(&report, &alloc::format!("{source}_fuel_{form_line}")),
                expected,
                "{source} line {form_line}"
            );
        }
    }

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let buses = data.fuel_tax_credit.as_mut().unwrap().fuels.bus_uses.as_mut().unwrap();
    buses.cng_public_bus.as_mut().unwrap().actual_cost = 1;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let buses = data.fuel_tax_credit.as_mut().unwrap().fuels.bus_uses.as_mut().unwrap();
    buses.cng_public_bus.as_mut().unwrap().actual_cost = 30_000;
    buses.diesel_school_bus.as_mut().unwrap().not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let buses = data.fuel_tax_credit.as_mut().unwrap().fuels.bus_uses.as_mut().unwrap();
    buses.diesel_school_bus.as_mut().unwrap().not_claimed_elsewhere = true;
    buses.lng_public_bus.as_mut().unwrap().quantity =
        FederalAlternativeFuelQuantity::Gge { thousandths: 100_000 };
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
}

#[test]
fn exported_fuel_rows_use_export_rates_once_for_business_and_partner() {
    let fuels = FederalFuelLines {
        gasoline_export: Some(fuel_use(100_000, 20_000)),
        aviation_gasoline_export: Some(fuel_use(100_000, 20_000)),
        diesel_export: Some(fuel_use(100_000, 20_000)),
        kerosene_export: Some(fuel_use(100_000, 20_000)),
        emulsion_export: Some(fuel_use(100_000, 20_000)),
        export_proof_complete: Some(true),
        ..FederalFuelLines::default()
    };
    assert_eq!(
        fuels.labeled_lines().filter(|(_, v)| *v != 0).collect::<Vec<_>>(),
        vec![("1d", 1_840), ("2c", 1_940), ("3e", 2_440), ("4d", 2_440), ("14b", 1_980)]
    );
    let business_fuels = fuels.clone();
    let partner_fuels = fuels;
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 21_300);
    for source in ["business_2", "partnership_3"] {
        for (form_line, amount) in
            [("1d", 1_840), ("2c", 1_940), ("3e", 2_440), ("4d", 2_440), ("14b", 1_980)]
        {
            assert_eq!(line_amount(&report, &alloc::format!("{source}_fuel_{form_line}")), amount);
        }
    }

    for source in &mut draft.income[1..3] {
        let fuels = match &mut source.income {
            FederalIncome::Business { data } => &mut data.fuel_tax_credit.as_mut().unwrap().fuels,
            FederalIncome::Partnership { data } => {
                &mut data.fuel_tax_credit.as_mut().unwrap().fuels
            }
            _ => unreachable!(),
        };
        fuels.dyed_diesel_export = Some(fuel_use(1_000_000, 200_000));
        fuels.dyed_kerosene_export = Some(fuel_use(1_000_000, 200_000));
    }
    let with_dyed = supported(&draft);
    assert_eq!(with_dyed.tax, baseline.tax);
    assert_eq!(return_line(&with_dyed, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(return_line(&with_dyed, "31"), return_line(&baseline, "31") + 21_700);
    for source in ["business_2", "partnership_3"] {
        for form_line in ["16a", "16b"] {
            assert_eq!(line_amount(&with_dyed, &alloc::format!("{source}_fuel_{form_line}")), 100);
        }
    }
    assert_eq!(estimate(&restart(draft.clone()).draft), with_dyed);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.export_proof_complete = None;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.export_proof_complete = Some(false);
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.export_proof_complete = Some(true);
    fuels.kerosene_export.as_mut().unwrap().not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.kerosene_export.as_mut().unwrap().not_claimed_elsewhere = true;
    fuels.diesel_export.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let fuels = &mut data.fuel_tax_credit.as_mut().unwrap().fuels;
    fuels.diesel_export.as_mut().unwrap().actual_cost = 20_000;
    fuels.dyed_kerosene_export.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
}

#[test]
fn train_diesel_and_off_highway_emulsion_credit_direct_and_partner_claims() {
    let fuels = FederalFuelLines {
        diesel_trains: Some(fuel_use(50_500, 10_000)),
        emulsion_off_highway: Some(fuel_use(20_000, 4_000)),
        ..empty_fuel_lines()
    };
    // 50.5 train gallons × 24.3 cents = $12.2715; 20 emulsion gallons ×
    // 19.7 cents = $3.94. The two Form 4136 lines round to $16.21 per claim.
    assert_eq!(
        fuels.labeled_lines().filter(|(_, v)| *v != 0).collect::<Vec<_>>(),
        vec![("3c", 1227), ("14a", 394)]
    );
    let business_fuels = fuels.clone();
    let partner_fuels = fuels;
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 3_200);
    for source in ["business_2", "partnership_3"] {
        for (form_line, expected) in [("3c", 1227), ("14a", 394)] {
            assert_eq!(
                line_amount(&report, &alloc::format!("{source}_fuel_{form_line}")),
                expected
            );
        }
    }

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let claim = data.fuel_tax_credit.as_mut().unwrap();
    claim.fuels.diesel_trains.as_mut().unwrap().qualified = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let claim = data.fuel_tax_credit.as_mut().unwrap();
    claim.fuels.diesel_trains.as_mut().unwrap().qualified = true;
    claim.fuels.emulsion_off_highway.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
}

#[test]
fn five_gallon_based_alternative_fuels_reach_form_4136_for_business_and_partner() {
    let gallons = Some(fuel_use(10_000, 3_000));
    let fuels = FederalFuelLines {
        p_series_off_highway: gallons.clone(),
        hydrogen_off_highway: gallons.clone(),
        fischer_tropsch_off_highway: gallons.clone(),
        biomass_liquid_off_highway: gallons.clone(),
        biomass_gas_off_highway: gallons,
        ..empty_fuel_lines()
    };
    // Independent Form 4136 rates for 10 gallons on each line: $1.83,
    // $1.83, $2.43, $2.43 and $1.83, or $10.35 per source.
    assert_eq!(
        fuels.labeled_lines().filter(|(_, v)| *v != 0).collect::<Vec<_>>(),
        vec![("11b", 183), ("11d", 183), ("11e", 243), ("11f", 243), ("11h", 183)]
    );
    let business_fuels = fuels.clone();
    let partner_fuels = fuels;
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 2_100);
    for source in ["business_2", "partnership_3"] {
        for (form_line, expected) in
            [("11b", 183), ("11d", 183), ("11e", 243), ("11f", 243), ("11h", 183)]
        {
            assert_eq!(
                line_amount(&report, &alloc::format!("{source}_fuel_{form_line}")),
                expected
            );
        }
    }

    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .biomass_gas_off_highway
        .as_mut()
        .unwrap()
        .not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
}

#[test]
fn aviation_gasoline_type_10_reaches_direct_and_partner_2b_without_repeating_2a() {
    let fuels = FederalFuelLines {
        aviation_gasoline_commercial: Some(fuel_use(100_000, 20_000)),
        aviation_gasoline_type_10: Some(aviation_gasoline_type_10_uses()),
        ..FederalFuelLines::default()
    };
    assert_eq!(
        fuels.labeled_lines().filter(|(_, amount)| *amount != 0).collect::<Vec<_>>(),
        vec![("2a", 1_500), ("2b", 9_650)]
    );
    let business_fuels = fuels.clone();
    let partner_fuels = fuels;
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 22_300);
    for source in ["business_2", "partnership_3"] {
        for (line, amount) in [("2a", 1_500), ("2b", 9_650)] {
            assert_eq!(line_amount(&report, &alloc::format!("{source}_fuel_{line}")), amount);
        }
    }

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let type_10 =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_type_10.as_mut().unwrap();
    type_10.mineral_and_tree_airport_condition = Some(false);
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let type_10 =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_type_10.as_mut().unwrap();
    type_10.mineral_and_tree_airport_condition = Some(true);
    type_10.fixed_wing_acute_care_condition = None;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let type_10 =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_type_10.as_mut().unwrap();
    type_10.fixed_wing_acute_care_condition = Some(true);
    type_10.distinct_gallons = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let type_10 =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_type_10.as_mut().unwrap();
    type_10.distinct_gallons = true;
    type_10.helicopter_emergency_medical.as_mut().unwrap().qualified = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let type_10 =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_type_10.as_mut().unwrap();
    type_10.helicopter_emergency_medical.as_mut().unwrap().qualified = true;
    type_10.helicopter_tree_work.as_mut().unwrap().not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let type_10 =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_type_10.as_mut().unwrap();
    type_10.helicopter_tree_work.as_mut().unwrap().not_claimed_elsewhere = true;
    type_10.helicopter_tree_work.as_mut().unwrap().actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let type_10 =
        data.fuel_tax_credit.as_mut().unwrap().fuels.aviation_gasoline_type_10.as_mut().unwrap();
    type_10.helicopter_tree_work.as_mut().unwrap().actual_cost = 20_000;
    type_10.helicopter_tree_work.as_mut().unwrap().gallons = 0.0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .aviation_gasoline_type_10
        .as_mut()
        .unwrap()
        .helicopter_tree_work
        .as_mut()
        .unwrap()
        .gallons = 100.0;

    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().statement_complete = false;
    let r = supported(&draft);
    assert!(noted(&r, "WARN_PARTNER_FUEL_CREDIT_LEFT_OUT"), "{r:?}");
}

#[test]
fn off_highway_kerosene_preserves_original_tax_rate_for_business_and_partner() {
    let fuels = FederalFuelLines {
        kerosene_off_highway_taxed_044: Some(fuel_use(100_000, 20_000)),
        kerosene_off_highway_taxed_219: Some(fuel_use(100_000, 20_000)),
        ..empty_fuel_lines()
    };
    assert_eq!(
        fuels.labeled_lines().filter(|(_, v)| v != &0).collect::<Vec<_>>(),
        vec![("4e", 430), ("4f", 2_180)]
    );
    let business_fuels = fuels.clone();
    let partner_fuels = fuels;
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = assert_fuel_payment(&draft, &baseline, 5_200);
    for source in ["business_2", "partnership_3"] {
        for (form_line, amount) in [("4e", 430), ("4f", 2_180)] {
            assert_eq!(line_amount(&report, &alloc::format!("{source}_fuel_{form_line}")), amount);
        }
    }

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .kerosene_off_highway_taxed_044
        .as_mut()
        .unwrap()
        .not_claimed_elsewhere = false;
    fuel_use_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    let line = data
        .fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .kerosene_off_highway_taxed_044
        .as_mut()
        .unwrap();
    line.not_claimed_elsewhere = true;
    line.actual_cost = 0;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
}

#[test]
fn standard_business_mileage_uses_midyear_rates_and_separate_basis_component() {
    let vehicle = BusinessMileage {
        id: 0,
        commuting: 3000,
        parking_tolls: 20_000,
        interest: 100_000,
        property_tax: 50_000,
        ..native_business_mileage(10000, 10000, 25000)
    };
    assert!(vehicle.valid());
    assert_eq!(vehicle.expense(), 1_625_000); // 7250 + 7600 + 200 + 1200.
    assert_eq!(vehicle.basis_reduction(), 700_000);
    let mut d = complete(FederalStatus::Single);
    let mut business = cash_business(50000);
    let FederalIncome::Business { data } = &mut business else { panic!() };
    data.vehicles = vec![vehicle.clone()];
    add(&mut d, income(business));
    let r = supported(&d);
    let mut reference = d.clone();
    reference.income[1].income = cash_business(33750);
    assert_eq!(r.tax, supported(&reference).tax);
    assert_eq!(restart(d.clone()).draft, d);
    let mut leased = vehicle.clone();
    leased.owned = false;
    assert!(!leased.valid());
    leased.interest = 0;
    assert!(leased.valid());
    assert_eq!(leased.basis_reduction(), 0);
    let mut invalid = vehicle;
    invalid.commuting = 5001;
    assert!(!invalid.valid());
}

#[test]
fn first_home_exception_caps_gross_costs_with_basis_and_checks_history() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        retirement_entry(
            Owner::Taxpayer,
            RetirementKind::Ira,
            10000,
            RetirementTreatment::Exception {
                second: None,
                distributed: 20260101,
                facts: RetirementException::FirstHome {
                    costs: 200000,
                    paid_on: 20260201,
                    prior_used: 0,
                    eligible: true,
                },
                early_simple: false,
            },
        ),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 800);
    assert_eq!(value(&r, "first_home_used_taxpayer"), 2000);
    let mut another = d.income[1].clone();
    let FederalIncome::Retirement { data } = &mut another.income else { unreachable!() };
    {
        let mut data = edit_retirement(data);
        let Some(RetirementTreatment::Exception { facts, .. }) = &mut data.ordinary_distribution
        else {
            unreachable!()
        };
        let RetirementException::FirstHome { prior_used, .. } = facts else { unreachable!() };
        *prior_used = 10000;
    }
    add(&mut d, another);
    unsupported(&d, "Income");
    d.income.pop();
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut data = edit_retirement(data);
        data.box_2a = 900000;
        data.taxable = Some(RetirementTaxable::Payer);
    }
    let r = supported(&d);
    assert_eq!(value(&r, "early_distribution_tax"), 720);
    assert_eq!(value(&r, "first_home_used_taxpayer"), 2000);
}

#[test]
fn undistributed_fund_gain_reports_income_credit_and_share_basis_once() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(undistributed_gain()));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 65000);
    assert_eq!(value(&r, "capital_gain"), 5000);
    assert_eq!(value(&r, "rate_28_gain"), 3000);
    assert_eq!(value(&r, "income_tax"), 5383);
    assert_eq!(value(&r, "undistributed_gain_tax_credit"), 1050);
    let basis_id = alloc::format!("fund_{}_basis_increase", d.income[1].id);
    assert_eq!(value(&r, &basis_id), 3950);
    assert_eq!(return_line(&r, "25d"), 650000);
    assert_eq!(return_line(&r, "31"), 105000);
    assert_eq!(r.payments, Some(755000));
    assert_eq!(restart(d.clone()).draft, d);
    // Capital losses can eliminate gain without eliminating the fund tax credit
    // or changing this particular holding's cost-basis adjustment.
    add(&mut d, income(collectible_sale(0, 10000, GainTerm::Short, false)));
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "rate_28_gain"), 0);
    assert_eq!(value(&r, &basis_id), 3950);
    assert_eq!(r.payments, Some(755000));
}

#[test]
fn noncash_liquidation_includes_property_once_and_preserves_its_new_basis() {
    let mut d = complete(FederalStatus::Single);
    let mut distribution = capital_return(3000, 4000, GainTerm::Long);
    let FederalIncome::ReturnOfCapital { kind, property, .. } = &mut distribution else { panic!() };
    *kind = CapitalDistributionKind::FinalLiquidation;
    *property = vec![LiquidationProperty { label: "Distributed shares".into(), value: 600000 }];
    add(&mut d, income(distribution));
    let id = d.income[1].id;
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 5000);
    assert_eq!(value(&r, &alloc::format!("distribution_{id}_property_0_basis")), 6000);
    assert_eq!(crate::interview::document_amount(&d.income[1].income, false), Some(500000));
    assert_eq!(restart(d.clone()).draft, d);
    let FederalIncome::ReturnOfCapital { basis, .. } = &mut d.income[1].income else { panic!() };
    *basis = 1500000;
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 3000);
    assert_eq!(value(&r, &alloc::format!("distribution_{id}_property_0_basis")), 6000);
}
#[test]
fn noncash_liquidation_keeps_interim_basis_and_rejects_nonliquidating_use() {
    let mut d = complete(FederalStatus::Single);
    let mut distribution = capital_return(0, 10000, GainTerm::Long);
    let FederalIncome::ReturnOfCapital { kind, property, .. } = &mut distribution else { panic!() };
    *kind = CapitalDistributionKind::InterimLiquidation;
    *property = vec![LiquidationProperty { label: "Property".into(), value: 400000 }];
    add(&mut d, income(distribution));
    let r = supported(&d);
    let id = d.income[1].id;
    assert_eq!(value(&r, "capital_gain"), 0);
    assert_eq!(value(&r, &alloc::format!("distribution_{id}_remaining_basis")), 6000);
    let FederalIncome::ReturnOfCapital { kind, .. } = &mut d.income[1].income else { panic!() };
    *kind = CapitalDistributionKind::Nondividend;
    unsupported(&d, "Income:");
}

#[test]
fn at_par_purchased_debt_reduces_basis_for_principal_collected() {
    let mut d = complete(FederalStatus::Single);
    let mut debt = purchased_bad_debt(10000, 10000, 200);
    let FederalIncome::NonbusinessBadDebt {
        data:
            crate::NonbusinessBadDebt {
                source: BadDebtSource::PurchasedNonsecurity { principal_collected, .. },
                ..
            },
    } = &mut debt
    else {
        panic!()
    };
    *principal_collected = 300_000;
    add(&mut d, income(debt));
    let r = supported(&d);
    assert_eq!(value(&r, "bad_debt_2_short_term_loss"), -7200);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 4200);
    assert_eq!(restart(d.clone()).draft, d);

    let FederalIncome::NonbusinessBadDebt {
        data:
            crate::NonbusinessBadDebt {
                source: BadDebtSource::PurchasedNonsecurity { price_paid, principal_collected, .. },
                ..
            },
    } = &mut d.income[1].income
    else {
        panic!()
    };
    *price_paid = 400_000;
    *principal_collected = 100_000;
    // A discounted purchase with a collection is a page rule the person can
    // fix, not an estimate refusal.
    assert!(!d.valid_stored());
    let FederalIncome::NonbusinessBadDebt { data } = &d.income[1].income else { panic!() };
    assert_eq!(data.rule(), Some("BAD_DEBT_COLLECTED_NEEDS_FACE_PRICE"));
}

#[test]
fn purchased_debt_requires_acquisition_value_nonsecurity_and_unchanged_basis() {
    for case in 0..5 {
        let mut d = complete(FederalStatus::Single);
        let mut debt = purchased_bad_debt(10000, 4000, 200);
        let FederalIncome::NonbusinessBadDebt { data: crate::NonbusinessBadDebt { source, .. } } =
            &mut debt
        else {
            panic!()
        };
        let BadDebtSource::PurchasedNonsecurity {
            had_value_when_bought,
            security,
            basis_adjustments,
            ..
        } = source
        else {
            panic!()
        };
        match case {
            0 => *had_value_when_bought = false,
            1 => *security = true,
            2 => *basis_adjustments = true,
            3 => *security = false,
            4 => *basis_adjustments = false,
            _ => unreachable!(),
        }
        add(&mut d, income(debt));
        // No value when bought takes no loss; a security or other basis
        // changes leave the loss out with a notice.
        let r = supported(&d);
        assert_eq!(r.lines.iter().any(|line| line.id.starts_with("bad_debt_")), case >= 3);
        assert_eq!(noted(&r, "WARN_BAD_DEBT_LEFT_OUT"), case == 1 || case == 2);
    }
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(purchased_bad_debt(10000, 4000, 200)));
    let FederalIncome::NonbusinessBadDebt {
        data:
            crate::NonbusinessBadDebt {
                source: BadDebtSource::PurchasedNonsecurity { price_paid, acquisition_costs, .. },
                ..
            },
    } = &mut d.income[1].income
    else {
        panic!()
    };
    *price_paid = 100_000_000_000;
    *acquisition_costs = 1;
    assert!(!d.valid_stored());
}

#[test]
fn insolvent_personal_contractor_deposit_uses_unreturned_cash_basis() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(contractor_bad_debt(6000, 1000)));
    let r = supported(&d);
    assert_eq!(value(&r, "bad_debt_2_short_term_loss"), -5000);
    assert_eq!(value(&r, "capital_gain"), -3000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 2000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn joint_survivor_cola_is_taxable_and_does_not_change_basis_cap() {
    let mut a = joint_annuity(true);
    a.increase = 6000; // Actual increase received during only part of the year.
    assert!(a.valid(426000));
    assert_eq!(a.excluded(), 217140);
    a.recovered = a.cost - 100;
    assert_eq!(a.excluded(), 100);
}

#[test]
fn estate_paid_medical_election_flows_through_agi_floor() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20260615, ordinary: true, estate_medical: 3000000 });
    let r = supported(&d);
    assert_eq!(value(&r, "estate_paid_medical_election"), 30000);
    assert_eq!(return_line(&r, "12e"), 2550000); // $30,000 - 7.5% of $60,000.
    assert_eq!(restart(d.clone()).draft, d);
    d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().estate_medical = 0;
    // Zero is no election: nothing is deducted here.
    let r = supported(&d);
    assert!(r.lines.iter().all(|l| l.id != "estate_paid_medical_election"));
}

#[test]
fn old_general_rule_pension_applies_public_safety_after_basis_recovery() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 19280101;
    add(&mut d, old_general_entry(RetirementKind::Pension, old_general_annuity(19930101, 500000)));
    let id = d.income.last().unwrap().id;
    public_safety(&mut d, id, 3000, 3000, PublicSafetyCoverage::Medical);
    let r = supported(&d);
    assert_eq!(value(&r, &alloc::format!("annuity_{id}_basis_recovery")), 504);
    assert_eq!(value(&r, "public_safety_1_excluded"), 3000);
    assert_eq!(return_line(&r, "5b"), 849600);
    assert_eq!(return_line(&r, "5a"), 1200000);
}

#[test]
fn nonqualified_disability_exception_preserves_calculated_basis_recovery() {
    let mut d = nonqualified_exception(RetirementException::Disability { began: 20250101 });
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
    {
        let mut data = edit_retirement(data);
        data.gross = 300000;
        data.taxable = Some(RetirementTaxable::GeneralAnnuity { data: fixed_annuity() });
    }
    let r = supported(&d);
    assert_eq!(value(&r, "annuity_2_basis_recovery"), 1500);
    assert_eq!(return_line(&r, "5b"), 150000);
    assert_eq!(value(&r, "net_investment_income_tax"), 57);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(return_line(&r, "25d"), 21300); // $123 annuity plus $90 Additional Medicare withholding.
}

#[test]
fn qualified_withdrawal_coordinates_basis_and_public_safety_exclusion() {
    let mut d = qualified_withdrawal(5000, 12500, 10000);
    public_safety(&mut d, 2, 2000, 2000, PublicSafetyCoverage::Medical);
    let r = supported(&d);
    assert_eq!(value(&r, "pension_2_basis_recovery"), 4000);
    assert_eq!(value(&r, "public_safety_1_excluded"), 1000);
    assert_eq!(value(&r, "public_safety_medical_before_coordination"), 1000);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
}

#[test]
fn incoming_employer_basis_flows_through_ira_distribution_and_conversion() {
    for treatment in [
        RetirementTreatment::Ordinary,
        RetirementTreatment::Conversion { amount: 500000, early: false },
    ] {
        let mut d = incoming_plan_rollover();
        add(&mut d, basis_distribution(Owner::Taxpayer, 5000, treatment));
        let r = supported(&d);
        assert_eq!(return_line(&r, "5b"), 0);
        assert_eq!(return_line(&r, "4b"), 400000);
        assert_eq!(value(&r, "taxpayer_ira_incoming_plan_basis"), 4000);
        assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 1000);
        assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 3000);
        assert_eq!(value(&r, "pension_2_remaining_basis"), 6000);
        assert_eq!(restart(d.clone()).draft, d);
        d.income.reverse();
        assert_eq!(supported(&d).tax, r.tax);
    }
    let r = supported(&incoming_plan_rollover());
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 4000);
}

#[test]
fn incoming_employer_basis_is_not_available_for_qcd_pretax_exclusion() {
    let mut d = incoming_plan_rollover();
    d.people.as_mut().unwrap().taxpayer.birth_date = 19500101;
    add(&mut d, basis_distribution(Owner::Taxpayer, 20000, RetirementTreatment::Ordinary));
    let a = d.adjustments.as_mut().unwrap();
    a.ira_basis[0].year_end = 0;
    a.qualified_charity = vec![qcd(Owner::Taxpayer, 20261215, 20000, 0)];
    let r = supported(&d);
    assert_eq!(value(&r, "qcd_exclusion"), 16000);
    assert_eq!(value(&r, "qcd_cash_gifts"), 4000);
    assert_eq!(value(&r, "taxpayer_ira_basis_recovered"), 4000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
    assert_eq!(return_line(&r, "4b"), 0);
}

#[test]
fn incoming_employer_basis_requires_owner_record_and_supported_deposit_year() {
    let mut d = incoming_plan_rollover();
    d.adjustments.as_mut().unwrap().ira_basis.clear();
    unsupported(&d, "IRA basis");
    let mut wrong_owner = incoming_plan_rollover();
    wrong_owner.adjustments.as_mut().unwrap().ira_basis[0].owner = Owner::Spouse;
    unsupported(&wrong_owner, "IRA basis");
    {
        let destination = RolloverDestination::EmployerPlan;
        let mut d = incoming_plan_rollover();
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
        {
            let mut data = edit_retirement(data);
            let Some(RetirementTreatment::Rollover { destination: target, .. }) =
                &mut data.ordinary_distribution
            else {
                panic!()
            };
            *target = destination;
        }
        unsupported(&d, "Income:");
    }
}

#[test]
fn de_minimis_uses_full_item_cost_and_does_not_create_qbi_property_basis() {
    let mut d = de_minimis_business(250001, false);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    data.expenses[0].amount = 100000; // Business allocation cannot rescue an over-limit item.
    unsupported(&d, "Income:");
    let mut d = de_minimis_business(250000, false);
    d.income[0] = wage(1, Owner::Taxpayer, 50000000, 0);
    // The 2026 active-business minimum survives, but expensed property adds no UBIA.
    assert_eq!(value(&supported(&d), "qbi_deduction"), 400);
    let FederalIncome::Business { data } = &mut d.income[1].income else { panic!() };
    match &mut data.activity {
        BusinessActivity::Ordinary { receipts, .. }
        | BusinessActivity::Gambling { receipts, .. } => *receipts = 100000,
        _ => panic!("expected nonfarm test business"),
    };
    d.income[0] = wage(1, Owner::Taxpayer, 6000000, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 41000);
    assert_eq!(value(&r, "self_employment_tax"), 0);
}
#[test]
fn insolvency_exclusion_preserves_pub4681_personal_basis_limit_and_allocation() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, insolvency_debt(1500, 7500, 4600, 6100));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(value(&r, "debt_2_insolvency_exclusion"), 1500);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 100);
    assert_eq!(line_amount(&r, "debt_2_property_0_reduction"), 9091);
    assert_eq!(line_amount(&r, "debt_2_property_1_reduction"), 909);
    assert_eq!(line_amount(&r, "debt_2_property_0_remaining_basis"), 490909);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn insolvency_partial_exclusion_and_basis_reduction_do_not_double_reduce_income() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, insolvency_debt(5000, 10000, 8000, 20000));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), value(&before, "agi") + 3000);
    assert_eq!(value(&r, "debt_2_insolvency_exclusion"), 2000);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 2000);
    assert_eq!(value(&r, "personal_debt_cancellation"), 3000);
}
#[test]
fn insolvency_basis_reduction_respects_zero_limit_and_zero_assets() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, insolvency_debt(1500, 7500, 4600, 5000));
    assert_eq!(value(&supported(&d), "debt_2_basis_reduction"), 0);
    if let FederalIncome::PersonalDebt { data } = &mut d.income[1].income
        && let Some(crate::CanceledDebtChoice::Insolvent { property, basis_and_cash_after, .. }) =
            &mut data.treatment
    {
        property.clear();
        *basis_and_cash_after = 1000000;
    }
    assert_eq!(value(&supported(&d), "debt_2_basis_reduction"), 0);
}
#[test]
fn insolvency_asset_order_does_not_change_allocated_basis_and_insolvency_is_not_assumed() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, insolvency_debt(1500, 7500, 7600, 6100));
    assert_eq!(value(&supported(&d), "personal_debt_cancellation"), 1500);
    if let FederalIncome::PersonalDebt { data } = &mut d.income[1].income
        && let Some(crate::CanceledDebtChoice::Insolvent { assets, property, .. }) =
            &mut data.treatment
    {
        *assets = 460000;
        property.reverse();
    }
    let r = supported(&d);
    assert_eq!(line_amount(&r, "debt_2_property_1_reduction"), 9091);
}
#[test]
fn bankruptcy_personal_debt_exclusion_is_not_capped_by_solvency_or_basis() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, bankruptcy_debt(20000, 10000, 0));
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(value(&r, "debt_2_bankruptcy_exclusion"), 20000);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 5500);
    assert_eq!(value(&r, "debt_2_property_0_remaining_basis"), 0);
    assert_eq!(value(&r, "debt_2_property_1_remaining_basis"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn bankruptcy_basis_limit_uses_surviving_liabilities_and_never_creates_a_deduction() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, bankruptcy_debt(1500, 6100, 6000));
    let r = supported(&d);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 100);
    assert_eq!(line_amount(&r, "debt_2_property_0_reduction"), 9091);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    if let FederalIncome::PersonalDebt { data } = &mut d.income[1].income
        && let Some(crate::CanceledDebtChoice::Bankruptcy { liabilities_after, .. }) =
            &mut data.treatment
    {
        *liabilities_after = 700000;
    }
    assert_eq!(value(&supported(&d), "debt_2_basis_reduction"), 0);
}
#[test]
fn seller_price_reduction_changes_basis_without_taxable_income_or_form982() {
    let mut d = complete(FederalStatus::Single);
    let before = supported(&d);
    add(&mut d, price_reduction_debt());
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), value(&before, "agi"));
    assert_eq!(value(&r, "debt_2_price_reduction"), 2000);
    assert_eq!(value(&r, "debt_2_remaining_basis"), 8000);
    assert!(!r.lines.iter().any(|v| v.id == "debt_2_basis_reduction"));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn seller_price_adjustment_rejects_insolvency_insufficient_basis_and_unconfirmed_facts() {
    for which in 0..3 {
        let mut d = complete(FederalStatus::Single);
        let mut entry = price_reduction_debt();
        if let FederalIncome::PersonalDebt { data } = &mut entry.income
            && let Some(crate::CanceledDebtChoice::SellerReduction {
                basis,
                assets,
                liabilities,
                ..
            }) = &mut data.treatment
        {
            match which {
                0 => *assets = *liabilities - 1,
                1 => *basis = data.form.amount - 1,
                _ => *liabilities = data.form.amount - 1,
            }
        }
        add(&mut d, entry);
        unsupported(&d, "Seller purchase-price adjustment");
    }
}
#[test]
fn seller_price_adjustment_accepts_exact_solvency_and_preserves_cent_basis() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = price_reduction_debt();
    if let FederalIncome::PersonalDebt { data } = &mut entry.income
        && let Some(crate::CanceledDebtChoice::SellerReduction {
            basis, assets, liabilities, ..
        }) = &mut data.treatment
    {
        data.form.amount = 199999;
        *assets = *liabilities;
        *basis = 200000;
    }
    add(&mut d, entry);
    let r = supported(&d);
    assert_eq!(line_amount(&r, "debt_2_remaining_basis"), 1);
}

#[test]
fn personal_property_profit_offsets_real_estate_losses_without_duplicate_income() {
    let mut d = complete(FederalStatus::Single);
    let mut loss = rental(1000, 6000); // 15,000 loss after depreciation.
    loss.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: loss }));
    add(&mut d, income(personal_rent(12000, 2000)));
    let r = supported(&d);
    assert_eq!(value(&r, "personal_rental_passive_income"), 10000);
    assert_eq!(value(&r, "rental_income"), -10000);
    assert_eq!(value(&r, "rental_suspended_loss"), 5000);
    assert_eq!(value(&r, "amt_rental_income"), -10000);
    assert_eq!(value(&r, "amt_rental_suspended_loss"), 5000);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn real_estate_profit_releases_personal_rental_expenses_on_schedule_one() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_rent(1000, 13000)));
    add(&mut d, income(FederalIncome::Rental { data: rental(20000, 0) }));
    let r = supported(&d);
    assert_eq!(value(&r, "rental_income"), 10000);
    assert_eq!(value(&r, "nonbusiness_personal_rent_expenses"), 11000);
    assert_eq!(value(&r, "personal_rental_2_suspended_loss"), 2000);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(return_line(&r, "8"), 1100000);
    assert_eq!(return_line(&r, "10"), 1100000);
}

#[test]
fn final_return_suspended_personal_rental_loss_ends_with_the_return() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20260615, ordinary: true, estate_medical: 0 });
    add(&mut d, income(personal_rent(1000, 13000)));
    // The suspended loss ends with the final return: computed, with the note.
    assert!(noted(&supported(&d), "NOTE_FINAL_RETURN_CARRYOVERS_END"));
}

#[test]
fn zero_basis_personal_rental_remains_passive_under_strict_basis_test() {
    let mut d = complete(FederalStatus::Single);
    let mut rent = personal_rent(10000, 2000);
    let FederalIncome::NonbusinessProperty { kind, .. } = &mut rent else { panic!() };
    *kind = NonbusinessPropertyKind::PersonalRental {
        related: alloc::vec![],
        basis: 0,
        depreciable_basis: 0,
        assets: vec![],
        history: PersonalRentalHistory::NONE,
    };
    add(&mut d, income(rent));
    let mut loss = rental(1000, 6000);
    loss.participation = crate::RentalParticipation::NotActive;
    add(&mut d, income(FederalIncome::Rental { data: loss }));
    investment_borrowing(&mut d, 10000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "personal_rental_passive_income"), 8000);
    assert_eq!(value(&r, "personal_rent_2_investment_income"), 0);
    assert_eq!(value(&r, "investment_interest_deduction"), 0);
    assert_eq!(value(&r, "rental_income"), -8000);
    assert_eq!(value(&r, "rental_suspended_loss"), 7000);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn older_continuing_equipment_keeps_its_table_basis_and_convention() {
    let mut asset = continuing_equipment(20161001, 456250, true);
    asset.cost = 1000000;
    asset.acquired = 20160101;
    asset.class = EquipmentClass::Twenty;
    asset.method = EquipmentMethod::StraightLine;
    asset.history = EquipmentHistory::Continuing {
        amt: EquipmentAmtHistory::Same,
        prior_depreciation: 456250,
        initial_section179: 0,
        carryforward: 0,
        mid_quarter: true,
        schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
    };
    // Table straight line on the original $10,000 is $500, not 5% of
    // remaining $5,437.50. This continuing asset is outside the 2026 MQ test.
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(personal_rent_with_assets(5000, 0, vec![asset])));
    let r = supported(&d);
    assert_eq!(value(&r, "personal_rental_2_asset_0_macrs"), 500);
    assert_eq!(line_amount(&r, "personal_rental_2_asset_0_remaining_basis"), 493750);
    assert!(!crate::depreciation::mid_quarter(&d));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn separate_amt_asset_uses_its_own_original_table_basis() {
    let mut asset = separate_amt_equipment();
    if let EquipmentHistory::Continuing { amt, .. } = &mut asset.history {
        *amt = EquipmentAmtHistory::Separate {
            prior_depreciation: 868900,
            schedule: EquipmentSchedule::Tables { initial_bonus: 0 },
        };
    }
    // Pub. 946 Table A-14: year 10 is 8.74% of original basis; year 11
    // recovers the remaining 4.37%. Regular manual history stays separate.
    let d = business_with_bonus(50000, asset);
    let r = supported(&d);
    assert_eq!(line_amount(&r, "business_2_asset_0_amt_depreciation"), 87400);
    assert_eq!(line_amount(&r, "business_2_asset_0_amt_remaining_basis"), 43700);
    assert_eq!(line_amount(&r, "business_2_asset_0_macrs"), 65536);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn prior_nol_does_not_reduce_ira_taxable_basis_or_early_distribution_tax() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, basis_distribution(Owner::Taxpayer, 20000, RetirementTreatment::Early));
    d.adjustments.as_mut().unwrap().ira_basis =
        vec![prior_ira_basis(Owner::Taxpayer, 20000, 80000)];
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 1600000);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 16000);
    assert_eq!(value(&r, "prior_nol_deduction"), 47920);
    assert_eq!(value(&r, "agi"), 28080);
    assert_eq!(value(&r, "early_distribution_tax"), 1600);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_preserves_inherited_ira_basis_and_death_exception() {
    let mut d = complete(FederalStatus::Single);
    add_inherited_ira(&mut d, 10000, 5000, 40000);
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
    let r = supported(&d);
    assert_eq!(return_line(&r, "4b"), 900000);
    assert_eq!(value(&r, "inherited_ira_2_basis_remaining"), 4000);
    assert_eq!(value(&r, "prior_nol_deduction"), 42320);
    assert_eq!(value(&r, "agi"), 26680);
    assert_eq!(value(&r, "nol_2025_carryforward"), 57680);
    assert_eq!(value(&r, "early_distribution_tax"), 0);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_preserves_inherited_estate_tax_deduction_and_consumed_value() {
    let mut d = complete(FederalStatus::Single);
    add_inherited_ira(&mut d, 12000, 0, 0);
    let ded = d.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    set_inherited_estate(&mut d, inherited_estate(12000));
    let before = supported(&d);
    let base = value(&before, "taxable_income");
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "inherited_estate_tax_deduction"), 2772);
    assert_eq!(value(&r, "inherited_ira_2_estate_ird_value_remaining"), 0);
    assert_eq!(return_line(&r, "4b"), 1200000);
    assert_eq!(value(&r, "agi"), -28000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 100000 - base);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_capital_distributions_recover_basis_before_limit_income() {
    for (amount, gain, remaining, deduction) in [(1000, 0, 4000, 35120), (8000, 3000, 0, 37520)] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(capital_return(amount, 5000, GainTerm::Long)));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "distribution_2_remaining_basis"), remaining);
        assert_eq!(value(&r, "capital_gain"), gain);
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, "nol_2025_carryforward"), 100000 - deduction);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_liquidation_finality_controls_basis_and_loss_carryforward() {
    let mut d = complete(FederalStatus::Single);
    let mut entry = capital_return(2000, 10000, GainTerm::Long);
    if let FederalIncome::ReturnOfCapital { kind, .. } = &mut entry {
        *kind = CapitalDistributionKind::InterimLiquidation;
    }
    add(&mut d, income(entry));
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "distribution_2_remaining_basis"), 8000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 0);
    if let FederalIncome::ReturnOfCapital { kind, .. } = &mut d.income[1].income {
        *kind = CapitalDistributionKind::FinalLiquidation;
    }
    let r = supported(&d);
    assert_eq!(value(&r, "distribution_2_remaining_basis"), 0);
    assert_eq!(value(&r, "capital_loss_carryforward"), 8000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 56100);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn prior_nol_undistributed_gain_preserves_special_rates_credit_and_basis() {
    for (year, deduction, closing, tax) in [(2025, 39120, 60880, 978), (2017, 100000, 51100, 0)] {
        let mut d = complete(FederalStatus::Single);
        let mut fund = undistributed_gain();
        let FederalIncome::UndistributedGain { data } = &mut fund else { panic!() };
        data.form.unrecaptured_section_1250_gain = 200000;
        add(&mut d, income(fund));
        d.screening.as_mut().unwrap().nol = prior_nol(&[(year, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), 5000);
        assert_eq!(value(&r, "rate_28_gain"), 3000);
        assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 2000);
        assert_eq!(value(&r, "prior_nol_deduction"), deduction);
        assert_eq!(value(&r, "agi"), 65000 - deduction);
        assert_eq!(value(&r, "regular_income_tax"), tax);
        assert_eq!(value(&r, "undistributed_gain_tax_credit"), 1050);
        assert_eq!(value(&r, "fund_2_basis_increase"), 3950);
        assert_eq!(value(&r, &alloc::format!("nol_{year}_carryforward")), closing);
        assert_eq!(r.payments, Some(755000));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn prior_nol_seller_price_adjustment_changes_basis_without_consuming_loss() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, price_reduction_debt());
    d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
    let r = supported(&d);
    assert_eq!(value(&r, "debt_2_remaining_basis"), 8000);
    assert_eq!(value(&r, "personal_debt_cancellation"), 0);
    assert_eq!(value(&r, "agi"), -40000);
    assert_eq!(value(&r, "nol_2017_carryforward"), 56100);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn debt_exclusions_reduce_prior_nol_after_current_tax_before_basis() {
    for (entry, section) in [
        (insolvency_debt(1500, 7500, 4600, 6100), "Insolvency tax attributes"),
        (bankruptcy_debt(1500, 6100, 6000), "Bankruptcy tax attributes"),
    ] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, entry);
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 100000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "nol_2017_carryforward"), 54600);
        assert_eq!(value(&r, "nol_2017_debt_reduction"), 1500);
        assert_eq!(value(&r, "regular_total_nol_carryforward"), 54600);
        assert_eq!(value(&r, "prior_nol_deduction"), 100000);
        assert_eq!(value(&r, "agi"), -40000);
        assert_eq!(value(&r, "debt_2_basis_reduction"), 0);
        let mut joint = complete(FederalStatus::Joint);
        joint.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 200000)]);
        add(&mut joint, d.income[1].clone());
        unsupported(&joint, section);
        // Attribute reduction follows the current return: an entirely used
        // prior NOL leaves no remaining NOL to reduce.
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 20000)]);
        let r = supported(&d);
        assert_eq!(value(&r, "nol_2017_carryforward"), 0);
        assert_eq!(value(&r, "agi"), 40000);
        assert_eq!(value(&r, "debt_2_basis_reduction"), 100);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn debt_exclusion_remaining_after_nol_reduces_personal_basis_once() {
    for entry in [insolvency_debt(1500, 7500, 4600, 10000), bankruptcy_debt(1500, 10000, 6000)] {
        let mut d = complete(FederalStatus::Single);
        d.screening.as_mut().unwrap().nol = prior_nol(&[(2017, 45000)]);
        add(&mut d, entry);
        let r = supported(&d);
        assert_eq!(value(&r, "nol_2017_debt_reduction"), 1100);
        assert_eq!(value(&r, "regular_total_nol_carryforward"), 0);
        assert_eq!(value(&r, "debt_2_regular_nol_reduction"), 1100);
        assert_eq!(value(&r, "debt_2_basis_reduction"), 400);
        let remaining: i64 = r
            .lines
            .iter()
            .filter(|l| l.id.starts_with("debt_2_property_") && l.id.ends_with("remaining_basis"))
            .map(|l| l.amount)
            .sum();
        assert_eq!(remaining, 510000);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn debt_reduces_different_current_nols_without_reducing_business_basis() {
    let mut asset = separate_amt_equipment();
    asset.cost *= 100;
    if let EquipmentHistory::Continuing { prior_depreciation, amt, .. } = &mut asset.history {
        *prior_depreciation *= 100;
        if let EquipmentAmtHistory::Separate { prior_depreciation, .. } = amt {
            *prior_depreciation *= 100;
        }
    }
    let mut d = business_with_bonus(0, asset);
    let before = supported(&d);
    add(&mut d, bankruptcy_debt(1500, 10000, 6000));
    let r = supported(&d);
    assert_eq!(value(&r, "current_net_operating_loss"), 5536);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 27395);
    assert_eq!(value(&r, "nol_carryforward"), 4036);
    assert_eq!(value(&r, "amt_nol_carryforward"), 25895);
    assert_eq!(value(&r, "debt_3_basis_reduction"), 0);
    assert_eq!(value(&r, "debt_3_amt_basis_reduction"), 0);
    assert_eq!(
        value(&r, "business_2_asset_0_remaining_basis"),
        value(&before, "business_2_asset_0_remaining_basis")
    );
    assert_eq!(r.tax, before.tax);
    let FederalIncome::PersonalDebt { data } = &mut d.income[2].income else { panic!() };
    data.form.amount = 600000;
    unsupported(&d, "Bankruptcy tax attributes");
}

#[test]
fn debt_reduces_expiring_atnol_before_using_amt_property_basis() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().atnol = prior_atnol(&[(2006, 100000, false)]);
    add(&mut d, bankruptcy_debt(1500, 10000, 6000));
    let r = supported(&d);
    assert_eq!(value(&r, "atnol_2006_used"), 54000);
    assert_eq!(value(&r, "atnol_2006_debt_reduction"), 1500);
    assert_eq!(value(&r, "atnol_2006_expired"), 44500);
    assert_eq!(value(&r, "atnol_2006_carryforward"), 0);
    assert_eq!(value(&r, "amt_total_nol_carryforward"), 0);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 1500);
    assert_eq!(value(&r, "debt_2_amt_basis_reduction"), 0);
}

#[test]
fn debt_amt_basis_inputs_validate_and_bound_the_separate_reduction() {
    let mut debt = bankruptcy_debt(1500, 10000, 6000);
    let FederalIncome::PersonalDebt { data } = &mut debt.income else { panic!() };
    let Some(crate::CanceledDebtChoice::Bankruptcy { amt_basis_and_cash_after, .. }) =
        &mut data.treatment
    else {
        panic!()
    };
    *amt_basis_and_cash_after = Some(610000);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, debt.clone());
    let r = supported(&d);
    assert_eq!(value(&r, "debt_2_basis_reduction"), 1500);
    assert_eq!(value(&r, "debt_2_amt_basis_reduction"), 100);
}

#[test]
fn separate_amt_sale_basis_nets_both_terms_with_prior_amt_losses() {
    let mut d = complete(FederalStatus::Single);
    d.screening.as_mut().unwrap().capital_history = vec![amt_only_capital_history()];
    add(
        &mut d,
        income(sale_with_amt(ordinary_investment_sale(8000, 10000, GainTerm::Short), 5000, 0)),
    );
    add(
        &mut d,
        income(sale_with_amt(ordinary_investment_sale(12000, 2000, GainTerm::Long), 17000, 0)),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 8000);
    // AMT short 3,000 less 10,000 carry; long loss 5,000.
    assert_eq!(value(&r, "amt_capital_gain"), -3000);
    assert_eq!(value(&r, "amt_capital_short_carryforward"), 4000);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 5000);
    assert_eq!(value(&r, "capital_loss_carryforward"), 0);
}

#[test]
fn separate_amt_sale_basis_recomputes_collectible_pool() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(sale_with_amt(collectible_sale(12000, 2000, GainTerm::Long, false), 7000, 0)),
    );
    add(
        &mut d,
        income(sale_with_amt(ordinary_investment_sale(5000, 10000, GainTerm::Short), 6000, 0)),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 5000);
    assert_eq!(value(&r, "rate_28_gain"), 5000);
    assert_eq!(value(&r, "amt_capital_gain"), 4000);
    assert_eq!(value(&r, "amt_rate_28_gain"), 4000);
}

#[test]
fn separate_amt_sale_basis_recomputes_investment_income_and_election_cap() {
    for term in [GainTerm::Short, GainTerm::Long] {
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(sale_with_amt(ordinary_investment_sale(12000, 0, term), 10000, 0)));
        investment_borrowing(&mut d, 5000, 0, 0);
        if term == GainTerm::Long {
            d.deductions.as_mut().unwrap().investment_interest.as_mut().unwrap().election =
                InvestmentElection::CapitalFirst { amount: 1000000 };
        }
        let r = supported(&d);
        assert_eq!(value(&r, "investment_interest_deduction"), 5000);
        assert_eq!(value(&r, "amt_investment_interest_deduction"), 2000);
        assert_eq!(value(&r, "amt_investment_interest_carryforward"), 3000);
        assert_eq!(value(&r, "amt_capital_gain"), 2000);
        if term == GainTerm::Long {
            assert_eq!(value(&r, "amt_investment_elected_capital_gain"), 2000);
        }
    }
}

#[test]
fn qualifying_iso_sale_derives_regular_and_amt_basis_without_repeating_exercise() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::IsoSale { data: qualifying_iso_sale() }));
    let r = supported(&d);
    assert_eq!(value(&r, "iso_sale_2_regular_basis"), 10000);
    assert_eq!(value(&r, "iso_sale_2_amt_basis"), 100000);
    assert_eq!(value(&r, "iso_sale_2_capital_gain"), 50000);
    assert_eq!(value(&r, "iso_sale_2_amt_capital_gain"), -40000);
    assert_eq!(value(&r, "agi"), 110000);
    assert_eq!(value(&r, "capital_gain"), 50000);
    assert_eq!(value(&r, "amt_capital_gain"), -3000);
    assert_eq!(value(&r, "amt_capital_adjustment"), -53000);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 37000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 57000);
    assert_eq!(return_line(&r, "1a"), 6000000);
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn scorp_interview_preserves_source_and_shows_basis_without_claiming_return_coverage() {
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(scorp_document()),
        }),
    );
    assert!(d.valid_stored());
    assert_eq!(restart(d.clone()).draft, d);
    assert_eq!(crate::interview::document_amount(&d.income.last().unwrap().income, false), None);
    unsupported(&d, "S-corporation AMT");
    let r = estimate(&d);
    for (suffix, amount) in [
        ("basis allowed loss", 13_000),
        ("basis suspended loss", 2_000),
        ("loan 1 basis", 0),
        ("stock basis", 0),
    ] {
        scorp_note(&r, id, suffix, amount);
    }
    assert!(r.review_notes.iter().any(|note| {
        note.display
            == Message::new(
                "NOTE_SCORP_STOCK_BASIS",
                vec![MessageArg::Number(id), MessageArg::usd_cents(0)],
            )
    }));
    assert!(r.review_notes.iter().any(|note| {
        note.display
            == Message::new(
                "NOTE_SCORP_LOAN_BASIS",
                vec![MessageArg::Number(id), MessageArg::usd_cents(0), MessageArg::Number(1)],
            )
    }));
}

#[test]
fn scorp_unknown_basis_sources_do_not_publish_partial_basis_as_complete() {
    let mut d = complete(FederalStatus::Single);
    let mut source = scorp_document();
    source.special = vec![crate::ScorpSpecial::Farming];
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = estimate(&d);
    assert!(noted(&r, "WARN_SCORP_K1_LEFT_OUT"));
    assert!(!r.review_notes.iter().any(|v| v.starts_with("S-corporation document ")));
}

#[test]
fn scorp_stored_validation_rejects_invalid_basis_and_open_account_records() {
    let mut source = scorp_document();
    source.loans[0].basis += 1;
    assert!(!source.valid());
    source.loans[0].basis -= 1;
    source.prior_nondeductible = 1;
    assert!(!source.valid());
    source.losses_first = true;
    assert!(source.valid());
    source.loans[0].kind = ScorpLoanKind::OpenAccount;
    source.loans.push(source.loans[0].clone());
    assert!(!source.valid());
    source.loans.pop();
    source.loans[0].principal = 2_500_001;
    assert!(!source.valid());
}

// T2-48f fixtures: Form 7203 Part I stock basis and Part II debt basis
// boundaries (row 48). Each K-1 has no `risk`/`participation`/`amt` record,
// so the return stays Unsupported for "S-corporation AMT" (and the
// unresolved passive-pool classification), exactly like
// `scorp_interview_preserves_source_and_shows_basis_without_claiming_return_coverage`
// above; the basis figures in `review_notes` are computed independently by
// `ScorpK1::report` regardless of that overall state. Totals below are
// worked by hand from the i7203 Part I/II worksheet order that
// `StockBasis::calculate`/`with_loans` implement: opening + additions +
// income, less distributions (excess becomes distribution gain), less
// nondeductible expenses (before or after losses per the 1.1367-1(g)
// election), less allowed loss; any leftover current-year income (net
// increase) restores debt basis before it can raise stock basis; the
// unabsorbed loss carries to debt basis up to its own basis; whatever is
// still unabsorbed is suspended.

#[test]
fn scorp_stock_basis_reduced_to_zero_suspends_the_excess_ordinary_loss() {
    // Opening stock basis $5,000; K-1 box 1 ordinary loss $8,000; no loans,
    // so debt basis cannot absorb any of the excess. Basis absorbs $5,000 of
    // the loss (ending stock basis $0); the remaining $3,000 is suspended.
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    let source =
        ScorpK1 { opening: Some(500_000), ordinary: -800_000, loans: vec![], ..scorp_document() };
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    unsupported(&d, "S-corporation AMT");
    let r = estimate(&d);
    for (suffix, amount) in
        [("stock basis", 0), ("basis allowed loss", 5_000), ("basis suspended loss", 3_000)]
    {
        scorp_note(&r, id, suffix, amount);
    }
}

#[test]
fn scorp_distribution_above_stock_basis_is_reported_as_capital_gain() {
    // Opening stock basis $3,000, no income; box 16 code D nondividend
    // distribution $6,000. The first $3,000 returns basis to zero; the
    // remaining $3,000 exceeds basis and is a shareholder capital gain
    // (Form 7203 Part I line 6, separate from ordinary loss allowance).
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    let source = ScorpK1 {
        opening: Some(300_000),
        ordinary: 0,
        distributions: 600_000,
        loans: vec![],
        ..scorp_document()
    };
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    unsupported(&d, "S-corporation AMT");
    let r = estimate(&d);
    for (suffix, amount) in [
        ("distribution gain", 3_000),
        ("stock basis", 0),
        ("basis allowed loss", 0),
        ("basis suspended loss", 0),
    ] {
        scorp_note(&r, id, suffix, amount);
    }
}

#[test]
fn scorp_net_increase_restores_debt_basis_before_increasing_stock_basis() {
    // Opening stock basis $2,000, K-1 box 1 ordinary income $3,000 (no
    // losses or distributions), so this year's net increase is $3,000. One
    // formal note with a $800 basis deficit (principal $1,000, basis $200)
    // from a prior reduction. Reg. 1.1367-2(c)(1): a net increase restores
    // debt basis before it raises stock basis, so $800 of the $3,000
    // restores the note to its full $1,000 basis; only the remaining $2,200
    // adds to the $2,000 opening stock basis, for an ending stock basis of
    // $4,200 ($2,000 + $3,000 - $800 diverted to debt restoration).
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    let source = ScorpK1 {
        opening: Some(200_000),
        ordinary: 300_000,
        loans: vec![ScorpLoan {
            repayment_term: ScorpNoteHolding::NoRepayment,
            kind: ScorpLoanKind::FormalNote,
            principal: 100_000,
            basis: 20_000,
            advances: 0,
            repayments: 0,
        }],
        ..scorp_document()
    };
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    unsupported(&d, "S-corporation AMT");
    let r = estimate(&d);
    for (suffix, amount) in
        [("stock basis", 4_200), ("loan 1 basis", 1_000), ("loan 1 restoration", 800)]
    {
        scorp_note(&r, id, suffix, amount);
    }
}

#[test]
fn scorp_loss_exactly_equal_to_stock_basis_is_fully_allowed() {
    // Opening stock basis $4,000, K-1 box 1 ordinary loss $4,000, no loans.
    // Basis and loss are exactly equal, so the entire loss is allowed with
    // nothing suspended and ending stock basis exactly $0, not negative.
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    let source =
        ScorpK1 { opening: Some(400_000), ordinary: -400_000, loans: vec![], ..scorp_document() };
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    unsupported(&d, "S-corporation AMT");
    let r = estimate(&d);
    for (suffix, amount) in
        [("basis allowed loss", 4_000), ("basis suspended loss", 0), ("stock basis", 0)]
    {
        scorp_note(&r, id, suffix, amount);
    }
}

#[test]
fn scorp_nondeductible_expenses_ordering_differs_under_the_losses_first_election() {
    // Same facts under both orderings: opening stock basis $5,000, K-1 box 1
    // ordinary loss $4,000, box 16 code C nondeductible expenses $3,000, no
    // loans, no continuing prior-year nondeductible carryforward.
    //
    // Default order (Reg. 1.1367-1(f), no 1.1367-1(g) election): nondeductible
    // expenses reduce basis before losses. $3,000 of basis absorbs the
    // expenses first, leaving $2,000 of basis for the $4,000 loss: $2,000
    // allowed, $2,000 suspended, and no nondeductible carryforward exists
    // under the default (it is not eligible to carry).
    //
    // Losses-first election (Reg. 1.1367-1(g)): losses reduce basis before
    // nondeductible expenses. The $4,000 loss is fully absorbed by the
    // $5,000 basis ($4,000 allowed, $0 suspended), leaving $1,000 of basis
    // for the $3,000 expenses; $1,000 of expenses is absorbed and the
    // remaining $2,000 carries forward to next year under the election.
    let base = |losses_first: bool| ScorpK1 {
        opening: Some(500_000),
        ordinary: -400_000,
        nondeductible: 300_000,
        losses_first,
        loans: vec![],
        ..scorp_document()
    };

    let mut default_d = complete(FederalStatus::Single);
    let default_id = default_d.next_id;
    add(
        &mut default_d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(base(false)),
        }),
    );
    unsupported(&default_d, "S-corporation AMT");
    let default_r = estimate(&default_d);
    for (suffix, amount) in
        [("basis allowed loss", 2_000), ("basis suspended loss", 2_000), ("nondeductible carry", 0)]
    {
        scorp_note(&default_r, default_id, suffix, amount);
    }

    let mut election_d = complete(FederalStatus::Single);
    let election_id = election_d.next_id;
    add(
        &mut election_d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(base(true)),
        }),
    );
    unsupported(&election_d, "S-corporation AMT");
    let election_r = estimate(&election_d);
    for (suffix, amount) in
        [("basis allowed loss", 4_000), ("basis suspended loss", 0), ("nondeductible carry", 2_000)]
    {
        scorp_note(&election_r, election_id, suffix, amount);
    }
}

#[test]
fn scorp_loss_exceeding_stock_basis_with_a_loan_and_a_distribution_above_basis() {
    // A shareholder whose K-1 loss exceeds stock basis with a shareholder
    // loan, then a distribution above basis, all in one Form 7203: opening
    // stock basis $4,000, no additions/other income; box 16 code D
    // distribution $4,500; K-1 box 1 ordinary loss $6,000; one formal note
    // with full basis (principal and basis both $2,000, no deficit).
    //
    // Part I: before distributions the basis is $4,000 (opening + additions
    // + income). The $4,500 distribution first returns all $4,000 of basis
    // to zero, and the excess $500 is a shareholder capital gain. With
    // stock basis already at $0, none of the $6,000 loss is allowed against
    // stock; the full $6,000 carries to Part II.
    // Part II: the note has no basis deficit, so this year's (zero) net
    // increase restores nothing. The note's $2,000 basis absorbs $2,000 of
    // the carried loss, ending the note at $0 debt basis with no
    // restoration; the remaining $4,000 of loss is suspended.
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    let source = ScorpK1 {
        opening: Some(400_000),
        ordinary: -600_000,
        distributions: 450_000,
        loans: vec![ScorpLoan {
            repayment_term: ScorpNoteHolding::NoRepayment,
            kind: ScorpLoanKind::FormalNote,
            principal: 200_000,
            basis: 200_000,
            advances: 0,
            repayments: 0,
        }],
        ..scorp_document()
    };
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    unsupported(&d, "S-corporation AMT");
    let r = estimate(&d);
    for (suffix, amount) in [
        ("stock basis", 0),
        ("distribution gain", 500),
        ("basis allowed loss", 2_000),
        ("basis suspended loss", 4_000),
        ("loan 1 basis", 0),
        ("loan 1 restoration", 0),
    ] {
        scorp_note(&r, id, suffix, amount);
    }
}

#[test]
fn scorp_at_risk_interview_applies_history_after_basis_and_survives_avro() {
    let mut source = scorp_document();
    source.risk = scorp_risk_history();
    let (d, r, id) = scorp_risk_result(source);
    assert_eq!(restart(d.clone()).draft, d);
    scorp_note(&r, id, "basis allowed loss", 13_000);
    scorp_note(&r, id, "basis suspended loss", 2_000);
    scorp_note(&r, id, "investment before current income", 5_000);
    scorp_note(&r, id, "loss allowed through at-risk", 5_000);
    scorp_note(&r, id, "loss still suspended by at-risk", 10_000);
}

#[test]
fn scorp_at_risk_financing_excludes_protected_funds_and_their_returns() {
    let mut source = scorp_document();
    source.opening = Some(0);
    source.loans.clear();
    source.additions = 1_000_000;
    source.ordinary = -800_000;
    let mut risk = scorp_risk_history();
    risk.start = ScorpRiskStart::New;
    risk.prior_loss = 0;
    risk.deducted = 0;
    risk.excluded_funding = 600_000;
    source.risk = risk;
    let (_, r, id) = scorp_risk_result(source.clone());
    scorp_note(&r, id, "loss allowed through at-risk", 4_000);
    scorp_note(&r, id, "loss still suspended by at-risk", 4_000);
    source.distributions = 200_000;
    source.risk.excluded_returns = 200_000;
    let (_, r, id) = scorp_risk_result(source);
    scorp_note(&r, id, "investment before current income", 4_000);
}

#[test]
fn scorp_at_risk_recapture_uses_current_profit_but_cannot_release_current_losses() {
    let mut source = scorp_document();
    source.ordinary = 100_000;
    source.distributions = 300_000;
    let mut risk = scorp_risk_history();
    risk.start = ScorpRiskStart::Detailed { capacity: 100_000, profit: 0, loss: 0 };
    risk.prior_loss = 0;
    source.risk = risk;
    let (_, r, id) = scorp_risk_result(source.clone());
    scorp_note(&r, id, "recapture income", 1_000);
    scorp_note(&r, id, "recapture deduction for next year", 1_000);
    source.ordinary = -100_000;
    let (_, r, id) = scorp_risk_result(source);
    scorp_note(&r, id, "recapture income", 2_000);
    scorp_note(&r, id, "loss allowed through at-risk", 0);
    scorp_note(&r, id, "loss still suspended by at-risk", 1_000);
}

#[test]
fn scorp_at_risk_missing_or_inconsistent_financing_is_not_assumed_safe() {
    let mut source = scorp_document();
    let mut risk = scorp_risk_history();
    risk.start = ScorpRiskStart::Unknown;
    source.risk = risk;
    let (_, r, _) = scorp_risk_result(source.clone());
    assert!(r.issues.iter().any(|v| v.section == "S-corporation at-risk"));
    assert!(!r.review_notes.iter().any(|v| v.contains("loss allowed through at-risk:")));
    source.risk.excluded_funding = 1;
    assert!(!source.valid());
    source.risk.excluded_funding = 0;
    source.risk.excluded_returns = 1;
    assert!(!source.valid());
    source.risk.excluded_returns = 0;
    source.risk.start = ScorpRiskStart::New;
    assert!(!source.valid());
}

#[test]
fn prepared_scorp_results_feed_passive_pool_without_repeating_basis_or_at_risk() {
    use crate::tests::support::loss_limits::{PassiveActivity, passive_suspensions};
    let mut source = scorp_document();
    let mut risk = scorp_risk_history();
    risk.start = ScorpRiskStart::Detailed { capacity: 800_000, profit: 0, loss: 0 };
    risk.prior_loss = 0;
    source.risk = risk;
    let basis = source.basis_result().unwrap();
    let computed = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(basis.allowed[K1LossClass::Ordinary].get()),
            crate::kernel::ratio::dollars(
                computed.ledger.risk_allowed[K1LossClass::Ordinary].get()
            ),
            crate::kernel::ratio::dollars(
                computed.ledger.risk_suspended[K1LossClass::Ordinary].get()
            )
        ),
        (13_000, 8_000, 5_000)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(computed.income.get()),
            computed.year_end.map(crate::kernel::ratio::dollars),
            crate::kernel::ratio::dollars(computed.recapture.income.get())
        ),
        (0, Some(0), 0)
    );
    // Classification is supplied here, not inferred from K-1 ownership:
    // a passive K-1 activity shares $6,000 passive rental profit.
    let balances = passive_suspensions(
        &[
            PassiveActivity {
                key: (),
                net: crate::kernel::ratio::dollars(
                    computed.income.get()
                        - computed.ledger.risk_allowed[K1LossClass::Ordinary].get(),
                ),
                rental_allowance: false,
                ineligible_prior: 0,
            },
            PassiveActivity { key: (), net: 6_000, rental_allowance: true, ineligible_prior: 0 },
        ],
        0,
        25_000,
    )
    .unwrap();
    assert_eq!(balances, vec![2_000, 0]);
    let (_, r, id) = scorp_risk_result(source);
    scorp_note(
        &r,
        id,
        "loss allowed through at-risk",
        crate::kernel::ratio::dollars(computed.ledger.risk_allowed[K1LossClass::Ordinary].get()),
    );
    scorp_note(
        &r,
        id,
        "year-end at-risk investment before recapture",
        crate::kernel::ratio::dollars(computed.year_end.unwrap()),
    );
}

#[test]
fn prepared_scorp_results_reject_unknown_and_mismatched_sources() {
    let mut source = scorp_document();
    source.special = vec![crate::ScorpSpecial::Farming];
    assert!(source.basis_result().is_err());
    source.special = Vec::new();
    source.risk = scorp_risk_history();
    let mut basis = source.basis_result().unwrap();
    basis.allowed[K1LossClass::Ordinary] =
        crate::kernel::carry::Amount::new(basis.allowed[K1LossClass::Ordinary].get() + 100)
            .unwrap();
    assert!(source.risk.calculate(&source, &basis).is_err());
    basis.allowed = LossAmounts::default();
    assert!(source.risk.calculate(&source, &basis).is_err());
}

#[test]
fn scorp_participation_hour_thresholds_are_strict_and_spouse_time_is_included() {
    let mut p = participation_facts();
    p.minutes = 30_000;
    assert_eq!(p.material(), crate::participation::Participation::No);
    p.minutes += 1;
    assert_eq!(p.material(), crate::participation::Participation::Yes);
    p.minutes = 6_000;
    p.at_least_as_much = true;
    assert_eq!(p.material(), crate::participation::Participation::No);
    p.minutes += 1;
    assert_eq!(p.material(), crate::participation::Participation::Yes);
    p.at_least_as_much = false;
    p.minutes = 300 * 60 + 201 * 60; // shareholder plus spouse, regardless of filing status
    assert_eq!(p.material(), crate::participation::Participation::Yes);
}

#[test]
fn scorp_participation_substantially_all_and_historical_tests_do_not_require_500_hours() {
    let mut p = participation_facts();
    p.minutes = 60;
    p.substantially_all = true;
    assert_eq!(p.material(), crate::participation::Participation::Yes);
    p.substantially_all = false;
    p.minutes = 0;
    p.prior_ten = 4;
    assert_eq!(p.material(), crate::participation::Participation::No);
    p.prior_ten = 5;
    assert_eq!(p.material(), crate::participation::Participation::Yes);
    p.prior_ten = 0;
    p.personal_service = true;
    p.service_years = 3;
    assert_eq!(p.material(), crate::participation::Participation::Yes);
    p.service_years = 2;
    assert_eq!(p.material(), crate::participation::Participation::No);
}

#[test]
fn scorp_participation_management_exclusions_and_aggregation_remain_distinct() {
    let mut p = participation_facts();
    p.minutes = 12_000;
    p.management_minutes = 9_000;
    p.regular = true;
    assert_eq!(p.material(), crate::participation::Participation::Yes);
    p.paid_manager = true;
    assert_eq!(p.material(), crate::participation::Participation::No);
    p.paid_manager = false;
    p.other_manager_more = true;
    assert_eq!(p.material(), crate::participation::Participation::No);
    p.regular = false;
    p.other_manager_more = false;
    p.other_business_over_100 = true;
    assert_eq!(p.material(), crate::participation::Participation::No);
    assert!(p.left_out());
    p.minutes = 6_000;
    p.management_minutes = 0;
    assert_eq!(p.material(), crate::participation::Participation::No); // not itself a significant-participation activity
    p.ordinary_scope = false;
    assert_eq!(p.material(), crate::participation::Participation::No);
    assert!(p.left_out());
}

#[test]
fn scorp_participation_source_persists_and_does_not_enable_unfinished_return() {
    let mut source = scorp_document();
    let mut p = participation_facts();
    p.prior_ten = 5;
    source.participation = Some(p);
    let (d, r, id) = scorp_risk_result(source.clone());
    assert_eq!(restart(d.clone()).draft, d);
    assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!(
        "S-corporation document {id}, material participation: established."
    ))));
    assert!(r.review_notes.iter().any(|note| {
        note.display
            == Message::new("NOTE_SCORP_MATERIAL_ESTABLISHED", vec![MessageArg::Number(id)])
    }));
    source.participation.as_mut().unwrap().management_minutes = 1;
    assert!(!source.valid());
}

#[test]
fn nonpassive_scorp_profit_changes_rental_magi_without_funding_passive_losses() {
    // Base MAGI $60,000 plus $60,000 current K-1 profit minus $10,000
    // of that activity's old passive loss = $110,000. Rental allowance $20,000.
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(nonpassive_scorp(60_000, 10_000)),
        }),
    );
    let rental_id = d.next_id;
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 20_000) }));
    let r = estimate(&d);
    passive_note(&r, id, 0, 0);
    passive_note(&r, rental_id, -20_000, 10_000);
    assert!(r.review_notes.iter().any(|n| n.contains("nonpassive net after own prior passive loss: $50000; prior passive loss absorbed by own income: $10000.")));
}

#[test]
fn nonpassive_scorp_current_loss_does_not_consume_other_passive_income() {
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(nonpassive_scorp(-15_000, 2_000)),
        }),
    );
    let rental_id = d.next_id;
    add(&mut d, income(FederalIncome::Rental { data: rental(13_000, 0) }));
    let r = estimate(&d);
    passive_note(&r, id, -2_000, 0);
    passive_note(&r, rental_id, 3_000, 0);
    assert!(r.review_notes.iter().any(|n| n.contains("nonpassive net after own prior passive loss: $-8000; prior passive loss absorbed by own income: $0.")));
    scorp_note(&r, id, "basis suspended loss", 2_000);
    scorp_note(&r, id, "loss still suspended by at-risk", 5_000);
    let mut worksheet = r.clone();
    let schedule_e = crate::extended_rules::rentals(&d, 60_000, &mut worksheet);
    assert_eq!(schedule_e.income, -7_000); // current -8,000 + rent 3,000 - old loss 2,000
    assert_eq!(value(&worksheet, "rental_magi"), 52_000);
}

#[test]
fn scorp_qbi_source_connects_fifo_to_actual_basis_and_survives_restart() {
    let source = scorp_loss_qbi_source();
    assert!(source.valid());
    let (d, r, id) = scorp_risk_result(source.clone());
    assert_eq!(restart(d.clone()).draft, d);
    scorp_note(&r, id, "QBI loss passing basis limits", 5_000);
    assert!(
        r.review_notes
            .iter()
            .any(|v| v.contains("2026 basis-loss carryforward: $6000.00; QBI portion: $3000.00."))
    );
    assert!(r.tax.is_none() && r.refund.is_none());
    let release = source.loss_qbi.calculate(&source, &source.basis_result().unwrap()).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(release.amount.get()), 13_000);
    assert_eq!(
        crate::kernel::ratio::dollars(
            release.remaining.iter().map(|v| v.remaining.get()).sum::<i64>()
        ),
        6_000
    );
}

#[test]
fn scorp_qbi_source_rejects_inconsistent_years_and_loss_character() {
    let baseline = scorp_loss_qbi_source();
    for case in [0, 2, 3, 4, 5] {
        let mut source = baseline.clone();
        let qbi = &mut source.loss_qbi;
        match case {
            0 => qbi.history[0].year = 2017,
            2 => qbi.history[1].qualified = 1,
            3 => qbi.history[0].remaining -= 1,
            4 => qbi.current = 1_500_001,
            5 => qbi.history[0].remaining_qualified = 200_001,
            _ => unreachable!(),
        }
        assert!(!source.valid(), "case {case}");
    }
}

#[test]
fn scorp_qbi_zero_history_and_positive_income_do_not_invent_losses() {
    let mut source = scorp_document();
    source.ordinary = 1_000_000;
    source.qbi_statement = complete_ordinary_scorp(10_000, true).qbi_statement;
    // Zero and empty tracking (nothing to track) is valid and reports no
    // QBI loss; a QBI loss portion without a loss is not.
    assert!(source.loss_qbi.is_empty() && source.valid());
    source.loss_qbi.current = 1;
    assert!(!source.valid());
    source.loss_qbi = Default::default();
    let (_, r, _) = scorp_risk_result(source);
    assert!(!r.review_notes.iter().any(|v| v.contains("QBI loss passing basis limits")));
}

#[test]
fn scorp_qbi_history_does_not_silently_truncate_incomplete_basis_results() {
    let source = scorp_loss_qbi_source();
    let mut basis = source.basis_result().unwrap();
    basis.suspended = LossAmounts::default();
    assert!(source.loss_qbi.calculate(&source, &basis).is_err());
}

#[test]
fn scorp_qbi_at_risk_consumes_its_own_prior_history_before_new_basis_release() {
    let mut source = scorp_loss_qbi_source();
    source.risk = scorp_risk_history(); // $5,000 opening, $2,000 prior at-risk loss
    source.loss_qbi.at_risk = vec![loss_year(2020, 4_000, 2_000, 2_000, 1_000)];
    let basis = source.basis_result().unwrap();
    let release = source.loss_qbi.after_at_risk(&source, &basis).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(release.amount.get()), 5_000);
    assert_eq!(crate::kernel::ratio::dollars(release.qbi.get()), 2_154); // prior $1,000 + $3,000 * (5,000 / 13,000)
    assert_eq!(
        (
            crate::kernel::ratio::dollars(release.remaining[1].original.get()),
            crate::kernel::ratio::dollars(release.remaining[1].original_qbi.get())
        ),
        (10_000, 3_846)
    );
    assert_eq!(crate::kernel::ratio::dollars(release.remaining[0].remaining.get()), 0);
    assert_eq!(
        (
            release.remaining[1].year,
            crate::kernel::ratio::dollars(release.remaining[1].remaining.get()),
            crate::kernel::ratio::dollars(release.remaining[1].remaining_qbi.get())
        ),
        (2026, 10_000, 3_846)
    );
    let (d, r, id) = scorp_risk_result(source);
    assert_eq!(restart(d.clone()).draft, d);
    scorp_note(&r, id, "QBI loss passing at-risk limits", 2_154);
    assert!(
        r.review_notes.iter().any(
            |n| n.contains("2026 at-risk-loss carryforward: $10000.00; QBI portion: $3846.15.")
        )
    );
}

#[test]
fn scorp_qbi_at_risk_full_suspension_preserves_basis_release_character() {
    let mut source = scorp_loss_qbi_source();
    let mut risk = scorp_risk_history();
    risk.start = ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
    risk.prior_loss = 0;
    source.risk = risk;
    source.loss_qbi.at_risk = vec![];
    let basis = source.basis_result().unwrap();
    let r = source.loss_qbi.after_at_risk(&source, &basis).unwrap();
    assert_eq!(
        (crate::kernel::ratio::dollars(r.amount.get()), crate::kernel::ratio::dollars(r.qbi.get())),
        (0, 0)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(r.remaining[0].remaining.get()),
            crate::kernel::ratio::dollars(r.remaining[0].remaining_qbi.get())
        ),
        (13_000, 5_000)
    );
    assert_eq!(crate::kernel::ratio::dollars(basis.suspended[K1LossClass::Ordinary].get()), 6_000); // still a distinct basis carryforward
}

#[test]
fn scorp_qbi_at_risk_income_releases_old_loss_without_new_basis_loss() {
    let mut source = scorp_document();
    source.ordinary = 100_000;
    let mut risk = scorp_risk_history();
    risk.start = ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
    source.risk = risk;
    source.loss_qbi = ScorpLossQbi {
        at_risk: vec![loss_year(2021, 2_000, 1_000, 2_000, 1_000)],
        ..Default::default()
    };
    let r = source.loss_qbi.after_at_risk(&source, &source.basis_result().unwrap()).unwrap();
    assert_eq!(
        (crate::kernel::ratio::dollars(r.amount.get()), crate::kernel::ratio::dollars(r.qbi.get())),
        (1_000, 500)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(r.remaining[0].remaining.get()),
            crate::kernel::ratio::dollars(r.remaining[0].remaining_qbi.get())
        ),
        (1_000, 500)
    );
}

#[test]
fn scorp_qbi_at_risk_keeps_current_qualified_ratio_through_two_limits() {
    let mut source = scorp_document();
    source.ordinary = -10_000_000;
    source.opening = Some(6_000_000);
    source.loans.clear();
    let mut risk = scorp_risk_history();
    risk.start = ScorpRiskStart::Detailed { capacity: 3_000_000, profit: 0, loss: 0 };
    risk.prior_loss = 0;
    source.risk = risk;
    source.loss_qbi =
        ScorpLossQbi { passive: vec![], current: 8_000_000, history: vec![], at_risk: vec![] };
    let basis = source.basis_result().unwrap();
    let qbi = &source.loss_qbi;
    assert_eq!(
        crate::kernel::ratio::dollars(qbi.calculate(&source, &basis).unwrap().qbi.get()),
        48_000
    );
    let risk = qbi.after_at_risk(&source, &basis).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.amount.get()),
            crate::kernel::ratio::dollars(risk.qbi.get())
        ),
        (30_000, 24_000)
    );
    assert_eq!(crate::kernel::ratio::dollars(risk.remaining[0].remaining_qbi.get()), 24_000);
}

#[test]
fn scorp_qbi_at_risk_history_must_reconcile_and_has_no_pre_2018_qbi() {
    let mut source = scorp_loss_qbi_source();
    source.risk = scorp_risk_history();
    source.loss_qbi.at_risk = vec![];
    assert!(!source.valid()); // cannot confirm zero when the at-risk source has $2,000
    source.loss_qbi.at_risk = vec![loss_year(2017, 2_000, 1_000, 2_000, 1_000)];
    assert!(!source.valid()); // pre-2018 QBI forbidden at every limiting provision
}

#[test]
fn scorp_statement_keeps_historical_sstb_loss_outside_current_phaseout() {
    let source = scorp_current_qbi_source();
    let activities = source
        .qbi_statement
        .as_ref()
        .unwrap()
        .activities(&source, crate::kernel::carry::Amount::new(0).unwrap(), 0)
        .unwrap();
    assert_eq!(activities.len(), 2);
    assert_eq!(activities[0].income, 40_000);
    assert_eq!(activities[1].income, -8_000);
    assert_eq!(activities[1].service, Some(false));
    let mut result = estimate(&complete(FederalStatus::Single));
    // Halfway through the 2026 SSTB range: current $40k becomes $20k,
    // historical $8k remains $8k. Wage limit permits 20% of the $12k net.
    let (deduction, carry) =
        crate::business_rules::qbi(&activities, 239_250, 0, FederalStatus::Single, &mut result);
    assert_eq!((deduction, carry), (2_400, 0));
    let (deduction, carry) =
        crate::business_rules::qbi(&activities, 300_000, 0, FederalStatus::Single, &mut result);
    assert_eq!((deduction, carry), (0, 8_000));
}

#[test]
fn live_scorp_statement_reaches_existing_qbi_engine_and_persists() {
    let source = scorp_current_qbi_source();
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    assert_eq!(restart(d.clone()).draft, d);
    let mut r = estimate(&d);
    scorp_note(&r, id, "QBI income less released historical QBI losses", 32_000);
    let schedule = crate::extended_rules::rentals(&d, 60_000, &mut r);
    assert_eq!(
        schedule
            .scorp_suspensions
            .iter()
            .map(|(id, cents)| (*id, crate::kernel::ratio::dollars(cents.get())))
            .collect::<Vec<_>>(),
        vec![(id, 0)]
    );
    let source = scorp_current_qbi_source();
    let activities = source
        .qbi_statement
        .as_ref()
        .unwrap()
        .activities(&source, crate::kernel::carry::Amount::new(0).unwrap(), 0)
        .unwrap();
    let computed =
        crate::business_rules::qbi(&activities, 100_000, 0, FederalStatus::Single, &mut r);
    assert_eq!(computed, (6_400, 0));
    assert!(r.tax.is_none());
}

#[test]
fn scorp_statement_uses_allocated_wages_and_ubia_for_high_income_limit() {
    let mut source = nonpassive_scorp(100_000, 0);
    source.qbi_statement = Some(ScorpQbiStatement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 10_000_000,
        wages: 0,
        property: 40_000_000,
        service: false,
        complete: true,
    });
    let activities = source
        .qbi_statement
        .as_ref()
        .unwrap()
        .activities(&source, crate::kernel::carry::Amount::new(0).unwrap(), 0)
        .unwrap();
    assert_eq!(activities.len(), 1); // no redundant zero-loss history required
    let mut r = estimate(&complete(FederalStatus::Single));
    assert_eq!(
        crate::business_rules::qbi(&activities, 300_000, 0, FederalStatus::Single, &mut r),
        (10_000, 0)
    );
    source.qbi_statement.as_mut().unwrap().wages = 3_000_000;
    let activities = source
        .qbi_statement
        .as_ref()
        .unwrap()
        .activities(&source, crate::kernel::carry::Amount::new(0).unwrap(), 0)
        .unwrap();
    assert_eq!(
        crate::business_rules::qbi(&activities, 300_000, 0, FederalStatus::Single, &mut r),
        (17_500, 0)
    );
}

#[test]
fn scorp_statement_requires_history_and_accepts_current_loss_attribution() {
    let mut source = scorp_current_qbi_source();
    source.loss_qbi = Default::default();
    assert!(
        source
            .qbi_statement
            .as_ref()
            .unwrap()
            .activities(&source, crate::kernel::carry::Amount::new(0).unwrap(), 0)
            .is_err()
    );
    source.qbi_statement.as_mut().unwrap().income = 4_000_001;
    assert!(!source.valid());
    let mut source = passive_qbi_source();
    source.qbi_statement = Some(ScorpQbiStatement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 0,
        wages: 0,
        property: 0,
        service: false,
        complete: true,
    });
    let activities = source
        .qbi_statement
        .as_ref()
        .unwrap()
        .activities(&source, crate::kernel::carry::Amount::new(0).unwrap(), 0)
        .unwrap();
    assert_eq!(activities[0].income, -4_000);
}

#[test]
fn scorp_current_loss_sstb_eligibility_is_fixed_before_carryforward_creation() {
    let source = scorp_current_loss_statement();
    let statement = source.qbi_statement.as_ref().unwrap();
    for (taxable, qualified, basis_carry, risk_carry) in
        [(201_750, 4_000, 1_000, 2_500), (239_250, 2_000, 500, 1_250), (276_750, 0, 0, 0)]
    {
        let prepared = statement.prepare(&source, taxable, FederalStatus::Single).unwrap();
        let qbi = &prepared.loss_qbi;
        let basis = prepared.basis_result().unwrap();
        let current_balance = |r: crate::scorp_basis::LossRelease| {
            crate::kernel::ratio::dollars(
                r.remaining.iter().find(|v| v.year == 2026).unwrap().remaining_qbi.get(),
            )
        };
        assert_eq!(current_balance(qbi.calculate(&prepared, &basis).unwrap()), basis_carry);
        assert_eq!(current_balance(qbi.after_at_risk(&prepared, &basis).unwrap()), risk_carry);
        let activities = statement
            .activities(&prepared, crate::kernel::carry::Amount::new(2_000 * 100).unwrap(), 0)
            .unwrap();
        assert_eq!(activities[0].income, -qualified);
        assert_eq!(activities[0].service, Some(false)); // eligibility already applied once
        let mut result = estimate(&complete(FederalStatus::Single));
        assert_eq!(
            crate::business_rules::qbi(&activities, taxable, 0, FederalStatus::Single, &mut result)
                .1,
            qualified
        );
    }
    assert_eq!(source.loss_qbi.current, 750_000); // source statement stays raw
}

#[test]
fn scorp_current_loss_remains_distinct_after_mixed_basis_and_at_risk_releases() {
    let mut source = scorp_loss_qbi_source();
    source.risk = scorp_risk_history();
    source.participation = Some(participation_facts());
    source.participation.as_mut().unwrap().minutes = 30_001;
    source.passive = ScorpPassiveHistory { loss: 0, complete: true };
    let qbi = &mut source.loss_qbi;
    qbi.at_risk = vec![loss_year(2020, 4_000, 2_000, 2_000, 1_000)];
    qbi.passive = vec![];
    source.qbi_statement = Some(ScorpQbiStatement {
        ptp_income: 0,
        reit_dividends: 0,
        income: 0,
        wages: 0,
        property: 0,
        service: false,
        complete: true,
    });
    let activities = source
        .qbi_statement
        .as_ref()
        .unwrap()
        .activities(&source, crate::kernel::carry::Amount::new(0).unwrap(), 0)
        .unwrap();
    // 2026-10-19 MoneyPrecision: current and historical QBI report separately.
    assert_eq!(
        (activities[0].source_income_cents, activities[1].source_income_cents),
        (Some(-103_847), Some(-111_538))
    );
    assert_eq!((activities[0].income, activities[1].income), (-1_038, -1_115));
    assert_eq!(activities.iter().map(|v| v.income).sum::<i64>(), -2_153);
}

#[test]
fn scorp_current_nonservice_loss_is_not_phased_out() {
    let mut source = scorp_current_loss_statement();
    source.qbi_statement.as_mut().unwrap().service = false;
    let statement = source.qbi_statement.as_ref().unwrap();
    let original = statement.prepare(&source, 201_750, FederalStatus::Single).unwrap().loss_qbi;
    for (taxable, status) in [(201_751, FederalStatus::Single), (300_000, FederalStatus::Joint)] {
        assert_eq!(statement.prepare(&source, taxable, status).unwrap().loss_qbi, original);
    }
}

#[test]
fn live_scorp_niit_distinguishes_passive_ordinary_and_nonpassive_trading_profit() {
    for (material, kind, expected) in [
        (false, ScorpTradeKind::Ordinary, 10_000),
        (true, ScorpTradeKind::Ordinary, 0),
        (false, ScorpTradeKind::FinancialTrading, 10_000),
        (true, ScorpTradeKind::FinancialTrading, 10_000),
    ] {
        let mut source = passive_scorp_document();
        source.ordinary = 1_000_000;
        source.passive.loss = 0;
        source.participation.as_mut().unwrap().minutes = if material { 30_001 } else { 0 };
        source.trade_kind = kind;
        let (d, id) = investment_tax_draft(source, 0);
        let r = estimate(&d);
        investment_tax_note(&r, expected, if expected == 0 { 0 } else { 380 });
        assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!(
            "S-corporation document {id}, investment income after activity limits: ${expected};"
        ))));
        assert_eq!(restart(d.clone()).draft, d);
        assert!(r.tax.is_none());
    }
}

#[test]
fn live_scorp_niit_excludes_ordinary_nonpassive_loss_but_includes_trading_loss() {
    for (kind, expected, tax) in
        [(ScorpTradeKind::Ordinary, 10_000, 380), (ScorpTradeKind::FinancialTrading, 2_000, 76)]
    {
        let mut source = nonpassive_scorp(-15_000, 0);
        source.trade_kind = kind;
        let (d, _) = investment_tax_draft(source, 10_000);
        investment_tax_note(&estimate(&d), expected, tax);
    }
}

#[test]
fn k1_excess_business_loss_uses_2026_filing_limits_independent_of_wages_or_qbi() {
    for (status, loss, expected) in [
        (FederalStatus::Single, 256_000, 0),
        (FederalStatus::Single, 400_000, 144_000),
        (FederalStatus::Joint, 512_000, 0),
        (FederalStatus::Joint, 600_000, 88_000),
    ] {
        for wages in [10_000_000, 100_000_000] {
            let mut d = complete(status);
            d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
            add(
                &mut d,
                income(FederalIncome::Scorp {
                    data: crate::tests::support::builders::scorp_source(scorp_large_loss(
                        loss, loss, true,
                    )),
                }),
            );
            let r = estimate(&d);
            k1_excess_note(&r, -loss, expected);
            if expected > 0 {
                assert!(r.issues.iter().any(|i| i.section == "K-1 excess-loss allocation"));
            }
            assert!(r.tax.is_none());
        }
    }
}

#[test]
fn k1_excess_business_loss_only_uses_losses_passing_prior_limits() {
    for (capacity, material, net) in [(100_000, true, -100_000), (400_000, false, 0)] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(scorp_large_loss(
                    400_000, capacity, material,
                )),
            }),
        );
        k1_excess_note(&estimate(&d), net, 0);
    }
}

#[test]
fn k1_ordinary_profit_absorbs_other_business_loss_without_qbi_statement() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(scorp_large_loss(
                400_000, 400_000, true,
            )),
        }),
    );
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(nonpassive_scorp(100_000, 0)),
        }),
    );
    k1_excess_note(&estimate(&d), -300_000, 44_000);
}

#[test]
fn sole_k1_excess_loss_defers_only_its_qualified_fraction() {
    for (capacity, qualified, excess, deferred, remaining) in [
        (400_000, 200_000, 144_000, 72_000, 128_000),
        (400_000, 400_000, 144_000, 144_000, 256_000),
        (400_000, 0, 144_000, 0, 0),
        (300_000, 200_000, 44_000, 22_000, 128_000),
    ] {
        let source = sole_qbi_loss(400_000, capacity, qualified);
        let mut activities = source
            .qbi_statement
            .as_ref()
            .unwrap()
            .activities(&source, crate::kernel::carry::Amount::new(0).unwrap(), 0)
            .unwrap();
        assert_eq!(
            crate::scorp_qbi::sole_business_excess(&source, &mut activities, excess).unwrap(),
            deferred
        );
        assert_eq!(activities[0].income, -remaining);
    }
}

#[test]
fn live_sole_k1_excess_loss_reports_qualified_nol_separately() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 100_000_000, 0);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(sole_qbi_loss(
                400_000, 400_000, 200_000,
            )),
        }),
    );
    let r = estimate(&d);
    k1_excess_note(&r, -400_000, 144_000);
    assert!(!r.issues.iter().any(|i| i.section == "K-1 excess-loss allocation"));
    assert!(r.review_notes.iter().any(|n| n.starts_with(
        "K-1 excess-loss character: NOL carryforward $144000; qualified portion $72000."
    )));
    assert!(r.tax.is_none()); // independent AMT and complete K-1 integration still pending
}

#[test]
fn sole_k1_excess_allocation_rejects_mixed_class_or_invalid_amount_without_mutation() {
    let source = sole_qbi_loss(400_000, 400_000, 200_000);
    for case in 0..3 {
        let mut owner = source.clone();
        let mut activities = owner
            .qbi_statement
            .as_ref()
            .unwrap()
            .activities(&owner, crate::kernel::carry::Amount::new(0).unwrap(), 0)
            .unwrap();
        let excess = match case {
            0 => {
                owner.trade_kind = ScorpTradeKind::FinancialTrading;
                144_000
            }
            1 => -1,
            _ => 400_001,
        };
        assert!(crate::scorp_qbi::sole_business_excess(&owner, &mut activities, excess).is_err());
        assert_eq!(activities[0].income, -200_000);
    }
}

#[test]
fn scorp_equal_amt_source_records_produce_zero_adjustment_and_persist() {
    let mut source = nonpassive_scorp(10_000, 0);
    source.amt = Some(amt_records(&source));
    let (d, _) = investment_tax_draft(source, 0);
    let r = estimate(&d);
    amt_k1_note(&r, 10_000, 10_000);
    assert_eq!(restart(d.clone()).draft, d);
    // No box 17 code V statement: the K-1 adds no QBI, with a note.
    assert!(noted(&r, "WARN_K1_QBI_NO_STATEMENT"), "{r:?}");
}

#[test]
fn scorp_amt_uses_independent_basis_at_risk_and_passive_balances() {
    let mut source = passive_scorp_document();
    let mut amt = amt_records(&source);
    amt.adjustment = 500_000; // regular -15k -> AMT -10k
    amt.stock = 700_000;
    amt.loans[0].basis = 200_000;
    amt.risk.start = ScorpRiskStart::Detailed { capacity: 600_000, profit: 0, loss: 0 };
    amt.passive.loss = 100_000;
    source.amt = Some(amt);
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    add(&mut d, income(FederalIncome::Rental { data: rental(16_000, 0) }));
    let r = estimate(&d);
    passive_note(&r, id, -6_000, 4_000);
    assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!("AMT passive activity {id}, net after passive limits: $-6000; remaining passive loss: $1000."))));
    let amt_note = &r
        .review_notes
        .iter()
        .find(|n| n.starts_with(&alloc::format!("AMT passive activity {id},")))
        .unwrap()
        .display;
    assert_eq!(amt_note.key, "NOTE_AMT_PASSIVE_ACTIVITY_NET");
    assert_eq!(
        amt_note.args,
        vec![
            MessageArg::Number(id),
            MessageArg::usd_dollars(-6_000),
            MessageArg::usd_dollars(1_000)
        ]
    );
    for (kind, amount) in [("basis", 1_000), ("at-risk", 3_000)] {
        assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!(
            "AMT S-corporation document {id}, {kind} suspended loss: ${amount}."
        ))));
    }
}

#[test]
fn amt_k1_nonpassive_income_does_not_replace_regular_passive_allowance_magi() {
    let mut source = nonpassive_scorp(60_000, 0);
    let mut amt = amt_records(&source);
    amt.adjustment = 4_000_000;
    source.amt = Some(amt);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 20_000) }));
    let mut r = estimate(&d);
    amt_k1_note(&r, 100_000, 60_000);
    let alternate = crate::extended_rules::rentals_for(&d, 60_000, &mut r, true);
    assert_eq!(value(&r, "amt_rental_magi"), 120_000);
    assert_eq!(alternate.income, 85_000); // 100k K-1 less 15k special rental allowance
    assert_eq!(alternate.passive_loss, 15_000);
}

#[test]
fn amt_k1_loan_mapping_and_completeness_are_checked_without_regular_fallback() {
    let mut source = passive_scorp_document();
    let amt = amt_records(&source);
    for case in 0..3 {
        let mut bad = amt.clone();
        match case {
            0 => bad.loans.clear(),
            1 => bad.loans[0].loan = 2,
            _ => bad.loans[0].basis = 300_001,
        }
        source.amt = Some(bad);
        assert!(!source.valid());
    }
    // Other AMT differences: the regular records stand in, with a notice.
    let mut incomplete = amt;
    incomplete.complete = false;
    source.amt = Some(incomplete);
    assert!(source.valid());
    let (d, _) = investment_tax_draft(source.clone(), 0);
    assert!(noted(&estimate(&d), "WARN_SCORP_AMT_LEFT_OUT"));
    source.amt = None;
    let (d, _) = investment_tax_draft(source, 0);
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.section == "S-corporation AMT"));
    assert!(!r.review_notes.iter().any(|n| n.starts_with("K-1 AMT activity worksheet:")));
}

#[test]
fn k1_regular_and_amt_nol_use_separate_business_losses_without_personal_deduction() {
    let mut source = sole_qbi_loss(100_000, 100_000, 100_000);
    let mut amt = amt_records(&source);
    amt.adjustment = 2_000_000;
    source.amt = Some(amt);
    let mut d = complete(FederalStatus::Single); // 60k wages
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = estimate(&d);
    k1_nol_note(&r, "regular", 40_000, 0, 40_000);
    k1_nol_note(&r, "AMT", 20_000, 0, 20_000);
    assert_eq!(r.state, ResultState::EstimatedForSupportedScope);
    assert_eq!(value(&r, "current_net_operating_loss"), 40_000);
    assert_eq!(value(&r, "amt_current_net_operating_loss"), 20_000);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn k1_nol_adds_excess_business_loss_once_with_independent_amt_adjustment() {
    let mut source = sole_qbi_loss(400_000, 400_000, 200_000);
    let mut amt = amt_records(&source);
    amt.adjustment = 10_000_000;
    source.amt = Some(amt);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = estimate(&d);
    k1_nol_note(&r, "regular", 196_000, 144_000, 340_000);
    k1_nol_note(&r, "AMT", 196_000, 44_000, 240_000);
}

#[test]
fn k1_nol_excludes_losses_suspended_by_prior_activity_limits() {
    for (capacity, material, expected) in [(80_000, true, 20_000), (100_000, false, 0)] {
        let mut source = sole_qbi_loss(100_000, capacity, 50_000);
        if !material {
            source.participation = Some(participation_facts());
        }
        source.amt = Some(amt_records(&source));
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        let r = estimate(&d);
        k1_nol_note(&r, "regular", expected, 0, expected);
        k1_nol_note(&r, "AMT", expected, 0, expected);
    }
}

#[test]
fn k1_nol_does_not_publish_unresolved_alternative_balances() {
    let source = sole_qbi_loss(100_000, 100_000, 50_000);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.section == "S-corporation AMT"));
    assert!(!r.review_notes.iter().any(|n| n.contains("loss carryforward worksheet:")));
}

#[test]
fn scorp_owner_account_trading_is_nonpassive_without_material_participation() {
    for ordinary in [-15_000, 10_000] {
        let mut source = nonpassive_scorp(ordinary, 0);
        source.trade_kind = ScorpTradeKind::FinancialTrading;
        source.participation = None;
        let activity = source.activity_result().unwrap();
        assert_eq!(crate::kernel::ratio::dollars(activity.passive), 0);
        assert_eq!(
            crate::kernel::ratio::dollars(activity.nonpassive),
            if ordinary < 0 { -8_000 } else { 10_000 }
        );
        source.trade_kind = ScorpTradeKind::Ordinary;
        assert!(source.activity_result().is_err());
    }
}

#[test]
fn k1_without_qbi_statement_names_the_source_document() {
    // No box 17 code V statement: the K-1 adds no QBI, and the note names it.
    let (d, id) = investment_tax_draft(nonpassive_scorp(10_000, 0), 0);
    let r = estimate(&d);
    assert!(qbi_record_issues(&r, id).is_empty());
    let label = d.income.iter().find(|e| e.id == id).unwrap().label.clone();
    assert!(r.review_notes.iter().any(|n| n.display
        == Message::new("WARN_K1_QBI_NO_STATEMENT", alloc::vec![MessageArg::Text(label.clone())])));
}

#[test]
fn k1_confirmed_empty_qbi_limit_histories_need_no_further_records() {
    let mut source = sole_qbi_loss(10_000, 10_000, 0);
    source.loss_qbi.at_risk = vec![];
    source.loss_qbi.passive = vec![];
    let (d, id) = investment_tax_draft(source, 0);
    assert!(qbi_record_issues(&estimate(&d), id).is_empty());
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn complete_ordinary_k1_produces_1040_totals_without_self_employment_tax() {
    for material in [false, true] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(complete_ordinary_scorp(
                    10_000, material,
                )),
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 70_000);
        assert_eq!(value(&r, "qbi_deduction"), 2_000);
        assert_eq!(value(&r, "taxable_income"), 51_900);
        // $51,900 taxable income uses the $51,900–51,950 Tax Table row.
        assert_eq!(value(&r, "regular_income_tax"), 6_136);
        assert_eq!(value(&r, "self_employment_tax"), 0);
        assert_eq!(return_line(&r, "8"), 1_000_000);
        assert_eq!(return_line(&r, "15"), 5_190_000);
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn complete_ordinary_k1_loss_reaches_return_and_keeps_separate_qbi_carryforward() {
    let mut source = sole_qbi_loss(10_000, 10_000, 5_000);
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 50_000);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 5_000);
    assert_eq!(return_line(&r, "8"), -1_000_000);
}
#[test]
fn ordinary_k1_complete_scope_still_rejects_missing_sources_and_additional_items() {
    for case in 0..2 {
        let mut source = complete_ordinary_scorp(10_000, true);
        match case {
            0 => source.amt = None,
            _ => source.special = vec![crate::ScorpSpecial::Farming],
        }
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        let r = estimate(&d);
        // Incomplete basis records leave the K-1 out with a notice.
        if case == 1 {
            assert!(noted(&r, "WARN_SCORP_K1_LEFT_OUT"), "case {case}: {r:?}");
            continue;
        }
        assert_eq!(r.state, ResultState::Unsupported, "case {case}: {r:?}");
        assert!(r.tax.is_none() && r.refund.is_none() && r.return_lines.is_empty());
        assert!(!r.issues.is_empty());
    }
    // An incomplete QBI statement (decision 3): the corporation's QBI is left
    // out of the deduction with a notice that tax may be lower; a loss says
    // tax may be higher.
    let mut source = complete_ordinary_scorp(10_000, true);
    source.qbi_statement.as_mut().unwrap().complete = false;
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert!(noted(&r, "WARN_SCORP_QBI_LEFT_OUT"));
    assert_eq!(value(&r, "qbi_deduction"), 0);
    let mut source = sole_qbi_loss(10_000, 10_000, 5_000);
    source.amt = Some(amt_records(&source));
    source.qbi_statement.as_mut().unwrap().complete = false;
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    assert!(noted(&supported(&d), "WARN_SCORP_QBI_LOSS_LEFT_OUT"));
    // No section 199A statement (box 17 code V) with the K-1: it adds no
    // QBI (Reg. 1.199A-6(b)(3)(iii)); a loss is left out with a notice.
    let mut source = complete_ordinary_scorp(10_000, true);
    source.qbi_statement = None;
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert!(noted(&r, "WARN_K1_QBI_NO_STATEMENT"), "{r:?}");
    assert_eq!(value(&r, "qbi_deduction"), 0);
    let mut source = sole_qbi_loss(10_000, 10_000, 5_000);
    source.amt = Some(amt_records(&source));
    source.qbi_statement = None;
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    assert!(noted(&supported(&d), "WARN_K1_QBI_LOSS_NO_STATEMENT"));
}

#[test]
fn deceased_shareholder_k1_is_issued_for_the_decedents_period_and_suspended_losses_need_the_answer()
{
    // Pub. 559 "S Corporation Income": the K-1 enters as issued. Only a
    // suspended-loss history needs the death-adjustment answer.
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.died =
        Some(FinalReturn { bonds: None, date: 20260615, ordinary: true, estate_medical: 0 });
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(complete_ordinary_scorp(
                10_000, true,
            )),
        }),
    );
    supported(&d);
    let id = d.income.len() - 1;
    let FederalIncome::Scorp { data } = &mut d.income[id].income else { unreachable!() };
    // A passive loss suspended from 2025, with the records the K-1 needs.
    let mut with_history = nonpassive_scorp(10_000, 15_000);
    with_history.loss_qbi = ScorpLossQbi {
        passive: vec![loss_year(2020, 15_000, 7_500, 15_000, 7_500)],
        ..Default::default()
    };
    with_history.amt = Some(amt_records(&with_history));
    let mut without_history = with_history.clone();
    without_history.drop_suspended_history();
    without_history.loss_qbi =
        ScorpLossQbi { current: 0, history: vec![], at_risk: vec![], passive: vec![] };
    without_history.amt = Some(amt_records(&without_history));
    let _ = data;
    d.income[id].income =
        FederalIncome::Scorp { data: crate::tests::support::builders::scorp_source(with_history) };
    // No other adjustment applies cannot hold with a suspended loss.
    unsupported(&d, "Final-return K-1");
    // Saying one applies leaves the suspended losses out, with the notice.
    d.people.as_mut().unwrap().taxpayer.died.as_mut().unwrap().ordinary = false;
    let left_out = supported(&d);
    assert!(noted(&left_out, "WARN_FINAL_RETURN_ADJUSTMENTS_LEFT_OUT"));
    // The same return with the K-1 as if it never had the history.
    let mut control = d.clone();
    control.income[id].income = FederalIncome::Scorp {
        data: crate::tests::support::builders::scorp_source(without_history),
    };
    assert_eq!(left_out.tax, supported(&control).tax);
}
#[test]
fn complete_k1_recapture_still_blocks_regular_or_amt_return_independently() {
    for (regular, alternative) in [(true, false), (false, true), (true, true)] {
        let source = complete_scorp_recapture(regular, alternative);
        let mut d = complete(FederalStatus::Single);
        let id = d.next_id;
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        add(&mut d, income(FederalIncome::Rental { data: rental(16_000, 0) }));
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::Unsupported);
        assert!(r.tax.is_none() && r.refund.is_none() && r.return_lines.is_empty());
        assert!(qbi_record_issues(&r, id).is_empty()); // all required source records supplied
        let section = if regular { "S-corporation passive pool" } else { "S-corporation AMT" };
        assert!(
            r.issues
                .iter()
                .any(|i| i.section == section && i.message.contains("At-risk recapture"))
        );
        scorp_note(&r, id, "recapture income", if regular { 1_000 } else { 0 });
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn complete_k1_distribution_without_negative_at_risk_balance_remains_supported() {
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(complete_scorp_recapture(
                false, false,
            )),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 61_000);
    scorp_note(&r, id, "recapture income", 0);
    scorp_note(&r, id, "recapture deduction for next year", 0);
}

#[test]
fn k1_backup_withholding_increases_payments_once_without_changing_tax_or_basis() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(complete_ordinary_scorp(
                10_000, true,
            )),
        }),
    );
    let before = supported(&d);
    let FederalIncome::Scorp { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut source_edit = edit_scorp(data);
        let data = &mut *source_edit;
        let basis = data.basis_result().unwrap();
        data.withholding = 250_000;
        assert_eq!(data.basis_result().unwrap(), basis);
    }
    let after = supported(&d);
    for key in ["agi", "qbi_deduction", "regular_income_tax", "alternative_minimum_tax"] {
        assert_eq!(value(&before, key), value(&after, key));
    }
    assert_eq!(return_line(&after, "25d") - return_line(&before, "25d"), 250_000);
    assert_eq!(return_line(&after, "25c") - return_line(&before, "25c"), 250_000);
    assert_eq!(return_line(&after, "33") - return_line(&before, "33"), 250_000);
    assert_eq!(after.tax, before.tax);
    assert_eq!(after.refund.unwrap() - before.refund.unwrap(), 250_000);
    assert!(after.review_notes.iter().any(|n| n.contains("retain the K-1 for attachment")));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn k1_backup_withholding_is_not_capped_by_net_business_income() {
    let mut source = sole_qbi_loss(10_000, 10_000, 5_000);
    source.withholding = 1_000_000;
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 50_000);
    assert_eq!(value(&r, "scorp_2_backup_withholding"), 10_000);
    assert!(return_line(&r, "25d") >= 1_000_000);
}
#[test]
fn k1_backup_withholding_checks_stored_amount_bounds() {
    let mut source = complete_ordinary_scorp(10_000, true);
    source.withholding = 100_000_000_000;
    assert!(source.valid());
}

#[test]
fn multiple_k1_current_losses_with_uniform_qbi_character_produce_return_totals() {
    for (first_qbi, second_qbi, deferred, current) in
        [(100_000, 100_000, 72_000, 128_000), (200_000, 200_000, 144_000, 256_000), (0, 0, 0, 0)]
    {
        let mut d = complete(FederalStatus::Single);
        for qualified in [first_qbi, second_qbi] {
            let mut source = sole_qbi_loss(200_000, 200_000, qualified);
            source.amt = Some(amt_records(&source));
            add(
                &mut d,
                income(FederalIncome::Scorp {
                    data: crate::tests::support::builders::scorp_source(source),
                }),
            );
        }
        let r = supported(&d);
        assert_eq!(value(&r, "excess_business_loss_carryforward"), 144_000);
        assert_eq!(value(&r, "qbi_loss_carryforward"), current);
        assert_eq!(value(&r, "nol_carryforward"), 340_000);
        assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!(
            "K-1 excess-loss character: NOL carryforward $144000; qualified portion ${deferred}."
        ))));
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn multiple_k1_loss_only_qbi_proportions_allocate_current_excess() {
    let mut d = complete(FederalStatus::Single);
    for qualified in [100_000, 200_000] {
        let mut source = sole_qbi_loss(200_000, 200_000, qualified);
        source.amt = Some(amt_records(&source));
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
    }
    let r = supported(&d);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 144_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 192_000);
    assert!(r.review_notes.iter().any(|n| n.starts_with(
        "K-1 excess-loss character: NOL carryforward $144000; qualified portion $108000."
    )));
}

#[test]
fn uniform_k1_excess_uses_post_basis_losses_and_preserves_input_on_failure() {
    let first = sole_qbi_loss(400_000, 200_000, 200_000);
    let second = sole_qbi_loss(200_000, 200_000, 100_000);
    let mut activities = alloc::vec::Vec::new();
    for source in [&first, &second] {
        activities.extend(
            source
                .qbi_statement
                .as_ref()
                .unwrap()
                .activities(source, crate::kernel::carry::Amount::new(0).unwrap(), 0)
                .unwrap(),
        );
    }
    let before = activities.clone();
    assert!(
        crate::scorp_qbi::uniform_business_excess(&[&first, &second], &mut activities, 400_001)
            .is_err()
    );
    for (a, b) in activities.iter().zip(&before) {
        assert_eq!(a.income, b.income);
    }
    assert_eq!(
        crate::scorp_qbi::uniform_business_excess(&[&first, &second], &mut activities, 144_000)
            .unwrap(),
        72_000
    );
    assert_eq!(activities.iter().map(|v| v.income).sum::<i64>(), -128_000);
}

#[test]
fn k1_uniform_qbi_profit_offsets_current_loss_before_excess_limit() {
    for reverse in [false, true] {
        let mut loss = sole_qbi_loss(400_000, 400_000, 200_000);
        loss.amt = Some(amt_records(&loss));
        let mut profit = complete_ordinary_scorp(100_000, true);
        profit.qbi_statement.as_mut().unwrap().income = 5_000_000;
        let mut d = complete(FederalStatus::Single);
        let sources = if reverse { [profit, loss] } else { [loss, profit] };
        for data in sources {
            add(
                &mut d,
                income(FederalIncome::Scorp {
                    data: crate::tests::support::builders::scorp_source(data),
                }),
            );
        }
        let r = supported(&d);
        assert_eq!(value(&r, "excess_business_loss_carryforward"), 44_000);
        assert_eq!(value(&r, "qbi_loss_carryforward"), 128_000);
        assert_eq!(value(&r, "nol_carryforward"), 240_000);
        assert!(r.review_notes.iter().any(|n| n.starts_with(
            "K-1 excess-loss character: NOL carryforward $44000; qualified portion $22000."
        )));
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn k1_profit_with_different_qbi_fraction_does_not_receive_a_guessed_excess_allocation() {
    let mut loss = sole_qbi_loss(400_000, 400_000, 200_000);
    loss.amt = Some(amt_records(&loss));
    let profit = complete_ordinary_scorp(100_000, true); // 100% qualified profit, 50% qualified loss
    let mut d = complete(FederalStatus::Single);
    for data in [loss, profit] {
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(data),
            }),
        );
    }
    unsupported(&d, "K-1 excess-loss allocation");
}
#[test]
fn k1_uniform_excess_does_not_change_positive_qbi_activity() {
    let loss = sole_qbi_loss(400_000, 400_000, 200_000);
    let mut profit = complete_ordinary_scorp(100_000, true);
    profit.qbi_statement.as_mut().unwrap().income = 5_000_000;
    let sources = [&loss, &profit];
    let mut activities = alloc::vec::Vec::new();
    for source in sources {
        activities.extend(
            source
                .qbi_statement
                .as_ref()
                .unwrap()
                .activities(source, crate::kernel::carry::Amount::new(0).unwrap(), 0)
                .unwrap(),
        );
    }
    assert!(crate::scorp_qbi::uniform_business_excess(&sources, &mut activities, 300_001).is_err());
    assert_eq!((activities[0].income, activities[1].income), (-200_000, 50_000));
    assert_eq!(
        crate::scorp_qbi::uniform_business_excess(&sources, &mut activities, 44_000).unwrap(),
        22_000
    );
    assert_eq!((activities[0].income, activities[1].income), (-178_000, 50_000));
}

#[test]
fn sstb_k1_profit_eligibility_precedes_uniform_excess_loss_allocation() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 51_135_000, 0);
    let mut loss = sole_qbi_loss(400_000, 400_000, 200_000);
    loss.amt = Some(amt_records(&loss));
    let mut profit = complete_ordinary_scorp(100_000, true);
    profit.qbi_statement.as_mut().unwrap().service = true;
    for data in [loss, profit] {
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(data),
            }),
        );
    }
    let r = supported(&d);
    // Before-QBI taxable income 239,250 gives a 50% SSTB applicable percentage.
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 44_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 128_000);
    assert!(r.review_notes.iter().any(|n| n.starts_with(
        "K-1 excess-loss character: NOL carryforward $44000; qualified portion $22000."
    )));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn live_k1_sstb_income_wages_and_property_are_not_reduced_twice() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 15_535_000, 0);
    let mut source = complete_ordinary_scorp(100_000, true);
    let statement = source.qbi_statement.as_mut().unwrap();
    statement.service = true;
    statement.wages = 1_000_000;
    statement.property = 20_000_000;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    // Eligible QBI 50k, wages 5k, UBIA 100k. Wage/property limit 3,750;
    // 10,000 less half of (10,000 - 3,750) = 6,875.
    assert_eq!(value(&r, "qbi_deduction"), 6_875);
    let FederalIncome::Scorp { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut source_edit = edit_scorp(data);
        let data = &mut *source_edit;
        let statement = data.qbi_statement.as_mut().unwrap();
        statement.service = false;
        statement.income /= 2;
        statement.wages /= 2;
        statement.property /= 2;
    }
    assert_eq!(value(&supported(&d), "qbi_deduction"), 6_875);
}
#[test]
fn k1_service_classification_computes_both_answers_at_high_income() {
    let mut source = complete_ordinary_scorp(100_000, true);
    source.qbi_statement.as_mut().unwrap().service = true;
    let (d, _) = investment_tax_draft(source.clone(), 0);
    let specified = supported(&d);
    source.qbi_statement.as_mut().unwrap().service = false;
    let (d, _) = investment_tax_draft(source, 0);
    assert!(specified.tax > supported(&d).tax);
}

#[test]
fn released_historical_k1_losses_with_single_qbi_character_reach_form461() {
    for at_risk in [false, true] {
        for qualified in [0, 400_000] {
            let mut d = complete(FederalStatus::Single);
            add(
                &mut d,
                income(FederalIncome::Scorp {
                    data: crate::tests::support::builders::scorp_source(historical_k1_excess(
                        qualified, at_risk,
                    )),
                }),
            );
            let r = supported(&d);
            assert_eq!(value(&r, "excess_business_loss_carryforward"), 144_000);
            assert_eq!(
                value(&r, "qbi_loss_carryforward"),
                if qualified == 0 { 0 } else { 256_000 }
            );
            assert_eq!(value(&r, "nol_carryforward"), 340_000);
            let deferred = if qualified == 0 { 0 } else { 144_000 };
            assert!(r.review_notes.iter().any(|n| n.starts_with(&alloc::format!("K-1 excess-loss character: NOL carryforward $144000; qualified portion ${deferred}."))));
            assert_eq!(restart(d.clone()).draft, d);
        }
    }
}
#[test]
fn historical_k1_mixed_qualified_character_allocates_to_new_loss_limit() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(historical_k1_excess(
                200_000, false,
            )),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 128_000);
    assert!(r.review_notes.iter().any(|n| n.starts_with(
        "K-1 excess-loss character: NOL carryforward $144000; qualified portion $72000."
    )));
}

#[test]
fn shareholder_loan_output_shows_open_account_threshold_and_next_year_records() {
    for (principal, advance, separate) in [(20_000, 5_000, false), (20_000, 5_001, true)] {
        let mut source = complete_ordinary_scorp(10_000, true);
        source.loans = vec![ScorpLoan {
            repayment_term: ScorpNoteHolding::NoRepayment,
            kind: ScorpLoanKind::OpenAccount,
            principal: principal * 100,
            basis: principal * 100,
            advances: advance * 100,
            repayments: 0,
        }];
        source.amt = Some(amt_records(&source));
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        let r = supported(&d);
        let expected = if separate {
            "Next year: retain this as a separate former open account that exceeded $25000."
        } else {
            "Next year: retain this current open account; annual advance/repayment netting applies."
        };
        assert!(r.review_notes.iter().any(|n| n.contains(expected)));
        assert!(r.review_notes.iter().any(|n| {
            n.contains(&alloc::format!("loan 1 principal: ${}.00.", principal + advance))
        }));
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn shareholder_repaid_former_open_account_keeps_classification_and_nontaxable_amount() {
    let mut source = complete_ordinary_scorp(10_000, true);
    source.loans = vec![ScorpLoan {
        repayment_term: ScorpNoteHolding::NoRepayment,
        kind: ScorpLoanKind::FormerOpenAccount,
        principal: 500_000,
        basis: 500_000,
        advances: 0,
        repayments: 100_000,
    }];
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert!(r.review_notes.iter().any(|n| n.contains(
        "loan 1 principal: $4000.00. Next year: retain this as a separate former open account"
    )));
    assert!(r.review_notes.iter().any(|n| n.contains("loan 1 nontaxable repayment: $1000.00.")));
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn scorp_tax_exempt_sources_release_loss_without_becoming_business_income() {
    let mut source = sole_qbi_loss(6_000, 0, 3_000);
    source.exempt_interest = 200_000;
    source.exempt = 200_000;
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(
        basis.allowed,
        LossAmounts::of(&[(
            K1LossClass::Ordinary,
            crate::kernel::carry::Amount::new(4_000 * 100).unwrap()
        )])
    );
    assert_eq!(
        basis.suspended,
        LossAmounts::of(&[(
            K1LossClass::Ordinary,
            crate::kernel::carry::Amount::new(2_000 * 100).unwrap()
        )])
    );
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        (
            risk.investment.map(crate::kernel::ratio::dollars),
            crate::kernel::ratio::dollars(risk.income.get()),
            crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Ordinary].get())
        ),
        (Some(4_000), 0, 4_000)
    );
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 56_000);
    assert_eq!(return_line(&r, "2a"), 200_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 2_000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn scorp_private_activity_interest_matches_the_shared_interest_amt_path() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    let mut source = complete_ordinary_scorp(10_000, true);
    source.exempt_interest = 3_000_000;
    source.private_activity = 2_000_000;
    source.amt = Some(amt_records(&source));
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let actual = supported(&d);
    let FederalIncome::Scorp { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut source_edit = edit_scorp(data);
        let data = &mut *source_edit;
        data.exempt_interest = 0;
        data.private_activity = 0;
    }
    add(
        &mut d,
        income(FederalIncome::Interest {
            data: interest_source(native_exempt_interest(3_000_000, 2_000_000)),
        }),
    );
    let expected = supported(&d);
    assert_eq!(return_line(&actual, "2a"), 3_000_000);
    for name in
        ["agi", "alternative_minimum_taxable_income", "qbi_deduction", "net_investment_income_tax"]
    {
        assert_eq!(value(&actual, name), value(&expected, name), "{name}");
    }
    assert_eq!(actual.tax, expected.tax);
}
#[test]
fn scorp_exempt_interest_enters_social_security_combined_income_but_other_exempt_does_not() {
    let mut d = empty_return(FederalStatus::Single);
    let mut source = complete_ordinary_scorp(10_000, true);
    source.exempt = 1_000_000;
    source.amt = Some(amt_records(&source));
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    add(&mut d, income(social_security_net(2_000_000, 0)));
    assert_eq!(value(&supported(&d), "taxable_social_security"), 0);
    let FederalIncome::Scorp { data } = &mut d.income[0].income else { unreachable!() };
    {
        let mut source_edit = edit_scorp(data);
        let data = &mut *source_edit;
        data.exempt = 0;
        data.exempt_interest = 1_000_000;
    }
    let r = supported(&d);
    // 10k ordinary + 10k exempt interest + half of 20k benefits = 30k.
    assert_eq!(value(&r, "taxable_social_security"), 2_500);
    assert_eq!(return_line(&r, "2a"), 1_000_000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn scorp_private_activity_amount_is_a_subset_not_additional_exempt_income() {
    let mut source = complete_ordinary_scorp(0, true);
    source.private_activity = 1;
    assert!(!source.valid());
    source.exempt_interest = 1;
    assert!(source.valid());
}
#[test]
fn open_account_gain_is_excluded_from_the_at_risk_principal_reduction() {
    for kind in [ScorpLoanKind::OpenAccount, ScorpLoanKind::FormerOpenAccount] {
        let source = scorp_repaid_reduced_open_account(kind);
        let basis = source.basis_result().unwrap();
        assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].ordinary_gain.get()), 2_000);
        assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].nontaxable_repayment.get()), 3_000);
        let risk = source.risk.calculate(&source, &basis).unwrap();
        assert_eq!(
            (
                risk.investment.map(crate::kernel::ratio::dollars),
                risk.year_end.map(crate::kernel::ratio::dollars),
                crate::kernel::ratio::dollars(risk.income.get())
            ),
            (Some(3_000), Some(3_000), 0)
        );
        assert_eq!(crate::kernel::ratio::dollars(risk.recapture.income.get()), 0);
        let mut d = complete(FederalStatus::Single);
        let id = d.next_id;
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        let r = estimate(&d);
        scorp_note(&r, id, "year-end at-risk investment before recapture", 3_000);
        assert_eq!(r.state, ResultState::EstimatedForSupportedScope);
        assert_eq!(value(&r, "scorp_ordinary_loan_gain"), 2_000);
        // Form 4797 Part II line 11 (4797-D4).
        assert_eq!(printed_line(&r, "scorp_ordinary_loan_gain"), (TaxForm::Form4797, "11".into()));
        assert_eq!(value(&r, "agi"), 62_000);
        assert_eq!(return_line(&r, "8"), 200_000);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn open_account_netting_and_separate_debt_repayments_preserve_at_risk_basis_recovery() {
    for (kind, ending) in
        [(ScorpLoanKind::OpenAccount, 3_000), (ScorpLoanKind::FormerOpenAccount, 3_333)]
    {
        let mut source = scorp_repaid_reduced_open_account(kind);
        source.loans[0].advances = 200_000;
        source.loans[0].repayments = 700_000;
        let basis = source.basis_result().unwrap();
        let risk = source.risk.calculate(&source, &basis).unwrap();
        assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].basis.get()), ending);
        assert_eq!(risk.year_end.map(crate::kernel::ratio::dollars), Some(ending));
        assert_eq!(crate::kernel::ratio::dollars(risk.income.get()), 0);
    }
}

#[test]
fn ordinary_shareholder_loan_gain_is_investment_income_without_earned_or_qbi_income() {
    let source = scorp_repaid_reduced_open_account(ScorpLoanKind::OpenAccount);
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 302_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 76);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    k1_eic_investment_note(&r, 0);
    assert_eq!(return_line(&r, "8"), 200_000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn ordinary_shareholder_loan_gain_funds_separate_regular_and_amt_investment_interest() {
    let mut source = scorp_repaid_reduced_open_account(ScorpLoanKind::OpenAccount);
    let amt = source.amt.as_mut().unwrap();
    amt.loans[0].basis = 800_000;
    amt.risk.start = ScorpRiskStart::Detailed { capacity: 800_000, profit: 0, loss: 0 };
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    investment_borrowing(&mut d, 3_000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 2_000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 1_000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 1_000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 2_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
}
#[test]
fn amt_only_shareholder_loan_gain_does_not_invent_regular_income() {
    let mut source = scorp_repaid_reduced_open_account(ScorpLoanKind::OpenAccount);
    source.loans[0].basis = 1_000_000;
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "scorp_ordinary_loan_gain"), 0);
    assert_eq!(value(&r, "scorp_amt_loan_gain_adjustment"), 2_000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 62_000);
}
#[test]
fn shareholder_loan_repayment_gains_cannot_release_corporate_passive_losses() {
    for mut source in [
        scorp_repaid_reduced_open_account(ScorpLoanKind::OpenAccount),
        repaid_written_note(GainTerm::Short),
        repaid_written_note(GainTerm::Long),
    ] {
        source.participation = Some(participation_facts());
        source.passive.loss = 1_000_000;
        source.loss_qbi = ScorpLossQbi {
            passive: vec![loss_year(2017, 10_000, 0, 10_000, 0)],
            ..Default::default()
        };
        source.amt = Some(amt_records(&source));
        let mut d = complete(FederalStatus::Single);
        let id = d.next_id;
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 62_000);
        passive_note(&r, id, 0, 10_000);
    }
}

#[test]
fn missing_amt_loan_basis_does_not_publish_an_invented_gain_adjustment() {
    let mut source = scorp_repaid_reduced_open_account(ScorpLoanKind::OpenAccount);
    source.amt = None;
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = estimate(&d);
    assert_eq!(r.state, ResultState::Unsupported);
    scorp_note(&r, id, "loan 1 gain", 2_000);
    assert!(r.lines.is_empty());
    assert!(r.return_lines.is_empty());
    assert!(r.tax.is_none());
    assert!(r.refund.is_none());
    assert!(r.issues.iter().any(|issue| issue.section == "S-corporation AMT"));
}
#[test]
fn written_note_gain_uses_independent_amt_basis_and_recovered_at_risk_principal() {
    for term in [GainTerm::Short, GainTerm::Long] {
        let mut source = repaid_written_note(term);
        let amt = source.amt.as_mut().unwrap();
        amt.loans[0].basis = 800_000;
        amt.risk.start = ScorpRiskStart::Detailed { capacity: 800_000, profit: 0, loss: 0 };
        let basis = source.basis_result().unwrap();
        let risk = source.risk.calculate(&source, &basis).unwrap();
        assert_eq!(risk.year_end.map(crate::kernel::ratio::dollars), Some(3_000));
        assert_eq!(crate::kernel::ratio::dollars(risk.income.get()), 0);
        let alt = source.amt.as_ref().unwrap().source(&source).unwrap();
        let alt_basis = alt.basis_result().unwrap();
        assert_eq!(
            alt.risk
                .calculate(&alt, &alt_basis)
                .unwrap()
                .year_end
                .map(crate::kernel::ratio::dollars),
            Some(4_000)
        );
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
        let mut comparison = d.clone();
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        add(&mut comparison, income(investment_sale_gain(term, 2_000, 1_000)));
        let r = supported(&d);
        let expected = supported(&comparison);
        assert_eq!(r.return_lines, expected.return_lines);
        for key in [
            "capital_gain",
            "amt_capital_gain",
            "alternative_minimum_taxable_income",
            "net_investment_income_tax",
        ] {
            assert_eq!(value(&r, key), value(&expected, key));
        }
        assert_eq!(value(&r, "amt_capital_gain"), 1_000);
    }
}
#[test]
fn mixed_note_new_advances_require_transaction_basis_instead_of_annual_proration() {
    let mut source = mixed_note_repayments();
    source.loans[0].advances = 100_000;
    assert!(source.valid());
    assert_eq!(
        source.loan_gains(&source.basis_result().unwrap()).unwrap_err().0,
        "ISSUE_SCORP_NOTE_ADVANCES_WITH_DATED_REPAYMENTS"
    );
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    unsupported(&d, "S-corporation loan holding period");
}
#[test]
fn dated_note_basis_restoration_precedes_both_holding_periods() {
    let mut source = mixed_note_repayments();
    source.ordinary = 200_000;
    source.qbi_statement.as_mut().unwrap().income = 200_000;
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].restoration.get()), 2_000);
    assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].capital_gain.get()), 1_000);
    assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].basis.get()), 4_000);
    assert_eq!(
        source.loan_gains(&basis).unwrap().map(crate::kernel::ratio::dollars),
        [0, 400, 600]
    );
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 63_000);
    assert_eq!(value(&r, "capital_gain"), 1_000);
}
#[test]
fn k1_portfolio_income_matches_separate_interest_and_dividend_return_paths() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    let mut comparison = d.clone();
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(scorp_portfolio()),
        }),
    );
    add(
        &mut comparison,
        income(FederalIncome::Interest { data: interest_source(native_interest(300_000, 0)) }),
    );
    add(
        &mut comparison,
        income(FederalIncome::Dividend {
            data: dividend_source(FederalDividend {
                section199a: ReitDividends::None,
                ..native_dividend(400_000, 200_000, 0)
            }),
        }),
    );
    let r = supported(&d);
    assert_eq!(r.return_lines, supported(&comparison).return_lines);
    assert_eq!(return_line(&r, "2b"), 300_000);
    assert_eq!(return_line(&r, "3a"), 200_000);
    assert_eq!(return_line(&r, "3b"), 400_000);
    assert_eq!(return_line(&r, "8"), 0);
    assert_eq!(value(&r, "agi"), 307_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 266);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(value(&r, "self_employment_tax"), 0);
    k1_eic_investment_note(&r, 7_000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn k1_portfolio_increases_basis_and_same_activity_risk_without_becoming_passive_income() {
    for material in [true, false] {
        let mut source = sole_qbi_loss(20_000, 0, 20_000);
        source.interest = 1_000_000;
        source.dividends = 500_000;
        source.qualified_dividends = 300_000;
        if !material {
            source.participation = Some(participation_facts());
        }
        source.amt = Some(amt_records(&source));
        let basis = source.basis_result().unwrap();
        assert_eq!(
            basis.allowed,
            LossAmounts::of(&[(
                K1LossClass::Ordinary,
                crate::kernel::carry::Amount::new(15_000 * 100).unwrap()
            )])
        );
        assert_eq!(
            basis.suspended,
            LossAmounts::of(&[(
                K1LossClass::Ordinary,
                crate::kernel::carry::Amount::new(5_000 * 100).unwrap()
            )])
        );
        let risk = source.risk.calculate(&source, &basis).unwrap();
        assert_eq!(crate::kernel::ratio::dollars(risk.income.get()), 15_000);
        assert_eq!(
            crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Ordinary].get()),
            15_000
        );
        assert_eq!(
            crate::kernel::ratio::dollars(risk.ledger.risk_suspended[K1LossClass::Ordinary].get()),
            0
        );
        assert_eq!(risk.year_end.map(crate::kernel::ratio::dollars), Some(0));
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
        let id = d.next_id;
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), if material { 300_000 } else { 315_000 });
        assert_eq!(value(&r, "qbi_loss_carryforward"), if material { 15_000 } else { 0 });
        if !material {
            passive_note(&r, id, 0, 15_000);
        }
        k1_eic_investment_note(&r, 15_000);
        assert_eq!(value(&r, "net_investment_income_tax"), 570);
        scorp_note(&r, id, "net loss absorbed by at-risk including nondeductible expenses", 0);
    }
}
#[test]
fn k1_portfolio_loss_limits_use_independent_regular_and_amt_balances() {
    let mut source = sole_qbi_loss(20_000, 5_000, 20_000);
    source.interest = 1_000_000;
    let mut amt = amt_records(&source);
    amt.stock = 0;
    amt.risk.start = ScorpRiskStart::Detailed { capacity: 0, profit: 0, loss: 0 };
    source.amt = Some(amt);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 55_000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 60_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 15_000);
    amt_k1_note(&r, -10_000, -15_000);
}
#[test]
fn k1_portfolio_never_releases_a_former_or_current_passive_loss() {
    for material in [true, false] {
        let mut source = scorp_portfolio();
        if !material {
            source.participation = Some(participation_facts());
        }
        source.passive.loss = 1_000_000;
        source.loss_qbi = ScorpLossQbi {
            passive: vec![loss_year(2017, 10_000, 0, 10_000, 0)],
            ..Default::default()
        };
        source.amt = Some(amt_records(&source));
        let mut d = complete(FederalStatus::Single);
        let id = d.next_id;
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 67_000);
        passive_note(&r, id, 0, 10_000);
        assert_eq!(value(&r, "qbi_deduction"), 0);
    }
}
#[test]
fn k1_qualified_dividends_are_excluded_from_investment_interest_without_election() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(scorp_portfolio()),
        }),
    );
    investment_borrowing(&mut d, 8_000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 5_000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 5_000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 3_000);
}
#[test]
fn k1_portfolio_source_bounds_and_activity_confirmation_remain_required() {
    let mut source = scorp_portfolio();
    source.qualified_dividends = source.dividends + 1;
    assert!(!source.valid());
    source.qualified_dividends = source.dividends;
    assert!(source.valid());
    source.interest = 0;
    source.special = vec![crate::ScorpSpecial::Farming];
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    assert!(noted(&supported(&d), "WARN_SCORP_K1_LEFT_OUT"));
}
#[test]
fn k1_portfolio_restores_reduced_debt_basis_without_duplicate_qualified_dividends() {
    let mut source = repaid_written_note(GainTerm::Long);
    source.interest = 200_000;
    source.dividends = 100_000;
    source.qualified_dividends = 100_000;
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].restoration.get()), 3_000);
    assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].basis.get()), 4_500);
    assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].capital_gain.get()), 500);
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 63_500);
    assert_eq!(value(&r, "capital_gain"), 500);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    scorp_note(&r, id, "year-end at-risk investment before recapture", 4_500);
}
#[test]
fn k1_capital_gains_match_investment_returns_with_independent_amt_amounts() {
    let mut source = complete_ordinary_scorp(0, true);
    source.capital = Some(k1_capital(3_000, 5_000));
    source.amt = Some(amt_records(&source));
    source.amt.as_mut().unwrap().capital = Some(k1_capital(1_000, 2_000));
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
    let mut comparison = d.clone();
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    add(&mut comparison, income(investment_sale_gain(GainTerm::Short, 3_000, 1_000)));
    add(&mut comparison, income(investment_sale_gain(GainTerm::Long, 5_000, 2_000)));
    let r = supported(&d);
    assert_eq!(r.return_lines, supported(&comparison).return_lines);
    assert_eq!(value(&r, "capital_gain"), 8_000);
    assert_eq!(value(&r, "amt_capital_gain"), 3_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 304);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    k1_eic_investment_note(&r, 8_000);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn k1_ordinary_and_capital_losses_share_basis_and_risk_without_losing_character() {
    let mut source = sole_qbi_loss(30_000, 30_000, 30_000);
    source.capital = Some(k1_capital(-10_000, -20_000));
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_200_000, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(basis.allowed, four_classes([15_000, 5_000, 10_000, 0]));
    assert_eq!(basis.suspended, four_classes([15_000, 5_000, 10_000, 0]));
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Ordinary].get()),
        6_000
    );
    assert_eq!(
        risk.ledger.risk_allowed.capital().map(crate::kernel::ratio::dollars),
        [2_000, 4_000, 0]
    );
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_suspended[K1LossClass::Ordinary].get()),
        9_000
    );
    assert_eq!(
        risk.ledger.risk_suspended.capital().map(crate::kernel::ratio::dollars),
        [3_000, 6_000, 0]
    );
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 51_000);
    assert_eq!(value(&r, "capital_gain"), -3_000);
    assert_eq!(value(&r, "capital_short_carryforward"), 0);
    assert_eq!(value(&r, "capital_long_carryforward"), 3_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 6_000);
    for tax in ["regular", "AMT"] {
        assert!(r.review_notes.iter().any(|n| n.contains(&alloc::format!(
            "{tax} short-term portfolio capital: net -$2000.00; basis-suspended loss $5000.00; at-risk-suspended loss $3000.00."
        ))));
        assert!(r.review_notes.iter().any(|n| n.contains(&alloc::format!(
            "{tax} long-term excluding collectibles portfolio capital: net -$4000.00; basis-suspended loss $10000.00; at-risk-suspended loss $6000.00."
        ))));
    }
    let message = r
        .review_notes
        .iter()
        .map(|note| &note.display)
        .find(|message| message.key == "NOTE_SCORP_CAPITAL_REGULAR_SHORT")
        .expect("localized capital summary");
    assert_eq!(
        message.args,
        vec![
            crate::MessageArg::Number(id),
            crate::MessageArg::usd_cents(-200_000),
            crate::MessageArg::usd_cents(500_000),
            crate::MessageArg::usd_cents(300_000),
        ]
    );
    scorp_note(&r, id, "net loss absorbed by at-risk including nondeductible expenses", 12_000);
}
#[test]
fn k1_prior_at_risk_capital_losses_do_not_consume_basis_twice() {
    let mut source = complete_ordinary_scorp(0, true);
    source.opening = Some(0);
    source.loans.clear();
    let mut capital = k1_capital(0, 0);
    capital.risk_short = 600_000;
    capital.risk_long = 400_000;
    source.capital = Some(capital);
    source.risk.start = ScorpRiskStart::Detailed { capacity: 500_000, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 0);
    assert_eq!(basis.allowed, LossAmounts::default());
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        risk.ledger.risk_allowed.capital().map(crate::kernel::ratio::dollars),
        [3_000, 2_000, 0]
    );
    assert_eq!(
        risk.ledger.risk_suspended.capital().map(crate::kernel::ratio::dollars),
        [3_000, 2_000, 0]
    );
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 57_000);
    assert_eq!(value(&r, "capital_long_carryforward"), 2_000);
}
#[test]
fn k1_capital_gains_can_fund_ordinary_loss_limits_without_becoming_qbi() {
    let mut source = sole_qbi_loss(10_000, 0, 10_000);
    source.capital = Some(k1_capital(4_000, 6_000));
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(basis.allowed, four_classes([10_000, 0, 0, 0]));
    assert_eq!(
        crate::kernel::ratio::dollars(source.activity_result().unwrap().nonpassive),
        -10_000
    );
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "capital_gain"), 10_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 10_000);
}
#[test]
fn k1_missing_amt_capital_records_never_substitute_zero_or_regular_amounts() {
    let mut source = complete_ordinary_scorp(0, true);
    source.capital = Some(k1_capital(1_000, 0));
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    unsupported(&d, "S-corporation AMT");
}
#[test]
fn k1_capital_source_rejects_impossible_history() {
    let mut source = sole_qbi_loss(0, 0, 0);
    source.capital = Some(ScorpCapital { risk_short: 100, ..k1_capital(0, 0) });
    source.risk.start = ScorpRiskStart::New;
    source.risk.deducted = 0;
    assert!(!source.valid());
}

#[test]
fn k1_capital_loss_histories_and_schedule_d_carries_differ_for_amt() {
    let mut source = complete_ordinary_scorp(0, true);
    source.loans.clear();
    source.opening = Some(1_000_000);
    source.capital = Some(k1_capital(-6_000, -4_000));
    source.risk.start = ScorpRiskStart::Detailed { capacity: 500_000, profit: 0, loss: 0 };
    let mut amt = amt_records(&source);
    amt.stock = 400_000;
    amt.risk.start = ScorpRiskStart::Detailed { capacity: 200_000, profit: 0, loss: 0 };
    source.amt = Some(amt);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), -3_000);
    assert_eq!(value(&r, "capital_long_carryforward"), 2_000);
    assert_eq!(value(&r, "amt_capital_gain"), -2_000);
    assert_eq!(value(&r, "amt_capital_long_carryforward"), 0);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 58_000);
    assert!(r.review_notes.iter().any(|n| n.contains("AMT short-term portfolio capital: net -$1200.00; basis-suspended loss $3600.00; at-risk-suspended loss $1200.00.")));
    assert!(r.review_notes.iter().any(|n| n.contains("AMT long-term excluding collectibles portfolio capital: net -$800.00; basis-suspended loss $2400.00; at-risk-suspended loss $800.00.")));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn k1_capital_investment_interest_uses_each_tax_system_and_excludes_unelected_long_gain() {
    let mut source = complete_ordinary_scorp(0, true);
    source.capital = Some(k1_capital(3_000, 5_000));
    source.amt = Some(amt_records(&source));
    source.amt.as_mut().unwrap().capital = Some(k1_capital(1_000, 2_000));
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    investment_borrowing(&mut d, 4_000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 3_000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 1_000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 1_000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 3_000);
}

#[test]
fn k1_mixed_capital_categories_share_basis_with_ordinary_losses_before_schedule_d() {
    // $10,000 of positive capital income increases basis. The $20,000 total
    // ordinary/capital losses share it equally; cross-netting capital first
    // would wrongly suspend the entire ordinary loss.
    for capital in [k1_capital(10_000, -10_000), k1_capital(-10_000, 10_000)] {
        let mut source = sole_qbi_loss(10_000, 0, 10_000);
        let short_gain = capital.short > 0;
        source.capital = Some(capital);
        source.amt = Some(amt_records(&source));
        let basis = source.basis_result().unwrap();
        let expected = if short_gain {
            four_classes([5_000, 0, 5_000, 0])
        } else {
            four_classes([5_000, 5_000, 0, 0])
        };
        assert_eq!(basis.allowed, expected);
        assert_eq!(basis.suspended, expected);
        assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 0);
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 60_000);
        assert_eq!(value(&r, "capital_gain"), 5_000);
        assert_eq!(value(&r, "qbi_loss_carryforward"), 5_000);
        assert_eq!(return_line(&r, "8"), -500_000);
        assert_eq!(value(&r, "capital_short_carryforward"), 0);
        assert_eq!(value(&r, "capital_long_carryforward"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn k1_prior_basis_capital_loss_competes_with_current_ordinary_loss_even_with_capital_gain() {
    for same_term in [true, false] {
        let mut source = sole_qbi_loss(10_000, 0, 10_000);
        source.capital = Some(ScorpCapital {
            basis_short: if same_term { 1_000_000 } else { 0 },
            basis_long: if same_term { 0 } else { 1_000_000 },
            ..k1_capital(10_000, 0)
        });
        source.amt = Some(amt_records(&source));
        let basis = source.basis_result().unwrap();
        let expected = if same_term {
            four_classes([5_000, 5_000, 0, 0])
        } else {
            four_classes([5_000, 0, 5_000, 0])
        };
        assert_eq!(basis.allowed, expected);
        assert_eq!(basis.suspended, expected);
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 60_000);
        assert_eq!(value(&r, "capital_gain"), 5_000);
        assert_eq!(value(&r, "qbi_loss_carryforward"), 5_000);
    }
}

#[test]
fn k1_mixed_capital_amt_recomputes_loss_allocation_and_investment_interest() {
    let mut source = sole_qbi_loss(12_000, 0, 12_000);
    source.capital = Some(k1_capital(12_000, -12_000));
    let mut amt = amt_records(&source);
    amt.capital = Some(k1_capital(6_000, -6_000));
    source.amt = Some(amt);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    investment_borrowing(&mut d, 8_000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "capital_gain"), 6_000);
    assert_eq!(value(&r, "amt_capital_gain"), 4_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 6_000);
    assert_eq!(value(&r, "investment_interest_deduction"), 6_000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 4_000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 2_000);
    assert_eq!(value(&r, "amt_investment_interest_carryforward"), 4_000);
    amt_k1_note(&r, -4_000, -6_000);
    assert!(r.review_notes.iter().any(|n| n.contains("AMT long-term excluding collectibles portfolio capital: net -$2000.00; basis-suspended loss $4000.00; at-risk-suspended loss $0.00.")));
}

#[test]
fn k1_capital_gain_and_prior_risk_loss_preserve_basis_and_risk_separation() {
    let mut source = sole_qbi_loss(12_000, 0, 12_000);
    source.capital = Some(ScorpCapital { risk_long: 2_400_000, ..k1_capital(12_000, 0) });
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(basis.allowed, four_classes([12_000, 0, 0, 0]));
    assert_eq!(basis.suspended, LossAmounts::default());
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Ordinary].get()),
        4_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_suspended[K1LossClass::Ordinary].get()),
        8_000
    );
    assert_eq!(
        risk.ledger.risk_allowed.capital().map(crate::kernel::ratio::dollars),
        [0, 8_000, 0]
    );
    assert_eq!(
        risk.ledger.risk_suspended.capital().map(crate::kernel::ratio::dollars),
        [0, 16_000, 0]
    );
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "capital_gain"), 4_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 4_000);
}

#[test]
fn k1_mixed_capital_with_sufficient_basis_matches_separate_investment_sales() {
    for (short, long) in [(4_000, -1_000), (-1_000, 4_000), (1_000, -4_000), (-4_000, 1_000)] {
        let mut source = complete_ordinary_scorp(0, true);
        source.opening = Some(1_000_000);
        source.loans.clear();
        source.risk.start = ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 };
        source.capital = Some(k1_capital(short, long));
        source.amt = Some(amt_records(&source));
        assert_eq!(source.basis_result().unwrap().suspended, LossAmounts::default());
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
        let mut comparison = d.clone();
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        add(&mut comparison, income(investment_sale_gain(GainTerm::Short, short, short)));
        add(&mut comparison, income(investment_sale_gain(GainTerm::Long, long, long)));
        let r = supported(&d);
        let expected = supported(&comparison);
        assert_eq!(r.return_lines, expected.return_lines);
        for name in [
            "capital_gain",
            "amt_capital_gain",
            "net_investment_income_tax",
            "capital_short_carryforward",
            "capital_long_carryforward",
        ] {
            assert_eq!(value(&r, name), value(&expected, name));
        }
    }
}
#[test]
fn k1_collectible_component_matches_sales_without_double_counting_box_8a() {
    for (long, collectible) in [(5_000, 3_000), (1_000, 4_000), (-1_000, -4_000), (-5_000, -3_000)]
    {
        let mut source = complete_ordinary_scorp(0, true);
        source.loans.clear();
        source.opening = Some(2_000_000);
        source.risk.start = ScorpRiskStart::Detailed { capacity: 2_000_000, profit: 0, loss: 0 };
        source.capital = Some(k1_collectible_capital(-1_000, long, collectible));
        let mut amt = amt_records(&source);
        amt.capital = Some(k1_collectible_capital(1_000, 2_000, -1_000));
        source.amt = Some(amt);
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
        let mut comparison = d.clone();
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        add(&mut comparison, income(investment_sale_gain(GainTerm::Short, -1_000, 1_000)));
        add(
            &mut comparison,
            income(investment_sale_gain(GainTerm::Long, long - collectible, 3_000)),
        );
        add(&mut comparison, income(collectible_investment_gain(collectible, -1_000)));
        let r = supported(&d);
        let expected = supported(&comparison);
        assert_eq!(r.return_lines, expected.return_lines);
        for name in [
            "capital_gain",
            "amt_capital_gain",
            "rate_28_gain",
            "amt_rate_28_gain",
            "net_investment_income_tax",
            "capital_long_carryforward",
        ] {
            assert_eq!(value(&r, name), value(&expected, name));
        }
        assert_eq!(value(&r, "qbi_deduction"), 0);
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn k1_collectible_loss_keeps_its_rate_character_through_both_limits() {
    let mut source = sole_qbi_loss(30_000, 35_000, 30_000);
    source.capital = Some(k1_collectible_capital(-10_000, -30_000, -20_000));
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_400_000, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(basis.allowed, four_classes([15_000, 5_000, 5_000, 10_000]));
    assert_eq!(basis.suspended, basis.allowed);
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Ordinary].get()),
        6_000
    );
    assert_eq!(
        risk.ledger.risk_allowed.capital().map(crate::kernel::ratio::dollars),
        [2_000, 2_000, 4_000]
    );
    assert_eq!(
        risk.ledger.risk_suspended.capital().map(crate::kernel::ratio::dollars),
        [3_000, 3_000, 6_000]
    );
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 51_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 6_000);
    assert_eq!(value(&r, "capital_long_carryforward"), 5_000);
    for system in ["regular", "AMT"] {
        assert!(r.review_notes.iter().any(|n| n.contains(&alloc::format!("{system} long-term collectibles (28%) portfolio capital: net -$4000.00; basis-suspended loss $10000.00; at-risk-suspended loss $6000.00."))));
        assert!(r.review_notes.iter().any(|n| n.contains(&alloc::format!("{system} total long-term portfolio capital including collectibles: net -$6000.00; basis-suspended loss $15000.00; at-risk-suspended loss $9000.00."))));
    }
}
#[test]
fn k1_collectible_and_other_long_term_categories_do_not_cancel_before_basis() {
    for collectible in [-10_000, 10_000] {
        let mut source = sole_qbi_loss(10_000, 0, 10_000);
        source.capital = Some(k1_collectible_capital(0, 0, collectible));
        source.amt = Some(amt_records(&source));
        assert_eq!(crate::kernel::ratio::dollars(source.capital_gains()), 10_000);
        let basis = source.basis_result().unwrap();
        assert_eq!(
            crate::kernel::ratio::dollars(basis.allowed[K1LossClass::Ordinary].get()),
            5_000
        );
        assert_eq!(
            crate::kernel::ratio::dollars(basis.suspended[K1LossClass::Ordinary].get()),
            5_000
        );
        assert_eq!(
            crate::kernel::ratio::dollars(
                basis.allowed[if collectible > 0 {
                    K1LossClass::LongTerm
                } else {
                    K1LossClass::Collectibles
                }]
                .get()
            ),
            5_000
        );
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 60_000);
        assert_eq!(value(&r, "capital_gain"), 5_000);
        assert_eq!(value(&r, "rate_28_gain"), if collectible > 0 { 5_000 } else { 0 });
        assert_eq!(value(&r, "qbi_loss_carryforward"), 5_000);
    }
}
#[test]
fn k1_prior_collectible_loss_histories_do_not_duplicate_long_term_totals() {
    let mut source = complete_ordinary_scorp(0, true);
    source.loans.clear();
    source.opening = Some(1_000_000);
    source.capital = Some(ScorpCapital {
        basis_long: 1_000_000,
        basis_collectibles: 600_000,
        risk_long: 1_000_000,
        risk_collectibles: 400_000,
        ..k1_capital(0, 0)
    });
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(basis.allowed, four_classes([0, 0, 4_000, 6_000]));
    assert_eq!(basis.suspended, LossAmounts::default());
    assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 0);
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        risk.ledger.risk_allowed.capital().map(crate::kernel::ratio::dollars),
        [0, 5_000, 5_000]
    );
    assert_eq!(
        risk.ledger.risk_suspended.capital().map(crate::kernel::ratio::dollars),
        [0, 5_000, 5_000]
    );
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 57_000);
    assert_eq!(value(&r, "capital_long_carryforward"), 7_000);
    assert!(r.review_notes.iter().any(|n| n.contains("regular total long-term portfolio capital including collectibles: net -$10000.00; basis-suspended loss $0.00; at-risk-suspended loss $10000.00.")));
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn k1_collectible_histories_validate_components_and_conserve_rounded_totals() {
    let mut capital = ScorpCapital {
        long: 149,
        collectibles: 50,
        basis_long: 149,
        basis_collectibles: 50,
        risk_long: 149,
        risk_collectibles: 50,
        ..k1_capital(0, 0)
    };
    assert!(capital.valid());
    assert_eq!(capital.gains(), [0, 99, 50]);
    assert_eq!(capital.losses(), [0, 99, 50]);
    assert_eq!(capital.prior_risk(), [0, 99, 50]);
    capital.basis_collectibles = 150;
    assert!(!capital.valid());
    capital.basis_collectibles = 50;
    capital.risk_collectibles = 150;
    assert!(!capital.valid());
}

#[test]
fn k1_collectible_rate_amount_recomputes_after_independent_amt_basis_limits() {
    let mut source = sole_qbi_loss(12_000, 0, 12_000);
    source.capital = Some(k1_collectible_capital(0, 0, 12_000));
    let mut amt = amt_records(&source);
    amt.capital = Some(k1_collectible_capital(0, 0, 6_000));
    source.amt = Some(amt);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "capital_gain"), 6_000);
    assert_eq!(value(&r, "amt_capital_gain"), 4_000);
    assert_eq!(value(&r, "rate_28_gain"), 6_000);
    assert_eq!(value(&r, "amt_rate_28_gain"), 4_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 6_000);
    amt_k1_note(&r, -4_000, -6_000);
    for (system, collectible, loss, suspended) in
        [("regular", 12_000, 6_000, 6_000), ("AMT", 6_000, 2_000, 4_000)]
    {
        assert!(r.review_notes.iter().any(|n| n.contains(&alloc::format!("{system} long-term collectibles (28%) portfolio capital: net ${collectible}.00; basis-suspended loss $0.00; at-risk-suspended loss $0.00."))));
        assert!(r.review_notes.iter().any(|n| n.contains(&alloc::format!("{system} long-term excluding collectibles portfolio capital: net -${loss}.00; basis-suspended loss ${suspended}.00; at-risk-suspended loss $0.00."))));
    }
}
#[test]
fn k1_portfolio_1250_sources_match_fund_rate_treatment_without_extra_income() {
    for (partnership, fund) in [(3_000, 0), (0, 3_000), (1_000, 2_000)] {
        let mut source = complete_ordinary_scorp(0, true);
        source.capital = Some(k1_portfolio_1250(5_000, partnership, fund));
        source.amt = Some(amt_records(&source));
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 0);
        let mut comparison = d.clone();
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        add(
            &mut comparison,
            income(FederalIncome::Dividend {
                data: dividend_source(FederalDividend {
                    unrecaptured_distributions: 300_000,
                    ..native_dividend(0, 0, 500_000)
                }),
            }),
        );
        let r = supported(&d);
        assert_eq!(r.return_lines, supported(&comparison).return_lines);
        assert_eq!(value(&r, "agi"), 305_000);
        assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 3_000);
        assert_eq!(value(&r, "amt_unrecaptured_section_1250_gain"), 3_000);
        assert_eq!(value(&r, "net_investment_income_tax"), 190);
        assert_eq!(value(&r, "qbi_deduction"), 0);
        assert!(r.review_notes.iter().any(|n| n.contains(&alloc::format!("regular portfolio unrecaptured section 1250 gain: partnership-interest source ${partnership}.00, fund/trust source ${fund}.00."))));
        assert_eq!(restart(d.clone()).draft, d);
    }
}
#[test]
fn k1_portfolio_1250_offsets_follow_collectible_then_25_percent_order_for_each_tax() {
    let mut source = complete_ordinary_scorp(0, true);
    source.capital = Some(ScorpCapital {
        short: -500_000,
        collectibles: 300_000,
        ..k1_portfolio_1250(10_000, 0, 4_000)
    });
    let mut amt = amt_records(&source);
    amt.capital.as_mut().unwrap().short = -200_000;
    amt.capital.as_mut().unwrap().unrecaptured_fund = 200_000;
    source.amt = Some(amt);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "capital_gain"), 5_000);
    assert_eq!(value(&r, "rate_28_gain"), 0);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 2_000);
    assert_eq!(value(&r, "amt_capital_gain"), 8_000);
    assert_eq!(value(&r, "amt_rate_28_gain"), 1_000);
    assert_eq!(value(&r, "amt_unrecaptured_section_1250_gain"), 2_000);
}
#[test]
fn k1_portfolio_1250_gain_funds_loss_limits_before_opposite_long_term_offset() {
    let mut source = sole_qbi_loss(12_000, 0, 12_000);
    source.capital = Some(k1_portfolio_1250(0, 12_000, 0));
    let mut amt = amt_records(&source);
    amt.capital = Some(k1_portfolio_1250(0, 6_000, 0));
    source.amt = Some(amt);
    let basis = source.basis_result().unwrap();
    assert_eq!(basis.allowed, four_classes([6_000, 0, 6_000, 0]));
    assert_eq!(basis.suspended, basis.allowed);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60_000);
    assert_eq!(value(&r, "capital_gain"), 6_000);
    assert_eq!(value(&r, "unrecaptured_section_1250_gain"), 6_000);
    assert_eq!(value(&r, "amt_capital_gain"), 4_000);
    assert_eq!(value(&r, "amt_unrecaptured_section_1250_gain"), 4_000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 6_000);
    amt_k1_note(&r, -4_000, -6_000);
}
#[test]
fn k1_portfolio_1250_amt_component_does_not_fall_back_to_regular_tax() {
    for (regular, amt) in [(4_000, 0), (0, 4_000), (1_000, 3_000)] {
        let mut source = complete_ordinary_scorp(0, true);
        source.capital = Some(k1_portfolio_1250(5_000, 0, regular));
        let mut alternate = amt_records(&source);
        alternate.capital = Some(k1_portfolio_1250(5_000, 0, amt));
        source.amt = Some(alternate);
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Scorp {
                data: crate::tests::support::builders::scorp_source(source),
            }),
        );
        let r = supported(&d);
        assert_eq!(value(&r, "capital_gain"), 5_000);
        assert_eq!(value(&r, "amt_capital_gain"), 5_000);
        assert_eq!(value(&r, "unrecaptured_section_1250_gain"), regular);
        assert_eq!(value(&r, "amt_unrecaptured_section_1250_gain"), amt);
    }
}
#[test]
fn k1_portfolio_1250_unelected_gain_is_excluded_from_investment_interest() {
    let mut source = complete_ordinary_scorp(0, true);
    source.capital =
        Some(ScorpCapital { short: 100_000, ..k1_portfolio_1250(5_000, 2_000, 1_000) });
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    investment_borrowing(&mut d, 6_000, 0, 0);
    let r = supported(&d);
    assert_eq!(value(&r, "investment_interest_deduction"), 1_000);
    assert_eq!(value(&r, "amt_investment_interest_deduction"), 1_000);
    assert_eq!(value(&r, "investment_interest_carryforward"), 5_000);
}
#[test]
fn k1_portfolio_1250_source_validation_remains_explicit() {
    let mut capital = k1_portfolio_1250(0, 1_000, 2_000);
    assert!(capital.valid()); // Offset by a loss in another long-term category.
    assert_eq!(capital.gains().map(crate::kernel::ratio::dollars), [0, 3_000, 0]);
    assert_eq!(capital.losses().map(crate::kernel::ratio::dollars), [0, 3_000, 0]);
    capital.unrecaptured_partnership = 0;
    capital.unrecaptured_fund = 200_000;
    assert!(capital.valid());
}
#[test]
fn k1_section1231_losses_share_basis_and_risk_with_ordinary_and_portfolio_losses() {
    let mut source = sole_qbi_loss(30_000, 50_000, 30_000);
    source.capital = Some(k1_capital(-10_000, -20_000));
    source.section1231 = Some(k1_section1231(-40_000, 0));
    source.risk.start = ScorpRiskStart::Detailed { capacity: 2_000_000, profit: 0, loss: 0 };
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(basis.allowed, five_classes([15_000, 5_000, 10_000, 0, 20_000]));
    assert_eq!(basis.suspended, basis.allowed);
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Ordinary].get()),
        6_000
    );
    assert_eq!(
        risk.ledger.risk_allowed.capital().map(crate::kernel::ratio::dollars),
        [2_000, 4_000, 0]
    );
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Section1231].get()),
        8_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_suspended[K1LossClass::Section1231].get()),
        12_000
    );
    assert_eq!(risk.year_end.map(crate::kernel::ratio::dollars), Some(0));
    let mut d = complete(FederalStatus::Single);
    let id = d.next_id;
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    unsupported(&d, "Section 1231 lookback");
    let r = estimate(&d);
    for system in ["regular", "AMT"] {
        assert!(r.review_notes.iter().any(|n| n.contains(&alloc::format!("{system} section 1231 worksheet: net before passive limits -$8000.00; basis-suspended loss $20000.00; at-risk-suspended loss $12000.00; prior passive-suspended loss $0.00; business-asset section 1250 component $0.00."))));
        let note = &r
            .review_notes
            .iter()
            .find(|n| n.contains(&alloc::format!("{system} section 1231 worksheet:")))
            .unwrap()
            .display;
        assert_eq!(
            note.key,
            if system == "AMT" {
                "NOTE_SCORP_AMT_SECTION1231"
            } else {
                "NOTE_SCORP_REGULAR_SECTION1231"
            }
        );
        assert_eq!(note.args[0], MessageArg::Number(id));
        assert_eq!(note.args[1], MessageArg::usd_cents(-800_000));
    }
    scorp_note(&r, id, "net loss absorbed by at-risk including nondeductible expenses", 20_000);
    assert!(r.tax.is_none());
    assert!(r.return_lines.is_empty());
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn k1_section1231_gain_funds_limits_without_adding_1250_component_twice() {
    let mut source = sole_qbi_loss(30_000, 0, 30_000);
    // A net K-1 can have a larger 1250 component when other business assets
    // have losses. The metadata does not increase basis or at-risk income.
    source.section1231 = Some(k1_section1231(20_000, 30_000));
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(basis.allowed, five_classes([20_000, 0, 0, 0, 0]));
    assert_eq!(basis.suspended, five_classes([10_000, 0, 0, 0, 0]));
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(risk.income.get()), 20_000);
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Ordinary].get()),
        20_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Section1231].get()),
        0
    );
    assert_eq!(risk.year_end.map(crate::kernel::ratio::dollars), Some(0));
    assert!(source.activity_result().is_err());
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.contains("regular section 1231 worksheet: net before passive limits $20000.00; basis-suspended loss $0.00; at-risk-suspended loss $0.00; prior passive-suspended loss $0.00; business-asset section 1250 component $30000.00.")));
    unsupported(&d, "Section 1231 lookback");
}
#[test]
fn k1_section1231_prior_suspensions_cross_only_their_remaining_limit() {
    let mut source = complete_ordinary_scorp(0, true);
    source.opening = Some(1_200_000);
    source.loans.clear();
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 };
    source.section1231 = Some(K1Section1231 {
        basis_loss: 1_200_000,
        risk_loss: 800_000,
        passive_loss: 1_000_000,
        ..k1_section1231(0, 0)
    });
    reconcile_qbi_tracking(&mut source);
    source.amt = Some(amt_records(&source));
    let basis = source.basis_result().unwrap();
    assert_eq!(basis.allowed, five_classes([0, 0, 0, 0, 12_000]));
    assert_eq!(basis.suspended, LossAmounts::default());
    assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 0);
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Section1231].get()),
        10_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(risk.ledger.risk_suspended[K1LossClass::Section1231].get()),
        10_000
    );
    assert_eq!(risk.ledger.risk_allowed.capital().map(crate::kernel::ratio::dollars), [0; 3]);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.contains("regular section 1231 worksheet: net before passive limits -$10000.00; basis-suspended loss $0.00; at-risk-suspended loss $10000.00; prior passive-suspended loss $10000.00;")));
    assert!(r.tax.is_none());
}
#[test]
fn k1_section1231_regular_and_amt_suspensions_remain_independent() {
    let mut source = complete_ordinary_scorp(0, true);
    source.loans.clear();
    source.opening = Some(2_000_000);
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_000_000, profit: 0, loss: 0 };
    source.section1231 = Some(k1_section1231(-30_000, 0));
    let mut amt = amt_records(&source);
    amt.stock = 1_200_000;
    amt.risk.start = ScorpRiskStart::Detailed { capacity: 800_000, profit: 0, loss: 0 };
    amt.section1231 = Some(k1_section1231(-20_000, 0));
    source.amt = Some(amt);
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.contains("regular section 1231 worksheet: net before passive limits -$10000.00; basis-suspended loss $10000.00; at-risk-suspended loss $10000.00;")));
    assert!(r.review_notes.iter().any(|n| n.contains("AMT section 1231 worksheet: net before passive limits -$8000.00; basis-suspended loss $8000.00; at-risk-suspended loss $4000.00;")));
    assert_eq!(restart(d.clone()).draft, d);
    unsupported(&d, "Section 1231 lookback");
}
#[test]
fn k1_section1231_missing_amt_record_is_not_an_implicit_zero() {
    let mut source = complete_ordinary_scorp(0, true);
    source.section1231 = Some(k1_section1231(1_000, 0));
    assert!(source.valid()); // Incomplete input can be retained for later work.
    assert!(source.amt.as_ref().unwrap().source(&source).is_err());
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    unsupported(&d, "S-corporation AMT");
}
#[test]
fn k1_confirmed_empty_section1231_record_preserves_existing_return_support() {
    let mut source = complete_ordinary_scorp(10_000, true);
    let mut baseline = complete(FederalStatus::Single);
    add(
        &mut baseline,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source.clone()),
        }),
    );
    source.section1231 = Some(k1_section1231(0, 0));
    source.property_qbi.complete = true;
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    assert_eq!(supported(&d).return_lines, supported(&baseline).return_lines);
    assert_eq!(restart(d.clone()).draft, d);
}
#[test]
fn k1_section1231_source_rejects_impossible_first_year_history_and_unknown_scope() {
    let mut source = sole_qbi_loss(0, 0, 0);
    source.risk.start = ScorpRiskStart::New;
    source.risk.deducted = 0;
    for (basis_loss, risk_loss, passive_loss) in [(100, 0, 0), (0, 100, 0), (0, 0, 100)] {
        source.section1231 =
            Some(K1Section1231 { basis_loss, risk_loss, passive_loss, ..k1_section1231(0, 0) });
        assert!(!source.valid());
    }
    source.section1231 = Some(k1_section1231(0, 0));
    assert!(source.valid());
}

#[test]
fn k1_property_and_ordinary_losses_share_pool_but_preserve_character() {
    let source = passive_property_scorp(-6_000, -4_000);
    let activity = source.pool_activity_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(activity.passive), -10_000);
    assert_eq!(
        activity.characters.unwrap().passive.map(crate::kernel::ratio::dollars),
        [-6_000, -4_000]
    );
    let (d, id) = passive_property_pool(source, 5_000);
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
    assert_eq!(pool.passive_loss, 5_000);
    assert_eq!(pool.k1_section1231, vec![property_result(id, -2_000, 2_000, 0)]);
    assert_eq!(pool.income, 2_000); // $5k other profit less $3k ordinary loss.
    assert_eq!(pool.scorp_business, 2_000);
    assert_eq!(pool.scorp_niit_adjustment, 0);
    assert!(r.review_notes.iter().any(|n| n.contains("ordinary net $-3000, ordinary suspended loss $3000; section 1231 net $-2000, section 1231 suspended loss $2000")));
    unsupported(&d, "Section 1231 lookback");
    let final_result = estimate(&d);
    assert!(final_result.tax.is_none());
    assert!(final_result.return_lines.is_empty());
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn k1_property_gain_releases_ordinary_loss_without_becoming_schedule_e_income() {
    let mut source = passive_property_scorp(-5_000, 3_000);
    source.section1231.as_mut().unwrap().passive_loss = 200_000;
    source.amt = Some(amt_records(&source));
    let (d, id) = passive_property_pool(source, 0);
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
    assert_eq!(pool.passive_loss, 4_000);
    assert_eq!(pool.income, -1_000);
    assert_eq!(pool.k1_section1231, vec![property_result(id, 1_000, 0, 0)]);
}

#[test]
fn k1_property_pool_uses_prior_loss_net_weights_not_gross_prior_balances() {
    let mut source = passive_property_scorp(4_000, -6_000);
    source.passive.loss = 600_000;
    source.amt = Some(amt_records(&source));
    let (d, id) = passive_property_pool(source, 4_000);
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
    assert_eq!(pool.passive_loss, 4_000);
    assert_eq!(pool.income, 3_000);
    assert_eq!(pool.k1_section1231, vec![property_result(id, -3_000, 3_000, 0)]);
}

#[test]
fn k1_property_passive_allocation_has_independent_amt_characters() {
    let mut source = passive_property_scorp(-6_000, -4_000);
    let amt = source.amt.as_mut().unwrap();
    amt.adjustment = 400_000;
    amt.section1231.as_mut().unwrap().net = -800_000;
    let (d, id) = passive_property_pool(source, 5_000);
    for (amt, ordinary, property, carry) in
        [(false, 2_000, -2_000, 2_000), (true, 4_000, -4_000, 4_000)]
    {
        let mut r = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals_for(&d, 200_000, &mut r, amt);
        assert_eq!(pool.passive_loss, 5_000);
        assert_eq!(pool.income, ordinary);
        assert_eq!(pool.k1_section1231, vec![property_result(id, property, carry, 0)]);
    }
    assert_eq!(restart(d.clone()).draft, d);
}

#[test]
fn k1_property_passive_inputs_reject_unresolved_sources() {
    let mut source = passive_property_scorp(-6_000, -4_000);
    source.passive.complete = false;
    assert!(source.pool_activity_result().is_err());
    source.passive.complete = true;
    source.participation.as_mut().unwrap().minutes = 30_001;
    assert!(source.pool_activity_result().is_ok());
    // The ordinary-only QBI adapter cannot quietly consume combined amounts.
    source.participation.as_mut().unwrap().minutes = 0;
    assert!(source.activity_result().is_err());
    assert!(source.pool_activity_result().is_ok());
}

#[test]
fn k1_property_passive_pool_receives_only_basis_and_risk_allowed_losses() {
    let mut source = passive_property_scorp(-30_000, -40_000);
    source.opening = Some(3_500_000);
    source.risk.start = ScorpRiskStart::Detailed { capacity: 1_400_000, profit: 0, loss: 0 };
    source.passive.loss = 300_000;
    source.section1231.as_mut().unwrap().passive_loss = 600_000;
    source.amt = Some(amt_records(&source));
    // Shared $35k basis allows $15k ordinary/$20k property; shared $14k
    // at-risk allows $6k/$8k. Prior passive losses bypass both earlier limits.
    assert_eq!(
        source
            .pool_activity_result()
            .unwrap()
            .characters
            .unwrap()
            .passive
            .map(crate::kernel::ratio::dollars),
        [-9_000, -14_000]
    );
    let (d, id) = passive_property_pool(source, 11_500);
    let mut r = estimate(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
    assert_eq!(pool.passive_loss, 11_500);
    assert_eq!(pool.income, 7_000);
    assert_eq!(pool.k1_section1231, vec![property_result(id, -7_000, 7_000, 0)]);
}

#[test]
fn k1_portfolio_gains_cannot_release_passive_property_or_ordinary_losses() {
    for gain in [0, 10_000] {
        let mut source = passive_property_scorp(-6_000, -4_000);
        source.capital = Some(k1_capital(0, gain));
        source.interest = gain * 100;
        source.amt = Some(amt_records(&source));
        let (d, id) = passive_property_pool(source, 5_000);
        let mut r = estimate(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals(&d, 200_000, &mut r);
        assert_eq!(pool.passive_loss, 5_000);
        assert_eq!(pool.income, 2_000);
        assert_eq!(pool.k1_section1231, vec![property_result(id, -2_000, 2_000, 0)]);
    }
}

#[test]
fn property_excess_loss_follows_basis_limits_and_joint_threshold() {
    let draft = property_excess_draft(400000, 300000, 300000, 200000, FederalStatus::Single);
    let r = supported(&draft);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 44000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 128000);
    assert_eq!(value(&r, "nol_carryforward"), 240000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 256000);
    let draft = property_excess_draft(800000, 700000, 900000, 800000, FederalStatus::Joint);
    let r = supported(&draft);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 288000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 188000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 512000);
    assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 512000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 512000);
    assert_eq!(r.tax, Some(0));
}

#[test]
fn property_loss_operating_profit_respects_basis_amt_and_sstb_independence() {
    let mut draft = property_loss_operating_profit_draft(200000, 100000);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    {
        let mut source_edit = edit_scorp(data);
        let data = &mut *source_edit;
        data.opening = Some(26000000); // $360k basis with operating profit; property loss limited.
        data.amt.as_mut().unwrap().section1231.as_mut().unwrap().net = -50000000;
    }
    let r = supported(&draft);
    assert_eq!(value(&r, "excess_business_loss_carryforward"), 4000);
    assert_eq!(value(&r, "amt_excess_business_loss_carryforward"), 144000);
    assert_eq!(value(&r, "qbi_loss_carryforward"), 78000);
    assert_eq!(value(&r, "nol_carryforward"), 200000);
    assert_eq!(value(&r, "amt_nol_carryforward"), 340000);
    assert_eq!(value(&r, "section1231_2026_next_lookback"), 356000);
    assert_eq!(value(&r, "amt_section1231_2026_next_lookback"), 356000);
    assert_eq!(r.tax, Some(0));
    let mut draft = property_loss_operating_profit_draft(400000, 100000);
    draft.income[0] = wage(1, Owner::Taxpayer, 51135000, 0);
    let FederalIncome::Scorp { data } = &mut draft.income[1].income else { unreachable!() };
    {
        let mut source_edit = edit_scorp(data);
        let data = &mut *source_edit;
        data.qbi_statement.as_mut().unwrap().service = true;
    }
    let r = supported(&draft);
    // Half-qualified profit $50k, loss $200k and deferred loss $22k.
    assert_eq!(value(&r, "qbi_loss_carryforward"), 128000);
    assert_eq!(value(&r, "taxable_income"), 239250);
    assert_eq!(value(&r, "regular_income_tax"), 53016);
    assert_eq!(return_line(&r, "16"), 5301600);
    // AMTI $311,350 less $90,100 exemption gives $221,250 * 26%
    // = $57,525 tentative tax, exceeding regular tax by $4,509.
    assert_eq!(value(&r, "alternative_minimum_tax"), 4509);
    assert_eq!(r.tax, Some(6032700));
}

#[test]
fn plan_gross_exceptions_use_calculated_account_basis_and_complete_return() {
    for (facts, relief, penalty) in [
        (gross_birth_claim(1000), 3200, 1280),
        (RetirementException::Levy { amount: 1000000 }, 8000, 800),
        (
            RetirementException::Emergency {
                balance: 10000000,
                history: EmergencyHistory::NONE,
                eligible: true,
            },
            800,
            1520,
        ),
        (
            RetirementException::DomesticAbuse {
                plan: "Plan A".into(),
                event: 20260101,
                balance: 10000000,
                prior_used: 0,
                prior_plan_used: 0,
                eligible: true,
            },
            8400,
            760,
        ),
    ] {
        let d = plan_gross_exception(facts, 20000);
        let r = supported(&d);
        assert_eq!(return_line(&r, "5a"), 2000000);
        assert_eq!(return_line(&r, "5b"), 1600000);
        assert_eq!(value(&r, "pension_2_basis_recovery"), 4000);
        assert_eq!(value(&r, "pension_2_remaining_basis"), 16000);
        assert_eq!(value(&r, "retirement_2_qualified_taxable"), relief);
        assert_eq!(value(&r, "early_distribution_tax"), penalty);
        assert_eq!(r.tax, Some((21374 + penalty) * 100));
        assert_eq!(supported(&restart(d).draft).tax, r.tax);
    }
}

#[test]
fn qualified_full_discharge_uses_basis_ratio_for_gross_limited_exceptions() {
    for (facts, relief, penalty) in [
        (gross_birth_claim(1000), 3200, 1280),
        (RetirementException::Levy { amount: 1000000 }, 8000, 800),
        (
            RetirementException::Emergency {
                balance: 2000000,
                history: EmergencyHistory::NONE,
                eligible: true,
            },
            800,
            1520,
        ),
        (
            RetirementException::DomesticAbuse {
                plan: "Plan A".into(),
                event: 20260101,
                balance: 2000000,
                prior_used: 0,
                prior_plan_used: 0,
                eligible: true,
            },
            8000,
            800,
        ),
    ] {
        let mut d = plan_gross_exception(facts, 20000);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
        {
            let mut data = edit_retirement(data);
            data.taxable = Some(RetirementTaxable::FullDischarge {
                data: crate::ContractDischarge { investment: 400000, eligible: true },
            });
        }
        let r = supported(&d);
        assert_eq!(return_line(&r, "5a"), 2000000);
        assert_eq!(return_line(&r, "5b"), 1600000);
        assert_eq!(value(&r, "annuity_2_basis_recovery"), 4000);
        assert_eq!(value(&r, "retirement_2_qualified_taxable"), relief);
        assert_eq!(value(&r, "early_distribution_tax"), penalty);
        assert_eq!(value(&r, "agi"), 136000);
        assert_eq!(r.tax, Some((21374 + penalty) * 100));
        assert_eq!(supported(&restart(d).draft).tax, r.tax);
    }
}

#[test]
fn qualified_full_discharge_exception_preserves_full_basis_and_surrender_loss() {
    for investment in [20000, 24000] {
        let mut d = plan_gross_exception(gross_birth_claim(0), 20000);
        let FederalIncome::Retirement { data } = &mut d.income[1].income else { panic!() };
        {
            let mut data = edit_retirement(data);
            data.taxable = Some(RetirementTaxable::FullDischarge {
                data: crate::ContractDischarge { investment: investment * 100, eligible: true },
            });
        }
        let r = supported(&d);
        assert_eq!(return_line(&r, "5b"), 0);
        assert_eq!(value(&r, "retirement_2_qualified_gross"), 5000);
        assert_eq!(value(&r, "retirement_2_qualified_taxable"), 0);
        assert_eq!(value(&r, "annuity_2_surrender_loss"), investment - 20000);
        assert_eq!(value(&r, "agi"), 120000);
        assert_eq!(r.tax, Some(1757000));
    }
}

#[test]
fn foreign_investment_assets_use_average_basis_not_income_or_ending_balance() {
    let mut d = foreign_asset_return();
    let r = supported(&d);
    // Pub514's 40/60 asset ratio gives800/1,200 of2,000 interest. Here
    // gross-income allocation would incorrectly assign only120 abroad.
    assert_eq!(value(&r, "foreign_investment_regular_group_4"), 1200);
    assert_eq!(value(&r, "foreign_investment_regular_group_7"), 800);
    assert_eq!(value(&r, "foreign_investment_amt_group_4"), 1200);
    assert_eq!(form_1116_any(&r, false, "_worldwide_income"), 98000);
    assert_eq!(value(&r, "foreign_tax_credit"), 600);
    assert_eq!(form_1116_value(&r, "foreign_amt_passive_net_foreign_income"), 4800);
    assert_eq!(r.tax, Some(1567800));
    assert_eq!(supported(&restart(d.clone()).draft), r);
    if let ForeignTaxClaim::Ordinary {
        investment_assets: ForeignInvestmentAssets::TaxBook { assets, .. },
        ..
    } = &mut d.credits.as_mut().unwrap().foreign_tax
    {
        assets.reverse();
    }
    assert_eq!(supported(&d), r);
}

#[test]
fn foreign_mixed_asset_uses_gross_yield_to_characterize_basis() {
    let mut d = foreign_asset_return();
    if let ForeignTaxClaim::Ordinary { investment_assets, .. } =
        &mut d.credits.as_mut().unwrap().foreign_tax
    {
        *investment_assets = ForeignInvestmentAssets::TaxBook {
            assets: vec![foreign_asset(
                "Mixed portfolio asset",
                InvestmentAssetSource::Mixed {
                    domestic: 400000,
                    foreign: vec![InvestmentAssetYield {
                        source: "Foreign deposits".into(),
                        income: 600000,
                    }],
                },
                100000,
                100000,
            )],
        };
    }
    let r = supported(&d);
    assert_eq!(value(&r, "foreign_investment_regular_group_4"), 1200);
    assert_eq!(value(&r, "foreign_investment_regular_group_7"), 800);
    assert_eq!(value(&r, "foreign_investment_amt_group_4"), 1200);
    assert_eq!(r.tax, Some(1567800));
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn foreign_daily_asset_weights_partial_year_holdings_and_separate_amt_basis() {
    for changes in [
        vec![daily_basis(20260101, 60000, 120000), daily_basis(20260315, 0, 0)],
        vec![daily_basis(20260101, 0, 0), daily_basis(20261020, 60000, 120000)],
    ] {
        let mut d = foreign_asset_return();
        if let ForeignTaxClaim::Ordinary {
            sources,
            investment_assets: ForeignInvestmentAssets::TaxBook { assets, .. },
            ..
        } = &mut d.credits.as_mut().unwrap().foreign_tax
        {
            // Domestic basis40,000 all year; foreign basis60,000 for73days.
            assets[0].basis = InvestmentAssetBasis::Average {
                beginning: 4000000,
                ending: 4000000,
                amt_beginning: 4000000,
                amt_ending: 4000000,
            };
            assets[1].basis = InvestmentAssetBasis::Daily { changes };
            sources[0].tax = 200000;
        }
        let r = supported(&d);
        // Regular foreign average12,000; 2,000*12/52=462. AMT
        // foreign average24,000; 2,000*24/64=750. Endpoint-only
        // averaging would incorrectly assign857regular interest abroad.
        assert_eq!(value(&r, "foreign_investment_regular_group_4"), 462);
        assert_eq!(value(&r, "foreign_investment_regular_group_7"), 1538);
        assert_eq!(value(&r, "foreign_investment_amt_group_4"), 750);
        assert_eq!(value(&r, "foreign_tax_credit"), 920);
        assert_eq!(r.tax, Some(1535800));
        assert_eq!(supported(&restart(d).draft), r);
    }
}

#[test]
fn foreign_daily_asset_rejects_incomplete_or_unordered_basis_histories() {
    for changes in [
        vec![daily_basis(20260102, 60000, 60000)],
        vec![daily_basis(20260101, 60000, 60000), daily_basis(20260101, 0, 0)],
        vec![
            daily_basis(20260101, 60000, 60000),
            daily_basis(20260301, 0, 0),
            daily_basis(20260201, 60000, 60000),
        ],
    ] {
        let mut d = foreign_asset_return();
        if let ForeignTaxClaim::Ordinary {
            investment_assets: ForeignInvestmentAssets::TaxBook { assets, .. },
            ..
        } = &mut d.credits.as_mut().unwrap().foreign_tax
        {
            assets[1].basis = InvestmentAssetBasis::Daily { changes };
        }
        assert!(!d.valid_stored());
        let r = estimate(&d);
        assert_eq!(r.state, ResultState::Incomplete);
        assert!(r.tax.is_none() && r.lines.is_empty());
        assert!(r.issues.iter().any(|i| i.section == "Saved return"));
    }
}

#[test]
fn reported_stock_compensation_and_established_sale_basis_are_not_taxed_twice() {
    // NSO: $10,000 exercise cost + $20,000 already taxed compensation.
    // RSU: $0 purchase cost + $30,000 already taxed compensation.
    // Both have established $30,000 sale basis; total W-2 wages are $80,000.
    for proceeds in [35000, 25000] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, 8000000, 0);
        add(&mut d, income(ordinary_investment_sale(proceeds, 30000, GainTerm::Short)));
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), if proceeds == 35000 { 85000 } else { 77000 });
        assert_eq!(value(&r, "capital_gain"), if proceeds == 35000 { 5000 } else { -3000 });
        assert_eq!(
            value(&r, "capital_loss_carryforward"),
            if proceeds == 35000 { 0 } else { 2000 }
        );
        assert_eq!(supported(&restart(d.clone()).draft), r);
        let FederalIncome::Sale { data } = &mut d.income[1].income else { unreachable!() };
        data.special = crate::SaleSpecial::Another;
        assert!(noted(&supported(&d), "WARN_SALE_OTHER_LEFT_OUT"));
    }
}

#[test]
fn employee_stock_source_records_derive_compensation_basis_and_holding_period() {
    for paid in [0, 1000000] {
        for acquired in [20250101, 20260101] {
            for owner in [Owner::Taxpayer, Owner::Spouse] {
                let mut d = empty_return(FederalStatus::Joint);
                add(&mut d, wage(0, owner, 8000000, 0));
                let lot = EmployeeStockSale {
                    acquired,
                    sold: 20260701,
                    paid,
                    market_value: 3000000,
                    fees: 10000,
                    proceeds: 3500000,
                    wash_sale: 0,
                    withholding: 0,
                    reported_basis: None,
                    reporting: crate::BasisReporting::ReportedToIrs,
                    special: false,
                };
                let mut entry = income(FederalIncome::EmployeeStockSale { data: lot });
                entry.owner = crate::DocumentOwner::from(owner);
                add(&mut d, entry);
                let r = supported(&d);
                assert_eq!(
                    value(&r, &alloc::format!("employee_stock_{}_compensation", d.income[1].id)),
                    (3000000 - paid) / 100
                );
                assert_eq!(
                    value(&r, &alloc::format!("employee_stock_{}_basis", d.income[1].id)),
                    30100
                );
                assert_eq!(value(&r, "capital_gain"), 4900);
                assert_eq!(value(&r, "agi"), 84900);
                assert_eq!(supported(&restart(d.clone()).draft), r);
                let FederalIncome::EmployeeStockSale { data } = &d.income[1].income else {
                    unreachable!()
                };
                let sale = data.sale();
                assert_eq!(
                    sale.term,
                    if acquired == 20250101 { GainTerm::Long } else { GainTerm::Short }
                );
                d.income[1].income = FederalIncome::Sale { data: sale_source(sale) };
                let ordinary = supported(&d);
                assert_eq!(r.tax, ordinary.tax);
                assert_eq!(value(&r, "agi"), value(&ordinary, "agi"));
            }
        }
    }
}

#[test]
fn fractional_iso_sales_preserve_compensation_and_separate_amt_basis() {
    for exercised in [20240313, 20250601, 20260101] {
        let mut lot = qualifying_iso_sale();
        lot.exercised = exercised;
        if exercised != 20240313 {
            lot.granted = 20250101;
        }
        lot.shares = 0.125;
        lot.proceeds = 7500;
        assert!(lot.valid());
        assert_eq!(lot.compensation(), if exercised == 20240313 { 0 } else { 6250 });
        assert_eq!(lot.sale().basis_amount(), if exercised == 20240313 { 1250 } else { 7500 });
        assert_eq!(
            lot.sale().amt.different().unwrap().basis,
            if exercised == 20260101 { 7500 } else { 12500 }
        );
        assert_eq!(
            lot.amt_compensation_adjustment(),
            if exercised == 20250601 { -6250 } else { 0 }
        );
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::IsoSale { data: lot.clone() }));
        let r = supported(&d);
        assert_eq!(supported(&restart(d.clone()).draft), r);
        // Equivalent whole-share lot: identical source totals and complete return.
        lot.shares = 1.0;
        lot.exercise_price = 1250;
        lot.market_price = 12500;
        d.income[1].income = FederalIncome::IsoSale { data: lot };
        assert_eq!(supported(&d), r);
    }
    let mut lot = qualifying_iso_sale();
    {
        let quantity = 1000000.000001;
        lot.shares = quantity;
        assert!(!lot.valid());
    }
}

#[test]
fn iso_option_purchase_cost_reduces_spread_and_enters_sale_basis() {
    let mut d = complete(FederalStatus::Single);
    let mut exercise = iso_exercise("Synthetic paid option", 2000.0, 1000, 11000);
    exercise.more.as_mut().unwrap().option_cost = 1000000;
    d.screening.as_mut().unwrap().iso_exercises = vec![exercise];
    let r = supported(&d);
    assert_eq!(value(&r, "agi"), 60000);
    assert_eq!(value(&r, "alternative_minimum_taxable_income"), 250000);
    assert_eq!(supported(&restart(d.clone()).draft), r);
    d.screening.as_mut().unwrap().iso_exercises[0].more.as_mut().unwrap().option_cost = 21000000;
    assert_eq!(value(&supported(&d), "alternative_minimum_taxable_income"), 60000);
    for early in [false, true] {
        let mut lot = if early { disqualifying_iso_sale() } else { qualifying_iso_sale() };
        lot.option_cost = 100000;
        assert!(lot.valid());
        assert_eq!(lot.compensation(), if early { 4900000 } else { 0 });
        assert_eq!(lot.sale().basis_amount(), if early { 6000000 } else { 1100000 });
        assert_eq!(lot.sale().amt.different().unwrap().basis, 10000000);
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::IsoSale { data: lot.clone() }));
        let r = supported(&d);
        assert_eq!(value(&r, "agi"), 109000);
        assert_eq!(supported(&restart(d).draft), r);
        lot.option_cost = 10000001;
        lot.proceeds = 12000000;
        assert!(lot.valid());
        assert_eq!(lot.compensation(), 0);
        assert_eq!(lot.sale().basis_amount(), 11000001);
        assert_eq!(lot.sale().amt.different().unwrap().basis, 11000001);
    }
}

#[test]
fn next_year_employer_cash_rollover_keeps_basis_out_of_current_ira_distributions() {
    let mut d = incoming_plan_rollover();
    let FederalIncome::Retirement { data } = &mut d.income[1].income else { unreachable!() };
    {
        let mut data = edit_retirement(data);
        let Some(RetirementTreatment::Rollover { destination, .. }) =
            &mut data.ordinary_distribution
        else {
            unreachable!()
        };
        *destination = RolloverDestination::IraNextYear;
    }
    // $4k basis reaches the IRA in 2027, not during this $5k 2026 withdrawal.
    add(&mut d, basis_distribution(Owner::Taxpayer, 5000, RetirementTreatment::Ordinary));
    let r = supported(&d);
    assert_eq!(return_line(&r, "5b"), 0);
    assert_eq!(return_line(&r, "4b"), 500000);
    assert_eq!(value(&r, "pension_2_next_year_ira_basis"), 4000);
    assert_eq!(value(&r, "taxpayer_ira_incoming_plan_basis"), 0);
    assert_eq!(value(&r, "taxpayer_ira_outstanding_rollover"), 0);
    assert_eq!(value(&r, "taxpayer_ira_basis_carryforward"), 0);
    assert_eq!(supported(&restart(d).draft), r);
}

#[test]
fn partner_source_basis_uses_tax_records_not_book_capital() {
    let mut source = PartnerBasisRecord {
        opening: Some(1000000),
        contributions: 0,
        opening_liabilities: 500000,
        ending_liabilities: 1000000,
        assumed: 0,
        transferred: 0,
        distributions: 0,
        prior_loss: 200000,
        prior_nondeductible: 30000,
        complete: true,
    };
    let result = source.ordinary_result(-1800000, 100000, 20000).unwrap();
    assert_eq!(
        result.allowed,
        LossAmounts::of(&[(
            K1LossClass::Ordinary,
            crate::kernel::carry::Amount::new(15500 * 100).unwrap()
        )])
    );
    assert_eq!(
        result.suspended,
        LossAmounts::of(&[(
            K1LossClass::Ordinary,
            crate::kernel::carry::Amount::new(4500 * 100).unwrap()
        )])
    );
    assert_eq!(crate::kernel::ratio::dollars(result.nondeductible_used.get()), 500);
    assert_eq!(crate::kernel::ratio::dollars(result.ending.get()), 0);
    let restored = PartnerBasisRecord::from_avro(&source.to_avro()).unwrap();
    assert_eq!(restored, source);
    assert_eq!(restored.ordinary_result(-1800000, 100000, 20000).unwrap(), result);
    source.complete = false;
    assert!(source.ordinary_result(0, 0, 0).is_err());
}

#[test]
fn limited_partner_participation_uses_only_the_three_permitted_tests() {
    let mut general = participation_facts();
    general.minutes = 6000;
    general.substantially_all = true;
    assert_eq!(
        PartnerParticipation::General { facts: general }.material(),
        crate::participation::Participation::Yes
    );
    let mut limited = PartnerParticipation::Limited {
        minutes: 6000,
        prior_ten: 0,
        personal_service: false,
        service_years: 0,
    };
    assert_eq!(limited.material(), crate::participation::Participation::No);
    if let PartnerParticipation::Limited { minutes, .. } = &mut limited {
        *minutes = 30000;
    }
    assert_eq!(limited.material(), crate::participation::Participation::No); // Exactly 500 hours is insufficient.
    if let PartnerParticipation::Limited { minutes, .. } = &mut limited {
        *minutes = 30001;
    }
    assert_eq!(limited.material(), crate::participation::Participation::Yes);
    assert_eq!(PartnerParticipation::from_avro(&limited.to_avro()).unwrap(), limited);
    if let PartnerParticipation::Limited { minutes, prior_ten, .. } = &mut limited {
        *minutes = 0;
        *prior_ten = 5;
    }
    assert_eq!(limited.material(), crate::participation::Participation::Yes);
    if let PartnerParticipation::Limited { prior_ten, personal_service, service_years, .. } =
        &mut limited
    {
        *prior_ten = 0;
        *personal_service = true;
        *service_years = 3;
    }
    assert_eq!(limited.material(), crate::participation::Participation::Yes);
    // 2026-10-16 D-2026-10-15-13: prior personal-service years have
    // no arbitrary century ceiling; the statutory test compares with three.
    if let PartnerParticipation::Limited { service_years, .. } = &mut limited {
        *service_years = u32::MAX;
    }
    assert_eq!(limited.material(), crate::participation::Participation::Yes);
    // Two earlier personal-service years do not establish it.
    if let PartnerParticipation::Limited { service_years, .. } = &mut limited {
        *service_years = 2;
    }
    assert_eq!(limited.material(), crate::participation::Participation::No);
    // Not a personal-service activity: the five-of-ten-year count decides.
    if let PartnerParticipation::Limited { personal_service, service_years, .. } = &mut limited {
        *personal_service = false;
        *service_years = 0;
    }
    assert_eq!(limited.material(), crate::participation::Participation::No);
}

#[test]
fn partner_nonrecourse_basis_does_not_create_at_risk_capacity() {
    let mut basis = partner_basis_record();
    basis.contributions = 200000;
    basis.ending_liabilities = 800000;
    let mut risk = partner_risk_record();
    let result = risk.calculate(&basis, -1000000, 0, 0).unwrap();
    assert_eq!(
        result.basis.allowed,
        LossAmounts::of(&[(
            K1LossClass::Ordinary,
            crate::kernel::carry::Amount::new(10000 * 100).unwrap()
        )])
    );
    assert_eq!(
        result.basis.suspended,
        LossAmounts::of(&[(K1LossClass::Ordinary, crate::kernel::carry::Amount::new(0).unwrap())])
    );
    assert_eq!(
        crate::kernel::ratio::dollars(result.ledger.risk_allowed[K1LossClass::Ordinary].get()),
        2000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(result.ledger.risk_suspended[K1LossClass::Ordinary].get()),
        8000
    );
    assert_eq!(result.risk_end.map(crate::kernel::ratio::dollars), Some(0));
    risk.ending_debt = 800000; // Established personal risk, not merely K-1 debt.
    let result = risk.calculate(&basis, -1000000, 0, 0).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(result.ledger.risk_allowed[K1LossClass::Ordinary].get()),
        10000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(result.ledger.risk_suspended[K1LossClass::Ordinary].get()),
        0
    );
    risk.ending_debt = 800001;
    assert!(risk.calculate(&basis, -1000000, 0, 0).is_err());
    risk.ending_debt = 0;
    let restored = PartnerRiskRecord::from_avro(&risk.to_avro()).unwrap();
    assert_eq!(restored, risk);
    assert_eq!(
        crate::kernel::ratio::dollars(
            restored.calculate(&basis, -1000000, 0, 0).unwrap().ledger.risk_allowed
                [K1LossClass::Ordinary]
                .get()
        ),
        2000
    );
    risk.start = ScorpRiskStart::Unknown;
    assert!(risk.calculate(&basis, -1000000, 0, 0).is_err());
}

#[test]
fn partner_sequential_limits_keep_carries_separate_and_recapture_negative_risk() {
    let mut basis = partner_basis_record();
    basis.opening = Some(50000);
    let mut risk = partner_risk_record();
    risk.start = ScorpRiskStart::Detailed { capacity: 25000, profit: 0, loss: 0 };
    let first = risk.calculate(&basis, -100000, 0, 20000).unwrap();
    assert_eq!(
        first.basis.allowed,
        LossAmounts::of(&[(
            K1LossClass::Ordinary,
            crate::kernel::carry::Amount::new(300 * 100).unwrap()
        )])
    );
    assert_eq!(
        first.basis.suspended,
        LossAmounts::of(&[(
            K1LossClass::Ordinary,
            crate::kernel::carry::Amount::new(700 * 100).unwrap()
        )])
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(first.ledger.risk_allowed[K1LossClass::Ordinary].get()),
            crate::kernel::ratio::dollars(first.ledger.risk_suspended[K1LossClass::Ordinary].get())
        ),
        (150, 150)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(first.nondeductible_used.get()),
            crate::kernel::ratio::dollars(first.nondeductible_suspended.get())
        ),
        (100, 100)
    );
    // An illustrative following-year source projection: risk carries do NOT
    // consume outside basis again. The engine itself does not advance tax years.
    basis.opening = Some(first.basis.ending.get());
    basis.contributions = 100000;
    basis.prior_loss = first.basis.suspended[K1LossClass::Ordinary].get();
    risk.start = ScorpRiskStart::Detailed { capacity: 25000, profit: 0, loss: 25000 };
    risk.prior_loss = first.ledger.risk_suspended[K1LossClass::Ordinary].get();
    risk.prior_nondeductible = first.nondeductible_suspended.get();
    let next = risk.calculate(&basis, 0, 0, 0).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(next.basis.ending.get()), 300);
    assert_eq!(
        crate::kernel::ratio::dollars(next.ledger.risk_allowed[K1LossClass::Ordinary].get()),
        850
    );
    assert_eq!(crate::kernel::ratio::dollars(next.nondeductible_used.get()), 100);
    assert_eq!(next.risk_end.map(crate::kernel::ratio::dollars), Some(50));
    basis = partner_basis_record();
    basis.opening = Some(200000);
    basis.distributions = 100000;
    risk = partner_risk_record();
    risk.start = ScorpRiskStart::Detailed { capacity: 50000, profit: 0, loss: 0 };
    risk.deducted = 40000;
    risk.recaptured = 10000;
    let result = risk.calculate(&basis, 20000, 0, 0).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(result.basis.ending.get()), 1200);
    assert_eq!(result.risk_end.map(crate::kernel::ratio::dollars), Some(-300));
    assert_eq!(crate::kernel::ratio::dollars(result.recapture.income.get()), 300);
    assert_eq!(crate::kernel::ratio::dollars(result.recapture.next_year_deduction.get()), 300);
    risk.deducted = 20000;
    assert_eq!(
        crate::kernel::ratio::dollars(
            risk.calculate(&basis, 20000, 0, 0).unwrap().recapture.income.get()
        ),
        100
    );
}

#[test]
fn partner_former_passive_loss_releases_against_own_income_before_pool() {
    let mut record = partner_activity_record();
    let mut facts = participation_facts();
    facts.minutes = 30001;
    let active = PartnerParticipation::General { facts };
    let result = record.pool_result(active.material()).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(result.prior_released), 1000);
    assert_eq!(crate::kernel::ratio::dollars(result.nonpassive), 0);
    assert_eq!(crate::kernel::ratio::dollars(result.passive), -500);
    // Partners do not receive the real-estate active-participation allowance.
    let suspended = crate::tests::support::loss_limits::passive_suspensions(
        &[crate::tests::support::loss_limits::PassiveActivity {
            key: (),
            net: crate::kernel::ratio::dollars(result.passive),
            rental_allowance: false,
            ineligible_prior: 0,
        }],
        200,
        25000,
    )
    .unwrap();
    assert_eq!(suspended, [300]);
    record.ordinary = -100000;
    let result = record.pool_result(active.material()).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(result.nonpassive),
            crate::kernel::ratio::dollars(result.passive),
            crate::kernel::ratio::dollars(result.prior_released)
        ),
        (-1000, -1500, 0)
    );
    let passive = PartnerParticipation::General { facts: participation_facts() };
    let result = record.pool_result(passive.material()).unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(result.nonpassive),
            crate::kernel::ratio::dollars(result.passive)
        ),
        (0, -2500)
    );
    record.risk.start = ScorpRiskStart::Unknown;
    assert!(record.pool_result(passive.material()).is_err()); // No zero substitution.
}

#[test]
fn partnership_input_saves_and_restarts_without_silently_omitting_income() {
    let regular = partner_activity_record();
    let mut alternative = regular.clone();
    alternative.ordinary = 60000;
    alternative.passive_loss = 120000;
    let data = PartnerK1 {
        source: None,
        research_credit_line: None,
        amt_record_choice: None,

        amt_section179_income_carry: None,
        reporting_period: None,
        section179_sales: alloc::vec::Vec::new(),
        qbi_aggregation_group: None,
        section179_carryover_sold: 0,
        schedule_k3: None,
        rental_facts: crate::K1RentalFacts::default(),
        income_source: None,
        gross_personal_services: None,
        guaranteed_services: 0,
        guaranteed_capital: 0,
        health_guaranteed: 0,
        health_distributions: 0,
        regular,
        alternative: Some(alternative),
        participation: Some(PartnerParticipation::General { facts: participation_facts() }),
        trade_kind: ScorpTradeKind::Ordinary,
        limited_for_se: Some(false),
        reported_se: Some(0),
        se_history: PartnerSeHistory { basis: 0, risk: 0, passive: 0 },
        se_gross: None,
        qbi: None,
        qbi_records: Default::default(),
        health_insurance: BusinessHealthInsurance::None,
        pensions: 0,
        retirement: PartnerRetirement::None,
        fuel_tax_credit: None,
        prior_fuel_credit: None,
        final_k1: false,
        income: Vec::new(),
        deductions: Vec::new(),
        deferred_compensation_tax: 0,
        crp_payments: 0,
        interest_term: None,
        disposition: Some(crate::K1Disposition::Kept),
        sale: crate::PartnerSaleParts::default(),
        special: Vec::new(),
        publicly_traded: false,
        separate: None,
        section179: 0,
        section179_carryover: 0,
        credits: Vec::new(),
        unentered: vec![],
        investment_property: Vec::new(),
        investment_confirmed: None,
        housing_confirmed: None,
    };
    assert!(data.valid());
    assert_eq!(
        crate::kernel::ratio::dollars(
            data.regular.pool_result(data.material_for_losses()).unwrap().passive
        ),
        -500
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            data.alternative
                .as_ref()
                .unwrap()
                .pool_result(data.material_for_losses())
                .unwrap()
                .passive
        ),
        -600
    );
    // The M1 income page offers W-2, 1099-INT and 1099-DIV only; the K-1 is
    // built directly on the draft until its milestone adds it to a page.
    let mut saved = complete(FederalStatus::Single);
    add(&mut saved, income(FederalIncome::Partnership { data }));
    saved = crate::tests::support::builders::known_fact_partnership_draft(saved);
    assert!(crate::federal_rules::validate_income(&saved.income[1]).is_ok());
    let restored = restart(saved.clone());
    assert_eq!(restored.draft, saved);
    let mut reviewed = restored.draft;
    if let FederalIncome::Partnership { data } = &mut reviewed.income[1].income {
        let assigned = data.source.as_ref().unwrap().assigned;
        let mut paper =
            crate::K1PartnershipDocument::from_record("Synthetic partnership", data).unwrap();
        let mut page = paper.calculation_page();
        page.special = vec![PartnerSpecial::PropertyContributed];
        paper.replace_personal_page(&page);
        *data = paper.record();
        data.source.as_mut().unwrap().assigned = assigned;
        assert!(data.valid()); // Unsupported basis records can be saved.
    }
    // Special basis records leave the K-1 out with a notice.
    let r = supported(&reviewed);
    assert!(noted(&r, "WARN_PARTNERSHIP_K1_LEFT_OUT"), "{r:?}");
}

#[test]
fn ordinary_partner_profit_reaches_complete_return_and_survives_restart() {
    let mut draft = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(20_000, true);
    partner.qbi = Some(partner_qbi_statement(20_000));
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));

    let result = supported(&draft);
    assert_eq!(return_line(&result, "8"), 2_000_000);
    assert_eq!(return_line(&result, "11a / 11b"), 8_000_000);
    assert_eq!(return_line(&result, "13b"), 400_000);
    assert_eq!(return_line(&result, "15"), 5_990_000);
    assert_eq!(return_line(&result, "16"), 789_600);
    assert_eq!(return_line(&result, "23"), 0);
    assert_eq!(result.balance, Some(139_600));
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn partnership_k1_fuel_gallons_create_partner_payment_without_repeating_income() {
    let mut draft = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(20_000, true);
    partner.qbi = Some(partner_qbi_statement(20_000));
    partner.fuel_tax_credit = Some(PartnerFuelTaxCredit {
        fuels: FederalFuelLines {
            gasoline_off_highway: Some(fuel_use(100_000, 20_000)),
            ..empty_fuel_lines()
        }
        .into(),
        statement_complete: true,
        farm_activity: false,
    });
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));

    let report = supported(&draft);
    assert_eq!(return_line(&report, "8"), 2_000_000);
    assert_eq!(return_line(&report, "11a / 11b"), 8_000_000);
    assert_eq!(return_line(&report, "31"), 1_800);
    assert_eq!(report.balance, Some(137_800));
    assert_eq!(line_amount(&report, "partnership_2_fuel_1a"), 1_830);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    // No: no credit is claimed from this K-1, with its scope notice.
    data.fuel_tax_credit.as_mut().unwrap().statement_complete = false;
    let r = supported(&draft);
    assert!(noted(&r, "WARN_PARTNER_FUEL_CREDIT_LEFT_OUT"), "{r:?}");
}

#[test]
fn estate_trust_portfolio_k1_preserves_parent_and_special_gain_boxes() {
    let k1 = FederalEstateTrustPortfolioK1 {
        interest: 100_000,
        ordinary_dividends: 200_000,
        qualified_dividends: 50_000,
        short_gain: 30_000,
        long_gain: 70_000,
        rate_28_gain: 10_000,
        unrecaptured_gain: 20_000,
        backup_withholding: 12_345,
        ..empty_estate_portfolio()
    };
    let mut draft = complete(FederalStatus::Single);
    let baseline = supported(&draft);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(k1.clone()),
        }),
    );
    let report = supported(&draft);
    assert_eq!(return_line(&report, "2b"), return_line(&baseline, "2b") + 100_000);
    assert_eq!(return_line(&report, "3b"), return_line(&baseline, "3b") + 200_000);
    assert_eq!(return_line(&report, "3a"), return_line(&baseline, "3a") + 50_000);
    assert_eq!(return_line(&report, "7a"), return_line(&baseline, "7a") + 100_000);
    assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b") + 400_000);
    assert_eq!(value(&report, "rate_28_gain"), 100);
    assert_eq!(value(&report, "unrecaptured_section_1250_gain"), 200);
    let mut no_backup = draft.clone();
    let mut data = estate_trust_code_d_data(&mut no_backup);
    data.backup_withholding = 0;
    drop(data);
    let without_backup = supported(&no_backup);
    assert_eq!(report.tax, without_backup.tax);
    assert_eq!(return_line(&report, "25d"), return_line(&without_backup, "25d") + 12_300);
    assert_eq!(return_line(&report, "25c"), return_line(&without_backup, "25c") + 12_300);
    assert_eq!(return_line(&report, "33"), return_line(&without_backup, "33") + 12_300);
    assert_eq!(value(&report, "estate_trust_2_backup_withholding"), 123);
    assert_eq!(line_amount(&report, "estate_trust_2_backup_withholding"), 12_345);
    assert!(report.review_notes.iter().any(|note| note.contains("retain the K-1 for attachment")));
    assert_eq!(crate::interview::document_amount(&draft.income[1].income, false), Some(400_000));
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.complete_portfolio_only = false;
    drop(data);
    let unclassified = estimate(&draft);
    assert_eq!(unclassified.state, ResultState::Unsupported);
    assert!(
        unclassified
            .issues
            .iter()
            .any(|issue| issue.display.key == "ISSUE_ESTATE_OTHER_SOURCE_REQUIRED")
    );
    // A mandatory source boundary clears estimate output through the ordinary
    // result lifecycle. It does not discard the accepted source amounts.
    let FederalIncome::EstateTrustPortfolio { data } = &draft.income[1].income else {
        unreachable!()
    };
    assert_eq!(data.form.interest, 100_000);
    assert_eq!(data.form.ordinary_dividends, 200_000);
    assert_eq!(crate::interview::document_amount(&draft.income[1].income, false), None);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.complete_portfolio_only = true;
    data.qualified_dividends = data.ordinary_dividends + 1;
    drop(data);
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.qualified_dividends = 50_000;
    data.unrecaptured_gain = data.long_gain;
    drop(data);
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());

    let mut high_income = complete(FederalStatus::Single);
    high_income.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(
        &mut high_income,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(k1),
        }),
    );
    let high_report = supported(&high_income);
    assert_eq!(value(&high_report, "net_investment_income_tax"), 152);
    assert_eq!(estimate(&restart(high_income.clone()).draft), high_report);
}

#[test]
fn estate_trust_final_code_11b_property_tax_uses_beneficiary_salt_limit() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    let ded = draft.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                interest: 1_000_000,
                ..empty_estate_portfolio()
            }),
        }),
    );
    let baseline = supported(&draft);
    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        data.statements.final_personal_property_tax = 350_000;
        data.statements.final_taxes_only = true;
        data.form.final_k1 = true;
        data.answers.final_year_successor = Some(true);
    }
    let property = supported(&draft);
    assert_eq!(value(&property, "estate_trust_2_11b_personal_property_tax"), 3_500);
    assert_eq!(value(&property, "taxable_income"), value(&baseline, "taxable_income") - 3_500);
    assert_eq!(return_line(&property, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(
        value(&property, "alternative_minimum_taxable_income"),
        value(&baseline, "alternative_minimum_taxable_income")
    );
    assert_eq!(value(&property, "net_investment_income_tax"), 380);
    assert_eq!(estimate(&restart(draft.clone()).draft), property);

    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        data.statements.final_personal_property_tax = 0;
        data.statements.final_real_estate_tax = 350_000;
    }
    let real_estate = supported(&draft);
    assert_eq!(value(&real_estate, "estate_trust_2_11b_real_estate_tax"), 3_500);
    assert_eq!(value(&real_estate, "taxable_income"), value(&baseline, "taxable_income") - 3_500);
    assert_eq!(
        value(&real_estate, "alternative_minimum_taxable_income"),
        value(&baseline, "alternative_minimum_taxable_income")
    );
    assert_eq!(value(&real_estate, "net_investment_income_tax"), 380);
    assert_eq!(estimate(&restart(draft.clone()).draft), real_estate);

    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        data.statements.final_real_estate_tax = 0;
        data.statements.final_state_income_tax = 350_000;
    }
    {
        let facts = &mut draft.deductions.as_mut().unwrap().itemized;
        facts.personal_tax =
            PersonalTaxElection::Income { paid: 0, allocation: TaxIncomeAllocation::CurrentReturn };
    }
    let income_tax = supported(&draft);
    assert_eq!(value(&income_tax, "estate_trust_2_11b_state_income_tax"), 3_500);
    assert_eq!(value(&income_tax, "taxable_income"), value(&baseline, "taxable_income") - 3_500);
    assert_eq!(
        value(&income_tax, "alternative_minimum_taxable_income"),
        value(&baseline, "alternative_minimum_taxable_income")
    );
    assert_eq!(value(&income_tax, "net_investment_income_tax"), 380);
    assert_eq!(estimate(&restart(draft.clone()).draft), income_tax);
    {
        let facts = &mut draft.deductions.as_mut().unwrap().itemized;
        facts.personal_tax = PersonalTaxElection::Sales { paid: 0 };
    }
    let sales_election = supported(&draft);
    assert_eq!(value(&sales_election, "taxable_income"), value(&baseline, "taxable_income"));
    assert!(
        sales_election
            .review_notes
            .iter()
            .any(|note| note.contains("state income tax was not deducted"))
    );
    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        data.statements.final_state_income_tax = 0;
        data.statements.final_state_sales_tax = 350_000;
    }
    let general_sales = supported(&draft);
    assert_eq!(value(&general_sales, "estate_trust_2_11b_state_sales_tax"), 3_500);
    assert_eq!(value(&general_sales, "taxable_income"), value(&baseline, "taxable_income") - 3_500);
    assert_eq!(
        value(&general_sales, "alternative_minimum_taxable_income"),
        value(&baseline, "alternative_minimum_taxable_income")
    );
    assert_eq!(estimate(&restart(draft.clone()).draft), general_sales);
    {
        let facts = &mut draft.deductions.as_mut().unwrap().itemized;
        facts.personal_tax =
            PersonalTaxElection::Income { paid: 0, allocation: TaxIncomeAllocation::CurrentReturn };
    }
    let income_election = supported(&draft);
    assert_eq!(value(&income_election, "taxable_income"), value(&baseline, "taxable_income"));
    assert!(
        income_election
            .review_notes
            .iter()
            .any(|note| note.contains("general sales tax was not deducted"))
    );

    {
        let facts = &mut draft.deductions.as_mut().unwrap().itemized;
        facts.personal_property_taxes = 3_900_000;
        facts.personal_tax =
            PersonalTaxElection::Income { paid: 0, allocation: TaxIncomeAllocation::CurrentReturn };
    }
    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        data.statements.final_personal_property_tax = 350_000;
        data.statements.final_real_estate_tax = 350_000;
        data.statements.final_state_income_tax = 350_000;
        data.statements.final_state_sales_tax = 0;
    }
    let capped = supported(&draft);
    let mut no_trust_tax = draft.clone();
    if let FederalIncome::EstateTrustPortfolio { data } = &mut no_trust_tax.income[1].income {
        data.statements.final_personal_property_tax = 0;
        data.statements.final_real_estate_tax = 0;
        data.statements.final_state_income_tax = 0;
        data.statements.final_state_sales_tax = 0;
    }
    let own_only = supported(&no_trust_tax);
    assert_eq!(value(&capped, "taxable_income"), value(&own_only, "taxable_income") - 1_400);
    assert_eq!(
        value(&capped, "alternative_minimum_taxable_income"),
        value(&own_only, "alternative_minimum_taxable_income")
    );
    assert_eq!(estimate(&restart(draft.clone()).draft), capped);
    let mut incompatible_fiduciary_elections = draft.clone();
    if let FederalIncome::EstateTrustPortfolio { data } =
        &mut incompatible_fiduciary_elections.income[1].income
    {
        data.statements.final_state_sales_tax = 100_000;
    }
    unsupported(&incompatible_fiduciary_elections, "Income:");

    draft.deductions.as_mut().unwrap().itemize_override = None;
    let standard = supported(&draft);
    let mut standard_without = draft.clone();
    if let FederalIncome::EstateTrustPortfolio { data } = &mut standard_without.income[1].income {
        data.statements.final_personal_property_tax = 0;
        data.statements.final_real_estate_tax = 0;
        data.statements.final_state_income_tax = 0;
        data.statements.final_state_sales_tax = 0;
    }
    // 2026-11-02: automatic selection retains the itemized SALT claim.
    assert_eq!(standard.tax, Some(4_755_800));
    assert_eq!(supported(&standard_without).tax, Some(4_800_600));
    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        data.answers.final_year_successor = Some(false);
    }
    unsupported(&draft, "Income:");
    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        data.form.final_k1 = true;
        data.answers.final_year_successor = Some(true);
        data.statements.final_taxes_only = false;
    }
    unsupported(&draft, "Income:");
}

#[test]
fn estate_trust_final_codes_11e_and_11f_keep_regular_and_amt_loss_years_separate() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                interest: 1_000_000,
                final_k1: true,
                final_year_successor: Some(true),
                ..empty_estate_portfolio()
            }),
        }),
    );
    let baseline = supported(&draft);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.final_nol = Some(FinalEstateTrustNol {
        terminated_on: None,
        years: vec![FinalEstateTrustNolYear {
            original_period: None,
            year: 2025,
            remaining_years: None,
            regular_balance: 10_000_000,
            alternative_balance: 8_000_000,
        }],
        fiduciary_history_complete: true,
        taxable_periods: vec![],
    });
    drop(data);
    let report = supported(&draft);
    assert_eq!(value(&report, "estate_trust_2_11e_regular_nol"), 100_000);
    assert_eq!(value(&report, "estate_trust_2_11f_amt_nol"), 80_000);
    assert_eq!(value(&report, "prior_nol_deduction"), 100_000);
    assert_eq!(value(&report, "prior_atnol_deduction"), 80_000);
    assert_eq!(value(&report, "agi"), value(&baseline, "agi") - 100_000);
    assert_eq!(value(&report, "estate_trust_2_nol_2025_carryforward"), 0);
    assert_eq!(value(&report, "estate_trust_2_atnol_2025_carryforward"), 0);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.final_nol.as_mut().unwrap().years.push(FinalEstateTrustNolYear {
        original_period: None,
        year: 2017,
        remaining_years: Some(11),
        regular_balance: 4_000_000,
        alternative_balance: 3_000_000,
    });
    // The fiduciary records establish calendar taxable years, including the
    // final 2026 year: nine following years leave eleven beneficiary years.
    // These are derived source facts, not a published IRS numerical example.
    let history = data.final_nol.as_mut().unwrap();
    history.terminated_on = Some(20261231);
    history.taxable_periods = (2017..=2026)
        .map(|year| crate::FinalEstateTrustTaxPeriod {
            period: crate::FinalEstateTrustLossPeriod {
                began: year * 10000 + 101,
                ended: year * 10000 + 1231,
            },
            accounting: crate::FinalEstateTrustAccountingYear::CalendarFiscalOrShort,
        })
        .collect();
    for loss in &mut history.years {
        loss.original_period = Some(crate::FinalEstateTrustLossPeriod {
            began: loss.year * 10000 + 101,
            ended: loss.year * 10000 + 1231,
        });
    }
    drop(data);
    let multiple = supported(&draft);
    assert_eq!(value(&multiple, "estate_trust_2_11e_regular_nol"), 140_000);
    assert_eq!(value(&multiple, "estate_trust_2_11f_amt_nol"), 110_000);
    assert_eq!(value(&multiple, "prior_nol_deduction"), 140_000);
    assert_eq!(value(&multiple, "prior_atnol_deduction"), 110_000);
    assert_eq!(value(&multiple, "estate_trust_2_nol_2017_used"), 40_000);
    assert_eq!(value(&multiple, "estate_trust_2_atnol_2017_used"), 30_000);
    assert_eq!(value(&multiple, "agi"), value(&baseline, "agi") - 140_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), multiple);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.final_nol.as_mut().unwrap().years[1].year = 2025;
    drop(data);
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.final_nol.as_mut().unwrap().years.pop();
    // Later branches deliberately exercise a new post-2017 source year;
    // remove this fixture's finite-period history rather than forge its dates.
    let history = data.final_nol.as_mut().unwrap();
    history.taxable_periods.clear();
    history.terminated_on = None;
    history.years[0].original_period = None;

    drop(data);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.final_year_successor = Some(false);
    drop(data);
    unsupported(&draft, "Synthetic document");
    let mut data = estate_trust_code_d_data(&mut draft);
    data.final_k1 = true;
    data.final_year_successor = Some(true);
    data.final_nol.as_mut().unwrap().fiduciary_history_complete = false;
    drop(data);
    assert!(noted(&supported(&draft), "WARN_ESTATE_TRUST_NOL_LEFT_OUT"));
    let mut data = estate_trust_code_d_data(&mut draft);
    data.final_nol.as_mut().unwrap().fiduciary_history_complete = true;
    drop(data);
    draft.screening.as_mut().unwrap().nol = prior_nol(&[(2024, 10_000)]);
    let combined = supported(&draft);
    assert_eq!(value(&combined, "prior_nol_deduction"), 110_000);
    assert_eq!(value(&combined, "prior_atnol_deduction"), 80_000);

    draft.screening.as_mut().unwrap().nol = NolHistory::None;
    draft.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    let mut data = estate_trust_code_d_data(&mut draft);
    let loss = &mut data.final_nol.as_mut().unwrap().years[0];
    loss.year = 2026;
    loss.regular_balance = 30_000_000;
    loss.alternative_balance = 30_000_000;
    drop(data);
    let limited = supported(&draft);
    assert!((1..300_000).contains(&value(&limited, "prior_nol_deduction")));
    assert!((1..300_000).contains(&value(&limited, "prior_atnol_deduction")));
    assert!(value(&limited, "estate_trust_2_nol_2026_carryforward") > 0);
    assert!(value(&limited, "estate_trust_2_atnol_2026_carryforward") > 0);
    assert_eq!(estimate(&restart(draft.clone()).draft), limited);
}

#[test]
fn estate_trust_code_14i_dividends_use_existing_reit_qbi_without_repeating_income() {
    let mut direct = complete(FederalStatus::Single);
    direct.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(&mut direct, income(reit_income(10_000, 6_000, true)));
    let direct_report = supported(&direct);

    let mut trust = complete(FederalStatus::Single);
    trust.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(
        &mut trust,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                ordinary_dividends: 1_000_000,
                section199a_dividends: ReitDividends::Reported {
                    amount: 600_000,
                    eligible: Some(true),
                },
                code_i_dividends_only: true,
                ..empty_estate_portfolio()
            }),
        }),
    );
    let report = supported(&trust);
    for id in ["3b", "11a / 11b", "24a"] {
        assert_eq!(return_line(&report, id), return_line(&direct_report, id));
    }
    assert_eq!(value(&report, "qbi_deduction"), 1_200);
    assert_eq!(value(&report, "net_investment_income_tax"), 380);
    assert_eq!(value(&report, "estate_trust_2_14i_section199a_dividends"), 6_000);
    assert_eq!(report.tax, direct_report.tax);
    assert_eq!(estimate(&restart(trust.clone()).draft), report);

    if let FederalIncome::EstateTrustPortfolio { data } = &mut trust.income[1].income {
        data.statements.code_i_dividends_only = false;
    }
    unsupported(&trust, "Income:");
    if let FederalIncome::EstateTrustPortfolio { data } = &mut trust.income[1].income {
        data.statements.code_i_dividends_only = true;
        data.statements.section199a_dividends =
            ReitDividends::Reported { amount: 600_000, eligible: Some(false) };
    }
    unsupported(&trust, "Income:");
    if let FederalIncome::EstateTrustPortfolio { data } = &mut trust.income[1].income {
        data.statements.section199a_dividends =
            ReitDividends::Reported { amount: 1_000_001, eligible: Some(true) };
    }
    assert!(crate::federal_rules::validate_income_amount(&trust.income[1].income).is_err());
}

#[test]
fn estate_trust_box_10_interest_ird_estate_tax_is_itemized_and_reduces_niit() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    let ded = draft.deductions.as_mut().unwrap();
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                interest: 1_000_000,
                ..empty_estate_portfolio()
            }),
        }),
    );
    let baseline = supported(&draft);
    assert_eq!(value(&baseline, "net_investment_income_tax"), 380);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.box_10_interest_ird_estate_tax = 500_000;
    data.box_10_interest_ird_only = true;
    drop(data);
    let itemized_report = supported(&draft);
    assert_eq!(value(&itemized_report, "estate_trust_2_10_estate_tax"), 5_000);
    assert_eq!(value(&itemized_report, "estate_trust_ird_estate_tax_deduction"), 5_000);
    assert_eq!(value(&itemized_report, "niit_itemized_deductions"), 5_000);
    assert_eq!(value(&itemized_report, "net_investment_income_tax"), 190);
    assert_eq!(return_line(&itemized_report, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(
        value(&itemized_report, "taxable_income"),
        value(&baseline, "taxable_income") - 5_000
    );
    assert_eq!(
        value(&itemized_report, "alternative_minimum_taxable_income"),
        value(&baseline, "alternative_minimum_taxable_income") - 5_000
    );
    assert_eq!(estimate(&restart(draft.clone()).draft), itemized_report);

    draft.deductions.as_mut().unwrap().itemize_override = None;
    let standard = supported(&draft);
    assert_eq!(value(&standard, "estate_trust_ird_estate_tax_deduction"), 0);
    assert_eq!(value(&standard, "net_investment_income_tax"), 380);
    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        data.statements.box_10_interest_ird_only = false;
    }
    unsupported(&draft, "Income:");
    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        data.statements.box_10_interest_ird_only = true;
        data.form.interest = 0;
    }
    unsupported(&draft, "Income:");
    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        let mut data = edit_estate(data);
        data.interest = 1_000_000;
        data.niit_adjustment = Some(EstateTrustNiitAdjustment { line_7: 0, modified_agi: 0 });
    }
    unsupported(&draft, "Income:");
}

#[test]
fn estate_trust_box_12b_refigures_amt_qualified_dividends_without_repeating_code_a() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                ordinary_dividends: 1_000_000,
                qualified_dividends: 1_000_000,
                ..empty_estate_portfolio()
            }),
        }),
    );
    let baseline = supported(&draft);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = 5_000_000;
    data.amt_box12b = 5_000_000;
    data.amt_box12_components_confirmed = true;
    drop(data);
    let code_b = supported(&draft);
    assert_eq!(
        value(&code_b, "alternative_minimum_taxable_income"),
        value(&baseline, "alternative_minimum_taxable_income") + 50_000
    );
    assert_eq!(return_line(&code_b, "3a"), return_line(&baseline, "3a"));
    assert_eq!(return_line(&code_b, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(value(&code_b, "estate_trust_2_12a_amt_adjustment"), 50_000);
    assert_eq!(value(&code_b, "estate_trust_2_12b_amt_qualified_dividends"), 50_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), code_b);

    let mut standalone = draft.clone();
    let mut data = estate_trust_code_d_data(&mut standalone);
    data.amt_box12b = 0;
    data.amt_box12_components_confirmed = false;
    data.amt_box12a_standalone = true;
    drop(data);
    let ordinary_amt = supported(&standalone);
    assert_eq!(
        value(&code_b, "alternative_minimum_taxable_income"),
        value(&ordinary_amt, "alternative_minimum_taxable_income")
    );
    assert!(
        value(&code_b, "alternative_minimum_tax") < value(&ordinary_amt, "alternative_minimum_tax")
    );

    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = -500_000;
    data.amt_box12b = -500_000;
    drop(data);
    let negative = supported(&draft);
    assert_eq!(
        value(&negative, "alternative_minimum_taxable_income"),
        value(&baseline, "alternative_minimum_taxable_income") - 5_000
    );
    assert_eq!(value(&negative, "estate_trust_2_12b_amt_qualified_dividends"), -5_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), negative);

    let mut with_investment_interest = draft.clone();
    investment_borrowing(&mut with_investment_interest, 100, 0, 0);
    unsupported(&with_investment_interest, "Estate/trust AMT qualified dividends");

    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12_components_confirmed = false;
    drop(data);
    unsupported(&draft, "Income:");
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12_components_confirmed = true;
    data.amt_box12a = -600_000;
    drop(data);
    unsupported(&draft, "Income:");
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = -500_000;
    data.amt_box12b = -1_000_001;
    drop(data);
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
}

#[test]
fn estate_trust_box_12cd_positive_refigures_amt_schedule_d_once() {
    for (short, component) in
        [(true, "estate_trust_2_12c_amt_short_gain"), (false, "estate_trust_2_12d_amt_long_gain")]
    {
        let mut draft = estate_trust_code_c_draft(0, 10_000_000);
        let mut data = estate_trust_code_d_data(&mut draft);
        data.tax_exempt_interest = 20_000_000;
        data.private_activity_interest = 20_000_000;
        drop(data);
        if short {
            add(&mut draft, income(ordinary_investment_sale(0, 4_000, GainTerm::Short)));
        }
        let baseline = supported(&draft);
        let mut data = estate_trust_code_d_data(&mut draft);
        data.amt_box12a = 500_000;
        if short {
            data.amt_box12c = 500_000;
        } else {
            data.amt_box12d = 500_000;
        }
        data.amt_box12_components_confirmed = true;
        drop(data);
        let report = supported(&draft);
        assert_eq!(value(&report, "estate_trust_2_12a_amt_adjustment"), 5_000);
        assert_eq!(value(&report, component), 5_000);
        assert_eq!(
            value(&report, "amt_capital_gain"),
            value(&baseline, "amt_capital_gain") + 5_000
        );
        assert_eq!(value(&report, "amt_capital_adjustment"), 0);
        assert_eq!(
            value(&report, "alternative_minimum_taxable_income"),
            value(&baseline, "alternative_minimum_taxable_income") + 5_000
        );
        assert_eq!(value(&report, "agi"), value(&baseline, "agi"));
        assert_eq!(return_line(&report, "7a"), return_line(&baseline, "7a"));
        assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b"));
        assert_eq!(estimate(&restart(draft.clone()).draft), report);
        let mut data = estate_trust_code_d_data(&mut draft);
        data.amt_box12c = 0;
        data.amt_box12d = 0;
        data.amt_box12_components_confirmed = false;
        data.amt_box12a_standalone = true;
        drop(data);
        let standalone = supported(&draft);
        assert_eq!(
            value(&standalone, "alternative_minimum_taxable_income"),
            value(&report, "alternative_minimum_taxable_income")
        );
        assert_eq!(
            value(&standalone, "amt_capital_gain") + 5_000,
            value(&report, "amt_capital_gain")
        );
        assert!(
            value(&report, "alternative_minimum_tax")
                < value(&standalone, "alternative_minimum_tax"),
            "code C/D preferential gain versus standalone ordinary adjustment: short={short}"
        );
    }
}

#[test]
fn estate_trust_box_12cd_negative_preserves_amt_capital_loss_limit_and_carryover() {
    for (short, long, term, carry) in [
        (5_000_000, 0, GainTerm::Short, "amt_capital_short_carryforward"),
        (0, 5_000_000, GainTerm::Long, "amt_capital_long_carryforward"),
    ] {
        let mut draft = estate_trust_code_c_draft(short, long);
        add(&mut draft, income(ordinary_investment_sale(0, 20_000, term)));
        let baseline = supported(&draft);
        let mut data = estate_trust_code_d_data(&mut draft);
        data.amt_box12a = -5_000_000;
        if short != 0 {
            data.amt_box12c = -5_000_000;
        } else {
            data.amt_box12d = -5_000_000;
        }
        data.amt_box12_components_confirmed = true;
        drop(data);
        let report = supported(&draft);
        assert_eq!(value(&baseline, "capital_gain"), 30_000);
        assert_eq!(value(&report, "amt_capital_gain"), -3_000);
        assert_eq!(value(&report, "amt_capital_adjustment"), 17_000);
        assert_eq!(
            value(&report, "alternative_minimum_taxable_income"),
            value(&baseline, "alternative_minimum_taxable_income") - 33_000
        );
        assert_eq!(value(&report, carry), 17_000);
        assert_eq!(value(&report, "agi"), value(&baseline, "agi"));
        assert_eq!(return_line(&report, "7a"), return_line(&baseline, "7a"));
        assert_eq!(estimate(&restart(draft.clone()).draft), report);
    }
}

#[test]
fn estate_trust_box_12c_rejects_unconfirmed_or_unreconciled_sources_and_form_4952() {
    let mut draft = estate_trust_code_c_draft(0, 0);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = 500_000;
    data.amt_box12c = 500_000;
    drop(data);
    unsupported(&draft, "Income:");
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12_components_confirmed = true;
    drop(data);
    supported(&draft);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = 400_000;
    drop(data);
    unsupported(&draft, "Income:");
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = 500_000;
    data.amt_box12b = 500_000;
    data.amt_box12_components_confirmed = true;
    drop(data);
    unsupported(&draft, "Income:");
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12b = 0;
    data.amt_box12_components_confirmed = false;
    data.amt_box12c = -100_000;
    data.amt_box12a = -100_000;
    drop(data);
    unsupported(&draft, "Income:");
    let mut data = estate_trust_code_d_data(&mut draft);
    data.short_gain = 200_000;
    data.amt_box12_components_confirmed = true;
    drop(data);
    supported(&draft);
    investment_borrowing(&mut draft, 100, 0, 0);
    unsupported(&draft, "Estate/trust AMT short-term gain");
}

#[test]
fn estate_trust_box_12d_requires_complete_source_and_separate_special_rate_worksheets() {
    let mut draft = estate_trust_code_c_draft(0, 10_000_000);
    estate_trust_code_d_data(&mut draft).amt_box12a = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12d = 500_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    supported(&draft);
    estate_trust_code_d_data(&mut draft).amt_box12a = 400_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12a = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12c = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12c = 0;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = false;
    estate_trust_code_d_data(&mut draft).rate_28_gain = 100_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).rate_28_gain = 0;
    estate_trust_code_d_data(&mut draft).unrecaptured_gain = 100_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).unrecaptured_gain = 0;
    estate_trust_code_d_data(&mut draft).amt_box12a = -11_000_000;
    estate_trust_code_d_data(&mut draft).amt_box12d = -11_000_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12a = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12d = 500_000;
    investment_borrowing(&mut draft, 100, 0, 0);
    unsupported(&draft, "Estate/trust AMT long-term gain");
}

#[test]
fn estate_trust_box_12e_signed_refigures_matching_amt_long_and_1250_gain_once() {
    for (adjustment, gain, tax, standalone_tax) in
        [(500_000, 5_000, 1_250, -150), (-500_000, -5_000, -1_250, 150)]
    {
        let mut draft = estate_trust_code_c_draft(0, 10_000_000);
        let mut data = estate_trust_code_d_data(&mut draft);
        data.unrecaptured_gain = 1_000_000;
        data.tax_exempt_interest = 10_000_000;
        data.private_activity_interest = 10_000_000;
        drop(data);
        let baseline = supported(&draft);
        let mut data = estate_trust_code_d_data(&mut draft);
        data.amt_box12a = adjustment;
        data.amt_box12d = adjustment;
        data.amt_box12e = adjustment;
        data.amt_box12e_included_in_d = true;
        data.amt_box12_components_confirmed = true;
        drop(data);
        let report = supported(&draft);
        if adjustment > 0 {
            assert_eq!(value(&report, "estate_trust_2_12a_amt_adjustment"), 5_000);
            assert_eq!(value(&report, "estate_trust_2_12e_amt_unrecaptured_gain"), 5_000);
            assert_eq!(value(&report, "amt_capital_adjustment"), 0);
        }
        assert_eq!(value(&report, "amt_capital_gain"), value(&baseline, "amt_capital_gain") + gain);
        assert_eq!(
            value(&report, "amt_unrecaptured_section_1250_gain"),
            value(&baseline, "amt_unrecaptured_section_1250_gain") + gain
        );
        assert_eq!(
            value(&report, "alternative_minimum_taxable_income"),
            value(&baseline, "alternative_minimum_taxable_income") + gain
        );
        assert_eq!(value(&report, "agi"), value(&baseline, "agi"));
        assert_eq!(return_line(&report, "7a"), return_line(&baseline, "7a"));
        assert_eq!(
            value(&report, "alternative_minimum_tax") - value(&baseline, "alternative_minimum_tax"),
            tax
        );
        assert_eq!(estimate(&restart(draft.clone()).draft), report);
        let mut data = estate_trust_code_d_data(&mut draft);
        data.amt_box12d = 0;
        data.amt_box12e = 0;
        data.amt_box12e_included_in_d = false;
        data.amt_box12_components_confirmed = false;
        data.amt_box12a_standalone = true;
        drop(data);
        let standalone = supported(&draft);
        assert_eq!(
            value(&standalone, "alternative_minimum_taxable_income"),
            value(&report, "alternative_minimum_taxable_income")
        );
        if adjustment > 0 {
            assert_eq!(
                value(&standalone, "amt_capital_gain"),
                value(&report, "amt_capital_gain") - 5_000
            );
        }
        assert_eq!(
            value(&report, "alternative_minimum_tax")
                - value(&standalone, "alternative_minimum_tax"),
            standalone_tax
        );
    }
}

#[test]
fn estate_trust_box_12ef_partial_signed_refigures_only_special_portion() {
    for (section_1250, adjustment, special, gain, pool_gain, tax) in [
        (true, 500_000, 200_000, 5_000, 2_000, 950),
        (true, -500_000, -200_000, -5_000, -2_000, -950),
        (false, 500_000, 200_000, 5_000, 2_000, 1_010),
        (false, -500_000, -200_000, -5_000, -2_000, -1_010),
    ] {
        let mut draft = estate_trust_code_c_draft(0, 10_000_000);
        let mut data = estate_trust_code_d_data(&mut draft);
        if section_1250 {
            data.unrecaptured_gain = 1_000_000;
        } else if adjustment < 0 {
            data.rate_28_gain = 1_000_000;
        }
        data.tax_exempt_interest = 10_000_000;
        data.private_activity_interest = 10_000_000;
        drop(data);
        let baseline = supported(&draft);
        let mut data = estate_trust_code_d_data(&mut draft);
        data.amt_box12a = adjustment;
        data.amt_box12d = adjustment;
        if section_1250 {
            data.amt_box12e = special;
            data.amt_box12e_included_in_d = true;
        } else {
            data.amt_box12f = special;
            data.amt_box12f_included_in_d = true;
        }
        data.amt_box12_components_confirmed = true;
        let (component, pool) = if section_1250 {
            ("estate_trust_2_12e_amt_unrecaptured_gain", "amt_unrecaptured_section_1250_gain")
        } else {
            ("estate_trust_2_12f_amt_rate_28_gain", "amt_rate_28_gain")
        };
        drop(data);
        let partial = supported(&draft);
        if adjustment > 0 {
            assert_eq!(value(&partial, "estate_trust_2_12a_amt_adjustment"), 5_000);
            assert_eq!(value(&partial, "estate_trust_2_12d_amt_long_gain"), 5_000);
            assert_eq!(value(&partial, "amt_capital_adjustment"), 0);
        }
        assert_eq!(value(&partial, component), pool_gain);
        assert_eq!(
            value(&partial, "amt_capital_gain"),
            value(&baseline, "amt_capital_gain") + gain
        );
        assert_eq!(value(&partial, pool), value(&baseline, pool) + pool_gain);
        assert_eq!(
            value(&partial, "alternative_minimum_taxable_income"),
            value(&baseline, "alternative_minimum_taxable_income") + gain
        );
        assert_eq!(value(&partial, "agi"), value(&baseline, "agi"));
        assert_eq!(return_line(&partial, "7a"), return_line(&baseline, "7a"));
        assert_eq!(
            value(&partial, "alternative_minimum_tax")
                - value(&baseline, "alternative_minimum_tax"),
            tax
        );
        assert_eq!(estimate(&restart(draft.clone()).draft), partial);
        if adjustment > 0 {
            let mut data = estate_trust_code_d_data(&mut draft);
            if section_1250 {
                data.amt_box12e = 500_000;
            } else {
                data.amt_box12f = 500_000;
            }
            drop(data);
            let entirely_special = supported(&draft);
            assert_eq!(
                value(&entirely_special, "alternative_minimum_taxable_income"),
                value(&partial, "alternative_minimum_taxable_income")
            );
            assert_eq!(value(&entirely_special, pool), value(&partial, pool) + 3_000);
            assert!(
                value(&entirely_special, "alternative_minimum_tax")
                    > value(&partial, "alternative_minimum_tax")
            );
        }
    }
}

#[test]
fn estate_trust_box_12e_partial_rejects_unconfirmed_or_inconsistent_sources() {
    let mut draft = estate_trust_code_c_draft(0, 10_000_000);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.unrecaptured_gain = 1_000_000;
    data.amt_box12a = 500_000;
    data.amt_box12d = 500_000;
    data.amt_box12e = 200_000;
    drop(data);
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12e_included_in_d = true;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    supported(&draft);

    estate_trust_code_d_data(&mut draft).amt_box12a = 400_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12a = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12e = 600_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12e = -200_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12e = 0;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12e = 200_000;
    estate_trust_code_d_data(&mut draft).amt_box12f = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12f_included_in_d = true;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12f = 0;
    estate_trust_code_d_data(&mut draft).amt_box12f_included_in_d = false;
    estate_trust_code_d_data(&mut draft).rate_28_gain = 100_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).rate_28_gain = 0;

    estate_trust_code_d_data(&mut draft).amt_box12a = -5_000_000;
    estate_trust_code_d_data(&mut draft).amt_box12d = -5_000_000;
    estate_trust_code_d_data(&mut draft).amt_box12e = -1_100_000;
    unsupported(&draft, "Income:");
}

#[test]
fn estate_trust_box_12e_requires_matching_long_gain_and_nonnegative_adjusted_pool() {
    let mut draft = estate_trust_code_c_draft(0, 10_000_000);
    estate_trust_code_d_data(&mut draft).unrecaptured_gain = 1_000_000;
    estate_trust_code_d_data(&mut draft).amt_box12a = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12e = 500_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12e_included_in_d = true;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12d = 500_000;
    supported(&draft);
    estate_trust_code_d_data(&mut draft).amt_box12a = 400_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12a = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12d = 400_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12d = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = false;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    estate_trust_code_d_data(&mut draft).amt_box12e = -1_000_001;
    estate_trust_code_d_data(&mut draft).amt_box12d = -1_000_001;
    estate_trust_code_d_data(&mut draft).amt_box12a = -1_000_001;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12e = -11_000_000;
    estate_trust_code_d_data(&mut draft).amt_box12d = -11_000_000;
    estate_trust_code_d_data(&mut draft).amt_box12a = -11_000_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12e = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12d = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12a = 500_000;
    investment_borrowing(&mut draft, 100, 0, 0);
    unsupported(&draft, "Estate/trust AMT unrecaptured section 1250 gain");
}

#[test]
fn estate_trust_box_12f_refigures_only_amt_28_percent_rate_pool() {
    let mut draft = estate_trust_code_c_draft(0, 10_000_000);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.tax_exempt_interest = 20_000_000;
    data.private_activity_interest = 20_000_000;
    drop(data);
    let baseline = supported(&draft);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = 5_000_000;
    data.amt_box12d = 5_000_000;
    data.amt_box12_components_confirmed = true;
    drop(data);
    let ordinary_long = supported(&draft);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12_components_confirmed = false;
    data.amt_box12f = 5_000_000;
    data.amt_box12f_included_in_d = true;
    data.amt_box12_components_confirmed = true;
    drop(data);
    let report = supported(&draft);
    assert_eq!(value(&report, "estate_trust_2_12a_amt_adjustment"), 50_000);
    assert_eq!(value(&report, "estate_trust_2_12d_amt_long_gain"), 50_000);
    assert_eq!(value(&report, "estate_trust_2_12f_amt_rate_28_gain"), 50_000);
    assert_eq!(value(&report, "amt_capital_gain"), value(&baseline, "amt_capital_gain") + 50_000);
    assert_eq!(value(&report, "amt_capital_gain"), value(&ordinary_long, "amt_capital_gain"));
    assert_eq!(value(&report, "amt_capital_adjustment"), 0);
    assert_eq!(
        value(&report, "alternative_minimum_taxable_income"),
        value(&ordinary_long, "alternative_minimum_taxable_income")
    );
    assert_eq!(
        value(&report, "amt_rate_28_gain"),
        value(&ordinary_long, "amt_rate_28_gain") + 50_000
    );
    assert!(
        value(&report, "alternative_minimum_tax")
            > value(&ordinary_long, "alternative_minimum_tax")
    );
    assert_eq!(value(&report, "agi"), value(&baseline, "agi"));
    assert_eq!(return_line(&report, "7a"), return_line(&baseline, "7a"));
    assert_eq!(estimate(&restart(draft.clone()).draft), report);
}

#[test]
fn estate_trust_box_12f_negative_removes_regular_28_percent_component_once() {
    let mut draft = estate_trust_code_c_draft(0, 10_000_000);
    estate_trust_code_d_data(&mut draft).rate_28_gain = 5_000_000;
    let baseline = supported(&draft);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = -5_000_000;
    data.amt_box12d = -5_000_000;
    data.amt_box12f = -5_000_000;
    data.amt_box12f_included_in_d = true;
    data.amt_box12_components_confirmed = true;
    drop(data);
    let report = supported(&draft);
    assert_eq!(value(&report, "estate_trust_2_12f_amt_rate_28_gain"), -50_000);
    assert_eq!(value(&report, "amt_rate_28_gain"), 0);
    assert_eq!(value(&report, "amt_capital_gain"), value(&baseline, "amt_capital_gain") - 50_000);
    assert_eq!(value(&report, "amt_capital_adjustment"), 0);
    assert_eq!(
        value(&report, "alternative_minimum_taxable_income"),
        value(&baseline, "alternative_minimum_taxable_income") - 50_000
    );
    assert_eq!(value(&report, "agi"), value(&baseline, "agi"));
    assert_eq!(estimate(&restart(draft.clone()).draft), report);
}

#[test]
fn estate_trust_box_12f_partial_rejects_unconfirmed_or_inconsistent_sources() {
    let mut draft = estate_trust_code_c_draft(0, 10_000_000);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = 500_000;
    data.amt_box12d = 500_000;
    data.amt_box12f = 200_000;
    drop(data);
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12f_included_in_d = true;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    supported(&draft);

    estate_trust_code_d_data(&mut draft).amt_box12a = 400_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12a = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12f = 600_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12f = -200_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12f = 0;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12f = 200_000;
    estate_trust_code_d_data(&mut draft).amt_box12e = 200_000;
    estate_trust_code_d_data(&mut draft).amt_box12e_included_in_d = true;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    supported(&draft);
    estate_trust_code_d_data(&mut draft).amt_box12e = 400_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12e = 0;
    estate_trust_code_d_data(&mut draft).amt_box12e_included_in_d = false;
    estate_trust_code_d_data(&mut draft).amt_box12c = 100_000;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12c = 0;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = false;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    estate_trust_code_d_data(&mut draft).unrecaptured_gain = 100_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).unrecaptured_gain = 0;

    estate_trust_code_d_data(&mut draft).amt_box12a = -500_000;
    estate_trust_code_d_data(&mut draft).amt_box12d = -500_000;
    estate_trust_code_d_data(&mut draft).amt_box12f = -200_000;
    unsupported(&draft, "Income:"); // No regular 28%-rate gain remains to reduce.
    estate_trust_code_d_data(&mut draft).rate_28_gain = 10_000_000;
    unsupported(&draft, "Income:"); // Adjusted 28%-rate gain exceeds adjusted long-term gain.
    estate_trust_code_d_data(&mut draft).rate_28_gain = 1_000_000;
    supported(&draft);
}

#[test]
fn estate_trust_box_12f_requires_aligned_confirmed_source_and_rate_character() {
    let mut draft = estate_trust_code_c_draft(0, 10_000_000);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = 500_000;
    data.amt_box12d = 500_000;
    data.amt_box12f = 500_000;
    drop(data);
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12f_included_in_d = true;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    supported(&draft);
    estate_trust_code_d_data(&mut draft).amt_box12a = 400_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12a = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12f = 400_000;
    supported(&draft);
    estate_trust_code_d_data(&mut draft).amt_box12f = 600_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12f = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = false;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    estate_trust_code_d_data(&mut draft).amt_box12c = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12c = 0;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = false;
    estate_trust_code_d_data(&mut draft).amt_box12e = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12e_included_in_d = true;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12e = 0;
    estate_trust_code_d_data(&mut draft).amt_box12e_included_in_d = false;
    estate_trust_code_d_data(&mut draft).unrecaptured_gain = 100_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).unrecaptured_gain = 0;
    estate_trust_code_d_data(&mut draft).amt_box12a = -500_000;
    estate_trust_code_d_data(&mut draft).amt_box12d = -500_000;
    estate_trust_code_d_data(&mut draft).amt_box12f = -500_000;
    unsupported(&draft, "Income:"); // No regular 28%-rate gain remains to reduce.
    estate_trust_code_d_data(&mut draft).amt_box12a = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12d = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12f = 500_000;
    investment_borrowing(&mut draft, 100, 0, 0);
    unsupported(&draft, "Estate/trust AMT long-term gain");
}

#[test]
fn estate_trust_box_12ef_signed_refigures_both_rate_pools_once() {
    for (adjustment, e, f, gain, e_gain, f_gain, tax) in [
        (500_000, 100_000, 200_000, 5_000, 1_000, 2_000, 1_110),
        (-500_000, -100_000, -200_000, -5_000, -1_000, -2_000, -1_110),
    ] {
        let mut draft = estate_trust_code_c_draft(0, 10_000_000);
        let mut data = estate_trust_code_d_data(&mut draft);
        if adjustment < 0 {
            data.unrecaptured_gain = 1_000_000;
            data.rate_28_gain = 1_000_000;
        }
        data.tax_exempt_interest = 10_000_000;
        data.private_activity_interest = 10_000_000;
        drop(data);
        let baseline = supported(&draft);

        let mut data = estate_trust_code_d_data(&mut draft);
        data.amt_box12a = adjustment;
        data.amt_box12d = adjustment;
        data.amt_box12e = e;
        data.amt_box12f = f;
        data.amt_box12e_included_in_d = true;
        data.amt_box12_components_confirmed = true;
        data.amt_box12f_included_in_d = true;
        drop(data);
        let report = supported(&draft);
        if adjustment > 0 {
            assert_eq!(value(&report, "estate_trust_2_12a_amt_adjustment"), 5_000);
            assert_eq!(value(&report, "estate_trust_2_12d_amt_long_gain"), 5_000);
        }
        assert_eq!(value(&report, "estate_trust_2_12e_amt_unrecaptured_gain"), e_gain);
        assert_eq!(value(&report, "estate_trust_2_12f_amt_rate_28_gain"), f_gain);
        assert_eq!(value(&report, "amt_capital_gain"), value(&baseline, "amt_capital_gain") + gain);
        assert_eq!(
            value(&report, "amt_unrecaptured_section_1250_gain"),
            value(&baseline, "amt_unrecaptured_section_1250_gain") + e_gain
        );
        assert_eq!(
            value(&report, "amt_rate_28_gain"),
            value(&baseline, "amt_rate_28_gain") + f_gain
        );
        if adjustment > 0 {
            assert_eq!(value(&report, "amt_capital_adjustment"), 0);
        }
        assert_eq!(
            value(&report, "alternative_minimum_taxable_income"),
            value(&baseline, "alternative_minimum_taxable_income") + gain
        );
        assert_eq!(value(&report, "agi"), value(&baseline, "agi"));
        assert_eq!(return_line(&report, "7a"), return_line(&baseline, "7a"));
        assert_eq!(
            value(&report, "alternative_minimum_tax") - value(&baseline, "alternative_minimum_tax"),
            tax,
            "$1,000 section 1250 at 25%, $2,000 at 28%, and the $2,000 ordinary long-term remainder at 15%"
        );
        assert_eq!(estimate(&restart(draft.clone()).draft), report);
    }
}

#[test]
fn estate_trust_box_12ef_rejects_incomplete_or_overlapping_allocations() {
    let mut draft = estate_trust_code_c_draft(0, 10_000_000);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = 500_000;
    data.amt_box12d = 500_000;
    data.amt_box12e = 100_000;
    data.amt_box12f = 200_000;
    drop(data);
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12e_included_in_d = true;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12f_included_in_d = true;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    supported(&draft);

    estate_trust_code_d_data(&mut draft).amt_box12a = 400_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12a = 500_000;
    estate_trust_code_d_data(&mut draft).amt_box12e = 400_000;
    unsupported(&draft, "Income:"); // E and F would exceed D.
    estate_trust_code_d_data(&mut draft).amt_box12e = -100_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12e = 100_000;
    estate_trust_code_d_data(&mut draft).amt_box12f = -200_000;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12f = 200_000;
    estate_trust_code_d_data(&mut draft).amt_box12c = 100_000;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12c = 0;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = false;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;

    estate_trust_code_d_data(&mut draft).amt_box12a = -500_000;
    estate_trust_code_d_data(&mut draft).amt_box12d = -500_000;
    estate_trust_code_d_data(&mut draft).amt_box12e = -100_000;
    estate_trust_code_d_data(&mut draft).amt_box12f = -200_000;
    unsupported(&draft, "Income:"); // Neither regular special-rate pool can go negative.
    estate_trust_code_d_data(&mut draft).unrecaptured_gain = 100_000;
    unsupported(&draft, "Income:"); // The 28%-rate pool still goes negative.
    estate_trust_code_d_data(&mut draft).rate_28_gain = 200_000;
    supported(&draft);

    estate_trust_code_d_data(&mut draft).unrecaptured_gain = 5_000_000;
    estate_trust_code_d_data(&mut draft).rate_28_gain = 5_000_000;
    unsupported(&draft, "Income:"); // Combined adjusted pools exceed adjusted long gain.
    estate_trust_code_d_data(&mut draft).amt_box12e = -200_000;
    estate_trust_code_d_data(&mut draft).amt_box12f = -300_000;
    supported(&draft); // Both adjusted pools exactly exhaust adjusted long gain.
    investment_borrowing(&mut draft, 100, 0, 0);
    unsupported(&draft, "Estate/trust AMT long-term gain");
}

#[test]
fn estate_trust_box_12_bcd_components_with_ef_subclasses_refigure_amt_once() {
    for direction in [1, -1] {
        let mut draft = estate_trust_code_c_draft(1_000_000, 10_000_000);
        let mut data = estate_trust_code_d_data(&mut draft);
        data.ordinary_dividends = 1_000_000;
        data.qualified_dividends = 1_000_000;
        data.tax_exempt_interest = 10_000_000;
        data.private_activity_interest = 10_000_000;
        if direction < 0 {
            data.unrecaptured_gain = 1_000_000;
            data.rate_28_gain = 1_000_000;
        }
        drop(data);
        let baseline = supported(&draft);
        let mut data = estate_trust_code_d_data(&mut draft);
        data.amt_box12a = direction * 800_000;
        data.amt_box12b = direction * 100_000;
        data.amt_box12c = direction * 200_000;
        data.amt_box12d = direction * 500_000;
        data.amt_box12e = direction * 100_000;
        data.amt_box12f = direction * 200_000;
        data.amt_box12_components_confirmed = true;
        data.amt_box12e_included_in_d = true;
        data.amt_box12f_included_in_d = true;
        drop(data);
        let report = supported(&draft);
        assert_eq!(value(&report, "estate_trust_2_12a_amt_adjustment"), direction * 8_000);
        assert_eq!(value(&report, "estate_trust_2_12b_amt_qualified_dividends"), direction * 1_000);
        assert_eq!(value(&report, "estate_trust_2_12c_amt_short_gain"), direction * 2_000);
        assert_eq!(value(&report, "estate_trust_2_12d_amt_long_gain"), direction * 5_000);
        assert_eq!(value(&report, "estate_trust_2_12e_amt_unrecaptured_gain"), direction * 1_000);
        assert_eq!(value(&report, "estate_trust_2_12f_amt_rate_28_gain"), direction * 2_000);
        assert_eq!(
            value(&report, "amt_capital_gain"),
            value(&baseline, "amt_capital_gain") + direction * 7_000
        );
        assert_eq!(
            value(&report, "amt_unrecaptured_section_1250_gain"),
            value(&baseline, "amt_unrecaptured_section_1250_gain") + direction * 1_000
        );
        assert_eq!(
            value(&report, "amt_rate_28_gain"),
            value(&baseline, "amt_rate_28_gain") + direction * 2_000
        );
        assert_eq!(value(&report, "amt_capital_adjustment"), 0);
        assert_eq!(
            value(&report, "alternative_minimum_taxable_income"),
            value(&baseline, "alternative_minimum_taxable_income") + direction * 8_000
        );
        assert_eq!(value(&report, "agi"), value(&baseline, "agi"));
        assert_eq!(return_line(&report, "7a"), return_line(&baseline, "7a"));
        assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b"));
        assert_eq!(
            value(&report, "alternative_minimum_tax") - value(&baseline, "alternative_minimum_tax"),
            direction * 1_820
        );
        assert_eq!(estimate(&restart(draft.clone()).draft), report);
    }
}

#[test]
fn estate_trust_box_12_bcd_reconciliation_rejects_omissions_and_opposing_signs() {
    let mut draft = estate_trust_code_c_draft(1_000_000, 10_000_000);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.ordinary_dividends = 1_000_000;
    data.qualified_dividends = 1_000_000;
    data.amt_box12a = 800_000;
    data.amt_box12b = 100_000;
    data.amt_box12c = 200_000;
    data.amt_box12d = 500_000;
    drop(data);
    unsupported(&draft, "Income:"); // A complete source attribution is required.
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = true;
    supported(&draft);
    estate_trust_code_d_data(&mut draft).amt_box12a = 700_000;
    unsupported(&draft, "Income:"); // A cannot omit a component.
    estate_trust_code_d_data(&mut draft).amt_box12a = 800_000;
    estate_trust_code_d_data(&mut draft).amt_box12b = -100_000;
    estate_trust_code_d_data(&mut draft).amt_box12c = 400_000;
    unsupported(&draft, "Income:"); // Signed offsetting components need a separate statement.
    estate_trust_code_d_data(&mut draft).amt_box12b = 100_000;
    estate_trust_code_d_data(&mut draft).amt_box12c = 200_000;
    estate_trust_code_d_data(&mut draft).amt_box12e = 400_000;
    estate_trust_code_d_data(&mut draft).amt_box12f = 200_000;
    estate_trust_code_d_data(&mut draft).amt_box12e_included_in_d = true;
    estate_trust_code_d_data(&mut draft).amt_box12f_included_in_d = true;
    unsupported(&draft, "Income:"); // The subclasses cannot exceed D.
    estate_trust_code_d_data(&mut draft).amt_box12e = 100_000;
    supported(&draft);
    estate_trust_code_d_data(&mut draft).amt_box12f_included_in_d = false;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12f_included_in_d = true;
    estate_trust_code_d_data(&mut draft).amt_box12a_standalone = true;
    unsupported(&draft, "Income:");
    estate_trust_code_d_data(&mut draft).amt_box12a_standalone = false;
    estate_trust_code_d_data(&mut draft).amt_box12b = 0;
    estate_trust_code_d_data(&mut draft).amt_box12c = 0;
    estate_trust_code_d_data(&mut draft).amt_box12d = 0;
    estate_trust_code_d_data(&mut draft).amt_box12e = 0;
    estate_trust_code_d_data(&mut draft).amt_box12f = 0;
    estate_trust_code_d_data(&mut draft).amt_box12e_included_in_d = false;
    estate_trust_code_d_data(&mut draft).amt_box12f_included_in_d = false;
    estate_trust_code_d_data(&mut draft).amt_box12a = 0;
    estate_trust_code_d_data(&mut draft).amt_box12_components_confirmed = false;
    estate_trust_code_d_data(&mut draft).amt_box12b = -100_000;
    estate_trust_code_d_data(&mut draft).amt_box12c = 100_000;
    unsupported(&draft, "Income:"); // Opposing components do not disappear when A nets to zero.
}

#[test]
fn estate_trust_standalone_box_12a_changes_amt_without_changing_regular_agi() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                interest: 1_000_000,
                ..empty_estate_portfolio()
            }),
        }),
    );
    let baseline = supported(&draft);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = 10_000_000;
    data.amt_box12a_standalone = true;
    drop(data);
    let positive = supported(&draft);
    assert_eq!(
        value(&positive, "alternative_minimum_taxable_income"),
        value(&baseline, "alternative_minimum_taxable_income") + 100_000
    );
    assert_eq!(return_line(&positive, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert!(
        value(&positive, "alternative_minimum_tax") > value(&baseline, "alternative_minimum_tax")
    );
    assert_eq!(value(&positive, "estate_trust_2_12a_amt_adjustment"), 100_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), positive);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.amt_box12a = -5_000_000;
    drop(data);
    let negative = supported(&draft);
    assert_eq!(
        value(&negative, "alternative_minimum_taxable_income"),
        value(&baseline, "alternative_minimum_taxable_income") - 50_000
    );
    assert_eq!(return_line(&negative, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(value(&negative, "estate_trust_2_12a_amt_adjustment"), -50_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), negative);

    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        data.statements.amt_box12a_standalone = false;
    }
    unsupported(&draft, "Income:");
    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        data.statements.amt_box12a_standalone = true;
    }
    draft.screening.as_mut().unwrap().nol = prior_nol(&[(2025, 1_000)]);
    unsupported(&draft, "Estate/trust AMT adjustment");
}

#[test]
fn estate_trust_box_14h_adjusts_niit_and_magi_without_repeating_portfolio_income() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                interest: 1_000_000,
                ..empty_estate_portfolio()
            }),
        }),
    );
    let baseline = supported(&draft);
    assert_eq!(value(&baseline, "net_investment_income_tax"), 380);
    assert_eq!(value(&baseline, "net_investment_modified_agi"), 260_000);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.niit_adjustment =
        Some(EstateTrustNiitAdjustment { line_7: 500_000, modified_agi: 500_000 });
    drop(data);
    let positive = supported(&draft);
    assert_eq!(return_line(&positive, "2b"), return_line(&baseline, "2b"));
    assert_eq!(return_line(&positive, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(value(&positive, "net_investment_modified_agi"), 265_000);
    assert_eq!(value(&positive, "net_investment_income_tax"), 570);
    assert_eq!(value(&positive, "estate_trust_2_14h_niit_line_7"), 5_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), positive);

    // An invalid stored amount cannot be edited again, so the refusal is checked on a copy.
    let mut invalid = draft.income[1].income.clone();
    if let FederalIncome::EstateTrustPortfolio { data } = &mut invalid {
        let mut data = edit_estate(data);
        data.niit_adjustment.as_mut().unwrap().modified_agi = 0;
    }
    assert!(crate::federal_rules::validate_income_amount(&invalid).is_err());
    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[1].income {
        let mut data = edit_estate(data);
        data.niit_adjustment =
            Some(EstateTrustNiitAdjustment { line_7: -500_000, modified_agi: -200_000 });
    }
    draft.income[0] = wage(1, Owner::Taxpayer, 19_500_000, 3_000_000);
    let negative = supported(&draft);
    assert_eq!(value(&negative, "net_investment_modified_agi"), 203_000);
    assert_eq!(value(&negative, "net_investment_income_tax"), 114);
    assert_eq!(value(&negative, "estate_trust_2_14h_niit_line_7"), -5_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), negative);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.niit_adjustment.as_mut().unwrap().modified_agi = -500_001;
    drop(data);
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_err());
}

#[test]
fn estate_trust_tax_exempt_interest_reaches_social_security_provisional_income() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    add(&mut draft, income(social_security_net(2_000_000, 0)));
    let baseline = supported(&draft);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                tax_exempt_interest: 2_000_000,
                ..empty_estate_portfolio()
            }),
        }),
    );
    let report = supported(&draft);
    assert_eq!(return_line(&baseline, "6b"), 0);
    assert_eq!(return_line(&report, "2a"), 2_000_000);
    assert_eq!(return_line(&report, "6b"), 250_000);
    assert_eq!(return_line(&report, "11a / 11b"), 250_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[2].income {
        data.statements.private_activity_interest = 500_000;
    }
    let pab = supported(&draft);
    assert_eq!(return_line(&pab, "2a"), return_line(&report, "2a"));
    assert_eq!(return_line(&pab, "11a / 11b"), return_line(&report, "11a / 11b"));
    assert_eq!(
        value(&pab, "alternative_minimum_taxable_income"),
        value(&report, "alternative_minimum_taxable_income") + 5_000
    );
    assert_eq!(value(&pab, "estate_trust_3_14a_private_activity"), 5_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), pab);

    let mut direct = draft.clone();
    if let FederalIncome::EstateTrustPortfolio { data } = &mut direct.income[2].income {
        let mut data = edit_estate(data);
        data.tax_exempt_interest = 0;
        data.private_activity_interest = 0;
    }
    add(
        &mut direct,
        income(FederalIncome::Interest {
            data: interest_source(native_exempt_interest(2_000_000, 500_000)),
        }),
    );
    let direct_report = supported(&direct);
    assert_eq!(pab.tax, direct_report.tax);
    assert_eq!(
        value(&pab, "alternative_minimum_taxable_income"),
        value(&direct_report, "alternative_minimum_taxable_income")
    );

    if let FederalIncome::EstateTrustPortfolio { data } = &mut draft.income[2].income {
        data.statements.private_activity_interest = 2_000_001;
    }
    assert!(crate::federal_rules::validate_income_amount(&draft.income[2].income).is_err());
}

#[test]
fn estate_trust_allocated_estimated_tax_joins_line_26_once_after_cent_rounding() {
    let mut draft = complete(FederalStatus::Single);
    draft.payments.as_mut().unwrap().estimated = unknown_estimated(49, PaymentPayer::Taxpayer);

    let baseline = supported(&draft);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                allocated_estimated_tax: 100_049,
                form_1041t_timely: true,
                ..empty_estate_portfolio()
            }),
        }),
    );
    let report = supported(&draft);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(return_line(&report, "26"), return_line(&baseline, "26") + 100_100);
    assert_eq!(return_line(&report, "33"), return_line(&baseline, "33") + 100_100);
    assert_eq!(report.refund.unwrap() - baseline.refund.unwrap(), 100_100);
    assert_eq!(line_amount(&report, "estate_trust_2_allocated_estimated_tax"), 100_049);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.form_1041t_timely = false;
    // Without a timely Form 1041-T the payment stays the fiduciary's.
    drop(data);
    let r = supported(&draft);
    assert!(noted(&r, "WARN_ESTATE_TRUST_PAYMENT_LEFT_OUT"));
    assert!(
        r.lines
            .iter()
            .all(|line| line.id != "estate_trust_2_allocated_estimated_tax" || line.amount == 0)
    );
}

#[test]
fn estate_trust_box_5_royalties_use_existing_portfolio_calculations() {
    let mut direct = complete(FederalStatus::Single);
    direct.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(&mut direct, income(royalty(10_000, 0)));
    let direct_report = supported(&direct);

    let mut trust = complete(FederalStatus::Single);
    trust.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(
        &mut trust,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                royalty_income: 1_000_000,
                box_5_royalty_only: true,
                ..empty_estate_portfolio()
            }),
        }),
    );
    let report = supported(&trust);
    for id in ["8", "11a / 11b", "24a", "25d", "33"] {
        assert_eq!(return_line(&report, id), return_line(&direct_report, id));
    }
    assert_eq!(report.tax, direct_report.tax);
    assert_eq!(value(&report, "net_investment_income_tax"), 380);
    assert_eq!(value(&report, "nonbusiness_royalty_income"), 10_000);
    assert_eq!(value(&report, "estate_trust_2_5_royalty_income"), 10_000);
    assert_eq!(crate::interview::document_amount(&trust.income[1].income, false), Some(1_000_000));
    assert_eq!(estimate(&restart(trust.clone()).draft), report);

    // The fiduciary's established straight-line sound-recording schedule
    // supplies one box 9A amount for this nonbusiness royalty activity.
    let mut direct_depreciated = direct.clone();
    let FederalIncome::Royalty { data } = &mut direct_depreciated.income[1].income else {
        unreachable!()
    };
    data.expenses.push(RentalExpense {
        category: RentalExpenseCategory::Other { label: "Established royalty deduction".into() },
        amount: 250_000,
        treatment: BusinessExpenseKind::Ordinary,
    });
    let direct_depreciated_report = supported(&direct_depreciated);
    let mut trust_depreciated = trust.clone();
    let mut data = estate_trust_code_d_data(&mut trust_depreciated);
    data.box_9_royalty_depreciation = 250_000;
    drop(data);
    unsupported(&trust_depreciated, "Income:"); // Sound-recording basis and AMT character are unconfirmed.
    assert_eq!(crate::interview::document_amount(&trust_depreciated.income[1].income, false), None);
    let mut data = estate_trust_code_d_data(&mut trust_depreciated);
    data.box_9_royalty_sound_recording_only = true;
    drop(data);
    let depreciated = supported(&trust_depreciated);
    for id in ["8", "11a / 11b", "24a", "25d", "33"] {
        assert_eq!(return_line(&depreciated, id), return_line(&direct_depreciated_report, id));
    }
    assert_eq!(depreciated.tax, direct_depreciated_report.tax);
    assert_eq!(value(&depreciated, "estate_trust_2_5_royalty_income"), 10_000);
    assert_eq!(value(&depreciated, "estate_trust_2_9a_royalty_depreciation"), 2_500);
    assert_eq!(value(&depreciated, "nonbusiness_royalty_income"), 7_500);
    assert_eq!(value(&depreciated, "agi"), value(&report, "agi") - 2_500);
    assert_eq!(
        value(&depreciated, "alternative_minimum_taxable_income"),
        value(&report, "alternative_minimum_taxable_income") - 2_500
    );
    assert_eq!(value(&depreciated, "net_investment_income_tax"), 285);
    assert!(
        depreciated
            .review_notes
            .iter()
            .any(|note| note.contains("per-asset sound-recording basis"))
    );
    assert_eq!(
        crate::interview::document_amount(&trust_depreciated.income[1].income, false),
        Some(750_000)
    );
    assert_eq!(estimate(&restart(trust_depreciated.clone()).draft), depreciated);

    let mut data = estate_trust_code_d_data(&mut trust_depreciated);
    data.box_9_royalty_depreciation = 1_000_001;
    drop(data);
    unsupported(&trust_depreciated, "Income:"); // Net royalty losses need another calculation.
    let mut data = estate_trust_code_d_data(&mut trust_depreciated);
    data.box_9_royalty_depreciation = 250_000;
    data.box_9_royalty_depletion = 150_000;
    data.box_9_royalty_cost_depletion_only = true;
    drop(data);
    unsupported(&trust_depreciated, "Income:"); // A second box 9 property is not reconciled.
    let mut data = estate_trust_code_d_data(&mut trust_depreciated);
    data.box_9_royalty_depletion = 0;
    data.box_9_royalty_cost_depletion_only = false;
    data.box_9_royalty_amortization = 100_000;
    data.box_9_royalty_amortization_only = true;
    drop(data);
    unsupported(&trust_depreciated, "Income:"); // Section 197 assets need their own source allocation.
    let mut data = estate_trust_code_d_data(&mut trust_depreciated);
    data.box_9_royalty_amortization = 0;
    data.box_9_royalty_amortization_only = false;
    data.niit_adjustment = Some(EstateTrustNiitAdjustment { line_7: 0, modified_agi: 0 });
    drop(data);
    unsupported(&trust_depreciated, "Income:"); // Box 14H could duplicate the deduction.
    let mut data = estate_trust_code_d_data(&mut trust_depreciated);
    data.niit_adjustment = None;
    data.final_k1 = true;
    data.final_year_successor = Some(true);
    drop(data);
    unsupported(&trust_depreciated, "Income:"); // Property succession needs a basis transfer.
    let mut data = estate_trust_code_d_data(&mut trust_depreciated);
    data.final_year_successor = Some(false);
    data.box_9_royalty_sound_recording_only = false;
    drop(data);
    unsupported(&trust_depreciated, "Income:"); // Depreciation method and AMT status are unconfirmed.

    // A fiduciary's single-property cost-depletion allocation is reported
    // separately in box 9B, then deducted once from the same box 5 royalties.
    let mut direct_depleted = direct.clone();
    let FederalIncome::Royalty { data } = &mut direct_depleted.income[1].income else {
        unreachable!()
    };
    data.expenses.push(RentalExpense {
        category: RentalExpenseCategory::Other { label: "Established royalty deduction".into() },
        amount: 150_000,
        treatment: BusinessExpenseKind::Ordinary,
    });
    let direct_depleted_report = supported(&direct_depleted);
    let mut trust_depleted = trust.clone();
    let mut data = estate_trust_code_d_data(&mut trust_depleted);
    data.box_9_royalty_depletion = 150_000;
    drop(data);
    unsupported(&trust_depleted, "Income:"); // Property basis, method and AMT source are unconfirmed.
    assert_eq!(crate::interview::document_amount(&trust_depleted.income[1].income, false), None);
    let mut data = estate_trust_code_d_data(&mut trust_depleted);
    data.box_9_royalty_cost_depletion_only = true;
    drop(data);
    let depleted = supported(&trust_depleted);
    for id in ["8", "11a / 11b", "24a", "25d", "33"] {
        assert_eq!(return_line(&depleted, id), return_line(&direct_depleted_report, id));
    }
    assert_eq!(depleted.tax, direct_depleted_report.tax);
    assert_eq!(value(&depleted, "estate_trust_2_5_royalty_income"), 10_000);
    assert_eq!(value(&depleted, "estate_trust_2_9b_royalty_depletion"), 1_500);
    assert_eq!(value(&depleted, "nonbusiness_royalty_income"), 8_500);
    assert_eq!(value(&depleted, "agi"), value(&report, "agi") - 1_500);
    assert_eq!(
        value(&depleted, "alternative_minimum_taxable_income"),
        value(&report, "alternative_minimum_taxable_income") - 1_500
    );
    assert_eq!(value(&depleted, "net_investment_income_tax"), 323);
    assert!(depleted.review_notes.iter().any(|note| note.contains("per-property depletion basis")));
    assert_eq!(
        crate::interview::document_amount(&trust_depleted.income[1].income, false),
        Some(850_000)
    );
    assert_eq!(estimate(&restart(trust_depleted.clone()).draft), depleted);

    let mut data = estate_trust_code_d_data(&mut trust_depleted);
    data.box_9_royalty_depletion = 1_000_001;
    drop(data);
    unsupported(&trust_depleted, "Income:"); // Net royalty losses need another calculation.
    let mut data = estate_trust_code_d_data(&mut trust_depleted);
    data.box_9_royalty_depletion = 150_000;
    data.box_9_royalty_amortization = 50_000;
    data.box_9_royalty_amortization_only = true;
    drop(data);
    unsupported(&trust_depleted, "Income:"); // A second box 9 activity/method is not reconciled.
    let mut data = estate_trust_code_d_data(&mut trust_depleted);
    data.box_9_royalty_amortization = 0;
    data.box_9_royalty_amortization_only = false;
    data.niit_adjustment = Some(EstateTrustNiitAdjustment { line_7: 0, modified_agi: 0 });
    drop(data);
    unsupported(&trust_depleted, "Income:"); // The deduction may recur in box 14H.
    let mut data = estate_trust_code_d_data(&mut trust_depleted);
    data.niit_adjustment = None;
    data.final_k1 = true;
    data.final_year_successor = Some(true);
    drop(data);
    unsupported(&trust_depleted, "Income:"); // Property succession needs basis tracking.
    let mut data = estate_trust_code_d_data(&mut trust_depleted);
    data.final_year_successor = Some(false);
    data.box_9_royalty_cost_depletion_only = false;
    drop(data);
    unsupported(&trust_depleted, "Income:"); // AMT basis/method may differ.

    // The fiduciary's box 9 code C amount is separate from box 5, but
    // follows the same Schedule E royalty, AMT and NIIT path as a direct
    // expense of the same section 212 income-producing activity.
    let mut direct_deducted = direct.clone();
    let FederalIncome::Royalty { data } = &mut direct_deducted.income[1].income else {
        unreachable!()
    };
    data.expenses.push(RentalExpense {
        category: RentalExpenseCategory::Other { label: "Established royalty amortization".into() },
        amount: 200_000,
        treatment: BusinessExpenseKind::Ordinary,
    });
    let direct_deducted_report = supported(&direct_deducted);
    let mut trust_deducted = trust.clone();
    let mut data = estate_trust_code_d_data(&mut trust_deducted);
    data.box_9_royalty_amortization = 200_000;
    drop(data);
    unsupported(&trust_deducted, "Income:"); // Source attribution is unconfirmed.
    let mut data = estate_trust_code_d_data(&mut trust_deducted);
    data.box_9_royalty_amortization_only = true;
    drop(data);
    let deducted = supported(&trust_deducted);
    for id in ["8", "11a / 11b", "24a", "25d", "33"] {
        assert_eq!(return_line(&deducted, id), return_line(&direct_deducted_report, id));
    }
    assert_eq!(deducted.tax, direct_deducted_report.tax);
    assert_eq!(value(&deducted, "estate_trust_2_5_royalty_income"), 10_000);
    assert_eq!(value(&deducted, "estate_trust_2_9c_royalty_amortization"), 2_000);
    assert_eq!(value(&deducted, "nonbusiness_royalty_income"), 8_000);
    assert_eq!(value(&deducted, "agi"), value(&report, "agi") - 2_000);
    assert_eq!(
        value(&deducted, "alternative_minimum_taxable_income"),
        value(&report, "alternative_minimum_taxable_income") - 2_000
    );
    assert_eq!(value(&deducted, "net_investment_income_tax"), 304);
    assert_eq!(
        crate::interview::document_amount(&trust_deducted.income[1].income, false),
        Some(800_000)
    );
    assert_eq!(estimate(&restart(trust_deducted.clone()).draft), deducted);

    let mut data = estate_trust_code_d_data(&mut trust_deducted);
    data.box_9_royalty_amortization = 1_000_001;
    drop(data);
    unsupported(&trust_deducted, "Income:"); // Net royalty losses remain outside this path.
    let mut data = estate_trust_code_d_data(&mut trust_deducted);
    data.box_9_royalty_amortization = 200_000;
    data.niit_adjustment = Some(EstateTrustNiitAdjustment { line_7: 0, modified_agi: 0 });
    drop(data);
    unsupported(&trust_deducted, "Income:"); // Code H could repeat the deduction.
    let mut data = estate_trust_code_d_data(&mut trust_deducted);
    data.niit_adjustment = None;
    data.box_5_royalty_only = false;
    drop(data);
    unsupported(&trust_deducted, "Income:"); // The activity's royalty character is unconfirmed.

    let mut data = estate_trust_code_d_data(&mut trust);
    data.box_5_royalty_only = false;
    drop(data);
    unsupported(&trust, "Income:");
    assert_eq!(crate::interview::document_amount(&trust.income[1].income, false), None);
}

#[test]
fn estate_trust_final_capital_loss_carryovers_keep_term_and_unused_balance() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                short_gain: 1_000_000,
                long_gain: 1_000_000,
                rate_28_gain: 200_000,
                unrecaptured_gain: 100_000,
                final_short_carry: 400_000,
                final_long_carry: 300_000,
                final_k1: true,
                final_year_successor: Some(true),
                final_carry_same_tax_character: true,
                ..empty_estate_portfolio()
            }),
        }),
    );
    let report = supported(&draft);
    assert_eq!(return_line(&report, "7a"), 1_300_000);
    assert_eq!(value(&report, "net_investment_income_tax"), 494);
    assert_eq!(value(&report, "rate_28_gain"), 0);
    assert_eq!(value(&report, "unrecaptured_section_1250_gain"), 0);
    assert_eq!(value(&report, "capital_loss_carryforward"), 0);
    assert_eq!(value(&report, "estate_trust_2_11c_short_carry"), 4_000);
    assert_eq!(value(&report, "estate_trust_2_11d_long_carry"), 3_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.short_gain = 0;
    data.long_gain = 0;
    data.rate_28_gain = 0;
    data.unrecaptured_gain = 0;
    data.final_short_carry = 500_000;
    data.final_long_carry = 500_000;
    drop(data);
    let loss_only = supported(&draft);
    assert_eq!(return_line(&loss_only, "7a"), -300_000);
    assert_eq!(value(&loss_only, "capital_short_carryforward"), 2_000);
    assert_eq!(value(&loss_only, "capital_long_carryforward"), 5_000);
    assert_eq!(value(&loss_only, "amt_capital_short_carryforward"), 2_000);
    assert_eq!(crate::interview::document_amount(&draft.income[1].income, false), Some(-1_000_000));
    assert_eq!(estimate(&restart(draft.clone()).draft), loss_only);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.final_year_successor = Some(false);
    drop(data);
    unsupported(&draft, "Income:");
    assert_eq!(crate::interview::document_amount(&draft.income[1].income, false), None);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.final_k1 = true;
    data.final_year_successor = Some(true);
    data.final_carry_same_tax_character = false;
    drop(data);
    unsupported(&draft, "Income:");
}

// Instructions for Schedule K-1 (Form 1041) for a Beneficiary, box 11 codes C and D: the
// carryovers go on Schedule D lines 5 and 12, not the filer's own lines 6 and 14.
#[test]
fn estate_trust_final_capital_loss_carryovers_print_on_schedule_d_lines_5_and_12() {
    for (short, long) in [(0, 900_000), (500_000, 0)] {
        let mut draft = complete(FederalStatus::Single);
        draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
        add(
            &mut draft,
            income(FederalIncome::EstateTrustPortfolio {
                data: crate::tests::support::builders::estate_source(
                    FederalEstateTrustPortfolioK1 {
                        final_short_carry: short,
                        final_long_carry: long,
                        final_k1: true,
                        final_year_successor: Some(true),
                        final_carry_same_tax_character: true,
                        ..empty_estate_portfolio()
                    },
                ),
            }),
        );
        let report = supported(&draft);
        assert_eq!(value(&report, "schedule_d_line_5"), -short / 100);
        assert_eq!(value(&report, "schedule_d_line_12"), -long / 100);
        assert_eq!(value(&report, "schedule_d_line_6"), 0);
        assert_eq!(value(&report, "schedule_d_line_14"), 0);
        assert_eq!(value(&report, "schedule_d_line_7"), -short / 100);
        assert_eq!(value(&report, "schedule_d_line_15"), -long / 100);
        assert_eq!(value(&report, "schedule_d_line_16"), -(short + long) / 100);
        assert_eq!(value(&report, "schedule_d_line_21"), -(short + long).min(300_000) / 100);
    }
}

#[test]
fn estate_trust_final_67e_deduction_reduces_agi_without_second_carryover() {
    let mut draft = complete(FederalStatus::Single);
    draft.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    add(
        &mut draft,
        income(FederalIncome::EstateTrustPortfolio {
            data: crate::tests::support::builders::estate_source(FederalEstateTrustPortfolioK1 {
                interest: 1_000_000,
                final_67e_deduction: 500_000,
                final_67e_no_niit_allocation: true,
                final_k1: true,
                final_year_successor: Some(true),
                ..empty_estate_portfolio()
            }),
        }),
    );
    let report = supported(&draft);
    let mut without = draft.clone();
    let mut data = estate_trust_code_d_data(&mut without);
    data.final_67e_deduction = 0;
    drop(data);
    let baseline = supported(&without);
    assert_eq!(return_line(&report, "9"), return_line(&baseline, "9"));
    assert_eq!(return_line(&report, "10"), return_line(&baseline, "10") + 500_000);
    assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b") - 500_000);
    assert_eq!(value(&report, "estate_trust_67e_deduction"), 5_000);
    assert_eq!(schedule_1_line(&report, "24k"), 5_000);
    assert_eq!(schedule_1_line(&report, "26"), 5_000);
    assert_eq!(value(&report, "net_investment_income_tax"), 380);
    assert_eq!(value(&report, "nol_carryforward"), 0);
    assert!(report.tax < baseline.tax);
    assert_eq!(crate::interview::document_amount(&draft.income[1].income, false), Some(500_000));
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let mut unabsorbed = draft.clone();
    unabsorbed.income[0] = wage(1, Owner::Taxpayer, 0, 0);
    let mut data = estate_trust_code_d_data(&mut unabsorbed);
    data.interest = 0;
    data.final_67e_deduction = 1_000_000;
    drop(data);
    let no_income = supported(&unabsorbed);
    assert_eq!(return_line(&no_income, "10"), 1_000_000);
    assert_eq!(value(&no_income, "nol_carryforward"), 0);
    assert_eq!(estimate(&restart(unabsorbed.clone()).draft), no_income);

    let mut data = estate_trust_code_d_data(&mut draft);
    data.final_year_successor = Some(false);
    drop(data);
    unsupported(&draft, "Income:");
    let mut data = estate_trust_code_d_data(&mut draft);
    data.final_k1 = true;
    data.final_year_successor = Some(true);
    data.final_67e_no_niit_allocation = false;
    drop(data);
    unsupported(&draft, "Income:");
    assert_eq!(crate::interview::document_amount(&draft.income[1].income, false), None);
}

#[test]
fn prior_partner_fuel_credit_is_schedule_e_income_without_repeating_current_payment() {
    let mut baseline_draft = complete(FederalStatus::Single);
    let mut ordinary = ordinary_partner_source(20_000, true);
    ordinary.qbi = Some(partner_qbi_statement(20_000));
    add(&mut baseline_draft, income(FederalIncome::Partnership { data: ordinary.clone() }));
    let baseline = supported(&baseline_draft);

    ordinary.prior_fuel_credit = Some(PartnerPriorFuelCredit {
        claimed_credit: 10_000,
        filed_in_return_year: true,
        cash_method: true,
        full_prior_tax_benefit: true,
        partial_prior_year_benefit: None,
        prior_niit_recovery_excluded: true,
        not_reported_elsewhere: true,
    });
    ordinary.fuel_tax_credit = Some(PartnerFuelTaxCredit {
        fuels: FederalFuelLines {
            gasoline_off_highway: Some(fuel_use(100_000, 20_000)),
            ..FederalFuelLines::default()
        }
        .into(),
        statement_complete: true,
        farm_activity: false,
    });
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Partnership { data: ordinary }));
    let report = supported(&draft);
    assert_eq!(return_line(&report, "8"), return_line(&baseline, "8") + 10_000);
    assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b") + 10_000);
    assert_eq!(return_line(&report, "13b"), return_line(&baseline, "13b") + 2_000);
    assert_eq!(return_line(&report, "15"), return_line(&baseline, "15") + 8_000);
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 1_800);
    assert_eq!(value(&report, "partnership_2_prior_fuel_credit_recovery"), 100);
    assert_eq!(value(&report, "partnership_2_fuel_1a"), 18);
    assert_eq!(return_line(&report, "23"), return_line(&baseline, "23"));
    assert_eq!(estimate(&restart(draft.clone()).draft), report);
    let mut high_baseline = baseline_draft;
    high_baseline.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 650_000);
    let mut high_with_recovery = draft.clone();
    high_with_recovery.income[0] = wage(1, Owner::Taxpayer, 30_000_000, 650_000);
    let high_baseline = supported(&high_baseline);
    let high_report = supported(&high_with_recovery);
    assert_eq!(return_line(&high_report, "23"), return_line(&high_baseline, "23"));

    // Answers and unknowns: each confirmation's No proceeds.
    let recovery = |draft: &FederalDraft| {
        let r = supported(draft);
        let line = r
            .lines
            .iter()
            .find(|l| l.id == "partnership_2_prior_fuel_credit_recovery")
            .map(|l| l.amount / 100);
        (r, line)
    };
    let prior = |draft: &mut FederalDraft| {
        let FederalIncome::Partnership { data } = &mut draft.income[1].income else {
            unreachable!()
        };
        data.prior_fuel_credit.as_mut().unwrap().clone()
    };
    let set = |draft: &mut FederalDraft, value: crate::PartnerPriorFuelCredit| {
        let FederalIncome::Partnership { data } = &mut draft.income[1].income else {
            unreachable!()
        };
        data.prior_fuel_credit = Some(value);
    };
    let original = prior(&mut draft);
    // Not claimed on a 2025 return filed in 2026, or an accrual activity:
    // this route does not report it; left out with a notice.
    for (filed_in_return_year, cash_method) in [(false, true), (true, false)] {
        set(
            &mut draft,
            crate::PartnerPriorFuelCredit { filed_in_return_year, cash_method, ..original.clone() },
        );
        let (r, line) = recovery(&draft);
        assert_eq!(line, None);
        assert!(noted(&r, "WARN_PARTNER_FUEL_RECOVERY_LEFT_OUT"), "{r:?}");
    }
    // Already in another entry: not added again, without a notice.
    set(
        &mut draft,
        crate::PartnerPriorFuelCredit { not_reported_elsewhere: false, ..original.clone() },
    );
    let (r, line) = recovery(&draft);
    assert_eq!(line, None);
    assert!(!noted(&r, "WARN_PARTNER_FUEL_RECOVERY_LEFT_OUT"));
    // A partial prior benefit includes the full credit.
    {
        let answer = false;
        let key = "WARN_PARTNER_FUEL_PARTIAL_BENEFIT_LEFT_OUT";
        set(
            &mut draft,
            crate::PartnerPriorFuelCredit { full_prior_tax_benefit: answer, ..original.clone() },
        );
        let (r, line) = recovery(&draft);
        assert_eq!(line, Some(100));
        assert!(noted(&r, key), "{r:?}");
    }
    // A recovery of a deduction that reduced NIIT keeps its regular-tax part.
    set(
        &mut draft,
        crate::PartnerPriorFuelCredit { prior_niit_recovery_excluded: false, ..original.clone() },
    );
    let (r, line) = recovery(&draft);
    assert_eq!(line, Some(100));
    assert!(noted(&r, "WARN_PARTNER_FUEL_NIIT_RECOVERY_LEFT_OUT"), "{r:?}");
    set(&mut draft, original);
    let FederalIncome::Partnership { data } = &mut draft.income[1].income else { unreachable!() };
    data.regular.passive_loss = 100;
    unsupported(&draft, "Partnership passive pool");
}

#[test]
fn partnership_and_scorp_share_passive_pool_without_losing_source_identity() {
    let mut d = complete(FederalStatus::Single);
    let mut source = ordinary_partner_source(-6000, false);
    source.alternative.as_mut().unwrap().ordinary = -300000;
    add(&mut d, income(FederalIncome::Partnership { data: source }));
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(complete_ordinary_scorp(
                4000, false,
            )),
        }),
    );
    let mut report = supported(&complete(FederalStatus::Single));
    let result = crate::extended_rules::rentals(&d, 40000, &mut report);
    assert!(result.inputs_complete, "{:?}", report.issues);
    assert_eq!(result.income, 0);
    assert_eq!(result.passive_loss, 2000);
    assert_eq!(result.scorp_business, 4000);
    assert_eq!(
        result.partners,
        vec![crate::extended_rules::PartnerPassiveResult {
            id: 2,
            net: -4000,
            nonpassive: 0,
            suspended: crate::tests::support::builders::k1_amount(2000),
            prior_released: 0,
        }]
    );
    assert_eq!(value(&report, "partnership_2_passive_suspended"), 2000);
    assert!(!report.lines.iter().any(|line| line.id == "rental_2_suspended_loss"));
    let mut alternative_report = supported(&complete(FederalStatus::Single));
    let alternative = crate::extended_rules::rentals_for(&d, 40000, &mut alternative_report, true);
    assert!(alternative.inputs_complete);
    assert_eq!(alternative.income, 1000);
    assert_eq!(alternative.partners[0].net, -3000);
    assert_eq!(crate::kernel::ratio::dollars(alternative.partners[0].suspended.get()), 0);
    d.income.swap(1, 2);
    let mut reversed_report = supported(&complete(FederalStatus::Single));
    let reversed = crate::extended_rules::rentals(&d, 40000, &mut reversed_report);
    assert_eq!(reversed.income, result.income);
    assert_eq!(reversed.partners, result.partners);
    assert_eq!(restart(d.clone()).draft, d);
    // The partnership K-1 has no box 20 code Z statement: its loss is left
    // out of QBI with a note.
    assert!(noted(&supported(&d), "WARN_K1_QBI_LOSS_NO_STATEMENT"));
}

#[test]
fn nonpassive_partner_does_not_absorb_other_passive_losses_and_amt_uses_regular_magi() {
    let mut d = complete(FederalStatus::Single);
    let mut source = ordinary_partner_source(8000, true);
    source.alternative.as_mut().unwrap().ordinary = 600000;
    add(&mut d, income(FederalIncome::Partnership { data: source }));
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(complete_ordinary_scorp(
                -2000, false,
            )),
        }),
    );
    for alternative in [false, true] {
        let mut report = supported(&complete(FederalStatus::Single));
        report.lines.clear(); // Inspect this pool evaluation, not the seed return's MAGI.
        let pool = crate::extended_rules::rentals_for(&d, 40000, &mut report, alternative);
        assert!(pool.inputs_complete, "{:?}", report.issues);
        assert_eq!(pool.income, if alternative { 6000 } else { 8000 });
        assert_eq!(pool.passive_loss, 2000);
        assert_eq!(pool.partners[0].nonpassive, pool.income);
        assert_eq!(
            value(&report, if alternative { "amt_rental_magi" } else { "rental_magi" }),
            48000
        );
    }
    if let FederalIncome::Partnership { data } = &mut d.income[1].income {
        data.alternative = None;
    }
    let mut report = supported(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals_for(&d, 40000, &mut report, true);
    assert!(!pool.inputs_complete);
    assert!(pool.partners.is_empty());
    assert!(report.issues.iter().any(|issue| issue.section == "Partnership AMT"));
}

#[test]
fn partnership_se_source_is_independent_and_unknown_is_not_zero() {
    let mut source = ordinary_partner_source(8000, true);
    source.limited_for_se = Some(false);
    source.reported_se = Some(750025);
    assert!(source.valid());
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: source.clone() }));
    assert_eq!(restart(d.clone()).draft, d);
    let report = estimate(&d);
    assert!(!report.issues.iter().any(|i| i.section == "Partnership self-employment source"));
    // Source differences survive; they are not silently replaced by box 1.
    assert_ne!(source.reported_se, Some(source.regular.ordinary));
    source.reported_se = None;
    assert!(source.valid()); // Unresolved records remain editable and saveable.
    d.income[1].income = FederalIncome::Partnership { data: source.clone() };
    unsupported(&d, "Partnership self-employment source");
    source.reported_se = Some(-100000000000);
    assert!(source.valid());
}

#[test]
fn partnership_loss_allocation_depends_on_other_rentals_magi_allowance() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(-20000, false);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(-2000000);
    partner.regular.basis.opening = Some(2000000);
    partner.regular.risk.start = ScorpRiskStart::Detailed { capacity: 2000000, profit: 0, loss: 0 };
    partner.alternative = Some(partner.regular.clone());
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let property = rental(0, 20000); // $10,000 annual depreciation: $30,000 total loss.
    add(&mut d, income(FederalIncome::Rental { data: property }));
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(complete_ordinary_scorp(
                10000, false,
            )),
        }),
    );
    // Form 8582 VI removes the rental-only allowance before VII allocates
    // unallowed losses. This changes the partnership's share of passive income,
    // even though its own eligibility for the rental allowance stays false.
    for (magi, expected_partner, expected_total) in [(100000, -8000, -25000), (150000, -4000, 0)] {
        for amt in [false, true] {
            let mut report = supported(&complete(FederalStatus::Single));
            report.lines.clear();
            let result = crate::extended_rules::rentals_for(&d, magi, &mut report, amt);
            assert!(result.inputs_complete, "{:?}", report.issues);
            assert_eq!(result.partners[0].net, expected_partner);
            // AMT carries actual-source earnings to related deductions; it does not file
            // another Schedule SE. Both tax systems retain this resolved passive share.
            assert_eq!(result.partner_se.len(), 1);
            assert_eq!(result.partner_se[0].1.earnings, expected_partner);
            assert_eq!(
                crate::kernel::ratio::dollars(result.partners[0].suspended.get()),
                20000 + expected_partner
            );
            assert_eq!(result.income, expected_total);
            assert_eq!(result.passive_loss, 40000 + expected_total);
        }
    }
    assert_eq!(restart(d.clone()).draft, d);
    // The partnership K-1 has no box 20 code Z statement: its loss is left
    // out of QBI with a note.
    assert!(noted(&supported(&d), "WARN_K1_QBI_LOSS_NO_STATEMENT"));
}

#[test]
fn partner_se_loss_character_survives_each_separate_limit() {
    let mut v = ordinary_partner_source(-1000, false);
    v.limited_for_se = Some(false);
    v.reported_se = Some(-100000);
    v.regular.basis.opening = Some(50000);
    v.regular.basis.prior_loss = 100000;
    v.regular.risk.start = ScorpRiskStart::Detailed { capacity: 25000, profit: 0, loss: 0 };
    v.regular.risk.prior_loss = 50000;
    v.regular.passive_loss = 25000;
    v.se_history = PartnerSeHistory { basis: 0, risk: 25000, passive: 12500 };
    // $2,000 basis loss: $1,000 SE, $500 allowed -> $250 SE.
    // $1,000 risk loss: $500 SE, $250 allowed -> $125 SE.
    // $500 passive loss: $250 SE, $200 allowed -> $100 SE.
    let r = v
        .se_after_passive(
            crate::kernel::carry::Amount::new(300 * 100).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    assert_eq!(r.earnings, -100);
    assert_eq!(r.basis_suspended, 750);
    assert_eq!(r.risk_suspended, 375);
    assert_eq!(r.passive_suspended, 150);
    assert_eq!(r.basis_suspended + r.risk_suspended + r.passive_suspended - r.earnings, 1375);
    assert_eq!(PartnerK1::from_avro(&v.to_avro()).unwrap(), v);
    v.se_history = PartnerSeHistory { basis: 100001, risk: 0, passive: 0 };
    assert!(
        v.se_after_passive(
            crate::kernel::carry::Amount::new(300 * 100).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap()
        )
        .is_err()
    );
}

#[test]
fn partner_se_former_passive_and_limited_partner_keep_historical_character() {
    let mut v = ordinary_partner_source(1000, true);
    v.limited_for_se = Some(false);
    v.reported_se = Some(100000);
    v.regular.passive_loss = 150000;
    v.se_history = PartnerSeHistory { basis: 0, risk: 0, passive: 75000 };
    let r = v
        .se_after_passive(
            crate::kernel::carry::Amount::new(300 * 100).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    assert_eq!(r.earnings, 400); // $1,200 of prior loss released; half retains SE character.
    assert_eq!(r.passive_suspended, 150);
    v.limited_for_se = Some(true);
    v.reported_se = Some(0);
    assert_eq!(
        v.se_after_passive(
            crate::kernel::carry::Amount::new(300 * 100).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap()
        )
        .unwrap()
        .earnings,
        -600
    );
    assert!(
        v.se_after_passive(
            crate::kernel::carry::Amount::new(501 * 100).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap()
        )
        .is_err()
    );
    v.reported_se = Some(100000);
    assert!(
        v.se_after_passive(
            crate::kernel::carry::Amount::new(300 * 100).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap()
        )
        .is_err()
    ); // No guaranteed services in ordinary scope.
}

#[test]
fn partner_owner_se_combines_businesses_once_and_keeps_spouse_separate() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(cash_business(10000)));
    let business_id = d.income.last().unwrap().id;
    let mut partner = ordinary_partner_source(20000, true);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(2000000);
    add(&mut d, income(FederalIncome::Partnership { data: partner.clone() }));
    let partner_id = d.income.last().unwrap().id;
    let mut spouse = income(FederalIncome::Partnership { data: partner.clone() });
    spouse.owner = crate::DocumentOwner::Spouses;
    add(&mut d, spouse);
    let spouse_id = d.income.last().unwrap().id;
    let mut sources = vec![
        (
            partner_id,
            partner
                .se_after_passive(
                    crate::kernel::carry::Amount::new(0).unwrap(),
                    crate::kernel::carry::Amount::new(0).unwrap(),
                )
                .unwrap(),
        ),
        (
            spouse_id,
            partner
                .se_after_passive(
                    crate::kernel::carry::Amount::new(0).unwrap(),
                    crate::kernel::carry::Amount::new(0).unwrap(),
                )
                .unwrap(),
        ),
    ];
    let r =
        crate::partnership::owner_actual_se(&d, Owner::Taxpayer, 18000000, 0, &sources).unwrap();
    assert_eq!((r.profit, r.net, r.tax, r.half), (30000, 27705, 1361, 681));
    assert_eq!(r.attribution.get(&business_id), Some(&227));
    assert_eq!(r.attribution.get(&partner_id), Some(&454));
    assert!(!r.attribution.contains_key(&spouse_id));
    let spouse = crate::partnership::owner_actual_se(&d, Owner::Spouse, 0, 0, &sources).unwrap();
    assert_eq!((spouse.profit, spouse.net, spouse.tax, spouse.half), (20000, 18470, 2826, 1413));
    sources.pop();
    assert!(crate::partnership::owner_actual_se(&d, Owner::Spouse, 0, 0, &sources).is_err());
    sources.push((
        partner_id,
        partner
            .se_after_passive(
                crate::kernel::carry::Amount::new(0).unwrap(),
                crate::kernel::carry::Amount::new(0).unwrap(),
            )
            .unwrap(),
    ));
    assert!(crate::partnership::owner_actual_se(&d, Owner::Taxpayer, 0, 0, &sources).is_err());
}

#[test]
fn partner_owner_se_uses_deductible_loss_and_one_combined_threshold() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(10000)));
    let mut partner = ordinary_partner_source(-20000, false);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(-2000000);
    // Only $10,000 passes basis/risk; $9,600 is then suspended by passive rules.
    let source = partner
        .se_after_passive(
            crate::kernel::carry::Amount::new(9600 * 100).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    assert_eq!(source.earnings, -400);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let id = d.income.last().unwrap().id;
    let r =
        crate::partnership::owner_actual_se(&d, Owner::Taxpayer, 0, 0, &[(id, source)]).unwrap();
    assert_eq!((r.profit, r.net, r.tax, r.half), (9600, 8866, 1356, 678));
    assert_eq!(r.attribution.get(&id), Some(&0));
    d.income[1].income = cash_business(200);
    if let FederalIncome::Partnership { data } = &mut d.income[2].income {
        data.regular.ordinary = 25000;
        data.reported_se = Some(25000);
    }
    let source = if let FederalIncome::Partnership { data } = &d.income[2].income {
        data.se_after_passive(
            crate::kernel::carry::Amount::new(0).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap()
    } else {
        unreachable!()
    };
    let r =
        crate::partnership::owner_actual_se(&d, Owner::Taxpayer, 0, 0, &[(id, source)]).unwrap();
    assert_eq!((r.profit, r.net, r.tax, r.half), (450, 416, 64, 32));
    // Neither source alone reaches $400 net, but together they do.
}

#[test]
fn partner_without_qbi_statement_adds_no_qbi_after_se_projection() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(10000)));
    let mut partner = ordinary_partner_source(20000, true);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(2000000);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let report = estimate(&d);
    assert!(noted(&report, "WARN_K1_QBI_NO_STATEMENT"), "{report:?}");
    assert!(!report.issues.iter().any(|issue| issue.section == "Partnership QBI statement"));
    assert!(report.review_notes.iter().any(|note| {
        note.contains("actual-method SE projection for taxpayer")
            && note.contains("combined profit $30000")
            && note.contains("SE tax $4239")
            && note.contains("actual-method alternative")
            && note.contains(
                "Additional partnership items require their own source records and review.",
            )
    }));
    let actual_note = &report
        .review_notes
        .iter()
        .find(|note| note.contains("actual-method SE projection for taxpayer"))
        .unwrap()
        .display;
    assert_eq!(actual_note.key, "NOTE_PARTNERSHIP_SE_ACTUAL_TAXPAYER");
    assert_eq!(actual_note.args[0], MessageArg::usd_dollars(30_000));
    assert_eq!(actual_note.args[2], MessageArg::usd_dollars(4_239));
    assert_eq!(estimate(&restart(d).draft), report);
}

#[test]
fn partnership_se_coordinator_reconciles_health_magi_and_passive_loss() {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 12500000, 0);
    add(&mut d, income(health_business(10000, 10000, false)));
    let mut partner = ordinary_partner_source(-20000, false);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(-2000000);
    partner.regular.basis.opening = Some(2000000);
    partner.regular.risk.start = ScorpRiskStart::Detailed { capacity: 2000000, profit: 0, loss: 0 };
    partner.alternative = Some(partner.regular.clone());
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    add(&mut d, income(FederalIncome::Rental { data: rental(0, 20000) }));
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(complete_ordinary_scorp(
                10000, false,
            )),
        }),
    );
    let report = estimate(&d);
    // Fixed-point worksheet: MAGI $125,332 -> allowance $12,334 ->
    // partner suspended $14,690 -> SE profit $4,690 -> net $4,331 ->
    // tax $663, half $332 -> C health $9,668 -> same MAGI $125,332.
    assert!(
        report.review_notes.iter().any(|note| note.contains("coordination stabilized")
            && note.contains("SE tax $663")
            && note.contains("half-SE deduction $332")
            && note.contains("health deduction $9668")
            && note.contains("rental MAGI base $125332")),
        "{:?}",
        report.review_notes
    );
    let coordination = &report
        .review_notes
        .iter()
        .find(|note| note.contains("coordination stabilized"))
        .unwrap()
        .display;
    assert_eq!(coordination.key, "NOTE_PARTNERSHIP_SE_COORDINATION");
    assert_eq!(coordination.args[0], MessageArg::usd_dollars(663));
    assert_eq!(coordination.args[2], MessageArg::usd_dollars(9_668));
    assert_eq!(
        report.review_notes.iter().filter(|n| n.contains("coordination stabilized")).count(),
        1
    );
    assert!(!report.issues.iter().any(|i| i.section == "Partnership deduction coordination"));
    assert_eq!(estimate(&restart(d).draft), report);
    assert!(report.tax.is_some());
    assert!(noted(&report, "WARN_QBI_GROSS_ALLOCATION_LEFT_OUT"));
}

#[test]
fn partnership_se_allocation_does_not_charge_c_health_with_partner_half_tax() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(health_business(1000, 1000, false)));
    let mut partner = ordinary_partner_source(3000, true);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(300000);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = estimate(&d);
    // Owner half-SE $283: C $71, partnership $212. C deduction is $929,
    // not $717 obtained by charging the entire owner's half-SE to C.
    assert!(
        r.review_notes.iter().any(|note| note.contains("coordination stabilized")
            && note.contains("SE tax $565")
            && note.contains("half-SE deduction $283")
            && note.contains("health deduction $929")),
        "{:?}",
        r.review_notes
    );
    d.income[1].income = sep_business(10000, 1800);
    if let FederalIncome::Partnership { data } = &mut d.income[2].income {
        data.regular.ordinary = 2000000;
        data.alternative = Some(data.regular.clone());
        data.reported_se = Some(2000000);
    }
    let r = estimate(&d);
    assert!(!r.issues.iter().any(|i| i.section == "Business retirement"), "{:?}", r.issues);
    assert!(r.review_notes.iter().any(|note| note.contains("coordination stabilized")
        && note.contains("retirement plan deduction $1800")));
}

#[test]
fn partnership_ordinary_losses_enter_separate_regular_and_amt_business_limits() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(-300000, true);
    partner.regular.basis.opening = Some(30000000);
    partner.regular.risk.start =
        ScorpRiskStart::Detailed { capacity: 30000000, profit: 0, loss: 0 };
    let mut amt = partner.regular.clone();
    amt.ordinary = -28000000;
    partner.alternative = Some(amt);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("regular excess-business-loss worksheet")
            && n.contains("business net $-300000")
            && n.contains("excess business loss $44000")),
        "{:?}",
        r.review_notes
    );
    assert!(r.review_notes.iter().any(|n| n.contains("AMT excess-business-loss worksheet")
        && n.contains("business net $-280000")
        && n.contains("excess business loss $24000")));
    assert!(r.tax.is_none());
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn partnership_business_income_offsets_c_loss_without_being_se_income() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(-300000)));
    let partner = ordinary_partner_source(50000, true); // Established limited-for-SE classification.
    assert_eq!(partner.reported_se, Some(0));
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.contains("regular excess-business-loss worksheet")
        && n.contains("business net $50000")
        && n.contains("excess business loss $0")));
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("coordination stabilized") && n.contains("SE tax $0"))
    );
    // The partnership K-1 has no box 20 code Z statement: it adds no QBI,
    // so only the Schedule C loss reaches the QBI calculation.
    assert!(noted(&r, "WARN_K1_QBI_NO_STATEMENT"), "{r:?}");
    assert!(r.tax.is_some(), "{:?}", r.issues);
}

#[test]
fn partner_operating_loss_is_not_mistaken_for_single_scorp_property_loss() {
    let mut d = property_excess_draft(400000, 300000, 500000, 400000, FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: ordinary_partner_source(-10000, true) }));
    // The property-only allocation shortcut cannot allocate the entire combined
    // deferral to property when a separate partnership has an operating loss.
    unsupported(&d, "Property excess business loss");
}

#[test]
fn partnership_eic_investment_excludes_nonpassive_and_earned_amounts_once() {
    for (active, reported, expected) in
        [(true, 0, 0), (true, 8000, 0), (false, 0, 8000), (false, 3000, 5000), (false, 8000, 0)]
    {
        let mut d = complete(FederalStatus::Single);
        let mut partner = ordinary_partner_source(8000, active);
        if reported != 0 {
            partner.limited_for_se = Some(false);
            partner.reported_se = Some(reported * 100);
        }
        add(&mut d, income(FederalIncome::Partnership { data: partner }));
        let mut report = supported(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals(&d, 40000, &mut report);
        assert!(pool.inputs_complete, "{:?}", report.issues);
        assert_eq!(pool.income, 8000);
        assert_eq!(pool.income + pool.partner_niit_adjustment, expected);
        assert_eq!(pool.eic_passive_income(), expected, "active {active}, SE {reported}");
        assert_eq!(pool.partner_se[0].1.passive_earnings, if active { 0 } else { reported });
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn partnership_eic_investment_does_not_deduct_se_losses_twice() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(-2000, false);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(-100000); // Half of the ordinary loss has SE character.
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(complete_ordinary_scorp(
                1000, false,
            )),
        }),
    );
    let mut report = supported(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 40000, &mut report);
    assert!(pool.inputs_complete, "{:?}", report.issues);
    assert_eq!(pool.income, 0); // $1,000 of partner loss allowed against S-corp income.
    assert_eq!(pool.partner_se[0].1.earnings, -500);
    assert_eq!(pool.partner_se[0].1.passive_earnings, -500);
    assert_eq!(pool.eic_passive_income(), 500); // Other $500 loss already reduces earned income.
    assert_eq!(pool.income + pool.partner_niit_adjustment, 500);
}

#[test]
fn partnership_trading_classification_is_independent_of_participation_and_se() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(8000, false);
    partner.trade_kind = ScorpTradeKind::FinancialTrading;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(complete_ordinary_scorp(
                -2000, false,
            )),
        }),
    );
    let mut report = supported(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 40000, &mut report);
    assert!(pool.inputs_complete, "{:?}", report.issues);
    assert_eq!(pool.income, 8000); // Trading profit does not release other passive losses.
    assert_eq!(pool.partners[0].nonpassive, 8000);
    assert_eq!(pool.passive_loss, 2000);
    assert_eq!(pool.eic_passive_income(), 0);
    assert_eq!(pool.income + pool.partner_niit_adjustment, 8000); // But it remains NII.
    assert_eq!(restart(d.clone()).draft, d);
    let result = estimate(&d);
    assert!(result.review_notes.iter().any(|n| n.contains("Partnership document")
        && n.contains("investment income after activity limits: $8000")));
    // Zero QBI loss tracking beside the S-corporation statement is a value:
    // its loss has no QBI portion, and the return computes.
    assert!(result.tax.is_some(), "{:?}", result.issues);
    if let FederalIncome::Partnership { data } = &mut d.income[1].income {
        data.limited_for_se = Some(false);
        data.reported_se = Some(800000);
    }
    let mut report = supported(&complete(FederalStatus::Single));
    assert!(!crate::extended_rules::rentals(&d, 40000, &mut report).inputs_complete);
}

#[test]
fn partnership_current_qbi_uses_allocated_half_se_and_shared_deduction_engine() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(10000)));
    let mut partner = ordinary_partner_source(20000, true);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(2000000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 2000000 });
    partner.qbi = Some(partner_qbi_statement(20000));
    partner.qbi_records.qualified_se_gross = 2000000;
    add(&mut d, income(FederalIncome::Partnership { data: partner.clone() }));
    let id = d.income.last().unwrap().id;
    let se = partner
        .se_after_passive(
            crate::kernel::carry::Amount::new(0).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    let owner =
        crate::partnership::owner_actual_se(&d, Owner::Taxpayer, 6000000, 0, &[(id, se)]).unwrap();
    assert_eq!(owner.half, 2120);
    let half = owner.attribution[&id];
    assert_eq!(half, 1413);
    let activity = partner
        .qbi
        .as_ref()
        .unwrap()
        .activity(
            &partner,
            crate::kernel::carry::Amount::new(0).unwrap(),
            half,
            70000,
            FederalStatus::Single,
        )
        .unwrap();
    assert_eq!(activity.income, 18587);
    let mut report = supported(&complete(FederalStatus::Single));
    assert_eq!(
        crate::business_rules::qbi(&[activity], 70000, 0, FederalStatus::Single, &mut report).0,
        3717
    );
    let result = estimate(&d);
    assert!(
        result
            .review_notes
            .iter()
            .any(|n| n.contains(&alloc::format!("Partnership document {id}, current QBI"))
                && n.contains("half-SE deduction $1413")
                && n.contains("eligibility: $18587")),
        "{:?}",
        result.review_notes
    );
    let qbi_note = &result
        .review_notes
        .iter()
        .find(|n| n.contains(&alloc::format!("Partnership document {id}, current QBI")))
        .unwrap()
        .display;
    assert_eq!(qbi_note.key, "NOTE_PARTNERSHIP_CURRENT_QBI");
    assert_eq!(qbi_note.args[0], MessageArg::Number(id));
    assert_eq!(qbi_note.args[1], MessageArg::usd_dollars(1413));
    assert_eq!(qbi_note.args[2], MessageArg::usd_dollars(18587));
    assert_eq!(estimate(&restart(d).draft), result);
    assert!(result.tax.is_some(), "{:?}", result.issues);
}

#[test]
fn partnership_qbi_applies_sstb_once_and_wage_property_limits() {
    let mut partner = ordinary_partner_source(100000, true);
    let mut statement = partner_qbi_statement(100000);
    statement.wages = 2000000;
    statement.property = 10000000;
    let (base, range) = (201750, 75000);
    let taxable = base + range / 2;
    statement.service = true;
    partner.qbi = Some(statement.clone());
    let activity = statement
        .activity(
            &partner,
            crate::kernel::carry::Amount::new(0).unwrap(),
            1000,
            taxable,
            FederalStatus::Single,
        )
        .unwrap();
    assert_eq!(activity.income, 49500);
    assert_eq!(activity.wages, 10000);
    assert_eq!(activity.property, 50000);
    assert_eq!(activity.service, Some(false)); // Shared engine must not phase SSTB twice.
    let mut report = supported(&complete(FederalStatus::Single));
    assert_eq!(
        crate::business_rules::qbi(&[activity], taxable, 0, FederalStatus::Single, &mut report).0,
        7450
    );
    statement.service = false;
    let activity = statement
        .activity(
            &partner,
            crate::kernel::carry::Amount::new(0).unwrap(),
            0,
            400000,
            FederalStatus::Single,
        )
        .unwrap();
    assert_eq!(
        crate::business_rules::qbi(&[activity], 400000, 0, FederalStatus::Single, &mut report).0,
        10000
    );
}

#[test]
fn partnership_qbi_missing_statement_loss_history_and_mixed_se_are_explicit() {
    // No box 20 code Z statement: the K-1 adds no QBI, with a note.
    let mut d = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(8000, true);
    add(&mut d, income(FederalIncome::Partnership { data: partner.clone() }));
    let r = estimate(&d);
    assert!(noted(&r, "WARN_K1_QBI_NO_STATEMENT"), "{r:?}");
    assert!(!r.issues.iter().any(|i| i.section == "Partnership QBI statement"));
    let mut statement = partner_qbi_statement(4000);
    assert!(
        statement
            .activity(
                &partner,
                crate::kernel::carry::Amount::new(0).unwrap(),
                0,
                50000,
                FederalStatus::Single
            )
            .is_ok()
    );
    statement.income = 800000;
    partner.regular.passive_loss = 100000;
    assert!(
        statement
            .activity(
                &partner,
                crate::kernel::carry::Amount::new(0).unwrap(),
                0,
                50000,
                FederalStatus::Single
            )
            .err()
            .unwrap()
            .1
            .contains("historical")
    );
    partner.regular.passive_loss = 0;
    statement.service = true;
    assert!(
        statement
            .activity(
                &partner,
                crate::kernel::carry::Amount::new(0).unwrap(),
                0,
                400000,
                FederalStatus::Single
            )
            .is_ok()
    );
    statement.income = 900000;
    assert!(!statement.valid(&partner));
}

#[test]
fn partnership_qbi_current_loss_passes_three_limits_and_preserves_carries() {
    let mut partner = ordinary_partner_source(-10000, false);
    partner.regular.basis.opening = Some(600000);
    partner.regular.risk.start = ScorpRiskStart::Detailed { capacity: 300000, profit: 0, loss: 0 };
    let statement = partner_qbi_statement(-6000);
    let result = statement
        .calculate(
            &partner,
            crate::tests::support::builders::k1_amount(2000),
            0,
            50000,
            FederalStatus::Single,
        )
        .unwrap();
    assert_eq!(result.current.income, -600);
    assert_eq!(result.historical, 0);
    for (carry, (amount, qualified)) in
        result.carries.iter().zip([(4000, 2400), (3000, 1800), (2000, 1200)])
    {
        assert_eq!(carry.remaining.len(), 1);
        let year = &carry.remaining[0];
        assert_eq!(
            (
                year.year,
                crate::kernel::ratio::dollars(year.original.get()),
                crate::kernel::ratio::dollars(year.original_qbi.get()),
                crate::kernel::ratio::dollars(year.remaining.get()),
                crate::kernel::ratio::dollars(year.remaining_qbi.get())
            ),
            (2026, amount, qualified, amount, qualified)
        );
    }
}

#[test]
fn partnership_qbi_pre_2018_basis_loss_releases_before_qualified_current_loss() {
    let mut partner = ordinary_partner_source(-1000, true);
    partner.regular.basis.opening = Some(150000);
    partner.regular.basis.prior_loss = 100000;
    let mut statement = partner_qbi_statement(-1000);
    partner.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2017, 1000, 0, 1000, 0)],
        risk: vec![],
        passive: vec![],
    });
    let result = statement
        .calculate(
            &partner,
            crate::tests::support::builders::k1_amount(0),
            0,
            50000,
            FederalStatus::Single,
        )
        .unwrap();
    assert_eq!(result.current.income, -500);
    assert_eq!(result.historical, 0);
    assert_eq!(crate::kernel::ratio::dollars(result.carries[0].qbi.get()), 500);
    assert_eq!(
        crate::kernel::ratio::dollars(
            result.carries[0].remaining.iter().map(|v| v.remaining.get()).sum::<i64>()
        ),
        500
    );
    // Loss-year SSTB qualification is fixed before basis release and retained.
    statement.service = true;
    let result = statement
        .calculate(
            &partner,
            crate::tests::support::builders::k1_amount(0),
            0,
            239250,
            FederalStatus::Single,
        )
        .unwrap();
    assert_eq!(result.current.income, -250);
    assert_eq!(
        crate::kernel::ratio::dollars(
            result.carries[0].remaining.last().unwrap().remaining_qbi.get()
        ),
        250
    );
}

#[test]
fn partnership_historical_qbi_is_not_rephased_with_current_sstb_income() {
    let mut partner = ordinary_partner_source(1000, true);
    partner.regular.passive_loss = 150000;
    let mut statement = partner_qbi_statement(1000);
    statement.service = true;
    partner.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![],
        risk: vec![],
        passive: vec![loss_year(2020, 1500, 600, 1500, 600)],
    });
    let result = statement
        .calculate(
            &partner,
            crate::tests::support::builders::k1_amount(300),
            0,
            400000,
            FederalStatus::Single,
        )
        .unwrap();
    assert_eq!(result.current.income, 0); // Current SSTB eligibility is fully phased out.
    assert_eq!(result.historical, 480); // $1,200 historical release retains 40% qualification.
    assert_eq!(crate::kernel::ratio::dollars(result.carries[2].remaining[0].remaining.get()), 300);
    assert_eq!(
        crate::kernel::ratio::dollars(result.carries[2].remaining[0].remaining_qbi.get()),
        120
    );
    let mut d = complete(FederalStatus::Single);
    partner.qbi = Some(statement);
    partner.se_history = PartnerSeHistory { basis: 0, risk: 0, passive: 0 };
    partner.alternative = Some(partner.regular.clone());
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    assert_eq!(restart(d.clone()).draft, d);
    let result = estimate(&d);
    assert!(
        result.review_notes.iter().any(|n| n.contains("released historical qualified loss: $400")),
        "{:?}",
        result.review_notes
    );
    assert!(
        result.review_notes.iter().any(|n| n.contains("2020 passive-loss carryforward: $500.00")
            && n.contains("QBI portion $200"))
    );
    assert!(result.tax.is_some(), "{:?}", result.issues);
}

#[test]
fn partnership_qbi_history_must_reconcile_each_limit_and_pre_2018_character() {
    let mut partner = ordinary_partner_source(0, true);
    partner.regular.basis.prior_loss = 100000;
    let statement = partner_qbi_statement(0);
    partner.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2017, 1000, 100, 1000, 100)],
        risk: vec![],
        passive: vec![],
    });
    assert!(
        statement
            .calculate(
                &partner,
                crate::tests::support::builders::k1_amount(0),
                0,
                50000,
                FederalStatus::Single
            )
            .is_err()
    );
    partner.qbi_records.history.as_mut().unwrap().basis = vec![loss_year(2020, 500, 500, 500, 500)];
    assert!(
        statement
            .calculate(
                &partner,
                crate::tests::support::builders::k1_amount(0),
                0,
                50000,
                FederalStatus::Single
            )
            .is_err()
    );
    partner.qbi_records.history.as_mut().unwrap().basis =
        vec![loss_year(2020, 1000, 500, 1000, 500)];
    let result = statement
        .calculate(
            &partner,
            crate::tests::support::builders::k1_amount(0),
            0,
            50000,
            FederalStatus::Single,
        )
        .unwrap();
    assert_eq!(result.current.income, 0);
    assert_eq!(result.historical, 500);
}

#[test]
fn partnership_qbi_risk_and_passive_histories_each_release_oldest_first() {
    let mut partner = ordinary_partner_source(0, false);
    partner.regular.basis.prior_loss = 100000;
    partner.regular.risk.prior_loss = 100000;
    partner.regular.risk.start = ScorpRiskStart::Detailed { capacity: 150000, profit: 0, loss: 0 };
    partner.regular.passive_loss = 100000;
    let statement = partner_qbi_statement(0);
    partner.qbi_records.history = Some(PartnerQbiHistory {
        basis: vec![loss_year(2020, 1000, 500, 1000, 500)],
        risk: vec![loss_year(2019, 1000, 1000, 1000, 1000)],
        passive: vec![loss_year(2017, 1000, 0, 1000, 0)],
    });
    let result = statement
        .calculate(
            &partner,
            crate::tests::support::builders::k1_amount(1000),
            0,
            50000,
            FederalStatus::Single,
        )
        .unwrap();
    assert_eq!(result.current.income, 0);
    assert_eq!(crate::kernel::ratio::dollars(result.carries[0].qbi.get()), 500);
    assert_eq!(crate::kernel::ratio::dollars(result.carries[1].qbi.get()), 1250); // Old risk QBI first, then half of new basis release.
    assert_eq!(
        crate::kernel::ratio::dollars(
            result.carries[1].remaining.last().unwrap().remaining_qbi.get()
        ),
        250
    );
    assert_eq!(result.historical, 417); // Old passive loss is non-QBI; new release is 5/6 qualified.
    assert_eq!(
        crate::kernel::ratio::dollars(
            result.carries[2].remaining.last().unwrap().remaining_qbi.get()
        ),
        833
    );
    assert_eq!(
        crate::kernel::ratio::dollars(result.carries[2].remaining.last().unwrap().remaining.get()),
        1000
    );
}

#[test]
fn mixed_partner_qbi_half_se_uses_gross_sources_instead_of_net_ratio() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(cash_business(10000)));
    let business_id = d.income.last().unwrap().id;
    let mut partner = ordinary_partner_source(20000, true);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(2000000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 9000000 });
    let statement = partner_qbi_statement(5000);
    partner.qbi_records.qualified_se_gross = 3000000;
    partner.qbi = Some(statement.clone());
    add(&mut d, income(FederalIncome::Partnership { data: partner.clone() }));
    let id = d.income.last().unwrap().id;
    let allocation = crate::partnership_qbi::half_se_by_gross(&d, [2120, 0]).unwrap().sources;
    assert_eq!(allocation[&business_id], 212);
    assert_eq!(allocation[&id], 636); // 30k of total100k gross; neither net20k nor QBI5k is a weight.
    assert_eq!(
        statement
            .calculate(
                &partner,
                crate::tests::support::builders::k1_amount(0),
                allocation[&id],
                50000,
                FederalStatus::Single
            )
            .unwrap()
            .current
            .income,
        4364
    );
    let result = estimate(&d);
    assert!(
        result
            .review_notes
            .iter()
            .any(|n| n.contains(&alloc::format!("Partnership document {id}, current QBI"))
                && n.contains("half-SE deduction $636")
                && n.contains("eligibility: $4364")),
        "{:?}",
        result.review_notes
    );
    assert_eq!(estimate(&restart(d.clone()).draft), result);
    d.income.reverse();
    assert_eq!(
        crate::partnership_qbi::half_se_by_gross(&d, [2120, 0]).unwrap().sources,
        allocation
    );
}

#[test]
fn partner_qbi_gross_allocation_requires_gross_and_a_fitting_qualified_part() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(8000, true);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(800000);
    partner.qbi = Some(partner_qbi_statement(4000));
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    assert!(crate::partnership_qbi::half_se_by_gross(&d, [500, 0]).is_err());
    if let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income {
        data.se_gross = Some(PartnerSeGross::Nonfarm { gross: 1000000 });
    }
    // A qualified part of zero allocates none of the deduction to QBI.
    let id = d.income.last().unwrap().id;
    assert_eq!(crate::partnership_qbi::half_se_by_gross(&d, [500, 0]).unwrap().sources[&id], 0);
    if let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income {
        data.qbi_records.qualified_se_gross = 1000001;
    }
    assert!(crate::partnership_qbi::half_se_by_gross(&d, [500, 0]).is_err());
    assert!(crate::partnership_qbi::half_se_by_gross(&d, [0, 0]).is_ok()); // No deduction to allocate.
}

#[test]
fn partner_personal_health_premiums_reduce_agi_and_qbi_without_reducing_se() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_health_source(6000, false) }));
    let id = d.income.last().unwrap().id;
    let report = estimate(&d);
    assert!(
        report.review_notes.iter().any(|n| n.contains("coordination stabilized")
            && n.contains("SE tax $2826")
            && n.contains("health deduction $6000")),
        "{:?}",
        report.review_notes
    );
    assert!(report.review_notes.iter().any(|n| {
        n.contains(&alloc::format!("Partnership document {id}, current QBI"))
            && n.contains("eligibility: $12587")
    }));
    assert_eq!(estimate(&restart(d.clone()).draft), report);
    assert!(report.tax.is_some(), "{:?}", report.issues);
    if let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income {
        data.health_insurance = BusinessHealthInsurance::None;
    }
    assert!(estimate(&d).review_notes.iter().any(|n| n.contains("coordination stabilized")
        && n.contains("SE tax $2826")
        && n.contains("health deduction $0")));
}

#[test]
fn partner_health_income_cap_and_employer_months_feed_medical_remainder() {
    for (employer, deduction, medical) in [(false, 18587, 641300), (true, 0, 2500000)] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Partnership { data: partner_health_source(25000, employer) }),
        );
        let id = d.income.last().unwrap().id;
        let halves = alloc::collections::BTreeMap::from([(id, 1413)]);
        let earnings = alloc::collections::BTreeMap::from([(id, 20000)]);
        let mut report = supported(&complete(FederalStatus::Single));
        let result = crate::health::calculate(
            &d,
            [1413, 0],
            Some(&halves),
            &earnings,
            &Default::default(),
            &Default::default(),
            None,
            &mut report,
        );
        assert_eq!(result.deduction, deduction);
        assert_eq!(result.medical, medical);
        assert_eq!(result.by_business[&id], deduction);
    }
    // Form 6251 Related Adjustments and IRC 162(l)(2)(A): refigure the
    // source's earned-income limit, retaining the actual half-SE deduction.
    // The existing $20,000 source has $2,826 SE tax and $1,413 half-SE;
    // $25,000 premiums deduct $18,587 regularly. Its independent $10,000
    // AMT business source permits $8,587, so the related adjustment is $10,000.
    // https://www.irs.gov/instructions/i6251
    let mut d = complete(FederalStatus::Single);
    let mut partner = partner_health_source(25_000, false);
    partner.alternative.as_mut().unwrap().ordinary = 1_000_000;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let report = supported(&d);
    assert_eq!(value(&report, "schedule_1_self_employed_health_insurance"), 18_587);
    assert_eq!(value(&report, "amt_self_employed_health_deduction"), 8_587);
    assert_eq!(value(&report, "amt_related_adjustments"), 10_000);
    assert_eq!(value(&report, "self_employment_tax"), 2_826);
    assert_eq!(value(&report, "schedule_1_half_self_employment_tax"), 1_413);
    assert!(!noted(&report, "NOTE_AMT_EARNED_BENEFIT_SOURCE"));
    assert_eq!(estimate(&restart(d).draft), report);
}

#[test]
fn partner_health_mixed_qbi_uses_qualified_gross_share_once() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = partner_health_source(6000, false);
    partner.qbi.as_mut().unwrap().income = 1000000;
    partner.qbi_records.qualified_se_gross = 1000000;
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = estimate(&d);
    // Half-SE1413 -> qualified707; insurance6000 -> qualified3000.
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("current QBI") && n.contains("eligibility: $6293")),
        "{:?}",
        r.review_notes
    );
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("coordination stabilized") && n.contains("health deduction $6000"))
    );
    assert!(!r.issues.iter().any(|v| v.section == "Partner health QBI allocation"));
}

#[test]
fn partner_sep_coordinates_health_qbi_and_keeps_se_tax_unchanged() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = partner_health_source(6000, false);
    partner.pensions = 300000;
    partner.retirement = PartnerRetirement::Sep { rate: 25.0 };
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("coordination stabilized")
            && n.contains("SE tax $2826")
            && n.contains("health deduction $6000")
            && n.contains("retirement plan deduction $3000")),
        "{:?}",
        r.review_notes
    );
    assert!(r.review_notes.iter().any(|n| n.contains("current QBI")
        && n.contains("eligibility: $9587")
        && n.contains("retirement deduction $3000")));
    assert_eq!(estimate(&restart(d).draft), r);
    assert!(r.tax.is_some(), "{:?}", r.issues);
}

// Pub. 560 (2026) chapters 3 to 5 and the Schedule 1 line 16 instructions: a partner's own
// SIMPLE IRA, solo 401(k) or defined benefit plan on this partnership's SE earnings
// ($19,999.99, half SE tax $1,413, so $18,586.99, printed as $18,587).
fn partner_plan_deduction(plan: BusinessRetirement) -> i64 {
    let mut d = complete(FederalStatus::Single);
    let mut partner = partner_health_source(0, false);
    partner.reported_se = Some(1999999);
    partner.pensions = 1;
    partner.retirement = PartnerRetirement::Plan(plan);
    let se = partner
        .se_after_passive(
            crate::kernel::carry::Amount::new(0).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let id = d.income.last().unwrap().id;
    let halves = alloc::collections::BTreeMap::from([(id, 1413)]);
    let mut r = supported(&complete(FederalStatus::Single));
    let result = crate::sep::calculate(&d, [1413, 0], Some(&halves), &[(id, se)], &mut r);
    assert!(!r.issues.iter().any(|v| v.section == "Business retirement"));
    // 2026-10-19: a partner's plan rows are labelled as the partnership's.
    assert!(
        r.lines.iter().any(|l| l.id == alloc::format!("partnership_{id}_owner_plan_deduction"))
    );
    result.by_business[&id]
}

#[test]
fn partner_own_plans_deduct_like_the_business_plans() {
    let qualified = |deferral| BusinessRetirement::Qualified {
        rate: 0.0,
        contribution: 0,
        deferral,
        catch_up: 0,
        roth: 0,
    };
    assert_eq!(partner_plan_deduction(qualified(500_000)), 5000);
    // Deferrals cannot exceed the plan earnings (worksheet line 14).
    assert_eq!(partner_plan_deduction(qualified(2_450_000)), 18587);
    assert_eq!(
        partner_plan_deduction(BusinessRetirement::Simple {
            deferral: 1_000_000,
            employer: SimpleEmployer::Match { rate: 3.0 },
            employer_contribution: 40_000,
        }),
        10_000 + 400
    );
    // The actuary's amount is a source fact, limited only to the earnings.
    let defined = |contribution| BusinessRetirement::DefinedBenefit { contribution };
    assert_eq!(partner_plan_deduction(defined(1_000_000)), 10_000);
    assert_eq!(partner_plan_deduction(defined(5_000_000)), 18587);
}

#[test]
fn partner_sep_exact_compensation_cents_do_not_authorize_an_excess_deposit() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = partner_health_source(0, false);
    partner.reported_se = Some(1999999);
    partner.pensions = 371739;
    partner.retirement = PartnerRetirement::Sep { rate: 25.0 };
    let se = partner
        .se_after_passive(
            crate::kernel::carry::Amount::new(0).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    assert_eq!(se.earnings, 20000); // Tax-line rounding is not plan deposit authority.
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let id = d.income.last().unwrap().id;
    let halves = alloc::collections::BTreeMap::from([(id, 1413)]);
    let mut r = supported(&complete(FederalStatus::Single));
    let result = crate::sep::calculate(&d, [1413, 0], Some(&halves), &[(id, se.clone())], &mut r);
    assert_eq!(result.by_business[&id], 3717);
    assert!(!r.issues.iter().any(|v| v.section == "Business retirement"));
    if let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income {
        data.pensions += 1;
    }
    let mut r = supported(&complete(FederalStatus::Single));
    crate::sep::calculate(&d, [1413, 0], Some(&halves), &[(id, se)], &mut r);
    assert!(r.issues.iter().any(|v| v.section == "Business retirement"));
}

#[test]
fn partner_sep_reduces_the_shared_health_income_limit() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = partner_health_source(25000, false);
    partner.pensions = 300000;
    partner.retirement = PartnerRetirement::Sep { rate: 25.0 };
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("coordination stabilized")
            && n.contains("health deduction $15587")
            && n.contains("retirement plan deduction $3000")),
        "{:?}",
        r.review_notes
    );
    assert!(
        r.review_notes.iter().any(|n| n.contains("current QBI") && n.contains("eligibility: $0"))
    );
}

#[test]
fn partner_sep_multiple_plans_and_unresolved_compensation_remain_explicit() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = partner_health_source(0, false);
    partner.pensions = 10000;
    partner.retirement = PartnerRetirement::Sep { rate: 25.0 };
    let se = partner
        .se_after_passive(
            crate::kernel::carry::Amount::new(0).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let id = d.income.last().unwrap().id;
    let mut r = supported(&complete(FederalStatus::Single));
    crate::sep::calculate(&d, [1413, 0], None, &[], &mut r);
    assert!(r.issues.iter().any(|v| v.section == "Partner retirement"));
    add(&mut d, income(sep_business(10000, 100)));
    let mut r = supported(&complete(FederalStatus::Single));
    crate::sep::calculate(&d, [1413, 0], None, &[(id, se)], &mut r);
    assert!(r.issues.iter().any(|v| v.section == "Business retirement"));
}

#[test]
fn partner_sep_uses_actual_profit_when_se_tax_uses_optional_earnings() {
    let mut d = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(1000, true);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(100000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 900000 });
    partner.pensions = 10820;
    partner.retirement = PartnerRetirement::Sep { rate: 25.0 };
    let se = partner
        .se_after_passive(
            crate::kernel::carry::Amount::new(0).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let id = d.income.last().unwrap().id;
    // Optional earnings6000 -> tax918, half459. Actual plan earnings remain1000.
    let halves = alloc::collections::BTreeMap::from([(id, 459)]);
    let mut r = supported(&complete(FederalStatus::Single));
    let result = crate::sep::calculate(&d, [459, 0], Some(&halves), &[(id, se)], &mut r);
    assert_eq!(result.by_business[&id], 108);
    assert!(!r.issues.iter().any(|v| v.section == "Business retirement"));
    assert_eq!(line_amount(&r, &alloc::format!("business_{id}_sep_limit")), 10820);
}

#[test]
fn partner_service_payments_do_not_release_passive_losses_or_add_basis() {
    for active in [false, true] {
        let mut partner = ordinary_partner_source(-4000, active);
        partner.guaranteed_services = 600000;
        partner.reported_se = Some(600000); // Limited partner: services only.
        let before = partner.regular.pool_result(partner.material_for_losses()).unwrap();
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::Partnership { data: partner }));
        let mut r = supported(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals(&d, 40000, &mut r);
        assert!(pool.inputs_complete, "{:?}", r.issues);
        assert_eq!(pool.income, if active { 2000 } else { 6000 });
        assert_eq!(
            crate::kernel::ratio::dollars(pool.partners[0].suspended.get()),
            if active { 0 } else { 4000 }
        );
        assert_eq!(pool.partners[0].prior_released, 0);
        assert_eq!(pool.partner_se[0].1.earnings, 6000);
        assert_eq!(pool.partner_se[0].1.passive_earnings, 0);
        assert_eq!(pool.eic_passive_income(), 0);
        assert_eq!(pool.income + pool.partner_niit_adjustment, 0);
        let FederalIncome::Partnership { data } = &d.income.last().unwrap().income else {
            unreachable!()
        };
        assert_eq!(data.regular.pool_result(data.material_for_losses()).unwrap(), before);
        assert_eq!(restart(d.clone()).draft, d);
    }
}

#[test]
fn partner_service_payments_do_not_release_former_passive_carries() {
    let mut partner = ordinary_partner_source(0, true);
    partner.regular.passive_loss = 300000;
    partner.alternative = Some(partner.regular.clone());
    partner.se_history = PartnerSeHistory { basis: 0, risk: 0, passive: 0 };
    partner.guaranteed_services = 600000;
    partner.reported_se = Some(600000);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let mut r = supported(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 40000, &mut r);
    assert!(pool.inputs_complete, "{:?}", r.issues);
    assert_eq!(pool.income, 6000);
    assert_eq!(crate::kernel::ratio::dollars(pool.partners[0].suspended.get()), 3000);
    assert_eq!(pool.partners[0].prior_released, 0);
}

#[test]
fn partner_service_payments_survive_disallowed_ordinary_se_loss() {
    let mut partner = ordinary_partner_source(-4000, false);
    partner.limited_for_se = Some(false);
    partner.guaranteed_services = 600000;
    partner.reported_se = Some(200000); // Box 14 nets the ordinary loss.
    let se = partner
        .se_after_passive(
            crate::kernel::carry::Amount::new(4000 * 100).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    assert_eq!(se.earnings, 6000);
    assert_eq!(se.passive_earnings, 0);
    assert_eq!(se.passive_suspended, 4000);
    let se = partner
        .se_after_passive(
            crate::kernel::carry::Amount::new(2000 * 100).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    assert_eq!(se.earnings, 4000);
    assert_eq!(se.passive_earnings, -2000);
    partner.reported_se = Some(700000);
    assert!(
        partner
            .se_after_passive(
                crate::kernel::carry::Amount::new(4000 * 100).unwrap(),
                crate::kernel::carry::Amount::new(0).unwrap()
            )
            .is_err()
    );
}

#[test]
fn partner_service_payments_are_excluded_from_qualified_gross_allocation() {
    let mut partner = partner_health_source(0, false);
    partner.guaranteed_services = 1000000;
    partner.reported_se = Some(3000000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 3000000 });
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let id = d.income.last().unwrap().id;
    let halves = crate::partnership_qbi::half_se_by_gross(&d, [2120, 0]).unwrap().sources;
    assert_eq!(halves[&id], 1413); // 20k qualified / 30k total; services are not QBI.
    let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.qbi_records.qualified_se_gross = 3000000;
    assert!(crate::partnership_qbi::half_se_by_gross(&d, [2120, 0]).is_err());
}

#[test]
fn partner_limited_service_payments_coordinate_se_and_qbi() {
    let mut partner = ordinary_partner_source(20000, false);
    partner.guaranteed_services = 600000;
    partner.reported_se = Some(600000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 600000 });
    let qbi = partner_qbi_statement(20000);
    partner.qbi_records.qualified_se_gross = 0;
    partner.qbi = Some(qbi);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let id = d.income.last().unwrap().id;
    let halves = crate::partnership_qbi::half_se_by_gross(&d, [424, 0]).unwrap().sources;
    assert_eq!(halves[&id], 0);
    let r = estimate(&d);
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("coordination stabilized") && n.contains("SE tax $848")),
        "{:?}",
        r.review_notes
    );
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("current QBI") && n.contains("eligibility: $20000")),
        "{:?}",
        r.review_notes
    );
    assert!(r.tax.is_some(), "{:?}", r.issues);
}

#[test]
fn partner_service_payments_enter_amt_without_changing_ordinary_loss_limits() {
    let mut partner = ordinary_partner_source(-4000, false);
    partner.alternative.as_mut().unwrap().ordinary = -200000;
    partner.guaranteed_services = 600000;
    partner.reported_se = Some(600000);
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let mut r = supported(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals_for(&d, 40000, &mut r, true);
    assert!(pool.inputs_complete, "{:?}", r.issues);
    assert_eq!(pool.income, 6000);
    assert_eq!(crate::kernel::ratio::dollars(pool.partners[0].suspended.get()), 2000);
    assert_eq!(pool.partners[0].nonpassive, 6000);
}

#[test]
fn partner_capital_payments_keep_se_and_investment_classifications_separate() {
    for limited in [true, false] {
        for active in [false, true] {
            let mut partner = ordinary_partner_source(0, active);
            partner.guaranteed_services = 200000;
            partner.guaranteed_capital = 600000;
            partner.limited_for_se = Some(limited);
            partner.reported_se = Some(if limited { 200000 } else { 800000 });
            let mut d = complete(FederalStatus::Single);
            add(&mut d, income(FederalIncome::Partnership { data: partner }));
            let mut r = supported(&complete(FederalStatus::Single));
            let pool = crate::extended_rules::rentals(&d, 40000, &mut r);
            assert!(pool.inputs_complete, "{:?}", r.issues);
            assert_eq!(pool.income, 8000);
            assert_eq!(pool.partners[0].nonpassive, 8000);
            assert_eq!(pool.partner_se[0].1.earnings, if limited { 2000 } else { 8000 });
            assert_eq!(pool.income + pool.partner_niit_adjustment, if limited { 6000 } else { 0 });
            // Pub. 596 Worksheet 1 uses passive Schedule E net, unlike NIIT.
            assert_eq!(pool.eic_passive_income(), 0);
            assert_eq!(restart(d.clone()).draft, d);
        }
    }
}

#[test]
fn partner_capital_payments_preserve_regular_and_amt_passive_carries() {
    let mut partner = ordinary_partner_source(-4000, false);
    partner.regular.passive_loss = 300000;
    partner.alternative = Some(partner.regular.clone());
    partner.alternative.as_mut().unwrap().ordinary = -200000;
    partner.se_history = PartnerSeHistory { basis: 0, risk: 0, passive: 0 };
    partner.guaranteed_capital = 600000;
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    for (amt, carry) in [(false, 7000), (true, 5000)] {
        let mut r = supported(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals_for(&d, 40000, &mut r, amt);
        assert!(pool.inputs_complete, "{:?}", r.issues);
        assert_eq!(pool.income, 6000);
        assert_eq!(crate::kernel::ratio::dollars(pool.partners[0].suspended.get()), carry);
        assert_eq!(pool.partners[0].prior_released, 0);
    }
}

#[test]
fn partner_capital_payments_reconcile_box14_before_loss_attribution() {
    let mut partner = ordinary_partner_source(-4000, false);
    partner.limited_for_se = Some(false);
    partner.guaranteed_capital = 600000;
    partner.reported_se = Some(200000);
    let se = partner
        .se_after_passive(
            crate::kernel::carry::Amount::new(4000 * 100).unwrap(),
            crate::kernel::carry::Amount::new(0).unwrap(),
        )
        .unwrap();
    assert_eq!(se.earnings, 6000);
    assert_eq!(se.passive_suspended, 4000);
    partner.limited_for_se = Some(true);
    assert!(
        partner
            .se_after_passive(
                crate::kernel::carry::Amount::new(4000 * 100).unwrap(),
                crate::kernel::carry::Amount::new(0).unwrap()
            )
            .is_err()
    ); // Capital cannot enter limited-partner box 14.
    partner.reported_se = Some(0);
    assert_eq!(
        partner
            .se_after_passive(
                crate::kernel::carry::Amount::new(4000 * 100).unwrap(),
                crate::kernel::carry::Amount::new(0).unwrap()
            )
            .unwrap()
            .earnings,
        0
    );
}

#[test]
fn partner_capital_payments_never_become_qualified_gross_income() {
    let mut partner = partner_health_source(0, false);
    partner.guaranteed_capital = 1000000;
    partner.reported_se = Some(3000000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 3000000 });
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let id = d.income.last().unwrap().id;
    assert_eq!(crate::partnership_qbi::half_se_by_gross(&d, [2120, 0]).unwrap().sources[&id], 1413);
    let r = estimate(&d);
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("coordination stabilized") && n.contains("SE tax $4239")),
        "{:?}",
        r.review_notes
    );
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("current QBI") && n.contains("eligibility: $18587")),
        "{:?}",
        r.review_notes
    );
    assert!(r.tax.is_some(), "{:?}", r.issues);
    let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.qbi_records.qualified_se_gross = 3000000;
    assert!(crate::partnership_qbi::half_se_by_gross(&d, [2120, 0]).is_err());
}

#[test]
fn partner_capital_payments_enter_only_general_partner_plan_compensation() {
    for limited in [true, false] {
        let mut partner = ordinary_partner_source(0, true);
        partner.guaranteed_capital = 1999999;
        partner.limited_for_se = Some(limited);
        partner.reported_se = Some(if !limited { 1999999 } else { 0 });
        partner.pensions = 0;
        partner.retirement = PartnerRetirement::Sep { rate: 25.0 };
        let se = partner
            .se_after_passive(
                crate::kernel::carry::Amount::new(0).unwrap(),
                crate::kernel::carry::Amount::new(0).unwrap(),
            )
            .unwrap();
        let mut d = complete(FederalStatus::Single);
        add(&mut d, income(FederalIncome::Partnership { data: partner }));
        let id = d.income.last().unwrap().id;
        let half = if !limited { 1413 } else { 0 };
        let halves = alloc::collections::BTreeMap::from([(id, half)]);
        let mut r = supported(&complete(FederalStatus::Single));
        crate::sep::calculate(&d, [half, 0], Some(&halves), &[(id, se)], &mut r);
        let line =
            r.lines.iter().find(|v| v.id == alloc::format!("business_{id}_sep_limit")).unwrap();
        assert_eq!(line.amount, if !limited { 371739 } else { 0 });
    }
}

#[test]
fn partner_paid_health_enters_income_once_and_reuses_shared_deduction() {
    let partner = partner_paid_health_source(6000, false);
    assert!(partner.health_payment_check().is_ok());
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("coordination stabilized")
            && n.contains("SE tax $3674")
            && n.contains("health deduction $6000")),
        "{:?}",
        r.review_notes
    );
    assert!(
        r.review_notes.iter().any(|n| n.contains("current QBI")
            && n.contains("eligibility: $13972")
            && n.contains("Health-insurance deduction $6000")
            && n.contains("combined qualified deduction of $4615")),
        "{:?}",
        r.review_notes
    );
    let mut report = supported(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 40000, &mut report);
    assert_eq!(pool.income, 26000); // Subset field does not add another $6,000.
    assert_eq!(pool.partner_se[0].1.earnings, 26000);
    assert_eq!(estimate(&restart(d).draft), r);
    assert!(r.tax.is_some(), "{:?}", r.issues);
}

#[test]
fn partner_paid_health_employer_eligibility_does_not_remove_taxable_income() {
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Partnership { data: partner_paid_health_source(6000, true) }),
    );
    let id = d.income.last().unwrap().id;
    let halves = alloc::collections::BTreeMap::from([(id, 1837)]);
    let earnings = alloc::collections::BTreeMap::from([(id, 26000)]);
    let mut report = supported(&complete(FederalStatus::Single));
    let result = crate::health::calculate(
        &d,
        [1837, 0],
        Some(&halves),
        &earnings,
        &Default::default(),
        &Default::default(),
        None,
        &mut report,
    );
    assert_eq!(result.deduction, 0);
    assert_eq!(result.medical, 600000);
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.contains("coordination stabilized")
        && n.contains("SE tax $3674")
        && n.contains("health deduction $0")));
}

#[test]
fn partner_paid_health_reconciles_taxable_subset_before_deduction() {
    let mut partner = partner_paid_health_source(6000, false);
    partner.health_guaranteed += 1;
    assert!(partner.health_payment_check().is_err());
    partner.guaranteed_services += 1;
    assert!(partner.health_payment_check().is_err()); // Still exceeds premium records.
    partner.health_guaranteed = 600000;
    partner.guaranteed_services = 500000;
    assert!(partner.health_payment_check().is_err()); // Still exceeds K-1 services.
    partner.guaranteed_services = 600000;
    partner.health_insurance = BusinessHealthInsurance::None;
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    assert!(estimate(&d).issues.iter().any(|i| i.section == "Partner health insurance"
        && i.message.contains("Taxable partnership-paid")));
}

#[test]
fn partner_paid_health_limited_partner_uses_service_earnings_cap() {
    let mut partner = partner_paid_health_source(6000, false);
    partner.limited_for_se = Some(true);
    partner.reported_se = Some(600000);
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 600000 });
    partner.qbi_records.qualified_se_gross = 0;
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("coordination stabilized")
            && n.contains("SE tax $848")
            && n.contains("health deduction $5576")),
        "{:?}",
        r.review_notes
    );
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("current QBI") && n.contains("eligibility: $20000"))
    );
}

#[test]
fn partner_paid_health_reconciles_full_ltc_payment_before_age_limit() {
    let mut partner = partner_paid_health_source(10000, false);
    partner.health_insurance = BusinessHealthInsurance::Plan {
        months: vec![],
        long_term_care: vec![BusinessCarePremium {
            person: -1,
            months: vec![BusinessCareMonth {
                month: 1,
                premiums: 1000000,
                employer_eligible: false,
                reimbursed: None,
            }],
            qualified: CareContractQualification::Qualified,
        }],
        establishment: crate::HealthEstablishment::Partnership {
            payment: crate::EntityPremiumPayment::EntityPaid,
        },
    };
    assert!(partner.health_payment_check().is_ok()); // Full $10k, not the $4,960 cap.
    let mut d = complete(FederalStatus::Single);
    set_age(&mut d, -1, 70);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("coordination stabilized")
            && n.contains("SE tax $4239")
            && n.contains("health deduction $4960")),
        "{:?}",
        r.review_notes
    );
}

#[test]
fn partner_distribution_health_reduces_basis_once_without_guaranteed_income() {
    let partner = partner_distribution_health_source();
    assert!(partner.health_payment_check().is_ok());
    let base = partner_health_source(6000, false)
        .regular
        .pool_result(crate::participation::Participation::Yes)
        .unwrap();
    let paid = partner.regular.pool_result(crate::participation::Participation::Yes).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(base.activity.basis.ending.get())
            - crate::kernel::ratio::dollars(paid.activity.basis.ending.get()),
        6000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            base.activity.risk_investment.unwrap() - paid.activity.risk_investment.unwrap()
        ),
        6000
    );
    assert_eq!(
        partner
            .se_after_passive(
                crate::kernel::carry::Amount::new(0).unwrap(),
                crate::kernel::carry::Amount::new(0).unwrap()
            )
            .unwrap()
            .earnings,
        20000
    );
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    let r = estimate(&d);
    assert!(r.review_notes.iter().any(|n| n.contains("coordination stabilized")
        && n.contains("SE tax $2826")
        && n.contains("health deduction $6000")));
    assert!(
        r.review_notes
            .iter()
            .any(|n| n.contains("current QBI") && n.contains("eligibility: $12587"))
    );
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn partner_distribution_health_requires_both_basis_records_to_include_payment() {
    let mut partner = partner_distribution_health_source();
    partner.regular.basis.distributions -= 1;
    assert!(partner.health_payment_check().is_err());
    partner.regular.basis.distributions += 1;
    partner.alternative.as_mut().unwrap().basis.distributions -= 1;
    assert!(partner.health_payment_check().is_err());
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner }));
    assert!(estimate(&d).issues.iter().any(|i| {
        i.section == "Partner health insurance"
            && i.message.contains("Distribution-paid")
            && Some({
                let m = &i.display;
                m.key.as_str()
            }) == Some("ISSUE_PARTNER_DISTRIBUTION_HEALTH_PREMIUMS")
    }));
}

#[test]
fn partner_distribution_health_cannot_overlap_taxable_premium_subset() {
    let mut partner = partner_distribution_health_source();
    partner.health_guaranteed = 100000;
    partner.guaranteed_services = 100000;
    assert!(partner.health_payment_check().is_err()); // $7k subsets cannot fit $6k premiums.
    partner.health_distributions = 500000;
    assert!(partner.health_payment_check().is_ok()); // Mixed funding without duplication.
    partner.health_insurance = BusinessHealthInsurance::None;
    assert!(partner.health_payment_check().is_err());
}

#[test]
fn partner_distribution_health_can_limit_ordinary_losses_without_adding_income() {
    let mut partner = ordinary_partner_source(-8000, true);
    partner.health_insurance = partner_health_source(6000, false).health_insurance;
    partner.health_distributions = 600000;
    partner.regular.basis.distributions = 600000;
    partner.alternative = Some(partner.regular.clone());
    assert!(partner.health_payment_check().is_ok());
    let pool = partner.regular.pool_result(crate::participation::Participation::Yes).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(pool.activity.basis.allowed[K1LossClass::Ordinary].get()),
        4000
    ); // $10k opening less $6k distribution.
    assert_eq!(
        crate::kernel::ratio::dollars(pool.activity.basis.suspended[K1LossClass::Ordinary].get()),
        4000
    );
    assert_eq!(crate::kernel::ratio::dollars(pool.nonpassive), -4000);
}

#[test]
fn partner_excess_loss_preserves_qualified_and_nonqualified_fractions() {
    for (qualified, deferred) in [(300000, 44000), (180000, 26400), (0, 0)] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Partnership { data: partner_excess_loss_source(qualified) }),
        );
        let r = estimate(&d);
        assert!(
            r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")
                && n.contains("NOL carryforward $44000;")
                && n.contains(&alloc::format!("qualified portion ${deferred};"))),
            "{:?}",
            r.review_notes
        );
        assert!(
            !r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"),
            "{:?}",
            r.issues
        );
        assert!(r.tax.is_some(), "{:?}", r.issues);
        assert_eq!(estimate(&restart(d).draft), r);
    }
}

#[test]
fn partner_excess_loss_updates_current_qbi_only_by_qualified_suspension() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }));
    let mut r = supported(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 40000, &mut r);
    let mut activities = vec![crate::business_rules::QbiActivity {
        source_wages_cents: Some(0),
        source_property_cents: Some(0),
        source_income_cents: Some(-18_000_000),
        owner: None,
        income: -180000,
        wages: 0,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    }];
    let qualified =
        crate::partnership_qbi::loss_only_excess(&d, &pool, &mut activities, 44000).unwrap();
    assert_eq!(qualified, 26400);
    assert_eq!(activities[0].income, -153600); // Not -136,000 from deferring all $44k.
}

#[test]
fn partner_excess_loss_mixed_sources_fail_explicitly_instead_of_using_fallback() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }));
    add(&mut d, income(cash_business(10000)));
    assert!(estimate(&d).issues.iter().any(|i| i.section == "Partnership excess-loss allocation"));
    d.income.pop();
    let FederalIncome::Partnership { data } = &mut d.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.guaranteed_capital = 1000000;
    assert!(estimate(&d).issues.iter().any(|i| i.section == "Partnership excess-loss allocation"));
}

#[test]
fn partner_excess_loss_missing_qbi_statement_never_emits_resolved_character() {
    let mut d = complete(FederalStatus::Single);
    let mut data = partner_excess_loss_source(180000);
    data.qbi = None;
    add(&mut d, income(FederalIncome::Partnership { data }));
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"));
    assert!(!r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")));
}

#[test]
fn partner_uniform_profit_offset_preserves_qualified_excess_fraction() {
    let d = partner_uniform_profit_draft(6000);
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")
            && n.contains("NOL carryforward $34000;")
            && n.contains("qualified portion $20400;")),
        "{:?}",
        r.review_notes
    );
    assert!(
        !r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"),
        "{:?}",
        r.issues
    );
    assert_eq!(estimate(&restart(d).draft), r);
    assert!(r.tax.is_some(), "{:?}", r.issues);
}

#[test]
fn partner_uniform_profit_offset_changes_only_loss_activity() {
    let d = partner_uniform_profit_draft(6000);
    let mut r = supported(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 40000, &mut r);
    let activity = |income| crate::business_rules::QbiActivity {
        source_wages_cents: Some(0),
        source_property_cents: Some(0),
        source_income_cents: Some(income * 100),
        owner: None,
        income,
        wages: 0,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    };
    let mut activities = vec![activity(-180000), activity(6000)];
    assert_eq!(
        crate::partnership_qbi::current_profit_excess(&d, &pool, &mut activities, 34000).unwrap(),
        20400
    );
    assert_eq!(activities[0].income, -159600);
    assert_eq!(activities[1].income, 6000);
    let mut invalid = vec![activity(-180000), activity(11000)];
    assert!(crate::partnership_qbi::current_profit_excess(&d, &pool, &mut invalid, 34000).is_err());
    assert_eq!(invalid[0].income, -180000); // Failed validation cannot mutate live QBI.
}

#[test]
fn partner_single_loss_profit_offsets_keep_independent_qualified_fractions() {
    for profit_qualified in [0, 5000, 10000] {
        let d = partner_uniform_profit_draft(profit_qualified);
        let r = estimate(&d);
        assert!(
            r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")
                && n.contains("NOL carryforward $34000; qualified portion $20400;")),
            "{:?} {:?}",
            r.review_notes,
            r.issues
        );
        assert!(!r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"));
        let mut report = supported(&complete(FederalStatus::Single));
        let pool = crate::extended_rules::rentals(&d, 40000, &mut report);
        let activity = |income| crate::business_rules::QbiActivity {
            source_wages_cents: Some(0),
            source_property_cents: Some(0),
            source_income_cents: Some(income * 100),
            owner: None,
            income,
            wages: 0,
            property: 0,
            active: true,
            service: Some(false),
            group: None,
        };
        let mut values = vec![activity(-180000), activity(profit_qualified)];
        assert_eq!(
            crate::partnership_qbi::current_profit_excess(&d, &pool, &mut values, 34000).unwrap(),
            20400
        );
        assert_eq!(values[0].income, -159600);
        assert_eq!(values[1].income, profit_qualified);
        assert_eq!(estimate(&restart(d).draft), r);
    }
}

#[test]
fn partner_multiple_mixed_losses_with_profit_remain_unresolved() {
    let mut d = partner_uniform_profit_draft(5000);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(90000) }));
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"));
    assert!(!r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")));
}

#[test]
fn partner_schedule_c_loss_combination_preserves_nonqualified_fraction() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }));
    add(&mut d, income(cash_business(-100000)));
    let r = estimate(&d);
    // $400k total loss, $280k qualified: 70% of $144k deferral is $100,800.
    assert!(
        r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")
            && n.contains("NOL carryforward $144000;")
            && n.contains("qualified portion $100800;")),
        "{:?}",
        r.review_notes
    );
    assert!(!r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"));
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn partner_schedule_c_loss_allocation_conserves_total_current_qualified_loss() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }));
    add(&mut d, income(cash_business(-100000)));
    let mut r = supported(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 40000, &mut r);
    let activity = |income| crate::business_rules::QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income,
        wages: 0,
        property: 0,
        active: true,
        service: Some(false),
        group: None,
    };
    let mut activities = vec![activity(-180000), activity(-100000)];
    // The first activity is the existing whole-dollar K1 fixture; the
    // Schedule C activity keeps its owning annual adapter.
    activities[0].source_income_cents = Some(-18_000_000);
    activities[0].source_wages_cents = Some(0);
    activities[0].source_property_cents = Some(0);
    assert_eq!(
        crate::partnership_qbi::loss_only_excess(&d, &pool, &mut activities, 144000).unwrap(),
        100800
    );
    assert_eq!(activities.iter().map(|v| v.income).sum::<i64>(), -179200);
    assert_eq!(activities[0].income, -115200);
    assert_eq!(activities[1].income, -64000);
}

#[test]
fn partner_schedule_c_loss_source_survives_restart() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }));
    let mut business = cash_business(-100000);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.specified_service = false;
    add(&mut d, income(business.clone()));
    let known = estimate(&d);
    assert_eq!(estimate(&restart(d).draft), known);
}

#[test]
fn partner_scorp_loss_combination_preserves_qualified_fraction() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }));
    let mut scorp = sole_qbi_loss(100000, 100000, 50000);
    scorp.amt = Some(amt_records(&scorp));
    add(
        &mut d,
        income(FederalIncome::Scorp { data: crate::tests::support::builders::scorp_source(scorp) }),
    );
    let r = estimate(&d);
    // $230k qualified / $400k total loss; $144k excess -> $82,800 qualified.
    assert!(
        r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")
            && n.contains("NOL carryforward $144000;")
            && n.contains("qualified portion $82800;")),
        "{:?} {:?}",
        r.review_notes,
        r.issues
    );
    assert!(!r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"));
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn partner_scorp_missing_qbi_never_reports_resolved_loss_character() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }));
    let mut scorp = sole_qbi_loss(100000, 100000, 50000);
    scorp.qbi_statement = None;
    scorp.amt = Some(amt_records(&scorp));
    add(
        &mut d,
        income(FederalIncome::Scorp { data: crate::tests::support::builders::scorp_source(scorp) }),
    );
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"));
    assert!(!r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")));
}

#[test]
fn partner_sstb_loss_uses_loss_year_eligibility_before_suspension() {
    for (wages, deferred) in [(6000000, 100800), (100000000, 64800)] {
        let mut d = complete(FederalStatus::Single);
        d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
        add(
            &mut d,
            income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }),
        );
        let mut business = cash_business(-100000);
        let FederalIncome::Business { data } = &mut business else { unreachable!() };
        data.specified_service = true;
        add(&mut d, income(business));
        let r = estimate(&d);
        assert!(
            r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")
                && n.contains(&alloc::format!("qualified portion ${deferred};"))),
            "{:?} {:?}",
            r.review_notes,
            r.issues
        );
    }
}

#[test]
fn partner_sstb_loss_partial_phase_is_not_reapplied_after_deferral() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }));
    let mut business = cash_business(-100000);
    let FederalIncome::Business { data } = &mut business else { unreachable!() };
    data.specified_service = true;
    add(&mut d, income(business));
    let mut r = supported(&complete(FederalStatus::Single));
    let pool = crate::extended_rules::rentals(&d, 40000, &mut r);
    let activity = |income, service| crate::business_rules::QbiActivity {
        source_wages_cents: None,
        source_property_cents: None,
        source_income_cents: None,
        owner: None,
        income,
        service,
        wages: 0,
        property: 0,
        active: true,
        group: None,
    };
    let mut activities = vec![activity(-180000, Some(false)), activity(-100000, Some(true))];
    // The first activity is the existing whole-dollar K1 fixture; the
    // Schedule C activity keeps its owning annual adapter.
    activities[0].source_income_cents = Some(-18_000_000);
    activities[0].source_wages_cents = Some(0);
    activities[0].source_property_cents = Some(0);
    // 2026 single midpoint: threshold $201,750 plus half of $75,000.
    let taxable = 239250;
    let deferred = crate::partnership_qbi::allocate_excess(
        &d,
        &pool,
        &mut activities,
        144000,
        taxable,
        FederalStatus::Single,
    )
    .unwrap();
    assert_eq!(deferred, 82800); // 57.5% qualified after SSTB eligibility.
    assert_eq!(activities[0].income, -115200);
    assert_eq!(activities[1].income, -32000);
    let again = crate::business_rules::eligible_qbi_activity(
        &activities[1],
        taxable,
        FederalStatus::Single,
    )
    .unwrap();
    assert_eq!(again.income, -32000);
}

#[test]
fn partner_foreign_loss_enters_excess_denominator_without_entering_qbi() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }));
    add(&mut d, income(foreign_service_business(-100000)));
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")
            && n.contains("NOL carryforward $144000;")
            && n.contains("qualified portion $64800;")),
        "{:?} {:?}",
        r.review_notes,
        r.issues
    );
    assert!(!r.issues.iter().any(|i| i.section == "Foreign business loss allocation"
        || i.section == "Partnership excess-loss allocation"));
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn partner_foreign_and_domestic_losses_share_one_conserved_deferral() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(180000) }));
    add(&mut d, income(foreign_service_business(-100000)));
    add(&mut d, income(cash_business(-100000)));
    let r = estimate(&d);
    // $280k QBI / $500k total loss; 56% of $244k is $136,640.
    assert!(
        r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")
            && n.contains("NOL carryforward $244000;")
            && n.contains("qualified portion $136640;")),
        "{:?}",
        r.review_notes
    );
}

#[test]
fn partner_single_partial_loss_retains_fraction_after_foreign_profit() {
    for (qualified, deferred) in [(0, 0), (180000, 20824), (300000, 34707)] {
        let mut d = complete(FederalStatus::Single);
        add(
            &mut d,
            income(FederalIncome::Partnership { data: partner_excess_loss_source(qualified) }),
        );
        add(&mut d, income(foreign_service_business(10000)));
        let r = estimate(&d);
        assert!(
            r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")
                && n.contains("NOL carryforward $34707;")
                && n.contains(&alloc::format!("qualified portion ${deferred};"))),
            "{:?} {:?}",
            r.review_notes,
            r.issues
        );
        assert!(!r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"
            || i.section == "Foreign business loss allocation"));
        assert_eq!(estimate(&restart(d).draft), r);
    }
}

#[test]
fn partner_fully_qualified_losses_keep_character_after_foreign_profit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(300000) }));
    add(&mut d, income(foreign_service_business(10000)));
    let r = estimate(&d);
    // Foreign half-SE $707 reduces the foreign net profit used by Form 461.
    assert!(
        r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")
            && n.contains("NOL carryforward $34707;")
            && n.contains("qualified portion $34707;")),
        "{:?} {:?}",
        r.review_notes,
        r.issues
    );
    assert!(!r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"
        || i.section == "Foreign business loss allocation"));
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn partner_foreign_profit_with_nonqualified_loss_keeps_allocation_unresolved() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(300000) }));
    add(&mut d, income(foreign_service_business(10000)));
    add(&mut d, income(foreign_service_business(-10000)));
    let r = estimate(&d);
    assert!(r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"));
    assert!(!r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")));
}

#[test]
fn partner_nonqualified_losses_remain_nonqualified_after_foreign_profit() {
    let mut d = complete(FederalStatus::Single);
    add(&mut d, income(FederalIncome::Partnership { data: partner_excess_loss_source(0) }));
    add(&mut d, income(foreign_service_business(-10000)));
    add(&mut d, income(foreign_service_business(20000)));
    let r = estimate(&d);
    // Net foreign SE profit is $10,000: half-SE is $707.
    // $310,000 losses - $20,000 profit + $707 - $256,000 = $34,707.
    assert!(
        r.review_notes.iter().any(|n| n.contains("Partnership excess-loss character")
            && n.contains(
                "NOL carryforward $34707; qualified portion $0; nonqualified portion $34707."
            )),
        "{:?} {:?}",
        r.review_notes,
        r.issues
    );
    assert!(!r.issues.iter().any(|i| i.section == "Partnership excess-loss allocation"
        || i.section == "Foreign business loss allocation"));
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn young_partner_source_uses_personal_services_not_se_classification() {
    let mut data = ordinary_partner_source(10000, true);
    // Missing source classification or services without the gross: taken
    // as capital, flagged for the warning.
    assert_eq!(data.young_earned(10000, 707), Ok((0, true)));
    data.income_source = None;
    assert_eq!(data.young_earned(10000, 707), Ok((0, true)));
    data.income_source = Some(BusinessIncomeSource::Capital);
    assert_eq!(data.young_earned(10000, 707), Ok((0, false)));
    data.income_source = Some(BusinessIncomeSource::Services);
    assert_eq!(data.young_earned(10000, 707), Ok((0, true)));
    data.gross_personal_services = Some(2000000);
    assert_eq!(data.young_earned(10000, 707), Ok((20000, false)));
    data.income_source = Some(BusinessIncomeSource::Mixed { compensation: 500000 });
    assert_eq!(data.young_earned(10000, 707), Ok((2788, false)));
    assert_eq!(data.young_earned(-1000, 0), Ok((5000, false)));
    data.guaranteed_services = 10000;
    assert!(data.young_earned(10000, 707).is_err());
    data.income_source = Some(BusinessIncomeSource::Capital);
    assert_eq!(data.young_earned(10000, 707), Ok((100, false)));
    data.guaranteed_capital = 20000;
    assert_eq!(data.young_earned(10000, 707), Ok((100, false)));
    data.income_source = Some(BusinessIncomeSource::Services);
    assert_eq!(data.young_earned(10000, 707), Ok((20100, false)));
}

#[test]
fn young_partner_thirty_percent_answer_needs_a_net_profit() {
    // T2-src, i8615 "Earned income": 30% of the partner's share of net
    // profit after half SE, with no ceiling (and so no amount) without one.
    let mut data = ordinary_partner_source(10000, true);
    data.income_source = Some(BusinessIncomeSource::MixedThirtyPercent);
    // 30% x (10,000 - 707) = 2,787.90 -> 2,788.
    assert_eq!(data.young_earned(10000, 707), Ok((2788, false)));
    // Boundaries: zero and negative ordinary profit give no amount, so the
    // income is taken as capital, flagged for the warning.
    assert_eq!(data.young_earned(0, 0), Ok((0, true)));
    assert_eq!(data.young_earned(-1000, 0), Ok((0, true)));
    // A $100 guaranteed service payment: ordinary 10,000 - 100 = 9,900;
    // 30% x 9,900 = 2,970, plus the payment 100 = 3,070.
    data.guaranteed_services = 10000;
    assert_eq!(data.young_earned(10000, 0), Ok((3070, false)));
}

#[test]
fn young_partner_thirty_percent_loss_is_taken_as_capital_with_a_warning() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100615;
    d.people.as_mut().unwrap().taxpayer.parent_alive = true;
    let mut data = ordinary_partner_source(-1000, true);
    data.income_source = Some(BusinessIncomeSource::MixedThirtyPercent);
    add(
        &mut d,
        FederalIncomeEntry {
            label: "Family orchard LP".into(),
            ..income(FederalIncome::Partnership { data })
        },
    );
    let r = estimate(&d);
    assert!(!r.issues.iter().any(|i| i.section == "Income: Family orchard LP"), "{:?}", r.issues);
    assert!(noted(&r, "WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL"), "{r:?}");
}

#[test]
fn young_partner_classification_is_saved_and_reported() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100615;
    d.people.as_mut().unwrap().taxpayer.parent_alive = true;
    let mut data = ordinary_partner_source(10000, true);
    data.income_source = Some(BusinessIncomeSource::Mixed { compensation: 500000 });
    data.qbi = Some(partner_qbi_statement(10000));
    add(&mut d, income(FederalIncome::Partnership { data }));
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("young-filer earned income: $3000")),
        "{:?}",
        r.review_notes
    );
    let note = &r
        .review_notes
        .iter()
        .find(|n| n.contains("young-filer earned income: $3000"))
        .unwrap()
        .display;
    assert_eq!(note.key, "NOTE_PARTNERSHIP_YOUNG_FILER_EARNED");
    assert_eq!(note.args[1], MessageArg::usd_dollars(3_000));
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn young_partner_services_without_gross_fact_are_taken_as_capital() {
    let mut d = complete(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100615;
    d.people.as_mut().unwrap().taxpayer.parent_alive = true;
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    let mut data = ordinary_partner_source(10000, true);
    data.income_source = Some(BusinessIncomeSource::Services);
    data.qbi = Some(partner_qbi_statement(10000));
    add(&mut d, income(FederalIncome::Partnership { data }));
    let r = estimate(&d);
    assert!(
        !r.issues.iter().any(|i| i.section == "Young-partner earned income"
            || i.section == "Dependent partner earned income"),
        "{:?}",
        r.issues
    );
    assert!(noted(&r, "WARN_KIDDIE_SOURCE_ASSUMED_CAPITAL"), "{r:?}");
}

#[test]
fn young_limited_partner_service_guarantee_enters_earnings_once() {
    let mut d = empty_return(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100615;
    d.people.as_mut().unwrap().taxpayer.parent_alive = true;
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    let mut data = ordinary_partner_source(0, true);
    data.income_source = Some(BusinessIncomeSource::Capital);
    data.guaranteed_services = 600000;
    data.reported_se = Some(600000);
    data.se_gross = Some(PartnerSeGross::Nonfarm { gross: 600000 });
    data.qbi = Some(partner_qbi_statement(0));
    add(&mut d, income(FederalIncome::Partnership { data }));
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("young-filer earned income: $6000")),
        "{:?}",
        r.review_notes
    );
    assert!(
        r.review_notes.iter().any(|n| n.contains(
            "Dependent standard-deduction earned income from partnership sources: $6000;"
        ) && n
            .contains("total earned income after the shared half-SE deduction: $5576;")
            && n.contains("standard deduction: $6026.")),
        "{:?}",
        r.review_notes
    );
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn young_partner_alternate_worksheet_uses_actual_se_loss() {
    let mut d = empty_return(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100615;
    d.people.as_mut().unwrap().taxpayer.parent_alive = true;
    let mut data = ordinary_partner_source(-1000, true);
    data.income_source = Some(BusinessIncomeSource::Capital);
    data.limited_for_se = Some(false);
    data.reported_se = Some(-100000);
    data.qbi = Some(partner_qbi_statement(0));
    add(&mut d, income(FederalIncome::Partnership { data }));
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("young-filer earned income: $0;")
            && n.contains("net self-employment loss addback: $1000.")),
        "{:?} {:?}",
        r.review_notes,
        r.issues
    );
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn young_mixed_partner_guarantees_allocate_half_se_before_profit_ceiling() {
    let mut data = ordinary_partner_source(10000, true);
    data.income_source = Some(BusinessIncomeSource::Mixed { compensation: 500000 });
    data.guaranteed_services = 600000;
    data.guaranteed_capital = 200000;
    data.limited_for_se = Some(false);
    assert!(data.young_earned(18000, 2800).is_err());
    data.se_gross = Some(PartnerSeGross::Nonfarm { gross: 2800000 });
    // $2,000 of the $2,800 half-SE deduction belongs to ordinary profit.
    // 30% of ($10,000 - $2,000), capped by $5,000 reasonable pay, is $2,400.
    assert_eq!(data.young_earned(18000, 2800), Ok((8400, false)));
    // Dependent standard-deduction earned income starts before the half-SE
    // adjustment and includes the $6,000 payment for services once.
    assert_eq!(data.young_earned(18000, 0), Ok((9000, false)));
    data.guaranteed_capital = 0;
    assert_eq!(data.young_earned(16000, 0), Ok((9000, false)));
}

#[test]
fn young_mixed_partner_guarantees_flow_through_return_coordination() {
    let mut d = empty_return(FederalStatus::Single);
    d.people.as_mut().unwrap().taxpayer.birth_date = 20100615;
    d.people.as_mut().unwrap().taxpayer.parent_alive = true;
    d.people.as_mut().unwrap().taxpayer.dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    let mut data = ordinary_partner_source(10000, true);
    data.income_source = Some(BusinessIncomeSource::Mixed { compensation: 500000 });
    data.guaranteed_services = 600000;
    data.guaranteed_capital = 200000;
    data.limited_for_se = Some(false);
    data.reported_se = Some(1800000);
    data.se_gross = Some(PartnerSeGross::Nonfarm { gross: 2800000 });
    let qbi = partner_qbi_statement(10000);
    data.qbi_records.qualified_se_gross = 2000000;
    data.qbi = Some(qbi);
    add(&mut d, income(FederalIncome::Partnership { data }));
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains("young-filer earned income: $8727;")
            && n.contains("net self-employment loss addback: $0.")),
        "{:?} {:?}",
        r.review_notes,
        r.issues
    );
    assert!(
        r.review_notes.iter().any(|n| n.contains(
            "Dependent standard-deduction earned income from partnership sources: $9000;"
        ) && n
            .contains("total earned income after the shared half-SE deduction: $7728;")
            && n.contains("standard deduction: $8178.")),
        "{:?}",
        r.review_notes
    );
    assert_eq!(estimate(&restart(d).draft), r);
}

#[test]
fn appreciated_public_stock_uses_fmv_thirty_percent_or_annual_basis_election() {
    let mut d = complete(FederalStatus::Single);
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts.push(public_stock_gift());
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let fmv = supported(&d);
    assert_eq!(value(&fmv, "charity_property_1_claim"), 20_000);
    assert_eq!(value(&fmv, "charity_capital_gain_thirty_current_deduction"), 17_700);
    assert_eq!(value(&fmv, "charity_capital_gain_thirty_return_year_carryforward"), 2_300);
    assert_eq!(value(&fmv, "charity_return_year_carryforward"), 2_300);
    assert_eq!(value(&fmv, "form_8283_section_a_value"), 20_000);
    assert_eq!(estimate(&restart(d.clone()).draft), fmv);

    // 2026-11-02: automatic selection retains the same itemized facts and limits.
    d.deductions.as_mut().unwrap().itemize_override = None;
    let automatic = supported(&d);
    assert_eq!(return_line(&automatic, "12f"), 0);
    assert_eq!(value(&automatic, "charity_capital_gain_thirty_return_year_carryforward"), 2_300);
    assert_eq!(value(&automatic, "charity_return_year_carryforward"), 2_300);

    let ded = d.deductions.as_mut().unwrap();
    ded.basis_election = true;
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let basis = supported(&d);
    assert_eq!(value(&basis, "charity_property_1_claim"), 5_000);
    assert_eq!(value(&basis, "charity_property_fifty_current_deduction"), 4_700);
    assert_eq!(value(&basis, "charity_return_year_carryforward"), 0);
    assert_eq!(value(&basis, "form_8283_section_a_value"), 5_000);
    assert_eq!(estimate(&restart(d.clone()).draft), basis);

    d.deductions.as_mut().unwrap().property_gifts[0].acquired = 20250615;
    assert_eq!(value(&supported(&d), "charity_property_fifty_current_deduction"), 4_700);
    d.deductions.as_mut().unwrap().property_gifts[0].acquired = 20240101;
    d.deductions.as_mut().unwrap().property_gifts[0].value = 500_000;
    assert_eq!(value(&supported(&d), "charity_property_fifty_current_deduction"), 4_700);
    d.deductions.as_mut().unwrap().property_gifts[0].value = 2_000_000;
    d.deductions.as_mut().unwrap().property_gifts.push(public_stock_gift());
    assert_eq!(value(&supported(&d), "charity_property_fifty_current_deduction"), 9_700);
}

#[test]
fn gifted_public_stock_with_section_1015_basis_tacks_the_donors_holding_period() {
    let mut d = complete(FederalStatus::Single);
    let mut gift = public_stock_gift();
    gift.how_acquired = AcquiredBy::Gift;
    gift.acquired = 20240101;
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts.push(gift);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let r = supported(&d);
    assert_eq!(value(&r, "charity_property_1_claim"), 20_000);
    assert_eq!(value(&r, "charity_capital_gain_thirty_current_deduction"), 17_700);
    assert_eq!(estimate(&restart(d.clone()).draft), r);

    d.deductions.as_mut().unwrap().property_gifts[0].how_acquired = AcquiredBy::Gift;
    d.deductions.as_mut().unwrap().property_gifts[0].acquired = 20260101;
    assert_eq!(value(&supported(&d), "charity_property_fifty_current_deduction"), 4_700);
    assert!(d.valid_stored());
}

#[test]
fn gifted_public_stock_between_gift_value_and_donor_basis_has_no_hypothetical_gain() {
    let mut d = complete(FederalStatus::Single);
    let mut gift = public_stock_gift();
    gift.how_acquired = AcquiredBy::Gift;
    gift.acquired = 20240101;
    gift.basis = 150_000;
    gift.value = 100_000;
    let ded = d.deductions.as_mut().unwrap();
    ded.property_gifts.push(gift);
    ded.itemize_override = Some(ItemizeOverride::Itemize);
    let r = supported(&d);
    assert_eq!(value(&r, "charity_property_1_claim"), 1_000);
    assert_eq!(value(&r, "charity_property_fifty_current_deduction"), 700);
    assert_eq!(estimate(&restart(d.clone()).draft), r);
}

#[test]
fn retained_card_issuer_blocks_cash_and_partner_income_paths() {
    let mut direct = retained_card_issuer_business();
    direct.activity = BusinessActivity::Ordinary {
        returns: 0,
        receipts: 4_000_000,
        accounting: BusinessAccounting::Cash,
    };
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Business { data: direct }));
    unsupported(&draft, "Fuel tax credit");

    let mut partner = ordinary_partner_source(20_000, true);
    partner.qbi = Some(partner_qbi_statement(20_000));
    partner.fuel_tax_credit = Some(PartnerFuelTaxCredit {
        fuels: FederalFuelLines {
            card_issuer: Some(retained_card_issuer_claim()),
            ..FederalFuelLines::default()
        }
        .into(),
        statement_complete: true,
        farm_activity: false,
    });
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));
    unsupported(&draft, "Partnership fuel tax credit");
}

#[test]
fn registered_card_issuer_buyer_consent_refund_reaches_direct_and_partner_returns() {
    let mut direct = consented_card_issuer_claim("direct-consent", 100_000);
    direct.sales.truncate(1);
    assert!(direct.valid() && direct.qualified());
    let business_fuels =
        FederalFuelLines { card_issuer: Some(direct), ..FederalFuelLines::default() };
    let mut partner_claim = consented_card_issuer_claim("partner-consent", 50_000);
    partner_claim.sales = vec![partner_claim.sales.remove(2)];
    assert!(partner_claim.valid() && partner_claim.qualified());
    let partner_fuels =
        FederalFuelLines { card_issuer: Some(partner_claim), ..FederalFuelLines::default() };
    let (draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = supported(&draft);
    assert_eq!(line_amount(&report, "business_2_fuel_13a"), 2430,);
    assert_eq!(line_amount(&report, "partnership_3_fuel_13c_219"), 1090,);
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 3500);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(report.balance, baseline.balance.map(|value| value - 3500));
    assert_eq!(value(&report, "agi"), value(&baseline, "agi"));
    assert_eq!(estimate(&restart(draft.clone()).draft), report);
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn commercial_aviation_vendor_lines_8a_8b_reach_direct_and_partner_payments() {
    let business_fuels = FederalFuelLines {
        vendor_aviation_kerosene: Some(aviation_vendor_claim(100_000)),
        ..FederalFuelLines::default()
    };
    let partner_fuels = FederalFuelLines {
        vendor_aviation_kerosene: Some(aviation_vendor_claim(50_000)),
        ..FederalFuelLines::default()
    };
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = supported(&draft);
    for (id, cents) in [
        ("business_2_fuel_8a", 1_750),
        ("business_2_fuel_8b", 2_000),
        ("partnership_3_fuel_8a", 875),
        ("partnership_3_fuel_8b", 1_000),
    ] {
        assert_eq!(line_amount(&report, id), cents, "{id}");
    }
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 5_600);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    {
        let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
        data.fuel_tax_credit
            .as_mut()
            .unwrap()
            .fuels
            .vendor_aviation_kerosene
            .as_mut()
            .unwrap()
            .sales[0]
            .buyer_waiver_current = false;
    }
    fuel_left_out(&draft);
    {
        let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
        let claim =
            data.fuel_tax_credit.as_mut().unwrap().fuels.vendor_aviation_kerosene.as_mut().unwrap();
        claim.sales[0].buyer_waiver_current = true;
        claim.sales[1].sale_reference = claim.sales[0].sale_reference.clone();
    }
    assert_eq!(estimate(&draft).state, ResultState::Incomplete);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit.as_mut().unwrap().fuels.vendor_aviation_kerosene.as_mut().unwrap().sales
        [1]
    .sale_reference = "synthetic-sale-8b".into();
    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .vendor_aviation_kerosene
        .as_mut()
        .unwrap()
        .registration_active = false;
    fuel_left_out(&draft);
}

#[test]
fn noncommercial_aviation_vendor_8c_reaches_direct_and_partner_payments() {
    let business_fuels = FederalFuelLines {
        vendor_aviation_noncommercial: Some(noncommercial_aviation_vendor_claim(100_000)),
        ..FederalFuelLines::default()
    };
    let partner_fuels = FederalFuelLines {
        vendor_aviation_noncommercial: Some(noncommercial_aviation_vendor_claim(50_000)),
        ..FederalFuelLines::default()
    };
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = supported(&draft);
    for (id, cents) in [("business_2_fuel_8c", 250), ("partnership_3_fuel_8c", 125)] {
        assert_eq!(line_amount(&report, id), cents, "{id}");
    }
    assert_eq!(line_amount(&report, "fuel_tax_credit"), 375);
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 400);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(restart(draft.clone()).draft, draft);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .vendor_aviation_noncommercial
        .as_mut()
        .unwrap()
        .sales[0]
        .buyer_certificate_current = false;
    fuel_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .vendor_aviation_noncommercial
        .as_mut()
        .unwrap()
        .sales[0]
        .buyer_certificate_current = true;
    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .vendor_aviation_noncommercial
        .as_mut()
        .unwrap()
        .sales[0]
        .taxed_at_244 = false;
    fuel_left_out(&draft);

    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .vendor_aviation_noncommercial
        .as_mut()
        .unwrap()
        .sales[0]
        .taxed_at_244 = true;
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.source = BusinessSource::ForeignServices { confirmed: true, totalization_exempt: false };
    unsupported(&draft, "Fuel tax credit");
}

#[test]
fn nontaxable_aviation_vendor_8d_8e_8f_reach_direct_and_partner_payments() {
    let business_fuels = FederalFuelLines {
        vendor_aviation_nontaxable: Some(nontaxable_aviation_vendor_claim(100_000)),
        ..FederalFuelLines::default()
    };
    let partner_fuels = FederalFuelLines {
        vendor_aviation_nontaxable: Some(nontaxable_aviation_vendor_claim(50_000)),
        ..FederalFuelLines::default()
    };
    let (mut draft, baseline) =
        fuel_credit_sources(cash_business(10_000), business_fuels, partner_fuels, false);
    let report = supported(&draft);
    for (id, cents) in [
        ("business_2_fuel_8d", 2_430),
        ("business_2_fuel_8e", 2_180),
        ("business_2_fuel_8f", 10),
        ("partnership_3_fuel_8d", 1_215),
        ("partnership_3_fuel_8e", 1_090),
        ("partnership_3_fuel_8f", 5),
    ] {
        assert_eq!(line_amount(&report, id), cents, "{id}");
    }
    assert_eq!(line_amount(&report, "fuel_tax_credit"), 6_930);
    assert_eq!(return_line(&report, "31"), return_line(&baseline, "31") + 6_900);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(restart(draft.clone()).draft, draft);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);

    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .vendor_aviation_nontaxable
        .as_mut()
        .unwrap()
        .sales[0]
        .buyer_document_current = false;
    fuel_left_out(&draft);
    let FederalIncome::Business { data } = &mut draft.income[1].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .vendor_aviation_nontaxable
        .as_mut()
        .unwrap()
        .sales[0]
        .buyer_document_current = true;
    let FederalIncome::Partnership { data } = &mut draft.income[2].income else { unreachable!() };
    data.fuel_tax_credit
        .as_mut()
        .unwrap()
        .fuels
        .vendor_aviation_nontaxable
        .as_mut()
        .unwrap()
        .ua_registration
        .as_mut()
        .unwrap()
        .active = false;
    fuel_left_out(&draft);
}

// T2-29: Schedule K-1 (1065) separately stated items. Every expected amount
// below is worked by hand from the Partner's Instructions for Schedule K-1
// (Form 1065) (boxes 5, 6a/6b, 8, 9a, 13A, 15O, 18A, 19A and the partner
// basis worksheet), the 2026 Form 1040 instructions and Rev. Proc. 2025-32
// (single: $16,100 standard deduction; 10% to $12,400, 12% to $50,400, 22% to
// $105,700; qualified dividends and capital gain 0% to $49,450).

fn partner_separate_items() -> PartnerSeparateTotals {
    PartnerSeparateTotals {
        foreign_taxes: 0,
        interest: 0,
        dividends: 0,
        qualified_dividends: 0,
        royalties: 0,
        short: 0,
        long: 0,
        collectibles: 0,
        unrecaptured_partnership: 0,
        unrecaptured_fund: 0,
        cash_charity: 0,
        backup_withholding: 0,
        exempt_interest: 0,
    }
}

// Active general partner, box 1 $20,000, no SE earnings (box 14A zero for an
// established limited-partner SE classification), QBI statement $20,000,
// outside basis and at-risk amount $10,000 at the start of the year.
fn separate_items_partner(separate: PartnerSeparateTotals) -> PartnerK1 {
    let mut partner = ordinary_partner_source(20_000, true);
    partner.qbi = Some(partner_qbi_statement(20_000));
    partner.separate = Some(separate);
    partner
}

fn separate_items_draft(partner: PartnerK1) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));
    draft
}

fn set_partner_start(partner: &mut PartnerK1, opening: i64, distributions: i64) {
    for record in core::iter::once(&mut partner.regular).chain(partner.alternative.as_mut()) {
        record.basis.opening = Some(opening * 100);
        record.basis.distributions = distributions * 100;
        record.risk.start =
            ScorpRiskStart::Detailed { capacity: opening * 100, profit: 0, loss: 0 };
    }
}

#[test]
fn partner_separately_stated_items_reach_the_return_by_hand() {
    let mut items = partner_separate_items();
    items.interest = 120_000; // box 5 $1,200
    items.dividends = 200_000; // box 6a $2,000
    items.qualified_dividends = 150_000; // box 6b $1,500
    items.short = -50_000; // box 8 ($500)
    items.long = 300_000; // box 9a $3,000
    items.cash_charity = 100_000; // box 13 code A $1,000
    items.backup_withholding = 24_000; // box 15 code O $240
    items.exempt_interest = 40_000; // box 18 code A $400
    let mut partner = separate_items_partner(items);
    set_partner_start(&mut partner, 10_000, 5_000); // box 19 code A $5,000
    // Basis worksheet: line 1 $10,000; line 4 increases $20,000 + $1,200 +
    // $2,000 + $3,000 + $400 = $26,600; line 7 $36,600; line 8a $5,000;
    // line 11a $31,600. Part II: short-term loss $500 and cash gift $1,000
    // fit; ending basis $30,100.
    // At risk: $10,000 - $5,000 + $400 tax-exempt interest = $5,400; income
    // $20,000 + $1,200 + $2,000 + $3,000 = $26,200 absorbs the $500 loss;
    // ending $5,400 + $26,200 - $500 = $31,100.
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 30_100);
    assert_eq!(
        pool.activity.basis.allowed,
        LossAmounts::of(&[
            (K1LossClass::ShortTerm, crate::kernel::carry::Amount::new(500 * 100).unwrap()),
            (K1LossClass::Charity, crate::kernel::carry::Amount::new(1_000 * 100).unwrap())
        ])
    );
    assert_eq!(pool.activity.risk_end.map(crate::kernel::ratio::dollars), Some(31_100));
    assert_eq!(crate::kernel::ratio::dollars(pool.nonpassive), 20_000);
    let draft = separate_items_draft(partner);
    let result = supported(&draft);
    // Lines 2a $400, 2b $1,200, 3a $1,500, 3b $2,000.
    assert_eq!(return_line(&result, "2a"), 40_000);
    assert_eq!(return_line(&result, "2b"), 120_000);
    assert_eq!(return_line(&result, "3a"), 150_000);
    assert_eq!(return_line(&result, "3b"), 200_000);
    // Schedule D line 7 ($500) and line 15 $3,000: line 16 $2,500.
    assert_eq!(return_line(&result, "7a"), 250_000);
    // Schedule 1 line 5 / Schedule E line 32: box 1 $20,000 nonpassive.
    assert_eq!(return_line(&result, "8"), 2_000_000);
    // $60,000 + $1,200 + $2,000 + $2,500 + $20,000.
    assert_eq!(return_line(&result, "11a / 11b"), 8_570_000);
    assert_eq!(return_line(&result, "12e"), 1_610_000);
    // Non-itemizer cash gift: box 13 code A $1,000, within the $1,000 cap.
    assert_eq!(return_line(&result, "12f"), 100_000);
    // QBI 20% x $20,000 = $4,000; taxable income limit 20% x ($85,700 -
    // $17,100 - $4,000 net capital gain) = $12,920 does not bind.
    assert_eq!(return_line(&result, "13b"), 400_000);
    // $85,700 - $16,100 - $1,000 - $4,000.
    assert_eq!(return_line(&result, "15"), 6_460_000);
    // Worksheet: ordinary $60,600 by the Tax Table (midpoint $60,625:
    // $1,240 + $4,560 + 22% x $10,225 = $8,049.50, rounded $8,050) plus 15% x
    // $4,000 = $600 (the 0% band ends at $49,450, below $60,600).
    assert_eq!(return_line(&result, "16"), 865_000);
    assert_eq!(return_line(&result, "25c"), 24_000);
    // $8,650 - ($6,500 W-2 + $240 backup withholding).
    assert_eq!(result.balance, Some(191_000));
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn partner_separate_losses_exactly_absorbed_by_basis_are_allowed_and_one_dollar_less_carries() {
    // Box 1 zero; box 8 ($2,000) and box 13 code A $1,000 against a $3,000
    // opening basis and at-risk amount: Part II allows the full $3,000.
    let mut items = partner_separate_items();
    items.short = -200_000;
    items.cash_charity = 100_000;
    let mut partner = ordinary_partner_source(0, true);
    partner.qbi = Some(partner_qbi_statement(0));
    partner.separate = Some(items);
    set_partner_start(&mut partner, 3_000, 0);
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 0);
    let result = supported(&separate_items_draft(partner.clone()));
    // AGI $60,000 - $2,000 = $58,000; taxable $58,000 - $16,100 - $1,000 =
    // $40,900; Tax Table midpoint $40,925: $1,240 + 12% x $28,525 = $4,663.
    assert_eq!(return_line(&result, "7a"), -200_000);
    assert_eq!(return_line(&result, "15"), 4_090_000);
    assert_eq!(return_line(&result, "16"), 466_300);
    // One dollar less basis: $2,999 split pro rata; the first class takes
    // $1,999 and carries the dollar, and the return follows the allowed part.
    set_partner_start(&mut partner, 2_999, 0);
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 0);
    assert_eq!(
        crate::kernel::ratio::dollars(
            pool.activity.ledger.basis_allowed[K1LossClass::ShortTerm].get()
        ),
        1_999
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            pool.activity.ledger.basis_allowed[K1LossClass::Charity].get()
        ),
        1_000
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            pool.activity.ledger.basis_suspended[K1LossClass::ShortTerm].get()
        ),
        1
    );
    assert_eq!(pool.activity.ledger.basis_suspended[K1LossClass::Charity].get(), 33);
    let result = supported(&separate_items_draft(partner));
    assert_eq!(return_line(&result, "7a"), -199_900);
    assert_eq!(return_line(&result, "12f"), 100_000);
    assert!(
        result
            .review_notes
            .iter()
            .any(|v| v
                .contains("short-term capital loss carried to 2027 — $1 because of your basis")),
        "{:?}",
        result.review_notes
    );
}

#[test]
fn partner_cash_distribution_equal_to_basis_has_no_gain_and_one_more_is_section_731_gain() {
    // Line 7: $10,000 + $20,000 box 1 + $1,000 box 5 = $31,000.
    let mut items = partner_separate_items();
    items.interest = 100_000;
    let mut partner = separate_items_partner(items);
    set_partner_start(&mut partner, 10_000, 31_000);
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.distribution_gain.get()), 0);
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 0);
    let result = supported(&separate_items_draft(partner.clone()));
    // AGI $60,000 + $1,000 + $20,000 = $81,000; QBI $4,000; taxable $60,900;
    // midpoint $60,925: $5,800 + 22% x $10,525 = $8,115.50, rounded $8,116.
    assert_eq!(return_line(&result, "15"), 6_090_000);
    assert_eq!(return_line(&result, "16"), 811_600);
    // Unit B9-1: one dollar more is section 731(a)(1) gain on Form 8949 box C
    // or F by the holding period of the interest; without an answer the
    // holding period is taken as one year or less, with a warning.
    set_partner_start(&mut partner, 10_000, 31_001);
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.distribution_gain.get()), 1);
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.disposal.distribution_gain), 1);
    let result = supported(&separate_items_draft(partner.clone()));
    assert!(noted(&result, "WARN_K1_TERM_ASSUMED"));
    assert_eq!(return_line(&result, "7a"), 100);
    partner.interest_term = Some(GainTerm::Long);
    let result = supported(&separate_items_draft(partner));
    assert!(!noted(&result, "WARN_K1_TERM_ASSUMED"));
    assert_eq!(return_line(&result, "7a"), 100);
}

#[test]
fn partner_box_8_loss_meets_the_3000_capital_loss_limit() {
    // Box 8 ($5,000) passes basis ($30,000) and at risk; Schedule D line 21
    // allows ($3,000). AGI $60,000 + $20,000 - $3,000 = $77,000; taxable
    // $77,000 - $16,100 - $4,000 QBI = $56,900; midpoint $56,925: $5,800 +
    // 22% x $6,525 = $7,235.50, rounded $7,236.
    let mut items = partner_separate_items();
    items.short = -500_000;
    let draft = separate_items_draft(separate_items_partner(items));
    let result = supported(&draft);
    assert_eq!(return_line(&result, "7a"), -300_000);
    assert_eq!(return_line(&result, "15"), 5_690_000);
    assert_eq!(return_line(&result, "16"), 723_600);
}

#[test]
fn partner_cash_gift_above_the_non_itemizer_cap_is_limited_to_1000() {
    let mut items = partner_separate_items();
    items.cash_charity = 150_000;
    let result = supported(&separate_items_draft(separate_items_partner(items)));
    assert_eq!(return_line(&result, "12f"), 100_000);
}

#[test]
fn partner_qualified_dividends_above_box_6a_are_rejected() {
    let mut items = partner_separate_items();
    items.dividends = 100_000;
    items.qualified_dividends = 100_000;
    let draft = separate_items_draft(separate_items_partner(items.clone()));
    assert!(crate::federal_rules::validate_income(&draft.income[1]).is_ok());
    items.qualified_dividends = 100_001;
    let draft = separate_items_draft(separate_items_partner(items));
    assert!(crate::federal_rules::validate_income(&draft.income[1]).is_err());
}

#[test]
fn partner_boxes_not_calculated_are_refused_by_name() {
    let mut partner = separate_items_partner(partner_separate_items());
    partner.unentered = vec![PartnerK1Item::OtherIncome, PartnerK1Item::RentalDisposition];
    let draft = separate_items_draft(partner);
    unsupported(&draft, "Income: Synthetic document");
    let result = estimate(&draft);
    for text in ["box 11 codes B, D, I", "partnership rental activity"] {
        assert!(
            result.issues.iter().any(|i| i.message.contains(text)),
            "{text}: {:?}",
            result.issues
        );
    }
}

// T2-31: Schedule K-1 (1120-S) box 12 code A, box 16 code D above stock
// basis and the named refusals. Every expected amount below is worked by
// hand from the Shareholder's Instructions for Schedule K-1 (Form 1120-S)
// (boxes 4, 5a/5b, 12A, 13O, 16A, 16D), the Form 7203 instructions (Part I
// increases, then distributions with the excess as capital gain, then
// losses and deductions against stock and debt basis), the Form 6198
// instructions (line 2a disposition gain), the 2026 Form 1040 instructions
// and Rev. Proc. 2025-32 (single: $16,100 standard deduction and the $1,000
// line 12f cash-gift allowance; 10% to $12,400, 12% to $50,400, 22% to
// $105,700; qualified dividends and capital gain 0% to $49,450).

// Materially participating shareholder, box 1 `profit`, opening stock basis
// $10,000 and one $3,000 written note at full basis (debt basis $3,000);
// retained Form 6198 amount at risk $13,000 (stock and the note), no
// prior deducted losses; QBI statement equal to box 1.
fn shareholder_k1(profit: i64) -> ScorpK1 {
    let mut source = complete_ordinary_scorp(profit, true);
    let risk = &mut source.risk;
    risk.start = ScorpRiskStart::Detailed { capacity: 1_300_000, profit: 0, loss: 0 };
    risk.deducted = 0;
    source
}

fn shareholder_draft(mut source: ScorpK1) -> FederalDraft {
    source.amt = Some(amt_records(&source));
    let mut d = complete(FederalStatus::Single);
    add(
        &mut d,
        income(FederalIncome::Scorp {
            data: crate::tests::support::builders::scorp_source(source),
        }),
    );
    d
}

#[test]
fn shareholder_k1_items_and_distribution_above_basis_reach_the_return_by_hand() {
    let mut source = shareholder_k1(20_000); // box 1 $20,000
    source.interest = 120_000; // box 4 $1,200
    source.dividends = 200_000; // box 5a $2,000
    source.qualified_dividends = 150_000; // box 5b $1,500
    source.cash_charity = 100_000; // box 12 code A $1,000
    source.withholding = 24_000; // box 13 code O $240
    source.exempt_interest = 40_000; // box 16 code A $400
    source.distributions = 3_500_000; // box 16 code D $35,000
    source.stock_term = Some(GainTerm::Long);
    // Form 7203 Part I: $10,000 + ($20,000 + $1,200 + $2,000 + $400) =
    // $33,600 before distributions; the $35,000 distribution leaves $0 and
    // $1,400 capital gain. Part II/III: the $1,000 gift passes through the
    // note's $3,000 debt basis (ending $2,000); nothing is suspended.
    let basis = source.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.distribution_gain.get()), 1_400);
    assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 0);
    assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].basis.get()), 2_000);
    assert_eq!(crate::kernel::ratio::dollars(basis.allowed[K1LossClass::Charity].get()), 1_000);
    // Form 6198: $13,000 - $35,000 + $400 = ($21,600); income $20,000 +
    // $1,200 + $2,000 + $1,400 = $24,600; year end $3,000, no recapture.
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(
        (
            risk.investment.map(crate::kernel::ratio::dollars),
            crate::kernel::ratio::dollars(risk.income.get())
        ),
        (Some(-21_600), 24_600)
    );
    assert_eq!(
        (
            risk.year_end.map(crate::kernel::ratio::dollars),
            crate::kernel::ratio::dollars(risk.recapture.income.get())
        ),
        (Some(3_000), 0)
    );
    let draft = shareholder_draft(source);
    let result = supported(&draft);
    // Lines 2a $400, 2b $1,200, 3a $1,500, 3b $2,000.
    assert_eq!(return_line(&result, "2a"), 40_000);
    assert_eq!(return_line(&result, "2b"), 120_000);
    assert_eq!(return_line(&result, "3a"), 150_000);
    assert_eq!(return_line(&result, "3b"), 200_000);
    // Form 8949 Part II and Schedule D line 10/16: $1,400 long-term.
    assert_eq!(return_line(&result, "7a"), 140_000);
    // Schedule 1 line 5 / Schedule E line 32: box 1 $20,000 nonpassive.
    assert_eq!(return_line(&result, "8"), 2_000_000);
    // $60,000 + $1,200 + $2,000 + $1,400 + $20,000.
    assert_eq!(return_line(&result, "11a / 11b"), 8_460_000);
    assert_eq!(return_line(&result, "12e"), 1_610_000);
    assert_eq!(return_line(&result, "12f"), 100_000);
    // QBI 20% x $20,000 = $4,000; limit 20% x ($84,600 - $17,100 - $2,900
    // net capital gain) = $12,920 does not bind.
    assert_eq!(return_line(&result, "13b"), 400_000);
    // $84,600 - $16,100 - $1,000 - $4,000.
    assert_eq!(return_line(&result, "15"), 6_350_000);
    // Worksheet: ordinary $63,500 - $2,900 = $60,600 by the Tax Table
    // (midpoint $60,625: $1,240 + $4,560 + 22% x $10,225 = $8,049.50,
    // rounded $8,050) plus 15% x $2,900 = $435; $8,485 is below the $8,688
    // table tax on all $63,500.
    assert_eq!(return_line(&result, "16"), 848_500);
    assert_eq!(return_line(&result, "25c"), 24_000);
    // $8,485 - ($6,500 W-2 + $240 backup withholding).
    assert_eq!(result.balance, Some(174_500));
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn shareholder_cash_gift_exactly_absorbed_by_basis_is_allowed_and_one_dollar_more_carries() {
    // Box 1 zero; no note; opening stock basis $1,000 and box 12 code A
    // $1,000: Form 7203 Part III allows the whole gift. At $1,001, $1 stays
    // suspended by section 1366(d) and needs a carryforward.
    let mut source = shareholder_k1(0);
    source.loans.clear();
    source.amt = None; // rebuilt by shareholder_draft for the changed loans
    source.opening = Some(100_000);
    source.cash_charity = 100_000;
    let basis = source.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 0);
    assert_eq!(crate::kernel::ratio::dollars(basis.allowed[K1LossClass::Charity].get()), 1_000);
    let result = supported(&shareholder_draft(source.clone()));
    // $60,000 - $16,100 - $1,000 = $42,900; midpoint $42,925: $1,240 +
    // 12% x $30,525 = $4,903.
    assert_eq!(return_line(&result, "12f"), 100_000);
    assert_eq!(return_line(&result, "15"), 4_290_000);
    assert_eq!(return_line(&result, "16"), 490_300);
    // $1,001: the gift takes the whole $1,000 and the last dollar carries.
    source.cash_charity = 100_100;
    let basis = source.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.allowed[K1LossClass::Charity].get()), 1_000);
    assert_eq!(crate::kernel::ratio::dollars(basis.suspended[K1LossClass::Charity].get()), 1);
    let result = supported(&shareholder_draft(source));
    assert_eq!(return_line(&result, "12f"), 100_000);
    assert!(
        result.review_notes.iter().any(
            |v| v.contains("cash contributions carried to 2027 — $1 because of your basis, $0")
        ),
        "{:?}",
        result.review_notes
    );
}

#[test]
fn shareholder_cash_gift_above_the_non_itemizer_cap_is_limited_to_1000() {
    let mut source = shareholder_k1(20_000);
    source.cash_charity = 150_000;
    let result = supported(&shareholder_draft(source));
    assert_eq!(return_line(&result, "12f"), 100_000);
}

#[test]
fn shareholder_distribution_equal_to_basis_has_no_gain_and_one_dollar_more_is_capital_gain() {
    // Part I: $10,000 + $20,000 box 1 = $30,000 before distributions.
    let mut source = shareholder_k1(20_000);
    source.distributions = 3_000_000;
    let basis = source.basis_result().unwrap();
    assert_eq!(
        (
            crate::kernel::ratio::dollars(basis.stock.get()),
            crate::kernel::ratio::dollars(basis.distribution_gain.get())
        ),
        (0, 0)
    );
    let result = supported(&shareholder_draft(source.clone()));
    // $80,000 - $16,100 - $4,000 QBI = $59,900; midpoint $59,925: $5,800 +
    // 22% x $9,525 = $7,895.50, rounded $7,896.
    assert_eq!(return_line(&result, "15"), 5_990_000);
    assert_eq!(return_line(&result, "16"), 789_600);
    // $30,001: a $1 gain whose character needs the shares' holding period;
    // without an answer it is one year or less, with a warning (unit B9-1).
    source.distributions = 3_000_100;
    let assumed = supported(&shareholder_draft(source.clone()));
    assert!(noted(&assumed, "WARN_K1_TERM_ASSUMED"));
    assert_eq!(return_line(&assumed, "7a"), 100);
    source.stock_term = Some(GainTerm::Short);
    let result = supported(&shareholder_draft(source));
    // Short-term $1: AGI $80,001; QBI limit 20% x $63,901 = $12,780 does not
    // bind; taxable $59,901, the same $59,900-$59,950 table row.
    assert_eq!(return_line(&result, "7a"), 100);
    assert_eq!(return_line(&result, "15"), 5_990_100);
    assert_eq!(return_line(&result, "16"), 789_600);
}

#[test]
fn shareholder_distribution_gain_meets_the_investment_tax_threshold() {
    // Box 1 $10,000; Part I $20,000 before distributions; $20,001 leaves a
    // $1 long-term gain. With wages $189,999 modified AGI is exactly the
    // $200,000 single threshold and no Form 8960 tax applies; one more
    // dollar of wages needs the section 1411(c)(4) disposition adjustment.
    let draft = |wages: i64| {
        let mut source = shareholder_k1(10_000);
        source.distributions = 2_000_100;
        source.stock_term = Some(GainTerm::Long);
        let mut d = shareholder_draft(source);
        d.income[0] = wage(1, Owner::Taxpayer, wages * 100, 650_000);
        d
    };
    let result = supported(&draft(189_999));
    assert_eq!(return_line(&result, "11a / 11b"), 20_000_000);
    unsupported(&draft(190_000), "K-1 gain on the interest");
}

#[test]
fn shareholder_boxes_not_calculated_and_passive_distribution_gain_are_refused_by_name() {
    let mut source = shareholder_k1(20_000);
    source.unentered = vec![ScorpK1Item::OtherIncome, ScorpK1Item::RentalDisposition];
    let draft = shareholder_draft(source);
    unsupported(&draft, "Income: Synthetic document");
    let result = estimate(&draft);
    for text in [
        "box 10 codes B, D, E, F, G, I",
        "rental activity's installment sale, like-kind exchange, casualty or theft, or disposition to a related buyer",
    ] {
        assert!(
            result.issues.iter().any(|i| i.message.contains(text)),
            "{text}: {:?}",
            result.issues
        );
    }
    assert!(
        result.issues.iter().any(|issue| issue.display.key == "ISSUE_SCORP_K1_RENTAL_DISPOSITION")
    );
    // A passive shareholder's gain above basis is taken as not passive
    // (unit B9-1) and no longer refused for Form 8582.
    let mut source = shareholder_k1(20_000);
    source.participation.as_mut().unwrap().minutes = 0;
    source.distributions = 3_000_100;
    source.stock_term = Some(GainTerm::Long);
    let result = estimate(&shareholder_draft(source));
    assert!(
        result.issues.iter().all(|i| i.section != "S-corporation distribution"),
        "{:?}",
        result.issues
    );
}

// T2-29b: Schedule K-1 (1065) box 12 and (1120-S) box 11 section 179 through
// Form 4562 Part I. Every expected amount below is worked by hand from the
// 2025 Instructions for Form 4562 (Part I lines 1-13; the 2026 draft form
// keeps the same lines), Regulations sections 1.179-2(b)(3) (the K-1 share
// joins the owner's line 8 but the entity's property is not the owner's line
// 2 cost), 1.179-2(c)(2)(v) (the partner's share of the partnership's
// active taxable income is line 11 income) and 1.179-3(h)(1) (the full
// allocated amount reduces the owner's basis whether or not it is deductible
// this year), the Partner's Instructions for Schedule K-1 (Form 1065)
// (box 12 to Schedule E line 28 column (j); box 14 code A reduced by the
// section 179 claimed), the Shareholder's Instructions for Schedule K-1
// (Form 1120-S) box 11, the Instructions for Form 8995 (a section 179 amount
// suspended by section 179(b)(3) is not QBI until allowed), the 2026 Form
// 1040 instructions and Rev. Proc. 2025-32 (single: $16,100 standard
// deduction; 10% to $12,400, 12% to $50,400, 22% to $105,700).

// Materially participating partner (limited-partner SE classification, box
// 14A zero), box 1 `profit`, box 12 `section179`, QBI statement equal to box
// 1, outside basis and amount at risk $10,000 at the start of the year.
fn section179_partner(profit: i64, section179: i64) -> PartnerK1 {
    let mut partner = ordinary_partner_source(profit, true);
    partner.qbi = Some(partner_qbi_statement(profit));
    partner.section179 = section179 * 100;
    partner
}

fn set_partner_risk_capacity(partner: &mut PartnerK1, capacity: i64) {
    for record in core::iter::once(&mut partner.regular).chain(partner.alternative.as_mut()) {
        record.risk.start =
            ScorpRiskStart::Detailed { capacity: capacity * 100, profit: 0, loss: 0 };
    }
}

#[test]
fn partner_section179_reaches_form_4562_schedule_e_basis_and_qbi_by_hand() {
    let partner = section179_partner(20_000, 5_000); // box 1 $20,000, box 12 $5,000
    // Basis worksheet: line 1 $10,000 + box 1 $20,000 = $30,000; Part II
    // section 179 $5,000 (the full K-1 share, Reg. 1.179-3(h)(1)); ending
    // $25,000. At risk before the deduction: $10,000 + $20,000 = $30,000.
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 25_000);
    assert_eq!(
        pool.activity.basis.allowed,
        LossAmounts::of(&[(
            K1LossClass::Section179,
            crate::kernel::carry::Amount::new(5_000 * 100).unwrap()
        )])
    );
    assert_eq!(pool.activity.risk_end.map(crate::kernel::ratio::dollars), Some(30_000));
    let draft = separate_items_draft(partner);
    let result = supported(&draft);
    // Form 4562 Part I: line 1 $2,560,000; line 2 $0 (the partnership's
    // property is not the partner's cost); line 5 $2,560,000; line 6 "from
    // Schedule K-1 (Form 1065)" $5,000; lines 8 and 9 $5,000; line 10 $0;
    // line 11 = smaller of line 5 or W-2 $60,000 + box 1 $20,000 = $80,000;
    // line 12 $5,000; line 13 $0.
    assert_eq!(value(&result, "section179_eligible_cost"), 0);
    assert_eq!(value(&result, "section179_dollar_limit"), 2_560_000);
    assert_eq!(value(&result, "section179_business_income_limit"), 80_000);
    assert_eq!(value(&result, "section179_deduction"), 5_000);
    assert_eq!(value(&result, "section179_carryforward"), 0);
    assert_eq!(value(&result, "partnership_2_section179"), 5_000);
    assert_eq!(value(&result, "partnership_2_section179_carryforward"), 0);
    // Schedule E line 28: column (k) $20,000, column (j) $5,000; line 32
    // $15,000 to Schedule 1 line 5.
    assert_eq!(return_line(&result, "8"), 1_500_000);
    // $60,000 + $15,000.
    assert_eq!(return_line(&result, "11a / 11b"), 7_500_000);
    assert_eq!(return_line(&result, "12e"), 1_610_000);
    // QBI: statement $20,000 less the $5,000 allowed = $15,000; 20% = $3,000;
    // the taxable income limit 20% x ($75,000 - $16,100) = $11,780 does not bind.
    assert_eq!(return_line(&result, "13b"), 300_000);
    // $75,000 - $16,100 - $3,000.
    assert_eq!(return_line(&result, "15"), 5_590_000);
    // Tax Table row $55,900-$55,950, midpoint $55,925: $1,240 + $4,560 +
    // 22% x $5,525 = $7,015.50, rounded $7,016.
    assert_eq!(return_line(&result, "16"), 701_600);
    // $7,016 - $6,500 W-2 withholding.
    assert_eq!(result.balance, Some(51_600));
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn general_partner_section179_claimed_reduces_box_14a_earnings() {
    // Box 14A $20,000 for a general partner; the $5,000 allowed on Form 4562
    // line 12 is section 179 "claimed" and reduces Schedule SE earnings to
    // $15,000 (Partner's Instructions, box 14 code A). A limited partner's
    // box 14A already excludes it.
    let mut partner = section179_partner(20_000, 5_000);
    partner.limited_for_se = Some(false);
    partner.reported_se = Some(2_000_000);
    let se = partner
        .se_after_passive(
            crate::kernel::carry::Amount::new(0).unwrap(),
            crate::kernel::carry::Amount::new(5_000 * 100).unwrap(),
        )
        .unwrap();
    assert_eq!(se.earnings, 15_000);
    assert_eq!(
        partner
            .se_after_passive(
                crate::kernel::carry::Amount::new(0).unwrap(),
                crate::kernel::carry::Amount::new(0).unwrap()
            )
            .unwrap()
            .earnings,
        20_000
    );
    // More than the K-1 share cannot be claimed.
    assert!(
        partner
            .se_after_passive(
                crate::kernel::carry::Amount::new(0).unwrap(),
                crate::kernel::carry::Amount::new(5_001 * 100).unwrap()
            )
            .is_err()
    );
    let mut limited = section179_partner(20_000, 5_000);
    limited.reported_se = Some(0);
    assert_eq!(
        limited
            .se_after_passive(
                crate::kernel::carry::Amount::new(0).unwrap(),
                crate::kernel::carry::Amount::new(5_000 * 100).unwrap()
            )
            .unwrap()
            .earnings,
        0
    );
    // On the return the SE source note carries the reduced earnings.
    partner.se_gross = Some(PartnerSeGross::Nonfarm { gross: 2_000_000 });
    partner.qbi_records.qualified_se_gross = 2_000_000;
    let result = supported(&separate_items_draft(partner));
    assert!(
        result.review_notes.iter().any(|n| n.starts_with(
            "Partnership document 2, SE source including taxable guaranteed payments after regular activity limits: $15000;"
        )),
        "{:?}",
        result.review_notes
    );
    let se_note = &result
        .review_notes
        .iter()
        .find(|n| n.starts_with("Partnership document 2, SE source"))
        .unwrap()
        .display;
    assert_eq!(se_note.key, "NOTE_PARTNERSHIP_SE_SOURCE");
    assert_eq!(se_note.args[1], MessageArg::usd_dollars(15_000));
    assert_eq!(value(&result, "partnership_2_section179"), 5_000);
}

#[test]
fn partner_section179_meets_the_business_income_limit_and_carries_the_excess() {
    // Box 1 $5,000, box 12 $70,000, basis and at-risk $100,000. Line 11 =
    // W-2 $60,000 + box 1 $5,000 = $65,000; line 12 $65,000; line 13
    // carryover to 2027 $5,000. Basis still falls by the full $70,000:
    // $100,000 + $5,000 - $70,000 = $35,000; at risk $105,000 covers the
    // $65,000 deduction.
    let mut partner = section179_partner(5_000, 70_000);
    set_partner_start(&mut partner, 100_000, 0);
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 35_000);
    assert_eq!(pool.activity.risk_end.map(crate::kernel::ratio::dollars), Some(105_000));
    let result = supported(&separate_items_draft(partner));
    assert_eq!(value(&result, "section179_business_income_limit"), 65_000);
    assert_eq!(value(&result, "section179_deduction"), 65_000);
    assert_eq!(value(&result, "section179_carryforward"), 5_000);
    assert_eq!(value(&result, "partnership_2_section179"), 65_000);
    assert_eq!(value(&result, "partnership_2_section179_carryforward"), 5_000);
    // Schedule E line 28: (k) $5,000, j $65,000; line 32 ($60,000).
    assert_eq!(return_line(&result, "8"), -6_000_000);
    // AGI $60,000 - $60,000 = $0; QBI $5,000 - $65,000 is a qualified loss
    // carried forward, no deduction; no tax.
    assert_eq!(return_line(&result, "11a / 11b"), 0);
    assert_eq!(return_line(&result, "13b"), 0);
    assert_eq!(return_line(&result, "16"), 0);
    // Exactly at the limit nothing carries; one dollar more carries $1.
    for (section179, carry) in [(65_000, 0), (65_001, 1)] {
        let mut partner = section179_partner(5_000, section179);
        set_partner_start(&mut partner, 100_000, 0);
        let result = supported(&separate_items_draft(partner));
        assert_eq!(value(&result, "section179_deduction"), 65_000);
        assert_eq!(value(&result, "partnership_2_section179_carryforward"), carry);
    }
}

#[test]
fn partner_section179_exactly_absorbed_by_basis_is_allowed_and_one_dollar_less_carries() {
    // Box 1 zero, box 12 $5,000 against a $5,000 opening basis and amount at
    // risk: Part II allows the whole class; ending basis $0. Line 11 is the
    // $60,000 of wages, so Form 4562 allows all $5,000.
    let mut partner = section179_partner(0, 5_000);
    set_partner_start(&mut partner, 5_000, 0);
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 0);
    assert_eq!(
        pool.activity.basis.allowed,
        LossAmounts::of(&[(
            K1LossClass::Section179,
            crate::kernel::carry::Amount::new(5_000 * 100).unwrap()
        )])
    );
    let result = supported(&separate_items_draft(partner.clone()));
    // Schedule E ($5,000); AGI $55,000; the QBI statement $0 less $5,000 is
    // a qualified loss carryforward; taxable $55,000 - $16,100 = $38,900;
    // midpoint $38,925: $1,240 + 12% x $26,525 = $4,423.
    assert_eq!(return_line(&result, "8"), -500_000);
    assert_eq!(return_line(&result, "15"), 3_890_000);
    assert_eq!(return_line(&result, "16"), 442_300);
    // $4,999 of basis: the basis allows $4,999, which is the Form 4562 line 6
    // item, and the last dollar carries by basis to 2027 (column E).
    set_partner_start(&mut partner, 4_999, 0);
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(
        crate::kernel::ratio::dollars(
            pool.activity.ledger.basis_allowed[K1LossClass::Section179].get()
        ),
        4_999
    );
    assert_eq!(
        crate::kernel::ratio::dollars(
            pool.activity.ledger.basis_suspended[K1LossClass::Section179].get()
        ),
        1
    );
    let result = supported(&separate_items_draft(partner));
    assert_eq!(value(&result, "partnership_2_section179"), 4_999);
    assert_eq!(return_line(&result, "8"), -499_900);
    assert!(
        result.review_notes.iter().any(|v| v.contains(
            "section 179 deduction carried to 2027 — $1 because of your basis, $0 because of the at-risk limits"
        )),
        "{:?}",
        result.review_notes
    );
}

#[test]
fn partner_section179_exactly_within_the_amount_at_risk_is_allowed_and_one_dollar_less_is_refused()
{
    // Outside basis $10,000 but only $5,000 at risk (Form 6198): the $5,000
    // allowed on Form 4562 line 12 fits exactly; at $4,999 the excess needs
    // an at-risk carryforward kept by character.
    let mut partner = section179_partner(0, 5_000);
    set_partner_risk_capacity(&mut partner, 5_000);
    assert_eq!(
        partner
            .pool(&partner.regular)
            .unwrap()
            .activity
            .risk_end
            .map(crate::kernel::ratio::dollars),
        Some(5_000)
    );
    let result = supported(&separate_items_draft(partner.clone()));
    assert_eq!(value(&result, "partnership_2_section179"), 5_000);
    set_partner_risk_capacity(&mut partner, 4_999);
    let draft = separate_items_draft(partner);
    unsupported(&draft, "Partnership section 179");
    let refused = estimate(&draft);
    assert!(
        refused
            .issues
            .iter()
            .any(|i| i.section == "Partnership section 179" && i.message.contains("Form 6198")),
        "{:?}",
        refused.issues
    );
}

#[test]
fn partner_section179_meets_the_dollar_limit_and_one_dollar_more_is_refused() {
    // Box 1 $3,000,000 and box 12 $2,560,000 on a $3,000,000 basis: line 2
    // is $0 (no phaseout from the partnership's property), line 5
    // $2,560,000, line 11 $3,060,000; line 12 $2,560,000. One more dollar
    // exceeds the dollar limit and is lost, not carried.
    let mut partner = section179_partner(3_000_000, 2_560_000);
    set_partner_start(&mut partner, 3_000_000, 0);
    let result = supported(&separate_items_draft(partner));
    assert_eq!(value(&result, "section179_dollar_limit"), 2_560_000);
    assert_eq!(value(&result, "section179_deduction"), 2_560_000);
    assert_eq!(value(&result, "partnership_2_section179_carryforward"), 0);
    let mut partner = section179_partner(3_000_000, 2_560_001);
    set_partner_start(&mut partner, 3_000_000, 0);
    unsupported(&separate_items_draft(partner), "Section 179");
}

#[test]
fn partner_section179_situations_not_calculated_are_refused_by_name() {
    let refused = |partner: PartnerK1, section: &str, text: &str| {
        let result = estimate(&separate_items_draft(partner));
        assert_eq!(result.state, ResultState::Unsupported, "{result:?}");
        assert!(
            result.issues.iter().any(|i| i.section == section && i.message.contains(text)),
            "{section} / {text}: {:?}",
            result.issues
        );
    };
    // A partner without material participation: the deduction joins the
    // activity's Form 8582 net ($20,000 - $5,000), allowed in full.
    let mut passive = ordinary_partner_source(20_000, false);
    passive.qbi = Some(partner_qbi_statement(20_000));
    passive.section179 = 500_000;
    let result = supported(&separate_items_draft(passive));
    assert_eq!(value(&result, "partnership_2_section179"), 5_000);
    assert_eq!(value(&result, "partnership_2_section179_passive_allowed"), 5_000);
    assert_eq!(value(&result, "partnership_2_section179_passive_unallowed"), 0);
    // 2026-10-16 D6 source coordination supersedes the blanket PTP refusal.
    // Preserve this original source's absent AMT income-carry facts: missing
    // history remains unavailable rather than becoming an invented zero.
    let mut ptp = section179_partner(20_000, 5_000);
    ptp.publicly_traded = true;
    let result = estimate(&separate_items_draft(ptp));
    assert_eq!(result.state, ResultState::Unsupported);
    assert!(result.issues.iter().any(|issue| {
        issue.display.key == "ISSUE_SECTION179_SELECTED_SOURCE_UNAVAILABLE"
            && matches!(&issue.context, crate::IssueContext::Corrective { source, sections }
                if sections.is_empty() && matches!(source.as_ref(), crate::IssueContext::Income { id: 2, .. }))
    }));
    // 2026-10-19: approved depreciation map R3 applies Reg.1.179-2(c)(2),6
    // to active services compensation separately from entity income. This
    // supersedes the former blanket box4a refusal: 60000+5000+1000=66000,
    // leaving4000 of the70000 election in its existing income-limit carry.
    let mut guaranteed = section179_partner(5_000, 70_000);
    set_partner_start(&mut guaranteed, 100_000, 0);
    guaranteed.guaranteed_services = 100_000;
    guaranteed.reported_se = Some(100_000);
    let services = supported(&separate_items_draft(guaranteed));
    assert_eq!(value(&services, "section179_business_income_limit"), 66_000);
    assert_eq!(value(&services, "section179_deduction"), 66_000);
    assert_eq!(value(&services, "section179_carryforward"), 4_000);
    // Partner health insurance with a K-1 section 179 share.
    let mut health = partner_health_source(6_000, false);
    health.section179 = 500_000;
    refused(health, "Partnership section 179", "health-insurance or retirement");
}

// Materially participating shareholder from the T2-31 fixture (box 1
// `profit`, opening stock basis $10,000, a $3,000 note at full basis, amount
// at risk $13,000) with box 11 `section179`.
fn section179_shareholder(profit: i64, section179: i64) -> ScorpK1 {
    let mut source = shareholder_k1(profit);
    source.section179 = section179 * 100;
    source
}

#[test]
fn shareholder_section179_reaches_form_4562_schedule_e_basis_and_qbi_by_hand() {
    let source = section179_shareholder(20_000, 5_000); // box 1 $20,000, box 11 $5,000
    // Form 7203: Part I $10,000 + $20,000 = $30,000; Part III section 179
    // $5,000 against stock (the full K-1 share, Reg. 1.179-3(h)(1)); ending
    // stock basis $25,000, the note's $3,000 debt basis untouched. Form
    // 6198 before the deduction: $13,000 + $20,000 = $33,000.
    let basis = source.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 25_000);
    assert_eq!(crate::kernel::ratio::dollars(basis.allowed[K1LossClass::Section179].get()), 5_000);
    assert_eq!(crate::kernel::ratio::dollars(basis.loans[0].basis.get()), 3_000);
    let risk = source.risk.calculate(&source, &basis).unwrap();
    assert_eq!(risk.year_end.map(crate::kernel::ratio::dollars), Some(33_000));
    let draft = shareholder_draft(source);
    let result = supported(&draft);
    // Form 4562 Part I: line 2 $0; line 5 $2,560,000; line 6 "from Schedule
    // K-1 (Form 1120-S)" $5,000; line 11 = W-2 $60,000 + box 1 $20,000 =
    // $80,000; line 12 $5,000; line 13 $0.
    assert_eq!(value(&result, "section179_eligible_cost"), 0);
    assert_eq!(value(&result, "section179_business_income_limit"), 80_000);
    assert_eq!(value(&result, "section179_deduction"), 5_000);
    assert_eq!(value(&result, "scorp_2_section179"), 5_000);
    assert_eq!(value(&result, "scorp_2_section179_carryforward"), 0);
    // Schedule E line 28: (k) $20,000, j $5,000; line 32 $15,000.
    assert_eq!(return_line(&result, "8"), 1_500_000);
    assert_eq!(return_line(&result, "11a / 11b"), 7_500_000);
    // QBI $20,000 - $5,000 = $15,000; 20% = $3,000.
    assert_eq!(return_line(&result, "13b"), 300_000);
    // $75,000 - $16,100 - $3,000; midpoint $55,925: $1,240 + $4,560 + 22% x
    // $5,525 = $7,015.50, rounded $7,016; less $6,500 withheld.
    assert_eq!(return_line(&result, "15"), 5_590_000);
    assert_eq!(return_line(&result, "16"), 701_600);
    assert_eq!(result.balance, Some(51_600));
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn shareholder_section179_exactly_absorbed_by_basis_is_allowed_and_one_dollar_more_carries() {
    // Box 1 zero, no note, opening stock basis $5,000 and box 11 $5,000:
    // Form 7203 Part III allows the whole class (ending basis $0); the
    // $13,000 at risk and the $60,000 of wages on line 11 cover it. At
    // $5,001, $1 is suspended by section 1366(d).
    let mut source = section179_shareholder(0, 5_000);
    source.loans.clear();
    source.amt = None; // rebuilt by shareholder_draft for the changed loans
    source.opening = Some(500_000);
    let basis = source.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 0);
    assert_eq!(crate::kernel::ratio::dollars(basis.allowed[K1LossClass::Section179].get()), 5_000);
    let result = supported(&shareholder_draft(source.clone()));
    // Schedule E ($5,000); AGI $55,000; taxable $38,900; midpoint $38,925:
    // $1,240 + 12% x $26,525 = $4,423.
    assert_eq!(return_line(&result, "8"), -500_000);
    assert_eq!(return_line(&result, "16"), 442_300);
    // $5,001: the basis allows $5,000, the Form 4562 line 6 item, and the
    // last dollar carries.
    source.section179 = 500_100;
    let basis = source.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.allowed[K1LossClass::Section179].get()), 5_000);
    assert_eq!(crate::kernel::ratio::dollars(basis.suspended[K1LossClass::Section179].get()), 1);
    let result = supported(&shareholder_draft(source));
    assert_eq!(return_line(&result, "8"), -500_000);
    assert!(
        result.review_notes.iter().any(|v| v.contains(
            "section 179 deduction carried to 2027 — $1 because of your basis, $0 because of the at-risk limits"
        )),
        "{:?}",
        result.review_notes
    );
}

#[test]
fn shareholder_section179_without_material_participation_joins_form_8582() {
    // Box 1 $20,000 less box 11 $5,000 is the passive activity's net income.
    let mut source = complete_ordinary_scorp(20_000, false);
    source.section179 = 500_000;
    let result = supported(&shareholder_draft(source));
    assert_eq!(value(&result, "scorp_2_section179"), 5_000);
    assert_eq!(value(&result, "scorp_2_section179_passive_allowed"), 5_000);
    assert_eq!(value(&result, "scorp_2_section179_passive_unallowed"), 0);
}

// T2-31p (question C13): a Schedule K-1 (1120-S) section 179 carryover
// from 2025 enters Form 4562 line 10. Worked by hand from the 2025
// Instructions for Form 4562 Part I (line 10 is the line 13 carryover of
// the prior year; line 12 is the smaller of line 9 + line 10 and line 11;
// line 13 = line 9 + line 10 - line 12), Reg. 1.179-3(a)-(b) (a carryover
// is deducted in the year allowed within the dollar and income limits and
// never reduces basis again), the Instructions for Form 8995 (a section 179
// amount disallowed after 2017 becomes QBI when allowed) and the 2026 Form
// 1040 figures of the T2-29b fixture above.
#[test]
fn shareholder_section179_carryover_joins_form_4562_line_10_by_hand() {
    // Box 1 $20,000, box 11 $5,000, carryover $2,000 (T2-29b fixture).
    //   Line 8 $5,000; line 9 $5,000; line 10 $2,000; line 11 $60,000 W-2 +
    //   $20,000 box 1 = $80,000; line 12 $7,000; line 13 $0. Form 7203 is
    //   unchanged: ending stock basis $25,000 (the carryover reduced basis
    //   in 2025). Form 6198 $33,000 covers the $7,000.
    //   Schedule E line 28 (j) $7,000: $20,000 - $7,000 = $13,000.
    //   AGI $73,000; $73,000 - $16,100 = $56,900; QBI 20% x ($20,000 -
    //   $7,000) = $2,600 (limit 20% x $56,900 = $11,380); taxable $54,300.
    //   Tax Table midpoint $54,325: $1,240 + $4,560 + 22% x $3,925 =
    //   $6,663.50, rounded $6,664; balance $6,664 - $6,500 = $164.
    let mut source = section179_shareholder(20_000, 5_000);
    source.section179_carryover = 200_000;
    let basis = source.basis_result().unwrap();
    assert_eq!(crate::kernel::ratio::dollars(basis.stock.get()), 25_000);
    let draft = shareholder_draft(source);
    let result = supported(&draft);
    assert_eq!(value(&result, "section179_deduction"), 7_000);
    assert_eq!(value(&result, "scorp_2_section179_prior_carryover"), 2_000);
    assert_eq!(value(&result, "scorp_2_section179"), 7_000);
    assert_eq!(value(&result, "scorp_2_section179_carryforward"), 0);
    assert_eq!(return_line(&result, "8"), 1_300_000);
    assert_eq!(return_line(&result, "11a / 11b"), 7_300_000);
    assert_eq!(return_line(&result, "13b"), 260_000);
    assert_eq!(return_line(&result, "15"), 5_430_000);
    assert_eq!(return_line(&result, "16"), 666_400);
    assert_eq!(result.balance, Some(16_400));
    assert_eq!(estimate(&restart(draft).draft), result);
}

// Box 1 zero, no box 11, W-2 wages $3,000 ($0 withheld) and a carryover:
// line 11 is $3,000, so a $3,000 carryover is allowed in full and one more
// dollar is carried again to 2027 (line 13 $1).
fn carryover_only_draft(carryover: i64) -> FederalDraft {
    let mut source = section179_shareholder(0, 0);
    source.section179_carryover = carryover * 100;
    let mut d = shareholder_draft(source);
    d.income[0] = wage(1, Owner::Taxpayer, 300_000, 0);
    d
}

#[test]
fn shareholder_section179_carryover_equal_to_the_income_limit_is_allowed_in_full() {
    let result = supported(&carryover_only_draft(3_000));
    assert_eq!(value(&result, "section179_business_income_limit"), 3_000);
    assert_eq!(value(&result, "section179_deduction"), 3_000);
    assert_eq!(value(&result, "scorp_2_section179_carryforward"), 0);
    // Schedule E: $0 - $3,000; AGI $3,000 - $3,000 = $0.
    assert_eq!(return_line(&result, "8"), -300_000);
    assert_eq!(return_line(&result, "11a / 11b"), 0);
}

#[test]
fn shareholder_section179_carryover_one_dollar_above_the_income_limit_carries_again() {
    let result = supported(&carryover_only_draft(3_001));
    assert_eq!(value(&result, "section179_deduction"), 3_000);
    assert_eq!(value(&result, "scorp_2_section179"), 3_000);
    assert_eq!(value(&result, "scorp_2_section179_carryforward"), 1);
    assert_eq!(value(&result, "section179_carryforward"), 1);
    assert_eq!(return_line(&result, "8"), -300_000);
}

// Main's hand-worked boxes 6–8 fixtures under signed activity source records.
fn estate_trust_activity_draft(wages: i64) -> FederalDraft {
    let mut draft = estate_trust_code_c_draft(0, 0);
    draft.income[0] = wage(1, Owner::Taxpayer, wages, 650_000);
    draft.next_id = 6;
    let mut data = estate_trust_code_d_data(&mut draft);
    data.activities = crate::EstateActivities {
        box6: 0,
        box7: 0,
        box8: 0,
        activities: [
            crate::EstateActivityKind::Business,
            crate::EstateActivityKind::RentalRealEstate,
            crate::EstateActivityKind::OtherRental,
        ]
        .into_iter()
        .enumerate()
        .map(|(i, kind)| crate::EstateActivity {
            id: 3 + i as i64,
            name: ["Business", "Real estate", "Other rental"][i].into(),
            kind,
            income: 0,
            depreciation: 0,
            depletion: 0,
            amortization: 0,
            participation: crate::EstateParticipation::Passive,
            carried: crate::EstatePassiveCarry { regular: 0, alternative: 0 },
            amt_difference: 0,
        })
        .collect(),
    };
    drop(data);
    draft
}
fn estate_fixture_incomes(
    data: &mut FederalEstateTrustPortfolioK1,
    incomes: [i64; 3],
    deductions: [i64; 3],
    material: bool,
) {
    data.activities.box6 = incomes[0];
    data.activities.box7 = incomes[1];
    data.activities.box8 = incomes[2];
    for (i, a) in data.activities.activities.iter_mut().enumerate() {
        a.income = incomes[i];
        a.depreciation = deductions[i];
    }
    data.activities.activities[0].participation = if material {
        crate::EstateParticipation::Material(ScorpParticipation {
            minutes: 30_001,
            ..participation_facts()
        })
    } else {
        crate::EstateParticipation::Passive
    };
}
fn estate_fixture_qbi(data: &mut FederalEstateTrustPortfolioK1, income: i64, wages: i64) {
    data.qbi.statements = vec![crate::EstateQbiStatement {
        id: 0,
        allocations: Vec::new(),
        name: "Fiduciary business source".into(),
        income,
        wages,
        property: 0,
        service: false,
        kind: crate::EstateQbiKind::Business,
        activities: vec![3],
        passive_history: vec![],
    }];
}
#[test]
fn estate_trust_boxes_6_to_8_flow_to_schedule_e_niit_and_qbi_by_hand() {
    // $60,000 wages+$1,000 interest+$40,000 nonpassive business+
    // ($12,000-$2,000)+$3,000 passive rent=$114,000 AGI. QBI20%*$38,000
    // =$7,600; taxable114,000-16,100-7,600=$90,300; NIIT0.
    let mut draft = estate_trust_activity_draft(6_000_000);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.interest = 100_000;
    estate_fixture_incomes(&mut data, [4_000_000, 1_200_000, 300_000], [0, 200_000, 0], true);
    estate_fixture_qbi(&mut data, 3_800_000, 1_000_000);
    drop(data);
    let r = supported(&draft);
    assert_eq!(value(&r, "schedule_1_rental_income"), 53_000);
    assert_eq!(return_line(&r, "11a / 11b"), 11_400_000);
    assert_eq!(value(&r, "qbi_deduction"), 7_600);
    assert_eq!(return_line(&r, "15"), 9_030_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 0);
    assert_eq!(value(&r, "estate_trust_2_activity_3_33f"), 40_000);
    assert_eq!(value(&r, "estate_trust_2_activity_4_33d"), 12_000);
    assert_eq!(value(&r, "estate_trust_2_activity_4_33c"), 2_000);
    assert_eq!(value(&r, "estate_trust_2_activity_5_33d"), 3_000);
    assert_eq!(estimate(&restart(draft.clone()).draft), r);
    // $250,000 wages: AGI304,000. Wage cap50%*$10,000=$5,000;
    // taxable282,900. NII1,000+13,000=14,000; NIIT3.8%=532.
    let mut high = draft.clone();
    high.income[0] = wage(1, Owner::Taxpayer, 25_000_000, 5_000_000);
    let r = supported(&high);
    assert_eq!(return_line(&r, "11a / 11b"), 30_400_000);
    assert_eq!(value(&r, "qbi_deduction"), 5_000);
    assert_eq!(return_line(&r, "15"), 28_290_000);
    assert_eq!(value(&r, "form_8960_business_rental_income"), 53_000);
    assert_eq!(value(&r, "form_8960_nonpassive_business_adjustment"), -40_000);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 14_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 532);
    estate_trust_code_d_data(&mut high).activities.activities[0].participation =
        crate::EstateParticipation::Passive;
    let r = supported(&high);
    assert_eq!(return_line(&r, "11a / 11b"), 30_400_000);
    assert_eq!(value(&r, "form_8960_nonpassive_business_adjustment"), 0);
    assert_eq!(value(&r, "form_8960_net_investment_income"), 54_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 2_052);
    assert_eq!(value(&r, "qbi_deduction"), 5_000);
}
#[test]
fn estate_trust_box_9_deduction_retains_equal_and_above_income_boundaries() {
    let baseline = supported(&estate_trust_activity_draft(6_000_000));
    let mut draft = estate_trust_activity_draft(6_000_000);
    estate_fixture_incomes(
        &mut estate_trust_code_d_data(&mut draft),
        [0, 500_000, 250_000],
        [0, 500_000, 250_000],
        false,
    );
    let r = supported(&draft);
    assert_eq!(return_line(&r, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    // Approved source design accepts one-cent losses in each box, and the
    // existing passive owner suspends them instead of rejecting the K-1.
    for i in 0..3 {
        for (income, deduction) in [(500_000, 500_001), (0, 1)] {
            let mut over = estate_trust_activity_draft(6_000_000);
            let mut incomes = [0; 3];
            let mut deductions = [0; 3];
            incomes[i] = income;
            deductions[i] = deduction;
            estate_fixture_incomes(
                &mut estate_trust_code_d_data(&mut over),
                incomes,
                deductions,
                false,
            );
            assert!(crate::federal_rules::validate_income_amount(&over.income[1].income).is_ok());
            let r = supported(&over);
            assert_eq!(return_line(&r, "11a / 11b"), return_line(&baseline, "11a / 11b"));
        }
    }
}
#[test]
fn estate_trust_qbi_statement_preserves_independent_source_and_taxable_income_threshold() {
    // At201,750 beforeQBI:20%*40,000=8,000; taxable193,750;
    // NIIT3.8%*17,850=678 whole dollars. At276,750, no wage/UBIA=>0QBI.
    let mut draft = estate_trust_activity_draft(17_785_000);
    let mut data = estate_trust_code_d_data(&mut draft);
    estate_fixture_incomes(&mut data, [4_000_000, 0, 0], [0; 3], false);
    estate_fixture_qbi(&mut data, 4_000_000, 0);
    drop(data);
    let r = supported(&draft);
    assert_eq!(return_line(&r, "11a / 11b"), 21_785_000);
    assert_eq!(value(&r, "qbi_deduction"), 8_000);
    assert_eq!(return_line(&r, "15"), 19_375_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 678);
    let mut phased = draft.clone();
    phased.income[0] = wage(1, Owner::Taxpayer, 25_285_000, 650_000);
    let r = supported(&phased);
    assert_eq!(value(&r, "qbi_deduction"), 0);
    assert_eq!(return_line(&r, "15"), 27_675_000);
    assert_eq!(value(&r, "net_investment_income_tax"), 1_520);
    // QBI source is not capped by taxable DNI. A one-cent difference is
    // retained; both source values round to39,000 and QBI7,800.
    let mut data = estate_trust_code_d_data(&mut draft);
    data.activities.activities[0].depreciation = 100_000;
    data.qbi.statements[0].income = 3_900_001;
    drop(data);
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_ok());
    assert_eq!(value(&supported(&draft), "qbi_deduction"), 7_800);
    estate_trust_code_d_data(&mut draft).qbi.statements[0].income = 3_900_000;
    assert_eq!(value(&supported(&draft), "qbi_deduction"), 7_800);
}
#[test]
fn estate_trust_code_f_opens_the_farmer_test_and_retains_independent_gross_source() {
    let mut draft = estate_trust_activity_draft(6_000_000);
    let mut data = estate_trust_code_d_data(&mut draft);
    estate_fixture_incomes(&mut data, [3_000_000, 0, 0], [0; 3], true);
    data.gross_farming_fishing_income = 3_000_000;
    drop(data);
    let pay = draft.payments.as_mut().unwrap();
    pay.prior_year = Some(PriorYearTax {
        full_year: true,
        tax: 10_000_000,
        agi: Some(10_000_000),
        spouse_tax: 0,
    });
    pay.farmer_fisherman_prior_year = false;
    let r = supported(&draft);
    assert_eq!(value(&r, "farming_fishing_gross_income_return_year"), 30_000);
    assert_eq!(value(&r, "gross_income_return_year"), 90_000);
    assert_eq!(return_line(&r, "11a / 11b"), 9_000_000);
    estate_trust_code_d_data(&mut draft).gross_farming_fishing_income = 3_000_001;
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_ok());
    assert_eq!(value(&supported(&draft), "farming_fishing_gross_income_return_year"), 30_000);
}
#[test]
fn estate_trust_boxes_6_to_8_retain_known_income_with_unresolved_participation() {
    let mut draft = estate_trust_activity_draft(6_000_000);
    let mut data = estate_trust_code_d_data(&mut draft);
    estate_fixture_incomes(&mut data, [1_000_000, 0, 0], [0; 3], false);
    data.activities.activities[0].participation = crate::EstateParticipation::Unresolved;
    estate_fixture_qbi(&mut data, 500_000, 0);
    drop(data);
    let r = supported(&draft);
    assert_eq!(return_line(&r, "11a / 11b"), 7_000_000);
    assert!(noted(&r, "WARN_ESTATE_QBI_SOURCE_LEFT_OUT"));
    estate_trust_code_d_data(&mut draft).activities.activities[0].participation =
        crate::EstateParticipation::Passive;
    assert_eq!(value(&supported(&draft), "qbi_deduction"), 1_000);
    assert_eq!(crate::interview::document_amount(&draft.income[1].income, false), None);
}

// T2-29p question C9: Form 6198 is filed only for a loss (Instructions for
// Form 6198, "Who Must File"), so a continuing partner without 2025
// at-risk figures needs none while the at-risk stage limits nothing: the
// year's losses entering it do not exceed the activity's income, nothing is
// carried at risk from 2025, no box 12 amount waits for its at-risk test and
// no earlier deducted loss is left to recapture.
fn unknown_history_partner(profit: i64) -> PartnerK1 {
    let mut partner = section179_partner(profit, 0);
    set_partner_start(&mut partner, 10_000, 0);
    for record in core::iter::once(&mut partner.regular).chain(partner.alternative.as_mut()) {
        record.risk.start = ScorpRiskStart::Unknown;
    }
    partner
}

#[test]
fn partner_profit_without_at_risk_history_needs_no_form_6198() {
    // Box 1 $20,000: basis $10,000 + $20,000 = $30,000; nothing to limit.
    // Schedule E $20,000; AGI $80,000; QBI $4,000; taxable $80,000 -
    // $16,100 - $4,000 = $59,900; midpoint $59,925: $1,240 + $4,560 + 22% x
    // $9,525 = $7,895.50, rounded $7,896.
    let partner = unknown_history_partner(20_000);
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 30_000);
    assert_eq!(pool.activity.risk_end, None);
    assert_eq!(crate::kernel::ratio::dollars(pool.nonpassive), 20_000);
    let result = supported(&separate_items_draft(partner));
    assert_eq!(return_line(&result, "8"), 2_000_000);
    assert_eq!(return_line(&result, "15"), 5_990_000);
    assert_eq!(return_line(&result, "16"), 789_600);
}

#[test]
fn partner_loss_equal_to_activity_income_needs_no_history_and_one_dollar_more_does() {
    // Box 1 zero, box 5 $1,000 interest and a box 8 loss: at $1,000 the loss
    // is absorbed by the activity's own income; at $1,001 Form 6198 is due.
    let with_loss = |loss: i64| {
        let mut items = partner_separate_items();
        items.interest = 100_000;
        items.short = -loss * 100;
        let mut partner = unknown_history_partner(0);
        partner.qbi = Some(partner_qbi_statement(0));
        partner.separate = Some(items);
        partner
    };
    let partner = with_loss(1_000);
    assert_eq!(partner.pool(&partner.regular).unwrap().activity.risk_end, None);
    supported(&separate_items_draft(partner));
    let partner = with_loss(1_001);
    assert_eq!(
        partner.pool(&partner.regular).unwrap_err(),
        ("ISSUE_PARTNER_AT_RISK_HISTORY_NEEDED", PartnerRiskRecord::HISTORY_NEEDED)
    );
    unsupported(&separate_items_draft(partner), "Partnership activity");
}

#[test]
fn partner_at_risk_carryover_or_recapture_history_still_needs_form_6198() {
    for edit in [
        (|r: &mut PartnerRiskRecord| r.prior_loss = 100) as fn(&mut PartnerRiskRecord),
        |r| r.prior_nondeductible = 100,
        |r| r.deducted = 100,
    ] {
        let mut partner = unknown_history_partner(20_000);
        edit(&mut partner.regular.risk);
        assert_eq!(
            partner.pool(&partner.regular).unwrap_err(),
            ("ISSUE_PARTNER_AT_RISK_HISTORY_NEEDED", PartnerRiskRecord::HISTORY_NEEDED)
        );
    }
    // Deducted losses already fully recaptured leave nothing to recapture.
    let mut partner = unknown_history_partner(20_000);
    partner.regular.risk.deducted = 100;
    partner.regular.risk.recaptured = 100;
    assert!(partner.pool(&partner.regular).is_ok());
    // A box 1 loss.
    let partner = unknown_history_partner(-1);
    assert_eq!(
        partner.pool(&partner.regular).unwrap_err(),
        ("ISSUE_PARTNER_AT_RISK_HISTORY_NEEDED", PartnerRiskRecord::HISTORY_NEEDED)
    );
}

// T2-29p question C13: the K-1's section 179 carried from 2025 is a Form
// 4562 line 10 item (Instructions for Form 4562, line 10; Partner's
// Instructions for Schedule K-1 (Form 1065), box 12): not a line 8 election,
// subject to this year's line 11 business income limit, deducted on Schedule
// E line 28 column (j) of this activity, and already out of basis.
#[test]
fn partner_section179_carryover_joins_form_4562_line_10_by_hand() {
    // Box 1 $20,000, no box 12, carryover $3,000; basis and at-risk $10,000.
    // Line 8 $0; line 10 $3,000; line 11 $60,000 + $20,000 = $80,000; line
    // 12 $3,000; line 13 $0. Basis: $10,000 + $20,000 = $30,000 (the
    // carryover reduced basis in its own year). Schedule E $20,000 - $3,000
    // = $17,000; AGI $77,000; QBI 20% x $17,000 = $3,400 (limit 20% x
    // $60,900 = $12,180); taxable $77,000 - $16,100 - $3,400 = $57,500;
    // midpoint $57,525: $1,240 + $4,560 + 22% x $7,125 = $7,367.50, rounded
    // $7,368; balance $7,368 - $6,500 = $868.
    let mut partner = section179_partner(20_000, 0);
    partner.section179_carryover = 300_000;
    let pool = partner.pool(&partner.regular).unwrap();
    assert_eq!(crate::kernel::ratio::dollars(pool.activity.basis.ending.get()), 30_000);
    let result = supported(&separate_items_draft(partner));
    assert_eq!(value(&result, "section179_business_income_limit"), 80_000);
    assert_eq!(value(&result, "section179_deduction"), 3_000);
    assert_eq!(value(&result, "partnership_2_section179"), 3_000);
    assert_eq!(value(&result, "partnership_2_section179_carryforward"), 0);
    assert_eq!(return_line(&result, "8"), 1_700_000);
    assert_eq!(return_line(&result, "13b"), 340_000);
    assert_eq!(return_line(&result, "15"), 5_750_000);
    assert_eq!(return_line(&result, "16"), 736_800);
    assert_eq!(result.balance, Some(86_800));
}

#[test]
fn partner_section179_carryover_meets_the_business_income_limit_and_carries_the_rest() {
    // Box 1 $5,000, carryover $70,000, basis and at-risk $100,000. Line 11
    // $60,000 + $5,000 = $65,000 = line 12; $5,000 carries to 2027. Basis
    // $100,000 + $5,000 = $105,000, unchanged by the carryover. Exactly at
    // the limit nothing carries.
    for (carryover, carry) in [(70_000, 5_000), (65_000, 0)] {
        let mut partner = section179_partner(5_000, 0);
        set_partner_start(&mut partner, 100_000, 0);
        partner.section179_carryover = carryover * 100;
        assert_eq!(
            crate::kernel::ratio::dollars(
                partner.pool(&partner.regular).unwrap().activity.basis.ending.get()
            ),
            105_000
        );
        let result = supported(&separate_items_draft(partner));
        assert_eq!(value(&result, "section179_deduction"), 65_000);
        assert_eq!(value(&result, "partnership_2_section179_carryforward"), carry);
    }
    // Without the 2025 at-risk figures the allowed carryover cannot be tested
    // against the amount at risk (question C9).
    let mut partner = unknown_history_partner(20_000);
    partner.section179_carryover = 300_000;
    unsupported(&separate_items_draft(partner), "Partnership section 179");
}

// T2-31r question C9 for shareholders: Form 6198 is filed only when an
// at-risk activity "incurred a loss", and "the at-risk rules of section 465
// limit the amount of the loss you can deduct to the amount at risk"
// (Instructions for Form 6198, Purpose of Form and Who Must File); line 21:
// a line 5 loss equal to or less than the amount at risk is reported in
// full. A continuing shareholder without 2025 at-risk figures therefore
// needs none while the year's losses entering the stage do not exceed the
// activity's income, nothing is carried at risk from 2025, box 11 is zero
// and no earlier deducted loss is left to recapture.
fn unknown_history_shareholder(profit: i64) -> ScorpK1 {
    let mut source = shareholder_k1(profit);
    let risk = &mut source.risk;
    risk.start = ScorpRiskStart::Unknown;
    risk.prior_loss = 0;
    risk.deducted = 0;
    source.amt = Some(amt_records(&source));
    source
}

fn scorp_risk_of(
    source: &ScorpK1,
) -> Result<crate::scorp_risk::ScorpRiskResult, crate::scorp_risk::ScorpRiskError> {
    let basis = source.basis_result().unwrap();
    source.risk.calculate(source, &basis)
}

fn refused_without_risk_history(source: ScorpK1, text: &str) {
    let result = estimate(&shareholder_draft(source));
    assert_eq!(result.state, ResultState::Unsupported, "{result:?}");
    assert!(result.issues.iter().any(|i| i.message.contains(text)), "{:?}", result.issues);
    for issue in result.issues.iter().filter(|issue| issue.section == "S-corporation at-risk") {
        assert_eq!(
            Some({
                let message = &issue.display;
                message.key.as_str()
            }),
            Some("ISSUE_SCORP_RISK_HISTORY_NEEDED")
        );
    }
}

#[test]
fn shareholder_profit_without_at_risk_history_needs_no_form_6198() {
    // Box 1 $20,000, opening stock basis $10,000, the $3,000 note; no loss
    // enters Form 6198, so the amount at risk is not figured. Schedule E
    // line 32 $20,000; AGI $60,000 + $20,000 = $80,000; QBI 20% x $20,000 =
    // $4,000 (limit 20% x $63,900 = $12,780); taxable $80,000 - $16,100 -
    // $4,000 = $59,900; Tax Table midpoint $59,925: $1,240 + $4,560 + 22% x
    // $9,525 = $7,895.50, rounded $7,896; balance $7,896 - $6,500 = $1,396.
    let source = unknown_history_shareholder(20_000);
    let risk = scorp_risk_of(&source).unwrap();
    assert_eq!(
        (
            risk.investment.map(crate::kernel::ratio::dollars),
            risk.year_end.map(crate::kernel::ratio::dollars)
        ),
        (None, None)
    );
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.income.get()),
            crate::kernel::ratio::dollars(risk.ledger.risk_allowed[K1LossClass::Ordinary].get()),
            crate::kernel::ratio::dollars(risk.ledger.risk_suspended[K1LossClass::Ordinary].get())
        ),
        (20_000, 0, 0)
    );
    let draft = shareholder_draft(source);
    let result = supported(&draft);
    assert_eq!(return_line(&result, "8"), 2_000_000);
    assert_eq!(return_line(&result, "11a / 11b"), 8_000_000);
    assert_eq!(return_line(&result, "13b"), 400_000);
    assert_eq!(return_line(&result, "15"), 5_990_000);
    assert_eq!(return_line(&result, "16"), 789_600);
    assert_eq!(result.balance, Some(139_600));
    assert!(
        result.review_notes.iter().any(|n| n.contains("no Form 6198 is needed")),
        "{:?}",
        result.review_notes
    );
    assert_eq!(estimate(&restart(draft).draft), result);
}

#[test]
fn shareholder_loss_equal_to_activity_income_needs_no_history_and_one_dollar_more_does() {
    // Box 1 zero, box 4 $1,000 interest (Form 6198 line 3 income of the
    // activity) and a box 7 short-term loss. At $1,000 the loss equals the
    // income: line 5 is zero and the loss is allowed in full. AGI $60,000 +
    // $1,000 - $1,000 = $60,000; taxable $60,000 - $16,100 = $43,900; Tax
    // Table midpoint $43,925: $1,240 + 12% x $31,525 = $5,023. At $1,001
    // the $1 net loss needs the amount at risk, so the history is refused.
    let with_loss = |loss: i64| {
        let mut source = unknown_history_shareholder(0);
        source.interest = 100_000;
        source.capital = Some(k1_capital(-loss, 0));
        source.amt = Some(amt_records(&source));
        source
    };
    let source = with_loss(1_000);
    let risk = scorp_risk_of(&source).unwrap();
    assert_eq!(risk.year_end.map(crate::kernel::ratio::dollars), None);
    assert_eq!(
        (
            crate::kernel::ratio::dollars(risk.income.get()),
            risk.ledger.risk_allowed.capital().map(crate::kernel::ratio::dollars),
            risk.ledger.risk_suspended.capital().map(crate::kernel::ratio::dollars)
        ),
        (1_000, [1_000, 0, 0], [0; 3])
    );
    let result = supported(&shareholder_draft(source));
    assert_eq!(return_line(&result, "11a / 11b"), 6_000_000);
    assert_eq!(return_line(&result, "15"), 4_390_000);
    assert_eq!(return_line(&result, "16"), 502_300);
    let source = with_loss(1_001);
    assert_eq!(
        scorp_risk_of(&source).unwrap_err(),
        crate::scorp_risk::ScorpRiskError::HistoryNeeded
    );
    refused_without_risk_history(source, ScorpRisk::HISTORY_NEEDED);
}

#[test]
fn shareholder_at_risk_carryover_or_recapture_history_still_needs_form_6198() {
    for edit in [
        (|s: &mut ScorpK1| s.risk.prior_loss = 100) as fn(&mut ScorpK1),
        |s| s.risk.prior_nondeductible = 100,
        |s| s.risk.deducted = 100,
        |s| s.capital = Some(ScorpCapital { risk_short: 100, ..k1_capital(0, 0) }),
        // Box 11 waits for its own at-risk test.
        |s| s.section179 = 100_000,
        // A box 1 loss with no activity income.
        |s| s.ordinary = -100,
    ] {
        let mut source = unknown_history_shareholder(20_000);
        edit(&mut source);
        reconcile_qbi_tracking(&mut source);
        source.amt = Some(amt_records(&source));
        if source.ordinary < 0 {
            source.qbi_statement.as_mut().unwrap().income = 0;
        }
        assert_eq!(
            scorp_risk_of(&source).unwrap_err(),
            crate::scorp_risk::ScorpRiskError::HistoryNeeded
        );
        refused_without_risk_history(source, ScorpRisk::HISTORY_NEEDED);
    }
    // Deducted losses already fully recaptured leave nothing to recapture.
    let mut source = unknown_history_shareholder(20_000);
    source.risk.deducted = 100;
    source.risk.recaptured = 100;
    assert!(scorp_risk_of(&source).is_ok());
    // A section 179 carryover (Form 4562 line 10) is tested against the
    // amount at risk, which needs the history.
    let mut source = unknown_history_shareholder(20_000);
    source.section179_carryover = 300_000;
    refused_without_risk_history(source, "2025 Form 6198 figures");
}

// T2-cy item 5. Form 1040 instructions, line 12e: "Use this worksheet only
// if someone can claim you, or your spouse if filing jointly, as a
// dependent", and its earned income includes "other compensation received
// for personal services you performed". A joint return whose spouse alone
// can be claimed therefore reads the spouse's partnership service income.
#[test]
fn joint_return_with_a_claimable_spouse_reads_partner_service_income() {
    // By hand: gross personal-service income $20,000 from the spouse's
    // partnership (Services source). Worksheet line 2: 20,000 + 450 =
    // 20,450; line 3 (married filing jointly, 2026) 32,200; line 4a the
    // smaller, 20,450.
    let mut d = empty_return(FederalStatus::Joint);
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().dependency =
        FilerDependency::CanBeClaimed { claimed: true };
    let mut data = ordinary_partner_source(10000, true);
    data.income_source = Some(BusinessIncomeSource::Services);
    data.gross_personal_services = Some(2000000);
    data.qbi = Some(partner_qbi_statement(10000));
    add(
        &mut d,
        FederalIncomeEntry {
            owner: crate::DocumentOwner::Spouses,
            ..income(FederalIncome::Partnership { data })
        },
    );
    let r = estimate(&d);
    assert!(
        r.review_notes.iter().any(|n| n.contains(
            "Dependent standard-deduction earned income from partnership sources: $20000;"
        ) && n.contains("standard deduction: $20450.")),
        "{:?}",
        r.review_notes
    );
    // Boundary: neither spouse claimable, so the worksheet does not apply
    // and the partnership source is not read for it.
    d.people.as_mut().unwrap().spouse.as_mut().unwrap().dependency = FilerDependency::Independent;
    assert!(!estimate(&d).review_notes.iter().any(|n| {
        n.contains("Dependent standard-deduction earned income from partnership sources")
    }));
}

// Unit u7-k1: AI Import on the three Schedule K-1 kinds.
mod k1_ai_import {
    use alloc::{
        format,
        string::{String, ToString},
        vec::Vec,
    };

    // The fields an item's AI Import selects, read from its source: each
    // `libertas_llm_doc_field` marker names the next field, prefixed by its
    // enum variant, with the marker's arguments.
    fn ai_import_fields(source: &str, item: &str) -> Vec<String> {
        let start = source.find(item).expect("item");
        let body = &source[start..];
        let body = &body[..body.find("\n}").expect("item end")];
        let mut fields = Vec::new();
        let mut variant = String::new();
        let mut marker: Option<String> = None;
        for line in body.lines().skip(1) {
            let text = line.trim();
            if text.starts_with("//") {
                continue;
            }
            if line.starts_with("    ")
                && !line.starts_with("     ")
                && text.ends_with('{')
                && !text.starts_with("pub ")
            {
                variant = text.trim_end_matches('{').trim().to_string() + ".";
            }
            if let Some(rest) = text.strip_prefix("#[libertas_llm_doc_field") {
                marker = Some(rest.trim_end_matches(']').to_string());
            } else if let Some(args) = marker.as_ref()
                && !text.starts_with("#[")
                && let Some((name, _)) = text.trim_start_matches("pub ").split_once(':')
            {
                let prefix = if line.starts_with("        ") { variant.as_str() } else { "" };
                fields.push(format!("{prefix}{name}{args}"));
                marker = None;
            }
        }
        fields
    }

    #[test]
    fn k1_ai_import_selects_the_printed_boxes_only() {
        let page = include_str!("../../passthrough/k1_page.rs");
        let partnership_source = include_str!("../../passthrough/partnership_source.rs");
        let scorp_source = include_str!("../../passthrough/scorp_source.rs");
        let estate_source = include_str!("../../passthrough/estate_trust_paper.rs");
        let model = crate::federal_model::HUB_SOURCE;
        let credits = include_str!("../../credits/business_credit_page.rs");
        let strict = "(allow_empty = false)";
        let check = |source: &str, item: &str, expected: &[&str]| {
            assert_eq!(ai_import_fields(source, item), expected, "{item}");
        };
        // 2026-10-27: manual entry and AI collect the same printed fields and
        // typed answers established by the fiduciary's actual attachments.
        // Personal participation, entitlement and history remain separate.
        // https://www.irs.gov/instructions/i1041sk1
        {
            let item = "pub struct K1EstateTrustDocument {";
            let head = &estate_source[..estate_source.find(item).unwrap()];
            let action = &head[head.rfind("#[libertas_llm_doc_trans(").expect(item)..];
            assert!(action.contains("Schedule K-1 (Form 1041)"), "{item}");
            assert!(action.contains("Both sections are the same fields used for manual entry"));
            assert!(action.contains("Never substitute entity totals"));
            assert!(action.contains("beneficiary participation, passive history, entitlement"));
        }
        check(estate_source, "pub struct K1EstateTrustDocument {", &["form", "statements"]);
        {
            let (item, form) = ("pub struct K1EstateTrust {", "Schedule K-1 (Form 1041)");
            let head = &page[..page.find(item).unwrap()];
            let action = &head[head.rfind("#[libertas_llm_doc_trans(").expect(item)..];
            assert!(action.contains(form), "{item}");
            assert!(action.contains("validation = \"$.document_facts\""));
            assert!(action.contains("the existing list"));
        }
        let head = &partnership_source
            [..partnership_source.find("pub struct K1PartnershipDocument {").unwrap()];
        let action = &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..];
        assert!(action.contains("Schedule K-1 (Form 1065)"));
        assert!(action.contains("validation = \"$.document_facts\""));
        assert!(action.contains("never import its Form 3800 line"));
        check(partnership_source, "pub struct K1PartnershipDocument {", &["form", "statements"]);
        check(
            include_str!("../../core/shared/printed_state.rs"),
            "pub struct PrintedParty {",
            &[
                "name(allow_empty = true)",
                "federal_id(allow_empty = true)",
                "address(allow_empty = true)",
            ],
        );
        check(
            page,
            "pub struct K1EstateTrust {",
            &[
                "estate_trust",
                "fiduciary",
                "beneficiary",
                "entity_period",
                "interest",
                "ordinary_dividends",
                "qualified_dividends",
                "short_gain",
                "long_gain",
                "rate_28_gain",
                "unrecaptured_gain",
                "gross_farming_fishing_income(blank = 0)",
                "more",
            ],
        );
        check(
            page,
            "pub struct EstateCurrentPeriodPage {",
            &["began(allow_empty = false)", "ended(allow_empty = false)", "accounting"],
        );
        check(
            include_str!("../../passthrough/k1_period.rs"),
            "pub struct K1EntityTaxPeriod {",
            &["began(allow_empty = false)", "ended(allow_empty = false)"],
        );
        // The fiduciary's actual period dates and accounting election are
        // document facts, including original loss periods in its successor statement.
        check(model, "pub struct FinalEstateTrustTaxPeriod {", &["period", "accounting"]);
        check(
            model,
            "pub struct FinalEstateTrustLossPeriod {",
            &[&format!("began{strict}"), &format!("ended{strict}")],
        );
        check(
            page,
            "pub struct K1EstateTrustOtherBoxes {",
            &[
                "ird_estate_tax",
                "amt",
                "minimum_tax_exclusion",
                "estimated_tax",
                "backup_withholding",
                "tax_exempt_interest",
                "private_activity_interest",
                "niit",
                "section199a_dividends(blank = 0)",
                "unentered",
            ],
        );
        check(
            page,
            "pub enum K1EstateTrustAmt {",
            &[
                &format!("CodeAOnly.a{strict}"),
                &format!("Components.a{strict}"),
                "Components.b",
                "Components.c",
                "Components.d",
                "Components.e",
                "Components.f",
            ],
        );
        check(
            page,
            "pub enum K1EstateTrustEstimatedTax {",
            &[&format!("Allocated.amount{strict}"), "Allocated.filed_1041t"],
        );
        check(
            model,
            "pub struct EstateTrustNiitAdjustment {",
            &[&format!("line_7{strict}"), &format!("modified_agi{strict}")],
        );
        // 2026-10-26: paper and actual source statements are the import targets.
        // https://www.irs.gov/pub/irs-dft/f1120ssk--dft.pdf
        let head = &scorp_source[..scorp_source.find("pub struct K1ScorpDocument {").unwrap()];
        let action = &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..];
        assert!(action.contains("Schedule K-1 (Form 1120-S)"));
        // A fiscal-year paper can print 2025 and end in 2026, so the paper is
        // checked by its actual period and not by the printed calendar year.
        assert!(!action.contains("validation = \"$.document_facts\""));
        assert!(action.contains("Its actual ending must be in 2026"));
        assert!(action.contains("Do not import personal, completed owner forms"));
        assert!(action.contains("Code M Research never supplies its Form 3800 line"));
        check(scorp_source, "pub struct K1ScorpDocument {", &["form", "statements"]);
        check(
            scorp_source,
            "pub struct K1ScorpPaper {",
            &[
                "reporting_period",
                "final_k1",
                "amended_k1",
                "corporation",
                "irs_center(allow_empty = true)",
                "corporation_shares",
                "shareholder",
                "reporting_party",
                "shareholder_type(allow_empty = true)",
                "allocation_percentage",
                "shareholder_shares",
                "loans",
                "ordinary",
                "rental_real_estate",
                "other_rental",
                "interest",
                "dividends",
                "qualified_dividends",
                "royalties",
                "short",
                "long",
                "collectibles",
                "unrecaptured",
                "section1231",
                "income",
                "section179",
                "deductions",
                "credits",
                "k3_attached",
                "amt",
                "basis",
                "information",
                "multiple_at_risk",
                "multiple_passive",
            ],
        );
        check(scorp_source, "pub struct ScorpPersonalRecords {", &[]);
        check(
            include_str!("../../passthrough/k1_items.rs"),
            "pub struct K1Section179Sale {",
            &[
                &format!("description{strict}"),
                &format!("acquired{strict}"),
                &format!("sold{strict}"),
                "regular",
            ],
        );
        check(
            include_str!("../../passthrough/k1_items.rs"),
            "pub struct K1Section179SaleAmounts {",
            &[
                &format!("price{strict}"),
                &format!("cost{strict}"),
                &format!("depreciation{strict}"),
                &format!("section179{strict}"),
            ],
        );
        // Boxes 2 and 3 and the statement's rental amounts are imported;
        // the participation answer is the filer's own and is kept.
        check(
            page,
            "pub struct K1RentalBoxes {",
            &[
                "real_estate",
                "real_estate_section1231",
                "real_estate_unrecaptured",
                "other",
                "other_section1231",
                "other_unrecaptured",
                "real_estate_qbi",
                "other_qbi",
            ],
        );
        check(
            include_str!("../../passthrough/k1_rental.rs"),
            "pub struct K1RentalQbi {",
            &["wages", "property"],
        );
        check(
            page,
            "pub enum K1ScorpQbi {",
            &[
                &format!("Statement.income{strict}"),
                &format!("Statement.wages{strict}"),
                &format!("Statement.property{strict}"),
                "Statement.service",
                "Statement.current_loss",
                "Statement.property_qbi",
                "Statement.reit_dividends",
                "Statement.ptp_income",
            ],
        );
        // The statement's own QBI loss and section 1231 amounts are import
        // targets. The shareholder's loss histories and confirmation stay
        // in the always-present non-imported qbi_records sibling even when
        // the selected statement variant changes.
        let qbi = include_str!("../../passthrough/scorp_qbi.rs");
        check(qbi, "pub struct ScorpLossQbi {", &[]);
        check(qbi, "pub struct ScorpPropertyQbi {", &[]);
        check(page, "pub struct K1ScorpLossHistory {", &[]);
        check(page, "pub struct K1ScorpQbiRecords {", &[]);
        check(
            include_str!("../../passthrough/partnership_qbi.rs"),
            "pub struct PartnerQbiRecords {",
            &[],
        );
        check(page, "pub struct K1ScorpPropertyQbi {", &["gain", "qualified_1250", "current"]);
        let field_type = |source: &str, item: &str, name: &str| -> String {
            let body = &source[source.find(item).expect(item)..];
            let body = &body[..body.find("\n}").expect("item end")];
            let line = body
                .lines()
                .map(|l| l.trim().trim_start_matches("pub "))
                .find(|l| l.starts_with(&format!("{name}:")))
                .expect(name);
            line[name.len() + 1..].trim().to_string()
        };
        for (name, ty) in [("current_loss", "i64,"), ("property_qbi", "K1ScorpPropertyQbi,")] {
            assert_eq!(field_type(page, "pub enum K1ScorpQbi {", name), ty, "{name}");
        }
        check(page, "pub(crate) struct K1ScorpOtherBoxes {", &[]);
        check(page, "pub(crate) struct K1Scorp {", &[]);
        // 2026-10-26: import the printed credit rows and attached statements,
        // not the derived editor or personal eligibility choices.
        // https://www.irs.gov/pub/irs-dft/f1120ssk--dft.pdf
        check(credits, "pub struct K1CreditPage {", &[]);
        check(credits, "pub enum K1CreditCodePage {", &[]);
        // The investment credit items (Form 3468) are read from the statement;
        // the personal confirmation is never an import target.
        let investment = include_str!("../../credits/investment_credit.rs");
        check(
            investment,
            "pub enum K1InvestmentProperty {",
            &[
                &format!("RehabilitationRental.expenditures{strict}"),
                &format!("RehabilitationRental.placed_year{strict}"),
                &format!("RehabilitationOther.expenditures{strict}"),
                &format!("RehabilitationOther.placed_year{strict}"),
                &format!("Energy.basis{strict}"),
                "Energy.rate",
                &format!("CleanElectricity.basis{strict}"),
                "CleanElectricity.storage",
                "CleanElectricity.rate",
            ],
        );
        check(
            investment,
            "pub struct EnergyRate {",
            &["increased_rate", "domestic_content", "energy_community"],
        );
        assert!(!page.contains(
            "#[libertas_llm_doc_field]\n    #[libertas_default($null)]\n    pub investment_confirmed"
        ));
        check(
            scorp_source,
            "pub struct ScorpStatements {",
            &[
                "qbi",
                "rentals",
                "property",
                "income",
                "credits",
                "investment_property",
                "section179_sales",
                "private_activity",
                "situations",
            ],
        );
        check(scorp_source, "pub struct ScorpSource {", &[]);
        check(
            partnership_source,
            "pub struct K1PartnershipPaper {",
            &[
                "reporting_period",
                "final_k1",
                "amended_k1",
                "partnership",
                "irs_center(allow_empty = true)",
                "publicly_traded",
                "partner",
                "general_partner",
                "limited_partner",
                "domestic_partner",
                "foreign_partner",
                "disregarded_entity",
                "entity_type(allow_empty = true)",
                "retirement_plan",
                "share",
                "share_sale",
                "share_exchange",
                "liabilities",
                "lower_tier_liabilities",
                "guaranteed_liabilities",
                "capital_account",
                "contributed_built_in_property",
                "unrecognized_704c_beginning",
                "unrecognized_704c_ending",
                "ordinary",
                "rental_real_estate",
                "other_rental",
                "services",
                "capital",
                "total_guaranteed",
                "interest",
                "dividends",
                "qualified_dividends",
                "dividend_equivalents",
                "royalties",
                "short",
                "long",
                "collectibles",
                "unrecaptured",
                "section1231",
                "income",
                "section179",
                "deductions",
                "self_employment",
                "credits",
                "k3_attached",
                "amt",
                "exempt",
                "distributions",
                "information",
                "foreign_taxes",
                "multiple_at_risk",
                "multiple_passive",
            ],
        );
        check(
            partnership_source,
            "pub struct PartnershipStatements {",
            &[
                "qbi",
                "rentals",
                "property",
                "income",
                "credits",
                "investment_property",
                "section179_sales",
                "situations",
            ],
        );
        check(
            partnership_source,
            "pub struct K1CodeRow {",
            &["code(allow_empty = true)", "amount", "text(allow_empty = true)", "statement"],
        );
        check(
            partnership_source,
            "pub struct K1Liabilities {",
            &["nonrecourse", "qualified_nonrecourse", "recourse"],
        );
        check(partnership_source, "pub struct K1MoneyPair {", &["beginning", "ending"]);
        check(partnership_source, "pub struct PartnershipIncomeStatement {", &["code", "amount"]);
        check(
            partnership_source,
            "pub struct PartnershipCreditStatement {",
            &["code", "line", "amount"],
        );
        check(partnership_source, "pub struct PartnerPersonalRecords {", &[]);
        check(page, "pub(crate) struct K1Partnership {", &[]);
        check(page, "pub(crate) struct K1PartnershipOtherBoxes {", &[]);
        check(page, "pub(crate) enum SeGross {", &[]);
        let items = include_str!("../../passthrough/k1_items.rs");
        check(items, "pub struct PartnerIncome {", &[]);
        check(items, "pub struct PartnerDeduction {", &[]);
        check(
            page,
            "pub enum PartnerQbi {",
            &[
                &format!("Statement.income{strict}"),
                "Statement.wages",
                "Statement.property",
                "Statement.service",
                "Statement.section1231_qbi",
                "Statement.reit_dividends",
                "Statement.ptp_income",
            ],
        );
        // 2026-10-26: supported K-3 source fields are part of the actual attachments.
        assert!(scorp_source.contains("pub schedule_k3: Option<crate::ScheduleK3Paper>"));
        // The filer's own facts stay out of every K-1 import.
        for item in [
            "pub struct PartnerBasis {",
            "pub enum BasisStart {",
            "pub struct PartnerCarried {",
            "pub struct PartnerAtRisk {",
            "pub enum K1ScorpStart {",
            "pub struct K1ScorpCarried {",
            "pub struct K1ScorpAtRisk {",
            "pub struct K1ScorpBasis {",
            "pub enum K1EstateTrustRoyalties {",
            "pub enum K1FinalYear {",
            "pub enum Participation {",
            "pub enum K1IncomeSource {",
        ] {
            assert!(ai_import_fields(page, item).is_empty(), "{item}");
        }
    }

    // Schedule K-3 has its own AI Import. It reads the printed
    // Part II line 24, Section 2 and Part III lines and nothing else: each
    // passive row's withholding treatment is the filer's own answer and
    // stays out of the import (the category is imported by variant name only).
    #[test]
    fn k3_ai_import_selects_the_printed_lines_only() {
        let k3 = include_str!("../../passthrough/schedule_k3.rs");
        let check = |item: &str, expected: &[&str]| {
            assert_eq!(ai_import_fields(k3, item), expected, "{item}");
        };
        let head = &k3[..k3.find("pub struct ScheduleK3 {").unwrap()];
        let action = &head[head.rfind("#[libertas_llm_doc_trans(").unwrap()..];
        assert!(action.contains("Schedule K-3 (Form 1065 or Form 1120-S)"));
        assert!(action.contains("validation = \"$.document_facts\""));
        assert!(action.contains("replace the existing lists"));
        assert!(action.contains("withholding treatment comes from the person's own records"));
        check(
            "pub struct ScheduleK3 {",
            &[
                "countries",
                "us_gross",
                "branch",
                "passive",
                "general",
                "partner_deductions",
                "partner_interest",
                "items",
            ],
        );
        check("pub struct K3Country {", &["country", "category", "income", "qualified", "tax"]);
        check("pub enum K3Category {", &[]);
        // The person's withholding answers stay outside every imported list.
        check("pub struct K3Withholding {", &[]);
        check(
            "pub struct K3CategoryAmounts {",
            &["deductions", "interest", "reduction", "capital"],
        );
        check("pub struct K3Capital {", &["short_gain", "long_gain", "short_loss", "long_loss"]);
        check(
            "pub struct K3PartnerInterest {",
            &["business", "passive_activity", "business_assets", "passive_assets"],
        );
        check("pub struct K3Assets {", &["us", "branch", "passive", "general"]);
        let source = include_str!("../../passthrough/partnership_source.rs");
        assert_eq!(
            ai_import_fields(source, "pub struct PartnershipK3Source {"),
            ai_import_fields(k3, "pub struct ScheduleK3 {")
        );
        assert!(
            !ai_import_fields(source, "pub struct PartnershipStatements {")
                .contains(&"schedule_k3".into())
        );
        let scorp_source = include_str!("../../passthrough/scorp_source.rs");
        assert!(
            !ai_import_fields(scorp_source, "pub struct ScorpStatements {")
                .contains(&"schedule_k3".into())
        );
        // The complete printed K-3 stays attached, with independent actions
        // for its header and the two printed Part II sections.
        for item in [
            "K3PartnershipPaper",
            "K3ScorpPaper",
            "K3PaperIncome1065",
            "K3PaperDeductions1065",
            "K3PaperIncome1120S",
            "K3PaperDeductions1120S",
        ] {
            let declaration = format!("pub struct {item} {{");
            let head = &k3[..k3.find(&declaration).expect(item)];
            let action = &head[head.rfind("#[libertas_llm_doc_trans(").expect(item)..];
            assert!(!action.contains("\n}\n"), "{item} needs its own import action");
            assert!(action.contains("validation = \"$.document_facts\""), "{item}");
        }
        assert!(
            !ai_import_fields(scorp_source, "pub struct ScorpPersonalRecords {")
                .contains(&"k3_withholding".into())
        );
    }
}

#[test]
fn estate_shared_qbi_rounds_one_activity_once_and_is_statement_order_independent() {
    // Current tax loss3, allowed1; two qualified portions1+1 give qualified
    // loss round(1*2/3)=1, rather than round(1/3)+round(1/3)=0.
    // Material QBI8 less qualified loss1 =>7;20%=>1 whole dollar.
    let mut draft = estate_trust_activity_draft(6_000_000);
    let mut data = estate_trust_code_d_data(&mut draft);
    estate_fixture_incomes(&mut data, [-300, 100, 800], [0; 3], false);
    data.activities.activities[2].kind = crate::EstateActivityKind::Business;
    data.activities.activities[2].participation =
        crate::EstateParticipation::Material(ScorpParticipation {
            minutes: 30_001,
            ..participation_facts()
        });
    data.activities.box6 = 500;
    data.activities.box8 = 0;
    data.qbi.statements = ["First", "Second"]
        .into_iter()
        .map(|name| crate::EstateQbiStatement {
            id: 0,
            name: name.into(),
            income: 300,
            wages: 0,
            property: 0,
            service: false,
            kind: crate::EstateQbiKind::Business,
            activities: vec![3, 5],
            passive_history: vec![],
            allocations: vec![
                crate::EstateQbiAllocation { activity: 3, income: -100, passive_history: vec![] },
                crate::EstateQbiAllocation { activity: 5, income: 400, passive_history: vec![] },
            ],
        })
        .collect();
    drop(data);
    let r = supported(&draft);
    assert_eq!(value(&r, "qbi_deduction"), 1);
    assert!(!noted(&r, "WARN_ESTATE_QBI_SOURCE_LEFT_OUT"));
    estate_trust_code_d_data(&mut draft).qbi.statements.reverse();
    assert_eq!(value(&supported(&draft), "qbi_deduction"), 1);
    assert_eq!(value(&supported(&restart(draft.clone()).draft), "qbi_deduction"), 1);
}

#[test]
fn unit47_estate_metadata_and_omitted_final_sources_keep_actual_native_bounds() {
    // Additional deduction records retain their named omitted-benefit treatment.
    let mut draft = estate_trust_activity_draft(6_000_000);
    let baseline = supported(&draft);
    let mut data = estate_trust_code_d_data(&mut draft);
    data.activities.activities[0].id = i64::MAX;
    assert!(data.activities.activities[0].issue().is_none());
    data.activities.activities[0].id = 3;
    let donor = k1_niit_portfolio_entry(K1PortfolioNiitSource::Estate, false, true);
    let FederalIncome::EstateTrustPortfolio { data: donor } = donor.income else { unreachable!() };
    let mut finals = donor.statements.final_character.unwrap();
    finals.section67e_nii = 0;
    data.final_k1 = true;
    data.final_year_successor = Some(true);
    // Dated correction 2026-10-20: box 11 code B investment interest is now
    // figured on Form 4952 (i1041sk1 box 11 code B), so its amount has the
    // money range; an unclassified other deduction keeps the named omission.
    finals.additional =
        vec![
            crate::EstateFinalDeduction::Other { description: "Synthetic source deduction".into() };
            101
        ];
    data.final_character = Some(finals);
    drop(data);
    assert!(crate::federal_rules::validate_income_amount(&draft.income[1].income).is_ok());
    let report = supported(&draft);
    assert!(noted(&report, "WARN_ESTATE_FINAL_OTHER_LEFT_OUT"));
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(return_line(&report, "11a / 11b"), return_line(&baseline, "11a / 11b"));
    assert_eq!(restart(draft.clone()).draft, draft);
}

#[test]
fn partial_partner_fuel_recovery_uses_one_rounded_income_for_regular_amt_and_qbi() {
    let mut baseline = complete(FederalStatus::Single);
    let mut partner = ordinary_partner_source(20_000, true);
    partner.qbi = Some(partner_qbi_statement(20_000));
    add(&mut baseline, income(FederalIncome::Partnership { data: partner.clone() }));
    let baseline_report = supported(&baseline);
    // The retained comparison's $16,647 - $15,929 = $718 income,
    // rather than the $5,000 credit or the $73 comparison tax.
    for (cents, whole) in [(71_800, 718), (71_849, 718), (71_850, 719)] {
        let mut prior = partial_partner_fuel_recovery();
        let partial = prior.partial_prior_year_benefit.as_mut().unwrap();
        partial.refigured_prior_year_line_11b = partial.refigured_prior_year_line_14 + cents;
        assert!(prior.valid());
        assert_eq!(prior.taxable_amount(), cents);
        partner.prior_fuel_credit = Some(prior);
        let mut draft = baseline.clone();
        draft.income[1].income = FederalIncome::Partnership { data: partner.clone() };
        let report = supported(&draft);
        assert_eq!(line_amount(&report, "partnership_2_prior_fuel_credit_recovery"), cents);
        assert_eq!(return_line(&report, "8"), return_line(&baseline_report, "8") + whole * 100);
        assert_eq!(return_line(&report, "13b"), return_line(&baseline_report, "13b") + 14_400);
        assert!(!noted(&report, "WARN_PARTNER_FUEL_PARTIAL_BENEFIT_LEFT_OUT"));
        for amt in [false, true] {
            let mut result = report.clone();
            let actual = crate::extended_rules::rentals_for(&draft, 80_000, &mut result, amt);
            let expected = crate::extended_rules::rentals_for(&baseline, 80_000, &mut result, amt);
            assert_eq!(actual.income, expected.income + whole);
        }
        assert_eq!(
            crate::interview::document_amount(&draft.income[1].income, false),
            Some((20_000 + whole) * 100)
        );
        assert_eq!(supported(&restart(draft).draft), report);
    }
}

#[test]
fn partial_partner_fuel_recovery_keeps_conservative_fallback_and_validates_inactive_data() {
    let good = partial_partner_fuel_recovery();
    // Each existing qualification independently declines this narrow comparison.
    for case in 0..7 {
        let mut prior = good.clone();
        match case {
            0 => prior.partial_prior_year_benefit = None,
            1 => prior.partial_prior_year_benefit.as_mut().unwrap().comparison_complete = false,
            2 => {
                prior.partial_prior_year_benefit.as_mut().unwrap().active_unlimited_deduction =
                    false
            }
            3 => prior.partial_prior_year_benefit.as_mut().unwrap().no_prior_amt = false,
            4 => {
                prior.partial_prior_year_benefit.as_mut().unwrap().no_prior_credits_or_carryovers =
                    false
            }
            5 => {
                prior
                    .partial_prior_year_benefit
                    .as_mut()
                    .unwrap()
                    .original_standard_only_deduction = false
            }
            6 => prior.full_prior_tax_benefit = true,
            _ => unreachable!(),
        }
        assert!(prior.valid());
        let mut draft = complete(FederalStatus::Single);
        let mut partner = ordinary_partner_source(20_000, true);
        partner.prior_fuel_credit = Some(prior);
        add(&mut draft, income(FederalIncome::Partnership { data: partner }));
        let report = supported(&draft);
        assert_eq!(value(&report, "partnership_2_prior_fuel_credit_recovery"), 5_000);
        assert_eq!(noted(&report, "WARN_PARTNER_FUEL_PARTIAL_BENEFIT_LEFT_OUT"), case != 6);
    }
}

// Smith round 9: john-audit-8986.pdf Part V line 5, (d) 50, (g) 20, (h) 70.
// Line 17 interest is asked only when the 2025 Tax Table row moves.
#[test]
fn bba_audit_smith_paper_asks_line_17_only_for_an_increase() {
    let (paper, _) = FederalBbaAuditInterest {
        source_id: "SYNTH8986JOHN2026".into(),
        furnished_on: 20261015,
        interest_adjustment: 20_00,
        original_partnership_interest: 50_00,
        prior_status: FederalStatus::Joint,
        ..bba_interest_reduction()
    }
    .into_parts();
    let answers =
        |wages: i64, taxable: i64, tax: i64, interest: Option<i64>, prepared: Option<bool>| {
            crate::BbaAuditAnswers {
                prior_wages: Some(wages),
                prior_taxable_income: Some(taxable),
                prior_income_tax: Some(tax),
                prior_status: Some(FederalStatus::Joint),
                single_audit_adjustment: Some(true),
                simple_prior_return: Some(true),
                no_other_effects: Some(true),
                line_17_interest: interest,
                interest_calculation_prepared: prepared,
            }
        };
    let issue = |a: crate::BbaAuditAnswers| {
        a.model(1, &paper).and_then(|m| m.refigure()).map(|r| r.line_14()).map_err(|e| e.0)
    };
    // The walk's answers: 70,020 stays in the 70,000-70,050 row, so line 17 is not
    // asked; the wages do not reconcile to 70,000.
    assert_eq!(
        issue(answers(10_000_000, 7_000_000, 792_600, None, Some(true))),
        Err("ISSUE_BBA_AUDIT_PRIOR_INCOME_RECONCILIATION")
    );
    assert_eq!(issue(answers(10_145_000, 7_000_000, 792_600, None, Some(true))), Ok(0));
    assert_eq!(issue(answers(10_000_000, 6_855_000, 775_200, None, None)), Ok(0));
    // 88,580 moves the row: line 17 is required, and accepted when documented.
    let row = |i, p| answers(12_003_000, 8_858_000, 1_015_200, i, p);
    for missing in [row(None, Some(true)), row(None, None), row(Some(1_00), Some(false))] {
        assert_eq!(issue(missing), Err("ISSUE_BBA_AUDIT_INTEREST_DOCUMENTATION"));
    }
    assert_eq!(issue(row(Some(1_00), Some(true))), Ok(6));
}
