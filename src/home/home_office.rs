// Canonical home-site calculation and reporting. Accepted sites own each
// physical home and its sources; invocation context owns computed AGI/caps.
use crate::kernel::ratio::exact::Exact;
use crate::{FederalBusiness, FederalDraft, FederalIncome, FederalResult, FederalStatus};

// Form 8829 in cents. The itemizing route's line 11 is column (b) when the
// Line 11 Worksheet's Step 1 applies (all the home's real estate taxes) and
// column (a) under Step 2 (the business share within the state and local tax
// limit, the rest on line 17 column (a)); the standard route has no line 11 and
// puts all the taxes on line 17 column (b).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Lines {
    // Line 7 in hundredths of a percent, for the explanations only.
    pub percent: i64,
    // Whether the itemizing columns were used, and whether Step 1 applied.
    pub itemizes: bool,
    pub step1: bool,
    pub l8: i64,
    // Line 10 column (b): the deductible mortgage interest and mortgage
    // insurance premiums of the home.
    pub l9: i64,
    pub l10: i64,
    pub l11a: i64,
    pub l11b: i64,
    // Line 12: lines 9 + 10 + 11 per column.
    pub l12a: i64,
    pub l12b: i64,
    pub l13: i64,
    pub l14: i64,
    pub l15: i64,
    pub l16: i64,
    pub l17: i64,
    pub l18: i64,
    pub l19: i64,
    pub l20a: i64,
    pub l20b: i64,
    pub l21: i64,
    pub l22: i64,
    pub l23a: i64,
    pub l23b: i64,
    pub l24: i64,
    pub l25: i64,
    pub l26: i64,
    pub l27: i64,
    pub l28: i64,
    pub l29: i64,
    pub l30: i64,
    pub l31: i64,
    pub l32: i64,
    pub l33: i64,
    pub l34: i64,
    pub l35: i64,
    pub l36: i64,
    pub l37: i64,
    pub l38: i64,
    pub l39: i64,
    pub l40: i64,
    // Line 41 in thousandths of a percent (scale 100,000).
    pub l41: i64,
    pub l42: i64,
    pub l43: i64,
    pub l44: i64,
    // Line 11 Worksheet: 6 (business share, off Schedule A line 5b), 7a
    // (personal taxes left) and whether line 8 needed the modified AGI test.
    pub w6: i64,
    pub w7a: i64,
    pub w_magi_test: bool,
}

pub(crate) fn itemizes(d: &FederalDraft) -> bool {
    d.deductions.as_ref().is_some_and(|v| {
        matches!(
            v.choice,
            crate::DeductionSelection::Itemized { .. }
                | crate::DeductionSelection::Automatic {
                    expenses: crate::ItemizedChoice::Yes { .. }
                }
        )
    })
}

pub(crate) fn with_route(d: &FederalDraft, itemize: bool) -> FederalDraft {
    let mut working = d.clone();
    if let Some(ded) = working.deductions.as_mut()
        && let crate::DeductionSelection::Automatic {
            expenses: crate::ItemizedChoice::Yes { facts },
        } = &ded.choice
    {
        ded.choice = if itemize {
            crate::DeductionSelection::Itemized { facts: facts.clone() }
        } else {
            crate::DeductionSelection::Automatic { expenses: crate::ItemizedChoice::No }
        };
    }
    working
}

fn separate(d: &FederalDraft) -> bool {
    d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Separate)
}

// Non-home property losses remain with their actual business source.
fn losses(d: &FederalDraft, b: &FederalBusiness) -> Result<i64, &'static str> {
    let mid_quarter = crate::depreciation::mid_quarter(d);
    let mut losses = 0i64;
    for sale in b.assets.iter().filter_map(|asset| asset.sale_for(mid_quarter, false)) {
        if sale.gain < 0 {
            let loss = sale
                .gain
                .checked_neg()
                .and_then(|value| value.checked_mul(100))
                .ok_or("HOME_OUTPUT_UNAVAILABLE")?;
            losses = losses.checked_add(loss).ok_or("HOME_OUTPUT_UNAVAILABLE")?;
        }
    }
    Ok(losses)
}

