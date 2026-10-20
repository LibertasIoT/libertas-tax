// Coverage: Form 8995 / 8995-A — Supported (audited 2026-09-25).
//   Lines: 20% QBI per activity (Schedule C, rental, K-1), 2026 thresholds/phase-in, SSTB
//   reduction, W-2 wage/UBIA limit, §199A REIT/PTP dividends, prior business and REIT/PTP loss
//   carryforwards, taxable-income cap and $400 active-business minimum. Excluded: aggregation,
//   cooperative/DPAD items, PTP items, former-spouse/missing carryforward history. An unknown
//   SSTB answer above the threshold is assumed (SSTB for income, not for a loss) with the note
//   WARN_QBI_SPECIFIED_SERVICE_ASSUMED (2026-10-05, Answers and unknowns).
//   Evidence: business_rules.rs (qbi_with_history, qbi_phase, QbiHistory, ReitDividends),
//   partnership_qbi.rs, scorp_qbi.rs; tests: qbi_2026_phase_in_service_limits_and_loss_netting,
//   live_current_sstb_loss_reports_only_income_adjusted_carryforwards (investment.rs),
//   combined_property_qbi_counts_one_business_wage_limit (business.rs); Rev. Proc. 2025-32, i8995
//   Unit C13a (2026-09-25): qbi_aggregation_group on FederalBusiness and FederalRental; activities
//   sharing a label combine QBI, W-2 wages and UBIA before the wage/UBIA limit (Reg. 1.199A-1(d)(2)(iv)(B));
//   a loss member nets inside its group and keeps its wages; a specified-service member is refused
//   above the threshold (Reg. 1.199A-4(b)(1)(iii)); below the threshold labels are ignored; the
//   election is a review note naming the group. Single-business K-1 grouping
//   now consumes qbi_aggregation.rs source/election readiness; farm income
//   averaging with a merged group is reported out of scope. STORAGE_FORMAT 11. Tests: business.rs
//   qbi_aggregation_combines_wages_and_ubia_before_the_limit, aggregation_has_no_effect_below_the_threshold,
//   aggregated_loss_member_nets_inside_its_group_and_keeps_its_wages,
//   aggregation_refuses_a_specified_service_member_above_the_threshold.
//   Unit C13b (2026-09-25): PartnerK1.publicly_traded routes the K-1's qualified income into the
//   REIT/PTP component (Form 8995 line 6 / 8995-A line 28) in federal_rules.rs, never wage/UBIA
//   limited or aggregated; a PTP loss carries through reit_ptp (line 19). A PTP with wages, UBIA,
//   history or suspended QBI balances retains its named source boundary. Current
//   ordinary SSTB eligibility runs once in partnership_qbi before this separate
//   component; suspended/original-vintage and property/PTP source feeds remain
//   their owners' responsibility. Tests: business.rs, qbi_current_ptp
//   ptp_qbi_is_separate_and_carries_its_own_loss, ptp_is_never_aggregated,
//   ptp_income_reaches_the_return_through_the_same_reit_ptp_component.
//   C-fixture (2026-09-25, tests/activity/business.rs): qbi_return_below_the_threshold_matches_hand_computation
//   (Form 8995 lines 1-17 with the taxable-income cap binding, by hand),
//   phase_in_with_the_limit_above_the_unlimited_amount_takes_the_full_deduction (the map's unproved
//   edge), phase_in_reduces_the_excess_by_the_phase_in_percentage (the 2026 $75,000 single range),
//   specified_service_above_the_range_gets_nothing_while_reit_qualifies,
//   taxable_income_cap_limits_the_qbi_deduction.
//   Line audit QBI-D1 to D7 (unit F12, 2026-10-10): tips allowed under §224 are removed from the
//   business's QBI; Form 8995 line 5 is 20% of line 4 (Form 8995-A rounds each column); a Form 1099-PATR
//   box 7 patron files Form 8995-A with lines 13/14 and Schedule D line 6 (zero without farm W-2 wages);
//   distributions are QBI only as box 8 reports them; Schedule C and Form 4835 assets count for the UBIA
//   limit only while their depreciable period has not ended (BusinessAsset::qbi_property); a released
//   earlier-year qualified loss is part of line 3, not a line 1 row (QbiFiling.released). Tests:
//   tests/cross/line_audit_qbi_f.rs qbi_d1_* to qbi_d5_*, farm_rental.rs qbi_d6_*,
//   partnership_no_statement.rs qbi_d7_*.

