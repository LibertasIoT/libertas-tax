// form_1040 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_employee_wages(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::EmployeeWages,
) -> Result<(), CollectFailure> {
    let Source { owner, context, .. } = *src;
    employee_wages_into(
        cx.d,
        owner,
        context,
        #[cfg(test)]
        src.source,
        t,
        r,
        data,
    );
    Ok(())
}

// The Form 8919 / employee-pay payroll owner, shared with the section 179
// before-deduction earned-income view.
pub(crate) fn employee_wages_into(
    d: &FederalDraft,
    owner: usize,
    context: &crate::IssueContext,
    #[cfg(test)] source: &str,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::EmployeeWages,
) {
    match data.income() {
        Ok(amount) => {
            let railroad =
                data.railroad().map(|source| source.known().map(|value| value.unwrap_or(0)));
            let covered = data.coverage();
            let physical = |values: [i64; 2]| {
                if d.setup.as_ref().is_some_and(|setup| joint(setup.filing_status())) {
                    values
                } else if owner == 0 {
                    [values[0], 0]
                } else {
                    [0, values[1]]
                }
            };

            let wage_total = crate::kernel::ratio::checked_sum(t.wages.into_iter().chain([amount]));
            let withholding_total = crate::kernel::ratio::checked_sum([
                t.withholding_w2,
                t.withholding_1099,
                t.withholding_other,
                data.withholding,
                railroad.map_or(0, |values| values[4]),
            ]);
            let medicare_total = crate::kernel::ratio::checked_sum(
                physical(t.medicare_wages)
                    .into_iter()
                    .chain(physical(t.employee_payroll_medicare))
                    .chain(physical(t.employee_unknown_medicare))
                    .chain(physical([0, 1].map(|index| t.employee_tips[index].medicare)))
                    .chain([if covered[1] != Some(false) && railroad.is_none() {
                        amount
                    } else {
                        0
                    }]),
            );
            let social_total = crate::kernel::ratio::checked_sum(
                physical(t.social_wages)
                    .into_iter()
                    .chain(physical(t.employee_payroll_social))
                    .chain(physical(t.employee_unknown_social))
                    .chain(physical([0, 1].map(|index| t.employee_tips[index].social)))
                    .chain([railroad.map_or_else(
                        || if covered[0] != Some(false) { amount } else { 0 },
                        |values| {
                            values[0].min(crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS)
                        },
                    )]),
            );
            let rrta_total = crate::kernel::ratio::checked_sum(
                physical(t.rrta_compensation)
                    .into_iter()
                    .chain([railroad.map_or(0, |values| values[0])]),
            );
            let payroll_total = crate::kernel::ratio::checked_sum(
                physical(t.social_tax)
                    .into_iter()
                    .chain(physical(t.medicare_tax))
                    .chain(physical(t.rrta_medicare))
                    .chain([
                        railroad.map_or(0, |values| values[1]),
                        railroad.map_or(0, |values| values[3]),
                    ]),
            );
            if [
                wage_total,
                withholding_total,
                medicare_total,
                social_total,
                rrta_total,
                payroll_total,
            ]
            .iter()
            .any(Option::is_none)
            {
                crate::issue_localized!(
                    r,
                    context,
                    source,
                    "EMPLOYEE_PAYROLL_RANGE",
                    "The combined employee payroll amounts exceed the supported monetary range."
                );
                return;
            }
            // Check the owner's cent boundary before adding source facts.
            let coverage = data.coverage();
            let social = if coverage[0] == Some(true) { amount } else { 0 };
            let medicare = if coverage[1] == Some(true) { amount } else { 0 };
            let standard = data.reason().is_some();
            let Some((wages, social, medicare)) = t.wages[owner]
                .checked_add(amount)
                .zip(t.employee_payroll_social[owner].checked_add(social))
                .zip(t.employee_payroll_medicare[owner].checked_add(medicare))
                .map(|((wages, social), medicare)| (wages, social, medicare))
            else {
                crate::issue_localized!(
                    r,
                    context,
                    source,
                    "EMPLOYEE_PAYROLL_RANGE",
                    "The combined employee payroll amounts exceed the supported monetary range."
                );
                return;
            };
            t.wages[owner] = wages;
            t.employee_payroll_social[owner] = social;
            t.employee_payroll_medicare[owner] = medicare;
            if standard {
                t.employee_wages[owner] += amount;
                t.employee_8919[owner] += amount;
            } else {
                t.employee_wages_not_covered[owner] += amount;
                if coverage[0] == Some(true) {
                    t.employee_other_social[owner] += amount;
                }
                if coverage[1] == Some(true) {
                    t.employee_other_medicare[owner] += amount;
                }
            }
            if data.railroad().is_none() {
                if coverage[0].is_none() {
                    t.employee_unknown_social[owner] += amount;
                }
                if coverage[1].is_none() {
                    t.employee_unknown_medicare[owner] += amount;
                }
            }
            if let Some(source) = data.railroad() {
                let known = source.known();
                t.employee_rrta_compensation_unknown[owner] |= known[0].is_none();
                t.employee_rrta_withholding_unknown[owner] |= known[4].is_none();
                let payroll = source.known_payroll();
                crate::forms::w2::collect_railroad(t, owner, &payroll);
                t.social_wages[owner] += known[0]
                    .unwrap_or(0)
                    .min(crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS);
                t.social_tax[owner] += known[1].unwrap_or(0);
            }
            if data.form1099 {
                t.withholding_1099 += data.withholding;
            } else {
                t.withholding_other += data.withholding;
            }
            if matches!(data.reason(), Some(crate::EmployeeWageReason::Ss8Pending { .. })) {
                r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Form 8919 code G: retain the separately filed Form SS-8 and its filing date. IRS review of employee status remains pending; the App does not submit that request or guarantee its outcome.", crate::Message::new("NOTE_FORM_8919_CODE_G_RETAIN_THE_SEPARATELY_FILED_FORM_SS_8_AND_ITS_F_74675BA7", alloc::vec![])));
            }
        }
        Err(_message) => {
            crate::issue_localized!(r, context, source, "ISSUE_EMPLOYEE_WAGES_RECORDS", _message,)
        }
    }
}

pub(super) fn collect_wage(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    w: &crate::WageSource,
) -> Result<(), CollectFailure> {
    let Source { owner, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    let w = &crate::FederalWage::from(w);
    crate::forms::w2::collect(
        &mut crate::forms::w2::WageContext {
            totals: t,
            result: r,
            owner,
            context: context.clone(),
            #[cfg(test)]
            source,
        },
        w,
    );
    for entry in &w.other_box12 {
        if let Some((key, _message)) = w2_box12_refusal(entry.code) {
            crate::issue_localized!(r, context, source, key, _message,);
        }
    }
    Ok(())
}

pub(super) struct WithholdingOut {
    pub(super) withholding_25a: i64,
    pub(super) withholding_25b: i64,
    pub(super) withholding_other_dollars: i64,
}

pub(super) fn withholding_phase(r: &mut FederalResult, t: &mut Totals) -> WithholdingOut {
    // Preserve the existing separately reported withholding boundaries once.
    // These annual Dollars are not unrounded source Cents.
    let withholding_25a = dollars(t.withholding_w2);
    let withholding_25b = dollars(t.withholding_1099);
    let withholding_other_dollars = dollars(t.withholding_other);
    if let Some(mut sources) = t.native_withholding_sources.take() {
        sources.settle(
            [t.withholding_w2, t.withholding_1099, t.withholding_other],
            [withholding_25a, withholding_25b, withholding_other_dollars],
        );
        r.withholding_sources = Some(sources);
    }
    WithholdingOut { withholding_25a, withholding_25b, withholding_other_dollars }
}

pub(super) struct Form1040LiabilityIn<'a> {
    pub(super) remaining_tax: i64,
    pub(super) child_nonrefund: i64,
    pub(super) adoption_used: i64,
    pub(super) mortgage_credit_used: i64,
    pub(super) dc_homebuyer_credit: i64,
    pub(super) residential_energy_credit: i64,
    pub(super) personal_refueling_credit: i64,
    pub(super) general_business_credit: i64,
    pub(super) minimum_credit: i64,
    pub(super) bond_credit: i64,
    pub(super) se_tax: i64,
    pub(super) tip_payroll: i64,
    pub(super) employee_wage_tax: i64,
    pub(super) uncollected_payroll: i64,
    pub(super) golden_parachute_tax: i64,
    pub(super) household_tax: i64,
    pub(super) additional_medicare: i64,
    pub(super) niit: i64,
    pub(super) early_tax: i64,
    pub(super) rmd_tax: i64,
    pub(super) hsa_tax: i64,
    pub(super) medicare_msa_tax: i64,
    pub(super) archer_msa_tax: i64,
    pub(super) archer_excess_tax: i64,
    pub(super) account_tax: i64,
    pub(super) able_excess_tax: i64,
    pub(super) coverdell_excess_tax: i64,
    pub(super) trump_excess_tax: i64,
    pub(super) trump_earnings_tax: i64,
    pub(super) ira_excess_tax: i64,
    pub(super) mortgage_recapture_tax: i64,
    pub(super) investment_recapture_tax: i64,
    pub(super) deferred_tax: i64,
    pub(super) bba_schedule_2_adjustment: i64,
    pub(super) regular_tax: i64,
    pub(super) foreign_credit: i64,
    pub(super) premium_repayment: i64,
    pub(super) eic: i64,
    pub(super) actc: i64,
    pub(super) education_refund: i64,
    pub(super) premium_refund: i64,
    pub(super) elective_payment: i64,
    pub(super) chapter_one_part_ii_tax: i64,
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) regular_capital_amounts: crate::capital::CapitalAmounts,
    pub(super) capital_in: [i64; 2],
    pub(super) elected: [i64; 2],
    pub(super) regular_tax_worksheet: crate::income_tax::RegularTaxWorksheet,
    pub(super) limited_credits: crate::income_tax::LimitedPersonalCredits,
    pub(super) credit_potentials: crate::income_tax::PersonalCreditPotentials,
    pub(super) foreign_basis: crate::form_1116::CreditBasis,
    pub(super) averaged_tax: crate::farm_averaging::Calculation<'a>,
    pub(super) parent_rate_required: bool,
    pub(super) has_business_credit: bool,
    pub(super) bba_audit: &'a Option<crate::bba_audit::BbaAuditAdjustment>,
    pub(super) adoption: &'a crate::adoption::Credit,
    pub(super) use_itemized: bool,
}

