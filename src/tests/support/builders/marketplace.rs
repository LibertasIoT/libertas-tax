use crate::tests::*;

// Legacy oracle notation is test scaffolding only; accepted records use canonical IDs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::tests) struct MarketplacePolicy {
    pub(in crate::tests) label: alloc::string::String,
    pub(in crate::tests) premium: i64,
    pub(in crate::tests) advance: i64,
    pub(in crate::tests) allocation: MarketplaceAllocation,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::tests) enum MarketplaceAllocation {
    NotShared,
    FormerSpouse { percent: u32 },
    Spouse,
    NoAdvance { percent: u32 },
    Shared { percent: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::tests) struct MarketplaceMonth {
    pub(in crate::tests) month: u32,
    pub(in crate::tests) policies: Vec<MarketplacePolicy>,
    pub(in crate::tests) benchmark: i64,
    pub(in crate::tests) eligible: MarketplaceEligibility,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::tests) enum MarketplaceEligibility {
    Everyone,
    NoOne,
}

pub(in crate::tests) fn marketplace_month(
    month: u32,
    premium: i64,
    benchmark: i64,
    advance: i64,
) -> MarketplaceMonth {
    MarketplaceMonth {
        month,
        policies: vec![MarketplacePolicy {
            label: "Synthetic policy".into(),
            premium: premium * 100,
            advance: advance * 100,
            allocation: MarketplaceAllocation::NotShared,
        }],
        benchmark: benchmark * 100,
        eligible: MarketplaceEligibility::Everyone,
    }
}
// The existing by-hand arithmetic fixtures supply explicit neutral source
// completion: taxpayer-only ordinary coverage, and one synthetic other family
// when the old fixture specifies a shared allocation. These are test facts,
// not production inference from a label or from missing statement fields.
pub(in crate::tests) fn marketplace_claim(
    d: &mut FederalDraft,
    months: alloc::vec::Vec<MarketplaceMonth>,
) -> MarketplaceClaim {
    let mut facts = MarketplaceFacts {
        separate: if d.setup.as_ref().is_some_and(|s| s.filing_status() == FederalStatus::Separate)
        {
            SeparateRelief::No
        } else {
            SeparateRelief::NotSeparate
        },
        ..MarketplaceFacts::default()
    };
    for m in months {
        if m.policies.iter().filter(|p| p.allocation == MarketplaceAllocation::NotShared).count()
            > 1
        {
            // Explicit actual whole-family benchmark from the published fixture;
            // repeated statements do not add a second family benchmark.
            facts.benchmarks.push(MarketplaceBenchmarkMonth {
                month: m.month,
                groups: vec![MarketplaceBenchmarkGroup {
                    state: TaxState::CA,
                    rating_area: "Fixture area".into(),
                    people: vec![-1],
                    source: MarketplaceBenchmarkSource::Marketplace { amount: m.benchmark },
                }],
                allocated: vec![],
            });
        }
        for p in m.policies {
            let at = if let Some(at) = facts.forms.iter().position(|f| f.issuer == p.label) {
                at
            } else {
                let id = d.next_id;
                d.next_id += 1;
                facts.forms.push(MarketplacePolicyRecord {
                    id,
                    issuer: p.label.clone(),
                    policy_number: "Fixture source".into(),
                    amounts: PolicyAmounts::ByMonth { months: vec![] },
                    lawful_premiums: vec![],
                    enrolling: MarketplaceFamilyRef::ThisReturn,
                    status: MarketplaceStatementStatus::Original,
                    coverage: vec![],
                    allocation: vec![],
                    payments: vec![],
                    medical_costs: vec![],
                });
                facts.forms.len() - 1
            };
            let mut covered = vec![-1];
            let shared = !matches!(p.allocation, MarketplaceAllocation::NotShared);
            if shared && facts.other_families.is_empty() {
                let family = d.next_id;
                d.next_id += 1;
                let person = d.next_id;
                d.next_id += 1;
                facts
                    .other_families
                    .push(MarketplaceFamily { id: family, label: "Other fixture taxpayer".into() });
                facts.covered.push(MarketplaceCoveredPerson {
                    id: person,
                    identity: MarketplacePersonIdentity::OtherFamily {
                        family,
                        label: "Other fixture enrollee".into(),
                    },
                });
            }
            if shared {
                let other = MarketplaceFamilyRef::Other { family: facts.other_families[0].id };
                covered.push(facts.covered[0].id);
                let agreed = |percent| MarketplaceAgreement::Agreed {
                    share: MarketplaceFraction { numerator: percent, denominator: 100 },
                };
                let kind = match p.allocation {
                    MarketplaceAllocation::FormerSpouse { percent } => {
                        MarketplaceAllocationKind::FormerSpouses {
                            pair: MarketplaceMaritalPair {
                                one: MarketplaceFamilyRef::ThisReturn,
                                two: other.clone(),
                            },
                            agreement: agreed(percent),
                            third_families: vec![],
                        }
                    }
                    MarketplaceAllocation::Spouse => MarketplaceAllocationKind::MarriedSeparate {
                        pair: MarketplaceMaritalPair {
                            one: MarketplaceFamilyRef::ThisReturn,
                            two: other.clone(),
                        },
                        one_benchmark: MarketplaceBenchmarkSource::Marketplace {
                            amount: m.benchmark,
                        },
                        two_benchmark: MarketplaceBenchmarkSource::Marketplace {
                            amount: m.benchmark,
                        },
                        third_families: vec![],
                    },
                    MarketplaceAllocation::NoAdvance { percent } => {
                        MarketplaceAllocationKind::NoAdvance {
                            families: vec![
                                MarketplaceNoAdvanceShare {
                                    family: MarketplaceFamilyRef::ThisReturn,
                                    benchmark: m.benchmark,
                                },
                                MarketplaceNoAdvanceShare {
                                    family: other,
                                    benchmark: m.benchmark * i64::from(100 - percent)
                                        / i64::from(percent),
                                },
                            ],
                        }
                    }
                    MarketplaceAllocation::Shared { percent } => {
                        MarketplaceAllocationKind::OtherShared {
                            families: vec![
                                MarketplaceFamilyShare {
                                    family: MarketplaceFamilyRef::ThisReturn,
                                    agreement: agreed(percent),
                                },
                                MarketplaceFamilyShare {
                                    family: other,
                                    agreement: agreed(100 - percent),
                                },
                            ],
                        }
                    }
                    MarketplaceAllocation::NotShared => unreachable!(),
                };
                facts.forms[at].allocation.push(MarketplaceAllocationPeriod {
                    from: m.month,
                    to: m.month,
                    kind,
                });
            }
            let PolicyAmounts::ByMonth { months } = &mut facts.forms[at].amounts else {
                unreachable!()
            };
            months.push(PolicyMonth {
                month: m.month,
                premium: p.premium,
                benchmark: Some(m.benchmark),
                advance: p.advance,
            });
            facts.forms[at].coverage.push(MarketplacePolicyCoverage {
                month: m.month,
                people: covered.clone(),
                entry: vec![],
            });
            for person in covered {
                if !facts.eligibility.iter().any(|r| r.month == m.month && r.person == person) {
                    let status = if person != -1 || m.eligible == MarketplaceEligibility::Everyone {
                        MarketplacePersonEligibility::Eligible
                    } else if m.eligible == MarketplaceEligibility::NoOne {
                        MarketplacePersonEligibility::OtherMec {
                            kind: MarketplaceMecKind::Medicare,
                        }
                    } else {
                        MarketplacePersonEligibility::Unknown
                    };
                    facts.eligibility.push(MarketplaceEligibilityMonth {
                        month: m.month,
                        person,
                        status,
                    });
                }
            }
        }
    }
    if facts.forms.iter().any(|p| {
        (1..=12).any(|m| {
            crate::marketplace_sources::policy_month(&p.amounts, m).is_some_and(|(_, _, a)| a > 0)
        })
    }) {
        // Explicit neutral enrollment estimate facts for below-FPL fixtures.
        facts.income_qualification = MarketplaceIncomeQualification::MarketplaceEstimate {
            estimated_at_least_100: true,
            accurate_disclosure: true,
            not_intentional_or_reckless: true,
        };
    }
    MarketplaceClaim::Yes { facts: alloc::boxed::Box::new(facts) }
}
pub(in crate::tests) fn marketplace_facts_mut(d: &mut FederalDraft) -> &mut MarketplaceFacts {
    let MarketplaceClaim::Yes { facts } = &mut d.credits.as_mut().unwrap().marketplace else {
        panic!("source fixture")
    };
    facts
}

pub(in crate::tests) fn marketplace_nonmedical_child(increase: Option<i64>) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, 10_000_000, 0);
    add_child(&mut d);
    let child = &mut d.dependents[0];
    child.use_for = PersonUse::HealthCoverageOnly;
    child.birth_date = 20000102;
    child.situation =
        DependentSituation::Supported { your_support_over_half: false, gross_income: 0 };
    child.facts = PersonFacts::OWN_SUPPORT.0;
    let child_id = child.id;
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(&mut d, vec![marketplace_month(1, 10000, 1100, 0)]);
    let facts = marketplace_facts_mut(&mut d);
    facts.forms[0].coverage[0].people.push(child_id);
    facts.memberships.push(MarketplacePersonFamily {
        person: child_id,
        family: MarketplaceNonreturnFamily::NoFamily,
    });
    facts.attestations.push(MarketplaceAttestation {
        person: child_id,
        attester: MarketplaceAttester::ThisReturn,
    });
    facts.eligibility.push(MarketplaceEligibilityMonth {
        month: 1,
        person: child_id,
        status: MarketplacePersonEligibility::OtherMec {
            kind: MarketplaceMecKind::EmployerEmployee,
        },
    });
    facts.benchmarks.push(MarketplaceBenchmarkMonth {
        month: 1,
        groups: vec![MarketplaceBenchmarkGroup {
            state: TaxState::CA,
            rating_area: "Actual eligible-parent area".into(),
            people: vec![-1],
            source: MarketplaceBenchmarkSource::Marketplace { amount: 110000 },
        }],
        allocated: vec![],
    });
    facts.forms[0].medical_costs.push(MarketplaceMedicalCostMonth {
        month: 1,
        child_increase: increase,
        tax_free_child: None,
    });
    let facts = itemized(d.deductions.as_mut().unwrap()).clone();
    d.deductions.as_mut().unwrap().choice = DeductionSelection::Itemized { facts };
    d
}