// Section 199A: 2026 parameters from Rev. Proc. 2025-32 and Form 8995-A.
// Source facts feed one per-activity engine; no user-calculated QBI inputs.
use crate::kernel::phase;
use crate::kernel::ratio::{fraction, positive};
use crate::{FederalResult, FederalStatus};
use alloc::vec::Vec;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct QbiActivity {
    // Runtime source attribution only; never an App input or stored field.
    pub owner: Option<crate::Owner>,
    pub income: i64,
    // Same selected source endpoint before its final report projection. None
    // means this owner supplies an explicitly annual reported adapter.
    pub source_income_cents: Option<i64>,
    pub source_wages_cents: Option<i64>,
    pub source_property_cents: Option<i64>,
    pub wages: i64,
    pub property: i64,
    pub active: bool,
    pub service: Option<bool>,
    pub group: Option<alloc::string::String>,
}

impl QbiActivity {
    pub(crate) fn with_owner(mut self, owner: crate::Owner) -> Self {
        self.owner = Some(owner);
        self
    }
}

pub(crate) mod projection;

// Keep the filed annual SE tax and deduction rounding while removing church
// employee/clergy earnings before attribution to business and partner sources.
// Both Form8958 counterparts use this same source projection.
pub(crate) fn business_se_totals(tax: i64, half: i64, net: i64, nonbusiness: i64) -> [i64; 2] {
    [tax, half].map(|amount| {
        amount - if net > 0 { crate::kernel::ratio::fraction(amount, nonbusiness, net) } else { 0 }
    })
}

// Share the same source attribution between Form 2555 and section 199A.
// Allocate once after the owner's actual/optional SE election is resolved.
#[cfg(test)]
pub(crate) fn half_se_by_business(
    d: &crate::FederalDraft,
    owner: usize,
    half: i64,
    optional: &alloc::collections::BTreeMap<i64, i64>,
) -> alloc::collections::BTreeMap<i64, i64> {
    match half_se_by_business_with_section179(d, owner, half, optional, None, None) {
        Ok(value) => value,
        Err(_) => unreachable!("legacy half-SE calculation has no selected allocation"),
    }
}

