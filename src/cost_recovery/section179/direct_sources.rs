use crate::{BusinessActivity, FederalDraft, FederalIncome};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    vec::Vec,
};

use super::{
    Section179DirectBusinessIncomeSource, Section179IncomeSystem, Section179PreparedIssue,
    prepare_direct_business,
};

fn current_conduct_supported(
    draft: &FederalDraft,
    source: i64,
    participation: &crate::direct_participation::DirectParticipationView,
    missing_work: &'static str,
) -> Result<(), Section179PreparedIssue> {
    // Reg. 1.179-2(c)(6), (8) needs this separate taxpayer's work;
    // the source records only combined owner-and-spouse participation.
    if draft
        .setup
        .as_ref()
        .is_some_and(|setup| setup.filing_status() == crate::FederalStatus::Separate)
    {
        return Err(Section179PreparedIssue::SourceBoundaryUnavailable(
            source,
            "ISSUE_SECTION179_SEPARATE_OWN_WORK",
        ));
    }
    if !participation.active_conduct_established(source) {
        return Err(Section179PreparedIssue::SourceBoundaryUnavailable(source, missing_work));
    }
    Ok(())
}

// Reg. 1.179-2(c)(6) requires current meaningful management or operations.
// Section 469's historical and deemed tests alone do not establish that work.
// A materially passive source also needs its native allowed-loss result before
// its taxable income can be certified; operating net is not that result.
pub(super) fn income_source_supported(
    draft: &FederalDraft,
    source: i64,
    business: &crate::FederalBusiness,
    participation: &crate::direct_participation::DirectParticipationView,
) -> Result<(), Section179PreparedIssue> {
    use Section179PreparedIssue as E;
    match business.activity {
        BusinessActivity::Gambling { .. } => {
            gambling_source_supported(draft, source, business, participation)
        }
        BusinessActivity::Ordinary { .. } | BusinessActivity::Farming { .. } => {
            current_conduct_supported(
                draft,
                source,
                participation,
                "ISSUE_SECTION179_DIRECT_CURRENT_WORK",
            )?;
            if participation.get(source) != crate::participation::Participation::Yes {
                return Err(E::SourceBoundaryUnavailable(
                    source,
                    "ISSUE_SECTION179_DIRECT_PAL_INCOME",
                ));
            }
            Ok(())
        }
    }
}

// This source-only cut does not authorize a wagering section 179 deduction
// or any passive-loss treatment. The same check governs preparation and reports.
pub(super) fn gambling_source_supported(
    draft: &FederalDraft,
    source: i64,
    business: &crate::FederalBusiness,
    participation: &crate::direct_participation::DirectParticipationView,
) -> Result<(), Section179PreparedIssue> {
    use Section179PreparedIssue as E;
    current_conduct_supported(
        draft,
        source,
        participation,
        "ISSUE_SECTION179_WAGERING_CURRENT_WORK",
    )?;
    if !matches!(business.activity, BusinessActivity::Gambling { eligible: true, .. }) {
        return Err(E::SourceBoundaryUnavailable(source, "ISSUE_SECTION179_WAGERING_CURRENT_WORK"));
    }
    if business
        .assets
        .iter()
        .any(|asset| asset.available179() != 0 || asset.amt_section179_carry() != Ok(0))
        || crate::home_office::uses_activity(draft, source)
    {
        return Err(E::SourceBoundaryUnavailable(
            source,
            "ISSUE_SECTION179_WAGERING_OWN_DEDUCTION",
        ));
    }
    // The checked property collector intentionally excludes wagering. Certify
    // absence from actual source events, never from a missing collector entry.
    if business.sale.is_some()
        || business.assets.iter().any(|asset| {
            !matches!(asset.disposal, crate::AssetDisposal::No)
                || asset.partial.is_some()
                || asset.earlier_business_percent.is_some()
        })
    {
        return Err(E::SourceBoundaryUnavailable(source, "ISSUE_SECTION179_WAGERING_PROPERTY"));
    }
    if !matches!(business.source, crate::BusinessSource::UnitedStates) {
        return Err(E::SourceBoundaryUnavailable(
            source,
            "ISSUE_RENTAL_FOREIGN_OR_OTHER_PROPERTY_INCOME_WITH_SECTION_179_NEEDS_ACTI_F72A67AD",
        ));
    }
    Ok(())
}

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

