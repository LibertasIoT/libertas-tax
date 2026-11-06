pub(crate) use crate::marketplace_claim_source::facts;
pub use crate::marketplace_claim_source::{
    MarketplaceClaim, MarketplaceDependent, MarketplaceResidence,
};

use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::exact::{Exact, ratio, whole};
use crate::kernel::ratio::{fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::{form_line, line};
use crate::{FederalResult, FederalStatus, FormLine, TaxForm};
use alloc::vec::Vec;

// Pub974's Iterative method uses dollars and cents throughout its interim
// Form8962 calculations. Ordinary filing projections retain their existing
// whole-dollar boundaries. This is a runtime unit, never an accepted fact.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CalculationMoney {
    Dollars,
    Cents,
}
impl CalculationMoney {
    pub(crate) fn scale(self) -> i64 {
        match self {
            Self::Dollars => 1,
            Self::Cents => 100,
        }
    }
    pub(crate) fn units(self, cents: &Exact) -> i64 {
        crate::kernel::ratio::exact::rounded(&(cents * ratio(self.scale(), 100).unwrap()))
            .expect("bounded Marketplace money")
    }
    pub(crate) fn cents(self, value: i64) -> Exact {
        whole(value) * ratio(100, self.scale()).unwrap()
    }
    pub(crate) fn dollars(self, value: i64) -> i64 {
        fraction(value, 1, self.scale())
    }
}

pub(crate) fn w2_sources(d: &crate::FederalDraft) -> Vec<(i64, i64, i64)> {
    d.income
        .iter()
        .filter_map(|entry| {
            let crate::FederalIncome::Wage { data } = &entry.income else {
                return None;
            };
            let data = crate::FederalWage::from(data);
            let amount: i64 = data
                .other_box12
                .iter()
                .filter(|r| r.code == crate::W2Code::FF)
                .map(|r| r.amount)
                .sum();
            (amount > 0).then_some((
                entry.id,
                if entry.owner == crate::DocumentOwner::Yours { -1 } else { -2 },
                amount,
            ))
        })
        .collect()
}

pub(crate) fn current_people(d: &crate::FederalDraft) -> (Vec<i64>, Vec<(i64, u32)>) {
    let current = crate::federal_rules::people_students(d).iter().map(|p| p.id).collect();
    let mut births = Vec::new();
    if let Some(people) = &d.people {
        births.push((-1, people.taxpayer.birth_date));
        if let Some(spouse) = people.spouse.as_ref() {
            births.push((-2, spouse.birth_date));
        }
    }
    births.extend(d.dependents().iter().map(|p| (p.id, p.birth_date)));
    (current, births)
}

pub(crate) fn family_people(d: &crate::FederalDraft) -> Vec<i64> {
    let mut ids = Vec::new();
    if let Some(people) = &d.people {
        // IRC36B-1(d): claimability excludes a filer from their own tax
        // family even when nobody actually claims them. Their advance-credit
        // responsibility comes from the accepted attestation, not a member
        // manufactured for the poverty guideline or the membership picker.
        if !people.taxpayer.dependency.claimable() {
            ids.push(-1);
        }
        if crate::interview::joint(d)
            && people.spouse.as_ref().is_some_and(|spouse| !spouse.dependency.claimable())
        {
            ids.push(-2);
        }
        let family = crate::federal_rules::family(d, &mut FederalResult::empty(d.revision), people);
        ids.extend_from_slice(family.dependent_ids());
    }
    ids
}

