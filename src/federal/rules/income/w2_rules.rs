// w2 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// The 2026 Form W-2 instructions identify C/J/V/Y/DD/GG/HH as amounts
// already reflected in the wage boxes or informational amounts. Retain them
// without adding wages, contributions or tax. Other supported codes feed
// their existing calculation owners; H and Z still require unsupported rules.
// https://www.irs.gov/pub/irs-prior/iw2w3--2026.pdf, pages 20–23.
pub(super) fn w2_box12_refusal(code: W2Code) -> Option<(&'static str, &'static str)> {
    Some(match code {
        W2Code::H => (
            "ISSUE_W2_BOX12_H_UNSUPPORTED",
            "W-2 box 12 code H (section 501(c)(18)(D) plan contributions, Schedule 1 line 24f) is not yet connected from this page.",
        ),
        W2Code::Z => (
            "ISSUE_W2_BOX12_Z_UNSUPPORTED",
            "W-2 box 12 code Z (section 409A deferred compensation income, additional tax) is not yet supported.",
        ),
        // A/B/M/N/TP/TT have their own stored fields (never in other_box12); P's effect
        // is calculated (Form 3903 line 4).
        W2Code::A
        | W2Code::B
        | W2Code::M
        | W2Code::N
        | W2Code::K
        | W2Code::P
        | W2Code::T
        | W2Code::TP
        | W2Code::TT
        | W2Code::D
        | W2Code::E
        | W2Code::F
        | W2Code::G
        | W2Code::S
        | W2Code::L
        | W2Code::W
        | W2Code::R
        | W2Code::AA
        | W2Code::BB
        | W2Code::EE
        | W2Code::FF
        | W2Code::Q
        | W2Code::II
        | W2Code::C
        | W2Code::J
        | W2Code::V
        | W2Code::Y
        | W2Code::DD
        | W2Code::GG
        | W2Code::HH
        | W2Code::TA => return None,
    })
}

// One cleanup boundary for required-source or actual native-output failure.
// Preserve separately known source metadata, including payment withholding.
pub(super) fn payroll_source_issue(key: &str) -> bool {
    matches!(
        key,
        "ISSUE_EMPLOYEE_PAYROLL_COVERAGE_UNAVAILABLE"
            | "ISSUE_EMPLOYEE_RAILROAD_ORDINARY_TAX"
            | "ISSUE_EMPLOYEE_RRTA_COMPENSATION_UNAVAILABLE"
            | "ISSUE_EMPLOYEE_FOREIGN_GOVERNMENT_PAYROLL"
    )
}

pub(super) fn unsettled_payroll_agi(mut result: FederalResult) -> FederalResult {
    if let Some(known) = &mut result.payroll_known_rows {
        known.agi = false;
        withhold_required_source_result(&mut result);
    }
    result
}

pub(super) fn known_payroll_row(known: crate::federal_model::PayrollKnownRows, id: &str) -> bool {
    if id == "agi" {
        return known.agi;
    }
    for owner in 0..2 {
        if let Some(part) = id.strip_prefix(&alloc::format!("employee_wages_{owner}_")) {
            return known.form8919[owner]
                && match part {
                    "pay" | "wage_base" | "medicare_tax" => true,
                    "used_base" | "remaining_base" | "social_base" | "social_tax" | "tax" => {
                        known.social[owner]
                    }
                    _ => false,
                };
        }
        if let Some(part) = id.strip_prefix(&alloc::format!("employee_coverage_{owner}_")) {
            return match part {
                "social_tax" => known.coverage_parts[owner][0],
                "medicare_tax" => known.coverage_parts[owner][1],
                "tax" => known.coverage_parts[owner].iter().all(|known| *known),
                _ => false,
            };
        }
    }
    match id {
        "employee_tip_income"
        | "employee_wages_not_covered"
        | "additional_medicare_w2_wages"
        | "additional_medicare_tip_wages"
        | "additional_medicare_employee_wages"
        | "additional_medicare_other_employee_wages"
        | "additional_medicare_wages_threshold"
        | "additional_medicare_rrta_threshold"
        | "additional_medicare_se_threshold"
        | "additional_medicare_ordinary_withheld"
        | "additional_medicare_withholding_wages"
        | "additional_medicare_regular_withholding"
        | "additional_medicare_ordinary_additional_withheld" => true,
        "additional_medicare_wages_total"
        | "additional_medicare_wages_excess"
        | "additional_medicare_se_wage_offset"
        | "additional_medicare_se_remaining_threshold" => known.medicare_wages,
        "additional_medicare_payroll_wages" => known.additional_parts[0],
        "additional_medicare_rrta_compensation" | "additional_medicare_rrta_excess" => {
            known.railroad_compensation
        }
        "additional_medicare_rrta_tax" => known.additional_parts[1],
        "additional_medicare_wages_and_rrta" | "additional_medicare_wages" => {
            known.additional_parts[..2].iter().all(|value| *value)
        }
        "additional_medicare_se_income" => known.se_net,
        "additional_medicare_se_excess" => known.se_net && known.medicare_wages,
        "additional_medicare_se_tax" => known.additional_parts[2],
        "additional_medicare_rrta_withheld" => known.railroad_withholding,
        "additional_medicare_withheld" => known.railroad_withholding,
        "additional_medicare_tax" => known.additional_parts.iter().all(|value| *value),
        _ => false,
    }
}

