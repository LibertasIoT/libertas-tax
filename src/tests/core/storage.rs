use super::*;
use crate::migration124::{Migration124, migrate_in};
use crate::return_records::{self, Records};
use crate::storage::{LoadedReturn, RecordTransaction, ReturnStore, Returns, persist_in, read_in};
use crate::tests::cross::interview::open_topic;
use alloc::collections::BTreeMap;
use core::num::NonZeroUsize;
use libertas::AvroDecode;
use libertas::LibertasUser;
use libertas::data::{KeyCodec, KeyDecode, wire};
use std::panic::{AssertUnwindSafe, catch_unwind};
extern crate std;

// Synthetic legacy fixtures use the frozen body. Independent baseline bytes
// below were produced by the actual pre-HOME native encoder (2026-10-18).
#[derive(libertas_macros::LibertasAvroEncode)]
#[allow(clippy::large_enum_variant)]
enum LegacyFixtureEnvelope {
    StorageFormat { format: u32 },
    Draft { draft: crate::migration124::Draft124 },
}

// Native bounded Data boundary, not a real Hub. All values and requests use the
// SDK codec. A private transaction snapshot becomes visible only on commit.
#[derive(Clone, Default)]
struct Store {
    values: BTreeMap<Vec<u8>, Vec<u8>>,
    reads: usize,
    writes: usize,
    pages: usize,
    mutations: usize,
    mutation_bytes: usize,
    fail_read: bool,
    fail_after: Option<usize>,
    fail_commit: bool,
    cookies: usize,
}
struct NativeTransaction {
    values: BTreeMap<Vec<u8>, Vec<u8>>,
    mutations: usize,
    mutation_bytes: usize,
    pages: usize,
    fail_read: bool,
    fail_after: Option<usize>,
}
impl NativeTransaction {
    fn new(store: &Store) -> Self {
        Self {
            values: store.values.clone(),
            mutations: 0,
            mutation_bytes: 0,
            pages: 0,
            fail_read: store.fail_read,
            fail_after: store.fail_after,
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
    fn exists(&mut self, key: &TaxDataKey) -> bool {
        self.values.contains_key(&key.to_key_bytes().unwrap())
    }
    fn get(&mut self, key: &TaxDataKey) -> Option<TaxAppData> {
        assert!(!self.fail_read, "injected fatal Data read failure");
        self.values
            .get(&key.to_key_bytes().unwrap())
            .map(|value| TaxAppData::from_avro(value).expect("fatal corrupt Data value"))
    }
    fn records(&mut self, user: LibertasUser, year: i32) -> Records {
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
    fn commit(&mut self, tx: NativeTransaction) {
        assert!(!self.fail_commit, "injected fatal Data commit failure");
        self.values = tx.values;
        self.mutations = tx.mutations;
        self.mutation_bytes = tx.mutation_bytes;
        self.pages += tx.pages;
        if tx.mutations > 0 {
            self.writes += 1;
        }
    }
    fn seed(&mut self, user: LibertasUser, draft: FederalDraft) {
        for (key, value) in return_records::project(user, &draft, 1).unwrap() {
            self.values.insert(key.to_key_bytes().unwrap(), value.to_avro());
        }
    }
    fn legacy(&mut self, user: LibertasUser, draft: FederalDraft) {
        self.values.insert(
            TaxDataKey::StorageFormat { user }.to_key_bytes().unwrap(),
            LegacyFixtureEnvelope::StorageFormat { format: 124 }.to_avro(),
        );
        self.values.insert(
            TaxDataKey::Draft { user }.to_key_bytes().unwrap(),
            LegacyFixtureEnvelope::Draft { draft: draft.try_into().unwrap() }.to_avro(),
        );
    }

    fn saved(&self, user: LibertasUser) -> FederalDraft {
        self.saved_year(user, 2026)
    }
    fn saved_year(&self, user: LibertasUser, year: i32) -> FederalDraft {
        read_in(&mut NativeTransaction::new(self), user, year).unwrap().unwrap().draft
    }
    fn bytes(&self, user: LibertasUser) -> usize {
        let prefix = TaxDataKey::return_record_prefix(user, 2026).unwrap();
        self.values
            .iter()
            .filter(|(key, _)| key.starts_with(&prefix))
            .map(|(_, value)| value.len())
            .sum()
    }
    fn migrate(&mut self, user: LibertasUser, year: i32) -> Result<Migration124, &'static str> {
        let mut tx = NativeTransaction::new(self);
        let result = migrate_in(&mut tx, user, year)?;
        self.commit(tx);
        Ok(result)
    }
}
impl ReturnStore for Store {
    fn load_or_initialize(&mut self, user: LibertasUser, year: i32) -> LoadedReturn {
        self.reads += 1;
        let mut tx = NativeTransaction::new(self);
        let loaded = read_in(&mut tx, user, year).expect("fatal keyed corruption");
        self.commit(tx);
        loaded.unwrap_or_else(|| {
            assert!(
                !self.values.contains_key(&TaxDataKey::Draft { user }.to_key_bytes().unwrap()),
                "quiesced migration required"
            );
            self.cookies += 1;
            let mut draft = FederalDraft::empty(alloc::format!("fresh-{}", self.cookies));
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
        let mut tx = NativeTransaction::new(self);
        let saved =
            persist_in(&mut tx, user, base, draft, changed).expect("fatal keyed corruption");
        self.commit(tx);
        saved
    }
}
fn cache(bytes: usize, store: Store) -> Returns<Store> {
    Returns::with_store(NonZeroUsize::new(bytes).unwrap(), store)
}
fn finish(user: LibertasUser, draft: &FederalDraft) -> P {
    P::Finish {
        user,
        cookie: draft.cookie.clone(),
        revision: draft.revision,
        page: TaxSection::Review as i32,
        previous: TaxSection::Review as i32,
        review: true,
        progress: vec![],
    }
}

#[test]
fn stable_legacy_roots_decode_exact_124_bytes_and_do_not_move() {
    assert_eq!(TaxAppData::StorageFormat { format: 2 }.to_avro(), [0, 4]);
    let draft = complete(FederalStatus::Joint);
    let bytes = TaxAppData::Draft { draft: draft.clone().try_into().unwrap() }.to_avro();
    assert_eq!(bytes[0], 2);
    assert_eq!(
        TaxAppData::from_avro(&bytes).unwrap(),
        TaxAppData::Draft { draft: draft.try_into().unwrap() }
    );
    for user in [0, 1, u32::MAX as i64 + 1, i64::MAX] {
        for (tag, key) in
            [(0u8, TaxDataKey::StorageFormat { user }), (1, TaxDataKey::Draft { user })]
        {
            let bytes = key.to_key_bytes().unwrap();
            assert_eq!(bytes, [&[0, tag][..], &(user as u64 ^ (1 << 63)).to_be_bytes()].concat());
            assert_eq!(TaxDataKey::decode_key(&bytes).unwrap(), key);
        }
    }
}

#[test]
fn full_projection_preserves_all_22_fields_and_rejects_missing_duplicate_and_wrong_namespace() {
    for draft in [
        FederalDraft::empty("absent".into()),
        complete(FederalStatus::Joint),
        benefit_return(FederalStatus::Joint),
    ] {
        let mut records = return_records::project(TEST_USER, &draft, 23).unwrap();
        records.reverse();
        let (assembled, header) =
            return_records::assemble(TEST_USER, draft.tax_year, records.clone()).unwrap();
        assert_eq!(assembled, draft);
        assert_eq!(header.next_filed_copy, 23);
        for index in 0..records.len() {
            let mut bad = records.clone();
            bad.remove(index);
            assert!(return_records::assemble(TEST_USER, draft.tax_year, bad).is_err());
        }
        let mut duplicate = records.clone();
        duplicate.push(records[0].clone());
        assert!(return_records::assemble(TEST_USER, draft.tax_year, duplicate).is_err());
        assert!(return_records::assemble(TEST_USER + 1, draft.tax_year, records).is_err());
    }
}

#[test]
fn warm_navigation_and_estimates_do_no_writes_and_identity_is_checked_before_loading() {
    let mut returns = cache(64 * 1024 * 1024, Store::default());
    assert!(returns.handle(0, P::OpenInterview { user: 0 }).is_none());
    assert!(returns.handle(1, P::OpenInterview { user: 2 }).is_none());
    assert_eq!(returns.store().reads, 0);
    assert!(
        returns
            .handle(
                1,
                P::EstimateSetup { user: 1, value: ReturnSetup { filing: Filing::MarriedJointly } }
            )
            .is_some()
    );
    let before = returns.interview(1).draft.clone();
    assert!(returns.store().values.is_empty(), "running Estimate creates no records");
    assert!(returns.handle(1, P::Enter { user: 1 }).is_some());
    assert_eq!(returns.store().writes, 0);
    assert!(returns.handle(1, P::OpenInterview { user: 1 }).is_some());
    assert_eq!(returns.store().writes, 1, "first editor persists its identity before reply");
    assert_eq!(returns.interview(1).draft, before);
    let writes = returns.store().writes;
    assert!(returns.handle(1, P::OpenInterview { user: 1 }).is_some());
    assert_eq!(returns.store().reads, 1);
    let mut preview = crate::tests::support::client::survey::survey_preview_document(
        IncomeDocument::W2(crate::tests::cross::interview::w2(6_200_000)),
    );
    if let P::PreviewDocument { user, foreign_countries, .. } = &mut preview {
        *user = 1;
        *foreign_countries = crate::foreign_accounts::countries();
    }
    assert!(returns.handle(1, preview).is_some());
    assert_eq!(returns.store().writes, writes);
}

#[test]
fn byte_lru_promotes_hits_and_reloads_only_the_evicted_user() {
    let mut store = Store::default();
    for user in 1..=3 {
        let mut draft = complete(FederalStatus::Single);
        draft.cookie = alloc::format!("saved-{user}");
        draft.revision = user;
        store.seed(user, draft);
    }
    let small_bytes = store.bytes(1);
    let mut large = store.saved(1);
    while return_records::value_bytes(&return_records::project(4, &large, 1).unwrap())
        <= 2 * small_bytes
    {
        let id = large.next_id;
        large.income.push(wage(id, Owner::Taxpayer, 7_000_000, 700_000));
        large.next_id += 1;
    }
    assert!(large.valid_stored());
    store.seed(4, large);
    let budget = small_bytes + store.bytes(4);
    let mut returns = cache(budget, store);
    returns.interview(1);
    let before = returns.interview(2).draft.clone();
    returns.interview(3);
    assert_eq!(returns.len(), 3, "three small returns fit");
    returns.interview(1);
    returns.interview(4);
    assert_eq!(returns.len(), 2, "one large return displaces two small returns");
    assert_eq!(returns.encoded_bytes(), budget);
    let reads = returns.store().reads;
    returns.interview(1);
    assert_eq!(returns.store().reads, reads);
    assert_eq!(returns.interview(2).draft, before);
    assert_eq!(returns.store().reads, reads + 1);
    assert_eq!(returns.len(), 2);
}

#[test]
fn exact_record_bytes_track_save_growth_shrink_and_oversized_request_lifetime() {
    let draft = complete(FederalStatus::Single);
    let mut store = Store::default();
    store.seed(TEST_USER, draft.clone());
    store.seed(2, draft.clone());
    let budget = store.bytes(TEST_USER);
    let mut returns = cache(budget, store);
    returns.interview(2);
    let documents = (0..3)
        .map(|_| {
            crate::tests::cross::interview::document(
                0,
                IncomeDocument::W2(crate::tests::cross::interview::w2(6_000_000)),
            )
        })
        .collect();
    let request = crate::tests::cross::interview::save_documents(&draft, documents);
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, request).unwrap());
    let grown = returns.store().saved(TEST_USER);
    assert!(returns.store().bytes(TEST_USER) > budget);
    assert_eq!(returns.len(), 1);
    assert_eq!(returns.encoded_bytes(), budget);
    let reads = returns.store().reads;
    returns.interview(2);
    assert_eq!(returns.store().reads, reads);
    assert!(returns.handle(TEST_USER, P::ReviewAccepted { user: TEST_USER }).is_some());
    assert_eq!(returns.len(), 1);
    let request = crate::tests::cross::interview::save_documents(&grown, vec![]);
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, request).unwrap());
    assert_eq!(returns.encoded_bytes(), returns.store().bytes(TEST_USER));
    let small = returns.store().saved(TEST_USER);
    assert_eq!(returns.interview(TEST_USER).draft, small);
    // The cached growth path removes itself after commit, preserving no hidden active return.
    let documents = (0..3)
        .map(|_| {
            crate::tests::cross::interview::document(
                0,
                IncomeDocument::W2(crate::tests::cross::interview::w2(6_000_000)),
            )
        })
        .collect();
    let request = crate::tests::cross::interview::save_documents(&small, documents);
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, request).unwrap());
    assert_eq!(returns.len(), 0);
    assert_eq!(returns.encoded_bytes(), 0);
}

