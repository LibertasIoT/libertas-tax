// R10-03 part A: must-answer facts are asked, never pre-answered. The W-2
// medical-account contribution year is a required strict question that opens
// before the HSA page; the other filer's HSA eligibility and a mortgage
// refund's origin block their page's save while open (D-2026-10-30-10).
use super::*;
use crate::tests::cross::interview::{
    accepted, hsa_account, hsa_employer_year_begin, hsa_page_value, mortgage_editor, open_hsa_page,
    rejected_with,
};
use crate::tests::support::client::own_questions::answer_questions;
use crate::{DocumentStrictAnswer, HealthEmployerYear};

fn w2_with_code_w(amount: i64) -> FederalDraft {
    let mut draft = complete(FederalStatus::Single);
    let entry = draft.income.iter_mut().find(|entry| entry.id == 1).unwrap();
    let FederalIncome::Wage { data: crate::WageSource::W2(crate::W2Source { form: data, .. }) } =
        &mut entry.income
    else {
        panic!("expected the accepted W-2")
    };
    data.box12.push(crate::PrintedBox12Entry { code: W2Code::W, amount: Some(amount) });
    draft
}

// The raw HSA page, opened without the questions page (the unanswered year is
// not on it).
fn raw_page(session: &mut Interview) -> P {
    let request = crate::tests::cross::interview::go_to(
        &session.draft,
        crate::tests::cross::interview::PAGE_ADJUSTMENTS,
        TaxSection::Hsa,
    );
    match send(session, request) {
        Reply::Edit(page @ P::SaveHsa { .. }) => page,
        other => panic!("expected the HSA page, got {other:?}"),
    }
}

#[test]
fn employer_year_is_asked_required_blocks_the_save_and_round_trips() {
    let mut session = Interview::new(TEST_USER, w2_with_code_w(100_000));
    // The year opens as a required question: no stored binding, no row.
    let begin = hsa_employer_year_begin(&mut session).expect("the year is asked first");
    let P::BeginDocumentQuestions { questions, .. } = &begin else { unreachable!() };
    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].key, "employer/1/hsa");
    assert_eq!(
        questions[0].allowed_answers,
        vec![DocumentStrictAnswer::EmployerYear { value: HealthEmployerYear::CurrentYear }.index()]
    );
    assert!(session.draft.adjustments.as_ref().unwrap().health_employer_w2.is_empty());
    let mut page = raw_page(&mut session);
    assert!(hsa_page_value(&page).0.employer_sources.is_empty());
    // The page does not save while the year is open.
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 50_000)];
    value.family_split = FamilySplit::NotApplicable;
    let before = session.draft.clone();
    rejected_with(send(&mut session, page), "HEALTH_EMPLOYER_YEAR_REQUIRED");
    assert_eq!(session.draft, before);
    // The questions page needs its answer.
    let empty = crate::tests::support::client::own_questions::questions_request(&begin, vec![]);
    rejected_with(
        crate::tests::support::client::own_questions::wire(&mut session, empty),
        "DOCUMENT_QUESTIONS_CHANGED",
    );
    assert_eq!(session.draft, before);
    // Answered: the binding is stored and the page opens with the answered row.
    accepted(&answer_questions(
        &mut session,
        &begin,
        vec![DocumentStrictAnswer::EmployerYear {
            value: HealthEmployerYear::Split { current: 40_000 },
        }],
    ));
    assert!(hsa_employer_year_begin(&mut session).is_none());
    let mut page = open_hsa_page(&mut session);
    assert_eq!(
        hsa_page_value(&page).0.employer_sources[0].allocation,
        HealthEmployerYear::Split { current: 40_000 }
    );
    // A current-year employer amount needs the owner's account record.
    let before = session.draft.clone();
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.family_split = FamilySplit::NotApplicable;
    rejected_with(send(&mut session, page.clone()), "HEALTH_EMPLOYER_ACCOUNT_REQUIRED");
    assert_eq!(session.draft, before);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 50_000)];
    accepted(&send(&mut session, page));
    let saved = session.draft.clone();
    assert_eq!(saved.adjustments.as_ref().unwrap().hsa[0].employer, 40_000);
    let mut restored = restart(saved.clone());
    assert!(hsa_employer_year_begin(&mut restored).is_none());
    let reopened = open_hsa_page(&mut restored);
    assert_eq!(
        hsa_page_value(&reopened).0.employer_sources[0].allocation,
        HealthEmployerYear::Split { current: 40_000 }
    );
    assert_eq!(restored.draft, saved);
}

