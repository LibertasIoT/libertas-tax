use crate::kernel::lines::{LineValue, emit, row};
use crate::kernel::lines::{line, line_cents};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, positive};
use crate::{FederalDraft, FederalIncome, FederalResult, Owner};
use alloc::{collections::BTreeMap, format};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

/// Owner retirement contribution
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum BusinessRetirement {
    /// None
    None,
    /// Traditional SEP
    /// The plan is timely established and funded; this is your only self-employment business and your only self-employed plan, with no related-employer plan, excess carryover or special treatment. Employees' SEP contributions, at the same plan rate, go on Schedule C line 19.
    Sep {
        /// Employer contribution for your own SEP IRA for 2026
        /// Include timely contributions paid by the return deadline including extensions. Exclude regular personal IRA contributions, Roth SEP contributions, employee contributions and prior-year amounts. Do not list this as a business expense.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        contribution: i64,
        /// Plan contribution rate in hundredths of one percent
        /// Enter the employer rate stated in your plan, before the self-employed adjustment: 1000 means 10%, 1050 means 10.50%, and 2500 means 25%. The App calculates the reduced owner rate; do not enter 20% for a 25% plan.
        // Pub. 560: the SEP/qualified-plan employer rate cannot exceed 25%, in hundredths.
        #[libertas_number(min = 0, max = 2500)]
        rate: u32,
    },
    /// SIMPLE IRA
    /// Your compensation is this business's net earnings from self-employment (Schedule SE line 4) before your SIMPLE contributions. Employees' matching or nonelective contributions go on Schedule C line 19.
    Simple {
        /// Your salary reduction contributions for 2026
        /// At most $17,000, plus a catch-up of $4,000 from age 50 ($5,250 at ages 60 to 63), and not more than your compensation; the App applies your age from your birth date. Your elective deferrals under a 401(k) or other plan of an employer count toward the same yearly limit.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        deferral: i64,
        /// Employer part of the plan
        employer: SimpleEmployer,
        /// Your employer contribution for yourself for 2026
        /// The match or nonelective amount you deposited for your own account by the return due date including extensions. The App deducts no more than the plan formula allows.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        employer_contribution: i64,
    },
    /// 401(k), profit-sharing, money purchase or other defined-contribution plan
    /// One plan of this business; the App figures the Pub. 560 Deduction Worksheet for Self-Employed. Employees' contributions go on Schedule C line 19.
    Qualified {
        /// Plan contribution rate in hundredths of one percent
        /// The employer rate stated in your plan, before the self-employed adjustment: 2500 means 25%. Zero for a plan with elective deferrals only.
        // Pub. 560: the SEP/qualified-plan employer rate cannot exceed 25%, in hundredths.
        #[libertas_number(min = 0, max = 2500)]
        rate: u32,
        /// Employer contribution for yourself for 2026
        /// The profit-sharing or money-purchase amount you deposited for your own account by the return due date including extensions, without the deferrals below.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        contribution: i64,
        /// Your elective deferrals for 2026
        /// At most $24,500 across this plan and any employer plan you deferred into (W-2 box 12 codes D, E, F, S, AA, BB). Catch-up contributions go below.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        deferral: i64,
        /// Your catch-up contributions for 2026
        /// Only from age 50: at most $8,000, or $11,250 at ages 60 to 63; the App applies your age from your birth date.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        catch_up: i64,
        /// Designated Roth part of the deferrals and catch-up
        /// Not deductible; the rest of the deferrals is.
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        #[libertas_default(0)]
        roth: i64,
    },
    /// Defined benefit plan
    /// The deductible contribution is figured by the plan's actuary; the App deducts it up to this business's net earnings and does not check the actuarial limit.
    DefinedBenefit {
        /// Deductible contribution for 2026 from the actuary's report
        #[libertas_money("USD", 2)]
        #[libertas_number(min = 0)]
        contribution: i64,
    },
}

/// Employer part of a SIMPLE IRA
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport,
)]
#[libertas_inline_choices]
pub enum SimpleEmployer {
    /// Matching contribution
    Match {
        /// Match rate in hundredths of one percent
        /// 300 means the usual dollar-for-dollar match up to 3% of compensation; at least 100 (1%) and only in 2 of 5 years below 300.
        // Pub. 560: the SIMPLE employer matching rate cannot exceed 3%, in hundredths.
        #[libertas_number(min = 100, max = 300)]
        rate: u32,
    },
    /// 2% nonelective contribution
    /// 2% of compensation up to $360,000 for every eligible employee and yourself, whether or not they defer.
    Nonelective,
}

