pub use crate::adoption_source::{
    AdoptionBenefitHistory, AdoptionBenefits, AdoptionCarry, AdoptionChild,
    AdoptionEmployerPayment, AdoptionExpense, AdoptionFinalization, AdoptionKind, AdoptionLimits,
    AdoptionPriorCredit, JointAdoptionExclusion,
};
pub(crate) use crate::adoption_source::{timing, valid};
// Form 8839; annual parameters are from Rev. Proc. 2025-32.
use crate::kernel::lines::{LineValue, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::line_emission::emit as emit_lines;
use crate::line_emission::line;
use crate::{FederalResult, FederalSetup, FederalStatus, Owner};
use alloc::{string::String, vec::Vec};
use libertas_macros::{LibertasAvroDecode, LibertasAvroEncode, LibertasExport};

pub(crate) struct Credit {
    pub refundable: i64,
    pub nonrefundable: i64,
    current: i64,
}
pub(crate) fn calculate(
    children: &[AdoptionChild],
    carries: &[AdoptionCarry],
    magi: i64,
    setup: &FederalSetup,
    r: &mut FederalResult,
) -> Credit {
    let mut credit = Credit { refundable: 0, nonrefundable: 0, current: 0 };
    for (index, child) in children.iter().enumerate() {
        let context = child_context(index, child);
        if !child.eligible {
            child_left_out(child, &context, r);
            continue;
        }
        let prior = dollars(child.prior.previous) + dollars(child.prior.expenses);
        let limits = child.limits.amounts(&context, r);
        emit_lines(
            r,
            &mut (),
            [("credit", limits[0]), ("exclusion", limits[1])],
            |(kind, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("adoption_{}_{}_limit", index + 1, kind),
                    LineValue::MoneyDollars(amount),
                    &["credits.adoptions.limits"],
                    "https://www.irs.gov/instructions/i8839",
                    crate::Message::new(
                        "EXPLAIN_ADOPTION_N_N_LIMIT",
                        alloc::vec![crate::MessageArg::Text(child.label.clone())],
                    ),
                ))
            },
        );
        let cap = positive(limits[0] - prior);
        let (finalized, domestic) = timing(&child.kind);
        let special = matches!(child.kind, AdoptionKind::Domestic { special_needs: true, .. })
            && finalized == Some(2026);
        let expenses: i64 = child
            .expenses
            .iter()
            .filter(|e| {
                if finalized.is_some_and(|y| y < 2026) {
                    e.year == 2026
                } else if domestic {
                    e.year == 2025 || (e.year == 2026 && finalized == Some(2026))
                } else {
                    finalized == Some(2026)
                }
            })
            .map(|e| e.amount)
            .sum();
        let base = if special { cap } else { phase::cap(cap, [dollars(expenses)]) };
        let base = eligible_current_base(child, &context, setup, base, r);
        let current = phase::linear_remaining(
            base,
            magi,
            crate::year_constants::ADOPTION_PHASEOUT_START_DOLLARS,
            crate::year_constants::ADOPTION_PHASEOUT_RANGE_DOLLARS,
        );
        let refundable = phase::cap(current, [crate::year_constants::ADOPTION_REFUNDABLE_DOLLARS]);
        credit.refundable += refundable;
        credit.current += current - refundable;
    }
    credit.nonrefundable = credit.current + carries.iter().map(|v| dollars(v.amount)).sum::<i64>();
    credit
}

pub(crate) fn carryforward(
    credit: &Credit,
    carries: &[AdoptionCarry],
    used: i64,
    r: &mut FederalResult,
) {
    use crate::kernel::carry::{Amount, Dollars, Entry, consume_by_key};
    let entries: alloc::vec::Vec<_> = (2021..=2026)
        .map(|year| {
            let available = if year == 2026 {
                credit.current
            } else {
                carries.iter().filter(|v| v.year == year).map(|v| dollars(v.amount)).sum()
            };
            Entry {
                key: year,
                amount: Amount::<Dollars>::new(available).expect("nonnegative adoption credit"),
                last_use_year: Some(year + 5),
            }
        })
        .collect();
    let settlements = consume_by_key(
        &entries,
        Amount::new(used).expect("nonnegative credit use"),
        2026,
        |year| *year,
    )
    .expect("adoption years are ordered");
    let mut total = 0;
    for (entry, settlement) in entries.iter().zip(settlements) {
        let year = entry.key;
        let available = entry.amount.get();
        // The expired oldest balance is still displayed, but not carried.
        let remaining = settlement.closing.get() + settlement.expired.get();
        if year > 2021 {
            total += remaining;
        }
        if available > 0 {
            line(
                r,
                &alloc::format!(
                    "adoption_{}_{}",
                    if year == 2021 { "expired" } else { "carry" },
                    year
                ),
                remaining,
                &["credits.adoptions", "credits.adoption_carry"],
                "https://www.irs.gov/instructions/i8839",
                if year == 2021 {
                    msg!("EXPLAIN_ADOPTION_N_N")
                } else {
                    msg!("EXPLAIN_ADOPTION_N_N_2")
                },
            );
        }
    }
    line(
        r,
        "adoption_carryforward",
        total,
        &["credits.adoptions", "credits.adoption_carry"],
        "https://www.irs.gov/instructions/i8839",
        msg!("EXPLAIN_ADOPTION_CARRYFORWARD"),
    );
}

