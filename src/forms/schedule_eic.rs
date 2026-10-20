// Coverage: Schedule EIC — Supported (audited 2026-09-24).
//   Lines: Draft 2026 EIC table lookup on earned income then AGI, 0–3 children, joint/other
//   columns, $12,200 investment-income limit (Pub 596 Worksheet 1), valid SSN, qualifying-child
//   tests (relationship, residency, age/student/disabled, competing claim), childless age/US-home
//   rules, dependent-filer and another's-qualifying-child disallowance, MFS separated rule,
//   foreign-exclusion bar. Form 8814 (2026-10-09, child_election.rs): Worksheet 1 line 2 adds the
//   child's tax-exempt interest (line 1b) and line 4 the election's line 12 less its Alaska
//   Permanent Fund share (Worksheet 2); lines 3 and 7 already hold lines 9 and 10. Caveat: draft
//   table pending final reconciliation.
//   Forms 8332 and 2120 (unit form-8332, 2026-10-09): the qualifying child is determined without
//   regard to IRC 152(e) (section 32(c)(3)(A)), so the custodial parent who released the child
//   keeps the child for the credit and the parent holding the release, whose child did not live
//   with them over half the year, and a multiple support relative have none; tests:
//   custodial_parent_who_released_keeps_head_of_household_eic_and_care and
//   noncustodial_parent_with_a_release_gets_the_child_tax_credit_only (family.rs).
//   Unit family credits F8 (2026-10-10): a child attains 19 or 24 on the birthday (Rev. Rul.
//   2003-72); a child born and died in 2026 without an SSN is a qualifying child (Schedule EIC
//   line 2 "Died"); a married child counts only when the filer can claim the child as a
//   dependent or could but for the section 152(e) release (2025 i1040 "Married child"); the
//   separated spouse filing separately keeps the self-only credit when the qualifying child has
//   no valid SSN (Pub. 596 Rule 3); a joint return skips the "qualifying child of another
//   person" and "can be claimed as a dependent" tests (Step 3 question 2, Step 4 question 4);
//   the special-needs deemed excess reduces earned income; Schedule EIC lines 1, 2, 3, 4a/4b and
//   6 are rows per child (the form lists three; a self-only credit lists Child 1), shown when the
//   credit is claimed. Tests: line_audit_family_credits.rs d1_*, d2_*, d4_*, d5_*, d6_*, d7_*,
//   d13_*.
//   Evidence: forms/schedule_eic.rs (compute: the Step 1–4 gate and the Worksheet 1 investment
//   income; eic_amount, eic_table_2026; the credit on Form 1040 line 27a), federal_rules.rs
//   (family: dependent classification; the earned-income figure shared with Schedule 8812),
//   federal_model.rs FederalDependent; tests:
//   draft_2026_eic_table_matches_every_published_interval_and_column,
//   eic_worksheet_uses_earned_income_then_only_applicable_agi_lookup (health_accounts.rs),
//   another_taxpayers_eic_child_disallows_credit_without_blocking_return,
//   final_joint_childless_eic_can_use_deceased_spouses_age (family.rs); draft 2026 EIC table
//   fixture irs-eic-2026-draft.csv, Pub 596
//   C-fixture (2026-09-25, src/tests/family/family.rs): childless_eic_return_matches_hand_computation (the
//   childless lookup on earned income then AGI by hand), eic_investment_income_limit_is_12200,
//   child_under_six_months_residency_is_not_an_eic_child, eic_lookup_takes_the_smaller_of_earned_income_and_agi.

// Schedule EIC and the EIC worksheet: eligibility, investment income and the
// table credit. Amounts are whole dollars.
use crate::kernel::lines::{form_line, line};
use crate::kernel::phase;
use crate::kernel::ratio::positive;
use crate::{
    FederalDependent, FederalPeople, FederalResult, FormLine, TaxForm, extended_rules::RentalResult,
};
use core::cmp::min;

const SOURCE: &str = "https://www.irs.gov/irb/2025-45_IRB";
const INPUTS: &[&str] = &["agi", "income_tax", "credits", "income", "payments"];

