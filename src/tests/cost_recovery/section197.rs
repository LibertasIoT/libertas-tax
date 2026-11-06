use super::*;
use crate::tests::cross::interview::{
    business, reopened_businesses, save_businesses, stored_business,
};
use crate::tests::support::builders::ordinary_intangible;
use crate::tests::support::client::cohorts::save_existing_business;

#[test]
fn section197_classifies_actual_right_and_transaction_before_ordinary_recovery() {
    let base = ordinary_intangible(1_800_000, 20260701, 20200101, 0);
    for (kind, obtained, context, expected) in [
        (
            IntangibleKind::Goodwill,
            IntangibleObtained::Purchased,
            IntangibleAcquisitionContext::EntireOperatingBusinessAssets,
            None,
        ),
        (
            IntangibleKind::Goodwill,
            IntangibleObtained::Created,
            IntangibleAcquisitionContext::SeparateRight,
            Some("INTANGIBLE_SELF_CREATED"),
        ),
        (
            IntangibleKind::Goodwill,
            IntangibleObtained::Created,
            IntangibleAcquisitionContext::EntireOperatingBusinessAssets,
            None,
        ),
        (
            IntangibleKind::Trademark,
            IntangibleObtained::Created,
            IntangibleAcquisitionContext::SeparateRight,
            None,
        ),
        (
            IntangibleKind::GovernmentRight,
            IntangibleObtained::Created,
            IntangibleAcquisitionContext::EntireOperatingBusinessAssets,
            None,
        ),
        (
            IntangibleKind::GovernmentRight,
            IntangibleObtained::Created,
            IntangibleAcquisitionContext::SeparateRight,
            Some("INTANGIBLE_FINITE_SOURCE"),
        ),
        (
            IntangibleKind::Noncompete,
            IntangibleObtained::Created,
            IntangibleAcquisitionContext::BusinessOwnershipInterest,
            None,
        ),
        (
            IntangibleKind::Noncompete,
            IntangibleObtained::Purchased,
            IntangibleAcquisitionContext::SeparateRight,
            Some("INTANGIBLE_NONCOMPETE_TRANSACTION"),
        ),
        (
            IntangibleKind::FilmBookRecording,
            IntangibleObtained::Purchased,
            IntangibleAcquisitionContext::EntireOperatingBusinessAssets,
            None,
        ),
        (
            IntangibleKind::FilmBookRecording,
            IntangibleObtained::Purchased,
            IntangibleAcquisitionContext::SeparateRight,
            Some("INTANGIBLE_CREATIVE_SOURCE"),
        ),
        (
            IntangibleKind::Software,
            IntangibleObtained::Purchased,
            IntangibleAcquisitionContext::EntireOperatingBusinessAssets,
            Some("INTANGIBLE_SOFTWARE_SOURCE"),
        ),
        (
            IntangibleKind::PatentCopyrightKnowhow,
            IntangibleObtained::Purchased,
            IntangibleAcquisitionContext::SeparateRight,
            Some("INTANGIBLE_FINITE_SOURCE"),
        ),
        (
            IntangibleKind::Goodwill,
            IntangibleObtained::Gift,
            IntangibleAcquisitionContext::EntireOperatingBusinessAssets,
            Some("INTANGIBLE_TRANSFER_SOURCE"),
        ),
        (
            IntangibleKind::Goodwill,
            IntangibleObtained::Purchased,
            IntangibleAcquisitionContext::SelectedOperatingAssets {
                acquired: "Customer-facing assets".into(),
                retained_by_seller: "Other operating division".into(),
            },
            Some("INTANGIBLE_SELECTED_ASSETS"),
        ),
    ] {
        let mut record = base.clone();
        record.source.kind = kind;
        record.source.obtained = obtained;
        record.source.context = context;
        assert_eq!(record.boundary(), expected, "{:?}", record.source);
    }
}

