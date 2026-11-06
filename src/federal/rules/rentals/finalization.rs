use super::*;

pub(crate) struct FinalizationInput<'input, 'draft> {
    pub(crate) allocated_entries: &'input [alloc::borrow::Cow<'draft, crate::FederalIncomeEntry>],
    pub(crate) care: &'input crate::care::CareDeductions,
    pub(crate) native_partner_se: Vec<(i64, crate::partnership::PartnerSeResult)>,
    pub(crate) inputs_complete: bool,
    pub(crate) source_issue: Option<crate::activity_risk::SourceIssue>,
}

pub(crate) fn finish(
    context: &rental_setup::RentalContext<'_, '_>,
    input: FinalizationInput<'_, '_>,
    reporting: rental_reporting::ReportingOutput,
    r: &mut FederalResult,
) -> RentalResult {
    let form_context = context.form;
    let d = form_context.draft;
    let source_claims = form_context.claims;
    let amt = context.amt;
    let home_use = context.home_use;
    let prefix = if amt { "amt_" } else { "" };
    let FinalizationInput {
        allocated_entries,
        care,
        native_partner_se,
        inputs_complete,
        mut source_issue,
    } = input;
    let rental_reporting::ReportingOutput {
        mut result,
        post_pal,
        personal_display_balances,
        professional_pass_through_rentals,
    } = reporting;
    macro_rules! rental_source {
        ($value:expr) => {
            match $value {
                Ok(value) => value,
                Err(issue) => {
                    issue.report(r);
                    source_issue.get_or_insert(issue);
                    continue;
                }
            }
        };
    }
    // Schedule E totals sum its separately reported activity rows and columns.
    // Exact source stages remain in the selected owner packets and ledgers.
    result.income = result.income_by_id.values().sum();
    if !amt && d.any_professional() {
        let mut total = 0;
        for entry in allocated_entries {
            // A short-stay business variant is not rental real estate and a
            // section 280A home year is outside section 469 (section 469(j)(10)).
            if let FederalIncome::Rental { data } = &entry.income
                && data.participation.professional()
                && data.property_type.business_participation().is_none()
                && !used_as_home(data)
                && d.professional(crate::community::professional_owner(
                    d.setup.as_ref(),
                    entry.owner.owned(),
                ))
                .is_some()
            {
                let current = r
                    .lines
                    .iter()
                    .find(|l| l.id == alloc::format!("rental_{}_line21_income", entry.id))
                    .map_or(0, |l| crate::kernel::ratio::trunc_i64(l.amount, 100));
                let allowed = r
                    .lines
                    .iter()
                    .find(|l| l.id == alloc::format!("rental_{}_line22_allowed_loss", entry.id))
                    .map_or(0, |l| crate::kernel::ratio::trunc_i64(l.amount, 100));
                total += positive(current) + allowed;
                total += rental_source!(crate::asset_sale::rental_source(
                    form_context,
                    entry.id,
                    false,
                    Some(home_use)
                ))
                .map_or(0, |sale| sale.gain);
            }
        }
        // K-1 professional rental amounts are already source-tagged by their
        // owning module; this consumer does not change its pool mechanics.
        total += professional_pass_through_rentals;
        form_line(
            r,
            FormLine::new(TaxForm::ScheduleE, "43"),
            "schedule_e_line43_professional_rentals",
            total,
            &["income.rental", "income.partnership", "income.scorp"],
            PASSIVE,
            msg!("EXPLAIN_SCHEDULE_E_PROFESSIONAL_RENTALS"),
        );
    }
    if result.inputs_complete {
        debug_assert_eq!(result.income_by_id.values().sum::<i64>(), result.income);
        debug_assert_eq!(
            result.ordinary_business_by_id.values().sum::<i64>(),
            result.ordinary_pass_through_business()
        );
    }
    line(
        r,
        &alloc::format!("{prefix}rental_income"),
        result.income,
        &["income.rental", "rental_magi"],
        PASSIVE,
        msg!("EXPLAIN_0RENTAL_INCOME"),
    );
    line(
        r,
        &alloc::format!("{prefix}rental_suspended_loss"),
        result.passive_loss,
        &["income.rental", "rental_magi"],
        PASSIVE,
        msg!("EXPLAIN_0RENTAL_SUSPENDED_LOSS"),
    );
    if amt && result.inputs_complete {
        result.partner_se.extend(native_partner_se.iter().cloned());
        for entry in allocated_entries {
            if let FederalIncome::Partnership { data } = &entry.income
                && let Some(partner) = result.partners.iter().find(|v| v.id == entry.id)
            {
                let section179 = crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                    result.section179_allowed_cents(d, entry.id),
                )
                .expect("selected section179 source allowance is nonnegative");
                // An unavailable character projection keeps the existing
                // related-benefit omission; the settled activity net is valid.
                if let Ok(mut se) = data.amt_earned_after_passive(partner.suspended, section179) {
                    if data.limited_for_se == Some(false) {
                        let deduction = care.source(entry.id);
                        se.earnings = crate::kernel::carry::native_balance(se.earnings, deduction);
                        se.earnings_cents = crate::kernel::carry::native_balance(
                            se.earnings_cents,
                            deduction * 100,
                        );
                    }
                    result.partner_se.push((entry.id, se));
                }
            }
        }
    }
    if !amt && result.inputs_complete {
        // Native nonpassive income was already excluded above by its group.
        // Only the independently established passive SE subset remains here.
        for (id, se) in native_partner_se {
            result.partner_niit_adjustment -= se.passive_earnings;
            *result.nii_adjustment_by_id.entry(id).or_default() -= se.passive_earnings;
            result.partner_se.push((id, se));
        }
        for entry in allocated_entries {
            if let FederalIncome::Partnership { data } = &entry.income
                && let Some(partner) = result.partners.iter().find(|v| v.id == entry.id)
            {
                let counterpart = d.setup.as_ref().is_some_and(crate::community::applies)
                    && entry.owner == crate::DocumentOwner::Spouses;
                match crate::community::partner_se(
                    d,
                    entry,
                    data,
                    partner.suspended,
                    crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                        result.section179_allowed_cents(d, entry.id),
                    )
                    .expect("selected section179 source allowance is nonnegative"),
                ) {
                    Ok(mut se) => {
                        // General partners subtract partner-level business expenses
                        // from the K-1 14A source; a limited partner's excluded
                        // distributive share never becomes SE income.
                        if data.limited_for_se == Some(false) {
                            let deduction = care.source(entry.id);
                            se.earnings =
                                crate::kernel::carry::native_balance(se.earnings, deduction);
                            se.earnings_cents = crate::kernel::carry::native_balance(
                                se.earnings_cents,
                                deduction * 100,
                            );
                        }
                        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
                            "Partnership document {}, SE source including taxable guaranteed payments after regular activity limits: ${}; SE-character suspended losses: basis ${}, at-risk ${}, passive ${}. Intermediate attribution before Schedule SE and other return-level coordination, not additional Schedule E income.",
                            entry.id, se.earnings, se.basis_suspended, se.risk_suspended, se.passive_suspended,
                        ), Message::new("NOTE_PARTNERSHIP_SE_SOURCE", alloc::vec![
                            MessageArg::Number(entry.id), MessageArg::usd_dollars(se.earnings),
                            MessageArg::usd_dollars(se.basis_suspended), MessageArg::usd_dollars(se.risk_suspended),
                            MessageArg::usd_dollars(se.passive_suspended),
                        ])));
                        // Form 8960 line 4b excludes ordinary nonpassive income
                        // and passive SE income once. Trading remains investment
                        // income regardless of participation; remove only its SE
                        // items, if present, using the independent source record.
                        let adjustment = match data.trade_kind {
                            crate::ScorpTradeKind::Ordinary => {
                                -partner.nonpassive
                                    - if counterpart {
                                        0
                                    } else {
                                        crate::community::income_amount(
                                            d,
                                            entry.id,
                                            se.passive_earnings,
                                        )
                                    }
                                    + if data.limited_for_se == Some(true) {
                                        // Capital guarantees remain NII even though
                                        // section 469 treats them as nonpassive.
                                        dollars(data.guaranteed_capital)
                                    } else {
                                        0
                                    }
                            }
                            crate::ScorpTradeKind::FinancialTrading => {
                                if counterpart {
                                    0
                                } else {
                                    -crate::community::income_amount(d, entry.id, se.earnings)
                                }
                            }
                        };
                        result.partner_niit_adjustment += adjustment;
                        *result.nii_adjustment_by_id.entry(entry.id).or_default() += adjustment;
                        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] alloc::format!(
                            "Partnership document {}, investment income after activity limits: ${}; Form 8960 adjustment: ${adjustment}. Additional return-level limits remain separate.",
                            entry.id, partner.net + adjustment,
                        ), Message::new("NOTE_PARTNERSHIP_INVESTMENT_INCOME", alloc::vec![
                            MessageArg::Number(entry.id), MessageArg::usd_dollars(partner.net + adjustment), MessageArg::usd_dollars(adjustment),
                        ])));
                        if counterpart {
                            result.counterpart_partner_se.push((entry.id, se));
                        } else {
                            result.partner_se.push((entry.id, se));
                        }
                    }
                    Err((key, _message)) => {
                        #[cfg(test)]
                        let message = _message;

                        crate::issue_localized!(
                            r,
                            &crate::interview::income_issue_context(entry),
                            "Partnership self-employment loss attribution",
                            if counterpart
                                && key
                                    == "ISSUE_PARTNERSHIP_RECONCILE_BOX_14_LESS_GUARANTEED_PAYMENTS_F10E8824"
                            {
                                "ISSUE_COMMUNITY_COUNTERPART_PARTNER_SE_UNRESOLVED"
                            } else {
                                key
                            },
                            if counterpart
                                && key
                                    == "ISSUE_PARTNERSHIP_RECONCILE_BOX_14_LESS_GUARANTEED_PAYMENTS_F10E8824"
                            {
                                "Your spouse's partnership self-employment loss is held back by activity limits. This App cannot reconcile that partner's Schedule SE or the required Form 8958 allocation; this separate return is not calculated."
                            } else {
                                message
                            },
                        );
                        result.inputs_complete = false;
                    }
                }
            }
        }
    }
    // Real stage operands remain available for diagnosis, but an incomplete
    // native context cannot supply a final annual source amount. Keep None;
    // the existing owner issues still govern the return's unsupported state.
    if result.inputs_complete && r.issues.is_empty() {
        for (id, source) in &mut result.operating_sources {
            source.filed_ordinary_dollars = result.income_by_id.get(id).copied();
            if let Some(foreign) = result.foreign_sources.get_mut(id) {
                foreign.operating =
                    crate::kernel::source_fact::SourceFact::Reported(source.clone());
                foreign.filed_ordinary_dollars = source.filed_ordinary_dollars.map_or(
                    crate::kernel::source_fact::SourceFact::Unavailable,
                    crate::kernel::source_fact::SourceFact::Reported,
                );
            }
        }
    }
    result.source_issue = source_issue.clone().or(result.source_issue.take());
    result.inputs_complete &= result.source_issue.is_none();
    if result.inputs_complete {
        debug_assert!(post_pal.sources.iter().all(|source| {
            source.characters.iter().all(|amount| {
                amount.suspended_cents >= 0 && amount.suspended_cents <= positive(-amount.net_cents)
            }) && source.characters.iter().map(|amount| amount.net_cents).sum::<i64>()
                == source.net_cents
                && source.characters.iter().map(|amount| amount.suspended_cents).sum::<i64>()
                    == source.suspended_cents
                && source.characters.iter().map(|amount| amount.allowed_net_cents).sum::<i64>()
                    == source.allowed_net_cents
        }));
        for source in &post_pal.sources {
            if source.component == PostPalComponent::DirectRental
                && let Some(foreign) = result.foreign_sources.get_mut(&source.income)
            {
                foreign.post_pal = crate::kernel::source_fact::SourceFact::Reported(source.clone());
            }
        }
        result.post_pal = Some(post_pal);
    }
    if let Some(allowed) = result.personal_expenses_cents() {
        // Preserve independent annual categories: round the combined regular
        // current costs and combined final allowed costs, then subtract. A
        // rounded held adjustment subtracted from raw cents fails across zero.
        result.personal_expense_adjustment = dollars(
            result.personal_inputs.values().map(|input| input.regular_expenses_cents).sum(),
        ) - dollars(allowed.values().sum());
        // Show a complete expense reconciliation even when all history is used
        // or net income is recharacterized and never enters the loss allocation.
        for e in allocated_entries {
            if let FederalIncome::NonbusinessProperty {
                kind: NonbusinessPropertyKind::PersonalRental { .. },
                ..
            } = &e.income
            {
                let input = &result.personal_inputs[&e.id];
                let prior = dollars(input.prior_cents);
                let closing_cents = result
                    .post_pal
                    .as_ref()
                    .expect("complete personal costs")
                    .sources
                    .iter()
                    .find(|source| {
                        source.income == e.id
                            && source.component == PostPalComponent::PersonalProperty
                    })
                    .expect("same-invocation personal source")
                    .suspended_cents;
                let closing = match personal_display_balances.get(&e.id) {
                    Some(balance) => *balance,
                    // Positive and recharacterized sources have a complete
                    // zero hold from the native packet, outside loss rows.
                    None => dollars(closing_cents),
                };
                emit_lines(
                    r,
                    &mut (),
                    [
                        (
                            "prior_suspended_expenses",
                            prior,
                            msg!("EXPLAIN_0PERSONAL_RENTAL_N_N_PRIOR_SUSPENDED_EXPENSES"),
                        ),
                        (
                            "allowed_expenses",
                            dollars(allowed[&e.id]),
                            msg!("EXPLAIN_0PERSONAL_RENTAL_N_N_ALLOWED_EXPENSES"),
                        ),
                        (
                            "suspended_loss",
                            closing,
                            msg!("EXPLAIN_0PERSONAL_RENTAL_N_N_SUSPENDED_LOSS"),
                        ),
                    ],
                    |(suffix, amount, explanation), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!("{prefix}personal_rental_{}_{suffix}", e.id),
                            LineValue::MoneyDollars(amount),
                            &["income.nonbusiness_property", "rental_magi"],
                            PASSIVE,
                            explanation,
                        ))
                    },
                );
            }
        }
    }
    if inputs_complete {
        for entry in allocated_entries {
            let (kind, source_path) = match entry.income {
                FederalIncome::PartnershipActivities { .. } => {
                    ("partnership", "income.partnership.activities")
                }
                FederalIncome::ScorpActivities { .. } => ("scorp", "income.scorp.activities"),
                _ => continue,
            };
            if let Some(value) = result.k1_amounts.iter().find(|v| v.id == entry.id) {
                line(
                    r,
                    &alloc::format!("{prefix}{kind}_{}_schedule_e", entry.id),
                    value.net,
                    &[source_path],
                    PASSIVE,
                    if kind == "partnership" {
                        msg!("EXPLAIN_0PARTNERSHIP_N_ACTIVITY_NET")
                    } else {
                        msg!("EXPLAIN_SCORP_ACTIVITY_NET")
                    },
                );
            }
        }
    }
    // Settle the existing property rows once from this invocation's actual
    // basis/risk ledger and final PAL output, before any Form4797 lookback.
    if result.inputs_complete {
        for entry in allocated_entries {
            let settled = crate::k1_sales::facts(entry, amt,
                result.k1_ledgers.iter().find(|(id, _)| *id == entry.id).map(|(_, ledger)| ledger))
                .and_then(|facts| facts.map(|facts| {
                    let sales = facts.settle_property(&result, entry.id)?;
                    let range = (crate::k1_activity::ordinary::RANGE_KEY,
                        crate::fixture_english!("Review the document amounts; they must be within the supported input range."));
                    let reported_net = result.k1_section1231.iter().find(|value| value.id == entry.id)
                        .map(|value| value.reported_net(&sales).ok_or(range)).transpose()?;
                    Ok((sales, reported_net))
                }).transpose());
            match settled {
                Ok(Some((sales, reported_net))) => {
                    if let Some(value) =
                        result.k1_section1231.iter_mut().find(|value| value.id == entry.id)
                    {
                        value.net = reported_net.expect("selected property row");
                        value.section179_sales_cents = sales;
                    }
                }
                Ok(None) => {}
                Err((key, _message)) => {
                    crate::issue_localized!(
                        r,
                        &crate::IssueContext::income_id(entry.id),
                        "K-1 section 179 sale",
                        key,
                        _message
                    );
                    result.inputs_complete = false;
                    source_issue = Some(crate::activity_risk::SourceIssue::new(
                        entry.id,
                        crate::activity_risk::ProjectionIssue::Source(key),
                    ));
                }
            }
        }
    }
    if result.inputs_complete && source_issue.is_none() {
        for entry in crate::community::property_entries(d) {
            let FederalIncome::Rental { data } = &entry.income else { continue };
            if !used_as_home(data) {
                continue;
            }
            let settled = if data.buildings.iter().any(|cost| cost.casualty.is_some()) {
                crate::asset_sale::rental_allowed_property_rows(
                    form_context,
                    &result,
                    entry.id,
                    data,
                    amt,
                )
                .and_then(|rows| {
                    let current = rows
                        .map_or(Ok(0), |(source, rows)| source.rental_casualty_income(&rows))
                        .map_err(|issue| crate::activity_risk::SourceIssue::new(entry.id, issue))?;
                    let current = Some(current);
                    current
                        .and_then(|amount| amount.checked_mul(100))
                        .and_then(|amount| {
                            amount.checked_add(
                                result
                                    .prior_home_casualty_by_id
                                    .get(&entry.id)
                                    .copied()
                                    .unwrap_or(0),
                            )
                        })
                        .ok_or_else(|| {
                            crate::activity_risk::SourceIssue::new(
                                entry.id,
                                crate::activity_risk::ProjectionIssue::Arithmetic(
                                    crate::kernel::loss_limits::LossLimitError::Overflow,
                                ),
                            )
                        })
                })
            } else {
                Ok(result.prior_home_casualty_by_id.get(&entry.id).copied().unwrap_or(0))
            };
            match settled {
                Ok(amount) => {
                    result.settled_home_casualty_by_id.insert(entry.id, amount);
                }
                Err(issue) => {
                    issue.report(r);
                    source_issue.get_or_insert(issue);
                }
            }
        }
    }
    result.source_issue = source_issue.clone().or(result.source_issue.take());
    result.inputs_complete &= result.source_issue.is_none();
    // Bind the monetary projection after this invocation's selected source and
    // passive owners have finished. Trial invocations build their own packet.
    result.source_projection = Some(alloc::sync::Arc::new(
        crate::community::SourceProjection::produce(d, &result, amt, source_claims),
    ));

    result
}
