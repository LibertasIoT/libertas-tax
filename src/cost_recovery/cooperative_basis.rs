// Canonical equipment sources for cooperative purchase refunds. The normal
// association owner resolves identity and parentage before any basis is read.
use crate::kernel::{
    ratio::{
        self,
        exact::{self, Exact},
    },
    source_fact::SourceFact,
};
use crate::{BusinessAsset, FederalDraft, FederalIncome, Owner, RecordChoice};
use alloc::vec::Vec;

// One accepted receipt's use of the shared physical-property pool. Keep the
// original issuer component facts attached to its actual accepted source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AppliedRefund {
    pub parent: i64,
    pub allocation: i64,
    pub record: crate::CoopPropertyRefund,
    pub basis_reduction: [Exact; 2],
    pub taxable_excess: [Exact; 2],
}

pub(crate) struct PurchaseBasis<'a> {
    pub asset: &'a BusinessAsset,
    pub effective_year: u32,
    pub opening_basis: [i64; 2],
    pub refunds: Vec<AppliedRefund>,
}
impl PurchaseBasis<'_> {
    pub(crate) fn reduction(&self, alternative: bool) -> Option<i64> {
        let system = usize::from(alternative);
        let reduction: Exact =
            self.refunds.iter().map(|refund| &refund.basis_reduction[system]).sum();
        // D-2026-10-19-01: compute the complete adjusted balance inside this
        // owner, then project it once to native cents at the schedule boundary.
        // Never round the refund operand before subtracting it.
        let remaining = exact::whole(self.opening_basis[system]) - reduction;
        self.opening_basis[system].checked_sub(exact::rounded(&remaining)?)
    }
    pub(crate) fn new_basis(&self, selected: &BusinessAsset) -> Option<i64> {
        let refunds = ratio::checked_sum(self.refunds.iter().map(|refund| refund.record.amount))?;
        let net_cost = crate::kernel::phase::floor(selected.cost.checked_sub(refunds)?, 0);
        let basis = selected.business_basis_on_cost(net_cost);
        (basis >= 0).then_some(basis)
    }
    pub(crate) fn excess(&self, alternative: bool) -> Exact {
        self.refunds.iter().map(|r| &r.taxable_excess[usize::from(alternative)]).sum()
    }
}

