use crate::FederalResult;
use crate::farm_land_model::{
    FarmLandAcquisition, FarmLandAllocation, FarmLandAmt, FarmLandSale, FarmLandSituation,
};
pub(crate) use crate::farm_land_model::{LandGain, ResolvedFarmLandSale};
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::ratio::{fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::{FormLine, TaxForm};
use alloc::format;

impl FarmLandAllocation {
    fn sources<'a>(
        &'a self,
        parcel: &crate::FarmLandBasis,
    ) -> [(&'static str, &'a crate::SourceCents, i64); 4] {
        [
            ("regular", &self.regular, parcel.regular),
            ("amt", &self.amt, parcel.amt),
            ("conservation", &self.conservation, parcel.prior_conservation),
            ("amt_conservation", &self.amt_conservation, parcel.prior_amt_conservation),
        ]
    }
    pub(crate) fn issue(
        &self,
        parcel: &crate::FarmLandBasis,
    ) -> Option<crate::validation::ValidationIssue> {
        for (field, source, bound) in self.sources(parcel) {
            if source
                .source_fact()
                .and_then(|fact| {
                    fact.check_reported(|amount| {
                        if *amount <= bound { Ok(()) } else { Err("FARM_LAND_PART_ALLOCATION") }
                    })
                })
                .is_err()
            {
                return Some(
                    crate::validation::ValidationIssue::new("FARM_LAND_PART_ALLOCATION")
                        .at_field("amount")
                        .in_field(field)
                        .in_field("allocation"),
                );
            }
        }
        None
    }
    fn amounts(&self, parcel: &crate::FarmLandBasis) -> Option<[i64; 4]> {
        let mut amounts = [0; 4];
        for (index, (_, source, bound)) in self.sources(parcel).into_iter().enumerate() {
            amounts[index] = source
                .source_fact()
                .ok()?
                .check_reported(|amount| if *amount <= bound { Ok(()) } else { Err(()) })
                .ok()?
                .require(|| ())
                .ok()?
                .unwrap_or(0);
        }
        Some(amounts)
    }
}
// Validate physical opening-source capacities before any sale is accepted.
// i128 keeps a long list of otherwise valid cent amounts from wrapping.
pub(crate) fn partial_capacity_issue(
    farm: &crate::FarmIncome,
    parcel: &crate::FarmLandBasis,
    index: usize,
) -> Option<crate::validation::ValidationIssue> {
    let sale = &farm.land_sales[index];
    let FarmLandSituation::PartialParcel { allocation } = &sale.situation else {
        return None;
    };
    for (field, _, bound) in allocation.sources(parcel) {
        let mut total = 0i128;
        for sale in
            farm.land_sales.iter().take(index + 1).filter(|sale| sale.basis_property == parcel.id)
        {
            if let FarmLandSituation::PartialParcel { allocation } = &sale.situation {
                let (_, source, _) =
                    allocation.sources(parcel).into_iter().find(|(name, _, _)| *name == field)?;
                // Known shares give a lower bound for validation. An unknown
                // share still makes the calculated retained balance unavailable.
                let amount = match source.source_fact().ok()? {
                    crate::kernel::source_fact::SourceFact::Reported(amount) => amount,
                    crate::kernel::source_fact::SourceFact::NotReported
                    | crate::kernel::source_fact::SourceFact::Unavailable => 0,
                };
                total += i128::from(amount);
            }
        }
        if total > i128::from(bound) {
            return Some(
                crate::validation::ValidationIssue::new("FARM_LAND_PART_ALLOCATION")
                    .at_field("amount")
                    .in_field(field)
                    .in_field("allocation")
                    .in_field("situation"),
            );
        }
    }
    None
}

