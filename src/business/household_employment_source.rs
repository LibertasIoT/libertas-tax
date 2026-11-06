// Household payroll records and their intrinsic validation.
use crate::common::Owner;
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Household employer
/// One entry per filer/employer, with all employees paid by that employer. These are wages you paid, not your income. Do not repeat employment taxes reported on Forms 941, 943, 944 or 940, including taxes already reported and paid on your behalf by an authorized government agency or third-party agent. Merely hiring a payroll service does not remove your responsibility.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdEmployer {
    /// Employer on this return
    #[libertas_default(Taxpayer)]
    pub owner: Owner,
    /// Employees
    /// Keep each employee in one record, even when their treatment or state changes.
    pub workers: Vec<HouseholdWorker>,
    /// 2025 cash wages by calendar quarter
    /// Enter one payroll total for each quarter. Exclude pay to your spouse, parent and child while under 21. These records establish the prior-year FUTA test; they are not taxed again in 2026.
    // Tax structure: the prior-year FUTA test has exactly four calendar quarters.
    #[libertas_size(min = 4, max = 4)]
    pub prior_quarters: Vec<HouseholdQuarter>,
    /// State unemployment records
    /// Include each state once. Rates and state taxable wages come from agency notices and payroll reports, not a calculated federal credit.
    // Fixed code catalog: state() admits 53 unemployment jurisdictions; native validation requires each jurisdiction once.
    #[libertas_size(max = 53)]
    pub states: Vec<HouseholdState>,
    /// Complete Schedule H payroll records
    /// Confirm household employee status, gross cash pay and all employees. Exclude independent contractors, wages reported on separate business payroll returns and unresolved corrections. Enter supported state disability-plan notices with the affected employee. Other third-party payroll liability transfers and treaty or territory exceptions need separate treatment. This App calculates the return; it does not run payroll or file employee W-2s. Your payroll records and the W-2s you issued show it. No stops the estimate: Schedule H for those situations is not calculated.
    pub complete: bool,
}

/// Prior-year household payroll quarter
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdQuarter {
    /// Calendar quarter in 2025
    /// 1 is January–March, 2 April–June, 3 July–September, 4 October–December.
    // Tax structure: each calendar year has four quarters.
    #[libertas_number(min = 1, max = 4)]
    pub quarter: u32,
    /// Cash wages excluding spouse, parent and child-under-21 pay
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cash: i64,
}

/// Household employee
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdWorker {
    /// Employee label
    /// Use a synthetic label in this prototype; keep identifying payroll records separately.
    #[libertas_size(min = 1)]
    pub label: String,
    /// Cash wage payments
    /// Split records when state coverage or family/age treatment changes. Enter payments in date order when different states share a date.
    pub pay: Vec<HouseholdPay>,
    /// State disability-plan notice
    /// Leave empty only when no state-plan payments affect this employee's Schedule H. Do not also enter those payments in cash wages above.
    /// [NullText]
    /// No state disability-plan payments affect this worker
    /// [SetText]
    /// The state disability-plan notice is
    pub disability: Option<HouseholdDisability>,
    /// Employee federal income tax actually withheld
    /// This is a Schedule H liability, not your own withholding credit. Exclude income tax withheld and remitted by the state on disability payments. Include actual employee income tax paid from your funds; that payment must also be in gross cash wages. Do not add employee Social Security/Medicare taxes paid from your funds to cash wages.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub withholding: i64,
}

/// State disability-plan payroll notice
/// Use a complete state liability notice together with either reported taxable amounts or source plan records. This path applies when employer payroll liability remains with you under Schedule H.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdDisability {
    /// Sick-pay amounts and source
    /// Choose the state's reported payroll-taxable amounts or calculate them from gross payments and plan records. Do not enter the same payment in both sources or in regular wages.
    pub wages: HouseholdSickWages,
    /// Employee Social Security tax withheld and deposited by the state
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub social_withheld: i64,
    /// Employee Medicare tax withheld and deposited by the state
    /// Include Additional Medicare tax in this figure if the notice includes it. Exclude federal income tax withheld by the state; it is not an employer liability or a personal payment credit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub medicare_withheld: i64,
    /// Complete reconciled state-plan notice
    /// Confirm timely notice transferring employer Social Security/Medicare and FUTA liability to you, and the state deposits shown here. For reported amounts, confirm ordinary Social Security/Medicare/FUTA treatment before annual caps. For plan records, confirm all funding and work/benefit facts; the App calculates exclusions. Missing liability notices, unresolved funding, conflicting wage limits or private-plan transfers still need further treatment. The state program's notice shows it. No stops the estimate: who owes these payroll taxes is not determined.
    pub complete: bool,
}

