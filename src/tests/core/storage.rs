use super::*;
use crate::return_records;
use crate::storage::{
    GenerationMismatch, ReturnStore, Returns, persist_with_in, read_in, reset_mismatched_in,
};
use crate::tests::support::client::income_pages::open_documents;
use crate::tests::support::storage::{NativeTransaction, Store};
use core::num::NonZeroUsize;
use libertas::LibertasUser;
use libertas::data::{KeyCodec, wire};
use std::panic::{AssertUnwindSafe, catch_unwind};
extern crate std;

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
        refund: crate::tests::cross::interview::whole_refund(),
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
    let documents = (0..3)
        .map(|_| {
            crate::tests::cross::interview::document(
                0,
                IncomeSource::Wage {
                    value: crate::WageDocument::W2(wage_document(
                        crate::tests::cross::interview::w2(6_000_000),
                    )),
                },
            )
        })
        .collect();
    let request =
        crate::tests::cross::interview::save_documents(returns.interview(TEST_USER), documents);
    returns.interview(2);
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
    let navigation = crate::tests::cross::interview::navigate(&grown, TaxSection::Documents);
    let Reply::Edit(mut request @ P::SaveDocuments { .. }) =
        returns.handle(TEST_USER, navigation).unwrap()
    else {
        panic!("actual oversized source editor")
    };
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    documents.common.wages.documents.clear();
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, request).unwrap());
    assert_eq!(returns.encoded_bytes(), returns.store().bytes(TEST_USER));
    let small = returns.store().saved(TEST_USER);
    assert_eq!(returns.interview(TEST_USER).draft, small);
    // The cached growth path removes itself after commit, preserving no hidden active return.
    let documents = (0..3)
        .map(|_| {
            crate::tests::cross::interview::document(
                0,
                IncomeSource::Wage {
                    value: crate::WageDocument::W2(wage_document(
                        crate::tests::cross::interview::w2(6_000_000),
                    )),
                },
            )
        })
        .collect();
    let request =
        crate::tests::cross::interview::save_documents(returns.interview(TEST_USER), documents);
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
    let format_key = return_records::format_key(TEST_USER, draft.tax_year).to_key_bytes().unwrap();
    assert_eq!(
        TaxAppData::from_avro(&returns.store().values[&format_key]).unwrap(),
        TaxAppData::StorageFormat { format: STORAGE_FORMAT }
    );
    assert!(matches!(
        returns.handle(TEST_USER, finish(TEST_USER, &draft)),
        Some(Reply::Response(P::Problem { .. }))
    ));
    assert_eq!(returns.store().writes, 1);
    let request = crate::tests::cross::interview::save_documents(
        returns.interview(TEST_USER),
        vec![crate::tests::cross::interview::document(
            saved.income[0].id,
            IncomeSource::Wage {
                value: crate::WageDocument::W2(wage_document(crate::tests::cross::interview::w2(
                    6_000_000,
                ))),
            },
        )],
    );
    // The existing candidate owner decides whether these source facts changed.
    let _ = returns.handle(TEST_USER, request);
    let saved = returns.store().saved(TEST_USER);
    let mut cold = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(cold.interview(TEST_USER).draft, saved);
}

