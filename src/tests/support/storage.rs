use crate::return_records::{self, Records};
use crate::storage::{
    GenerationMismatch, LoadedReturn, RecordTransaction, ReturnStore, persist_in, persist_with_in,
    read_for_open_in, read_in, reset_mismatched_in,
};
use crate::*;
use alloc::{collections::BTreeMap, vec::Vec};
use core::borrow::Borrow;
use libertas::LibertasUser;
use libertas::data::{KeyCodec, KeyDecode, wire};

// Native bounded Data boundary, not a real Hub. All values and requests use the
// SDK codec. A private transaction snapshot becomes visible only on commit.
#[derive(Clone, Default)]
pub(in crate::tests) struct Store {
    pub(in crate::tests) values: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(in crate::tests) reads: usize,
    pub(in crate::tests) writes: usize,
    pub(in crate::tests) pages: usize,
    pub(in crate::tests) mutations: usize,
    pub(in crate::tests) mutation_bytes: usize,
    pub(in crate::tests) fail_read: bool,
    pub(in crate::tests) fail_after: Option<usize>,
    pub(in crate::tests) fail_commit: bool,
    pub(in crate::tests) backend_value_limit: Option<usize>,
    pub(in crate::tests) cookies: usize,
    pub(in crate::tests) accepted_callbacks: usize,
    pub(in crate::tests) reset_removed: Option<usize>,
}
pub(in crate::tests) struct NativeTransaction {
    pub(in crate::tests) values: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(in crate::tests) mutations: usize,
    pub(in crate::tests) mutation_bytes: usize,
    pub(in crate::tests) pages: usize,
    pub(in crate::tests) scans: usize,
    pub(in crate::tests) key_scans: usize,
    pub(in crate::tests) block_versioned_reads: bool,
    pub(in crate::tests) fail_read: bool,
    pub(in crate::tests) fail_after: Option<usize>,
    pub(in crate::tests) backend_value_limit: usize,
}
impl NativeTransaction {
    pub(in crate::tests) fn new(store: &Store) -> Self {
        Self {
            values: store.values.clone(),
            mutations: 0,
            mutation_bytes: 0,
            pages: 0,
            scans: 0,
            key_scans: 0,
            block_versioned_reads: false,
            fail_read: store.fail_read,
            fail_after: store.fail_after,
            backend_value_limit: store.backend_value_limit.unwrap_or(wire::MAX_VALUE_BYTES),
        }
    }
    fn admit(&mut self, operation: wire::Operation, bytes: usize) {
        let request = wire::Request {
            version: wire::VERSION,
            id: 1,
            checked: false,
            command: wire::Command::ExecuteInTransaction { transaction: 1, operation },
        };
        assert!(request.to_avro().len() <= wire::MAX_REQUEST_BYTES, "Data ResourceLimit frame");
        self.mutations += 1;
        self.mutation_bytes += bytes;
        assert!(self.mutation_bytes <= 16 * 1024 * 1024, "Data ResourceLimit mutation bytes");
        assert_ne!(self.fail_after, Some(self.mutations), "injected fatal Data write failure");
    }
}
impl RecordTransaction for NativeTransaction {
    fn get(&mut self, key: &TaxDataKey) -> Option<TaxAppData> {
        assert!(
            !self.block_versioned_reads
                || matches!(key, TaxDataKey::ReturnRecord { copy: 0, kind: 0, id: 0, .. }),
            "versioned header/body decode attempted"
        );
        assert!(!self.fail_read, "injected fatal Data read failure");
        self.values
            .get(&key.to_key_bytes().unwrap())
            .map(|value| TaxAppData::from_avro(value).expect("fatal corrupt Data value"))
    }
    fn has_records(&mut self, user: LibertasUser, year: i32) -> bool {
        self.key_scans += 1;
        self.pages += 1;
        assert!(!self.fail_read, "injected fatal Data key scan failure");
        let prefix = TaxDataKey::return_record_prefix(user, year).unwrap();
        self.values.keys().any(|key| key.starts_with(&prefix))
    }
    fn delete_records(&mut self, user: LibertasUser, year: i32) -> usize {
        self.key_scans += 1;
        assert!(!self.fail_read, "injected fatal Data key scan failure");
        let prefix = TaxDataKey::return_record_prefix(user, year).unwrap();
        let keys: Vec<_> =
            self.values.keys().filter(|key| key.starts_with(&prefix)).cloned().collect();
        let count = keys.len();
        for key in keys {
            self.delete(&TaxDataKey::decode_key(&key).unwrap());
        }
        count
    }
    fn records(&mut self, user: LibertasUser, year: i32) -> Records {
        self.scans += 1;
        assert!(!self.block_versioned_reads, "versioned body scan attempted");
        assert!(!self.fail_read, "injected fatal Data scan failure");
        let prefix = TaxDataKey::return_record_prefix(user, year).unwrap();
        let mut page_entries = 0;
        let mut page_bytes = 0;
        let mut records = Vec::new();
        for (key, bytes) in self.values.iter().filter(|(key, _)| key.starts_with(&prefix)) {
            let size = key.len() + bytes.len();
            if page_entries == 4096 || page_bytes + size > 8 * 1024 * 1024 {
                self.pages += 1;
                page_entries = 0;
                page_bytes = 0;
            }
            assert!(size <= 8 * 1024 * 1024);
            page_entries += 1;
            page_bytes += size;
            records.push((
                TaxDataKey::decode_key(key).unwrap(),
                TaxAppData::from_avro(bytes).expect("fatal corrupt Data scan value"),
            ));
        }
        self.pages += 1;
        records
    }
    fn put(&mut self, key: &TaxDataKey, value: &TaxAppData) {
        let key = key.to_key_bytes().unwrap();
        let value = value.to_avro();
        assert!(value.len() <= wire::MAX_VALUE_BYTES, "Data ResourceLimit value");
        assert!(value.len() <= self.backend_value_limit, "backend ResourceLimit value");
        self.admit(
            wire::Operation::Put { key: key.clone(), value: value.clone() },
            key.len() + value.len(),
        );
        self.values.insert(key, value);
    }
    fn delete(&mut self, key: &TaxDataKey) {
        let key = key.to_key_bytes().unwrap();
        self.admit(wire::Operation::Delete(key.clone()), key.len());
        self.values.remove(&key);
    }
}
impl Store {
    pub(in crate::tests) fn commit(&mut self, tx: NativeTransaction) {
        assert!(!self.fail_commit, "injected fatal Data commit failure");
        self.values = tx.values;
        self.mutations = tx.mutations;
        self.mutation_bytes = tx.mutation_bytes;
        self.pages += tx.pages;
        if tx.mutations > 0 {
            self.writes += 1;
        }
    }
    pub(in crate::tests) fn seed(&mut self, user: LibertasUser, draft: FederalDraft) {
        for (key, value) in return_records::project(user, &draft, 1).unwrap() {
            self.values.insert(key.to_key_bytes().unwrap(), value.to_avro());
        }
    }
    pub(in crate::tests) fn saved(&self, user: LibertasUser) -> FederalDraft {
        self.saved_year(user, 2026)
    }
    pub(in crate::tests) fn saved_year(&self, user: LibertasUser, year: i32) -> FederalDraft {
        read_in(&mut NativeTransaction::new(self), user, year).unwrap().unwrap().draft
    }
    pub(in crate::tests) fn bytes(&self, user: LibertasUser) -> usize {
        let prefix = TaxDataKey::return_record_prefix(user, 2026).unwrap();
        self.values
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .map(|(_, value)| value.len())
            .sum()
    }
}
impl ReturnStore for Store {
    fn try_load(
        &mut self,
        user: LibertasUser,
        year: i32,
    ) -> Result<LoadedReturn, GenerationMismatch> {
        self.reads += 1;
        let mut tx = NativeTransaction::new(self);
        let loaded = read_for_open_in(&mut tx, user, year);
        self.commit(tx);
        Ok(loaded?.unwrap_or_else(|| {
            self.cookies += 1;
            let mut draft = FederalDraft::empty(alloc::format!("fresh-{}", self.cookies));
            draft.tax_year = year;
            LoadedReturn::fresh(draft, user)
        }))
    }
    fn reset_mismatched(
        &mut self,
        user: LibertasUser,
        year: i32,
        expected: GenerationMismatch,
    ) -> Option<usize> {
        let mut tx = NativeTransaction::new(self);
        let removed =
            reset_mismatched_in(&mut tx, user, year, expected).expect("fatal keyed corruption");
        self.commit(tx);
        self.reset_removed = removed;
        removed
    }
    fn persist(
        &mut self,
        user: LibertasUser,
        base: Option<&WorkingHeader>,
        draft: &FederalDraft,
        changed: bool,
    ) -> bool {
        let mut tx = NativeTransaction::new(self);
        let saved =
            persist_in(&mut tx, user, base, draft, changed).expect("fatal keyed corruption");
        self.commit(tx);
        saved
    }
    fn persist_with<T, D: Borrow<FederalDraft>>(
        &mut self,
        user: LibertasUser,
        base: Option<&WorkingHeader>,
        year: i32,
        after_accept: impl FnOnce() -> (D, bool, T),
    ) -> Option<T> {
        let mut tx = NativeTransaction::new(self);
        let mut invoked = false;
        let saved = persist_with_in(&mut tx, user, base, year, || {
            invoked = true;
            after_accept()
        })
        .expect("fatal keyed corruption");
        self.accepted_callbacks += usize::from(invoked);
        self.commit(tx);
        saved
    }
}