pub(crate) fn family_sources_valid(
    facts: &crate::MarketplaceFacts,
    d: &crate::FederalDraft,
) -> bool {
    family_sources_issue(facts, d).is_none()
}
pub(crate) fn family_sources_issue(
    facts: &crate::MarketplaceFacts,
    d: &crate::FederalDraft,
) -> Option<crate::validation::ValidationIssue> {
    use crate::validation::ValidationIssue;
    let invalid =
        || ValidationIssue::new("MESSAGE_CHECK_THE_MONTHS_AND_AMOUNTS_OF_EACH_FORM_1095_A");
    let family = family_people(d);
    if let crate::MarketplaceMarriage::Alternative { you, spouse, .. } = &facts.marriage
        && !(crate::interview::joint(d)
            && you
                .dependents
                .iter()
                .chain(&spouse.dependents)
                .all(|id| *id > 0 && family.contains(id)))
    {
        return Some(invalid().at_field("marriage"));
    }
    if let Some(index) = facts.dependents.iter().position(|dependent| {
        d.income.iter().any(|entry| {
            matches!(&entry.income, crate::FederalIncome::ChildElection { data }
            if data.child_id == dependent.person && data.lines().reported()
                && data.omission(&d.dependents, crate::interview::joint(d)).is_none())
        })
    }) {
        return Some(invalid().at_field("person").in_list("dependents", index));
    }
    if let Some(index) =
        facts.dependent_qsehra_employees.iter().position(|id| !(*id > 0 && family.contains(id)))
    {
        return Some(invalid().at_list_item("dependent_qsehra_employees", index));
    }
    if let Some(index) = facts.memberships.iter().position(|row| family.contains(&row.person)) {
        return Some(invalid().at_field("person").in_list("memberships", index));
    }
    if let Some(index) = facts.qsehra.iter().position(|q| {
        !(q.employee == -1
            || q.employee == -2 && crate::interview::joint(d)
            || q.employee > 0 && family.contains(&q.employee))
    }) {
        return Some(invalid().at_field("employee").in_list("qsehra", index));
    }
    facts
        .employee_qsehra_facts
        .iter()
        .position(|e| !(e.employee > 0 && family.contains(&e.employee)))
        .map(|index| invalid().at_field("employee").in_list("employee_qsehra_facts", index))
}

#[cfg(test)]
pub(crate) fn valid(claim: &MarketplaceClaim, d: &crate::FederalDraft) -> bool {
    valid_with_issue(claim, d, &mut None)
}
pub(crate) fn valid_with_issue(
    claim: &MarketplaceClaim,
    d: &crate::FederalDraft,
    _issue: &mut Option<crate::validation::ValidationIssue>,
) -> bool {
    facts(claim).is_none_or(|facts| {
        let (people, _) = current_people(d);
        family_sources_valid(facts, d)
            && crate::marketplace_sources::valid(facts, &people, &w2_sources(d))
    })
}

pub(crate) fn policy_answer_issue(
    facts: &crate::MarketplaceFacts,
    d: &crate::FederalDraft,
    policy: &crate::MarketplacePolicyRecord,
) -> Option<crate::validation::ValidationIssue> {
    if let Some(issue) = crate::marketplace_sources::policy_answer_issue(policy) {
        return Some(issue);
    }
    let family = family_people(d);
    let (current, births) = current_people(d);
    let people = crate::marketplace_projection::SourcePeople {
        family: &family,
        current: &current,
        births: &births,
    };
    crate::health::policy_costs_issue(facts, &people, policy).or_else(|| {
        let field =
            if !crate::marketplace_projection::allocation_policy_valid(facts, &people, policy) {
                "allocation"
            } else if !crate::marketplace_projection::expense_policy_valid(facts, &people, policy) {
                "payments"
            } else {
                return None;
            };
        Some(
            crate::validation::ValidationIssue::new(
                "MESSAGE_CHECK_THE_MONTHS_AND_AMOUNTS_OF_EACH_FORM_1095_A",
            )
            .at_field(field),
        )
    })
}

