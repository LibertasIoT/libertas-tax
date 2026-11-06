// Coverage: IRC179D direct rental-owner EECBP, 2026 documentary
// cohort. Actual whole buildings and cost rows retain separate identities.
// IRS i7205(12/2025), RP2025-32§3.25, IRC179D(b),(e),(f),(i),
// P.L.119-21§70507. Statutory179D(b)(1) controls the cap. Retrofit adjusted
// basis and other years remain named source boundaries; UBIA retains original cost.
use crate::kernel::{
    ratio::{self, exact},
    source_fact::SourceFact,
};
use crate::{FederalIncome, FederalRental};
use alloc::vec::Vec;

pub use crate::rental_energy_model::{
    BuildingRecoveryRecord, RentalEnergyBuildingFacts, RentalEnergyBuildingFactsPage,
    RentalEnergyCertification, RentalEnergyClaim, RentalEnergyClaimPage,
    RentalEnergyDeductionSource, RentalEnergyIncreasedAmount, RentalEnergyProject,
    RentalEnergySystem,
};

pub(crate) const HISTORY: &str = "ISSUE_RENTAL_ENERGY_HISTORY_UNAVAILABLE";
pub(crate) const CERTIFICATION: &str = "ISSUE_RENTAL_ENERGY_CERTIFICATION_UNAVAILABLE";
pub(crate) const RETROFIT: &str = "ISSUE_RENTAL_ENERGY_RETROFIT_BASIS_UNAVAILABLE";
pub(crate) const YEAR: &str = "ISSUE_RENTAL_ENERGY_YEAR_UNAVAILABLE";
pub(crate) const BASIS: &str = "ISSUE_RENTAL_ENERGY_BASIS_UNAVAILABLE";
pub(crate) const HISTORICAL_BASIS: &str = "ISSUE_RENTAL_ENERGY_HISTORICAL_BASIS_UNAVAILABLE";

// Printed precision is two decimal places, matching the source declaration: the stored natural
// value becomes an exact ratio of whole hundredths through the shared boundary.
fn decimal(value: f64) -> Option<exact::Exact> {
    exact::ratio(crate::kernel::natural::exact(value, 100)?, 100)
}

pub(crate) fn source_issue(v: &FederalRental) -> Option<&'static str> {
    let valid_date = crate::federal_rules::valid_date;
    for building in &v.energy_buildings {
        if building.recovery.as_ref().is_some_and(|facts| !facts.valid())
            || building.building_source.trim().is_empty()
            || decimal(building.square_feet).is_none_or(|area| area <= exact::whole(0))
            || building.ownership_numerator <= 0
            || building.ownership_denominator < building.ownership_numerator
        {
            return Some("RENTAL_ENERGY_BUILDING_RECORDS");
        }
        for (i, row) in building.deductions.iter().enumerate() {
            if row.claimant_source.trim().is_empty()
                || row.amount.source_fact().is_err()
                || building.deductions[..i].iter().any(|prior| prior.tax_year == row.tax_year)
            {
                return Some("RENTAL_ENERGY_HISTORY_RECORDS");
            }
        }
    }
    for (i, cost) in v.buildings.iter().enumerate() {
        let Some(claim) = &cost.energy else { continue };
        if claim.project_source.trim().is_empty()
            || claim.physical_building <= 0
            || !v.energy_buildings.iter().any(|b| b.id == claim.physical_building)
            || !valid_date(claim.construction_began)
            || !valid_date(claim.installation_began)
            || claim.installation_began < claim.construction_began
            || claim.installation_began > cost.in_service
            || !(0..=cost.cost - cost.land - cost.rehabilitation_reduction())
                .contains(&claim.qualifying_cost)
            || v.buildings[..i]
                .iter()
                .filter_map(|b| b.energy.as_ref())
                .any(|prior| prior.project_source == claim.project_source)
        {
            return Some("RENTAL_ENERGY_CLAIM_RECORDS");
        }
        let cert = match &claim.project {
            RentalEnergyProject::Eecbp { certification } => certification.as_ref(),
            RentalEnergyProject::Retrofit { final_certification, .. } => Some(final_certification),
        };
        if cert.is_some_and(|c| {
            c.source_reference.trim().is_empty()
                || c.certifier.trim().is_empty()
                || c.license_source.trim().is_empty()
                || c.calculation_source.trim().is_empty()
                || !valid_date(c.certified)
                || c.certified < claim.installation_began
                || decimal(c.savings_percent)
                    .is_none_or(|p| p < exact::whole(0) || p > exact::whole(100))
        }) || matches!(&claim.increased, RentalEnergyIncreasedAmount::WageAndApprenticeship { source_reference } if source_reference.trim().is_empty())
            || matches!(claim.increased, RentalEnergyIncreasedAmount::EarlyInstallation)
                && claim.installation_began >= 20230129
        {
            return Some("RENTAL_ENERGY_CERTIFICATION_RECORDS");
        }
    }
    None
}

