// form_1099_r items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_inherited_ira(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::FederalInheritedIra,
) -> Result<(), CollectFailure> {
    let Collect { d, qcd, basis, .. } = *cx;
    let Source { e, owner, .. } = *src;
    // The payments that name this record are one group: its
    // gross and withholding come from them, its basis and estate
    // tax from the record.
    let payments = crate::inherited_ira::payments(d, e.id);
    let gross = payments.iter().map(|(_, gross, _)| gross).sum::<i64>();
    let withholding = payments.iter().map(|(_, _, held)| held).sum::<i64>();
    t.native_withholding_sources
        .as_mut()
        .unwrap()
        .inherited_payment_ids
        .insert(e.id, payments.iter().map(|(entry, _, _)| entry.id).collect());
    let taxable = if data.roth() {
        basis.roth.taxable.get(&e.id).copied().unwrap_or(0)
    } else {
        *basis
            .inherited
            .taxable
            .get(&e.id)
            .expect("traditional inherited group calculated before collection")
    };
    if let Some(estate) = data.estate_tax() {
        t.inherited_estate_tax += estate.deduction(
            r,
            e.id,
            qcd.exclusion_for_entry(e.id).map(|exclusion| taxable - exclusion),
        );
    }
    t.retirement += taxable;
    t.injured_retirement[0][owner] += taxable;
    t.retirement_gross[0] += gross;
    t.retirement_taxable[0] += taxable;
    t.withholding_1099 += withholding;
    Ok(())
}

pub(super) fn collect_roth(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    stored: &crate::R1099Source,
) -> Result<(), CollectFailure> {
    let Collect { d, basis, .. } = *cx;
    let Source { e, owner, context, .. } = *src;
    let crate::RothWithdrawal { gross, withholding, inherited, .. } =
        &crate::RothWithdrawal::from(stored);
    // An inherited withdrawal is counted with its record's group;
    // with no record to use, the whole withdrawal is taxed.
    let taxable = match crate::inherited_ira::link_of(d, e) {
        Some(crate::inherited_ira::Link::Record(_)) => return Ok(()),
        Some(link) => {
            crate::inherited_ira::link_issue(r, context, link);
            *gross
        }
        None => {
            debug_assert!(*inherited == crate::InheritedLink::No);
            basis.roth.taxable.get(&e.id).copied().unwrap_or(0)
        }
    };
    t.retirement += taxable;
    t.injured_retirement[0][owner] += taxable;
    t.retirement_gross[0] += gross;
    t.retirement_taxable[0] += taxable;
    t.withholding_1099 += withholding;
    Ok(())
}

