use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
pub use crate::optional_se_source::{NonfarmSeHistory, SeElection};
use crate::{BusinessActivity, FederalDraft, FederalIncome, FederalResult, Owner};
use alloc::vec::Vec;

#[derive(Default)]
pub(crate) struct OptionalSe {
    pub net: i64,
    pub eic_adjustment: i64,
    pub child_adjustment: i64,
    // Ministerial wages/housing are SE income, not a business deduction source.
    pub nonbusiness_net: i64,
    pub weights: Vec<(i64, i64)>,
    // The same filed optional earnings total, attributed using source cents.
    // This is not an independent optional-method limit or SE-tax computation.
    pub weights_cents: Vec<(i64, i64)>,
    pub lines: OptionalLines,
}
// What an elected optional method changes on the printed Schedule SE (whole
// dollars): a farm method skips lines 1a and 1b, the nonfarm method skips line 2,
// line 3 holds only the actual profit of the method not elected, and line 4b is
// the total of lines 15 and 17.
#[derive(Clone, Copy, Default)]
pub(crate) struct OptionalLines {
    pub farm: bool,
    pub nonfarm: bool,
    pub regular_profit: i64,
    pub optional_earnings: i64,
}
#[cfg(test)]
pub(crate) fn calculate(
    d: &FederalDraft,
    owner: Owner,
    r: &mut FederalResult,
) -> Option<OptionalSe> {
    let source_claims = crate::related_forms::effective_claimants(d);
    calculate_with_claims(d, owner, r, &source_claims)
}

