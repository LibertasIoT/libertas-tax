// 2026-10-16 D05: record-based confirmations are explicit Yes/No; negative fixtures test No.
use super::support::assert::{accepted, rejected_with};
use super::support::client::unit50_interview_native::NativeInterviewCase;
use super::*;
// Shared native producers and page traversal; keep established topic-call paths.
use super::support::builders::native_income::{document, native_w2 as w2, owned};
use super::support::client::income_pages::{open_topic, page_documents, save_topic};
use crate::tests::cross::interview::{
    from_review, k1_1065, k1_1120s, refusal, rental_page, review_state, save_whole_return,
    saved_setup,
};

// Form 8379 (T3-8379): the "Anything else?" injured spouse item opens its
// page, which saves onto FederalDraft.injured_spouse and reopens unchanged.
fn injured_spouse_editor(reply: Reply) -> (InjuredSpouse, i32, i32, bool) {
    match crate::tests::support::client::native_editor(reply, &[]) {
        Reply::Edit(P::SaveInjuredSpouse { value, page, previous, review, .. }) => {
            (value, page, previous, review)
        }
        other => panic!("expected the injured spouse page, got {other:?}"),
    }
}
fn save_injured_spouse(s: &mut Interview, value: InjuredSpouse) -> Reply {
    let d = s.draft.clone();
    send(
        s,
        P::SaveInjuredSpouse {
            user: TEST_USER,
            cookie: d.cookie.clone(),
            revision: d.revision,
            page: TaxSection::InjuredSpouse as i32,
            previous: TaxSection::Payments as i32,
            review: true,
            estimate: vec![],
            warnings: vec![],
            progress: vec![],
            value,
        },
    )
}

#[test]
fn injured_spouse_item_opens_its_page_and_round_trips() {
    let mut d = complete(FederalStatus::Joint);
    d.income.push(wage(2, Owner::Spouse, 2_000_000, 50_000));
    d.next_id = 3;
    d.payments.as_mut().unwrap().estimated = Some(100_000);
    d.payments.as_mut().unwrap().estimated_payments = vec![crate::DatedPayment {
        date: 20260415,
        amount: 100_000,
        payer: PaymentPayer::Joint,
        refund_property: None,
    }];
    let mut s = Interview::new(TEST_USER, d);
    // Ticking the checklist's item opens the page with its defaults and the
    // joint estimated payments ($1,000) for the client rule; Back opens the
    // last shown section.
    let (value, page, previous, _) =
        injured_spouse_editor(save_whole_return(&mut s, WholeReturnTopics::INJURED_SPOUSE.0));
    assert_eq!(page, TaxSection::InjuredSpouse as i32);
    assert_eq!(previous, TaxSection::Payments as i32);
    assert_eq!(
        (value.injured, value.debt, value.obligated),
        (Owner::Taxpayer, OffsetDebt::FederalNontax, false)
    );
    assert_eq!(value.community, InjuredCommunity::No);
    assert_eq!(value.estimated, EstimatedSplit::IrsFormula);
    assert_eq!(value.joint_estimated, Some(100_000));
    assert!(value.claimants.is_empty() && value.allocations.is_empty());
    assert_eq!(value.states.iter().map(|s| s.id).collect::<Vec<_>>(), (1..=9).collect::<Vec<_>>());
    // Independently: wages100,000/20,000, basic deduction16,100 each;
    // MFS taxes13,176/393 allocate1,000 joint payments as971/29.
    // Joint tax10,043 exceeds8,000 payments, so the protected refund is0.
    assert_eq!(
        value.known.iter().map(|r| (r.joint, r.injured, r.other)).collect::<Vec<_>>(),
        vec![
            (12_000_000, 10_000_000, 2_000_000),
            (0, 0, 0),
            (0, 0, 0),
            (3_220_000, 1_610_000, 1_610_000),
            (0, 0, 0),
            (0, 0, 0),
            (0, 0, 0),
            (700_000, 650_000, 50_000),
            (100_000, 97_100, 2_900),
        ]
    );
    assert_eq!(value.estimated_share, Some(0));
    // Until it is saved, the review lists it.
    let (items, ..) = review_state(&mut s);
    assert!(
        items.iter().any(|i| i.section == TaxSection::InjuredSpouse
            && status_text(&i.status) == "Not answered yet"),
        "{items:?}"
    );
    let mut answered = value.clone();
    answered.debt = OffsetDebt::ChildSupport;
    answered.estimated = EstimatedSplit::Agreed { injured: 60_000 };
    // The derived fields are ignored on save.
    answered.joint_estimated = Some(1);
    answered.estimated_share = Some(1);
    assert!(matches!(
        save_injured_spouse(&mut s, answered.clone()),
        Reply::Response(P::Review { .. })
    ));
    let stored = s.draft.injured_spouse.clone().unwrap();
    assert_eq!(stored.debt, OffsetDebt::ChildSupport);
    assert_eq!(stored.estimated, EstimatedSplit::Agreed { injured: 60_000 });
    let r = supported(&s.draft);
    let _share =
        r.lines.iter().find(|l| l.id == "form_8379_injured_refund_estimate").unwrap().amount;
    // A restart reopens the same answers with the share derived again.
    let mut s = restart(s.draft.clone());
    let (reopened, ..) = injured_spouse_editor(from_review(&mut s, TaxSection::InjuredSpouse));
    assert_eq!(reopened.debt, OffsetDebt::ChildSupport);
    assert_eq!(reopened.estimated, EstimatedSplit::Agreed { injured: 60_000 });
    assert_eq!(reopened.estimated_share, Some(0));
    assert_eq!(
        reopened.known.last().map(|r| (r.joint, r.injured, r.other)),
        Some((100_000, 60_000, 40_000))
    );
    // "Your answers" shows the page.
    let (.., answers) = review_state(&mut s);
    assert!(answers.iter().any(|a| a.section == TaxSection::InjuredSpouse
        && status_text(&a.item) == "Injured spouse allocation (Form 8379)"
        && status_text(&a.status) == "Entered"));
    // Unticking the item drops the answers.
    save_whole_return(&mut s, 0);
    assert_eq!(s.draft.injured_spouse, None);
    let message = refusal(from_review(&mut s, TaxSection::InjuredSpouse));
    assert!(message.contains("not available"), "{message}");
}

