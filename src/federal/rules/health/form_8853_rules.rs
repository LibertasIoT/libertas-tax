// form_8853 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_ltc_benefits(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::LtcSource,
    ltc_reference_rule: Option<&'static str>,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source {
        e,
        owner,
        context,
        #[cfg(test)]
        source,
        ..
    } = *src;
    t.interest += data.record().interest;
    if let Some(key) = data.record().character_rule(d, e.owner.owned()).or(ltc_reference_rule) {
        crate::issue_localized!(
            r,
            &context,
            &source,
            key,
            "This LTC receipt needs its actual source character and independently established taxable treatment before a complete estimate is available.",
        );
        return Ok(());
    }
    // Freeze only after the native source/linked-character check.
    // Benefits retain their separate care or linked-annuity owner.
    t.injured_ltc_nii.insert(e.id, data.record().interest);
    if let Some((taxable, excluded)) = data.record().accident_health() {
        t.care_benefits += taxable;
        let mut receipts = [0; 2];
        receipts[owner] = taxable;
        t.injured_ltc.push(crate::care_benefit_return::InjuredLtcPart {
            key: alloc::format!("ltc_{}_nonqualified", e.id),
            taxable,
            receipts: Some(receipts),
        });
        emit_lines(
            r,
            &mut (),
            [("nonqualified_taxable", taxable), ("nonqualified_excluded", excluded)],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("ltc_benefit_{}_{suffix}", e.id),
                    LineValue::MoneyCents(amount),
                    &["income.ltc_benefits"],
                    "https://www.ecfr.gov/current/title-26/section-1.105-1",
                    msg!("EXPLAIN_LTC_NONQUALIFIED_TREATMENT"),
                ))
            },
        );
    }
    if matches!(
        data.record().effective_contract(),
        crate::LtcContract::Nonqualified {
            treatment: crate::NonqualifiedCareTreatment::LifeAnnuity { .. }
        }
    ) && data.record().benefits > 0
    {
        note_once_scoped(r, msg!("WARN_LTC_ANNUITY_EXCLUSION_UNAVAILABLE"), context.clone());
    }
    let context = context.clone().manual();
    match crate::care_benefit_return::link_of(d, e) {
        // No periods record to place the per diem payments: they are
        // taxed in full (the notice is in unknowns.rs).
        Some(crate::care_benefit_return::Link::Missing) => {
            let taxable = data.record().periodic();
            t.care_benefits += taxable;
            let mut receipts = [0; 2];
            receipts[owner] = taxable;
            t.injured_ltc.push(crate::care_benefit_return::InjuredLtcPart {
                key: alloc::format!("ltc_{}_unlinked", e.id),
                taxable,
                receipts: Some(receipts),
            });
        }
        Some(crate::care_benefit_return::Link::OtherOwner) => {
            crate::require_localized!(
                r,
                false,
                &context,
                &source,
                "ISSUE_LTC_PERIODS_OTHER_OWNER",
                "This 1099-LTC names a Long-term care periods record that belongs to the other filer. Choose one of this filer's own.",
            )
        }
        _ => {}
    }
    if data.record().excluded() > 0 {
        line_cents(
            r,
            &alloc::format!("ltc_benefit_{}_exempt", e.id),
            data.record().excluded(),
            &["income.ltc_benefits"],
            "https://www.irs.gov/instructions/i8853",
            msg!("EXPLAIN_LTC_BENEFIT_N_EXEMPT"),
        );
    }
    Ok(())
}

pub(super) fn collect_ltc_insured(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::FederalLtcInsured,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source {
        e,
        context,
        #[cfg(test)]
        source,
        ..
    } = *src;
    let context = context.clone().manual();
    let forms = crate::care_benefit_return::payments(d, e.id);
    let received: i64 = forms.iter().map(|(_, amount)| amount).sum();
    match data.rows(received) {
        Some(periods) => {
            t.injured_ltc_nii.insert(e.id, 0);
            for (index, period) in periods.iter().enumerate() {
                let (limit, allocated_limit, excluded, taxable) = period.amounts();
                t.care_benefits += taxable;
                t.injured_ltc.push(crate::care_benefit_return::injured_period(
                    e.id,
                    index,
                    period,
                    &forms,
                    periods.len() == 1,
                    taxable,
                ));
                emit_lines(
                    r,
                    &mut (),
                    [
                        ("received", period.received),
                        ("limit", limit),
                        ("allocated_limit", allocated_limit),
                        ("excluded", excluded),
                        ("taxable", taxable),
                    ],
                    |(suffix, amount), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!("ltc_benefit_{}_{}_{suffix}", e.id, index + 1),
                            LineValue::MoneyCents(amount),
                            &["income.ltc_benefits"],
                            "https://www.irs.gov/instructions/i8853",
                            msg!("EXPLAIN_LTC_BENEFIT_N_N_N"),
                        ))
                    },
                );
            }
        }
        None => crate::require_localized!(
            r,
            false,
            &context,
            &source,
            "ISSUE_LTC_PERIOD_PAYMENTS_DIFFER",
            "The payments of the periods do not total the per diem payments on the 1099-LTCs that name this record. Make each period's payments add up to them.",
        ),
    }
    Ok(())
}

