// schedule_f items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// Why a routed farm document has no farm or farm rental to receive it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Routing {
    // The owner has no farm or farm rental.
    Missing,
    // The owner has several and the document names none.
    Several,
}
// Farm receipts use the same component claimant as the activity editor.
pub(crate) fn routed_farm(
    d: &FederalDraft,
    owner: usize,
    component: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, Routing> {
    if let Some(id) = source_claims.get(&component) {
        return Ok(*id);
    }
    if crate::farm::sources(d)
        .any(|source| usize::from(source.entry.owner == crate::DocumentOwner::Spouses) == owner)
    {
        Err(Routing::Several)
    } else {
        Err(Routing::Missing)
    }
}
// Box 9 verifies the owner's complete set of printed statements against the
// owner's own CCC lots. The lots already recognize the taxable gain; the paper
// is corroboration, never another receipt. Aggregate before comparing so split
// statements and several farms do not each have to equal the owner's total.
pub(super) fn ccc_market_gain_reconciled(d: &FederalDraft, owner: crate::DocumentOwner) -> bool {
    let Some(owner) = owner.tax_owner() else {
        return false;
    };
    let printed: i64 = d
        .income
        .iter()
        .filter(|entry| entry.owner.tax_owner() == Some(owner))
        .filter_map(|entry| match &entry.income {
            FederalIncome::GovernmentPayment { data } => Some(data.form.market_gain.unwrap_or(0)),
            _ => None,
        })
        .sum();
    let calculated: i64 = crate::farm::sources(d)
        .filter(|source| source.entry.owner.tax_owner() == Some(owner))
        .filter_map(|source| source.farm.ccc.as_ref())
        .map(|ccc| ccc.part_one()[0])
        .sum();
    printed == calculated
}

// Keep the report's source predicates and diagnostics at the receipt owner.
// The bool following each key identifies its existing corrective context.
pub(super) fn farm_receipt_requirements(
    data: &crate::FederalUnemployment,
    routing: Result<i64, Routing>,
    market_gain_reconciled: bool,
) -> [(bool, &'static str, bool, &'static str); 3] {
    [
        (
            data.market_gain == 0 || market_gain_reconciled,
            "ISSUE_FORM_1099_G_BOX_9_MARKET_GAIN_CANNOT_BE_RECONCILED_TO_THIS_RETURN_S_165FF0EF",
            false,
            "Form 1099-G box 9 market gain cannot be reconciled to this return's own Commodity Credit Corporation loan calculation.",
        ),
        (
            data.agriculture_payment == 0 || routing != Err(Routing::Missing),
            "ISSUE_FORM_1099_G_BOX_7_AGRICULTURE_PAYMENTS_NEED_AN_EXISTING_FARM_ACTIVI_99130ACC",
            true,
            "Form 1099-G box 7 agriculture payments need an existing farm or farm rental on this return for the same owner before they can reach Schedule F line 4a/4b or Form 4835 line 3.",
        ),
        (
            data.agriculture_payment == 0 || !matches!(routing, Err(Routing::Several)),
            "ISSUE_FORM_1099_G_BOX_7_AGRICULTURE_PAYMENTS_NEED_A_CHOSEN_FARM",
            false,
            "Form 1099-G box 7 agriculture payments need the farm or farm rental that receives them: relate this payment on the receiving farm or farm rental's page.",
        ),
    ]
}

// `owner`'s 2026 Form 1099-G box 7 agriculture-program payments routed to
// this farm or farm rental (`routed_farm`), so this owner's SE-tax total
// (`Totals::business`), QBI activity income and that activity's own Schedule F
// line 4a/4b or Form 4835 line 3 display lines use the same combined figure.
// Returns 0 for every other activity id, including every non-farm business.
pub(crate) fn farm_1099g_program(
    d: &FederalDraft,
    owner: usize,
    farm_id: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> i64 {
    d.income
        .iter()
        .filter(|e| usize::from(e.owner == crate::DocumentOwner::Spouses) == owner)
        .filter_map(|e| match &e.income {
            FederalIncome::GovernmentPayment { data } => {
                Some(crate::FederalUnemployment::from(data))
            }
            _ => None,
        })
        .filter(|data| data.agriculture_payment != 0)
        .filter(|data| {
            routed_farm(d, owner, data.component_ids.agriculture, source_claims) == Ok(farm_id)
        })
        .map(|data| data.agriculture_payment)
        .sum()
}
// Form 1099-PATR totals of the owner's cooperative documents routed to this
// farm or farm rental's line 3a/3b like the 1099-G box 7 payments: every
// other activity sees zeros.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FarmCooperativeTotals {
    pub distributions: i64,
    pub taxable: i64,
    pub nonqualified: i64,
    pub qualified_payments: i64,
}
#[cfg(test)]
pub(crate) fn farm_cooperative(
    d: &FederalDraft,
    owner: usize,
    farm_id: i64,
) -> Result<FarmCooperativeTotals, crate::Section179PreparedIssue> {
    let source_claims = crate::related_forms::effective_claimants(d);
    farm_cooperative_with_claims(d, owner, farm_id, &source_claims)
}

pub(crate) fn farm_cooperative_with_claims(
    d: &FederalDraft,
    owner: usize,
    farm_id: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<FarmCooperativeTotals, crate::Section179PreparedIssue> {
    let owner = if owner == 0 { Owner::Taxpayer } else { Owner::Spouse };
    let totals =
        crate::cooperative::receipt_totals_with_claims(d, owner, farm_id, false, source_claims)?;
    Ok(FarmCooperativeTotals {
        distributions: totals.distributions,
        taxable: totals.taxable,
        nonqualified: totals.nonqualified,
        qualified_payments: totals.qualification[0],
    })
}
// Farm income that enters this farm's profit from the owner's documents
// rather than the farm's own records: 1099-G boxes 6/7 and taxable
// 1099-PATR distributions.

pub(crate) fn farm_routed_income_with_refunds_with_claims(
    d: &FederalDraft,
    owner: usize,
    activity: i64,
    alternative: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::Section179PreparedIssue> {
    let person = if owner == 0 { Owner::Taxpayer } else { Owner::Spouse };
    let cooperative = crate::cooperative::receipt_totals_with_claims(
        d,
        person,
        activity,
        alternative,
        source_claims,
    )?;
    farm_1099g_program(d, owner, activity, source_claims)
        .checked_add(crate::farm_insurance_paper::income_delta(d, activity, source_claims))
        .and_then(|amount| amount.checked_add(cooperative.taxable))
        .and_then(|amount| {
            amount.checked_add(crate::information_returns::farm_grants_with_claims(
                d,
                activity,
                source_claims,
            ))
        })
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))
}

pub(crate) fn farm_receipts_with_refunds_with_claims(
    d: &FederalDraft,
    activity: i64,
    business: &crate::FederalBusiness,
    alternative: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::Section179PreparedIssue> {
    let owner = d
        .income
        .iter()
        .find(|entry| entry.id == activity)
        .map(|entry| usize::from(entry.owner == crate::DocumentOwner::Spouses))
        .ok_or(crate::Section179PreparedIssue::UnknownSource(activity))?;
    business
        .receipts()
        .checked_add(farm_routed_income_with_refunds_with_claims(
            d,
            owner,
            activity,
            alternative,
            source_claims,
        )?)
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))
}

pub(crate) fn farm_routed_income_with_claims(
    d: &FederalDraft,
    owner: usize,
    farm_id: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::Section179PreparedIssue> {
    farm_routed_income_with_refunds_with_claims(d, owner, farm_id, false, source_claims)
}
// The farm-wide income figure: the farm's own receipts plus the routed income
// above. The section 175 limit, the Schedule J farm income and the at-risk
// loss test read it, so none of them has a copy of its own.

pub(crate) fn farm_receipts_with_claims(
    d: &FederalDraft,
    farm_id: i64,
    b: &crate::FederalBusiness,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::Section179PreparedIssue> {
    farm_receipts_with_refunds_with_claims(d, farm_id, b, false, source_claims)
}
