// The last page before the review: it is required, an option plans its
// page, withdrawing removes only the plan, and document evidence stays
// separate from the person's explicit answers.
use super::*;
use crate::tests::cross::interview::{
    accepted, open_often_missed, save_often_missed, save_without_form,
};

#[test]
fn the_page_is_required_before_the_review_can_finish() {
    let mut d = complete(FederalStatus::Single);
    d.finished_sections.retain(|s| *s != TaxSection::OftenMissed);
    let mut s = Interview::new(TEST_USER, d);
    // Resume stops at the page, and the review names it as the finishing row.
    let reply = send(&mut s, P::OpenInterview { user: TEST_USER });
    let Reply::Response(P::BeginOftenMissed { .. }) = reply else {
        panic!("unanswered required choices must use workflow construction: {reply:?}")
    };
    assert!(!s.draft.finished_sections.contains(&TaxSection::OftenMissed));
    let Reply::Response(P::Review { remaining_items, allowed_actions, .. }) =
        send(&mut s, P::ReviewAccepted { user: TEST_USER })
    else {
        panic!("review expected")
    };
    assert!(remaining_items.iter().any(|i| i.section == TaxSection::OftenMissed));
    assert!(!allowed_actions.contains(&(libertas_macros::variant_index!(P::StartFinish) as i32)));
    // "None of these" answers it.
    accepted(&save_often_missed(&mut s, OftenMissed::default()));
    assert!(s.draft.finished_sections.contains(&TaxSection::OftenMissed));
}

#[test]
fn a_checked_option_plans_its_page_and_unchecking_removes_only_the_plan() {
    for (bit, topic) in [
        (MissedIncome::SCHOLARSHIP, TaxSection::OtherIncome),
        (MissedIncome::PRIVATE_LOAN_INTEREST, TaxSection::InterestDividends),
    ] {
        let mut draft = complete(FederalStatus::Single);
        if topic == TaxSection::InterestDividends {
            draft.income[0].income = savings_income(1_000, 0);
            draft.income[0].label = "Synthetic Bank".into();
        }
        let mut s = Interview::new(TEST_USER, draft);
        // Each option plans its record editor, which the walk opens again.
        let reply = save_often_missed(&mut s, OftenMissed { income: bit.0, ..Default::default() });
        match (&reply, topic) {
            (Reply::Edit(P::SaveWagesAnswers { page, .. }), TaxSection::Wages) => {
                assert_eq!(*page, TaxSection::Wages as i32);
            }
            (Reply::Edit(P::SaveIncomes { topic: actual, .. }), _) => {
                assert_eq!(*actual, topic);
            }
            _ => panic!("expected the planned topic editor, got {reply:?}"),
        }
        assert!(crate::interview::planned_sections(&s.draft).contains(&topic));
        assert!(!s.draft.finished_sections.contains(&topic));
        // An existing plan saved again does not reopen its visited topic.
        s.draft.finished_sections.push(topic);
        s.draft.finished_sections.push(TaxSection::Income);
        accepted(&save_often_missed(&mut s, OftenMissed { income: bit.0, ..Default::default() }));
        assert!(s.draft.finished_sections.contains(&topic));
        // Unchecking it withdraws the plan and removes nothing else.
        let incomes = s.draft.income.clone();
        accepted(&save_often_missed(&mut s, OftenMissed::default()));
        assert!(!crate::interview::planned_sections(&s.draft).contains(&topic));
        assert_eq!(s.draft.income, incomes);
    }
}

#[test]
fn direct_tip_answer_plans_wages_and_withdrawing_it_retains_records() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let answer = crate::IncomeWithoutFormAnswers { tips: true, ..Default::default() };
    let reply = save_without_form(&mut s, answer.clone());
    assert!(matches!(reply, Reply::Edit(P::SaveWagesAnswers { .. })), "{reply:?}");
    assert!(crate::interview::planned_sections(&s.draft).contains(&TaxSection::Wages));
    assert!(!s.draft.finished_sections.contains(&TaxSection::Wages));
    s.draft.finished_sections.push(TaxSection::Wages);
    s.draft.finished_sections.push(TaxSection::Income);
    accepted(&save_without_form(&mut s, answer));
    assert!(s.draft.finished_sections.contains(&TaxSection::Wages));
    let incomes = s.draft.income.clone();
    accepted(&save_without_form(&mut s, Default::default()));
    assert!(!crate::interview::planned_sections(&s.draft).contains(&TaxSection::Wages));
    assert_eq!(s.draft.income, incomes);
}

