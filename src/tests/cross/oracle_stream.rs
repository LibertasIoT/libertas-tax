// Replay keeps native inputs on disk and reads one manifest case at a time.
use super::*;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use std::io::{BufReader, BufWriter};

pub(super) struct Corpus {
    root: PathBuf,
}
impl Drop for Corpus {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
struct Cases<F>(F);
impl<'de, F: FnMut(Case)> DeserializeSeed<'de> for Cases<F> {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, decoder: D) -> Result<(), D::Error> {
        decoder.deserialize_seq(self)
    }
}
impl<'de, F: FnMut(Case)> Visitor<'de> for Cases<F> {
    type Value = ();
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an oracle case array")
    }
    fn visit_seq<A: SeqAccess<'de>>(mut self, mut cases: A) -> Result<(), A::Error> {
        while let Some(case) = cases.next_element()? {
            (self.0)(case);
        }
        Ok(())
    }
}
impl Corpus {
    pub(super) fn open(path: &Path) -> std::sync::Arc<Self> {
        static SHARED: std::sync::Mutex<BTreeMap<PathBuf, std::sync::Weak<Corpus>>> =
            std::sync::Mutex::new(BTreeMap::new());
        let mut shared = SHARED.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        shared.retain(|_, corpus| corpus.strong_count() != 0);
        if let Some(corpus) = shared.get(path).and_then(std::sync::Weak::upgrade) {
            return corpus;
        }
        let corpus = std::sync::Arc::new(Self::stage(path));
        shared.insert(path.to_path_buf(), std::sync::Arc::downgrade(&corpus));
        corpus
    }
    fn stage(path: &Path) -> Self {
        let corpus = Self { root: control_root() };
        fs::create_dir_all(corpus.root.join("inputs")).unwrap();
        let mut reader = zstd::stream::read::Decoder::new(fs::File::open(path).unwrap()).unwrap();
        let mut header = [0; 12];
        reader.read_exact(&mut header).unwrap();
        assert_eq!(header, HEADER, "unsupported oracle stream");
        let count = read_u64(&mut reader);
        let mut buffer = [0; 65536];
        for _ in 0..count {
            let length = read_u64(&mut reader).checked_sub(64).expect("short oracle input");
            let mut id = [0; 64];
            reader.read_exact(&mut id).unwrap();
            let id = std::str::from_utf8(&id).unwrap();
            assert!(id.bytes().all(|c| c.is_ascii_hexdigit()), "invalid oracle input ID");
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(corpus.root.join("inputs").join(id))
                .expect("duplicate oracle input or unwritable input file");
            let mut hash = Sha256::new();
            hash.update(b"tax-oracle-v1/avro\0");
            let mut bytes = reader.by_ref().take(length);
            let mut copied = 0;
            loop {
                let size = bytes.read(&mut buffer).unwrap();
                if size == 0 {
                    break;
                }
                file.write_all(&buffer[..size]).unwrap();
                hash.update(&buffer[..size]);
                copied += size as u64;
            }
            assert_eq!(copied, length, "truncated oracle input");
            assert_eq!(hex(&hash.finalize()), id);
        }
        let length = read_u64(&mut reader);
        let mut manifest = fs::File::create(corpus.root.join("cases.json")).unwrap();
        assert_eq!(
            std::io::copy(&mut reader.by_ref().take(length), &mut manifest).unwrap(),
            length
        );
        assert_eq!(reader.read(&mut [0]).unwrap(), 0, "trailing oracle records");
        let mut seen = BTreeSet::new();
        corpus.cases(|case| {
            assert_eq!(case.version, 1, "unsupported oracle version");
            let id: [u8; 32] = Sha256::digest(serde_json::to_vec(&case.input).unwrap()).into();
            assert!(seen.insert(id), "duplicate oracle case");
            assert!(
                case.tests.windows(2).all(|w| w[0] < w[1]),
                "duplicate or unsorted oracle origins"
            );
            assert!(case.tests.iter().all(|name| name.contains("::")), "invalid oracle origin");
            let check = |id: &str| {
                assert!(id.len() == 64 && id.bytes().all(|c| c.is_ascii_hexdigit()));
                assert!(corpus.root.join("inputs").join(id).is_file(), "missing oracle input");
            };
            match case.input {
                Input::Estimate { draft } => check(&draft),
                Input::Send { session, request } => {
                    check(&session.draft);
                    check(&request);
                }
            }
        });
        assert!(!seen.is_empty(), "figure oracle corpus must not be empty");
        corpus
    }
    fn cases(&self, visit: impl FnMut(Case)) {
        let file = BufReader::new(fs::File::open(self.root.join("cases.json")).unwrap());
        let mut decoder = serde_json::Deserializer::from_reader(file);
        Cases(visit).deserialize(&mut decoder).unwrap();
        decoder.end().unwrap();
    }
    fn materialize(&self, case: Case) -> super::Corpus {
        let mut inputs = BTreeMap::new();
        let mut load = |id: &str| {
            inputs
                .entry(id.to_string())
                .or_insert_with(|| fs::read(self.root.join("inputs").join(id)).unwrap());
        };
        match &case.input {
            Input::Estimate { draft } => load(draft),
            Input::Send { session, request } => {
                load(&session.draft);
                load(request);
            }
        }
        super::Corpus { inputs, cases: std::vec![case] }
    }
    pub(super) fn replay(&self, shard: usize, of: usize, directory: Option<&Path>) {
        let suffix = if of == 1 { String::new() } else { format!("-{shard}") };
        let mut files = directory.map(|directory| {
            fs::create_dir_all(directory).unwrap();
            (
                BufWriter::new(
                    fs::File::create(directory.join(format!("cases{suffix}.jsonl"))).unwrap(),
                ),
                BufWriter::new(
                    fs::File::create(directory.join(format!("changed{suffix}.jsonl"))).unwrap(),
                ),
            )
        });
        let workers = (std::thread::available_parallelism().unwrap().get() / of).max(1);
        let changes = AtomicUsize::new(0);
        let failure = std::sync::Mutex::new(None);
        // A rendezvous channel bounds queued cases independently of corpus size.
        let (sender, receiver) = std::sync::mpsc::sync_channel::<(usize, Case)>(0);
        let receiver = std::sync::Arc::new(std::sync::Mutex::new(receiver));
        let files_lock = std::sync::Mutex::new((0, &mut files));
        let ordered = std::sync::Condvar::new();
        std::thread::scope(|scope| {
            for _ in 0..workers {
                let receiver = std::sync::Arc::clone(&receiver);
                let (failure, files_lock, changes, ordered) =
                    (&failure, &files_lock, &changes, &ordered);
                scope.spawn(move || {
                    let _guard = ReplayGuard::new();
                    loop {
                        let Ok((index, case)) = receiver.lock().unwrap().recv() else { break };
                        let corpus = self.materialize(case);
                        let case = &corpus.cases[0];
                        let actual = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            super::replay(&corpus, &case.input)
                        })).map_err(|panic| {
                            panic.downcast_ref::<String>().cloned()
                                .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                                .unwrap_or_else(|| "replay panicked".to_string())
                        });
                        if directory.is_none() {
                            if actual.as_ref() != Ok(&case.expected) {
                                failure.lock().unwrap().get_or_insert_with(|| {
                                    format!("figure oracle changed: {} expected={:?} actual={actual:?}", case_id(case), case.expected)
                                });
                            }
                        } else {
                            let mut state = files_lock.lock().unwrap();
                            while state.0 != index {
                                state = ordered.wait(state).unwrap();
                            }
                            let (cases, changed) = state.1.as_mut().unwrap();
                            serde_json::to_writer(&mut *cases, &serde_json::json!({"input": case_id(case), "tests": case.tests})).unwrap();
                            cases.write_all(b"\n").unwrap();
                            if actual.as_ref() != Ok(&case.expected) {
                                changes.fetch_add(1, Ordering::Relaxed);
                                serde_json::to_writer(&mut *changed, &serde_json::json!({"input": case_id(case), "tests": case.tests, "expected": case.expected, "actual": actual})).unwrap();
                                changed.write_all(b"\n").unwrap();
                            }
                            state.0 += 1;
                            ordered.notify_all();
                        }
                    }
                });
            }
            drop(receiver);
            let mut index = 0;
            self.cases(|case| {
                if index % of == shard {
                    sender.send((index / of, case)).expect("oracle replay worker failed");
                }
                index += 1;
            });
            drop(sender);
        });
        if let Some((cases, changed)) = &mut files {
            cases.flush().unwrap();
            changed.flush().unwrap();
        }
        if let Some(message) = failure.into_inner().unwrap() {
            panic!("{message}");
        }
        let changes = changes.load(Ordering::Relaxed);
        assert!(
            changes == 0,
            "figure oracle listed {changes} changed case(s) in {}",
            directory.unwrap().join(format!("changed{suffix}.jsonl")).display()
        );
    }
}
