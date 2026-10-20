// Frozen native Avro bodies from main 701d0e944a37d9ce1d93d0e0bcff6c77d25614e6.
// Declaration order, variant order and native integer widths are the format-124
// contract. Only the ancestors of changed HOME facts are copied; other facts
// continue to use their unchanged owners. No presentation rules apply here.
use crate::*;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Retained format-124 FederalIncomeEntry body.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalIncomeEntry124 {
    pub id: i64,
    pub label: String,
    pub owner: Owner,
    pub community: CommunityShare,
    pub income: FederalIncome124,
    pub state_source: Option<crate::IncomeStateSource>,
    pub refund_management: Option<crate::RefundManagement>,
}

/// Retained format-124 rental carrier; declaration order and widths stay frozen.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalRental124 {
    pub energy_buildings: Vec<crate::RentalEnergyBuildingFacts>,
    pub property_qbi_history: Vec<crate::RentalPropertyLossYear>,
    pub mortgage_loans: Vec<i64>,
    pub rent: i64,
    pub expenses: Vec<RentalExpense>,
    pub buildings: Vec<RentalBuilding>,
    pub assets: Vec<crate::BusinessAsset>,
    pub participation: RentalParticipation,
    pub ordinary_rental: bool,
    pub at_risk: Option<RentalAtRisk>,
    pub qbi_business: RentalBusiness,
    pub history: RentalHistory,
    pub method_changes: Vec<crate::MethodChange>,
    pub payroll: BusinessPayroll,
    pub qbi_aggregation_group: Option<String>,
    pub personal_use: RentalPersonalUse,
    pub home_operating_carryover: i64,
    pub home_depreciation_carryover: i64,
    pub property_type: RentalPropertyType,
    pub sale: crate::RentalSale,
    pub statement: Option<crate::Rental1099S>,
}

impl From<FederalRental124> for FederalRental {
    fn from(value: FederalRental124) -> Self {
        Self {
            energy_buildings: value.energy_buildings,
            property_qbi_history: value.property_qbi_history,
            mortgage_loans: value.mortgage_loans,
            rent: value.rent,
            expenses: value.expenses,
            buildings: value.buildings,
            assets: value.assets,
            participation: value.participation,
            ordinary_rental: value.ordinary_rental,
            at_risk: value.at_risk,
            qbi_business: value.qbi_business,
            history: value.history,
            method_changes: value.method_changes,
            payroll: value.payroll,
            qbi_aggregation_group: value.qbi_aggregation_group,
            personal_use: value.personal_use,
            home_operating_carryover: value.home_operating_carryover,
            // The old zero default did not record an actual source answer.
            home_depreciation_carryover: if value.home_depreciation_carryover == 0 {
                SourceCents::Unavailable
            } else {
                SourceCents::Reported { amount: value.home_depreciation_carryover }
            },
            property_type: value.property_type,
            sale: value.sale,
            statement: value.statement,
            amt_home_depreciation_carryover: SourceCents::Unavailable,
            home_casualty_carryover: SourceCents::Unavailable,
            amt_home_casualty_carryover: SourceCents::Unavailable,
        }
    }
}

#[cfg(test)]
impl TryFrom<FederalRental> for FederalRental124 {
    type Error = &'static str;
    fn try_from(value: FederalRental) -> Result<Self, Self::Error> {
        if [
            &value.amt_home_depreciation_carryover,
            &value.home_casualty_carryover,
            &value.amt_home_casualty_carryover,
        ]
        .iter()
        .any(|source| !matches!(source, SourceCents::Unavailable))
        {
            return Err("format124 cannot retain rental carry source facts");
        }
        let regular = match value.home_depreciation_carryover {
            SourceCents::Unavailable => 0,
            SourceCents::Reported { amount } if amount > 0 => amount,
            _ => {
                return Err(
                    "format124 cannot distinguish an actual rental carry zero from its default",
                );
            }
        };
        Ok(Self {
            energy_buildings: value.energy_buildings,
            property_qbi_history: value.property_qbi_history,
            mortgage_loans: value.mortgage_loans,
            rent: value.rent,
            expenses: value.expenses,
            buildings: value.buildings,
            assets: value.assets,
            participation: value.participation,
            ordinary_rental: value.ordinary_rental,
            at_risk: value.at_risk,
            qbi_business: value.qbi_business,
            history: value.history,
            method_changes: value.method_changes,
            payroll: value.payroll,
            qbi_aggregation_group: value.qbi_aggregation_group,
            personal_use: value.personal_use,
            home_operating_carryover: value.home_operating_carryover,
            home_depreciation_carryover: regular,
            property_type: value.property_type,
            sale: value.sale,
            statement: value.statement,
        })
    }
}

