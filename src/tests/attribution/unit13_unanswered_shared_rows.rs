use super::*;
use crate::tests::support::client::{draft_from_native_records, native_draft_records};

#[test]
fn unit13_unanswered_shared_credit_rows_do_not_publish_internal_bounds() {
    let mut observations = Vec::new();
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            let mut claim = allocation();
            claim.injured = injured;
            let mut draft = injured_return((60_000, 5_000), (20_000, 500), 0, claim);
            draft.credits.as_mut().unwrap().adoptions.push(adoption_child(
                AdoptionKind::Domestic {
                    finalized: AdoptionFinalization::Pending,
                    special_needs: false,
                },
                &[(2025, 3000)],
            ));
            if reverse {
                draft.income.reverse();
            }
            let result = supported(&draft);
            // The existing adoption fixture: AGI80000 minus MFJ32200.
            // The native tax-table midpoint47825 gives2480+2763=5243;
            // withholding5500 plus refundable3000 leaves3257.
            assert_eq!(value(&result, "agi"), 80_000);
            assert_eq!(value(&result, "taxable_income"), 47_800);
            assert_eq!(value(&result, "regular_income_tax"), 5243);
            assert_eq!(result.tax, Some(524_300));
            assert_eq!(result.refund, Some(325_700));
            assert_eq!(column(&result, "17", "a"), 3000);
            let index = usize::from(injured == Owner::Spouse);
            assert_eq!(column(&result, "13a", "b"), [60_000, 20_000][index]);
            assert_eq!(column(&result, "19", "b"), [5000, 500][index]);
            let plan = result.injured_spouse.as_ref().unwrap();
            assert!(plan.required.iter().any(|row| row.component == "adoption_refundable_credit"));
            assert!(!plan.accepted.iter().any(|row| row.component == "adoption_refundable_credit"));
            let mut ordinary = draft.clone();
            ordinary.injured_spouse = None;
            let baseline = supported(&ordinary);
            assert_eq!(result.return_lines, baseline.return_lines);
            assert_eq!(
                (result.tax, result.refund, result.balance),
                (baseline.tax, baseline.refund, baseline.balance)
            );
            let tax_year = draft.tax_year;
            let stored = native_draft_records(&draft);
            let reopened = draft_from_native_records(tax_year, &stored);
            assert_eq!(native_draft_records(&reopened), stored);
            assert_eq!(restart(reopened).draft, draft);
            observations.push(result.lines.iter().any(|line| {
                matches!(line.id.as_str(), "form_8379_line_17_b" | "form_8379_line_17_c")
            }));
        }
    }
    assert!(
        observations.iter().all(|present| !present),
        "an unanswered shared credit must not publish its internal lower-bound owner columns: {observations:?}"
    );
}