/// Household sick-pay source
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HouseholdSickWages {
    /// State notice reports the payroll-taxable portion
    /// Amounts already exclude employee-funded and other exempt benefits and have the same ordinary FICA/FUTA treatment before annual caps.
    Reported {
        /// Notice payments
        pay: Vec<HouseholdPay>,
    },
    /// Calculate from gross payments and plan records
    /// Employer payroll liability and actual state deposits still require a complete state notice. Gross benefits alone do not establish who owes the tax.
    Plan {
        /// Plan funding
        funding: HouseholdSickFunding,
        /// Gross benefit payments and work history
        pay: Vec<HouseholdSickPayment>,
    },
}

/// Sick-pay plan funding
/// Use the applicable employee class when its contributions can be determined separately. Employer amounts include only contributions excluded from employee income; after-tax employee funding includes state disability taxes paid by the employee.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HouseholdSickFunding {
    /// Uninsured fund with at least one full prior year
    /// Enter contributions for 2023–2025 if the plan existed for at least three years at the start of 2026, otherwise the one or two preceding years for which it existed. Use the applicable employee class when its contributions can be determined separately.
    Fund {
        /// Prior calendar-year contributions
        // Tax structure: the sick-pay contribution ratio uses at most three preceding policy or fund years.
        #[libertas_size(min = 1, max = 3)]
        years: Vec<HouseholdSickContribution>,
    },
    /// Uninsured fund with less than one full year at the start of 2026
    /// Use actual 2026 contributions through the determination date, or a periodic determination for the succeeding period. Preserve the same determination for all reporting; do not substitute insurance premium estimates.
    NewFund {
        /// Contribution determinations and periods to which they apply
        #[libertas_size(min = 1)]
        periods: Vec<HouseholdSickFundPeriod>,
    },
    /// Individual insurance policy
    /// The employer share uses the current policy year's premiums, not a group-policy historical average. Enter each policy period containing a payment; do not overlap periods.
    Individual {
        /// Policy periods and premium records
        #[libertas_size(min = 1)]
        periods: Vec<HouseholdSickPolicy>,
    },
    /// Fully employer-funded
    /// Include employee pretax cafeteria-plan contributions as employer funding.
    Employer,
    /// Group-policy premium history
    /// Enter the last three policy years whose net premiums were known at the beginning of 2026, or all such years if fewer than three were known. Include employee pretax cafeteria-plan funding in employer cost. The App computes the employer share from actual costs; do not enter a calculated taxable benefit.
    Group {
        /// Completed policy years
        // Tax structure: the sick-pay contribution ratio uses at most three preceding policy or fund years.
        #[libertas_size(min = 1, max = 3)]
        years: Vec<HouseholdSickPremium>,
    },
    /// Group policy with no policy-year net premiums known at the start of 2026
    /// Supply a reasonable first-year estimate, or actual policy-year net premiums ascertained during 2026. Keep the determination consistent for all reporting purposes.
    NewGroup {
        /// Employer cost, including employee pretax contributions
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        employer: i64,
        /// Employee after-tax cost
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        employee: i64,
    },
}

/// Uninsured sick-pay fund contributions
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdSickContribution {
    /// Calendar year
    // Tax structure: this retained source belongs to a year before the 2026 return year.
    #[libertas_number(min = 2023, max = 2025)]
    pub year: u32,
    /// Employer contributions, including employee pretax amounts
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employer: i64,
    /// Employee after-tax contributions
    /// Include employee taxes funding a state temporary disability program.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employee: i64,
}

