// Coverage: Schedule H — Supported (audited 2026-09-24).
//   Lines: Parts I–IV: SS/Medicare/Additional Medicare, federal withholding, FUTA with multi-state
//   credits and final credit-reduction rate, prior-year quarter test, sick pay and state
//   disability-plan notices, Schedule 2 flow. Refusal: state-deposited tax exceeding combined
//   liability must be reconciled. An unknown final FUTA credit-reduction rate
//   claims no state credit, with a warning, until the official rate is entered.
//   Evidence: M/household_employment.rs; T/family.rs
//   household_futa_allocates_first_seven_thousand_across_states_by_payment_date; T/retirement.rs
//   household_futa_quarter_test_aggregates_workers_not_annual_pay; T/mod.rs household_employer;
//   T/health_accounts.rs
//   household_disability_combines_regular_wages_before_social_and_medicare_caps; IRS Pub 926.
//   Page (u5-credits, 2026-09-28): the Other taxes list's "Household employees" item
//   (OtherTax::HouseholdEmployees, screening.household_employment, rows by owner through
//   allowed_owners; RareSituation::HouseholdEmployer removed). Re-verified against the 2026
//   draft Schedule H (line A $3,000, lines 2/4/6 rates, line C and line 9 $1,000 quarter test,
//   Section A 0.6%, total to 2026 Schedule 2 line 17a) and the $184,500 2026 wage base: no
//   calculation change. Interview fixtures and boundaries: T/other_taxes.rs
//   schedule_h_through_the_other_taxes_page, schedule_h_line_a_threshold_is_three_thousand_dollars,
//   schedule_h_futa_quarter_test_is_one_thousand_dollars, schedule_h_owner_rules_are_checked_on_save.

// Schedule H calculation from payroll records, not employee income on this return.
use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::{FederalDraft, FederalResult, FederalStatus, Owner};
use alloc::{string::String, vec::Vec};
use libertas_macros::*;

/// Household employer
/// One entry per filer/employer, with all employees paid by that employer. These are wages you paid, not your income. Do not repeat employment taxes reported on Forms 941, 943, 944 or 940, including taxes already reported and paid on your behalf by an authorized government agency or third-party agent. Merely hiring a payroll service does not remove your responsibility.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdEmployer {
    /// Employer on this return
    #[libertas_default(Taxpayer)]
    #[libertas_constrained_by("$.allowed_owners")]
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
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    pub social_withheld: i64,
    /// Employee Medicare tax withheld and deposited by the state
    /// Include Additional Medicare tax in this figure if the notice includes it. Exclude federal income tax withheld by the state; it is not an employer liability or a personal payment credit.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
        #[libertas_number(min = 0)]
        employer: i64,
        /// Employee after-tax cost
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    pub employer: i64,
    /// Employee after-tax contributions
    /// Include employee taxes funding a state temporary disability program.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    pub employer: i64,
    /// Employee after-tax contributions during the record period
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    pub employer: i64,
    /// Employee after-tax premiums for this policy year
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
    pub employer: i64,
    /// Employee after-tax cost
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    #[libertas_number(min = 0)]
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
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
    #[libertas_number(min = 0)]
    pub required: i64,
    /// State unemployment contributions actually paid
    /// Include only employer contributions for 2026, including state-certified excess credits. Exclude employee deductions, penalties, interest, administrative taxes and voluntary rate-reduction payments. Do not include planned payments.
    pub payments: Vec<HouseholdStatePayment>,
    /// Final 2026 FUTA credit-reduction rate, millionths
    /// Copy the official 2026 federal rate: 3,000 means 0.3%; zero means no reduction. Leave empty while unknown; the estimate assumes no FUTA state credit until the official final rate is entered. A potential rate or a prior-year rate is not a final 2026 rate.
    /// [NullText]
    /// The final 2026 FUTA credit-reduction rate is unknown
    /// [SetText]
    /// The final 2026 FUTA credit-reduction rate in millionths is
    // Tax structure: the FUTA credit reduction cannot exceed the 5.4% maximum credit (54,000 millionths).
    #[libertas_number(min = 0, max = 54000)]
    pub reduction: Option<u32>,
}

/// State experience-rate period
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
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
    #[libertas_number(min = 0)]
    pub wages: i64,
    /// Assigned state experience rate, millionths
    /// 27,000 means 2.7%. Use zero only for a granted zero rate; leave empty when no rate was assigned. Split changed rates into separate dated periods.
    /// [NullText]
    /// No state experience rate was assigned
    /// [SetText]
    /// The assigned state experience rate in millionths is
    // Tax structure: this unemployment percentage is expressed in millionths and cannot exceed 100%.
    #[libertas_number(min = 0, max = 1000000)]
    pub rate: Option<u32>,
}

