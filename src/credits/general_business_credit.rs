pub(crate) use crate::business_credit_source::ConditionalCarrybackOrigin;
pub use crate::business_credit_source::{
    BusinessCredit, BusinessCreditCarryback, BusinessCreditCarryforward, BusinessCreditHistory,
    CarbonOxideCreditOrigin, CarbonOxideEquipmentCredit, CarryFiler, CreditElection, CreditSource,
    EmployerTipFicaCarryforward, EnteredBusinessCredit, PassiveWorksheet, RenewableFacilityCredit,
    RenewableFacilityCreditOrigin,
};
use crate::business_credit_source::{CreditOriginPart, valid_origin_details};

use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::{form_line, line, line_cents};
use crate::validation::ValidationIssue;
use crate::year_constants::{
    PAID_LEAVE_BASE_PERCENT_HUNDREDTHS, PAID_LEAVE_COMPENSATION_LIMIT_2026_CENTS,
    PAID_LEAVE_MAX_PERCENT_HUNDREDTHS, PAID_LEAVE_MAX_WEEKS, PAID_LEAVE_MIN_RATE_HUNDREDTHS,
    PAID_LEAVE_MIN_WEEKLY_HOURS, PAID_LEAVE_RATE_POINTS_PER_PERCENT_POINT,
    SMALL_EMPLOYER_HEALTH_CREDIT_PERCENT, SMALL_EMPLOYER_HEALTH_FTE_HOURS,
    SMALL_EMPLOYER_HEALTH_FTE_LIMIT, SMALL_EMPLOYER_HEALTH_FTE_PHASEOUT_RANGE,
    SMALL_EMPLOYER_HEALTH_FTE_PHASEOUT_START, SMALL_EMPLOYER_HEALTH_WAGE_AMOUNT_DOLLARS,
    SMALL_EMPLOYER_HEALTH_WAGE_ROUNDING_DOLLARS,
};
use crate::{BusinessPayroll, FederalResult, FederalStatus, FormLine, TaxForm};
use alloc::{string::String, vec::Vec};
use core::cmp::{max, min};
use libertas_macros::*;

/// One calendar month of eligible-service wages and customer tips for one employee
/// Enter all twelve months in calendar order, including zero months. The wages exclude tips but include all this employer's Medicare-taxable cash pay for the month; tips are cash or charged customer tips reported to this employer and subject to its Social Security and Medicare taxes. Exclude mandatory service charges and unreported tips.
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployerTipMonth {
    /// Time worked
    /// Form 8846 compares monthly wages with $5.15 per hour for food or beverages, or $7.25 per hour for qualifying beauty services.
    #[libertas_number(min = 0, max = 1000, step = 0.01)]
    #[libertas_physical_unit("hour")]
    pub hours: f64,
    /// Medicare-taxable wages paid by this employer before tips
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wages_excluding_tips: i64,
    /// Customer tips reported to this employer and taxed for employer FICA
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub reported_tips: i64,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for EmployerTipMonth {}

/// One tipped employee's 2026 payroll record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployerTipEmployee {
    /// Employee's W-2
    /// Choose this employee's W-2 in this business's Employee payroll. Enter a missing W-2 under Employees' W-2 wages first.
    pub label: String,
    /// January through December 2026 payroll records
    // Tax structure: one entry for each of the twelve calendar months.
    #[libertas_size(min = 12, max = 12)]
    /// ----
    /// Calendar month
    pub months: Vec<EmployerTipMonth>,
}

/// One tipped employee's 2026 payroll record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployerTipEmployeePage {
    /// Employee's W-2
    /// Choose this employee's W-2 in this business's Employee payroll. Enter a missing W-2 under Employees' W-2 wages first.
    #[libertas_enum_source("^^^^.payroll.Yes.statements")]
    pub label: String,
    /// January through December 2026 payroll records
    // Tax structure: one entry for each of the twelve calendar months.
    #[libertas_size(min = 12, max = 12)]
    /// ----
    /// Calendar month
    pub months: Vec<EmployerTipMonth>,
}

impl From<&EmployerTipEmployee> for EmployerTipEmployeePage {
    fn from(value: &EmployerTipEmployee) -> Self {
        Self { label: value.label.clone(), months: value.months.clone() }
    }
}
impl From<&EmployerTipEmployeePage> for EmployerTipEmployee {
    fn from(value: &EmployerTipEmployeePage) -> Self {
        Self { label: value.label.clone(), months: value.months.clone() }
    }
}

/// Services generating this employer's reported customer tips
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum EmployerTipService {
    /// Providing, delivering, or serving food or beverages for consumption
    FoodOrBeverage,
    /// Customarily tipped eligible beauty services
    /// Select every category generating the entered tips. At least one must be selected, and every entered tip must come from these categories. Unlisted tipped services need separate review.
    BeautyServices {
        /// Barbering or hair care
        barbering_and_hair_care: bool,
        /// Nail care
        nail_care: bool,
        /// Esthetics
        esthetics: bool,
        /// Body or spa treatment
        body_and_spa_treatment: bool,
        /// All entered tips came from the selected eligible services
        /// The business's employee tip reports by service show it. No leaves this credit out of this estimate.
        eligible_tips_only: bool,
    },
    /// Another business or an unresolved service category
    Other,
}

impl EmployerTipService {
    fn basis_cents_per_hour(&self) -> Option<i64> {
        match self {
            Self::FoodOrBeverage => Some(515),
            Self::BeautyServices {
                barbering_and_hair_care,
                nail_care,
                esthetics,
                body_and_spa_treatment,
                eligible_tips_only: true,
            } if *barbering_and_hair_care
                || *nail_care
                || *esthetics
                || *body_and_spa_treatment =>
            {
                Some(725)
            }
            _ => None,
        }
    }
}

/// Current-year Form 8846 for one active Schedule C employer
/// This source computes Form 8846 lines 1–4 and feeds Form 3800 Part III line 4f. Enter every tipped employee and all twelve months. Do not also include the employer FICA on these reported tips in Business expenses; the App deducts only the part not credited. Include other payroll wages and taxes in ordinary business expenses once.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployerTipFicaCredit {
    /// Service category that generated these customer tips
    /// Food and beverage employers use the January 1, 2007 $5.15 hourly basis. Qualifying beauty-service employers use the 2026 $7.25 hourly basis. Both require customary customer tipping and employer FICA actually paid or incurred on the entered tips.
    pub service: EmployerTipService,
    /// Tipped employees on this business's 2026 Forms W-2
    #[libertas_size(min = 1)]
    /// ----
    /// Employee payroll and tip record
    pub employees: Vec<EmployerTipEmployee>,
    /// Actual employer FICA paid or incurred on these reported customer tips
    /// Reconcile the employer Social Security and Medicare tax on these tips to payroll records and Forms 941, including payroll-period rounding. Enter cents. Exclude employee tax, Additional Medicare Tax, non-tip wages, unreported tips, and mandatory service charges. Ordinary Business expenses must omit this amount: the App deducts it after reducing it by Form 8846 line 4.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_employer_tip_fica_paid: i64,
    /// Current-year unused credit actually used on a 2025 carryback return
    /// Enter whole dollars from the retained 2025 Form 3800 carryback worksheet and Form 1040-X or Form 1045, or zero after confirming no 2025 capacity or another valid reason for no use. The App calculates 2026 tax use first; only the remaining 2026 credit is available for a 2025 carryback, then a 2027 carryforward.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub used_prior_year_carryback: i64,
    /// Confirm eligible service, payroll and carryback records
    /// This is one directly owned, actively operated U.S. Schedule C employer. All entered tips came from customer services in the selected eligible food/beverage or beauty categories where tipping is customary. Every tipped employee and monthly wage/tip report is entered. The employee reports and employer FICA payments were reconciled to payroll returns and W-2 box 5; no employee exceeds this employer's 2026 Social Security wage base, no predecessor or successor payroll, no service charge, no pass-through or passive credit, and no credit election or other Form 3800 amount is omitted. Ordinary expenses exclude employer FICA on these tips but include all other payroll costs once. The retained 2025 return supports the entered carryback use, including zero. No leaves this credit out of this estimate.
    pub confirmed: bool,
}

/// Current-year Form 8846 for one active Schedule C employer
/// This source computes Form 8846 lines 1–4 and feeds Form 3800 Part III line 4f. Enter every tipped employee and all twelve months. Do not also include the employer FICA on these reported tips in Business expenses; the App deducts only the part not credited. Include other payroll wages and taxes in ordinary business expenses once.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployerTipFicaCreditPage {
    /// Service category that generated these customer tips
    /// Food and beverage employers use the January 1, 2007 $5.15 hourly basis. Qualifying beauty-service employers use the 2026 $7.25 hourly basis. Both require customary customer tipping and employer FICA actually paid or incurred on the entered tips.
    pub service: EmployerTipService,
    /// Tipped employees on this business's 2026 Forms W-2
    #[libertas_size(min = 1)]
    /// ----
    /// Employee payroll and tip record
    pub employees: Vec<EmployerTipEmployeePage>,
    /// Actual employer FICA paid or incurred on these reported customer tips
    /// Reconcile the employer Social Security and Medicare tax on these tips to payroll records and Forms 941, including payroll-period rounding. Enter cents. Exclude employee tax, Additional Medicare Tax, non-tip wages, unreported tips, and mandatory service charges. Ordinary Business expenses must omit this amount: the App deducts it after reducing it by Form 8846 line 4.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub actual_employer_tip_fica_paid: i64,
    /// Current-year unused credit actually used on a 2025 carryback return
    /// Enter whole dollars from the retained 2025 Form 3800 carryback worksheet and Form 1040-X or Form 1045, or zero after confirming no 2025 capacity or another valid reason for no use. The App calculates 2026 tax use first; only the remaining 2026 credit is available for a 2025 carryback, then a 2027 carryforward.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub used_prior_year_carryback: i64,
    /// Confirm eligible service, payroll and carryback records
    /// This is one directly owned, actively operated U.S. Schedule C employer. All entered tips came from customer services in the selected eligible food/beverage or beauty categories where tipping is customary. Every tipped employee and monthly wage/tip report is entered. The employee reports and employer FICA payments were reconciled to payroll returns and W-2 box 5; no employee exceeds this employer's 2026 Social Security wage base, no predecessor or successor payroll, no service charge, no pass-through or passive credit, and no credit election or other Form 3800 amount is omitted. Ordinary expenses exclude employer FICA on these tips but include all other payroll costs once. The retained 2025 return supports the entered carryback use, including zero. No leaves this credit out of this estimate.
    pub confirmed: bool,
}

impl From<&EmployerTipFicaCredit> for EmployerTipFicaCreditPage {
    fn from(value: &EmployerTipFicaCredit) -> Self {
        Self {
            service: value.service.clone(),
            employees: value.employees.iter().map(Into::into).collect(),
            actual_employer_tip_fica_paid: value.actual_employer_tip_fica_paid,
            used_prior_year_carryback: value.used_prior_year_carryback,
            confirmed: value.confirmed,
        }
    }
}
impl From<&EmployerTipFicaCreditPage> for EmployerTipFicaCredit {
    fn from(value: &EmployerTipFicaCreditPage) -> Self {
        Self {
            service: value.service.clone(),
            employees: value.employees.iter().map(Into::into).collect(),
            actual_employer_tip_fica_paid: value.actual_employer_tip_fica_paid,
            used_prior_year_carryback: value.used_prior_year_carryback,
            confirmed: value.confirmed,
        }
    }
}

impl EmployerTipFicaCredit {
    pub(crate) fn valid(&self) -> bool {
        !self.employees.is_empty()
            && self.employees.len() <= 100
            && (0..=100_000_000_000).contains(&self.actual_employer_tip_fica_paid)
            && (0..=100_000_000_000).contains(&self.used_prior_year_carryback)
            && self.used_prior_year_carryback % 100 == 0
            && self.employees.iter().enumerate().all(|(index, employee)| {
                !employee.label.trim().is_empty()
                    && !self.employees[..index].iter().any(|prior| prior.label == employee.label)
                    && employee.months.len() == 12
                    && employee.months.iter().all(|month| {
                        crate::kernel::natural::exact(month.hours, 100)
                            .is_some_and(|hours| hours <= 100_000)
                            && (0..=100_000_000_000).contains(&month.wages_excluding_tips)
                            && (0..=100_000_000_000).contains(&month.reported_tips)
                    })
            })
    }

    pub(crate) fn qualified(&self, payroll: &BusinessPayroll) -> bool {
        let BusinessPayroll::Yes { statements, .. } = payroll else {
            return false;
        };
        if !self.valid()
            || self.service.basis_cents_per_hour().is_none()
            || !self.confirmed
            || self.actual_employer_tip_fica_paid < self.credit() * 100
        {
            return false;
        }
        self.employees.iter().all(|employee| {
            let matching: Vec<_> =
                statements.iter().filter(|statement| statement.label == employee.label).collect();
            let wages: i64 = employee.months.iter().map(|month| month.wages_excluding_tips).sum();
            let tips: i64 = employee.months.iter().map(|month| month.reported_tips).sum();
            matching.len() == 1
                && tips > 0
                && wages + tips == matching[0].medicare
                && wages + tips <= crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS
                && employee.months.iter().all(|month| {
                    month.reported_tips == 0 || crate::kernel::natural::units(month.hours, 100) > 0
                })
        })
    }

    fn form_lines(&self) -> (i64, i64, i64, i64) {
        let basis_cents_per_hour = self.service.basis_cents_per_hour().unwrap_or(0);
        let total_tips: i64 =
            self.employees.iter().flat_map(|e| &e.months).map(|m| m.reported_tips).sum();
        let noncreditable: i64 = self
            .employees
            .iter()
            .flat_map(|e| &e.months)
            .map(|month| {
                // Section 45B uses $5.15 for food/beverage or $7.25 for
                // eligible beauty service. The shortfall is monthly and cannot
                // exclude more than that month's taxed customer tips.
                let minimum_wages = crate::kernel::ratio::half_offset_i64(
                    crate::kernel::natural::units(month.hours, 100) * basis_cents_per_hour,
                    100,
                );
                phase::cap(
                    month.reported_tips,
                    [positive(minimum_wages - month.wages_excluding_tips)],
                )
            })
            .sum();
        let line1 = dollars(total_tips);
        let line2 = dollars(noncreditable);
        let line3 = line1 - line2;
        let line4 = fraction(line3, 765, 10_000);
        (line1, line2, line3, line4)
    }

    pub(crate) fn credit(&self) -> i64 {
        self.form_lines().3
    }

    pub(crate) fn remaining_fica_deduction_cents(&self) -> i64 {
        positive(self.actual_employer_tip_fica_paid - self.credit() * 100)
    }

    pub(crate) fn report(&self, business_id: i64, r: &mut FederalResult) {
        let (line1, line2, line3, line4) = self.form_lines();
        let explanation_key = match &self.service {
            EmployerTipService::FoodOrBeverage => "EXPLAIN_FORM8846_FOOD_HOURLY_BASIS",
            EmployerTipService::BeautyServices { .. } => "EXPLAIN_FORM8846_BEAUTY_HOURLY_BASIS",
            EmployerTipService::Other => "EXPLAIN_FORM8846_UNRESOLVED_HOURLY_BASIS",
        };
        let basis = self.service.basis_cents_per_hour().unwrap_or(0);
        emit_lines(
            r,
            &mut (),
            [
                ("1", line1, msg!("EXPLAIN_BUSINESS_N_FORM8846_N_LINE1")),
                ("2", line2, msg!("EXPLAIN_BUSINESS_N_FORM8846_N_LINE2")),
                ("3", line3, msg!("EXPLAIN_BUSINESS_N_FORM8846_N_LINE3")),
                ("4", line4, msg!("EXPLAIN_BUSINESS_N_FORM8846_N_LINE4")),
            ],
            |(printed, amount, explanation), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form8846, printed)),
                    &alloc::format!("business_{business_id}_form8846_line{printed}"),
                    LineValue::MoneyDollars(amount),
                    &["income.business.employer_tip_fica_credit", "income.business.payroll"],
                    "https://www.irs.gov/pub/irs-dft/i8846--dft.pdf",
                    explanation,
                ))
            },
        );
        line_cents(
            r,
            &alloc::format!("business_{business_id}_form8846_hourly_basis"),
            basis,
            &["income.business.employer_tip_fica_credit.service"],
            "https://www.irs.gov/pub/irs-dft/i8846--dft.pdf",
            crate::Message::new(explanation_key, alloc::vec![crate::MessageArg::usd_cents(basis)]),
        );
        emit_lines(
            r,
            &mut (),
            [
                (
                    "actual_employer_tip_fica_paid",
                    self.actual_employer_tip_fica_paid,
                    msg!("EXPLAIN_BUSINESS_N_FORM8846_N_ACTUAL_EMPLOYER_TIP_FICA_PAID"),
                ),
                (
                    "remaining_employer_fica_deduction",
                    self.remaining_fica_deduction_cents(),
                    msg!("EXPLAIN_BUSINESS_N_FORM8846_N_REMAINING_EMPLOYER_FICA_DEDUCTION"),
                ),
            ],
            |(suffix, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("business_{business_id}_form8846_{suffix}"),
                    LineValue::MoneyCents(amount),
                    &["income.business.employer_tip_fica_credit", "income.business.payroll"],
                    "https://www.irs.gov/pub/irs-dft/i8846--dft.pdf",
                    explanation,
                ))
            },
        );
    }
}

/// Credit code on the K-1
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum K1CreditCode {
    /// Code J work opportunity credit
    WorkOpportunity,
    /// Code N credit for employer Social Security and Medicare taxes
    EmployerTipFica,
    /// Code AE small employer pension plan start-up costs and contributions credit
    PensionStartup,
    /// Code AF small employer auto-enrollment credit
    AutoEnrollment,
    /// Code AG small employer military spouse participation credit
    MilitarySpouse,
    /// Code BA small employer health insurance premiums credit
    SmallEmployerHealth,
    /// Code BB employer credit for paid family and medical leave
    PaidLeave,
    /// Code C low-income housing credit (section 42(j)(5))
    LowIncomeHousingJ5,
    /// Code D low-income housing credit (other)
    LowIncomeHousingOther,
    /// Code A ZeroEmissionNuclear
    ZeroEmissionNuclear,
    /// Code B AdvancedNuclear
    AdvancedNuclear,
    /// Code F OtherRentalRealEstate
    OtherRentalRealEstate,
    /// Code G OtherRental
    OtherRental,
    /// Code K DisabledAccess
    DisabledAccess,
    /// Code L EmpowermentZone
    EmpowermentZone,
    /// Code M Research
    Research,
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
    RenewableElectricity,
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
    Transferred,
    /// Code ZZ Other
    Other,
}