// A contextual view borrows the accepted property. Basis adjustments belong to
// the checked schedule input, never to stored cost, history or credit fields.
pub(crate) struct AssetView<'a> {
    pub asset: &'a BusinessAsset,
    pub purchase: PurchaseBasis<'a>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct AssetAmounts {
    pub bonus: i64,
    pub macrs: i64,
    pub opening: i64,
    pub sale_cost_cents: i64,
    pub personal_basis_cents: Option<i64>,
    pub investment_reduction: i64,
    pub entered_credit_reduction: i64,
    pub partial: Option<crate::asset_sale::PartialOperands>,
    pub qualified179_cost: i64,
}
impl AssetAmounts {
    pub(crate) fn deduction(self) -> Option<i64> {
        self.bonus.checked_add(self.macrs)
    }
}
impl AssetView<'_> {
    pub(crate) fn amounts(&self, mid_quarter: bool, alternative: bool) -> Option<AssetAmounts> {
        use crate::kernel::cost_recovery::gds;
        let selected =
            if alternative { self.asset.amt_projection_or_self() } else { self.asset.clone() };
        let new_basis = if matches!(selected.history, crate::EquipmentHistory::New { .. }) {
            Some(self.purchase.new_basis(&selected)?)
        } else {
            None
        };
        let reduction = match new_basis {
            Some(basis) => selected.business_basis().checked_sub(basis)?,
            None => self.purchase.reduction(alternative)?,
        };
        let gross =
            ratio::checked_sum(self.purchase.refunds.iter().map(|refund| refund.record.amount))?;
        // Personal cost is the actual nonbusiness share, before any business
        // credit reduction. Subtract the exact refund share before projecting
        // the complete remaining personal basis once at the sale boundary.
        let personal_cost =
            exact::whole(selected.cost) - selected.current_share_of(selected.cost)?;
        let personal_refund = exact::whole(gross) - selected.current_share_of(gross)?;
        let remaining = (personal_cost - personal_refund).max(exact::whole(0));
        let personal_basis_cents = Some(exact::rounded(&remaining)?);
        let current_cost = if new_basis.is_some() {
            selected.cost.checked_sub(gross)?.max(0)
        } else {
            selected.cost
        };
        if new_basis.is_some() && !selected.investment_shape_on_cost(current_cost) {
            return None;
        }
        let original_sale_cost = selected.business_share_of(selected.cost);
        let sale_cost_cents = if new_basis.is_some() {
            selected.business_share_of(current_cost)
        } else {
            let historical_reduction: Exact = self
                .purchase
                .refunds
                .iter()
                .map(|refund| &refund.basis_reduction[usize::from(alternative)])
                .sum();
            let current_reduction = if historical_reduction == exact::whole(0) {
                historical_reduction
            } else {
                let historical_use = selected.historical_share_of(100)?;
                if historical_use == exact::whole(0) {
                    return None;
                }
                historical_reduction * selected.current_share_of(100)? / historical_use
            };
            exact::rounded(
                &(exact::whole(original_sale_cost) - current_reduction).max(exact::whole(0)),
            )?
        };
        let investment_reduction = selected.investment_basis_reduction_on(current_cost);
        let entered_credit_reduction = if selected.investment_credit.is_some() {
            0
        } else {
            selected.business_share_of(current_cost) - selected.business_basis_on_cost(current_cost)
        };
        let basis_event = self
            .purchase
            .refunds
            .iter()
            .any(|refund| refund.basis_reduction[usize::from(alternative)] != exact::whole(0));
        // A real later-year event selects the continuation schedule even if
        // the complete adjusted balance rounds back to its old cent value.
        if reduction == 0 && (new_basis.is_some() || !basis_event) {
            return Some(AssetAmounts {
                bonus: selected.bonus_amount(),
                macrs: selected.macrs_amount(mid_quarter),
                opening: selected.opening_basis(),
                sale_cost_cents,
                personal_basis_cents,
                investment_reduction,
                entered_credit_reduction,
                partial: None,
                qualified179_cost: selected.qualified179_cost(),
            });
        }
        if selected.class == crate::EquipmentClass::ThirtyNine {
            return Some(AssetAmounts {
                bonus: 0,
                macrs: selected
                    .purchase_building_amount(reduction, self.purchase.effective_year)?,
                opening: selected.opening_basis().checked_sub(reduction)?,
                sale_cost_cents,
                personal_basis_cents,
                investment_reduction,
                entered_credit_reduction,
                partial: if let Some(part) = &selected.partial {
                    let original = exact::whole(selected.business_share_of(part.cost_of_part));
                    let exact_reduction: Exact = self
                        .purchase
                        .refunds
                        .iter()
                        .map(|refund| &refund.basis_reduction[usize::from(alternative)])
                        .sum();
                    let refund = exact_reduction * exact::ratio(part.cost_of_part, selected.cost)?;
                    Some(crate::asset_sale::PartialOperands {
                        full_year: selected
                            .purchase_building_full_year(reduction, self.purchase.effective_year)?,
                        cost: exact::rounded(&(original - refund).max(exact::whole(0)))?,
                        original_part_cost: part.cost_of_part,
                        original_total_cost: selected.cost,
                    })
                } else {
                    None
                },
                qualified179_cost: 0,
            });
        }
        if let Some(macrs) = crate::auto_recovery::purchase_post_recovery_amount(
            &selected,
            self.purchase.effective_year,
            reduction,
        ) {
            return Some(AssetAmounts {
                bonus: 0,
                macrs,
                opening: selected.opening_basis().checked_sub(reduction)?,
                sale_cost_cents,
                personal_basis_cents,
                investment_reduction,
                entered_credit_reduction,
                partial: None,
                qualified179_cost: 0,
            });
        }
        let mut schedule =
            selected.checked_gds_schedule(mid_quarter, self.purchase.effective_year)?;
        let opening;
        let qualified179_cost;
        match schedule.history {
            gds::History::New { elected179_cents } => {
                // Refunds received in the service year reduce acquisition cost
                // before the election and bonus stages. Preserve an excessive
                // entered election as unavailable; never change that election.
                schedule.history_basis_cents = new_basis?;
                opening = schedule.history_basis_cents.checked_sub(elected179_cents)?;
                if opening < 0 {
                    return None;
                }
                schedule.raw_bonus_cents = selected.raw_bonus_on(opening);
                qualified179_cost =
                    if selected.qualified179_cost() > 0 { schedule.history_basis_cents } else { 0 };
            }
            gds::History::Continuing {
                prior_depreciation_cents, carryforward179_cents, ..
            } => {
                let adjusted = schedule
                    .history_basis_cents
                    .checked_sub(prior_depreciation_cents)?
                    .checked_sub(reduction)?;
                opening = adjusted.checked_sub(carryforward179_cents)?;
                if opening < 0 {
                    return None;
                }
                schedule.continuation = Some(gds::AdjustedBasisContinuation {
                    effective_year: self.purchase.effective_year,
                    adjusted_basis_cents: adjusted,
                    prior_depreciation_cents,
                    declining_rate: schedule
                        .continuation_declining_rate(self.purchase.effective_year),
                });
                qualified179_cost = 0;
            }
        }
        let SourceFact::Reported(amounts) = gds::calculate(SourceFact::Reported(schedule)) else {
            return None;
        };
        let room =
            if crate::auto_recovery::amount(&selected, self.purchase.effective_year).is_some() {
                crate::auto_recovery::original_cap(&selected, self.purchase.effective_year)?
            } else {
                selected.limit_room()
            };
        let bonus = crate::kernel::phase::cap(schedule.raw_bonus_cents, [room]);
        let macrs = crate::kernel::phase::cap(amounts.current_macrs_cents, [room - bonus]);
        Some(AssetAmounts {
            bonus,
            macrs,
            opening,
            sale_cost_cents,
            personal_basis_cents,
            investment_reduction,
            entered_credit_reduction,
            partial: None,
            qualified179_cost,
        })
    }
}