// What the return supplies to Schedule EIC and the EIC worksheet.
pub(crate) struct ScheduleEicContext<'a> {
    // Step 1 filer tests and Step 4 childless tests.
    pub(crate) people: &'a FederalPeople,
    // Pub 596 Worksheet 1 lines 11–13 passive amounts.
    pub(crate) rental: &'a RentalResult,
    // Pub 596 Worksheet 1 lines 1–10.
    pub(crate) interest: i64,
    pub(crate) exempt: i64,
    // Form 8814 line 1b, added to the tax-exempt interest on line 2.
    pub(crate) child_exempt: i64,
    // Form 8814 line 12 less the Alaska Permanent Fund share: line 4 (Pub 596
    // Worksheet 1 line 4 and Worksheet 2).
    pub(crate) child_other: i64,
    pub(crate) dividends: i64,
    // Short- plus long-term capital gain, including Form 4797 capital.
    pub(crate) capital: i64,
    // The Form 4797 capital that Worksheet 1 removes from the capital line.
    pub(crate) property_long_term: i64,
    pub(crate) property_net: i64,
    // Form 2555 elected: no EIC.
    pub(crate) foreign_election: bool,
    // Married filing separately and the separated-spouse exception.
    pub(crate) separate: bool,
    pub(crate) separated: bool,
    // A filer can be claimed as another taxpayer's dependent.
    pub(crate) dependent_filer: bool,
    // Step 3: qualifying children from the dependent classification.
    pub(crate) children: usize,
    // Children who meet every qualifying-child test but have no valid SSN
    // (Pub. 596 Rule 2): Step 3 question 1 is Yes for them, so a separated
    // spouse filing separately keeps the self-only credit.
    pub(crate) children_without_ssn: usize,
    // Schedule EIC: the dependents and the identities of the children it lists.
    pub(crate) dependents: &'a [FederalDependent],
    pub(crate) child_ids: &'a [i64],
    pub(crate) children_without_ssn_ids: &'a [i64],
    // Step 5: earned income including the optional-method adjustment.
    pub(crate) earned: i64,
    pub(crate) agi: i64,
    pub(crate) joint: bool,
    // A running Form 8862 ban (section 32(k)): the credit is zero and no rows
    // are shown; `ScheduleEic::banned` is what the credit would have been.
    pub(crate) banned: bool,
}

// The credit for Form 1040 line 27a and the Step 2 investment income.
pub(crate) struct ScheduleEic {
    pub(crate) credit: i64,
    // The credit a Form 8862 ban took away.
    pub(crate) banned: i64,
    // Pub 596 Worksheet 1 investment income, also shown in the S-corporation review note.
    pub(crate) investment: i64,
    // A filer is another taxpayer's EIC qualifying child.
    another_child: bool,
}

pub(crate) fn compute(ctx: &ScheduleEicContext<'_>, r: &mut FederalResult) -> ScheduleEic {
    let p = ctx.people;
    let passive_property =
        ctx.rental.k1_section1231.iter().map(|value| value.net - value.nonpassive).sum::<i64>();
    let investment = ctx.interest
        + ctx.exempt
        + ctx.child_exempt
        + ctx.child_other
        + ctx.dividends
        + positive(ctx.capital - ctx.property_long_term)
        // Publication 596 Worksheet 1 removes Form 4797 capital from line 7
        // and counts passive property amounts once in lines 11–13.
        // It counts passive net, not all Schedule E.
        // Unlike NIIT, it does not include a nonpassive financial-trading business.
        + positive(ctx.rental.eic_passive_income() + passive_property)
        + positive(ctx.property_net);
    line(
        r,
        "eic_investment_income",
        investment,
        &["income"],
        "https://www.irs.gov/publications/p596",
        msg!("EXPLAIN_EIC_INVESTMENT_INCOME"),
    );
    let valid_eic_ssn = p.taxpayer.valid_ssn && p.spouse.as_ref().is_none_or(|v| v.valid_ssn);
    let eic_age =
        core::iter::once(&p.taxpayer).chain(p.spouse.iter()).any(|v| v.childless_eic_age());
    let childless_home = p.taxpayer.us_home_over_half_year
        && p.spouse.as_ref().is_none_or(|v| v.us_home_over_half_year);
    // Joint returns skip the other-taxpayer qualifying-child and dependent-filer tests.
    let filers = || core::iter::once(&p.taxpayer).chain(p.spouse.iter());
    let not_another_child = ctx.joint || filers().all(|v| !v.another_eitc_child);
    let dependent_filer = !ctx.joint && ctx.dependent_filer;
    let other_tests = valid_eic_ssn
        && investment <= 12200
        && (ctx.children > 0 || (eic_age && childless_home && !dependent_filer));
    // Pub. 596 Rule 3, special rule for separated spouses: a qualifying child
    // who lacks a valid SSN still qualifies the filer for the self-only EIC.
    let separated_eic =
        ctx.separate && ctx.separated && ctx.children + ctx.children_without_ssn > 0;
    let amount = if !ctx.foreign_election && (!ctx.separate || separated_eic) && other_tests {
        eic_amount(ctx.earned, ctx.agi, ctx.children, ctx.joint)
    } else {
        0
    };
    let credit = if not_another_child { amount } else { 0 };
    if credit > 0 && !ctx.banned {
        schedule_eic_rows(ctx, r);
    }
    ScheduleEic {
        credit: if ctx.banned { 0 } else { credit },
        banned: if ctx.banned { credit } else { 0 },
        investment,
        another_child: !ctx.joint
            && (p.taxpayer.another_eitc_child
                || p.spouse.as_ref().is_some_and(|v| v.another_eitc_child)),
    }
}