pub(crate) fn half_se_by_business_with_section179(
    d: &crate::FederalDraft,
    owner: usize,
    half: i64,
    optional: &alloc::collections::BTreeMap<i64, i64>,
    selected: Option<&crate::Section179PreparedAllocation>,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<alloc::collections::BTreeMap<i64, i64>, crate::Section179PreparedIssue> {
    let mut entries = Vec::new();
    for e in &d.income {
        if let crate::FederalIncome::Business { data } = &e.income
            && usize::from(e.owner == crate::Owner::Spouse) == owner
            && !crate::statutory_wages::business(d, e.id)
        {
            let weight = match optional.get(&e.id) {
                Some(weight) => *weight,
                None => crate::kernel::ratio::dollars(
                    data.earned_receipts()
                        - crate::section179_return::expenses(d, e.id, data, selected, home).map(
                            |expense| {
                                expense
                                    - selected.map_or(0, |allocation| {
                                        allocation.ordinary_risk_adjustment(e.id)
                                    })
                            },
                        )?
                        - crate::clergy::business_se_exclusion_with_section179(
                            d, e.id, data, selected, home,
                        )?,
                ),
            };
            entries.push((e.id, weight));
        }
    }
    Ok(half_se_allocation(half, entries))
}

// Inputs are resolved owner-specific SE attribution weights in one common
// unit, not raw K-1 box 14 amounts or Schedule E taxable income. The amount
// and weights may use dollars or cents; the allocator preserves the amount. Source adapters own
// loss limits and actual/optional-method selection. Preserve partner IDs for
// these worksheets; QBI uses its own gross-income attribution under section 199A.
pub(crate) fn half_se_allocation(
    half: i64,
    entries: impl IntoIterator<Item = (i64, i64)>,
) -> alloc::collections::BTreeMap<i64, i64> {
    let mut entries: Vec<_> = entries.into_iter().collect();
    entries.sort_by_key(|(id, _)| *id);
    let weights: Vec<_> = entries.iter().map(|(_, amount)| positive(*amount)).collect();
    entries
        .into_iter()
        .map(|(id, _)| id)
        .zip(crate::kernel::ratio::allocate_remaining(half, &weights))
        .collect()
}

#[cfg(test)]
pub(crate) fn qbi(
    activities: &[QbiActivity],
    taxable: i64,
    gains: i64,
    status: FederalStatus,
    r: &mut FederalResult,
) -> (i64, i64) {
    let value = qbi_with_history(activities, taxable, gains, status, QbiOther::default(), r);
    (value.deduction, value.business_loss)
}

#[derive(Default)]
pub(crate) struct QbiOther {
    pub business_loss: i64,
    pub reit: i64,
    pub reit_loss: i64,
}
// How the return files its QBI form.
#[derive(Default)]
pub(crate) struct QbiFiling {
    // Positions in the activity list of released earlier-year qualified
    // losses. They net like any loss, but Form 8995 prints them on line 3
    // with the carryforward, not as a line 1 row (2026 i8995 lines 1 and 3).
    pub released: Vec<usize>,
    // A patron of a specified agricultural or horticultural cooperative
    // (Form 1099-PATR box 7) files Form 8995-A at any income, whose line 3
    // rounds each column.
    pub patron: bool,
    // Activity-aligned Schedule D reductions, already supported by actual facts.
    pub patron_allocations: Vec<[i64; 2]>,
    pub dpad: i64,
}
// The Form 8995 / 8995-A lines of one computation, in whole dollars.
// Activity rows are (QBI before loss netting, after netting, 20% of it, the
// component after the wage/UBIA limit), one per resulting trade or aggregation.
// Source members and intermediate limits remain transient PRINT evidence.
#[derive(Default)]
pub(crate) struct QbiColumn {
    pub wages: [i64; 3],
    pub property: [i64; 2],
    pub wage_limit: i64,
    pub full_limit: i64,
    pub phase_reduction: i64,
    pub phased: i64,
    pub loss: i64,
}
#[derive(Default)]
pub(crate) struct QbiLines {
    pub activities: Vec<[i64; 4]>,
    pub columns: Vec<QbiColumn>,
    pub source_values: Vec<[i64; 3]>,
    pub members: Vec<Vec<usize>>,
    pub phase: [i64; 3],
    pub losses: [i64; 4],
    // One flag per activity row: a released earlier-year loss, which is not
    // a row of its own.
    pub released: Vec<bool>,
    pub grouped: bool,
    pub patron: bool,
    pub patron_reductions: Vec<i64>,
    pub patron_allocations: Vec<[i64; 2]>,
    pub pre_patron: Vec<i64>,
    pub dpad: i64,
    pub total_income: i64,
    pub prior_loss: i64,
    pub net_income: i64,
    pub component: i64,
    pub reit: i64,
    pub prior_reit_loss: i64,
    pub reit_net: i64,
    pub reit_component: i64,
    pub before_limit: i64,
    pub taxable: i64,
    pub gains: i64,
    pub limit_base: i64,
    pub income_limit: i64,
    pub limited: i64,
    pub minimum: i64,
}
pub(crate) struct QbiResult {
    pub lines: QbiLines,
    pub deduction: i64,
    // Eligible source QBI before return-wide aggregation/loss netting; its
    // actual owners remain independent of the joint deduction calculation.
    pub owner_income: Option<[i64; 2]>,
    pub owner_active: Option<[i64; 2]>,
    // Exact activity components only when the final income cap/minimum does
    // not alter their sum (plus the independent REIT component).
    pub activity_deductions: Option<Vec<i64>>,
    // Conserved source shares of the business component, aligned with the
    // input activities even when aggregation or the income cap applies.
    // REIT/PTP and section 199A(g) remain separate owner outputs.
    pub activity_shares: Vec<i64>,
    pub business_loss: i64,
    pub reit_loss: i64,
}
fn qbi_threshold(status: FederalStatus) -> i64 {
    match status {
        FederalStatus::Joint => crate::year_constants::QBI_THRESHOLD_JOINT_DOLLARS,
        FederalStatus::Separate => crate::year_constants::QBI_THRESHOLD_SEPARATE_DOLLARS,
        _ => crate::year_constants::QBI_THRESHOLD_OTHER_DOLLARS,
    }
}
pub(crate) fn qbi_phase(taxable: i64, status: FederalStatus) -> (i64, i64) {
    let threshold = qbi_threshold(status);
    let range = if status == FederalStatus::Joint {
        crate::year_constants::QBI_PHASE_RANGE_JOINT_DOLLARS
    } else {
        crate::year_constants::QBI_PHASE_RANGE_OTHER_DOLLARS
    };
    phase::phase_position(taxable, threshold, range)
}

// The sole SSTB applicability policy. Only the sign of `income` is read,
// so callers retain their own explicit Dollar or source-cent unit. The
// existing activity facade rounds its legacy outputs; source-cent callers
// keep the factor exact until their prescribed reported category boundary.
pub(crate) fn qbi_eligibility_factor(
    income: i64,
    service: Option<bool>,
    grouped: bool,
    taxable: i64,
    status: FederalStatus,
) -> Result<(i64, i64), (&'static str, &'static str)> {
    let (phased, range) = qbi_phase(taxable, status);
    // None is an internal missing statutory-business source, never an answer
    // offered to a person. Preserve its conservative sign-specific treatment;
    // assume_service supplies the existing WARN_QBI_SPECIFIED_SERVICE_ASSUMED.
    let service = service.unwrap_or(income >= 0);
    if phased > 0 && service && grouped {
        return Err((
            "ISSUE_QBI_SERVICE_BUSINESS_AGGREGATION",
            crate::fixture_english!(
                "A specified service trade or business cannot join an aggregation group (Form 8995-A Schedule B); remove its group above the income threshold."
            ),
        ));
    }
    Ok((if service { range - phased } else { range }, range))
}

// Apply the current SSTB percentage once. Mark the internal result as already
// eligible so downstream loss allocation and the deduction engine share it.
// Historical eligible losses already carry service=No and remain unchanged.
pub(crate) fn eligible_qbi_activity(
    activity: &QbiActivity,
    taxable: i64,
    status: FederalStatus,
) -> Result<QbiActivity, (&'static str, &'static str)> {
    let (applicable, range) = qbi_eligibility_factor(
        activity.source_income_cents.unwrap_or(activity.income),
        activity.service,
        activity.group.is_some(),
        taxable,
        status,
    )?;
    Ok(QbiActivity {
        source_wages_cents: activity
            .source_wages_cents
            .map(|cents| phase::remaining_fraction(cents, applicable, range)),
        source_property_cents: activity
            .source_property_cents
            .map(|cents| phase::remaining_fraction(cents, applicable, range)),
        source_income_cents: activity
            .source_income_cents
            .map(|cents| phase::remaining_fraction(cents, applicable, range)),
        owner: activity.owner,
        income: activity.source_income_cents.map_or_else(
            || phase::remaining_fraction(activity.income, applicable, range),
            |cents| {
                crate::kernel::ratio::dollars(phase::remaining_fraction(cents, applicable, range))
            },
        ),
        wages: activity.source_wages_cents.map_or_else(
            || phase::remaining_fraction(activity.wages, applicable, range),
            |cents| {
                crate::kernel::ratio::dollars(phase::remaining_fraction(cents, applicable, range))
            },
        ),
        property: activity.source_property_cents.map_or_else(
            || phase::remaining_fraction(activity.property, applicable, range),
            |cents| {
                crate::kernel::ratio::dollars(phase::remaining_fraction(cents, applicable, range))
            },
        ),
        active: activity.active,
        service: Some(false),
        group: activity.group.clone(),
    })
}

// An absent statutory-business source retains its existing conservative note
// when the income threshold makes the missing classification matter.
pub(crate) fn assume_service(
    activity: &QbiActivity,
    taxable: i64,
    status: FederalStatus,
    r: &mut FederalResult,
    context: crate::IssueContext,
) {
    if activity.service.is_none() && qbi_phase(taxable, status).0 > 0 {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("WARN_QBI_SPECIFIED_SERVICE_ASSUMED", alloc::vec![]),
            context,
        );
    }
}