// Opening capacity belongs to the native asset owner, separately for regular
// and AMT. Current-year acquisition capacity precedes the current election,
// bonus and MACRS stages. Continuing capacity excludes actual prior recovery and the native179 reserve.
fn opening(asset: &BusinessAsset) -> Option<i64> {
    let basis = match asset.history {
        crate::EquipmentHistory::New { .. } => asset.history_basis(),
        // Native opening basis excludes the unused election reserved for179;
        // that separate carry remains owned by the existing annual allocator.
        crate::EquipmentHistory::Continuing { .. } => asset.opening_basis(),
    };
    (basis >= 0).then_some(basis)
}

pub(crate) fn purchases(
    draft: &FederalDraft,
    owner: Owner,
    activity: i64,
    property: i64,
) -> Result<SourceFact<PurchaseBasis<'_>>, crate::Section179PreparedIssue> {
    let mut refunds = Vec::new();
    for source in crate::cooperative::sources(draft) {
        if source.owner != owner || source.allocation.activity != Some(activity) {
            continue;
        }
        for record in source.allocation.properties {
            if record.property != Some(property) || record.amount == 0 {
                continue;
            }
            if record.distribution != crate::CoopDistributionBox::Patronage
                || record.treatment != crate::CoopPropertyTreatment::Purchase
            {
                return Ok(SourceFact::Unavailable);
            }
            refunds.push(AppliedRefund {
                parent: source.parent,
                allocation: source.allocation.id,
                record,
                basis_reduction: [exact::whole(0), exact::whole(0)],
                taxable_excess: [exact::whole(0), exact::whole(0)],
            });
        }
    }
    if refunds.is_empty() {
        return Ok(SourceFact::NotReported);
    }
    let Some(asset) = asset(draft, owner, Some(activity), property) else {
        return Ok(SourceFact::Unavailable);
    };
    let alternative = asset.amt_projection_or_self();
    let (Some(regular_opening), Some(alternative_opening)) =
        (opening(asset), opening(&alternative))
    else {
        return Ok(SourceFact::Unavailable);
    };
    let opening_basis = [regular_opening, alternative_opening];
    let total = ratio::checked_sum(refunds.iter().map(|refund| refund.record.amount))
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?;
    let Some(income_total) = asset.historical_share_of(total) else {
        return Ok(SourceFact::Unavailable);
    };
    if income_total < exact::whole(0) || total <= 0 {
        return Ok(SourceFact::Unavailable);
    }
    // These receipts adjust one physical asset at the same effective date.
    // Their actual source-cent weights preserve issuer qualifications and
    // accepted identities without assigning a rounding residue to one payer.
    let reductions = opening_basis.map(|opening| income_total.clone().min(exact::whole(opening)));
    for refund in &mut refunds {
        let Some(weight) = exact::ratio(refund.record.amount, total) else {
            return Ok(SourceFact::Unavailable);
        };
        for (system, reduction) in reductions.iter().enumerate() {
            refund.basis_reduction[system] = reduction * &weight;
            refund.taxable_excess[system] = (&income_total - reduction) * &weight;
        }
    }
    let Ok(effective_year) = u32::try_from(draft.tax_year) else {
        return Ok(SourceFact::Unavailable);
    };
    Ok(SourceFact::Reported(PurchaseBasis { asset, effective_year, opening_basis, refunds }))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RefundIncome {
    pub ordinary: i64,
    // Actual issuer qualified-payment, non-SSTB and SSTB portions of excess.
    pub qualification: [i64; 3],
    pub related_sstb_expenses: i64,
}
pub(crate) fn income(
    draft: &FederalDraft,
    owner: Owner,
    activity: i64,
    alternative: bool,
) -> Result<SourceFact<RefundIncome>, crate::Section179PreparedIssue> {
    let mut properties = alloc::collections::BTreeSet::new();
    for source in crate::cooperative::sources(draft) {
        if source.owner == owner && source.allocation.activity == Some(activity) {
            for refund in source.allocation.properties {
                if refund.amount > 0
                    && refund.treatment == crate::CoopPropertyTreatment::Purchase
                    && let Some(property) = refund.property
                    && asset(draft, owner, Some(activity), property).is_some()
                {
                    properties.insert(property);
                }
            }
        }
    }
    if properties.is_empty() {
        return Ok(SourceFact::NotReported);
    }
    let mut ordinary = exact::whole(0);
    let mut qualification = [exact::whole(0), exact::whole(0), exact::whole(0)];
    let mut source_sstb = alloc::collections::BTreeMap::<(i64, i64), Exact>::new();
    for property in properties {
        let SourceFact::Reported(purchase) = purchases(draft, owner, activity, property)? else {
            return Ok(SourceFact::Unavailable);
        };
        ordinary += purchase.excess(alternative);
        for refund in purchase.refunds {
            let excess = &refund.taxable_excess[usize::from(alternative)];
            for (index, qualified) in [
                refund.record.qualified_payments,
                refund.record.qualified_items,
                refund.record.sstb_items,
            ]
            .into_iter()
            .enumerate()
            {
                let Some(weight) = exact::ratio(qualified, refund.record.amount) else {
                    return Ok(SourceFact::Unavailable);
                };
                let amount = excess * weight;
                if index == 2 {
                    *source_sstb
                        .entry((refund.parent, refund.allocation))
                        .or_insert_with(|| exact::whole(0)) += &amount;
                }
                qualification[index] += amount;
            }
        }
    }
    // Expenses retain their actual issuer/allocation association. Only the
    // newly taxable SSTB portion can add to the existing related-expense cap.
    let sources = crate::cooperative::sources(draft);
    let mut related_expenses = exact::whole(0);
    for ((parent, allocation), excess) in source_sstb {
        let Some(source) = sources
            .iter()
            .find(|source| source.parent == parent && source.allocation.id == allocation)
        else {
            return Ok(SourceFact::Unavailable);
        };
        let Some(qualified) = source.allocation.retained_qualification() else {
            return Ok(SourceFact::Unavailable);
        };
        let expenses = ratio::checked_sum(
            source.allocation.sstb_expenses.iter().map(|expense| expense.amount),
        )
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?;
        let prior = exact::whole(qualified[2]);
        let limit = exact::whole(expenses);
        related_expenses += (&prior + excess).min(limit.clone()) - prior.min(limit);
    }
    let related_sstb_expenses = exact::rounded(&related_expenses)
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?;
    // Box7 overlaps the other classes; box8, box9 and the nonqualified
    // remainder are disjoint. Preserve that source footing before projection.
    let disjoint = &qualification[1] + &qualification[2];
    if disjoint > ordinary || qualification.iter().any(|amount| amount < &exact::whole(0)) {
        return Ok(SourceFact::Unavailable);
    }
    let payments = exact::rounded(&qualification[0])
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?;
    let nonqualified = &ordinary - &disjoint;
    let ordinary = exact::rounded(&ordinary)
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?;
    // Money precision: splitting one rounded source declares the first class.
    // Existing form order is box8, box9, then the nonqualified remainder.
    let mut prefix = exact::ExactPrefix::default();
    let mut cumulative = exact::whole(0);
    let mut classes = [0; 3];
    for (target, amount) in
        classes.iter_mut().zip([&qualification[1], &qualification[2], &nonqualified])
    {
        cumulative += amount;
        let projected = exact::rounded(&cumulative)
            .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?;
        *target = prefix.next(amount, |_| projected);
    }
    Ok(SourceFact::Reported(RefundIncome {
        ordinary,
        qualification: [payments, classes[0], classes[1]],
        related_sstb_expenses,
    }))
}