impl K1CreditCode {
    /// Every code the App reads, the one catalog: `K1Credit::valid_list`, the
    /// four `max =` bounds and the texts that list the codes follow it (a test
    /// compares each).
    pub(crate) const ALL: [Self; 44] = [
        Self::WorkOpportunity,
        Self::EmployerTipFica,
        Self::PensionStartup,
        Self::AutoEnrollment,
        Self::MilitarySpouse,
        Self::SmallEmployerHealth,
        Self::PaidLeave,
        Self::LowIncomeHousingJ5,
        Self::LowIncomeHousingOther,
        Self::ZeroEmissionNuclear,
        Self::AdvancedNuclear,
        Self::OtherRentalRealEstate,
        Self::OtherRental,
        Self::DisabledAccess,
        Self::EmpowermentZone,
        Self::Research,
        Self::CoalGasificationCooperative,
        Self::AdvancedEnergyCooperative,
        Self::ManufacturingInvestmentCooperative,
        Self::CleanElectricityCooperative,
        Self::EnergyCooperative,
        Self::RehabilitationCooperative,
        Self::AdvancedManufacturingProduction,
        Self::CleanElectricityProduction,
        Self::CleanFuelProduction,
        Self::CleanHydrogen,
        Self::OrphanDrug,
        Self::EnhancedOilRecovery,
        Self::RenewableElectricity,
        Self::Biodiesel,
        Self::NewMarkets,
        Self::EmployerChildcare,
        Self::LowSulfurDiesel,
        Self::RailroadTrack,
        Self::MarginalWells,
        Self::DistilledSpirits,
        Self::EnergyEfficientHome,
        Self::RefuelingProperty,
        Self::DifferentialWage,
        Self::CarbonOxide,
        Self::CleanVehicle,
        Self::CommercialCleanVehicle,
        Self::Transferred,
        Self::Other,
    ];

    /// The code letters printed on the K-1.
    #[cfg(test)]
    pub(crate) fn letters(self) -> &'static str {
        match self {
            Self::WorkOpportunity => "J",
            Self::EmployerTipFica => "N",
            Self::PensionStartup => "AE",
            Self::AutoEnrollment => "AF",
            Self::MilitarySpouse => "AG",
            Self::SmallEmployerHealth => "BA",
            Self::PaidLeave => "BB",
            Self::LowIncomeHousingJ5 => "C",
            Self::LowIncomeHousingOther => "D",
            Self::ZeroEmissionNuclear => "A",
            Self::AdvancedNuclear => "B",
            Self::OtherRentalRealEstate => "F",
            Self::OtherRental => "G",
            Self::DisabledAccess => "K",
            Self::EmpowermentZone => "L",
            Self::Research => "M",
            Self::CoalGasificationCooperative => "P",
            Self::AdvancedEnergyCooperative => "Q",
            Self::ManufacturingInvestmentCooperative => "R",
            Self::CleanElectricityCooperative => "S",
            Self::EnergyCooperative => "T",
            Self::RehabilitationCooperative => "U",
            Self::AdvancedManufacturingProduction => "V",
            Self::CleanElectricityProduction => "W",
            Self::CleanFuelProduction => "X",
            Self::CleanHydrogen => "Y",
            Self::OrphanDrug => "Z",
            Self::EnhancedOilRecovery => "AA",
            Self::RenewableElectricity => "AB",
            Self::Biodiesel => "AC",
            Self::NewMarkets => "AD",
            Self::EmployerChildcare => "AH",
            Self::LowSulfurDiesel => "AI",
            Self::RailroadTrack => "AJ",
            Self::MarginalWells => "AK",
            Self::DistilledSpirits => "AL",
            Self::EnergyEfficientHome => "AM",
            Self::RefuelingProperty => "AO",
            Self::DifferentialWage => "AV",
            Self::CarbonOxide => "AW",
            Self::CleanVehicle => "AY",
            Self::CommercialCleanVehicle => "AZ",
            Self::Transferred => "BC",
            Self::Other => "ZZ",
        }
    }

    pub(crate) fn credit(self) -> BusinessCredit {
        match self {
            Self::WorkOpportunity => BusinessCredit::WorkOpportunity,
            Self::EmployerTipFica => BusinessCredit::EmployerTipFica,
            Self::PensionStartup => BusinessCredit::PensionStartup,
            Self::AutoEnrollment => BusinessCredit::AutoEnrollment,
            Self::MilitarySpouse => BusinessCredit::MilitarySpouse,
            Self::SmallEmployerHealth => BusinessCredit::SmallEmployerHealth,
            Self::PaidLeave => BusinessCredit::PaidLeave,
            Self::LowIncomeHousingJ5 => BusinessCredit::LowIncomeHousing,
            Self::LowIncomeHousingOther => BusinessCredit::LowIncomeHousing,
            Self::ZeroEmissionNuclear => BusinessCredit::ZeroEmissionNuclear,
            Self::AdvancedNuclear => BusinessCredit::AdvancedNuclear,
            Self::DisabledAccess => BusinessCredit::DisabledAccess,
            Self::EmpowermentZone => BusinessCredit::EmpowermentZone,
            Self::CoalGasificationCooperative => BusinessCredit::AdvancedCoalGasification,
            Self::AdvancedEnergyCooperative => BusinessCredit::AdvancedEnergyProject,
            Self::ManufacturingInvestmentCooperative => {
                BusinessCredit::AdvancedManufacturingInvestment
            }
            Self::CleanElectricityCooperative => BusinessCredit::CleanElectricity,
            Self::EnergyCooperative => BusinessCredit::Energy,
            Self::RehabilitationCooperative => BusinessCredit::Rehabilitation,
            Self::AdvancedManufacturingProduction => {
                BusinessCredit::AdvancedManufacturingProduction
            }
            Self::CleanElectricityProduction => BusinessCredit::CleanElectricityProduction,
            Self::CleanFuelProduction => BusinessCredit::CleanFuel,
            Self::CleanHydrogen => BusinessCredit::CleanHydrogen,
            Self::OrphanDrug => BusinessCredit::OrphanDrug,
            Self::EnhancedOilRecovery => BusinessCredit::EnhancedOilRecovery,
            Self::Biodiesel => BusinessCredit::Biodiesel,
            Self::NewMarkets => BusinessCredit::NewMarkets,
            Self::EmployerChildcare => BusinessCredit::EmployerChildcare,
            Self::LowSulfurDiesel => BusinessCredit::LowSulfurDiesel,
            Self::RailroadTrack => BusinessCredit::RailroadTrack,
            Self::MarginalWells => BusinessCredit::MarginalWells,
            Self::DistilledSpirits => BusinessCredit::DistilledSpirits,
            Self::EnergyEfficientHome => BusinessCredit::EnergyEfficientHome,
            Self::RefuelingProperty => BusinessCredit::RefuelingProperty,
            Self::DifferentialWage => BusinessCredit::DifferentialWage,
            Self::CarbonOxide => BusinessCredit::CarbonOxide,
            Self::CleanVehicle => BusinessCredit::CleanVehicle,
            Self::CommercialCleanVehicle => BusinessCredit::CommercialCleanVehicle,
            Self::Other => BusinessCredit::Other,
            Self::OtherRental => BusinessCredit::Other,
            Self::OtherRentalRealEstate => BusinessCredit::Other,
            Self::RenewableElectricity => BusinessCredit::Other,
            Self::Research => BusinessCredit::Other,
            Self::Transferred => BusinessCredit::Other,
        }
    }

    pub(crate) fn needs_line(self) -> bool {
        matches!(
            self,
            Self::OtherRentalRealEstate
                | Self::OtherRental
                | Self::Research
                | Self::RenewableElectricity
                | Self::Transferred
                | Self::Other
        )
    }
    pub(crate) fn line_allows(self, line: BusinessCredit) -> bool {
        match self {
            Self::Research => {
                matches!(line, BusinessCredit::Research | BusinessCredit::ResearchSmallBusiness)
            }
            Self::RenewableElectricity => matches!(
                line,
                BusinessCredit::RenewableElectricity
                    | BusinessCredit::RenewableElectricitySpecified
            ),
            Self::Transferred => line.info().transferable,
            Self::OtherRentalRealEstate | Self::OtherRental | Self::Other => {
                line.carryback_allowed()
            }
            _ => false,
        }
    }

    pub(crate) fn housing(self) -> bool {
        matches!(self, Self::LowIncomeHousingJ5 | Self::LowIncomeHousingOther)
    }

    // A low-income housing credit belongs to the partnership's rental real
    // estate activity (section 469(c)(2)), the activity of the K-1's box 2;
    // every other code follows the owner's participation in the business.
    pub(crate) fn rental(self) -> bool {
        matches!(
            self,
            Self::LowIncomeHousingJ5 | Self::LowIncomeHousingOther | Self::OtherRentalRealEstate
        )
    }
}

fn divided_origin_parts<P: CreditOriginPart>(
    parts: &[P],
    gross: i64,
    share: crate::community::Share,
) -> Vec<P> {
    let divided_amount = share.filer(gross);
    let mut order: Vec<usize> = (0..parts.len()).collect();
    order.sort_by(|&a, &b| parts[a].identity().cmp(parts[b].identity()));
    let weights: Vec<i64> = order.iter().map(|&i| parts[i].amount()).collect();
    let owner_amount = if share.filer_owns() {
        divided_amount
    } else {
        crate::kernel::carry::native_balance(gross, divided_amount)
    };
    let owner_parts = crate::kernel::ratio::allocate_prefix(owner_amount, &weights);
    let mut result = parts.to_vec();
    for (i, owned) in order.into_iter().zip(owner_parts) {
        result[i].set_amount(if share.filer_owns() {
            owned
        } else {
            crate::kernel::carry::native_balance(parts[i].amount(), owned)
        });
    }
    result.retain(|part| part.amount() > 0);
    result
}

/// Credit on the K-1
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1Credit {
    /// Code
    pub code: K1CreditCode,
    /// Form 3800 line supplied by the statement or eligible-small-business answer
    pub line: Option<BusinessCredit>,
    /// Amount
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

impl K1Credit {
    pub(crate) fn credit(&self) -> BusinessCredit {
        self.line.unwrap_or_else(|| self.code.credit())
    }

    #[cfg(test)]
    pub(crate) fn valid_list(items: &[Self]) -> bool {
        Self::issue_list(items).is_none()
    }
    pub(crate) fn source_valid_list(items: &[Self]) -> bool {
        Self::issue_list_with_mode(items, false).is_none()
    }
    #[cfg(test)]
    pub(crate) fn issue_list(items: &[Self]) -> Option<ValidationIssue> {
        Self::issue_list_with_mode(items, true)
    }
    pub(crate) fn issue_list_with_mode(
        items: &[Self],
        require_answers: bool,
    ) -> Option<ValidationIssue> {
        if items.len() > 60 {
            return Some(ValidationIssue::new("K1_CREDIT_CODE_TWICE").at_field("credits"));
        }
        for (index, item) in items.iter().enumerate() {
            let field = if !K1CreditCode::ALL.contains(&item.code) {
                Some("code")
            } else if !(1..=100_000_000_000).contains(&item.amount) {
                Some("amount")
            } else if !(if item.code.needs_line() {
                item.line.is_some_and(|line| item.code.line_allows(line))
            } else {
                item.line.is_none()
            }) {
                Some("code")
            } else if item.carbon_oxide_origin.as_ref().is_some_and(|origin| {
                let CarbonOxideCreditOrigin::Statement { equipment } = origin;
                !valid_origin_details(equipment)
                    || (require_answers
                        && (item.credit() != BusinessCredit::CarbonOxide
                            || !origin.valid(item.amount)))
            }) {
                Some("carbon_oxide_origin")
            } else if item.renewable_facility_origin.as_ref().is_some_and(|origin| {
                let RenewableFacilityCreditOrigin::Statement { facilities } = origin;
                !valid_origin_details(facilities)
                    || (require_answers
                        && (item.credit() != BusinessCredit::RenewableElectricitySpecified
                            || !origin.valid(item.amount)))
            }) {
                Some("renewable_facility_origin")
            } else if items[..index]
                .iter()
                .any(|prior| prior.code == item.code && prior.line == item.line)
            {
                Some("code")
            } else {
                None
            };
            if let Some(field) = field {
                return Some(
                    ValidationIssue::new("K1_CREDIT_CODE_TWICE")
                        .at_field(field)
                        .in_list("credits", index),
                );
            }
        }
        None
    }
}

/// Targeted group on the certification
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum WorkOpportunityGroup {
    /// Another targeted group ($6,000 of first-year wages)
    /// Such as TANF or SNAP recipients, ex-felons, vocational rehabilitation referrals, SSI recipients, long-term unemployment recipients, designated community residents or other veterans.
    Standard,
    /// Veteran with a service-connected disability hired within a year of discharge ($12,000)
    DisabledVeteran,
    /// Veteran unemployed at least 6 months in the year before hiring ($14,000)
    LongUnemployedVeteran,
    /// Veteran with a service-connected disability and unemployed at least 6 months ($24,000)
    DisabledLongUnemployedVeteran,
    /// Long-term family assistance recipient ($10,000 each year)
    LongTermFamilyAssistance {
        /// Second-year wages paid in 2026
        /// Wages for work in the year that begins one year after the employee began work.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        #[libertas_default(0)]
        second_year_paid_in_year: i64,
        /// Second-year wages counted on your 2025 Form 5884 line 1c
        #[libertas_money("USD", 2)]
        // Section 51 tax structure: long-term family-assistance second-year qualified wages are limited to $10,000.
        #[libertas_number(min = 0, max = 1000000)]
        #[libertas_default(0)]
        second_year_paid_in_prior_year: i64,
    },
}

impl WorkOpportunityGroup {
    fn first_year_cap(&self) -> i64 {
        match self {
            Self::Standard => crate::year_constants::WOTC_FIRST_YEAR_CAP_STANDARD_CENTS,
            Self::DisabledVeteran => {
                crate::year_constants::WOTC_FIRST_YEAR_CAP_DISABLED_VETERAN_CENTS
            }
            Self::LongUnemployedVeteran => {
                crate::year_constants::WOTC_FIRST_YEAR_CAP_LONG_UNEMPLOYED_VETERAN_CENTS
            }
            Self::DisabledLongUnemployedVeteran => {
                crate::year_constants::WOTC_FIRST_YEAR_CAP_DISABLED_LONG_UNEMPLOYED_VETERAN_CENTS
            }
            Self::LongTermFamilyAssistance { .. } => {
                crate::year_constants::WOTC_FIRST_YEAR_CAP_FAMILY_ASSISTANCE_CENTS
            }
        }
    }
}

/// Certified employee
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WorkOpportunityEmployee {
    /// Employee's W-2
    /// Choose this employee's W-2 in this business's Employee payroll. Enter a missing W-2 under Employees' W-2 wages first.
    pub label: String,
    /// Targeted group on the certification
    #[libertas_default(Standard)]
    pub group: WorkOpportunityGroup,
    /// Date the employee began work
    /// The credit covers only employees who began work before 2026. First-year wages in 2026 need a start date in 2025 after January 1; long-term family assistance second-year wages need a start date after January 1, 2024.
    #[libertas_date_only]
    pub began: u32,
    /// Time worked for you in the employee's first year
    /// At least 120. Fewer than 400 earns 25% of first-year wages; 400 or more earns 40%.
    // Tax structure: the employee’s first-year period contains at most 366 days of 24 hours.
    #[libertas_number(min = 120, max = 8784)]
    #[libertas_physical_unit("hour")]
    pub first_year_hours: u32,
    /// First-year wages paid in 2026
    /// FUTA-type wages, without the $7,000 wage base, for work in the year that began on the start date.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub first_year_paid_in_year: i64,
    /// First-year wages counted on your 2025 Form 5884
    /// These use up the employee's first-year wage limit.
    #[libertas_money("USD", 2)]
    // Section 51 tax structure: the prior-year first-year qualified wage limit for this employee class.
    #[libertas_number(min = 0, max = $WOTC_FIRST_YEAR_CAP_DISABLED_LONG_UNEMPLOYED_VETERAN_CENTS)]
    #[libertas_default(0)]
    pub first_year_paid_in_prior_year: i64,
}

/// Certified employee
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WorkOpportunityEmployeePage {
    /// Employee's W-2
    /// Choose this employee's W-2 in this business's Employee payroll. Enter a missing W-2 under Employees' W-2 wages first.
    #[libertas_enum_source("^^^^.payroll.Yes.statements")]
    pub label: String,
    /// Targeted group on the certification
    #[libertas_default(Standard)]
    pub group: WorkOpportunityGroup,
    /// Date the employee began work
    /// The credit covers only employees who began work before 2026. First-year wages in 2026 need a start date in 2025 after January 1; long-term family assistance second-year wages need a start date after January 1, 2024.
    #[libertas_date_only]
    pub began: u32,
    /// Time worked for you in the employee's first year
    /// At least 120. Fewer than 400 earns 25% of first-year wages; 400 or more earns 40%.
    // Tax structure: the employee’s first-year period contains at most 366 days of 24 hours.
    #[libertas_number(min = 120, max = 8784)]
    #[libertas_physical_unit("hour")]
    pub first_year_hours: u32,
    /// First-year wages paid in 2026
    /// FUTA-type wages, without the $7,000 wage base, for work in the year that began on the start date.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub first_year_paid_in_year: i64,
    /// First-year wages counted on your 2025 Form 5884
    /// These use up the employee's first-year wage limit.
    #[libertas_money("USD", 2)]
    // Section 51 tax structure: the prior-year first-year qualified wage limit for this employee class.
    #[libertas_number(min = 0, max = $WOTC_FIRST_YEAR_CAP_DISABLED_LONG_UNEMPLOYED_VETERAN_CENTS)]
    #[libertas_default(0)]
    pub first_year_paid_in_prior_year: i64,
}

impl From<&WorkOpportunityEmployee> for WorkOpportunityEmployeePage {
    fn from(value: &WorkOpportunityEmployee) -> Self {
        Self {
            label: value.label.clone(),
            group: value.group.clone(),
            began: value.began,
            first_year_hours: value.first_year_hours,
            first_year_paid_in_year: value.first_year_paid_in_year,
            first_year_paid_in_prior_year: value.first_year_paid_in_prior_year,
        }
    }
}
impl From<&WorkOpportunityEmployeePage> for WorkOpportunityEmployee {
    fn from(value: &WorkOpportunityEmployeePage) -> Self {
        Self {
            label: value.label.clone(),
            group: value.group.clone(),
            began: value.began,
            first_year_hours: value.first_year_hours,
            first_year_paid_in_year: value.first_year_paid_in_year,
            first_year_paid_in_prior_year: value.first_year_paid_in_prior_year,
        }
    }
}

impl WorkOpportunityEmployee {
    // Qualified 2026 first- and second-year wages in cents after each limit.
    fn qualified(&self) -> (i64, i64) {
        let first = phase::cap(
            self.first_year_paid_in_year,
            [positive(self.group.first_year_cap() - self.first_year_paid_in_prior_year)],
        );
        let second = match &self.group {
            WorkOpportunityGroup::LongTermFamilyAssistance {
                second_year_paid_in_year,
                second_year_paid_in_prior_year,
            } => phase::cap(
                *second_year_paid_in_year,
                [positive(1_000_000 - second_year_paid_in_prior_year)],
            ),
            _ => 0,
        };
        (first, second)
    }

    fn paid_in_year(&self) -> i64 {
        self.first_year_paid_in_year
            + match &self.group {
                WorkOpportunityGroup::LongTermFamilyAssistance {
                    second_year_paid_in_year, ..
                } => *second_year_paid_in_year,
                _ => 0,
            }
    }

    // The start-date windows the rule language cannot express (no date
    // literals): the credit ends for employees beginning work after 2025.
    pub(crate) fn date_rule(&self) -> Option<&'static str> {
        let second = matches!(
            self.group,
            WorkOpportunityGroup::LongTermFamilyAssistance {
                second_year_paid_in_year, ..
            } if second_year_paid_in_year > 0
        );
        if self.began > 20251231 {
            // Statutory WOTC employment-start deadline; it does not move with the return year.
            return Some("WOTC_BEGAN_AFTER_2025");
        }
        if (self.first_year_paid_in_year > 0 && self.began < 20250102)
            || (second && self.began < 20240102)
        {
            return Some("WOTC_WAGES_OUTSIDE_THE_YEAR");
        }
        None
    }
}