/// Retained format-124 FederalIncome body.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[allow(clippy::large_enum_variant)]
pub enum FederalIncome124 {
    EmployeeWages {
        data: crate::EmployeeWages,
    },
    Partnership {
        data: crate::PartnerK1,
    },
    Scorp {
        data: crate::ScorpK1,
    },
    EstateTrustPortfolio {
        data: FederalEstateTrustPortfolioK1,
    },
    HealthLoanAssistance {
        data: crate::HealthLoanAssistance,
    },
    WildfireRelief {
        data: FederalWildfireRelief,
    },
    DisasterRelief {
        data: FederalDisasterRelief,
    },
    CasualtyLivingInsurance {
        data: FederalCasualtyLivingInsurance,
    },
    DisasterMitigation {
        data: FederalDisasterMitigation,
    },
    PersonalDebt {
        data: crate::PersonalDebtCancellation,
    },
    InheritedIra {
        data: crate::FederalInheritedIra,
    },
    Roth {
        data: crate::RothWithdrawal,
    },
    LtcBenefits {
        data: crate::FederalLtcPayment,
    },
    Able {
        data: crate::AbleDistribution,
    },
    TrumpDeath {
        data: crate::TrumpDeathAcquisition,
    },
    TrumpAbleRollover {
        data: crate::TrumpAbleRollover,
    },
    Royalty {
        data: crate::FederalRoyalty,
    },
    NonbusinessProperty {
        kind: NonbusinessPropertyKind,
        receipts: i64,
        expenses: i64,
        ordinary: bool,
    },
    SmallBusinessStock {
        data: crate::Section1244Stock,
    },
    QualifiedSmallBusinessSale {
        data: crate::QsbIssuerSale,
    },
    TaxRefund {
        amount: i64,
        history: crate::TaxRefundHistory,
        withholding: i64,
    },
    WorthlessSecurity {
        data: WorthlessSecurity,
    },
    NonbusinessBadDebt {
        data: NonbusinessBadDebt,
    },
    HomeSale {
        data: crate::HomeSale,
    },
    ReturnOfCapital {
        kind: CapitalDistributionKind,
        amount: i64,
        property: Vec<LiquidationProperty>,
        basis: i64,
        term: GainTerm,
        special: bool,
    },
    UndistributedGain {
        data: UndistributedGain,
    },
    HobbySales {
        data: crate::HobbySales,
    },
    OtherIncome {
        data: crate::OtherIncomeItem,
    },
    Nec1099 {
        data: crate::FederalForm1099Nec,
    },
    Misc1099 {
        data: crate::FederalForm1099Misc,
    },
    K1099 {
        data: crate::FederalForm1099K,
    },
    Scholarship {
        data: crate::Scholarship,
    },
    JuryDuty {
        data: JuryPay,
    },
    Gambling {
        data: FederalGambling,
    },
    Alimony {
        data: crate::AlimonyPayment,
    },
    ForeignEmployment {
        data: crate::ForeignWages,
    },
    Rental {
        data: FederalRental124,
    },
    Wage {
        data: FederalWage,
    },
    Interest {
        data: FederalInterest,
    },
    Dividend {
        data: FederalDividend,
    },
    NuaSale {
        data: crate::NuaSale,
    },
    EmployeeStockSale {
        data: crate::EmployeeStockSale,
    },
    EsppSale {
        data: crate::EsppSale,
    },
    IsoSale {
        data: crate::IsoSale,
    },
    Sale {
        data: FederalSale,
    },
    Retirement {
        data: FederalRetirement,
    },
    SocialSecurity {
        data: FederalSocial,
    },
    Unemployment {
        data: FederalUnemployment,
    },
    Business {
        data: FederalBusiness124,
    },
    Oid {
        data: FederalOid,
    },
    Cooperative {
        data: crate::FederalCooperative,
    },
    PropertyAcquisition {
        data: crate::PropertyAcquisition,
    },
    SavingsBonds {
        data: FederalSavingsBonds,
    },
    ChildElection {
        data: FederalChildElection,
    },
    Section1256 {
        data: FederalSection1256,
    },
    EducationDistribution {
        data: crate::FederalEducationDistribution,
    },
    EducationAccountExpenses {
        data: crate::FederalEducationAccountExpenses,
    },
    FarmRental {
        data: crate::FederalFarmRental,
    },
    MedicalDistribution {
        data: crate::FederalMedicalDistribution,
    },
    LtcInsured {
        data: crate::FederalLtcInsured,
    },
    OpportunityFund {
        data: crate::QofInvestment,
    },
    OpportunityFundDeferral {
        data: crate::QofDeferral,
    },
    PartnershipActivities {
        data: alloc::boxed::Box<crate::PartnerActivityK1>,
    },
    ScorpActivities {
        data: crate::ScorpActivityK1,
    },
}

