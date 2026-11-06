// Coverage: Form7206 source and deduction owner.
// Complete monthly/LTC editor facts are retained under the actual Business,
// PartnerK1 or ScorpK1; stable income IDs identify plans, and an S-corp plan
// selects an existing same-owner W-2 for its Medicare-wage limit. No wage or
// K-1 income is added here. Ordinary and LTC employer eligibility are separate;
// child-only nonmedical remainders never enter ScheduleA. One insured-person
// age pool is shared across business, personal and public-safety LTC premiums.
// An entity plan without qualifying payment/income reporting has no162(l)
// capacity; independently eligible medical premiums remain available.
// Source: Form7206/i7206 (2026 draft numbering), Pub502, Notice2008-1,
// Pub974 and RevProc2025-32. Ordinary calculations preserve their whole-
// dollar worksheet inputs; Iterative work retains admitted source cents.
// Linked1099-G/PATR farm income reuses farm_routed_income. Partner and
// optional-method projections come from their existing source owners; the
// fixed filed Schedule1line15 total is allocated in cents, not recomputed.
// Tests: iterative_health_limit_reuses_linked_farm_income_in_source_cents,
// iterative_health_partner_source_and_filed_half_attribution_keep_cents,
// iterative_health_optional_attribution_copies_filed_totals_in_cents.
// Existing named limits remain for Form2555 plan
// coordination and owner retirement plans other than supported SEP. Ministers
// reuse clergy expense addbacks and the resolved business-only half-SE share;
// common-law employee pay/housing does not supply a Form7206 trade limit.
// Canonical Marketplace assignments now supply specified premiums once by
// stable statement/month/activity identity. Source fractions and monthly PTC
// allocation remain exact until one shared whole-dollar form projection.
// The legacy monthly Marketplace checkbox is removed. Nonspecified canonical
// expenses use the same trade capacity. Actual child/nonchild cost and funding
// classes conserve each source through deduction and retained credit; missing
// medical attribution withholds that benefit. Broader annual-owner cent
// fitting and retirement coordination remain open in the owning audit; this
// checkpoint does not claim complete coupled deduction support.
// Evidence: health_accounts Form7206 fixtures, partner health fixtures and
// interview::scorp_health_plan_uses_actual_wage_limit_and_survives_k1_resave.

pub use crate::medical_care_source::{CareContractQualification, MedicalCarePremium};
pub(crate) use crate::medical_care_source::{
    medical_care_issue, medical_expenses_issue, medical_source_amount,
};

use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{Exact, dollar_allocations, whole};
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::line;
use crate::marketplace::CalculationMoney;
use crate::{FederalDraft, FederalIncome, FederalResult};
use alloc::{collections::BTreeMap, format, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Actual payment of premiums under an entity-established plan
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]
pub enum EntityPremiumPayment {
    /// The entity paid the insurer and reported the premiums in your existing taxable income
    /// The existing K-1 guaranteed payments or W-2 wages include these premiums. This answer does not add another income amount.
    EntityPaid,
    /// The entity reimbursed premiums you paid
    Reimbursed {
        /// Actual reimbursement included in these premiums
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        amount: i64,
        /// Reimbursement was included in the existing K-1 guaranteed payments or W-2 wages
        /// Check the existing K-1 guaranteed-payment statement or W-2 box 1 reporting. This does not add another income amount.
        included_in_income: bool,
    },
    /// Your own-name policy was not paid or reimbursed by the entity
    /// These premiums are not established under that entity for this deduction.
    IndividualUnreimbursed,
    /// You paid a policy in the partnership's name
    /// The partnership established the policy in its own name and you paid the insurer. This partnership-only choice does not assert an own-name policy, require a fabricated reimbursement, or add guaranteed-payment income.
    PartnershipPolicyYouPaid,
}

/// Business under which this health plan was established
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]

pub enum HealthEstablishment {
    /// Sole proprietor plan, including a policy in your own name
    SoleProprietor,
    /// Partnership plan
    Partnership { payment: EntityPremiumPayment },
    /// Plan of an S corporation in which you own more than 2 percent
    /// Entity-paid or reimbursed premiums must already be reported in existing W-2 box1 wages. This plan does not add them again.
    Scorporation {
        payment: EntityPremiumPayment,
        /// Actual same-owner W-2 from this corporation
        #[libertas_number(min = 1)]
        wage: i64,
    },
}

/// Business under which this health plan was established
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    LibertasAvroEncode,
    LibertasAvroDecode,
    LibertasExport,
    libertas_macros::VariantIndex,
)]

pub enum HealthEstablishmentPage {
    /// Sole proprietor plan, including a policy in your own name
    SoleProprietor,
    /// Partnership plan
    Partnership { payment: EntityPremiumPayment },
    /// Plan of an S corporation in which you own more than 2 percent
    /// Entity-paid or reimbursed premiums must already be reported in existing W-2 box1 wages. This plan does not add them again.
    Scorporation {
        payment: EntityPremiumPayment,
        /// Actual W-2 from this corporation for the K-1 owner
        /// Select this corporation's existing wage statement. Medicare wages supply the earned-income limit; no wages are added here.
        #[libertas_number(min = 1)]
        #[libertas_enum_source("$.health_wages")]
        wage: i64,
    },
}
impl From<HealthEstablishmentPage> for HealthEstablishment {
    fn from(v: HealthEstablishmentPage) -> Self {
        match v {
            HealthEstablishmentPage::SoleProprietor => Self::SoleProprietor,
            HealthEstablishmentPage::Partnership { payment } => Self::Partnership { payment },
            HealthEstablishmentPage::Scorporation { payment, wage } => {
                Self::Scorporation { payment, wage }
            }
        }
    }
}
impl From<HealthEstablishment> for HealthEstablishmentPage {
    fn from(v: HealthEstablishment) -> Self {
        match v {
            HealthEstablishment::SoleProprietor => Self::SoleProprietor,
            HealthEstablishment::Partnership { payment } => Self::Partnership { payment },
            HealthEstablishment::Scorporation { payment, wage } => {
                Self::Scorporation { payment, wage }
            }
        }
    }
}
impl HealthEstablishment {
    pub(crate) fn included(&self) -> bool {
        match self {
            Self::SoleProprietor => true,
            Self::Partnership { payment } | Self::Scorporation { payment, .. } => match payment {
                EntityPremiumPayment::EntityPaid
                | EntityPremiumPayment::PartnershipPolicyYouPaid => true,
                EntityPremiumPayment::Reimbursed { included_in_income, .. } => *included_in_income,
                EntityPremiumPayment::IndividualUnreimbursed => false,
            },
        }
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        match self {
            Self::SoleProprietor => None,
            Self::Partnership { payment } | Self::Scorporation { payment, .. } => {
                if matches!(payment, EntityPremiumPayment::Reimbursed { amount, .. } if *amount < 0)
                {
                    return Some(
                        ValidationIssue::new("PROBLEM_INVALID_HEALTH_PLAN_SOURCE")
                            .at_field("amount")
                            .in_field("payment"),
                    );
                }
                if matches!(self, Self::Scorporation { wage, .. } if *wage <= 0) {
                    return Some(
                        ValidationIssue::new("PROBLEM_INVALID_HEALTH_PLAN_SOURCE").at_field("wage"),
                    );
                }
                matches!(
                    self,
                    Self::Scorporation {
                        payment: EntityPremiumPayment::PartnershipPolicyYouPaid,
                        ..
                    }
                )
                .then_some(
                    ValidationIssue::new("HEALTH_PARTNERSHIP_POLICY_ON_CORPORATION")
                        .at_field("payment"),
                )
            }
        }
    }
}

/// Actual entity reimbursement attributable to this ordinary coverage month
/// This is a funded subset of the existing paid premium, not another expense.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct ReimbursedHealthPremiums {
    /// Included ordinary premiums actually reimbursed by this entity
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub premiums: i64,
    /// Included reimbursement of the nonmedical child premium increase
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub child_increase: i64,
}

/// Health coverage month
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct BusinessHealthMonth {
    /// Month in 2026
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// After-tax premiums paid for this month's coverage
    /// Include medical, dental, vision and voluntary Medicare premiums paid in 2026. Exclude Marketplace premiums: select their saved Form 1095-A in the policy assignments instead. Exclude tax-free reimbursements, HSA/FSA payments and tax-free retired public-safety distributions. Do not enter these premiums elsewhere. For an assigned Marketplace month with no other premiums, enter zero here to retain its actual employer eligibility.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub premiums: i64,
    /// Included premium increase for a child under 27 who does not qualify for medical itemization
    /// This is part of the total above, not an additional premium. Enter only the increase attributable to adding your son, daughter, stepchild, adopted child or legally placed foster child who is under 27 at December 31, 2026 and is not a medical-expense dependent. Enter zero if adding the child did not increase family premiums. Amounts not used by the self-employed deduction cannot enter Schedule A. Medical-expense dependents include qualifying children of divorced parents and dependents allowed under the special medical-expense rules, even if not claimed on your return.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default(0)]
    pub child_increase: i64,
    /// Actual entity reimbursement of this month's included premiums
    /// Complete only for an entity Reimbursed plan. For a partial reimbursement, identify the actual funded ordinary and child-increase portions, including an explicit zero when none of this month's premium was reimbursed. No proportional allocation is inferred.
    #[libertas_default($null)]
    pub reimbursed: Option<ReimbursedHealthPremiums>,
    /// Eligible for employer-subsidized health coverage during any part of this month
    /// Include eligibility through your employer, your spouse's employer, or the employer of your dependent or child under 27, even if you declined enrollment. Check the employer’s benefit eligibility records for this month.
    pub employer_eligible: bool,
}

/// Qualified long-term-care coverage month
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]

pub struct BusinessCareMonth {
    /// Month in 2026
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// After-tax premium paid for this coverage
    /// Exclude tax-free reimbursements and payments funded with tax-free HSA/FSA or public-safety retirement amounts. Do not repeat these premiums in ordinary health coverage or itemized medical expenses.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    pub premiums: i64,
    /// Included premium actually reimbursed by this entity
    /// Complete only for an entity Reimbursed plan. A partial plan reimbursement requires the actual funded amount for this contract and month, including zero. Do not repeat it as another premium or income amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 0, max = 100000000000)]
    #[libertas_default($null)]
    pub reimbursed: Option<i64>,
    /// Eligible for employer-subsidized long-term-care insurance during this month
    /// Consider your employer and your spouse's employer, even if you declined enrollment. This is eligibility for long-term-care coverage, not ordinary medical insurance. Check the employer’s long-term-care benefit eligibility records for this month.
    pub employer_eligible: bool,
}

