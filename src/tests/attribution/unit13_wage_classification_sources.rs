use super::*;
use crate::tests::cross::interview::{accepted, open_document_center};

// 2026-10-14 dispatcher42d62e01 corrects the original destination-based
// hypotheses: pure W-2 amounts retain their source on native8t/8u/8s.
// Original12 hypotheses/source/native proof remain in the private dated seal.
// MedicaidCombined's two original source expectations remain hypotheses;
// its mixed50c/50c rounding tie still needs the finite reconciliation below.

macro_rules! cases {
    ($(($name:ident, $case:ident, $owner:ident)),+ $(,)?) => {$(
        #[test]
        fn $name() { check_wage_classification_source(WageClassificationCase::$case, Owner::$owner); }
    )+};
}

cases!(
    (unit13_wage_classification_deferred_taxpayer, Deferred, Taxpayer),
    (unit13_wage_classification_deferred_spouse, Deferred, Spouse),
    (unit13_wage_classification_inmate_taxpayer, Inmate, Taxpayer),
    (unit13_wage_classification_inmate_spouse, Inmate, Spouse),
    (unit13_wage_classification_mixed_taxpayer, Mixed, Taxpayer),
    (unit13_wage_classification_mixed_spouse, Mixed, Spouse),
    (unit13_wage_classification_medicaid_box1_taxpayer, MedicaidBox1, Taxpayer),
    (unit13_wage_classification_medicaid_box1_spouse, MedicaidBox1, Spouse),
    (unit13_wage_classification_medicaid_codeii_taxpayer, MedicaidCodeII, Taxpayer),
    (unit13_wage_classification_medicaid_codeii_spouse, MedicaidCodeII, Spouse),
    (unit13_wage_classification_medicaid_combined_taxpayer, MedicaidCombined, Taxpayer),
    (unit13_wage_classification_medicaid_combined_spouse, MedicaidCombined, Spouse),
);

#[test]
fn unit13_wage_classification_non_code_l_excess_taxpayer() {
    check_non_code_l_wage_source(Owner::Taxpayer);
}

#[test]
fn unit13_wage_classification_non_code_l_excess_spouse() {
    check_non_code_l_wage_source(Owner::Spouse);
}

#[test]
fn unit13_wage_classification_combined_source_rounding_diagnostic() {
    // This captures a finite unresolved classification boundary. It does not
    // replace the two original fixed-source hypotheses above or establish a
    // new tie policy from a conserved total.
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let mut draft =
                    wage_classification_fixture(WageClassificationCase::MedicaidCombined, owner);
                draft.injured_spouse.as_mut().unwrap().injured = injured;
                if reversed {
                    draft.income.reverse();
                }
                let result = supported(&draft);
                assert_wage_classification_money(&result, WageClassificationCase::MedicaidCombined);
                let mut session = Interview::new(TEST_USER, draft.clone());
                crate::tests::support::client::admit_native_fixture_facts(&mut session);
                let source = open_document_center(&mut session);
                accepted(&send(&mut session, source));
                let draft = session.draft.clone();
                let accepted_result = supported(&draft);
                let mut expected = result;
                expected.revision = accepted_result.revision;
                assert_eq!(accepted_result, expected);
                let result = accepted_result;
                let request = open_document_center(&mut session);
                accepted(&send(&mut session, request));
                assert_eq!(session.draft, draft);
                assert_eq!(estimate(&session.draft), result);
                let saved = roundtrip_native_draft(session.draft.clone());
                let restarted = restart(saved);
                assert_eq!(estimate(&restarted.draft), result);
            }
        }
    }
}