/// Retained format-124 FederalBusiness body.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalBusiness124 {
    pub principal_business: String,
    pub business_code: Option<u32>,
    pub ein: String,
    pub address: String,
    pub started_current_year: bool,
    pub information_returns: InformationReturnFiling,
    pub other_income: Vec<crate::OtherBusinessIncome>,
    pub de_minimis: Option<crate::DeMinimisElection>,
    pub leased_vehicles: Vec<crate::BusinessLeasedVehicle>,
    pub depletion: Option<crate::BusinessDepletion>,
    pub ministerial: bool,
    pub notary_profit: i64,
    pub notary_gross: Option<i64>,

    pub source: BusinessSource,
    pub income_source: crate::BusinessIncomeSource,
    pub activity: BusinessActivity,
    pub inventory: crate::BusinessInventory,
    pub startup: crate::BusinessStartup,
    pub method_changes: Vec<crate::MethodChange>,
    pub intangibles: Vec<crate::BusinessIntangible>,
    pub home_office: BusinessHomeOffice124,
    pub health_insurance: crate::BusinessHealthInsurance,
    pub retirement: crate::BusinessRetirement,
    pub vehicles: Vec<crate::BusinessMileage>,
    pub assets: Vec<crate::BusinessAsset>,
    pub expenses: Vec<FederalBusinessExpense>,
    pub tips: Option<FederalBusinessTips>,
    pub overtime: Option<FederalBusinessOvertime>,
    pub fuel_tax_credit: Option<FederalFuelTaxCredit>,
    pub cash_fuel_tax_recovery: Option<FederalCashFuelRecovery>,
    pub employer_tip_fica_credit: Option<crate::EmployerTipFicaCredit>,
    pub work_opportunity_credit: Option<crate::WorkOpportunityCredit>,
    pub retirement_plan_credit: Option<crate::RetirementPlanCredit>,
    pub small_employer_health_credit: Option<crate::SmallEmployerHealthCredit>,
    pub paid_leave_credit: Option<crate::PaidLeaveCredit>,
    pub material_participation: bool,
    pub specified_service: bool,
    pub fully_at_risk: bool,
    pub payroll: crate::BusinessPayroll,
    pub special_treatment: bool,
    pub qbi_aggregation_group: Option<String>,
    pub withholding: i64,
    pub commercial_fishing: bool,
    pub sale: Option<crate::BusinessSale>,
}

/// Retained format-124 FederalDeductions body.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalDeductions124 {
    pub investment_interest: Option<crate::InvestmentBorrowing>,
    pub mortgage: Vec<crate::MortgageLoan>,
    pub repayment: crate::RepaymentClaim,
    pub annuity_losses: Vec<crate::AnnuityLoss>,
    pub choice: DeductionSelection,
    pub spouse_itemizes: bool,
    pub cash_charity: i64,
    pub fund_cash_charity: i64,
    pub other_cash_charity: i64,
    pub volunteer_expenses: Vec<crate::VolunteerExpense>,
    pub hosted_students: Vec<crate::HostedStudent>,
    pub whaling_captains: Vec<crate::WhalingCaptainClaim>,
    pub property_gifts: Vec<crate::PropertyGift>,
    pub appraisals: Vec<crate::Appraisal>,
    pub basis_election: bool,
    pub charity_carryovers: Vec<crate::CharityCarryover>,
    pub personal_casualties: Vec<PersonalCasualty124>,
    pub vehicle_loans: Vec<FederalVehicleInterest>,
}

/// Retained format-124 BusinessHomeOffice body.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessHomeOffice124 {
    None,
    Simplified { months: Vec<HomeOfficeMonth124>, prior_operating: i64, prior_depreciation: i64 },
    ActualExpenses { office: ActualOffice124, eligible: bool },
}

/// Retained format-124 HomeOfficeMonth body.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HomeOfficeMonth124 {
    pub month: u32,
    pub days: u32,
    pub square_feet: u32,
}

/// Retained format-124 OfficeHome body.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum OfficeHome124 {
    Owned {
        mortgage_interest: i64,
        mortgage_insurance: i64,
        real_estate_taxes: i64,
        basis: i64,
        land: i64,
        first_used: u32,
    },
    Rented {
        rent: i64,
    },
}

/// Retained format-124 ActualOffice body.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct ActualOffice124 {
    pub office_area: u32,
    pub home_area: u32,
    pub home: OfficeHome124,
    pub insurance: i64,
    pub repairs: i64,
    pub utilities: i64,
    pub other: i64,
    pub direct: i64,
    pub prior_operating: i64,
    pub prior_depreciation: i64,
}

/// Retained format-124 PersonalCasualty body.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PersonalCasualty124 {
    pub label: String,
    pub date: u32,
    pub declaration: CasualtyDeclaration,
    pub zip: String,
    pub properties: Vec<CasualtyProperty124>,
    pub source_complete: bool,
}

