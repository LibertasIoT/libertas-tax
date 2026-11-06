// schedule_1 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_other_income(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::OtherIncomeItem,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, owner, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    t.injured_other_sources.insert(e.id, Some(exact::whole(0)));
    t.withholding_1099 += data.withholding;
    crate::other_income::recovery_issues(
        d,
        e,
        r,
        context,
        #[cfg(test)]
        source,
    );
    crate::other_income::collect(
        &mut crate::other_income::Ctx {
            totals: t,
            result: r,
            id: e.id,
            owner,
            context: context.clone(),
        },
        &data.kind,
        data.amount,
    );
    Ok(())
}

pub(super) fn collect_income_without_form(d: &FederalDraft, t: &mut Totals, r: &mut FederalResult) {
    let Some(other) = d.without_form.as_ref().and_then(|answers| answers.other.as_ref()) else {
        return;
    };
    let item = crate::OtherIncomeItem {
        kind: crate::OtherIncomeKind::Other { description: other.description.clone() },
        amount: other.amount,
        withholding: 0,
    };
    // This answer belongs to the page, not to a payer statement. Its amount
    // uses the existing income owner without creating a document or recipient.
    let amount = item.lines().other_8z;
    t.other_8z += amount;
    crate::line_emission::line_cents(
        r,
        "income_without_form_other",
        amount,
        &["without_form.other"],
        "https://www.irs.gov/publications/p525",
        Message::new(
            "EXPLAIN_OTHER_INCOME_N_OTHER",
            alloc::vec![MessageArg::Text(other.description.clone())],
        ),
    );
    // Pub. 555 requires the actual recipient and property character before
    // dividing a separate return's income. An annual amount supplies neither.
    crate::require_localized!(
        r,
        amount == 0 || !d.setup.as_ref().is_some_and(crate::community::applies),
        &crate::IssueContext::Section(crate::TaxSection::IncomeWithoutForm).manual(),
        "Income without a form",
        "ISSUE_WITHOUT_FORM_COMMUNITY_SOURCE_NEEDED",
        "To divide this income for a community-property return, its actual recipient and community or separate-property treatment are needed.",
    );
}

pub(super) fn collect_nec1099(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::information_returns::Nec1099Source,
) -> Result<(), CollectFailure> {
    let Source { e, owner, context, .. } = *src;
    let data = crate::FederalForm1099Nec::from(data);
    t.injured_other_sources.insert(e.id, Some(exact::whole(0)));
    t.withholding_1099 += data.withholding;
    t.golden_parachute_tax[owner] += fraction(
        data.golden_parachute,
        crate::year_constants::GOLDEN_PARACHUTE_EXCISE_PERCENT,
        100,
    );
    crate::information_returns::collect_local(
        &mut crate::other_income::Ctx {
            totals: t,
            result: r,
            id: e.id,
            owner,
            context: context.clone(),
        },
        data.belongs.as_ref(),
        data.compensation,
    );
    Ok(())
}

