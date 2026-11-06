use super::*;
use crate::tests::cross::interview::accepted;
use crate::tests::support::client::injured_credit::original_map_boundaries::open_r9;

#[test]
fn unit13_original_r9_farmland_deferral_preserves_money_sources_and_saved_claim() {
    let original = farmland_installment_return();
    let original_result = supported(&original);
    assert_eq!(value(&original_result, "agi"), 192935);
    assert_eq!(value(&original_result, "regular_income_tax"), 26978);
    assert_eq!(original_result.tax, Some(4110800));
    for farm_owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reversed in [false, true] {
                let mut draft = original.clone();
                let joint = complete(FederalStatus::Joint);
                draft.people = joint.people;
                draft.setup = joint.setup;
                let FederalIncome::Wage { data } = &mut draft.income[0].income else { panic!() };
                edit_wage(data).withholding = 5_000_000;
                draft.income[1].owner = crate::DocumentOwner::from(farm_owner);
                let farm_id = draft.income[1].id;
                add(&mut draft, wage(0, Owner::Spouse, 2_000_000, 100_000));
                if reversed {
                    draft.income.reverse();
                }
                let plain = supported(&draft);
                assert_eq!(value(&plain, "agi"), 212935);
                assert_eq!(value(&plain, "regular_income_tax"), 22297);
                assert_eq!(plain.tax, Some(3642700));
                assert_eq!(plain.farmland_installments.as_ref().unwrap().applicable, 600000);
                let source_income = draft.income.clone();
                let mut claim = allocation();
                claim.injured = injured;
                draft.set_situations(SpouseSituations::INJURED_SPOUSE.0, true);
                draft.injured_spouse = Some(claim);
                let claimed = supported(&draft);
                assert_eq!(
                    (claimed.tax, claimed.refund, claimed.balance, claimed.payments),
                    (plain.tax, plain.refund, plain.balance, plain.payments)
                );
                assert!(
                    claimed
                        .review_notes
                        .iter()
                        .any(|note| note.display.key == "WARN_8379_FARMLAND_ESTIMATE_UNAVAILABLE")
                );
                assert!(claimed.injured_spouse.as_ref().unwrap().required.is_empty());
                let mut session = restart(draft);
                let page = open_r9(&mut session);
                accepted(&send(&mut session, roundtrip_native_request(page)));
                assert_eq!(session.draft.income, source_income);
                let result = supported(&session.draft);
                assert_eq!(
                    (result.tax, result.refund, result.balance, result.payments),
                    (plain.tax, plain.refund, plain.balance, plain.payments)
                );
                assert_eq!(result.farmland_installments, plain.farmland_installments);
                assert_eq!(column(&result, "17", "a"), 6000);
                assert!(!result.lines.iter().any(|line| matches!(
                    line.id.as_str(),
                    "form_8379_line_17_b" | "form_8379_line_17_c"
                )));
                assert!(
                    !result
                        .review_notes
                        .iter()
                        .any(|note| note.display.key == "WARN_8379_ALLOCATION_CHOICES_NEEDED")
                );
                assert!(
                    result
                        .review_notes
                        .iter()
                        .any(|note| note.display.key == "WARN_8379_FARMLAND_ESTIMATE_UNAVAILABLE")
                );
                assert!(
                    !result
                        .lines
                        .iter()
                        .any(|line| line.id == crate::injured_spouse::ESTIMATE_LINE)
                );
                let expected = if farm_owner == injured { [140000, 0] } else { [0, 140000] };
                assert_eq!([column(&result, "13b", "b"), column(&result, "13b", "c")], expected);
                assert_eq!([column(&result, "20", "b"), column(&result, "20", "c")], [0, 0]);
                let draft = roundtrip_native_draft(session.draft.clone());
                let mut reopened = restart(draft);
                let page = open_r9(&mut reopened);
                accepted(&send(&mut reopened, page));
                assert_eq!(reopened.draft.income, source_income);
                assert!(reopened.draft.income.iter().any(|row| row.id == farm_id));
                assert_eq!(supported(&reopened.draft), result);
            }
        }
    }
}