/// Work opportunity credit (Form 5884)
/// For wages paid in 2026 to employees certified by the state workforce agency who began work before 2026. The App figures lines 1a–1c and 2 and reduces this business's wages deduction by the credit.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WorkOpportunityCredit {
    /// Certified employees with credit wages in 2026
    #[libertas_unordered]
    #[libertas_size(min = 1)]
    /// ----
    /// Employee
    pub employees: Vec<WorkOpportunityEmployee>,
    /// Certifications and wages confirmed
    /// Each employee has a state workforce agency certification requested on Form 8850 within 28 days of starting work, is not your relative or dependent, did not work for you before, and more than half of the wages were for work in this business. The wages exclude federally funded on-the-job training pay and wages used for another employment credit. Business expenses include these wages in full on line 26. Each employee's state workforce agency certification (the Form 8850 response) shows it. A successor employer, a controlled group, summer youth employees or designated community residents need No, which leaves this credit out of this estimate.
    pub confirmed: bool,
}

/// Work opportunity credit (Form 5884)
/// For wages paid in 2026 to employees certified by the state workforce agency who began work before 2026. The App figures lines 1a–1c and 2 and reduces this business's wages deduction by the credit.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct WorkOpportunityCreditPage {
    /// Certified employees with credit wages in 2026
    #[libertas_unordered]
    #[libertas_size(min = 1)]
    /// ----
    /// Employee
    pub employees: Vec<WorkOpportunityEmployeePage>,
    /// Certifications and wages confirmed
    /// Each employee has a state workforce agency certification requested on Form 8850 within 28 days of starting work, is not your relative or dependent, did not work for you before, and more than half of the wages were for work in this business. The wages exclude federally funded on-the-job training pay and wages used for another employment credit. Business expenses include these wages in full on line 26. Each employee's state workforce agency certification (the Form 8850 response) shows it. A successor employer, a controlled group, summer youth employees or designated community residents need No, which leaves this credit out of this estimate.
    pub confirmed: bool,
}

impl From<&WorkOpportunityCredit> for WorkOpportunityCreditPage {
    fn from(value: &WorkOpportunityCredit) -> Self {
        Self {
            employees: value.employees.iter().map(Into::into).collect(),
            confirmed: value.confirmed,
        }
    }
}
impl From<&WorkOpportunityCreditPage> for WorkOpportunityCredit {
    fn from(value: &WorkOpportunityCreditPage) -> Self {
        Self {
            employees: value.employees.iter().map(Into::into).collect(),
            confirmed: value.confirmed,
        }
    }
}

impl WorkOpportunityCredit {
    pub(crate) fn valid(&self) -> bool {
        (1..=100).contains(&self.employees.len())
            && self.employees.iter().enumerate().all(|(index, e)| {
                !e.label.trim().is_empty()
                    && !self.employees[..index].iter().any(|prior| prior.label == e.label)
                    && (120..=8784).contains(&e.first_year_hours)
                    && (0..=100_000_000_000).contains(&e.first_year_paid_in_year)
                    && (0..=e.group.first_year_cap()).contains(&e.first_year_paid_in_prior_year)
                    && match &e.group {
                        WorkOpportunityGroup::LongTermFamilyAssistance {
                            second_year_paid_in_year,
                            second_year_paid_in_prior_year,
                        } => {
                            (0..=100_000_000_000).contains(second_year_paid_in_year)
                                && (0..=1_000_000).contains(second_year_paid_in_prior_year)
                        }
                        _ => true,
                    }
            })
    }

    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if let Some(index) =
            self.employees.iter().enumerate().find_map(|(i, e)| {
                self.employees[..i].iter().any(|p| p.label == e.label).then_some(i)
            })
        {
            return Some(
                ValidationIssue::new("WOTC_EMPLOYEE_TWICE")
                    .at_field("label")
                    .in_list("employees", index),
            );
        }
        self.employees.iter().enumerate().find_map(|(index, e)| {
            (e.first_year_paid_in_prior_year > e.group.first_year_cap())
                .then(|| {
                    ValidationIssue::new("WOTC_PRIOR_YEAR_WAGES_ABOVE_CAP")
                        .at_field("first_year_paid_in_prior_year")
                })
                .or_else(|| e.date_rule().map(|key| ValidationIssue::new(key).at_field("began")))
                .map(|issue| issue.in_list("employees", index))
        })
    }

    // Each employee matches one W-2 of the business, whose Medicare wages hold
    // the 2026 credit wages, and the Schedule C wage expense holds them all.
    pub(crate) fn qualified(&self, payroll: &BusinessPayroll, wage_expense: i64) -> bool {
        let BusinessPayroll::Yes { statements, .. } = payroll else {
            return false;
        };
        self.valid()
            && self.confirmed
            && self.rule().is_none()
            && self.employees.iter().map(|e| e.paid_in_year()).sum::<i64>() <= wage_expense
            && self.employees.iter().all(|e| {
                let matching: Vec<_> = statements.iter().filter(|s| s.label == e.label).collect();
                matching.len() == 1 && e.paid_in_year() <= matching[0].medicare
            })
    }

    // Form 5884 lines 1a, 1b, 1c and 2, whole dollars.
    pub(crate) fn form_lines(&self) -> [i64; 4] {
        let (mut short, mut long, mut second) = (0, 0, 0);
        for e in &self.employees {
            let (first, later) = e.qualified();
            if e.first_year_hours < 400 {
                short += first;
            } else {
                long += first;
            }
            second += later;
        }
        let a = fraction(dollars(short), 25, 100);
        let b = fraction(dollars(long), 40, 100);
        let c = fraction(dollars(second), 50, 100);
        [a, b, c, a + b + c]
    }

    pub(crate) fn credit(&self) -> i64 {
        self.form_lines()[3]
    }

    pub(crate) fn report(&self, business_id: i64, r: &mut FederalResult) {
        let lines = self.form_lines();
        emit_lines(
            r,
            &mut (),
            [
                ("1a", lines[0], msg!("EXPLAIN_BUSINESS_N_FORM5884_LINE0_1A")),
                ("1b", lines[1], msg!("EXPLAIN_BUSINESS_N_FORM5884_LINE0_1B")),
                ("1c", lines[2], msg!("EXPLAIN_BUSINESS_N_FORM5884_LINE0_1C")),
                ("2", lines[3], msg!("EXPLAIN_BUSINESS_N_FORM5884_LINE0_N")),
            ],
            |(printed, amount, explanation), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form5884, printed)),
                    &alloc::format!("business_{business_id}_form5884_line{printed}"),
                    LineValue::MoneyDollars(amount),
                    &["income.business.work_opportunity_credit"],
                    "https://www.irs.gov/instructions/i5884",
                    explanation,
                ))
            },
        );
    }
}

/// Plan start-up costs (Form 8881 Part I, Section 1)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PlanStartupCosts {
    /// First credit year
    /// The year the plan became effective, or the year before it when you elected that year. The credit applies to that year and the next two.
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 2024, max = 2026)]
    pub first_credit_year: u32,
    /// Employees paid at least $5,000 in the year before the first credit year (line A)
    /// 50 or fewer earns 100% of the costs; 51 to 100 earns 50%.
    // Section 45E eligible-employer limit: no more than 100 employees paid at least $5,000.
    #[libertas_number(min = 1, max = 100)]
    pub employees: u32,
    /// Non-highly compensated employees eligible for the plan in 2026 (line 3)
    #[libertas_number(min = 1)]
    pub eligible_employees: u32,
    /// Qualified start-up costs paid in 2026 (line 1)
    /// Costs to set up or administer the plan and educate employees about it, included once in Business expenses; the App reduces that deduction by the credit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub costs: i64,
}

impl PlanStartupCosts {
    // Form 8881 lines 1–5, whole dollars.
    pub(crate) fn form_lines(&self) -> [i64; 5] {
        let line1 = dollars(self.costs);
        let line2 = if self.employees <= 50 { line1 } else { fraction(line1, 50, 100) };
        let line3 = i64::from(self.eligible_employees) * 250;
        let line4 = phase::bound(line3, 500, 5000);
        [line1, line2, line3, line4, phase::cap(line2, [line4])]
    }
}

/// Small employer retirement plan credits (Form 8881)
/// For a plan this business started for its employees. The App figures the start-up costs credit and the $500 auto-enrollment credit.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct RetirementPlanCredit {
    /// Plan start-up costs
    /// [NullText]
    /// I am not claiming a plan start-up costs credit
    /// [SetText]
    /// The plan start-up costs are
    #[libertas_default($null)]
    pub startup: Option<PlanStartupCosts>,
    /// First year the plan included automatic enrollment (Part II)
    /// Leave empty when the plan has no eligible automatic contribution arrangement. The $500 credit applies in that year and the next two while the arrangement continues.
    /// [NullText]
    /// The plan has no eligible automatic enrollment arrangement
    /// [SetText]
    /// The plan first included eligible automatic enrollment in
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 2024, max = 2026)]
    #[libertas_default($null)]
    pub auto_enrollment_since: Option<u32>,
    /// Eligible small employer confirmed
    /// This business had no more than 100 employees paid at least $5,000 in the year before the first credit year, maintained no plan for substantially the same employees in the three years before, and is not part of a controlled group or affiliated service group. The prior-year payroll (employees paid at least $5,000) and the plan history show it. The employer contributions credit (lines 6a–6g) and the military spouse participation credit (Part III) need No, which leaves this credit out of this estimate.
    pub confirmed: bool,
}

impl RetirementPlanCredit {
    pub(crate) fn valid(&self) -> bool {
        self.startup.as_ref().is_none_or(|s| {
            (2024..=2026).contains(&s.first_credit_year)
                && (1..=100).contains(&s.employees)
                && (1..=100_000).contains(&s.eligible_employees)
                && (0..=100_000_000_000).contains(&s.costs)
        }) && self.auto_enrollment_since.is_none_or(|y| (2024..=2026).contains(&y))
    }

    pub(crate) fn startup_credit(&self) -> i64 {
        self.startup.as_ref().map_or(0, |s| s.form_lines()[4])
    }

    pub(crate) fn auto_enrollment_credit(&self) -> i64 {
        if self.auto_enrollment_since.is_some() { 500 } else { 0 }
    }

    pub(crate) fn report(&self, business_id: i64, r: &mut FederalResult) {
        let mut rows: Vec<(&str, i64, crate::Message)> = Vec::new();
        if let Some(startup) = &self.startup {
            let l = startup.form_lines();
            rows.extend([
                ("1", l[0], msg!("EXPLAIN_BUSINESS_N_FORM8881_LINE_N")),
                ("2", l[1], msg!("EXPLAIN_BUSINESS_N_FORM8881_LINE_N_2")),
                ("3", l[2], msg!("EXPLAIN_BUSINESS_N_FORM8881_LINE_N_3")),
                ("4", l[3], msg!("EXPLAIN_BUSINESS_N_FORM8881_LINE_N_4")),
                ("5", l[4], msg!("EXPLAIN_BUSINESS_N_FORM8881_LINE_N_5")),
            ]);
        }
        if self.auto_enrollment_since.is_some() {
            rows.push(("9", 500, msg!("EXPLAIN_BUSINESS_N_FORM8881_LINE_N_6")));
        }
        emit_lines(r, &mut (), rows, |(printed, amount, explanation), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form8881, printed)),
                &alloc::format!("business_{business_id}_form8881_line{printed}"),
                LineValue::MoneyDollars(amount),
                &["income.business.retirement_plan_credit"],
                "https://www.irs.gov/instructions/i8881",
                explanation,
            ))
        });
    }
}

/// How you offered the employees' coverage
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum HealthCreditCoverage {
    /// Through a SHOP Marketplace
    /// Qualified health plans bought through the Small Business Health Options Program on HealthCare.gov or your state's SHOP, including a direct enrollment process where your state offers one. A stand-alone dental plan offered through SHOP counts.
    Marketplace,
    /// Outside SHOP in a county without SHOP plans, after a 2025 credit
    /// Your principal business address is in a county where SHOP had no qualified health plans for 2026, and you properly claimed this credit for 2025 (Notice 2018-27).
    CountyException,
    /// Other group coverage
    /// Coverage not bought through SHOP, such as a group policy from an insurer or broker, or an individual coverage or small employer health reimbursement arrangement. It earns no credit: this credit is left out of the estimate with a notice and the premiums stay deducted in full.
    Other,
}

/// Coverage tier
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum HealthCoverageTier {
    /// Employee-only coverage
    EmployeeOnly,
    /// Any other tier
    /// Employee and spouse, employee and children, family or dependent coverage.
    Family,
}

/// The employee's 2026 coverage under your arrangement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployeeHealthCoverage {
    /// Coverage tier
    #[libertas_default(EmployeeOnly)]
    pub tier: HealthCoverageTier,
    /// Time in 2026 the employee was enrolled
    // Tax structure: a calendar year has twelve months.
    #[libertas_number(min = 1, max = 12)]
    #[libertas_physical_unit("month")]
    pub months: u32,
    /// Total premium charged for the employee's coverage in those months
    /// Your share plus the employee's share, from the SHOP invoices, without any tobacco surcharge.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub total_premium: i64,
    /// Premiums you paid (Worksheet 4 column (b))
    /// Your own share only. Not what the employee paid, including pretax salary reductions through a cafeteria plan, and not a tobacco surcharge. A state subsidy paid directly to the insurer counts as paid by you; do not subtract a state subsidy or credit paid to you (enter it below as state premium subsidies and tax credits).
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub employer_paid: i64,
}

/// Hours of service in 2026 (Worksheet 1 column (b))
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HealthCreditService {
    /// Hours of service
    Hours {
        /// Service time, at most 2,080
        /// Time paid for work plus paid vacation, holiday, sick and other leave (at most 160 for one continuous leave). You may instead count 8 for each day, or 40 for each week, that has any service.
        // Form 8941 tax structure: count no more than 2,080 service hours per employee.
        #[libertas_number(min = 1, max = $SMALL_EMPLOYER_HEALTH_FTE_HOURS)]
        #[libertas_physical_unit("hour")]
        hours: u32,
    },
    /// Seasonal worker who worked 120 days or fewer
    /// Such as holiday-season retail or summer-only work. Their hours and wages do not count toward full-time equivalents or average wages; premiums you paid for them do.
    Seasonal,
}

impl HealthCreditService {
    fn hours(&self) -> u32 {
        match self {
            Self::Hours { hours } => *hours,
            Self::Seasonal => 0,
        }
    }
}

/// Employee for the health insurance credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthCreditEmployee {
    /// Employee's W-2
    /// Choose this employee's W-2 in this business's Employee payroll; its box 5 Medicare wages are this employee's wages. Enter a missing W-2 under Employees' W-2 wages first.
    pub label: String,
    /// Hours of service
    pub service: HealthCreditService,
    /// Health coverage you provided in 2026
    /// Leave empty when the employee was not enrolled.
    /// [NullText]
    /// This employee was not enrolled in health coverage I provided
    /// [SetText]
    /// The health coverage I provided was
    #[libertas_default($null)]
    pub coverage: Option<EmployeeHealthCoverage>,
}

/// Employee for the health insurance credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthCreditEmployeePage {
    /// Employee's W-2
    /// Choose this employee's W-2 in this business's Employee payroll; its box 5 Medicare wages are this employee's wages. Enter a missing W-2 under Employees' W-2 wages first.
    #[libertas_enum_source("^^^^.payroll.Yes.statements")]
    pub label: String,
    /// Hours of service
    pub service: HealthCreditService,
    /// Health coverage you provided in 2026
    /// Leave empty when the employee was not enrolled.
    /// [NullText]
    /// This employee was not enrolled in health coverage I provided
    /// [SetText]
    /// The health coverage I provided was
    #[libertas_default($null)]
    pub coverage: Option<EmployeeHealthCoverage>,
}

impl From<&HealthCreditEmployee> for HealthCreditEmployeePage {
    fn from(value: &HealthCreditEmployee) -> Self {
        Self {
            label: value.label.clone(),
            service: value.service.clone(),
            coverage: value.coverage.clone(),
        }
    }
}
impl From<&HealthCreditEmployeePage> for HealthCreditEmployee {
    fn from(value: &HealthCreditEmployeePage) -> Self {
        Self {
            label: value.label.clone(),
            service: value.service.clone(),
            coverage: value.coverage.clone(),
        }
    }
}

/// Small employer health insurance premiums credit (Form 8941)
/// For premiums this business paid for its employees' coverage through a SHOP Marketplace. The App figures Form 8941 lines 1–16 and reduces the employee benefit programs deduction (Schedule C line 14) by the credit.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SmallEmployerHealthCredit {
    /// How you offered the coverage (line A)
    pub coverage: HealthCreditCoverage,
    /// First year of your two-year credit period (line C)
    /// 2026 when no earlier return of yours or a predecessor's claimed this credit; 2025 when your 2025 return was the first. If a return for 2014 through 2024 claimed it, the two-year period has ended and no credit is allowed; do not add this credit.
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 2025, max = 2026)]
    pub first_credit_year: u32,
    /// Average employee-only premium for your county
    /// From the 2026 average premium table at the end of the Form 8941 instructions (published in November 2026), for the county of this business's principal address: the Employee-Only column.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub employee_only_average: i64,
    /// Average family premium for your county
    /// The same table row's "Dependent, Family, etc." column.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub family_average: i64,
    /// Employees (Worksheet 1)
    /// Everyone you employed in 2026, including seasonal workers, except: you, a partner or owner of more than 5% (more than 2% of an S corporation), anyone's spouse among these, their children and other descendants, siblings and step-siblings, parents and other ancestors, stepparents, nieces, nephews, aunts, uncles and in-laws, and a household member who is the dependent of one of them.
    #[libertas_unordered]
    #[libertas_size(min = 1)]
    /// ----
    /// Employee
    pub employees: Vec<HealthCreditEmployee>,
    /// State premium subsidies and tax credits (line 10)
    /// State tax credits you can take, and state premium subsidies paid to you or to the insurer, for the premiums entered above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub state_subsidies: i64,
    /// Eligible small employer confirmed
    /// Your arrangement pays the same percentage, at least 50%, of each enrolled employee's premium (or a composite or list-billing arrangement the Form 8941 instructions allow) through plans in this one county's rating area; the business is not in Hawaii. Every employee listed above has a W-2 in this business's Employee payroll, and you have no other business with employees, no household employee and no leased employee; this business is not part of a controlled group, an affiliated service group or a commonly controlled group (a spouse's separate business counts only if either of you works in or manages the other's). The premiums you paid are included once in this business's employee benefit programs expense. Your payroll and hour records, SHOP invoices and earlier returns show it. No leaves this credit out of this estimate.
    pub confirmed: bool,
}

