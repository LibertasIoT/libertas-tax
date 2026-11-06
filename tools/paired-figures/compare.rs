use crate::{
    Args, Result, TempDir,
    capture::Capture,
    debug, fail,
    json::{Json, object, val},
    structure::Structure,
};
use serde_json::{Value, json};
use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Condvar, Mutex},
};

#[derive(Default)]
pub struct Counts {
    values: Vec<(String, u64)>,
    size_evidence: Vec<Json>,
}
impl Counts {
    pub fn add(&mut self, key: &str, n: u64) {
        if let Some((_, count)) = self.values.iter_mut().find(|(name, _)| name == key) {
            *count += n;
        } else {
            self.values.push((key.into(), n));
        }
    }
    pub fn get(&self, key: &str) -> u64 {
        self.values.iter().find(|(name, _)| name == key).map_or(0, |(_, n)| *n)
    }
    fn merge(&mut self, other: &Self) {
        self.size_evidence.extend(other.size_evidence.iter().cloned());
        for (key, n) in &other.values {
            self.add(key, *n);
        }
    }
    pub fn json(&self) -> Json {
        object(self.values.iter().map(|(key, n)| (key.clone(), val(*n))))
    }
}
pub fn delta(old: &BTreeMap<String, Value>, new: &BTreeMap<String, Value>) -> Vec<Json> {
    old.keys()
        .chain(new.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|key| match (old.get(*key), new.get(*key)) {
            (Some(a), Some(b)) => !crate::py_equal(a, b),
            (None, None) => false,
            _ => true,
        })
        .map(|key| {
            object([
                ("path", val(key.clone())),
                ("old", val(old.get(key).cloned().unwrap_or(Value::Null))),
                ("new", val(new.get(key).cloned().unwrap_or(Value::Null))),
                ("old_present", val(old.contains_key(key))),
                ("new_present", val(new.contains_key(key))),
            ])
        })
        .collect()
}
fn currency_delta(old: &BTreeMap<String, Value>, new: &BTreeMap<String, Value>) -> Vec<Json> {
    let semantic = |values: &BTreeMap<String, Value>| {
        values.iter().map(|(key, value)| (key.clone(), debug::currency_semantic(value))).collect()
    };
    let mut result = delta(&semantic(old), &semantic(new));
    for item in &mut result {
        if let Json::Object(fields) = item {
            let path = fields
                .iter()
                .find_map(|(key, value)| match (key.as_str(), value) {
                    ("path", Json::Value(Value::String(path))) => Some(path.clone()),
                    _ => None,
                })
                .expect("delta always emits a path");
            for (key, value) in fields {
                let source = match key.as_str() {
                    "old" => old.get(&path),
                    "new" => new.get(&path),
                    _ => None,
                };
                if let Some(source) = source {
                    *value = debug::currency_projection_json(source);
                }
            }
        }
    }
    result
}