#[test]
fn successful_save_commits_before_reply_and_unchanged_or_stale_edits_never_write() {
    let draft = complete(FederalStatus::Single);
    let mut store = Store::default();
    store.seed(TEST_USER, draft.clone());
    let mut returns = cache(64 * 1024 * 1024, store);
    assert!(matches!(
        returns.handle(TEST_USER, finish(TEST_USER, &draft)),
        Some(Reply::Response(P::Finished { .. }))
    ));
    let saved = returns.store().saved(TEST_USER);
    assert!(saved.finished);
    assert_eq!(saved.revision, draft.revision + 1);
    assert_eq!(returns.store().writes, 1);
    assert!(!returns.interview(TEST_USER).dirty);
    assert!(matches!(
        returns.handle(TEST_USER, finish(TEST_USER, &draft)),
        Some(Reply::Response(P::Problem { .. }))
    ));
    assert_eq!(returns.store().writes, 1);
    let request = crate::tests::cross::interview::save_documents(
        &saved,
        vec![crate::tests::cross::interview::document(
            saved.income[0].id,
            IncomeDocument::W2(crate::tests::cross::interview::w2(6_000_000)),
        )],
    );
    // The existing candidate owner decides whether these source facts changed.
    let _ = returns.handle(TEST_USER, request);
    let saved = returns.store().saved(TEST_USER);
    let mut cold = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(cold.interview(TEST_USER).draft, saved);
}