pub(super) fn collect_retirement(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    stored: &crate::RetirementSource,
) -> Result<(), CollectFailure> {
    let Collect { d, basis, .. } = *cx;
    let Source {
        e,
        owner,
        context,
        #[cfg(test)]
        source,
        ..
    } = *src;
    let v = &crate::FederalRetirement::from(stored);
    // An inherited IRA payment is taxed with its record's group
    if v.taxable.is_none() || v.ordinary_distribution.is_none() {
        return Ok(());
    }
    // (the InheritedIra arm); with no record to use, the whole
    // payment is taxed as the payer printed it, box 2a
    // (WARN_INHERITED_IRA_RECORD_MISSING).
    if let Some(crate::RetirementTaxable::Inherited { .. }) = v.taxable {
        let payer = v.box_2a;
        match crate::inherited_ira::link_of(d, e) {
            Some(crate::inherited_ira::Link::Record(_)) => {}
            Some(link) => {
                crate::inherited_ira::link_issue(r, context, link);
                t.retirement += payer;
                t.injured_retirement[0][owner] += payer;
                t.retirement_gross[0] += v.gross;
                t.retirement_taxable[0] += payer;
                t.withholding_1099 += v.withholding;
            }
            None => {}
        }
        return Ok(());
    }
    if let Some(crate::RetirementTaxable::Railroad { .. }) = v.taxable {
        let ordinary = v.ordinary_distribution == Some(RetirementTreatment::Ordinary)
            && d.people
                .as_ref()
                .and_then(|p| if owner == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() })
                .is_some_and(|p| p.attained(60, false));
        let survivor = matches!(&v.ordinary_distribution,
            Some(RetirementTreatment::Exception {
                distributed, facts: crate::RetirementException::Death { died },
                second: None,
                early_simple: false,
            }) if died <= distributed);
        crate::require_localized!(
            r,
            (ordinary || survivor) && v.kind == RetirementKind::Pension,
            &context,
            &source,
            "ISSUE_CONFIRM_ORDINARY_AGE_60_OR_LATER_RRB_PENSION_TREATMENT_OR_DOCUMENTE_15AB4CB8",
            "Confirm ordinary age-60-or-later RRB pension treatment or documented survivor payments after death. Unresolved repayments, disability-wage routing, unresolved shared employee contributions and other adjustments require additional worksheets.",
        );
    }
    if let Some(crate::RetirementTaxable::CivilServiceDeath {
        data: crate::SurvivorLumpSum { cost, recovered, .. },
    }) = v.taxable
    {
        crate::require_localized!(
            r,
            v.kind == RetirementKind::Pension
                && matches!(
                    v.ordinary_distribution,
                    Some(RetirementTreatment::Exception {
                        facts: RetirementException::Death { .. },
                        early_simple: false,
                        ..
                    })
                ),
            &context,
            &source,
            "ISSUE_CONFIRM_THE_COMPLETE_ORDINARY_CSRS_FERS_DEATH_BENEFIT_LUMP_SUM_TO_T_0DC2F9CD",
            "Confirm the complete ordinary CSRS/FERS death-benefit lump sum to this individual and documented death treatment. Split payments, rollovers, special elections, estate/trust returns and final-annuitant deductions require separate treatment.",
        );
        let taxable = v.taxable_before_rollover();
        emit_lines(
            r,
            &mut (),
            [
                ("taxable", taxable),
                ("cost_recovered", v.gross - taxable),
                ("unrecovered_cost", phase::floor(cost - recovered - v.gross, 0)),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("civil_service_death_{}_{suffix}", e.id),
                    LineValue::MoneyCents(amount),
                    &["income.retirement.taxable"],
                    "https://www.irs.gov/publications/p721",
                    msg!("EXPLAIN_CIVIL_SERVICE_DEATH_N_N"),
                ))
            },
        );
    }
    if let Some(crate::RetirementTaxable::ChildAnnuity { ref data }) = v.taxable {
        crate::require_localized!(
            r,
            d.people
                .as_ref()
                .and_then(|p| { if owner == 0 { Some(&p.taxpayer) } else { p.spouse.as_ref() } })
                .is_some_and(|p| data.matches_birth(p.birth_date))
                && v.kind == RetirementKind::Pension
                && matches!(v.ordinary_distribution, Some(RetirementTreatment::Exception {
                facts: RetirementException::Death { died }, ..
            }) if died <= data.started),
            &context,
            &source,
            "ISSUE_CONFIRM_THE_SINGLE_CHILD_CSRS_FERS_TEMPORARY_ANNUITY_AND_DEATH_EXCE_A6A0D1A2",
            "Confirm the single-child CSRS/FERS temporary annuity and death exception. Spouse/lifetime/multiple-child benefits and changed entitlement periods need their own worksheets.",
        );
        let (excluded, deferred, remaining) = data.result();
        emit_lines(
            r,
            &mut (),
            [
                ("income_exclusion", excluded),
                ("deferred_exclusion", deferred),
                ("remaining_cost", remaining),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("child_annuity_{}_{suffix}", e.id),
                    LineValue::MoneyCents(amount),
                    &["income.retirement.taxable"],
                    "https://www.irs.gov/publications/p721",
                    msg!("EXPLAIN_CHILD_ANNUITY_N_N"),
                ))
            },
        );
    }
    if let Some((data, covered_gross)) = v.pension_basis() {
        crate::require_localized!(
            r,
            v.kind == RetirementKind::Pension
                && !matches!(
                    v.ordinary_distribution,
                    Some(RetirementTreatment::Rollover { .. })
                        | Some(RetirementTreatment::Conversion { .. })
                ),
            &context,
            &source,
            "ISSUE_CONFIRM_THE_QUALIFIED_PENSION_SIMPLIFIED_METHOD_FACTS_IRA_ROLLOVER_924A685D",
            "The Simplified Method applies to a qualified pension; IRA, rollover, conversion and special allocation cases require their own worksheets.",
        );
        emit_lines(
            r,
            &mut (),
            [
                (
                    "income_exclusion",
                    data.excluded(covered_gross),
                    msg!("EXPLAIN_PENSION_N_N_INCOME_EXCLUSION"),
                ),
                ("basis_recovery", data.recovery(), msg!("EXPLAIN_PENSION_N_N_BASIS_RECOVERY")),
                (
                    "monthly_exclusion",
                    data.monthly(),
                    msg!("EXPLAIN_PENSION_N_N_MONTHLY_EXCLUSION"),
                ),
            ],
            |(suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("pension_{}_{suffix}", e.id),
                    LineValue::MoneyCents(amount),
                    &["income.retirement.taxable"],
                    "https://www.irs.gov/publications/p575",
                    explanation,
                ))
            },
        );
        if data.share.is_some() || !data.changes.is_empty() {
            line_cents(
                r,
                &alloc::format!("pension_{}_common_monthly_exclusion", e.id),
                data.common_monthly(),
                &["income.retirement.taxable"],
                "https://www.irs.gov/publications/p575",
                msg!("EXPLAIN_PENSION_N_COMMON_MONTHLY_EXCLUSION"),
            );
        }
        if data.earlier_months > 0 {
            line_cents(
                r,
                &alloc::format!("pension_{}_earlier_common_recovery", e.id),
                data.earlier_recovery(),
                &["income.retirement.taxable"],
                "https://www.irs.gov/publications/p575",
                msg!("EXPLAIN_PENSION_N_EARLIER_COMMON_RECOVERY"),
            );
        }
        if data.started >= 19870101 {
            line_cents(
                r,
                &alloc::format!("pension_{}_remaining_basis", e.id),
                data.remaining_after_payments(),
                &["income.retirement.taxable"],
                "https://www.irs.gov/publications/p575",
                msg!("EXPLAIN_PENSION_N_REMAINING_BASIS"),
            );
        }
    }
    let taxable = basis.taxable(e.id, v);
    t.disability[owner] +=
        phase::floor(v.disability.amount() - basis.public_safety.excluded(e.id), 0);
    crate::require_localized!(
        r,
        v.disability.eligible()
            && (v.disability.amount() == 0
                || (v.kind == RetirementKind::Pension
                    && v.disability.amount() <= taxable + basis.public_safety.excluded(e.id))),
        &context,
        &source,
        "ISSUE_SCHEDULE_R_DISABILITY_INCOME_MUST_BE_A_QUALIFYING_EMPLOYER_PENSION_4173D621",
        "Schedule R disability income must be a qualifying employer pension, within its taxable amount, with the required disability and mandatory-retirement-age facts.",
    );
    if let Some(crate::RetirementTaxable::PropertyProceeds { ref data }) = v.taxable {
        crate::require_localized!(
            r,
            v.kind == RetirementKind::Pension
                && matches!(v.ordinary_distribution, Some(RetirementTreatment::Rollover { .. }))
                && basis.public_safety.excluded(e.id) == 0,
            &context,
            &source,
            "ISSUE_CONFIRM_THE_COMPLETE_NONEMPLOYER_STOCK_SALE_NO_AFTER_TAX_BASIS_OR_S_B0466E40",
            "Distributed property sold for a rollover needs a pension source rolled over by its proceeds. NUA and mixed-property allocations require separate calculations.",
        );
        let sale = data.sale(v.gross, v.rolled_amount());
        collect_sale(
            t,
            &sale,
            e,
            r,
            context,
            #[cfg(test)]
            source,
        );
        emit_lines(
            r,
            &mut (),
            [
                ("retained_pension", data.pension(v.gross, v.rolled_amount())),
                ("retained_capital_gain", sale.gain()),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("property_rollover_{}_{suffix}", e.id),
                    LineValue::MoneyCents(amount),
                    &["income.retirement.taxable", "income.retirement.ordinary_distribution"],
                    "https://www.irs.gov/publications/p575",
                    msg!("EXPLAIN_PROPERTY_ROLLOVER_N_N"),
                ))
            },
        );
    }
    if let Some(crate::RetirementTaxable::FullDischarge {
        data: crate::ContractDischarge { investment, .. },
    }) = v.taxable
    {
        crate::require_localized!(
            r,
            matches!(v.kind, RetirementKind::Pension | RetirementKind::Nonqualified)
                && !matches!(v.ordinary_distribution, Some(RetirementTreatment::Conversion { .. }))
                && (v.kind != RetirementKind::Pension || v.supported_plan_rollover()),
            &context,
            &source,
            "ISSUE_CONFIRM_A_COMPLETE_CASH_DISCHARGE_OF_THIS_PENSION_OR_COMMERCIAL_ANN_27AF5A7B",
            "Confirm a complete cash discharge of this pension or commercial annuity contract with remaining investment reconciled. Partial payments, IRA/Roth, exchange, NUA, special elections and final-annuitant death deductions use other treatments.",
        );
        emit_lines(
            r,
            &mut (),
            [
                (
                    "basis_recovery",
                    phase::cap(investment, [v.gross]),
                    msg!("EXPLAIN_ANNUITY_N_N_BASIS_RECOVERY"),
                ),
                (
                    "surrender_loss",
                    phase::floor(investment - v.gross, 0),
                    msg!("EXPLAIN_ANNUITY_N_N_SURRENDER_LOSS"),
                ),
                (
                    "rolled_basis",
                    phase::floor(v.rolled_amount() - v.taxable_before_rollover(), 0),
                    msg!("EXPLAIN_ANNUITY_N_N_ROLLED_BASIS"),
                ),
            ],
            |(suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("annuity_{}_{suffix}", e.id),
                    LineValue::MoneyCents(amount),
                    &["income.retirement.gross", "income.retirement.taxable"],
                    "https://www.irs.gov/publications/p575",
                    explanation,
                ))
            },
        );
    }
    if let Some(crate::RetirementTaxable::QualifiedWithdrawal {
        data: crate::PlanDistribution { investment, .. },
    }) = v.taxable
    {
        crate::require_localized!(
            r,
            v.kind == RetirementKind::Pension
                && !matches!(v.ordinary_distribution, Some(RetirementTreatment::Conversion { .. }))
                && v.supported_plan_rollover(),
            &context,
            &source,
            "ISSUE_CONFIRM_AN_ORDINARY_QUALIFIED_EMPLOYER_PLAN_DISTRIBUTION_WITH_CORRE_987E0E35",
            "Confirm an ordinary qualified employer-plan distribution with correctly allocated nonforfeitable balance and investment. Roth, IRA, unsupported after-tax rollover, repayment, NUA, legacy contribution-first treatment, full-discharge loss and special lump-sum elections need separate calculations.",
        );
        let recovered = v.gross - v.taxable_before_rollover();
        emit_lines(
            r,
            &mut (),
            [
                (
                    "rolled_basis",
                    phase::floor(v.rolled_amount() - v.taxable_before_rollover(), 0),
                    msg!("EXPLAIN_PENSION_N_N_ROLLED_BASIS_2"),
                ),
                ("basis_recovery", recovered, msg!("EXPLAIN_PENSION_N_N_BASIS_RECOVERY_3")),
                (
                    "remaining_basis",
                    investment - recovered,
                    msg!("EXPLAIN_PENSION_N_N_REMAINING_BASIS"),
                ),
            ],
            |(suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("pension_{}_{suffix}", e.id),
                    LineValue::MoneyCents(amount),
                    &["income.retirement.gross", "income.retirement.taxable"],
                    "https://www.irs.gov/publications/p575",
                    explanation,
                ))
            },
        );
    }
    if let Some(crate::RetirementTaxable::LegacyAnnuity { ref data }) = v.taxable {
        crate::require_localized!(
            r,
            v.kind == RetirementKind::Nonqualified && data.confirmed(),
            &context,
            &source,
            "ISSUE_CONFIRM_PRE_AUGUST_14_1982_CONTRACT_ISSUANCE_AND_THE_INSURER_S_SEPA_99B832D6",
            "Confirm pre-August 14, 1982 contract issuance and the insurer's separate old/new investment and earnings records. Loans, pledges, surrenders, exchanges, losses within a portion and unresolved allocations require further calculations.",
        );
        let p = data.parts(v.gross);
        emit_lines(
            r,
            &mut (),
            [
                ("old_basis_recovery", p[0]),
                ("old_taxable_earnings", p[1]),
                ("new_taxable_earnings", p[2]),
                ("new_basis_recovery", p[3]),
                ("old_basis_remaining", data.before.investment - p[0]),
                ("new_basis_remaining", data.after.investment - p[3]),
            ],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("legacy_annuity_{}_{suffix}", e.id),
                    LineValue::MoneyCents(amount),
                    &["income.retirement.taxable", "income.retirement.gross"],
                    "https://www.irs.gov/publications/p575",
                    msg!("EXPLAIN_LEGACY_ANNUITY_N_N"),
                ))
            },
        );
    }
    if let Some(crate::RetirementTaxable::DeferredWithdrawal {
        data: crate::AnnuityDistribution { loan_pledge, value, investment, ref related, .. },
    }) = v.taxable
    {
        crate::require_localized!(
            r,
            v.kind == RetirementKind::Nonqualified,
            &context,
            &source,
            "ISSUE_CONFIRM_A_COMMERCIAL_ANNUITY_WITHDRAWAL_OR_NEWLY_TAXABLE_LOAN_PLEDG_4505AB3B",
            "Confirm a commercial annuity withdrawal or newly taxable loan/pledge before annuity payments begin, with ordinary earnings-first treatment and complete pre-withdrawal contract records. Include all required related contracts with reconciled balances before this payment. Full surrender, exchange, legacy investment and unresolved grouping need separate calculations.",
        );
        let recovered = if loan_pledge { 0 } else { v.gross - v.taxable_income() };
        let increase = if loan_pledge { v.taxable_income() } else { 0 };
        let [_, investment] = crate::retirement::annuity_balances(value, investment, related);
        // This row, unlike the pre-withdrawal pool, is an actual
        // MoneyCents output. Recovery can make a wide pool fit.
        let remaining_basis = crate::kernel::ratio::exact::rounded(
            &(investment - crate::kernel::ratio::exact::whole(recovered)
                + crate::kernel::ratio::exact::whole(increase)),
        );
        if remaining_basis.is_none() {
            crate::issue_localized!(
                r,
                &context,
                &source,
                "ANNUITY_REMAINING_BASIS_UNAVAILABLE",
                "The combined annuity's remaining basis cannot be represented in the required amount field.",
            );
        }
        for (suffix, amount, explanation) in [
            (
                "taxable_withdrawal",
                Some(v.taxable_income()),
                msg!("EXPLAIN_ANNUITY_N_N_TAXABLE_WITHDRAWAL"),
            ),
            ("basis_recovery", Some(recovered), msg!("EXPLAIN_ANNUITY_N_N_BASIS_RECOVERY_2")),
            ("remaining_basis", remaining_basis, msg!("EXPLAIN_ANNUITY_N_N_REMAINING_BASIS")),
            ("basis_increase", Some(increase), msg!("EXPLAIN_ANNUITY_N_N_BASIS_INCREASE")),
        ] {
            if suffix == "basis_increase" && !loan_pledge {
                continue;
            }
            let Some(amount) = amount else {
                continue;
            };
            line_cents(
                r,
                &alloc::format!("annuity_{}_{suffix}", e.id),
                amount,
                &["income.retirement.gross", "income.retirement.taxable"],
                "https://www.govinfo.gov/content/pkg/USCODE-2024-title26/html/USCODE-2024-title26-subtitleA-chap1-subchapB-partII-sec72.htm",
                explanation,
            );
        }
    }
    if let Some(crate::RetirementTaxable::VariableAnnuity { data }) = v.taxable {
        let death = d.people.as_ref().and_then(|people| {
            if e.owner == crate::DocumentOwner::Yours {
                people.taxpayer.death_date()
            } else {
                people.spouse.as_ref().and_then(|person| person.death_date())
            }
        });
        crate::require_localized!(
            r,
            data.schedule_valid(death) && (data.payments != 0 || v.gross == 0),
            &context,
            &source,
            "ISSUE_ENTER_THE_COMPLETE_2026_VARIABLE_ANNUITY_SCHEDULE_THROUGH_DEATH_OR_60B5DF1C",
            "Enter the complete 2026 variable-annuity schedule through death or contract expiration, including earlier-recipient installments where applicable. Zero installments require zero gross receipts and the owner's final return. Keep beneficiary and estate payments separate.",
        );
        crate::require_localized!(
            r,
            data.earlier.is_none()
                || (matches!(v.ordinary_distribution, Some(RetirementTreatment::Exception {
                    facts: RetirementException::Death { died }, ..
                }) if died / 10000 == 2026)
                    && !matches!(
                        data.history,
                        crate::VariableHistory::Prior { refigure: Some(_), .. }
                    )),
            &context,
            &source,
            "ISSUE_EARLIER_VARIABLE_PAYMENTS_REQUIRE_A_DESIGNATED_SURVIVOR_AFTER_A_202_C4019F88",
            "Earlier variable payments require a designated survivor after a 2026 death and the same annual exclusion for both recipients. A new shortfall election during the transition requires separate allocation.",
        );
        crate::require_localized!(
            r,
            v.kind == RetirementKind::Nonqualified,
            &context,
            &source,
            "ISSUE_CONFIRM_THE_VARIABLE_COMMERCIAL_ANNUITY_PAYMENT_SCHEDULE_AND_HISTOR_BFD4FEBA",
            "Confirm the variable commercial annuity payment schedule and history; reduced survivor benefits, split recipients and other special adjustments need further calculation.",
        );
        for (suffix, amount, explanation) in [
            (
                "annual_exclusion",
                data.annual(),
                msg!("EXPLAIN_VARIABLE_ANNUITY_N_N_ANNUAL_EXCLUSION"),
            ),
            (
                "basis_recovery",
                data.excluded(v.gross),
                msg!("EXPLAIN_VARIABLE_ANNUITY_N_N_BASIS_RECOVERY"),
            ),
            (
                "remaining_basis",
                data.cost - data.recovered() - data.earlier_recovery() - data.excluded(v.gross),
                msg!("EXPLAIN_VARIABLE_ANNUITY_N_N_REMAINING_BASIS"),
            ),
            (
                "earlier_recovery",
                data.earlier_recovery(),
                msg!("EXPLAIN_VARIABLE_ANNUITY_N_N_EARLIER_RECOVERY"),
            ),
        ] {
            if suffix == "earlier_recovery" && data.earlier.is_none() {
                continue;
            }
            line_cents(
                r,
                &alloc::format!("variable_annuity_{}_{suffix}", e.id),
                amount,
                &["income.retirement.taxable"],
                "https://www.irs.gov/publications/p939",
                explanation,
            );
        }
        if matches!(data.history, crate::VariableHistory::Prior { refigure: Some(_), .. }) {
            line_cents(
                r,
                &alloc::format!("variable_annuity_{}_election", e.id),
                data.cost,
                &["income.retirement.taxable"],
                "https://www.irs.gov/publications/p939",
                Message::new(
                    "EXPLAIN_VARIABLE_ANNUITY_N_ELECTION",
                    alloc::vec![
                        MessageArg::DateOnly(i64::from(data.started)),
                        MessageArg::Number(i64::from(data.age)),
                        MessageArg::DateOnly(i64::from(data.period)),
                        MessageArg::usd_cents(data.recovered())
                    ],
                ),
            );
        }
    }
    if let Some(crate::RetirementTaxable::GeneralAnnuity { data }) = v.taxable {
        crate::require_localized!(
            r,
            (v.kind == RetirementKind::Nonqualified
                || (v.kind == RetirementKind::Pension && data.qualified_method()))
                && !matches!(
                    v.ordinary_distribution,
                    Some(RetirementTreatment::Rollover { .. })
                        | Some(RetirementTreatment::Conversion { .. })
                ),
            &context,
            &source,
            "ISSUE_CONFIRM_THE_FIXED_PERIOD_OR_LIFETIME_GENERAL_RULE_FACTS_AND_RETAINE_6D9431CB",
            "Confirm the fixed-period or lifetime General Rule facts and retained table/method elections. A qualified-plan start on or after November 19, 1996 needs original age at least 75 and at least five years of guaranteed payments. Otherwise use the required Simplified Method or the applicable additional worksheet. Variable payments, refund features other than entered single-life guaranteed installments, exchanges and special ownership cases need additional calculations.",
        );
        for (suffix, amount, explanation) in [
            (
                "refund_adjustment",
                data.refund_adjustment(),
                msg!("EXPLAIN_ANNUITY_N_N_REFUND_ADJUSTMENT"),
            ),
            ("expected_return", data.expected(), msg!("EXPLAIN_ANNUITY_N_N_EXPECTED_RETURN")),
            ("basis_recovery", data.excluded(), msg!("EXPLAIN_ANNUITY_N_N_BASIS_RECOVERY_3")),
            (
                "remaining_basis",
                phase::floor(
                    data.cost - data.recovered - data.earlier_recovery() - data.excluded(),
                    0,
                ),
                msg!("EXPLAIN_ANNUITY_N_N_REMAINING_BASIS_2"),
            ),
            (
                "earlier_recovery",
                data.earlier_recovery(),
                msg!("EXPLAIN_ANNUITY_N_N_EARLIER_RECOVERY"),
            ),
        ] {
            if suffix == "earlier_recovery" && data.earlier_payments == 0 {
                continue;
            }
            if suffix == "remaining_basis" && data.started < 19870101 {
                continue;
            }
            line_cents(
                r,
                &alloc::format!("annuity_{}_{suffix}", e.id),
                amount,
                &["income.retirement.taxable"],
                "https://www.irs.gov/publications/p939",
                explanation,
            );
        }
    }
    if v.kind == RetirementKind::Nonqualified {
        crate::require_localized!(
            r,
            matches!(
                v.taxable,
                Some(crate::RetirementTaxable::Payer)
                    | Some(crate::RetirementTaxable::GeneralAnnuity { .. })
                    | Some(crate::RetirementTaxable::VariableAnnuity { .. })
                    | Some(crate::RetirementTaxable::LegacyAnnuity { .. })
                    | Some(crate::RetirementTaxable::DeferredWithdrawal { .. })
                    | Some(crate::RetirementTaxable::FullDischarge { .. })
            ) && matches!(
                v.ordinary_distribution,
                Some(RetirementTreatment::Ordinary)
                    | Some(RetirementTreatment::QualifiedCareCharge)
                    | Some(RetirementTreatment::Early)
                    | Some(RetirementTreatment::Exception {
                        facts: crate::RetirementException::Death { .. }
                            | crate::RetirementException::Disability { .. }
                            | crate::RetirementException::ImmediateAnnuity { .. },
                        ..
                    })
            ),
            &context,
            &source,
            "ISSUE_A_NONQUALIFIED_ANNUITY_NEEDS_A_PAYER_DETERMINED_AMOUNT_OR_SUPPORTED_F9E3034E",
            "A nonqualified annuity needs a payer-determined amount or supported annuity calculation, and ordinary, early-without-exception, death, permanent-disability or immediate-annuity treatment. Other basis methods, exchanges outside the direct-exchange calculation, repayments and other section 72(q) exceptions need additional worksheets.",
        );
        t.nonqualified_annuity += taxable;
        t.injured_nonqualified_annuity[owner] += taxable;
    }
    if let Some(RetirementTreatment::QualifiedCareCharge) = v.ordinary_distribution {
        line_cents(
            r,
            &alloc::format!("retirement_{}_ltc_investment_reduction", e.id),
            v.box_8a,
            &["income.retirement"],
            "https://www.irs.gov/pub/irs-pdf/i1099r.pdf",
            msg!("EXPLAIN_RETIREMENT_N_LTC_INVESTMENT_REDUCTION"),
        );
    }
    // An employer disability pension's payments before the plan's
    // minimum retirement age are wages (Form 1040 line 1h), not
    // line 5a/5b pension income (Pub. 525 "Disability Pensions").
    // A corrective distribution of excess contributions or deferrals from an
    // employer plan is wages on line 1h, less the excess deferral already counted
    // from the W-2s (Form 1040 instructions, line 1h; Pub. 525), not line 5a or 5b.
    let plan_corrective =
        matches!(v.ordinary_distribution, Some(RetirementTreatment::Corrective { .. }))
            && !v.kind.ira();
    let wages = match v.ordinary_distribution {
        Some(RetirementTreatment::Corrective { excess_deferral }) if plan_corrective => {
            phase::floor(taxable - excess_deferral, 0)
        }
        _ => v.disability_wages(taxable),
    };
    t.disability_pension[owner] += wages;
    let category = usize::from(!v.kind.ira());
    if !plan_corrective {
        t.retirement += taxable - wages;
        t.retirement_gross[category] += match &v.taxable {
            // Code 6, a section 1035 exchange, and code W, long-term care charges,
            // are not income: "You need not report this on your tax return" (Pub.
            // 575). A code P or R amount belongs on an earlier year's return.
            _ if v.box7
                & (crate::DistributionCodes::CODE_6.0
                    | crate::DistributionCodes::CODE_W.0
                    | crate::DistributionCodes::CODE_P.0
                    | crate::DistributionCodes::CODE_R.0)
                != 0 =>
            {
                0
            }
            // An elected Form 4972 distribution is outside line 5a; Part II alone
            // leaves its ordinary part there (the same amount as line 5b).
            _ if matches!(v.ordinary_distribution, Some(RetirementTreatment::LumpSum { .. })) => {
                taxable
            }
            _ => v.gross,
        } - wages;
        t.retirement_taxable[category] += taxable - wages;
        t.injured_retirement[category][owner] += taxable - wages;
    }
    t.withholding_1099 += v.withholding;
    crate::require_localized!(
        r,
        v.ordinary_distribution != Some(crate::RetirementTreatment::EarlySimple)
            || v.kind == RetirementKind::Ira,
        &context,
        &source,
        "ISSUE_RETIREMENT_INCOME_NEEDS_A_PAYER_DETERMINED_AMOUNT_OR_SUPPORTED_PENS_EA370F3F",
        "Retirement income needs a payer-determined amount or supported pension/IRA basis worksheet and a supported distribution treatment; unlisted exceptions need another worksheet.",
    );
    // The 1099-R page stores every box 7a code or
    // combination it does not calculate as
    // `RetirementTreatment::Unsupported` with the printed codes
    // and refuses it here by name, the same "stored, not
    // rejected" pattern as the W-2 box 12 codes below.
    if v.ordinary_distribution == Some(RetirementTreatment::Unsupported) {
        let (_english, display) = crate::DistributionCodes::refusal(v.box7);
        issue_message(
            r,
            context,
            #[cfg(test)]
            source,
            #[cfg(test)]
            _english,
            display,
        );
    }
    if let Some(RetirementTreatment::Conversion { .. }) = v.ordinary_distribution {
        crate::require_localized!(
            r,
            v.kind == RetirementKind::Ira
                && (v.taxable == Some(crate::RetirementTaxable::IraBasis)
                    || (v.taxable == Some(crate::RetirementTaxable::Payer) && v.box_2a == v.gross)),
            &context,
            &source,
            "ISSUE_CONFIRM_AN_ELIGIBLE_TRADITIONAL_TO_ROTH_IRA_CONVERSION_COMPLETED_BY_ABF78E0D",
            "Confirm an eligible traditional-to-Roth IRA conversion completed by year-end. Use the basis worksheet for after-tax contributions; inherited, employer-plan, RMD and other special conversions require another worksheet.",
        );
    }
    if let Some(RetirementTreatment::Rollover { amount, .. }) = v.ordinary_distribution {
        if v.kind == RetirementKind::Pension
            && v.calculated_plan_basis()
            && matches!(
                v.ordinary_distribution,
                Some(RetirementTreatment::Rollover {
                    destination: crate::RolloverDestination::IraNextYear,
                    ..
                })
            )
        {
            line_cents(
                r,
                &alloc::format!("pension_{}_next_year_ira_basis", e.id),
                v.rolled_basis(),
                &["income.retirement.taxable", "income.retirement.ordinary_distribution"],
                "https://www.irs.gov/instructions/i8606",
                msg!("EXPLAIN_PENSION_N_NEXT_YEAR_IRA_BASIS"),
            );
        }

        if let Some(RetirementTreatment::Rollover {
            destination: crate::RolloverDestination::DirectEmployerPlan { .. },
            ..
        }) = v.ordinary_distribution
        {
            crate::require_localized!(
                r,
                v.kind == RetirementKind::Pension,
                &context,
                &source,
                "ISSUE_CONFIRM_A_DIRECT_TRANSFER_FROM_AN_EMPLOYER_PLAN_TO_A_QUALIFIED_EMPL_11495C83",
                "Confirm a direct transfer from an employer plan to a qualified employee plan or 403(b) with separate taxable and after-tax accounting. IRA sources and indirect transfers cannot use this destination.",
            );
        }
        crate::require_localized!(
            r,
            (v.kind == RetirementKind::Ira
                && v.taxable == Some(crate::RetirementTaxable::IraBasis))
                || (v.kind == RetirementKind::Pension
                    && v.calculated_plan_basis()
                    && v.supported_plan_rollover())
                || (v.taxable == Some(crate::RetirementTaxable::Payer) && v.box_2a == v.gross)
                || ((v.taxable == Some(crate::RetirementTaxable::Payer) && v.box_2a == 0)
                    && amount == v.gross)
                // A rollover into a Roth account (code G, H or B) is taxed on the
                // payer's box 2a whatever it is.
                || (matches!(
                    v.ordinary_distribution,
                    Some(RetirementTreatment::Rollover {
                        destination: crate::RolloverDestination::Roth,
                        ..
                    })
                ) && matches!(v.taxable, Some(crate::RetirementTaxable::Payer))),
            &context,
            &source,
            "ISSUE_CONFIRM_AN_ELIGIBLE_ROLLOVER_AND_RECONCILE_ITS_SOURCE_IRA_AFTER_TAX_B39A03E6",
            "Confirm an eligible rollover and reconcile its source. IRA after-tax basis requires the owner worksheet; employer-plan calculated basis permits after-tax amounts into a traditional IRA by year-end with its basis record, or a confirmed direct employer-plan transfer with separate basis accounting. A payer-reported zero-tax direct rollover must transfer the full distribution.",
        );
    }
    Ok(())
}

