use crate::{
    BusinessCredit, BusinessCreditCarryback, BusinessCreditCarryforward, BusinessCreditHistory,
    CarryFiler, CreditElection, CreditSource, EnteredBusinessCredit, PassiveWorksheet,
};
use alloc::vec::Vec;
use libertas_macros::*;

/// S corporation that paid this W-2
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum WageScorpEmployer {
    /// One of my S corporations
    Corporation {
        /// Corporation's Schedule K-1 on this return
        source: i64,
    },
}
/// S corporation that paid this W-2
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum WageScorpEmployerView {
    /// One of my S corporations
    Corporation {
        /// Corporation's Schedule K-1 on this return
        #[libertas_enum_source("$.wage_scorporations")]
        source: i64,
    },
}
impl From<&WageScorpEmployer> for WageScorpEmployerView {
    fn from(value: &WageScorpEmployer) -> Self {
        match value {
            WageScorpEmployer::Corporation { source } => Self::Corporation { source: *source },
        }
    }
}
impl From<&WageScorpEmployerView> for WageScorpEmployer {
    fn from(value: &WageScorpEmployerView) -> Self {
        match value {
            WageScorpEmployerView::Corporation { source } => Self::Corporation { source: *source },
        }
    }
}
impl WageScorpEmployer {
    pub(crate) fn id(&self) -> i64 {
        match self {
            Self::Corporation { source } => *source,
        }
    }
    #[cfg(test)]
    pub(crate) fn from_id(source: i64) -> Self {
        Self::Corporation { source }
    }
}

/// The business that earned the research credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum ResearchCreditBusiness {
    /// An actual business or eligible K-1 on this return
    Business {
        /// Business whose current taxable income belongs to this historical credit
        /// Select only the same economic business, including an established continuation.
        #[libertas_enum_source("$.research_businesses")]
        source: i64,
    },
}
impl ResearchCreditBusiness {
    fn id(&self) -> i64 {
        match self {
            Self::Business { source } => *source,
        }
    }
    fn from_id(source: i64) -> Self {
        Self::Business { source }
    }
}

/// Where an entered credit comes from
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CreditSourcePage {
    /// A business you run (Schedule C)
    Business {
        /// Business
        /// [NullText]
        /// Use my only Schedule C business
        /// [SetText]
        /// The business is
        #[libertas_enum_source("$.businesses")]
        #[libertas_default($null)]
        business: Option<i64>,
        /// Entered expenses and basis already reflect the credit's reduction
        /// No leaves this credit out of the estimate.
        reduced: bool,
    },
    /// A farm (Schedule F or Form 4835)
    Farm {
        /// Farm
        /// [NullText]
        /// Use my only farm
        /// [SetText]
        /// The farm is
        #[libertas_enum_source("$.farms")]
        #[libertas_default($null)]
        farm: Option<i64>,
        /// Entered expenses and basis already reflect the credit's reduction
        reduced: bool,
    },
    /// A rental property (Schedule E)
    Rental {
        /// Rental
        /// [NullText]
        /// Use my only rental property
        /// [SetText]
        /// The rental property is
        #[libertas_enum_source("$.rentals")]
        #[libertas_default($null)]
        rental: Option<i64>,
        /// Entered expenses and basis already reflect the credit's reduction
        reduced: bool,
    },
    /// Bought for cash under a section 6418 transfer election
    /// Passive credit only. The price is not deductible and this credit cannot be sold again.
    Bought,
}
impl From<CreditSourcePage> for CreditSource {
    fn from(v: CreditSourcePage) -> Self {
        match v {
            CreditSourcePage::Business { business, reduced } => {
                Self::Business { business, reduced }
            }
            CreditSourcePage::Farm { farm, reduced } => Self::Farm { farm, reduced },
            CreditSourcePage::Rental { rental, reduced } => Self::Rental { rental, reduced },
            CreditSourcePage::Bought => Self::Bought,
        }
    }
}
impl From<CreditSource> for CreditSourcePage {
    fn from(v: CreditSource) -> Self {
        match v {
            CreditSource::Business { business, reduced } => Self::Business { business, reduced },
            CreditSource::Farm { farm, reduced } => Self::Farm { farm, reduced },
            CreditSource::Rental { rental, reduced } => Self::Rental { rental, reduced },
            CreditSource::Bought => Self::Bought,
        }
    }
}

