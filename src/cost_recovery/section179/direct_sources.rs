use crate::{BusinessActivity, FederalDraft, FederalIncome};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};

use super::{
    Section179DirectBusinessIncomeSource, Section179IncomeSystem, Section179PreparedIssue,
    prepare_direct_business,
};

/// One selected Schedule C/F source before section179 and home-office limits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section179DirectIncomeSource {
    pub source: i64,
    pub system: Section179IncomeSystem,
    pub operating_net_cents: i64,
    /// Native recognized property gain, including ordinary and1231 characters.
    pub recognized_property_gain_dollars: i64,
}
impl Section179DirectIncomeSource {
    /// Represent the native annual property amount at the existing allocator's
    /// Cents boundary. Its legacy section1231 field receives the whole gain;
    /// this conversion neither reclassifies it nor creates source-cent precision.
    pub fn prepared_input(
        &self,
    ) -> Result<Section179DirectBusinessIncomeSource, Section179PreparedIssue> {
        let property = self
            .recognized_property_gain_dollars
            .checked_mul(100)
            .ok_or(Section179PreparedIssue::AmountOverflow(self.source))?;
        Ok(Section179DirectBusinessIncomeSource {
            source: self.source,
            system: self.system,
            operating_net_cents: Some(self.operating_net_cents),
            section1231_net_cents: Some(property),
        })
    }
}

// Retain the existing source owner's selected expense recipe independently
// from allocator admission. Read-side unavailability must not change a supplied
// income allocation's policy or replace a missing pool with a local deduction.
struct SelectedOperatingCosts {
    conservation: Result<BTreeMap<i64, i64>, Section179PreparedIssue>,
    supplies: Result<BTreeMap<i64, [i64; 4]>, Section179PreparedIssue>,
    alternative: bool,
}
impl SelectedOperatingCosts {
    fn new(
        draft: &FederalDraft,
        alternative: bool,
        home: Option<&crate::home_office::HomeContext>,
        sources: Option<&BTreeSet<i64>>,
    ) -> Self {
        let farms = || {
            draft.income.iter().filter_map(|entry| {
                if sources.is_some_and(|sources| !sources.contains(&entry.id)) {
                    return None;
                }
                let FederalIncome::Business { data: business } = &entry.income else {
                    return None;
                };
                let BusinessActivity::Farming { data: farm } = &business.activity else {
                    return None;
                };
                Some(farm)
            })
        };
        let conservation = if farms().any(|farm| farm.conservation.is_some()) {
            crate::farm_conservation::selected_expense_adjustments(draft, alternative)
                .map_err(|(id, key)| Section179PreparedIssue::SourceBoundaryUnavailable(id, key))
        } else {
            Ok(BTreeMap::new())
        };
        // Preserve the source getter's conservation-before-supplies priority;
        // no later pool runs after that required source has already failed.
        let supplies = match &conservation {
            Err(issue) => Err(issue.clone()),
            Ok(_) if farms().any(|farm| farm.supplies.is_some()) => {
                crate::farm_supplies::selected_amounts_with_home(draft, alternative, home).map_err(
                    |(id, key)| Section179PreparedIssue::SourceBoundaryUnavailable(id, key),
                )
            }
            Ok(_) => Ok(BTreeMap::new()),
        };
        Self { conservation, supplies, alternative }
    }

    fn validate(&self) -> Result<(), Section179PreparedIssue> {
        self.conservation.as_ref().map_err(Clone::clone)?;
        self.supplies.as_ref().map_err(Clone::clone)?;
        Ok(())
    }