#[test]
fn injured_spouse_item_is_offered_only_on_a_joint_return() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    assert!(
        refusal(save_whole_return(&mut s, WholeReturnTopics::INJURED_SPOUSE.0))
            .contains("joint return")
    );
    // Leaving a joint return erases the item and its answers.
    let mut d = complete(FederalStatus::Joint);
    d.checklist.whole_return = WholeReturnTopics::INJURED_SPOUSE.0;
    d.injured_spouse = Some(InjuredSpouseAllocation::defaults());
    let mut single = d.clone();
    single.setup = complete(FederalStatus::Single).setup;
    crate::interview::prune(&mut single);
    assert_eq!(single.checklist.whole_return, 0);
    assert_eq!(single.injured_spouse, None);
}

// INJURED_SPOUSE_ESTIMATED_ABOVE_PAYMENTS on both sides: the agreed part
// may equal the $1,000 of joint payments but not exceed it by a cent.
#[test]
fn injured_spouse_agreed_payments_are_checked_on_both_sides() {
    let mut d = complete(FederalStatus::Joint);
    d.payments.as_mut().unwrap().estimated = Some(100_000);
    d.payments.as_mut().unwrap().estimated_payments = vec![crate::DatedPayment {
        date: 20260415,
        amount: 100_000,
        payer: PaymentPayer::Joint,
        refund_property: None,
    }];
    d.checklist.whole_return = WholeReturnTopics::INJURED_SPOUSE.0;
    let mut s = Interview::new(TEST_USER, d);
    let (mut value, ..) = injured_spouse_editor(from_review(&mut s, TaxSection::InjuredSpouse));
    value.estimated = EstimatedSplit::Agreed { injured: 100_001 };
    rejected_with(
        save_injured_spouse(&mut s, value.clone()),
        "INJURED_SPOUSE_ESTIMATED_ABOVE_PAYMENTS",
    );
    assert_eq!(s.draft.injured_spouse, None);
    value.estimated = EstimatedSplit::Agreed { injured: 100_000 };
    accepted(&save_injured_spouse(&mut s, value));
    assert_eq!(
        s.draft.injured_spouse.as_ref().unwrap().estimated,
        EstimatedSplit::Agreed { injured: 100_000 }
    );
}
// Community property (Form 8958) on the Setup and income pages: the choice, the
// answer each row asks, and what a return that is not one offers.
#[test]
fn community_property_choice_sets_the_rows_answers_and_the_spouses_documents() {
    let base = complete(FederalStatus::Separate);
    let mut s = Interview::new(TEST_USER, base.clone());
    let choice = CommunityProperty::Yes {
        state: CommunityState::Texas,
        rule: CommunityRule::LivingApartAllYear,
    };
    saved_setup(
        &mut s,
        ReturnSetup {
            filing: Filing::MarriedSeparately {
                spouse_boxes: SeparateSpouseBoxes::NoneOfThese,
                community_property: choice,
            },
        },
    );
    assert_eq!(s.draft.setup.as_ref().unwrap().community_property, choice);
    assert_eq!(
        crate::interview::return_setup(s.draft.setup.as_ref().unwrap()).filing,
        Filing::MarriedSeparately {
            spouse_boxes: SeparateSpouseBoxes::NoneOfThese,
            community_property: choice
        }
    );
    assert_eq!(restart(s.draft.clone()).draft, s.draft);
    // The wages page offers the three answers and the spouse's documents.
    let page = open_topic(&mut s, TaxSection::Wages);
    let P::SaveIncomes { allowed_community, allowed_owners, .. } = &page else { unreachable!() };
    assert_eq!(
        allowed_community,
        &vec![
            libertas_macros::variant_index!(CommunityShare::Community) as i32,
            libertas_macros::variant_index!(CommunityShare::Separate) as i32,
            libertas_macros::variant_index!(CommunityShare::Partly) as i32,
        ]
    );
    assert!(allowed_owners.contains(&(Owner::Spouse as i32)));
    // The row's answer, with the spouse's own W-2 saved under "Your spouse".
    let mut mine = IncomeDocumentItem {
        investment_caps: None,
        community: CommunityShare::Community,
        ..document(0, IncomeDocument::W2(w2(3_000_000)))
    };
    mine.owner = Owner::Taxpayer;
    let theirs = IncomeDocumentItem {
        investment_caps: None,
        owner: Owner::Spouse,
        community: CommunityShare::Partly { percent: 40 },
        ..document(0, IncomeDocument::W2(w2(2_000_000)))
    };
    let request =
        save_topic(&s.draft.clone(), TaxSection::Wages, vec![mine.clone(), theirs.clone()]);
    accepted(&send(&mut s, request));
    assert_eq!(s.draft.income.len(), 2);
    assert!(s.draft.income.iter().any(|e| {
        e.owner == Owner::Spouse && e.community == CommunityShare::Partly { percent: 40 }
    }));
    // It reopens with the answers it was saved with.
    let reopened = open_topic(&mut s, TaxSection::Wages);
    assert!(page_documents(&reopened).iter().any(|item| {
        item.owner == Owner::Spouse && item.community == CommunityShare::Partly { percent: 40 }
    }));
    // The same checks run again on save.
    for (case, share, key, path) in [
        (
            "income-source-1329",
            CommunityShare::NotNeeded,
            "COMMUNITY_ANSWER_NOT_ALLOWED",
            "[documents/documents/0/community]",
        ),
        (
            "income-source-1330",
            CommunityShare::Partly { percent: 100 },
            "COMMUNITY_PERCENT_OUT_OF_RANGE",
            "[documents/documents/0/community/percent]",
        ),
    ] {
        let bad = IncomeDocumentItem { investment_caps: None, community: share, ..mine.clone() };
        let request = save_topic(&s.draft.clone(), TaxSection::Wages, vec![bad]);
        rejected_with(current_income_refusal(case, &mut s, request, key, path), key);
    }
    // A return that is not a community-property one asks nothing: the answer is
    // "Not needed", and the spouse's documents go with the choice.
    saved_setup(
        &mut s,
        ReturnSetup {
            filing: Filing::MarriedSeparately {
                spouse_boxes: SeparateSpouseBoxes::NoneOfThese,
                community_property: CommunityProperty::No,
            },
        },
    );
    assert!(s.draft.income.iter().all(|e| e.owner == Owner::Taxpayer));
    assert!(s.draft.income.iter().all(|e| e.community == CommunityShare::NotNeeded));
    let page = open_topic(&mut s, TaxSection::Wages);
    let P::SaveIncomes { allowed_community, .. } = &page else { unreachable!() };
    assert_eq!(
        allowed_community,
        &vec![libertas_macros::variant_index!(CommunityShare::NotNeeded) as i32]
    );
    let answered = IncomeDocumentItem {
        investment_caps: None,
        community: CommunityShare::Community,
        ..document(0, IncomeDocument::W2(w2(3_000_000)))
    };
    let request = save_topic(&s.draft.clone(), TaxSection::Wages, vec![answered]);
    rejected_with(
        current_income_refusal(
            "income-source-1351",
            &mut s,
            request,
            "COMMUNITY_ANSWER_NOT_ALLOWED",
            "[documents/documents/0/community]",
        ),
        "COMMUNITY_ANSWER_NOT_ALLOWED",
    );
}