pub(crate) fn withhold_required_source_result(result: &mut FederalResult) {
    // The policy exists only after a completed payroll pass. Any other required
    // failure still owns the original full cleanup, including on the outer pass.
    let payroll = result.payroll_known_rows.filter(|_| {
        !result.issues.is_empty()
            && result.issues.iter().all(|issue| payroll_source_issue(&issue.display.key))
    });
    result.state = if payroll.is_some()
        && result
            .issues
            .iter()
            .all(|issue| issue.display.key == "ISSUE_EMPLOYEE_PAYROLL_COVERAGE_UNAVAILABLE")
    {
        ResultState::Incomplete
    } else {
        ResultState::Unsupported
    };
    result.lines.retain(|line| {
        line.id == FORM_8615_UNEARNED
            || payroll.is_some_and(|known| known_payroll_row(known, &line.id))
    });
    // The final payment pass emits this row only after establishing complete
    // annual sources. Preserve that certified amount across outer cleanup;
    // any additional required-source failure invalidates this exception.
    let keep_estimates = estimated_timing_only(result);
    result.return_lines.retain(|row| {
        keep_estimates
            && row
                .form_line
                .as_ref()
                .is_some_and(|line| line.form == TaxForm::Form1040 && line.line == "26")
    });
    result.tax = None;
    result.farmland_installments = None;
    result.refundable_credits = None;
    result.payments = None;
    result.credit_capacity = None;
    result.refund = None;
    result.balance = None;
}

pub(super) struct EmployeePayrollOut {
    pub(super) tip_income: i64,
    pub(super) w2_wages: i64,
    pub(super) uncollected_payroll: i64,
    pub(super) golden_parachute_tax: i64,
    pub(super) tip_payroll: i64,
    pub(super) tip_payroll_owners: [i64; 2],
    pub(super) employee_wage_income: i64,
    pub(super) employee_wage_tax: i64,
    pub(super) employee_wage_taxes: [i64; 2],
    pub(super) form_8919_tax: i64,
}