#[test]
fn accepted_callback_runs_once_after_identity_match_and_stores_final_cache_values() {
    let draft = complete(FederalStatus::Single);
    let base = return_records::header(&draft, 1);
    let mut stale = base.clone();
    stale.revision += 1;
    let mut store = Store::default();
    store.seed(TEST_USER, draft.clone());
    let mut calls = 0;
    assert_eq!(
        store.persist_with(TEST_USER, Some(&stale), draft.tax_year, || {
            calls += 1;
            (draft.clone(), true, ())
        }),
        None
    );
    assert_eq!(calls, 0);
    assert_eq!(store.writes, 0);
    let mut interview = Interview::new(TEST_USER, draft.clone());
    assert_eq!(
        store.persist_with(TEST_USER, Some(&base), draft.tax_year, || {
            calls += 1;
            assert!(matches!(
                interview.handle(finish(TEST_USER, &draft)),
                Some(Reply::Response(P::Finished { .. }))
            ));
            (interview.draft.clone(), interview.dirty, 7)
        }),
        Some(7)
    );
    assert_eq!(calls, 1);
    assert_eq!(store.writes, 1);
    let saved = store.saved(TEST_USER);
    assert_eq!(saved, interview.draft);
    assert!(saved.finished);
    assert_eq!(saved.revision, draft.revision + 1);
    let mut returns = cache(1024 * 1024, store);
    assert_eq!(returns.interview(TEST_USER).draft, saved);
    assert_eq!(returns.encoded_bytes(), returns.store().bytes(TEST_USER));
    let saved_base = return_records::header(&saved, 1);
    assert!(returns.store_mut().persist(TEST_USER, Some(&saved_base), &saved, false));
    assert_eq!(returns.store().writes, 1);
    assert_eq!(returns.encoded_bytes(), returns.store().bytes(TEST_USER));
}

#[test]
fn document_save_constructs_once_after_persisted_match_and_reloads_a_stale_editor() {
    let draft = complete(FederalStatus::Single);
    let mut store = Store::default();
    store.seed(TEST_USER, draft.clone());
    let mut returns = cache(64 * 1024 * 1024, store);
    let mut request = open_documents(returns.interview(TEST_USER));
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let crate::WageDocument::W2(form) = &mut documents.common.wages.documents[0].document else {
        unreachable!()
    };
    *form.wages.as_mut().expect("printed wages") += 10_000;
    let mut competing = draft.clone();
    competing.revision += 1;
    returns.store_mut().seed(TEST_USER, competing.clone());
    assert!(matches!(returns.handle(TEST_USER, request), Some(Reply::Response(P::Problem { .. }))));
    assert_eq!(returns.store().accepted_callbacks, 0);
    assert_eq!(returns.store().writes, 0);
    assert_eq!(returns.interview(TEST_USER).draft, competing);

    let mut request = open_documents(returns.interview(TEST_USER));
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let crate::WageDocument::W2(form) = &mut documents.common.wages.documents[0].document else {
        unreachable!()
    };
    *form.wages.as_mut().expect("printed wages") += 10_000;
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, request).unwrap());
    assert_eq!(returns.store().accepted_callbacks, 1);
    assert_eq!(returns.store().writes, 1);
    let saved = returns.store().saved(TEST_USER);
    assert_eq!(saved.revision, competing.revision + 1);
    assert_eq!(returns.interview(TEST_USER).draft, saved);
    let encoded = returns.encoded_bytes();
    let unchanged = open_documents(returns.interview(TEST_USER));
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, unchanged).unwrap());
    assert_eq!(returns.store().accepted_callbacks, 2);
    assert_eq!(returns.store().writes, 1);
    assert_eq!(returns.store().saved(TEST_USER), saved);
    assert_eq!(returns.interview(TEST_USER).draft, saved);
    assert_eq!(returns.encoded_bytes(), encoded);
    let mut cold = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(cold.interview(TEST_USER).draft, saved);
}