// Changing the second child's separate-return claimant refreshes the complete
// editor. The actual joint pools are CTC$4,043 and ACTC$357, with $2,200
// total credit capacity per child. Forged hidden totals/caps cannot enlarge it.
#[test]
fn injured_spouse_mixed_child_pools_refresh_and_reject_forged_combined_cap() {
    let mut d = complete(FederalStatus::Joint);
    d.income = vec![
        wage(1, Owner::Taxpayer, 5_000_000, 800_000),
        wage(2, Owner::Spouse, 2_000_000, 100_000),
    ];
    d.dependents = vec![child(3), child(4)];
    d.next_id = 5;
    d.checklist.whole_return = WholeReturnTopics::INJURED_SPOUSE.0;
    let mut s = Interview::new(TEST_USER, d);
    let (mut editor, ..) = injured_spouse_editor(from_review(&mut s, TaxSection::InjuredSpouse));
    editor.claimants[1].claimant = Owner::Spouse;
    let before = s.draft.clone();
    // Explicit native fixture answers use the existing 1,843/357 case below;
    // an actual untouched Workflow editor remains unanswered until user input.
    let (mut refreshed, ..) = injured_spouse_editor(crate::tests::support::client::native_editor(
        save_injured_spouse(&mut s, editor),
        &[("ctc_odc_pool", 184_300), ("actc_pool", 35_700)],
    ));
    assert_eq!(s.draft, before);
    assert_eq!(refreshed.allocations.len(), 2);
    assert_eq!(
        refreshed.allocations.iter().find(|a| a.component == "ctc_odc_pool").unwrap().joint,
        404_300
    );
    assert_eq!(
        refreshed.allocations.iter().find(|a| a.component == "actc_pool").unwrap().joint,
        35_700
    );
    for a in &mut refreshed.allocations {
        a.injured = if a.component == "ctc_odc_pool" { 220_000 } else { 35_700 };
        a.joint = 999_999_999;
    }
    refreshed.child_credit_cap_injured = 999_999_999;
    refreshed.child_credit_cap_other = 999_999_999;
    refreshed.known_child_credit_injured = 0;
    refreshed.known_child_credit_other = 0;
    rejected_with(
        save_injured_spouse(&mut s, refreshed.clone()),
        "INJURED_SPOUSE_COMBINED_CHILD_CREDIT_ABOVE_CAP",
    );
    assert_eq!(s.draft, before);
    for a in &mut refreshed.allocations {
        if a.component == "ctc_odc_pool" {
            a.injured = 184_300;
        }
    }
    accepted(&save_injured_spouse(&mut s, refreshed));
    let r = supported(&s.draft);
    assert_eq!(value(&r, "form_8379_line_16_b"), 1_843);
    assert_eq!(value(&r, "form_8379_line_17_b"), 357);
    assert_eq!(value(&r, "form_8379_line_16_c") + value(&r, "form_8379_line_17_c"), 2_200);
    let accepted_draft = s.draft.clone();
    let (mut duplicate, ..) = injured_spouse_editor(from_review(&mut s, TaxSection::InjuredSpouse));
    duplicate.claimants[1].person = duplicate.claimants[0].person;
    rejected_with(save_injured_spouse(&mut s, duplicate), "INJURED_SPOUSE_CLAIMANTS_NOT_CURRENT");
    assert_eq!(s.draft, accepted_draft);
}