pub(super) struct Form1040LiabilityOut {
    pub(super) total: i64,
    pub(super) installments: Option<crate::farm_deferral::FarmTaxInstallments>,
    pub(super) installment_first: i64,
    pub(super) installment_offset: i64,
    pub(super) current_tax: i64,
    pub(super) refundable: i64,
}

pub(super) fn form_1040_liability_phase(
    pass: &Pass,
    r: &mut FederalResult,
    i: Form1040LiabilityIn<'_>,
) -> Form1040LiabilityOut {
    let &Pass { d, c, p, repayment, context, .. } = pass;
    let source_claims = context.source_claims;
    let Form1040LiabilityIn {
        remaining_tax,
        child_nonrefund,
        adoption_used,
        mortgage_credit_used,
        dc_homebuyer_credit,
        residential_energy_credit,
        personal_refueling_credit,
        general_business_credit,
        minimum_credit,
        bond_credit,
        se_tax,
        tip_payroll,
        employee_wage_tax,
        uncollected_payroll,
        golden_parachute_tax,
        household_tax,
        additional_medicare,
        niit,
        early_tax,
        rmd_tax,
        hsa_tax,
        medicare_msa_tax,
        archer_msa_tax,
        archer_excess_tax,
        account_tax,
        able_excess_tax,
        coverdell_excess_tax,
        trump_excess_tax,
        trump_earnings_tax,
        ira_excess_tax,
        mortgage_recapture_tax,
        investment_recapture_tax,
        deferred_tax,
        bba_schedule_2_adjustment,
        regular_tax,
        foreign_credit,
        premium_repayment,
        eic,
        actc,
        education_refund,
        premium_refund,
        elective_payment,
        chapter_one_part_ii_tax,
        rental,
        regular_capital_amounts,
        capital_in,
        elected,
        regular_tax_worksheet,
        limited_credits,
        credit_potentials,
        foreign_basis,
        averaged_tax,
        parent_rate_required,
        has_business_credit,
        bba_audit,
        adoption,
        use_itemized,
    } = i;
    let total = remaining_tax
        - child_nonrefund
        - adoption_used
        - mortgage_credit_used
        - dc_homebuyer_credit
        - residential_energy_credit
        - personal_refueling_credit
        - general_business_credit
        - minimum_credit
        - bond_credit
        + se_tax
        + tip_payroll
        + employee_wage_tax
        + uncollected_payroll
        + golden_parachute_tax
        + household_tax
        + additional_medicare
        + niit
        + early_tax
        + rmd_tax
        + hsa_tax
        + medicare_msa_tax
        + archer_msa_tax
        + archer_excess_tax
        + account_tax
        + able_excess_tax
        + coverdell_excess_tax
        + trump_excess_tax
        + trump_earnings_tax
        + ira_excess_tax
        + mortgage_recapture_tax
        + investment_recapture_tax
        + deferred_tax
        - bba_schedule_2_adjustment;
    let installments = crate::farm_deferral::calculate_with_claims(
        d,
        crate::farm_deferral::TaxContext {
            pool: rental,
            capital: regular_capital_amounts,
            capital_prior: capital_in,
            capital_election: elected,
            regular: regular_tax_worksheet,
            actual_regular: regular_tax,
            actual_personal: limited_credits.nonrefundable_total(),
            credits: credit_potentials,
            foreign: &c.foreign_tax,
            foreign_basis: matches!(c.foreign_tax, crate::ForeignTaxClaim::Ordinary { .. })
                .then_some(&foreign_basis),
            actual_foreign: foreign_credit,
            statement_taxes: crate::extended_rules::payee_statement_taxes(d),
            premium_repayment,
            averaging: averaged_tax.comparison,
            total,
            ready: !parent_rate_required
                && p.taxpayer.died.is_none()
                && p.spouse.as_ref().is_none_or(|v| v.died.is_none()),
        },
        r,
        source_claims,
    );
    crate::require_localized!(
        r,
        (c.mortgage_credit.is_none()
            && !has_business_credit
            && c.dc_homebuyer_carry.is_none()
            && c.residential_energy_carry.is_none()
            && c.personal_refueling.is_empty()
            && c.clean_vehicles.as_ref().is_none_or(|claim| claim.vehicles.is_empty())
            && c.legacy_ev_passive_allowed.is_none()
            && investment_recapture_tax == 0
            && c.education_recapture.is_none())
            || installments.is_none(),
        &(crate::IssueContext::Section(crate::TaxSection::Farming)).manual(),
        "Farmland tax installments",
        "ISSUE_FORM_1062_TAX_DEFERRAL_ELECTION_WITH_A_BUSINESS_MORTGAGE_DC_HOMEBUY_129E38EF",
        "Form 1062 tax-deferral election with a business, mortgage, DC homebuyer, residential energy, clean vehicle, legacy EV or refueling credit, or with prior education-credit recapture, needs a combined tax worksheet.",
    );
    let deferred = installments.as_ref().map_or(0, |v| dollars(v.remaining));
    let installment_first = installments.as_ref().map_or(0, |v| dollars(v.first));
    let installment_offset = installments.as_ref().map_or(0, |v| dollars(v.applicable));
    let current_tax = total - deferred;
    crate::require_localized!(
        r,
        repayment.amount == 0 || installments.is_none(),
        &crate::IssueContext::Section(crate::TaxSection::Deductions)
            .alternatives(&[crate::TaxSection::Deductions, crate::TaxSection::Farming]),
        "Repayment comparison",
        "ISSUE_REPAYMENT_COMPARISON_WITH_A_FARMLAND_TAX_DEFERRAL_ELECTION_REQUIRES_C1EF8D22",
        "Repayment comparison with a farmland tax-deferral election requires coordination of the two complete tax methods.",
    );
    crate::require_localized!(
        r,
        bba_audit.is_none() || installments.is_none(),
        &(crate::IssueContext::Section(crate::TaxSection::MoreCredits)).manual(),
        "Form 8978 BBA audit",
        "ISSUE_A_SIGNED_FORM_8978_ADJUSTMENT_COMBINED_WITH_FORM_1062_FARMLAND_TAX_6C431405",
        "A signed Form 8978 adjustment combined with Form 1062 farmland tax installments needs a combined tax worksheet.",
    );
    crate::require_localized!(
        r,
        bba_audit.is_none() || repayment.amount == 0,
        &(crate::IssueContext::Section(crate::TaxSection::MoreCredits)).manual(),
        "Form 8978 BBA audit",
        "ISSUE_A_SIGNED_FORM_8978_ADJUSTMENT_COMBINED_WITH_A_SECTION_1341_REPAYMEN_4272DD73",
        "A signed Form 8978 adjustment combined with a section 1341 repayment comparison needs a combined tax refigure.",
    );
    crate::require_localized!(
        r,
        !has_business_credit || repayment.amount == 0,
        &crate::IssueContext::Section(crate::TaxSection::Deductions)
            .alternatives(&[crate::TaxSection::Deductions, crate::TaxSection::MoreCredits]),
        "General business credit",
        "ISSUE_A_FORM_3800_CURRENT_OR_CARRIED_CREDIT_WITH_A_SECTION_1341_REPAYMENT_A267A4E8",
        "A Form 3800 current or carried credit with a section 1341 repayment needs a combined tax comparison.",
    );
    let refundable = eic
        + actc
        + education_refund
        + adoption.refundable
        + premium_refund
        + repayment.credit
        + elective_payment;
    line(
        r,
        "chapter_one_net_tax",
        remaining_tax
            - child_nonrefund
            - bba_schedule_2_adjustment
            - adoption_used
            - mortgage_credit_used
            - dc_homebuyer_credit
            - residential_energy_credit
            - personal_refueling_credit
            - general_business_credit
            - minimum_credit
            - bond_credit
            + chapter_one_part_ii_tax
            - refundable,
        &["income_tax", "credits", "income"],
        "https://www.govinfo.gov/link/uscode/26/1341",
        msg!("EXPLAIN_CHAPTER_ONE_NET_TAX"),
    );
    line(
        r,
        "repayment_credit",
        repayment.credit,
        &["deductions.repayment"],
        "https://www.law.cornell.edu/uscode/text/26/1341",
        msg!("EXPLAIN_REPAYMENT_CREDIT"),
    );
    form_line(
        r,
        FormLine::new(TaxForm::ScheduleA, "17h"),
        "repayment_itemized_amount",
        if use_itemized { repayment.deduction } else { 0 },
        &["deductions.repayment"],
        "https://www.irs.gov/publications/p525",
        msg!("EXPLAIN_REPAYMENT_ITEMIZED_AMOUNT"),
    );
    Form1040LiabilityOut {
        total,
        installments,
        installment_first,
        installment_offset,
        current_tax,
        refundable,
    }
}

pub(super) struct CreditLineReportingIn<'a, 'd> {
    pub(super) education_used: i64,
    pub(super) form_2441_context: &'a crate::forms::form_2441::Form2441Context<'d>,
    pub(super) creditable_tax: i64,
    pub(super) care_used: i64,
    pub(super) elderly: i64,
    pub(super) saver: i64,
    pub(super) schedule_8812: &'a crate::forms::schedule_8812::Schedule8812,
    pub(super) child_credits: &'a crate::forms::schedule_8812::ChildCredits,
    pub(super) schedule_eic: &'a crate::forms::schedule_eic::ScheduleEic,
    pub(super) education_refund: i64,
    pub(super) additional_medicare: i64,
    pub(super) additional_medicare_self_employment: i64,
    pub(super) additional_medicare_wages: i64,
    pub(super) niit: i64,
    pub(super) payments: i64,
    pub(super) total: i64,
}

