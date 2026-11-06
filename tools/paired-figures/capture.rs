//! Bounded-memory capture indexing. Independent zstd frames retain physical offsets.
use crate::{Result, fail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::value::RawValue;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

/// Streaming input shared by capture reduction and artifact verification.
/// A failed consumer must stop and reap the gzip producer before returning.
pub(crate) struct Input {
    reader: BufReader<Box<dyn Read + Send>>,
    child: Option<Child>,
}
impl Input {
    pub(crate) fn open(path: &Path) -> Result<Self> {
        let mut child = None;
        let source: Box<dyn Read + Send> = match path.extension().and_then(|s| s.to_str()) {
            Some("gz") => {
                let mut gzip = Command::new("gzip")
                    .args(["-dc", "--"])
                    .arg(path)
                    .stdout(Stdio::piped())
                    .spawn()?;
                let source = gzip.stdout.take().ok_or("missing gzip stdout")?;
                child = Some(gzip);
                Box::new(source)
            }
            Some("zst") => Box::new(zstd::stream::read::Decoder::new(File::open(path)?)?),
            _ => Box::new(File::open(path)?),
        };
        Ok(Self { reader: BufReader::with_capacity(1024 * 1024, source), child })
    }
    pub(crate) fn finish(&mut self) -> Result<()> {
        if let Some(child) = &mut self.child {
            let status = child.wait()?;
            self.child = None;
            if !status.success() {
                return fail("gzip input decoding failed");
            }
        }
        Ok(())
    }
}
impl Read for Input {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(bytes)
    }
}
impl BufRead for Input {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        self.reader.fill_buf()
    }
    fn consume(&mut self, len: usize) {
        self.reader.consume(len);
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct TestStats {
    pub bytes: u64,
    pub observations: u64,
    pub estimates: u64,
    pub largest_record: u64,
}

#[derive(Deserialize)]
struct Header {
    version: u64,
    test: String,
    index: usize,
    #[serde(rename = "kind")]
    _kind: String,
}
fn final_marker(bytes: &[u8]) -> Result<bool> {
    Ok(std::str::from_utf8(bytes)?.contains("\"kind\":\"Estimate\""))
}
#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub struct RawSpan {
    pub start: usize,
    pub len: usize,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Entry {
    pub offset: u64,
    pub len: u64,
    pub kind: String,
    pub index: i128,
    pub version: Option<u64>,
    pub debug_hash: Option<[u8; 32]>,
    pub money_hash: Option<[u8; 32]>,
    pub debug_span: Option<RawSpan>,
    pub money_span: Option<RawSpan>,
    pub final_eligible: bool,
}
pub struct Capture {
    pub path: PathBuf,
    pub compressed: bool,
    pub tests: BTreeMap<String, Vec<Entry>>,
    pub stats: BTreeMap<String, TestStats>,
    temporary: bool,
}
impl Drop for Capture {
    fn drop(&mut self) {
        if self.temporary {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
pub(crate) struct RawFields<'a> {
    pub(crate) fields: BTreeMap<String, &'a RawValue>,
    pub(crate) duplicates: std::collections::BTreeSet<String>,
}
impl<'de> Deserialize<'de> for RawFields<'de> {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = RawFields<'de>;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("capture event object")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> std::result::Result<Self::Value, M::Error> {
                let mut result =
                    RawFields { fields: BTreeMap::new(), duplicates: Default::default() };
                while let Some((key, value)) = map.next_entry::<String, &'de RawValue>()? {
                    if result.fields.insert(key.clone(), value).is_some() {
                        result.duplicates.insert(key);
                    }
                }
                Ok(result)
            }
        }
        deserializer.deserialize_map(Visitor)
    }
}
enum Prepared {
    Ignore,
    Count { test: String, bytes: u64, estimate: bool },
    Entry { test: String, bytes: u64, entry: Box<Entry>, validation: Result<()> },
}
impl Prepared {
    fn apply(self, capture: &mut Capture, validate: bool) -> Result<()> {
        match self {
            Self::Ignore => {}
            Self::Count { test, bytes, estimate } => capture.count(test, bytes, estimate),
            Self::Entry { test, bytes, entry, validation } => {
                capture.count(test.clone(), bytes, entry.kind == "Estimate");
                let entries = capture.tests.entry(test.clone()).or_default();
                if validate
                    && (!matches!(entry.version, Some(1 | 2))
                        || entry.index != entries.len() as i128)
                {
                    return fail(format!(
                        "nonsequential/unsupported capture: {test} #{}",
                        entry.index
                    ));
                }
                // Source ordering and version checks precede compact semantic errors.
                validation?;
                entries.push(*entry);
            }
        }
        Ok(())
    }
}
fn prepare(bytes: &[u8], offset: u64, len: u64, validate: bool) -> Result<Prepared> {
    // RawValue validates all JSON syntax without allocating payload trees.
    // Final-only scans historically ignore malformed rows without the exact marker.
    let marker = final_marker(bytes)?;
    let fields = serde_json::from_slice::<RawFields<'_>>(bytes);
    let fields = match fields {
        Ok(fields) => fields,
        Err(_) if !validate && !marker => return Ok(Prepared::Ignore),
        Err(error) => return Err(error.into()),
    };
    let duplicate = |keys: &[&str]| {
        keys.iter()
            .find(|key| fields.duplicates.contains(**key))
            .map(|key| format!("duplicate field `{key}`"))
    };
    if let Some(error) = duplicate(&["test", "kind"]) {
        if !validate && !marker {
            return Ok(Prepared::Ignore);
        }
        return fail(error);
    }
    let duplicates = fields.duplicates;
    let fields = fields.fields;
    let test = fields.get("test").and_then(|raw| serde_json::from_str::<String>(raw.get()).ok());
    let kind = fields.get("kind").and_then(|raw| serde_json::from_str::<String>(raw.get()).ok());
    let logical_bytes = bytes.len() as u64;
    let estimate = kind.as_deref() == Some("Estimate");
    if test.as_deref().is_some_and(excluded_capacity_test) {
        return Ok(Prepared::Count { test: test.unwrap(), bytes: logical_bytes, estimate });
    }
    if !validate && !marker {
        return Ok(match test {
            Some(test) => Prepared::Count { test, bytes: logical_bytes, estimate },
            None => Prepared::Ignore,
        });
    }
    if let Some(key) = ["index", "version"]
        .into_iter()
        .find(|key| (*key != "version" || validate) && duplicates.contains(*key))
    {
        return fail(format!("duplicate field `{key}`"));
    }
    let test = test.ok_or("capture test must be a string")?;
    let kind = kind.ok_or("capture kind must be a string")?;
    let index: i128 =
        serde_json::from_str(fields.get("index").ok_or("missing field `index`")?.get())?;
    let version = fields.get("version").and_then(|raw| serde_json::from_str::<u64>(raw.get()).ok());
    if !validate && kind != "Estimate" {
        return Ok(Prepared::Count { test, bytes: logical_bytes, estimate });
    }
    let span = |key: &str| {
        fields.get(key).map(|raw| RawSpan {
            start: raw.get().as_ptr() as usize - bytes.as_ptr() as usize,
            len: raw.get().len(),
        })
    };
    let debug_span = span("debug");
    let money_span = span("money");
    let hash = |span: RawSpan| Sha256::digest(&bytes[span.start..span.start + span.len]).into();
    let entry = Entry {
        offset,
        len,
        final_eligible: marker && kind == "Estimate",
        kind,
        index,
        version,
        debug_hash: debug_span.filter(|span| bytes[span.start] == b'"').map(hash),
        money_hash: money_span.map(hash),
        debug_span,
        money_span,
    };
    let validation = if validate && version == Some(2) {
        serde_json::from_slice(bytes)
            .map_err(crate::Error::from)
            .and_then(|event| crate::compact::validate_compact_event(&event))
    } else {
        Ok(())
    };
    Ok(Prepared::Entry { test, bytes: logical_bytes, entry: Box::new(entry), validation })
}

