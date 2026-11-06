// form_8889 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
// One Form 1099-SA's amounts onto the totals and the return lines: Form 8853 Section A for
// an Archer MSA, Section B for a Medicare Advantage MSA, and the death-of-the-holder income
// of an inherited account (Form 8889 lines 14a-16 for an HSA).
pub(super) fn medical_distribution(
    r: &mut FederalResult,
    t: &mut Totals,
    e: &FederalIncomeEntry,
    context: &crate::IssueContext,
    data: &crate::FederalMedicalDistribution,
    amounts: &crate::medical_account::SaAmounts,
    owner: usize,
) -> Result<(), crate::medical_account::OutputFailure> {
    use crate::{MedicalSaAccount as A, MedicalSaPayment as P};
    let id = e.id;
    let payment = data.effective_payment().unwrap();
    let native = |amount: &Exact| crate::medical_account::required_output(amount, context.clone());
    if matches!(data.account, A::Hsa) && !matches!(payment, P::Inherited { .. }) {
        let totals = &mut t.hsa_distributions[owner];
        totals.gross += exact::whole(data.gross);
        totals.taxable += &amounts.taxable;
        totals.penalized += &amounts.penalized;
        match payment {
            P::Own { medical, rollover, .. } => {
                totals.medical += exact::whole(*medical);
                totals.rollover +=
                    rollover.as_ref().map_or_else(|| exact::whole(0), |v| v.amount());
            }
            P::Unreported { event: crate::MedicalDeemed::Pledged { .. }, .. } => {
                totals.deemed += &amounts.taxable;
                totals.pledged += &amounts.taxable;
                totals.pledged_penalized += &amounts.penalized;
            }
            P::Prohibited { .. } | P::Unreported { .. } => totals.deemed += &amounts.taxable,
            P::ExcessReturned => totals.returned_excess += exact::whole(data.gross),
            _ => {}
        }
        return Ok(());
    }
    match payment {
        P::Own { medical, rollover, .. } => {
            let rolled = rollover.as_ref().map_or_else(|| exact::whole(0), |v| v.amount());
            if let Some(form) = data.medicare_form(id, owner, context.clone(), amounts) {
                line_cents(
                    r,
                    &alloc::format!("medicare_msa_{id}_taxable"),
                    native(&amounts.taxable)?,
                    &["income.medical_distribution"],
                    "https://www.irs.gov/instructions/i8853",
                    msg!("EXPLAIN_MEDICARE_MSA_N_N"),
                );
                t.medicare_forms.push(form);
            } else {
                t.archer_msa += &amounts.taxable;
                t.archer_msa_owner[owner] += &amounts.taxable;
                t.archer_penalized += &amounts.penalized;
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("gross", data.gross),
                        ("medical", *medical),
                        ("rollover", native(&rolled)?),
                        ("taxable", native(&amounts.taxable)?),
                        ("penalty_base", native(&amounts.penalized)?),
                    ],
                    |(suffix, amount), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!("archer_msa_{id}_{suffix}"),
                            LineValue::MoneyCents(amount),
                            &["income.medical_distribution"],
                            "https://www.irs.gov/instructions/i8853",
                            msg!("EXPLAIN_ARCHER_MSA_N_N"),
                        ))
                    },
                );
            }
        }
        P::Prohibited { .. } | P::Unreported { .. } => {
            if let Some(form) = data.medicare_form(id, owner, context.clone(), amounts) {
                emit_lines(
                    r,
                    &mut (),
                    [("taxable", native(&amounts.taxable)?), ("deemed", data.gross)],
                    |(suffix, amount), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!("medicare_msa_{id}_{suffix}"),
                            LineValue::MoneyCents(amount),
                            &["income.medical_distribution"],
                            "https://www.irs.gov/instructions/i8853",
                            msg!("EXPLAIN_MEDICARE_MSA_N_N"),
                        ))
                    },
                );
                t.medicare_forms.push(form);
            } else {
                t.archer_msa += &amounts.taxable;
                t.archer_msa_owner[owner] += &amounts.taxable;
                t.archer_penalized += &amounts.penalized;
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("gross", data.gross),
                        ("deemed", data.gross),
                        ("taxable", native(&amounts.taxable)?),
                        ("penalty_base", native(&amounts.penalized)?),
                    ],
                    |(suffix, amount), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!("archer_msa_{id}_{suffix}"),
                            LineValue::MoneyCents(amount),
                            &["income.medical_distribution"],
                            "https://www.irs.gov/instructions/i8853",
                            msg!("EXPLAIN_ARCHER_MSA_N_N"),
                        ))
                    },
                );
            }
        }
        P::ExcessReturned => {
            t.archer_excess_earnings += &amounts.earnings;
            line_cents(
                r,
                &alloc::format!("archer_msa_{id}_excess_earnings"),
                native(&amounts.earnings)?,
                &["income.medical_distribution"],
                "https://www.irs.gov/pub/irs-pdf/f1099sa.pdf",
                msg!("EXPLAIN_ARCHER_MSA_N_EXCESS_EARNINGS"),
            );
        }
        P::Inherited { died, beneficiary } => {
            let value = &data.box4_value.unwrap();
            let slot = match data.account {
                A::Hsa => 0,
                A::MedicareAdvantage => 2,
                A::Archer => 1,
            };
            t.inherited_medical[slot] += &amounts.inherited;
            t.inherited_medical_earnings += &amounts.earnings;
            let source = if slot == 0 {
                "https://www.irs.gov/instructions/i8889"
            } else {
                "https://www.irs.gov/instructions/i8853"
            };
            for (suffix, amount, whole) in [
                ("value", exact::whole(dollars(*value)), true),
                ("medical", amounts.inherited_medical.clone(), true),
                ("taxable", amounts.inherited.clone(), true),
                ("earnings", amounts.earnings.clone(), false),
            ] {
                let id_text = alloc::format!("inherited_medical_{id}_{suffix}");
                if whole {
                    let cents = &amount * exact::whole(100);
                    if suffix == "taxable" {
                        native(&cents)?;
                    } else if crate::medical_account_return::explanation_output(
                        &cents,
                        context.clone(),
                        r,
                    )
                    .is_none()
                    {
                        continue;
                    }
                    line(
                        r,
                        &id_text,
                        native(&amount)?,
                        &["income.medical_distribution"],
                        source,
                        msg!("EXPLAIN_INHERITED_MEDICAL_N_N"),
                    );
                } else {
                    line_cents(
                        r,
                        &id_text,
                        native(&amount)?,
                        &["income.medical_distribution"],
                        source,
                        msg!("EXPLAIN_INHERITED_MEDICAL_N_N"),
                    );
                }
            }
            if died / 10000 < 2026 {
                r.review_notes.push(ReviewNote::localized(
                    #[cfg(test)] alloc::format!("Inherited medical account — {}: death-date value and qualifying medical offsets belong to the {} return, including any required amendment. They are not included or deducted again in 2026. Earnings after the death are calculated in this entry.", e.label, died / 10000),
                    Message::new("NOTE_INHERITED_MEDICAL_PRIOR_YEAR", alloc::vec![
                        crate::interview::income_display_arg(e), MessageArg::Number((died / 10000) as i64),
                    ]),
                ));
            } else {
                r.review_notes.push(ReviewNote::localized(
                    #[cfg(test)] alloc::format!("Inherited medical account — {}: retain an account-specific death statement and medical payment records. Payments within one year may change this death-year calculation; earnings after the death are the part of box 1 above box 4.", e.label),
                    Message::new("NOTE_INHERITED_MEDICAL_DEATH_RECORDS", alloc::vec![crate::interview::income_display_arg(e)]),
                ));
            }
            if amounts.decline {
                r.review_notes.push(ReviewNote::localized(
                    #[cfg(test)] alloc::format!("Inherited medical account — {}: the account paid out less than its value at the death. No deduction is available for the decline; only earnings above that value would be income.", e.label),
                    Message::new("NOTE_INHERITED_MEDICAL_DECLINE", alloc::vec![crate::interview::income_display_arg(e)]),
                ));
            }
            let _ = beneficiary;
        }
    }
    Ok(())
}

