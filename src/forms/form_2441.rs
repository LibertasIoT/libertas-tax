// Coverage: Form 2441 (2026), Parts I–III and prior-year Worksheet A.
//   Provider facts, deemed income, per-W-2 benefit source answers and the
//   own-business deduction are entered on Child and dependent care. Existing
//   dependent classification owns qualifying dependent costs and identification.
//   Sources: IRS 2026 draft Form2441/instructions; Pub503 (2025 final).
use crate::kernel::lines::{LineValue, emit, row as calculation_row};
use crate::kernel::phase;
use crate::kernel::ratio::{dollars, fraction, positive};
use crate::{
    FederalCredits, FederalDraft, FederalResult, FederalSetup, FederalStatus, FormLine, TaxForm,
};
use crate::{federal_rules::LAW, kernel::lines::form_line};
const SOURCE: &str = "https://www.irs.gov/pub/irs-dft/i2441--dft.pdf";
// Actual native credit components, also retained when no form rows are needed.
pub(crate) struct CreditComponents {
    pub total: i64,
    pub current: i64,
    pub prior: i64,
    pub expenses_smallest: i64,
    pub rate: i64,
}

pub(crate) struct Form2441Context<'a> {
    pub(crate) draft: &'a FederalDraft,
    pub(crate) setup: &'a FederalSetup,
    pub(crate) credits: &'a FederalCredits,
    pub(crate) joint: bool,
    pub(crate) care_people: i64,
    pub(crate) care_expenses: i64,
    pub(crate) earned: [i64; 2],
    pub(crate) optional_eic: [i64; 2],
    pub(crate) deduction_owners: [i64; 2],
}
pub(crate) struct Form2441 {
    pub(crate) earned: [i64; 2],
    pub(crate) excluded: i64,
    pub(crate) deducted: i64,
    pub(crate) taxable: [i64; 2],
    // Source classification of this already settled annual-dollar amount.
    // A prior-year plan statement alone does not identify a current W-2.
    pub(crate) taxable_from_w2: [bool; 2],
}
// One native-cents owner for the actual W-2, own-plan and grace sources.
// No record-count limit stands in for checked aggregate arithmetic.
pub(crate) struct BenefitSources {
    line12: i64,
    forfeited: i64,
    own: i64,
    own_received: [i64; 2],
    grace: i64,
    grace_by_owner: [i64; 2],
    employee_from_w2: [i64; 2],
    employee: [i64; 2],
    employee_total: i64,
}
pub(crate) fn benefit_sources(
    draft: &FederalDraft,
    care: &crate::FederalCare,
) -> Option<BenefitSources> {
    use crate::kernel::ratio::checked_sum;
    let rows = draft
        .income
        .iter()
        .filter_map(|entry| {
            let crate::FederalIncome::Wage { data } = &entry.income else {
                return None;
            };
            if data.care_benefits <= 0 {
                return None;
            }
            let answer = care.benefits.iter().find(|answer| answer.wage == entry.id);
            Some(
                data.care_benefits
                    .checked_sub(answer.and_then(|answer| answer.in_box_1).unwrap_or(0))
                    .map(|net| {
                        (entry.owner, net, answer.and_then(|answer| answer.forfeited).unwrap_or(0))
                    }),
            )
        })
        .collect::<Option<alloc::vec::Vec<_>>>()?;
    let own_received = [crate::Owner::Taxpayer, crate::Owner::Spouse].map(|owner| {
        checked_sum(
            care.own_business
                .iter()
                .filter(|source| source.owner == owner)
                .map(|source| source.received),
        )
    });
    let own_received = [own_received[0]?, own_received[1]?];
    let own = checked_sum(own_received)?;
    let line12 = checked_sum(rows.iter().map(|(_, net, _)| *net).chain(core::iter::once(own)))?;
    let forfeited = checked_sum(rows.iter().map(|(_, _, unused)| *unused))?;
    let grace_by_owner = [crate::Owner::Taxpayer, crate::Owner::Spouse].map(|owner| {
        checked_sum(
            care.grace
                .iter()
                .filter(|source| source.owner == owner)
                .map(|source| source.used_in_return_year),
        )
    });
    let grace_by_owner = [grace_by_owner[0]?, grace_by_owner[1]?];
    let grace = checked_sum(grace_by_owner)?;
    let employee_from_w2 = [crate::Owner::Taxpayer, crate::Owner::Spouse].map(|owner| {
        let amounts = rows
            .iter()
            .filter(|(source_owner, _, _)| *source_owner == owner)
            .map(|(_, net, unused)| net.checked_sub(*unused))
            .collect::<Option<alloc::vec::Vec<_>>>()?;
        checked_sum(amounts)
    });
    let employee_from_w2 = [employee_from_w2[0]?, employee_from_w2[1]?];
    let employee = [
        employee_from_w2[0].checked_add(grace_by_owner[0])?,
        employee_from_w2[1].checked_add(grace_by_owner[1])?,
    ];
    let employee_total = checked_sum(employee)?;
    Some(BenefitSources {
        line12,
        forfeited,
        own,
        own_received,
        grace,
        grace_by_owner,
        employee_from_w2,
        employee,
        employee_total,
    })
}

