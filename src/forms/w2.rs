// Coverage: Form W-2 — Supported (unit B10-2; audited 2026-10-12).
//   Boxes 1–8 and 10–14: ordinary wages/withholding, allocated/actual/noncash tips,
//   qualified tips without code TP, D/E/F/S/G/H/AA/BB/EE deferrals, W/T benefits,
//   A/B/M/N uncollected payroll tax, K section 4999 tax, Q combat pay, II Medicaid
//   waiver exclusions and TA employer contributions. Statutory box 13 routes gross
//   pay once to the same owner's expenses-only Schedule C (or a zero-expense C),
//   retains QBI, uses gross pay for earned income and excludes the profit from SE.
//   Box 14a RRTA compensation/tier 1/tier 2/Medicare/actual additional withholding
//   are separate source facts: Form 8959 Parts I–V, 25c, excess tier 1,
//   Schedule SE wage base and Schedule 8812 payroll calculation reuse those facts.
//   Railroad tips remain income without Form 4137 tax; employer reporting and
//   multiple-employer tier 2 Form 843 notices are explicit. Form CT-2 employee
//   representative compensation and single-employer
//   excess Social Security/tier 1 require their named unsupported corrections.
//   Clergy/minister exemptions reuse clergy.rs and the shared per-owner SE owner.
//   Sources: 2026 final iw2w3; 2026 draft Form 8959; 2025 i8959/i1040gi fallback;
//   Pub. 590-A difficulty-of-care and combat compensation; IRC 408(o)(5), 219(f)(7).
//   Evidence: src/tests/income/wage_specials.rs A–F and boundaries; interview wage_specials_*
//   save/import/Avro tests; supported() Form 8959 footing; existing business,
//   family, retirement, health-account and withholding oracles remain independent.

// Form W-2 income record: the per-employer arm of the federal income
// collection loop. Monetary source values are cents.
use crate::{FederalResult, FederalWage, federal_rules::Totals, kernel::ratio::fraction};

// Where one W-2 record accumulates while the federal income records are collected.
pub(crate) struct WageContext<'a> {
    pub(crate) totals: &'a mut Totals,
    pub(crate) result: &'a mut FederalResult,
    pub(crate) owner: usize,
    pub(crate) context: crate::IssueContext,
    #[cfg(test)]
    pub(crate) source: &'a str,
}

