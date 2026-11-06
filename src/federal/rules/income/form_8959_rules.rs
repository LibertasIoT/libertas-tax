// form_8959 items of src/federal/federal_rules.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;

// Form8959's existing three final tax parts. The actual invocation and source
// availability endpoints use the same owner, including the SE threshold offset.
pub(super) fn additional_medicare_parts(
    wages: i64,
    railroad: i64,
    net: i64,
    threshold: i64,
) -> [i64; 3] {
    let remaining =
        crate::kernel::carry::native_remaining::<crate::kernel::carry::Dollars>(threshold, wages)
            .get();
    [
        fraction(positive(wages - threshold), 9, 1000),
        fraction(phase::phase_out(railroad, threshold), 9, 1000),
        fraction(positive(net - remaining), 9, 1000),
    ]
}

pub(super) struct PayrollTaxIn<'a> {
    pub(super) tip_payroll: i64,
    pub(super) employee_wage_tax: i64,
    pub(super) uncollected_payroll: i64,
    pub(super) separate_niit: bool,
    pub(super) net: [i64; 2],
    pub(super) payroll_agi_known: bool,
    pub(super) payroll_se_net_known: bool,
    pub(super) se_church_wages: [i64; 2],
    pub(super) half: [i64; 2],
    pub(super) employee_coverage_possible: [[i64; 2]; 2],
    pub(super) employee_coverage_taxes: [[i64; 2]; 2],
    pub(super) se: [i64; 2],
    pub(super) employee_payroll_complete: bool,
    pub(super) employee_medicare_complete: bool,
    pub(super) withholding_25a: i64,
    pub(super) withholding_25b: i64,
    pub(super) withholding_other_dollars: i64,
    pub(super) rental: &'a crate::extended_rules::RentalResult,
    pub(super) se_sources_complete: [bool; 2],
}

pub(super) struct PayrollTaxOut {
    pub(super) payroll_withheld: i64,
    pub(super) payroll_other: i64,
    pub(super) excess_social: i64,
    pub(super) niit_threshold: i64,
    pub(super) known_additional_parts: [i64; 3],
    pub(super) additional_medicare_self_employment: i64,
    pub(super) additional_medicare_wages: i64,
    pub(super) additional_medicare_rrta_withheld: i64,
    pub(super) possible_additional_parts: [i64; 3],
    pub(super) medicare_withheld: i64,
    pub(super) injured_medicare_withholding:
        Box<crate::injured_spouse::income_categories::NativeMedicareWithholding>,
    pub(super) additional_medicare: i64,
    pub(super) additional_medicare_sources:
        Box<crate::injured_spouse::income_categories::NativeAdditionalMedicare>,
    pub(super) employee_payroll_complete: bool,
    pub(super) payroll_agi_known: bool,
    pub(super) payroll_se_net_known: bool,
}

