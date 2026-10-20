use crate::interview::{Interview, Reply};
use crate::return_records::{self, RecordResult, Records};
use crate::{FederalDraft, TaxAppData, TaxDataKey, TaxInterviewProtocol, WorkingHeader};
use alloc::vec::Vec;
use core::{num::NonZeroUsize, ops::Bound};
use libertas::LibertasUser;
use libertas::data::{Data, EncodedKey, KeyRange, Mode, ScanOptions, Target, Transaction};
use lru::LruCache;

// The current endpoint authorizes only its existing year. Storage helpers take
// an explicit year; multi-year financial rule selection remains its own owner.
fn active_year() -> i32 {
    crate::year_constants::TAX_YEAR as i32
}

pub(crate) struct LoadedReturn {
    pub(crate) draft: FederalDraft,
    pub(crate) header: Option<WorkingHeader>,
    pub(crate) encoded_bytes: usize,
}
impl LoadedReturn {
    pub(crate) fn fresh(draft: FederalDraft, user: LibertasUser) -> Self {
        let records = return_records::project(user, &draft, 1).expect("fresh return projects");
        Self { draft, header: None, encoded_bytes: return_records::value_bytes(&records) }
    }
}

// This boundary is also used by native bounded transaction tests. Production
// calls ordinary SDK methods, whose failures terminate the task.
pub(crate) trait RecordTransaction {
    fn get(&mut self, key: &TaxDataKey) -> Option<TaxAppData>;
    fn exists(&mut self, key: &TaxDataKey) -> bool;
    fn records(&mut self, user: LibertasUser, year: i32) -> Records;
    fn put(&mut self, key: &TaxDataKey, value: &TaxAppData);
    fn delete(&mut self, key: &TaxDataKey);
}
impl RecordTransaction for Transaction<TaxDataKey, TaxAppData> {
    fn exists(&mut self, key: &TaxDataKey) -> bool {
        // Transaction has no exists operation; both legacy keys here are checked
        // through a keys-only range, so unknown body bytes are never decoded.
        let encoded = EncodedKey::new(key).expect("data key");
        let range = KeyRange::new(Bound::Included(encoded.clone()), Bound::Included(encoded));
        !self.scan_keys(&range, &ScanOptions::new(1)).items.is_empty()
    }
    fn get(&mut self, key: &TaxDataKey) -> Option<TaxAppData> {
        self.get(key.into())
    }
    fn records(&mut self, user: LibertasUser, year: i32) -> Records {
        // A bounded user/year range also confines the isolated ALL migration
        // transaction. No scan ever enumerates another user's return.
        let key =
            |copy, kind, id| TaxDataKey::ReturnRecord { user, tax_year: year, copy, kind, id };
        let range = KeyRange::new(
            Bound::Included(EncodedKey::new(&key(i64::MIN, 0, i64::MIN)).expect("return key")),
            Bound::Included(
                EncodedKey::new(&key(i64::MAX, u16::MAX, i64::MAX)).expect("return key"),
            ),
        );
        let mut options = ScanOptions::new(4096);
        let mut records = Vec::new();
        loop {
            // The platform enforces both 4096 entries and its 8 MiB page budget.
            let page = self.scan(&range, &options);
            records.extend(
                page.items
                    .into_iter()
                    .map(|entry| (entry.key.decode().expect("corrupt return key"), entry.value)),
            );
            options.continuation = page.next;
            if options.continuation.is_none() {
                break;
            }
        }
        records
    }
    fn put(&mut self, key: &TaxDataKey, value: &TaxAppData) {
        self.put(key.into(), value);
    }
    fn delete(&mut self, key: &TaxDataKey) {
        self.delete(key.into());
    }
}

pub(crate) fn read_in(
    tx: &mut impl RecordTransaction,
    user: LibertasUser,
    year: i32,
) -> RecordResult<Option<LoadedReturn>> {
    let metadata = match tx.get(&return_records::header_key(user, year)) {
        Some(TaxAppData::WorkingHeader { header }) => {
            return_records::known_header(&header, year)?;
            Some(header)
        }
        None => None,
        _ => return Err("Invalid working header payload"),
    };
    let all = tx.records(user, year);
    if metadata.is_none() {
        return if all.is_empty() { Ok(None) } else { Err("Records without working header") };
    }
    validate_namespace(user, year, &all)?;
    let records = all
        .into_iter()
        .filter(|(key, _)| matches!(key, TaxDataKey::ReturnRecord { copy: 0, .. }))
        .collect();
    let bytes = return_records::value_bytes(&records);
    let (draft, header) = return_records::assemble(user, year, records)?;
    if Some(&header) != metadata.as_ref() {
        return Err("Header changed within transaction");
    }
    Ok(Some(LoadedReturn { draft, header: Some(header), encoded_bytes: bytes }))
}

fn validate_namespace(user: LibertasUser, year: i32, all: &Records) -> RecordResult<()> {
    for (key, _) in all {
        if !matches!(key, TaxDataKey::ReturnRecord { user: owner, tax_year, copy, .. }
            if *owner == user && *tax_year == year && *copy >= 0)
        {
            return Err("Invalid return namespace");
        }
    }
    Ok(())
}