pub(super) fn collect_misc1099(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::Misc1099Source,
) -> Result<(), CollectFailure> {
    let Source { e, owner, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    let data = &crate::Form1099Misc::from(data);
    t.injured_other_sources.insert(e.id, Some(exact::whole(0)));
    t.withholding_1099 += data.withholding;
    crate::require_localized!(
        r,
        data.deferred == 0,
        context,
        source,
        "ISSUE_FORM_1099_MISC_BOX_15_SECTION_409A",
        "Form 1099-MISC box 15 nonqualified deferred compensation that fails section 409A carries a 20% additional tax and premium interest (Schedule 2 line 13h) that are not calculated; the deferral history they need is not kept.",
    );
    if data.substitute != 0 {
        t.substitute += data.substitute;
        t.injured_substitute[owner] += data.substitute;
        crate::other_income::collect_8z(
            &mut crate::other_income::Ctx {
                totals: t,
                result: r,
                id: e.id,
                owner,
                context: context.clone(),
            },
            "substitute_payments",
            data.substitute,
            "EXPLAIN_OTHER_INCOME_N_SUBSTITUTE_PAYMENTS",
        );
    }
    // The relief calculations own their routes; the form stays as printed.
    match &data.other_use {
        Some(crate::IncomeUse::DisasterRelief { relief }) => {
            let relief = relief.relief(data.other);
            collect_disaster_relief(_cx, src, t, r, &relief)?;
            t.injured_other_amount(e.id, Some(relief.on_return()));
        }
        Some(crate::IncomeUse::WildfireRelief { relief }) => {
            let relief = relief.relief(data.other);
            collect_wildfire_relief(_cx, src, t, r, &relief)?;
            t.injured_other_amount(e.id, Some(relief.on_return()));
        }
        _ => {}
    }
    if let Some(belongs) = &data.other_use {
        crate::information_returns::collect_local(
            &mut crate::other_income::Ctx {
                totals: t,
                result: r,
                id: e.id,
                owner,
                context: context.clone(),
            },
            Some(belongs),
            data.other,
        );
    }
    Ok(())
}

pub(super) fn collect_k1099(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::information_returns::K1099Source,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, context, .. } = *src;
    t.injured_other_sources.insert(e.id, Some(exact::whole(0)));
    t.withholding_1099 += crate::FederalForm1099K::from(data).withholding;
    for part in &data.parts {
        match part.belongs {
            Some(crate::KUse::NotIncome) => t.not_income_1099k += part.amount,
            Some(crate::KUse::PersonalGain)
                if !d.income.iter().any(|sale| {
                    sale.owner == e.owner
                        && matches!(&sale.income, FederalIncome::Sale { data }
                            if data.form.reporting == crate::BasisReporting::NoForm1099B)
                }) =>
            {
                t.injured_other_amount(e.id, None);
                note_once_scoped(
                    r,
                    Message::new(
                        "NOTE_1099K_GAIN_NEEDS_SALE",
                        alloc::vec![crate::interview::income_display_arg(e)],
                    ),
                    context.clone().corrective(crate::TaxSection::InvestmentSales),
                );
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn collect_scholarship(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::Scholarship,
) -> Result<(), CollectFailure> {
    let Source { e, owner, context, .. } = *src;
    let (taxable, compensation, ordinary) = data.taxable();
    t.scholarship += taxable;
    t.graduate_compensation[owner] += compensation;
    match ordinary {
        true => {}
        false => note_once_scoped(
            r,
            Message::new(
                "WARN_SCHOLARSHIP_SPECIAL_LEFT_OUT",
                alloc::vec![crate::interview::income_display_arg(e)],
            ),
            context.clone(),
        ),
    }
    Ok(())
}

pub(super) fn collect_jury_duty(
    _cx: &mut Collect<'_>,
    _src: &Source<'_, '_>,
    t: &mut Totals,
    _r: &mut FederalResult,
    data: &JuryPay,
) -> Result<(), CollectFailure> {
    t.jury += data.received;
    t.jury_remitted += data.remitted;
    Ok(())
}

pub(super) fn collect_alimony(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::AlimonyPayment,
) -> Result<(), CollectFailure> {
    let Source { owner, .. } = *src;
    let amount = data.checked(
        r,
        &crate::IssueContext::Section(crate::TaxSection::OtherIncome),
        false,
        #[cfg(test)]
        "Alimony received",
    );
    t.alimony[owner] += amount;
    if data.decree {
        t.alimony_compensation[owner] += amount;
    }
    let recapture = data.recaptured(
        r,
        &crate::IssueContext::Section(crate::TaxSection::OtherIncome),
        false,
        #[cfg(test)]
        "Alimony recapture (received)",
    );
    t.alimony_recapture_deduction += recapture;
    t.injured_alimony_recapture[owner] += recapture;
    Ok(())
}

pub(super) struct AlimonyOut {
    pub(super) alimony_recaptured: i64,
    pub(super) alimony_recaptured_owners: [i64; 2],
    pub(super) alimony: i64,
    pub(super) alimony_paid_cents: i64,
    pub(super) alimony_paid: i64,
}

pub(super) fn alimony_phase(pass: &Pass<'_>, r: &mut FederalResult, t: &mut Totals) -> AlimonyOut {
    let &Pass { a, j, .. } = pass;
    let mut alimony_recaptured = 0;
    let mut alimony_recaptured_owners = [0; 2];
    for paid in &a.alimony_paid {
        let owner = usize::from(paid.owner == Owner::Spouse);
        crate::require_localized!(
            r,
            owner == 0 || j,
            &crate::IssueContext::Section(crate::TaxSection::MoreAdjustments),
            "Alimony paid",
            "ISSUE_SPOUSE_PAID_ALIMONY_REQUIRES_A_JOINT_RETURN_7AD721BB",
            "Spouse-paid alimony requires a joint return.",
        );
        let recaptured = paid.payment.recaptured(
            r,
            &crate::IssueContext::Section(crate::TaxSection::MoreAdjustments),
            true,
            #[cfg(test)]
            "Alimony recapture (paid)",
        );
        alimony_recaptured += recaptured;
        alimony_recaptured_owners[owner] += recaptured;
        // Former IRC 219(f)(1) includes income under section 71 for a
        // 71(b)(2)(A) instrument; section 71(f) includes payer recapture.
        if paid.payment.decree {
            t.alimony_compensation[owner] += recaptured;
        }
    }
    let alimony = dollars(t.alimony.iter().sum::<i64>() + alimony_recaptured);
    let alimony_paid_cents = a
        .alimony_paid
        .iter()
        .map(|v| {
            v.payment.checked(
                r,
                &crate::IssueContext::Section(crate::TaxSection::MoreAdjustments),
                true,
                #[cfg(test)]
                "Alimony paid",
            )
        })
        .sum::<i64>()
        + t.alimony_recapture_deduction;
    let alimony_paid = dollars(alimony_paid_cents);
    line(
        r,
        "alimony_recapture_income",
        dollars(alimony_recaptured),
        &["adjustments.alimony_paid.recapture"],
        "https://www.irs.gov/publications/p504",
        msg!("EXPLAIN_ALIMONY_RECAPTURE_INCOME"),
    );
    line(
        r,
        "alimony_recapture_deduction",
        dollars(t.alimony_recapture_deduction),
        &["income.alimony.recapture"],
        "https://www.irs.gov/publications/p504",
        msg!("EXPLAIN_ALIMONY_RECAPTURE_DEDUCTION"),
    );
    AlimonyOut {
        alimony_recaptured,
        alimony_recaptured_owners,
        alimony,
        alimony_paid_cents,
        alimony_paid,
    }
}

pub(super) struct SelfEmployedRetirementIn<'a> {
    pub(super) half: [i64; 2],
    pub(super) compensation: [i64; 2],
    pub(super) compensation_earned_profit: [i64; 2],
    pub(super) business_half_se: &'a alloc::collections::BTreeMap<i64, i64>,
}

pub(super) struct SelfEmployedRetirementOut<'a> {
    pub(super) partner_half_attribution: Option<&'a alloc::collections::BTreeMap<i64, i64>>,
    pub(super) compensation_nonbusiness: [i64; 2],
    pub(super) sep: crate::sep::SepResult,
    pub(super) sep_deduction: i64,
    pub(super) compensation: [i64; 2],
}

pub(super) fn self_employed_retirement_phase<'map>(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    partner_pass: &PartnerPass,
    input: SelfEmployedRetirementIn<'map>,
) -> core::ops::ControlFlow<(), SelfEmployedRetirementOut<'map>> {
    let &Pass { d, context, selected_regular, office_context, .. } = pass;
    let source_claims = context.source_claims;
    let SelfEmployedRetirementIn {
        half,
        mut compensation,
        compensation_earned_profit,
        business_half_se,
    } = input;
    // Pub.517: ministerial employee wages/housing never consume a ScheduleC
    // Form7206 or SEP limit. Reuse the resolved activity shares instead of
    // reallocating the owner's entire half-SE deduction over businesses alone.
    let use_activity_half = !partner_pass.inputs.partners.is_empty()
        || match crate::clergy::calculate_with_section179_with_claims(
            d,
            Owner::Taxpayer,
            selected_regular,
            Some(office_context),
            source_claims,
        ) {
            Ok(value) => value,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        }
        .extra_se()
            != 0
        || match crate::clergy::calculate_with_section179_with_claims(
            d,
            Owner::Spouse,
            selected_regular,
            Some(office_context),
            source_claims,
        ) {
            Ok(value) => value,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        }
        .extra_se()
            != 0;
    let partner_half_attribution = use_activity_half.then_some(business_half_se);
    let compensation_nonbusiness: [i64; 2] =
        core::array::from_fn(|i| compensation[i] - positive(compensation_earned_profit[i]));
    let sep = match crate::sep::calculate_with_section179_with_claims(
        context.forms(d),
        half,
        partner_half_attribution,
        &partner_pass.inputs.partners,
        selected_regular,
        r,
        Some(office_context),
    ) {
        Ok(value) => value,
        Err(issue) => {
            crate::section179_return::Issue::from(issue).report(r);
            r.state = ResultState::Unsupported;
            return core::ops::ControlFlow::Break(());
        }
    };
    let sep_deduction = sep.owners.iter().sum::<i64>();
    for (value, deduction) in compensation.iter_mut().zip(sep.owners) {
        *value = positive(*value - deduction);
    }
    core::ops::ControlFlow::Continue(SelfEmployedRetirementOut {
        partner_half_attribution,
        compensation_nonbusiness,
        sep,
        sep_deduction,
        compensation,
    })
}

