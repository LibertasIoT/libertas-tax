// Transient PRINT projection of the actual section199A calculation. Every
// amount comes from its source owner; no eligibility or allocation is rerun.
use super::QbiLines;
use crate::kernel::lines::{LineValue, form_value};
use crate::{FederalResult, FormLine, TaxForm};

pub(crate) fn emit(r: &mut FederalResult, form: TaxForm, lines: &QbiLines) {
    if form != TaxForm::Form8995A {
        return;
    }
    let source = "https://www.irs.gov/pub/irs-dft/f8995a--dft.pdf";
    let inputs = ["income", "screening.qbi_history", "agi", "deduction"];
    let [threshold, phased, range] = lines.phase;
    let mut group_number = 0;
    for (index, ([income, netted, twenty, component], column)) in
        lines.activities.iter().zip(&lines.columns).enumerate()
    {
        let number = index + 1;
        let phase_in = phased > 0 && phased < range && column.wage_limit < *twenty;
        let mut rows = alloc::vec![];
        // Existing ungrouped rows retain their IDs and values. Aggregation
        // columns were formerly omitted altogether and need their own rows.
        if lines.grouped {
            rows.extend_from_slice(&[("2", *netted), ("3", *twenty), ("15", *component)]);
        }
        if phased > 0 {
            rows.extend_from_slice(&[
                ("4", column.wages[0]),
                ("5", column.wages[1]),
                ("6", column.wages[2]),
                ("7", column.property[0]),
                ("8", column.property[1]),
                ("9", column.wages[2] + column.property[1]),
                ("10", column.wage_limit),
                ("11", column.full_limit),
            ]);
        }
        if phase_in {
            rows.extend_from_slice(&[
                ("12", column.phased),
                ("17", *twenty),
                ("18", column.wage_limit),
                ("19", *twenty - column.wage_limit),
                ("20", lines.taxable),
                ("21", threshold),
                ("22", phased),
                ("23", range),
                ("25", column.phase_reduction),
                ("26", column.phased),
            ]);
            form_value(
                r,
                FormLine::new(form, "24"),
                &alloc::format!("qbi_column_{number}_line_24"),
                LineValue::PercentageHundredths(crate::kernel::ratio::fraction(
                    phased, 10000, range,
                )),
                &inputs,
                source,
                msg!("EXPLAIN_QBI_COMPUTED_COLUMN"),
            );
        }
        if !lines.patron || lines.grouped {
            rows.push(("13", lines.pre_patron[index]));
        }
        if lines.patron && lines.grouped {
            rows.push(("14", lines.patron_reductions[index]));
        }
        for (line, amount) in rows {
            form_value(
                r,
                FormLine::new(form, line),
                &alloc::format!("qbi_column_{number}_line_{line}"),
                LineValue::MoneyDollars(amount),
                &inputs,
                source,
                msg!("EXPLAIN_QBI_COMPUTED_COLUMN"),
            );
        }
        if lines.losses[1] > 0 {
            for (name, cell, amount) in [
                ("income", "1(a)", *income),
                ("reduction", "1(b)", -column.loss),
                ("adjusted", "1(c)", *netted),
            ] {
                form_value(
                    r,
                    FormLine::subrow(form, &alloc::format!("Schedule C {cell}")),
                    &alloc::format!("qbi_loss_column_{number}_{name}"),
                    LineValue::MoneyDollars(amount),
                    &inputs,
                    "https://www.irs.gov/pub/irs-pdf/f8995ac.pdf",
                    msg!("EXPLAIN_QBI_COMPUTED_COLUMN"),
                );
            }
        }
        let members = &lines.members[index];
        if members.len() < 2 {
            continue;
        }
        group_number += 1;
        for member in members {
            for (axis, name, cell) in
                [(0, "income", "(c)"), (1, "wages", "(d)"), (2, "property", "(e)")]
            {
                form_value(
                    r,
                    FormLine::subrow(form, &alloc::format!("Schedule B {group_number} 3{cell}")),
                    &alloc::format!("qbi_aggregation_{group_number}_member_{}_{name}", member + 1),
                    LineValue::MoneyDollars(lines.source_values[*member][axis]),
                    &inputs,
                    "https://www.irs.gov/pub/irs-pdf/f8995ab.pdf",
                    msg!("EXPLAIN_QBI_AGGREGATION_SOURCE"),
                );
            }
        }
        for (name, cell, amount) in [
            ("income", "(c)", *income),
            ("wages", "(d)", column.wages[0]),
            ("property", "(e)", column.property[0]),
        ] {
            form_value(
                r,
                FormLine::new(form, &alloc::format!("Schedule B {group_number} 4{cell}")),
                &alloc::format!("qbi_aggregation_{group_number}_total_{name}"),
                LineValue::MoneyDollars(amount),
                &inputs,
                "https://www.irs.gov/pub/irs-pdf/f8995ab.pdf",
                msg!("EXPLAIN_QBI_AGGREGATION_SOURCE"),
            );
        }
    }
    form_value(
        r,
        FormLine::new(form, "16"),
        "qbi_columns_total",
        LineValue::MoneyDollars(lines.component),
        &inputs,
        source,
        msg!("EXPLAIN_QBI_COMPUTED_COLUMN"),
    );
    if lines.losses[1] > 0 {
        for (line, amount) in [
            ("2", -lines.losses[0]),
            ("3", -lines.losses[1]),
            ("4", lines.losses[2]),
            ("5", -lines.losses[3]),
        ] {
            form_value(
                r,
                FormLine::new(form, &alloc::format!("Schedule C {line}")),
                &alloc::format!("qbi_loss_line_{line}"),
                LineValue::MoneyDollars(amount),
                &inputs,
                "https://www.irs.gov/pub/irs-pdf/f8995ac.pdf",
                msg!("EXPLAIN_QBI_COMPUTED_COLUMN"),
            );
        }
    }
}
