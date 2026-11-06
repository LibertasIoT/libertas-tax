// Optional Form8962 PartV; one canonical source graph and monthly PTC owner.
// Pub974(2025), pp38–44, WorksheetsI–V, and26CFR1.36B-4(b)(2).
// The expired enhancement means the2026 calculation restores the >400% stop
// shown in Pub974(2020), pp41–42, using current2026 rates/poverty amounts.
use crate::kernel::ratio::exact::whole;
#[cfg(test)]
use crate::marketplace::premium_assistance;
use crate::marketplace::{CalculatedMarketplace, contribution};
use crate::marketplace::{
    CalculationMoney, contribution_with_money, premium_assistance_with_money,
    reduce_credit_with_money,
};
use crate::marketplace_projection::SourcePeople;
use crate::*;
use alloc::{vec, vec::Vec};

pub(crate) struct Context<'a> {
    pub facts: &'a MarketplaceFacts,
    pub people: &'a SourcePeople<'a>,
    pub joint: bool,
}
struct FamilyWorksheet {
    size: i64,
    monthly: i64,
    start: u32,
    stop: u32,
    columns: Vec<(u32, i64)>,
}
pub(crate) struct Alternative {
    pub columns: Vec<(u32, i64)>,
    you: Option<FamilyWorksheet>,
    spouse: Option<FamilyWorksheet>,
}

fn worksheet(
    context: &Context<'_>,
    family: &MarketplaceMarriageFamily,
    person: i64,
    income: i64,
    last_month: u32,
    money: CalculationMoney,
) -> Result<Option<FamilyWorksheet>, &'static str> {
    let mut ids = vec![person];
    ids.extend_from_slice(&family.dependents);
    let months = crate::marketplace_projection::family_coverage_months(
        context.facts,
        context.people,
        &ids,
        last_month,
    );
    let Some(start) = months.first().copied() else {
        return Ok(None);
    };
    let stop = *months.last().expect("nonempty coverage months");
    let size = ids.len() as i64;
    // WorksheetsI/III explicitly round lines2,6,7 to whole dollars.
    // Preserve those specific steps even during interim Iterative computation.
    let half = crate::kernel::ratio::fraction(income, 1, 2 * money.scale());
    let contribution = contribution(context.facts.residence, half, size);
    if contribution.percent > 400 {
        return Ok(None);
    }
    let projection = crate::marketplace_projection::project_selected_with_income(
        context.facts,
        context.people,
        true,
        &money.cents(income),
        Some(&family.premarriage_policies),
        &family.benchmarks,
        last_month,
    )
    .map_err(|_| "PTC_MARRIAGE_SOURCE_REQUIRED")?;
    let columns = (start..=stop)
        .map(|month| {
            let value = projection.rows.iter().find(|row| row.month == month).map_or(0, |row| {
                premium_assistance_with_money(
                    row,
                    contribution.monthly * money.scale(),
                    true,
                    money,
                )
            });
            (month, value)
        })
        .collect();
    Ok(Some(FamilyWorksheet { size, monthly: contribution.monthly, start, stop, columns }))
}