pub(crate) fn has_actual(draft: &FederalDraft) -> bool {
    draft.home_sites.iter().any(|site| {
        site.claims.iter().any(|claim| {
            claim.qualifies() && matches!(claim.method, crate::OfficeMethod::Actual { .. })
        })
    })
}

pub(crate) fn uses_activity(draft: &FederalDraft, source: i64) -> bool {
    draft.home_sites.iter().any(|site| {
        site.claims.iter().any(|claim| {
            claim.qualifies() && claim.activities.iter().any(|activity| activity.activity == source)
        })
    })
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ScheduleAShares {
    pub whole_home_taxes_cents: i64,
    pub interest_cents: [Exact; 2],
    pub taxes: Exact,
    pub points_cents: [Exact; 2],
    pub insurance_premiums_cents: Exact,
}
// Schedule A facts the itemizing columns rely on, in whole dollars except
// where noted.
pub(crate) struct ScheduleAFacts {
    pub magi: i64,
    pub limited_interest: i64,
    pub paid_interest: i64,
    pub loan_interest_cents: i64,
    pub property_taxes_cents: i64,
    pub mortgage_credit: i64,
    pub estate_taxes: i64,
}
// Transient invocation facts observed by the existing complete-return pass.
// Default is only the first solver input; it is never accepted source data.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct HomeContext {
    pub agi_dollars: i64,
    pub salt_magi_dollars: Option<i64>,
    pub salt_cap_cents: i64,
    pub modified_agi_cents: alloc::collections::BTreeMap<i64, i64>,
}
impl HomeContext {
    pub(crate) fn stable_with(&self, next: &Self) -> bool {
        self.salt_magi_dollars
            .zip(next.salt_magi_dollars)
            .is_some_and(|(previous, observed)| previous.abs_diff(observed) <= 1)
            && self.salt_cap_cents == next.salt_cap_cents
            && self.modified_agi_cents.len() == next.modified_agi_cents.len()
            && self.modified_agi_cents.iter().all(|(site, amount)| {
                next.modified_agi_cents.get(site).is_some_and(|next| amount.abs_diff(*next) <= 100)
            })
    }
}

pub(crate) fn next_context(
    draft: &FederalDraft,
    agi_dollars: i64,
    salt_magi_dollars: i64,
    salt_cap_cents: i64,
    selected: Option<&crate::Section179PreparedAllocation>,
    current: &HomeContext,
) -> Result<HomeContext, crate::Section179PreparedIssue> {
    // The Line 11 Worksheet stops on successive observed SALT MAGI, keeping
    // this pass's results and its input cap rather than applying another cap.
    let salt_cap_cents = if current
        .salt_magi_dollars
        .is_some_and(|previous| previous.abs_diff(salt_magi_dollars) <= 1)
    {
        current.salt_cap_cents
    } else {
        salt_cap_cents
    };
    let mut result = HomeContext {
        agi_dollars,
        salt_magi_dollars: Some(salt_magi_dollars),
        salt_cap_cents,
        modified_agi_cents: Default::default(),
    };
    let profits = invocation_profits(draft, selected, current)?;
    let calculations =
        site_calculation::calculate(draft, &profits, current).map_err(|(site, key)| {
            crate::Section179PreparedIssue::SourceBoundaryUnavailable(site, key)
        })?;
    for site in &draft.home_sites {
        let mut excluded_net = 0i64;
        for calculation in calculations.iter().filter(|value| value.site == site.id) {
            let home_deduction =
                crate::kernel::ratio::checked_sum(calculation.deductions.values().copied())
                    .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
            let casualty = calculation.casualty.allowed_casualty_cents().ok_or(
                crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                    site.id,
                    "HOME_CASUALTY_CHARACTER_UNAVAILABLE",
                ),
            )?;
            // 2026-10-17, i8829 line9/MIP Step1 and dispatcher D06:
            // exclude home gross and home-use deductions only. Nonhome
            // business costs and every other actual adjustment stay observed.
            let net = calculation
                .gross_cents
                .checked_sub(home_deduction)
                .and_then(|net| net.checked_sub(casualty))
                .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
            excluded_net = excluded_net
                .checked_add(net)
                .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
        }
        result.modified_agi_cents.insert(
            site.id,
            agi_dollars
                .checked_mul(100)
                .and_then(|agi| agi.checked_sub(excluded_net))
                .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?,
        );
    }

    Ok(result)
}

