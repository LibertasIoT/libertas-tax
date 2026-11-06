// Native reported cash categories and actual source-property relationships.
// Document order never determines a money residual. Equivalent legal bins
// are combined before exact source-cent weighting (Money precision contract).
use super::{RefundFunds, owner_index};
use crate::{FederalDraft, FederalIncomeEntry, FederalResult, Owner};

// Retained by the existing line20 projection, not a second payment allocator.
#[derive(Default)]
pub(super) struct PaymentFunding {
    pub(super) direct_dollars: [[i64; 2]; 2],
    pub(super) joint_dollars: [i64; 2],
    pub(super) estate_cents: Option<[crate::kernel::ratio::exact::Exact; 2]>,
}

#[derive(Clone, PartialEq, Eq)]
struct PaymentBin {
    owner: Owner,
    community: crate::kernel::ratio::exact::Exact,
    management: Option<crate::RefundManagement>,
}

fn payment_bin(
    amount: i64,
    property: &crate::RefundSourcePropertyFacts,
    injured: Owner,
    state: u16,
) -> Option<PaymentBin> {
    use crate::kernel::ratio::exact::{ratio, whole};
    if amount <= 0 {
        return None;
    }
    let (owner, community, management) = match *property {
        crate::RefundSourcePropertyFacts::Separate { owner } => (owner, whole(0), None),
        crate::RefundSourcePropertyFacts::Community { management } => {
            (injured, whole(1), Some(management))
        }
        crate::RefundSourcePropertyFacts::Partly { community, separate_owner, management }
            if (0..=amount).contains(&community) =>
        {
            (separate_owner, ratio(community, amount)?, Some(management))
        }
        _ => return None,
    };
    Some(PaymentBin {
        // Pure community has no separate-person component to distinguish.
        owner: if community == whole(1) { injured } else { owner },
        management: (state == 7 && community > whole(0)).then_some(management).flatten(),
        community,
    })
}

