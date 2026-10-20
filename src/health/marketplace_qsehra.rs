// IRC36B(c)(4); Notice2017-67, pp40–46, Q65–69; Pub974 (2025), pp14–18.
// https://www.irs.gov/pub/irs-drop/n-17-67.pdf
// https://www.irs.gov/pub/irs-drop/rp-25-25.pdf (2026: 9.96%)
// Affordability uses the age-applicable SELF-ONLY benefit. Credit reduction
// uses the ACTUAL permitted self/family benefit, regardless of reimbursements.
use crate::kernel::phase;
use crate::kernel::ratio::exact::{Exact, ratio, whole};
use crate::{
    MarketplaceFacts, MarketplaceQsehra, MarketplaceQsehraEmployeeHousehold,
    MarketplaceQsehraIncome, MarketplaceQsehraIncomeSource,
};
use alloc::vec::Vec;

pub(crate) struct Assessment {
    pub affordable: bool,
    pub permitted: Exact,
}

// Direct notice facts have the same client and normal Save constraints. A
// dependent's W-2 belongs to that dependent's return, not either filer-owned
// source catalog; its actual employer notice supplies this arrangement.
#[cfg(test)]
pub(crate) fn entered_rule(facts: &crate::MarketplaceFacts) -> Option<&'static str> {
    entered_issue(facts).map(|issue| issue.key)
}

pub(crate) fn entered_issue(
    facts: &crate::MarketplaceFacts,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    for (index, q) in facts.qsehra.iter().enumerate() {
        if q.employee > 0 && !matches!(q.source, crate::MarketplaceQsehraSource::Notice { .. }) {
            return Some(
                ValidationIssue::new("PTC_DEPENDENT_QSEHRA_NOTICE_REQUIRED")
                    .at_field("source")
                    .in_list("qsehra", index),
            );
        }
        if let crate::MarketplaceQsehraSource::Notice { annual_permitted, .. } = &q.source
            && q.months.iter().try_fold(0_i64, |sum, row| sum.checked_add(row.actual))
                != Some(*annual_permitted)
        {
            return Some(
                ValidationIssue::new("PTC_QSEHRA_BENEFIT_MISMATCH")
                    .at_field("months")
                    .in_list("qsehra", index),
            );
        }
    }
    for (index, e) in facts.employee_qsehra_facts.iter().enumerate() {
        let employee = |issue: ValidationIssue| issue.in_list("employee_qsehra_facts", index);
        if let Some(source) = &e.income {
            if components(facts, e.employee, source).is_none() {
                return Some(employee(
                    ValidationIssue::new("PTC_QSEHRA_EMPLOYEE_RETURN_REQUIRED").at_field("income"),
                ));
            }
            if let Some(issue) =
                income_issue(facts, e.employee, source, |issue| employee(issue.in_field("income")))
            {
                return Some(issue);
            }
        }
        let Some(household) = &e.household else {
            return Some(employee(
                ValidationIssue::new("PTC_DEPENDENT_QSEHRA_RECONCILIATION_REQUIRED")
                    .at_field("household"),
            ));
        };
        let MarketplaceQsehraEmployeeHousehold::Known { spouse, dependents } = household else {
            continue;
        };
        if e.income.is_none() {
            return Some(employee(
                ValidationIssue::new("PTC_DEPENDENT_QSEHRA_RECONCILIATION_REQUIRED")
                    .at_field("income"),
            ));
        }
        if spouse == &Some(e.employee) {
            return Some(employee(
                ValidationIssue::new("PTC_QSEHRA_RELATION_INVALID")
                    .at_field("spouse")
                    .in_field("household"),
            ));
        }
        if let Some((member_index, _)) = dependents.iter().enumerate().find(|(i, m)| {
            m.person == e.employee
                || spouse == &Some(m.person)
                || dependents[..*i].iter().any(|p| p.person == m.person)
        }) {
            return Some(employee(
                ValidationIssue::new("PTC_QSEHRA_RELATION_INVALID")
                    .at_field("person")
                    .in_list("dependents", member_index)
                    .in_field("household"),
            ));
        }
        for (member_index, member) in dependents.iter().enumerate() {
            if let Some(source) = &member.income {
                let entered = |issue: ValidationIssue| {
                    employee(
                        issue
                            .in_field("income")
                            .in_list("dependents", member_index)
                            .in_field("household"),
                    )
                };
                if components(facts, member.person, source).is_none() {
                    return Some(employee(
                        ValidationIssue::new("PTC_QSEHRA_EMPLOYEE_RETURN_REQUIRED")
                            .at_field("income")
                            .in_list("dependents", member_index)
                            .in_field("household"),
                    ));
                }
                if let Some(issue) = income_issue(facts, member.person, source, entered) {
                    return Some(issue);
                }
            }
        }
    }
    None
}