#[test]
fn persisted_revision_or_cookie_conflict_discards_candidate_and_reloads_without_replaying() {
    for different_cookie in [false, true] {
        let draft = complete(FederalStatus::Single);
        let mut store = Store::default();
        store.seed(TEST_USER, draft.clone());
        let mut returns = cache(64 * 1024 * 1024, store);
        returns.interview(TEST_USER);
        let mut competing = draft.clone();
        competing.revision += 1;
        if different_cookie {
            competing.cookie = "replacement".into();
        }
        returns.store_mut().seed(TEST_USER, competing.clone());
        let before = returns.store().values.clone();
        assert!(matches!(
            returns.handle(TEST_USER, finish(TEST_USER, &draft)),
            Some(Reply::Response(P::Problem { .. }))
        ));
        assert_eq!(returns.store().values, before);
        assert_eq!(returns.interview(TEST_USER).draft, competing);
        assert!(!returns.interview(TEST_USER).dirty);
        assert_eq!(returns.store().writes, 0);
    }
}

#[test]
fn failed_write_and_failed_commit_preserve_old_state_and_restart_without_success() {
    for at_commit in [false, true] {
        let draft = complete(FederalStatus::Single);
        let mut store = Store::default();
        store.seed(TEST_USER, draft.clone());
        let before = store.values.clone();
        let mut returns = cache(64 * 1024 * 1024, store);
        returns.interview(TEST_USER);
        if at_commit {
            returns.store_mut().fail_commit = true;
        } else {
            returns.store_mut().fail_after = Some(1);
        }
        assert!(
            catch_unwind(AssertUnwindSafe(|| returns.handle(TEST_USER, finish(TEST_USER, &draft))))
                .is_err()
        );
        assert_eq!(returns.store().values, before);
        assert!(returns.interview(TEST_USER).dirty);
        let mut store = returns.store().clone();
        store.fail_after = None;
        store.fail_commit = false;
        let mut restarted = cache(64 * 1024 * 1024, store);
        assert_eq!(restarted.interview(TEST_USER).draft, draft);
        assert!(matches!(
            restarted.handle(TEST_USER, finish(TEST_USER, &draft)),
            Some(Reply::Response(P::Finished { .. }))
        ));
    }
}

#[test]
fn corrupt_keyed_returns_and_failed_reads_never_reset_or_cache() {
    for case in 0..5 {
        let mut store = Store::default();
        let draft = complete(FederalStatus::Single);
        store.seed(TEST_USER, draft);
        let key = return_records::header_key(TEST_USER, 2026).to_key_bytes().unwrap();
        match case {
            0 => {
                store.values.remove(&key);
            }
            1 => {
                store.values.insert(key, vec![255]);
            }
            2 => {
                let missing = TaxDataKey::ReturnRecord {
                    user: TEST_USER,
                    tax_year: 2026,
                    copy: 0,
                    kind: 15,
                    id: 0,
                };
                store.values.remove(&missing.to_key_bytes().unwrap());
            }
            3 => {
                let mut header = match TaxAppData::from_avro(&store.values[&key]).unwrap() {
                    TaxAppData::WorkingHeader { header } => header,
                    _ => unreachable!(),
                };
                header.layout += 1;
                store.values.insert(key, TaxAppData::WorkingHeader { header }.to_avro());
            }
            _ => store.fail_read = true,
        }
        let before = store.values.clone();
        let mut returns = cache(64 * 1024 * 1024, store);
        assert!(
            catch_unwind(AssertUnwindSafe(
                || returns.handle(TEST_USER, P::OpenInterview { user: TEST_USER })
            ))
            .is_err()
        );
        assert_eq!(returns.len(), 0);
        assert_eq!(returns.store().values, before);
        assert_eq!(returns.store().writes, 0);
    }
}

#[test]
fn migration_124_preserves_two_users_high_ids_nested_links_and_distinct_year_namespace() {
    let mut first = complete(FederalStatus::Joint);
    first.income[0].id = u32::MAX as i64 + 21;
    first.next_id = i64::MAX - 1;
    first.revision = 83;
    let second = benefit_return(FederalStatus::Joint);
    let mut other_year = first.clone();
    other_year.tax_year = 2027;
    // Storage namespace isolation only: this is not a claim of 2027 tax-rule support.
    let mut store = Store::default();
    store.legacy(TEST_USER, first.clone());
    store.legacy(2, second.clone());
    store.seed(TEST_USER, other_year.clone());
    let untouched = store.values.clone();
    assert_eq!(store.migrate(TEST_USER, 2026).unwrap(), Migration124::Migrated);
    assert_eq!(store.saved(TEST_USER), first);
    assert_eq!(store.saved_year(TEST_USER, 2027), other_year);
    assert_eq!(
        store.values[&TaxDataKey::Draft { user: 2 }.to_key_bytes().unwrap()],
        untouched[&TaxDataKey::Draft { user: 2 }.to_key_bytes().unwrap()]
    );
    let once = store.values.clone();
    assert_eq!(store.migrate(TEST_USER, 2026).unwrap(), Migration124::AlreadyMigrated);
    assert_eq!(store.values, once);
    assert_eq!(store.migrate(2, 2026).unwrap(), Migration124::Migrated);
    assert_eq!(store.saved(2), second);
    let mut returns = cache(64 * 1024 * 1024, store);
    let request = crate::tests::cross::interview::save_documents(&first, vec![]);
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, request).unwrap());
    let pruned = returns.store().saved(TEST_USER);
    assert!(pruned.income.is_empty());
    assert_eq!(pruned.next_id, first.next_id);
    assert_eq!(returns.store().saved_year(TEST_USER, 2027), other_year);
    assert_eq!(returns.store().saved(2), second);
}