/// One employer's adoption assistance for this child, on the page
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AdoptionEmployerPick {
    /// Employee receiving the benefit
    pub owner: Owner,
    /// Employer's W-2
    /// Choose one ordinary W-2 of this employee. Use the corrected W-2 instead of entering the original and correction twice.
    #[libertas_enum_source("^<AdoptionChildPage>.wages")]
    pub wage: i64,
    /// This employer's W-2 box 12 code T for this child
    /// Enter the qualified-plan amount paid in 2026 as whole dollars. It is already in that W-2's Social Security and Medicare wages, but not box 1 or box 2; do not add a separate withholding amount.
    #[libertas_money("USD", 2)]
    // Leave headroom for exact cents totals.
    #[libertas_number(min = 100, max = 100000000000)]
    pub received: i64,
}

/// Employer adoption assistance for this child, on the page
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub enum AdoptionBenefitsPage {
    /// No employer adoption benefits or special-needs plan exclusion
    None,
    /// Employer adoption assistance program
    Program {
        /// Employee receiving the assistance
        owner: Owner,
        /// Current-year benefits, Form W-2 box 12 code T
        /// Combine assistance for this child from this employee's employers. These benefits must not already be in W-2 box 1. Use zero for an eligible special-needs plan exclusion without actual benefits.
        #[libertas_money("USD", 2)]
        // Leave headroom for exact cents totals.
        #[libertas_number(min = 0, max = 100000000000)]
        received: i64,
        /// Earlier benefits for this child
        history: AdoptionBenefitHistory,
        /// Ordinary qualified plan and complete benefit records
        /// Confirm a written qualified assistance plan; no more-than-2% S-corporation ownership, benefits for a second employee, benefits already included in W-2 box 1, or unreported earlier benefits. Credit expenses exclude all reimbursed amounts, even if the benefits become taxable. Your employer's plan documents show it. No treats these benefits as taxable, with a notice.
        /// Check the employer plan, matched Forms W-2 and earlier benefit records.
        ordinary: bool,
    },
    /// Qualified benefits paid to both spouses by separate employers
    JointEmployees {
        /// 2026 W-2 code T payments for this child
        /// Enter each employer once, including different employers of the same employee. Both spouses must have a payment. When two children use this path, every matched W-2's entire code T amount must belong to exactly one child; do not reuse a W-2 across children. The App applies one exclusion per child and never adds code T to W-2 box 1 or withholding.
        #[libertas_size(min = 2)]
        payments: Vec<AdoptionEmployerPick>,
        /// Employee attribution of a partly available actual-benefit exclusion
        /// Automatic splits a partly excluded child's benefits in proportion to each employee's code T; it only moves each spouse's earned income for Form 2441. Choose a confirmed split only to document a different one from supporting records; it cannot change Form 8839 line 29.
        allocation: JointAdoptionExclusion,
        /// Earlier benefits for this child
        /// Benefits excluded for this child on an earlier Form 8839 reduce this year's $17,670 limit (line 20). Foreign benefits taxed before the adoption became final are not supported on this path.
        history: AdoptionBenefitHistory,
        /// Ordinary plans and complete W-2s confirmed
        /// Yes on a joint return for U.S. children: each employer had a written qualified adoption assistance plan, every matched W-2's whole code T amount is entered once for one child and sits in that W-2's Social Security and Medicare wages but not box 1 or box 2, and no reimbursed expense dollar belongs to another child. A special-needs child whose adoption became final in 2026 excludes the whole remaining limit whatever the code T; any excess reduces joint Form 1040 line 1f. Credit expenses exclude all reimbursements. No treats these benefits as taxable, with a notice.
        /// Check the employer plan, matched Forms W-2 and earlier benefit records.
        ordinary: bool,
    },
}