#[test]
fn an_option_plans_the_page_of_a_deduction_or_credit_it_names() {
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_often_missed(
        &mut s,
        OftenMissed {
            income: MissedIncome::DISALLOWED_CREDIT.0,
            benefits: MissedBenefits::EDUCATOR.0,
            ..Default::default()
        },
    ));
    let planned = crate::interview::planned_sections(&s.draft);
    assert!(planned.contains(&TaxSection::MoreCredits));
    assert!(planned.contains(&TaxSection::Educator));
}

#[test]
fn document_evidence_plans_topics_without_answering_or_erasing_the_persons_choices() {
    use crate::tests::support::client::save_often_missed_marketplace_source;
    let mut s = Interview::new(TEST_USER, complete(FederalStatus::Single));
    accepted(&save_often_missed_marketplace_source(&mut s, true));
    let P::SaveOftenMissed { value, .. } = open_often_missed(&mut s) else { unreachable!() };
    assert_eq!(value.income, 0);
    assert!(crate::interview::planned_sections(&s.draft).contains(&TaxSection::Marketplace));
    // A real answer remains accepted even while the document also plans it.
    accepted(&save_often_missed(
        &mut s,
        OftenMissed { income: MissedIncome::MARKETPLACE.0, ..Default::default() },
    ));
    assert_eq!(s.draft.often_missed.income, MissedIncome::MARKETPLACE.0);
    // Removing the document evidence cannot erase that answer.
    accepted(&save_often_missed_marketplace_source(&mut s, false));
    assert!(crate::interview::planned_sections(&s.draft).contains(&TaxSection::Marketplace));
    accepted(&save_often_missed(&mut s, OftenMissed::default()));
    assert!(!crate::interview::planned_sections(&s.draft).contains(&TaxSection::Marketplace));
}

#[test]
fn explicit_none_and_positive_answers_reopen_after_native_restart() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    for answer in [
        OftenMissed::default(),
        OftenMissed {
            income: MissedIncome::SCHOLARSHIP.0,
            unprepared: vec![Unprepared::AlternativeMinimumTax],
            benefits: 0,
        },
    ] {
        accepted(&save_often_missed(&mut session, answer.clone()));
        session = restart(session.draft.clone());
        let P::SaveOftenMissed { value, .. } = open_often_missed(&mut session) else {
            unreachable!()
        };
        assert_eq!(value, crate::federal_model::OftenMissedAnswers::from(answer));
    }
}

#[test]
fn empty_positive_branches_use_structured_atomic_save_validation() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    // The income question is a Bitflags list: its "None of these" (zero)
    // is an answer and saves (R4-11).
    let mut page = open_often_missed(&mut session);
    let P::SaveOftenMissed { value, .. } = &mut page else { unreachable!() };
    value.income = 0;
    accepted(&send(&mut session, page));
    assert_eq!(session.draft.often_missed.income, 0);
    let mut page = open_often_missed(&mut session);
    let P::SaveOftenMissed { value, .. } = &mut page else { unreachable!() };
    value.unprepared = Some(Vec::new());
    let before = session.draft.clone();
    let Reply::Response(P::Problem { message, field: field_path }) = send(&mut session, page)
    else {
        panic!("an empty selected branch requires an explicit None answer")
    };
    assert!(crate::tests::support::client::status_text(&message).contains("None of these"));
    assert_eq!(field_path.as_deref(), Some("[value/unprepared]"));
    assert_eq!(session.draft, before);
}

