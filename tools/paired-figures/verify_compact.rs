//! Verify compact artifact parity without materializing diagnostic audit arrays.
use crate::{Args, Result, capture::Input, fail};
use serde::Deserialize;
use serde_json::{Value, value::RawValue};
use std::{
    io::BufRead,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
struct Pair<'a> {
    test: String,
    index: u64,
    old_kind: String,
    new_kind: String,
    kind_mismatch: bool,
    identical_debug: bool,
    money_keys_compared: u64,
    #[serde(borrow)]
    money_changes: &'a RawValue,
    #[serde(borrow)]
    declared_reason: &'a RawValue,
    #[serde(borrow)]
    numeric_audit: &'a RawValue,
    #[serde(borrow)]
    currency_changes: &'a RawValue,
    #[serde(borrow)]
    structural_changes: &'a RawValue,
}
fn array_nonempty(raw: &RawValue) -> Result<bool> {
    let text = raw.get().trim();
    if !text.starts_with('[') {
        return fail("audit and money changes must be available arrays");
    }
    // RawValue has already validated the entire JSON value, including its tail.
    Ok(text[1..text.len() - 1].bytes().any(|byte| !byte.is_ascii_whitespace()))
}
fn equal_value(a: &RawValue, b: &RawValue) -> Result<bool> {
    Ok(a.get() == b.get()
        || serde_json::from_str::<Value>(a.get())? == serde_json::from_str::<Value>(b.get())?)
}
fn compare_pair(a: &[u8], b: &[u8]) -> Result<()> {
    let a: Pair<'_> = serde_json::from_slice(a)?;
    let b: Pair<'_> = serde_json::from_slice(b)?;
    if a.test != b.test
        || a.index != b.index
        || a.old_kind != b.old_kind
        || a.new_kind != b.new_kind
        || a.kind_mismatch != b.kind_mismatch
        || a.identical_debug != b.identical_debug
        || a.money_keys_compared != b.money_keys_compared
        || !equal_value(a.money_changes, b.money_changes)?
        || !equal_value(a.declared_reason, b.declared_reason)?
    {
        return fail(format!("pair fields differ: {}#{}", a.test, a.index));
    }
    array_nonempty(a.money_changes)?;
    array_nonempty(b.money_changes)?;
    for (name, old, new) in [
        ("numeric_audit", a.numeric_audit, b.numeric_audit),
        ("currency_changes", a.currency_changes, b.currency_changes),
        ("structural_changes", a.structural_changes, b.structural_changes),
    ] {
        if array_nonempty(old)? != array_nonempty(new)? {
            return fail(format!("{name} verdict differs: {}#{}", a.test, a.index));
        }
    }
    Ok(())
}
fn pair_path(dir: &Path) -> Result<PathBuf> {
    let paths: Vec<_> = ["result.pairs.jsonl.gz", "result.pairs.jsonl.zst", "result.pairs.jsonl"]
        .into_iter()
        .map(|name| dir.join(name))
        .filter(|path| path.exists())
        .collect();
    if paths.len() != 1 {
        return fail(format!("expected exactly one pairs container in {}", dir.display()));
    }
    Ok(paths.into_iter().next().unwrap())
}
// Metadata is small enough for deep value comparison. Preserve every field and
// reject duplicate object keys rather than silently accepting the last spelling.
fn metadata(path: &Path) -> Result<Value> {
    fn validate(raw: &RawValue) -> Result<()> {
        match raw.get().as_bytes()[0] {
            b'{' => {
                let fields: crate::capture::RawFields<'_> = serde_json::from_str(raw.get())?;
                if !fields.duplicates.is_empty() {
                    return fail("duplicate metadata key");
                }
                for raw in fields.fields.values() {
                    validate(raw)?;
                }
            }
            b'[' => {
                for raw in serde_json::from_str::<Vec<&RawValue>>(raw.get())? {
                    validate(raw)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let bytes = std::fs::read(path)?;
    let raw: &RawValue = serde_json::from_slice(&bytes)?;
    validate(raw)?;
    Ok(serde_json::from_slice(&bytes)?)
}
fn verify(reference: &Path, candidate: &Path) -> Result<u64> {
    for name in ["result.summary.json", "final.json", "final-deltas.json"] {
        if metadata(&reference.join(name))? != metadata(&candidate.join(name))? {
            return fail(format!("{name} differs"));
        }
    }
    let mut a = Input::open(&pair_path(reference)?)?;
    let mut b = Input::open(&pair_path(candidate)?)?;
    let (mut left, mut right) = (Vec::new(), Vec::new());
    let mut pairs = 0;
    loop {
        left.clear();
        right.clear();
        let old = a.read_until(b'\n', &mut left)?;
        let new = b.read_until(b'\n', &mut right)?;
        if old == 0 && new == 0 {
            break;
        }
        if old == 0 || new == 0 {
            return fail(format!("pair row count differs after {pairs}"));
        }
        compare_pair(&left, &right).map_err(|error| format!("pair row {}: {error}", pairs + 1))?;
        pairs += 1;
    }
    a.finish()?;
    b.finish()?;
    Ok(pairs)
}
pub(crate) fn run(args: &Args) -> Result<i32> {
    let pairs = verify(Path::new(&args.positional[0]), Path::new(&args.positional[1]))?;
    println!("{}", serde_json::json!({"status":"PASS", "pairs_compared":pairs}));
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::process::{Command, Stdio};
    fn row() -> Value {
        json!({"test":"case", "index":0, "old_kind":"Estimate", "new_kind":"Estimate",
            "kind_mismatch":false,"identical_debug":false,"money_keys_compared":1,
            "money_changes":[{"old":1,"new":2}],"declared_reason":{"why":"shape"},
            "numeric_audit":[{"old":1,"new":2}],"currency_changes":[],"structural_changes":[]})
    }
    #[test]
    fn typed_fields_values_and_audit_verdicts_fail_closed() {
        let reference = row().to_string();
        let mut compact = row();
        compact["numeric_audit"] = json!([{"kind":"Fingerprint"}]);
        assert!(compare_pair(reference.as_bytes(), compact.to_string().as_bytes()).is_ok());
        let reordered = reference.replace(r#"{"new":2,"old":1}"#, r#"{ "old":1, "new":2 }"#);
        assert_ne!(reference, reordered);
        assert!(compare_pair(reference.as_bytes(), reordered.as_bytes()).is_ok());
        for (key, value) in [
            ("test", json!(1)),
            ("old_kind", json!("Send")),
            ("new_kind", json!(null)),
            ("identical_debug", json!(0)),
            ("index", json!(true)),
            ("kind_mismatch", json!(0)),
            ("money_keys_compared", json!(-1)),
            ("money_changes", json!([{"old":1,"new":3}])),
            ("declared_reason", json!("wrong")),
            ("numeric_audit", json!([])),
            ("currency_changes", json!([1])),
            ("structural_changes", json!([1])),
            ("numeric_audit", Value::Null),
            ("currency_changes", Value::Null),
            ("structural_changes", Value::Null),
            ("money_changes", Value::Null),
        ] {
            let mut bad = row();
            bad[key] = value;
            assert!(
                compare_pair(reference.as_bytes(), bad.to_string().as_bytes()).is_err(),
                "{key}"
            );
        }
        for key in row().as_object().unwrap().keys() {
            let mut bad = row();
            bad.as_object_mut().unwrap().remove(key);
            assert!(
                compare_pair(reference.as_bytes(), bad.to_string().as_bytes()).is_err(),
                "{key}"
            );
            let duplicate =
                reference.replacen('{', &format!("{{{}:{},", json!(key), row()[key]), 1);
            assert!(compare_pair(reference.as_bytes(), duplicate.as_bytes()).is_err(), "{key}");
        }
        for bad in ["{}", "null", "[]", "{", "{} trailing", "\n"] {
            assert!(compare_pair(reference.as_bytes(), bad.as_bytes()).is_err());
        }
        let malformed =
            reference.replace(r#""numeric_audit":[{"new":2,"old":1}]"#, r#""numeric_audit":[1,]"#);
        assert_ne!(reference, malformed);
        assert!(compare_pair(reference.as_bytes(), malformed.as_bytes()).is_err());
    }
    fn write_pair(dir: &Path, codec: &str, rows: &[u8]) {
        let path = dir.join(format!("result.pairs.jsonl.{codec}"));
        if codec == "zst" {
            std::fs::write(path, zstd::stream::encode_all(rows, 1).unwrap()).unwrap();
        } else {
            use std::io::Write;
            let mut child = Command::new("gzip")
                .arg("-c")
                .stdin(Stdio::piped())
                .stdout(std::fs::File::create(path).unwrap())
                .spawn()
                .unwrap();
            child.stdin.take().unwrap().write_all(rows).unwrap();
            assert!(child.wait().unwrap().success());
        }
    }
    #[test]
    fn containers_metadata_order_eof_and_decoder_status_are_checked() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let a = temp.0.join("a");
        let b = temp.0.join("b");
        for dir in [&a, &b] {
            std::fs::create_dir(dir).unwrap();
            for name in ["result.summary.json", "final.json", "final-deltas.json"] {
                std::fs::write(dir.join(name), r#"{"differences":170,"excluded":{"estimates":2}}"#)
                    .unwrap();
            }
        }
        let mut second = row();
        second["index"] = json!(1);
        let rows = format!("{}\n{}\n", row(), second);
        write_pair(&a, "gz", rows.as_bytes());
        write_pair(&b, "zst", rows.as_bytes());
        assert_eq!(verify(&a, &b).unwrap(), 2);
        assert_eq!(verify(&b, &a).unwrap(), 2);
        std::fs::write(b.join("result.pairs.jsonl"), &rows).unwrap();
        std::fs::remove_file(b.join("result.pairs.jsonl.zst")).unwrap();
        assert_eq!(verify(&a, &b).unwrap(), 2);
        std::fs::remove_file(b.join("result.pairs.jsonl")).unwrap();
        assert!(verify(&a, &b).is_err());
        for rows in [
            format!("{}\n", row()),
            format!("{}\n{}\n{}\n", row(), second, row()),
            format!("{second}\n{}\n", row()),
            "broken\n".into(),
        ] {
            write_pair(&b, "zst", rows.as_bytes());
            assert!(verify(&a, &b).is_err());
        }
        write_pair(&b, "zst", rows.as_bytes());
        for name in ["result.summary.json", "final.json", "final-deltas.json"] {
            let path = b.join(name);
            let original = std::fs::read(&path).unwrap();
            for bad in [
                r#"{"differences":170,"excluded":{"estimates":1}}"#,
                "{",
                r#"{"differences":170,"excluded":{"estimates":1,"estimates":2}}"#,
            ] {
                std::fs::write(&path, bad).unwrap();
                assert!(verify(&a, &b).is_err());
            }
            std::fs::remove_file(&path).unwrap();
            assert!(verify(&a, &b).is_err());
            std::fs::write(&path, original).unwrap();
        }
        write_pair(&b, "gz", rows.as_bytes());
        assert!(verify(&a, &b).is_err());
        std::fs::remove_file(b.join("result.pairs.jsonl.gz")).unwrap();
        let path = a.join("result.pairs.jsonl.gz");
        let mut truncated = std::fs::read(&path).unwrap();
        truncated.truncate(truncated.len() - 4);
        std::fs::write(path, truncated).unwrap();
        assert!(verify(&a, &b).is_err());
    }
}