pub(super) fn add_payments(
    d: &FederalDraft,
    native: &PaymentFunding,
    injured: Owner,
    state: u16,
    funds: &mut RefundFunds,
) -> Option<()> {
    use crate::kernel::ratio::exact::whole;
    // The actual income owner does not establish refund-fund property.
    if native.estate_cents.as_ref()?.iter().any(|v| *v != whole(0)) {
        return None;
    }
    let pay = d.payments.as_ref()?;
    for category in 0..2 {
        for pool in 0..3 {
            let dollars = if pool == 2 {
                native.joint_dollars[category]
            } else {
                native.direct_dollars[category][pool]
            };
            if dollars < 0 {
                return None;
            }
            if dollars == 0 {
                continue;
            }
            let payer = if pool == 0 {
                crate::PaymentPayer::Taxpayer
            } else if pool == 1 {
                crate::PaymentPayer::Spouse
            } else {
                crate::PaymentPayer::Joint
            };
            let mut bins: alloc::vec::Vec<(PaymentBin, crate::kernel::ratio::exact::Exact)> =
                alloc::vec::Vec::new();
            let mut total = whole(0);
            let mut add =
                |amount: i64, property: Option<&crate::RefundSourcePropertyFacts>| -> Option<()> {
                    if amount < 0 {
                        return None;
                    }
                    if amount == 0 {
                        return Some(());
                    }
                    let bin = payment_bin(amount, property?, injured, state)?;
                    total += whole(amount);
                    if let Some((_, weight)) = bins.iter_mut().find(|(known, _)| *known == bin) {
                        *weight += whole(amount);
                    } else {
                        bins.push((bin, whole(amount)));
                    }
                    Some(())
                };
            if category == 0 {
                for (amount, _, property) in pay.estimated.funding().filter(|(_, p, _)| *p == payer)
                {
                    add(amount, property)?;
                }
            } else if pay.extension_payer == Some(payer) {
                add(pay.extension, pay.extension_refund_property.as_ref())?;
            }
            if total == whole(0) {
                return None;
            }
            for (bin, raw) in bins {
                let amount = whole(dollars.checked_mul(100)?) * (raw / &total);
                let community = &amount * bin.community;
                funds.source(
                    amount,
                    community,
                    bin.owner,
                    (injured, state),
                    bin.management,
                    false,
                )?;
            }
        }
    }
    Some(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Bin {
    community_percent: u32,
    management: Option<crate::RefundManagement>,
    earned: bool,
}

fn bin(
    d: &FederalDraft,
    entry: &FederalIncomeEntry,
    state: u16,
    earned_needed: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<Bin> {
    let percent =
        crate::community::refund_community_percent(&entry.income, entry.community, state)?;
    Some(Bin {
        community_percent: percent,
        // These facts affect only the states whose existing legal rule reads
        // them. Irrelevant accepted facts are retained, not erased.
        management: (state == 7 && percent > 0).then_some(entry.refund_management).flatten(),
        earned: if percent > 0 && earned_needed {
            crate::community::refund_earned(d, entry, source_claims)?
        } else {
            false
        },
    })
}

pub(super) fn management_needed(
    d: &FederalDraft,
    r: &FederalResult,
    entry: &FederalIncomeEntry,
) -> Option<bool> {
    if !super::texas(d) {
        return Some(false);
    }
    if !r.financial_source_scope_complete {
        return None;
    }
    // Source/property demand is decided before a planned line 36 election.
    match (r.credit_capacity, r.balance) {
        (Some(0), Some(balance)) if balance >= 0 => return Some(false),
        (Some(refund), Some(0)) if refund > 0 => {}
        _ => return None,
    }
    let percent = crate::community::refund_community_percent(&entry.income, entry.community, 7)?;
    if percent == 0 {
        return Some(false);
    }
    let first = d.injured_spouse.as_ref().map_or(0, |a| owner_index(a.injured));
    let owners = r.withholding_sources.as_deref()?.projected(d, first)?;
    let sources = r.withholding_sources.as_deref()?;
    let mut needed = false;
    for (category, category_owners) in owners.iter().enumerate() {
        let matching = sources.rows.iter().find(|row| row.id == entry.id);
        if matching.is_none_or(|row| row.cents[category] == 0)
            || category_owners[owner_index(entry.owner.owned())] == 0
        {
            continue;
        }
        // Approved exact source weights are positive for each positive row;
        // no document receives an arbitrary rounded residual.
        needed = true;
    }
    Some(needed)
}

pub(super) fn add_withholding(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    r: &FederalResult,
    injured: Owner,
    state: u16,
    earned_needed: bool,
    funds: &mut RefundFunds,
) -> Option<()> {
    let d = form_context.draft;
    let source_claims = form_context.claims;
    let owners = r.withholding_sources.as_deref()?.projected(d, owner_index(injured))?;
    let sources = r.withholding_sources.as_deref()?;
    for (category, amounts) in owners.into_iter().enumerate() {
        for (person, dollars) in amounts.into_iter().enumerate() {
            if dollars == 0 {
                continue;
            }
            let owner = if person == 0 { Owner::Taxpayer } else { Owner::Spouse };
            use crate::kernel::ratio::exact::{ratio, whole};
            let mut bins: alloc::vec::Vec<(Bin, i64)> = alloc::vec::Vec::new();
            let mut raw_total = 0_i64;
            for row in
                sources.rows.iter().filter(|row| row.owner == owner && row.cents[category] > 0)
            {
                let entry = d.income.iter().find(|entry| entry.id == row.id)?;
                let current = bin(d, entry, state, earned_needed, source_claims)?;
                let raw = row.cents[category];
                raw_total = raw_total.checked_add(raw)?;
                if let Some((_, weight)) =
                    bins.iter_mut().find(|(previous, _)| *previous == current)
                {
                    *weight = weight.checked_add(raw)?;
                } else {
                    bins.push((current, raw));
                }
            }
            let reported_cents = dollars.checked_mul(100)?;
            for (legal, raw) in bins {
                let amount = whole(reported_cents) * ratio(raw, raw_total)?;
                let community = &amount * ratio(i64::from(legal.community_percent), 100)?;
                funds.source(
                    amount,
                    community,
                    owner,
                    (injured, state),
                    legal.management,
                    legal.earned,
                )?;
            }
        }
    }
    Some(())
}