pub(super) fn retirement_adjustments_phase(pass: &Pass<'_>, r: &mut FederalResult, t: &mut Totals) {
    let &Pass { d, basis, qcd, .. } = pass;
    let ira_repayment_excluded: i64 = basis.repayment_excluded.iter().sum();
    t.retirement -= ira_repayment_excluded;
    t.retirement_taxable[0] -= ira_repayment_excluded;
    for owner in 0..2 {
        t.injured_retirement[0][owner] -= basis.repayment_excluded[owner];
    }
    // Form 8915-F lines 15 and 26 replace the designated parts' full taxable
    // amounts on Form 1040 lines 5b and 4b.
    let disaster_result = crate::disaster::calculate(d, r, basis, qcd);
    let disaster = disaster_result.categories;
    for category in 0..2 {
        for owner in 0..2 {
            t.injured_retirement[category][owner] += disaster_result.owners[category][owner];
        }
    }
    t.retirement += disaster[0] + disaster[1];
    t.retirement_taxable[0] += disaster[0];
    t.retirement_taxable[1] += disaster[1];
}

pub(super) struct RetirementIncomeReportOut {
    pub(super) retirement_taxable: [i64; 2],
    pub(super) retirement: i64,
    pub(super) trump_death: i64,
    pub(super) conversion: i64,
}