#[test]
fn section197_origin_evidence_distinguishes_creation_from_existence_and_renewal() {
    let mut record = ordinary_intangible(1_800_000, 20260701, 20200101, 0);
    for (origin, expected) in [
        (None, Some("INTANGIBLE_HISTORIC_SOURCE")),
        (
            Some(IntangibleOriginEvidence::ExistedBy { date: 20260701 }),
            Some("INTANGIBLE_HISTORIC_SOURCE"),
        ),
        (
            Some(IntangibleOriginEvidence::CreatedOn { date: 19930810 }),
            Some("INTANGIBLE_HISTORIC_SOURCE"),
        ),
        (Some(IntangibleOriginEvidence::CreatedOn { date: 19930811 }), None),
        (
            Some(IntangibleOriginEvidence::CreatedAfter { date: 19930809 }),
            Some("INTANGIBLE_HISTORIC_SOURCE"),
        ),
        (Some(IntangibleOriginEvidence::CreatedAfter { date: 19930810 }), None),
    ] {
        record.source.first_existed = origin;
        assert_eq!(record.boundary(), expected);
    }
    record.source.obtained = IntangibleObtained::Renewal { previous_intangible_id: None };
    record.source.first_existed = Some(IntangibleOriginEvidence::ExistedBy { date: 20260701 });
    assert_eq!(record.boundary(), Some("INTANGIBLE_HISTORIC_SOURCE"));
}

#[test]
fn section197_prior_history_and_unknown_event_use_lease_facts_are_not_zero() {
    let base = ordinary_intangible(1_800_000, 20250701, 20200101, 60_000);
    assert_eq!(base.deduction(), 120_000);
    let mut record = base.clone();
    record.prior_claimed.amt = SourceCents::Unavailable;
    assert_eq!(record.boundary(), Some("INTANGIBLE_PRIOR_UNAVAILABLE"));
    record.prior_claimed.amt = SourceCents::NotReported;
    assert_eq!(record.boundary(), Some("INTANGIBLE_PRIOR_CORRECTION"));
    record = base.clone();
    record.other_events = None;
    assert_eq!(record.boundary(), Some("INTANGIBLE_EVENTS_UNAVAILABLE"));
    record.other_events = Some(vec![IntangibleEventKind::PurchasePriceReduction]);
    assert_eq!(record.boundary(), Some("INTANGIBLE_RECORDED_EVENTS"));
    record = base.clone();
    record.use_end = IntangibleUseEnd::RecordsUnavailable;
    assert_eq!(record.boundary(), Some("INTANGIBLE_USE_UNAVAILABLE"));
    record.use_end = IntangibleUseEnd::Ended {
        record: AmortizationEnd { date: 20260401, how: DisposalKind::Scrapped },
    };
    assert_eq!(record.boundary(), Some("INTANGIBLE_DISPOSITION_GROUP"));
    record = base;
    record.source.lease_status = IntangibleLeaseStatus::Leased;
    assert_eq!(record.boundary(), Some("INTANGIBLE_LEASE_SOURCE"));
    record.source.lease_status = IntangibleLeaseStatus::RecordsUnavailable;
    assert_eq!(record.boundary(), Some("INTANGIBLE_LEASE_UNAVAILABLE"));
}

#[test]
fn section197_unknown_source_refuses_native_section179_income_certification() {
    for (receipts, equipment_cost, elected, binds) in
        [(10_000_000, 500_000, 100_000, false), (1_000_000, 10_000_000, 9_000_000, true)]
    {
        let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
        let mut page = business(receipts);
        let mut equipment =
            crate::tests::cross::interview::asset(equipment_cost, 20260701, AssetClass::Five);
        equipment.acquired = Some(20260701);
        equipment.gds_mut().unwrap().history = AssetHistory::New {
            first_year: FirstYearDeduction::Section179 {
                description: "Synthetic purchased equipment".into(),
                elected,
            },
        };
        page.assets.push(equipment);
        let mut intangible = ordinary_intangible(1_800_000, 20260701, 20200101, 0);
        intangible.other_events = None;
        page.more.intangibles.push(intangible.into());
        accepted(&save_businesses(&mut session, vec![page]));
        let (activity, _) = stored_business(&session);
        assert!(session.draft.valid_stored());
        for system in [Section179IncomeSystem::Regular, Section179IncomeSystem::Alternative] {
            let sources =
                crate::section179::section179_direct_income_sources(&session.draft, system);
            assert!(
                matches!(
                    sources,
                    Err(Section179PreparedIssue::SourceBoundaryUnavailable(
                        id,
                        "INTANGIBLE_EVENTS_UNAVAILABLE"
                    )) if id == activity
                ),
                "{system:?}: {sources:?}"
            );
        }
        // 2026-10-23: app-contract Answers and unknowns permits a disclosed
        // conservative benefit omission, never certification of the raw unknown.
        let result = supported(&session.draft);
        assert!(noted(&result, "WARN_INTANGIBLE_BENEFIT_LEFT_OUT"));
        assert!(!result.financial_source_scope_complete);
        let mut baseline = session.draft.clone();
        let entry = baseline.income.iter_mut().find(|entry| entry.id == activity).unwrap();
        let FederalIncome::Business { data } = &mut entry.income else { panic!() };
        data.intangibles.clear();
        let baseline = supported(&baseline);
        assert_eq!(value(&result, "agi"), value(&baseline, "agi"));
        assert_eq!(value(&result, "total_tax"), value(&baseline, "total_tax"));
        if binds {
            assert_eq!(
                value(&result, "section179_deduction"),
                value(&result, "section179_business_income_limit")
            );
            assert!(value(&result, "section179_deduction") < elected / 100);
        }
    }
}