pub(super) fn credit_line_reporting_phase(
    pass: &Pass,
    r: &mut FederalResult,
    i: CreditLineReportingIn<'_, '_>,
) {
    let &Pass { c, .. } = pass;
    let CreditLineReportingIn {
        education_used,
        form_2441_context,
        creditable_tax,
        care_used,
        elderly,
        saver,
        schedule_8812,
        child_credits,
        schedule_eic,
        education_refund,
        additional_medicare,
        additional_medicare_self_employment,
        additional_medicare_wages,
        niit,
        payments,
        total,
    } = i;
    line(
        r,
        "education_credit",
        education_used,
        &["agi", "income_tax", "credits", "income", "payments"],
        "https://www.irs.gov/instructions/i8863",
        msg!("EXPLAIN_EDUCATION_CREDIT"),
    );
    if form_2441_context.claims_care(c) {
        crate::forms::form_2441::emit_line_10(positive(creditable_tax), r);
    }
    crate::forms::form_2441::emit_line_11(care_used, r);
    emit_lines(
        r,
        &mut (),
        [
            (
                "elderly_credit",
                elderly,
                "https://www.irs.gov/instructions/i1040sr",
                msg!("EXPLAIN_ELDERLY_CREDIT"),
            ),
            (
                "saver_credit",
                saver,
                "https://www.irs.gov/pub/irs-pdf/f8880.pdf",
                msg!("EXPLAIN_SAVER_CREDIT"),
            ),
        ],
        |(id, value, source, explanation), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(value),
                &["agi", "income_tax", "credits", "income", "payments"],
                source,
                explanation,
            ))
        },
    );
    schedule_8812.emit_lines(child_credits, r);
    schedule_eic.emit_credit(r);
    for (id, value, source, explanation) in [
        (
            "refundable_education_credit",
            education_refund,
            "https://www.irs.gov/instructions/i8863",
            msg!("EXPLAIN_REFUNDABLE_EDUCATION_CREDIT"),
        ),
        (
            "additional_medicare_tax",
            additional_medicare,
            "https://www.irs.gov/instructions/i8959",
            msg!("EXPLAIN_ADDITIONAL_MEDICARE_TAX"),
        ),
        (
            "additional_medicare_self_employment",
            additional_medicare_self_employment,
            "https://www.irs.gov/instructions/i8959",
            msg!("EXPLAIN_ADDITIONAL_MEDICARE_SELF_EMPLOYMENT"),
        ),
        (
            "additional_medicare_wages",
            additional_medicare_wages,
            "https://www.irs.gov/instructions/i8959",
            msg!("EXPLAIN_ADDITIONAL_MEDICARE_WAGES"),
        ),
        (
            "net_investment_income_tax",
            niit,
            "https://www.irs.gov/instructions/i8960",
            msg!("EXPLAIN_NET_INVESTMENT_INCOME_TAX"),
        ),
        ("payments", payments, "https://www.irs.gov/publications/p505", msg!("EXPLAIN_PAYMENTS")),
        ("total_tax", total, "https://www.irs.gov/publications/p505", msg!("EXPLAIN_TOTAL_TAX")),
    ] {
        // Schedule 2 line 6 (Form 8960), line 11 (Form 8959 Part IV line 18) and
        // line 17b (Form 8959 Part III line 12).
        let schedule_2 = match id {
            "net_investment_income_tax" => Some("6"),
            "additional_medicare_self_employment" => Some("11"),
            "additional_medicare_wages" => Some("17b"),
            _ => None,
        };
        push_line(
            r,
            schedule_2.map(|line| FormLine::new(TaxForm::Schedule2, line)),
            id,
            value * 100,
            &["agi", "income_tax", "credits", "income", "payments"],
            source,
            explanation,
        );
    }
}

pub(super) struct PrecreditTaxReportingIn {
    pub(super) use_itemized: bool,
    pub(super) foreign_credit: i64,
    pub(super) regular_liability: i64,
    pub(super) amt: i64,
    pub(super) bba_credit: i64,
    pub(super) deduction: i64,
    pub(super) senior_ded: i64,
    pub(super) qbi_ded: i64,
    pub(super) taxable: i64,
}

pub(super) struct PrecreditTaxReportingOut {
    pub(super) income_tax: i64,
    pub(super) bba_schedule_3_credit: i64,
    pub(super) bba_after_line_18: i64,
    pub(super) creditable_tax: i64,
    pub(super) regular_creditable_tax: i64,
}

pub(super) fn precredit_tax_reporting_phase(
    r: &mut FederalResult,
    input: PrecreditTaxReportingIn,
    qbi_line: impl Fn(&str, &str) -> Option<FormLine>,
) -> PrecreditTaxReportingOut {
    let PrecreditTaxReportingIn {
        use_itemized,
        foreign_credit,
        regular_liability,
        amt,
        bba_credit,
        deduction,
        senior_ded,
        qbi_ded,
        taxable,
    } = input;
    form_line(
        r,
        FormLine::new(TaxForm::Form1116, "35"),
        "foreign_tax_credit",
        foreign_credit,
        &["credits.foreign_tax", "regular_income_tax"],
        "https://www.irs.gov/instructions/i1116",
        msg!("EXPLAIN_FOREIGN_TAX_CREDIT"),
    );
    // Form 1040 line 18 includes the Form 8814 tax of line 16 (inside regular_liability): every
    // credit limit worksheet starts from line 18 and the tax is part of total tax (line 24a).
    let income_tax = regular_liability + amt;
    #[cfg(test)]
    record_credit_base("Form 1040 line 18 less the AMT", income_tax - amt);
    // One current-return capacity follows the published negative Form 8978
    // worksheet. Reserve actual Schedule 3 line 6l once before the existing
    // personal-credit limit owners; keep the full decrease for 6251/1116 above.
    let bba_schedule_3_credit = phase::cap(bba_credit, [income_tax]);
    let bba_after_line_18 = bba_credit - bba_schedule_3_credit;
    let creditable_tax = positive(income_tax - foreign_credit - bba_schedule_3_credit);
    let regular_creditable_tax =
        positive(regular_liability - foreign_credit - bba_schedule_3_credit);
    line(
        r,
        "deduction",
        deduction,
        &["agi", "deductions"],
        LAW,
        if use_itemized { msg!("EXPLAIN_DEDUCTION") } else { msg!("EXPLAIN_DEDUCTION_2") },
    );
    line(
        r,
        "senior_deduction",
        senior_ded,
        &["agi", "people"],
        LAW,
        msg!("EXPLAIN_SENIOR_DEDUCTION"),
    );
    push_line(
        r,
        qbi_line("17", "41"),
        "qbi_deduction",
        qbi_ded * 100,
        &["income.business", "self_employment_tax", "agi", "deduction"],
        LAW,
        msg!("EXPLAIN_QBI_DEDUCTION"),
    );
    line(
        r,
        "taxable_income",
        taxable,
        &[
            "agi",
            "deduction",
            "senior_deduction",
            "tips_deduction",
            "overtime_deduction",
            "vehicle_interest_deduction",
            "qbi_deduction",
        ],
        "https://www.irs.gov/publications/p505",
        msg!("EXPLAIN_TAXABLE_INCOME"),
    );
    line(
        r,
        "income_tax",
        income_tax,
        &["regular_income_tax", "form_8978_line_14", "alternative_minimum_tax"],
        ANNUAL,
        msg!("EXPLAIN_INCOME_TAX"),
    );
    PrecreditTaxReportingOut {
        income_tax,
        bba_schedule_3_credit,
        bba_after_line_18,
        creditable_tax,
        regular_creditable_tax,
    }
}

pub(super) struct RegularLiabilityIn<'a> {
    pub(super) clean_vehicle: &'a crate::clean_vehicle::PersonalCredits,
    pub(super) regular_tax: i64,
    pub(super) premium_repayment: i64,
}

pub(super) struct RegularLiabilityOut {
    pub(super) bba_audit: Option<crate::bba_audit::BbaAuditAdjustment>,
    pub(super) bba_credit: i64,
    pub(super) bba_additional_tax: i64,
    pub(super) child_tax: i64,
    pub(super) clean_vehicle_repayment: i64,
    pub(super) education_recapture_tax: i64,
    pub(super) lump_sum_tax: i64,
    pub(super) regular_liability: i64,
}

