//! Reuse the annual deduction owners for the hypothetical Form4562 stage.
//! Pub.946, Business Income Limit and Two different taxable income limits:
//! https://www.irs.gov/publications/p946
use crate::{
    FederalDraft, FederalIncome, FederalResult, Owner, Section179PreparedAllocation,
    Section179PreparedIssue as Issue,
};
use alloc::collections::{BTreeMap, BTreeSet};

pub(crate) struct Deductions {
    pub amount_cents: i64,
    pub sources: BTreeSet<i64>,
    pub regular_half: [i64; 2],
}

// These dependencies need their actual return-stage suppliers before the
// existing refusal can be removed. No unsupported source becomes zero.
fn sources(d: &FederalDraft, selected: &Section179PreparedAllocation) -> Option<BTreeSet<i64>> {
    if d.setup.as_ref().is_some_and(crate::community::applies)
        || d.screening.as_ref().is_some_and(|s| {
            !s.clergy.is_empty() || s.optional_se.iter().any(|e| e.farm || e.nonfarm.is_some())
        })
    {
        return None;
    }
    let mut sources = BTreeSet::new();
    let mut health = false;
    for entry in &d.income {
        match &entry.income {
            FederalIncome::Business { data } => {
                if selected.participation.get(entry.id) != crate::participation::Participation::Yes
                    || !data.all_at_risk()
                    || !matches!(data.source, crate::BusinessSource::UnitedStates)
                    || !matches!(
                        data.activity,
                        crate::BusinessActivity::Ordinary { .. }
                            | crate::BusinessActivity::Farming { .. }
                    )
                    || crate::clergy::business(data)
                {
                    return None;
                }
                let has_health =
                    !matches!(data.health_insurance, crate::BusinessHealthInsurance::None);
                health |= has_health;
                if has_health || !matches!(data.retirement, crate::BusinessRetirement::None) {
                    sources.insert(entry.id);
                }
            }
            FederalIncome::EmployeeWages { .. }
            | FederalIncome::Partnership { .. }
            | FederalIncome::Scorp { .. }
            | FederalIncome::PartnershipActivities { .. }
            | FederalIncome::ScorpActivities { .. } => return None,
            FederalIncome::Wage { data }
                if crate::FederalWage::from(data).coverage == crate::W2Coverage::Ministerial =>
            {
                return None;
            }
            _ => {}
        }
    }
    if sources.is_empty()
        || (health
            && d.adjustments.as_ref().is_some_and(|a| !a.health_marketplace.policies.is_empty()))
    {
        return None;
    }
    Some(sources)
}

pub(super) struct RegularEarnedSources {
    pub payroll: alloc::boxed::Box<crate::federal_rules::Totals>,
    pub profit: [i64; 2],
    pub half: [i64; 2],
}

// The W-2 owner retains physical payroll coverage, tips and railroad wages.
// These operands precede, and do not depend on, the hypothetical deduction.
pub(super) fn regular_earned_sources(
    d: &FederalDraft,
    selected: &Section179PreparedAllocation,
    claims: &BTreeMap<i64, i64>,
    partners: &[(i64, crate::partnership::PartnerSeResult)],
    r: &mut FederalResult,
) -> Result<RegularEarnedSources, Issue> {
    let mut payroll = crate::federal_rules::boxed(crate::federal_rules::Totals::default);
    for entry in &d.income {
        if let FederalIncome::Wage { data } = &entry.income {
            let data = &crate::FederalWage::from(data);
            crate::forms::w2::collect(
                &mut crate::forms::w2::WageContext {
                    totals: &mut payroll,
                    result: r,
                    owner: usize::from(entry.owner == crate::DocumentOwner::Spouses),
                    context: crate::interview::income_issue_context(entry),
                    #[cfg(test)]
                    source: "Section 179 payroll",
                },
                data,
            );
        }
    }
    // Form 8919 pay follows the W-2 wages, as in the return collector; the
    // care owner admits only Form 8919 payroll sources.
    for entry in &d.income {
        if let FederalIncome::EmployeeWages { data } = &entry.income {
            crate::federal_rules::employee_wages_into(
                d,
                usize::from(entry.owner == crate::DocumentOwner::Spouses),
                &crate::interview::income_issue_context(entry),
                #[cfg(test)]
                "Section 179 Form 8919 pay",
                &mut payroll,
                r,
                data,
            );
        }
    }
    if !r.issues.is_empty() {
        return Err(Issue::OwnerIssues(core::mem::take(&mut r.issues)));
    }
    let mut half = [0; 2];
    let mut profit = [0; 2];
    for (i, owner) in [Owner::Taxpayer, Owner::Spouse].into_iter().enumerate() {
        // Schedule SE line 8c: Form 8919 line 10 wages join the wage base.
        let social = payroll.social_wages[i]
            .checked_add(payroll.employee_tips[i].social)
            .and_then(|v| v.checked_add(payroll.employee_8919[i]))
            .ok_or(Issue::AggregateAmountOverflow)?;
        let se = crate::partnership::owner_actual_se_with_section179_with_claims(
            crate::related_forms::RelatedFormContext::bound(d, claims, &selected.participation),
            owner,
            social,
            payroll.church_wages[i],
            partners,
            Some(selected),
            Some(&selected.home_context),
        )
        .map_err(|issue| match issue {
            crate::partnership::OwnerSeIssue::Prepared(issue) => issue,
            crate::partnership::OwnerSeIssue::Rule((key, _)) => {
                Issue::SourceBoundaryUnavailable(0, key)
            }
        })?;
        half[i] = se.half;
        profit[i] = se.profit;
    }
    Ok(RegularEarnedSources { payroll, profit, half })
}

