use serde_json::{Value, json};
use std::{path::Path, process::Command};

struct Scratch(std::path::PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tax-paired-figures")).args(args).output().unwrap()
}
fn pairs(prefix: &Path) -> Vec<String> {
    let zstd = prefix.with_extension("pairs.jsonl.zst");
    let (codec, path) = if zstd.exists() {
        ("zstd", zstd)
    } else {
        ("gzip", prefix.with_extension("pairs.jsonl.gz"))
    };
    let output = Command::new(codec).args(["-dc", "--"]).arg(path).output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().lines().map(str::to_owned).collect()
}
#[test]
fn differences_only_preserves_summary_exit_and_exact_selected_records() {
    let scratch =
        Scratch(std::env::temp_dir().join(format!("tax-filter-cli-{}", std::process::id())));
    std::fs::create_dir(&scratch.0).unwrap();
    let old_path = scratch.0.join("old.jsonl");
    let new_path = scratch.0.join("new.jsonl");
    let map_path = scratch.0.join("map.json");
    std::fs::write(&map_path, json!({"version":1,"variant_paths":{"W2":["Wage","W2"]},"source_fields":[],"catalogs":[],"field_paths":[]}).to_string()).unwrap();
    let debug_pairs = [
        ("Reply { value: 1 }", "Reply { value: 1 }"),
        ("Reply { value: 1 }", "Reply { value: 1 } "),
        ("Reply { value: 1 }", "Reply { value: 1 }"),
        ("Reply { value: 1 }", "Reply { value: 1 }"),
        ("Reply { value: 1 }", "Reply { value: 2 }"),
        (
            "Currency { currency_code: USD, scale: 2, minor_units: 100 }",
            "Currency { currency_code: EUR, scale: 2, minor_units: 100 }",
        ),
        ("Reply { value: Before }", "Reply { value: After }"),
    ];
    let mut old = String::new();
    let mut new = String::new();
    for (index, (a, b)) in debug_pairs.into_iter().enumerate() {
        let event = |debug: &str| json!({"version":1,"test":"control","index":index,"kind":"Estimate","debug":debug,"money":{"top":{"tax":100},"return_rows":[],"lines":[]}});
        let left = event(a);
        let mut right = event(b);
        if index == 2 {
            right["money"]["top"]["tax"] = json!(101);
        }
        if index == 3 {
            right["kind"] = json!("Problem");
        }
        old.push_str(&format!("{left}\n"));
        new.push_str(&format!("{right}\n"));
    }
    // Unmatched events remain summarized without introducing extra pair records.
    new.push_str(
        &json!({"version":1,"test":"new-only","index":0,"kind":"Problem","debug":"None"})
            .to_string(),
    );
    new.push('\n');
    std::fs::write(&old_path, old).unwrap();
    std::fs::write(&new_path, new).unwrap();
    for structural in [false, true] {
        let full = scratch.0.join(format!("full-{structural}"));
        let small = scratch.0.join(format!("small-{structural}"));
        let mut args = vec![
            "compare",
            "--old",
            old_path.to_str().unwrap(),
            "--new",
            new_path.to_str().unwrap(),
            "--jobs",
            "2",
            "--output",
            full.to_str().unwrap(),
        ];
        if structural {
            args.extend(["--structure-map", map_path.to_str().unwrap()]);
        }
        let legacy = cli(&args);
        args[8] = small.to_str().unwrap();
        args.push("--differences-only");
        let filtered = cli(&args);
        assert_eq!(legacy.status.code(), Some(1), "{}", String::from_utf8_lossy(&legacy.stderr));
        assert_eq!(
            filtered.status.code(),
            legacy.status.code(),
            "{}",
            String::from_utf8_lossy(&filtered.stderr)
        );
        assert_eq!(filtered.stdout, legacy.stdout);
        let summary = std::fs::read(full.with_extension("summary.json")).unwrap();
        assert_eq!(std::fs::read(small.with_extension("summary.json")).unwrap(), summary);
        let summary: Value = serde_json::from_slice(&summary).unwrap();
        assert_eq!(summary["counts"]["pairs_compared"], 7);
        assert_eq!(summary["counts"]["pairs_with_identical_debug"], 3);
        assert_eq!(summary["counts"]["new_only_events"], 1);
        let original = pairs(&full);
        assert_eq!(original.len(), 7);
        assert_eq!(pairs(&small), original[1..]);
        let debug_only: Value = serde_json::from_str(&original[1]).unwrap();
        for key in ["money_changes", "numeric_audit", "currency_changes", "structural_changes"] {
            assert_eq!(debug_only[key], json!([]));
        }
        let typed: Value = serde_json::from_str(&original[2]).unwrap();
        assert_eq!(typed["identical_debug"], true);
        assert_eq!(typed["money_changes"][0]["new"], 101);
        let mismatch: Value = serde_json::from_str(&original[3]).unwrap();
        assert_eq!(mismatch["identical_debug"], true);
        assert_eq!(mismatch["kind_mismatch"], true);
        if structural {
            assert!(summary["counts"]["pairs_with_structural_changes"].as_u64().unwrap() > 0);
        }
    }
}
#[test]
fn flag_is_boolean_and_compare_only() {
    for args in [
        vec!["compare", "--old", "a", "--new", "b", "--output", "c", "--differences-only", "true"],
        vec!["compare", "--old", "a", "--new", "b", "--output", "c", "--differences-only=false"],
        vec!["compress", "a", "b", "--differences-only"],
        vec!["final", "a", "b", "--differences-only"],
        vec!["final-figures", "a", "b", "--differences-only"],
    ] {
        assert_eq!(cli(&args).status.code(), Some(2), "{args:?}");
    }
}

