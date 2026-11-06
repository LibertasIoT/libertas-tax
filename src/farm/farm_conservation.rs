use crate::validation::ValidationIssue;
// Coverage: section175 conservation deductions, partial.
// Actual plan/work/land-use and original-election/approved-change facts establish
// eligibility. Current costs, district assessment
// years and retained regular/AMT carry link to accepted parcels of their own farm.
// The existing taxpayer-wide25% pool allocates by eligible activity cost with
// stable accepted-ID exact-cent remainder; this split is a bookkeeping convention.
// Separate AMT carries feed the existing business/rental AMT income collectors.
// Schedule F line12 and Form4835 line10 report the allowed deduction.
// Tests: src/tests/farm/farm.rs, src/tests/farm/farm_rental.rs, src/tests/farm/farm_specials.rs.
// Sources: Pub225(2025)ch5;26CFR1.175-4/-5/-6.
// Current unrelated fully paid MACRS livestock sales use their actual draft,
// breeding or dairy purpose and the canonical recognized gain; sport is excluded.
// Remaining work: other economic production-gross source classification, supported
// current-sale basis/1252 allocation, capitalization-method scope and the distinct
// AMT prepaid denominator. The last combination names a required-tax boundary.
use crate::kernel::phase;
use crate::kernel::ratio::{fraction, positive};
use crate::line_emission::line_cents;
use crate::{FederalDraft, FederalResult};
use alloc::{collections::BTreeMap, format, vec::Vec};

use crate::farm_conservation_model::land_capital_cost;
#[allow(unused_imports)]
pub use crate::farm_conservation_model::{
    ConservationAssessment, ConservationAssessmentPayment, ConservationCost, FarmConservation,
    FarmConservationCapitalMethod, FarmConservationMethod,
};

impl FarmConservation {
    pub(crate) fn report_with_claims(
        &self,
        _gross: i64,
        d: &FederalDraft,
        id: i64,
        form: crate::farm::FarmForm,
        r: &mut FederalResult,
        source_claims: &alloc::collections::BTreeMap<i64, i64>,
    ) {
        let context = d
            .income
            .iter()
            .find(|e| e.id == id)
            .map(crate::interview::income_issue_context)
            .unwrap_or(crate::IssueContext::Section(crate::TaxSection::Farming));
        crate::require_localized!(
            r,
            consistent_methods(d),
            &context,
            "Farm conservation method",
            "FARM_CONSERVATION_METHOD_CONFLICT",
            "A taxpayer's never-elected capitalization history conflicts with an established section175 deduction method on another farm. Resolve the original returns or actual project-specific IRS consent before treating these costs.",
        );
        if self.facts_omitted() {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_FARM_CONSERVATION_FACTS_OMITTED"),
                context.clone(),
            );
        }
        let input = form.input("conservation");
        let deceased = owner_deceased(d, id);
        let allowed = deductions_with_claims(d, source_claims);
        if let Some((limit, _)) = &allowed {
            line_cents(
                r,
                &format!("farm_{id}_conservation_income_limit"),
                *limit,
                &[form.root()],
                "https://www.irs.gov/publications/p225",
                msg!("EXPLAIN_FARM_N_CONSERVATION_INCOME_LIMIT"),
            );
        }