/// One insured person's long-term-care contracts with the same qualification facts
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessCarePremium {
    /// Insured person
    /// The person's identity on the people page: -1 for you, -2 for your spouse, otherwise the dependent's identity. Combine only contracts with the same qualification facts, or enter separate rows for this person. The annual age limit is shared with this person's other long-term-care records, found by this identity and the age at December 31, 2026 from the people page. Allocate each paid premium dollar to only one source; do not repeat public-safety premiums here. Enter personal premiums in Deductions; do not leave policies sharing this limit unentered.
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_default(0)]
    pub person: i64,
    /// Paid coverage months
    /// Combine contracts with the same qualification facts for each month; never enter the same paid premium twice. Actual deductions consume the shared age limit in taxpayer then spouse business-entry order; unused capacity remains available to later businesses.
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 1, max = 12)]
    pub months: Vec<BusinessCareMonth>,
    /// Actual contract qualification
    /// Nonqualified contracts receive no qualified-LTC premium benefit; known income and other independent source facts remain retained.
    pub qualified: CareContractQualification,
}
/// One insured person's long-term-care premiums under this plan
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct BusinessCarePremiumPage {
    /// Insured person
    #[libertas_enum_source("$.health_people")]
    pub person: i64,
    /// Actual paid months
    // Tax structure: at most one eligibility or coverage entry per calendar month.
    #[libertas_size(min = 1, max = 12)]
    #[libertas_unordered]
    pub months: Vec<BusinessCareMonth>,
    /// Actual contract qualification
    /// Nonqualified contracts receive no qualified-LTC premium benefit; known income and other independent source facts remain retained.
    pub qualified: CareContractQualification,
}
impl From<BusinessCarePremiumPage> for BusinessCarePremium {
    fn from(v: BusinessCarePremiumPage) -> Self {
        Self { person: v.person, months: v.months, qualified: v.qualified }
    }
}
impl From<BusinessCarePremium> for BusinessCarePremiumPage {
    fn from(v: BusinessCarePremium) -> Self {
        Self { person: v.person, months: v.months, qualified: v.qualified }
    }
}
impl BusinessCarePremium {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let invalid =
            |field| ValidationIssue::new("PROBLEM_INVALID_HEALTH_PLAN_SOURCE").at_field(field);
        if self.person == 0 {
            return Some(invalid("person"));
        }
        if !self.qualified.valid() {
            return Some(invalid("qualified"));
        }
        if self.months.is_empty() {
            return Some(invalid("months"));
        }
        for (index, month) in self.months.iter().enumerate() {
            if let Some(issue) = month.issue() {
                return Some(issue.in_list("months", index));
            }
            if self.months[..index].iter().any(|v| v.month == month.month) {
                return Some(invalid("month").in_list("months", index));
            }
        }
        None
    }
}

// Rev. Proc. 2025-32 section 4.27, source cents.
pub(crate) fn care_limit(age: u32) -> i64 {
    match age {
        0..=40 => crate::year_constants::LONG_TERM_CARE_PREMIUM_LIMITS_2026_CENTS[0],
        41..=50 => crate::year_constants::LONG_TERM_CARE_PREMIUM_LIMITS_2026_CENTS[1],
        51..=60 => crate::year_constants::LONG_TERM_CARE_PREMIUM_LIMITS_2026_CENTS[2],
        61..=70 => crate::year_constants::LONG_TERM_CARE_PREMIUM_LIMITS_2026_CENTS[3],
        _ => crate::year_constants::LONG_TERM_CARE_PREMIUM_LIMITS_2026_CENTS[4],
    }
}

// The age at the end of 2026 that sets a person's premium limit: the person's
// own on the people page, one source for every coverage record. A record that
// names someone no longer on the return is refused.
fn care_age(
    d: &FederalDraft,
    r: &mut FederalResult,
    person: i64,
    context: &crate::IssueContext,
) -> u32 {
    let age = crate::interview::person_age(d, person);
    crate::require_localized!(
        r,
        age.is_some(),
        context,
        "Long-term-care insurance",
        "ISSUE_A_LONG_TERM_CARE_PREMIUM_NAMES_SOMEONE_WHO_IS_NOT_ON_THIS_RETURN",
        "A long-term-care premium names someone who is not on this return. Choose the insured person from the people on the return.",
    );
    age.unwrap_or(0)
}
// The insured person's label for an issue.
fn care_label(d: &FederalDraft, person: i64) -> Vec<u8> {
    crate::federal_rules::people_students(d)
        .into_iter()
        .find(|p| p.id == person)
        .map(|p| p.label)
        .unwrap_or_default()
}

#[derive(Default)]
struct CarePool {
    age: u32,
    medical_paid: i64,
    medical_used: i64,
    paid: i64,
    used: i64,
}

/// Business owner's health insurance
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessHealthInsurance {
    /// None
    None,
    /// Premiums paid
    Plan {
        /// Ordinary health coverage months
        /// Enter each month once, combining policies established under this business. Never assign the same premiums to another business or spouse.
        // Tax structure: at most one eligibility or coverage entry per calendar month.
        #[libertas_size(max = 12)]
        months: Vec<BusinessHealthMonth>,
        /// Qualified long-term-care policies
        /// Enter separately from ordinary health months; employer eligibility and per-person age caps differ. Ordinary coverage months may be empty when only long-term-care premiums were paid.
        long_term_care: Vec<BusinessCarePremium>,
        /// Actual plan establishment and payment
        establishment: HealthEstablishment,
    },
}
impl BusinessHealthInsurance {
    pub(crate) fn valid(&self) -> bool {
        self.issue().is_none()
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let Self::Plan { months, long_term_care, establishment } = self else {
            return None;
        };
        if let Some(issue) = establishment.issue() {
            return Some(issue.in_field("establishment"));
        }
        for (index, care) in long_term_care.iter().enumerate() {
            if let Some(issue) = care.issue() {
                return Some(issue.in_list("long_term_care", index));
            }
        }
        for (index, month) in months.iter().enumerate() {
            if let Some(issue) = month.issue() {
                return Some(issue.in_list("months", index));
            }
            if months[..index].iter().any(|v| v.month == month.month) {
                return Some(
                    ValidationIssue::new("PROBLEM_INVALID_HEALTH_PLAN_SOURCE")
                        .at_field("month")
                        .in_list("months", index),
                );
            }
        }
        None
    }
}

#[derive(Clone, Copy)]
enum ReimbursementMode {
    All,
    None,
    Actual,
}

// Reconcile the existing entity payment to its actual premium subsets. A full
// reimbursement is determinate without extra allocation facts; a partial one
// is never spread by list order or a guessed proportion.
fn reimbursement_mode(
    d: &FederalDraft,
    activity: i64,
    plan: &BusinessHealthInsurance,
) -> Option<ReimbursementMode> {
    let BusinessHealthInsurance::Plan { establishment, months, long_term_care } = plan else {
        return Some(ReimbursementMode::None);
    };
    let payment = match establishment {
        HealthEstablishment::SoleProprietor => return Some(ReimbursementMode::All),
        HealthEstablishment::Partnership { payment }
        | HealthEstablishment::Scorporation { payment, .. } => payment,
    };
    let EntityPremiumPayment::Reimbursed { amount, .. } = payment else {
        return Some(
            if matches!(
                payment,
                EntityPremiumPayment::EntityPaid | EntityPremiumPayment::PartnershipPolicyYouPaid
            ) {
                ReimbursementMode::All
            } else {
                ReimbursementMode::None
            },
        );
    };
    let assignments: Vec<_> = d
        .adjustments
        .as_ref()
        .into_iter()
        .flat_map(|a| &a.health_marketplace.policies)
        .filter(|a| a.activity == activity)
        .collect();
    let total: Exact = months
        .iter()
        .map(|m| whole(m.premiums))
        .chain(long_term_care.iter().flat_map(|p| &p.months).map(|m| whole(m.premiums)))
        .sum();
    let no_details = months.iter().all(|m| m.reimbursed.is_none())
        && long_term_care.iter().flat_map(|p| &p.months).all(|m| m.reimbursed.is_none())
        && assignments.iter().all(|a| a.reimbursed.is_none());
    if no_details && *amount == 0 {
        return Some(ReimbursementMode::None);
    }
    if no_details {
        let full = if assignments.is_empty() {
            Some(total)
        } else {
            let (sources, issues) =
                d.adjustments.as_ref().unwrap().health_marketplace.source_premiums(d);
            (issues.is_empty()
                && assignments.iter().all(|a| {
                    sources
                        .iter()
                        .any(|s| (s.policy, s.month, s.activity) == (a.policy, a.month, a.activity))
                }))
            .then(|| {
                total
                    + sources
                        .iter()
                        .filter(|s| s.activity == activity)
                        .map(|s| &s.paid)
                        .sum::<crate::kernel::ratio::exact::Exact>()
            })
        };
        if full.is_some_and(|v| v == crate::kernel::ratio::exact::whole(*amount)) {
            return Some(ReimbursementMode::All);
        }
    }
    if months.iter().any(|m| m.premiums > 0 && m.reimbursed.is_none())
        || long_term_care
            .iter()
            .flat_map(|p| &p.months)
            .any(|m| m.premiums > 0 && m.reimbursed.is_none())
        || assignments.iter().any(|a| a.reimbursed.is_none())
    {
        return None;
    }
    let assigned: Exact = months
        .iter()
        .filter_map(|m| m.reimbursed.as_ref())
        .map(|v| whole(v.premiums))
        .chain(
            long_term_care.iter().flat_map(|p| &p.months).filter_map(|m| m.reimbursed).map(whole),
        )
        .chain(assignments.iter().filter_map(|a| a.reimbursed.as_ref()).map(|v| whole(v.premiums)))
        .sum();
    (assigned == whole(*amount)).then_some(ReimbursementMode::Actual)
}

fn funded_health(month: &BusinessHealthMonth, mode: ReimbursementMode) -> BusinessHealthMonth {
    let (premiums, child_increase) = match mode {
        ReimbursementMode::All => (month.premiums, month.child_increase),
        ReimbursementMode::None => (0, 0),
        ReimbursementMode::Actual => {
            month.reimbursed.as_ref().map_or((0, 0), |v| (v.premiums, v.child_increase))
        }
    };
    BusinessHealthMonth { premiums, child_increase, ..month.clone() }
}
fn funded_care(month: &BusinessCareMonth, mode: ReimbursementMode) -> i64 {
    match mode {
        ReimbursementMode::All => month.premiums,
        ReimbursementMode::None => 0,
        ReimbursementMode::Actual => month.reimbursed.unwrap_or(0),
    }
}

// Profit/SE/SEP are whole-dollar worksheet amounts; premiums are source cents.
#[cfg(test)]
pub(crate) fn amounts(
    months: &[BusinessHealthMonth],
    profit: i64,
    profits: i64,
    half_se: i64,
    sep: i64,
) -> (i64, i64) {
    let allocated = if profits == 0 { 0 } else { fraction(half_se, profit, profits) };
    amounts_allocated(months, profit, allocated, sep)
}

#[cfg(test)]
fn amounts_allocated(
    months: &[BusinessHealthMonth],
    profit: i64,
    allocated: i64,
    sep: i64,
) -> (i64, i64) {
    amounts_allocated_money(months, profit, allocated, sep, CalculationMoney::Dollars)
}

fn amounts_allocated_money(
    months: &[BusinessHealthMonth],
    profit: i64,
    allocated: i64,
    sep: i64,
    money: CalculationMoney,
) -> (i64, i64) {
    let premiums = money.units(&whole(months.iter().map(|m| m.premiums).sum()));
    let available = money
        .units(&whole(months.iter().filter(|m| !m.employer_eligible).map(|m| m.premiums).sum()));
    let deduction = phase::cap(available, [positive(profit - allocated - sep)]);
    // Pub. 502's nondependent-child examples allocate the adjustment to
    // the incremental child premium first. Only otherwise medical-eligible
    // costs may flow to Schedule A; excluded employer months remain excluded
    // from the above-the-line allocation.
    let child = money.units(&whole(months.iter().map(|m| m.child_increase).sum()));
    let eligible_child = money.units(&whole(
        months.iter().filter(|m| !m.employer_eligible).map(|m| m.child_increase).sum(),
    ));
    (deduction, positive(premiums - child - deduction + phase::cap(deduction, [eligible_child])))
}