#[test]
fn migration_unknown_format_is_checked_before_body_and_failure_leaves_old_bytes_intact() {
    let first = complete(FederalStatus::Single);
    let mut store = Store::default();
    store.legacy(TEST_USER, first.clone());
    store.values.insert(
        TaxDataKey::StorageFormat { user: TEST_USER }.to_key_bytes().unwrap(),
        TaxAppData::StorageFormat { format: 123 }.to_avro(),
    );
    store.values.insert(TaxDataKey::Draft { user: TEST_USER }.to_key_bytes().unwrap(), vec![255]);
    assert_eq!(store.migrate(TEST_USER, 2026), Err("Unsupported legacy storage format"));
    for at_commit in [false, true] {
        let mut store = Store::default();
        store.legacy(TEST_USER, first.clone());
        let old = store.values.clone();
        if at_commit {
            store.fail_commit = true;
        } else {
            store.fail_after = Some(3);
        }
        assert!(catch_unwind(AssertUnwindSafe(|| store.migrate(TEST_USER, 2026))).is_err());
        assert_eq!(store.values, old);
        store.fail_after = None;
        store.fail_commit = false;
        assert_eq!(store.migrate(TEST_USER, 2026).unwrap(), Migration124::Migrated);
        let mut restarted = cache(64 * 1024 * 1024, store);
        assert_eq!(restarted.interview(TEST_USER).draft, first);
        assert!(matches!(
            restarted.handle(TEST_USER, finish(TEST_USER, &first)),
            Some(Reply::Response(P::Finished { .. }))
        ));
    }
}

#[test]
fn wage_specials_saved_sources_survive_store_cache_cold_read_and_link_deletion() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(cash_business(0)));
    let business_id = d.income[1].id;
    let mut store = Store::default();
    store.seed(TEST_USER, d.clone());
    let mut returns = cache(64 * 1024 * 1024, store);
    let mut statutory = crate::tests::cross::interview::w2(5_000_000);
    statutory.more.statutory = true;
    statutory.answers.statutory_business = Some(business_id);
    statutory.answers.medicaid_waiver = 100_000;
    statutory.more.box12 = vec![
        Box12Entry { code: W2Code::Q, amount: 200_000 },
        Box12Entry { code: W2Code::II, amount: 300_000 },
    ];
    let mut railroad = crate::tests::cross::interview::document(
        0,
        IncomeDocument::W2(crate::tests::cross::interview::w2(26_000_000)),
    );
    railroad.owner = Owner::Spouse;
    let IncomeDocument::W2(w) = &mut railroad.document else { panic!() };
    w.social_wages = 0;
    w.social_tax = 0;
    w.medicare_wages = 0;
    w.medicare_tax = 0;
    w.more.payroll = W2Payroll::Railroad {
        compensation: 26_000_000,
        tier_1: 1_143_900,
        tier_2: 670_000,
        medicare: 377_000,
        additional_medicare: 54_000,
    };
    let request = crate::tests::cross::interview::save_documents(
        &d,
        vec![crate::tests::cross::interview::document(1, IncomeDocument::W2(statutory)), railroad],
    );
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, request).unwrap());
    let accepted = returns.interview(TEST_USER).draft.clone();
    assert!(accepted.valid_stored());
    assert_eq!(returns.store().saved(TEST_USER), accepted);
    let mut cold = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(cold.interview(TEST_USER).draft, accepted);
    let request =
        crate::tests::cross::interview::save_topic(&accepted, TaxSection::SelfEmployment, vec![]);
    crate::tests::cross::interview::accepted(&cold.handle(TEST_USER, request).unwrap());
    let deleted = cold.interview(TEST_USER).draft.clone();
    let FederalIncome::Wage { data } = &deleted.income[0].income else { panic!() };
    assert_eq!(data.statutory_business, None);
    assert_eq!(data.wages, 5_000_000);
    assert_eq!(data.medicaid_waiver, 100_000);
    let mut reopened = cache(64 * 1024 * 1024, cold.store().clone());
    assert_eq!(reopened.interview(TEST_USER).draft, deleted);
    assert_eq!(reopened.store().saved(TEST_USER), deleted);
}

// F18 lifecycle fixtures use the actual storage owner on a fresh cache.
pub(in crate::tests) fn f18_cold(draft: &FederalDraft) {
    assert!(draft.valid_stored());
    assert_eq!(crate::tests::support::client::roundtrip_native_draft(draft.clone()), *draft);
    let mut store = Store::default();
    store.seed(TEST_USER, draft.clone());
    let mut returns = cache(64 * 1024 * 1024, store);
    assert_eq!(returns.interview(TEST_USER).draft, *draft);
}

#[test]
fn care_completion_normal_store_save_cold_read_and_reopen_preserve_supplied_facts() {
    let draft = benefit_return(FederalStatus::Joint);
    let mut store = Store::default();
    store.seed(TEST_USER, draft);
    let mut returns = cache(64 * 1024 * 1024, store);
    let draft = returns.interview(TEST_USER).draft.clone();
    let request = P::Navigate {
        user: TEST_USER,
        cookie: draft.cookie,
        revision: draft.revision,
        from: TaxSection::Review as i32,
        section: TaxSection::Care.into(),
        navigation: NavigationAvailability::default(),
    };
    let Reply::Edit(mut page @ P::SaveCare { .. }) = returns.handle(TEST_USER, request).unwrap()
    else {
        panic!()
    };
    if let P::SaveCare { value, .. } = &mut page {
        value.providers[0].name = "Cold-read provider".into();
        value.plan_maximum = CarePlanMaximum::Plans { maximum: 500_000 };
    }
    let reply = returns.handle(TEST_USER, page).unwrap();
    assert!(!matches!(reply, Reply::Response(P::Problem { .. })), "{reply:?}");
    let accepted = returns.interview(TEST_USER).draft.clone();
    assert_eq!(returns.store().saved(TEST_USER), accepted);
    assert!(accepted.valid_stored());
    let durable = returns.store().clone();
    drop(returns);
    let mut cold = cache(64 * 1024 * 1024, durable);
    assert_eq!(cold.interview(TEST_USER).draft, accepted);
    let saved = cold.store().saved(TEST_USER);
    assert_eq!(saved.credits.as_ref().unwrap().care.providers[0].name, "Cold-read provider");
    assert_eq!(
        saved.credits.as_ref().unwrap().care.plan_maximum,
        CarePlanMaximum::Plans { maximum: 500_000 }
    );
    assert_eq!(estimate(&saved), estimate(&accepted));
    let request = P::Navigate {
        user: TEST_USER,
        cookie: saved.cookie,
        revision: saved.revision,
        from: TaxSection::Review as i32,
        section: TaxSection::Care.into(),
        navigation: NavigationAvailability::default(),
    };
    let Reply::Edit(P::SaveCare { value, .. }) = cold.handle(TEST_USER, request).unwrap() else {
        panic!()
    };
    assert_eq!(value.providers[0].id, accepted.credits.as_ref().unwrap().care.providers[0].id);
    assert_eq!(value.providers[0].name, "Cold-read provider");
    assert_eq!(value.plan_maximum, CarePlanMaximum::Plans { maximum: 500_000 });
}