/// Adoption expenses and benefits for one child, on the page
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
pub struct AdoptionChildPage {
    /// Child or adoption record label
    #[libertas_size(min = 1)]
    pub label: String,
    /// Jurisdiction and finalization
    pub kind: AdoptionKind,
    /// Per-child limits available to this return
    pub limits: AdoptionLimits,
    /// Payments from your records
    pub expenses: Vec<AdoptionExpense>,
    /// Previous claim for the same child
    pub prior: AdoptionPriorCredit,
    /// Employer adoption assistance
    pub benefits: AdoptionBenefitsPage,
    /// W-2 employers on this return
    #[libertas_hidden]
    #[libertas_read_only]
    #[libertas_output_only]
    pub wages: Vec<crate::WageChoice>,
    /// Child lived in your home for more than half of 2026
    /// Required for the current-credit or benefit-exclusion exception when married filing separately. The App combines this fact with your marital living arrangement and home-cost contribution in Setup. Other filing statuses do not require this adoption residence test. School, medical or custody records show the child's address.
    pub home_over_half_year: bool,
    /// Eligible child and expenses confirmed
    /// The child was under 18 or unable to care for themselves when expenses were paid; this is not adoption of your spouse's child. Requirements for identifying the child and lawful adoption are met. Combine related attempts for the same child, and resolve unsuccessful-attempt allocations before entry. Credit expenses exclude every employer reimbursement. Keep the per-child limit agreement consistent with every other adopting taxpayer. Adoption papers, the child's age or special-needs determination and your expense receipts show it. No leaves this child's credit and employer-benefit exclusion out of the estimate with a notice; that situation is not supported.
    pub eligible: bool,
}

// The page counterparts of the stored adoption records: the same facts, with
// each employer payment choosing its W-2 from the keyed catalog its child
// carries (the stored child has none, and More credits also stores refused
// credits as the same union, so the catalog sits below it).
impl AdoptionChild {
    pub(crate) fn page(&self, wages: &[crate::WageChoice]) -> AdoptionChildPage {
        AdoptionChildPage {
            label: self.label.clone(),
            kind: self.kind.clone(),
            limits: self.limits.clone(),
            expenses: self.expenses.clone(),
            prior: self.prior.clone(),
            benefits: (&self.benefits).into(),
            wages: wages.to_vec(),
            home_over_half_year: self.home_over_half_year,
            eligible: self.eligible,
        }
    }
}
impl AdoptionChildPage {
    pub(crate) fn record(&self) -> AdoptionChild {
        AdoptionChild {
            label: self.label.clone(),
            kind: self.kind.clone(),
            limits: self.limits.clone(),
            expenses: self.expenses.clone(),
            prior: self.prior.clone(),
            benefits: (&self.benefits).into(),
            home_over_half_year: self.home_over_half_year,
            eligible: self.eligible,
        }
    }
    // The W-2 identities this child's employer payments name.
    pub(crate) fn payments(&self) -> &[AdoptionEmployerPick] {
        match &self.benefits {
            AdoptionBenefitsPage::JointEmployees { payments, .. } => payments,
            AdoptionBenefitsPage::None | AdoptionBenefitsPage::Program { .. } => &[],
        }
    }
}
record_page_conversions!(AdoptionEmployerPayment, AdoptionEmployerPick, [&], |value, Target| {
    Target { owner: value.owner, wage: value.wage, received: value.received }
});

record_page_conversions!(AdoptionBenefits, AdoptionBenefitsPage, [&], |value, Target, Source| {
    match value {
        Source::None => Target::None,
        Source::Program { owner, received, history, ordinary } => Target::Program {
            owner: *owner,
            received: *received,
            history: history.clone(),
            ordinary: *ordinary,
        },
        Source::JointEmployees { payments, allocation, history, ordinary } => Target::JointEmployees {
            payments: payments.iter().map(Into::into).collect(),
            allocation: allocation.clone(),
            history: history.clone(),
            ordinary: *ordinary,
        },
    }
});

