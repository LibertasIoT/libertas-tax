use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tax-paired-figures")).args(args).output().unwrap()
}
fn records(path: &Path) -> Vec<Value> {
    let zstd = path.with_extension("zst");
    let (codec, path) = if zstd.exists() { ("zstd", zstd.as_path()) } else { ("gzip", path) };
    let out = Command::new(codec).args(["-dc", "--"]).arg(path).output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
#[test]
fn compact_capture_preserves_complete_verdict_counts_changed_pairs_and_final_figures() {
    let temp =
        Scratch(std::env::temp_dir().join(format!("tax-compact-cli-{}", std::process::id())));
    std::fs::create_dir(&temp.0).unwrap();
    let row = |test: &str, index, debug: String, tax| json!({"version":1,"test":test,"index":index,"kind":"Estimate","debug":debug,"money":{"top":{"tax":tax},"lines":[],"return_rows":[]}});
    let large =
        |n| format!("Reply {{ numbers: [{}], changed: {n} }}", vec!["100"; 20000].join(","));
    let mut a = vec![
        row("case", 0, "Reply { a: 1, text: Before }".into(), 10),
        row("case", 1, "Reply { a: 1, text: Before }".into(), 10),
        row("case", 2, "Reply { a: 1, text: Before }".into(), 10),
        row("case", 3, "Reply { a: 1 }".into(), 10),
        row("unit53_capacity::control", 0, large(1), 20),
        row(
            "unit53_capacity::control",
            1,
            "Currency { currency_code: USD, scale: 2, minor_units: 100 }".into(),
            20,
        ),
    ];
    let mut b = a.clone();
    b[1]["debug"] = json!("Reply { a: 1, text: After }");
    b[2]["debug"] = json!("Reply { a: 2, text: Before }");
    b[3]["money"]["top"]["tax"] = json!(11);
    b[4]["debug"] = json!(large(2));
    b[5]["debug"] = json!("Currency { currency_code: EUR, scale: 2, minor_units: 100 }");
    a.push(json!({"version":1,"test":"send","index":0,"kind":"Send","debug":"Reply { saved: 1 }","money":null}));
    b.push(json!({"version":1,"test":"send","index":0,"kind":"Send","debug":"Reply { saved: 2 }","money":null}));
    let old = temp.0.join("old.jsonl");
    let new = temp.0.join("new.jsonl");
    for (path, rows) in [(&old, &a), (&new, &b)] {
        std::fs::write(path, rows.iter().map(|v| v.to_string() + "\n").collect::<String>())
            .unwrap();
    }
    let old_compact = temp.0.join("old.zst");
    let new_compact = temp.0.join("new.zst");
    for (input, output) in [(&old, &old_compact), (&new, &new_compact)] {
        let before = std::fs::read(input).unwrap();
        let out = run(&["reduce", input.to_str().unwrap(), output.to_str().unwrap()]);
        assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
        assert_eq!(std::fs::read(input).unwrap(), before);
    }
    let full = temp.0.join("full");
    let small = temp.0.join("small");
    let mixed = temp.0.join("mixed");
    for (left, right, prefix, filtered) in [
        (&old, &new, &full, false),
        (&old_compact, &new_compact, &small, true),
        (&old, &new_compact, &mixed, true),
    ] {
        let mut args = vec![
            "compare",
            "--old",
            left.to_str().unwrap(),
            "--new",
            right.to_str().unwrap(),
            "--output",
            prefix.to_str().unwrap(),
            "--jobs",
            "2",
        ];
        if filtered {
            args.push("--differences-only");
        }
        let out = run(&args);
        assert_eq!(out.status.code(), Some(1), "{}", String::from_utf8_lossy(&out.stderr));
    }
    let summary = std::fs::read(full.with_extension("summary.json")).unwrap();
    assert_eq!(std::fs::read(small.with_extension("summary.json")).unwrap(), summary);
    assert_eq!(std::fs::read(mixed.with_extension("summary.json")).unwrap(), summary);
    let selected = |rows: Vec<Value>| {
        rows.into_iter()
            .filter(|v| {
                v["identical_debug"] != true
                    || v["kind_mismatch"] == true
                    || ["money_changes", "numeric_audit", "currency_changes", "structural_changes"]
                        .iter()
                        .any(|k| v[k].as_array().is_some_and(|v| !v.is_empty()))
            })
            .map(|v| (v["test"].as_str().unwrap().to_owned(), v["index"].as_u64().unwrap()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        selected(records(&full.with_extension("pairs.jsonl.gz"))),
        selected(records(&small.with_extension("pairs.jsonl.gz")))
    );
    assert_eq!(
        selected(records(&full.with_extension("pairs.jsonl.gz"))),
        selected(records(&mixed.with_extension("pairs.jsonl.gz")))
    );
    let final_full = run(&["final", old.to_str().unwrap(), new.to_str().unwrap()]);
    let final_small = run(&["final", old_compact.to_str().unwrap(), new_compact.to_str().unwrap()]);
    assert_eq!(final_small.status.code(), final_full.status.code());
    assert_eq!(final_small.stdout, final_full.stdout);
    let mut bad = tax_paired_figures::compact_event(&a[0]).unwrap();
    bad.as_object_mut().unwrap().remove("numeric_sha256");
    let broken = temp.0.join("bad.jsonl");
    std::fs::write(&broken, bad.to_string() + "\n").unwrap();
    let out = run(&[
        "compare",
        "--old",
        broken.to_str().unwrap(),
        "--new",
        new_compact.to_str().unwrap(),
        "--output",
        temp.0.join("bad-result").to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(!temp.0.join("bad-result.summary.json").exists());
}

#[test]
fn unavailable_audits_preserve_equal_debug_and_reject_changed_debug() {
    let temp =
        Scratch(std::env::temp_dir().join(format!("tax-unavailable-cli-{}", std::process::id())));
    std::fs::create_dir(&temp.0).unwrap();
    for (case, text) in [
        "opaque @".to_owned(),
        "@".repeat(65537),
        "Currency { scale: 2, minor_units: 100 }".to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let old = temp.0.join(format!("old-{case}.jsonl"));
        let new = temp.0.join(format!("new-{case}.jsonl"));
        let row = |tax| json!({"version":1,"test":"opaque","index":0,"kind":"Estimate","debug":text,"money":{"top":{"tax":tax},"lines":[],"return_rows":[]}});
        std::fs::write(&old, row(10).to_string() + "\n").unwrap();
        std::fs::write(&new, row(11).to_string() + "\n").unwrap();
        let a = temp.0.join(format!("a-{case}.zst"));
        let b = temp.0.join(format!("b-{case}.zst"));
        for (input, output) in [(&old, &a), (&new, &b)] {
            let out = run(&["reduce", input.to_str().unwrap(), output.to_str().unwrap()]);
            assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
        }
        let full = temp.0.join(format!("full-{case}"));
        let compact = temp.0.join(format!("compact-{case}"));
        for (left, right, prefix) in [(&old, &new, &full), (&a, &b, &compact)] {
            let out = run(&[
                "compare",
                "--old",
                left.to_str().unwrap(),
                "--new",
                right.to_str().unwrap(),
                "--output",
                prefix.to_str().unwrap(),
            ]);
            assert_eq!(out.status.code(), Some(1), "{}", String::from_utf8_lossy(&out.stderr));
            assert!(prefix.with_extension("summary.json").exists());
        }
        assert_eq!(
            std::fs::read(full.with_extension("summary.json")).unwrap(),
            std::fs::read(compact.with_extension("summary.json")).unwrap()
        );
        let mut changed = row(11);
        changed["debug"] = json!(format!("{text}#"));
        std::fs::write(&new, changed.to_string() + "\n").unwrap();
        let changed_path = temp.0.join(format!("changed-{case}.zst"));
        assert_eq!(
            run(&["reduce", new.to_str().unwrap(), changed_path.to_str().unwrap()]).status.code(),
            Some(0)
        );
        for (left, right, label) in [(&old, &new, "legacy"), (&a, &changed_path, "compact")] {
            let prefix = temp.0.join(format!("bad-{case}-{label}"));
            let out = run(&[
                "compare",
                "--old",
                left.to_str().unwrap(),
                "--new",
                right.to_str().unwrap(),
                "--output",
                prefix.to_str().unwrap(),
            ]);
            assert_eq!(out.status.code(), Some(1));
            assert!(!prefix.with_extension("summary.json").exists());
        }
        let mut malformed = tax_paired_figures::compact_event(&row(10)).unwrap();
        malformed["numeric_count"] = json!(0);
        assert!(tax_paired_figures::validate_compact_event(&malformed).is_err());
    }
}

#[test]
fn oversized_debug_keeps_typed_figures_and_explicit_unavailable_audits() {
    use tax_paired_figures::compact::{DEBUG_PARSE_LIMIT, SIZE_AUDIT_ERROR};
    let temp = Scratch(std::env::temp_dir().join(format!("tax-size-cli-{}", std::process::id())));
    std::fs::create_dir(&temp.0).unwrap();
    // A malformed first token proves that oversized Debug never enters the parser.
    let mut row = json!({"version":1,"test":"size","index":0,"kind":"Estimate",
        "debug":"", "money":{"top":{"tax":10001},"return_rows":[],"lines":[]}});
    row["debug"] = Value::String("@".repeat(DEBUG_PARSE_LIMIT + 1));
    let old = temp.0.join("old.jsonl");
    let new = temp.0.join("new.jsonl");
    let write_row = |path: &Path, row: &Value| {
        let mut file = std::fs::File::create(path).unwrap();
        serde_json::to_writer(&mut file, row).unwrap();
        std::io::Write::write_all(&mut file, b"\n").unwrap();
    };
    write_row(&old, &row);
    if let Value::String(text) = &mut row["debug"] {
        text.push('@');
    }
    write_row(&new, &row);
    drop(row);
    let old_compact = temp.0.join("old.zst");
    let new_compact = temp.0.join("new.zst");
    for (input, output) in [(&old, &old_compact), (&new, &new_compact)] {
        let out = run(&["reduce", input.to_str().unwrap(), output.to_str().unwrap()]);
        assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    }
    for (label, a, b) in [
        ("full", &old, &new),
        ("compact", &old_compact, &new_compact),
        ("mixed", &old, &new_compact),
        ("reverse-mixed", &old_compact, &new),
    ] {
        let prefix = temp.0.join(label);
        let out = run(&[
            "compare",
            "--old",
            a.to_str().unwrap(),
            "--new",
            b.to_str().unwrap(),
            "--output",
            prefix.to_str().unwrap(),
            "--jobs",
            "1",
            "--differences-only",
        ]);
        assert_eq!(out.status.code(), Some(1), "{}", String::from_utf8_lossy(&out.stderr));
        let final_out = run(&["final", a.to_str().unwrap(), b.to_str().unwrap()]);
        assert_eq!(final_out.status.code(), Some(0));
        let final_summary: Value = serde_json::from_slice(&final_out.stdout).unwrap();
        let summary_bytes = std::fs::read(prefix.with_extension("summary.json")).unwrap();
        let summary: Value = serde_json::from_slice(&summary_bytes).unwrap();
        assert!(summary.get("excluded_oversize_tests").is_none());
        assert!(final_summary.get("excluded_oversize_tests").is_none());
        assert_eq!(summary["tests_selected"]["size"], json!({"old":1,"new":1}));
        assert_eq!(final_summary["tests_compared"], 1);
        assert_eq!(summary["counts"]["unexplained_pairs"], 1);
        assert_eq!(summary["counts"]["pairs_with_unavailable_numeric_audit"], 1);
        assert_eq!(summary["counts"]["pairs_with_unavailable_currency_audit"], 1);
        assert!(summary["counts"].get("pairs_with_numeric_audit_changes").is_none());
        assert_eq!(summary["digest_only_size"][0]["test"], "size");
        assert_eq!(summary["digest_only_size"][0]["index"], 0);
        assert_eq!(summary["digest_only_size"][0]["audit_evidence"], "digest-only (size)");
        assert_eq!(summary["digest_only_size"][0]["old_debug_bytes"], DEBUG_PARSE_LIMIT + 1);
        assert_eq!(summary["digest_only_size"][0]["new_debug_bytes"], DEBUG_PARSE_LIMIT + 2);
        let pairs = records(&prefix.with_extension("pairs.jsonl.gz"));
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0]["numeric_audit"], Value::Null);
        assert_eq!(pairs[0]["currency_changes"], Value::Null);
        assert_eq!(pairs[0]["money_changes"], json!([]));
        assert_eq!(pairs[0]["audit_evidence"]["numeric"], "digest-only (size)");
        assert_ne!(
            pairs[0]["audit_evidence"]["old_debug_sha256"],
            pairs[0]["audit_evidence"]["new_debug_sha256"]
        );
    }
    // Read a real size-only record, then vary just the small typed-money payload.
    let bytes = zstd::stream::decode_all(std::fs::File::open(&old_compact).unwrap()).unwrap();
    let mut a: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(a["audit_error"], SIZE_AUDIT_ERROR);
    let mut b = a.clone();
    b["money"]["top"]["tax"] = json!(10002);
    let compact_a = temp.0.join("a.jsonl");
    let compact_b = temp.0.join("b.jsonl");
    write_row(&compact_a, &a);
    write_row(&compact_b, &b);
    let prefix = temp.0.join("cent");
    let out = run(&[
        "compare",
        "--old",
        compact_a.to_str().unwrap(),
        "--new",
        compact_b.to_str().unwrap(),
        "--output",
        prefix.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(1));
    let pairs = records(&prefix.with_extension("pairs.jsonl.gz"));
    assert_eq!(pairs[0]["identical_debug"], true);
    assert_eq!(pairs[0]["numeric_audit"], json!([]));
    assert_eq!(pairs[0]["money_changes"][0]["old"], 10001);
    assert_eq!(pairs[0]["money_changes"][0]["new"], 10002);
    let mut small = json!({"version":1,"test":"size","index":0,"kind":"Estimate",
        "debug":"Reply { amount: 1 }", "money":b["money"]});
    small = tax_paired_figures::compact_event(&small).unwrap();
    write_row(&compact_b, &small);
    let prefix = temp.0.join("one-size-cent");
    let out = run(&[
        "compare",
        "--old",
        compact_a.to_str().unwrap(),
        "--new",
        compact_b.to_str().unwrap(),
        "--output",
        prefix.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(1));
    let pairs = records(&prefix.with_extension("pairs.jsonl.gz"));
    assert_eq!(pairs[0]["numeric_audit"], Value::Null);
    assert_eq!(pairs[0]["currency_changes"], Value::Null);
    assert_eq!(pairs[0]["money_changes"][0]["old"], 10001);
    assert_eq!(pairs[0]["money_changes"][0]["new"], 10002);
    write_row(&compact_b, &a);
    let prefix = temp.0.join("equal");
    let out = run(&[
        "compare",
        "--old",
        compact_a.to_str().unwrap(),
        "--new",
        compact_b.to_str().unwrap(),
        "--output",
        prefix.to_str().unwrap(),
        "--differences-only",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert!(records(&prefix.with_extension("pairs.jsonl.gz")).is_empty());
    let summary: Value =
        serde_json::from_slice(&std::fs::read(prefix.with_extension("summary.json")).unwrap())
            .unwrap();
    assert_eq!(summary["counts"]["pairs_with_identical_debug"], 1);
    assert_eq!(summary["digest_only_size"][0]["identical_debug"], true);
    // A mapping cannot silently disappear just because Debug was omitted by size.
    let mapping = temp.0.join("structure.json");
    std::fs::write(
        &mapping,
        json!({"version":1,"variant_paths":{"Reply":["Reply"]},"source_fields":[],"catalogs":[],"field_paths":[]})
            .to_string(),
    )
    .unwrap();
    {
        let input = &compact_a;
        let prefix = temp.0.join("mapped");
        let out = run(&[
            "compare",
            "--old",
            input.to_str().unwrap(),
            "--new",
            input.to_str().unwrap(),
            "--output",
            prefix.to_str().unwrap(),
            "--structure-map",
            mapping.to_str().unwrap(),
        ]);
        assert_eq!(out.status.code(), Some(1));
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("size-limited Debug was omitted"),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!prefix.with_extension("summary.json").exists());
    }
    a["debug_bytes"] = json!(DEBUG_PARSE_LIMIT);
    assert!(tax_paired_figures::validate_compact_event(&a).is_err());
}