// §36B(c)(4)(C), Notice2017-67§K and Reg1.36B-1(e): affordability
// uses the employee's own household income. The claiming taxpayer's credit
// reduction under(c)(4)(B) is separate. Never substitute the parent's income.
fn components(
    facts: &MarketplaceFacts,
    person: i64,
    source: &MarketplaceQsehraIncomeSource,
) -> Option<MarketplaceQsehraIncome> {
    match source {
        MarketplaceQsehraIncomeSource::RequiredReturn => {
            facts.dependents.iter().find(|r| r.person == person).map(|r| MarketplaceQsehraIncome {
                agi: r.agi,
                exempt_interest: r.exempt_interest,
                foreign_exclusion: r.foreign_exclusion,
                social: r.social,
                taxable_social: r.taxable_social,
            })
        }
        MarketplaceQsehraIncomeSource::Documentary { components } => Some(components.clone()),
    }
}
fn income_issue(
    facts: &MarketplaceFacts,
    person: i64,
    source: &MarketplaceQsehraIncomeSource,
    entered: impl Fn(crate::validation::ValidationIssue) -> crate::validation::ValidationIssue,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    let income = components(facts, person, source)?;
    if income.taxable_social > income.social {
        let issue =
            ValidationIssue::new("PTC_QSEHRA_SOCIAL_ABOVE_GROSS").at_field("taxable_social");
        return Some(match source {
            MarketplaceQsehraIncomeSource::Documentary { .. } => {
                entered(issue.in_field("components"))
            }
            MarketplaceQsehraIncomeSource::RequiredReturn => {
                let index = facts.dependents.iter().position(|r| r.person == person)?;
                issue.in_list("dependents", index)
            }
        });
    }
    if let MarketplaceQsehraIncomeSource::Documentary { components: documentary } = source
        && let Some(existing) =
            components(facts, person, &MarketplaceQsehraIncomeSource::RequiredReturn)
        && existing != *documentary
    {
        return Some(entered(
            ValidationIssue::new("PTC_QSEHRA_RETURN_CONFLICT").at_field("components"),
        ));
    }
    None
}
fn source_income(
    facts: &MarketplaceFacts,
    person: i64,
    source: &MarketplaceQsehraIncomeSource,
) -> Option<Exact> {
    let v = components(facts, person, source)?;
    Some(
        whole(v.agi) + whole(v.exempt_interest) + whole(v.foreign_exclusion) + whole(v.social)
            - whole(v.taxable_social),
    )
}

pub(crate) fn employee_income(facts: &MarketplaceFacts, employee: i64) -> Option<Exact> {
    let e = facts.employee_qsehra_facts.iter().find(|e| e.employee == employee)?;
    let MarketplaceQsehraEmployeeHousehold::Known { dependents, .. } = e.household.as_ref()? else {
        return None;
    };
    let mut income = source_income(facts, employee, e.income.as_ref()?)?;
    for member in dependents {
        if let Some(source) = &member.income {
            income += source_income(facts, member.person, source)?;
        }
    }
    Some(income)
}

pub(crate) fn employee_related(facts: &MarketplaceFacts, employee: i64) -> Option<Vec<i64>> {
    let e = facts.employee_qsehra_facts.iter().find(|e| e.employee == employee)?;
    let MarketplaceQsehraEmployeeHousehold::Known { spouse, dependents } = e.household.as_ref()?
    else {
        return None;
    };
    let mut related = alloc::vec![employee];
    related.extend(spouse);
    related.extend(dependents.iter().map(|m| m.person));
    Some(related)
}

pub(crate) fn dependent_sources_complete(facts: &MarketplaceFacts) -> bool {
    facts
        .dependent_qsehra_employees
        .iter()
        .copied()
        .chain(facts.qsehra.iter().filter(|q| q.employee > 0).map(|q| q.employee))
        .all(|employee| {
            facts.qsehra.iter().any(|q| q.employee == employee)
                && employee_income(facts, employee).is_some()
                && employee_related(facts, employee).is_some()
        })
}