/// Small employer health insurance premiums credit (Form 8941)
/// For premiums this business paid for its employees' coverage through a SHOP Marketplace. The App figures Form 8941 lines 1–16 and reduces the employee benefit programs deduction (Schedule C line 14) by the credit.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct SmallEmployerHealthCreditPage {
    /// How you offered the coverage (line A)
    pub coverage: HealthCreditCoverage,
    /// First year of your two-year credit period (line C)
    /// 2026 when no earlier return of yours or a predecessor's claimed this credit; 2025 when your 2025 return was the first. If a return for 2014 through 2024 claimed it, the two-year period has ended and no credit is allowed; do not add this credit.
    // Tax-year source window: only the stated current or prior tax years apply.
    #[libertas_number(min = 2025, max = 2026)]
    pub first_credit_year: u32,
    /// Average employee-only premium for your county
    /// From the 2026 average premium table at the end of the Form 8941 instructions (published in November 2026), for the county of this business's principal address: the Employee-Only column.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub employee_only_average: i64,
    /// Average family premium for your county
    /// The same table row's "Dependent, Family, etc." column.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub family_average: i64,
    /// Employees (Worksheet 1)
    /// Everyone you employed in 2026, including seasonal workers, except: you, a partner or owner of more than 5% (more than 2% of an S corporation), anyone's spouse among these, their children and other descendants, siblings and step-siblings, parents and other ancestors, stepparents, nieces, nephews, aunts, uncles and in-laws, and a household member who is the dependent of one of them.
    #[libertas_unordered]
    #[libertas_size(min = 1)]
    /// ----
    /// Employee
    pub employees: Vec<HealthCreditEmployeePage>,
    /// State premium subsidies and tax credits (line 10)
    /// State tax credits you can take, and state premium subsidies paid to you or to the insurer, for the premiums entered above.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub state_subsidies: i64,
    /// Eligible small employer confirmed
    /// Your arrangement pays the same percentage, at least 50%, of each enrolled employee's premium (or a composite or list-billing arrangement the Form 8941 instructions allow) through plans in this one county's rating area; the business is not in Hawaii. Every employee listed above has a W-2 in this business's Employee payroll, and you have no other business with employees, no household employee and no leased employee; this business is not part of a controlled group, an affiliated service group or a commonly controlled group (a spouse's separate business counts only if either of you works in or manages the other's). The premiums you paid are included once in this business's employee benefit programs expense. Your payroll and hour records, SHOP invoices and earlier returns show it. No leaves this credit out of this estimate.
    pub confirmed: bool,
}

impl From<&SmallEmployerHealthCredit> for SmallEmployerHealthCreditPage {
    fn from(value: &SmallEmployerHealthCredit) -> Self {
        Self {
            coverage: value.coverage,
            first_credit_year: value.first_credit_year,
            employee_only_average: value.employee_only_average,
            family_average: value.family_average,
            employees: value.employees.iter().map(Into::into).collect(),
            state_subsidies: value.state_subsidies,
            confirmed: value.confirmed,
        }
    }
}
impl From<&SmallEmployerHealthCreditPage> for SmallEmployerHealthCredit {
    fn from(value: &SmallEmployerHealthCreditPage) -> Self {
        Self {
            coverage: value.coverage,
            first_credit_year: value.first_credit_year,
            employee_only_average: value.employee_only_average,
            family_average: value.family_average,
            employees: value.employees.iter().map(Into::into).collect(),
            state_subsidies: value.state_subsidies,
            confirmed: value.confirmed,
        }
    }
}

impl SmallEmployerHealthCredit {
    pub(crate) fn valid(&self) -> bool {
        (2025..=2026).contains(&self.first_credit_year)
            && (1..=100_000_000).contains(&self.employee_only_average)
            && (1..=100_000_000).contains(&self.family_average)
            && (0..=100_000_000_000).contains(&self.state_subsidies)
            && (1..=100).contains(&self.employees.len())
            && self.employees.iter().enumerate().all(|(index, e)| {
                !e.label.trim().is_empty()
                    && !self.employees[..index].iter().any(|prior| prior.label == e.label)
                    && match &e.service {
                        HealthCreditService::Hours { hours } => {
                            (1..=SMALL_EMPLOYER_HEALTH_FTE_HOURS).contains(hours)
                        }
                        HealthCreditService::Seasonal => true,
                    }
                    && e.coverage.as_ref().is_none_or(|c| {
                        (1..=12).contains(&c.months)
                            && (1..=100_000_000_000).contains(&c.total_premium)
                            && (1..=c.total_premium).contains(&c.employer_paid)
                    })
            })
    }