/// Only the explicit stress-test module is outside the figure comparison.
fn excluded_capacity_test(name: &str) -> bool {
    name.starts_with("tests::core_tests::unit53_capacity::")
}
pub fn exclude_capacity(old: &mut Capture, new: &mut Capture) {
    old.tests.retain(|name, _| !excluded_capacity_test(name));
    new.tests.retain(|name, _| !excluded_capacity_test(name));
}
pub fn add_exclusions(report: &mut crate::json::Json, old: &Capture, new: &Capture) {
    let missing = TestStats::default();
    let entries: BTreeMap<_, _> = old
        .stats
        .keys()
        .chain(new.stats.keys())
        .filter(|name| excluded_capacity_test(name))
        .map(|name| {
            let before = old.stats.get(name).unwrap_or(&missing);
            let after = new.stats.get(name).unwrap_or(&missing);
            (
                name.clone(),
                crate::json::object([
                    ("old_observations", crate::json::val(before.observations)),
                    ("new_observations", crate::json::val(after.observations)),
                    ("old_estimates", crate::json::val(before.estimates)),
                    ("new_estimates", crate::json::val(after.estimates)),
                ]),
            )
        })
        .collect();
    if !entries.is_empty()
        && let crate::json::Json::Object(fields) = report
    {
        fields.push(("excluded_capacity_tests".into(), crate::json::object(entries)));
    }
}

const CACHE_VERSION: u64 = 5;
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
struct SourceIdentity {
    dev: u64,
    ino: u64,
    size: u64,
    mtime: i64,
    mtime_nsec: i64,
    ctime: i64,
    ctime_nsec: i64,
}
impl SourceIdentity {
    fn read(path: &Path) -> Result<Self> {
        use std::os::unix::fs::MetadataExt;
        let m = std::fs::metadata(path)?;
        Ok(Self {
            dev: m.dev(),
            ino: m.ino(),
            size: m.len(),
            mtime: m.mtime(),
            mtime_nsec: m.mtime_nsec(),
            ctime: m.ctime(),
            ctime_nsec: m.ctime_nsec(),
        })
    }
    fn key(&self, strict: bool) -> String {
        format!(
            "capture-v{CACHE_VERSION}-{:x}-{}.json",
            Sha256::digest(serde_json::to_vec(self).expect("identity JSON")),
            u8::from(strict)
        )
    }
}
#[derive(Serialize, Deserialize)]
struct CachedIndex {
    format: u64,
    identity: SourceIdentity,
    strict: bool,
    compressed: bool,
    backing: Option<PathBuf>,
    tests: BTreeMap<String, Vec<Entry>>,
    stats: BTreeMap<String, TestStats>,
}

// This bounds admitted source bytes, including partially read rows and ordered
// results. A row larger than the budget is admitted alone, never truncated.
const LINE_WINDOW_BYTES: usize = 64 * 1024 * 1024;
#[derive(Default)]
struct AdmissionState {
    records: usize,
    bytes: usize,
    cancelled: bool,
}
#[derive(Default)]
struct Admission {
    state: std::sync::Mutex<AdmissionState>,
    changed: std::sync::Condvar,
}
impl Admission {
    fn reserve(&self, window: usize, limit: usize, current: Option<usize>, bytes: usize) -> bool {
        let mut state = self.state.lock().expect("line admission");
        while !state.cancelled
            && match current {
                None => state.records >= window,
                Some(held) => state.bytes.saturating_add(bytes) > limit && state.bytes != held,
            }
        {
            state = self.changed.wait(state).expect("line admission");
        }
        if state.cancelled {
            return false;
        }
        if current.is_none() {
            state.records += 1;
        }
        state.bytes += bytes;
        true
    }
    fn release(&self, bytes: usize) {
        let mut state = self.state.lock().expect("line admission");
        state.records -= 1;
        state.bytes -= bytes;
        self.changed.notify_all();
    }
    fn cancel(&self) {
        self.state.lock().expect("line admission").cancelled = true;
        self.changed.notify_all();
    }
}

fn admitted_line(
    reader: &mut impl BufRead,
    admission: &Admission,
    limit: usize,
) -> Result<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok(Some(bytes));
        }
        let end = available.iter().position(|byte| *byte == b'\n').map(|index| index + 1);
        let len = end.unwrap_or(available.len());
        if !admission.reserve(0, limit, Some(bytes.len()), len) {
            return Ok(None);
        }
        bytes.extend_from_slice(&available[..len]);
        reader.consume(len);
        if end.is_some() {
            return Ok(Some(bytes));
        }
    }
}

fn ordered_lines<T: Send>(
    mut reader: impl BufRead + Send,
    jobs: usize,
    byte_limit: usize,
    map: impl Fn(&[u8], u64) -> Result<T> + Sync,
    apply: impl FnMut(T) -> Result<()>,
) -> Result<()> {
    let mut offset = 0;
    ordered_records(
        move |admission, limit| {
            Ok(admitted_line(&mut reader, admission, limit)?.and_then(|bytes| {
                if bytes.is_empty() {
                    return None;
                }
                let start = offset;
                offset += bytes.len() as u64;
                Some((bytes, start))
            }))
        },
        jobs,
        byte_limit,
        map,
        apply,
    )
}

fn ordered_records<M: Send, T: Send>(
    mut read: impl FnMut(&Admission, usize) -> Result<Option<(Vec<u8>, M)>> + Send,
    jobs: usize,
    byte_limit: usize,
    map: impl Fn(&[u8], M) -> Result<T> + Sync,
    mut apply: impl FnMut(T) -> Result<()>,
) -> Result<()> {
    use std::sync::{Mutex, mpsc};
    let jobs = jobs.max(1);
    let window = jobs.saturating_mul(2);
    let admission = Admission::default();
    let (work, queue) = mpsc::sync_channel::<(usize, M, Vec<u8>)>(window);
    let queue = Mutex::new(queue);
    let (completed, results) = mpsc::channel();
    std::thread::scope(|scope| {
        let finished = completed.clone();
        let admission = &admission;
        scope.spawn(move || {
            let mut ordinal = 0;
            while admission.reserve(window, byte_limit, None, 0) {
                match read(admission, byte_limit) {
                    Ok(None) => {
                        let _ = finished.send((ordinal, 0, None));
                        break;
                    }
                    Ok(Some((bytes, metadata))) => {
                        if work.send((ordinal, metadata, bytes)).is_err() {
                            break;
                        }
                        ordinal += 1;
                    }
                    Err(error) => {
                        let _ = finished.send((ordinal, 0, Some(Err(error))));
                        break;
                    }
                }
            }
        });
        for _ in 0..jobs {
            let queue = &queue;
            let map = &map;
            let completed = completed.clone();
            scope.spawn(move || {
                loop {
                    let record = { queue.lock().expect("record queue").recv() };
                    let Ok((ordinal, metadata, bytes)) = record else {
                        break;
                    };
                    let len = bytes.len();
                    let result = map(&bytes, metadata);
                    drop(bytes);
                    if completed.send((ordinal, len, Some(result))).is_err() {
                        break;
                    }
                }
            });
        }
        drop(completed);
        let result = (|| {
            let mut pending = BTreeMap::new();
            let mut next = 0;
            loop {
                let (ordinal, len, result) = results.recv()?;
                pending.insert(ordinal, (len, result));
                while let Some((len, result)) = pending.remove(&next) {
                    let Some(result) = result else {
                        return Ok(());
                    };
                    apply(result?)?;
                    next += 1;
                    admission.release(len);
                }
            }
        })();
        // Wake byte/record admission and disconnect every channel on every exit,
        // including an ordered map or apply error.
        admission.cancel();
        drop(results);
        result
    })
}

