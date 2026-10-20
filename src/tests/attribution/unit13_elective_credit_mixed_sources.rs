use super::*;
use crate::tests::cross::interview::{accepted, from_review};
use crate::tests::support::assert::injured_credit::elective_credit_mixed_sources::money;
use crate::tests::support::builders::injured_credit::elective_credit_mixed_sources::Family;
use crate::tests::support::client::injured_credit::elective_credit_mixed_sources::{
    actual_columns, ownership, persist, reproject,
};
use crate::tests::support::client::{draft_from_native_records, native_draft_records};

#[test]
fn unit13_elective_mixed_financial_source_save_and_restart_before_ownership() {
    for family in [Family::NoUse, Family::BindingUse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for order in 0..4 {
                persist(family, injured, order);
            }
        }
    }
}

#[test]
fn unit13_elective_mixed_no_use_preserves_both_actual_business_owners() {
    ownership(Family::NoUse);
}

#[test]
fn unit13_elective_mixed_binding_use_reuses_one_bounded_line16_answer() {
    ownership(Family::BindingUse);
}

#[test]
fn unit13_elective_mixed_unknown_and_mismatched_native_packet_stay_unavailable() {
    let mut failures = vec![];
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let d = persist(Family::BindingUse, injured, 0);
        let actual = money(&d, Family::BindingUse);
        let source = actual.elective_sources.as_ref().unwrap();
        assert!(source.valid());
        assert_eq!((source.joint, source.used_joint), (38808, 11192));
        assert_eq!(source.groups.len(), 1);
        assert_eq!(source.groups[0].use_bounds(0), Some([0, 5000]));
        assert_eq!(source.groups[0].use_bounds(1), Some([6192, 11192]));
        assert_eq!(source.groups[0].net_owners(Some([4000, 7192])), Some([1000, 37808]));
        let control = reproject(&d, &actual);
        actual_columns(&control, Family::BindingUse, injured, false);
        assert!(control.lines.iter().any(|row| row.id == crate::injured_spouse::ESTIMATE_LINE));
        for mode in [
            "missing",
            "mismatch",
            "empty-component",
            "contradictory-full-owners",
            "unknown-allowance",
        ] {
            let mut changed = actual.clone();
            match mode {
                "missing" => changed.elective_sources = None,
                "mismatch" => changed.elective_sources.as_mut().unwrap().joint += 1,
                "empty-component" => {
                    changed.elective_sources.as_mut().unwrap().groups[0].component.clear()
                }
                "contradictory-full-owners" => {
                    changed.elective_sources.as_mut().unwrap().groups[0].allowed_owners =
                        Some([45000, 5000])
                }
                _ => {
                    changed.elective_sources.as_mut().unwrap().groups[0].allowed_owners = None;
                    assert!(changed.elective_sources.as_ref().unwrap().valid());
                }
            }
            let result = reproject(&d, &changed);
            let unknown = ["16", "17"].into_iter().all(|line| {
                ["b", "c"].into_iter().all(|col| {
                    !result
                        .lines
                        .iter()
                        .any(|row| row.id == alloc::format!("form_8379_line_{line}_{col}"))
                })
            });
            let no_estimate =
                !result.lines.iter().any(|row| row.id == crate::injured_spouse::ESTIMATE_LINE);
            let no_replacement = result.injured_spouse.as_ref().unwrap().required.is_empty();

            if !(unknown && no_estimate && no_replacement) {
                failures.push(alloc::format!("{injured:?}/{mode}"));
            }
            assert_eq!(
                (
                    result.tax,
                    result.payments,
                    result.refundable_credits,
                    result.refund,
                    result.balance
                ),
                (
                    actual.tax,
                    actual.payments,
                    actual.refundable_credits,
                    actual.refund,
                    actual.balance
                )
            );
        }
    }
    assert!(failures.is_empty(), "genuine elective metadata owner failures: {failures:?}");
}

#[test]
fn unit13_elective_mixed_zero_net_positive_used_unknown_capacity_has_no_estimate() {
    let mut failures = vec![];
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        let mut d = persist(Family::BindingUse, injured, 0);
        // This separate5000D source fixture has no retained11192D-pool
        // choice. Its unique3000/2000 ownership is automatic, not a supplied
        // replacement for an unanswered shared control.
        d.injured_spouse.as_mut().unwrap().allocations.clear();
        let mut s = restart(d);
        let Reply::Edit(mut request @ P::SaveMoreCredits { .. }) =
            from_review(&mut s, TaxSection::MoreCredits)
        else {
            panic!()
        };
        let P::SaveMoreCredits { value: credit_page, .. } = &mut request else { unreachable!() };
        let MoreCredit::BusinessCredits(history) = credit_page
            .credits
            .iter_mut()
            .find(|credit| matches!(credit, MoreCredit::BusinessCredits(_)))
            .unwrap()
        else {
            unreachable!()
        };
        for row in &mut history.credits {
            //2026-10-15:411's unexecuted11000 election hypothesis was wrong:
            // netEPE=min(elected,allowed-used), not elected minus used.
            // This separate fixture instead has gross/elected5000D, fully
            // consumed by native use5000D. Original B50000 remains unchanged.
            row.amount = if row.amount == 500000 { 300000 } else { 200000 };
            row.election = CreditElection::ElectivePayment { amount: row.amount };
        }
        accepted(&send(&mut s, request));
        let tax_year = s.draft.tax_year;
        let bytes = native_draft_records(&s.draft);
        let draft = draft_from_native_records(tax_year, &bytes);
        let actual = supported(&restart(draft.clone()).draft);
        for (key, expected) in [
            ("agi", 190000),
            ("taxable_income", 157800),
            ("regular_income_tax", 24140),
            ("form_3800_line_14", 12948),
            ("general_business_credit", 5000),
        ] {
            assert_eq!(value(&actual, key), expected);
        }
        assert_eq!(actual.elective_sources.as_ref().unwrap().joint, 0);
        assert_eq!(
            (actual.tax, actual.payments, actual.refundable_credits, actual.refund, actual.balance),
            (Some(1914000), Some(10000), Some(0), Some(0), Some(1904000))
        );
        let mut plain = draft.clone();
        plain.injured_spouse = None;
        assert_eq!(actual.return_lines, supported(&plain).return_lines);
        let control = reproject(&draft, &actual);
        let i = usize::from(injured == Owner::Spouse);
        assert_eq!(
            [column(&control, "16", "b"), column(&control, "16", "c")],
            [[3000, 2000][i], [3000, 2000][1 - i]]
        );
        if injured == Owner::Taxpayer {
            assert!(control.lines.iter().any(|row| row.id == crate::injured_spouse::ESTIMATE_LINE));
        }
        for mode in ["unknown-allowance", "missing"] {
            let mut changed = actual.clone();
            if mode == "missing" {
                changed.elective_sources = None;
            } else {
                changed.elective_sources.as_mut().unwrap().groups[0].allowed_owners = None;
            }
            let result = reproject(&draft, &changed);
            let unknown = ["b", "c"].into_iter().all(|col| {
                !result.lines.iter().any(|row| row.id == alloc::format!("form_8379_line_16_{col}"))
            });
            let no_estimate =
                !result.lines.iter().any(|row| row.id == crate::injured_spouse::ESTIMATE_LINE);
            let no_replacement = result.injured_spouse.as_ref().unwrap().required.is_empty();

            if !(unknown && no_estimate && no_replacement) {
                failures.push((injured, mode));
            }
        }
    }
    assert!(failures.is_empty(), "unknown positive used capacity with zero netEPE: {failures:?}");
}
