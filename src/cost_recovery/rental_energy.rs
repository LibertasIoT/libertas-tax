// Coverage: IRC179D direct rental-owner EECBP, 2026 documentary
// cohort. Actual whole buildings and cost rows retain separate identities.
// IRS i7205(12/2025), RP2025-32§3.25, IRC179D(b),(e),(f),(i),
// P.L.119-21§70507. Statutory179D(b)(1) controls the cap. Retrofit adjusted
// basis and other years remain named source boundaries; UBIA retains original cost.
use crate::kernel::{
    ratio::{self, exact},
    source_fact::SourceFact,
};
use crate::{FederalIncome, FederalRental, SourceCents};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// A physical building's section 179D records
/// Enter each actual whole building once, even when several cost rows belong to it.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one actual physical building's section 179D certification or conditioned-area schedule exactly. Return only the printed building designation and conditioned whole-building square footage, retaining the printed decimal precision. Report missing, illegible, conflicting or multiple-building values as errors. Do not calculate area, ownership shares or a deduction. Do not change record IDs, cost-source associations, ownership fractions or current/prior deduction histories; those remain actual personal records.",
    doc_type = "application/pdf,image/*"
)]
pub struct RentalEnergyBuildingFacts {
    /// Record identity
    #[libertas_hidden]
    #[libertas_default(0)]
    pub id: i64,
    /// Building designation on the certification and ownership records
    #[libertas_size(min = 1)]
    #[libertas_llm_doc_field]
    pub building_source: String,
    /// Conditioned building area in square feet
    /// Copy the documented whole-building area, with up to two decimal places. Do not enter just your ownership share.
    #[libertas_pattern(r"^[0-9]+(\.[0-9]{1,2})?$")]
    #[libertas_llm_doc_field]
    pub square_feet: String,
    /// Numerator of your documented ownership share of the building allowance
    #[libertas_number(min = 1)]
    pub ownership_numerator: i64,
    /// Denominator of that ownership share
    #[libertas_number(min = 1)]
    pub ownership_denominator: i64,
    /// Building deduction history from all relevant parties' records
    /// Include the current year and each of the preceding three years. Each year needs an actual record establishing all relevant parties' deductions or their absence; an empty list is unavailable history. Do not enter the cost of this year's project here.
    /// ----
    /// Documentary year
    pub deductions: Vec<RentalEnergyDeductionSource>,
}

/// A documented year's whole-building section 179D deductions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalEnergyDeductionSource {
    /// Actual tax year of the deduction records
    pub tax_year: u32,
    /// Source identifying the applicable claimants and their complete records
    #[libertas_size(min = 1)]
    pub claimant_source: String,
    /// Deductions established by those records
    /// For the current year, include other parties' already claimed deductions, excluding the new cost-row claims entered here. Not reported means the records establish no deductions, not missing records.
    pub amount: SourceCents,
}

/// System covered by the certification
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalEnergySystem {
    /// Interior lighting
    InteriorLighting,
    /// Heating, cooling, ventilation and hot water
    HeatingCoolingVentilationHotWater,
    /// Building envelope
    BuildingEnvelope,
}

/// Actual energy certification
/// Use the qualified professional's signed certification; the App does not calculate engineering savings or certify legal qualification.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one signed section 179D energy certification exactly. Return only the printed certifier name, certification date, professional license and jurisdiction, reference-standard and qualified-software declaration, and savings percentage. Preserve printed percentage precision; do not compute engineering savings or legal qualification. Report missing, illegible or conflicting selected facts as errors. Do not invent or change the retained document reference, unrelated-party personal answer, physical-building or cost-row IDs, ownership, prior basis or deduction histories. A Form 7205 computed deduction is not a savings percentage.",
    doc_type = "application/pdf,image/*"
)]
pub struct RentalEnergyCertification {
    /// Certification document reference
    #[libertas_size(min = 1)]
    pub source_reference: String,
    /// Certifier's name
    #[libertas_size(min = 1)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub certifier: String,
    /// Certification date
    #[libertas_date_only]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub certified: u32,
    /// Professional license and building jurisdiction on the certification
    #[libertas_size(min = 1)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub license_source: String,
    /// The certifier is unrelated to you
    pub unrelated_to_claimant: bool,
    /// Reference standard and qualified-software declaration on the certification
    #[libertas_size(min = 1)]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub calculation_source: String,
    /// Printed savings percentage
    /// Copy the actual percentage, including up to two decimal places; do not calculate savings yourself.
    #[libertas_pattern(r"^[0-9]{1,3}(\.[0-9]{1,2})?$")]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub savings_percent: String,
}