impl Capture {
    fn count(&mut self, test: String, bytes: u64, estimate: bool) {
        let stats = self.stats.entry(test).or_default();
        stats.bytes += bytes;
        stats.observations += 1;
        stats.estimates += u64::from(estimate);
        stats.largest_record = stats.largest_record.max(bytes);
    }
    #[cfg(test)]
    pub fn index(path: &str, scratch: &Path, validate: bool) -> Result<Self> {
        Self::index_cached(path, scratch, validate, None)
    }
    pub fn index_cached(
        path: &str,
        scratch: &Path,
        validate: bool,
        cache: Option<&Path>,
    ) -> Result<Self> {
        Self::index_cached_parallel(path, scratch, validate, cache, 1)
    }
    pub fn index_cached_parallel(
        path: &str,
        scratch: &Path,
        validate: bool,
        cache: Option<&Path>,
        jobs: usize,
    ) -> Result<Self> {
        let identity = SourceIdentity::read(Path::new(path))?;
        if let Some(cache) = cache {
            std::fs::create_dir_all(cache)?;
            for strict in if validate { vec![true] } else { vec![true, false] } {
                let sidecar = cache.join(identity.key(strict));
                if let Ok(file) = File::open(sidecar)
                    && let Ok(saved) =
                        serde_json::from_reader::<_, CachedIndex>(BufReader::new(file))
                    && saved.identity == identity
                    && saved.strict == strict
                    && saved.format == CACHE_VERSION
                {
                    let backing = saved.backing.unwrap_or_else(|| PathBuf::from(path));
                    if backing.is_file() {
                        let mut result = Self {
                            path: backing,
                            compressed: saved.compressed,
                            tests: saved.tests,
                            stats: saved.stats,
                            temporary: false,
                        };
                        if !validate {
                            result.final_only();
                        }
                        if SourceIdentity::read(Path::new(path))? != identity {
                            return fail("capture changed during index loading");
                        }
                        return Ok(result);
                    }
                }
            }
        }
        let result = Self::scan(path, scratch, validate, jobs)?;
        if SourceIdentity::read(Path::new(path))? != identity {
            return fail("capture changed during indexing");
        }
        if let Some(cache) = cache {
            // Transcoded offsets must never outlive their scratch backing file.
            // Keep a durable independent-frame stream beside its cached metadata.
            let backing = if result.temporary {
                let destination =
                    cache.join(format!("{}.{}.zst", identity.key(validate), crate::unique()));
                std::fs::copy(&result.path, &destination)?;
                Some(destination)
            } else {
                None
            };
            let saved = CachedIndex {
                format: CACHE_VERSION,
                identity: identity.clone(),
                strict: validate,
                compressed: result.compressed,
                backing,
                tests: result.tests.clone(),
                stats: result.stats.clone(),
            };
            let temporary =
                cache.join(format!("index-{}-{}.tmp", std::process::id(), crate::unique()));
            let write = (|| -> Result<()> {
                let mut file = BufWriter::with_capacity(
                    1024 * 1024,
                    OpenOptions::new().write(true).create_new(true).open(&temporary)?,
                );
                serde_json::to_writer(&mut file, &saved)?;
                file.flush()?;
                std::fs::rename(&temporary, cache.join(identity.key(validate)))?;
                Ok(())
            })();
            if write.is_err() {
                let _ = std::fs::remove_file(temporary);
            }
            write?;
        }
        Ok(result)
    }
    fn final_only(&mut self) {
        self.tests.retain(|_, entries| {
            entries.retain(|entry| entry.final_eligible);
            !entries.is_empty()
        });
    }
    fn scan(path: &str, scratch: &Path, validate: bool, jobs: usize) -> Result<Self> {
        let path = PathBuf::from(path);
        let mut result = Self {
            compressed: path.extension().is_some_and(|s| s == "zst"),
            path,
            tests: BTreeMap::new(),
            stats: BTreeMap::new(),
            temporary: false,
        };
        if result.path.extension().is_some_and(|s| s == "gz") {
            // gzip has no portable random-access offsets. Transcode only the temporary
            // working stream to independent frames; the source is always read-only.
            let mut source = Input::open(&result.path)?;
            result.transcode(&mut source, scratch, validate, jobs)?;
            source.finish()?;
        } else if result.compressed {
            if result.scan_frames(validate, jobs, LINE_WINDOW_BYTES)? {
                // General frames may span several rows or split one row. Decode
                // the logical JSONL stream before building event seek points.
                let source = zstd::stream::read::Decoder::new(File::open(&result.path)?)?;
                result.transcode(BufReader::new(source), scratch, validate, jobs)?;
            }
        } else if jobs > 1 {
            result.scan_parallel(validate, jobs)?;
        } else {
            let mut reader = BufReader::with_capacity(1024 * 1024, File::open(&result.path)?);
            let mut line = Vec::new();
            let mut offset = 0;
            loop {
                line.clear();
                let len = reader.read_until(b'\n', &mut line)?;
                if len == 0 {
                    break;
                }
                prepare(&line, offset, len as u64, validate)?.apply(&mut result, validate)?;
                offset += len as u64;
            }
        }
        if validate && result.tests.is_empty() && result.stats.is_empty() {
            return fail("empty capture");
        }
        Ok(result)
    }
    // Returns true when a general frame requires the logical-stream fallback.
    fn scan_frames(&mut self, validate: bool, jobs: usize, byte_limit: usize) -> Result<bool> {
        let mut reader = Some(BufReader::with_capacity(1024 * 1024, File::open(&self.path)?));
        let mut fallback = false;
        ordered_records(
            move |admission, limit| {
                while let Some(mut source) = reader.take() {
                    if source.fill_buf()?.is_empty() {
                        return Ok(None);
                    }
                    let offset = source.stream_position()?;
                    let decoder = zstd::stream::read::Decoder::with_buffer(source)?.single_frame();
                    let mut decoded = BufReader::new(decoder);
                    let Some(line) = admitted_line(&mut decoded, admission, limit)? else {
                        return Ok(None);
                    };
                    if line.is_empty() {
                        reader = Some(decoded.into_inner().finish());
                        continue;
                    }
                    if !decoded.fill_buf()?.is_empty() || !line.ends_with(b"\n") {
                        // Stop producing at the fallback; earlier rows still own errors.
                        return Ok(Some((line, None)));
                    }
                    let mut source = decoded.into_inner().finish();
                    let len = source.stream_position()? - offset;
                    reader = Some(source);
                    return Ok(Some((line, Some((offset, len)))));
                }
                Ok(None)
            },
            jobs,
            byte_limit,
            |bytes, span| {
                span.map(|(offset, len)| prepare(bytes, offset, len, validate)).transpose()
            },
            |prepared| {
                if let Some(prepared) = prepared {
                    prepared.apply(self, validate)?;
                } else {
                    fallback = true;
                }
                Ok(())
            },
        )?;
        Ok(fallback)
    }
    fn scan_parallel(&mut self, validate: bool, jobs: usize) -> Result<()> {
        let reader = BufReader::with_capacity(1024 * 1024, File::open(&self.path)?);
        self.scan_parallel_reader(reader, validate, jobs)
    }
    fn scan_parallel_reader(
        &mut self,
        mut reader: impl BufRead + Send,
        validate: bool,
        jobs: usize,
    ) -> Result<()> {
        ordered_lines(
            &mut reader,
            jobs,
            LINE_WINDOW_BYTES,
            |bytes, offset| prepare(bytes, offset, bytes.len() as u64, validate),
            |prepared| prepared.apply(self, validate),
        )
    }
    fn transcode(
        &mut self,
        input: impl BufRead + Send,
        scratch: &std::path::Path,
        validate: bool,
        jobs: usize,
    ) -> Result<()> {
        let temporary =
            scratch.join(format!("capture-{}-{}.zst", std::process::id(), crate::unique()));
        let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
        self.path = temporary;
        self.compressed = true;
        self.temporary = true;
        self.tests.clear();
        self.stats.clear();
        let mut offset = 0;
        ordered_lines(
            input,
            jobs,
            LINE_WINDOW_BYTES,
            |line, _| {
                let frame = zstd::stream::encode_all(line, 1)?;
                let prepared = prepare(line, 0, frame.len() as u64, validate)?;
                Ok((frame, prepared))
            },
            |(frame, mut prepared)| {
                if let Prepared::Entry { entry, .. } = &mut prepared {
                    entry.offset = offset;
                }
                prepared.apply(self, validate)?;
                file.write_all(&frame)?;
                offset += frame.len() as u64;
                Ok(())
            },
        )?;
        Ok(())
    }
    pub fn reader(&self) -> Result<Reader> {
        Ok(Reader {
            file: BufReader::new(File::open(&self.path)?),
            compressed: self.compressed,
            bytes: Vec::new(),
        })
    }
}
pub struct Reader {
    file: BufReader<File>,
    compressed: bool,
    bytes: Vec<u8>,
}
impl Reader {
    fn load(&mut self, entry: &Entry) -> Result<()> {
        self.file.seek(SeekFrom::Start(entry.offset))?;
        self.bytes.clear();
        if self.compressed {
            let mut decoder =
                zstd::stream::read::Decoder::with_buffer(&mut self.file)?.single_frame();
            decoder.read_to_end(&mut self.bytes)?;
        } else {
            self.file.by_ref().take(entry.len).read_to_end(&mut self.bytes)?;
        }
        Ok(())
    }
    pub fn raw(&mut self, entry: &Entry) -> Result<&[u8]> {
        self.load(entry)?;
        Ok(&self.bytes)
    }
    pub fn event(&mut self, entry: &Entry) -> Result<Value> {
        Ok(serde_json::from_slice(self.raw(entry)?)?)
    }
    pub fn money(&mut self, entry: &Entry) -> Result<Value> {
        #[derive(Deserialize)]
        struct Money {
            money: Value,
        }
        if let Some(span) = entry.money_span {
            if self.compressed {
                self.load(entry)?;
                return Ok(serde_json::from_slice(&self.bytes[span.start..span.start + span.len])?);
            }
            self.file.seek(SeekFrom::Start(entry.offset + span.start as u64))?;
            self.bytes.clear();
            self.file.by_ref().take(span.len as u64).read_to_end(&mut self.bytes)?;
            return Ok(serde_json::from_slice(&self.bytes)?);
        }
        self.load(entry)?;
        let row: Money = serde_json::from_slice(&self.bytes)?;
        Ok(row.money)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn row(test: &str, index: usize, debug: &str) -> Vec<u8> {
        let mut bytes=serde_json::to_vec(&serde_json::json!({"version":1,"test":test,"index":index,"kind":"Send","debug":debug,"money":null})).unwrap();
        bytes.push(b'\n');
        bytes
    }
    #[test]
    fn independent_frames_seek_to_interleaved_test_events() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("events.zst");
        let mut file = File::create(&path).unwrap();
        for bytes in [
            row("alpha", 0, "Reply { x: 1 }"),
            row("beta", 0, "None"),
            row("alpha", 1, "Reply { x: 2 }"),
        ] {
            file.write_all(&zstd::stream::encode_all(bytes.as_slice(), 1).unwrap()).unwrap();
        }
        drop(file);
        let capture = Capture::index(path.to_str().unwrap(), &temp.0, true).unwrap();
        let mut reader = capture.reader().unwrap();
        assert_eq!(reader.event(&capture.tests["alpha"][1]).unwrap()["debug"], "Reply { x: 2 }");
        assert_eq!(reader.event(&capture.tests["beta"][0]).unwrap()["debug"], "None");
        assert_eq!(reader.event(&capture.tests["alpha"][0]).unwrap()["index"], 0);
    }
    #[test]
    fn raw_index_rejects_nonsequential_events_and_empty_capture() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("events.jsonl");
        std::fs::write(&path, row("alpha", 1, "None")).unwrap();
        assert!(Capture::index(path.to_str().unwrap(), &temp.0, true).is_err());
        std::fs::write(&path, []).unwrap();
        assert!(Capture::index(path.to_str().unwrap(), &temp.0, true).is_err());
    }
}