pub(crate) fn employee_facts_valid(facts: &MarketplaceFacts, people: &[i64]) -> bool {
    let income_valid = |source: &MarketplaceQsehraIncomeSource| match source {
        MarketplaceQsehraIncomeSource::RequiredReturn => true,
        MarketplaceQsehraIncomeSource::Documentary { components: v } => {
            [v.exempt_interest, v.foreign_exclusion, v.social, v.taxable_social]
                .iter()
                .all(|v| *v >= 0)
                && v.taxable_social <= v.social
        }
    };
    facts.employee_qsehra_facts.iter().enumerate().all(|(i, e)| {
        e.employee > 0
            && people.contains(&e.employee)
            && !facts.employee_qsehra_facts[..i].iter().any(|p| p.employee == e.employee)
            && e.income.as_ref().is_none_or(income_valid)
            && e.household.as_ref().is_none_or(|h| match h {
                MarketplaceQsehraEmployeeHousehold::OtherPeople => true,
                MarketplaceQsehraEmployeeHousehold::Known { spouse, dependents } => {
                    spouse.is_none_or(|id| people.contains(&id))
                        && dependents.iter().all(|m| {
                            people.contains(&m.person) && m.income.as_ref().is_none_or(income_valid)
                        })
                }
            })
    })
}