#[cfg(test)]
pub(crate) fn qbi_with_history(
    activities: &[QbiActivity],
    taxable: i64,
    gains: i64,
    status: FederalStatus,
    other: QbiOther,
    r: &mut FederalResult,
) -> QbiResult {
    qbi_filed(activities, taxable, gains, status, other, &QbiFiling::default(), r)
}

pub(crate) fn qbi_filed(
    activities: &[QbiActivity],
    taxable: i64,
    gains: i64,
    status: FederalStatus,
    other: QbiOther,
    filing: &QbiFiling,
    r: &mut FederalResult,
) -> QbiResult {
    let (phased, range) = qbi_phase(taxable, status);
    // Form 8995-A rounds line 3 in each column; Form 8995 line 5 is 20% of line 4.
    let per_column = phased > 0 || filing.patron;
    let mut values = Vec::new();
    for activity in activities {
        match eligible_qbi_activity(activity, taxable, status) {
            Ok(value) => values.push(value),
            Err((key, _message)) => {
                #[cfg(test)]
                let message = _message;

                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Business,
                    "Business",
                    key,
                    message,
                );
                values.push(QbiActivity { group: None, ..activity.clone() });
            }
        }
    }
    let owner_income = values.iter().try_fold([0; 2], |mut owners, activity| {
        if activity.income != 0 {
            owners[usize::from(activity.owner? == crate::Owner::Spouse)] += activity.income;
        }
        Some(owners)
    });
    let owner_active =
        values.iter().filter(|v| v.active).try_fold([0; 2], |mut owners, activity| {
            if activity.income != 0 {
                owners[usize::from(activity.owner? == crate::Owner::Spouse)] += activity.income;
            }
            Some(owners)
        });
    let active: i64 = values.iter().filter(|v| v.active).map(|v| v.income).sum();
    let ungrouped = values.len();
    // Released earlier-year qualified losses join the carryforward on line 3.
    let released_rows: Vec<bool> = (0..ungrouped).map(|i| filing.released.contains(&i)).collect();
    let released_loss: i64 = values
        .iter()
        .zip(&released_rows)
        .filter(|(_, released)| **released)
        .map(|(v, _)| positive(-v.income))
        .sum();
    let mut patron_allocations: Vec<[i64; 2]> = (0..values.len())
        .map(|i| filing.patron_allocations.get(i).copied().unwrap_or([0; 2]))
        .collect();
    let source_values = values.iter().map(|v| [v.income, v.wages, v.property]).collect();
    let source_weights: Vec<i64> = values.iter().map(|v| positive(v.income)).collect();
    let active_weights: Vec<i64> =
        values.iter().map(|v| if v.active { positive(v.income) } else { 0 }).collect();
    let mut members = (0..values.len()).map(|i| alloc::vec![i]).collect();
    let values = if phased > 0 {
        aggregate(values, &mut patron_allocations, &mut members, r)
    } else {
        values
    };
    // Schedule D has one row for each trade, business or aggregation.
    // Apply its lesser-of rule after combining the actual source amounts.
    let reductions: Vec<i64> = patron_allocations
        .iter()
        .map(|[income, wages]| {
            phase::cap(fraction(positive(*income), 9, 100), [fraction(positive(*wages), 1, 2)])
        })
        .collect();
    let profit: i64 = values.iter().map(|v| positive(v.income)).sum();
    let losses: i64 = values.iter().map(|v| positive(-v.income)).sum::<i64>() + other.business_loss;
    let weights: Vec<_> = values.iter().map(|v| positive(v.income)).collect();
    let netted_losses = phase::cap(losses, [profit]);
    let allocated = crate::kernel::ratio::allocate_remaining(netted_losses, &weights);
    let mut deduction = 0;
    let mut components = Vec::new();
    let mut rows = Vec::new();
    let mut pre_patron = Vec::new();
    let mut twenties = Vec::new();
    let mut columns = Vec::new();
    for (index, (v, loss)) in values.iter().zip(allocated).enumerate() {
        let income = positive(v.income) - loss;
        let twenty = fraction(income, 20, 100);
        let half_wages = fraction(v.wages, 1, 2);
        let quarter_wages = fraction(v.wages, 1, 4);
        let property_share = fraction(v.property, 25, 1000);
        let limit = phase::floor(half_wages, quarter_wages + property_share);
        let phase_reduction = phase::remaining_fraction(positive(twenty - limit), phased, range);
        let limited = twenty - phase_reduction;
        columns.push(QbiColumn {
            wages: [v.wages, half_wages, quarter_wages],
            property: [v.property, property_share],
            wage_limit: limit,
            full_limit: phase::cap(twenty, [limit]),
            phase_reduction,
            phased: limited,
            loss,
        });
        pre_patron.push(limited);
        let component = positive(limited - reductions[index]);
        deduction += component;
        components.push(component);
        twenties.push(twenty);
        rows.push([v.income, income, twenty, component]);
    }
    let total_income = values.iter().map(|v| v.income).sum::<i64>();
    let net_income = positive(total_income - other.business_loss);
    // Form 8995 line 5 multiplies line 4 once; the per-column amounts of
    // Form 8995-A line 3 are rounded one by one.
    if !per_column {
        deduction = fraction(net_income, 20, 100);
        components = crate::kernel::ratio::allocate_remaining(deduction, &twenties);
    }
    let reit = fraction(positive(other.reit - other.reit_loss), 20, 100);
    let limit_base = positive(taxable - gains);
    let income_limit = fraction(limit_base, 20, 100);
    let minimum = if active >= 1000 { 400 } else { 0 };
    let limited = phase::cap(deduction + reit, [income_limit]);
    let dpad = phase::cap(filing.dpad, [positive(taxable - limited)]);
    let total = phase::floor(limited + dpad, minimum);
    let mut component_weights = components.clone();
    component_weights.push(reit);
    let limited_components = crate::kernel::ratio::allocate_remaining(limited, &component_weights);
    let mut activity_shares = alloc::vec![0;ungrouped];
    for (indices, amount) in members.iter().zip(limited_components) {
        let weights: Vec<i64> = indices.iter().map(|i| source_weights[*i]).collect();
        for (index, part) in
            indices.iter().zip(crate::kernel::ratio::allocate_remaining(amount, &weights))
        {
            activity_shares[*index] += part;
        }
    }
    if total > limited + dpad {
        for (share, extra) in activity_shares
            .iter_mut()
            .zip(crate::kernel::ratio::allocate_remaining(total - limited - dpad, &active_weights))
        {
            *share += extra;
        }
    }
    let lines = QbiLines {
        activities: rows,
        columns,
        source_values,
        members,
        phase: [qbi_threshold(status), phased, range],
        losses: [other.business_loss, losses, profit, netted_losses],
        patron_reductions: reductions,
        patron_allocations,
        pre_patron,
        dpad,
        released: released_rows,
        grouped: values.len() != ungrouped,
        // Line 1 holds the other activities; line 3 holds the released losses.
        total_income: total_income + released_loss,
        prior_loss: -(other.business_loss + released_loss),
        net_income,
        component: deduction,
        reit: other.reit,
        prior_reit_loss: -other.reit_loss,
        reit_net: positive(other.reit - other.reit_loss),
        reit_component: reit,
        before_limit: deduction + reit,
        taxable,
        gains,
        limit_base,
        income_limit,
        limited: phase::cap(deduction + reit, [income_limit]),
        minimum,
        patron: filing.patron,
    };
    QbiResult {
        lines,
        deduction: total,
        owner_income,
        owner_active,
        activity_shares,
        activity_deductions: (dpad == 0 && total == deduction + reit && values.len() == ungrouped)
            .then_some(components),
        business_loss: crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            losses, profit,
        )
        .get(),
        reit_loss: crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            other.reit_loss,
            other.reit,
        )
        .get(),
    }
}

