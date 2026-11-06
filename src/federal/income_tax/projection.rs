// Form6251's actual preferential-rate computation. The tax owner supplies all
// components, including unrounded rate cents and its once-rounded final tax.
use super::AmtTaxResult;
use crate::kernel::lines::LineValue;
use crate::line_emission::form_value;
use crate::{FederalResult, FormLine, TaxForm};

pub(crate) fn emit(result: &mut FederalResult, computation: &AmtTaxResult, required: bool) {
    if !required {
        return;
    }
    let inputs = ["income", "deductions", "alternative_minimum_taxable_income"];
    let source = "https://www.irs.gov/pub/irs-dft/f6251--dft.pdf";
    for &(line, cents) in &computation.part_three {
        form_value(
            result,
            FormLine::new(TaxForm::Form6251, &alloc::format!("{line}")),
            &alloc::format!("amt_capital_line_{line}"),
            LineValue::MoneyCents(cents),
            &inputs,
            source,
            msg!("EXPLAIN_AMT_CAPITAL_COMPUTATION"),
        );
    }
    if let Some(values) = &computation.foreign {
        for (line, &cents) in ["1", "2c", "3", "4", "5", "6"].into_iter().zip(values) {
            form_value(
                result,
                FormLine::subrow(
                    TaxForm::Form6251,
                    &alloc::format!("Foreign Earned Income Tax Worksheet {line}"),
                ),
                &alloc::format!("amt_foreign_worksheet_line_{line}"),
                LineValue::MoneyCents(cents),
                &inputs,
                "https://www.irs.gov/instructions/i6251",
                msg!("EXPLAIN_AMT_FOREIGN_COMPUTATION"),
            );
        }
    }
}
