use crate::tests::*;

pub(in crate::tests) fn k1_supported(draft: &FederalDraft) -> FederalResult {
    let result = estimate(draft);
    assert_eq!(result.state, ResultState::EstimatedForSupportedScope, "{:?}", result.issues);
    result
}

pub(in crate::tests) fn k1_supported_without_issues(draft: &FederalDraft) -> FederalResult {
    let result = k1_supported(draft);
    assert!(result.issues.is_empty(), "{:?}", result.issues);
    result
}

pub(in crate::tests) fn k1_form4797_sum(result: &FederalResult, line: &str) -> i64 {
    result
        .lines
        .iter()
        .filter(|row| {
            row.form_line
                .as_ref()
                .is_some_and(|form| form.form == TaxForm::Form4797 && form.line == line)
        })
        .map(|row| row.amount / 100)
        .sum()
}
