// Coverage: Form 1099-PATR retained direct-patron sources (2026).
// Raw payer boxes remain import facts. This module owns retained per-activity
// components, source guards and adapters to existing QBI/3800/income owners.
// Canonical FarmLand purchase refunds within both regular/AMT opening bases and
// current direct in-full sale proceeds use the accepted Farm owner. Taxable basis
// excess, positive property QBI qualifiers, historical/instalment dispositions,
// depreciable assets (Brown) and compound K-1/passive retain named dependencies.
// Farm section175 consumes source production character, not all taxable receipts.
// Source: IRS Form 1099-PATR Rev. April 2025; T.D. 9947 final Reg. 1.199A-7;
// 2026 draft Form 8995-A and Form 3800; Pub 225(2025).
pub(crate) use crate::cooperative_source::automatic;
use crate::cooperative_source::{
    CoopAllocation, CoopCreditIncome, CoopDistributionBox, CoopIncomeUse, CoopInvestmentKind,
    CoopNoticeHolding, CoopNoticeKind, CoopNoticeStatus, CoopOtherItem, CoopOtherKind,
    CoopPatronAllocation, CoopPropertyRefund, CoopPropertyTreatment,
};
pub(crate) use crate::federal_model::input_issue;
use crate::kernel::phase;
use crate::validation::ValidationIssue;
use alloc::vec::Vec;

pub(crate) fn other_ids(income: &crate::FederalIncome) -> Vec<i64> {
    match income {
        crate::FederalIncome::Cooperative { data } => {
            data.more.credit_facts.other.iter().map(|row| row.id).collect()
        }
        _ => Vec::new(),
    }
}

pub(crate) fn other_ids_mut(income: &mut crate::FederalIncome) -> Vec<&mut i64> {
    match income {
        crate::FederalIncome::Cooperative { data } => {
            data.more.credit_facts.other.iter_mut().map(|row| &mut row.id).collect()
        }
        _ => Vec::new(),
    }
}

pub(crate) fn eligible_activities(d: &crate::FederalDraft, owner: crate::Owner) -> Vec<i64> {
    d.income
        .iter()
        .filter(|e| e.owner == crate::DocumentOwner::from(owner))
        .filter_map(|e| match &e.income {
            crate::FederalIncome::Business { data }
                if data.source == crate::BusinessSource::UnitedStates
                    && matches!(
                        data.activity,
                        crate::BusinessActivity::Ordinary { .. }
                            | crate::BusinessActivity::Farming { .. }
                    ) =>
            {
                Some(e.id)
            }
            crate::FederalIncome::FarmRental { .. } => Some(e.id),
            _ => None,
        })
        .collect()
}

// Retained parent and child identities are never rewritten by this projection.
#[derive(Clone)]
pub(crate) struct CoopSource {
    pub parent: i64,
    pub owner: crate::Owner,
    pub allocation: CoopAllocation,
    pub activity: Option<i64>,
    // Transient position in the retained source allocation vector. None only for
    // sources() synthesizing a projection from an empty vector; Some is never
    // proof that the original submitted Page contained that row.
    pub allocation_index: Option<usize>,
}
#[cfg(test)]
pub(crate) fn sources(d: &crate::FederalDraft) -> Vec<CoopSource> {
    let source_claims = crate::related_forms::effective_claimants(d);
    sources_with_claims(d, &source_claims)
}

