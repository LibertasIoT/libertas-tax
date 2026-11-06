// nol items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn nol_modified_income_with_claims(
    d: &FederalDraft,
    context: EstimateContext<'_>,
    earlier: i64,
    charity_limit: Option<i64>,
    charity_class_ceiling: Option<([i64; 8], u8)>,
    unresolved: &mut Option<FederalResult>,
) -> i64 {
    let mut prior = crate::nol::NolRun {
        deduction: earlier,
        modify_income: true,
        exclude_qbi: true,
        charity_limit,
        charity_class_ceiling,
        ..Default::default()
    };
    let result = estimate_with_ira_with_claims(d, context, None, None, None, &mut prior);
    if result.state != ResultState::EstimatedForSupportedScope {
        *unresolved = Some(result);
        return 0;
    }
    if let Some(modified) = prior.taxable_base {
        modified - earlier
            + prior.qsbs_excluded
            + prior.section1202_medical_adjustment
            + prior.section1202_salt_adjustment
            + prior.section1202_casualty_adjustment
            + prior.section1202_charity_adjustment
            + prior.section1202_section68_adjustment
            + prior.section1202_loan_adjustment
            + prior.section1202_ira_adjustment
            + prior.section1202_social_adjustment
    } else {
        *unresolved = Some(result);
        0
    }
}

pub(super) struct NolCarryoversIn<'a> {
    pub(super) amt_nol: i64,
    pub(super) amt_excess_business_loss: i64,
    pub(super) amt: i64,
    pub(super) loss_carryovers: crate::debt::LossCarryovers,
    pub(super) nol_run: &'a crate::nol::NolRun,
    pub(super) prior_atnol: &'a crate::atnol::AtnolResult,
    pub(super) nol: i64,
    pub(super) excess_business_loss: i64,
    pub(super) farm_route: crate::farm::FarmLossRoute,
    pub(super) ptp_nol_predebt_carry: bool,
    pub(super) has_atnol: bool,
}

