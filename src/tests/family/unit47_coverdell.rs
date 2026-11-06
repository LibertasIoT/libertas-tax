// Unit47 native Coverdell diagnostic/Save controls.
// Existing published monetary witness remains unchanged in tests/other_taxes.rs.
extern crate std;
use super::*;
use crate::tests::cross::interview::{
    open_other_taxes, other_taxes_request, save_other_taxes, save_you,
};

fn accepted(reply: Reply) {
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
}
pub(super) fn record() -> CoverdellExcessPage {
    CoverdellExcessPage {
        beneficiary: Owner::Taxpayer,
        contributors: vec![CoverdellContributorPage {
            label: "Actual organization source".into(),
            person: CoverdellContributorKindPage::Organization,
            contributions: vec![CoverdellContribution {
                account: "Actual ESA".into(),
                paid: 20261201,
                amount: 145_000,
            }],
        }],
        corrections: vec![],
        prior_excess: 55_000,
        prior_tax: 3300,
        distributions: 0,
        value: 1_000_000,
        special_needs: false,
    }
}
// A calculated tax listed before the Coverdell topic, so the topic's outer
// index is 1.
pub(super) fn lead_tax() -> OtherTax {
    OtherTax::HouseholdEmployees { employers: vec![(&household_employer(3_000)).into()] }
}
pub(super) fn topic(beneficiaries: Vec<CoverdellExcessPage>) -> OtherTax {
    OtherTax::CoverdellExcess { beneficiaries }
}
fn session() -> Interview {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut person = crate::interview::filer(&s.draft.people.as_ref().unwrap().taxpayer);
    person.birth_date = 20100301;
    let request = save_you(&s.draft.clone(), person);
    accepted(send(&mut s, request));
    accepted(save_other_taxes(&mut s, vec![topic(vec![record()])]));
    s
}
fn assert_refusal_unchanged(
    s: &mut Interview,
    beneficiaries: Vec<CoverdellExcessPage>,
    key: &str,
    expected_field: Option<&str>,
) {
    let before = s.draft.clone();
    let dirty = s.dirty;
    let page = open_other_taxes(s);
    let request = other_taxes_request(&page, &s.draft, vec![lead_tax(), topic(beneficiaries)]);
    let reply = send(s, request.clone());
    if let Ok(path) = std::env::var("TAX_UNIT47_REQUEST") {
        std::fs::write(path, request.to_avro()).unwrap();
        let Reply::Response(response) = &reply else { panic!("expected response") };
        std::fs::write(std::env::var("TAX_UNIT47_RESPONSE").unwrap(), response.to_avro()).unwrap();
    }
    let Reply::Response(P::Problem { message, field }) = reply else { panic!("expected {key}") };
    assert_eq!(message, libertas::libertas_formatted_text(key, &[]));
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, dirty);
    assert_eq!(field.as_deref(), expected_field);
    // The leading actual other-tax source proves the submitted outer index is1,
    // independent of accepted source IDs or filtered Coverdell positions.
}

#[test]
fn unit47_coverdell_duplicate_donor_label_uses_local_key_atomically() {
    let mut s = session();
    let mut candidate = record();
    candidate.contributors.push(CoverdellContributorPage {
        label: candidate.contributors[0].label.clone(),
        person: CoverdellContributorKindPage::Unavailable,
        contributions: vec![],
    });
    assert_refusal_unchanged(
        &mut s,
        vec![candidate],
        "COVERDELL_DONOR_TWICE",
        Some("[value/taxes/1/beneficiaries/0/contributors/1/label]"),
    );
}

#[test]
fn unit47_coverdell_duplicate_filer_with_distinct_labels_uses_local_key_atomically() {
    let mut s = session();
    let mut candidate = record();
    candidate.contributors = ["First label", "Distinct second label"]
        .into_iter()
        .map(|label| CoverdellContributorPage {
            label: label.into(),
            person: CoverdellContributorKindPage::Filer { owner: Owner::Taxpayer },
            contributions: vec![],
        })
        .collect();
    assert_refusal_unchanged(
        &mut s,
        vec![candidate],
        "COVERDELL_DONOR_TWICE",
        Some("[value/taxes/1/beneficiaries/0/contributors/1/person/owner]"),
    );
}