#[cfg(test)]
fn money(row: &Value, mapping: &Value) -> Result<BTreeMap<String, Value>> {
    if row["kind"] != "Estimate" {
        return Ok(BTreeMap::new());
    }
    project_money_values(&row["money"], mapping)
}
fn project_money_values(values: &Value, mapping: &Value) -> Result<BTreeMap<String, Value>> {
    let mut out = BTreeMap::new();
    for (key, value) in values["top"].as_object().ok_or("missing Estimate money.top")? {
        out.insert(format!("top.{key}"), value.clone());
    }
    for (index, line) in values["return_rows"]
        .as_array()
        .ok_or("missing Estimate money.return_rows")?
        .iter()
        .enumerate()
    {
        let money = line.get("money").ok_or("missing return row money")?;
        if !money.is_null() {
            let amount = line.get("amount").ok_or("missing return row amount")?;
            out.insert(format!("return_rows[{index}].amount"), amount.clone());
            out.insert(format!("return_rows[{index}].money"), money.clone());
        }
    }
    let mut sources = BTreeMap::new();
    let mut occurrences: BTreeMap<String, usize> = BTreeMap::new();
    for line in values["lines"].as_array().ok_or("missing Estimate money.lines")? {
        if line["kind"] != "Money" {
            continue;
        }
        let id = line["id"].as_str().ok_or("missing money line id")?;
        let mapping = mapping.as_object().ok_or("line identity map must be an object")?;
        let mapped = match mapping.get(id) {
            Some(value) => value.as_str().ok_or("line map destination must be a string")?,
            None => id,
        };
        let key = format!("line.{mapped}");
        if let Some(previous) = sources.insert(key.clone(), id)
            && previous != id
        {
            return fail(format!("different line identities map to the same line: {key}"));
        }
        let ordinal = occurrences.entry(key.clone()).or_default();
        let amount = line.get("amount").ok_or("missing money line amount")?;
        out.insert(format!("{key}#{ordinal}"), amount.clone());
        *ordinal += 1;
    }
    Ok(out)
}
fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
        Value::Number(n) => n.as_f64() != Some(0.0),
    }
}
fn compact_row(row: &Value) -> Result<std::borrow::Cow<'_, Value>> {
    if row["version"] == 2 {
        crate::compact::validate_compact_event(row)?;
        Ok(std::borrow::Cow::Borrowed(row))
    } else {
        Ok(std::borrow::Cow::Owned(crate::compact::compact_event(row)?))
    }
}
fn fingerprint_delta(old: &Value, new: &Value, name: &str) -> Vec<Json> {
    let hash_key = format!("{name}_sha256");
    let count_key = format!("{name}_count");
    if old[&hash_key] == new[&hash_key] && old[&count_key] == new[&count_key] {
        return Vec::new();
    }
    vec![object([
        ("kind", val("Fingerprint")),
        ("path", val(format!("$complete_{name}_audit"))),
        (
            "old",
            object([
                ("sha256", val(old[&hash_key].clone())),
                ("count", val(old[&count_key].clone())),
            ]),
        ),
        (
            "new",
            object([
                ("sha256", val(new[&hash_key].clone())),
                ("count", val(new[&count_key].clone())),
            ]),
        ),
        ("old_present", val(true)),
        ("new_present", val(true)),
    ])]
}
fn pair_prepared(
    test: &str,
    index: usize,
    (a, b): (&Value, &Value),
    reasons: &Value,
    structure: &mut Option<Structure>,
    counts: &mut Counts,
    prepared_money: impl FnOnce() -> Result<(MoneyMap, MoneyMap)>,
) -> Result<(Json, bool)> {
    counts.add("pairs_compared", 1);
    let kind_mismatch = a["kind"] != b["kind"];
    let compact_mode = a["version"] == 2
        || b["version"] == 2
        || crate::compact::oversized_debug(a)
        || crate::compact::oversized_debug(b);
    if compact_mode && structure.is_some() {
        if crate::compact::oversized_debug(a)
            || crate::compact::oversized_debug(b)
            || crate::compact::size_limited(a)
            || crate::compact::size_limited(b)
        {
            return fail(
                "structural mapping requires complete Debug; size-limited Debug was omitted",
            );
        }
        return fail(
            "structural mapping requires complete legacy Debug captures, not compact captures",
        );
    }
    let compact_old = compact_mode.then(|| compact_row(a)).transpose()?;
    let compact_new = compact_mode.then(|| compact_row(b)).transpose()?;
    let compact_pair = compact_old.as_deref().zip(compact_new.as_deref());
    let identical = if let Some((old, new)) = compact_pair {
        old["debug_sha256"] == new["debug_sha256"] && old["debug_bytes"] == new["debug_bytes"]
    } else {
        a.get("debug").ok_or("missing old Debug payload")?
            == b.get("debug").ok_or("missing new Debug payload")?
    };
    if !identical && let Some((old, new)) = compact_pair {
        for row in [old, new] {
            if let Some(error) = row["audit_error"].as_str()
                && !crate::compact::size_limited(row)
            {
                return fail(format!("{test}#{index}: cannot audit changed Debug: {error}"));
            }
        }
    }
    let size_only = compact_pair.is_some_and(|(old, new)| {
        crate::compact::size_limited(old) || crate::compact::size_limited(new)
    });
    let unavailable = size_only && !identical;
    if size_only {
        let (old, new) = compact_pair.expect("size evidence requires compact observations");
        counts.add("pairs_with_size_limited_debug", 1);
        counts.size_evidence.push(object([
            ("test", val(test)),
            ("index", val(index as u64)),
            ("audit_evidence", val("digest-only (size)")),
            ("identical_debug", val(identical)),
            ("old_debug_bytes", val(old["debug_bytes"].clone())),
            ("new_debug_bytes", val(new["debug_bytes"].clone())),
            ("old_debug_sha256", val(old["debug_sha256"].clone())),
            ("new_debug_sha256", val(new["debug_sha256"].clone())),
        ]));
        if unavailable {
            counts.add("pairs_with_unavailable_numeric_audit", 1);
            counts.add("pairs_with_unavailable_currency_audit", 1);
        }
    }
    let fingerprint_only = compact_pair.is_some_and(|(old, new)| {
        !old["debug_retained"].as_bool().unwrap_or(false)
            || !new["debug_retained"].as_bool().unwrap_or(false)
    });
    let (numbers, currencies, structural) = if identical {
        counts.add("pairs_with_identical_debug", 1);
        (Vec::new(), Vec::new(), Vec::new())
    } else if unavailable {
        (Vec::new(), Vec::new(), Vec::new())
    } else if fingerprint_only {
        let (old, new) = compact_pair.expect("fingerprints require compact observations");
        (
            fingerprint_delta(old, new, "numeric"),
            fingerprint_delta(old, new, "currency"),
            Vec::new(),
        )
    } else {
        let calculate = (|| -> Result<_> {
            let old_text = a["debug"].as_str().ok_or("missing Debug payload")?;
            let new_text = b["debug"].as_str().ok_or("missing Debug payload")?;
            // Large captures compare corresponding trees without materializing
            // an owned full path for every unchanged number. This selects an
            // equivalent algorithm; no payload or audit value is omitted.
            if structure.is_none() && old_text.len().max(new_text.len()) > 1024 * 1024 {
                let (numbers, currencies) = debug::large_numeric_differences(old_text, new_text)?;
                return Ok((numbers, currencies, Vec::new()));
            }
            let (old, new) = if let Some(map) = structure {
                let mut old = debug::Parser::parse(old_text)?;
                let mut new = debug::Parser::parse(new_text)?;
                map.apply(&mut old, true)?;
                map.apply(&mut new, false)?;
                debug::normalize(&mut old)?;
                debug::normalize(&mut new)?;
                (
                    std::sync::Arc::new(debug::views(&old, true)?),
                    std::sync::Arc::new(debug::views(&new, true)?),
                )
            } else {
                (debug::cached_numeric_views(old_text)?, debug::cached_numeric_views(new_text)?)
            };
            Ok((
                delta(&old.numeric, &new.numeric),
                currency_delta(&old.currency, &new.currency),
                delta(&old.structural, &new.structural),
            ))
        })();
        calculate.map_err(|error| format!("{test}#{index}: {error}"))?
    };
    let (old_money, new_money) = prepared_money()?;
    let (keys, monetary) = if Arc::ptr_eq(&old_money, &new_money) {
        (old_money.len(), Vec::new())
    } else {
        let keys =
            old_money.len() + new_money.keys().filter(|key| !old_money.contains_key(*key)).count();
        (keys, delta(&old_money, &new_money))
    };
    let changed = unavailable
        || kind_mismatch
        || !monetary.is_empty()
        || !currencies.is_empty()
        || !numbers.is_empty()
        || !structural.is_empty();
    for (key, count) in [
        ("pairs_with_estimate_money_changes", (!monetary.is_empty()) as u64),
        ("pairs_with_currency_changes", (!currencies.is_empty()) as u64),
        ("typed_money_keys_compared", keys as u64),
        ("pairs_with_numeric_audit_changes", (!numbers.is_empty()) as u64),
        ("pairs_with_kind_mismatch", kind_mismatch as u64),
        ("pairs_with_structural_changes", (!structural.is_empty()) as u64),
    ] {
        if !unavailable
            || !matches!(key, "pairs_with_currency_changes" | "pairs_with_numeric_audit_changes")
        {
            counts.add(key, count);
        }
    }
    let reason = reasons
        .as_object()
        .ok_or("reasons must be an object")?
        .get(&format!("{test}#{index}"))
        .cloned()
        .unwrap_or(Value::Null);
    if changed && !truthy(&reason) {
        counts.add("unexplained_pairs", 1);
    }
    let audit = if unavailable {
        vec![
            ("structural_changes", Json::Array(structural)),
            ("numeric_audit", val(Value::Null)),
            ("currency_changes", val(Value::Null)),
        ]
    } else if identical {
        vec![
            ("numeric_audit", Json::Array(numbers)),
            ("currency_changes", Json::Array(currencies)),
            ("structural_changes", Json::Array(structural)),
        ]
    } else {
        vec![
            ("structural_changes", Json::Array(structural)),
            ("numeric_audit", Json::Array(numbers)),
            ("currency_changes", Json::Array(currencies)),
        ]
    };
    let mut fields = vec![
        ("test", val(test)),
        ("index", val(index as u64)),
        ("old_kind", val(a["kind"].clone())),
        ("new_kind", val(b["kind"].clone())),
        ("kind_mismatch", val(kind_mismatch)),
        ("identical_debug", val(identical)),
    ];
    fields.extend(audit);
    if let Some((old, new)) = compact_pair {
        fields.push((
            "audit_evidence",
            object([
                (
                    "numeric",
                    val(if size_only {
                        "digest-only (size)"
                    } else if identical {
                        "identical-debug-digest"
                    } else if fingerprint_only {
                        "complete-fingerprint"
                    } else {
                        "complete-values"
                    }),
                ),
                (
                    "currency",
                    val(if size_only {
                        "digest-only (size)"
                    } else if identical {
                        "identical-debug-digest"
                    } else if fingerprint_only {
                        "complete-fingerprint"
                    } else {
                        "complete-values"
                    }),
                ),
                ("old_debug_sha256", val(old["debug_sha256"].clone())),
                ("new_debug_sha256", val(new["debug_sha256"].clone())),
                ("old_omission_reason", val(old["omission_reason"].clone())),
                ("new_omission_reason", val(new["omission_reason"].clone())),
            ]),
        ));
        if size_only {
            fields.push(("old_debug_bytes", val(old["debug_bytes"].clone())));
            fields.push(("new_debug_bytes", val(new["debug_bytes"].clone())));
        }
        if !identical && monetary.is_empty() {
            fields.push((
                "diagnostic_debug",
                object([("old", val(old["debug"].clone())), ("new", val(new["debug"].clone()))]),
            ));
        }
    }
    fields.extend([
        ("money_keys_compared", val(keys as u64)),
        ("money_changes", Json::Array(monetary)),
        ("declared_reason", val(reason)),
    ]);
    Ok((object(fields), !identical || changed))
}
// Tests use the same comparison owner without the indexed input adapter.
#[cfg(test)]
fn pair(
    test: &str,
    index: usize,
    rows: (&Value, &Value),
    mapping: &Value,
    reasons: &Value,
    structure: &mut Option<Structure>,
    counts: &mut Counts,
) -> Result<(Json, bool)> {
    pair_prepared(test, index, rows, reasons, structure, counts, || {
        Ok((Arc::new(money(rows.0, mapping)?), Arc::new(money(rows.1, &json!({}))?)))
    })
}

