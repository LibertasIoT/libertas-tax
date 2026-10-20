use super::*;
use crate::tests::cross::interview::{accepted, from_review, open_topic};
use crate::tests::support::builders::injured_credit::fuel_credit_owner_sources::fixture;
use crate::tests::support::client::injured_credit::fuel_credit_owner_sources::{
    assert_owner_symmetry, saved,
};

#[test]
fn unit13_fuel_credit_sources_zero_business_pool_has_zero_owners() {
    let mut observations = Vec::new();
    for business in [[50, 49], [49, 50]] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                let result = saved(fixture(business, injured, reverse), business, 149, 1);
                let actual = [column(&result, "17", "b"), column(&result, "17", "c")];
                let expected = if business == [49, 50] || injured == Owner::Taxpayer {
                    [1, 0]
                } else {
                    [0, 1]
                };

                observations.push((actual, expected));
            }
        }
    }
    assert!(
        observations.iter().all(|(actual, expected)| actual == expected),
        "known native business0 cannot manufacture owner offsets after all8 native money/public Save/restart profiles: {observations:?}"
    );
}

#[test]
fn unit13_fuel_credit_sources_positive_business_pool_cannot_be_negative() {
    let mut observations = Vec::new();
    for business in [[150, 1], [1, 150]] {
        for injured in [Owner::Taxpayer, Owner::Spouse] {
            for reverse in [false, true] {
                let result = saved(fixture(business, injured, reverse), business, 201, 2);
                let actual = [column(&result, "17", "b"), column(&result, "17", "c")];
                let expected = if business == [150, 1] {
                    if injured == Owner::Taxpayer { [2, 0] } else { [0, 2] }
                } else if injured == Owner::Taxpayer {
                    [1, 1]
                } else {
                    [2, 0]
                };

                observations.push((actual, expected));
            }
        }
    }
    assert!(
        observations.iter().all(|(actual, expected)| actual == expected),
        "native business1 retains nonnegative source credit owners after all8 native money/public Save/restart profiles: {observations:?}"
    );
}

#[test]
fn unit13_fuel_credit_sources_one_whole_category_rounds_once() {
    // 2026-10-15: supersedes only the preserved94 new hypothesis.
    // Actual combined owner sources100c/50c form native joint$2.
    assert_owner_symmetry(
        [50, 50],
        150,
        2,
        [1, 1],
        "source credit projection must retain existing injured-first native-Dollar residue after all4 native money/public Save/restart profiles",
    );
}

#[test]
fn unit13_fuel_credit_sources_whole_category_injured_first_symmetry() {
    // Actual home50c belongs to TP; actual business50c belongs to SP.
    // The native whole category$1 rounds the selected injured owner
    // first and assigns the exact remaining Dollar to the other owner.
    assert_owner_symmetry([0, 50], 100, 1, [1, 0], "whole native category injured-first residue");
}

#[test]
fn unit13_fuel_credit_sources_closed_annual_zero_needs_no_home_choice() {
    for injured in [Owner::Taxpayer, Owner::Spouse] {
        for reverse in [false, true] {
            let mut draft = fixture([0, 0], injured, reverse);
            draft.injured_spouse.as_mut().unwrap().allocations.clear();
            draft
                .payments
                .as_mut()
                .unwrap()
                .home_kerosene
                .as_mut()
                .unwrap()
                .thousandths_of_gallon = 2016;
            let mut session = restart(draft);
            for _ in 0..2 {
                for topic in [TaxSection::Wages, TaxSection::PassThrough] {
                    let page = open_topic(&mut session, topic);
                    accepted(&send(&mut session, page));
                }
                let Reply::Edit(page @ P::SaveMoreCredits { .. }) =
                    from_review(&mut session, TaxSection::MoreCredits)
                else {
                    panic!()
                };
                accepted(&send(&mut session, page));
                let result = supported(&session.draft);
                assert_eq!(value(&result, "agi"), 30_000);
                assert_eq!(value(&result, "taxable_income"), 0);
                assert_eq!(
                    (result.tax, result.refund, result.payments, result.balance),
                    (Some(0), Some(10_000), Some(10_000), Some(0))
                );
                // 2026-10-15: native zero calculation is intentionally omitted.
                // The mandatory1040 line31 remains the independent zero oracle.
                assert_eq!(return_line(&result, "31"), 0);
                assert!(
                    result
                        .lines
                        .iter()
                        .find(|row| row.id == "schedule_3_fuel_tax_credit")
                        .is_none_or(|row| row.amount == 0)
                );
                assert_eq!(line_amount(&result, "fuel_tax_credit"), 49);
                assert_eq!(line_amount(&result, "home_kerosene_fuel_4a"), 49);
                assert_eq!(
                    [
                        column(&result, "17", "a"),
                        column(&result, "17", "b"),
                        column(&result, "17", "c")
                    ],
                    [0; 3]
                );
                assert!(
                    result
                        .injured_spouse
                        .as_ref()
                        .unwrap()
                        .required
                        .iter()
                        .all(|row| row.component != "home_fuel_tax_credit")
                );
                session = restart(session.draft);
                assert_eq!(estimate(&session.draft), result);
            }
        }
    }
}

#[test]
fn unit13_fuel_credit_sources_missing_or_unfooted_packet_stays_unknown() {
    use crate::injured_spouse::income_categories::NativeFuelCreditSources;
    for case in 0..4 {
        let draft = injured_return((30_000, 100), (0, 0), 0, allocation());
        let mut plain = draft.clone();
        plain.injured_spouse = None;
        let mut result = supported(&plain);
        assert_eq!(
            (value(&result, "agi"), result.tax, result.refund),
            (30_000, Some(0), Some(10_000))
        );
        assert_eq!(return_line(&result, "31"), 0);
        let mut facts = liability_portfolio_native_facts(&draft, Some([0; 2]), Some(&result));
        facts.income_categories.fuel_credit_sources = match case {
            0 => None,
            1 => Some(alloc::boxed::Box::new(NativeFuelCreditSources {
                joint_cents: 0,
                business_cents: [1, 0],
                home_cents: 0,
            })),
            2 => Some(alloc::boxed::Box::new(NativeFuelCreditSources {
                joint_cents: 0,
                business_cents: [-1, 1],
                home_cents: 0,
            })),
            _ => Some(alloc::boxed::Box::new(NativeFuelCreditSources {
                joint_cents: 0,
                business_cents: [0; 2],
                home_cents: 0,
            })),
        };
        let overpayment = result.refund.expect("ordinary pre-election overpayment");
        crate::injured_spouse::allocate(
            &draft,
            &mut result,
            draft.injured_spouse.as_ref().unwrap(),
            &facts,
            overpayment,
        );
        assert_eq!(
            (value(&result, "agi"), result.tax, result.refund),
            (30_000, Some(0), Some(10_000))
        );
        for column in ["b", "c"] {
            let actual = result
                .lines
                .iter()
                .find(|row| row.id == alloc::format!("form_8379_line_17_{column}"))
                .map(|row| row.amount);
            assert_eq!(
                actual,
                (case == 3).then_some(0),
                "packet case{case} column{column}: absence/raw mismatch/negative source must not become closed native zero"
            );
        }
    }
}