// One transient source projection for each actual established plan. Source
// cents and relative business parts remain exact until the shared form total.
pub(crate) struct SpecifiedPremiums {
    pub activity: i64,
    pub sources: Vec<crate::health_marketplace::HealthSourcePremium>,
    pub funding: BTreeMap<(i64, u32), Exact>,
    // WorksheetP usage on these same policy/month expenses, before W/Step3.
    pub nonspecified: BTreeMap<(i64, u32), Exact>,
    // WorksheetP/Form7206 continues only if its actual trade limit remains.
    pub continuing: bool,
    pub premiums: Exact,
    pub maximum: Exact,
    pub deduction: Exact,
}
// Allocate the same month's retained credit without rounding source cents.
fn source_credit(premium_share: &Exact, column: Option<&(u32, Exact, Exact)>) -> Exact {
    column
        .filter(|(_, premium, _)| *premium > whole(0))
        .map_or_else(|| whole(0), |(_, premium, credit)| premium_share * credit / premium)
}

impl SpecifiedPremiums {
    fn available_sources(
        &self,
        outcome: &crate::marketplace::MarketplaceOutcome,
    ) -> Vec<((i64, u32), Exact)> {
        self.sources
            .iter()
            .map(|source| {
                let credit = source_credit(
                    &source.credit_premium,
                    outcome.expense_columns.iter().find(|(month, _, _)| *month == source.month),
                );
                // Actual payment by an entity belongs to this source policy/month.
                // Its funded cost cannot migrate to an unreimbursed second policy.
                // Continuing Step3/5 subtraction has no zero floor. A
                // negative worksheet component offsets only this source's
                // selected WorksheetP amount before the final deduction.
                // WorksheetP's exhausted-trade stop retains its own result.
                let floor = if self.continuing {
                    -self
                        .nonspecified
                        .get(&(source.policy, source.month))
                        .cloned()
                        .unwrap_or_else(|| whole(0))
                } else {
                    whole(0)
                };
                let available = phase::cap(
                    self.funding[&(source.policy, source.month)].clone(),
                    [phase::floor(floor, &source.specified - credit)],
                );
                ((source.policy, source.month), available)
            })
            .collect()
    }

    fn minimum(&self) -> Exact {
        if self.continuing { -self.nonspecified.values().sum::<Exact>() } else { whole(0) }
    }

    pub(crate) fn after_credit(&self, outcome: &crate::marketplace::MarketplaceOutcome) -> Exact {
        let net = self.available_sources(outcome).into_iter().map(|(_, v)| v).sum();
        phase::cap(self.maximum.clone(), [net])
    }

    // Retain the actual selected deduction on its source expenses. For the
    // all-medical endpoint, proportional attribution among remaining funded
    // costs cannot change the combined deduction or medical remainder. Mixed
    // medical/SE-only classes require their separate source treatment instead.
    fn used_sources(
        &self,
        outcome: &crate::marketplace::MarketplaceOutcome,
    ) -> Vec<((i64, u32), Exact)> {
        let available = self.available_sources(outcome);
        let negative: Exact = available.iter().filter(|(_, v)| *v < whole(0)).map(|(_, v)| v).sum();
        let positive: Exact = available.iter().filter(|(_, v)| *v > whole(0)).map(|(_, v)| v).sum();
        // Simplified may retain a smaller Step3 component than the final
        // Step4 credit permits. Attribute that actual smaller deduction by
        // reducing the remaining nonspecified costs after required offsets;
        // do not fabricate additional medical usage from the later credit.
        if self.deduction < negative {
            let ordinary: Exact = self.nonspecified.values().sum();
            let remaining = &ordinary + &negative;
            let used = phase::floor(whole(0), ordinary + &self.deduction);
            return available
                .into_iter()
                .map(|(source, amount)| {
                    let ordinary =
                        self.nonspecified.get(&source).cloned().unwrap_or_else(|| whole(0));
                    let available = &ordinary + phase::cap(whole(0), [amount]);
                    let used = if remaining > whole(0) {
                        available * &used / &remaining
                    } else {
                        whole(0)
                    };
                    (source, used - ordinary)
                })
                .collect();
        }
        // Fully offset the selected nonspecified usage on a credited source
        // first. Scaling its negative component with unrelated positive
        // expenses would resurrect a duplicate source deduction.
        let used =
            phase::cap(positive.clone(), [phase::floor(whole(0), &self.deduction - &negative)]);
        available
            .into_iter()
            .map(|(source, amount)| {
                let amount = if amount <= whole(0) || positive == whole(0) {
                    amount
                } else {
                    amount * &used / &positive
                };
                (source, amount)
            })
            .collect()
    }
}
#[derive(Default)]
pub(crate) struct HealthResult {
    pub deduction: i64,
    pub medical: i64, // Cents, aggregated with other medical expenses before rounding.
    pub by_business: BTreeMap<i64, i64>,
    pub specified: Vec<SpecifiedPremiums>,
    pub nonspecified: BTreeMap<i64, Exact>,
    manual_work: BTreeMap<i64, Exact>,
}

impl HealthResult {
    pub(crate) fn work_deduction(&self) -> Exact {
        self.manual_work.values().sum::<Exact>() + self.canonical_deduction()
    }
    pub(crate) fn canonical_deduction(&self) -> Exact {
        self.nonspecified.values().sum::<Exact>()
            + self.specified.iter().map(|v| &v.deduction).sum::<Exact>()
    }
    pub(crate) fn marketplace_usage_valid(
        &self,
        outcome: &crate::marketplace::MarketplaceOutcome,
    ) -> bool {
        if self.specified.iter().any(|plan| {
            plan.deduction < plan.minimum() || plan.deduction > plan.after_credit(outcome)
        }) {
            return false;
        }
        let mut premiums = BTreeMap::<u32, Exact>::new();
        let mut paid = BTreeMap::<(i64, u32), Exact>::new();
        let mut credit_sources = BTreeMap::<(i64, u32), Exact>::new();
        let mut used = BTreeMap::<(i64, u32), Exact>::new();
        for plan in &self.specified {
            for (source, amount) in &plan.nonspecified {
                *used.entry(*source).or_insert_with(|| whole(0)) += amount;
            }
            for source in &plan.sources {
                *premiums.entry(source.month).or_insert_with(|| whole(0)) += &source.credit_premium;
                *paid.entry((source.policy, source.month)).or_insert_with(|| whole(0)) +=
                    &source.paid;
                *credit_sources.entry((source.policy, source.month)).or_insert_with(|| whole(0)) +=
                    &source.credit_premium;
            }
            let allocated = plan.used_sources(outcome);
            if allocated.iter().map(|(_, amount)| amount).sum::<Exact>() != plan.deduction {
                return false;
            }
            for (source, amount) in allocated {
                *used.entry(source).or_insert_with(|| whole(0)) += amount;
            }
        }
        used.iter().all(|(source, deduction)| {
            if *deduction < whole(0) {
                return false;
            }
            if *deduction == whole(0) {
                return true;
            }
            let credit = outcome
                .expense_columns
                .iter()
                .find(|(month, premium, _)| *month == source.1 && *premium > whole(0))
                .map_or_else(
                    || whole(0),
                    |column| source_credit(&credit_sources[source], Some(column)),
                );
            deduction + credit <= paid[source]
        }) && outcome.expense_columns.iter().all(|(month, premium, credit)| {
            *credit == whole(0) || premiums.get(month).is_none_or(|assigned| assigned <= premium)
        })
    }

    // Use one source expense projection after the selected PTC/health pair.
    // A benchmark allocation is not evidence of a nonmedical child's actual
    // added premium. Until that source is present, omit that policy's medical
    // benefit, while preserving its independent mandatory reconciliation.
    pub(crate) fn marketplace_medical(
        &self,
        d: &FederalDraft,
        outcome: &crate::marketplace::MarketplaceOutcome,
        r: &mut FederalResult,
    ) -> Exact {
        let Some(facts) =
            d.credits.as_ref().and_then(|c| crate::marketplace::facts(&c.marketplace))
        else {
            return whole(0);
        };
        let eligibility = person_eligibility(d);
        let family = crate::marketplace::family_people(d);
        let (current, births) = crate::marketplace::current_people(d);
        let people = crate::marketplace_projection::SourcePeople {
            family: &family,
            current: &current,
            births: &births,
        };
        let mut used = BTreeMap::<((i64, u32), i64), Exact>::new();
        for plan in &self.specified {
            for (source, amount) in &plan.nonspecified {
                *used.entry((*source, plan.activity)).or_insert_with(|| whole(0)) += amount;
            }
            for (source, amount) in plan.used_sources(outcome) {
                *used.entry((source, plan.activity)).or_insert_with(|| whole(0)) += amount;
            }
        }
        let mut medical = whole(0);
        for policy in &facts.forms {
            for month in 1..=12 {
                let Ok(Some(source)) = crate::marketplace_projection::medical_policy_month(
                    facts, &people, policy, month,
                ) else {
                    // The mandatory PTC owner already reports incomplete
                    // payment/family sources. An expense is never fabricated.
                    continue;
                };
                if source.paid == whole(0) {
                    continue;
                }
                let class = policy_medical_class(
                    &source,
                    policy.medical_costs.iter().find(|c| c.month == month),
                    &eligibility,
                );
                let PolicyMedicalClass::Known { medical: paid_medical, child } = class else {
                    crate::federal_rules::note_once_scoped(
                        r,
                        msg!("HEALTH_POLICY_MEDICAL_ALLOCATION_REQUIRED"),
                        crate::IssueContext::Section(crate::TaxSection::Marketplace)
                            .corrective(crate::TaxSection::Medical),
                    );
                    continue;
                };
                let credit = source_credit(
                    &source.credit_premium,
                    outcome
                        .expense_columns
                        .iter()
                        .find(|(m, premium, _)| *m == month && *premium > whole(0)),
                );
                let mut medical_used = whole(0);
                let mut funding_known = true;
                for (((policy_id, used_month), activity), deduction) in &used {
                    if *policy_id != policy.id || *used_month != month || *deduction == whole(0) {
                        continue;
                    }
                    let child_funded = if child == whole(0) {
                        Some(whole(0))
                    } else {
                        d.income
                            .iter()
                            .find(|e| e.id == *activity)
                            .and_then(|e| health_plan(&e.income))
                            .and_then(|plan| reimbursement_mode(d, *activity, plan))
                            .and_then(|funding| match funding {
                                ReimbursementMode::All => self
                                    .specified
                                    .iter()
                                    .find(|p| p.activity == *activity)
                                    .and_then(|p| {
                                        p.sources
                                            .iter()
                                            .find(|s| s.policy == policy.id && s.month == month)
                                    })
                                    .map(|s| &child * &s.paid / &source.paid),
                                ReimbursementMode::None => Some(whole(0)),
                                ReimbursementMode::Actual => d
                                    .adjustments
                                    .as_ref()
                                    .and_then(|a| {
                                        a.health_marketplace.policies.iter().find(|p| {
                                            p.policy == policy.id
                                                && p.month == month
                                                && p.activity == *activity
                                        })
                                    })
                                    .and_then(|p| p.reimbursed.as_ref())
                                    .and_then(|p| p.child)
                                    .map(whole),
                            })
                    };
                    let Some(child_funded) = child_funded else {
                        funding_known = false;
                        break;
                    };
                    medical_used += phase::floor(whole(0), deduction - child_funded);
                }
                if !funding_known {
                    crate::federal_rules::note_once_scoped(
                        r,
                        msg!("HEALTH_POLICY_MEDICAL_ALLOCATION_REQUIRED"),
                        crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits)
                            .corrective(crate::TaxSection::Marketplace)
                            .corrective(crate::TaxSection::Medical),
                    );
                    continue;
                }
                // Pub502 permits assigning the actual health deduction to
                // the nonmedical child's added cost first and separately
                // excludes premiums paid by retained PTC. Combining those
                // permissions here is our conservative source allocation,
                // not a published combined IRS worksheet. Tax-free funding
                // was already removed from both actual cost classes. Each
                // business consumes only its funded child subset above;
                // unused child cost never becomes a medical deduction.
                // If positive, D + credit + medical <= actual paid because
                // D's child allocation cannot exceed actual child cost. If
                // credit exhausts medical cost, the medical benefit is zero;
                // do not move credit or unused cost onto another source.
                medical += phase::floor(whole(0), paid_medical - credit - medical_used);
            }
        }
        medical
    }
}