// Shared with the existing native-capacity test: no copied lot fixture.
pub(in crate::tests) fn rehearse_heavy(draft: &FederalDraft) {
    assert!(draft.valid_stored());
    let legacy = TaxAppData::Draft { draft: draft.clone().try_into().unwrap() }.to_avro();
    assert!(legacy.len() <= wire::MAX_VALUE_BYTES, "legacy fixture must itself be persistable");
    let records = return_records::project(TEST_USER, draft, 1).unwrap();
    let expected_bytes: usize = records
        .iter()
        .map(|(key, value)| key.to_key_bytes().unwrap().len() + value.to_avro().len())
        .sum::<usize>()
        + TaxDataKey::Draft { user: TEST_USER }.to_key_bytes().unwrap().len();
    let mut store = Store::default();
    store.legacy(TEST_USER, draft.clone());
    let old = store.values.clone();
    store.fail_after = Some(records.len());
    assert!(catch_unwind(AssertUnwindSafe(|| store.migrate(TEST_USER, draft.tax_year))).is_err());
    assert_eq!(store.values, old);
    store.fail_after = None;
    assert_eq!(store.migrate(TEST_USER, draft.tax_year).unwrap(), Migration124::Migrated);
    assert_eq!(store.mutation_bytes, expected_bytes);
    assert_eq!(store.mutations, records.len() + 1);
    assert_eq!(store.saved(TEST_USER), *draft);
    let mutations = store.mutations;
    let bytes = store.mutation_bytes;
    store.pages = 0;
    let mut returns = cache(64 * 1024 * 1024, store);
    assert_eq!(returns.interview(TEST_USER).draft, *draft);
    if records.len() > 4096 {
        assert!(returns.store().pages >= 2);
    }
    assert_eq!(returns.encoded_bytes(), return_records::value_bytes(&records));
    let mut editor = Interview::new(TEST_USER, draft.clone());
    let mut page = open_topic(&mut editor, TaxSection::InvestmentSales);
    let P::SaveIncomes { documents, .. } = &mut page else { unreachable!() };
    for item in &mut documents.documents {
        let IncomeDocument::Sale1099B(sale) = &mut item.document else { unreachable!() };
        sale.description.push_str(" corrected");
    }
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, page).unwrap());
    let corrected = returns.store().saved(TEST_USER);
    assert_eq!(corrected.income.len(), draft.income.len());
    assert_eq!(corrected.next_id, draft.next_id);
    assert_eq!(corrected.revision, draft.revision + 1);
    if draft.income.len() > 4096 {
        assert!(returns.store().mutations > 4096);
    }
    let mut editor = Interview::new(TEST_USER, corrected.clone());
    let mut prune = open_topic(&mut editor, TaxSection::InvestmentSales);
    let P::SaveIncomes { documents, .. } = &mut prune else { unreachable!() };
    documents.documents.clear();
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, prune).unwrap());
    let pruned = returns.store().saved(TEST_USER);
    assert_eq!(pruned.income.len(), 1, "the W-2 stays");
    assert_eq!(pruned.next_id, draft.next_id);
    let mut restarted = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(restarted.interview(TEST_USER).draft, pruned);
    std::println!(
        "Unit53 full native bounded migration rows={} mutations={mutations} bytes={bytes}; correction and prune committed; real Hub not exercised",
        draft.income.len()
    );
}

#[test]
fn legacy_missing_marker_or_body_is_not_a_successful_empty_migration() {
    let mut empty = Store::default();
    assert_eq!(empty.migrate(TEST_USER, 2026).unwrap(), Migration124::NoLegacyReturn);
    for missing_marker in [false, true] {
        let mut store = Store::default();
        store.legacy(TEST_USER, complete(FederalStatus::Single));
        let missing = if missing_marker {
            TaxDataKey::StorageFormat { user: TEST_USER }
        } else {
            TaxDataKey::Draft { user: TEST_USER }
        };
        store.values.remove(&missing.to_key_bytes().unwrap());
        if missing_marker {
            store
                .values
                .insert(TaxDataKey::Draft { user: TEST_USER }.to_key_bytes().unwrap(), vec![255]);
        }
        let before = store.values.clone();
        assert!(store.migrate(TEST_USER, 2026).is_err());
        assert_eq!(store.values, before);
        assert_eq!(store.writes, 0);
    }
}

#[test]
fn warm_save_cannot_repair_corrupt_persisted_records_with_matching_revision() {
    let draft = complete(FederalStatus::Single);
    for unknown_kind in [false, true] {
        let mut store = Store::default();
        store.seed(TEST_USER, draft.clone());
        let mut returns = cache(64 * 1024 * 1024, store);
        returns.interview(TEST_USER);
        let key = TaxDataKey::ReturnRecord {
            user: TEST_USER,
            tax_year: 2026,
            copy: 0,
            kind: if unknown_kind { 99 } else { 15 },
            id: 0,
        }
        .to_key_bytes()
        .unwrap();
        if unknown_kind {
            returns
                .store_mut()
                .values
                .insert(key, TaxAppData::StorageFormat { format: 124 }.to_avro());
        } else {
            returns.store_mut().values.remove(&key);
        }
        let before = returns.store().values.clone();
        assert!(
            catch_unwind(AssertUnwindSafe(|| returns.handle(TEST_USER, finish(TEST_USER, &draft))))
                .is_err()
        );
        assert_eq!(returns.store().values, before);
        assert_eq!(returns.store().writes, 0);
    }
}

#[test]
fn invalid_stored_financial_answers_are_blocked_without_resetting_them() {
    let mut draft = complete(FederalStatus::Single);
    let FederalIncome::Wage { data } = &mut draft.income[0].income else { unreachable!() };
    data.wages = -1;
    assert!(!draft.valid_stored());
    let mut store = Store::default();
    store.seed(TEST_USER, draft.clone());
    let before = store.values.clone();
    let mut returns = cache(64 * 1024 * 1024, store);
    assert!(matches!(
        returns.handle(TEST_USER, P::OpenInterview { user: TEST_USER }),
        Some(Reply::Response(P::Problem { .. }))
    ));
    assert_eq!(returns.interview(TEST_USER).draft, draft);
    assert_eq!(returns.store().values, before);
}