// The questions page the endpoint opens for a save, or None when the save went on.
fn questions_after(session: &mut Interview, page: P) -> Option<P> {
    match crate::tests::support::client::own_questions::wire(session, page) {
        Reply::Response(begin @ P::BeginDocumentQuestions { .. }) => Some(begin),
        other => {
            accepted(&other);
            None
        }
    }
}

#[test]
fn other_filer_eligibility_is_asked_never_preanswered_and_round_trips() {
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 50_000)];
    value.family_split = FamilySplit::NotApplicable;
    assert!(value.other_coverage.is_none());
    // The save keeps the accounts and replies with the HSA questions page.
    let begin = questions_after(&mut session, page).expect("the eligibility is asked");
    let P::BeginDocumentQuestions { topic, questions, .. } = &begin else { unreachable!() };
    assert_eq!(*topic, TaxSection::Hsa);
    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].key, "other_coverage");
    let kind = DocumentStrictAnswer::OtherHsaEligibility {
        value: HsaEligibilityPage::FullYear { coverage: HsaCoverage::None },
    }
    .index();
    assert_eq!(questions[0].allowed_answers, vec![kind]);
    let a = session.draft.adjustments.as_ref().unwrap();
    assert_eq!(a.hsa.len(), 1);
    assert!(a.hsa_family_share.as_ref().is_none_or(|share| share.other_coverage.is_none()));
    // Reopening asks again; the page never shows an answer nobody gave.
    let reopened = crate::tests::cross::interview::hsa_other_begin(&mut session);
    assert!(reopened.is_some());
    // The page needs its answer.
    let empty = crate::tests::support::client::own_questions::questions_request(&begin, vec![]);
    rejected_with(
        crate::tests::support::client::own_questions::wire(&mut session, empty),
        "DOCUMENT_QUESTIONS_CHANGED",
    );
    // Answered: the HSA page reopens with the answered value.
    let answered = HsaEligibilityPage::FullYear { coverage: HsaCoverage::None };
    let reply = answer_questions(
        &mut session,
        &begin,
        vec![DocumentStrictAnswer::OtherHsaEligibility { value: answered.clone() }],
    );
    let Reply::Edit(P::SaveHsa { value, .. }) = &reply else { panic!("the HSA page: {reply:?}") };
    assert_eq!(value.other_coverage, Some(answered.clone()));
    let saved = session.draft.clone();
    let mut restored = restart(saved.clone());
    assert!(crate::tests::cross::interview::hsa_other_begin(&mut restored).is_none());
    let reopened = open_hsa_page(&mut restored);
    assert_eq!(hsa_page_value(&reopened).0.other_coverage, Some(answered));
    assert_eq!(restored.draft, saved);
    // Saved again, nothing is asked.
    assert!(questions_after(&mut restored, reopened).is_none());
}

#[test]
fn other_filer_eligibility_is_asked_only_when_needed_and_pruned_when_not() {
    // No contribution, no account: nothing to ask.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 0)];
    value.family_split = FamilySplit::NotApplicable;
    assert!(questions_after(&mut session, page).is_none());
    // Answered with one contributing account, then a second account: the answer
    // no longer applies and is pruned, not refused.
    let mut session = Interview::new(TEST_USER, complete(FederalStatus::Joint));
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    value.accounts = vec![hsa_account(Owner::Taxpayer, HsaCoverageYear::SelfOnly, 50_000)];
    value.family_split = FamilySplit::NotApplicable;
    value.other_coverage = Some(HsaEligibilityPage::FullYear { coverage: HsaCoverage::None });
    assert!(questions_after(&mut session, page).is_none());
    let mut page = open_hsa_page(&mut session);
    let P::SaveHsa { value, .. } = &mut page else { unreachable!() };
    assert!(value.other_coverage.is_some());
    value.accounts.push(hsa_account(Owner::Spouse, HsaCoverageYear::SelfOnly, 0));
    assert!(questions_after(&mut session, page).is_none());
    let share = session.draft.adjustments.as_ref().unwrap().hsa_family_share.as_ref();
    assert!(share.is_none_or(|share| share.other_coverage.is_none()));
}

