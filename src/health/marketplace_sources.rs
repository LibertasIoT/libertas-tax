// Shape and reference validation for canonical Marketplace facts. Both saved
// pages and transient estimates use these rules; echoed picker catalogs never
// establish ownership. Tax eligibility is calculated separately from validity.
use crate::*;
use alloc::collections::BTreeSet;

// Arithmetic exclusion: CalculationMoney::units projects annual premium/APTC
// pools into native interim Cents; MarketplaceDependent::income_cents and the
// federal household pool also still add source values in i64 before projection.
const MAX_MONEY: i64 = 100_000_000_000;
fn money(value: i64) -> bool {
    (0..=MAX_MONEY).contains(&value)
}
fn text(value: &str) -> bool {
    !value.trim().is_empty()
}
fn unique<T, K: Ord>(rows: &[T], key: impl Fn(&T) -> K) -> bool {
    let mut seen = BTreeSet::new();
    rows.iter().all(|r| seen.insert(key(r)))
}
fn months<T>(rows: &[T], month: impl Fn(&T) -> u32) -> bool {
    rows.iter().all(|r| (1..=12).contains(&month(r))) && unique(rows, month)
}

pub(crate) fn coverage_pending(policy: &MarketplacePolicyRecord) -> bool {
    crate::MarketplacePolicyRecordPage::from(&policy.form).status
        != MarketplaceStatementStatus::Void
        && (policy.coverage.is_empty()
            || (1..=12).any(|month| {
                policy_month(&crate::MarketplacePolicyRecordPage::from(&policy.form).amounts, month)
                    .is_some()
                    && !policy.coverage.iter().any(|coverage| coverage.month == month)
            })
            || policy.coverage.iter().any(|coverage| {
                policy_month(
                    &crate::MarketplacePolicyRecordPage::from(&policy.form).amounts,
                    coverage.month,
                )
                .is_none()
            }))
}

pub(crate) fn policy_answer_issue(
    policy: &MarketplacePolicyRecord,
) -> Option<crate::validation::ValidationIssue> {
    if coverage_pending(policy) {
        return Some(marketplace_issue("coverage"));
    }
    for (index, allocation) in policy.allocation.iter().enumerate() {
        if matches!(allocation.kind, MarketplaceAllocationKind::NoAdvance { .. })
            && (allocation.from..=allocation.to).any(|month| {
                policy_month(&crate::MarketplacePolicyRecordPage::from(&policy.form).amounts, month)
                    .is_some_and(|row| row.2 != 0)
            })
        {
            return Some(marketplace_issue("kind").in_list("allocation", index));
        }
    }
    for (index, lawful) in policy.lawful_premiums.iter().enumerate() {
        if let MarketplaceLawfulPremiumSource::ReferenceMonth { month } = lawful.source
            && policy_month(&crate::MarketplacePolicyRecordPage::from(&policy.form).amounts, month)
                .is_none()
        {
            return Some(marketplace_issue("source").in_list("lawful_premiums", index));
        }
    }
    for (index, payment) in policy.payments.iter().enumerate() {
        let invalid = policy_month(
            &crate::MarketplacePolicyRecordPage::from(&policy.form).amounts,
            payment.month,
        )
        .is_none_or(|(premium, _, advance)| {
            (premium != payment.paid + payment.unpaid
                && premium != payment.paid + payment.unpaid - payment.refunded)
                || matches!(payment.coverage,
                    MarketplacePaymentCoverage::NoQualifyingCoverage { issuer_returned_advance }
                    if issuer_returned_advance > advance)
        });
        if invalid {
            return Some(marketplace_issue("paid").in_list("payments", index));
        }
    }
    None
}

