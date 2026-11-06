use crate::filer_facts::FederalStatus;
use crate::validation::ValidationIssue;
use alloc::vec::Vec;
use core::cmp::max;
use libertas_macros::*;

/// 2025 employer tip FICA credit carried to the 2026 return
/// This is a single 2025-origin Form 8846 credit from an active, directly owned employer business. Retain the 2025 Form 8846 and Form 3800 Part III line 4f, the 2025 tax-use worksheet, and any 2024 carryback return. A 2025-origin credit is not reported in 2025 Form 3800 Part IV; it first appears in 2026 Part IV line 4f.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployerTipFicaCarryforward {
    /// Filing status on the 2025 credit return
    pub prior_status: FederalStatus,
    /// Whole-dollar credit reported on 2025 Form 3800 Part III line 4f, drawn from Form 8846 line 6
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub original_credit: i64,
    /// 2025 Form 3800 credit applied against 2025 tax
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub used_origin_year: i64,
    /// Credit applied on a 2024 carryback return
    /// Enter zero only after confirming no 2024 tax capacity or another valid reason the carryback did not use credit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub used_origin_prior_year_carryback: i64,
    /// Balance documented for 2026 Form 3800 Part IV line 4f column (f)
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub carryforward_to_current_year: i64,
    /// Confirm the narrow carryforward history
    /// The same filer or joint filers own this 2025 credit. It came directly from one active employer business (Form 8846 line 5 is zero), is not passive, was not transferred, recaptured, disallowed or adjusted, and all 2024 carryback use is final. No other general business credit or credit carryover is unentered. The 2025 Forms 8846 and 3800 (Part III carryforward lines) show it. No leaves this carryforward out of this estimate.
    pub ordinary: bool,
}
impl EmployerTipFicaCarryforward {
    pub(crate) fn valid(&self) -> bool {
        [
            self.original_credit,
            self.used_origin_year,
            self.used_origin_prior_year_carryback,
            self.carryforward_to_current_year,
        ]
        .iter()
        .all(|value| (0..=100000000000).contains(value) && value % 100 == 0)
    }
}
/// General business credit
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum BusinessCredit {
    /// Work opportunity credit (Form 5884)
    WorkOpportunity,
    /// Employer Social Security and Medicare taxes paid on tips (Form 8846)
    EmployerTipFica,
    /// Small employer pension plan start-up costs and contributions credit (Form 8881 Part I)
    PensionStartup,
    /// Alternative motor vehicle credit, business part (Form 8910)
    /// Carryforward only: the credit ended for vehicles bought after 2021. The personal part of the credit never carried forward.
    AlternativeMotorVehicle,
    /// Small employer health insurance premiums credit (Form 8941)
    SmallEmployerHealth,
    /// Small employer auto-enrollment credit (Form 8881 Part II)
    AutoEnrollment,
    /// Small employer military spouse participation credit (Form 8881 Part III)
    MilitarySpouse,
    /// Employer credit for paid family and medical leave (Form 8994)
    PaidLeave,
    /// Low-income housing credit (Form 8586)
    LowIncomeHousing,
    /// Low-income housing credit from buildings placed in service before 2008 (Form 8586)
    /// Carryforward only: the credit periods of those buildings have ended.
    LowIncomeHousingBefore2008,
    /// Rehabilitation credit (Form 3468 Part VII)
    /// The 20% credit for a certified historic building's rehabilitation, allowed 4% a year for five years.
    Rehabilitation,
    /// Energy credit (Form 3468 Part VI)
    /// Section 48: solar, small wind, energy storage and geothermal heat pump property whose construction began before 2025 (geothermal heat pumps through 2034).
    Energy,
    /// Clean electricity investment credit (Form 3468 Part V)
    /// Section 48E: solar, wind and energy storage facilities whose construction began after 2024.
    CleanElectricity,
    /// Clean fuel production credit allocated by an actual cooperative notice (Form 7218)
    /// This source-only patron allocation does not calculate a producer's credit.
    CleanFuel,
    /// Advanced coal project or gasification project credit (Form 3468 Part II, line 1a)
    AdvancedCoalGasification,
    /// Advanced energy project credit (Form 3468 Part III, line 1d)
    /// Section 48C, for a project with an allocation.
    AdvancedEnergyProject,
    /// Qualifying therapeutic discovery project credit (Form 8942, line 2w)
    /// Carryforward only.
    TherapeuticDiscovery,
    /// Advanced manufacturing investment credit (Form 3468 Part IV, line 1o)
    /// Section 48D semiconductor property.
    AdvancedManufacturingInvestment,
    /// Biofuel producer credit (Form 6478, line 4c)
    /// Carryforward only: the credit ended after 2024.
    BiofuelProducer,
    /// Credit for increasing research activities (Form 6765, line 1c)
    /// When you are not an eligible small business (average gross receipts above $50 million for the 3 years before), or a partner or shareholder who is not.
    Research,
    /// Research credit of an eligible small business (Form 6765, line 4i)
    /// When your business (and, for a K-1, both you and the entity) had average annual gross receipts of $50 million or less for the 3 years before. It can offset the alternative minimum tax.
    ResearchSmallBusiness,
    /// Enhanced oil recovery credit (Form 8830, line 1t)
    EnhancedOilRecovery,
    /// Disabled access credit (Form 8826, line 1e)
    /// Form 3800 takes at most $5,000 of it from all sources together.
    DisabledAccess,
    /// Renewable electricity production credit (Form 8835 Part II, line 1f)
    RenewableElectricity,
    /// Renewable electricity production credit in a facility's first four years (Form 8835 Part II, line 4e)
    /// For a facility placed in service after October 22, 2004, during its first four years of service. It can offset the alternative minimum tax.
    RenewableElectricitySpecified,
    /// Empowerment zone employment credit (Form 8844, line 3)
    /// Empowerment zones ended December 31, 2025: a 2026 amount comes only from a K-1 of an entity whose tax year began in 2025.
    EmpowermentZone,
    /// Renewal community employment credit (line 3)
    /// Carryforward only.
    RenewalCommunity,
    /// Indian employment credit (Form 8845, line 2c)
    /// Carryforward only.
    IndianEmployment,
    /// Orphan drug credit (Form 8820, line 1h)
    OrphanDrug,
    /// New markets credit (Form 8874, line 1i)
    NewMarkets,
    /// Employer-provided childcare facilities and services credit (Form 8882, line 1k)
    /// Form 3800 takes at most $500,000 of it from all sources together.
    EmployerChildcare,
    /// Railroad track maintenance credit (Form 8900, line 4g)
    /// Amounts from before 2008 are line 2m and cannot offset the alternative minimum tax.
    RailroadTrack,
    /// Biodiesel, renewable diesel and small agri-biodiesel producer credit (Form 8864, line 1l)
    Biodiesel,
    /// Low sulfur diesel fuel production credit (Form 8896, line 1m)
    LowSulfurDiesel,
    /// Credit for oil and gas production from marginal wells (Form 8904, line 1bb)
    /// A 2026 amount can be carried back five years.
    MarginalWells,
    /// Distilled spirits credit (Form 8906, line 1n)
    DistilledSpirits,
    /// Advanced nuclear power facility production credit (Form 7213 Part I, line 1cc)
    AdvancedNuclear,
    /// Nonconventional source fuel credit (Form 8907, line 2d)
    /// Carryforward only.
    NonconventionalFuel,
    /// Energy efficient home credit (Form 8908, line 1p)
    EnergyEfficientHome,
    /// Energy efficient appliance credit (Form 8909, line 2e)
    /// Carryforward only.
    EnergyEfficientAppliance,
    /// Alternative fuel vehicle refueling property credit, business part (Form 8911 Part I, line 1s)
    RefuelingProperty,
    /// Mine rescue team training credit (Form 8923, line 2f)
    /// Carryforward only.
    MineRescue,
    /// Agricultural chemicals security credit (Form 8931, line 2h)
    /// Carryforward only.
    AgriculturalChemicals,
    /// Credit for employer differential wage payments (Form 8932, line 1w)
    DifferentialWage,
    /// Carbon oxide sequestration credit (Form 8933, line 1x)
    CarbonOxide,
    /// Clean vehicle credit, business part (Form 8936 Part II, line 1y)
    /// Also the earlier qualified plug-in electric drive vehicle credit's business part.
    /// To use 2025 modified AGI for this credit's income limit, enter it under Clean vehicles and refueling property. Do not add a personal vehicle row for a completed business-form amount entered here.
    CleanVehicle,
    /// Employee retention credit for employers affected by qualified disasters (Form 5884-A, line 2a)
    /// Carryforward only.
    DisasterRetention,
    /// Employee retention credit, Hurricanes Katrina, Rita or Wilma (Form 5884-A Section A, line 2o)
    /// Carryforward only.
    HurricaneRetention,
    /// Employer housing credit, Hurricane Katrina (Form 5884-A Section B, line 2p)
    /// Carryforward only.
    HurricaneHousing,
    /// Employee retention credit, Midwestern disaster areas (Form 5884-A Section A, line 2q)
    /// Carryforward only.
    MidwesternRetention,
    /// Employer housing credit, Midwestern disaster areas (Form 5884-A Section B, line 2r)
    /// Carryforward only.
    MidwesternHousing,
    /// Zero-emission nuclear power production credit (Form 7213 Part II, line 1u)
    ZeroEmissionNuclear,
    /// Sustainable aviation fuel credit (Form 8864, line 1ff)
    /// Carryforward only: the credit ended after 2024.
    SustainableAviationFuel,
    /// Clean hydrogen production credit (Form 7210, line 1g)
    CleanHydrogen,
    /// Qualified commercial clean vehicle credit (Form 8936 Part V, line 1aa)
    CommercialCleanVehicle,
    /// Advanced manufacturing production credit (Form 7207, line 1b)
    AdvancedManufacturingProduction,
    /// Clean electricity production credit (Form 7211 Part II, line 1gg)
    CleanElectricityProduction,
    /// General credits from an electing large partnership (Form 1065-B, line 2i)
    /// Carryforward only.
    ElectingLargePartnership,
    /// Qualified electric vehicle credit, business part (Form 8834, line 2g)
    /// Carryforward only: 2006 amounts are in their last year.
    QualifiedElectricVehicle,
    /// New hire retention credit (Form 5884-B, line 2s)
    /// Carryforward only.
    NewHireRetention,
    /// Credit for contributions to community development corporations (Form 8847, line 2t)
    /// Carryforward only.
    CommunityDevelopment,
    /// Welfare-to-work credit (Form 8861, line 2u)
    /// Carryforward only.
    WelfareToWork,
    /// Another expired credit (line 2zz)
    /// Carryforward only. Keep a statement with the credit's name and form number.
    ExpiredOther,
    /// 2010 credit of an eligible small business (line 4y)
    /// Carryforward only: a 2010 credit treated as a specified credit (Public Law 111-240).
    SmallBusiness2010,
    /// Another general business credit (line 1zz)
    /// A credit with no line of its own on Form 3800 Part III.
    Other,
    /// Another specified credit (line 4z)
    OtherSpecified,
}
/// Form 8582-CR worksheet that limits a passive activity credit
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum PassiveWorksheet {
    /// Worksheet 3: low-income housing credit
    /// For property placed in service after 1989, or a pass-through interest bought after 1989. Earlier property or interests use Worksheet 2 and Part III.
    LowIncomeHousing,
    /// Worksheet 4: any other passive activity credit
    /// Every passive credit that is not a rental real estate, rehabilitation or low-income housing credit.
    Other,
    /// Worksheet 2: rehabilitation credit from a rental real estate activity
    /// For a rehabilitation credit of a rental real estate activity, with or without active participation: up to $25,000 of deduction equivalent, reduced by half of your income above $200,000 (Form 8582-CR Part III). A rehabilitation credit of any other passive activity uses Worksheet 4.
    Rehabilitation,
    /// Worksheet 1: credit of rental real estate with active participation
    /// Active participation is required both in the credit year and in 2026 for a prior unallowed credit.
    RentalActive,
    /// Worksheet 2: housing property or pass-through interest before 1990
    LowIncomeHousingBefore1990,
}
#[derive(Clone, Copy)]
enum CarrybackOrigin {
    FirstAllowedYear,
    // Section39(a)(4) applies notwithstanding the origin-year rule in39(d).
    ApplicableCreditException,
}
/// Everything fixed about one general business credit, kept in one place:
/// `BusinessCredit::info` has one arm per credit, and the accessors of
/// `BusinessCredit` read it.
pub(crate) struct CreditInfo {
    /// Id key of the calculation rows (`business_credit_<key>_<year>`).
    key: &'static str,
    /// Form name (shown by the tests; the page shows `form_key`).
    #[cfg_attr(not(test), allow(dead_code))]
    form: &'static str,
    /// Resource key of the form name.
    form_key: &'static str,
    /// Form 3800 line: Part III for a 2026 credit, Part IV for a carryforward.
    line: &'static str,
    /// The line of an amount determined before 2007, when it differs.
    early_line: Option<(&'static str, u32)>,
    /// Section 38(c)(4)(B): the first tax year the credit is a specified
    /// credit (0: always); None: never.
    specified_from: Option<u32>,
    /// The first tax year the credit could arise; the 20-year carryforward
    /// reaches 2026 from 2006 at the earliest.
    first_year: u32,
    /// The earliest year a 2026 amount may be carried back to: one year for
    /// an ordinary credit (section 39(a)(1)), three for the energy and clean
    /// electricity investment credits, which are applicable credits of
    /// section 6417(b) (section 39(a)(4); Form 3800 instructions). None: the
    /// credit holds carryforwards only (Form 8910 expired for 2026 property,
    /// and the pre-2008 buildings' low-income housing credit has no 2026
    /// amount); the client rule BUSINESS_CREDIT_CARRYFORWARD_ONLY is the twin.
    carryback_from: Option<u32>,
    // Internal outgoing-period policy, never a taxpayer answer or stored fact.
    // Current and retained-credit origin-year validation remains separate.
    carryback_origin: CarrybackOrigin,
    pub(crate) section_b: bool,
    pub(crate) cap_cents: Option<i64>,
    elective_payment: bool,
    pub(crate) transferable: bool,
    research: bool,
    pub(crate) magi_limited: bool,
}
const CREDIT_BASE: CreditInfo = CreditInfo {
    key: "",
    form: "",
    form_key: "",
    line: "",
    early_line: None,
    specified_from: None,
    first_year: 2006,
    carryback_from: Some(2025),
    carryback_origin: CarrybackOrigin::FirstAllowedYear,
    section_b: false,
    cap_cents: None,
    elective_payment: false,
    transferable: false,
    research: false,
    magi_limited: false,
};
impl BusinessCredit {
    // The instructions' "order in which credits are used" within one year.
    pub(crate) const ALL: [Self; 66] = [
        Self::Rehabilitation,
        Self::Energy,
        Self::AdvancedCoalGasification,
        Self::AdvancedEnergyProject,
        Self::TherapeuticDiscovery,
        Self::AdvancedManufacturingInvestment,
        Self::CleanElectricity,
        Self::WorkOpportunity,
        Self::BiofuelProducer,
        Self::Research,
        Self::LowIncomeHousingBefore2008,
        Self::LowIncomeHousing,
        Self::EnhancedOilRecovery,
        Self::DisabledAccess,
        Self::RenewableElectricity,
        Self::RenewableElectricitySpecified,
        Self::EmpowermentZone,
        Self::RenewalCommunity,
        Self::IndianEmployment,
        Self::EmployerTipFica,
        Self::OrphanDrug,
        Self::NewMarkets,
        Self::PensionStartup,
        Self::EmployerChildcare,
        Self::RailroadTrack,
        Self::Biodiesel,
        Self::LowSulfurDiesel,
        Self::MarginalWells,
        Self::DistilledSpirits,
        Self::AdvancedNuclear,
        Self::NonconventionalFuel,
        Self::EnergyEfficientHome,
        Self::EnergyEfficientAppliance,
        Self::AlternativeMotorVehicle,
        Self::RefuelingProperty,
        Self::MineRescue,
        Self::AgriculturalChemicals,
        Self::DifferentialWage,
        Self::CarbonOxide,
        Self::CleanVehicle,
        Self::SmallEmployerHealth,
        Self::DisasterRetention,
        Self::HurricaneRetention,
        Self::HurricaneHousing,
        Self::MidwesternRetention,
        Self::MidwesternHousing,
        Self::PaidLeave,
        Self::AutoEnrollment,
        Self::ZeroEmissionNuclear,
        Self::SustainableAviationFuel,
        Self::CleanHydrogen,
        Self::CommercialCleanVehicle,
        Self::AdvancedManufacturingProduction,
        Self::CleanElectricityProduction,
        Self::CleanFuel,
        Self::MilitarySpouse,
        Self::ElectingLargePartnership,
        Self::QualifiedElectricVehicle,
        Self::NewHireRetention,
        Self::CommunityDevelopment,
        Self::WelfareToWork,
        Self::ExpiredOther,
        Self::SmallBusiness2010,
        Self::ResearchSmallBusiness,
        Self::Other,
        Self::OtherSpecified,
    ];