#[cfg(test)]
pub(crate) fn assess(q: &MarketplaceQsehra, month: u32, income: i64) -> Option<Assessment> {
    assess_with_income(q, month, &(whole(income) * whole(100)))
}
pub(crate) fn assess_with_income(
    q: &MarketplaceQsehra,
    month: u32,
    income: &Exact,
) -> Option<Assessment> {
    let source = q.months.iter().find(|r| r.month == month)?;
    let protected_unaffordable = q
        .determinations
        .iter()
        .filter(|r| (r.from..=r.to).contains(&month))
        .max_by_key(|r| r.made_on)
        .is_some_and(|r| r.accurate_disclosure && r.unaffordable);
    let required = whole(phase::floor(source.self_benchmark - source.self_only, 0));
    let limit = phase::floor(income.clone(), whole(0)) * ratio(996, 120_000).unwrap();
    Some(Assessment {
        affordable: !protected_unaffordable && required <= limit,
        permitted: whole(source.actual),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MarketplaceQsehraDetermination, MarketplaceQsehraMonth, MarketplaceQsehraSource};
    use alloc::vec;

    fn qsehra() -> MarketplaceQsehra {
        MarketplaceQsehra {
            id: 1,
            employee: -1,
            source: MarketplaceQsehraSource::Notice {
                employer: "Actual employer".into(),
                annual_permitted: 30_000,
            },
            months: vec![MarketplaceQsehraMonth {
                month: 1,
                self_only: 20_000,
                actual: 30_000,
                self_benchmark: 60_000,
                covered: vec![-1],
            }],
            determinations: vec![],
        }
    }

    #[test]
    fn affordability_keeps_cents_and_uses_self_only_benefit() {
        // Derived 2026 rule: income$48,000 *9.96%/12=$398.40;
        // $600−$200=$400 is unaffordable. Equality at$398.40 is affordable.
        let mut q = qsehra();
        assert!(!assess(&q, 1, 48_000).unwrap().affordable);
        q.months[0].self_benchmark = 59_840;
        assert!(assess(&q, 1, 48_000).unwrap().affordable);
        q.months[0].self_benchmark += 1;
        assert!(!assess(&q, 1, 48_000).unwrap().affordable);
        // Actual family benefit does not replace the self-only affordability
        // input, and remains the actual permitted-benefit reduction source.
        q.months[0].actual = 67_000;
        let result = assess(&q, 1, 48_000).unwrap();
        assert!(!result.affordable);
        assert_eq!(result.permitted, whole(67_000));
    }

    #[test]
    fn iterative_income_cents_reach_actual_qsehra_affordability() {
        let mut q = qsehra();
        q.months[0].self_benchmark = 59_841;
        assert!(!assess_with_income(&q, 1, &whole(4_800_120)).unwrap().affordable);
        assert!(assess_with_income(&q, 1, &whole(4_800_121)).unwrap().affordable);
        assert!(!assess(&q, 1, 48_001).unwrap().affordable);
    }

    #[test]
    fn marketplace_protection_requires_actual_accurate_determination() {
        // Notice2017-67 Q67, p44: protection lasts through the provided
        // months until an actual new Marketplace determination.
        let mut q = qsehra();
        q.months[0].self_benchmark = 40_000;
        assert!(assess(&q, 1, 48_000).unwrap().affordable);
        q.determinations.push(MarketplaceQsehraDetermination {
            from: 1,
            to: 12,
            accurate_disclosure: true,
            unaffordable: true,
            made_on: 20251201,
        });
        assert!(!assess(&q, 1, 48_000).unwrap().affordable);
        q.determinations[0].accurate_disclosure = false;
        assert!(assess(&q, 1, 48_000).unwrap().affordable);
        q.determinations[0].accurate_disclosure = true;
        q.determinations.push(MarketplaceQsehraDetermination {
            from: 1,
            to: 12,
            accurate_disclosure: true,
            unaffordable: false,
            made_on: 20260115,
        });
        assert!(assess(&q, 1, 48_000).unwrap().affordable);
        assert!(assess(&q, 2, 48_000).is_none());
    }

    #[test]
    fn partial_year_self_and_family_benefits_are_not_divided_by_twelve_again() {
        // Notice2017-67 Q68–69, pp44–46: Jan–Aug self$330/month,
        // Sep–Dec family$670/month gives annualFF$5,320. The self-only
        // affordability source remains$330, including family months.
        let mut q = qsehra();
        q.months = (1..=12)
            .map(|month| MarketplaceQsehraMonth {
                month,
                self_only: 33_000,
                actual: if month <= 8 { 33_000 } else { 67_000 },
                self_benchmark: 100_000,
                covered: vec![-1],
            })
            .collect();
        assert_eq!(q.months.iter().map(|r| r.actual).sum::<i64>(), 532_000);
        assert_eq!(assess(&q, 8, 48_000).unwrap().permitted, whole(33_000));
        assert_eq!(assess(&q, 9, 48_000).unwrap().permitted, whole(67_000));
        q.months.retain(|r| r.month >= 9);
        assert_eq!(q.months.iter().map(|r| r.actual).sum::<i64>(), 268_000);
        assert_eq!(assess(&q, 9, 48_000).unwrap().permitted, whole(67_000));
    }
    #[test]
    fn dependent_employee_household_uses_joint_base_and_only_required_member_magi() {
        use crate::{MarketplaceQsehraEmployeeFacts, MarketplaceQsehraHouseholdMember};
        let documentary = |agi, exempt_interest, foreign_exclusion, social, taxable_social| {
            MarketplaceQsehraIncomeSource::Documentary {
                components: MarketplaceQsehraIncome {
                    agi,
                    exempt_interest,
                    foreign_exclusion,
                    social,
                    taxable_social,
                },
            }
        };
        let mut facts = MarketplaceFacts::default();
        facts.employee_qsehra_facts.push(MarketplaceQsehraEmployeeFacts {
            employee: 1,
            // Joint-return AGI already contains both spouses; spouse2 is not
            // independently added again. Required dependent3 adds all MAGI
            // components, while dependent4 refund-only return adds nothing.
            income: Some(documentary(1_000_000, 20_000, 30_000, 40_000, 10_000)),
            household: Some(MarketplaceQsehraEmployeeHousehold::Known {
                spouse: Some(2),
                dependents: vec![
                    MarketplaceQsehraHouseholdMember {
                        person: 3,
                        income: Some(documentary(100_000, 2_000, 3_000, 4_000, 1_000)),
                    },
                    MarketplaceQsehraHouseholdMember { person: 4, income: None },
                ],
            }),
        });
        assert_eq!(entered_rule(&facts), None);
        assert!(employee_facts_valid(&facts, &[1, 2, 3, 4]));
        assert_eq!(employee_income(&facts, 1), Some(whole(1_188_000)));
        assert_eq!(employee_related(&facts, 1), Some(vec![1, 2, 3, 4]));
        // A person absent from the claimant People catalog must stay an
        // explicit missing employee context; never silently omit a relative.
        assert!(!employee_facts_valid(&facts, &[1, 2, 3]));
        facts.employee_qsehra_facts[0].household =
            Some(MarketplaceQsehraEmployeeHousehold::OtherPeople);
        assert_eq!(entered_rule(&facts), None);
        assert_eq!(employee_income(&facts, 1), None);
        assert_eq!(employee_related(&facts, 1), None);
        facts.employee_qsehra_facts[0].household =
            Some(MarketplaceQsehraEmployeeHousehold::Known { spouse: None, dependents: vec![] });
        facts.employee_qsehra_facts[0].income = Some(MarketplaceQsehraIncomeSource::RequiredReturn);
        assert_eq!(entered_rule(&facts), Some("PTC_QSEHRA_EMPLOYEE_RETURN_REQUIRED"));
    }
}
