use super::*;
use crate::household_records::{HouseholdRecordError as E, assemble, project};
use libertas::data::{KeyCodec, KeyDecode};

#[test]
fn household_records_reconstruct_exact_facts_and_accepted_order_after_wire_roundtrip() {
    let mut original = complete(FederalStatus::Joint);
    original.next_id = i64::MAX - 1;
    original.revision = 83;
    original.cookie = "synthetic-household".into();
    let mut parent = ordinary_parent();
    parent.complete = false; // Unknown family facts stay unknown.
    parent.taxable = 15_000_037;
    parent.foreign_excluded = Some(0); // Reported zero stays distinct from absence.
    parent.gains = Some(ParentGains {
        qualified: 471,
        long_term: -239,
        combined: -637,
        collectibles: 0,
        unrecaptured: 0,
        investment_election: 0,
    });
    let people = original.people.as_mut().unwrap();
    people.parent_tax = Some(parent);
    people.spouse.as_mut().unwrap().first_name = "Second".into();
    people.taxpayer.died = death(20260615);
    people.taxpayer.died.as_mut().unwrap().estate_medical = 13_579;
    original.dependents = vec![child(i64::MAX - 2), child(u32::MAX as i64 + 9)];
    original.dependents[0].care = DependentCare::Yes { care_expenses: 10_037 };
    original.dependents[1].situation =
        DependentSituation::Supported { your_support_over_half: false, gross_income: 14_099 };
    // Equal labels must never establish identity or determine accepted order.
    original.dependents[1].last_name = original.dependents[0].last_name.clone();
    let mut records: Vec<_> = project(501, 7, &original)
        .unwrap()
        .into_iter()
        .map(|(key, value)| {
            (
                TaxDataKey::decode_key(&key.to_key_bytes().unwrap()).unwrap(),
                TaxAppData::from_avro(&value.to_avro()).unwrap(),
            )
        })
        .collect();
    records.sort_by_key(|(key, _)| key.to_key_bytes().unwrap());
    // Key order reverses these dependent IDs; explicit order must restore them.
    let mut restored = original.clone();
    restored.people = None;
    restored.dependents.clear();
    assemble(501, 7, &mut restored, records).unwrap();
    assert_eq!(restored, original); // Also protects all 21 out-of-scope fields.
}

#[test]
fn absence_spouse_discriminator_and_present_empty_dependents_remain_distinct() {
    for people_state in 0..3 {
        for children_present in [false, true] {
            let mut original = complete(FederalStatus::Joint);
            match people_state {
                0 => original.people = None,
                1 => original.people.as_mut().unwrap().spouse = SpouseFiler::No,
                _ => {}
            }
            original.next_id = 5;
            if children_present {
                original.dependents.push(child(4));
            }
            let records = project(1, 0, &original).unwrap();
            assert_eq!(records.len(), 1 + people_state + usize::from(children_present));
            let mut restored = complete(FederalStatus::Single);
            restored.next_id = original.next_id;
            let mut expected = restored.clone();
            expected.people = original.people;
            expected.dependents = original.dependents;
            assemble(1, 0, &mut restored, records).unwrap();
            assert_eq!(restored, expected);
        }
    }
}

#[test]
fn corrupt_household_records_never_partially_replace_the_destination() {
    let mut draft = complete(FederalStatus::Joint);
    draft.dependents = vec![child(2), child(3)];
    draft.next_id = 4;
    let records = project(1, 0, &draft).unwrap();
    for case in 0..21 {
        let mut broken = records.clone();
        let expected = match case {
            0 => {
                broken.remove(0);
                E::MissingRecord
            }
            1 => {
                broken.remove(1);
                E::MissingRecord
            }
            2 => {
                broken.remove(2);
                E::MissingRecord
            }
            3 => {
                broken.pop();
                E::MissingRecord
            }
            4 => {
                broken.push(broken[0].clone());
                E::DuplicateRecord
            }
            5 => {
                broken.push(broken[1].clone());
                E::DuplicateRecord
            }
            6 => {
                broken.push(broken[3].clone());
                E::DuplicateRecord
            }
            7 | 8 => {
                let TaxAppData::Dependent { position, .. } = &mut broken[4].1 else {
                    unreachable!()
                };
                *position = if case == 7 { 0 } else { 3 };
                E::InvalidOrder
            }
            9 => {
                let TaxAppData::Dependent { facts, .. } = &mut broken[3].1 else { unreachable!() };
                facts.id = 3;
                E::InvalidIdentity
            }
            10..=12 => {
                let TaxDataKey::ReturnRecord { user, tax_year, copy, .. } = &mut broken[0].0 else {
                    unreachable!()
                };
                match case {
                    10 => *user = 2,
                    11 => *tax_year = 2027,
                    _ => *copy = 1,
                }
                E::WrongNamespace
            }
            13 => {
                let TaxDataKey::ReturnRecord { id, .. } = &mut broken[1].0 else { unreachable!() };
                *id = 2;
                E::UnexpectedRecord
            }
            14 => {
                let TaxAppData::Household { record } = &mut broken[0].1 else { unreachable!() };
                record.people = None;
                E::UnexpectedRecord
            }
            15 => {
                let TaxAppData::Household { record } = &mut broken[0].1 else { unreachable!() };
                record.people.as_mut().unwrap().spouse_present = false;
                E::UnexpectedRecord
            }
            16 => {
                broken[0].1 = TaxAppData::StorageFormat { format: STORAGE_FORMAT };
                E::UnexpectedRecord
            }
            18 => {
                let TaxAppData::Dependent { position, .. } = &mut broken[3].1 else {
                    unreachable!()
                };
                *position = -1;
                E::InvalidOrder
            }
            19 => {
                let TaxAppData::Household { record } = &mut broken[0].1 else { unreachable!() };
                record.dependent_count = -1;
                E::MissingRecord
            }
            20 => {
                let TaxDataKey::ReturnRecord { id, .. } = &mut broken[3].0 else { unreachable!() };
                *id = 4;
                let TaxAppData::Dependent { facts, .. } = &mut broken[3].1 else { unreachable!() };
                facts.id = 4;
                E::InvalidIdentity
            }
            _ => {
                let TaxDataKey::ReturnRecord { kind, .. } = &mut broken[0].0 else {
                    unreachable!()
                };
                *kind = 99;
                E::UnexpectedRecord
            }
        };
        let mut destination = draft.clone();
        assert_eq!(assemble(1, 0, &mut destination, broken), Err(expected), "case {case}");
        assert_eq!(destination, draft, "case {case}");
    }
}

#[test]
fn projection_rejects_reused_nonpositive_and_unallocated_native_ids() {
    let mut draft = complete(FederalStatus::Single);
    draft.next_id = 5;
    for ids in [[0, 2], [-1, 2], [2, 2], [2, 5]] {
        draft.dependents = ids.into_iter().map(child).collect();
        assert_eq!(project(1, 0, &draft), Err(E::InvalidIdentity));
    }
    draft.dependents.clear();
    for (user, copy, next_id) in [(0, 0, 5), (1, -1, 5), (1, 0, 0), (1, 0, i64::MAX)] {
        draft.next_id = next_id;
        assert_eq!(project(user, copy, &draft), Err(E::InvalidIdentity));
        assert_eq!(assemble(user, copy, &mut draft, vec![]), Err(E::InvalidIdentity));
    }
}