#[test]
fn unit47_coverdell_duplicate_beneficiary_uses_local_key_atomically() {
    let mut s = session();
    assert_refusal_unchanged(
        &mut s,
        vec![record(), record()],
        "COVERDELL_BENEFICIARY_TWICE",
        Some("[value/taxes/1/beneficiaries/1/beneficiary]"),
    );
}

#[test]
fn unit47_coverdell_correction_loss_boundary_saves_reopens_and_restarts() {
    let mut s = session();
    let mut candidate = record();
    candidate.corrections.push(CoverdellCorrectionPage {
        account: "Actual ESA".into(),
        principal: 10_000,
        withdrawn: 20270501,
        earnings: -10_000,
        income_document: None,
    });
    // Zero proceeds is a valid equality boundary, not a negative deduction.
    accepted(save_other_taxes(&mut s, vec![topic(vec![candidate.clone()])]));
    let before = s.draft.clone();
    let baseline = estimate(&before);
    let mut again = restart(before.clone());
    let P::SaveOtherTaxes { value: shown, .. } = open_other_taxes(&mut again) else {
        unreachable!()
    };
    assert_eq!(shown.taxes, vec![topic(vec![candidate.clone()])]);
    accepted(save_other_taxes(&mut again, shown.taxes));
    assert_eq!(again.draft, before);
    assert_eq!(estimate(&again.draft), baseline);
    candidate.corrections[0].earnings = -10_001;
    assert_refusal_unchanged(
        &mut again,
        vec![candidate],
        "COVERDELL_LOSS_ABOVE_PRINCIPAL",
        Some("[value/taxes/1/beneficiaries/0/corrections/0/earnings]"),
    );
}

#[test]
fn unit47_coverdell_long_donor_account_labels_save_and_reopen() {
    let mut s = session();
    let mut candidate = record();
    // Donor and account names can occupy the full 400-byte allowance.
    let boundary = "🧾".repeat(100);
    candidate.contributors[0].label = boundary.clone();
    candidate.contributors[0].contributions[0].account = boundary.clone();
    accepted(save_other_taxes(&mut s, vec![topic(vec![candidate.clone()])]));
    let stored = s.draft.clone();
    let baseline = estimate(&stored);
    let mut again = restart(stored.clone());
    let P::SaveOtherTaxes { value: shown, .. } = open_other_taxes(&mut again) else {
        unreachable!()
    };
    assert_eq!(shown.taxes, vec![topic(vec![candidate.clone()])]);
    accepted(save_other_taxes(&mut again, shown.taxes));
    assert_eq!(again.draft, stored);
    assert_eq!(estimate(&again.draft), baseline);
}

#[test]
fn unit47_coverdell_cumulative_return_path_uses_submitted_correction_index() {
    let mut s = session();
    let mut candidate = record();
    candidate.corrections = [(100_000, 20261215), (45_001, 20270501)]
        .into_iter()
        .map(|(principal, withdrawn)| CoverdellCorrectionPage {
            account: "Actual ESA".into(),
            principal,
            withdrawn,
            earnings: 0,
            income_document: None,
        })
        .collect();
    // The first actual return is within deposits; the later cumulative145001
    // exceeds145000 by1cent. The first failing submitted correction is row1.
    assert_refusal_unchanged(
        &mut s,
        vec![candidate],
        "COVERDELL_RETURN_ABOVE_CONTRIBUTIONS",
        Some("[value/taxes/1/beneficiaries/0/corrections/1/principal]"),
    );
}

#[test]
fn unit47_coverdell_repeated_topic_path_uses_actual_second_topic() {
    let mut s = session();
    let before = s.draft.clone();
    let dirty = s.dirty;
    let Reply::Response(P::Problem { message, field }) =
        save_other_taxes(&mut s, vec![lead_tax(), topic(vec![record()]), topic(vec![record()])])
    else {
        panic!("expected duplicate topic refusal")
    };
    assert_eq!(message, libertas::libertas_formatted_text("COVERDELL_TOPIC_TWICE", &[]));
    assert_eq!(field.as_deref(), Some("[value/taxes/2/beneficiaries]"));
    assert_eq!(s.draft, before);
    assert_eq!(s.dirty, dirty);
}
