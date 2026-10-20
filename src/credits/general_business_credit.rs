// Coverage: Form 3800 — Partial (Unit16 source checkpoint, 2026-10-13).
//   Part I, Part II Sections A/B/C and Parts III/IV use the source-credit catalog,
//   chronological carry ledger, passive allowance, elective-payment and transfer columns.
//   Own source-form amounts and supported K-1 codes reach their recorded Form3800 line;
//   shared line caps precede allocation and the approved childcare-limit notice retains
//   its $500,000 boundary. Schedule3 lines6a/13c/14 include the resulting credit/payment.
//   Research capacity consumes actual business/K-1/rental income, source QBI and deductions,
//   the approved W-2 employer and retained Form4952 origin answers, and common deductions.
//   Current Form461/ScheduleD limits, actual nonpassive rental/QBI identities and
//   portfolio1250 subset amounts retain their existing owner results. Retained
//   capital and NOL source portions use actual retained owners and allowed
//   deductions. Retained charity cash and basis-election portions use actual
//   source original appreciation before allocating the owner's allowed deduction.
//   The existing narrow section1341 combined-comparison guard remains unchanged;
//   its unsupported counterfactual is not a completed source-consumer claim.
//   No generic notice substitutes for unfinished adapters. Whole-map gates remain due.
//   Changed-filer carries consume the genuine shared separate-tax owner at the
//   pre-current-3800 boundary, including delivered account and collected C/F sources.
//   Unavailable supplier source classes remain qualified; explicit component ratios
//   are not that interface. The narrow notice does not cover an unfinished adapter.
//   Printing and incoming2027 amended-return carrybacks remain outside this unit.
//   Approved specified-renewable4e origin authoring retains actual facility
//   identity, original DateOnly and exact source-credit cents. It reuses the
//   physical-origin validity and paired community projection owner; generic1f
//   classification, EPE and unresolved mixed unused-credit order stay separate.
//   Evidence: src/tests/credits/unit16_renewable_facility_origin.rs seven native controls and
//   actual direct/K1 source Save/reopen/restart. Existing seven45Q controls and
//   source lifecycles remain passing; final whole-map gates remain due.
//   Conditional45Q extended carrybacks retain actual equipment identity, original
//   DateOnly and source-credit cents on canonical entered/K1 owners, with exact
//   statement gross reconciliation and paired community source conservation.
//   Missing dates withhold only the older exception; mixed surviving unused origins
//   retain their named post-PAL/use/elective boundary without an invented use order.
//   Evidence: src/tests/credits/unit16_45q_equipment_origin.rs seven native controls and actual
//   source-row/K1 Save/reopen/restart; protected outer K1 import fitting is separate.
//   Evidence: src/tests/credits/form_3800.rs F1–F7, F8 component only, nonresearch F9, transfer and line-cap
//   controls; src/tests/credits/interview.rs actual fresh credit/history and W-2 source Save/reopen/restart;
//   src/tests/credits/credits.rs catalog/source rules. Whole-map acceptance and final gates remain due.
// Coverage: Form 8586 — Supported via the K-1 (audited 2026-10-09 against the 2026 DRAFT f8586 Rev.
//   December 2026 created 7/14/26, identical in its lines to the final Rev. December 2023; no
//   separate instructions exist; the final must be reread). Form 8586 itself is not filed: its
//   general instructions send a taxpayer whose only source is a pass-through entity straight to
//   Form 3800 Part III line 4d, which is what the App emits (no Form 8586 rows).
//   Lines: 4 (the K-1 (1065) box 15 / (1120-S) box 13 codes C and D, section 42(j)(5) and other,
//   one amount each; both are BusinessCredit::LowIncomeHousing, a specified credit from buildings
//   placed in service after 2007, section 38(c)(4)(B)(iii)); 5 to Form 3800 Part III line 4d column
//   (d). The credit is passive (section 469(c)(2)) unless the K-1's rental real estate activity is
//   nonpassive because the owner is a real estate professional who materially participated in it
//   (K1RentalParticipation::Professional with the recorded status): the same box 2 activity then
//   carries the loss and the credit to column (e). Carryforwards: Part IV line 4d (from 2008,
//   BusinessCredit::LowIncomeHousing) and line 2b (BusinessCredit::LowIncomeHousingBefore2008,
//   pre-2008 buildings, not specified, carryforward only: no 2026 amount, no carryback), with the
//   prior-year unallowed amount marked by its Form 8582-CR worksheet (PassiveWorksheet).
//   A building owner's source-form computation (Forms 8609-A/8586 lines 1–3) is not
//   performed here; a completed source-form amount uses General business credits.
//   Recapture (Form 8611; K-1 box 20 codes F/G and box 17 codes E/F) retains its
//   other-information boundary. Other box 15/13 general business credit codes now
//   use their actual credit catalog. A pre-1990 pass-through interest uses
//   Form 8582-CR Worksheet2 and PartIII through the housing-acquisition answer. Form 8586 lines 6 and 7 (estates and trusts) are not an
//   individual's. Evidence: src/tests/credits/housing_credit.rs fixture_a_the_special_allowance_allows_the_whole_
//   credit, fixture_a2_..., fixture_a3_a_pre_2008_carryforward_is_part_i_line_2b, fixture_b_...,
//   fixture_e_codes_c_and_d_merge_on_line_4d_and_each_code_once, fixture_e_carryover_rules_of_the_
//   housing_credits, fixture_f_a_k1_with_a_box_2_loss_and_a_credit_is_one_activity; src/tests/credits/interview.rs
//   k1_codes_c_and_d_reach_form_3800_line_4d_and_reopen, k1_import_instructions_name_codes_c_and_d_
//   and_the_pre_1990_case, business_credit_carryovers_round_trip_through_more_credits; 26 U.S.C. 42,
//   38(c)(4)(B)(iii), 469(c)(2), (c)(7); i1065sk1 (2025), i1120ssk (2026 draft), i3800 (2026 draft).
// Coverage: Form 8582-CR — Source implementation extended (2026-10-13).
//   Worksheets1–4 and PartsI–V classify the source credit, use net passive-income tax,
//   then the active-rental, pre-1990 housing/rehabilitation and post-1989 housing allowances.
//   Lines6/15/27/35 use the return's regular tax method, including Form8615 and ScheduleJ,
//   with AGI-dependent items held fixed. Allocation preserves passive closing balances.
//   PartVI's section469(j)(9) basis election remains refused; printing is deferred.
//   Evidence: src/tests/credits/form_3800.rs F3/F6/F7; src/tests/credits/housing_credit.rs method/allowance fixtures;
//   src/tests/credits/investment_credit.rs rental rehabilitation and Worksheet2 controls; IRS f8582cr,
//   i8582cr, section469(i)/(j)(5). Full Unit16 map acceptance remains due.
// Coverage: Form 5884 — Supported (audited 2026-09-27, unit T3-3800).
//   Lines: 1a (25%, 120–399 hours), 1b (40%, 400+ hours), 1c (50% second-year long-term family
//   assistance), 2 and 4 to Form 3800 line 4b; group limits $6,000/$12,000/$14,000/$24,000/
//   $10,000 less wages counted on the 2025 form; employees who began work before 2026 only (the
//   credit expired for later hires); Schedule C line 26 reduced by line 2; line 3 is reported
//   directly on Form 3800 from K-1 code J. Its source-form computation retains the
//   successor/controlled-group, summer-youth/designated-community-resident (no 2026
//   wages), passive/non-Schedule C and matching-W2 wage boundaries. Completed
//   source-form amounts use General business credits with the actual source.
//   Evidence: as Form 3800;
//   src/tests/credits/credits.rs form_5884_and_8881_limits_by_hand, form_5884_start_dates_and_payroll_are_refused_
//   by_name; i5884 (Rev. 3-2021).
// Coverage: Form 8881 — Supported (audited 2026-09-27, unit T3-3800).
//   Lines: Part I Section 1 lines A, 1–5 (100%/50% by line A, $500 floor, $250 per eligible
//   non-HCE, $5,000 cap; start-up deduction reduced by line 5), Part II line 9 ($500 for three
//   years), lines 7, 10, 14 reported directly on Form 3800 from K-1 codes AE, AF, AG.
//   Section 2 contributions and Part III own-plan amounts are not recomputed from
//   facts here; completed source-form amounts use General business credits.
//   Evidence: as Form 3800; i8881 (Rev. 12-2026 draft).
// Coverage: Form 8941 — Supported (audited 2026-10-09 against the 2026 DRAFT f8941 created 4/8/26
//   and the 2026 draft i8941 dated May 18, 2026; the final revision must be reread). One Schedule C
//   (ordinary activity, U.S. source, material participation, BusinessPayroll::Yes) per owner, on the
//   business page's "More about this business" (BusinessDetails.small_employer_health).
//   Lines: A (SHOP Marketplace or the county exception after a 2025 claim; other coverage earns no
//   credit and is left out with WARN_SMALL_EMPLOYER_HEALTH_NOT_SHOP), C (first credit year 2025 or
//   2026: the two-year credit period), 1 and 2 (FTEs: hours of service per 2,080, rounded down, at
//   least 1, at most 2,080 hours per employee, seasonal workers' hours and wages left out), 3 (average
//   box 5 wages of the matched W-2s, rounded down to $1,000), 4-7 (premiums paid, capped per enrolled
//   employee at the county average premium for the tier x months/12 x the employer's share; 50%),
//   8 (FTE phaseout, (FTEs - 10)/15), 9 (wage phaseout, (average - $34,100)/$34,100, Rev. Proc.
//   2025-32 section 3.09, year_constants.rs; none below $34,100, no credit above twice that),
//   10-12 (state subsidies and credits; the credit), 13-14 (enrolled employees and their FTEs),
//   15 (the owner's K-1 code BA amounts), 16 (to Form 3800 Part III line 4h); Worksheets 1-7. Schedule
//   C line 14 is reduced by line 12 (section 280C(h)); Form 7206 is not (the owner is not an employee,
//   section 45R(e)(1)(A)(i)). A listed employee is chosen from the business's payroll W-2s
//   (an EnumSource keyed on the W-2 label, also for Forms 5884 and 8846).
//   Entered by the person, not looked up: the two county average premiums of the 2026 table, which
//   the IRS publishes in November 2026 (draft i8941 p. 10 is a placeholder); a later unit may embed
//   the table. Excluded or left out, each stated on its page or by a note: line B (EIN) and the
//   Marketplace identifier (printing only), lines 17-20 (cooperatives, estates, trusts and tax-exempt
//   employers), the 35% tax-exempt rate, a farm employer's own source-form computation,
//   household employees, other businesses, leased employees, controlled groups, Hawaii
//   and several rating areas (No leaves the source computation out with
//   WARN_SMALL_EMPLOYER_HEALTH_CREDIT_LEFT_OUT), K-1 (1041) code G and Form 1099-PATR
//   box 12 allocations. Completed individual employer source-form amounts use
//   General business credits with the actual source. K-1 code BA and carryforwards reach Form 3800
//   line 4h on their own entries (specified credit). Lines 1, 2, 13 and 14 are counts (LineKind::Count,
//   federal_rules.rs form_count): the review shows their number in the row label, not as dollars.
//   Evidence: src/tests/credits/credits.rs form_8941_full_credit_with_a_seasonal_worker_and_a_partial_year (fixture A
//   and A2 with a K-1 code BA), form_8941_fte_phaseout_and_the_premium_cap (B, 14/15/25 FTEs),
//   form_8941_wage_phaseout_and_the_state_credit (C, the $68,000 and $69,000 boundaries),
//   form_8941_paths_notices_and_consistency; src/tests/credits/interview.rs
//   health_credit_round_trips_through_the_business_page_and_reduces_line_14,
//   health_credit_page_rules_are_checked_on_save,
//   employer_credit_rows_choose_their_w2_from_the_payroll; 2026 draft f8941 and i8941, i1040sc line
//   14, Rev. Proc. 2025-32, 26 U.S.C. 45R and 280C(h).
// Coverage: Form 8994 — Supported (audited 2026-10-09 against the 2026 DRAFT f8994 Rev. December 2026
//   created 5/5/26; no 2026 instructions are posted, so the rules come from 26 U.S.C. 45S as amended
//   by P.L. 119-21 section 70304, Notice 2026-28 and the Rev. December 2024 i8994 worksheets; the
//   final form and first instructions must be reread). One Schedule C (ordinary activity, U.S.
//   source, material participation, BusinessPayroll::Yes) per owner, on the business page's "More
//   about this business" (BusinessDetails.paid_leave). A listed employee is chosen from the
//   business's payroll W-2s (an EnumSource keyed on the W-2 label).
//   Lines: A, B, D (the confirmed written policy, 50% rate and non-interference statement; No leaves
//   the credit out with WARN_PAID_LEAVE_CREDIT_LEFT_OUT), C (an employee row or the insurance row,
//   PAID_LEAVE_NOTHING_CLAIMED), 1a (wage method: Worksheet Part 1, leave wages at most the normal
//   hourly rate x the leave hours, at most 12 weeks, times 12.5% + 0.25 point per point of the
//   policy's rate above 50%, at most 25%, section 45S(a)(2); qualifying employee: 20 hours a week and
//   2025 compensation of $96,000 or less are ranges on the row, year_constants.rs; wages counted for
//   Form 5884 and wages over the W-2's Medicare wages are refused, section 45S(g)), 1b (premium
//   method: Worksheet Part 2, creditable premiums at the lowest rate of the policies, Notice 2026-28),
//   2 (the owner's K-1 code BB amounts), 3 (to Form 3800 Part III line 4j). Schedule C line 26 is
//   reduced by line 1a and line 14 by line 1b (section 280C(a); the premium line is the inventor's
//   decision of 2026-10-09, the draft Schedule C instructions do not name it); the line 14 premiums
//   of Form 8941 and 8994 together stay within the employee benefit programs expense. One rate per
//   employee and one insurance row, the lowest rate when several apply. The one-year (or elected
//   six-month) service period, the leave purposes, state-paid leave (counts toward the policy
//   minimums, never toward the credit), the written policy and the controlled-group aggregation of
//   section 45S(c)(3) are statements inside the confirmed answer; one record per owner
//   (CREDIT_ONE_BUSINESS_PER_OWNER). K-1 code BB amounts and 2018-2025 carryforwards reach Form 3800 line
//   4j on their own entries (specified credit). Excluded: printing (name, identifying number), a
//   farm (Schedule F has no employer-credit fields), several rates for one employee, several
//   insurance rows, the section 45S(h) election out (add no record).
//   Evidence: src/tests/credits/credits.rs form_8994_wage_method_follows_the_instructions_example (fixture A),
//   form_8994_rate_floor_cap_and_the_twelve_week_limit (B, the ranges), form_8994_insurance_premiums_
//   and_k1_code_bb (C, C2, C3, passive and active K-1), form_8994_paths_notices_and_the_employee_tests
//   (D); src/tests/credits/interview.rs paid_leave_credit_round_trips_through_the_business_page_and_reduces_lines_
//   26_and_14, paid_leave_page_rules_are_checked_on_save, employer_credit_rows_choose_their_w2_from_
//   the_payroll; 2026 draft f8994, i8994 (Rev. 12-2024), Notice 2026-28, 26 U.S.C. 45S and 280C(a).