pub(crate) fn evaluate_with_money(
    context: Context<'_>,
    projection: &CalculatedMarketplace,
    income: i64,
    regular: &[(u32, i64)],
    money: CalculationMoney,
) -> Result<Option<Alternative>, &'static str> {
    let MarketplaceMarriage::Alternative { wedding, unmarried_at_start, you, spouse } =
        &context.facts.marriage
    else {
        return Ok(None);
    };
    let last_month = wedding / 100 % 100;
    if !context.joint || !unmarried_at_start {
        return Err("PTC_MARRIAGE_INELIGIBLE");
    }
    let advance: i64 = projection.rows.iter().map(|row| money.units(&row.advance)).sum();
    let regular_credit: i64 = regular.iter().map(|(_, value)| *value).sum();
    if advance <= regular_credit {
        return Ok(None);
    }
    let actual = contribution_with_money(
        context.facts.residence,
        income,
        context.people.family.len() as i64,
        money,
    );
    if actual.percent < 100 && !projection.below_fpl_qualified {
        return Err("PTC_MARRIAGE_INELIGIBLE");
    }
    // Actual recipient policy assignment consumes each retained form once.
    // A form covering both spouses is not divided by a guessed head count.
    if context
        .facts
        .forms
        .iter()
        .filter(|p| {
            crate::MarketplacePolicyRecordPage::from(&p.form).status
                != MarketplaceStatementStatus::Void
        })
        .any(|p| {
            p.coverage.iter().any(|c| {
                c.month <= last_month
                    && c.people.iter().any(|id| context.people.family.contains(id))
            }) && !you.premarriage_policies.contains(&p.id)
                && !spouse.premarriage_policies.contains(&p.id)
        })
    {
        return Err("PTC_MARRIAGE_SOURCE_REQUIRED");
    }
    let you = worksheet(&context, you, -1, income, last_month, money)?;
    let spouse = worksheet(&context, spouse, -2, income, last_month, money)?;
    let mut columns = Vec::new();
    for month in 1..=last_month {
        let mut present = false;
        let mut tentative = 0;
        for family in [&you, &spouse].into_iter().flatten() {
            if let Some((_, value)) = family.columns.iter().find(|(m, _)| *m == month) {
                present = true;
                tentative += value;
            }
        }
        if present {
            // The employee-family affordability assessment used actual joint
            // household income. Apply the taxpayer's permitted benefits once
            // after combining both spouses' alternative assistance amounts.
            let reduction = projection
                .rows
                .iter()
                .find(|r| r.month == month)
                .map_or_else(|| whole(0), |row| row.credit_reduction.clone());
            columns.push((month, reduce_credit_with_money(tentative, &reduction, money)));
        }
    }
    let alternative_total: i64 = columns.iter().map(|(_, value)| *value).sum();
    let regular_total: i64 = regular
        .iter()
        .filter(|(month, _)| columns.iter().any(|(m, _)| m == month))
        .map(|(_, value)| *value)
        .sum();
    if alternative_total <= regular_total {
        return Ok(None);
    }
    Ok(Some(Alternative { columns, you, spouse }))
}