#[cfg(test)]
pub(crate) fn calculate_with_claims(
    d: &FederalDraft,
    owner: Owner,
    r: &mut FederalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<OptionalSe> {
    calculate_with_partners_with_claims(d, owner, None, r, source_claims)
}

#[cfg(test)]
pub(crate) fn calculate_with_partners(
    d: &FederalDraft,
    owner: Owner,
    partners: Option<&[(i64, crate::partnership::PartnerSeResult)]>,
    r: &mut FederalResult,
) -> Option<OptionalSe> {
    let source_claims = crate::related_forms::effective_claimants(d);
    calculate_with_partners_with_claims(d, owner, partners, r, &source_claims)
}

#[cfg(test)]
pub(crate) fn calculate_with_partners_with_claims(
    d: &FederalDraft,
    owner: Owner,
    partners: Option<&[(i64, crate::partnership::PartnerSeResult)]>,
    r: &mut FederalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<OptionalSe> {
    match calculate_with_partners_with_section179_with_claims(
        d,
        owner,
        partners,
        None,
        r,
        None,
        source_claims,
    ) {
        Ok(value) => value,
        Err(_) => unreachable!("legacy optional-SE calculation has no selected allocation"),
    }
}

pub(crate) fn calculate_with_section179_with_claims(
    d: &FederalDraft,
    owner: Owner,
    selected: Option<&crate::Section179PreparedAllocation>,
    r: &mut FederalResult,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Option<OptionalSe>, crate::Section179PreparedIssue> {
    calculate_with_partners_with_section179_with_claims(
        d,
        owner,
        None,
        selected,
        r,
        home,
        source_claims,
    )
}

pub(crate) fn calculate_with_partners_with_section179_with_claims(
    d: &FederalDraft,
    owner: Owner,
    partners: Option<&[(i64, crate::partnership::PartnerSeResult)]>,
    selected: Option<&crate::Section179PreparedAllocation>,
    r: &mut FederalResult,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Option<OptionalSe>, crate::Section179PreparedIssue> {
    if crate::clergy::calculate_with_section179_with_claims(
        d,
        owner,
        selected,
        home,
        source_claims,
    )?
    .sect
    {
        return Ok(None);
    }
    if owner == Owner::Spouse {
        let Some(setup) = d.setup.as_ref() else {
            return Ok(None);
        };
        if !crate::federal_rules::joint(setup.filing_status()) {
            return Ok(None);
        }
    }
    let Some(screening) = d.screening.as_ref() else {
        return Ok(None);
    };
    let elections = &screening.optional_se;
    let Some(election) = elections.iter().find(|e| e.owner == owner) else {
        return Ok(None);
    };
    if elections.iter().filter(|e| e.owner == owner).count() != 1 {
        crate::federal_rules::note_once(r, msg!("WARN_OPTIONAL_METHOD_NOT_AVAILABLE"));
        return Ok(None);
    }
    if !election.farm && election.nonfarm.is_none() {
        return Ok(None);
    }
    let farm = election.farm;
    let nonfarm = election.nonfarm.is_some();
    let mut gross = [0; 2];
    let mut profit = [0; 2];
    let mut count = [0; 2];
    // One source list drives both election totals and deduction attribution.
    // Values remain cents until the existing per-category rounding boundary.
    let mut sources = Vec::new();
    for e in d.income.iter().filter(|e| e.owner == crate::DocumentOwner::from(owner)) {
        let source = match &e.income {
            FederalIncome::Business { data }
                if !(crate::statutory_wages::business(d, e.id)
                    || crate::clergy::calculate_with_section179_with_claims(
                        d,
                        owner,
                        selected,
                        home,
                        source_claims,
                    )?
                    .exempt
                        && crate::clergy::business(data)) =>
            {
                let category =
                    usize::from(!matches!(data.activity, BusinessActivity::Farming { .. }));
                // A farm's gross income (Schedule F line 9) and profit include the Form
                // 1099-G box 7 and 1099-PATR income routed to it.
                let routed = crate::federal_rules::farm_routed_income_with_claims(
                    d,
                    usize::from(owner == Owner::Spouse),
                    e.id,
                    source_claims,
                )?;
                // Neither totalization-exempt income nor notary services
                // supplies optional-method Social Security coverage.
                if matches!(
                    data.source,
                    crate::BusinessSource::ForeignServices { totalization_exempt: true, .. }
                ) {
                    continue;
                }
                if data.notary_profit > 0 && data.notary_gross.is_none() {
                    crate::federal_rules::note_once(r, msg!("WARN_OPTIONAL_METHOD_NOT_AVAILABLE"));
                    return Ok(None);
                }
                Some((
                    category,
                    data.receipts() + routed
                        - data.notary_gross.unwrap_or(0)
                        - if category == 1 { data.inventory.cost() } else { 0 },
                    data.earned_receipts() + routed
                        - crate::section179_return::expenses_with_claims(
                            d,
                            e.id,
                            data,
                            selected,
                            home,
                            source_claims,
                        )
                        .map(|expense| {
                            expense
                                - selected.map_or(0, |allocation| {
                                    allocation.ordinary_limit_adjustment(e.id)
                                })
                        })?
                        - crate::clergy::business_se_exclusion_with_section179_with_claims(
                            d,
                            e.id,
                            data,
                            selected,
                            home,
                            source_claims,
                        )?,
                ))
            }
            FederalIncome::Partnership { data } => {
                let resolved = partners.and_then(|items| {
                    let mut matching = items.iter().filter(|(id, _)| *id == e.id);
                    let first = matching.next()?;
                    matching.next().is_none().then_some(&first.1)
                });
                let input = data.se_gross.as_ref().map(|v| v.source());
                if let (Some(earnings), Some((category, gross))) = (resolved, input)
                    && (data.guaranteed_se()..=100000000000).contains(&gross)
                    && (data.limited_for_se == Some(false) || gross == data.guaranteed_services)
                {
                    Some((category, gross, earnings.earnings * 100))
                } else {
                    crate::require_localized!(
                        r,
                        false,
                        &crate::IssueContext::Section(crate::TaxSection::PassThrough),
                        "Partnership optional SE",
                        "ISSUE_RESOLVE_THIS_PARTNERSHIP_S_REGULAR_SE_EARNINGS_AND_K_1_BOX_14_B_C_G_E76F6153",
                        "Resolve this partnership's regular SE earnings and K-1 box 14 B/C gross-income category. Ordinary limited-partner income is not optional-method gross income; include entered guaranteed service payments in gross income.",
                    );
                    return Ok(None);
                }
            }
            FederalIncome::PartnershipActivities { data } => {
                let resolved = partners.and_then(|items| {
                    let mut matching = items.iter().filter(|(id, _)| *id == e.id);
                    let first = matching.next()?;
                    matching.next().is_none().then_some(&first.1)
                });
                match data.ordinary_self_employment() {
                    Ok(None) => None,
                    Ok(Some(source)) if resolved == Some(&source.net) => {
                        Some((source.category, source.gross, source.net.earnings_cents))
                    }
                    _ => {
                        crate::require_localized!(
                            r,
                            false,
                            &crate::interview::income_issue_context(e),
                            "Partnership optional SE",
                            "ISSUE_RESOLVE_THIS_PARTNERSHIP_S_REGULAR_SE_EARNINGS_AND_K_1_BOX_14_B_C_G_E76F6153",
                            "Resolve this partnership's regular SE earnings and K-1 box 14 B/C gross-income category. Ordinary limited-partner income is not optional-method gross income; include entered guaranteed service payments in gross income.",
                        );
                        return Ok(None);
                    }
                }
            }
            _ => None,
        };
        if let Some((category, source_gross, source_profit)) = source {
            count[category] += 1;
            gross[category] += source_gross;
            profit[category] += source_profit;
            sources.push((e.id, category, source_gross, source_profit));
        }
    }
    // Pub.517 nonfarm optional method uses all ministerial gross and net,
    // including church-employee ministerial wages and the full housing amount.
    // They have no business identity and never receive a business deduction.
    let clergy = crate::clergy::calculate_with_section179_with_claims(
        d,
        owner,
        selected,
        home,
        source_claims,
    )?;
    let clergy_gross = clergy.nonbusiness_gross();
    let clergy_profit = clergy.extra_se();
    if clergy_gross != 0 || clergy_profit != 0 {
        count[1] += 1;
        gross[1] += clergy_gross;
        profit[1] += clergy_profit;
    }
    let gross = gross.map(dollars);
    let profit = profit.map(dollars);
    let farm_optional = if farm {
        phase::cap(
            fraction(positive(gross[0]), 2, 3),
            [crate::year_constants::OPTIONAL_SE_MAXIMUM_DOLLARS],
        )
    } else {
        0
    };
    let nonfarm_optional = if nonfarm {
        phase::cap(
            fraction(positive(gross[1]), 2, 3),
            [crate::year_constants::OPTIONAL_SE_MAXIMUM_DOLLARS - farm_optional],
        )
    } else {
        0
    };
    let mut eligible = true;
    if farm {
        eligible &= count[0] > 0
            && (gross[0] <= crate::year_constants::OPTIONAL_SE_FARM_GROSS_LIMIT_DOLLARS
                || profit[0] < crate::year_constants::OPTIONAL_SE_NONFARM_PROFIT_LIMIT_DOLLARS);
    }
    if let Some(history) = &election.nonfarm {
        let history_ok = [
            history.earnings_third_prior_year,
            history.earnings_second_prior_year,
            history.earnings_prior_year,
        ]
        .iter()
        .filter(|v| **v >= 40000)
        .count()
            >= 2
            && history.valid();
        eligible &= count[1] > 0
            && profit[1] < crate::year_constants::OPTIONAL_SE_NONFARM_PROFIT_LIMIT_DOLLARS
            && profit[1] * 100000 < gross[1] * 72189
            && history_ok;
        eligible &= nonfarm_optional + farm_optional >= fraction(positive(profit[1]), 9235, 10000);
    }
    if !eligible {
        crate::federal_rules::note_once(r, msg!("WARN_OPTIONAL_METHOD_NOT_AVAILABLE"));
        return Ok(None);
    }
    let regular = if farm { 0 } else { profit[0] } + if nonfarm { 0 } else { profit[1] };
    let regular_net = if regular > 0 { fraction(regular, 9235, 10000) } else { regular };
    let net = positive(regular_net + farm_optional + nonfarm_optional);
    let tag = if owner == Owner::Taxpayer { "taxpayer" } else { "spouse" };
    emit_lines(
        r,
        &mut (),
        [
            ("farm", farm_optional, msg!("EXPLAIN_OPTIONAL_SE_N_N_FARM")),
            ("nonfarm", nonfarm_optional, msg!("EXPLAIN_OPTIONAL_SE_N_N_NONFARM")),
            ("net", net, msg!("EXPLAIN_OPTIONAL_SE_N_N_NET")),
        ],
        |(suffix, amount, note), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("optional_se_{tag}_{suffix}"),
                LineValue::MoneyDollars(amount),
                &["screening.optional_se", "income.business", "income.partnership"],
                "https://www.irs.gov/pub/irs-dft/f1040sse--dft.pdf",
                note,
            ))
        },
    );
    // Optional SE earnings change specified credit/health worksheets. Pub. 560,
    // Pub. 590-A and Pub. 969 contribution limits retain actual business income
    // after the applicable deductions; do not substitute this optional amount.
    let mut weights = Vec::new();
    let mut weights_cents = Vec::new();
    let mut nonbusiness_net = 0;
    for (category, selected, optional) in [(0, farm, farm_optional), (1, nonfarm, nonfarm_optional)]
    {
        let entries: Vec<_> = sources.iter().filter(|(_, kind, _, _)| *kind == category).collect();
        let mut source_weights: Vec<_> = entries
            .iter()
            .map(|(_, _, gross, profit)| positive(dollars(if selected { *gross } else { *profit })))
            .collect();
        let has_clergy = category == 1 && (clergy_gross != 0 || clergy_profit != 0);
        if has_clergy {
            source_weights.push(positive(dollars(if selected {
                clergy_gross
            } else {
                clergy_profit
            })));
        }
        let mut amounts = if selected {
            crate::kernel::ratio::allocate_remaining(optional, &source_weights)
        } else {
            source_weights
        };
        if has_clergy {
            let share = amounts.pop().unwrap_or(0);
            nonbusiness_net = if selected { share } else { fraction(share, 9235, 10000) };
        }
        weights.extend(entries.iter().zip(amounts).map(|((id, _, _, _), value)| (*id, value)));
        // Identity determines only residual attribution, never tax treatment.
        // Preserve the existing annual projection while making this cent feed
        // independent of accepted entry order.
        let mut entries_cents = entries.clone();
        entries_cents.sort_by_key(|(id, _, _, _)| *id);
        let mut source_cents: Vec<_> = entries_cents
            .iter()
            .map(|(id, _, gross, profit)| {
                positive(if selected {
                    *gross
                } else {
                    partners
                        .and_then(|items| items.iter().find(|(source, _)| source == id))
                        .map_or(*profit, |(_, source)| source.earnings_cents)
                })
            })
            .collect();
        if has_clergy {
            source_cents.push(positive(if selected { clergy_gross } else { clergy_profit }));
        }
        let mut amounts_cents = if selected {
            crate::kernel::ratio::allocate_remaining(optional * 100, &source_cents)
        } else {
            source_cents
        };
        if has_clergy {
            amounts_cents.pop();
        }
        weights_cents.extend(
            entries_cents.iter().zip(amounts_cents).map(|((id, _, _, _), value)| (*id, value)),
        );
    }
    Ok(Some(OptionalSe {
        nonbusiness_net,
        weights,
        weights_cents,
        net,
        lines: OptionalLines {
            farm,
            nonfarm,
            regular_profit: regular,
            optional_earnings: farm_optional + nonfarm_optional,
        },
        eic_adjustment: regular + farm_optional + nonfarm_optional - profit.iter().sum::<i64>(),
        // Schedule 8812 keeps actual nonfarm profit; farm earnings cannot rise
        // above actual profit, and an actual farm loss remains a loss.
        child_adjustment: if farm { phase::cap(profit[0], [farm_optional]) - profit[0] } else { 0 },
    }))
}