pub(super) fn regular_liability_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    input: RegularLiabilityIn<'_>,
) -> RegularLiabilityOut {
    let RegularLiabilityIn { clean_vehicle, regular_tax, premium_repayment } = input;
    let d = pass.d;
    let c = pass.c;
    let j = pass.j;
    let bba_audit = c.bba_audit_interest.as_ref().and_then(|source| match source.refigure() {
        Ok(value) => Some(value),
        Err((key, _message)) => {
            crate::issue_localized!(
                r,
                &if source.document.issue().is_some() {
                    crate::IssueContext::Section(crate::TaxSection::Documents)
                } else if key == "ISSUE_BBA_AUDIT_SCOPE" {
                    crate::IssueContext::Section(crate::TaxSection::MoreCredits).manual()
                } else {
                    crate::IssueContext::Section(crate::TaxSection::MoreCredits)
                },
                "Form 8978 BBA audit",
                key,
                _message,
            );
            None
        }
    });
    let bba_credit = bba_audit.as_ref().map_or(0, |value| value.credit());
    let bba_additional_tax = bba_audit.as_ref().map_or(0, |value| value.additional_tax());
    // Form 8814 line 15 joins Form 1040 line 16 (box 1) beside the Form 8978
    // addition. It is tax on the child's income, so it is not income Schedule J averages
    // (i8814 line 15); it is part of line 18, the base of the credit limits.
    let child_tax = t.child.tax;
    // Positive Form 8978 line 14 is included on Form 1040 line 16. Both Form
    // 1116 line 20 and Form 6251 line 10 use that line; the latter subtracts
    // only a negative Form 8978 line 14. Schedule 2 line 1z also participates.
    let clean_vehicle_repayment = clean_vehicle.new_repayment + clean_vehicle.used_repayment;
    // Recapture of an education credit (Form 8863) is entered on Form 1040 line 16 with box 3
    // checked ("ECR", 2025 Instructions for Form 1040, line 16), so it is part of line 18: every
    // credit limit and Form 6251 line 10 start from it.
    crate::require_localized!(
        r,
        c.education_recapture.as_ref().is_none_or(|history| history.filed_joint == j),
        &(crate::IssueContext::Section(crate::TaxSection::MoreCredits)).manual(),
        "Education recapture",
        "ISSUE_A_DIFFERENT_JOINT_FILING_UNIT_BETWEEN_2025_AND_2026_NEEDS_A_COMPLET_59BF671E",
        "A different joint-filing unit between 2025 and 2026 needs a complete prior-return recapture and ownership review.",
    );
    let education_recapture_tax =
        c.education_recapture.as_ref().map_or(0, |history| match history.additional_tax() {
            Ok(amount) => amount,
            Err((key, _message)) => {
                crate::issue_localized!(
                    r,
                    &if key == "ISSUE_EDUCATION_RECAPTURE_COMPLEX_PRIOR_RETURN" {
                        crate::IssueContext::Section(crate::TaxSection::MoreCredits).manual()
                    } else {
                        crate::IssueContext::Section(crate::TaxSection::MoreCredits)
                    },
                    "Education recapture",
                    key,
                    _message,
                );
                0
            }
        });
    if c.education_recapture.is_some() {
        line(
            r,
            "education_credit_recapture_tax",
            education_recapture_tax,
            &["credits.education_recapture"],
            "https://www.irs.gov/instructions/i8863",
            msg!("EXPLAIN_EDUCATION_CREDIT_RECAPTURE_TAX"),
        );
    }
    // Form 4972 line 30 reaches Form 1040 line 16 box 2: it is regular tax for every credit limit
    // (26 U.S.C. 26(b)(1)), but Form 1116 line 20 and Form 6251 line 10 exclude it.
    let lump_sum_tax = crate::lump_sum_return::calculate(d, r);
    // Form 8814's tax (Form 1040 line 16 box 1) is chapter 1 tax outside 26 U.S.C. 26(b)(2), so
    // it is regular tax liability: the one base of the Form 1116 line 20, Form 8801 line 22,
    // Form 3800, Form 8834 and Form 8911 limits and of Form 1040 line 18.
    let regular_liability = regular_tax
        + bba_additional_tax
        + child_tax
        + lump_sum_tax
        + education_recapture_tax
        + premium_repayment
        + clean_vehicle_repayment;
    RegularLiabilityOut {
        bba_audit,
        bba_credit,
        bba_additional_tax,
        child_tax,
        clean_vehicle_repayment,
        education_recapture_tax,
        lump_sum_tax,
        regular_liability,
    }
}

pub(super) struct RegularTaxSetupIn<'a> {
    pub(super) foreign: &'a crate::extended_rules::ForeignResult,
    pub(super) elected: [i64; 2],
    pub(super) qualified: i64,
    pub(super) tax_net_capital: i64,
    pub(super) schedule_d_18: i64,
    pub(super) schedule_d_19: i64,
    pub(super) taxable: i64,
    pub(super) excluded: i64,
}

pub(super) struct RegularTaxSetupOut {
    pub(super) current_preference: crate::income_tax::Preference,
    pub(super) regular_tax_pref: i64,
    pub(super) regular_collectible: i64,
    pub(super) regular_unrecaptured: i64,
    pub(super) regular_tax_worksheet: crate::income_tax::RegularTaxWorksheet,
    pub(super) regular_tax: i64,
}

pub(super) fn regular_tax_setup_phase(
    pass: &Pass<'_>,
    input: RegularTaxSetupIn<'_>,
) -> RegularTaxSetupOut {
    let &Pass { s, .. } = pass;
    let RegularTaxSetupIn {
        foreign,
        elected,
        qualified,
        tax_net_capital,
        schedule_d_18,
        schedule_d_19,
        taxable,
        excluded,
    } = input;
    let current_preference = crate::income_tax::Preference {
        dividends: positive(qualified - elected[1]),
        gain: tax_net_capital,
        line18: schedule_d_18,
        line19: schedule_d_19,
    };
    let regular_preference =
        if foreign.election() { current_preference.foreign(taxable) } else { current_preference };
    let [regular_tax_pref, regular_collectible, regular_unrecaptured] = regular_preference.capped();
    let regular_tax_worksheet = crate::income_tax::RegularTaxWorksheet {
        taxable,
        preference: current_preference,
        excluded,
        foreign_election: foreign.election(),
        status: s.filing_status(),
    };
    let regular_tax = regular_tax_worksheet.tax();
    RegularTaxSetupOut {
        current_preference,
        regular_tax_pref,
        regular_collectible,
        regular_unrecaptured,
        regular_tax_worksheet,
        regular_tax,
    }
}

pub(super) struct SuccessfulResultIn<'a> {
    pub(super) property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) amt_property_character: &'a Option<crate::section1231::Result1231>,
    pub(super) property_loss_deferred: Result<i64, (&'static str, &'static str)>,
    pub(super) amt_property_loss_deferred: Result<i64, (&'static str, &'static str)>,
    pub(super) installments: Option<crate::FarmTaxInstallments>,
    pub(super) additional: [i64; 3],
    pub(super) se: [i64; 2],
    pub(super) se_profit: [i64; 2],
    pub(super) social_wages: [i64; 2],
    pub(super) use_itemized: bool,
    pub(super) bba_audit: &'a Option<crate::bba_audit::BbaAuditAdjustment>,
    pub(super) total: i64,
    pub(super) refundable: i64,
    pub(super) payment_cents: i64,
    pub(super) refund_cents: i64,
    pub(super) balance_cents: i64,
    pub(super) senior_ded: i64,
    pub(super) itemized: i64,
    pub(super) itemized_reduction: i64,
    pub(super) bba_credit: i64,
    pub(super) bba_schedule_3_credit: i64,
    pub(super) bba_after_line_18: i64,
    pub(super) bba_schedule_2_adjustment: i64,
    pub(super) bba_unusable_decrease: i64,
    pub(super) bba_additional_tax: i64,
}

pub(super) fn successful_result_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: SuccessfulResultIn<'_>,
) -> i64 {
    let Pass { context, .. } = pass;
    let SuccessfulResultIn {
        property_character,
        amt_property_character,
        property_loss_deferred,
        amt_property_loss_deferred,
        installments,
        additional,
        se,
        se_profit,
        social_wages,
        use_itemized,
        bba_audit,
        total,
        refundable,
        payment_cents,
        refund_cents,
        balance_cents,
        senior_ded,
        itemized,
        itemized_reduction,
        bba_credit,
        bba_schedule_3_credit,
        bba_after_line_18,
        bba_schedule_2_adjustment,
        bba_unusable_decrease,
        bba_additional_tax,
    } = input;
    // Basis, at-risk and passive limits already determined these amounts.
    // The verified section 461(l) allocation removes deferred property loss
    // before each tax system's deductible lookback history is finalized.
    for (character, deferred, amt) in [
        (property_character, property_loss_deferred, false),
        (amt_property_character, amt_property_loss_deferred, true),
    ] {
        if let Some(character) = character {
            crate::section1231::record_final_history(
                character,
                deferred.expect("validated property deferral"),
                amt,
                r,
            );
        }
    }
    r.state = ResultState::EstimatedForSupportedScope;
    r.tax = Some(total * 100);
    r.farmland_installments = installments;
    r.refundable_credits = Some(refundable * 100);
    r.payments = Some(payment_cents);
    r.refund = Some(refund_cents);
    r.balance = Some(balance_cents);
    r.financial_source_scope_complete = context.financial_source_scope_complete;

    let additional_deductions = senior_ded + additional.iter().sum::<i64>();
    // Schedule AI column (d) figures not shown on Form 1040: the itemized
    // deductions taken (0 with the standard deduction) and the net profit
    // and Social Security wages of the person with self-employment tax.
    let se_person = (0..2).find(|i| se[*i] > 0);
    line(
        r,
        "schedule_ai_full_year_itemized",
        if use_itemized { itemized - itemized_reduction } else { 0 },
        &["deduction", "itemized_deduction_reduction"],
        "https://www.irs.gov/pub/irs-dft/i2210--dft.pdf",
        msg!("EXPLAIN_SCHEDULE_AI_FULL_YEAR_ITEMIZED"),
    );
    line(
        r,
        "schedule_ai_full_year_self_employment",
        se_person.map_or(0, |i| se_profit[i]),
        &["self_employment_tax"],
        "https://www.irs.gov/pub/irs-dft/i2210--dft.pdf",
        msg!("EXPLAIN_SCHEDULE_AI_FULL_YEAR_SELF_EMPLOYMENT"),
    );
    line_cents(
        r,
        "schedule_ai_full_year_social_wages",
        se_person.map_or(0, |i| social_wages[i]),
        &["income.social_wages"],
        "https://www.irs.gov/pub/irs-dft/i2210--dft.pdf",
        msg!("EXPLAIN_SCHEDULE_AI_FULL_YEAR_SOCIAL_WAGES"),
    );
    if bba_credit > 0 {
        form_line(
            r,
            FormLine::new(TaxForm::Schedule3, "6l"),
            "schedule_3_line_6l",
            bba_schedule_3_credit,
            &["form_8978_line_14", "income_tax"],
            "https://www.irs.gov/instructions/i1040gi",
            msg!("EXPLAIN_SCHEDULE_N_LINE_6L"),
        );
        if bba_after_line_18 > 0 {
            form_line(
                r,
                FormLine::new(TaxForm::Schedule2, "13z"),
                "schedule_2_line_13z_form_8978",
                -bba_schedule_2_adjustment,
                &[
                    "form_8978_line_14",
                    "schedule_3_line_6l",
                    "early_distribution_tax",
                    "hsa_additional_tax",
                    "hsa_testing_period_tax",
                ],
                "https://www.irs.gov/pub/irs-dft/f1040s2--dft.pdf",
                msg!("EXPLAIN_SCHEDULE_N_LINE_13Z_FORM_N"),
            );
            line(
                r,
                "form_8978_unusable_decrease",
                bba_unusable_decrease,
                &["form_8978_line_14", "schedule_3_line_6l", "schedule_2_line_13z_form_8978"],
                "https://www.irs.gov/instructions/i1040gi",
                msg!("EXPLAIN_FORM_N_UNUSABLE_DECREASE"),
            );
        }
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("Form 8978 is based on a 2026-furnished Form 8986 and the published 2025 Tax Table. Schedule 3 line 6l uses ${bba_schedule_3_credit}; 2026 draft Schedule 2 line 13z uses ${bba_schedule_2_adjustment} of the remaining decrease, leaving ${bba_unusable_decrease} unused. The published 2025 worksheet names that second line 17z; reconcile final 2026 instructions before filing."),
            Message::new("NOTE_FORM_8978_DECREASE", alloc::vec![
                MessageArg::usd_dollars(bba_schedule_3_credit), MessageArg::usd_dollars(bba_schedule_2_adjustment),
                MessageArg::usd_dollars(bba_unusable_decrease),
            ]),
        ));
    } else if bba_additional_tax > 0 {
        let source = bba_audit.as_ref().expect("validated audit adjustment");
        let net_payment_cents =
            r.balance.unwrap_or(0) - r.refund.unwrap_or(0) + source.line_17_interest;
        line_cents(
            r,
            "form_8978_line_17_interest",
            source.line_17_interest,
            &["credits.bba_audit_interest.answers.line_17_interest", "form_8978_line_14"],
            "https://www.irs.gov/instructions/i8978",
            msg!("EXPLAIN_FORM_N_LINE_N_INTEREST"),
        );
        line_cents(
            r,
            "form_8978_net_payment_after_interest",
            net_payment_cents,
            &["form_8978_line_17_interest", "form_8978_line_14", "payments"],
            "https://www.irs.gov/instructions/i8978",
            msg!("EXPLAIN_FORM_N_NET_PAYMENT_AFTER_INTEREST"),
        );
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("Form 8978 adds ${bba_additional_tax} of tax to Form 1040 line 16. The separate Form 8978 interest computation adds ${} to the reporting-year net payment, leaving ${} due or ${} of refund after interest. Form 1040 lines 34 and 37 exclude this interest; attach its calculation and reconcile with final 2026 instructions.", source.line_17_interest / 100, positive(net_payment_cents) / 100, positive(-net_payment_cents) / 100),
            Message::new("NOTE_FORM_8978_ADDITIONAL_TAX_AND_INTEREST", alloc::vec![
                MessageArg::usd_dollars(bba_additional_tax), MessageArg::usd_cents(source.line_17_interest),
                MessageArg::usd_cents(positive(net_payment_cents)), MessageArg::usd_cents(positive(-net_payment_cents)),
            ]),
        ));
    }
    additional_deductions
}