#[derive(Default)]
pub(crate) struct Benefits {
    pub wages: [i64; 2],
    // Subset of wages supported by actual current W2 codeT reconciliation.
    // Both arrays are signed native annual Dollars, not recovered cents.
    pub w2_wages: [i64; 2],
    // Ownership above remains known when an adoption-page-only Program has
    // no actual W2 source. This flag describes source classification only.
    pub source_class_complete: bool,
    pub excluded: i64,
    pub prior_taxable: i64,
    // Form 8839 line 29 can exceed actual code T for a finalized special-needs
    // child. Its excess reduces joint line 1f, not either employee's earnings.
    pub deemed_excess: i64,
}
pub(crate) fn has_benefits(children: &[AdoptionChild]) -> bool {
    children.iter().any(|c| !matches!(c.benefits, AdoptionBenefits::None))
}
pub(crate) fn benefits(
    children: &[AdoptionChild],
    income: &[crate::FederalIncomeEntry],
    setup: &FederalSetup,
    magi: Option<i64>,
    r: &mut FederalResult,
) -> Benefits {
    let mut result = Benefits {
        source_class_complete: magi.is_some() || !has_benefits(children),
        ..Default::default()
    };
    // Actual signed Program amounts with positive current receipts. The
    // existing codeT owner reconciler decides whether W2s cover them below.
    let mut program_taxable = [0; 2];
    let mut program_requires_w2 = [false; 2];
    let mut program_page_only = false;
    let status = setup.filing_status();
    for (child_index, child) in children.iter().enumerate() {
        let context = child_context(child_index, child);
        if let AdoptionBenefits::JointEmployees { payments, allocation, history, ordinary } =
            &child.benefits
        {
            let total = payments.iter().map(|p| p.received).sum::<i64>();
            let gross = dollars(total);
            let mut gross_by_employee = [0; 2];
            for payment in payments {
                gross_by_employee[usize::from(payment.owner == Owner::Spouse)] +=
                    dollars(payment.received);
            }
            let sources_match = payments.iter().all(|p| {
                let mut sources = income
                    .iter()
                    .filter(|entry| entry.owner == crate::DocumentOwner::from(p.owner) && entry.id == p.wage);
                matches!(
                    (sources.next(), sources.next()),
                    (Some(crate::FederalIncomeEntry {
                        income: crate::FederalIncome::Wage { data },
                        ..
                    }), None) if { let data = crate::FederalWage::from(data); data.medicare_wages >= data.wages + p.received
                        && data.social_wages >= (data.wages + p.received).min(crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS) }
                )
            });
            // Form 8839 line 20: benefits excluded for this child before 2026
            // reduce this year's limit; foreign benefits taxed before the
            // finalization need the foreign worksheet.
            let history_used = match history {
                AdoptionBenefitHistory::Exclusion { used } => dollars(*used),
                _ => 0,
            };
            let cap = positive(child.limits.amounts(&context, r)[1] - history_used);
            // Line 24: a state- or tribe-determined special-needs child whose
            // adoption became final in 2026 excludes the whole remaining limit
            // "whether or not" employer benefits were received.
            let special = matches!(
                child.kind,
                AdoptionKind::Domestic {
                    finalized: AdoptionFinalization::Final { year: 2026 },
                    special_needs: true,
                }
            );
            let domestic = matches!(child.kind, AdoptionKind::Domestic { .. });
            // A W-2 whose code T is entered for one child is no other child's.
            let distinct_child_sources = children.iter().enumerate().all(|(other_index, other)| {
                let AdoptionBenefits::JointEmployees { payments: other_payments, .. } =
                    &other.benefits
                else {
                    return true;
                };
                other_index == child_index
                    || !payments.iter().any(|payment| {
                        other_payments.iter().any(|other_payment| {
                            payment.owner == other_payment.owner
                                && payment.wage == other_payment.wage
                        })
                    })
            });
            if !child.eligible || !*ordinary {
                // No exclusion: every code T dollar stays taxable wages.
                if child.eligible {
                    benefits_taxable(child, &context, r);
                } else {
                    child_left_out(child, &context, r);
                }
                result.wages[0] += gross_by_employee[0];
                result.wages[1] += gross_by_employee[1];
                if sources_match && distinct_child_sources {
                    for (wages, gross) in result.w2_wages.iter_mut().zip(gross_by_employee) {
                        *wages += gross;
                    }
                } else {
                    result.source_class_complete = false;
                }
                continue;
            }
            // The foreign worksheet (the prior-year benefits divided between
            // the two employees) is the one case still refused.
            let unsupported_worksheet =
                !domestic || matches!(history, AdoptionBenefitHistory::TaxedForeign { .. });
            let source_complete = setup.filing_status() == FederalStatus::Joint
                && !unsupported_worksheet
                && sources_match
                && distinct_child_sources;
            let excluded = magi.map(|value| {
                let base = if special { cap } else { phase::cap(cap, [gross]) };
                phase::linear_remaining(
                    base,
                    value,
                    crate::year_constants::ADOPTION_PHASEOUT_START_DOLLARS,
                    crate::year_constants::ADOPTION_PHASEOUT_RANGE_DOLLARS,
                )
            });
            let mut unattributed = false;
            let employee_excluded = if source_complete {
                match (excluded, allocation) {
                    (Some(amount), JointAdoptionExclusion::Automatic) if amount >= gross => {
                        Some(gross_by_employee)
                    }
                    (Some(0), JointAdoptionExclusion::Automatic) => Some([0, 0]),
                    // A partly excluded child: each employee's part follows
                    // their share of the code T, the remainder to the spouse.
                    (Some(amount), JointAdoptionExclusion::Automatic) => {
                        let taxpayer = fraction(amount, gross_by_employee[0], gross);
                        Some([taxpayer, amount - taxpayer])
                    }
                    // An unconfirmed split leaves the partial exclusion out.
                    (Some(amount), JointAdoptionExclusion::Attributed { confirmed: false, .. })
                        if amount > 0 && amount < gross =>
                    {
                        crate::federal_rules::note_once_scoped(
                            r,
                            crate::Message::new(
                                "WARN_ADOPTION_EXCLUSION_SPLIT_LEFT_OUT",
                                alloc::vec![crate::MessageArg::Text(child.label.clone())],
                            ),
                            context.clone(),
                        );
                        unattributed = true;
                        Some([0, 0])
                    }
                    (
                        Some(amount),
                        JointAdoptionExclusion::Attributed { taxpayer, spouse, confirmed: true },
                    ) if amount > 0
                        && amount < gross
                        && dollars(*taxpayer) <= gross_by_employee[0]
                        && dollars(*spouse) <= gross_by_employee[1]
                        && dollars(*taxpayer) + dollars(*spouse) == amount =>
                    {
                        Some([dollars(*taxpayer), dollars(*spouse)])
                    }
                    _ => None,
                }
            } else {
                None
            };
            if !source_complete || unattributed {
                result.source_class_complete = false;
            }
            let excluded = if unattributed { Some(0) } else { excluded };
            let corrective_context = if unsupported_worksheet {
                context.clone().manual()
            } else if setup.filing_status() != FederalStatus::Joint {
                context
                    .clone()
                    .alternatives(&[crate::TaxSection::Setup, crate::TaxSection::MoreCredits])
            } else if !sources_match {
                context
                    .clone()
                    .alternatives(&[crate::TaxSection::MoreCredits, crate::TaxSection::Wages])
            } else {
                context.clone()
            };
            crate::require_localized!(
                r,
                source_complete && (magi.is_none() || employee_excluded.is_some()),
                &corrective_context,
                "Adoption benefits",
                "ISSUE_CONFIRM_DISJOINT_CORRECTED_W_2_CODE_T_SOURCES_QUALIFIED_PLANS_NO_PR_6D13D308",
                "Confirm disjoint corrected W-2 code T sources, qualified plans and a joint return for domestic children; benefits for a foreign child paid to both spouses, and foreign benefits taxed before finalization, are not calculated. Deemed line 29 excess reduces joint line 1f, not either employee's earnings.",
            );
            if children.len() > 1
                && let Some(excluded) = excluded
                && employee_excluded.is_some()
            {
                emit_lines(
                    r,
                    &mut (),
                    [
                        (
                            "excluded",
                            excluded,
                            msg!("EXPLAIN_ADOPTION_CHILD_N_BENEFITS_N_EXCLUDED"),
                        ),
                        (
                            "taxable",
                            gross - excluded,
                            msg!("EXPLAIN_ADOPTION_CHILD_N_BENEFITS_N_TAXABLE"),
                        ),
                    ],
                    |(suffix, amount, explanation), _, _| {
                        Some(calculation_row(
                            None,
                            &alloc::format!(
                                "adoption_child_{}_benefits_{}",
                                child_index + 1,
                                suffix
                            ),
                            LineValue::MoneyDollars(amount),
                            &["credits.adoptions.benefits", "income.wage"],
                            "https://www.irs.gov/pub/irs-dft/f8839--dft.pdf",
                            explanation,
                        ))
                    },
                );
            }
            for (index, payment) in payments.iter().enumerate() {
                if magi.is_some() && employee_excluded.is_some() {
                    let id = if children.len() == 1 {
                        alloc::format!("adoption_employer_payment_{}", index + 1)
                    } else {
                        alloc::format!(
                            "adoption_child_{}_employer_payment_{}",
                            child_index + 1,
                            index + 1
                        )
                    };
                    line(
                        r,
                        &id,
                        dollars(payment.received),
                        &["credits.adoptions.benefits", "income.wage"],
                        "https://www.irs.gov/instructions/iw2w3",
                        crate::Message::new(
                            "EXPLAIN_ADOPTION",
                            alloc::vec![income.iter().find(|e| e.id == payment.wage).map_or(
                                crate::MessageArg::Number(payment.wage),
                                |e| crate::MessageArg::Nested(alloc::boxed::Box::new(
                                    crate::interview::income_issue_context(e).label()
                                ))
                            )],
                        ),
                    );
                }
            }
            if let Some(employee_excluded) = employee_excluded {
                for (index, employee) in ["taxpayer", "spouse"].iter().enumerate() {
                    for (suffix, amount) in [
                        ("excluded", employee_excluded[index]),
                        ("taxable", gross_by_employee[index] - employee_excluded[index]),
                    ] {
                        let id = if children.len() == 1 {
                            alloc::format!("adoption_employer_{}_{}", suffix, employee)
                        } else {
                            alloc::format!(
                                "adoption_child_{}_employer_{}_{}",
                                child_index + 1,
                                suffix,
                                employee
                            )
                        };
                        line(
                            r,
                            &id,
                            amount,
                            &["credits.adoptions.benefits", "income.wage"],
                            "https://www.irs.gov/pub/irs-dft/f8839--dft.pdf",
                            msg!("EXPLAIN_ADOPTION_2"),
                        );
                    }
                    let taxable = gross_by_employee[index] - employee_excluded[index];
                    result.wages[index] += taxable;
                    result.w2_wages[index] += taxable;
                }
                let excluded = excluded.unwrap_or(0);
                if excluded > gross {
                    let excess = excluded - gross;
                    result.deemed_excess += excess;
                    if children.len() > 1 {
                        line(
                            r,
                            &alloc::format!(
                                "adoption_child_{}_special_needs_deemed_excess",
                                child_index + 1
                            ),
                            excess,
                            &["credits.adoptions.benefits", "income.wage"],
                            "https://www.irs.gov/pub/irs-dft/f8839--dft.pdf",
                            msg!("EXPLAIN_ADOPTION_CHILD_N_SPECIAL_NEEDS_DEEMED_EXCESS"),
                        );
                    }
                }
                result.excluded += excluded;
            } else {
                // Provisional MAGI includes all code T. An unsupported final split
                // cannot silently assign the available exclusion to an employee.
                result.wages[0] += gross_by_employee[0];
                result.wages[1] += gross_by_employee[1];
                result.source_class_complete = false;
            }
            continue;
        }
        let AdoptionBenefits::Program { owner, received, history, ordinary } = &child.benefits
        else {
            continue;
        };
        crate::require_localized!(
            r,
            *owner == Owner::Taxpayer || matches!(status, FederalStatus::Joint),
            &context
                .clone()
                .alternatives(&[crate::TaxSection::Setup, crate::TaxSection::MoreCredits]),
            "Adoption benefits",
            "ISSUE_CONFIRM_THE_QUALIFIED_PLAN_EMPLOYEE_OWNER_NONDUPLICATED_BENEFITS_AN_291CE261",
            "Your spouse's employer adoption benefits need a joint return. Multiple employees for one child need additional coordination.",
        );
        if !child.eligible || !*ordinary {
            if child.eligible {
                benefits_taxable(child, &context, r);
            } else {
                child_left_out(child, &context, r);
            }
            let employee = usize::from(*owner == Owner::Spouse);
            let taxable = dollars(*received);
            result.wages[employee] += taxable;
            program_taxable[employee] += taxable;
            program_requires_w2[employee] |= taxable != 0;
            continue;
        }
        let (finalized, domestic) = timing(&child.kind);
        let (prior, foreign_prior) = match history {
            AdoptionBenefitHistory::None => (0, 0),
            AdoptionBenefitHistory::Exclusion { used } => (*used, 0),
            AdoptionBenefitHistory::TaxedForeign { received } => {
                crate::require_localized!(
                    r,
                    !domestic && finalized.is_none_or(|year| year >= 2026),
                    &context,
                    "Adoption benefit history",
                    "ISSUE_EARLIER_TAXABLE_FOREIGN_BENEFITS_BELONG_TO_A_PENDING_FOREIGN_ADOPTI_1D3323FF",
                    "Earlier taxable foreign benefits belong to a pending foreign adoption or one finalized in 2026; earlier finalizations require the previously filed exclusion history.",
                );
                (0, if finalized == Some(2026) { *received } else { 0 })
            }
        };
        crate::require_localized!(
            r,
            domestic || prior == 0 || finalized.is_some_and(|year| year < 2026),
            &context,
            "Adoption benefits",
            "ISSUE_A_FOREIGN_ADOPTION_NOT_FINAL_BEFORE_RETURN_YEAR_CANNOT_HAVE_A_PRIOR_EXCLUS_1723F10D",
            "A foreign adoption not final before 2026 cannot have a prior exclusion-limit claim; prior taxable benefits need the foreign-adoption worksheet.",
        );
        let cap = positive(child.limits.amounts(&context, r)[1] - dollars(prior));
        let special = matches!(child.kind, AdoptionKind::Domestic { special_needs: true, .. })
            && finalized == Some(2026);
        let allowed = if !domestic && finalized.is_none() {
            0
        } else if special {
            cap
        } else {
            phase::cap(cap, [dollars(*received) + dollars(foreign_prior)])
        };
        let allowed = eligible_current_base(child, &context, setup, allowed, r);
        let excluded = magi.map_or(0, |m| {
            phase::linear_remaining(
                allowed,
                m,
                crate::year_constants::ADOPTION_PHASEOUT_START_DOLLARS,
                crate::year_constants::ADOPTION_PHASEOUT_RANGE_DOLLARS,
            )
        });
        let employee = usize::from(*owner == Owner::Spouse);
        let taxable = dollars(*received) - excluded;
        result.wages[employee] += taxable;
        if *received > 0 {
            program_taxable[employee] += taxable;
            program_requires_w2[employee] |= taxable != 0;
        } else {
            program_page_only |= taxable != 0;
        }
        result.excluded += excluded;
        result.prior_taxable += dollars(foreign_prior);
    }
    if program_page_only {
        result.source_class_complete = false;
    }
    w2_code_t(
        children,
        income,
        magi.is_none(),
        program_taxable,
        program_requires_w2,
        &mut result,
        r,
    );
    if result.deemed_excess > 0 {
        line(
            r,
            "adoption_joint_special_needs_deemed_excess",
            result.deemed_excess,
            &["credits.adoptions.benefits", "income.wage"],
            "https://www.irs.gov/pub/irs-dft/f8839--dft.pdf",
            msg!("EXPLAIN_ADOPTION_JOINT_SPECIAL_NEEDS_DEEMED_EXCESS"),
        );
    }
    result
}