// Zero regular availability skips the allocator callback, but its actual
// zero-179 selected expense source still owns ScheduleSE for an AMT carry.
pub(crate) fn zero_allowance_half(
    d: &FederalDraft,
    selected: &Section179PreparedAllocation,
    claims: &BTreeMap<i64, i64>,
) -> Result<Option<[i64; 2]>, Issue> {
    if sources(d, selected).is_none() {
        return Ok(None);
    }
    let mut r = crate::federal_rules::boxed(|| FederalResult::empty(d.revision));
    regular_earned_sources(d, selected, claims, &[], &mut r).map(|source| Some(source.half))
}

pub(crate) fn calculate(
    d: &FederalDraft,
    selected: &Section179PreparedAllocation,
    claims: &BTreeMap<i64, i64>,
    regular: Option<[i64; 2]>,
) -> Result<Option<Deductions>, Issue> {
    let Some(sources) = sources(d, selected) else {
        return Ok(None);
    };
    let mut r = crate::federal_rules::boxed(|| FederalResult::empty(d.revision));
    let half = if selected.income.system == crate::Section179IncomeSystem::Regular {
        regular_earned_sources(d, selected, claims, &[], &mut r)?.half
    } else {
        let Some(regular) = regular else {
            return Ok(None);
        };
        regular
    };
    let sep = crate::sep::calculate_hypothetical_with_section179_with_claims(
        crate::related_forms::RelatedFormContext::bound(d, claims, &selected.participation),
        half,
        None,
        &[],
        selected,
        &mut r,
        Some(&selected.home_context),
    )?;
    let public_safety =
        crate::public_safety::calculate(d, &mut r).map_err(|issue| match issue {
            crate::health::CalculationIssue::Prepared(issue) => issue,
            issue @ crate::health::CalculationIssue::Source(_) => {
                issue.report(&mut r);
                Issue::OwnerIssues(core::mem::take(&mut r.issues))
            }
        })?;
    let health = crate::health::calculate_with_money_with_claims(
        d,
        half,
        None,
        &BTreeMap::new(),
        &sep,
        &public_safety,
        None,
        crate::marketplace::CalculationMoney::Dollars,
        Some(selected),
        &mut r,
        Some(&selected.home_context),
        claims,
    )
    .map_err(|issue| match issue {
        crate::health::CalculationIssue::Prepared(issue) => issue,
        issue @ crate::health::CalculationIssue::Source(_) => {
            issue.report(&mut r);
            Issue::OwnerIssues(core::mem::take(&mut r.issues))
        }
    })?;
    if !r.issues.is_empty() {
        return Err(Issue::OwnerIssues(core::mem::take(&mut r.issues)));
    }
    let dollars = crate::kernel::ratio::checked_sum(sources.iter().flat_map(|id| {
        [
            sep.by_business.get(id).copied().unwrap_or(0),
            health.by_business.get(id).copied().unwrap_or(0),
        ]
    }))
    .ok_or(Issue::AggregateAmountOverflow)?;
    Ok(Some(Deductions {
        amount_cents: dollars.checked_mul(100).ok_or(Issue::AggregateAmountOverflow)?,
        sources,
        regular_half: half,
    }))
}
