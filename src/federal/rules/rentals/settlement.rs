use super::*;

pub(crate) struct SettlementInput<'input, 'source> {
    pub(crate) values: &'input [PassiveEntry<'source>],
    pub(crate) personal_inputs: &'input alloc::collections::BTreeMap<i64, PersonalRentalInputs>,
    pub(crate) initial_issues: usize,
    pub(crate) magi: i64,
    pub(crate) personal_profit: i64,
    pub(crate) personal_profit_cents: i64,
    pub(crate) nonpassive_rental_losses: bool,
    pub(crate) released_rental_losses: bool,
}

pub(crate) struct SettlementOutput<'pool> {
    pub(crate) inputs_complete: bool,
    pub(crate) pool_complete: bool,
    pub(crate) together: bool,
    pub(crate) line5: i64,
    pub(crate) line6: i64,
    pub(crate) line7: i64,
    pub(crate) allowance: i64,
    pub(crate) net_cents: Vec<i64>,
    pub(crate) prior_cents: Vec<i64>,
    pub(crate) ineligible_cents: Vec<i64>,
    pub(crate) losses: Vec<i64>,
    pub(crate) suspended: Vec<i64>,
    pub(crate) allocation:
        crate::kernel::loss_limits::PoolSettlement<'pool, i64, crate::kernel::carry::Cents>,
    pub(crate) special_allowance: i64,
    pub(crate) combined_cents: i64,
    pub(crate) health_ptc: HealthPassiveProjection,
}

pub(crate) enum SettlementStop {
    Source(crate::activity_risk::SourceIssue),
    Pool(crate::kernel::loss_limits::LossLimitError),
}