// W-2 box 12 code T against the adoption records (2025 i8839 "Employer-Provided
// Adoption Benefits"). Each employee's code T is accounted for by their
// adoption records: a joint-employee payment names its W-2, an owner-level
// program amount draws on the employee's W-2s in order. A code T dollar no
// record accounts for is taxable wages (Form 1040 line 1f) with a warning to
// enter the adoption; a record above its W-2s' code T is refused. An employee
// whose W-2s show no code T keeps the records as entered (the amounts then
// come from the adoption page alone). Rows, notes and issues come with the
// first, provisional-income call so each appears once.
fn w2_code_t(
    children: &[AdoptionChild],
    income: &[crate::FederalIncomeEntry],
    report: bool,
    program_taxable: [i64; 2],
    program_requires_w2: [bool; 2],
    result: &mut Benefits,
    r: &mut FederalResult,
) {
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        let employee = usize::from(owner == Owner::Spouse);
        let w2s: Vec<(&crate::FederalIncomeEntry, i64)> = income
            .iter()
            .filter(|entry| entry.owner == crate::DocumentOwner::from(owner))
            .filter_map(|entry| match &entry.income {
                crate::FederalIncome::Wage { data } => {
                    let data = &crate::FederalWage::from(data);
                    let code_t = data
                        .other_box12
                        .iter()
                        .filter(|c| c.code == crate::W2Code::T)
                        .map(|c| c.amount)
                        .sum::<i64>();
                    (code_t > 0).then_some((entry, code_t))
                }
                _ => None,
            })
            .collect();
        if w2s.is_empty() {
            if program_requires_w2[employee] {
                result.source_class_complete = false;
            }
            continue;
        }
        let mut left: Vec<i64> = w2s.iter().map(|(_, amount)| *amount).collect();
        let mut program = 0;
        let mut above: Option<&crate::FederalIncomeEntry> = None;
        for child in children {
            match &child.benefits {
                AdoptionBenefits::JointEmployees { payments, .. } => {
                    for payment in payments.iter().filter(|p| p.owner == owner) {
                        if let Some(position) = w2s.iter().position(|(e, _)| e.id == payment.wage) {
                            if left[position] < payment.received {
                                above = Some(w2s[position].0);
                            }
                            left[position] = positive(crate::kernel::carry::native_balance(
                                left[position],
                                payment.received,
                            ));
                        }
                    }
                }
                AdoptionBenefits::Program { owner: program_owner, received, .. }
                    if *program_owner == owner =>
                {
                    program += received
                }
                _ => {}
            }
        }
        for amount in &mut left {
            let used = phase::cap(program, [*amount]);
            *amount = crate::kernel::carry::native_balance(*amount, used);
            program = crate::kernel::carry::native_balance(program, used);
        }
        if program > 0 || above.is_some() {
            result.source_class_complete = false;
        } else {
            result.w2_wages[employee] += program_taxable[employee];
        }
        if report && (program > 0 || above.is_some()) {
            let entry = above.unwrap_or(w2s[0].0);
            crate::require_localized!(
                r,
                false,
                &crate::interview::income_issue_context(entry)
                    .alternatives(&[crate::TaxSection::Wages, crate::TaxSection::MoreCredits]),
                "Adoption benefits",
                "ISSUE_ADOPTION_CODE_T_ABOVE_W_2",
                "The adoption benefits entered for this employee are more than the W-2 box 12 code T amounts: correct the W-2 or the adoption benefits.",
            );
        }
        for ((entry, _), amount) in w2s.iter().zip(left) {
            let taxable = dollars(amount);
            if taxable == 0 {
                continue;
            }
            result.wages[employee] += taxable;
            result.w2_wages[employee] += taxable;
            if report {
                line(
                    r,
                    &alloc::format!("adoption_w2_{}_code_t_taxable", entry.id),
                    taxable,
                    &["income.wage", "credits.adoptions.benefits"],
                    "https://www.irs.gov/instructions/i8839",
                    crate::Message::new(
                        "EXPLAIN_ADOPTION_W2_N_CODE_T_TAXABLE",
                        alloc::vec![crate::interview::income_display_arg(entry)],
                    ),
                );
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(
                        "WARN_W2_CODE_T_TAXABLE_NO_ADOPTION_RECORD",
                        alloc::vec![crate::interview::income_display_arg(entry)],
                    ),
                    crate::interview::income_issue_context(entry)
                        .alternatives(&[crate::TaxSection::Wages, crate::TaxSection::MoreCredits]),
                );
            }
        }
    }
}