#[test]
fn first_accepted_save_creates_complete_records_and_unchanged_setup_keeps_revision() {
    let mut returns = cache(64 * 1024 * 1024, Store::default());
    let before = returns.interview(TEST_USER).draft.clone();
    assert!(returns.store().values.is_empty());
    let setup = ReturnSetup { filing: Filing::MarriedJointly };
    let request = crate::tests::cross::interview::save_setup(&before, setup.clone());
    assert!(!matches!(
        returns.handle(TEST_USER, request),
        Some(Reply::Response(P::Problem { .. }))
    ));
    let saved = returns.store().saved(TEST_USER);
    assert!(saved.setup.is_some());
    assert_eq!(saved.revision, 1);
    assert_eq!(returns.interview(TEST_USER).draft, saved);
    assert_eq!(returns.store().writes, 1);
    let request = crate::tests::cross::interview::save_setup(&saved, setup.clone());
    assert!(!matches!(
        returns.handle(TEST_USER, request),
        Some(Reply::Response(P::Problem { .. }))
    ));
    assert_eq!(returns.store().saved(TEST_USER), saved);
    assert_eq!(returns.store().writes, 1, "unchanged accepted page must not persist");
    let mut cold = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(cold.interview(TEST_USER).draft, saved);
    let mut competing = saved.clone();
    competing.revision += 1;
    returns.store_mut().seed(TEST_USER, competing.clone());
    let before = returns.store().values.clone();
    let request = crate::tests::cross::interview::save_setup(&saved, setup);
    assert!(matches!(returns.handle(TEST_USER, request), Some(Reply::Response(P::Problem { .. }))));
    assert_eq!(returns.store().values, before);
    assert_eq!(returns.interview(TEST_USER).draft, competing);
    assert_eq!(returns.store().writes, 1, "stale unchanged Save must not write");
}

#[test]
fn unrelated_users_do_not_increase_return_read_pages_or_reset_their_records() {
    let mut store = Store::default();
    for user in 2..=200 {
        store.values.insert(TaxDataKey::Draft { user }.to_key_bytes().unwrap(), vec![255]);
    }
    let untouched = store.values.clone();
    let mut returns = cache(64 * 1024 * 1024, store);
    returns.interview(1);
    assert_eq!(returns.store().reads, 1);
    assert_eq!(returns.store().pages, 1);
    assert_eq!(returns.store().values, untouched);
}

#[test]
fn admitted_oversized_payment_waiver_hits_data_limit_without_partial_save_or_truncation() {
    let draft = complete(FederalStatus::Single);
    let mut store = Store::default();
    store.seed(TEST_USER, draft.clone());
    let mut returns = cache(64 * 1024 * 1024, store);
    let request = P::Navigate {
        user: TEST_USER,
        cookie: draft.cookie.clone(),
        revision: draft.revision,
        from: TaxSection::Review as i32,
        section: TaxSection::Payments.into(),
        navigation: NavigationAvailability::default(),
    };
    let Some(Reply::Edit(mut page @ P::SavePayments { .. })) = returns.handle(TEST_USER, request)
    else {
        panic!("expected payments editor")
    };
    let P::SavePayments { value, .. } = &mut page else { unreachable!() };
    value.penalty = PenaltyChoice::Figure {
        prior_tax: 1_000_000,
        prior_agi: Some(6_000_000),
        full_year: true,
        prior_return: PriorReturn::SameReturn,
        farmer: Farmer::No,
        withholding: WithholdingTiming::Evenly,
        income: IncomeTiming::Evenly,
        waiver: Waiver::Full {
            reason: PenaltyWaiverReason::CasualtyOrDisasterOrUnusual,
            explanation: "X".repeat(wire::MAX_VALUE_BYTES + 1),
        },
    };
    assert!(page.to_avro().len() < wire::MAX_REQUEST_BYTES);
    let before = returns.store().values.clone();
    assert!(catch_unwind(AssertUnwindSafe(|| returns.handle(TEST_USER, page))).is_err());
    assert_eq!(returns.store().values, before);
    assert_eq!(returns.store().writes, 0);
    let candidate = &returns.interview(TEST_USER).draft;
    assert!(candidate.valid_stored(), "native accepted answers remain valid before Data limit");
    let Some(PenaltyWaiver::Full { explanation, .. }) =
        &candidate.payments.as_ref().unwrap().waiver
    else {
        unreachable!()
    };
    assert_eq!(explanation.len(), wire::MAX_VALUE_BYTES + 1);
    let mut restarted = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(restarted.interview(TEST_USER).draft, draft);
}

#[test]
fn uncached_fresh_editor_cookie_survives_open_save_and_failed_initialization() {
    let mut returns = cache(1, Store::default());
    assert!(
        returns
            .handle(
                TEST_USER,
                P::EstimateSetup {
                    user: TEST_USER,
                    value: ReturnSetup { filing: Filing::MarriedJointly }
                }
            )
            .is_some()
    );
    assert!(returns.store().values.is_empty());
    assert_eq!(returns.len(), 0);
    assert!(returns.handle(TEST_USER, P::OpenInterview { user: TEST_USER }).is_some());
    assert_eq!(returns.len(), 0);
    let opened = returns.store().saved(TEST_USER);
    assert!(returns.handle(TEST_USER, P::OpenInterview { user: TEST_USER }).is_some());
    assert_eq!(returns.store().saved(TEST_USER).cookie, opened.cookie);
    assert_eq!(returns.store().writes, 1, "reopening keeps the committed initial identity");
    let request = crate::tests::cross::interview::save_setup(
        &opened,
        ReturnSetup { filing: Filing::MarriedJointly },
    );
    assert!(!matches!(
        returns.handle(TEST_USER, request),
        Some(Reply::Response(P::Problem { .. }))
    ));
    let saved = returns.store().saved(TEST_USER);
    assert_eq!(saved.cookie, opened.cookie);
    assert_eq!(saved.revision, 1);
    assert!(saved.setup.is_some());
    assert_eq!(returns.len(), 0);
    let store = Store { fail_after: Some(3), ..Store::default() };
    let mut failed = cache(1, store);
    assert!(
        catch_unwind(AssertUnwindSafe(
            || failed.handle(TEST_USER, P::OpenInterview { user: TEST_USER })
        ))
        .is_err()
    );
    assert!(failed.store().values.is_empty());
    assert_eq!(failed.len(), 0);
    let mut store = failed.store().clone();
    store.fail_after = None;
    let mut restarted = cache(1, store);
    assert!(restarted.handle(TEST_USER, P::OpenInterview { user: TEST_USER }).is_some());
    assert!(restarted.store().saved(TEST_USER).valid_stored());
}