pub(super) fn collect_medical_distribution(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::SaSource,
) -> Result<(), CollectFailure> {
    let Collect { d, medical_projection, .. } = *cx;
    let Source {
        e,
        owner,
        context,
        #[cfg(test)]
        source,
        ..
    } = *src;
    let data = medical_projection
        .draft
        .as_ref()
        .and_then(|draft| {
            draft.income.iter().find_map(|source| match &source.income {
                FederalIncome::MedicalDistribution { data } if source.id == e.id => Some(data),
                _ => None,
            })
        })
        .unwrap_or(data);
    let data = &crate::FederalMedicalDistribution::from(data);
    let person = d
        .people
        .as_ref()
        .and_then(|p| if owner == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() });
    if let Some(person) = person {
        match data.amounts(person) {
            Ok(mut amounts) => {
                if matches!(data.account, crate::MedicalSaAccount::Archer)
                    && matches!(
                        data.effective_payment(),
                        Some(crate::MedicalSaPayment::ExcessReturned)
                    )
                    && d.adjustments.as_ref().is_some_and(|a| {
                        crate::hsa_correction::archer_linked(&a.archer, e.id, e.owner.owned())
                    })
                {
                    // Its actual signed NIA is consumed by the linked
                    // contribution owner, not again from printed box2.
                    amounts.earnings = exact::whole(0);
                }
                // This is after actual linked-Archer suppression,
                // not a second raw statement calculation.
                let source =
                    amounts.source(e.id, e.owner.owned(), data.calculation_account().unwrap());
                let income = &source.taxable + &source.earnings + &source.inherited;
                t.injured_other_sources.insert(source.id, Some(income.clone()));
                t.injured_accounts.source(source.id, source.owner, income);
                t.injured_medical.push(source);
                medical_distribution(r, t, e, context, data, &amounts, owner)?;
            }
            Err((key, _message)) => {
                crate::issue_localized!(r, &context, &source, key, _message,)
            }
        }
    }
    Ok(())
}