pub(in crate::tests) fn marketplace_wage_draft(
    wages: i64,
    months: Vec<MarketplaceMonth>,
) -> FederalDraft {
    let mut d = complete(FederalStatus::Single);
    d.income[0] = wage(1, Owner::Taxpayer, wages, 0);
    d.credits.as_mut().unwrap().marketplace = marketplace_claim(&mut d, months);
    d
}

// The source-page fixtures save their actual statement before assigning it to
// each established plan/month; ordinary premium amounts remain zero.
pub(in crate::tests) fn assign_marketplace_businesses(d: &mut FederalDraft) {
    let policy =
        crate::marketplace::facts(&d.credits.as_ref().unwrap().marketplace).unwrap().forms[0].id;
    d.adjustments.as_mut().unwrap().health_marketplace.policies = d
        .income
        .iter()
        .flat_map(|entry| match &entry.income {
            FederalIncome::Business { data } => match &data.health_insurance {
                BusinessHealthInsurance::Plan { months, .. } => months
                    .iter()
                    .map(|m| HealthPolicyAllocation {
                        policy,
                        month: m.month,
                        activity: entry.id,
                        parts: 1,
                        reimbursed: None,
                    })
                    .collect::<Vec<_>>(),
                _ => vec![],
            },
            _ => vec![],
        })
        .collect();
}

// Materialize the annual statement only after the scenario adds its actual sources.
// marketplace_month accepts whole dollars; statement and activity IDs use d.next_id.
pub(in crate::tests) fn annual_marketplace_businesses(d: &mut FederalDraft) {
    d.credits.as_mut().unwrap().marketplace =
        marketplace_claim(d, (1..=12).map(|m| marketplace_month(m, 600, 550, 300)).collect());
    assign_marketplace_businesses(d);
}