/// Retained format-124 CasualtyProperty body.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CasualtyProperty124 {
    pub label: String,
    pub location: String,
    pub acquisition: CasualtyAcquisition,
    pub basis: i64,
    pub before: i64,
    pub after: i64,
    pub reimbursement: i64,
    pub received: i64,
}

impl TryFrom<FederalBusiness124> for FederalBusiness {
    type Error = &'static str;
    fn try_from(value: FederalBusiness124) -> Result<Self, Self::Error> {
        if !matches!(value.home_office, BusinessHomeOffice124::None) {
            return Err("Legacy home-office source conversion unavailable");
        }
        Ok(Self {
            principal_business: value.principal_business,
            business_code: value.business_code,
            ein: value.ein,
            address: value.address,
            started_current_year: value.started_current_year,
            information_returns: value.information_returns,
            other_income: value.other_income,
            de_minimis: value.de_minimis,
            leased_vehicles: value.leased_vehicles,
            depletion: value.depletion,
            ministerial: value.ministerial,
            notary_profit: value.notary_profit,
            notary_gross: value.notary_gross,
            source: value.source,
            income_source: value.income_source,
            activity: value.activity,
            inventory: value.inventory,
            startup: value.startup,
            method_changes: value.method_changes,
            intangibles: value.intangibles,
            timber: Vec::new(),
            other_amortization: Vec::new(),
            health_insurance: value.health_insurance,
            retirement: value.retirement,
            vehicles: value.vehicles,
            assets: value.assets,
            expenses: value.expenses,
            tips: value.tips,
            overtime: value.overtime,
            fuel_tax_credit: value.fuel_tax_credit,
            cash_fuel_tax_recovery: value.cash_fuel_tax_recovery,
            employer_tip_fica_credit: value.employer_tip_fica_credit,
            work_opportunity_credit: value.work_opportunity_credit,
            retirement_plan_credit: value.retirement_plan_credit,
            small_employer_health_credit: value.small_employer_health_credit,
            paid_leave_credit: value.paid_leave_credit,
            material_participation: value.material_participation,
            specified_service: value.specified_service,
            at_risk: crate::RentalAtRisk::from_full_risk(value.fully_at_risk),
            payroll: value.payroll,
            special_treatment: value.special_treatment,
            qbi_aggregation_group: value.qbi_aggregation_group,
            withholding: value.withholding,
            commercial_fishing: value.commercial_fishing,
            sale: value.sale,
        })
    }
}

#[cfg(test)]
impl TryFrom<FederalBusiness> for FederalBusiness124 {
    type Error = &'static str;
    fn try_from(value: FederalBusiness) -> Result<Self, Self::Error> {
        if !value.other_amortization.is_empty() {
            return Err("format124 has no other amortization source field");
        }
        if !value.timber.is_empty() {
            return Err("format124 has no timber source field");
        }
        if matches!(&value.at_risk, crate::RentalAtRisk::Limited { record: Some(_), .. })
            || value.at_risk.prior(false) != 0
            || value.at_risk.prior(true) != 0
            || value.at_risk.prior_sale(false) != 0
            || value.at_risk.prior_sale(true) != 0
        {
            return Err("format124 has no direct business at-risk method or carry source");
        }
        let fully_at_risk = value.all_at_risk();
        Ok(Self {
            principal_business: value.principal_business,
            business_code: value.business_code,
            ein: value.ein,
            address: value.address,
            started_current_year: value.started_current_year,
            information_returns: value.information_returns,
            other_income: value.other_income,
            de_minimis: value.de_minimis,
            leased_vehicles: value.leased_vehicles,
            depletion: value.depletion,
            ministerial: value.ministerial,
            notary_profit: value.notary_profit,
            notary_gross: value.notary_gross,
            source: value.source,
            income_source: value.income_source,
            activity: value.activity,
            inventory: value.inventory,
            startup: value.startup,
            method_changes: value.method_changes,
            intangibles: value.intangibles,
            health_insurance: value.health_insurance,
            retirement: value.retirement,
            vehicles: value.vehicles,
            assets: value.assets,
            expenses: value.expenses,
            tips: value.tips,
            overtime: value.overtime,
            fuel_tax_credit: value.fuel_tax_credit,
            cash_fuel_tax_recovery: value.cash_fuel_tax_recovery,
            employer_tip_fica_credit: value.employer_tip_fica_credit,
            work_opportunity_credit: value.work_opportunity_credit,
            retirement_plan_credit: value.retirement_plan_credit,
            small_employer_health_credit: value.small_employer_health_credit,
            paid_leave_credit: value.paid_leave_credit,
            material_participation: value.material_participation,
            specified_service: value.specified_service,
            fully_at_risk,
            payroll: value.payroll,
            special_treatment: value.special_treatment,
            qbi_aggregation_group: value.qbi_aggregation_group,
            withholding: value.withholding,
            commercial_fishing: value.commercial_fishing,
            sale: value.sale,
            home_office: BusinessHomeOffice124::None,
        })
    }
}