pub(super) struct AgiAssemblyIn<'a> {
    pub(super) employee: &'a crate::employee_expenses::Calculation,
    pub(super) ira: &'a crate::ira::IraResult,
    pub(super) before_loan: i64,
    pub(super) loan: i64,
    pub(super) employee_ordinary: i64,
    pub(super) social: i64,
    pub(super) before_social: i64,
    pub(super) exempt: i64,
    pub(super) excluded: i64,
    pub(super) adoption_excluded: i64,
    pub(super) olympic_prize: i64,
}

pub(super) struct AgiAssemblyOut {
    pub(super) employee_counted: alloc::vec::Vec<usize>,
    pub(super) employee_deduction: i64,
    pub(super) social: i64,
    pub(super) olympic_taxable: i64,
    pub(super) agi: i64,
}

pub(super) fn agi_assembly_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    input: AgiAssemblyIn<'_>,
) -> AgiAssemblyOut {
    let d = pass.d;
    let s = pass.s;
    let separate = pass.separate;
    let social_election = pass.social_election;
    let pension = pass.pension;
    let AgiAssemblyIn {
        employee,
        ira,
        before_loan,
        loan,
        employee_ordinary,
        social,
        before_social,
        exempt,
        excluded,
        adoption_excluded,
        olympic_prize,
    } = input;
    // Form 2106: a performing artist's tests read the AGI before any artist's
    // deduction (section 62(b)(1)(C)); the reservist and fee-basis deductions
    // are already in it.
    let agi_before_artist = before_loan - loan;
    let employee_counted =
        employee.artists(agi_before_artist, separate && !s.lived_apart_all_year(), r);
    let employee_artist = employee.artist_deduction(&employee_counted);
    let employee_deduction = employee_ordinary + employee_artist;
    employee.emit(d, r, &employee_counted);
    // The Social Security Benefits Worksheet subtracts the line 12 amount actually claimed
    // (Schedule 1 lines 11 through 20, 23 and 25); the artist's tests above read the AGI
    // with benefits figured before it.
    let social_before_artist = social;
    let social = if employee_artist > 0 {
        social_election.taxable(
            dollars(t.social),
            before_social - ira.deduction - pension.deduction - employee_artist,
            exempt + excluded + adoption_excluded,
            s.filing_status(),
            s.lived_apart_all_year(),
        )
    } else {
        social
    };
    // Section 74(d): an Olympic or Paralympic medal or USOC prize is excluded
    // (line 24c) unless the adjusted gross income without the exclusion is above
    // $1,000,000 ($500,000 married filing separately). Everything above treats
    // the prize as excluded, which is also the adjusted gross income that
    // sections 85(c), 86, 135, 137, 219, 221 and 469 read (section 74(d)(2)(B)).
    let agi_excluded = agi_before_artist - employee_artist + social - social_before_artist;
    let olympic_taxable = if olympic_prize > 0
        && !crate::other_income::OtherIncomeItem::olympic_excluded(
            olympic_prize * 100,
            agi_excluded * 100,
            separate,
        ) {
        olympic_prize
    } else {
        0
    };
    form_line(
        r,
        FormLine::new(TaxForm::Schedule1, "24c"),
        "olympic_exclusion",
        olympic_prize - olympic_taxable,
        &["income.other_income", "agi"],
        "https://www.irs.gov/instructions/i1040gi",
        msg!("EXPLAIN_OLYMPIC_EXCLUSION"),
    );
    let agi = agi_excluded + olympic_taxable;
    AgiAssemblyOut { employee_counted, employee_deduction, social, olympic_taxable, agi }
}

pub(super) struct Form1040ReturnRowsIn<'a> {
    pub(super) actc: i64,
    pub(super) additional_deductions: i64,
    pub(super) additional_income: i64,
    pub(super) additional_taxes: i64,
    pub(super) adjustments: i64,
    pub(super) adoption: &'a crate::adoption::Credit,
    pub(super) adoption_wages: i64,
    pub(super) agi: i64,
    pub(super) amt: i64,
    pub(super) bba_additional_tax: i64,
    pub(super) benefit_wages: i64,
    pub(super) capital: i64,
    pub(super) cash_standard: i64,
    pub(super) child_nonrefund: i64,
    pub(super) child_tax: i64,
    pub(super) clean_vehicle_repayment: i64,
    pub(super) clergy_excess: i64,
    pub(super) combat_pay_election: i64,
    pub(super) credits: i64,
    pub(super) deduction: i64,
    pub(super) deferral_excess: i64,
    pub(super) disability_pension: i64,
    pub(super) dividends: i64,
    pub(super) education_recapture_tax: i64,
    pub(super) education_refund: i64,
    pub(super) eic: i64,
    pub(super) employee_wage_income: i64,
    pub(super) employee_wages_not_covered: i64,
    pub(super) estimated_dollars: i64,
    pub(super) exempt: i64,
    pub(super) foreign_wages: i64,
    pub(super) household_wages: i64,
    pub(super) income_tax: i64,
    pub(super) installment_first: i64,
    pub(super) interest: i64,
    pub(super) itemized: i64,
    pub(super) itemized_reduction: i64,
    pub(super) lump_sum_tax: i64,
    pub(super) medicaid_waiver_wages: i64,
    pub(super) moving_excess: i64,
    pub(super) other_credits: i64,
    pub(super) other_payments: i64,
    pub(super) payment_total_dollars: i64,
    pub(super) penalty: Option<i64>,
    pub(super) premium_repayment: i64,
    pub(super) qbi_ded: i64,
    pub(super) qualified: i64,
    pub(super) qualified_net_casualty: i64,
    pub(super) refund_net: i64,
    pub(super) refundable_subtotal: i64,
    pub(super) regular_tax: i64,
    pub(super) retirement_gross: [i64; 2],
    pub(super) retirement_taxable: [i64; 2],
    pub(super) social: i64,
    pub(super) social_gross: i64,
    pub(super) standard: i64,
    pub(super) strike_benefits: i64,
    pub(super) taxable: i64,
    pub(super) tip_income: i64,
    pub(super) total: i64,
    pub(super) total_income: i64,
    pub(super) use_itemized: bool,
    pub(super) w2_wages: i64,
    pub(super) wages: i64,
    pub(super) withholding_25a: i64,
    pub(super) withholding_25b: i64,
    pub(super) withholding_25c: i64,
    pub(super) withholding_25d: i64,
}