#[test]
fn failed_document_transaction_never_publishes_its_candidate_to_cache() {
    let draft = complete(FederalStatus::Single);
    let mut store = Store::default();
    store.seed(TEST_USER, draft.clone());
    let before = store.values.clone();
    let mut returns = cache(64 * 1024 * 1024, store);
    let mut request = open_documents(returns.interview(TEST_USER));
    let P::SaveDocuments { documents, .. } = &mut request else { unreachable!() };
    let crate::WageDocument::W2(form) = &mut documents.common.wages.documents[0].document else {
        unreachable!()
    };
    *form.wages.as_mut().expect("printed wages") += 10_000;
    returns.store_mut().fail_commit = true;
    assert!(catch_unwind(AssertUnwindSafe(|| returns.handle(TEST_USER, request))).is_err());
    assert_eq!(returns.store().accepted_callbacks, 1);
    assert_eq!(returns.store().values, before);
    assert_eq!(returns.interview(TEST_USER).draft, draft);
    assert!(!returns.interview(TEST_USER).dirty);
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
fn missing_or_unknown_generation_rejects_before_versioned_values_or_edits() {
    let draft = complete(FederalStatus::Single);
    let header = return_records::header(&draft, 1);
    for format in [None, Some(1), Some(STORAGE_FORMAT - 1), Some(STORAGE_FORMAT + 1)] {
        let mut store = Store::default();
        store.seed(TEST_USER, draft.clone());
        let key = return_records::format_key(TEST_USER, draft.tax_year).to_key_bytes().unwrap();
        let error = if let Some(format) = format {
            store.values.insert(key, TaxAppData::StorageFormat { format }.to_avro());
            "Unknown or corrupt storage format marker"
        } else {
            store.values.remove(&key);
            "Records without storage format marker"
        };
        let mut tx = NativeTransaction::new(&store);
        // A decoder sentinel models an unreadable retired header/body without
        // constructing malformed or out-of-bounds schema values.
        tx.block_versioned_reads = true;
        assert_eq!(read_in(&mut tx, TEST_USER, draft.tax_year).err(), Some(error));
        let mut admitted = false;
        assert_eq!(
            persist_with_in(&mut tx, TEST_USER, Some(&header), draft.tax_year, || {
                admitted = true;
                (&draft, true, ())
            }),
            Err(error)
        );
        assert!(!admitted, "incompatible generation must not admit an edit");
        assert_eq!(tx.scans, 0, "typed body records must remain unread");
        assert_eq!(tx.key_scans, if format.is_none() { 2 } else { 0 });
        assert_eq!(tx.mutations, 0);
        assert_eq!(tx.values, store.values);
    }
}

#[test]
fn wage_specials_saved_sources_survive_store_cache_and_cold_read() {
    let mut d = complete(FederalStatus::Joint);
    add(&mut d, income(cash_business(0)));
    let business_id = d.income[1].id;
    let mut store = Store::default();
    store.seed(TEST_USER, d.clone());
    let mut returns = cache(64 * 1024 * 1024, store);
    let mut statutory = w2_source(crate::tests::cross::interview::w2(5_000_000));
    statutory.more.statutory = true;
    statutory.answers.statutory_business =
        Some(StatutoryBusiness::Business { source: business_id });
    statutory.answers.medicaid_waiver = 100_000;
    statutory.more.box12 = vec![
        Box12Entry { code: W2Code::Q, amount: 200_000 },
        Box12Entry { code: W2Code::II, amount: 300_000 },
    ];
    let mut w = crate::tests::cross::interview::w2(26_000_000);
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
    let mut railroad = crate::tests::cross::interview::document(
        0,
        IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(w)) },
    );
    railroad.owner = crate::DocumentOwner::Spouses;
    let request = crate::tests::cross::interview::save_documents(
        returns.interview(TEST_USER),
        vec![
            crate::tests::cross::interview::document(
                1,
                IncomeSource::Wage { value: crate::WageDocument::W2(wage_document(statutory)) },
            ),
            railroad,
        ],
    );
    let supplied = crate::tests::support::client::income_pages::page_documents(&request);
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, request).unwrap());
    let answers = crate::tests::support::client::income_pages::information_return_answers(
        returns.interview(TEST_USER),
        &supplied,
    );
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, answers).unwrap());
    let accepted = returns.interview(TEST_USER).draft.clone();
    assert!(accepted.valid_stored());
    assert_eq!(returns.store().saved(TEST_USER), accepted);
    let mut cold = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(cold.interview(TEST_USER).draft, accepted);
}