fn child_context(index: usize, child: &AdoptionChild) -> crate::IssueContext {
    let section = crate::TaxSection::MoreCredits;
    crate::IssueContext::Record {
        section,
        identity: crate::IssueContext::Section(section).item("adoption-children", index).identity(),
        label: crate::Message::new(
            "TEXT",
            alloc::vec![crate::MessageArg::Text(child.label.clone())],
        ),
    }
}
// The current-year MFS exception is per child. Carryforwards use the original
// return and owned balance instead; do not gate them on current residence.
fn eligible_current_base(
    child: &AdoptionChild,
    context: &crate::IssueContext,
    setup: &FederalSetup,
    base: i64,
    r: &mut FederalResult,
) -> i64 {
    if base == 0 || setup.filing_status() != FederalStatus::Separate {
        return base;
    }
    if !setup.lived_apart_last_six_months()
        || !setup.paid_over_half_home
        || !child.home_over_half_year
    {
        crate::federal_rules::note_once_scoped(
            r,
            crate::Message::new(
                "NOTE_ADOPTION_SEPARATE_RETURN_EXCEPTION_NOT_MET",
                alloc::vec![crate::MessageArg::Text(child.label.clone())],
            ),
            context
                .clone()
                .alternatives(&[crate::TaxSection::Setup, crate::TaxSection::MoreCredits]),
        );
        return 0;
    }
    base
}