pub(super) fn form_1040_return_rows_phase(
    i: Form1040ReturnRowsIn<'_>,
) -> alloc::boxed::Box<[(&'static str, crate::Message, i64, crate::Message)]> {
    let Form1040ReturnRowsIn {
        actc,
        additional_deductions,
        additional_income,
        additional_taxes,
        adjustments,
        adoption,
        adoption_wages,
        agi,
        amt,
        bba_additional_tax,
        benefit_wages,
        capital,
        cash_standard,
        child_nonrefund,
        child_tax,
        clean_vehicle_repayment,
        clergy_excess,
        combat_pay_election,
        credits,
        deduction,
        deferral_excess,
        disability_pension,
        dividends,
        education_recapture_tax,
        education_refund,
        eic,
        employee_wage_income,
        employee_wages_not_covered,
        estimated_dollars,
        exempt,
        foreign_wages,
        household_wages,
        income_tax,
        installment_first,
        interest,
        itemized,
        itemized_reduction,
        lump_sum_tax,
        medicaid_waiver_wages,
        moving_excess,
        other_credits,
        other_payments,
        payment_total_dollars,
        penalty,
        premium_repayment,
        qbi_ded,
        qualified,
        qualified_net_casualty,
        refund_net,
        refundable_subtotal,
        regular_tax,
        retirement_gross,
        retirement_taxable,
        social,
        social_gross,
        standard,
        strike_benefits,
        taxable,
        tip_income,
        total,
        total_income,
        use_itemized,
        w2_wages,
        wages,
        withholding_25a,
        withholding_25b,
        withholding_25c,
        withholding_25d,
    } = i;
    // Construct the presentation rows outside the calculation frame and
    // iterate their heap storage without copying the large array back.
    boxed(|| {
        [
            ("1a", msg!("RETURN_1040_1A"), w2_wages, msg!("EXPLAIN_1040_1A")),
            ("1b", msg!("RETURN_1040_1B"), household_wages, msg!("EXPLAIN_1040_1B")),
            ("1d", msg!("RETURN_1040_1D"), medicaid_waiver_wages, msg!("EXPLAIN_1040_1D")),
            ("1i", msg!("RETURN_1040_1I"), combat_pay_election, msg!("EXPLAIN_1040_1I")),
            ("1c", msg!("RETURN_1040_1C"), tip_income, msg!("EXPLAIN_1040_1C")),
            ("1e", msg!("RETURN_1040_1E"), benefit_wages, msg!("EXPLAIN_1040_1E")),
            ("1f", msg!("RETURN_1040_1F"), adoption_wages, msg!("EXPLAIN_1040_1F")),
            ("1g", msg!("RETURN_1040_1G"), employee_wage_income, msg!("EXPLAIN_1040_1G")),
            (
                "1h",
                msg!("RETURN_1040_1H"),
                foreign_wages
                    + clergy_excess
                    + disability_pension
                    + moving_excess
                    + deferral_excess
                    + strike_benefits
                    + employee_wages_not_covered,
                msg!("EXPLAIN_1040_1H"),
            ),
            (
                "1z",
                msg!("RETURN_1040_1Z"),
                wages + adoption_wages + benefit_wages + foreign_wages,
                msg!("EXPLAIN_1040_1Z"),
            ),
            ("2a", msg!("RETURN_1040_2A"), exempt, msg!("EXPLAIN_1040_2A")),
            ("2b", msg!("RETURN_1040_2B"), interest, msg!("EXPLAIN_1040_2B")),
            ("3a", msg!("RETURN_1040_3A"), qualified, msg!("EXPLAIN_1040_3A")),
            ("3b", msg!("RETURN_1040_3B"), dividends, msg!("EXPLAIN_1040_3B")),
            ("4a", msg!("RETURN_1040_4A"), dollars(retirement_gross[0]), msg!("EXPLAIN_1040_4A")),
            ("4b", msg!("RETURN_1040_4B"), retirement_taxable[0], msg!("EXPLAIN_1040_4B")),
            ("5a", msg!("RETURN_1040_5A"), dollars(retirement_gross[1]), msg!("EXPLAIN_1040_5A")),
            ("5b", msg!("RETURN_1040_5B"), retirement_taxable[1], msg!("EXPLAIN_1040_5B")),
            ("6a", msg!("RETURN_1040_6A"), dollars(social_gross), msg!("EXPLAIN_1040_6A")),
            ("6b", msg!("RETURN_1040_6B"), social, msg!("EXPLAIN_1040_6B")),
            ("7a", msg!("RETURN_1040_7A"), capital, msg!("EXPLAIN_1040_7A")),
            ("8", msg!("RETURN_1040_8"), additional_income, msg!("EXPLAIN_1040_8")),
            ("9", msg!("RETURN_1040_9"), total_income, msg!("EXPLAIN_1040_9")),
            ("10", msg!("RETURN_1040_10"), adjustments, msg!("EXPLAIN_1040_10")),
            ("11a / 11b", msg!("RETURN_1040_11A_11B"), agi, msg!("EXPLAIN_1040_11A_11B")),
            (
                "12e",
                msg!("RETURN_1040_12E"),
                if use_itemized {
                    itemized - itemized_reduction
                } else {
                    standard + qualified_net_casualty
                },
                msg!("EXPLAIN_1040_12E"),
            ),
            (
                "12f",
                msg!("RETURN_1040_12F"),
                if use_itemized { 0 } else { cash_standard },
                msg!("EXPLAIN_1040_12F"),
            ),
            ("13a", msg!("RETURN_1040_13A"), additional_deductions, msg!("EXPLAIN_1040_13A")),
            ("13b", msg!("RETURN_1040_13B"), qbi_ded, msg!("EXPLAIN_1040_13B")),
            (
                "14",
                msg!("RETURN_1040_14"),
                deduction + additional_deductions + qbi_ded,
                msg!("EXPLAIN_1040_14"),
            ),
            ("15", msg!("RETURN_1040_15"), taxable, msg!("EXPLAIN_1040_15")),
            (
                "16",
                msg!("RETURN_1040_16"),
                regular_tax
                    + bba_additional_tax
                    + child_tax
                    + lump_sum_tax
                    + education_recapture_tax,
                msg!("EXPLAIN_1040_16"),
            ),
            (
                "17",
                msg!("RETURN_1040_17"),
                amt + premium_repayment + clean_vehicle_repayment,
                msg!("EXPLAIN_1040_17"),
            ),
            ("18", msg!("RETURN_1040_18"), income_tax, msg!("EXPLAIN_1040_18")),
            ("19", msg!("RETURN_1040_19"), child_nonrefund, msg!("EXPLAIN_1040_19")),
            ("20", msg!("RETURN_1040_20"), other_credits, msg!("EXPLAIN_1040_20")),
            ("21", msg!("RETURN_1040_21"), credits, msg!("EXPLAIN_1040_21")),
            ("22", msg!("RETURN_1040_22"), positive(income_tax - credits), msg!("EXPLAIN_1040_22")),
            ("23", msg!("RETURN_1040_23"), additional_taxes, msg!("EXPLAIN_1040_23")),
            ("24a", msg!("RETURN_1040_24A"), total, msg!("EXPLAIN_1040_24A")),
            ("24b", msg!("RETURN_1040_24B"), installment_first, msg!("EXPLAIN_1040_24B")),
            ("24c", msg!("RETURN_1040_24C"), total + installment_first, msg!("EXPLAIN_1040_24C")),
            ("25a", msg!("RETURN_1040_25A"), withholding_25a, msg!("EXPLAIN_1040_25A")),
            ("25b", msg!("RETURN_1040_25B"), withholding_25b, msg!("EXPLAIN_1040_25B")),
            ("25c", msg!("RETURN_1040_25C"), withholding_25c, msg!("EXPLAIN_1040_25C")),
            ("25d", msg!("RETURN_1040_25D"), withholding_25d, msg!("EXPLAIN_1040_25D")),
            ("26", msg!("RETURN_1040_26"), estimated_dollars, msg!("EXPLAIN_1040_26")),
            ("27a", msg!("RETURN_1040_27A"), eic, msg!("EXPLAIN_1040_27A")),
            ("28", msg!("RETURN_1040_28"), actc, msg!("EXPLAIN_1040_28")),
            ("29", msg!("RETURN_1040_29"), education_refund, msg!("EXPLAIN_1040_29")),
            ("30", msg!("RETURN_1040_30"), adoption.refundable, msg!("EXPLAIN_1040_30")),
            ("31", msg!("RETURN_1040_31"), other_payments, msg!("EXPLAIN_1040_31")),
            ("32a", msg!("RETURN_1040_32A"), refundable_subtotal, msg!("EXPLAIN_1040_32A")),
            ("32c", msg!("RETURN_1040_32C"), refundable_subtotal, msg!("EXPLAIN_1040_32C")),
            ("33", msg!("RETURN_1040_33"), payment_total_dollars, msg!("EXPLAIN_1040_33")),
            (
                "34",
                msg!("RETURN_1040_34"),
                positive(refund_net),
                if penalty.is_some() { msg!("EXPLAIN_1040_34") } else { msg!("EXPLAIN_1040_34_2") },
            ),
        ]
    })
}

pub(super) struct FinalReturnScheduleProjectionIn<'a> {
    pub(super) excess_social: i64,
    pub(super) premium_refund: i64,
    pub(super) fund_credit: i64,
    pub(super) fuel_credit: i64,
    pub(super) installment_offset: i64,
    pub(super) elective_payment: i64,
    pub(super) eic: i64,
    pub(super) actc: i64,
    pub(super) education_refund: i64,
    pub(super) adoption: &'a crate::adoption::Credit,
    pub(super) payments: i64,
    pub(super) refundable: i64,
    pub(super) premium_repayment: i64,
    pub(super) clean_vehicle: &'a crate::clean_vehicle::PersonalCredits,
    pub(super) clean_vehicle_repayment: i64,
    pub(super) amt: i64,
    pub(super) early_tax: i64,
    pub(super) account_tax: i64,
    pub(super) trump_earnings_tax: i64,
    pub(super) mortgage_recapture_tax: i64,
    pub(super) hsa_distribution_tax: i64,
    pub(super) hsa_testing_tax: i64,
    pub(super) archer_msa_tax: i64,
    pub(super) medicare_msa_tax: i64,
    pub(super) golden_parachute_tax: i64,
    pub(super) investment_recapture_tax: i64,
    pub(super) deferred_tax: i64,
    pub(super) hsa_chapter_one_tax: i64,
    pub(super) bba_schedule_2_adjustment: i64,
    pub(super) se_tax: i64,
    pub(super) niit: i64,
    pub(super) additional_medicare_self_employment: i64,
    pub(super) tip_payroll: i64,
    pub(super) form_8919_tax: i64,
    pub(super) household_tax: i64,
    pub(super) uncollected_payroll: i64,
    pub(super) additional_medicare_wages: i64,
    pub(super) rmd_tax: i64,
    pub(super) hsa_excess_tax: i64,
    pub(super) archer_excess_tax: i64,
    pub(super) able_excess_tax: i64,
    pub(super) coverdell_excess_tax: i64,
    pub(super) trump_excess_tax: i64,
    pub(super) ira_excess_tax: i64,
    pub(super) employee_wage_tax: i64,
    pub(super) additional_taxes: i64,
    pub(super) foreign_credit: i64,
    pub(super) care_used: i64,
    pub(super) education_used: i64,
    pub(super) saver: i64,
    pub(super) residential_energy_credit: i64,
    pub(super) general_business_credit: i64,
    pub(super) minimum_credit: i64,
    pub(super) adoption_used: i64,
    pub(super) elderly: i64,
    pub(super) new_clean_vehicle_credit: i64,
    pub(super) mortgage_credit_used: i64,
    pub(super) dc_homebuyer_credit: i64,
    pub(super) legacy_ev_credit: i64,
    pub(super) personal_refueling_credit: i64,
    pub(super) bond_credit: i64,
    pub(super) used_clean_vehicle_credit: i64,
    pub(super) bba_schedule_3_credit: i64,
    pub(super) other_credits: i64,
    pub(super) other_employee_payroll_tax: i64,
}