pub(super) fn nol_carryovers_phase(
    pass: &Pass,
    r: &mut FederalResult,
    i: NolCarryoversIn<'_>,
) -> core::ops::ControlFlow<()> {
    let &Pass { d, allocated_entries, p, .. } = pass;
    let NolCarryoversIn {
        amt_nol,
        amt_excess_business_loss,
        amt,
        loss_carryovers,
        nol_run,
        prior_atnol,
        nol,
        excess_business_loss,
        farm_route,
        ptp_nol_predebt_carry,
        has_atnol,
    } = i;
    let amt_loss_carryovers = crate::debt::LossCarryovers {
        current_nol: amt_nol,
        excess_business_loss: amt_excess_business_loss,
    };
    let debt_nol_reduction = crate::debt::reduce_attributes(
        d,
        r,
        amt,
        [loss_carryovers, amt_loss_carryovers],
        [
            nol_run
                .allocation
                .as_ref()
                .map_or_else(|| crate::kernel::ratio::exact::whole(0), |v| v.discharge_balance()),
            prior_atnol.discharge_balance(),
        ],
    );
    let current_regular = crate::kernel::carry::native_balance(
        loss_carryovers.total_nol() * 100,
        debt_nol_reduction.regular.current,
    );
    let current_alternative = crate::kernel::carry::native_balance(
        amt_loss_carryovers.total_nol() * 100,
        debt_nol_reduction.alternative.current,
    );
    emit_lines(
        r,
        &mut (),
        [
            ("nol_carryforward", current_regular),
            ("amt_nol_carryforward", current_alternative),
            ("current_nol_debt_reduction", debt_nol_reduction.regular.current),
            ("amt_current_nol_debt_reduction", debt_nol_reduction.alternative.current),
        ],
        |(id, amount), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyCents(amount),
                &["income", "income.personal_debt"],
                "https://www.irs.gov/instructions/i982",
                msg!("EXPLAIN_FEDERAL_RULES_16"),
            ))
        },
    );
    if allocated_entries.iter().any(|e| matches!(e.income, FederalIncome::Scorp { .. }))
        && r.issues.is_empty()
    {
        for (_label, key, loss, excess, reduction, carryforward) in [
            (
                crate::fixture_english!("regular"),
                "NOTE_K1_REGULAR_LOSS_CARRYFORWARD",
                nol,
                excess_business_loss,
                debt_nol_reduction.regular.current,
                current_regular,
            ),
            (
                crate::fixture_english!("AMT"),
                "NOTE_K1_AMT_LOSS_CARRYFORWARD",
                amt_nol,
                amt_excess_business_loss,
                debt_nol_reduction.alternative.current,
                current_alternative,
            ),
        ] {
            r.review_notes.push(ReviewNote::localized(
                #[cfg(test)] alloc::format!("K-1 {_label} loss carryforward worksheet: current operating loss ${loss}; excess business loss ${excess}; debt reduction ${}; total carryforward ${}. The excess loss is included once. Basis, at-risk and passive suspended losses remain separate. Other return eligibility checks remain separate.", dollars(reduction), dollars(carryforward)),
                Message::new(key, alloc::vec![
                    MessageArg::usd_dollars(loss), MessageArg::usd_dollars(excess),
                    MessageArg::usd_dollars(dollars(reduction)), MessageArg::usd_dollars(dollars(carryforward)),
                ]),
            ));
        }
    }
    if farm_route == crate::farm::FarmLossRoute::CarriedBack && r.issues.is_empty() {
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("2026 farming-loss carryback kept. The 2024 and 2025 refund is not in this estimate; claim it on Form 1045 or Form 1040-X. The current loss carryforwards shown, regular ${}; AMT ${}, are before the carryback: the carryforward is the loss less the amount the two earlier years absorb (section 172(b)(1)(B), Form 172 instructions), which this estimate does not compute, so the next-year carryover may be lower.", dollars(current_regular), dollars(current_alternative)),
            Message::new("NOTE_FARMING_LOSS_CARRYBACK_KEPT", alloc::vec![
                MessageArg::usd_dollars(dollars(current_regular)), MessageArg::usd_dollars(dollars(current_alternative)),
            ]),
        ));
    }
    if farm_route == crate::farm::FarmLossRoute::Waived && r.issues.is_empty() {
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("2026 farming-loss carryback waiver selected. Total current loss carryforwards after debt reductions: regular ${}; AMT ${}. These totals include nonfarm operating and Form 461 losses once. Attach the required section 172(b) statement within the applicable election deadline; the generally irrevocable election is not filed by this App.", dollars(current_regular), dollars(current_alternative)),
            Message::new("NOTE_FARMING_LOSS_CARRYBACK_WAIVER", alloc::vec![
                MessageArg::usd_dollars(dollars(current_regular)), MessageArg::usd_dollars(dollars(current_alternative)),
            ]),
        ));
    }
    if let Some(allocation) = &nol_run.allocation
        && let Err(issue) = allocation.record(current_regular, debt_nol_reduction.regular.prior, r)
    {
        issue.report(r);
        return core::ops::ControlFlow::Break(());
    }
    if ptp_nol_predebt_carry && debt_nol_reduction.regular.prior > 0 {
        note_once_scoped(
            r,
            Message::new("NOTE_NOL_PTP_EBL_DEBT_CHARACTER", alloc::vec![]),
            crate::IssueContext::Section(crate::TaxSection::SelfEmployment),
        );
    }
    if has_atnol
        && let Err(issue) =
            prior_atnol.record(current_alternative, debt_nol_reduction.alternative.prior, r)
    {
        issue.report(r);
        return core::ops::ControlFlow::Break(());
    }
    // Pub. 559 "Deduction for Losses": a decedent's unused loss carryovers end
    // with the final return; the survivor's own part of a joint return carries
    // forward, which the App does not divide. The carryforward lines stay as
    // computed, with one note.
    if p.taxpayer.died.is_some() || p.spouse.as_ref().is_some_and(|v| v.died.is_some()) {
        let has_carry = r.lines.iter().any(|v| {
            v.amount > 0
                && (v.id.contains("carryforward")
                    || v.id.contains("carryover")
                    || v.id.ends_with("_suspended_loss"))
        });
        if has_carry {
            note_once(r, msg!("NOTE_FINAL_RETURN_CARRYOVERS_END"));
        }
    }
    core::ops::ControlFlow::Continue(())
}