#[test]
fn the_often_missed_page_keeps_manual_topic_navigation_without_ai() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Single));
    let P::SaveOftenMissed { cookie, revision, page, navigation, all_topics, .. } =
        open_often_missed(&mut session)
    else {
        unreachable!()
    };
    assert!(
        crate::tests::support::client::status_text(&all_topics.label).contains("See all topics")
    );
    let before = session.draft.clone();
    let reply = send(
        &mut session,
        P::Navigate {
            user: TEST_USER,
            cookie,
            revision,
            from: page,
            section: TaxSection::MoreAdjustments.into(),
            navigation,
        },
    );
    assert!(matches!(reply, Reply::Edit(P::SaveMoreAdjustments { .. })), "{reply:?}");
    assert_eq!(session.draft, before);
}

// Beyond your documents asks what the person did or paid: a box a document
// checks (an HSA's 1099-SA, a distribution and not a contribution) is never
// pre-ticked. The note names the document, and only the person's own tick
// is saved and reopened.
#[test]
fn beyond_your_documents_never_pre_ticks_a_box_a_document_checks() {
    use crate::tests::cross::interview::{checklist_of, navigate, tick};
    let mut d = complete(FederalStatus::Single);
    add_hsa_distribution(&mut d, Owner::Taxpayer, 50_000, vec![], None, None);
    let mut s = Interview::new(TEST_USER, d);
    let open = |s: &mut Interview| {
        let request = navigate(&s.draft, TaxSection::Checklist);
        send(s, request)
    };
    let reply = open(&mut s);
    let Reply::Edit(P::SaveChecklist { from_documents, .. }) = &reply else {
        panic!("expected the checklist, got {reply:?}")
    };
    assert!(!from_documents.is_empty(), "the note names the 1099-SA");
    assert_eq!(checklist_of(&reply).paid & PaidTopics::HSA.0, 0, "not pre-ticked");
    // Saved as opened: nothing ticked.
    accepted(&send(&mut s, tick(reply, Checklist::NONE)));
    assert_eq!(s.draft.checklist.paid & PaidTopics::HSA.0, 0);
    // The person's own tick is saved and reopens ticked.
    let reply = open(&mut s);
    accepted(&send(&mut s, tick(reply, Checklist { paid: PaidTopics::HSA.0, ..Checklist::NONE })));
    assert_ne!(s.draft.checklist.paid & PaidTopics::HSA.0, 0);
    assert_ne!(checklist_of(&open(&mut s)).paid & PaidTopics::HSA.0, 0);
}

// The checklist's three questions are rare lists the walk's topics follow:
// each is chosen ("None of these" or the ticks), never assumed. Until first
// saved the page opens by WorkflowRequest (BeginChecklist) with nothing
// answered; "These apply" with nothing ticked is no answer; afterwards it
// reopens by EditRequest with the answers.
#[test]
fn beyond_your_documents_starts_unanswered_until_first_saved() {
    use crate::tests::cross::interview::{checklist_of, navigate, tick};
    let model = include_str!("../../core/checklist_model.rs");
    let answers = &model[model.find("pub struct ChecklistAnswers").unwrap()..];
    let answers = &answers[..answers.find("\n}\n").unwrap()];
    assert!(!answers.contains("libertas_default"), "{answers}");
    let mut d = complete(FederalStatus::Single);
    d.finished_sections.retain(|section| *section != TaxSection::Checklist);
    let mut s = Interview::new(TEST_USER, d);
    let open = |s: &mut Interview| {
        let request = navigate(&s.draft, TaxSection::Checklist);
        send(s, request)
    };
    let begin = open(&mut s);
    assert!(matches!(begin, Reply::Response(P::BeginChecklist { .. })), "{begin:?}");
    // "None of these" (each catalog's zero answer) three times is an answer;
    // the page then reopens by EditRequest.
    accepted(&send(&mut s, tick(begin, Checklist::NONE)));
    let reopened = open(&mut s);
    assert!(matches!(reopened, Reply::Edit(P::SaveChecklist { .. })), "{reopened:?}");
    assert_eq!(checklist_of(&reopened), Checklist::NONE);
}