// The complete current-property view is transient and bound to the same selected
// system and accepted-source claims as the native operating/179 operands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DirectIncomeParts {
    pub income: Section179DirectIncomeSource,
    pub operating_correction_cents: i64,
    pub property: Result<Vec<crate::asset_sale::DirectBusinessPropertyPart>, &'static str>,
}

// Retain the existing source owner's selected expense recipe independently
// from allocator admission. Read-side unavailability must not change a supplied
// income allocation's policy or replace a missing pool with a local deduction.
struct SelectedOperatingCosts {
    conservation: Result<BTreeMap<i64, i64>, Section179PreparedIssue>,
    supplies: Result<BTreeMap<i64, [i64; 4]>, Section179PreparedIssue>,
    alternative: bool,
    macrs: alloc::sync::Arc<crate::macrs_source::MacrsElectionView>,
}
impl SelectedOperatingCosts {
    fn new_with_claims(
        draft: &FederalDraft,
        alternative: bool,
        home: Option<&crate::home_office::HomeContext>,
        sources: Option<&BTreeSet<i64>>,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
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
            crate::farm_conservation::selected_expense_adjustments_with_claims(
                draft,
                alternative,
                source_claims,
            )
            .map_err(|(id, key)| Section179PreparedIssue::SourceBoundaryUnavailable(id, key))
        } else {
            Ok(BTreeMap::new())
        };
        // Preserve the source getter's conservation-before-supplies priority;
        // no later pool runs after that required source has already failed.
        let supplies = match &conservation {
            Err(issue) => Err(issue.clone()),
            Ok(_) if farms().any(|farm| farm.supplies.is_some()) => {
                crate::farm_supplies::selected_amounts_with_home_with_claims(
                    draft,
                    alternative,
                    home,
                    source_claims,
                )
                .map_err(|(id, key)| Section179PreparedIssue::SourceBoundaryUnavailable(id, key))
            }
            Ok(_) => Ok(BTreeMap::new()),
        };
        Self {
            conservation,
            supplies,
            alternative,
            macrs: home
                .map(|home| home.source_context(draft, source_claims).macrs())
                .unwrap_or_else(|| {
                    alloc::sync::Arc::new(crate::macrs_source::MacrsElectionView::new(draft))
                }),
        }
    }

    fn validate(&self) -> Result<(), Section179PreparedIssue> {
        self.conservation.as_ref().map_err(Clone::clone)?;
        self.supplies.as_ref().map_err(Clone::clone)?;
        Ok(())
    }

    fn cost_with_claims(
        &self,
        draft: &FederalDraft,
        id: i64,
        business: &crate::FederalBusiness,
        mid_quarter: bool,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
    ) -> Result<i64, Section179PreparedIssue> {
        use Section179PreparedIssue as E;
        let mut cost = crate::cooperative_basis::operating_cost_with_claims(
            draft,
            id,
            business,
            mid_quarter,
            self.alternative,
            source_claims,
            Some(self.macrs.as_ref()),
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

pub(super) fn selected_operating_operands_with_claims(
    draft: &FederalDraft,
    selected: &FederalDraft,
    system: Section179IncomeSystem,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> BTreeMap<i64, Result<DirectOperatingOperands, Section179PreparedIssue>> {
    let costs = SelectedOperatingCosts::new_with_claims(
        draft,
        system == Section179IncomeSystem::Alternative,
        home,
        None,
        source_claims,
    );
    let mid_quarter =
        crate::depreciation::mid_quarter_with_macrs(selected, Some(costs.macrs.as_ref()));
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
                    cost_cents: costs.cost_with_claims(
                        draft,
                        id,
                        business,
                        mid_quarter,
                        source_claims,
                    )?,
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
    let source_claims = crate::related_forms::effective_claimants(draft);
    section179_direct_income_sources_with_claims(draft, system, &source_claims)
}

pub fn section179_direct_income_sources_with_claims(
    draft: &FederalDraft,
    system: Section179IncomeSystem,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Vec<Section179DirectIncomeSource>, Section179PreparedIssue> {
    direct_income_sources_with_home_with_claims(draft, system, None, None, source_claims)
}

pub(crate) fn direct_income_sources_with_home_with_claims(
    draft: &FederalDraft,
    system: Section179IncomeSystem,
    home: Option<&crate::home_office::HomeContext>,
    sources: Option<&BTreeSet<i64>>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<Vec<Section179DirectIncomeSource>, Section179PreparedIssue> {
    Ok(crate::section179::direct_income_parts_with_home_with_claims(
        draft,
        system,
        home,
        sources,
        source_claims,
        None,
    )?
    .into_iter()
    .map(|source| {
        let _ = source.property;
        source.income
    })
    .collect())
}

pub(crate) fn direct_income_parts_with_home_with_claims(
    draft: &FederalDraft,
    system: Section179IncomeSystem,
    home: Option<&crate::home_office::HomeContext>,
    sources: Option<&BTreeSet<i64>>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
    actual_selected: Option<&crate::Section179PreparedAllocation>,
) -> Result<Vec<DirectIncomeParts>, Section179PreparedIssue> {
    use Section179PreparedIssue as E;
    if let Some((id, key)) = crate::nonmacrs::financial_issue(draft) {
        return Err(E::SourceBoundaryUnavailable(id, key));
    }
    if draft.setup.is_none() {
        return Err(E::MissingReturnContext);
    }
    let participation = home.map_or_else(
        || draft.direct_participation_view(),
        |home| home.source_context(draft, source_claims).participation(),
    );
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
        if matches!(business.activity, BusinessActivity::Gambling { .. }) {
            gambling_source_supported(draft, id, business, &participation)?;
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
    if let Some((id, key)) = crate::federal_rules::direct_business_receipt_source_issue_with_claims(
        draft,
        sources,
        source_claims,
    ) {
        return Err(E::SourceBoundaryUnavailable(id, key));
    }
    // Unrelated farm-rental limits do not make a plain Schedule C source
    // unavailable. When applicable, these owners retain the real whole pool.
    let costs =
        SelectedOperatingCosts::new_with_claims(draft, alternative, home, sources, source_claims);
    costs.validate()?;
    let mut property = BTreeMap::<i64, i64>::new();
    let (values, mut property_parts) =
        crate::asset_sale::checked_direct_business_property_sources_with_claims(
            draft,
            alternative,
            sources,
            source_claims,
            Some(costs.macrs.as_ref()),
        )
        .map_err(|(id, key)| E::SourceBoundaryUnavailable(id, key))?;
    for (id, gain) in values {
        let value = property.entry(id).or_default();
        *value = value.checked_add(gain.gain).ok_or(E::AmountOverflow(id))?;
    }
    let mid_quarter =
        crate::depreciation::mid_quarter_with_macrs(&selected, Some(costs.macrs.as_ref()));
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
            Some((entry, business))
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
            let mut income = crate::federal_rules::farm_receipts_with_refunds_with_claims(
                draft,
                id,
                original_business,
                alternative,
                source_claims,
            )?;
            let cost = costs.cost_with_claims(draft, id, business, mid_quarter, source_claims)?;
            if alternative {
                income = income
                    .checked_add(crate::method_change::amt_difference(&business.method_changes))
                    .ok_or(E::AmountOverflow(id))?;
            }
            // Only an actual zero-election native allocation supplies this
            // after179 expense endpoint. Before179 retains its source guard.
            let operating_correction_cents = match actual_selected {
                Some(selected) if selected.income.system != system => {
                    return Err(E::SourceIncomeUnavailable(id));
                }
                Some(selected) if selected.available_section179() == 0 => {
                    crate::clergy::nondeductible_with_section179_with_claims(
                        draft,
                        id,
                        Some(selected),
                        home,
                        source_claims,
                    )?
                }
                _ => crate::clergy::before179_nondeductible(draft, entry.owner.owned(), business)
                    .map_err(|key| E::SourceBoundaryUnavailable(id, key))?,
            };
            income = income.checked_add(operating_correction_cents).ok_or(E::AmountOverflow(id))?;
            let wagering = matches!(business.activity, BusinessActivity::Gambling { .. });
            let expense = if wagering { business.expense_from_operating(cost) } else { cost };
            Ok(DirectIncomeParts {
                operating_correction_cents,
                property: if wagering {
                    // The explicit source-event check above established absence.
                    Ok(Vec::new())
                } else {
                    property_parts.remove(&id).ok_or(E::SourceIncomeUnavailable(id))?
                },
                income: Section179DirectIncomeSource {
                    source: id,
                    system,
                    operating_net_cents: income
                        .checked_sub(expense)
                        .ok_or(E::AmountOverflow(id))?,
                    // Wagering absence was certified from actual source events;
                    // the C/F collector certifies the other source footprints.
                    recognized_property_gain_dollars: if wagering {
                        0
                    } else {
                        property.get(&id).copied().unwrap_or(0)
                    },
                },
            })
        })
        .collect()
}