impl BusinessRetirement {
    pub(crate) fn valid(&self) -> bool {
        self.rule().is_none()
    }
    pub(crate) fn rule(&self) -> Option<&'static str> {
        self.issue().map(|issue| issue.key)
    }
    pub(crate) fn issue(&self) -> Option<crate::validation::ValidationIssue> {
        use crate::validation::ValidationIssue;
        let invalid = |field| {
            ValidationIssue::new(
                "ISSUE_FEDERAL_RULES_REVIEW_THE_DOCUMENT_AMOUNTS__THEY_MUST_17724C26",
            )
            .at_field(field)
        };
        let deferral_limit =
            |field| ValidationIssue::new("OWNER_PLAN_DEFERRAL_ABOVE_LIMIT").at_field(field);
        match self {
            Self::None => None,
            Self::Sep { contribution, rate } => {
                if !amount_valid(*contribution) {
                    Some(invalid("contribution"))
                } else if *rate > 2500 {
                    Some(invalid("rate"))
                } else {
                    None
                }
            }
            Self::Simple { deferral, employer, employer_contribution } => {
                if !amount_valid(*deferral) {
                    Some(invalid("deferral"))
                } else if *deferral
                    > crate::year_constants::SIMPLE_DEFERRAL_LIMIT_2026_CENTS
                        + crate::year_constants::SIMPLE_CATCH_UP_AGES_60_63_2026_CENTS
                {
                    Some(deferral_limit("deferral"))
                } else if !amount_valid(*employer_contribution) {
                    Some(invalid("employer_contribution"))
                } else if matches!(employer, SimpleEmployer::Match { rate } if !(100..=300).contains(rate))
                {
                    Some(invalid("rate").in_field("employer"))
                } else {
                    None
                }
            }
            Self::Qualified { contribution, rate, deferral, catch_up, roth } => {
                for (field, amount) in [
                    ("contribution", *contribution),
                    ("deferral", *deferral),
                    ("catch_up", *catch_up),
                    ("roth", *roth),
                ] {
                    if !amount_valid(amount) {
                        return Some(invalid(field));
                    }
                }
                if *rate > 2500 {
                    Some(invalid("rate"))
                } else if *deferral > crate::year_constants::ELECTIVE_DEFERRAL_LIMIT_2026_CENTS {
                    Some(deferral_limit("deferral"))
                } else if *catch_up > crate::year_constants::ELECTIVE_CATCH_UP_AGES_60_63_2026_CENTS
                {
                    Some(deferral_limit("catch_up"))
                } else if *roth > *deferral + *catch_up {
                    Some(ValidationIssue::new("OWNER_PLAN_ROTH_ABOVE_DEFERRALS").at_field("roth"))
                } else {
                    None
                }
            }
            Self::DefinedBenefit { contribution } => {
                (!amount_valid(*contribution)).then(|| invalid("contribution"))
            }
        }
    }
}
/// Partnership retirement plan contribution
/// From your K-1 box 13 code R and the partnership's plan statement. A partner cannot establish a separate SEP for this partnership's income.
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum PartnerRetirement {
    /// No plan contribution
    None,
    /// Partnership-established traditional SEP
    /// The contribution is K-1 box 13 code R. The partnership timely established and funded this plan; all eligible employees and partners receive the required contributions; employee costs are already reflected in the K-1. This is your only self-employed retirement plan, with no related-employer aggregation, special allocation, excess carryover or correction.
    Sep {
        /// Partnership plan contribution rate in hundredths of one percent
        /// Use the stated employer rate before the self-employed reduction: 2500 means 25%, not the reduced 20% owner rate.
        // Pub. 560: the SEP/qualified-plan employer rate cannot exceed 25%, in hundredths.
        #[libertas_number(min = 0, max = 2500)]
        rate: u32,
    },
    /// IRA payment or another SEP
    /// This plan's deduction is not calculated yet; the return stops with a message naming it.
    Other,
    /// SIMPLE IRA, 401(k) or other defined-contribution plan, or defined-benefit plan
    /// Only Simple, Qualified and DefinedBenefit, with the same amounts and limits as the business plan of the same kind; the contributions are those you made for yourself.
    Plan(BusinessRetirement),
}
impl PartnerRetirement {
    pub(crate) fn valid(&self) -> bool {
        match self {
            Self::None | Self::Other => true,
            Self::Sep { rate, .. } => *rate <= 2500,
            Self::Plan(plan) => {
                matches!(
                    plan,
                    BusinessRetirement::Simple { .. }
                        | BusinessRetirement::Qualified { .. }
                        | BusinessRetirement::DefinedBenefit { .. }
                ) && plan.valid()
            }
        }
    }
}