fn health_plan(income: &FederalIncome) -> Option<&BusinessHealthInsurance> {
    match income {
        FederalIncome::Business { data } => Some(&data.health_insurance),
        FederalIncome::Partnership { data } => Some(&data.health_insurance),
        FederalIncome::Scorp { data } => Some(&data.personal.health_insurance),
        _ => None,
    }
}

// Source admission includes accepted multi-activity plans and their real wage
// links. Financial allowance owners still require their separate source feed;
// this view does not enroll graph records in health_plan's calculation inputs.
pub(crate) fn source_health_plan(income: &FederalIncome) -> Option<&BusinessHealthInsurance> {
    match income {
        FederalIncome::PartnershipActivities { data } => data.health_insurance.as_ref(),
        FederalIncome::ScorpActivities { data } => data.health_insurance.as_ref(),
        _ => health_plan(income),
    }
}

fn establishment_of(plan: &BusinessHealthInsurance) -> Option<&HealthEstablishment> {
    match plan {
        BusinessHealthInsurance::Plan { establishment, .. } => Some(establishment),
        BusinessHealthInsurance::None => None,
    }
}

// Source links are authorized from accepted income/person records, never the
// editor's echoed catalogs. The selected corporation is the actual K-1 row;
// choosing its wage source asserts that this existing statement is from it.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PersonEligibility {
    pub medical: bool,
    pub self_employed: bool,
}

pub(crate) enum PolicyMedicalClass {
    Known { medical: Exact, child: Exact },
    Unverified,
    Invalid,
}

fn policy_medical_cost_issue(
    source: &crate::marketplace_projection::MedicalPolicyPremium,
    cost: Option<&crate::MarketplaceMedicalCostMonth>,
) -> Option<crate::validation::ValidationIssue> {
    let gross = &source.paid + whole(source.tax_free);
    let child = cost.and_then(|c| c.child_increase);
    let funded = cost.and_then(|c| c.tax_free_child);
    let invalid = |field| {
        crate::validation::ValidationIssue::new(
            "ISSUE_INTERVIEW_REVIEW_THE_ACCEPTED_PAGE_VALUES_BEFORE_65B6D7B7",
        )
        .at_field(field)
    };
    if child.is_some_and(|c| c < 0 || whole(c) > gross) {
        Some(invalid("child_increase"))
    } else if funded.is_some_and(|f| f < 0 || f > source.tax_free || child.is_none_or(|c| f > c))
        || child.zip(funded).is_some_and(|(c, f)| whole(source.tax_free - f) > &gross - whole(c))
    {
        Some(invalid("tax_free_child"))
    } else {
        None
    }
}

pub(crate) fn policy_medical_class(
    source: &crate::marketplace_projection::MedicalPolicyPremium,
    cost: Option<&crate::MarketplaceMedicalCostMonth>,
    eligibility: &BTreeMap<i64, PersonEligibility>,
) -> PolicyMedicalClass {
    let child = cost.and_then(|c| c.child_increase);
    let funded = cost.and_then(|c| c.tax_free_child);
    if policy_medical_cost_issue(source, cost).is_some() {
        return PolicyMedicalClass::Invalid;
    }
    let has_child = source
        .people
        .iter()
        .any(|id| eligibility.get(id).is_some_and(|p| !p.medical && p.self_employed));
    // People corrections can make the same insured child medical-eligible.
    // Preserve the actual cost/funding source; its nonmedical classification
    // is then inapplicable, not a reason to block the People editor.
    if source
        .people
        .iter()
        .any(|id| !eligibility.get(id).is_some_and(|p| p.medical || p.self_employed))
    {
        return PolicyMedicalClass::Unverified;
    }
    let child = if has_child {
        let Some(child) = child else {
            return PolicyMedicalClass::Unverified;
        };
        let funded = if source.tax_free == 0 || child == 0 {
            0
        } else {
            let Some(funded) = funded else {
                return PolicyMedicalClass::Unverified;
            };
            funded
        };
        whole(child - funded)
    } else {
        whole(0)
    };
    PolicyMedicalClass::Known { medical: &source.paid - &child, child }
}

pub(crate) fn policy_costs_issue(
    facts: &crate::MarketplaceFacts,
    people: &crate::marketplace_projection::SourcePeople<'_>,
    policy: &crate::MarketplacePolicyRecord,
) -> Option<crate::validation::ValidationIssue> {
    for (cost_index, cost) in policy.medical_costs.iter().enumerate() {
        if let Ok(Some(source)) =
            crate::marketplace_projection::medical_policy_month(facts, people, policy, cost.month)
            && let Some(issue) = policy_medical_cost_issue(&source, Some(cost))
        {
            return Some(issue.in_list("medical_costs", cost_index));
        }
    }
    None
}

pub(crate) fn assigned_child_capacity(
    d: &FederalDraft,
    source: &crate::health_marketplace::HealthSourcePremium,
) -> Option<Exact> {
    let facts = d.credits.as_ref().and_then(|c| crate::marketplace::facts(&c.marketplace))?;
    let policy = facts.forms.iter().find(|p| p.id == source.policy)?;
    let eligibility = person_eligibility(d);
    let family = crate::marketplace::family_people(d);
    let (current, births) = crate::marketplace::current_people(d);
    let people = crate::marketplace_projection::SourcePeople {
        family: &family,
        current: &current,
        births: &births,
    };
    let payment =
        crate::marketplace_projection::medical_policy_month(facts, &people, policy, source.month)
            .ok()??;
    let PolicyMedicalClass::Known { child, .. } = policy_medical_class(
        &payment,
        policy.medical_costs.iter().find(|c| c.month == source.month),
        &eligibility,
    ) else {
        return None;
    };
    Some(if payment.paid == whole(0) { whole(0) } else { child * &source.paid / payment.paid })
}

// One People-owned source projection; age and relationship are never repeated
// on an LTC contract. Section162(l)'s child definition is152(f)(1), not every
// young person in the household. Section213 medical exceptions stay in the
// existing family owner.
pub(crate) fn person_eligibility(d: &FederalDraft) -> BTreeMap<i64, PersonEligibility> {
    let medical = crate::federal_rules::medical_dependent_ids(d);
    let self_employed = crate::federal_rules::self_employed_dependent_ids(d);
    let mut result = BTreeMap::new();
    if let Some(people) = &d.people {
        result.insert(-1, PersonEligibility { medical: true, self_employed: true });
        if people.spouse.is_some() {
            result.insert(-2, PersonEligibility { medical: true, self_employed: true });
        }
    }
    result.extend(d.dependents().iter().map(|person| {
        (
            person.id,
            PersonEligibility {
                medical: medical.contains(&person.id),
                self_employed: self_employed.contains(&person.id)
                    || (matches!(
                        person.relationship,
                        crate::Relationship::Child | crate::Relationship::Foster
                    ) && crate::federal_model::age_at_year_end(person.birth_date) < 27),
            },
        )
    }));
    result
}

pub(crate) fn plans_valid(d: &FederalDraft) -> bool {
    plans_issue(d).is_none()
}
// Same source-context checks and order as plans_valid; source identity is
// transient diagnostic metadata, never an accepted-record or request index.
pub(crate) fn plans_issue(
    d: &FederalDraft,
) -> Option<(Option<i64>, crate::validation::ValidationIssue)> {
    use crate::validation::ValidationIssue;
    let invalid = || ValidationIssue::new("PROBLEM_INVALID_HEALTH_PLAN_SOURCE");
    let people = crate::federal_rules::people_students(d);
    if d.deductions
        .as_ref()
        .into_iter()
        .flat_map(|v| v.long_term_care())
        .any(|premium| !people.iter().any(|p| p.id == premium.person))
    {
        return Some((None, invalid()));
    }
    for entry in &d.income {
        let Some(plan @ BusinessHealthInsurance::Plan { establishment, long_term_care, .. }) =
            source_health_plan(&entry.income)
        else {
            continue;
        };
        let matching = match establishment {
            HealthEstablishment::SoleProprietor => matches!(entry.income, FederalIncome::Business { .. }),
            HealthEstablishment::Partnership { .. } => matches!(entry.income, FederalIncome::Partnership { .. } | FederalIncome::PartnershipActivities { .. }),
            HealthEstablishment::Scorporation { wage, .. } => {
                matches!(entry.income, FederalIncome::Scorp { .. } | FederalIncome::ScorpActivities { .. })
                    && d.income.iter().any(|source| source.id == *wage && source.owner == entry.owner && matches!(source.income, FederalIncome::Wage { .. }))
                    && !d.income.iter().any(|other| other.id != entry.id && matches!(
                        source_health_plan(&other.income),
                        Some(BusinessHealthInsurance::Plan { establishment: HealthEstablishment::Scorporation { wage: other_wage, .. }, .. }) if other_wage == wage))
            }
        };
        if !matching {
            let issue = if matches!(establishment, HealthEstablishment::Scorporation { .. })
                && matches!(
                    entry.income,
                    FederalIncome::Scorp { .. } | FederalIncome::ScorpActivities { .. }
                ) {
                invalid().at_field("wage").in_field("establishment")
            } else {
                invalid().at_field("establishment")
            };
            return Some((Some(entry.id), issue));
        }
        if let Some(issue) = plan.issue() {
            // Keep this context owner's existing key; local plan admission has
            // already returned its own more specific key on its editable Page.
            return Some((
                Some(entry.id),
                ValidationIssue { key: invalid().key, field: issue.field },
            ));
        }
        if reimbursement_mode(d, entry.id, plan).is_none() {
            // Aggregate assignment may involve ordinary/LTC/Marketplace rows;
            // the whole submitted plan is the real governing record.
            return Some((Some(entry.id), invalid().at_record()));
        }
        if let Some(index) =
            long_term_care.iter().position(|p| !people.iter().any(|v| v.id == p.person))
        {
            return Some((
                Some(entry.id),
                invalid().at_field("person").in_list("long_term_care", index),
            ));
        }
    }
    None
}

