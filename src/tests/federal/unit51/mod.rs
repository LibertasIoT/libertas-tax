use crate::kernel::{explanation_rows::RowSpec, lines};
use crate::{FederalResult, FormLine, FormLineKind, LineKind, MessageArg, TaxForm};
use alloc::vec;

const SOURCE: RowSpec = RowSpec {
    id: "row",
    inputs: &["second", "first", "second"],
    source: "https://www.irs.gov/instructions/i1040sc",
    explanation: "EXPLAIN_SCHEDULE_B_SOURCE",
    form_line: Some((TaxForm::ScheduleB, "1", FormLineKind::Subrow)),
};

#[test]
fn unit51_descriptor_keeps_typed_values_and_complete_row_metadata() {
    let args = vec![MessageArg::Text("source".into()), MessageArg::Count(2)];
    for (value, amount, kind) in [
        (lines::LineValue::MoneyDollars(7), 700, LineKind::Money),
        (lines::LineValue::MoneyCents(-7), -7, LineKind::Money),
        (lines::LineValue::CountWhole(2), 200, LineKind::Count),
        (lines::LineValue::RatioTenThousandths(1234), 1234, LineKind::Ratio),
        (lines::LineValue::Year(2026), 202600, LineKind::Year),
        (lines::LineValue::PercentageHundredths(3500), 3500, LineKind::Percentage),
        (lines::LineValue::Text, 0, LineKind::Text),
    ] {
        let row = SOURCE.build(value, args.clone());
        assert_eq!((row.amount, row.kind), (amount, kind));
        assert_eq!((row.id.as_str(), row.rule.as_str()), ("row", "US-2026.row"));
        assert_eq!(row.inputs, vec!["second", "first", "second"]);
        assert_eq!(row.source, "https://www.irs.gov/instructions/i1040sc");
        assert_eq!(row.explanation.key, "EXPLAIN_SCHEDULE_B_SOURCE");
        assert_eq!(row.explanation.args, args);
        assert_eq!(row.form_line, Some(FormLine::subrow(TaxForm::ScheduleB, "1")));
    }
}

#[test]
fn unit51_table_uses_existing_emitter_order_and_optional_rows() {
    let plain = RowSpec { form_line: None, ..SOURCE };
    let printed =
        RowSpec { form_line: Some((TaxForm::ScheduleB, "1", FormLineKind::Line)), ..SOURCE };
    let mut result = FederalResult::empty(0);
    let mut counts = vec![];
    lines::emit(
        &mut result,
        &mut counts,
        [(&plain, Some(-1)), (&printed, None), (&plain, Some(0)), (&plain, Some(2))],
        |(spec, amount), counts, current| {
            counts.push(current.lines.len());
            amount.map(|amount| spec.build(lines::LineValue::MoneyCents(amount), vec![]))
        },
    );
    assert_eq!(counts, vec![0, 1, 1, 2]);
    assert_eq!(
        result.lines.iter().map(|row| row.amount).collect::<alloc::vec::Vec<_>>(),
        vec![-1, 0, 2]
    );
    assert!(result.lines.iter().all(|row| row.id == "row" && row.form_line.is_none()));
    assert_eq!(
        printed.build(lines::LineValue::MoneyCents(0), vec![]).form_line,
        Some(FormLine::new(TaxForm::ScheduleB, "1"))
    );
}