pub fn compress(args: &crate::Args) -> Result<i32> {
    use sha2::{Digest, Sha256};
    if args.positional.len() != 2 {
        return fail("usage: tax-paired-figures compress OLD_EVENTS OUTPUT.zst");
    }
    let input = &args.positional[0];
    let output = &args.positional[1];
    let mut destination = OpenOptions::new().write(true).create_new(true).open(output)?;
    let result = (|| -> Result<(u64, String)> {
        let mut reader = BufReader::new(File::open(input)?);
        let mut line = Vec::new();
        let mut hash = Sha256::new();
        let mut count = 0u64;
        loop {
            line.clear();
            if reader.read_until(b'\n', &mut line)? == 0 {
                break;
            }
            // Preserve every source byte, including the original event ordering.
            let _: Header = serde_json::from_slice(&line)?;
            hash.update(&line);
            destination.write_all(&zstd::stream::encode_all(line.as_slice(), 1)?)?;
            count += 1;
        }
        destination.flush()?;
        Ok((count, format!("{:x}", hash.finalize())))
    })();
    match result {
        Ok((events, sha256)) => {
            println!(
                "{}",
                crate::spaced_json(&serde_json::json!({"events":events,"sha256":sha256}))
            );
            Ok(0)
        }
        Err(error) => {
            drop(destination);
            let _ = std::fs::remove_file(output);
            Err(error)
        }
    }
}

/// Reduce a legacy capture without changing test/completion ordering or typed figures.
fn reduce_lines(
    reader: impl BufRead + Send,
    destination: &mut impl Write,
    jobs: usize,
    byte_limit: usize,
) -> Result<(u64, u64, u64, String)> {
    let mut encoder = zstd::stream::write::Encoder::new(destination, 1)?;
    encoder.include_checksum(true)?;
    let mut sequence = BTreeMap::<String, usize>::new();
    let mut hash = Sha256::new();
    let mut events = 0u64;
    let mut source_bytes = 0u64;
    let mut compact_bytes = 0u64;
    ordered_lines(
        reader,
        jobs,
        byte_limit,
        |line, _| {
            let header: Header = serde_json::from_slice(line)?;
            // Sequence/version errors precede compact errors for the same row.
            let compact = (|| -> Result<_> {
                let compact = crate::compact::compact_event(&serde_json::from_slice(line)?)?;
                let mut bytes = serde_json::to_vec(&compact)?;
                bytes.push(b'\n');
                Ok(bytes)
            })();
            Ok((header, line.len(), compact))
        },
        |(header, len, compact)| {
            let expected = sequence.entry(header.test.clone()).or_default();
            if header.version != 1 || header.index != *expected {
                return fail(format!(
                    "nonsequential/unsupported reduction source: {} #{}",
                    header.test, header.index
                ));
            }
            *expected += 1;
            let bytes = compact?;
            hash.update(&bytes);
            compact_bytes += bytes.len() as u64;
            source_bytes += len as u64;
            encoder.write_all(&bytes)?;
            events += 1;
            Ok(())
        },
    )?;
    encoder.finish()?;
    Ok((events, source_bytes, compact_bytes, format!("{:x}", hash.finalize())))
}

pub fn reduce(args: &crate::Args) -> Result<i32> {
    if args.positional.len() != 2 {
        return fail("usage: tax-paired-figures reduce OLD_EVENTS OUTPUT.zst");
    }
    let input = &args.positional[0];
    let output = &args.positional[1];
    let mut destination = OpenOptions::new().write(true).create_new(true).open(output)?;
    let result = (|| -> Result<(u64, u64, u64, String)> {
        let mut reader = Input::open(Path::new(input))?;
        let (events, source_bytes, compact_bytes, sha256) =
            reduce_lines(&mut reader, &mut destination, args.jobs, LINE_WINDOW_BYTES)?;
        if events == 0 {
            return fail("empty capture");
        }
        reader.finish()?;
        destination.flush()?;
        Ok((events, source_bytes, compact_bytes, sha256))
    })();
    match result {
        Ok((events, source_bytes, compact_bytes, sha256)) => {
            println!(
                "{}",
                crate::spaced_json(&serde_json::json!({
                    "events":events, "source_bytes":source_bytes, "compact_bytes":compact_bytes,
                    "compressed_bytes":destination.metadata()?.len(), "sha256":sha256,
                }))
            );
            Ok(0)
        }
        Err(error) => {
            drop(destination);
            let _ = std::fs::remove_file(output);
            Err(error)
        }
    }
}