pub(crate) fn current_cost(
    draft: &FederalDraft,
    activity: i64,
    asset: &BusinessAsset,
) -> Result<i64, crate::Section179PreparedIssue> {
    if !matches!(asset.history, crate::EquipmentHistory::New { .. }) {
        return Ok(asset.cost);
    }
    let entry = draft
        .income
        .iter()
        .find(|entry| entry.id == activity)
        .ok_or(crate::Section179PreparedIssue::UnknownSource(activity))?;
    match purchases(draft, entry.owner, activity, asset.id)? {
        SourceFact::NotReported => Ok(asset.cost),
        SourceFact::Reported(purchase) => {
            let total =
                ratio::checked_sum(purchase.refunds.iter().map(|refund| refund.record.amount))
                    .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?;
            Ok(asset
                .cost
                .checked_sub(total)
                .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?
                .max(0))
        }
        SourceFact::Unavailable => Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
            activity,
            "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
        )),
    }
}

pub(crate) fn qualified179_cost(
    draft: &FederalDraft,
    activity: i64,
    asset: &BusinessAsset,
) -> Result<i64, crate::Section179PreparedIssue> {
    let cost = asset.qualified179_cost();
    if cost == 0 {
        return Ok(0);
    }
    let Some(entry) = draft.income.iter().find(|entry| entry.id == activity) else {
        return Err(crate::Section179PreparedIssue::UnknownSource(activity));
    };
    match purchases(draft, entry.owner, activity, asset.id)? {
        SourceFact::NotReported => Ok(cost),
        SourceFact::Reported(purchase) => {
            let net = purchase.new_basis(purchase.asset);
            net.filter(|net| *net >= purchase.asset.elected179()).ok_or(
                crate::Section179PreparedIssue::SourceBoundaryUnavailable(
                    activity,
                    "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
                ),
            )
        }
        SourceFact::Unavailable => Err(crate::Section179PreparedIssue::SourceBoundaryUnavailable(
            activity,
            "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
        )),
    }
}