#[test]
fn unit21_k1_whole_sources_survive_save_restart_reopen_and_share_correction() {
    for scorp in [false, true] {
        let mut d = complete(FederalStatus::Separate);
        d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 600_000);
        d.income[0].community = CommunityShare::Community;
        d.setup.as_mut().unwrap().community_property =
            CommunityProperty::Yes { state: CommunityState::Arizona, rule: CommunityRule::Halves };
        let page = if scorp {
            let mut page = k1_1120s("Synthetic community corporation");
            page.ordinary = 4_000_000;
            page.participation = Participation::Over500Hours { hours: 600 };
            let K1ScorpQbi::Statement { income, .. } = &mut page.qbi else { unreachable!() };
            *income = 4_000_000;
            IncomeDocument::K1Scorp(page)
        } else {
            let mut page = k1_1065("Synthetic community partnership");
            page.ordinary = 4_000_000;
            page.interest = PartnerInterest::General {
                participation: Participation::Over500Hours { hours: 600 },
            };
            let PartnerQbi::Statement { income, .. } = &mut page.qbi else { unreachable!() };
            *income = 4_000_000;
            IncomeDocument::K1Partnership(page)
        };
        let mut item = owned(Owner::Spouse, 0, page.clone());
        item.community = CommunityShare::Community;
        let request = save_topic(&d, TaxSection::PassThrough, vec![item]);
        let mut session = Interview::new(TEST_USER, d);
        accepted(&send(&mut session, request));
        let stored = session.draft.clone();
        let mut session = restart(stored.clone());
        assert_eq!(session.draft, stored);
        let reopened = open_topic(&mut session, TaxSection::PassThrough);
        let mut documents = page_documents(&reopened).to_vec();
        assert_eq!(documents[0].document, page);
        assert_eq!(documents[0].owner, Owner::Spouse);
        assert_eq!(documents[0].community, CommunityShare::Community);
        assert_eq!(value(&supported(&session.draft), "agi"), 50_000);
        let revision = session.draft.revision;
        accepted(&send(&mut session, reopened));
        assert_eq!(session.draft.revision, revision);
        documents[0].community = CommunityShare::Partly { percent: 40 };
        let request = save_topic(&session.draft, TaxSection::PassThrough, documents);
        accepted(&send(&mut session, request));
        assert_eq!(value(&supported(&session.draft), "agi"), 38_000);
        let source =
            session.draft.income.iter().find(|entry| entry.owner == Owner::Spouse).unwrap();
        match &source.income {
            FederalIncome::Partnership { data } => assert_eq!(data.regular.ordinary, 4_000_000),
            FederalIncome::Scorp { data } => assert_eq!(data.ordinary, 4_000_000),
            _ => unreachable!(),
        }
        assert_eq!(restart(session.draft.clone()).draft, session.draft);
    }
}

