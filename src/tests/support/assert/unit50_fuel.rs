use crate::tests::*;

// Whole-dollar Form 1040 payment cents are supplied independently of raw Form 4136 lines.
pub(in crate::tests) fn assert_fuel_payment(
    draft: &FederalDraft,
    baseline: &FederalResult,
    payment: i64,
) -> FederalResult {
    let report = supported(draft);
    assert_eq!(report.tax, baseline.tax);
    assert_eq!(return_line(&report, "11a / 11b"), return_line(baseline, "11a / 11b"));
    assert_eq!(return_line(&report, "31"), return_line(baseline, "31") + payment);
    assert_eq!(estimate(&restart(draft.clone()).draft), report);
    report
}
