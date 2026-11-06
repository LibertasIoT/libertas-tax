// Coverage: Form 4137 — Supported (audited 2026-09-24).
//   Lines: Lines 1–13 per employer: actual monthly or allocated tips, under-$20-month exclusion,
//   noncash tips (income only), SS wage base, Medicare-only government coverage, Schedule 2 and
//   Additional Medicare flow; W-2 box 12 uncollected tax as facts. Railroad tips remain
//   taxable wage income with an employer-reporting notice, without Form 4137 tax.
//   Box 8 or actual cash tips qualify for Schedule 1-A without code TP when the person
//   confirms an eligible occupation; noncash tips never enter that deduction.
//   Evidence: M/employee_tips.rs, federal_rules.rs; T/business.rs
//   employee_tip_deduction_takes_the_larger_reported_amount_per_employer; T/retirement.rs
//   allocated_employee_tips_complete_return_keeps_w2_and_4137_separate; T/mod.rs work_deductions;
//   IRS f4137, Pub 531.

// Form 4137 and Publication 531. Monetary source values are cents.
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction};
use alloc::vec::Vec;
use libertas_macros::*;

/// Employee tips not included in W-2 box 1
/// Enter once with this employer. These amounts are additional wage income; the App calculates Form 4137 payroll taxes and carries covered tips into Additional Medicare tax.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployeeTips {
    /// Cash and charged tip records
    pub cash: CashTipRecords,
    /// Noncash tips not included in box 1
    /// Fair market value of tickets, goods and other noncash tips. These are taxable wages but are not subject to Social Security or Medicare tip tax.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub noncash: i64,
    /// Payroll coverage for this employer
    pub coverage: TipCoverage,
}

/// Cash and charged employee tips not reported on a W-2
/// Use your daily tip records, one record per employer. Do not enter the same unreported tips in the W-2 editor. Tips already included in wages belong only in the reported portion below; self-employed tips belong with your business.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CashTipsWithoutW2 {
    /// Employer name
    #[libertas_size(min = 1)]
    pub employer: alloc::string::String,
    /// Monthly cash and charged tips
    /// Include reported tips in each month's total so the $20 monthly payroll-tax rule uses all tips from this employer. Only the unreported portion adds income. Omit months with no tips; enter each month once.
    // Tax structure: one record per calendar month.
    #[libertas_size(max = 12)]
    /// ----
    /// Month
    pub months: Vec<CashTipMonth>,
    /// Payroll coverage for this employer
    pub coverage: TipCoverage,
    /// These tips qualify for the tips deduction
    /// Confirm a voluntarily tipped occupation on the IRS list, not a specified service trade or business. Check your occupation and tip records. No keeps the tips taxable without a deduction. The App applies the return's other deduction limits.
    pub qualified_occupation: bool,
}
impl CashTipsWithoutW2 {
    pub(crate) fn tips(&self) -> EmployeeTips {
        EmployeeTips {
            cash: CashTipRecords::Actual { months: self.months.clone() },
            noncash: 0,
            coverage: self.coverage.clone(),
        }
    }
}

/// Source of cash tip amounts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CashTipRecords {
    /// Actual monthly records
    Actual {
        /// Months with cash or charged tips
        /// Actual records establish income even when different from W-2 box 8. Omit months with no tips; do not repeat a month for the same employer.
        // Tax structure: actual cash-tip records have one unique entry per calendar month.
        #[libertas_size(max = 12)]
        /// ----
        /// Month
        months: Vec<CashTipMonth>,
    },
    /// W-2 allocated tips without adequate actual records
    Allocated {
        /// Box 8 allocated tips
        /// Enter the employer's amount when adequate records do not establish actual unreported tips. Do not add it again to box 1 or to actual monthly records.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
    },
}

/// Monthly cash tips for this employer
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct CashTipMonth {
    /// Calendar month
    // Tax structure: calendar months are numbered 1 through 12.
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Total cash, charged and shared tips received
    /// Include the reported portion below. Exclude tips paid over to other employees and noncash tips.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub received: i64,
    /// Portion reported to this employer and included in box 1
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub reported: i64,
}

