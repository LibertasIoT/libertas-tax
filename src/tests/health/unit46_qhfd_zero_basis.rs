// Bounded traditional IRA funding and annual basis lifecycle.
use crate::tests::cross::interview::{
    accepted, hsa_account, hsa_page_value, open_hsa_page, open_topic, rejected_with,
};
use super::*;

fn fixture(value: i64) -> FederalDraft {
    let mut draft = hsa_prior_basis_funding(1500, 200, value, 200, 0);
    let source = draft.adjustments.as_ref().unwrap().hsa[0].funding[0].income;
    let FederalIncome::Retirement { data } = &mut draft
        .income
        .iter_mut()
        .find(|row| row.id == source)
        .unwrap()
        .income
    else {
        unreachable!()
    };
    data.ordinary_distribution = RetirementTreatment::Ordinary;
    add(
        &mut draft,
        basis_distribution(Owner::Taxpayer, 300, RetirementTreatment::Ordinary),
    );
    let transfer = &mut draft.adjustments.as_mut().unwrap().hsa[0].funding[0];
    // Actual transfer-event aggregate FMV, not a receipt-date reconstruction.
    transfer.basis = HsaFundingBasis::PriorBasis {
        distributed: 20260615,
        value: value * 100,
        before_other_activity: false,
    };
    transfer.funded = 20260616; // Actual HSA receipt; no invented IRA distribution date.
    draft
}
fn check_oracle(draft: &FederalDraft) -> FederalResult {
    let output = supported(draft);
    assert!(!noted(&output, "WARN_HSA_FUNDING_LEFT_OUT"));
    assert_eq!(value(&output, "taxpayer_ira_basis_recovered"), 120);
    assert_eq!(value(&output, "taxpayer_ira_basis_carryforward"), 80);
    assert_eq!(return_line(&output, "4a"), 180_000);
    assert_eq!(return_line(&output, "4b"), 18_000);
    assert_eq!(value(&output, "early_distribution_tax"), 0);
    assert_eq!(value(&output, "agi"), 60_180);
    let transfer = &draft.adjustments.as_ref().unwrap().hsa[0].funding[0];
    assert_eq!(
        value(
            &output,
            &alloc::format!("hsa_funding_basis_{}_{}", transfer.income, transfer.funded)
        ),
        0
    );
    output
}

// Independently derived composition, not a published combined IRS example.
// Notice2008-51 otherwise-taxable-first rule, pp2–3; final2025 I8606 line7
// excludes QHFD and PartI annual ratio is200/(300+200)=.400 exactly. With no
// basis additions, transfer-date basis <=200; FMV1700 covers1500+200, hence
// funding basis0, ordinary nontax120/tax180, carry80. Source-year qualification
// remains2025 instructions applied to an otherwise neutral2026 synthetic return.
// https://www.irs.gov/pub/irs-drop/n-08-51.pdf
// https://www.irs.gov/instructions/i8606
#[test]
fn unit46_qhfd_sufficient_no_basis_use_is_independent_of_ordinary_order() {
    let before = fixture(1700); // Actual source FMV after ordinary300, before QHFD1500.
    let after = fixture(2000); // Actual source FMV before QHFD; ordinary follows.
    let before_output = check_oracle(&before);
    let after_output = check_oracle(&after);
    for row in [
        "agi",
        "taxpayer_ira_basis_recovered",
        "taxpayer_ira_basis_carryforward",
    ] {
        assert_eq!(value(&before_output, row), value(&after_output, row));
    }
    let mut reordered = before.clone();
    reordered.income.reverse();
    assert_eq!(supported(&reordered), before_output);
    assert_eq!(supported(&restart(before.clone()).draft), before_output);
}