    // The one descriptor of a credit.
    pub(crate) fn info(self) -> &'static CreditInfo {
        match self {
            // Specified only when determined in tax years after 2006.
            Self::WorkOpportunity => &CreditInfo {
                key: "work_opportunity",
                form: "Form 5884",
                form_key: "BUSINESS_CREDIT_FORM_WORK_OPPORTUNITY",
                line: "4b",
                early_line: Some(("2j", 2007)),
                specified_from: Some(2007),
                ..CREDIT_BASE
            },
            Self::EmployerTipFica => &CreditInfo {
                key: "tip_fica",
                form: "Form 8846",
                form_key: "BUSINESS_CREDIT_FORM_EMPLOYER_TIP_FICA",
                line: "4f",
                early_line: Some(("2l", 2007)),
                specified_from: Some(2007),
                ..CREDIT_BASE
            },
            Self::PensionStartup => &CreditInfo {
                key: "pension_startup",
                form: "Form 8881 Part I",
                form_key: "BUSINESS_CREDIT_FORM_PENSION_STARTUP",
                line: "1j",
                ..CREDIT_BASE
            },
            // Part IV only: the credit is expired, carryforward only. Section
            // 30B applies to property placed in service after 2005.
            Self::AlternativeMotorVehicle => &CreditInfo {
                key: "alternative_motor_vehicle",
                form: "Form 8910",
                form_key: "BUSINESS_CREDIT_FORM_ALTERNATIVE_MOTOR_VEHICLE",
                line: "2x",
                carryback_from: None,
                ..CREDIT_BASE
            },
            // Always specified.
            Self::SmallEmployerHealth => &CreditInfo {
                key: "small_employer_health",
                form: "Form 8941",
                form_key: "BUSINESS_CREDIT_FORM_SMALL_EMPLOYER_HEALTH",
                line: "4h",
                specified_from: Some(0),
                first_year: 2010,
                ..CREDIT_BASE
            },
            // Always specified. Section 45S applies to wages paid in taxable
            // years beginning after 2017.
            Self::PaidLeave => &CreditInfo {
                key: "paid_leave",
                form: "Form 8994",
                form_key: "BUSINESS_CREDIT_FORM_PAID_LEAVE",
                line: "4j",
                specified_from: Some(0),
                first_year: 2018,
                ..CREDIT_BASE
            },
            Self::AutoEnrollment => &CreditInfo {
                key: "auto_enrollment",
                form: "Form 8881 Part II",
                form_key: "BUSINESS_CREDIT_FORM_AUTO_ENROLLMENT",
                line: "1dd",
                first_year: 2020,
                ..CREDIT_BASE
            },
            Self::MilitarySpouse => &CreditInfo {
                key: "military_spouse",
                form: "Form 8881 Part III",
                form_key: "BUSINESS_CREDIT_FORM_MILITARY_SPOUSE",
                line: "1ee",
                first_year: 2023,
                ..CREDIT_BASE
            },
            // Specified for buildings placed in service after 2007 (section
            // 38(c)(4)(B)(iii)).
            Self::LowIncomeHousing => &CreditInfo {
                key: "low_income_housing",
                form: "Form 8586",
                form_key: "BUSINESS_CREDIT_FORM_LOW_INCOME_HOUSING",
                line: "4d",
                specified_from: Some(0),
                first_year: 2008,
                ..CREDIT_BASE
            },
            // Part IV only: pre-2008 buildings, carryforward only; their
            // carryforward is not specified (Form 3800 Part IV line 2b).
            Self::LowIncomeHousingBefore2008 => &CreditInfo {
                key: "low_income_housing_pre2008",
                form: "Form 8586 (pre-2008)",
                form_key: "BUSINESS_CREDIT_FORM_LOW_INCOME_HOUSING_PRE_2008",
                line: "2b",
                carryback_from: None,
                ..CREDIT_BASE
            },
            // Specified for expenditures after 2007 (section 38(c)(4)(B)(xi)).
            Self::Rehabilitation => &CreditInfo {
                key: "rehabilitation",
                form: "Form 3468 Part VII",
                form_key: "BUSINESS_CREDIT_FORM_REHABILITATION",
                line: "4k",
                specified_from: Some(2008),
                first_year: 2008,
                ..CREDIT_BASE
            },
            // Specified (section 38(c)(4)(B)(x)).
            Self::Energy => &CreditInfo {
                key: "energy",
                form: "Form 3468 Part VI",
                form_key: "BUSINESS_CREDIT_FORM_ENERGY",
                line: "4a",
                specified_from: Some(0),
                carryback_from: Some(2023),
                transferable: true,
                ..CREDIT_BASE
            },
            // Not specified. Section 48E applies to facilities placed in
            // service after 2024.
            Self::CleanFuel => &CreditInfo {
                key: "clean_fuel",
                form: "Form 7218",
                form_key: "BUSINESS_CREDIT_FORM_CLEAN_FUEL",
                line: "1q",
                first_year: 2025,
                carryback_from: Some(2023),
                carryback_origin: CarrybackOrigin::ApplicableCreditException,
                transferable: true,
                ..CREDIT_BASE
            },
            Self::CleanElectricity => &CreditInfo {
                key: "clean_electricity",
                form: "Form 3468 Part V",
                form_key: "BUSINESS_CREDIT_FORM_CLEAN_ELECTRICITY",
                line: "1v",
                first_year: 2025,
                carryback_from: Some(2023),
                carryback_origin: CarrybackOrigin::ApplicableCreditException,
                transferable: true,
                ..CREDIT_BASE
            },
            Self::AdvancedCoalGasification => &CreditInfo {
                key: "advanced_coal_gasification",
                form: "Form 3468 Part II",
                form_key: "BUSINESS_CREDIT_FORM_ADVANCED_COAL_GASIFICATION",
                line: "1a",
                first_year: 2006,
                carryback_from: Some(2025),
                ..CREDIT_BASE
            },
            Self::AdvancedEnergyProject => &CreditInfo {
                key: "advanced_energy_project",
                form: "Form 3468 Part III",
                form_key: "BUSINESS_CREDIT_FORM_ADVANCED_ENERGY_PROJECT",
                line: "1d",
                first_year: 2009,
                carryback_from: Some(2023),
                transferable: true,
                ..CREDIT_BASE
            },
            Self::TherapeuticDiscovery => &CreditInfo {
                key: "therapeutic_discovery",
                form: "Form 8942",
                form_key: "BUSINESS_CREDIT_FORM_THERAPEUTIC_DISCOVERY",
                line: "2w",
                first_year: 2009,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::AdvancedManufacturingInvestment => &CreditInfo {
                key: "advanced_manufacturing_investment",
                form: "Form 3468 Part IV",
                form_key: "BUSINESS_CREDIT_FORM_ADVANCED_MANUFACTURING_INVESTMENT",
                line: "1o",
                first_year: 2023,
                carryback_from: Some(2025),
                elective_payment: true,
                ..CREDIT_BASE
            },
            Self::BiofuelProducer => &CreditInfo {
                key: "biofuel_producer",
                form: "Form 6478",
                form_key: "BUSINESS_CREDIT_FORM_BIOFUEL_PRODUCER",
                line: "4c",
                first_year: 2006,
                carryback_from: None,
                specified_from: Some(0),
                ..CREDIT_BASE
            },
            Self::Research => &CreditInfo {
                key: "research",
                form: "Form 6765",
                form_key: "BUSINESS_CREDIT_FORM_RESEARCH",
                line: "1c",
                first_year: 2006,
                carryback_from: Some(2025),
                research: true,
                ..CREDIT_BASE
            },
            Self::ResearchSmallBusiness => &CreditInfo {
                key: "research_small_business",
                form: "Form 6765",
                form_key: "BUSINESS_CREDIT_FORM_RESEARCH_SMALL_BUSINESS",
                line: "4i",
                first_year: 2016,
                carryback_from: Some(2025),
                research: true,
                specified_from: Some(0),
                ..CREDIT_BASE
            },
            Self::EnhancedOilRecovery => &CreditInfo {
                key: "enhanced_oil_recovery",
                form: "Form 8830",
                form_key: "BUSINESS_CREDIT_FORM_ENHANCED_OIL_RECOVERY",
                line: "1t",
                first_year: 2006,
                carryback_from: Some(2025),
                ..CREDIT_BASE
            },
            Self::DisabledAccess => &CreditInfo {
                key: "disabled_access",
                form: "Form 8826",
                form_key: "BUSINESS_CREDIT_FORM_DISABLED_ACCESS",
                line: "1e",
                first_year: 2006,
                carryback_from: Some(2025),
                cap_cents: Some(500000),
                ..CREDIT_BASE
            },
            Self::RenewableElectricity => &CreditInfo {
                key: "renewable_electricity",
                form: "Form 8835 Part II",
                form_key: "BUSINESS_CREDIT_FORM_RENEWABLE_ELECTRICITY",
                line: "1f",
                first_year: 2006,
                // For 2026, post-2022 facilities are still within the first
                // four production years and belong on line4e. Generic line1f
                // therefore uses the ordinary section39 one-year window.
                carryback_from: Some(2025),
                transferable: true,
                ..CREDIT_BASE
            },
            Self::RenewableElectricitySpecified => &CreditInfo {
                key: "renewable_electricity_specified",
                form: "Form 8835 Part II",
                form_key: "BUSINESS_CREDIT_FORM_RENEWABLE_ELECTRICITY_SPECIFIED",
                line: "4e",
                first_year: 2006,
                carryback_from: Some(2023),
                specified_from: Some(0),
                transferable: true,
                ..CREDIT_BASE
            },
            Self::EmpowermentZone => &CreditInfo {
                key: "empowerment_zone",
                form: "Form 8844",
                form_key: "BUSINESS_CREDIT_FORM_EMPOWERMENT_ZONE",
                line: "3",
                first_year: 2006,
                carryback_from: Some(2025),
                section_b: true,
                ..CREDIT_BASE
            },
            Self::RenewalCommunity => &CreditInfo {
                key: "renewal_community",
                form: "Renewal community employment credit",
                form_key: "BUSINESS_CREDIT_FORM_RENEWAL_COMMUNITY",
                line: "3",
                first_year: 2006,
                carryback_from: None,
                section_b: true,
                ..CREDIT_BASE
            },
            Self::IndianEmployment => &CreditInfo {
                key: "indian_employment",
                form: "Form 8845",
                form_key: "BUSINESS_CREDIT_FORM_INDIAN_EMPLOYMENT",
                line: "2c",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::OrphanDrug => &CreditInfo {
                key: "orphan_drug",
                form: "Form 8820",
                form_key: "BUSINESS_CREDIT_FORM_ORPHAN_DRUG",
                line: "1h",
                first_year: 2006,
                carryback_from: Some(2025),
                ..CREDIT_BASE
            },
            Self::NewMarkets => &CreditInfo {
                key: "new_markets",
                form: "Form 8874",
                form_key: "BUSINESS_CREDIT_FORM_NEW_MARKETS",
                line: "1i",
                first_year: 2006,
                carryback_from: Some(2025),
                ..CREDIT_BASE
            },
            Self::EmployerChildcare => &CreditInfo {
                key: "employer_childcare",
                form: "Form 8882",
                form_key: "BUSINESS_CREDIT_FORM_EMPLOYER_CHILDCARE",
                line: "1k",
                first_year: 2006,
                carryback_from: Some(2025),
                cap_cents: Some(50000000),
                ..CREDIT_BASE
            },
            Self::RailroadTrack => &CreditInfo {
                key: "railroad_track",
                form: "Form 8900",
                form_key: "BUSINESS_CREDIT_FORM_RAILROAD_TRACK",
                line: "4g",
                first_year: 2006,
                carryback_from: Some(2025),
                specified_from: Some(2008),
                early_line: Some(("2m", 2008)),
                ..CREDIT_BASE
            },
            Self::Biodiesel => &CreditInfo {
                key: "biodiesel",
                form: "Form 8864",
                form_key: "BUSINESS_CREDIT_FORM_BIODIESEL",
                line: "1l",
                first_year: 2006,
                carryback_from: Some(2025),
                transferable: true,
                ..CREDIT_BASE
            },
            Self::LowSulfurDiesel => &CreditInfo {
                key: "low_sulfur_diesel",
                form: "Form 8896",
                form_key: "BUSINESS_CREDIT_FORM_LOW_SULFUR_DIESEL",
                line: "1m",
                first_year: 2006,
                carryback_from: Some(2025),
                ..CREDIT_BASE
            },
            Self::MarginalWells => &CreditInfo {
                key: "marginal_wells",
                form: "Form 8904",
                form_key: "BUSINESS_CREDIT_FORM_MARGINAL_WELLS",
                line: "1bb",
                first_year: 2006,
                carryback_from: Some(2021),
                ..CREDIT_BASE
            },
            Self::DistilledSpirits => &CreditInfo {
                key: "distilled_spirits",
                form: "Form 8906",
                form_key: "BUSINESS_CREDIT_FORM_DISTILLED_SPIRITS",
                line: "1n",
                first_year: 2018,
                carryback_from: Some(2025),
                ..CREDIT_BASE
            },
            Self::AdvancedNuclear => &CreditInfo {
                key: "advanced_nuclear",
                form: "Form 7213 Part I",
                form_key: "BUSINESS_CREDIT_FORM_ADVANCED_NUCLEAR",
                line: "1cc",
                first_year: 2006,
                carryback_from: Some(2025),
                ..CREDIT_BASE
            },
            Self::NonconventionalFuel => &CreditInfo {
                key: "nonconventional_fuel",
                form: "Form 8907",
                form_key: "BUSINESS_CREDIT_FORM_NONCONVENTIONAL_FUEL",
                line: "2d",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::EnergyEfficientHome => &CreditInfo {
                key: "energy_efficient_home",
                form: "Form 8908",
                form_key: "BUSINESS_CREDIT_FORM_ENERGY_EFFICIENT_HOME",
                line: "1p",
                first_year: 2006,
                carryback_from: Some(2025),
                ..CREDIT_BASE
            },
            Self::EnergyEfficientAppliance => &CreditInfo {
                key: "energy_efficient_appliance",
                form: "Form 8909",
                form_key: "BUSINESS_CREDIT_FORM_ENERGY_EFFICIENT_APPLIANCE",
                line: "2e",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::RefuelingProperty => &CreditInfo {
                key: "refueling_property",
                form: "Form 8911",
                form_key: "BUSINESS_CREDIT_FORM_REFUELING_PROPERTY",
                line: "1s",
                first_year: 2006,
                carryback_from: Some(2023),
                transferable: true,
                ..CREDIT_BASE
            },
            Self::MineRescue => &CreditInfo {
                key: "mine_rescue",
                form: "Form 8923",
                form_key: "BUSINESS_CREDIT_FORM_MINE_RESCUE",
                line: "2f",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::AgriculturalChemicals => &CreditInfo {
                key: "agricultural_chemicals",
                form: "Form 8931",
                form_key: "BUSINESS_CREDIT_FORM_AGRICULTURAL_CHEMICALS",
                line: "2h",
                first_year: 2008,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::DifferentialWage => &CreditInfo {
                key: "differential_wage",
                form: "Form 8932",
                form_key: "BUSINESS_CREDIT_FORM_DIFFERENTIAL_WAGE",
                line: "1w",
                first_year: 2008,
                carryback_from: Some(2025),
                ..CREDIT_BASE
            },
            Self::CarbonOxide => &CreditInfo {
                key: "carbon_oxide",
                form: "Form 8933",
                form_key: "BUSINESS_CREDIT_FORM_CARBON_OXIDE",
                line: "1x",
                first_year: 2008,
                carryback_from: Some(2023),
                elective_payment: true,
                transferable: true,
                ..CREDIT_BASE
            },
            Self::CleanVehicle => &CreditInfo {
                key: "clean_vehicle",
                form: "Form 8936 Part II",
                form_key: "BUSINESS_CREDIT_FORM_CLEAN_VEHICLE",
                line: "1y",
                first_year: 2010,
                carryback_from: Some(2025),
                magi_limited: true,
                ..CREDIT_BASE
            },
            Self::DisasterRetention => &CreditInfo {
                key: "disaster_retention",
                form: "Form 5884-A",
                form_key: "BUSINESS_CREDIT_FORM_DISASTER_RETENTION",
                line: "2a",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::HurricaneRetention => &CreditInfo {
                key: "hurricane_retention",
                form: "Form 5884-A Section A",
                form_key: "BUSINESS_CREDIT_FORM_HURRICANE_RETENTION",
                line: "2o",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::HurricaneHousing => &CreditInfo {
                key: "hurricane_housing",
                form: "Form 5884-A Section B",
                form_key: "BUSINESS_CREDIT_FORM_HURRICANE_HOUSING",
                line: "2p",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::MidwesternRetention => &CreditInfo {
                key: "midwestern_retention",
                form: "Form 5884-A Section A",
                form_key: "BUSINESS_CREDIT_FORM_MIDWESTERN_RETENTION",
                line: "2q",
                first_year: 2008,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::MidwesternHousing => &CreditInfo {
                key: "midwestern_housing",
                form: "Form 5884-A Section B",
                form_key: "BUSINESS_CREDIT_FORM_MIDWESTERN_HOUSING",
                line: "2r",
                first_year: 2008,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::ZeroEmissionNuclear => &CreditInfo {
                key: "zero_emission_nuclear",
                form: "Form 7213 Part II",
                form_key: "BUSINESS_CREDIT_FORM_ZERO_EMISSION_NUCLEAR",
                line: "1u",
                first_year: 2024,
                carryback_from: Some(2023),
                carryback_origin: CarrybackOrigin::ApplicableCreditException,
                transferable: true,
                ..CREDIT_BASE
            },
            Self::SustainableAviationFuel => &CreditInfo {
                key: "sustainable_aviation_fuel",
                form: "Form 8864 SAF",
                form_key: "BUSINESS_CREDIT_FORM_SUSTAINABLE_AVIATION_FUEL",
                line: "1ff",
                first_year: 2023,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::CleanHydrogen => &CreditInfo {
                key: "clean_hydrogen",
                form: "Form 7210",
                form_key: "BUSINESS_CREDIT_FORM_CLEAN_HYDROGEN",
                line: "1g",
                first_year: 2023,
                carryback_from: Some(2023),
                elective_payment: true,
                transferable: true,
                ..CREDIT_BASE
            },
            Self::CommercialCleanVehicle => &CreditInfo {
                key: "commercial_clean_vehicle",
                form: "Form 8936 Part V",
                form_key: "BUSINESS_CREDIT_FORM_COMMERCIAL_CLEAN_VEHICLE",
                line: "1aa",
                first_year: 2023,
                carryback_from: Some(2025),
                ..CREDIT_BASE
            },
            Self::AdvancedManufacturingProduction => &CreditInfo {
                key: "advanced_manufacturing_production",
                form: "Form 7207",
                form_key: "BUSINESS_CREDIT_FORM_ADVANCED_MANUFACTURING_PRODUCTION",
                line: "1b",
                first_year: 2023,
                carryback_from: Some(2023),
                elective_payment: true,
                transferable: true,
                ..CREDIT_BASE
            },
            Self::CleanElectricityProduction => &CreditInfo {
                key: "clean_electricity_production",
                form: "Form 7211 Part II",
                form_key: "BUSINESS_CREDIT_FORM_CLEAN_ELECTRICITY_PRODUCTION",
                line: "1gg",
                first_year: 2025,
                carryback_from: Some(2023),
                carryback_origin: CarrybackOrigin::ApplicableCreditException,
                transferable: true,
                ..CREDIT_BASE
            },
            Self::ElectingLargePartnership => &CreditInfo {
                key: "electing_large_partnership",
                form: "Form 1065-B",
                form_key: "BUSINESS_CREDIT_FORM_ELECTING_LARGE_PARTNERSHIP",
                line: "2i",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::QualifiedElectricVehicle => &CreditInfo {
                key: "qualified_electric_vehicle",
                form: "Form 8834",
                form_key: "BUSINESS_CREDIT_FORM_QUALIFIED_ELECTRIC_VEHICLE",
                line: "2g",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::NewHireRetention => &CreditInfo {
                key: "new_hire_retention",
                form: "Form 5884-B",
                form_key: "BUSINESS_CREDIT_FORM_NEW_HIRE_RETENTION",
                line: "2s",
                first_year: 2010,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::CommunityDevelopment => &CreditInfo {
                key: "community_development",
                form: "Form 8847",
                form_key: "BUSINESS_CREDIT_FORM_COMMUNITY_DEVELOPMENT",
                line: "2t",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::WelfareToWork => &CreditInfo {
                key: "welfare_to_work",
                form: "Form 8861",
                form_key: "BUSINESS_CREDIT_FORM_WELFARE_TO_WORK",
                line: "2u",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::ExpiredOther => &CreditInfo {
                key: "expired_other",
                form: "Another expired credit",
                form_key: "BUSINESS_CREDIT_FORM_EXPIRED_OTHER",
                line: "2zz",
                first_year: 2006,
                carryback_from: None,
                ..CREDIT_BASE
            },
            Self::SmallBusiness2010 => &CreditInfo {
                key: "small_business2010",
                form: "2010 eligible small business credit",
                form_key: "BUSINESS_CREDIT_FORM_SMALL_BUSINESS2010",
                line: "4y",
                first_year: 2010,
                carryback_from: None,
                specified_from: Some(0),
                ..CREDIT_BASE
            },
            Self::Other => &CreditInfo {
                key: "other",
                form: "Another general business credit",
                form_key: "BUSINESS_CREDIT_FORM_OTHER",
                line: "1zz",
                first_year: 2006,
                carryback_from: Some(2025),
                ..CREDIT_BASE
            },
            Self::OtherSpecified => &CreditInfo {
                key: "other_specified",
                form: "Another specified credit",
                form_key: "BUSINESS_CREDIT_FORM_OTHER_SPECIFIED",
                line: "4z",
                first_year: 2006,
                carryback_from: Some(2025),
                specified_from: Some(0),
                ..CREDIT_BASE
            },
        }
    }

    pub(crate) fn entered_choices() -> Vec<i32> {
        Self::ALL
            .iter()
            .filter(|c| c.carryback_allowed() && **c != Self::EmpowermentZone)
            .map(|c| *c as i32)
            .collect()
    }

    pub(crate) fn transferable(self) -> bool {
        self.info().transferable
    }

    pub(crate) fn election_choices(self) -> Vec<i32> {
        let mut choices = alloc::vec![0];
        if self.info().elective_payment {
            choices.push(1);
        }
        if self.info().transferable {
            choices.push(2);
        }
        choices
    }

    pub(crate) fn research(self) -> bool {
        self.info().research
    }

    pub(crate) fn order(self) -> usize {
        Self::ALL.iter().position(|c| *c == self).unwrap_or(0)
    }

    /// Whether a 2026 amount of this credit may be carried back to 2025.
    pub(crate) fn carryback_allowed(self) -> bool {
        self.info().carryback_from.is_some()
    }

    /// The earliest year a 2026 amount of this credit may be carried back to.
    pub(crate) fn carryback_first_year(self) -> u32 {
        let info = self.info();
        let period_start = info.carryback_from.unwrap_or(2025);
        match info.carryback_origin {
            CarrybackOrigin::FirstAllowedYear => max(period_start, self.first_year()),
            CarrybackOrigin::ApplicableCreditException => period_start,
        }
    }

    pub(crate) fn key(self) -> &'static str {
        self.info().key
    }

    // The credit named at the start of a calculation id's tail, with the rest
    // after it (`energy_current_year_used` -> Energy, `current_year_used`). The longest key
    // wins: `low_income_housing_pre2008` before `low_income_housing`.
    pub(crate) fn split_id(tail: &str) -> Option<(Self, &str)> {
        Self::ALL
            .iter()
            .filter_map(|credit| {
                tail.strip_prefix(credit.key())
                    .and_then(|rest| rest.strip_prefix('_'))
                    .map(|rest| (*credit, rest))
            })
            .max_by_key(|(credit, _)| credit.key().len())
    }

    // Form 8582-CR worksheet that limits this credit when it is passive:
    // Worksheet 3 for the post-1989 low-income housing credit, else Worksheet 4.
    pub(crate) fn worksheet(self) -> PassiveWorksheet {
        match self {
            Self::LowIncomeHousing | Self::LowIncomeHousingBefore2008 => {
                PassiveWorksheet::LowIncomeHousing
            }
            _ => PassiveWorksheet::Other,
        }
    }

    // Whether a passive amount of this credit may come from the worksheet: a
    // rehabilitation credit is Worksheet 2 when it belongs to a rental real
    // estate activity and Worksheet 4 otherwise.
    pub(crate) fn worksheet_allows(self, worksheet: PassiveWorksheet) -> bool {
        match self {
            Self::LowIncomeHousing | Self::LowIncomeHousingBefore2008 => matches!(
                worksheet,
                PassiveWorksheet::LowIncomeHousing | PassiveWorksheet::LowIncomeHousingBefore1990
            ),
            Self::Rehabilitation => {
                matches!(worksheet, PassiveWorksheet::Other | PassiveWorksheet::Rehabilitation)
            }
            _ => matches!(worksheet, PassiveWorksheet::Other | PassiveWorksheet::RentalActive),
        }
    }

    #[cfg(test)]
    pub(crate) fn form(self) -> &'static str {
        self.info().form
    }

    pub(crate) fn form_message_key(self) -> &'static str {
        self.info().form_key
    }

    // Whether the credit determined in `year` is a specified credit
    // (section 38(c)(4)(B)).
    pub(crate) fn specified(self, year: u32) -> bool {
        self.info().specified_from.is_some_and(|from| year >= from)
    }

    // Part III line for a 2026 credit; Part IV line for a carryforward.
    pub(crate) fn line(self, year: u32) -> &'static str {
        let info = self.info();
        match info.early_line {
            Some((early, first)) if year < first => early,
            _ => info.line,
        }
    }

    // The first tax year the credit could arise.
    pub(crate) fn first_year(self) -> u32 {
        self.info().first_year
    }
}
/// Actual carbon-capture equipment credited by Form8933 or its attached statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CarbonOxideEquipmentCredit {
    /// Equipment or process-train identifier from the source document
    /// Include the facility in this reference when needed to identify the actual equipment. Do not infer it from a business or asset name.
    #[libertas_size(min = 1)]
    pub identity: alloc::string::String,
    /// Original placed-in-service date of this equipment
    /// Use this equipment's first placed-in-service date, not a later acquisition, expansion or replacement date. A missing date keeps this source's extended carryback exception unavailable.
    /// [NullText]
    /// The original equipment date is unavailable
    /// [SetText]
    /// The equipment was originally placed in service on
    #[libertas_date_only]
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(max = 20261231)]
    #[libertas_default($null)]
    pub original_placed_on: Option<u32>,
    /// Credit attributable to this equipment on the source statement
    /// Before Form3800 limits. The equipment amounts must exactly equal this credit row's gross source amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
}
/// Origin of the carbon oxide credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CarbonOxideCreditOrigin {
    /// Actual equipment or process-train source statement
    Statement {
        /// Equipment that produced this credit
        /// Enter actual component amounts when one source statement includes multiple pieces of equipment. No proportional or estimated split.
        #[libertas_unordered]
        /// ----
        /// Equipment
        equipment: Vec<CarbonOxideEquipmentCredit>,
    },
}
// The two physical-source classes keep distinct questions and stored labels.
// Their structural validity, date facts and paired community projection have
// one native owner; no component credit-use ordering is decided here.
pub(crate) trait CreditOriginPart: Clone {
    fn identity(&self) -> &str;
    fn original_date(&self) -> Option<u32>;
    fn amount(&self) -> i64;
    fn set_amount(&mut self, value: i64);
}
impl CreditOriginPart for CarbonOxideEquipmentCredit {
    fn identity(&self) -> &str {
        &self.identity
    }
    fn original_date(&self) -> Option<u32> {
        self.original_placed_on
    }
    fn amount(&self) -> i64 {
        self.amount
    }
    fn set_amount(&mut self, value: i64) {
        self.amount = value;
    }
}
fn valid_origin_parts<P: CreditOriginPart>(parts: &[P], gross: i64) -> bool {
    valid_origin_details(parts)
        && parts.iter().try_fold(0i64, |sum, part| sum.checked_add(part.amount())) == Some(gross)
}
pub(crate) fn valid_origin_details<P: CreditOriginPart>(parts: &[P]) -> bool {
    !parts.is_empty()
        && parts.len() <= 60
        && parts.iter().enumerate().all(|(i, part)| {
            !part.identity().trim().is_empty()
                && (1..=100_000_000_000).contains(&part.amount())
                && part
                    .original_date()
                    .is_none_or(|date| crate::dates::source_valid_date(date) && date <= 20261231)
                && !parts[..i].iter().any(|prior| prior.identity() == part.identity())
        })
}
fn origin_parts_state<P: CreditOriginPart>(parts: &[P]) -> ConditionalCarrybackOrigin {
    parts.iter().fold(ConditionalCarrybackOrigin::default(), |state, part| {
        state.merge(match part.original_date() {
            None => ConditionalCarrybackOrigin::unknown(),
            Some(date) if date > 20221231 => ConditionalCarrybackOrigin(4),
            Some(_) => ConditionalCarrybackOrigin(2),
        })
    })
}
impl CarbonOxideCreditOrigin {
    pub(crate) fn valid(&self, gross: i64) -> bool {
        let Self::Statement { equipment } = self;
        valid_origin_parts(equipment, gross)
    }
    pub(crate) fn carryback_origin(&self) -> ConditionalCarrybackOrigin {
        let Self::Statement { equipment } = self;
        origin_parts_state(equipment)
    }
}
/// Actual renewable-electricity facility credited by Form8835 or its attached statement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RenewableFacilityCredit {
    /// Facility identifier from the source document
    /// Identify the actual facility that produced this credit. Do not infer it from a business name or another property's date.
    #[libertas_size(min = 1)]
    pub identity: alloc::string::String,
    /// Original placed-in-service date of this facility
    /// Use the original service date of the actual credited facility. A missing date keeps its extended carryback exception unavailable.
    /// [NullText]
    /// The original facility date is unavailable
    /// [SetText]
    /// The facility was originally placed in service on
    #[libertas_date_only]
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(max = 20261231)]
    #[libertas_default($null)]
    pub original_placed_on: Option<u32>,
    /// Credit attributable to this facility on the source statement
    /// Before Form3800 limits. The facility amounts must exactly equal this credit row's gross source amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub amount: i64,
}
/// Origin of the specified renewable-electricity credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RenewableFacilityCreditOrigin {
    /// Actual facility source statement
    Statement {
        /// Facilities that produced this credit
        /// Enter actual component amounts when the statement includes multiple facilities. No proportional or estimated split.
        #[libertas_unordered]
        /// ----
        /// Facility
        facilities: Vec<RenewableFacilityCredit>,
    },
}
impl CreditOriginPart for RenewableFacilityCredit {
    fn identity(&self) -> &str {
        &self.identity
    }
    fn original_date(&self) -> Option<u32> {
        self.original_placed_on
    }
    fn amount(&self) -> i64 {
        self.amount
    }
    fn set_amount(&mut self, value: i64) {
        self.amount = value;
    }
}
impl RenewableFacilityCreditOrigin {
    pub(crate) fn valid(&self, gross: i64) -> bool {
        let Self::Statement { facilities } = self;
        valid_origin_parts(facilities, gross)
    }
    pub(crate) fn carryback_origin(&self) -> ConditionalCarrybackOrigin {
        let Self::Statement { facilities } = self;
        origin_parts_state(facilities)
    }
}
/// General business credit figured on its own form
/// The credit from your own Form 8826, 6765, 8882, 8932 or another source form, entered at the Form 3800 Part III line that form names. A credit on a Schedule K-1 belongs with that K-1.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EnteredBusinessCredit {
    /// Credit
    /// The Form 3800 Part III line your credit form sends it to. Credits the App figures from your business page (Forms 5884, 8846, 8881, 8941, 8994) and from property (Form 3468) are not entered here unless you figured them yourself because the App cannot.
    pub credit: BusinessCredit,
    /// Where the credit comes from
    pub source: CreditSource,
    /// Credit from the form
    /// The amount your credit form reports for Form 3800, before any tax limit, in whole dollars. For the research credit, leave out any part elected as a payroll tax credit (Form 6765 line 34).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
    pub amount: i64,
    /// Elective payment or transfer
    /// Only with an IRS registration number for the property, obtained before filing.
    #[libertas_default(None)]
    pub election: CreditElection,
    /// Actual equipment origin for the carbon oxide credit
    /// Source-document facts only; this does not elect payment or determine any other credit.
    /// [NullText]
    /// I do not have the equipment origin details for the extended carryback
    /// [SetText]
    /// The equipment origin details are
    #[libertas_default($null)]
    pub carbon_oxide_origin: Option<CarbonOxideCreditOrigin>,
    /// Actual facility origin for the specified renewable-electricity credit
    /// Source-document facts only; this does not elect payment or determine a generic renewable-electricity credit.
    /// [NullText]
    /// I do not have the facility origin details for the extended carryback
    /// [SetText]
    /// The facility origin details are
    #[libertas_default($null)]
    pub renewable_facility_origin: Option<RenewableFacilityCreditOrigin>,
}
/// Where the credit comes from
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CreditSource {
    /// A business you run (Schedule C)
    Business {
        /// Business
        /// Empty uses your only business on Self-employment.
        #[libertas_default($null)]
        business: Option<i64>,
        /// Expenses and basis already reduced for this credit
        /// The credit form names the deduction or basis to reduce by the credit, such as wages for Form 8932, clinical testing expenses for Form 8820 or research expenses for Form 6765 (unless you elected the reduced credit), or the property's basis for Forms 8826 and 8882. Yes when the amounts entered for this business already reflect it. No leaves this credit out of the estimate.
        reduced: bool,
    },
    /// A farm (Schedule F or Form 4835)
    Farm {
        /// Farm
        /// Empty uses your only farm.
        #[libertas_default($null)]
        farm: Option<i64>,
        /// Expenses and basis already reduced for this credit
        /// The credit form names the deduction or basis to reduce by the credit. Yes when the amounts entered for this farm already reflect it. No leaves this credit out of the estimate.
        reduced: bool,
    },
    /// A rental property (Schedule E)
    /// A credit of a rental you actively participate in can use the $25,000 special allowance (Form 8582-CR Part II).
    Rental {
        /// Rental
        /// Empty uses your only rental.
        #[libertas_default($null)]
        rental: Option<i64>,
        /// Expenses and basis already reduced for this credit
        /// The credit form names the deduction or basis to reduce by the credit. Yes when the amounts entered for this rental already reflect it. No leaves this credit out of the estimate.
        reduced: bool,
    },
    /// Bought from another taxpayer (section 6418 transfer)
    /// A credit you paid cash for under a transfer election statement. It is a passive activity credit, used only against the tax on passive income; the price is not deductible.
    Bought,
}
/// Elective payment or transfer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_inline_choices]
pub enum CreditElection {
    /// Neither
    None,
    /// Elective payment (sections 6417 and 48D)
    /// For the advanced manufacturing production, clean hydrogen, carbon oxide sequestration or advanced manufacturing investment credit of property you hold directly. The credit first reduces your tax; what is left of the elected amount is paid to you (Schedule 3 line 13c).
    ElectivePayment {
        /// Gross elective payment amount
        /// Form 3800 Part III column (h), in whole dollars.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 100, max = 100000000000)]
        amount: i64,
    },
    /// Sold to another taxpayer (section 6418 transfer)
    /// The part you transferred for cash. The cash is not income, and the part sold is not your credit.
    Sold {
        /// Amount sold
        /// Form 3800 Part III column (f), in whole dollars.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 100, max = 100000000000)]
        amount: i64,
    },
}
/// Unused credit carried forward to 2026
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessCreditCarryforward {
    /// Credit
    pub credit: BusinessCredit,
    /// Tax year the credit arose
    /// A credit carries forward 20 years, so 2006 is the earliest year that reaches 2026.
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 2006, max = 2025)]
    pub origin_year: u32,
    /// Unused credit carried to 2026
    /// From your 2025 Form 3800 Part IV column (i) or Part III carryforward records, in whole dollars.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
    pub amount: i64,
    /// Form 8582-CR worksheet of a passive activity credit disallowed in 2025
    /// Choose the worksheet of your 2025 Form 8582-CR that carried the amount when it is a prior-year unallowed passive credit rather than a Form 3800 carryforward. Leave it empty for a Form 3800 carryforward. A low-income housing credit uses Worksheet 3; a rehabilitation credit of a rental real estate activity uses Worksheet 2; every other credit uses Worksheet 4.
    /// [NullText]
    /// This is a Form 3800 carryforward, not an unallowed passive activity credit
    /// [SetText]
    /// The prior unallowed passive credit came from
    pub passive: Option<PassiveWorksheet>,
    /// Whose credit
    pub filer: CarryFiler,
    /// Originating business whose current taxable income supports this research credit
    /// Absence affirmatively means no corresponding taxable income on this return.
    pub research_source: Option<i64>,
}
/// Whose credit
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum CarryFiler {
    /// The same filers as this return
    SameFilers,
    /// Yours alone
    /// It arose when you filed without your current spouse.
    You,
    /// Your spouse's alone
    /// It arose when your spouse filed without you.
    Spouse,
}
/// 2026 credit carried back to an earlier return
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessCreditCarryback {
    /// Credit
    pub credit: BusinessCredit,
    /// Year of the return it was carried back to
    /// Ordinary credits carry back to 2025. Applicable energy, nuclear, clean electricity and clean fuel credits carry back to 2023–2025; marginal-well credits carry back to 2021–2025.
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 2021, max = 2025)]
    #[libertas_default(2025)]
    pub year: u32,
    /// Amount used on that carryback return
    /// From the Form 1040-X or Form 1045 that carries the unused 2026 credit back, in whole dollars.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
    pub amount: i64,
    /// Originating current business for a research-credit carryback
    pub research_source: Option<i64>,
}
/// General business credit carryovers (Form 3800)
/// Enter credits carried from earlier years and 2026 credits carried back to earlier returns. The 2025 Form 8846 carryforward has its own item.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessCreditHistory {
    /// Credits figured on their own forms
    pub credits: Vec<EnteredBusinessCredit>,
    /// Credits carried forward to 2026 (Part IV)
    /// A credit carried from a property entered with its investment credit is added from that property, not here.
    #[libertas_unordered]
    /// ----
    /// Carryforward
    pub carryforwards: Vec<BusinessCreditCarryforward>,
    /// 2026 credits carried back to earlier returns
    #[libertas_unordered]
    /// ----
    /// Carryback
    pub carrybacks: Vec<BusinessCreditCarryback>,
    /// Your spouse has no general business credit, carryforward or carryback for 2026
    /// Married filing separately only. When true, the $25,000 amount on Form 3800 line 13 applies instead of $12,500.
    #[libertas_default(false)]
    pub spouse_without_credits: bool,
}
// One credit amount entering Form 3800, whole dollars. `year` is 2026 for a
// current-year credit and the originating year for a carryforward. A passive
// activity credit names its Form 8582-CR worksheet; it enters the tax limit
// only as far as that form allows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ConditionalCarrybackOrigin(u8);
impl ConditionalCarrybackOrigin {
    pub(crate) fn unknown() -> Self {
        Self(1)
    }
    pub(crate) fn from_origin(origin: Option<&CarbonOxideCreditOrigin>) -> Self {
        origin.map_or_else(Self::unknown, CarbonOxideCreditOrigin::carryback_origin)
    }
    pub(crate) fn from_source(
        credit: BusinessCredit,
        carbon: Option<&CarbonOxideCreditOrigin>,
        renewable: Option<&RenewableFacilityCreditOrigin>,
    ) -> Option<Self> {
        match credit {
            BusinessCredit::CarbonOxide => Some(Self::from_origin(carbon)),
            BusinessCredit::RenewableElectricitySpecified => Some(
                renewable
                    .map_or_else(Self::unknown, RenewableFacilityCreditOrigin::carryback_origin),
            ),
            _ => None,
        }
    }
    pub(crate) fn merge(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    pub(crate) fn extended(self) -> bool {
        self.0 == 4
    }
    pub(crate) fn mixed(self) -> bool {
        self.0 & 6 == 6
    }
    pub(crate) fn unknown_date(self) -> bool {
        self.0 & 1 != 0
    }
    // Statutory section 45/45Q carryback eligibility cutoff, not a relative return year.
    pub(crate) fn before2023(self) -> bool {
        self.0 == 2
    }
}

impl BusinessCreditHistory {
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        for (index, item) in self.credits.iter().enumerate() {
            let info = item.credit.info();
            if item.carbon_oxide_origin.as_ref().is_some_and(|origin| {
                item.credit != BusinessCredit::CarbonOxide || !origin.valid(item.amount)
            }) {
                return Some(
                    ValidationIssue::new("CARBON_OXIDE_SOURCE_STATEMENT_INVALID")
                        .at_field("carbon_oxide_origin")
                        .in_list("credits", index),
                );
            }
            if item.renewable_facility_origin.as_ref().is_some_and(|origin| {
                item.credit != BusinessCredit::RenewableElectricitySpecified
                    || !origin.valid(item.amount)
            }) {
                return Some(
                    ValidationIssue::new("RENEWABLE_FACILITY_SOURCE_STATEMENT_INVALID")
                        .at_field("renewable_facility_origin")
                        .in_list("credits", index),
                );
            }
            if !item.credit.carryback_allowed() || info.section_b {
                return Some(
                    ValidationIssue::new("BUSINESS_CREDIT_CARRYFORWARD_ONLY")
                        .at_field("credit")
                        .in_list("credits", index),
                );
            }
            match item.election {
                CreditElection::ElectivePayment { amount } => {
                    if !info.elective_payment {
                        return Some(
                            ValidationIssue::new("ELECTIVE_PAYMENT_NOT_FOR_THIS_CREDIT")
                                .at_field("credit")
                                .in_list("credits", index),
                        );
                    }
                    if !matches!(
                        item.source,
                        CreditSource::Business { .. } | CreditSource::Farm { .. }
                    ) {
                        return Some(
                            ValidationIssue::new("ELECTIVE_PAYMENT_OWN_PROPERTY")
                                .at_field("source")
                                .in_list("credits", index),
                        );
                    }
                    if amount > item.amount {
                        return Some(
                            ValidationIssue::new("BUSINESS_CREDIT_ELECTION_ABOVE_CREDIT")
                                .at_field("amount")
                                .in_field("election")
                                .in_list("credits", index),
                        );
                    }
                }
                CreditElection::Sold { amount } => {
                    if matches!(item.source, CreditSource::Bought) {
                        return Some(
                            ValidationIssue::new("BOUGHT_CREDIT_CANNOT_BE_SOLD")
                                .at_field("source")
                                .in_list("credits", index),
                        );
                    }
                    if !info.transferable {
                        return Some(
                            ValidationIssue::new("TRANSFER_NOT_FOR_THIS_CREDIT")
                                .at_field("credit")
                                .in_list("credits", index),
                        );
                    }
                    if amount > item.amount {
                        return Some(
                            ValidationIssue::new("BUSINESS_CREDIT_ELECTION_ABOVE_CREDIT")
                                .at_field("amount")
                                .in_field("election")
                                .in_list("credits", index),
                        );
                    }
                }
                CreditElection::None => {}
            }
            if matches!(item.source, CreditSource::Bought) && !info.transferable {
                return Some(
                    ValidationIssue::new("TRANSFER_NOT_FOR_THIS_CREDIT")
                        .at_field("credit")
                        .in_list("credits", index),
                );
            }
            if item.credit == BusinessCredit::DisabledAccess
                && item.amount > crate::year_constants::DISABLED_ACCESS_CREDIT_LIMIT_CENTS
            {
                return Some(
                    ValidationIssue::new("DISABLED_ACCESS_ABOVE_LIMIT")
                        .at_field("amount")
                        .in_list("credits", index),
                );
            }
        }
        for (index, item) in self.carryforwards.iter().enumerate() {
            if item.origin_year < item.credit.first_year()
                || (item.credit == BusinessCredit::SmallBusiness2010 && item.origin_year != 2010)
            {
                return Some(
                    ValidationIssue::new("BUSINESS_CREDIT_BEFORE_ITS_FIRST_YEAR")
                        .at_field("origin_year")
                        .in_list("carryforwards", index),
                );
            }
            if item.credit == BusinessCredit::EmployerTipFica && item.origin_year >= 2025 {
                return Some(
                    ValidationIssue::new("TIP_FICA_PRIOR_YEAR_HAS_ITS_OWN_ITEM")
                        .at_field("origin_year")
                        .in_list("carryforwards", index),
                );
            }
            if item.passive.is_some_and(|worksheet| !item.credit.worksheet_allows(worksheet)) {
                return Some(
                    ValidationIssue::new("BUSINESS_CREDIT_WORKSHEET_MISMATCH")
                        .at_field("passive")
                        .in_list("carryforwards", index),
                );
            }
            if self.carryforwards[..index].iter().any(|prior| {
                prior.credit == item.credit
                    && prior.origin_year == item.origin_year
                    && prior.passive == item.passive
                    && prior.filer == item.filer
                    && prior.research_source == item.research_source
            }) {
                return Some(
                    ValidationIssue::new("BUSINESS_CARRYFORWARD_TWICE")
                        .at_field("credit")
                        .in_list("carryforwards", index),
                );
            }
        }
        for (index, item) in self.carrybacks.iter().enumerate() {
            if item.credit == BusinessCredit::EmployerTipFica {
                return Some(
                    ValidationIssue::new("TIP_FICA_CARRYBACK_ON_ITS_CREDIT")
                        .at_field("credit")
                        .in_list("carrybacks", index),
                );
            }
            if !item.credit.carryback_allowed() {
                return Some(
                    ValidationIssue::new("BUSINESS_CREDIT_CARRYFORWARD_ONLY")
                        .at_field("credit")
                        .in_list("carrybacks", index),
                );
            }
            if !(item.credit.carryback_first_year()..=2025).contains(&item.year) {
                return Some(
                    ValidationIssue::new("BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED")
                        .at_field("year")
                        .in_list("carrybacks", index),
                );
            }
            if self.carrybacks[..index].iter().any(|prior| {
                prior.credit == item.credit
                    && prior.year == item.year
                    && prior.research_source == item.research_source
            }) {
                return Some(
                    ValidationIssue::new("BUSINESS_CARRYBACK_TWICE")
                        .at_field("credit")
                        .in_list("carrybacks", index),
                );
            }
        }
        None
    }
    pub(crate) fn valid(&self) -> bool {
        self.credits.len() <= 40
            && self.credits.iter().all(|c| {
                (100..=100_000_000_000).contains(&c.amount)
                    && c.carbon_oxide_origin.as_ref().is_none_or(|origin| {
                        c.credit == BusinessCredit::CarbonOxide && origin.valid(c.amount)
                    })
                    && c.renewable_facility_origin.as_ref().is_none_or(|origin| {
                        c.credit == BusinessCredit::RenewableElectricitySpecified
                            && origin.valid(c.amount)
                    })
                    && c.amount % 100 == 0
                    && match c.election {
                        CreditElection::None => true,
                        CreditElection::ElectivePayment { amount }
                        | CreditElection::Sold { amount } => {
                            (100..=100_000_000_000).contains(&amount) && amount % 100 == 0
                        }
                    }
            })
            && self.carryforwards.len() <= 60
            && self.carrybacks.len() <= 40
            && self.carryforwards.iter().all(|c| {
                (2006..=2025).contains(&c.origin_year)
                    && (100..=100_000_000_000).contains(&c.amount)
                    && c.amount % 100 == 0
            })
            && self.carrybacks.iter().all(|c| {
                (2021..=2025).contains(&c.year)
                    && (100..=100_000_000_000).contains(&c.amount)
                    && c.amount % 100 == 0
            })
    }
}