pub(crate) fn poverty(residence: MarketplaceResidence, family: i64) -> i64 {
    let (base, extra) = match residence {
        MarketplaceResidence::Contiguous => (15650, 5500),
        MarketplaceResidence::Alaska => (19550, 6880),
        MarketplaceResidence::Hawaii => (17990, 6330),
    };
    base + (family - 1) * extra
}
// Basis points; the household percentage is truncated by Form 8962 Worksheet 2.
pub(crate) fn applicable(percent: i64) -> i64 {
    match percent {
        ..=132 => 210,
        133..=149 => 314 + fraction(percent - 133, 419 - 314, 17),
        150..=199 => 419 + fraction(percent - 150, 660 - 419, 50),
        200..=249 => 660 + fraction(percent - 200, 844 - 660, 50),
        250..=299 => 844 + fraction(percent - 250, 996 - 844, 50),
        _ => 996,
    }
}
pub(crate) struct Contribution {
    pub fpl: i64,
    pub percent: i64,
    pub annual: i64,
    pub monthly: i64,
}
pub(crate) fn contribution(
    residence: MarketplaceResidence,
    income: i64,
    family: i64,
) -> Contribution {
    contribution_with_money(residence, income, family, CalculationMoney::Dollars)
}
pub(crate) fn contribution_with_money(
    residence: MarketplaceResidence,
    income: i64,
    family: i64,
    money: CalculationMoney,
) -> Contribution {
    let fpl = poverty(residence, family);
    let scaled_fpl = fpl * money.scale();
    let percent = if income > 4 * scaled_fpl {
        401
    } else {
        crate::kernel::ratio::trunc_i64(positive(income) * 100, scaled_fpl)
    };
    let annual = fraction(positive(income), applicable(percent), 10000);
    Contribution { fpl, percent, annual, monthly: fraction(annual, 1, 12) }
}

// Both regular and alternative worksheets use this same rounded-column rule.
#[cfg(test)]
pub(crate) fn premium_assistance(row: &CalculatedMonth, contribution: i64, allowed: bool) -> i64 {
    premium_assistance_with_money(row, contribution, allowed, CalculationMoney::Dollars)
}
pub(crate) fn premium_assistance_with_money(
    row: &CalculatedMonth,
    contribution: i64,
    allowed: bool,
    money: CalculationMoney,
) -> i64 {
    if !allowed || !row.eligible {
        return 0;
    }
    phase::cap(money.units(&row.premium), [positive(money.units(&row.benchmark) - contribution)])
}
pub(crate) fn reduce_credit_with_money(
    value: i64,
    reduction: &Exact,
    money: CalculationMoney,
) -> i64 {
    let cents = money.cents(value);
    money.units(&(&cents - phase::cap(cents.clone(), [reduction.clone()])))
}
// One filing projection of the shared work result. Monthly totals sum their
// displayed columns; annual rows round once. Lines26/27 then use those same
// filing24/25 amounts, never independently round a cent work difference.
fn filing_reconciliation(credit: i64, advance: i64, marriage: bool) -> (i64, i64) {
    (if marriage { 0 } else { positive(credit - advance) }, positive(advance - credit))
}