pub(super) struct RegularNolRefigureIn<'a> {
    pub(super) modified_ira:
        &'a Option<Result<crate::ira::IraResult, (&'static str, &'static str)>>,
    pub(super) ira: &'a crate::ira::IraResult,
    pub(super) salt_eligible: &'a crate::kernel::ratio::exact::Exact,
    pub(super) charity_claims: crate::charity::ClassClaims,
    pub(super) additional: [i64; 3],
    pub(super) kiddie_age: bool,
    pub(super) qsbs_with_prior_nol: bool,
    pub(super) adoption_exclusion_unchanged_under_qsbs: bool,
    pub(super) passive_unchanged_under_qsbs: bool,
    pub(super) use_itemized: bool,
    pub(super) foreign_magi: i64,
    pub(super) noncash_claim: i64,
    pub(super) hosted_student_expenses: i64,
    pub(super) whaling_expenses: i64,
    pub(super) salt: i64,
    pub(super) medical_deduction: i64,
    pub(super) medical_before_limit: i64,
    pub(super) agi: i64,
    pub(super) ordinary_casualty_deduction: i64,
    pub(super) casualty_net: i64,
    pub(super) qualified_net_casualty: i64,
    pub(super) charity_deduction: i64,
    pub(super) itemized: i64,
    pub(super) unlimited_deduction: i64,
    pub(super) senior_ded: i64,
    pub(super) qbi_ded: i64,
    pub(super) itemized_reduction: i64,
}

