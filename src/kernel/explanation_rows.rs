// Additive owner; consumers migrate after phase A merges.
#![allow(dead_code)]

use super::lines::{LineValue, row};
use crate::{CalculationLine, FormLine, FormLineKind, Message, MessageArg, TaxForm};
use alloc::vec::Vec;

// Static row metadata only. Amount units and emission remain owned by lines.
pub(crate) struct RowSpec {
    pub id: &'static str,
    pub inputs: &'static [&'static str],
    pub source: &'static str,
    pub explanation: &'static str,
    pub form_line: Option<(TaxForm, &'static str, FormLineKind)>,
}

impl RowSpec {
    pub fn build(&self, value: LineValue, args: Vec<MessageArg>) -> CalculationLine {
        let form_line = self.form_line.as_ref().map(|(form, line, kind)| match kind {
            FormLineKind::Line => FormLine::new(*form, line),
            FormLineKind::Subrow => FormLine::subrow(*form, line),
        });
        row(
            form_line,
            self.id,
            value,
            self.inputs,
            self.source,
            Message::new(self.explanation, args),
        )
    }
}