pub(super) fn payroll_tax_phase(
    pass: &Pass,
    r: &mut FederalResult,
    t: &Totals,
    i: PayrollTaxIn<'_>,
) -> PayrollTaxOut {
    let &Pass { d, s, j, separate, .. } = pass;
    let PayrollTaxIn {
        tip_payroll,
        employee_wage_tax,
        uncollected_payroll,
        separate_niit,
        net,
        mut payroll_agi_known,
        mut payroll_se_net_known,
        se_church_wages,
        half,
        employee_coverage_possible,
        employee_coverage_taxes,
        se,
        mut employee_payroll_complete,
        mut employee_medicare_complete,
        withholding_25a,
        withholding_25b,
        withholding_other_dollars,
        rental,
        se_sources_complete,
    } = i;
    let payroll_withheld =
        dollars(t.social_tax.iter().sum::<i64>() + t.medicare_tax.iter().sum::<i64>());
    let payroll_other = tip_payroll + employee_wage_tax + uncollected_payroll;
    let excess_social = (0..2)
        .map(|i| {
            dollars(positive(
                t.social_tax[i] + t.uncollected_social[i]
                    - crate::year_constants::MAX_EMPLOYEE_SOCIAL_SECURITY_TAX_CENTS,
            ))
        })
        .sum::<i64>();
    let medicare_threshold = if j {
        crate::year_constants::ADDITIONAL_MEDICARE_THRESHOLD_JOINT_DOLLARS
    } else if separate {
        crate::year_constants::ADDITIONAL_MEDICARE_THRESHOLD_SEPARATE_DOLLARS
    } else {
        crate::year_constants::ADDITIONAL_MEDICARE_THRESHOLD_OTHER_DOLLARS
    };
    // Section 1411(b): a surviving spouse shares the joint threshold for the
    // investment tax, though Form 8959 gives that status $200,000.
    let niit_threshold = if s.filing_status() == FederalStatus::Surviving {
        crate::year_constants::NIIT_THRESHOLD_SURVIVING_DOLLARS
    } else if separate_niit {
        crate::year_constants::ADDITIONAL_MEDICARE_THRESHOLD_SEPARATE_DOLLARS
    } else {
        medicare_threshold
    };
    let medicare_wages = dollars(t.medicare_wages.iter().sum())
        + dollars(t.employee_tips.iter().map(|v| v.medicare).sum())
        + dollars(t.employee_payroll_medicare.iter().sum());
    // Form 8959 order (2026 draft): Part I (wages, with Part II RRTA and Part III line 12)
    // applies the threshold first; Part IV (self-employment) applies whatever threshold remains.
    // Schedule 2 line 11 is the self-employment part (line 18); line 17b is the wages part (line 12).
    let additional_medicare_wages_remaining_threshold = crate::kernel::carry::native_remaining::<
        crate::kernel::carry::Dollars,
    >(medicare_threshold, medicare_wages)
    .get();
    let rrta_compensation = dollars(t.rrta_compensation.iter().sum());
    let known_additional_parts = additional_medicare_parts(
        medicare_wages,
        rrta_compensation,
        net.iter().sum(),
        medicare_threshold,
    );
    let [
        additional_medicare_payroll_wages,
        additional_medicare_rrta,
        additional_medicare_self_employment,
    ] = known_additional_parts;
    let additional_medicare_wages = additional_medicare_payroll_wages + additional_medicare_rrta;
    let additional_medicare_rrta_withheld = dollars(t.rrta_additional_withheld.iter().sum());
    let possible_employee_medicare = crate::kernel::ratio::checked_sum(
        t.employee_payroll_medicare.into_iter().chain(t.employee_unknown_medicare),
    )
    .expect("employee aggregate boundary");
    let possible_wages = medicare_wages - dollars(t.employee_payroll_medicare.iter().sum())
        + dollars(possible_employee_medicare);
    let possible_additional_parts = additional_medicare_parts(
        possible_wages,
        rrta_compensation,
        net.iter().sum(),
        medicare_threshold,
    );
    // Availability is checked after all known SS wages and tips, so a saturated
    // wage base does not demand an immaterial missing SS coverage answer.
    for entry in &d.income {
        let FederalIncome::EmployeeWages { data } = &entry.income else {
            continue;
        };
        let owner = usize::from(entry.owner == crate::DocumentOwner::Spouses);
        if owner == 1 && !j {
            continue;
        }
        let context = crate::interview::income_issue_context(entry);
        if (matches!(data.payroll, crate::EmployeePayrollSource::ForeignGovernment)
            && data.wages > 0)
            || (matches!(data.payroll, crate::EmployeePayrollSource::RailroadRepresentative { .. })
                && (data.wages > 0
                    || data
                        .railroad()
                        .is_some_and(|source| source.known()[0].is_none_or(|amount| amount > 0))))
        {
            payroll_agi_known = false;
        }
        if matches!(data.payroll, crate::EmployeePayrollSource::ForeignGovernment) && data.wages > 0
        {
            payroll_se_net_known = false;
        }
        if data.railroad().is_some_and(|source| source.known()[0].is_none()) {
            let saturated = se_from_net(
                net[owner] - church_employee_se(se_church_wages[owner]),
                t.social_wages[owner].max(crate::year_constants::SOCIAL_SECURITY_WAGE_BASE_CENTS),
                se_church_wages[owner],
            );
            payroll_agi_known &= saturated.2 == half[owner];
        }
        let key = match &data.payroll {
            crate::EmployeePayrollSource::Railroad { reported }
            | crate::EmployeePayrollSource::RailroadRepresentative { reported }
                if reported.known()[0].is_none() =>
            {
                Some("ISSUE_EMPLOYEE_RRTA_COMPENSATION_UNAVAILABLE")
            }
            crate::EmployeePayrollSource::Railroad { .. }
            | crate::EmployeePayrollSource::RailroadRepresentative { .. }
                if data.wages > 0
                    || data.railroad().is_some_and(|source| {
                        source.known()[0].is_none_or(|amount| amount > 0)
                    }) =>
            {
                Some("ISSUE_EMPLOYEE_RAILROAD_ORDINARY_TAX")
            }
            crate::EmployeePayrollSource::ForeignGovernment if data.wages > 0 => {
                Some("ISSUE_EMPLOYEE_FOREIGN_GOVERNMENT_PAYROLL")
            }
            _ => {
                let coverage = data.coverage();
                let changed_social = t.social_wages[owner]
                    .checked_add(t.employee_unknown_social[owner])
                    .expect("employee aggregate boundary");
                let changed_se = se_from_net(
                    net[owner] - church_employee_se(se_church_wages[owner]),
                    changed_social,
                    se_church_wages[owner],
                );
                if coverage[0].is_none() && data.wages > 0 {
                    payroll_agi_known &= changed_se.2 == half[owner];
                }
                (data.wages > 0
                    && ((coverage[0].is_none()
                        && employee_coverage_possible[owner][0]
                            != employee_coverage_taxes[owner][0])
                        || (coverage[1].is_none()
                            && employee_coverage_possible[owner][1]
                                != employee_coverage_taxes[owner][1])
                        || (coverage[1].is_none()
                            && possible_additional_parts != known_additional_parts)
                        || (coverage[0].is_none() && changed_se.1 != se[owner])))
                    .then_some("ISSUE_EMPLOYEE_PAYROLL_COVERAGE_UNAVAILABLE")
            }
        };
        if let Some(key) = key {
            if key == "ISSUE_EMPLOYEE_PAYROLL_COVERAGE_UNAVAILABLE"
                || key == "ISSUE_EMPLOYEE_RRTA_COMPENSATION_UNAVAILABLE"
            {
                employee_payroll_complete = false;
            }
            if data.railroad().is_none()
                && data.coverage()[1].is_none()
                && possible_additional_parts != known_additional_parts
            {
                employee_medicare_complete = false;
            }
            crate::issue_localized!(
                r,
                &context,
                "Employee payroll coverage",
                key,
                "Actual payroll coverage or ordinary railroad/foreign-government payroll treatment is not available; known income and payroll components are preserved."
            );
        }
    }
    // i8959 line 19 explicitly adds ordinary codes B/N to box 6, but excludes
    // uncollected RRTA Medicare tax. Lines 19 and 20 round their source totals once; lines 21
    // and 22 then use those printed dollars. The cent operands below stay
    // exact for the injured-spouse owner split.
    let ordinary_medicare_withheld =
        t.medicare_tax.iter().sum::<i64>() + t.ordinary_uncollected_medicare;
    let ordinary_medicare_regular = fraction(t.medicare_wages.iter().sum(), 145, 10000);
    let ordinary_lines = crate::forms::form_8959::ordinary_withholding_lines(
        ordinary_medicare_withheld,
        t.medicare_wages.iter().sum(),
    );
    let ordinary_additional_withheld = ordinary_lines.additional;
    let medicare_withheld = additional_medicare_rrta_withheld + ordinary_additional_withheld;
    // Only the sole native collector establishes these cash categories. Keep
    // this projection even when a later unrelated tax issue suppresses lines.
    // Missing person-attribution metadata is not missing ordinary withholding.
    r.payment_withholding = Withheld::from_native(
        withholding_25a,
        withholding_25b,
        withholding_other_dollars,
        medicare_withheld,
    );

    // Retain the actual native targets and raw source operands, including
    // signed cancellation after the existing regular-Medicare subtraction.
    // Injured-spouse allocation consumes these annual categories separately; no per-owner clamp.
    let injured_medicare_withholding =
        Box::new(crate::injured_spouse::income_categories::NativeMedicareWithholding {
            railroad_unavailable: t.employee_rrta_withholding_unknown.iter().any(|value| *value),
            rrta_dollars: additional_medicare_rrta_withheld,
            rrta_withheld_cents: t.rrta_additional_withheld,
            ordinary_dollars: ordinary_additional_withheld,
            ordinary_net_cents: positive(ordinary_medicare_withheld - ordinary_medicare_regular),
            ordinary_withheld_cents: [0, 1]
                .map(|i| t.medicare_tax[i] + t.ordinary_uncollected_medicare_owners[i]),
            ordinary_wages_cents: t.medicare_wages,
            ordinary_regular_cents: ordinary_medicare_regular,
        });
    if medicare_wages > medicare_threshold
        || rrta_compensation > medicare_threshold
        || additional_medicare_rrta_withheld > 0
        || ordinary_additional_withheld > 0
        || net.iter().sum::<i64>() > additional_medicare_wages_remaining_threshold
    {
        for (reference, id, amount) in [
            ("1", "additional_medicare_w2_wages", dollars(t.medicare_wages.iter().sum())),
            (
                "2",
                "additional_medicare_tip_wages",
                dollars(t.employee_tips.iter().map(|v| v.medicare).sum()),
            ),
            ("3", "additional_medicare_employee_wages", dollars(t.employee_8919.iter().sum())),
            ("4", "additional_medicare_wages_total", medicare_wages),
            ("5", "additional_medicare_wages_threshold", medicare_threshold),
            (
                "6",
                "additional_medicare_wages_excess",
                positive(medicare_wages - medicare_threshold),
            ),
            ("7", "additional_medicare_payroll_wages", additional_medicare_payroll_wages),
            ("8", "additional_medicare_rrta_compensation", rrta_compensation),
            ("9", "additional_medicare_rrta_threshold", medicare_threshold),
            (
                "10",
                "additional_medicare_rrta_excess",
                phase::phase_out(rrta_compensation, medicare_threshold),
            ),
            ("11", "additional_medicare_rrta_tax", additional_medicare_rrta),
            ("12", "additional_medicare_wages_and_rrta", additional_medicare_wages),
            ("13", "additional_medicare_se_income", net.iter().sum()),
            ("14", "additional_medicare_se_threshold", medicare_threshold),
            ("15", "additional_medicare_se_wage_offset", medicare_wages),
            (
                "16",
                "additional_medicare_se_remaining_threshold",
                additional_medicare_wages_remaining_threshold,
            ),
            (
                "17",
                "additional_medicare_se_excess",
                phase::phase_out(net.iter().sum(), additional_medicare_wages_remaining_threshold),
            ),
            ("18", "additional_medicare_se_tax", additional_medicare_self_employment),
            ("23", "additional_medicare_rrta_withheld", additional_medicare_rrta_withheld),
        ] {
            let unprojected = t.employee_other_medicare.iter().any(|amount| *amount > 0);
            if (unprojected
                && matches!(reference, "4" | "6" | "7" | "12" | "15" | "16" | "17" | "18"))
                || (t.employee_rrta_compensation_unknown.iter().any(|unknown| *unknown)
                    && matches!(reference, "8" | "10" | "11" | "12"))
                || (reference == "23"
                    && t.employee_rrta_withholding_unknown.iter().any(|unknown| *unknown))
            {
                line(
                    r,
                    id,
                    amount,
                    &["income.employee_wages"],
                    "https://www.irs.gov/instructions/i8959",
                    msg!("EXPLAIN_EMPLOYEE_COVERAGE_PAYROLL"),
                );
            } else {
                form_line(
                    r,
                    FormLine::new(TaxForm::Form8959, reference),
                    id,
                    amount,
                    &["income.wages"],
                    "https://www.irs.gov/instructions/i8959",
                    msg!("EXPLAIN_8959_WAGES_AND_RRTA"),
                );
            }
        }
        if t.employee_other_medicare.iter().any(|amount| *amount > 0) {
            line(
                r,
                "additional_medicare_other_employee_wages",
                dollars(t.employee_other_medicare.iter().sum()),
                &["income.employee_wages"],
                "https://www.irs.gov/instructions/i8959",
                msg!("EXPLAIN_EMPLOYEE_COVERAGE_PAYROLL"),
            );
        }
    }
    for i in 0..2 {
        if t.rrta_tier_2_forms[i] > 1 {
            note_once(r, msg!("NOTE_RAILROAD_TIER_2_FORM_843"));
        }
    }
    // Form 8959 line 12 (Schedule 2 line 17b) and line 18 (Schedule 2 line 11) are each a
    // whole-dollar amount, so Form 1040 line 23 adds the separately rounded parts.
    let additional_medicare = additional_medicare_wages + additional_medicare_self_employment;
    // Retain the actual settled native part pools and their same-call operands.
    let additional_medicare_sources =
        Box::new(crate::injured_spouse::income_categories::NativeAdditionalMedicare {
            unavailable_parts: [
                !employee_medicare_complete
                    && possible_additional_parts[0] != known_additional_parts[0],
                t.employee_rrta_compensation_unknown.iter().any(|value| *value),
                !employee_medicare_complete
                    && possible_additional_parts[2] != known_additional_parts[2],
            ],
            tax_dollars: [
                additional_medicare_payroll_wages,
                additional_medicare_rrta,
                additional_medicare_self_employment,
            ],
            payroll_categories: [
                (dollars(t.medicare_wages.iter().sum()), t.medicare_wages),
                (
                    dollars(t.employee_tips.iter().map(|tips| tips.medicare).sum()),
                    [0, 1].map(|i| t.employee_tips[i].medicare),
                ),
                (dollars(t.employee_payroll_medicare.iter().sum()), t.employee_payroll_medicare),
            ],
            railroad: (rrta_compensation, t.rrta_compensation),
            se_joint_dollars: net.iter().sum(),
            se_dollars: (rental.inputs_complete && se_sources_complete.iter().all(|known| *known))
                .then_some(net),
        });
    if r.lines
        .iter()
        .any(|line| line.form_line.as_ref().is_some_and(|line| line.form == TaxForm::Form8959))
    {
        emit_lines(
            r,
            &mut (),
            [
                ("19", "additional_medicare_ordinary_withheld", ordinary_lines.withheld),
                ("20", "additional_medicare_withholding_wages", ordinary_lines.wages),
                ("21", "additional_medicare_regular_withholding", ordinary_lines.regular),
                (
                    "22",
                    "additional_medicare_ordinary_additional_withheld",
                    ordinary_lines.additional,
                ),
            ],
            |(reference, id, amount), _, _| {
                Some(calculation_row(
                    Some(FormLine::new(TaxForm::Form8959, reference)),
                    id,
                    LineValue::MoneyCents(amount * 100),
                    &["income.wages"],
                    "https://www.irs.gov/instructions/i8959",
                    msg!("EXPLAIN_8959_WAGES_AND_RRTA"),
                ))
            },
        );
        form_line(
            r,
            FormLine::new(TaxForm::Form8959, "24"),
            "additional_medicare_total_withheld",
            medicare_withheld,
            &["income.wages"],
            "https://www.irs.gov/instructions/i8959",
            msg!("EXPLAIN_8959_WAGES_AND_RRTA"),
        );
    }
    PayrollTaxOut {
        payroll_withheld,
        payroll_other,
        excess_social,
        niit_threshold,
        known_additional_parts,
        additional_medicare_self_employment,
        additional_medicare_wages,
        additional_medicare_rrta_withheld,
        possible_additional_parts,
        medicare_withheld,
        injured_medicare_withholding,
        additional_medicare,
        additional_medicare_sources,
        employee_payroll_complete,
        payroll_agi_known,
        payroll_se_net_known,
    }
}