pub(super) struct HsaOut {
    pub(super) hsa: i64,
    pub(super) hsa_income: i64,
    pub(super) hsa_tax: i64,
    pub(super) hsa_distribution_tax: i64,
    pub(super) hsa_chapter_one_tax: i64,
    pub(super) hsa_testing_tax: i64,
    pub(super) hsa_excess_tax: i64,
    pub(super) hsa_work_income: Exact,
    pub(super) hsa_deduction_by_owner: [i64; 2],
}

pub(super) fn hsa_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
) -> core::ops::ControlFlow<(), HsaOut> {
    let &Pass { d, a, p, s, context, .. } = pass;
    let crate::hsa::CalculationResult {
        sources: hsa_sources,
        deduction: hsa,
        income: hsa_income,
        tax: hsa_tax,
        recapture: hsa_recapture,
        correction_income: hsa_correction_income,
        correction_employer_income: hsa_correction_employer_principal_income,
        excess_employer_income: hsa_excess_employer_other_income,
        excess_tax: hsa_excess_tax,
        last_month_income: hsa_last_month,
        testing_tax: hsa_testing_tax,
        work_income_cents: hsa_work_income,
        deduction_by_owner: hsa_deduction_by_owner,
    } = match crate::hsa::calculate(
        &a.hsa,
        p,
        s.filing_status(),
        &crate::hsa::ContributionContext {
            statements: &crate::contribution_documents::project(d).unwrap_or_default().hsa,
            msa: {
                let deposits = a.archer.deposits();
                [0, 1].map(|i| &deposits[i] + &context.medical.contributions.archer[i].personal)
            },
            failed: &context.medical.contributions.hsa,
            married: s.marital_state.married(),
            family_share: a.hsa_family_share.as_ref(),
        },
        &t.hsa_distributions,
        &d.income,
        r,
    ) {
        Ok(amounts) => amounts,
        Err(issue) => {
            issue.report(r);
            return core::ops::ControlFlow::Break(());
        }
    };
    t.injured_accounts.fixed_income([0, 1].map(|i| {
        hsa_sources.distribution_income[i]
            + hsa_sources.funding_income[i]
            + hsa_sources.last_month_income[i]
    }));
    t.injured_accounts.income(hsa_correction_income, hsa_sources.correction_income);
    t.injured_accounts
        .income(hsa_correction_employer_principal_income, hsa_sources.correction_employer_income);
    t.injured_accounts.income(hsa_excess_employer_other_income, hsa_sources.excess_employer_income);
    t.injured_accounts.fixed_tax([0, 1].map(|i| {
        hsa_sources.distribution_tax[i] + hsa_sources.testing_tax[i] + hsa_sources.excess_tax[i]
    }));
    let hsa_income = {
        let dollars = exact::dollars_exact(
            &((exact::whole(hsa_income) + &t.inherited_medical[0]) * exact::whole(100)),
        );
        match crate::medical_account::required_output(
            &(&dollars * exact::whole(100)),
            crate::IssueContext::Section(crate::TaxSection::OtherIncome)
                .alternatives(&[crate::TaxSection::OtherIncome, crate::TaxSection::Hsa]),
        ) {
            Ok(amount) => amount,
            Err(issue) => {
                issue.report(r);
                return core::ops::ControlFlow::Break(());
            }
        };
        match crate::medical_account::required_output(
            &dollars,
            crate::IssueContext::Section(crate::TaxSection::OtherIncome)
                .alternatives(&[crate::TaxSection::OtherIncome, crate::TaxSection::Hsa]),
        ) {
            Ok(amount) => amount,
            Err(issue) => {
                issue.report(r);
                return core::ops::ControlFlow::Break(());
            }
        }
    };
    line(
        r,
        "hsa_taxable_distributions",
        hsa_income,
        &["adjustments.hsa"],
        "https://www.irs.gov/instructions/i8889",
        msg!("EXPLAIN_HSA_TAXABLE_DISTRIBUTIONS"),
    );
    line(
        r,
        "hsa_additional_tax",
        hsa_tax,
        &["adjustments.hsa"],
        "https://www.irs.gov/instructions/i8889",
        msg!("EXPLAIN_HSA_ADDITIONAL_TAX"),
    );
    emit_lines(
        r,
        &mut (),
        [
            ("hsa_funding_recapture", hsa_recapture, msg!("EXPLAIN_HSA_FUNDING_RECAPTURE")),
            ("hsa_testing_period_tax", hsa_testing_tax, msg!("EXPLAIN_HSA_TESTING_PERIOD_TAX")),
        ],
        |(id, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &[
                    "adjustments.hsa.funding_history",
                    "adjustments.hsa.last_month_history",
                    "adjustments.hsa.last_month_loss",
                    "adjustments.hsa_family_share",
                ],
                "https://www.irs.gov/instructions/i8889",
                explanation,
            ))
        },
    );
    line(
        r,
        "hsa_correction_earnings",
        hsa_correction_income,
        &["adjustments.hsa.corrections"],
        "https://www.irs.gov/instructions/i8889",
        msg!("EXPLAIN_HSA_CORRECTION_EARNINGS"),
    );
    line(
        r,
        "hsa_correction_employer_principal_other_income",
        hsa_correction_employer_principal_income,
        &["adjustments.hsa.corrections"],
        "https://www.irs.gov/instructions/i8889",
        msg!("EXPLAIN_HSA_CORRECTION_EMPLOYER_PRINCIPAL_OTHER_INCOME"),
    );
    line(
        r,
        "hsa_return_year_excess_contribution_tax",
        hsa_excess_tax,
        &["adjustments.hsa.excess_accounting"],
        "https://www.irs.gov/pub/irs-dft/f5329--dft.pdf",
        msg!("EXPLAIN_HSA_N_EXCESS_CONTRIBUTION_TAX"),
    );
    line(
        r,
        "hsa_return_year_uncorrected_employer_excess_other_income",
        hsa_excess_employer_other_income,
        &[
            "adjustments.hsa.employer",
            "adjustments.hsa.excess_accounting",
            "adjustments.hsa.prior_employer_excess",
        ],
        "https://www.irs.gov/instructions/i8889",
        msg!("EXPLAIN_HSA_N_UNCORRECTED_EMPLOYER_EXCESS_OTHER_INCOME"),
    );
    line(
        r,
        "hsa_last_month_recapture",
        hsa_last_month,
        &[
            "adjustments.hsa.last_month_history",
            "adjustments.hsa.last_month_loss",
            "adjustments.hsa.coverage",
        ],
        "https://www.irs.gov/irb/2008-25_IRB",
        msg!("EXPLAIN_HSA_LAST_MONTH_RECAPTURE"),
    );
    let hsa_income = {
        let dollars = exact::dollars_exact(
            &(exact::whole(
                hsa_income
                    + hsa_recapture
                    + hsa_correction_income
                    + hsa_correction_employer_principal_income
                    + hsa_excess_employer_other_income
                    + hsa_last_month,
            ) * exact::whole(100)),
        );
        match crate::medical_account::required_output(
            &(&dollars * exact::whole(100)),
            crate::IssueContext::Section(crate::TaxSection::OtherIncome)
                .alternatives(&[crate::TaxSection::OtherIncome, crate::TaxSection::Hsa]),
        ) {
            Ok(amount) => amount,
            Err(issue) => {
                issue.report(r);
                return core::ops::ControlFlow::Break(());
            }
        };
        match crate::medical_account::required_output(
            &dollars,
            crate::IssueContext::Section(crate::TaxSection::OtherIncome)
                .alternatives(&[crate::TaxSection::OtherIncome, crate::TaxSection::Hsa]),
        ) {
            Ok(amount) => amount,
            Err(issue) => {
                issue.report(r);
                return core::ops::ControlFlow::Break(());
            }
        }
    };
    // Section 223 withdrawal and testing-period additions increase Chapter 1
    // tax; section 4973 excess-contribution tax is a Chapter 43 excise tax.
    let hsa_distribution_tax = hsa_tax;
    let hsa_chapter_one_tax = hsa_tax + hsa_testing_tax;
    let hsa_tax = hsa_chapter_one_tax + hsa_excess_tax;
    core::ops::ControlFlow::Continue(HsaOut {
        hsa,
        hsa_income,
        hsa_tax,
        hsa_distribution_tax,
        hsa_chapter_one_tax,
        hsa_testing_tax,
        hsa_excess_tax,
        hsa_work_income,
        hsa_deduction_by_owner,
    })
}