pub(super) fn regular_nol_refigure_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    nol_run: &mut crate::nol::NolRun,
    input: RegularNolRefigureIn<'_>,
) {
    let &Pass { s, a, ded, c, screen, allocated_entries, .. } = pass;
    let RegularNolRefigureIn {
        modified_ira,
        ira,
        salt_eligible,
        charity_claims,
        additional,
        kiddie_age,
        qsbs_with_prior_nol,
        adoption_exclusion_unchanged_under_qsbs,
        passive_unchanged_under_qsbs,
        use_itemized,
        foreign_magi,
        noncash_claim,
        hosted_student_expenses,
        whaling_expenses,
        salt,
        medical_deduction,
        medical_before_limit,
        agi,
        ordinary_casualty_deduction,
        casualty_net,
        qualified_net_casualty,
        charity_deduction,
        itemized,
        unlimited_deduction,
        senior_ded,
        qbi_ded,
        itemized_reduction,
    } = input;
    if screen.nol.present() {
        crate::require_localized!(
            r,
            !kiddie_age && !matches!(c.foreign_tax, crate::ForeignTaxClaim::Ordinary { .. }),
            &crate::IssueContext::Business,
            "Regular loss history",
            "ISSUE_PRIOR_REGULAR_NOL_WITH_YOUNG_FILER_OR_FORM_1116_REQUIRES_FURTHER_NOL_WORKSHEETS",
            "Prior regular NOL with a young filer or Form 1116 requires further coordinated NOL and carryforward worksheets.",
        );
    }
    // Form 172 Part II adds excluded section 1202 gain to modified income.
    // Its line 6 also refigures AGI-sensitive items; the ordinary return's
    // worksheets cannot stand in for that separate calculation.
    let ira_recalculation_supported = modified_ira.as_ref().is_none_or(|result| {
        result.as_ref().is_ok_and(|modified| {
            (modified.deduction == ira.deduction && modified.nondeductible == ira.nondeductible)
                || (a.qualified_charity.is_empty()
                    && !allocated_entries.iter().any(|entry| {
                        entry
                            .income
                            .retirement()
                            .is_some_and(|data| data.kind == crate::RetirementKind::Ira)
                    }))
        })
    });
    crate::require_localized!(
        r,
        !qsbs_with_prior_nol
            || (ira_recalculation_supported
                && adoption_exclusion_unchanged_under_qsbs
                && passive_unchanged_under_qsbs),
        &crate::IssueContext::Section(crate::TaxSection::InvestmentSales)
            .alternatives(&[crate::TaxSection::InvestmentSales, crate::TaxSection::SelfEmployment]),
        "Section 1202 prior NOL AGI",
        "ISSUE_A_PRIOR_REGULAR_NOL_AND_SECTION_1202_EXCLUSION_NEED_FURTHER_FORM_17_B3F898C0",
        "A prior regular NOL and section 1202 exclusion need further Form 172 Part II line 6 worksheets when a changed traditional IRA deduction also affects a same-year IRA distribution or QCD, an employer adoption-benefit exclusion changes, or the passive activity result changes under restored gain.",
    );
    // Part II lines 11–33 refigure medical, charitable and casualty amounts.
    // Refigure AGI-sensitive itemized limits for each earlier-loss absorption
    // pass, including the restored section 68 limitation.
    let modified_salt_cap = salt_cap_current_year(
        s.filing_status(),
        foreign_magi
            + dollars(t.qsbs_excluded)
            + nol_run.section1202_social_adjustment
            + nol_run.section1202_ira_adjustment
            + nol_run.section1202_loan_adjustment,
    );
    crate::require_localized!(
        r,
        !qsbs_with_prior_nol
            || !use_itemized
            || (ded.charity_carryovers.is_empty()
                && noncash_claim == 0
                && hosted_student_expenses == 0
                && whaling_expenses == 0),
        &crate::IssueContext::Section(crate::TaxSection::GiftCarryovers),
        "Section 1202 prior NOL itemization",
        "ISSUE_A_PRIOR_REGULAR_NOL_AND_SECTION_1202_EXCLUSION_WITH_PRIOR_CHARITABL_CB8CB26B",
        "A prior regular NOL and section 1202 exclusion with prior charitable carryovers, noncash, hosted-student or whaling expenses needs further Form 172 Part II modified-income worksheets. Current cash gifts, including percentage-limit excess, are supported.",
    );
    nol_run.section1202_salt_adjustment = if nol_run.modify_income && use_itemized {
        salt - exact::rounded(&phase::cap(salt_eligible.clone(), [exact::whole(modified_salt_cap)]))
            .expect("modified SALT deduction is bounded by its native statutory cap")
    } else {
        0
    };
    nol_run.section1202_medical_adjustment = if nol_run.modify_income && use_itemized {
        medical_deduction
            - positive(
                medical_before_limit
                    - fraction(
                        positive(
                            agi + dollars(t.qsbs_excluded)
                                + nol_run.section1202_social_adjustment
                                + nol_run.section1202_ira_adjustment
                                + nol_run.section1202_loan_adjustment,
                        ),
                        75,
                        1000,
                    ),
            )
    } else {
        0
    };
    nol_run.section1202_casualty_adjustment = if nol_run.modify_income && use_itemized {
        let modified_agi = positive(
            agi + dollars(t.qsbs_excluded)
                + nol_run.section1202_social_adjustment
                + nol_run.section1202_ira_adjustment
                + nol_run.section1202_loan_adjustment,
        );
        ordinary_casualty_deduction
            - positive(casualty_net - qualified_net_casualty - fraction(modified_agi, 1, 10))
    } else {
        0
    };
    nol_run.section1202_charity_adjustment =
        if nol_run.modify_income && use_itemized && t.qsbs_excluded > 0 {
            let modified_agi = positive(
                agi + dollars(t.qsbs_excluded)
                    + nol_run.section1202_social_adjustment
                    + nol_run.section1202_ira_adjustment
                    + nol_run.section1202_loan_adjustment,
            );
            let modified_charity = crate::charity::Worksheet::calculate(
                modified_agi,
                charity_claims,
                &[],
                ded.basis_election,
            );
            charity_deduction - nol_run.limit_charity(&modified_charity)
        } else {
            0
        };
    nol_run.section1202_section68_adjustment = if nol_run.modify_income && use_itemized {
        let modified_itemized = positive(
            itemized
                - nol_run.section1202_medical_adjustment
                - nol_run.section1202_salt_adjustment
                - nol_run.section1202_casualty_adjustment
                - nol_run.section1202_charity_adjustment,
        );
        let modified_taxable_before_limit = positive(
            agi - unlimited_deduction - senior_ded - additional.iter().sum::<i64>() - qbi_ded
                + dollars(t.qsbs_excluded)
                + nol_run.section1202_social_adjustment
                + nol_run.section1202_ira_adjustment
                + nol_run.section1202_loan_adjustment
                + nol_run.section1202_medical_adjustment
                + nol_run.section1202_salt_adjustment
                + nol_run.section1202_casualty_adjustment
                + nol_run.section1202_charity_adjustment,
        );
        fraction(
            phase::cap(
                modified_itemized,
                [positive(
                    modified_taxable_before_limit + modified_itemized
                        - brackets(s.filing_status())[5],
                )],
            ),
            2,
            37,
        ) - itemized_reduction
    } else {
        0
    };
}

