use super::*;
use crate::tests::support::assert::injured_credit::elective_credit_owner_sources::ordinary;
use crate::tests::support::builders::injured_credit::elective_credit_owner_sources::fixture;

#[test]
fn unit13_elective_credit_actual_business_owner_native_save_restart() {
    let mut observations = vec![];
    for owner in [Owner::Taxpayer, Owner::Spouse] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                let mut session = restart(fixture(owner, injured, reverse));
                let ids: Vec<_> = session.draft.income.iter().map(|row| row.id).collect();
                for _ in 0..2 {
                    save_native_credit_sources(
                        &mut session,
                        &[TaxSection::Documents, TaxSection::SelfEmployment],
                    );
                    let r = ordinary(&session.draft);
                    assert_eq!(
                        session.draft.income.iter().map(|row| row.id).collect::<Vec<_>>(),
                        ids
                    );
                    let draft = roundtrip_native_draft(session.draft.clone());
                    observations.push((owner, injured, reverse, r));
                    session = restart(draft);
                    ordinary(&session.draft);
                }
            }
        }
    }
    // Every actual source Save, ordinary financial oracle, source identity and
    // full persistence/restart check above passes before owner assertions.
    for (owner, injured, reverse, r) in observations {
        let expected = if owner == injured { [5000, 0] } else { [0, 5000] };
        assert_eq!(
            [column(&r, "17", "b"), column(&r, "17", "c")],
            expected,
            "business owner {owner:?}, injured {injured:?}, reverse {reverse}"
        );
    }
}