        let amt_allowed = deductions_for_with_claims(d, true, source_claims);
        if matches!(self, Self::Deduct { .. }) && (allowed.is_none() || amt_allowed.is_none()) {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_FARM_CONSERVATION_GROSS_SOURCE"),
                crate::asset_sale::conservation_livestock_gross_with_claims(
                    d,
                    false,
                    source_claims,
                )
                .err()
                .or_else(|| {
                    crate::asset_sale::conservation_livestock_gross_with_claims(
                        d,
                        true,
                        source_claims,
                    )
                    .err()
                })
                .unwrap_or_else(|| context.clone()),
            );
        }
        let deduction = allowed.as_ref().and_then(|(_, v)| v.get(&id)).copied().unwrap_or(0);
        let assessment_amounts = self.assessment_amounts(deceased);
        crate::require_localized!(
            r,
            !deceased || assessment_amounts[1] == 0,
            &crate::IssueContext::Section(crate::TaxSection::Farming),
            "Farm conservation",
            "ISSUE_AN_ASSESSMENT_FIRST_SPREAD_IN_THE_YEAR_OF_DEATH_REQUIRES_SEPARATE_F_18A72DF0",
            "An assessment first spread in the year of death requires separate final-year treatment; the statutory acceleration covers death during the nine succeeding years.",
        );
        let amt_deduction =
            amt_allowed.as_ref().and_then(|(_, values)| values.get(&id)).copied().unwrap_or(0);
        let available = |amt| {
            if d.setup.as_ref().is_some_and(crate::community::applies)
                && d.income.iter().any(|entry| {
                    entry.id == id && matches!(entry.income, crate::FederalIncome::Business { .. })
                })
            {
                column_available(d, id, self, amt, false)
            } else {
                Some(self.available_for_tax(deceased, amt))
            }
        };
        let close = |amt, deduction| {
            available(amt).and_then(|available| {
                crate::kernel::carry::closing_values::<crate::kernel::carry::Cents>(
                    available, deduction,
                )
                .ok()
                .map(|value| value.get())
            })
        };
        let amt_unused = close(true, amt_deduction);
        let unused = close(false, deduction);
        if unused.is_none() || amt_unused.is_none() {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_FARM_CONSERVATION_CARRY_SOURCE"),
                context.clone(),
            );
        }
        for (name, amount, explanation) in [
            (
                "conservation_amt_deduction",
                amt_deduction,
                msg!("EXPLAIN_FARM_CONSERVATION_AMT_DEDUCTION"),
            ),
            (
                "conservation_amt_carryforward",
                if deceased { 0 } else { amt_unused.unwrap_or(0) },
                msg!("EXPLAIN_FARM_CONSERVATION_AMT_CARRY"),
            ),
            (
                "conservation_expired",
                if deceased { unused.unwrap_or(0) } else { 0 },
                msg!("EXPLAIN_FARM_N_N_CONSERVATION_EXPIRED"),
            ),
            (
                "conservation_assessment_available",
                assessment_amounts[0],
                msg!("EXPLAIN_FARM_N_N_CONSERVATION_ASSESSMENT_AVAILABLE"),
            ),
            (
                "conservation_assessment_deferred",
                assessment_amounts[1],
                msg!("EXPLAIN_FARM_N_N_CONSERVATION_ASSESSMENT_DEFERRED"),
            ),
            ("conservation_deduction", deduction, msg!("EXPLAIN_FARM_N_N_CONSERVATION_DEDUCTION")),
            (
                "conservation_carryforward",
                if deceased { 0 } else { unused.unwrap_or(0) },
                msg!("EXPLAIN_FARM_N_N_CONSERVATION_CARRYFORWARD"),
            ),
            (
                "conservation_basis_addition",
                if let Self::Capitalize { method, .. } = self {
                    if method.established() { self.paid() } else { 0 }
                } else {
                    assessment_amounts[2]
                        + self
                            .costs()
                            .iter()
                            .filter(|cost| !cost.facts.qualifies() && land_capital_cost(cost))
                            .map(|cost| cost.amount)
                            .sum::<i64>()
                },
                msg!("EXPLAIN_FARM_N_N_CONSERVATION_BASIS_ADDITION"),
            ),
        ] {
            let tag = (name == "conservation_deduction").then(|| {
                crate::FormLine::new(
                    form.tax_form(),
                    match form {
                        crate::farm::FarmForm::ScheduleF => "12",
                        crate::farm::FarmForm::Form4835 => "10",
                    },
                )
            });
            if (name == "conservation_amt_carryforward" && amt_unused.is_none())
                || ((name == "conservation_carryforward" || name == "conservation_expired")
                    && unused.is_none())
            {
                continue;
            }
            crate::line_emission::push_line(
                r,
                tag,
                &format!("farm_{id}_{name}"),
                amount,
                &[&input, "people"],
                "https://www.irs.gov/publications/p225",
                explanation,
            );
        }
    }
}

