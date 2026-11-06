// Round 9 Smith walk: a partnership K-1 with box 12 section 179 alongside a
// commercial rental. Reg. 1.179-2(c) counts a rental toward the Form 4562
// line 11 business income limit only when it is actively conducted as a trade
// or business; the rentals page asks that once per rental, only while the
// return has a section 179 deduction.
use crate::Section179IncomeSystem;
use crate::tests::support::builders::rental;
use crate::tests::support::client::income_pages::open_topic;
use crate::tests::support::client::k1res::section179_session;
use crate::tests::*;
use alloc::string::String;

const RENTAL: i64 = 9_001;
const UNCLASSIFIED: &str =
    "ISSUE_RENTAL_FOREIGN_OR_OTHER_PROPERTY_INCOME_WITH_SECTION_179_NEEDS_ACTI_F72A67AD";

// Rent $20,000 less $2,000 expenses and $10,000 depreciation ($330,000 cost
// less $55,000 land over 27.5 years): $8,000 net.
fn add_rental(s: &mut Interview, answer: Option<bool>) {
    let mut entry = s.draft.income[0].clone();
    entry.id = RENTAL;
    entry.label = "200 Example Creek Road".into();
    entry.printed = None;
    entry.state_source = None;
    entry.refund_management = None;
    let mut data = rental(20_000, 2_000);
    data.section179_business = answer;
    entry.income = FederalIncome::Rental { data };
    s.draft.income.push(entry);
    s.draft.next_id = s.draft.next_id.max(RENTAL + 1);
}

fn limit(d: &FederalDraft) -> i64 {
    crate::section179_return::prepare(d, Section179IncomeSystem::Regular)
        .unwrap()
        .allocation
        .income
        .business_income_limit_cents
}

fn unclassified(d: &FederalDraft) -> bool {
    estimate(d).issues.iter().any(|issue| issue.display.key == UNCLASSIFIED)
}

fn rental_questions(s: &mut Interview) -> Vec<String> {
    let page = open_topic(s, TaxSection::Rentals);
    let key = alloc::format!("income/{RENTAL}/section179_business");
    let keys = match s.wire_reply(Reply::Edit(page), None) {
        Reply::Response(P::BeginDocumentQuestions { questions, .. }) => {
            questions.into_iter().map(|q| q.key).collect::<Vec<_>>()
        }
        Reply::Edit(P::SaveIncomes { questions, .. }) => {
            questions.into_iter().map(|q| q.key).collect()
        }
        other => panic!("the rentals page, got {other:?}"),
    };
    keys.into_iter().filter(|k| *k == key).collect()
}

#[test]
fn k1_section179_with_rental_asks_and_applies_active_business_answer() {
    // K-1 box 12 section 179 of $100.
    let mut s = section179_session(false, [0, 10_000], [0, 10_000]);
    let base = limit(&s.draft);
    add_rental(&mut s, None);
    // Unanswered: asked on the rentals page and still refused.
    assert!(crate::section179::rental_business_asked(&s.draft));
    assert_eq!(rental_questions(&mut s).len(), 1);
    assert!(unclassified(&s.draft));

    // Yes: the rental's $8,000 net joins the line 11 limit.
    let mut yes = s.draft.clone();
    let FederalIncome::Rental { data } = &mut yes.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.section179_business = Some(true);
    assert_eq!(limit(&yes), base + 800_000);
    assert!(!unclassified(&yes));

    // No: the rental stays out of the limit.
    let mut no = yes.clone();
    let FederalIncome::Rental { data } = &mut no.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.section179_business = Some(false);
    assert_eq!(limit(&no), base);
    assert!(!unclassified(&no));

    // A foreign rental keeps its unclassified boundary even when answered Yes.
    let mut foreign = yes.clone();
    let FederalIncome::Rental { data } = &mut foreign.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.location = crate::RentalLocation::Foreign { country: "FR".into() };
    assert!(unclassified(&foreign));
}

#[test]
fn rental_section179_question_absent_without_section179() {
    let mut s = section179_session(false, [0, 0], [0, 0]);
    add_rental(&mut s, None);
    assert!(!crate::section179::rental_business_asked(&s.draft));
    assert!(rental_questions(&mut s).is_empty());
    assert!(!unclassified(&s.draft));
}

// The Smith rental was sold in 2026. No excludes it without classifying its
// income, so it is supported whatever the rental holds; Yes would need its
// sale-year net and keeps the unclassified boundary.
#[test]
fn sold_rental_answered_no_is_supported_and_yes_is_refused() {
    let mut s = section179_session(false, [0, 10_000], [0, 10_000]);
    let base = limit(&s.draft);
    add_rental(&mut s, Some(false));
    let FederalIncome::Rental { data } = &mut s.draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.sale = crate::tests::investment::investment::sold_rental(0).sale;
    assert!(!matches!(data.sale, crate::RentalSale::No));
    assert_eq!(limit(&s.draft), base);
    assert!(!unclassified(&s.draft));

    let FederalIncome::Rental { data } = &mut s.draft.income.last_mut().unwrap().income else {
        unreachable!()
    };
    data.section179_business = Some(true);
    assert!(unclassified(&s.draft));
}

// Round 10 Smith walk (R10-02): the rental is created by its own first Rentals
// save while the K-1 section 179 is already on the return; that save's reply
// opens the questions page, which must already offer the section 179 question.
#[test]
fn first_rentals_save_offers_the_section179_business_question() {
    use crate::tests::cross::interview::{rental_page, save_rentals};
    let mut s = section179_session(false, [0, 10_000], [0, 10_000]);
    let reply = save_rentals(&mut s, vec![rental_page(100_000)]);
    let Reply::Edit(page) = reply else { panic!("the first Rentals save reopens the page") };
    let keys: Vec<String> = match s.wire_reply(Reply::Edit(page), None) {
        Reply::Response(P::BeginDocumentQuestions { questions, .. }) => {
            questions.into_iter().map(|q| q.key).collect()
        }
        Reply::Edit(P::SaveIncomes { questions, .. }) => {
            questions.into_iter().map(|q| q.key).collect()
        }
        other => panic!("the first Rentals save reopens the Rentals page, got {other:?}"),
    };
    assert!(keys.iter().any(|k| k.ends_with("/section179_business")), "{keys:?}");
}
