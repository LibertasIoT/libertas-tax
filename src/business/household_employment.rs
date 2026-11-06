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
//   Page (2026-09-28): the Other taxes list's "Household employees" item
//   (OtherTax::HouseholdEmployees, screening.household_employment, rows by owner through
//   allowed_owners; RareSituation::HouseholdEmployer removed). Re-verified against the 2026
//   draft Schedule H (line A $3,000, lines 2/4/6 rates, line C and line 9 $1,000 quarter test,
//   Section A 0.6%, total to 2026 Schedule 2 line 17a) and the $184,500 2026 wage base: no
//   calculation change. Interview fixtures and boundaries: T/other_taxes.rs
//   schedule_h_through_the_other_taxes_page, schedule_h_line_a_threshold_is_three_thousand_dollars,
//   schedule_h_futa_quarter_test_is_one_thousand_dollars, schedule_h_owner_rules_are_checked_on_save.

// Schedule H calculation from payroll records, not employee income on this return.
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::{FederalDraft, FederalResult, FederalStatus, Owner};
use alloc::vec::Vec;
use libertas_macros::*;

pub(crate) use crate::household_employment_source::valid;
pub use crate::household_employment_source::{
    HouseholdDisability, HouseholdEmployer, HouseholdPay, HouseholdQuarter,
    HouseholdSickContribution, HouseholdSickEnd, HouseholdSickEntitlement, HouseholdSickFundPeriod,
    HouseholdSickFunding, HouseholdSickPayment, HouseholdSickPolicy, HouseholdSickPremium,
    HouseholdSickWages, HouseholdState, HouseholdStatePayment, HouseholdStatePeriod,
    HouseholdWageKind, HouseholdWorker,
};

/// Household employer
/// One entry per filer/employer, with all employees paid by that employer. These are wages you paid, not your income. Do not repeat employment taxes reported on Forms 941, 943, 944 or 940, including taxes already reported and paid on your behalf by an authorized government agency or third-party agent. Merely hiring a payroll service does not remove your responsibility.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HouseholdEmployerView {
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

impl From<&HouseholdEmployer> for HouseholdEmployerView {
    fn from(value: &HouseholdEmployer) -> Self {
        Self {
            owner: value.owner,
            workers: value.workers.clone(),
            prior_quarters: value.prior_quarters.clone(),
            states: value.states.clone(),
            complete: value.complete,
        }
    }
}

impl From<&HouseholdEmployerView> for HouseholdEmployer {
    fn from(value: &HouseholdEmployerView) -> Self {
        Self {
            owner: value.owner,
            workers: value.workers.clone(),
            prior_quarters: value.prior_quarters.clone(),
            states: value.states.clone(),
            complete: value.complete,
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
                            fraction(wages, 54, 1000)
                                - fraction(
                                    wages,
                                    crate::kernel::natural::units(rate, 10_000),
                                    1000000,
                                ),
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
                        crate::kernel::natural::units(s.reduction.unwrap_or(0.0), 10_000),
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
                && e.states[0].reduction == Some(0.0)
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
