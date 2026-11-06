use super::*;
use crate::tests::cross::interview::{accepted, open_topic};
extern crate std;
use crate::tests::support::assert::injured_income::ltc_cent_sources::ordinary;
use crate::tests::support::builders::injured_income::ltc_cent_sources::fixture;
use crate::tests::support::client::injured_income::ltc_cent_sources::saved;

#[test]
fn unit13_ltc_cent_shared_periods_do_not_invent_a_taxable_owner() {
    let mut observations = Vec::new();
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            let mut draft = fixture(true);
            draft.injured_spouse.as_mut().unwrap().injured = injured;
            if reverse {
                draft.income.reverse();
            }
            let (_, result) = saved(draft);
            let owners = ["b", "c"].map(|column| {
                let key = alloc::format!("form_8379_line_13b_{column}");
                result.lines.iter().find(|line| line.id == key).map(|line| line.amount)
            });
            std::println!(
                "LTC shared sub-Dollar source: injured={injured:?} reverse={reverse} owners={owners:?}"
            );
            observations.push(owners);
        }
    }
    assert!(
        observations.iter().all(|owners| *owners == [None, None]),
        "unknown shared taxable owners must remain unavailable after all4 native money/Save/restart profiles: {observations:?}"
    );
}

#[test]
fn unit13_ltc_cent_exclusive_periods_keep_the_native_annual_owner() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            let mut draft = fixture(false);
            draft.injured_spouse.as_mut().unwrap().injured = injured;
            if reverse {
                draft.income.reverse();
            }
            let (_, result) = saved(draft);
            assert_eq!(
                [column(&result, "13b", "b"), column(&result, "13b", "c")],
                if injured == Owner::Taxpayer { [1, 0] } else { [0, 1] }
            );
        }
    }
}

#[test]
fn unit13_ltc_cent_explicit_bounded_answers_round_only_at_annual_boundary() {
    use crate::tests::cross::interview::from_review;
    for taxpayer in [[0, 0], [49, 49], [24, 25]] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                let mut draft = fixture(true);
                draft.injured_spouse.as_mut().unwrap().injured = injured;
                if reverse {
                    draft.income.reverse();
                }
                let before = supported(&draft);
                ordinary(&before);
                let mut session = restart(draft);
                let choices = taxpayer
                    .map(|amount| if injured == Owner::Taxpayer { amount } else { 49 - amount });
                let Reply::Edit(page) = crate::tests::support::client::native_editor(
                    from_review(&mut session, TaxSection::InjuredSpouse),
                    &[("ltc_3_period_1", choices[0]), ("ltc_3_period_2", choices[1])],
                ) else {
                    panic!("normal native WorkflowRequest must expose both actual shared periods")
                };
                accepted(&send(&mut session, page));
                let (draft, result) = saved(restart(session.draft).draft);
                let plan = result.injured_spouse.as_ref().unwrap();
                for (period, answer) in [1, 2].into_iter().zip(choices) {
                    let key = alloc::format!("ltc_3_period_{period}");
                    let row = plan.required.iter().find(|row| row.component == key).unwrap();
                    assert_eq!(
                        (row.joint, row.minimum, row.maximum, row.injured),
                        (49, 0, 49, answer)
                    );
                    assert_eq!(plan.accepted.iter().filter(|row| row.component == key).count(), 1);
                    assert_eq!(
                        draft
                            .injured_spouse
                            .as_ref()
                            .unwrap()
                            .allocations
                            .iter()
                            .find(|row| row.component == key)
                            .unwrap()
                            .injured,
                        answer
                    );
                }
                let columns = [column(&result, "13b", "b"), column(&result, "13b", "c")];
                let expected = match taxpayer {
                    [0, 0] if injured == Owner::Spouse => [1, 0],
                    [0, 0] => [0, 1],
                    [49, 49] if injured == Owner::Taxpayer => [1, 0],
                    [49, 49] => [0, 1],
                    // Actual49c/49c owner sums: the selected owner rounds to0,
                    // and the other keeps the existing native annual$1 residue.
                    [24, 25] => [0, 1],
                    _ => unreachable!(),
                };
                assert_eq!(columns, expected);
                let mut reopened = restart(draft);
                let Reply::Edit(page) = crate::tests::support::client::native_editor(
                    from_review(&mut reopened, TaxSection::InjuredSpouse),
                    &[],
                ) else {
                    panic!()
                };
                accepted(&send(&mut reopened, page));
                ordinary(&supported(&reopened.draft));
            }
        }
    }
}