#[path = "site_calculation.rs"]
mod site_calculation;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HomeIncome {
    pub gross_cents: i64,
    pub operating_cents: i64,
}

impl HomeIncome {
    // Schedule C line 7 excludes cost of goods sold, and line 28 does not
    // deduct that cost again. Schedule F's receipt owner already nets its
    // resale and inventory costs.
    pub(crate) fn from_source(
        draft: &FederalDraft,
        source: i64,
        business: &FederalBusiness,
        operating_cents: i64,
    ) -> Result<Self, crate::Section179PreparedIssue> {
        let goods = if matches!(business.activity, crate::BusinessActivity::Farming { .. }) {
            0
        } else {
            business.inventory.cost()
        };
        Ok(Self {
            gross_cents: crate::federal_rules::farm_receipts(draft, source, business)?
                .checked_sub(goods)
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(source))?,
            operating_cents: operating_cents
                .checked_sub(goods)
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(source))?,
        })
    }
}

pub(crate) fn deduction_with_context(
    draft: &FederalDraft,
    source: i64,
    profits: &alloc::collections::BTreeMap<i64, HomeIncome>,
    home: &HomeContext,
) -> Result<i64, crate::Section179PreparedIssue> {
    if !draft
        .income
        .iter()
        .any(|entry| entry.id == source && matches!(entry.income, FederalIncome::Business { .. }))
    {
        return Err(crate::Section179PreparedIssue::UnknownSource(source));
    }
    if draft.home_sites.is_empty() {
        return Ok(0);
    }
    let results = site_calculation::calculate(draft, profits, home).map_err(|(site, key)| {
        crate::Section179PreparedIssue::SourceBoundaryUnavailable(site, key)
    })?;
    let deduction = crate::kernel::ratio::checked_sum(
        results.iter().filter_map(|result| result.deductions.get(&source)).copied(),
    )
    .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
    let entry = draft
        .income
        .iter()
        .find(|entry| entry.id == source)
        .ok_or(crate::Section179PreparedIssue::UnknownSource(source))?;
    let FederalIncome::Business { data } = &entry.income else {
        return Err(crate::Section179PreparedIssue::WrongSourceKind(source));
    };
    let income = profits
        .get(&source)
        .ok_or(crate::Section179PreparedIssue::SourceIncomeUnavailable(source))?;
    if crate::unknowns::business_loss_needs_risk(
        draft,
        source,
        data,
        data.expense_with_office(
            income
                .operating_cents
                .checked_add(if matches!(data.activity, crate::BusinessActivity::Farming { .. }) {
                    0
                } else {
                    data.inventory.cost()
                })
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(source))?,
            deduction,
        ),
    )? {
        return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
            source,
            "HOME_AT_RISK_LOSS_SOURCE_UNAVAILABLE",
        ));
    }
    Ok(deduction)
}

fn invocation_profits(
    draft: &FederalDraft,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: &HomeContext,
) -> Result<alloc::collections::BTreeMap<i64, HomeIncome>, crate::Section179PreparedIssue> {
    draft
        .income
        .iter()
        .filter_map(|entry| {
            let FederalIncome::Business { data } = &entry.income else {
                return None;
            };
            Some(
                selected
                    .map_or_else(
                        || {
                            crate::section179_return::operating(
                                draft,
                                entry.id,
                                data,
                                None,
                                Some(home),
                            )
                            .and_then(|operating_cents| {
                                HomeIncome::from_source(draft, entry.id, data, operating_cents)
                            })
                        },
                        |value| value.home_income(entry.id),
                    )
                    .map(|profit| (entry.id, profit)),
            )
        })
        .collect()
}

