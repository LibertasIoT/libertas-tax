// Opt-in, ordered diagnostic capture. Independent of oracle blobs/digests.
// Capture entry points suspend heap accounting (tax_test_heap::untracked), so
// a captured run reports the same per-test peak as an uncaptured one.
use super::*;
use serde_json::json;
use tax_paired_figures::capture_budget::MAX_TEST_CAPTURE_BYTES;

std::thread_local! {
    static TEST_STATS: std::cell::RefCell<TestCapture> = const { std::cell::RefCell::new(TestCapture(TestStats::new())) };
    static SEQUENCE: Cell<u64> = const { Cell::new(0) };
    // Oracle figure replay names each observation by its corpus input digest.
    static ORIGIN: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}
struct CaptureStream {
    events: Option<std::sync::Mutex<std::fs::File>>,
    stats: std::sync::Mutex<std::fs::File>,
}
static STREAM: OnceLock<Option<CaptureStream>> = OnceLock::new();

struct TestStats {
    test: Option<String>,
    bytes: u64,
    observations: u64,
    largest_record: u64,
}
impl TestStats {
    const fn new() -> Self {
        Self { test: None, bytes: 0, observations: 0, largest_record: 0 }
    }
    fn observe(&mut self, test: &str, json_bytes: usize) {
        // The event serializer excludes the LF that capture_frame appends.
        let bytes = u64::try_from(json_bytes).unwrap().checked_add(1).unwrap();
        if let Some(previous) = &self.test {
            assert_eq!(previous, test, "capture thread changed test identity");
        } else {
            self.test = Some(String::from(test));
        }
        self.bytes = self.bytes.checked_add(bytes).unwrap();
        self.observations = self.observations.checked_add(1).unwrap();
        self.largest_record = self.largest_record.max(bytes);
    }
    fn report(&self, writer: &mut impl std::io::Write) {
        let Some(test) = &self.test else { return };
        serde_json::to_writer(
            &mut *writer,
            &json!({
                "test": test, "bytes": self.bytes, "observations": self.observations,
                "largest_record": self.largest_record,
                "over_budget": self.bytes > MAX_TEST_CAPTURE_BYTES,
            }),
        )
        .unwrap();
        writer.write_all(b"\n").unwrap();
    }
}
struct TestCapture(TestStats);
impl Drop for TestCapture {
    fn drop(&mut self) {
        let _untracked = tax_test_heap::untracked();
        if self.0.test.is_some() {
            // Test-thread teardown runs for passing and unwinding tests alike.
            let stream = STREAM.get().unwrap().as_ref().unwrap();
            self.0.report(&mut *stream.stats.lock().unwrap());
        }
    }
}
fn stream_enabled() -> bool {
    std::env::var_os("TAX_FIGURE_CAPTURE").is_some()
        || std::env::var_os("TAX_FIGURE_CAPTURE_STATS").is_some()
}

// Observations made while replaying one oracle input, in the same capture format as a
// test, with that input's digest in place of the test name.
pub(super) fn replay_as<T>(origin: &str, replay: impl FnOnce() -> T) -> T {
    ORIGIN.set(Some(origin.to_string()));
    SEQUENCE.set(0);
    let value = replay();
    ORIGIN.set(None);
    value
}

fn stream() -> Option<&'static CaptureStream> {
    STREAM
        .get_or_init(|| {
            if !stream_enabled() {
                return None;
            }
            let root = std::env::var_os("TAX_FIGURE_CAPTURE").map(PathBuf::from);
            let events = root.as_ref().map(|root| {
                assert!(root.is_absolute(), "TAX_FIGURE_CAPTURE must be absolute");
                fs::create_dir_all(root).unwrap();
                // Never mix a new run with an older raw or gzip capture.
                assert!(!root.join("events.jsonl").exists(), "capture already exists");
                assert!(!root.join("events.jsonl.gz").exists(), "capture already exists");
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(root.join("events.jsonl.zst"))
                    .unwrap()
            });
            let stats = std::env::var_os("TAX_FIGURE_CAPTURE_STATS")
                .map(PathBuf::from)
                .unwrap_or_else(|| root.as_ref().unwrap().join("events.stats.jsonl"));
            assert!(stats.is_absolute(), "TAX_FIGURE_CAPTURE_STATS must be absolute");
            let stats = OpenOptions::new().write(true).create_new(true).open(stats).unwrap();
            Some(CaptureStream {
                events: events.map(std::sync::Mutex::new),
                stats: std::sync::Mutex::new(stats),
            })
        })
        .as_ref()
}

