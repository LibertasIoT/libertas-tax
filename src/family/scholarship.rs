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
use crate::{EducationMethod, FederalEducation, ScholarshipInclusion};
use alloc::{vec, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Required scholarship services
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
pub enum ScholarshipService {
    /// No required teaching, research or other services
    None,
    /// National Health Service Corps Scholarship Program
    /// Confirm the award is under this specific program, not a loan repayment or other health-service program.
    NationalHealthServiceCorps,
    /// Armed Forces Health Professions Scholarship and Financial Assistance Program
    /// Confirm the named program. Do not include military wages or duplicate amounts reported on Form W-2.
    ArmedForcesHealthProfessions,
    /// Qualifying work-college work-learning-service program
    /// Confirm a comprehensive program operated by a work college under section 448(e) of the Higher Education Act. Ordinary work-study employment is not this exception.
    WorkCollege,
    /// Teaching, research or other services required for the award
    /// The part that pays for those services is taxable. Leave out any part already on your W-2.
    PaidServices {
        /// Part of the award that pays for the required services
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        amount: i64,
    },
}

/// Education scholarship or fellowship
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct EducationAward {
    /// Total award received
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub received: i64,
    /// Award restricted to room, board or other nonqualified costs
    /// Use the award's terms, not total living expenses. Zero for an unrestricted award.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub restricted: i64,
    /// Award funds spent on required tuition, fees, books, supplies and equipment
    /// Exclude room, board, travel, optional equipment, costs paid with other tax-free funds and expenses assigned to a credit or other tax benefit. Do not include funds restricted to nonqualified costs. Enter what the award actually paid toward tuition; a part you choose to treat as paying living costs so the education credit can use more of the tuition is chosen on the Education page and added to your income there, not here.
    #[libertas_money("USD", 2)]
    #[libertas_number(min = 0)]
    pub qualified_paid: i64,
    /// Candidate for a degree at an eligible educational institution
    /// Check your enrollment records or Form 1098-T. No taxes the whole award.
    pub degree_candidate: bool,
    /// Services required by the award
    /// The three named programs waive only the service restriction; degree and qualified-spending rules still apply.
    pub services: ScholarshipService,
    /// No other special exclusion
    /// No qualifying graduate/postdoctoral non-tuition stipend. Use the dedicated stipend choice for that award. No applies the ordinary rules without any special exclusion.
    /// Check the award letter and its conditions for any special exclusion.
    pub ordinary: bool,
}

/// Scholarship or fellowship not reported on Form W-2
/// Enter award and spending records. Do not duplicate wages. Tuition reductions, service awards outside the listed exceptions and foreign/treaty exclusions require separate review. A part of a grant you include in income to raise an education credit is chosen on the Education page.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum Scholarship {
    /// Education scholarship or fellowship
    Education {
        /// Award and spending records
        data: EducationAward,
    },
    /// Graduate or postdoctoral non-tuition fellowship or stipend
    GraduateStipend {
        /// Non-tuition payment received
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        received: i64,
        /// Entire payment aids graduate or postdoctoral study and is an ordinary taxable non-tuition fellowship or stipend
        /// Not W-2 pay, a service-required award, a tax-free emergency grant or a treaty-exempt payment. Check the award letter and whether it was paid on a W-2. No taxes the payment without counting it as IRA compensation.
        eligible: bool,
    },
}
impl Scholarship {
    // The page's client rule, repeated on save: spending, the restricted part
    // and the service part together stay within the award.
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if !(0..=100_000_000_000).contains(&self.amount()) {
            let issue =
                ValidationIssue::new("SCHOLARSHIP_SPENDING_ABOVE_AWARD").at_field("received");
            return Some(if matches!(self, Self::Education { .. }) {
                issue.in_field("data")
            } else {
                issue
            });
        }
        if let Self::Education { data } = self {
            let service_part = match data.services {
                ScholarshipService::PaidServices { amount } => amount,
                _ => 0,
            };
            if data.restricted < 0 || data.restricted > data.received {
                return Some(
                    ValidationIssue::new("SCHOLARSHIP_SPENDING_ABOVE_AWARD")
                        .at_field("restricted")
                        .in_field("data"),
                );
            }
            if !(0..=100_000_000_000).contains(&service_part) {
                return Some(
                    ValidationIssue::new("SCHOLARSHIP_SPENDING_ABOVE_AWARD")
                        .at_field("amount")
                        .in_field("services")
                        .in_field("data"),
                );
            }
            if data.qualified_paid < 0
                || data.qualified_paid > data.received - data.restricted - service_part
            {
                return Some(
                    ValidationIssue::new("SCHOLARSHIP_SPENDING_ABOVE_AWARD")
                        .at_field("qualified_paid")
                        .in_field("data"),
                );
            }
        }
        None
    }
    pub(crate) fn amount(&self) -> i64 {
        match self {
            Self::Education { data } => data.received,
            Self::GraduateStipend { received, .. } => *received,
        }
    }
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    // The taxable amount, the part that is IRA compensation, and the answer
    // given to `ordinary` (No applies the ordinary rules; the
    // caller states which). Pub. 970 Worksheet 1-1: the award less the part
    // spent on qualified expenses by a degree candidate. Qualified spending
    // never exceeds the award less its restricted and service parts (valid),
    // so the taxable amount equals line 9 for every input.
    pub(crate) fn taxable(&self) -> (i64, i64, bool) {
        match self {
            Self::Education { data } => (
                data.received - if data.degree_candidate { data.qualified_paid } else { 0 },
                0,
                data.ordinary,
            ),
            Self::GraduateStipend { received, eligible } => {
                (*received, if *eligible { *received } else { 0 }, true)
            }
        }
    }
}

// The parts of an Automatic entry's grant worth trying (Pub. 970 chapter 2,
// 2026 draft i8863 "Coordination with Pell grants"): the credit is piecewise
// linear in the adjusted qualified expenses (the American opportunity credit
// is 100% of the first half of its $4,000 and 25% of the second half, the
// lifetime learning credit 20% of up to $10,000), so the best part is none,
// the part that brings the expenses to a breakpoint, or the most the living
// costs and the grant allow. In cents, ascending, without duplicates; empty
// for an entry that is not Automatic.
pub(crate) fn candidates(entry: &FederalEducation) -> Vec<i64> {
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
pub(crate) fn most(entry: &FederalEducation) -> i64 {
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