#[test]
fn concurrent_initial_open_uses_the_committed_return_without_overwriting_it() {
    let mut returns = cache(64 * 1024 * 1024, Store::default());
    let provisional = returns.interview(TEST_USER).draft.clone();
    let mut competing = complete(FederalStatus::Single);
    competing.cookie = "committed-by-another-opener".into();
    assert_ne!(provisional.cookie, competing.cookie);
    returns.store_mut().seed(TEST_USER, competing.clone());
    let before = returns.store().values.clone();
    assert!(returns.handle(TEST_USER, P::OpenInterview { user: TEST_USER }).is_some());
    assert_eq!(returns.interview(TEST_USER).draft, competing);
    assert_eq!(returns.store().values, before);
    assert_eq!(returns.store().writes, 0);
}

#[test]
fn retained_124_fixture_conversion_keeps_answers_and_both_coverage_boolean_branches() {
    let mut cases =
        vec![FederalDraft::empty("124-empty".into()), benefit_return(FederalStatus::Joint)];
    for payroll_covered in [false, true] {
        let mut draft = complete(FederalStatus::Single);
        let mut source = omitted_employee_wages(13_579);
        let FederalIncome::EmployeeWages { data } = &mut source else { unreachable!() };
        data.complete = payroll_covered;
        add(&mut draft, income(source));
        cases.push(draft);
    }
    assert_eq!(
        LegacyFixtureEnvelope::StorageFormat { format: 124 }.to_avro(),
        TaxAppData::StorageFormat { format: 124 }.to_avro()
    );
    for draft in cases {
        let old_bytes =
            LegacyFixtureEnvelope::Draft { draft: draft.clone().try_into().unwrap() }.to_avro();
        let stored = TaxAppData::Draft { draft: draft.clone().try_into().unwrap() };
        assert_eq!(
            stored.to_avro(),
            old_bytes,
            "frozen-body fixture keeps its complete legacy envelope"
        );
        let TaxAppData::Draft { draft: retained } = TaxAppData::from_avro(&old_bytes).unwrap()
        else {
            unreachable!()
        };
        assert_eq!(
            retained.into_current().unwrap(),
            draft,
            "all current facts and old bool branches survive"
        );
    }
}

// 2026-10-18: produced by the original pre-HOME native App/SDK encoder, not
// by the retained DTOs. Stored business/office/casualty declarations match
// 701d0e944; the producing ad546ee6f export differs only in report routing.
#[test]
fn retained_124_native_home_and_casualty_bytes_keep_their_exact_facts() {
    use crate::migration124::{BusinessHomeOffice124, FederalIncome124, OfficeHome124};
    for (name, bytes) in [
        ("none", include_bytes!("../../../tests/fixtures/storage124/none.avro").as_slice()),
        (
            "simplified",
            include_bytes!("../../../tests/fixtures/storage124/simplified.avro").as_slice(),
        ),
        (
            "owned",
            include_bytes!("../../../tests/fixtures/storage124/actual-owned.avro").as_slice(),
        ),
        (
            "signed",
            include_bytes!("../../../tests/fixtures/storage124/actual-rented-signed.avro")
                .as_slice(),
        ),
    ] {
        let mut offset = 0;
        let value = TaxAppData::avro_decode(bytes, &mut offset).unwrap();
        assert_eq!(offset, bytes.len());
        assert_eq!(value.to_avro(), bytes);
        let TaxAppData::Draft { draft } = value else { panic!() };
        assert_eq!(draft.cookie, "synthetic-retained124-home");
        assert_eq!(draft.revision, 17);
        let FederalIncome124::Business { data } = &draft.income[0].income else { panic!() };
        assert!(matches!(
            data.activity,
            BusinessActivity::Ordinary {
                returns: 12_345,
                receipts: 5_000_000_123,
                accounting: BusinessAccounting::Accrual
            }
        ));
        let properties = &draft.deductions.as_ref().unwrap().personal_casualties[0].properties;
        assert_eq!(properties.len(), 2);
        assert_eq!(properties[0].basis, 6_000_000_123);
        assert_eq!(properties[0].before, 7_000_000_456);
        assert_eq!(properties[0].reimbursement, 2_000_000_321);
        assert_eq!(properties[1].basis, 123_456);
        assert_eq!(properties[1].received, 23_456);
        match (name, &data.home_office) {
            ("none", BusinessHomeOffice124::None) => {
                let counter = draft.next_id;
                let migrated = draft.into_current().unwrap();
                assert!(migrated.home_sites.is_empty());
                let event = &migrated.deductions.as_ref().unwrap().personal_casualties[0];
                assert_eq!(event.id, counter);
                assert_eq!(event.properties[0].id, counter + 1);
                assert_eq!(event.properties[1].id, counter + 2);
                assert_eq!(migrated.next_id, counter + 3);
                assert!(migrated.valid_stored());
            }
            ("simplified", BusinessHomeOffice124::Simplified { months, prior_operating, .. }) => {
                assert_eq!(*prior_operating, 4_294_967_397);
                assert_eq!(months[1].days, 14);
                assert_eq!(months[11].square_feet, 112);
                assert!(draft.into_current().is_err());
            }
            ("owned", BusinessHomeOffice124::ActualExpenses { office, eligible: true }) => {
                assert!(matches!(
                    office.home,
                    OfficeHome124::Owned {
                        basis: 5_000_000_123,
                        land: 1_000_000_321,
                        first_used: 20210315,
                        ..
                    }
                ));
                assert_eq!(office.direct, 56_789);
                assert!(draft.into_current().is_err());
            }
            ("signed", BusinessHomeOffice124::ActualExpenses { office, eligible: false }) => {
                assert_eq!(data.notary_profit, -4_294_967_399);
                assert_eq!(office.insurance, -12_345);
                assert_eq!(properties[0].received, -4_294_967_401);
                assert!(matches!(office.home, OfficeHome124::Rented { rent: 4_294_967_411 }));
                assert!(draft.into_current().is_err());
            }
            _ => panic!("retained office variant changed for {name}"),
        }
    }
}