#[cfg(test)]
mod transcode_tests {
    use super::*;
    #[test]
    fn general_streams_preserve_offsets_reader_cache_and_first_error() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let raw = temp.0.join("events.jsonl");
        let rows = [tests::row("alpha", 0, "None"), tests::row("alpha", 1, "Some(2)")];
        for (source, expected_error) in [
            (rows.concat(), None),
            (
                [tests::row("alpha", 7, "None"), b"broken\n".to_vec()].concat(),
                Some("nonsequential/unsupported capture: alpha #7"),
            ),
            (
                [tests::row("alpha", 0, "None"), b"broken\n".to_vec()].concat(),
                Some("expected value"),
            ),
        ] {
            std::fs::write(&raw, &source).unwrap();
            for (extension, bytes) in [
                ("zst", zstd::stream::encode_all(source.as_slice(), 1).unwrap()),
                ("gz", Command::new("gzip").arg("-c").arg(&raw).output().unwrap().stdout),
            ] {
                let path = temp.0.join(format!("events.{extension}"));
                std::fs::write(&path, &bytes).unwrap();
                let mut expected = None;
                for jobs in [1, 3] {
                    let cache = temp.0.join(format!("cache-{extension}-{jobs}"));
                    let result = Capture::index_cached_parallel(
                        path.to_str().unwrap(),
                        &temp.0,
                        true,
                        Some(&cache),
                        jobs,
                    );
                    assert_eq!(result.is_err(), expected_error.is_some());
                    let snapshot = match result {
                        Ok(capture) => {
                            assert!(capture.temporary);
                            let mut offset = 0;
                            let mut reader = capture.reader().unwrap();
                            for (row, entry) in rows.iter().zip(&capture.tests["alpha"]) {
                                assert_eq!(entry.offset, offset);
                                assert_eq!(reader.raw(entry).unwrap(), row);
                                offset += entry.len;
                            }
                            assert_eq!(std::fs::metadata(&capture.path).unwrap().len(), offset);
                            let cached = Capture::index_cached_parallel(
                                path.to_str().unwrap(),
                                &temp.0,
                                true,
                                Some(&cache),
                                jobs,
                            )
                            .unwrap();
                            assert_eq!(
                                cached.reader().unwrap().raw(&cached.tests["alpha"][1]).unwrap(),
                                rows[1]
                            );
                            let snapshot =
                                serde_json::to_value((&capture.tests, &capture.stats)).unwrap();
                            assert_eq!(
                                serde_json::to_value((&cached.tests, &cached.stats)).unwrap(),
                                snapshot
                            );
                            Ok(snapshot)
                        }
                        Err(error) => {
                            assert!(error.to_string().contains(expected_error.unwrap()));
                            assert!(!std::fs::read_dir(&temp.0).unwrap().any(|entry| {
                                entry.unwrap().file_name().to_string_lossy().starts_with("capture-")
                            }));
                            Err(error.to_string())
                        }
                    };
                    if let Some(expected) = &expected {
                        assert_eq!(&snapshot, expected);
                    }
                    expected = Some(snapshot);
                }
                assert_eq!(std::fs::read(&path).unwrap(), bytes);
            }
        }
    }
    #[test]
    fn compression_preserves_original_jsonl_bytes_and_refuses_existing_destination() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let source = temp.0.join("raw.jsonl");
        let output = temp.0.join("events.zst");
        let bytes=b"{\"version\":1,\"test\":\"case\",\"index\":0,\"kind\":\"Send\",\"debug\":\"None\",\"money\":null}\n";
        std::fs::write(&source, bytes).unwrap();
        let args = crate::Args {
            command: "compress".into(),
            options: BTreeMap::new(),
            positional: vec![source.to_str().unwrap().into(), output.to_str().unwrap().into()],
            shard: (0, 1),
            jobs: 1,
            differences_only: false,
        };
        assert_eq!(compress(&args).unwrap(), 0);
        assert_eq!(zstd::stream::decode_all(File::open(&output).unwrap()).unwrap(), bytes);
        assert!(compress(&args).is_err());
        assert_eq!(std::fs::read(source).unwrap(), bytes);
    }
}

#[cfg(test)]
mod final_filter_tests {
    use super::*;
    #[test]
    fn final_filter_skips_noncompact_estimates_and_malformed_send_without_version() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("final.jsonl");
        let bytes=b"malformed Send text\n{\"kind\": \"Estimate\", \"test\": \"ignored\", \"index\":0, \"money\":null}\n{\"kind\":\"Estimate\",\"test\":\"case\",\"index\":-7,\"money\":{}}\n";
        std::fs::write(&path, bytes).unwrap();
        let capture = Capture::index(path.to_str().unwrap(), &temp.0, false).unwrap();
        assert_eq!(capture.tests.len(), 1);
        assert_eq!(capture.tests["case"][0].index, -7);
        assert!(Capture::index(path.to_str().unwrap(), &temp.0, true).is_err());
        std::fs::write(&path, b"invalid utf8 \xff\n").unwrap();
        assert!(Capture::index(path.to_str().unwrap(), &temp.0, false).is_err());
    }
}

#[cfg(test)]
mod raw_index_tests {
    use super::tests::row;
    use super::*;

    fn parallel_control(bytes: &[u8], validate: bool) -> Option<String> {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("parallel.jsonl");
        std::fs::write(&path, bytes).unwrap();
        let sequential = Capture::index(path.to_str().unwrap(), &temp.0, validate);
        let parallel =
            Capture::index_cached_parallel(path.to_str().unwrap(), &temp.0, validate, None, 3);
        match (sequential, parallel) {
            (Ok(sequential), Ok(parallel)) => {
                assert_eq!(
                    serde_json::to_value(&sequential.tests).unwrap(),
                    serde_json::to_value(&parallel.tests).unwrap()
                );
                assert_eq!(
                    serde_json::to_value(&sequential.stats).unwrap(),
                    serde_json::to_value(&parallel.stats).unwrap()
                );
                let mut reader = parallel.reader().unwrap();
                for entries in parallel.tests.values() {
                    for entry in entries {
                        assert_eq!(
                            reader.raw(entry).unwrap(),
                            &bytes[entry.offset as usize..(entry.offset + entry.len) as usize]
                        );
                    }
                }
                None
            }
            (Err(sequential), Err(parallel)) => {
                assert_eq!(sequential.to_string(), parallel.to_string());
                Some(sequential.to_string())
            }
            _ => panic!("sequential and parallel indexing disagree"),
        }
    }

    #[test]
    fn parallel_raw_index_preserves_interleaving_exclusions_and_final_filter() {
        let rows = concat!(
            "{\"version\":1,\"test\":\"alpha\",\"kind\":\"Send\",\"index\":0,\"debug\":\"quote: \\u0031 ☃\",\"unknown\":[{}]}\n",
            "{\"version\":1,\"test\":\"tests::core_tests::unit53_capacity::skip\",\"kind\":\"Estimate\",\"index\":0}\n",
            "{\"version\":1,\"test\":\"beta\",\"kind\": \"Estimate\",\"index\":0,\"money\":{}}\n",
            "{\"version\":1,\"test\":\"alpha\",\"kind\":\"Estimate\",\"index\":1,\"money\":{\"x\":1.00}}"
        );
        for validate in [true, false] {
            assert!(parallel_control(rows.as_bytes(), validate).is_none());
        }
        let compact = crate::compact::compact_event(
            &serde_json::from_slice::<Value>(&row("compact", 0, "None")).unwrap(),
        )
        .unwrap();
        let bytes =
            [rows.as_bytes(), b"\n", serde_json::to_vec(&compact).unwrap().as_slice()].concat();
        assert!(parallel_control(&bytes, true).is_none());
        assert!(
            parallel_control(
                b"malformed Send\n{\"kind\":\"Estimate\",\"test\":\"case\",\"index\":-3}",
                false
            )
            .is_none()
        );
    }