    // The page rules, repeated for the server (the client twins are the
    // validation rules on this record and on EmployeeHealthCoverage).
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if let Some(index) =
            self.employees.iter().enumerate().find_map(|(i, e)| {
                self.employees[..i].iter().any(|p| p.label == e.label).then_some(i)
            })
        {
            return Some(
                ValidationIssue::new("SMALL_EMPLOYER_HEALTH_EMPLOYEE_TWICE")
                    .at_field("label")
                    .in_list("employees", index),
            );
        }
        if let Some(index) = self
            .employees
            .iter()
            .position(|e| e.coverage.as_ref().is_some_and(|c| c.employer_paid > c.total_premium))
        {
            return Some(
                ValidationIssue::new("SMALL_EMPLOYER_HEALTH_SHARE_ABOVE_PREMIUM")
                    .at_field("employer_paid")
                    .in_field("coverage")
                    .in_list("employees", index),
            );
        }
        if self.employees.iter().all(|e| e.coverage.is_none()) {
            return Some(
                ValidationIssue::new("SMALL_EMPLOYER_HEALTH_NOBODY_ENROLLED").at_field("employees"),
            );
        }
        (self.coverage == HealthCreditCoverage::CountyException && self.first_credit_year != 2025)
            .then(|| {
                ValidationIssue::new("SMALL_EMPLOYER_HEALTH_COUNTY_RELIEF_NEEDS_PRIOR_YEAR")
                    .at_field("first_credit_year")
            })
    }

    // Each listed employee matches one W-2 of the business, whose Medicare
    // wages are the employee's wages, and the premiums paid are within the
    // employee benefit programs expense (Schedule C line 14, cents).
    pub(crate) fn qualified(&self, payroll: &BusinessPayroll, benefits_expense: i64) -> bool {
        let BusinessPayroll::Yes { statements, .. } = payroll else {
            return false;
        };
        self.valid()
            && self.confirmed
            && self.coverage != HealthCreditCoverage::Other
            && self.rule().is_none()
            && self.premiums_paid() <= benefits_expense
            && self
                .employees
                .iter()
                .all(|e| statements.iter().filter(|s| s.label == e.label).count() == 1)
    }

    // Form 8941 line 4 in cents.
    pub(crate) fn premiums_paid(&self) -> i64 {
        self.employees.iter().filter_map(|e| e.coverage.as_ref()).map(|c| c.employer_paid).sum()
    }

    // Worksheets 2 and 7: one full-time equivalent per 2,080 hours, rounded
    // down, and one at least.
    fn fte(hours: u32) -> i64 {
        max(1, i64::from(crate::kernel::ratio::trunc_u32(hours, SMALL_EMPLOYER_HEALTH_FTE_HOURS)))
    }

    // Form 8941 lines 1-14 (indexes 0-13), whole dollars except the counts on
    // lines 1, 2, 13 and 14. A line the form skips is zero.
    pub(crate) fn form_lines(&self, payroll: &BusinessPayroll) -> [i64; 14] {
        let mut lines = [0; 14];
        let statements = match payroll {
            BusinessPayroll::Yes { statements, .. } => statements.as_slice(),
            BusinessPayroll::No => &[],
        };
        lines[0] = self.employees.len() as i64;
        let fte = Self::fte(self.employees.iter().map(|e| e.service.hours()).sum());
        lines[1] = fte;
        if fte >= SMALL_EMPLOYER_HEALTH_FTE_LIMIT {
            return lines;
        }
        // Worksheet 3: wages are box 5 of the matched W-2; a seasonal worker's are left out.
        let wages: i64 = self
            .employees
            .iter()
            .filter(|e| e.service != HealthCreditService::Seasonal)
            .filter_map(|e| statements.iter().find(|s| s.label == e.label))
            .map(|s| s.medicare)
            .sum();
        let rounding = SMALL_EMPLOYER_HEALTH_WAGE_ROUNDING_DOLLARS;
        let average = crate::kernel::ratio::trunc_i64(
            crate::kernel::ratio::trunc_i64(wages, fte),
            rounding * 100,
        ) * rounding;
        lines[2] = average;
        let dollar_amount = SMALL_EMPLOYER_HEALTH_WAGE_AMOUNT_DOLLARS;
        if average > 2 * dollar_amount {
            return lines;
        }
        // Worksheet 4: the cap of each enrolled employee is the county average
        // premium of the tier for the months enrolled, at the share the
        // employer paid.
        let capped: i64 = self
            .employees
            .iter()
            .filter_map(|e| e.coverage.as_ref())
            .map(|c| {
                let county_average = match c.tier {
                    HealthCoverageTier::EmployeeOnly => self.employee_only_average,
                    HealthCoverageTier::Family => self.family_average,
                };
                fraction(
                    county_average * i64::from(c.months),
                    c.employer_paid,
                    12 * c.total_premium,
                )
            })
            .sum();
        let line4 = dollars(self.premiums_paid());
        let line5 = dollars(capped);
        let line6 = phase::cap(line4, [line5]);
        let line7 = fraction(line6, SMALL_EMPLOYER_HEALTH_CREDIT_PERCENT, 100);
        // Worksheets 5 and 6, each reduction from line 7 at full precision.
        let line8 = phase::reduce_by_fraction(
            line7,
            positive(fte - SMALL_EMPLOYER_HEALTH_FTE_PHASEOUT_START),
            SMALL_EMPLOYER_HEALTH_FTE_PHASEOUT_RANGE,
        );
        let line9 = phase::phase_out(
            line8,
            fraction(line7, positive(average - dollar_amount), dollar_amount),
        );
        let line10 = dollars(self.state_subsidies);
        let line11 = positive(line4 - line10);
        let line12 = phase::cap(line9, [line11]);
        lines[3..12]
            .copy_from_slice(&[line4, line5, line6, line7, line8, line9, line10, line11, line12]);
        if line12 > 0 {
            let enrolled = || self.employees.iter().filter(|e| e.coverage.is_some());
            lines[12] = enrolled().count() as i64;
            lines[13] = Self::fte(enrolled().map(|e| e.service.hours()).sum());
        }
        lines
    }

    // The credit determined (line 12), whole dollars; the section 280C(h)
    // reduction of Schedule C line 14 is the same amount.
    pub(crate) fn credit(&self, payroll: &BusinessPayroll) -> i64 {
        self.form_lines(payroll)[11]
    }

    // Lines 1-16; `k1_credit` is the owner's K-1 code BA total (line 15),
    // whole dollars.
    pub(crate) fn report(
        &self,
        business_id: i64,
        payroll: &BusinessPayroll,
        k1_credit: i64,
        r: &mut FederalResult,
    ) {
        let l = self.form_lines(payroll);
        let rows = [
            ("1", l[0], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_1")),
            ("2", l[1], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_2")),
            ("3", l[2], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_3")),
            ("4", l[3], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_4")),
            ("5", l[4], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_5")),
            ("6", l[5], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_6")),
            ("7", l[6], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_7")),
            ("8", l[7], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_8")),
            ("9", l[8], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_9")),
            ("10", l[9], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_10")),
            ("11", l[10], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_11")),
            ("12", l[11], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_12")),
            ("13", l[12], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_13")),
            ("14", l[13], msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_14")),
            ("15", k1_credit, msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_15")),
            ("16", l[11] + k1_credit, msg!("EXPLAIN_BUSINESS_N_FORM8941_LINE_16")),
        ];
        for (printed, amount, explanation) in rows {
            // Lines 1, 2, 13 and 14 are numbers of people, not dollars.
            let emit = if matches!(printed, "1" | "2" | "13" | "14") {
                crate::line_emission::form_count
            } else {
                form_line
            };
            emit(
                r,
                FormLine::new(TaxForm::Form8941, printed),
                &alloc::format!("business_{business_id}_form8941_line{printed}"),
                amount,
                &["income.business.small_employer_health_credit", "income.business.payroll"],
                "https://www.irs.gov/pub/irs-dft/i8941--dft.pdf",
                explanation,
            );
        }
    }
}

/// One employee's 2026 paid family and medical leave
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PaidLeaveEmployee {
    /// Employee's W-2
    /// Choose this employee's W-2 in this business's Employee payroll. Enter a missing W-2 under Employees' W-2 wages first. Not your spouse, your child under 21 or your parent: their wages are not FUTA wages and earn no credit.
    pub label: String,
    /// Time the employee customarily works each week
    /// At least 20; an employee customarily working less is not a qualifying employee. It also sets the 12-week limit on the leave counted.
    #[libertas_number(min = $PAID_LEAVE_MIN_WEEKLY_HOURS)]
    #[libertas_physical_unit("hour")]
    pub weekly_hours: u32,
    /// 2025 compensation from you, annualized
    /// Pay you gave the employee in 2025 (Form W-2 box 1 plus pretax elective deferrals), figured for a full year when they worked part of 2025, pro rata for a part-time employee. More than $96,000 means the employee does not qualify; enter 0 for an employee who started in 2026.
    #[libertas_money("USD", 2)]
    // Section 45S qualifying-employee compensation limit for 2026.
    #[libertas_number(min = 0, max = $PAID_LEAVE_COMPENSATION_LIMIT_2026_CENTS)]
    pub prior_year_compensation: i64,
    /// Normal hourly wage rate
    /// The employee's usual pay per hour for work, without irregular overtime or discretionary bonuses; a salaried employee's pay converted to an hourly rate by a reasonable method.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub normal_hourly_rate: i64,
    /// Family and medical leave paid in 2026
    /// Count only leave under your written policy for a family and medical leave purpose, taken while the employee was a qualifying employee, and at most 12 weeks of the employee's usual hours.
    #[libertas_number(min = 0.01, step = 0.01)]
    #[libertas_physical_unit("hour")]
    pub leave_hours: f64,
    /// Wages you paid for that leave (worksheet column (b))
    /// Only what you paid under your policy for those hours. Not leave paid by a state or locality or required by state or local law, not leave paid by an insurance policy whose premiums you claim below, and not wages you counted for the work opportunity credit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub leave_wages: i64,
    /// Rate of payment under your policy
    /// The share of normal wages your policy pays during this leave; enter 100 for 100% or more. It must be at least 50%. If this employee's leave was paid at more than one rate, enter the lowest.
    #[libertas_number(min = $PAID_LEAVE_MIN_RATE_PERCENT, max = 100, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub rate: f64,
}

/// One employee's 2026 paid family and medical leave
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PaidLeaveEmployeePage {
    /// Employee's W-2
    /// Choose this employee's W-2 in this business's Employee payroll. Enter a missing W-2 under Employees' W-2 wages first. Not your spouse, your child under 21 or your parent: their wages are not FUTA wages and earn no credit.
    #[libertas_enum_source("^^^^.payroll.Yes.statements")]
    pub label: String,
    /// Time the employee customarily works each week
    /// At least 20; an employee customarily working less is not a qualifying employee. It also sets the 12-week limit on the leave counted.
    #[libertas_number(min = $PAID_LEAVE_MIN_WEEKLY_HOURS)]
    #[libertas_physical_unit("hour")]
    pub weekly_hours: u32,
    /// 2025 compensation from you, annualized
    /// Pay you gave the employee in 2025 (Form W-2 box 1 plus pretax elective deferrals), figured for a full year when they worked part of 2025, pro rata for a part-time employee. More than $96,000 means the employee does not qualify; enter 0 for an employee who started in 2026.
    #[libertas_money("USD", 2)]
    // Section 45S qualifying-employee compensation limit for 2026.
    #[libertas_number(min = 0, max = $PAID_LEAVE_COMPENSATION_LIMIT_2026_CENTS)]
    pub prior_year_compensation: i64,
    /// Normal hourly wage rate
    /// The employee's usual pay per hour for work, without irregular overtime or discretionary bonuses; a salaried employee's pay converted to an hourly rate by a reasonable method.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub normal_hourly_rate: i64,
    /// Family and medical leave paid in 2026
    /// Count only leave under your written policy for a family and medical leave purpose, taken while the employee was a qualifying employee, and at most 12 weeks of the employee's usual hours.
    #[libertas_number(min = 0.01, step = 0.01)]
    #[libertas_physical_unit("hour")]
    pub leave_hours: f64,
    /// Wages you paid for that leave (worksheet column (b))
    /// Only what you paid under your policy for those hours. Not leave paid by a state or locality or required by state or local law, not leave paid by an insurance policy whose premiums you claim below, and not wages you counted for the work opportunity credit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub leave_wages: i64,
    /// Rate of payment under your policy
    /// The share of normal wages your policy pays during this leave; enter 100 for 100% or more. It must be at least 50%. If this employee's leave was paid at more than one rate, enter the lowest.
    #[libertas_number(min = $PAID_LEAVE_MIN_RATE_PERCENT, max = 100, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub rate: f64,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for PaidLeaveEmployeePage {}

impl From<&PaidLeaveEmployee> for PaidLeaveEmployeePage {
    fn from(value: &PaidLeaveEmployee) -> Self {
        Self {
            label: value.label.clone(),
            weekly_hours: value.weekly_hours,
            prior_year_compensation: value.prior_year_compensation,
            normal_hourly_rate: value.normal_hourly_rate,
            leave_hours: value.leave_hours,
            leave_wages: value.leave_wages,
            rate: value.rate,
        }
    }
}
impl From<&PaidLeaveEmployeePage> for PaidLeaveEmployee {
    fn from(value: &PaidLeaveEmployeePage) -> Self {
        Self {
            label: value.label.clone(),
            weekly_hours: value.weekly_hours,
            prior_year_compensation: value.prior_year_compensation,
            normal_hourly_rate: value.normal_hourly_rate,
            leave_hours: value.leave_hours,
            leave_wages: value.leave_wages,
            rate: value.rate,
        }
    }
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for PaidLeaveEmployee {}

impl PaidLeaveEmployee {
    fn valid(&self) -> bool {
        !self.label.trim().is_empty()
            && (PAID_LEAVE_MIN_WEEKLY_HOURS..=80).contains(&self.weekly_hours)
            && (0..=PAID_LEAVE_COMPENSATION_LIMIT_2026_CENTS)
                .contains(&self.prior_year_compensation)
            && (1..=100_000).contains(&self.normal_hourly_rate)
            && (1..=96_000).contains(&crate::kernel::natural::units(self.leave_hours, 100))
            && (1..=100_000_000_000).contains(&self.leave_wages)
            && Self::rate_valid(self.rate)
            && !self.above_12_weeks()
    }

    fn rate_valid(rate: f64) -> bool {
        crate::kernel::natural::exact(rate, 100)
            .is_some_and(|rate| (PAID_LEAVE_MIN_RATE_HUNDREDTHS..=10_000).contains(&rate))
    }

    // Section 45S(b)(3): at most 12 weeks of the employee's usual hours.
    fn above_12_weeks(&self) -> bool {
        crate::kernel::natural::units(self.leave_hours, 100)
            > PAID_LEAVE_MAX_WEEKS * 100 * i64::from(self.weekly_hours)
    }

    // Section 45S(b)(1): wages at most the normal hourly rate for the leave
    // hours, in cents.
    fn credit_wages(&self) -> i64 {
        phase::cap(
            self.leave_wages,
            [fraction(
                self.normal_hourly_rate,
                crate::kernel::natural::units(self.leave_hours, 100),
                100,
            )],
        )
    }

    // Worksheet Part 1 column (d) in cents.
    fn credit_cents(&self) -> i64 {
        paid_leave_credit_cents(self.credit_wages(), self.rate)
    }
}

// The applicable percentage of section 45S(a)(2) (12.5% at a 50% rate, plus
// 0.25 point per point above it, at most 25%) times an amount in cents. The
// percentage is kept in quarters of a hundredth of a percent, so a rate of
// 66.67% needs no rounding of points.
fn paid_leave_credit_cents(amount: i64, rate: f64) -> i64 {
    let quarters = min(
        PAID_LEAVE_RATE_POINTS_PER_PERCENT_POINT * PAID_LEAVE_MAX_PERCENT_HUNDREDTHS,
        PAID_LEAVE_RATE_POINTS_PER_PERCENT_POINT * PAID_LEAVE_BASE_PERCENT_HUNDREDTHS
            + crate::kernel::natural::units(rate, 100)
            - PAID_LEAVE_MIN_RATE_HUNDREDTHS,
    );
    fraction(amount, quarters, PAID_LEAVE_RATE_POINTS_PER_PERCENT_POINT * 10_000)
}

/// Paid family and medical leave insurance (premium method)
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PaidLeaveInsurance {
    /// Creditable premiums you paid or incurred in 2026
    /// Only the part of the premiums that funds paid family and medical leave for qualifying employees that the wage method could credit: not coverage for other leave, for employees who do not qualify, or for leave required or paid by a state or locality. Split a premium that covers both by a reasonable method your records support. These premiums are included once in this business's employee benefit programs expense; the App reduces that deduction by the credit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 1, max = 100000000000)]
    pub premiums: i64,
    /// Rate of payment under the policy
    /// The share of normal wages the policy pays during leave, whether or not anyone took leave in 2026. With more than one policy, enter the lowest rate.
    #[libertas_number(min = $PAID_LEAVE_MIN_RATE_PERCENT, max = 100, step = 0.01)]
    #[libertas_physical_unit("percent")]
    pub rate: f64,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for PaidLeaveInsurance {}

/// Employer credit for paid family and medical leave (Form 8994)
/// For wages this business paid to qualifying employees on family and medical leave under its written policy, or premiums for a paid leave insurance policy. The App figures Form 8994 lines 1a–3 and reduces the wages and employee benefit programs deductions by the credit.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PaidLeaveCredit {
    /// Employees paid while on family and medical leave (wage method)
    /// Leave empty when you claim only insurance premiums.
    #[libertas_unordered]
    /// ----
    /// Employee
    pub employees: Vec<PaidLeaveEmployee>,
    /// Paid family and medical leave insurance (premium method)
    /// Leave empty when you have no such policy. Do not also count, in the wage rows above, leave this policy paid for.
    /// [NullText]
    /// I have no paid family and medical leave insurance policy to claim
    /// [SetText]
    /// The paid family and medical leave insurance policy is
    #[libertas_default($null)]
    pub insurance: Option<PaidLeaveInsurance>,
    /// Written policy and eligibility confirmed
    /// Before the leave was taken you had a written policy giving every qualifying employee at least 2 weeks a year of paid leave for family and medical leave purposes (a proportionate amount for anyone working fewer than 30 hours a week) at no less than 50% of normal wages, without excluding any class of qualifying employees; leave a state or locality requires or pays counts toward these minimums. If any qualifying employee is not covered by the Family and Medical Leave Act (for example, you have fewer than 50 employees), the policy says you will not interfere with or retaliate for the use of the leave, and you complied. Each listed employee had worked for you at least a year (or at least six months, if you chose that period) when the leave was taken. The entered leave was specifically for family and medical leave purposes, not general vacation, personal or sick leave. You have no other business, and this business is not part of a controlled group or a group under common control. Your written policy, payroll, leave and insurance records show it. No leaves this credit out of this estimate.
    pub confirmed: bool,
}

/// Employer credit for paid family and medical leave (Form 8994)
/// For wages this business paid to qualifying employees on family and medical leave under its written policy, or premiums for a paid leave insurance policy. The App figures Form 8994 lines 1a–3 and reduces the wages and employee benefit programs deductions by the credit.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PaidLeaveCreditPage {
    /// Employees paid while on family and medical leave (wage method)
    /// Leave empty when you claim only insurance premiums.
    #[libertas_unordered]
    /// ----
    /// Employee
    pub employees: Vec<PaidLeaveEmployeePage>,
    /// Paid family and medical leave insurance (premium method)
    /// Leave empty when you have no such policy. Do not also count, in the wage rows above, leave this policy paid for.
    /// [NullText]
    /// I have no paid family and medical leave insurance policy to claim
    /// [SetText]
    /// The paid family and medical leave insurance policy is
    #[libertas_default($null)]
    pub insurance: Option<PaidLeaveInsurance>,
    /// Written policy and eligibility confirmed
    /// Before the leave was taken you had a written policy giving every qualifying employee at least 2 weeks a year of paid leave for family and medical leave purposes (a proportionate amount for anyone working fewer than 30 hours a week) at no less than 50% of normal wages, without excluding any class of qualifying employees; leave a state or locality requires or pays counts toward these minimums. If any qualifying employee is not covered by the Family and Medical Leave Act (for example, you have fewer than 50 employees), the policy says you will not interfere with or retaliate for the use of the leave, and you complied. Each listed employee had worked for you at least a year (or at least six months, if you chose that period) when the leave was taken. The entered leave was specifically for family and medical leave purposes, not general vacation, personal or sick leave. You have no other business, and this business is not part of a controlled group or a group under common control. Your written policy, payroll, leave and insurance records show it. No leaves this credit out of this estimate.
    pub confirmed: bool,
}

impl From<&PaidLeaveCredit> for PaidLeaveCreditPage {
    fn from(value: &PaidLeaveCredit) -> Self {
        Self {
            employees: value.employees.iter().map(Into::into).collect(),
            insurance: value.insurance.clone(),
            confirmed: value.confirmed,
        }
    }
}
impl From<&PaidLeaveCreditPage> for PaidLeaveCredit {
    fn from(value: &PaidLeaveCreditPage) -> Self {
        Self {
            employees: value.employees.iter().map(Into::into).collect(),
            insurance: value.insurance.clone(),
            confirmed: value.confirmed,
        }
    }
}

impl PaidLeaveCredit {
    pub(crate) fn valid(&self) -> bool {
        self.employees.len() <= 100
            && self.employees.iter().all(PaidLeaveEmployee::valid)
            && self.insurance.as_ref().is_none_or(|i| {
                (1..=100_000_000_000).contains(&i.premiums) && PaidLeaveEmployee::rate_valid(i.rate)
            })
            && !self.duplicate_label()
    }

    fn duplicate_label_index(&self) -> Option<usize> {
        self.employees
            .iter()
            .enumerate()
            .find_map(|(i, e)| self.employees[..i].iter().any(|p| p.label == e.label).then_some(i))
    }
    fn duplicate_label(&self) -> bool {
        self.duplicate_label_index().is_some()
    }

    // The page rules, repeated for the server (the client twins are the
    // validation rules on this record and on PaidLeaveEmployee).
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<ValidationIssue> {
        if let Some(index) = self.duplicate_label_index() {
            return Some(
                ValidationIssue::new("PAID_LEAVE_EMPLOYEE_TWICE")
                    .at_field("label")
                    .in_list("employees", index),
            );
        }
        if self.employees.is_empty() && self.insurance.is_none() {
            return Some(ValidationIssue::new("PAID_LEAVE_NOTHING_CLAIMED").at_field("employees"));
        }
        self.employees.iter().position(PaidLeaveEmployee::above_12_weeks).map(|index| {
            ValidationIssue::new("PAID_LEAVE_ABOVE_12_WEEKS")
                .at_field("leave_hours")
                .in_list("employees", index)
        })
    }

    // Each listed employee matches one W-2 of the business, whose Medicare
    // wages hold the leave wages (and any work opportunity credit wages of the
    // same employee, section 45S(g)); the leave wages are within the wages
    // expense and the premiums, with a Form 8941 claim's, within the employee
    // benefit programs expense (Schedule C lines 26 and 14, cents).
    pub(crate) fn qualified(
        &self,
        payroll: &BusinessPayroll,
        wages_expense: i64,
        benefits_expense: i64,
        work_opportunity: Option<&WorkOpportunityCredit>,
        health_premiums: i64,
    ) -> bool {
        let BusinessPayroll::Yes { statements, .. } = payroll else {
            return false;
        };
        let premiums = self.insurance.as_ref().map_or(0, |i| i.premiums);
        self.valid()
            && self.confirmed
            && self.rule().is_none()
            && self.employees.iter().map(|e| e.leave_wages).sum::<i64>() <= wages_expense
            && premiums + health_premiums <= benefits_expense
            && self.employees.iter().all(|e| {
                let matching: Vec<_> = statements.iter().filter(|s| s.label == e.label).collect();
                let counted = work_opportunity.map_or(0, |w| {
                    w.employees
                        .iter()
                        .filter(|x| x.label == e.label)
                        .map(WorkOpportunityEmployee::paid_in_year)
                        .sum()
                });
                matching.len() == 1 && e.leave_wages + counted <= matching[0].medicare
            })
    }

    // Form 8994 lines 1a (wages) and 1b (premiums), whole dollars.
    pub(crate) fn form_lines(&self) -> [i64; 2] {
        let wages = dollars(self.employees.iter().map(PaidLeaveEmployee::credit_cents).sum());
        let premiums = self
            .insurance
            .as_ref()
            .map_or(0, |i| dollars(paid_leave_credit_cents(i.premiums, i.rate)));
        [wages, premiums]
    }

    // Line 1a: the section 280C(a) reduction of Schedule C line 26.
    pub(crate) fn wage_credit(&self) -> i64 {
        self.form_lines()[0]
    }

    // Line 1b: the section 280C(a) reduction of Schedule C line 14.
    pub(crate) fn premium_credit(&self) -> i64 {
        self.form_lines()[1]
    }

    pub(crate) fn credit(&self) -> i64 {
        self.wage_credit() + self.premium_credit()
    }

    // Lines 1a-3; `k1_credit` is the owner's K-1 code BB total (line 2), whole
    // dollars.
    pub(crate) fn report(&self, business_id: i64, k1_credit: i64, r: &mut FederalResult) {
        let [wages, premiums] = self.form_lines();
        emit_lines(
            r,
            &mut (),
            [
                ("1a", wages, msg!("EXPLAIN_BUSINESS_N_FORM8994_LINE_1A")),
                ("1b", premiums, msg!("EXPLAIN_BUSINESS_N_FORM8994_LINE_1B")),
                ("2", k1_credit, msg!("EXPLAIN_BUSINESS_N_FORM8994_LINE_2")),
                ("3", wages + premiums + k1_credit, msg!("EXPLAIN_BUSINESS_N_FORM8994_LINE_3")),
            ],
            |(printed, amount, explanation), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form8994, printed)),
                    &alloc::format!("business_{business_id}_form8994_line{printed}"),
                    LineValue::MoneyDollars(amount),
                    &["income.business.paid_leave_credit", "income.business.payroll"],
                    "https://www.irs.gov/pub/irs-dft/f8994--dft.pdf",
                    explanation,
                ))
            },
        );
    }
}

impl BusinessCreditHistory {
    fn actual_source_origin(
        &self,
        d: &crate::FederalDraft,
        credit: BusinessCredit,
    ) -> ConditionalCarrybackOrigin {
        let mut state = self
            .credits
            .iter()
            .filter(|item| item.credit == credit && dollars(item.amount) > 0)
            .fold(ConditionalCarrybackOrigin::default(), |state, item| {
                state.merge(
                    ConditionalCarrybackOrigin::from_source(
                        credit,
                        item.carbon_oxide_origin.as_ref(),
                        item.renewable_facility_origin.as_ref(),
                    )
                    .expect("conditional source class"),
                )
            });
        for entry in crate::community::entries(d) {
            let items = match &entry.income {
                crate::FederalIncome::Partnership { data } => data.credits.clone(),
                crate::FederalIncome::Scorp { data } => crate::ScorpK1::from(data).credits,
                _ => continue,
            };
            for item in
                items.iter().filter(|item| item.credit() == credit && dollars(item.amount) > 0)
            {
                state = state.merge(
                    ConditionalCarrybackOrigin::from_source(
                        credit,
                        item.carbon_oxide_origin.as_ref(),
                        item.renewable_facility_origin.as_ref(),
                    )
                    .expect("conditional source class"),
                );
            }
        }
        state
    }

    #[cfg(test)]
    pub(crate) fn source_rule(&self, d: &crate::FederalDraft) -> Option<&'static str> {
        self.source_issue(d).map(|issue| issue.key)
    }
    pub(crate) fn source_issue(&self, d: &crate::FederalDraft) -> Option<ValidationIssue> {
        for (index, item) in self.credits.iter().enumerate() {
            if !matches!(item.source, CreditSource::Bought)
                && resolve_source(d, &item.source).is_none()
            {
                return Some(
                    ValidationIssue::new("BUSINESS_CREDIT_SOURCE_UNAVAILABLE")
                        .at_field("source")
                        .in_list("credits", index),
                );
            }
        }
        for (index, item) in self.carryforwards.iter().enumerate() {
            if !crate::interview::joint(d) && item.filer != CarryFiler::SameFilers {
                return Some(
                    ValidationIssue::new("BUSINESS_CREDIT_FILER_INVALID")
                        .at_field("filer")
                        .in_list("carryforwards", index),
                );
            }
            if !item.credit.research() && item.research_source.is_some() {
                return Some(
                    ValidationIssue::new("BUSINESS_CREDIT_SOURCE_NOT_FOR_CREDIT")
                        .at_field("research_source")
                        .in_list("carryforwards", index),
                );
            }
            if let Some(id) = item.research_source {
                let found = d.income.iter().any(|e| {
                    e.id == id
                        && crate::business_credit_page::research_eligible(e)
                        && match item.filer {
                            CarryFiler::SameFilers => true,
                            CarryFiler::You => e.owner == crate::DocumentOwner::Yours,
                            CarryFiler::Spouse => e.owner == crate::DocumentOwner::Spouses,
                        }
                });
                if !found {
                    return Some(
                        ValidationIssue::new("BUSINESS_CREDIT_SOURCE_UNAVAILABLE")
                            .at_field("research_source")
                            .in_list("carryforwards", index),
                    );
                }
            }
        }
        for (index, item) in self.carrybacks.iter().enumerate() {
            if matches!(
                item.credit,
                BusinessCredit::CarbonOxide | BusinessCredit::RenewableElectricitySpecified
            ) && item.year < 2025
                && self.actual_source_origin(d, item.credit).before2023()
            {
                return Some(
                    ValidationIssue::new("BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED")
                        .at_field("year")
                        .in_list("carrybacks", index),
                );
            }
            if !item.credit.research() && item.research_source.is_some() {
                return Some(
                    ValidationIssue::new("BUSINESS_CREDIT_SOURCE_NOT_FOR_CREDIT")
                        .at_field("research_source")
                        .in_list("carrybacks", index),
                );
            }
            if item.research_source.is_some_and(|id| {
                !d.income
                    .iter()
                    .any(|e| e.id == id && crate::business_credit_page::research_eligible(e))
            }) {
                return Some(
                    ValidationIssue::new("BUSINESS_CREDIT_SOURCE_UNAVAILABLE")
                        .at_field("research_source")
                        .in_list("carrybacks", index),
                );
            }
        }
        None
    }
}

// Resolve an explicit source identity, or the documented sole-source null answer.
pub(crate) fn resolve_source<'a>(
    d: &'a crate::FederalDraft,
    source: &CreditSource,
) -> Option<&'a crate::FederalIncomeEntry> {
    let (selected, kind) = match source {
        CreditSource::Business { business, .. } => (*business, 0),
        CreditSource::Farm { farm, .. } => (*farm, 1),
        CreditSource::Rental { rental, .. } => (*rental, 2),
        CreditSource::Bought => return None,
    };
    let matches = |e: &&crate::FederalIncomeEntry| match kind {
        0 => {
            matches!(&e.income,crate::FederalIncome::Business{data} if matches!(data.activity,crate::BusinessActivity::Ordinary{..}))
        }
        1 => crate::farm::sources(d).any(|s| s.entry.id == e.id),
        _ => matches!(e.income, crate::FederalIncome::Rental { .. }),
    };
    let mut entries =
        d.income.iter().filter(matches).filter(|e| selected.is_none_or(|id| id == e.id));
    let entry = entries.next()?;
    entries.next().is_none().then_some(entry)
}

// Transient source projections from the accepted K-1 owner stages, in cents.
// Form4952 and charity owners still determine the deductions actually allowed.
#[derive(Default)]
pub(crate) struct ResearchItems {
    pub income: i64,
    pub business_loss: i64,
    pub capital_loss: i64,
    pub charity: [i64; 8],
    pub investment_interest: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CreditEntry {
    pub conditional_origin: Option<ConditionalCarrybackOrigin>,
    pub credit: BusinessCredit,
    pub year: u32,
    pub amount: i64,
    pub passive: Option<PassiveWorksheet>,
    pub transfer: i64,
    pub elective: i64,
    pub activity: Option<i64>,
    // Actual source owner before the existing printed-component merge.
    // This is separate from a carry's filer and never inferred from its ID.
    pub source_owner: Option<crate::Owner>,
    pub filer: CarryFiler,
}

pub(crate) use crate::return_carriers::{NativeElectiveGroup, NativeElectiveSources};

impl NativeElectiveGroup {
    fn checked_owners(owners: Option<[i64; 2]>, joint: i64) -> Option<[i64; 2]> {
        let owners = owners?;
        (owners.iter().all(|amount| *amount >= 0)
            && owners[0].checked_add(owners[1]) == Some(joint))
        .then_some(owners)
    }
    pub(crate) fn use_bounds(&self, first: usize) -> Option<[i64; 2]> {
        if first > 1 {
            return None;
        }
        if self.used == 0 {
            return Some([0; 2]);
        }
        let allowed = Self::checked_owners(self.allowed_owners, self.allowed)?;
        let elected = Self::checked_owners(self.elected_owners, self.elected)?;
        let other = 1 - first;
        if self.elective == self.elected {
            // Every elected amount is paid. The existing used-credit answer
            // cannot consume that same owner's already reserved payment.
            let capacity =
                [allowed[0].checked_sub(elected[0])?, allowed[1].checked_sub(elected[1])?];
            if capacity.iter().any(|amount| *amount < 0) {
                return None;
            }
            return Some([
                phase::floor(0, self.used.checked_sub(capacity[other])?),
                phase::cap(self.used, [capacity[first]]),
            ]);
        }
        if self.allowed.checked_sub(self.used)? != self.elective {
            return None;
        }
        // The remaining allowed pool is entirely elective payment. Reuse
        // one source-bounded line16 answer and derive line17 as its residual.
        let minimum = phase::floor(
            phase::floor(0, self.used.checked_sub(allowed[other])?),
            allowed[first].checked_sub(elected[first])?,
        );
        let maximum = phase::cap(
            self.used,
            [allowed[first], self.used.checked_sub(allowed[other].checked_sub(elected[other])?)?],
        );
        (minimum <= maximum).then_some([minimum, maximum])
    }
    pub(crate) fn net_owners(&self, used: Option<[i64; 2]>) -> Option<[i64; 2]> {
        if self.elective == 0 {
            return Some([0; 2]);
        }
        let elected = Self::checked_owners(self.elected_owners, self.elected)?;
        match elected {
            [amount, 0] if amount >= self.elective => return Some([self.elective, 0]),
            [0, amount] if amount >= self.elective => return Some([0, self.elective]),
            _ => {}
        }
        if self.elective == self.elected {
            return Some(elected);
        }
        if self.allowed.checked_sub(self.used)? != self.elective {
            return None;
        }
        let allowed = Self::checked_owners(self.allowed_owners, self.allowed)?;
        let used = Self::checked_owners(used, self.used)?;
        let net = [allowed[0].checked_sub(used[0])?, allowed[1].checked_sub(used[1])?];
        (net.iter().zip(elected).all(|(net, cap)| (0..=cap).contains(net))
            && net[0].checked_add(net[1]) == Some(self.elective))
        .then_some(net)
    }
}
impl NativeElectiveSources {
    pub(crate) fn valid(&self) -> bool {
        // An explicitly closed empty current-source set is meaningful for
        // native carry-only computations; None remains an unavailable packet.
        if self.groups.iter().enumerate().any(|(index, group)| {
            group.component.is_empty()
                || self.groups[..index].iter().any(|other| other.component == group.component)
        }) {
            return false;
        }
        let Some((joint, used)) =
            self.groups.iter().try_fold((0_i64, 0_i64), |(joint, used), group| {
                let contradictory_full_owners = group.allowed == group.amount
                    && group.allowed_owners.zip(group.elected_owners).is_some_and(
                        |(allowed, elected)| {
                            elected
                                .iter()
                                .zip(allowed)
                                .any(|(elected, capacity)| *elected > capacity)
                        },
                    );
                if contradictory_full_owners
                    || group.amount < 0
                    || !(0..=group.amount).contains(&group.allowed)
                    || !(0..=group.amount).contains(&group.elected)
                    || !(0..=group.allowed).contains(&group.used)
                    || !(0..=group.elected).contains(&group.elective)
                    || group.used.checked_add(group.elective)? > group.allowed
                    || group.allowed_owners.is_some()
                        && NativeElectiveGroup::checked_owners(group.allowed_owners, group.allowed)
                            .is_none()
                    || group.elected_owners.is_some()
                        && NativeElectiveGroup::checked_owners(group.elected_owners, group.elected)
                            .is_none()
                {
                    return None;
                }
                Some((joint.checked_add(group.elective)?, used.checked_add(group.used)?))
            })
        else {
            return false;
        };
        joint == self.joint && used <= self.used_joint && self.used_joint >= 0
    }
}

#[derive(Clone, Copy)]
struct CreditOrigins {
    amount: Option<[i64; 2]>,
    elected: Option<[i64; 2]>,
}
impl CreditOrigins {
    fn from_entry(entry: &CreditEntry) -> Self {
        let owners = |amount| {
            if amount < 0 {
                None
            } else if amount == 0 {
                Some([0; 2])
            } else {
                entry.source_owner.map(|owner| crate::kernel::attribution::exclusive(amount, owner))
            }
        };
        Self { amount: owners(entry.amount), elected: owners(entry.elective) }
    }
    fn merge(&mut self, other: Self) {
        let add = |left: Option<[i64; 2]>, right: Option<[i64; 2]>| {
            let (left, right) = (left?, right?);
            Some([left[0].checked_add(right[0])?, left[1].checked_add(right[1])?])
        };
        self.amount = add(self.amount, other.amount);
        self.elected = add(self.elected, other.elected);
    }
    fn allowed(self, amount: i64, allowed: i64) -> Option<[i64; 2]> {
        let owners = self.amount?;
        if owners[0].checked_add(owners[1])? != amount || !(0..=amount).contains(&allowed) {
            return None;
        }
        if allowed == amount {
            Some(owners)
        } else if allowed == 0 {
            Some([0; 2])
        } else {
            // A merged passive allowance is not a per-owner allowance.
            crate::kernel::attribution::sole_contributor(allowed, owners)
        }
    }
}

/// Individual Form 3800 Parts I–IV for the supported component credits.
/// Section B receives supported K-1 empowerment-zone credits; an own Form8844
/// computation remains unavailable. Part I line5 and Part II line35 (incoming
/// carrybacks from2027) belong to an amended2026 return.
pub(crate) struct TaxCapacity {
    pub status: FederalStatus,
    pub spouse_without_credits: bool,
    pub regular_tax_before_credits: i64,
    pub alternative_minimum_tax: i64,
    pub tentative_minimum_tax: i64,
    pub foreign_tax_credit: i64,
    pub other_nonrefundable_credits: i64,
    pub passive: PassiveCreditCapacity,
    pub research_numerators: Vec<(i64, i64)>,
    pub taxable_income: i64,
    pub separate_ratios: Option<[i64; 2]>,
}

// Form 8582-CR inputs figured with the return's regular tax method (the
// line 6 and line 35 worksheets of i8582cr), whole dollars.
pub(crate) struct PassiveCreditCapacity {
    // Line 6: the tax on taxable income less the tax on it without the net
    // passive income.
    pub net_passive_income_tax: i64,
    // Line 35 step E: $25,000 ($12,500 married filing separately living apart)
    // less the rental losses Form 8582 line 9 already let you deduct; zero
    // when married filing separately and living together (Part IV closed).
    pub remaining_allowance: i64,
    // Line 35 step H: the tax on taxable income less the tax on it reduced by
    // the remaining allowance.
    pub remaining_allowance_tax: i64,
    // Part III lines 21-27: the rehabilitation credit's special allowance.
    pub rehabilitation: RehabilitationAllowance,
    pub active: RehabilitationAllowance,
}

// Form 8582-CR Part III lines 21-27 (the $25,000 special allowance for the
// rehabilitation credit, section 469(i)(6)(B)), whole dollars. `completed` is
// false when Part III is not completed (married filing separately living
// together); every line is then zero.
#[derive(Clone, Copy, Default)]
pub(crate) struct RehabilitationAllowance {
    pub completed: bool,
    // Line 21: $250,000, or $125,000 married filing separately living apart.
    pub threshold: i64,
    // Line 22: modified adjusted gross income, not below zero.
    pub income: i64,
    // Line 23: line 21 less line 22.
    pub excess: i64,
    // Line 24: half of line 23, at most $25,000 ($12,500 married filing
    // separately).
    pub limit: i64,
    // Line 25: the rental losses Form 8582 line 9 let you deduct.
    pub used: i64,
    // Line 26: line 24 less line 25.
    pub remaining: i64,
    // Line 27: the tax on taxable income less the tax on it reduced by line 26.
    pub tax: i64,
}

const F8582CR: &str = "https://www.irs.gov/pub/irs-pdf/i8582cr.pdf";

fn passive_line(r: &mut FederalResult, printed: &str, amount: i64, explanation: crate::Message) {
    form_line(
        r,
        FormLine::new(TaxForm::Form8582Cr, printed),
        &alloc::format!("form8582cr_line_{printed}"),
        amount,
        &[
            "credits.business_credits",
            "income.business",
            "income.partnership",
            "income.scorp",
            "income_tax",
        ],
        F8582CR,
        explanation,
    );
}

// Form 8582-CR for the individual's passive activity credits that Form 3800
// takes: Part I (lines 2a-7), Part III (lines 17-30, the rehabilitation
// credit's special allowance; Part II is empty), Part IV (lines 31-37, for the
// credits of Worksheets 3 and 4 only) and Worksheets 6-9. Returns each entry's
// amount that Form 3800 may use: a nonpassive credit in full, a passive credit
// as far as Form 8582-CR allows it.
pub(crate) fn passive_credit_limit(
    entries: &[CreditEntry],
    capacity: &PassiveCreditCapacity,
    r: &mut FederalResult,
) -> Vec<i64> {
    let sum = |worksheet: PassiveWorksheet, current: bool| -> i64 {
        entries
            .iter()
            .filter(|e| e.passive == Some(worksheet) && (e.year == 2026) == current)
            .map(|e| e.amount)
            .sum()
    };
    let l1a = sum(PassiveWorksheet::RentalActive, true);
    let l1b = sum(PassiveWorksheet::RentalActive, false);
    let l1c = l1a + l1b;
    let l2a = sum(PassiveWorksheet::Rehabilitation, true)
        + sum(PassiveWorksheet::LowIncomeHousingBefore1990, true);
    let l2b = sum(PassiveWorksheet::Rehabilitation, false)
        + sum(PassiveWorksheet::LowIncomeHousingBefore1990, false);
    let l2c = l2a + l2b;
    let l3a = sum(PassiveWorksheet::LowIncomeHousing, true);
    let l3b = sum(PassiveWorksheet::LowIncomeHousing, false);
    let l3c = l3a + l3b;
    let l4a = sum(PassiveWorksheet::Other, true);
    let l4b = sum(PassiveWorksheet::Other, false);
    let l4c = l4a + l4b;
    let l5 = l1c + l2c + l3c + l4c;
    let mut allowed: Vec<i64> = entries.iter().map(|e| e.amount).collect();
    if l5 == 0 {
        return allowed;
    }
    let l6 = capacity.net_passive_income_tax;
    let l7 = positive(l5 - l6);
    // Part III: the rehabilitation credits take the special allowance after
    // the tax on net passive income (Part II, rental real estate with active
    // participation, is empty: line 16 is zero).
    let allowance = &capacity.rehabilitation;
    let l8 = phase::cap(l1c, [l7]);
    let l16 = if capacity.active.completed { phase::cap(l8, [capacity.active.tax]) } else { 0 };
    let l17 = l7;
    let l18 = l16;
    let l19 = l17 - l18;
    let l20 = phase::cap(l2c, [l19]);
    let l28 = l18;
    let l29 = positive(allowance.tax - l28);
    let l30 = if l7 > 0 && l2c > 0 && allowance.completed { phase::cap(l20, [l29]) } else { 0 };
    // Lines 31-36: the credits left after the tax on net passive income and
    // the rehabilitation allowance, and the low-income housing credit's part
    // of the remaining special allowance. Line 7 of zero allows every credit.
    let l31 = l19;
    let l32 = l30;
    let l33 = l31 - l32;
    let l34 = phase::cap(l3c, [l33]);
    let l35 = positive(capacity.remaining_allowance_tax - (l16 + l30));
    let l36 = if l7 > 0 && capacity.remaining_allowance > 0 { phase::cap(l34, [l35]) } else { 0 };
    let l37 = if l7 == 0 { l5 } else { phase::cap(l5, [l6 + l16 + l30 + l36]) };
    // Worksheet 6 allocates line 30 over the rehabilitation credits, Worksheet
    // 7 line 36 over the low-income housing credits; Worksheet 8 spreads the
    // unallowed credit over the other passive credits and what Worksheets 6
    // and 7 leave of the others (column (d)).
    let indexes = |worksheet: PassiveWorksheet| -> Vec<usize> {
        (0..entries.len()).filter(|&i| entries[i].passive == Some(worksheet)).collect()
    };
    let mut rehabilitation = indexes(PassiveWorksheet::Rehabilitation);
    rehabilitation.extend(indexes(PassiveWorksheet::LowIncomeHousingBefore1990));
    let active = indexes(PassiveWorksheet::RentalActive);
    let housing = indexes(PassiveWorksheet::LowIncomeHousing);
    let other = indexes(PassiveWorksheet::Other);
    let active_amounts: Vec<i64> = active.iter().map(|&i| entries[i].amount).collect();
    let worksheet5 = crate::kernel::ratio::allocate_remaining(l16, &active_amounts);
    let rehabilitation_amounts: Vec<i64> =
        rehabilitation.iter().map(|&i| entries[i].amount).collect();
    let housing_amounts: Vec<i64> = housing.iter().map(|&i| entries[i].amount).collect();
    let worksheet6 = crate::kernel::ratio::allocate_remaining(l30, &rehabilitation_amounts);
    let worksheet7 = crate::kernel::ratio::allocate_remaining(l36, &housing_amounts);
    let weights: Vec<i64> = active_amounts
        .iter()
        .zip(&worksheet5)
        .map(|(a, t)| a - t)
        .chain(other.iter().map(|&i| entries[i].amount))
        .chain(rehabilitation_amounts.iter().zip(&worksheet6).map(|(amount, taken)| amount - taken))
        .chain(housing_amounts.iter().zip(&worksheet7).map(|(amount, taken)| amount - taken))
        .collect();
    let unallowed = crate::kernel::ratio::allocate_remaining(l5 - l37, &weights);
    for (&i, unallowed) in
        active.iter().chain(other.iter()).chain(&rehabilitation).chain(&housing).zip(unallowed)
    {
        allowed[i] = entries[i].amount - unallowed;
    }
    for (printed, amount, explanation) in [
        ("2a", l2a, msg!("EXPLAIN_FORM8582CR_LINE_2A")),
        ("2b", l2b, msg!("EXPLAIN_FORM8582CR_LINE_2B")),
        ("2c", l2c, msg!("EXPLAIN_FORM8582CR_LINE_2C")),
        ("3a", l3a, msg!("EXPLAIN_FORM8582CR_LINE_3A")),
        ("3b", l3b, msg!("EXPLAIN_FORM8582CR_LINE_3B")),
        ("3c", l3c, msg!("EXPLAIN_FORM8582CR_LINE_3C")),
        ("4a", l4a, msg!("EXPLAIN_FORM8582CR_LINE_4A")),
        ("4b", l4b, msg!("EXPLAIN_FORM8582CR_LINE_4B")),
        ("4c", l4c, msg!("EXPLAIN_FORM8582CR_LINE_4C")),
        ("5", l5, msg!("EXPLAIN_FORM8582CR_LINE_5")),
        ("6", l6, msg!("EXPLAIN_FORM8582CR_LINE_6")),
        ("7", l7, msg!("EXPLAIN_FORM8582CR_LINE_7")),
    ] {
        // Lines of a worksheet no credit uses stay off the form.
        if amount != 0 || !printed.starts_with('2') {
            passive_line(r, printed, amount, explanation);
        }
    }
    if l1c > 0 {
        for (printed, amount) in [
            ("1a", l1a),
            ("1b", l1b),
            ("1c", l1c),
            ("8", l8),
            ("9", capacity.active.threshold),
            ("10", capacity.active.income),
            ("11", capacity.active.excess),
            ("12", capacity.active.limit),
            ("13", capacity.active.used),
            ("14", capacity.active.remaining),
            ("15", capacity.active.tax),
            ("16", l16),
        ] {
            if capacity.active.completed || matches!(printed, "1a" | "1b" | "1c" | "8" | "16") {
                passive_line(
                    r,
                    printed,
                    amount,
                    crate::Message::new(
                        "EXPLAIN_FORM8582CR_ACTIVE",
                        alloc::vec![crate::MessageArg::Text(printed.into())],
                    ),
                );
            }
        }
    }
    if l7 > 0 && l2c > 0 {
        for (printed, amount, explanation) in [
            ("17", l17, msg!("EXPLAIN_FORM8582CR_LINE_17")),
            ("18", l18, msg!("EXPLAIN_FORM8582CR_LINE_18")),
            ("19", l19, msg!("EXPLAIN_FORM8582CR_LINE_19")),
            ("20", l20, msg!("EXPLAIN_FORM8582CR_LINE_20")),
            ("21", allowance.threshold, msg!("EXPLAIN_FORM8582CR_LINE_21")),
            ("22", allowance.income, msg!("EXPLAIN_FORM8582CR_LINE_22")),
            ("23", allowance.excess, msg!("EXPLAIN_FORM8582CR_LINE_23")),
            ("24", allowance.limit, msg!("EXPLAIN_FORM8582CR_LINE_24")),
            ("25", allowance.used, msg!("EXPLAIN_FORM8582CR_LINE_25")),
            ("26", allowance.remaining, msg!("EXPLAIN_FORM8582CR_LINE_26")),
            ("27", allowance.tax, msg!("EXPLAIN_FORM8582CR_LINE_27")),
            ("28", l28, msg!("EXPLAIN_FORM8582CR_LINE_28")),
            ("29", l29, msg!("EXPLAIN_FORM8582CR_LINE_29")),
            ("30", l30, msg!("EXPLAIN_FORM8582CR_LINE_30")),
        ] {
            if allowance.completed || printed.parse::<u32>().is_ok_and(|n| !(21..=27).contains(&n))
            {
                passive_line(r, printed, amount, explanation);
            }
        }
    }
    if l7 > 0 {
        passive_line(r, "31", l31, msg!("EXPLAIN_FORM8582CR_LINE_31"));
        passive_line(r, "32", l32, msg!("EXPLAIN_FORM8582CR_LINE_32"));
        passive_line(r, "33", l33, msg!("EXPLAIN_FORM8582CR_LINE_33"));
        passive_line(r, "34", l34, msg!("EXPLAIN_FORM8582CR_LINE_34"));
        passive_line(r, "35", l35, msg!("EXPLAIN_FORM8582CR_LINE_35"));
        passive_line(r, "36", l36, msg!("EXPLAIN_FORM8582CR_LINE_36"));
    }
    passive_line(r, "37", l37, msg!("EXPLAIN_FORM8582CR_LINE_37"));
    allowed
}

const F3800: &str = "https://www.irs.gov/pub/irs-dft/i3800--dft.pdf";

fn part_line(r: &mut FederalResult, printed: &str, amount: i64, explanation: crate::Message) {
    form_line(
        r,
        FormLine::new(TaxForm::Form3800, printed),
        &alloc::format!("form_3800_line_{printed}"),
        amount,
        &[
            "credits.business_credits",
            "income.business",
            "income.partnership",
            "income.scorp",
            "income_tax",
            "alternative_minimum_tax",
        ],
        F3800,
        explanation,
    );
}

// Uses `budget` on the entries first-in, first-out: the earliest year first
// and, within one year, the instructions' credit order. Returns each entry's
// used amount in the entries' own order.
fn fifo(entries: &[CreditEntry], budget: i64) -> Vec<i64> {
    use crate::kernel::carry::{Amount, Dollars, Entry, consume_by_key};
    let ledger: Vec<_> = entries
        .iter()
        .map(|entry| Entry {
            key: entry,
            amount: Amount::<Dollars>::new(entry.amount).expect("validated business credit"),
            // Expiry is handled by the Form3800 owner after its current use.
            last_use_year: None,
        })
        .collect();
    consume_by_key(
        &ledger,
        Amount::new(budget).expect("nonnegative credit capacity"),
        2026,
        |entry| (entry.year, entry.credit.order()),
    )
    .expect("credit order is a permutation")
    .into_iter()
    .map(|entry| entry.used.get())
    .collect()
}

pub(crate) fn calculate(
    history: Option<&EmployerTipFicaCarryforward>,
    mut entries: Vec<CreditEntry>,
    carrybacks: &[(BusinessCredit, u32, i64, Option<i64>)],
    TaxCapacity {
        status,
        spouse_without_credits,
        regular_tax_before_credits,
        alternative_minimum_tax,
        tentative_minimum_tax,
        foreign_tax_credit,
        other_nonrefundable_credits,
        passive,
        research_numerators,
        taxable_income,
        separate_ratios,
    }: TaxCapacity,
    r: &mut FederalResult,
) -> (i64, i64) {
    if let Some(history) = history {
        crate::require_localized!(
            r,
            history.valid()
                && history.ordinary
                && history.prior_status == status
                && history.original_credit > 0
                && history.original_credit
                    == history.used_origin_year
                        + history.used_origin_prior_year_carryback
                        + history.carryforward_to_current_year,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            "Employer tip FICA credit carryforward",
            "ISSUE_USE_ONE_UNCHANGED_FILER_NONPASSIVE_PRIOR_YEAR_FORM_8846_CREDIT_WITH_COMPL_EB27FEE3",
            "Use one unchanged-filer, nonpassive 2025 Form 8846 credit with complete Form 3800 and carryback records. Enter whole-dollar Form 3800 amounts; the 2025 credit must exactly equal its 2025 use, 2024 carryback use and 2026 balance. A changed filing status needs the separate-liability computation; adjusted, passive or transferred histories need a separate Form 3800 review.",
        );
        entries.push(CreditEntry {
            source_owner: None,
            conditional_origin: None,
            transfer: 0,
            elective: 0,
            activity: None,
            filer: crate::CarryFiler::SameFilers,
            credit: BusinessCredit::EmployerTipFica,
            year: 2025,
            amount: dollars(history.carryforward_to_current_year),
            passive: None,
        });
    }
    // One entry per credit, year and passive class (Part III and IV lines).
    let mut merged: Vec<CreditEntry> = Vec::new();
    let mut origins: Vec<CreditOrigins> = Vec::new();
    for entry in entries.into_iter().filter(|e| e.amount > 0 || e.transfer != 0 || e.elective != 0)
    {
        let origin = CreditOrigins::from_entry(&entry);
        match merged.iter_mut().enumerate().find(|(_, m)| {
            m.credit == entry.credit
                && m.year == entry.year
                && m.passive == entry.passive
                && (!entry.credit.research() || m.activity == entry.activity)
                && m.filer == entry.filer
        }) {
            Some((index, existing)) => {
                origins[index].merge(origin);
                if existing.source_owner != entry.source_owner {
                    existing.source_owner = None;
                }
                if matches!(
                    entry.credit,
                    BusinessCredit::CarbonOxide | BusinessCredit::RenewableElectricitySpecified
                ) {
                    existing.conditional_origin = Some(
                        existing
                            .conditional_origin
                            .unwrap_or_else(ConditionalCarrybackOrigin::unknown)
                            .merge(
                                entry
                                    .conditional_origin
                                    .unwrap_or_else(ConditionalCarrybackOrigin::unknown),
                            ),
                    );
                }
                existing.amount += entry.amount;
                existing.transfer += entry.transfer;
                existing.elective += entry.elective;
            }
            None => {
                origins.push(origin);
                merged.push(entry);
            }
        }
    }
    let entries = merged;
    if entries.is_empty() && carrybacks.is_empty() && history.is_none() {
        return (0, 0);
    }
    // Form 8582-CR limits each passive activity credit; Form 3800 lines 3 and
    // 33 take what it allows.
    let allowed = passive_credit_limit(&entries, &passive, r);
    let sum = |specified: bool, current: bool, passive: bool| -> i64 {
        entries
            .iter()
            .filter(|e| {
                e.credit.specified(e.year) == specified
                    && !e.credit.info().section_b
                    && (e.year == 2026) == current
                    && e.passive.is_some() == passive
            })
            .map(|e| e.amount)
            .sum()
    };
    let allowed_sum = |specified: bool| -> i64 {
        entries
            .iter()
            .zip(&allowed)
            .filter(|(e, _)| {
                e.credit.specified(e.year) == specified
                    && !e.credit.info().section_b
                    && e.passive.is_some()
            })
            .map(|(_, a)| *a)
            .sum()
    };
    // Part I.
    let l1 = sum(false, true, false);
    let l2 = sum(false, true, true) + sum(false, false, true);
    let l3 = allowed_sum(false);
    let l4 = sum(false, false, false);
    let l6 = l1 + l3 + l4;
    // Part II Section A.
    let l7 = regular_tax_before_credits;
    let l8 = alternative_minimum_tax;
    let l9 = l7 + l8;
    let l10c = foreign_tax_credit + other_nonrefundable_credits;
    let l11 = positive(l9 - l10c);
    let l12 = positive(l7 - l10c);
    // Section 38(c)(6)(A): $12,500 for a married-separate filer unless the
    // spouse has no business credit, carryforward or carryback.
    let threshold =
        if status == FederalStatus::Separate && !spouse_without_credits { 12_500 } else { 25_000 };
    let l13 = fraction(positive(l12 - threshold), 25, 100);
    let l14 = tentative_minimum_tax;
    let l15 = phase::floor(l13, l14);
    let l16 = positive(l11 - l15);
    // Research capacity belongs to the actual originating business, shared across
    // lines 1c/4i and all vintages. Explicit historical null has no capacity.
    let numerator_total: i64 = research_numerators.iter().map(|(_, n)| positive(*n)).sum();
    let denominator = taxable_income.max(numerator_total);
    let research_caps: Vec<(i64, i64)> = research_numerators
        .iter()
        .map(|(id, n)| {
            (*id, if denominator > 0 { fraction(l11, positive(*n), denominator) } else { 0 })
        })
        .collect();
    let mut research_used = alloc::vec![0;research_caps.len()];
    let mut used = alloc::vec![0;entries.len()];
    let mut use_bucket = |kind: u8, budget: i64, limit: i64| {
        let mut indexes: Vec<usize> = (0..entries.len())
            .filter(|&i| {
                let e = &entries[i];
                match kind {
                    1 => e.credit.info().section_b,
                    2 => e.credit.specified(e.year) && !e.credit.info().section_b,
                    _ => !e.credit.specified(e.year) && !e.credit.info().section_b,
                }
            })
            .collect();
        indexes.sort_by_key(|&i| (entries[i].year, entries[i].credit.order(), i));
        let mut left = budget;
        let mut filer_used = [0; 2];
        for i in indexes {
            let e = &entries[i];
            let research = e
                .activity
                .and_then(|id| research_caps.iter().position(|(source, _)| *source == id));
            let research_left = if e.credit.research() {
                research.map_or(0, |j| positive(research_caps[j].1 - research_used[j]))
            } else {
                allowed[i]
            };
            let filer = match e.filer {
                CarryFiler::SameFilers => None,
                CarryFiler::You => Some(0),
                CarryFiler::Spouse => Some(1),
            };
            let filer_left = filer.map_or(allowed[i], |j| {
                separate_ratios.map_or(0, |ratios| {
                    positive(fraction(limit, ratios[j], 10_000) - filer_used[j])
                })
            });
            let take = fifo(
                &[CreditEntry {
                    amount: phase::cap(allowed[i], [research_left, filer_left]),
                    ..e.clone()
                }],
                left,
            )[0];
            used[i] = take;
            left = crate::kernel::carry::native_balance(left, take);
            if let Some(j) = research
                && e.credit.research()
            {
                research_used[j] += take;
            }
            if let Some(j) = filer {
                filer_used[j] += take;
            }
        }
        budget - left
    };
    let l17 = use_bucket(0, l16, l16);
    // Section B has its own statutory limitation and FIFO ledger.
    let l18 = fraction(l14, 75, 100);
    let l19 = phase::floor(l13, l18);
    let l20 = positive(l11 - l19);
    let l21 = positive(l20 - l17);
    let l22: i64 = entries
        .iter()
        .filter(|e| e.credit.info().section_b && e.passive.is_none())
        .map(|e| e.amount)
        .sum();
    let l23: i64 = entries
        .iter()
        .filter(|e| e.credit.info().section_b && e.passive.is_some())
        .map(|e| e.amount)
        .sum();
    let l24: i64 = entries
        .iter()
        .zip(&allowed)
        .filter(|(e, _)| e.credit.info().section_b && e.passive.is_some())
        .map(|(_, a)| *a)
        .sum();
    let l25 = l22 + l24;
    let l26 = use_bucket(1, phase::cap(l21, [l25]), l21);
    let l27 = positive(l11 - l13);
    let l28 = l17 + l26;
    let l29 = positive(l27 - l28);
    let l30 = sum(true, true, false);
    let l32 = sum(true, true, true) + sum(true, false, true);
    let l33 = allowed_sum(true);
    let l34 = sum(true, false, false);
    let l36 = l30 + l33 + l34;
    let l37 = use_bucket(2, l29, l29);
    let l38 = l28 + l37;
    let elective: Vec<i64> = entries
        .iter()
        .zip(&allowed)
        .zip(&used)
        .map(|((e, a), u)| phase::cap(e.elective, [positive(a - u)]))
        .collect();
    let total_elective: i64 = elective.iter().sum();
    // Current credit ownership applies to line16 even without an elective
    // payment. Reuse this same native pool/allowance/use projection; zero
    // elected amounts reserve no capacity and create no line17 question.
    let source_group = |entry: &CreditEntry| entry.year == 2026 || entry.elective > 0;
    // Also supply the closed empty current-source set for carry-only results,
    // preserving their separate history context rather than treating a missing
    // packet as permission to guess which context produced a positive credit.
    r.elective_sources = Some(alloc::boxed::Box::new(NativeElectiveSources {
        joint: total_elective,
        used_joint: l38,
        groups: entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| source_group(entry))
            .map(|(i, entry)| NativeElectiveGroup {
                component: {
                    let component = alloc::format!(
                        "business_credit_use_{}_{}_{}_{}",
                        entry.credit.key(),
                        entry.year,
                        entry.passive.map_or(0, |worksheet| worksheet as u8 + 1),
                        entry.filer as u8
                    );
                    // Research's existing native merger keeps activities
                    // separate. Use that actual identity for the same pool;
                    // it never supplies or implies a person's ownership.
                    if entry.credit.research() {
                        match entry.activity {
                            Some(id) => alloc::format!("{component}_activity_{id}"),
                            None => alloc::format!("{component}_activity_unavailable"),
                        }
                    } else {
                        component
                    }
                },
                label: entry.credit.form_message_key(),
                amount: entry.amount,
                elected: entry.elective,
                allowed: allowed[i],
                used: used[i],
                elective: elective[i],
                allowed_owners: origins[i].allowed(entry.amount, allowed[i]),
                elected_owners: origins[i].elected,
            })
            .collect(),
    }));
    if separate_ratios.is_none() && entries.iter().any(|e| e.filer != CarryFiler::SameFilers) {
        crate::federal_rules::note_once_scoped(
            r,
            msg!("WARN_BUSINESS_CREDIT_CARRYFORWARDS_LEFT_OUT"),
            crate::IssueContext::Section(crate::TaxSection::MoreCredits),
        );
    }
    let unused_origin = |credit| {
        entries
            .iter()
            .enumerate()
            .filter(|(i, e)| {
                e.credit == credit
                    && e.year == 2026
                    && crate::kernel::carry::native_balance(
                        crate::kernel::carry::native_balance(allowed[*i], used[*i]),
                        elective[*i],
                    ) > 0
            })
            .fold(ConditionalCarrybackOrigin::default(), |state, (_, e)| {
                state
                    .merge(e.conditional_origin.unwrap_or_else(ConditionalCarrybackOrigin::unknown))
            })
    };
    let mut available_carrybacks = Vec::new();
    for &(credit, year, amount, source) in carrybacks {
        let origin = unused_origin(credit);
        if matches!(
            credit,
            BusinessCredit::CarbonOxide | BusinessCredit::RenewableElectricitySpecified
        ) && year < 2025
            && !origin.extended()
        {
            if origin.mixed() {
                crate::require_localized!(
                    r,
                    false,
                    &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                    "Physical-source extended carryback",
                    if credit == BusinessCredit::CarbonOxide {
                        "CARBON_OXIDE_MIXED_CARRYBACK_ATTRIBUTION_REQUIRED"
                    } else {
                        "RENEWABLE_FACILITY_MIXED_CARRYBACK_ATTRIBUTION_REQUIRED"
                    },
                    "The mixed original service dates do not identify which unused source credit qualifies for the extended carryback.",
                );
            } else if origin.unknown_date() {
                crate::federal_rules::note_once_scoped(
                    r,
                    if credit == BusinessCredit::CarbonOxide {
                        msg!("WARN_CARBON_OXIDE_EXTENDED_CARRYBACK_NEEDS_DATE")
                    } else {
                        msg!("WARN_RENEWABLE_FACILITY_EXTENDED_CARRYBACK_NEEDS_DATE")
                    },
                    crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                );
            } else if origin.before2023() {
                crate::require_localized!(
                    r,
                    false,
                    &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
                    "Physical-source carryback year",
                    "BUSINESS_CARRYBACK_YEAR_NOT_ALLOWED",
                    "The requested year is outside the carryback period for this actual source origin.",
                );
            } else {
                // No unused current source: preserve the existing amount-limit
                // diagnostic instead of inventing an origin or a date boundary.
                available_carrybacks.push((credit, year, amount, source));
            }
        } else {
            available_carrybacks.push((credit, year, amount, source));
        }
    }
    let carrybacks = available_carrybacks.as_slice();
    // Carrybacks reduce only this year's credit that is not passive, or that
    // Form 8582-CR allowed, left after 2026 use.
    for (index, (credit, _, _, source)) in carrybacks.iter().enumerate() {
        // Each credit once: its carrybacks to every year together cannot exceed
        // what 2026 tax use leaves.
        if carrybacks[..index]
            .iter()
            .any(|(prior, _, _, prior_source)| prior == credit && prior_source == source)
        {
            continue;
        }
        let amount: i64 = carrybacks
            .iter()
            .filter(|(c, _, _, origin)| c == credit && origin == source)
            .map(|(_, _, a, _)| a)
            .sum();
        let unused: i64 = entries
            .iter()
            .zip(allowed.iter().zip(&used))
            .filter(|(e, _)| {
                e.credit == *credit
                    && e.year == 2026
                    && (!credit.research() || e.activity == *source)
            })
            .map(|(e, (a, u))| a - u - phase::cap(e.elective, [positive(a - u)]))
            .sum();
        crate::federal_rules::require_message(
            r,
            amount <= unused,
            &crate::IssueContext::Section(crate::TaxSection::MoreCredits),
            #[cfg(test)]
            "General business credit carryback",
            #[cfg(test)]
            alloc::format!(
                "The carryback of the {} credit cannot exceed the 2026 credit left after 2026 Form 3800 tax use (${unused}).",
                credit.form()
            ),
            crate::Message::new(
                "ISSUE_CREDIT_CARRYBACK_LIMIT",
                alloc::vec![
                    crate::MessageArg::Resource(credit.form_message_key().into()),
                    crate::MessageArg::Currency {
                        currency_code: "USD".into(),
                        scale: 0,
                        minor_units: unused,
                    },
                ],
            ),
        );
    }
    for (printed, amount, explanation) in [
        ("1", l1, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N")),
        ("2", l2, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_2")),
        ("3", l3, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_3")),
        ("4", l4, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_4")),
        ("6", l6, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_5")),
        ("7", l7, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_6")),
        ("8", l8, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_7")),
        ("9", l9, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_8")),
        ("10c", l10c, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_10C")),
        ("11", l11, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_9")),
        ("12", l12, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_10")),
        ("13", l13, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_11")),
        ("14", l14, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_12")),
        ("15", l15, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_13")),
        ("16", l16, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_14")),
        ("17", l17, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_15")),
        ("26", l26, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_16")),
        ("27", l27, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_17")),
        ("28", l28, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_18")),
        ("29", l29, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_19")),
        ("30", l30, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_20")),
        ("32", l32, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_21")),
        ("33", l33, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_22")),
        ("34", l34, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_23")),
        ("36", l36, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_24")),
        ("37", l37, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_25")),
        ("38", l38, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_N_26")),
    ] {
        part_line(r, printed, amount, explanation);
    }
    if l22 != 0 || l24 != 0 {
        for (printed, amount) in [
            ("18", l18),
            ("19", l19),
            ("20", l20),
            ("21", l21),
            ("22", l22),
            ("23", l23),
            ("24", l24),
            ("25", l25),
        ] {
            part_line(
                r,
                printed,
                amount,
                crate::Message::new(
                    "EXPLAIN_FORM3800_SECTION_B",
                    alloc::vec![crate::MessageArg::Text(printed.into())],
                ),
            );
        }
    }
    // The carryback of a credit to each earlier return goes against its 2026
    // entries in order, the earliest return first.
    let mut carried = alloc::vec![0; entries.len()];
    let mut carried_years: Vec<(usize, u32, i64)> = Vec::new();
    let mut ordered: Vec<&(BusinessCredit, u32, i64, Option<i64>)> = carrybacks.iter().collect();
    ordered.sort_by_key(|(_, year, _, _)| *year);
    for (credit, year, amount, source) in ordered {
        let mut left = *amount;
        for (i, e) in entries.iter().enumerate() {
            if e.credit == *credit
                && e.year == 2026
                && (!credit.research() || e.activity == *source)
            {
                let take =
                    phase::cap(left, [positive(allowed[i] - used[i] - elective[i] - carried[i])]);
                if take > 0 {
                    carried[i] += take;
                    carried_years.push((i, *year, take));
                }
                left = crate::kernel::carry::native_balance(left, take);
            }
        }
        if let Some(i) = (0..entries.len()).rev().find(|&i| {
            entries[i].credit == *credit
                && entries[i].year == 2026
                && (!credit.research() || entries[i].activity == *source)
        }) && left > 0
        {
            carried[i] += left;
            carried_years.push((i, *year, left));
        }
    }
    report_columns(&entries, &allowed, &used, &elective, &carried, r);
    for (i, e) in entries.iter().enumerate() {
        let suffix = if e.credit.research() || e.filer != CarryFiler::SameFilers {
            alloc::format!(
                "_source_{}_filer_{}",
                e.activity.map_or_else(
                    || alloc::string::String::from("none"),
                    |id| alloc::format!("{id}")
                ),
                e.filer as i32
            )
        } else {
            alloc::string::String::new()
        };
        let key = alloc::format!("{}{suffix}", e.credit.key());
        let printed = e.credit.line(e.year);
        let (id, explanation_key) = match (e.year == 2026, e.passive.is_some()) {
            (true, false) => {
                (alloc::format!("{key}_current_year"), "EXPLAIN_BUSINESS_CREDIT_CURRENT_NONPASSIVE")
            }
            (true, true) => (
                alloc::format!("{key}_current_year_passive"),
                "EXPLAIN_BUSINESS_CREDIT_CURRENT_PASSIVE",
            ),
            (false, false) => {
                (alloc::format!("{key}_{}", e.year), "EXPLAIN_BUSINESS_CREDIT_CARRY_NONPASSIVE")
            }
            (false, true) => (
                alloc::format!("{key}_{}_passive", e.year),
                "EXPLAIN_BUSINESS_CREDIT_CARRY_PASSIVE",
            ),
        };
        form_line(
            r,
            FormLine::new(TaxForm::Form3800, printed),
            &alloc::format!("business_credit_{id}"),
            e.amount,
            &["credits.business_credits", "income.business"],
            F3800,
            crate::Message::new(
                explanation_key,
                alloc::vec![
                    crate::MessageArg::Text(printed.into()),
                    crate::MessageArg::Number(i64::from(e.year)),
                    crate::MessageArg::Resource(e.credit.form_message_key().into()),
                ],
            ),
        );
        if e.passive.is_some() {
            // Worksheet 9: the part Form 8582-CR allows joins Form 3800 tax
            // use like a nonpassive credit; the rest waits, still passive.
            line(
                r,
                &alloc::format!("business_credit_{id}_to_next_year"),
                e.amount - allowed[i],
                &["credits.business_credits"],
                "https://www.irs.gov/instructions/i8582cr",
                msg!("EXPLAIN_BUSINESS_CREDIT_N_TO_N"),
            );
            form_line(
                r,
                FormLine::new(TaxForm::Form8582Cr, "37"),
                &alloc::format!("business_credit_{id}_allowed"),
                allowed[i],
                &["credits.business_credits"],
                F8582CR,
                msg!("EXPLAIN_BUSINESS_CREDIT_N_ALLOWED"),
            );
        }
        // The usable part of every entry of this credit and year (a
        // nonpassive credit, or what Form 8582-CR allowed) is used, carried
        // back, expired or carried forward as one set of rows.
        if entries[i + 1..].iter().any(|o| {
            o.credit == e.credit
                && o.year == e.year
                && (!e.credit.research() || o.activity == e.activity)
                && o.filer == e.filer
        }) {
            continue;
        }
        let group: Vec<usize> = (0..entries.len())
            .filter(|&j| {
                entries[j].credit == e.credit
                    && entries[j].year == e.year
                    && (!e.credit.research() || entries[j].activity == e.activity)
                    && entries[j].filer == e.filer
            })
            .collect();
        let usable: i64 = group.iter().map(|&j| allowed[j]).sum();
        if usable == 0 {
            continue;
        }
        let unit = if e.year == 2026 {
            alloc::format!("{key}_current_year")
        } else {
            alloc::format!("{key}_{}", e.year)
        };
        line(
            r,
            &alloc::format!("business_credit_{unit}_used"),
            group.iter().map(|&j| used[j]).sum(),
            &["credits.business_credits"],
            F3800,
            msg!("EXPLAIN_BUSINESS_CREDIT_N_USED"),
        );
        let mut years: Vec<(u32, i64)> = Vec::new();
        for &(index, year, amount) in &carried_years {
            if group.contains(&index) {
                match years.iter_mut().find(|(y, _)| *y == year) {
                    Some((_, total)) => *total += amount,
                    None => years.push((year, amount)),
                }
            }
        }
        emit_lines(r, &mut (), years, |(year, amount), _, _| {
            Some(calculation_row(
                None,
                &alloc::format!("business_credit_{unit}_carryback_{year}"),
                LineValue::MoneyDollars(amount),
                &["credits.business_credits.carrybacks"],
                F3800,
                msg!("EXPLAIN_BUSINESS_CREDIT_N_CARRYBACK_N"),
            ))
        });
        let left = crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            usable - group.iter().map(|&j| used[j] + elective[j]).sum::<i64>(),
            group.iter().map(|&j| carried[j]).sum::<i64>(),
        )
        .get();
        if e.year + 20 <= 2026 {
            line(
                r,
                &alloc::format!("business_credit_{unit}_expired"),
                left,
                &["credits.business_credits.carryforwards"],
                F3800,
                msg!("EXPLAIN_BUSINESS_CREDIT_N_EXPIRED"),
            );
        } else {
            line(
                r,
                &alloc::format!("business_credit_{unit}_to_next_year"),
                left,
                &["credits.business_credits"],
                F3800,
                msg!("EXPLAIN_BUSINESS_CREDIT_N_TO_N_2"),
            );
        }
    }
    // The Form 8846 lines of the dedicated 2025 record and business claim.
    let tip = |year: u32| {
        entries
            .iter()
            .zip(&used)
            .find(|(e, _)| {
                e.credit == BusinessCredit::EmployerTipFica && e.year == year && e.passive.is_none()
            })
            .map(|(e, u)| (e.amount, *u))
    };
    if let Some(history) = history {
        let (available, prior_used) = tip(2025).unwrap_or((0, 0));
        emit_lines(
            r,
            &mut (),
            [
                (
                    "tip_fica_prior_year_credit_origin",
                    dollars(history.original_credit),
                    msg!("EXPLAIN_TIP_FICA_N_CREDIT_ORIGIN"),
                ),
                (
                    "tip_fica_prior_year_credit_used",
                    dollars(history.used_origin_year),
                    msg!("EXPLAIN_TIP_FICA_N_CREDIT_USED"),
                ),
                (
                    "tip_fica_origin_prior_year_carryback_used",
                    dollars(history.used_origin_prior_year_carryback),
                    msg!("EXPLAIN_TIP_FICA_N_CARRYBACK_USED"),
                ),
                (
                    "tip_fica_credit_carryforward_to_next_year",
                    crate::kernel::carry::native_balance(available, prior_used),
                    msg!("EXPLAIN_TIP_FICA_CREDIT_CARRYFORWARD_N"),
                ),
            ],
            |(id, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(amount),
                    &["credits.tip_fica_carryforward", "income_tax", "alternative_minimum_tax"],
                    "https://www.irs.gov/pub/irs-dft/f3800--dft.pdf",
                    explanation,
                ))
            },
        );
    }
    if let Some((available, current_used)) = tip(2026) {
        let carried_back = carrybacks
            .iter()
            .find(|(c, ..)| *c == BusinessCredit::EmployerTipFica)
            .map_or(0, |(_, _, a, _)| *a);
        emit_lines(
            r,
            &mut (),
            [
                ("tip_fica_current_credit", available, msg!("EXPLAIN_TIP_FICA_CURRENT_CREDIT")),
                (
                    "tip_fica_current_credit_used",
                    current_used,
                    msg!("EXPLAIN_TIP_FICA_CURRENT_CREDIT_USED"),
                ),
                (
                    "tip_fica_current_credit_carryback_to_prior_year",
                    carried_back,
                    msg!("EXPLAIN_TIP_FICA_CURRENT_CREDIT_CARRYBACK_N"),
                ),
                (
                    "tip_fica_current_credit_carryforward_to_next_year",
                    crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
                        available - current_used,
                        carried_back,
                    )
                    .get(),
                    msg!("EXPLAIN_TIP_FICA_CURRENT_CREDIT_CARRYFORWARD_N"),
                ),
            ],
            |(id, amount, explanation), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(amount),
                    &[
                        "income.business.employer_tip_fica_credit",
                        "income_tax",
                        "alternative_minimum_tax",
                    ],
                    F3800,
                    explanation,
                ))
            },
        );
    }
    emit_lines(
        r,
        &mut (),
        [
            ("general_business_credit_limit", l29, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT_LIMIT")),
            ("general_business_credit", l38, msg!("EXPLAIN_GENERAL_BUSINESS_CREDIT")),
        ],
        |(id, amount, explanation), _, _| {
            Some(calculation_row(
                None,
                id,
                LineValue::MoneyDollars(amount),
                &["credits.business_credits", "income_tax", "alternative_minimum_tax"],
                F3800,
                explanation,
            ))
        },
    );
    (l38, total_elective)
}

// Aggregate printed columns once; source-specific use and carry rows remain
// separate in the ledger above, even when two sources share a printed line.
fn report_columns(
    entries: &[CreditEntry],
    allowed: &[i64],
    used: &[i64],
    elective: &[i64],
    carried: &[i64],
    r: &mut FederalResult,
) {
    let mut current: Vec<(&str, [i64; 7])> = Vec::new();
    let mut prior: Vec<(&str, [i64; 6])> = Vec::new();
    for (i, e) in entries.iter().enumerate() {
        let printed = e.credit.line(e.year);
        if e.year == 2026 {
            let gross = e.amount - e.transfer;
            let columns = [
                if e.passive.is_some() { gross } else { 0 },
                if e.passive.is_none() { gross } else { 0 },
                e.transfer,
                allowed[i],
                e.elective,
                used[i],
                elective[i],
            ];
            if let Some((_, values)) = current.iter_mut().find(|(line, _)| *line == printed) {
                for (v, a) in values.iter_mut().zip(columns) {
                    *v += a;
                }
            } else {
                current.push((printed, columns));
            }
        } else {
            let columns = [
                if e.passive.is_some() { e.amount } else { 0 },
                if e.passive.is_some() { allowed[i] } else { 0 },
                if e.passive.is_none() { e.amount } else { 0 },
                used[i],
                0,
                if e.year + 20 > 2026 { positive(allowed[i] - used[i] - carried[i]) } else { 0 },
            ];
            if let Some((_, values)) = prior.iter_mut().find(|(line, _)| *line == printed) {
                for (v, a) in values.iter_mut().zip(columns) {
                    *v += a;
                }
            } else {
                prior.push((printed, columns));
            }
        }
    }
    let mut current_totals = [[0; 7]; 3];
    for (printed, values) in &current {
        let bucket = if *printed == "3" {
            1
        } else if printed.starts_with('4') {
            2
        } else {
            0
        };
        for (v, a) in current_totals[bucket].iter_mut().zip(values) {
            *v += a;
        }
        for (column, amount) in ['d', 'e', 'f', 'g', 'h', 'i', 'j'].into_iter().zip(values) {
            credit_column(r, 3, printed, column, *amount);
        }
    }
    for (printed, values) in [
        ("2", current_totals[0]),
        ("5", current_totals[2]),
        ("6", core::array::from_fn(|i| current_totals.iter().map(|row| row[i]).sum())),
    ] {
        for (column, amount) in ['d', 'e', 'f', 'g', 'h', 'i', 'j'].into_iter().zip(values) {
            credit_column(r, 3, printed, column, amount);
        }
    }
    let mut prior_totals = [[0; 6]; 2];
    for (printed, values) in &prior {
        for (j, a) in values.iter().enumerate() {
            prior_totals[0][j] += if j == 0 || j == 1 { 0 } else { *a };
            prior_totals[1][j] += if j == 0 || j == 1 { *a } else { 0 };
        }
        // The form's lines 5/6 classify every column by passive/nonpassive source.
        for (column, amount) in ['d', 'e', 'f', 'g', 'h', 'i'].into_iter().zip(values) {
            credit_column(r, 4, printed, column, *amount);
        }
    }
    // Totals g/h/i also preserve the passive/nonpassive attribution of use and
    // closing balances rather than assigning them from the printed credit line.
    for row in &mut prior_totals {
        row[3..].fill(0);
    }
    for (i, e) in entries.iter().enumerate().filter(|(_, e)| e.year != 2026) {
        let bucket = usize::from(e.passive.is_some());
        prior_totals[bucket][3] += used[i];
        prior_totals[bucket][5] +=
            if e.year + 20 > 2026 { positive(allowed[i] - used[i] - carried[i]) } else { 0 };
    }
    for (printed, values) in [
        ("5", prior_totals[0]),
        ("6", prior_totals[1]),
        ("7", core::array::from_fn(|i| prior_totals.iter().map(|row| row[i]).sum())),
    ] {
        for (column, amount) in ['d', 'e', 'f', 'g', 'h', 'i'].into_iter().zip(values) {
            credit_column(r, 4, printed, column, amount);
        }
    }
}
fn credit_column(r: &mut FederalResult, part: u8, printed: &str, column: char, amount: i64) {
    let reference = alloc::format!("{printed}({column})");
    emit_lines(
        r,
        &mut (),
        [calculation_row(
            Some(FormLine::new(TaxForm::Form3800, &reference)),
            &alloc::format!("form_3800_part_{part}_line_{reference}"),
            LineValue::MoneyDollars(amount),
            &["credits.business_credits"],
            F3800,
            crate::Message::new(
                "EXPLAIN_FORM3800_COLUMN",
                alloc::vec![
                    crate::MessageArg::Number(i64::from(part)),
                    crate::MessageArg::Text(printed.into()),
                    crate::MessageArg::Text(alloc::format!("{column}"))
                ],
            ),
        )],
        |row, _, _| Some(row),
    );
}

pub(crate) fn entered_entries(
    form_context: crate::related_forms::RelatedFormContext<'_>,
    history: &BusinessCreditHistory,
    r: &mut FederalResult,
) -> Vec<CreditEntry> {
    let d = form_context.draft;
    let direct_participation = form_context.participation();
    let mut entries = Vec::new();
    for (index, item) in history.credits.iter().enumerate() {
        let source = resolve_source(d, &item.source);
        let reduced = match item.source {
            CreditSource::Business { reduced, .. }
            | CreditSource::Farm { reduced, .. }
            | CreditSource::Rental { reduced, .. } => reduced,
            CreditSource::Bought => true,
        };
        if !reduced {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new(
                    "WARN_ENTERED_BUSINESS_CREDIT_LEFT_OUT",
                    alloc::vec![crate::MessageArg::Resource(item.credit.form_message_key().into())],
                ),
                crate::IssueContext::Section(crate::TaxSection::MoreCredits)
                    .item("business_credits.credits", index),
            );
            continue;
        }
        if source.is_some_and(|entry| {
            matches!(entry.income, crate::FederalIncome::Business { .. })
                && direct_participation.get(entry.id)
                    == crate::participation::Participation::Unavailable
        }) {
            crate::require_localized!(
                r,
                false,
                &crate::IssueContext::Section(crate::TaxSection::MoreCredits)
                    .item("business_credits.credits", index),
                "Material participation",
                "DIRECT_PARTICIPATION_OTHER_ACTIVITY_SOURCE_REQUIRED",
                "Establish the qualifying hours and legal activity grouping of the other trades or businesses before applying the combined significant-participation test."
            );
            continue;
        }
        let passive = match source.map(|e| (&e.income, e.owner)) {
            Some((crate::FederalIncome::Business { .. }, _)) => {
                !(direct_participation.get(source.map_or(0, |entry| entry.id))
                    == crate::participation::Participation::Yes)
            }
            Some((crate::FederalIncome::Rental { data }, owner)) => {
                !data.participation.professional() || d.professional(owner.owned()).is_none()
            }
            Some(_) => true,
            None => matches!(item.source, CreditSource::Bought),
        };
        let worksheet = if let Some(crate::FederalIncomeEntry {
            income: crate::FederalIncome::Rental { data },
            ..
        }) = source
        {
            match item.credit {
                BusinessCredit::LowIncomeHousing | BusinessCredit::LowIncomeHousingBefore2008 => {
                    PassiveWorksheet::LowIncomeHousing
                }
                BusinessCredit::Rehabilitation => PassiveWorksheet::Rehabilitation,
                _ => {
                    if data.participation.active() {
                        PassiveWorksheet::RentalActive
                    } else {
                        PassiveWorksheet::Other
                    }
                }
            }
        } else {
            item.credit.worksheet()
        };
        let amount = dollars(item.amount);
        let transfer = match item.election {
            CreditElection::Sold { amount } => -dollars(amount),
            _ => {
                if matches!(item.source, CreditSource::Bought) {
                    amount
                } else {
                    0
                }
            }
        };
        let elective = match item.election {
            CreditElection::ElectivePayment { amount } => dollars(amount),
            _ => 0,
        };
        entries.push(CreditEntry {
            source_owner: source.map(|entry| entry.owner.owned()),
            conditional_origin: ConditionalCarrybackOrigin::from_source(
                item.credit,
                item.carbon_oxide_origin.as_ref(),
                item.renewable_facility_origin.as_ref(),
            ),
            credit: item.credit,
            year: 2026,
            amount: if matches!(item.source, CreditSource::Bought) {
                amount
            } else {
                amount + transfer
            },
            passive: passive.then_some(worksheet),
            transfer,
            elective,
            activity: source.map(|e| e.id),
            filer: CarryFiler::SameFilers,
        });
    }
    entries
}
pub(crate) fn cap_entries(entries: &mut [CreditEntry], magi_eligible: bool, r: &mut FederalResult) {
    for entry in entries.iter_mut().filter(|e| e.year == 2026 && e.credit.info().magi_limited) {
        if !magi_eligible {
            entry.amount = 0;
        }
    }
    for credit in [BusinessCredit::DisabledAccess, BusinessCredit::EmployerChildcare] {
        let indexes: Vec<usize> = (0..entries.len())
            .filter(|&i| entries[i].credit == credit && entries[i].year == 2026)
            .collect();
        let amounts: Vec<i64> = indexes.iter().map(|&i| entries[i].amount).collect();
        let total: i64 = amounts.iter().sum();
        let cap = dollars(credit.info().cap_cents.expect("capped credit"));
        if total > cap {
            if credit == BusinessCredit::EmployerChildcare {
                crate::federal_rules::note_once_scoped(
                    r,
                    msg!("WARN_EMPLOYER_CHILDCARE_LIMIT_ASSUMED"),
                    crate::IssueContext::Section(crate::TaxSection::MoreCredits)
                        .alternatives(&[crate::TaxSection::PassThrough]),
                );
            }
            for (i, amount) in
                indexes.into_iter().zip(crate::kernel::ratio::allocate_remaining(cap, &amounts))
            {
                entries[i].amount = amount;
            }
        }
    }
}

// The credit gross keeps its existing Share rounding. Source components follow
// that same owner target, in actual-identity order, with the counterpart taking
// the exact residual. Stored source facts are never rewritten. This monetary
// projection does not decide mixed-vintage within-class credit-use ordering.
impl crate::community::Divide for K1Credit {
    fn divided(&self, share: crate::community::Share) -> Self {
        let Self { code, line, amount, carbon_oxide_origin, renewable_facility_origin } = self;
        if carbon_oxide_origin.as_ref().is_some_and(|origin| !origin.valid(*amount))
            || renewable_facility_origin.as_ref().is_some_and(|origin| !origin.valid(*amount))
        {
            return self.clone();
        }
        let divided_amount = share.filer(*amount);
        let carbon_oxide_origin = carbon_oxide_origin.as_ref().and_then(|origin| {
            if divided_amount == 0 {
                return None;
            }
            let CarbonOxideCreditOrigin::Statement { equipment } = origin;
            let result = divided_origin_parts(equipment, *amount, share);
            Some(CarbonOxideCreditOrigin::Statement { equipment: result })
        });
        let renewable_facility_origin = renewable_facility_origin.as_ref().and_then(|origin| {
            if divided_amount == 0 {
                return None;
            }
            let RenewableFacilityCreditOrigin::Statement { facilities } = origin;
            Some(RenewableFacilityCreditOrigin::Statement {
                facilities: divided_origin_parts(facilities, *amount, share),
            })
        });
        Self {
            code: *code,
            line: *line,
            amount: divided_amount,
            carbon_oxide_origin,
            renewable_facility_origin,
        }
    }
}