use crate::kernel::lines::{LineValue, emit as emit_lines, row as calculation_row};
use crate::kernel::lines::{form_line, line, line_cents};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
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
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployerTipMonth {
    /// Hours worked in hundredths of an hour
    /// Enter 16050 for 160.50 hours. Form 8846 compares monthly wages with $5.15 per hour for food or beverages, or $7.25 per hour for qualifying beauty services.
    #[libertas_number(min = 0)]
    pub hours_hundredths: u32,
    /// Medicare-taxable wages paid by this employer before tips
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub wages_excluding_tips: i64,
    /// Customer tips reported to this employer and taxed for employer FICA
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub reported_tips: i64,
}

/// One tipped employee's 2026 payroll record
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployerTipEmployee {
    /// Employee's W-2
    /// Choose this employee's W-2 in this business's Employee payroll. Enter a missing W-2 under Employees' W-2 wages first.
    #[libertas_enum_source("^^^^.payroll.Yes.statements")]
    pub label: String,
    /// January through December 2026 payroll records
    /// ----
    /// Calendar month
    // Tax structure: one entry for each of the twelve calendar months.
    #[libertas_size(min = 12, max = 12)]
    pub months: Vec<EmployerTipMonth>,
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
    /// ----
    /// Employee payroll and tip record
    #[libertas_size(min = 1)]
    pub employees: Vec<EmployerTipEmployee>,
    /// Actual employer FICA paid or incurred on these reported customer tips
    /// Reconcile the employer Social Security and Medicare tax on these tips to payroll records and Forms 941, including payroll-period rounding. Enter cents. Exclude employee tax, Additional Medicare Tax, non-tip wages, unreported tips, and mandatory service charges. Ordinary Business expenses must omit this amount: the App deducts it after reducing it by Form 8846 line 4.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub actual_employer_tip_fica_paid: i64,
    /// Current-year unused credit actually used on a 2025 carryback return
    /// Enter whole dollars from the retained 2025 Form 3800 carryback worksheet and Form 1040-X or Form 1045, or zero after confirming no 2025 capacity or another valid reason for no use. The App calculates 2026 tax use first; only the remaining 2026 credit is available for a 2025 carryback, then a 2027 carryforward.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub used_prior_year_carryback: i64,
    /// Confirm eligible service, payroll and carryback records
    /// This is one directly owned, actively operated U.S. Schedule C employer. All entered tips came from customer services in the selected eligible food/beverage or beauty categories where tipping is customary. Every tipped employee and monthly wage/tip report is entered. The employee reports and employer FICA payments were reconciled to payroll returns and W-2 box 5; no employee exceeds this employer's 2026 Social Security wage base, no predecessor or successor payroll, no service charge, no pass-through or passive credit, and no credit election or other Form 3800 amount is omitted. Ordinary expenses exclude employer FICA on these tips but include all other payroll costs once. The retained 2025 return supports the entered carryback use, including zero. No leaves this credit out of this estimate.
    pub confirmed: bool,
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
                        month.hours_hundredths <= 100_000
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
                && employee
                    .months
                    .iter()
                    .all(|month| month.reported_tips == 0 || month.hours_hundredths > 0)
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
                    i64::from(month.hours_hundredths) * basis_cents_per_hour,
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

/// 2025 employer tip FICA credit carried to the 2026 return
/// This is a single 2025-origin Form 8846 credit from an active, directly owned employer business. Retain the 2025 Form 8846 and Form 3800 Part III line 4f, the 2025 tax-use worksheet, and any 2024 carryback return. A 2025-origin credit is not reported in 2025 Form 3800 Part IV; it first appears in 2026 Part IV line 4f.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct EmployerTipFicaCarryforward {
    /// Filing status on the 2025 credit return
    pub prior_status: FederalStatus,
    /// Whole-dollar credit reported on 2025 Form 3800 Part III line 4f, drawn from Form 8846 line 6
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub original_credit: i64,
    /// 2025 Form 3800 credit applied against 2025 tax
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub used_origin_year: i64,
    /// Credit applied on a 2024 carryback return
    /// Enter zero only after confirming no 2024 tax capacity or another valid reason the carryback did not use credit.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub used_origin_prior_year_carryback: i64,
    /// Balance documented for 2026 Form 3800 Part IV line 4f column (f)
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
struct CreditInfo {
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
    section_b: bool,
    cap_cents: Option<i64>,
    elective_payment: bool,
    transferable: bool,
    research: bool,
    magi_limited: bool,
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
    fn info(self) -> &'static CreditInfo {
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
    #[libertas_number(min = 1)]
    pub amount: i64,
}

/// Origin of the carbon oxide credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum CarbonOxideCreditOrigin {
    /// Actual equipment or process-train source statement
    Statement {
        /// Equipment that produced this credit
        /// Enter actual component amounts when one source statement includes multiple pieces of equipment. No proportional or estimated split.
        /// ----
        /// Equipment
        #[libertas_unordered]
        equipment: Vec<CarbonOxideEquipmentCredit>,
    },
}
// The two physical-source classes keep distinct questions and stored labels.
// Their structural validity, date facts and paired community projection have
// one native owner; no component credit-use ordering is decided here.
trait CreditOriginPart: Clone {
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
    !parts.is_empty()
        && parts.len() <= 60
        && parts.iter().enumerate().all(|(i, part)| {
            !part.identity().trim().is_empty()
                && (1..=100_000_000_000).contains(&part.amount())
                && part
                    .original_date()
                    .is_none_or(|date| crate::federal_rules::valid_date(date) && date <= 20261231)
                && !parts[..i].iter().any(|prior| prior.identity() == part.identity())
        })
        && parts.iter().try_fold(0i64, |sum, part| sum.checked_add(part.amount())) == Some(gross)
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
    #[libertas_number(min = 1)]
    pub amount: i64,
}
/// Origin of the specified renewable-electricity credit
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum RenewableFacilityCreditOrigin {
    /// Actual facility source statement
    Statement {
        /// Facilities that produced this credit
        /// Enter actual component amounts when the statement includes multiple facilities. No proportional or estimated split.
        /// ----
        /// Facility
        #[libertas_unordered]
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

/// Credit on the K-1
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct K1Credit {
    /// Code
    pub code: K1CreditCode,
    /// Form 3800 line supplied by the statement or eligible-small-business answer
    pub line: Option<BusinessCredit>,
    /// Amount
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
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

    pub(crate) fn valid_list(items: &[Self]) -> bool {
        Self::issue_list(items).is_none()
    }
    pub(crate) fn issue_list(items: &[Self]) -> Option<ValidationIssue> {
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
                item.credit() != BusinessCredit::CarbonOxide || !origin.valid(item.amount)
            }) {
                Some("carbon_oxide_origin")
            } else if item.renewable_facility_origin.as_ref().is_some_and(|origin| {
                item.credit() != BusinessCredit::RenewableElectricitySpecified
                    || !origin.valid(item.amount)
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
        #[libertas_number(min = 0)]
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
    #[libertas_enum_source("^^^^.payroll.Yes.statements")]
    pub label: String,
    /// Targeted group on the certification
    #[libertas_default(Standard)]
    pub group: WorkOpportunityGroup,
    /// Date the employee began work
    /// The credit covers only employees who began work before 2026. First-year wages in 2026 need a start date in 2025 after January 1; long-term family assistance second-year wages need a start date after January 1, 2024.
    #[libertas_date_only]
    pub began: u32,
    /// Hours worked for you in the employee's first year
    /// At least 120. Fewer than 400 hours earns 25% of first-year wages; 400 or more earns 40%.
    // Tax structure: the employee’s first-year period contains at most 366 days of 24 hours.
    #[libertas_number(min = 120, max = 8784)]
    pub first_year_hours: u32,
    /// First-year wages paid in 2026
    /// FUTA-type wages, without the $7,000 wage base, for work in the year that began on the start date.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
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
    /// ----
    /// Employee
    #[libertas_unordered]
    #[libertas_size(min = 1)]
    pub employees: Vec<WorkOpportunityEmployee>,
    /// Certifications and wages confirmed
    /// Each employee has a state workforce agency certification requested on Form 8850 within 28 days of starting work, is not your relative or dependent, did not work for you before, and more than half of the wages were for work in this business. The wages exclude federally funded on-the-job training pay and wages used for another employment credit. Business expenses include these wages in full on line 26. Each employee's state workforce agency certification (the Form 8850 response) shows it. A successor employer, a controlled group, summer youth employees or designated community residents need No, which leaves this credit out of this estimate.
    pub confirmed: bool,
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
    #[libertas_number(min = 0)]
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
    /// Months of 2026 the employee was enrolled
    // Tax structure: a calendar year has twelve months.
    #[libertas_number(min = 1, max = 12)]
    pub months: u32,
    /// Total premium charged for the employee's coverage in those months
    /// Your share plus the employee's share, from the SHOP invoices, without any tobacco surcharge.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub total_premium: i64,
    /// Premiums you paid (Worksheet 4 column (b))
    /// Your own share only. Not what the employee paid, including pretax salary reductions through a cafeteria plan, and not a tobacco surcharge. A state subsidy paid directly to the insurer counts as paid by you; do not subtract a state subsidy or credit paid to you (enter it below as state premium subsidies and tax credits).
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub employer_paid: i64,
}

/// Hours of service in 2026 (Worksheet 1 column (b))
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum HealthCreditService {
    /// Hours of service
    Hours {
        /// Hours of service, at most 2,080
        /// Hours paid for work plus paid vacation, holiday, sick and other leave (at most 160 hours for one continuous leave). You may instead count 8 hours for each day, or 40 hours for each week, with at least one hour of service.
        // Form 8941 tax structure: count no more than 2,080 service hours per employee.
        #[libertas_number(min = 1, max = $SMALL_EMPLOYER_HEALTH_FTE_HOURS)]
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
    #[libertas_number(min = 1)]
    pub employee_only_average: i64,
    /// Average family premium for your county
    /// The same table row's "Dependent, Family, etc." column.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub family_average: i64,
    /// Employees (Worksheet 1)
    /// Everyone you employed in 2026, including seasonal workers, except: you, a partner or owner of more than 5% (more than 2% of an S corporation), anyone's spouse among these, their children and other descendants, siblings and step-siblings, parents and other ancestors, stepparents, nieces, nephews, aunts, uncles and in-laws, and a household member who is the dependent of one of them.
    /// ----
    /// Employee
    #[libertas_unordered]
    #[libertas_size(min = 1)]
    pub employees: Vec<HealthCreditEmployee>,
    /// State premium subsidies and tax credits (line 10)
    /// State tax credits you can take, and state premium subsidies paid to you or to the insurer, for the premiums entered above.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    #[libertas_default(0)]
    pub state_subsidies: i64,
    /// Eligible small employer confirmed
    /// Your arrangement pays the same percentage, at least 50%, of each enrolled employee's premium (or a composite or list-billing arrangement the Form 8941 instructions allow) through plans in this one county's rating area; the business is not in Hawaii. Every employee listed above has a W-2 in this business's Employee payroll, and you have no other business with employees, no household employee and no leased employee; this business is not part of a controlled group, an affiliated service group or a commonly controlled group (a spouse's separate business counts only if either of you works in or manages the other's). The premiums you paid are included once in this business's employee benefit programs expense. Your payroll and hour records, SHOP invoices and earlier returns show it. No leaves this credit out of this estimate.
    pub confirmed: bool,
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
                crate::kernel::lines::form_count
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
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PaidLeaveEmployee {
    /// Employee's W-2
    /// Choose this employee's W-2 in this business's Employee payroll. Enter a missing W-2 under Employees' W-2 wages first. Not your spouse, your child under 21 or your parent: their wages are not FUTA wages and earn no credit.
    #[libertas_enum_source("^^^^.payroll.Yes.statements")]
    pub label: String,
    /// Hours the employee customarily works each week
    /// At least 20; an employee customarily working fewer hours is not a qualifying employee. It also sets the 12-week limit on the leave counted.
    #[libertas_number(min = $PAID_LEAVE_MIN_WEEKLY_HOURS)]
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
    #[libertas_number(min = 1)]
    pub normal_hourly_rate: i64,
    /// Hours of family and medical leave paid in 2026, in hundredths of an hour
    /// Enter 16000 for 160.00 hours. Count only leave under your written policy for a family and medical leave purpose, taken while the employee was a qualifying employee, and at most 12 weeks of the employee's usual hours.
    #[libertas_number(min = 1)]
    pub leave_hours_hundredths: u32,
    /// Wages you paid for that leave (worksheet column (b))
    /// Only what you paid under your policy for those hours. Not leave paid by a state or locality or required by state or local law, not leave paid by an insurance policy whose premiums you claim below, and not wages you counted for the work opportunity credit.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub leave_wages: i64,
    /// Rate of payment under your policy, in hundredths of a percent
    /// The share of normal wages your policy pays during this leave: enter 7500 for 75% and 10000 for 100% or more. It must be at least 50%. If this employee's leave was paid at more than one rate, enter the lowest.
    // Percentage encoding: 10,000 ten-thousandths is 100 percent.
    #[libertas_number(min = $PAID_LEAVE_MIN_RATE_HUNDREDTHS, max = 10000)]
    pub rate_hundredths: u32,
}

impl PaidLeaveEmployee {
    fn valid(&self) -> bool {
        !self.label.trim().is_empty()
            && (PAID_LEAVE_MIN_WEEKLY_HOURS..=80).contains(&self.weekly_hours)
            && (0..=PAID_LEAVE_COMPENSATION_LIMIT_2026_CENTS)
                .contains(&self.prior_year_compensation)
            && (1..=100_000).contains(&self.normal_hourly_rate)
            && (1..=96_000).contains(&self.leave_hours_hundredths)
            && (1..=100_000_000_000).contains(&self.leave_wages)
            && Self::rate_valid(self.rate_hundredths)
            && !self.above_12_weeks()
    }

    fn rate_valid(rate: u32) -> bool {
        (PAID_LEAVE_MIN_RATE_HUNDREDTHS..=10_000).contains(&i64::from(rate))
    }

    // Section 45S(b)(3): at most 12 weeks of the employee's usual hours.
    fn above_12_weeks(&self) -> bool {
        i64::from(self.leave_hours_hundredths)
            > PAID_LEAVE_MAX_WEEKS * 100 * i64::from(self.weekly_hours)
    }

    // Section 45S(b)(1): wages at most the normal hourly rate for the leave
    // hours, in cents.
    fn credit_wages(&self) -> i64 {
        phase::cap(
            self.leave_wages,
            [fraction(self.normal_hourly_rate, i64::from(self.leave_hours_hundredths), 100)],
        )
    }

    // Worksheet Part 1 column (d) in cents.
    fn credit_cents(&self) -> i64 {
        paid_leave_credit_cents(self.credit_wages(), self.rate_hundredths)
    }
}

// The applicable percentage of section 45S(a)(2) (12.5% at a 50% rate, plus
// 0.25 point per point above it, at most 25%) times an amount in cents. The
// percentage is kept in quarters of a hundredth of a percent, so a rate of
// 66.67% needs no rounding of points.
fn paid_leave_credit_cents(amount: i64, rate_hundredths: u32) -> i64 {
    let quarters = min(
        PAID_LEAVE_RATE_POINTS_PER_PERCENT_POINT * PAID_LEAVE_MAX_PERCENT_HUNDREDTHS,
        PAID_LEAVE_RATE_POINTS_PER_PERCENT_POINT * PAID_LEAVE_BASE_PERCENT_HUNDREDTHS
            + i64::from(rate_hundredths)
            - PAID_LEAVE_MIN_RATE_HUNDREDTHS,
    );
    fraction(amount, quarters, PAID_LEAVE_RATE_POINTS_PER_PERCENT_POINT * 10_000)
}

/// Paid family and medical leave insurance (premium method)
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PaidLeaveInsurance {
    /// Creditable premiums you paid or incurred in 2026
    /// Only the part of the premiums that funds paid family and medical leave for qualifying employees that the wage method could credit: not coverage for other leave, for employees who do not qualify, or for leave required or paid by a state or locality. Split a premium that covers both by a reasonable method your records support. These premiums are included once in this business's employee benefit programs expense; the App reduces that deduction by the credit.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 1)]
    pub premiums: i64,
    /// Rate of payment under the policy, in hundredths of a percent
    /// The share of normal wages the policy pays during leave, whether or not anyone took leave in 2026: enter 6000 for 60%. With more than one policy, enter the lowest rate.
    // Percentage encoding: 10,000 ten-thousandths is 100 percent.
    #[libertas_number(min = $PAID_LEAVE_MIN_RATE_HUNDREDTHS, max = 10000)]
    pub rate_hundredths: u32,
}

/// Employer credit for paid family and medical leave (Form 8994)
/// For wages this business paid to qualifying employees on family and medical leave under its written policy, or premiums for a paid leave insurance policy. The App figures Form 8994 lines 1a–3 and reduces the wages and employee benefit programs deductions by the credit.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct PaidLeaveCredit {
    /// Employees paid while on family and medical leave (wage method)
    /// Leave empty when you claim only insurance premiums.
    /// ----
    /// Employee
    #[libertas_unordered]
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

impl PaidLeaveCredit {
    pub(crate) fn valid(&self) -> bool {
        self.employees.len() <= 100
            && self.employees.iter().all(PaidLeaveEmployee::valid)
            && self.insurance.as_ref().is_none_or(|i| {
                (1..=100_000_000_000).contains(&i.premiums)
                    && PaidLeaveEmployee::rate_valid(i.rate_hundredths)
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
                .at_field("leave_hours_hundredths")
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
            .map_or(0, |i| dollars(paid_leave_credit_cents(i.premiums, i.rate_hundredths)));
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
    #[libertas_number(min = 100)]
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
        #[libertas_number(min = 100)]
        amount: i64,
    },
    /// Sold to another taxpayer (section 6418 transfer)
    /// The part you transferred for cash. The cash is not income, and the part sold is not your credit.
    Sold {
        /// Amount sold
        /// Form 3800 Part III column (f), in whole dollars.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 100)]
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
    #[libertas_number(min = 100)]
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
    #[libertas_number(min = 100)]
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
    /// ----
    /// Carryforward
    #[libertas_unordered]
    pub carryforwards: Vec<BusinessCreditCarryforward>,
    /// 2026 credits carried back to earlier returns
    /// ----
    /// Carryback
    #[libertas_unordered]
    pub carrybacks: Vec<BusinessCreditCarryback>,
    /// Your spouse has no general business credit, carryforward or carryback for 2026
    /// Married filing separately only. When true, the $25,000 amount on Form 3800 line 13 applies instead of $12,500.
    #[libertas_default(false)]
    pub spouse_without_credits: bool,
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
                crate::FederalIncome::Partnership { data } => &data.credits,
                crate::FederalIncome::Scorp { data } => &data.credits,
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
                            CarryFiler::You => e.owner == crate::Owner::Taxpayer,
                            CarryFiler::Spouse => e.owner == crate::Owner::Spouse,
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
    fn merge(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    fn extended(self) -> bool {
        self.0 == 4
    }
    fn mixed(self) -> bool {
        self.0 & 6 == 6
    }
    fn unknown_date(self) -> bool {
        self.0 & 1 != 0
    }
    // Statutory section 45/45Q carryback eligibility cutoff, not a relative return year.
    fn before2023(self) -> bool {
        self.0 == 2
    }
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeElectiveGroup {
    pub(crate) component: alloc::string::String,
    pub(crate) label: &'static str,
    pub(crate) amount: i64,
    pub(crate) elected: i64,
    pub(crate) allowed: i64,
    pub(crate) used: i64,
    pub(crate) elective: i64,
    pub(crate) allowed_owners: Option<[i64; 2]>,
    pub(crate) elected_owners: Option<[i64; 2]>,
}

// Native Form3800 amounts are already reported whole Dollars. These fields
// retain that precision and the same invocation's settled allowance/use.
// Current credits also retain ownership with no elective-payment election;
// prior non-elective carries keep their existing separate history context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NativeElectiveSources {
    pub(crate) joint: i64,
    pub(crate) used_joint: i64,
    pub(crate) groups: Vec<NativeElectiveGroup>,
}

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
    d: &crate::FederalDraft,
    history: &BusinessCreditHistory,
    r: &mut FederalResult,
) -> Vec<CreditEntry> {
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
        let passive = match source.map(|e| (&e.income, e.owner)) {
            Some((crate::FederalIncome::Business { data }, _)) => !data.material_participation,
            Some((crate::FederalIncome::Rental { data }, owner)) => {
                !data.participation.professional() || d.professional(owner).is_none()
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
            source_owner: source.map(|entry| entry.owner),
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