#[test]
fn statutory_wage_link_clear_survives_save_and_cold_read() {
    let accepted = statutory_storage_return(true);
    let mut store = Store::default();
    store.seed(TEST_USER, accepted.clone());
    let mut cold = cache(64 * 1024 * 1024, store);
    let mut clear_link = open_documents(cold.interview(TEST_USER));
    let P::SaveDocuments { documents, .. } = &mut clear_link else { unreachable!() };
    let row = documents
        .common
        .wages
        .documents
        .iter_mut()
        .find(|row| row.id == accepted.income[0].id)
        .unwrap();
    let crate::WageDocument::W2(wage) = &mut row.document else { unreachable!() };
    let mut answers = stored_w2_answers(&accepted, 0);
    answers.statutory_business = None;
    wage_document(W2Source { form: wage.clone(), answers });
    let supplied = crate::tests::support::client::income_pages::page_documents(&clear_link);
    crate::tests::cross::interview::accepted(&cold.handle(TEST_USER, clear_link).unwrap());
    let answers = crate::tests::support::client::income_pages::information_return_answers(
        cold.interview(TEST_USER),
        &supplied,
    );
    crate::tests::cross::interview::accepted(&cold.handle(TEST_USER, answers).unwrap());
    let cleared = cold.interview(TEST_USER).draft.clone();
    let FederalIncome::Wage { data } = &cleared.income[0].income else { panic!() };
    let data = &FederalWage::from(data);
    assert_eq!(data.statutory_business, None);
    assert_eq!(data.wages, 5_000_000);
    assert_eq!(data.medicaid_waiver, 100_000);
    let mut reopened = cache(64 * 1024 * 1024, cold.store().clone());
    assert_eq!(reopened.interview(TEST_USER).draft, cleared);
    assert_eq!(reopened.store().saved(TEST_USER), cleared);
}