pub(crate) struct Prepared {
    pub reductions: Vec<i64>,
    pub boundaries: Vec<&'static str>,
}

// Whole-building allowance allocated by actual eligible costs with the one
// exact allocator. No component repeats a building's area or ownership share.
pub(crate) fn calculate(v: &FederalRental, year: i32) -> Prepared {
    let mut result =
        Prepared { reductions: alloc::vec![0; v.buildings.len()], boundaries: Vec::new() };
    let Ok(year) = u32::try_from(year) else {
        result.boundaries.push(YEAR);
        return result;
    };
    // A building-wide history amount does not establish which retained old
    // cost source received its basis reduction. No label or row position can
    // supply the missing physical association/opening-basis source.
    if v.buildings.iter().any(|cost| cost.in_service / 10000 < year)
        && v.energy_buildings.iter().flat_map(|building| &building.deductions).any(|row| row.tax_year < year && matches!(row.amount.source_fact(), Ok(SourceFact::Reported(amount)) if amount > 0)) {
        result.boundaries.push(HISTORICAL_BASIS);
        return result;
    }
    for building in &v.energy_buildings {
        let claims: Vec<_> = v
            .buildings
            .iter()
            .enumerate()
            .filter_map(|(i, cost)| {
                cost.energy
                    .as_ref()
                    .filter(|c| c.physical_building == building.id)
                    .map(|claim| (i, cost, claim))
            })
            .collect();
        let claims: Vec<_> = claims
            .into_iter()
            .filter(|(_, _, claim)| {
                if claim.construction_began > 20260630 {
                    return false;
                }
                match &claim.project {
                    RentalEnergyProject::Eecbp { certification: Some(cert) } => {
                        cert.unrelated_to_claimant
                            && decimal(cert.savings_percent)
                                .is_none_or(|savings| savings >= exact::whole(25))
                    }
                    _ => true,
                }
            })
            .collect();
        if claims.is_empty() {
            continue;
        }
        if year != 2026 {
            result.boundaries.push(YEAR);
            continue;
        }
        let history: Option<Vec<_>> = (year - 3..=year)
            .map(|y| {
                building
                    .deductions
                    .iter()
                    .find(|r| r.tax_year == y)
                    .and_then(|r| r.amount.source_fact().ok())
            })
            .collect();
        let Some(history) = history else {
            result.boundaries.push(HISTORY);
            continue;
        };
        if history.iter().any(|f| matches!(f, SourceFact::Unavailable)) {
            result.boundaries.push(HISTORY);
            continue;
        }
        let prior: exact::Exact = history
            .iter()
            .filter_map(|fact| match fact {
                SourceFact::Reported(value) => Some(exact::whole(*value)),
                _ => None,
            })
            .sum();
        let mut eligible = Vec::new();
        let mut rate: Option<exact::Exact> = None;
        for (index, cost, claim) in claims {
            if claim.construction_began > 20260630 {
                continue;
            }
            let RentalEnergyProject::Eecbp { certification } = &claim.project else {
                result.boundaries.push(RETROFIT);
                continue;
            };
            let Some(cert) = certification.as_ref() else {
                result.boundaries.push(CERTIFICATION);
                continue;
            };
            if !cert.unrelated_to_claimant {
                continue;
            }
            if cost.in_service / 10000 != year
                || !matches!(cost.acquired, crate::RentalAcquisition::Purchased)
                || cost.partial.is_some()
                || !matches!(v.sale, crate::RentalSale::No)
                || !matches!(v.personal_use, crate::extended_model::RentalPersonalUse::No)
            {
                result.boundaries.push(BASIS);
                continue;
            }
            let Some(savings) = decimal(cert.savings_percent) else {
                result.boundaries.push(CERTIFICATION);
                continue;
            };
            if savings < exact::whole(25) {
                continue;
            }
            let (base, increment, cap) =
                if matches!(claim.increased, RentalEnergyIncreasedAmount::Ordinary) {
                    (59, 2, 119)
                } else {
                    (297, 12, 594)
                };
            let applicable = (exact::whole(base)
                + (savings - exact::whole(25)) * exact::whole(increment))
            .min(exact::whole(cap));
            // Different certifications/rates on one physical building require
            // a genuine building-level source allocation; do not pick the best.
            if rate.as_ref().is_some_and(|prior| *prior != applicable) {
                result.boundaries.push("ISSUE_RENTAL_ENERGY_BUILDING_ALLOCATION_UNAVAILABLE");
                eligible.clear();
                break;
            }
            rate = Some(applicable);
            eligible.push((index, claim.qualifying_cost));
        }
        let Some(rate) = rate else { continue };
        if eligible.is_empty() {
            continue;
        }
        let Some(area) = decimal(building.square_feet) else { continue };
        let Some(ownership) =
            exact::ratio(building.ownership_numerator, building.ownership_denominator)
        else {
            continue;
        };
        let total: exact::Exact = eligible.iter().map(|(_, cost)| exact::whole(*cost)).sum();
        // IRC179D(b)(1): subtract actual relevant deductions from the
        // applicable-rate building amount, then apply the documented share.
        // The Form7205 full-rate shortcut does not replace this statutory cap.
        let allowed = ((area * rate - prior).max(exact::whole(0)) * ownership).min(total);
        // D02 2026-10-19: compute the entire allowance exactly, then
        // round once half-up to cents. The same amount is the deduction
        // and179D(e) basis reduction; no operand is rounded.
        let allowed = exact::rounded_exact(&allowed);
        let Some(parts) = ratio::allocate_remaining_exact(
            &allowed,
            &eligible.iter().map(|(_, cost)| *cost).collect::<Vec<_>>(),
        ) else {
            result.boundaries.push(BASIS);
            continue;
        };
        for ((index, _), amount) in eligible.into_iter().zip(parts) {
            result.reductions[index] = amount;
        }
    }
    result.boundaries.sort_unstable();
    result.boundaries.dedup();
    result
}