// Schedule B (Form 8995-A): members sharing a group label above the phase-in
// floor become one trade or business. Reg. 1.199A-1(d)(2)(iv)(B) combines
// their QBI, W-2 wages and UBIA before the wage/UBIA limit, so a member's loss
// nets inside its group first and its wages still count; the combined group
// then enters Schedule C cross-business loss netting as one trade. SSTB members
// were refused by eligible_qbi_activity (Reg. 1.199A-4(b)(1)(iii)). A label
// used by a single activity aggregates nothing. The $400 minimum keeps the
// members' own active-participation income, computed before this merge.
fn aggregate(
    values: Vec<QbiActivity>,
    patron_allocations: &mut Vec<[i64; 2]>,
    members: &mut Vec<Vec<usize>>,
    r: &mut FederalResult,
) -> Vec<QbiActivity> {
    let mut merged: Vec<QbiActivity> = Vec::new();
    let mut groups: Vec<(usize, usize)> = Vec::new();
    let original_allocations = core::mem::take(patron_allocations);
    members.clear();
    let mut add_source = |left: Option<i64>, right: Option<i64>| match (left, right) {
        (Some(left), Some(right)) => match left.checked_add(right) {
            Some(total) => Some(total),
            None => {
                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Business,
                    "QBI aggregation group",
                    "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
                    "Review the document amounts; they must be within the supported input range."
                );
                crate::federal_rules::withhold_required_source_result(r);
                None
            }
        },
        _ => None,
    };
    for (index, v) in values.into_iter().enumerate() {
        let allocation = original_allocations[index];
        let found = v.group.as_ref().and_then(|label| {
            groups.iter_mut().find(|(i, _)| merged[*i].group.as_ref() == Some(label))
        });
        match found {
            Some((i, count)) => {
                let g = &mut merged[*i];
                g.income += v.income;
                g.source_income_cents = add_source(g.source_income_cents, v.source_income_cents);
                g.source_wages_cents = add_source(g.source_wages_cents, v.source_wages_cents);
                g.source_property_cents =
                    add_source(g.source_property_cents, v.source_property_cents);
                g.wages += v.wages;
                g.property += v.property;
                g.active |= v.active;
                *count += 1;
                patron_allocations[*i][0] += allocation[0];
                patron_allocations[*i][1] += allocation[1];
                members[*i].push(index);
            }
            None => {
                if v.group.is_some() {
                    groups.push((merged.len(), 1));
                }
                merged.push(v);
                patron_allocations.push(allocation);
                members.push(alloc::vec![index]);
            }
        }
    }
    for (i, count) in groups {
        if count > 1 {
            let g = &merged[i];
            r.review_notes.push(crate::federal_model::ReviewNote::localized(
                #[cfg(test)] alloc::format!(
                    "Aggregation elected under Reg. 1.199A-4 for group {}; attach Schedule B (Form 8995-A). Combined QBI ${}, W-2 wages ${}, UBIA ${} share one wage/property limit.",
                    g.group.as_deref().unwrap_or_default(),
                    g.income,
                    g.wages,
                    g.property
                ),
                crate::Message::new(
                    "NOTE_QBI_AGGREGATION_ELECTION",
                    alloc::vec![
                        crate::MessageArg::Text(g.group.as_deref().unwrap_or_default().into()),
                        crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: g.income },
                        crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: g.wages },
                        crate::MessageArg::Currency { currency_code: "USD".into(), scale: 0, minor_units: g.property },
                    ],
                ),
            ));
        }
    }
    merged
}

