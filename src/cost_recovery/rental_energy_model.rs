use crate::SourceCents;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// A physical building's section 179D records
/// Enter each actual whole building once, even when several cost rows belong to it.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one actual physical building's section 179D certification or conditioned-area schedule exactly. Return only the printed building designation and conditioned whole-building square footage, retaining the printed decimal precision. Report missing, illegible, conflicting or multiple-building values as errors. Do not calculate area, ownership shares or a deduction. Do not change record IDs, cost-source associations, ownership fractions or current/prior deduction histories; those remain actual personal records.",
    doc_type = "application/pdf,image/*"
)]
pub struct RentalEnergyBuildingFacts {
    /// Building recovery facts, separate from section179D records
    /// [NullText]
    /// Records unavailable
    /// [SetText]
    /// Building recovery facts
    pub recovery: Option<crate::BuildingUseSource>,
    /// Record identity
    #[libertas_hidden]
    #[libertas_default(0)]
    pub id: i64,
    /// Building designation on the certification and ownership records
    #[libertas_size(min = 1)]
    #[libertas_llm_doc_field]
    pub building_source: String,
    /// Conditioned building area
    /// Copy the documented whole-building area. Do not enter just your ownership share.
    #[libertas_number(min = 0.01, max = 1000000000, step = 0.01)]
    #[libertas_physical_unit("square-foot")]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub square_feet: f64,
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
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for RentalEnergyBuildingFacts {}

/// A physical building's section 179D records
/// Enter each actual whole building once, even when several cost rows belong to it.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_llm_doc_trans(
    instruction = "Read one actual physical building's section 179D certification or conditioned-area schedule exactly. Return only the printed building designation and conditioned whole-building square footage, retaining the printed decimal precision. Report missing, illegible, conflicting or multiple-building values as errors. Do not calculate area, ownership shares or a deduction. Do not change record IDs, cost-source associations, ownership fractions or current/prior deduction histories; those remain actual personal records.",
    doc_type = "application/pdf,image/*"
)]
pub struct RentalEnergyBuildingFactsPage {
    /// Building recovery facts, separate from section179D records
    /// [NullText]
    /// Records unavailable
    /// [SetText]
    /// Building recovery facts
    #[libertas_constrained_by("^.allowed_recovery")]
    pub recovery: Option<BuildingRecoveryRecord>,
    /// Applicable building recovery record
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_asset_amt_carry_choices")]
    pub allowed_recovery: Vec<i32>,
    /// Record identity
    #[libertas_hidden]
    #[libertas_default(0)]
    #[libertas_read_only]
    pub id: i64,
    /// Building designation on the certification and ownership records
    #[libertas_size(min = 1)]
    #[libertas_llm_doc_field]
    pub building_source: String,
    /// Conditioned building area
    /// Copy the documented whole-building area. Do not enter just your ownership share.
    #[libertas_number(min = 0.01, max = 1000000000, step = 0.01)]
    #[libertas_physical_unit("square-foot")]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub square_feet: f64,
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
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for RentalEnergyBuildingFactsPage {}

/// Actual building recovery facts, offered only where an improvement needs them
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum BuildingRecoveryRecord {
    /// Actual building recovery facts
    Record { facts: crate::BuildingUseSource },
}
impl From<RentalEnergyBuildingFacts> for RentalEnergyBuildingFactsPage {
    fn from(value: RentalEnergyBuildingFacts) -> Self {
        Self {
            allowed_recovery: if value.recovery.is_some() { alloc::vec![0] } else { Vec::new() },
            recovery: value.recovery.map(|facts| BuildingRecoveryRecord::Record { facts }),
            id: value.id,
            building_source: value.building_source,
            square_feet: value.square_feet,
            ownership_numerator: value.ownership_numerator,
            ownership_denominator: value.ownership_denominator,
            deductions: value.deductions,
        }
    }
}
impl From<RentalEnergyBuildingFactsPage> for RentalEnergyBuildingFacts {
    fn from(value: RentalEnergyBuildingFactsPage) -> Self {
        Self {
            recovery: value.recovery.map(|BuildingRecoveryRecord::Record { facts }| facts),
            id: value.id,
            building_source: value.building_source,
            square_feet: value.square_feet,
            ownership_numerator: value.ownership_numerator,
            ownership_denominator: value.ownership_denominator,
            deductions: value.deductions,
        }
    }
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
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
    /// Printed savings
    /// Copy the printed value; do not calculate savings yourself.
    #[libertas_number(min = 0, max = 100, step = 0.01)]
    #[libertas_physical_unit("percent")]
    #[libertas_llm_doc_field(allow_empty = false)]
    pub savings_percent: f64,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for RentalEnergyCertification {}

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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
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

impl crate::community::Divide for RentalEnergyClaim {
    fn divided(&self, share: crate::community::Share) -> Self {
        Self { qualifying_cost: share.filer(self.qualifying_cost), ..self.clone() }
    }
}