/// General business credit figured on its own source form
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("BusinessCreditElectionOptions")]
pub struct EnteredBusinessCreditPage {
    /// Credit and Form 3800 line
    #[libertas_constrained_by("$.entered_credits")]
    #[libertas_calculation_request("BusinessCreditElectionOptions")]
    pub credit: BusinessCredit,
    /// Source of this credit
    pub source: CreditSourcePage,
    /// Whole-dollar amount from the source form, excluding a research payroll-tax election
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
    pub amount: i64,
    /// Available elections
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_credit_elections")]
    #[libertas_output_only]
    pub allowed_elections: Vec<i32>,
    /// Elective payment or transfer
    /// Registration must be completed with the IRS before filing.
    #[libertas_constrained_by("^.allowed_elections")]
    #[libertas_default(None)]
    pub election: CreditElection,
    /// Actual equipment origin for the carbon oxide credit
    /// Source-document facts only; this does not elect payment or determine any other credit.
    /// [NullText]
    /// I do not have the equipment origin details for the extended carryback
    /// [SetText]
    /// The equipment origin details are
    #[libertas_default($null)]
    #[libertas_constrained_by("^.allowed_carbon_oxide_origins")]
    pub carbon_oxide_origin: Option<crate::CarbonOxideCreditOrigin>,
    /// Applicable carbon-capture source detail
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_carbon_oxide_origins")]
    #[libertas_output_only]
    pub allowed_carbon_oxide_origins: Vec<i32>,
    /// Actual facility origin for the specified renewable-electricity credit
    /// Source-document facts only; this does not elect payment or determine a generic renewable-electricity credit.
    /// [NullText]
    /// I do not have the facility origin details for the extended carryback
    /// [SetText]
    /// The facility origin details are
    #[libertas_default($null)]
    #[libertas_constrained_by("^.allowed_renewable_facility_origins")]
    pub renewable_facility_origin: Option<crate::RenewableFacilityCreditOrigin>,
    /// Applicable renewable-facility source detail
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_renewable_facility_origins")]
    #[libertas_output_only]
    pub allowed_renewable_facility_origins: Vec<i32>,
}
impl From<EnteredBusinessCreditPage> for EnteredBusinessCredit {
    fn from(v: EnteredBusinessCreditPage) -> Self {
        Self {
            credit: v.credit,
            source: v.source.into(),
            amount: v.amount,
            carbon_oxide_origin: v.carbon_oxide_origin,
            renewable_facility_origin: v.renewable_facility_origin,
            election: v.election,
        }
    }
}
impl From<EnteredBusinessCredit> for EnteredBusinessCreditPage {
    fn from(v: EnteredBusinessCredit) -> Self {
        Self {
            allowed_carbon_oxide_origins: carbon_oxide_origin_choices(v.credit),
            allowed_renewable_facility_origins: renewable_facility_origin_choices(v.credit),
            credit: v.credit,
            allowed_elections: election_choices(v.credit),
            source: v.source.into(),
            amount: v.amount,
            carbon_oxide_origin: v.carbon_oxide_origin,
            renewable_facility_origin: v.renewable_facility_origin,
            election: v.election,
        }
    }
}

/// Unused credit carried forward to 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("ResearchCreditSourceOptions")]
pub struct BusinessCreditCarryforwardPage {
    /// Credit
    #[libertas_calculation_request("ResearchCreditSourceOptions")]
    pub credit: BusinessCredit,
    /// Tax year the credit arose
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 2006, max = 2025)]
    pub origin_year: u32,
    /// Whole-dollar balance after recapture or adjustment
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
    pub amount: i64,
    /// Prior unallowed passive credit worksheet
    /// Worksheet 1 requires active participation both then and now. Pre-1990 housing uses Worksheet 2.
    /// [NullText]
    /// This is a Form 3800 carryforward, not an unallowed passive credit
    #[libertas_default($null)]
    pub passive: Option<PassiveWorksheet>,
    /// Whose credit
    #[libertas_constrained_by("$.allowed_carry_filers")]
    #[libertas_default(SameFilers)]
    pub filer: CarryFiler,
    /// Available originating-business question
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_research_choices")]
    #[libertas_output_only]
    pub allowed_research_choices: Vec<i32>,
    /// Current business corresponding to this research credit
    /// Choose the actual source or its documented continuation. The null answer keeps the credit but gives it no current research-credit capacity.
    /// [NullText]
    /// This originating business has no corresponding taxable income on this return
    #[libertas_constrained_by("^.allowed_research_choices")]
    pub research_source: Option<ResearchCreditBusiness>,
}
impl From<BusinessCreditCarryforwardPage> for BusinessCreditCarryforward {
    fn from(v: BusinessCreditCarryforwardPage) -> Self {
        Self {
            credit: v.credit,
            origin_year: v.origin_year,
            amount: v.amount,
            passive: v.passive,
            filer: v.filer,
            research_source: v.research_source.as_ref().map(ResearchCreditBusiness::id),
        }
    }
}
impl From<BusinessCreditCarryforward> for BusinessCreditCarryforwardPage {
    fn from(v: BusinessCreditCarryforward) -> Self {
        Self {
            credit: v.credit,
            origin_year: v.origin_year,
            amount: v.amount,
            passive: v.passive,
            filer: v.filer,
            allowed_research_choices: research_choices(v.credit),
            research_source: v.research_source.map(ResearchCreditBusiness::from_id),
        }
    }
}