#[test]
fn unlinked_statutory_wage_survives_business_deletion_and_cold_read() {
    let draft = statutory_storage_return(false);
    let mut store = Store::default();
    store.seed(TEST_USER, draft);
    let mut cold = cache(64 * 1024 * 1024, store);
    let request = crate::tests::cross::interview::save_topic(
        cold.interview(TEST_USER),
        TaxSection::SelfEmployment,
        vec![],
    );
    crate::tests::cross::interview::accepted(&cold.handle(TEST_USER, request).unwrap());
    let deleted = cold.interview(TEST_USER).draft.clone();
    let FederalIncome::Wage { data } = &deleted.income[0].income else { panic!() };
    let data = &FederalWage::from(data);
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
pub(in crate::tests) fn exercise_heavy(draft: &FederalDraft) {
    assert!(draft.valid_stored());
    let records = return_records::project(TEST_USER, draft, 1).unwrap();
    let expected_bytes: usize = records
        .iter()
        .map(|(key, value)| key.to_key_bytes().unwrap().len() + value.to_avro().len())
        .sum();
    let mut store = Store { fail_after: Some(records.len()), ..Store::default() };
    assert!(
        catch_unwind(AssertUnwindSafe(|| store.persist(TEST_USER, None, draft, true))).is_err()
    );
    assert!(store.values.is_empty());
    store.fail_after = None;
    assert!(store.persist(TEST_USER, None, draft, true));
    assert_eq!(store.mutation_bytes, expected_bytes);
    assert_eq!(store.mutations, records.len());
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
    let mut page = open_documents(returns.interview(TEST_USER));
    let P::SaveDocuments { documents, .. } = &mut page else { unreachable!() };
    for item in &mut documents.common.investment_sales.documents {
        let crate::InvestmentSaleDocument::Sale1099B(sale) = &mut item.document else {
            unreachable!()
        };
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
    let mut prune = open_documents(returns.interview(TEST_USER));
    let P::SaveDocuments { documents, .. } = &mut prune else { unreachable!() };
    documents.common.investment_sales.documents.clear();
    crate::tests::cross::interview::accepted(&returns.handle(TEST_USER, prune).unwrap());
    let pruned = returns.store().saved(TEST_USER);
    assert_eq!(pruned.income.len(), 1, "the W-2 stays");
    assert_eq!(pruned.next_id, draft.next_id);
    let mut restarted = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(restarted.interview(TEST_USER).draft, pruned);
    std::println!(
        "Unit53 full native bounded save rows={} mutations={mutations} bytes={bytes}; correction and prune committed; real Hub not exercised",
        draft.income.len()
    );
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
        store.values.insert(TaxDataKey::StorageFormat { user }.to_key_bytes().unwrap(), vec![255]);
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
    // Long enough that the payment section is the largest stored record of the return.
    let explanation =
        "The disaster closed my workplace and delayed my estimated tax payment. ".repeat(8);
    let explanation = explanation.trim_end();
    let P::SavePayments { value, .. } = &mut page else { unreachable!() };
    value.more.penalty = PenaltyChoice::Figure {
        prior_tax: 1_000_000,
        prior_agi: Some(6_000_000),
        full_year: true,
        prior_return: PriorReturn::SameReturn,
        farmer: Farmer::No,
        withholding: WithholdingTiming::Evenly,
        income: IncomeTiming::Evenly,
        waiver: Waiver::Full {
            reason: PenaltyWaiverReason::CasualtyOrDisasterOrUnusual,
            explanation: explanation.into(),
        },
    };
    assert!(page.to_avro().len() < wire::MAX_REQUEST_BYTES);
    // Admit the ordinary waiver through the same editor before modeling a backend
    // whose capacity is one encoded byte below this complete payment section.
    // The SDK value guard above keeps its real limit independently.
    let mut admitted = Interview::new(TEST_USER, draft.clone());
    crate::tests::cross::interview::accepted(&crate::tests::support::client::send(
        &mut admitted,
        page.clone(),
    ));
    let records = return_records::project(TEST_USER, &admitted.draft, 1).unwrap();
    let payment_bytes = records
        .iter()
        .find_map(|(_, value)| {
            matches!(value, TaxAppData::PaymentsSection { .. }).then(|| value.to_avro().len())
        })
        .unwrap();
    let limit = payment_bytes - 1;
    // Header and unrelated sections must fit, so the failed write is the intact
    // payment value rather than an earlier mutation.
    assert!(records.iter().all(|(_, value)| {
        matches!(value, TaxAppData::PaymentsSection { .. }) || value.to_avro().len() <= limit
    }));
    returns.store_mut().backend_value_limit = Some(limit);
    let before = returns.store().values.clone();
    assert!(catch_unwind(AssertUnwindSafe(|| returns.handle(TEST_USER, page))).is_err());
    assert_eq!(returns.store().values, before);
    assert_eq!(returns.store().writes, 0);
    let candidate = &returns.interview(TEST_USER).draft;
    assert!(candidate.valid_stored(), "native accepted answers remain valid before Data limit");
    let Some(PenaltyWaiver::Full { explanation: retained, .. }) =
        &candidate.payments.as_ref().unwrap().waiver
    else {
        unreachable!()
    };
    assert_eq!(retained, explanation);
    assert_eq!(candidate, &admitted.draft);
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
fn current_employee_payroll_sources_survive_native_storage_roundtrip() {
    use crate::{EmployeeCoverage, EmployeePayrollSource, EmployeeWageReason, SourceCents};
    let mut draft = complete(FederalStatus::Single);
    let railroad = employee_railroad_source([
        SourceCents::Reported { amount: 13_579 },
        SourceCents::Unavailable,
        SourceCents::NotReported,
        SourceCents::Reported { amount: 197 },
        SourceCents::NotReported,
    ]);
    for payroll in [
        EmployeePayrollSource::Unavailable,
        EmployeePayrollSource::Form8919 {
            reason: EmployeeWageReason::Ss8Determination { date: 20260501 },
        },
        EmployeePayrollSource::Government {
            social: EmployeeCoverage::NotCovered,
            medicare: EmployeeCoverage::Covered,
            reason: Some(EmployeeWageReason::Ss8Pending { filed_on: 20260601 }),
        },
        EmployeePayrollSource::Railroad { reported: railroad.clone() },
        EmployeePayrollSource::TotalizationCertificate,
        EmployeePayrollSource::ForeignGovernment,
        EmployeePayrollSource::RailroadRepresentative { reported: railroad },
    ] {
        let mut source = omitted_employee_wages(13_579);
        let FederalIncome::EmployeeWages { data } = &mut source else { unreachable!() };
        data.payroll = payroll;
        add(&mut draft, income(source));
    }
    f18_cold(&draft);
}

#[test]
fn applied_credit_native_save_reopen_restart_and_rejection_are_atomic() {
    use crate::tests::cross::interview::finish_request;
    let draft = complete(FederalStatus::Single);
    let mut store = Store::default();
    store.seed(TEST_USER, draft.clone());
    let mut returns = cache(64 * 1024 * 1024, store);
    let start = |d: &FederalDraft| P::StartFinish {
        user: TEST_USER,
        cookie: d.cookie.clone(),
        revision: d.revision,
    };
    let Some(Reply::Response(P::BeginFinish { mut refund, .. })) =
        returns.handle(TEST_USER, start(&draft))
    else {
        panic!("finish")
    };
    refund.credit_applied_next_year = 100_050;
    refund.credit_recipient = Some(OverpaymentRecipient::SpouseAccount);
    // Forged readonly capacity is ignored; the candidate computes its own.
    refund.credit_capacity = Some(i64::MAX);
    let request = finish_request(&draft, refund.clone());
    assert!(matches!(
        returns.handle(TEST_USER, request),
        Some(Reply::Response(P::Finished { .. }))
    ));
    let accepted = returns.store().saved(TEST_USER);
    let mut restarted = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(restarted.interview(TEST_USER).draft, accepted);
    let Some(Reply::Response(P::BeginFinish { refund: reopened, .. })) =
        restarted.handle(TEST_USER, start(&accepted))
    else {
        panic!("finish")
    };
    assert_eq!(
        (reopened.credit_applied_next_year, reopened.credit_recipient),
        (100_050, Some(OverpaymentRecipient::SpouseAccount))
    );
    let maximum = reopened.credit_capacity.unwrap();
    let before = restarted.store().values.clone();
    refund.credit_applied_next_year = maximum + 1; // raw cents, even if whole-dollar rounding fits
    let Some(Reply::Response(P::Problem { message, field, .. })) =
        restarted.handle(TEST_USER, finish_request(&accepted, refund.clone()))
    else {
        panic!("excess credit must be rejected")
    };
    assert_eq!(field.as_deref(), Some("[refund/credit_applied_next_year]"));
    assert_eq!(
        message,
        Message::new("PAY_CREDIT_ABOVE_CAPACITY", vec![MessageArg::usd_cents(maximum)]).encode()
    );
    assert_eq!(restarted.store().values, before);
    assert_eq!(restarted.interview(TEST_USER).draft, accepted);
    refund.credit_applied_next_year = 0;
    let request = finish_request(&accepted, refund);
    assert!(matches!(
        restarted.handle(TEST_USER, request),
        Some(Reply::Response(P::Finished { .. }))
    ));
    let corrected = restarted.store().saved(TEST_USER);
    assert_eq!(corrected.payments.as_ref().unwrap().credit_applied_next_year, 0);
    let mut cold = cache(64 * 1024 * 1024, restarted.store().clone());
    assert_eq!(cold.interview(TEST_USER).draft, corrected);
}

#[test]
fn mismatched_return_offers_confirmation_without_mutation_then_initializes_only_its_user_year() {
    for format in [None, Some(STORAGE_FORMAT - 1)] {
        let mut store = Store::default();
        let draft = complete(FederalStatus::Single);
        store.seed(TEST_USER, draft.clone());
        store.seed(TEST_USER + 1, draft.clone());
        let mut prior = draft.clone();
        prior.tax_year -= 1;
        store.seed(TEST_USER, prior);
        let key = return_records::format_key(TEST_USER, 2026).to_key_bytes().unwrap();
        match format {
            None => {
                store.values.remove(&key);
            }
            Some(format) => {
                store.values.insert(key, TaxAppData::StorageFormat { format }.to_avro());
            }
        }
        let prefix = TaxDataKey::return_record_prefix(TEST_USER, 2026).unwrap();
        let untouched: Vec<_> = store
            .values
            .iter()
            .filter(|(key, _)| !key.starts_with(&prefix))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let old_count = store.values.keys().filter(|key| key.starts_with(&prefix)).count();
        let before = store.values.clone();
        let mut returns = cache(1024 * 1024, store);
        for request in [P::Enter { user: TEST_USER }, P::OpenInterview { user: TEST_USER }] {
            let Some(Reply::Response(P::StoredReturnUnavailable {
                tax_year,
                format: observed,
                ..
            })) = returns.handle(TEST_USER, request)
            else {
                panic!("mismatch refusal")
            };
            assert_eq!((tax_year, observed), (2026, format));
        }
        assert!(matches!(
            returns.handle(TEST_USER, P::EstimateSetup {
                user: TEST_USER,
                value: ReturnSetup { filing: Filing::MarriedJointly },
            }),
            Some(Reply::Response(P::Problem { message, field: None }))
                if message == libertas::libertas_formatted_text("STORED_RETURN_OLDER_VERSION", &[])
        ));
        assert_eq!(
            returns.store().values,
            before,
            "leaving or cancelling the confirmation sends nothing"
        );
        assert_eq!(returns.store().writes, 0);
        assert!(matches!(
            returns
                .handle(TEST_USER, P::StartNewReturn { user: TEST_USER, tax_year: 2026, format }),
            Some(Reply::Response(P::BeginSetup { .. })) | Some(Reply::Edit(P::SaveSetup { .. }))
        ));
        let fresh = returns.store().saved(TEST_USER);
        let mut ordinary = cache(1024 * 1024, Store::default());
        ordinary.handle(TEST_USER, P::OpenInterview { user: TEST_USER });
        let mut expected = ordinary.store().saved(TEST_USER);
        expected.cookie = fresh.cookie.clone();
        assert_eq!(fresh, expected);
        assert_eq!(
            returns.store().writes,
            2,
            "one deletion transaction, then normal first editor commit"
        );
        assert_eq!(
            returns
                .store()
                .values
                .iter()
                .filter(|(key, _)| !key.starts_with(&prefix))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<Vec<_>>(),
            untouched
        );
        assert_eq!(returns.store().reset_removed, Some(old_count));
        assert!(
            matches!(TaxAppData::from_avro(&returns.store().values[&return_records::format_key(TEST_USER, 2026).to_key_bytes().unwrap()]).unwrap(), TaxAppData::StorageFormat { format } if format == STORAGE_FORMAT)
        );
    }
}

#[test]
fn confirmed_reset_rechecks_generation_and_never_decodes_retired_values() {
    let draft = complete(FederalStatus::Single);
    let mut store = Store::default();
    store.seed(TEST_USER, draft.clone());
    let marker = return_records::format_key(TEST_USER, 2026).to_key_bytes().unwrap();
    store.values.remove(&marker);
    let count = store.values.len();
    let mut tx = NativeTransaction::new(&store);
    tx.block_versioned_reads = true;
    assert_eq!(
        reset_mismatched_in(&mut tx, TEST_USER, 2026, GenerationMismatch { format: None }),
        Ok(Some(count))
    );
    assert!(tx.values.is_empty());
    assert_eq!(tx.scans, 0);
    assert_eq!(tx.mutations, count);
    store.values.insert(marker, TaxAppData::StorageFormat { format: STORAGE_FORMAT - 1 }.to_avro());
    let before = store.values.clone();
    let mut changed = NativeTransaction::new(&store);
    assert_eq!(
        reset_mismatched_in(&mut changed, TEST_USER, 2026, GenerationMismatch { format: None }),
        Ok(None)
    );
    assert_eq!(changed.values, before);
    assert_eq!(changed.mutations, 0);
    store.seed(TEST_USER, draft);
    let before = store.values.clone();
    let mut returns = cache(1024 * 1024, store);
    for tax_year in [2026, 2025] {
        let _ = returns
            .handle(TEST_USER, P::StartNewReturn { user: TEST_USER, tax_year, format: None });
        assert_eq!(returns.store().values, before);
    }
    assert_eq!(returns.store().writes, 0);
}

#[test]
fn corrupt_marker_or_current_required_records_never_offer_or_perform_reset() {
    for corrupt_marker in [false, true] {
        let mut store = Store::default();
        store.seed(TEST_USER, complete(FederalStatus::Single));
        if corrupt_marker {
            store.values.insert(
                return_records::format_key(TEST_USER, 2026).to_key_bytes().unwrap(),
                TaxAppData::SetupSection { setup: None }.to_avro(),
            );
        } else {
            store
                .values
                .remove(&return_records::header_key(TEST_USER, 2026).to_key_bytes().unwrap());
        }
        let before = store.values.clone();
        let mut returns = cache(1024 * 1024, store);
        assert!(
            catch_unwind(AssertUnwindSafe(
                || returns.handle(TEST_USER, P::OpenInterview { user: TEST_USER })
            ))
            .is_err()
        );
        let result = catch_unwind(AssertUnwindSafe(|| {
            returns.handle(
                TEST_USER,
                P::StartNewReturn { user: TEST_USER, tax_year: 2026, format: None },
            )
        }));
        if corrupt_marker {
            assert!(result.is_err());
        }
        assert_eq!(returns.store().values, before);
        assert_eq!(returns.store().writes, 0);
    }
}

#[test]
fn source_only_payment_parts_persist_before_reply_and_keep_ids_after_cold_read() {
    let (session, printed) =
        crate::tests::support::builders::related_forms::related_component_editor();
    let mut store = Store::default();
    store.seed(TEST_USER, session.draft);
    let mut returns = cache(64 * 1024 * 1024, store);
    accepted(&returns.handle(TEST_USER, printed).unwrap());
    let draft = &returns.interview(TEST_USER).draft;
    let source = draft
        .income
        .iter()
        .find_map(|entry| matches!(entry.income, FederalIncome::K1099 { .. }).then_some(entry.id))
        .unwrap();
    let request = P::SavePaymentParts {
        user: TEST_USER,
        cookie: draft.cookie.clone(),
        revision: draft.revision,
        key: alloc::format!("income/{source}/parts"),
        value: vec![
            crate::KPart { id: 0, amount: 15_000, belongs: None, refunded: 0 },
            crate::KPart { id: 0, amount: 25_000, belongs: None, refunded: 0 },
        ],
    };
    let request = P::from_avro(&request.to_avro()).unwrap();
    let Reply::Response(P::PaymentPartsSaved { value, revision }) =
        returns.handle(TEST_USER, request.clone()).unwrap()
    else {
        panic!("expected accepted source-only return")
    };
    let accepted = returns.store().saved(TEST_USER);
    assert_eq!(accepted, returns.interview(TEST_USER).draft);
    assert_eq!(accepted.revision, revision);
    let FederalIncome::K1099 { data } =
        &accepted.income.iter().find(|entry| entry.id == source).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.parts, value);
    assert!(data.parts.iter().all(|part| part.id > 0 && part.id != source));
    let mut cold = cache(64 * 1024 * 1024, returns.store().clone());
    assert_eq!(cold.interview(TEST_USER).draft, accepted);
    let before = cold.store().values.clone();
    error(cold.handle(TEST_USER, request).unwrap());
    assert_eq!(cold.interview(TEST_USER).draft, accepted);
    assert_eq!(cold.store().values, before, "stale source submission cannot change persisted data");
}