    fn cost(
        &self,
        draft: &FederalDraft,
        id: i64,
        business: &crate::FederalBusiness,
        mid_quarter: bool,
    ) -> Result<i64, Section179PreparedIssue> {
        use Section179PreparedIssue as E;
        let mut cost = crate::cooperative_basis::operating_cost(
            draft,
            id,
            business,
            mid_quarter,
            self.alternative,
        )?;
        if let BusinessActivity::Farming { data: farm } = &business.activity {
            if farm.conservation.is_some() {
                cost = cost
                    .checked_add(
                        *self
                            .conservation
                            .as_ref()
                            .map_err(Clone::clone)?
                            .get(&id)
                            .ok_or(E::SourceIncomeUnavailable(id))?,
                    )
                    .ok_or(E::AmountOverflow(id))?;
            }
            if farm.supplies.is_some() {
                cost = cost
                    .checked_add(
                        self.supplies
                            .as_ref()
                            .map_err(Clone::clone)?
                            .get(&id)
                            .ok_or(E::SourceIncomeUnavailable(id))?[2],
                    )
                    .ok_or(E::AmountOverflow(id))?;
            }
            if self.alternative {
                cost = cost
                    .checked_sub(crate::farm_production::business_amt_adjustment(draft, id))
                    .ok_or(E::AmountOverflow(id))?;
            }
        }
        Ok(cost)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DirectOperatingOperands {
    pub cost_cents: i64,
    pub receipts_cents: i64,
}

impl DirectOperatingOperands {
    pub(super) fn tentative_profit(
        &self,
        source: i64,
        allowed_cents: i64,
    ) -> Result<i64, Section179PreparedIssue> {
        self.cost_cents
            .checked_add(allowed_cents)
            .and_then(|operating| self.receipts_cents.checked_sub(operating))
            .ok_or(Section179PreparedIssue::AmountOverflow(source))
    }
}

pub(super) fn selected_operating_operands(
    draft: &FederalDraft,
    selected: &FederalDraft,
    system: Section179IncomeSystem,
    home: Option<&crate::home_office::HomeContext>,
) -> BTreeMap<i64, Result<DirectOperatingOperands, Section179PreparedIssue>> {
    let costs = SelectedOperatingCosts::new(
        draft,
        system == Section179IncomeSystem::Alternative,
        home,
        None,
    );
    let mid_quarter = crate::depreciation::mid_quarter(selected);
    let selected_businesses: BTreeMap<_, _> = selected
        .income
        .iter()
        .filter_map(|entry| {
            let FederalIncome::Business { data } = &entry.income else {
                return None;
            };
            Some((entry.id, data))
        })
        .collect();
    draft
        .income
        .iter()
        .filter_map(|original| {
            let FederalIncome::Business { data: original_business } = &original.income else {
                return None;
            };
            let id = original.id;
            let operands = (|| {
                let business = selected_businesses
                    .get(&id)
                    .ok_or(Section179PreparedIssue::MissingSource(id))?;
                Ok(DirectOperatingOperands {
                    cost_cents: costs.cost(draft, id, business, mid_quarter)?,
                    // Asset AMT projection changes deductible costs, not the
                    // original receipt owner's recapture or office input stage.
                    receipts_cents: original_business.receipts(),
                })
            })();
            Some((id, operands))
        })
        .collect()
}

/// Selected native Schedule C/F operands before section179 and home-office limits.
/// `draft` must be the same canonical supported/routed working draft used by
/// native collection and the preallocation coordinator. Statutory wages and
/// information-return receipts must already be routed exactly once. This getter
/// neither normalizes nor routes the draft; preparation owns omitted sources
/// and their notices, which this result does not certify as known zero.
/// Source owners certify the supplied view's selected pools before this getter's
/// asset projection. No final return result or narrower capital character supplies
/// an unavailable input. Returned rows retain the draft's source order.
pub fn section179_direct_income_sources(
    draft: &FederalDraft,
    system: Section179IncomeSystem,
) -> Result<Vec<Section179DirectIncomeSource>, Section179PreparedIssue> {
    direct_income_sources_with_home(draft, system, None, None)
}

pub(crate) fn direct_income_sources_with_home(
    draft: &FederalDraft,
    system: Section179IncomeSystem,
    home: Option<&crate::home_office::HomeContext>,
    sources: Option<&BTreeSet<i64>>,
) -> Result<Vec<Section179DirectIncomeSource>, Section179PreparedIssue> {
    use Section179PreparedIssue as E;
    if draft.setup.is_none() {
        return Err(E::MissingReturnContext);
    }
    let associations = crate::source_associations::SourceAssociations::new(draft);
    let alternative = system == Section179IncomeSystem::Alternative;
    let mut selected = draft.clone();
    let mut asset_ids = alloc::collections::BTreeSet::new();
    let mut has_business = false;
    for entry in &mut selected.income {
        if sources.is_some_and(|sources| !sources.contains(&entry.id)) {
            // Shared convention inputs still use the selected system's actual
            // asset view; omitted income is neither certified nor invented.
            if alternative && let FederalIncome::Business { data } = &mut entry.income {
                for asset in &mut data.assets {
                    *asset = asset.amt_projection_or_self();
                }
            }
            continue;
        }
        if !matches!(&entry.income, FederalIncome::Business { .. }) {
            continue;
        }
        let id = entry.id;
        associations.income(id).map_err(|_| E::UnknownSource(id))?;
        let business = prepare_direct_business(entry, alternative, &associations, &mut asset_ids)?;
        // The existing supplied-input allocator remains available. Its gambling
        // stage does not certify this getter's Ordinary/Farming source recipe.
        if matches!(business.activity, BusinessActivity::Gambling { .. }) {
            return Err(E::SourceIncomeUnavailable(id));
        }
        if !has_business && !crate::depreciation::consistent_elections(draft) {
            return Err(E::SourceBoundaryUnavailable(
                id,
                "ISSUE_USE_ONE_CONSISTENT_BONUS_GDS_METHOD_ELECTION_FOR_EACH_RECOVERY_CLAS_5ABD1EF0",
            ));
        }
        has_business = true;
        for asset in &business.assets {
            if asset.is_vehicle()
                && !(asset.vehicle_supported()
                    && matches!(
                        business.activity,
                        BusinessActivity::Ordinary { .. } | BusinessActivity::Farming { .. }
                    ))
            {
                return Err(E::SourceBoundaryUnavailable(
                    id,
                    "ISSUE_CONFIRM_AN_OWNED_5_YEAR_VEHICLE_ON_SCHEDULE_C_WITH_ACTUAL_EXPENSES_93473DC7",
                ));
            }
            if !asset.qualifies() {
                return Err(E::SourceBoundaryUnavailable(
                    id,
                    "ISSUE_CONFIRM_QUALIFYING_PURCHASED_EQUIPMENT_WITH_THE_ENTERED_SERVICE_DAT_EDE7B97E",
                ));
            }
        }
        if let BusinessActivity::Farming { data: farm } = &business.activity
            && let Some(key) = farm.specials.before179_source_issue(matches!(
                farm.accounting,
                crate::FarmAccounting::Cash { .. }
            ))
        {
            return Err(E::SourceBoundaryUnavailable(id, key));
        }
    }
    if !has_business {
        return Ok(Vec::new());
    }
    if let Some((id, key)) =
        crate::federal_rules::direct_business_receipt_source_issue(draft, sources)
    {
        return Err(E::SourceBoundaryUnavailable(id, key));
    }
    // Unrelated farm-rental limits do not make a plain Schedule C source
    // unavailable. When applicable, these owners retain the real whole pool.
    let costs = SelectedOperatingCosts::new(draft, alternative, home, sources);
    costs.validate()?;
    let mut property = BTreeMap::<i64, i64>::new();
    for (id, gain) in
        crate::asset_sale::checked_direct_business_sources(draft, alternative, sources)
            .map_err(|(id, key)| E::SourceBoundaryUnavailable(id, key))?
    {
        let value = property.entry(id).or_default();
        *value = value.checked_add(gain.gain).ok_or(E::AmountOverflow(id))?;
    }
    let mid_quarter = crate::depreciation::mid_quarter(&selected);
    selected
        .income
        .iter()
        .filter_map(|entry| {
            if sources.is_some_and(|sources| !sources.contains(&entry.id)) {
                return None;
            }
            let FederalIncome::Business { data: business } = &entry.income else {
                return None;
            };
            matches!(
                business.activity,
                BusinessActivity::Ordinary { .. } | BusinessActivity::Farming { .. }
            )
            .then_some((entry, business))
        })
        .map(|(entry, business)| {
            let id = entry.id;
            let original = associations.income(id).map_err(|_| E::UnknownSource(id))?;
            let FederalIncome::Business { data: original_business } = &original.income else {
                return Err(E::WrongSourceKind(id));
            };
            // Native AMT replaces deductible asset costs, not the receipt
            // owner's recapture income. Projected histories must not add a
            // second receipt delta absent from the native business adjustment.
            let mut income = crate::federal_rules::farm_receipts_with_refunds(
                draft,
                id,
                original_business,
                alternative,
            )?;
            let cost = costs.cost(draft, id, business, mid_quarter)?;
            if alternative {
                income = income
                    .checked_add(crate::method_change::amt_difference(&business.method_changes))
                    .ok_or(E::AmountOverflow(id))?;
            }
            income = income
                .checked_add(
                    crate::clergy::before179_nondeductible(draft, entry.owner, business)
                        .map_err(|key| E::SourceBoundaryUnavailable(id, key))?,
                )
                .ok_or(E::AmountOverflow(id))?;
            Ok(Section179DirectIncomeSource {
                source: id,
                system,
                operating_net_cents: income.checked_sub(cost).ok_or(E::AmountOverflow(id))?,
                // Absence is zero only after the native C/F collector certified
                // its complete source footprint above.
                recognized_property_gain_dollars: property.get(&id).copied().unwrap_or(0),
            })
        })
        .collect()
}