#[test]
fn mortgage_refund_origin_is_asked_for_a_nonzero_box_4_and_round_trips() {
    use crate::tests::support::builders::native_income::{first_lender_mortgage, mortgage_source};
    use crate::tests::support::client::own_questions::wire;
    let mut d = complete(FederalStatus::Single);
    let mut saved =
        mortgage_source(first_lender_mortgage(900_000, 40_000_000, AverageBalance::WithinLimit));
    saved.id = d.next_id;
    // Box 4 is zero on this loan: nothing is asked for it.
    saved.form.box_4 = 0;
    let mut new = crate::Mortgage1098Source::printed(saved.form.clone());
    new.id = saved.id + 1;
    new.form.box_1 = 100_000;
    new.form.box_2 = 5_000_000;
    new.form.box_4 = 5_000;
    new.assigned = crate::DocumentRecipient::Taxpayer;
    new.owner_source = Some(crate::OwnerSource::Added);
    d.deductions.as_mut().unwrap().mortgage = vec![saved.clone(), new.clone()];
    d.next_id = d.next_id.max(new.id + 1);
    let mut s = Interview::new(TEST_USER, d);
    let before = s.draft.clone();
    let request = crate::tests::cross::interview::navigate(&s.draft, TaxSection::Mortgage);
    let reply = wire(&mut s, request);
    let Reply::Response(begin @ P::BeginDocumentQuestions { .. }) = reply else {
        panic!("the refund origin is asked first: {reply:?}")
    };
    let P::BeginDocumentQuestions { topic, questions, .. } = &begin else { unreachable!() };
    assert_eq!(*topic, TaxSection::Mortgage);
    assert_eq!(questions.len(), 1, "only the loan with a box 4 refund is asked");
    assert_eq!(questions[0].key, alloc::format!("mortgage/{}", new.id));
    assert_eq!(
        questions[0].allowed_answers,
        vec![
            DocumentStrictAnswer::MortgageRefundOrigin { value: MortgageRefundOrigin::ThisYear }
                .index()
        ]
    );
    assert_eq!(s.draft, before);
    // Required: no answer, no progress.
    let empty = crate::tests::support::client::own_questions::questions_request(&begin, vec![]);
    rejected_with(wire(&mut s, empty), "DOCUMENT_QUESTIONS_CHANGED");
    assert_eq!(s.draft, before);
    // Answered: stored, and the mortgage page opens with it as an answered value.
    let reply = answer_questions(
        &mut s,
        &begin,
        vec![DocumentStrictAnswer::MortgageRefundOrigin {
            value: MortgageRefundOrigin::NotDeducted,
        }],
    );
    let page = mortgage_editor(reply);
    let P::SaveMortgages { value, .. } = &page else { unreachable!() };
    assert_eq!(
        value.loans.iter().find(|loan| loan.id == new.id).unwrap().refund_origin,
        Some(MortgageRefundOrigin::NotDeducted)
    );
    assert!(value.loans.iter().find(|loan| loan.id == saved.id).unwrap().refund_origin.is_none());
    assert!(
        value
            .loans
            .iter()
            .find(|loan| loan.id == saved.id)
            .unwrap()
            .allowed_refund_origins
            .is_empty()
    );
    accepted(&send(&mut s, page));
    let stored = s.draft.clone();
    let mut restored = restart(stored.clone());
    let request = crate::tests::cross::interview::navigate(&restored.draft, TaxSection::Mortgage);
    let Reply::Edit(P::SaveMortgages { value, .. }) = wire(&mut restored, request) else {
        panic!("nothing is asked again; the answered loans reopen by EditRequest")
    };
    assert_eq!(
        value.loans.iter().find(|loan| loan.id == new.id).unwrap().refund_origin,
        Some(MortgageRefundOrigin::NotDeducted)
    );
    assert_eq!(restored.draft, stored);
}

