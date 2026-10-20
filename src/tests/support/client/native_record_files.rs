extern crate std;

use super::lifecycle::{NativeDraftRecords, draft_from_native_records, native_draft_records};
use crate::{FederalDraft, TaxDataKey};
use alloc::collections::BTreeMap;
use libertas::data::KeyDecode;
use std::{fs, path::Path};

// Host fixture organization only: every file contains one unchanged SDK key
// or value. Numeric pairs retain the projection's record order across saves.
pub(in crate::tests) fn write_native_draft_records(
    path: impl AsRef<Path>,
    records: &NativeDraftRecords,
) {
    let path = path.as_ref();
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(path).unwrap(),
        Ok(_) => fs::remove_file(path).unwrap(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => panic!("cannot inspect native record fixture: {error}"),
    }
    fs::create_dir_all(path).unwrap();
    for (index, (key, value)) in records.iter().enumerate() {
        fs::write(path.join(alloc::format!("{index}.key")), key).unwrap();
        fs::write(path.join(alloc::format!("{index}.value")), value).unwrap();
    }
}

pub(in crate::tests) fn read_native_draft_records(path: impl AsRef<Path>) -> NativeDraftRecords {
    let mut pairs = BTreeMap::new();
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        assert!(entry.file_type().unwrap().is_file(), "expected a raw key/value file");
        let name = entry.file_name();
        let name = name.to_str().expect("native record filename is UTF-8");
        let (index, extension) = name.rsplit_once('.').expect("expected numbered key/value file");
        let index: usize = index.parse().expect("native record index is numeric");
        assert_eq!(name, alloc::format!("{index}.{extension}"), "canonical record index");
        let pair = pairs.entry(index).or_insert((None, None));
        let slot = match extension {
            "key" => &mut pair.0,
            "value" => &mut pair.1,
            _ => panic!("expected .key or .value file"),
        };
        assert!(slot.replace(entry.path()).is_none(), "duplicate record file");
    }
    pairs
        .into_iter()
        .enumerate()
        .map(|(expected, (index, (key, value)))| {
            assert_eq!(index, expected, "native record indices must be contiguous");
            (
                fs::read(key.expect("missing native record key")).unwrap(),
                fs::read(value.expect("missing native record value")).unwrap(),
            )
        })
        .collect()
}

pub(in crate::tests) fn write_native_draft(path: impl AsRef<Path>, draft: &FederalDraft) {
    write_native_draft_records(path, &native_draft_records(draft));
}

pub(in crate::tests) fn read_native_draft(path: impl AsRef<Path>) -> FederalDraft {
    let records = read_native_draft_records(path);
    let (key, _) = records.first().expect("accepted return has a header record");
    let TaxDataKey::ReturnRecord { tax_year, .. } = TaxDataKey::decode_key(key).unwrap() else {
        panic!("expected a modern return record key");
    };
    draft_from_native_records(tax_year, &records)
}
