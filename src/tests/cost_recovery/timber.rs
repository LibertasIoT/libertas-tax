use super::*;
use crate::tests::cross::interview::{
    assert_round_trip, business, save_businesses, stored_business,
};
use crate::tests::support::assert::{accepted, rejected_with};
use crate::tests::support::builders::timber::{timber_business, timber_cost, timber_mut};

// Original donor L controls, retained 2026-10-18: IRC194, Notice2006-47 C.2.
#[test]
fn timber_original_partial_and_absent_elections_keep_unelected_basis() {
    let mut cost = timber_cost();
    let recovery = cost.recovery(2026).unwrap();
    assert_eq!(
        (recovery.current_expense_cents, recovery.amortization_cents, recovery.remaining_cents),
        (1_000_000, 100_000, 1_300_000)
    );
    cost.facts.current_expense_elected = 500_000;
    cost.facts.amortizable_basis_elected = 1_900_000;
    let recovery = cost.recovery(2026).unwrap();
    assert_eq!(
        (recovery.current_expense_cents + recovery.amortization_cents, recovery.remaining_cents),
        (635_714, 1_764_286)
    );
    cost.facts.amortizable_basis_elected = 700_000;
    let recovery = cost.recovery(2026).unwrap();
    assert_eq!(
        (recovery.current_expense_cents + recovery.amortization_cents, recovery.remaining_cents),
        (550_000, 1_850_000)
    );
    cost.facts.current_expense_elected = 0;
    cost.facts.amortizable_basis_elected = 0;
    let recovery = cost.recovery(2026).unwrap();
    assert_eq!(
        (recovery.current_expense_cents, recovery.amortization_cents, recovery.remaining_cents),
        (0, 0, 2_400_000)
    );
}

#[test]
fn timber_reimbursement_reduces_basis_only_when_excluded() {
    let mut cost = timber_cost();
    cost.facts.government_reimbursement = 400_000;
    cost.facts.amortizable_basis_elected = 1_000_000;
    let recovery = cost.recovery(2026).unwrap();
    assert_eq!(
        (recovery.elected_basis_cents, recovery.amortization_cents, recovery.remaining_cents),
        (1_000_000, 71_429, 928_571)
    );
    cost.facts.reimbursement_taxable = true;
    let recovery = cost.recovery(2026).unwrap();
    assert_eq!(
        (recovery.elected_basis_cents, recovery.amortization_cents, recovery.remaining_cents),
        (1_000_000, 71_429, 1_328_571)
    );
    cost.facts.amortizable_basis_elected = 1_400_000;
    assert_eq!(cost.recovery(2026).unwrap().amortization_cents, 100_000);
    cost.facts.reimbursement_taxable = false;
    assert_eq!(cost.recovery(2026), None);
    assert!(!cost.valid());
}

#[test]
fn timber_actual_prior_history_and_disposal_month_do_not_restart_period() {
    let mut cost = timber_cost();
    cost.incurred = 20240401;
    cost.prior_allowed = crate::SourceCents::Reported { amount: 300_000 };
    let recovery = cost.recovery(2026).unwrap();
    assert_eq!(
        (recovery.current_expense_cents, recovery.amortization_cents, recovery.remaining_cents),
        (0, 200_000, 900_000)
    );
    cost.ended =
        Some(crate::AmortizationEnd { date: 20260921, how: crate::DisposalKind::Scrapped });
    let recovery = cost.recovery(2026).unwrap();
    assert_eq!(
        (recovery.current_expense_cents, recovery.amortization_cents, recovery.remaining_cents),
        (0, 150_000, 950_000)
    );
    cost.ended.as_mut().unwrap().date = 20250921;
    let recovery = cost.recovery(2026).unwrap();
    assert_eq!(
        (recovery.current_expense_cents, recovery.amortization_cents, recovery.remaining_cents),
        (0, 0, 1_100_000)
    );
    // 2026-10-18: historical end has no current sale and keeps actual basis;
    // the new property owner now resolves current supported dispositions.
    supported(&timber_business(cost));
}