pub(crate) fn compute(ctx: &Form2441Context<'_>, r: &mut FederalResult) -> Option<Form2441> {
    let c = &ctx.credits.care;
    let sources = benefit_sources(ctx.draft, c)?;
    let people = ctx.care_people + c.filer_people();
    let actual = core::array::from_fn(|i| ctx.earned[i] + ctx.optional_eic[i]);
    let mut earned = c.earned(actual, people, r);
    // Lines18/19 exclude the own-plan benefits on line12. Restore this
    // pass's deduction before removing the entire received benefit, so its
    // taxable portion cannot inflate the exclusion's earned-income limit.
    let own_received = sources.own_received;
    let benefit_actual =
        core::array::from_fn(|i| actual[i] + ctx.deduction_owners[i] - dollars(own_received[i]));
    let benefit_earned = c.earned(benefit_actual, people, r);
    let omitted = dollars(c.left_out(ctx.draft, r)?);
    if c.providers.is_empty() && c.has_benefits(ctx.draft) && people == 0 {
        crate::federal_rules::note_once_scoped(
            r,
            msg!("NOTE_CARE_NO_PROVIDER"),
            crate::IssueContext::Section(crate::TaxSection::Care),
        );
    }
    let costs = positive(
        dollars(ctx.care_expenses + c.filer_expenses() + c.unpaid_return_year_care.unwrap_or(0))
            - omitted,
    );
    let costs = if people > 0 { costs } else { 0 };
    for e in &ctx.draft.income {
        if matches!(&e.income, crate::FederalIncome::Wage { data } if data.care_benefits > 0)
            && !c.benefits.iter().any(|answer| answer.wage == e.id)
        {
            crate::federal_rules::note_once_scoped(
                r,
                msg!("WARN_CARE_PLAN_NOT_ENTERED"),
                crate::IssueContext::Section(crate::TaxSection::Care),
            );
        }
    }
    let line12 = dollars(sources.line12);
    let forfeited = dollars(sources.forfeited);
    let own = sources.own;
    let employee = sources.employee;
    let employee_from_w2 = sources.employee_from_w2;
    let grace_by_owner = sources.grace_by_owner;
    let line13 = dollars(sources.grace);
    let line15 = positive(crate::kernel::carry::native_balance(line12 + line13, forfeited));
    let line17 = phase::cap(line15, [costs]);
    let line18 = positive(benefit_earned[0]);
    let line19 = if ctx.joint {
        positive(benefit_earned[1])
    } else if ctx.setup.filing_status() == FederalStatus::Separate
        && !crate::care::considered_unmarried(ctx.setup, ctx.care_people)
    {
        match c.filing {
            crate::CareFilingStatus::Separate { spouse_earned_income: Some(v) } => dollars(v),
            _ => {
                if c.has_benefits(ctx.draft) {
                    crate::federal_rules::note_once(
                        r,
                        msg!("WARN_CARE_SPOUSE_EARNINGS_ASSUMED_ZERO"),
                    );
                }
                0
            }
        }
    } else {
        line18
    };
    let line20 = phase::cap(line17, [line18, line19]);
    let statutory = if ctx.setup.filing_status() == FederalStatus::Separate
        && !crate::care::considered_unmarried(ctx.setup, ctx.care_people)
    {
        crate::year_constants::DEPENDENT_CARE_BENEFIT_LIMIT_SEPARATE_DOLLARS
    } else {
        crate::year_constants::DEPENDENT_CARE_BENEFIT_LIMIT_DOLLARS
    };
    let line21 = match c.plan_maximum {
        crate::CarePlanMaximum::NoBenefits => 0,
        crate::CarePlanMaximum::Plans { maximum } => phase::cap(statutory, [dollars(maximum)]),
    };
    let line22 = dollars(own);
    let line23 =
        crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(line15, line22)
            .get();
    let deducted = phase::cap(line20, [line21, line22]);
    let excluded = crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
        phase::cap(line20, [line21]),
        deducted,
    )
    .get();
    let taxable_total =
        crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(line23, excluded)
            .get();
    let sum = sources.employee_total;
    let taxable = if sum > 0 {
        let shares = crate::kernel::ratio::allocate_prefix(taxable_total, &employee);
        [shares[0], shares[1]]
    } else {
        [0, taxable_total]
    };
    if ctx.claims_care(ctx.credits) {
        emit(
            r,
            &mut (),
            [
                ("12", "care_form2441_line12", line12),
                ("13", "care_form2441_line13", line13),
                ("14", "care_form2441_line14", forfeited),
                ("15", "care_form2441_line15", line15),
                ("16", "care_form2441_line16", costs),
                ("17", "care_form2441_line17", line17),
                ("18", "care_form2441_line18", line18),
                ("19", "care_form2441_line19", line19),
                ("20", "care_form2441_line20", line20),
                ("21", "care_form2441_line21", line21),
                ("22", "care_form2441_line22", line22),
                ("23", "care_form2441_line23", line23),
                ("24", "care_form2441_line24", deducted),
                ("25", "care_benefits_excluded", excluded),
                ("26", "care_benefits_taxable", taxable_total),
            ],
            |(line, id, amount), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form2441, line)),
                    id,
                    LineValue::MoneyDollars(amount),
                    &["credits.care"],
                    SOURCE,
                    msg!("EXPLAIN_FORM_2441"),
                ))
            },
        );
        emit(
            r,
            &mut (),
            [("care_taxable_taxpayer", taxable[0]), ("care_taxable_spouse", taxable[1])],
            |(id, amount), _, _| {
                Some(calculation_row(
                    None,
                    id,
                    LineValue::MoneyDollars(amount),
                    &["credits.care", "income.wage.care_benefits"],
                    SOURCE,
                    msg!("EXPLAIN_CARE_BENEFITS_TAXABLE"),
                ))
            },
        );
        emit(r, &mut (), c.providers.iter().enumerate(), |(i, p), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form2441, "1e")),
                &alloc::format!("care_provider_{i}_paid"),
                LineValue::MoneyDollars(dollars(p.paid)),
                &["credits.care.providers"],
                SOURCE,
                msg!("EXPLAIN_FORM_2441"),
            ))
        });
        emit(r, &mut (), c.filers.iter().enumerate(), |(i, f), _, _| {
            Some(calculation_row(
                Some(FormLine::new(TaxForm::Form2441, "B")),
                &alloc::format!("care_filer_{i}_deemed"),
                LineValue::MoneyDollars(positive(
                    earned[f.owner as usize] - actual[f.owner as usize],
                )),
                &["credits.care.filers"],
                SOURCE,
                msg!("EXPLAIN_FORM_2441"),
            ))
        });
    }
    for i in 0..2 {
        earned[i] += taxable[i];
    }
    if ctx.claims_care(ctx.credits) {
        emit(
            r,
            &mut (),
            [
                ("4", "care_earned_taxpayer", positive(earned[0])),
                (
                    "5",
                    "care_earned_spouse",
                    positive(if ctx.joint { earned[1] } else { earned[0] }),
                ),
            ],
            |(line, id, amount), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form2441, line)),
                    id,
                    LineValue::MoneyDollars(amount),
                    &["credits.care"],
                    SOURCE,
                    msg!("EXPLAIN_FORM_2441"),
                ))
            },
        );
    }
    Some(Form2441 {
        earned,
        excluded,
        deducted,
        taxable,
        taxable_from_w2: core::array::from_fn(|i| {
            taxable[i] == 0 || (employee_from_w2[i] > 0 && grace_by_owner[i] == 0)
        }),
    })
}
impl Form2441 {
    pub(crate) fn credit(
        &self,
        ctx: &Form2441Context<'_>,
        agi: i64,
        r: &mut FederalResult,
    ) -> CreditComponents {
        let c = &ctx.credits.care;
        let people = ctx.care_people + c.filer_people();
        let ceiling = if people >= 2 {
            crate::year_constants::CARE_EXPENSE_LIMIT_TWO_PERSONS_DOLLARS
        } else if people == 1 {
            crate::year_constants::CARE_EXPENSE_LIMIT_ONE_PERSON_DOLLARS
        } else {
            0
        };
        // PartIII removes benefits from both the actual expenses and annual ceiling.
        let benefits = self.excluded + self.deducted;
        let left_out = dollars(
            c.providers
                .iter()
                .filter(|p| {
                    p.relationship != crate::ProviderRelationship::NotRelated
                        || p.identification == crate::ProviderIdentification::NotObtained
                })
                .map(|p| p.paid)
                .sum(),
        );
        let costs = positive(dollars(ctx.care_expenses + c.filer_expenses()) - left_out);
        let line30 = crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            costs, benefits,
        )
        .get();
        let line29 = crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(
            ceiling, benefits,
        )
        .get();
        let line3 = phase::cap(line29, [line30]);
        let line6 = phase::cap(
            line3,
            [positive(self.earned[0]), if ctx.joint { positive(self.earned[1]) } else { i64::MAX }],
        );
        let rate = care_rate(agi, ctx.joint);
        let blocked = ctx.setup.filing_status() == FederalStatus::Separate
            && !crate::care::considered_unmarried(ctx.setup, ctx.care_people);
        let prior = c.prior_year.as_ref().map_or(0, |p| p.credit(r));
        let current = if blocked { 0 } else { fraction(line6, rate, 100) };
        let total = current + prior;
        if ctx.claims_care(ctx.credits) {
            emit(
                r,
                &mut (),
                [
                    ("27", "care_form2441_line27", ceiling),
                    ("28", "care_form2441_line28", benefits),
                    ("29", "care_form2441_line29", line29),
                    ("30", "care_form2441_line30", line30),
                    ("31", "care_form2441_line31", line3),
                    ("3", "care_expenses_allowed", line3),
                    ("6", "care_expenses_smallest", line6),
                    ("7", "care_agi", agi),
                    ("9a", "care_credit_before_prior", current),
                    ("9b", "prior_year_care_credit", prior),
                    ("9c", "care_credit_total", total),
                ],
                |(line, id, amount), _, _| {
                    Some(calculation_row(
                        Some(FormLine::new(TaxForm::Form2441, line)),
                        id,
                        LineValue::MoneyDollars(amount),
                        &["credits.care"],
                        SOURCE,
                        msg!("EXPLAIN_FORM_2441"),
                    ))
                },
            );
            crate::kernel::lines::form_value(
                r,
                FormLine::new(TaxForm::Form2441, "8"),
                "care_percent",
                LineValue::PercentageHundredths(rate * 100),
                &["agi"],
                SOURCE,
                msg!("EXPLAIN_CARE_PERCENT"),
            );
        }
        CreditComponents { total, current, prior, expenses_smallest: line6, rate }
    }
}
impl Form2441Context<'_> {
    pub(crate) fn claims_care(&self, c: &FederalCredits) -> bool {
        self.care_expenses + c.care.filer_expenses() > 0
            || c.care.active()
            || c.care.has_benefits(self.draft)
    }
}
// Line 10, the Credit Limit Worksheet amount: Form 1040 line 18 less the
// Schedule 3 line 1 foreign tax credit.
pub(crate) fn emit_line_10(limit: i64, r: &mut FederalResult) {
    form_line(
        r,
        FormLine::new(TaxForm::Form2441, "10"),
        "care_credit_limit",
        limit,
        &["agi", "income_tax", "credits"],
        SOURCE,
        msg!("EXPLAIN_CARE_CREDIT_LIMIT"),
    );
}

// Line 11 after the line 10 limit.
pub(crate) fn emit_line_11(credit: i64, r: &mut FederalResult) {
    form_line(
        r,
        FormLine::new(TaxForm::Form2441, "11"),
        "care_credit",
        credit,
        &["agi", "income_tax", "credits", "income", "payments"],
        LAW,
        msg!("EXPLAIN_CARE_CREDIT"),
    );
}

// Line 8.
pub(crate) fn care_rate(agi: i64, joint: bool) -> i64 {
    phase::floor(
        20,
        phase::floor(35, 50 - phase::steps(agi - 15000, 2000, phase::StepRounding::Up))
            - phase::steps(
                agi - if joint { 150000 } else { 75000 },
                if joint { 4000 } else { 2000 },
                phase::StepRounding::Up,
            ),
    )
}