pub(super) struct AdjustmentReportsIn {
    pub(super) savings_penalty: i64,
    pub(super) alimony: i64,
    pub(super) alimony_paid: i64,
    pub(super) moving_deduction: i64,
    pub(super) moving_excess: i64,
}

pub(super) fn adjustment_reports_phase(r: &mut FederalResult, i: AdjustmentReportsIn) {
    let AdjustmentReportsIn {
        savings_penalty,
        alimony,
        alimony_paid,
        moving_deduction,
        moving_excess,
    } = i;
    line(
        r,
        "savings_withdrawal_penalty",
        savings_penalty,
        &["income.interest.early_penalty"],
        "https://www.irs.gov/instructions/i1040gi",
        msg!("EXPLAIN_SAVINGS_WITHDRAWAL_PENALTY"),
    );
    line(
        r,
        "taxable_alimony",
        alimony,
        &["income.alimony"],
        "https://www.irs.gov/taxtopics/tc452",
        msg!("EXPLAIN_TAXABLE_ALIMONY"),
    );
    line(
        r,
        "alimony_paid_deduction",
        alimony_paid,
        &["adjustments.alimony_paid"],
        "https://www.irs.gov/taxtopics/tc452",
        msg!("EXPLAIN_ALIMONY_PAID_DEDUCTION"),
    );
    line(
        r,
        "moving_expense_deduction",
        moving_deduction,
        &["adjustments.moving"],
        "https://www.irs.gov/forms-pubs/about-form-3903",
        msg!("EXPLAIN_MOVING_EXPENSE_DEDUCTION"),
    );
    line(
        r,
        "moving_excess_reimbursement",
        moving_excess,
        &["adjustments.moving", "income.wages.box12"],
        "https://www.irs.gov/forms-pubs/about-form-3903",
        msg!("EXPLAIN_MOVING_EXCESS_REIMBURSEMENT"),
    );
}

