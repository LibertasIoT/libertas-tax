use super::*;
use crate::farm_insurance_paper::{self as crop, CropPaperClass, CropPaperFacts};
use crate::tests::income::information_returns::{misc, misc_form};
use alloc::format;

fn linked(rental: bool, amount: i64) -> (FederalDraft, i64, i64) {
    let mut draft = complete(FederalStatus::Single);
    add(
        &mut draft,
        income(if rental {
            FederalIncome::FarmRental { data: farm_rental_data(0, 0) }
        } else {
            raised_farm(0, 0)
        }),
    );
    let activity = draft.income.last().unwrap().id;
    add(
        &mut draft,
        income(misc(FederalForm1099Misc {
            crop_insurance: amount,
            withholding: 1_200,
            ..misc_form()
        })),
    );
    let source = crate::related_forms::catalog_for(&draft, activity)
        .into_iter()
        .find(|c| c.kind == crate::related_forms::DocumentKind::Form1099MiscCropInsurance)
        .unwrap()
        .id;
    claim_related_component(&mut draft, activity, source);
    (draft, activity, source)
}

#[test]
fn linked_crop_proceeds_count_once_before_questions_and_additional_proceeds_are_separate() {
    for rental in [false, true] {
        let (mut draft, activity, _) = linked(rental, 220_000);
        let claims = crate::related_forms::effective_claimants(&draft);
        assert_eq!(crop::resolved(&draft, activity, &claims).unwrap().income(true), 220_000);
        assert_eq!(crop::income_delta(&draft, activity, &claims), 220_000);
        let result = supported(&draft);
        assert_eq!(
            result.issues.iter().map(|issue| issue.display.key.as_str()).collect::<Vec<_>>(),
            vec!["CROP_PAPER_FACTS_NEEDED"],
        );
        let prefix = format!("farm_{activity}");
        assert_eq!(value(&result, &format!("{prefix}_crop_insurance_before_deferral")), 2_200);
        assert_eq!(return_line(&result, "25b"), 1_200);
        assert!(result.issues.iter().any(|i| i.display.key == "CROP_PAPER_FACTS_NEEDED"));
        let farm =
            crop::farm_mut(draft.income.iter_mut().find(|e| e.id == activity).unwrap()).unwrap();
        install_crop_records(
            farm,
            crop_records(vec![CropPayment::Other { amount: 30_000 }], 0, false, 0),
        );
        assert_eq!(crop::resolved(&draft, activity, &claims).unwrap().income(true), 250_000);
        assert_eq!(crop::income_delta(&draft, activity, &claims), 220_000);
        assert!(!crop::missing(&draft, activity, &claims));
        let result = supported(&draft);
        assert_eq!(value(&result, &format!("{prefix}_crop_insurance_taxable")), 2_500);
        assert_eq!(return_line(&result, "25b"), 1_200);
        assert_eq!(restart(draft.clone()).draft, draft);
    }
}