// Form8962 amounts are filing dollars; cent work/source allocations remain
// separate and never receive a manufactured filing-rounding residual.
#[derive(Clone, Default)]
pub(crate) struct MarketplaceOutcome {
    pub refundable: i64,
    pub repayment: i64,
    // Line24 may exceed the advances under an elected marriage alternative,
    // while line26 must stayzero. Expenses consume actual retained subsidy.
    #[cfg(test)]
    pub credit: i64,
    // Actual Step2/4 amount in cents, before final filing-dollar projection.
    pub calculation_credit: i64,
    #[cfg(test)]
    pub expense_credit: i64,
    #[cfg(test)]
    pub expense_monthly: Option<Vec<(u32, i64)>>,
    // Lines 12 to 23 column (e) by month when the monthly lines are used.
    #[cfg(test)]
    pub monthly: Option<Vec<(u32, i64)>>,
    // Months someone in the coverage family was enrolled (Pub. 974 Step 3 line 4).
    #[cfg(test)]
    pub enrolled: i64,
    // Actual column(a) source cents and retained subsidy for each month. The
    // health owner allocates this same credit among linked plans; it never
    // reconstructs gross policy premiums from another expense ledger.
    pub expense_columns: Vec<(u32, Exact, Exact)>,
}
// Pub. 974 Simplified Calculation Method, Step 3 Worksheet lines 3 to 9: the
// self-employed deduction for specified premiums after the part of the
// credit those months carry, limited to Worksheet X line 30.
#[cfg(test)]
pub(crate) fn specified_deduction(
    outcome: &MarketplaceOutcome,
    months: &[u32],
    premiums: i64,
    maximum: i64,
) -> i64 {
    let count = months.len() as i64;
    let monthly = if outcome.expense_credit == outcome.credit {
        &outcome.monthly
    } else {
        &outcome.expense_monthly
    };
    let share = match monthly {
        Some(monthly) if count < 12 && monthly.iter().any(|(_, v)| *v != monthly[0].1) => {
            monthly.iter().filter(|(m, _)| months.contains(m)).map(|(_, v)| *v).sum()
        }
        _ => fraction(outcome.expense_credit, count, outcome.enrolled.max(1)),
    };
    positive(premiums - share).min(maximum)
}
pub(crate) fn calculate(
    d: &crate::FederalDraft,
    income: i64,
    calculation_income: Option<i64>,
    family: &[i64],
    status: FederalStatus,
    dependent: bool,
    r: &mut FederalResult,
) -> MarketplaceOutcome {
    let Some(facts) = d.credits.as_ref().and_then(|c| facts(&c.marketplace)) else {
        return MarketplaceOutcome::default();
    };
    let unclaimed_filer = d.people.as_ref().is_some_and(|people| {
        core::iter::once(&people.taxpayer)
            .chain(people.spouse.iter())
            .any(|person| person.dependency.claimable() && !person.dependency.claimed())
    });
    if dependent && !unclaimed_filer {
        // The claiming taxpayer reconciles this dependent's advances.
        // They do not become a repayment on the dependent's own return.
        emit_lines(
            r,
            &mut (),
            [
                "premium_tax_credit",
                "premium_credit_advance",
                "premium_credit_repayment",
                "premium_credit_refundable",
            ],
            |id, _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(0),
                    &["credits.marketplace", "people"],
                    "https://www.irs.gov/instructions/i8962",
                    msg!("EXPLAIN_MARKETPLACE"),
                ))
            },
        );
        return MarketplaceOutcome::default();
    }
    crate::require_localized!(
        r,
        status == FederalStatus::Separate || facts.forms.iter().flat_map(|p| &p.allocation).all(|a| !matches!(&a.kind, crate::MarketplaceAllocationKind::MarriedSeparate { pair, .. } if pair.one == crate::MarketplaceFamilyRef::ThisReturn || pair.two == crate::MarketplaceFamilyRef::ThisReturn)),
        &crate::IssueContext::Section(crate::TaxSection::Marketplace),
        "Marketplace",
        "ISSUE_AN_ALLOCATION_WITH_YOUR_SPOUSE_APPLIES_ONLY_TO_SEPARATE_RETURNS_FOR_E5EBCD38",
        "An allocation with your spouse applies only to separate returns (Form 8962 Part IV allocation situation 2).",
    );
    crate::require_localized!(
        r,
        crate::marketplace_qsehra::dependent_sources_complete(facts)
            || facts.forms.iter().all(|p| crate::MarketplacePolicyRecordPage::from(&p.form).status
                == crate::MarketplaceStatementStatus::Void),
        &crate::IssueContext::Section(crate::TaxSection::Marketplace),
        "Marketplace",
        "PTC_DEPENDENT_QSEHRA_RECONCILIATION_REQUIRED",
        "A QSEHRA provided to a person you claim needs that employee's own affordability and family relationships. The App cannot substitute your whole-return family bar or omit the permitted-benefit reduction.",
    );
    let (current, births) = current_people(d);
    let people =
        crate::marketplace_projection::SourcePeople { family, current: &current, births: &births };
    let applicable = !dependent
        && (status != FederalStatus::Separate || facts.separate == crate::SeparateRelief::Yes);
    if status == FederalStatus::Separate && facts.separate == crate::SeparateRelief::NotSeparate {
        crate::federal_rules::note_once_scoped(
            r,
            msg!("WARN_MARKETPLACE_SEPARATE_RELIEF_ASSUMED_NO"),
            crate::IssueContext::Section(crate::TaxSection::Marketplace),
        );
    }
    let money = if calculation_income.is_some() {
        CalculationMoney::Cents
    } else {
        CalculationMoney::Dollars
    };
    let income = calculation_income.unwrap_or(income);
    let projection = match crate::marketplace_projection::project_selected_with_income(
        facts,
        &people,
        applicable,
        &money.cents(income),
        None,
        &facts.benchmarks,
        12,
    ) {
        Ok(projection) => projection,
        Err(issues) => {
            for issue in issues {
                let index = facts.forms.iter().position(|p| p.id == issue.policy).unwrap_or(0);
                crate::require_localized!(
                    r,
                    false,
                    &crate::IssueContext::Section(crate::TaxSection::Marketplace)
                        .item("policies", index)
                        .item("months", issue.month as usize - 1),
                    "Marketplace source",
                    issue.missing.resource(),
                    "Complete the identified source facts before reconciling this policy's advance credits.",
                );
            }
            return MarketplaceOutcome::default();
        }
    };
    calculate_months_with_money(
        &projection,
        income,
        family.len() as i64,
        money,
        (!dependent).then_some(crate::marketplace_marriage::Context {
            facts,
            people: &people,
            joint: status == FederalStatus::Joint,
        }),
        r,
    )
}