/// Qualified business income losses carried into 2026
/// Prior-return carryforwards; both zero when there are none. The amounts belong to the people filing this return and include all applicable QBI carryforward history. Combine separate prior returns only when their owners file this return; a former-spouse allocation, missing history or newly released suspended losses needs its own worksheet.
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    libertas_macros::LibertasAvroEncode,
    libertas_macros::LibertasAvroDecode,
    libertas_macros::LibertasExport,
)]
pub struct QbiHistory {
    /// Business loss carried from 2025 Form 8995 line 16 or Form 8995-A Schedule C line 6
    /// Enter its magnitude as a positive amount. Include only these filers' carryforwards, even if the business closed. Do not enter an NOL, capital loss or still-suspended passive/at-risk loss here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub business: i64,
    /// REIT/PTP loss carried from 2025 Form 8995 line 17 or Form 8995-A line 29
    /// Enter its magnitude as a positive amount. This separate loss category offsets qualified REIT/PTP income only, not ordinary business QBI.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub reit_ptp: i64,
    /// Ownership of retained QBI losses
    /// Only for a Form 8379 claim. Use actual source records; this never changes the joint calculation.
    /// [NullText]
    /// The retained loss ownership has not been established
    /// [SetText]
    /// The retained losses belong to the spouses as follows
    #[libertas_default($null)]
    pub injured_owners: Option<crate::InjuredQbiLossOwners>,
}
impl QbiHistory {
    /// No prior QBI loss.
    pub(crate) const NONE: Self = Self { business: 0, reit_ptp: 0, injured_owners: None };
    pub(crate) fn amounts(&self) -> [i64; 2] {
        [self.business, self.reit_ptp]
    }
    /// Whether any prior QBI loss is entered.
    pub(crate) fn any(&self) -> bool {
        self.amounts() != [0, 0]
    }
    pub(crate) fn valid(&self) -> bool {
        self.amounts().iter().all(|v| (0..=100000000000).contains(v))
            && self.injured_owners.as_ref().is_none_or(|v| v.valid(self.amounts()))
    }
}