#[test]
fn paper_deferral_asks_only_missing_source_facts_and_normal_practice() {
    let (draft, activity, source) = linked(false, 220_000);
    let mut session = Interview::new(TEST_USER, draft);
    let opened = {
        let request = P::OpenCropInsuranceFacts {
            user: TEST_USER,
            cookie: session.draft.cookie.clone(),
            revision: session.draft.revision,
            activity,
        };
        send(&mut session, request)
    };
    let Reply::Response(P::CropInsuranceFactsPage {
        accepted: None, papers, allowed_actions, ..
    }) = opened
    else {
        panic!("expected unanswered crop timing")
    };
    assert_eq!(papers.len(), 1);
    assert_eq!(papers[0].summary.amount, Some(220_000));
    assert!(
        !allowed_actions.contains(&(libertas_macros::variant_index!(P::OpenCropPaperFacts) as i32))
    );
    assert!(
        !allowed_actions.contains(&(libertas_macros::variant_index!(P::OpenCropPractice) as i32))
    );
    let included = {
        let request = P::SaveCropInsuranceTiming {
            user: TEST_USER,
            cookie: session.draft.cookie.clone(),
            revision: session.draft.revision,
            activity,
            timing: CropPaperTiming::Include,
        };
        send(&mut session, request)
    };
    let Reply::Response(P::CropInsuranceFactsPage { papers, crops, allowed_actions, .. }) =
        included
    else {
        panic!("expected included crop paper")
    };
    assert_eq!(papers.len(), 1);
    assert!(crops.is_empty());
    assert!(
        !allowed_actions.contains(&(libertas_macros::variant_index!(P::OpenCropPaperFacts) as i32))
    );
    let reply = {
        let request = P::SaveCropInsuranceTiming {
            user: TEST_USER,
            cookie: session.draft.cookie.clone(),
            revision: session.draft.revision,
            activity,
            timing: CropPaperTiming::Defer,
        };
        send(&mut session, request)
    };
    assert!(
        matches!(reply,Reply::Response(P::CropInsuranceFactsPage {papers,..}) if papers.len()==1)
    );
    let opened = {
        let request = P::OpenCropPaperFacts {
            user: TEST_USER,
            cookie: session.draft.cookie.clone(),
            revision: session.draft.revision,
            activity,
            source,
        };
        send(&mut session, request)
    };
    assert!(matches!(opened, Reply::Response(P::BeginCropPaperFacts { accepted: None, .. })));
    let summary = crate::related_forms::catalog_for(&session.draft, activity)
        .into_iter()
        .find(|p| p.id == source)
        .unwrap()
        .summary;
    let reply = {
        let request = P::SaveCropPaperFacts {
            user: TEST_USER,
            cookie: session.draft.cookie.clone(),
            revision: session.draft.revision,
            activity,
            source,
            summary,
            classification: CropPaperClass::OneCrop { crop: "Wheat".into(), year: 2026 },
        };
        send(&mut session, request)
    };
    assert!(
        matches!(reply,Reply::Response(P::CropInsuranceFactsPage {crops,..}) if crops.len()==1)
    );
    accepted(&answer_crop_practice(&mut session, "Wheat", 750_000));
    let claims = crate::related_forms::effective_claimants(&session.draft);
    assert!(!crop::missing(&session.draft, activity, &claims));
    assert_eq!(crop::resolved(&session.draft, activity, &claims).unwrap().income(true), 0);
    assert_eq!(crop::resolved(&session.draft, activity, &claims).unwrap().current_total(), 220_000);
    assert_eq!(return_line(&supported(&session.draft), "25b"), 1_200);
    assert_eq!(restart(session.draft.clone()).draft, session.draft);
}

#[test]
fn whole_crop_facts_follow_amount_corrections_and_unlink_prunes_source_facts() {
    let (mut draft, activity, source) = linked(false, 220_000);
    let farm = crop::farm_mut(draft.income.iter_mut().find(|e| e.id == activity).unwrap()).unwrap();
    farm.insurance_papers.timing = Some(CropPaperTiming::Defer);
    farm.insurance_papers.papers.push(CropPaperFacts {
        source,
        classification: CropPaperClass::OneCrop { crop: "Grain".into(), year: 2026 },
    });
    farm.insurance_papers.practice = crop_practice();
    let before = draft.clone();
    let FederalIncome::Misc1099 { data } = &mut draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.form.crop_insurance = Some(230_001);
    crop::follow_sources(&before, &mut draft);
    let claims = crate::related_forms::effective_claimants(&draft);
    assert_eq!(crop::resolved(&draft, activity, &claims).unwrap().current_total(), 230_001);
    assert_eq!(crop::resolved(&draft, activity, &claims).unwrap().income(true), 0);
    // With two eligible farms, removing the explicit link leaves no automatic receiver.
    add(&mut draft, income(raised_farm(0, 0)));
    let before = draft.clone();
    crop::farm_mut(draft.income.iter_mut().find(|e| e.id == activity).unwrap())
        .unwrap()
        .related
        .clear();
    crop::follow_sources(&before, &mut draft);
    let claims = crate::related_forms::effective_claimants(&draft);
    assert_eq!(crop::income_delta(&draft, activity, &claims), 0);
    let farm = crate::farm::sources(&draft).next().unwrap().farm;
    assert!(farm.insurance_papers.papers.is_empty());
    assert_eq!(farm.insurance_papers.timing, None);
}

#[test]
fn foreign_owner_paper_and_ordinary_business_claims_never_become_farm_insurance() {
    let (mut draft, activity, _) = linked(false, 220_000);
    let claims = crate::related_forms::effective_claimants(&draft);
    draft.income.last_mut().unwrap().owner = crate::DocumentOwner::Spouses;
    assert_eq!(crop::income_delta(&draft, activity, &claims), 0);
    assert_eq!(crop::income_delta(&draft, 9_999, &claims), 0);
    assert!(
        CropPaperClass::Mixed {
            portions: vec![CropLoss { crop: "Wheat".into(), year: 2026, amount: 220_000 }]
        }
        .payments(220_000)
        .is_none()
    );
}