pub(crate) fn persist_in(
    tx: &mut impl RecordTransaction,
    user: LibertasUser,
    base: Option<&WorkingHeader>,
    draft: &FederalDraft,
    changed: bool,
) -> RecordResult<bool> {
    let key = return_records::header_key(user, draft.tax_year);
    let current = match tx.get(&key) {
        Some(TaxAppData::WorkingHeader { header }) => {
            return_records::known_header(&header, draft.tax_year)?;
            Some(header)
        }
        None => None,
        _ => return Err("Invalid working header"),
    };
    if current.as_ref().map(|h| (&h.cookie, h.revision)) != base.map(|h| (&h.cookie, h.revision)) {
        return Ok(false);
    }
    let old = tx.records(user, draft.tax_year);
    validate_namespace(user, draft.tax_year, &old)?;
    if current.is_none() && !old.is_empty() {
        return Err("Records without working header");
    }
    if let Some(current) = &current {
        let working = old
            .iter()
            .filter(|(key, _)| matches!(key, TaxDataKey::ReturnRecord { copy: 0, .. }))
            .cloned()
            .collect();
        let (_, checked_header) = return_records::assemble(user, draft.tax_year, working)?;
        if &checked_header != current {
            return Err("Header changed within transaction");
        }
    }
    if !changed {
        return Ok(true);
    }
    let next_copy = current.as_ref().map_or(1, |h| h.next_filed_copy);
    let next = return_records::project(user, draft, next_copy)?;
    // SDK-encoded identities avoid quadratic comparisons for heavy returns.
    let old: alloc::collections::BTreeMap<_, _> = old
        .iter()
        .map(|(key, value)| (EncodedKey::new(key).expect("return key"), (key, value)))
        .collect();
    let next: alloc::collections::BTreeMap<_, _> = next
        .iter()
        .map(|(key, value)| (EncodedKey::new(key).expect("return key"), (key, value)))
        .collect();
    // Positive filed-copy namespaces are never touched by a working Save.
    for (encoded, (key, _)) in &old {
        if matches!(key, TaxDataKey::ReturnRecord { copy: 0, .. }) && !next.contains_key(encoded) {
            tx.delete(key);
        }
    }
    for (encoded, (key, value)) in &next {
        if old.get(encoded).is_none_or(|(_, v)| *v != *value) {
            tx.put(key, value);
        }
    }
    Ok(true)
}

pub(crate) trait ReturnStore {
    fn load_or_initialize(&mut self, user: LibertasUser, year: i32) -> LoadedReturn;
    fn persist(
        &mut self,
        user: LibertasUser,
        base: Option<&WorkingHeader>,
        draft: &FederalDraft,
        changed: bool,
    ) -> bool;
}

pub(crate) struct DataStore {
    data: Data<TaxDataKey, TaxAppData>,
}
impl ReturnStore for DataStore {
    fn load_or_initialize(&mut self, user: LibertasUser, year: i32) -> LoadedReturn {
        assert!(user > 0, "verified positive user required");
        let prefix = TaxDataKey::return_record_prefix(user, year).expect("return prefix");
        let target = Target::prefix(&prefix).expect("return target");
        let mut tx = self.data.begin(&target, Mode::ReadOnly);
        let loaded = read_in(&mut tx, user, year).expect("corrupt stored return");
        tx.commit();
        loaded.unwrap_or_else(|| {
            // Never mistake a not-yet-migrated monolith for an empty return.
            // This presence check does not decode or migrate legacy payloads.
            let legacy = TaxDataKey::Draft { user };
            let target = Target::keys(&[(&legacy).into()]).expect("legacy presence target");
            assert!(
                !self.data.exists(&target, (&legacy).into()),
                "quiesced format-124 migration required"
            );
            let mut draft = FederalDraft::empty(alloc::format!(
                "{:016x}{:016x}",
                libertas::libertas_get_random(8),
                libertas::libertas_get_random(8)
            ));
            draft.tax_year = year;
            LoadedReturn::fresh(draft, user)
        })
    }
    fn persist(
        &mut self,
        user: LibertasUser,
        base: Option<&WorkingHeader>,
        draft: &FederalDraft,
        changed: bool,
    ) -> bool {
        let prefix = TaxDataKey::return_record_prefix(user, draft.tax_year).expect("return prefix");
        let target = Target::prefix(&prefix).expect("return target");
        let mut tx = self.data.begin(&target, Mode::ReadWrite);
        let saved = persist_in(&mut tx, user, base, draft, changed).expect("corrupt stored return");
        tx.commit();
        saved
    }
}

struct CachedReturn {
    interview: Interview,
    encoded_bytes: usize,
    base: Option<crate::WorkingHeader>,
}

pub(crate) struct Returns<S = DataStore> {
    open: LruCache<(LibertasUser, i32), CachedReturn>,
    budget: NonZeroUsize,
    encoded_bytes: usize,
    store: S,
}