/// Increased section 179D amount
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalEnergyIncreasedAmount {
    /// Ordinary amount; no increased amount claimed
    Ordinary,
    /// Installation actually began before January 29, 2023
    EarlyInstallation,
    /// Actual wage and apprenticeship compliance records
    WageAndApprenticeship {
        /// Reference to the retained compliance records
        #[libertas_size(min = 1)]
        source_reference: String,
    },
}

/// Actual section 179D project treatment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RentalEnergyProject {
    /// Certified energy-efficient commercial building property
    Eecbp {
        /// Certification
        /// [NullText]
        /// A qualifying certification has not been established
        /// [SetText]
        /// The actual certification is
        #[libertas_default($null)]
        certification: Option<RentalEnergyCertification>,
    },
    /// Elected energy-efficient building retrofit
    /// This source is retained for review; its final-certification-year adjusted-basis calculation is not yet available.
    Retrofit {
        /// Original building service date
        #[libertas_date_only]
        original_building_service: u32,
        /// Qualified retrofit plan established
        #[libertas_date_only]
        plan_established: u32,
        /// Baseline certification date
        #[libertas_date_only]
        baseline_certified: u32,
        /// Actual baseline certification source
        #[libertas_size(min = 1)]
        baseline_source: String,
        /// Actual final certification
        final_certification: RentalEnergyCertification,
        /// Actual election source
        #[libertas_size(min = 1)]
        election_source: String,
    },
}

/// A section 179D claim on this cost row
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalEnergyClaim {
    /// Actual physical building
    /// Save the physical building's documentary facts in More about this property before claiming from its cost rows.
    pub physical_building: i64,
    /// Project and cost-source reference
    /// Identify this cost subset once. Do not repeat the same source costs on another row.
    #[libertas_size(min = 1)]
    pub project_source: String,
    /// Certified system
    pub system: RentalEnergySystem,
    /// Actual construction commencement
    #[libertas_date_only]
    pub construction_began: u32,
    /// Actual installation commencement
    #[libertas_date_only]
    pub installation_began: u32,
    /// Your actual qualifying cost within this row's depreciable cost
    /// This is already your ownership share. Exclude the building's other costs and costs entered on another row.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub qualifying_cost: i64,
    /// Actual project treatment
    pub project: RentalEnergyProject,
    /// Increased amount treatment
    pub increased: RentalEnergyIncreasedAmount,
}

/// A section 179D cost-row claim in the rental editor
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RentalEnergyClaimPage {
    /// Actual physical building
    /// Save the physical building's documentary facts in More about this property before claiming from its cost rows.
    #[libertas_enum_source("$.rental_energy_buildings")]
    pub physical_building: i64,
    /// Project and cost-source reference
    /// Identify this cost subset once. Do not repeat the same source costs on another row.
    #[libertas_size(min = 1)]
    pub project_source: String,
    /// Certified system
    pub system: RentalEnergySystem,
    /// Actual construction commencement
    #[libertas_date_only]
    pub construction_began: u32,
    /// Actual installation commencement
    #[libertas_date_only]
    pub installation_began: u32,
    /// Your actual qualifying cost within this row's depreciable cost
    /// This is already your ownership share. Exclude the building's other costs and costs entered on another row.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub qualifying_cost: i64,
    /// Actual project treatment
    pub project: RentalEnergyProject,
    /// Increased amount treatment
    pub increased: RentalEnergyIncreasedAmount,
}