// Notice 2008-51 example: $200 basis, $2,000 value, $1,500 QHFD leaves $200 basis
// in a $500 IRA; Form 8606 instructions keep the QHFD out of line 7.
// https://www.irs.gov/pub/irs-drop/n-08-51.pdf
#[test]
fn unit46_qhfd_notice_example_keeps_basis_with_other_activity_unconfirmed() {
    let mut draft = hsa_prior_basis_funding(1500, 200, 2000, 500, 0);
    draft.adjustments.as_mut().unwrap().hsa[0].funding[0].basis = HsaFundingBasis::PriorBasis {
        distributed: 20260115,
        value: 200_000,
        before_other_activity: false,
    };
    let output = supported(&draft);
    assert!(!noted(&output, "WARN_HSA_FUNDING_LEFT_OUT"));
    assert_eq!(value(&output, "hsa_funding_basis_2_20260115"), 0);
    assert_eq!(value(&output, "taxpayer_ira_basis_carryforward"), 200);
}

// Notice 2008-51: basis in an excess over the includible dollars does not carry to the HSA.
#[test]
fn unit46_qhfd_above_includible_dollars_uses_only_the_excess_basis() {
    let mut draft = fixture(1700);
    let HsaFundingBasis::PriorBasis { value: before, .. } =
        &mut draft.adjustments.as_mut().unwrap().hsa[0].funding[0].basis
    else {
        unreachable!()
    };
    *before -= 10_000; // 1600 less 200 basis leaves 1400 includible of 1500.
    let output = supported(&draft);
    assert!(!noted(&output, "WARN_HSA_FUNDING_LEFT_OUT"));
    let id = draft.adjustments.as_ref().unwrap().hsa[0].funding[0].income;
    assert_eq!(value(&output, &alloc::format!("hsa_funding_basis_{id}_20260616")), 100);
}

#[test]
fn unit46_qhfd_cannot_borrow_another_owners_basis() {
    let mut draft = fixture(1700);
    draft.adjustments.as_mut().unwrap().ira_basis[0].owner = Owner::Spouse;
    unsupported(&draft, "HSA funding");
}

#[test]
fn unit46_qhfd_zero_basis_normal_save_reopen_restart_and_forged_source_are_atomic() {
    let mut draft = fixture(1700);
    let funding = draft.adjustments.as_mut().unwrap().hsa[0].funding[0].clone();
    draft.adjustments.as_mut().unwrap().hsa.clear();
    let mut session = Interview::new(TEST_USER, draft);
    let mut request = open_hsa_page(&mut session);
    let P::SaveHsa { value: inputs, .. } = &mut request else {
        unreachable!()
    };
    inputs.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
    inputs.family_split = FamilySplit::NotApplicable;
    inputs.funding = vec![funding.row(Owner::Taxpayer)];
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    let output = check_oracle(&saved);
    let reopened = open_hsa_page(&mut session);
    assert_eq!(
        hsa_page_value(&reopened).0.funding,
        vec![funding.row(Owner::Taxpayer)]
    );
    assert_eq!(hsa_page_value(&reopened).0.funding[0].funded, 20260616);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    let mut restored = restart(saved.clone());
    assert_eq!(check_oracle(&restored.draft), output);
    let reopened = open_hsa_page(&mut restored);
    assert_eq!(
        hsa_page_value(&reopened).0.funding,
        vec![funding.row(Owner::Taxpayer)]
    );
    let mut invalid = reopened;
    let P::SaveHsa { value: inputs, .. } = &mut invalid else {
        unreachable!()
    };
    inputs.funding[0].source = 1;
    rejected_with(
        send(&mut restored, invalid),
        "HSA_FUNDING_SOURCE_NOT_OFFERED",
    );
    assert_eq!(restored.draft, saved);
    let mut removal = open_topic(&mut restored, TaxSection::Retirement);
    let P::SaveIncomes { documents, .. } = &mut removal else {
        unreachable!()
    };
    documents.documents.retain(|row| row.id != funding.income);
    rejected_with(
        send(&mut restored, removal),
        "HSA_FUNDING_SOURCE_NOT_OFFERED",
    );
    assert_eq!(restored.draft, saved);
    assert_eq!(check_oracle(&restored.draft), output);
}