impl Returns<DataStore> {
    pub(crate) fn new(budget: NonZeroUsize) -> Self {
        Self::with_store(budget, DataStore { data: Data::current() })
    }
}

impl<S: ReturnStore> Returns<S> {
    pub(crate) fn with_store(budget: NonZeroUsize, store: S) -> Self {
        Self { open: LruCache::unbounded(), budget, encoded_bytes: 0, store }
    }

    fn load(&mut self, user: LibertasUser) -> CachedReturn {
        let loaded = self.store.load_or_initialize(user, active_year());
        CachedReturn {
            encoded_bytes: loaded.encoded_bytes,
            base: loaded.header,
            interview: Interview::new(user, loaded.draft),
        }
    }

    fn retain(&mut self, user: LibertasUser, state: CachedReturn) {
        if state.encoded_bytes > self.budget.get() {
            return;
        }
        self.encoded_bytes += state.encoded_bytes;
        self.open.put((user, active_year()), state);
        self.evict();
    }

    fn evict(&mut self) {
        while self.encoded_bytes > self.budget.get() {
            let (_, evicted) = self.open.pop_lru().expect("over-budget cache is nonempty");
            self.encoded_bytes -= evicted.encoded_bytes;
        }
    }

    fn committed(user: LibertasUser, state: &mut CachedReturn) {
        let next_copy = state.base.as_ref().map_or(1, |header| header.next_filed_copy);
        let records = return_records::project(user, &state.interview.draft, next_copy)
            .expect("accepted return projects");
        state.encoded_bytes = return_records::value_bytes(&records);
        state.base = Some(return_records::header(&state.interview.draft, next_copy));
        state.interview.dirty = false;
        state.interview.accepted_save = false;
    }

    fn respond(
        store: &mut S,
        user: LibertasUser,
        state: &mut CachedReturn,
        request: TaxInterviewProtocol,
    ) -> Option<Reply> {
        if state.base.is_none() && matches!(&request, TaxInterviewProtocol::OpenInterview { .. }) {
            // The editor's server cookie must survive even when this return is
            // too large for the cache. Estimates/Enter do not create records.
            if store.persist(user, None, &state.interview.draft, true) {
                Self::committed(user, state);
            } else {
                // Another opener established the return. Open its current state;
                // no user mutation has run, and there is no initialization retry.
                let current = store.load_or_initialize(user, active_year());
                *state = CachedReturn {
                    interview: Interview::new(user, current.draft),
                    encoded_bytes: current.encoded_bytes,
                    base: current.header,
                };
            }
        }
        let reply = state.interview.handle(request)?;
        if state.interview.dirty || state.interview.accepted_save {
            let changed = state.interview.dirty;
            if !store.persist(user, state.base.as_ref(), &state.interview.draft, changed) {
                // The candidate reply is discarded. Reload current accepted data,
                // return the existing stale-editor diagnostic, and never replay.
                let current = store.load_or_initialize(user, active_year());
                *state = CachedReturn {
                    interview: Interview::new(user, current.draft),
                    encoded_bytes: current.encoded_bytes,
                    base: current.header,
                };
                return Some(crate::interview::stale_editor_reply());
            }
            state.interview.accepted_save = false;
            if !changed {
                return Some(reply);
            }
            Self::committed(user, state);
        }
        Some(reply)
    }

    pub(crate) fn handle(
        &mut self,
        user: LibertasUser,
        request: TaxInterviewProtocol,
    ) -> Option<Reply> {
        if user <= 0 || request.user() != Some(user) {
            return None;
        }
        if let Some(state) = self.open.get_mut(&(user, active_year())) {
            let before = state.encoded_bytes;
            let reply = Self::respond(&mut self.store, user, state, request);
            self.encoded_bytes = self.encoded_bytes - before + state.encoded_bytes;
            // A growing saved return must not evict unrelated entries before
            // discovering that it cannot itself be retained.
            if state.encoded_bytes > self.budget.get() {
                let state = self.open.pop(&(user, active_year())).expect("current return exists");
                self.encoded_bytes -= state.encoded_bytes;
            }
            self.evict();
            reply
        } else {
            // Oversized returns belong to this request and are dropped afterwards.
            let mut state = self.load(user);
            let reply = Self::respond(&mut self.store, user, &mut state, request);
            self.retain(user, state);
            reply
        }
    }
}

#[cfg(test)]
impl<S: ReturnStore> Returns<S> {
    pub(crate) fn interview(&mut self, user: LibertasUser) -> &mut Interview {
        if !self.open.contains(&(user, active_year())) {
            let state = self.load(user);
            self.retain(user, state);
        }
        &mut self
            .open
            .get_mut(&(user, active_year()))
            .expect("test return fits cache budget")
            .interview
    }

    pub(crate) fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }

    pub(crate) fn store(&self) -> &S {
        &self.store
    }

    pub(crate) fn store_mut(&mut self) -> &mut S {
        &mut self.store
    }

    pub(crate) fn len(&self) -> usize {
        self.open.len()
    }
}
