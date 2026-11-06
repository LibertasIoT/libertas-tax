// form_1099_g items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

pub(super) fn collect_tax_refund(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::GovernmentPaymentSource,
) -> Result<(), CollectFailure> {
    let Collect { d, source_claims, .. } = *cx;
    let Source { e, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    let crate::FederalTaxRefund { amount, history, withholding: _ } = &data.into();
    let source_before = [t.tax_refund, t.recovery_8z];
    crate::information_returns::refund_destination_issues_with_claims(
        d,
        e,
        r,
        context,
        #[cfg(test)]
        source,
        source_claims,
    );
    for deduction_year in history.deduction_years() {
        crate::require_localized!(
            r,
            !cx.itemized_refund_years.contains(&deduction_year),
            context,
            source,
            "ISSUE_COMBINE_INCOME_TAX_REFUNDS_DEDUCTED_ON_THE_SAME_PRIOR_FEDERAL_RETUR_38ED09DD",
            "Combine income-tax refunds deducted on the same prior federal return before calculating its tax benefit.",
        );
        cx.itemized_refund_years.push(deduction_year);
    }
    match history.prior_current_breakdown(*amount) {
        Ok(Some((prior_years, current_share))) => {
            let TaxRefundHistory::IncomeTaxPriorYearsAndCurrent {
                gross_current_income_tax_paid,
                ..
            } = history
            else {
                unreachable!()
            };
            crate::require_localized!(
                r,
                !cx.current_year_state_refund_seen,
                context,
                source,
                "ISSUE_THIS_NARROW_CURRENT_YEAR_PAYMENT_ROUTE_SUPPORTS_ONE_RECONCILED_REFU_9C1CE48E",
                "This narrow current-year payment route supports one reconciled refund only; combine or separately review other 2026 state-tax refunds before calculating the deduction.",
            );
            cx.current_year_state_refund_seen = true;
            let ded = d.deductions.as_ref().unwrap();
            let net_current_paid = gross_current_income_tax_paid - current_share;
            let income_tax_elected = ded.state_income_tax_elected();
            let sales_tax_elected = ded.state_sales_tax_elected();
            // The invocation supplies automatic candidates' routes. An
            // explicit override or a separately filing spouse can require
            // itemizing without a forced invocation context.
            let itemizes = cx
                .home
                .and_then(|home| home.itemizes)
                .or_else(|| (d.deduction_choice() == DeductionChoice::Itemized).then_some(true));
            let standard_deduction = itemizes == Some(false);
            crate::require_localized!(
                r,
                if income_tax_elected {
                    ded.income_tax_paid() == net_current_paid
                } else {
                    standard_deduction || sales_tax_elected
                },
                &context.clone().corrective(if income_tax_elected {
                    crate::TaxSection::Taxes
                } else {
                    crate::TaxSection::Deductions
                }),
                source,
                "ISSUE_RECONCILE_THE_RETAINED_GROSS_2026_STATE_INCOME_TAX_PAYMENT_LEDGER_T_FBABDEAF",
                "Reconcile the retained gross 2026 state-income-tax payment ledger with net state income tax paid when income tax is elected. Otherwise retain the ledger without a state-income-tax deduction and confirm the standard-deduction or sales-tax election.",
            );
            let taxable: i64 = prior_years.iter().map(|(_, _, taxable)| taxable).sum();
            t.tax_refund += taxable;
            let mut allocations = String::new();
            for (index, &(year, prior_share, year_taxable)) in prior_years.iter().enumerate() {
                line_cents(
                    r,
                    &alloc::format!("state_refund_{}_{}_allocated", e.id, year),
                    prior_share,
                    &["income.tax_refund"],
                    "https://www.irs.gov/publications/p525",
                    msg!("EXPLAIN_STATE_REFUND_N_N_ALLOCATED"),
                );
                line_cents(
                    r,
                    &alloc::format!("state_refund_{}_{}_taxable", e.id, year),
                    year_taxable,
                    &["income.tax_refund"],
                    "https://www.irs.gov/publications/p525",
                    msg!("EXPLAIN_STATE_REFUND_N_N_TAXABLE"),
                );
                if index > 0 {
                    allocations.push_str(", ");
                }
                allocations.push_str(&alloc::format!(
                    "${}.{:02} to {}",
                    prior_share / 100,
                    prior_share % 100,
                    year
                ));
            }
            line_cents(
                r,
                &alloc::format!("state_refund_{}_2026_allocated", e.id),
                current_share,
                &["income.tax_refund"],
                "https://www.irs.gov/publications/p525",
                if income_tax_elected {
                    msg!("EXPLAIN_STATE_REFUND_N_N_ALLOCATED_2")
                } else {
                    msg!("EXPLAIN_STATE_REFUND_N_N_ALLOCATED_3")
                },
            );
            line_cents(
                r,
                &alloc::format!("state_refund_{}_2026_net_income_tax_paid", e.id),
                net_current_paid,
                if income_tax_elected {
                    &["deductions.itemized.personal_tax", "income.tax_refund"]
                } else {
                    &["income.tax_refund"]
                },
                "https://www.irs.gov/publications/p525",
                if income_tax_elected {
                    msg!("EXPLAIN_STATE_REFUND_N_N_NET_INCOME_TAX_PAID")
                } else {
                    msg!("EXPLAIN_STATE_REFUND_N_N_NET_INCOME_TAX_PAID_2")
                },
            );
            #[cfg(test)]
            let current_treatment = if income_tax_elected {
                "The 2026 share has already reduced the net state-income-tax payment entered for Schedule A; the normal election and SALT cap determine the allowed deduction."
            } else if standard_deduction {
                "The net 2026 state-income-tax payment is not deducted because the standard deduction is selected."
            } else if sales_tax_elected {
                "The net 2026 state-income-tax payment is not deducted because general sales tax is elected."
            } else {
                "No state-income-tax deduction is elected; the retained payment ledger does not create a deduction."
            };
            let treatment_key = if income_tax_elected {
                "NOTE_STATE_REFUND_CURRENT_INCOME_TAX"
            } else if standard_deduction {
                "NOTE_STATE_REFUND_CURRENT_STANDARD"
            } else if sales_tax_elected {
                "NOTE_STATE_REFUND_CURRENT_SALES_TAX"
            } else {
                "NOTE_STATE_REFUND_CURRENT_NO_ELECTION"
            };
            r.review_notes.push(ReviewNote::localized(
                #[cfg(test)] alloc::format!("State/local income-tax refund document {}: {} and ${}.{:02} to 2026, allocated by tax paid in each year. Prior-year taxable shares total ${}.{:02} on Schedule 1 line 1. {} Retain the state payment ledger and each prior federal Schedule A.", e.id, allocations, current_share / 100, current_share % 100, taxable / 100, taxable % 100, current_treatment),
                Message::new(treatment_key, alloc::vec![
                    MessageArg::Number(e.id), MessageArg::usd_cents(current_share), MessageArg::usd_cents(taxable),
                ]),
            ));
            for &(year, prior_share, year_taxable) in &prior_years {
                r.review_notes.push(ReviewNote::localized(
                    #[cfg(test)] alloc::format!("State/local income-tax refund document {}: ${}.{:02} allocated to {year}, of which ${}.{:02} is taxable.", e.id, prior_share / 100, prior_share % 100, year_taxable / 100, year_taxable % 100),
                    Message::new("NOTE_STATE_REFUND_PRIOR_ALLOCATION", alloc::vec![
                        MessageArg::Number(e.id), MessageArg::usd_cents(prior_share),
                        MessageArg::Number(year as i64), MessageArg::usd_cents(year_taxable),
                    ]),
                ));
            }
        }
        Ok(None) => match history.multi_year_breakdown(*amount) {
            Ok(Some(years)) => {
                let taxable: i64 = years.iter().map(|(_, _, taxable)| taxable).sum();
                t.tax_refund += taxable;
                let mut allocations = String::new();
                for (index, &(year, allocated, year_taxable)) in years.iter().enumerate() {
                    line_cents(
                        r,
                        &alloc::format!("state_refund_{}_{}_allocated", e.id, year),
                        allocated,
                        &["income.tax_refund"],
                        "https://www.irs.gov/publications/p525",
                        msg!("EXPLAIN_STATE_REFUND_N_N_ALLOCATED_4"),
                    );
                    line_cents(
                        r,
                        &alloc::format!("state_refund_{}_{}_taxable", e.id, year),
                        year_taxable,
                        &["income.tax_refund"],
                        "https://www.irs.gov/publications/p525",
                        msg!("EXPLAIN_STATE_REFUND_N_N_TAXABLE_2"),
                    );
                    if index > 0 {
                        allocations.push_str(", ");
                    }
                    allocations.push_str(&alloc::format!(
                        "${}.{:02} to {}",
                        allocated / 100,
                        allocated % 100,
                        year
                    ));
                }
                r.review_notes.push(ReviewNote::localized(
                    #[cfg(test)] alloc::format!("State/local income-tax refund document {}: allocated {} by the tax paid in each year. Taxable shares total ${}.{:02} on Schedule 1 line 1. Retain the state payment ledger and each federal year's Schedule A records; no 2026 payment is included.", e.id, allocations, taxable / 100, taxable % 100),
                    Message::new("NOTE_STATE_REFUND_PRIOR_ONLY", alloc::vec![MessageArg::Number(e.id), MessageArg::usd_cents(taxable)]),
                ));
                for &(year, allocated, year_taxable) in &years {
                    r.review_notes.push(ReviewNote::localized(
                        #[cfg(test)] alloc::format!("State/local income-tax refund document {}: ${}.{:02} allocated to {year}, of which ${}.{:02} is taxable.", e.id, allocated / 100, allocated % 100, year_taxable / 100, year_taxable % 100),
                        Message::new("NOTE_STATE_REFUND_PRIOR_ALLOCATION", alloc::vec![
                            MessageArg::Number(e.id), MessageArg::usd_cents(allocated),
                            MessageArg::Number(year as i64), MessageArg::usd_cents(year_taxable),
                        ]),
                    ));
                }
            }
            Ok(None) => match history.taxable(*amount) {
                // A single-year income tax refund is figured with
                // its year's worksheet (other_income::collect_years).
                Ok(value) => {
                    if history.single_year().is_none() {
                        t.tax_refund += value
                    }
                }
                Err((key, _message)) => {
                    t.injured_other_amount(e.id, None);
                    crate::issue_localized!(r, context, source, key, _message,)
                }
            },
            Err((key, _message)) => {
                t.injured_other_amount(e.id, None);
                crate::issue_localized!(r, context, source, key, _message,)
            }
        },
        Err((key, _message)) => {
            t.injured_other_amount(e.id, None);
            crate::issue_localized!(r, context, source, key, _message,)
        }
    }
    if history.single_year().is_none() && !matches!(history, TaxRefundHistory::Unanswered) {
        for value in [t.tax_refund - source_before[0], t.recovery_8z - source_before[1]] {
            t.injured_other_amount(e.id, Some(value));
        }
    }
    Ok(())
}

pub(super) fn collect_unemployment(
    cx: &mut Collect<'_>,
    src: &Source<'_, '_>,
    t: &mut Totals,
    r: &mut FederalResult,
    data: &crate::GovernmentPaymentSource,
) -> Result<(), CollectFailure> {
    let Collect { d, source_claims, .. } = *cx;
    let Source { e, owner, context, .. } = *src;
    #[cfg(test)]
    let source = src.source;
    let v = &crate::FederalUnemployment::from(data);
    let repaid =
        v.repayments.iter().filter(|p| p.received_year == 2026).map(|p| p.amount).sum::<i64>();

    crate::require_localized!(
        r,
        repaid <= v.amount,
        context,
        source,
        "ISSUE_SAME_YEAR_UNEMPLOYMENT_REPAYMENTS_CANNOT_EXCEED_THIS_SOURCE_S_GROSS_BC0A30D5",
        "Same-year unemployment repayments cannot exceed this source's gross 2026 benefits. Reconcile the receipt years and payment records.",
    );
    let net = phase::floor(v.amount - repaid, 0);
    t.unemployment += net;
    t.injured_unemployment[owner] += net;
    // Same-year repayments reduce this actual source once. Routed
    // grants/farm payments belong to their destination, not twice.
    t.injured_other_amount(e.id, Some(net));
    if v.rtaa != 0 {
        crate::other_income::collect_8z(
            &mut crate::other_income::Ctx {
                totals: t,
                result: r,
                id: e.id,
                owner,
                context: context.clone(),
            },
            "rtaa",
            v.rtaa,
            "EXPLAIN_OTHER_INCOME_N_RTAA",
        );
    }
    let routing = routed_farm(
        d,
        usize::from(e.owner == crate::DocumentOwner::Spouses),
        v.component_ids.agriculture,
        source_claims,
    );
    // Recipient corrections use the original accepted catalog, independently
    // of this working copy's retained activities.
    for (available, key, corrective, _english) in
        farm_receipt_requirements(v, routing, ccc_market_gain_reconciled(d, e.owner))
            .into_iter()
            .take(1)
    {
        let issue_context = if corrective {
            context.clone().corrective(crate::TaxSection::Farming)
        } else {
            context.clone()
        };
        crate::require_localized!(r, available, &issue_context, source, key, _english,);
    }
    // A grant to you is line 8z; one to a business you run is that
    // business's receipts (information_returns::route).
    if matches!(v.grant_use, Some(crate::FederalGrantUse::You)) {
        t.taxable_grants += v.taxable_grants;
        t.injured_other_amount(e.id, Some(v.taxable_grants));
    }
    // A medical-leave share (Yes) or a possible one (Not sure)
    // is not separated: the whole box 10 amount is taxed.
    t.family_leave += v.family_leave;
    t.injured_other_amount(e.id, Some(v.family_leave));
    if v.family_leave > 0 && v.family_leave_includes_medical {
        note_once_scoped(r, msg!("WARN_FAMILY_LEAVE_TAXED_IN_FULL"), context.clone());
    }
    emit_lines(
        r,
        &mut (),
        [
            ("gross", v.amount, msg!("EXPLAIN_UNEMPLOYMENT_N_N_GROSS")),
            ("repaid", repaid, msg!("EXPLAIN_UNEMPLOYMENT_N_N_REPAID")),
            ("taxable", net, msg!("EXPLAIN_UNEMPLOYMENT_N_N_TAXABLE")),
            ("taxable_grants", v.taxable_grants, msg!("EXPLAIN_UNEMPLOYMENT_N_N_TAXABLE_GRANTS")),
            (
                "agriculture_payment",
                v.agriculture_payment,
                msg!("EXPLAIN_UNEMPLOYMENT_N_N_AGRICULTURE_PAYMENT"),
            ),
            ("family_leave", v.family_leave, msg!("EXPLAIN_UNEMPLOYMENT_N_N_FAMILY_LEAVE")),
        ],
        |(suffix, amount, note), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("unemployment_{}_{suffix}", e.id),
                LineValue::MoneyCents(amount),
                &[&alloc::format!("income.{}", e.id)],
                "https://www.irs.gov/publications/p525",
                note,
            ))
        },
    );
    Ok(())
}