    #[test]
    fn parallel_raw_index_reports_the_first_error_in_source_order() {
        let prefix = row("alpha", 0, "None");
        for (first, second, expected) in [
            (
                r#"{"version":2,"test":"alpha","kind":"Send","index":7}"#,
                "broken",
                "nonsequential/unsupported capture: alpha #7",
            ),
            (
                r#"{"version":2,"test":"alpha","kind":"Send","index":1}"#,
                "broken",
                "compact capture debug_bytes must be an unsigned integer",
            ),
            (
                r#"{"version":3,"test":"alpha","kind":"Send","index":1}"#,
                "broken",
                "nonsequential/unsupported capture: alpha #1",
            ),
            (
                r#"{"version":1,"test":"alpha","kind":"Send","index":1,"test":"alpha"}"#,
                "broken",
                "duplicate field `test`",
            ),
            (
                r#"{"version":1,"test":"alpha","kind":"Send","index":1,"index":1}"#,
                "broken",
                "duplicate field `index`",
            ),
            ("broken", r#"{"version":1,"test":"alpha","kind":"Send","index":9}"#, ""),
        ] {
            let bytes = [prefix.as_slice(), first.as_bytes(), b"\n", second.as_bytes()].concat();
            let error = parallel_control(&bytes, true).unwrap();
            assert!(error.contains(expected), "{error}");
        }
        for bytes in
            [b"invalid utf8 \xff\n".as_slice(), b"{\"kind\":\"Estimate\",\"kind\":\"Send\"}", b""]
        {
            assert!(parallel_control(bytes, true).is_some());
        }
    }

    #[test]
    fn parallel_raw_index_cancels_admission_and_preserves_ordered_io_errors() {
        let mut bytes = b"broken\n".to_vec();
        for index in 0..128 {
            bytes.extend(row("alpha", index, "None"));
        }
        assert!(parallel_control(&bytes, true).is_some());
        struct ReadFailure(std::io::Cursor<Vec<u8>>);
        impl Read for ReadFailure {
            fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
                self.0.read(bytes)
            }
        }
        impl BufRead for ReadFailure {
            fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
                if self.0.position() == self.0.get_ref().len() as u64 {
                    return Err(std::io::Error::other("injected read failure"));
                }
                self.0.fill_buf()
            }
            fn consume(&mut self, len: usize) {
                self.0.consume(len);
            }
        }
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("empty.jsonl");
        std::fs::write(&path, []).unwrap();
        let mut capture = Capture::index(path.to_str().unwrap(), &temp.0, false).unwrap();
        for (bytes, expected) in [
            (row("alpha", 0, "None"), "injected read failure"),
            (row("alpha", 9, "None"), "nonsequential/unsupported capture: alpha #9"),
        ] {
            capture.tests.clear();
            let error = capture
                .scan_parallel_reader(ReadFailure(std::io::Cursor::new(bytes)), true, 3)
                .unwrap_err();
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn spans_hash_raw_json_spelling_and_money_reads_only_its_value() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("raw.jsonl");
        let row = br#"{"version":1,"test":"case","kind":"Estimate","index":0,"debug":"quote: \" and \u0031","money": {"amount":1.00},"ignored":[true,null]}"#;
        std::fs::write(&path, row).unwrap();
        let capture = Capture::index(path.to_str().unwrap(), &temp.0, true).unwrap();
        let entry = &capture.tests["case"][0];
        assert_eq!(entry.version, Some(1));
        let debug = entry.debug_span.unwrap();
        let money = entry.money_span.unwrap();
        assert_eq!(&row[debug.start..debug.start + debug.len], br#""quote: \" and \u0031""#);
        assert_eq!(
            entry.debug_hash,
            Some(Sha256::digest(&row[debug.start..debug.start + debug.len]).into())
        );
        assert_eq!(entry.money_hash, Some(Sha256::digest(br#"{"amount":1.00}"#).into()));
        assert_eq!(&row[money.start..money.start + money.len], br#"{"amount":1.00}"#);
        let mut reader = capture.reader().unwrap();
        assert_eq!(reader.raw(entry).unwrap(), row);
        assert_eq!(reader.money(entry).unwrap()["amount"], 1.0);
        let fallback = Entry { money_span: None, ..entry.clone() };
        assert_eq!(reader.money(&fallback).unwrap()["amount"], 1.0);
        std::fs::write(&path, br#"{"version":1,"test":"case","kind":"Send","index":0}"#).unwrap();
        let capture = Capture::index(path.to_str().unwrap(), &temp.0, true).unwrap();
        let entry = &capture.tests["case"][0];
        assert!(entry.debug_hash.is_none());
        assert!(entry.money_hash.is_none());
        assert!(capture.reader().unwrap().money(entry).is_err());
        for debug in ["null", "7", "{}", "[]", "false"] {
            std::fs::write(&path, format!(r#"{{"version":1,"test":"case","kind":"Send","index":0,"debug":{debug},"money":null}}"#)).unwrap();
            let capture = Capture::index(path.to_str().unwrap(), &temp.0, true).unwrap();
            assert!(capture.tests["case"][0].debug_span.is_some());
            assert!(capture.tests["case"][0].debug_hash.is_none());
        }
    }

    #[test]
    fn strict_sidecar_reuse_preserves_exact_final_marker_and_invalidates_source_changes() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let cache = temp.0.join("cache");
        let path = temp.0.join("raw.jsonl");
        let rows = concat!(
            "{\"version\":1,\"test\":\"case\",\"kind\": \"Estimate\",\"index\":0,\"money\":null}\n",
            "{\"version\":1,\"test\":\"case\",\"kind\":\"Send\",\"index\":1,\"money\":null}\n",
            "{\"version\":1,\"test\":\"case\",\"kind\":\"Estimate\",\"index\":2,\"money\":null}\n"
        );
        std::fs::write(&path, rows).unwrap();
        let source = path.to_str().unwrap();
        let strict =
            Capture::index_cached_parallel(source, &temp.0, true, Some(&cache), 3).unwrap();
        assert_eq!(strict.tests["case"].len(), 3);
        let final_capture = Capture::index_cached(source, &temp.0, false, Some(&cache)).unwrap();
        assert_eq!(final_capture.tests["case"].len(), 1);
        assert_eq!(final_capture.tests["case"][0].index, 2);
        assert_eq!(std::fs::read_dir(&cache).unwrap().count(), 1);
        std::fs::write(&path, b"malformed Send\n{\"test\":\"case\",\"kind\":\"Estimate\",\"index\":-3,\"money\":{}}\n").unwrap();
        let final_capture = Capture::index_cached(source, &temp.0, false, Some(&cache)).unwrap();
        assert_eq!(final_capture.tests["case"][0].index, -3);
        assert!(Capture::index_cached(source, &temp.0, true, Some(&cache)).is_err());
    }

    #[test]
    fn cached_general_frames_keep_durable_backing_after_scratch_cleanup() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let cache = temp.0.join("cache");
        let path = temp.0.join("raw.zst");
        let rows = b"{\"version\":1,\"test\":\"case\",\"kind\":\"Estimate\",\"index\":0,\"money\":{}}\n{\"version\":1,\"test\":\"case\",\"kind\":\"Estimate\",\"index\":1,\"money\":null}\n";
        std::fs::write(&path, zstd::stream::encode_all(rows.as_slice(), 1).unwrap()).unwrap();
        let scratch = crate::TempDir::new(&temp.0).unwrap();
        let first =
            Capture::index_cached(path.to_str().unwrap(), &scratch.0, true, Some(&cache)).unwrap();
        let temporary = first.path.clone();
        drop(first);
        drop(scratch);
        assert!(!temporary.exists());
        let reused =
            Capture::index_cached(path.to_str().unwrap(), &temp.0, false, Some(&cache)).unwrap();
        assert!(reused.path.starts_with(&cache));
        assert_eq!(reused.reader().unwrap().money(&reused.tests["case"][1]).unwrap(), Value::Null);
    }

    #[test]
    fn raw_syntax_and_duplicate_headers_fail_without_weakening_exclusion() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("raw.jsonl");
        for row in [
            r#"{"version":1,"test":"case","kind":"Send","index":0,"debug":"bad\q"}"#,
            r#"{"version":1,"test":"case","kind":"Send","index":0,"index":0}"#,
            r#"{"version":1,"version":1,"test":"case","kind":"Send","index":0}"#,
        ] {
            std::fs::write(&path, row).unwrap();
            assert!(Capture::index(path.to_str().unwrap(), &temp.0, true).is_err());
        }
        std::fs::write(&path, r#"{"version":2,"test":"tax_capacity","kind":"Estimate","index":0,"debug":null,"money":{}}"#).unwrap();
        assert!(Capture::index(path.to_str().unwrap(), &temp.0, true).is_err());
    }
}

#[cfg(test)]
mod frame_index_tests {
    use super::tests::row;
    use super::*;

    fn frames(rows: &[Vec<u8>]) -> Vec<u8> {
        rows.iter().flat_map(|row| zstd::stream::encode_all(row.as_slice(), 1).unwrap()).collect()
    }

    #[test]
    fn parallel_frames_preserve_physical_spans_statistics_and_cached_final_filter() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("events.zst");
        let raw = temp.0.join("events.jsonl");
        let compact = crate::compact::compact_event(
            &serde_json::from_slice::<Value>(&row("compact", 0, "Some(2)")).unwrap(),
        )
        .unwrap();
        let mut compact = serde_json::to_vec(&compact).unwrap();
        compact.push(b'\n');
        let rows = vec![
            row("alpha", 0, "None"),
            row("tests::core_tests::unit53_capacity::skip", 0, "None"),
            b"{\"version\":1,\"test\":\"beta\",\"kind\": \"Estimate\",\"index\":0,\"money\":{}}\n".to_vec(),
            b"{\"version\":1,\"test\":\"alpha\",\"kind\":\"Estimate\",\"index\":1,\"money\":{\"x\":1.00}}\n".to_vec(),
            compact,
        ];
        std::fs::write(&path, frames(&rows)).unwrap();
        std::fs::write(&raw, rows.concat()).unwrap();
        let mut expected = Capture::index(raw.to_str().unwrap(), &temp.0, true).unwrap();
        let mut offset = 0;
        for bytes in &rows {
            let event: Value = serde_json::from_slice(bytes).unwrap();
            let len = zstd::stream::encode_all(bytes.as_slice(), 1).unwrap().len() as u64;
            if let Some(entries) = expected.tests.get_mut(event["test"].as_str().unwrap()) {
                let entry = &mut entries[event["index"].as_u64().unwrap() as usize];
                entry.offset = offset;
                entry.len = len;
            }
            offset += len;
        }
        for jobs in [1, 3] {
            let cache = temp.0.join(format!("cache-{jobs}"));
            let capture = Capture::index_cached_parallel(
                path.to_str().unwrap(),
                &temp.0,
                true,
                Some(&cache),
                jobs,
            )
            .unwrap();
            assert!(!capture.temporary);
            assert_eq!(
                serde_json::to_value(&capture.tests).unwrap(),
                serde_json::to_value(&expected.tests).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&capture.stats).unwrap(),
                serde_json::to_value(&expected.stats).unwrap()
            );
            let mut reader = capture.reader().unwrap();
            for entries in capture.tests.values() {
                for entry in entries {
                    let event = reader.event(entry).unwrap();
                    assert_eq!(event["index"].as_u64().unwrap() as i128, entry.index);
                }
            }
            let cached = Capture::index_cached_parallel(
                path.to_str().unwrap(),
                &temp.0,
                false,
                Some(&cache),
                jobs,
            )
            .unwrap();
            assert_eq!(cached.tests.len(), 1);
            assert_eq!(cached.tests["alpha"][0].index, 1);
            let uncached =
                Capture::index_cached_parallel(path.to_str().unwrap(), &temp.0, false, None, jobs)
                    .unwrap();
            assert_eq!(
                serde_json::to_value(&uncached.tests).unwrap(),
                serde_json::to_value(&cached.tests).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&uncached.stats).unwrap(),
                serde_json::to_value(&cached.stats).unwrap()
            );
            assert_eq!(
                serde_json::to_value(&cached.stats).unwrap(),
                serde_json::to_value(&capture.stats).unwrap()
            );
            // Tiny budgets exercise an oversized row alone and accumulated rows.
            for limit in [17, rows.concat().len()] {
                let mut bounded = Capture {
                    path: path.clone(),
                    compressed: true,
                    tests: BTreeMap::new(),
                    stats: BTreeMap::new(),
                    temporary: false,
                };
                assert!(!bounded.scan_frames(true, jobs, limit).unwrap());
                assert_eq!(
                    serde_json::to_value(&bounded.tests).unwrap(),
                    serde_json::to_value(&capture.tests).unwrap()
                );
            }
        }
    }

    #[test]
    fn parallel_frames_keep_first_error_before_decoder_error_or_general_fallback() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("events.zst");
        for (first, expected) in [
            (
                b"{\"version\":2,\"test\":\"alpha\",\"kind\":\"Send\",\"index\":7}\n".as_slice(),
                "nonsequential/unsupported capture: alpha #7",
            ),
            (
                b"{\"version\":2,\"test\":\"alpha\",\"kind\":\"Send\",\"index\":0}\n",
                "compact capture debug_bytes must be an unsigned integer",
            ),
            (b"broken\n", "expected value"),
            (b"invalid utf8 \xff\n", "invalid utf-8"),
        ] {
            for tail in [
                frames(&[row("beta", 0, "None"), row("beta", 1, "None")]),
                zstd::stream::encode_all(b"general\nframe\n".as_slice(), 1).unwrap(),
                vec![0x28, 0xb5, 0x2f, 0xfd],
            ] {
                let bytes = [frames(&[first.to_vec()]), tail].concat();
                std::fs::write(&path, bytes).unwrap();
                let mut errors = Vec::new();
                for jobs in [1, 3] {
                    errors.push(
                        Capture::index_cached_parallel(
                            path.to_str().unwrap(),
                            &temp.0,
                            true,
                            None,
                            jobs,
                        )
                        .err()
                        .unwrap()
                        .to_string(),
                    );
                }
                assert_eq!(errors[0], errors[1]);
                assert!(errors[0].contains(expected), "{}", errors[0]);
            }
        }
        for bytes in [vec![], frames(&[vec![]]), vec![0x28, 0xb5, 0x2f, 0xfd]] {
            std::fs::write(&path, bytes).unwrap();
            let errors: Vec<_> = [1, 3]
                .into_iter()
                .map(|jobs| {
                    Capture::index_cached_parallel(
                        path.to_str().unwrap(),
                        &temp.0,
                        true,
                        None,
                        jobs,
                    )
                    .err()
                    .unwrap()
                    .to_string()
                })
                .collect();
            assert_eq!(errors[0], errors[1]);
        }
    }

    #[test]
    fn parallel_frames_fall_back_for_multiple_rows_split_rows_and_missing_lf() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("events.zst");
        let rows = [row("alpha", 0, "None"), row("alpha", 1, "Some(2)")];
        let mut no_lf = rows.concat();
        no_lf.pop();
        for parts in [
            vec![rows.concat()],
            vec![rows[0][..9].to_vec(), [rows[0][9..].to_vec(), rows[1].clone()].concat()],
            vec![no_lf],
        ] {
            let source = frames(&parts);
            std::fs::write(&path, &source).unwrap();
            let mut expected = None;
            for jobs in [1, 3] {
                let capture = Capture::index_cached_parallel(
                    path.to_str().unwrap(),
                    &temp.0,
                    true,
                    None,
                    jobs,
                )
                .unwrap();
                assert!(capture.temporary);
                let snapshot = (
                    serde_json::to_value(&capture.tests).unwrap(),
                    serde_json::to_value(&capture.stats).unwrap(),
                );
                if let Some(expected) = &expected {
                    assert_eq!(&snapshot, expected);
                }
                expected = Some(snapshot);
                assert_eq!(
                    capture.reader().unwrap().event(&capture.tests["alpha"][1]).unwrap()["debug"],
                    "Some(2)"
                );
            }
            assert_eq!(std::fs::read(&path).unwrap(), source);
        }
        let source = frames(&[vec![], rows[0].clone(), vec![], rows[1].clone()]);
        std::fs::write(&path, source).unwrap();
        for jobs in [1, 3] {
            let capture =
                Capture::index_cached_parallel(path.to_str().unwrap(), &temp.0, true, None, jobs)
                    .unwrap();
            assert!(!capture.temporary);
            assert_eq!(capture.tests["alpha"].len(), 2);
        }
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;

    fn padded(test: &str, index: usize, kind: &str, size: usize) -> Vec<u8> {
        let mut row = format!(r#"{{"version":1,"test":"{test}","index":{index},"kind":"{kind}","money":{{}},"debug":"opaque @","padding":""#).into_bytes();
        row.resize(size - 3, b'x');
        row.extend_from_slice(b"\"}\n");
        assert_eq!(row.len(), size);
        row
    }

    #[test]
    fn selection_ignores_volume_and_excludes_only_the_exact_capacity_module() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("events.jsonl");
        let names = [
            "ordinary",
            "tests::health::hsa_capacity",
            "tests::core_tests::unit53_capacity_other::ordinary",
        ];
        let mut rows = Vec::new();
        for name in names {
            rows.extend(padded(name, 0, "Estimate", 200));
        }
        let stress = "tests::core_tests::unit53_capacity::stress";
        let old_only = "tests::core_tests::unit53_capacity::old_only";
        let new_only = "tests::core_tests::unit53_capacity::new_only";
        rows.extend(padded(stress, 0, "Send", 200));
        rows.extend(padded(stress, 1, "Estimate", 200));
        let new_path = temp.0.join("new.jsonl");
        let mut new_rows = rows.clone();
        rows.extend(
            format!(r#"{{"test":"{old_only}","kind":"Estimate","debug":"opaque @"}}"#).bytes(),
        );
        new_rows.extend(
            format!(r#"{{"version":2,"test":"{stress}","kind":"Estimate","debug":null}}"#).bytes(),
        );
        new_rows.push(b'\n');
        new_rows.extend(
            format!(r#"{{"version":2,"test":"{new_only}","kind":"Send","debug":null}}"#).bytes(),
        );
        std::fs::write(&path, rows).unwrap();
        std::fs::write(&new_path, new_rows).unwrap();
        for strict in [true, false] {
            let mut old = Capture::index(path.to_str().unwrap(), &temp.0, strict).unwrap();
            let mut new = Capture::index(new_path.to_str().unwrap(), &temp.0, strict).unwrap();
            for stats in old.stats.values_mut() {
                stats.bytes = 16_000_001;
            }
            exclude_capacity(&mut old, &mut new);
            assert_eq!(old.tests.keys().collect::<Vec<_>>(), new.tests.keys().collect::<Vec<_>>());
            assert_eq!(old.tests.len(), names.len());
            let mut report = crate::json::object([] as [(String, crate::json::Json); 0]);
            add_exclusions(&mut report, &old, &new);
            let report: Value = serde_json::from_str(&report.render(None)).unwrap();
            assert_eq!(
                report["excluded_capacity_tests"],
                serde_json::json!({
                (stress): {"old_observations":2,"new_observations":3,"old_estimates":1,"new_estimates":2},
                (old_only): {"old_observations":1,"new_observations":0,"old_estimates":1,"new_estimates":0},
                (new_only): {"old_observations":0,"new_observations":1,"old_estimates":0,"new_estimates":0}})
            );
        }
    }

    #[test]
    fn sidecar_retains_all_statistics_and_invalidates_old_format() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let path = temp.0.join("events.jsonl");
        let cache = temp.0.join("cache");
        let mut bytes = padded("send_only", 0, "Send", 200);
        bytes.extend(padded("mixed", 0, "Send", 200));
        bytes.extend(padded("mixed", 1, "Estimate", 200));
        bytes.extend(padded("tests::core_tests::unit53_capacity::stress", 0, "Estimate", 200));
        std::fs::write(&path, &bytes).unwrap();
        Capture::index_cached(path.to_str().unwrap(), &temp.0, true, Some(&cache)).unwrap();
        let sidecar = std::fs::read_dir(&cache).unwrap().next().unwrap().unwrap().path();
        let mut saved: Value = serde_json::from_reader(File::open(&sidecar).unwrap()).unwrap();
        assert_eq!(
            saved["stats"]["send_only"],
            serde_json::json!({"bytes":200,"observations":1,"estimates":0,"largest_record":200})
        );
        let reused =
            Capture::index_cached(path.to_str().unwrap(), &temp.0, false, Some(&cache)).unwrap();
        assert_eq!(reused.stats["mixed"].observations, 2);
        assert_eq!(reused.stats["mixed"].estimates, 1);
        assert_eq!(reused.stats["tests::core_tests::unit53_capacity::stress"].estimates, 1);
        saved["format"] = (CACHE_VERSION - 1).into();
        saved["stats"] = serde_json::json!({});
        std::fs::write(&sidecar, serde_json::to_vec(&saved).unwrap()).unwrap();
        let capture =
            Capture::index_cached(path.to_str().unwrap(), &temp.0, true, Some(&cache)).unwrap();
        assert_eq!(capture.tests.len(), 2);
        assert_eq!(capture.stats["send_only"].bytes, 200);
        assert_eq!(capture.stats["mixed"].estimates, 1);
        let mut saved: Value = serde_json::from_reader(File::open(&sidecar).unwrap()).unwrap();
        saved["format"] = CACHE_VERSION.into();
        saved["stats"]["mixed"].as_object_mut().unwrap().remove("estimates");
        std::fs::write(&sidecar, serde_json::to_vec(&saved).unwrap()).unwrap();
        let capture =
            Capture::index_cached(path.to_str().unwrap(), &temp.0, true, Some(&cache)).unwrap();
        assert_eq!(capture.stats["mixed"].observations, 2);
        assert_eq!(capture.stats["mixed"].estimates, 1);
    }
}

#[cfg(test)]
mod reduction_tests {
    use super::tests::row;
    use super::*;

    #[test]
    fn parallel_reduction_preserves_decoded_values_counts_and_checksum() {
        let rows = [
            row("alpha", 0, "Reply { amount: 1.20, label: \"☃\" }"),
            row("beta", 0, "opaque @"),
            row("alpha", 1, "Some([9, -2])"),
            row("tests::core_tests::unit53_capacity::keep", 0, "None"),
        ];
        let mut source = rows.concat();
        source.pop(); // Preserve the historical treatment of a final row without LF.
        let mut expected = Vec::new();
        for line in source.split(|byte| *byte == b'\n') {
            let compact =
                crate::compact::compact_event(&serde_json::from_slice(line).unwrap()).unwrap();
            let mut bytes = serde_json::to_vec(&compact).unwrap();
            bytes.push(b'\n');
            expected.extend(bytes);
        }
        for jobs in [1, 3] {
            // Tiny limits exercise both accumulated admission and exclusive oversized
            // records without allocating a capacity-sized fixture.
            for limit in [17, source.len()] {
                let mut output = Vec::new();
                let result = reduce_lines(
                    BufReader::with_capacity(13, source.as_slice()),
                    &mut output,
                    jobs,
                    limit,
                )
                .unwrap();
                assert_ne!(output[4] & 4, 0); // The frame carries a content checksum.
                let mut decoder = zstd::stream::read::Decoder::with_buffer(output.as_slice())
                    .unwrap()
                    .single_frame();
                let mut decoded = Vec::new();
                decoder.read_to_end(&mut decoded).unwrap();
                assert_eq!(decoded, expected);
                assert!(decoder.finish().is_empty());
                assert!(zstd::stream::decode_all(&output[..output.len() - 1]).is_err());
                let last = output.len() - 1;
                output[last] ^= 1;
                assert!(zstd::stream::decode_all(output.as_slice()).is_err());
                output[last] ^= 1;
                assert_eq!(zstd::stream::decode_all(output.as_slice()).unwrap(), expected);
                assert_eq!(
                    result,
                    (
                        4,
                        source.len() as u64,
                        expected.len() as u64,
                        format!("{:x}", Sha256::digest(&expected))
                    )
                );
            }
        }
    }

    #[test]
    fn parallel_reduction_orders_errors_and_removes_only_its_failed_output() {
        let temp = crate::TempDir::new(&std::env::temp_dir()).unwrap();
        let source = temp.0.join("source.jsonl");
        let output = temp.0.join("compact.zst");
        for bytes in [
            b"{\"version\":2,\"test\":\"alpha\",\"index\":7,\"kind\":\"Send\"}\nbroken\n"
                .as_slice(),
            b"broken\n{\"version\":1,\"test\":\"alpha\",\"index\":7,\"kind\":\"Send\"}\n",
            b"{\"version\":1,\"test\":\"alpha\",\"index\":0,\"kind\":\"Send\"}\nbroken\n",
            b"",
        ] {
            std::fs::write(&source, bytes).unwrap();
            let mut errors = Vec::new();
            for jobs in [1, 3] {
                let args = crate::Args {
                    command: "reduce".into(),
                    options: BTreeMap::new(),
                    positional: vec![
                        source.to_str().unwrap().into(),
                        output.to_str().unwrap().into(),
                    ],
                    shard: (0, 1),
                    jobs,
                    differences_only: false,
                };
                errors.push(reduce(&args).unwrap_err().to_string());
                assert!(!output.exists());
                assert_eq!(std::fs::read(&source).unwrap(), bytes);
                std::fs::write(&output, b"keep existing").unwrap();
                assert!(reduce(&args).is_err());
                assert_eq!(std::fs::read(&output).unwrap(), b"keep existing");
                std::fs::remove_file(&output).unwrap();
            }
            assert_eq!(errors[0], errors[1]);
            if bytes.starts_with(b"{\"version\":2") {
                assert_eq!(errors[0], "nonsequential/unsupported reduction source: alpha #7");
            }
        }
    }

    #[test]
    fn ordered_line_mapping_uses_workers_and_keeps_result_order() {
        let barrier = std::sync::Barrier::new(2);
        let mut offsets = Vec::new();
        ordered_lines(
            b"first\nsecond\n".as_slice(),
            2,
            64,
            |_, offset| {
                barrier.wait();
                Ok(offset)
            },
            |offset| {
                offsets.push(offset);
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(offsets, [0, 6]);
    }

    #[test]
    fn byte_admission_cancels_while_waiting_for_an_oversized_row() {
        let admission = Admission::default();
        assert!(admission.reserve(2, 4, None, 0));
        assert!(admission.reserve(2, 4, Some(0), 9));
        assert!(admission.reserve(2, 4, None, 0));
        std::thread::scope(|scope| {
            let waiter = scope.spawn(|| admission.reserve(2, 4, Some(0), 1));
            admission.cancel();
            assert!(!waiter.join().unwrap());
        });
    }
}