pub(super) struct Schedule1OutputIn {
    pub(super) taxable_grants: i64,
    pub(super) family_leave: i64,
    pub(super) casualty_living_insurance: i64,
    pub(super) education_income: i64,
    pub(super) trump_death: i64,
    pub(super) other_8z: i64,
    pub(super) medicaid_waiver_excluded: i64,
    pub(super) gambling: i64,
    pub(super) personal_debt: i64,
    pub(super) income_excluded: i64,
    pub(super) care_benefits: i64,
    pub(super) medicare_msa: i64,
    pub(super) archer_total_income: i64,
    pub(super) hsa_income: i64,
    pub(super) jury: i64,
    pub(super) prizes: i64,
    pub(super) hobby: i64,
    pub(super) iso_unreported: i64,
    pub(super) personal_rent: i64,
    pub(super) excess_business_loss: i64,
    pub(super) able_income: i64,
    pub(super) scholarship: i64,
    pub(super) alaska: i64,
    pub(super) olympic_prize: i64,
    pub(super) deferred_pension: i64,
    pub(super) inmate_pay: i64,
    pub(super) digital: i64,
    pub(super) not_for_profit: i64,
    pub(super) jury_remitted: i64,
    pub(super) olympic_taxable: i64,
    pub(super) personal_rent_expenses: i64,
    pub(super) housing_deduction: i64,
    pub(super) estate_67e: i64,
    pub(super) property_ordinary: i64,
    pub(super) loan_gain: i64,
    pub(super) stock_loss: i64,
    pub(super) tax_refund: i64,
    pub(super) alimony: i64,
    pub(super) msa_deduction: i64,
    pub(super) schedule_c: i64,
    pub(super) royalties: i64,
    pub(super) schedule_f: i64,
    pub(super) additional_income: i64,
    pub(super) educator: i64,
    pub(super) hsa: i64,
    pub(super) half_se: i64,
    pub(super) sep_deduction: i64,
    pub(super) savings_penalty: i64,
    pub(super) moving_deduction: i64,
    pub(super) alimony_paid: i64,
    pub(super) loan: i64,
    pub(super) adjustments: i64,
    pub(super) child_other: i64,
    pub(super) unemployment: i64,
    pub(super) nol_deduction: i64,
    pub(super) rental_income: i64,
    pub(super) health_deduction: i64,
    pub(super) ira_deduction: i64,
}