type MoneyMap = Arc<BTreeMap<String, Value>>;
type MoneyKey = ([u8; 32], [u8; 32]);
#[derive(Default)]
struct MoneyCache {
    entries: std::collections::VecDeque<(MoneyKey, MoneyMap, usize)>,
    bytes: usize,
}
impl MoneyCache {
    const BYTES: usize = 8 * 1024 * 1024;
    const ENTRIES: usize = 128;
    fn get(&mut self, key: &MoneyKey) -> Option<MoneyMap> {
        let index = self.entries.iter().position(|(stored, _, _)| stored == key)?;
        let entry = self.entries.remove(index)?;
        let result = entry.1.clone();
        self.entries.push_back(entry);
        Some(result)
    }
    fn insert(&mut self, key: MoneyKey, values: &MoneyMap) -> Result<()> {
        // Include owned paths, nested values, tree nodes and allocator overhead.
        // Only normalized projections are retained, never complete observations.
        let bytes = values.iter().try_fold(0usize, |sum, (path, value)| -> Result<usize> {
            Ok(sum.saturating_add(path.len()).saturating_add(
                serde_json::to_vec(value)?.len().saturating_mul(16).saturating_add(256),
            ))
        })?;
        if bytes > Self::BYTES {
            return Ok(());
        }
        while self.bytes.saturating_add(bytes) > Self::BYTES || self.entries.len() >= Self::ENTRIES
        {
            if let Some((_, _, removed)) = self.entries.pop_front() {
                self.bytes -= removed;
            }
        }
        self.entries.push_back((key, values.clone(), bytes));
        self.bytes += bytes;
        Ok(())
    }
    fn normalized<'a>(
        &mut self,
        entry: &crate::capture::Entry,
        mapping: &Value,
        mapping_hash: [u8; 32],
        values: impl FnOnce() -> Result<Cow<'a, Value>>,
    ) -> Result<MoneyMap> {
        if entry.kind != "Estimate" {
            return Ok(Arc::new(BTreeMap::new()));
        }
        let key = entry.money_hash.map(|hash| (hash, mapping_hash));
        if let Some(values) = key.as_ref().and_then(|key| self.get(key)) {
            return Ok(values);
        }
        let values = Arc::new(project_money_values(values()?.as_ref(), mapping)?);
        if let Some(key) = key {
            self.insert(key, &values)?;
        }
        Ok(values)
    }
}
#[derive(Clone, Copy)]
struct Chunk {
    test: usize,
    start: usize,
    end: usize,
    weight: u64,
}
struct TestResult {
    path: PathBuf,
    counts: Counts,
    applied: Vec<(String, u64)>,
}
// The sliding completion window bounds completed spool files even when an early
// chunk is slow. Memory weights limit concurrent AST expansion, not input size.
struct Admission {
    next: usize,
    completed: usize,
    active_bytes: u64,
    stopped: bool,
}
struct Scheduler {
    state: Mutex<Admission>,
    changed: Condvar,
    budget: u64,
    window: usize,
}
impl Scheduler {
    fn take(&self, chunks: &[Chunk]) -> Option<(usize, Chunk)> {
        let mut state = self.state.lock().unwrap();
        loop {
            if state.stopped || state.next == chunks.len() {
                return None;
            }
            let chunk = chunks[state.next];
            let weight = chunk.weight.min(self.budget);
            if state.next < state.completed.saturating_add(self.window)
                && state.active_bytes.saturating_add(weight) <= self.budget
            {
                let index = state.next;
                state.next += 1;
                state.active_bytes += weight;
                return Some((index, chunk));
            }
            state = self.changed.wait(state).unwrap();
        }
    }
    fn release(&self, chunk: Chunk) {
        let mut state = self.state.lock().unwrap();
        state.active_bytes -= chunk.weight.min(self.budget);
        self.changed.notify_all();
    }
    fn complete(&self, completed: usize) {
        self.state.lock().unwrap().completed = completed;
        self.changed.notify_all();
    }
    fn stop(&self) {
        self.state.lock().unwrap().stopped = true;
        self.changed.notify_all();
    }
}
fn raw_identical(a: &crate::capture::Entry, b: &crate::capture::Entry) -> bool {
    a.version == Some(1)
        && b.version == Some(1)
        && a.debug_hash.is_some()
        && a.debug_hash == b.debug_hash
        && a.debug_span.is_some_and(|span| span.len < crate::compact::DEBUG_PARSE_LIMIT)
        && b.debug_span.is_some_and(|span| span.len < crate::compact::DEBUG_PARSE_LIMIT)
}
#[allow(clippy::too_many_arguments)]
fn chunk_compare(
    test: &str,
    number: usize,
    chunk: Chunk,
    (old, new): (&Capture, &Capture),
    mappings: &Value,
    reasons: &Value,
    mut structure: Option<Structure>,
    output: (&Path, bool),
    cache: &mut MoneyCache,
) -> Result<TestResult> {
    use sha2::{Digest, Sha256};
    let (temp, differences_only) = output;
    let a = old.tests.get(test).map_or(&[][..], Vec::as_slice);
    let b = new.tests.get(test).map_or(&[][..], Vec::as_slice);
    let path = temp.join(format!("pairs-{number}"));
    let mut stream = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(&path)?);
    let mut counts = Counts::default();
    let mut old_reader = old.reader()?;
    let mut new_reader = new.reader()?;
    if chunk.start == chunk.end {
        stream.flush()?;
        return Ok(TestResult {
            path,
            counts,
            applied: structure.map_or(Vec::new(), |map| map.applied),
        });
    }
    let empty_map = json!({});
    let mappings = mappings.as_object().ok_or("line maps must be an object")?;
    let mapping = mappings.get(test).or_else(|| mappings.get("*")).unwrap_or(&empty_map);
    let mapping_hash: [u8; 32] = Sha256::digest(serde_json::to_vec(mapping)?).into();
    let empty_hash: [u8; 32] = Sha256::digest(b"{}").into();
    for index in chunk.start..chunk.end {
        let (a, b) = (&a[index], &b[index]);
        if a.len.max(b.len) >= 1024 * 1024 {
            eprintln!(
                "observation test={test} index={index} old_bytes={} new_bytes={}",
                a.len, b.len
            );
        }
        // Raw hashes cover JSON value bytes including quotes. Differently encoded
        // equal strings fall back to decoded equality; compact null is no proof.
        let (left, right) = if raw_identical(a, b) {
            (json!({"kind":a.kind,"debug":""}), json!({"kind":b.kind,"debug":""}))
        } else {
            (old_reader.event(a)?, new_reader.event(b)?)
        };
        let prepared_money = || {
            let old_money = cache.normalized(a, mapping, mapping_hash, || {
                if let Some(values) = left.get("money") {
                    Ok(Cow::Borrowed(values))
                } else if a.money_span.is_some() {
                    Ok(Cow::Owned(old_reader.money(a)?))
                } else {
                    let mut row = old_reader.event(a)?;
                    Ok(Cow::Owned(row.get_mut("money").map(Value::take).unwrap_or(Value::Null)))
                }
            })?;
            let new_money = if a.kind == b.kind
                && mapping_hash == empty_hash
                && a.money_hash.is_some()
                && a.money_hash == b.money_hash
            {
                old_money.clone()
            } else {
                cache.normalized(b, &empty_map, empty_hash, || {
                    if let Some(values) = right.get("money") {
                        Ok(Cow::Borrowed(values))
                    } else if b.money_span.is_some() {
                        Ok(Cow::Owned(new_reader.money(b)?))
                    } else {
                        let mut row = new_reader.event(b)?;
                        Ok(Cow::Owned(row.get_mut("money").map(Value::take).unwrap_or(Value::Null)))
                    }
                })?
            };
            Ok((old_money, new_money))
        };
        let (item, differs) = pair_prepared(
            test,
            index,
            (&left, &right),
            reasons,
            &mut structure,
            &mut counts,
            prepared_money,
        )?;
        if !differences_only || differs {
            item.write_to(&mut stream, None)?;
            stream.write_all(b"\n")?;
        }
    }
    stream.flush()?;
    Ok(TestResult { path, counts, applied: structure.map_or(Vec::new(), |map| map.applied) })
}
pub fn run(args: &Args) -> Result<i32> {
    if args.options.contains_key("index") {
        return fail(
            "legacy SQLite --index is unsupported: Rust builds an in-memory capture-offset index from --old and --new",
        );
    }
    if args.shard.1 <= 0 || args.shard.0 < 0 || args.shard.0 >= args.shard.1 {
        return fail("test shard requires 0 <= INDEX < COUNT");
    }
    let output = args.required("output")?;
    let temp =
        TempDir::new(std::path::Path::new(output).parent().unwrap_or(std::path::Path::new(".")))?;
    let structure_path = args.options.get("structure-map");
    if structure_path.is_some()
        && (args.options.contains_key("line-map") || args.options.contains_key("reasons"))
    {
        return fail("strict structural comparison disallows reasons and line-ID overrides");
    }
    let structure = structure_path.map(|path| Structure::read(path)).transpose()?;
    let mappings = args.read_json("line-map")?;
    let reasons = args.read_json("reasons")?;
    let indexing = crate::timing::PhaseTimer::new("index");
    let cache_dir = args.options.get("capture-index-dir").map(Path::new);
    let index_jobs = args.jobs.div_ceil(2);
    let (mut old, mut new) = std::thread::scope(|scope| {
        let a = scope.spawn(|| {
            Capture::index_cached_parallel(
                args.required("old")?,
                &temp.0,
                true,
                cache_dir,
                index_jobs,
            )
        });
        let b = Capture::index_cached_parallel(
            args.required("new")?,
            &temp.0,
            true,
            cache_dir,
            index_jobs,
        );
        Ok::<_, crate::Error>((a.join().map_err(|_| "capture indexing worker panicked")??, b?))
    })?;
    crate::capture::exclude_capacity(&mut old, &mut new);
    indexing.finish();
    let pairing = crate::timing::PhaseTimer::new("pair");
    let tests: Vec<_> = old
        .tests
        .keys()
        .chain(new.tests.keys())
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .filter(|(index, _)| index % args.shard.1 as usize == args.shard.0 as usize)
        .map(|(_, name)| name)
        .collect();
    let chunks: Vec<_> = tests
        .iter()
        .enumerate()
        .flat_map(|(test, name)| {
            let a = old.tests.get(name).map_or(&[][..], Vec::as_slice);
            let b = new.tests.get(name).map_or(&[][..], Vec::as_slice);
            let paired = a.len().min(b.len());
            (0..paired.max(1)).step_by(32).map(move |start| {
                let end = (start + 32).min(paired);
                let raw =
                    (start..end).map(|index| a[index].len.max(b[index].len)).max().unwrap_or(0);
                Chunk { test, start, end, weight: raw.saturating_mul(512).max(32 * 1024 * 1024) }
            })
        })
        .collect();
    let budget = args
        .options
        .get("memory-budget-mib")
        .map(|n| n.parse::<u64>())
        .transpose()?
        .unwrap_or(10 * 1024)
        .saturating_mul(1024 * 1024);
    let scheduler = Scheduler {
        state: Mutex::new(Admission { next: 0, completed: 0, active_bytes: 0, stopped: false }),
        changed: Condvar::new(),
        budget,
        window: args.jobs.saturating_mul(2).max(1),
    };
    let codec = args.options.get("pairs-compression").map_or("zstd", String::as_str);
    let extension = if codec == "gzip" { "gz" } else { "zst" };
    let output_file = File::create(format!("{output}.pairs.jsonl.{extension}"))?;
    let mut compressor = Command::new(if codec == "gzip" { "gzip" } else { "zstd" });
    compressor.args(if codec == "gzip" {
        &["-n", "-9", "-c"][..]
    } else {
        &["-q", "-9", "--long=27", "--check", "-T1", "-c"][..]
    });
    let mut compressor = compressor.stdin(Stdio::piped()).stdout(output_file).spawn()?;
    let mut pair_output =
        BufWriter::new(compressor.stdin.take().ok_or("missing compressor stdin")?);
    let mut counts = Counts::default();
    let mut applied = Counts::default();
    let mut selected = Vec::new();
    let mut unmatched = Vec::new();
    let mut old_only = 0u64;
    let mut new_only = 0u64;
    let (send, receive) = std::sync::mpsc::sync_channel(args.jobs.max(1));
    let analysis = std::thread::scope(|scope| -> Result<()> {
        for _ in 0..args.jobs.min(chunks.len().max(1)) {
            let send = send.clone();
            let (tests, chunks, scheduler) = (&tests, &chunks, &scheduler);
            let (old, new, mappings, reasons, structure, temp) =
                (&old, &new, &mappings, &reasons, &structure, &temp.0);
            scope.spawn(move || {
                let mut cache = MoneyCache::default();
                while let Some((index, chunk)) = scheduler.take(chunks) {
                    let result = chunk_compare(
                        &tests[chunk.test],
                        index,
                        chunk,
                        (old, new),
                        mappings,
                        reasons,
                        structure.clone(),
                        (temp, args.differences_only),
                        &mut cache,
                    );
                    scheduler.release(chunk);
                    if send.send((index, result)).is_err() {
                        break;
                    }
                }
            });
        }
        drop(send);
        let mut pending = BTreeMap::new();
        let mut expected = 0;
        // Drain on errors so blocked senders can exit before the scoped join.
        let mut error = None;
        for (index, result) in receive {
            if error.is_some() {
                continue;
            }
            pending.insert(index, result);
            let merge = (|| -> Result<()> {
                while let Some(result) = pending.remove(&expected) {
                    let result = result?;
                    counts.merge(&result.counts);
                    for (key, count) in result.applied {
                        applied.add(&key, count);
                    }
                    std::io::copy(
                        &mut BufReader::new(File::open(&result.path)?),
                        &mut pair_output,
                    )?;
                    std::fs::remove_file(result.path)?;
                    expected += 1;
                    scheduler.complete(expected);
                }
                Ok(())
            })();
            if let Err(failure) = merge {
                scheduler.stop();
                error = Some(failure);
            }
        }
        if let Some(error) = error {
            return Err(error);
        }
        if expected != chunks.len() {
            return fail("comparison worker ended without a result");
        }
        Ok(())
    });
    if analysis.is_err() {
        let _ = compressor.kill();
    }
    let flushed = pair_output.flush();
    drop(pair_output);
    let status = compressor.wait()?;
    analysis?;
    flushed?;
    if !status.success() {
        return fail("pair output compression failed");
    }
    pairing.finish();
    for test in &tests {
        let a = old.tests.get(test).map_or(&[][..], Vec::as_slice);
        let b = new.tests.get(test).map_or(&[][..], Vec::as_slice);
        let paired = a.len().min(b.len());
        selected.push((
            test.clone(),
            object([("old", val(a.len() as u64)), ("new", val(b.len() as u64))]),
        ));
        let leftovers = |entries: &[crate::capture::Entry]| {
            Json::Array(
                entries
                    .iter()
                    .enumerate()
                    .skip(paired)
                    .map(|(index, e)| {
                        object([("index", val(index as u64)), ("kind", val(e.kind.clone()))])
                    })
                    .collect(),
            )
        };
        if a.len() != b.len() {
            unmatched.push((
                test.clone(),
                object([("old_only", leftovers(a)), ("new_only", leftovers(b))]),
            ));
        }
        old_only += (a.len() - paired) as u64;
        new_only += (b.len() - paired) as u64;
    }
    counts.add("old_only_events", old_only);
    counts.add("new_only_events", new_only);
    let structure_report = if let Some(path) = structure_path {
        object([("path", val(path.clone())), ("applied", applied.json())])
    } else {
        val(Value::Null)
    };
    let mut report = object([
        ("counts", counts.json()),
        ("unmatched_by_test", object(unmatched)),
        ("structure_mapping", structure_report),
        (
            "test_shard",
            object([("index", val(args.shard.0 as u64)), ("count", val(args.shard.1 as u64))]),
        ),
        ("tests_selected", object(selected)),
        (
            "limitations",
            val(json!([
                "Byte-identical complete Debug payloads prove all contained numbers unchanged without parsing.",
                "Debug numeric audit is a superset: IDs, counts, dates, encoded text bytes are NOT automatically tax figures.",
                if counts.size_evidence.is_empty() {
                    "Only explicit Estimate Money lines and top fields are typed money; nested financial carriers remain in the complete numeric audit."
                } else {
                    "Only explicit Estimate Money lines and top fields are typed money; changed size-limited Debug has unavailable numeric and Currency audits and requires review."
                },
                "Pairing is strictly same test and completion ordinal; kind mismatches are never resynchronized.",
                "CalculationLine numeric and currency subtrees match by exact stable id and ordered occurrence; no duplicate amount is dropped.",
                "Every other collection retains its order; explicit mappings cannot merge distinct line identities.",
                "Added/removed events need review; changed test call sequences can shift later ordinal pairs.",
                "No figure-equivalence claim until every delta, mapping and unmatched event is reviewed."
            ])),
        ),
    ]);
    if !counts.size_evidence.is_empty()
        && let Json::Object(fields) = &mut report
    {
        fields.push(("digest_only_size".into(), Json::Array(counts.size_evidence.clone())));
    }
    crate::capture::add_exclusions(&mut report, &old, &new);
    std::fs::write(format!("{output}.summary.json"), format!("{}\n", report.render(Some(2))))?;
    let sorted: BTreeMap<_, _> = counts.values.iter().cloned().collect();
    // Python's stdout uses default separators even though pair records are compact.
    println!("{}", crate::spaced_json(&serde_json::to_value(sorted)?));
    Ok(i32::from(counts.get("unexplained_pairs") > 0 || old_only > 0 || new_only > 0))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn event(debug: &str, lines: Value) -> Value {
        json!({"kind":"Estimate","debug":debug,"money":{"top":{"tax":100},"return_rows":[],"lines":lines}})
    }
    #[test]
    fn one_cent_and_ordered_duplicates_survive() {
        let a = event(
            "opaque @",
            json!([{"id":"tax","kind":"Money","amount":10001},{"id":"tax","kind":"Money","amount":20000}]),
        );
        let b = event(
            "opaque @",
            json!([{"id":"tax","kind":"Money","amount":10002},{"id":"tax","kind":"Money","amount":19999}]),
        );
        let mut counts = Counts::default();
        let result =
            pair("case", 0, (&a, &b), &json!({}), &json!({}), &mut None, &mut counts).unwrap();
        let result: Value = serde_json::from_str(&result.0.render(None)).unwrap();
        assert_eq!(result["money_changes"][0]["path"], "line.tax#0");
        assert_eq!(result["money_changes"][1]["old"], 20000);
        assert_eq!(counts.get("unexplained_pairs"), 1);
    }
    #[test]
    fn normalized_money_cache_reuses_owner_projection_and_bounds_retention() {
        let mut cache = MoneyCache::default();
        let mapping = json!({});
        let row = event(
            "None",
            json!([
                {"id":"tax","kind":"Money","amount":10001},
                {"id":"tax","kind":"Money","amount":20000}
            ]),
        );
        let entry = crate::capture::Entry {
            kind: "Estimate".into(),
            money_hash: Some([1; 32]),
            ..Default::default()
        };
        let first = cache
            .normalized(&entry, &mapping, [0; 32], || Ok(Cow::Borrowed(&row["money"])))
            .unwrap();
        assert_eq!(*first, money(&row, &mapping).unwrap());
        let reused = cache
            .normalized(&entry, &mapping, [0; 32], || {
                panic!("an identical raw money block must not be decoded again")
            })
            .unwrap();
        assert!(Arc::ptr_eq(&first, &reused));
        for n in 0..256u16 {
            let mut key = [0; 32];
            key[..2].copy_from_slice(&n.to_le_bytes());
            cache.insert((key, [2; 32]), &first).unwrap();
        }
        assert_eq!(cache.entries.len(), MoneyCache::ENTRIES);
        assert!(cache.bytes <= MoneyCache::BYTES);
        assert!(cache.get(&([1; 32], [0; 32])).is_none());
        let oversized =
            Arc::new(BTreeMap::from([("large".into(), json!("x".repeat(MoneyCache::BYTES)))]));
        cache.insert(([9; 32], [3; 32]), &oversized).unwrap();
        assert!(cache.get(&([9; 32], [3; 32])).is_none());
        assert!(cache.bytes <= MoneyCache::BYTES);
    }
    #[test]
    fn distinct_line_ids_cannot_collide() {
        let row = event(
            "None",
            json!([{"id":"old","kind":"Money","amount":1},{"id":"new","kind":"Money","amount":1}]),
        );
        assert!(
            money(&row, &json!({"old":"new"}))
                .unwrap_err()
                .to_string()
                .contains("different line identities")
        );
    }
}