pub(super) struct CurrentNolIn<'a> {
    pub(super) ira: &'a crate::ira::IraResult,
    pub(super) regular_capital: &'a crate::capital::CapitalResult,
    pub(super) property_character: Option<&'a crate::section1231::Result1231>,
    pub(super) amt_property_character: Option<&'a crate::section1231::Result1231>,
    pub(super) regular_capital_amounts: crate::capital::CapitalAmounts,
    pub(super) amt_capital_amounts: crate::capital::CapitalAmounts,
    pub(super) estate_business_carry: [Option<[i64; 2]>; 2],
    pub(super) capital_in: [i64; 2],
    pub(super) amt_capital_in: [i64; 2],
    pub(super) additional: [i64; 3],
    pub(super) use_itemized: bool,
    pub(super) agi: i64,
    pub(super) deduction: i64,
    pub(super) senior_ded: i64,
    pub(super) interest: i64,
    pub(super) dividends: i64,
    pub(super) retirement: i64,
    pub(super) trump_death: i64,
    pub(super) social: i64,
    pub(super) hsa_income: i64,
    pub(super) alimony: i64,
    pub(super) gambling: i64,
    pub(super) jury: i64,
    pub(super) scholarship: i64,
    pub(super) prizes: i64,
    pub(super) other_line_8: i64,
    pub(super) olympic_taxable: i64,
    pub(super) taxable_grants: i64,
    pub(super) family_leave: i64,
    pub(super) casualty_living_insurance: i64,
    pub(super) personal_debt: i64,
    pub(super) care_benefits: i64,
    pub(super) medicare_msa: i64,
    pub(super) archer_total_income: i64,
    pub(super) able_income: i64,
    pub(super) education_income: i64,
    pub(super) tax_refund: i64,
    pub(super) property_net: i64,
    pub(super) loan_gain: i64,
    pub(super) k1_business_ordinary: i64,
    pub(super) hobby: i64,
    pub(super) claimed_casualty: i64,
    pub(super) itemized_reduction: i64,
    pub(super) itemized: i64,
    pub(super) sep_deduction: i64,
    pub(super) hsa: i64,
    pub(super) msa_deduction: i64,
    pub(super) loan: i64,
    pub(super) savings_penalty: i64,
    pub(super) alimony_paid: i64,
    pub(super) jury_remitted: i64,
    pub(super) estate_67e: i64,
    pub(super) capital: i64,
    pub(super) salt: i64,
    pub(super) excess_business_loss: i64,
}

pub(super) struct CurrentNolOut {
    pub(super) nonbusiness_capital_net: i64,
    pub(super) amt_nonbusiness_capital_net: i64,
    pub(super) amt_business_capital_net: i64,
    pub(super) nonbusiness_income: i64,
    pub(super) regular_business_casualty: i64,
    pub(super) nonbusiness_deductions: i64,
    pub(super) nol: i64,
    pub(super) prior_capital_source_known: bool,
    pub(super) loss_carryovers: crate::debt::LossCarryovers,
}