pub(super) struct FinalReturnScheduleProjectionOut {
    pub(super) other_payments: i64,
    pub(super) refundable_subtotal: i64,
    pub(super) payment_total_dollars: i64,
}

pub(super) fn final_return_schedule_projection_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    i: FinalReturnScheduleProjectionIn<'_>,
) -> Option<FinalReturnScheduleProjectionOut> {
    let &Pass { pay, repayment, .. } = pass;
    let FinalReturnScheduleProjectionIn {
        excess_social,
        premium_refund,
        fund_credit,
        fuel_credit,
        installment_offset,
        elective_payment,
        eic,
        actc,
        education_refund,
        adoption,
        payments,
        refundable,
        premium_repayment,
        clean_vehicle,
        clean_vehicle_repayment,
        amt,
        early_tax,
        account_tax,
        trump_earnings_tax,
        mortgage_recapture_tax,
        hsa_distribution_tax,
        hsa_testing_tax,
        archer_msa_tax,
        medicare_msa_tax,
        golden_parachute_tax,
        investment_recapture_tax,
        deferred_tax,
        hsa_chapter_one_tax,
        bba_schedule_2_adjustment,
        se_tax,
        niit,
        additional_medicare_self_employment,
        tip_payroll,
        form_8919_tax,
        household_tax,
        uncollected_payroll,
        additional_medicare_wages,
        rmd_tax,
        hsa_excess_tax,
        archer_excess_tax,
        able_excess_tax,
        coverdell_excess_tax,
        trump_excess_tax,
        ira_excess_tax,
        employee_wage_tax,
        additional_taxes,
        foreign_credit,
        care_used,
        education_used,
        saver,
        residential_energy_credit,
        general_business_credit,
        minimum_credit,
        adoption_used,
        elderly,
        new_clean_vehicle_credit,
        mortgage_credit_used,
        dc_homebuyer_credit,
        legacy_ev_credit,
        personal_refueling_credit,
        bond_credit,
        used_clean_vehicle_credit,
        bba_schedule_3_credit,
        other_credits,
        other_employee_payroll_tax,
    } = i;
    // Presentation only: every figure comes from this calculation's inputs
    // and intermediate results, never a separate return-calculation engine.
    // Line references follow the IRS 2026 ATS draft dated September 1, 2026.
    let other_payments = crate::kernel::ratio::checked_sum([
        dollars(pay.extension),
        excess_social,
        premium_refund,
        repayment.credit,
        fund_credit,
        fuel_credit,
        installment_offset,
        elective_payment,
    ])?;
    let refundable_subtotal = crate::kernel::ratio::checked_sum([
        eic,
        actc,
        education_refund,
        adoption.refundable,
        other_payments,
    ])?;
    let payment_total_dollars =
        crate::kernel::ratio::checked_sum([payments, refundable, installment_offset])?;
    // Schedules 2 and 3 as the forms print them: every nonzero line is a row (the amounts
    // that already carry their line are lines 4, 6, 11, 13a and 17b of Schedule 2 and 13a
    // of Schedule 3, and 6l and 13z are tagged where they are figured), and each subtotal is
    // the form's own sum, so the rows foot to Form 1040 lines 17, 20, 23 and 31.
    for (form_line_of, id, amount, explanation) in [
        (
            FormLine::new(TaxForm::Schedule3, "13c"),
            "schedule_3_elective_payment",
            elective_payment,
            msg!("EXPLAIN_SCHEDULE_3_ELECTIVE_PAYMENT"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "1a"),
            "schedule_2_premium_credit_repayment",
            premium_repayment,
            msg!("EXPLAIN_SCHEDULE_2_1A"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "1b"),
            "schedule_2_new_clean_vehicle_repayment",
            clean_vehicle.new_repayment,
            msg!("EXPLAIN_SCHEDULE_2_1B"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "1c"),
            "schedule_2_used_clean_vehicle_repayment",
            clean_vehicle.used_repayment,
            msg!("EXPLAIN_SCHEDULE_2_1C"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "1z"),
            "schedule_2_credit_repayments_total",
            premium_repayment + clean_vehicle_repayment,
            msg!("EXPLAIN_SCHEDULE_2_1Z"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "2"),
            "schedule_2_alternative_minimum_tax",
            amt,
            msg!("EXPLAIN_SCHEDULE_2_2"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "3"),
            "schedule_2_part_one_total",
            premium_repayment + clean_vehicle_repayment + amt,
            msg!("EXPLAIN_SCHEDULE_2_3"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "5"),
            "schedule_2_tax_favored_account_taxes",
            early_tax + account_tax + trump_earnings_tax,
            msg!("EXPLAIN_SCHEDULE_2_5"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "13b"),
            "schedule_2_mortgage_subsidy_recapture",
            mortgage_recapture_tax,
            msg!("EXPLAIN_SCHEDULE_2_13B"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "13c"),
            "schedule_2_hsa_distribution_tax",
            hsa_distribution_tax,
            msg!("EXPLAIN_SCHEDULE_2_13C"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "13d"),
            "schedule_2_hsa_testing_period_tax",
            hsa_testing_tax,
            msg!("EXPLAIN_SCHEDULE_2_13D"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "13e"),
            "schedule_2_archer_msa_tax",
            archer_msa_tax,
            msg!("EXPLAIN_SCHEDULE_2_13E"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "13f"),
            "schedule_2_medicare_msa_tax",
            medicare_msa_tax,
            msg!("EXPLAIN_SCHEDULE_2_13F"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "13k"),
            "schedule_2_golden_parachute_tax",
            golden_parachute_tax,
            msg!("EXPLAIN_SCHEDULE_2_13K"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "14"),
            "schedule_2_other_taxes_total",
            investment_recapture_tax
                + deferred_tax
                + golden_parachute_tax
                + mortgage_recapture_tax
                + hsa_chapter_one_tax
                + archer_msa_tax
                + medicare_msa_tax
                - bba_schedule_2_adjustment,
            msg!("EXPLAIN_SCHEDULE_2_14"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "15"),
            "schedule_2_income_and_other_taxes_subtotal",
            se_tax
                + early_tax
                + account_tax
                + trump_earnings_tax
                + niit
                + additional_medicare_self_employment
                + investment_recapture_tax
                + deferred_tax
                + golden_parachute_tax
                + mortgage_recapture_tax
                + hsa_chapter_one_tax
                + archer_msa_tax
                + medicare_msa_tax
                - bba_schedule_2_adjustment,
            msg!("EXPLAIN_SCHEDULE_2_15"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "16a"),
            "schedule_2_unreported_tip_tax",
            tip_payroll,
            msg!("EXPLAIN_SCHEDULE_2_16A"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "16b"),
            "schedule_2_uncollected_wage_tax",
            form_8919_tax,
            msg!("EXPLAIN_SCHEDULE_2_16B"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "16c"),
            "schedule_2_payroll_tax_subtotal",
            tip_payroll + form_8919_tax,
            msg!("EXPLAIN_SCHEDULE_2_16C"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "17a"),
            "schedule_2_household_employment_tax",
            household_tax,
            msg!("EXPLAIN_SCHEDULE_2_17A"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "17c"),
            "schedule_2_uncollected_tip_tax",
            uncollected_payroll,
            msg!("EXPLAIN_SCHEDULE_2_17C"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "17d"),
            "schedule_2_employment_tax_subtotal",
            household_tax + additional_medicare_wages + uncollected_payroll,
            msg!("EXPLAIN_SCHEDULE_2_17D"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "18"),
            "schedule_2_excess_contribution_taxes",
            rmd_tax
                + hsa_excess_tax
                + archer_excess_tax
                + able_excess_tax
                + coverdell_excess_tax
                + trump_excess_tax
                + ira_excess_tax,
            msg!("EXPLAIN_SCHEDULE_2_18"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "20"),
            "schedule_2_employment_and_other_total",
            tip_payroll
                + employee_wage_tax
                + household_tax
                + additional_medicare_wages
                + uncollected_payroll
                + rmd_tax
                + hsa_excess_tax
                + archer_excess_tax
                + able_excess_tax
                + coverdell_excess_tax
                + trump_excess_tax
                + ira_excess_tax,
            msg!("EXPLAIN_SCHEDULE_2_20"),
        ),
        (
            FormLine::new(TaxForm::Schedule2, "21"),
            "schedule_2_total_additional_taxes",
            additional_taxes,
            msg!("EXPLAIN_SCHEDULE_2_21"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "1"),
            "schedule_3_foreign_tax_credit",
            foreign_credit,
            msg!("EXPLAIN_SCHEDULE_3_1"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "2"),
            "schedule_3_care_credit",
            care_used,
            msg!("EXPLAIN_SCHEDULE_3_2"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "3"),
            "schedule_3_education_credit",
            education_used,
            msg!("EXPLAIN_SCHEDULE_3_3"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "4"),
            "schedule_3_saver_credit",
            saver,
            msg!("EXPLAIN_SCHEDULE_3_4"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "5a"),
            "schedule_3_residential_clean_energy_credit",
            residential_energy_credit,
            msg!("EXPLAIN_SCHEDULE_3_5A"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "6a"),
            "schedule_3_general_business_credit",
            general_business_credit,
            msg!("EXPLAIN_SCHEDULE_3_6A"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "6b"),
            "schedule_3_minimum_tax_credit",
            minimum_credit,
            msg!("EXPLAIN_SCHEDULE_3_6B"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "6c"),
            "schedule_3_adoption_credit",
            adoption_used,
            msg!("EXPLAIN_SCHEDULE_3_6C"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "6d"),
            "schedule_3_elderly_disabled_credit",
            elderly,
            msg!("EXPLAIN_SCHEDULE_3_6D"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "6f"),
            "schedule_3_new_clean_vehicle_credit",
            new_clean_vehicle_credit,
            msg!("EXPLAIN_SCHEDULE_3_6F"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "6g"),
            "schedule_3_mortgage_interest_credit",
            mortgage_credit_used,
            msg!("EXPLAIN_SCHEDULE_3_6G"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "6h"),
            "schedule_3_dc_homebuyer_credit",
            dc_homebuyer_credit,
            msg!("EXPLAIN_SCHEDULE_3_6H"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "6i"),
            "schedule_3_electric_vehicle_credit",
            legacy_ev_credit,
            msg!("EXPLAIN_SCHEDULE_3_6I"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "6j"),
            "schedule_3_refueling_property_credit",
            personal_refueling_credit,
            msg!("EXPLAIN_SCHEDULE_3_6J"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "6k"),
            "schedule_3_tax_credit_bond_credit",
            bond_credit,
            msg!("EXPLAIN_SCHEDULE_3_6K"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "6m"),
            "schedule_3_used_clean_vehicle_credit",
            used_clean_vehicle_credit,
            msg!("EXPLAIN_SCHEDULE_3_6M"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "7"),
            "schedule_3_other_nonrefundable_total",
            general_business_credit
                + minimum_credit
                + adoption_used
                + elderly
                + new_clean_vehicle_credit
                + mortgage_credit_used
                + dc_homebuyer_credit
                + legacy_ev_credit
                + personal_refueling_credit
                + bond_credit
                + bba_schedule_3_credit
                + used_clean_vehicle_credit,
            msg!("EXPLAIN_SCHEDULE_3_7"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "8"),
            "schedule_3_nonrefundable_total",
            other_credits,
            msg!("EXPLAIN_SCHEDULE_3_8"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "9"),
            "schedule_3_net_premium_tax_credit",
            premium_refund,
            msg!("EXPLAIN_SCHEDULE_3_9"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "10"),
            "schedule_3_extension_payment",
            dollars(pay.extension),
            msg!("EXPLAIN_SCHEDULE_3_10"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "11"),
            "schedule_3_excess_social_security",
            excess_social,
            msg!("EXPLAIN_SCHEDULE_3_11"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "12"),
            "schedule_3_fuel_tax_credit",
            fuel_credit,
            msg!("EXPLAIN_SCHEDULE_3_12"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "13b"),
            "schedule_3_repayment_tax_credit",
            repayment.credit,
            msg!("EXPLAIN_SCHEDULE_3_13B"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "13e"),
            "schedule_3_installment_deferral_payment",
            installment_offset,
            msg!("EXPLAIN_SCHEDULE_3_13E"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "14"),
            "schedule_3_other_payments_total",
            fund_credit + repayment.credit + installment_offset + elective_payment,
            msg!("EXPLAIN_SCHEDULE_3_14"),
        ),
        (
            FormLine::new(TaxForm::Schedule3, "15"),
            "schedule_3_refundable_total",
            other_payments,
            msg!("EXPLAIN_SCHEDULE_3_15"),
        ),
    ] {
        if amount != 0
            && other_employee_payroll_tax > 0
            && form_line_of.form == TaxForm::Schedule2
            && matches!(form_line_of.line.as_str(), "20" | "21")
        {
            // No invented Schedule2 destination for a non8919 worksheet.
            // Retain the actual total as an unnumbered estimate amount.
            line(
                r,
                id,
                amount,
                &["income", "credits", "payments"],
                "https://www.irs.gov/irb/2012-52_IRB",
                msg!("EXPLAIN_EMPLOYEE_COVERAGE_PAYROLL"),
            );
        } else if amount != 0 {
            form_line(
                r,
                form_line_of,
                id,
                amount,
                &["income", "credits", "payments"],
                "https://www.irs.gov/instructions/i1040gi",
                explanation,
            );
        }
    }
    Some(FinalReturnScheduleProjectionOut {
        other_payments,
        refundable_subtotal,
        payment_total_dollars,
    })
}