#[test]
fn rental_paper_insurance_uses_one_existing_community_fraction_with_odd_cents() {
    let (mut draft, activity, _) = linked(true, 220_001);
    let separate = complete(FederalStatus::Separate);
    draft.setup = separate.setup;
    draft.setup.as_mut().unwrap().community_property = community_arizona();
    draft.income.iter_mut().find(|e| e.id == activity).unwrap().community =
        CommunityShare::Community;
    let claims = crate::related_forms::effective_claimants(&draft);
    let reported = crop::report_value(&draft, activity, &claims).unwrap();
    assert_eq!(reported.current_total(), 110_001);
    assert_eq!(reported.income(true), 110_001);
    assert_eq!(crop::income_delta(&draft, activity, &claims), 220_001);
}

#[test]
fn paper_deferral_needs_practice_only_for_current_year_physical_crops() {
    for classification in [
        CropPaperClass::Nondeferrable,
        CropPaperClass::OneCrop { crop: "Prior damaged crop".into(), year: 2025 },
    ] {
        let (mut draft, activity, source) = linked(false, 220_000);
        let farm =
            crop::farm_mut(draft.income.iter_mut().find(|e| e.id == activity).unwrap()).unwrap();
        farm.insurance_papers.timing = Some(CropPaperTiming::Defer);
        farm.insurance_papers.papers = vec![CropPaperFacts { source, classification }];
        let claims = crate::related_forms::effective_claimants(&draft);
        assert!(crop::crop_names(&draft, activity, &claims).is_empty());
        assert!(!crop::missing(&draft, activity, &claims));
        let insurance = crop::resolved(&draft, activity, &claims).unwrap();
        assert!(insurance.source_issue().is_none());
        assert_eq!(insurance.income(true), 220_000);
        let result = supported(&draft);
        assert!(noted(&result, "WARN_CROP_PRACTICE_DEFERRAL_LEFT_OUT"));
        assert!(!result.issues.iter().any(|i| i.display.key == "CROP_PAPER_FACTS_NEEDED"));
    }
    let insurance = FarmInsurance::Payments {
        current: vec![
            CropPayment::Physical { crop: "Earlier damage".into(), amount: 100_000, year: 2025 },
            CropPayment::Physical { crop: "Current damage".into(), amount: 120_000, year: 2026 },
        ],
        previous_deferred: 0,
        withholding: 0,
        election: CropDeferral::Defer {
            practice: vec![FarmCropPractice {
                crop: "Current damage".into(),
                normal_total: 1_000_000,
                normal_following: 750_000,
            }],
        },
    };
    assert!(insurance.source_issue().is_none());
    assert_eq!(insurance.income(true), 100_000);
}

#[test]
fn accrual_crop_paper_details_show_printed_amount_without_cash_timing_questions() {
    let (mut draft, activity, _) = linked(false, 220_000);
    let farm = crop::farm_mut(draft.income.iter_mut().find(|entry| entry.id == activity).unwrap())
        .unwrap();
    farm.accounting = FarmAccounting::Accrual {
        sales: 0,
        opening: 0,
        closing: 0,
        purchases: 0,
        custom_hire: 0,
        program: 0,
        other: 0,
    };
    let mut session = Interview::new(TEST_USER, draft);
    let request = P::OpenCropInsuranceFacts {
        user: TEST_USER,
        cookie: session.draft.cookie.clone(),
        revision: session.draft.revision,
        activity,
    };
    let Reply::Response(P::CropInsuranceFactsPage { papers, crops, allowed_actions, .. }) =
        send(&mut session, request)
    else {
        panic!("expected accrual paper details")
    };
    assert_eq!(papers.len(), 1);
    assert_eq!(papers[0].summary.amount, Some(220_000));
    assert!(crops.is_empty());
    assert_eq!(
        allowed_actions,
        vec![libertas_macros::variant_index!(P::CloseCropInsuranceFacts) as i32]
    );
    let claims = crate::related_forms::effective_claimants(&session.draft);
    assert!(!crop::missing(&session.draft, activity, &claims));
    assert_eq!(crop::resolved(&session.draft, activity, &claims).unwrap().income(false), 220_000);
}