/// 2026 credit carried back to an earlier return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_calculation_root("ResearchCreditSourceOptions")]
pub struct BusinessCreditCarrybackPage {
    /// Credit
    #[libertas_constrained_by("$.carryback_credits")]
    #[libertas_calculation_request("ResearchCreditSourceOptions")]
    pub credit: BusinessCredit,
    /// Earlier return year
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 2021, max = 2025)]
    #[libertas_default(2025)]
    pub year: u32,
    /// Whole-dollar amount used on that earlier return
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
    pub amount: i64,
    /// Available originating-business question
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_research_choices")]
    #[libertas_output_only]
    pub allowed_research_choices: Vec<i32>,
    /// Current business that generated this research credit
    /// A research carryback must match the unused credit of that actual source.
    /// [NullText]
    /// No corresponding current research-credit source is selected
    #[libertas_constrained_by("^.allowed_research_choices")]
    pub research_source: Option<ResearchCreditBusiness>,
}
impl From<BusinessCreditCarrybackPage> for BusinessCreditCarryback {
    fn from(v: BusinessCreditCarrybackPage) -> Self {
        Self {
            credit: v.credit,
            year: v.year,
            amount: v.amount,
            research_source: v.research_source.as_ref().map(ResearchCreditBusiness::id),
        }
    }
}
impl From<BusinessCreditCarryback> for BusinessCreditCarrybackPage {
    fn from(v: BusinessCreditCarryback) -> Self {
        Self {
            credit: v.credit,
            year: v.year,
            amount: v.amount,
            allowed_research_choices: research_choices(v.credit),
            research_source: v.research_source.map(ResearchCreditBusiness::from_id),
        }
    }
}

/// General business credits (Form 3800)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessCreditHistoryPage {
    /// Credits figured on their own source forms
    #[libertas_unordered]
    /// ----
    /// Credit
    pub credits: Vec<EnteredBusinessCreditPage>,
    /// Credits carried forward to 2026
    #[libertas_unordered]
    /// ----
    /// Carryforward
    pub carryforwards: Vec<BusinessCreditCarryforwardPage>,
    /// 2026 credits carried back to earlier returns
    #[libertas_unordered]
    /// ----
    /// Carryback
    pub carrybacks: Vec<BusinessCreditCarrybackPage>,
    /// Your spouse has no general business credit for 2026
    /// Married filing separately only: includes carryforwards and carrybacks.
    #[libertas_default(false)]
    pub spouse_without_credits: bool,
}
impl From<BusinessCreditHistoryPage> for BusinessCreditHistory {
    fn from(v: BusinessCreditHistoryPage) -> Self {
        Self {
            credits: v.credits.into_iter().map(Into::into).collect(),
            carryforwards: v.carryforwards.into_iter().map(Into::into).collect(),
            carrybacks: v.carrybacks.into_iter().map(Into::into).collect(),
            spouse_without_credits: v.spouse_without_credits,
        }
    }
}
impl From<BusinessCreditHistory> for BusinessCreditHistoryPage {
    fn from(v: BusinessCreditHistory) -> Self {
        Self {
            credits: v.credits.into_iter().map(Into::into).collect(),
            carryforwards: v.carryforwards.into_iter().map(Into::into).collect(),
            carrybacks: v.carrybacks.into_iter().map(Into::into).collect(),
            spouse_without_credits: v.spouse_without_credits,
        }
    }
}

pub(crate) fn research_choices(credit: BusinessCredit) -> Vec<i32> {
    if credit.research() { alloc::vec![0] } else { Vec::new() }
}
pub(crate) fn election_choices(credit: BusinessCredit) -> Vec<i32> {
    credit.election_choices()
}

pub(crate) fn research_eligible(entry: &crate::FederalIncomeEntry) -> bool {
    matches!(&entry.income,
        crate::FederalIncome::Business {data} if matches!(data.activity,crate::BusinessActivity::Ordinary{..}|crate::BusinessActivity::Farming{..}))
        || matches!(
            entry.income,
            crate::FederalIncome::Partnership { .. }
                | crate::FederalIncome::Scorp { .. }
                | crate::FederalIncome::PartnershipActivities { .. }
                | crate::FederalIncome::ScorpActivities { .. }
                | crate::FederalIncome::Rental { .. }
                | crate::FederalIncome::FarmRental { .. }
        )
}
pub(crate) fn research_businesses(d: &crate::FederalDraft) -> Vec<crate::RecordChoice> {
    d.income
        .iter()
        .filter(|e| research_eligible(e))
        .map(|e| crate::RecordChoice {
            id: e.id,
            label: crate::interview::income_issue_context(e).label().encode(),
        })
        .collect()
}

