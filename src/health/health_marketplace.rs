pub use crate::health_marketplace_source::{
    HealthMarketplace, HealthPolicyAllocation, HealthPolicyBenchmark, HealthPtcMethod,
    ReimbursedPolicyPremiums,
};
// Accepted self-employed plan assignments refer to the canonical Marketplace
// policy catalog. Source premiums, advances and tax families remain there.
// Pub974 (2025), pp49–59: coverage-family SLCSP allocation precedes the
// consistent reasonable allocation among businesses; Worksheets W/X are shared.
use crate::kernel::ratio::exact::{Exact, whole};
use crate::*;
use alloc::{collections::BTreeMap, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Marketplace policy established under a business
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthPolicyAllocationPage {
    /// Saved Marketplace statement
    /// Save the actual Form1095-A in Marketplace coverage before selecting it here.
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.ptc_policies")]
    pub policy: i64,
    /// Coverage month
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Business or entity under which the plan is established
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.health_activities")]
    pub activity: i64,
    /// Parts allocated to this business
    /// Relative parts implement your actual consistent reasonable allocation among businesses. The same allocation is used for premiums and credit in each calculation pass.
    #[libertas_number(min = 1)]
    #[libertas_default(1)]
    pub parts: u32,
    /// Actual entity reimbursement of this assigned policy month
    /// Complete only for an entity Reimbursed plan; partial reimbursements require actual source attribution, including zero. Premium money remains in the saved Marketplace statement.
    /// [NullText]
    /// This policy month is not part of an entity-reimbursed plan
    /// [SetText]
    /// The entity's actual reimbursement of this policy month is
    pub reimbursed: Option<ReimbursedPolicyPremiums>,
}

/// Actual benchmarks for a policy covering people outside the coverage family
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct HealthPolicyBenchmarkPage {
    /// Saved Marketplace statement
    #[libertas_number(min = 1)]
    #[libertas_enum_source("$.ptc_policies")]
    pub policy: i64,
    /// Coverage month
    // Tax structure: calendar months are January (1) through December (12).
    #[libertas_number(min = 1, max = 12)]
    pub month: u32,
    /// Actual applicable SLCSP for the policy's coverage-family members
    /// Obtain the applicable source amount for these people. The App does not divide premiums by a head count.
    pub coverage: MarketplaceBenchmarkSourcePage,
    /// Actual applicable SLCSP for the policy's other enrolled family members
    /// This distinct benchmark allocates specified and nonspecified premiums before business assignments. It does not replace Form8962's actual coverage-family benchmark.
    pub noncoverage: MarketplaceBenchmarkSourcePage,
}
impl From<HealthPolicyAllocationPage> for HealthPolicyAllocation {
    fn from(v: HealthPolicyAllocationPage) -> Self {
        Self {
            policy: v.policy,
            month: v.month,
            activity: v.activity,
            parts: v.parts,
            reimbursed: v.reimbursed,
        }
    }
}
impl From<HealthPolicyAllocation> for HealthPolicyAllocationPage {
    fn from(v: HealthPolicyAllocation) -> Self {
        Self {
            policy: v.policy,
            month: v.month,
            activity: v.activity,
            parts: v.parts,
            reimbursed: v.reimbursed,
        }
    }
}
impl From<HealthPolicyBenchmarkPage> for HealthPolicyBenchmark {
    fn from(v: HealthPolicyBenchmarkPage) -> Self {
        Self {
            policy: v.policy,
            month: v.month,
            coverage: v.coverage.into(),
            noncoverage: v.noncoverage.into(),
        }
    }
}
impl From<HealthPolicyBenchmark> for HealthPolicyBenchmarkPage {
    fn from(v: HealthPolicyBenchmark) -> Self {
        Self {
            policy: v.policy,
            month: v.month,
            coverage: v.coverage.into(),
            noncoverage: v.noncoverage.into(),
        }
    }
}

pub(crate) fn activities(d: &FederalDraft) -> Vec<i64> {
    d.income
        .iter()
        .filter(|e| {
            matches!(
                e.income,
                FederalIncome::Business { .. }
                    | FederalIncome::Partnership { .. }
                    | FederalIncome::Scorp { .. }
                    | FederalIncome::PartnershipActivities { .. }
                    | FederalIncome::ScorpActivities { .. }
            )
        })
        .map(|e| e.id)
        .collect()
}

// One transient allocation of canonical policy payments. PartIV, lawful
// coverage, payment exceptions and coverage-family benchmarks are resolved
// before business parts. No source amount is stored on a second plan ledger.
#[derive(Clone, Debug)]
pub(crate) struct HealthSourcePremium {
    pub policy: i64,
    pub month: u32,
    pub activity: i64,
    pub paid: Exact,
    pub specified: Exact,
    pub credit_premium: Exact,
    #[cfg(test)]
    pub advance: Exact,
}

impl HealthMarketplace {
    pub(crate) fn project_sources(
        &self,
        facts: &MarketplaceFacts,
        people: &crate::marketplace_projection::SourcePeople<'_>,
    ) -> (Vec<HealthSourcePremium>, Vec<crate::marketplace_projection::SourceIssue>) {
        let mut groups = BTreeMap::<(i64, u32), Vec<&HealthPolicyAllocation>>::new();
        for row in &self.policies {
            groups.entry((row.policy, row.month)).or_default().push(row);
        }
        let mut projected = Vec::new();
        let mut issues = Vec::new();
        for ((policy, month), assignments) in groups {
            match crate::marketplace_projection::health_policy_month(
                facts,
                people,
                policy,
                month,
                &self.benchmarks,
            ) {
                Ok(Some(source)) => {
                    let total: Exact = assignments.iter().map(|a| whole(i64::from(a.parts))).sum();
                    // Validation may project a forged candidate before its scalar
                    // bounds are checked. Invalid zero parts are a source error,
                    // never a division panic or an accepted equal allocation.
                    if total == whole(0) {
                        issues.push(crate::marketplace_projection::SourceIssue {
                            policy,
                            month,
                            person: None,
                            missing: crate::marketplace_projection::MissingSource::Allocation,
                        });
                        continue;
                    }
                    for assignment in assignments {
                        let share = whole(i64::from(assignment.parts)) / &total;
                        projected.push(HealthSourcePremium {
                            policy,
                            month,
                            activity: assignment.activity,
                            paid: &source.paid * &share,
                            specified: &source.specified * &share,
                            credit_premium: &source.credit_premium * &share,
                            #[cfg(test)]
                            advance: &source.advance * share,
                        });
                    }
                }
                Ok(None) => {}
                Err(issue) => issues.push(issue),
            }
        }
        (projected, issues)
    }

    pub(crate) fn source_premiums(
        &self,
        d: &FederalDraft,
    ) -> (Vec<HealthSourcePremium>, Vec<crate::marketplace_projection::SourceIssue>) {
        let Some(facts) =
            d.credits.as_ref().and_then(|c| crate::marketplace::facts(&c.marketplace))
        else {
            return (Vec::new(), Vec::new());
        };
        let family = crate::marketplace::family_people(d);
        let (current, births) = crate::marketplace::current_people(d);
        self.project_sources(
            facts,
            &crate::marketplace_projection::SourcePeople {
                family: &family,
                current: &current,
                births: &births,
            },
        )
    }

    pub(crate) fn valid(&self, d: &FederalDraft) -> bool {
        self.issue(d).is_none()
    }
    pub(crate) fn issue(&self, d: &FederalDraft) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let invalid = |field| {
            ValidationIssue::new("PROBLEM_INVALID_HEALTH_POLICY_ASSIGNMENT").at_field(field)
        };
        let facts = d.credits.as_ref().and_then(|c| crate::marketplace::facts(&c.marketplace));
        let policy = |id, month| {
            facts.is_some_and(|f| {
                f.forms.iter().any(|p| {
                    p.id == id
                        && crate::MarketplacePolicyRecordPage::from(&p.form).status
                            != MarketplaceStatementStatus::Void
                        && crate::marketplace_sources::policy_month(
                            &crate::MarketplacePolicyRecordPage::from(&p.form).amounts,
                            month,
                        )
                        .is_some()
                })
            })
        };
        let (sources, _) = self.source_premiums(d);
        for source in &sources {
            let Some((index, assignment)) = self.policies.iter().enumerate().find(|(_, a)| {
                (a.policy, a.month, a.activity) == (source.policy, source.month, source.activity)
            }) else {
                continue;
            };
            let Some(v) = &assignment.reimbursed else {
                continue;
            };
            let funded = |field| invalid(field).in_field("reimbursed").in_list("policies", index);
            if crate::kernel::ratio::exact::whole(v.premiums) > source.paid {
                return Some(funded("premiums"));
            }
            if crate::kernel::ratio::exact::whole(v.coverage) > source.specified {
                return Some(funded("coverage"));
            }
            if crate::kernel::ratio::exact::whole(v.premiums - v.coverage)
                > &source.paid - &source.specified
            {
                return Some(funded("premiums"));
            }
            if let Some(child) = v.child
                && let Some(capacity) = crate::health::assigned_child_capacity(d, source)
                && (crate::kernel::ratio::exact::whole(child) > capacity
                    || crate::kernel::ratio::exact::whole(v.premiums - child)
                        > &source.paid - capacity)
            {
                return Some(funded("child"));
            }
        }
        let activities = activities(d);
        for (index, p) in self.policies.iter().enumerate() {
            if !policy(p.policy, p.month) {
                return Some(invalid("policy").in_list("policies", index));
            }
            if !(1..=12).contains(&p.month) {
                return Some(invalid("month").in_list("policies", index));
            }
            if !activities.contains(&p.activity) {
                return Some(invalid("activity").in_list("policies", index));
            }
            if p.parts == 0 {
                return Some(invalid("parts").in_list("policies", index));
            }
            if let Some(issue) = p.reimbursed.as_ref().and_then(ReimbursedPolicyPremiums::issue) {
                return Some(issue.in_field("reimbursed").in_list("policies", index));
            }
            if self.policies[..index]
                .iter()
                .any(|v| (v.policy, v.month, v.activity) == (p.policy, p.month, p.activity))
            {
                return Some(invalid("policy").in_list("policies", index));
            }
        }
        for (index, b) in self.benchmarks.iter().enumerate() {
            if !policy(b.policy, b.month) {
                return Some(invalid("policy").in_list("benchmarks", index));
            }
            if !(1..=12).contains(&b.month) {
                return Some(invalid("month").in_list("benchmarks", index));
            }
            if self.benchmarks[..index].iter().any(|v| (v.policy, v.month) == (b.policy, b.month)) {
                return Some(invalid("policy").in_list("benchmarks", index));
            }
            if !facts.is_some_and(|f| crate::marketplace_sources::benchmark(&b.coverage, f)) {
                return Some(invalid("coverage").in_list("benchmarks", index));
            }
            if !facts.is_some_and(|f| crate::marketplace_sources::benchmark(&b.noncoverage, f)) {
                return Some(invalid("noncoverage").in_list("benchmarks", index));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::ratio::exact::{dollars, whole};
    use alloc::vec;

    // Pub974 (2025 final printed January12,2026), p49 Gary/Sue example:
    // premiums15000, coverage-familySLCSP12000, Sue'sSLCSP6000 =>10000/5000.
    // Source-derived worksheet inputs; synthetic one-month aggregation tests
    // only this source allocation, without inventing an entire published return.
    #[test]
    fn health_marketplace_gary_uses_actual_benchmarks_before_business_parts() {
        let mut facts = MarketplaceFacts::default();
        facts.forms.push(MarketplacePolicyRecord {
            printed: None,
            id: 1,
            form: crate::MarketplacePolicyRecordPage {
                recipient: Default::default(),
                id: 1,
                issuer: "Source insurer".into(),
                policy_number: "Source policy".into(),
                amounts: PolicyAmounts::ByMonth {
                    months: vec![PolicyMonth {
                        month: 1,
                        premium: 1_500_000,
                        benchmark: Some(1_200_000),
                        advance: 420_000,
                    }],
                },
                status: MarketplaceStatementStatus::Original,
            }
            .into(),
            lawful_premiums: vec![],
            enrolling: MarketplaceFamilyRef::ThisReturn,
            coverage: vec![MarketplacePolicyCoverage {
                month: 1,
                people: vec![-1, -2, 2, 3],
                entry: vec![],
            }],
            allocation: vec![],
            payments: vec![],
            medical_costs: vec![],
        });
        for person in [-1, -2, 2, 3] {
            facts.eligibility.push(MarketplaceEligibilityMonth {
                month: 1,
                person,
                status: if person == -2 {
                    MarketplacePersonEligibility::OtherMec {
                        kind: MarketplaceMecKind::EmployerEmployee,
                    }
                } else {
                    MarketplacePersonEligibility::Eligible
                },
            });
        }
        let ids = [-1, -2, 2, 3];
        let people = crate::marketplace_projection::SourcePeople {
            family: &ids,
            current: &ids,
            births: &[],
        };
        assert!(
            crate::marketplace_projection::health_policy_month(&facts, &people, 1, 1, &[]).is_err()
        );
        let sources = [HealthPolicyBenchmark {
            policy: 1,
            month: 1,
            coverage: MarketplaceBenchmarkSource::Marketplace { amount: 1_200_000 },
            noncoverage: MarketplaceBenchmarkSource::Marketplace { amount: 600_000 },
        }];
        let projection =
            crate::marketplace_projection::health_policy_month(&facts, &people, 1, 1, &sources)
                .unwrap()
                .unwrap();
        assert_eq!(dollars(&projection.specified), Some(10_000));
        assert_eq!(dollars(&(&projection.paid - &projection.specified)), Some(5_000));
        assert_eq!(dollars(&projection.advance), Some(4_200));
        // Preserve source cents and an exact ratio, not the displayed66.67%.
        assert_eq!(projection.specified, whole(1_000_000));
        // Derived People-only enrollee boundary: a current nonreturn person
        // still belongs to the actual policy expense set when this return
        // attests for them. Do not turn the whole policy into specified cost
        // merely because every FINAL tax-family member is PTC-eligible.
        let mut nonreturn = facts.clone();
        for person in &mut nonreturn.forms[0].coverage[0].people {
            if *person == -2 {
                *person = 4;
            }
        }
        for row in &mut nonreturn.eligibility {
            if row.person == -2 {
                row.person = 4;
            }
        }
        nonreturn.memberships.push(MarketplacePersonFamily {
            person: 4,
            family: MarketplaceNonreturnFamily::NoFamily,
        });
        nonreturn
            .attestations
            .push(MarketplaceAttestation { person: 4, attester: MarketplaceAttester::ThisReturn });
        let nonreturn_people = crate::marketplace_projection::SourcePeople {
            family: &[-1, 2, 3],
            current: &[-1, 4, 2, 3],
            births: &[],
        };
        let nonreturn_source = crate::marketplace_projection::health_policy_month(
            &nonreturn,
            &nonreturn_people,
            1,
            1,
            &sources,
        )
        .unwrap()
        .unwrap();
        assert_eq!(nonreturn_source.specified, whole(1_000_000));
        assert!(
            crate::marketplace_projection::health_policy_month(
                &nonreturn,
                &nonreturn_people,
                1,
                1,
                &[],
            )
            .is_err()
        );
        // The same saved source is established under two actual activities.
        // Relative parts split the already source-qualified amounts exactly;
        // neither premium nor credit is repeated for a second business.
        let mut coordination = HealthMarketplace {
            policies: vec![
                HealthPolicyAllocation {
                    policy: 1,
                    month: 1,
                    activity: 41,
                    parts: 1,
                    reimbursed: None,
                },
                HealthPolicyAllocation {
                    policy: 1,
                    month: 1,
                    activity: 42,
                    parts: 2,
                    reimbursed: None,
                },
            ],
            benchmarks: sources.to_vec(),
            ..HealthMarketplace::default()
        };
        let (allocated, issues) = coordination.project_sources(&facts, &people);
        assert!(issues.is_empty());
        assert_eq!(allocated.len(), 2);
        assert_eq!((allocated[0].policy, allocated[0].month, allocated[0].activity), (1, 1, 41));
        assert_eq!((allocated[1].policy, allocated[1].month, allocated[1].activity), (1, 1, 42));
        assert_eq!(&allocated[0].specified + &allocated[1].specified, whole(1_000_000));
        assert_eq!(&allocated[0].paid + &allocated[1].paid, whole(1_500_000));
        assert_eq!(&allocated[0].credit_premium + &allocated[1].credit_premium, whole(1_500_000));
        assert_eq!(&allocated[0].credit_premium * whole(3), whole(1_500_000));
        assert_eq!(&allocated[0].advance + &allocated[1].advance, whole(420_000));
        assert_eq!(&allocated[0].specified * whole(3), whole(1_000_000));
        coordination.policies.reverse();
        let (reordered, issues) = coordination.project_sources(&facts, &people);
        assert!(issues.is_empty());
        assert_eq!(allocated[0].specified, reordered[1].specified);
        assert_eq!(allocated[0].credit_premium, reordered[1].credit_premium);
        coordination.benchmarks.clear();
        let (missing, issues) = coordination.project_sources(&facts, &people);
        assert!(missing.is_empty());
        assert_eq!(issues.len(), 1);
        assert_eq!((issues[0].policy, issues[0].month), (1, 1));
        // Derived actual-funding extension to the published Gary allocation:
        // actual tax-free funding $2,000, of which $500 applies to the
        // coverage family. Pub502 Insurance Premiums excludes actual tax-free
        // funding, not a QSEHRA permitted benefit or a guessed SLCSP share.
        facts.forms[0].payments.push(MarketplacePaymentMonth {
            month: 1,
            paid: 1_500_000,
            refunded: 0,
            tax_free_premiums: 200_000,
            tax_free_coverage: None,
            unpaid: 0,
            coverage: MarketplacePaymentCoverage::PaidByDeadline,
            appeal: None,
        });
        let missing =
            crate::marketplace_projection::health_policy_month(&facts, &people, 1, 1, &sources)
                .err()
                .unwrap();
        assert_eq!(
            missing.missing,
            crate::marketplace_projection::MissingSource::HealthExpenseAllocation
        );
        facts.forms[0].payments[0].tax_free_coverage = Some(50_000);
        let funded =
            crate::marketplace_projection::health_policy_month(&facts, &people, 1, 1, &sources)
                .unwrap()
                .unwrap();
        assert_eq!(dollars(&funded.specified), Some(9_500));
        // Pub974 Step3 sole-plan line6 uses the policy's full PTC, not
        // the separate 2/3 SLCSP share of its deductible expense.
        assert_eq!(funded.credit_premium, whole(1_500_000));
        assert_eq!(dollars(&funded.paid), Some(13_000));
        assert_eq!(dollars(&(&funded.paid - &funded.specified)), Some(3_500));
        assert_eq!(dollars(&funded.advance), Some(4_200));
        // Mandatory credit source columns remain gross. Reimbursement
        // attribution is an expense issue, not a missing PTC source fact.
        facts.benchmarks.push(MarketplaceBenchmarkMonth {
            month: 1,
            groups: vec![MarketplaceBenchmarkGroup {
                state: TaxState::CA,
                rating_area: "Actual source area".into(),
                people: vec![-1, 2, 3],
                source: MarketplaceBenchmarkSource::Marketplace { amount: 1_200_000 },
            }],
            allocated: vec![],
        });
        let credit = crate::marketplace_projection::project(&facts, &people, true, 48_000).unwrap();
        assert_eq!(credit.rows[0].premium, whole(1_500_000));
        assert_eq!(credit.rows[0].medical_premium, whole(1_300_000));
        assert_eq!(credit.rows[0].advance, whole(420_000));
        assert!(crate::marketplace_projection::expense_sources_valid(&facts, &people));
        facts.forms[0].payments[0].tax_free_coverage = Some(1_000_001);
        assert!(
            crate::marketplace_projection::health_policy_month(&facts, &people, 1, 1, &sources)
                .is_err()
        );
        facts.forms[0].payments[0].tax_free_coverage = None;
        facts.forms[0].payments[0].tax_free_premiums = 500_001;
        facts.forms[0].payments[0].tax_free_coverage = Some(0);
        assert!(
            crate::marketplace_projection::health_policy_month(&facts, &people, 1, 1, &sources)
                .is_err()
        );
        facts.forms[0].payments[0].tax_free_premiums = 1_500_001;
        assert!(!crate::marketplace_projection::expense_sources_valid(&facts, &people));
        facts.forms[0].payments[0].tax_free_premiums = 200_000;
        facts.forms[0].payments[0].tax_free_coverage = None;
        // Pub974 Coverage family and Notice2017-67 Q65: QSEHRA's credit
        // bar is distinct from non-Marketplace MEC. This classifies source
        // premiums only; actual section162(l) employer eligibility and
        // tax-free reimbursements must still be applied by the plan owner.
        facts.eligibility.iter_mut().find(|r| r.person == -2).unwrap().status =
            MarketplacePersonEligibility::AffordableExternalQsehra;
        let with_credit_bar =
            crate::marketplace_projection::health_policy_month(&facts, &people, 1, 1, &[])
                .unwrap()
                .unwrap();
        assert_eq!(dollars(&with_credit_bar.specified), Some(13_000));
        // All remaining return members are in the coverage family. An
        // explicit actual offset contradicting the determinate whole amount
        // must not be ignored merely because no SLCSP split is necessary.
        facts.forms[0].payments[0].tax_free_coverage = Some(0);
        assert!(
            crate::marketplace_projection::health_policy_month(&facts, &people, 1, 1, &[]).is_err()
        );
        facts.forms[0].payments[0].tax_free_coverage = Some(200_000);
        assert_eq!(
            dollars(
                &crate::marketplace_projection::health_policy_month(&facts, &people, 1, 1, &[])
                    .unwrap()
                    .unwrap()
                    .specified
            ),
            Some(13_000)
        );
    }
}