pub(crate) fn source_amounts(
    draft: &FederalDraft,
    activity: i64,
    asset: &BusinessAsset,
    mid_quarter: bool,
    alternative: bool,
) -> Result<AssetAmounts, crate::Section179PreparedIssue> {
    use crate::Section179PreparedIssue as E;
    let Some(entry) = draft.income.iter().find(|entry| entry.id == activity) else {
        return Err(E::UnknownSource(activity));
    };
    match purchases(draft, entry.owner, activity, asset.id)? {
        SourceFact::NotReported => {
            let selected = if alternative { asset.amt_projection_or_self() } else { asset.clone() };
            Ok(AssetAmounts {
                bonus: selected.bonus_amount(),
                macrs: selected.macrs_amount(mid_quarter),
                opening: selected.opening_basis(),
                sale_cost_cents: selected.business_basis()
                    + selected.investment_basis_reduction()
                    + selected.entered_credit_reduction(),
                personal_basis_cents: None,
                investment_reduction: selected.investment_basis_reduction(),
                entered_credit_reduction: selected.entered_credit_reduction(),
                partial: None,
                qualified179_cost: selected.qualified179_cost(),
            })
        }
        SourceFact::Reported(purchase) => AssetView { asset, purchase }
            .amounts(mid_quarter, alternative)
            .ok_or(E::SourceBoundaryUnavailable(activity, "ISSUE_COOP_PROPERTY_HISTORY_NEEDED")),
        SourceFact::Unavailable => {
            Err(E::SourceBoundaryUnavailable(activity, "ISSUE_COOP_PROPERTY_HISTORY_NEEDED"))
        }
    }
}