impl ResolvedFarmLandSale {
    pub(crate) fn report(
        &self,
        id: i64,
        index: usize,
        form: crate::farm::FarmForm,
        r: &mut FederalResult,
    ) {
        let input = form.input("land_sales");
        if self.deferral {
            crate::farm_deferral::report(self, r);
        }
        if self.payment.installment() {
            let short = self.acquisition == FarmLandAcquisition::Purchased
                && !crate::dates::held_more_than_one_year(self.acquired, self.sold);
            if short {
                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::Farming),
                    "Farm land sale",
                    "ISSUE_FARM_LAND_INSTALLMENT_SHORT_HOLDING_PERIOD",
                    "An installment sale of farm land held one year or less is not calculated (Form 6252 line 26 to Form 4797 line 11).",
                );
            } else if self.deferral {
                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::Farming),
                    "Farm land sale",
                    "ISSUE_FARM_LAND_INSTALLMENT_WITH_DEFERRAL",
                    "An installment sale of farm land with the Form 1062 tax-payment election is not calculated (Form 6252 and Form 1062 line 15).",
                );
            } else if let Some((key, _message)) = self.payment.refusal(self.proceeds) {
                #[cfg(test)]
                let message = _message;

                crate::issue_localized!(
                    r,
                    &crate::IssueContext::Section(crate::TaxSection::Farming),
                    "Farm land sale",
                    key,
                    message,
                );
            }
        }
        if let Some(form) = self.installment_for(false) {
            crate::installment_return::report_sale(
                r,
                &format!("farm_{id}_land_{index}_form6252"),
                &input,
                &form,
                0,
                true,
            );
        }
        let v = self.calculate();
        if self.installment_for(false).is_some() {
            let (pool1252, pool1255) = self.ordinary_pools_for(false);
            emit_lines(
                r,
                &mut (),
                [
                    (
                        "conservation_recapture_remaining",
                        positive(pool1252 - v.recapture),
                        "EXPLAIN_FARM_LAND_INSTALLMENT_RECAPTURE_HISTORY",
                    ),
                    (
                        "cost_sharing_recapture_remaining",
                        positive(pool1255 - positive(v.ordinary - v.recapture)),
                        "EXPLAIN_FARM_LAND_INSTALLMENT_RECAPTURE_HISTORY",
                    ),
                ],
                |(name, amount, key), _, _| {
                    Some(calculation_row(
                        None,
                        &format!("farm_{id}_land_{index}_{name}"),
                        LineValue::MoneyDollars(amount),
                        &[&input],
                        "https://www.irs.gov/instructions/i6252",
                        crate::Message::new(key, alloc::vec![]),
                    ))
                },
            );
        }
        // Form 4797: the section 1231 part is line 2, or line 4 once Form 6252
        // line 26 reports it; a parcel held one year or less is ordinary gain
        // or loss on line 11.
        let short = self.acquisition == FarmLandAcquisition::Purchased
            && !crate::dates::held_more_than_one_year(self.acquired, self.sold);
        emit_lines(
            r,
            &mut (),
            [
                ("gain", v.gain, msg!("EXPLAIN_FARM_N_LAND_N_N_GAIN"), None),
                (
                    "conservation_recapture",
                    v.recapture,
                    msg!("EXPLAIN_FARM_N_LAND_N_N_CONSERVATION_RECAPTURE"),
                    None,
                ),
                (
                    "cost_sharing_recapture",
                    if self.acquisition == FarmLandAcquisition::Inherited
                        || crate::dates::held_more_than_one_year(self.acquired, self.sold)
                    {
                        positive(v.ordinary - v.recapture)
                    } else {
                        0
                    },
                    msg!("EXPLAIN_FARM_LAND_1255_RECAPTURE"),
                    None,
                ),
                (
                    "ordinary",
                    v.ordinary,
                    msg!("EXPLAIN_FARM_N_LAND_N_N_ORDINARY"),
                    short.then_some("11"),
                ),
                (
                    "section1231",
                    v.section1231,
                    msg!("EXPLAIN_FARM_N_LAND_N_N_SECTION1231"),
                    (!short).then_some(if self.installment_for(false).is_some() {
                        "4"
                    } else {
                        "2"
                    }),
                ),
            ],
            |(name, amount, explanation, form_line), _, _| {
                Some(calculation_row(
                    form_line.map(|line| FormLine::new(TaxForm::Form4797, line)),
                    &format!("farm_{id}_land_{index}_{name}"),
                    LineValue::MoneyCents(amount * 100),
                    &[&input],
                    "https://www.irs.gov/instructions/i4797",
                    explanation,
                ))
            },
        );
    }
}