pub(super) fn employee_payroll_phase(r: &mut FederalResult, t: &mut Totals) -> EmployeePayrollOut {
    let tip_income = dollars(t.employee_tips.iter().map(|v| v.income).sum());
    let w2_wages = dollars(
        t.wages.iter().sum::<i64>()
            - t.employee_tips.iter().map(|v| v.income).sum::<i64>()
            - t.employee_wages.iter().sum::<i64>()
            - t.employee_wages_not_covered.iter().sum::<i64>(),
    );
    let uncollected_payroll =
        dollars(t.uncollected_social.iter().sum::<i64>() + t.uncollected_medicare);
    form_line(
        r,
        FormLine::new(TaxForm::FormW2, "12"),
        "w2_uncollected_payroll_tax",
        uncollected_payroll,
        &["income.uncollected_tax"],
        "https://www.irs.gov/instructions/i1040gi",
        msg!("EXPLAIN_W2_UNCOLLECTED_PAYROLL_TAX"),
    );
    let golden_parachute_tax = dollars(t.golden_parachute_tax.iter().sum());
    let mut tip_payroll = 0;
    let mut tip_payroll_owners = [0; 2];
    for (owner, owner_tax) in tip_payroll_owners.iter_mut().enumerate() {
        let taxes = crate::employee_tips::payroll(t.social_wages[owner], &t.employee_tips[owner]);
        *owner_tax = taxes.iter().sum::<i64>();
        tip_payroll += *owner_tax;
        t.social_wages[owner] += t.employee_tips[owner].social;
        emit_lines(
            r,
            &mut (),
            [("social", taxes[0]), ("medicare", taxes[1])],
            |(suffix, amount), _, _| {
                Some(calculation_row(
                    None,
                    &alloc::format!("employee_tips_{owner}_{suffix}_tax"),
                    LineValue::MoneyDollars(amount),
                    &["income.employee_tips", "income.social_wages", "income.social_tips"],
                    "https://www.irs.gov/pub/irs-pdf/f4137.pdf",
                    msg!("EXPLAIN_EMPLOYEE_TIPS_N_N_TAX"),
                ))
            },
        );
    }
    line(
        r,
        "employee_tip_income",
        tip_income,
        &["income.employee_tips"],
        "https://www.irs.gov/pub/irs-pdf/f4137.pdf",
        msg!("EXPLAIN_EMPLOYEE_TIP_INCOME"),
    );
    let employee_wage_income = dollars(t.employee_wages.iter().sum());
    let mut employee_wage_tax = 0;
    let mut employee_wage_taxes = [0; 2];
    for (owner, owner_tax) in employee_wage_taxes.iter_mut().enumerate() {
        let amount = t.employee_8919[owner];
        let taxes = crate::employee_tips::payroll(
            t.social_wages[owner],
            &crate::employee_tips::TipAmounts { income: amount, social: amount, medicare: amount },
        );
        // Form 8919 lines 7–10 use the same physical employee's payroll base.
        // Line 8 already includes W-2/RRTA wages and Form 4137 covered tips.
        let wage_base = crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_DOLLARS;
        let used_base = dollars(t.social_wages[owner]);
        let remaining_base = phase::floor(wage_base - used_base, 0);
        let social_base = phase::cap(dollars(amount), [remaining_base]);
        let person_tax = taxes.iter().sum::<i64>();
        employee_wage_tax += person_tax;
        *owner_tax = person_tax;
        t.social_wages[owner] += amount;
        // One printed Form 8919 per person with pay; the lines stay plain
        // calculation lines for a person without any (zero).
        let printed = amount != 0;
        for (reference, id, value, inputs, explanation) in [
            (
                "6",
                alloc::format!("employee_wages_{owner}_pay"),
                dollars(amount),
                &["income.employee_wages"][..],
                msg!("EXPLAIN_EMPLOYEE_WAGES_N_PAY"),
            ),
            (
                "7",
                alloc::format!("employee_wages_{owner}_wage_base"),
                wage_base,
                &[][..],
                msg!("EXPLAIN_EMPLOYEE_WAGES_WAGE_BASE"),
            ),
            (
                "8",
                alloc::format!("employee_wages_{owner}_used_base"),
                used_base,
                &["income.social_wages", "income.social_tips", "income.employee_tips"][..],
                msg!("EXPLAIN_EMPLOYEE_WAGES_USED_BASE"),
            ),
            (
                "9",
                alloc::format!("employee_wages_{owner}_remaining_base"),
                remaining_base,
                &["income.social_wages", "income.social_tips", "income.employee_tips"][..],
                msg!("EXPLAIN_EMPLOYEE_WAGES_REMAINING_BASE"),
            ),
            (
                "10",
                alloc::format!("employee_wages_{owner}_social_base"),
                social_base,
                &["income.employee_wages", "income.social_wages"][..],
                msg!("EXPLAIN_EMPLOYEE_WAGES_N_SOCIAL_BASE"),
            ),
            (
                "11",
                alloc::format!("employee_wages_{owner}_social_tax"),
                taxes[0],
                &["income.employee_wages", "income.social_wages", "income.employee_tips"][..],
                msg!("EXPLAIN_EMPLOYEE_WAGES_N_N_TAX"),
            ),
            (
                "12",
                alloc::format!("employee_wages_{owner}_medicare_tax"),
                taxes[1],
                &["income.employee_wages", "income.social_wages", "income.employee_tips"][..],
                msg!("EXPLAIN_EMPLOYEE_WAGES_N_N_TAX"),
            ),
            (
                "13",
                alloc::format!("employee_wages_{owner}_tax"),
                taxes.iter().sum::<i64>(),
                &["income.employee_wages", "income.social_wages"][..],
                msg!("EXPLAIN_EMPLOYEE_WAGES_N_TAX"),
            ),
        ] {
            if printed {
                form_line(
                    r,
                    FormLine::new(TaxForm::Form8919, reference),
                    &id,
                    value,
                    inputs,
                    "https://www.irs.gov/pub/irs-pdf/f8919.pdf",
                    explanation,
                );
            } else if reference == "11" || reference == "12" {
                line(
                    r,
                    &id,
                    value,
                    inputs,
                    "https://www.irs.gov/pub/irs-pdf/f8919.pdf",
                    explanation,
                );
            }
        }
    }
    // Actual Form8919 pay goes first: its line8 contains reported W2/RRTA
    // and4137 wages only. The separate state/local worksheet then consumes
    // the remaining SS wage base; it never fabricates an8919 line8 operand.
    let form_8919_tax = employee_wage_tax;
    EmployeePayrollOut {
        tip_income,
        w2_wages,
        uncollected_payroll,
        golden_parachute_tax,
        tip_payroll,
        tip_payroll_owners,
        employee_wage_income,
        employee_wage_tax,
        employee_wage_taxes,
        form_8919_tax,
    }
}