// Schedule EIC (2026 draft) lines 1-6 for each listed child. The form lists
// three children; a self-only credit lists Child 1 alone (the child without a
// valid SSN). Names and the SSN status are text; the App never holds the
// SSN numbers themselves.
fn schedule_eic_rows(ctx: &ScheduleEicContext<'_>, r: &mut FederalResult) {
    let p = ctx.people;
    let source = "https://www.irs.gov/pub/irs-dft/f1040sei--dft.pdf";
    let inputs = ["dependents"];
    let ids = if ctx.children > 0 { ctx.child_ids } else { ctx.children_without_ssn_ids };
    let younger_than = p
        .spouse
        .as_ref()
        .filter(|_| ctx.joint)
        .map_or(p.taxpayer.birth_date, |v| p.taxpayer.birth_date.min(v.birth_date));
    for (index, id) in ids.iter().take(3).enumerate() {
        let Some(child) = ctx.dependents.iter().find(|v| v.id == *id) else {
            continue;
        };
        let number = index + 1;
        let encoded = || crate::MessageArg::Encoded(child.name());
        let mut row = |line: &str, name: &str, amount: i64, key: &str| {
            crate::kernel::lines::form_value(
                r,
                FormLine::new(TaxForm::ScheduleEic, line),
                &alloc::format!("schedule_eic_child_{number}_{name}"),
                if matches!(line, "1" | "2") {
                    crate::kernel::lines::LineValue::Text
                } else if line == "3" {
                    crate::kernel::lines::LineValue::Year(amount)
                } else {
                    crate::kernel::lines::LineValue::CountWhole(amount)
                },
                &inputs,
                source,
                crate::Message::new(key, alloc::vec![encoded()]),
            );
        };
        row("1", "name", 1, "EXPLAIN_SCHEDULE_EIC_CHILD_N_NAME");
        if child.valid_ssn {
            row("2", "ssn", 1, "EXPLAIN_SCHEDULE_EIC_CHILD_N_SSN");
        } else if child.born_and_died() {
            row("2", "died", 1, "EXPLAIN_SCHEDULE_EIC_CHILD_N_DIED");
        }
        row(
            "3",
            "birth_year",
            i64::from(child.birth_date / 10000),
            "EXPLAIN_SCHEDULE_EIC_CHILD_N_BIRTH_YEAR",
        );
        // Line 3: a child born after 2007 who is younger than the filer skips
        // lines 4a and 4b.
        if !(child.birth_date >= 20080101 && child.birth_date > younger_than) {
            if child.student() {
                row("4a", "student", 1, "EXPLAIN_SCHEDULE_EIC_CHILD_N_STUDENT");
            } else if child.disabled() {
                row("4b", "disabled", 1, "EXPLAIN_SCHEDULE_EIC_CHILD_N_DISABLED");
            }
        }
        let months = match child.residency {
            crate::DependentResidency::WholeYearException => 12,
            crate::DependentResidency::Days { days } => i64::from(days.div_ceil(30)).clamp(7, 12),
        };
        row("6", "months", months, "EXPLAIN_SCHEDULE_EIC_CHILD_N_MONTHS");
    }
}

impl ScheduleEic {
    pub(crate) fn emit_credit(&self, r: &mut FederalResult) {
        form_line(
            r,
            FormLine::new(TaxForm::Form1040, "27a"),
            "earned_income_credit",
            self.credit,
            INPUTS,
            SOURCE,
            if self.another_child {
                msg!("EXPLAIN_EARNED_INCOME_CREDIT")
            } else {
                msg!("EXPLAIN_EARNED_INCOME_CREDIT_2")
            },
        );
    }
}