// Property consumers explicitly use the native property projection. C/F
// sources remain whole until Totals settlement; Form4835 already owns a
// divided property view. Never infer a fraction from rounded copied costs.
fn property_operands(
    draft: &FederalDraft,
    activity: i64,
    asset: &BusinessAsset,
    mid_quarter: bool,
    alternative: bool,
) -> Result<(BusinessAsset, AssetAmounts), crate::Section179PreparedIssue> {
    use crate::Section179PreparedIssue as E;
    let entry =
        draft.income.iter().find(|entry| entry.id == activity).ok_or(E::UnknownSource(activity))?;
    let purchase = match purchases(draft, entry.owner, activity, asset.id)? {
        SourceFact::NotReported => {
            return Ok((
                asset.clone(),
                source_amounts(draft, activity, asset, mid_quarter, alternative)?,
            ));
        }
        SourceFact::Unavailable => {
            return Err(E::SourceBoundaryUnavailable(
                activity,
                "ISSUE_COOP_PROPERTY_HISTORY_NEEDED",
            ));
        }
        SourceFact::Reported(purchase) => purchase,
    };
    let canonical = purchase.asset;
    let mut amounts = AssetView { asset: canonical, purchase }
        .amounts(mid_quarter, alternative)
        .ok_or(E::SourceBoundaryUnavailable(activity, "ISSUE_COOP_PROPERTY_HISTORY_NEEDED"))?;
    if !matches!(entry.income, FederalIncome::FarmRental { .. }) {
        return Ok((canonical.clone(), amounts));
    }
    let property = crate::community::property_entry(draft, entry);
    let FederalIncome::FarmRental { data } = &property.income else {
        return Err(E::WrongSourceKind(activity));
    };
    let selected = data
        .assets
        .iter()
        .find(|selected| selected.id == asset.id)
        .ok_or(E::UnknownSource(activity))?;
    let project = |amount| crate::community::property_amount(draft, activity, amount);
    amounts.bonus = project(amounts.bonus);
    amounts.macrs = project(amounts.macrs);
    amounts.opening = project(amounts.opening);
    amounts.sale_cost_cents = project(amounts.sale_cost_cents);
    amounts.personal_basis_cents = amounts.personal_basis_cents.map(project);
    amounts.investment_reduction = project(amounts.investment_reduction);
    amounts.entered_credit_reduction = project(amounts.entered_credit_reduction);
    amounts.qualified179_cost = project(amounts.qualified179_cost);
    if let Some(partial) = &mut amounts.partial {
        partial.cost = project(partial.cost);
        partial.full_year = project(partial.full_year);
    }
    Ok((selected.clone(), amounts))
}