// Keep selected-source failures distinct from actual health carrier failures.
#[derive(Debug)]
pub(crate) enum CalculationIssue {
    Prepared(crate::Section179PreparedIssue),
    Source(crate::IssueContext),
}
impl From<crate::Section179PreparedIssue> for CalculationIssue {
    fn from(issue: crate::Section179PreparedIssue) -> Self {
        Self::Prepared(issue)
    }
}
impl CalculationIssue {
    pub(crate) fn source(context: crate::IssueContext) -> Self {
        Self::Source(context)
    }
    pub(crate) fn report(self, result: &mut FederalResult) {
        match self {
            Self::Prepared(issue) => crate::section179_return::Issue::from(issue).report(result),
            Self::Source(context) => {
                crate::issue_localized!(
                    result,
                    &context,
                    "Health premium source",
                    "HEALTH_SOURCE_AMOUNTS_UNAVAILABLE",
                    "The combined health premium source amounts exceed the supported arithmetic range.",
                );
                result.lines.clear();
                result.return_lines.clear();
            }
        }
        result.state = crate::ResultState::Unsupported;
    }
}
pub(crate) fn source_amount(
    amount: Option<i64>,
    context: crate::IssueContext,
) -> Result<i64, CalculationIssue> {
    amount.ok_or_else(|| CalculationIssue::source(context))
}

// Schedule A owns one projection after all actual Cents/Exact components join.
// Widen each component before addition, then enforce the actual Dollar carrier.
pub(crate) fn medical_total(
    components: &[i64],
    marketplace: &Exact,
) -> Result<i64, CalculationIssue> {
    let total = components.iter().map(|amount| whole(*amount)).sum::<Exact>() + marketplace;
    source_amount(
        crate::kernel::ratio::exact::dollars(&total),
        crate::IssueContext::Section(crate::TaxSection::Medical),
    )
}