impl TryFrom<FederalIncome124> for FederalIncome {
    type Error = &'static str;
    fn try_from(value: FederalIncome124) -> Result<Self, Self::Error> {
        Ok(match value {
            FederalIncome124::EmployeeWages { data } => Self::EmployeeWages { data },
            FederalIncome124::Partnership { data } => Self::Partnership { data },
            FederalIncome124::Scorp { data } => Self::Scorp { data },
            FederalIncome124::EstateTrustPortfolio { data } => Self::EstateTrustPortfolio { data },
            FederalIncome124::HealthLoanAssistance { data } => Self::HealthLoanAssistance { data },
            FederalIncome124::WildfireRelief { data } => Self::WildfireRelief { data },
            FederalIncome124::DisasterRelief { data } => Self::DisasterRelief { data },
            FederalIncome124::CasualtyLivingInsurance { data } => {
                Self::CasualtyLivingInsurance { data }
            }
            FederalIncome124::DisasterMitigation { data } => Self::DisasterMitigation { data },
            FederalIncome124::PersonalDebt { data } => Self::PersonalDebt { data },
            FederalIncome124::InheritedIra { data } => Self::InheritedIra { data },
            FederalIncome124::Roth { data } => Self::Roth { data },
            FederalIncome124::LtcBenefits { data } => Self::LtcBenefits { data },
            FederalIncome124::Able { data } => Self::Able { data },
            FederalIncome124::TrumpDeath { data } => Self::TrumpDeath { data },
            FederalIncome124::TrumpAbleRollover { data } => Self::TrumpAbleRollover { data },
            FederalIncome124::Royalty { data } => Self::Royalty { data },
            FederalIncome124::NonbusinessProperty { kind, receipts, expenses, ordinary } => {
                Self::NonbusinessProperty { kind, receipts, expenses, ordinary }
            }
            FederalIncome124::SmallBusinessStock { data } => Self::SmallBusinessStock { data },
            FederalIncome124::QualifiedSmallBusinessSale { data } => {
                Self::QualifiedSmallBusinessSale { data }
            }
            FederalIncome124::TaxRefund { amount, history, withholding } => {
                Self::TaxRefund { amount, history, withholding }
            }
            FederalIncome124::WorthlessSecurity { data } => Self::WorthlessSecurity { data },
            FederalIncome124::NonbusinessBadDebt { data } => Self::NonbusinessBadDebt { data },
            FederalIncome124::HomeSale { data } => Self::HomeSale { data },
            FederalIncome124::ReturnOfCapital { kind, amount, property, basis, term, special } => {
                Self::ReturnOfCapital { kind, amount, property, basis, term, special }
            }
            FederalIncome124::UndistributedGain { data } => Self::UndistributedGain { data },
            FederalIncome124::HobbySales { data } => Self::HobbySales { data },
            FederalIncome124::OtherIncome { data } => Self::OtherIncome { data },
            FederalIncome124::Nec1099 { data } => Self::Nec1099 { data },
            FederalIncome124::Misc1099 { data } => Self::Misc1099 { data },
            FederalIncome124::K1099 { data } => Self::K1099 { data },
            FederalIncome124::Scholarship { data } => Self::Scholarship { data },
            FederalIncome124::JuryDuty { data } => Self::JuryDuty { data },
            FederalIncome124::Gambling { data } => Self::Gambling { data },
            FederalIncome124::Alimony { data } => Self::Alimony { data },
            FederalIncome124::ForeignEmployment { data } => Self::ForeignEmployment { data },
            FederalIncome124::Rental { data } => Self::Rental { data: data.into() },
            FederalIncome124::Wage { data } => Self::Wage { data },
            FederalIncome124::Interest { data } => Self::Interest { data },
            FederalIncome124::Dividend { data } => Self::Dividend { data },
            FederalIncome124::NuaSale { data } => Self::NuaSale { data },
            FederalIncome124::EmployeeStockSale { data } => Self::EmployeeStockSale { data },
            FederalIncome124::EsppSale { data } => Self::EsppSale { data },
            FederalIncome124::IsoSale { data } => Self::IsoSale { data },
            FederalIncome124::Sale { data } => Self::Sale { data },
            FederalIncome124::Retirement { data } => Self::Retirement { data },
            FederalIncome124::SocialSecurity { data } => Self::SocialSecurity { data },
            FederalIncome124::Unemployment { data } => Self::Unemployment { data },
            FederalIncome124::Business { data } => Self::Business { data: data.try_into()? },
            FederalIncome124::Oid { data } => Self::Oid { data },
            FederalIncome124::Cooperative { data } => Self::Cooperative { data },
            FederalIncome124::PropertyAcquisition { data } => Self::PropertyAcquisition { data },
            FederalIncome124::SavingsBonds { data } => Self::SavingsBonds { data },
            FederalIncome124::ChildElection { data } => Self::ChildElection { data },
            FederalIncome124::Section1256 { data } => Self::Section1256 { data },
            FederalIncome124::EducationDistribution { data } => {
                Self::EducationDistribution { data }
            }
            FederalIncome124::EducationAccountExpenses { data } => {
                Self::EducationAccountExpenses { data }
            }
            FederalIncome124::FarmRental { data } => Self::FarmRental { data },
            FederalIncome124::MedicalDistribution { data } => Self::MedicalDistribution { data },
            FederalIncome124::LtcInsured { data } => Self::LtcInsured { data },
            FederalIncome124::OpportunityFund { data } => Self::OpportunityFund { data },
            FederalIncome124::OpportunityFundDeferral { data } => {
                Self::OpportunityFundDeferral { data }
            }
            FederalIncome124::PartnershipActivities { data } => {
                Self::PartnershipActivities { data }
            }
            FederalIncome124::ScorpActivities { data } => Self::ScorpActivities { data },
        })
    }
}

