//! Final-estimate and whole-sequence comparison; no flow resynchronization.
use crate::{
    Args, Result, TempDir,
    capture::{Capture, Entry, Reader},
    json::{Json, object, val},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicUsize, Ordering},
};

pub fn flat(value: &Value, path: &str, out: &mut BTreeMap<String, Value>) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                flat(
                    value,
                    &if path.is_empty() { key.clone() } else { format!("{path}.{key}") },
                    out,
                );
            }
        }
        Value::Array(items) => {
            for (index, value) in items.iter().enumerate() {
                flat(value, &format!("{path}[{index}]"), out);
            }
        }
        _ => {
            out.insert(path.into(), value.clone());
        }
    }
}
fn flattened(value: &Value) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    flat(value, "", &mut out);
    out
}
fn estimates(capture: &Capture) -> BTreeMap<String, Vec<&Entry>> {
    capture
        .tests
        .iter()
        .filter_map(|(test, entries)| {
            let mut estimates: Vec<_> =
                entries.iter().filter(|entry| entry.kind == "Estimate").collect();
            estimates.sort_by_key(|entry| entry.index);
            if estimates.is_empty() { None } else { Some((test.clone(), estimates)) }
        })
        .collect()
}
fn sha1_for_tie(text: &str) -> Result<String> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut process =
        Command::new("sha1sum").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
    process.stdin.take().ok_or("missing sha1sum stdin")?.write_all(text.as_bytes())?;
    let result = process.wait_with_output()?;
    if !result.status.success() {
        return crate::fail("sha1sum failed for duplicate estimate index");
    }
    let output = String::from_utf8(result.stdout)?;
    let digest = output.split_whitespace().next().ok_or("missing SHA1 digest")?;
    if digest.len() != 40 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return crate::fail("invalid SHA1 digest");
    }
    Ok(digest.into())
}
fn sequence_order<'a>(entries: &[&'a Entry], reader: &mut Reader) -> Result<Vec<&'a Entry>> {
    let mut ordered = Vec::with_capacity(entries.len());
    let mut start = 0;
    while start < entries.len() {
        let mut end = start + 1;
        while end < entries.len() && entries[end].index == entries[start].index {
            end += 1;
        }
        if end - start == 1 {
            ordered.push(entries[start]);
        } else {
            let mut group = Vec::new();
            for entry in &entries[start..end] {
                group.push((sha1_for_tie(&crate::spaced_json(&reader.money(entry)?))?, *entry));
            }
            group.sort_by(|a, b| a.0.cmp(&b.0));
            ordered.extend(group.into_iter().map(|(_, entry)| entry));
        }
        start = end;
    }
    Ok(ordered)
}
fn same_money_bytes(old: &Entry, new: &Entry) -> bool {
    old.money_hash.zip(new.money_hash).is_some_and(|(a, b)| a == b)
}
fn same_money(old: &Entry, new: &Entry, a: &mut Reader, b: &mut Reader) -> Result<bool> {
    if same_money_bytes(old, new) {
        return Ok(true);
    }
    Ok(crate::spaced_json(&a.money(old)?) == crate::spaced_json(&b.money(new)?))
}
fn matching_money_group(base: &[&Entry], tip: &[&Entry]) -> bool {
    if base.len() != tip.len() {
        return false;
    }
    if base.iter().zip(tip).all(|(a, b)| same_money_bytes(a, b)) {
        return true;
    }
    let hashes = |entries: &[&Entry]| {
        let mut hashes =
            entries.iter().map(|entry| entry.money_hash).collect::<Option<Vec<_>>>()?;
        hashes.sort_unstable();
        Some(hashes)
    };
    hashes(base).zip(hashes(tip)).is_some_and(|(a, b)| a == b)
}
fn sequence_verdict(
    base: &[&Entry],
    tip: &[&Entry],
    a: &mut Reader,
    b: &mut Reader,
) -> Result<String> {
    // Python sorts (index, SHA1) for sequences, independently of its last-observed
    // tie rule for the final estimate. Unique ordinals need no hash subprocess.
    let mut common = 0;
    while common < base.len() && common < tip.len() {
        let group_end = |entries: &[&Entry]| {
            common + entries[common..].partition_point(|entry| entry.index == entries[common].index)
        };
        let old_end = group_end(base);
        let new_end = group_end(tip);
        if old_end != new_end
            || !matching_money_group(&base[common..old_end], &tip[common..new_end])
        {
            break;
        }
        common = old_end;
    }
    let base_order = sequence_order(&base[common..], a)?;
    let tip_order = sequence_order(&tip[common..], b)?;
    for (old, new) in base_order.iter().zip(&tip_order) {
        if !same_money(old, new, a, b)? {
            break;
        }
        common += 1;
    }
    Ok(verdict(common, base.len(), tip.len()))
}
fn verdict(common: usize, base: usize, tip: usize) -> String {
    if common == base && tip > base {
        format!("tip only adds steps ({base} base estimates identical, {} added)", tip - base)
    } else if common == tip && base > tip {
        format!("tip removes steps ({tip} estimates identical, {} removed)", base - tip)
    } else {
        format!("diverges at estimate {} of {base} base / {tip} tip", common + 1)
    }
}
pub fn run(args: &Args) -> Result<i32> {
    if args.positional.len() < 2 || args.positional.len() > 3 {
        return crate::fail("usage: tax-paired-figures final OLD_EVENTS NEW_EVENTS [DELTAS_OUT]");
    }
    let scratch = args
        .positional
        .get(2)
        .and_then(|path| std::path::Path::new(path).parent())
        .map(std::path::Path::to_path_buf)
        .or_else(|| std::env::var_os("TMPDIR").map(Into::into))
        .unwrap_or(std::env::current_dir()?);
    let temp = TempDir::new(&scratch)?;
    let index_timer = crate::timing::PhaseTimer::new("final_index");
    let cache = args.options.get("capture-index-dir").map(std::path::Path::new);
    let (mut old, mut new) = std::thread::scope(|scope| {
        let a = scope.spawn(|| Capture::index_cached(&args.positional[0], &temp.0, false, cache));
        let b = Capture::index_cached(&args.positional[1], &temp.0, false, cache);
        Ok::<_, crate::Error>((a.join().map_err(|_| "capture indexing worker panicked")??, b?))
    })?;
    crate::capture::exclude_capacity(&mut old, &mut new);
    index_timer.finish();
    let final_timer = crate::timing::PhaseTimer::new("final");
    let old_estimates = estimates(&old);
    let new_estimates = estimates(&new);
    let old_keys: BTreeSet<_> = old_estimates.keys().collect();
    let new_keys: BTreeSet<_> = new_estimates.keys().collect();
    let common: Vec<_> = old_keys.intersection(&new_keys).copied().collect();
    let only_old: Vec<_> = old_keys.difference(&new_keys).map(|test| (*test).clone()).collect();
    let only_new: Vec<_> = new_keys.difference(&old_keys).map(|test| (*test).clone()).collect();
    let next = AtomicUsize::new(0);
    let (send, receive) = std::sync::mpsc::channel();
    let comparisons = std::thread::scope(|scope| {
        for _ in 0..args.jobs.min(common.len().max(1)) {
            let send = send.clone();
            let common = &common;
            let next = &next;
            let old = &old;
            let new = &new;
            let old_estimates = &old_estimates;
            let new_estimates = &new_estimates;
            scope.spawn(move || {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    if index >= common.len() {
                        break;
                    }
                    let test = common[index];
                    let result = (|| -> Result<_> {
                        let before = &old_estimates[test];
                        let after = &new_estimates[test];
                        let before_final = before.last().unwrap();
                        let after_final = after.last().unwrap();
                        if same_money_bytes(before_final, after_final) {
                            return Ok(None);
                        }
                        let mut a = old.reader()?;
                        let mut b = new.reader()?;
                        let before_money = a.money(before_final)?;
                        let after_money = b.money(after_final)?;
                        if crate::spaced_json(&before_money) == crate::spaced_json(&after_money) {
                            return Ok(None);
                        }
                        let verdict = sequence_verdict(before, after, &mut a, &mut b)?;
                        Ok(Some((verdict, flattened(&before_money), flattened(&after_money))))
                    })();
                    if send.send((index, result)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(send);
        receive.into_iter().collect::<BTreeMap<_, _>>()
    });
    let mut differ = Vec::new();
    let mut verdicts = Vec::new();
    let mut needs_review = Vec::new();
    let mut one_side_empty = Vec::new();
    let mut figures_changed = Vec::new();
    let mut deltas = Vec::new();
    for (index, result) in comparisons {
        let Some((verdict, before, after)) = result? else {
            continue;
        };
        let test = &common[index];
        differ.push((*test).clone());
        if verdict.starts_with("diverges") {
            needs_review.push((*test).clone());
        }
        verdicts.push(((*test).clone(), val(verdict)));
        if !before.values().any(|value| !value.is_null())
            || !after.values().any(|value| !value.is_null())
        {
            one_side_empty.push((*test).clone());
        } else {
            figures_changed.push((*test).clone());
        }
        let changed =
            before.keys().chain(after.keys()).collect::<BTreeSet<_>>().into_iter().filter_map(
                |key| {
                    let old = before.get(key).cloned().unwrap_or(Value::Null);
                    let new = after.get(key).cloned().unwrap_or(Value::Null);
                    if crate::py_equal(&old, &new) {
                        None
                    } else {
                        Some((key.clone(), Json::Array(vec![val(old), val(new)])))
                    }
                },
            );
        deltas.push(((*test).clone(), object(changed)));
    }
    if let Some(path) = args.positional.get(2) {
        std::fs::write(path, object(deltas).render(Some(1)))?;
    }
    let mut report = object([
        ("tests_compared", val(common.len() as u64)),
        ("final_figures_differ", val(differ.len() as u64)),
        ("differ_one_side_empty", val(one_side_empty.len() as u64)),
        ("differ_figures_changed", val(figures_changed.len() as u64)),
        ("differ_needs_review", val(needs_review.len() as u64)),
        ("final_estimate_only_old", val(only_old.len() as u64)),
        ("final_estimate_only_new", val(only_new.len() as u64)),
        (
            "examples",
            object([
                ("differ", val(serde_json::to_value(&differ)?)),
                ("verdicts", object(verdicts)),
                ("needs_review", val(serde_json::to_value(needs_review)?)),
                ("one_side_empty", val(serde_json::to_value(one_side_empty)?)),
                ("figures_changed", val(serde_json::to_value(figures_changed)?)),
                ("only_old", val(serde_json::to_value(&only_old)?)),
                ("only_new", val(serde_json::to_value(&only_new)?)),
            ]),
        ),
    ]);
    crate::capture::add_exclusions(&mut report, &old, &new);
    println!("{}", report.render(Some(1)));
    final_timer.finish();
    Ok(i32::from(!differ.is_empty() || !only_old.is_empty() || !only_new.is_empty()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequence_prefix_and_divergence_keep_estimate_ordinals() {
        assert_eq!(verdict(2, 2, 4), "tip only adds steps (2 base estimates identical, 2 added)");
        assert_eq!(verdict(2, 4, 2), "tip removes steps (2 estimates identical, 2 removed)");
        assert_eq!(verdict(1, 3, 4), "diverges at estimate 2 of 3 base / 4 tip");
    }
    #[test]
    fn empty_containers_are_absent_but_zero_is_a_figure() {
        assert!(flattened(&serde_json::json!({"a":[],"b":{}})).is_empty());
        assert_eq!(flattened(&serde_json::json!({"a":[null,0]}))["a[1]"], 0);
    }
}

#[cfg(test)]
mod duplicate_index_tests {
    use super::*;
    #[test]
    fn duplicate_index_hash_order_is_separate_from_last_capture_final() {
        assert_eq!(sha1_for_tie("abc").unwrap(), "a9993e364706816aba3e25717850c26c9cd0d89d");
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let old = temp.0.join("old.jsonl");
        let new = temp.0.join("new.jsonl");
        let row = |amount| {
            format!(
                "{{\"kind\":\"Estimate\",\"test\":\"case\",\"index\":5,\"money\":{{\"x\":{amount}}}}}\n"
            )
        };
        std::fs::write(&old, format!("{}{}", row(1), row(2))).unwrap();
        std::fs::write(&new, format!("{}{}", row(2), row(1))).unwrap();
        let old = Capture::index(old.to_str().unwrap(), &temp.0, false).unwrap();
        let new = Capture::index(new.to_str().unwrap(), &temp.0, false).unwrap();
        let a = estimates(&old);
        let b = estimates(&new);
        let mut reader_a = old.reader().unwrap();
        let mut reader_b = new.reader().unwrap();
        assert_eq!(reader_a.money(a["case"].last().unwrap()).unwrap()["x"], 2);
        assert_eq!(reader_b.money(b["case"].last().unwrap()).unwrap()["x"], 1);
        assert_eq!(
            sequence_verdict(&a["case"], &b["case"], &mut reader_a, &mut reader_b).unwrap(),
            "diverges at estimate 3 of 2 base / 2 tip"
        );
    }
}

#[cfg(test)]
#[path = "final_figures_tests.rs"]
mod raw_money_tests;