// Source cents in; deduction whole dollars out, matching the return line.
pub(crate) fn amounts(contribution: i64, rate: u32, profit: i64, half_se: i64) -> (i64, i64) {
    let earnings = positive(profit - half_se * 100);
    // Pub. 560: r / (1 + r). Keep the rational rate until the cents
    // boundary; never round a permissible deposit upward. The compensation
    // ceiling is multiplied by the original plan rate, not the reduced rate.
    let rate = i64::from(rate);
    let limit = phase::cap(
        phase::cap(
            crate::kernel::ratio::trunc_i64(earnings * rate, 10000 + rate),
            [crate::kernel::ratio::trunc_i64(
                crate::year_constants::SEP_COMPENSATION_LIMIT_CENTS * rate,
                10000,
            )],
        ),
        [crate::year_constants::SEP_ANNUAL_ADDITIONS_LIMIT_CENTS],
    );
    (limit, dollars(phase::cap(contribution, [limit])))
}
#[derive(Default)]
pub(crate) struct SepResult {
    pub owners: [i64; 2],
    pub by_business: BTreeMap<i64, i64>,
    // The same allowed source contribution before its filing-dollar row.
    pub by_business_cents: BTreeMap<i64, i64>,
}
#[cfg(test)]
pub(crate) fn calculate(
    d: &FederalDraft,
    half: [i64; 2],
    half_by_activity: Option<&BTreeMap<i64, i64>>,
    partners: &[(i64, crate::partnership::PartnerSeResult)],
    r: &mut FederalResult,
) -> SepResult {
    match calculate_with_section179(d, half, half_by_activity, partners, None, r, None) {
        Ok(result) => result,
        Err(_) => unreachable!("legacy retirement calculation has no selected allocation"),
    }
}