pub(super) struct ReturnRowsIn {
    pub(super) return_rows: alloc::boxed::Box<[(&'static str, Message, i64, Message)]>,
    pub(super) penalty: Option<i64>,
    pub(super) penalty_explanation: Message,
    pub(super) credit_pending: bool,
    pub(super) refund_net: i64,
    pub(super) payment_net: i64,
    pub(super) cash_refund_cents: i64,
    pub(super) reported_credit_cents: i64,
    pub(super) final_balance_cents: i64,
    pub(super) refundable: i64,
    pub(super) total: i64,
    pub(super) final_refund_cents: i64,
}

pub(super) fn return_rows_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: ReturnRowsIn,
    injured_facts: &crate::injured_spouse::JointFacts<'_>,
) -> Option<()> {
    let &Pass { d, pay, context, .. } = pass;
    let ReturnRowsIn {
        return_rows,
        penalty,
        penalty_explanation,
        credit_pending,
        refund_net,
        payment_net,
        cash_refund_cents,
        reported_credit_cents,
        final_balance_cents,
        refundable,
        total,
        final_refund_cents,
    } = input;
    let mut settlement_rows = match penalty {
        None => vec![(
            "37",
            msg!("RETURN_1040_37"),
            positive(match refund_net.checked_neg() {
                Some(value) => value,
                None => {
                    crate::underpayment::amount_unavailable(r);
                    return None;
                }
            }),
            msg!("EXPLAIN_1040_37"),
        )],
        Some(penalty) => {
            // Line 37 is the amount owed including the line 38 penalty;
            // when the overpayment is larger, the penalty comes out of it
            // on line 35a and nothing is owed.
            let net = payment_net;
            let mut lines =
                vec![("37", msg!("RETURN_1040_37_2"), positive(net), msg!("EXPLAIN_1040_37_2"))];
            if net < 0 {
                lines.push(("35a", msg!("RETURN_1040_35A"), -net, msg!("EXPLAIN_1040_35A")));
            }
            lines.push(("38", msg!("RETURN_1040_38"), penalty, penalty_explanation));
            lines
        }
    };
    if credit_pending {
        settlement_rows.retain(|(reference, ..)| !matches!(*reference, "35a" | "36"));
    } else if refund_net > 0 || penalty.is_some_and(|v| v > positive(refund_net)) {
        settlement_rows.retain(|(reference, ..)| *reference != "35a");
        settlement_rows.push((
            "35a",
            msg!("RETURN_1040_35A"),
            dollars(cash_refund_cents),
            msg!("EXPLAIN_1040_35A"),
        ));
        settlement_rows.push((
            "36",
            msg!("RETURN_1040_36"),
            dollars(reported_credit_cents),
            msg!("EXPLAIN_1040_36"),
        ));
        settlement_rows
            .sort_by_key(|(reference, ..)| FormLine::new(TaxForm::Form1040, reference).order());
    }
    for (reference, label, amount, explanation) in
        return_rows.into_vec().into_iter().chain(settlement_rows)
    {
        let form_line = FormLine::new(TaxForm::Form1040, reference);
        let cents = if matches!(
            reference,
            "25a"
                | "25b"
                | "25c"
                | "25d"
                | "26"
                | "31"
                | "32a"
                | "32c"
                | "33"
                | "34"
                | "35a"
                | "36"
                | "37"
                | "38"
        ) {
            match amount.checked_mul(100) {
                Some(value) => value,
                None => {
                    crate::underpayment::amount_unavailable(r);
                    return None;
                }
            }
        } else {
            amount * 100
        };
        r.return_lines.push(FederalAmount {
            label: label.encode(),
            amount: cents,
            explanation: explanation.encode(),
            form_line: Some(form_line),
            money: Some(cents),
        });
    }

    // Draft Schedule 3-A tests affected credits against tax, not against
    // withholding. The proposal is not applied as finalized law. Advisory
    // notes must not enter `issues`, which suppresses supported calculations.
    // Source: https://www.irs.gov/pub/irs-dft/f1040s3a--dft.pdf
    if penalty.is_some() || pay.credit_applied_next_year > 0 {
        r.refund = if credit_pending { None } else { Some(cash_refund_cents) };
        r.balance = Some(final_balance_cents);
    }
    if refundable > total {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Review required before filing: this refund includes refundable credits exceeding your calculated tax. Proposed 2026 Schedule 3-A rules may reduce that portion depending on eligibility. This prototype has not applied the proposed reduction or verified that eligibility. Check the final IRS rules before relying on this refund.", crate::Message::new("NOTE_REVIEW_REQUIRED_BEFORE_FILING_THIS_REFUND_INCLUDES_REFUNDABLE_CR_33EE1376", alloc::vec![])));
    }
    // Form 8379: the injured spouse allocation of this joint
    // return and the estimate of the injured spouse's refund share, from
    // the per-person amounts the calculation already holds.
    if let Some(allocation) = d.injured_spouse.as_ref() {
        crate::injured_spouse::allocate(
            context.forms(d),
            r,
            allocation,
            injured_facts,
            final_refund_cents,
        );
    }
    Some(())
}