pub(super) fn current_nol_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    nol_run: &mut crate::nol::NolRun,
    input: CurrentNolIn<'_>,
) -> CurrentNolOut {
    let &Pass { screen, pension, .. } = pass;
    let CurrentNolIn {
        ira,
        regular_capital,
        property_character,
        amt_property_character,
        regular_capital_amounts,
        amt_capital_amounts,
        estate_business_carry,
        capital_in,
        amt_capital_in,
        additional,
        use_itemized,
        agi,
        deduction,
        senior_ded,
        interest,
        dividends,
        retirement,
        trump_death,
        social,
        hsa_income,
        alimony,
        gambling,
        jury,
        scholarship,
        prizes,
        other_line_8,
        olympic_taxable,
        taxable_grants,
        family_leave,
        casualty_living_insurance,
        personal_debt,
        care_benefits,
        medicare_msa,
        archer_total_income,
        able_income,
        education_income,
        tax_refund,
        property_net,
        loan_gain,
        k1_business_ordinary,
        hobby,
        claimed_casualty,
        itemized_reduction,
        itemized,
        sep_deduction,
        hsa,
        msa_deduction,
        loan,
        savings_penalty,
        alimony_paid,
        jury_remitted,
        estate_67e,
        capital,
        salt,
        excess_business_loss,
    } = input;
    // Section 151's senior deduction reduces the section 172(a) limit base.
    // The carryover pass excludes it under section 172(b)(2)/(d)(3).
    nol_run.taxable_base =
        Some(agi + nol_run.deduction - deduction - senior_ded - additional.iter().sum::<i64>());
    nol_run.qsbs_excluded = dollars(t.qsbs_excluded);
    // Form 172: personal deductions cannot create an NOL beyond nonbusiness
    // income, capital losses beyond gains cannot create an NOL, and QBI and
    // section 151 senior deductions are excluded. Employee wages, unemployment
    // and rental income are business income for this worksheet.
    let estate_business_loss =
        estate_business_carry.map(|parts| parts.map_or(0, |v| dollars(v[0] + v[1])));
    let nonbusiness_capital_net = regular_capital_amounts.short + regular_capital_amounts.long
        - capital_in.iter().sum::<i64>()
        + nol_run.qsbs_excluded
        + estate_business_loss[0];
    let business_capital_net =
        property_character.as_ref().map_or(0, |v| v.long_term) - estate_business_loss[0];
    let amt_nonbusiness_capital_net = amt_capital_amounts.short + amt_capital_amounts.long
        - amt_capital_in.iter().sum::<i64>()
        + nol_run.qsbs_excluded
        + estate_business_loss[1];
    let amt_business_capital_net =
        amt_property_character.as_ref().map_or(0, |v| v.long_term) - estate_business_loss[1];
    let nonbusiness_income = interest
        + dividends
        + t.child.other
        + retirement
        + trump_death
        + social
        + hsa_income
        + alimony
        + gambling
        + jury
        + scholarship
        + prizes
        + other_line_8
        + olympic_taxable
        + taxable_grants
        + family_leave
        + casualty_living_insurance
        + personal_debt
        + care_benefits
        + medicare_msa
        + archer_total_income
        + able_income
        + education_income
        + tax_refund
        + property_net
        + loan_gain
        - k1_business_ordinary
        + hobby
        + positive(nonbusiness_capital_net);
    // Section 172(d)(4)(C) treats allowed section 165(c)(3) casualty losses
    // as business deductions for NOL purposes, including state-declared
    // losses newly eligible in 2026. An increased standard deduction remains
    // a standard deduction. Allocate the section 68 reduction consistently
    // with the other itemized source deductions above.
    let regular_business_casualty = if use_itemized {
        claimed_casualty - fraction(itemized_reduction, claimed_casualty, itemized.max(1))
    } else {
        0
    };
    // The moving deduction (Form 3903) is a business deduction here and stays
    // out of this sum: Pub. 536 (NOL Step 2, line 6) lists moving expenses of
    // Armed Forces members on active duty, like educator expenses, among the
    // business deductions, and section 172(d)(4)(A) treats an employee's
    // services as a trade or business (the intelligence-community mover).
    let nonbusiness_deductions = deduction - regular_business_casualty
        + sep_deduction
        + dollars(hsa)
        + msa_deduction
        + ira.deduction
        + pension.deduction
        + loan
        + additional[2]
        + savings_penalty
        + alimony_paid
        + jury_remitted
        + estate_67e;
    let prior_capital_source_known =
        screen.capital_history.iter().all(crate::extended_rules::all_capital_is_investment);
    crate::require_localized!(
        r,
        prior_capital_source_known
            || capital_in.iter().all(|v| *v == 0)
            || business_capital_net <= 0
            || agi + nol_run.deduction - deduction - additional.iter().sum::<i64>() >= 0,
        &crate::IssueContext::Section(crate::TaxSection::InvestmentSales),
        "NOL capital-loss source",
        "ISSUE_PRIOR_CAPITAL_LOSSES_NOT_ESTABLISHED_AS_INVESTMENT_LOSSES_NEED_BUSI_1755E978",
        "Prior capital losses not established as investment losses need business/nonbusiness source allocation when current property gains enter an NOL. A Form 8960 difference alone does not establish that allocation.",
    );
    let computed_nol = crate::business_rules::current_nol(
        agi + nol_run.deduction - deduction - additional.iter().sum::<i64>(),
        crate::business_rules::NolCapital {
            return_capital: capital,
            nonbusiness_capital: nonbusiness_capital_net,
            business_capital: business_capital_net,
            section1202_excluded: nol_run.qsbs_excluded,
            schedule_capital_net: regular_capital.short + regular_capital.long,
        },
        nonbusiness_deductions,
        nonbusiness_income - positive(nonbusiness_capital_net),
    );
    let nol = if estate_business_carry[0].is_none()
        && agi + nol_run.deduction - deduction - additional.iter().sum::<i64>() < 0
    {
        note_once_scoped(
            r,
            msg!("WARN_ESTATE_CAPITAL_NOL_LEFT_OUT"),
            crate::IssueContext::Section(crate::TaxSection::PassThrough).manual(),
        );
        0
    } else {
        computed_nol
    };
    crate::require_localized!(
        r,
        nol == 0 || !use_itemized || salt == 0 || nonbusiness_deductions <= nonbusiness_income,
        &crate::IssueContext::Section(crate::TaxSection::Taxes),
        "Business loss",
        "ISSUE_AN_NOL_WITH_EXCESS_PERSONAL_DEDUCTIONS_AND_ITEMIZED_STATE_INCOME_TA_414E8FA6",
        "An NOL with excess personal deductions and itemized state income tax needs business/personal state-tax allocation.",
    );
    line(
        r,
        "current_net_operating_loss",
        nol,
        &["agi", "deductions", "capital_gain"],
        "https://www.irs.gov/instructions/i172",
        msg!("EXPLAIN_CURRENT_NET_OPERATING_LOSS"),
    );
    let loss_carryovers = crate::debt::LossCarryovers { current_nol: nol, excess_business_loss };
    line(
        r,
        "excess_business_loss_carryforward",
        loss_carryovers.excess_business_loss,
        &["excess_business_loss"],
        "https://www.irs.gov/instructions/i172",
        msg!("EXPLAIN_EXCESS_BUSINESS_LOSS_CARRYFORWARD"),
    );
    CurrentNolOut {
        nonbusiness_capital_net,
        amt_nonbusiness_capital_net,
        amt_business_capital_net,
        nonbusiness_income,
        regular_business_casualty,
        nonbusiness_deductions,
        nol,
        prior_capital_source_known,
        loss_carryovers,
    }
}