#[test]
fn section197_native_save_reopen_restart_correction_and_predecessor_identity() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut page = business(5_000_000);
    let mut first = ordinary_intangible(1_800_000, 20250701, 20200101, 60_000);
    first.source.lease_status = IntangibleLeaseStatus::RecordsUnavailable;
    first.other_events = None;
    first.use_end = IntangibleUseEnd::RecordsUnavailable;
    page.more.intangibles = vec![first.into()];
    accepted(&save_businesses(&mut session, vec![page]));
    let (activity, stored) = stored_business(&session);
    let id = stored.intangibles[0].id;
    assert!(id > activity && id < session.draft.next_id);
    assert!(stored.intangibles[0].valid());
    let saved = session.draft.clone();
    let mut page = reopened_businesses(saved.clone()).remove(0);
    assert_eq!(page.more.intangible_predecessor_choices.len(), 1);
    assert_eq!(page.more.intangible_predecessor_choices[0].id, id);
    assert_eq!(
        Message::decode(&page.more.intangible_predecessor_choices[0].label).english(),
        stored.intangibles[0].label
    );
    assert_eq!(
        page.more.intangibles[0].source.lease_status,
        IntangibleLeaseStatus::RecordsUnavailable
    );
    assert_eq!(page.more.intangibles[0].other_events, None);
    assert_eq!(page.more.intangibles[0].use_end, IntangibleUseEnd::RecordsUnavailable);
    accepted(&save_existing_business(&mut session, activity, page.clone()));
    assert_eq!(session.draft, saved);
    page.more.intangibles[0].source.lease_status = IntangibleLeaseStatus::NotLeased;
    page.more.intangibles[0].other_events = Some(vec![]);
    page.more.intangibles[0].use_end = IntangibleUseEnd::StillHeldAndUsed;
    let mut renewal = ordinary_intangible(180_000, 20260701, 20200101, 0);
    renewal.source.obtained = IntangibleObtained::Renewal { previous_intangible_id: Some(id) };
    renewal.source.first_existed = Some(IntangibleOriginEvidence::CreatedOn { date: 20250701 });
    page.more.intangibles.push(renewal.into());
    accepted(&save_existing_business(&mut session, activity, page));
    let (_, corrected) = stored_business(&session);
    assert_eq!(corrected.intangibles[0].id, id);
    let renewal_id = corrected.intangibles[1].id;
    assert_ne!(id, renewal_id);
    let sources = crate::source_associations::SourceAssociations::new(&session.draft);
    assert_eq!(sources.intangible_predecessor(activity, renewal_id).unwrap().unwrap().id, id);
    assert!(sources.asset(activity, id).is_err());
    assert!(sources.intangible(activity + 100, id).is_err());
    let mut invalid = reopened_businesses(session.draft.clone()).remove(0);
    invalid.more.intangibles[1].source.obtained =
        IntangibleObtainedPage::Renewal { previous_intangible_id: Some(activity) };
    let before = session.draft.clone();
    let reply = save_existing_business(&mut session, activity, invalid);
    assert!(matches!(reply, Reply::Response(P::Problem { .. })));
    assert_eq!(session.draft, before);
    let mut reordered = reopened_businesses(session.draft.clone()).remove(0);
    reordered.more.intangibles.swap(0, 1);
    accepted(&save_existing_business(&mut session, activity, reordered));
    assert_eq!(stored_business(&session).1.intangibles[1].id, id);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
    let mut page = reopened_businesses(session.draft.clone()).remove(0);
    page.more.intangibles[1].source.lease_status = IntangibleLeaseStatus::RecordsUnavailable;
    accepted(&save_existing_business(&mut session, activity, page));
    let saved = session.draft.clone();
    let mut notes = vec![];
    let working = crate::unknowns::supported(&saved, &mut notes).unwrap();
    let entry = working.income.iter().find(|entry| entry.id == activity).unwrap();
    let FederalIncome::Business { data } = &entry.income else { panic!() };
    assert_eq!(data.intangibles.len(), 1);
    assert_eq!(data.intangibles[0].id, renewal_id);
    assert_eq!(
        data.intangibles[0].source.obtained,
        IntangibleObtained::Renewal { previous_intangible_id: Some(id) }
    );
    supported(&saved);
    assert_eq!(restart(saved.clone()).draft, saved);
    assert_eq!(session.draft, saved);
}