fn phase_out_start(children: usize, joint: bool) -> i64 {
    match (children == 0, joint) {
        (true, false) => 10860,
        (true, true) => 18140,
        (false, false) => 23890,
        (false, true) => 31160,
    }
}

// The whole-dollar AGI values at which the credit's slope in AGI changes for some child count and
// filing status: the last dollar of the table's maximum plateau, the last dollar with a credit and
// the first without. The scholarship election tries them (a staircase of $50 rows around a linear
// trend is best at the end of a linear stretch).
pub(crate) fn agi_kinks() -> alloc::vec::Vec<i64> {
    let mut kinks = alloc::vec::Vec::new();
    for children in 0..=3 {
        for joint in [false, true] {
            let maximum = [664, 4427, 7316, 8231][children];
            let mut row =
                crate::kernel::ratio::trunc_i64(phase_out_start(children, joint), 50) * 50;
            while row < 70250 && eic_table_2026(row, children, joint) == maximum {
                row += 50;
            }
            kinks.push(row - 1);
            while row < 70250 && eic_table_2026(row, children, joint) > 0 {
                row += 50;
            }
            kinks.push(row - 1);
            kinks.push(row);
        }
    }
    kinks.sort_unstable();
    kinks.dedup();
    kinks
}

pub(crate) fn eic_amount(earned: i64, agi: i64, children: usize, joint: bool) -> i64 {
    let earned_credit = eic_table_2026(earned, children, joint);
    if earned_credit == 0 {
        return 0;
    }
    // The EIC worksheet first looks up earned income. It looks up AGI only when
    // AGI reaches the applicable phase-out threshold, and takes the lesser
    // table credit. A low AGI in the phase-in range must not reduce the credit.
    if agi < phase_out_start(children, joint) {
        earned_credit
    } else {
        phase::cap(earned_credit, [eic_table_2026(agi, children, joint)])
    }
}

// The year names this published table, not the return selected by a caller.
pub(crate) fn eic_table_2026(income: i64, children: usize, joint: bool) -> i64 {
    // Draft IRS Publication 1040 (2026), pp. 16-27, as published 2026-09-16:
    // https://www.irs.gov/pub/irs-dft/p1040--dft.pdf
    // All 11,232 ordinary cells match the $50 interval midpoint formula below.
    // The eight maximum-straddling cells and eight starred cutoff cells are
    // literal published exceptions; see the exhaustive fixture test.
    if !(1..70250).contains(&income) {
        return 0;
    }
    let n = min(children, 3);
    let row = (crate::kernel::ratio::trunc_i64(income, 50)) * 50;
    let row = if row == 0 { 1 } else { row };
    let starred = match (row, n, joint) {
        (19500, 0, false) => Some((19540, 2)),
        (26800, 0, true) => Some((26820, 1)),
        (51550, 1, false) => Some((51593, 3)),
        (58600, 2, false) => Some((58629, 3)),
        (58850, 1, true) => Some((58863, 1)),
        (62950, 3, false) => Some((62974, 2)),
        (65850, 2, true) => Some((65899, 5)),
        (70200, 3, true) => Some((70244, 5)),
        _ => None,
    };
    if let Some((cutoff, credit)) = starred {
        return if income < cutoff { credit } else { 0 };
    }
    let maximum = [664, 4427, 7316, 8231][n];
    // The first published interval is $1-$49; later intervals are $50 wide.
    let midpoint_twice = row + if row == 1 { 50 } else { row + 50 };
    let phase_in = phase::cap(
        maximum,
        [crate::kernel::ratio::half_offset_i64(midpoint_twice * [765, 3400, 4000, 4500][n], 20000)],
    );
    let start = match (n == 0, joint) {
        (true, false) => 10860,
        (true, true) => 18140,
        (false, false) => 23890,
        (false, true) => 31160,
    };
    let phased_numerator = positive(
        maximum * 20000 - positive(midpoint_twice - 2 * start) * [765, 1598, 2106, 2106][n],
    );
    let phase_out = crate::kernel::ratio::half_offset_i64(phased_numerator, 20000);
    let ordinary = phase::cap(phase_in, [phase_out]);
    match (row, n, joint) {
        (10850, 0, false) => 664,
        (18250, 2, _) => 7316,
        (18250, 3, _) => 8231,
        (31150, 1, true) => 4427,
        (31150, 2, true) => 7316,
        (31150, 3, true) => 8231,
        _ => ordinary,
    }
}