pub(in crate::tests) fn report_heap(test: &str, peak_heap_bytes: usize) {
    let Some(stream) = stream() else { return };
    let mut writer = stream.stats.lock().unwrap();
    serde_json::to_writer(
        &mut *writer,
        &json!({
            "kind": "heap", "test": test, "peak_heap_bytes": peak_heap_bytes,
        }),
    )
    .unwrap();
    writer.write_all(b"\n").unwrap();
}

fn record(kind: &str, money: serde_json::Value, debug: String) {
    let origin = ORIGIN.with_borrow(Clone::clone);
    if origin.is_none() && REPLAYING.get() {
        return;
    }
    let Some(stream) = stream() else { return };
    let thread = std::thread::current();
    let test = match &origin {
        Some(origin) => origin.as_str(),
        None => thread.name().expect("figure capture requires a named test thread"),
    };
    assert!(origin.is_some() || test.contains("::"), "figure capture requires a test origin");
    let test_thread = thread.name().expect("figure statistics require a named test thread");
    let index = SEQUENCE.get();
    SEQUENCE.set(index.checked_add(1).unwrap());
    // Compact captures preserve every observation and its complete comparison
    // fingerprints while bounding the optional diagnostic text.
    let event = json!({
        "version": 1, "test": test, "index": index, "kind": kind,
        "money": money, "debug": debug,
    });
    let event = if std::env::var_os("TAX_FIGURE_CAPTURE_LEAN").is_some() {
        tax_paired_figures::compact::compact_event(&event).unwrap()
    } else {
        event
    };
    // Serialize before acquiring the shared file to keep JSON escaping outside
    // the shared write lock.
    let bytes = serde_json::to_vec(&event).unwrap();
    // One complete frame per event permits indexed seeks without an expanded
    // capture or a process-exit flush. Compression stays outside the file lock.
    let json_bytes = bytes.len();
    // Capture byte statistics describe the actual diagnostic payload.
    if let Some(events) = &stream.events {
        let frame = capture_frame(bytes).unwrap();
        events.lock().unwrap().write_all(&frame).unwrap();
    }
    TEST_STATS.with_borrow_mut(|stats| stats.0.observe(test_thread, json_bytes));
}

fn capture_frame(mut json: Vec<u8>) -> std::io::Result<Vec<u8>> {
    json.push(b'\n');
    zstd::stream::encode_all(json.as_slice(), 1)
}

#[test]
fn compressed_events_are_complete_seekable_frames_and_preserve_the_jsonl_stream() {
    use std::io::{BufReader, Cursor, Read, Seek};
    let events = [
        br#"{"debug":"Reply { value: 100 }","index":0,"kind":"Send","money":null,"test":"capture::case","version":1}"#.as_slice(),
        br#"{"debug":"Reply { value: 101 }","index":1,"kind":"Send","money":null,"test":"capture::case","version":1}"#.as_slice(),
    ];
    let frames: Vec<_> =
        events.iter().map(|event| capture_frame(event.to_vec()).unwrap()).collect();
    let stream = frames.concat();
    let expected: Vec<_> =
        events.iter().flat_map(|event| event.iter().copied().chain([b'\n'])).collect();
    assert_eq!(zstd::stream::decode_all(stream.as_slice()).unwrap(), expected);
    let mut reader = BufReader::new(Cursor::new(stream));
    for (event, frame) in events.iter().zip(&frames) {
        let offset = reader.stream_position().unwrap();
        let mut decoder = zstd::stream::read::Decoder::with_buffer(reader).unwrap().single_frame();
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, [*event, b"\n"].concat());
        reader = decoder.finish();
        assert_eq!(reader.stream_position().unwrap(), offset + frame.len() as u64);
    }
}

#[test]
fn byte_statistics_count_utf8_and_lf_across_sequence_resets() {
    let mut stats = TestStats::new();
    let event = serde_json::to_vec(&json!({ "debug": "é\n" })).unwrap();
    for origin in ["first", "second"] {
        replay_as(origin, || stats.observe("capture::one_test", event.len()));
    }
    assert_eq!(stats.bytes, 2 * (event.len() as u64 + 1));
    assert_eq!(stats.observations, 2);
    assert_eq!(stats.largest_record, event.len() as u64 + 1);
}

