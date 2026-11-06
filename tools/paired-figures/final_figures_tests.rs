use super::*;

#[test]
fn matching_money_hash_avoids_decoding_and_missing_hash_does_not_match() {
    let temp = TempDir::new(&std::env::temp_dir()).unwrap();
    let path = temp.0.join("events.jsonl");
    // Indexing does not need to decode the money value. A deliberately absent
    // value makes a read observable if the matching-hash shortcut regresses.
    std::fs::write(&path, b"{\"kind\":\"Estimate\",\"test\":\"case\",\"index\":0}\n").unwrap();
    let capture = Capture::index(path.to_str().unwrap(), &temp.0, false).unwrap();
    let mut a = capture.reader().unwrap();
    let mut b = capture.reader().unwrap();
    let entry = &capture.tests["case"][0];
    let mut old = entry.clone();
    let mut new = entry.clone();
    old.money_hash = Some([7; 32]);
    new.money_hash = old.money_hash;
    assert!(same_money(&old, &new, &mut a, &mut b).unwrap());
    new.money_hash = Some([8; 32]);
    assert_eq!(
        sequence_verdict(&[&old, &new], &[&new, &old], &mut a, &mut b).unwrap(),
        "diverges at estimate 3 of 2 base / 2 tip"
    );
    new.money_hash = None;
    assert!(!same_money_bytes(&old, &new));
    assert!(same_money(&old, &new, &mut a, &mut b).is_err());
}

#[test]
fn different_money_encodings_use_spaced_json_and_preserve_scalar_types() {
    let temp = TempDir::new(&std::env::temp_dir()).unwrap();
    let path = temp.0.join("events.jsonl");
    let values = [
        r#"{"z":2,"a":1}"#,
        r#"{ "a" : 1, "z" : 2 }"#,
        "1e3",
        "1000.0",
        "1000",
        "true",
        "1",
        "18446744073709551615",
        "18446744073709551614",
    ];
    let rows: String = values
        .iter()
        .enumerate()
        .map(|(index, money)| {
            format!(
                "{{\"kind\":\"Estimate\",\"test\":\"case\",\"index\":{index},\"money\":{money}}}\n"
            )
        })
        .collect();
    std::fs::write(&path, rows).unwrap();
    let capture = Capture::index(path.to_str().unwrap(), &temp.0, false).unwrap();
    let entries = &capture.tests["case"];
    let mut a = capture.reader().unwrap();
    let mut b = capture.reader().unwrap();
    for (old, new, equal) in
        [(0, 1, true), (2, 3, true), (3, 4, false), (5, 6, false), (7, 8, false)]
    {
        assert_eq!(same_money(&entries[old], &entries[new], &mut a, &mut b).unwrap(), equal);
    }
}