#[test]
fn timber_save_reopen_restart_correction_and_forms_use_one_owner() {
    let mut page = business(10_000_000);
    page.more.timber = vec![timber_cost()];
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    let (id, data) = stored_business(&session);
    assert_eq!(data.timber, page.more.timber);
    assert_round_trip(&mut session, &[page.clone()]);
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_net_profit")), 89_000);
    assert_eq!(
        line_amount(&result, &alloc::format!("business_{id}_timber_0_amortization")),
        100_000
    );
    assert_eq!(
        line_amount(&result, &alloc::format!("business_{id}_timber_0_remaining")),
        1_300_000
    );
    assert_eq!(value(&result, &alloc::format!("business_{id}_form_4562b_total")), 1_000);
    assert_eq!(
        printed_line(&result, &alloc::format!("business_{id}_timber_0_amortization")),
        (TaxForm::Form4562B, "1(f)".into())
    );
    assert_eq!(
        printed_line(&result, &alloc::format!("business_{id}_form_4562b_total")),
        (TaxForm::Form4562B, "3".into())
    );
    page.more.timber[0].facts.current_expense_elected = 500_000;
    page.more.timber[0].facts.amortizable_basis_elected = 700_000;
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    assert_round_trip(&mut session, &[page]);
    let (id, _) = stored_business(&session);
    let corrected = supported(&restart(session.draft.clone()).draft);
    assert_eq!(value(&corrected, &alloc::format!("business_{id}_net_profit")), 94_500);
    assert_eq!(value(&corrected, &alloc::format!("business_{id}_form_4562b_total")), 500);
}

#[test]
fn timber_missing_changed_and_duplicate_histories_stay_unavailable() {
    let mut draft = timber_business(timber_cost());
    timber_mut(&mut draft).prior_allowed = crate::SourceCents::Unavailable;
    assert!(draft.valid_stored());
    unsupported(&draft, "Income:");
    timber_mut(&mut draft).prior_allowed = crate::SourceCents::Reported { amount: 0 };
    timber_mut(&mut draft).original_records_complete = false;
    unsupported(&draft, "Income:");
    timber_mut(&mut draft).original_records_complete = true;
    timber_mut(&mut draft).facts.domestic_commercial_timber = false;
    unsupported(&draft, "Income:");
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut first = business(10_000_000);
    first.more.timber.push(timber_cost());
    let mut duplicate = business(10_000_000);
    duplicate.more.timber = first.more.timber.clone();
    rejected_with(save_businesses(&mut session, vec![first, duplicate]), "TIMBER_COHORT_ONCE");
}

#[test]
fn timber_property_limit_uses_other_elections_and_preserves_old_mfs_amount() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Separate));
    let mut page = business(10_000_000);
    page.more.timber.push(timber_cost());
    rejected_with(
        save_businesses(&mut session, vec![page.clone()]),
        "TIMBER_PROPERTY_EXPENSE_LIMIT",
    );
    page.more.timber[0].facts.current_expense_elected = 500_000;
    page.more.timber[0].facts.amortizable_basis_elected = 1_900_000;
    accepted(&save_businesses(&mut session, vec![page.clone()]));
    let (id, _) = stored_business(&session);
    assert_eq!(
        line_amount(
            &supported(&session.draft),
            &alloc::format!("business_{id}_timber_0_amortization")
        ),
        135_714
    );
    page.more.timber[0].facts.other_current_expense = 1;
    rejected_with(
        save_businesses(&mut session, vec![page.clone()]),
        "TIMBER_PROPERTY_EXPENSE_LIMIT",
    );
    let cost = &mut page.more.timber[0];
    cost.incurred = 20240401;
    cost.facts.current_expense_elected = 1_000_000;
    cost.facts.amortizable_basis_elected = 1_400_000;
    cost.facts.other_current_expense = 0;
    cost.prior_allowed = crate::SourceCents::Reported { amount: 300_000 };
    accepted(&save_businesses(&mut session, vec![page]));
    let (id, _) = stored_business(&session);
    let result = supported(&session.draft);
    assert_eq!(value(&result, &alloc::format!("business_{id}_form_4562b_earlier")), 2_000);
    assert_eq!(line_amount(&result, &alloc::format!("business_{id}_timber_0_current_expense")), 0);
}

#[test]
fn timber_documented_absence_is_zero_and_invalid_reported_history_is_rejected() {
    let mut cost = timber_cost();
    cost.prior_allowed = crate::SourceCents::NotReported;
    assert_eq!(cost.recovery(2026).unwrap().amortization_cents, 100_000);
    assert!(timber_business(cost.clone()).valid_stored());
    cost.incurred = 20240401;
    cost.prior_allowed = crate::SourceCents::Reported { amount: 1_400_001 };
    assert!(!cost.valid());
}

#[test]
fn timber_invalid_save_locates_submitted_fields_and_is_atomic() {
    for (pages, key, path) in crate::tests::support::builders::timber::timber_invalid_editors() {
        let original = complete(FederalStatus::Separate);
        let mut session = Interview::new(TEST_USER, original.clone());
        let reply = save_businesses(&mut session, pages);
        let Reply::Response(P::Problem { field, .. }) = &reply else { panic!("{reply:?}") };
        assert_eq!(field.as_deref(), Some(path));
        rejected_with(reply, key);
        assert_eq!(session.draft, original);
    }
}