pub(super) fn schedule_1_output_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    input: Schedule1OutputIn,
) -> Result<(), crate::medical_account::OutputFailure> {
    let &Pass { pension, .. } = pass;
    let Schedule1OutputIn {
        taxable_grants,
        family_leave,
        casualty_living_insurance,
        education_income,
        trump_death,
        other_8z,
        medicaid_waiver_excluded,
        gambling,
        personal_debt,
        income_excluded,
        care_benefits,
        medicare_msa,
        archer_total_income,
        hsa_income,
        jury,
        prizes,
        hobby,
        iso_unreported,
        personal_rent,
        excess_business_loss,
        able_income,
        scholarship,
        alaska,
        olympic_prize,
        deferred_pension,
        inmate_pay,
        digital,
        not_for_profit,
        jury_remitted,
        olympic_taxable,
        personal_rent_expenses,
        housing_deduction,
        estate_67e,
        property_ordinary,
        loan_gain,
        stock_loss,
        tax_refund,
        alimony,
        msa_deduction,
        schedule_c,
        royalties,
        schedule_f,
        additional_income,
        educator,
        hsa,
        half_se,
        sep_deduction,
        savings_penalty,
        moving_deduction,
        alimony_paid,
        loan,
        adjustments,
        child_other,
        unemployment,
        nol_deduction,
        rental_income,
        health_deduction,
        ira_deduction,
    } = input;
    let other_income_8z = taxable_grants
        + family_leave
        + casualty_living_insurance
        + child_other
        + education_income
        + trump_death
        + other_8z;
    // Lines 9 and 25 are the form's own totals of the lines shown above them. Line 8b, 8h,
    // 8j, 8r and 24a rows come from their own calculations; each other term of line 10 or 26
    // is a row here, so the rows foot.
    let total_other_income = -medicaid_waiver_excluded - nol_deduction + gambling + personal_debt
        - income_excluded
        + care_benefits
        + medicare_msa
        + archer_total_income
        + hsa_income
        + jury
        + prizes
        + hobby
        + iso_unreported
        + personal_rent
        + excess_business_loss
        + able_income
        + scholarship
        + alaska
        + olympic_prize
        + deferred_pension
        + inmate_pay
        + digital
        + not_for_profit
        + other_income_8z;
    let total_other_adjustments = jury_remitted
        + (olympic_prize - olympic_taxable)
        + personal_rent_expenses
        + pension.deduction
        + housing_deduction
        + estate_67e;
    crate::k1_sales::report_ordinary_total(r, property_ordinary + loan_gain - stock_loss);
    for (reference, id, amount, explanation) in [
        ("1", "schedule_1_taxable_refunds", tax_refund, msg!("EXPLAIN_SCHEDULE_1_1")),
        ("2a", "schedule_1_alimony_received", alimony, msg!("EXPLAIN_SCHEDULE_1_2A")),
        (
            "4",
            "schedule_1_other_gains",
            property_ordinary + loan_gain - stock_loss,
            msg!("EXPLAIN_SCHEDULE_1_4"),
        ),
        ("7", "schedule_1_unemployment", dollars(unemployment), msg!("EXPLAIN_SCHEDULE_1_7")),
        ("8a", "schedule_1_net_operating_loss", -nol_deduction, msg!("EXPLAIN_SCHEDULE_1_8A")),
        ("8c", "schedule_1_canceled_debt", personal_debt, msg!("EXPLAIN_SCHEDULE_1_8C")),
        (
            "8d",
            "schedule_1_foreign_earned_income_exclusion",
            -income_excluded,
            msg!("EXPLAIN_SCHEDULE_1_8D"),
        ),
        (
            "8e",
            "schedule_1_medical_savings_income",
            {
                let dollars = exact::dollars_exact(
                    &((exact::whole(care_benefits)
                        + exact::whole(medicare_msa)
                        + exact::whole(archer_total_income))
                        * exact::whole(100)),
                );
                crate::medical_account::required_output(
                    &(&dollars * exact::whole(100)),
                    crate::IssueContext::Section(crate::TaxSection::OtherIncome)
                        .alternatives(&[crate::TaxSection::OtherIncome, crate::TaxSection::Hsa]),
                )?;
                crate::medical_account::required_output(
                    &dollars,
                    crate::IssueContext::Section(crate::TaxSection::OtherIncome)
                        .alternatives(&[crate::TaxSection::OtherIncome, crate::TaxSection::Hsa]),
                )?
            },
            msg!("EXPLAIN_SCHEDULE_1_8E"),
        ),
        ("8f", "schedule_1_hsa_income", hsa_income, msg!("EXPLAIN_SCHEDULE_1_8F")),
        ("8i", "schedule_1_prizes_and_awards", prizes, msg!("EXPLAIN_SCHEDULE_1_8I")),
        ("8k", "schedule_1_stock_options", iso_unreported, msg!("EXPLAIN_SCHEDULE_1_8K")),
        ("8l", "schedule_1_personal_property_rental", personal_rent, msg!("EXPLAIN_SCHEDULE_1_8L")),
        (
            "8p",
            "schedule_1_excess_business_loss",
            excess_business_loss,
            msg!("EXPLAIN_SCHEDULE_1_8P"),
        ),
        ("8q", "schedule_1_able_distributions", able_income, msg!("EXPLAIN_SCHEDULE_1_8Q")),
        ("23", "schedule_1_archer_msa_deduction", msa_deduction, msg!("EXPLAIN_SCHEDULE_1_23")),
        (
            "24b",
            "schedule_1_personal_property_rental_expenses",
            personal_rent_expenses,
            msg!("EXPLAIN_SCHEDULE_1_24B"),
        ),
        (
            "24f",
            "schedule_1_pension_contributions",
            pension.deduction,
            msg!("EXPLAIN_SCHEDULE_1_24F"),
        ),
        ("24j", "schedule_1_housing_deduction", housing_deduction, msg!("EXPLAIN_SCHEDULE_1_24J")),
        ("24k", "schedule_1_estate_deductions", estate_67e, msg!("EXPLAIN_SCHEDULE_1_24K")),
        ("9", "schedule_1_total_other_income", total_other_income, msg!("EXPLAIN_SCHEDULE_1_9")),
        (
            "25",
            "schedule_1_total_other_adjustments",
            total_other_adjustments,
            msg!("EXPLAIN_SCHEDULE_1_25"),
        ),
        ("3", "schedule_1_business_income", schedule_c, msg!("EXPLAIN_N")),
        ("5", "schedule_1_rental_income", rental_income + royalties, msg!("EXPLAIN_N_2")),
        ("6", "schedule_1_farm_income", schedule_f, msg!("EXPLAIN_N_3")),
        ("8z", "schedule_1_other_income", other_income_8z, msg!("EXPLAIN_8Z")),
        ("10", "additional_income", additional_income, msg!("EXPLAIN_N_4")),
        ("11", "schedule_1_educator_expenses", educator, msg!("EXPLAIN_N_5")),
        ("13", "schedule_1_hsa_deduction", dollars(hsa), msg!("EXPLAIN_N_6")),
        ("15", "schedule_1_half_self_employment_tax", half_se, msg!("EXPLAIN_N_7")),
        ("16", "schedule_1_self_employed_retirement", sep_deduction, msg!("EXPLAIN_N_8")),
        ("17", "schedule_1_self_employed_health_insurance", health_deduction, msg!("EXPLAIN_N_9")),
        ("18", "schedule_1_early_withdrawal_penalty", savings_penalty, msg!("EXPLAIN_N_10")),
        ("14", "schedule_1_moving_expenses", moving_deduction, msg!("EXPLAIN_SCHEDULE_1_14")),
        ("19a", "schedule_1_alimony_paid", alimony_paid, msg!("EXPLAIN_19A")),
        ("20", "schedule_1_ira_deduction", ira_deduction, msg!("EXPLAIN_N_11")),
        ("21", "schedule_1_student_loan_interest", loan, msg!("EXPLAIN_N_12")),
        ("26", "schedule_1_adjustments", adjustments, msg!("EXPLAIN_N_13")),
    ] {
        if amount != 0 {
            form_line(
                r,
                FormLine::new(TaxForm::Schedule1, reference),
                id,
                amount,
                &["income", "adjustments"],
                "https://www.irs.gov/instructions/i1040gi",
                explanation,
            );
        }
    }

    Ok(())
}