pub(crate) fn settle<'pool>(
    context: &rental_setup::RentalContext<'_, '_>,
    input: SettlementInput<'_, '_>,
    pool: &'pool mut Vec<crate::kernel::loss_limits::PassiveActivity<i64>>,
    r: &mut FederalResult,
) -> Result<SettlementOutput<'pool>, SettlementStop> {
    let d = context.form.draft;
    let amt = context.amt;
    let prefix = if amt { "amt_" } else { "" };
    let SettlementInput {
        values,
        personal_inputs,
        initial_issues,
        magi,
        personal_profit,
        personal_profit_cents,
        nonpassive_rental_losses,
        released_rental_losses,
    } = input;
    let inputs_complete = r.issues.len() == initial_issues;
    line(
        r,
        &alloc::format!("{prefix}rental_magi"),
        magi,
        &["income.non_rental", "adjustments"],
        PASSIVE,
        msg!("EXPLAIN_0RENTAL_MAGI"),
    );
    let pool_complete =
        r.issues.iter().all(|i| !amt && i.context == crate::IssueContext::ScorpAlternativeMinimum);
    let s = d.setup.as_ref().unwrap();
    let separate = s.filing_status() == FederalStatus::Separate;
    // Form 8582 Part I caution: married filing separately and lived with the
    // spouse at any time in the year, skip Part II (go to line 10), and use
    // Part V rather than Part IV (2025 i8582).
    let together = separate && !s.lived_apart_all_year();
    // Lines 5 through 8: $150,000 ($75,000 separate) less MAGI that is not
    // below zero (line 6), half of it (line 8, a half dollar rounds up), at
    // most $25,000 ($12,500 separate), section 469(i)(2), (3)(A), (5).
    let line5 = if separate {
        crate::year_constants::PASSIVE_ALLOWANCE_PHASEOUT_END_SEPARATE_DOLLARS
    } else {
        crate::year_constants::PASSIVE_ALLOWANCE_PHASEOUT_END_DOLLARS
    };
    let line6 = positive(magi);
    let line7 = positive(line5 - line6);
    let allowance = if together {
        0
    } else {
        phase::cap(
            if separate {
                crate::year_constants::PASSIVE_ALLOWANCE_SEPARATE_DOLLARS
            } else {
                crate::year_constants::PASSIVE_ALLOWANCE_DOLLARS
            },
            [phase::offset_steps(line7, 1, 2)],
        )
    };
    line(
        r,
        &alloc::format!("{prefix}personal_rental_passive_income"),
        personal_profit,
        &["income.nonbusiness_property"],
        PASSIVE,
        msg!("EXPLAIN_0PERSONAL_RENTAL_PASSIVE_INCOME"),
    );
    let working = values
        .iter()
        .map(|entry| {
            if entry.personal {
                let input = &personal_inputs[&entry.id];
                Ok((
                    input.receipts_cents - input.expenses_cents - input.prior_cents,
                    input.prior_cents,
                    0,
                ))
            } else if let Some(source) = &entry.source_cents {
                Ok((source.net, source.prior, source.ineligible_prior))
            } else {
                Ok((
                    annual_dollars_to_working_cents(entry.net)?,
                    annual_dollars_to_working_cents(entry.prior)?,
                    annual_dollars_to_working_cents(entry.ineligible_prior)?,
                ))
            }
        })
        .collect::<Result<Vec<_>, crate::kernel::loss_limits::LossLimitError>>();
    let working = match working {
        Ok(value) => value,
        Err(error) => {
            let issue = crate::activity_risk::SourceIssue::new(0, error.into());
            issue.report(r);
            return Err(SettlementStop::Source(issue));
        }
    };
    let net_cents: Vec<_> = working.iter().map(|value| value.0).collect();
    let prior_cents: Vec<_> = working.iter().map(|value| value.1).collect();
    let ineligible_cents: Vec<_> = working.iter().map(|value| value.2).collect();
    let losses: alloc::vec::Vec<i64> = values.iter().map(|entry| positive(-entry.net)).collect();
    // Direct/farm rental history passes activity_risk's source check;
    // K-1 owners validate their carried losses before constructing activities.
    // Their ineligible prior is a remaining prior loss, never a signed net.
    *pool = values
        .iter()
        .enumerate()
        .map(|(index, entry)| crate::kernel::loss_limits::PassiveActivity {
            key: entry.id,
            net: net_cents[index],
            rental_allowance: entry.rental_allowance && !together,
            ineligible_prior: ineligible_cents[index],
        })
        .collect();
    let allocation = crate::kernel::loss_limits::passive_pool(SourceFact::Reported(
        crate::kernel::loss_limits::PoolInput {
            activities: pool,
            other_passive_income: crate::kernel::carry::Amount::<crate::kernel::carry::Cents>::new(
                personal_profit_cents,
            )
            .expect("native passive income is nonnegative"),
            allowance: crate::kernel::carry::Amount::new(allowance * 100)
                .expect("native passive allowance is nonnegative"),
        },
    ));
    let allocation = match allocation {
        Ok(SourceFact::Reported(value)) => value,
        Ok(_) => unreachable!("reported pool remains reported"),
        Err(error @ crate::kernel::loss_limits::LossLimitError::Overflow) => {
            // Actual source sums exceeding the native pool receiver are
            // unavailable; do not panic or impose an entered-money ceiling.
            crate::issue_localized!(
                r,
                &crate::IssueContext::Section(crate::TaxSection::PassThrough),
                "Partnership activity allocation",
                "ISSUE_K1_ACTIVITY_OWNER_OUTPUT_UNAVAILABLE",
                "This estimate does not yet calculate the combined basis, at-risk and passive limits for these activity records."
            );
            return Err(SettlementStop::Pool(error));
        }
        Err(error) => panic!("validated rental amounts fit the passive allocation: {error:?}"),
    };
    let suspended = allocation
        .annual_suspensions()
        .expect("validated rental amounts fit the annual passive projection");
    // The losses the passive income offsets come first; the special allowance
    // lets the rest through (the shared passive_pool settlement).
    let passive_profit_cents =
        personal_profit_cents + net_cents.iter().map(|net| positive(*net)).sum::<i64>();
    let special_allowance = dollars(allocation.special_allowance());
    let active: i64 = values
        .iter()
        .filter(|e| e.rental_allowance && !together)
        .map(|e| {
            e.source_cents.as_ref().map_or((e.net + e.ineligible_prior) * 100, |source| {
                source.net + source.ineligible_prior
            })
        })
        .sum();
    let combined_cents = personal_profit_cents + net_cents.iter().sum::<i64>();
    let health_ptc = HealthPassiveProjection {
        deferred: active < 0 && combined_cents < 0,
        rentals: values
            .iter()
            .zip(&suspended)
            .enumerate()
            .filter_map(|(index, (entry, carried))| {
                if !entry.real_estate() {
                    return None;
                }
                let current_cents =
                    entry.source_cents.as_ref().map_or(
                        (entry.net - entry.disposition_gain + entry.prior) * 100,
                        |source| source.net - source.disposition_gain + source.prior,
                    );
                let prior_cents =
                    entry.source_cents.as_ref().map_or(entry.prior * 100, |source| source.prior);
                let held_cents = entry.source_cents.as_ref().map_or(*carried * 100, |_| {
                    // Same selected activity slot as the passive settlement.
                    allocation.suspended()[index]
                });
                Some(HealthRentalActivity {
                    source: entry.id,
                    current: dollars(current_cents),
                    prior: dollars(prior_cents),
                    allowed_loss: dollars(positive(-current_cents) + prior_cents - held_cents),
                })
            })
            .collect(),
        mixed_losses: values
            .iter()
            .enumerate()
            .any(|(index, e)| !e.real_estate() && (net_cents[index] < 0 || prior_cents[index] > 0))
            && (passive_profit_cents > 0
                || values.iter().any(|e| {
                    e.real_estate()
                        && e.source_cents
                            .as_ref()
                            .map_or((e.net - e.disposition_gain + e.prior) * 100, |source| {
                                source.net - source.disposition_gain + source.prior
                            })
                            > 0
                })),
        // The special list names passive losses, whereas WorksheetX line1
        // says any RE rental losses. No inspected primary example resolves
        // their intersection with a concurrent nonpassive professional loss.
        nonpassive_losses: nonpassive_rental_losses
            || values.iter().any(|e| {
                e.real_estate()
                    && e.source_cents
                        .as_ref()
                        .map_or(e.nonpassive * 100, |source| source.nonpassive)
                        < 0
            }),
        released_outside_pool: values.iter().any(|e| {
            e.real_estate()
                && e.source_cents
                    .as_ref()
                    .map_or(e.disposition_gain != 0 || e.prior_released > 0, |source| {
                        source.disposition_gain != 0 || source.prior_released > 0
                    })
        }) || released_rental_losses,
    };
    Ok(SettlementOutput {
        inputs_complete,
        pool_complete,
        together,
        line5,
        line6,
        line7,
        allowance,
        net_cents,
        prior_cents,
        ineligible_cents,
        losses,
        suspended,
        allocation,
        special_allowance,
        combined_cents,
        health_ptc,
    })
}
