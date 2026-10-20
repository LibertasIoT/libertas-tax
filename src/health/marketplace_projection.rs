// Transient source-to-month projection for the one Form 8962 engine.
// Pub974 (2025), pp6,19–38; 26 CFR 1.36B-3(c),(d),(f) and 1.36B-4.
// https://www.irs.gov/publications/p974
// Source cents and exact allocation fractions survive until the form's own
// whole-dollar boundaries. No projected month or family total is persisted.
use crate::kernel::phase;
use crate::kernel::ratio::exact::{Exact, whole};
use crate::marketplace::{CalculatedMarketplace, CalculatedMonth};
use crate::marketplace_allocation::{AllocatedBenchmark, allocate};
use crate::*;
use alloc::vec::Vec;

type Family = MarketplaceFamilyRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MissingSource {
    Family,
    Attester,
    Eligibility,
    EnrollmentEvent,
    Allocation,
    LawfulPremium,
    LawfulPayment,
    Benchmark,
    PaymentCoverage,
    HealthBenchmark,
    HealthExpenseAllocation,
    DependentQsehra,
}
impl MissingSource {
    pub(crate) fn resource(self) -> &'static str {
        match self {
            Self::Family => "PTC_SOURCE_FAMILY_REQUIRED",
            Self::Attester => "PTC_SOURCE_ATTESTER_REQUIRED",
            Self::Eligibility => "PTC_SOURCE_ELIGIBILITY_REQUIRED",
            Self::EnrollmentEvent => "PTC_SOURCE_ENROLLMENT_REQUIRED",
            Self::Allocation => "PTC_SOURCE_ALLOCATION_REQUIRED",
            Self::LawfulPremium => "PTC_SOURCE_LAWFUL_PREMIUM_REQUIRED",
            Self::LawfulPayment => "PTC_SOURCE_LAWFUL_PAYMENT_REQUIRED",
            Self::Benchmark => "PTC_SOURCE_BENCHMARK_REQUIRED",
            Self::PaymentCoverage => "PTC_SOURCE_PAYMENT_REQUIRED",
            Self::HealthBenchmark => "HEALTH_POLICY_BENCHMARK_REQUIRED",
            Self::HealthExpenseAllocation => "HEALTH_POLICY_EXPENSE_ALLOCATION_REQUIRED",
            Self::DependentQsehra => "PTC_DEPENDENT_QSEHRA_RECONCILIATION_REQUIRED",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceIssue {
    pub policy: i64,
    pub month: u32,
    pub person: Option<i64>,
    pub missing: MissingSource,
}

pub(crate) struct SourcePeople<'a> {
    // Final tax-family membership is calculated by the existing family owner;
    // the Marketplace editor cannot add somebody to this list.
    pub family: &'a [i64],
    pub current: &'a [i64],
    pub births: &'a [(i64, u32)],
}

fn issue(policy: i64, month: u32, person: Option<i64>, missing: MissingSource) -> SourceIssue {
    SourceIssue { policy, month, person, missing }
}

fn final_family(
    person: i64,
    facts: &MarketplaceFacts,
    people: &SourcePeople<'_>,
) -> Option<Family> {
    if people.family.contains(&person) {
        return Some(Family::ThisReturn);
    }
    if people.current.contains(&person) {
        return match &facts.memberships.iter().find(|r| r.person == person)?.family {
            MarketplaceNonreturnFamily::OtherFamily { family } => {
                Some(Family::Other { family: *family })
            }
            MarketplaceNonreturnFamily::NoFamily => None,
        };
    }
    match &facts.covered.iter().find(|r| r.id == person)?.identity {
        MarketplacePersonIdentity::OtherFamily { family, .. } => {
            Some(Family::Other { family: *family })
        }
        MarketplacePersonIdentity::NoFamily { .. } => None,
    }
}

fn no_family(person: i64, facts: &MarketplaceFacts, people: &SourcePeople<'_>) -> bool {
    if people.current.contains(&person) {
        facts
            .memberships
            .iter()
            .any(|r| r.person == person && matches!(r.family, MarketplaceNonreturnFamily::NoFamily))
    } else {
        facts.covered.iter().any(|r| {
            r.id == person && matches!(r.identity, MarketplacePersonIdentity::NoFamily { .. })
        })
    }
}

// The attesting family reconciles advances for a person in no final tax family
// under 1.36B-4(a)(1)(ii). This does not make that person eligible for PTC.
fn responsibility(
    person: i64,
    facts: &MarketplaceFacts,
    people: &SourcePeople<'_>,
) -> Result<Family, MissingSource> {
    if let Some(family) = final_family(person, facts, people) {
        return Ok(family);
    }
    if !no_family(person, facts, people) {
        return Err(MissingSource::Family);
    }
    match &facts.attestations.iter().find(|r| r.person == person).map(|r| &r.attester) {
        Some(MarketplaceAttester::ThisReturn) => Ok(Family::ThisReturn),
        Some(MarketplaceAttester::OtherFamily { family }) => Ok(Family::Other { family: *family }),
        _ => Err(MissingSource::Attester),
    }
}

fn status(
    facts: &MarketplaceFacts,
    month: u32,
    person: i64,
) -> Option<&MarketplacePersonEligibility> {
    facts.eligibility.iter().find(|r| r.month == month && r.person == person).map(|r| &r.status)
}

fn enrolled(
    coverage: &MarketplacePolicyCoverage,
    person: i64,
    people: &SourcePeople<'_>,
) -> Result<bool, MissingSource> {
    let Some(entry) = coverage.entry.iter().find(|r| r.person == person) else {
        return Ok(true);
    };
    let first = crate::PTC_TAX_YEAR * 10000 + coverage.month * 100 + 1;
    if matches!(entry.basis, MarketplaceEnrollmentBasis::IncomeOnlySpecialEnrollment) {
        return Ok(false);
    }
    if entry.start <= first && entry.end >= first {
        return Ok(true);
    }
    let event = match &entry.basis {
        MarketplaceEnrollmentBasis::Birth => people
            .births
            .iter()
            .find(|(id, _)| *id == person)
            .map(|(_, birth)| *birth)
            .ok_or(MissingSource::EnrollmentEvent)?,
        MarketplaceEnrollmentBasis::Adoption { event }
        | MarketplaceEnrollmentBasis::Foster { event }
        | MarketplaceEnrollmentBasis::CourtOrder { event } => *event,
        MarketplaceEnrollmentBasis::Ordinary
        | MarketplaceEnrollmentBasis::QualifyingSpecialEnrollment { .. } => return Ok(false),
        MarketplaceEnrollmentBasis::IncomeOnlySpecialEnrollment => return Ok(false),
    };
    Ok(event == entry.start && event / 100 == first / 100 && entry.end >= event)
}

fn covered(policy: &MarketplacePolicyRecord, month: u32) -> Option<&MarketplacePolicyCoverage> {
    policy.coverage.iter().find(|r| r.month == month)
}
fn same_set(one: &[i64], two: &[i64]) -> bool {
    one.len() == two.len() && one.iter().all(|id| two.contains(id))
}

fn benchmark(
    source: &MarketplaceBenchmarkSource,
    month: u32,
    facts: &MarketplaceFacts,
) -> Option<Exact> {
    match source {
        MarketplaceBenchmarkSource::Marketplace { amount } => Some(whole(*amount)),
        MarketplaceBenchmarkSource::Statement { policy } => {
            let p = facts
                .forms
                .iter()
                .find(|r| r.id == *policy && r.status != MarketplaceStatementStatus::Void)?;
            crate::marketplace_sources::policy_month(&p.amounts, month)?.1.map(whole)
        }
        MarketplaceBenchmarkSource::ReferenceMonth { policy, month: reference } => {
            let p = facts
                .forms
                .iter()
                .find(|r| r.id == *policy && r.status != MarketplaceStatementStatus::Void)?;
            let lawful = |m| -> Option<Vec<i64>> {
                covered(p, m)?
                    .people
                    .iter()
                    .copied()
                    .filter_map(|person| match status(facts, m, person) {
                        Some(MarketplacePersonEligibility::NotLawfullyPresent) => None,
                        Some(MarketplacePersonEligibility::Unknown) | None => Some(None),
                        Some(_) => Some(Some(person)),
                    })
                    .collect()
            };
            if !same_set(&lawful(month)?, &lawful(*reference)?) {
                return None;
            }
            crate::marketplace_sources::policy_month(&p.amounts, *reference)?.1.map(whole)
        }
    }
}

fn payment(
    policy: &MarketplacePolicyRecord,
    month: u32,
    premium: i64,
    advance: i64,
) -> Result<(i64, i64, bool), MissingSource> {
    let Some(payment) = policy.payments.iter().find(|r| r.month == month) else {
        return Ok((premium, advance, true));
    };
    if let MarketplacePaymentCoverage::NoQualifyingCoverage { issuer_returned_advance } =
        payment.coverage
    {
        // A no-coverage month whose advances have not been returned still
        // needs corrected issuer facts; do not turn it into phantom tax.
        return if issuer_returned_advance == advance {
            Ok((0, 0, false))
        } else {
            Err(MissingSource::PaymentCoverage)
        };
    }
    let appeal_paid = payment.appeal.as_ref().is_some_and(|a| {
        a.enrolled <= crate::TAX_YEAR_END_DATE
            && a.paid_on.is_some_and(|paid| {
                crate::dates::add_days_checked(a.determined, 120).is_some_and(|due| paid <= due)
            })
    });
    let coverage = match payment.coverage {
        MarketplacePaymentCoverage::PaidByDeadline => {
            payment.unpaid == 0 && (payment.appeal.is_none() || appeal_paid)
        }
        MarketplacePaymentCoverage::FirstGraceMonth => true,
        MarketplacePaymentCoverage::ThresholdMet { issuer_provided } => issuer_provided,
        MarketplacePaymentCoverage::EmergencyOrder { issuer_provided, .. } => issuer_provided,
        MarketplacePaymentCoverage::NoQualifyingCoverage { .. } => false,
    };
    // The source statement may already reflect the refund. Actual paid,
    // refunded and unpaid facts establish the deductible amount once.
    Ok((payment.paid - payment.refunded, advance, coverage))
}

struct PolicyMonthProjection {
    policy: i64,
    premium: Exact,
    medical: Exact,
    gross_medical: Exact,
    advance: Exact,
    benchmark: Option<Exact>,
    eligible: Vec<i64>,
    shared: bool,
}

#[derive(Clone, Copy)]
enum PolicyPurpose {
    Credit,
    BusinessPremiums,
}

fn project_policy(
    policy: &MarketplacePolicyRecord,
    month: u32,
    facts: &MarketplaceFacts,
    people: &SourcePeople<'_>,
    qsehra_affordable: &[i64],
    benchmarks: &[MarketplaceBenchmarkMonth],
    purpose: PolicyPurpose,
) -> Result<Option<PolicyMonthProjection>, SourceIssue> {
    if policy.status == MarketplaceStatementStatus::Void {
        return Ok(None);
    }
    let Some((source_premium, source_benchmark, source_advance)) =
        crate::marketplace_sources::policy_month(&policy.amounts, month)
    else {
        return Ok(None);
    };
    let fail = |person, missing| issue(policy.id, month, person, missing);
    let coverage =
        covered(policy, month).ok_or_else(|| fail(None, MissingSource::EnrollmentEvent))?;
    let mut counts: Vec<(Family, usize)> = Vec::new();
    let mut eligible = Vec::new();
    let mut lawful = Vec::new();
    for &person in &coverage.people {
        let family = match responsibility(person, facts, people) {
            Ok(family) => family,
            // With no advances and no final family, an unknown attester
            // cannot create either credit or repayment. Do not assign the
            // person to a fictional family just to produce an explicit zero.
            // A mixed-family policy still needs its real source allocation.
            Err(MissingSource::Attester)
                if source_advance == 0
                    && coverage.people.iter().all(|id| no_family(*id, facts, people)) =>
            {
                continue;
            }
            Err(missing) => return Err(fail(Some(person), missing)),
        };
        if let Some((_, count)) = counts.iter_mut().find(|(f, _)| f == &family) {
            *count += 1;
        } else {
            counts.push((family, 1));
        }
        let eligibility = status(facts, month, person)
            .filter(|v| !matches!(v, MarketplacePersonEligibility::Unknown))
            .ok_or_else(|| fail(Some(person), MissingSource::Eligibility))?;
        if *eligibility != MarketplacePersonEligibility::NotLawfullyPresent {
            lawful.push(person);
        }
        if final_family(person, facts, people) == Some(Family::ThisReturn)
            && (*eligibility == MarketplacePersonEligibility::Eligible
                || matches!(purpose, PolicyPurpose::BusinessPremiums)
                    && *eligibility == MarketplacePersonEligibility::AffordableExternalQsehra)
            && !qsehra_affordable.contains(&person)
            && enrolled(coverage, person, people).map_err(|m| fail(Some(person), m))?
        {
            eligible.push(person);
        }
    }
    let (paid_premium, advance, payment_eligible) =
        payment(policy, month, source_premium, source_advance).map_err(|m| fail(None, m))?;
    if !payment_eligible {
        eligible.clear();
    }
    let mut premium = whole(paid_premium);
    if lawful.len() != coverage.people.len() && !eligible.is_empty() {
        let correction = policy
            .lawful_premiums
            .iter()
            .find(|r| r.month == month && same_set(&r.people, &lawful))
            .ok_or_else(|| fail(None, MissingSource::LawfulPremium))?;
        premium = match correction.source {
            MarketplaceLawfulPremiumSource::Marketplace { amount } => whole(amount),
            MarketplaceLawfulPremiumSource::ReferenceMonth { month: reference } => {
                let reference_coverage = covered(policy, reference)
                    .ok_or_else(|| fail(None, MissingSource::LawfulPremium))?;
                if !same_set(&reference_coverage.people, &lawful) {
                    return Err(fail(None, MissingSource::LawfulPremium));
                }
                let amount = crate::marketplace_sources::policy_month(&policy.amounts, reference)
                    .ok_or_else(|| fail(None, MissingSource::LawfulPremium))?
                    .0;
                whole(amount)
            }
        };
        // Pub974 p6 and pp19–22 establish different source facts: lawful
        // enrollment cost does not allocate a current partial payment/refund.
        // Never replace actual payments with a larger reference-month cost.
        let payment_changed =
            policy.payments.iter().any(|r| r.month == month && (r.unpaid != 0 || r.refunded != 0));
        if let Some(paid) = correction.paid {
            if paid < 0 || paid > paid_premium || whole(paid) > premium {
                return Err(fail(None, MissingSource::LawfulPayment));
            }
            premium = whole(paid);
        } else if payment_changed {
            return Err(fail(None, MissingSource::LawfulPayment));
        }
    }
    let allocation = policy.allocation.iter().find(|r| (r.from..=r.to).contains(&month));
    let (premium_share, advance_share, benchmark_amount, shared) = if let Some(a) = allocation {
        let shares = allocate(&a.kind, &policy.enrolling, &counts, |s| benchmark(s, month, facts))
            .ok_or_else(|| fail(None, MissingSource::Allocation))?;
        match shares.into_iter().find(|r| r.family == Family::ThisReturn) {
            Some(share) => {
                let amount = match share.benchmark {
                    AllocatedBenchmark::FamilyAmount(amount) => Some(amount),
                    AllocatedBenchmark::SourceShare(fraction) => {
                        let source = benchmarks
                            .iter()
                            .find(|r| r.month == month)
                            .and_then(|r| r.allocated.iter().find(|r| r.policy == policy.id));
                        let amount = match source {
                            Some(source) => benchmark(&source.source, month, facts),
                            None => source_benchmark.map(whole),
                        };
                        amount.map(|v| v * fraction)
                    }
                };
                (share.premium, share.advance, amount, true)
            }
            None => (whole(0), whole(0), Some(whole(0)), true),
        }
    } else {
        if counts.is_empty() && source_advance == 0 {
            (whole(0), whole(0), Some(whole(0)), false)
        } else if counts.len() != 1 {
            return Err(fail(None, MissingSource::Allocation));
        } else if counts[0].0 == Family::ThisReturn {
            (whole(1), whole(1), source_benchmark.map(whole), false)
        } else {
            (whole(0), whole(0), Some(whole(0)), false)
        }
    };
    let allocated = premium * &premium_share;
    let gross_medical = whole(paid_premium) * premium_share;
    let tax_free =
        policy.payments.iter().find(|r| r.month == month).map_or(0, |r| r.tax_free_premiums);
    let medical = phase::floor(whole(0), &gross_medical - whole(tax_free));
    Ok(Some(PolicyMonthProjection {
        policy: policy.id,
        premium: if eligible.is_empty() { whole(0) } else { allocated },
        medical,
        gross_medical,
        advance: whole(advance) * advance_share,
        benchmark: benchmark_amount,
        eligible,
        shared,
    }))
}

// Source cents for the existing health deduction owner. PartIV has already
// allocated the policy to this return. The coverage-family split precedes
// business assignments and uses actual SLCSP sources (Pub974 p49, Gary).
pub(crate) struct HealthPolicyPremium {
    pub paid: Exact,
    pub specified: Exact,
    pub credit_premium: Exact,
    #[cfg(test)]
    pub advance: Exact,
}

// Expense consumers use actual paid premiums and the People owner's medical
// eligibility, independently of the tax family used to reconcile Form8962.
pub(crate) struct MedicalPolicyPremium {
    pub paid: Exact,
    pub tax_free: i64,
    pub credit_premium: Exact,
    pub people: Vec<i64>,
}

pub(crate) fn medical_policy_month(
    facts: &MarketplaceFacts,
    people: &SourcePeople<'_>,
    policy: &MarketplacePolicyRecord,
    month: u32,
) -> Result<Option<MedicalPolicyPremium>, SourceIssue> {
    let Some(projected) = project_policy(
        policy,
        month,
        facts,
        people,
        &[],
        &facts.benchmarks,
        PolicyPurpose::Credit,
    )?
    else {
        return Ok(None);
    };
    let coverage = covered(policy, month).expect("projected source coverage");
    Ok(Some(MedicalPolicyPremium {
        paid: projected.medical,
        tax_free: policy
            .payments
            .iter()
            .find(|p| p.month == month)
            .map_or(0, |p| p.tax_free_premiums),
        credit_premium: projected.premium,
        people: coverage.people.clone(),
    }))
}

pub(crate) fn health_policy_month(
    facts: &MarketplaceFacts,
    people: &SourcePeople<'_>,
    policy: i64,
    month: u32,
    sources: &[HealthPolicyBenchmark],
) -> Result<Option<HealthPolicyPremium>, SourceIssue> {
    let Some(source) = facts.forms.iter().find(|p| p.id == policy) else {
        return Err(issue(policy, month, None, MissingSource::Allocation));
    };
    // Affordable QSEHRA bars credit but is not non-Marketplace MEC.
    // Actual employer-subsidized eligibility for section162(l) is a separate
    // plan fact; it does not redefine this source coverage-family split.
    let Some(projected) = project_policy(
        source,
        month,
        facts,
        people,
        &[],
        &facts.benchmarks,
        PolicyPurpose::BusinessPremiums,
    )?
    else {
        return Ok(None);
    };
    let coverage = covered(source, month).expect("projected source coverage");
    let lawful_family = coverage
        .people
        .iter()
        .filter(|person| {
            responsibility(**person, facts, people) == Ok(Family::ThisReturn)
                && status(facts, month, **person)
                    != Some(&MarketplacePersonEligibility::NotLawfullyPresent)
        })
        .count();
    // Only amounts actually paid can supply a section162(l) deduction.
    // A source-authorized reference premium used for PTC may differ from cash.
    let paid = projected.medical;
    let available = phase::cap(projected.gross_medical.clone(), [projected.premium.clone()]);
    let specified_share = if projected.eligible.is_empty() {
        whole(0)
    } else if projected.eligible.len() == lawful_family {
        whole(1)
    } else {
        let missing = || issue(policy, month, None, MissingSource::HealthBenchmark);
        let row =
            sources.iter().find(|r| r.policy == policy && r.month == month).ok_or_else(missing)?;
        let numerator = benchmark(&row.coverage, month, facts).ok_or_else(missing)?;
        let other = benchmark(&row.noncoverage, month, facts).ok_or_else(missing)?;
        let denominator = &numerator + other;
        if numerator <= whole(0) || denominator <= whole(0) {
            return Err(missing());
        }
        numerator / denominator
    };
    let gross_specified = available * &specified_share;
    // Subsidy allocation follows the actual enrollment-premium source, not
    // the smaller cash cost left after partial payment or tax-free funding.
    let credit_premium = project_policy(
        source,
        month,
        facts,
        people,
        &[],
        &facts.benchmarks,
        PolicyPurpose::Credit,
    )?
    .expect("same accepted source month")
    .premium;
    let payment = source.payments.iter().find(|r| r.month == month);
    let tax_free = payment.map_or(0, |r| r.tax_free_premiums);
    let return_members = coverage
        .people
        .iter()
        .filter(|person| responsibility(**person, facts, people) == Ok(Family::ThisReturn))
        .count();
    let actual_offset = payment.and_then(|r| r.tax_free_coverage);
    let allocation_error = || issue(policy, month, None, MissingSource::HealthExpenseAllocation);
    let excluded = if tax_free == 0 || gross_specified == whole(0) {
        if actual_offset.is_some_and(|v| v != 0) {
            return Err(allocation_error());
        }
        whole(0)
    } else if projected.eligible.len() == return_members {
        if actual_offset.is_some_and(|v| v != tax_free) {
            return Err(allocation_error());
        }
        whole(tax_free)
    } else {
        let missing = || issue(policy, month, None, MissingSource::HealthExpenseAllocation);
        let actual = payment.and_then(|r| r.tax_free_coverage).ok_or_else(missing)?;
        if whole(actual) > gross_specified
            || whole(tax_free - actual) > &projected.gross_medical - &gross_specified
        {
            return Err(missing());
        }
        whole(actual)
    };
    let specified = phase::floor(whole(0), gross_specified - excluded);
    Ok(Some(HealthPolicyPremium {
        paid,
        specified,
        credit_premium,
        #[cfg(test)]
        advance: projected.advance,
    }))
}

// Complete source responsibility makes OtherShared defaults/agreements
// decidable at Save. Reuse the one allocation owner; missing family/attester
// facts remain accepted sources needing correction rather than invented zeros.
pub(crate) fn allocation_sources_valid(
    facts: &MarketplaceFacts,
    people: &SourcePeople<'_>,
) -> bool {
    facts.forms.iter().filter(|policy| policy.status != MarketplaceStatementStatus::Void).all(
        |policy| {
            policy.coverage.iter().all(|coverage| {
                let Some(period) = policy
                    .allocation
                    .iter()
                    .find(|period| (period.from..=period.to).contains(&coverage.month))
                else {
                    return true;
                };
                if !matches!(period.kind, MarketplaceAllocationKind::OtherShared { .. }) {
                    return true;
                }
                let mut counts: Vec<(Family, usize)> = Vec::new();
                for person in &coverage.people {
                    let Ok(family) = responsibility(*person, facts, people) else {
                        return true;
                    };
                    if let Some((_, count)) =
                        counts.iter_mut().find(|(existing, _)| *existing == family)
                    {
                        *count += 1;
                    } else {
                        counts.push((family, 1));
                    }
                }
                allocate(&period.kind, &policy.enrolling, &counts, |_| None).is_some()
            })
        },
    )
}

// Conditional source bound: unknown mandatory policy facts remain accepted
// for normal correction, but a complete projection cannot overstate the
// actual return's tax-free expense offset. Never infer it from an agreement.
pub(crate) fn expense_sources_valid(facts: &MarketplaceFacts, people: &SourcePeople<'_>) -> bool {
    facts.forms.iter().all(|policy| {
        policy.payments.iter().all(|payment| {
            if payment.tax_free_premiums == 0 {
                return true;
            }
            match project_policy(
                policy,
                payment.month,
                facts,
                people,
                &[],
                &facts.benchmarks,
                PolicyPurpose::BusinessPremiums,
            ) {
                Ok(Some(projected)) => whole(payment.tax_free_premiums) <= projected.gross_medical,
                _ => true,
            }
        })
    })
}

#[cfg(test)]
pub(crate) fn project(
    facts: &MarketplaceFacts,
    people: &SourcePeople<'_>,
    applicable_taxpayer: bool,
    income: i64,
) -> Result<CalculatedMarketplace, Vec<SourceIssue>> {
    project_selected(facts, people, applicable_taxpayer, income, None, &facts.benchmarks, 12)
}

// Worksheets II/IV consume an explicit source subset while reference-month
// links continue to resolve against the one complete canonical catalog.
#[cfg(test)]
pub(crate) fn project_selected(
    facts: &MarketplaceFacts,
    people: &SourcePeople<'_>,
    applicable_taxpayer: bool,
    income: i64,
    selected: Option<&[i64]>,
    benchmarks: &[MarketplaceBenchmarkMonth],
    last_month: u32,
) -> Result<CalculatedMarketplace, Vec<SourceIssue>> {
    project_selected_with_income(
        facts,
        people,
        applicable_taxpayer,
        &(whole(income) * whole(100)),
        selected,
        benchmarks,
        last_month,
    )
}
pub(crate) fn project_selected_with_income(
    facts: &MarketplaceFacts,
    people: &SourcePeople<'_>,
    applicable_taxpayer: bool,
    income: &Exact,
    selected: Option<&[i64]>,
    benchmarks: &[MarketplaceBenchmarkMonth],
    last_month: u32,
) -> Result<CalculatedMarketplace, Vec<SourceIssue>> {
    let mut rows = Vec::new();
    let mut issues = Vec::new();
    let mut shared = false;
    for month in 1..=last_month {
        if !facts.forms.iter().any(|p| {
            p.status != MarketplaceStatementStatus::Void
                && selected.is_none_or(|ids| ids.contains(&p.id))
                && crate::marketplace_sources::policy_month(&p.amounts, month).is_some()
        }) {
            continue;
        }
        let mut qsehra_affordable = Vec::new();
        let mut credit_reduction = whole(0);
        for q in facts.qsehra.iter().filter(|q| people.family.contains(&q.employee)) {
            if !q.months.iter().any(|m| m.month == month) {
                continue;
            }
            let employee_income = if q.employee > 0 {
                let Some(value) = crate::marketplace_qsehra::employee_income(facts, q.employee)
                else {
                    issues.push(issue(0, month, Some(q.employee), MissingSource::DependentQsehra));
                    continue;
                };
                value
            } else {
                income.clone()
            };
            let Some(assessment) =
                crate::marketplace_qsehra::assess_with_income(q, month, &employee_income)
            else {
                continue;
            };
            // §36B(c)(4)(B) determines the coverage month without regard to
            // (A), so the actual permitted-benefit reduction is not filtered
            // solely to unaffordable arrangements.
            credit_reduction += &assessment.permitted;
            if assessment.affordable {
                let related = if q.employee > 0 {
                    let Some(related) =
                        crate::marketplace_qsehra::employee_related(facts, q.employee)
                    else {
                        issues.push(issue(
                            0,
                            month,
                            Some(q.employee),
                            MissingSource::DependentQsehra,
                        ));
                        continue;
                    };
                    related
                } else {
                    people.family.to_vec()
                };
                for person in related {
                    if !qsehra_affordable.contains(&person) {
                        qsehra_affordable.push(person);
                    }
                }
            }
        }
        let mut policies = Vec::new();
        for p in facts.forms.iter().filter(|p| selected.is_none_or(|ids| ids.contains(&p.id))) {
            match project_policy(
                p,
                month,
                facts,
                people,
                &qsehra_affordable,
                benchmarks,
                PolicyPurpose::Credit,
            ) {
                Ok(Some(p)) => policies.push(p),
                Ok(None) => {}
                Err(issue) => issues.push(issue),
            }
        }
        if policies.is_empty() {
            continue;
        }
        let mut eligible = Vec::new();
        for p in &policies {
            for person in &p.eligible {
                if !eligible.contains(person) {
                    eligible.push(*person);
                }
            }
        }
        let mut amount = whole(0);
        let unallocated: Vec<_> =
            policies.iter().filter(|p| !p.shared && !p.eligible.is_empty()).collect();
        let allocated: Vec<_> =
            policies.iter().filter(|p| p.shared && !p.eligible.is_empty()).collect();
        if let Some(correction) =
            benchmarks.iter().find(|r| r.month == month).filter(|_| !eligible.is_empty())
        {
            let mut supplied = Vec::new();
            for group in &correction.groups {
                for &person in &group.people {
                    supplied.push(person);
                }
                if let Some(value) = benchmark(&group.source, month, facts) {
                    amount += value;
                } else {
                    issues.push(issue(0, month, None, MissingSource::Benchmark));
                }
            }
            let mut expected = Vec::new();
            for p in &unallocated {
                for person in &p.eligible {
                    if !expected.contains(person) {
                        expected.push(*person);
                    }
                }
            }
            if !same_set(&supplied, &expected) {
                issues.push(issue(0, month, None, MissingSource::Benchmark));
            }
        } else if unallocated.len() == 1 {
            let p = unallocated[0];
            let source =
                facts.forms.iter().find(|r| r.id == p.policy).and_then(|p| covered(p, month));
            // An unchanged, wholly eligible single policy can use its printed
            // benchmark. Mixed or multiple-policy families require actual
            // applicable family/location components, never a head-count guess.
            if source.is_some_and(|c| same_set(&c.people, &p.eligible)) {
                if let Some(value) = &p.benchmark {
                    amount += value;
                } else {
                    issues.push(issue(p.policy, month, None, MissingSource::Benchmark));
                }
            } else {
                issues.push(issue(p.policy, month, None, MissingSource::Benchmark));
            }
        } else if !unallocated.is_empty() {
            issues.push(issue(0, month, None, MissingSource::Benchmark));
        }
        for p in allocated {
            if let Some(value) = &p.benchmark {
                amount += value;
            } else {
                issues.push(issue(p.policy, month, None, MissingSource::Benchmark));
            }
        }
        shared |= policies.iter().any(|p| p.shared);
        rows.push(CalculatedMonth {
            month,
            premium: policies.iter().map(|p| &p.premium).sum(),
            #[cfg(test)]
            medical_premium: policies.iter().map(|p| &p.medical).sum(),
            credit_reduction,
            benchmark: amount,
            advance: policies.iter().map(|p| &p.advance).sum(),
            eligible: !eligible.is_empty(),
        });
    }
    if !issues.is_empty() {
        return Err(issues);
    }
    let below_fpl_qualified = matches!(
        facts.income_qualification,
        MarketplaceIncomeQualification::MarketplaceEstimate {
            estimated_at_least_100: true,
            accurate_disclosure: true,
            not_intentional_or_reckless: true,
        }
    ) && rows.iter().any(|m| m.advance > whole(0));
    Ok(CalculatedMarketplace {
        residence: facts.residence,
        rows,
        shared,
        applicable_taxpayer,
        below_fpl_qualified,
    })
}

pub(crate) fn family_coverage_months(
    facts: &MarketplaceFacts,
    people: &SourcePeople<'_>,
    family: &[i64],
    last_month: u32,
) -> Vec<u32> {
    (1..=last_month)
        .filter(|month| {
            facts.forms.iter().any(|p| {
                p.status != MarketplaceStatementStatus::Void
                    && covered(p, *month).is_some_and(|coverage| {
                        family.iter().any(|id| {
                            coverage.people.contains(id)
                                && enrolled(coverage, *id, people) == Ok(true)
                        })
                    })
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::String, vec};

    fn policy(people: Vec<i64>) -> MarketplacePolicyRecord {
        MarketplacePolicyRecord {
            id: 1,
            issuer: "Source issuer".into(),
            policy_number: String::new(),
            amounts: PolicyAmounts::ByMonth {
                months: vec![PolicyMonth {
                    month: 1,
                    premium: 100_000,
                    benchmark: Some(120_000),
                    advance: 60_000,
                }],
            },
            lawful_premiums: vec![],
            enrolling: Family::ThisReturn,
            status: MarketplaceStatementStatus::Original,
            coverage: vec![MarketplacePolicyCoverage { month: 1, people, entry: vec![] }],
            allocation: vec![],
            payments: vec![],
            medical_costs: vec![],
        }
    }
    fn facts(people: &[i64]) -> MarketplaceFacts {
        MarketplaceFacts {
            forms: vec![policy(people.to_vec())],
            eligibility: people
                .iter()
                .map(|person| MarketplaceEligibilityMonth {
                    month: 1,
                    person: *person,
                    status: MarketplacePersonEligibility::Eligible,
                })
                .collect(),
            ..MarketplaceFacts::default()
        }
    }
    fn area(people: Vec<i64>, amount: i64) -> MarketplaceBenchmarkMonth {
        MarketplaceBenchmarkMonth {
            month: 1,
            groups: vec![MarketplaceBenchmarkGroup {
                state: TaxState::CA,
                rating_area: "Actual source area".into(),
                people,
                source: MarketplaceBenchmarkSource::Marketplace { amount },
            }],
            allocated: vec![],
        }
    }

    #[test]
    fn mixed_mec_preserves_enrollment_premium_and_uses_actual_eligible_benchmark() {
        // Derived Pub974 pp6–7 coverage-family rule: full $1,000 premium,
        // actual eligible-family $900 SLCSP, not a two-person head-count split.
        let mut f = facts(&[-1, -2]);
        f.eligibility[1].status =
            MarketplacePersonEligibility::OtherMec { kind: MarketplaceMecKind::Medicare };
        let people = SourcePeople { family: &[-1, -2], current: &[-1, -2], births: &[] };
        assert!(project(&f, &people, true, 48_000).is_err());
        f.benchmarks.push(area(vec![-1], 90_000));
        let p = project(&f, &people, true, 48_000).unwrap();
        assert_eq!(p.rows[0].premium, whole(100_000));
        assert_eq!(p.rows[0].benchmark, whole(90_000));
        assert_eq!(p.rows[0].advance, whole(60_000));
        f.benchmarks[0].groups[0].people.push(-2);
        assert!(project(&f, &people, true, 48_000).is_err());
    }

    #[test]
    fn allocated_and_unallocated_benchmarks_combine_actual_components_once() {
        // Approved Unit24 fixtureI, i8962 PartII column(b): allocated$300
        // plus the unallocated eligible-family$600 gives$900, never$1500.
        let mut f = facts(&[-1, 20]);
        f.other_families.push(MarketplaceFamily { id: 10, label: "Other actual family".into() });
        f.covered.push(MarketplaceCoveredPerson {
            id: 20,
            identity: MarketplacePersonIdentity::OtherFamily {
                family: 10,
                label: "Other enrollee".into(),
            },
        });
        if let PolicyAmounts::ByMonth { months } = &mut f.forms[0].amounts {
            months[0].benchmark = Some(60_000);
        }
        let share = |family| MarketplaceFamilyShare {
            family,
            agreement: MarketplaceAgreement::Agreed {
                share: MarketplaceFraction { numerator: 1, denominator: 2 },
            },
        };
        f.forms[0].allocation.push(MarketplaceAllocationPeriod {
            from: 1,
            to: 1,
            kind: MarketplaceAllocationKind::OtherShared {
                families: vec![share(Family::ThisReturn), share(Family::Other { family: 10 })],
            },
        });
        let mut second = policy(vec![-1]);
        second.id = 2;
        if let PolicyAmounts::ByMonth { months } = &mut second.amounts {
            months[0].premium = 70_000;
            months[0].benchmark = Some(60_000);
            months[0].advance = 0;
        }
        f.forms.push(second.clone());
        f.benchmarks.push(area(vec![-1], 60_000));
        let people = SourcePeople { family: &[-1], current: &[-1], births: &[] };
        let p = project(&f, &people, true, 48_000).unwrap();
        assert_eq!(p.rows[0].premium, whole(120_000));
        assert_eq!(p.rows[0].benchmark, whole(90_000));
        assert_eq!(p.rows[0].advance, whole(30_000));
        assert_eq!(crate::marketplace::premium_assistance(&p.rows[0], 200, true), 700);
        second.id = 3;
        f.forms.push(second);
        let repeated = project(&f, &people, true, 48_000).unwrap();
        assert_eq!(repeated.rows[0].benchmark, whole(90_000));
        assert_eq!(repeated.rows[0].advance, whole(30_000));
        f.benchmarks.clear();
        assert!(
            matches!(project(&f, &people, true, 48_000), Err(issues) if issues.iter().any(|i| i.missing == MissingSource::Benchmark))
        );
        f.benchmarks.push(area(vec![-1, 20], 60_000));
        assert!(project(&f, &people, true, 48_000).is_err());
        f.benchmarks[0].groups[0].people = vec![-1];
        let duplicate = f.benchmarks[0].groups[0].clone();
        f.benchmarks[0].groups.push(duplicate);
        assert!(project(&f, &people, true, 48_000).is_err());
    }

    #[test]
    fn three_family_source_allocates_premium_benchmark_and_advance_once() {
        // Pub974 (2025), pp37–38 Example1: source $8,000/$9,000/$4,500;
        // Bill25%, Sharon40%, Erik35%. One synthetic month isolates the
        // published annual allocation oracle without changing rounding stages.
        let mut f = facts(&[-1, 20, 21]);
        f.other_families = vec![
            MarketplaceFamily { id: 10, label: "Sharon".into() },
            MarketplaceFamily { id: 11, label: "Erik".into() },
        ];
        f.covered = vec![
            MarketplaceCoveredPerson {
                id: 20,
                identity: MarketplacePersonIdentity::OtherFamily {
                    family: 10,
                    label: "Sharon".into(),
                },
            },
            MarketplaceCoveredPerson {
                id: 21,
                identity: MarketplacePersonIdentity::OtherFamily {
                    family: 11,
                    label: "Erik".into(),
                },
            },
        ];
        f.forms[0].amounts = PolicyAmounts::ByMonth {
            months: vec![PolicyMonth {
                month: 1,
                premium: 800_000,
                benchmark: Some(900_000),
                advance: 450_000,
            }],
        };
        let share = |family, numerator| MarketplaceFamilyShare {
            family,
            agreement: MarketplaceAgreement::Agreed {
                share: MarketplaceFraction { numerator, denominator: 100 },
            },
        };
        f.forms[0].allocation.push(MarketplaceAllocationPeriod {
            from: 1,
            to: 1,
            kind: MarketplaceAllocationKind::OtherShared {
                families: vec![
                    share(Family::ThisReturn, 25),
                    share(Family::Other { family: 10 }, 40),
                    share(Family::Other { family: 11 }, 35),
                ],
            },
        });
        let p =
            project(&f, &SourcePeople { family: &[-1], current: &[-1], births: &[] }, true, 48_000)
                .unwrap();
        assert!(p.shared);
        assert_eq!(p.rows[0].premium, whole(200_000));
        assert_eq!(p.rows[0].benchmark, whole(225_000));
        assert_eq!(p.rows[0].advance, whole(112_500));
        // Derived expense boundary: the actual return has only $2,000
        // of this $8,000 shared premium. A return-attributable tax-free
        // offset is not divided by the agreement again and cannot exceed it.
        f.forms[0].payments.push(MarketplacePaymentMonth {
            month: 1,
            paid: 800_000,
            refunded: 0,
            tax_free_premiums: 100_000,
            tax_free_coverage: None,
            unpaid: 0,
            coverage: MarketplacePaymentCoverage::PaidByDeadline,
            appeal: None,
        });
        let people = SourcePeople { family: &[-1], current: &[-1], births: &[] };
        assert!(expense_sources_valid(&f, &people));
        let funded = project(&f, &people, true, 48_000).unwrap();
        assert_eq!(funded.rows[0].premium, whole(200_000));
        assert_eq!(funded.rows[0].medical_premium, whole(100_000));
        f.forms[0].payments[0].tax_free_premiums = 200_001;
        assert!(!expense_sources_valid(&f, &people));
    }

    #[test]
    fn actual_payments_refunds_and_grace_exception_have_separate_effects() {
        // Derived Pub974 p6: $1,000 liability, $600 APTC+$300 cash,
        // $100 unpaid. No exception means no monthly credit; first grace
        // month permits credit using $900 paid premium. The paid expense
        // remains available to its owning deduction calculation in either case.
        let mut f = facts(&[-1]);
        f.forms[0].payments.push(MarketplacePaymentMonth {
            tax_free_premiums: 0,
            tax_free_coverage: None,
            month: 1,
            paid: 90_000,
            refunded: 0,
            unpaid: 10_000,
            coverage: MarketplacePaymentCoverage::PaidByDeadline,
            appeal: None,
        });
        let people = SourcePeople { family: &[-1], current: &[-1], births: &[] };
        let p = project(&f, &people, true, 48_000).unwrap();
        assert!(!p.rows[0].eligible);
        assert_eq!(p.rows[0].premium, whole(0));
        assert_eq!(p.rows[0].medical_premium, whole(90_000));
        assert_eq!(p.rows[0].advance, whole(60_000));
        f.forms[0].payments[0].coverage = MarketplacePaymentCoverage::FirstGraceMonth;
        assert_eq!(project(&f, &people, true, 48_000).unwrap().rows[0].premium, whole(90_000));
        f.forms[0].payments[0] = MarketplacePaymentMonth {
            tax_free_premiums: 0,
            tax_free_coverage: None,
            month: 1,
            paid: 100_000,
            refunded: 10_000,
            unpaid: 0,
            coverage: MarketplacePaymentCoverage::PaidByDeadline,
            appeal: None,
        };
        // Both a gross source and an issuer-adjusted net source reconcile to
        // the same actual paid expense; a refund is never subtracted twice.
        assert!(crate::marketplace_sources::valid(&f, &[-1], &[]));
        assert_eq!(project(&f, &people, true, 48_000).unwrap().rows[0].premium, whole(90_000));
        let PolicyAmounts::ByMonth { months } = &mut f.forms[0].amounts else { unreachable!() };
        months[0].premium = 90_000;
        assert!(crate::marketplace_sources::valid(&f, &[-1], &[]));
        assert_eq!(project(&f, &people, true, 48_000).unwrap().rows[0].premium, whole(90_000));
    }

    #[test]
    fn lawful_reference_cost_never_replaces_actual_partial_payments_or_refunds() {
        // Derived intersection of Pub974 (2025) p6 paid-premium limits and
        // pp19–22 lawful reference premiums. The source separately establishes
        // $1,000 lawful cost and $500 actual lawful net paid; no allocation
        // of a partial payment to lawful people is inferred by the App.
        for reference in [false, true] {
            for refund in [false, true] {
                let mut f = facts(&[-1, -2]);
                f.eligibility[1].status = MarketplacePersonEligibility::NotLawfullyPresent;
                let PolicyAmounts::ByMonth { months } = &mut f.forms[0].amounts else {
                    unreachable!()
                };
                months[0].premium = 150_000;
                months[0].advance = 40_000;
                months.push(PolicyMonth {
                    month: 2,
                    premium: 100_000,
                    benchmark: Some(90_000),
                    advance: 0,
                });
                f.forms[0].coverage.push(MarketplacePolicyCoverage {
                    month: 2,
                    people: vec![-1],
                    entry: vec![],
                });
                f.eligibility.push(MarketplaceEligibilityMonth {
                    month: 2,
                    person: -1,
                    status: MarketplacePersonEligibility::Eligible,
                });
                f.benchmarks.push(area(vec![-1], 90_000));
                f.forms[0].payments.push(MarketplacePaymentMonth {
                    tax_free_premiums: 0,
                    tax_free_coverage: None,
                    month: 1,
                    paid: if refund { 150_000 } else { 50_000 },
                    refunded: if refund { 100_000 } else { 0 },
                    unpaid: if refund { 0 } else { 100_000 },
                    coverage: MarketplacePaymentCoverage::FirstGraceMonth,
                    appeal: None,
                });
                f.forms[0].lawful_premiums.push(MarketplaceLawfulPremiumMonth {
                    month: 1,
                    people: vec![-1],
                    source: if reference {
                        MarketplaceLawfulPremiumSource::ReferenceMonth { month: 2 }
                    } else {
                        MarketplaceLawfulPremiumSource::Marketplace { amount: 100_000 }
                    },
                    paid: None,
                });
                let people = SourcePeople { family: &[-1, -2], current: &[-1, -2], births: &[] };
                assert!(
                    project(&f, &people, true, 48_000)
                        .err()
                        .unwrap()
                        .iter()
                        .any(|i| i.missing == MissingSource::LawfulPayment)
                );
                f.forms[0].lawful_premiums[0].paid = Some(50_000);
                let result = project(&f, &people, true, 48_000).unwrap();
                assert_eq!(result.rows[0].premium, whole(50_000));
                assert_eq!(result.rows[0].medical_premium, whole(50_000));
                assert_eq!(result.rows[0].advance, whole(40_000));
                f.forms[0].lawful_premiums[0].paid = Some(50_001);
                assert!(project(&f, &people, true, 48_000).is_err());
            }
        }
    }

    #[test]
    fn lawful_reference_month_substitution_is_distinct_from_current_cash_paid() {
        // Pub974 (2025), pp19–22, Reference month for enrollment premiums
        // and WorksheetA line2 explicitly substitute the matching lawful-only
        // month. They do not prescribe min(current gross, reference cost).
        // Synthetic source amounts deliberately distinguish that rule from
        // actual current medical payments and partial-payment allocation.
        let mut f = facts(&[-1, -2]);
        f.eligibility[1].status = MarketplacePersonEligibility::NotLawfullyPresent;
        let PolicyAmounts::ByMonth { months } = &mut f.forms[0].amounts else { unreachable!() };
        months.push(PolicyMonth {
            month: 2,
            premium: 200_000,
            benchmark: Some(220_000),
            advance: 0,
        });
        f.forms[0].coverage.push(MarketplacePolicyCoverage {
            month: 2,
            people: vec![-1],
            entry: vec![],
        });
        f.eligibility.push(MarketplaceEligibilityMonth {
            month: 2,
            person: -1,
            status: MarketplacePersonEligibility::Eligible,
        });
        f.forms[0].lawful_premiums.push(MarketplaceLawfulPremiumMonth {
            month: 1,
            people: vec![-1],
            source: MarketplaceLawfulPremiumSource::ReferenceMonth { month: 2 },
            paid: None,
        });
        f.benchmarks.push(area(vec![-1], 220_000));
        let people = SourcePeople { family: &[-1, -2], current: &[-1, -2], births: &[] };
        let projection = project(&f, &people, true, 48_000).unwrap();
        assert_eq!(projection.rows[0].premium, whole(200_000));
        assert_eq!(projection.rows[0].medical_premium, whole(100_000));
        assert_eq!(projection.rows[0].advance, whole(60_000));
        let health = health_policy_month(&f, &people, 1, 1, &[]).unwrap().unwrap();
        assert_eq!(health.paid, whole(100_000));
        assert_eq!(health.specified, whole(100_000));
        assert_eq!(health.credit_premium, whole(200_000));
    }

    #[test]
    fn external_employee_affordability_is_person_specific_not_a_parent_bar() {
        // IRC36B(c)(4)(A), Notice2017-67§K: the employee's spouse/dependents
        // are barred. The employee's parent/sibling is not their dependent by
        // being on the same return. Here actual tax-family facts establish
        // two people; only the second has the known external-employee bar.
        let mut f = facts(&[-1, 3]);
        f.eligibility[1].status = MarketplacePersonEligibility::AffordableExternalQsehra;
        f.benchmarks.push(area(vec![-1], 80_000));
        let people = SourcePeople { family: &[-1, 3], current: &[-1, 3], births: &[] };
        let projection = project(&f, &people, true, 48_000).unwrap();
        assert!(projection.rows[0].eligible);
        assert_eq!(projection.rows[0].benchmark, whole(80_000));
        assert_eq!(projection.rows[0].advance, whole(60_000));
        let mut result = FederalResult::empty(0);
        let outcome = crate::marketplace::calculate_months(&projection, 48_000, 2, &mut result);
        assert!(outcome.credit > 0);
    }

    #[test]
    fn no_final_family_attester_reconciles_advance_without_creating_credit() {
        // Reg1.36B-4(a)(1)(ii), attesting taxpayer for a person ultimately
        // in no tax family. Existing People identity is reused, not aliased.
        let mut f = facts(&[-1]);
        f.memberships.push(MarketplacePersonFamily {
            person: -1,
            family: MarketplaceNonreturnFamily::NoFamily,
        });
        let people = SourcePeople { family: &[], current: &[-1], births: &[] };
        assert!(project(&f, &people, false, 30_000).is_err());
        f.attestations
            .push(MarketplaceAttestation { person: -1, attester: MarketplaceAttester::ThisReturn });
        let p = project(&f, &people, false, 30_000).unwrap();
        assert!(!p.rows[0].eligible);
        assert_eq!(p.rows[0].premium, whole(0));
        assert_eq!(p.rows[0].advance, whole(60_000));
        let mut result = FederalResult::empty(0);
        let result = crate::marketplace::calculate_months(&p, 30_000, 1, &mut result);
        assert_eq!(result.credit, 0);
        assert_eq!(result.repayment, 600);
    }
    #[test]
    fn qsehra_self_only_affordability_bars_the_whole_family_and_annual_rounding_stays_annual() {
        // Derived IRC36B(c)(4)(A): self-only arrangement for taxpayer also
        // bars spouse's credit; covered-source subset does not narrow the bar.
        let mut f = facts(&[-1, -2]);
        f.qsehra.push(MarketplaceQsehra {
            id: 2,
            employee: -1,
            source: MarketplaceQsehraSource::Notice {
                employer: "Actual employer".into(),
                annual_permitted: 30_000,
            },
            months: vec![MarketplaceQsehraMonth {
                month: 1,
                self_only: 20_000,
                actual: 30_000,
                self_benchmark: 59_840,
                covered: vec![-1],
            }],
            determinations: vec![],
        });
        let people = SourcePeople { family: &[-1, -2], current: &[-1, -2], births: &[] };
        let projection = project(&f, &people, true, 48_000).unwrap();
        assert!(!projection.rows[0].eligible);
        assert_eq!(projection.rows[0].advance, whole(60_000));
        assert_eq!(projection.rows[0].medical_premium, whole(100_000));

        // Pub974 WorksheetQ PartIII applied to an annual Form8962 line11:
        // synthetic 2026 income$48,000/family1 gives annual contribution$4,781
        // and tentative annual credit$7,219. Jan–Aug benefits$330, Sep–Dec$670
        // cap against$7,219/12. Final credit$2,173, not twelve monthly Form8962
        // rounds ($2,176), nor $7,219−all$5,320 benefits ($1,899).
        let annual = CalculatedMarketplace {
            residence: MarketplaceResidence::Contiguous,
            shared: false,
            applicable_taxpayer: true,
            below_fpl_qualified: false,
            rows: (1..=12)
                .map(|month| CalculatedMonth {
                    month,
                    premium: whole(100_000),
                    medical_premium: whole(100_000),
                    benchmark: whole(100_000),
                    advance: whole(60_000),
                    eligible: true,
                    credit_reduction: whole(if month <= 8 { 33_000 } else { 67_000 }),
                })
                .collect(),
        };
        let mut result = FederalResult::empty(0);
        let outcome = crate::marketplace::calculate_months(&annual, 48_000, 1, &mut result);
        assert_eq!(outcome.credit, 2_173);
        assert_eq!(outcome.repayment, 5_027);
        assert!(outcome.monthly.is_none());
    }
}