#[test]
fn section197_benefit_boundaries_warn_once_per_source_and_keep_known_income() {
    let original = intangible_business(20250701, 20200101);
    let mut baseline = original.clone();
    let FederalIncome::Business { data } = &mut baseline.income[1].income else { panic!() };
    data.intangibles.clear();
    let baseline = supported(&baseline);
    for case in 0..10 {
        let mut draft = original.clone();
        let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
        let record = &mut data.intangibles[0];
        match case {
            0 => record.source.kind = IntangibleKind::Software,
            1 => record.source.first_existed = None,
            2 => record.source.lease_status = IntangibleLeaseStatus::Leased,
            3 => record.source.lease_status = IntangibleLeaseStatus::RecordsUnavailable,
            4 => record.prior_claimed.regular = SourceCents::Unavailable,
            5 => record.use_end = IntangibleUseEnd::RecordsUnavailable,
            6 => record.other_events = None,
            7 | 8 => {
                record.use_end = IntangibleUseEnd::Ended {
                    record: AmortizationEnd {
                        date: 20260401,
                        how: if case == 7 {
                            DisposalKind::Scrapped
                        } else {
                            DisposalKind::PersonalUse
                        },
                    },
                }
            }
            _ => record.source.obtained = IntangibleObtained::Gift,
        }
        let accepted = draft.clone();
        let result = supported(&draft);
        let warnings: Vec<_> = result
            .review_notes
            .iter()
            .filter(|note| note.display.key == "WARN_INTANGIBLE_BENEFIT_LEFT_OUT")
            .collect();
        assert_eq!(warnings.len(), 1, "case {case}");
        assert!(warnings[0].context.is_some());
        assert!(!result.financial_source_scope_complete);
        assert_eq!(value(&result, "agi"), value(&baseline, "agi"), "case {case}");
        assert_eq!(draft, accepted);
    }
}

#[test]
fn section197_actual_mandatory_tax_precedes_an_uncomputed_benefit() {
    for exchange in [false, true] {
        let mut draft = intangible_business(20250701, 20200101);
        let FederalIncome::Business { data } = &mut draft.income[1].income else { panic!() };
        let record = &mut data.intangibles[0];
        record.source.kind = IntangibleKind::Software;
        record.other_events = None;
        let key = if exchange {
            record.use_end = IntangibleUseEnd::RecordsUnavailable;
            record.other_events = Some(vec![IntangibleEventKind::Exchange]);
            "INTANGIBLE_RECORDED_EVENTS"
        } else {
            record.use_end = IntangibleUseEnd::Ended {
                record: AmortizationEnd {
                    date: 20260401,
                    how: DisposalKind::Sold {
                        proceeds: 2_000_000,
                        selling_expenses: 0,
                        payment: SalePayment::InFull,
                        buyer: Buyer::Unrelated,
                        livestock: crate::SoldLivestock::No,
                    },
                },
            };
            "INTANGIBLE_DISPOSITION_GROUP"
        };
        assert_eq!(record.boundary(), Some(key));
        let result = estimate(&draft);
        assert_eq!(result.state, ResultState::Unsupported, "{result:?}");
        assert!(result.issues.iter().any(|issue| issue.display.key == key), "{result:?}");
        assert!(
            result.tax.is_none()
                && result.refund.is_none()
                && result
                    .lines
                    .iter()
                    .all(|line| line.id == crate::federal_rules::FORM_8615_UNEARNED)
        );
    }
}

