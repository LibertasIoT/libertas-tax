use crate::kernel::lines::{LineValue, row};
use crate::{CalculationLine, FederalResult, FormLine};

pub(crate) fn emit<S, I, F>(result: &mut FederalResult, state: &mut S, entries: I, mut make: F)
where
    I: IntoIterator,
    F: FnMut(I::Item, &mut S, &FederalResult) -> Option<CalculationLine>,
{
    for entry in entries {
        if let Some(row) = make(entry, state, result) {
            result.lines.push(row);
        }
    }
}

pub(crate) fn line(
    result: &mut FederalResult,
    id: &str,
    amount: i64,
    inputs: &[&str],
    source: &str,
    explanation: crate::Message,
) {
    emit(
        result,
        &mut (),
        [row(None, id, LineValue::MoneyDollars(amount), inputs, source, explanation)],
        |row, _, _| Some(row),
    );
}
pub(crate) fn line_cents(
    result: &mut FederalResult,
    id: &str,
    amount: i64,
    inputs: &[&str],
    source: &str,
    explanation: crate::Message,
) {
    emit(
        result,
        &mut (),
        [row(None, id, LineValue::MoneyCents(amount), inputs, source, explanation)],
        |row, _, _| Some(row),
    );
}
pub(crate) fn form_line(
    result: &mut FederalResult,
    form_line: FormLine,
    id: &str,
    amount: i64,
    inputs: &[&str],
    source: &str,
    explanation: crate::Message,
) {
    emit(
        result,
        &mut (),
        [row(Some(form_line), id, LineValue::MoneyDollars(amount), inputs, source, explanation)],
        |row, _, _| Some(row),
    );
}
pub(crate) fn form_count(
    result: &mut FederalResult,
    form_line: FormLine,
    id: &str,
    count: i64,
    inputs: &[&str],
    source: &str,
    explanation: crate::Message,
) {
    emit(
        result,
        &mut (),
        [row(Some(form_line), id, LineValue::CountWhole(count), inputs, source, explanation)],
        |row, _, _| Some(row),
    );
}
pub(crate) fn form_value(
    result: &mut FederalResult,
    form_line: FormLine,
    id: &str,
    value: LineValue,
    inputs: &[&str],
    source: &str,
    explanation: crate::Message,
) {
    emit(
        result,
        &mut (),
        [row(Some(form_line), id, value, inputs, source, explanation)],
        |row, _, _| Some(row),
    );
}
pub(crate) fn form_ratio(
    result: &mut FederalResult,
    form_line: FormLine,
    id: &str,
    ten_thousandths: i64,
    inputs: &[&str],
    source: &str,
    explanation: crate::Message,
) {
    emit(
        result,
        &mut (),
        [row(
            Some(form_line),
            id,
            LineValue::RatioTenThousandths(ten_thousandths),
            inputs,
            source,
            explanation,
        )],
        |row, _, _| Some(row),
    );
}
pub(crate) fn push_line(
    result: &mut FederalResult,
    form_line: Option<FormLine>,
    id: &str,
    amount: i64,
    inputs: &[&str],
    source: &str,
    explanation: crate::Message,
) {
    emit(
        result,
        &mut (),
        [row(form_line, id, LineValue::MoneyCents(amount), inputs, source, explanation)],
        |row, _, _| Some(row),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, MessageArg, TaxForm};
    use alloc::{vec, vec::Vec};

    #[test]
    fn print_review_keeps_more_than_256_rows_and_text_has_no_money() {
        let mut result = FederalResult::empty(1);
        for id in 1..=300 {
            form_line(
                &mut result,
                FormLine::new(TaxForm::ScheduleC, "31"),
                &alloc::format!("business_{id}_net_profit"),
                id,
                &["income.business"],
                "https://www.irs.gov/instructions/i1040sc",
                Message::new("EXPLAIN_BUSINESS_N_NET_PROFIT", vec![]),
            );
        }
        form_value(
            &mut result,
            FormLine::new(TaxForm::ScheduleA, "19"),
            "itemized_below_standard_box",
            LineValue::Text,
            &["deductions"],
            "https://www.irs.gov/pub/irs-dft/f1040sa--dft.pdf",
            Message::new("PRINT_BOX_CHECKED", vec![]),
        );
        let review = crate::draft_return(&result);
        assert_eq!(review.len(), 301);
        assert_eq!(review.iter().filter_map(|row| row.money).sum::<i64>(), 4515000);
        let text = review
            .iter()
            .find(|row| row.form_line.as_ref().unwrap().form == TaxForm::ScheduleA)
            .unwrap();
        assert_eq!(text.money, None);
        assert_eq!(text.amount, 0);
        assert_eq!(Message::decode(&text.label).key, "DRAFT_ROW_WITH_TEXT");
    }

    #[test]
    fn print_subrow_survives_its_typed_total_without_repeating_the_total() {
        let mut result = FederalResult::empty(1);
        let printed = FormLine::new(TaxForm::ScheduleB, "1");
        result.return_lines.push(crate::FederalAmount {
            label: Message::new("LINE_SCHEDULE_B_INTEREST_N_OTHER", vec![MessageArg::Count(1)])
                .encode(),
            amount: 500,
            money: Some(500),
            explanation: Message::new("EXPLAIN_SCHEDULE_B_SOURCE", vec![]).encode(),
            form_line: Some(printed.clone()),
        });
        for form in [printed, FormLine::subrow(TaxForm::ScheduleB, "1")] {
            form_line(
                &mut result,
                form,
                "schedule_b_interest_1_other",
                5,
                &[],
                "source",
                Message::new("EXPLAIN_SCHEDULE_B_SOURCE", vec![]),
            );
        }
        let rows = crate::draft_return(&result);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].form_line.as_ref().unwrap().kind, crate::FormLineKind::Line);
        assert_eq!(rows[1].form_line.as_ref().unwrap().kind, crate::FormLineKind::Subrow);
        assert_eq!(rows[0].money, rows[1].money);
        result.return_lines = rows.clone();
        assert_eq!(crate::draft_return(&result), rows);
    }

    #[test]
    fn factories_observe_prior_rows_and_preserve_nested_order_without_filtering() {
        let mut result = FederalResult::empty(1);
        let mut calls = Vec::new();
        for source in [2, 1] {
            emit(&mut result, &mut calls, [false, true, true], |include, calls, current| {
                calls.push((source, include, current.lines.len()));
                if !include {
                    return None;
                }
                let amount = current.lines.last().map_or(-1, |r| r.amount + 1);
                Some(row(
                    None,
                    &alloc::format!("source_{source}"),
                    LineValue::MoneyCents(amount),
                    &[],
                    "source",
                    Message::new("EXPLAIN_FEDERAL_RULES_16", vec![]),
                ))
            });
        }
        assert_eq!(
            calls,
            vec![
                (2, false, 0),
                (2, true, 0),
                (2, true, 1),
                (1, false, 2),
                (1, true, 2),
                (1, true, 3)
            ]
        );
        assert_eq!(
            result.lines.iter().map(|r| (r.id.as_str(), r.amount)).collect::<Vec<_>>(),
            vec![("source_2", -1), ("source_2", 0), ("source_1", 1), ("source_1", 2)]
        );
        assert!(result.lines.iter().all(|r| r.form_line.is_none()));
    }
}