/// Printed K-1 credit code
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum K1CreditCodePage {
    /// Code J WorkOpportunity
    WorkOpportunity,
    /// Code N EmployerTipFica
    EmployerTipFica,
    /// Code AE PensionStartup
    PensionStartup,
    /// Code AF AutoEnrollment
    AutoEnrollment,
    /// Code AG MilitarySpouse
    MilitarySpouse,
    /// Code BA SmallEmployerHealth
    SmallEmployerHealth,
    /// Code BB PaidLeave
    PaidLeave,
    /// Code C LowIncomeHousingJ5
    LowIncomeHousingJ5,
    /// Code D LowIncomeHousingOther
    LowIncomeHousingOther,
    /// Code A ZeroEmissionNuclear
    ZeroEmissionNuclear,
    /// Code B AdvancedNuclear
    AdvancedNuclear,
    /// Code F OtherRentalRealEstate
    OtherRentalRealEstate {
        /// Form 3800 line
        /// Use the Form 3800 line the attached credit statement names.
        #[libertas_constrained_by("$.k1_credit_lines")]
        line: BusinessCredit,
    },
    /// Code G OtherRental
    OtherRental {
        /// Form 3800 line
        /// Use the Form 3800 line the attached credit statement names.
        #[libertas_constrained_by("$.k1_credit_lines")]
        line: BusinessCredit,
    },
    /// Code K DisabledAccess
    DisabledAccess,
    /// Code L EmpowermentZone
    EmpowermentZone,
    /// Code M Research
    Research {
        /// Form 3800 line
        /// Choose line 4i only when both you and the entity meet the eligible-small-business test (average annual gross receipts of $50 million or less for the prior three years); otherwise line 1c. This is your own answer, not a printed K-1 fact.
        #[libertas_constrained_by("$.research_credit_lines")]
        line: BusinessCredit,
    },
    /// Code P CoalGasificationCooperative
    CoalGasificationCooperative,
    /// Code Q AdvancedEnergyCooperative
    AdvancedEnergyCooperative,
    /// Code R ManufacturingInvestmentCooperative
    ManufacturingInvestmentCooperative,
    /// Code S CleanElectricityCooperative
    CleanElectricityCooperative,
    /// Code T EnergyCooperative
    EnergyCooperative,
    /// Code U RehabilitationCooperative
    RehabilitationCooperative,
    /// Code V AdvancedManufacturingProduction
    AdvancedManufacturingProduction,
    /// Code W CleanElectricityProduction
    CleanElectricityProduction,
    /// Code X CleanFuelProduction
    CleanFuelProduction,
    /// Code Y CleanHydrogen
    CleanHydrogen,
    /// Code Z OrphanDrug
    OrphanDrug,
    /// Code AA EnhancedOilRecovery
    EnhancedOilRecovery,
    /// Code AB RenewableElectricity
    RenewableElectricity {
        /// Form 3800 line
        /// Use line 4e only when the statement establishes that the facility is in its first four years; otherwise line 1f.
        #[libertas_constrained_by("$.renewable_credit_lines")]
        line: BusinessCredit,
    },
    /// Code AC Biodiesel
    Biodiesel,
    /// Code AD NewMarkets
    NewMarkets,
    /// Code AH EmployerChildcare
    EmployerChildcare,
    /// Code AI LowSulfurDiesel
    LowSulfurDiesel,
    /// Code AJ RailroadTrack
    RailroadTrack,
    /// Code AK MarginalWells
    MarginalWells,
    /// Code AL DistilledSpirits
    DistilledSpirits,
    /// Code AM EnergyEfficientHome
    EnergyEfficientHome,
    /// Code AO RefuelingProperty
    RefuelingProperty,
    /// Code AV DifferentialWage
    DifferentialWage,
    /// Code AW CarbonOxide
    CarbonOxide,
    /// Code AY CleanVehicle
    CleanVehicle,
    /// Code AZ CommercialCleanVehicle
    CommercialCleanVehicle,
    /// Code BC Transferred
    Transferred {
        /// Form 3800 line
        /// Use the Form 3800 line the attached credit statement names.
        #[libertas_constrained_by("$.transferable_credit_lines")]
        line: BusinessCredit,
    },
    /// Code ZZ Other
    Other {
        /// Form 3800 line
        /// Use the Form 3800 line the attached credit statement names.
        #[libertas_constrained_by("$.k1_credit_lines")]
        line: BusinessCredit,
    },
}
/// Credit reported on Schedule K-1
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1CreditPage {
    /// Printed credit code
    pub code: K1CreditCodePage,
    /// Reported amount
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
    /// Actual equipment origin for the carbon oxide credit
    /// Source-document facts only; this does not elect payment or determine any other credit.
    /// [NullText]
    /// I do not have the equipment origin details for the extended carryback
    /// [SetText]
    /// The equipment origin details are
    #[libertas_default($null)]
    #[libertas_constrained_by("^.allowed_carbon_oxide_origins")]
    #[libertas_hidden]
    pub carbon_oxide_origin: Option<crate::CarbonOxideCreditOrigin>,
    /// Applicable carbon-capture source detail
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_carbon_oxide_origins")]
    #[libertas_output_only]
    pub allowed_carbon_oxide_origins: Vec<i32>,
    /// Actual facility origin for the specified renewable-electricity credit
    /// Source-document facts only; this does not elect payment or determine a generic renewable-electricity credit.
    /// [NullText]
    /// I do not have the facility origin details for the extended carryback
    /// [SetText]
    /// The facility origin details are
    #[libertas_default($null)]
    #[libertas_constrained_by("^.allowed_renewable_facility_origins")]
    #[libertas_hidden]
    pub renewable_facility_origin: Option<crate::RenewableFacilityCreditOrigin>,
    /// Applicable renewable-facility source detail
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_copy_from("$.initial_renewable_facility_origins")]
    #[libertas_output_only]
    pub allowed_renewable_facility_origins: Vec<i32>,
}
impl From<K1CreditPage> for crate::K1Credit {
    fn from(v: K1CreditPage) -> Self {
        let (code, line) = stored_k1_credit_code(v.code);
        Self {
            code,
            line,
            amount: v.amount,
            carbon_oxide_origin: v.carbon_oxide_origin,
            renewable_facility_origin: v.renewable_facility_origin,
        }
    }
}
impl From<crate::K1Credit> for K1CreditPage {
    fn from(v: crate::K1Credit) -> Self {
        let allowed_carbon_oxide_origins = carbon_oxide_origin_choices(v.credit());
        let allowed_renewable_facility_origins = renewable_facility_origin_choices(v.credit());
        let code = match v.code {
            crate::K1CreditCode::WorkOpportunity => K1CreditCodePage::WorkOpportunity,
            crate::K1CreditCode::EmployerTipFica => K1CreditCodePage::EmployerTipFica,
            crate::K1CreditCode::PensionStartup => K1CreditCodePage::PensionStartup,
            crate::K1CreditCode::AutoEnrollment => K1CreditCodePage::AutoEnrollment,
            crate::K1CreditCode::MilitarySpouse => K1CreditCodePage::MilitarySpouse,
            crate::K1CreditCode::SmallEmployerHealth => K1CreditCodePage::SmallEmployerHealth,
            crate::K1CreditCode::PaidLeave => K1CreditCodePage::PaidLeave,
            crate::K1CreditCode::LowIncomeHousingJ5 => K1CreditCodePage::LowIncomeHousingJ5,
            crate::K1CreditCode::LowIncomeHousingOther => K1CreditCodePage::LowIncomeHousingOther,
            crate::K1CreditCode::ZeroEmissionNuclear => K1CreditCodePage::ZeroEmissionNuclear,
            crate::K1CreditCode::AdvancedNuclear => K1CreditCodePage::AdvancedNuclear,
            crate::K1CreditCode::OtherRentalRealEstate => K1CreditCodePage::OtherRentalRealEstate {
                line: v.line.expect("accepted K-1 credit has its required line"),
            },
            crate::K1CreditCode::OtherRental => K1CreditCodePage::OtherRental {
                line: v.line.expect("accepted K-1 credit has its required line"),
            },
            crate::K1CreditCode::DisabledAccess => K1CreditCodePage::DisabledAccess,
            crate::K1CreditCode::EmpowermentZone => K1CreditCodePage::EmpowermentZone,
            crate::K1CreditCode::Research => K1CreditCodePage::Research {
                line: v.line.expect("accepted K-1 credit has its required line"),
            },
            crate::K1CreditCode::CoalGasificationCooperative => {
                K1CreditCodePage::CoalGasificationCooperative
            }
            crate::K1CreditCode::AdvancedEnergyCooperative => {
                K1CreditCodePage::AdvancedEnergyCooperative
            }
            crate::K1CreditCode::ManufacturingInvestmentCooperative => {
                K1CreditCodePage::ManufacturingInvestmentCooperative
            }
            crate::K1CreditCode::CleanElectricityCooperative => {
                K1CreditCodePage::CleanElectricityCooperative
            }
            crate::K1CreditCode::EnergyCooperative => K1CreditCodePage::EnergyCooperative,
            crate::K1CreditCode::RehabilitationCooperative => {
                K1CreditCodePage::RehabilitationCooperative
            }
            crate::K1CreditCode::AdvancedManufacturingProduction => {
                K1CreditCodePage::AdvancedManufacturingProduction
            }
            crate::K1CreditCode::CleanElectricityProduction => {
                K1CreditCodePage::CleanElectricityProduction
            }
            crate::K1CreditCode::CleanFuelProduction => K1CreditCodePage::CleanFuelProduction,
            crate::K1CreditCode::CleanHydrogen => K1CreditCodePage::CleanHydrogen,
            crate::K1CreditCode::OrphanDrug => K1CreditCodePage::OrphanDrug,
            crate::K1CreditCode::EnhancedOilRecovery => K1CreditCodePage::EnhancedOilRecovery,
            crate::K1CreditCode::RenewableElectricity => K1CreditCodePage::RenewableElectricity {
                line: v.line.expect("accepted K-1 credit has its required line"),
            },
            crate::K1CreditCode::Biodiesel => K1CreditCodePage::Biodiesel,
            crate::K1CreditCode::NewMarkets => K1CreditCodePage::NewMarkets,
            crate::K1CreditCode::EmployerChildcare => K1CreditCodePage::EmployerChildcare,
            crate::K1CreditCode::LowSulfurDiesel => K1CreditCodePage::LowSulfurDiesel,
            crate::K1CreditCode::RailroadTrack => K1CreditCodePage::RailroadTrack,
            crate::K1CreditCode::MarginalWells => K1CreditCodePage::MarginalWells,
            crate::K1CreditCode::DistilledSpirits => K1CreditCodePage::DistilledSpirits,
            crate::K1CreditCode::EnergyEfficientHome => K1CreditCodePage::EnergyEfficientHome,
            crate::K1CreditCode::RefuelingProperty => K1CreditCodePage::RefuelingProperty,
            crate::K1CreditCode::DifferentialWage => K1CreditCodePage::DifferentialWage,
            crate::K1CreditCode::CarbonOxide => K1CreditCodePage::CarbonOxide,
            crate::K1CreditCode::CleanVehicle => K1CreditCodePage::CleanVehicle,
            crate::K1CreditCode::CommercialCleanVehicle => K1CreditCodePage::CommercialCleanVehicle,
            crate::K1CreditCode::Transferred => K1CreditCodePage::Transferred {
                line: v.line.expect("accepted K-1 credit has its required line"),
            },
            crate::K1CreditCode::Other => K1CreditCodePage::Other {
                line: v.line.expect("accepted K-1 credit has its required line"),
            },
        };
        Self {
            allowed_carbon_oxide_origins,
            allowed_renewable_facility_origins,
            code,
            amount: v.amount,
            carbon_oxide_origin: v.carbon_oxide_origin,
            renewable_facility_origin: v.renewable_facility_origin,
        }
    }
}