#[test]
fn section197_projection_requires_facts_without_current_year_prior_defaults() {
    let original = ordinary_intangible(1_800_000, 20250701, 20200101, 60_000);
    let mut page = BusinessIntangiblePage::from(original.clone());
    assert_eq!(page.prior_claimed.regular, Some(SourceCents::Reported { amount: 60_000 }));
    assert_eq!(page.prior_claimed.amt, Some(SourceCents::Reported { amount: 60_000 }));
    page.prior_claimed.regular = Some(SourceCents::NotReported);
    page.prior_claimed.amt = None;
    let corrected = page.record(Some(&original));
    assert_eq!(corrected.prior_claimed.regular, SourceCents::NotReported);
    assert_eq!(corrected.prior_claimed.amt, SourceCents::Unavailable);
    let reopened = BusinessIntangiblePage::from(corrected);
    assert_eq!(reopened.prior_claimed.regular, Some(SourceCents::NotReported));
    assert_eq!(reopened.prior_claimed.amt, None);
    page.acquired = 20260701;
    page.source.kind = IntangibleKind::Trademark;
    page.source.obtained = IntangibleObtainedPage::Created;
    page.source.first_existed = None;
    page.prior_claimed = IntangiblePriorClaimedPage { regular: None, amt: None };
    let projected = page.record(Some(&original));
    assert_eq!(projected.source.first_existed, original.source.first_existed);
    assert_eq!(projected.prior_claimed, original.prior_claimed);
    let (origin, prior) = BusinessIntangiblePage::choices(
        &page.source.kind,
        &page.source.obtained.clone().into(),
        &page.source.context,
        page.acquired,
        page.business_started,
    );
    assert!(origin.is_empty() && prior.is_empty());
    let new_current = page.record(None);
    assert_eq!(new_current.prior_claimed.regular, SourceCents::Unavailable);
    assert_eq!(new_current.prior_claimed.amt, SourceCents::Unavailable);
    assert_eq!(new_current.boundary(), None);
}

#[test]
fn section197_source_options_do_not_require_the_answers_they_reveal() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let reply = send(
        &mut session,
        P::IntangibleSourceOptions {
            user: TEST_USER,
            kind: IntangibleKind::Goodwill,
            obtained: IntangibleObtained::Purchased,
            context: IntangibleAcquisitionContext::EntireOperatingBusinessAssets,
            acquired: 20250701,
            business_started: 20200101,
        },
    );
    let Reply::Response(P::IntangibleSourceChoices { origin_choices, prior_choices }) = reply
    else {
        panic!()
    };
    assert_eq!(origin_choices, vec![0, 1, 2]);
    assert_eq!(prior_choices, vec![1, 2]);
}

#[test]
fn section197_page_acquisition_views_preserve_every_native_answer() {
    for native in [
        IntangibleObtained::Purchased,
        IntangibleObtained::Created,
        IntangibleObtained::ContractForExistingRight,
        IntangibleObtained::Renewal { previous_intangible_id: None },
        IntangibleObtained::Renewal { previous_intangible_id: Some(71) },
        IntangibleObtained::Gift,
        IntangibleObtained::Inheritance,
        IntangibleObtained::EntityContribution,
        IntangibleObtained::EntityDistribution,
        IntangibleObtained::Exchange,
    ] {
        let page = IntangibleObtainedPage::from(native.clone());
        assert_eq!(IntangibleObtained::from(page), native);
    }
    let draft = intangible_business(20260701, 20200101);
    let FederalIncome::Business { data } = &draft.income[1].income else { panic!() };
    assert!(crate::amortization_records::predecessor_choices(data).is_empty());
}