#[cfg(test)]
impl TryFrom<FederalIncome> for FederalIncome124 {
    type Error = &'static str;
    fn try_from(value: FederalIncome) -> Result<Self, Self::Error> {
        Ok(match value {
            FederalIncome::EmployeeWages { data } => Self::EmployeeWages { data },
            FederalIncome::Partnership { data } => Self::Partnership { data },
            FederalIncome::Scorp { data } => Self::Scorp { data },
            FederalIncome::EstateTrustPortfolio { data } => Self::EstateTrustPortfolio { data },
            FederalIncome::HealthLoanAssistance { data } => Self::HealthLoanAssistance { data },
            FederalIncome::WildfireRelief { data } => Self::WildfireRelief { data },
            FederalIncome::DisasterRelief { data } => Self::DisasterRelief { data },
            FederalIncome::CasualtyLivingInsurance { data } => {
                Self::CasualtyLivingInsurance { data }
            }
            FederalIncome::DisasterMitigation { data } => Self::DisasterMitigation { data },
            FederalIncome::PersonalDebt { data } => Self::PersonalDebt { data },
            FederalIncome::InheritedIra { data } => Self::InheritedIra { data },
            FederalIncome::Roth { data } => Self::Roth { data },
            FederalIncome::LtcBenefits { data } => Self::LtcBenefits { data },
            FederalIncome::Able { data } => Self::Able { data },
            FederalIncome::TrumpDeath { data } => Self::TrumpDeath { data },
            FederalIncome::TrumpAbleRollover { data } => Self::TrumpAbleRollover { data },
            FederalIncome::Royalty { data } => Self::Royalty { data },
            FederalIncome::NonbusinessProperty { kind, receipts, expenses, ordinary } => {
                Self::NonbusinessProperty { kind, receipts, expenses, ordinary }
            }
            FederalIncome::SmallBusinessStock { data } => Self::SmallBusinessStock { data },
            FederalIncome::QualifiedSmallBusinessSale { data } => {
                Self::QualifiedSmallBusinessSale { data }
            }
            FederalIncome::TaxRefund { amount, history, withholding } => {
                Self::TaxRefund { amount, history, withholding }
            }
            FederalIncome::WorthlessSecurity { data } => Self::WorthlessSecurity { data },
            FederalIncome::NonbusinessBadDebt { data } => Self::NonbusinessBadDebt { data },
            FederalIncome::HomeSale { data } => Self::HomeSale { data },
            FederalIncome::ReturnOfCapital { kind, amount, property, basis, term, special } => {
                Self::ReturnOfCapital { kind, amount, property, basis, term, special }
            }
            FederalIncome::UndistributedGain { data } => Self::UndistributedGain { data },
            FederalIncome::HobbySales { data } => Self::HobbySales { data },
            FederalIncome::OtherIncome { data } => Self::OtherIncome { data },
            FederalIncome::Nec1099 { data } => Self::Nec1099 { data },
            FederalIncome::Misc1099 { data } => Self::Misc1099 { data },
            FederalIncome::K1099 { data } => Self::K1099 { data },
            FederalIncome::Scholarship { data } => Self::Scholarship { data },
            FederalIncome::JuryDuty { data } => Self::JuryDuty { data },
            FederalIncome::Gambling { data } => Self::Gambling { data },
            FederalIncome::Alimony { data } => Self::Alimony { data },
            FederalIncome::ForeignEmployment { data } => Self::ForeignEmployment { data },
            FederalIncome::Rental { data } => Self::Rental { data: data.try_into()? },
            FederalIncome::Wage { data } => Self::Wage { data },
            FederalIncome::Interest { data } => Self::Interest { data },
            FederalIncome::Dividend { data } => Self::Dividend { data },
            FederalIncome::NuaSale { data } => Self::NuaSale { data },
            FederalIncome::EmployeeStockSale { data } => Self::EmployeeStockSale { data },
            FederalIncome::EsppSale { data } => Self::EsppSale { data },
            FederalIncome::IsoSale { data } => Self::IsoSale { data },
            FederalIncome::Sale { data } => Self::Sale { data },
            FederalIncome::Retirement { data } => Self::Retirement { data },
            FederalIncome::SocialSecurity { data } => Self::SocialSecurity { data },
            FederalIncome::Unemployment { data } => Self::Unemployment { data },
            FederalIncome::Business { data } => Self::Business { data: data.try_into()? },
            FederalIncome::Oid { data } => Self::Oid { data },
            FederalIncome::Cooperative { data } => Self::Cooperative { data },
            FederalIncome::PropertyAcquisition { data } => Self::PropertyAcquisition { data },
            FederalIncome::SavingsBonds { data } => Self::SavingsBonds { data },
            FederalIncome::ChildElection { data } => Self::ChildElection { data },
            FederalIncome::Section1256 { data } => Self::Section1256 { data },
            FederalIncome::EducationDistribution { data } => Self::EducationDistribution { data },
            FederalIncome::EducationAccountExpenses { data } => {
                Self::EducationAccountExpenses { data }
            }
            FederalIncome::FarmRental { data } => Self::FarmRental { data },
            FederalIncome::MedicalDistribution { data } => Self::MedicalDistribution { data },
            FederalIncome::LtcInsured { data } => Self::LtcInsured { data },
            FederalIncome::OpportunityFund { data } => Self::OpportunityFund { data },
            FederalIncome::OpportunityFundDeferral { data } => {
                Self::OpportunityFundDeferral { data }
            }
            FederalIncome::PartnershipActivities { data } => Self::PartnershipActivities { data },
            FederalIncome::ScorpActivities { data } => Self::ScorpActivities { data },
        })
    }
}