// A child whose eligibility is not confirmed gets no credit or exclusion.
fn child_left_out(child: &AdoptionChild, context: &crate::IssueContext, r: &mut FederalResult) {
    crate::federal_rules::note_once_scoped(
        r,
        crate::Message::new(
            "WARN_ADOPTION_CHILD_LEFT_OUT",
            alloc::vec![crate::MessageArg::Text(child.label.clone())],
        ),
        context.clone(),
    );
}

// Unconfirmed plan facts: the benefits stay taxable wages.
fn benefits_taxable(child: &AdoptionChild, context: &crate::IssueContext, r: &mut FederalResult) {
    crate::federal_rules::note_once_scoped(
        r,
        crate::Message::new(
            "WARN_ADOPTION_BENEFITS_LEFT_OUT",
            alloc::vec![crate::MessageArg::Text(child.label.clone())],
        ),
        context.clone(),
    );
}

impl AdoptionLimits {
    fn amounts(&self, context: &crate::IssueContext, r: &mut FederalResult) -> [i64; 2] {
        match self {
            Self::Full => [crate::year_constants::ADOPTION_CREDIT_MAXIMUM_DOLLARS; 2],
            Self::Shared { credit, exclusion, agreed } => {
                if !*agreed {
                    crate::federal_rules::note_once_scoped(
                        r,
                        msg!("WARN_ADOPTION_SHARED_LIMIT_LEFT_OUT"),
                        context.clone(),
                    );
                    return [0; 2];
                }
                [dollars(*credit), dollars(*exclusion)]
            }
        }
    }
}

/// Accepted adoption child answers
#[derive(Clone, Debug, PartialEq, Eq, LibertasAvroEncode, LibertasAvroDecode, LibertasExport)]
#[libertas_read_only]
pub struct AdoptionChildInitial {
    pub label: String,
    pub kind: AdoptionKind,
    pub limits: AdoptionLimits,
    pub expenses: Vec<AdoptionExpense>,
    pub prior: AdoptionPriorCredit,
    pub benefits: AdoptionBenefits,
    pub wages: Vec<crate::WageChoice>,
    pub home_over_half_year: bool,
    pub eligible: bool,
}

impl From<AdoptionChildPage> for AdoptionChildInitial {
    fn from(value: AdoptionChildPage) -> Self {
        Self {
            label: value.label,
            kind: value.kind,
            limits: value.limits,
            expenses: value.expenses,
            prior: value.prior,
            benefits: (&value.benefits).into(),
            wages: value.wages,
            home_over_half_year: value.home_over_half_year,
            eligible: value.eligible,
        }
    }
}