#[test]
fn unavailable_legacy_home_migration_keeps_every_original_record() {
    for bytes in [
        include_bytes!("../../../tests/fixtures/storage124/simplified.avro").as_slice(),
        include_bytes!("../../../tests/fixtures/storage124/actual-owned.avro").as_slice(),
    ] {
        let mut store = Store::default();
        store.values.insert(
            TaxDataKey::StorageFormat { user: TEST_USER }.to_key_bytes().unwrap(),
            TaxAppData::StorageFormat { format: 124 }.to_avro(),
        );
        store
            .values
            .insert(TaxDataKey::Draft { user: TEST_USER }.to_key_bytes().unwrap(), bytes.to_vec());
        let original = store.values.clone();
        assert!(store.migrate(TEST_USER, 2026).is_err());
        assert_eq!(store.values, original);
        assert_eq!(store.writes, 0);
        assert_eq!(store.mutations, 0);
    }
}

#[test]
fn legacy_casualty_identity_allocation_cannot_exhaust_the_counter() {
    let TaxAppData::Draft { mut draft } =
        TaxAppData::from_avro(include_bytes!("../../../tests/fixtures/storage124/none.avro"))
            .unwrap()
    else {
        panic!()
    };
    draft.next_id = i64::MAX - 2;
    draft.deductions.as_mut().unwrap().personal_casualties[0].properties.truncate(1);
    assert!(draft.into_current().is_err());
}

#[test]
fn applied_credit_native_save_reopen_restart_and_rejection_are_atomic() {
    use crate::tests::cross::interview::{navigate, payments_value, save_payments};
    let draft = complete(FederalStatus::Single);
    let mut store = Store::default();
    store.seed(TEST_USER, draft.clone());
    let mut returns = cache(64 * 1024 * 1024, store);
    let request = navigate(&draft, TaxSection::Payments);
    let Some(Reply::Edit(page)) = returns.handle(TEST_USER, request) else { panic!("payments") };
    let mut value = payments_value(&page).0.clone();
    value.credit_applied_next_year = 100_050;
    value.credit_recipient = Some(OverpaymentRecipient::SpouseAccount);
    // Forged readonly capacity is ignored; the candidate computes its own.
    value.credit_capacity = Some(i64::MAX);
    let request = save_payments(&draft, value.clone());
    crate::tests::support::assert::accepted(&returns.handle(TEST_USER, request).unwrap());
    let accepted = returns.store().saved(TEST_USER);
    let mut restarted = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(restarted.interview(TEST_USER).draft, accepted);
    let request = navigate(&accepted, TaxSection::Payments);
    let Some(Reply::Edit(page)) = restarted.handle(TEST_USER, request) else { panic!("payments") };
    let reopened = payments_value(&page).0;
    assert_eq!(
        (reopened.credit_applied_next_year, reopened.credit_recipient),
        (100_050, Some(OverpaymentRecipient::SpouseAccount))
    );
    let maximum = reopened.credit_capacity.unwrap();
    let before = restarted.store().values.clone();
    value.credit_applied_next_year = maximum + 1; // raw cents, even if whole-dollar rounding fits
    let Some(Reply::Response(P::Problem { message, field, .. })) =
        restarted.handle(TEST_USER, save_payments(&accepted, value.clone()))
    else {
        panic!("excess credit must be rejected")
    };
    assert_eq!(field.as_deref(), Some("[value/credit_applied_next_year]"));
    assert_eq!(
        message,
        Message::new("PAY_CREDIT_ABOVE_CAPACITY", vec![MessageArg::usd_cents(maximum)]).encode()
    );
    assert_eq!(restarted.store().values, before);
    assert_eq!(restarted.interview(TEST_USER).draft, accepted);
    value.credit_applied_next_year = 0;
    let request = save_payments(&accepted, value);
    crate::tests::support::assert::accepted(&restarted.handle(TEST_USER, request).unwrap());
    let corrected = restarted.store().saved(TEST_USER);
    assert_eq!(corrected.payments.as_ref().unwrap().credit_applied_next_year, 0);
    let mut cold = cache(64 * 1024 * 1024, restarted.store().clone());
    assert_eq!(cold.interview(TEST_USER).draft, corrected);
}

#[test]
fn unit42_prior_home_carry_frozen124_bridge_never_invents_amt_or_character() {
    let mut source = rental(12_000, 0);
    source.home_depreciation_carryover = crate::SourceCents::Reported { amount: 12_345 };
    // This fixture represents only the facts present in the retained carrier.
    source.amt_home_depreciation_carryover = crate::SourceCents::Unavailable;
    source.home_casualty_carryover = crate::SourceCents::Unavailable;
    source.amt_home_casualty_carryover = crate::SourceCents::Unavailable;
    let retained = crate::migration124::FederalRental124::try_from(source).unwrap();
    let bytes = retained.to_avro();
    let mut offset = 0;
    let decoded = crate::migration124::FederalRental124::avro_decode(&bytes, &mut offset).unwrap();
    assert_eq!(offset, bytes.len());
    assert_eq!(decoded.to_avro(), bytes);
    assert_eq!(decoded.home_depreciation_carryover, 12_345);
    let current = FederalRental::from(decoded);
    assert_eq!(
        current.home_depreciation_carryover,
        crate::SourceCents::Reported { amount: 12_345 }
    );
    assert_eq!(current.amt_home_depreciation_carryover, crate::SourceCents::Unavailable);
    assert_eq!(current.home_casualty_carryover, crate::SourceCents::Unavailable);
    assert_eq!(current.amt_home_casualty_carryover, crate::SourceCents::Unavailable);
}

#[test]
fn unit42_prior_home_carry_frozen124_zero_is_unknown_and_inverse_preserves_facts() {
    let mut source = rental(12_000, 0);
    source.home_depreciation_carryover = crate::SourceCents::Unavailable;
    source.amt_home_depreciation_carryover = crate::SourceCents::Unavailable;
    source.home_casualty_carryover = crate::SourceCents::Unavailable;
    source.amt_home_casualty_carryover = crate::SourceCents::Unavailable;
    let retained = crate::migration124::FederalRental124::try_from(source.clone()).unwrap();
    assert_eq!(retained.home_depreciation_carryover, 0);
    let current = FederalRental::from(retained);
    assert_eq!(current, source);
    for (field, fact) in [
        (0, crate::SourceCents::NotReported),
        (0, crate::SourceCents::Reported { amount: 0 }),
        (1, crate::SourceCents::Reported { amount: 1 }),
        (2, crate::SourceCents::Reported { amount: 0 }),
        (3, crate::SourceCents::NotReported),
    ] {
        let mut unrepresentable = source.clone();
        *match field {
            0 => &mut unrepresentable.home_depreciation_carryover,
            1 => &mut unrepresentable.amt_home_depreciation_carryover,
            2 => &mut unrepresentable.home_casualty_carryover,
            _ => &mut unrepresentable.amt_home_casualty_carryover,
        } = fact;
        assert!(
            crate::migration124::FederalIncome124::try_from(FederalIncome::Rental {
                data: unrepresentable
            })
            .is_err()
        );
    }
}