impl Alternative {
    pub(crate) fn print(&self, result: &mut FederalResult) {
        for (family, entries) in [
            (
                &self.you,
                [
                    ("35a", "marketplace_marriage_you_family", false),
                    ("35b", "marketplace_marriage_you_contribution", true),
                    ("35c", "marketplace_marriage_you_start", false),
                    ("35d", "marketplace_marriage_you_stop", false),
                ],
            ),
            (
                &self.spouse,
                [
                    ("36a", "marketplace_marriage_spouse_family", false),
                    ("36b", "marketplace_marriage_spouse_contribution", true),
                    ("36c", "marketplace_marriage_spouse_start", false),
                    ("36d", "marketplace_marriage_spouse_stop", false),
                ],
            ),
        ] {
            if let Some(family) = family {
                for ((printed, id, money), value) in entries.into_iter().zip([
                    family.size,
                    family.monthly,
                    i64::from(family.start),
                    i64::from(family.stop),
                ]) {
                    let write = if money {
                        crate::line_emission::form_line
                    } else {
                        crate::line_emission::form_count
                    };
                    write(
                        result,
                        FormLine::new(TaxForm::Form8962, printed),
                        id,
                        value,
                        &["credits.marketplace", "agi"],
                        "https://www.irs.gov/publications/p974",
                        msg!("EXPLAIN_MARKETPLACE_MARRIAGE"),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    fn policy(id: i64, person: i64) -> MarketplacePolicyRecord {
        MarketplacePolicyRecord {
            printed: None,
            id,
            form: crate::MarketplacePolicyRecordPage {
                recipient: Default::default(),
                id,
                issuer: "Actual source".into(),
                policy_number: String::new(),
                amounts: PolicyAmounts::ByMonth {
                    months: vec![PolicyMonth {
                        month: 1,
                        premium: 120_000,
                        benchmark: Some(120_000),
                        advance: 110_000,
                    }],
                },
                status: MarketplaceStatementStatus::Original,
            }
            .into(),
            lawful_premiums: vec![],
            enrolling: MarketplaceFamilyRef::ThisReturn,
            coverage: vec![MarketplacePolicyCoverage {
                month: 1,
                people: vec![person],
                entry: vec![],
            }],
            allocation: vec![],
            payments: vec![],
            medical_costs: vec![],
        }
    }
    fn spouse(policy: i64) -> MarketplaceMarriageFamily {
        MarketplaceMarriageFamily {
            dependents: vec![],
            premarriage_policies: vec![policy],
            benchmarks: vec![],
        }
    }
    fn facts() -> MarketplaceFacts {
        MarketplaceFacts {
            forms: vec![policy(1, -1), policy(2, -2)],
            eligibility: vec![-1, -2]
                .into_iter()
                .map(|person| MarketplaceEligibilityMonth {
                    month: 1,
                    person,
                    status: MarketplacePersonEligibility::Eligible,
                })
                .collect(),
            benchmarks: vec![MarketplaceBenchmarkMonth {
                month: 1,
                allocated: vec![],
                groups: vec![MarketplaceBenchmarkGroup {
                    state: TaxState::CA,
                    rating_area: "Actual joint coverage area".into(),
                    people: vec![-1, -2],
                    source: MarketplaceBenchmarkSource::Marketplace { amount: 240_000 },
                }],
            }],
            marriage: MarketplaceMarriage::Alternative {
                wedding: 20260215,
                unmarried_at_start: true,
                you: spouse(1),
                spouse: spouse(2),
            },
            ..MarketplaceFacts::default()
        }
    }
    fn calculate(
        facts: &MarketplaceFacts,
        income: i64,
    ) -> (crate::marketplace::MarketplaceOutcome, FederalResult) {
        let family = [-1, -2];
        let people = SourcePeople { family: &family, current: &family, births: &[] };
        let projection =
            crate::marketplace_projection::project(facts, &people, true, income).unwrap();
        let mut result = FederalResult::empty(0);
        let outcome = crate::marketplace::calculate_months_with_marriage(
            &projection,
            income,
            2,
            Some(Context { facts, people: &people, joint: true }),
            &mut result,
        );
        (outcome, result)
    }

    // Pub974(2025), printed42–43 WorksheetsI/III lines2,6,7 expressly
    // round half-income and annual/monthly contributions to dollars. The
    // method-wide cents path preserves these specific worksheet operations.
    #[test]
    fn iterative_marriage_preserves_explicit_contribution_rounding() {
        let mut facts = facts();
        let paper = &mut facts.forms[0].form;
        paper.month_mut(1).premium = Some(120_049);
        paper.month_mut(1).benchmark = Some(120_049);
        let ids = [-1, -2];
        let people = SourcePeople { family: &ids, current: &ids, births: &[] };
        let context = Context { facts: &facts, people: &people, joint: true };
        let result = worksheet(&context, &spouse(1), -1, 4_800_099, 2, CalculationMoney::Cents)
            .unwrap()
            .unwrap();
        // Half48000.99 rounds24000; annual1039/month87. Retained actual
        // premium1200.49 minus87 gives1113.49, not a rounded1113 or1114.
        assert_eq!(result.monthly, 87);
        assert_eq!(result.columns, vec![(1, 111_349)]);
    }

    // Derived2026 fixture under Pub974 WorksheetsI–V, pp38–44; sources explicitly
    // assign one Jan1095A to each spouse, weddingFeb15, bothunmarriedJan1.
    // Each A/B1200,C1100; jointincome48000, no othermonths. 2026 FPL/rates:
    // joint226%→annual3629/month302; half24000 family1→153%→1039/month87.
    // Regular E2098, alternative2×1113=2226, APTC2200; election gives no extra26.
    #[test]
    fn marriage_alternative_can_exceed_advances_without_creating_refundable_credit() {
        let (outcome, result) = calculate(&facts(), 48_000);
        assert_eq!((outcome.credit, outcome.repayment, outcome.refundable), (2226, 0, 0));
        assert_eq!(
            outcome
                .expense_columns
                .iter()
                .map(|(_, premium, subsidy)| premium - subsidy)
                .sum::<crate::kernel::ratio::exact::Exact>(),
            whole(20_000),
        );
        assert_eq!(outcome.expense_credit, 2200);
        assert_eq!(crate::marketplace::specified_deduction(&outcome, &[1], 2400, 2400), 200);
        let mut regular = facts();
        regular.marriage = MarketplaceMarriage::Regular;
        let (ordinary, _) = calculate(&regular, 48_000);
        assert_eq!(
            (ordinary.credit, ordinary.expense_credit, ordinary.repayment),
            (2098, 2098, 102)
        );
        assert_eq!(
            ordinary
                .expense_columns
                .iter()
                .map(|(_, premium, subsidy)| premium - subsidy)
                .sum::<crate::kernel::ratio::exact::Exact>(),
            whole(30_200),
        );
        assert_eq!(crate::marketplace::specified_deduction(&ordinary, &[1], 2400, 2400), 302);
        for printed in ["35a", "35b", "35c", "35d", "36a", "36b", "36c", "36d"] {
            assert!(
                result
                    .lines
                    .iter()
                    .any(|line| line.form_line == Some(FormLine::new(TaxForm::Form8962, printed)))
            );
        }
    }

    // Derived2026 boundary: joint90000/21150>400%; half45000/15650=287%,
    // applicable9.56%, annual4302/month359; two source1200benchmarks→1682.
    // 26CFR1.36B-4(b)(2) Example4 establishes the above-joint-cutoff route;
    // current2026 rates replace that example's historical dollars/rates.
    #[test]
    fn marriage_above_joint_cutoff_uses_each_alternative_family_cutoff() {
        let (outcome, _) = calculate(&facts(), 90_000);
        assert_eq!((outcome.credit, outcome.repayment, outcome.refundable), (1682, 518, 0));
        let (outcome, _) = calculate(&facts(), 130_000);
        assert_eq!((outcome.credit, outcome.repayment), (0, 2200));
    }

    #[test]
    fn marriage_missing_optional_benchmark_preserves_regular_mandatory_reconciliation() {
        let mut facts = facts();
        let paper = &mut facts.forms[0].form;
        paper.month_mut(1).benchmark = None; // Regular joint correction remains known.
        let (outcome, result) = calculate(&facts, 48_000);
        assert_eq!((outcome.credit, outcome.repayment), (2098, 102));
        assert!(result.issues.is_empty());
        assert!(
            result
                .review_notes
                .iter()
                .any(|note| note.display.key == "PTC_MARRIAGE_SOURCE_REQUIRED")
        );
    }

    #[test]
    fn marriage_unmarried_source_fact_and_complete_policy_assignments_are_required() {
        let mut facts = facts();
        let MarketplaceMarriage::Alternative { unmarried_at_start, .. } = &mut facts.marriage
        else {
            unreachable!()
        };
        *unmarried_at_start = false;
        let (outcome, result) = calculate(&facts, 48_000);
        assert_eq!(outcome.repayment, 102);
        assert!(
            result.review_notes.iter().any(|note| note.display.key == "PTC_MARRIAGE_INELIGIBLE")
        );
        let MarketplaceMarriage::Alternative { unmarried_at_start, spouse, .. } =
            &mut facts.marriage
        else {
            unreachable!()
        };
        *unmarried_at_start = true;
        spouse.premarriage_policies.clear();
        let (outcome, result) = calculate(&facts, 48_000);
        assert_eq!(outcome.repayment, 102);
        assert!(
            result
                .review_notes
                .iter()
                .any(|note| note.display.key == "PTC_MARRIAGE_SOURCE_REQUIRED")
        );
    }

    #[test]
    fn marriage_combines_spouse_worksheets_before_one_qsehra_reduction() {
        let mut facts = facts();
        facts.qsehra.push(MarketplaceQsehra {
            id: 3,
            employee: -1,
            source: MarketplaceQsehraSource::Notice {
                employer: "Actual employer".into(),
                annual_permitted: 10_000,
            },
            months: vec![MarketplaceQsehraMonth {
                month: 1,
                self_only: 10_000,
                actual: 10_000,
                self_benchmark: 400_000,
                covered: vec![-1],
            }],
            determinations: vec![],
        });
        let (outcome, _) = calculate(&facts, 48_000);
        assert_eq!((outcome.credit, outcome.repayment), (2126, 74));
    }
}

#[cfg(test)]
mod published_worksheet_tests {
    use super::*;

    // Pub974(2025 final, printedJan12,2026), pp44–46, Paulette/Quentin.
    // Published worksheet arithmetic only, not a fabricated2026 complete return:
    // source2025 contributions400/148 beforemarriage,768 afterward; current2026
    // percentage/poverty owners intentionally are not substituted into this oracle.
    // The prose line8a9604 contradicts117000×.0788=9220 and subsequent line8b;
    // use the actual printed monthly worksheet values and flag the typo in evidence.
    #[test]
    fn marriage_published_paulette_quentin_worksheet_totals_match_7021_and_1402() {
        let row = |premium: i64, benchmark: i64| crate::marketplace::CalculatedMonth {
            month: 1,
            premium: whole(premium * 100),
            benchmark: whole(benchmark * 100),
            medical_premium: whole(premium * 100),
            credit_reduction: whole(0),
            advance: whole(0),
            eligible: true,
        };
        let before = premium_assistance(&row(500, 433), 400, true)
            + premium_assistance(&row(1000, 833), 148, true);
        let regular_before = premium_assistance(&row(1500, 1266), 768, true);
        let after = premium_assistance(&row(1350, 1167), 768, true);
        assert_eq!((before, regular_before, after), (718, 498, 399));
        let alternative = before * 7 + after * 5;
        let regular = regular_before * 7 + after * 5;
        let advance = 794 * 7 + 573 * 5;
        assert_eq!((alternative, advance - alternative), (7021, 1402));
        assert_eq!((regular, advance - regular), (5481, 2942));
    }
}