#[test]
fn changed_paper_crops_prune_only_orphaned_normal_practice() {
    let (mut draft, activity, source) = linked(false, 220_000);
    let classifications = CropPaperClass::SeveralCrops {
        portions: vec![
            CropLoss { crop: "Wheat".into(), year: 2026, amount: 120_000 },
            CropLoss { crop: "Corn".into(), year: 2026, amount: 100_000 },
        ],
    };
    let farm = crop::farm_mut(draft.income.iter_mut().find(|entry| entry.id == activity).unwrap())
        .unwrap();
    farm.insurance = FarmInsuranceRecords::Payments {
        current: vec![CropPayment::Physical { crop: "Barley".into(), year: 2026, amount: 10_000 }],
        previous_deferred: 0,
        withholding: 0,
    };
    farm.insurance_papers.timing = Some(CropPaperTiming::Defer);
    farm.insurance_papers.papers =
        vec![CropPaperFacts { source, classification: classifications.clone() }];
    farm.insurance_papers.practice = ["Wheat", "Corn", "Barley"]
        .into_iter()
        .map(|crop| FarmCropPractice {
            crop: crop.into(),
            normal_total: 1_000_000,
            normal_following: 750_000,
        })
        .collect();
    let before = draft.clone();
    crop::farm_mut(draft.income.iter_mut().find(|entry| entry.id == activity).unwrap())
        .unwrap()
        .insurance_papers
        .papers[0]
        .classification = CropPaperClass::OneCrop { crop: "Wheat".into(), year: 2026 };
    crop::follow_sources(&before, &mut draft);
    let farm = crate::farm::sources(&draft).find(|farm| farm.entry.id == activity).unwrap().farm;
    assert_eq!(
        farm.insurance_papers.practice.iter().map(|row| row.crop.as_str()).collect::<Vec<_>>(),
        vec!["Wheat", "Barley"]
    );
    let before = draft.clone();
    crop::farm_mut(draft.income.iter_mut().find(|entry| entry.id == activity).unwrap())
        .unwrap()
        .insurance_papers
        .papers[0]
        .classification = classifications;
    crop::follow_sources(&before, &mut draft);
    let claims = crate::related_forms::effective_claimants(&draft);
    assert!(
        crop::missing(&draft, activity, &claims),
        "a newly identified crop needs its actual practice answer"
    );
}

#[test]
fn farm_resave_preserves_unknown_or_explicit_crop_timing_without_answering_it() {
    use crate::tests::cross::interview::{cash_farm, document};
    use crate::tests::support::client::income_pages::open_topic;
    use crate::tests::support::client::income_pages::save_topic;
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let mut farm = cash_farm(0, 0);
    farm.insurance = CropInsurance::Payments {
        physical: vec![],
        price_only: 30_000,
        from_prior_year: 0,
        withholding: 0,
    };
    let request = save_topic(
        &mut session,
        TaxSection::Farming,
        vec![document(0, IncomeSource::Record { value: crate::IncomeRecord::Farm(farm) })],
    );
    accepted(&send(&mut session, request));
    let activity = crate::farm::sources(&session.draft).next().unwrap().entry.id;
    let request = open_topic(&mut session, TaxSection::Farming);
    accepted(&send(&mut session, request));
    let farm = crate::farm::sources(&session.draft)
        .find(|source| source.entry.id == activity)
        .unwrap()
        .farm;
    assert_eq!(farm.insurance_papers.timing, None);
    let claims = crate::related_forms::effective_claimants(&session.draft);
    assert!(crop::missing(&session.draft, activity, &claims));
    accepted(&answer_crop_timing(&mut session, CropPaperTiming::Include));
    let request = open_topic(&mut session, TaxSection::Farming);
    accepted(&send(&mut session, request));
    let farm = crate::farm::sources(&session.draft)
        .find(|source| source.entry.id == activity)
        .unwrap()
        .farm;
    assert_eq!(farm.insurance_papers.timing, Some(CropPaperTiming::Include));
    assert!(!crop::missing(&session.draft, activity, &claims));
}

#[test]
fn linked_crop_receipts_supply_the_farms_half_se_gross_allocation_weight() {
    let (mut draft, activity, _) = linked(false, 220_000);
    crop::farm_mut(draft.income.iter_mut().find(|entry| entry.id == activity).unwrap())
        .unwrap()
        .insurance_papers
        .timing = Some(CropPaperTiming::Include);
    add(&mut draft, income(raised_farm(4_400, 0)));
    let other = draft.income.last().unwrap().id;
    let before = draft.clone();
    let allocation = crate::partnership_qbi::half_se_by_gross(&draft, [300, 0]).unwrap();
    assert_eq!(allocation.sources[&activity], 100);
    assert_eq!(allocation.sources[&other], 200);
    assert_eq!(allocation.sources.values().sum::<i64>(), 300);
    assert_eq!(draft, before);
    supported(&draft);
}