/// First-year uninsured fund determination
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdSickFundPeriod {
    /// Contribution-record period begins in 2026
    /// Use January 1 for a year-to-date determination, or the first date of the month or quarter used for a periodic determination.
    #[libertas_date_only]
    pub from: u32,
    /// Contribution-record period ends
    /// Actual contributions through this date determine the share. It cannot follow the first payment to which the determination applies.
    #[libertas_date_only]
    pub through: u32,
    /// First payment date covered by this determination
    #[libertas_date_only]
    pub starts: u32,
    /// Last payment date covered by this determination
    /// Periods must not overlap. A monthly or quarterly determination applies to the succeeding period.
    #[libertas_date_only]
    pub ends: u32,
    /// Employer contributions during the record period
    /// Include pretax employee amounts.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employer: i64,
    /// Employee after-tax contributions during the record period
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employee: i64,
}

/// Individual sick-pay policy period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdSickPolicy {
    /// First date of policy year
    #[libertas_date_only]
    pub starts: u32,
    /// Last date of policy year
    #[libertas_date_only]
    pub ends: u32,
    /// Employer premiums for this policy year
    /// Include pretax employee contributions; exclude contributions included in the employee's taxable income.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employer: i64,
    /// Employee after-tax premiums for this policy year
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employee: i64,
}

/// Completed sick-pay policy year
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdSickPremium {
    /// Policy year ending date before 2026
    #[libertas_date_only]
    pub ended: u32,
    /// Employer cost, including employee pretax contributions
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employer: i64,
    /// Employee after-tax cost
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub employee: i64,
}

/// Gross sick-pay payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdSickPayment {
    /// Gross payment before withholding or funding exclusions
    /// Enter the full benefit as cash. The App computes its payroll-taxable portion and preserves its actual date and state coverage.
    pub payment: HouseholdPay,
    /// Last day actually worked before this payment
    /// Even one later day of work restarts the six-calendar-month period. Supply the correct last work date for each payment.
    #[libertas_date_only]
    pub last_worked: u32,
    /// Social Security disability-insurance entitlement
    /// Leave empty if not entitled under Social Security Act section 223(a). This is separate from merely receiving sick pay or being disabled.
    /// [NullText]
    /// Not entitled to Social Security disability insurance
    /// [SetText]
    /// Social Security disability-insurance entitlement is
    pub disability: Option<HouseholdSickEntitlement>,
    /// Death or disability retirement ending employment
    /// Leave empty if neither applies. Regular accrued vacation or other amounts payable even without termination are not automatically exempt.
    /// [NullText]
    /// Neither death nor disability retirement ended employment
    /// [SetText]
    /// Employment ended through death or disability retirement as follows
    pub ended: Option<HouseholdSickEnd>,
}

/// Social Security disability-insurance entitlement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdSickEntitlement {
    /// Entitlement date
    #[libertas_date_only]
    pub date: u32,
    /// No services performed for the employer during the period covered by this payment
    pub no_services: bool,
}

/// Employment ended by death or disability retirement
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HouseholdSickEnd {
    /// Employee died
    Death {
        /// Date of death
        #[libertas_date_only]
        date: u32,
        /// This payment is under a definite plan and would not be payable without the death
        /// False preserves normal payroll treatment during the death year; payments after that calendar year are excluded.
        definite_plan: bool,
    },
    /// Employment ended through disability retirement
    Retirement {
        /// Termination date
        #[libertas_date_only]
        date: u32,
        /// This payment is under a definite plan and would not be payable without disability retirement
        definite_plan: bool,
    },
}