impl TryFrom<FederalIncomeEntry124> for FederalIncomeEntry {
    type Error = &'static str;
    fn try_from(value: FederalIncomeEntry124) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            label: value.label,
            owner: value.owner,
            community: value.community,
            income: value.income.try_into()?,
            state_source: value.state_source,
            refund_management: value.refund_management,
        })
    }
}

#[cfg(test)]
impl TryFrom<FederalIncomeEntry> for FederalIncomeEntry124 {
    type Error = &'static str;
    fn try_from(value: FederalIncomeEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            label: value.label,
            owner: value.owner,
            community: value.community,
            income: value.income.try_into()?,
            state_source: value.state_source,
            refund_management: value.refund_management,
        })
    }
}

impl From<FederalDeductions124> for FederalDeductions {
    fn from(value: FederalDeductions124) -> Self {
        Self {
            investment_interest: value.investment_interest,
            mortgage: value.mortgage,
            repayment: value.repayment,
            annuity_losses: value.annuity_losses,
            choice: value.choice,
            spouse_itemizes: value.spouse_itemizes,
            cash_charity: value.cash_charity,
            fund_cash_charity: value.fund_cash_charity,
            other_cash_charity: value.other_cash_charity,
            volunteer_expenses: value.volunteer_expenses,
            hosted_students: value.hosted_students,
            whaling_captains: value.whaling_captains,
            property_gifts: value.property_gifts,
            appraisals: value.appraisals,
            basis_election: value.basis_election,
            charity_carryovers: value.charity_carryovers,
            personal_casualties: value.personal_casualties.into_iter().map(Into::into).collect(),
            vehicle_loans: value.vehicle_loans,
        }
    }
}

impl From<PersonalCasualty124> for PersonalCasualty {
    fn from(value: PersonalCasualty124) -> Self {
        Self {
            id: 0,
            label: value.label,
            date: value.date,
            declaration: value.declaration,
            zip: value.zip,
            properties: value.properties.into_iter().map(Into::into).collect(),
            source_complete: value.source_complete,
        }
    }
}

impl From<CasualtyProperty124> for CasualtyProperty {
    fn from(value: CasualtyProperty124) -> Self {
        Self {
            id: 0,
            home_site: None,
            business_damage: None,
            label: value.label,
            location: value.location,
            acquisition: value.acquisition,
            basis: value.basis,
            before: value.before,
            after: value.after,
            reimbursement: value.reimbursement,
            received: value.received,
        }
    }
}