pub(crate) fn report_with_section179(
    draft: &FederalDraft,
    result: &mut FederalResult,
    source: i64,
    _business: &FederalBusiness,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&HomeContext>,
) -> Result<(), crate::Section179PreparedIssue> {
    if draft.home_sites.is_empty() {
        return Ok(());
    }
    let home = home.ok_or(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
        source,
        "HOME_CONTEXT_REQUIRED",
    ))?;
    let calculations =
        site_calculation::calculate(draft, &invocation_profits(draft, selected, home)?, home)
            .map_err(|(site, key)| {
                crate::Section179PreparedIssue::SourceBoundaryUnavailable(site, key)
            })?;
    for calculation in calculations {
        // One Form8829 per physical home and owner, emitted from its leading
        // actual activity; each activity's own deduction remains separately allocated.
        let site = draft
            .home_sites
            .iter()
            .find(|site| site.id == calculation.site)
            .ok_or(crate::Section179PreparedIssue::UnknownSource(calculation.site))?;
        let claim = site
            .claims
            .iter()
            .find(|claim| claim.owner == calculation.owner)
            .ok_or(crate::Section179PreparedIssue::UnknownSource(calculation.site))?;
        if claim.activities.first().map(|activity| activity.activity) != Some(source) {
            continue;
        }
        site_calculation::report_lines(result, &calculation, site, claim);
        if !site.casualties.is_empty() {
            result.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] "Shared casualty reductions were allocated proportionally by each property's remaining net loss, with cent rounding in original property order.",
                msg!("NOTE_HOME_CASUALTY_PROPORTIONAL_REDUCTIONS")));
        }
        if !claim.qualifies() {
            result.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] "Qualifying business use of this home was not established; its requested home-use deduction was left out.",
                msg!("WARN_HOME_USE_NOT_ESTABLISHED")));
        }
    }
    Ok(())
}

pub(crate) fn schedule_a_shares(
    draft: &FederalDraft,
) -> Result<ScheduleAShares, crate::Section179PreparedIssue> {
    use crate::kernel::ratio::exact;
    let mut result = ScheduleAShares::default();
    let mut taxes = exact::whole(0);
    let mut interest = [exact::whole(0), exact::whole(0)];
    let mut points = [exact::whole(0), exact::whole(0)];
    let mut insurance = exact::whole(0);
    let loans = draft.deductions.as_ref().map_or(&[][..], |ded| ded.mortgage_loans());
    let allowances = crate::mortgage_limit::loan_allowances(loans, separate(draft));
    for site in &draft.home_sites {
        let source_taxes = crate::kernel::ratio::checked_sum(
            site.costs
                .iter()
                .filter(|cost| matches!(cost.kind, crate::HomeCostKind::PropertyTax))
                .map(|cost| cost.amount),
        )
        .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
        result.whole_home_taxes_cents = result
            .whole_home_taxes_cents
            .checked_add(source_taxes)
            .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
        for claim in &site.claims {
            if !claim.qualifies() || !matches!(claim.method, crate::OfficeMethod::Actual { .. }) {
                continue;
            }
            let columns = site.cost_columns(claim).ok_or(
                crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                    site.id,
                    "HOME_USE_FRACTION_UNAVAILABLE",
                ),
            )?;
            let fraction = claim.fraction(site.home_area).ok_or(
                crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                    site.id,
                    "HOME_USE_FRACTION_UNAVAILABLE",
                ),
            )?;
            taxes += &columns.direct[0] + &columns.indirect[0] * &fraction;
            let paid = site.paid_fraction(claim.owner).ok_or(
                crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                    site.id,
                    "HOME_PAYMENT_SHARE_UNAVAILABLE",
                ),
            )?;
            for loan in
                allowances.iter().filter(|loan| site.mortgage_loans.contains(&loan.canonical_id))
            {
                interest[usize::from(!loan.reported_on_1098)] +=
                    exact::whole(loan.regular_interest_cents) * &fraction * &paid;
            }
            for loan in
                allowances.iter().filter(|loan| site.mortgage_loans.contains(&loan.canonical_id))
            {
                points[usize::from(!loan.points_reported_on_1098)] +=
                    exact::whole(loan.regular_points_cents) * &fraction * &paid;
            }
            let loan_insurance = crate::kernel::ratio::checked_sum(
                allowances
                    .iter()
                    .filter(|loan| site.mortgage_loans.contains(&loan.canonical_id))
                    .map(|loan| loan.regular_insurance_cents),
            )
            .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
            insurance += exact::whole(loan_insurance) * fraction * paid;
        }
    }
    result.taxes = taxes;
    result.interest_cents = interest;
    result.points_cents = points;
    result.insurance_premiums_cents = insurance;
    Ok(result)
}