#[test]
fn raw_shortcuts_chunks_codecs_and_encoded_strings_preserve_exact_records() {
    let scratch =
        Scratch(std::env::temp_dir().join(format!("tax-fast-pair-{}", std::process::id())));
    std::fs::create_dir(&scratch.0).unwrap();
    let old = scratch.0.join("old.jsonl");
    let new = scratch.0.join("new.jsonl");
    let encoded = scratch.0.join("encoded.jsonl");
    let mapping = scratch.0.join("mapping.json");
    std::fs::write(&mapping, r#"{"*":{"left":"right"}}"#).unwrap();
    let mut before = String::new();
    let mut after = String::new();
    let mut alternate = String::new();
    for test in ["first", "second"] {
        for index in 0..97 {
            let debug = if index % 7 == 0 { "Reply { amount: 2 }" } else { "Reply { amount: 1 }" };
            let event = |id, debug: &str, amount| {
                json!({"version":1,"test":test,"index":index,
                "kind":"Estimate","debug":debug,"money":{"top":{"tax":100,"credit":0},
                "return_rows":[{"amount":100,"money":{"currency_code":"USD","minor_units":100,"scale":2}}],
                "lines":[{"id":id,"kind":"Money","amount":amount},{"id":id,"kind":"Money","amount":200}]}})
            };
            let left = event("left", "Reply { amount: 1 }", 100);
            let mut right = event("right", debug, if index % 5 == 0 { 101 } else { 100 });
            if index % 11 == 0 {
                right["kind"] = json!("Problem");
            }
            before.push_str(&format!("{left}\n"));
            after.push_str(&format!("{right}\n"));
            // Identical decoded strings with distinct raw JSON spellings must use
            // the original decoded equality path and produce identical records.
            alternate.push_str(&format!("{}\n", right.to_string().replace("Reply", r"\u0052eply")));
        }
    }
    std::fs::write(&old, before).unwrap();
    std::fs::write(&new, after).unwrap();
    std::fs::write(&encoded, alternate).unwrap();
    let mut baseline = None;
    for (name, input, jobs, codec, budget) in [
        ("serial", &new, "1", "gzip", "1"),
        ("parallel", &new, "12", "zstd", "10240"),
        ("encoded", &encoded, "12", "zstd", "1"),
    ] {
        let prefix = scratch.0.join(name);
        let out = cli(&[
            "compare",
            "--old",
            old.to_str().unwrap(),
            "--new",
            input.to_str().unwrap(),
            "--output",
            prefix.to_str().unwrap(),
            "--line-map",
            mapping.to_str().unwrap(),
            "--jobs",
            jobs,
            "--pairs-compression",
            codec,
            "--memory-budget-mib",
            budget,
        ]);
        assert_eq!(out.status.code(), Some(1), "{}", String::from_utf8_lossy(&out.stderr));
        let summary = std::fs::read(prefix.with_extension("summary.json")).unwrap();
        let records = pairs(&prefix);
        assert_eq!(records.len(), 194);
        let report: Value = serde_json::from_slice(&summary).unwrap();
        assert_eq!(report["counts"]["typed_money_keys_compared"], 194 * 6);
        if let Some((expected_summary, expected_records, expected_stdout)) = &baseline {
            assert_eq!(&summary, expected_summary);
            assert_eq!(&records, expected_records);
            assert_eq!(&out.stdout, expected_stdout);
        } else {
            baseline = Some((summary, records, out.stdout));
        }
    }
}

#[test]
fn identical_raw_debug_does_not_hide_absent_or_malformed_typed_money() {
    let scratch =
        Scratch(std::env::temp_dir().join(format!("tax-fast-errors-{}", std::process::id())));
    std::fs::create_dir(&scratch.0).unwrap();
    let input = scratch.0.join("input.jsonl");
    let prefix = scratch.0.join("result");
    for (money, expected) in [
        (None, "missing Estimate money.top"),
        (
            Some(json!({"top":{},"return_rows":[],"lines":[{"kind":"Money","id":"x"}]})),
            "missing money line amount",
        ),
        (
            Some(json!({"top":{},"return_rows":[{"money":{}}],"lines":[]})),
            "missing return row amount",
        ),
    ] {
        let mut row =
            json!({"version":1,"test":"case","index":0,"kind":"Estimate","debug":"opaque @"});
        if let Some(money) = money {
            row["money"] = money;
        }
        std::fs::write(&input, format!("{row}\n")).unwrap();
        let out = cli(&[
            "compare",
            "--old",
            input.to_str().unwrap(),
            "--new",
            input.to_str().unwrap(),
            "--output",
            prefix.to_str().unwrap(),
        ]);
        assert_eq!(out.status.code(), Some(1));
        assert!(
            String::from_utf8_lossy(&out.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!prefix.with_extension("summary.json").exists());
    }
}