#[test]
fn unit21_actual_client_rental_row_saves_reopens_and_preserves_whole_history() {
    extern crate std;
    let mut d = complete(FederalStatus::Separate);
    d.income[0] = wage(1, Owner::Taxpayer, 6_000_000, 600_000);
    d.income[0].community = CommunityShare::Community;
    d.setup.as_mut().unwrap().community_property =
        CommunityProperty::Yes { state: CommunityState::Arizona, rule: CommunityRule::Halves };
    let mut session = Interview::new(TEST_USER, d);
    let seed = open_topic(&mut session, TaxSection::Rentals);
    if NativeInterviewCase("TAX_UNIT21_CLIENT").required_calculation(&mut session) {
        return;
    }
    if let Ok(directory) = std::env::var("TAX_UNIT21_CLIENT_SEED_DIR") {
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(directory.join("seed.avro"), seed.to_avro()).unwrap();
        let mut page = rental_page(2_400_000);
        page.address = "Synthetic community rental".into();
        page.building.cost = 33_000_000;
        page.building.land = 5_500_000;
        page.building.in_service = 20200101;
        page.expenses.repairs = 1_800_000;
        page.more.suspended =
            SuspendedLoss { regular: 600_000, amt: 600_000, active_in_loss_year: true };
        let mut item = owned(Owner::Spouse, 0, IncomeDocument::Rental(page));
        item.community = CommunityShare::Community;
        let mut answers = seed;
        let P::SaveIncomes { documents, .. } = &mut answers else { unreachable!() };
        documents.documents = vec![item];
        std::fs::write(directory.join("answers.avro"), answers.to_avro()).unwrap();
        return;
    }
    let request = if let Ok(path) = std::env::var("TAX_UNIT21_CLIENT_PAYLOAD") {
        P::from_avro(
            &std::fs::read(path).expect("actual Studio editor-created community rental request"),
        )
        .unwrap()
    } else {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/unit21-community-rental-client.avro");
        crate::tests::support::client::captured_rental_request_before_nullable_risk(
            &std::fs::read(path).expect("original Studio editor-created community rental request"),
            402,
        )
    };
    accepted(&send(&mut session, request));
    let saved = session.draft.clone();
    let source = saved.income.iter().find(|entry| entry.owner == Owner::Spouse).unwrap();
    assert_eq!(source.community, CommunityShare::Community);
    let FederalIncome::Rental { data } = &source.income else { unreachable!() };
    assert_eq!(data.rent, 2_400_000);
    assert_eq!(data.history.regular, 600_000);
    assert_eq!(data.history.amt, 600_000);
    let r = supported(&saved);
    assert_eq!(value(&r, "agi"), 30_000);
    assert_eq!(value(&r, "rental_suspended_loss"), 5_000);
    assert!(noted(&r, "NOTE_COMMUNITY_CARRY_IS_THIS_RETURNS_SHARE"));
    let mut session = restart(saved.clone());
    assert_eq!(session.draft, saved);
    let reopened = open_topic(&mut session, TaxSection::Rentals);
    let documents = page_documents(&reopened);
    assert_eq!(documents[0].community, CommunityShare::Community);
    assert_eq!(documents[0].owner, Owner::Spouse);
    let IncomeDocument::Rental(page) = &documents[0].document else { unreachable!() };
    assert_eq!(page.rent, 2_400_000);
    assert_eq!(page.more.suspended.regular, 600_000);
    accepted(&send(&mut session, reopened));
    assert_eq!(session.draft, saved);
    let mut correction = open_topic(&mut session, TaxSection::Rentals);
    let P::SaveIncomes { documents, .. } = &mut correction else { unreachable!() };
    documents.documents[0].community = CommunityShare::Separate;
    accepted(&send(&mut session, correction));
    let corrected = supported(&session.draft);
    assert_eq!(value(&corrected, "agi"), 30_000);
    assert_eq!(value(&corrected, "rental_suspended_loss"), 0);
    assert!(noted(&corrected, "NOTE_COMMUNITY_ITEM_IS_THE_SPOUSES"));
    let FederalIncome::Rental { data } =
        &session.draft.income.iter().find(|entry| entry.owner == Owner::Spouse).unwrap().income
    else {
        unreachable!()
    };
    assert_eq!(data.history.regular, 600_000);
}