// Form 7206, lines 4–8: allocate each owner's deductible SE tax over
// profitable businesses only. Losses affect SE tax but not this denominator.
#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(crate) fn calculate(
    d: &FederalDraft,
    half: [i64; 2],
    half_by_activity: Option<&BTreeMap<i64, i64>>,
    optional_earnings: &BTreeMap<i64, i64>,
    sep: &crate::sep::SepResult,
    public_safety: &crate::public_safety::PublicSafetyResult,
    specified_override: Option<&BTreeMap<i64, Exact>>,
    r: &mut FederalResult,
) -> HealthResult {
    let source_claims = crate::related_forms::effective_claimants(d);
    calculate_with_claims(
        d,
        half,
        half_by_activity,
        optional_earnings,
        sep,
        public_safety,
        specified_override,
        r,
        &source_claims,
    )
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(crate) fn calculate_with_claims(
    d: &FederalDraft,
    half: [i64; 2],
    half_by_activity: Option<&BTreeMap<i64, i64>>,
    optional_earnings: &BTreeMap<i64, i64>,
    sep: &crate::sep::SepResult,
    public_safety: &crate::public_safety::PublicSafetyResult,
    specified_override: Option<&BTreeMap<i64, Exact>>,
    r: &mut FederalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> HealthResult {
    match calculate_with_money_with_claims(
        d,
        half,
        half_by_activity,
        optional_earnings,
        sep,
        public_safety,
        specified_override,
        CalculationMoney::Dollars,
        None,
        r,
        None,
        source_claims,
    ) {
        Ok(result) => result,
        Err(issue) => panic!("known health source projection: {issue:?}"),
    }
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(crate) fn calculate_cents(
    d: &FederalDraft,
    half: [i64; 2],
    half_by_activity: Option<&BTreeMap<i64, i64>>,
    optional_earnings: &BTreeMap<i64, i64>,
    sep: &crate::sep::SepResult,
    public_safety: &crate::public_safety::PublicSafetyResult,
    specified_override: Option<&BTreeMap<i64, Exact>>,
    r: &mut FederalResult,
) -> HealthResult {
    let source_claims = crate::related_forms::effective_claimants(d);
    calculate_cents_with_claims(
        d,
        half,
        half_by_activity,
        optional_earnings,
        sep,
        public_safety,
        specified_override,
        r,
        &source_claims,
    )
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub(crate) fn calculate_cents_with_claims(
    d: &FederalDraft,
    half: [i64; 2],
    half_by_activity: Option<&BTreeMap<i64, i64>>,
    optional_earnings: &BTreeMap<i64, i64>,
    sep: &crate::sep::SepResult,
    public_safety: &crate::public_safety::PublicSafetyResult,
    specified_override: Option<&BTreeMap<i64, Exact>>,
    r: &mut FederalResult,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> HealthResult {
    match calculate_with_money_with_claims(
        d,
        half,
        half_by_activity,
        optional_earnings,
        sep,
        public_safety,
        specified_override,
        CalculationMoney::Cents,
        None,
        r,
        None,
        source_claims,
    ) {
        Ok(result) => result,
        Err(issue) => panic!("known health source projection: {issue:?}"),
    }
}

#[cfg(test)]
#[allow(clippy::too_many_arguments)]
pub(crate) fn calculate_with_money(
    d: &FederalDraft,
    half: [i64; 2],
    half_by_activity: Option<&BTreeMap<i64, i64>>,
    optional_earnings: &BTreeMap<i64, i64>,
    sep: &crate::sep::SepResult,
    public_safety: &crate::public_safety::PublicSafetyResult,
    specified_override: Option<&BTreeMap<i64, Exact>>,
    money: CalculationMoney,
    selected: Option<&crate::Section179PreparedAllocation>,
    r: &mut FederalResult,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<HealthResult, CalculationIssue> {
    let source_claims = crate::related_forms::effective_claimants(d);
    calculate_with_money_with_claims(
        d,
        half,
        half_by_activity,
        optional_earnings,
        sep,
        public_safety,
        specified_override,
        money,
        selected,
        r,
        home,
        &source_claims,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn calculate_with_money_with_claims(
    d: &FederalDraft,
    half: [i64; 2],
    half_by_activity: Option<&BTreeMap<i64, i64>>,
    optional_earnings: &BTreeMap<i64, i64>,
    sep: &crate::sep::SepResult,
    public_safety: &crate::public_safety::PublicSafetyResult,
    specified_override: Option<&BTreeMap<i64, Exact>>,
    money: CalculationMoney,
    selected: Option<&crate::Section179PreparedAllocation>,
    r: &mut FederalResult,
    home: Option<&crate::home_office::HomeContext>,
    source_claims: &alloc::collections::BTreeMap<i64, i64>,
) -> Result<HealthResult, CalculationIssue> {
    let mut result = HealthResult::default();
    let (canonical, source_issues) =
        d.adjustments.as_ref().map(|a| a.health_marketplace.source_premiums(d)).unwrap_or_default();
    for source in &canonical {
        if d.income
            .iter()
            .find(|e| e.id == source.activity)
            .and_then(|e| health_plan(&e.income))
            .is_none_or(|p| matches!(p, BusinessHealthInsurance::None))
        {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("HEALTH_POLICY_PLAN_MONTH_REQUIRED"),
                crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits),
            );
        }
    }
    for issue in source_issues {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new(issue.missing.resource(), alloc::vec![]),
            crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits)
                .alternatives(&[crate::TaxSection::Marketplace]),
        );
    }
    let eligibility = person_eligibility(d);
    // The shared age limits, one pool for each insured person by identity; the
    // age is the person's own on the people page (F3, 2026-10-09).
    let mut care = BTreeMap::<i64, CarePool>::new();
    for (person, paid) in &public_safety.long_term {
        // Section 402(l) excluded dollars have already been removed. Keep the
        // unexcluded premiums in the shared age pool, not a second deduction.
        care.insert(
            *person,
            CarePool {
                age: care_age(
                    d,
                    r,
                    *person,
                    &crate::IssueContext::Section(crate::TaxSection::Retirement),
                ),
                paid: *paid,
                medical_paid: *paid,
                ..CarePool::default()
            },
        );
    }
    for p in d.deductions.as_ref().into_iter().flat_map(|ded| ded.long_term_care()).filter(|p| {
        p.qualified.qualified() && eligibility.get(&p.person).is_some_and(|v| v.medical)
    }) {
        // Optional benefit eligibility never removes the source premium facts.
        let age =
            care_age(d, r, p.person, &crate::IssueContext::Section(crate::TaxSection::Medical));
        let pool = care.entry(p.person).or_insert(CarePool { age, ..CarePool::default() });
        let context = crate::IssueContext::Section(crate::TaxSection::Medical);
        pool.paid = source_amount(pool.paid.checked_add(p.premiums), context.clone())?;
        pool.medical_paid = source_amount(pool.medical_paid.checked_add(p.premiums), context)?;
    }
    for e in &d.income {
        if let FederalIncome::Partnership { data } = &e.income
            && let Err(error) = data.health_payment_check()
        {
            let (key, _english) = error.message();
            #[cfg(test)]
            let english = _english;

            crate::require_localized!(
                r,
                false,
                &crate::interview::income_issue_context(e).alternatives(&[
                    crate::TaxSection::SelfEmployedBenefits,
                    crate::TaxSection::PassThrough,
                ]),
                "Partner health insurance",
                key,
                english,
            );
        }
        let Some(BusinessHealthInsurance::Plan { long_term_care, .. }) = health_plan(&e.income)
        else {
            continue;
        };
        for (premium_index, p) in long_term_care.iter().enumerate().filter(|(_, p)| {
            p.qualified.qualified()
                && eligibility.get(&p.person).is_some_and(|v| v.medical || v.self_employed)
        }) {
            let source = crate::interview::income_issue_context(e);
            let context = crate::IssueContext::Record {
                section: crate::TaxSection::SelfEmployedBenefits,
                identity: source.item("long_term_care", premium_index).identity(),
                label: crate::Message::new(
                    "NOTE_WITH_LABEL",
                    alloc::vec![
                        crate::MessageArg::Nested(alloc::boxed::Box::new(source.label())),
                        crate::MessageArg::Encoded(care_label(d, p.person)),
                    ],
                ),
            }
            .manual();
            crate::require_localized!(
                r,
                // Ineligible premiums were left out before the rules ran.
                eligibility.get(&p.person).is_some_and(|v| v.medical || v.self_employed),
                &context,
                "Long-term-care insurance",
                "ISSUE_CONFIRM_QUALIFIED_COVERAGE_ELIGIBLE_FAMILY_RELATIONSHIP_MEDICAL_ELI_40924044",
                "Confirm qualified coverage, eligible family relationship, medical eligibility and each month's employer long-term-care eligibility.",
            );
            let age = care_age(d, r, p.person, &context);
            let pool = care.entry(p.person).or_insert(CarePool { age, ..CarePool::default() });
            let paid = source_amount(
                crate::kernel::ratio::checked_sum(p.months.iter().map(|month| month.premiums)),
                context.clone(),
            )?;
            pool.paid = source_amount(pool.paid.checked_add(paid), context.clone())?;

            if eligibility.get(&p.person).is_some_and(|v| v.medical) {
                pool.medical_paid = source_amount(pool.medical_paid.checked_add(paid), context)?;
            }
        }
    }
    // Work in the selected worksheet units. Actual Schedule C/F profit and
    // S-corp Medicare wages already have admitted source-cent owners, so do
    // not round those limits before Iterative work. Partner/optional earnings
    // and half-SE deductions still come from their annual owners. The selected
    // earnings/attribution feed has the same units as this worksheet; no
    // second optional earnings or Schedule1line15 calculation occurs here.
    for (owner, half_se) in half.iter().enumerate() {
        let entries: Vec<_> = d.income.iter()
            .filter(|e| !(d.setup.as_ref().is_some_and(crate::community::applies)
                && e.owner == crate::DocumentOwner::Spouses && matches!(e.income, FederalIncome::Partnership { .. })))
            .filter(|e| usize::from(e.owner == crate::DocumentOwner::Spouses) == owner)
            .filter_map(|e| {
                let plan = health_plan(&e.income)?;
                let profit = match &e.income {
                    FederalIncome::Business { data } => match optional_earnings.get(&e.id) {
                        Some(amount) => *amount,
                        None => {
                            let expenses = match crate::section179_return::expenses_with_claims(d, e.id, data, selected,home, source_claims).map(|expense| expense - selected.map_or(0, |allocation| allocation.ordinary_limit_adjustment(e.id))) {
                                Ok(expenses) => expenses,
                                Err(issue) => return Some(Err(issue)),
                            };
                            let nondeductible = match crate::clergy::nondeductible_with_section179_with_claims(d, e.id, selected, home, source_claims) {
                                Ok(amount) => amount,
                                Err(issue) => return Some(Err(issue)),
                            };
                            let routed = match crate::federal_rules::farm_routed_income_with_claims(d, owner, e.id, source_claims) {
                                Ok(amount) => amount,
                                Err(issue) => return Some(Err(issue)),
                            };
                            positive(money.units(&whole(data.earned_receipts() + routed - expenses + nondeductible)))
                        }
                    },
                    FederalIncome::Partnership { .. } => {
                        crate::require_localized!(r, matches!(plan, BusinessHealthInsurance::None) || optional_earnings.contains_key(&e.id), &crate::interview::income_issue_context(e).corrective(crate::TaxSection::PassThrough), "Partner health insurance", "ISSUE_RESOLVE_THIS_PARTNERSHIP_S_REGULAR_SELF_EMPLOYMENT_EARNINGS_BEFORE_CB3167F1", "Resolve this partnership's regular self-employment earnings before its health deduction.");
                        positive(optional_earnings.get(&e.id).copied().unwrap_or(0))
                    }
                    FederalIncome::Scorp { .. } => {
                        match plan {
                            BusinessHealthInsurance::Plan { establishment: HealthEstablishment::Scorporation { wage, .. }, .. } => d.income.iter().find_map(|source| {
                                if source.id != *wage || source.owner != e.owner { return None; }
                                match &source.income {
                                    FederalIncome::Wage { data } => Some(money.units(&whole(crate::FederalWage::from(data).medicare_wages))),
                                    _ => None,
                                }
                            }).unwrap_or(0),
                            _ => 0,
                        }
                    }
                    _ => unreachable!(),
                };
                Some(Ok((e.id, plan, profit, crate::interview::income_issue_context(e))))
            }).collect::<Result<Vec<_>, crate::Section179PreparedIssue>>()?;
        let profits: i64 = entries
            .iter()
            .filter(|(id, _, _, _)| {
                !d.income
                    .iter()
                    .any(|e| e.id == *id && matches!(e.income, FederalIncome::Scorp { .. }))
            })
            .map(|(_, _, profit, _)| *profit)
            .sum();
        // WorksheetW copies the same filed half-SE deduction. In cent work,
        // use the shared source allocator to preserve that total across the
        // actual profit ratios, including its final residual cent.
        let default_half_cents = (money.scale() == 100 && half_by_activity.is_none()).then(|| {
            crate::business_rules::half_se_allocation(
                *half_se * 100,
                entries
                    .iter()
                    .filter(|(id, _, _, _)| {
                        !d.income
                            .iter()
                            .any(|e| e.id == *id && matches!(e.income, FederalIncome::Scorp { .. }))
                    })
                    .map(|(id, _, profit, _)| (*id, *profit)),
            )
        });
        let half_by_activity = half_by_activity.or(default_half_cents.as_ref());
        for (id, plan, profit, source) in entries {
            let BusinessHealthInsurance::Plan { months, long_term_care, .. } = plan else {
                continue;
            };
            // Manual premiums exclude canonical Marketplace payments. The
            // saved policy assignments are the only source of those costs.
            let originals = months;
            let funding =
                reimbursement_mode(d, id, plan).expect("accepted reconciled entity payment");
            let adjusted: Vec<_> = originals.iter().map(|m| funded_health(m, funding)).collect();
            let unfunded_medical = money.units(&whole(originals.iter().map(|m| m.premiums).sum()))
                - money.units(&whole(originals.iter().map(|m| m.child_increase).sum()))
                - money.units(&whole(adjusted.iter().map(|m| m.premiums).sum()))
                + money.units(&whole(adjusted.iter().map(|m| m.child_increase).sum()));
            let months = &adjusted;

            let allocated =
                if matches!(establishment_of(plan), Some(HealthEstablishment::Scorporation { .. }))
                {
                    0
                } else {
                    half_by_activity.map_or_else(
                        || {
                            if profits == 0 {
                                0
                            } else {
                                fraction(*half_se * money.scale(), profit, profits)
                            }
                        },
                        |amounts| amounts.get(&id).copied().unwrap_or(0),
                    )
                };
            let limit = if establishment_of(plan).is_some_and(|v| v.included()) {
                let pension_cents = sep.by_business_cents.get(&id).copied().unwrap_or(0);
                positive(profit - allocated - money.units(&whole(pension_cents)))
            } else {
                // An own-name entity policy without qualifying payment, or
                // unconfirmed income reporting, has no section162(l) capacity.
                // Its independently eligible medical cost remains available.
                0
            };
            let child_available = money.units(&whole(
                months.iter().filter(|m| !m.employer_eligible).map(|m| m.child_increase).sum(),
            ));
            let mut capacity = positive(limit - phase::cap(child_available, [limit]));
            let mut care_deduction = 0;
            // Child-only premiums cannot become Schedule A expenses. Use those
            // first, then ordinary medical premiums, then medical-eligible LTC.
            for medical_eligible in [false, true] {
                if medical_eligible {
                    let ordinary =
                        amounts_allocated_money(months, limit, 0, care_deduction, money).0;
                    capacity = positive(limit - care_deduction - ordinary);
                }
                for p in long_term_care.iter().filter(|p| {
                    p.qualified.qualified()
                        && eligibility
                            .get(&p.person)
                            .is_some_and(|v| v.self_employed && v.medical == medical_eligible)
                }) {
                    let pool = care.get_mut(&p.person).unwrap();
                    let available = money.units(&whole(
                        p.months
                            .iter()
                            .filter(|m| !m.employer_eligible)
                            .map(|m| funded_care(m, funding))
                            .sum(),
                    ));
                    let used = phase::cap(
                        phase::cap(capacity, [available]),
                        [positive(money.units(&whole(care_limit(pool.age))) - pool.used)],
                    );
                    pool.used += used;
                    if medical_eligible {
                        pool.medical_used += used;
                    }
                    capacity = crate::kernel::carry::native_balance(capacity, used);
                    care_deduction += used;
                }
            }
            let (ordinary, medical) =
                amounts_allocated_money(months, limit, 0, care_deduction, money);
            let medical = positive(medical + unfunded_medical);
            let deduction = ordinary + care_deduction;
            // Pub974 W4–13 retains the actual established-trade limit.
            // Section162(l)(2)(A) does not let another trade's earnings fund
            // this plan. Shared W/Step3 totals are formed by the consumer.
            let sources: Vec<_> = canonical
                .iter()
                .filter(|s| s.activity == id)
                .filter(|s| originals.iter().any(|m| m.month == s.month && !m.employer_eligible))
                .cloned()
                .collect();
            if canonical
                .iter()
                .any(|s| s.activity == id && !originals.iter().any(|m| m.month == s.month))
            {
                crate::federal_rules::note_once_scoped(
                    r,
                    msg!("HEALTH_POLICY_PLAN_MONTH_REQUIRED"),
                    source.clone().corrective(crate::TaxSection::SelfEmployedBenefits),
                );
            }
            if !sources.is_empty() {
                let premiums: Exact = sources.iter().map(|s| &s.specified).sum();
                let capacity = money.cents(positive(limit - deduction));
                // Pub974 WorksheetP/Form7206 first consumes nonspecified
                // premiums, including actual mixed-coverage source costs.
                // These exact fractions share this trade's existing limit;
                // they never become another manually entered premium row.
                let ordinary_funding: Vec<_> = sources
                    .iter()
                    .map(|s| {
                        let available = match funding {
                            ReimbursementMode::All => &s.paid - &s.specified,
                            ReimbursementMode::None => whole(0),
                            ReimbursementMode::Actual => d
                                .adjustments
                                .as_ref()
                                .unwrap()
                                .health_marketplace
                                .policies
                                .iter()
                                .find(|a| {
                                    (a.policy, a.month, a.activity)
                                        == (s.policy, s.month, s.activity)
                                })
                                .and_then(|a| a.reimbursed.as_ref())
                                .map_or_else(|| whole(0), |v| whole(v.premiums - v.coverage)),
                        };
                        ((s.policy, s.month), available)
                    })
                    .collect();
                let ordinary_available: Exact =
                    ordinary_funding.iter().map(|(_, amount)| amount).sum();
                let ordinary = phase::cap(capacity.clone(), [ordinary_available.clone()]);
                let nonspecified = if ordinary_available > whole(0) {
                    ordinary_funding
                        .into_iter()
                        .map(|(source, amount)| (source, amount * &ordinary / &ordinary_available))
                        .collect()
                } else {
                    BTreeMap::new()
                };
                result.nonspecified.insert(id, ordinary.clone());
                let capacity = capacity - &ordinary;
                let continuing = capacity > whole(0);
                let minimum = if continuing { -ordinary } else { whole(0) };
                let funding: BTreeMap<_, _> = sources
                    .iter()
                    .map(|s| {
                        (
                            (s.policy, s.month),
                            match funding {
                                ReimbursementMode::All => s.specified.clone(),
                                ReimbursementMode::None => whole(0),
                                ReimbursementMode::Actual => whole(
                                    d.adjustments
                                        .as_ref()
                                        .unwrap()
                                        .health_marketplace
                                        .policies
                                        .iter()
                                        .find(|a| {
                                            (a.policy, a.month, a.activity)
                                                == (s.policy, s.month, s.activity)
                                        })
                                        .and_then(|a| a.reimbursed.as_ref())
                                        .map_or(0, |v| v.coverage),
                                ),
                            },
                        )
                    })
                    .collect();
                let funded: Exact = funding.values().sum();
                let maximum = phase::cap(premiums.clone(), [phase::cap(capacity, [funded])]);
                let specified = specified_override.and_then(|v| v.get(&id)).map_or_else(
                    || maximum.clone(),
                    |v| phase::floor(minimum.clone(), phase::cap(v.clone(), [maximum.clone()])),
                );
                result.specified.push(SpecifiedPremiums {
                    activity: id,
                    sources,
                    funding,
                    nonspecified,
                    continuing,
                    premiums,
                    maximum,
                    deduction: specified,
                });
            }
            if !long_term_care.is_empty() {
                line(
                    r,
                    &format!("business_{id}_long_term_care_deduction"),
                    money.dollars(care_deduction),
                    &["income.business.health_insurance"],
                    "https://www.irs.gov/instructions/i7206",
                    msg!("EXPLAIN_BUSINESS_N_LONG_TERM_CARE_DEDUCTION"),
                );
            }
            let excluded = positive(
                money.units(&whole(originals.iter().map(|m| m.premiums).sum()))
                    - ordinary
                    - medical,
            );
            if excluded > 0 {
                line(
                    r,
                    &format!("business_{id}_excluded_child_premiums"),
                    money.dollars(excluded),
                    &["income.business.health_insurance"],
                    "https://www.irs.gov/publications/p502",
                    msg!("EXPLAIN_BUSINESS_N_EXCLUDED_CHILD_PREMIUMS"),
                );
            }
            result.manual_work.insert(id, money.cents(deduction));
            result.medical = source_amount(
                result
                    .medical
                    .checked_add(crate::kernel::ratio::trunc_i64(medical * 100, money.scale())),
                source.clone(),
            )?;
            line(
                r,
                &format!("business_{id}_medical_premiums"),
                money.dollars(medical),
                &["income.business.health_insurance"],
                "https://www.irs.gov/instructions/i7206",
                msg!("EXPLAIN_BUSINESS_N_MEDICAL_PREMIUMS"),
            );
        }
    }
    let mut candidates: BTreeMap<_, _> =
        result.specified.iter().map(|v| (v.activity, v.deduction.clone())).collect();
    for (&activity, amount) in &result.nonspecified {
        *candidates.entry(activity).or_insert_with(|| whole(0)) += amount;
    }
    let allocated = dollar_allocations(&candidates);
    emit_lines(r, &mut (), &allocated, |(&activity, &value), _, _| {
        Some(calculation_row(
            None,
            &format!("business_{}_marketplace_health_insurance", activity),
            LineValue::MoneyDollars(value),
            &["income.business.health_insurance", "credits.marketplace"],
            "https://www.irs.gov/publications/p974",
            msg!("EXPLAIN_BUSINESS_N_MARKETPLACE_HEALTH_INSURANCE"),
        ))
    });
    for (&activity, amount) in &result.manual_work {
        *candidates.entry(activity).or_insert_with(|| whole(0)) += amount;
    }
    result.by_business = dollar_allocations(&candidates);
    result.deduction = result.by_business.values().sum();
    emit_lines(r, &mut (), &result.by_business, |(&id, &deduction), _, _| {
        Some(calculation_row(
            None,
            &format!("business_{id}_health_insurance"),
            LineValue::MoneyDollars(deduction),
            &["income.business.health_insurance", "self_employment_tax"],
            "https://www.irs.gov/instructions/i7206",
            msg!("EXPLAIN_BUSINESS_N_HEALTH_INSURANCE"),
        ))
    });
    for (index, (_, pool)) in care.iter().enumerate() {
        let eligible = dollars(phase::cap(pool.paid, [care_limit(pool.age)]));
        let medical_cents = phase::cap(
            positive(
                phase::cap(pool.paid, [care_limit(pool.age)])
                    - crate::kernel::ratio::trunc_i64(pool.used * 100, money.scale()),
            ),
            [positive(
                pool.medical_paid
                    - crate::kernel::ratio::trunc_i64(pool.medical_used * 100, money.scale()),
            )],
        );
        let medical = dollars(medical_cents);
        result.medical = source_amount(
            result.medical.checked_add(medical_cents),
            crate::IssueContext::Section(crate::TaxSection::Medical),
        )?;
        emit_lines(
            r,
            &mut (),
            [
                ("eligible", eligible, msg!("EXPLAIN_LONG_TERM_CARE_N_N_ELIGIBLE")),
                ("medical", medical, msg!("EXPLAIN_LONG_TERM_CARE_N_N_MEDICAL")),
                (
                    "excluded",
                    positive(dollars(pool.paid) - money.dollars(pool.used) - medical),
                    msg!("EXPLAIN_LONG_TERM_CARE_N_N_EXCLUDED"),
                ),
            ],
            |(suffix, amount, note), _, _| {
                Some(calculation_row(
                    None,
                    &format!("long_term_care_{}_{suffix}", index + 1),
                    LineValue::MoneyDollars(amount),
                    &[
                        "income.business.health_insurance",
                        "screening.public_safety",
                        "deductions.long_term_care",
                    ],
                    "https://www.irs.gov/irb/2025-45_IRB",
                    note,
                ))
            },
        );
    }
    Ok(result)
}

// Source-owned month relationships shared by stored-plan validity and Save adapters.
impl ReimbursedHealthPremiums {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if self.premiums < 0 {
            return Some(
                ValidationIssue::new("PROBLEM_INVALID_HEALTH_PLAN_SOURCE").at_field("premiums"),
            );
        }
        (!(0..=self.premiums).contains(&self.child_increase)).then_some(
            ValidationIssue::new("HEALTH_REIMBURSED_CHILD_ABOVE_TOTAL").at_field("child_increase"),
        )
    }
}
impl BusinessHealthMonth {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if !(1..=12).contains(&self.month) {
            return Some(
                ValidationIssue::new("PROBLEM_INVALID_HEALTH_PLAN_SOURCE").at_field("month"),
            );
        }
        // Arithmetic exclusion: amounts_allocated_money and unfunded_medical
        // still sum monthly native Cents and scale worksheet Dollars by 100.
        if !(0..=100000000000).contains(&self.premiums) {
            return Some(
                ValidationIssue::new("PROBLEM_INVALID_HEALTH_PLAN_SOURCE").at_field("premiums"),
            );
        }
        if !(0..=self.premiums).contains(&self.child_increase) {
            return Some(
                ValidationIssue::new("HEALTH_CHILD_ABOVE_PREMIUMS").at_field("child_increase"),
            );
        }
        if let Some(value) = &self.reimbursed {
            if !(0..=self.premiums).contains(&value.premiums) {
                return Some(
                    ValidationIssue::new("HEALTH_REIMBURSEMENT_ABOVE_SOURCE")
                        .at_field("premiums")
                        .in_field("reimbursed"),
                );
            }
            if let Some(issue) = value.issue() {
                return Some(issue.in_field("reimbursed"));
            }
            if value.child_increase > self.child_increase {
                return Some(
                    ValidationIssue::new("HEALTH_REIMBURSEMENT_ABOVE_SOURCE")
                        .at_field("child_increase")
                        .in_field("reimbursed"),
                );
            }
            if value.premiums - value.child_increase > self.premiums - self.child_increase {
                return Some(
                    ValidationIssue::new("HEALTH_REIMBURSEMENT_ABOVE_SOURCE")
                        .at_field("premiums")
                        .in_field("reimbursed"),
                );
            }
        }
        None
    }
}
impl BusinessCareMonth {
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        if !(1..=12).contains(&self.month) {
            return Some(
                ValidationIssue::new("PROBLEM_INVALID_HEALTH_PLAN_SOURCE").at_field("month"),
            );
        }
        // Arithmetic exclusion: amounts_allocated_money and unfunded_medical
        // still sum monthly native Cents and scale worksheet Dollars by 100.
        if !(0..=100000000000).contains(&self.premiums) {
            return Some(
                ValidationIssue::new("PROBLEM_INVALID_HEALTH_PLAN_SOURCE").at_field("premiums"),
            );
        }
        self.reimbursed.is_some_and(|value| !(0..=self.premiums).contains(&value)).then_some(
            ValidationIssue::new("HEALTH_REIMBURSEMENT_ABOVE_SOURCE").at_field("reimbursed"),
        )
    }
}

#[cfg(test)]
mod specified_source_tests {
    use super::*;
    use crate::health_marketplace::HealthSourcePremium;

    fn source(policy: i64, month: u32) -> HealthSourcePremium {
        HealthSourcePremium {
            policy,
            month,
            activity: 1,
            paid: whole(100000),
            specified: whole(100000),
            credit_premium: whole(100000),
            advance: whole(0),
        }
    }

    // Derived Notice2008-1/Pub974 boundary: entity cash reimbursement600
    // caps the NET source deduction1000−400=600. It does not replace the
    // gross enrollment premium1000 before subtracting the same credit.
    #[test]
    fn mixed_expense_classes_preserve_the_full_policy_credit_and_total_source_cost() {
        // Pub974(2025), pp47–49 WorksheetP and p53 Step3: a sole plan
        // subtracts full PTC, independently of the SLCSP expense split.
        // Derived source1000, specified2/3, nonspecified1/3, credit400:
        // specified deduction266.666..., nonspecified333.333..., total600.
        let mut actual = source(10, 1);
        actual.specified = whole(100000) * crate::kernel::ratio::exact::ratio(2, 3).unwrap();
        let nonspecified = &actual.paid - &actual.specified;
        let mut plan = SpecifiedPremiums {
            activity: 1,
            sources: alloc::vec![actual.clone()],
            funding: [((10, 1), actual.specified.clone())].into_iter().collect(),
            nonspecified: [((10, 1), nonspecified.clone())].into_iter().collect(),
            continuing: true,
            premiums: actual.specified.clone(),
            maximum: actual.specified.clone(),
            deduction: whole(0),
        };
        let outcome = crate::marketplace::MarketplaceOutcome {
            expense_columns: alloc::vec![(1, whole(100000), whole(40000))],
            ..Default::default()
        };
        plan.deduction = plan.after_credit(&outcome);
        assert_eq!(&plan.deduction + &nonspecified, whole(60000));
        let mut health = HealthResult {
            nonspecified: [(1, nonspecified.clone())].into_iter().collect(),
            specified: alloc::vec![plan],
            ..Default::default()
        };
        assert_eq!(health.canonical_deduction() + whole(40000), actual.paid);
        assert!(health.marketplace_usage_valid(&outcome));
        // No per-class clipping can certify an extra source cent.
        *health.specified[0].nonspecified.get_mut(&(10, 1)).unwrap() += whole(1);
        assert!(!health.marketplace_usage_valid(&outcome));
    }

    #[test]
    fn different_policy_expense_shares_do_not_change_the_enrollment_credit_proportions() {
        // Derived Pub974 p53 reasonable source method: two policies1000/2000
        // in the same month carry200/400 of the month's600 credit. Their
        // specified expense shares1/2 and3/4 do not change those proportions.
        let outcome = crate::marketplace::MarketplaceOutcome {
            expense_columns: alloc::vec![(1, whole(300000), whole(60000))],
            ..Default::default()
        };
        let mut plans = Vec::new();
        for (activity, policy, paid, specified, expected) in
            [(1, 10, 100000, 50000, 30000), (2, 20, 200000, 150000, 110000)]
        {
            let mut actual = source(policy, 1);
            actual.activity = activity;
            actual.paid = whole(paid);
            actual.specified = whole(specified);
            actual.credit_premium = whole(paid);
            let mut plan = SpecifiedPremiums {
                activity,
                sources: alloc::vec![actual],
                funding: [((policy, 1), whole(specified))].into_iter().collect(),
                nonspecified: [((policy, 1), whole(paid - specified))].into_iter().collect(),
                continuing: true,
                premiums: whole(specified),
                maximum: whole(specified),
                deduction: whole(0),
            };
            plan.deduction = plan.after_credit(&outcome);
            assert_eq!(plan.deduction, whole(expected));
            plans.push(plan);
        }
        let mut health = HealthResult {
            nonspecified: [(1, whole(50000)), (2, whole(50000))].into_iter().collect(),
            specified: plans,
            ..Default::default()
        };
        assert_eq!(health.canonical_deduction() + whole(60000), whole(300000));
        assert!(health.marketplace_usage_valid(&outcome));
        health.specified.reverse();
        assert!(health.marketplace_usage_valid(&outcome));
        assert_eq!(health.canonical_deduction(), whole(240000));
    }

    #[test]
    fn actual_entity_funding_caps_net_expense_under_the_original_source() {
        let mut plan = SpecifiedPremiums {
            activity: 1,
            sources: alloc::vec![source(10, 1)],
            funding: [((10, 1), whole(60000))].into_iter().collect(),
            nonspecified: BTreeMap::new(),
            continuing: true,
            premiums: whole(100000),
            maximum: whole(60000),
            deduction: whole(0),
        };
        let outcome = crate::marketplace::MarketplaceOutcome {
            expense_columns: alloc::vec![(1, whole(100000), whole(40000))],
            ..Default::default()
        };
        assert_eq!(plan.after_credit(&outcome), whole(60000));
        // A second policy is unreimbursed. The first policy's funded cost
        // cannot pay that policy when its own expense is fully credited.
        plan.sources.push(source(20, 2));
        plan.funding.insert((20, 2), whole(0));
        plan.premiums = whole(200000);
        let outcome = crate::marketplace::MarketplaceOutcome {
            expense_columns: alloc::vec![
                (1, whole(100000), whole(100000)),
                (2, whole(100000), whole(0))
            ],
            ..Default::default()
        };
        assert_eq!(plan.after_credit(&outcome), whole(0));
    }

    #[test]
    fn selected_deduction_stays_with_remaining_funded_source_costs() {
        let mut plan = SpecifiedPremiums {
            activity: 1,
            sources: alloc::vec![source(10, 1), source(20, 2)],
            funding: [((10, 1), whole(100000)), ((20, 2), whole(0))].into_iter().collect(),
            nonspecified: BTreeMap::new(),
            continuing: true,
            premiums: whole(200000),
            maximum: whole(100000),
            deduction: whole(40001),
        };
        let outcome = crate::marketplace::MarketplaceOutcome {
            expense_columns: alloc::vec![
                (1, whole(100000), whole(20000)),
                (2, whole(100000), whole(0))
            ],
            ..Default::default()
        };
        let used: BTreeMap<_, _> = plan.used_sources(&outcome).into_iter().collect();
        assert_eq!(used[&(10, 1)], whole(40001));
        assert_eq!(used[&(20, 2)], whole(0));
        plan.sources.reverse();
        assert_eq!(plan.used_sources(&outcome).into_iter().collect::<BTreeMap<_, _>>(), used);
        // A capped deduction apportions only the remaining actually funded
        // costs; retain fractions rather than rounding each source to cents.
        plan.funding.insert((20, 2), whole(40000));
        let used: BTreeMap<_, _> = plan.used_sources(&outcome).into_iter().collect();
        assert_eq!(used.values().sum::<Exact>(), plan.deduction);
        assert_eq!(
            used[&(10, 1)],
            whole(40001) * crate::kernel::ratio::exact::ratio(2, 3).unwrap()
        );
        assert_eq!(
            used[&(20, 2)],
            whole(40001) * crate::kernel::ratio::exact::ratio(1, 3).unwrap()
        );
    }

    // Derived one-plan Pub974 Step3 line6/7 and Pub502 funding boundary:
    // allocable PTC is not reduced by tax-free funding or a source-authorized
    // reference enrollment premium that differs from actual current cash.
    // These are source arithmetic cases, not published complete IRS returns.
    #[test]
    fn net_expense_does_not_reduce_the_enrollment_subsidy_allocation() {
        let outcome = crate::marketplace::MarketplaceOutcome {
            expense_columns: alloc::vec![(1, whole(100000), whole(40000))],
            ..Default::default()
        };
        for (paid, expected) in [(80000, 40000), (50000, 10000)] {
            let mut actual = source(10, 1);
            actual.paid = whole(paid);
            actual.specified = whole(paid);
            // The actual Form8962 column(a) enrollment source is1000,
            // independent of800 net tax-free-adjusted or500 actual cash.
            let plan = SpecifiedPremiums {
                activity: 1,
                sources: alloc::vec![actual],
                funding: [((10, 1), whole(paid))].into_iter().collect(),
                nonspecified: BTreeMap::new(),
                continuing: true,
                premiums: whole(paid),
                maximum: whole(paid),
                deduction: whole(expected),
            };
            assert_eq!(plan.after_credit(&outcome), whole(expected));
            assert_eq!(plan.used_sources(&outcome), alloc::vec![((10, 1), whole(expected))]);
            let mut health = HealthResult { specified: alloc::vec![plan], ..Default::default() };
            assert!(health.marketplace_usage_valid(&outcome));
            health.specified[0].deduction += whole(1);
            assert!(!health.marketplace_usage_valid(&outcome));
        }
        let mut duplicate = source(10, 1);
        duplicate.credit_premium = whole(100001);
        let health = HealthResult {
            specified: alloc::vec![SpecifiedPremiums {
                activity: 1,
                sources: alloc::vec![duplicate],
                funding: [((10, 1), whole(100000))].into_iter().collect(),
                nonspecified: BTreeMap::new(),
                continuing: true,
                premiums: whole(100000),
                maximum: whole(100000),
                deduction: whole(0),
            }],
            ..Default::default()
        };
        assert!(!health.marketplace_usage_valid(&outcome));
    }
    // Derived Pub974(2025 final), pp53–54 Step3/5 and WorksheetW14:
    //10000 specified−10800 credit=−800 work; plus5000 nonspecified
    // gives4200 actual deduction. Never clamp the component before summing.
    // https://www.irs.gov/publications/p974
    #[test]
    fn high_credit_offsets_only_its_same_source_nonspecified_usage() {
        let mut actual = source(10, 1);
        actual.paid = whole(1500000);
        actual.specified = whole(1000000);
        actual.credit_premium = whole(1500000);
        let mut plan = SpecifiedPremiums {
            activity: 1,
            sources: alloc::vec![actual],
            funding: [((10, 1), whole(1000000))].into_iter().collect(),
            nonspecified: [((10, 1), whole(500000))].into_iter().collect(),
            continuing: true,
            premiums: whole(1000000),
            maximum: whole(1000000),
            deduction: whole(0),
        };
        let outcome = crate::marketplace::MarketplaceOutcome {
            expense_columns: alloc::vec![(1, whole(1500000), whole(1080000))],
            ..Default::default()
        };
        plan.deduction = plan.after_credit(&outcome);
        assert_eq!(plan.deduction, whole(-80000));
        let mut health = HealthResult {
            specified: alloc::vec![plan],
            nonspecified: [(1, whole(500000))].into_iter().collect(),
            ..Default::default()
        };
        assert_eq!(health.canonical_deduction(), whole(420000));
        assert!(health.marketplace_usage_valid(&outcome));
        // A conservative Simplified Step3 amount remains the actual
        // selected deduction even when final Step4 permits a larger amount.
        let later = crate::marketplace::MarketplaceOutcome {
            expense_columns: alloc::vec![(1, whole(1500000), whole(1027100))],
            ..Default::default()
        };
        health.specified[0].deduction = whole(-163200);
        assert_eq!(
            health.specified[0].used_sources(&later),
            alloc::vec![((10, 1), whole(-163200))]
        );
        assert!(health.marketplace_usage_valid(&later));
        health.specified[0].deduction = whole(-80000);
        // Zero specified entity funding is distinct from an exhausted
        // earned-income limit: continuation still retains this offset.
        health.specified[0].maximum = whole(0);
        health.specified[0].funding.insert((10, 1), whole(0));
        assert_eq!(health.specified[0].after_credit(&outcome), whole(-80000));
        // WorksheetP exhausted-trade stop keeps its own2000 deduction;
        // it does not continue the separate specified-premium subtraction.
        let plan = &mut health.specified[0];
        plan.continuing = false;
        plan.maximum = whole(0);
        plan.nonspecified.insert((10, 1), whole(200000));
        plan.deduction = plan.after_credit(&outcome);
        health.nonspecified.insert(1, whole(200000));
        assert_eq!(plan.deduction, whole(0));
        assert_eq!(health.canonical_deduction(), whole(200000));
        assert!(health.marketplace_usage_valid(&outcome));
    }

    #[test]
    fn capped_positive_source_cannot_scale_away_a_required_negative_offset() {
        // Derived source1:1000cost,600specified,400ordinary,700credit;
        // source2:1000cost,1000specified,200credit. A capped300 work
        // component must use−100/400, then combine source1 ordinary400.
        // Proportionally scaling the negative would duplicate source1 cost.
        let mut first = source(10, 1);
        first.specified = whole(60000);
        let second = source(20, 2);
        let plan = SpecifiedPremiums {
            activity: 1,
            sources: alloc::vec![first, second],
            funding: [((10, 1), whole(60000)), ((20, 2), whole(100000))].into_iter().collect(),
            nonspecified: [((10, 1), whole(40000))].into_iter().collect(),
            continuing: true,
            premiums: whole(160000),
            maximum: whole(30000),
            deduction: whole(30000),
        };
        let outcome = crate::marketplace::MarketplaceOutcome {
            expense_columns: alloc::vec![
                (1, whole(100000), whole(70000)),
                (2, whole(100000), whole(20000))
            ],
            ..Default::default()
        };
        let used: BTreeMap<_, _> = plan.used_sources(&outcome).into_iter().collect();
        assert_eq!(used[&(10, 1)], whole(-10000));
        assert_eq!(used[&(20, 2)], whole(40000));
        assert_eq!(used.values().sum::<Exact>(), plan.deduction);
        let mut health = HealthResult {
            nonspecified: [(1, whole(40000))].into_iter().collect(),
            specified: alloc::vec![plan],
            ..Default::default()
        };
        assert!(health.marketplace_usage_valid(&outcome));
        health.specified[0].sources.reverse();
        assert_eq!(
            health.specified[0].used_sources(&outcome).into_iter().collect::<BTreeMap<_, _>>(),
            used
        );
        assert!(health.marketplace_usage_valid(&outcome));
        // Independently required overclaimed-cent and negative-total bounds.
        health.specified[0].deduction = whole(-40001);
        assert!(!health.marketplace_usage_valid(&outcome));
    }
}