// One transient representation feeds the existing Form 8962 engine. Source
// identities and allocation agreements remain in the accepted policy records.
pub(crate) struct CalculatedMonth {
    pub month: u32,
    pub premium: Exact,
    #[cfg(test)]
    pub medical_premium: Exact,
    pub credit_reduction: Exact,
    pub benchmark: Exact,
    pub advance: Exact,
    pub eligible: bool,
}

pub(crate) struct CalculatedMarketplace {
    pub residence: MarketplaceResidence,
    pub rows: Vec<CalculatedMonth>,
    pub shared: bool,
    pub applicable_taxpayer: bool,
    pub below_fpl_qualified: bool,
}

#[cfg(test)]
pub(crate) fn calculate_months(
    projection: &CalculatedMarketplace,
    income: i64,
    family: i64,
    r: &mut FederalResult,
) -> MarketplaceOutcome {
    calculate_months_with_marriage(projection, income, family, None, r)
}
#[cfg(test)]
pub(crate) fn calculate_months_with_marriage(
    projection: &CalculatedMarketplace,
    income: i64,
    family: i64,
    marriage: Option<crate::marketplace_marriage::Context<'_>>,
    r: &mut FederalResult,
) -> MarketplaceOutcome {
    calculate_months_with_money(projection, income, family, CalculationMoney::Dollars, marriage, r)
}
pub(crate) fn calculate_months_with_money(
    projection: &CalculatedMarketplace,
    income: i64,
    family: i64,
    money: CalculationMoney,
    marriage: Option<crate::marketplace_marriage::Context<'_>>,
    r: &mut FederalResult,
) -> MarketplaceOutcome {
    let CalculatedMarketplace { residence, rows, shared, applicable_taxpayer, below_fpl_qualified } =
        projection;
    let Contribution { fpl, percent, annual, monthly } = if family == 0 {
        // An unclaimed dependent has no tax family of their own. Only
        // responsible advances are reconciled; PartI cannot invent one.
        Contribution { fpl: 0, percent: 0, annual: 0, monthly: 0 }
    } else {
        contribution_with_money(*residence, income, family, money)
    };
    let annual_advance: Exact = rows.iter().map(|m| &m.advance).sum();
    let mut advance = rows.iter().map(|m| money.units(&m.advance)).sum();
    // Below the poverty line the credit is allowed when advance credit was
    // paid on the Marketplace's enrollment estimate (Form 8962 line 5).
    let allowed =
        *applicable_taxpayer && percent <= 400 && (percent >= 100 || *below_fpl_qualified);
    for CalculatedMonth { premium, benchmark, eligible, .. } in rows {
        crate::require_localized!(
            r,
            !allowed || *premium == whole(0) || !eligible || *benchmark > whole(0),
            &crate::IssueContext::Section(crate::TaxSection::Marketplace),
            "Marketplace",
            "ISSUE_OBTAIN_THE_APPLICABLE_SECOND_LOWEST_COST_SILVER_PLAN_PREMIUM_FROM_T_01337A24",
            "Obtain the applicable second-lowest-cost silver plan premium from the Marketplace or HealthCare.gov's tax tool before claiming credit for a paid coverage month (Form 8962 lines 11 to 23 column (b)).",
        );
    }
    // Part IV sends the return to the monthly lines 12 to 23.
    let mut full_year = !*shared
        && rows.len() == 12
        && rows.iter().all(|v| {
            (&v.premium, &v.benchmark, v.eligible) == (&rows[0].premium, &rows[0].benchmark, true)
        });
    if full_year {
        advance = money.units(&annual_advance);
    }
    let mut column_e: Vec<(u32, i64)> = rows
        .iter()
        .map(|row| {
            (
                row.month,
                reduce_credit_with_money(
                    premium_assistance_with_money(row, monthly, allowed, money),
                    &row.credit_reduction,
                    money,
                ),
            )
        })
        .collect();
    let mut credit = if !allowed {
        0
    } else if full_year {
        let tentative = phase::cap(
            money.units(&(&rows[0].premium * whole(12))),
            [positive(money.units(&(&rows[0].benchmark * whole(12))) - annual)],
        );
        // Pub974 WorksheetQ PartsII/III: an annual Form8962 amount is
        // divided by12 before monthly permitted benefits are capped. It is
        // not replaced by twelve separately rounded monthly PTC amounts.
        let monthly_cents = money.cents(tentative) / whole(12);
        let reduction: Exact = rows
            .iter()
            .map(|m| phase::cap(monthly_cents.clone(), [m.credit_reduction.clone()]))
            .sum();
        money.units(&(money.cents(tentative) - reduction))
    } else {
        column_e.iter().map(|v| v.1).sum()
    };
    let mut elected_marriage = false;
    if let Some(context) = marriage {
        match crate::marketplace_marriage::evaluate_with_money(
            context, projection, income, &column_e, money,
        ) {
            Ok(Some(alternative)) => {
                // The winning optional worksheet forces monthly reconciliation.
                // Its source advances are the regular allocated amounts, once.
                for (month, value) in &mut column_e {
                    if let Some((_, alternative_value)) =
                        alternative.columns.iter().find(|(m, _)| m == month)
                    {
                        *value = *alternative_value;
                    }
                }
                full_year = false;
                advance = rows.iter().map(|m| money.units(&m.advance)).sum();
                credit = column_e.iter().map(|(_, value)| *value).sum();
                elected_marriage = true;
                alternative.print(r);
            }
            Ok(None) => {}
            Err(resource) => crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new(resource, alloc::vec![]),
                crate::IssueContext::Section(crate::TaxSection::Marketplace),
            ),
        }
    }
    let refundable = if elected_marriage { 0 } else { positive(credit - advance) };
    let repayment = positive(advance - credit);
    // Pub502: no expense deduction for PTC or retainedAPTC; an alternative
    // line24 amount above the actual subsidy does not fund any premium.
    let expense_credit = advance + refundable - repayment;
    let expense_monthly = (!full_year).then(|| {
        if expense_credit < credit {
            rows.iter().map(|row| (row.month, money.units(&row.advance))).collect()
        } else {
            column_e.clone()
        }
    });
    let filing_credit = if full_year {
        money.dollars(credit)
    } else {
        column_e.iter().map(|(_, value)| money.dollars(*value)).sum()
    };
    let filing_advance = if full_year {
        money.dollars(advance)
    } else {
        rows.iter().map(|row| money.dollars(money.units(&row.advance))).sum()
    };
    let (filing_refundable, filing_repayment) =
        filing_reconciliation(filing_credit, filing_advance, elected_marriage);
    let credit_note = if !*applicable_taxpayer {
        msg!("EXPLAIN_PREMIUM_TAX_CREDIT")
    } else if allowed && percent < 100 {
        msg!("EXPLAIN_PREMIUM_TAX_CREDIT_2")
    } else {
        msg!("EXPLAIN_PREMIUM_TAX_CREDIT_3")
    };
    for (id, value, explanation) in [
        (
            "marketplace_household_income",
            money.dollars(income),
            msg!("EXPLAIN_MARKETPLACE_HOUSEHOLD_INCOME"),
        ),
        ("marketplace_poverty_line", fpl, msg!("EXPLAIN_MARKETPLACE_POVERTY_LINE")),
        (
            "marketplace_annual_contribution",
            money.dollars(annual),
            msg!("EXPLAIN_MARKETPLACE_ANNUAL_CONTRIBUTION"),
        ),
        ("premium_tax_credit", filing_credit, credit_note),
        ("premium_credit_advance", filing_advance, msg!("EXPLAIN_PREMIUM_CREDIT_ADVANCE")),
        ("premium_credit_repayment", filing_repayment, msg!("EXPLAIN_PREMIUM_CREDIT_REPAYMENT")),
        ("premium_credit_refundable", filing_refundable, msg!("EXPLAIN_PREMIUM_CREDIT_REFUNDABLE")),
    ] {
        // Form 8962 Part III: line 24 total credit, 25 advance, 26 net
        // credit, 27 excess advance payment (the 2026 form reserves lines 28 and 29; line 27 before the repealed
        // line 28 limitation in 2025).
        let printed = match id {
            "premium_tax_credit" => Some("24"),
            "premium_credit_advance" => Some("25"),
            "premium_credit_refundable" => Some("26"),
            "premium_credit_repayment" => Some("27"),
            _ => None,
        };
        let inputs = &["credits.marketplace", "agi"];
        let source = "https://www.irs.gov/irb/2025-32_IRB";
        match printed {
            Some(printed) => form_line(
                r,
                FormLine::new(TaxForm::Form8962, printed),
                id,
                value,
                inputs,
                source,
                explanation,
            ),
            None => line(r, id, value, inputs, source, explanation),
        }
    }
    MarketplaceOutcome {
        refundable: filing_refundable,
        repayment: filing_repayment,
        #[cfg(test)]
        credit: filing_credit,
        calculation_credit: credit * (crate::kernel::ratio::trunc_i64(100, money.scale())),
        #[cfg(test)]
        expense_credit: money.dollars(expense_credit),
        #[cfg(test)]
        expense_monthly: expense_monthly
            .as_ref()
            .map(|v| v.iter().map(|(m, x)| (*m, money.dollars(*x))).collect()),
        #[cfg(test)]
        monthly: (!full_year)
            .then(|| column_e.iter().map(|(m, x)| (*m, money.dollars(*x))).collect()),
        #[cfg(test)]
        enrolled: rows.len() as i64,
        expense_columns: rows
            .iter()
            .map(|row| {
                let retained = expense_monthly.as_ref().map_or_else(
                    || money.cents(expense_credit) / whole(rows.len() as i64),
                    |columns| {
                        money
                            .cents(columns.iter().find(|(m, _)| *m == row.month).map_or(0, |v| v.1))
                    },
                );
                (row.month, row.premium.clone(), retained)
            })
            .collect(),
    }
}