/// Section 199A dividends
#[derive(
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    libertas_macros::LibertasAvroEncode,
    libertas_macros::LibertasAvroDecode,
    libertas_macros::LibertasExport,
)]
pub enum ReitDividends {
    /// None
    #[default]
    None,
    /// Form 1099-DIV box 5 or a fiduciary's Form 1041 K-1 box 14 code I statement
    Reported {
        /// Section 199A dividends already included in box 1a ordinary dividends
        /// Include qualified REIT dividends passed through a regulated investment company. These are not box 1b qualified dividends or capital gain distributions.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
        /// All these dividends satisfy the holding-period and payment-obligation rules
        /// The reporting source must satisfy the applicable share-holding and related-payment rules. For directly held shares, generally hold them for more than 45 days in the applicable 91-day period, excluding days with diminished risk. For an estate or trust allocation, use the fiduciary's qualified dividend statement. Split differently treated lots into separate income entries without duplicating amounts. Check the share-holding records or fiduciary’s qualified dividend statement. No takes no section 199A deduction on these dividends.
        eligible: bool,
    },
}
impl ReitDividends {
    pub(crate) fn amount(&self) -> i64 {
        match self {
            Self::None => 0,
            Self::Reported { amount, .. } => *amount,
        }
    }
    pub(crate) fn qualified(&self) -> i64 {
        match self {
            Self::Reported { amount, eligible: true } => *amount,
            _ => 0,
        }
    }
}