#[test]
fn byte_budget_is_strictly_greater_than_the_limit() {
    let mut stats = TestStats::new();
    // Count synthetic lengths without allocating a budget-sized fixture.
    stats.observe("capture::boundary", usize::try_from(MAX_TEST_CAPTURE_BYTES - 1).unwrap());
    let mut report = Vec::new();
    stats.report(&mut report);
    let exact: serde_json::Value = serde_json::from_slice(&report).unwrap();
    assert_eq!(exact["over_budget"], false);
    stats.observe("capture::boundary", 0);
    report.clear();
    stats.report(&mut report);
    let above: serde_json::Value = serde_json::from_slice(&report).unwrap();
    assert_eq!(above["bytes"], MAX_TEST_CAPTURE_BYTES + 1);
    assert_eq!(above["over_budget"], true);
}

pub(super) fn estimate(result: &FederalResult) {
    let _untracked = tax_test_heap::untracked();
    if std::env::var_os("TAX_FIGURE_CAPTURE").is_none() {
        return;
    }
    let lines: Vec<_> = result
        .lines
        .iter()
        .map(|line| {
            json!({
                "id": line.id, "kind": format!("{:?}", line.kind), "amount": line.amount,
            })
        })
        .collect();
    let return_rows: Vec<_> = result
        .return_lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            json!({
                "index": index, "amount": line.amount, "money": line.money,
                "form_line": format!("{:?}", line.form_line),
            })
        })
        .collect();
    record(
        "Estimate",
        json!({
            "top": {
                "tax": result.tax, "refundable_credits": result.refundable_credits,
                "payments": result.payments, "refund": result.refund, "balance": result.balance,
                "farmland_installments.applicable": result.farmland_installments.as_ref().map(|v| v.applicable),
                "farmland_installments.first": result.farmland_installments.as_ref().map(|v| v.first),
                "farmland_installments.remaining": result.farmland_installments.as_ref().map(|v| v.remaining),
                "farmland_installments.current": result.farmland_installments.as_ref().map(|v| v.current),
            },
            "lines": lines,
            "return_rows": return_rows,
        }),
        format!("{result:?}"),
    );
}

pub(super) fn send(session: &Interview, reply: &Reply) {
    let _untracked = tax_test_heap::untracked();
    if std::env::var_os("TAX_FIGURE_CAPTURE").is_none() {
        return;
    }
    // Match every Send output owner: reply plus the saved session values
    // whose Avro digest the oracle otherwise records without retaining them.
    record(
        "Send",
        serde_json::Value::Null,
        format!(
            "SendOutput {{ reply: {reply:?}, draft: {:?}, user: {:?}, flags: {:?} }}",
            session.draft,
            session.user,
            session.oracle_flags(),
        ),
    );
}

// One save with its estimate; the parity test below runs it as a child process.
#[test]
fn heap_parity_fixture() {
    drop(crate::tests::support::client::care_w2_sources(1));
}

#[test]
fn capture_does_not_change_the_measured_heap_peak() {
    let fixture = format!("{}::heap_parity_fixture", module_path!().split_once("::").unwrap().1);
    let root = control_root();
    fs::create_dir_all(&root).unwrap();
    let run = |name: &str, capture: bool| {
        let stats = root.join(format!("{name}.stats.jsonl"));
        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        child.args(["--exact", &fixture, "--test-threads=1", "--quiet"]);
        for (key, _) in
            std::env::vars_os().filter(|(key, _)| key.to_string_lossy().starts_with("TAX_"))
        {
            child.env_remove(key);
        }
        child.env("TAX_FIGURE_CAPTURE_STATS", &stats);
        if capture {
            child.env("TAX_FIGURE_CAPTURE", root.join(name)).env("TAX_FIGURE_CAPTURE_LEAN", "1");
            child.env("TAX_ORACLE_CAPTURE", root.join(format!("{name}-oracle")));
        }
        let output = child.output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let lines: Vec<serde_json::Value> = fs::read_to_string(stats)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .filter(|line: &serde_json::Value| line["test"] == fixture.as_str())
            .collect();
        let field = |key: &str| lines.iter().find_map(|line| line[key].as_u64());
        (field("peak_heap_bytes").unwrap(), field("observations"))
    };
    let (uncaptured, none) = run("off", false);
    let (captured, observations) = run("on", true);
    assert_eq!(none, None);
    assert!(observations.unwrap() > 0, "the captured run recorded no figure events");
    assert!(root.join("on-oracle").join("cases").read_dir().unwrap().next().is_some());
    assert_eq!(captured, uncaptured, "capture changed the fixture's measured heap peak");
    fs::remove_dir_all(root).unwrap();
}