pub(super) struct WageAdjustmentsOut {
    pub(super) moving: crate::moving::Moving,
    pub(super) iso_unreported: i64,
    pub(super) iso_amt_wage_adjustment: i64,
    pub(super) moving_excess: i64,
    pub(super) moving_deduction: i64,
    pub(super) disability_pension: i64,
    pub(super) deferral_excess: i64,
}

pub(super) fn wage_adjustments_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &mut Totals,
) -> WageAdjustmentsOut {
    let &Pass { d, p, j, .. } = pass;
    let iso_unreported = dollars(t.iso_unreported.iter().sum());
    let iso_amt_wage_adjustment = dollars(t.iso_amt_wage_adjustment.iter().sum());
    line(
        r,
        "iso_unreported_compensation",
        iso_unreported,
        &["income.iso_sale.in_wages", "income.espp_sale.in_wages"],
        "https://www.irs.gov/publications/p525",
        msg!("EXPLAIN_ISO_UNREPORTED_COMPENSATION"),
    );
    line(
        r,
        "amt_iso_compensation_adjustment",
        iso_amt_wage_adjustment,
        &["income.iso_sale"],
        "https://www.irs.gov/instructions/i6251",
        msg!("EXPLAIN_AMT_ISO_COMPENSATION_ADJUSTMENT"),
    );
    let moving = crate::moving::figure(d, r, j);
    t.moving_excess = moving.excess;
    let moving_excess = moving.excess_dollars();
    let moving_deduction = moving.deduction_dollars();
    let disability_pension = dollars(t.disability_pension.iter().sum());
    let deferrals = [
        crate::elective_deferrals::excess(d, Owner::Taxpayer, p.taxpayer.birth_date),
        p.spouse.as_ref().filter(|_| j).map_or_else(Default::default, |spouse| {
            crate::elective_deferrals::excess(d, Owner::Spouse, spouse.birth_date)
        }),
    ];
    t.deferral_excess = [deferrals[0].taxable, deferrals[1].taxable];
    let deferral_excess = dollars(t.deferral_excess.iter().sum());
    if deferral_excess > 0 {
        note_once(r, msg!("NOTE_EXCESS_ELECTIVE_DEFERRALS"));
        if deferrals.iter().any(|e| e.taxable > 0 && e.several) {
            note_once(r, msg!("NOTE_EXCESS_ELECTIVE_DEFERRALS_SEVERAL_W2S"));
        }
    }
    WageAdjustmentsOut {
        moving,
        iso_unreported,
        iso_amt_wage_adjustment,
        moving_excess,
        moving_deduction,
        disability_pension,
        deferral_excess,
    }
}