// Section 172(d), shared by regular and alternative-tax current losses.
// Supply the income and allowed deductions of the chosen tax system; never
// obtain ATNOL merely by adding depreciation to an already-limited regular NOL.
pub(crate) struct NolCapital {
    pub return_capital: i64,
    pub nonbusiness_capital: i64,
    pub business_capital: i64,
    pub section1202_excluded: i64,
    pub schedule_capital_net: i64,
}

pub(crate) fn current_nol(
    taxable_before_qbi: i64,
    capital: NolCapital,
    nonbusiness_deductions: i64,
    nonbusiness_ordinary_income: i64,
) -> i64 {
    let NolCapital {
        return_capital,
        nonbusiness_capital,
        business_capital,
        section1202_excluded,
        schedule_capital_net,
    } = capital;
    if section1202_excluded > 0 {
        // Form 172 lines 2–24. A section 1202 exclusion appears on line 17
        // even when other sales turn Schedule D into a net capital loss.
        // The ordinary shortcut below cannot represent lines 16–22 in that
        // case: the exclusion must remain visible independently of line 3.
        let nonbusiness_loss = positive(-nonbusiness_capital);
        let nonbusiness_gain = positive(nonbusiness_capital);
        let excess_deductions =
            positive(nonbusiness_deductions - (nonbusiness_gain + nonbusiness_ordinary_income));
        let available_nonbusiness_gain = phase::cap(
            phase::floor(
                nonbusiness_gain + nonbusiness_ordinary_income - nonbusiness_deductions,
                0,
            ),
            [nonbusiness_gain],
        );
        let business_loss = positive(-business_capital);
        let business_gain = positive(business_capital);
        let capital_adjustment =
            nonbusiness_loss + positive(business_loss - available_nonbusiness_gain - business_gain);
        let schedule_loss = positive(-schedule_capital_net);
        let remaining_schedule_loss = positive(schedule_loss - section1202_excluded);
        let deductible_schedule_loss = positive(-return_capital);
        let disallowed_schedule_loss = positive(remaining_schedule_loss - deductible_schedule_loss);
        let retained_schedule_loss = positive(deductible_schedule_loss - remaining_schedule_loss);
        let capital_modification = positive(capital_adjustment - disallowed_schedule_loss);
        return positive(
            -(taxable_before_qbi
                + excess_deductions
                + section1202_excluded
                + retained_schedule_loss
                + capital_modification),
        );
    }
    // Form 172 lines 2–22: nonbusiness capital losses cannot absorb business
    // gains. Personal deductions use ordinary nonbusiness income first, then
    // nonbusiness capital gains; only the remaining gains offset business losses.
    let excess_deductions = positive(nonbusiness_deductions - nonbusiness_ordinary_income);
    let nonbusiness_gain = positive(nonbusiness_capital);
    let disallowed_deductions = positive(excess_deductions - nonbusiness_gain);
    let available_gain = positive(nonbusiness_gain - excess_deductions);
    let disallowed_business_loss = positive(-business_capital - available_gain);
    let modified_capital = nonbusiness_gain + business_capital + disallowed_business_loss;
    positive(-(taxable_before_qbi - return_capital + modified_capital + disallowed_deductions))
}

// Allocate amounts; legal character, dates and retained facts are unchanged.
crate::community::divide_enum!(ReitDividends {
    records: [Reported { amounts: [amount], facts: [eligible] },],
    units: [None],
    tuples: [],
});
