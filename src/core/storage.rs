use crate::interview::{Interview, Reply};
use crate::return_records::{self, RecordResult, Records};
use crate::{FederalDraft, TaxAppData, TaxDataKey, TaxInterviewProtocol, WorkingHeader};
use alloc::{boxed::Box, vec::Vec};
use core::{borrow::Borrow, num::NonZeroUsize, ops::Bound};
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
    fn has_records(&mut self, user: LibertasUser, year: i32) -> bool;
    fn records(&mut self, user: LibertasUser, year: i32) -> Records;
    fn delete_records(&mut self, user: LibertasUser, year: i32) -> usize;
    fn put(&mut self, key: &TaxDataKey, value: &TaxAppData);
    fn delete(&mut self, key: &TaxDataKey);
}
impl RecordTransaction for Transaction<TaxDataKey, TaxAppData> {
    fn get(&mut self, key: &TaxDataKey) -> Option<TaxAppData> {
        self.get(key.into())
    }
    fn has_records(&mut self, user: LibertasUser, year: i32) -> bool {
        !self.scan_keys(&record_range(user, year), &ScanOptions::new(1)).items.is_empty()
    }
    fn records(&mut self, user: LibertasUser, year: i32) -> Records {
        let range = record_range(user, year);
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
    fn delete_records(&mut self, user: LibertasUser, year: i32) -> usize {
        let range = record_range(user, year);
        let mut options = ScanOptions::new(4096);
        let mut removed = 0;
        loop {
            let page = self.scan_keys(&range, &options);
            for key in page.items {
                self.delete((&key).into());
                removed += 1;
            }
            options.continuation = page.next;
            if options.continuation.is_none() {
                return removed;
            }
        }
    }
    fn put(&mut self, key: &TaxDataKey, value: &TaxAppData) {
        self.put(key.into(), value);
    }
    fn delete(&mut self, key: &TaxDataKey) {
        self.delete(key.into());
    }
}

fn record_range(user: LibertasUser, year: i32) -> KeyRange<TaxDataKey> {
    // A bounded user/year range: no scan ever enumerates another user's return.
    let key = |copy, kind, id| TaxDataKey::ReturnRecord { user, tax_year: year, copy, kind, id };
    KeyRange::new(
        Bound::Included(EncodedKey::new(&key(i64::MIN, 0, i64::MIN)).expect("return key")),
        Bound::Included(EncodedKey::new(&key(i64::MAX, u16::MAX, i64::MAX)).expect("return key")),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GenerationMismatch {
    pub(crate) format: Option<u32>,
}

enum Generation {
    Fresh,
    Current,
    Mismatch(GenerationMismatch),
}

fn generation_in(
    tx: &mut impl RecordTransaction,
    user: LibertasUser,
    year: i32,
) -> RecordResult<Generation> {
    // Only the stable single-u32 marker is decoded before admitting this layout.
    match tx.get(&return_records::format_key(user, year)) {
        Some(TaxAppData::StorageFormat { format }) if format != crate::STORAGE_FORMAT => {
            Ok(Generation::Mismatch(GenerationMismatch { format: Some(format) }))
        }
        Some(value) => {
            return_records::known_format(&value)?;
            Ok(Generation::Current)
        }
        None if tx.has_records(user, year) => {
            Ok(Generation::Mismatch(GenerationMismatch { format: None }))
        }
        None => Ok(Generation::Fresh),
    }
}

fn current_generation(
    tx: &mut impl RecordTransaction,
    user: LibertasUser,
    year: i32,
) -> RecordResult<bool> {
    match generation_in(tx, user, year)? {
        Generation::Fresh => Ok(false),
        Generation::Current => Ok(true),
        Generation::Mismatch(GenerationMismatch { format: None }) => {
            Err("Records without storage format marker")
        }
        Generation::Mismatch(_) => Err("Unknown or corrupt storage format marker"),
    }
}

pub(crate) fn reset_mismatched_in(
    tx: &mut impl RecordTransaction,
    user: LibertasUser,
    year: i32,
    expected: GenerationMismatch,
) -> RecordResult<Option<usize>> {
    match generation_in(tx, user, year)? {
        Generation::Mismatch(actual) if actual == expected => {
            Ok(Some(tx.delete_records(user, year)))
        }
        _ => Ok(None),
    }
}

pub(crate) fn read_for_open_in(
    tx: &mut impl RecordTransaction,
    user: LibertasUser,
    year: i32,
) -> Result<Option<LoadedReturn>, GenerationMismatch> {
    match generation_in(tx, user, year).expect("corrupt stored return") {
        Generation::Fresh => Ok(None),
        Generation::Current => Ok(read_current_in(tx, user, year).expect("corrupt stored return")),
        Generation::Mismatch(mismatch) => Err(mismatch),
    }
}

#[cfg(test)]
pub(crate) fn read_in(
    tx: &mut impl RecordTransaction,
    user: LibertasUser,
    year: i32,
) -> RecordResult<Option<LoadedReturn>> {
    if !current_generation(tx, user, year)? {
        return Ok(None);
    }
    read_current_in(tx, user, year)
}

fn read_current_in(
    tx: &mut impl RecordTransaction,
    user: LibertasUser,
    year: i32,
) -> RecordResult<Option<LoadedReturn>> {
    let metadata = match tx.get(&return_records::header_key(user, year)) {
        Some(TaxAppData::WorkingHeader { header }) => {
            return_records::known_header(&header, year)?;
            Some(header)
        }
        None => return Err("Records without working header"),
        _ => return Err("Invalid working header payload"),
    };
    let all = tx.records(user, year);
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

#[cfg(test)]
pub(crate) fn persist_in(
    tx: &mut impl RecordTransaction,
    user: LibertasUser,
    base: Option<&WorkingHeader>,
    draft: &FederalDraft,
    changed: bool,
) -> RecordResult<bool> {
    persist_with_in(tx, user, base, draft.tax_year, || (draft, changed, ()))
        .map(|saved| saved.is_some())
}

pub(crate) fn persist_with_in<T, D: Borrow<FederalDraft>>(
    tx: &mut impl RecordTransaction,
    user: LibertasUser,
    base: Option<&WorkingHeader>,
    year: i32,
    after_accept: impl FnOnce() -> (D, bool, T),
) -> RecordResult<Option<T>> {
    let initialized = current_generation(tx, user, year)?;
    let key = return_records::header_key(user, year);
    let current = if !initialized {
        None
    } else {
        match tx.get(&key) {
            Some(TaxAppData::WorkingHeader { header }) => {
                return_records::known_header(&header, year)?;
                Some(header)
            }
            None => return Err("Records without working header"),
            _ => return Err("Invalid working header"),
        }
    };
    if current.as_ref().map(|h| (&h.cookie, h.revision)) != base.map(|h| (&h.cookie, h.revision)) {
        return Ok(None);
    }
    let old = if initialized { tx.records(user, year) } else { Vec::new() };
    validate_namespace(user, year, &old)?;
    if let Some(current) = &current {
        let working = old
            .iter()
            .filter(|(key, _)| matches!(key, TaxDataKey::ReturnRecord { copy: 0, .. }))
            .cloned()
            .collect();
        let (_, checked_header) = return_records::assemble(user, year, working)?;
        if &checked_header != current {
            return Err("Header changed within transaction");
        }
    }
    let (draft, changed, result) = after_accept();
    let draft = draft.borrow();
    if draft.tax_year != year {
        return Err("Return year changed during save");
    }
    if !changed {
        return Ok(Some(result));
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
    Ok(Some(result))
}

pub(crate) trait ReturnStore {
    fn try_load(
        &mut self,
        user: LibertasUser,
        year: i32,
    ) -> Result<LoadedReturn, GenerationMismatch>;
    fn reset_mismatched(
        &mut self,
        user: LibertasUser,
        year: i32,
        expected: GenerationMismatch,
    ) -> Option<usize>;

    fn persist(
        &mut self,
        user: LibertasUser,
        base: Option<&WorkingHeader>,
        draft: &FederalDraft,
        changed: bool,
    ) -> bool {
        self.persist_with(user, base, draft.tax_year, || (draft, changed, ())).is_some()
    }
    fn persist_with<T, D: Borrow<FederalDraft>>(
        &mut self,
        user: LibertasUser,
        base: Option<&WorkingHeader>,
        year: i32,
        after_accept: impl FnOnce() -> (D, bool, T),
    ) -> Option<T>;
}

pub(crate) struct DataStore {
    data: Data<TaxDataKey, TaxAppData>,
}
impl ReturnStore for DataStore {
    fn try_load(
        &mut self,
        user: LibertasUser,
        year: i32,
    ) -> Result<LoadedReturn, GenerationMismatch> {
        assert!(user > 0, "verified positive user required");
        let prefix = TaxDataKey::return_record_prefix(user, year).expect("return prefix");
        let target = Target::prefix(&prefix).expect("return target");
        let mut tx = self.data.begin(&target, Mode::ReadOnly);
        let loaded = read_for_open_in(&mut tx, user, year);
        tx.commit();
        Ok(loaded?.unwrap_or_else(|| {
            let mut draft = FederalDraft::empty(alloc::format!(
                "{:016x}{:016x}",
                libertas::libertas_get_random(8),
                libertas::libertas_get_random(8)
            ));
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
        let prefix = TaxDataKey::return_record_prefix(user, year).expect("return prefix");
        let target = Target::prefix(&prefix).expect("return target");
        let mut tx = self.data.begin(&target, Mode::ReadWrite);
        let removed =
            reset_mismatched_in(&mut tx, user, year, expected).expect("corrupt stored return");
        tx.commit();
        if let Some(count) = removed {
            libertas::libertas_log(
                libertas::LogLevel::Info,
                &alloc::format!(
                    "Started new return: user={user} tax_year={year} removed_records={count}"
                ),
            );
        }
        removed
    }
    fn persist_with<T, D: Borrow<FederalDraft>>(
        &mut self,
        user: LibertasUser,
        base: Option<&WorkingHeader>,
        year: i32,
        after_accept: impl FnOnce() -> (D, bool, T),
    ) -> Option<T> {
        let prefix = TaxDataKey::return_record_prefix(user, year).expect("return prefix");
        let target = Target::prefix(&prefix).expect("return target");
        let mut tx = self.data.begin(&target, Mode::ReadWrite);
        let saved = persist_with_in(&mut tx, user, base, year, after_accept)
            .expect("corrupt stored return");
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

    fn load(&mut self, user: LibertasUser) -> Result<CachedReturn, GenerationMismatch> {
        let loaded = self.store.try_load(user, active_year())?;
        Ok(CachedReturn {
            encoded_bytes: loaded.encoded_bytes,
            base: loaded.header,
            interview: Interview::new(user, loaded.draft),
        })
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

    fn reload(store: &mut S, user: LibertasUser, state: &mut CachedReturn) {
        let current = store.try_load(user, active_year()).expect("incompatible stored return");
        *state = CachedReturn {
            interview: Interview::new(user, current.draft),
            encoded_bytes: current.encoded_bytes,
            base: current.header,
        };
    }

    // The protocol boundary: the person's page fields in, the canonical
    // interview request through, the person's page out (followup_page).
    fn respond(
        store: &mut S,
        user: LibertasUser,
        state: &mut CachedReturn,
        request: TaxInterviewProtocol,
        wire: bool,
    ) -> Option<Reply> {
        if !wire {
            return Self::respond_canonical(store, user, state, request);
        }
        let (request, targets) = match state.interview.wire_request(request) {
            Ok(request) => request,
            Err(reply) => return Some(*reply),
        };
        let reply = Self::respond_canonical(store, user, state, request)?;
        Some(state.interview.wire_reply(reply, targets.as_ref()))
    }

    fn respond_canonical(
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
                Self::reload(store, user, state);
            }
        }
        if matches!(request, TaxInterviewProtocol::SaveDocuments { .. })
            && let Err(reply) = state.interview.admit_request(&request)
        {
            return reply.map(|reply| *reply);
        }
        if let TaxInterviewProtocol::SaveDocuments { documents, document_owners, review, .. } =
            request
        {
            let mut candidate: Option<Box<Interview>> = None;
            let prepared = store.persist_with(
                user,
                state.base.as_ref(),
                state.interview.draft.tax_year,
                || {
                    // The persisted identity admits construction; neither a stale
                    // Save nor a failed transaction publishes a candidate to cache.
                    let interview = candidate.insert(Box::new(
                        state.interview.candidate_editor(state.interview.draft.clone()),
                    ));
                    let prepared = interview.prepare_document_center(documents, &document_owners);
                    (&interview.draft, interview.dirty, prepared)
                },
            );
            let Some(prepared) = prepared else {
                Self::reload(store, user, state);
                return Some(crate::interview::stale_editor_reply());
            };
            let candidate = *candidate.expect("accepted document save constructs a candidate");
            if candidate.accepted_save {
                state.interview = candidate;
                if state.interview.dirty {
                    Self::committed(user, state);
                } else {
                    state.interview.accepted_save = false;
                }
            }
            return Some(state.interview.finish_document_center_save(prepared, review));
        }
        let reply = state.interview.handle(request)?;
        if state.interview.dirty || state.interview.accepted_save {
            let changed = state.interview.dirty;
            if !store.persist(user, state.base.as_ref(), &state.interview.draft, changed) {
                // The candidate reply is discarded. Reload current accepted data,
                // return the existing stale-editor diagnostic, and never replay.
                Self::reload(store, user, state);
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

    // The interview's canonical form, for tests that drive the cache directly.
    #[cfg(test)]
    pub(crate) fn handle(
        &mut self,
        user: LibertasUser,
        request: TaxInterviewProtocol,
    ) -> Option<Reply> {
        self.handle_with(user, request, false)
    }

    // A request as the endpoint receives it, answered as the endpoint sends it.
    pub(crate) fn handle_wire(
        &mut self,
        user: LibertasUser,
        request: TaxInterviewProtocol,
    ) -> Option<Reply> {
        self.handle_with(user, request, true)
    }

    fn handle_with(
        &mut self,
        user: LibertasUser,
        request: TaxInterviewProtocol,
        wire: bool,
    ) -> Option<Reply> {
        if user <= 0 || request.user() != Some(user) {
            return None;
        }
        let request = if let TaxInterviewProtocol::StartNewReturn { tax_year, format, .. } = request
        {
            if tax_year != active_year()
                || self
                    .store
                    .reset_mismatched(user, active_year(), GenerationMismatch { format })
                    .is_none()
            {
                return Some(crate::interview::stale_editor_reply());
            }
            if let Some(old) = self.open.pop(&(user, active_year())) {
                self.encoded_bytes -= old.encoded_bytes;
            }
            TaxInterviewProtocol::OpenInterview { user }
        } else {
            request
        };
        if let Some(state) = self.open.get_mut(&(user, active_year())) {
            let before = state.encoded_bytes;
            let reply = Self::respond(&mut self.store, user, state, request, wire);
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
            let mut state = match self.load(user) {
                Ok(state) => state,
                Err(mismatch) => {
                    let message =
                        libertas::libertas_formatted_text("STORED_RETURN_OLDER_VERSION", &[]);
                    if request.is_calculation() {
                        return Some(Reply::Response(TaxInterviewProtocol::Problem {
                            message,
                            field: None,
                        }));
                    }
                    return Some(Reply::Response(TaxInterviewProtocol::StoredReturnUnavailable {
                        action: libertas::libertas_formatted_text("START_NEW_RETURN", &[]),
                        tax_year: active_year(),
                        format: mismatch.format,
                        message,
                    }));
                }
            };
            let reply = Self::respond(&mut self.store, user, &mut state, request, wire);
            self.retain(user, state);
            reply
        }
    }
}

#[cfg(test)]
impl<S: ReturnStore> Returns<S> {
    pub(crate) fn interview(&mut self, user: LibertasUser) -> &mut Interview {
        if !self.open.contains(&(user, active_year())) {
            let state = self.load(user).expect("test return uses current generation");
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