impl From<&RentalEnergyClaim> for RentalEnergyClaimPage {
    fn from(source: &RentalEnergyClaim) -> Self {
        Self {
            physical_building: source.physical_building,
            project_source: source.project_source.clone(),
            system: source.system.clone(),
            construction_began: source.construction_began,
            installation_began: source.installation_began,
            qualifying_cost: source.qualifying_cost,
            project: source.project.clone(),
            increased: source.increased.clone(),
        }
    }
}
impl From<&RentalEnergyClaimPage> for RentalEnergyClaim {
    fn from(source: &RentalEnergyClaimPage) -> Self {
        Self {
            physical_building: source.physical_building,
            project_source: source.project_source.clone(),
            system: source.system.clone(),
            construction_began: source.construction_began,
            installation_began: source.installation_began,
            qualifying_cost: source.qualifying_cost,
            project: source.project.clone(),
            increased: source.increased.clone(),
        }
    }
}

pub(crate) const HISTORY: &str = "ISSUE_RENTAL_ENERGY_HISTORY_UNAVAILABLE";
pub(crate) const CERTIFICATION: &str = "ISSUE_RENTAL_ENERGY_CERTIFICATION_UNAVAILABLE";
pub(crate) const RETROFIT: &str = "ISSUE_RENTAL_ENERGY_RETROFIT_BASIS_UNAVAILABLE";
pub(crate) const YEAR: &str = "ISSUE_RENTAL_ENERGY_YEAR_UNAVAILABLE";
pub(crate) const BASIS: &str = "ISSUE_RENTAL_ENERGY_BASIS_UNAVAILABLE";
pub(crate) const HISTORICAL_BASIS: &str = "ISSUE_RENTAL_ENERGY_HISTORICAL_BASIS_UNAVAILABLE";

// Printed decimal precision is two places, matching the source declaration.
// Preserve the source as an exact ratio; checked arithmetic never truncates it.
fn decimal(value: &str) -> Option<exact::Exact> {
    if value.ends_with('.') {
        return None;
    }
    let (whole, tail) = value.split_once('.').map_or((value, ""), |parts| parts);
    if whole.is_empty()
        || tail.len() > 2
        || !whole.bytes().chain(tail.bytes()).all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let scale = 10_i64.checked_pow(tail.len() as u32)?;
    let numerator = whole
        .parse::<i64>()
        .ok()?
        .checked_mul(scale)?
        .checked_add(if tail.is_empty() { 0 } else { tail.parse().ok()? })?;
    exact::ratio(numerator, scale)
}

pub(crate) fn ids(income: &FederalIncome) -> Vec<i64> {
    match income {
        FederalIncome::Rental { data } => data
            .energy_buildings
            .iter()
            .map(|b| b.id)
            .chain(data.buildings.iter().map(|b| b.id))
            .collect(),
        _ => Vec::new(),
    }
}
pub(crate) fn ids_mut(income: &mut FederalIncome) -> Vec<&mut i64> {
    match income {
        FederalIncome::Rental { data } => {
            let associated = !data.energy_buildings.is_empty()
                || data.buildings.iter().any(|cost| cost.energy.is_some());
            data.energy_buildings
                .iter_mut()
                .map(|b| &mut b.id)
                .chain(
                    data.buildings
                        .iter_mut()
                        .filter(|cost| associated || cost.id != 0)
                        .map(|b| &mut b.id),
                )
                .collect()
        }
        _ => Vec::new(),
    }
}

pub(crate) fn source_issue(v: &FederalRental) -> Option<&'static str> {
    let valid_date = crate::federal_rules::valid_date;
    for building in &v.energy_buildings {
        if building.building_source.trim().is_empty()
            || decimal(&building.square_feet).is_none_or(|area| area <= exact::whole(0))
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
                || decimal(&c.savings_percent)
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
                            && decimal(&cert.savings_percent)
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
            let Some(savings) = decimal(&cert.savings_percent) else {
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
        let Some(area) = decimal(&building.square_feet) else { continue };
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

impl crate::community::Divide for RentalEnergyClaim {
    fn divided(&self, share: crate::community::Share) -> Self {
        Self { qualifying_cost: share.filer(self.qualifying_cost), ..self.clone() }
    }
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
            crate::community::share(setup, entry.owner, entry.community, &entry.income)
        })
    {
        for reduction in &mut prepared.reductions {
            *reduction = share.filer(*reduction);
        }
    }
    prepared
}