pub(crate) fn collect(ctx: &mut WageContext<'_>, w: &FederalWage) {
    let t = &mut *ctx.totals;
    let r = &mut *ctx.result;
    let owner = ctx.owner;
    #[cfg(test)]
    let source = ctx.source;
    // Box 1 pay that Schedule 1 reports on line 8t (a nonqualified plan's or
    // nongovernmental 457 plan's pension) or 8u (pay while an inmate) is not
    // Form 1040 line 1a wages, and so not earned income for the credit.
    if w.statutory_employee {
        t.statutory_gross[owner] += w.wages;
        if w.statutory_business.is_none() {
            t.statutory_profit[owner] += w.wages;
        }
    } else {
        t.wages[owner] += w.wages - w.deferred_pension - w.inmate_pay;
    }
    t.medicaid_in_box1[owner] += w.medicaid_waiver;
    for code in &w.other_box12 {
        match code.code {
            crate::W2Code::Q => t.combat_pay[owner] += code.amount,
            crate::W2Code::II => t.medicaid_excluded[owner] += code.amount,
            crate::W2Code::TA => crate::require_localized!(
                r,
                code.amount <= crate::year_constants::TRUMP_EMPLOYER_EXCLUSION_CENTS,
                &ctx.context,
                source,
                "W2_TA_ABOVE_LIMIT",
                "A W-2 code TA exclusion cannot exceed $2,500; ask the employer to correct it.",
            ),
            _ => {}
        }
    }
    let (social_wages, social_tax, medicare_wages, medicare_tax) = match w.payroll {
        crate::W2Payroll::SocialSecurity => {
            (w.social_wages + w.social_tips, w.social_tax, w.medicare_wages, w.medicare_tax)
        }
        crate::W2Payroll::Railroad {
            compensation,
            tier_1,
            tier_2,
            medicare,
            additional_medicare,
        } => {
            crate::require_localized!(
                r,
                w.social_wages == 0
                    && w.social_tips == 0
                    && w.social_tax == 0
                    && w.medicare_wages == 0
                    && w.medicare_tax == 0,
                &ctx.context,
                source,
                "W2_RAILROAD_WITH_SOCIAL_SECURITY_BOXES",
                "Railroad retirement compensation belongs in box 14a; boxes 3 through 7 must be blank.",
            );
            t.rrta_compensation[owner] += compensation;
            t.rrta_medicare[owner] += medicare;
            t.rrta_additional_withheld[owner] += additional_medicare;
            t.rrta_tier_2_forms[owner] += i64::from(tier_2 > 0);
            if w.employee_tips.is_some() {
                crate::federal_rules::note_once_scoped(
                    r,
                    crate::Message::new(
                        "NOTE_RAILROAD_TIPS_REPORT_TO_EMPLOYER",
                        alloc::vec::Vec::new(),
                    ),
                    ctx.context.clone(),
                );
            }
            (
                crate::kernel::phase::cap(
                    compensation,
                    [crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS],
                ),
                tier_1,
                compensation,
                medicare,
            )
        }
    };
    t.deferred_pension[owner] += w.deferred_pension;
    t.inmate_pay[owner] += w.inmate_pay;
    if w.coverage == crate::W2Coverage::ChurchEmployee {
        t.church_wages[owner] += w.wages;
    }
    crate::require_localized!(
        r,
        w.coverage == crate::W2Coverage::Ordinary || (w.social_wages == 0 && w.medicare_wages == 0),
        &ctx.context,
        source,
        "ISSUE_A_CHURCH_EMPLOYEE_W_2_MUST_SHOW_ZERO_SOCIAL_SECURITY_AND_MEDICARE_W_F52B07D4",
        "A church employee or ministerial W-2 must show zero Social Security and Medicare wages (boxes 3 and 5); box 1 is reported on Schedule SE line 5a for church employees or line 2 for ministerial services.",
    );
    if let Some(facts) = &w.employee_tips {
        match facts.calculate() {
            Ok(tips) => {
                t.wages[owner] += tips.income;
                t.employee_tips[owner].income += tips.income;
                t.employee_tips[owner].social += tips.social;
                t.employee_tips[owner].medicare += tips.medicare;
            }
            Err(_message) => {
                #[cfg(test)]
                let message = _message;
                crate::issue_localized!(
                    r,
                    &ctx.context,
                    "Employee tips",
                    "ISSUE_EMPLOYEE_TIPS_RECORDS",
                    message,
                )
            }
        }
    }
    t.disability[owner] += w.disability.amount();
    crate::require_localized!(
        r,
        w.disability.eligible(),
        &ctx.context,
        source,
        "ISSUE_CONFIRM_THE_DISABILITY_RETIREMENT_AGE_AND_MEDICAL_STATEMENT_REQUIRE_978C4752",
        "Confirm the disability-retirement, age and medical-statement requirements for these Schedule R wages.",
    );
    t.social_wages[owner] += social_wages;
    t.social_tax[owner] += social_tax;
    let uncollected_social = w.uncollected_tax.as_ref().map_or(0, |v| v.social());
    let uncollected_medicare = w.uncollected_tax.as_ref().map_or(0, |v| v.medicare());
    if w.uncollected_tax.is_some() {
        crate::require_localized!(
            r,
            uncollected_medicare <= fraction(medicare_wages, 145, 10000) + 1,
            &ctx.context,
            "Uncollected payroll tax",
            "ISSUE_RECONCILE_W_2_BOX_12_SOCIAL_SECURITY_ORDINARY_MEDICARE_AMOUNTS_WITH_6BFF7BFD",
            "Reconcile W-2 box 12 Social Security/ordinary Medicare amounts with the covered wage boxes; exclude railroad and Additional Medicare tax amounts.",
        );
    }
    t.uncollected_social[owner] += uncollected_social;
    t.uncollected_medicare += uncollected_medicare;
    t.uncollected_medicare_owners[owner] += uncollected_medicare;
    if matches!(w.payroll, crate::W2Payroll::SocialSecurity) {
        t.ordinary_uncollected_medicare += uncollected_medicare;
        t.ordinary_uncollected_medicare_owners[owner] += uncollected_medicare;
        t.medicare_wages[owner] += w.medicare_wages;
        t.medicare_tax[owner] += w.medicare_tax;
    }
    t.withholding_w2 += w.withholding;
    // 2026 W-2 instructions, code K: the printed amount is the tax itself.
    // Any withholding of it is already included in box 2; never add it twice.
    t.golden_parachute_tax[owner] += w
        .other_box12
        .iter()
        .filter(|entry| matches!(entry.code, crate::W2Code::K))
        .map(|entry| entry.amount)
        .sum::<i64>();
    {
        let v = &w.work_deductions;
        // W-2 code TP already substantiates the tips it reports; only Form 4137
        // tips beyond it need employee-tip records, as their unreported tips.
        let beyond_w2 = (v.form4137_tips() - v.w2_tips()).max(0);
        crate::require_localized!(
            r,
            beyond_w2 == 0
                || w.employee_tips.as_ref().is_some_and(|source| match &source.cash {
                    crate::CashTipRecords::Actual { months } => {
                        beyond_w2
                            <= months
                                .iter()
                                .map(|month| (month.received - month.reported).max(0))
                                .sum()
                    }
                    crate::CashTipRecords::Allocated { .. } => true,
                }),
            &ctx.context,
            source,
            "ISSUE_RECONCILE_THIS_EMPLOYER_S_QUALIFIED_FORM_4137_COLUMN_1_C_CASH_CHARG_8A2B870D",
            "Reconcile this employer's qualified Form 4137 column 1(c) cash/charged tips with its employee-tip records; Form 4137 cannot supply a deduction without the corresponding wage-income source.",
        );
        // No: these tips or this premium do not qualify, so no deduction.
        if v.tips_eligible() {
            t.tips[owner] += v.tips();
        }
        if v.overtime_eligible() {
            t.overtime[owner] += v.overtime();
        }
    }
    crate::require_localized!(
        r,
        social_wages <= crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS
            && social_tax + uncollected_social
                <= crate::year_constants::MAX_EMPLOYEE_SOCIAL_SECURITY_TAX_CENTS
            && social_tax + uncollected_social <= fraction(social_wages, 62, 1000) + 1,
        &ctx.context,
        source,
        "ISSUE_EXCESS_SOCIAL_SECURITY_WITHHOLDING_BY_ONE_EMPLOYER_REQUIRES_EMPLOYE_35C66F28",
        "Excess Social Security withholding by one employer requires employer correction, not the multiple-employer credit.",
    );
    crate::require_localized!(
        r,
        medicare_tax + uncollected_medicare + 1 >= fraction(medicare_wages, 145, 10000),
        &ctx.context,
        source,
        "ISSUE_MEDICARE_WITHHOLDING_BELOW_ORDINARY_PAYROLL_TAX_REQUIRES_SPECIAL_PA_3993480B",
        "Medicare withholding below ordinary payroll tax requires special payroll review.",
    );
}
