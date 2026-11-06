use crate::{CalculationLine, FormLine, LineKind, Message};
use alloc::string::String;

// Units belong to the caller. Emission never rounds or changes a tax figure.
pub(crate) enum LineValue {
    MoneyDollars(i64),
    MoneyCents(i64),
    CountWhole(i64),
    RatioTenThousandths(i64),
    Year(i64),
    PercentageHundredths(i64),
    Text,
}

pub(crate) fn row(
    form_line: Option<FormLine>,
    id: &str,
    value: LineValue,
    inputs: &[&str],
    source: &str,
    explanation: Message,
) -> CalculationLine {
    let (amount, kind) = match value {
        LineValue::MoneyDollars(v) => (v * 100, LineKind::Money),
        LineValue::MoneyCents(v) => (v, LineKind::Money),
        LineValue::CountWhole(v) => (v * 100, LineKind::Count),
        LineValue::RatioTenThousandths(v) => (v, LineKind::Ratio),
        LineValue::Year(v) => (v * 100, LineKind::Year),
        LineValue::PercentageHundredths(v) => (v, LineKind::Percentage),
        LineValue::Text => (0, LineKind::Text),
    };
    let id = id.into();
    CalculationLine {
        rule: alloc::format!("US-2026.{id}"),
        id,
        amount,
        kind,
        inputs: inputs.iter().map(|s| String::from(*s)).collect(),
        source: source.into(),
        explanation,
        form_line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MessageArg, TaxForm};
    use alloc::vec;

    #[test]
    fn units_and_all_row_fields_preserve_zero_negative_and_duplicate_inputs() {
        let explanation = Message::new(
            "EXPLAIN_SCHEDULE_B_SOURCE",
            vec![
                MessageArg::Text("source".into()),
                MessageArg::Resource("SCHEDULE_B_PAYER".into()),
            ],
        );
        for (value, amount, kind) in [
            (LineValue::MoneyDollars(-7), -700, LineKind::Money),
            (LineValue::MoneyCents(-7), -7, LineKind::Money),
            (LineValue::MoneyCents(0), 0, LineKind::Money),
            (LineValue::CountWhole(3), 300, LineKind::Count),
            (LineValue::RatioTenThousandths(1234), 1234, LineKind::Ratio),
            (LineValue::Year(2008), 200800, LineKind::Year),
            (LineValue::PercentageHundredths(3500), 3500, LineKind::Percentage),
            (LineValue::Text, 0, LineKind::Text),
        ] {
            assert_eq!(
                row(
                    Some(FormLine::new(TaxForm::Form1116, "3f")),
                    "id",
                    value,
                    &["second", "first", "second"],
                    "source",
                    explanation.clone(),
                ),
                CalculationLine {
                    id: "id".into(),
                    amount,
                    kind,
                    inputs: vec!["second".into(), "first".into(), "second".into()],
                    rule: "US-2026.id".into(),
                    source: "source".into(),
                    explanation: explanation.clone(),
                    form_line: Some(FormLine::new(TaxForm::Form1116, "3f")),
                }
            );
        }
    }
}