pub(crate) fn k1_credit_lines() -> Vec<i32> {
    BusinessCredit::ALL.iter().filter(|c| c.carryback_allowed()).map(|c| *c as i32).collect()
}
pub(crate) fn research_credit_lines() -> Vec<i32> {
    alloc::vec![BusinessCredit::Research as i32, BusinessCredit::ResearchSmallBusiness as i32]
}
pub(crate) fn renewable_credit_lines() -> Vec<i32> {
    alloc::vec![
        BusinessCredit::RenewableElectricity as i32,
        BusinessCredit::RenewableElectricitySpecified as i32
    ]
}
pub(crate) fn transferable_credit_lines() -> Vec<i32> {
    BusinessCredit::ALL.iter().filter(|c| c.transferable()).map(|c| *c as i32).collect()
}

impl K1CreditCodePage {
    pub(crate) fn housing(&self) -> bool {
        matches!(self, Self::LowIncomeHousingJ5 | Self::LowIncomeHousingOther)
    }
}

pub(crate) fn carbon_oxide_origin_choices(credit: BusinessCredit) -> Vec<i32> {
    if credit == BusinessCredit::CarbonOxide { alloc::vec![0] } else { Vec::new() }
}
pub(crate) fn k1_carbon_oxide_origin_choices(code: K1CreditCodePage) -> Vec<i32> {
    let (code, line) = stored_k1_credit_code(code);
    carbon_oxide_origin_choices(line.unwrap_or_else(|| code.credit()))
}