pub(super) fn retirement_income_report_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
) -> RetirementIncomeReportOut {
    let &Pass { allocated_entries, basis, .. } = pass;
    // IRA and pension distributions occupy separate return lines. Round each
    // category once so the displayed return reconciles with the tax engine.
    let rollovers: i64 = allocated_entries
        .iter()
        .filter_map(|e| match &e.income {
            FederalIncome::Retirement { data: stored } => {
                let v = &crate::FederalRetirement::from(stored);
                match v.ordinary_distribution {
                    Some(RetirementTreatment::Rollover { amount, .. }) => Some(amount),
                    _ => None,
                }
            }
            _ => None,
        })
        .sum();
    line(
        r,
        "retirement_rollovers",
        dollars(rollovers),
        &["income.retirement"],
        "https://www.irs.gov/retirement-plans/plan-participant-employee/rollovers-of-retirement-plan-and-ira-distributions",
        msg!("EXPLAIN_RETIREMENT_ROLLOVERS"),
    );
    let retirement_taxable = t.retirement_taxable.map(dollars);
    let retirement = retirement_taxable.iter().sum::<i64>();
    let trump_death = dollars(t.trump_death);
    line(
        r,
        "trump_death_taxable_income",
        trump_death,
        &["income.trump_death"],
        "https://www.irs.gov/irb/2025-52_IRB",
        msg!("EXPLAIN_TRUMP_DEATH_TAXABLE_INCOME"),
    );
    let conversion = dollars(
        allocated_entries
            .iter()
            .map(|e| match &e.income {
                FederalIncome::Retirement { data: stored } => {
                    let data = &crate::FederalRetirement::from(stored);
                    basis.converted(e.id, data)
                }
                _ => 0,
            })
            .sum(),
    );
    line(
        r,
        "roth_conversion_taxable",
        conversion,
        &["income.retirement", "adjustments.ira_basis"],
        "https://www.irs.gov/instructions/i8606",
        msg!("EXPLAIN_ROTH_CONVERSION_TAXABLE"),
    );
    RetirementIncomeReportOut { retirement_taxable, retirement, trump_death, conversion }
}