pub(crate) fn graph_issue(draft: &crate::FederalDraft) -> Option<&'static str> {
    let sources = crate::source_associations::SourceAssociations::new(draft);
    for entry in &draft.income {
        let FederalIncome::Rental { data } = &entry.income else { continue };
        if data.energy_buildings.iter().any(|building| building.id <= 0) {
            return Some("RENTAL_ENERGY_BUILDING_ASSOCIATION");
        }
        for cost in &data.buildings {
            if let Some(claim) = &cost.energy
                && (sources.physical_building(entry.id, cost.id).is_err()
                    || !data.energy_buildings.iter().any(|b| b.id == claim.physical_building))
            {
                return Some("RENTAL_ENERGY_BUILDING_ASSOCIATION");
            }
        }
    }
    None
}

// Compute each real whole-building cap before the existing community owner
// projects its monetary source share. Never apply a whole cap to divided cost.
pub(crate) fn for_source(draft: &crate::FederalDraft, source: i64, whole: bool) -> Prepared {
    let Some(entry) = draft.income.iter().find(|entry| entry.id == source) else {
        return Prepared { reductions: Vec::new(), boundaries: alloc::vec![BASIS] };
    };
    let FederalIncome::Rental { data } = &entry.income else {
        return Prepared { reductions: Vec::new(), boundaries: alloc::vec![BASIS] };
    };
    let sources = crate::source_associations::SourceAssociations::new(draft);
    if data
        .buildings
        .iter()
        .filter(|cost| cost.energy.is_some())
        .any(|cost| sources.physical_building(source, cost.id).is_err())
    {
        return Prepared {
            reductions: alloc::vec![0; data.buildings.len()],
            boundaries: alloc::vec![BASIS],
        };
    }
    let mut prepared = calculate(data, draft.tax_year);
    if !whole
        && let Some(share) = draft.setup.as_ref().and_then(|setup| {
            crate::community::share(setup, entry.owner.owned(), entry.community, &entry.income)
        })
    {
        for reduction in &mut prepared.reductions {
            *reduction = share.filer(*reduction);
        }
    }
    prepared
}