pub(crate) fn renewable_facility_origin_choices(credit: BusinessCredit) -> Vec<i32> {
    if credit == BusinessCredit::RenewableElectricitySpecified {
        alloc::vec![0]
    } else {
        Vec::new()
    }
}
pub(crate) fn k1_renewable_facility_origin_choices(code: K1CreditCodePage) -> Vec<i32> {
    let (code, line) = stored_k1_credit_code(code);
    renewable_facility_origin_choices(line.unwrap_or_else(|| code.credit()))
}
fn stored_k1_credit_code(code: K1CreditCodePage) -> (crate::K1CreditCode, Option<BusinessCredit>) {
    match code {
        K1CreditCodePage::WorkOpportunity => (crate::K1CreditCode::WorkOpportunity, None),
        K1CreditCodePage::EmployerTipFica => (crate::K1CreditCode::EmployerTipFica, None),
        K1CreditCodePage::PensionStartup => (crate::K1CreditCode::PensionStartup, None),
        K1CreditCodePage::AutoEnrollment => (crate::K1CreditCode::AutoEnrollment, None),
        K1CreditCodePage::MilitarySpouse => (crate::K1CreditCode::MilitarySpouse, None),
        K1CreditCodePage::SmallEmployerHealth => (crate::K1CreditCode::SmallEmployerHealth, None),
        K1CreditCodePage::PaidLeave => (crate::K1CreditCode::PaidLeave, None),
        K1CreditCodePage::LowIncomeHousingJ5 => (crate::K1CreditCode::LowIncomeHousingJ5, None),
        K1CreditCodePage::LowIncomeHousingOther => {
            (crate::K1CreditCode::LowIncomeHousingOther, None)
        }
        K1CreditCodePage::ZeroEmissionNuclear => (crate::K1CreditCode::ZeroEmissionNuclear, None),
        K1CreditCodePage::AdvancedNuclear => (crate::K1CreditCode::AdvancedNuclear, None),
        K1CreditCodePage::OtherRentalRealEstate { line } => {
            (crate::K1CreditCode::OtherRentalRealEstate, Some(line))
        }
        K1CreditCodePage::OtherRental { line } => (crate::K1CreditCode::OtherRental, Some(line)),
        K1CreditCodePage::DisabledAccess => (crate::K1CreditCode::DisabledAccess, None),
        K1CreditCodePage::EmpowermentZone => (crate::K1CreditCode::EmpowermentZone, None),
        K1CreditCodePage::Research { line } => (crate::K1CreditCode::Research, Some(line)),
        K1CreditCodePage::CoalGasificationCooperative => {
            (crate::K1CreditCode::CoalGasificationCooperative, None)
        }
        K1CreditCodePage::AdvancedEnergyCooperative => {
            (crate::K1CreditCode::AdvancedEnergyCooperative, None)
        }
        K1CreditCodePage::ManufacturingInvestmentCooperative => {
            (crate::K1CreditCode::ManufacturingInvestmentCooperative, None)
        }
        K1CreditCodePage::CleanElectricityCooperative => {
            (crate::K1CreditCode::CleanElectricityCooperative, None)
        }
        K1CreditCodePage::EnergyCooperative => (crate::K1CreditCode::EnergyCooperative, None),
        K1CreditCodePage::RehabilitationCooperative => {
            (crate::K1CreditCode::RehabilitationCooperative, None)
        }
        K1CreditCodePage::AdvancedManufacturingProduction => {
            (crate::K1CreditCode::AdvancedManufacturingProduction, None)
        }
        K1CreditCodePage::CleanElectricityProduction => {
            (crate::K1CreditCode::CleanElectricityProduction, None)
        }
        K1CreditCodePage::CleanFuelProduction => (crate::K1CreditCode::CleanFuelProduction, None),
        K1CreditCodePage::CleanHydrogen => (crate::K1CreditCode::CleanHydrogen, None),
        K1CreditCodePage::OrphanDrug => (crate::K1CreditCode::OrphanDrug, None),
        K1CreditCodePage::EnhancedOilRecovery => (crate::K1CreditCode::EnhancedOilRecovery, None),
        K1CreditCodePage::RenewableElectricity { line } => {
            (crate::K1CreditCode::RenewableElectricity, Some(line))
        }
        K1CreditCodePage::Biodiesel => (crate::K1CreditCode::Biodiesel, None),
        K1CreditCodePage::NewMarkets => (crate::K1CreditCode::NewMarkets, None),
        K1CreditCodePage::EmployerChildcare => (crate::K1CreditCode::EmployerChildcare, None),
        K1CreditCodePage::LowSulfurDiesel => (crate::K1CreditCode::LowSulfurDiesel, None),
        K1CreditCodePage::RailroadTrack => (crate::K1CreditCode::RailroadTrack, None),
        K1CreditCodePage::MarginalWells => (crate::K1CreditCode::MarginalWells, None),
        K1CreditCodePage::DistilledSpirits => (crate::K1CreditCode::DistilledSpirits, None),
        K1CreditCodePage::EnergyEfficientHome => (crate::K1CreditCode::EnergyEfficientHome, None),
        K1CreditCodePage::RefuelingProperty => (crate::K1CreditCode::RefuelingProperty, None),
        K1CreditCodePage::DifferentialWage => (crate::K1CreditCode::DifferentialWage, None),
        K1CreditCodePage::CarbonOxide => (crate::K1CreditCode::CarbonOxide, None),
        K1CreditCodePage::CleanVehicle => (crate::K1CreditCode::CleanVehicle, None),
        K1CreditCodePage::CommercialCleanVehicle => {
            (crate::K1CreditCode::CommercialCleanVehicle, None)
        }
        K1CreditCodePage::Transferred { line } => (crate::K1CreditCode::Transferred, Some(line)),
        K1CreditCodePage::Other { line } => (crate::K1CreditCode::Other, Some(line)),
    }
}