pub(crate) fn sources_with_claims(
    d: &crate::FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Vec<CoopSource> {
    let claims = source_claims;
    d.income
        .iter()
        .flat_map(|e| {
            let crate::FederalIncome::Cooperative { data } = &e.income else {
                return Vec::new();
            };
            let data = &crate::FederalCooperative::from(data);
            let allocations = if data.allocations.is_empty() {
                alloc::vec![automatic(data)]
            } else {
                data.allocations.clone()
            };
            let explicit = !data.allocations.is_empty();
            allocations
                .into_iter()
                .enumerate()
                .map(|(index, allocation)| CoopSource {
                    parent: e.id,
                    owner: e.owner.owned(),
                    activity: claims
                        .get(&if allocation.id == 0 { e.id } else { allocation.id })
                        .copied(),
                    allocation,
                    allocation_index: explicit.then_some(index),
                })
                .collect()
        })
        .collect()
}

// Canonical land and equipment use their accepted property owners. No new
// identity or basis pool is stored; historical-sale treatment stays explicit.

pub(crate) fn land_refund_supported_with_claims(
    d: &crate::FederalDraft,
    s: &CoopSource,
    p: &CoopPropertyRefund,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> bool {
    if p.amount == 0 {
        return true;
    }
    let Some(property) = p.property else {
        return false;
    };
    if let Some(asset) = crate::cooperative_basis::asset(d, s.owner, s.activity, property) {
        if p.distribution != CoopDistributionBox::Patronage
            || p.treatment != CoopPropertyTreatment::Purchase
        {
            return false;
        }
        let Some(activity) = s.activity else {
            return false;
        };
        return [false, true].into_iter().all(|alternative| {
            match crate::cooperative_basis::source_amounts_with_claims(
                d,
                activity,
                asset,
                crate::depreciation::mid_quarter(d),
                alternative,
                source_claims,
                None,
            ) {
                Ok(_) | Err(crate::Section179PreparedIssue::AmountOverflow(_)) => true,
                Err(_) => false,
            }
        });
    }
    let Some(farm) = crate::farm::sources(d).find(|f| {
        f.entry.owner == crate::DocumentOwner::from(s.owner)
            && Some(f.entry.id) == s.activity
            && f.farm.land_basis.iter().any(|land| land.id == property)
    }) else {
        return false;
    };
    let land = farm.farm.land_basis.iter().find(|land| land.id == property).unwrap();
    if p.distribution != CoopDistributionBox::Patronage {
        return false;
    }
    match p.treatment {
        CoopPropertyTreatment::Purchase => {
            let total = land_adjustments_with_claims(d, farm.entry.id, property, source_claims).0;
            // Different regular/AMT taxable excess needs an income-owner seam;
            // do not suppress that required tax by merely clamping basis.
            total <= phase::cap(land.regular, [land.amt])
        }
        CoopPropertyTreatment::CurrentSale => {
            p.qualified_items + p.sstb_items + p.qualified_payments == 0
                && farm.farm.land_sales.iter().any(|sale| {
                    sale.basis_property == property
                        && sale.situation == crate::FarmLandSituation::Direct
                        && matches!(sale.payment, crate::SalePayment::InFull)
                })
        }
        CoopPropertyTreatment::EarlierDisposition { .. } => false,
    }
}

#[cfg(test)]
pub(crate) fn land_adjustments(
    d: &crate::FederalDraft,
    activity: i64,
    property: i64,
) -> (i64, i64) {
    let source_claims = crate::related_forms::effective_claimants(d);
    land_adjustments_with_claims(d, activity, property, &source_claims)
}

pub(crate) fn land_adjustments_with_claims(
    d: &crate::FederalDraft,
    activity: i64,
    property: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> (i64, i64) {
    sources_with_claims(d, source_claims)
        .iter()
        .filter(|s| s.activity == Some(activity))
        .flat_map(|s| &s.allocation.properties)
        .filter(|p| {
            p.property == Some(property) && p.distribution == CoopDistributionBox::Patronage
        })
        .fold((0, 0), |(basis, proceeds), p| match p.treatment {
            CoopPropertyTreatment::Purchase => (basis + p.amount, proceeds),
            CoopPropertyTreatment::CurrentSale => (basis, proceeds + p.amount),
            CoopPropertyTreatment::EarlierDisposition { .. } => (basis, proceeds),
        })
}

// These are the receipt source owner's existing report requirements. The
// prepared before179 view uses the same facts without collecting tax again.

pub(crate) fn recipient_required(source: &CoopSource) -> bool {
    source.allocation.ordinary() != 0
}

fn receipt_requirements_with_claims(
    d: &crate::FederalDraft,
    source: &CoopSource,
    eligible: &[i64],
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> [(bool, &'static str, &'static str, &'static str); 4] {
    let a = &source.allocation;
    // Box 5 is gross redemption, not the holder's basis or notice character.
    let redeemed: i64 = a
        .redemptions
        .iter()
        .filter(|v| v.distribution == CoopDistributionBox::Redeemed)
        .map(|v| v.amount)
        .sum();
    let personal_redemptions: i64 = a
        .redemptions
        .iter()
        .filter(|v| {
            v.kind == CoopNoticeKind::Patronage && v.distribution == CoopDistributionBox::Redeemed
        })
        .map(|v| v.amount)
        .sum();
    [
        (!recipient_required(source) || source.activity.is_some_and(|id| eligible.contains(&id)),
         "ISSUE_COOP_ACTIVITY_NEEDED", "Cooperative recipient",
         "The known cooperative income needs its actual same-owner Schedule C, farm or farm-rental recipient; no activity or income is guessed."),
        (a.properties.iter().all(|p| land_refund_supported_with_claims(d, source, p, source_claims)),
         "ISSUE_COOP_PROPERTY_HISTORY_NEEDED", "Cooperative property refund",
         "The capital-purchase refund needs the canonical property owner's effective-dated regular/AMT basis or disposition result. The refund is not simply taxed as ordinary income."),
        (redeemed == a.distributions.redeemed && a.redemptions.iter().all(|v|
            v.kind != CoopNoticeKind::Unresolved && !matches!(v.holding,CoopNoticeHolding::Unresolved)
            && v.amount <= v.stated_value
            && !matches!(v.holding,CoopNoticeHolding::Acquired{basis,..} if v.amount < basis || a.personal.total() > 0 || !a.properties.is_empty())),
         "ISSUE_COOP_NOTICE_HISTORY_NEEDED", "Cooperative notice history",
         "Record notice history covering the entire box 5 redeemed amount, including actual ownership, basis and stated value. Above-stated-value proceeds, losses or basis shared with an excluded part need their source allocation; no basis or gain character is guessed."),
        (a.personal.nonpatronage == 0 && a.personal.per_unit_retains == 0 && a.personal.redeemed <= personal_redemptions,
         "ISSUE_COOP_DISTRIBUTION_CHARACTER_NEEDED", "Cooperative distribution character",
         "Purchase exclusions require the original patronage-dividend or patronage-notice source. Nonpatronage distributions and per-unit retains are not assumed to be personal-purchase refunds."),
    ]
}

fn credit_income_known(item: &CoopOtherItem, eligible: &[i64]) -> bool {
    !matches!(item.income, CoopCreditIncome::Unresolved)
        && (!matches!(item.income,CoopCreditIncome::Inclusion{amount,..} if amount > 0)
            || item.activity.is_some_and(|id| eligible.contains(&id)))
}

// The supplied working view already owns support/routing and accepted global
// graph validation. Certify only present contributors which can reach C/F;
// a known unrelated farm rental does not invalidate their receipt sources.

pub(crate) fn before179_receipt_source_issue_with_claims(
    d: &crate::FederalDraft,
    activities: &[(i64, crate::Owner)],
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<(i64, &'static str)> {
    let sources = sources_with_claims(d, source_claims);
    for entry in &d.income {
        let crate::FederalIncome::Cooperative { data } = &entry.income else {
            continue;
        };
        let data = &crate::FederalCooperative::from(data);
        if !activities.iter().any(|(_, owner)| *owner == entry.owner.owned()) {
            continue;
        }
        let eligible = eligible_activities(d, entry.owner.owned());
        let applicable = |activity: Option<i64>| match activity {
            Some(id) if eligible.contains(&id) => activities.contains(&(id, entry.owner.owned())),
            // A missing or invalid relationship cannot establish exclusion from
            // this owner's actual C/F activities.
            _ => true,
        };
        let mut relevant = false;
        for source in sources.iter().filter(|source| source.parent == entry.id) {
            if !applicable(source.activity) {
                continue;
            }
            relevant = true;
            if let Some((_, key, _, _)) =
                receipt_requirements_with_claims(d, source, &eligible, source_claims)
                    .into_iter()
                    .find(|(known, _, _, _)| !known)
            {
                return Some((entry.id, key));
            }
        }
        let income = data.credit_facts.other.iter().filter(|item| applicable(item.activity));
        let mut credit_issue = false;
        for item in income {
            relevant = true;
            credit_issue |= !credit_income_known(item, &eligible);
        }
        if relevant && !data.valid() {
            return Some((entry.id, "ISSUE_COOP_SOURCE_INVALID"));
        }
        if credit_issue {
            return Some((entry.id, "ISSUE_COOP_CREDIT_INCOME_NEEDED"));
        }
    }
    None
}

pub(crate) fn report_sources_with_claims(
    d: &crate::FederalDraft,
    e: &crate::FederalIncomeEntry,
    r: &mut crate::FederalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) {
    let crate::FederalIncome::Cooperative { data } = &e.income else {
        return;
    };
    let data = &crate::FederalCooperative::from(data);
    let context = crate::interview::income_issue_context(e);
    let eligible = eligible_activities(d, e.owner.owned());
    for v in sources_with_claims(d, source_claims).into_iter().filter(|v| v.parent == e.id) {
        // Recipient correction is reported once against the accepted catalog.
        for (available, key, _section, _english) in
            receipt_requirements_with_claims(d, &v, &eligible, source_claims).into_iter().skip(1)
        {
            crate::require_localized!(r, available, &context, _section, key, _english,);
        }
    }
    crate::require_localized!(
        r,
        data.valid() && graph_rule_with_claims(d, source_claims).is_none(),
        &context,
        "Cooperative source facts",
        "ISSUE_COOP_SOURCE_INVALID",
        "Correct the cooperative source amounts and component records before estimating.",
    );
}

// Validate accepted graph links without manufacturing a recipient or property ID.
#[cfg(test)]
pub(crate) fn graph_rule(d: &crate::FederalDraft) -> Option<&'static str> {
    let source_claims = crate::related_forms::effective_claimants(d);
    graph_rule_with_claims(d, &source_claims)
}

pub(crate) fn graph_rule_with_claims(
    d: &crate::FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<&'static str> {
    graph_issue_with_claims(d, source_claims).map(|(_, issue)| issue.key)
}

// The ID is the actual accepted/candidate income record, never a request position.
// Local paths address FederalCooperative. The Save owner maps them to Patr1099.more
// and correlates record IDs with submitted documents. A prepared allocation can
// be absent in the original Page: the owner must then target its real collection.
pub(crate) fn graph_issue(d: &crate::FederalDraft) -> Option<(i64, ValidationIssue)> {
    let source_claims = crate::related_forms::effective_claimants(d);
    graph_issue_with_claims(d, &source_claims)
}

pub(crate) fn graph_issue_with_claims(
    d: &crate::FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<(i64, ValidationIssue)> {
    graph_issue_with_reconciliation_scope(d, source_claims, None)
}

// A topic owns reconciliation of its supplied sources. Intrinsic identities,
// bounds and the other graph checks still inspect the complete accepted graph.
pub(crate) fn graph_issue_for_sources(
    d: &crate::FederalDraft,
    sources: &alloc::collections::BTreeSet<i64>,
) -> Option<(i64, ValidationIssue)> {
    let source_claims = crate::related_forms::effective_claimants(d);
    graph_issue_with_reconciliation_scope(d, &source_claims, Some(sources))
}

fn graph_issue_with_reconciliation_scope(
    d: &crate::FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
    reconciliation_sources: Option<&alloc::collections::BTreeSet<i64>>,
) -> Option<(i64, ValidationIssue)> {
    let mut seen = alloc::collections::BTreeSet::new();
    let sources = sources_with_claims(d, source_claims);
    let center = crate::related_forms::catalog(d);
    for e in &d.income {
        let crate::FederalIncome::Cooperative { data } = &e.income else {
            continue;
        };
        let data = &crate::FederalCooperative::from(data);
        if let Some(mut issue) = data.issue() {
            // Preserve the graph gate's historical diagnostic context.
            issue.key = "COOP_ALLOCATION_INVALID";
            return Some((e.id, issue));
        }
        let eligible = eligible_activities(d, e.owner.owned());
        for id in &eligible {
            let governing = sources
                .iter()
                .filter(|s| s.activity == Some(*id))
                .filter(|s| s.allocation.patron_allocation.is_some())
                .count();
            if governing > 1 {
                // The original global condition can fail while examining an
                // unrelated cooperative. Attribute it to a real second source.
                let offending = sources
                    .iter()
                    .filter(|s| s.activity == Some(*id))
                    .filter(|s| s.allocation.patron_allocation.is_some())
                    .nth(1)
                    .expect("multiple governing cooperative sources");
                let issue = ValidationIssue::new("COOP_PATRON_SOURCE_DUPLICATE");
                let issue = match offending.allocation_index {
                    Some(index) => {
                        issue.at_field("patron_allocation").in_list("allocations", index)
                    }
                    None => issue.at_field("allocations"),
                };
                return Some((offending.parent, issue));
            }
        }
        let mut sums = [0; 8];
        for (index, a) in data.allocations.iter().enumerate() {
            if a.id <= 0
                || !seen.insert(a.id)
                || d.income
                    .iter()
                    .any(|e| e.id == a.id || crate::farm::ccc_lot_ids(&e.income).contains(&a.id))
            {
                return Some((
                    e.id,
                    ValidationIssue::new("COOP_ALLOCATION_ID_INVALID")
                        .at_field("id")
                        .in_list("allocations", index),
                ));
            }
            let activity = sources
                .iter()
                .find(|source| source.parent == e.id && source.allocation.id == a.id)
                .and_then(|source| source.activity);
            // A printed property reference can be retained before its component is
            // claimed. Validate its identity against the catalog's eligible owners;
            // only the effective claimant determines its tax treatment.
            let property_activities = match activity {
                Some(activity) => alloc::vec![activity],
                None => center
                    .iter()
                    .find(|component| component.id == a.id)
                    .map(|component| component.eligible.clone())
                    .unwrap_or_default(),
            };
            if let Some(property_index) = a.properties.iter().position(|p| {
                p.property.is_some_and(|property| {
                    crate::farm::sources(d)
                        .filter(|f| property_activities.contains(&f.entry.id))
                        .map(|f| {
                            f.farm.land_basis.iter().filter(|land| land.id == property).count()
                        })
                        .sum::<usize>()
                        + property_activities
                            .iter()
                            .filter(|activity| {
                                crate::source_associations::SourceAssociations::new(d)
                                    .asset(**activity, property)
                                    .is_ok()
                            })
                            .count()
                        != 1
                })
            }) {
                return Some((
                    e.id,
                    ValidationIssue::new("COOP_PROPERTY_LINK_UNAVAILABLE")
                        .at_field("property")
                        .in_list("properties", property_index)
                        .in_list("allocations", index),
                ));
            }
            for (to, n) in sums[..4].iter_mut().zip(a.distributions.values()) {
                *to += n;
            }
            for (to, n) in sums[4..].iter_mut().zip([
                a.qualified_payments,
                a.qualified_items,
                a.sstb_items,
                a.section_199a_g,
            ]) {
                *to += n;
            }
        }
        if !data.allocations.is_empty()
            && reconciliation_sources.is_none_or(|sources| sources.contains(&e.id))
            && sums
                != [
                    data.patronage,
                    data.nonpatronage,
                    data.per_unit_retains,
                    data.redeemed,
                    data.qualified_payments,
                    data.qualified_items,
                    data.sstb_items,
                    data.section_199a_g,
                ]
        {
            return Some((
                e.id,
                ValidationIssue::new("COOP_ALLOCATIONS_DO_NOT_RECONCILE").at_field("allocations"),
            ));
        }
        for (index, a) in data.allocations.iter().enumerate() {
            let Some(id) = sources
                .iter()
                .find(|source| source.parent == e.id && source.allocation.id == a.id)
                .and_then(|source| source.activity)
            else {
                continue;
            };
            let Some(business) = d.income.iter().find_map(|e| match &e.income {
                crate::FederalIncome::Business { data } if e.id == id => Some(data),
                _ => None,
            }) else {
                if !a.sstb_expenses.is_empty() {
                    return Some((
                        e.id,
                        ValidationIssue::new("COOP_SSTB_EXPENSE_INVALID")
                            .at_field("sstb_expenses")
                            .in_list("allocations", index),
                    ));
                }
                continue;
            };
            if matches!(a.patron_allocation, Some(CoopPatronAllocation::NoWages))
                && business.payroll.qbi_wages() != 0
            {
                return Some((
                    e.id,
                    ValidationIssue::new("COOP_PATRON_WAGES_NOT_ZERO")
                        .at_field("patron_allocation")
                        .in_list("allocations", index),
                ));
            }
            for (expense_index, expense) in a.sstb_expenses.iter().enumerate() {
                if sources
                    .iter()
                    .filter(|s| s.activity == Some(id))
                    .flat_map(|s| &s.allocation.sstb_expenses)
                    .filter(|x| x.source == expense.source)
                    .count()
                    > 1
                {
                    return Some((
                        e.id,
                        ValidationIssue::new("COOP_SSTB_SOURCE_REPEATED")
                            .at_field("source")
                            .in_list("sstb_expenses", expense_index)
                            .in_list("allocations", index),
                    ));
                }
                let total: i64 = sources
                    .iter()
                    .filter(|s| s.activity == Some(id))
                    .flat_map(|s| &s.allocation.sstb_expenses)
                    .filter(|x| x.category == expense.category)
                    .map(|x| x.amount)
                    .sum();
                let entries: Vec<_> = business
                    .expenses
                    .iter()
                    .filter(|x| x.category == expense.category)
                    .cloned()
                    .collect();
                if total > crate::FederalBusinessExpense::total(&entries) {
                    return Some((
                        e.id,
                        ValidationIssue::new("COOP_SSTB_EXPENSE_INVALID")
                            .at_field("amount")
                            .in_list("sstb_expenses", expense_index)
                            .in_list("allocations", index),
                    ));
                }
            }
        }
        if data.credit_facts.activity.is_some_and(|id| !eligible.contains(&id)) {
            return Some((
                e.id,
                ValidationIssue::new("COOP_RECIPIENT_LINK_INVALID")
                    .at_field("activity")
                    .in_field("credit_facts"),
            ));
        }
        if let Some(index) = data
            .credit_facts
            .other
            .iter()
            .position(|v| v.activity.is_some_and(|id| !eligible.contains(&id)))
        {
            return Some((
                e.id,
                ValidationIssue::new("COOP_RECIPIENT_LINK_INVALID")
                    .at_field("activity")
                    .in_list("other", index)
                    .in_field("credit_facts"),
            ));
        }
    }
    None
}

pub(crate) type QbiFiling = (Vec<[i64; 2]>, i64, [i64; 2], Vec<i64>);

#[cfg(test)]
pub(crate) fn qbi_filing(
    d: &crate::FederalDraft,
    activities: &mut [crate::business_rules::QbiActivity],
    business_ids: &alloc::collections::BTreeMap<usize, i64>,
    taxable: i64,
    status: crate::FederalStatus,
    _prior_loss: i64,
    r: &mut crate::FederalResult,
) -> QbiFiling {
    let source_claims = crate::related_forms::effective_claimants(d);
    qbi_filing_with_claims(
        crate::related_forms::RelatedFormContext::original(d, &source_claims),
        activities,
        business_ids,
        taxable,
        status,
        _prior_loss,
        r,
    )
}

#[cfg(test)]
pub(crate) fn qbi_filing_with_claims(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    activities: &mut [crate::business_rules::QbiActivity],
    business_ids: &alloc::collections::BTreeMap<usize, i64>,
    taxable: i64,
    status: crate::FederalStatus,
    _prior_loss: i64,
    r: &mut crate::FederalResult,
) -> QbiFiling {
    let crate::related_forms::RelatedFormContext { draft: d, claims: source_claims, .. } =
        form_context;
    match qbi_filing_with_section179_with_claims(
        d,
        activities,
        business_ids,
        taxable,
        status,
        None,
        r,
        None,
        source_claims,
    ) {
        Ok(value) => value,
        Err(_) => unreachable!("legacy section179 path is infallible"),
    }
}

// Forward the existing invocation operands and its borrowed home context together.

#[allow(clippy::too_many_arguments)]
pub(crate) fn qbi_filing_with_section179_with_claims(
    d: &crate::FederalDraft,
    activities: &mut [crate::business_rules::QbiActivity],
    business_ids: &alloc::collections::BTreeMap<usize, i64>,
    taxable: i64,
    status: crate::FederalStatus,
    selected: Option<&crate::Section179PreparedAllocation>,
    r: &mut crate::FederalResult,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<QbiFiling, crate::Section179PreparedIssue> {
    use crate::kernel::ratio::{dollars, fraction, positive};
    let sources = sources_with_claims(d, source_claims);
    let mut reductions = alloc::vec![[0;2]; activities.len()];
    let mut dpad = 0;
    let mut owner_dpad = [0; 2];
    let mut dpad_weights = alloc::vec![0;activities.len()];
    for (index, id) in business_ids {
        let Some(entry) = d.income.iter().find(|e| e.id == *id) else {
            continue;
        };
        let crate::FederalIncome::Business { data: business } = &entry.income else {
            continue;
        };
        let shares: Vec<_> = sources.iter().filter(|s| s.activity == Some(*id)).collect();
        if shares.is_empty() {
            continue;
        }
        let receipts =
            receipt_totals_with_claims(d, entry.owner.owned(), *id, false, source_claims)?;
        let original = activities[*index].clone();
        let share = d.setup.as_ref().and_then(|setup| {
            crate::community::share(setup, entry.owner.owned(), entry.community, &entry.income)
        });
        let divide = |amount: i64| share.map_or(amount, |s| s.filer(amount));
        let mut eligible_gross = receipts.qualification[0];
        let mut sstb = receipts.qualification[2];
        let mut related_expenses = receipts.related_sstb_expenses;
        let mut books = [0; 3];
        let mut books_complete = false;
        let mut safe_harbor = false;
        let mut qualification_complete = true;
        for share in &shares {
            let a = &share.allocation;
            let context = crate::IssueContext::income_id(share.parent);
            if let Some(q) = a.retained_qualification() {
                related_expenses +=
                    phase::cap(q[2], [a.sstb_expenses.iter().map(|e| e.amount).sum::<i64>()]);
            } else {
                qualification_complete = false;
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new("WARN_COOP_QUALIFICATION_LEFT_OUT", alloc::vec![]),
                    context.clone(),
                );
            }
            if a.section_199a_g > 0 {
                if matches!(&a.notice, Some(CoopNoticeStatus::Received { .. })) {
                    let amount = divide(a.section_199a_g);
                    dpad += amount;
                    owner_dpad[usize::from(share.owner == crate::Owner::Spouse)] += amount;
                    dpad_weights[*index] += amount;
                } else {
                    crate::federal_rules::note_once_scoped(
                        r,
                        crate::Message::new("WARN_COOP_DPAD_NOTICE_NEEDED", alloc::vec![]),
                        context.clone(),
                    );
                }
            }
        }
        let governing: Vec<_> =
            shares.iter().filter_map(|s| s.allocation.patron_allocation.as_ref()).collect();
        if governing.len() == 1 {
            match governing[0] {
                CoopPatronAllocation::Books {
                    expenses_excluding_wages,
                    wages,
                    other_deductions,
                    ..
                } => {
                    books = [*expenses_excluding_wages, *wages, *other_deductions];
                    books_complete = true;
                }
                CoopPatronAllocation::SafeHarbor => safe_harbor = true,
                _ => {}
            }
        }
        eligible_gross = divide(eligible_gross);
        sstb = divide(sstb);
        related_expenses = phase::cap(divide(related_expenses), [sstb]);
        books = books.map(divide);
        // The cooperative SSTB item has its own eligibility classification;
        // patron wages/UBIA remain the actual directly conducted trade's.
        // Related paid expenses move classes without a second tax deduction.
        if sstb > 0 {
            let service = crate::business_rules::QbiActivity {
                source_wages_cents: None,
                source_property_cents: None,
                source_income_cents: None,
                owner: None,
                income: dollars(sstb - related_expenses),
                wages: 0,
                property: 0,
                active: original.active,
                service: Some(true),
                group: None,
            };
            let eligible = if original.service != Some(false) {
                service.income
            } else {
                crate::business_rules::eligible_qbi_activity(&service, taxable, status)
                    .map_or(0, |a| a.income)
            };
            let adjustment = dollars(related_expenses) + eligible;
            activities[*index].income += adjustment;
            if let Some(source) = activities[*index].source_income_cents {
                let adjustment = crate::extended_rules::annual_dollars_to_working_cents(adjustment)
                    .map_err(|_| crate::Section179PreparedIssue::AmountOverflow(*id))?;
                activities[*index].source_income_cents = Some(
                    source
                        .checked_add(adjustment)
                        .ok_or(crate::Section179PreparedIssue::AmountOverflow(*id))?,
                );
            }
        }
        let wage_cents = divide(business.payroll.qbi_wages());
        if eligible_gross == 0 || wage_cents == 0 {
            continue;
        }
        let activity_cents = positive(activities[*index].income * 100);
        let gross =
            divide(business.receipts() + receipts.qualification[1] + receipts.qualification[2]);
        let expenses = divide(crate::section179_return::expenses_with_claims(
            d,
            *id,
            business,
            selected,
            home,
            source_claims,
        )?);
        // Reconcile the actual below-line deduction pool before adding or
        // phasing the cooperative SSTB class. Excluded SSTB income and an
        // overall loss are not extra deduction-allocation capacity.
        let ordinary_gross = divide(business.receipts() + receipts.qualification[1]);
        let qbi_other = positive(ordinary_gross - expenses - original.income * 100);
        let source_values = if books_complete
            && books[1] <= wage_cents
            && books[0] <= positive(expenses - wage_cents)
            && books[2] <= qbi_other
        {
            Some((positive(eligible_gross - books.iter().sum::<i64>()), books[1]))
        } else if safe_harbor
            && crate::business_rules::qbi_phase(taxable, status).0 == 0
            && gross > 0
            && eligible_gross <= gross
        {
            Some((
                fraction(activity_cents, eligible_gross, gross),
                fraction(wage_cents, eligible_gross, gross),
            ))
        } else {
            None
        };
        if let Some((income, wages)) = source_values.filter(|_| qualification_complete) {
            let qualified = crate::business_rules::QbiActivity {
                source_wages_cents: None,
                source_property_cents: None,
                source_income_cents: None,
                owner: None,
                income: dollars(income),
                wages: dollars(wages),
                property: 0,
                active: original.active,
                service: original.service,
                group: None,
            };
            if let Ok(value) =
                crate::business_rules::eligible_qbi_activity(&qualified, taxable, status)
            {
                reductions[*index] = [value.income, value.wages];
            }
        } else {
            // Actual allocation evidence is missing; known taxable income
            // and qualified loss history remain intact.
            if activities[*index].income > 0 {
                activities[*index].income = phase::cap(activities[*index].income, [0]);
            }
            activities[*index].wages = 0;
            activities[*index].property = 0;
            activities[*index].source_income_cents =
                activities[*index].source_income_cents.map(|source| source.min(0));
            activities[*index].source_wages_cents =
                activities[*index].source_wages_cents.map(|_| 0);
            activities[*index].source_property_cents =
                activities[*index].source_property_cents.map(|_| 0);
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("WARN_COOP_PATRON_ALLOCATION_NEEDED", alloc::vec![]),
                crate::IssueContext::income_id(*id),
            );
        }
    }
    // Form 4835 and pass-through QBI need their actual accepted statement
    // interface; source income is still reported by its existing owner.
    for share in sources.iter().filter(|s| !business_ids.values().any(|id| Some(*id) == s.activity))
    {
        if share.allocation.qualified_items
            + share.allocation.sstb_items
            + share.allocation.section_199a_g
            > 0
        {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("WARN_COOP_QBI_RECIPIENT_NEEDED", alloc::vec![]),
                crate::IssueContext::income_id(share.parent),
            );
        }
    }
    Ok((
        reductions,
        dollars(dpad),
        owner_dpad,
        crate::kernel::ratio::allocate_remaining(dollars(dpad), &dpad_weights),
    ))
}

// Section 175 gross is production income, not every taxable Schedule F receipt.
// Keep this economic classification at the cooperative source owner; Farm owns
// the combined limit, deductions and carry records.
#[cfg(test)]
pub(crate) fn conservation_gross(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    activity: i64,
) -> Option<i64> {
    let source_claims = crate::related_forms::effective_claimants(d);
    conservation_gross_with_claims(d, owner, activity, &source_claims)
}

pub(crate) fn conservation_gross_with_claims(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    activity: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<i64> {
    // Required credit income is retained in ordinary profit, but this source
    // shape does not establish that it arose from agricultural production.
    if additional_income(d, owner, activity) != 0 {
        return None;
    }
    sources_with_claims(d, source_claims)
        .into_iter()
        .filter(|s| s.owner == owner && s.activity == Some(activity))
        .try_fold(0, |gross, s| {
            let ordinary = s.allocation.ordinary();
            if ordinary == 0 {
                return Some(gross);
            }
            match s.allocation.income_use {
                CoopIncomeUse::ProductionSales => Some(gross + ordinary),
                CoopIncomeUse::DeductiblePurchases => Some(gross),
                CoopIncomeUse::Other { .. } | CoopIncomeUse::Unresolved => None,
            }
        })
}

// Native-cent receipt output for C/F/4835. Legally equivalent amounts
// combine inside their source owner before its once-cent output projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReceiptTotals {
    pub distributions: i64,
    pub taxable: i64,
    pub nonqualified: i64,
    pub qualification: [i64; 3],
    pub related_sstb_expenses: i64,
}
#[cfg(test)]
pub(crate) fn receipt_totals(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    activity: i64,
    alternative: bool,
) -> Result<ReceiptTotals, crate::Section179PreparedIssue> {
    let source_claims = crate::related_forms::effective_claimants(d);
    receipt_totals_with_claims(d, owner, activity, alternative, &source_claims)
}

pub(crate) fn receipt_totals_with_claims(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    activity: i64,
    alternative: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<ReceiptTotals, crate::Section179PreparedIssue> {
    use crate::kernel::{ratio::exact, source_fact::SourceFact};
    let additional = additional_income(d, owner, activity);
    let mut distributions = 0_i64;
    let mut related_sstb_expenses = 0;
    let mut taxable = exact::whole(additional);
    let mut nonqualified = exact::whole(additional);
    let mut qualification = [exact::whole(0), exact::whole(0), exact::whole(0)];
    for source in sources_with_claims(d, source_claims)
        .into_iter()
        .filter(|source| source.owner == owner && source.activity == Some(activity))
    {
        let ordinary = source.allocation.ordinary();
        let qualified = source.allocation.retained_qualification().unwrap_or([0; 3]);
        distributions = distributions
            .checked_add(source.allocation.distributions.total())
            .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?;
        taxable += exact::whole(ordinary);
        nonqualified += exact::whole(phase::floor(ordinary - qualified[1], 0));
        for (target, amount) in qualification.iter_mut().zip(qualified) {
            *target += exact::whole(amount);
        }
    }
    match crate::cooperative_basis::income_with_claims(
        d,
        owner,
        activity,
        alternative,
        source_claims,
    )? {
        SourceFact::NotReported => (),
        SourceFact::Unavailable => {
            return Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                activity,
                "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
            ));
        }
        SourceFact::Reported(excess) => {
            related_sstb_expenses = excess.related_sstb_expenses;
            nonqualified += exact::whole(excess.ordinary - excess.qualification[1]);
            taxable += exact::whole(excess.ordinary);
            for (target, amount) in qualification.iter_mut().zip(excess.qualification) {
                *target += exact::whole(amount);
            }
        }
    }
    let overflow = || crate::Section179PreparedIssue::AmountOverflow(activity);
    let [Some(payments), Some(non_sstb), Some(sstb)] =
        qualification.map(|amount| exact::rounded(&amount))
    else {
        return Err(overflow());
    };
    Ok(ReceiptTotals {
        distributions,
        taxable: exact::rounded(&taxable).ok_or_else(overflow)?,
        nonqualified: exact::rounded(&nonqualified).ok_or_else(overflow)?,
        qualification: [payments, non_sstb, sstb],
        related_sstb_expenses,
    })
}

pub(crate) fn additional_income(
    d: &crate::FederalDraft,
    owner: crate::Owner,
    activity: i64,
) -> i64 {
    d.income
        .iter()
        .filter(|e| e.owner == crate::DocumentOwner::from(owner))
        .filter_map(|e| match &e.income {
            crate::FederalIncome::Cooperative { data } => {
                Some(crate::FederalCooperative::from(data).credit_facts.other)
            }
            _ => None,
        })
        .flatten()
        .filter(|v| v.activity == Some(activity))
        .map(|v| match v.income {
            CoopCreditIncome::Inclusion { amount, .. } => amount,
            _ => 0,
        })
        .sum()
}

pub(crate) fn credit_entries(
    d: &crate::FederalDraft,
    r: &mut crate::FederalResult,
) -> Vec<crate::general_business_credit::CreditEntry> {
    use crate::{BusinessCredit as C, general_business_credit::CreditEntry};
    let mut result = Vec::new();
    for e in &d.income {
        let crate::FederalIncome::Cooperative { data } = &e.income else {
            continue;
        };
        let v = &crate::FederalCooperative::from(data).credit_facts;
        let eligible = eligible_activities(d, e.owner.owned());
        let context = crate::IssueContext::income_id(e.id);
        let notice = matches!(&v.notice, Some(CoopNoticeStatus::Received { .. }));
        let mut add =
            |credit: C, amount: i64, activity: Option<i64>, r: &mut crate::FederalResult| {
                if amount == 0 {
                    return;
                }
                if !notice || !activity.is_some_and(|id| eligible.contains(&id)) {
                    crate::federal_rules::note_once_scoped(
                        r,
                        crate::Message::new("WARN_COOP_CREDIT_SOURCE_NEEDED", alloc::vec![]),
                        context.clone(),
                    );
                    return;
                }
                if v.participation.is_none() {
                    crate::federal_rules::note_once_scoped(
                        r,
                        crate::Message::new("WARN_COOP_CREDIT_PASSIVE", alloc::vec![]),
                        context.clone(),
                    );
                }
                result.push(CreditEntry {
                    source_owner: Some(e.owner.owned()),
                    conditional_origin: None,
                    transfer: 0,
                    elective: 0,
                    activity: None,
                    filer: crate::CarryFiler::SameFilers,
                    credit,
                    year: crate::year_constants::TAX_YEAR,
                    amount: crate::kernel::ratio::dollars(amount),
                    passive: (v.participation != Some(true)).then_some(credit.worksheet()),
                });
            };
        if v.investment > 0 && v.investment_shares.is_empty() {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("WARN_COOP_CREDIT_SOURCE_NEEDED", alloc::vec![]),
                context.clone(),
            );
        }
        for share in &v.investment_shares {
            let credit = match share.kind {
                CoopInvestmentKind::Rehabilitation => Some(C::Rehabilitation),
                CoopInvestmentKind::Energy => Some(C::Energy),
                CoopInvestmentKind::CleanElectricity => Some(C::CleanElectricity),
                _ => None,
            };
            if let Some(credit) = credit {
                add(credit, share.amount, v.activity, r);
            } else if share.amount > 0 {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_COOP_OTHER_BENEFIT_LEFT_OUT",
                        alloc::vec![
                            crate::MessageArg::Text(match &share.kind {
                                CoopInvestmentKind::Other { description } => description.clone(),
                                _ => share.source.clone(),
                            }),
                            crate::MessageArg::Text(share.source.clone()),
                        ],
                    ),
                    context.clone(),
                );
            }
        }
        add(C::WorkOpportunity, v.work_opportunity, v.activity, r);
        for item in &v.other {
            let credit = match item.kind {
                CoopOtherKind::CleanFuel => Some(C::CleanFuel),
                CoopOtherKind::SmallEmployerHealth => Some(C::SmallEmployerHealth),
                _ => None,
            };
            if let Some(credit) = credit {
                add(credit, item.amount, item.activity, r);
            } else if item.amount > 0 {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_COOP_OTHER_BENEFIT_LEFT_OUT",
                        alloc::vec![
                            match &item.kind {
                                CoopOtherKind::EmpowermentZone =>
                                    crate::MessageArg::Resource("COOP_EMPOWERMENT_ZONE".into()),
                                CoopOtherKind::LowSulfurDiesel =>
                                    crate::MessageArg::Resource("COOP_LOW_SULFUR_DIESEL".into()),
                                CoopOtherKind::DifferentialWage =>
                                    crate::MessageArg::Resource("COOP_DIFFERENTIAL_WAGE".into()),
                                CoopOtherKind::SmallAgriBiodiesel =>
                                    crate::MessageArg::Resource("COOP_SMALL_AGRI_BIODIESEL".into()),
                                CoopOtherKind::SmallRefinerDeduction =>
                                    crate::MessageArg::Resource("COOP_SMALL_REFINER".into()),
                                CoopOtherKind::Other { description } =>
                                    crate::MessageArg::Text(description.clone()),
                                _ => crate::MessageArg::Text(item.source.clone()),
                            },
                            crate::MessageArg::Text(item.source.clone()),
                        ],
                    ),
                    context.clone(),
                );
            }
            crate::require_localized!(
                r,
                credit_income_known(item, &eligible),
                &context,
                "Cooperative credit income",
                "ISSUE_COOP_CREDIT_INCOME_NEEDED",
                "The actual source does not establish the required credit-related income or its accepted recipient. No required income is guessed.",
            );
        }
    }
    result
}