/// Household wage payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdPay {
    /// Payment date in 2026
    #[libertas_date_only]
    pub date: u32,
    /// Gross cash wages
    /// Include cash, checks and taxable cash benefits before withholding, including employee income tax you paid. Exclude noncash benefits, qualified excluded reimbursements, and the employee Social Security/Medicare share you paid from your funds. Use payroll records, not take-home pay. Within a state disability notice, use its payroll-taxable portion instead of the total benefit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub cash: i64,
    /// Relationship and age for these wages
    pub treatment: HouseholdWageKind,
    /// State unemployment jurisdiction
    /// Two-letter postal code. Use the state's payroll assignment, not the employee's mailing address.
    // Legal identifier: use the two-letter postal abbreviation for the unemployment jurisdiction.
    #[libertas_size(min = 2, max = 2)]
    pub state: String,
    /// These wages are covered by the state's unemployment law
    /// Include wages above the state's annual taxable wage cap when the employment itself is covered. Choose false for wages excluded by state law, not merely because the cap was reached.
    pub state_covered: bool,
}

/// Household wage relationship
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HouseholdWageKind {
    /// Ordinary employee age 18 or older throughout 2026
    /// Also use for your child after reaching age 21; split wages paid before that birthday.
    Ordinary,
    /// Your spouse
    Spouse,
    /// Your child while under age 21
    ChildUnder21,
    /// Your parent
    Parent {
        /// Cared for your child living with you who was under 18 or needed adult personal care for four continuous weeks in the service quarter
        child_care: bool,
        /// You were divorced and not remarried, widowed, or living with a spouse unable to care for the child for four continuous weeks in the service quarter
        marital_condition: bool,
    },
    /// Other employee under age 18 at any time in 2026
    Minor {
        /// Student during the year
        student: bool,
        /// Household service was the employee's principal occupation
        principal_occupation: bool,
    },
}

/// State unemployment credit records
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdState {
    /// State postal code
    // Legal identifier: use the two-letter postal abbreviation for the unemployment jurisdiction.
    #[libertas_size(min = 2, max = 2)]
    pub state: String,
    /// Agency experience-rate periods
    /// One row per rate period. The App computes the additional credit, including a granted zero rate. An absent rate means the state did not assign an experience rate.
    pub periods: Vec<HouseholdStatePeriod>,
    /// Total 2026 employer unemployment contributions required by this state
    /// Copy the state account assessment, excluding penalties, interest and voluntary payments. This establishes whether all required contributions were paid on time; it is not the federal credit.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub required: i64,
    /// State unemployment contributions actually paid
    /// Include only employer contributions for 2026, including state-certified excess credits. Exclude employee deductions, penalties, interest, administrative taxes and voluntary rate-reduction payments. Do not include planned payments.
    pub payments: Vec<HouseholdStatePayment>,
    /// Final 2026 FUTA credit-reduction rate
    /// Copy the official 2026 federal rate; zero means no reduction. Leave empty while unknown; the estimate assumes no FUTA state credit until the official final rate is entered. A potential rate or a prior-year rate is not a final 2026 rate.
    /// [NullText]
    /// The final 2026 FUTA credit-reduction rate is unknown
    /// [SetText]
    /// The final 2026 FUTA credit-reduction rate is
    // Tax structure: the FUTA credit reduction cannot exceed the 5.4% maximum credit.
    #[libertas_number(min = 0, max = 5.4, step = 0.0001)]
    #[libertas_physical_unit("percent")]
    pub reduction: Option<f64>,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for HouseholdState {}

/// State experience-rate period
#[derive(Clone, Debug, PartialEq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdStatePeriod {
    /// First date of rate period in 2026
    #[libertas_date_only]
    pub starts: u32,
    /// Last date of rate period in 2026
    #[libertas_date_only]
    pub ends: u32,
    /// State taxable wages from payroll report
    /// Use wages subject to the state act, including wages that would be taxable without a granted zero experience rate. Do not substitute the federal $7,000 wage base.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub wages: i64,
    /// Assigned state experience rate
    /// Use zero only for a granted zero rate; leave empty when no rate was assigned. Split changed rates into separate dated periods.
    /// [NullText]
    /// No state experience rate was assigned
    /// [SetText]
    /// The assigned state experience rate is
    // Tax structure: this unemployment percentage cannot exceed 100%.
    #[libertas_number(min = 0, max = 100, step = 0.0001)]
    #[libertas_physical_unit("percent")]
    pub rate: Option<f64>,
}
// Natural-unit quantities are validated finite values, so equality is reflexive.
impl Eq for HouseholdStatePeriod {}

