// form_5329 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_able(
    _cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::AbleSource,
) -> Result<(), CollectFailure> {
    let Source { e, owner, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    let data = &crate::AbleDistribution::from(data);
    if data.annual_to_non_beneficiary()
        && let Err((key, _english)) = data.amounts()
    {
        crate::issue_localized!(r, context, source, key, _english);
    }
    let Ok(amounts) = data.amounts() else { return Ok(()) };
    let (rolled, rolled_earnings) = data.rollover();
    if rolled > 0 {
        emit_lines(
            r,
            &mut (),
            [("rollover", rolled), ("rollover_earnings", rolled_earnings)],
            |(suffix, value), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("able_{}_{}", e.id, suffix),
                    LineValue::MoneyCents(value),
                    &["income.able.treatment.rollover"],
                    "https://www.irs.gov/irb/2020-50_IRB",
                    msg!("EXPLAIN_ABLE_N_N_2"),
                ))
            },
        );
    }
    if let Some(crate::AbleTreatment::Annual { following_expenses, .. }) = data.treatment
        && following_expenses > 0
    {
        line_cents(
            r,
            &alloc::format!("able_{}_following_expenses", e.id),
            following_expenses,
            &["income.able.treatment"],
            "https://www.irs.gov/irb/2020-50_IRB",
            msg!("EXPLAIN_ABLE_N_FOLLOWING_EXPENSES"),
        );
        r.review_notes.push(ReviewNote::localized(
            #[cfg(test)] alloc::format!("ABLE expense election — {}: retain the first-60-days payment records and do not count the elected expenses again in 2027.", e.label),
            crate::Message::new("NOTE_ABLE_EXPENSE_ELECTION_RECORDS", alloc::vec![crate::interview::income_display_arg(e)]),
        ));
    }
    let taxable = amounts.taxable;
    t.able[owner] += taxable;
    t.able_penalty_base[owner] += amounts.penalized;
    t.injured_other_sources.insert(e.id, Some(exact::whole(taxable)));
    t.injured_accounts.source(e.id, e.owner.owned(), exact::whole(taxable));
    if matches!(data.treatment, Some(crate::AbleTreatment::Annual { .. })) {
        line_cents(
            r,
            &alloc::format!("able_{}_annual_taxable", e.id),
            taxable,
            &["income.able.treatment"],
            "https://www.irs.gov/irb/2020-50_IRB",
            msg!("EXPLAIN_ABLE_N_ANNUAL_TAXABLE"),
        );
    }
    if let Some(crate::AbleTreatment::Corrective { year }) = data.treatment {
        t.able_corrections += taxable;
        if year < 2026 {
            r.review_notes.push(ReviewNote::localized(#[cfg(test)] alloc::format!(
                "ABLE correction — {}: the returned contribution's earnings belong on the contributor's {} return, not the 2026 return. Review that return for an amendment.", e.label, year
            ), Message::new("NOTE_ABLE_CORRECTION_PRIOR_RETURN", alloc::vec![
                crate::interview::income_display_arg(e), MessageArg::Number(year as i64),
            ])));
        }
    }
    Ok(())
}

pub(super) fn collect_trump_death(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    stored: &crate::R1099Source,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    let data = &crate::TrumpDeathAcquisition::from(stored);
    let recipient = d.people.as_ref().and_then(|people| {
        if e.owner == crate::DocumentOwner::Yours {
            Some(&people.taxpayer)
        } else {
            people.spouse.as_ref()
        }
    });
    // The estate case is the child's own final return: the owner
    // died after Trump accounts began and before the year the
    // owner turns 18 (section 530A(d)(6)). Another acquirer
    // reports the account on their own return.
    let recipient_valid = recipient.is_some_and(|person| {
        if data.beneficiary_estate {
            person.birth_date > 20081231
                && person.death_date().is_some_and(|date| (20260704..=20261231).contains(&date))
        } else {
            // A child who died in 2026 cannot be the one who acquired the account.
            !(person.birth_date > 20081231 && person.death_date().is_some())
        }
    });
    crate::require_localized!(
        r,
        recipient_valid,
        context,
        source,
        "ISSUE_CONFIRM_THE_TRUSTEE_S_DEATH_DATE_VALUE_AND_BASIS_THE_QUALIFIED_ACCO_5D336F75",
        "Confirm the trustee's death-date value and basis, the qualified account and complete distributions. The beneficiary's estate acquisition belongs on that beneficiary's final return; another acquirer reports it on their own return. Unentered excess or other distributions need separate treatment.",
    );
    if !recipient_valid {
        return Ok(());
    }
    let Some(taxable) = data.taxable() else {
        return Ok(());
    };
    t.trump_death += taxable;
    t.withholding_1099 += data.withholding;
    line_cents(
        r,
        &alloc::format!("trump_death_{}_value", e.id),
        data.value,
        &["income.trump_death.value"],
        "https://www.irs.gov/irb/2025-52_IRB",
        msg!("EXPLAIN_TRUMP_DEATH_N_VALUE"),
    );
    line_cents(
        r,
        &alloc::format!("trump_death_{}_taxable", e.id),
        taxable,
        &["income.trump_death.value", "income.trump_death.basis"],
        "https://www.irs.gov/instructions/i1099r",
        msg!("EXPLAIN_TRUMP_DEATH_N_TAXABLE"),
    );
    r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Trump-account death distribution: verify the final 2026 Form 1040 line placement before filing. This calculation includes the section 530A(d)(6) taxable value once as other income.", crate::Message::new("NOTE_TRUMP_ACCOUNT_DEATH_DISTRIBUTION_VERIFY_THE_FINAL_2026_FORM_1040_5BA96871", alloc::vec![])));
    Ok(())
}