#[test]
fn unit13_ltc_cent_invalid_or_duplicate_answers_never_commit() {
    use crate::tests::cross::interview::from_review;
    for invalid in [-1, 50, 99] {
        let mut session = restart(fixture(true));
        let Reply::Edit(page) = crate::tests::support::client::native_editor(
            from_review(&mut session, TaxSection::InjuredSpouse),
            &[("ltc_3_period_1", invalid), ("ltc_3_period_2", 0)],
        ) else {
            panic!()
        };
        let before = session.draft.clone();
        let dirty = session.dirty;
        let reply = send(&mut session, page);
        assert!(!matches!(reply, Reply::Response(P::Review { .. })));
        assert_eq!(session.draft, before);
        assert_eq!(session.dirty, dirty);
        ordinary(&supported(&session.draft));
    }
    let mut session = restart(fixture(true));
    let Reply::Edit(mut page) = crate::tests::support::client::native_editor(
        from_review(&mut session, TaxSection::InjuredSpouse),
        &[("ltc_3_period_1", 0), ("ltc_3_period_2", 0)],
    ) else {
        panic!()
    };
    let P::SaveInjuredSpouse { value, .. } = &mut page else { panic!() };
    value.new_allocations.push(value.allocations[0].clone());
    let before = session.draft.clone();
    let dirty = session.dirty;
    assert!(!matches!(send(&mut session, page), Reply::Response(P::Review { .. })));
    assert_eq!(session.draft, before);
    assert_eq!(session.dirty, dirty);
    ordinary(&supported(&session.draft));
}

#[test]
fn unit13_ltc_cent_closed_annual_zero_needs_no_owner_choice() {
    use crate::tests::cross::interview::from_review;
    let mut observations = Vec::new();
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            let mut draft = fixture(true);
            draft.injured_spouse.as_mut().unwrap().injured = injured;
            for entry in &mut draft.income {
                match &mut entry.income {
                    FederalIncome::LtcInsured { data } => {
                        data.periods = LtcPeriods::One {
                            starts: 20260601,
                            ends: 20260601,
                            services: 0,
                            reimbursements: 0,
                            recipient: CareBenefitRecipient::Insured { other_payees: false },
                        };
                    }
                    FederalIncome::LtcBenefits { data } => {
                        data.form.benefits = Some(if entry.owner == crate::DocumentOwner::Yours {
                            21_525
                        } else {
                            21_524
                        });
                    }
                    _ => {}
                }
            }
            if reverse {
                draft.income.reverse();
            }
            let mut session = restart(draft);
            for _ in 0..2 {
                let page = open_topic(&mut session, TaxSection::OtherIncome);
                accepted(&send(&mut session, page));
                let result = supported(&session.draft);
                assert_eq!(value(&result, "agi"), 30_000);
                assert_eq!(value(&result, "taxable_income"), 0);
                assert_eq!(
                    (result.tax, result.refund, result.balance),
                    (Some(0), Some(10_000), Some(0))
                );
                assert_eq!(value(&result, "taxable_ltc_benefits"), 0);
                assert_eq!(line_amount(&result, "ltc_benefit_3_1_taxable"), 49);
                session = restart(session.draft);
                assert_eq!(estimate(&session.draft), result);
            }
            let result = supported(&session.draft);
            let columns = ["b", "c"].map(|column| {
                result
                    .lines
                    .iter()
                    .find(|row| row.id == alloc::format!("form_8379_line_13b_{column}"))
                    .map(|row| row.amount)
            });
            let Reply::Response(P::BeginInjuredSpouse { value, .. }) =
                from_review(&mut session, TaxSection::InjuredSpouse)
            else {
                panic!()
            };
            let choices = value
                .allocations
                .iter()
                .map(|row| &row.component)
                .chain(value.new_allocations.iter().map(|row| &row.component))
                .filter(|key| key.starts_with("ltc_"))
                .count();
            std::println!(
                "LTC closed annual zero: injured={injured:?} reverse={reverse} choices={choices} columns={columns:?}"
            );
            observations.push((choices, columns));
        }
    }
    assert!(
        observations.iter().all(|row| *row == (0, [Some(0), Some(0)])),
        "closed nonnegative49c/native0 has unique annual0/0 owners and needs no question, after all4 native money/Save/restart profiles: {observations:?}"
    );
}
