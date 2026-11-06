use super::*;

#[test]
fn approved_d4_reserves_line_6l_once_and_only_absorbs_chapter_one_tax() {
    // Approved D4: $1,000 decrease; line18 $600; Chapter1 PartII $300;
    // payroll $500. AOTC recapture: $2,500 - ($7,000-$5,100) = $600.
    let mut draft = uncollected_wage(10_000, 500, 0, true);
    draft.deductions.as_mut().unwrap().itemize_override = None;
    draft.credits.as_mut().unwrap().education_recapture = Some(PriorYearEducationCreditRecapture {
        method: EducationMethod::AmericanOpportunity,
        filed_qualified_expenses: 700_000,
        after_filing_reduction: 510_000,
        filed_education_magi: 5_000_000,
        filed_joint: false,
        filed_credit_used: 250_000,
        ordinary_prior_return: true,
    });
    add(&mut draft, income(archer_msa(1500))); // §220: 20% x $1,500.
    draft.credits.as_mut().unwrap().minimum_tax =
        Some(crate::MinimumTaxHistories::from_current_unit(minimum_tax_history()));
    draft.credits.as_mut().unwrap().bba_audit_interest =
        Some(crate::tests::support::builders::added_bba(bba_thousand_dollar_reduction()));
    let result = supported(&draft);
    assert_eq!(value(&result, "form_8978_line_14"), -1000);
    assert_eq!(return_line(&result, "18"), 60_000);
    assert_eq!(value(&result, "schedule_3_line_6l"), 600);
    assert_eq!(value(&result, "schedule_2_line_13z_form_8978"), -300);
    assert_eq!(value(&result, "form_8978_unusable_decrease"), 100);
    assert_eq!(value(&result, "minimum_tax_credit"), 0);
    assert_eq!(return_line(&result, "23"), 50_000);
    assert_eq!(result.tax, Some(50_000));
    assert_eq!(supported(&restart(draft).draft), result);
}

#[test]
fn native_msa_able_and_mortgage_outputs_absorb_the_same_remaining_decrease() {
    // Independent owners: Archer 20% x300=$60; ABLE 10% x500=$50;
    // mortgage gift gain $200 caps recapture at half the gain=$100.
    let mut draft = bba_low_tax_return();
    add(&mut draft, income(archer_msa(300)));
    add(&mut draft, income(able_distribution(1000, 500, 0)));
    draft.credits.as_mut().unwrap().mortgage_subsidies =
        vec![crate::tests::federal::other_taxes::gift(20200601, 200_200, 200_000, 0, 60)];
    let result = supported(&draft);
    assert_eq!(value(&result, "archer_msa_additional_tax"), 60);
    assert_eq!(value(&result, "account_additional_tax"), 50);
    assert_eq!(value(&result, "mortgage_subsidy_recapture_tax"), 100);
    assert_eq!(value(&result, "schedule_3_line_6l"), 0);
    assert_eq!(value(&result, "schedule_2_line_13z_form_8978"), -210);
    assert_eq!(value(&result, "form_8978_unusable_decrease"), 790);
    assert_eq!(return_line(&result, "23"), 0);
    let mut session = restart(draft);
    let P::SaveOtherTaxes { value: shown, .. } =
        crate::tests::cross::interview::open_other_taxes(&mut session)
    else {
        unreachable!()
    };
    crate::tests::federal::other_taxes::ok(&crate::tests::cross::interview::save_other_taxes(
        &mut session,
        shown.taxes,
    ));
    assert_eq!(supported(&restart(session.draft).draft), result);
}

#[test]
fn chapter_43_excise_remains_due_beside_eligible_chapter_one_tax() {
    let mut draft = bba_low_tax_return();
    add(&mut draft, income(archer_msa(300))); // $60 Chapter 1 tax.
    let mut hsa = hsa_entry(Owner::Taxpayer, HsaCoverage::None, 0);
    hsa.excess_accounting = Some(HsaExcessAccounting {
        prior_year: Some(HsaPriorExcess { line_48: 100_000, line_49: 6_000 }),
        employer_excess_in_wages: 0,
        year_end_value: 100_000,
        later_return_year_deposits: 0,
        later_employer_deposits: 0,
    });
    draft.adjustments.as_mut().unwrap().hsa.push(hsa);
    let result = supported(&draft);
    assert_eq!(value(&result, "hsa_return_year_excess_contribution_tax"), 60);
    assert_eq!(value(&result, "schedule_2_line_13z_form_8978"), -60);
    assert_eq!(value(&result, "form_8978_unusable_decrease"), 940);
    assert_eq!(return_line(&result, "23"), 6_000);
}

#[test]
fn medicare_msa_and_deferred_compensation_retain_the_native_chapter_one_amounts() {
    let mut draft = bba_low_tax_return();
    add(&mut draft, income(medicare_msa(500, MedicareMsaPrior::None)));
    // The existing K-1 box20AI source owns the complete interest+additional-tax
    // amount. §409A(a)(1)(B) increases Chapter1 tax by their sum, not just 20%.
    let mut partner = ordinary_partner_source(0, true);
    partner.deferred_compensation_tax = 30_000;
    add(&mut draft, income(FederalIncome::Partnership { data: partner }));
    let result = supported(&draft);
    assert_eq!(value(&result, "medicare_msa_additional_tax"), 250);
    assert_eq!(value(&result, "k1_deferred_compensation_tax"), 300);
    assert_eq!(value(&result, "schedule_2_line_13z_form_8978"), -550);
    assert_eq!(value(&result, "form_8978_unusable_decrease"), 450);
    assert_eq!(return_line(&result, "23"), 0);
}