// The answered refund origin is read-only on the Mortgage page; a correction
// reopens the questions page with the stored answer, and the save path refuses
// a page echo that differs from it.
#[test]
fn answered_mortgage_refund_origin_is_read_only_and_corrected_through_its_question() {
    use crate::tests::support::builders::native_income::{first_lender_mortgage, mortgage_source};
    use crate::tests::support::client::own_questions::wire;
    let mut d = complete(FederalStatus::Single);
    let mut zero =
        mortgage_source(first_lender_mortgage(900_000, 40_000_000, AverageBalance::WithinLimit));
    zero.id = d.next_id;
    zero.form.box_4 = 0;
    let mut refund = crate::Mortgage1098Source::printed(zero.form.clone());
    refund.id = zero.id + 1;
    refund.form.box_1 = 100_000;
    refund.form.box_2 = 5_000_000;
    refund.form.box_4 = 5_000;
    refund.assigned = crate::DocumentRecipient::Taxpayer;
    refund.owner_source = Some(crate::OwnerSource::Added);
    refund.refund_origin = Some(MortgageRefundOrigin::NotDeducted);
    refund.answered = true;
    d.deductions.as_mut().unwrap().mortgage = vec![zero.clone(), refund.clone()];
    d.next_id = d.next_id.max(refund.id + 1);
    let mut s = Interview::new(TEST_USER, d);
    let request = crate::tests::cross::interview::navigate(&s.draft, TaxSection::Mortgage);
    let Reply::Edit(P::SaveMortgages { value, document_questions, .. }) = wire(&mut s, request)
    else {
        panic!("the answered loans open by EditRequest")
    };
    // The statically read-only row offers its stored answer; box 4 zero has none.
    let row = value.loans.iter().find(|l| l.id == refund.id).unwrap();
    assert_eq!(row.refund_origin, Some(MortgageRefundOrigin::NotDeducted));
    let none = value.loans.iter().find(|l| l.id == zero.id).unwrap();
    assert!(none.refund_origin.is_none() && none.allowed_refund_origins.is_empty());
    assert_eq!(document_questions.len(), 1);
    assert_eq!(document_questions[0].topic, TaxSection::Mortgage);
    // A page echo that differs from the stored answer is refused.
    let request = crate::tests::cross::interview::navigate(&s.draft, TaxSection::Mortgage);
    let page = mortgage_editor(wire(&mut s, request));
    let mut tampered = page.clone();
    let P::SaveMortgages { value, .. } = &mut tampered else { unreachable!() };
    value.loans.iter_mut().find(|l| l.id == refund.id).unwrap().refund_origin =
        Some(MortgageRefundOrigin::ThisYear);
    let before = s.draft.clone();
    rejected_with(send(&mut s, tampered), "DOCUMENT_QUESTIONS_CHANGED");
    assert_eq!(s.draft, before);
    accepted(&send(&mut s, page));
    // The link reopens the question answered and editable.
    let open = P::OpenDocumentQuestions {
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        review: false,
        topic: TaxSection::Mortgage,
    };
    let Reply::Edit(P::SaveDocumentQuestions { topic, questions, .. }) = wire(&mut s, open) else {
        panic!("the answered question reopens by EditRequest")
    };
    assert_eq!(topic, TaxSection::Mortgage);
    assert_eq!(questions.len(), 1);
    assert_eq!(questions[0].key, alloc::format!("mortgage/{}", refund.id));
    assert_eq!(
        questions[0].answer,
        DocumentStrictAnswer::MortgageRefundOrigin { value: MortgageRefundOrigin::NotDeducted }
    );
    // A changed answer saves and shows on the Mortgage page.
    let mut changed = questions.clone();
    changed[0].answer =
        DocumentStrictAnswer::MortgageRefundOrigin { value: MortgageRefundOrigin::ThisYear };
    let request = P::SaveDocumentQuestions {
        user: TEST_USER,
        cookie: s.draft.cookie.clone(),
        revision: s.draft.revision,
        page: crate::tests::cross::interview::PAGE_ADJUSTMENTS,
        previous: crate::tests::cross::interview::PAGE_ADJUSTMENTS,
        review: false,
        estimate: None,
        progress: Vec::new(),
        topic: TaxSection::Mortgage,
        questions: changed,
        situations: crate::DocumentSituations::NoneOfThese,
        allowed_situations: Vec::new(),
        situation_answers: vec![0],
        situation_documents: Vec::new(),
        disaster_choices: Vec::new(),
        inherited_iras: Vec::new(),
        businesses: Vec::new(),
        people: Vec::new(),
        inherited_roth_iras: Vec::new(),
        wage_scorporations: Vec::new(),
    };
    let P::SaveMortgages { value, .. } = mortgage_editor(wire(&mut s, request)) else {
        unreachable!()
    };
    assert_eq!(
        value.loans.iter().find(|l| l.id == refund.id).unwrap().refund_origin,
        Some(MortgageRefundOrigin::ThisYear)
    );
    let stored = s.draft.deductions.as_ref().unwrap().mortgage.iter().find(|m| m.id == refund.id);
    assert_eq!(stored.unwrap().refund_origin, Some(MortgageRefundOrigin::ThisYear));
}
