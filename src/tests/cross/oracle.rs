// Test-only figure oracle. Baselines change only for declared behavior changes.
// Result encoding v1 hashes full derived Debug, including private provenance;
// a Rust representation change therefore also requires an explicit baseline change.
extern crate std;

#[path = "monetary_capture.rs"]
mod monetary_capture;

use crate::interview::{Interview, Reply};
use crate::{FederalDraft, FederalResult, TaxInterviewProtocol as P};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write as IoWrite};
use std::path::{Path, PathBuf};
use std::string::{String, ToString};
use std::sync::{
    OnceLock,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};
use std::{format, vec::Vec};

std::thread_local! { static REPLAYING: Cell<bool> = const { Cell::new(false) }; }
static CAPTURE: OnceLock<Option<PathBuf>> = OnceLock::new();
static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Session {
    user: libertas::LibertasUser,
    draft: String,
    flags: [bool; 3],
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Input {
    Estimate { draft: String },
    Send { session: Session, request: String },
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Output {
    Estimate { result: String },
    Send { kind: String, reply: String, state: String },
}
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Case {
    version: u32,
    input: Input,
    expected: Output,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tests: Vec<String>,
}

struct HashWriter(Sha256);
impl fmt::Write for HashWriter {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.0.update(value.as_bytes());
        Ok(())
    }
}
fn hex(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(text, "{byte:02x}").unwrap();
    }
    text
}
fn digest(domain: &str, bytes: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(domain.as_bytes());
    hash.update([0]);
    hash.update(bytes);
    hex(&hash.finalize())
}
fn result_digest(result: &FederalResult) -> String {
    let mut writer = HashWriter(Sha256::new());
    writer.write_str("tax-oracle-v1/full-result-debug\0").unwrap();
    write!(&mut writer, "{result:?}").unwrap();
    hex(&writer.0.finalize())
}
fn capture_dir() -> Option<&'static Path> {
    if REPLAYING.get() {
        return None;
    }
    CAPTURE
        .get_or_init(|| {
            std::env::var_os("TAX_ORACLE_CAPTURE").map(|v| {
                let path = PathBuf::from(v);
                assert!(
                    path.is_absolute(),
                    "TAX_ORACLE_CAPTURE must be an absolute on-disk directory"
                );
                path
            })
        })
        .as_deref()
}
// Publish complete files atomically across test threads and processes. Never overwrite
// a baseline: identical inputs yielding different outputs are a capture failure.
// Paths this process already published compare by content hash in memory, without I/O.
fn publish(path: &Path, bytes: &[u8]) {
    static PUBLISHED: std::sync::Mutex<BTreeMap<PathBuf, [u8; 32]>> =
        std::sync::Mutex::new(BTreeMap::new());
    let hash: [u8; 32] = Sha256::digest(bytes).into();
    if let Some(seen) = PUBLISHED.lock().unwrap().get(path) {
        assert_eq!(*seen, hash, "oracle collision at {}", path.display());
        return;
    }
    publish_file(path, bytes);
    PUBLISHED.lock().unwrap().insert(path.to_path_buf(), hash);
}
fn publish_file(path: &Path, bytes: &[u8]) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    if path.exists() {
        assert_eq!(fs::read(path).unwrap(), bytes, "oracle collision at {}", path.display());
        return;
    }
    let temporary = path.with_extension(format!(
        "{}-{}.tmp",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary).unwrap();
    file.write_all(bytes).unwrap();
    drop(file);
    let linked = fs::hard_link(&temporary, path);
    fs::remove_file(&temporary).unwrap();
    match linked {
        Ok(()) => (),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            assert_eq!(fs::read(path).unwrap(), bytes, "oracle collision at {}", path.display());
        }
        Err(error) => panic!("cannot publish {}: {error}", path.display()),
    }
}
fn blob(root: &Path, bytes: &[u8]) -> String {
    let id = digest("tax-oracle-v1/avro", bytes);
    publish(&root.join("inputs").join(format!("{id}.avro")), bytes);
    id
}
fn save(root: &Path, input: Input, expected: Output) {
    let id = digest("tax-oracle-v1/input", &serde_json::to_vec(&input).unwrap());
    let case = Case { version: 1, input, expected, tests: Vec::new() };
    publish(&root.join("cases").join(format!("{id}.json")), &serde_json::to_vec(&case).unwrap());
    let thread = std::thread::current();
    let test = thread.name().expect("oracle capture requires a named test thread");
    assert!(test.contains("::"), "oracle capture requires a test origin");
    let origin = serde_json::to_vec(&(id.as_str(), test)).unwrap();
    let origin_id = digest("tax-oracle-v1/origin", &origin);
    publish(&root.join("origins").join(format!("{origin_id}.json")), &origin);
}
pub(crate) fn record_estimate(draft: &FederalDraft, result: &FederalResult) {
    monetary_capture::estimate(result);
    let Some(root) = capture_dir() else {
        return;
    };
    let input = Input::Estimate { draft: blob(root, &draft.to_avro()) };
    save(root, input, Output::Estimate { result: result_digest(result) });
}
pub(crate) fn begin_send(session: &Interview, request: &P) -> Option<InputCapture> {
    let root = capture_dir()?;
    Some(InputCapture(Input::Send {
        session: Session {
            user: session.user,
            draft: blob(root, &session.draft.to_avro()),
            flags: session.oracle_flags(),
        },
        request: blob(root, &request.to_avro()),
    }))
}
pub(crate) struct InputCapture(Input);
fn send_output(session: &Interview, reply: &Reply) -> Output {
    let (kind, value) = match reply {
        Reply::Response(p) => ("Response", p),
        Reply::Edit(p) => ("Edit", p),
    };
    let state = Session {
        user: session.user,
        draft: digest("tax-oracle-v1/avro", &session.draft.to_avro()),
        flags: session.oracle_flags(),
    };
    Output::Send {
        kind: kind.to_string(),
        reply: digest("tax-oracle-v1/reply-avro", &value.to_avro()),
        state: digest("tax-oracle-v1/session", &serde_json::to_vec(&state).unwrap()),
    }
}
pub(crate) fn finish_send(capture: Option<InputCapture>, session: &Interview, reply: &Reply) {
    monetary_capture::send(session, reply);
    if let Some(InputCapture(input)) = capture {
        save(capture_dir().unwrap(), input, send_output(session, reply));
    }
}
struct ReplayGuard(bool);
impl ReplayGuard {
    fn new() -> Self {
        Self(REPLAYING.replace(true))
    }
}
impl Drop for ReplayGuard {
    fn drop(&mut self) {
        REPLAYING.set(self.0);
    }
}
// zstd stream: header, u64-LE input count, then length-prefixed records.
// Each input record is its 64-byte ASCII ID followed by native Avro bytes;
// the last record is the JSON digest manifest. No outputs are stored.
const HEADER: &[u8] = b"TAXORACLE01\n";
struct Corpus {
    inputs: BTreeMap<String, Vec<u8>>,
    cases: Vec<Case>,
}
fn read_u64(reader: &mut impl Read) -> u64 {
    let mut bytes = [0; 8];
    reader.read_exact(&mut bytes).unwrap();
    u64::from_le_bytes(bytes)
}
fn read_record(reader: &mut impl Read) -> Vec<u8> {
    let length = usize::try_from(read_u64(reader)).unwrap();
    let mut bytes = std::vec![0; length];
    reader.read_exact(&mut bytes).unwrap();
    bytes
}
fn write_record(writer: &mut impl IoWrite, bytes: &[u8]) {
    writer.write_all(&u64::try_from(bytes.len()).unwrap().to_le_bytes()).unwrap();
    writer.write_all(bytes).unwrap();
}
fn load_corpus(path: &Path) -> Corpus {
    let mut reader = zstd::stream::read::Decoder::new(fs::File::open(path).unwrap()).unwrap();
    let mut header = [0; 12];
    reader.read_exact(&mut header).unwrap();
    assert_eq!(header, HEADER, "unsupported oracle stream");
    let count = read_u64(&mut reader);
    let mut inputs = BTreeMap::new();
    for _ in 0..count {
        let record = read_record(&mut reader);
        let (id, bytes) = record.split_at(64);
        let id = std::str::from_utf8(id).unwrap();
        assert_eq!(digest("tax-oracle-v1/avro", bytes), id);
        assert!(inputs.insert(id.to_string(), bytes.to_vec()).is_none(), "duplicate oracle input");
    }
    let cases: Vec<Case> = serde_json::from_slice(&read_record(&mut reader)).unwrap();
    assert_eq!(reader.read(&mut [0]).unwrap(), 0, "trailing oracle records");
    assert!(!cases.is_empty(), "figure oracle corpus must not be empty");
    let corpus = Corpus { inputs, cases };
    validate_corpus(&corpus);
    corpus
}
fn case_id(case: &Case) -> String {
    digest("tax-oracle-v1/input", &serde_json::to_vec(&case.input).unwrap())
}
fn validate_corpus(corpus: &Corpus) {
    let mut seen = BTreeSet::new();
    for case in &corpus.cases {
        assert_eq!(case.version, 1, "unsupported oracle version");
        assert!(seen.insert(case_id(case)), "duplicate oracle case");
        match &case.input {
            Input::Estimate { draft } => {
                load_blob(corpus, draft);
            }
            Input::Send { session, request } => {
                load_blob(corpus, &session.draft);
                load_blob(corpus, request);
            }
        }
        assert!(case.tests.windows(2).all(|w| w[0] < w[1]), "duplicate or unsorted oracle origins");
        assert!(case.tests.iter().all(|name| name.contains("::")), "invalid oracle origin");
    }
}
fn paths(directory: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(directory).unwrap().map(|p| p.unwrap().path()).collect();
    paths.sort();
    paths
}
fn read_capture(source: &Path) -> Corpus {
    let mut inputs = BTreeMap::new();
    for path in paths(&source.join("inputs")) {
        let id = path.file_stem().unwrap().to_str().unwrap().to_string();
        let bytes = fs::read(&path).unwrap();
        assert_eq!(digest("tax-oracle-v1/avro", &bytes), id);
        assert!(inputs.insert(id, bytes).is_none(), "duplicate capture input");
    }
    let mut cases = BTreeMap::new();
    for path in paths(&source.join("cases")) {
        let case: Case = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        let id = case_id(&case);
        assert_eq!(path.file_stem().unwrap().to_str().unwrap(), id);
        assert!(cases.insert(id, case).is_none(), "duplicate capture case");
    }
    let mut origins: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for path in paths(&source.join("origins")) {
        let bytes = fs::read(&path).unwrap();
        assert_eq!(
            path.file_stem().unwrap().to_str().unwrap(),
            digest("tax-oracle-v1/origin", &bytes)
        );
        let (id, test): (String, String) = serde_json::from_slice(&bytes).unwrap();
        assert!(cases.contains_key(&id), "origin refers to missing capture case");
        assert!(origins.entry(id).or_default().insert(test), "duplicate capture origin");
    }
    for (id, case) in &mut cases {
        case.tests =
            origins.remove(id).expect("capture case has no test origin").into_iter().collect();
    }
    assert!(!cases.is_empty(), "empty oracle capture");
    let corpus = Corpus { inputs, cases: cases.into_values().collect() };
    validate_corpus(&corpus);
    corpus
}
fn check_retained(previous: &Corpus, packed: &Corpus) {
    let by_input: BTreeMap<_, _> = packed.cases.iter().map(|case| (case_id(case), case)).collect();
    for case in &previous.cases {
        let id = case_id(case);
        let current = by_input.get(&id).expect("pack dropped an oracle case");
        assert_eq!(current.version, case.version);
        assert_eq!(current.input, case.input);
        assert_eq!(current.expected, case.expected, "pack changed oracle digest {id}");
        assert!(
            case.tests.iter().all(|test| current.tests.contains(test)),
            "pack dropped test provenance"
        );
    }
    assert!(
        previous.inputs.keys().all(|id| packed.inputs.contains_key(id)),
        "pack dropped oracle input"
    );
}
#[inline(never)]
fn decode_status(corpus: &Corpus, input: &Input) -> String {
    let draft = match input {
        Input::Estimate { draft } => draft,
        Input::Send { session, .. } => &session.draft,
    };
    if let Err(error) = FederalDraft::from_avro(load_blob(corpus, draft)) {
        return format!("unavailable in current schema: {error:?}");
    }
    if let Input::Send { request, .. } = input
        && let Err(error) = P::from_avro(load_blob(corpus, request))
    {
        return format!("unavailable in current schema: {error:?}");
    }
    "decodable by current schema (historical output values unavailable)".to_string()
}
fn diff_corpora(previous: &Corpus, packed: &Corpus) -> String {
    assert!(
        previous.cases.iter().all(|case| !case.tests.is_empty()),
        "seed test provenance on the unchanged baseline with strict pack before refreshing"
    );
    let old: BTreeMap<_, _> = previous.cases.iter().map(|case| (case_id(case), case)).collect();
    let new: BTreeMap<_, _> = packed.cases.iter().map(|case| (case_id(case), case)).collect();
    let mut counts: BTreeMap<&str, [usize; 2]> = BTreeMap::new();
    for (side, corpus) in [previous, packed].into_iter().enumerate() {
        for case in &corpus.cases {
            for test in &case.tests {
                counts.entry(test).or_default()[side] += 1;
            }
        }
    }
    let mut groups: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for id in old.keys().chain(new.keys()).collect::<BTreeSet<_>>() {
        let before = old.get(id);
        let after = new.get(id);
        if let (Some(a), Some(b)) = (before, after)
            && a.expected == b.expected
            && a.tests == b.tests
        {
            continue;
        }
        let tests: BTreeSet<_> = before
            .into_iter()
            .flat_map(|case| &case.tests)
            .chain(after.into_iter().flat_map(|case| &case.tests))
            .collect();
        let line = format!(
            "  {id} old={} new={}\n    old-input={} new-input={}\n",
            before.map_or_else(
                || "absent".to_string(),
                |case| serde_json::to_string(&case.expected).unwrap()
            ),
            after.map_or_else(
                || "absent".to_string(),
                |case| serde_json::to_string(&case.expected).unwrap()
            ),
            before
                .map_or_else(|| "absent".to_string(), |case| decode_status(previous, &case.input)),
            after.map_or_else(|| "absent".to_string(), |case| decode_status(packed, &case.input))
        );
        for test in tests {
            groups.entry(test).or_default().push(line.clone());
        }
    }
    let added = new.keys().filter(|id| !old.contains_key(*id)).count();
    let removed = old.keys().filter(|id| !new.contains_key(*id)).count();
    let changed = old
        .iter()
        .filter(|(id, case)| new.get(*id).is_some_and(|current| current.expected != case.expected))
        .count();
    let mut report = format!(
        "Cases: old={} new={} added={added} removed={removed} changed_same_input={changed}\n",
        old.len(),
        new.len()
    );
    report.push_str("Output digests only; historical output values unavailable. Different input IDs are added/removed cases, never paired.\n");
    for (test, lines) in groups {
        let [old_count, new_count] = counts[test];
        writeln!(report, "{test} old_cases={old_count} new_cases={new_count}").unwrap();
        for line in lines {
            report.push_str(&line);
        }
    }
    report
}
// Strict packing is also the provenance-seeding path: all old bytes and digests survive.
// Declared refresh replaces them only after the complete candidate passes replay.
fn pack_corpus(source: &Path, destination: &Path) {
    let packed = read_capture(source);
    let previous = destination.exists().then(|| load_corpus(destination));
    let refresh = std::env::var("TAX_ORACLE_REFRESH").ok();
    if let Some(reason) = &refresh {
        assert!(!reason.trim().is_empty(), "refresh needs a declared reason");
        let report = diff_corpora(previous.as_ref().expect("refresh requires a baseline"), &packed);
        let report_path =
            std::env::var_os("TAX_ORACLE_DIFF").expect("refresh requires TAX_ORACLE_DIFF");
        let report = format!("Declared refresh: {reason}\n{report}");
        fs::write(report_path, &report).unwrap();
        std::print!("{report}");
    } else if let Some(previous) = &previous {
        check_retained(previous, &packed);
    }
    let temporary = destination.with_extension(format!(
        "{}-{}.tmp",
        std::process::id(),
        TEMP_ID.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    let file = OpenOptions::new().write(true).create_new(true).open(&temporary).unwrap();
    let temporary = TemporaryFile(temporary);
    let mut writer = zstd::stream::write::Encoder::new(file, 9).unwrap();
    writer.include_checksum(true).unwrap();
    writer.write_all(HEADER).unwrap();
    writer.write_all(&u64::try_from(packed.inputs.len()).unwrap().to_le_bytes()).unwrap();
    for (id, bytes) in &packed.inputs {
        let mut record = id.as_bytes().to_vec();
        record.extend_from_slice(bytes);
        write_record(&mut writer, &record);
    }
    write_record(&mut writer, &serde_json::to_vec(&packed.cases).unwrap());
    drop(writer.finish().unwrap());
    promote_candidate(&temporary, destination, &packed);
}
struct TemporaryFile(PathBuf);
impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn promote_candidate(temporary: &TemporaryFile, destination: &Path, packed: &Corpus) {
    let candidate = load_corpus(&temporary.0);
    assert_eq!(candidate.cases, packed.cases);
    assert_eq!(candidate.inputs, packed.inputs);
    replay_corpus(&candidate);
    // The candidate is a destination sibling, so promotion is one atomic rename.
    fs::rename(&temporary.0, destination).unwrap();
}
fn load_blob<'a>(corpus: &'a Corpus, id: &str) -> &'a [u8] {
    corpus.inputs.get(id).expect("missing oracle input")
}
// Separate execution frames keep large draft/request values out of the corpus loop.
#[inline(never)]
fn replay(corpus: &Corpus, input: &Input) -> Output {
    match input {
        Input::Estimate { draft } => {
            let draft = FederalDraft::from_avro(load_blob(corpus, draft)).unwrap();
            Output::Estimate { result: result_digest(&crate::federal_rules::estimate(&draft)) }
        }
        Input::Send { session, request } => {
            let draft = FederalDraft::from_avro(load_blob(corpus, &session.draft)).unwrap();
            let mut interview = Interview::oracle_restore(session.user, draft, session.flags);
            let request = P::from_avro(load_blob(corpus, request)).unwrap();
            let reply = interview.handle(request).expect("captured request returned a reply");
            monetary_capture::send(&interview, &reply);
            send_output(&interview, &reply)
        }
    }
}
const SHARDS: usize = 8;
// Whole-corpus work that cannot be split stays in the single `figure_equivalence_oracle` test.
fn single_test_mode() -> bool {
    ["TAX_ORACLE_PACK", "TAX_ORACLE_FIGURES", "TAX_ORACLE_REFRESH"]
        .iter()
        .any(|name| std::env::var_os(name).is_some())
}
fn corpus_path() -> PathBuf {
    std::env::var_os("TAX_ORACLE_REPLAY").map(PathBuf::from).unwrap_or_else(|| {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/oracle/corpus.zst")
    })
}
// Replay and list modes: the corpus is decoded once and replayed by SHARDS independent tests.
fn replay_shard(shard: usize) {
    if single_test_mode() {
        return;
    }
    static CORPUS: OnceLock<Corpus> = OnceLock::new();
    let corpus = CORPUS.get_or_init(|| load_corpus(&corpus_path()));
    match std::env::var_os("TAX_ORACLE_LIST") {
        Some(directory) => list_changes(corpus, Path::new(&directory), (shard, SHARDS)),
        None => assert!(replay_cases(corpus, false, (shard, SHARDS)).is_empty()),
    }
}
macro_rules! oracle_shards {
    ($($name:ident = $shard:expr),*) => {$(
        #[test]
        fn $name() {
            replay_shard($shard);
        }
    )*};
}
oracle_shards!(
    figure_equivalence_oracle_shard_0 = 0,
    figure_equivalence_oracle_shard_1 = 1,
    figure_equivalence_oracle_shard_2 = 2,
    figure_equivalence_oracle_shard_3 = 3,
    figure_equivalence_oracle_shard_4 = 4,
    figure_equivalence_oracle_shard_5 = 5,
    figure_equivalence_oracle_shard_6 = 6,
    figure_equivalence_oracle_shard_7 = 7
);
// Pack, figures and refresh run here, whole.
#[test]
fn figure_equivalence_oracle() {
    if !single_test_mode() {
        return;
    }
    let path = corpus_path();
    if let Some(source) = std::env::var_os("TAX_ORACLE_PACK") {
        pack_corpus(&PathBuf::from(source), &path);
    } else if let Some(ids) = std::env::var_os("TAX_ORACLE_FIGURES") {
        replay_figures(&load_corpus(&path), Path::new(&ids));
    } else {
        replay_corpus(&load_corpus(&path));
    }
}
fn replay_corpus(corpus: &Corpus) {
    assert!(replay_cases(corpus, false, (0, 1)).is_empty());
}
type Change = (usize, Result<Output, String>);
// Independent cases use ordinary platform parallelism and default thread stacks.
// Suppression is thread-local, so each worker owns its replay guard. The first
// mismatch panics its worker unless `list` collects every changed case instead.
// Only the cases with `index % of == shard` run, on `parallelism / of` workers.
fn replay_cases(corpus: &Corpus, list: bool, (shard, of): (usize, usize)) -> Vec<Change> {
    let _guard = ReplayGuard::new();
    let next = AtomicUsize::new(0);
    let changes = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..(std::thread::available_parallelism().unwrap().get() / of).max(1) {
            scope.spawn(|| {
                let _guard = ReplayGuard::new();
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed) * of + shard;
                    let Some(case) = corpus.cases.get(index) else {
                        break;
                    };
                    assert_eq!(case.version, 1, "unsupported oracle version");
                    if !list {
                        assert_eq!(
                            case.expected,
                            replay(corpus, &case.input),
                            "figure oracle changed: {}",
                            case_id(case)
                        );
                        continue;
                    }
                    let actual = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        replay(corpus, &case.input)
                    }))
                    .map_err(|panic| {
                        panic
                            .downcast_ref::<String>()
                            .cloned()
                            .or_else(|| panic.downcast_ref::<&str>().map(|text| text.to_string()))
                            .unwrap_or_else(|| "replay panicked".to_string())
                    });
                    if actual.as_ref() != Ok(&case.expected) {
                        changes.lock().unwrap().push((index, actual));
                    }
                }
            });
        }
    });
    let mut changes = changes.into_inner().unwrap();
    changes.sort_by_key(|change| change.0);
    changes
}
// List mode: replay the shard's cases, write each changed one, then fail once with the count.
// A single shard (0, 1) writes cases.jsonl and changed.jsonl; shard k of n writes cases-k.jsonl
// and changed-k.jsonl for its `index % n == k` cases.
fn list_changes(corpus: &Corpus, directory: &Path, (shard, of): (usize, usize)) {
    let changes = replay_cases(corpus, true, (shard, of));
    fs::create_dir_all(directory).unwrap();
    let suffix = if of == 1 { String::new() } else { format!("-{shard}") };
    let mut cases = Vec::new();
    for case in corpus.cases.iter().skip(shard).step_by(of) {
        let line = serde_json::json!({"input": case_id(case), "tests": case.tests});
        serde_json::to_writer(&mut cases, &line).unwrap();
        cases.push(b'\n');
    }
    fs::write(directory.join(format!("cases{suffix}.jsonl")), cases).unwrap();
    let mut lines = Vec::new();
    for (index, actual) in &changes {
        let case = &corpus.cases[*index];
        let change = serde_json::json!({
            "input": case_id(case), "tests": case.tests, "expected": case.expected,
            "actual": actual,
        });
        serde_json::to_writer(&mut lines, &change).unwrap();
        lines.push(b'\n');
    }
    let path = directory.join(format!("changed{suffix}.jsonl"));
    fs::write(&path, lines).unwrap();
    assert!(
        changes.is_empty(),
        "figure oracle listed {} changed case(s) in {}",
        changes.len(),
        path.display()
    );
}
// Figure mode: replay the listed input digests with the figure capture observing,
// one capture origin per digest (TAX_FIGURE_CAPTURE names the output directory).
fn replay_figures(corpus: &Corpus, ids: &Path) {
    assert!(std::env::var_os("TAX_FIGURE_CAPTURE").is_some(), "figure replay needs a capture");
    let cases: BTreeMap<_, _> = corpus.cases.iter().map(|case| (case_id(case), case)).collect();
    let ids = fs::read_to_string(ids).unwrap();
    let ids: Vec<_> = ids.lines().filter(|id| !id.is_empty()).collect();
    let next = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..std::thread::available_parallelism().unwrap().get() {
            scope.spawn(|| {
                let _guard = ReplayGuard::new();
                while let Some(id) = ids.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let case = cases.get(*id).unwrap_or_else(|| panic!("no oracle case {id}"));
                    monetary_capture::replay_as(id, || replay(corpus, &case.input));
                }
            });
        }
    });
}
fn control_root() -> PathBuf {
    let executable = std::env::current_exe().unwrap();
    let stem = format!("{}-{}", std::process::id(), TEMP_ID.fetch_add(1, Ordering::Relaxed));
    executable.parent().unwrap().join("oracle-controls").join(stem)
}
#[test]
fn list_mode_writes_every_changed_case_and_fails_once() {
    // A one-case slice of the real corpus: its own digest passes, a changed one is listed.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/oracle/corpus.zst");
    let mut corpus = load_corpus(&path);
    let index =
        corpus.cases.iter().position(|case| matches!(case.input, Input::Estimate { .. })).unwrap();
    corpus.cases = std::vec![corpus.cases.swap_remove(index)];
    let root = control_root();
    list_changes(&corpus, &root, (0, 1));
    assert_eq!(fs::read_to_string(root.join("changed.jsonl")).unwrap(), "");
    let id = case_id(&corpus.cases[0]);
    let listed: serde_json::Value =
        serde_json::from_str(fs::read_to_string(root.join("cases.jsonl")).unwrap().trim()).unwrap();
    assert_eq!(listed["input"], id.as_str());
    let actual = std::mem::replace(
        &mut corpus.cases[0].expected,
        Output::Estimate { result: "changed".to_string() },
    );
    let failure = std::panic::catch_unwind(|| list_changes(&corpus, &root, (0, 1))).unwrap_err();
    assert!(failure.downcast_ref::<String>().unwrap().contains("listed 1 changed case(s)"));
    let line: serde_json::Value =
        serde_json::from_str(fs::read_to_string(root.join("changed.jsonl")).unwrap().trim())
            .unwrap();
    assert_eq!(line["input"], id.as_str());
    assert_eq!(line["expected"], serde_json::json!({"Estimate": {"result": "changed"}}));
    assert_eq!(line["actual"], serde_json::json!({"Ok": actual}));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn strict_pack_seeds_only_provenance() {
    fn corpus(tests: Vec<String>, result: &str) -> Corpus {
        Corpus {
            inputs: BTreeMap::new(),
            cases: std::vec![Case {
                version: 1,
                input: Input::Estimate { draft: "same-input".to_string() },
                expected: Output::Estimate { result: result.to_string() },
                tests,
            }],
        }
    }
    let previous = corpus(Vec::new(), "original");
    let seeded = corpus(std::vec!["tests::builder".to_string()], "original");
    check_retained(&previous, &seeded);
    assert!(
        std::panic::catch_unwind(|| check_retained(&seeded, &corpus(Vec::new(), "original")))
            .is_err()
    );
    assert!(
        std::panic::catch_unwind(|| check_retained(&previous, &corpus(Vec::new(), "changed")))
            .is_err()
    );
}

#[test]
fn refresh_diff_groups_actual_origins_without_pairing_inputs() {
    fn corpus(draft: &str, result: &str, tests: &[&str]) -> Corpus {
        Corpus {
            inputs: BTreeMap::from([(draft.to_string(), Vec::new())]),
            cases: std::vec![Case {
                version: 1,
                input: Input::Estimate { draft: draft.to_string() },
                expected: Output::Estimate { result: result.to_string() },
                tests: tests.iter().map(|test| test.to_string()).collect(),
            }],
        }
    }
    let old = corpus("old-input", "old-digest", &["tests::one", "tests::two"]);
    let changed = corpus("old-input", "new-digest", &["tests::one", "tests::two"]);
    let report = diff_corpora(&old, &changed);
    assert!(report.contains("changed_same_input=1"));
    assert_eq!(report.matches("old-digest").count(), 2);
    assert_eq!(report.matches("new-digest").count(), 2);
    let new_shape = corpus("new-input", "new-digest", &["tests::one"]);
    let report = diff_corpora(&old, &new_shape);
    assert!(report.contains("added=1 removed=1 changed_same_input=0"));
    assert!(report.contains("old=absent"));
    assert!(report.contains("new=absent"));
    assert!(report.contains("tests::two"));
    assert!(report.contains("unavailable in current schema"));
    let unknown = corpus("old-input", "old-digest", &[]);
    assert!(std::panic::catch_unwind(|| diff_corpora(&unknown, &changed)).is_err());
}

#[test]
fn failed_candidate_preserves_baseline_and_removes_temporary() {
    let executable = std::env::current_exe().unwrap();
    let root = executable.parent().unwrap().join("oracle-controls");
    fs::create_dir_all(&root).unwrap();
    let stem = format!("{}-{}", std::process::id(), TEMP_ID.fetch_add(1, Ordering::Relaxed));
    let destination = root.join(format!("{stem}.baseline"));
    let candidate = root.join(format!("{stem}.candidate"));
    let mut baseline = OpenOptions::new().write(true).create_new(true).open(&destination).unwrap();
    baseline.write_all(b"original baseline").unwrap();
    drop(baseline);
    let mut file = OpenOptions::new().write(true).create_new(true).open(&candidate).unwrap();
    let temporary = TemporaryFile(candidate.clone());
    file.write_all(b"invalid compressed candidate").unwrap();
    drop(file);
    assert!(
        std::panic::catch_unwind(move || promote_candidate(
            &temporary,
            &destination,
            &Corpus { inputs: BTreeMap::new(), cases: Vec::new() }
        ))
        .is_err()
    );
    assert!(!candidate.exists(), "failed owned candidate must be removed");
    let destination = root.join(format!("{stem}.baseline"));
    assert_eq!(fs::read(&destination).unwrap(), b"original baseline");
    fs::remove_file(destination).unwrap();
}