pub(crate) fn sale(
    draft: &FederalDraft,
    activity: i64,
    asset: &BusinessAsset,
    mid_quarter: bool,
    alternative: bool,
) -> Result<Option<crate::asset_sale::AssetGain>, crate::Section179PreparedIssue> {
    let (asset, amounts) = property_operands(draft, activity, asset, mid_quarter, alternative)?;
    Ok(asset.sale_with_purchase(mid_quarter, alternative, Some(amounts)))
}

pub(crate) fn partial(
    draft: &FederalDraft,
    activity: i64,
    asset: &BusinessAsset,
    mid_quarter: bool,
    alternative: bool,
) -> Result<Option<crate::asset_sale::PartialGain>, crate::Section179PreparedIssue> {
    let (asset, amounts) = property_operands(draft, activity, asset, mid_quarter, alternative)?;
    Ok(asset.partial_with_purchase(amounts.partial))
}

pub(crate) fn depreciation(
    draft: &FederalDraft,
    activity: i64,
    assets: &[BusinessAsset],
    mid_quarter: bool,
    alternative: bool,
) -> Result<i64, crate::Section179PreparedIssue> {
    let mut total = 0_i64;
    for asset in assets {
        let amount = source_amounts(draft, activity, asset, mid_quarter, alternative)?
            .deduction()
            .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?;
        total = total
            .checked_add(amount)
            .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))?;
    }
    Ok(total)
}

pub(crate) fn operating_cost(
    draft: &FederalDraft,
    activity: i64,
    business: &crate::FederalBusiness,
    mid_quarter: bool,
    alternative: bool,
) -> Result<i64, crate::Section179PreparedIssue> {
    let has_refund = crate::cooperative::sources(draft).iter().any(|source| {
        source.allocation.activity == Some(activity)
            && source.allocation.properties.iter().any(|record| record.amount > 0)
    });
    if !has_refund {
        return Ok(business.operating_cost_before_supplies(mid_quarter));
    }
    let depreciation = depreciation(draft, activity, &business.assets, mid_quarter, alternative)?;
    business
        .operating_cost_with_asset_deductions(depreciation)
        .ok_or(crate::Section179PreparedIssue::AmountOverflow(activity))
}

pub(crate) fn asset(
    draft: &FederalDraft,
    owner: Owner,
    activity: Option<i64>,
    property: i64,
) -> Option<&BusinessAsset> {
    let activity = activity?;
    if !crate::cooperative::eligible_activities(draft, owner).contains(&activity) {
        return None;
    }
    // A corrupt retained cross-kind collision must not select equipment by
    // iteration order. The cooperative graph reports the existing link issue.
    if crate::farm::sources(draft).any(|source| {
        source.entry.id == activity && source.farm.land_basis.iter().any(|land| land.id == property)
    }) {
        return None;
    }
    crate::source_associations::SourceAssociations::new(draft).asset(activity, property).ok()
}

pub(crate) fn choices(draft: &FederalDraft) -> Vec<RecordChoice> {
    let mut choices = Vec::new();
    for entry in &draft.income {
        let assets = match &entry.income {
            FederalIncome::Business { data } => data.assets.as_slice(),
            FederalIncome::FarmRental { data } => data.assets.as_slice(),
            _ => continue,
        };
        for (index, value) in assets.iter().enumerate() {
            if asset(draft, entry.owner, Some(entry.id), value.id).is_some() {
                choices.push(RecordChoice {
                    id: value.id,
                    label: crate::Message::new(
                        "FARM_PARCEL_CHOICE",
                        alloc::vec![
                            crate::MessageArg::Text(entry.label.clone()),
                            BusinessAsset::display_arg(assets, index, 0),
                        ],
                    )
                    .encode(),
                });
            }
        }
    }
    choices
}