pub(crate) fn calculate_with_section179(
    d: &FederalDraft,
    half: [i64; 2],
    half_by_activity: Option<&BTreeMap<i64, i64>>,
    partners: &[(i64, crate::partnership::PartnerSeResult)],
    selected: Option<&crate::Section179PreparedAllocation>,
    r: &mut FederalResult,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<SepResult, crate::Section179PreparedIssue> {
    calculate_with_income(d, half, half_by_activity, partners, None, selected, r, home)
}

// AMT changes the chapter 1 deduction limit, not the actual deposit or
// its regular-tax excess-contribution test. Income overrides are source
// cents already computed by the owning business/AMT collector.
// Forward the existing invocation operands and its borrowed home context together.
#[allow(clippy::too_many_arguments)]
pub(crate) fn calculate_amt_with_section179(
    d: &FederalDraft,
    half: [i64; 2],
    half_by_activity: Option<&BTreeMap<i64, i64>>,
    partners: &[(i64, crate::partnership::PartnerSeResult)],
    income: &BTreeMap<i64, i64>,
    selected: Option<&crate::Section179PreparedAllocation>,
    r: &mut FederalResult,
    home: Option<&crate::home_office::HomeContext>,
) -> Result<SepResult, crate::Section179PreparedIssue> {
    calculate_with_income(d, half, half_by_activity, partners, Some(income), selected, r, home)
}

// Forward the existing invocation operands and its borrowed home context together.
#[allow(clippy::too_many_arguments)]
fn calculate_with_income(
    d: &FederalDraft,
    half: [i64; 2],
    half_by_activity: Option<&BTreeMap<i64, i64>>,
    partners: &[(i64, crate::partnership::PartnerSeResult)],
    income: Option<&BTreeMap<i64, i64>>,
    selected: Option<&crate::Section179PreparedAllocation>,
    r: &mut FederalResult,

    home: Option<&crate::home_office::HomeContext>,
) -> Result<SepResult, crate::Section179PreparedIssue> {
    let mut result = SepResult::default();
    for e in &d.income {
        if d.setup.as_ref().is_some_and(crate::community::applies)
            && e.owner == Owner::Spouse
            && matches!(e.income, FederalIncome::Partnership { .. })
        {
            continue;
        }
        let owner = usize::from(e.owner == Owner::Spouse);
        let (contribution, rate, profit) = match &e.income {
            FederalIncome::Business { data: b } => {
                if !matches!(
                    b.retirement,
                    BusinessRetirement::None | BusinessRetirement::Sep { .. }
                ) {
                    let count = d
                        .income
                        .iter()
                        .filter(|v| {
                            v.owner == e.owner && matches!(v.income, FederalIncome::Business { .. })
                        })
                        .count();
                    let plans = d
                        .income
                        .iter()
                        .filter(|v| {
                            v.owner == e.owner
                                && match &v.income {
                                    FederalIncome::Business { data } => {
                                        !matches!(data.retirement, BusinessRetirement::None)
                                    }
                                    FederalIncome::Partnership { data } => {
                                        !matches!(data.retirement, PartnerRetirement::None)
                                    }
                                    _ => false,
                                }
                        })
                        .count();
                    crate::require_localized!(
                        r,
                        count == 1 && plans == 1,
                        &crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits),
                        "Business retirement",
                        "ISSUE_MULTIPLE_SELF_EMPLOYED_PLANS_NEED_COMBINED_PLAN_LIMIT_AND_RELATED_E_661C9DAF",
                        "Multiple self-employed plans need combined plan-limit and related-employer worksheets.",
                    );
                    let profit = match income.and_then(|v| v.get(&e.id)).copied() {
                        Some(profit) => profit,
                        None => {
                            let expenses =
                                crate::section179_return::expenses(d, e.id, b, selected, home)
                                    .map(|expense| {
                                        expense
                                            - selected.map_or(0, |allocation| {
                                                allocation.ordinary_risk_adjustment(e.id)
                                            })
                                    })?;
                            b.earned_receipts() - expenses
                        }
                    };
                    let half = half_by_activity
                        .map_or(half[owner], |v| v.get(&e.id).copied().unwrap_or(0));
                    let allowed = owner_plan(d, e, &b.retirement, profit, half, r);
                    let deduction = dollars(allowed);
                    result.owners[owner] += deduction;
                    result.by_business.insert(e.id, deduction);
                    result.by_business_cents.insert(e.id, allowed);
                    continue;
                }
                let BusinessRetirement::Sep { contribution, rate } = b.retirement else {
                    continue;
                };
                let count = d
                    .income
                    .iter()
                    .filter(|v| {
                        v.owner == e.owner && matches!(v.income, FederalIncome::Business { .. })
                    })
                    .count();
                crate::require_localized!(
                    r,
                    count == 1,
                    &crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits),
                    "Business retirement",
                    "ISSUE_THIS_SEP_WORKSHEET_REQUIRES_ONE_SELF_EMPLOYMENT_BUSINESS_FOR_THIS_O_0FB340F4",
                    "The owner plan worksheet requires one self-employment business and one self-employed plan for this owner.",
                );
                let expenses = crate::section179_return::expenses(d, e.id, b, selected, home).map(
                    |expense| {
                        expense
                            - selected
                                .map_or(0, |allocation| allocation.ordinary_risk_adjustment(e.id))
                    },
                )?;
                (
                    contribution,
                    rate,
                    b.earned_receipts() - expenses
                        + crate::clergy::nondeductible_with_section179(d, e.id, selected, home)?,
                )
            }
            FederalIncome::Partnership { data } => {
                let rate = match data.retirement {
                    PartnerRetirement::None => {
                        // Box 13 code R without its plan (the self-employed
                        // benefits page makes this impossible; T2-43p).
                        crate::require_localized!(
                            r,
                            data.pensions == 0,
                            &crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits),
                            "Partner retirement",
                            "ISSUE_SCHEDULE_K_1_1065_BOX_13_CODE_R_REPORTS_A_RETIREMENT_PLAN_PAYMENT_F_39BE1646",
                            "Schedule K-1 (1065) box 13 code R reports a retirement plan payment for you: choose its plan for this partnership under self-employed health insurance and retirement plan (Schedule 1 line 16; Form 7206 line 9).",
                        );
                        continue;
                    }
                    PartnerRetirement::Other => {
                        crate::require_localized!(
                            r,
                            false,
                            &crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits)
                                .manual(),
                            "Partner retirement",
                            "ISSUE_K_1_BOX_13_CODE_R_FOR_A_SIMPLE_IRA_401_K_DEFINED_BENEFIT_PLAN_OR_IR_F5178D19",
                            "K-1 box 13 code R for an IRA payment or a SEP with other facts (Schedule 1 line 16 or 20; Form 7206 line 9) is not calculated.",
                        );
                        continue;
                    }
                    PartnerRetirement::Sep { rate } => rate,
                    PartnerRetirement::Plan(_) => 0,
                };
                let mut sources = partners.iter().filter(|(id, _)| *id == e.id);
                let Some((_, source)) = sources.next() else {
                    crate::require_localized!(
                        r,
                        false,
                        &crate::interview::income_issue_context(e)
                            .corrective(crate::TaxSection::PassThrough),
                        "Partner retirement",
                        "ISSUE_RESOLVE_ACTUAL_PARTNERSHIP_SE_EARNINGS_BEFORE_CALCULATING_PLAN_COMP_84586D00",
                        "Resolve actual partnership SE earnings before calculating plan compensation.",
                    );
                    continue;
                };
                crate::require_localized!(
                    r,
                    sources.next().is_none(),
                    &crate::interview::income_issue_context(e)
                        .corrective(crate::TaxSection::PassThrough),
                    "Partner retirement",
                    "ISSUE_USE_ONE_RESOLVED_SE_SOURCE_PER_PARTNERSHIP_137CA69B",
                    "Use one resolved SE source per partnership.",
                );
                let Some(reported) = data.reported_se else {
                    crate::require_localized!(
                        r,
                        false,
                        &crate::interview::income_issue_context(e)
                            .corrective(crate::TaxSection::PassThrough),
                        "Partner retirement",
                        "ISSUE_ESTABLISH_K_1_BOX_14_CODE_A_BEFORE_PLAN_COMPENSATION_C9E84E0E",
                        "Establish K-1 box 14 code A before plan compensation.",
                    );
                    continue;
                };
                // Keep current positive source cents: rounded SE tax-line income
                // must not authorize a deposit above the exact plan ceiling.
                // Allowed SE-character losses come from the resolved tax ledger.
                let gross = phase::floor(reported - data.guaranteed_se(), 0) + data.guaranteed_se();
                let rounded = dollars(phase::floor(reported - data.guaranteed_se(), 0))
                    + dollars(data.guaranteed_services)
                    + if !data.limited_for_se { dollars(data.guaranteed_capital) } else { 0 };
                let profit = gross - (rounded - source.earnings) * 100;
                if let PartnerRetirement::Plan(plan) = &data.retirement {
                    single_plan(d, e, r);
                    let profit = income.and_then(|v| v.get(&e.id)).copied().unwrap_or(profit);
                    let half = half_by_activity
                        .map_or(half[owner], |v| v.get(&e.id).copied().unwrap_or(0));
                    let allowed = owner_plan(d, e, plan, profit, half, r);
                    result.owners[owner] += dollars(allowed);
                    result.by_business.insert(e.id, dollars(allowed));
                    result.by_business_cents.insert(e.id, allowed);
                    continue;
                }
                (data.pensions, rate, profit)
            }
            _ => continue,
        };
        single_plan(d, e, r);
        // Pub. 560 chapter 5 starts with actual Schedule C/F profit, including
        // when SE tax was elected on optional earnings.
        // Contributions are source cents;
        // do not let whole-dollar tax-line rounding authorize an excess deposit.
        let (limit, deduction) = amounts(
            contribution,
            rate,
            income.and_then(|v| v.get(&e.id)).copied().unwrap_or(profit),
            half_by_activity
                .map_or(half[owner], |amounts| amounts.get(&e.id).copied().unwrap_or(0)),
        );
        crate::require_localized!(
            r,
            income.is_some() || contribution <= limit,
            &crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits),
            "Business retirement",
            "ISSUE_THE_SEP_CONTRIBUTION_EXCEEDS_THE_2026_INCOME_BASED_PLAN_RATE_COMPEN_A4C70725",
            "The SEP contribution exceeds the 2026 income-based, plan-rate compensation or $72,000 annual limit; excess correction or carryover needs a separate worksheet.",
        );
        result.owners[owner] += deduction;
        result.by_business.insert(e.id, deduction);
        result.by_business_cents.insert(e.id, phase::cap(contribution, [limit]));
        line_cents(
            r,
            &format!("business_{}_sep_limit", e.id),
            limit,
            &["income.business", "self_employment_tax"],
            "https://www.irs.gov/retirement-plans/sep-contribution-limits-including-grandfathered-sarseps",
            msg!("EXPLAIN_BUSINESS_N_SEP_LIMIT"),
        );
        line(
            r,
            &format!("business_{}_sep_deduction", e.id),
            deduction,
            &["income.business.retirement"],
            "https://www.irs.gov/publications/p560",
            msg!("EXPLAIN_BUSINESS_N_SEP_DEDUCTION"),
        );
    }
    Ok(result)
}