// The section 175 limit belongs to the taxpayer, not to each Schedule F.
// Allocate a binding limit in proportion to each activity's eligible cost.
// Allocation is a bookkeeping convention; the IRS does not mandate this split.
// Accepted source IDs determine the exact-cent remainder independently of order.

fn deductions_with_claims(
    d: &FederalDraft,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<(i64, BTreeMap<i64, i64>)> {
    deductions_for_with_claims(d, false, source_claims)
}

// Prepared direct-business inputs need the actual selected pool, not the
// legacy report's zero fallback. Keep the pool's existing community view.

pub(crate) fn selected_expense_adjustments_with_claims(
    d: &FederalDraft,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<BTreeMap<i64, i64>, (i64, &'static str)> {
    let farms: Vec<_> = d
        .income
        .iter()
        .filter_map(|entry| {
            let crate::FederalIncome::Business { data: business } = &entry.income else {
                return None;
            };
            let crate::BusinessActivity::Farming { data: farm } = &business.activity else {
                return None;
            };
            Some((entry.id, business, farm, farm.conservation.as_ref()?))
        })
        .collect();
    let Some((first, _, _, _)) = farms.first() else {
        return Ok(BTreeMap::new());
    };
    if !consistent_methods(d) {
        return Err((*first, "FARM_CONSERVATION_METHOD_CONFLICT"));
    }
    if let Some((id, _, _, _)) =
        farms.iter().find(|(_, _, _, conservation)| conservation.facts_omitted())
    {
        return Err((*id, "WARN_FARM_CONSERVATION_FACTS_OMITTED"));
    }
    let allowed = if farms
        .iter()
        .any(|(_, _, _, conservation)| matches!(conservation, FarmConservation::Deduct { .. }))
    {
        deductions_for_with_claims(d, amt, source_claims)
            .ok_or((*first, "WARN_FARM_CONSERVATION_GROSS_SOURCE"))?
            .1
    } else {
        BTreeMap::new()
    };
    farms.into_iter().map(|(id, business, farm, conservation)| {
        if owner_deceased(d, id) && conservation.assessment_amounts(true)[1] != 0 {
            return Err((id, "ISSUE_AN_ASSESSMENT_FIRST_SPREAD_IN_THE_YEAR_OF_DEATH_REQUIRES_SEPARATE_F_18A72DF0"));
        }
        let selected = if matches!(conservation, FarmConservation::Deduct { .. }) {
            if d.setup.as_ref().is_some_and(crate::community::applies) {
                operator_deduction_with_claims(d, id, amt, source_claims).ok_or((id, "WARN_FARM_CONSERVATION_GROSS_SOURCE"))?
            } else { *allowed.get(&id).ok_or((id, "WARN_FARM_CONSERVATION_GROSS_SOURCE"))? }
        } else { 0 };
        let local = conservation.deduction(farm.gross() + crate::method_change::increase(&business.method_changes));
        Ok((id, selected - local))
    }).collect()
}

// A retained tax balance belongs to the taxpayer whose source records
// supplied it. Community current costs divide; historical balances do not.
fn column_available(
    d: &FederalDraft,
    id: i64,
    conservation: &FarmConservation,
    amt: bool,
    counterpart: bool,
) -> Option<i64> {
    let whole_current = conservation.current_available(owner_deceased(d, id), amt);
    let current = crate::community::SourceAmount::income(d, id, whole_current);
    let current = if counterpart { current.counterpart } else { current.filer };
    let retained = match conservation {
        FarmConservation::Deduct { carry, .. } => {
            carry.iter().map(|row| if amt { row.amt } else { row.regular }).sum()
        }
        _ => 0,
    };
    if retained == 0 {
        return Some(current);
    }
    let entry = d.income.iter().find(|entry| entry.id == id)?;
    let owner_is_filer = entry.owner == crate::DocumentOwner::Yours;
    if owner_is_filer != counterpart { current.checked_add(retained) } else { None }
}

fn deductions_for_with_claims(
    d: &FederalDraft,
    amt: bool,
    claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<(i64, BTreeMap<i64, i64>)> {
    deductions_for_column_with_claims(d, amt, claims, false)
}

fn deductions_for_column_with_claims(
    d: &FederalDraft,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
    counterpart: bool,
) -> Option<(i64, BTreeMap<i64, i64>)> {
    if !consistent_methods(d) {
        return None;
    }
    // Schedule F farms and Form 4835 farm rentals share one limit: the rent
    // is gross income from the business of farming (Reg. 1.175-3, 1.175-5).
    let livestock =
        crate::asset_sale::physical_conservation_livestock_gross_with_claims(d, amt, source_claims)
            .ok()?;
    let income_part = |id, amount| {
        let source = crate::community::SourceAmount::income(d, id, amount);
        if counterpart { source.counterpart } else { source.filer }
    };
    let allocated_entries = crate::community::entries(d);
    let farms: Vec<_> = allocated_entries
        .iter()
        .filter_map(|entry| {
            let (farm, method_increase) = match &entry.income {
                crate::FederalIncome::Business { data } => match &data.activity {
                    crate::BusinessActivity::Farming { data: farm } => {
                        (farm, crate::method_change::increase(&data.method_changes))
                    }
                    _ => return None,
                },
                crate::FederalIncome::FarmRental { data } => (&data.farm, 0),
                _ => return None,
            };
            let gross = farm.conservation_gross().and_then(|gross| {
                let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
                let cooperative = crate::cooperative::conservation_gross_with_claims(
                    d,
                    entry.owner.owned(),
                    entry.id,
                    source_claims,
                )?;
                Some(
                    (if matches!(entry.income, crate::FederalIncome::Business { .. }) {
                        income_part(entry.id, gross + method_increase)
                    } else {
                        gross + method_increase
                    }) + {
                        let amount = *livestock.get(&entry.id)?;
                        if matches!(entry.income, crate::FederalIncome::Business { .. }) {
                            let source =
                                crate::community::SourceAmount::property(d, entry.id, amount);
                            if counterpart { source.counterpart } else { source.filer }
                        } else if counterpart {
                            return None;
                        } else {
                            amount
                        }
                    } + income_part(
                        entry.id,
                        cooperative
                            + crate::farm_insurance_paper::income_delta(d, entry.id, source_claims)
                            + crate::federal_rules::farm_1099g_program(
                                d,
                                owner,
                                entry.id,
                                source_claims,
                            ),
                    ),
                )
            });
            Some((entry.as_ref(), farm, gross))
        })
        .collect();
    let owner = farms.first()?.0.owner;
    let joint = d.setup.as_ref().is_some_and(|s| s.filing_status() == crate::FederalStatus::Joint);
    let community = d.setup.as_ref().is_some_and(crate::community::applies);
    if farms
        .iter()
        .any(|(e, _, gross)| (!joint && !community && e.owner != owner) || gross.is_none())
    {
        return None;
    }
    let gross = farms.iter().map(|(_, _, gross)| gross.unwrap()).sum::<i64>();
    let limit = fraction(positive(gross), 25, 100);
    let mut available = BTreeMap::new();
    let mut unavailable = Vec::new();
    for (entry, farm, _) in &farms {
        let amount = if let Some(conservation) = &farm.conservation {
            if community && matches!(entry.income, crate::FederalIncome::Business { .. }) {
                column_available(d, entry.id, conservation, amt, counterpart)
            } else {
                Some(conservation.available_for_tax(owner_deceased(d, entry.id), amt))
            }
        } else {
            Some(0)
        };
        match amount {
            Some(amount) => {
                available.insert(entry.id, amount);
            }
            None => {
                unavailable.push((entry.id, farm.conservation.as_ref()?));
            }
        }
    }
    if !unavailable.is_empty() {
        // Current costs alone can fill a single source's cap without knowing
        // its historical balance. Other positive source capacities would need
        // actual historical attribution for the pool's source allocation.
        if unavailable.len() != 1 || available.values().any(|amount| *amount != 0) {
            return None;
        }
        let (id, conservation) = unavailable[0];
        let current = income_part(id, conservation.current_available(owner_deceased(d, id), amt));
        if current < limit {
            return None;
        }
        available.insert(id, current);
    }
    let total: i64 = available.values().sum();
    let used = phase::cap(total, [limit]);
    let weights: Vec<_> = available.values().copied().collect();
    let allocated = crate::kernel::ratio::allocate_remaining(used, &weights);
    Some((limit, available.into_keys().zip(allocated).collect()))
}

// Section1402(a)(5)(A) assigns the actual business gross and deductions to
// its operator. Retained income-tax carry does not establish SE history.
// A current cost that fills the cap proves the current deduction regardless
// of an unknown nonnegative prior balance; no closing SE carry is inferred.
pub(crate) fn operator_deduction_with_claims(
    d: &FederalDraft,
    id: i64,
    amt: bool,
    claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<i64> {
    if !d.setup.as_ref().is_some_and(crate::community::applies) {
        return deductions_for_with_claims(d, amt, claims)?.1.get(&id).copied();
    }
    let owner = d.income.iter().find(|entry| entry.id == id)?.owner;
    let livestock =
        crate::asset_sale::physical_conservation_livestock_gross_with_claims(d, amt, claims)
            .ok()?;
    let mut gross = 0;
    let mut current = BTreeMap::new();
    let mut historical = false;
    for source in crate::farm::sources(d).filter(|source| source.entry.owner == owner) {
        let crate::FederalIncome::Business { data: business } = &source.entry.income else {
            // A nonoperator rental's own deduction attribution is separate.
            if source.farm.conservation.is_some() {
                return None;
            }
            continue;
        };
        gross += source.farm.conservation_gross()?
            + crate::farm_insurance_paper::income_delta(d, source.entry.id, claims)
            + crate::method_change::increase(&business.method_changes)
            + *livestock.get(&source.entry.id)?
            + crate::cooperative::conservation_gross_with_claims(
                d,
                owner.owned(),
                source.entry.id,
                claims,
            )?
            + crate::federal_rules::farm_1099g_program(
                d,
                usize::from(owner == crate::DocumentOwner::Spouses),
                source.entry.id,
                claims,
            );
        if let Some(conservation) = &source.farm.conservation {
            historical |= matches!(conservation, FarmConservation::Deduct { carry, .. } if carry.iter().any(|row| if amt { row.amt != 0 } else { row.regular != 0 }));
            current.insert(
                source.entry.id,
                conservation.current_available(owner_deceased(d, source.entry.id), amt),
            );
        }
    }
    let limit = fraction(positive(gross), 25, 100);
    let total: i64 = current.values().sum();
    if historical && (total < limit || current.values().filter(|amount| **amount != 0).count() != 1)
    {
        return None;
    }
    let used = phase::cap(total, [limit]);
    let weights: Vec<_> = current.values().copied().collect();
    let allocations = crate::kernel::ratio::allocate_remaining(used, &weights);
    current.into_keys().zip(allocations).find(|(source, _)| *source == id).map(|(_, amount)| amount)
}

pub(crate) fn income_deduction_with_claims(
    d: &FederalDraft,
    id: i64,
    amt: bool,
    claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<i64> {
    deductions_for_with_claims(d, amt, claims)?.1.get(&id).copied()
}

pub(crate) fn counterpart_deduction_with_claims(
    d: &FederalDraft,
    id: i64,
    amt: bool,
    claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<i64> {
    deductions_for_column_with_claims(d, amt, claims, true)?.1.get(&id).copied()
}

#[cfg(test)]
pub(crate) fn expense_adjustment(d: &FederalDraft, id: i64, b: &crate::FederalBusiness) -> i64 {
    let source_claims = crate::related_forms::effective_claimants(d);
    expense_adjustment_with_claims(d, id, b, &source_claims)
}

pub(crate) fn expense_adjustment_with_claims(
    d: &FederalDraft,
    id: i64,
    b: &crate::FederalBusiness,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> i64 {
    let crate::BusinessActivity::Farming { data } = &b.activity else {
        return 0;
    };
    let Some(c) = &data.conservation else {
        return 0;
    };
    let gross = data.gross() + crate::method_change::increase(&b.method_changes);
    // The deduction the cost stage already took is the one on the farm's own
    // records; the routed 1099-G and 1099-PATR income raises the limit.
    operator_deduction_with_claims(d, id, false, source_claims).unwrap_or(0) - c.deduction(gross)
}

// Form 1099-G box 7 and 1099-PATR income routed to this farm or farm rental
// is gross income from farming for the section 175(b) limit.

fn routed_with_claims(
    d: &FederalDraft,
    entry: &crate::FederalIncomeEntry,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::Section179PreparedIssue> {
    crate::federal_rules::farm_routed_income_with_claims(
        d,
        usize::from(entry.owner == crate::DocumentOwner::Spouses),
        entry.id,
        source_claims,
    )
}

pub(crate) fn routed_to_with_claims(
    d: &FederalDraft,
    id: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<i64, crate::Section179PreparedIssue> {
    let entry = d
        .income
        .iter()
        .find(|entry| entry.id == id)
        .ok_or(crate::Section179PreparedIssue::UnknownSource(id))?;
    routed_with_claims(d, entry, source_claims)
}

// The deduction a farm rental takes: its share of the pooled section 175 limit,
// or zero with a scoped benefit notice when actual production gross is unknown.

pub(crate) fn rental_deduction_with_claims(
    d: &FederalDraft,
    id: i64,
    farm: &crate::FarmIncome,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> i64 {
    if farm.conservation.is_none() {
        return 0;
    }
    deductions_for_with_claims(d, amt, source_claims)
        .and_then(|(_, v)| v.get(&id).copied())
        .unwrap_or(0)
}

fn owner_deceased(d: &FederalDraft, id: i64) -> bool {
    crate::farm::owner_death_date(d, id).is_some()
}

impl FarmConservation {
    pub(crate) fn parcel_issue(&self, farm: &crate::FarmIncome) -> Option<ValidationIssue> {
        let parcel = |id| farm.land_basis.iter().find(|p| p.id > 0 && p.id == id);
        for (i, cost) in self.costs().iter().enumerate() {
            if parcel(cost.property).is_none() {
                return Some(
                    ValidationIssue::new("FARM_CONSERVATION_PROPERTY_SOURCE")
                        .at_field("property")
                        .in_list("costs", i),
                );
            }
        }
        if let Self::Deduct { assessments, carry, method, .. } = self {
            if let FarmConservationMethod::OriginalElection { year } = method {
                if let Some((i, _)) =
                    carry.iter().enumerate().find(|(_, record)| record.year < *year)
                {
                    return Some(
                        ValidationIssue::new("FARM_CONSERVATION_METHOD_HISTORY")
                            .at_field("year")
                            .in_list("carry", i),
                    );
                }
                for (i, record) in assessments.iter().enumerate() {
                    if let Some((j, _)) = record
                        .payments
                        .iter()
                        .enumerate()
                        .find(|(_, payment)| payment.amount != 0 && payment.year < *year)
                    {
                        return Some(
                            ValidationIssue::new("FARM_CONSERVATION_METHOD_HISTORY")
                                .at_field("year")
                                .in_list("payments", j)
                                .in_list("assessments", i),
                        );
                    }
                }
            }
            for (i, record) in assessments.iter().enumerate() {
                if parcel(record.property).is_none() {
                    return Some(
                        ValidationIssue::new("FARM_CONSERVATION_PROPERTY_SOURCE")
                            .at_field("property")
                            .in_list("assessments", i),
                    );
                }
            }
            for (i, record) in carry.iter().enumerate() {
                if parcel(record.property).is_none() {
                    return Some(
                        ValidationIssue::new("FARM_CONSERVATION_PROPERTY_SOURCE")
                            .at_field("property")
                            .in_list("carry", i),
                    );
                }
            }
            if let Some((i, _)) = carry
                .iter()
                .enumerate()
                .find(|(_, record)| !(1954..crate::year_constants::TAX_YEAR).contains(&record.year))
            {
                return Some(
                    ValidationIssue::new("FARM_CONSERVATION_CARRY_YEAR")
                        .at_field("year")
                        .in_list("carry", i),
                );
            }
            if let Some((i, _)) = carry.iter().enumerate().find(|(i, record)| {
                carry[..*i]
                    .iter()
                    .any(|prior| prior.property == record.property && prior.year == record.year)
            }) {
                return Some(
                    ValidationIssue::new("FARM_CONSERVATION_CARRY_DUPLICATE")
                        .at_field("year")
                        .in_list("carry", i),
                );
            }
            for (i, record) in assessments.iter().enumerate() {
                if let Some((j, _)) = record.payments.iter().enumerate().find(|(_, payment)| {
                    payment.current_date.is_some()
                        && payment.year != crate::year_constants::TAX_YEAR
                }) {
                    return Some(
                        ValidationIssue::new("FARM_CONSERVATION_PAYMENT_DATE")
                            .at_field("current_date")
                            .in_list("payments", j)
                            .in_list("assessments", i),
                    );
                }
            }
            for (i, record) in assessments.iter().enumerate() {
                if let Some((j, _)) = record.payments.iter().enumerate().find(|(_, payment)| {
                    payment.year < parcel(record.property).unwrap().acquired / 10000
                }) {
                    return Some(
                        ValidationIssue::new("FARM_CONSERVATION_BEFORE_ACQUIRED")
                            .at_field("year")
                            .in_list("payments", j)
                            .in_list("assessments", i),
                    );
                }
            }
            if let Some((i, _)) = carry
                .iter()
                .enumerate()
                .find(|(_, record)| record.year < parcel(record.property).unwrap().acquired / 10000)
            {
                return Some(
                    ValidationIssue::new("FARM_CONSERVATION_BEFORE_ACQUIRED")
                        .at_field("year")
                        .in_list("carry", i),
                );
            }
        }
        for (i, cost) in self.costs().iter().enumerate() {
            if cost
                .paid_or_incurred
                .is_some_and(|date| date < parcel(cost.property).unwrap().acquired)
            {
                return Some(
                    ValidationIssue::new("FARM_CONSERVATION_BEFORE_ACQUIRED")
                        .at_field("paid_or_incurred")
                        .in_list("costs", i),
                );
            }
        }
        None
    }
}

// The business collector has already deducted the regular allowed amount.
// Supply the separate legal AMT carry/pool difference once; no second tax engine.

pub(crate) fn business_amt_adjustment_with_claims(
    d: &FederalDraft,
    id: i64,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> i64 {
    let source = crate::farm::sources(d).find(|source| source.entry.id == id && !source.rental);
    let Some(source) = source else {
        return 0;
    };
    if source.farm.conservation.is_none() {
        return 0;
    }
    let allowed = |amt| operator_deduction_with_claims(d, id, amt, source_claims).unwrap_or(0);
    allowed(false) - allowed(true)
}

// Ephemeral disposal projection from the same legal section175 pool. Aggregate
// current/carry character by accepted parcel and source year before allocating
// within an activity; unordered cost rows cannot change exact-cent ownership.
// Twenty-five-percent-limited carry never becomes basis. Only actual capital
// costs and the unexpired assessment spread do (Pub225ch5).

pub(crate) fn parcel_adjustment_with_claims(
    d: &FederalDraft,
    id: i64,
    farm: &crate::FarmIncome,
    property: i64,
    sold: u32,
    amt: bool,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Option<(i64, i64)> {
    if !consistent_methods(d) {
        return None;
    }
    let Some(conservation) = &farm.conservation else {
        return Some((0, 0));
    };
    let costs = conservation.costs();
    // An actual sale needs actual current timing, rather than an invented
    // payment day or a guessed allocation of post-disposition obligations.
    if costs
        .iter()
        .filter(|cost| cost.property == property && cost.amount != 0)
        .any(|cost| cost.paid_or_incurred.is_none_or(|date| date > sold))
    {
        return None;
    }
    match conservation {
        FarmConservation::Capitalize { method, .. } => {
            if !method.established()
                && costs.iter().any(|cost| cost.property == property && cost.amount != 0)
            {
                return None;
            }
            Some((
                costs
                    .iter()
                    .filter(|cost| cost.property == property && land_capital_cost(cost))
                    .map(|cost| cost.amount)
                    .sum(),
                0,
            ))
        }
        FarmConservation::Deduct { assessments, carry, method, .. } => {
            if !method.established() {
                return None;
            }
            if assessments.iter().filter(|record| record.property == property).any(|record| {
                record.payments.iter().any(|payment| {
                    payment.amount != 0
                        && payment.year == crate::year_constants::TAX_YEAR
                        && payment.current_date.is_none_or(|date| date > sold)
                })
            }) {
                return None;
            }
            let allowed =
                deductions_for_with_claims(d, amt, source_claims)?.1.get(&id).copied().unwrap_or(0);
            let deceased = owner_deceased(d, id);
            let mut weights = BTreeMap::<(i64, u32), i64>::new();
            for cost in costs.iter().filter(|cost| cost.facts.qualifies()) {
                *weights.entry((cost.property, crate::year_constants::TAX_YEAR)).or_default() +=
                    cost.amount;
            }
            for record in carry {
                *weights.entry((record.property, record.year)).or_default() +=
                    if amt { record.amt } else { record.regular };
            }
            let mut basis = costs
                .iter()
                .filter(|cost| {
                    cost.property == property && !cost.facts.qualifies() && land_capital_cost(cost)
                })
                .map(|cost| cost.amount)
                .sum::<i64>();
            for assessment in assessments {
                if !assessment.facts.qualifies() {
                    if assessment.property == property {
                        basis += assessment
                            .payments
                            .iter()
                            .filter(|payment| payment.year == crate::year_constants::TAX_YEAR)
                            .map(|payment| payment.amount)
                            .sum::<i64>();
                    }
                    continue;
                }
                let amounts = assessment.amounts_for(deceased);
                *weights
                    .entry((assessment.property, crate::year_constants::TAX_YEAR))
                    .or_default() += amounts[0];
                if assessment.property == property {
                    basis += amounts[2] + amounts[1];
                }
            }
            let amounts = weights.values().copied().collect::<Vec<_>>();
            let allocated = crate::kernel::ratio::allocate_remaining(allowed, &amounts);
            let recapture = weights
                .into_keys()
                .zip(allocated)
                .filter(|((parcel, _), _)| *parcel == property)
                .map(|(_, amount)| amount)
                .sum();
            Some((basis, recapture))
        }
    }
}

// Original method is taxpayer-wide; an actual IRS consent can identify a
// different farm/project. Joint spouses remain separate method owners even
// though their eligible gross and costs share the joint-return25%limit.
pub(crate) fn consistent_methods(d: &FederalDraft) -> bool {
    method_issue(d).is_none()
}
pub(crate) fn method_issue(d: &FederalDraft) -> Option<(i64, crate::validation::ValidationIssue)> {
    crate::farm::sources(d).find(|source| {
        matches!(&source.farm.conservation,
            Some(FarmConservation::Capitalize { method: FarmConservationCapitalMethod::NeverElected, .. }))
            && crate::farm::sources(d).any(|other| other.entry.owner == source.entry.owner
                && (other.farm.land_basis.iter().any(|parcel| parcel.prior_conservation > 0 || parcel.prior_amt_conservation > 0)
                    || matches!(&other.farm.conservation, Some(FarmConservation::Deduct { method, .. }) if method.established())))
    }).map(|source| (source.entry.id, crate::validation::ValidationIssue::new("FARM_CONSERVATION_METHOD_CONFLICT").at_field("method").in_field("conservation")))
}