/// Accepted entered business credit answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct EnteredBusinessCreditInitial {
    pub credit: BusinessCredit,
    pub source: CreditSourcePage,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub amount: i64,
    pub allowed_elections: Vec<i32>,
    pub election: CreditElection,
    pub carbon_oxide_origin: Option<crate::CarbonOxideCreditOrigin>,
    pub allowed_carbon_oxide_origins: Vec<i32>,
    pub renewable_facility_origin: Option<crate::RenewableFacilityCreditOrigin>,
    pub allowed_renewable_facility_origins: Vec<i32>,
}

impl From<EnteredBusinessCreditPage> for EnteredBusinessCreditInitial {
    fn from(value: EnteredBusinessCreditPage) -> Self {
        Self {
            credit: value.credit,
            source: value.source,
            amount: value.amount,
            allowed_elections: value.allowed_elections,
            election: value.election,
            carbon_oxide_origin: value.carbon_oxide_origin,
            allowed_carbon_oxide_origins: value.allowed_carbon_oxide_origins,
            renewable_facility_origin: value.renewable_facility_origin,
            allowed_renewable_facility_origins: value.allowed_renewable_facility_origins,
        }
    }
}

/// Accepted business credit carryforward answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct BusinessCreditCarryforwardInitial {
    pub credit: BusinessCredit,
    pub origin_year: u32,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub amount: i64,
    pub passive: Option<PassiveWorksheet>,
    pub filer: CarryFiler,
    pub allowed_research_choices: Vec<i32>,
    pub research_source: Option<ResearchCreditBusiness>,
}

