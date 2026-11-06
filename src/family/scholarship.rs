// Coverage: Scholarships and fellowships (Schedule 1 line 8r) — Supported (unit B6b, 2026-10-09;
//   Pub. 970 chapter 1 and Worksheet 1-1, Pub. 590-A, 26 U.S.C. 117 and 219(f)(1)).
//   Lines: the Other income page's Scholarship or fellowship kind (ScholarshipGrant): the award
//   less the part a degree candidate spent on qualified expenses (Worksheet 1-1 line 9, which
//   equals received less qualified_paid because that spending never exceeds the award less its
//   restricted and service parts); a non-candidate's whole award; required services paid by the
//   award (PaidServices amount) are taxable, the named NHSC, Armed Forces and work-college
//   programs are not service payments; a graduate or postdoctoral stipend is also IRA
//   compensation. Taxable scholarship is earned for the dependent standard deduction and not
//   EIC earned income. Exclusions: foreign and treaty exclusions and tuition reductions
//   (ordinary No applies the ordinary rules with a note). Unit B11 KIDS
//   (2026-10-11): the allocation election with an education credit is calculated on the
//   Education page (ScholarshipInclusion on StudentMore / FederalEducationExpenses): the
//   included part of the 1098-T box 5 grant is added to Schedule 1 line 8r of a filer or
//   spouse student by federal_rules.rs (education_N_scholarship_included), noted for a
//   dependent student, and `candidates` lists the Automatic choices.
//   Evidence: this module, federal_rules.rs; src/tests/income/relief_income.rs g_* (fixture G: 10,000 award,
//   5,000 taxable, tax 5,623 and 5,383), src/tests/family/family.rs
//   named_scholarship_service_programs_preserve_only_qualified_expense_exclusion.
use crate::kernel::phase;
pub use crate::scholarship_source::{EducationAward, Scholarship, ScholarshipService};
use crate::{EducationMethod, ScholarshipInclusion};
use alloc::{vec, vec::Vec};

// The parts of an Automatic entry's grant worth trying (Pub. 970 chapter 2,
// 2026 draft i8863 "Coordination with Pell grants"): the credit is piecewise
// linear in the adjusted qualified expenses (the American opportunity credit
// is 100% of the first half of its $4,000 and 25% of the second half, the
// lifetime learning credit 20% of up to $10,000), so the best part is none,
// the part that brings the expenses to a breakpoint, or the most the living
// costs and the grant allow. In cents, ascending, without duplicates; empty
// for an entry that is not Automatic.
pub(crate) fn candidates(entry: &crate::federal_model::EducationAnnualProjection) -> Vec<i64> {
    let ScholarshipInclusion::Automatic { living_costs } = entry.expenses.scholarship_in_income
    else {
        return Vec::new();
    };
    let most = phase::floor(phase::cap(entry.expenses.assistance, [living_costs]), 0);
    let mut plain = entry.expenses.clone();
    plain.scholarship_in_income = ScholarshipInclusion::None;
    let base = plain.qualified(entry.method());
    let cap = match entry.method() {
        EducationMethod::AmericanOpportunity => {
            crate::year_constants::AMERICAN_OPPORTUNITY_EXPENSE_CAP_DOLLARS
        }
        EducationMethod::LifetimeLearning => {
            crate::year_constants::LIFETIME_LEARNING_EXPENSE_CAP_DOLLARS
        }
    } * 100;
    let mut parts = vec![0, most];
    if entry.method() == EducationMethod::AmericanOpportunity {
        parts.push(phase::bound(crate::kernel::ratio::trunc_i64(cap, 2) - base, 0, most));
    }
    parts.push(phase::bound(cap - base, 0, most));
    parts.sort_unstable();
    parts.dedup();
    parts
}

// The most of the grant an Automatic entry can include, in cents: the grant, within the living
// costs.
pub(crate) fn most(entry: &crate::federal_model::EducationAnnualProjection) -> i64 {
    match entry.expenses.scholarship_in_income {
        ScholarshipInclusion::Automatic { living_costs } => {
            phase::floor(phase::cap(entry.expenses.assistance, [living_costs]), 0)
        }
        _ => 0,
    }
}

// The parts at which the rest of the return's total tax less refundable credits changes slope
// or jumps, from the return with none of the grant included (`base`): where the earned income
// credit table's maximum plateau and phase-out end, where taxable income turns positive (the
// deduction is used up), and where it crosses an ordinary bracket edge. The credit's own
// breakpoints are `candidates`. In cents, within (0, most).
pub(crate) fn return_kinks(
    status: crate::FederalStatus,
    base: &crate::FederalResult,
    most: i64,
) -> Vec<i64> {
    let amount = |id: &str| base.lines.iter().find(|l| l.id == id).map(|l| l.amount);
    let (Some(agi), Some(taxable)) = (amount("agi"), amount("taxable_income")) else {
        return Vec::new();
    };
    let mut parts: Vec<i64> = crate::forms::schedule_eic::agi_kinks()
        .into_iter()
        .map(|dollars| dollars * 100 - agi)
        .collect();
    // Taxable income grows one for one with the included part once the deduction is used up.
    let mut income = taxable;
    if taxable == 0
        && let Some(deduction) = amount("deduction")
    {
        let used_up = phase::floor(deduction - agi, 0);
        parts.push(used_up);
        income = -used_up;
    }
    if let Some(rates) = crate::income_tax::Rates::for_year(2026, status) {
        parts.extend(rates.ordinary.iter().map(|top| top * 100 - income));
    }
    parts.retain(|part| (1..most).contains(part));
    parts.sort_unstable();
    parts.dedup();
    parts
}