/// Employee tip payroll coverage
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum TipCoverage {
    /// Social Security and Medicare
    SocialAndMedicare,
    /// Government employment subject only to Medicare
    MedicareOnlyGovernment,
    /// Railroad retirement; report tips to the employer instead of using Form 4137
    Railroad,
}

#[derive(Default)]
pub(crate) struct TipAmounts {
    pub income: i64,
    pub social: i64,
    pub medicare: i64,
}

impl EmployeeTips {
    pub(crate) fn valid(&self) -> bool {
        let money = |v| (0..=100000000000).contains(&v);
        money(self.noncash)
            && match &self.cash {
                CashTipRecords::Allocated { amount } => money(*amount),
                CashTipRecords::Actual { months } => months.iter().enumerate().all(|(i, m)| {
                    (1..=12).contains(&m.month)
                        && money(m.received)
                        && (0..=m.received).contains(&m.reported)
                        && !months[..i].iter().any(|prior| prior.month == m.month)
                }),
            }
    }
    pub(crate) fn calculate(&self) -> Result<TipAmounts, &'static str> {
        if !self.valid() {
            return Err(crate::fixture_english!(
                "Check the 2026 tip records: enter each month once, with reported tips no more than the tips received."
            ));
        }
        let (cash, covered) = match &self.cash {
            CashTipRecords::Allocated { amount } => (*amount, *amount),
            CashTipRecords::Actual { months } => {
                months.iter().fold((0, 0), |(cash, covered), m| {
                    let unreported = m.received - m.reported;
                    (cash + unreported, covered + if m.received >= 2000 { unreported } else { 0 })
                })
            }
        };
        Ok(TipAmounts {
            income: cash + self.noncash,
            social: if self.coverage == TipCoverage::SocialAndMedicare { covered } else { 0 },
            medicare: if self.coverage == TipCoverage::Railroad { 0 } else { covered },
        })
    }
}

// Each spouse uses separate employee-payroll worksheets. Apply this after W-2
// wages for Form 4137, then after covered tips for Form 8919, before Schedule SE.
pub(crate) fn payroll(wages: i64, tips: &TipAmounts) -> [i64; 2] {
    let social_base = phase::cap(
        dollars(tips.social),
        [phase::floor(
            crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_DOLLARS - dollars(wages),
            0,
        )],
    );
    [fraction(social_base, 62, 1000), fraction(dollars(tips.medicare), 145, 10000)]
}

impl crate::FederalWage {
    pub(crate) fn total_income(&self) -> Result<i64, &'static str> {
        self.employee_tips
            .as_ref()
            .map_or(Ok(self.wages), |tips| tips.calculate().map(|value| self.wages + value.income))
    }
}

/// Uncollected payroll tax from W-2 box 12
/// Copy the tax amounts, not the underlying tips or insurance value. Income and covered wages are already in the W-2 wage boxes; these amounts do not add income or actual withholding. Railroad taxes need separate treatment.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct W2UncollectedTax {
    /// Code A — Social Security tax on reported tips
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tip_social: i64,
    /// Code B — Medicare tax on reported tips
    /// Ordinary Medicare only; do not include Additional Medicare tax.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub tip_medicare: i64,
    /// Code M — Social Security tax on former-employee group-term life insurance
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub life_social: i64,
    /// Code N — Medicare tax on former-employee group-term life insurance
    /// Ordinary Medicare only; do not include Additional Medicare tax.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub life_medicare: i64,
}
impl W2UncollectedTax {
    pub(crate) fn valid(&self) -> bool {
        [self.tip_social, self.tip_medicare, self.life_social, self.life_medicare]
            .iter()
            .all(|v| (0..=100000000000).contains(v))
    }
    pub(crate) fn social(&self) -> i64 {
        self.tip_social + self.life_social
    }
    pub(crate) fn medicare(&self) -> i64 {
        self.tip_medicare + self.life_medicare
    }
}