impl From<BusinessCreditCarryforwardPage> for BusinessCreditCarryforwardInitial {
    fn from(value: BusinessCreditCarryforwardPage) -> Self {
        Self {
            credit: value.credit,
            origin_year: value.origin_year,
            amount: value.amount,
            passive: value.passive,
            filer: value.filer,
            allowed_research_choices: value.allowed_research_choices,
            research_source: value.research_source,
        }
    }
}

/// Accepted business credit carryback answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct BusinessCreditCarrybackInitial {
    pub credit: BusinessCredit,
    pub year: u32,
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = -100000000000, max = 100000000000)]
    pub amount: i64,
    pub allowed_research_choices: Vec<i32>,
    pub research_source: Option<ResearchCreditBusiness>,
}

impl From<BusinessCreditCarrybackPage> for BusinessCreditCarrybackInitial {
    fn from(value: BusinessCreditCarrybackPage) -> Self {
        Self {
            credit: value.credit,
            year: value.year,
            amount: value.amount,
            allowed_research_choices: value.allowed_research_choices,
            research_source: value.research_source,
        }
    }
}

/// Accepted business credit history answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct BusinessCreditHistoryInitial {
    pub credits: Vec<EnteredBusinessCreditInitial>,
    pub carryforwards: Vec<BusinessCreditCarryforwardInitial>,
    pub carrybacks: Vec<BusinessCreditCarrybackInitial>,
    pub spouse_without_credits: bool,
}

impl From<BusinessCreditHistoryPage> for BusinessCreditHistoryInitial {
    fn from(value: BusinessCreditHistoryPage) -> Self {
        Self {
            credits: value.credits.into_iter().map(Into::into).collect(),
            carryforwards: value.carryforwards.into_iter().map(Into::into).collect(),
            carrybacks: value.carrybacks.into_iter().map(Into::into).collect(),
            spouse_without_credits: value.spouse_without_credits,
        }
    }
}