pub(crate) fn schedule_a_checks_with_section179(
    draft: &FederalDraft,
    _result: &mut FederalResult,
    facts: ScheduleAFacts,
    _selected: Option<&crate::Section179PreparedAllocation>,
    _home: Option<&HomeContext>,
) -> Result<(), crate::Section179PreparedIssue> {
    // The canonical mortgage source has already applied debt and premium limits;
    // none of these old aggregate figures authorizes a second entered amount.
    let _ = (
        facts.magi,
        facts.limited_interest,
        facts.paid_interest,
        facts.loan_interest_cents,
        facts.property_taxes_cents,
        facts.estate_taxes,
    );
    if facts.estate_taxes != 0 && has_actual(draft) {
        return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
            0,
            "HOME_ESTATE_SALT_SOURCE_UNAVAILABLE",
        ));
    }
    if facts.mortgage_credit != 0
        && draft.home_sites.iter().any(|site| !site.mortgage_loans.is_empty())
    {
        return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
            0,
            "HOME_MORTGAGE_CREDIT_SOURCE_LINK_UNAVAILABLE",
        ));
    }
    Ok(())
}

// The current canonical source slice admits home losses, whose recognized
// business gains/recapture remain explicitly unavailable in the casualty owner.
// With no admitted gains, Form4684 Section B net losses are ordinary losses.
pub(crate) fn casualty_ordinary(
    draft: &FederalDraft,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: &HomeContext,
) -> Result<i64, crate::Section179PreparedIssue> {
    crate::kernel::ratio::checked_sum(casualty_by_owner(draft, selected, home)?)
        .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)
}

pub(crate) fn casualty_by_owner(
    draft: &FederalDraft,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: &HomeContext,
) -> Result<[i64; 2], crate::Section179PreparedIssue> {
    let mut amounts = [0i64; 2];
    if draft.home_sites.is_empty() {
        return Ok(amounts);
    }
    for calculation in
        site_calculation::calculate(draft, &invocation_profits(draft, selected, home)?, home)
            .map_err(|(site, key)| {
                crate::Section179PreparedIssue::SourceBoundaryUnavailable(site, key)
            })?
    {
        let loss = calculation.casualty.allowed_casualty_cents().ok_or(
            crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                calculation.site,
                "HOME_CASUALTY_CHARACTER_UNAVAILABLE",
            ),
        )?;
        let owner = usize::from(calculation.owner == crate::Owner::Spouse);
        amounts[owner] = amounts[owner]
            .checked_sub(loss)
            .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
    }
    Ok(amounts)
}

/// Match the return's once-rounded ordinary amount while preserving its known
/// taxpayer/spouse source attribution through the existing native projection.
pub(crate) fn casualty_by_owner_dollars(
    draft: &FederalDraft,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: &HomeContext,
) -> Result<[i64; 2], crate::Section179PreparedIssue> {
    let cents = casualty_by_owner(draft, selected, home)?;
    let total = crate::kernel::ratio::checked_sum(cents)
        .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)?;
    crate::kernel::ratio::project_rounded_pair(crate::kernel::ratio::dollars(total), cents, 0)
        .ok_or(crate::Section179PreparedIssue::AggregateAmountOverflow)
}