// One self-employed plan per owner (Pub. 560: combined plan limits are separate worksheets).
fn single_plan(d: &FederalDraft, e: &crate::FederalIncomeEntry, r: &mut FederalResult) {
    let plans = d
        .income
        .iter()
        .filter(|v| {
            v.owner == e.owner
                && match &v.income {
                    FederalIncome::Business { data } => {
                        !matches!(data.retirement, BusinessRetirement::None)
                    }
                    FederalIncome::Partnership { data } => {
                        matches!(
                            data.retirement,
                            PartnerRetirement::Sep { .. } | PartnerRetirement::Plan(_)
                        )
                    }
                    _ => false,
                }
        })
        .count();
    crate::require_localized!(
        r,
        plans == 1,
        &crate::IssueContext::Section(crate::TaxSection::SelfEmployedBenefits).manual(),
        "Business retirement",
        "ISSUE_MULTIPLE_SELF_EMPLOYED_PLANS_NEED_COMBINED_PLAN_LIMIT_AND_RELATED_E_661C9DAF",
        "Multiple self-employed plans need combined plan-limit and related-employer worksheets.",
    );
}

fn amount_valid(v: i64) -> bool {
    (0..=100000000000).contains(&v)
}

fn catch_up(age: u32, simple: bool) -> i64 {
    use crate::year_constants::*;
    match (age, simple) {
        (60..=63, true) => SIMPLE_CATCH_UP_AGES_60_63_2026_CENTS,
        (50.., true) => SIMPLE_CATCH_UP_2026_CENTS,
        (60..=63, false) => ELECTIVE_CATCH_UP_AGES_60_63_2026_CENTS,
        (50.., false) => ELECTIVE_CATCH_UP_2026_CENTS,
        _ => 0,
    }
}
// Deduction Worksheet for Self-Employed, Pub.560 chapter5. Source cents remain
// exact through the reduced employer rate; the output deduction is rounded once.
pub(crate) fn worksheet(
    earnings: i64,
    rate: u32,
    deferral: i64,
    catch: i64,
    roth: i64,
) -> [i64; 22] {
    use crate::year_constants::*;
    let mut s = [0; 22];
    s[3] = phase::floor(earnings, 0);
    s[5] = crate::kernel::ratio::trunc_i64(s[3] * i64::from(rate), 10000 + i64::from(rate));
    s[6] = crate::kernel::ratio::trunc_i64(SEP_COMPENSATION_LIMIT_CENTS * i64::from(rate), 10000);
    s[7] = phase::cap(s[5], [s[6]]);
    s[8] = SEP_ANNUAL_ADDITIONS_LIMIT_CENTS;
    s[9] = phase::floor(phase::cap(deferral, [ELECTIVE_DEFERRAL_LIMIT_2026_CENTS]), 0);
    s[10] = phase::floor(s[8] - s[9], 0);
    s[11] = phase::floor(s[3] - s[9], 0);
    s[12] = crate::kernel::ratio::trunc_i64(s[11], 2);
    s[13] = phase::cap(s[7], [s[10], s[12]]);
    s[14] = phase::floor(s[3] - s[13], 0);
    s[15] = phase::cap(s[9], [s[14]]);
    s[16] = phase::floor(s[14] - s[15], 0);
    s[17] = phase::floor(catch, 0);
    s[18] = phase::cap(s[16], [s[17]]);
    s[19] = s[13] + s[15] + s[18];
    s[20] = phase::cap(roth, [s[15] + s[18]]);
    s[21] = phase::floor(s[19] - s[20], 0);
    s
}
pub(crate) fn simple(
    profit: i64,
    deferral: i64,
    employer: SimpleEmployer,
    deposit: i64,
    age: u32,
    other_deferrals: i64,
) -> (i64, i64) {
    use crate::year_constants::*;
    let comp = crate::kernel::ratio::fraction(phase::floor(profit, 0), 9235, 10000);
    let cap = phase::cap(
        SIMPLE_DEFERRAL_LIMIT_2026_CENTS + catch_up(age, true),
        [phase::floor(
            ELECTIVE_DEFERRAL_LIMIT_2026_CENTS + catch_up(age, false) - other_deferrals,
            0,
        )],
    );
    let allowed = phase::cap(deferral, [cap, comp]);
    let employer_cap = match employer {
        SimpleEmployer::Match { rate } => {
            phase::cap(allowed, [crate::kernel::ratio::trunc_i64(comp * i64::from(rate), 10000)])
        }
        SimpleEmployer::Nonelective => crate::kernel::ratio::trunc_i64(
            phase::cap(comp, [SEP_COMPENSATION_LIMIT_CENTS]) * 2,
            100,
        ),
    };
    let actual = allowed + phase::cap(deposit, [employer_cap]);
    (deferral + deposit, actual)
}
fn owner_plan(
    d: &FederalDraft,
    entry: &crate::FederalIncomeEntry,
    plan: &BusinessRetirement,
    profit: i64,
    half: i64,
    r: &mut FederalResult,
) -> i64 {
    use crate::year_constants::*;
    let age = d
        .people
        .as_ref()
        .and_then(|p| {
            if entry.owner == Owner::Taxpayer { Some(&p.taxpayer) } else { p.spouse.as_ref() }
        })
        .map_or(0, |p| crate::federal_model::age_at_year_end(p.birth_date));
    // A partner's K-1 plan rows are labelled as the partnership's, not a business's.
    let (kind, step_prefix) = if matches!(entry.income, FederalIncome::Partnership { .. }) {
        ("partnership", "partner_owner_plan")
    } else {
        ("business", "owner_plan")
    };
    let other = crate::elective_deferrals::total(d, entry.owner);
    let earnings = phase::floor(profit - half * 100, 0);
    let (actual, allowed) = match *plan {
        BusinessRetirement::Simple { deferral, employer, employer_contribution } => {
            simple(profit, deferral, employer, employer_contribution, age, other)
        }
        BusinessRetirement::Qualified {
            rate,
            contribution,
            deferral,
            catch_up: catch_up_amount,
            roth,
        } => {
            let catch = catch_up_amount;
            let regular =
                phase::cap(deferral, [phase::floor(ELECTIVE_DEFERRAL_LIMIT_2026_CENTS - other, 0)]);
            let catch = phase::cap(
                catch,
                [phase::floor(
                    catch_up(age, false)
                        - phase::floor(other - ELECTIVE_DEFERRAL_LIMIT_2026_CENTS, 0),
                    0,
                )],
            );
            let mut steps = worksheet(earnings, rate, regular, catch, roth);
            steps[1] = profit;
            steps[2] = half * 100;
            emit(
                r,
                &mut (),
                [1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21],
                |n, _, _| {
                    Some(row(
                        None,
                        &format!("{step_prefix}_{}_step{n}", entry.id),
                        LineValue::MoneyCents(steps[n]),
                        &["income.business.retirement"],
                        "https://www.irs.gov/publications/p560",
                        crate::Message::new(
                            "EXPLAIN_OWNER_PLAN_STEP",
                            alloc::vec![crate::MessageArg::Number(n as i64)],
                        ),
                    ))
                },
            );
            let actual = contribution + deferral + catch_up_amount - roth;
            (actual, phase::cap(actual, [steps[21]]))
        }
        BusinessRetirement::DefinedBenefit { contribution } => {
            crate::federal_rules::note_once_scoped(
                r,
                crate::Message::new("NOTE_DEFINED_BENEFIT_ACTUARY", alloc::vec![]),
                crate::interview::income_issue_context(entry)
                    .corrective(crate::TaxSection::SelfEmployedBenefits),
            );
            (contribution, phase::cap(contribution, [earnings]))
        }
        _ => return 0,
    };
    if allowed < actual {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new("NOTE_OWNER_PLAN_LIMITED", alloc::vec![]),
            crate::interview::income_issue_context(entry)
                .corrective(crate::TaxSection::SelfEmployedBenefits),
        );
    }
    line_cents(
        r,
        &format!("{kind}_{}_owner_plan_deduction", entry.id),
        allowed,
        &["income.business.retirement"],
        "https://www.irs.gov/publications/p560",
        crate::Message::new("EXPLAIN_OWNER_PLAN_DEDUCTION", alloc::vec![]),
    );
    allowed
}