#[cfg(test)]
impl TryFrom<FederalDeductions> for FederalDeductions124 {
    type Error = &'static str;
    fn try_from(value: FederalDeductions) -> Result<Self, Self::Error> {
        Ok(Self {
            investment_interest: value.investment_interest,
            mortgage: value.mortgage,
            repayment: value.repayment,
            annuity_losses: value.annuity_losses,
            choice: value.choice,
            spouse_itemizes: value.spouse_itemizes,
            cash_charity: value.cash_charity,
            fund_cash_charity: value.fund_cash_charity,
            other_cash_charity: value.other_cash_charity,
            volunteer_expenses: value.volunteer_expenses,
            hosted_students: value.hosted_students,
            whaling_captains: value.whaling_captains,
            property_gifts: value.property_gifts,
            appraisals: value.appraisals,
            basis_election: value.basis_election,
            charity_carryovers: value.charity_carryovers,
            personal_casualties: value
                .personal_casualties
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            vehicle_loans: value.vehicle_loans,
        })
    }
}

#[cfg(test)]
impl TryFrom<PersonalCasualty> for PersonalCasualty124 {
    type Error = &'static str;
    fn try_from(value: PersonalCasualty) -> Result<Self, Self::Error> {
        if value.id != 0 {
            return Err("Current casualty facts cannot be retained in format 124");
        }
        Ok(Self {
            label: value.label,
            date: value.date,
            declaration: value.declaration,
            zip: value.zip,
            properties: value
                .properties
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_, _>>()?,
            source_complete: value.source_complete,
        })
    }
}

#[cfg(test)]
impl TryFrom<CasualtyProperty> for CasualtyProperty124 {
    type Error = &'static str;
    fn try_from(value: CasualtyProperty) -> Result<Self, Self::Error> {
        if value.id != 0 || value.home_site.is_some() || value.business_damage.is_some() {
            return Err("Current casualty facts cannot be retained in format 124");
        }
        Ok(Self {
            label: value.label,
            location: value.location,
            acquisition: value.acquisition,
            basis: value.basis,
            before: value.before,
            after: value.after,
            reimbursement: value.reimbursement,
            received: value.received,
        })
    }
}

// Retained format-124 payments: no next-year election was asked or recorded.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct FederalPayments124 {
    pub estimated: Option<i64>,
    pub extension: i64,
    pub home_kerosene: Option<FederalHomeKerosene>,
    pub estimated_payments: Vec<crate::DatedPayment>,
    pub prior_year: Option<crate::PriorYearTax>,
    pub withholding_by_date: bool,
    pub withholding_periods: Option<crate::PeriodWithholding>,
    pub annualized: bool,
    pub annualized_income: Option<crate::AnnualizedIncome>,
    pub farmer_fisherman_prior_year: bool,
    pub farmer_paid_by_march: bool,
    pub waiver: Option<crate::PenaltyWaiver>,
    pub next_year: NextYearEstimate,
    pub joint: bool,
    pub extension_payer: Option<crate::PaymentPayer>,
    pub extension_refund_property: Option<crate::RefundSourcePropertyFacts>,
}
impl From<FederalPayments124> for FederalPayments {
    fn from(value: FederalPayments124) -> Self {
        Self {
            estimated: value.estimated,
            extension: value.extension,
            home_kerosene: value.home_kerosene,
            estimated_payments: value.estimated_payments,
            prior_year: value.prior_year,
            withholding_by_date: value.withholding_by_date,
            withholding_periods: value.withholding_periods,
            annualized: value.annualized,
            annualized_income: value.annualized_income,
            farmer_fisherman_prior_year: value.farmer_fisherman_prior_year,
            farmer_paid_by_march: value.farmer_paid_by_march,
            waiver: value.waiver,
            next_year: value.next_year.into(),
            joint: value.joint,
            extension_payer: value.extension_payer,
            extension_refund_property: value.extension_refund_property,
            credit_applied_next_year: 0,
            credit_recipient: None,
        }
    }
}

#[cfg(test)]
impl TryFrom<FederalPayments> for FederalPayments124 {
    type Error = &'static str;
    fn try_from(value: FederalPayments) -> Result<Self, Self::Error> {
        if value.credit_applied_next_year != 0 || value.credit_recipient.is_some() {
            return Err("A current planned credit cannot be represented by format 124");
        }
        Ok(Self {
            estimated: value.estimated,
            extension: value.extension,
            home_kerosene: value.home_kerosene,
            estimated_payments: value.estimated_payments,
            prior_year: value.prior_year,
            withholding_by_date: value.withholding_by_date,
            withholding_periods: value.withholding_periods,
            annualized: value.annualized,
            annualized_income: value.annualized_income,
            farmer_fisherman_prior_year: value.farmer_fisherman_prior_year,
            farmer_paid_by_march: value.farmer_paid_by_march,
            waiver: value.waiver,
            next_year: value.next_year.into(),
            joint: value.joint,
            extension_payer: value.extension_payer,
            extension_refund_property: value.extension_refund_property,
        })
    }
}
