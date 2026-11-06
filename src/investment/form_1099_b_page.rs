// form_1099_b items of src/federal/interview/mod.rs, split out by tools/split-hubs.
#[allow(unused_imports)]
use super::*;
pub(crate) use crate::capital_sale_model::native_sale_issue;
// The Sale1099B native guards, in their existing order: a wash-sale loss within the lot's
// loss, negative proceeds only for an option you wrote (which is short-term), an option premium
// only on an investment, and market discount only on a bond with a basis.
pub(super) fn sale_issue(v: &Sale1099B) -> Option<ValidationIssue> {
    if v.term.is_none() {
        return Some(ValidationIssue::new("SALE_TERM_NEEDED").at_field("term"));
    }
    native_sale_issue(&FederalSale::from(&crate::SaleSource::printed(v.clone())))
}

impl Interview {
    // The investment sales a passive Form 1116 source row can name.
    pub(super) fn capital_sale_choices(&self) -> Vec<crate::RecordChoice> {
        self.income_records(|entry| matches!(entry.income, FederalIncome::Sale { .. }))
    }
}