pub(super) struct ArcherMsaIn<'a> {
    pub(super) half: [i64; 2],
    pub(super) optional_weights: &'a alloc::collections::BTreeMap<i64, i64>,
    pub(super) archer_msa: i64,
}

pub(super) struct ArcherMsaOut {
    pub(super) msa_deduction_cents: Exact,
    pub(super) archer_excess_tax: i64,
    pub(super) archer_employer_income_cents: Exact,
    pub(super) archer_correction_earnings_cents: Exact,
    pub(super) archer_correction_employer_cents: Exact,
    pub(super) archer_prior_employer_cents: Exact,
    pub(super) msa_deduction: i64,
    pub(super) archer_total_income: i64,
}

pub(super) fn archer_msa_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
    i: ArcherMsaIn<'_>,
) -> core::ops::ControlFlow<(), ArcherMsaOut> {
    let &Pass { d, selected_regular, office_context, context, .. } = pass;
    let source_claims = context.source_claims;
    let ArcherMsaIn { half, optional_weights, archer_msa } = i;
    let (
        msa_deduction_cents,
        archer_excess_tax,
        archer_employer_income_cents,
        archer_correction_earnings_cents,
        archer_correction_employer_cents,
        archer_prior_employer_cents,
        archer_sources,
    ) = match crate::archer_contributions::calculate_with_section179_with_claims(
        d,
        half,
        optional_weights,
        t.archer_msa_owner.clone(),
        selected_regular,
        &context.medical.contributions.archer,
        r,
        Some(office_context),
        source_claims,
    ) {
        Ok(amounts) => amounts,
        Err(crate::archer_contributions::CalculationIssue::Section179(issue)) => {
            crate::section179_return::Issue::from(issue).report(r);
            r.state = ResultState::Unsupported;
            return core::ops::ControlFlow::Break(());
        }
        Err(crate::archer_contributions::CalculationIssue::Output(issue)) => {
            issue.report(r);
            return core::ops::ControlFlow::Break(());
        }
    };
    let msa_deduction = {
        let dollars = exact::dollars_exact(&msa_deduction_cents);
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
    t.injured_accounts.msa_deduction(msa_deduction, archer_sources.deduction);
    t.injured_accounts.fixed_tax(archer_sources.excess_tax);
    let archer_contribution_income = {
        let dollars = exact::dollars_exact(
            &(&archer_employer_income_cents
                + &archer_correction_earnings_cents
                + &archer_correction_employer_cents
                + &archer_prior_employer_cents),
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
    t.injured_accounts.income(
        archer_contribution_income,
        [0, 1].map(|i| archer_sources.income.iter().map(|group| &group[i]).sum()),
    );
    let archer_total_income = {
        let dollars = exact::dollars_exact(
            &((exact::whole(archer_msa) + exact::whole(archer_contribution_income))
                * exact::whole(100)),
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
    core::ops::ControlFlow::Continue(ArcherMsaOut {
        msa_deduction_cents,
        archer_excess_tax,
        archer_employer_income_cents,
        archer_correction_earnings_cents,
        archer_correction_employer_cents,
        archer_prior_employer_cents,
        msa_deduction,
        archer_total_income,
    })
}

pub(super) struct MedicalDistributionsOut {
    pub(super) archer_msa: i64,
    pub(super) archer_msa_tax: i64,
    pub(super) medicare_msa: i64,
    pub(super) medicare_msa_tax: i64,
    pub(super) medical_earnings: i64,
}

pub(super) fn medical_distributions_phase(
    r: &mut FederalResult,
    t: &mut Totals,
) -> core::ops::ControlFlow<(), MedicalDistributionsOut> {
    let archer_msa_distribution = {
        let dollars = exact::dollars_exact(&t.archer_msa);
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
    let archer_msa = {
        let dollars = exact::dollars_exact(
            &((exact::whole(archer_msa_distribution) + &t.inherited_medical[1])
                * exact::whole(100)),
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
    let archer_msa_tax = {
        let dollars = exact::dollars_exact(
            &(exact::rounded_exact(&(exact::dollars_exact(&t.archer_penalized) / exact::whole(5)))
                * exact::whole(100)),
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
        "archer_msa_additional_tax",
        archer_msa_tax,
        &["income.archer_msa"],
        "https://www.irs.gov/instructions/i8853",
        msg!("EXPLAIN_ARCHER_MSA_ADDITIONAL_TAX"),
    );
    line(
        r,
        "archer_msa_taxable_distributions",
        archer_msa,
        &["income.medical_inheritance"],
        "https://www.irs.gov/instructions/i8853",
        msg!("EXPLAIN_ARCHER_MSA_TAXABLE_DISTRIBUTIONS"),
    );
    let (medicare_income_owners, medicare_tax_owners) =
        match crate::medical_account_return::medicare_totals(&t.medicare_forms, r) {
            Ok(amounts) => amounts,
            Err(issue) => {
                issue.report(r);
                return core::ops::ControlFlow::Break(());
            }
        };
    let medicare_msa_distributions =
        medicare_income_owners.iter().map(|v| exact::whole(*v)).sum::<Exact>();
    let medicare_msa_tax = {
        let dollars = exact::dollars_exact(
            &(medicare_tax_owners.iter().map(|v| exact::whole(*v)).sum::<Exact>()
                * exact::whole(100)),
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
    t.injured_accounts.fixed_income(medicare_income_owners);
    t.injured_accounts.fixed_tax(medicare_tax_owners);
    let mut archer_sources: [Exact; 2] = Default::default();
    let mut archer_penalized: [Exact; 2] = Default::default();
    let mut medical_earnings_sources: [Exact; 2] = Default::default();
    let mut inherited_sources: [Exact; 2] = Default::default();
    for source in &t.injured_medical {
        let owner = usize::from(source.owner == Owner::Spouse);
        if matches!(source.account, crate::MedicalSaAccount::Archer) {
            archer_sources[owner] += &source.taxable;
            archer_penalized[owner] += &source.penalized;
        }
        medical_earnings_sources[owner] += &source.earnings;
        // Native inheritance is already rounded whole dollars at its source.
        inherited_sources[owner] += exact::dollars_exact(&source.inherited);
    }
    t.injured_accounts.income(archer_msa_distribution, archer_sources);
    t.injured_accounts.archer_tax(archer_msa_tax, archer_penalized);
    t.injured_accounts.fixed_income([
        match crate::medical_account::required_output(
            &inherited_sources[0].clone(),
            crate::IssueContext::Section(crate::TaxSection::OtherIncome)
                .alternatives(&[crate::TaxSection::OtherIncome, crate::TaxSection::Hsa]),
        ) {
            Ok(amount) => amount,
            Err(issue) => {
                issue.report(r);
                return core::ops::ControlFlow::Break(());
            }
        },
        match crate::medical_account::required_output(
            &inherited_sources[1].clone(),
            crate::IssueContext::Section(crate::TaxSection::OtherIncome)
                .alternatives(&[crate::TaxSection::OtherIncome, crate::TaxSection::Hsa]),
        ) {
            Ok(amount) => amount,
            Err(issue) => {
                issue.report(r);
                return core::ops::ControlFlow::Break(());
            }
        },
    ]);
    let medicare_msa = {
        let dollars = exact::dollars_exact(
            &((medicare_msa_distributions + &t.inherited_medical[2]) * exact::whole(100)),
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
    emit_lines(
        r,
        &mut (),
        [
            ("medicare_msa_taxable_distributions", medicare_msa),
            ("medicare_msa_additional_tax", medicare_msa_tax),
        ],
        |(id, amount), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["income.medical_distribution"],
                "https://www.irs.gov/instructions/i8853",
                msg!("EXPLAIN_FEDERAL_RULES_2"),
            ))
        },
    );
    // Schedule 1 line 8z: earnings after an inherited account holder's death and a
    // returned Archer MSA excess (1099-SA recipient instructions).
    let medical_earnings = {
        let dollars =
            exact::dollars_exact(&(&t.inherited_medical_earnings + &t.archer_excess_earnings));
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
    t.injured_accounts.income(medical_earnings, medical_earnings_sources);
    if let Some(inherited_medical_earnings) = crate::medical_account_return::explanation_output(
        &t.inherited_medical_earnings,
        crate::IssueContext::Section(crate::TaxSection::OtherIncome),
        r,
    ) {
        line_cents(
            r,
            "inherited_medical_earnings",
            inherited_medical_earnings,
            &["income.medical_distribution"],
            "https://www.irs.gov/pub/irs-pdf/f1099sa.pdf",
            msg!("EXPLAIN_INHERITED_MEDICAL_EARNINGS"),
        );
    }
    core::ops::ControlFlow::Continue(MedicalDistributionsOut {
        archer_msa,
        archer_msa_tax,
        medicare_msa,
        medicare_msa_tax,
        medical_earnings,
    })
}
