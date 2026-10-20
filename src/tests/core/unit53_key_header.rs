use crate::TaxDataKey;
use alloc::vec::Vec;
use libertas::data::{KeyCodec, KeyDecode, Target};

#[test]
fn semantic_key_has_fixed_field_boundaries_and_full_width_identities() {
    let key = TaxDataKey::ReturnRecord {
        user: 0x0102_0304_0506_0708,
        tax_year: 2026,
        copy: i64::MAX,
        kind: 0x0102,
        id: i64::MAX - 1,
    };
    let bytes = key.to_key_bytes().unwrap();
    assert_eq!(bytes.len(), 32);
    assert_eq!(&bytes[..2], &[0, 2]);
    assert_eq!(&bytes[2..10], &[0x81, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(&bytes[10..14], &[0x80, 0, 7, 0xea]);
    assert_eq!(&bytes[14..22], &[0xff; 8]);
    assert_eq!(&bytes[22..24], &[1, 2]);
    assert_eq!(&bytes[24..], &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe]);
    assert_eq!(TaxDataKey::decode_key(&bytes).unwrap(), key);
    assert!(TaxDataKey::decode_key(&bytes[..31]).is_err());
    assert!(TaxDataKey::decode_key(&[bytes.as_slice(), &[0]].concat()).is_err());
}

#[test]
fn user_year_prefix_includes_copies_and_kinds_but_isolates_other_returns() {
    let user = i64::MAX;
    let prefix = TaxDataKey::return_record_prefix(user, 2026).unwrap();
    assert_eq!(prefix.len(), 14);
    assert!(Target::<TaxDataKey>::prefix(&prefix).is_ok());
    for (owner, year, copy, kind, id, included) in [
        (user, 2026, 0, 1, 0, true),
        (user, 2026, 1, 42, 0, true),
        (user, 2026, i64::MAX, u16::MAX, i64::MAX, true),
        (user - 1, 2026, 0, 1, 0, false),
        (user, 2027, 0, 1, 0, false),
    ] {
        let key = TaxDataKey::ReturnRecord { user: owner, tax_year: year, copy, kind, id };
        assert_eq!(key.to_key_bytes().unwrap().starts_with(&prefix), included);
    }
    for key in [TaxDataKey::StorageFormat { user }, TaxDataKey::Draft { user }] {
        assert!(!key.to_key_bytes().unwrap().starts_with(&prefix));
    }
}

#[test]
fn signed_fields_keep_numeric_order_and_do_not_alias() {
    let keys: Vec<_> = [i64::MIN, -1, 0, 1, u32::MAX as i64 + 1, i64::MAX]
        .into_iter()
        .map(|id| TaxDataKey::ReturnRecord { user: 1, tax_year: 2026, copy: 0, kind: 42, id })
        .collect();
    let bytes: Vec<_> = keys.iter().map(|key| key.to_key_bytes().unwrap()).collect();
    assert!(bytes.windows(2).all(|pair| pair[0] < pair[1]));
    for (key, bytes) in keys.iter().zip(bytes.iter()) {
        assert_eq!(&TaxDataKey::decode_key(bytes).unwrap(), key);
    }
    let years: Vec<_> = [i32::MIN, -1, 0, 2026, i32::MAX]
        .into_iter()
        .map(|year| TaxDataKey::return_record_prefix(1, year).unwrap())
        .collect();
    assert!(years.windows(2).all(|pair| pair[0] < pair[1]));
}