// Aggregate parcels by their real business, never by a synthetic K-1 source.
#[cfg(test)]
pub(crate) fn sources(draft: &crate::FederalDraft, amt: bool) -> alloc::vec::Vec<(i64, LandGain)> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    sources_with_claims(draft, amt, &source_claims)
}

#[cfg(test)]
pub(crate) fn sources_with_claims(
    draft: &crate::FederalDraft,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> alloc::vec::Vec<(i64, LandGain)> {
    source_scopes_with_claims(draft, amt, source_claims)
        .into_iter()
        .filter_map(|(id, value, _)| value.map(|value| (id, value)))
        .collect()
}

// Keep each source's legacy totals and missing-source status together; an
// unrelated rental must not hide the status of a direct Schedule C/F source.

pub(crate) fn source_scopes_with_claims(
    draft: &crate::FederalDraft,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> alloc::vec::Vec<(i64, Option<LandGain>, Option<&'static str>)> {
    // A farm rental's land sales are reported under its own identity.
    crate::community::property_entries(draft)
        .into_iter()
        .filter_map(|entry| {
            let farm = match &entry.income {
                crate::FederalIncome::Business { data } => match &data.activity {
                    crate::BusinessActivity::Farming { data } => data,
                    _ => return None,
                },
                crate::FederalIncome::FarmRental { data } => &data.farm,
                _ => return None,
            };
            let (value, issue) =
                source_view_with_claims(draft, entry.id, farm, amt, true, source_claims);
            Some((entry.id, value, issue))
        })
        .collect()
}

// The same actual-parcel owner serves the return's property view and Form8958's
// whole-source column. Conservation and cooperative basis adjustments stay here.

pub(crate) fn source_with_claims(
    draft: &crate::FederalDraft,
    id: i64,
    farm: &crate::FarmIncome,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<LandGain> {
    source_view_with_claims(draft, id, farm, amt, false, source_claims).0
}

fn source_view_with_claims(
    draft: &crate::FederalDraft,
    id: i64,
    farm: &crate::FarmIncome,
    amt: bool,
    allocated: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> (Option<LandGain>, Option<&'static str>) {
    let (components, issue) =
        source_components_with_claims(draft, id, farm, amt, allocated, source_claims);
    if farm.land_sales.is_empty() {
        return (None, issue);
    }
    let mut total = LandGain::zero();
    for (_, value) in components {
        total.gain += value.gain;
        total.recapture += value.recapture;
        total.ordinary += value.ordinary;
        total.section1231 += value.section1231;
    }
    (Some(total), issue)
}

// Preserve each actual recognized parcel component for the same source's
// loss-row reporting. Resolution and tax character stay in this owner.

pub(crate) fn source_components_with_claims(
    draft: &crate::FederalDraft,
    id: i64,
    farm: &crate::FarmIncome,
    amt: bool,
    allocated: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> (alloc::vec::Vec<(usize, LandGain)>, Option<&'static str>) {
    let mut components = alloc::vec::Vec::new();
    let mut issue = None;
    for (index, sale) in farm.land_sales.iter().enumerate() {
        let Some(sale) = sale.resolve_view_with_claims(draft, id, farm, allocated, source_claims)
        else {
            issue.get_or_insert(sale.source_key());
            continue;
        };
        components.push((index, sale.calculate_for(amt)));
    }
    (components, issue)
}

#[cfg(test)]
pub(crate) fn qbi(
    draft: &crate::FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: Option<&crate::section1231::Result1231>,
    id: i64,
) -> Result<i64, crate::section1231::SourceError> {
    let source_claims = crate::related_forms::effective_claimants(draft);
    qbi_cents_with_claims(draft, pool, character, id, &source_claims)
        .map(crate::kernel::ratio::dollars)
}

pub(crate) fn qbi_cents_with_claims(
    draft: &crate::FederalDraft,
    pool: &crate::extended_rules::RentalResult,
    character: Option<&crate::section1231::Result1231>,
    id: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::section1231::SourceError> {
    let Some((_, own)) =
        crate::asset_sale::allowed_direct_sources_with_claims(draft, pool, false, source_claims)?
            .into_iter()
            .find(|(source, _)| *source == id)
    else {
        return Ok(0);
    };
    if own.section1231 == 0 {
        return Ok(own.ordinary);
    }
    let character = character.ok_or((
        "ISSUE_FARM_LAND_RESOLVE_BUSINESS_PROPERTY_SECTION_1231_CHARACTER_BEFORE_7F94E201",
        crate::fixture_english!(
            "Resolve business-property section 1231 character before business QBI."
        ),
    ))?;
    use crate::section1231::PropertyCharacter;
    let ordinary = match crate::section1231::activity_character_cents_with_claims(
        draft,
        pool,
        character,
        id,
        false,
        source_claims,
    )? {
        PropertyCharacter::Capital => 0,
        PropertyCharacter::Ordinary => own.section1231,
        PropertyCharacter::Recaptured(amount) => amount,
    };
    Ok(own.ordinary + ordinary)
}

impl FarmLandSale {
    fn partial_amounts(
        &self,
        draft: &crate::FederalDraft,
        id: i64,
        allocated: bool,
    ) -> Option<([i64; 4], [i64; 4])> {
        let source = crate::farm::sources(draft).find(|source| source.entry.id == id)?;
        let parcel = source
            .farm
            .land_basis
            .iter()
            .find(|parcel| parcel.id > 0 && parcel.id == self.basis_property)?;
        let mut parts = source
            .farm
            .land_sales
            .iter()
            .filter(|sale| sale.basis_property == parcel.id)
            .map(|sale| {
                let FarmLandSituation::PartialParcel { allocation } = &sale.situation else {
                    return None;
                };
                Some((sale.id, allocation.amounts(parcel)?))
            })
            .collect::<Option<alloc::vec::Vec<_>>>()?;
        parts.sort_by_key(|(id, _)| *id);
        if parts.iter().any(|(id, _)| *id <= 0)
            || parts.windows(2).any(|pair| pair[0].0 == pair[1].0)
        {
            return None;
        }
        let mut remaining =
            [parcel.regular, parcel.amt, parcel.prior_conservation, parcel.prior_amt_conservation];
        for (_, amounts) in &parts {
            for index in 0..4 {
                remaining[index] =
                    remaining[index].checked_sub(amounts[index]).filter(|amount| *amount >= 0)?;
            }
        }
        // Project one physical source in accepted sale-ID order, followed by
        // the retained part. The shared prefix owner keeps all cent shares
        // within that source, including when rows are renamed or reordered.
        let mut prefixes: [crate::kernel::ratio::Prefix; 4] =
            core::array::from_fn(|_| Default::default());
        let project = |amount| {
            if allocated { crate::community::property_amount(draft, id, amount) } else { amount }
        };
        let mut selected = None;
        for (id, amounts) in parts {
            let amounts =
                core::array::from_fn(|index| prefixes[index].next(amounts[index], project));
            if id == self.id {
                selected = Some(amounts);
            }
        }
        let retained =
            core::array::from_fn(|index| prefixes[index].next(remaining[index], project));
        Some((selected?, retained))
    }
    #[cfg(test)]
    pub(crate) fn resolve(
        &self,
        draft: &crate::FederalDraft,
        id: i64,
        farm: &crate::FarmIncome,
    ) -> Option<ResolvedFarmLandSale> {
        let source_claims = crate::related_forms::effective_claimants(draft);
        self.resolve_with_claims(draft, id, farm, &source_claims)
    }

    pub(crate) fn resolve_with_claims(
        &self,
        draft: &crate::FederalDraft,
        id: i64,
        farm: &crate::FarmIncome,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
    ) -> Option<ResolvedFarmLandSale> {
        self.resolve_view_with_claims(draft, id, farm, false, source_claims)
    }
    // Monetary parcel inputs already use the return's property fraction;
    // project externally linked cooperative adjustments by that same fraction.
    #[cfg(test)]
    pub(crate) fn resolve_for_return(
        &self,
        draft: &crate::FederalDraft,
        id: i64,
        farm: &crate::FarmIncome,
    ) -> Option<ResolvedFarmLandSale> {
        let source_claims = crate::related_forms::effective_claimants(draft);
        self.resolve_for_return_with_claims(draft, id, farm, &source_claims)
    }

    pub(crate) fn resolve_for_return_with_claims(
        &self,
        draft: &crate::FederalDraft,
        id: i64,
        farm: &crate::FarmIncome,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
    ) -> Option<ResolvedFarmLandSale> {
        self.resolve_view_with_claims(draft, id, farm, true, source_claims)
    }

    fn resolve_view_with_claims(
        &self,
        draft: &crate::FederalDraft,
        id: i64,
        farm: &crate::FarmIncome,
        allocated: bool,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
    ) -> Option<ResolvedFarmLandSale> {
        let parcel = farm.land_basis.iter().find(|p| p.id > 0 && p.id == self.basis_property)?;
        if !self.valid() || !self.situation.direct() || parcel.acquired > self.sold {
            return None;
        }
        let mut history = parcel.prior_cost_sharing.clone();
        for source in farm.specials.cost_sharing.iter().filter(|source| source.asset == parcel.id) {
            if source.received > self.sold {
                return None;
            }
            let amounts = source.current_amounts()?;
            if amounts.valuation_missing {
                return None;
            }
            history.push(crate::Farm126History {
                received: source.received,
                excluded: amounts.excludable,
                statement: source.description.clone(),
            });
        }
        let cost_sharing_recapture = history
            .iter()
            .map(|h| {
                let elapsed =
                    (10..20).find(|years| self.sold <= crate::dates::add_years(h.received, *years));
                let percent = elapsed.map_or(0, |years| 100 - (years as i64 - 10) * 10);
                fraction(h.excluded, percent, 100)
            })
            .sum();
        let regular175 = crate::farm_conservation::parcel_adjustment_with_claims(
            draft,
            id,
            farm,
            parcel.id,
            self.sold,
            false,
            source_claims,
        )?;
        let amt175 = crate::farm_conservation::parcel_adjustment_with_claims(
            draft,
            id,
            farm,
            parcel.id,
            self.sold,
            true,
            source_claims,
        )?;
        let (mut cooperative_basis, mut cooperative_proceeds) =
            crate::cooperative::land_adjustments_with_claims(draft, id, parcel.id, source_claims);
        if allocated {
            cooperative_basis = crate::community::property_amount(draft, id, cooperative_basis);
            cooperative_proceeds =
                crate::community::property_amount(draft, id, cooperative_proceeds);
        }
        let (regular, amt, conservation, amt_conservation) = match &self.situation {
            FarmLandSituation::PartialParcel { .. } => {
                // These sources belong to the whole parcel. Until they identify
                // the affected tract, no common fraction establishes their
                // sold/retained allocation (Pub. 225, adjusted basis of part sold).
                let regular_basis = farm.parcel_basis(parcel.id, false)?;
                let amt_basis = farm.parcel_basis(parcel.id, true)?;
                if cost_sharing_recapture != 0
                    || regular_basis != parcel.regular
                    || amt_basis != parcel.amt
                    || regular175 != (0, 0)
                    || amt175 != (0, 0)
                    || cooperative_basis != 0
                    || cooperative_proceeds != 0
                {
                    return None;
                }
                // Preserve the existing mandatory basis-owner limitations even
                // when the entered split itself is known.
                let ([regular, amt, conservation, amt_conservation], _) =
                    self.partial_amounts(draft, id, allocated)?;
                (regular, amt, conservation, amt_conservation)
            }
            _ => (
                farm.parcel_basis(parcel.id, false)? - cooperative_basis + regular175.0,
                farm.parcel_basis(parcel.id, true)? - cooperative_basis + amt175.0,
                parcel.prior_conservation + regular175.1,
                parcel.prior_amt_conservation + amt175.1,
            ),
        };
        Some(ResolvedFarmLandSale {
            label: self.label.clone(),
            acquisition: parcel.acquisition,
            acquired: parcel.acquired,
            sold: self.sold,
            proceeds: self.proceeds + cooperative_proceeds,
            selling: self.selling,
            basis: regular,
            conservation,
            amt: FarmLandAmt::Different { basis: amt, conservation: amt_conservation },
            deferral: self.deferral,
            eligible: true,
            payment: self.payment.clone(),
            cost_sharing_recapture,
        })
        .filter(ResolvedFarmLandSale::valid)
    }
    fn source_key(&self) -> &'static str {
        if matches!(self.situation, FarmLandSituation::PartialParcel { .. }) {
            "ISSUE_FARM_LAND_PART_ALLOCATION_NEEDED"
        } else if self.situation == FarmLandSituation::Direct {
            "ISSUE_FARM_LAND_LINKED_BASIS_NEEDED"
        } else {
            "ISSUE_FARM_LAND_TRANSACTION_OWNER_NEEDED"
        }
    }
    #[cfg(test)]
    pub(crate) fn report(
        &self,
        draft: &crate::FederalDraft,
        farm: &crate::FarmIncome,
        id: i64,
        index: usize,
        form: crate::farm::FarmForm,
        r: &mut FederalResult,
    ) {
        let source_claims = crate::related_forms::effective_claimants(draft);
        self.report_with_claims(
            crate::related_forms::RelatedFormContext::original(draft, &source_claims),
            farm,
            id,
            index,
            form,
            r,
        )
    }

    pub(crate) fn report_with_claims(
        &self,
        form_context: crate::related_forms::RelatedFormContext<'_>,
        farm: &crate::FarmIncome,
        id: i64,
        index: usize,
        form: crate::farm::FarmForm,
        r: &mut FederalResult,
    ) {
        let crate::related_forms::RelatedFormContext { draft, claims: source_claims, .. } =
            form_context;
        if let Some(sale) = self.resolve_for_return_with_claims(draft, id, farm, source_claims) {
            sale.report(id, index, form, r);
            if matches!(self.situation, FarmLandSituation::PartialParcel { .. }) {
                // Retained balances close the same physical source projection.
                let first = crate::farm::sources(draft)
                    .find(|source| source.entry.id == id)
                    .and_then(|source| {
                        source
                            .farm
                            .land_sales
                            .iter()
                            .filter(|sale| sale.basis_property == self.basis_property)
                            .min_by_key(|sale| sale.id)
                    });
                if first.is_some_and(|sale| sale.id == self.id)
                    && let Some((_, [regular, amt, conservation, amt_conservation])) =
                        self.partial_amounts(draft, id, true)
                {
                    for (name, amount) in [
                        ("regular", regular),
                        ("amt", amt),
                        ("conservation", conservation),
                        ("amt_conservation", amt_conservation),
                    ] {
                        crate::line_emission::line_cents(
                            r,
                            &format!("farm_{id}_land_{index}_retained_{name}"),
                            amount,
                            &[&form.input("land_sales"), &form.input("land_basis")],
                            "https://www.irs.gov/publications/p225",
                            msg!("EXPLAIN_FARM_LAND_RETAINED_PART"),
                        );
                    }
                }
            }
        } else {
            let context = if matches!(self.situation, FarmLandSituation::PartialParcel { .. }) {
                crate::IssueContext::income_id(id).item("land_sales", index).named(&self.label)
            } else {
                crate::IssueContext::Section(crate::TaxSection::Farming)
            };
            crate::require_localized!(
                r,
                false,
                &context,
                "Farm parcel disposition source",
                self.source_key(),
                "This sale needs its canonical parcel basis, recapture history and actual transaction owner. Partial, related-party, exchange, special-basis or unresolved transfers cannot be omitted as an unclaimed benefit.",
            );
        }
    }
}