/// State unemployment payment
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdStatePayment {
    /// Payment date
    #[libertas_date_only]
    pub date: u32,
    /// Employer contribution for 2026
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub amount: i64,
}

fn money(v: i64) -> bool {
    (0..=100000000000).contains(&v)
}
fn state(v: &str) -> bool {
    "AL AK AZ AR CA CO CT DE DC FL GA HI ID IL IN IA KS KY LA ME MD MA MI MN MS MO MT NE NV NH NJ NM NY NC ND OH OK OR PA RI SC SD TN TX UT VT VA WA WV WI WY PR VI".split_whitespace().any(|s| s == v)
}
impl HouseholdWageKind {
    fn fica(&self) -> bool {
        match self {
            Self::Ordinary => true,
            Self::Parent { child_care, marital_condition } => *child_care && *marital_condition,
            Self::Minor { student, principal_occupation } => !*student && *principal_occupation,
            Self::Spouse | Self::ChildUnder21 => false,
        }
    }
    fn futa(&self) -> bool {
        matches!(self, Self::Ordinary | Self::Minor { .. })
    }
}
fn valid_pay(pay: &[HouseholdPay]) -> bool {
    pay.len() <= 400
        && pay.iter().all(|p| p.date / 10000 == 2026 && money(p.cash) && state(&p.state))
}
impl HouseholdSickFunding {
    fn ratio(&self, date: u32) -> (i64, i64) {
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
    fn len(&self) -> usize {
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
struct PayrollPayment<'a> {
    source: &'a HouseholdPay,
    fica: i64,
    futa: i64,
    state_paid: bool,
}
impl<'a> PayrollPayment<'a> {
    fn new(source: &'a HouseholdPay, amount: i64, state_paid: bool) -> Self {
        Self {
            source,
            fica: if source.treatment.fica() { amount } else { 0 },
            futa: if source.treatment.futa() { amount } else { 0 },
            state_paid,
        }
    }
}
fn month(date: u32) -> u32 {
    date / 10000 * 12 + date / 100 % 100
}
impl HouseholdDisability {
    fn valid(&self) -> bool {
        self.wages.valid() && money(self.social_withheld) && money(self.medicare_withheld)
    }
    fn payments(&self) -> Vec<PayrollPayment<'_>> {
        match &self.wages {
            HouseholdSickWages::Reported { pay } => {
                pay.iter().map(|p| PayrollPayment::new(p, p.cash, true)).collect()
            }
            HouseholdSickWages::Plan { funding, pay } => {
                pay.iter()
                    .map(|p| {
                        let (employer, total) = funding.ratio(p.payment.date);
                        let mut amount = fraction(p.payment.cash, employer, total);
                        let ended = match p.ended {
                            Some(HouseholdSickEnd::Death { date, definite_plan }) => {
                                p.payment.date / 10000 > date / 10000
                                    || (definite_plan && p.payment.date >= date)
                            }
                            Some(HouseholdSickEnd::Retirement { date, definite_plan }) => {
                                definite_plan && p.payment.date >= date
                            }
                            None => false,
                        };
                        if month(p.payment.date) > month(p.last_worked) + 6 || ended {
                            amount = 0;
                        }
                        let mut resolved = PayrollPayment::new(&p.payment, amount, true);
                        if p.disability
                            .as_ref()
                            .is_some_and(|d| d.date / 10000 < 2026 && d.no_services)
                        {
                            resolved.fica = 0; // This statutory exclusion does not exclude FUTA.
                        }
                        resolved
                    })
                    .collect()
            }
        }
    }
}
impl HouseholdWorker {
    fn payments(&self) -> Vec<PayrollPayment<'_>> {
        let mut values: Vec<_> =
            self.pay.iter().map(|p| PayrollPayment::new(p, p.cash, false)).collect();
        if let Some(disability) = &self.disability {
            values.extend(disability.payments());
        }
        values
    }
}
impl HouseholdEmployer {
    fn valid(&self) -> bool {
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
                    && s.reduction.is_none_or(|v| v <= 54000)
                    && s.payments.len() <= 100
                    && s.payments.iter().all(|p| money(p.amount))
                    && s.periods.len() <= 20
                    && s.periods.iter().enumerate().all(|(j, p)| {
                        p.starts / 10000 == 2026
                            && p.ends / 10000 == 2026
                            && p.starts <= p.ends
                            && money(p.wages)
                            && p.rate.is_none_or(|v| v <= 1000000)
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

pub(crate) fn calculate(d: &FederalDraft, r: &mut FederalResult) -> i64 {
    let records = &d.screening.as_ref().unwrap().household_employment;
    let mut total = 0;
    for e in records {
        crate::require_localized!(
            r,
            e.valid()
                && e.complete
                && (e.owner == Owner::Taxpayer
                    || d.setup.as_ref().unwrap().filing_status() == FederalStatus::Joint),
            &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
            "Household employment",
            "ISSUE_CONFIRM_COMPLETE_PAYROLL_RECORDS_AND_ONE_ELIGIBLE_EMPLOYER_RECORD_P_8E42C894",
            "Confirm complete payroll records and one eligible employer record per filer; unresolved payroll exceptions need their own treatment.",
        );
        if !e.valid() {
            continue;
        }
        let mut quarters = [0_i64; 4];
        let mut social = 0;
        let mut medicare = 0;
        let mut additional = 0;
        let mut withholding = 0;
        let mut state_withheld = 0;
        let mut futa = 0;
        let mut state_futa = alloc::vec![0_i64;e.states.len()];
        let mut missing_state = false;
        let mut uncovered = 0;
        for w in &e.workers {
            let mut payments = w.payments();
            let covered: i64 = payments.iter().map(|p| p.fica).sum();
            if covered >= 300000 {
                social +=
                    phase::cap(covered, [crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS]);
                medicare += covered;
                additional += positive(covered - 20000000);
            }
            if let Some(notice) = &w.disability {
                let state_covered: i64 =
                    payments.iter().filter(|p| p.state_paid).map(|p| p.fica).sum();
                let social_limit = if covered >= 300000 {
                    fraction(
                        phase::cap(
                            state_covered,
                            [crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS],
                        ),
                        62,
                        1000,
                    )
                } else {
                    0
                };
                let medicare_limit = if covered >= 300000 {
                    fraction(state_covered, 145, 10000)
                        + fraction(
                            phase::cap(state_covered, [positive(covered - 20000000)]),
                            9,
                            1000,
                        )
                } else {
                    0
                };
                // Notices sum tax rounded on each payment. Allow those cent differences
                // without replacing the actual state deposits with calculated amounts.
                let rounding = (notice.wages.len() as i64 + 1) / 2 + 1;
                crate::require_localized!(
                    r,
                    notice.complete
                        && notice.social_withheld <= social_limit + rounding
                        && notice.medicare_withheld <= medicare_limit + 2 * rounding,
                    &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
                    "Household disability notice",
                    "ISSUE_CONFIRM_THE_STATE_PAYROLL_NOTICE_EMPLOYER_LIABILITY_AND_DEPOSITED_E_CFDFB10A",
                    "Confirm the state payroll notice, employer liability and deposited employee taxes; reconcile withholding with the covered wages and annual limits.",
                );
                state_withheld += notice.social_withheld + notice.medicare_withheld;
            }
            withholding += w.withholding;
            let mut remaining = 700000;
            payments.sort_by_key(|p| p.source.date);
            for p in payments {
                if p.futa == 0 {
                    continue;
                }
                let quarter = ((p.source.date / 100 % 100 - 1) / 3) as usize;
                quarters[quarter] += p.futa;
                let taxable = phase::cap(p.futa, [remaining]);
                remaining = crate::kernel::carry::native_balance(remaining, taxable);
                futa += taxable;
                if !p.source.state_covered {
                    uncovered += taxable;
                }
                if p.source.state_covered && taxable > 0 {
                    if let Some(i) = e.states.iter().position(|s| s.state == p.source.state) {
                        state_futa[i] += taxable;
                    } else {
                        // Delay the missing-state issue until FUTA liability is established.
                        missing_state = true;
                    }
                }
            }
        }
        let due_futa =
            quarters.iter().chain(e.prior_quarters.iter().map(|v| &v.cash)).any(|v| *v >= 100000);
        let mut unemployment = 0;
        let mut futa_credit = 0;
        let mut credit_reduction = 0;
        if due_futa {
            crate::require_localized!(
                r,
                !missing_state,
                &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
                "Household employment",
                "ISSUE_ENTER_UNEMPLOYMENT_RECORDS_AND_THE_FINAL_CURRENT_YEAR_CREDIT_REDUCTION_RATE_07BEA69A",
                "Enter unemployment records and the final 2026 credit-reduction rate for each state covering FUTA wages.",
            );
            let mut additional_credit = 0;
            let mut timely = 0;
            let mut late = 0;
            let mut reduction = 0;
            let mut unknown_reduction = false;
            for (i, s) in e.states.iter().enumerate() {
                for p in &s.periods {
                    if let Some(rate) = p.rate {
                        let wages = dollars(p.wages);
                        additional_credit += positive(
                            fraction(wages, 54, 1000) - fraction(wages, i64::from(rate), 1000000),
                        );
                    }
                }
                timely +=
                    s.payments.iter().filter(|p| p.date <= 20270415).map(|p| p.amount).sum::<i64>();
                late +=
                    s.payments.iter().filter(|p| p.date > 20270415).map(|p| p.amount).sum::<i64>();
                if state_futa[i] > 0 {
                    if s.reduction.is_none() {
                        unknown_reduction = true;
                        let owner_context =
                            crate::IssueContext::owner(crate::TaxSection::OtherTaxes, e.owner);
                        let context = crate::IssueContext::Record {
                            section: crate::TaxSection::OtherTaxes,
                            identity: owner_context.item("household-states", i).identity(),
                            label: crate::Message::new(
                                "NOTE_WITH_LABEL",
                                alloc::vec![
                                    crate::MessageArg::Nested(alloc::boxed::Box::new(
                                        owner_context.label()
                                    )),
                                    crate::MessageArg::Text(s.state.clone()),
                                ],
                            ),
                        };
                        crate::federal_rules::note_once_scoped(
                            r,
                            crate::Message::new(
                                "WARN_HOUSEHOLD_FUTA_CREDIT_ASSUMED_NONE",
                                alloc::vec![
                                    crate::MessageArg::Resource(
                                        if e.owner == Owner::Spouse {
                                            "PERSON_SPOUSE"
                                        } else {
                                            "PERSON_YOU"
                                        }
                                        .into(),
                                    ),
                                    crate::MessageArg::Text(s.state.clone()),
                                ],
                            ),
                            context,
                        );
                    }
                    reduction += fraction(
                        dollars(state_futa[i]),
                        i64::from(s.reduction.unwrap_or(0)),
                        1000000,
                    );
                }
            }
            let wages = dollars(futa);
            let maximum_credit = fraction(wages, 54, 1000);
            let base_credit = additional_credit + dollars(timely);
            let late_credit = fraction(
                phase::cap(positive(maximum_credit - base_credit), [dollars(late)]),
                90,
                100,
            );
            let credit = if unknown_reduction {
                0
            } else {
                positive(phase::cap(maximum_credit, [base_credit + late_credit]) - reduction)
            };
            let section_a = e.states.len() == 1
                && uncovered == 0
                && e.states[0].reduction == Some(0)
                && timely >= e.states[0].required;
            credit_reduction = reduction;
            unemployment =
                if section_a { fraction(wages, 6, 1000) } else { fraction(wages, 6, 100) - credit };
            futa_credit = fraction(wages, 6, 100) - unemployment;
        }
        let ss = fraction(dollars(social), 124, 1000);
        let med = fraction(dollars(medicare), 29, 1000);
        let add = fraction(dollars(additional), 9, 1000);
        let income = dollars(withholding);
        let state_offset = dollars(state_withheld);
        crate::require_localized!(
            r,
            state_offset <= ss + med + add,
            &crate::IssueContext::Section(crate::TaxSection::OtherTaxes),
            "Household disability notice",
            "ISSUE_STATE_DEPOSITED_EMPLOYEE_PAYROLL_TAX_CANNOT_EXCEED_COMBINED_SCHEDUL_F2437532",
            "State-deposited employee payroll tax cannot exceed combined Schedule H payroll tax; reconcile the notice before completing this return.",
        );
        let employer_total = ss + med + add + income - state_offset + unemployment;
        total += employer_total;
        let owner = usize::from(e.owner == Owner::Spouse);
        emit_lines(
            r,
            &mut (),
            [
                ("social_wages", dollars(social)),
                ("medicare_wages", dollars(medicare)),
                ("futa_wages", if due_futa { dollars(futa) } else { 0 }),
                ("futa_credit", futa_credit),
                ("futa_credit_reduction", credit_reduction),
                ("social_tax", ss),
                ("medicare_tax", med),
                ("additional_medicare", add),
                ("employee_withholding", income),
                ("state_disability_tax_offset", state_offset),
                ("futa_tax", unemployment),
                ("tax", employer_total),
            ],
            |(name, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("household_{owner}_{name}"),
                    LineValue::MoneyDollars(amount),
                    &["screening.household_employment"],
                    "https://www.irs.gov/publications/p926",
                    msg!("EXPLAIN_HOUSEHOLD_N_N"),
                ))
            },
        );
    }
    if !records.is_empty() {
        r.review_notes.push(crate::federal_model::ReviewNote::localized(#[cfg(test)] "Household payroll: Schedule H taxes enter Schedule 2, not your personal withholding credit. Include eligible care or medical employment costs in their expense records only once; do not automatically deduct all wages or taxes. Separate W-2 reporting and payroll filing obligations remain outside this prototype.", crate::Message::new("NOTE_HOUSEHOLD_PAYROLL_SCHEDULE_H_TAXES_ENTER_SCHEDULE_2_NOT_YOUR_PER_3D92019D", alloc::vec![])));
    }
    total
}