#[cfg(test)]
mod iterative_money_tests {
    use super::*;
    use crate::MarketplaceResidence;
    use alloc::vec;

    fn row(month: u32) -> CalculatedMonth {
        CalculatedMonth {
            month,
            premium: whole(100_049),
            medical_premium: whole(100_049),
            benchmark: whole(110_051),
            advance: whole(65_025),
            credit_reduction: whole(0),
            eligible: true,
        }
    }
    fn projection(rows: Vec<CalculatedMonth>) -> CalculatedMarketplace {
        CalculatedMarketplace {
            residence: MarketplaceResidence::Contiguous,
            rows,
            shared: false,
            applicable_taxpayer: true,
            below_fpl_qualified: false,
        }
    }
    // Derived 2026 endpoint under Pub974(2025), printed52–54 method-wide
    // cents caution; current RevProc2025-25 rate9.96%, not2025 enhanced rates.
    #[test]
    fn iterative_marketplace_money_preserves_monthly_columns_before_form_projection() {
        let source = projection(vec![row(1)]);
        let mut printed = FederalResult::empty(0);
        let outcome = calculate_months_with_money(
            &source,
            6_000_000,
            1,
            CalculationMoney::Cents,
            None,
            &mut printed,
        );
        assert_eq!(outcome.calculation_credit, 60_251);
        assert_eq!(outcome.expense_columns[0].2, whole(60_251));
        assert_eq!(outcome.expense_columns[0].1, whole(100_049));
        assert_eq!(outcome.credit, 603);
        assert_eq!(outcome.repayment, 47);
        assert_eq!(outcome.refundable, 0);
        // Work subsidy602.51 remains independent of filing650−47=603.
        assert_eq!(
            outcome.expense_columns[0].1.clone() - &outcome.expense_columns[0].2,
            whole(39_798)
        );
        for (printed_line, value) in [("24", 603), ("25", 650), ("26", 0), ("27", 47)] {
            assert_eq!(
                printed
                    .lines
                    .iter()
                    .find(|line| line.form_line
                        == Some(FormLine::new(TaxForm::Form8962, printed_line)))
                    .unwrap()
                    .amount,
                value * 100
            );
        }
        let normal = calculate_months(&source, 60_000, 1, &mut FederalResult::empty(0));
        assert_eq!(normal.credit, 603);
        assert_eq!(normal.expense_columns[0].2, whole(60_300));
    }
    #[test]
    fn iterative_filing_totals_sum_monthly_columns_before_reconciliation() {
        let source = projection(
            (1..=2)
                .map(|month| {
                    let mut value = row(month);
                    value.benchmark = whole(110_049);
                    value
                })
                .collect(),
        );
        let outcome = calculate_months_with_money(
            &source,
            6_000_000,
            1,
            CalculationMoney::Cents,
            None,
            &mut FederalResult::empty(0),
        );
        assert_eq!(outcome.calculation_credit, 120_498);
        assert_eq!(outcome.monthly, Some(vec![(1, 602), (2, 602)]));
        assert_eq!(outcome.credit, 1204);
        assert_eq!(outcome.repayment, 96);
        assert_eq!(outcome.expense_columns.iter().map(|row| &row.2).sum::<Exact>(), whole(120_498));
    }
    #[test]
    fn filing_refund_and_marriage_reconcile_without_work_rounding_feedback() {
        assert_eq!(filing_reconciliation(650, 603, false), (47, 0));
        assert_eq!(filing_reconciliation(650, 603, true), (0, 0));
        // Cent work100.51−100.49=2c; filing101−100=$1.
        assert_eq!(filing_reconciliation(100, 101, false), (0, 1));
    }
    // The annual route retains its own amount before monthly QSEHRA caps;
    // twelve separately rounded monthly contributions differ by two cents.
    #[test]
    fn iterative_marketplace_money_preserves_annual_route_and_exact_expense_share() {
        let source = projection(
            (1..=12)
                .map(|month| {
                    let mut r = row(month);
                    r.credit_reduction = whole(1_001);
                    r
                })
                .collect(),
        );
        let outcome = calculate_months_with_money(
            &source,
            6_000_100,
            1,
            CalculationMoney::Cents,
            None,
            &mut FederalResult::empty(0),
        );
        assert_eq!(outcome.calculation_credit, 710_990);
        assert_eq!(outcome.expense_columns.iter().map(|r| &r.2).sum::<Exact>(), whole(710_990));
        assert!(outcome.monthly.is_none());
        assert_eq!(outcome.expense_columns[0].2, ratio(710_990, 12).unwrap());
    }
    #[test]
    fn iterative_marketplace_money_checks_exact_income_before_poverty_truncation() {
        assert_eq!(
            contribution_with_money(
                MarketplaceResidence::Contiguous,
                6_260_000,
                1,
                CalculationMoney::Cents
            )
            .percent,
            400
        );
        assert_eq!(
            contribution_with_money(
                MarketplaceResidence::Contiguous,
                6_260_001,
                1,
                CalculationMoney::Cents
            )
            .percent,
            401
        );
    }
}