fn row_issue(
    premium: i64,
    benchmark: Option<i64>,
    advance: i64,
) -> Option<crate::validation::ValidationIssue> {
    let invalid = |field| {
        crate::validation::ValidationIssue::new(
            "MESSAGE_CHECK_THE_MONTHS_AND_AMOUNTS_OF_EACH_FORM_1095_A",
        )
        .at_field(field)
    };
    if !money(premium) {
        Some(invalid("premium"))
    } else if !benchmark.is_none_or(money) {
        Some(invalid("benchmark"))
    } else if !money(advance) {
        Some(invalid("advance"))
    } else if advance > premium {
        Some(
            crate::validation::ValidationIssue::new("MARKETPLACE_ADVANCE_ABOVE_PREMIUM")
                .at_field("advance"),
        )
    } else {
        None
    }
}
pub(crate) fn amounts_issue(value: &PolicyAmounts) -> Option<crate::validation::ValidationIssue> {
    let invalid = |field| {
        crate::validation::ValidationIssue::new(
            "MESSAGE_CHECK_THE_MONTHS_AND_AMOUNTS_OF_EACH_FORM_1095_A",
        )
        .at_field(field)
    };
    match value {
        PolicyAmounts::AllYear { premium, benchmark, advance } => {
            row_issue(*premium, *benchmark, *advance)
        }
        PolicyAmounts::ByMonth { months: rows } => {
            if rows.is_empty() {
                return Some(invalid("months"));
            }
            if let Some((index, _)) = rows.iter().enumerate().find(|(index, row)| {
                !(1..=12).contains(&row.month)
                    || rows[..*index].iter().any(|prior| prior.month == row.month)
            }) {
                return Some(invalid("month").in_list("months", index));
            }
            rows.iter().enumerate().find_map(|(index, row)| {
                row_issue(row.premium, row.benchmark, row.advance)
                    .map(|issue| issue.in_list("months", index))
            })
        }
    }
}
fn paper_months(paper: &Form1095APaper) -> [(&'static str, &Option<Form1095ACoverageAmounts>); 12] {
    [
        ("january", &paper.january),
        ("february", &paper.february),
        ("march", &paper.march),
        ("april", &paper.april),
        ("may", &paper.may),
        ("june", &paper.june),
        ("july", &paper.july),
        ("august", &paper.august),
        ("september", &paper.september),
        ("october", &paper.october),
        ("november", &paper.november),
        ("december", &paper.december),
    ]
}
pub(crate) fn paper_issue(paper: &Form1095APaper) -> Option<crate::validation::ValidationIssue> {
    paper_months(paper).into_iter().find_map(|(field, row)| {
        let row = row.as_ref()?;
        row_issue(row.premium.unwrap_or(0), row.benchmark, row.advance.unwrap_or(0))
            .map(|issue| issue.in_field(field))
    })
}
pub(crate) fn dependent_income_issue(
    value: &MarketplaceDependent,
) -> Option<crate::validation::ValidationIssue> {
    (value.taxable_social > value.social).then(|| {
        crate::validation::ValidationIssue::new("MARKETPLACE_TAXABLE_SOCIAL_ABOVE_GROSS")
            .at_field("taxable_social")
    })
}
// The editable source relationships whose bool results previously lost their
// own key and actual submitted row before Marketplace authorization.
pub(crate) fn entered_amounts_issue(
    facts: &MarketplaceFacts,
) -> Option<crate::validation::ValidationIssue> {
    facts
        .forms
        .iter()
        .enumerate()
        .find_map(|(index, form)| {
            let issue = if paper_months(&form.form).into_iter().all(|(_, row)| row.is_none()) {
                amounts_issue(&PolicyAmounts::ByMonth { months: alloc::vec![] })
                    .map(|issue| issue.at_record())
            } else {
                paper_issue(&form.form)
            };
            issue.map(|issue| issue.in_field("value").in_list("forms", index))
        })
        .or_else(|| {
            facts.dependents.iter().enumerate().find_map(|(index, person)| {
                dependent_income_issue(person).map(|issue| issue.in_list("dependents", index))
            })
        })
}

pub(crate) fn policy_month(value: &PolicyAmounts, month: u32) -> Option<(i64, Option<i64>, i64)> {
    if !(1..=12).contains(&month) {
        return None;
    }
    match value {
        PolicyAmounts::AllYear { premium, benchmark, advance } => {
            Some((*premium, *benchmark, *advance))
        }
        PolicyAmounts::ByMonth { months } => {
            months.iter().find(|r| r.month == month).map(|r| (r.premium, r.benchmark, r.advance))
        }
    }
}

fn family(value: &MarketplaceFamilyRef, facts: &MarketplaceFacts) -> bool {
    match value {
        MarketplaceFamilyRef::ThisReturn => true,
        MarketplaceFamilyRef::Other { family } => {
            facts.other_families.iter().any(|r| r.id == *family)
        }
    }
}
fn person(value: i64, facts: &MarketplaceFacts, people: &[i64]) -> bool {
    people.contains(&value) || facts.covered.iter().any(|r| r.id == value)
}
fn people_valid(values: &[i64], facts: &MarketplaceFacts, people: &[i64], nonempty: bool) -> bool {
    (!nonempty || !values.is_empty())
        && unique(values, |v| *v)
        && values.iter().all(|v| person(*v, facts, people))
}
pub(crate) fn benchmark(value: &MarketplaceBenchmarkSource, facts: &MarketplaceFacts) -> bool {
    match value {
        MarketplaceBenchmarkSource::Marketplace { amount } => money(*amount),
        MarketplaceBenchmarkSource::Statement { policy } => {
            facts.forms.iter().any(|r| r.id == *policy)
        }
        MarketplaceBenchmarkSource::ReferenceMonth { policy, month } => {
            facts.forms.iter().any(|r| {
                r.id == *policy
                    && policy_month(
                        &crate::MarketplacePolicyRecordPage::from(&r.form).amounts,
                        *month,
                    )
                    .is_some()
            })
        }
    }
}
fn agreement(value: &MarketplaceAgreement) -> bool {
    match value {
        MarketplaceAgreement::NoAgreement => true,
        MarketplaceAgreement::Agreed { share } => share.exact().is_some(),
    }
}
fn allocation(
    value: &MarketplaceAllocationKind,
    facts: &MarketplaceFacts,
    _enrolling: &MarketplaceFamilyRef,
) -> bool {
    use MarketplaceAllocationKind as K;
    match value {
        K::FormerSpouses { pair, agreement: share, third_families } => {
            family(&pair.one, facts)
                && family(&pair.two, facts)
                && pair.one != pair.two
                && agreement(share)
                && marital_families(third_families, pair, facts)
        }
        K::MarriedSeparate { pair, third_families, one_benchmark, two_benchmark } => {
            family(&pair.one, facts)
                && family(&pair.two, facts)
                && pair.one != pair.two
                && marital_families(third_families, pair, facts)
                && benchmark(one_benchmark, facts)
                && benchmark(two_benchmark, facts)
        }
        K::NoAdvance { families } => {
            !families.is_empty()
                && family_rows_unique(families, |r| &r.family)
                && families.iter().all(|r| family(&r.family, facts) && money(r.benchmark))
                && families.iter().any(|r| r.benchmark > 0)
        }
        K::OtherShared { families } => {
            !families.is_empty()
                && family_rows_unique(families, |r| &r.family)
                && families.iter().all(|r| family(&r.family, facts) && agreement(&r.agreement))
        }
    }
}
fn family_rows_unique<T>(rows: &[T], key: impl Fn(&T) -> &MarketplaceFamilyRef) -> bool {
    rows.iter().enumerate().all(|(i, r)| !rows[..i].iter().any(|p| key(p) == key(r)))
}
fn marital_families(
    rows: &[MarketplaceMaritalFamilyShare],
    pair: &MarketplaceMaritalPair,
    facts: &MarketplaceFacts,
) -> bool {
    family_rows_unique(rows, |r| &r.family)
        && rows.iter().all(|r| {
            family(&r.family, facts)
                && r.family != pair.one
                && r.family != pair.two
                && agreement(&r.one)
                && agreement(&r.two)
        })
}

// `people` and W2 identities are regenerated from the current accepted return.
// Candidate-local IDs are assigned before this post-pruning graph check.
pub(crate) fn valid(facts: &MarketplaceFacts, people: &[i64], w2: &[(i64, i64, i64)]) -> bool {
    valid_issue(facts, people, w2).is_none()
}
pub(crate) fn valid_issue(
    facts: &MarketplaceFacts,
    people: &[i64],
    w2: &[(i64, i64, i64)],
) -> Option<crate::validation::ValidationIssue> {
    with_catalog_issue(facts, facts, people, w2)
}

// Existing picker identities authorize links; the candidate supplies each
// linked source's proposed contents. A new row must be saved before another
// row can select it, exactly as the ordinary editor's accepted catalog works.
#[cfg(test)]
pub(crate) fn authorized(
    facts: &MarketplaceFacts,
    accepted: &MarketplaceFacts,
    people: &[i64],
    w2: &[(i64, i64, i64)],
) -> bool {
    authorized_issue(facts, accepted, people, w2).is_none()
}
pub(crate) fn authorized_issue(
    facts: &MarketplaceFacts,
    accepted: &MarketplaceFacts,
    people: &[i64],
    w2: &[(i64, i64, i64)],
) -> Option<crate::validation::ValidationIssue> {
    let mut catalog = facts.clone();
    catalog.forms.retain(|r| accepted.forms.iter().any(|p| p.id == r.id));
    catalog.other_families.retain(|r| accepted.other_families.iter().any(|p| p.id == r.id));
    catalog.covered.retain(|r| accepted.covered.iter().any(|p| p.id == r.id));
    with_catalog_issue(facts, &catalog, people, w2)
}

fn catalog_valid(
    facts: &MarketplaceFacts,
    catalog: &MarketplaceFacts,
    people: &[i64],
    w2: &[(i64, i64, i64)],
) -> bool {
    let identities: alloc::vec::Vec<_> = facts
        .forms
        .iter()
        .map(|r| r.id)
        .chain(facts.other_families.iter().map(|r| r.id))
        .chain(facts.covered.iter().map(|r| r.id))
        .chain(facts.qsehra.iter().map(|r| r.id))
        .collect();
    if identities.iter().any(|id| *id <= 0 || people.contains(id))
        || !unique(&identities, |id| *id)
        || facts.other_families.iter().any(|r| !text(&r.label))
        || facts.covered.iter().any(|r| match &r.identity {
            MarketplacePersonIdentity::OtherFamily { family: id, label } => {
                !catalog.other_families.iter().any(|f| f.id == *id) || !text(label)
            }
            MarketplacePersonIdentity::NoFamily { label } => !text(label),
        })
    {
        return false;
    }
    if facts.forms.iter().any(|p| {
        !text(&p.form.issuer)
            || !family(&p.enrolling, catalog)
            || !months(&p.coverage, |r| r.month)
            || p.coverage.iter().any(|r| {
                !people_valid(&r.people, catalog, people, true)
                    || !unique(&r.entry, |e| e.person)
                    || r.entry.iter().any(|e| {
                        !r.people.contains(&e.person)
                            || !crate::federal_rules::valid_date(e.start)
                            || !crate::federal_rules::valid_date(e.end)
                            || e.start < crate::TAX_YEAR_START_DATE
                            || e.end > crate::TAX_YEAR_END_DATE
                            || match &e.basis {
                                MarketplaceEnrollmentBasis::Adoption { event }
                                | MarketplaceEnrollmentBasis::Foster { event }
                                | MarketplaceEnrollmentBasis::CourtOrder { event }
                                | MarketplaceEnrollmentBasis::QualifyingSpecialEnrollment {
                                    event,
                                    ..
                                } => {
                                    !crate::federal_rules::valid_date(*event)
                                        || !(crate::TAX_YEAR_START_DATE..=crate::TAX_YEAR_END_DATE)
                                            .contains(event)
                                }
                                _ => false,
                            }
                    })
            })
            || p.allocation.iter().enumerate().any(|(i, a)| {
                !(1..=12).contains(&a.from)
                    || !(1..=12).contains(&a.to)
                    || !allocation(&a.kind, catalog, &p.enrolling)
                    || p.allocation[..i]
                        .iter()
                        .any(|prior| a.from <= prior.to && prior.from <= a.to)
            })
            || !months(&p.lawful_premiums, |r| r.month)
            || p.lawful_premiums.iter().any(|r| {
                !people_valid(&r.people, catalog, people, true)
                    || r.paid.is_some_and(|paid| !money(paid))
                    || !p.coverage.iter().any(|c| {
                        c.month == r.month && r.people.iter().all(|v| c.people.contains(v))
                    })
                    || match r.source {
                        MarketplaceLawfulPremiumSource::Marketplace { amount } => !money(amount),
                        MarketplaceLawfulPremiumSource::ReferenceMonth { .. } => false,
                    }
            })
            || !months(&p.payments, |r| r.month)
            || !months(&p.medical_costs, |r| r.month)
            || p.medical_costs.iter().any(|r| {
                !p.coverage.iter().any(|c| c.month == r.month)
                    || r.child_increase.is_some_and(|v| !money(v))
                    || r.tax_free_child.is_some_and(|v| !money(v))
            })
            || p.payments.iter().any(|r| {
                !money(r.paid)
                    || !money(r.refunded)
                    || !money(r.unpaid)
                    || !money(r.tax_free_premiums)
                    || r.tax_free_coverage.is_some_and(|amount| !money(amount))
                    || r.refunded > r.paid
                    || match &r.coverage {
                        MarketplacePaymentCoverage::EmergencyOrder { order_date, .. } => {
                            !crate::federal_rules::valid_date(*order_date)
                        }
                        MarketplacePaymentCoverage::NoQualifyingCoverage {
                            issuer_returned_advance,
                        } => !money(*issuer_returned_advance),
                        _ => false,
                    }
                    || r.appeal.as_ref().is_some_and(|a| {
                        !crate::federal_rules::valid_date(a.determined)
                            || crate::dates::add_days_checked(a.determined, 120).is_none()
                            || !crate::federal_rules::valid_date(a.enrolled)
                            || a.paid_on.is_some_and(|d| !crate::federal_rules::valid_date(d))
                    })
            })
    }) {
        return false;
    }
    if !unique(&facts.memberships, |r| r.person)
        || facts.memberships.iter().any(|r| {
            !people.contains(&r.person)
                || match r.family {
                    MarketplaceNonreturnFamily::OtherFamily { family } => {
                        !catalog.other_families.iter().any(|f| f.id == family)
                    }
                    MarketplaceNonreturnFamily::NoFamily => false,
                }
        })
    {
        return false;
    }
    if !unique(&facts.attestations, |r| r.person)
        || facts.attestations.iter().any(|r| {
            !person(r.person, catalog, people)
                || match r.attester {
                    MarketplaceAttester::OtherFamily { family: id } => {
                        !catalog.other_families.iter().any(|f| f.id == id)
                    }
                    _ => false,
                }
        })
        || !unique(&facts.eligibility, |r| (r.person, r.month))
        || facts
            .eligibility
            .iter()
            .any(|r| !(1..=12).contains(&r.month) || !person(r.person, catalog, people))
        || !months(&facts.benchmarks, |r| r.month)
        || facts.benchmarks.iter().any(|r| {
            !unique(&r.allocated, |a| a.policy)
                || r.groups.iter().any(|g| {
                    !text(&g.rating_area)
                        || !people_valid(&g.people, catalog, people, true)
                        || !benchmark(&g.source, catalog)
                })
                || r.groups.iter().enumerate().any(|(i, g)| {
                    r.groups[..i].iter().any(|p| p.people.iter().any(|v| g.people.contains(v)))
                })
                || r.allocated.iter().any(|a| {
                    !catalog.forms.iter().any(|p| p.id == a.policy)
                        || !benchmark(&a.source, catalog)
                })
        })
    {
        return false;
    }
    if !crate::marketplace_qsehra::employee_facts_valid(facts, people) || facts.qsehra.iter().enumerate().any(|(i, q)| {
        matches!(q.source, MarketplaceQsehraSource::W2 { document } if facts.qsehra[..i].iter().any(|prior| matches!(prior.source, MarketplaceQsehraSource::W2 { document: other } if other == document))) ||
        !people.contains(&q.employee)
            || q.months.is_empty() || !months(&q.months, |r| r.month)
            || q.months.iter().any(|r| !money(r.self_only) || !money(r.actual) || !money(r.self_benchmark)
                || !people_valid(&r.covered, catalog, people, true))
            || match &q.source {
                MarketplaceQsehraSource::W2 { document } => !w2.iter().any(|(id, owner, annual)| {
                    id == document && *owner == q.employee && *annual == q.months.iter().map(|r| r.actual).sum::<i64>()
                }),
                MarketplaceQsehraSource::Notice { employer, annual_permitted } => !text(employer) || !money(*annual_permitted),
            }
            || q.determinations.iter().enumerate().any(|(i, r)| {
                !(1..=12).contains(&r.from) || !(1..=12).contains(&r.to) || !crate::federal_rules::valid_date(r.made_on)
                    || q.determinations[..i].iter().any(|p| r.made_on == p.made_on && r.from <= p.to && p.from <= r.to)
            })
    }) { return false; }
    if let MarketplaceMarriage::Alternative { wedding, you, spouse, .. } = &facts.marriage
        && (!crate::federal_rules::valid_date(*wedding)
            || !(crate::TAX_YEAR_START_DATE..=crate::TAX_YEAR_END_DATE).contains(wedding)
            || [you, spouse].iter().any(|f| {
                !months(&f.benchmarks, |r| r.month)
                    || f.benchmarks.iter().any(|r| {
                        !unique(&r.allocated, |a| a.policy)
                            || r.groups.iter().any(|g| {
                                !text(&g.rating_area)
                                    || !people_valid(&g.people, catalog, people, true)
                                    || !benchmark(&g.source, catalog)
                            })
                            || r.groups.iter().enumerate().any(|(i, g)| {
                                r.groups[..i]
                                    .iter()
                                    .any(|p| p.people.iter().any(|v| g.people.contains(v)))
                            })
                            || r.allocated.iter().any(|a| {
                                !catalog.forms.iter().any(|p| p.id == a.policy)
                                    || !benchmark(&a.source, catalog)
                            })
                    })
                    || !unique(&f.dependents, |v| *v)
                    || f.dependents.iter().any(|v| *v <= 0 || !people.contains(v))
                    || !unique(&f.premarriage_policies, |v| *v)
                    || f.premarriage_policies
                        .iter()
                        .any(|v| !catalog.forms.iter().any(|p| p.id == *v))
            })
            || you.dependents.iter().any(|v| spouse.dependents.contains(v))
            || you.premarriage_policies.iter().any(|v| spouse.premarriage_policies.contains(v)))
    {
        return false;
    }
    unique(&facts.dependent_qsehra_employees, |v| *v)
        && facts.dependent_qsehra_employees.iter().all(|id| *id > 0 && people.contains(id))
        && unique(&facts.dependents, |r| r.person)
        && facts.dependents.iter().all(|r| {
            r.person > 0
                && people.contains(&r.person)
                && (-MAX_MONEY..=MAX_MONEY).contains(&r.agi)
                && [r.exempt_interest, r.foreign_exclusion, r.social, r.taxable_social]
                    .iter()
                    .all(|v| money(*v))
        })
}

// Source-derived read-only values are validated before page-to-fact conversion
// drops them. Hidden catalog echoes are intentionally absent from this check.
pub(crate) fn page_readonly_issue(
    facts: &MarketplaceFactsPage,
) -> Option<crate::validation::ValidationIssue> {
    for (form_index, form) in facts.forms.iter().enumerate() {
        for (payment_index, payment) in form.payments.iter().enumerate() {
            if payment.appeal.as_ref().is_some_and(|appeal| {
                crate::dates::add_days_checked(appeal.determined, 120)
                    != Some(appeal.required_deadline)
            }) {
                return Some(
                    marketplace_issue("required_deadline")
                        .in_field("appeal")
                        .in_list("payments", payment_index)
                        .in_list("forms", form_index),
                );
            }
        }
    }
    None
}
#[cfg(test)]
pub(crate) fn page_readonly_valid(facts: &MarketplaceFactsPage) -> bool {
    page_readonly_issue(facts).is_none()
}

fn marketplace_issue(field: &'static str) -> crate::validation::ValidationIssue {
    crate::validation::ValidationIssue::new(
        "MESSAGE_CHECK_THE_MONTHS_AND_AMOUNTS_OF_EACH_FORM_1095_A",
    )
    .at_field(field)
}
fn with_catalog_issue(
    facts: &MarketplaceFacts,
    catalog: &MarketplaceFacts,
    people: &[i64],
    w2: &[(i64, i64, i64)],
) -> Option<crate::validation::ValidationIssue> {
    source_issue(facts).or_else(|| {
        (!catalog_valid(facts, catalog, people, w2)).then(|| marketplace_issue("forms"))
    })
}
fn agreement_issue(value: &MarketplaceAgreement) -> Option<crate::validation::ValidationIssue> {
    match value {
        MarketplaceAgreement::NoAgreement => None,
        MarketplaceAgreement::Agreed { share } => {
            share.issue().map(|issue| issue.in_field("share"))
        }
    }
}
fn allocation_source_issue(
    value: &MarketplaceAllocationKind,
    enrolling: &MarketplaceFamilyRef,
) -> Option<crate::validation::ValidationIssue> {
    match value {
        MarketplaceAllocationKind::OtherShared { families } => {
            crate::marketplace_allocation::entered_shares_issue(families, enrolling)
        }
        MarketplaceAllocationKind::FormerSpouses { agreement, third_families, .. } => {
            agreement_issue(agreement).map(|issue| issue.in_field("agreement")).or_else(|| {
                third_families.iter().enumerate().find_map(|(index, row)| {
                    agreement_issue(&row.one)
                        .map(|issue| issue.in_field("one"))
                        .or_else(|| agreement_issue(&row.two).map(|issue| issue.in_field("two")))
                        .map(|issue| issue.in_list("third_families", index))
                })
            })
        }
        MarketplaceAllocationKind::MarriedSeparate { third_families, .. } => {
            third_families.iter().enumerate().find_map(|(index, row)| {
                agreement_issue(&row.one)
                    .map(|issue| issue.in_field("one"))
                    .or_else(|| agreement_issue(&row.two).map(|issue| issue.in_field("two")))
                    .map(|issue| issue.in_list("third_families", index))
            })
        }
        MarketplaceAllocationKind::NoAdvance { .. } => None,
    }
}
// Actual editable sources, before IDs/catalog authorization. Returned paths
// use the Page's matching forms/coverage/entry/allocation/payments fields.
pub(crate) fn source_issue(facts: &MarketplaceFacts) -> Option<crate::validation::ValidationIssue> {
    if let Some(issue) = entered_amounts_issue(facts) {
        return Some(issue);
    }
    for (form_index, form) in facts.forms.iter().enumerate() {
        let in_form =
            |issue: crate::validation::ValidationIssue| issue.in_list("forms", form_index);
        for (coverage_index, coverage) in form.coverage.iter().enumerate() {
            for (entry_index, entry) in coverage.entry.iter().enumerate() {
                if entry.start > entry.end {
                    return Some(in_form(
                        crate::validation::ValidationIssue::new("PTC_COVERAGE_DATES_REVERSED")
                            .at_field("end")
                            .in_list("entry", entry_index)
                            .in_list("coverage", coverage_index),
                    ));
                }
            }
        }
        for (index, allocation) in form.allocation.iter().enumerate() {
            if allocation.from > allocation.to {
                return Some(in_form(
                    crate::validation::ValidationIssue::new("PTC_MONTHS_REVERSED")
                        .at_field("to")
                        .in_list("allocation", index),
                ));
            }
            if let Some(issue) = allocation_source_issue(&allocation.kind, &form.enrolling) {
                return Some(in_form(issue.in_field("kind").in_list("allocation", index)));
            }
        }
        for (index, costs) in form.medical_costs.iter().enumerate() {
            if costs
                .tax_free_child
                .is_some_and(|amount| costs.child_increase.is_none_or(|cost| amount > cost))
            {
                return Some(in_form(
                    crate::validation::ValidationIssue::new(
                        "HEALTH_POLICY_CHILD_FUNDING_ABOVE_COST",
                    )
                    .at_field("tax_free_child")
                    .in_list("medical_costs", index),
                ));
            }
        }
        for (index, payment) in form.payments.iter().enumerate() {
            if payment.tax_free_premiums > payment.paid - payment.refunded {
                return Some(in_form(
                    crate::validation::ValidationIssue::new("PTC_TAX_FREE_PREMIUMS_ABOVE_NET_PAID")
                        .at_field("tax_free_premiums")
                        .in_list("payments", index),
                ));
            }
            if payment.tax_free_coverage.is_some_and(|amount| amount > payment.tax_free_premiums) {
                return Some(in_form(
                    crate::validation::ValidationIssue::new("PTC_TAX_FREE_COVERAGE_ABOVE_TOTAL")
                        .at_field("tax_free_coverage")
                        .in_list("payments", index),
                ));
            }
        }
    }
    if let Some(issue) = crate::marketplace_qsehra::entered_issue(facts) {
        return Some(issue);
    }
    for (index, qsehra) in facts.qsehra.iter().enumerate() {
        for (determination_index, determination) in qsehra.determinations.iter().enumerate() {
            if determination.from > determination.to {
                return Some(
                    crate::validation::ValidationIssue::new("PTC_MONTHS_REVERSED")
                        .at_field("to")
                        .in_list("determinations", determination_index)
                        .in_list("qsehra", index),
                );
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{string::String, vec};

    fn policy(id: i64) -> MarketplacePolicyRecord {
        MarketplacePolicyRecord {
            printed: None,
            id,
            form: crate::MarketplacePolicyRecordPage {
                recipient: Default::default(),
                id,
                issuer: "Issuer".into(),
                policy_number: String::new(),
                amounts: PolicyAmounts::ByMonth {
                    months: vec![PolicyMonth {
                        month: 1,
                        premium: 100_000,
                        benchmark: None,
                        advance: 60_000,
                    }],
                },
                status: MarketplaceStatementStatus::Original,
            }
            .into(),
            lawful_premiums: vec![],
            enrolling: MarketplaceFamilyRef::ThisReturn,
            coverage: vec![MarketplacePolicyCoverage { month: 1, people: vec![-1], entry: vec![] }],
            allocation: vec![],
            payments: vec![],
            medical_costs: vec![],
        }
    }

    #[test]
    fn links_use_saved_source_identity_and_candidate_contents() {
        let mut accepted =
            MarketplaceFacts { forms: vec![policy(1), policy(2)], ..MarketplaceFacts::default() };
        let correction = MarketplaceBenchmarkMonth {
            month: 1,
            groups: vec![MarketplaceBenchmarkGroup {
                state: TaxState::CA,
                rating_area: "Actual area".into(),
                people: vec![-1],
                source: MarketplaceBenchmarkSource::Statement { policy: 1 },
            }],
            allocated: vec![],
        };
        accepted.benchmarks.push(correction);
        assert!(valid(&accepted, &[-1], &[]));
        let mut candidate = accepted.clone();
        candidate.forms.reverse();
        candidate.forms.iter_mut().find(|p| p.id == 1).unwrap().form.issuer = "Renamed".into();
        assert!(authorized(&candidate, &accepted, &[-1], &[]));
        // An added source can be saved, but a guessed new ID cannot authorize
        // a linked selector in that same request.
        candidate.forms.push(policy(3));
        assert!(authorized(&candidate, &accepted, &[-1], &[]));
        candidate.benchmarks[0].groups[0].source =
            MarketplaceBenchmarkSource::Statement { policy: 3 };
        assert!(valid(&candidate, &[-1], &[]));
        assert!(!authorized(&candidate, &accepted, &[-1], &[]));
        // Post-pruning graph validation permits removing both sides together.
        candidate.forms.clear();
        assert!(!valid(&candidate, &[-1], &[]));
        candidate.benchmarks.clear();
        assert!(valid(&candidate, &[-1], &[]));
    }

    #[test]
    fn blank_benchmark_and_readonly_appeal_deadline_round_trip() {
        let mut facts = MarketplaceFacts { forms: vec![policy(1)], ..MarketplaceFacts::default() };
        facts.forms[0].payments.push(MarketplacePaymentMonth {
            tax_free_premiums: 0,
            tax_free_coverage: None,
            month: 1,
            paid: 100_000,
            refunded: 0,
            unpaid: 0,
            coverage: MarketplacePaymentCoverage::PaidByDeadline,
            appeal: Some(MarketplaceAppeal {
                determined: 20261231,
                enrolled: 20260101,
                paid_on: Some(20270430),
            }),
        });
        let page: MarketplaceFactsPage = facts.clone().into();
        assert!(page_readonly_valid(&page));
        assert_eq!(page.forms[0].payments[0].appeal.as_ref().unwrap().required_deadline, 20270430);
        let reopened: MarketplaceFacts = page.clone().into();
        assert_eq!(reopened, facts);
        assert_eq!(
            policy_month(&reopened.forms[0].form.amounts(), 1),
            Some((100_000, None, 60_000))
        );
        let mut forged = page;
        forged.forms[0].payments[0].appeal.as_mut().unwrap().required_deadline = 20270501;
        assert!(!page_readonly_valid(&forged));
        let paper = &mut facts.forms[0].form;
        paper.month_mut(1).benchmark = Some(0);
        assert_eq!(
            policy_month(&facts.forms[0].form.amounts(), 1),
            Some((100_000, Some(0), 60_000))
        );
    }

    #[test]
    fn qsehra_reads_one_employee_owned_w2_and_reconciles_current_benefit() {
        let mut facts = MarketplaceFacts::default();
        facts.qsehra.push(MarketplaceQsehra {
            id: 2,
            employee: -1,
            source: MarketplaceQsehraSource::W2 { document: 1 },
            months: vec![MarketplaceQsehraMonth {
                month: 1,
                self_only: 20_000,
                actual: 30_000,
                self_benchmark: 60_000,
                covered: vec![-1],
            }],
            determinations: vec![],
        });
        assert!(valid(&facts, &[-1, -2], &[(1, -1, 30_000)]));
        assert!(!valid(&facts, &[-1, -2], &[(1, -2, 30_000)]));
        assert!(!valid(&facts, &[-1, -2], &[(1, -1, 30_001)]));
        let mut duplicate = facts.qsehra[0].clone();
        duplicate.id = 3;
        facts.qsehra.push(duplicate);
        assert!(!valid(&facts, &[-1, -2], &[(1, -1, 30_000)]));
        facts.qsehra.pop();
        facts.qsehra[0].determinations = vec![
            MarketplaceQsehraDetermination {
                from: 1,
                to: 12,
                accurate_disclosure: true,
                unaffordable: true,
                made_on: 20251201,
            },
            MarketplaceQsehraDetermination {
                from: 1,
                to: 12,
                accurate_disclosure: true,
                unaffordable: false,
                made_on: 20260115,
            },
        ];
        assert!(valid(&facts, &[-1, -2], &[(1, -1, 30_000)]));
        facts.qsehra[0].determinations[1].made_on = 20251201;
        assert!(!valid(&facts, &[-1, -2], &[(1, -1, 30_000)]));
        facts.qsehra[0].determinations.clear();
        facts.qsehra[0].months[0].covered.push(99);
        assert!(!valid(&facts, &[-1, -2], &[(1, -1, 30_000)]));
    }
}