pub(super) struct OtherWageIncomeIn {
    pub(super) w2_wages: i64,
    pub(super) tip_income: i64,
    pub(super) employee_wage_income: i64,
    pub(super) disability_pension: i64,
    pub(super) moving_excess: i64,
    pub(super) deferral_excess: i64,
}

pub(super) struct OtherWageIncomeOut {
    pub(super) household_wages: i64,
    pub(super) strike_benefits: i64,
    pub(super) employee_wages_not_covered: i64,
    pub(super) medicaid_waiver_wages: i64,
    pub(super) medicaid_waiver_excluded: i64,
    pub(super) clergy_housing_excess: [i64; 2],
    pub(super) clergy_excess: i64,
    pub(super) wages: i64,
}

pub(super) fn other_wage_income_phase(
    pass: &Pass<'_>,
    r: &mut FederalResult,
    t: &Totals,
    input: OtherWageIncomeIn,
) -> core::ops::ControlFlow<(), OtherWageIncomeOut> {
    let &Pass { d, selected_regular, office_context, context, .. } = pass;
    let source_claims = context.source_claims;
    let OtherWageIncomeIn {
        w2_wages,
        tip_income,
        employee_wage_income,
        disability_pension,
        moving_excess,
        deferral_excess,
    } = input;
    let household_wages = dollars(t.household_wages.iter().sum());
    let strike_benefits = dollars(t.strike_benefits.iter().sum());
    let employee_wages_not_covered = dollars(t.employee_wages_not_covered.iter().sum());
    let medicaid_waiver_wages =
        dollars(t.medicaid_taxable.iter().sum::<i64>() + t.medicaid_excluded.iter().sum::<i64>());
    let medicaid_waiver_excluded =
        dollars(t.medicaid_excluded.iter().sum::<i64>() + t.medicaid_in_box1.iter().sum::<i64>());
    // Form1040 aggregates both owners' cents before its line1h projection.
    let clergy_housing_excess = [
        match crate::clergy::calculate_with_section179_with_claims(
            d,
            Owner::Taxpayer,
            selected_regular,
            Some(office_context),
            source_claims,
        ) {
            Ok(value) => value,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        }
        .housing_excess,
        match crate::clergy::calculate_with_section179_with_claims(
            d,
            Owner::Spouse,
            selected_regular,
            Some(office_context),
            source_claims,
        ) {
            Ok(value) => value,
            Err(issue) => {
                crate::section179_return::Issue::from(issue).report(r);
                r.state = ResultState::Unsupported;
                return core::ops::ControlFlow::Break(());
            }
        }
        .housing_excess,
    ];
    let clergy_excess = dollars(clergy_housing_excess.iter().sum());
    let wages = clergy_excess
        + household_wages
        + strike_benefits
        + employee_wages_not_covered
        + medicaid_waiver_wages
        + w2_wages
        + tip_income
        + employee_wage_income
        + disability_pension
        + moving_excess
        + deferral_excess;
    for (id, amount, key) in [
        ("household_wages", household_wages, "EXPLAIN_1040_1B"),
        ("medicaid_waiver_wages", medicaid_waiver_wages, "EXPLAIN_1040_1D"),
        ("strike_benefits", strike_benefits, "EXPLAIN_STRIKE_BENEFITS"),
        (
            "employee_wages_not_covered",
            employee_wages_not_covered,
            "EXPLAIN_EMPLOYEE_WAGES_NOT_COVERED",
        ),
    ] {
        line(
            r,
            id,
            amount,
            &["income.wages", "income.other_income"],
            "https://www.irs.gov/instructions/i1040gi",
            Message::new(key, alloc::vec![]),
        );
    }
    core::ops::ControlFlow::Continue(OtherWageIncomeOut {
        household_wages,
        strike_benefits,
        employee_wages_not_covered,
        medicaid_waiver_wages,
        medicaid_waiver_excluded,
        clergy_housing_excess,
        clergy_excess,
        wages,
    })
}
