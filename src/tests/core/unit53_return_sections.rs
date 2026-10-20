extern crate std;

use super::*;
use crate::return_sections::SectionProjectionError as E;
use libertas::data::{KeyCodec, KeyDecode, wire};

#[test]
fn setup_checklist_roundtrip_preserves_absence_bytes_and_other_twenty_fields() {
    let mut source = complete(FederalStatus::Joint);
    source.tax_year = 2030;
    source.checklist = Checklist {
        income: u32::MAX,
        adjustments: 3,
        deductions: 5,
        credit_topics: 7,
        other_taxes: 11,
        payments: 13,
        whole_return: 17,
    };
    for label in [None, Some(vec![]), Some(vec![0, 255, 128, 1])] {
        let mut source = source.clone();
        match label {
            None => source.setup = None,
            Some(label) => source.setup.as_mut().unwrap().label = label,
        }
        let mut records = source.disassemble_setup_sections(i64::MAX, i64::MAX).unwrap();
        for (key, value) in &mut records {
            *key = TaxDataKey::decode_key(&key.to_key_bytes().unwrap()).unwrap();
            *value = TaxAppData::from_avro(&value.to_avro()).unwrap();
        }
        records.reverse(); // Scan ordering is not part of assembly's contract.
        let mut destination = complete(FederalStatus::Single);
        destination.tax_year = source.tax_year;
        destination.cookie = "different destination".into();
        destination.revision = 42;
        destination.finished = true;
        destination.finished_sections.reverse();
        let mut expected = destination.clone();
        expected.setup = source.setup.clone();
        expected.checklist = source.checklist;
        destination.assemble_setup_sections(i64::MAX, i64::MAX, &records).unwrap();
        assert_eq!(destination, expected, "all20 unrelated fields stay byte-for-byte equal");
        assert_eq!(
            destination.disassemble_setup_sections(i64::MAX, i64::MAX).unwrap(),
            source.disassemble_setup_sections(i64::MAX, i64::MAX).unwrap()
        );
    }
}

#[test]
fn malformed_complete_record_sets_never_modify_destination() {
    let source = complete(FederalStatus::Joint);
    let valid = source.disassemble_setup_sections(TEST_USER, 0).unwrap();
    let mut cases = vec![vec![], vec![valid[0].clone()], vec![valid[1].clone()]];
    cases.push(vec![valid[0].clone(), valid[0].clone()]);
    cases.push(vec![valid[1].clone(), valid[1].clone()]);
    cases.push(vec![valid[0].clone(), valid[1].clone(), valid[0].clone()]);
    let mut wrong_payload = valid.clone();
    wrong_payload[1].1 = TaxAppData::StorageFormat { format: STORAGE_FORMAT };
    cases.push(wrong_payload);
    let mut swapped = valid.clone();
    swapped[0].1 = valid[1].1.clone();
    swapped[1].1 = valid[0].1.clone();
    cases.push(swapped);
    for (user, tax_year, copy, kind, id) in [
        (TEST_USER + 1, source.tax_year, 0, 6, 0),
        (TEST_USER, source.tax_year + 1, 0, 6, 0),
        (TEST_USER, source.tax_year, 1, 6, 0),
        (TEST_USER, source.tax_year, 0, 5, 0),
        (TEST_USER, source.tax_year, 0, 7, 0),
        (TEST_USER, source.tax_year, 0, 6, 1),
    ] {
        let mut records = valid.clone();
        records[1].0 = TaxDataKey::ReturnRecord { user, tax_year, copy, kind, id };
        cases.push(records);
    }
    let mut legacy_key = valid.clone();
    legacy_key[1].0 = TaxDataKey::Draft { user: TEST_USER };
    cases.push(legacy_key);
    for records in cases {
        let mut destination = complete(FederalStatus::Single);
        let before = destination.clone();
        assert!(destination.assemble_setup_sections(TEST_USER, 0, &records).is_err());
        assert_eq!(destination, before);
    }
    for (user, copy) in [(0, 0), (-1, 0), (TEST_USER, -1)] {
        let mut destination = source.clone();
        assert_eq!(source.disassemble_setup_sections(user, copy), Err(E::Namespace));
        assert_eq!(destination.assemble_setup_sections(user, copy, &valid), Err(E::Namespace));
        assert_eq!(destination, source);
    }
}

#[test]
fn native_accepted_setup_and_checklist_have_small_real_encoded_values() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let request = P::Navigate {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        from: 0,
        section: TaxSection::Setup.into(),
        navigation: NavigationAvailability::default(),
    };
    let Reply::Edit(mut page @ P::SaveSetup { .. }) = send(&mut session, request) else {
        panic!("expected native setup editor");
    };
    let P::SaveSetup { value, .. } = &mut page else { unreachable!() };
    value.filing = Filing::MarriedJointly;
    accepted(&send(&mut session, page));
    let expected = Message::new("LABEL_RETURN", vec![MessageArg::Number(2026)]).encode();
    assert_eq!(session.draft.setup.as_ref().unwrap().label, expected);
    assert!(session.draft.valid_stored());
    let records = session.draft.disassemble_setup_sections(TEST_USER, 0).unwrap();
    let sizes: Vec<_> = records.iter().map(|(_, value)| value.to_avro().len()).collect();
    // Native Save generates the label and keeps only finite setup facts.
    // This measures that admitted path, not arbitrary synthetic label bytes.
    assert!(sizes.iter().all(|size| *size < 1024));
    assert!(sizes.iter().all(|size| *size <= wire::MAX_VALUE_BYTES));
    std::println!("native setup/checklist Avro sizes: {sizes:?}");
}