#[cfg(test)]
mod currency_order_tests {
    use super::*;
    #[test]
    fn debug_currency_projection_and_capture_money_keep_their_distinct_orders() {
        let before = serde_json::json!({"currency_code":"USD","minor_units":10001,"scale":2});
        let after = serde_json::json!({"currency_code":"USD","minor_units":10002,"scale":2});
        let a = serde_json::json!({"kind":"Estimate","debug":"Reply { cash: Currency { currency_code: USD, scale: 2, minor_units: 10001 } }","money":{"top":{},"return_rows":[{"amount":10001,"money":before}],"lines":[]}});
        let mut b = a.clone();
        b["debug"] =
            "Reply { cash: Currency { currency_code: USD, scale: 2, minor_units: 10002 } }".into();
        b["money"]["return_rows"][0]["money"] = after;
        let mut counts = Counts::default();
        let text = pair("case", 0, (&a, &b), &json!({}), &json!({}), &mut None, &mut counts)
            .unwrap()
            .0
            .render(None);
        assert!(text.contains(r#""old":{"currency_code":{"$tag":"USD"},"scale":{"$number":"2"},"minor_units":{"$number":"10001"}}"#));
        assert!(text.contains(r#""old":{"currency_code":"USD","minor_units":10001,"scale":2}"#));
    }
}