pub(super) fn collect_trump_able_rollover(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    _t: &mut Totals,
    r: &mut FederalResult,
    stored: &crate::R1099Source,
) -> Result<(), CollectFailure> {
    let Collect { d, .. } = *cx;
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    let data = &crate::TrumpAbleRollover::from(stored);
    let beneficiary = d.people.as_ref().and_then(|people| {
        if e.owner == crate::DocumentOwner::Yours {
            Some(&people.taxpayer)
        } else {
            people.spouse.as_ref()
        }
    });
    let age_and_life = beneficiary.is_some_and(|person| {
        // The calendar year the beneficiary turns 17, day-exact: a
        // beneficiary born January 1, 2010 attains 17 on December
        // 31, 2026; one born January 1, 2009 did so in 2025.
        crate::federal_model::age_at_year_end(person.birth_date) == 17
            && person.death_date().is_none_or(|date| date >= data.transferred)
    });
    let no_unresolved_excess = d.screening.as_ref().is_some_and(|screen| {
        screen.trump_accounts.iter().filter(|account| account.owner == e.owner.owned()).all(
            |account| {
                let limit_excess = positive(
                    account.contributions
                        - account.exempt
                        - crate::year_constants::TRUMP_ACCOUNT_ANNUAL_LIMIT_CENTS,
                );
                account.returns.iter().all(|correction| correction.received <= data.transferred)
                    && account.returns.iter().map(|v| v.principal).sum::<i64>() >= limit_excess
            },
        )
    });
    crate::require_localized!(
        r,
        age_and_life && no_unresolved_excess,
        context,
        source,
        "ISSUE_CONFIRM_A_FULL_DIRECT_TRANSFER_OF_THIS_17_YEAR_OLD_BENEFICIARY_S_OW_33BA2D11",
        "Confirm a full direct transfer of this 17-year-old beneficiary's own Trump account to their eligible ABLE account in 2026, and resolve any excess contributions before transfer. Partial, indirect or mismatched transfers require separate treatment.",
    );
    if !age_and_life || !no_unresolved_excess {
        return Ok(());
    }
    line_cents(
        r,
        &alloc::format!("trump_able_rollover_{}_gross", e.id),
        data.amount,
        &["income.trump_able_rollover"],
        "https://www.irs.gov/instructions/i1099r",
        msg!("EXPLAIN_TRUMP_ABLE_ROLLOVER_N_GROSS"),
    );
    Ok(())
}

pub(super) struct AccountDistributionsIn<'a> {
    pub(super) education: &'a crate::education_account::EducationTotals,
}

pub(super) struct AccountDistributionsOut {
    pub(super) able_excess_tax: i64,
    pub(super) trump_excess_tax: i64,
    pub(super) trump_earnings_tax: i64,
    pub(super) able_income: i64,
    pub(super) account_distribution_taxes: crate::retirement_return::AccountDistributionTaxes,
    pub(super) account_tax: i64,
}

pub(super) fn account_distributions_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
    i: AccountDistributionsIn<'_>,
) -> AccountDistributionsOut {
    let &Pass { d, p, s, screen, .. } = pass;
    let AccountDistributionsIn { education } = i;
    let able_excess_owners =
        crate::able::excess_tax(&screen.able_contributions, s.filing_status(), r);
    let able_excess_tax = able_excess_owners.iter().sum::<i64>();
    t.injured_accounts.fixed_tax(able_excess_owners);
    for entry in
        d.income.iter().filter(|entry| crate::trump_account::printed_excess(entry).is_some())
    {
        let linked = screen
            .trump_accounts
            .iter()
            .filter(|account| account.owner == entry.owner.owned())
            .flat_map(|account| &account.returns)
            .any(|row| row.document_id == Some(entry.id));
        crate::require_localized!(
            r,
            linked,
            &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
            "Trump-account corrective return",
            "DOCUMENT_TRUMP_ACCOUNT_FACTS",
            "Complete this beneficiary's contributions and the printed corrective distribution's receipt and deadline facts on Other taxes.",
        );
    }
    let (trump_excess_owners, trump_earnings_owners) = crate::trump_account::calculate(
        &crate::trump_account::project_accounts(d),
        p,
        s.filing_status(),
        r,
    );
    let trump_excess_tax = trump_excess_owners.iter().sum::<i64>();
    let trump_earnings_tax = trump_earnings_owners.iter().sum::<i64>();
    t.injured_accounts.fixed_tax(trump_excess_owners);
    t.injured_accounts.fixed_tax(trump_earnings_owners);
    let able_income = dollars(t.able.iter().sum());
    t.injured_accounts.income(able_income, t.able.map(exact::whole));
    // Form 5329 Part II: ABLE and education account distributions together.
    let account_distribution_taxes =
        crate::retirement_return::account_additional_tax(r, t.able, t.able_penalty_base, education);
    let account_tax = account_distribution_taxes.total_dollars;
    emit_lines(
        r,
        &mut (),
        [
            ("able_taxable_distributions", able_income),
            ("able_corrective_earnings", dollars(t.able_corrections)),
        ],
        |(id, amount), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["income.able"],
                "https://www.irs.gov/publications/p907",
                msg!("EXPLAIN_FEDERAL_RULES"),
            ))
        },
    );
    AccountDistributionsOut {
        able_excess_tax,
        trump_excess_tax,
        trump_earnings_tax,
        able_income,
        account_distribution_taxes,
        account_tax,
    }
}