/// State unemployment payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdStatePayment {
    /// Payment date
    #[libertas_date_only]
    pub date: u32,
    /// Employer contribution for 2026
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub amount: i64,
}

fn money(v: i64) -> bool {
    (0..=100000000000).contains(&v)
}
fn state(v: &str) -> bool {
    "AL AK AZ AR CA CO CT DE DC FL GA HI ID IL IN IA KS KY LA ME MD MA MI MN MS MO MT NE NV NH NJ NM NY NC ND OH OK OR PA RI SC SD TN TX UT VT VA WA WV WI WY PR VI".split_whitespace().any(|s| s == v)
}
impl HouseholdWageKind {
    pub(crate) fn fica(&self) -> bool {
        match self {
            Self::Ordinary => true,
            Self::Parent { child_care, marital_condition } => *child_care && *marital_condition,
            Self::Minor { student, principal_occupation } => !*student && *principal_occupation,
            Self::Spouse | Self::ChildUnder21 => false,
        }
    }
    pub(crate) fn futa(&self) -> bool {
        matches!(self, Self::Ordinary | Self::Minor { .. })
    }
}
fn valid_pay(pay: &[HouseholdPay]) -> bool {
    pay.len() <= 400
        && pay.iter().all(|p| p.date / 10000 == 2026 && money(p.cash) && state(&p.state))
}
impl HouseholdSickFunding {
    pub(crate) fn ratio(&self, date: u32) -> (i64, i64) {
        match self {
            Self::Employer => (1, 1),
            Self::Fund { years } => {
                let employer = years.iter().map(|y| y.employer).sum::<i64>();
                (employer, employer + years.iter().map(|y| y.employee).sum::<i64>())
            }
            Self::NewFund { periods } => {
                let p = periods
                    .iter()
                    .find(|p| p.starts <= date && date <= p.ends)
                    .expect("validated fund determination period");
                (p.employer, p.employer + p.employee)
            }
            Self::Individual { periods } => {
                let p = periods
                    .iter()
                    .find(|p| p.starts <= date && date <= p.ends)
                    .expect("validated payment policy period");
                (p.employer, p.employer + p.employee)
            }
            Self::Group { years } => {
                let employer: i64 = years.iter().map(|y| y.employer).sum();
                (employer, employer + years.iter().map(|y| y.employee).sum::<i64>())
            }
            Self::NewGroup { employer, employee } => (*employer, employer + employee),
        }
    }
    fn valid(&self) -> bool {
        match self {
            Self::Employer => true,
            Self::Fund { years } => {
                !years.is_empty()
                    && years.len() <= 3
                    && years.iter().enumerate().all(|(i, y)| {
                        y.year <= 2025
                            && y.year >= 2026 - years.len() as u32
                            && money(y.employer)
                            && money(y.employee)
                            && !years[..i].iter().any(|p| p.year == y.year)
                    })
                    && years.iter().map(|y| y.employer + y.employee).sum::<i64>() > 0
            }
            Self::NewFund { periods } => {
                !periods.is_empty()
                    && periods.len() <= 400
                    && periods.iter().enumerate().all(|(i, p)| {
                        p.from >= 20260101
                            && p.from <= p.through
                            && p.through <= p.starts
                            && p.starts <= p.ends
                            && p.ends <= 20261231
                            && money(p.employer)
                            && money(p.employee)
                            && p.employer + p.employee > 0
                            && !periods[..i]
                                .iter()
                                .any(|q| p.starts <= q.ends && q.starts <= p.ends)
                    })
            }
            Self::Individual { periods } => {
                !periods.is_empty()
                    && periods.len() <= 20
                    && periods.iter().enumerate().all(|(i, p)| {
                        p.starts <= p.ends
                            && p.starts <= 20261231
                            && p.ends >= 20260101
                            && money(p.employer)
                            && money(p.employee)
                            && p.employer + p.employee > 0
                            && !periods[..i]
                                .iter()
                                .any(|q| p.starts <= q.ends && q.starts <= p.ends)
                    })
            }
            Self::Group { years } => {
                !years.is_empty()
                    && years.len() <= 3
                    && years.iter().enumerate().all(|(i, y)| {
                        y.ended < 20260101
                            && money(y.employer)
                            && money(y.employee)
                            && y.employer + y.employee > 0
                            && !years[..i].iter().any(|v| v.ended == y.ended)
                    })
            }
            Self::NewGroup { employer, employee } => {
                money(*employer) && money(*employee) && employer + employee > 0
            }
        }
    }
}
impl HouseholdSickWages {
    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Reported { pay } => pay.len(),
            Self::Plan { pay, .. } => pay.len(),
        }
    }
    fn valid(&self) -> bool {
        match self {
            Self::Reported { pay } => valid_pay(pay),
            Self::Plan { funding, pay } => {
                funding.valid()
                    && pay.len() <= 400
                    && pay.iter().all(|p| {
                        valid_pay(core::slice::from_ref(&p.payment))
                            && match funding {
                                HouseholdSickFunding::Individual { periods } => {
                                    periods.iter().any(|y| {
                                        y.starts <= p.payment.date && p.payment.date <= y.ends
                                    })
                                }
                                HouseholdSickFunding::NewFund { periods } => {
                                    periods.iter().any(|y| {
                                        y.starts <= p.payment.date && p.payment.date <= y.ends
                                    })
                                }
                                _ => true,
                            }
                            && p.last_worked <= p.payment.date
                            && p.ended.as_ref().is_none_or(|end| match end {
                                HouseholdSickEnd::Death { date, .. }
                                | HouseholdSickEnd::Retirement { date, .. } => {
                                    p.last_worked <= *date
                                }
                            })
                            && p.disability.as_ref().is_none_or(|d| d.date <= p.payment.date)
                    })
            }
        }
    }
}
impl HouseholdDisability {
    fn valid(&self) -> bool {
        self.wages.valid() && money(self.social_withheld) && money(self.medicare_withheld)
    }
}
impl HouseholdEmployer {
    pub(crate) fn valid(&self) -> bool {
        self.workers.len() <= 100
            && self.prior_quarters.len() == 4
            && self.prior_quarters.iter().enumerate().all(|(i, v)| {
                money(v.cash)
                    && (1..=4).contains(&v.quarter)
                    && !self.prior_quarters[..i].iter().any(|q| q.quarter == v.quarter)
            })
            && self.states.len() <= 53
            && self.workers.iter().all(|w| {
                !w.label.trim().is_empty()
                    && money(w.withholding)
                    && valid_pay(&w.pay)
                    && w.disability.as_ref().is_none_or(HouseholdDisability::valid)
            })
            && self.states.iter().enumerate().all(|(i, s)| {
                state(&s.state)
                    && money(s.required)
                    && !self.states[..i].iter().any(|other| other.state == s.state)
                    && s.reduction.is_none_or(|v| {
                        crate::kernel::natural::exact(v, 10_000)
                            .is_some_and(|v| (0..=54000).contains(&v))
                    })
                    && s.payments.len() <= 100
                    && s.payments.iter().all(|p| money(p.amount))
                    && s.periods.len() <= 20
                    && s.periods.iter().enumerate().all(|(j, p)| {
                        p.starts / 10000 == 2026
                            && p.ends / 10000 == 2026
                            && p.starts <= p.ends
                            && money(p.wages)
                            && p.rate.is_none_or(|v| {
                                crate::kernel::natural::exact(v, 10_000)
                                    .is_some_and(|v| (0..=1_000_000).contains(&v))
                            })
                            && s.periods[..j].iter().all(|q| p.ends < q.starts || p.starts > q.ends)
                    })
            })
    }
}
pub(crate) fn valid(records: &[HouseholdEmployer]) -> bool {
    records
        .iter()
        .enumerate()
        .all(|(i, e)| e.valid() && !records[..i].iter().any(|p| p.owner == e.owner))
}
